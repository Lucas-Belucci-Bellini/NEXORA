# NEXORA — MOD SDK

> The Mod SDK packages the public NEXORA development experience: project templates, schemas, CLI, validators, local test runtime and documentation.

## Runtime vs SDK
`Mod Runtime` loads and executes mods. `Mod SDK` helps authors create, validate, package and test them.

## SDK contents
```text
CLI
Templates
Schemas
API bindings
Content validator
Package builder
Local test server/client
Debug tools
Docs generator
Example mods
```

## Project flow
```text
nexora mod init
→ author
→ nexora mod validate
→ test
→ build
→ package
→ fingerprint
```

## Templates
Content mod, scripted mod, library mod, worldgen mod, UI mod, server mod and total-conversion style projects share the same manifest contract.

## API discovery
Generate typed bindings and registry summaries from the selected NEXORA API version.

## Validation
Check manifest, namespaces, dependencies, schemas, permissions, asset references, licenses/provenance and package limits.

## Local testing
SDK can launch isolated development worlds and headless servers. Development resources are separated from real saves.

## Packaging
Output includes manifest, resources, registry snapshot/fingerprint, dependency metadata and optional debug symbols.

## Compatibility
Mod API version and game version are explicit. SDK can produce a compatibility report before packaging.

## Security
The SDK must not grant development privileges to release mods. Packages are still validated by the Runtime/Server.

## API sketch
```ts
interface IModSDK {
  init(template: ModTemplate): Project;
  validate(project: Project): ValidationReport;
  build(project: Project): ModPackage;
  test(project: Project, scenario: TestScenario): TestResult;
}
```

## Invariants
- SDK artifacts remain compatible with Runtime validation.
- Development shortcuts never become implicit runtime permissions.
