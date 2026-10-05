//! The language gate's "one cross-language tool call"
//! (`ENGINE ARCHITECTURE AND TECHNOLOGY DECISION.md` §17, ADR-0034).
//!
//! One tool, written in a language other than the core's, called across the
//! boundary layer the language map gives tools, and timed. The tool is
//! `tools/catalog-digest/catalog_digest.py`: Python, the map's research and
//! automation language, standard library only. It computes what
//! `content/first-generation/CATALOG.md` records for every albedo — its size,
//! the FNV-1a 64 of its PNG's bytes, and the header that says it is 16×16
//! RGBA8 — and answers over **IPC**: a versioned, line-delimited contract on
//! the child process's stdin and stdout (`tools/catalog-digest/README.md`),
//! through `std::process` and nothing else. No crate, no `unsafe`.
//!
//! # Correctness first
//!
//! As everywhere in this harness, nothing is timed until the answer is
//! known to be right. Rust computes the digest itself
//! ([`digest_bytes`], over `nexora_foundation::hashing`) from the bytes it
//! wrote, and the tool's answer must equal it before a cold call or a round
//! trip is timed, and on every one that is. A refusal must also cross as a
//! structured answer (`err unreadable`), not as a traceback: the language
//! map's rule for errors at a boundary.
//!
//! # What is measured
//!
//! * `tool.cold_call` — start the interpreter, one request, exit. What a
//!   tool run once costs, and why a per-tick call is out of the question.
//! * `tool.warm_roundtrip` — one request and its answer over a tool that is
//!   already running. The cheapest the IPC boundary gets.
//! * `tool.python_work` — of that round trip, the tool's own time for the
//!   work, by its clock. The difference is the boundary.
//! * `tool.digest_in_rust` — the same work in this process, no boundary.
//!
//! Read against `ffi.empty_crossing` (about 1.2 ns, baseline Appendix D):
//! three rungs of one ladder, in-process call, IPC to a running tool, a tool
//! started for the call. These are the evidence for
//! `NEXORA LANGUAGE AND FFI BOUNDARY.md`'s rule that hot loops do not cross a
//! language boundary.
//!
//! # No interpreter is a gap, not a failure
//!
//! Without a Python 3.8+ interpreter (`python3`, then `python`) the stage is
//! declared not measured, with the reason, as the RHI stage is without an
//! adapter. `NEXORA_PYTHON=none` declares it; `NEXORA_PYTHON=<program>` names
//! the one interpreter to use. An interpreter that answers and then gets the
//! digest wrong is an error: a wrong answer, not a missing one.

use std::cell::RefCell;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::hashing::{crc32, fnv1a64};

use crate::{consume, measure, record_bytes, Budget, Measurement, Unit};

/// The protocol this client speaks: the first token of the tool's banner.
pub const PROTOCOL: &str = "nexora-tool/1";

/// The tool's name, as its banner gives it.
pub const TOOL_NAME: &str = "catalog-digest";

/// The tool's source, embedded at build time so the benchmark binary carries
/// the exact tool it was built with and runs from any directory.
pub const TOOL_SOURCE: &str = include_str!("../../../tools/catalog-digest/catalog_digest.py");

/// The interpreters tried, in order, when `NEXORA_PYTHON` names none.
pub const CANDIDATES: [&str; 2] = ["python3", "python"];

/// How every interpreter is started: isolated from `PYTHON*` variables and
/// the user's site (`-I`), with no site-packages at all (`-S`, which makes
/// "standard library only" structural), and writing no bytecode (`-B`).
pub const INTERPRETER_FLAGS: [&str; 3] = ["-I", "-S", "-B"];

/// The oldest Python the tool is written for.
pub const MINIMUM_PYTHON: (u32, u32) = (3, 8);

/// Why the stage did not run when no interpreter answered.
pub const NO_INTERPRETER: &str =
    "no Python 3.8+ interpreter answered: tried `python3`, then `python`";

/// Why the stage did not run when the machine says it has no Python.
pub const DECLARED_NO_PYTHON: &str = "NEXORA_PYTHON=none: this machine declares no Python";

/// Why the stage did not run when `NEXORA_PYTHON` named something else.
pub const NAMED_NOT_PYTHON: &str =
    "NEXORA_PYTHON names a program that is not a Python 3.8+ interpreter";

/// The first visual generation's shape, which the fixture has (CATALOG.md).
pub const FIRST_GENERATION_IHDR: Ihdr = Ihdr {
    width: 16,
    height: 16,
    depth: 8,
    color: 6,
};

/// Round trips timed together in one sample: one is tens of microseconds.
const WARM_ITERATIONS: u32 = 100;

/// Same-work digests in Rust timed together in one sample.
const RUST_ITERATIONS: u32 = 200;

// --- the contract ------------------------------------------------------------

