#!/usr/bin/env python3
"""catalog-digest: the first visual generation catalog's digest columns, as a tool.

`content/first-generation/CATALOG.md` records, for every albedo, its size and
the FNV-1a 64 of its PNG's bytes, next to its resolution and format. This tool
computes those columns for any file, and answers over NEXORA tool IPC,
protocol version 1: one request per line on stdin, one answer per line on
stdout. The contract is written down in `README.md`, next to this file.

It is the language gate's "one cross-language tool call"
(`ENGINE ARCHITECTURE AND TECHNOLOGY DECISION.md` §17, ADR-0034): Python, the
language map's research and automation language, called by the Rust benchmark
across a language-neutral boundary, every answer checked against Rust's own.

Standard library only, by rule: the benchmark runs it as
`python -I -S -B catalog_digest.py`, and `-S` leaves site-packages off the
path, so a third-party import would fail on every machine, not only some.
"""

import sys
import time
import zlib

PROTOCOL = "nexora-tool/1"
TOOL = "catalog-digest/1"

PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"

# FNV-1a, 64-bit, from the published specification: the values
# `nexora_foundation::hashing` uses.
FNV_OFFSET_BASIS = 0xCBF29CE484222325
FNV_PRIME = 0x00000100000001B3
MASK_64 = 0xFFFFFFFFFFFFFFFF


def fnv1a64(data):
    """FNV-1a 64 of `data`, as the catalog's last column records it."""
    value = FNV_OFFSET_BASIS
    for byte in data:
        value = ((value ^ byte) * FNV_PRIME) & MASK_64
    return value


def ihdr(data):
    """(width, height, bit depth, colour type) from a PNG's header, or None.

    None unless the file begins with the PNG signature, its first chunk is a
    thirteen-byte IHDR, and that chunk's CRC matches: a header that fails its
    own checksum describes nothing.
    """
    if len(data) < 33 or data[:8] != PNG_SIGNATURE:
        return None
    if int.from_bytes(data[8:12], "big") != 13 or data[12:16] != b"IHDR":
        return None
    if zlib.crc32(data[12:29]) != int.from_bytes(data[29:33], "big"):
        return None
    width = int.from_bytes(data[16:20], "big")
    height = int.from_bytes(data[20:24], "big")
    return (width, height, data[24], data[25])


def digest(path):
    """The answer to `digest <path>`, and the time the work took, in ns."""
    started = time.perf_counter_ns()
    with open(path, "rb") as handle:
        data = handle.read()
    value = fnv1a64(data)
    header = ihdr(data)
    work_ns = time.perf_counter_ns() - started
    shown = "none" if header is None else "%d,%d,%d,%d" % header
    return "fnv1a64=0x%016x bytes=%d ihdr=%s work_ns=%d" % (value, len(data), shown, work_ns)


def one_line(text):
    """A message that cannot break the framing: one line, never empty."""
    flat = " ".join(str(text).split())
    return flat or "unknown"


def answer(raw):
    """The response line, without its terminator, for one request line."""
    try:
        line = raw.decode("utf-8")
    except UnicodeDecodeError:
        return "- err bad-request the request is not UTF-8"
    if line.endswith("\n"):
        line = line[:-1]
    if line.endswith("\r"):
        line = line[:-1]
    parts = line.split(" ", 2)
    request_id = parts[0]
    if not (request_id.isascii() and request_id.isdigit()) or len(request_id) > 20:
        return "- err bad-request a request begins with a decimal id"
    if len(parts) < 2 or not parts[1]:
        return "%s err bad-request no verb" % request_id
    verb = parts[1]
    if verb != "digest":
        return "%s err unknown-verb %s" % (request_id, one_line(verb))
    if len(parts) < 3 or not parts[2]:
        return "%s err bad-request digest needs a path" % request_id
    try:
        return "%s ok %s" % (request_id, digest(parts[2]))
    except OSError as error:
        return "%s err unreadable %s" % (request_id, one_line(error.strerror or error))


def serve(stdin, stdout):
    """Speak the protocol until the client closes stdin."""
    stdout.write(("%s %s\n" % (PROTOCOL, TOOL)).encode("ascii"))
    stdout.flush()
    for raw in iter(stdin.readline, b""):
        stdout.write(answer(raw).encode("utf-8") + b"\n")
        stdout.flush()
    return 0


if __name__ == "__main__":
    try:
        sys.exit(serve(sys.stdin.buffer, sys.stdout.buffer))
    except BrokenPipeError:
        # The client went away mid-answer: nobody is left to tell.
        sys.exit(1)
