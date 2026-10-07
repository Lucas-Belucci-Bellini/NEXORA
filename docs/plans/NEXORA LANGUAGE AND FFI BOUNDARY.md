# NEXORA — LANGUAGE AND FFI BOUNDARY

## Principle
Language boundaries exist to isolate responsibilities, not to split hot code arbitrarily.

## Current architectural direction
```text
RUST
→ Core
→ ECS / simulation
→ voxel/world
→ jobs
→ persistence
→ networking core
→ server

RUST / C OR C++
→ platform-native graphics integration where justified
→ RHI backends

TYPESCRIPT / RUST
→ editor
→ SDK
→ developer tooling

PYTHON
→ research
→ offline generation/analysis
→ automation
```

This is a candidate architecture until the technology benchmark freezes the final stack.

## Boundary rules
- Hot simulation loops must not cross FFI repeatedly.
- Ownership crossing FFI must be explicit.
- Memory allocation/free responsibility must be defined on both sides.
- ABI contracts must use stable representations only where long-term ABI stability is required.
- Handles should be preferred over sharing mutable object graphs.
- Callbacks across boundaries must define thread and lifetime rules.
- Errors must cross the boundary as structured status/results, not language-specific exceptions.

## Interoperability layers
```text
same-process FFI
IPC
file/resource contract
network protocol
```

Use the lowest-complexity mechanism that satisfies the requirement.

## Anti-pattern
```text
Rust → C++ → Python → TypeScript → Rust
```
for a per-frame or per-tick operation.

## Preferred pattern
```text
Rust simulation
    ↓ batch result
stable boundary
    ↓
Tools / graphics / analysis
```

## Final lock
The final language map must be recorded only after the benchmark defined by the Technology Benchmark Plan.
