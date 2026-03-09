# KOGI OS Architecture

## Overview
KOGI OS is structured into two primary components:
- **Kernel** (`kernel.zig`): Handles all core, low-level, privileged operations, hardware abstraction, protection, privacy, and security. Runs in kernel mode.
- **System** (`system.zig`): Manages UI, application, and user-level logic. Supports custom user applications and independent worker programs. Runs in user mode.

## Modes
- **Kernel Mode**: Full access to hardware and core OS resources. Only kernel code executes here. Protection barrier prevents user code from accessing privileged operations directly.
- **User Mode**: Restricted access. User applications and system-level UI/app logic run here. Requests to kernel are mediated via a secure interface.

## Protection, Privacy, Security Barrier
- Enforced by the kernel using mode checks and explicit API boundaries.
- System and user applications cannot access kernel internals or privileged resources directly.
- All communication between system and kernel passes through validated interfaces (see `enforceModeBarrier`).

## Custom User Applications
- System provides APIs and management for launching, monitoring, and controlling independent worker programs.
- Kernel schedules, isolates, and secures these processes.

## File Structure
- `kernel.zig`: Kernel struct, mode switching, protection barrier logic.
- `system.zig`: System struct, UI/app/session/dashboard/app manager, kernel pointer, user mode logic.

## Example Usage
```zig
const kernel = Kernel.init(allocator);
const system = System.init(allocator, &kernel);
```

## Extensibility
- Add new subsystems to either kernel or system as needed.
- Extend protection barrier logic for advanced security models.
