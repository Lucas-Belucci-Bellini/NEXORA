#!/usr/bin/env python3
"""NEXORA local validation bridge.

The container this repository is developed in has no GPU and no display, and
the architecture freeze cannot close without both (DEBT-0008, the RHI). This
script is how a run on a real machine becomes evidence the repository can read:

    python3 scripts/local-validation.py run      # on the machine, then commit
    python3 scripts/local-validation.py check    # anywhere: is it still current?

`run` records the commit, the machine (without anything that identifies a
person or a device), and the result of every check that machine can execute,
into docs/validation/local/NEXORA-LOCAL-VALIDATION.{json,md}. `check` reads that
report back and classifies it against HEAD, because a report is evidence about
the commit it ran on and nothing newer.

Two rules this script keeps, and why:

* Nothing is marked as passed that did not run. The native RHI backend is a
  check (`rhi_native`): it opens this machine's GPU, runs the conformance
  suite, uploads a texture and draws, reading both back (ADR-0026). The window
  host is another (`window`): it opens a window, presents to it, and reads the
  first frame back from the surface where the platform allows (ADR-0027). A
  renderer, real input devices and client mode are recorded as
  NOT_IMPLEMENTED, because the engine has no code for them yet --
  a real GPU cannot validate code that does not exist, and a report that said
  "not tested" would invite someone to test it.
* Nothing personal is recorded: no hostname, user name, serial number, MAC
  address or absolute path. Command output is kept only as short tails, with
  the repository root and the home directory replaced by placeholders.

Standard library only (ADR-0002's spirit applies to tools too), Python 3.8+.
"""

from __future__ import annotations

import argparse
import datetime as _dt
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

SCHEMA = 1
REPO = Path(__file__).resolve().parent.parent
REPORT_DIR = Path("docs/validation/local")
REPORT_JSON = "NEXORA-LOCAL-VALIDATION.json"
REPORT_MD = "NEXORA-LOCAL-VALIDATION.md"

# Paths whose change after a report makes that report stale. Documentation
# does not: a report about the engine stays true when a README moves.
RELEVANT = ["engine/", "tools/", "content/", "benchmarks/", "Cargo.toml", "Cargo.lock"]

STATUSES = ("PASS", "FAIL", "SKIPPED", "NOT_IMPLEMENTED")

# What a real machine is needed for, and why none of it can pass yet.
# The window, its surface (swapchain) and presentation left this list with
# ADR-0027: they are the `window` check now.
HARDWARE_GATED = [
    ("rendering", "no renderer: the native backend draws one triangle (rhi_native) and presents "
                  "a 16x16 target (window); nothing draws the world; meshes are data (ADR-0012)"),
    ("input_devices", "no real device has produced a signal (DEBT-0043)"),
    ("client_mode", "the runtime starts headless only; client mode is Phase 1's exit"),
    ("benchmark_gpu_stages", "partly built: the RHI stage (device, fence, upload, draw) runs inside "
                             "benchmark_cpu on this machine's adapter; window frame time and camera "
                             "have no implementation (DEBT-0008)"),
]


# --------------------------------------------------------------------------
# small helpers


def _exe(name: str) -> str:
    return name + (".exe" if os.name == "nt" else "")


def _run(args, timeout=None, cwd=REPO):
    """Run a command; return (code, combined output, seconds). Never raises."""
    began = time.monotonic()
    try:
        done = subprocess.run(
            args,
            cwd=str(cwd),
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            timeout=timeout,
            text=True,
            # The engine's tools write UTF-8 on every OS; the locale's code
            # page (cp1252 on many Windows machines) would misread it.
            encoding="utf-8",
            errors="replace",
        )
        return done.returncode, done.stdout, time.monotonic() - began
    except (OSError, subprocess.TimeoutExpired) as cause:
        return None, f"{type(cause).__name__}: {cause}", time.monotonic() - began


def _probe(args, timeout=20):
    code, out, _ = _run(args, timeout=timeout)
    return out.strip() if code == 0 else None


