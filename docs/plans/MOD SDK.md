# NEXORA — MOD SDK

> The Mod SDK packages the authoring experience for NEXORA: templates, schemas, CLI, validators, package builder and local test tooling.

## Runtime vs SDK
Mod Runtime loads/executes. SDK helps authors create, validate, test and package.

## Components
CLI, templates, API bindings, schemas, validators, package builder, local test server/client, debug tools and documentation generator.

## Flow
`nexora mod init → author → validate → test → build → package → fingerprint`.

## Validation
Manifest, namespace, dependencies, permissions, assets, licenses/provenance and compatibility.

## Security
Development shortcuts never become runtime privileges. Final packages are revalidated by Mod Runtime/Server.

## API
```ts
interface IModSDK {
  init(template: ModTemplate): Project;
  validate(project: Project): ValidationReport;
  build(project: Project): ModPackage;
  test(project: Project, scenario: TestScenario): TestResult;
}
```

## Invariants
SDK artifacts remain compatible with Runtime validation and are independently fingerprintable.
