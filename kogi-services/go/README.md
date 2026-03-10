# kogi-services-go

Go networking, pub/sub, message-gateway, and component communication infrastructure for Kogi MVP.

## Services
- `gateway` (port 8090): centralized message/event/data gateway management, pub/sub, routing, component registry, and engine ingest mirroring.
- `services/auth` (port 9001)
- `services/portfolio` (port 9002)
- `services/exchange` (port 9004)
- `services/ims` (port 9005)
- `services/office` (port 9006)

## Libraries
- `lib/eventbus`: in-memory pub/sub bus with history, topic stats, wildcard subscriptions, and source/target metadata.
- `lib/mesh`: component registry + topic routes + component-to-component message routing history.

## Gateway API additions
- `POST /api/v1/gateway/components/register`
- `GET /api/v1/gateway/components`
- `POST /api/v1/gateway/pubsub/publish`
- `GET /api/v1/gateway/pubsub/history`
- `GET /api/v1/gateway/pubsub/topics`
- `POST /api/v1/gateway/network/send`
- `GET /api/v1/gateway/network/history`
- `GET /api/v1/gateway/network/routes`
- `GET /api/v1/gateway/engine/stream`

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
