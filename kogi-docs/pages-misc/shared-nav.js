// shared-nav.js — inject sidebar on every page
function buildSidebar(activePage) {
  const nav = [
    { section: 'Platform' },
    { id: 'index',      label: 'Dashboard',       icon: '⬡', href: 'index.html' },
    { id: 'grids',      label: 'Grids',            icon: '▦', href: 'grids.html' },
    { section: 'Data' },
    { id: 'hypercube',  label: 'Hypercubes',       icon: '⊞', href: 'hypercube.html' },
    { id: 'schema',     label: 'Schema Editor',    icon: '⊹', href: 'schema.html' },
    { id: 'query',      label: 'HyperQL Console',  icon: '›_', href: 'query.html' },
    { id: 'eventlog',   label: 'Event Log',        icon: '≡', href: 'eventlog.html' },
    { section: 'Graph' },
    { id: 'graph',      label: 'Hypergraph',       icon: '◎', href: 'graph.html' },
    { id: 'federation', label: 'Federation',       icon: '⟷', href: 'federation.html' },
    { section: 'Intelligence' },
    { id: 'ai',         label: 'AI Computation',   icon: '✦', href: 'ai.html' },
    { section: 'Administration' },
    { id: 'admin',      label: 'Identity & Spaces',icon: '⊚', href: 'admin.html' },
  ];

  let html = `
    <div class="sidebar-header">
      <div class="sidebar-wordmark">Apapo Platform</div>
      <div class="sidebar-title">Hypergrid</div>
      <div class="sidebar-badge">v1.0 · Production</div>
    </div>`;

  let inSection = false;
  nav.forEach(item => {
    if (item.section) {
      if (inSection) html += '<div class="sidebar-sep"></div>';
      html += `<div class="sidebar-section">${item.section}</div>`;
      inSection = true;
    } else {
      const active = item.id === activePage ? 'active' : '';
      html += `
        <a href="${item.href}" class="nav-item ${active}">
          <span class="nav-icon">${item.icon}</span>
          <span>${item.label}</span>
        </a>`;
    }
  });

  html += `
    <div class="sidebar-spacer"></div>
    <div class="sidebar-status">
      <div class="status-row"><span class="dot green"></span>Grid: kogi-production</div>
      <div class="status-row"><span class="dot green"></span>PostgreSQL · Redis · Kafka</div>
      <div class="status-row"><span class="dot amber"></span>2 federation peers</div>
      <div class="status-row" style="margin-top:8px;font-size:10px;color:#4a4a46">
        node-us-east-1 · @admin
      </div>
    </div>`;

  document.getElementById('sidebar').innerHTML = html;
}
