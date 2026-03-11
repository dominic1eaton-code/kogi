package main

import (
    "bytes"
    "context"
    "encoding/json"
    "fmt"
    "io"
    "log"
    "net/http"
    "net/url"
    "os"
    "os/exec"
    "path/filepath"
    "sort"
    "strconv"
    "strings"
    "sync"
    "time"

    "kogi.services/lib/ops"
)

type ingestRecord struct {
    ID         string `json:"id"`
    Payload    string `json:"payload"`
    ReceivedAt string `json:"received_at"`
}

type gatewayPublishRequest struct {
    Topic    string            `json:"topic"`
    Payload  string            `json:"payload"`
    Source   string            `json:"source"`
    Target   string            `json:"target"`
    Metadata map[string]string `json:"metadata"`
}

var (
    ingestMu    sync.Mutex
    ingestLog   []ingestRecord
    controlMode = "stopped"
    engineRuntime *ops.Runtime
)

func main() {
    engineRuntime = ops.Init("engine-service")
    engineRuntime.State("init")
    mux := http.NewServeMux()
    engineRuntime.State("configure")
    engineRuntime.WatchSignals()

    mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
        writeJSON(w, http.StatusOK, map[string]string{"status": "ok", "service": "engine-service"})
    })

    mux.HandleFunc("/api/v1/engine/runtime", func(w http.ResponseWriter, r *http.Request) {
        ingestMu.Lock()
        count := len(ingestLog)
        mode := controlMode
        ingestMu.Unlock()

        writeJSON(w, http.StatusOK, map[string]interface{}{
            "service":         "engine-service",
            "component_id":    "kogi.services.engine",
            "module_id":       "kogi.engine",
            "gateway":         gatewayURL(),
            "network_manager": "kogi-go-network",
            "control_mode":    mode,
            "ingest_count":    count,
            "system_bridge":   "scala",
            "system_endpoint": engineEndpoint(),
            "system_command":  engineCommandHint(),
            "grpc":            grpcModeSummary(),
            "engine_subengines": []string{
                "AnalyticsEngine",
                "TelemetryEngine",
                "RecommendationEngine",
                "OptimizationEngine",
                "SearchEngine",
                "QueryEngine",
            },
            "publishes": []string{
                "engine.flow.processed",
                "engine.snapshot.generated",
            },
            "subscribes": []string{
                "engine.ingest",
                "host.orchestrator.booted",
            },
            "engine_stream": "engine.ingest",
        })
    })

    mux.HandleFunc("/api/v1/engine/control", func(w http.ResponseWriter, r *http.Request) {
        if r.Method != http.MethodPost {
            writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
            return
        }

        var payload map[string]string
        _ = json.NewDecoder(r.Body).Decode(&payload)
        action := payload["action"]
        if action == "" {
            action = "start"
        }

        ingestMu.Lock()
        controlMode = action
        ingestMu.Unlock()
        if engineRuntime != nil {
            engineRuntime.Status("control_mode", action)
            switch strings.ToLower(action) {
            case "pause", "paused":
                engineRuntime.State("paused")
            case "stop", "shutdown", "shutting_down":
                engineRuntime.State("shutting_down")
            case "start", "running":
                engineRuntime.State("running")
            }
        }

        engineResponse, forwarded, forwardErr := forwardEngineControl(action)
        gatewayResponse, gatewayErr := publishGatewayMessage(gatewayPublishRequest{
            Topic:   "engine.control.requested",
            Payload: string(mustJSON(map[string]string{"action": action})),
            Source:  "kogi.services.engine",
            Target:  "kogi.engine",
            Metadata: map[string]string{
                "control_mode": action,
            },
        })
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "status":  "ok",
            "service": "engine-service",
            "action":  action,
            "forwarded": forwarded,
            "forward_error": forwardErr,
            "engine_response": engineResponse,
            "gateway_response": gatewayResponse,
            "gateway_error": gatewayErr,
        })
    })

    mux.HandleFunc("/api/v1/engine/ingest", func(w http.ResponseWriter, r *http.Request) {
        if r.Method != http.MethodPost {
            writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
            return
        }

        raw, _ := io.ReadAll(r.Body)
        payload := strings.TrimSpace(string(raw))
        if payload == "" {
            payload = "{}"
        }
        payloadMap := parsePayloadMap(payload)

        ingestMu.Lock()
        ingestLog = append(ingestLog, ingestRecord{
            ID:         "ingest-" + time.Now().UTC().Format("20060102150405"),
            Payload:    payload,
            ReceivedAt: time.Now().UTC().Format(time.RFC3339),
        })
        count := len(ingestLog)
        ingestMu.Unlock()
        if engineRuntime != nil {
            engineRuntime.Status("ingest_count", fmt.Sprintf("%d", count))
        }

        ingestOptions := map[string]string{
            "topic":     "engine.ingest",
            "source":    "kogi.services.engine",
            "target":    "kogi.engine",
            "flow-id":   fmt.Sprintf("flow-%d", time.Now().UnixMilli()),
            "timestamp-ms": fmt.Sprintf("%d", time.Now().UnixMilli()),
        }
        engineResponse, forwarded, forwardErr := forwardEngineIngest(payload, payloadMap, ingestOptions)
        gatewayResponse, gatewayErr := publishGatewayMessage(gatewayPublishRequest{
            Topic:    "engine.ingest",
            Payload:  payload,
            Source:   "kogi.services.engine",
            Target:   "kogi.engine",
            Metadata: map[string]string{"ingest_count": fmt.Sprintf("%d", count)},
        })
        writeJSON(w, http.StatusAccepted, map[string]interface{}{
            "status":       "queued",
            "service":      "engine-service",
            "ingest_count": count,
            "forwarded":    forwarded,
            "forward_error": forwardErr,
            "engine_response": engineResponse,
            "gateway_response": gatewayResponse,
            "gateway_error": gatewayErr,
        })
    })

    mux.HandleFunc("/api/v1/engine/snapshot", func(w http.ResponseWriter, r *http.Request) {
        ingestMu.Lock()
        count := len(ingestLog)
        mode := controlMode
        ingestMu.Unlock()

        engineResponse, forwarded, forwardErr := forwardEngineSnapshot()
        writeJSON(w, http.StatusOK, map[string]interface{}{
            "engine":       "kogi-engine",
            "control_mode": mode,
            "ingest_count": count,
            "status":       "active",
            "forwarded":    forwarded,
            "forward_error": forwardErr,
            "engine_response": engineResponse,
        })
    })

    mux.HandleFunc("/api/v1/engine/messages", func(w http.ResponseWriter, r *http.Request) {
        if r.Method == http.MethodGet {
            limit := queryInt(r, "limit", 100)
            topic := r.URL.Query().Get("topic")
            history, err := gatewayHistory(limit, topic)
            if err != nil {
                writeJSON(w, http.StatusBadGateway, map[string]string{"error": err.Error()})
                return
            }
            writeJSON(w, http.StatusOK, map[string]interface{}{
                "gateway": gatewayURL(),
                "history": history,
            })
            return
        }

        if r.Method != http.MethodPost {
            writeJSON(w, http.StatusMethodNotAllowed, map[string]string{"error": "method_not_allowed"})
            return
        }

        var request gatewayPublishRequest
        if err := json.NewDecoder(r.Body).Decode(&request); err != nil {
            writeJSON(w, http.StatusBadRequest, map[string]string{"error": "invalid_json"})
            return
        }
        if request.Topic == "" {
            writeJSON(w, http.StatusBadRequest, map[string]string{"error": "topic_required"})
            return
        }
        request.Source = defaultIfEmpty(request.Source, "kogi.services.engine")
        request.Target = defaultIfEmpty(request.Target, "kogi.engine")

        gatewayResponse, gatewayErr := publishGatewayMessage(request)
        engineResponse := map[string]interface{}(nil)
        engineForwarded := false
        engineError := ""

        if shouldForwardToEngine(request.Topic, request.Target) {
            engineResponse, engineForwarded, engineError = forwardEngineFromMessage(request)
        }

        writeJSON(w, http.StatusAccepted, map[string]interface{}{
            "status":          "accepted",
            "gateway_response": gatewayResponse,
            "gateway_error":   gatewayErr,
            "engine_response": engineResponse,
            "engine_forwarded": engineForwarded,
            "engine_error":    engineError,
        })
    })

    addr := resolveAddr("9014", "KOGI_ENGINE_PORT")
    engineRuntime.State("running")
    engineRuntime.Status("ok", "listening="+addr)
    log.Printf("engine-service listening on %s", addr)
    log.Fatal(http.ListenAndServe(addr, ops.WithHTTPDebug(engineRuntime, mux)))
}

