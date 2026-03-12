//go:build windows
// +build windows

package serviceutil

import (
	"fmt"
	"unsafe"
)

// openSharedLibPath is a Windows stub that forces DLL calls to fall back to exe mode.
func openSharedLibPath(path string) (*sharedLib, error) {
	return nil, fmt.Errorf("dll loading not implemented on windows without cgo")
}

// closeSharedLib closes an open shared library handle (noop on Windows stub).
func closeSharedLib(lib *sharedLib) {
	if lib == nil {
		return
	}
}

// lookupSymbol resolves a named export (stub).
func lookupSymbol(lib *sharedLib, name string) (symbol, error) {
	return symbol{}, fmt.Errorf("dlsym not implemented on windows without cgo")
}

// callSymNoArgs calls a symbol with no args (stub).
func callSymNoArgs(sym symbol) unsafe.Pointer {
	return nil
}

// callSymWithArgs calls a symbol with args (stub).
func callSymWithArgs(sym symbol, argsJSON string) unsafe.Pointer {
	return nil
}

// callFreeString releases the returned string (stub).
func callFreeString(sym symbol, ptr unsafe.Pointer) {
}

// cStringToGoString converts a C string pointer to Go string (stub).
func cStringToGoString(ptr unsafe.Pointer) string {
	return ""
}
