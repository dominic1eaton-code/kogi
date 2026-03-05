# KOGI - Operating System for Independent Workers

**KOGI** is a comprehensive, open-source operating system designed for independent workers of all types: freelancers, contractors, consultants, gig workers, artists, musicians, software developers, gamers, and anyone who wants to organize their work, projects, and assets into a unified, manageable system.

Unlike traditional application-based portfolio management, KOGI is architected as a complete **operating system** for worker autonomy—providing system-level abstractions for identity management, workspace organization, connection registries, contact directories, and asset vaults.

## 🎯 Core Vision

KOGI is an OS (not just an app) that empowers independent workers to:
- **Boot up** a complete personal work system
- **Manage identities** - Create and maintain worker profiles across multiple roles
- **Organize workspaces** - Structure all work artifacts, projects, and deliverables
- **Connect globally** - Registry system for managing accounts across platforms
- **Track relationships** - Directory for contacts, organizations, and business relationships
- **Value assets** - Vault system for tracking resources, equipment, and capital
- **Execute tasks** - Post, assign, and complete work engagements
- **Earn money** - Calculate and track earnings across all work types

## ✨ Core Subsystems

### 👤 Identity Management (User Accounts)

The Identity subsystem manages worker profiles in the KOGI OS, analogous to user accounts in traditional operating systems.

- **Identity Types** - contractors, consultants, freelancers, gig workers, artists, musicians, developers, gamers, custom types
- **Profiles** - names, email, hourly rates, skills, activation status
- **Skills System** - detailed capability tracking for each identity
- **Filtering** - query identities by type and capabilities
- **Status Control** - activate/deactivate identities as needed

### 🌐 Connection Registry

The registry manages all external connections and accounts for identities—like a system mount table or device registry in traditional OS.

- **Connection Types**:
  - Social Media (Twitter, LinkedIn, Instagram, etc.)
  - Work Platforms (Upwork, Fiverr, Freelance sites)
  - Personal Accounts (email, messaging, personal platforms)
  - Email Accounts (Gmail, custom domains, business email)
  - Software Accounts (GitHub, dev tools, SaaS)
  - Custom Connection Types
- **Account Details** - usernames, providers, details, activation status
- **Registry Queries** - find connections by provider or type

### 💼 Personal Workspace

The Workspace is analogous to a user's home directory in traditional OS, organizing all work-related items hierarchically.

**Universal Workspace Collections** - Create collections for anything:
- Programs, Projects, Tasks
- Investments, Financial Assets
- Music, Artwork, Creative Works
- Software, Code, Applications
- Products, Solutions, Services
- Artifacts, Works, Deliverables
- Custom Collections

**Workspace Structure**:
- **Workspace** - Top-level personal container
- **Collections** - Grouped items by type (e.g., "Music", "Code", "Products")
- **Items** - Individual entries (songs, projects, services, etc.)
- **Metadata** - Rich classification for every item

**Workspace Features**:
- Entity type and class classification (19 types, 12 classes)
- Flexible tagging system
- Custom fields for domain-specific data
- Timestamps and ownership tracking
- Hierarchical organization
- Full-text search and filtering

### 🔍 Search & Discovery

- **Index-Based Lookup** - Fast O(n) searches across entire workspace
- **Multi-Criteria Filtering**:
  - Name pattern matching
  - Entity type filtering
  - Entity class filtering
  - Category and tag filtering
  - Owner-based queries
- **Collection Queries** - Find collections by type
- **Direct ID Lookup** - Fast entity retrieval

### 👥 Directory System

The Directory manages contacts and organizations—analogous to system user databases.

- **Organization Management** - Track clients, companies, organizations
  - Organization name, industry, notes
  - Unlimited contacts per organization
- **Contact Tracking** - Professional and personal contacts
  - Name, email, phone
  - Per-contact notes
  - Relationship tracking
- **Directory Queries** - Look up organizations and their contacts

### 💰 Asset Vault