func forwardEngineControl(action string) (map[string]interface{}, bool, string) {
    if engineRuntime != nil {
        engineRuntime.Message("send", "engine.control", "kogi.services.engine", "kogi.engine", action)
    }
    if resp, attempted, err, required := grpcAttempt("Control", grpcControlRequest(action)); attempted {
        if err == "" {
            return resp, true, ""
        }
        if required {
            return nil, true, err
        }
    }
    response, err := callEngineCLI("control", nil, map[string]string{"mode": action})
    if err == nil {
        return response, true, ""
    }
    endpoint := engineEndpoint()
    if endpoint == "" {
        return nil, false, err.Error()
    }
    raw, _ := json.Marshal(map[string]string{"action": action})
    httpResponse, httpErr := callEngineHTTP(endpoint+"/control", raw)
    if httpErr != nil {
        return nil, true, httpErr.Error()
    }
    return httpResponse, true, ""
}

func forwardEngineIngest(payload string, payloadMap map[string]string, options map[string]string) (map[string]interface{}, bool, string) {
    if engineRuntime != nil {
        engineRuntime.Message("send", "engine.ingest", "kogi.services.engine", "kogi.engine", payload)
    }
    options = grpcFlowDefaults(options)
    grpcTopic := defaultIfEmpty(options["topic"], "engine.ingest")
    grpcSource := defaultIfEmpty(options["source"], "kogi.services.engine")
    grpcTarget := defaultIfEmpty(options["target"], "kogi.engine")
    if resp, attempted, err, required := grpcAttempt(
        "Ingest",
        grpcIngestRequest(grpcTopic, grpcSource, grpcTarget, payloadMap, options),
    ); attempted {
        if err == "" {
            return resp, true, ""
        }
        if required {
            return nil, true, err
        }
    }
    response, err := callEngineCLI("ingest", payloadMap, options)
    if err == nil {
        return response, true, ""
    }
    endpoint := engineEndpoint()
    if endpoint == "" {
        return nil, false, err.Error()
    }
    httpResponse, httpErr := callEngineHTTP(endpoint+"/ingest", []byte(payload))
    if httpErr != nil {
        return nil, true, httpErr.Error()
    }
    return httpResponse, true, ""
}

