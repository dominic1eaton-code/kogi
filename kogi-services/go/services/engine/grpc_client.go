package main

import (
    "context"
    "errors"
    "fmt"
    "os"
    "strings"
    "time"

    "google.golang.org/grpc"
    "google.golang.org/grpc/credentials/insecure"
    "google.golang.org/protobuf/types/known/structpb"
)

type engineGrpcConfig struct {
    Addr     string
    Enabled  bool
    Required bool
    Timeout  time.Duration
}

func loadEngineGrpcConfig() engineGrpcConfig {
    mode := strings.ToLower(strings.TrimSpace(os.Getenv("KOGI_ENGINE_GRPC_MODE")))
    addr := strings.TrimSpace(os.Getenv("KOGI_ENGINE_GRPC_ADDR"))

    if mode == "off" || mode == "disabled" || mode == "false" || mode == "0" {
        return engineGrpcConfig{Addr: addr, Enabled: false, Required: false, Timeout: 2 * time.Second}
    }

    enabled := false
    if addr != "" {
        enabled = true
    } else if mode == "required" || mode == "on" || mode == "true" || mode == "1" {
        addr = "127.0.0.1:9100"
        enabled = true
    }

    required := mode == "required"

    timeout := 2 * time.Second
    if raw := strings.TrimSpace(os.Getenv("KOGI_ENGINE_GRPC_TIMEOUT")); raw != "" {
        if parsed, err := time.ParseDuration(raw); err == nil {
            timeout = parsed
        }
    }

    return engineGrpcConfig{Addr: addr, Enabled: enabled, Required: required, Timeout: timeout}
}

func grpcInvokeEngine(ctx context.Context, method string, payload map[string]interface{}) (map[string]interface{}, error) {
    cfg := loadEngineGrpcConfig()
    if !cfg.Enabled {
        return nil, errors.New("grpc disabled")
    }
    if payload == nil {
        payload = map[string]interface{}{}
    }

    if engineRuntime != nil {
        engineRuntime.Debugf("grpc invoke method=%s addr=%s", method, cfg.Addr)
    }

    ctx, cancel := context.WithTimeout(ctx, cfg.Timeout)
    defer cancel()

    conn, err := grpc.DialContext(ctx, cfg.Addr, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithBlock())
    if err != nil {
        if engineRuntime != nil {
            engineRuntime.Debugf("grpc dial failed addr=%s err=%s", cfg.Addr, err.Error())
        }
        return nil, err
    }
    defer conn.Close()

    req, err := structpb.NewStruct(payload)
    if err != nil {
        return nil, err
    }
    var resp structpb.Struct
    fullMethod := "/kogi.engine.v1.EngineService/" + method
    if err := conn.Invoke(ctx, fullMethod, req, &resp); err != nil {
        if engineRuntime != nil {
            engineRuntime.Debugf("grpc invoke failed method=%s err=%s", method, err.Error())
        }
        return nil, err
    }

    if engineRuntime != nil {
        engineRuntime.Debugf("grpc invoke ok method=%s", method)
    }

    return resp.AsMap(), nil
}

func grpcAttempt(method string, payload map[string]interface{}) (map[string]interface{}, bool, string, bool) {
    cfg := loadEngineGrpcConfig()
    if !cfg.Enabled {
        return nil, false, "", false
    }
    resp, err := grpcInvokeEngine(context.Background(), method, payload)
    if err != nil {
        return nil, true, err.Error(), cfg.Required
    }
    return resp, true, "", cfg.Required
}

func stringMapToInterface(input map[string]string) map[string]interface{} {
    if len(input) == 0 {
        return map[string]interface{}{}
    }
    output := make(map[string]interface{}, len(input))
    for key, value := range input {
        output[key] = value
    }
    return output
}

func grpcControlRequest(action string) map[string]interface{} {
    return map[string]interface{}{
        "action": action,
    }
}

func grpcIngestRequest(topic, source, target string, payload map[string]string, options map[string]string) map[string]interface{} {
    request := map[string]interface{}{
        "topic":       topic,
        "source":      source,
        "target":      target,
        "payload":     stringMapToInterface(payload),
        "flow_id":     options["flow-id"],
        "timestamp_ms": options["timestamp-ms"],
    }
    if request["flow_id"] == "" {
        request["flow_id"] = options["flow_id"]
    }
    if request["timestamp_ms"] == "" {
        request["timestamp_ms"] = options["timestamp_ms"]
    }
    return request
}

func grpcSnapshotRequest(hostID string, windowMs int64) map[string]interface{} {
    return map[string]interface{}{
        "host_id":   hostID,
        "window_ms": windowMs,
    }
}

func grpcStatusRequest() map[string]interface{} {
    return map[string]interface{}{}
}

func grpcModeSummary() map[string]interface{} {
    cfg := loadEngineGrpcConfig()
    return map[string]interface{}{
        "addr":       cfg.Addr,
        "enabled":    cfg.Enabled,
        "required":   cfg.Required,
        "timeout_ms": cfg.Timeout.Milliseconds(),
        "service":    "kogi.engine.v1.EngineService",
    }
}

func ensureGrpcOption(options map[string]string, key, fallback string) map[string]string {
    if options == nil {
        options = map[string]string{}
    }
    if options[key] == "" {
        options[key] = fallback
    }
    return options
}

func grpcFlowDefaults(options map[string]string) map[string]string {
    options = ensureGrpcOption(options, "flow-id", fmt.Sprintf("flow-%d", time.Now().UnixMilli()))
    options = ensureGrpcOption(options, "timestamp-ms", fmt.Sprintf("%d", time.Now().UnixMilli()))
    return options
}
