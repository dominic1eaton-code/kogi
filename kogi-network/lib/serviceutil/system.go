//go:build !windows
// +build !windows

package serviceutil

// dll_unix.go
//
// cgo-free dlopen/dlsym implementation for Linux and macOS using syscall.
// On these platforms the Rust DLL is a .so (Linux) or .dylib (macOS).

import (
	"fmt"
	"syscall"
	"unsafe"
)

// openSharedLibPath opens a shared library by its absolute path.
func openSharedLibPath(path string) (*sharedLib, error) {
	const RTLD_NOW = 0x2
	handle, _, err := syscall.Syscall(syscall.SYS_OPEN, // replaced below
		uintptr(unsafe.Pointer(syscall.StringBytePtr(path))),
		uintptr(RTLD_NOW),
		0,
	)
	// The above is a simplification.  The real dlopen call is:
	//   handle = dlopen(path, RTLD_NOW)
	// We approximate it via the purego pattern.  For production use, import
	// "github.com/ebitengine/purego" which provides OpenLibrary / NewCallback.
	//
	// Fallback: return an error so callRustDLL degrades to the exe bridge.
	_ = handle
	_ = err
	return nil, fmt.Errorf("dlopen not implemented without cgo; use purego or exe bridge")
}

// closeSharedLib closes an open shared library handle.
func closeSharedLib(lib *sharedLib) {
	if lib == nil {
		return
	}
	// dlclose(lib.handle) would go here.
}

// lookupSymbol resolves a named export from an open shared library.
func lookupSymbol(lib *sharedLib, name string) (symbol, error) {
	// dlsym(lib.handle, name) would go here.
	return symbol{}, fmt.Errorf("dlsym not implemented without cgo")
}

// callSymNoArgs calls a symbol with no arguments and returns the result pointer.
func callSymNoArgs(sym symbol) unsafe.Pointer {
	if sym.ptr == 0 {
		return nil
	}
	fn := *(*func() uintptr)(unsafe.Pointer(&sym.ptr))
	r := fn()
	return unsafe.Pointer(r) //nolint:unsafeptr
}

// callSymWithArgs calls a symbol with a JSON string argument.
func callSymWithArgs(sym symbol, argsJSON string) unsafe.Pointer {
	if sym.ptr == 0 {
		return nil
	}
	cstr, _ := syscall.BytePtrFromString(argsJSON)
	fn := *(*func(uintptr) uintptr)(unsafe.Pointer(&sym.ptr))
	r := fn(uintptr(unsafe.Pointer(cstr)))
	return unsafe.Pointer(r) //nolint:unsafeptr
}

// callFreeString calls the kogi_free_string symbol to release a returned string.
func callFreeString(sym symbol, ptr unsafe.Pointer) {
	if sym.ptr == 0 || ptr == nil {
		return
	}
	fn := *(*func(uintptr))(unsafe.Pointer(&sym.ptr))
	fn(uintptr(ptr))
}

// cStringToGoString converts a C null-terminated string pointer to a Go string.
func cStringToGoString(ptr unsafe.Pointer) string {
	if ptr == nil {
		return ""
	}
	// Walk until NUL byte.
	n := 0
	for *(*byte)(unsafe.Pointer(uintptr(ptr) + uintptr(n))) != 0 {
		n++
	}
	return string((*[1 << 30]byte)(ptr)[:n:n])
}
