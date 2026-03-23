

**Hypergrid**

**N-Dimensional Distributed Spreadsheet System**

*API Specification — Version 1.0*

| Attribute | Value |
| :---- | :---- |
| Document | Hypergrid API Specification v1.0 |
| System | Hypergrid N-Dimensional Distributed Spreadsheet System (NDSS) |
| Platforms | Apapo · Kogi IW-OS · Ume B-OS · Qala SF-OS · Any platform on Hypergrid substrate |
| Version | 1.0 — March 2026 |
| Status | Authoritative Reference — Active Design |
| Audience | API consumers · Integration engineers · SDK authors · Platform developers · Domain system builders |
| Base URL | https://{grid\_host}/api/v1 |
| Protocols | REST (JSON over HTTPS/2) · gRPC (internal) · WebSocket (WSS) · Server-Sent Events (SSE) |
| Auth | Bearer JWT (RS256) · API Key (X-API-Key header) · mTLS (internal gRPC only) |
| Rate Limiting | Per-identity: 1,000 req/min default. Per-endpoint overrides documented per section. |
| Idempotency | POST/PUT/PATCH: support X-Idempotency-Key header (stored 24h in Redis). |
| Versioning | URL path versioning: /api/v1, /api/v2. Breaking changes always bump the major version. |
| Content-Type | application/json for REST. application/x-ndjson for streaming responses. |
| Classification | Confidential — Internal Use Only |

# **Table of Contents**

# **Part I — Foundations**

## **1\. Overview**

The Hypergrid API exposes the complete functionality of the Hypergrid N-Dimensional Distributed Spreadsheet System over HTTPS/2 REST, WebSocket, and gRPC interfaces. The API is the canonical interface for all platform interactions: creating and querying N-dimensional Hypercubes, executing HyperQL queries, managing the Hypergraph, provisioning Spaces and Workspaces, managing identities and namespaces, submitting AI computation requests, and operating federation between Grid deployments.

The API is organized into eleven resource groups: Grids, Hypercubes, HyperCells, HyperQL, Hypergraph, Spaces, Workspaces, Namespaces, Identity, AI, and Federation. Each group has a consistent sub-resource structure and follows the same authentication, error, and pagination conventions.

## **2\. Authentication**

### **2.1 Authentication Methods**

| Method | Header / Mechanism | Scope | Notes |
| :---- | :---- | :---- | :---- |
| Bearer JWT | Authorization: Bearer {jwt} | All REST and WebSocket endpoints | RS256 signed JWT. Issued by Hypergrid identity service. Contains: sovereign\_entity\_id, identity\_tag, space\_id, scopes, exp. Max lifetime: 24h. Refresh via POST /auth/token/refresh. |
| API Key | X-API-Key: {key} | All REST endpoints (not WebSocket) | Long-lived. Created via POST /identity/api-keys. Scoped to specific cubes, spaces, or operations. For server-to-server automation. |
| mTLS | Client certificate in TLS handshake | Internal gRPC only (service mesh) | For inter-service communication only. Not exposed externally. Certificate issued by platform PKI. |
| Session Token | Cookie: hg\_session={token} | Browser clients (claude.ai-style apps) | Short-lived (4h). Automatically refreshed by client SDK. CSRF-protected via SameSite=Strict \+ CSRF header. |

### **2.2 JWT Token Structure**

| // JWT payload structure (RS256) {   "sub":              "sovereign\_entity\_id:uuid",   "iss":              "https://grid.hypergrid.io/auth",   "aud":              "hypergrid-api",   "exp":              1780000000,           // Unix timestamp   "iat":              1779913600,   "jti":              "unique-token-id",    // for revocation   "hg": {     "sovereign\_entity\_id": "uuid",     "identity\_tag":        "@alice-dev",   // active partition     "space\_id":            "uuid",         // active Space (optional)     "workspace\_id":        "uuid",         // active Workspace (optional)     "permission\_tier":     4,              // Editor     "scopes": \[       "grid:read",       "cubes:read",       "cubes:write",       "graph:read",       "spaces:read",       "namespaces:read"     \],     "cube\_restrictions":   \[\],             // empty \= all cubes     "grid\_id":             "uuid"          // which grid this token is for   } } |
| :---- |

### **2.3 Auth Error Codes**

| HTTP Status | Code | Meaning |
| :---- | :---- | :---- |
| 401 | unauthorized | Missing or malformed token. No Authorization header present or token cannot be parsed. |
| 401 | token\_expired | JWT exp has passed. Refresh using POST /auth/token/refresh. |
| 401 | token\_revoked | Token jti has been revoked (logout or key rotation). |
| 403 | insufficient\_tier | Authenticated but PermissionTier is below the minimum required for this operation. |
| 403 | scope\_not\_granted | The required scope (e.g. cubes:write) is not present in the token's scopes array. |
| 403 | cube\_restricted | Token has cube\_restrictions set and the requested cube is not in the allowed list. |
| 403 | isolation\_violation | Cross-partition access attempted on a HardIsolated partition without cross-partition scope. |
| 403 | policy\_denied | A PolicyEngine evaluation blocked this operation. Error body contains policy\_id and reason. |

## **3\. Base URL and Versioning**

| // Production base URLs https://{grid\_host}/api/v1                         // REST API v1 wss://{grid\_host}/ws/v1                            // WebSocket v1 grpc://{grid\_host}:9000                            // gRPC (internal only) https://{grid\_host}/api/v1/events                  // Server-Sent Events stream   // Grid host examples kogi.hypergrid.io          // Kogi production Grid ume.hypergrid.io           // Ume production Grid qala.hypergrid.io          // Qala production Grid localhost:8080             // Local development grid.company.internal      // Self-hosted deployment   // API version path /api/v1/{resource}         // Current stable version /api/v2/{resource}         // Next major version (when released)   // Per-grid scoping (when one host serves multiple grids) /api/v1/grids/{grid\_id}/cubes/{id}    // Fully qualified /api/v1/cubes/{id}                    // Implicit: uses grid from JWT |
| :---- |

## **4\. Common Request / Response Conventions**

### **4.1 Request Headers**

| Header | Required | Description |
| :---- | :---- | :---- |
| Authorization | Yes (JWT) | Bearer {jwt}. Required on all authenticated endpoints. |
| X-API-Key | Yes (API key) | Mutually exclusive with Authorization. For server-to-server. |
| Content-Type | Yes (write ops) | application/json for all request bodies. |
| Accept | No | application/json (default). application/x-ndjson for streaming. |
| X-Idempotency-Key | No | UUID v4. For POST/PUT/PATCH operations. Prevents duplicate processing. Stored 24h. |
| X-Request-ID | No | Client-provided request identifier for distributed tracing. Echoed back in response. |
| X-Grid-ID | No | Override the grid targeted by this request (must match JWT grid\_id or federation peer). |
| X-Identity-Tag | No | Override the active identity partition for this request (must be owned by caller). |
| X-Space-ID | No | Set the active Space context for this request. |
| X-Workspace-ID | No | Set the active Workspace context for this request. |
| If-Match | No | Optimistic concurrency: provide current VectorClock version. Returns 409 if stale. |
| Prefer | No | return=minimal (no body on success) | return=representation (full entity). Default: representation. |

### **4.2 Response Headers**

| Header | Description |
| :---- | :---- |
| X-Request-ID | Echo of client X-Request-ID or server-generated UUID. Use for support tickets. |
| X-RateLimit-Limit | Maximum requests per minute for this endpoint. |
| X-RateLimit-Remaining | Remaining requests in the current rate limit window. |
| X-RateLimit-Reset | Unix timestamp when the rate limit window resets. |
| X-Vector-Clock | VectorClock state of the mutated entity after this write operation (JSON-serialized). |
| X-Entity-Version | Monotonic version counter of the entity after this write. |
| X-Conflict-ID | Present when a CRDT conflict was recorded during this write (not an error — write still succeeded). |
| X-Deprecation-Notice | Present when this endpoint or parameter is deprecated. Value \= replacement endpoint path. |
| X-Grid-ID | The Grid that handled this request. |
| Link | Pagination links: rel=next, rel=prev, rel=first, rel=last with full URL. |
| ETag | Entity tag for the current representation. Use with If-Match for optimistic concurrency. |

### **4.3 Pagination**

All list endpoints use cursor-based pagination for consistency across large datasets. Offset-based pagination is also supported for smaller datasets but is deprecated for large cubes.

| // Query parameters for all list endpoints ?limit=50                    // Items per page. Default: 50\. Max: 500\. ?cursor=eyJpZCI6InVpZCJ9    // Opaque cursor from previous response Link header ?offset=100                  // Offset-based (deprecated for cubes \> 10K rows) ?order\_by=updated\_at         // Attribute key to sort by ?order\_dir=desc              // asc | desc. Default: desc for updated\_at, asc otherwise ?include\_archived=false      // Include soft-archived rows. Default: false. ?include\_shadow=false        // Include ShadowRows from linked Grids. Default: false.   // Paginated response envelope {   "data": \[...\],             // Array of results   "meta": {     "total":       1842,     // Total matching rows (estimated for large cubes)     "count":       50,       // Items in this page     "cursor":      "eyJpZCI6InVpZCJ9",  // cursor for next page (null on last page)     "next\_cursor": "eyJpZCI6InVpZDIifQ", // explicit next cursor     "has\_more":    true,     // false on last page     "limit":       50,     "offset":      0   },   "links": {     "self":  "https://grid.host/api/v1/cubes/id/rows?limit=50",     "next":  "https://grid.host/api/v1/cubes/id/rows?limit=50\&cursor=eyJ...",     "prev":  null,           // null on first page     "first": "https://grid.host/api/v1/cubes/id/rows?limit=50",     "last":  "https://grid.host/api/v1/cubes/id/rows?limit=50\&cursor=last"   } } |
| :---- |

### **4.4 Error Response Schema**

| // All errors follow this structure {   "error": {     "code":       "not\_found",              // Machine-readable error code (snake\_case)     "message":    "Hypercube not found",    // Human-readable message     "detail":     "Cube ID abc-123 does not exist in grid xyz-456",  // Additional context     "field":      "cube\_id",                // Field that caused the error (validation errors)     "request\_id": "req-abc-123",            // X-Request-ID echo     "docs\_url":   "https://docs.hypergrid.io/errors/not\_found",     "meta": {                               // Extra context (present on some errors)       "required\_tier": 4,                  // For insufficient\_tier errors       "current\_tier":  2,       "policy\_id":     "pol-uuid",          // For policy\_denied errors       "conflict\_id":   "conf-uuid",         // For version\_conflict errors       "retry\_after":   30                   // For rate\_limit\_exceeded errors (seconds)     }   } } |
| :---- |

### **4.5 Complete Error Code Registry**

| HTTP | Code | Description |
| :---- | :---- | :---- |
| 400 | bad\_request | Malformed request syntax, invalid JSON, or structurally invalid body. |
| 400 | validation\_error | Request body or query params fail schema validation. See error.field. |
| 400 | invalid\_dim\_coordinate | The N-dim coordinate in the request doesn't match the Hypercube's declared dimensionality. |
| 400 | invalid\_hyperql | HyperQL query syntax error. See error.detail for parser error position. |
| 400 | invalid\_crdt\_operation | CrdtOperation payload is malformed or references an unknown attribute key. |
| 400 | cyclic\_dependency | Adding this Hierarchy or Dependency edge would create a cycle in the graph. |
| 400 | schema\_conflict | Proposed schema change conflicts with existing schema (e.g. duplicate axis name). |
| 400 | formula\_invalid | Formula expression in AttributeKeyDef has a syntax or semantic error. |
| 400 | idempotency\_conflict | An X-Idempotency-Key was reused with a different request body. |
| 401 | unauthorized | Missing, malformed, or unparseable auth token. |
| 401 | token\_expired | JWT has expired. Refresh with POST /auth/token/refresh. |
| 401 | token\_revoked | Token has been revoked via logout or key rotation. |
| 403 | insufficient\_tier | Caller's PermissionTier is below the minimum required for this operation. |
| 403 | scope\_not\_granted | Required scope is absent from the token's scopes array. |
| 403 | cube\_restricted | Token has cube\_restrictions and this cube is not in the allowed list. |
| 403 | isolation\_violation | Cross-partition access to HardIsolated partition attempted without authorization. |
| 403 | policy\_denied | PolicyEngine evaluation blocked this operation. See meta.policy\_id. |
| 403 | governance\_required | This mutation requires a GovernanceProposal vote. See meta for proposal type. |
| 403 | consent\_required | This edge type requires target consent before proceeding. |
| 403 | shadow\_read\_only | Write attempted on a ShadowRow attribute not in write\_back\_attrs. |
| 404 | not\_found | Requested resource does not exist or is not visible to the caller. |
| 404 | cube\_not\_found | Hypercube ID does not exist in this Grid. |
| 404 | row\_not\_found | D₁ key not found in this Hypercube. |
| 404 | cell\_not\_found | N-dim coordinate not found in the Hypercube (sparse cube, coordinate is empty). |
| 404 | edge\_not\_found | HypergraphEdge ID not found. |
| 404 | space\_not\_found | Space ID not found or caller is not a member. |
| 404 | namespace\_not\_found | NamespacePath does not resolve to any known entity. |
| 404 | snapshot\_not\_found | Snapshot ID not found or has expired from the snapshot store. |
| 409 | version\_conflict | If-Match ETag does not match current entity version. See meta.conflict\_id. |
| 409 | already\_exists | Entity with this ID or namespace path already exists. |
| 409 | invalid\_state\_transition | Attempted lifecycle state transition is not valid in the configured Lattice. |
| 409 | merge\_conflict | CRDT merge produced a conflict. Write was recorded but conflict needs review. |
| 422 | unsupported\_encoding | Requested StorageEncoding is incompatible with the declared axis types. |
| 422 | dim\_limit\_exceeded | Attempt to add more than 16 DimensionAxes to a Hypercube. |
| 422 | formula\_resource\_limit | Formula evaluation exceeded allowed cell refs, rows, depth, time, or memory. |
| 429 | rate\_limit\_exceeded | Too many requests. See X-RateLimit-\* headers. Retry after meta.retry\_after seconds. |
| 500 | internal\_error | Unexpected server error. Idempotent requests may be safely retried. |
| 502 | upstream\_error | Upstream dependency (PostgreSQL, Redis, Kafka) returned an error. |
| 503 | service\_unavailable | Service temporarily unavailable (startup, maintenance). Retry-After header present. |
| 504 | gateway\_timeout | Request timed out. For long-running HyperQL queries, use async query endpoint. |

# **Part II — Grid API**

## **5\. Grids**

A Grid is the root container for a complete Hypergrid deployment. It owns all Hypercubes, the Hypergraph, EventLog, CrdtLog, NamespaceRegistry, Spaces, and federation peer relationships. Each domain system (Kogi, Ume, Qala) is a separate Grid deployment.

### **5.1 Grid Object Schema**

| {   "grid\_id":          "uuid",   "name":             "kogi-production",   "version":          "1.0.0",            // Hypergrid substrate version   "namespace\_path":   "hypergrid://kogi-production/",   "domain\_system":    "kogi",             // kogi | ume | qala | standalone   "node\_id":          "node-us-east-1",   "status":           "active",           // provisioning | active | maintenance | deprecated   "cube\_count":       42,   "federation\_peers": \["uuid-ume", "uuid-qala"\],   "space\_count":      1204,   "row\_estimate":     84200000,           // approximate total HyperRows across all cubes   "created\_at":       "2026-01-15T09:00:00Z",   "updated\_at":       "2026-03-22T14:30:00Z",   "meta": {     "description":    "Kogi Independent Worker OS Grid",     "region":         "us-east-1",     "contact\_email":  "ops@kogi.io"   } } |
| :---- |

| GET | /api/v1/grids |
| :---: | :---- |

*List all Grids accessible to the authenticated caller, including federated peer Grids.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| limit | query | integer | No | Max results. Default: 20\. Max: 100\. |
| cursor | query | string | No | Pagination cursor from previous response. |
| include\_peers | query | boolean | No | Include federated peer Grids. Default: false. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: Grid\[\], meta: PaginationMeta } |
| 401 | Unauthorized | { error: { code: 'unauthorized', ... } } |

| GET | /api/v1/grids/{grid\_id} |
| :---: | :---- |

*Get full details of a specific Grid by ID.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| grid\_id | path | uuid | Yes | Grid identifier. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | Grid object |
| 404 | Not found | { error: { code: 'not\_found', ... } } |

| POST | /api/v1/grids |
| :---: | :---- |

*Provision a new Grid deployment. Admin-tier operation.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| X-Idempotency-Key | header | string | No | Idempotency key for safe retries. |

**Request Body:**

