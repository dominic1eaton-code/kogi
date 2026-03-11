package ops

import (
    "flag"
    "io"
    "log"
    "net/http"
    "os"
    "os/signal"
    "strings"
    "syscall"
    "time"
)

const maxPayloadLen = 256

type Runtime struct {
    Service   string
    Debug     bool
    Silent    bool
    StartedAt time.Time
}

func Init(service string) *Runtime {
    debug := flag.Bool("debug", envBool("KOGI_DEBUG"), "enable debug logging")
    silent := flag.Bool("silent", envBool("KOGI_SILENT"), "suppress stdout/stderr logging (background mode)")
    flag.Parse()

    rt := &Runtime{
        Service:   service,
        Debug:     *debug,
        Silent:    *silent,
        StartedAt: time.Now().UTC(),
    }

    if rt.Silent {
        log.SetOutput(io.Discard)
    }

    if rt.DebugEnabled() {
        log.Printf("[%s] debug enabled", rt.Service)
    }

    return rt
}

func (r *Runtime) DebugEnabled() bool {
    return r != nil && r.Debug && !r.Silent
}

func (r *Runtime) Debugf(format string, args ...any) {
    if !r.DebugEnabled() {
        return
    }
    log.Printf("[%s] "+format, append([]any{r.Service}, args...)...)
}

func (r *Runtime) State(state string) {
    r.Debugf("state=%s", state)
}

func (r *Runtime) Status(status, note string) {
    if note == "" {
        r.Debugf("status=%s", status)
        return
    }
    r.Debugf("status=%s note=%s", status, note)
}

func (r *Runtime) Message(direction, topic, source, target, payload string) {
    r.Debugf("message %s topic=%s source=%s target=%s payload=%s",
        direction,
        emptyIfUnknown(topic),
        emptyIfUnknown(source),
        emptyIfUnknown(target),
        truncate(payload),
    )
}

func (r *Runtime) Publish(topic, source, target, payload string) {
    r.Debugf("publish topic=%s source=%s target=%s payload=%s",
        emptyIfUnknown(topic),
        emptyIfUnknown(source),
        emptyIfUnknown(target),
        truncate(payload),
    )
}

func (r *Runtime) Subscribe(kind, topic, consumer string) {
    r.Debugf("subscribe kind=%s topic=%s consumer=%s",
        emptyIfUnknown(kind),
        emptyIfUnknown(topic),
        emptyIfUnknown(consumer),
    )
}

func (r *Runtime) WatchSignals() {
    if r == nil {
        return
    }
    ch := make(chan os.Signal, 1)
    signal.Notify(ch, os.Interrupt, syscall.SIGTERM)
    go func() {
        sig := <-ch
        r.State("shutting_down")
        r.Status("signal", sig.String())
        os.Exit(0)
    }()
}

func WithHTTPDebug(rt *Runtime, next http.Handler) http.Handler {
    if rt == nil || !rt.DebugEnabled() {
        return next
    }
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        start := time.Now()
        rt.Debugf("message received kind=http method=%s path=%s remote=%s",
            r.Method, r.URL.Path, r.RemoteAddr)
        recorder := &statusRecorder{ResponseWriter: w}
        next.ServeHTTP(recorder, r)
        status := recorder.status
        if status == 0 {
            status = http.StatusOK
        }
        rt.Debugf("message sent kind=http method=%s path=%s status=%d bytes=%d duration_ms=%d",
            r.Method,
            r.URL.Path,
            status,
            recorder.bytes,
            time.Since(start).Milliseconds(),
        )
    })
}

type statusRecorder struct {
    http.ResponseWriter
    status int
    bytes  int
}

func (r *statusRecorder) WriteHeader(code int) {
    r.status = code
    r.ResponseWriter.WriteHeader(code)
}

func (r *statusRecorder) Write(b []byte) (int, error) {
    if r.status == 0 {
        r.status = http.StatusOK
    }
    n, err := r.ResponseWriter.Write(b)
    r.bytes += n
    return n, err
}

func truncate(value string) string {
    if value == "" {
        return ""
    }
    if len(value) <= maxPayloadLen {
        return value
    }
    return value[:maxPayloadLen] + "..."
}

func emptyIfUnknown(value string) string {
    if value == "" {
        return "-"
    }
    return value
}

func envBool(key string) bool {
    raw := strings.TrimSpace(os.Getenv(key))
    if raw == "" {
        return false
    }
    raw = strings.ToLower(raw)
    switch raw {
    case "1", "true", "yes", "on":
        return true
    default:
        return false
    }
}
