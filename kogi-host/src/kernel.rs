//! kernel.rs — Rust ↔ Zig Kernel Bridge
//!
//! This module owns the complete Rust-side interface to the Zig kernel.
//! It is structured in three layers:
//!
//! ┌──────────────────────────────────────────────────────────┐
//! │  KernelBridge trait  — stable, mockable Rust interface   │
//! ├──────────────────────────────────────────────────────────┤
//! │  ZigKernelBridge     — production impl via FFI syscalls  │
//! ├──────────────────────────────────────────────────────────┤
//! │  ffi (unsafe)        — raw extern "C" declarations       │
//! └──────────────────────────────────────────────────────────┘
//!
//! The FFI symbols are exported by `ffi.zig` and linked from the Zig kernel
//! shared library (`libkogi_kernel`).  Every symbol follows the
//! `kogi_kernel_*` naming convention so the linker needs no rename map.
//!
//! Usage
//! ─────
//! ```rust
//! let mut bridge = ZigKernelBridge::new()?;
//!
//! bridge.register_module("my.mod", "worker")?;
//! bridge.start_module("my.mod")?;
//! bridge.publish_event("my.mod.ready", r#"{"status":"ok"}"#)?;
//!
//! let stats = bridge.get_stats()?;
//! println!("active modules: {}", stats.active_modules);
//!
//! bridge.stop_module("my.mod")?;
//! // ZigKernelBridge::drop() calls kogi_kernel_deinit automatically.
//! ```

use std::ffi::{CString, c_char, c_int};

// ─────────────────────────────────────────────────────────────────────────────
// C-compatible structs  (must mirror the `extern struct` types in ffi.zig)
// ─────────────────────────────────────────────────────────────────────────────

/// Resource limits passed to `kogi_kernel_register_component`.
/// Memory is expressed in MiB to match the Zig side.
///
/// Mirrors `FfiResourceLimits` in ffi.zig — do **not** reorder fields.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FfiResourceLimits {
    pub memory_limit_mb:    u64,
    pub max_processes:      u32,
    pub max_files:          u32,
    pub max_resource_units: u32,
    pub max_connections:    u32,
}

impl Default for FfiResourceLimits {
    fn default() -> Self {
        Self {
            memory_limit_mb:    64,
            max_processes:      16,
            max_files:          1024,
            max_resource_units: 512,
            max_connections:    32,
        }
    }
}

/// Atomic kernel stats snapshot returned by `kogi_kernel_get_stats`.
/// All `usize` kernel fields are widened to `u64` for cross-platform safety.
///
/// Mirrors `FfiKernelStats` in ffi.zig — do **not** reorder fields.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct FfiKernelStats {
    // Privilege  (0 = privileged, 1 = user)
    pub mode: i32,

    // Memory (MiB)
    pub memory_used_mb:  u64,
    pub memory_total_mb: u64,

    // Processes
    pub live_processes: u64,
    pub idle_workers:   u64,
    pub busy_workers:   u64,
    pub queued_tasks:   u64,

    // Modules
    pub total_modules:  u64,
    pub active_modules: u64,

    // Services
    pub total_services:   u64,
    pub running_services: u64,
    pub faulted_services: u64,

    // Network
    pub open_sockets:       u64,
    pub active_connections: u64,

    // Events
    pub kernel_events_total:   u64,
    pub kernel_events_log_len: u64,
    pub kernel_event_overflow: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// ComponentClass — mirrors the i32 class_tag convention in ffi.zig
// ─────────────────────────────────────────────────────────────────────────────

/// Class tag values that map to `mod_mod.ModuleClass` in the Zig kernel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ComponentClass {
    Kernel   = 0,
    Host     = 1,
    Server   = 2,
    Engine   = 3,
    Services = 4,
    Module   = 5,
}

// ─────────────────────────────────────────────────────────────────────────────
// KernelMode
// ─────────────────────────────────────────────────────────────────────────────

/// Kernel privilege mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelMode {
    /// Full kernel-level access (requires host/root role on the Zig side).
    Privileged,
    /// Unprivileged shell; read-only inspection only.
    User,
}

// ─────────────────────────────────────────────────────────────────────────────
// KernelBridge trait
// ─────────────────────────────────────────────────────────────────────────────

