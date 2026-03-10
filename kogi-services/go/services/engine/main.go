package main

import (
    "encoding/json"
    "io"
    "log"
    "net/http"
    "os"
    "strings"
    "sync"
    "time"
)

type ingestRecord struct {
    ID         string `json:"id"`
    Payload    string `json:"payload"`
    ReceivedAt string `json:"received_at"`
}

var (
    ingestMu    sync.Mutex
    ingestLog   []ingestRecord
    controlMode = "stopped"
)

func main() {
    mux := http.NewServeMux()

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
            "gateway":         "http://127.0.0.1:8090",
            "network_manager": "kogi-go-network",
            "control_mode":    mode,
            "ingest_count":    count,
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

        writeJSON(w, http.StatusOK, map[string]interface{}{
            "status":  "ok",
            "service": "engine-service",
            "action":  action,
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

        ingestMu.Lock()
        ingestLog = append(ingestLog, ingestRecord{
            ID:         "ingest-" + time.Now().UTC().Format("20060102150405"),
            Payload:    payload,
            ReceivedAt: time.Now().UTC().Format(time.RFC3339),
        })
        count := len(ingestLog)
        ingestMu.Unlock()

        writeJSON(w, http.StatusAccepted, map[string]interface{}{
            "status":       "queued",
            "service":      "engine-service",
            "ingest_count": count,
        })
    })

    mux.HandleFunc("/api/v1/engine/snapshot", func(w http.ResponseWriter, r *http.Request) {
        ingestMu.Lock()
        count := len(ingestLog)
        mode := controlMode
        ingestMu.Unlock()

        writeJSON(w, http.StatusOK, map[string]interface{}{
            "engine":       "kogi-engine",
            "control_mode": mode,
            "ingest_count": count,
            "status":       "active",
        })
    })

    addr := resolveAddr("9014", "KOGI_ENGINE_PORT")
    log.Printf("engine-service listening on %s", addr)
    log.Fatal(http.ListenAndServe(addr, mux))
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
