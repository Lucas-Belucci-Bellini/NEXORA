# NEXORA — COMMUNICATION SYSTEM

> Communication models how information moves through the world. It is separate from Networking: Networking transports game protocol data between computers; Communication is a simulated world capability.

## Model
```text
Message / Signal
→ Medium
→ Network
→ Propagation
→ Delay / Loss
→ Recipient
→ Knowledge
```

## Media
Messenger, mail, road courier, telegraph, radio, wired network, satellite, quantum/fictional mod-defined channels and dimensional links.

## Core concepts
`CommunicationNetwork`, `Channel`, `Message`, `Signal`, `Endpoint`, `Route`, `DeliveryPolicy`, `CommunicationDelay`.

## Delay
Communication can have physical/logical delay based on distance, medium, infrastructure, technology and dimension. The system must not assume instant global knowledge.

## Reliability
Messages can be guaranteed, retryable, lossy, broadcast or encrypted according to definitions.

## Knowledge integration
Delivered messages become observations/evidence available to the Knowledge System. A recipient may misunderstand or reject them.

## Civilization integration
Governments, merchants, researchers, factions and settlements can own communication networks.

## Space
Interplanetary communication uses Space topology and propagation constraints. Interstellar or dimensional communication is data-driven rather than hardcoded.

## API sketch
```ts
interface ICommunicationSystem {
  send(message: CommunicationMessage): DeliveryHandle;
  createNetwork(definition: CommunicationNetworkDefinition): NetworkID;
  queryRoute(request: CommunicationRouteRequest): CommunicationRoute;
  estimateDelay(route: CommunicationRoute): Duration;
}
```

## Security
Authentication, authorization and encryption are world capabilities where appropriate; server protocol security remains Networking/Security responsibility.

## Persistence
Persist durable networks, endpoints and queued messages that are explicitly persistent. Short-lived signals remain transient.

## Debug
`nexora comm networks`, `route`, `queue`, `trace`, `delay`.

## Tests
Delayed news, broken network, relay restoration, faction-only channels, satellite link and multi-hop message delivery.

## Invariants
- Communication does not modify Knowledge directly without a delivery event.
- Simulated communication is distinct from multiplayer transport.
- Distance and infrastructure can matter.