| {   "name":           "my-grid",         // required: URL-safe grid name   "domain\_system":  "kogi",            // kogi | ume | qala | standalone   "description":    "My grid",   "region":         "us-east-1",       // deployment region   "encoding\_default": "Hybrid",        // default StorageEncoding for new cubes   "plugin\_ids":     \["time-axis", "geo-axis", "hierarchy-axis"\],   "meta":           {} } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | Grid provisioned | Grid object |
| 400 | Validation error | { error: { code: 'validation\_error', field: '...', ... } } |
| 403 | Insufficient tier | { error: { code: 'insufficient\_tier', ... } } |
| 409 | Already exists | { error: { code: 'already\_exists', ... } } |
| Note: Grid provisioning is an async operation. The Grid transitions from status=provisioning to status=active once bootstrap completes. Poll GET /grids/{id} until status=active. |  |  |

| GET | /api/v1/grids/{grid\_id}/health |
| :---: | :---- |

*Get health and telemetry summary for a Grid: cell store connectivity, CRDT lag, federation peer status, AI engine status.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| grid\_id | path | uuid | Yes | Grid identifier. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { status, cell\_store, crdt\_lag\_ms, peers: \[{peer\_id, status, lag\_ms}\], ai\_engines: \[{id, status}\] } |
| 503 | Grid degraded | Health object with non-healthy status fields |

# **Part III — Hypercube API**

## **6\. Hypercubes**

A Hypercube is an N-dimensional grid: H \= (D₁, D₂, ..., Dₙ). It is the generalization of a spreadsheet sheet. Hypercubes own a set of DimensionAxes and an AttributeKeyRegistry that defines the schema of every HyperCell within the cube. The actual cell data lives in the Grid's CellStoreBackend.

### **6.1 Hypercube Object Schema**

| {   "cube\_id":          "uuid",   "grid\_id":          "uuid",   "name":             "kogi.portfolio.components",   "namespace\_path":   "hypergrid://kogi/personal/alice/cubes/components/",   "space\_id":         "uuid",                   // owning Space (nullable)   "n":                2,                         // dimensionality (2..=16)   "dims": \[     {       "axis\_id":       "uuid",       "dim\_index":     1,       "name":          "entity",       "axis\_type":     "EntityAxis",             // EntityAxis|PropertyAxis|TimeAxis|...       "key\_type":      "Uuid",       "key\_cardinality":"Dense",       "ordering":      "Unordered",       "nullable":      false,       "index\_strategy":"Hash",       "plugin\_id":     null     },     {       "axis\_id":       "uuid",       "dim\_index":     2,       "name":          "property",       "axis\_type":     "PropertyAxis",       "key\_type":      "String",       "key\_cardinality":"Sparse",       "ordering":      "Lexicographic",       "nullable":      false,       "index\_strategy":"GIN",       "plugin\_id":     null     }   \],   "encoding":         "Hybrid",                  // Dense|Sparse|Hybrid   "attr\_count":       48,                        // registered attribute keys   "row\_count":        2840000,                   // estimated HyperRows (D₁ cardinality)   "schema\_version":   3,   "status":           "active",                  // active|archived|migrating   "visibility":       "Tenant",   "plugin\_ids":       \["time-axis"\],   "governance": {     "require\_vote\_for": \["remove\_attr", "change\_visibility"\]   },   "created\_at":       "2026-01-15T09:00:00Z",   "updated\_at":       "2026-03-22T14:30:00Z" } |
| :---- |

| GET | /api/v1/grids/{grid\_id}/cubes |
| :---: | :---- |

*List all Hypercubes in a Grid accessible to the caller.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| grid\_id | path | uuid | Yes | Grid identifier. |
| space\_id | query | uuid | No | Filter to cubes owned by a specific Space. |
| name\_prefix | query | string | No | Filter by cube name prefix (e.g. 'kogi.portfolio'). |
| limit | query | integer | No | Max results. Default: 50\. Max: 200\. |
| cursor | query | string | No | Pagination cursor. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: Hypercube\[\], meta: PaginationMeta } |
| 403 | Forbidden | Caller has no access to this Grid. |
| Note: This endpoint returns cube schemas, not cell data. To query cells, use the HyperRows or HyperQL endpoints. |  |  |

| POST | /api/v1/grids/{grid\_id}/cubes |
| :---: | :---- |

*Create a new Hypercube in a Grid. Registers dimension axes and initial attribute keys.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| grid\_id | path | uuid | Yes | Grid identifier. |
| X-Idempotency-Key | header | string | No | Idempotency key. |

**Request Body:**

| {   "name":       "acme.crm.contacts",       // required: unique within grid, dot-namespaced   "space\_id":   "uuid",                    // optional: owning Space   "dims": \[                                // required: at least 2 dimensions     {       "dim\_index":     1,       "name":          "contact",       "axis\_type":     "EntityAxis",       // EntityAxis|PropertyAxis|TimeAxis|GeoAxis|                                            //   CategoryAxis|HierarchyAxis|TenantAxis|                                            //   ScenarioAxis|VersionAxis|OrdinalAxis|Custom       "key\_type":      "Uuid",       "key\_cardinality":"Dense",       "nullable":      false,       "index\_strategy":"Hash"     },     {       "dim\_index":     2,       "name":          "property",       "axis\_type":     "PropertyAxis",       "key\_type":      "String",       "key\_cardinality":"Sparse",       "nullable":      false,       "index\_strategy":"GIN"     }   \],   "encoding":    "Hybrid",                 // Dense|Sparse|Hybrid   "attr\_keys": \[                           // optional: initial attribute keys     {       "key":           "name",       "display\_name":  "Contact Name",       "attr\_type":     "Text",       "write\_permission": 3,               // Contributor+       "crdt\_semantics": "LastWriteWins",       "visibility":    "Tenant"     }   \],   "plugin\_ids":  \[\],   "visibility":  "Tenant",   "governance":  { "require\_vote\_for": \["remove\_attr"\] } } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | Cube created | Hypercube object |
| 400 | Validation error | Invalid dim config, duplicate axis index, unsupported axis\_type. |
| 403 | Insufficient tier | Requires Editor+ tier. |
| 409 | Already exists | Cube with this name already exists in this Grid. |
| 422 | Unsupported encoding | Hybrid encoding not compatible with declared axis types. |
| Note: Creating a Hypercube is a zero-downtime schema operation. The cube is immediately queryable once created. Add attribute keys via POST /cubes/{id}/attrs without any downtime. |  |  |

| GET | /api/v1/cubes/{cube\_id} |
| :---: | :---- |

*Get complete schema for a Hypercube: all dimension axes and the full AttributeKeyRegistry.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| include\_attrs | query | boolean | No | Include full AttributeKeyRegistry in response. Default: true. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | Hypercube object with dims\[\] and attr\_registry\[\] |
| 404 | Not found | Cube does not exist or is not visible to caller. |

| PATCH | /api/v1/cubes/{cube\_id} |
| :---: | :---- |

*Update Hypercube metadata: name, description, visibility, governance config. Does not modify dims or attr\_keys.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |

**Request Body:**

| {   "name":        "acme.crm.contacts.v2",   // optional: rename (creates ns alias)   "description": "CRM contact records",    // optional   "visibility":  "Public",                 // optional: Public|Tenant|Identity|Private   "governance":  { "require\_vote\_for": \["remove\_attr","change\_visibility"\] } } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Updated | Updated Hypercube object |
| 403 | Insufficient tier | Requires Manager+ tier. |
| 409 | Governance required | Visibility change requires GovernanceProposal vote. |

| DELETE | /api/v1/cubes/{cube\_id} |
| :---: | :---- |

*Archive a Hypercube. Sets status=archived. Does not delete data — all cells remain accessible via AS\_OF queries and the EventLog.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 204 No Content | Archived | No body. |
| 403 | Insufficient tier | Requires Owner tier. |
| 409 | Governance required | Archiving this cube requires a GovernanceProposal vote. |
| Note: Archiving is irreversible without an Owner restoring the cube via PATCH status=active. All data is preserved. |  |  |

### **6.2 Dimension Axis API**

| GET | /api/v1/cubes/{cube\_id}/dims |
| :---: | :---- |

*List all DimensionAxes for a Hypercube.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: DimensionAxis\[\] } |

| POST | /api/v1/cubes/{cube\_id}/dims |
| :---: | :---- |

*Add a new DimensionAxis to an existing Hypercube (N+1). Zero-downtime schema evolution. Existing cells gain the new dimension with null/default key.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| X-Idempotency-Key | header | string | No | Idempotency key. |

**Request Body:**

| {   "dim\_index":      3,                    // must be current\_n \+ 1   "name":           "time\_period",   "axis\_type":      "TimeAxis",   "key\_type":       "Timestamp",   "key\_cardinality":"Sparse",   "nullable":       true,   "ordering":       "Chronological",   "index\_strategy": "BRIN",              // BRIN recommended for TimeAxis   "plugin\_id":      "time-axis"          // built-in time-axis plugin } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | Axis added | DimensionAxis object |
| 400 | Invalid dim\_index | dim\_index must equal current n+1. |
| 403 | Insufficient tier | Requires Manager+ tier. |
| 409 | Governance required | Adding a dimension may require a GovernanceProposal vote. |
| 422 | Dim limit exceeded | Hypercube already has 16 dimensions (maximum). |
| Note: Adding a dimension is a SchemaEvent written to the SchemaEventLog. All federation peers receive and apply the AddDimension SchemaEvent automatically. |  |  |

| GET | /api/v1/cubes/{cube\_id}/dims/{axis\_id} |
| :---: | :---- |

*Get a specific DimensionAxis by ID.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| axis\_id | path | uuid | Yes | DimensionAxis identifier. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | DimensionAxis object |
| 404 | Not found | Axis does not exist on this cube. |

| PATCH | /api/v1/cubes/{cube\_id}/dims/{axis\_id} |
| :---: | :---- |

*Update a DimensionAxis: rename, change index strategy, update ordering. Does not change axis\_type or key\_type (breaking — requires migration).*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| axis\_id | path | uuid | Yes | DimensionAxis identifier. |

**Request Body:**

| {   "name":           "fiscal\_quarter",    // rename (creates alias)   "index\_strategy": "BRIN",             // update index strategy   "description":    "Fiscal quarter key" } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Updated | Updated DimensionAxis |
| 400 | Validation error | Cannot change axis\_type or key\_type via PATCH. |
| 403 | Insufficient tier | Requires Manager+ tier. |

### **6.3 Attribute Key Registry API**

| GET | /api/v1/cubes/{cube\_id}/attrs |
| :---: | :---- |

*List all registered AttributeKeyDefs for a Hypercube.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| attr\_type | query | string | No | Filter by AttributeType (e.g. AI, Computed, Text). |
| dimension\_scope | query | string | No | Filter by dimension scope (e.g. axis\_id). |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: AttributeKeyDef\[\] } |

| POST | /api/v1/cubes/{cube\_id}/attrs |
| :---: | :---- |

*Register a new attribute key in the Hypercube's AttributeKeyRegistry. Zero-downtime. Existing cells return default\_value for the new key.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| X-Idempotency-Key | header | string | No | Idempotency key. |

**Request Body:**

| {   "key":             "lead\_score",          // required: unique within cube   "display\_name":    "Lead Score",   "attr\_type":       "AI",                  // Text|Number|Currency|Percent|Bool|Date|                                             //   DateTime|Duration|Enum|MultiEnum|                                             //   Relation|MultiRelation|User|Tag|                                             //   Json|Computed|AI|Formula|Audit|Custom   "description":     "AI-computed lead quality score (0–100)",   "dimension\_scope": "AllDimensions",       // AllDimensions | { specific\_axes: \[axis\_id\] }   "default\_value":   null,   "required":        false,   "visibility":      "Tenant",             // Public|Tenant|Identity|Owner   "write\_permission":7,                    // System (AI-write only)   "crdt\_semantics":  "LastWriteWins",      // LWW|OR-Set|GrowOnlyCounter|PNCounter|                                            //   MaxRegister|MinRegister|Lattice|                                            //   AppendOnlyLog|JsonCrdt|Custom   "computation": {                         // for Computed and AI attr\_types     "type":          "AiEngine",     "engine\_id":     "crm.lead\_score\_engine.v1",     "model\_name":    "lead\_score\_v2",     "input\_attrs":   \["stage","last\_contact\_at","value","source","interaction\_count"\],     "staleness\_ttl": 7200                  // seconds before AI signal considered stale   },   "aggregation":     \["AVG","MIN","MAX"\],   "index\_strategy":  "BTree",   "searchable":      false,   "version\_track":   true,   "export\_label":    "Lead Quality Score" } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | Attr key registered | AttributeKeyDef object |
| 400 | Validation error | Invalid crdt\_semantics for attr\_type, duplicate key name, etc. |
| 403 | Insufficient tier | Requires Editor+ tier. |
| 409 | Already exists | Attribute key with this name already registered. |
| Note: Registering an attribute key is a SchemaEvent. All federation peers receive and apply it automatically. Existing cells get default\_value for this key on next read. |  |  |

| GET | /api/v1/cubes/{cube\_id}/attrs/{attr\_key} |
| :---: | :---- |

*Get a specific AttributeKeyDef by key name.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| attr\_key | path | string | Yes | Attribute key name (e.g. 'health\_score'). |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | AttributeKeyDef object |
| 404 | Not found | Attr key not registered on this cube. |

| PATCH | /api/v1/cubes/{cube\_id}/attrs/{attr\_key} |
| :---: | :---- |

*Update an AttributeKeyDef: rename (with alias), change display\_name, update default\_value, change write\_permission, update aggregation list. Cannot change crdt\_semantics of an existing key without schema version bump.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| attr\_key | path | string | Yes | Attribute key name. |

**Request Body:**

| {   "display\_name":   "Lead Quality Score",   "description":    "Updated description",   "default\_value":  0.0,   "visibility":     "Tenant",   "write\_permission":7,   "export\_label":   "Score" } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Updated | Updated AttributeKeyDef |
| 400 | Validation error | crdt\_semantics change not allowed via PATCH. |
| 403 | Insufficient tier | Requires Manager+ tier. |

| DELETE | /api/v1/cubes/{cube\_id}/attrs/{attr\_key} |
| :---: | :---- |

*Tombstone (soft-delete) an attribute key. The key is marked as tombstoned and removed from schema validation. Existing cell values are preserved in the EventLog for AS\_OF queries but not returned in live queries.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| attr\_key | path | string | Yes | Attribute key name. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 204 No Content | Tombstoned | No body. |
| 403 | Insufficient tier | Requires Owner tier. |
| 409 | Governance required | Removing an attribute key requires GovernanceProposal vote if cube.governance.require\_vote\_for includes 'remove\_attr'. |
| Note: Tombstoning is irreversible. Historical values for this key remain accessible via AS\_OF queries. Active cells will no longer return this key. |  |  |

# **Part IV — HyperRow and HyperCell API**

## **7\. HyperRows**

A HyperRow is all cells sharing the same D₁ key — the canonical entity in a Hypercube. It is the generalization of a spreadsheet row. The HyperRow API provides entity-level CRUD with CRDT-safe writes, pagination, filtering, and time-travel.

### **7.1 HyperRow Object Schema**

| {   "d1\_key":     "uuid",                    // D₁ key — entity identifier   "cube\_id":    "uuid",   "grid\_id":    "uuid",   "attributes": {                          // attribute map: key → TypedAttrValue     "name":          { "value": "Acme Corp", "type": "Text",                        "version": 14, "last\_actor": "user-1:@alice:node-1",                        "updated\_at": "2026-03-22T10:00:00Z" },     "status":        { "value": "Active", "type": "Enum",                        "version": 3, "last\_actor": "user-1:@alice:node-1",                        "updated\_at": "2026-03-20T08:00:00Z" },     "tags":          { "value": \["enterprise","tier-1"\], "type": "TagSet",                        "version": 7, "crdt": "OR-Set" },     "lead\_score":    { "value": 87.4, "type": "AI",                        "computed\_by": "crm.lead\_score\_engine.v1",                        "computed\_at": "2026-03-22T09:50:00Z",                        "confidence": 0.92 },     "budget":        { "value": { "amount": "50000.00", "currency": "USD" },                        "type": "Currency" }   },   "vector\_clock": { "node-us-east-1": 42, "node-eu-west-1": 37 },   "version":      55,   "visibility":   "Tenant",   "created\_at":   "2026-01-20T09:00:00Z",   "updated\_at":   "2026-03-22T10:00:00Z",   "is\_shadow":    false,   "edge\_refs":    \["edge-uuid-1", "edge-uuid-2"\] } |
| :---- |

| GET | /api/v1/cubes/{cube\_id}/rows |
| :---: | :---- |

*List HyperRows in a Hypercube with optional DimSlice filtering, sorting, grouping. Returns paginated results.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| filter | query | string | No | DimSlice predicate in HyperQL WHERE syntax. URL-encoded. Example: D2='status' AND value='Active' |
| sort | query | string | No | Sort: {attr\_key}:{asc|desc}. Example: updated\_at:desc |
| group\_by | query | string | No | Group by attr\_key. Returns grouped result structure. |
| attrs | query | string | No | Comma-separated list of attr\_keys to include. Default: all. Example: name,status,health\_score |
| include\_shadow | query | boolean | No | Include ShadowRows from linked Grids. Default: false. |
| identity\_tag | query | string | No | Scope to a specific identity partition. Example: @alice-dev |
| space\_id | query | uuid | No | Scope to rows tagged to a specific Space. |
| as\_of | query | datetime | No | Time-travel: return row state at this UTC timestamp. ISO 8601\. |
| limit | query | integer | No | Page size. Default: 50\. Max: 500\. |
| cursor | query | string | No | Pagination cursor. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: HyperRow\[\], meta: PaginationMeta } |
| 400 | Invalid filter | Malformed DimSlice predicate syntax. See error.detail. |
| 404 | Cube not found |  |
| Note: For complex queries spanning multiple cubes, use the HyperQL endpoint POST /cubes/{id}/query instead. |  |  |

| POST | /api/v1/cubes/{cube\_id}/rows |
| :---: | :---- |

*Create a new HyperRow. Accepts a D₁ key and initial attribute values. Produces SetAttr CrdtOperations for all provided attributes.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| X-Idempotency-Key | header | string | No | Idempotency key. Recommended to prevent duplicate creation. |

**Request Body:**

| {   "d1\_key":    "uuid",                    // optional: server-generated if omitted   "attributes": {     "name":    "Acme Corp",     "status":  "Active",     "tags":    \["enterprise","tier-1"\],   // OR-Set: each tag added as separate OR-Set Add     "budget":  { "amount": "50000.00", "currency": "USD" },     "owner":   "user-uuid"   },   "visibility": "Tenant",                // row-level visibility   "space\_id":   "uuid",                  // optional Space association   "identity\_tags": \["@alice-dev"\]        // optional: partition tags } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | Row created | HyperRow object with server-assigned d1\_key |
| 400 | Validation error | Required attributes missing, type mismatch. |
| 403 | Insufficient tier | Requires Contributor+ tier. |
| 409 | Already exists | A row with this d1\_key already exists. |
| Note: All attribute writes produce CrdtOperations and are recorded in the EventLog. Set-valued attributes (TagSet, MultiEnum, MultiRelation) are initialized with AddToSet OR-Set operations, one per element. |  |  |

| GET | /api/v1/cubes/{cube\_id}/rows/{d1\_key} |
| :---: | :---- |

*Get all cells for a single HyperRow by D₁ key. Returns the complete attribute map.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| d1\_key | path | uuid | Yes | D₁ key (entity identifier). |
| attrs | query | string | No | Comma-separated attr keys to include. |
| as\_of | query | datetime | No | Time-travel: return state at this timestamp. |
| include\_computed | query | boolean | No | Include Tier-1 formula-computed attrs. Default: true. |
| include\_ai | query | boolean | No | Include Tier-2 AI-computed attrs. Default: true. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | HyperRow object |
| 404 | Not found | D₁ key not found in this cube. |
| Note: For as\_of queries, the server performs EventLog replay from the nearest snapshot. For hot cubes, this may take up to 500ms. Use async queries for historical analysis workloads. |  |  |

| PATCH | /api/v1/cubes/{cube\_id}/rows/{d1\_key} |
| :---: | :---- |

*Update attributes of a HyperRow. Each attribute write generates the appropriate CrdtOperation (SetAttr, AddToSet, IncrCounter, TransitionState). Partial update — only listed attributes are changed.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| d1\_key | path | uuid | Yes | D₁ key. |
| If-Match | header | string | No | ETag / VectorClock version for optimistic concurrency. |

**Request Body:**

| {   "attributes": {     "name":       "Acme Corporation",       // SetAttr LWW     "status":     "Inactive",              // TransitionState (if Lattice CRDT)     "tags":       {       "add":    \["fortune-500"\],            // AddToSet OR-Set       "remove": \["tier-1"\]                  // RemoveFromSet OR-Set (by value lookup)     },     "restart\_count": { "incr": 1 },        // IncrCounter (GrowOnly)     "budget":     { "amount": "75000.00", "currency": "USD" }   } } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Updated | Updated HyperRow object with new vector\_clock |
| 400 | Invalid state transition | Attempted lifecycle transition not valid in Lattice. |
| 403 | Insufficient tier | Requires Editor+ tier for most attrs. Specific attrs may require higher tier. |
| 403 | Shadow read-only | Attempted write to shadow attr not in write\_back\_attrs. |
| 409 | Version conflict | If-Match ETag does not match current version. |
| Note: CRDT conflicts (concurrent writes from multiple nodes) do NOT return 409\. They are resolved per CRDT semantics and a ConflictRecord is generated. The X-Conflict-ID response header will be set if a conflict was recorded. |  |  |

| DELETE | /api/v1/cubes/{cube\_id}/rows/{d1\_key} |
| :---: | :---- |

*Soft-archive a HyperRow. Sets status=Archived. The row remains accessible via as\_of queries and the EventLog. Does not delete the physical cell data.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| d1\_key | path | uuid | Yes | D₁ key. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 204 No Content | Archived | No body. |
| 403 | Insufficient tier | Requires Owner tier. |
| Note: Physical deletion is not supported in Hypergrid. All data is preserved in the EventLog for audit and time-travel. To hard-delete (GDPR erasure), use the GDPR erasure endpoint POST /identity/erasure. |  |  |

## **8\. HyperCells — N-Dimensional Coordinate API**

The HyperCell API provides direct access to individual N-dimensional coordinates. Use this when you need to read or write a specific cell at a full coordinate tuple, including higher dimensions (D₃, D₄...). For entity-level operations, prefer the HyperRow API.

| GET | /api/v1/cubes/{cube\_id}/cells/{coord} |
| :---: | :---- |

*Get a single HyperCell at a specific N-dimensional coordinate. The coord path parameter is a comma-separated tuple of dimension key values in order D₁..Dₙ.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| coord | path | string | Yes | Comma-separated coordinate tuple. Example: 'uuid-entity,health\_score,2026-Q2,EMEA' for N=4. |
| as\_of | query | datetime | No | Time-travel: return cell state at this timestamp. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | HyperCell object with attributes map |
| 404 | Not found | No cell exists at this coordinate (sparse cube — coordinate is empty). Not an error for writes — use PUT to create. |
| Note: For sparse cubes, a 404 at a coordinate means no data exists there. This is normal for TimeAxis and GeoAxis dimensions where most coordinate combinations are empty. |  |  |

| PUT | /api/v1/cubes/{cube\_id}/cells/{coord} |
| :---: | :---- |

*Write a complete attribute map to a specific N-dimensional coordinate. Creates the cell if it doesn't exist. All provided attributes are written as SetAttr CrdtOperations.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| coord | path | string | Yes | Comma-separated N-dim coordinate tuple. |

**Request Body:**

| {   "attributes": {     "value":      87.4,     "data\_type":  "Number",     "format":     "0.0",     "computed\_by":"kogi.health\_score\_engine.v2",     "confidence": 0.92   } } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Updated/Created | HyperCell object |
| 400 | Invalid coordinate | Coordinate tuple length does not match cube.n. |
| 403 | Insufficient tier | Requires Editor+ or System tier for AI attrs. |

| PATCH | /api/v1/cubes/{cube\_id}/cells/{coord}/{attr\_key} |
| :---: | :---- |

*Update a single attribute at a specific N-dimensional coordinate. Generates the appropriate CrdtOperation based on the attr\_key's configured CrdtSemantics.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| coord | path | string | Yes | N-dim coordinate tuple. |
| attr\_key | path | string | Yes | Attribute key to update. |

**Request Body:**

| // For LWW attrs: { "value": "new\_value" }   // For OR-Set attrs (TagSet, MultiEnum, MultiRelation): { "add": \["tag1","tag2"\], "remove": \["old-tag"\] }   // For Counter attrs: { "incr": 5 }   // For Lattice (status transition): { "from": "Active", "to": "Deprecated" }   // For JSON CRDT: { "deep\_merge": { "config\_key": "new\_value" } } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Updated | { cell: HyperCell, conflict\_id: null | uuid } |
| 400 | Invalid operation | incr \< 0 on GrowOnly counter, invalid state transition, etc. |
| 403 | Permission denied | write\_permission tier not met for this attr\_key. |
| Note: If the write produces a CRDT conflict (concurrent write from another node arrived before this one), the response still returns 200 OK but includes a non-null conflict\_id in the response body. The X-Conflict-ID response header is also set. |  |  |

| GET | /api/v1/cubes/{cube\_id}/rows/{d1\_key}/history |
| :---: | :---- |

*Get the full EventLog for a HyperRow: all mutations, actors, timestamps, before/after values. Paginated, most recent first.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| d1\_key | path | uuid | Yes | D₁ key. |
| attr\_key | query | string | No | Filter to changes of a specific attribute key. |
| since | query | datetime | No | Only return events after this timestamp. |
| until | query | datetime | No | Only return events before this timestamp. |
| limit | query | integer | No | Default: 100\. Max: 1000\. |
| cursor | query | string | No | Pagination cursor. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: EventLogEntry\[\], meta: PaginationMeta } |
| 403 | Insufficient tier | Requires Editor+ tier. |
| 404 | Not found | HyperRow not found. |
| Note: EventLog entries are immutable and cannot be deleted. For GDPR erasure requests, the actor\_id in EventLog entries is anonymized but the structural record (what changed, when) is preserved. |  |  |

# **Part V — HyperQL Query API**

## **9\. HyperQL — N-Dimensional Query Language**

HyperQL is Hypergrid's native N-dimensional query language. It extends SQL-like syntax with operations specific to N-dimensional grids: DimSlice filtering, DimFold aggregation, DimExpand pivot, TRAVERSE GRAPH Hypergraph traversal, AS\_OF time-travel, and INCLUDE SHADOW CELLS for cross-Grid queries. The HyperQL API exposes synchronous (small result sets) and asynchronous (large analytical queries) execution modes.

### **9.1 Synchronous Query**

| POST | /api/v1/cubes/{cube\_id}/query |
| :---: | :---- |

*Execute a HyperQL query synchronously against a specific Hypercube. For result sets under 100,000 rows. Larger queries should use the async endpoint.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Target Hypercube. Can be overridden in the HyperQL query FROM clause. |
| timeout\_ms | query | integer | No | Max query execution time. Default: 10,000ms. Max: 30,000ms. |

**Request Body:**

| {   "query":   "SELECT D1.entity\_id, cell\[D1,'name'\].value AS name, cell\[D1,'health\_score'\].value AS score FROM kogi.portfolio.components WHERE cell\[D1,'status'\].value \= '"Active"' AND cell\[D1,'health\_score'\].value \> 70 ORDER BY cell\[D1,'health\_score'\].value DESC LIMIT 20",     // Optional: explicitly set query options (override inline)   "options": {     "as\_of":          "2026-03-01T00:00:00Z",  // time-travel     "identity\_tag":   "@alice-dev",             // partition scope     "space\_id":       "uuid",                   // Space scope     "include\_shadow": false,                    // include shadow cells?     "max\_rows":       10000                     // row limit guard   } } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Query executed | { columns: \[{ name, type }\], rows: \[\[...\], ...\], meta: { rows\_returned, execution\_ms, plan\_notes } } |
| 400 | HyperQL parse error | { error: { code: 'invalid\_hyperql', detail: 'Unexpected token at position 42', ... } } |
| 400 | Validation error | Invalid as\_of timestamp, unknown cube name, unregistered attr\_key. |
| 403 | Insufficient scope | Caller does not have read access to referenced cubes. |
| 504 | Query timeout | Query exceeded timeout\_ms. Use async endpoint for long-running queries. |
| Note: Result rows are arrays in column order matching the columns array. Null values are represented as JSON null. TypedAttrValue types are mapped to JSON types: Text→string, Number→number, Currency→{amount,currency}, TagSet→string\[\], Json→object. |  |  |

### **9.2 Asynchronous Query**

| POST | /api/v1/query/async |
| :---: | :---- |

*Submit a HyperQL query for asynchronous execution. Returns a query\_id immediately. Results can be polled or streamed via SSE. Use for large analytical queries, DimFold aggregations over millions of rows, or cross-cube joins.*

**Request Body:**

| {   "query":        "SELECT D1.entity\_id, FOLD D3 WITH SUM(value) AS total\_revenue FROM kogi.portfolio.finances WHERE D2.metric\_name \= 'revenue' AND D3.period BETWEEN '2025-Q1' AND '2026-Q4' GROUP BY D1.entity\_id ORDER BY total\_revenue DESC",   "output\_format":"json",               // json | ndjson | parquet | arrow | csv   "callback\_url": "https://app.example.com/webhooks/query",  // optional webhook   "options": {     "max\_rows":    1000000,     "timeout\_ms":  300000               // 5 minute timeout for async   } } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 202 Accepted | Query queued | { query\_id: 'uuid', status: 'queued', estimated\_ms: 5000, poll\_url: '/api/v1/query/async/{query\_id}', stream\_url: '/api/v1/query/async/{query\_id}/stream' } |
| 400 | HyperQL parse error |  |
| 403 | Insufficient scope |  |
| Note: Async queries are executed in a separate worker pool and do not block the request thread. Maximum result set: 10M rows. For larger datasets, use the export endpoint. |  |  |

| GET | /api/v1/query/async/{query\_id} |
| :---: | :---- |

*Poll status of an async query. Returns results inline when complete for result sets under 1MB. For larger results, use the stream endpoint.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| query\_id | path | uuid | Yes | Async query ID from POST /query/async. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Pending | { query\_id, status: 'running', progress\_pct: 42, started\_at, estimated\_ms\_remaining } |
| 200 OK | Complete (small result) | { query\_id, status: 'complete', columns, rows, meta: { rows\_returned, execution\_ms } } |
| 200 OK | Complete (large result) | { query\_id, status: 'complete', result\_url: '/api/v1/query/async/{id}/download', rows\_total, size\_bytes } |
| 200 OK | Failed | { query\_id, status: 'failed', error: { code, message, detail } } |
| 404 | Not found | Query ID expired (TTL: 1 hour) or not found. |

| GET | /api/v1/query/async/{query\_id}/stream |
| :---: | :---- |

*Stream results of a completed async query as NDJSON (newline-delimited JSON). Each line is one result row.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| query\_id | path | uuid | Yes | Completed async query ID. |
| format | query | string | No | ndjson (default) | csv | parquet |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Streaming | Content-Type: application/x-ndjson. Stream of row objects. |
| 202 Accepted | Still running | Query not yet complete. |
| 404 | Not found | Query ID expired or not found. |
| Note: Set Accept: application/x-ndjson and process the response as a stream. Each line is a JSON object with column-name keys. The final line is a metadata summary: { '\_\_meta': { rows\_total, execution\_ms, truncated } }. |  |  |

### **9.3 Natural Language Query**

| POST | /api/v1/query/nl |
| :---: | :---- |

*Translate a natural language question into HyperQL and execute it. Powered by the registered AIEngineAdapter NL-to-HyperQL bridge. Returns both the generated HyperQL and the query results.*

**Request Body:**

| {   "question":    "Which of my active projects are over budget this quarter?",   "cube\_context":\["kogi.portfolio.components"\],   // which cubes to consider   "identity\_tag":"@alice-dev",   "space\_id":    "uuid",   "return\_query":true                             // also return the generated HyperQL } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { generated\_hyperql: string, columns, rows, meta: { confidence, execution\_ms } } |
| 400 | AI engine not configured | NL query requires a registered AIEngineAdapter. |
| 400 | Query translation failed | AI engine could not translate the question to HyperQL. |
| Note: The generated HyperQL is returned in generated\_hyperql for transparency. Users can copy it to the /cubes/{id}/query endpoint to execute it directly with modifications. Confidence score reflects AI certainty in the translation. |  |  |

### **9.4 DimFold and DimExpand**

| POST | /api/v1/cubes/{cube\_id}/fold |
| :---: | :---- |

*Execute a DimFold aggregation: collapse one dimension axis by aggregating all its key values into a scalar summary. The result is an (N-1)-dimensional projection.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |

**Request Body:**

| {   "fold\_axis":   "time\_axis\_id",              // axis to collapse   "agg\_fn":      "SUM",                       // SUM|AVG|MIN|MAX|COUNT|COUNT\_UNIQUE|                                               //   MEDIAN|PERCENTILE|STDDEV|DISTRIBUTION   "attr\_key":    "value",   "filter": {     "d2\_key":    "revenue",                   // pre-fold filter on D₂     "d4\_key\_in": \["EMEA","APAC"\]              // pre-fold filter on D₄   },   "group\_by":    \["d1\_key"\],                  // optional post-fold grouping   "order\_by":    "folded\_value",   "order\_dir":   "desc",   "limit":       100 } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { columns: \[{ name, type }\], rows: \[\[...\]\] } |
| 400 | Invalid axis | Specified axis does not exist on this cube. |
| Note: DimFold is equivalent to FOLD D\_n WITH AGG(attr\_key) in HyperQL. For cubes with N\>4 or result sets over 1M rows, this operation is automatically routed to the ClickHouse analytics backend. |  |  |

| POST | /api/v1/cubes/{cube\_id}/expand |
| :---: | :---- |

*Execute a DimExpand pivot: expand one dimension's key set into separate result columns. Equivalent to a PIVOT or EXPAND Dₙ AS COLUMNS(…) in HyperQL.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |

**Request Body:**

| {   "expand\_axis": "category\_axis\_id",          // axis to expand   "value\_expr":  { "agg\_fn": "AVG", "attr\_key": "value" },   "filter": {     "d2\_key":    "score"   },   "group\_by":    \["d1\_key", "d3\_key"\],        // remaining axes to group by   "col\_prefix":  "cat\_",                      // prefix for generated column names   "limit":       500 } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { columns: \[{ name, type }\], rows: \[...\] } |
| Note: DimExpand dynamically generates one output column per unique key value in the expand\_axis. Column names are prefixed with col\_prefix if provided. For axes with high cardinality (\>100 unique values), set a filter first. |  |  |

# **Part VI — Hypergraph API**

## **10\. Hypergraph Nodes**

Every entity in every Hypercube is automatically registered as a HypergraphNode. Nodes are created implicitly when HyperRows are created. The Nodes API provides direct access to node attributes and metadata.

| GET | /api/v1/graph/nodes/{node\_id} |
| :---: | :---- |

*Get a HypergraphNode by ID.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| node\_id | path | uuid | Yes | HypergraphNode identifier. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { node\_id, node\_type, entity\_ref, attributes, namespace\_path, visibility } |
| 404 | Not found | Node does not exist or is not visible to caller. |

| GET | /api/v1/graph/nodes/{node\_id}/tree |
| :---: | :---- |

*Get the link tree rooted at a node: a directed subgraph of all reachable nodes up to max\_depth via specified edge types. Respects VisibilityMask — hidden nodes silently excluded.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| node\_id | path | uuid | Yes | Root node identifier. |
| max\_depth | query | integer | No | Maximum traversal depth. Default: 3\. Max: 10\. |
| min\_depth | query | integer | No | Minimum depth to include. Default: 0\. |
| edge\_type | query | string | No | Filter to specific edge type. Comma-separated for multiple. Default: all types. |
| direction | query | string | No | Outbound | Inbound | Both. Default: Outbound. |
| include\_attrs | query | boolean | No | Include node attributes in response. Default: false (IDs only for performance). |
| space\_id | query | uuid | No | Filter to edges scoped to a specific Space. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { root: NodeId, nodes: \[{ node\_id, depth, node\_type, entity\_ref, attrs? }\], edges: \[{ edge\_id, from, to, edge\_type, weight }\], total\_nodes, total\_edges } |
| 404 | Not found | Root node not found. |
| 422 | Depth limit | max\_depth \> 10 is not allowed. |
| Note: For link forests (all trees rooted at an identity), use GET /graph/nodes/{id}/forest. For the shortest path between two nodes, use GET /graph/path. |  |  |

| GET | /api/v1/graph/nodes/{node\_id}/forest |
| :---: | :---- |

*Get the complete link forest for an identity: all link trees rooted at all nodes owned by this identity. Returns the union of all their ego graphs.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| node\_id | path | uuid | Yes | Identity node identifier. |
| max\_depth | query | integer | No | Tree depth. Default: 3\. |
| edge\_type | query | string | No | Edge type filter. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { identity\_node\_id, roots: NodeId\[\], nodes: \[...\], edges: \[...\], total\_nodes, total\_edges } |
| 403 | Insufficient tier | Requires Owner tier to see full forest. |

| GET | /api/v1/graph/nodes/{node\_id}/ego |
| :---: | :---- |

*Get the ego network (depth 1+2) centered on a node: all direct connections and their connections.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| node\_id | path | uuid | Yes | Center node identifier. |
| edge\_type | query | string | No | Edge type filter. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { center: NodeId, depth\_1: \[...\], depth\_2: \[...\], edges: \[...\] } |

## **11\. Hypergraph Edges**

### **11.1 Edge Object Schema**

| {   "edge\_id":        "uuid",   "from\_node":      "uuid",   "to\_node":        "uuid",   "edge\_type":      "Hierarchy",            // Hierarchy|Dependency|Association|Contains|                                             //   Derives|CrossGridLink|ShadowOf|                                             //   SpaceMembership|Collaborates|References|                                             //   FederationPeer|NamespaceAlias|ComputedFrom|                                             //   SubscribesTo|InvestedIn|Employs|Custom(String)   "direction":      "Directed",             // Directed|Bidirectional   "weight":         1.0,                    // 0.0–1.0   "attributes":     {     "role":         "contributor",          // edge-level attributes     "since":        "2026-01-15T09:00:00Z"   },   "visibility":     "Tenant",   "consent\_status": "Accepted",             // None|Pending|Accepted|Declined|Revoked   "consent\_config": {     "mirrored\_attrs":    \["name","status","health\_score"\],     "write\_back\_attrs":  \[\],     "update\_policy":     "RealTime"         // RealTime|Batched|Manual   },   "space\_scope":    null,                   // null \= global   "shadow\_cell\_id": "uuid",                 // null if no shadow cell   "expires\_at":     null,   "created\_at":     "2026-01-20T09:00:00Z",   "created\_by":     "user-uuid:@alice:node-1" } |
| :---- |

| GET | /api/v1/graph/edges |
| :---: | :---- |

*List HypergraphEdges with optional filters.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| from\_node | query | uuid | No | Filter by source node. |
| to\_node | query | uuid | No | Filter by target node. |
| edge\_type | query | string | No | Filter by edge type. Comma-separated. |
| consent\_status | query | string | No | Filter by consent status: Pending|Accepted|Declined|Revoked. |
| space\_id | query | uuid | No | Filter to edges scoped to a Space. |
| limit | query | integer | No | Default: 50\. Max: 500\. |
| cursor | query | string | No | Pagination cursor. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: HypergraphEdge\[\], meta: PaginationMeta } |

| POST | /api/v1/graph/edges |
| :---: | :---- |

*Create a HypergraphEdge. For edge types that require consent (CrossGridLink, Collaborates, InvestedIn, Employs), the edge is created with consent\_status=Pending and a consent invitation is sent to the target node owner.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| X-Idempotency-Key | header | string | No | Idempotency key. |

**Request Body:**

| {   "from\_node":   "uuid",   "to\_node":     "uuid",   "edge\_type":   "CrossGridLink",         // see EdgeType enum   "direction":   "Directed",   "weight":      1.0,   "attributes":  { "relationship": "contractor" },   "space\_scope": null,   "expires\_at":  null,     // For CrossGridLink, Collaborates, InvestedIn, Employs:   "consent\_request": {     "message":          "I'd like to link our projects to share status updates.",     "proposed\_mirrored\_attrs":   \["name","status","health\_score"\],     "proposed\_write\_back\_attrs": \[\],     "proposed\_update\_policy":    "RealTime"   } } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | Edge created / consent pending | HypergraphEdge object with consent\_status=None or Pending |
| 400 | Cyclic dependency | Adding this Hierarchy or Dependency edge would create a cycle. |
| 403 | Insufficient tier | Requires Contributor+ tier. |
| 409 | Already exists | An edge of this type between these nodes already exists. |
| Note: For edges requiring consent, the response has consent\_status=Pending. The target node owner receives a consent invitation. Poll GET /graph/edges/{id} for status changes, or subscribe to the WebSocket space activity stream. |  |  |

| GET | /api/v1/graph/edges/{edge\_id} |
| :---: | :---- |

*Get a specific HypergraphEdge by ID.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| edge\_id | path | uuid | Yes | Edge identifier. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | HypergraphEdge object |
| 404 | Not found | Edge does not exist or is not visible. |

| PATCH | /api/v1/graph/edges/{edge\_id} |
| :---: | :---- |

*Update edge attributes: weight, attribute map, consent\_config, expiry. Cannot change from\_node, to\_node, or edge\_type.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| edge\_id | path | uuid | Yes | Edge identifier. |

**Request Body:**

| {   "weight":      0.8,   "attributes":  { "role": "lead\_contributor" },   "consent\_config": {     "mirrored\_attrs":  \["name","status","health\_score","risk\_score"\],     "update\_policy":   "Batched"            // RealTime|Batched|Manual   },   "expires\_at":  "2027-01-01T00:00:00Z" } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Updated | Updated HypergraphEdge |
| 403 | Insufficient tier | Requires Owner tier of this edge. |

| POST | /api/v1/graph/edges/{edge\_id}/consent |
| :---: | :---- |

*Accept or decline a pending consent invitation for a CrossGridLink or similar edge type. Called by the target node owner.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| edge\_id | path | uuid | Yes | Edge identifier. |

**Request Body:**

| {   "decision":    "accept",                 // accept | decline   "mirrored\_attrs":    \["name","status"\],  // subset of proposed\_mirrored\_attrs   "write\_back\_attrs":  \[\],                 // subset of proposed\_write\_back\_attrs   "update\_policy":     "RealTime",   "message":     "Happy to link — sharing name and status only." } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Consent recorded | Updated HypergraphEdge with consent\_status=Accepted|Declined. If Accepted, shadow cell provisioning begins asynchronously. |
| 400 | Invalid decision | Decision must be accept or decline. |
| 403 | Not the target owner | Only the target node owner can respond to consent. |
| 404 | Edge not found |  |
| Note: When consent is Accepted, the shadow cell provisioning pipeline begins. The shadow\_cell\_id field on the edge will be populated once provisioning completes (poll or use WebSocket stream). |  |  |

| DELETE | /api/v1/graph/edges/{edge\_id} |
| :---: | :---- |

*Delete a HypergraphEdge. For CrossGridLink edges with an active shadow cell, triggers the shadow cell disconnection protocol: shadow\_status set to Disconnected. Historical sync data preserved in EventLog.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| edge\_id | path | uuid | Yes | Edge identifier. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 204 No Content | Deleted | No body. |
| 403 | Insufficient tier | Requires Owner tier of this edge. |
| Note: After deletion, existing ShadowRows from this edge remain readable but are marked shadow\_status=Disconnected. They will not receive further updates. |  |  |

## **12\. Shadow Cells**

| GET | /api/v1/graph/shadow-cells |
| :---: | :---- |

*List all ShadowCells in this Grid that were provisioned from linked external Grids via CrossGridLink edges.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| source\_grid\_id | query | uuid | No | Filter by source Grid. |
| shadow\_status | query | string | No | Filter: Active|Degraded|Disconnected. |
| limit | query | integer | No | Default: 50\. |
| cursor | query | string | No | Pagination cursor. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: ShadowCell\[\], meta: PaginationMeta } |

| POST | /api/v1/graph/shadow-cells/{shadow\_id}/sync |
| :---: | :---- |

*Manually trigger a MirrorAttribute sync for a ShadowCell from its source entity. Forces an immediate fetch of the consented attributes from the source Grid.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| shadow\_id | path | uuid | Yes | ShadowCell identifier. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Sync complete | { shadow\_id, synced\_attrs: \[...\], synced\_at, shadow\_status: 'Active' } |
| 403 | Insufficient tier | Requires Editor+ tier. |
| 404 | Not found | ShadowCell ID not found. |
| 409 | Link revoked | The CrossGridLink has been revoked. Sync not possible. |
| 502 | Source unavailable | Source Grid did not respond. shadow\_status set to Degraded. |

## **13\. Graph Analytics**

| GET | /api/v1/graph/path |
| :---: | :---- |

*Find the shortest weighted path between two nodes in the Hypergraph using Dijkstra's algorithm. Edge weight is used as the path cost (lower weight \= preferred path).*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| from | query | uuid | Yes | Source node ID. |
| to | query | uuid | Yes | Target node ID. |
| edge\_type | query | string | No | Filter to specific edge types along path. |
| max\_hops | query | integer | No | Maximum path length. Default: 10\. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Path found | { path: \[NodeId\], edges: \[EdgeId\], total\_weight, hop\_count } |
| 404 | No path | No path exists between the two nodes with given constraints. |
| Note: Returns the path as an ordered array of node IDs and the edges traversed. Total weight is the sum of edge weights along the path. |  |  |

| GET | /api/v1/graph/clusters |
| :---: | :---- |

*Detect community clusters in the Hypergraph using the Louvain algorithm. Returns cluster assignments for all nodes within a Space or subgraph.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| space\_id | query | uuid | Yes | Space whose link subgraph to analyze. |
| edge\_type | query | string | No | Only consider specified edge types. |
| min\_cluster\_size | query | integer | No | Minimum nodes per cluster. Default: 3\. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { clusters: \[{ cluster\_id, node\_ids, modularity\_score }\], total\_clusters, overall\_modularity } |
| 422 | Graph too large | Space subgraph exceeds 1M nodes. Use sampled approximation or reduce scope. |
| Note: Cluster detection uses Louvain modularity optimization. Results are stored as a computed attribute on each node HyperRow for fast subsequent access. For graphs \> 100K nodes, sampled random-walk approximation is used. |  |  |

| GET | /api/v1/graph/centrality |
| :---: | :---- |

*Compute centrality scores for nodes in the Hypergraph: betweenness, closeness, and eigenvector (PageRank-style) centrality.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| space\_id | query | uuid | No | Scope to Space subgraph. |
| metric | query | string | No | betweenness | closeness | eigenvector | all. Default: all. |
| top\_n | query | integer | No | Return only top N nodes by centrality. Default: 100\. Max: 1000\. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { nodes: \[{ node\_id, betweenness, closeness, eigenvector }\], computed\_at } |
| 202 Accepted | Computing | For large graphs, computation is async. Returns query\_id for polling. |
| Note: Centrality scores are written back as AI-computed attributes on HyperRow nodes for fast cached access. Full recomputation is triggered on significant graph topology changes. |  |  |

# **Part VII — Spaces and Workspaces API**

## **14\. Spaces**

### **14.1 Space Object Schema**

| {   "space\_id":        "uuid",   "space\_type":      "Organization",         // Personal|Team|Organization|Cooperative|                                              //   Collective|Community|Federation|                                              //   Project|Research|Event|Studio|Factory   "slug":            "pamoja-capital",   "name":            "Pamoja Capital Cooperative",   "namespace\_path":  "kogi://coop/pamoja-capital/",   "grid\_id":         "uuid",   "visibility":      "Internal",             // Private|Protected|Internal|Public|Unlisted   "status":          "Active",               // Draft|Active|Paused|Restricted|                                              //   Dissolving|Dissolved|Archived   "member\_count":    47,   "cube\_ids":        \["uuid-1", "uuid-2"\],   "workspace\_count": 12,   "governance": {     "vote\_threshold\_pct":  51,     "quorum\_pct":          30,     "proposal\_types":      \["policy","budget","member","constitution"\],     "multisig\_required":   false   },   "treasury\_ref":    { "account\_id": "kogi-bank-uuid", "balance\_summary": "..." },   "link\_node\_id":    "uuid",   "federation\_links":\["uuid-fed-1"\],   "created\_at":      "2025-06-01T09:00:00Z",   "updated\_at":      "2026-03-20T14:00:00Z" } |
| :---- |

| GET | /api/v1/spaces |
| :---: | :---- |

*List all Spaces the authenticated caller is a member of, or publicly discoverable Spaces.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| space\_type | query | string | No | Filter by SpaceType. Comma-separated. |
| visibility | query | string | No | Public | Internal | All (requires auth). Default: shows accessible Spaces. |
| name\_search | query | string | No | Full-text search on Space name and description. |
| limit | query | integer | No | Default: 50\. |
| cursor | query | string | No | Pagination cursor. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: Space\[\], meta: PaginationMeta } |

| POST | /api/v1/spaces |
| :---: | :---- |

*Create a new Space.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| X-Idempotency-Key | header | string | No | Idempotency key. |

**Request Body:**

| {   "space\_type":  "Team",   "slug":        "engineering-alpha",          // URL-safe, unique within parent namespace   "name":        "Engineering Alpha Squad",   "description": "Core platform engineering team",   "visibility":  "Internal",   "governance": {     "vote\_threshold\_pct": 51,     "quorum\_pct":         30   },   "parent\_space\_id": "uuid",                  // optional: nest under a parent Space   "template\_id":     "uuid",                  // optional: apply a Space template   "initial\_members": \[                        // optional: invite initial members     { "user\_id": "uuid", "role": "Manager" }   \] } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | Space created | Space object |
| 400 | Validation error | Invalid slug, missing required fields. |
| 409 | Slug taken | A Space with this slug already exists. |
| Note: Creating a Space provisions: a root Hypercube for the Space, a KLNK LinkNode for the Space in the Hypergraph, a Namespace entry at the declared namespace\_path, and a system Workspace template. |  |  |

| GET | /api/v1/spaces/{space\_id} |
| :---: | :---- |

*Get full Space details including member roster, cube list, and governance config.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| space\_id | path | uuid | Yes | Space identifier. |
| include\_members | query | boolean | No | Include full member roster. Default: false. |
| include\_cubes | query | boolean | No | Include cube list. Default: false. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | Space object |
| 403 | Not a member | Caller is not a member and Space is not Public. |
| 404 | Not found |  |

| PATCH | /api/v1/spaces/{space\_id} |
| :---: | :---- |

*Update Space configuration: name, description, visibility, governance config.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| space\_id | path | uuid | Yes | Space identifier. |

**Request Body:**

| {   "name":        "Engineering Alpha (Platform)",   "description": "Updated description",   "visibility":  "Public",   "governance": {     "vote\_threshold\_pct": 66   } } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Updated | Updated Space object |
| 403 | Insufficient tier | Requires Manager+ tier within this Space. |
| 409 | Governance required | Visibility changes may require a GovernanceProposal vote. |

### **14.2 Space Membership**

| GET | /api/v1/spaces/{space\_id}/members |
| :---: | :---- |

*List Space members with roles, join dates, and contribution weights.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| space\_id | path | uuid | Yes | Space identifier. |
| role | query | string | No | Filter by role: Owner|Governor|Steward|Treasurer|Editor|Contributor|Member|Viewer. |
| limit | query | integer | No | Default: 100\. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: \[{ user\_id, identity\_tag, role, joined\_at, contribution\_weight, stake\_pct }\], meta: PaginationMeta } |
| 403 | Not a member | Requires Member tier. |

| POST | /api/v1/spaces/{space\_id}/members |
| :---: | :---- |

*Add a new member to a Space or accept a pending invitation.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| space\_id | path | uuid | Yes | Space identifier. |

**Request Body:**

| {   "user\_id":      "uuid",   "identity\_tag": "@alice-dev",   "role":         "Contributor",          // role within the Space   "message":      "Welcome to the team\!" } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | Member added | SpaceMember object |
| 403 | Insufficient tier | Requires Steward+ tier to add members. |
| 409 | Already a member | User is already a member of this Space. |

| PATCH | /api/v1/spaces/{space\_id}/members/{user\_id} |
| :---: | :---- |

*Update a member's role within a Space.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| space\_id | path | uuid | Yes | Space identifier. |
| user\_id | path | uuid | Yes | Member user ID. |

**Request Body:**

| { "role": "Manager" } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Updated | Updated SpaceMember object |
| 403 | Insufficient tier | Requires Steward+ tier to change member roles. |

| DELETE | /api/v1/spaces/{space\_id}/members/{user\_id} |
| :---: | :---- |

*Remove a member from a Space.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| space\_id | path | uuid | Yes | Space identifier. |
| user\_id | path | uuid | Yes | Member user ID. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 204 No Content | Removed | No body. |
| 403 | Insufficient tier | Requires Manager+ tier to remove members. Members can remove themselves. |

### **14.3 Space Governance**

| GET | /api/v1/spaces/{space\_id}/governance/proposals |
| :---: | :---- |

*List governance proposals for a Space.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| space\_id | path | uuid | Yes | Space identifier. |
| status | query | string | No | open | passed | rejected | all. Default: open. |
| limit | query | integer | No | Default: 20\. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: GovernanceProposal\[\], meta } |

| POST | /api/v1/spaces/{space\_id}/governance/proposals |
| :---: | :---- |

*Submit a governance proposal to a Space.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| space\_id | path | uuid | Yes | Space identifier. |

**Request Body:**

| {   "title":       "Increase vote threshold to 66%",   "description": "To better represent minority voices, we propose raising the voting threshold.",   "proposal\_type": "policy",               // policy|budget|member|constitution|dissolution|custom   "action": {     "type":      "update\_governance",     "changes":   { "vote\_threshold\_pct": 66 }   },   "voting\_deadline": "2026-04-15T23:59:59Z" } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | Proposal submitted | GovernanceProposal object with status=Open |
| 403 | Insufficient tier | Requires Manager+ tier. |

| POST | /api/v1/spaces/{space\_id}/governance/proposals/{proposal\_id}/vote |
| :---: | :---- |

*Cast a vote on an open governance proposal.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| space\_id | path | uuid | Yes | Space identifier. |
| proposal\_id | path | uuid | Yes | Proposal identifier. |

**Request Body:**

| {   "vote":    "yes",                        // yes | no | abstain   "comment": "I support this change." } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Vote cast | { proposal\_id, your\_vote, current\_tally: { yes, no, abstain, quorum\_reached, threshold\_met } } |
| 400 | Voting closed | Voting deadline has passed or proposal is no longer open. |
| 409 | Already voted | Caller has already voted on this proposal. |

## **15\. Workspaces**

| POST | /api/v1/workspaces |
| :---: | :---- |

*Create a new Workspace within a Space.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| X-Idempotency-Key | header | string | No | Idempotency key. |

**Request Body:**

| {   "name":          "Q2 Planning Workspace",   "space\_id":      "uuid",                  // required   "workspace\_type":"Personal",              // Personal|Shared|Project|Template   "description":   "Planning session for Q2 2026",   "initial\_config": {     "open\_cubes": \[       { "cube\_id": "uuid", "view\_id": "uuid" }     \],     "dim\_context": {       "time\_axis\_id": "2026-Q2"             // default time slice for all cubes     }   } } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | Workspace created | Workspace object |
| 403 | Insufficient tier | Requires Editor+ tier. |

| GET | /api/v1/workspaces/{workspace\_id} |
| :---: | :---- |

*Get current Workspace state including open cubes, dim context, and active session info.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| workspace\_id | path | uuid | Yes | Workspace identifier. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | Workspace object with full session state |
| 403 | Not a member | Caller is not a member of the Space owning this Workspace. |

| PUT | /api/v1/workspaces/{workspace\_id}/dim-context |
| :---: | :---- |

*Set the workspace-level DimensionKey defaults. When set, ALL open cubes in this Workspace automatically show the specified dimension slice. This is the N-dimensional generalization of freeze panes.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| workspace\_id | path | uuid | Yes | Workspace identifier. |

**Request Body:**

| {   "dim\_context": {     "time\_axis\_uuid":     "2026-Q2",      // all cubes default to Q2 time slice     "geo\_axis\_uuid":      "EMEA",         // all cubes default to EMEA geo slice     "scenario\_axis\_uuid": "base\_case"     // all cubes show base case scenario   } } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Updated | Updated Workspace object |
| 400 | Invalid axis ID | One or more axis IDs not found in any open cube. |
| Note: Setting a dim\_context does not modify any cell data. It adds a DimSlice predicate to all queries executed within this Workspace session, providing a coherent multi-cube view of the same dimensional slice. |  |  |

| POST | /api/v1/workspaces/{workspace\_id}/session |
| :---: | :---- |

*Open a Workspace session. Returns a WebSocket URL for real-time collaborative editing and a session token.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| workspace\_id | path | uuid | Yes | Workspace identifier. |

**Request Body:**

| { "client\_info": { "platform": "web", "user\_agent": "..." } } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Session opened | { session\_id, websocket\_url, session\_token, expires\_at } |
| 403 | Not a member |  |
| Note: Use the websocket\_url to connect to the collaborative editing WebSocket channel. The session\_token is required as a query parameter: ?session\_token={token}. Sessions expire after 1 hour of inactivity. |  |  |

# **Part VIII — Namespace API**

## **16\. Namespace Registry**

The Namespace system provides globally unique, hierarchical URI addressing for every entity. The NamespacePath scheme is: hypergrid://{grid\_name}/{space\_type}/{slug}/{entity\_type}/{entity\_slug}/... for Hypergrid-native entities, and domain-specific prefixes (kogi://, ume://, qala://) for domain system entities.

| GET | /api/v1/ns/resolve |
| :---: | :---- |

*Resolve a NamespacePath URI to its entity ID, entity type, and Grid location. Supports aliases and federated entries.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| path | query | string | Yes | NamespacePath URI to resolve. URL-encoded. Example: kogi://coop/pamoja-capital/ |
| include\_meta | query | boolean | No | Include full NamespaceEntry metadata. Default: false. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Resolved | { entity\_id: 'uuid', entity\_type: 'Space', grid\_id: 'uuid', namespace\_path: '...', canonical\_url: '...', resolved\_from: 'direct|alias|federated' } |
| 404 | Not found | Path does not resolve to any known entity. This path may not have been registered. |
| 302 / redirect\_url | Alias | When the path is an alias, returns redirect\_url pointing to the canonical path. |
| Note: Namespace resolution is Redis-cached with a 300s TTL. The response includes resolved\_from to indicate whether this was a direct match, alias redirect, or federated lookup from a peer Grid. |  |  |

| POST | /api/v1/ns |
| :---: | :---- |

*Register a new NamespacePath entry, binding it to an entity.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| X-Idempotency-Key | header | string | No | Idempotency key. |

**Request Body:**

| {   "path":         "kogi://coop/pamoja-capital/projects/webapp-2026/",   "entity\_id":    "uuid",   "entity\_type":  "Row",                   // Grid|Cube|Row|Space|Workspace|Identity|Custom   "visibility":   "Public",   "canonical\_url":"https://kogi.io/coop/pamoja-capital/projects/webapp-2026" } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | Registered | NamespaceEntry object |
| 400 | Invalid path | Path format is invalid or conflicts with system-reserved paths. |
| 403 | Insufficient tier | Must be Owner of the parent namespace segment. |
| 409 | Already exists | A namespace entry at this path already exists. |
| Note: Namespace registration is propagated to all federation peers automatically. Cross-linked entities become resolvable at all peer Grids once registered. |  |  |

| GET | /api/v1/ns/list |
| :---: | :---- |

*List namespace entries under a path prefix.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| prefix | query | string | Yes | Namespace path prefix. Example: kogi://coop/pamoja-capital/ |
| depth | query | integer | No | Maximum depth below prefix. Default: 1 (direct children only). Max: 5\. |
| entity\_type | query | string | No | Filter by entity type. |
| limit | query | integer | No | Default: 100\. Max: 1000\. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: NamespaceEntry\[\], meta: PaginationMeta } |

| POST | /api/v1/ns/{namespace\_id}/alias |
| :---: | :---- |

*Create an additional path alias that resolves to the same entity as the canonical path.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| namespace\_id | path | uuid | Yes | NamespaceEntry identifier. |

**Request Body:**

| { "alias\_path": "kogi://webapp-2026/" } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | Alias created | { alias\_path, canonical\_path, namespace\_id } |
| 403 | Insufficient tier | Requires Owner of this namespace entry. |
| 409 | Already exists | Alias path is already in use. |

| PATCH | /api/v1/ns/{namespace\_id} |
| :---: | :---- |

*Rename a namespace path segment. Creates an alias from the old path to the new path, so existing references continue to work.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| namespace\_id | path | uuid | Yes | NamespaceEntry identifier. |

**Request Body:**

| {   "path":         "kogi://coop/pamoja-capital/projects/webapp-v2-2026/",   "create\_alias": true                     // create alias from old path (default: true) } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Renamed | Updated NamespaceEntry |
| 403 | Insufficient tier | Requires Owner of this namespace entry. |
| 409 | Already exists | New path is already in use. |

| POST | /api/v1/ns/{namespace\_id}/federate |
| :---: | :---- |

*Replicate a namespace entry to a federation peer Grid, making it resolvable at the peer.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| namespace\_id | path | uuid | Yes | NamespaceEntry identifier. |

**Request Body:**

| { "peer\_grid\_id": "uuid" } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Federated | { namespace\_id, federated\_to: GridId\[\] } |
| 403 | Insufficient tier | Requires Admin tier. |
| 404 | Peer not found | Peer Grid not registered as a federation peer. |

| GET | /api/v1/ns/health |
| :---: | :---- |

*Get namespace health report: stale entries, broken aliases, unresolvable paths, federation sync status.*

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { total\_entries, stale\_count, broken\_aliases, avg\_resolve\_latency\_ms, federation\_sync\_status: { peer\_id, last\_sync\_at, pending\_count }\[\] } |
| 403 | Admin required | Requires Admin tier. |

# **Part IX — Identity and Access API**

## **17\. Identity Management**

| GET | /api/v1/identity/me |
| :---: | :---- |

*Get the authenticated caller's identity profile: SovereignEntity, all TenantPartitions, active spaces and workspaces.*

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { sovereign\_entity\_id, partitions: TenantPartition\[\], active\_spaces: Space\[\], api\_keys: \[{ key\_id, name, scopes, last\_used }\] } |

| GET | /api/v1/identity/partitions |
| :---: | :---- |

*List all TenantPartitions (identity handles) for the authenticated SovereignEntity.*

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: TenantPartition\[\] } |

| POST | /api/v1/identity/partitions |
| :---: | :---- |

*Create a new TenantPartition (a new named identity handle) for the authenticated SovereignEntity.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| X-Idempotency-Key | header | string | No | Idempotency key. |

**Request Body:**

| {   "identity\_handle":  "@alice-creative",     // @handle — unique within SovereignEntity   "display\_name":     "Alice (Creative Work)",   "isolation\_level":  "SoftIsolated",        // None|SoftIsolated|HardIsolated   "visibility\_mask": {     "public\_attrs":   \["name","bio","tags","portfolio\_url"\],     "follower\_attrs": \["name","bio","tags","portfolio\_url","recent\_projects"\],     "owner\_only\_attrs":\["bank\_account\_ref","tax\_id","ssn\_ref"\]   } } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | Partition created | TenantPartition object |
| 400 | Invalid handle | Handle contains invalid characters or is reserved. |
| 409 | Handle taken | An identity handle with this name already exists. |

| PATCH | /api/v1/identity/partitions/{partition\_id} |
| :---: | :---- |

*Update a TenantPartition: display name, isolation level, visibility mask.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| partition\_id | path | uuid | Yes | Partition identifier. |

**Request Body:**

| {   "display\_name":    "Alice — Developer Identity",   "isolation\_level": "HardIsolated",   "visibility\_mask": {     "public\_attrs":  \["name","bio"\]   } } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Updated | Updated TenantPartition |
| 400 | Cannot downgrade isolation | HardIsolated cannot be downgraded without explicit force flag. |

| POST | /api/v1/identity/partitions/merge |
| :---: | :---- |

*Initiate a CrossIdentityMerge: merge two TenantPartitions of the same SovereignEntity. Irreversible for KLNK node merging. Full checkpoint created before merge begins.*

**Request Body:**

| {   "source\_partition\_id": "uuid",            // partition to merge FROM (will be archived)   "target\_partition\_id": "uuid",            // partition to merge INTO (survives)   "dry\_run":             true,              // simulate without executing (default: true)   "create\_namespace\_alias": true            // create redirect from source @handle to target } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Dry run complete | { source\_handle, target\_handle, rows\_to\_retag, shared\_rows, namespace\_aliases\_to\_create, klnk\_edges\_to\_merge, checkpoint\_size\_estimate\_mb } |
| 202 Accepted | Merge initiated (dry\_run=false) | { merge\_id, checkpoint\_id, status: 'in\_progress', estimated\_completion\_at } |
| 400 | Cannot merge | Source and target belong to different SovereignEntities. |
| Note: Always perform a dry run first to review the impact. The actual merge creates a pre-merge snapshot (restorable within 7 days). KLNK node merging is permanent. |  |  |

| POST | /api/v1/identity/api-keys |
| :---: | :---- |

*Create a new API Key for server-to-server access.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| X-Idempotency-Key | header | string | No | Idempotency key. |

**Request Body:**

| {   "name":        "kogi-engine-service-key",   "scopes":      \["cubes:read","cells:write:ai","eventlog:append"\],   "cube\_restrictions": \["uuid-cube-1", "uuid-cube-2"\],   // empty \= all cubes   "expires\_at":  "2027-01-01T00:00:00Z",                 // null \= non-expiring   "description": "Used by kogi-engine WritebackService" } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | API key created | { key\_id, name, api\_key: '...', scopes, expires\_at }. NOTE: api\_key value shown ONCE — store securely. |
| 400 | Invalid scope | One or more scopes are not recognized. |
| Note: The api\_key secret value is shown only in the 201 response. It cannot be retrieved again. If lost, delete and recreate the key. |  |  |

| DELETE | /api/v1/identity/api-keys/{key\_id} |
| :---: | :---- |

*Revoke an API Key immediately.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| key\_id | path | uuid | Yes | API key identifier. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 204 No Content | Revoked | No body. All in-flight requests using this key will fail immediately. |

# **Part X — AI and Intelligence API**

## **18\. AI Computation**

| POST | /api/v1/ai/compute |
| :---: | :---- |

*Request Tier-2 AI-computed attribute values for a batch of entities. Triggers AIEngineAdapter.compute\_batch() on the specified engine. Results are written back to HyperCells via WritebackService and returned synchronously if available within timeout.*

**Request Body:**

| {   "engine\_id":   "kogi.health\_score\_engine.v2",   "cube\_id":     "uuid",   "entity\_ids":  \["uuid-1","uuid-2","uuid-3"\],   "force":       false,                    // force recompute even if within cache TTL   "sync":        true,                     // wait for results (up to timeout\_ms)   "timeout\_ms":  5000                      // max wait. if exceeded, returns job\_id for polling } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Results ready (sync=true, within timeout) | { results: \[{ entity\_id, outputs: { attr\_key: value, ... }, confidence, computed\_at }\] } |
| 202 Accepted | Computing (sync=false or timed out) | { job\_id, status: 'queued', poll\_url: '...' } |
| 400 | Engine not found | engine\_id not registered. |
| 403 | Insufficient tier | Requires Editor+ tier. |
| Note: AI computation is billed per-entity per-compute. Use force=false (default) to serve cached results within the engine's configured staleness\_ttl. |  |  |

| GET | /api/v1/ai/anomalies |
| :---: | :---- |

*List AI-detected anomalies in a cube or DimSlice. Anomalies are detected by registered AIEngineAdapter plugins via detect\_anomalies().*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | query | uuid | Yes | Hypercube to scan. |
| d1\_key | query | uuid | No | Scope to a specific HyperRow. |
| attr\_key | query | string | No | Scope to a specific attribute key. |
| severity\_min | query | number | No | Minimum anomaly severity (0.0–1.0). Default: 0.3. |
| kind | query | string | No | SuddenChange|PatternBreak|OutlierValue|StateChange |
| since | query | datetime | No | Only anomalies detected after this timestamp. |
| limit | query | integer | No | Default: 50\. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: \[{ anomaly\_id, entity\_id, attr\_key, kind, severity, baseline, deviation, detected\_at }\], meta: PaginationMeta } |

| POST | /api/v1/ai/anomalies/{anomaly\_id}/resolve |
| :---: | :---- |

*Mark an anomaly as acknowledged or resolved. Does not delete the anomaly record from the EventLog.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| anomaly\_id | path | uuid | Yes | Anomaly identifier. |

**Request Body:**

| {   "resolution":  "acknowledged",           // acknowledged | resolved | false\_positive   "note":        "Intentional budget increase — Q2 push." } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Resolved | Updated anomaly record |

| GET | /api/v1/ai/engines |
| :---: | :---- |

*List all registered AIEngineAdapter plugins and their health status.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| status | query | string | No | Healthy|Degraded|Unhealthy — filter by health status. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: \[{ engine\_id, name, version, capabilities, health, output\_attrs, cache\_hit\_rate }\] } |
| 403 | Admin required |  |

## **19\. WritebackService — AI Engine Write Protocol**

The WritebackService is the gRPC endpoint used exclusively by AI engines to write computed attribute values into HyperCells. It is not accessible over REST from external clients. The following documents the protocol for implementors building AIEngineAdapter plugins that write back to Hypergrid.

| // WritebackService gRPC interface (proto3) service WritebackService {     rpc WriteColumn(WriteColumnRequest) returns (WriteColumnResponse);     rpc WriteBatch(WriteBatchRequest)   returns (WriteBatchResponse);     rpc WriteAnomalyFlag(WriteAnomalyRequest) returns (WriteAnomalyResponse); }   message WriteColumnRequest {     string grid\_id         \= 1;     string cube\_id         \= 2;     string d1\_key          \= 3;   // entity D₁ key     string attr\_key        \= 4;   // must be attr\_type=AI or System     google.protobuf.Value value    \= 5;     float  confidence      \= 6;   // 0.0–1.0     string source\_engine   \= 7;   // registered engine\_id     int64  computation\_ts  \= 8;   // nanosecond epoch     int64  ttl\_seconds     \= 9;   // Redis CellCache TTL     string model\_version   \= 10;     string identity\_tag    \= 11;  // optional partition scope     string space\_id        \= 12;  // optional Space scope     // Optional: N-dim coordinates for higher dimensions     repeated DimKeyValue extra\_dims \= 13; }   message WriteBatchRequest {     repeated WriteColumnRequest writes \= 1;     bool    atomic \= 2;           // if true: all or nothing }   // Processing pipeline (7 steps): // 1\. Validate source\_engine registered and has AIWriter permission for attr\_key // 2\. Evaluate PolicyEngine for blocking policies // 3\. Threshold check: anomaly alert if value crosses configured threshold // 4\. Write as SetAttr CrdtOp with actor=ai::{source\_engine}::{model\_version} // 5\. Update Redis CellCache with TTL // 6\. Append EventEntry::AIColumnUpdated to EventLog // 7\. WebSocket broadcast CellUpdateEvent to subscribed Workspaces |
| :---- |

## **20\. Oba AI Assistant API**

Oba is the domain-specific AI assistant layer built on top of the Hypergrid AI infrastructure. It provides reactive query answering, proactive anomaly surfacing, and approval-first autonomous action preparation.

| POST | /api/v1/ai/oba/query |
| :---: | :---- |

*Submit a natural language query to Oba. Returns both AI-generated insights and structured data from HyperQL execution.*

**Request Body:**

| {   "question":    "Show me everything at risk this week in my engineering portfolio",   "cube\_context":\["kogi.portfolio.components"\],   "identity\_tag":"@alice-dev",   "space\_id":    "uuid",   "return\_hyperql": true,   "return\_insights": true                  // AI narrative in addition to data } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { answer: string, generated\_hyperql: string, columns, rows, insights: \[{ type, message, affected\_rows }\], suggestions: \[ObaSuggestion\] } |

| GET | /api/v1/ai/oba/hints/{entity\_id} |
| :---: | :---- |

*Get pending Oba proactive hints for a specific entity: anomaly warnings, completion suggestions, upcoming deadlines, risk flags.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| entity\_id | path | uuid | Yes | Entity (HyperRow D₁ key). |
| cube\_id | query | uuid | Yes | Cube containing the entity. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { entity\_id, hints: \[{ hint\_id, type, message, severity, suggested\_action, expires\_at }\] } |

| POST | /api/v1/ai/oba/actions |
| :---: | :---- |

*Submit an Oba autonomous action for user approval. The action is described in structured form with a human-readable preview. No mutations occur until the user confirms via POST /ai/oba/actions/{action\_id}/confirm.*

**Request Body:**

| {   "action\_type":  "batch\_archive",   "description":  "Archive 7 completed projects older than 90 days",   "preview": {     "affected\_entities": \["uuid-1","uuid-2","..."\],     "hyperql\_preview":   "UPDATE components SET status='Archived' WHERE ...",     "reversible":        true   },   "cube\_id":      "uuid",   "expires\_at":   "2026-03-23T17:00:00Z" } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | Action staged | { action\_id, status: 'awaiting\_confirmation', preview, expires\_at } |
| Note: Oba actions have a configurable expiry window. If not confirmed before expires\_at, the action is discarded without any mutations. |  |  |

| POST | /api/v1/ai/oba/actions/{action\_id}/confirm |
| :---: | :---- |

*Confirm and execute a pending Oba autonomous action. This is the only way to execute an Oba action — no autonomous execution occurs without explicit user confirmation.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| action\_id | path | uuid | Yes | Oba action identifier. |

**Request Body:**

| { "confirmed": true } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Action executed | { action\_id, status: 'completed', mutations\_applied, entities\_affected } |
| 400 | Action expired | Action has expired. Submit a new action request. |
| 404 | Not found | Action ID not found. |
| Note: All mutations from a confirmed Oba action are recorded in the EventLog with actor=oba::{action\_id}. They can be reversed via POST /ai/oba/actions/{action\_id}/undo within the undo window (default: 30 minutes). |  |  |

# **Part XI — Federation API**

## **21\. Federation Peers**

| GET | /api/v1/federation/peers |
| :---: | :---- |

*List all registered federation peers for this Grid.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| status | query | string | No | Active|Degraded|Disconnected — filter by peer status. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: \[{ peer\_id, grid\_id, peer\_url, trust\_level, status, lag\_ms, last\_heartbeat\_at, schema\_aligned }\] } |
| 403 | Admin required |  |

| POST | /api/v1/federation/peers |
| :---: | :---- |

*Register a new federation peer. Initiates the full 8-phase federation handshake.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| X-Idempotency-Key | header | string | No | Idempotency key. |

**Request Body:**

| {   "peer\_url":    "https://ume.hypergrid.io",   "peer\_grid\_id":"uuid",   "trust\_level": "grid",                   // grid|namespace|identity   "description": "Ume B-OS Grid" } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | Peer registered | { peer\_id, grid\_id, peer\_url, trust\_level, status: 'handshaking' } |
| 400 | Invalid peer URL | Peer URL is unreachable or does not respond to federation handshake. |
| 403 | Admin required |  |
| 409 | Already registered | This peer Grid is already registered. |
| Note: Federation registration initiates the 8-phase handshake asynchronously. Poll GET /federation/peers to track progress. Phases: Discovery → Handshake → Schema Sync → Cell Delta Sync → Conflict Surface → Namespace Sync → Heartbeat → Shadow Sync. |  |  |

| GET | /api/v1/federation/peers/{peer\_id} |
| :---: | :---- |

*Get detailed status for a specific federation peer: VectorClock lag, schema alignment, active shadow syncs.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| peer\_id | path | uuid | Yes | Federation peer identifier. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { peer\_id, status, vector\_clock\_lag, schema\_version\_local, schema\_version\_remote, unsynced\_ops\_count, active\_shadow\_syncs, last\_sync\_at } |

| POST | /api/v1/federation/sync |
| :---: | :---- |

*Trigger a manual full delta sync with all registered federation peers. Used to force synchronization after a network partition or maintenance window.*

**Request Body:**

| { "peer\_id": "uuid" } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 202 Accepted | Sync initiated | { sync\_id, status: 'running', peers\_syncing: \[uuid\] } |
| 403 | Admin required |  |

| GET | /api/v1/federation/conflicts |
| :---: | :---- |

*List unresolved CRDT conflicts that require human review. Conflicts arise when concurrent writes from different federation nodes cannot be automatically merged.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | query | uuid | No | Filter to conflicts in a specific Hypercube. |
| entity\_id | query | uuid | No | Filter to conflicts for a specific entity. |
| resolved | query | boolean | No | Include resolved conflicts. Default: false. |
| limit | query | integer | No | Default: 50\. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: \[{ conflict\_id, cube\_id, entity\_id, attr\_key, value\_a, value\_b, node\_a, node\_b, detected\_at, resolved }\] } |
| 403 | Editor required |  |

| POST | /api/v1/federation/conflicts/{conflict\_id}/resolve |
| :---: | :---- |

*Resolve a CRDT conflict by choosing a resolution strategy: keep\_local, keep\_remote, merge (JSON CRDT), or provide a custom value.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| conflict\_id | path | uuid | Yes | Conflict identifier. |

**Request Body:**

| {   "resolution":    "keep\_remote",          // keep\_local|keep\_remote|merge|custom   "custom\_value":  null,                   // required if resolution=custom   "note":          "Remote value is correct — was an accidental local edit." } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Resolved | { conflict\_id, resolved\_value, resolved\_by, resolved\_at } |
| 404 | Not found | Conflict ID not found. |

# **Part XII — Export, Snapshots, and EventLog**

## **22\. Export**

| GET | /api/v1/cubes/{cube\_id}/export |
| :---: | :---- |

*Export a Hypercube or HypercubeView in the requested format. Supports CSV, XLSX, JSON, JSON-LD, Apache Parquet, and Apache Arrow. For large cubes, an async job is returned.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| format | query | string | Yes | csv | xlsx | json | jsonld | parquet | arrow |
| view\_id | query | uuid | No | Apply a saved HypercubeView (filters, sorts, dim slices). |
| filter | query | string | No | Inline DimSlice filter in HyperQL WHERE syntax. |
| attrs | query | string | No | Comma-separated attribute keys to export. Default: all. |
| as\_of | query | datetime | No | Export historical snapshot at this timestamp. |
| include\_schema | query | boolean | No | Include cube schema in export. Default: true for json/jsonld/parquet. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Streaming download (\< 50MB) | Binary file stream with appropriate Content-Type and Content-Disposition headers. |
| 202 Accepted | Async export queued (\>= 50MB) | { export\_id, status: 'queued', estimated\_size\_mb, download\_url\_expires\_in: 3600 } |
| Note: For N\>2 cubes, CSV and XLSX exports automatically apply DimFold to reduce to 2D. Set view\_id to control which DimFolds are applied. JSON and Parquet preserve the full N-dimensional structure. |  |  |

| GET | /api/v1/export/{export\_id} |
| :---: | :---- |

*Poll the status of an async export job and get the download URL when complete.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| export\_id | path | uuid | Yes | Export job identifier. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Pending | { export\_id, status: 'running', progress\_pct } |
| 200 OK | Complete | { export\_id, status: 'complete', download\_url, size\_bytes, expires\_at } |
| 200 OK | Failed | { export\_id, status: 'failed', error } |
| Note: Download URLs are pre-signed S3 URLs valid for 1 hour. The export file is retained for 24 hours after generation. |  |  |

## **23\. Snapshots**

| POST | /api/v1/cubes/{cube\_id}/snapshot |
| :---: | :---- |

*Create a named point-in-time snapshot of a Hypercube. Snapshots enable efficient AS\_OF time-travel queries and disaster recovery.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |

**Request Body:**

| {   "label":       "before-q2-migration",    // human-readable label for this snapshot   "description": "Pre-migration snapshot before Q2 2026 data import",   "retain\_until":"2026-12-31T23:59:59Z"   // null \= use default retention (90 days) } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | Snapshot queued | { snapshot\_id, cube\_id, label, status: 'queued', estimated\_size\_mb } |
| 403 | Owner required |  |
| Note: Snapshot creation is asynchronous. Large cubes may take several minutes. Poll GET /cubes/{id}/snapshots/{snapshot\_id} for status. |  |  |

| GET | /api/v1/cubes/{cube\_id}/snapshots |
| :---: | :---- |

*List all snapshots for a Hypercube.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| status | query | string | No | complete|pending|failed — filter by status. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: \[{ snapshot\_id, label, status, size\_bytes, created\_at, retain\_until }\] } |

| POST | /api/v1/cubes/{cube\_id}/restore/{snapshot\_id} |
| :---: | :---- |

*Restore a Hypercube from a snapshot. All cell data is rolled back to the snapshot state. EventLog is preserved — the restore itself is recorded as a RestoreEvent.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| snapshot\_id | path | uuid | Yes | Snapshot to restore from. |

**Request Body:**

| {   "dry\_run": true,                         // simulate without executing (default: true)   "confirm": "RESTORE"                     // required when dry\_run=false: type 'RESTORE' } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Dry run result | { rows\_affected, attrs\_rolled\_back, estimated\_duration\_ms } |
| 202 Accepted | Restore initiated | { restore\_id, status: 'running', estimated\_completion\_at } |
| 400 | Confirmation required | dry\_run=false requires confirm='RESTORE'. |
| 403 | Owner required |  |
| 404 | Snapshot not found | Snapshot expired or ID not found. |
| Note: Restoration creates a checkpoint of the current state before rolling back (restorable within 7 days). The EventLog records the full restore as a single RestoreEvent, preserving the complete audit trail of all mutations since the snapshot. |  |  |

## **24\. EventLog**

| GET | /api/v1/cubes/{cube\_id}/eventlog |
| :---: | :---- |

*Query the EventLog for a Hypercube. All mutations, schema changes, AI writebacks, governance actions, and CRDT conflicts are recorded here. Immutable — no entry can be modified or deleted.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| entity\_id | query | uuid | No | Filter to events for a specific HyperRow (D₁ key). |
| attr\_key | query | string | No | Filter to events affecting a specific attribute key. |
| event\_type | query | string | No | Filter by EventEntry type. Comma-separated. |
| actor | query | string | No | Filter by actor ID (user, AI engine, federation node). |
| since | query | datetime | No | Events after this timestamp. |
| until | query | datetime | No | Events before this timestamp. |
| limit | query | integer | No | Default: 100\. Max: 1000\. |
| cursor | query | string | No | Pagination cursor. EventLog is paginated newest-first. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: EventLogEntry\[\], meta: PaginationMeta } |
| 403 | Editor required |  |

### **24.1 EventLogEntry Object Schema**

| {   "entry\_id":     "uuid",   "cube\_id":      "uuid",   "entity\_id":    "uuid",                 // D₁ key of affected entity   "attr\_key":     "status",               // affected attribute key   "event\_type":   "SetAttr",              // SetAttr|AddToSet|RemoveFromSet|IncrCounter|                                           //   TransitionState|AppendLog|SchemaChange|                                           //   AIColumnUpdated|AnomalyDetected|Restored|                                           //   ShadowSync|FederationSync|GovernanceAction   "before\_value": { "value": "Draft", "type": "Enum" },   "after\_value":  { "value": "Active", "type": "Enum" },   "delta":        { "op": "SetAttr", "attr\_key": "status", "value": "Active", ... },   "vector\_clock": { "node-us-east-1": 42 },   "actor":        "user-uuid:@alice-dev:node-us-east-1",   "identity\_tag": "@alice-dev",   "space\_id":     "uuid",   "workspace\_id": "uuid",   "created\_at":   "2026-03-22T10:00:00Z",   "is\_conflict":  false } |
| :---- |

# **Part XIII — WebSocket and Streaming API**

## **25\. WebSocket Channels**

Hypergrid's WebSocket API provides real-time cell mutation streaming, collaborative editing, Space activity feeds, shadow cell synchronization, and CRDT federation gossip. All WebSocket connections use WSS (WebSocket over TLS). Authentication is via session\_token query parameter (returned by POST /workspaces/{id}/session).

### **25.1 Cube Stream — Real-Time Cell Mutations**

| Attribute | Value |
| :---- | :---- |
| URL | wss://{grid\_host}/ws/v1/cubes/{cube\_id}/stream |
| Auth | ?session\_token={token} or ?api\_key={key} |
| Protocol | JSON messages over WSS |
| Message direction | Server → Client (subscribe) and Client → Server (subscribe/unsubscribe) |

| // Subscribe to all cells in a cube: // Client → Server { "op": "subscribe", "scope": "cube", "cube\_id": "uuid" }   // Subscribe to a specific HyperRow: { "op": "subscribe", "scope": "row", "cube\_id": "uuid", "d1\_key": "uuid" }   // Subscribe to a specific attribute key across all rows: { "op": "subscribe", "scope": "attr", "cube\_id": "uuid", "attr\_key": "health\_score" }   // Subscribe to a DimSlice: { "op": "subscribe", "scope": "slice", "cube\_id": "uuid",   "filter": "D2='status' AND value='Active'" }   // Unsubscribe: { "op": "unsubscribe", "subscription\_id": "sub-uuid" }   // Heartbeat (client must send every 30s to keep connection alive): { "op": "ping" } // Server response: { "op": "pong", "server\_ts": "2026-03-22T10:00:00Z" }   // ── Incoming events (Server → Client) ──────────────────────────────────── // Cell updated: {   "event": "cell\_updated",   "subscription\_id": "sub-uuid",   "cube\_id":   "uuid",   "d1\_key":    "uuid",   "attr\_key":  "health\_score",   "new\_value": { "value": 87.4, "type": "AI", "confidence": 0.92 },   "vector\_clock": { "node-us-east-1": 43 },   "actor":     "ai::kogi.health\_score\_engine.v2::v2.1",   "ts":        "2026-03-22T10:00:01Z" }   // Row created: { "event": "row\_created", "cube\_id": "uuid", "d1\_key": "uuid", "ts": "..." }   // Row archived: { "event": "row\_archived", "cube\_id": "uuid", "d1\_key": "uuid", "ts": "..." }   // Schema event (attr key added): { "event": "attr\_registered", "cube\_id": "uuid", "attr\_key": "new\_field", "ts": "..." }   // AI writeback: { "event": "ai\_writeback", "cube\_id": "uuid", "d1\_key": "uuid",   "engine": "kogi.health\_score\_engine.v2", "attr\_key": "health\_score",   "value": 91.2, "confidence": 0.95, "ts": "..." }   // CRDT conflict: { "event": "conflict\_detected", "cube\_id": "uuid", "d1\_key": "uuid",   "attr\_key": "status", "conflict\_id": "uuid", "ts": "..." } |
| :---- |

### **25.2 Workspace Session — Collaborative Editing**

| Attribute | Value |
| :---- | :---- |
| URL | wss://{grid\_host}/ws/v1/workspaces/{workspace\_id}/session |
| Auth | ?session\_token={token} (from POST /workspaces/{id}/session) |
| Purpose | Real-time collaborative editing: cursor presence, cell locking, CRDT synchronization, Oba AI annotations |

| // ── Client → Server ───────────────────────────────────────────────────── // Move cursor: { "op": "cursor\_move", "cube\_id": "uuid", "coord": "d1\_key,attr\_key", "ts": "..." }   // Lock a cell for editing: { "op": "cell\_lock", "cube\_id": "uuid", "coord": "d1\_key,attr\_key" }   // Unlock a cell: { "op": "cell\_unlock", "cube\_id": "uuid", "coord": "d1\_key,attr\_key" }   // Write a cell (sends CrdtOperation to server): { "op": "cell\_write", "cube\_id": "uuid", "coord": "...", "attr\_key": "name",   "value": "New Name", "crdt\_op": "SetAttr", "client\_ts": "..." }   // Set workspace DimContext: { "op": "set\_dim\_context", "dim\_context": { "time\_axis\_id": "2026-Q2" } }   // ── Server → Client ───────────────────────────────────────────────────── // Presence: another user joined: { "event": "presence\_joined", "user\_id": "uuid", "identity\_tag": "@bob-dev",   "cursor": { "cube\_id": "uuid", "coord": "..." }, "ts": "..." }   // Presence: user left: { "event": "presence\_left", "user\_id": "uuid", "ts": "..." }   // Cell locked by another user: { "event": "cell\_locked", "cube\_id": "uuid", "coord": "...",   "locked\_by": "uuid", "locked\_by\_name": "Bob", "ts": "..." }   // Cell unlocked: { "event": "cell\_unlocked", "cube\_id": "uuid", "coord": "...", "ts": "..." }   // DimContext changed by another user (Shared Workspace): { "event": "dim\_context\_changed", "dim\_context": { "time\_axis\_id": "2026-Q3" },   "changed\_by": "uuid", "ts": "..." }   // Oba AI hint for a row: { "event": "oba\_hint", "entity\_id": "uuid", "hint": {     "hint\_id":    "uuid",     "type":       "anomaly\_warning",     "message":    "Budget utilization at 94% — 3 days until quarter end.",     "severity":   0.8,     "expires\_at": "..."   }, "ts": "..." }   // Write conflict notification: { "event": "conflict\_detected", "conflict\_id": "uuid", "cube\_id": "uuid",   "coord": "...", "your\_value": "A", "winning\_value": "B", "ts": "..." }   // Sync complete acknowledgment (your write was committed): { "event": "sync\_complete", "client\_ts": "...", "server\_ts": "...", "version": 55 } |
| :---- |

### **25.3 Space Activity Stream**

| Attribute | Value |
| :---- | :---- |
| URL | wss://{grid\_host}/ws/v1/spaces/{space\_id}/activity |
| Auth | ?session\_token={token} |
| Purpose | Space-level activity feed: member events, governance actions, treasury events, graph edge changes, shadow cell updates |

| // Events emitted on this channel: { "event": "member\_joined",     "user\_id": "uuid", "role": "Contributor", "ts": "..." } { "event": "member\_left",       "user\_id": "uuid", "ts": "..." } { "event": "member\_role\_changed","user\_id": "uuid", "old\_role": "Contributor", "new\_role": "Manager", "ts": "..." } { "event": "proposal\_created",  "proposal\_id": "uuid", "title": "...", "ts": "..." } { "event": "vote\_submitted",    "proposal\_id": "uuid", "current\_tally": {...}, "ts": "..." } { "event": "proposal\_resolved", "proposal\_id": "uuid", "outcome": "passed", "ts": "..." } { "event": "edge\_created",      "edge\_id": "uuid", "edge\_type": "CrossGridLink", "ts": "..." } { "event": "edge\_consent\_accepted","edge\_id": "uuid", "ts": "..." } { "event": "shadow\_cell\_updated","shadow\_id": "uuid", "attr\_key": "...", "ts": "..." } { "event": "treasury\_event",    "event\_type": "distribution", "amount": {...}, "ts": "..." } |
| :---- |

### **25.4 Shadow Sync Stream**

| Attribute | Value |
| :---- | :---- |
| URL | wss://{grid\_host}/ws/v1/graph/shadow-sync |
| Auth | ?session\_token={token} |
| Purpose | Real-time MirrorAttribute delta stream for all ShadowCells in this Grid |

| // Events emitted: { "event": "shadow\_attr\_updated", "shadow\_id": "uuid", "source\_grid": "uuid",   "attr\_key": "status", "new\_value": {...}, "ts": "..." } { "event": "shadow\_connected",    "shadow\_id": "uuid", "edge\_id": "uuid", "ts": "..." } { "event": "shadow\_disconnected", "shadow\_id": "uuid", "reason": "link\_revoked", "ts": "..." } { "event": "shadow\_degraded",     "shadow\_id": "uuid", "reason": "source\_timeout", "ts": "..." } { "event": "shadow\_reconnected",  "shadow\_id": "uuid", "ts": "..." } |
| :---- |

### **25.5 CRDT Federation Sync Stream**

| Attribute | Value |
| :---- | :---- |
| URL | wss://{grid\_host}/ws/v1/crdt/federation/{peer\_id} |
| Auth | mTLS (internal service mesh only) |
| Purpose | Real-time CRDT gossip stream between two Grid federation nodes |

| // Messages (bidirectional): // Batch of CrdtOperations from peer: { "op": "crdt\_op\_batch", "peer\_id": "uuid", "batch\_id": "uuid",   "ops": \[ CrdtOperation, ... \],   "vector\_clock": { "node-id": 42 }, "ts": "..." }   // Acknowledge receipt of a batch: { "op": "ack", "batch\_id": "uuid" }   // VectorClock heartbeat exchange: { "op": "heartbeat", "vector\_clock": { "node-id": 42 }, "ts": "..." }   // Schema delta (new dimension or attr key): { "op": "schema\_delta", "cube\_id": "uuid",   "events": \[ SchemaEvent, ... \], "ts": "..." }   // Conflict surfaced during apply: { "op": "conflict\_surfaced", "conflict": ConflictRecord, "ts": "..." } |
| :---- |

# **Part XIV — Server-Sent Events (SSE)**

## **26\. SSE Endpoints**

Server-Sent Events provide a lightweight, one-directional streaming alternative to WebSocket. Use SSE for read-only monitoring, dashboard feeds, and integration with systems that consume SSE natively (e.g. browser EventSource API, some serverless environments).

| GET | /api/v1/cubes/{cube\_id}/events |
| :---: | :---- |

*Subscribe to a real-time SSE stream of cell mutations for a Hypercube.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| entity\_id | query | uuid | No | Filter to a specific HyperRow. |
| attr\_key | query | string | No | Filter to a specific attribute key. |
| event\_types | query | string | No | Comma-separated event types to receive. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Streaming | Content-Type: text/event-stream. SSE event stream. |
| 403 | Insufficient scope |  |
| Note: SSE connections are kept alive with comment heartbeats every 30s. The Last-Event-ID header is supported for reconnection — the server will replay missed events from the specified event ID. |  |  |

| GET | /api/v1/spaces/{space\_id}/events |
| :---: | :---- |

*Subscribe to a real-time SSE stream of Space activity events.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| space\_id | path | uuid | Yes | Space identifier. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Streaming | Content-Type: text/event-stream. |

| // SSE event format: id: evt-uuid-123 event: cell\_updated data: {"cube\_id":"uuid","d1\_key":"uuid","attr\_key":"health\_score","new\_value":87.4,"ts":"..."}   id: evt-uuid-124 event: row\_created data: {"cube\_id":"uuid","d1\_key":"new-entity-uuid","ts":"..."}   : heartbeat 2026-03-22T10:00:30Z   // Reconnection: client sends Last-Event-ID header \= "evt-uuid-124" // Server replays all events since that ID (up to 1000 events, 24h max window) |
| :---- |

# **Part XV — HypercubeView API**

## **27\. Views**

| GET | /api/v1/cubes/{cube\_id}/views |
| :---: | :---- |

*List all HypercubeViews for a cube. Views with visibility=Public or SpaceShared are visible to all; Private views only to their creator.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| visibility | query | string | No | Private|Shared|SpaceShared|Public|Template — filter. |
| render\_mode | query | string | No | Filter by render mode. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { data: HypercubeView\[\], meta: PaginationMeta } |

| POST | /api/v1/cubes/{cube\_id}/views |
| :---: | :---- |

*Create a new HypercubeView.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |

**Request Body:**

| {   "name":           "Q2 Active Projects by Health",   "primary\_axis":   "entity\_axis\_id",        // D₁ → rows   "secondary\_axis": "property\_axis\_id",       // D₂ → columns   "dim\_slices": \[                            // filter on D₃+ axes     { "axis\_id": "time\_axis\_id", "op": "eq", "value": "2026-Q2" }   \],   "dim\_folds":  \[\],   "dim\_expands":\[\],   "attr\_schema": \[                           // column order and display     { "attr\_key": "name",         "width": 300, "pinned": true },     { "attr\_key": "status",       "width": 120 },     { "attr\_key": "health\_score", "width": 100 },     { "attr\_key": "budget\_remaining","width": 120 }   \],   "filters": \[     { "attr\_key": "status",       "op": "eq", "value": "Active" },     { "attr\_key": "item\_type",    "op": "eq", "value": "Project" }   \],   "sorts": \[     { "attr\_key": "health\_score", "direction": "asc" }   \],   "groups": \[\],   "render\_mode": "Grid2D",                  // Grid2D|Kanban|Gantt|Calendar|Timeline|                                             //   HierarchyTree|Treemap|NetworkGraph|                                             //   LinkForest|PivotTable|HeatMap|Matrix|                                             //   N-DimExplorer|Custom(plugin\_id)   "highlight\_rules": \[     { "condition": "health\_score \< 50", "color": "\#FEE2E2", "label": "At Risk" }   \],   "shadow\_policy":  "Include",              // Include|Exclude|ShadowOnly   "graph\_overlay":  false,   "ai\_overlay":     true,   "visibility":     "SpaceShared" } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 201 Created | View created | HypercubeView object |
| 400 | Invalid render mode | Render mode not compatible with specified axes. |
| 403 | Insufficient tier | Requires Editor+ tier. |

| GET | /api/v1/cubes/{cube\_id}/views/{view\_id}/rows |
| :---: | :---- |

*Render rows for a HypercubeView with all view configuration applied: DimSlices, DimFolds, DimExpands, filters, sorts, groups, shadow policy, and highlight rules.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| view\_id | path | uuid | Yes | HypercubeView identifier. |
| identity\_tag | query | string | No | Override identity partition for this render. |
| as\_of | query | datetime | No | Time-travel render. |
| limit | query | integer | No | Default: 50\. Max: 500\. |
| cursor | query | string | No | Pagination cursor. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Success | { view\_id, columns: \[{ attr\_key, display\_name, width, pinned }\], rows: \[{ d1\_key, attrs, highlight\_color?, is\_shadow?, group\_label? }\], groups?: \[...\], summary\_row?: {...}, meta: PaginationMeta } |
| 404 | View not found |  |
| Note: Group headers are included as special rows with group\_label and aggregated values. The summary\_row (if configured) appears as the first row. ShadowRows are marked with is\_shadow=true and the source grid/identity. |  |  |

| PATCH | /api/v1/cubes/{cube\_id}/views/{view\_id} |
| :---: | :---- |

*Update a HypercubeView configuration.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| view\_id | path | uuid | Yes | HypercubeView identifier. |

**Request Body:**

| {   "name":       "Q2 Active Projects by Risk",   "sorts":      \[{ "attr\_key": "risk\_score", "direction": "desc" }\],   "visibility": "Public" } |
| :---- |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 200 OK | Updated | Updated HypercubeView |
| 403 | Insufficient tier | Requires Editor+ tier. Updating visibility to Public requires Manager+. |

| DELETE | /api/v1/cubes/{cube\_id}/views/{view\_id} |
| :---: | :---- |

*Delete a HypercubeView. Does not affect any cell data.*

**Parameters:**

| Name | In | Type | Required | Description |
| :---- | :---- | :---- | :---- | :---- |
| cube\_id | path | uuid | Yes | Hypercube identifier. |
| view\_id | path | uuid | Yes | HypercubeView identifier. |

**Responses:**

| Status | Meaning | Response Body |
| :---- | :---- | :---- |
| 204 No Content | Deleted | No body. |
| 403 | Insufficient tier | Requires Owner of this view. |

# **Part XVI — Rate Limits, Resilience Patterns, and SDK Guide**

## **28\. Rate Limits**

| Endpoint Group | Default Limit | Burst | Headers |
| :---- | :---- | :---- | :---- |
| All REST endpoints | 1,000 req/min per identity | 200 req/10s | X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset |
| POST /cubes/{id}/rows | 500 req/min per identity | 100 req/10s | Same headers |
| PATCH /cubes/{id}/rows/{id} | 500 req/min per identity | 100 req/10s | Same headers |
| POST /cubes/{id}/query | 200 req/min per identity | 50 req/10s | Same headers |
| POST /query/async | 50 req/min per identity | 10 req/10s | Same headers |
| POST /query/nl | 60 req/min per identity | 15 req/10s | Also: X-AI-Credit-Remaining |
| POST /ai/compute | 200 req/min per identity | 50 req/10s | Also: X-AI-Credit-Remaining |
| POST /graph/edges | 200 req/min per identity | 50 req/10s | Same headers |
| WebSocket connections | 10 concurrent per identity | — | Connection rejected with 429 if limit exceeded |
| Federation sync endpoints | No limit (internal) | — | mTLS only — not accessible externally |

| Note: Rate limits can be increased via plan upgrades. Contact support for enterprise rate limit customization. WritebackService (AI engine writeback) is excluded from per-identity rate limits — it uses per-engine quotas. |
| :---- |

## **29\. Resilience Patterns for API Consumers**

| Pattern | When to Apply | Implementation Guidance |
| :---- | :---- | :---- |
| Idempotency Keys | All POST/PUT/PATCH requests that create or mutate data | Generate a UUID v4 per operation attempt. Set X-Idempotency-Key header. On network failure, retry with the same key. The server stores results for 24 hours — duplicate requests return the same response. |
| Exponential Backoff | 429 (rate limit), 503 (unavailable), 502 (upstream error) | Initial delay: 1s. Factor: 2\. Max delay: 60s. Add ±10% jitter. Do NOT retry on 400, 401, 403, 404, 409\. |
| Optimistic Concurrency | PATCH operations on frequently-updated rows | Include If-Match header with current ETag from GET response. On 409 version\_conflict: re-fetch, re-apply your change, retry PATCH with new ETag. |
| Read-Your-Writes | After a write, if you immediately read the same entity | The server guarantees read-your-writes for 5 seconds by routing to the same PostgreSQL primary. For guaranteed consistency: include X-After-Write: {vector\_clock} header on subsequent reads. |
| Circuit Breaking | Calling Hypergrid from a service with SLA dependencies | Trip circuit at 50% error rate over 10s. Half-open state: allow 1 probe request every 30s. On trip, serve cached or degraded response. |
| Streaming Large Queries | Any query expected to return \>10,000 rows | Use POST /query/async and stream results via GET /query/async/{id}/stream. Set Accept: application/x-ndjson for line-by-line processing. |
| Webhook Delivery | Receiving real-time updates in server-side integrations | Register webhooks via POST /webhooks. The server will POST events with HMAC-SHA256 signatures. Validate X-Hypergrid-Signature header on receipt. Respond with 200 within 5s. Failed webhooks retried 5 times with exponential backoff. |
| Cache Key Strategy | Client-side caching of HyperRow data | Cache key: {cube\_id}:{d1\_key}:{attrs\_hash}. Invalidate on: ETag change (compare X-Entity-Version header), WebSocket cell\_updated event, or TTL expiry. Recommended TTL: 60s for Tier-2 AI attrs, 5s for LWW scalar attrs. |

## **30\. SDK Quick Reference**

### **30.1 TypeScript / JavaScript SDK**

| import { HypergridClient, HyperQL } from '@hypergrid/sdk';   const client \= new HypergridClient({   baseUrl:   'https://kogi.hypergrid.io/api/v1',   apiKey:    process.env.HYPERGRID\_API\_KEY,   gridId:    'kogi-production-grid-id', });   // List HyperRows with DimSlice filter const rows \= await client.cubes('kogi.portfolio.components').rows.list({   filter:   "D2='status' AND value='Active' AND D2='health\_score' AND value \< 70",   attrs:    \['name', 'status', 'health\_score', 'budget\_remaining'\],   sort:     'health\_score:asc',   limit:    50, });   // Read a single HyperRow const component \= await client.cubes('kogi.portfolio.components').rows.get(entityId);   // Write attributes (CRDT-safe) await client.cubes('kogi.portfolio.components').rows.update(entityId, {   attributes: {     name:   'Renamed Project',     status: { from: 'Active', to: 'Deprecated' },    // Lattice transition     tags:   { add: \['q2-2026'\], remove: \['q1-2026'\] } // OR-Set   } });   // Execute HyperQL const result \= await client.query(   HyperQL.select('D1.entity\_id', 'name', 'health\_score')     .from('kogi.portfolio.components')     .where("status \= 'Active' AND health\_score \< 70")     .orderBy('health\_score', 'asc')     .limit(20) );   // Subscribe to real-time updates (WebSocket) const sub \= client.cubes('kogi.portfolio.components').subscribe({   scope: 'attr', attrKey: 'health\_score',   onUpdate: (event) \=\> { console.log('Health score updated:', event); } });   // Natural language query const nlResult \= await client.ai.nlQuery({   question:    'Which projects are over budget this quarter?',   cubeContext: \['kogi.portfolio.components'\], }); console.log(nlResult.generatedHyperql); // transparency: show generated HyperQL |
| :---- |

### **30.2 Python SDK**

| from hypergrid import HypergridClient, HyperQL, DimSlice   client \= HypergridClient(     base\_url="https://kogi.hypergrid.io/api/v1",     api\_key=os.environ\["HYPERGRID\_API\_KEY"\],     grid\_id="kogi-production-grid-id", )   \# List HyperRows rows \= client.cubes\["kogi.portfolio.components"\].rows.list(     filter=DimSlice.where("status").eq("Active").and\_("health\_score").lt(70),     attrs=\["name", "status", "health\_score"\],     limit=50, )   \# Create a HyperRow new\_row \= client.cubes\["kogi.portfolio.components"\].rows.create({     "name": "New Project",     "status": "Draft",     "tags": \["q2-2026", "engineering"\],     "budget": {"amount": "10000.00", "currency": "USD"}, })   \# Execute async HyperQL job \= client.query.async\_execute(     "SELECT D1.entity\_id, FOLD D3 WITH SUM(value) AS total "     "FROM kogi.portfolio.finances "     "WHERE D2.metric\_name \= 'revenue' "     "GROUP BY D1.entity\_id ORDER BY total DESC",     output\_format="parquet" ) \# Stream results as Parquet with open("results.parquet", "wb") as f:     for chunk in job.stream():         f.write(chunk)   \# Graph traversal tree \= client.graph.nodes\[node\_id\].tree(max\_depth=3, edge\_type="Hierarchy") print(f"Found {len(tree.nodes)} nodes in hierarchy")   \# Time-travel query historical \= client.cubes\["kogi.portfolio.components"\].rows.list(     as\_of="2026-01-01T00:00:00Z",     filter=DimSlice.where("status").eq("Active"), ) |
| :---- |

### **30.3 Rust SDK**

| use hypergrid\_sdk::{HypergridClient, HyperQL, DimCoordinate, CrdtOp};   let client \= HypergridClient::builder()     .base\_url("https://kogi.hypergrid.io/api/v1")     .api\_key(std::env::var("HYPERGRID\_API\_KEY")?)     .build()?;   // Get a specific HyperCell (N-dim coordinate) let coord \= DimCoordinate::new(vec\!\[     DimKey::Uuid(entity\_id),     DimKey::Text("health\_score".into()),     DimKey::Text("2026-Q2".into()),    // D3=TimeAxis     DimKey::Text("EMEA".into()),       // D4=GeoAxis \]); let cell \= client.cubes("qala.solutions.metrics").cells().get(\&coord).await?;   // Write with CRDT operation client.cubes("kogi.portfolio.components")     .cells()     .patch\_attr(\&entity\_coord, "status", CrdtOp::TransitionState {         from: "Active".into(),         to:   "Deprecated".into(),     })     .await?;   // Subscribe to real-time updates (async stream) let mut stream \= client     .cubes("kogi.portfolio.components")     .rows()     .subscribe\_row(entity\_id)     .await?;   while let Some(event) \= stream.next().await {     match event {         CellEvent::Updated { attr\_key, new\_value, .. } \=\> {             println\!("Attr '{}' updated to {:?}", attr\_key, new\_value);         }         CellEvent::AiWriteback { engine, attr\_key, value, confidence } \=\> {             println\!("AI engine '{}' updated '{}'={:?} (conf={})",                 engine, attr\_key, value, confidence);         }         \_ \=\> {}     } } |
| :---- |

# **Part XVII — Performance Targets and Appendices**

## **31\. API Performance Targets**

| Operation | Target p50 | Target p99 | Notes |
| :---- | :---- | :---- | :---- |
| GET /cubes/{id}/rows (50 rows) | \< 30ms | \< 150ms | Single cube, 2D dense, no fold. Includes computed Tier-1 attrs. Redis cache serves AI attrs. |
| GET /cubes/{id}/rows (500 rows) | \< 100ms | \< 400ms | Larger page size with sort. Index-assisted sort on scored columns. |
| GET /cubes/{id}/rows/{d1\_key} | \< 5ms | \< 20ms | Single HyperRow primary key lookup. Redis serves AI attr cache. |
| PATCH /cubes/{id}/rows/{d1\_key} | \< 10ms | \< 50ms | Single entity, 1–5 attr updates. CrdtLog Redis write \+ async PG flush. |
| POST /cubes/{id}/query (small, \< 1K rows) | \< 100ms | \< 500ms | Simple DimSlice query, no fold. PostgreSQL index-assisted. |
| POST /cubes/{id}/fold (1M rows) | \< 2s | \< 8s | Single DimFold on N=3 cube. ClickHouse backend for \> 100K rows. |
| POST /query/nl (NL → HyperQL \+ execute) | \< 2s | \< 8s | NL translation \+ HyperQL execution. Depends on AI engine latency. |
| GET /ns/resolve (Redis hit) | \< 2ms | \< 8ms | Redis-cached namespace resolution. |
| GET /ns/resolve (PG fallback) | \< 10ms | \< 40ms | Cache miss — PostgreSQL index lookup. |
| POST /graph/edges (no consent) | \< 15ms | \< 60ms | Edge creation without consent flow. |
| GET /graph/nodes/{id}/tree (depth=3) | \< 100ms | \< 500ms | BFS on PostgreSQL adjacency list. Cached for hot nodes. |
| GET /graph/path (6 hops) | \< 200ms | \< 800ms | Dijkstra on PG with adjacency index. |
| POST /cubes/{id}/snapshot | \< 5s | \< 30s | Snapshot large cube. Async — 202 returned immediately. |
| POST /cubes/{id}/restore/{snap\_id} | \< 30s | \< 5min | Full cube restore. Async — 202 returned immediately. |
| WebSocket cell\_updated broadcast (100 clients) | \< 30ms | \< 150ms | Redis pub/sub fanout. Measured end-to-end from write commit. |
| Shadow sync delta (MirrorAttribute) | \< 500ms | \< 2s | Kafka event → consumer → PG write → WebSocket push. |

## **32\. Appendix A — AttributeType Reference**

| AttributeType | JSON Representation | Default CRDT | Notes |
| :---- | :---- | :---- | :---- |
| Text | string | LWW | UTF-8 string. Max 65,535 bytes. |
| Number | number (float64) | LWW | IEEE 754 double. Use Currency for monetary values. |
| Integer | number (integer) | LWW | 64-bit signed integer. |
| Currency | { amount: string, currency: string } | LWW | Decimal amount string (no float imprecision). ISO 4217 currency code. |
| Percent | number (0.0–100.0) | LWW | Stored as float. Rendered as percentage. |
| Bool | boolean | LWW | true | false. |
| Date | string (YYYY-MM-DD) | LWW | ISO 8601 date only. No timezone. |
| DateTime | string (ISO 8601\) | LWW | UTC timestamp. Stored with nanosecond precision. |
| Duration | number (seconds) | LWW | Duration in seconds. Rendered as human-readable (2h 30m). |
| Enum | string | LWW | One of a declared variant set. Validated at write time. |
| MultiEnum | string\[\] | OR-Set | Multiple enum variants. OR-Set: concurrent adds survive. |
| Relation | { cube\_id: uuid, d1\_key: uuid } | LWW | Reference to one HyperRow in any cube. |
| MultiRelation | { cube\_id, d1\_key }\[\] | OR-Set | References to multiple HyperRows. |
| User | uuid (identity\_id) | LWW | Reference to an identity in the Identity system. |
| Tag | string | LWW | Single tag string. |
| TagSet | string\[\] | OR-Set | Set of tags. OR-Set: concurrent tag additions both survive. |
| Json | any JSON value | LWW (or JsonCRDT) | Arbitrary JSON. Use JsonCRDT semantics for deep-merge behavior. |
| Computed | matches output attr\_type | LWW (system) | Tier-1 formula result. System-write on read evaluation. |
| AI | matches output attr\_type | LWW (system) | Tier-2 AI signal. Written by AIEngineAdapter via WritebackService. |
| Formula | { expression: string, result: any } | LWW (system) | Stored formula expression \+ last computed result. |
| Audit | { actor, action, ts } | AppendOnlyLog | Append-only audit record. System-managed. |
| Custom | any JSON | Plugin-defined | Registered via HypercubePlugin AttributeTypePlugin interface. |

## **33\. Appendix B — EdgeType Quick Reference**

| EdgeType | Consent? | Shadow? | Cycle Detection? | CRDT | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| Hierarchy | No | No | Yes | OR-Set (edges) | Parent-child. Enables ROLLUP/DRILLDOWN. Max depth enforced by configured limit. |
| Dependency | No | Optional | Yes | OR-Set (edges) | Blocking. from\_node cannot complete before to\_node. |
| Association | No | No | No | OR-Set (edges) | Soft lateral reference. No blocking semantics. |
| Contains | No | No | No | OR-Set (edges) | Ownership containment. |
| Derives | No | Optional | No | OR-Set (edges) | Versioning / fork relationship. |
| CrossGridLink | Yes | Yes | No | Consent: LWW | Inter-Grid connection. Core link network atom. |
| ShadowOf | System | N/A | No | System | System edge. Inverse of CrossGridLink. Auto-created on consent. |
| SpaceMembership | SpaceType | No | No | OR-Set (members) | Identity → Space. Role on edge attrs. |
| Collaborates | Mutual | Yes | No | Consent: LWW | Bidirectional. Both parties consent. |
| References | No | No | No | OR-Set (edges) | Citation/reference. Lighter than Dependency. |
| FederationPeer | Grid-trust | System | No | System | Grid-to-Grid CRDT federation link. |
| NamespaceAlias | No | No | No | LWW | Namespace path alias/redirect. |
| ComputedFrom | No | No | No | System | Provenance / lineage edge. |
| SubscribesTo | No | Optional | No | OR-Set (subs) | Change notification subscription. |
| InvestedIn | Yes | Yes | No | Consent: LWW | Economic investment. Platform-specific semantics. |
| Employs | Yes | Yes | No | Consent: LWW | Employment/contracting. Worker consents. |
| Custom(String) | Plugin | Plugin | Plugin | Plugin | Custom edge type registered via HG-PLUGIN EdgeTypePlugin. |

## **34\. Appendix C — CRDT Semantics Quick Reference**

| CrdtSemantics | Merge Rule | REST write body | EventLog Op | Suitable For |
| :---- | :---- | :---- | :---- | :---- |
| LastWriteWins | Higher VectorClock timestamp wins. Tiebreak: ActorId lex sort. | { value: ... } | SetAttr | All scalar fields: Text, Number, Enum, Bool, Date, Json, references |
| OR-Set | All concurrent Adds survive. Tagged Removes only remove their exact entry. | { add: \[...\], remove: \[...\] } | AddToSet / RemoveFromSet | TagSet, MultiEnum, MultiRelation, owner lists, child ID sets |
| GrowOnlyCounter | Sum of all increments. Never decrements. | { incr: N }  (N must be \> 0\) | IncrCounter | view\_count, like\_count, follower\_count, restart\_count |
| PNCounter | Sum of (pos\_ops − neg\_ops) per node. | { incr: N } (N any sign) | IncrCounter | budget\_spent, hours\_logged (correctable accumulators) |
| MaxRegister | max(value\_a, value\_b). | { value: N } | SetAttr | version numbers, sequence numbers, generation counters |
| MinRegister | min(value\_a, value\_b). | { value: N } | SetAttr | earliest\_deadline, creation timestamps |
| Lattice | join(state\_a, state\_b) per configured partial order. No backward transitions. | { from: state, to: state } | TransitionState | lifecycle\_state, ComponentStatus, SolutionLifecycle, CCR status |
| AppendOnlyLog | All appends survive. Causally ordered by VectorClock. | { append: value } | AppendLog | Comment threads, change\_history, governance\_audit |
| JsonCrdt | Deep-merge: recursive LWW per leaf JSON key. | { deep\_merge: { key: value } } | DeepMergeJson | JSON configuration blobs requiring partial-key updates |
| Custom | Plugin provides merge(a, b, ctx) → value. | Plugin-defined | Plugin-defined | Domain-pack fields, third-party attribute types |

## **35\. Appendix D — PermissionTier Reference**

| Tier | Integer | Label | Can Write | Kogi | Ume | Qala |
| :---- | :---- | :---- | :---- | :---- | :---- | :---- |
| Public | 0 | Public/anonymous | Only public-visibility attrs on read-only endpoints | Anonymous read | Anonymous read | Public solution browsing |
| Viewer | 1 | Viewer | No write access | Follow/watch portfolio | Employee read-only | SDE observer |
| Subscriber | 1 | Subscriber | No write access | Subscribe to updates | Subscribe to alerts | Release notifications |
| Member | 2 | Member | Own standard fields | Portfolio collaborator | Org member | SDE team member |
| Contributor | 3 | Contributor | Content fields on assigned rows | Portfolio contributor | Record submitter | Junior developer |
| Editor | 4 | Editor | All content \+ config \+ timeline \+ budget fields | Portfolio editor | Domain record editor | Developer with SDE access |
| Manager | 5 | Manager | All Editor \+ governance \+ member roster \+ Space settings | Portfolio manager | Department head | Tech Lead / Release Manager |
| Owner | 6 | Owner | All fields including governance records \+ archive | Full portfolio ownership | System Administrator | Factory Admin |
| Admin | 7 | Admin (platform) | All fields \+ platform override | Platform Admin | Ume Kernel Admin | Qala Platform Admin |
| System | 8 | System (internal) | AI-computed attrs \+ federation sync \+ system metadata | AI engine / sync agent | Kernel services | AI Agent / CI pipeline |

**End of Document**

*Hypergrid API Specification v1.0  ·  March 2026  ·  Confidential — Internal Use Only*

*N-Dimensional Distributed Spreadsheet System  ·  Apapo · Kogi · Ume · Qala*