/// Stable, mockable interface to the Zig kernel.
///
/// Both `ZigKernelBridge` (production) and `MockKernelBridge` (tests) implement
/// this trait so callers never take a concrete type dependency on FFI.
pub trait KernelBridge {
    // ── Module management ────────────────────────────────────────────────────

    /// Register a module by `id` (unique dotted name) and `kind` (role string).
    fn register_module(&mut self, id: &str, kind: &str) -> Result<(), String>;

    /// Start a previously registered module.
    fn start_module(&mut self, id: &str) -> Result<(), String>;

    /// Stop a running module without deregistering it.
    fn stop_module(&mut self, id: &str) -> Result<(), String>;

    /// Stop and fully deregister a module, releasing all resources.
    fn remove_module(&mut self, id: &str) -> Result<(), String>;

    // ── Component registration ───────────────────────────────────────────────

    /// Register a platform component with a full network descriptor and
    /// explicit resource limits.  The component is started immediately.
    fn register_component(
        &mut self,
        id:              &str,
        class:           ComponentClass,
        endpoint:        &str,
        network_manager: &str,
        limits:          &FfiResourceLimits,
    ) -> Result<(), String>;

    // ── Events ───────────────────────────────────────────────────────────────

    /// Publish a custom cross-cutting event to the kernel event log.
    fn publish_event(&mut self, topic: &str, payload: &str) -> Result<(), String>;

    // ── Privilege / mode ─────────────────────────────────────────────────────

    /// Switch the kernel's privilege mode.
    fn set_mode(&mut self, mode: KernelMode) -> Result<(), String>;

    /// Elevate the host session to privileged mode.
    fn enter_privileged(&mut self) -> Result<(), String>;

    /// Drop the host session back to user mode.
    fn exit_privileged(&mut self) -> Result<(), String>;

    // ── Bootstrap ────────────────────────────────────────────────────────────

    /// Bootstrap all five core platform components in a single call.
    fn bootstrap_core(&mut self) -> Result<(), String>;

    /// Provision the kogi.office module with its canonical resource budget.
    fn bootstrap_office(&mut self) -> Result<(), String>;

    // ── Introspection ────────────────────────────────────────────────────────

    /// Return a snapshot of kernel-wide stats.
    fn get_stats(&self) -> Result<FfiKernelStats, String>;

    /// Return the total number of events recorded since init.
    fn event_count(&self) -> u64;

    /// Return the total number of registered modules.
    fn module_count(&self) -> u64;
}

// ─────────────────────────────────────────────────────────────────────────────
// Raw FFI declarations  (unsafe — do not call directly)
// ─────────────────────────────────────────────────────────────────────────────

#[link(name = "kogi_kernel")]
unsafe extern "C" {
    // Lifecycle
    fn kogi_kernel_init() -> bool;
    fn kogi_kernel_deinit();

    // Privilege / mode
    fn kogi_kernel_set_mode(mode: c_int) -> bool;
    fn kogi_kernel_enter_privileged() -> bool;
    fn kogi_kernel_exit_privileged() -> bool;

    // Module management
    fn kogi_kernel_register_module(id: *const c_char, kind: *const c_char) -> bool;
    fn kogi_kernel_module_start(id: *const c_char) -> bool;
    fn kogi_kernel_module_stop(id: *const c_char) -> bool;
    fn kogi_kernel_module_remove(id: *const c_char) -> bool;

    // Component registration
    fn kogi_kernel_register_component(
        id:              *const c_char,
        class_tag:       c_int,
        endpoint:        *const c_char,
        network_manager: *const c_char,
        limits:          *const FfiResourceLimits,
    ) -> bool;

    // Bootstrap helpers
    fn kogi_kernel_bootstrap_core()   -> bool;
    fn kogi_kernel_bootstrap_office() -> bool;

    // Events
    fn kogi_kernel_publish_event(topic: *const c_char, payload: *const c_char) -> bool;
    fn kogi_kernel_event_count() -> u64;

    // Stats / introspection
    fn kogi_kernel_module_count() -> u64;
    fn kogi_kernel_get_stats(out: *mut FfiKernelStats) -> bool;
}

