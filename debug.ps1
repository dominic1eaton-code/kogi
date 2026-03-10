

netstat -ano | findstr :8090
Get-Process -Name go -ErrorAction SilentlyContinue | Select-Object Id,ProcessName,Path
tasklist /FI "PID eq 37272

netstat -ano | findstr :8090
taskkill /PID <PID_FROM_NETSTAT> /F
go run ./kogi-services/go/gateway

$env:KOGI_GATEWAY_PORT="18090"
go run ./kogi-services/go/gateway

.\tools\build\run_go_service.ps1 -Service gateway -Port 18090


