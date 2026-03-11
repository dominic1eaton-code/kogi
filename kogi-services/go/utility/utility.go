package main

// utility.go
//
// General-purpose utilities shared by office_service.go and
// portfolio_service.go (and any future Go service in this package).
//
// Contents
// --------
//  1.  RustExternalCode  — call external Rust code as exes, DLLs, or
//                          subprocesses (replaces the old portfolioCallRust /
//                          officeCallRust pair and extends them with DLL and
//                          subprocess support)
//  2.  DLL bridge        — load kogi_office.dll / kogi_portfolio.dll and call
//                          exported C functions directly (CGo-free, pure Go
//                          syscall on Windows; dlopen on Linux/macOS)
//  3.  HTTP helpers      — writeJSON, decodeBody, trimPrefix, resolveAddr,
//                          responseRecorder
//  4.  Gateway pub/sub   — publish, subscribe (exact & prefix), replay,
//                          dead-letter polling
//  5.  Health reporting  — reportHealth, healthLoop
//  6.  File-system       — fileExists, findRepoRoot
//  7.  Address utilities — toListenAddr

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"time"
	"unsafe"
)

// =============================================================================
// §1 — RustExternalCode  (general-purpose Rust interop)
// =============================================================================

// rustRequest is the envelope sent to a Rust exe via --request <json>.
type rustRequest struct {
	Action  string      `json:"action"`
	Payload interface{} `json:"payload,omitempty"`
}

// RustCallMode selects how the Go service communicates with Rust code.
type RustCallMode int

const (
	// RustModeExe spawns the binary as a child process (--request <json>).
	RustModeExe RustCallMode = iota
	// RustModeDLL loads a shared library and calls an exported C function via
	// the platform's native dynamic-linking mechanism.
	RustModeDLL
	// RustModeSubprocess is like RustModeExe but keeps the child alive and
	// communicates over stdin/stdout (reserved for future use).
	RustModeSubprocess
)

// RustBridgeConfig is the configuration for one Rust bridge endpoint.
type RustBridgeConfig struct {
	// Mode selects the interop mechanism.
	Mode RustCallMode
	// EnvBinKey is the environment variable that overrides the binary/DLL path.
	EnvBinKey string
	// WellKnownName is the executable or DLL stem used for PATH / repo lookup.
	// e.g. "kogi-portfolio-system" or "kogi_office"
	WellKnownName string
	// RepoSubPaths is the list of relative paths under the repo root to search.
	RepoSubPaths []string
	// ServiceLabel is used in log messages.
	ServiceLabel string
	// Timeout for a single call.
	Timeout time.Duration
}

// resolveRustBin resolves the path to a Rust binary or DLL using the config.
func resolveRustBin(cfg RustBridgeConfig) (string, error) {
	// 1. Explicit environment override.
	if p := os.Getenv(cfg.EnvBinKey); p != "" {
		return p, nil
	}

	// 2. PATH lookup (exe mode) or LD_LIBRARY_PATH / system lookup (DLL mode).
	if cfg.Mode == RustModeExe || cfg.Mode == RustModeSubprocess {
		if p, err := exec.LookPath(cfg.WellKnownName); err == nil {
			return p, nil
		}
	}

	// 3. Repository-relative heuristic.
	if root, ok := findRepoRoot(); ok {
		ext := ""
		switch cfg.Mode {
		case RustModeExe, RustModeSubprocess:
			if runtime.GOOS == "windows" {
				ext = ".exe"
			}
		case RustModeDLL:
			switch runtime.GOOS {
			case "windows":
				ext = ".dll"
			case "darwin":
				ext = ".dylib"
			default:
				ext = ".so"
			}
		}
		for _, sub := range cfg.RepoSubPaths {
			candidate := filepath.Join(root, sub, cfg.WellKnownName+ext)
			if fileExists(candidate) {
				return candidate, nil
			}
		}
	}
	return "", fmt.Errorf("[%s] binary not found; set %s", cfg.ServiceLabel, cfg.EnvBinKey)
}