// ─────────────────────────────────────────────────────────────────────────────
// ZigKernelBridge — production implementation via FFI syscalls
// ─────────────────────────────────────────────────────────────────────────────

/// Production `KernelBridge` backed by the Zig kernel via `ffi.zig` syscalls.
///
/// Initialises the kernel singleton on construction and automatically tears it
/// down on `Drop`.  Only one `ZigKernelBridge` should exist per process; the
/// Zig side guards against double-init and returns `true` if already running.
pub struct ZigKernelBridge;

impl ZigKernelBridge {
    /// Initialise the Zig kernel and return a bridge handle.
    ///
    /// # Errors
    /// Returns an error string if `kogi_kernel_init` returns false, which
    /// indicates that the Zig kernel failed to boot (allocator error, event
    /// subsystem failure, etc.).
    pub fn new() -> Result<Self, String> {
        let ok = unsafe { kogi_kernel_init() };
        if ok {
            Ok(Self)
        } else {
            Err("kogi_kernel_init: kernel boot failed".into())
        }
    }

    // ── CString helpers ──────────────────────────────────────────────────────

    fn cstr(s: &str) -> Result<CString, String> {
        CString::new(s).map_err(|e| format!("invalid string: {e}"))
    }

    fn syscall_bool(ok: bool, name: &'static str) -> Result<(), String> {
        if ok { Ok(()) } else { Err(format!("syscall {name} returned false")) }
    }
}

impl Drop for ZigKernelBridge {
    fn drop(&mut self) {
        unsafe { kogi_kernel_deinit() }
    }
}

impl KernelBridge for ZigKernelBridge {
    // ── Module management ────────────────────────────────────────────────────

    fn register_module(&mut self, id: &str, kind: &str) -> Result<(), String> {
        let c_id   = Self::cstr(id)?;
        let c_kind = Self::cstr(kind)?;
        let ok = unsafe { kogi_kernel_register_module(c_id.as_ptr(), c_kind.as_ptr()) };
        Self::syscall_bool(ok, "kogi_kernel_register_module")
    }

    fn start_module(&mut self, id: &str) -> Result<(), String> {
        let c_id = Self::cstr(id)?;
        let ok = unsafe { kogi_kernel_module_start(c_id.as_ptr()) };
        Self::syscall_bool(ok, "kogi_kernel_module_start")
    }

    fn stop_module(&mut self, id: &str) -> Result<(), String> {
        let c_id = Self::cstr(id)?;
        let ok = unsafe { kogi_kernel_module_stop(c_id.as_ptr()) };
        Self::syscall_bool(ok, "kogi_kernel_module_stop")
    }

    fn remove_module(&mut self, id: &str) -> Result<(), String> {
        let c_id = Self::cstr(id)?;
        let ok = unsafe { kogi_kernel_module_remove(c_id.as_ptr()) };
        Self::syscall_bool(ok, "kogi_kernel_module_remove")
    }

    // ── Component registration ───────────────────────────────────────────────

    fn register_component(
        &mut self,
        id:              &str,
        class:           ComponentClass,
        endpoint:        &str,
        network_manager: &str,
        limits:          &FfiResourceLimits,
    ) -> Result<(), String> {
        let c_id      = Self::cstr(id)?;
        let c_ep      = Self::cstr(endpoint)?;
        let c_net_mgr = Self::cstr(network_manager)?;
        let ok = unsafe {
            kogi_kernel_register_component(
                c_id.as_ptr(),
                class as c_int,
                c_ep.as_ptr(),
                c_net_mgr.as_ptr(),
                limits as *const FfiResourceLimits,
            )
        };
        Self::syscall_bool(ok, "kogi_kernel_register_component")
    }

    // ── Events ───────────────────────────────────────────────────────────────

    fn publish_event(&mut self, topic: &str, payload: &str) -> Result<(), String> {
        let c_topic   = Self::cstr(topic)?;
        let c_payload = Self::cstr(payload)?;
        let ok = unsafe { kogi_kernel_publish_event(c_topic.as_ptr(), c_payload.as_ptr()) };
        Self::syscall_bool(ok, "kogi_kernel_publish_event")
    }

    // ── Privilege / mode ─────────────────────────────────────────────────────