The Vault manages resources, equipment, intellectual property, and capital—like filesystem storage in traditional OS.

- **Asset Types**:
  - Equipment (computers, hardware, instruments, tools)
  - Software (licenses, subscriptions, digital tools)
  - Intellectual Property (patents, designs, trademarks)
  - Furniture & Office Equipment
  - Vehicles
  - Custom Asset Types
- **Asset Tracking** - name, description, value, acquisition date
- **Portfolio Valuation** - calculate total asset value
- **Asset Queries** - inventory and value tracking

### 📋 Task Management & Engagements

The engagement system matches identities (workers) with tasks (work):

- **Task System** - Post work assignments with:
  - Title, description, budget, deadline
  - Skill requirements
  - Assignment tracking
  - Completion status
- **Engagements** - Link identities to tasks
  - Start/end dates
  - Hourly rates
  - Hours tracking
  - Status (active, completed, terminated)
- **Hours Logging** - Track billable hours
- **Earnings Calculation** - Automatic earnings computation (hours × rate)

### 📊 System Statistics & Analytics

- **Identity Stats** - active count, type distribution, connection counts
- **Task Analytics** - total posted, active vs completed, assignments
- **Engagement Reports** - count, status distribution, duration tracking
- **Earnings Reports** - per-identity earnings, total system earnings
- **Workspace Analytics** - collection counts, item distribution, type stats

### 🔐 Security & Access Control

Enterprise-grade security with authentication, authorization, and role-based access control:

#### Authentication & Credentials
- **Credential Management** - Secure password storage with salting and hashing (Argon2, Scrypt, PBKDF2)
- **Session Management** - Token-based sessions with configurable timeouts
- **Account Lockout** - Automatic lockout after failed login attempts
- **Multi-Factor Authentication (MFA)** - TOTP, Email, SMS, Hardware token support
- **Backup Codes** - Recovery codes for MFA-enabled accounts

#### Authorization & Role-Based Access Control (RBAC)
- **System Roles** - Predefined roles with permission sets:
  - **Admin** - Full system access (create/delete users, manage settings, audit logs)
  - **Worker** - Standard user access (manage own work, create tasks, manage workspace)
  - **Contractor** - Limited access (view own work, limited task access)
  - **Moderator** - Content and user management
  - **Guest** - Read-only access to public resources
  - **Custom** - Custom permission sets
- **Permission System** - Granular permissions for all operations:
  - Identity: create, read, update, delete, manage roles
  - Workspace: create, read, update, delete, share
  - Tasks: create, read, update, delete, complete
  - Engagements: create, read, update, delete, complete
  - Directory: manage contacts and organizations
  - Vault: manage assets
  - System: audit logs, settings, user management
- **Role Assignment** - Assign roles to identities with optional expiration
- **Permission Delegation** - Fine-grained permission checking for all operations

#### Access Control
- **Access Levels**:
  - **Private** - Only resource owner has access
  - **Protected** - Owner + specific identities
  - **Internal** - All authenticated users
  - **Public** - Anyone (no auth required)
- **Access Control Lists (ACLs)** - Fine-grained access per resource
- **Ownership Tracking** - Track who owns each resource

#### Audit Logging & Monitoring
- **Comprehensive Audit Log** - All security events tracked with timestamps
- **Event Types**:
  - Login/Logout events
  - Password changes and resets
  - Role assignments and revocations
  - Permission grants and denials
  - Data access, modification, deletion
  - Session management events
  - MFA status changes
  - API key management
  - Suspicious activity
- **Per-Identity Audit Trail** - Query audit logs for specific identities
- **IP Tracking** - Log IP addresses for all security events
- **Device Fingerprinting** - Optional device fingerprint tracking

#### Data Protection Features
- **Secure Session Tokens** - Cryptographically secure session identifiers
- **Password Hashing** - Multiple hash algorithms with configurable salt
- **Automatic Session Expiration** - Configurable session timeout (default 1 hour)
- **Account Lockout** - Prevent brute force attacks
- **Activity Logging** - Track all sensitive operations

