# NEXORA — RESEARCH UI

> Research UI visualizes questions, evidence, hypotheses, projects and discoveries. Research remains authoritative; UI only presents state and dispatches commands.

## Views
```text
Research Questions
Hypotheses
Evidence
Experiments
Projects
Institutions
Publications
Technology Outcomes
Knowledge Graph
```

## Flow
```text
Research Snapshot
→ ViewModel
→ User intent
→ Research Command
→ Research System
→ Event
→ Updated ViewModel
```

## Evidence
Show source, confidence, status and provenance without converting uncertainty into false certainty.

## Projects
Display resources, researchers, facilities, progress, blockers and expected outcomes.

## Discovery
New discoveries may open technology or quests through existing systems; UI does not grant capabilities directly.

## API sketch
```ts
interface IResearchUI {
  open(project?: ResearchProjectID): void;
  snapshot(scope: ResearchScope): ResearchViewModel;
  dispatch(action: ResearchUIAction): void;
}
```

## Tests
Stale project state, permission limits, failed funding command, localization and uncertainty display.

## Invariants
- UI never writes Knowledge/Research state directly.
- Evidence uncertainty remains visible.
