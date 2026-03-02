# KOGI - Independent Worker Portfolio & Portfolio Management System

**KOGI** is a comprehensive, open-source portfolio management and work organization system designed for independent workers of all types: freelancers, contractors, consultants, gig workers, artists, musicians, software developers, gamers, and anyone who wants to organize their work, projects, and assets into a single cohesive, indexable, filterable, and searchable portfolio.

## 🎯 Core Vision

KOGI empowers independent workers to:
- **Organize** all their work, projects, products, services, and assets
- **Manage** clients, contacts, jobs, gigs, contracts, and tasks
- **Track** earnings, hours, deliverables, and performance
- **Search & Filter** everything across their entire portfolio
- **Control** their professional capital and intellectual property
- **Maintain** a unified system of record for all work-related activities

## ✨ Key Features

### 👥 Worker Management

- **Diverse Worker Types** - Support for contractors, consultants, freelancers, gig workers, project-based workers, artists, musicians, developers, gamers, and custom types
- **Worker Profiles** - Track worker names, email, hourly rates, skills, and activation status
- **Skill Tracking** - Maintain detailed skill lists for each worker
- **Worker Filtering** - Query and filter workers by type and skills
- **Status Management** - Activate/deactivate workers as needed

### 💼 Account Management

- **Multi-Account Support** - Each worker can manage multiple accounts across different platforms
- **Account Types**:
  - Social Media (Twitter, LinkedIn, Instagram, TikTok, etc.)
  - Work Accounts (Upwork, Fiverr, LinkedIn, freelance platforms)
  - Personal Accounts (email, messaging, personal platforms)
  - Email Accounts (Gmail, custom domains, work email)
  - Software Accounts (GitHub, development tools, SaaS platforms)
  - Custom Account Types
- **Account Details** - Username, provider, custom details, activation status
- **Account Filtering** - Find accounts by provider or type

### 📋 Portfolio System (Generic & Flexible)

**Universal Portfolio Structure** - Create portfolios for literally anything:
- Programs, Projects, Tasks
- Investments, Financial Assets
- Music, Artwork, Creative Works
- Software, Code, Applications
- Products, Solutions, Services
- Artifacts, Deliverables, Works-in-Progress
- Custom Portfolio Types

**Portfolio Components**:
- **Portfolios** - Top-level containers for organizing work
- **Collections** - Grouped items by type (e.g., "Music Portfolio", "Artwork Collection")
- **Portfolio Items** - Individual entries (songs, artwork pieces, code projects, services, etc.)
- **Metadata** - Rich classification system for every item

**Portfolio Features**:
- Entity Classification (type, class, category)
- Flexible Tagging System
- Custom Fields for domain-specific data
- Timestamps (created, updated)
- Ownership Tracking
- Hierarchical Organization

### 🔍 Search & Filtering

- **Index-Based Search** - Fast O(n) lookups across entire portfolio
- **Multi-Criteria Filtering**:
  - By Name Pattern (substring matching)
  - By Entity Type (portfolio, project, music, artwork, code, etc.)
  - By Entity Class (strategic, operational, creative, technical, etc.)
  - By Category
  - By Tags
  - By Owner
- **Collection Queries** - Retrieve all collections of a specific type
- **Entity Lookup** - Direct ID-based lookups

### 👔 CRM (Customer Relationship Management)

- **Client Management** - Track all clients and organizations
  - Client name, industry, notes
  - Unlimited contacts per client
- **Contact Tracking**:
  - Contact name, email, phone
  - Per-contact notes
  - Social and professional connections
- **Client Queries** - Look up clients and their contacts
- **Relationship History** - All interactions in one place

### 💰 Assets & Resources

- **Asset Types**:
  - Equipment (computers, hardware, instruments, tools)
  - Software (licenses, subscriptions, digital tools)
  - Intellectual Property (patents, trademarks, designs)
  - Furniture & Office Equipment
  - Vehicles
  - Custom Asset Types
- **Asset Tracking** - Name, description, value, acquisition date
- **Portfolio Valuation** - Calculate total asset value
- **Asset Management** - Add, track, and query assets

### 💼 Job & Contract Management

- **Job Posting** - Create work assignments with details
  - Title, description, budget, deadline
  - Skill requirements
  - Assignment tracking
- **Contract Management** - Link workers to jobs
  - Start/end dates
  - Hourly rates
  - Hours tracking
  - Contract status (active, completed, terminated)
- **Hours Logging** - Track billable and non-billable hours
- **Earnings Calculation** - Automatic worker earnings based on hours × rate
- **Contract Completion** - Mark contracts done and update job status

### 📊 Reporting & Analytics

- **Worker Statistics**:
  - Active worker count
  - Worker type distribution
  - Account counts per worker
- **Job Analytics**:
  - Total jobs posted
  - Active vs completed jobs
  - Job assignment tracking
- **Contract Reports**:
  - Contract count and status
  - Contract duration tracking
  - Termination tracking
- **Earnings Reports**:
  - Per-worker earnings calculation
  - Total earnings across all workers
  - Hours-based invoicing support
- **Portfolio Statistics**:
  - Collection counts
  - Item counts
  - Type distribution

### 🗂️ Data Organization & Management

**Hierarchical Structure**:
- Portfolios contain collections and items
- Collections organize items by type
- Items contain detailed metadata
- All searchable and filterable

