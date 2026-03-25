# Kogi Wallet

The `kogi-wallet` crate implements the kogi-bank wallet backend layer and exposes UI-friendly snapshots for banking, ledger, escrow, invoices, investments, funding, benefits, grants, group economics, campaigns, debts, and taxes. All wallet and account resources are also registered into the portfolio system (`kogi-portfolio`) as portfolio components.

**Key Modules**
- `src/system.rs` - `WalletSystem` orchestration + portfolio linkage.
- `src/model.rs` - Wallet/account/transaction domain models.
- `src/snapshots.rs` - UI snapshot payloads (mirrors Angular wallet pages).
- `src/ffi.rs` - C ABI bridge for Go services (`kogi_wallet` DLL).

**Build (Rust)**
```bash
cd kogi-wallet
cargo build
```

**Build DLL (Rust cdylib)**
```bash
cd kogi-wallet
cargo build --release
```

The build emits `kogi_wallet.dll` (Windows) in `kogi-wallet/target/release`.

**Service API (Curl)**
Run the Go service first (see `kogi-wallet/service`). Examples:

```bash
# Health
curl http://localhost:9013/health

# Init
curl -X POST http://localhost:9013/api/v1/wallet/init ^
  -H "Content-Type: application/json" ^
  -d "{\"owner_id\":\"00000000-0000-0000-0000-000000000001\"}"

# Dashboard
curl http://localhost:9013/api/v1/wallet/dashboard

# Banking
curl http://localhost:9013/api/v1/wallet/banking

# Ledger
curl http://localhost:9013/api/v1/wallet/ledger

# Escrow
curl http://localhost:9013/api/v1/wallet/escrow

# Invoices
curl http://localhost:9013/api/v1/wallet/invoices

# Investments
curl http://localhost:9013/api/v1/wallet/investments

# Funding
curl http://localhost:9013/api/v1/wallet/funding

# Benefits
curl http://localhost:9013/api/v1/wallet/benefits

# Grants
curl http://localhost:9013/api/v1/wallet/grants

# Group Economics
curl http://localhost:9013/api/v1/wallet/group-economics

# Campaigns
curl http://localhost:9013/api/v1/wallet/campaigns

# Debts
curl http://localhost:9013/api/v1/wallet/debts

# Taxes
curl http://localhost:9013/api/v1/wallet/taxes

# Wallets + Accounts
curl http://localhost:9013/api/v1/wallet/wallets
curl http://localhost:9013/api/v1/wallet/accounts
curl http://localhost:9013/api/v1/wallet/wallets/overview
```