### 📝 Logging System

Comprehensive structured logging infrastructure with multiple output destinations and filtering:

#### Logging Features
- **Log Levels** - Trace, Debug, Info, Warn, Error, Fatal
- **Structured Logging** - Modules, messages, context, and metadata
- **Multiple Outputs** - Write to stdout, stderr, files, memory, or network
- **Log Filtering** - Query logs by level or module
- **Ring Buffer** - Configurable maximum entry count (default 10,000)
- **Thread-Safe** - Mutex-protected concurrent access
- **Timestamps** - All entries timestamped for correlation

### 📡 Event Management System

Publish-subscribe event bus for system-wide event handling:

#### Event Types
- **Identity Events** - identity_created, identity_deleted, identity_updated, identity_role_changed
- **Task Events** - task_created, task_completed, task_assigned, task_deleted
- **Engagement Events** - engagement_created, engagement_completed, engagement_terminated
- **Workspace Events** - workspace_created, collection_created, item_added
- **Connection Events** - connection_added, connection_removed
- **Directory Events** - organization_added, contact_added
- **Vault Events** - vault_item_added, vault_item_removed
- **Security Events** - session_created, session_ended, role_assigned
- **State Events** - checkpoint_created, backup_completed, restore_completed
- **Node Events** - node_joined, node_left, node_failed
- **System Events** - system_started, system_shutdown, system_error

#### Event Features
- **Pub-Sub Pattern** - Subscribe to event types with callback handlers
- **Event Publishing** - Atomic event distribution to all subscribers
- **Event History** - Retain event log with configurable max size
- **Event Filtering** - Query events by type
- **Metadata Support** - Rich event data and context

### 💾 State Management System

Sophisticated state management with snapshots, checkpoints, backups, and restore capabilities:

#### Checkpoint & Backup Features
- **Automatic Checkpoints** - Periodic state snapshots (configurable interval)
- **Checkpoint Retrieval** - Restore from any historical checkpoint
- **Compressed Backups** - Support for Gzip, Zstandard, LZ4 compression
- **Backup Retention** - Configurable retention periods with automatic cleanup
- **Checksums** - Verify backup integrity
- **Restore Points** - Named recovery points for easy reference
- **State Hashing** - Detect state changes between checkpoints

#### State Operations
- **Create Checkpoint** - Snapshot current system state with hash
- **Create Backup** - Compress and store checkpoint data
- **Create Restore Point** - Named recovery point for business operations
- **Restore from Checkpoint** - Roll back to any previous checkpoint
- **Restore from Backup** - Recover from archived backup file
- **Auto-Cleanup** - Remove expired backups based on retention policy

### 🔗 Distributed Node System

Enterprise-grade distributed cluster management with redundancy and automatic failover:

#### Cluster Features
- **Multi-Node Clusters** - Support for up to 100+ nodes per cluster
- **Node Roles** - Primary, Replica, Backup, Learner nodes
- **Automatic Failover** - Primary promotion when leader fails
- **Health Monitoring** - Configurable health checks with status tracking
- **Heartbeat Protocol** - Detect node failures and recovery
- **Load Balancing** - Round-robin, least-connections, weighted, random, IP-hash, response-time strategies

#### Node Management
- **Node Registration** - Add nodes to cluster dynamically
- **Status Tracking** - Online, Degraded, Offline, Failing, Recovering, Initializing
- **Health Checks** - CPU, Memory, Disk usage; Response time; Error counts
- **Replication** - State replication between nodes with sequence tracking
- **Cluster Statistics** - Real-time cluster health metrics

#### Failover & Recovery
- **Automatic Detection** - Detect failed nodes via heartbeat timeout
- **Replica Promotion** - Automatically promote replicas to primary
- **Connection Rerouting** - Redirect requests to available nodes
- **Data Replication** - Maintain replicas across multiple nodes
- **Recovery Tracking** - Monitor node recovery progress

