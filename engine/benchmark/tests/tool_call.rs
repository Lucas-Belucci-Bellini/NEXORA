//! The cross-language tool-call stage against the real tool (ADR-0034).
//!
//! `tools/catalog-digest/catalog_digest.py`, run by this machine's Python
//! and spoken to over the contract in `tools/catalog-digest/README.md`. Its
//! answers are held to Rust's own (`nexora_foundation::hashing`) on the
//! published FNV-1a vectors and on a first-generation PNG, and its refusals
//! to the contract's codes.
//!
//! No interpreter is a failure here, not a skip, as no adapter is for the RHI
//! stage: a machine without Python says so with `NEXORA_PYTHON=none`, and the
//! stage must then be declared not measured, with that reason, and carry no
//! number.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use nexora_benchmark::tool::{
    cold_call, digest_bytes, find_interpreter, fixture_png, parse_banner, parse_response, probe,
    tool_call, Answer, Interpreter, Session, CANDIDATES, DECLARED_NO_PYTHON, INTERPRETER_FLAGS,
    TOOL_SOURCE,
};
use nexora_benchmark::Budget;

fn declared_absent() -> bool {
    std::env::var("NEXORA_PYTHON").as_deref() == Ok("none")
}

/// The interpreter the stage itself would pick, or a failure saying how to
/// declare that there is none.
fn interpreter() -> Interpreter {
    let (found, tried) = match std::env::var("NEXORA_PYTHON") {
        Ok(named) if !named.is_empty() => (probe(&named), format!("NEXORA_PYTHON=`{named}`")),
        _ => (
            find_interpreter(&CANDIDATES),
            "`python3`, then `python`".to_owned(),
        ),
    };
    found.unwrap_or_else(|| {
        panic!(
            "no Python 3.8+ interpreter answered (tried {tried}): \
             install one, or set NEXORA_PYTHON=none to declare there is none"
        )
    })
}

