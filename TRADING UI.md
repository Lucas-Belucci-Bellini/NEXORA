# NEXORA — TRADING UI

> Trading UI presents available offers and creates explicit trade intents. Economy owns markets; Inventory owns item state; server owns authority.

## Flow
```text
Market / Seller State
→ ViewModel
→ Offer Selection
→ Trade Command
→ Economy Validation
→ Transaction
→ Event
→ UI Update
```

## Screens
Market board, buy/sell, orders, contracts, price history, stock, route status and trade confirmation.

## Offers
An offer includes seller, item/fluid/resource reference, quantity, quality, price, currency, expiry and delivery conditions.

## Atomicity
Trade execution uses transaction IDs and reserves resources before commit. Duplicate submission must be idempotent.

## Permissions
Ownership, faction access, licenses and market restrictions are validated server-side.

## API sketch
```ts
interface ITradingUI {
  openMarket(id: MarketID): void;
  quote(request: TradeQuoteRequest): TradeQuote;
  submit(request: TradeRequest): Promise<TradeResult>;
}
```

## Accessibility / Localization
All text is localized and all actions expose semantic UI commands.

## Tests
Stale offer, price change, duplicate click, insufficient funds, insufficient stock, disconnect during trade and successful commit.

## Invariants
- UI never edits balances directly.
- A committed trade cannot be committed twice.
