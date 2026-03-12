package serviceutil

import (
	"net/http"
	"time"
)

// Publish sends a pub/sub event to the gateway bus.
func Publish(gatewayBase, sourceID, topic, payload string, meta map[string]string) {
	publish(gatewayBase, sourceID, topic, payload, meta)
}

// GatewaySubscribePrefix registers a prefix subscription on the gateway.
func GatewaySubscribePrefix(gatewayBase, prefix, consumer string) {
	gatewaySubscribePrefix(gatewayBase, prefix, consumer)
}

// SubscribeGatewayPrefix polls the gateway prefix-history endpoint.
func SubscribeGatewayPrefix(gatewayBase string, prefixes []string, handler func(topic, payload string)) {
	subscribeGatewayPrefix(gatewayBase, prefixes, handler)
}

// SubscribeGatewayExact polls the gateway history endpoint for exact topics.
func SubscribeGatewayExact(gatewayBase string, topics []string, handler func(topic, payload string)) {
	subscribeGatewayExact(gatewayBase, topics, handler)
}

// ReplayGateway fetches a time-windowed replay from the gateway.
func ReplayGateway(
	gatewayBase, topic, prefix, from, to string,
	appendFn func(topic, payload, source string),
) {
	replayGateway(gatewayBase, topic, prefix, from, to, appendFn)
}

// PollDeadLetters polls the gateway dead-letter queue.
func PollDeadLetters(gatewayBase string, predicate func(topic string) bool, logFn func(id, topic string)) {
	pollDeadLetters(gatewayBase, predicate, logFn)
}

// PollInbound polls the gateway /pubsub/receive endpoint.
func PollInbound(gatewayBase string, topics []string) {
	pollInbound(gatewayBase, topics)
}

// RegisterWithGateway registers a service with the gateway mesh registry.
func RegisterWithGateway(
	gatewayBase, serviceID, selfEndpoint, healthPath string,
	publishTopics, subscribeTopics []string,
	prefixSubs []struct{ Prefix, Consumer string },
) {
	registerWithGateway(gatewayBase, serviceID, selfEndpoint, healthPath, publishTopics, subscribeTopics, prefixSubs)
}

// ReportHealth sends a health record to the gateway mesh registry.
func ReportHealth(gatewayBase, serviceID string, healthy bool, note string) {
	reportHealth(gatewayBase, serviceID, healthy, note)
}

// HealthLoop reports health on an interval.
func HealthLoop(gatewayBase, serviceID string, interval time.Duration, checkFn func() (healthy bool, note string)) {
	healthLoop(gatewayBase, serviceID, interval, checkFn)
}

// WriteJSON encodes payload as JSON and writes it with the given status code.
func WriteJSON(w http.ResponseWriter, status int, payload interface{}) {
	writeJSON(w, status, payload)
}

// DecodeBody reads r.Body into dst and returns an error if the JSON is invalid.
func DecodeBody(r *http.Request, dst interface{}) error {
	return decodeBody(r, dst)
}

// TrimPrefix removes prefix from r.URL.Path.
func TrimPrefix(r *http.Request, prefix string) string {
	return trimPrefix(r, prefix)
}

// ResolveAddr returns a listen address, checking envKey then KOGI_PORT then default.
func ResolveAddr(defaultPort, envKey string) string {
	return resolveAddr(defaultPort, envKey)
}

// ToListenAddr ensures the port string starts with ":".
func ToListenAddr(port string) string {
	return toListenAddr(port)
}

// ProxyGateway performs a GET to gatewayBase+path and copies the response to w.
func ProxyGateway(w http.ResponseWriter, gatewayBase, path string, fallback interface{}) {
	proxyGateway(w, gatewayBase, path, fallback)
}

// OfficeRust calls the office Rust bridge and returns fallback on error.
func OfficeRust(funcName string, payload, fallback interface{}) interface{} {
	return officeRust(funcName, payload, fallback)
}

// OfficeCallRust calls the office Rust bridge and returns error on failure.
func OfficeCallRust(funcName string, payload interface{}) (interface{}, error) {
	return officeCallRust(funcName, payload)
}

// OfficeResolveBinary resolves the office executable path.
func OfficeResolveBinary() (string, error) {
	return officeResolveBinary()
}

// OfficeResolveBinaryHint returns a human-readable lookup hint.
func OfficeResolveBinaryHint() string {
	return officeResolveBinaryHint()
}

// PortfolioRust calls the portfolio Rust bridge and returns fallback on error.
func PortfolioRust(funcName string, payload, fallback interface{}) interface{} {
	return portfolioRust(funcName, payload, fallback)
}

// PortfolioCallRust calls the portfolio Rust bridge and returns error on failure.
func PortfolioCallRust(funcName string, payload interface{}) (interface{}, error) {
	return portfolioCallRust(funcName, payload)
}

// PortfolioResolveBinary resolves the portfolio executable path.
func PortfolioResolveBinary() (string, error) {
	return portfolioResolveBinary()
}

// PortfolioResolveBinaryHint returns a human-readable lookup hint.
func PortfolioResolveBinaryHint() string {
	return portfolioResolveBinaryHint()
}