**Rich Metadata**:
- Entity types (20+ types supported)
- Entity classes (12+ classes supported)
- Categories (custom per item)
- Tags (unlimited, multi-category)
- Custom fields (string key-value pairs)
- Timestamps
- Ownership information
- Parent-child relationships

### 🛡️ Data Management

- **Memory Safety** - Built in Zig with manual memory management
- **Complete Cleanup** - Proper deallocation of all allocated strings and collections
- **No Memory Leaks** - Recursive deallocation through all nested structures
- **Error Handling** - Proper error propagation and handling

### 🧪 Testing & Quality

- **Comprehensive Unit Tests** (40+ tests)
  - Worker management tests
  - Portfolio operations tests
  - CRM functionality tests
  - Asset management tests
  - Kernel integration tests
  - Search and filter tests
- **Integration Tests** - Full end-to-end workflows
- **Demo Mode** - Interactive demo showcasing all features

### 💻 Technical Architecture

**Modular Design**:
- `worker.zig` - Worker and account management
- `portfolio.zig` - Generic portfolio system
- `kernel.zig` - Core job/contract management engine
- `crm.zig` - Client relationship management
- `assets.zig` - Asset and resource tracking
- `accounts.zig` - Multi-account management
- `cli.zig` - Command-line interface with demo mode

**Language**: Zig (systems programming language)
**Memory Model**: GeneralPurposeAllocator with proper lifecycle management
**Data Structures**: ArrayLists, HashMaps for efficient storage and lookup

## 🚀 Use Cases

### Freelancers & Contractors
- Manage multiple clients and contracts
- Track hours and earnings
- Maintain portfolio of completed projects
- Organize invoicing and payment information

### Creative Professionals
- Portfolio of artwork, music, or creative works
- Client management and contact tracking
- Project and deliverable tracking
- Earnings and hours tracking

### Software Developers
- Code portfolio and project management
- Contract and gig tracking
- Skill maintenance and development
- Client and account management

### Independent Consultants
- Client and prospect management
- Project portfolio and history
- Contract terms and tracking
- Earnings and utilization reports

### Gig Workers
- Multiple platform account management
- Gig and task assignment tracking
- Earnings per platform
- Client ratings and feedback tracking

### Musicians & Artists
- Portfolio of works (music, artwork, performances)
- Client/venue management
- Project and commission tracking
- Asset and equipment inventory

### Multi-Disciplinary Workers
- Organize work across multiple domains
- Unified client management
- Cross-domain project tracking
- Consolidated earnings and reporting

## 📦 What's Included

```
kogi/
├── src/
│   ├── main.zig              # Application entry point
│   ├── root.zig              # Library exports
│   ├── worker.zig            # Worker management
│   ├── accounts.zig          # Account management
│   ├── portfolio.zig         # Portfolio system
│   ├── kernel.zig            # Job/contract engine
│   ├── crm.zig               # Client management
│   ├── assets.zig            # Asset tracking
│   ├── cli.zig               # CLI interface
│   └── tests.zig             # Unit tests
├── build.zig                 # Build configuration
├── build.zig.zon             # Dependency manifest
└── README.md                 # This file
```

## 🔧 Building & Running

### Prerequisites
- Zig 0.15.2 or later

### Build
```bash
zig build
```

### Run
```bash
zig build run
```

### Run Tests
```bash
zig build test
```

## 📊 Demo

The application includes a comprehensive demo showcasing all features:
- Creates 3 sample workers with different types and skills
- Posts 3 jobs with skill requirements
- Creates 3 contracts linking workers to jobs
- Logs hours on contracts
- Displays worker earnings reports
- Shows system statistics
- Lists all workers and active jobs

## 🎓 Architecture Highlights

### Separation of Concerns
- **Worker Module** - Handles worker lifecycle, skills, and accounts
- **Portfolio Module** - Generic, flexible portfolio organization
- **Kernel Module** - Job and contract management engine
- **CRM Module** - Client and contact relationship management
- **Assets Module** - Resource and asset tracking
- **CLI Module** - User interface and interactions

### Design Patterns
- **Module Pattern** - Each subsystem is self-contained
- **Manager Pattern** - Dedicated managers for each domain (WorkerManager, PortfolioManager, etc.)
- **Delegation Pattern** - Kernel delegates to specialized managers
- **Index Pattern** - Fast lookups via IndexEntry system
- **Factory Pattern** - Consistent entity creation

### Extensibility
- New entity types can be added to EntityType enum
- New entity classes via EntityClass enum
- Custom fields in Metadata for domain-specific data
- Flexible account and asset type systems
- Generic portfolio collections for any purpose

## 🔮 Future Enhancements

Potential features for future versions:
- Web UI for desktop and mobile access
- Database persistence (SQLite, PostgreSQL)
- Advanced analytics and reporting
- Invoice and payment management
- Time tracking and invoicing automation
- Social features and collaboration
- API for third-party integrations
- Backup and sync capabilities
- Tax preparation support
- Multi-user/team support

## 📝 License

[Specify your license here]

## 🤝 Contributing

Contributions welcome! Please follow the existing code patterns and include tests for new features.

## 📧 Support

For issues, questions, or suggestions, please open an issue or contact the maintainers.

---

**KOGI** - Organize Your Work, Manage Your Portfolio, Grow Your Career.
