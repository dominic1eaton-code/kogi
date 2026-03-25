package serviceutil

import (
	"path/filepath"
	"time"
)

// MarketplaceDLLConfig is the bridge config for the kogi_marketplace DLL.
var MarketplaceDLLConfig = RustBridgeConfig{
	Mode:          RustModeDLL,
	EnvBinKey:     "KOGI_MARKETPLACE_DLL",
	WellKnownName: "kogi_marketplace",
	RepoSubPaths: []string{
		filepath.Join("kogi-marketplace", "target", "release"),
		filepath.Join("kogi-marketplace", "target", "debug"),
		filepath.Join("kogi-marketplace"),
	},
	ServiceLabel: "marketplace-dll",
	Timeout:      5 * time.Second,
}

// MarketplaceExeConfig is the fallback exe bridge config for the marketplace service.
var MarketplaceExeConfig = RustBridgeConfig{
	Mode:          RustModeExe,
	EnvBinKey:     "KOGI_MARKETPLACE_SYSTEM_BIN",
	WellKnownName: "kogi-marketplace-system",
	RepoSubPaths: []string{
		filepath.Join("kogi-marketplace", "target", "debug"),
		filepath.Join("kogi-marketplace", "target", "release"),
		filepath.Join("kogi-marketplace"),
	},
	ServiceLabel: "marketplace-svc",
	Timeout:      5 * time.Second,
}

func callMarketplaceDLL(funcName string, payload interface{}) (interface{}, error) {
	result, err := CallRust(MarketplaceDLLConfig, funcName, payload)
	if err != nil {
		return CallRust(MarketplaceExeConfig, funcName, payload)
	}
	return result, nil
}

func marketplaceRust(funcName string, payload, fallback interface{}) interface{} {
	r, err := callMarketplaceDLL(funcName, payload)
	if err != nil {
		return fallback
	}
	return r
}

func marketplaceCallRust(funcName string, payload interface{}) (interface{}, error) {
	return callMarketplaceDLL(funcName, payload)
}

func marketplaceResolveBinary() (string, error) {
	return resolveRustBin(MarketplaceExeConfig)
}

func marketplaceResolveBinaryHint() string {
	return resolveBinaryHint(MarketplaceExeConfig)
}