### 🌐 Networking & Server System

Complete networking infrastructure for remote access and inter-node communication:

#### Server Features
- **HTTP/HTTPS Server** - Support for multiple protocols (HTTP, gRPC, WebSocket, TCP, UDP)
- **Request Routing** - Path-based endpoint routing with method matching
- **Connection Management** - Track active connections with metadata
- **Request/Response History** - Complete history of all network interactions
- **Keep-Alive Support** - Optional connection reuse
- **Compression** - Optional response compression
- **Thread Pool** - Configurable worker thread pool for concurrent requests

#### API Framework
- **Endpoint Registration** - Register route handlers dynamically
- **Authentication** - Per-endpoint authentication requirements
- **Request/Response Objects** - Structured network communication
- **Headers** - Full header support for both requests and responses
- **Status Codes** - Standard HTTP status codes (200, 400, 401, 403, 404, 500, etc.)

#### Network Client
- **Remote Communication** - Connect to other KOGI OS instances
- **Request Forwarding** - Send requests to remote nodes
- **Load Balancing** - Work with cluster for distributed requests
- **Statistics** - Track sent/received requests and responses
- **Error Handling** - Robust error handling for network failures

### 🛡️ System Architecture

**Modular Design** (Microkernel-like):
- `system.zig` - Core OS engine (System)
- `identity.zig` - Identity management subsystem
- `workspace.zig` - Workspace/personal work environment
- `registry.zig` - Connection registry subsystem
- `directory.zig` - Contact directory subsystem
- `vault.zig` - Asset vault subsystem
- `security.zig` - Security, authentication, authorization, RBAC
- `logging.zig` - Logging system with multiple outputs
- `events.zig` - Event bus and pub-sub pattern
- `state.zig` - State management, checkpoints, backups, restore
- `distributed.zig` - Distributed cluster and node management
- `networking.zig` - Network server, routing, and inter-node communication
- `cli.zig` - Shell interface

**Technology Stack**:
- **Language**: Zig (systems programming language)
- **Memory Model**: GeneralPurposeAllocator with proper lifecycle management
- **Collections**: ArrayLists, StringHashMaps for efficient storage
- **Arch Pattern**: Delegation, separation of concerns
- **Concurrency**: Thread-safe operations with Mutexes
- **Distribution**: Multi-node cluster support with replication

## 🚀 Use Cases

### Freelancers & Contractors
- Manage multiple clients across different platforms
- Track time and earnings across engagements
- Maintain portfolio of completed work
- Organize invoicing information

### Creative Professionals
- Portfolio of artwork, music, creative works
- Client and contact management
- Project tracking and deliverable organization
- Earnings and hours tracking

### Software Developers
- Code and project portfolio
- Multiple platform account management
- Engagement tracking across clients
- Skill and capability maintenance

### Independent Consultants
- Client and prospect management
- Project portfolio and history
- Engagement terms tracking
- Earnings and utilization analytics

### Gig Workers
- Multiple platform account management
- Gig and task assignment tracking
- Per-platform earnings tracking
- Client and rating tracking

### Artists & Musicians
- Portfolio of works (music, art, performances)
- Client/venue management
- Project and commission tracking
- Equipment and asset inventory

## 📦 What's Included