# The run's temporary directory, once there is one; its path is noise.
SCRATCH: Path | None = None


def scrub(text: str) -> str:
    """Remove what would identify the machine's owner from command output."""
    out = text
    if SCRATCH is not None:
        out = out.replace(str(SCRATCH), "<scratch>")
    out = out.replace(str(REPO), "<repo>")
    tmp = tempfile.gettempdir()
    out = re.sub(re.escape(tmp) + r"[/\\]nexora-local-validation-[^/\\\s]+", "<scratch>", out)
    home = str(Path.home())
    if len(home) > 1:
        out = out.replace(home, "<home>")
    user = os.environ.get("USER") or os.environ.get("USERNAME")
    if user and len(user) > 2:
        out = re.sub(rf"\b{re.escape(user)}\b", "<user>", out)
    # MAC addresses and anything that calls itself a serial.
    out = re.sub(r"\b[0-9A-Fa-f]{2}(?:[:-][0-9A-Fa-f]{2}){5}\b", "<mac>", out)
    out = re.sub(r"(?i)(serial[^:=\n]*[:=]\s*)\S+", r"\1<serial>", out)
    return out


def tail(text: str, lines: int = 12) -> list:
    return scrub(text).strip().splitlines()[-lines:]


# --------------------------------------------------------------------------
# environment


def detect_gpus() -> list:
    """Best effort, names and driver versions only."""
    found = []
    smi = _probe(["nvidia-smi", "--query-gpu=name,driver_version", "--format=csv,noheader"])
    if smi:
        for line in smi.splitlines():
            name, _, driver = line.partition(",")
            found.append({"name": name.strip(), "driver": driver.strip() or None, "via": "nvidia-smi"})
    system = platform.system()
    if system == "Windows":
        ps = _probe([
            "powershell", "-NoProfile", "-Command",
            "Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion | ConvertTo-Json",
        ])
        if ps:
            try:
                rows = json.loads(ps)
                for row in rows if isinstance(rows, list) else [rows]:
                    found.append({"name": row.get("Name"), "driver": row.get("DriverVersion"), "via": "cim"})
            except ValueError:
                pass
    elif system == "Darwin":
        sp = _probe(["system_profiler", "SPDisplaysDataType", "-json"])
        if sp:
            try:
                for row in json.loads(sp).get("SPDisplaysDataType", []):
                    found.append({"name": row.get("sppci_model"), "driver": None, "via": "system_profiler"})
            except ValueError:
                pass
    elif system == "Linux" and not found:
        lspci = _probe(["lspci", "-mm"])
        for line in (lspci or "").splitlines():
            if re.search(r'"(VGA compatible controller|3D controller|Display controller)"', line):
                fields = re.findall(r'"([^"]*)"', line)
                found.append({"name": " ".join(fields[1:3]), "driver": None, "via": "lspci"})
    return found


def detect_display() -> str:
    system = platform.system()
    if system == "Linux":
        if os.environ.get("WAYLAND_DISPLAY"):
            return "wayland"
        if os.environ.get("DISPLAY"):
            return "x11"
        return "none"
    # A desktop session on Windows or macOS almost always has one, but this
    # script does not open a window to find out, so it does not claim it.
    return "not probed"


def cpu_model() -> str | None:
    system = platform.system()
    if system == "Linux":
        try:
            for line in Path("/proc/cpuinfo").read_text().splitlines():
                if line.startswith("model name"):
                    return line.split(":", 1)[1].strip()
        except OSError:
            return None
    if system == "Darwin":
        return _probe(["sysctl", "-n", "machdep.cpu.brand_string"])
    if system == "Windows":
        return _probe([
            "powershell", "-NoProfile", "-Command",
            "(Get-CimInstance Win32_Processor | Select-Object -First 1).Name",
        ]) or platform.processor() or None
    return platform.processor() or None


