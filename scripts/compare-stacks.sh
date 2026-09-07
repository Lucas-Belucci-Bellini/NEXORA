#!/usr/bin/env bash
#
# Compare the Rust engine kernels against the C++ reference implementation.
#
# The order matters and is the whole point: CONFORMANCE FIRST. Two stacks that
# disagree about what they compute cannot be compared on how fast they compute
# it, and a timing table is a very convincing way to publish a wrong number. If
# a single digest differs this script stops and prints the difference; nothing
# is timed.
#
# Usage:
#   scripts/compare-stacks.sh            # full run, the numbers that get quoted
#   scripts/compare-stacks.sh --smoke    # 3 samples, for checking the plumbing
#
# The C++ side is built by EVERY C++20 compiler found, not just one. That is
# the control that keeps the comparison honest: on this machine the CRC-32
# kernel ran in 668 us under g++ and 353 us under clang++, against 364 us for
# Rust. Timed against g++ alone, Rust looks 1.8x faster than "C++"; the real
# difference was the optimizer backend, and Rust and clang++ -- both LLVM --
# agree to within a few percent. A single-compiler comparison would have
# published a language finding that does not exist.
#
# Requires at least one C++20 compiler. Everything else in this repository
# builds without one -- see ADR-0009.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_ROOT}"

SMOKE=""
for argument in "$@"; do
  case "${argument}" in
    --smoke) SMOKE="--smoke" ;;
    -h | --help)
      sed -n '3,17p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "unknown argument: ${argument}" >&2
      exit 2
      ;;
  esac
done

OUTPUT_DIR="${OUTPUT_DIR:-${REPO_ROOT}/run/compare-stacks}"
mkdir -p "${OUTPUT_DIR}"

# Every C++ compiler available, so "C++" is never one backend's result. An
# explicit CXX overrides the search and pins the run to that one compiler.
COMPILERS=()
if [[ -n "${CXX:-}" ]]; then
  COMPILERS+=("${CXX}")
else
  for candidate in g++ clang++; do
    if command -v "${candidate}" > /dev/null 2>&1; then
      COMPILERS+=("${candidate}")
    fi
  done
fi

if [[ ${#COMPILERS[@]} -eq 0 ]]; then
  echo "error: no C++ compiler found (looked for g++, clang++). Set CXX." >&2
  exit 1
fi

echo "==> building the Rust stack"
cargo build --release -p nexora-benchmark --features cpp

# --------------------------------------------------------------- conformance
echo
echo "==> conformance: does every build compute the same thing?"
./target/release/nexora-benchmark --conformance > "${OUTPUT_DIR}/conformance-rust.tsv"

for compiler in "${COMPILERS[@]}"; do
  tag="$(basename "${compiler}")"
  echo "    building c++ with ${tag}: $(${compiler} --version | head -1)"
  make -s -C benchmarks/cpp CXX="${compiler}" clean all
  ./benchmarks/cpp/build/conformance > "${OUTPUT_DIR}/conformance-${tag}.tsv"

  if ! diff -u "${OUTPUT_DIR}/conformance-rust.tsv" "${OUTPUT_DIR}/conformance-${tag}.tsv" \
    > "${OUTPUT_DIR}/conformance-${tag}.diff"; then
    echo
    echo "CONFORMANCE FAILED for ${tag} -- it disagrees with Rust. Nothing timed." >&2
    echo "(left: rust, right: ${tag})" >&2
    sed -n '3,$p' "${OUTPUT_DIR}/conformance-${tag}.diff" >&2
    exit 1
  fi
  echo "      matches Rust."
done

DIGESTS="$(wc -l < "${OUTPUT_DIR}/conformance-rust.tsv" | tr -d ' ')"
echo "    ${DIGESTS} digests, identical across $(( ${#COMPILERS[@]} + 1 )) builds."

# -------------------------------------------------------------------- timing
echo
echo "==> timing (conformance passed, so the numbers describe the same work)"
./target/release/nexora-benchmark ${SMOKE} \
  --scratch "${OUTPUT_DIR}/scratch" | tee "${OUTPUT_DIR}/timings-rust.txt"

for compiler in "${COMPILERS[@]}"; do
  tag="$(basename "${compiler}")"
  echo
  echo "--- c++ / ${tag} ---"
  make -s -C benchmarks/cpp CXX="${compiler}" clean all
  ./benchmarks/cpp/build/bench ${SMOKE} | tee "${OUTPUT_DIR}/timings-${tag}.md"
done

echo
echo "==> written to ${OUTPUT_DIR}"
echo "    Read the tables side by side, and read ADR-0009 before drawing a"
echo "    conclusion from them: this compares KERNELS, not engines -- and where"
echo "    two C++ builds disagree with each other by more than either disagrees"
echo "    with Rust, that row is telling you about a compiler, not a language."