/// A scratch directory of this test's own, holding the tool.
fn scratch(test: &str) -> (PathBuf, PathBuf) {
    let dir = std::env::temp_dir().join(format!("nexora-tool-call-{test}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("scratch");
    let script = dir.join("catalog_digest.py");
    std::fs::write(&script, TOOL_SOURCE).expect("tool");
    (dir, script)
}

#[test]
fn the_stage_measures_the_real_tool_or_declares_why_not() {
    let (dir, _) = scratch("stage");
    let budget = Budget {
        warmup_iterations: 0,
        samples: 1,
        iterations_per_sample: 1,
    };
    let stage = tool_call(budget, &dir).expect("checked answers, or a declared gap");
    let _ = std::fs::remove_dir_all(&dir);

    if declared_absent() {
        assert_eq!(stage.gap, Some(DECLARED_NO_PYTHON));
        assert!(
            stage.measurements.is_empty(),
            "a declared gap has no number"
        );
        assert_eq!(stage.interpreter, None);
        return;
    }
    assert_eq!(
        stage.gap, None,
        "the stage did not run: set NEXORA_PYTHON=none to declare no Python"
    );
    let described = stage.interpreter.expect("the interpreter is recorded");
    assert!(described.starts_with("Python 3."), "{described}");

    let value = |name: &str| {
        stage
            .measurements
            .iter()
            .find(|m| m.name == name)
            .unwrap_or_else(|| panic!("missing {name}"))
            .median()
    };
    let cold = value("tool.cold_call");
    let warm = value("tool.warm_roundtrip");
    let work = value("tool.python_work");
    let rust = value("tool.digest_in_rust");
    assert!(cold > 0.0 && warm > 0.0 && work > 0.0 && rust > 0.0);
    // The tool's work happens inside the round trip it is reported in.
    assert!(
        work <= warm,
        "work {work} ns outside a {warm} ns round trip"
    );
    assert_eq!(
        value("tool.payload_bytes") as usize,
        fixture_png().len(),
        "the payload is the fixture"
    );
}

#[test]
fn the_tool_agrees_with_rust_on_every_byte_it_is_given() {
    if declared_absent() {
        return;
    }
    let interpreter = interpreter();
    let (dir, script) = scratch("agree");

    let png = fixture_png();
    let mut damaged = png.clone();
    damaged[20] ^= 0x01; // the IHDR no longer matches its CRC
    let files: [(&str, &[u8]); 6] = [
        ("empty.bin", b""),
        ("a.bin", b"a"),
        ("foobar.bin", b"foobar"),
        ("albedo.png", &png),
        ("damaged.png", &damaged),
        ("name with spaces.png", &png),
    ];

    let mut session = Session::start(&interpreter, &script).expect("the banner");
    for (name, bytes) in files {
        let path = dir.join(name);
        std::fs::write(&path, bytes).expect("written");
        let response = session.digest(&path).expect("answered");
        let Answer::Digest { digest, .. } = response.answer else {
            panic!("{name}: refused: {:?}", response.answer);
        };
        assert_eq!(digest, digest_bytes(bytes), "{name}");
    }
    // The published FNV-1a 64 vector, as the tool says it, not as Rust does.
    let foobar = session.digest(&dir.join("foobar.bin")).expect("answered");
    assert!(matches!(
        foobar.answer,
        Answer::Digest { digest, .. } if digest.fnv1a64 == 0x8594_4171_f739_67e8 && digest.ihdr.is_none()
    ));
    // A missing file is a structured refusal, and the session goes on.
    let absent = session
        .digest(&dir.join("no-such-file.png"))
        .expect("answered");
    assert!(
        matches!(&absent.answer, Answer::Refused { code, .. } if code == "unreadable"),
        "{:?}",
        absent.answer
    );
    let after = session.digest(&dir.join("albedo.png")).expect("answered");
    assert!(matches!(after.answer, Answer::Digest { digest, .. } if digest == digest_bytes(&png)));
    session.finish().expect("a clean exit at the end of input");

    // A tool run once says the same.
    let once = cold_call(&interpreter, &script, &dir.join("albedo.png")).expect("one call");
    assert!(matches!(once.answer, Answer::Digest { digest, .. } if digest == digest_bytes(&png)));
    let _ = std::fs::remove_dir_all(&dir);
}

/// Requests the benchmark's client never formats, sent raw: the tool's half
/// of the grammar, refused by name rather than by a traceback.
#[test]
fn the_tool_refuses_what_the_contract_does_not_define() {
    if declared_absent() {
        return;
    }
    let interpreter = interpreter();
    let (dir, script) = scratch("refuse");
    let mut child = Command::new(&interpreter.program)
        .args(INTERPRETER_FLAGS)
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("started");
    let requests: &[&[u8]] = &[
        b"abc digest x\n",
        b"5 frob x\n",
        b"6 digest\n",
        b"7\n",
        b"8 digest \xff\xfe\n",
        b"9 digest also-missing.png\r\n",
    ];
    {
        let mut stdin = child.stdin.take().expect("stdin");
        for request in requests {
            stdin.write_all(request).expect("sent");
        }
    }
    let output = child.wait_with_output().expect("exited");
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty(), "nothing goes to stderr");

    let text = String::from_utf8(output.stdout).expect("UTF-8");
    let mut lines = text.split_terminator('\n');
    assert_eq!(parse_banner(lines.next().expect("banner")).expect("v1"), 1);
    let answers: Vec<(Option<u64>, String)> = lines
        .map(|line| {
            let response = parse_response(line).expect("on the grammar");
            match response.answer {
                Answer::Refused { code, .. } => (response.id, code),
                Answer::Digest { .. } => panic!("{line}: not refused"),
            }
        })
        .collect();
    let expected: Vec<(Option<u64>, String)> = [
        (None, "bad-request"),
        (Some(5), "unknown-verb"),
        (Some(6), "bad-request"),
        (Some(7), "bad-request"),
        (None, "bad-request"),
        (Some(9), "unreadable"),
    ]
    .into_iter()
    .map(|(id, code)| (id, code.to_owned()))
    .collect();
    assert_eq!(answers, expected, "one answer per request, in order");
}