/// A PNG's header fields, as the IHDR chunk states them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ihdr {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Bits per channel.
    pub depth: u8,
    /// PNG colour type: 6 is RGBA.
    pub color: u8,
}

/// The catalog's digest columns for one file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Digest {
    /// FNV-1a 64 of every byte of the file.
    pub fnv1a64: u64,
    /// The file's size.
    pub bytes: u64,
    /// The PNG header, or `None` when the file is not a PNG with a valid one.
    pub ihdr: Option<Ihdr>,
}

/// What the tool said about one request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// The digest, and the tool's own time for computing it.
    Digest {
        /// The digest.
        digest: Digest,
        /// Nanoseconds the tool spent on the work, by its own clock.
        work_ns: u64,
    },
    /// A structured refusal.
    Refused {
        /// A stable, machine-readable code: `bad-request`, `unknown-verb`,
        /// `unreadable`.
        code: String,
        /// One line of explanation, for a person.
        message: String,
    },
}

/// One response line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    /// The request it answers; `None` when the request was too malformed to
    /// carry an id the tool could echo.
    pub id: Option<u64>,
    /// What it said.
    pub answer: Answer,
}

/// Format the request line `<id> digest <path>\n`.
///
/// # Errors
///
/// The path is not UTF-8, is empty, or holds a line break: none of those can
/// be framed as one line of the contract.
pub fn format_request(id: u64, path: &Path) -> Result<String> {
    let text = path
        .to_str()
        .ok_or_else(|| malformed("a request path must be UTF-8"))?;
    if text.is_empty() || text.contains(['\n', '\r']) {
        return Err(
            malformed("a request path must be one non-empty line").with_context("path", text)
        );
    }
    Ok(format!("{id} digest {text}\n"))
}

/// Check the tool's first line and return the tool's version.
///
/// # Errors
///
/// The banner is not `nexora-tool/1 catalog-digest/<version>`: another
/// protocol version, another tool, or not a banner at all.
pub fn parse_banner(line: &str) -> Result<u32> {
    let line = strip_terminator(line);
    let mut tokens = line.split(' ');
    let (Some(protocol), Some(tool), None) = (tokens.next(), tokens.next(), tokens.next()) else {
        return Err(malformed("the banner is not two tokens").with_context("line", clip(line)));
    };
    if protocol != PROTOCOL {
        return Err(
            malformed("the tool speaks another protocol, or another version of it")
                .with_context("expected", PROTOCOL)
                .with_context("line", clip(line)),
        );
    }
    let version = tool
        .strip_prefix(TOOL_NAME)
        .and_then(|rest| rest.strip_prefix('/'))
        .and_then(decimal::<u32>)
        .ok_or_else(|| {
            malformed("the banner names another tool").with_context("line", clip(line))
        })?;
    Ok(version)
}

/// Parse one response line.
///
/// `<id> ok fnv1a64=0x<16 hex> bytes=<n> ihdr=<w,h,depth,colour|none> work_ns=<n>`,
/// or `<id> err <code> <message>`; `<id>` is `-` when the request carried
/// none. Every key of `ok` is required, once, and nothing else is accepted:
/// a field this version does not define is another version.
///
/// # Errors
///
/// The line does not follow the grammar above.
pub fn parse_response(line: &str) -> Result<Response> {
    let line = strip_terminator(line);
    let bad = |message: &'static str| malformed(message).with_context("line", clip(line));
    if line.contains(['\n', '\r']) {
        return Err(bad("a response is one line"));
    }
    let mut head = line.splitn(3, ' ');
    let (Some(id), Some(status)) = (head.next(), head.next()) else {
        return Err(bad("a response is an id, a status and its fields"));
    };
    let id = match id {
        "-" => None,
        digits => Some(decimal::<u64>(digits).ok_or_else(|| bad("the id is not decimal"))?),
    };
    let rest = head.next().unwrap_or("");

    let answer = match status {
        "err" => {
            let (code, message) = rest
                .split_once(' ')
                .ok_or_else(|| bad("a refusal is a code and a message"))?;
            if code.is_empty()
                || !code
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
                || message.is_empty()
            {
                return Err(bad(
                    "a refusal's code is lowercase words and its message is not empty",
                ));
            }
            Answer::Refused {
                code: code.to_owned(),
                message: message.to_owned(),
            }
        }
        "ok" => {
            let (mut fnv, mut bytes, mut ihdr, mut work) = (None, None, None, None);
            for field in rest.split(' ') {
                let (key, value) = field
                    .split_once('=')
                    .ok_or_else(|| bad("a field is key=value"))?;
                let slot_was_empty = match key {
                    "fnv1a64" => fnv
                        .replace(
                            parse_fnv(value).ok_or_else(|| {
                                bad("fnv1a64 is 0x and sixteen lowercase hex digits")
                            })?,
                        )
                        .is_none(),
                    "bytes" => bytes
                        .replace(decimal::<u64>(value).ok_or_else(|| bad("bytes is decimal"))?)
                        .is_none(),
                    "ihdr" => ihdr
                        .replace(
                            parse_ihdr(value)
                                .ok_or_else(|| bad("ihdr is none or width,height,depth,colour"))?,
                        )
                        .is_none(),
                    "work_ns" => work
                        .replace(decimal::<u64>(value).ok_or_else(|| bad("work_ns is decimal"))?)
                        .is_none(),
                    _ => return Err(bad("a field this protocol version does not define")),
                };
                if !slot_was_empty {
                    return Err(bad("a field appears twice"));
                }
            }
            let (Some(fnv1a64), Some(bytes), Some(ihdr), Some(work_ns)) = (fnv, bytes, ihdr, work)
            else {
                return Err(bad("a digest needs fnv1a64, bytes, ihdr and work_ns"));
            };
            Answer::Digest {
                digest: Digest {
                    fnv1a64,
                    bytes,
                    ihdr,
                },
                work_ns,
            }
        }
        _ => return Err(bad("the status is neither ok nor err")),
    };
    Ok(Response { id, answer })
}