def memory_bytes() -> int | None:
    system = platform.system()
    try:
        if system == "Linux":
            for line in Path("/proc/meminfo").read_text().splitlines():
                if line.startswith("MemTotal:"):
                    return int(line.split()[1]) * 1024
        if system == "Darwin":
            raw = _probe(["sysctl", "-n", "hw.memsize"])
            return int(raw) if raw else None
        if system == "Windows":
            raw = _probe([
                "powershell", "-NoProfile", "-Command",
                "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory",
            ])
            return int(raw) if raw else None
    except (OSError, ValueError):
        return None
    return None


def detect_host() -> dict:
    """What kind of machine this is: `ci`, `container`, or `machine`.

    Only a `machine` produces local evidence. A container or a CI runner is
    exactly the environment the bridge exists to get away from, and a report
    from one committed as "local" would claim hardware that was not there.
    A virtual machine counts as a machine: an operator's development box may
    be one, and whether it has a GPU is recorded separately.
    """
    if any(os.environ.get(name, "").lower() in ("1", "true") for name in ("CI", "GITHUB_ACTIONS")):
        return {"kind": "ci", "virtualization": None}
    container = None
    if Path("/.dockerenv").exists():
        container = "docker"
    elif Path("/run/.containerenv").exists():
        container = "podman"
    else:
        detected = _probe(["systemd-detect-virt", "--container"])
        if detected and detected != "none":
            container = detected
        else:
            try:
                cgroup = Path("/proc/1/cgroup").read_text()
                match = re.search(r"(docker|kubepods|containerd|lxc)", cgroup)
                container = match.group(1) if match else None
            except OSError:
                pass
    if container:
        return {"kind": "container", "virtualization": container}
    virt = _probe(["systemd-detect-virt", "--vm"])
    return {"kind": "machine", "virtualization": virt if virt and virt != "none" else None}


def environment() -> dict:
    return {
        "host": detect_host(),
        "os": platform.system(),
        "os_release": platform.release(),
        "arch": platform.machine(),
        "cpu": cpu_model(),
        "logical_cpus": os.cpu_count(),
        "memory_bytes": memory_bytes(),
        "gpus": detect_gpus(),
        "display": detect_display(),
        "rustc": _probe(["rustc", "-V"]),
        "cargo": _probe(["cargo", "-V"]),
        "python": platform.python_version(),
    }


def git_state() -> dict:
    sha = _probe(["git", "rev-parse", "HEAD"])
    branch = _probe(["git", "rev-parse", "--abbrev-ref", "HEAD"])
    # Tracked changes only: an untracked file cannot reach the build without a
    # tracked `mod` line or manifest entry changing too.
    porcelain = _probe(["git", "status", "--porcelain", "--untracked-files=no"]) or ""
    changed = [line for line in porcelain.splitlines() if line.strip()]
    return {"commit": sha, "branch": branch, "dirty": bool(changed), "dirty_files": len(changed)}


# --------------------------------------------------------------------------
# checks


# The most lines of a check's output a report keeps whole. The benchmark's
# markdown is ~130 lines; the bound only stops a runaway tool from filling
# the report.
MAX_KEPT_LINES = 500


def whole(text: str) -> list:
    """All of a tool's output, scrubbed and bounded -- for the output that is
    the point of the check, like the benchmark's measurements."""
    lines = scrub(text).rstrip().splitlines()
    if len(lines) > MAX_KEPT_LINES:
        return lines[:MAX_KEPT_LINES] + [f"... {len(lines) - MAX_KEPT_LINES} more lines not kept"]
    return lines


def check(id_, args, summary_from=None, timeout=3600, keep_whole=False):
    # A full run takes minutes; say what is running so a quiet terminal does
    # not look like a hang.
    print(f"running  {id_} ...", flush=True)
    code, out, seconds = _run(args, timeout=timeout)
    status = "PASS" if code == 0 else "FAIL"
    summary = summary_from(out) if summary_from and code == 0 else None
    if summary is None:
        summary = "exit 0" if code == 0 else f"exit {code}"
    return {
        "id": id_,
        "status": status,
        "seconds": round(seconds, 1),
        "command": " ".join(scrub(str(a)) for a in args),
        "summary": scrub(summary),
        "tail": tail(out),
        "output": whole(out) if keep_whole else None,
    }