```
kogi/
├── src/
│   ├── main.zig              # OS entry point
│   ├── root.zig              # Library exports
│   ├── system.zig            # Core OS kernel
│   ├── identity.zig          # Identity/user management
│   ├── workspace.zig         # Personal workspace
│   ├── registry.zig          # Connection registry
│   ├── directory.zig         # Contact directory
│   ├── vault.zig             # Asset vault
│   ├── security.zig          # Security & access control
│   ├── logging.zig           # Logging system
│   ├── events.zig            # Event management
│   ├── state.zig             # State management & backups
│   ├── distributed.zig       # Distributed cluster
│   ├── networking.zig        # Networking & server
│   ├── cli.zig               # Shell interface
│   ├── portfolio.zig         # Legacy portfolio (backward compat)
│   ├── kernel.zig            # Legacy kernel (backward compat)
│   ├── worker.zig            # Legacy worker (backward compat)
│   ├── accounts.zig          # Legacy accounts (backward compat)
│   ├── crm.zig               # Legacy CRM (backward compat)
│   ├── assets.zig            # Legacy assets (backward compat)
│   └── tests.zig             # Unit tests
├── build.zig                 # Build configuration
├── build.zig.zon             # Dependency manifest
└── README.md                 # This file
```

## 🔧 Building & Running

### Prerequisites
- Zig 0.15.2 or later

### Build the OS
```bash
zig build
```

### Boot the KOGI OS with Demo
```bash
zig build run
```

### Run System Tests
```bash
zig build test
```

## 📊 Demo

The KOGI OS includes a comprehensive demo that showcases all system subsystems:
- Creates 3 sample identities with different types and skills
- Posts 3 sample tasks with skill requirements
- Creates 3 engagements linking identities to tasks
- Logs hours on engagements
- Displays system statistics
- Shows identity profiles and active tasks
- Generates earnings reports
- Displays all subsystems in action

## 🎓 Architecture Highlights

### OS-Level Design
Rather than application design, KOGI employs **operating system architecture**:

**Subsystem Delegation** - Each subsystem (Identity, Workspace, Registry, Directory, Vault, Security, Logging, Events, State, Distributed, Networking) manages its own domain with clear boundaries and delegation patterns.

**System Kernel** - The System core delegates to specialized subsystem managers rather than implementing all logic itself.

**Separation of Concerns** - Worker identity is completely separate from workspace, which is separate from connections, assets, security, logging, events, state, distribution, and networking.

**Extensibility** - New subsystems can be added without modifying existing ones. Custom entity types, classes, connection types, asset types, events, and more are all pluggable.

**Enterprise-Grade Features** - Built-in support for:
- Distributed deployment across multiple nodes
- Automatic state backups and recovery
- Comprehensive audit logging
- Event-driven architecture
- Redundancy and failover
- Network communication

### 🔄 Process Management System

Process management provides complete lifecycle and resource management for running work processes:

- **Process States** - created, ready, running, suspended, waiting, terminated
- **Priority Scheduling** - idle, low, normal, high, critical with priority-based round-robin scheduling
- **Resource Limits** - Memory, CPU, threads, open files per process with enforcement
- **Event Tracking** - 9 event types tracking all process state transitions
- **Preemptive Scheduling** - Configurable time-slice based scheduling
- **Process Events** - Created, started, suspended, resumed, blocked, unblocked, context_switched, resource_exceeded, terminated
- **Thread-Safe Operations** - Mutex protection for all process manager operations

### 🧠 Memory Management System

Complete virtual memory management with allocation tracking and memory protection:

- **Virtual Memory** - 4KB page-aligned memory allocation and tracking
- **Memory Pages** - MemoryPage structures with permissions (read/write/execute/privileged)
- **Memory Regions** - Kernel, heap, stack, code, data, shared, I/O regions with per-region permissions
- **Allocation Tracking** - Track all allocations per process with addresses and sizes
- **Memory Statistics** - Total/used/free memory, resident/swapped pages, page fault tracking
- **Page Faults** - Separate tracking for major and minor page faults
- **Memory Protection** - R/W/X permissions with privileged execution modes
- **Configurable Memory** - Default 4GB total memory, easily reconfigurable

### 📁 Log Rotation Support

- **File Rotation** - Automatically rotate log files when they exceed a configured maximum size.
- **Retention Policy** - Keep a fixed number of rotated log files and overwrite older ones in a round-robin fashion.
- **Zero-Downtime** - Logging continues seamlessly into a new file after rotation.

### 🔍 Observability, Debugging & Optimization

