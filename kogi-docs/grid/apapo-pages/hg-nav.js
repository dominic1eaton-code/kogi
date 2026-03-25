/* hg-nav.js — Shared navigation, toast, and modal helpers */

const HG_PAGES = [
  { id:'dashboard', icon:'⬡', label:'Dashboard', href:'index.html', section:'Platform' },
  { id:'grid',      icon:'⊞', label:'Grid Manager', href:'grid.html', section:'Platform' },
  { id:'cubes',     icon:'◧', label:'Hypercubes', href:'cubes.html', section:'Data' },
  { id:'cells',     icon:'⊡', label:'HyperCells', href:'cells.html', section:'Data' },
  { id:'graph',     icon:'◎', label:'Hypergraph', href:'graph.html', section:'Graph' },
  { id:'graph-nd',  icon:'⬡', label:'N-Dim Graph', href:'graph-nd.html', section:'Graph' },
  { id:'hyperql',   icon:'≡', label:'HyperQL', href:'query.html', section:'Query' },
  { id:'spaces',    icon:'◻', label:'Spaces', href:'spaces.html', section:'Platform' },
  { id:'identity',  icon:'◈', label:'Identity', href:'identity.html', section:'Platform' },
  { id:'crdt',      icon:'⟳', label:'CRDT / Sync', href:'crdt.html', section:'Engine' },
  { id:'ai',        icon:'✦', label:'AI Engine', href:'ai.html', section:'Engine' },
  { id:'eventlog',  icon:'↯', label:'Event Log', href:'eventlog.html', section:'Engine' },
  { id:'kogi',      icon:'◈', label:'Kogi', href:'kogi.html', section:'Domain OS' },
  { id:'ume',       icon:'◫', label:'Ume', href:'ume.html', section:'Domain OS' },
  { id:'qala',      icon:'◆', label:'Qala', href:'qala.html', section:'Domain OS' },
  { id:'federation',icon:'⊕', label:'Federation', href:'federation.html', section:'Infra' },
];

function renderNav(activeId) {
  const nav = document.getElementById('hg-iconav');
  if (!nav) return;

  const sections = {};
  HG_PAGES.forEach(p => { if (!sections[p.section]) sections[p.section] = []; sections[p.section].push(p); });

  let html = '';
  Object.entries(sections).forEach(([sec, pages], si) => {
    if (si > 0) html += '<div class="iconav-divider"></div>';
    pages.forEach(p => {
      html += `<a class="iconav-item${p.id===activeId?' active':''}" href="${p.href}" title="${p.label}">
        <span>${p.icon}</span>
        <span class="tooltip">${p.label}</span>
      </a>`;
    });
  });
  nav.innerHTML = html;
}

// ── TOAST ──
let _toastT;
function showToast(msg, type='info') {
  let t = document.getElementById('hg-toast');
  if (!t) { t = document.createElement('div'); t.id='hg-toast'; t.className='toast'; document.body.appendChild(t); }
  t.textContent = msg;
  if (type==='error') t.style.borderColor = 'var(--rose)';
  else if (type==='success') t.style.borderColor = 'var(--green)';
  else t.style.borderColor = '';
  t.classList.add('show');
  clearTimeout(_toastT);
  _toastT = setTimeout(() => t.classList.remove('show'), 2400);
}

// ── MODAL ──
function openModal(id) { const m = document.getElementById(id); if (m) m.classList.add('open'); }
function closeModal(id) { const m = document.getElementById(id); if (m) m.classList.remove('open'); }
document.addEventListener('click', e => { if (e.target.classList.contains('modal-overlay')) closeModal(e.target.id); });

// ── TAB SWITCHER ──
function switchTab(groupId, tabId, el) {
  const group = document.querySelectorAll('[data-tab-group="' + groupId + '"]');
  group.forEach(t => t.classList.remove('active'));
  if (el) el.classList.add('active');
  const panels = document.querySelectorAll('[data-tab-panel-group="' + groupId + '"]');
  panels.forEach(p => p.style.display = p.dataset.tabPanel === tabId ? '' : 'none');
}

// ── COPY TO CLIPBOARD ──
function copyText(text) {
  navigator.clipboard.writeText(text).then(() => showToast('Copied to clipboard', 'success'));
}

// ── FORMAT HELPERS ──
function fmtDate(ts) {
  if (!ts) return '—';
  const d = new Date(ts);
  return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: '2-digit' });
}
function fmtDateFull(ts) {
  if (!ts) return '—';
  return new Date(ts).toLocaleString('en-US', { month: 'short', day: 'numeric', year: 'numeric', hour: '2-digit', minute: '2-digit' });
}
function bumpVer(v, part) {
  const clean = v.replace(/-.*$/, '');
  const p = clean.split('.').map(Number);
  if (part === 'major') return `${p[0]+1}.0.0`;
  if (part === 'minor') return `${p[0]}.${(p[1]||0)+1}.0`;
  if (part === 'patch') return `${p[0]}.${p[1]||0}.${(p[2]||0)+1}`;
  return v;
}

// ── COLOR PICKER ──
const HG_COLORS = [
  '#00d8b0','#3d84e8','#8b5cf6','#3cb878','#e8a832',
  '#e8526a','#e87032','#2dd4bf','#e054a8','#c8a84b',
  '#5ca8bf','#a8bf5c','#bf5c7a','#7cbf5c','#5c7abf',
];
function buildColorPicker(containerId, current, onChange) {
  const el = document.getElementById(containerId);
  if (!el) return;
  el.innerHTML = '';
  el.className = 'color-grid';
  HG_COLORS.forEach(c => {
    const sw = document.createElement('div');
    sw.className = 'csw' + (c === current ? ' sel' : '');
    sw.style.background = c;
    sw.onclick = () => {
      el.querySelectorAll('.csw').forEach(s => s.classList.remove('sel'));
      sw.classList.add('sel');
      if (onChange) onChange(c);
    };
    el.appendChild(sw);
  });
}