def _test_totals(out: str) -> str:
    passed = failed = 0
    for match in re.finditer(r"test result: \w+\. (\d+) passed; (\d+) failed", out):
        passed += int(match.group(1))
        failed += int(match.group(2))
    return f"{passed} passed, {failed} failed"


def _lines(*prefixes):
    def pick(out: str) -> str:
        keep = [re.sub(r"\s{2,}", " ", line.strip()) for line in out.splitlines()
                if line.startswith(prefixes)]
        return "; ".join(keep) or "exit 0"
    return pick


# --------------------------------------------------------------------------
# preflight: is this machine able to build anything at all?

WINDOWS_LINKER = (
    "Rust on Windows links with Microsoft's C++ linker (link.exe), and it is not installed.\n"
    "Install the Visual Studio Build Tools with the C++ workload, then open a NEW terminal:\n"
    "  winget install Microsoft.VisualStudio.2022.BuildTools --override "
    "\"--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended\"\n"
    "or download them from https://visualstudio.microsoft.com/visual-cpp-build-tools/ and tick\n"
    "\"Desktop development with C++\". Visual Studio Code is a different product and does not include it."
)
UNIX_LINKER = {
    "Darwin": "Install Apple's command line tools, then retry:  xcode-select --install",
    "Linux": "Install a C toolchain, then retry:  sudo apt install build-essential  (or your distribution's equivalent)",
}


def explain_build_failure(output: str, system: str) -> str:
    """Turn a failed probe build into the one thing to install."""
    if "link.exe" in output:
        return WINDOWS_LINKER
    if re.search(r"linker `(cc|clang|gcc)` not found", output):
        return UNIX_LINKER.get(system, "Install a C toolchain (a linker named cc), then retry.")
    return "A one-line Rust program did not build here; the compiler said:\n" + "\n".join(tail(output))


def preflight() -> list:
    """Problems that would stop every check, found in seconds instead of
    after minutes of building. Empty means the machine can build."""
    problems = []
    for tool, how in (
        ("git", "https://git-scm.com/downloads"),
        ("cargo", "https://rustup.rs"),
        ("rustc", "https://rustup.rs"),
    ):
        if shutil.which(tool) is None:
            problems.append(f"`{tool}` is not on PATH -- install it from {how}, then open a new terminal.")
    if problems:
        return problems
    probe = Path(tempfile.mkdtemp(prefix="nexora-local-validation-preflight-"))
    try:
        source = probe / "probe.rs"
        source.write_text("fn main() {}\n", encoding="utf-8")
        # From the repository, so rustup uses (and installs) the pinned toolchain.
        code, out, _ = _run(["rustc", str(source), "-o", str(probe / _exe("probe"))], timeout=900)
        if code != 0:
            problems.append(explain_build_failure(out, platform.system()))
    finally:
        shutil.rmtree(probe, ignore_errors=True)
    return problems


