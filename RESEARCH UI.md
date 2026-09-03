# NEXORA — RESEARCH UI

> Research UI presents questions, evidence, hypotheses, projects and discoveries; Research/Knowledge remain authoritative.

## Views
Questions, hypotheses, evidence, experiments, projects, institutions, publications, technology outcomes and knowledge graph.

## Flow
`Snapshot → ViewModel → User intent → Command → Research → Event → UI update`.

## Evidence
Show source, confidence, status and provenance; uncertainty must remain visible.

## API
```ts
interface IResearchUI {
  open(project?: ResearchProjectID): void;
  snapshot(scope: ResearchScope): ResearchViewModel;
  dispatch(action: ResearchUIAction): void;
}
```

## Tests
Stale project, failed funding command, permissions, localization and uncertainty display.
