# kogi-database

PostgreSQL schema for Kogi MVP.

## Apply schema
```sql
\i kogi-database/postgres/schema.sql
\i kogi-database/postgres/seed.sql
```

## Included domains
- auth and sessions
- IMS identities, personas, roles, worker types, and multi-profile configuration
- profile-scoped connection registry, contact directory, and asset vault
- kernel registry/events/scheduler
- portfolio and resources
- WBS/stories/sprints
- market orders and reviews
- exchange wallet, immutable ledger, escrow
- community spaces/messages
- work automation and OKRs
- crowdfunding/capital pools
- autonomous organizations and governance log
- AI agent session and action audit