def run_checks(scratch: Path, quick: bool) -> list:
    release = REPO / "target" / "release"
    headless = str(release / _exe("nexora-headless"))
    forge = str(release / _exe("nexora-texture-forge"))
    bench = str(release / _exe("nexora-benchmark"))
    probe = str(release / _exe("nexora-rhi-probe"))
    window_probe = str(release / _exe("nexora-window-probe"))
    slice_lines = _lines("result", "memory ", "content ", "queries", "rhi ", "probes verified")

    results = [check("build_release", ["cargo", "build", "--workspace", "--release"])]
    built = results[0]["status"] == "PASS"
    if quick:
        results.append(skipped("tests", "--quick"))
    else:
        results.append(check("tests", ["cargo", "test", "--workspace", "--all-targets"], _test_totals))

    def needs_build(id_, args, summary=None, keep_whole=False):
        if not built:
            return skipped(id_, "the release build failed")
        return check(id_, args, summary, keep_whole=keep_whole)

    results.append(needs_build("headless_slice", [headless, "--quiet", "--save", str(scratch / "w.nxsv")], slice_lines))
    results.append(needs_build("headless_slice_content", [
        headless, "--quiet", "--save", str(scratch / "c.nxsv"),
        "--content", "content/first-generation/blocks.json"], slice_lines))
    results.append(needs_build("forge_first_generation", [
        forge, "build", "content/first-generation/plan.json",
        "--out", str(scratch / "fg"), "--recipes", "content/recipes"], _lines("build ", "index ")))
    results.append(needs_build("headless_slice_textures", [
        headless, "--quiet", "--radius", "1", "--save", str(scratch / "t.nxsv"),
        "--content", "content/first-generation/blocks.json",
        "--resources", str(scratch / "fg")], slice_lines))
    # The native RHI backend on this machine's GPU: device, conformance,
    # WGSL shaders, upload and draw, both read back (ADR-0026). A machine with
    # no GPU says so; it is never recorded as a pass.
    if os.environ.get("NEXORA_GPU") == "none":
        results.append(skipped("rhi_native", "NEXORA_GPU=none: this machine declares no GPU"))
    else:
        results.append(needs_build("rhi_native", [probe],
                                   _lines("adapter", "conformance", "upload", "draw", "bound",
                                          "result")))
    # The window host: a window, its surface, the conformance suite with
    # presentation on, and frames shown in it, the first read back from the
    # surface where the platform allows (ADR-0027). No display is declared,
    # like no GPU; otherwise its absence is a failure.
    if os.environ.get("NEXORA_DISPLAY") == "none":
        results.append(skipped("window", "NEXORA_DISPLAY=none: this machine declares no display"))
    elif os.environ.get("NEXORA_GPU") == "none":
        results.append(skipped("window", "NEXORA_GPU=none: nothing can present without a GPU"))
    else:
        results.append(needs_build("window", [window_probe],
                                   _lines("adapter", "window", "surface", "conformance", "frame",
                                          "presented", "result")))
    # The CPU benchmark is the point of a second machine for DEBT-0013 and
    # DEBT-0008: the container's numbers are one machine's. It also runs the
    # RHI stage on this machine's adapter and names the adapter in its
    # environment table; the check keeps its id so old reports still compare.
    results.append(needs_build("benchmark_cpu", [bench, "--markdown"] + (["--smoke"] if quick else [])
                               + ["--scratch", str(scratch / "bench")],
                               lambda out: "see the report's benchmark section", keep_whole=True))
    return results


def skipped(id_, why):
    return {"id": id_, "status": "SKIPPED", "seconds": 0.0, "command": None, "summary": why, "tail": []}


# --------------------------------------------------------------------------
# report


def build_report(quick: bool) -> dict:
    global SCRATCH
    scratch = Path(tempfile.mkdtemp(prefix="nexora-local-validation-"))
    SCRATCH = scratch
    try:
        checks = run_checks(scratch, quick)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    benchmark = next((c for c in checks if c["id"] == "benchmark_cpu"), None)
    return {
        "schema": SCHEMA,
        "generated_at": _dt.datetime.now(_dt.timezone.utc).replace(microsecond=0).isoformat(),
        "git": git_state(),
        "environment": environment(),
        "quick": quick,
        "checks": checks,
        "hardware_gated": [
            {"id": id_, "status": "NOT_IMPLEMENTED", "reason": reason} for id_, reason in HARDWARE_GATED
        ],
        "benchmark_tail": benchmark["tail"] if benchmark else [],
        # The measurements themselves: a second machine's numbers are what
        # DEBT-0013 waits for, and the tail alone is only the list of stages
        # that have no number.
        "benchmark_output": (benchmark.get("output") or []) if benchmark else [],
        "verdict": {
            "executed_pass": all(c["status"] == "PASS" for c in checks if c["status"] != "SKIPPED"),
            "failures": [c["id"] for c in checks if c["status"] == "FAIL"],
        },
    }


