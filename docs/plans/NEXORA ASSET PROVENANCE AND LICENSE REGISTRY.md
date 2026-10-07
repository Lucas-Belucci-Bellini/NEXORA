# NEXORA — ASSET PROVENANCE AND LICENSE REGISTRY

## Purpose
Provide a machine-readable and reviewable record of where every release asset came from and under which terms it may be distributed.

## Asset record
```text
AssetID
Name
Type
Version
Author/Creator
Origin
SourceTool
SourceRepository
License
LicenseURL
ModificationStatus
AttributionRequired
RedistributionAllowed
CommercialUseAllowed
ReleaseStatus
Reviewer
ReviewDate
Notes
```

## Provenance classes
```text
ORIGINAL
OWNED_SOURCE
LICENSED_THIRD_PARTY
PUBLIC_DOMAIN
EDITOR_GENERATED
PROCEDURAL_DERIVATIVE
EXPERIMENTAL
REJECTED
```

## Registry rules
- No release asset may have unknown provenance.
- A third-party asset must have its license recorded.
- License compatibility is evaluated against the actual NEXORA distribution model.
- Attribution and notice obligations must be preserved.
- Asset modifications must remain traceable.
- Replaced assets remain in historical records even after removal from the current build.

## Source-of-truth
The registry is the authoritative answer to “where did this asset come from?”. Git history records changes; the provenance registry records origin and release clearance.

## AI-assisted content
When a tool contributes to an asset, record the tool/workflow and applicable usage terms. Tool use never removes the requirement for provenance and release review.

## Release states
```text
DRAFT
REVIEW
CLEARED
RESTRICTED
BLOCKED
REMOVED
```

## Audit
Release preparation should verify:
```text
all packaged assets
→ registry lookup
→ license check
→ attribution check
→ provenance check
→ package approval
```