An integrated subsystem for deeper runtime insight and performance improvements:

- **Breakpoints & Logpoints** – Set breakpoints by file/line or function, optionally with conditions.
- **System Metrics** – Capture CPU, memory, thread, and I/O statistics over time.
- **Performance Profiling** – Track call counts and aggregate execution time per function.
- **Optimization Hints** – Store actionable recommendations based on observed metrics.
- **Runtime Debugger Hooks** – Enable or disable breakpoints programmatically for live debugging.

### 📊 Trace & Audit Management System

Comprehensive execution tracing and audit trail management for observability:

- **Trace Events** - 13 event types: function_call, function_return, syscall_enter/exit, context_switch, interrupt, exception, I/O operations, memory access, lock operations
- **Trace Manager** - Record and query execution traces with configurable trace levels (off, critical, normal, verbose, debug)
- **Audit Records** - 15+ audit action types: process create/terminate, memory operations, file operations, permission checks, security events, logins, configuration changes
- **Audit Results** - success, failure, denied, error status tracking
- **Audit Severity** - info, warning, critical, alert levels
- **Performance Metrics** - CPU usage, memory usage, I/O operations, context switches, page faults per process
- **Distributed Tracing** - Support for trace correlation and distributed trace collection
- **Ring Buffers** - Configurable limits (100K traces, 100K audit records default) with automatic overflow handling

### 🚀 Bootloader & Kernel Management System

Low-level system boot and kernel initialization management:

- **Boot Modes** - normal, safe, recovery, maintenance, diagnostic
- **Boot Phases** - firmware, bootloader, kernel_load, kernel_init, drivers, services, complete
- **Boot Devices** - disk, network, USB, CDROM, PXE boot device types
- **Kernel Modules** - Register, load, and initialize kernel modules with dependency tracking
- **Boot Configuration** - Boot device selection, mode, verbose output, debug mode, safe mode, timeouts
- **Boot Statistics** - Phase timing, modules loaded count, error tracking
- **Boot Events** - Comprehensive event logging through entire boot process
- **Boot Severity** - info, warning, error, critical levels for boot events
- **Configurable Kernel Version** - Track kernel version information

### Design Patterns Used
- **Module Pattern** - Each subsystem is a self-contained module
- **Manager Pattern** - Dedicated managers for each domain
- **Delegation Pattern** - System delegates to managers
- **Factory Pattern** - Consistent entity creation
- **Index Pattern** - Fast lookups via index system
- **Pub-Sub Pattern** - Event bus for decoupled communication
- **State Pattern** - Checkpoint and restore capabilities
- **Load Balancing Pattern** - Distribute work across nodes
- **Observer Pattern** - Event handlers and trace listeners
- **Ring Buffer Pattern** - Circular buffers for trace/audit history

## 🔮 Future Enhancements

Potential features for future KOGI OS versions:
- **Persistence Layer** - SQLite/PostgreSQL/RocksDB backend for state storage
- **REST API** - HTTP endpoints for integrating external applications
- **Web UI** - Dashboard for visualization and management
- **Multi-Session** - Support for multiple concurrent workers
- **Plugins** - Plugin system for extending subsystems
- **Backup/Restore** - System state backup and recovery
- **Notifications** - Event system for system state changes
- **Distributed** - Network-based multi-node support
- **Encryption** - End-to-end encryption for sensitive data
- **OAuth/OIDC** - OAuth2 and OpenID Connect provider support
- **API Keys** - Long-lived API key management and rotation
- **Two-Factor Auth** - Enhanced MFA options
- **Compliance** - GDPR, CCPA, compliance reporting

## 📝 License

[Specify your license here]

## 🤝 Contributing

Contributions welcome! Please follow the existing module architecture and include tests for new subsystems.

## 📧 Support

For issues, questions, or suggestions, please open an issue or contact the maintainers.

---

**KOGI OS** - Your Personal Operating System for Independent Work.