    fn set_mode(&mut self, mode: KernelMode) -> Result<(), String> {
        let tag = match mode {
            KernelMode::Privileged => 0,
            KernelMode::User       => 1,
        };
        let ok = unsafe { kogi_kernel_set_mode(tag) };
        Self::syscall_bool(ok, "kogi_kernel_set_mode")
    }

    fn enter_privileged(&mut self) -> Result<(), String> {
        let ok = unsafe { kogi_kernel_enter_privileged() };
        Self::syscall_bool(ok, "kogi_kernel_enter_privileged")
    }

    fn exit_privileged(&mut self) -> Result<(), String> {
        let ok = unsafe { kogi_kernel_exit_privileged() };
        Self::syscall_bool(ok, "kogi_kernel_exit_privileged")
    }

    // ── Bootstrap ────────────────────────────────────────────────────────────

    fn bootstrap_core(&mut self) -> Result<(), String> {
        let ok = unsafe { kogi_kernel_bootstrap_core() };
        Self::syscall_bool(ok, "kogi_kernel_bootstrap_core")
    }

    fn bootstrap_office(&mut self) -> Result<(), String> {
        let ok = unsafe { kogi_kernel_bootstrap_office() };
        Self::syscall_bool(ok, "kogi_kernel_bootstrap_office")
    }

    // ── Introspection ────────────────────────────────────────────────────────

    fn get_stats(&self) -> Result<FfiKernelStats, String> {
        let mut stats = FfiKernelStats::default();
        let ok = unsafe { kogi_kernel_get_stats(&mut stats as *mut FfiKernelStats) };
        if ok { Ok(stats) } else { Err("kogi_kernel_get_stats: kernel not initialised".into()) }
    }

    fn event_count(&self) -> u64 {
        unsafe { kogi_kernel_event_count() }
    }

