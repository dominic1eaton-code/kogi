# Kogi Wallet Service

Go microservice that exposes wallet snapshots by calling the Rust `kogi_wallet` library over the C ABI.

**Run**
```bash
cd kogi-wallet/service
go run .
```

**Environment**
- `KOGI_WALLET_PORT` (default `9013`)
- `KOGI_WALLET_DLL` to point at a custom `kogi_wallet.dll`
- `KOGI_WALLET_SYSTEM_BIN` if using the fallback exe bridge
