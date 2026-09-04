# NEXORA — SECURITY THREAT MODEL

## Principle
The client, mods, scripts and network peers are potential sources of invalid input. Trust is explicit, never assumed.

## Trust zones
```text
TRUSTED ENGINE
TRUSTED SERVER
AUTHORIZED NATIVE MOD
SANDBOXED MOD/SCRIPT
CLIENT
REMOTE PEER
UNTRUSTED CONTENT
```

## Threat classes
```text
state forgery
command abuse
packet flooding
duplication
resource exhaustion
save tampering
script escape
native mod corruption
identity spoofing
permission escalation
content injection
replay/desync abuse
```

## Security pipeline
```text
INPUT
→ PARSE
→ VALIDATE
→ AUTHORIZE
→ RATE LIMIT
→ EXECUTE
→ RECORD
```

## Rules
- Server validates authoritative gameplay commands.
- Clients never directly mutate authoritative world state.
- Mods and scripts receive only declared capabilities.
- Resource quotas are mandatory for untrusted execution.
- Administrative actions are authenticated and auditable.
- Persistence input is validated before entering runtime state.
- Network messages have size, frequency and semantic validation limits.

## Security by subsystem
Every major subsystem documents:
```text
trust boundary
allowed callers
privileged operations
quotas
validation
logging
recovery behavior
```

## Release security gate
A release candidate must include a threat review for network, persistence, scripting, mods and distribution surfaces.