/// A decimal number with nothing else in it: no sign, no space, no `+`.
fn decimal<T: std::str::FromStr>(text: &str) -> Option<T> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn parse_fnv(value: &str) -> Option<u64> {
    let hex = value.strip_prefix("0x")?;
    if hex.len() != 16
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return None;
    }
    u64::from_str_radix(hex, 16).ok()
}

/// `none` is `Some(None)`: a valid answer that the file is not a PNG.
fn parse_ihdr(value: &str) -> Option<Option<Ihdr>> {
    if value == "none" {
        return Some(None);
    }
    let mut parts = value.split(',');
    let header = Ihdr {
        width: decimal(parts.next()?)?,
        height: decimal(parts.next()?)?,
        depth: decimal(parts.next()?)?,
        color: decimal(parts.next()?)?,
    };
    parts.next().is_none().then_some(Some(header))
}

fn strip_terminator(line: &str) -> &str {
    let line = line.strip_suffix('\n').unwrap_or(line);
    line.strip_suffix('\r').unwrap_or(line)
}

fn clip(line: &str) -> String {
    line.chars().take(160).collect()
}

// --- Rust's own answer ---------------------------------------------------------

/// The digest of a file's bytes, computed by Rust: the answer the tool's is
/// checked against.
#[must_use]
pub fn digest_bytes(data: &[u8]) -> Digest {
    Digest {
        fnv1a64: fnv1a64(data),
        bytes: data.len() as u64,
        ihdr: read_ihdr(data),
    }
}

/// The same work the tool does for one request, in this process: read the
/// file, then [`digest_bytes`].
///
/// # Errors
///
/// The file cannot be read.
pub fn digest_file(path: &Path) -> Result<Digest> {
    let data =
        std::fs::read(path).map_err(|cause| io_error("could not read a digested file", &cause))?;
    Ok(digest_bytes(&data))
}

/// The IHDR a PNG states, under the tool's rule: the signature, then a
/// thirteen-byte IHDR whose CRC matches. Anything else is `None`.
#[must_use]
pub fn read_ihdr(data: &[u8]) -> Option<Ihdr> {
    let be32 = |at: usize| -> Option<u32> {
        Some(u32::from_be_bytes(data.get(at..at + 4)?.try_into().ok()?))
    };
    if data.get(..8)? != PNG_SIGNATURE || be32(8)? != 13 || data.get(12..16)? != b"IHDR" {
        return None;
    }
    if crc32(data.get(12..29)?) != be32(29)? {
        return None;
    }
    Some(Ihdr {
        width: be32(16)?,
        height: be32(20)?,
        depth: data[24],
        color: data[25],
    })
}

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// The asset every call digests: a 16×16 RGBA8 PNG, the first generation's
/// shape, its texels drawn from a four-colour stone palette by a seeded RNG
/// so it compresses like the catalog's albedos do (a few hundred bytes).
///
/// Written by the engine's own deflate; the engine's own decoder reads it
/// back in this module's tests.
#[must_use]
pub fn fixture_png() -> Vec<u8> {
    const PALETTE: [[u8; 4]; 4] = [
        [0x8a, 0x84, 0x7c, 0xff],
        [0x6f, 0x6a, 0x63, 0xff],
        [0xa3, 0x9e, 0x95, 0xff],
        [0x58, 0x54, 0x4f, 0xff],
    ];
    /// PNG filter type 0: the scanline is stored as it is.
    const FILTER_NONE: u8 = 0;
    let mut rng = nexora_foundation::rng::Rng::from_seed(0x0C47_A106);
    let raw: Vec<u8> = (0..16)
        .flat_map(|_| {
            let mut scanline = vec![FILTER_NONE];
            for _ in 0..16 {
                scanline.extend_from_slice(&PALETTE[rng.next_below(4) as usize]);
            }
            scanline
        })
        .collect();
    let mut zlib = vec![0x78, 0x01];
    zlib.extend(nexora_foundation::deflate::deflate(&raw));
    zlib.extend(adler32(&raw).to_be_bytes());

    let header = FIRST_GENERATION_IHDR;
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend(header.width.to_be_bytes());
    ihdr.extend(header.height.to_be_bytes());
    ihdr.extend([header.depth, header.color, 0, 0, 0]);

    let mut png = PNG_SIGNATURE.to_vec();
    for (kind, payload) in [
        (b"IHDR", &ihdr[..]),
        (b"IDAT", &zlib[..]),
        (b"IEND", &[][..]),
    ] {
        png.extend((payload.len() as u32).to_be_bytes());
        let start = png.len();
        png.extend(kind);
        png.extend(payload);
        let crc = crc32(&png[start..]);
        png.extend(crc.to_be_bytes());
    }
    png
}