func forwardEngineSnapshot() (map[string]interface{}, bool, string) {
    if engineRuntime != nil {
        engineRuntime.Message("send", "engine.snapshot", "kogi.services.engine", "kogi.engine", "{}")
    }
    if resp, attempted, err, required := grpcAttempt(
        "Snapshot",
        grpcSnapshotRequest("kogi-host-001", 5*60*1000),
    ); attempted {
        if err == "" {
            return resp, true, ""
        }
        if required {
            return nil, true, err
        }
    }
    response, err := callEngineCLI("snapshot", nil, nil)
    if err == nil {
        return response, true, ""
    }
    endpoint := engineEndpoint()
    if endpoint == "" {
        return nil, false, err.Error()
    }
    httpResponse, httpErr := callEngineHTTP(endpoint+"/snapshot", nil)
    if httpErr != nil {
        return nil, true, httpErr.Error()
    }
    return httpResponse, true, ""
}

func forwardEngineFromMessage(request gatewayPublishRequest) (map[string]interface{}, bool, string) {
    if engineRuntime != nil {
        engineRuntime.Message("receive", request.Topic, request.Source, request.Target, request.Payload)
    }
    topic := strings.ToLower(request.Topic)
    payloadMap := parsePayloadMap(request.Payload)

    if strings.Contains(topic, "control") {
        action := payloadMap["action"]
        if action == "" {
            action = strings.TrimSpace(request.Payload)
        }
        if action == "" {
            action = "start"
        }
        return forwardEngineControl(action)
    }

    options := map[string]string{
        "topic":        request.Topic,
        "source":       request.Source,
        "target":       request.Target,
        "flow-id":      fmt.Sprintf("flow-%d", time.Now().UnixMilli()),
        "timestamp-ms": fmt.Sprintf("%d", time.Now().UnixMilli()),
    }
    return forwardEngineIngest(request.Payload, payloadMap, options)
}