def render_markdown(report: dict) -> str:
    env, git = report["environment"], report["git"]
    lines = [
        "# NEXORA — local validation report",
        "",
        "Written by `scripts/local-validation.py run`. Evidence about **one commit on one",
        "machine**; `python3 scripts/local-validation.py check` says whether it still",
        "describes HEAD. See `docs/validation/local/README.md`.",
        "",
        "| | |",
        "| --- | --- |",
        f"| commit | `{git['commit']}`{' (dirty: %d files)' % git['dirty_files'] if git['dirty'] else ''} |",
        f"| branch | `{git['branch']}` |",
        f"| generated | {report['generated_at']} |",
        f"| OS | {env['os']} {env['os_release']} ({env['arch']}) |",
        f"| CPU | {env['cpu']} — {env['logical_cpus']} logical |",
        f"| memory | {env['memory_bytes']} bytes |",
        f"| GPU | {'; '.join(filter(None, (g.get('name') for g in env['gpus']))) or 'none detected'} |",
        f"| display | {env['display']} |",
        f"| toolchain | {env['rustc']} · {env['cargo']} |",
        "",
        "## Checks that ran",
        "",
        "| check | status | seconds | summary |",
        "| --- | --- | ---: | --- |",
    ]
    for c in report["checks"]:
        lines.append(f"| `{c['id']}` | **{c['status']}** | {c['seconds']} | {c['summary']} |")
    lines += [
        "",
        "## What this machine could not validate, and why",
        "",
        "Not \"not tested\": the engine has no code for these yet, so no machine can.",
        "",
        "| item | status | reason |",
        "| --- | --- | --- |",
    ]
    for item in report["hardware_gated"]:
        lines.append(f"| `{item['id']}` | {item['status']} | {item['reason']} |")
    if report.get("benchmark_output"):
        # Markdown already: the benchmark is run with --markdown.
        # Its headings are demoted one level so they nest under this section.
        lines += ["", "## Benchmark (CPU stages, and the RHI stage on this machine's adapter)", ""]
        lines += ["#" + line if line.startswith("#") else line for line in report["benchmark_output"]]
    elif report.get("benchmark_tail"):
        lines += ["", "## CPU benchmark (tail)", "", "```text", *report["benchmark_tail"], "```"]
    failures = report["verdict"]["failures"]
    lines += ["", "## Verdict", "", "Every executed check passed." if not failures
              else "Failed: " + ", ".join(f"`{f}`" for f in failures) + "."]
    return "\n".join(lines) + "\n"


# --------------------------------------------------------------------------
# check: is the evidence still about HEAD?


def validate(report: dict) -> list:
    """Structural problems with a report; empty means it is well formed."""
    problems = []
    if report.get("schema") != SCHEMA:
        problems.append(f"schema is {report.get('schema')!r}, this script reads {SCHEMA}")
    commit = (report.get("git") or {}).get("commit")
    if not commit or not re.fullmatch(r"[0-9a-f]{40}", commit):
        problems.append("git.commit is not a full commit id")
    for c in report.get("checks", []):
        if c.get("status") not in STATUSES:
            problems.append(f"check {c.get('id')!r} has status {c.get('status')!r}")
    for item in report.get("hardware_gated", []):
        if item.get("status") == "PASS":
            problems.append(f"{item.get('id')!r} is hardware-gated and cannot pass")
    host = ((report.get("environment") or {}).get("host") or {}).get("kind")
    if host != "machine":
        problems.append(f"the report ran on a {host!r} host; local evidence must come from a machine")
    return problems