// callRustExe spawns a Rust binary as a child process, passes the request
// envelope via --request <json>, reads stdout, and returns the decoded result.
func callRustExe(cfg RustBridgeConfig, action string, payload interface{}) (interface{}, error) {
	bin, err := resolveRustBin(cfg)
	if err != nil {
		return nil, err
	}
	envelope, _ := json.Marshal(rustRequest{Action: action, Payload: payload})
	timeout := cfg.Timeout
	if timeout == 0 {
		timeout = 5 * time.Second
	}
	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()
	cmd := exec.CommandContext(ctx, bin, "--request", string(envelope))
	out, err := cmd.CombinedOutput()
	if err != nil {
		return nil, fmt.Errorf("[%s] exe action=%s: %w – %s",
			cfg.ServiceLabel, action, err, strings.TrimSpace(string(out)))
	}
	var result interface{}
	if err := json.Unmarshal(out, &result); err != nil {
		return nil, fmt.Errorf("[%s] exe parse action=%s: %w", cfg.ServiceLabel, action, err)
	}
	return result, nil
}

// callRustDLL loads a Rust shared library and calls an exported C function
// whose signature is:
//
//	char* kogi_<func_name>(const char* args_json);   // for functions with args
//	char* kogi_<func_name>();                         // for zero-arg functions
//
// The returned C string is freed via the exported kogi_free_string symbol.
// This implementation uses platform-native dynamic linking (syscall on Windows,
// dlopen/dlsym via cgo-free unsafe tricks on Linux/macOS via the purego pattern).
//
// NOTE: On Linux/macOS this requires that the .so/.dylib is on LD_LIBRARY_PATH
//       or is specified as an absolute path.  On Windows the .dll must be in
//       PATH or its directory.
//
// For production use, consider the "github.com/ebitengine/purego" package which
// provides a CGo-free dlopen/dlsym implementation; the implementation below
// falls back to subprocess execution when the DLL cannot be loaded, ensuring
// the service degrades gracefully.
func callRustDLL(cfg RustBridgeConfig, funcName string, argsJSON string) (interface{}, error) {
	lib, err := openSharedLib(cfg)
	if err != nil {
		// Graceful degradation: treat DLL call as exe call with a synthetic action.
		log.Printf("[%s] DLL load failed (%v); falling back to exe mode", cfg.ServiceLabel, err)
		return callRustExe(cfg, funcName, json.RawMessage(argsJSON))
	}
	defer closeSharedLib(lib)

	sym, err := lookupSymbol(lib, "kogi_"+funcName)
	if err != nil {
		return nil, fmt.Errorf("[%s] symbol kogi_%s not found: %w", cfg.ServiceLabel, funcName, err)
	}

	// Call the function.  The symbol is typed as:
	//   func(argsPtr unsafe.Pointer) unsafe.Pointer
	var rawResult unsafe.Pointer
	if argsJSON == "" {
		rawResult = callSymNoArgs(sym)
	} else {
		rawResult = callSymWithArgs(sym, argsJSON)
	}

	if rawResult == nil {
		return nil, fmt.Errorf("[%s] kogi_%s returned null", cfg.ServiceLabel, funcName)
	}

	// Convert the returned C string and free it via the exported free function.
	resultStr := cStringToGoString(rawResult)
	if freeSymbol, err2 := lookupSymbol(lib, "kogi_free_string"); err2 == nil {
		callFreeString(freeSymbol, rawResult)
	}

	var result interface{}
	if err := json.Unmarshal([]byte(resultStr), &result); err != nil {
		return nil, fmt.Errorf("[%s] DLL JSON parse kogi_%s: %w", cfg.ServiceLabel, funcName, err)
	}
	return result, nil
}

// callRustSubprocess launches a Rust binary as a persistent subprocess and
// communicates over its stdin/stdout using newline-delimited JSON.  This is a
// stub for future use; the current implementation delegates to callRustExe.
func callRustSubprocess(cfg RustBridgeConfig, action string, payload interface{}) (interface{}, error) {
	// TODO: implement persistent subprocess with stdin/stdout newline-delimited JSON.
	return callRustExe(cfg, action, payload)
}