/// Adler-32, as zlib defines it.
fn adler32(data: &[u8]) -> u32 {
    const MODULUS: u32 = 65_521;
    let (mut low, mut high) = (1u32, 0u32);
    for &byte in data {
        low = (low + u32::from(byte)) % MODULUS;
        high = (high + low) % MODULUS;
    }
    (high << 16) | low
}

// --- the interpreter -------------------------------------------------------------

/// A Python interpreter that answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interpreter {
    /// The program started, as named: `python3`, `python`, or a path.
    pub program: String,
    /// Its version, major, minor and micro.
    pub version: (u32, u32, u32),
    /// `sys.implementation.name`: `cpython`, `pypy`, ...
    pub implementation: String,
}

impl Interpreter {
    /// One line for the report's environment.
    #[must_use]
    pub fn describe(&self) -> String {
        let (major, minor, micro) = self.version;
        format!(
            "Python {major}.{minor}.{micro} ({}, via {})",
            self.implementation, self.program
        )
    }
}

/// What the probe asks: the version and the implementation, on one line.
const PROBE: &str =
    "import sys; v = sys.version_info; print(v[0], v[1], v[2], sys.implementation.name)";

/// Ask `program` whether it is a Python 3.8+ interpreter, started the way the
/// tool will be. A program that is missing, fails, or prints anything else
/// (a Python 2 rejects `-I`; Windows' store alias for an absent Python exits
/// non-zero) is not one.
#[must_use]
pub fn probe(program: &str) -> Option<Interpreter> {
    let output = Command::new(program)
        .args(INTERPRETER_FLAGS)
        .args(["-c", PROBE])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_probe(program, std::str::from_utf8(&output.stdout).ok()?)
}

fn parse_probe(program: &str, text: &str) -> Option<Interpreter> {
    let mut tokens = text.split_whitespace();
    let major = decimal(tokens.next()?)?;
    let minor = decimal(tokens.next()?)?;
    let micro = decimal(tokens.next()?)?;
    let implementation = tokens.next()?.to_owned();
    if tokens.next().is_some() || major != MINIMUM_PYTHON.0 || minor < MINIMUM_PYTHON.1 {
        return None;
    }
    Some(Interpreter {
        program: program.to_owned(),
        version: (major, minor, micro),
        implementation,
    })
}

/// The first of `candidates` that is a Python 3.8+ interpreter.
#[must_use]
pub fn find_interpreter(candidates: &[&str]) -> Option<Interpreter> {
    candidates.iter().find_map(|program| probe(program))
}

fn spawn(interpreter: &Interpreter, script: &Path) -> Result<Child> {
    Command::new(&interpreter.program)
        .args(INTERPRETER_FLAGS)
        .arg(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|cause| {
            io_error("could not start the tool", &cause)
                .with_context("program", interpreter.program.clone())
        })
}

// --- the two ways to call it -------------------------------------------------------

/// A tool run once: start it, check its banner, send one request, close its
/// input, and wait for it to exit with nothing more said.
///
/// # Errors
///
/// The tool could not start, exited unsuccessfully, broke the contract, or
/// answered another request than the one sent. A refusal is not an error
/// here: it is returned, and the caller decides whether it was expected.
pub fn cold_call(interpreter: &Interpreter, script: &Path, path: &Path) -> Result<Response> {
    let request = format_request(1, path)?;
    let mut child = spawn(interpreter, script)?;
    let wrote = match child.stdin.take() {
        // Dropped at the end of this arm: the end of input ends the session.
        Some(mut stdin) => stdin.write_all(request.as_bytes()),
        None => Ok(()),
    };
    let output = child
        .wait_with_output()
        .map_err(|cause| io_error("the tool could not be waited for", &cause))?;
    if !output.status.success() {
        return Err(wrong("the tool exited unsuccessfully")
            .with_context("status", output.status.to_string())
            .with_context("stderr", clip(&String::from_utf8_lossy(&output.stderr))));
    }
    wrote.map_err(|cause| io_error("could not send the request", &cause))?;
    let text = String::from_utf8(output.stdout)
        .map_err(|_| malformed("the tool's output is not UTF-8"))?;
    let mut lines = text.split_terminator('\n');
    parse_banner(lines.next().unwrap_or(""))?;
    let response = parse_response(
        lines
            .next()
            .ok_or_else(|| wrong("the tool exited without answering"))?,
    )?;
    if lines.next().is_some() {
        return Err(wrong("the tool said more than it was asked"));
    }
    if response.id != Some(1) {
        return Err(wrong("the tool answered another request than the one sent"));
    }
    Ok(response)
}

