// nodesrc/html_gen_playground.js
// 目的:
// - 既存 html_gen.js は維持したまま、チュートリアル向けの実行可能 HTML を生成する。
// - pre>code(language-neplg2) をクリックすると、ポップアップエディタで Run / Interrupt / 出力確認ができる。

const { renderNode, renderInlines } = require('./html_gen');
const { parseInlines } = require('./parser');
const fs = require('fs');
const path = require('path');

function escapeHtml(s) {
    return String(s)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
}

function renderBody(ast) {
    return renderNode(ast, { rewriteLinks: true });
}

function renderToc(tocLinks) {
    if (!Array.isArray(tocLinks) || tocLinks.length === 0) {
        return '';
    }
    const items = tocLinks.map(link => {
        const depth = Number.isFinite(link.depth) ? Math.max(0, Math.min(6, link.depth)) : 0;

        const labelNodes = parseInlines(String(link.label || ''));
        const labelHtml = renderInlines(labelNodes);

        if (link.isGroup) {
            return `<li><div class="toc-group depth-${depth}">${labelHtml}</div></li>`;
        }
        const cls = link.active ? `toc-link active depth-${depth}` : `toc-link depth-${depth}`;
        return `<li><a class="${cls}" href="${escapeHtml(String(link.href || ''))}">${labelHtml}</a></li>`;
    }).join('\n');
    return `<aside class="doc-sidebar"><div class="sidebar-header"><div class="toc-title">Getting Started</div></div><ul class="toc-list">${items}</ul></aside>`;
}