// CallRust dispatches to the appropriate backend based on cfg.Mode.
func CallRust(cfg RustBridgeConfig, actionOrFunc string, payload interface{}) (interface{}, error) {
	switch cfg.Mode {
	case RustModeDLL:
		var argsJSON string
		if payload != nil {
			b, err := json.Marshal(payload)
			if err != nil {
				return nil, fmt.Errorf("CallRust marshal: %w", err)
			}
			argsJSON = string(b)
		}
		return callRustDLL(cfg, actionOrFunc, argsJSON)
	case RustModeSubprocess:
		return callRustSubprocess(cfg, actionOrFunc, payload)
	default: // RustModeExe
		return callRustExe(cfg, actionOrFunc, payload)
	}
}

// CallRustOrFallback calls CallRust and returns fallback on any error.
func CallRustOrFallback(cfg RustBridgeConfig, actionOrFunc string, payload, fallback interface{}) interface{} {
	r, err := CallRust(cfg, actionOrFunc, payload)
	if err != nil {
		log.Printf("[%s] rust call %s err=%v", cfg.ServiceLabel, actionOrFunc, err)
		return fallback
	}
	return r
}

// =============================================================================
// §1a — Platform DLL shims
//
// These thin wrappers isolate platform-specific dynamic-linking from the rest
// of the code.  They use Go's syscall/windows package on Windows and the
// POSIX dlopen/dlsym API on Linux/macOS via unsafe tricks.
// =============================================================================

// sharedLib is an opaque handle to an open shared library.
type sharedLib struct {
	handle uintptr
	path   string
}

// symbol is an opaque pointer to an exported function in a shared library.
type symbol struct {
	ptr uintptr
}

func openSharedLib(cfg RustBridgeConfig) (*sharedLib, error) {
	path, err := resolveRustBin(cfg)
	if err != nil {
		return nil, err
	}
	return openSharedLibPath(path)
}

// =============================================================================
// §2 — Pre-configured bridges for office and portfolio services
// =============================================================================

// OfficeDLLConfig is the bridge config for the kogi_office DLL.
var OfficeDLLConfig = RustBridgeConfig{
	Mode:          RustModeDLL,
	EnvBinKey:     "KOGI_OFFICE_DLL",
	WellKnownName: "kogi_office",
	RepoSubPaths: []string{
		filepath.Join("kogi-modules", "office", "target", "release"),
		filepath.Join("kogi-modules", "office", "target", "debug"),
		filepath.Join("kogi-modules", "office"),
	},
	ServiceLabel: "office-dll",
	Timeout:      5 * time.Second,
}

// OfficeExeConfig is the fallback exe bridge config for the office service.
var OfficeExeConfig = RustBridgeConfig{
	Mode:          RustModeExe,
	EnvBinKey:     "KOGI_OFFICE_SYSTEM_BIN",
	WellKnownName: "kogi-office-system",
	RepoSubPaths: []string{
		filepath.Join("kogi-modules", "office", "target", "debug"),
		filepath.Join("kogi-modules", "office", "target", "release"),
		filepath.Join("kogi-modules", "office"),
	},
	ServiceLabel: "office-svc",
	Timeout:      5 * time.Second,
}

// PortfolioDLLConfig is the bridge config for the kogi_portfolio DLL.
var PortfolioDLLConfig = RustBridgeConfig{
	Mode:          RustModeDLL,
	EnvBinKey:     "KOGI_PORTFOLIO_DLL",
	WellKnownName: "kogi_portfolio",
	RepoSubPaths: []string{
		filepath.Join("kogi-modules", "portfolio", "target", "release"),
		filepath.Join("kogi-modules", "portfolio", "target", "debug"),
		filepath.Join("kogi-modules", "portfolio"),
	},
	ServiceLabel: "portfolio-dll",
	Timeout:      5 * time.Second,
}

// PortfolioExeConfig is the fallback exe bridge config for the portfolio service.
var PortfolioExeConfig = RustBridgeConfig{
	Mode:          RustModeExe,
	EnvBinKey:     "KOGI_PORTFOLIO_SYSTEM_BIN",
	WellKnownName: "kogi-portfolio-system",
	RepoSubPaths: []string{
		filepath.Join("kogi-modules", "portfolio", "target", "debug"),
		filepath.Join("kogi-modules", "portfolio", "target", "release"),
		filepath.Join("kogi-modules", "portfolio"),
	},
	ServiceLabel: "portfolio-svc",
	Timeout:      5 * time.Second,
}

