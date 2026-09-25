#!/usr/bin/env bash
#
# Measure how much a benchmark row moves when NOTHING about it changes.
#
# `DEBT-0039`: `voxel.get_paletted` read 11.4 ns at one commit and 7.8-8.2 ns at
# the next, which changed `engine/physics` and `engine/benchmark` and not one
# line of `engine/world`. The release profile is `lto = "thin"` with
# `codegen-units = 1`, so touching any crate in the workspace relinks the whole
# binary and moves the measured code.
#
# That makes the method this repository has been using -- move one thing, check
# the controls did not move -- necessary and NOT sufficient. The controls held
# still across both of those runs and `get_paletted` moved 30% anyway.
#
# So this script measures the noise floor directly. It builds the SAME SOURCE
# n times with a neutral edit between each, runs all of them, and reports the
# spread per row. Whatever spread it finds is the amount a row can move for no
# reason at all, and no published gain smaller than that is a finding.
#
# The neutral edit is a comment appended to `engine/benchmark/src/lib.rs` -- a
# crate that contains none of the measured kernels but is linked into the same
# binary, which is exactly the shape of the change that caused DEBT-0039. The
# edit is reverted on exit, including on failure.
#
# `--repeat-build` inside the runner would not answer this. The variation is in
# the link, not in the execution, so it has to be separate builds. Two builds
# give one gap, which is not a spread, so the default is three.
#
# Usage:
#   scripts/build-spread.sh              # three builds, the numbers that get quoted
#   scripts/build-spread.sh --builds 5   # more builds, tighter bound on the floor
#   scripts/build-spread.sh --smoke      # fewer samples, for checking the plumbing

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_ROOT}"

MARKER_FILE="engine/benchmark/src/lib.rs"
SMOKE=""
BUILDS=3
while [ $# -gt 0 ]; do
    case "$1" in
        --smoke) SMOKE="--smoke"; shift ;;
        --builds) BUILDS="${2:-}"; shift 2 ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done
if ! [ "${BUILDS}" -ge 2 ] 2>/dev/null; then
    echo "--builds needs at least 2; one build cannot show a spread" >&2
    exit 2
fi

WORK="$(mktemp -d)"
PRISTINE="${WORK}/lib.rs.pristine"
cp "${MARKER_FILE}" "${PRISTINE}"

cleanup() {
    # Always put the source back. A script that measures build noise must not
    # become a source of it.
    cp "${PRISTINE}" "${MARKER_FILE}"
    rm -rf "${WORK}"
}
trap cleanup EXIT

# The tree must be clean for the comparison to mean anything: an uncommitted
# edit would make "the same source" false before the first build.
if ! git diff --quiet -- "${MARKER_FILE}"; then
    echo "refusing to run: ${MARKER_FILE} has uncommitted changes" >&2
    exit 1
fi

run_build() {
    local label="$1" out="$2"
    echo "== build ${label} ==" >&2
    cargo build -q -p nexora-benchmark --release >&2
    cargo run -q -p nexora-benchmark --release -- --markdown ${SMOKE} 2>/dev/null > "${out}"
}

OUTPUTS=()
for index in $(seq 1 "${BUILDS}"); do
    if [ "${index}" -gt 1 ]; then
        # A comment, different each time, so every build is a fresh compile and
        # link. It changes no behaviour and no generated code in this crate.
        printf '\n// build-spread marker %s: neutral edit, reverted by scripts/build-spread.sh\n' \
            "${index}" >> "${MARKER_FILE}"
    fi
    run_build "${index}/${BUILDS}" "${WORK}/run-${index}.md"
    OUTPUTS+=("${WORK}/run-${index}.md")
done

cp "${PRISTINE}" "${MARKER_FILE}"

python3 "${REPO_ROOT}/scripts/build-spread-report.py" "${OUTPUTS[@]}"
