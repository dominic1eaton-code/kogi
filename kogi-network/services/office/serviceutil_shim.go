package main

import (
	"net/http"
	"time"

	"kogi.network/lib/serviceutil"
)

func publish(gatewayBase, sourceID, topic, payload string, meta map[string]string) {
	serviceutil.Publish(gatewayBase, sourceID, topic, payload, meta)
}

func gatewaySubscribePrefix(gatewayBase, prefix, consumer string) {
	serviceutil.GatewaySubscribePrefix(gatewayBase, prefix, consumer)
}

func subscribeGatewayExact(gatewayBase string, topics []string, handler func(topic, payload string)) {
	serviceutil.SubscribeGatewayExact(gatewayBase, topics, handler)
}

func subscribeGatewayPrefix(gatewayBase string, prefixes []string, handler func(topic, payload string)) {
	serviceutil.SubscribeGatewayPrefix(gatewayBase, prefixes, handler)
}

func replayGateway(
	gatewayBase, topic, prefix, from, to string,
	appendFn func(topic, payload, source string),
) {
	serviceutil.ReplayGateway(gatewayBase, topic, prefix, from, to, appendFn)
}

func pollDeadLetters(gatewayBase string, predicate func(topic string) bool, logFn func(id, topic string)) {
	serviceutil.PollDeadLetters(gatewayBase, predicate, logFn)
}

func reportHealth(gatewayBase, serviceID string, healthy bool, note string) {
	serviceutil.ReportHealth(gatewayBase, serviceID, healthy, note)
}

func healthLoop(gatewayBase, serviceID string, interval time.Duration, checkFn func() (healthy bool, note string)) {
	serviceutil.HealthLoop(gatewayBase, serviceID, interval, checkFn)
}

func registerWithGateway(
	gatewayBase, serviceID, selfEndpoint, healthPath string,
	publishTopics, subscribeTopics []string,
	prefixSubs []struct{ Prefix, Consumer string },
) {
	serviceutil.RegisterWithGateway(gatewayBase, serviceID, selfEndpoint, healthPath, publishTopics, subscribeTopics, prefixSubs)
}

func writeOfficeJSON(w http.ResponseWriter, status int, payload interface{}) {
	serviceutil.WriteJSON(w, status, payload)
}

func decodeOfficeBody(r *http.Request, dst interface{}) error {
	return serviceutil.DecodeBody(r, dst)
}

func officeTrimPrefix(r *http.Request, prefix string) string {
	return serviceutil.TrimPrefix(r, prefix)
}

func resolveOfficeAddr(defaultPort, envKey string) string {
	return serviceutil.ResolveAddr(defaultPort, envKey)
}

func officeResolveBinary() (string, error) {
	return serviceutil.OfficeResolveBinary()
}

func officeResolveBinaryHint() string {
	return serviceutil.OfficeResolveBinaryHint()
}

func officeRust(funcName string, payload, fallback interface{}) interface{} {
	return serviceutil.OfficeRust(funcName, payload, fallback)
}

func officeCallRust(funcName string, payload interface{}) (interface{}, error) {
	return serviceutil.OfficeCallRust(funcName, payload)
}

func officeProxyGateway(w http.ResponseWriter, path string, fallback interface{}) {
	serviceutil.ProxyGateway(w, officeGateway, path, fallback)
}

var OfficeDLLConfig = serviceutil.OfficeDLLConfig
var OfficeExeConfig = serviceutil.OfficeExeConfig

type responseRecorder struct {
	header http.Header
	code   int
	body   []byte
}

func (r *responseRecorder) Header() http.Header { return r.header }
func (r *responseRecorder) WriteHeader(code int) { r.code = code }
func (r *responseRecorder) Write(b []byte) (int, error) {
	r.body = append(r.body, b...)
	return len(b), nil
}
