# Kogi Marketplace Backend

The Marketplace backend powers the Marketplace UI surfaces in `kogi-ui` and is anchored on the Kogi Portfolio Master Spreadsheet. Listings are projections of portfolio items rather than a separate source of truth.

**Source Of Truth**
- All marketplace items originate as Portfolio components in `kogi-portfolio`.
- The Master Portfolio Spreadsheet remains the canonical store for names, status, visibility, tags, and ownership.
- Marketplace-specific fields can be supplied through portfolio metadata (row `ext` or component metadata properties) and are merged into listings at runtime.

**Listing Derivation Rules**
- Include portfolio items whose `item_type` is one of: `Gig`, `Contract`, `Job`, `Task`, `Asset`, `Artifact`, `Resource`, `Campaign`, `Grant`, `Investment`, `Template`, `Service`, `Bundle`.
- Include portfolio items tagged with `marketplace` or containing `ext.marketplace` metadata.
- Listing ID equals the portfolio component ID so version history and governance remain centralized.
- Listing status, visibility, and analytics are mapped directly from the portfolio row.

**Rust Core (`kogi-marketplace`)**
- `MarketplaceSystem` composes a `PortfolioSystem` and derives listings from portfolio rows.
- FFI functions expose snapshots and listings for the Go service.

**Go Service (`kogi-network/services/marketplace`)**
- `GET /health`
- `GET /api/v1/marketplace/runtime`
- `POST /api/v1/marketplace/init`
- `POST /api/v1/marketplace/refresh`
- `GET /api/v1/marketplace/state`
- `GET /api/v1/marketplace/listings`
- `GET /api/v1/marketplace/listings/owner/{owner_id}`
- `GET /api/v1/marketplace/listings/{listing_id}`
- `POST /api/v1/marketplace/listings/publish`
- `GET /api/v1/marketplace/dashboard`
- `GET /api/v1/marketplace/market/overview`
- `GET /api/v1/marketplace/market/browse`
- `GET /api/v1/marketplace/market/labor`
- `GET /api/v1/marketplace/market/grants`
- `GET /api/v1/marketplace/market/crm`
- `GET /api/v1/marketplace/listings/catalog`
- `GET /api/v1/marketplace/listings/detail`
- `GET /api/v1/marketplace/listings/mine`
- `GET /api/v1/marketplace/campaigns/overview`
- `GET /api/v1/marketplace/campaigns/discover`

**Reference Specs**
- `kogi-marketplace-exchange-sdd-updated.md` documents the marketplace and exchange feature set that the backend aligns to.