// callOfficeDLL calls an exported function in kogi_office.dll, falling back to
// the exe bridge if the DLL cannot be loaded.
func callOfficeDLL(funcName string, payload interface{}) (interface{}, error) {
	result, err := CallRust(OfficeDLLConfig, funcName, payload)
	if err != nil {
		// DLL unavailable — degrade to exe bridge with the same function name
		// used as the action string.
		return CallRust(OfficeExeConfig, funcName, payload)
	}
	return result, nil
}

// officeRust calls the office Rust bridge and returns fallback on error.
func officeRust(funcName string, payload, fallback interface{}) interface{} {
	r, err := callOfficeDLL(funcName, payload)
	if err != nil {
		log.Printf("[office-svc] rust func=%s err=%v", funcName, err)
		return fallback
	}
	return r
}

// callPortfolioDLL calls an exported function in kogi_portfolio.dll (or the
// kogi_office.dll portfolio functions), falling back to the exe bridge.
func callPortfolioDLL(funcName string, payload interface{}) (interface{}, error) {
	// The portfolio FFI functions live in kogi_office.dll (unified DLL), so we
	// try the office DLL first, then fall back to a dedicated portfolio DLL.
	result, err := CallRust(OfficeDLLConfig, funcName, payload)
	if err != nil {
		result, err = CallRust(PortfolioDLLConfig, funcName, payload)
		if err != nil {
			return CallRust(PortfolioExeConfig, funcName, payload)
		}
	}
	return result, nil
}

// portfolioRust calls the portfolio Rust bridge and returns fallback on error.
func portfolioRust(funcName string, payload, fallback interface{}) interface{} {
	r, err := callPortfolioDLL(funcName, payload)
	if err != nil {
		log.Printf("[portfolio-svc] rust func=%s err=%v", funcName, err)
		return fallback
	}
	return r
}

// portfolioCallRust is a direct call that returns an error on failure (used
// when the handler must distinguish success from failure before publishing).
func portfolioCallRust(funcName string, payload interface{}) (interface{}, error) {
	return callPortfolioDLL(funcName, payload)
}

// officeCallRust is the direct-error variant used by the office service.
func officeCallRust(funcName string, payload interface{}) (interface{}, error) {
	return callOfficeDLL(funcName, payload)
}

// =============================================================================
// §3 — HTTP helpers
// =============================================================================

// writeJSON encodes payload as JSON and writes it with the given status code.
func writeJSON(w http.ResponseWriter, status int, payload interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(payload)
}

// writeOfficeJSON is an alias of writeJSON kept for backward compatibility with
// existing office_service.go call-sites.
func writeOfficeJSON(w http.ResponseWriter, status int, payload interface{}) {
	writeJSON(w, status, payload)
}

// decodeBody reads r.Body into dst and returns an error if the JSON is invalid.
func decodeBody(r *http.Request, dst interface{}) error {
	defer r.Body.Close()
	b, _ := io.ReadAll(r.Body)
	if err := json.Unmarshal(b, dst); err != nil {
		return fmt.Errorf("invalid JSON: %w", err)
	}
	return nil
}

// decodeOfficeBody is an alias of decodeBody kept for backward compatibility.
func decodeOfficeBody(r *http.Request, dst interface{}) error {
	return decodeBody(r, dst)
}

// trimPrefix removes prefix from r.URL.Path.
func trimPrefix(r *http.Request, prefix string) string {
	return strings.TrimPrefix(r.URL.Path, prefix)
}

// officeTrimPrefix is an alias of trimPrefix kept for backward compatibility.
func officeTrimPrefix(r *http.Request, prefix string) string {
	return trimPrefix(r, prefix)
}

