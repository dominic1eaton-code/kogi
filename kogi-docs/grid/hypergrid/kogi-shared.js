// KOGI Shared Navigation helper
// All page links are relative; update BASE if needed
const KOGI_NAV = [
  { href:'portfolio-grid.html',    label:'⊞ Grid',          id:'grid' },
  { href:'portfolio-programs.html',label:'◈ Programs',       id:'programs' },
  { href:'portfolio-resources.html',label:'◎ Resources',     id:'resources' },
  { sep: true },
  { href:'portfolio-analytics.html',label:'∿ Analytics',     id:'analytics' },
  { href:'portfolio-governance.html',label:'⊛ Governance',   id:'governance' },
  { href:'portfolio-graph.html',   label:'⋈ Graph',          id:'graph' },
  { sep: true },
  { href:'portfolio-benefits.html',label:'⊕ Benefits',       id:'benefits' },
  { href:'portfolio-collaboration.html',label:'⊗ Collab',    id:'collaboration' },
];
function buildSubNav(activeId) {
  const el = document.getElementById('sub-nav');
  if (!el) return;
  KOGI_NAV.forEach(item => {
    if (item.sep) { const d=document.createElement('div');d.className='sn-sep';el.appendChild(d); return; }
    const a = document.createElement('a');
    a.href = item.href; a.className='sn-item'+(item.id===activeId?' active':'');
    a.textContent = item.label; el.appendChild(a);
  });
}
