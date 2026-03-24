/* hg-data.js — Shared mock data store for Hypergrid UI */

window.HGData = (() => {

// ── GRIDS ──
const grids = [
  { id:'grid-kogi', name:'Kogi Grid', domain:'Kogi IW-OS', host:'kogi.hypergrid.io', node_id:'us-east:kogi-01', status:'running', cubes:7, federation_peers:3, health:94, version:'2.1.3', color:'#00d8b0' },
  { id:'grid-ume',  name:'Ume Grid',  domain:'Ume B-OS',   host:'ume.hypergrid.io',  node_id:'us-west:ume-01',  status:'running', cubes:42,health:88, version:'1.8.0', color:'#3d84e8', federation_peers:5 },
  { id:'grid-qala', name:'Qala Grid', domain:'Qala SF-OS',  host:'qala.hypergrid.io', node_id:'eu-west:qala-01', status:'degraded',cubes:9, health:71, version:'1.2.0', color:'#8b5cf6', federation_peers:2 },
  { id:'grid-dev',  name:'Dev Grid',  domain:'Local Dev',   host:'localhost:8080',    node_id:'local:dev-01',    status:'running', cubes:3, health:100,version:'2.1.3', color:'#e8a832', federation_peers:0 },
];

// ── HYPERCUBES ──
const cubes = [
  { id:'cube-kogi-components', grid:'grid-kogi', name:'kogi.portfolio.components', dims:2, rows:1842, cols:24, status:'active', color:'#00d8b0', description:'All Component nodes (items, containers)', cube_type:'Item', dim_axes:['EntityAxis(Uuid)','PropertyAxis(String)'] },
  { id:'cube-kogi-kpis', grid:'grid-kogi', name:'kogi.portfolio.kpis', dims:3, rows:8240, cols:12, status:'active', color:'#00d8b0', description:'KPI time-series metrics', cube_type:'Metrics', dim_axes:['EntityAxis','PropertyAxis','TimeAxis'] },
  { id:'cube-kogi-actions', grid:'grid-kogi', name:'kogi.portfolio.actions', dims:2, rows:48200, cols:8, status:'active', color:'#00d8b0', description:'ActionKind event records', cube_type:'EventLog', dim_axes:['EntityAxis','PropertyAxis'] },
  { id:'cube-kogi-benefits', grid:'grid-kogi', name:'kogi.portfolio.benefits', dims:3, rows:340, cols:18, status:'active', color:'#00d8b0', description:'BenefitAccount portfolio items (§19)', cube_type:'Item', dim_axes:['EntityAxis','PropertyAxis','CategoryAxis'] },
  { id:'cube-ume-modules', grid:'grid-ume', name:'ume.kernel.modules', dims:2, rows:42, cols:17, status:'active', color:'#3d84e8', description:'All 42 OrgModule registrations', cube_type:'Registry', dim_axes:['EntityAxis','PropertyAxis'] },
  { id:'cube-ume-finance', grid:'grid-ume', name:'ume.finance.accounts', dims:3, rows:2840, cols:14, status:'active', color:'#3d84e8', description:'Finance & Accounting module', cube_type:'Data', dim_axes:['EntityAxis','PropertyAxis','TimeAxis'] },
  { id:'cube-ume-soko', grid:'grid-ume', name:'ume.soko.campaigns', dims:4, rows:420, cols:20, status:'active', color:'#3d84e8', description:'Soko marketing campaigns N=4', cube_type:'Data', dim_axes:['EntityAxis','PropertyAxis','TimeAxis','CategoryAxis'] },
  { id:'cube-ume-grc', grid:'grid-ume', name:'ume.grc.risks', dims:3, rows:890, cols:16, status:'active', color:'#3d84e8', description:'GRC risk register', cube_type:'Risk', dim_axes:['EntityAxis','PropertyAxis','TimeAxis'] },
  { id:'cube-qala-solutions', grid:'grid-qala', name:'qala.solutions', dims:2, rows:284, cols:22, status:'active', color:'#8b5cf6', description:'Solution records (all types)', cube_type:'Item', dim_axes:['EntityAxis','PropertyAxis'] },
  { id:'cube-qala-sdes', grid:'grid-qala', name:'qala.sdes', dims:3, rows:128, cols:19, status:'active', color:'#8b5cf6', description:'Solution Development Environments N=3', cube_type:'Item', dim_axes:['EntityAxis','PropertyAxis','VersionAxis'] },
  { id:'cube-qala-artifacts', grid:'grid-qala', name:'qala.artifacts', dims:2, rows:4800, cols:12, status:'active', color:'#8b5cf6', description:'Artifact records', cube_type:'Artifact', dim_axes:['EntityAxis','PropertyAxis'] },
];

// ── ATTRIBUTE KEY DEFS ──
const attrKeys = [
  { key:'id', type:'Text', crdt:'LWW', tier:'Public', required:true, ai:false, description:'Canonical UUID identifier', cube:'cube-kogi-components' },
  { key:'name', type:'Text', crdt:'LWW', tier:'Public', required:true, ai:false, description:'Display name', cube:'cube-kogi-components' },
  { key:'description', type:'Text', crdt:'LWW', tier:'Public', required:false, ai:false, description:'Rich text description', cube:'cube-kogi-components' },
  { key:'status', type:'Enum', crdt:'LWW', tier:'Editor', required:true, ai:false, description:'Lifecycle status: Draft|Active|Paused|Completed|Archived|Deleted', cube:'cube-kogi-components' },
  { key:'state', type:'Enum', crdt:'LWW', tier:'Editor', required:false, ai:false, description:'Operational state: Running|Blocked|Sealed|etc', cube:'cube-kogi-components' },
  { key:'version', type:'Text', crdt:'MaxRegister', tier:'Editor', required:true, ai:false, description:'Semver version string — always monotonically increasing', cube:'cube-kogi-components' },
  { key:'owners', type:'Json', crdt:'OrSet', tier:'Owner', required:true, ai:false, description:'Vec<UserId> — OR-Set concurrent-safe', cube:'cube-kogi-components' },
  { key:'tags', type:'Tag', crdt:'OrSet', tier:'Contributor', required:false, ai:false, description:'Flat string tag set — OR-Set', cube:'cube-kogi-components' },
  { key:'budget', type:'Number', crdt:'LWW', tier:'Manager', required:false, ai:false, description:'Allocated budget f64', cube:'cube-kogi-components' },
  { key:'budget_spent', type:'Number', crdt:'PnCounter', tier:'Manager', required:false, ai:false, description:'Consumed budget — P/N counter for concurrent spend', cube:'cube-kogi-components' },
  { key:'health_score', type:'Number', crdt:'LWW', tier:'System', required:false, ai:true, description:'AI-computed health 0–100', cube:'cube-kogi-components', computed_by:'KogiHealthEngine.v2', confidence:0.91 },
  { key:'risk_score', type:'Number', crdt:'LWW', tier:'System', required:false, ai:true, description:'AI-computed risk 0–100', cube:'cube-kogi-components', computed_by:'KogiRiskEngine.v1', confidence:0.87 },
  { key:'views', type:'Number', crdt:'GrowOnlyCounter', tier:'Public', required:false, ai:false, description:'Impression counter — grows only', cube:'cube-kogi-components' },
  { key:'visibility', type:'Enum', crdt:'LWW', tier:'Owner', required:true, ai:false, description:'Public|Protected|Private|Unlisted|DraftOnly', cube:'cube-kogi-components' },
  { key:'vector_clock', type:'Json', crdt:'LWW', tier:'System', required:true, ai:false, description:'NodeId→timestamp causal ordering map', cube:'cube-kogi-components' },
];

// ── GRAPH EDGES ──
const edges = [
  { id:'e1', source:'node-alice', target:'node-brand-project', edge_type:'Hierarchy', weight:1.0, directed:true, consent:'N/A', grid:'grid-kogi', color:'#3d84e8' },
  { id:'e2', source:'node-alice', target:'node-carol', edge_type:'Collaborates', weight:0.9, directed:false, consent:'Accepted', grid:'grid-kogi', color:'#00d8b0' },
  { id:'e3', source:'node-alice', target:'node-acme-corp', edge_type:'CrossGridLink', weight:0.8, directed:true, consent:'Accepted', grid:'grid-kogi', grid2:'grid-ume', color:'#8b5cf6' },
  { id:'e4', source:'node-brand-project', target:'node-design-lib', edge_type:'Dependency', weight:1.0, directed:true, consent:'N/A', grid:'grid-kogi', color:'#e8a832' },
  { id:'e5', source:'node-acme-corp', target:'node-finance-module', edge_type:'Contains', weight:1.0, directed:true, consent:'N/A', grid:'grid-ume', color:'#e87032' },
  { id:'e6', source:'node-qala-factory', target:'node-kogi-platform', edge_type:'CrossGridLink', weight:1.0, directed:true, consent:'Accepted', grid:'grid-qala', grid2:'grid-kogi', color:'#8b5cf6' },
  { id:'e7', source:'node-carol', target:'node-dev-coop', edge_type:'OrgMembership', weight:1.0, directed:true, consent:'Accepted', grid:'grid-kogi', color:'#3cb878' },
  { id:'e8', source:'node-brand-project', target:'node-strategy-deck', edge_type:'Association', weight:0.7, directed:false, consent:'N/A', grid:'grid-kogi', color:'#e054a8' },
];

// ── NODES ──
const nodes = [
  { id:'node-alice', label:'@alice-dev', type:'Identity', domain:'Kogi', cube:'cube-kogi-components', color:'#00d8b0', x:350, y:180, size:14 },
  { id:'node-brand-project', label:'Brand Identity', type:'Project', domain:'Kogi', cube:'cube-kogi-components', color:'#8b5cf6', x:220, y:120, size:11 },
  { id:'node-carol', label:'@carol', type:'Identity', domain:'Kogi', cube:'cube-kogi-components', color:'#00d8b0', x:480, y:140, size:10 },
  { id:'node-acme-corp', label:'Acme Corp', type:'Organization', domain:'Ume', cube:'cube-ume-modules', color:'#3d84e8', x:520, y:250, size:13 },
  { id:'node-design-lib', label:'Design System', type:'Asset', domain:'Kogi', cube:'cube-kogi-components', color:'#e8a832', x:150, y:200, size:9 },
  { id:'node-finance-module', label:'Finance Module', type:'OrgModule', domain:'Ume', cube:'cube-ume-modules', color:'#3d84e8', x:620, y:300, size:10 },
  { id:'node-qala-factory', label:'Root Factory', type:'Factory', domain:'Qala', cube:'cube-qala-solutions', color:'#8b5cf6', x:350, y:350, size:13 },
  { id:'node-kogi-platform', label:'Kogi Platform', type:'Solution', domain:'Qala', cube:'cube-qala-solutions', color:'#00d8b0', x:220, y:350, size:11 },
  { id:'node-dev-coop', label:'Dev Collective', type:'Collective', domain:'Kogi', cube:'cube-kogi-components', color:'#3cb878', x:580, y:160, size:9 },
  { id:'node-strategy-deck', label:'Q3 Strategy', type:'Artifact', domain:'Kogi', cube:'cube-kogi-components', color:'#e054a8', x:120, y:130, size:8 },
];

// ── SPACES ──
const spaces = [
  { id:'space-alice-personal', name:'@alice-dev Personal', type:'Personal', domain:'Kogi', slug:'kogi://alice-dev/personal/', members:1, cubes:3, health:94, status:'active', color:'#00d8b0' },
  { id:'space-design-team', name:'Design Team Portfolio', type:'Team', domain:'Kogi', slug:'kogi://portfolio/design-team/', members:8, cubes:5, health:88, status:'active', color:'#8b5cf6' },
  { id:'space-acme-org', name:'Acme Corp', type:'Organization', domain:'Ume', slug:'ume://acme-corp/', members:240, cubes:42, health:82, status:'active', color:'#3d84e8' },
  { id:'space-root-factory', name:'Qala Root Factory', type:'Factory', domain:'Qala', slug:'qala://root-factory/', members:12, cubes:9, health:71, status:'degraded', color:'#8b5cf6' },
  { id:'space-dev-coop', name:'Dev Collective', type:'Community', domain:'Kogi', slug:'kogi://cooperative/dev-coop/', members:48, cubes:4, health:91, status:'active', color:'#3cb878' },
];

// ── IDENTITIES ──
const identities = [
  { id:'id-alice', entity:'@alice-dev', sovereign_id:'sov-alice', partitions:['kogi','ume-acme'], tier:4, active_space:'space-alice-personal', scopes:['grid:read','cubes:write','graph:read'], color:'#00d8b0' },
  { id:'id-carol', entity:'@carol', sovereign_id:'sov-carol', partitions:['kogi'], tier:3, active_space:'space-design-team', scopes:['grid:read','cubes:write'], color:'#3cb878' },
  { id:'id-acme', entity:'acme-corp', sovereign_id:'sov-acme', partitions:['ume'], tier:5, active_space:'space-acme-org', scopes:['grid:read','grid:write','cubes:write','federation:manage'], color:'#3d84e8' },
  { id:'id-oba', entity:'oba-ai-agent', sovereign_id:'sov-oba', partitions:['kogi','qala'], tier:2, active_space:'space-alice-personal', scopes:['grid:read','cubes:read','ai:write'], color:'#e8a832' },
];

// ── EVENTLOG ──
const events = [
  { id:'ev1', kind:'ComponentCreated', entity_id:'node-brand-project', actor:'@alice-dev', ts:'2026-03-20T10:02:00Z', grid:'grid-kogi', payload:{name:'Brand Identity System',status:'active'} },
  { id:'ev2', kind:'CrdtMergeApplied', entity_id:'node-alice', actor:'system', ts:'2026-03-20T10:05:22Z', grid:'grid-kogi', payload:{ops:3, merged_from:'eu-west:kogi-02'} },
  { id:'ev3', kind:'EdgeAdded', entity_id:'e2', actor:'@alice-dev', ts:'2026-03-20T10:08:14Z', grid:'grid-kogi', payload:{edge_type:'Collaborates',target:'@carol'} },
  { id:'ev4', kind:'ComponentStatusChanged', entity_id:'node-brand-project', actor:'@carol', ts:'2026-03-20T11:22:00Z', grid:'grid-kogi', payload:{from:'Draft',to:'Active'} },
  { id:'ev5', kind:'AiComputationCompleted', entity_id:'node-alice', actor:'KogiHealthEngine', ts:'2026-03-20T12:00:00Z', grid:'grid-kogi', payload:{attr:'health_score',value:87,confidence:0.91} },
  { id:'ev6', kind:'SnapshotSaved', entity_id:'grid-kogi', actor:'system', ts:'2026-03-20T12:30:00Z', grid:'grid-kogi', payload:{label:'hourly-checkpoint'} },
  { id:'ev7', kind:'FederationSyncCompleted', entity_id:'grid-kogi', actor:'federation', ts:'2026-03-20T13:00:00Z', grid:'grid-kogi', payload:{peer:'eu-west:kogi-02',ops_applied:14} },
  { id:'ev8', kind:'PolicyDenied', entity_id:'node-acme-corp', actor:'@carol', ts:'2026-03-20T13:15:00Z', grid:'grid-ume', payload:{policy:'budget-cap-policy',reason:'Budget allocation exceeds cap'} },
  { id:'ev9', kind:'ComponentUpdated', entity_id:'cube-qala-solutions', actor:'@qala-agent', ts:'2026-03-20T14:00:00Z', grid:'grid-qala', payload:{attr:'health_score',value:71,confidence:0.84} },
  { id:'ev10',kind:'ResourceAllocated', entity_id:'node-brand-project', actor:'@alice-dev', ts:'2026-03-20T14:30:00Z', grid:'grid-kogi', payload:{kind:'Budget',amount:25000,currency:'USD'} },
];

// ── AI ENGINES ──
const aiEngines = [
  { id:'ai-kogi-health', name:'KogiHealthEngine', version:'v2', domain:'Kogi', attr:'health_score', model:'GradientBoosting', status:'running', last_run:'2m ago', confidence_avg:0.91, color:'#00d8b0' },
  { id:'ai-kogi-risk',   name:'KogiRiskEngine',   version:'v1', domain:'Kogi', attr:'risk_score',   model:'RandomForest', status:'running', last_run:'5m ago', confidence_avg:0.87, color:'#e8526a' },
  { id:'ai-kogi-anomaly',name:'KogiAnomalyEngine',version:'v1', domain:'Kogi', attr:'anomaly_flag', model:'IsolationForest', status:'running', last_run:'1m ago', confidence_avg:0.84, color:'#e8a832' },
  { id:'ai-kogi-align',  name:'KogiAlignmentEngine',version:'v1',domain:'Kogi',attr:'alignment_score',model:'NNClassifier', status:'idle', last_run:'12m ago', confidence_avg:0.79, color:'#3d84e8' },
  { id:'ai-ume-health',  name:'UmeKernelHealth',  version:'v1', domain:'Ume',  attr:'health_score', model:'LogisticRegression', status:'running', last_run:'3m ago', confidence_avg:0.88, color:'#3d84e8' },
  { id:'ai-qala-ccr',    name:'QalaCCRReviewer',  version:'v1', domain:'Qala', attr:'ccr_risk',     model:'LLM+Rules', status:'running', last_run:'7m ago', confidence_avg:0.92, color:'#8b5cf6' },
  { id:'ai-nl-query',    name:'NL-to-HyperQL',    version:'v1', domain:'All',  attr:'N/A',          model:'GPT-4o',    status:'running', last_run:'now', confidence_avg:0.95, color:'#e054a8' },
];

// ── FEDERATION PEERS ──
const fedPeers = [
  { id:'peer-eu-kogi', grid:'grid-kogi', node_id:'eu-west:kogi-02', status:'synced', last_seen:'12s ago', lag_ops:2, trusted:true, crdt_ops_pending:0 },
  { id:'peer-ap-kogi', grid:'grid-kogi', node_id:'ap-east:kogi-03', status:'lagging', last_seen:'4m ago', lag_ops:48, trusted:true, crdt_ops_pending:48 },
  { id:'peer-eu-ume',  grid:'grid-ume',  node_id:'eu-west:ume-02',  status:'synced', last_seen:'3s ago', lag_ops:0, trusted:true, crdt_ops_pending:0 },
  { id:'peer-us-qala', grid:'grid-qala', node_id:'us-east:qala-02', status:'disconnected', last_seen:'18m ago', lag_ops:142, trusted:true, crdt_ops_pending:142 },
];

return { grids, cubes, attrKeys, edges, nodes, spaces, identities, events, aiEngines, fedPeers };

})();