// resolveAddr returns a ":port" listen address, checking envKey then KOGI_PORT
// then falling back to defaultPort.
func resolveAddr(defaultPort, envKey string) string {
	if p := os.Getenv(envKey); p != "" {
		return toListenAddr(p)
	}
	if p := os.Getenv("KOGI_PORT"); p != "" {
		return toListenAddr(p)
	}
	return ":" + defaultPort
}

// resolveOfficeAddr is an alias of resolveAddr kept for backward compatibility.
func resolveOfficeAddr(defaultPort, envKey string) string {
	return resolveAddr(defaultPort, envKey)
}

// toListenAddr ensures the port string starts with ":".
func toListenAddr(port string) string {
	if strings.HasPrefix(port, ":") {
		return port
	}
	return ":" + port
}

// officeToListenAddr / portfolioToListenAddr are aliases kept for backward
// compatibility.
func officeToListenAddr(port string) string     { return toListenAddr(port) }
func portfolioToListenAddr(port string) string  { return toListenAddr(port) }

// responseRecorder captures an http.ResponseWriter so a handler's output can
// be inspected before being forwarded (used by proxy-and-publish helpers).
type responseRecorder struct {
	header http.Header
	code   int
	body   []byte
}

func (r *responseRecorder) Header() http.Header        { return r.header }
func (r *responseRecorder) WriteHeader(code int)        { r.code = code }
func (r *responseRecorder) Write(b []byte) (int, error) { r.body = append(r.body, b...); return len(b), nil }

// =============================================================================
// §4 — Gateway pub/sub
// =============================================================================

// publish sends a pub/sub event to the gateway bus.
func publish(gatewayBase, sourceID, topic, payload string, meta map[string]string) {
	body, _ := json.Marshal(map[string]interface{}{
		"topic": topic, "payload": payload,
		"source": sourceID, "metadata": meta,
	})
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost,
		gatewayBase+"/api/v1/gateway/pubsub/publish", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Printf("[pubsub] publish topic=%s err=%v", topic, err)
		return
	}
	defer resp.Body.Close()
}

// gatewaySubscribePrefix registers a prefix subscription on the gateway.
func gatewaySubscribePrefix(gatewayBase, prefix, consumer string) {
	body, _ := json.Marshal(map[string]string{"prefix": prefix, "consumer": consumer})
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost,
		gatewayBase+"/api/v1/gateway/pubsub/subscribe/prefix", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Printf("[pubsub] prefix subscribe prefix=%s err=%v", prefix, err)
		return
	}
	defer resp.Body.Close()
}

// subscribeGatewayPrefix polls the gateway prefix-history endpoint in a loop,
// calling handler for each event.  Run in its own goroutine.
func subscribeGatewayPrefix(gatewayBase string, prefixes []string, handler func(topic, payload string)) {
	for {
		for _, prefix := range prefixes {
			func() {
				ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
				defer cancel()
				req, _ := http.NewRequestWithContext(ctx, http.MethodGet,
					fmt.Sprintf("%s/api/v1/gateway/pubsub/history/prefix?prefix=%s&limit=10",
						gatewayBase, prefix), nil)
				resp, err := http.DefaultClient.Do(req)
				if err != nil {
					return
				}
				defer resp.Body.Close()
				var result struct {
					Events []struct {
						Topic   string `json:"topic"`
						Payload string `json:"payload"`
					} `json:"events"`
				}
				if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
					return
				}
				for _, e := range result.Events {
					handler(e.Topic, e.Payload)
				}
			}()
		}
		time.Sleep(3 * time.Second)
	}
}

// subscribeGatewayExact polls the gateway history endpoint for exact topics.
func subscribeGatewayExact(gatewayBase string, topics []string, handler func(topic, payload string)) {
	for {
		for _, topic := range topics {
			func() {
				ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
				defer cancel()
				req, _ := http.NewRequestWithContext(ctx, http.MethodGet,
					fmt.Sprintf("%s/api/v1/gateway/pubsub/history?limit=5&topic=%s",
						gatewayBase, topic), nil)
				resp, err := http.DefaultClient.Do(req)
				if err != nil {
					return
				}
				defer resp.Body.Close()
				var result struct {
					Events []struct {
						Topic   string `json:"topic"`
						Payload string `json:"payload"`
					} `json:"events"`
				}
				if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
					return
				}
				for _, e := range result.Events {
					handler(e.Topic, e.Payload)
				}
			}()
		}
		time.Sleep(3 * time.Second)
	}
}

