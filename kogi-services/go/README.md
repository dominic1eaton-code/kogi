# kogi-services-go

Go networking and microservice infrastructure for Kogi MVP.

## Services
- `gateway` (port 8090)
- `services/auth` (port 9001)
- `services/portfolio` (port 9002)
- `services/exchange` (port 9004)
- `services/ims` (port 9005)
- `services/office` (port 9006)
- `lib/eventbus` in-memory pub/sub adapter

## Build
```powershell
go build ./kogi-services/go/...
```

## Run examples
```powershell
go run ./kogi-services/go/gateway
go run ./kogi-services/go/services/auth
go run ./kogi-services/go/services/office
```
