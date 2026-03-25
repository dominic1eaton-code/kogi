# Kogi Portfolio Service

Go microservice that exposes portfolio snapshots by calling the Rust `kogi_portfolio` library over the C ABI.

**Run**
```bash
cd kogi-portfolio/service
go run .
```

**Environment**
- `KOGI_PORTFOLIO_PORT` (default `9012`)
- `KOGI_PORTFOLIO_DLL` to point at a custom `kogi_portfolio.dll`
- `KOGI_PORTFOLIO_SYSTEM_BIN` if using the fallback exe bridge
