// nodesrc/html_gen_playground.js
// 目的:
// - 既存 html_gen.js は維持したまま、チュートリアル向けの実行可能 HTML を生成する。
// - pre>code(language-neplg2) をクリックすると、ポップアップエディタで Run / Interrupt / 出力確認ができる。

const { renderNode } = require('./html_gen');

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
        const cls = link.active ? ' class="toc-link active"' : ' class="toc-link"';
        return `<li><a${cls} href="${escapeHtml(String(link.href || ''))}">${escapeHtml(String(link.label || ''))}</a></li>`;
    }).join('\n');
    return `<aside class="doc-sidebar"><div class="toc-title">Getting Started</div><ul class="toc-list">${items}</ul></aside>`;
}

function wrapHtmlPlayground(body, title, description, moduleJsPathOpt) {
    const t = title || 'NEPLg2 Tutorial';
    const d = description || 'NEPLg2 tutorial with interactive runnable examples.';
    const moduleJsPath = (moduleJsPathOpt && String(moduleJsPathOpt)) || './nepl-web.js';
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
html,body{background:var(--bg);color:var(--fg);font-family:system-ui,-apple-system,Segoe UI,Roboto,Helvetica,Arial;line-height:1.65;}
.doc-layout{max-width:1260px;margin:24px auto;padding:0 16px;display:grid;grid-template-columns:260px 1fr;gap:18px;}
main{min-width:0;}
a{color:var(--accent);}
hr{border:none;border-top:1px solid var(--border);margin:24px 0;}
.nm-sec{padding:0.5em;padding-left:2em;margin:1em;border-left:3px solid var(--border);border-radius:1em;}
h1,h2,h3,h4,h5,h6{margin:18px 0 10px;}
p{margin:10px 0;}
ul{margin:10px 0 10px 22px;}
.nm-code{background:var(--code);border:1px solid var(--border);border-radius:12px;padding:12px;overflow:auto;}
.nm-code code{font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:13px;white-space:pre;}
.nm-code-inline{background:rgba(255,255,255,0.06);border:1px solid rgba(255,255,255,0.10);border-radius:8px;padding:1px 6px;}
.nm-gloss, .nm-ruby{ruby-position:over;}
.nm-gloss rt{font-size:0.72em;color:var(--muted);line-height:1.1;}
.nm-gloss-note{display:block;}
.math-inline{color:var(--muted);}
.math-display{display:block;padding:8px 10px;margin:8px 0;background:rgba(255,255,255,0.03);border:1px dashed var(--border);border-radius:10px;}
.nm-doctest-meta{display:inline-block;margin:8px 0 2px;padding:3px 10px;border:1px solid var(--border);border-radius:999px;color:var(--muted);font-size:12px;background:rgba(255,255,255,0.03);}
.nm-toggle{display:inline-block;margin:6px 0 12px;padding:6px 10px;border-radius:10px;border:1px solid #2f3f58;background:#0f141b;color:#d6d6d6;cursor:pointer;}
.nm-hidden{display:none;}
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
  display:none; align-items:center; justify-content:center; z-index:9999;
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
.play-btn{padding:6px 10px; border-radius:8px; border:1px solid var(--border); background:#0f141b; color:var(--fg); cursor:pointer;}
.play-btn:hover{border-color:#355186;}
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
#play-stdout-wrap{background:#081018; min-height:0; position:relative;}
#play-stdout-view{
  margin:0;
  height:100%;
  overflow:auto;
  white-space:pre-wrap;
  word-break:break-word;
  box-sizing:border-box;
  padding:12px;
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
.doc-sidebar{
  position:sticky;
  top:16px;
  align-self:start;
  background:var(--card);
  border:1px solid var(--border);
  border-radius:12px;
  padding:10px 10px 12px;
  max-height:calc(100vh - 36px);
  overflow:auto;
}
.toc-title{
  font-size:12px;
  letter-spacing:.04em;
  color:var(--muted);
  margin:2px 0 8px;
}
.toc-list{list-style:none;margin:0;padding:0;display:flex;flex-direction:column;gap:4px;}
.toc-link{
  display:block;
  padding:6px 8px;
  border-radius:8px;
  color:var(--fg);
  text-decoration:none;
  border:1px solid transparent;
  font-size:13px;
}
.toc-link:hover{border-color:var(--border);background:rgba(255,255,255,0.04);}
.toc-link.active{border-color:#355186;background:rgba(122,162,247,0.18);}
@media (max-width: 920px){
  .doc-layout{grid-template-columns:1fr;}
  .doc-sidebar{position:static;max-height:none;}
}
</style>
<script>
function nmToggleHidden(btn){
  const pre = btn.previousElementSibling;
  if(!pre) return;
  const nodes = pre.querySelectorAll('.nm-hidden');
  if(nodes.length === 0) return;
  const cur = nodes[0].style.display;
  const show = (cur === 'none' || cur === '');
  for(const n of nodes){
    n.style.display = show ? 'inline' : 'none';
  }
  btn.textContent = show ? '前置き( | 行)を隠す' : '前置き( | 行)を表示';
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

function makeWorkerScript() {
  return \`
self.onmessage = async (e) => {
  const { wasmBytes, stdinText } = e.data;
  let memory = null;
  let stdinOffset = 0;
  const stdin = new TextEncoder().encode(stdinText || '');
  const wasi = {
    fd_write(fd, iovs, iovs_len, nwritten){
      if(!memory) return 5;
      const view = new DataView(memory.buffer);
      let total = 0;
      for(let i=0;i<iovs_len;i++){
        const ptr = view.getUint32(iovs + i*8, true);
        const len = view.getUint32(iovs + i*8 + 4, true);
        const bytes = new Uint8Array(memory.buffer, ptr, len);
        self.postMessage({type:'stdout', fd, text:new TextDecoder().decode(bytes)});
        total += len;
      }
      view.setUint32(nwritten, total, true);
      return 0;
    },
    fd_read(fd, iovs, iovs_len, nread){
      if(fd !== 0) return 0;
      if(!memory) return 5;
      const view = new DataView(memory.buffer);
      let read = 0;
      for(let i=0;i<iovs_len;i++){
        const ptr = view.getUint32(iovs + i*8, true);
        const len = view.getUint32(iovs + i*8 + 4, true);
        const remain = stdin.length - stdinOffset;
        const take = Math.min(len, Math.max(0, remain));
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
    self.postMessage({ type: 'done' });
  } catch (err) {
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

function ansiColorFg(n) {
  const map = {
    30:'#111111',31:'#ff6b6b',32:'#59c37a',33:'#f7d154',34:'#6da8ff',35:'#d291ff',36:'#62d6e8',37:'#f0f0f0',
    90:'#7a7a7a',91:'#ff8a8a',92:'#7de0a0',93:'#ffe28a',94:'#8ec0ff',95:'#e1b2ff',96:'#87e8f5',97:'#ffffff',
  };
  return map[n] || null;
}

function ansiColorBg(n) {
  const map = {
    40:'#111111',41:'#7a1f1f',42:'#1f5f2f',43:'#6d5a1f',44:'#1f3f7a',45:'#5a2a7a',46:'#1f6170',47:'#cfcfcf',
    100:'#4f4f4f',101:'#a63a3a',102:'#3b8d4f',103:'#9a8136',104:'#3b6bb5',105:'#7b4ab5',106:'#3f8f9e',107:'#f5f5f5',
  };
  return map[n] || null;
}

function ansiToHtml(input) {
  const re = new RegExp(String.fromCharCode(27) + '\\\\[([0-9;]*)m', 'g');
  const state = { bold: false, underline: false, fg: null, bg: null };
  const chunks = [];
  let last = 0;

  function styleText(text) {
    if (!text) return;
    let style = '';
    if (state.bold) style += 'font-weight:700;';
    if (state.underline) style += 'text-decoration:underline;';
    if (state.fg) style += 'color:' + state.fg + ';';
    if (state.bg) style += 'background:' + state.bg + ';';
    if (!style) {
      chunks.push(escapeHtml(text));
      return;
    }
    chunks.push('<span style="' + style + '">' + escapeHtml(text) + '</span>');
  }

  let m;
  while ((m = re.exec(input)) !== null) {
    styleText(input.slice(last, m.index));
    last = re.lastIndex;

    const codes = (m[1] === '' ? ['0'] : m[1].split(';')).map(x => parseInt(x, 10));
    for (const c of codes) {
      if (c === 0) {
        state.bold = false; state.underline = false; state.fg = null; state.bg = null;
      } else if (c === 1) {
        state.bold = true;
      } else if (c === 4) {
        state.underline = true;
      } else if (c === 22) {
        state.bold = false;
      } else if (c === 24) {
        state.underline = false;
      } else if (c === 39) {
        state.fg = null;
      } else if (c === 49) {
        state.bg = null;
      } else {
        const fg = ansiColorFg(c);
        const bg = ansiColorBg(c);
        if (fg) state.fg = fg;
        if (bg) state.bg = bg;
      }
    }
  }
  styleText(input.slice(last));
  return chunks.join('');
}

window.addEventListener('DOMContentLoaded', () => {
  for(const pre of document.querySelectorAll('pre.nm-code')){
    const hasHidden = pre.querySelector('.nm-hidden');
    if(!hasHidden) continue;
    for(const n of pre.querySelectorAll('.nm-hidden')){
      n.style.display = 'none';
    }
    const btn = document.createElement('button');
    btn.className = 'nm-toggle';
    btn.textContent = '前置き( | 行)を表示';
    btn.onclick = () => nmToggleHidden(btn);
    pre.insertAdjacentElement('afterend', btn);
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

  function stopRun(message) {
    if (worker) {
      worker.terminate();
      worker = null;
    }
    running = false;
    if (message) setStatus(message, 'err');
  }

  runBtn.onclick = async () => {
    if (running) return;
    setStdoutText('');
    setStatus('compiling...', '');
    try {
      const bindings = await loadBindings();
      const wasmBytes = bindings.compile_source(src.value);
      setStatus('running...', '');
      const blob = new Blob([makeWorkerScript()], { type: 'text/javascript' });
      worker = new Worker(URL.createObjectURL(blob));
      running = true;
      worker.onmessage = (ev) => {
        const msg = ev.data || {};
        if (msg.type === 'stdout') {
          setStdoutText(stdoutText + String(msg.text || ''));
        } else if (msg.type === 'done') {
          running = false;
          setStatus('done', 'ok');
          worker && worker.terminate();
          worker = null;
        } else if (msg.type === 'error') {
          running = false;
          setStatus('runtime error', 'err');
          setStdoutText(stdoutText + '\\n[error] ' + String(msg.message || ''));
          worker && worker.terminate();
          worker = null;
        }
      };
      worker.postMessage({ wasmBytes, stdinText: stdin.value || '' });
    } catch (e) {
      running = false;
      setStatus('compile failed', 'err');
      setStdoutText(stdoutText + '[compile error] ' + String((e && e.message) || e));
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

  for (const code of document.querySelectorAll('pre.nm-code > code.language-neplg2')) {
    const pre = code.parentElement;
    pre.classList.add('nm-runnable');
    pre.title = 'Click to run in popup editor';
    pre.addEventListener('click', () => {
      title.textContent = document.title + ' - runnable snippet';
      src.value = code.textContent || '';
      stdin.value = '';
      setStdoutText('');
      setStatus('ready', 'ok');
      overlay.classList.add('open');
      src.focus();
    });
  }
});
</script>
</head>
<body>
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
        <textarea id="play-stdin" spellcheck="false" placeholder="stdin"></textarea>
        <div id="play-stdout-wrap">
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