// pollInbound polls the gateway /pubsub/receive endpoint for specific topics and
// logs each event.  Suitable for simple inbound monitoring.
func pollInbound(gatewayBase string, topics []string) {
	for {
		for _, topic := range topics {
			func() {
				ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
				defer cancel()
				req, _ := http.NewRequestWithContext(ctx, http.MethodGet,
					fmt.Sprintf("%s/api/v1/gateway/pubsub/receive?topic=%s&limit=5",
						gatewayBase, topic), nil)
				resp, err := http.DefaultClient.Do(req)
				if err != nil {
					return
				}
				defer resp.Body.Close()
				var result struct {
					Events []struct {
						Topic   string `json:"topic"`
						Payload string `json:"payload"`
						Source  string `json:"source"`
					} `json:"events"`
				}
				if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
					return
				}
				for _, e := range result.Events {
					log.Printf("[inbound] topic=%s source=%s payload=%s", e.Topic, e.Source, e.Payload)
				}
			}()
		}
		time.Sleep(4 * time.Second)
	}
}

// proxyGateway performs a GET to gatewayBase+path and copies the response to w.
// On error it writes fallback as JSON.
func proxyGateway(w http.ResponseWriter, gatewayBase, path string, fallback interface{}) {
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodGet, gatewayBase+path, nil)
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		writeJSON(w, http.StatusOK, fallback)
		return
	}
	defer resp.Body.Close()
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(resp.StatusCode)
	_, _ = io.Copy(w, resp.Body)
}

// officeProxyGateway is a backward-compat alias.
func officeProxyGateway(w http.ResponseWriter, path string, fallback interface{}) {
	proxyGateway(w, officeGateway, path, fallback)
}

// portfolioProxyGateway is a backward-compat alias.
func portfolioProxyGateway(w http.ResponseWriter, path string, fallback interface{}) {
	proxyGateway(w, portfolioGateway, path, fallback)
}