def classify(report: dict, head: str | None, changed: list | None) -> dict:
    """Classify each item. `changed` is the relevant paths changed since the
    report's commit, or None when that commit is unknown here."""
    commit = report["git"]["commit"]
    if report["git"]["dirty"]:
        # Uncommitted changes were in the tree: the run describes no commit.
        freshness = "UNCOMMITTED_TREE"
    elif head == commit:
        freshness = "CURRENT"
    elif changed is None:
        freshness = "UNKNOWN_COMMIT"
    elif changed:
        freshness = "STALE_LOCAL_EVIDENCE"
    else:
        freshness = "CURRENT_NO_RELEVANT_CHANGE"
    usable = freshness in ("CURRENT", "CURRENT_NO_RELEVANT_CHANGE")
    items = {}
    for c in report["checks"]:
        if c["status"] == "PASS":
            items[c["id"]] = "VERIFIED_ON_LOCAL_HARDWARE" if usable else "STALE_LOCAL_EVIDENCE"
        elif c["status"] == "FAIL":
            items[c["id"]] = "FAILED_ON_LOCAL_HARDWARE" if usable else "STALE_LOCAL_EVIDENCE"
        else:
            items[c["id"]] = "NOT_TESTED_LOCALLY"
    for item in report.get("hardware_gated", []):
        items[item["id"]] = "NOT_TESTED_LOCALLY"
    return {"freshness": freshness, "items": items, "changed": changed or []}


def changed_since(commit: str) -> list | None:
    code, _, _ = _run(["git", "cat-file", "-e", f"{commit}^{{commit}}"])
    if code != 0:
        return None
    code, out, _ = _run(["git", "diff", "--name-only", f"{commit}..HEAD", "--", *RELEVANT])
    if code != 0:
        return None
    return [line for line in out.splitlines() if line.strip()]


# --------------------------------------------------------------------------
# self-test: the classification rules, without git history or a machine


def self_test() -> int:
    base = {
        "schema": SCHEMA,
        "git": {"commit": "a" * 40, "dirty": False},
        "environment": {"host": {"kind": "machine", "virtualization": None}},
        "checks": [
            {"id": "tests", "status": "PASS"},
            {"id": "headless_slice", "status": "FAIL"},
            {"id": "benchmark_cpu", "status": "SKIPPED"},
        ],
        "hardware_gated": [{"id": "rhi", "status": "NOT_IMPLEMENTED"}],
    }
    assert validate(base) == [], validate(base)
    current = classify(base, "a" * 40, [])
    assert current["freshness"] == "CURRENT"
    assert current["items"] == {
        "tests": "VERIFIED_ON_LOCAL_HARDWARE",
        "headless_slice": "FAILED_ON_LOCAL_HARDWARE",
        "benchmark_cpu": "NOT_TESTED_LOCALLY",
        "rhi": "NOT_TESTED_LOCALLY",
    }, current
    docs_only = classify(base, "b" * 40, [])
    assert docs_only["freshness"] == "CURRENT_NO_RELEVANT_CHANGE"
    assert docs_only["items"]["tests"] == "VERIFIED_ON_LOCAL_HARDWARE"
    stale = classify(base, "b" * 40, ["engine/world/src/world.rs"])
    assert stale["freshness"] == "STALE_LOCAL_EVIDENCE"
    assert stale["items"]["tests"] == "STALE_LOCAL_EVIDENCE"
    assert stale["items"]["headless_slice"] == "STALE_LOCAL_EVIDENCE"
    assert classify(base, "b" * 40, None)["freshness"] == "UNKNOWN_COMMIT"
    dirty = dict(base, git={"commit": "a" * 40, "dirty": True})
    assert classify(dirty, "a" * 40, [])["freshness"] == "UNCOMMITTED_TREE"
    assert classify(dirty, "a" * 40, [])["items"]["tests"] == "STALE_LOCAL_EVIDENCE"
    lying = dict(base, hardware_gated=[{"id": "rhi", "status": "PASS"}])
    assert any("cannot pass" in p for p in validate(lying))
    assert validate(dict(base, git={"commit": "HEAD", "dirty": False}))
    for kind in ("container", "ci"):
        from_elsewhere = dict(base, environment={"host": {"kind": kind}})
        assert any("must come from a machine" in p for p in validate(from_elsewhere)), kind
    assert scrub("aa:bb:cc:dd:ee:ff serial number: X1") == "<mac> serial number: <serial>"
    leaked = str(Path(tempfile.gettempdir()) / "nexora-local-validation-abc123" / "fg")
    assert scrub(leaked) == str(Path("<scratch>") / "fg"), scrub(leaked)
    assert _lines("result")("result             OK\nother") == "result OK"
    many = "\n".join(f"| row {n} |" for n in range(MAX_KEPT_LINES + 20))
    kept = whole(many)
    assert len(kept) == MAX_KEPT_LINES + 1 and kept[-1].startswith("... 20 more"), kept[-1]
    assert whole("a\nb\n") == ["a", "b"], "short output is kept whole, not tailed"
    missing_msvc = "error: linker `link.exe` not found\n  = note: program not found"
    assert "Build Tools" in explain_build_failure(missing_msvc, "Windows")
    assert "Desktop development with C++" in explain_build_failure(missing_msvc, "Windows")
    assert "xcode-select" in explain_build_failure("error: linker `cc` not found", "Darwin")
    assert "build-essential" in explain_build_failure("error: linker `cc` not found", "Linux")
    assert "said" in explain_build_failure("error[E0425]: something else", "Linux")
    print("local-validation self-test: ok")
    return 0


