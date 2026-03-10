

netstat -ano | findstr :8090
Get-Process -Name go -ErrorAction SilentlyContinue | Select-Object Id,ProcessName,Path
tasklist /FI "PID eq 37272

netstat -ano | findstr :8090
taskkill /PID <PID_FROM_NETSTAT> /F
go run ./kogi-services/go/gateway

$env:KOGI_GATEWAY_PORT="18090"
go run ./kogi-services/go/gateway

.\tools\build\run_go_service.ps1 -Service gateway -Port 18090


# git add .;git commit -am "incomplete platform update codex commans"
# git ls-files | xargs wc -l
# Valkey/redis memory cache system for caching frequently accessed data and reducing database load.
# kafka for real-time data streaming and event-driven architecture, enabling efficient communication between microservices and components.