// replayGateway fetches a time-windowed replay from the gateway and appends
// the results to a caller-supplied event log via the appendFn callback.
func replayGateway(
	gatewayBase, topic, prefix, from, to string,
	appendFn func(topic, payload, source string),
) {
	q := ""
	switch {
	case prefix != "":
		q = fmt.Sprintf("prefix=%s&from=%s&to=%s", prefix, from, to)
	case topic != "":
		q = fmt.Sprintf("topic=%s&from=%s&to=%s", topic, from, to)
	default:
		return
	}
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodGet,
		gatewayBase+"/api/v1/gateway/pubsub/replay?"+q, nil)
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return
	}
	defer resp.Body.Close()
	var result struct {
		Events []struct {
			Topic   string `json:"topic"`
			Payload string `json:"payload"`
			Source  string `json:"source"`
		} `json:"events"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return
	}
	for _, e := range result.Events {
		appendFn(e.Topic, e.Payload, e.Source)
	}
}

// pollDeadLetters polls the gateway dead-letter queue and calls logFn for each
// entry whose topic matches predicate.
func pollDeadLetters(gatewayBase string, predicate func(topic string) bool, logFn func(id, topic string)) {
	for {
		time.Sleep(60 * time.Second)
		ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
		req, _ := http.NewRequestWithContext(ctx, http.MethodGet,
			gatewayBase+"/api/v1/gateway/pubsub/dead-letters?limit=20", nil)
		resp, err := http.DefaultClient.Do(req)
		cancel()
		if err != nil {
			continue
		}
		var result struct {
			DeadLetters []struct {
				Topic string `json:"topic"`
				ID    string `json:"id"`
			} `json:"dead_letters"`
		}
		if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
			resp.Body.Close()
			continue
		}
		resp.Body.Close()
		for _, dl := range result.DeadLetters {
			if predicate(dl.Topic) {
				logFn(dl.ID, dl.Topic)
			}
		}
	}
}

// registerWithGateway registers this service with the gateway mesh registry and
// then subscribes to each (prefix, consumer) pair.
func registerWithGateway(
	gatewayBase, serviceID, selfEndpoint, healthPath string,
	publishTopics, subscribeTopics []string,
	prefixSubs []struct{ Prefix, Consumer string },
) {
	body, _ := json.Marshal(map[string]interface{}{
		"id": serviceID, "kind": "service",
		"endpoint": selfEndpoint, "health_path": healthPath,
		"network_manager": "kogi-go-network", "status": "active",
		"metadata": map[string]string{
			"publishes":  strings.Join(publishTopics, ","),
			"subscribes": strings.Join(subscribeTopics, ","),
		},
	})
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost,
		gatewayBase+"/api/v1/gateway/components/register", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Printf("[%s] gateway registration err=%v", serviceID, err)
		return
	}
	defer resp.Body.Close()
	log.Printf("[%s] registered with gateway %s", serviceID, gatewayBase)

	for _, sub := range prefixSubs {
		gatewaySubscribePrefix(gatewayBase, sub.Prefix, sub.Consumer)
	}
}

// =============================================================================
// §5 — Health reporting
// =============================================================================

// reportHealth sends a health record to the gateway mesh registry.
func reportHealth(gatewayBase, serviceID string, healthy bool, note string) {
	body, _ := json.Marshal(map[string]interface{}{
		"id": serviceID, "healthy": healthy, "note": note,
	})
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodPost,
		gatewayBase+"/api/v1/gateway/components/health", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Printf("[%s] health report err=%v", serviceID, err)
		return
	}
	defer resp.Body.Close()
}

// healthLoop calls checkFn every interval and reports the result via reportHealth.
func healthLoop(gatewayBase, serviceID string, interval time.Duration, checkFn func() (healthy bool, note string)) {
	for {
		time.Sleep(interval)
		healthy, note := checkFn()
		reportHealth(gatewayBase, serviceID, healthy, note)
	}
}

// =============================================================================
// §6 — File-system utilities
// =============================================================================

// fileExists reports whether p names an existing regular file.
func fileExists(p string) bool {
	info, err := os.Stat(p)
	return err == nil && !info.IsDir()
}

// officeFileExists / portfolioFileExists are backward-compat aliases.
func officeFileExists(p string) bool     { return fileExists(p) }
func portfolioFileExists(p string) bool  { return fileExists(p) }

// findRepoRoot walks parent directories looking for a kogi-modules directory or
// go.work file, returning the root path and true when found.
func findRepoRoot() (string, bool) {
	cur, _ := os.Getwd()
	for i := 0; i < 8; i++ {
		if _, err := os.Stat(filepath.Join(cur, "kogi-modules")); err == nil {
			return cur, true
		}
		if _, err := os.Stat(filepath.Join(cur, "go.work")); err == nil {
			return cur, true
		}
		parent := filepath.Dir(cur)
		if parent == cur {
			break
		}
		cur = parent
	}
	return "", false
}

// officeFindRepoRoot / portfolioFindRepoRoot are backward-compat aliases.
func officeFindRepoRoot() (string, bool)     { return findRepoRoot() }
func portfolioFindRepoRoot() (string, bool)  { return findRepoRoot() }

// resolveBinaryHint returns the resolved binary path or an empty string.
func resolveBinaryHint(cfg RustBridgeConfig) string {
	if p, err := resolveRustBin(cfg); err == nil {
		return p
	}
	return ""
}

func officeResolveBinaryHint() string     { return resolveBinaryHint(OfficeExeConfig) }
func officeResolveBinary() (string, error) { return resolveRustBin(OfficeExeConfig) }

func portfolioResolveBinary() (string, error) { return resolveRustBin(PortfolioExeConfig) }
func portfolioResolveBinaryHint() string { return resolveBinaryHint(PortfolioExeConfig) }

// =============================================================================
// §7 — Platform-specific dynamic linking (DLL) implementation stubs
//
// The real implementations would use syscall on Windows and dlopen/dlsym on
// Linux/macOS via unsafe.Pointer tricks.  For production use, consider the
// "github.com/ebitengine/purego" package which provides a CGo-free dlopen/dlsym.
// =============================================================================