func callEngineCLI(action string, payload map[string]string, options map[string]string) (map[string]interface{}, error) {
    cmdPath, args, workdir, err := resolveEngineCLICommand(action, payload, options)
    if err != nil {
        return nil, err
    }

    ctx, cancel := context.WithTimeout(context.Background(), 4*time.Second)
    defer cancel()

    cmd := exec.CommandContext(ctx, cmdPath, args...)
    if workdir != "" {
        cmd.Dir = workdir
    }
    output, err := cmd.CombinedOutput()
    if err != nil {
        return nil, fmt.Errorf("engine cli failed: %w %s", err, strings.TrimSpace(string(output)))
    }
    return parseJSONResponse(string(output)), nil
}

func resolveEngineCLICommand(action string, payload map[string]string, options map[string]string) (string, []string, string, error) {
    if cli := strings.TrimSpace(os.Getenv("KOGI_ENGINE_CLI")); cli != "" {
        return cli, buildEngineArgs(action, payload, options), "", nil
    }

    if path, err := exec.LookPath("kogi-engine-cli"); err == nil {
        return path, buildEngineArgs(action, payload, options), "", nil
    }

    if sbtPath, err := exec.LookPath("sbt"); err == nil {
        workdir := ""
        if root, ok := findRepoRoot(); ok {
            workdir = filepath.Join(root, "kogi-engine")
        }
        cmdString := buildSbtCommand(buildEngineArgs(action, payload, options))
        return sbtPath, []string{"-Dsbt.log.noformat=true", cmdString}, workdir, nil
    }

    return "", nil, "", fmt.Errorf("kogi engine CLI not found (set KOGI_ENGINE_CLI or install sbt)")
}

func buildEngineArgs(action string, payload map[string]string, options map[string]string) []string {
    args := []string{"--action", action}

    if options != nil {
        for _, key := range sortedKeys(options) {
            if key == "action" {
                continue
            }
            args = append(args, "--"+key, options[key])
        }
    }

    if payload != nil {
        for _, key := range sortedKeys(payload) {
            args = append(args, "--payload", key+"="+payload[key])
        }
    }

    return args
}

func buildSbtCommand(args []string) string {
    formatted := make([]string, 0, len(args))
    for _, arg := range args {
        formatted = append(formatted, quoteForSbt(arg))
    }
    return "runMain kogi.engine.KogiEngineCli " + strings.Join(formatted, " ")
}

func quoteForSbt(value string) string {
    if value == "" {
        return "\"\""
    }
    if strings.ContainsAny(value, " \t\"") {
        escaped := strings.ReplaceAll(value, "\"", "\\\"")
        return "\"" + escaped + "\""
    }
    return value
}

func callEngineHTTP(endpoint string, payload []byte) (map[string]interface{}, error) {
    if payload == nil {
        resp, err := http.Get(endpoint)
        if err != nil {
            return nil, err
        }
        defer resp.Body.Close()
        body, _ := io.ReadAll(resp.Body)
        return parseJSONResponse(string(body)), nil
    }

    raw, err := postJSON(endpoint, payload)
    if err != nil {
        return nil, err
    }
    return parseJSONResponse(raw), nil
}

func engineCommandHint() string {
    if cli := strings.TrimSpace(os.Getenv("KOGI_ENGINE_CLI")); cli != "" {
        return cli
    }
    if path, err := exec.LookPath("kogi-engine-cli"); err == nil {
        return path
    }
    if path, err := exec.LookPath("sbt"); err == nil {
        return path + " runMain kogi.engine.KogiEngineCli"
    }
    return ""
}

func parsePayloadMap(payload string) map[string]string {
    payload = strings.TrimSpace(payload)
    if payload == "" || payload == "{}" {
        return map[string]string{}
    }

    var decoded map[string]interface{}
    if err := json.Unmarshal([]byte(payload), &decoded); err != nil {
        return map[string]string{"raw": payload}
    }

    result := make(map[string]string, len(decoded))
    for key, value := range decoded {
        result[key] = fmt.Sprintf("%v", value)
    }
    return result
}

func parseJSONResponse(raw string) map[string]interface{} {
    raw = strings.TrimSpace(raw)
    if raw == "" {
        return map[string]interface{}{}
    }

    var decoded map[string]interface{}
    if err := json.Unmarshal([]byte(raw), &decoded); err == nil {
        return decoded
    }

    return map[string]interface{}{"raw": raw}
}

func mustJSON(value interface{}) []byte {
    raw, _ := json.Marshal(value)
    return raw
}

func sortedKeys(values map[string]string) []string {
    keys := make([]string, 0, len(values))
    for key := range values {
        keys = append(keys, key)
    }
    sort.Strings(keys)
    return keys
}