function buildPlaygroundVfsOverrides() {
    const rels = [
        'stdlib/kp/kpread.nepl',
        'stdlib/kp/kpwrite.nepl',
        'stdlib/kp/kpgraph.nepl',
        'stdlib/kp/kpsearch.nepl',
        'stdlib/kp/kpprefix.nepl',
        'stdlib/kp/kpdsu.nepl',
        'stdlib/kp/kpfenwick.nepl',
    ];
    const out = {};
    for (const rel of rels) {
        const abs = path.resolve(process.cwd(), rel);
        if (!fs.existsSync(abs)) continue;
        const key = '/stdlib/' + rel.replace(/^stdlib\//, '').replace(/\\/g, '/');
        out[key] = fs.readFileSync(abs, 'utf8');
    }
    return out;
}

function wrapHtmlPlayground(body, title, description, moduleJsPathOpt) {
    const t = title || 'NEPLg2 Tutorial';
    const d = description || 'NEPLg2 tutorial with interactive runnable examples.';
    const moduleJsPath = (moduleJsPathOpt && String(moduleJsPathOpt)) || './nepl-web.js';
    const vfsOverrides = buildPlaygroundVfsOverrides();
    const vfsOverridesJson = JSON.stringify(vfsOverrides);
    const tocHtml = (arguments[4] && String(arguments[4])) || '';
    return `<!doctype html>
<html lang="ja">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>${escapeHtml(t)}</title>
<meta name="description" content="${escapeHtml(d)}"/>
<meta property="og:title" content="${escapeHtml(t)}"/>
<meta property="og:description" content="${escapeHtml(d)}"/>
<meta property="og:type" content="article"/>
<meta name="twitter:card" content="summary"/>
<meta name="twitter:title" content="${escapeHtml(t)}"/>
<meta name="twitter:description" content="${escapeHtml(d)}"/>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Klee+One:wght@400;600&display=swap" rel="stylesheet">
<style>
:root{
  --bg:#0b0f19;
  --fg:#e6edf3;
  --muted:#aab6c3;
  --card:#121a2a;
  --border:#23304a;
  --code:#0f1626;
  --accent:#7aa2f7;
  --ok:#59c37a;
  --err:#ff6b6b;
}
html,body{
  background:var(--bg);
  color:var(--fg);
  font-family:'Klee One', system-ui,-apple-system,Segoe UI,Roboto,Helvetica,Arial;
  line-height:1.65;
  margin:0;
  padding:0;
}
.doc-layout{
  display:grid;
  grid-template-columns:280px 1fr;
  min-height:100vh;
}
main{
  min-width:0;
  padding:24px 40px;
  max-width:1200px;
  margin:0 auto;
  width:100%;
  box-sizing:border-box;
}
a{color:var(--accent);}
.global-play-link{
  display:inline-flex;
  align-items:center;
  gap:6px;
  padding:6px 10px;
  border-radius:999px;
  border:1px solid var(--border);
  background:rgba(11,15,25,0.92);
  color:var(--fg);
  text-decoration:none;
  font-size:12px;
  transition:all 0.2s;
}
.global-play-link:hover{border-color:#355186;background:rgba(18,26,42,0.96);}
.global-play-link.pc-only{
  position:fixed;
  right:14px;
  top:12px;
  z-index:10000;
}
hr{border:none;border-top:1px solid var(--border);margin:24px 0;}
.nm-sec{padding:0.5em;padding-left:2em;margin:1em;border-left:3px solid var(--border);border-radius:1em;}
h1,h2,h3,h4,h5,h6{margin:18px 0 10px;}
p{margin:10px 0;}
ul{margin:10px 0 10px 22px;}
.nm-code-container{
  border:1px solid var(--border);
  border-radius:12px;
  background:var(--card);
  margin:24px 0;
  overflow:hidden;
}
.nm-code-header{
  display:flex;
  align-items:center;
  gap:8px;
  padding:8px 12px;
  background:rgba(255,255,255,0.03);
  border-bottom:1px solid var(--border);
  flex-wrap:wrap;
}
.nm-badge-main{
  display:inline-block;padding:2px 8px;border-radius:6px;background:#7aa2f7;color:#1a202e;font-size:11px;font-weight:bold;letter-spacing:.05em;
}
.nm-badge-flag{
  display:inline-block;padding:2px 8px;border-radius:6px;border:1px solid var(--border);background:rgba(0,0,0,0.2);color:var(--muted);font-size:11px;
}
.nm-code-content{position:relative;}
.nm-code{background:var(--code);padding:12px;overflow:auto;margin:0;border:none;border-radius:0;}
.nm-code code{font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:13px;white-space:pre;}
.nm-syn-keyword{color:#7aa2f7;}
.nm-syn-string{color:#9ece6a;}
.nm-syn-number{color:#ff9e64;}
.nm-syn-comment{color:#7f8ea3;}
.nm-syn-boolean{color:#e0af68;}
.nm-syn-function{color:#73daca;}
.nm-syn-operator{color:#c0caf5;}
.nm-syn-punctuation{color:#a9b1d6;}
.nm-syn-type{color:#e5c07b;}
.nm-syn-constant{color:#bb9af7;}
.nm-code-inline{background:rgba(255,255,255,0.06);border:1px solid rgba(255,255,255,0.10);border-radius:8px;padding:1px 6px;}
.nm-gloss, .nm-ruby{ruby-position:over;}
.nm-gloss rt{font-size:0.72em;color:var(--muted);line-height:1.1;}
.nm-gloss-note{display:block;}
.math-inline{color:var(--muted);}
.math-display{display:block;padding:8px 10px;margin:8px 0;background:rgba(255,255,255,0.03);border:1px dashed var(--border);border-radius:10px;}
.nm-doctest-meta{display:inline-block;margin:8px 0 2px;padding:3px 10px;border:1px solid var(--border);border-radius:999px;color:var(--muted);font-size:12px;background:rgba(255,255,255,0.03);}
.nm-doctest-block{margin:10px 0 12px;}
.nm-doctest-row{display:flex;align-items:flex-start;gap:8px;margin:6px 0;}
.nm-doctest-badge{display:inline-block;min-width:56px;text-align:center;padding:2px 8px;border-radius:999px;border:1px solid var(--border);background:rgba(255,255,255,0.03);color:var(--muted);font-size:11px;line-height:1.5;letter-spacing:.03em;}
.nm-doctest-pre{margin:0;padding:8px 10px;white-space:pre-wrap;word-break:break-word;background:rgba(255,255,255,0.03);border:1px solid var(--border);border-radius:8px;font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:12px;line-height:1.45;flex:1;}
.nm-doctest-inline{padding:2px 8px;border:1px solid var(--border);border-radius:8px;background:rgba(255,255,255,0.03);font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:12px;}
.nm-toggle{display:inline-block;margin:6px 0 12px;padding:6px 10px;border-radius:10px;border:1px solid #2f3f58;background:#0f141b;color:#d6d6d6;cursor:pointer;transition:all 0.2s;}
.nm-toggle:hover{background:#1a202e;}
.nm-hidden{display:none;}
.nm-expand-marker{
  display:block;
  width:100%;
  box-sizing:border-box;
  margin:4px 0;
  padding:2px 8px;
  border-radius:4px;
  background:rgba(122,162,247,0.15);
  color:var(--accent);
  font-size:11px;
  cursor:pointer;
  user-select:none;
  border:1px solid rgba(122,162,247,0.3);
}
.nm-expand-marker:hover{
  background:rgba(122,162,247,0.25);
}
.nm-runnable{cursor:pointer;position:relative;}
.nm-runnable::after{
  content:"Click to run";
  position:absolute;
  right:10px;
  top:8px;
  font-size:11px;
  color:var(--muted);
  background:rgba(0,0,0,.35);
  border:1px solid var(--border);
  border-radius:8px;
  padding:2px 6px;
}
#play-overlay{
  position:fixed; inset:0; background:rgba(0,0,0,.55);
  display:none; align-items:center; justify-content:center; z-index:999900;
}
#play-overlay.open{display:flex;}
#play-modal{
  width:min(1100px,95vw); height:min(760px,92vh);
  background:var(--card); border:1px solid var(--border); border-radius:12px;
  display:grid; grid-template-rows:auto 1fr auto; overflow:hidden;
}
#play-head,#play-foot{display:flex; align-items:center; gap:8px; padding:10px 12px; border-bottom:1px solid var(--border);}
#play-foot{border-bottom:none; border-top:1px solid var(--border);}
#play-title{font-weight:600; flex:1;}
.play-btn{padding:6px 10px; border-radius:8px; border:1px solid var(--border); background:#0f141b; color:var(--fg); cursor:pointer;transition:all 0.2s;}
.play-btn:hover{border-color:#355186;}
@keyframes nm-spin {
  to { transform: rotate(360deg); }
}
.play-btn.running {
  color: var(--accent);
  border-color: var(--accent);
  cursor: wait;
  padding-right: 32px;
  position: relative;
}
.play-btn.running::after {
  content: "";
  position: absolute;
  right: 10px;
  top: 50%;
  margin-top: -7px;
  width: 14px;
  height: 14px;
  border: 2px solid var(--accent);
  border-top-color: transparent;
  border-radius: 50%;
  animation: nm-spin 1s linear infinite;
}
#play-editor{
  display:grid; grid-template-columns:1fr 40%;
  min-height:0;
}
#play-src,#play-stdin,#play-stdout-raw{
  width:100%; height:100%; resize:none; box-sizing:border-box;
  font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;
  font-size:13px; line-height:1.45; border:none; outline:none; color:var(--fg); background:#0b1322;
  padding:12px;
}
#play-right{display:grid; grid-template-rows:120px 1fr; border-left:1px solid var(--border); min-height:0;}
#play-stdin{background:#0a1620; border-bottom:1px solid var(--border);}
#play-stdin-wrap{position:relative; min-height:0;}
#play-stdin{padding-top:30px;}
#play-stdout-wrap{background:#081018; min-height:0; position:relative;}
#play-stdout-view{
  margin:0;
  height:100%;
  overflow:auto;
  white-space:pre-wrap;
  word-break:break-word;
  box-sizing:border-box;
  padding:30px 12px 12px;
  font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;
  font-size:13px; line-height:1.45;
}
#play-stdout-raw{
  position:absolute;
  inset:0;
  opacity:0;
  pointer-events:none;
}
#play-status{font-size:12px; color:var(--muted);}
.ok{color:var(--ok);} .err{color:var(--err);}
.io-label{
  position:absolute;
  left:10px;
  top:8px;
  font-size:11px;
  letter-spacing:.04em;
  color:var(--muted);
  border:1px solid var(--border);
  border-radius:999px;
  padding:2px 8px;
  background:rgba(0,0,0,.25);
  z-index:2;
}
.doc-sidebar{
  position:sticky;
  top:0;
  height:100vh;
  background:var(--bg);
  border:1px solid var(--border);
  border-left:none;
  border-radius:0;
  padding:16px;
  overflow-y:auto;
  box-sizing:border-box;
  scrollbar-width:thin;
  scrollbar-color:#425779 #121a2a;
}
.doc-sidebar::-webkit-scrollbar{width:10px;height:10px;}
.doc-sidebar::-webkit-scrollbar-track{background:#121a2a;border-radius:8px;}
.doc-sidebar::-webkit-scrollbar-thumb{background:#425779;border-radius:8px;border:2px solid #121a2a;}
.doc-sidebar::-webkit-scrollbar-thumb:hover{background:#5a76a8;}
.sidebar-header{
  display:flex;
  align-items:center;
  justify-content:space-between;
  margin-bottom:8px;
}
.toc-title{
  font-size:12px;
  letter-spacing:.04em;
  color:var(--muted);
  margin:2px 0;
}
.mobile-header{
  display:none;
  align-items:center;
  gap:12px;
  padding:10px 16px;
  border-bottom:1px solid var(--border);
  background:var(--bg);
  position:sticky;
  top:0;
  z-index:900;
}
.sidebar-toggle{
  background:transparent;
  border:1px solid var(--border);
  color:var(--fg);
  font-size:18px;
  padding:2px 8px;
  cursor:pointer;
  border-radius:6px;
  line-height:1;
}
.sidebar-toggle:hover{
  background:rgba(255,255,255,0.05);
}
.toc-list{list-style:none;margin:0;padding:0;display:flex;flex-direction:column;gap:4px;}
.toc-group{
  color:var(--muted);
  font-size:12px;
  letter-spacing:.02em;
  margin-top:8px;
  padding:4px 8px;
}
.toc-link{
  display:block;
  padding:6px 8px;
  border-radius:8px;
  color:var(--fg);
  text-decoration:none;
  border:1px solid transparent;
  font-size:13px;
  transition:all 0.2s;
}
.toc-link:hover{border-color:var(--border);background:rgba(255,255,255,0.04);}
.toc-link.active{border-color:#355186;background:rgba(122,162,247,0.18);}
.depth-1{padding-left:14px;}
.depth-2{padding-left:24px;}
.depth-3{padding-left:34px;}
.depth-4{padding-left:44px;}

/* サイドバーオーバーレイ（モバイル用） */
.sidebar-overlay{
  display:none;
  position:fixed;
  inset:0;
  background:rgba(0,0,0,0.6);
  z-index:999;
}

/* スマホ対応（768px以下） */
@media (max-width: 768px){
  .doc-layout{
    display:block;
  }
  .mobile-header{
    display:flex;
  }
  .global-play-link.pc-only{
    display:none;
  }
  
  .doc-sidebar{
    position:fixed;
    top:0;
    left:0;
    bottom:0;
    width:280px;
    z-index:1000;
    transform:translateX(-100%);
    transition:transform 0.3s ease;
    border-right:1px solid var(--border);
  }
  
  .doc-sidebar.mobile-open{
    transform:translateX(0);
  }
  
  .sidebar-overlay.mobile-open{
    display:block;
  }
  
  main{
    padding:16px;
  }
  
  /* モーダルの調整 */
  #play-modal{
    width:100%;
    height:100%;
    max-height:100vh;
    border-radius:0;
  }
  
  #play-editor{
    grid-template-columns:1fr;
    grid-template-rows:50% 50%;
  }
  
  #play-right{
    border-left:none;
    border-top:1px solid var(--border);
  }
}

/* スマホ対応（第2段階：480px以下） */
@media (max-width: 480px){
  .doc-layout{
    margin:12px auto;
    padding:0 8px;
  }
  
  .nm-code{
    padding:10px;
    font-size:12px;
  }
  
  h1{font-size:1.6em;}
  h2{font-size:1.4em;}
  h3{font-size:1.2em;}
  
  #play-head,#play-foot{
    padding:8px 10px;
    gap:6px;
  }
  
  #play-title{
    font-size:13px;
  }
  
  .play-btn{
    padding:5px 8px;
    font-size:12px;
  }
  
  #play-src,#play-stdin,#play-stdout-raw{
    font-size:12px;
    padding:10px;
  }
  
  #play-editor{
    grid-template-rows:45% 55%;
  }
  
  #play-right{
    grid-template-rows:100px 1fr;
  }
}
</style>
<script>
function nmExpandHidden(marker, nodes){
  marker.style.display = 'none';
  for(const n of nodes){
    n.style.display = 'inline';
  }
}

async function loadBindings() {
  if (window.wasmBindings && typeof window.wasmBindings.compile_source === 'function') {
    return window.wasmBindings;
  }
  const modUrl = new URL('${escapeHtml(moduleJsPath)}', location.href).toString();
  const mod = await import(modUrl);
  if (typeof mod.default === 'function') {
    await mod.default();
  }
  window.wasmBindings = mod;
  return mod;
}

const __TUTORIAL_VFS_OVERRIDES__ = ${vfsOverridesJson};

function makeWorkerScript() {
  return \`
self.onmessage = async (e) => {
  const { wasmBytes, stdinText } = e.data;
  let memory = null;
  let stdinOffset = 0;
  const stdin = new TextEncoder().encode(stdinText || '');
  const decoder = new TextDecoder();
  function toHex(bytes) {
    let out = '';
    for (let i = 0; i < bytes.length; i++) {
      const h = bytes[i].toString(16).padStart(2, '0');
      out += (i === 0 ? '' : ' ') + h;
    }
    return out;
  }
  const wasi = {
    fd_write(fd, iovs, iovs_len, nwritten){
      if(!memory) return 5;
      const view = new DataView(memory.buffer);
      if(fd !== 1 && fd !== 2){
        view.setUint32(nwritten, 0, true);
        return 8; // BADF
      }
      let total = 0;
      for(let i=0;i<iovs_len;i++){
        const ptr = view.getUint32(iovs + i*8, true);
        const len = view.getUint32(iovs + i*8 + 4, true);
        if (ptr >= memory.buffer.byteLength) continue;
        const maxLen = memory.buffer.byteLength - ptr;
        const take = Math.min(len, maxLen);
        const bytes = new Uint8Array(memory.buffer, ptr, take);
        const text = decoder.decode(bytes, { stream: true });
        const bytesHex = toHex(bytes);
        self.postMessage({type:'stdout_bytes', fd, bytesHex, len: take});
        if (text.length > 0) {
          self.postMessage({type:'stdout', fd, text});
        }
        total += take;
      }
      view.setUint32(nwritten, total, true);
      return 0;
    },
    fd_read(fd, iovs, iovs_len, nread){
      if(!memory) return 5;
      const view = new DataView(memory.buffer);
      if(fd !== 0){
        view.setUint32(nread, 0, true);
        return 8; // BADF
      }
      let read = 0;
      for(let i=0;i<iovs_len;i++){
        const ptr = view.getUint32(iovs + i*8, true);
        const len = view.getUint32(iovs + i*8 + 4, true);
        if (ptr >= memory.buffer.byteLength) continue;
        const maxLen = memory.buffer.byteLength - ptr;
        const cap = Math.min(len, maxLen);
        const remain = stdin.length - stdinOffset;
        const take = Math.min(cap, Math.max(0, remain));
        if (take > 0) {
          new Uint8Array(memory.buffer, ptr, take).set(stdin.subarray(stdinOffset, stdinOffset + take));
          stdinOffset += take;
          read += take;
        }
      }
      view.setUint32(nread, read, true);
      return 0;
    },
    fd_close(){ return 0; }, fd_seek(){ return 0; }, fd_fdstat_get(){ return 0; },
    environ_get(){ return 0; }, environ_sizes_get(){ return 0; },
    args_get(){ return 0; }, args_sizes_get(){ return 0; },
    clock_time_get(){ return 0; }, random_get(){ return 0; },
    proc_exit(code){ throw new Error('proc_exit:' + code); }
  };
  try {
    const { instance } = await WebAssembly.instantiate(wasmBytes, { wasi_snapshot_preview1: wasi });
    memory = instance.exports.memory;
    if (instance.exports._start) instance.exports._start();
    else if (instance.exports.main) instance.exports.main();
    const tail = decoder.decode();
    if (tail.length > 0) {
      self.postMessage({ type:'stdout', fd: 1, text: tail });
    }
    self.postMessage({ type: 'done' });
  } catch (err) {
    const tail = decoder.decode();
    if (tail.length > 0) {
      self.postMessage({ type:'stdout', fd: 1, text: tail });
    }
    self.postMessage({ type: 'error', message: String(err && err.message || err) });
  }
};
\`;
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

const ansiRegex = /\\x1b\\[([0-9;]*)m/g;
const ansiMap = {
  '0': '</span>',
  '1': '<span style="font-weight:bold">',
  '30': '<span style="color:#3a3f4b">', '31': '<span style="color:#ff6b6b">',
  '32': '<span style="color:#59c37a">', '33': '<span style="color:#e0af68">',
  '34': '<span style="color:#7aa2f7">', '35': '<span style="color:#bb9af7">',
  '36': '<span style="color:#73daca">', '37': '<span style="color:#c0caf5">',
  '90': '<span style="color:#6b7280">', '91': '<span style="color:#ff757f">',
  '92': '<span style="color:#6dd697">', '93': '<span style="color:#e7b970">',
  '94': '<span style="color:#8fb3ff">', '95': '<span style="color:#c9b1ff">',
  '96': '<span style="color:#8ae6d8">', '97': '<span style="color:#dde1e6">',
};

function ansiToHtml(text) {
  const esc = escapeHtml(text);
  let out = '';
  let lastIndex = 0;
  let match;
  ansiRegex.lastIndex = 0;
  while ((match = ansiRegex.exec(esc)) !== null) {
    out += esc.slice(lastIndex, match.index);
    const codes = match[1].split(';');
    for (const c of codes) {
      if (ansiMap[c]) out += ansiMap[c];
    }
    lastIndex = ansiRegex.lastIndex;
  }
  out += esc.slice(lastIndex);
  return out;
}

function highlightArticleNeplBlocks() {
  const kwds = new Set([
    'fn','let','mut','set','if','then','else','cond','while','do',
    'break','continue','return','match','case','import','export',
    'type','struct','enum','trait','impl','for','in','as','use',
    'pub','mod','const','static','unsafe','async','await','yield',
    'block','tuple'
  ]);
  const types = new Set([
    'i32','i64','u32','u64','u8','f32','f64','bool','str','char','void','unit',
    'Result','Option','Vec','String','StringBuilder','List','Set','Self'
  ]);
  const constants = new Set([
    'true','false','Ok','Err','Some','None'
  ]);
  const builtins = new Set([
    // Core Math
    'add','sub','mul','div','div_s','div_u','mod','mod_s','rem_s','rem_u',
    'eq','ne','lt','le','gt','ge','and','or','not','xor',
    'i32_add','i32_sub','i32_mul','i32_div_s','i32_div_u','i32_rem_s','i32_rem_u',
    'i32_and','i32_or','i32_xor','i32_shl','i32_shr_s','i32_shr_u',
    'i32_clz','i32_ctz','i32_popcnt','i32_eq','i32_ne','i32_lt_s','i32_lt_u',
    'i32_le_s','i32_le_u','i32_gt_s','i32_gt_u','i32_ge_s','i32_ge_u',
    'i64_add','i64_sub','i64_mul','i64_div_s','i64_div_u','i64_rem_s','i64_rem_u',
    'i64_and','i64_or','i64_xor','i64_shl','i64_shr_s','i64_shr_u',
    'i64_extend_i32_s','i64_extend_i32_u',
    'f32_add','f32_sub','f32_mul','f32_div','f32_sqrt','f32_abs','f32_neg',
    'f32_ceil','f32_floor','f32_trunc','f32_nearest','f32_min','f32_max',
    'f32_copysign','f32_eq','f32_ne','f32_lt','f32_le','f32_gt','f32_ge',
    'f64_add','f64_sub','f64_mul','f64_div','f64_sqrt','f64_abs','f64_neg',
    'f64_ceil','f64_floor','f64_trunc','f64_nearest','f64_min','f64_max',
    'f64_copysign','f64_eq','f64_ne','f64_lt','f64_le','f64_gt','f64_ge',
    // Memory
    'load','store','load_i32','store_i32','load_u8','store_u8',
    'alloc','dealloc','realloc',
    // IO
    'print','println','print_i32','println_i32','read_line','read_all',
    // Cast
    'cast','bitcast','from_i32','to_i32',
    // Test
    'assert','assert_eq_i32','test_checked','test_fail',
    // Stdlib - Vec
    'vec_new','vec_push','vec_get','vec_len','vec_cap','vec_is_empty',
    'vec_set','vec_pop','vec_clear','vec_free',
    // Stdlib - String
    'len','concat','concat3','str_eq','str_slice','str_trim','str_split',
    'str_starts_with','str_ends_with','string_builder_new','sb_append',
    'sb_append_i32','sb_build',
    // Stdlib - Option/Result
    'some','none','is_some','is_none','unwrap','unwrap_or','option_map',
    'ok','err','is_ok','is_err','unwrap_ok','unwrap_err','result_context',
    // Stdlib - Collections
    'hashmap_new','hashmap_insert','hashmap_get','hashmap_contains','hashmap_remove','hashmap_len','hashmap_free',
    'hashset_new','hashset_insert','hashset_contains','hashset_remove','hashset_len','hashset_free',
    'btreemap_new','btreemap_insert','btreemap_get','btreemap_contains','btreemap_remove','btreemap_len','btreemap_clear','btreemap_free',
    'btreeset_new','btreeset_insert','btreeset_contains','btreeset_remove','btreeset_len','btreeset_clear','btreeset_free',
    'list_nil','list_cons','list_head','list_tail','list_len','list_get','list_free','list_reverse',
    'stack_new','stack_push','stack_pop','stack_peek','stack_len','stack_is_empty','stack_clear','stack_free',
    // Stdlib - KP
    'scanner_new','scanner_read_i32','scanner_read_f64','scanner_read_f32',
    'writer_new','writer_write_i32','writer_write_i64','writer_write_f64_ln',
    'writer_write_f32_ln','writer_write_str','writer_writeln','writer_flush','writer_free'
  ]);

  function hl(code) {
    const lines = String(code || '').split('\\n');
    return lines.map(ln => {
      let out = '';
      let i = 0;
      while (i < ln.length) {
        if (ln[i] === '/' && ln[i+1] === '/') {
          out += '<span class="nm-syn-comment">' + esc(ln.slice(i)) + '</span>';
          break;
        }
        if (ln[i] === '"') {
          const j = ln.indexOf('"', i + 1);
          const s = (j < 0) ? ln.slice(i) : ln.slice(i, j + 1);
          out += '<span class="nm-syn-string">' + esc(s) + '</span>';
          i += s.length;
          continue;
        }
        if (ln[i] === '#') {
            let j = i + 1;
            while (j < ln.length && /[a-zA-Z0-9_]/.test(ln[j])) j++;
            const tok = ln.slice(i, j);
            if (j > i + 1) {
                 out += '<span class="nm-syn-keyword">' + esc(tok) + '</span>';
                 i = j;
                 continue;
            }
        }
        if (/[0-9]/.test(ln[i])) {
          let j = i;
          if (ln[i] === '0' && (ln[i+1] === 'x' || ln[i+1] === 'X')) {
             j += 2;
             while (j < ln.length && /[0-9a-fA-F]/.test(ln[j])) j++;
          } else {
             while (j < ln.length && /[0-9.]/.test(ln[j])) j++;
          }
          out += '<span class="nm-syn-number">' + esc(ln.slice(i, j)) + '</span>';
          i = j;
          continue;
        }
        if (/[a-zA-Z_]/.test(ln[i])) {
          let j = i;
          while (j < ln.length && /[a-zA-Z0-9_]/.test(ln[j])) j++;
          const tok = ln.slice(i, j);
          if (kwds.has(tok)) {
            out += '<span class="nm-syn-keyword">' + esc(tok) + '</span>';
          } else if (types.has(tok)) {
            out += '<span class="nm-syn-type">' + esc(tok) + '</span>';
          } else if (constants.has(tok)) {
            out += '<span class="nm-syn-constant">' + esc(tok) + '</span>';
          } else if (builtins.has(tok)) {
            out += '<span class="nm-syn-function">' + esc(tok) + '</span>';
          } else {
            out += esc(tok);
          }
          i = j;
          continue;
        }
        if ('<>()[]{}:,;'.includes(ln[i])) {
          out += '<span class="nm-syn-punctuation">' + esc(ln[i]) + '</span>';
          i++;
          continue;
        }
        if ('+-*/%=!&|'.includes(ln[i])) {
          out += '<span class="nm-syn-operator">' + esc(ln[i]) + '</span>';
          i++;
          continue;
        }
        out += esc(ln[i]);
        i++;
      }
      return out;
    }).join('\\n');
  }

  function esc(s) {
    return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
  }

  for (const code of document.querySelectorAll('pre.nm-code > code.language-neplg2')) {
    const nodes = Array.from(code.childNodes);
    const frag = document.createDocumentFragment();
    for (const node of nodes) {
      if (node.nodeType === 3) {
        const span = document.createElement('span');
        span.innerHTML = hl(node.textContent);
        while (span.firstChild) frag.appendChild(span.firstChild);
      } else if (node.nodeType === 1 && node.classList.contains('nm-hidden')) {
        const span = document.createElement('span');
        span.className = 'nm-hidden';
        span.style.display = 'none';
        span.innerHTML = hl(node.textContent);
        frag.appendChild(span);
      } else {
        frag.appendChild(node.cloneNode(true));
      }
    }
    code.innerHTML = '';
    code.appendChild(frag);
  }
}

function findDoctestStdinFor(codeContent) {
  const wrapper = codeContent.closest('.nm-code-container');
  if (!wrapper) return '';
  const footer = wrapper.querySelector('.nm-code-footer');
  if (footer) {
    for (const row of footer.querySelectorAll('.nm-doctest-row')) {
      const badge = row.querySelector('.nm-doctest-badge');
      const pre2 = row.querySelector('.nm-doctest-pre');
      if (badge && pre2 && badge.textContent.trim().toLowerCase() === 'stdin') {
        return pre2.textContent || '';
      }
    }
  }
  return '';
}

document.addEventListener('DOMContentLoaded', () => {
  highlightArticleNeplBlocks();

  for (const code of document.querySelectorAll('pre.nm-code > code')) {
    const hiddenNodes = Array.from(code.querySelectorAll('.nm-hidden'));
    if (hiddenNodes.length === 0) continue;

    // Group consecutive hidden nodes
    let groups = [];
    let currentGroup = [];
    for (const node of hiddenNodes) {
      // Simple grouping: if nodes are adjacent in DOM or separated only by whitespace text
      if (currentGroup.length > 0) {
        const last = currentGroup[currentGroup.length - 1];
        if (last.nextSibling === node || (last.nextSibling && last.nextSibling.nodeType === 3 && !last.nextSibling.textContent.trim() && last.nextSibling.nextSibling === node)) {
          currentGroup.push(node);
        } else {
          groups.push(currentGroup);
          currentGroup = [node];
        }
      } else {
        currentGroup.push(node);
      }
    }
    if (currentGroup.length > 0) groups.push(currentGroup);

    for (const group of groups) {
      if (group.length === 1) {
        group[0].style.display = 'inline';
        continue;
      }
      const first = group[0];
      const marker = document.createElement('span');
      marker.className = 'nm-expand-marker';
      marker.textContent = '[expand+]';
      marker.title = '省略を展開';
      marker.onclick = () => nmExpandHidden(marker, group);
      first.parentNode.insertBefore(marker, first);
    }
  }

  const overlay = document.getElementById('play-overlay');
  const title = document.getElementById('play-title');
  const src = document.getElementById('play-src');
  const stdin = document.getElementById('play-stdin');
  const stdoutRaw = document.getElementById('play-stdout-raw');
  const stdoutView = document.getElementById('play-stdout-view');
  const status = document.getElementById('play-status');
  const runBtn = document.getElementById('play-run');
  const stopBtn = document.getElementById('play-stop');
  const closeBtn = document.getElementById('play-close');
  let worker = null;
  let running = false;
  let stdoutText = '';
  let stdoutHexLines = [];

  function setStdoutText(next) {
    stdoutText = String(next || '');
    stdoutRaw.value = stdoutText;
    stdoutView.innerHTML = ansiToHtml(stdoutText);
    stdoutView.scrollTop = stdoutView.scrollHeight;
  }

  function setStatus(text, cls) {
    status.className = cls || '';
    status.textContent = text;
  }

  function setRunningState(active) {
    running = active;
    if (active) {
      runBtn.textContent = 'Running';
      runBtn.classList.add('running');
    } else {
      runBtn.textContent = 'Run';
      runBtn.classList.remove('running');
    }
  }

  function stopRun(message) {
    if (worker) {
      worker.terminate();
      worker = null;
    }
    setRunningState(false);
    if (message) setStatus(message, 'err');
  }

  runBtn.onclick = async () => {
    if (running) return;
    setRunningState(true);
    setStdoutText('');
    stdoutHexLines = [];
    setStatus('compiling...', '');
    console.log('[Tutorial Runner] source:\\n' + (src.value || ''));
    console.log('[Tutorial Runner] stdin:\\n' + (stdin.value || ''));
    try {
      const bindings = await loadBindings();
      if (!running) return; // stopped during load
      let wasmBytes = null;
      if (typeof bindings.compile_source_with_vfs === 'function') {
        wasmBytes = bindings.compile_source_with_vfs('/virtual/entry.nepl', src.value, __TUTORIAL_VFS_OVERRIDES__);
        console.log('[Tutorial Runner] compile_source_with_vfs overrides:', Object.keys(__TUTORIAL_VFS_OVERRIDES__).length);
      } else {
        wasmBytes = bindings.compile_source(src.value);
        console.log('[Tutorial Runner] compile_source (no vfs override API)');
      }
      setStatus('running...', '');
      const blob = new Blob([makeWorkerScript()], { type: 'text/javascript' });
      worker = new Worker(URL.createObjectURL(blob));
      worker.onmessage = (ev) => {
        const msg = ev.data || {};
        if (msg.type === 'stdout') {
          setStdoutText(stdoutText + String(msg.text || ''));
        } else if (msg.type === 'stdout_bytes') {
          const line = '[len=' + String(msg.len || 0) + '] ' + String(msg.bytesHex || '');
          stdoutHexLines.push(line);
          console.log('[Tutorial Runner] stdout bytes:', line);
        } else if (msg.type === 'done') {
          setRunningState(false);
          setStatus('done', 'ok');
          console.log('[Tutorial Runner] stdout:\\n' + stdoutText);
          console.log('[Tutorial Runner] stdout bytes all:\\n' + stdoutHexLines.join('\\n'));
          worker && worker.terminate();
          worker = null;
        } else if (msg.type === 'error') {
          setRunningState(false);
          setStatus('runtime error', 'err');
          setStdoutText(stdoutText + '\\n[error] ' + String(msg.message || ''));
          console.log('[Tutorial Runner] runtime error:', String(msg.message || ''));
          console.log('[Tutorial Runner] stdout (partial):\\n' + stdoutText);
          console.log('[Tutorial Runner] stdout bytes (partial):\\n' + stdoutHexLines.join('\\n'));
          worker && worker.terminate();
          worker = null;
        }
      };
      worker.postMessage({ wasmBytes, stdinText: stdin.value || '' });
    } catch (e) {
      setRunningState(false);
      setStatus('compile failed', 'err');
      setStdoutText(stdoutText + '[compile error] ' + String((e && e.message) || e));
      console.log('[Tutorial Runner] compile failed:', String((e && e.message) || e));
      console.log('[Tutorial Runner] stdout (partial):\\n' + stdoutText);
    }
  };

  stopBtn.onclick = () => {
    if (!running) return;
    stopRun('interrupted');
  };

  closeBtn.onclick = () => {
    stopRun('');
    overlay.classList.remove('open');
  };

  overlay.addEventListener('click', (ev) => {
    if (ev.target === overlay) {
      closeBtn.onclick();
    }
  });

  // Add Run buttons to neplg2 code blocks
  for (const code of document.querySelectorAll('pre.nm-code > code.language-neplg2')) {
    const pre = code.parentElement;
    const container = pre.parentElement; // nm-code-content or just a wrapper
    const btn = document.createElement('button');
    btn.className = 'nm-run-btn';
    btn.textContent = '▶ Run';
    btn.title = 'Run in playground';
    btn.onclick = () => {
      title.textContent = document.title + ' - runnable snippet';
      let text = '';
      for (const node of code.childNodes) {
        if (node.nodeType === 1 && node.classList.contains('nm-expand-marker')) {
          continue;
        }
        text += node.textContent;
      }
      src.value = text;
      stdin.value = findDoctestStdinFor(container);
      setStdoutText('');
      setStatus('ready', 'ok');
      overlay.classList.add('open');
      src.focus();
    };
    // If inside nm-code-content, append there. Otherwise (standalone code block), wrap it.
    if (container.classList.contains('nm-code-content')) {
      container.appendChild(btn);
    } else {
      // Standalone code block case (not in doctest wrapper)
      const wrapper = document.createElement('div');
      wrapper.className = 'nm-code-content';
      wrapper.style.position = 'relative';
      pre.parentNode.insertBefore(wrapper, pre);
      wrapper.appendChild(pre);
      wrapper.appendChild(btn);
    }
  }

  const sidebar = document.querySelector('.doc-sidebar');
  const activeLink = sidebar ? sidebar.querySelector('.toc-link.active') : null;
  if (sidebar && activeLink) {
    const sidebarRect = sidebar.getBoundingClientRect();
    const activeRect = activeLink.getBoundingClientRect();
    const currentTop = sidebar.scrollTop;
    const activeTopInSidebar = activeRect.top - sidebarRect.top + currentTop;
    const targetTop = activeTopInSidebar - (sidebar.clientHeight / 2) + (activeRect.height / 2);
    sidebar.scrollTop = Math.max(0, targetTop);
  }
  window.scrollTo(0, 0);
  
  // モバイルサイドバートグル機能
  const sidebarToggle = document.querySelector('.sidebar-toggle');
  const sidebarOverlay = document.createElement('div');
  sidebarOverlay.className = 'sidebar-overlay';
  document.body.appendChild(sidebarOverlay);
  
  function toggleSidebar() {
    if (sidebar) {
      sidebar.classList.toggle('mobile-open');
      sidebarOverlay.classList.toggle('mobile-open');
    }
  }
  
  if (sidebarToggle) {
    sidebarToggle.addEventListener('click', (e) => {
      e.stopPropagation();
      toggleSidebar();
    });
  }
  
  sidebarOverlay.addEventListener('click', toggleSidebar);
  
  // サイドバー内のリンククリック時にサイドバーを閉じる（モバイルのみ）
  if (sidebar) {
    const sidebarLinks = sidebar.querySelectorAll('.toc-link');
    sidebarLinks.forEach(link => {
      link.addEventListener('click', () => {
        if (window.innerWidth <= 768) {
          sidebar.classList.remove('mobile-open');
          sidebarOverlay.classList.remove('mobile-open');
        }
      });
    });
  }
});
</script>
</head>
<body>
<div class="mobile-header">
  <button class="sidebar-toggle" aria-label="メニューを開く">☰</button>
  <div style="font-weight:600;font-size:14px;">${escapeHtml(t)}</div>
  <a class="global-play-link" href="https://neknaj.github.io/NEPLg2/" target="_blank" rel="noopener noreferrer" style="margin-left:auto;">Web Playground</a>
</div>
<a class="global-play-link pc-only" href="https://neknaj.github.io/NEPLg2/" target="_blank" rel="noopener noreferrer">Web Playground</a>
<div class="doc-layout">
${tocHtml}
<main>
${body}
</main>
</div>

<div id="play-overlay">
  <div id="play-modal" role="dialog" aria-modal="true" aria-label="NEPLg2 Runnable Snippet">
    <div id="play-head">
      <div id="play-title">Runnable Snippet</div>
      <button id="play-run" class="play-btn">Run</button>
      <button id="play-stop" class="play-btn">Interrupt</button>
      <button id="play-close" class="play-btn">Close</button>
    </div>
    <div id="play-editor">
      <textarea id="play-src" spellcheck="false"></textarea>
      <div id="play-right">
        <div id="play-stdin-wrap">
          <div class="io-label">Standard Input (stdin)</div>
          <textarea id="play-stdin" spellcheck="false" placeholder="stdin"></textarea>
        </div>
        <div id="play-stdout-wrap">
          <div class="io-label">Program Output (stdout/stderr)</div>
          <pre id="play-stdout-view"></pre>
          <textarea id="play-stdout-raw" spellcheck="false" readonly placeholder="stdout / stderr"></textarea>
        </div>
      </div>
    </div>
    <div id="play-foot">
      <span id="play-status">ready</span>
    </div>
  </div>
</div>
</body>
</html>`;
}

function renderHtmlPlayground(ast, opt) {
    const title = (opt && opt.title) ? opt.title : 'NEPLg2 Tutorial';
    const description = (opt && opt.description)
        ? opt.description
        : 'NEPLg2 tutorial with interactive runnable examples.';
    const moduleJsPath = (opt && opt.moduleJsPath) ? String(opt.moduleJsPath) : './nepl-web.js';
    const tocHtml = renderToc((opt && opt.tocLinks) ? opt.tocLinks : []);
    const body = renderBody(ast);
    return wrapHtmlPlayground(body, title, description, moduleJsPath, tocHtml);
}

module.exports = {
    renderHtmlPlayground,
};