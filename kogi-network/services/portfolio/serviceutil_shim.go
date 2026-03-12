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

func reportHealth(gatewayBase, serviceID string, healthy bool, note string) {
	serviceutil.ReportHealth(gatewayBase, serviceID, healthy, note)
}

func healthLoop(gatewayBase, serviceID string, interval time.Duration, checkFn func() (healthy bool, note string)) {
	serviceutil.HealthLoop(gatewayBase, serviceID, interval, checkFn)
}

func pollDeadLetters(gatewayBase string, predicate func(topic string) bool, logFn func(id, topic string)) {
	serviceutil.PollDeadLetters(gatewayBase, predicate, logFn)
}

func pollInbound(gatewayBase string, topics []string) {
	serviceutil.PollInbound(gatewayBase, topics)
}

func registerWithGateway(
	gatewayBase, serviceID, selfEndpoint, healthPath string,
	publishTopics, subscribeTopics []string,
	prefixSubs []struct{ Prefix, Consumer string },
) {
	serviceutil.RegisterWithGateway(gatewayBase, serviceID, selfEndpoint, healthPath, publishTopics, subscribeTopics, prefixSubs)
}

func writeJSON(w http.ResponseWriter, status int, payload interface{}) {
	serviceutil.WriteJSON(w, status, payload)
}

func decodeBody(r *http.Request, dst interface{}) error {
	return serviceutil.DecodeBody(r, dst)
}

func trimPrefix(r *http.Request, prefix string) string {
	return serviceutil.TrimPrefix(r, prefix)
}

func resolveAddr(defaultPort, envKey string) string {
	return serviceutil.ResolveAddr(defaultPort, envKey)
}

func portfolioResolveBinary() (string, error) {
	return serviceutil.PortfolioResolveBinary()
}

func portfolioResolveBinaryHint() string {
	return serviceutil.PortfolioResolveBinaryHint()
}

func portfolioRust(funcName string, payload, fallback interface{}) interface{} {
	return serviceutil.PortfolioRust(funcName, payload, fallback)
}

func portfolioCallRust(funcName string, payload interface{}) (interface{}, error) {
	return serviceutil.PortfolioCallRust(funcName, payload)
}

func portfolioProxyGateway(w http.ResponseWriter, path string, fallback interface{}) {
	serviceutil.ProxyGateway(w, portfolioGateway, path, fallback)
}

var PortfolioDLLConfig = serviceutil.PortfolioDLLConfig
