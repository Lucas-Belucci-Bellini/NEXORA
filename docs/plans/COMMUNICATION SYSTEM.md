# NEXORA — COMMUNICATION SYSTEM

> Communication simulates how information moves through the world. It is separate from multiplayer Networking.

## Model
`Message/Signal → Medium → Network → Propagation → Delay/Loss → Recipient → Knowledge`.

## Media
Courier, mail, telegraph, radio, wired networks, satellites and mod-defined/dimensional channels.

## Concepts
`CommunicationNetwork`, `Channel`, `Message`, `Signal`, `Endpoint`, `Route`, `DeliveryPolicy`, `CommunicationDelay`.

## Rules
Distance, infrastructure, technology and medium may introduce delay and loss. Delivered messages become observations/evidence; recipients can misunderstand them.

## Integration
Knowledge/Research, Civilization, Social/Factions, Sensors, Space and World Events.

## API
```ts
interface ICommunicationSystem {
  send(message: CommunicationMessage): DeliveryHandle;
  createNetwork(definition: CommunicationNetworkDefinition): NetworkID;
  queryRoute(request: CommunicationRouteRequest): CommunicationRoute;
  estimateDelay(route: CommunicationRoute): Duration;
}
```

## Security
World communication permissions are separate from multiplayer transport security. Sensitive channels require explicit authority.

## Tests
Delayed news, broken relay, multi-hop routing, satellite link and faction-restricted communication.
