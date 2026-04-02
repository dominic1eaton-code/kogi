// KOGI Portfolio System — Shared Navigation v2
const KOGI_NAV = [
  { href:'portfolio-grid.html',         label:'⊞ Grid',          id:'grid' },
  { href:'portfolio-components.html',   label:'◫ Components',    id:'components' },
  { href:'portfolio-items.html',        label:'◈ Items',         id:'items' },
  { href:'portfolio-containers.html',   label:'▣ Containers',    id:'containers' },
  { sep: true },
  { href:'portfolio-programs.html',     label:'◎ Programs',      id:'programs' },
  { href:'portfolio-resources.html',    label:'⊙ Resources',     id:'resources' },
  { href:'portfolio-people.html',       label:'◉ People',        id:'people' },
  { sep: true },
  { href:'portfolio-commerce.html',     label:'◆ Commerce',      id:'commerce' },
  { href:'portfolio-finance.html',      label:'$ Finance',       id:'finance' },
  { href:'portfolio-timelines.html',    label:'⌇ Timelines',     id:'timelines' },
  { sep: true },
  { href:'portfolio-analytics.html',    label:'∿ Analytics',     id:'analytics' },
  { href:'portfolio-governance.html',   label:'⊛ Governance',    id:'governance' },
  { href:'portfolio-graph.html',        label:'⋈ Graph',         id:'graph' },
  { sep: true },
  { href:'portfolio-benefits.html',     label:'⊕ Benefits',      id:'benefits' },
  { href:'portfolio-collaboration.html',label:'⊗ Collab',        id:'collaboration' },
];

function buildSubNav(activeId) {
  const el = document.getElementById('sub-nav');
  if (!el) return;
  KOGI_NAV.forEach(item => {
    if (item.sep) {
      const d = document.createElement('div');
      d.className = 'sn-sep';
      el.appendChild(d);
      return;
    }
    const a = document.createElement('a');
    a.href = item.href;
    a.className = 'sn-item' + (item.id === activeId ? ' active' : '');
    a.textContent = item.label;
    el.appendChild(a);
  });
}

// ── shared colour + type maps ──────────────────────────────────────────
const KOGI_COLORS = [
  '#00d4aa','#c8a84b','#7c5cbf','#c45c7a','#c87c3a',
  '#3a7cc8','#4aab6d','#bf5ca0','#5ca8bf','#a0bf5c',
  '#bf7c5c','#7abf5c','#5c7abf','#bf5c7a','#5cb8bf',
  '#d4aa00','#aa00d4','#d40055','#00aad4','#d46800',
];

const CAT_COLORS = {
  finance:'#4aab6d', tech:'#3a7cc8', creative:'#7c5cbf',
  strategy:'#c8a84b', legal:'#c45c7a', infra:'#c87c3a',
  community:'#bf5ca0', research:'#00d4aa', education:'#5ca8bf',
  product:'#a0bf5c', commerce:'#d4aa00', people:'#5c7abf',
  time:'#aa5cbf', containers:'#5cbf7a', capital:'#c87c3a',
  health:'#c45c7a', work:'#3a7cc8',
};
const CAT_LABELS = {
  finance:'Finance', tech:'Technology', creative:'Creative',
  strategy:'Strategy', legal:'Legal', infra:'Infrastructure',
  community:'Community', research:'Research', education:'Education',
  product:'Product', commerce:'Commerce', people:'People',
  time:'Time', containers:'Containers', capital:'Capital',
  health:'Health', work:'Work',
};
const STATUS_LABELS = { active:'Active', draft:'Draft', paused:'Paused', completed:'Complete', archived:'Archived' };
const STATUS_CLS    = { active:'sta', draft:'std', paused:'stp', completed:'stc', archived:'stx' };

const EDGE_TYPES = {
  hierarchy:    { label:'Hierarchy',   color:'#00d4aa', icon:'↗', desc:'Parent → Child' },
  dependency:   { label:'Dependency',  color:'#c8a84b', icon:'→', desc:'Blocks B' },
  link:         { label:'Link',        color:'#7c5cbf', icon:'⟷', desc:'Generic ref' },
  contains:     { label:'Contains',    color:'#3a7cc8', icon:'⊂', desc:'Container → Item' },
  resourceshare:{ label:'ResShare',    color:'#c87c3a', icon:'⇌', desc:'Shared resource' },
  attribution:  { label:'Attribution', color:'#bf5ca0', icon:'◎', desc:'Contributor' },
  investment:   { label:'Investment',  color:'#4aab6d', icon:'$', desc:'Investor → item' },
  membership:   { label:'Membership',  color:'#5ca8bf', icon:'⊙', desc:'Member of' },
  governs:      { label:'Governs',     color:'#c45c7a', icon:'⊛', desc:'Policy → item' },
  derives:      { label:'Derives',     color:'#a0bf5c', icon:'⇒', desc:'Source → derived' },
};

// shared uid generator
let _gid = 2000;
const uid = () => 'k' + (++_gid) + '_' + Math.random().toString(36).slice(2,5);