    fn module_count(&self) -> u64 {
        unsafe { kogi_kernel_module_count() }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MockKernelBridge — in-process stub for unit tests
// ─────────────────────────────────────────────────────────────────────────────

/// In-memory `KernelBridge` implementation for unit tests.  No FFI calls are
/// made; all operations are recorded as counters that tests can inspect.
#[derive(Debug, Default)]
pub struct MockKernelBridge {
    pub registered_modules:    usize,
    pub registered_components: usize,
    pub published_events:      usize,
    pub mode:                  &'static str,
}

impl MockKernelBridge {
    pub fn new() -> Self {
        Self { mode: "privileged", ..Default::default() }
    }
}

impl KernelBridge for MockKernelBridge {
    fn register_module(&mut self, id: &str, kind: &str) -> Result<(), String> {
        self.registered_modules += 1;
        println!("[mock-kernel] register_module id={id} kind={kind}");
        Ok(())
    }

    fn start_module(&mut self, id: &str) -> Result<(), String> {
        println!("[mock-kernel] start_module id={id}");
        Ok(())
    }

    fn stop_module(&mut self, id: &str) -> Result<(), String> {
        println!("[mock-kernel] stop_module id={id}");
        Ok(())
    }

    fn remove_module(&mut self, id: &str) -> Result<(), String> {
        println!("[mock-kernel] remove_module id={id}");
        Ok(())
    }

    fn register_component(
        &mut self,
        id:              &str,
        class:           ComponentClass,
        endpoint:        &str,
        network_manager: &str,
        limits:          &FfiResourceLimits,
    ) -> Result<(), String> {
        self.registered_components += 1;
        println!(
            "[mock-kernel] register_component id={id} class={class:?} \
             endpoint={endpoint} net_mgr={network_manager} \
             limits(mem={}MiB proc={} files={} res={} conn={})",
            limits.memory_limit_mb,
            limits.max_processes,
            limits.max_files,
            limits.max_resource_units,
            limits.max_connections,
        );
        Ok(())
    }

    fn publish_event(&mut self, topic: &str, payload: &str) -> Result<(), String> {
        self.published_events += 1;
        println!("[mock-kernel] publish_event topic={topic} payload={payload}");
        Ok(())
    }

    fn set_mode(&mut self, mode: KernelMode) -> Result<(), String> {
        self.mode = match mode {
            KernelMode::Privileged => "privileged",
            KernelMode::User       => "user",
        };
        println!("[mock-kernel] set_mode mode={}", self.mode);
        Ok(())
    }

    fn enter_privileged(&mut self) -> Result<(), String> {
        self.mode = "privileged";
        println!("[mock-kernel] enter_privileged");
        Ok(())
    }

    fn exit_privileged(&mut self) -> Result<(), String> {
        self.mode = "user";
        println!("[mock-kernel] exit_privileged");
        Ok(())
    }

    fn bootstrap_core(&mut self) -> Result<(), String> {
        println!("[mock-kernel] bootstrap_core");
        Ok(())
    }

    fn bootstrap_office(&mut self) -> Result<(), String> {
        println!("[mock-kernel] bootstrap_office");
        Ok(())
    }

    fn get_stats(&self) -> Result<FfiKernelStats, String> {
        Ok(FfiKernelStats {
            mode:               if self.mode == "privileged" { 0 } else { 1 },
            total_modules:      self.registered_modules  as u64,
            active_modules:     self.registered_modules  as u64,
            kernel_events_total: self.published_events   as u64,
            ..Default::default()
        })
    }

    fn event_count(&self) -> u64 { self.published_events as u64 }
    fn module_count(&self) -> u64 { self.registered_modules as u64 }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_limits() -> FfiResourceLimits {
        FfiResourceLimits {
            memory_limit_mb:    128,
            max_processes:      8,
            max_files:          512,
            max_resource_units: 256,
            max_connections:    16,
        }
    }

    #[test]
    fn mock_register_module_increments_counter() {
        let mut bridge = MockKernelBridge::new();
        bridge.register_module("test.mod", "worker").unwrap();
        assert_eq!(bridge.registered_modules, 1);
        assert_eq!(bridge.module_count(), 1);
    }

    #[test]
    fn mock_register_component_increments_counter() {
        let mut bridge = MockKernelBridge::new();
        let limits = make_limits();
        bridge
            .register_component("test.server", ComponentClass::Server, "http://0.0.0.0:9000", "kogi-go-network", &limits)
            .unwrap();
        assert_eq!(bridge.registered_components, 1);
    }

    #[test]
    fn mock_publish_event_increments_counter() {
        let mut bridge = MockKernelBridge::new();
        bridge.publish_event("test.topic", r#"{"msg":"hello"}"#).unwrap();
        assert_eq!(bridge.published_events, 1);
        assert_eq!(bridge.event_count(), 1);
    }

    #[test]
    fn mock_mode_transitions() {
        let mut bridge = MockKernelBridge::new();
        assert_eq!(bridge.mode, "privileged");

        bridge.exit_privileged().unwrap();
        assert_eq!(bridge.mode, "user");

        bridge.enter_privileged().unwrap();
        assert_eq!(bridge.mode, "privileged");

        bridge.set_mode(KernelMode::User).unwrap();
        assert_eq!(bridge.mode, "user");
    }

    #[test]
    fn mock_stats_reflects_state() {
        let mut bridge = MockKernelBridge::new();
        bridge.register_module("a", "worker").unwrap();
        bridge.register_module("b", "worker").unwrap();
        bridge.publish_event("e", "{}").unwrap();

        let stats = bridge.get_stats().unwrap();
        assert_eq!(stats.total_modules, 2);
        assert_eq!(stats.kernel_events_total, 1);
        assert_eq!(stats.mode, 0); // privileged
    }

    #[test]
    fn ffi_resource_limits_repr_c_size() {
        // Guard: ensure the C layout hasn't been accidentally widened.
        // u64 + u32*4 = 8 + 16 = 24 bytes (no padding on any supported target).
        assert_eq!(std::mem::size_of::<FfiResourceLimits>(), 24);
    }

    #[test]
    fn ffi_kernel_stats_fields_accessible() {
        let s = FfiKernelStats {
            mode:               0,
            memory_used_mb:     512,
            memory_total_mb:    4096,
            total_modules:      5,
            active_modules:     3,
            kernel_events_total: 42,
            ..Default::default()
        };
        assert_eq!(s.memory_used_mb, 512);
        assert_eq!(s.active_modules, 3);
    }
}