func gatewayURL() string {
    if url := strings.TrimSpace(os.Getenv("KOGI_GATEWAY_URL")); url != "" {
        return strings.TrimRight(url, "/")
    }
    return "http://127.0.0.1:8090"
}

func publishGatewayMessage(request gatewayPublishRequest) (map[string]interface{}, string) {
    endpoint := gatewayURL() + "/api/v1/gateway/pubsub/publish"
    if engineRuntime != nil {
        engineRuntime.Publish(request.Topic, request.Source, request.Target, request.Payload)
    }
    raw := mustJSON(request)
    response, err := postJSON(endpoint, raw)
    if err != nil {
        if engineRuntime != nil {
            engineRuntime.Debugf("publish failed endpoint=%s err=%s", endpoint, err.Error())
        }
        return nil, err.Error()
    }
    if engineRuntime != nil {
        engineRuntime.Debugf("publish ack endpoint=%s", endpoint)
    }
    return parseJSONResponse(response), ""
}

func gatewayHistory(limit int, topic string) (map[string]interface{}, error) {
    endpoint := fmt.Sprintf("%s/api/v1/gateway/pubsub/history?limit=%d", gatewayURL(), limit)
    if topic != "" {
        endpoint += "&topic=" + url.QueryEscape(topic)
    }
    if engineRuntime != nil {
        engineRuntime.Subscribe("history", topic, "engine-service")
        engineRuntime.Debugf("history request endpoint=%s", endpoint)
    }
    resp, err := http.Get(endpoint)
    if err != nil {
        if engineRuntime != nil {
            engineRuntime.Debugf("history failed endpoint=%s err=%s", endpoint, err.Error())
        }
        return nil, err
    }
    defer resp.Body.Close()
    body, _ := io.ReadAll(resp.Body)
    if engineRuntime != nil {
        engineRuntime.Debugf("history response endpoint=%s status=%d bytes=%d", endpoint, resp.StatusCode, len(body))
    }
    return parseJSONResponse(string(body)), nil
}

func shouldForwardToEngine(topic, target string) bool {
    normalized := strings.ToLower(topic + " " + target)
    return strings.Contains(normalized, "engine.") || strings.Contains(normalized, "kogi.engine")
}

func queryInt(r *http.Request, key string, fallback int) int {
    value := r.URL.Query().Get(key)
    if value == "" {
        return fallback
    }
    parsed, err := strconv.Atoi(value)
    if err != nil || parsed <= 0 {
        return fallback
    }
    return parsed
}

func defaultIfEmpty(value, fallback string) string {
    if value == "" {
        return fallback
    }
    return value
}

func findRepoRoot() (string, bool) {
    cwd, err := os.Getwd()
    if err != nil {
        return "", false
    }

    current := cwd
    for i := 0; i < 8; i++ {
        if pathExists(filepath.Join(current, "kogi-engine")) || pathExists(filepath.Join(current, "go.work")) {
            return current, true
        }
        parent := filepath.Dir(current)
        if parent == current {
            break
        }
        current = parent
    }
    return "", false
}

func pathExists(path string) bool {
    _, err := os.Stat(path)
    return err == nil
}


func engineEndpoint() string {
    endpoint := strings.TrimSpace(os.Getenv("KOGI_ENGINE_ENDPOINT"))
    return strings.TrimRight(endpoint, "/")
}

func postJSON(endpoint string, payload []byte) (string, error) {
    resp, err := http.Post(endpoint, "application/json", bytes.NewReader(payload))
    if err != nil {
        return "", err
    }
    defer resp.Body.Close()
    body, _ := io.ReadAll(resp.Body)
    return string(body), nil
}

func writeJSON(w http.ResponseWriter, status int, payload interface{}) {
    w.Header().Set("Content-Type", "application/json")
    w.WriteHeader(status)
    _ = json.NewEncoder(w).Encode(payload)
}

func resolveAddr(defaultPort, servicePortEnv string) string {
    if port := os.Getenv(servicePortEnv); port != "" {
        return toListenAddr(port)
    }
    if port := os.Getenv("KOGI_PORT"); port != "" {
        return toListenAddr(port)
    }
    return ":" + defaultPort
}

func toListenAddr(port string) string {
    if strings.HasPrefix(port, ":") {
        return port
    }
    return ":" + port
}