/// A tool that stays running between requests.
#[derive(Debug)]
pub struct Session {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    last_id: u64,
    line: String,
}

impl Session {
    /// Start the tool and check its banner.
    ///
    /// # Errors
    ///
    /// The tool could not start, or its first line is not this protocol's
    /// banner.
    pub fn start(interpreter: &Interpreter, script: &Path) -> Result<Self> {
        let mut child = spawn(interpreter, script)?;
        let (Some(stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(wrong("the tool's pipes were not opened"));
        };
        let mut session = Self {
            child,
            stdin: Some(stdin),
            stdout: BufReader::new(stdout),
            last_id: 0,
            line: String::new(),
        };
        let banner = session.read_line();
        if let Err(error) = banner.and_then(|line| parse_banner(&line)) {
            return Err(error.with_context("stderr", session.abandon()));
        }
        Ok(session)
    }

    /// One request and its answer.
    ///
    /// # Errors
    ///
    /// The pipe broke, the answer broke the contract, or it answered another
    /// request than the one sent.
    pub fn digest(&mut self, path: &Path) -> Result<Response> {
        self.last_id += 1;
        let request = format_request(self.last_id, path)?;
        self.stdin
            .as_mut()
            .ok_or_else(|| wrong("the session has ended"))?
            .write_all(request.as_bytes())
            .map_err(|cause| io_error("could not send the request", &cause))?;
        let line = self.read_line()?;
        let response = parse_response(&line)?;
        if response.id != Some(self.last_id) {
            return Err(wrong("the tool answered another request than the one sent")
                .with_context("sent", self.last_id.to_string())
                .with_context("line", clip(&line)));
        }
        Ok(response)
    }

    /// Close the tool's input and require that it exits cleanly, having said
    /// nothing it was not asked.
    ///
    /// # Errors
    ///
    /// The tool wrote more, or exited unsuccessfully.
    pub fn finish(mut self) -> Result<()> {
        drop(self.stdin.take());
        let mut rest = String::new();
        let drained = self.stdout.read_to_string(&mut rest);
        let status = self
            .child
            .wait()
            .map_err(|cause| io_error("the tool could not be waited for", &cause))?;
        drained.map_err(|cause| io_error("could not read the tool's output", &cause))?;
        if !rest.is_empty() {
            return Err(
                wrong("the tool said more than it was asked").with_context("line", clip(&rest))
            );
        }
        if !status.success() {
            return Err(wrong("the tool exited unsuccessfully")
                .with_context("status", status.to_string())
                .with_context("stderr", self.abandon()));
        }
        Ok(())
    }

    fn read_line(&mut self) -> Result<String> {
        self.line.clear();
        let read = self
            .stdout
            .read_line(&mut self.line)
            .map_err(|cause| io_error("could not read the tool's answer", &cause))?;
        if read == 0 {
            return Err(wrong("the tool closed its output before answering"));
        }
        Ok(std::mem::take(&mut self.line))
    }

    /// Stop the tool and return what it wrote to stderr, for a diagnosis.
    fn abandon(&mut self) -> String {
        drop(self.stdin.take());
        let _ = self.child.kill();
        let _ = self.child.wait();
        let mut stderr = String::new();
        if let Some(mut pipe) = self.child.stderr.take() {
            let _ = pipe.read_to_string(&mut stderr);
        }
        clip(stderr.trim())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // Never leave an interpreter behind, whatever ended the session.
        if let Ok(None) = self.child.try_wait() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

// --- the stage -----------------------------------------------------------------

/// What the tool-call stage found on this machine.
#[derive(Debug)]
pub struct ToolStage {
    /// The measurements, empty when nothing was measured.
    pub measurements: Vec<Measurement>,
    /// The interpreter that ran the tool, as [`Interpreter::describe`] says.
    pub interpreter: Option<String>,
    /// Why nothing was measured, when nothing was.
    pub gap: Option<&'static str>,
}

impl ToolStage {
    fn missing(reason: &'static str) -> Self {
        Self {
            measurements: Vec::new(),
            interpreter: None,
            gap: Some(reason),
        }
    }
}

/// Find an interpreter, check the tool's answers, and time the call.
///
/// `scratch` receives the tool's source and the PNG it digests.
///
/// # Errors
///
/// The scratch directory cannot be written, or an interpreter answered and
/// then the tool broke the contract or got a digest wrong. No interpreter is
/// a gap, not an error.
pub fn tool_call(budget: Budget, scratch: &Path) -> Result<ToolStage> {
    let interpreter = match std::env::var("NEXORA_PYTHON") {
        Ok(named) if named == "none" => return Ok(ToolStage::missing(DECLARED_NO_PYTHON)),
        Ok(named) if !named.is_empty() => match probe(&named) {
            Some(interpreter) => interpreter,
            None => return Ok(ToolStage::missing(NAMED_NOT_PYTHON)),
        },
        _ => match find_interpreter(&CANDIDATES) {
            Some(interpreter) => interpreter,
            None => return Ok(ToolStage::missing(NO_INTERPRETER)),
        },
    };
    measure_with(budget, scratch, &interpreter)
}

fn measure_with(budget: Budget, scratch: &Path, interpreter: &Interpreter) -> Result<ToolStage> {
    let dir = scratch.join("tool-call");
    let script = dir.join("catalog_digest.py");
    let asset = dir.join("albedo.png");
    let absent = dir.join("no-such-albedo.png");
    let fixture = fixture_png();
    std::fs::create_dir_all(&dir)
        .and_then(|()| std::fs::write(&script, TOOL_SOURCE))
        .and_then(|()| std::fs::write(&asset, &fixture))
        .map_err(|cause| io_error("could not write the tool and its asset", &cause))?;
    if absent.exists() {
        std::fs::remove_file(&absent)
            .map_err(|cause| io_error("could not clear the absent asset's path", &cause))?;
    }

    // Rust's answer, from the bytes it wrote; the file must read back as it.
    let expected = digest_bytes(&fixture);
    if expected.ihdr != Some(FIRST_GENERATION_IHDR) || digest_file(&asset)? != expected {
        return Err(wrong(
            "the fixture is not the 16x16 RGBA8 PNG that was written",
        ));
    }
    let agrees = |response: &Response| matches!(response.answer, Answer::Digest { digest, .. } if digest == expected);

    // Correctness first: a tool run once, a running tool's first answer, and
    // a refusal that crosses as a structured answer.
    let first = cold_call(interpreter, &script, &asset)?;
    if !agrees(&first) {
        return Err(disagreement(&first, &expected));
    }
    let mut session = Session::start(interpreter, &script)?;
    let warm = session.digest(&asset)?;
    if !agrees(&warm) {
        return Err(disagreement(&warm, &expected));
    }
    let refused = session.digest(&absent)?;
    if !matches!(&refused.answer, Answer::Refused { code, .. } if code == "unreadable") {
        return Err(
            wrong("a file that does not exist was not refused as unreadable")
                .with_context("answer", format!("{:?}", refused.answer)),
        );
    }

    let failure = RefCell::new(None::<Error>);
    let fail = |error: Error| {
        failure.borrow_mut().get_or_insert(error);
    };

    let mut out = vec![measure(
        "tool.cold_call",
        "Start the Python tool, check its banner, one digest request answered and checked, exit: a tool run once",
        Budget {
            warmup_iterations: 1,
            iterations_per_sample: 1,
            ..budget
        },
        || match cold_call(interpreter, &script, &asset) {
            Ok(response) if agrees(&response) => {}
            Ok(response) => fail(disagreement(&response, &expected)),
            Err(error) => fail(error),
        },
    )];

    let warm_budget = Budget {
        iterations_per_sample: WARM_ITERATIONS,
        ..budget
    };
    let mut work = Vec::new();
    out.push(measure(
        "tool.warm_roundtrip",
        "One digest request and its checked answer over a running tool's stdin/stdout (IPC); against tool.digest_in_rust, and ffi.empty_crossing at ~1.2 ns (Appendix D)",
        warm_budget,
        || match session.digest(&asset) {
            Ok(Response {
                answer: Answer::Digest { digest, work_ns },
                ..
            }) if digest == expected => work.push(work_ns),
            Ok(response) => fail(disagreement(&response, &expected)),
            Err(error) => fail(error),
        },
    ));
    session.finish()?;
    if let Some(error) = failure.into_inner() {
        return Err(error);
    }

    // The tool's own clock, averaged over the round trips of each sample, so
    // its samples line up with the round trip's.
    let per_sample = warm_budget.iterations_per_sample.max(1) as usize;
    let samples: Vec<f64> = work
        .get(warm_budget.warmup_iterations as usize..)
        .unwrap_or(&[])
        .chunks(per_sample)
        .map(|chunk| chunk.iter().sum::<u64>() as f64 / chunk.len() as f64)
        .collect();
    out.push(Measurement {
        name: "tool.python_work",
        note: "Of each round trip, the tool's own time for the digest in Python, by its clock: the rest is the boundary",
        unit: Unit::TimePerOp,
        samples,
        bytes_per_op: None,
    });

    let wrong_in_rust = RefCell::new(None::<Error>);
    out.push(measure(
        "tool.digest_in_rust",
        "The same digest in this process, in Rust: read the 16x16 PNG, FNV-1a 64 and its IHDR, no boundary crossed",
        Budget {
            iterations_per_sample: RUST_ITERATIONS,
            ..budget
        },
        || match digest_file(&asset) {
            Ok(digest) if digest == expected => {
                consume(digest);
            }
            Ok(_) => {
                wrong_in_rust
                    .borrow_mut()
                    .get_or_insert(wrong("Rust's own digest changed between reads"));
            }
            Err(error) => {
                wrong_in_rust.borrow_mut().get_or_insert(error);
            }
        },
    ));
    if let Some(error) = wrong_in_rust.into_inner() {
        return Err(error);
    }
    out.push(record_bytes(
        "tool.payload_bytes",
        "Bytes of the 16x16 RGBA8 PNG every call digests: the catalog's size column",
        expected.bytes,
    ));

    Ok(ToolStage {
        measurements: out,
        interpreter: Some(interpreter.describe()),
        gap: None,
    })
}

fn disagreement(response: &Response, expected: &Digest) -> Error {
    wrong("the tool's answer is not Rust's")
        .with_context("expected", format!("{expected:?}"))
        .with_context("answer", format!("{:?}", response.answer))
}

fn wrong(message: &'static str) -> Error {
    Error::new(Domain::Content, "benchmark-tool", message).with_recovery(Recovery::Manual)
}

fn malformed(message: &'static str) -> Error {
    Error::new(Domain::Content, "benchmark-tool", message).with_recovery(Recovery::Reject)
}

fn io_error(message: &'static str, cause: &std::io::Error) -> Error {
    wrong(message).with_context("cause", cause.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINE: &str = "7 ok fnv1a64=0xa2817db906db5171 bytes=344 ihdr=16,16,8,6 work_ns=41200\n";

    #[test]
    fn a_request_is_one_line_with_its_id_verb_and_path() {
        let path = Path::new("C:\\scratch dir\\albedo.png");
        assert_eq!(
            format_request(12, path).expect("frames"),
            "12 digest C:\\scratch dir\\albedo.png\n"
        );
        // A path that would break the framing is refused, not sent.
        for broken in ["", "a\nb", "a\rb"] {
            assert!(format_request(1, Path::new(broken)).is_err(), "{broken:?}");
        }
    }

    #[test]
    fn the_banner_names_the_protocol_version_and_the_tool() {
        assert_eq!(
            parse_banner("nexora-tool/1 catalog-digest/1\n").expect("ok"),
            1
        );
        assert_eq!(
            parse_banner("nexora-tool/1 catalog-digest/3\r\n").expect("ok"),
            3
        );
        for wrong in [
            "",
            "nexora-tool/2 catalog-digest/1",
            "nexora-tool/1 texture-forge/1",
            "nexora-tool/1 catalog-digest",
            "nexora-tool/1 catalog-digest/x",
            "nexora-tool/1 catalog-digest/1 extra",
            "Traceback (most recent call last):",
        ] {
            assert!(parse_banner(wrong).is_err(), "{wrong:?}");
        }
    }

    #[test]
    fn a_digest_answer_parses_every_field() {
        let response = parse_response(LINE).expect("parses");
        assert_eq!(response.id, Some(7));
        assert_eq!(
            response.answer,
            Answer::Digest {
                digest: Digest {
                    fnv1a64: 0xa281_7db9_06db_5171,
                    bytes: 344,
                    ihdr: Some(FIRST_GENERATION_IHDR),
                },
                work_ns: 41_200,
            }
        );
        // Fields in another order are the same answer; `none` is an answer.
        let reordered =
            parse_response("8 ok work_ns=1 ihdr=none bytes=6 fnv1a64=0x85944171f73967e8")
                .expect("parses");
        assert_eq!(
            reordered.answer,
            Answer::Digest {
                digest: Digest {
                    fnv1a64: 0x8594_4171_f739_67e8,
                    bytes: 6,
                    ihdr: None,
                },
                work_ns: 1,
            }
        );
    }

    #[test]
    fn a_refusal_parses_with_its_code_and_message() {
        let response = parse_response("3 err unreadable No such file or directory").expect("ok");
        assert_eq!(response.id, Some(3));
        assert_eq!(
            response.answer,
            Answer::Refused {
                code: "unreadable".into(),
                message: "No such file or directory".into(),
            }
        );
        let anonymous =
            parse_response("- err bad-request a request begins with a decimal id").expect("ok");
        assert_eq!(anonymous.id, None);
    }

    #[test]
    fn anything_off_the_grammar_is_refused_by_name() {
        for line in [
            "",
            "7",
            "7 maybe fnv1a64=0x0000000000000000",
            "+7 ok fnv1a64=0xa2817db906db5171 bytes=344 ihdr=none work_ns=1",
            "x ok fnv1a64=0xa2817db906db5171 bytes=344 ihdr=none work_ns=1",
            // A missing, a repeated, an unknown and a malformed field.
            "7 ok fnv1a64=0xa2817db906db5171 bytes=344 ihdr=none",
            "7 ok fnv1a64=0xa2817db906db5171 bytes=344 bytes=344 ihdr=none work_ns=1",
            "7 ok fnv1a64=0xa2817db906db5171 bytes=344 ihdr=none work_ns=1 colour=6",
            "7 ok fnv1a64=0xA2817DB906DB5171 bytes=344 ihdr=none work_ns=1",
            "7 ok fnv1a64=0xa2817db906db517 bytes=344 ihdr=none work_ns=1",
            "7 ok fnv1a64=0xa2817db906db5171 bytes=-1 ihdr=none work_ns=1",
            "7 ok fnv1a64=0xa2817db906db5171 bytes=344 ihdr=16,16,8 work_ns=1",
            "7 ok fnv1a64=0xa2817db906db5171 bytes=344 ihdr=16,16,8,6,0 work_ns=1",
            "7 ok fnv1a64=0xa2817db906db5171 bytes=344 ihdr=16,16,256,6 work_ns=1",
            "7 ok fnv1a64=0xa2817db906db5171  bytes=344 ihdr=none work_ns=1",
            // Refusals need a lowercase code and a message.
            "7 err unreadable",
            "7 err Unreadable gone",
            "7 err  gone",
            // One line, no more.
            "7 ok fnv1a64=0xa2817db906db5171 bytes=344\nihdr=none work_ns=1",
        ] {
            assert!(parse_response(line).is_err(), "{line:?} parsed");
        }
    }

    #[test]
    fn rust_computes_the_catalogs_columns() {
        // FNV-1a 64 reference vectors from the published specification.
        assert_eq!(digest_bytes(b"").fnv1a64, 0xcbf2_9ce4_8422_2325);
        assert_eq!(digest_bytes(b"a").fnv1a64, 0xaf63_dc4c_8601_ec8c);
        assert_eq!(digest_bytes(b"foobar").fnv1a64, 0x8594_4171_f739_67e8);
        assert_eq!(digest_bytes(b"foobar").ihdr, None);

        let png = fixture_png();
        let digest = digest_bytes(&png);
        assert_eq!(digest.bytes, png.len() as u64);
        assert_eq!(digest.ihdr, Some(FIRST_GENERATION_IHDR));

        // A header that fails its own CRC describes nothing.
        let mut damaged = png.clone();
        damaged[20] ^= 0x01;
        assert_eq!(read_ihdr(&damaged), None);
        assert_eq!(read_ihdr(&png[..32]), None);
    }

    #[test]
    fn the_fixture_is_a_first_generation_png_the_engine_reads() {
        // The engine's one PNG decoder (ADR-0022), not this module's reader.
        let png = fixture_png();
        let decoded = nexora_image::png::decode(&png).expect("decodes");
        assert_eq!(decoded.resolution.width, 16);
        assert_eq!(decoded.resolution.height, 16);
        assert_eq!(decoded.layout, nexora_asset::texture::ChannelLayout::Rgba);
        assert_eq!(decoded.pixels.len(), 16 * 16 * 4);
        // Compressed like the catalog's albedos: hundreds of bytes, not 1 KiB.
        assert!(png.len() < 600, "{} bytes", png.len());
        assert_eq!(fixture_png(), png, "the fixture is reproducible");
    }

    #[test]
    fn the_probe_accepts_python_3_8_and_later_only() {
        let found = parse_probe("python3", "3 14 6 cpython\n").expect("3.14");
        assert_eq!(found.version, (3, 14, 6));
        assert_eq!(found.describe(), "Python 3.14.6 (cpython, via python3)");
        assert!(parse_probe("python", "3 8 0 pypy").is_some());
        for text in [
            "3 7 9 cpython",
            "2 7 18 cpython",
            "4 0 0 cpython",
            "",
            "3 11",
            "3 11 9 cpython extra",
        ] {
            assert!(parse_probe("python", text).is_none(), "{text:?}");
        }
    }

    #[test]
    fn a_missing_interpreter_is_a_gap_and_never_a_number() {
        // What a machine without Python sees: nothing on the path answers.
        assert_eq!(
            find_interpreter(&["nexora-no-such-python-a", "nexora-no-such-python-b"]),
            None
        );
        let stage = ToolStage::missing(NO_INTERPRETER);
        assert!(stage.measurements.is_empty());
        assert_eq!(stage.interpreter, None);
        assert_eq!(stage.gap, Some(NO_INTERPRETER));
    }
}
