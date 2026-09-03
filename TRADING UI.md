# NEXORA — TRADING UI

> Trading UI presents market offers and submits explicit trade intents. Economy owns markets and transactions.

## Flow
`Market State → ViewModel → Offer Selection → Trade Command → Economy Validation → Transaction → Event → UI`.

## Offer
Seller, item/fluid/resource, quantity, quality, price, currency, expiry and delivery terms.

## Safety
Trades reserve resources and use transaction IDs. Stale offers and duplicate submissions are rejected/idempotent.

## API
```ts
interface ITradingUI {
  openMarket(id: MarketID): void;
  quote(request: TradeQuoteRequest): TradeQuote;
  submit(request: TradeRequest): Promise<TradeResult>;
}
```

## Integration
Economy, Inventory, Item, Logistics, Civilization, Localization and Accessibility.

## Tests
Stale price, insufficient funds/stock, disconnect, duplicate click and successful commit.