# --------------------------------------------------------------------------


def main() -> int:
    # A Windows console may use a code page that cannot print every character
    # a tool emits; replace those rather than dying on them.
    for stream in (sys.stdout, sys.stderr):
        if hasattr(stream, "reconfigure"):
            stream.reconfigure(errors="replace")
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="verb", required=True)
    run = sub.add_parser("run", help="run every check this machine can, and write the report")
    run.add_argument("--quick", action="store_true", help="skip the test suite; smoke benchmark")
    run.add_argument("--out", type=Path, default=None,
                     help="report directory (default: docs/validation/local; refused on a container or CI)")
    chk = sub.add_parser("check", help="classify an existing report against HEAD")
    chk.add_argument("--report", type=Path, default=REPO / REPORT_DIR / REPORT_JSON)
    chk.add_argument("--allow-missing", action="store_true", help="exit 0 when there is no report")
    sub.add_parser("self-test", help="check the classification rules")
    sub.add_parser("preflight", help="check this machine can build, in seconds, before a full run")
    args = parser.parse_args()

    if args.verb == "self-test":
        return self_test()

    if args.verb == "preflight" or args.verb == "run":
        print("preflight: can this machine build Rust?", flush=True)
        problems = preflight()
        if problems:
            for problem in problems:
                print(f"\nNOT READY: {problem}")
            print("\nNothing was built and no report was written. Fix the above and run again.")
            return 4
        print("preflight: ok")
        if args.verb == "preflight":
            return 0

    if args.verb == "run":
        host = detect_host()
        if args.out is None:
            if host["kind"] != "machine":
                print(f"this is a {host['kind']} ({host['virtualization']}), not a machine: its report "
                      "is not local evidence. Pass --out to write it somewhere else.")
                return 2
            args.out = REPO / REPORT_DIR
        report = build_report(args.quick)
        args.out.mkdir(parents=True, exist_ok=True)
        (args.out / REPORT_JSON).write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        (args.out / REPORT_MD).write_text(render_markdown(report), encoding="utf-8")
        for c in report["checks"]:
            print(f"{c['status']:<8} {c['id']:<26} {c['summary']}")
        print(f"report: {scrub(str(args.out / REPORT_MD))}")
        return 0 if report["verdict"]["executed_pass"] else 1

    if not args.report.exists():
        print(f"no local validation report at {scrub(str(args.report))}")
        return 0 if args.allow_missing else 3
    try:
        report = json.loads(args.report.read_text(encoding="utf-8"))
    except ValueError as cause:
        print(f"the report is not JSON: {cause}")
        return 2
    problems = validate(report)
    if problems:
        for problem in problems:
            print(f"malformed: {problem}")
        return 2
    verdict = classify(report, git_state()["commit"], changed_since(report["git"]["commit"]))
    print(f"report commit {report['git']['commit'][:12]} on {report['environment']['os']}: "
          f"{verdict['freshness']}")
    for path in verdict["changed"][:20]:
        print(f"  changed since: {path}")
    for id_, status in verdict["items"].items():
        print(f"  {id_:<26} {status}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
