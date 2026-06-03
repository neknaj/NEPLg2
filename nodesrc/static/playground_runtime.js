// nodesrc/static/playground_runtime.js

/**
 * NEPLg2 Playground Runtime
 * Handles:
 * - Syntax highlighting
 * - Code execution via WebAssembly worker
 * - In-page search UI
 * - UI interactions (popups, sidebars)
 */

export function initPlayground(config) {
  const {
    searchIndex: __SEARCH_INDEX__,
    rootPrefix: __ROOT_PREFIX__,
    vfsOverrides: __TUTORIAL_VFS_OVERRIDES__,
    moduleJsPath,
    tocHtml,
    tocTitle,
    title,
  } = config;

  // --- UI Injection ---
  function injectUI() {
    // Mobile Header
    const mobileHeader = document.createElement('div');
    mobileHeader.className = 'mobile-header';
    mobileHeader.innerHTML = `
      <button class="sidebar-toggle" aria-label="メニューを開く">☰</button>
      <div style="font-weight:600;font-size:14px;">${escapeHtml(title || 'NEPLg2')}</div>
      <a class="global-play-link" href="https://neknaj.github.io/NEPLg2/" target="_blank" rel="noopener noreferrer" style="margin-left:auto;">Web Playground</a>
    `;
    document.body.prepend(mobileHeader);

    // PC Global Play Link
    const pcPlayLink = document.createElement('a');
    pcPlayLink.className = 'global-play-link pc-only';
    pcPlayLink.href = 'https://neknaj.github.io/NEPLg2/';
    pcPlayLink.target = '_blank';
    pcPlayLink.rel = 'noopener noreferrer';
    pcPlayLink.textContent = 'Web Playground';
    document.body.appendChild(pcPlayLink);

    // Sidebar Overlay
    const sidebarOverlay = document.createElement('div');
    sidebarOverlay.className = 'sidebar-overlay';
    document.body.appendChild(sidebarOverlay);

    // Play Overlay (Modal)
    const playOverlay = document.createElement('div');
    playOverlay.id = 'play-overlay';
    playOverlay.innerHTML = `
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
    `;
    document.body.appendChild(playOverlay);

    // Sidebar
    const layout = document.querySelector('.doc-layout');
    if (layout && tocHtml) {
      const aside = document.createElement('aside');
      aside.className = 'doc-sidebar';
      const searchBoxHtml = `
        <div class="sidebar-header">
          <div class="toc-title">${escapeHtml(tocTitle || 'Getting Started')}</div>
        </div>
        <div class="search-wrap" id="doc-search-wrap">
          <div style="display:flex;gap:4px;">
            <select class="search-kind" id="doc-search-kind" aria-label="種類で絞り込み">
              <option value="all">All</option>
              <option value="module">mod</option>
              <option value="struct">str</option>
              <option value="enum">enum</option>
              <option value="trait">trait</option>
              <option value="impl">impl</option>
              <option value="fn">fn</option>
              <option value="let">let</option>
            </select>
            <div style="position:relative;flex:1;">
              <input class="search-input" id="doc-search-input" type="search" placeholder="検索..." autocomplete="off" spellcheck="false" style="width:100%; box-sizing:border-box;"/>
              <button class="search-clear" id="doc-search-clear" aria-label="クリア">×</button>
            </div>
          </div>
          <div style="margin-top:4px;">
            <input class="search-input" id="doc-search-path" type="text" placeholder="Filter by path (e.g. math.nepl)..." autocomplete="off" spellcheck="false" style="width:100%; box-sizing:border-box; font-size:12px;"/>
          </div>
          <div class="search-results" id="doc-search-results" role="listbox"></div>
        </div>
        <ul class="toc-list">${tocHtml}</ul>
      `;
      aside.innerHTML = searchBoxHtml;
      layout.prepend(aside);
    }
  }

  injectUI();

  // --- TOC open/close state (localStorage persistence + auto-open active ancestors) ---
  function initTocState() {
    const STORAGE_KEY = 'nepl-toc-open';
    let state = {};
    try { state = JSON.parse(localStorage.getItem(STORAGE_KEY) || '{}'); } catch {}
    const sidebar = document.querySelector('.doc-sidebar');
    if (!sidebar) return;
    const allDetails = sidebar.querySelectorAll('details.toc-group-details');
    // Apply saved state
    allDetails.forEach(details => {
      if (details.id && details.id in state) {
        details.open = state[details.id];
      }
    });
    // Force-open ancestors of active link
    const active = sidebar.querySelector('.toc-link.active');
    if (active) {
      let el = active.parentElement;
      while (el && el !== sidebar) {
        if (el.tagName === 'DETAILS') el.open = true;
        el = el.parentElement;
      }
    }
    // Persist on toggle
    allDetails.forEach(details => {
      details.addEventListener('toggle', () => {
        if (!details.id) return;
        state[details.id] = details.open;
        try { localStorage.setItem(STORAGE_KEY, JSON.stringify(state)); } catch {}
      });
    });
  }
  initTocState();

  // --- Utility functions ---

  function nmExpandHidden(marker, nodes) {
    marker.style.display = 'none';
    for (const n of nodes) {
      n.style.display = 'inline';
    }
  }

  async function loadBindings() {
    if (window.wasmBindings && typeof window.wasmBindings.compile_source === 'function') {
      return window.wasmBindings;
    }
    const modUrl = new URL(moduleJsPath, location.href).toString();
    const mod = await import(modUrl);
    if (typeof mod.default === 'function') {
      await mod.default();
    }
    window.wasmBindings = mod;
    return mod;
  }

  let tutorialCompilerSession = null;

  function compilerApiForRun(bindings) {
    if (!tutorialCompilerSession && typeof bindings.CompilerSession === 'function') {
      tutorialCompilerSession = new bindings.CompilerSession();
    }
    return tutorialCompilerSession || bindings;
  }

  function compileRunnableSnippet(bindings, sourceText) {
    const compilerApi = compilerApiForRun(bindings);
    if (typeof compilerApi.compile_source_with_vfs_and_profile === 'function') {
      return compilerApi.compile_source_with_vfs_and_profile('/virtual/entry.nepl', sourceText, {}, 'debug');
    }
    if (typeof bindings.compile_source_with_vfs_and_profile === 'function') {
      return bindings.compile_source_with_vfs_and_profile('/virtual/entry.nepl', sourceText, {}, 'debug');
    }
    if (typeof bindings.compile_source_with_vfs_and_stdlib === 'function' && typeof bindings.get_bundled_stdlib_vfs === 'function') {
      const stdlibVfs = bindings.get_bundled_stdlib_vfs();
      return bindings.compile_source_with_vfs_and_stdlib('/virtual/entry.nepl', sourceText, {}, stdlibVfs);
    }
    if (typeof bindings.compile_source === 'function') {
      return bindings.compile_source(sourceText);
    }
    return null;
  }

  function makeWorkerScript() {
    return `
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
      let readCount = 0;
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
          readCount += take;
        }
      }
      view.setUint32(nread, readCount, true);
      return 0;
    },
    fd_close(){ return 0; }, fd_seek(){ return 0; }, fd_fdstat_get(){ return 0; },
    environ_get(){ return 0; }, environ_sizes_get(){ return 0; },
    args_get(){ return 0; }, args_sizes_get(){ return 0; },
    clock_time_get(){ return 0; }, random_get(){ return 0; },
    proc_exit(code){ throw new Error('proc_exit:' + code); },
    // Filesystem stubs — not supported in browser; return ENOTSUP (52)
    path_open(){ return 52; },
    fd_prestat_get(){ return 8; },
    fd_prestat_dir_name(){ return 8; },
    path_filestat_get(){ return 52; },
    path_create_directory(){ return 52; },
    path_remove_directory(){ return 52; },
    path_unlink_file(){ return 52; },
    path_rename(){ return 52; },
    fd_readdir(){ return 52; },
  };
  const log = (msg) => self.postMessage({ type: 'log', message: msg });
  try {
    log('Worker started. WASM bytes: ' + (wasmBytes ? wasmBytes.byteLength : 'null'));
    const { instance } = await WebAssembly.instantiate(wasmBytes, { wasi_snapshot_preview1: wasi });
    memory = instance.exports.memory;
    if (instance.exports._start) {
      instance.exports._start();
    } else if (instance.exports.main) {
      instance.exports.main();
    }
    const tail = decoder.decode();
    if (tail.length > 0) {
      self.postMessage({ type:'stdout', fd: 1, text: tail });
    }
    self.postMessage({ type: 'done' });
  } catch (err) {
    if (err && err.message && err.message.startsWith('proc_exit:')) {
      const exitCode = err.message.split(':')[1];
      log('proc_exit called with code: ' + exitCode);
      const tail = decoder.decode();
      if (tail.length > 0) {
        self.postMessage({ type:'stdout', fd: 1, text: tail });
      }
      if (exitCode === '0') {
        self.postMessage({ type: 'done' });
      } else {
        self.postMessage({ type: 'error', message: 'Exit with code ' + exitCode });
      }
      return;
    }
    log('Worker error: ' + String(err && err.message || err));
    const tail = decoder.decode();
    if (tail.length > 0) {
      self.postMessage({ type:'stdout', fd: 1, text: tail });
    }
    self.postMessage({ type: 'error', message: String(err && err.message || err) });
  }
};
`;
  }

  function escapeHtml(s) {
    return String(s)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

  const ansiRegex = /\x1b\[([0-9;]*)m/g;
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
      'fn', 'let', 'mut', 'set', 'if', 'then', 'else', 'cond', 'while', 'do',
      'break', 'continue', 'return', 'match', 'case', 'import', 'export',
      'type', 'struct', 'enum', 'trait', 'impl', 'for', 'in', 'as', 'use',
      'pub', 'mod', 'const', 'static', 'unsafe', 'async', 'await', 'yield',
      'block', 'tuple'
    ]);
    const types = new Set([
      'i32', 'i64', 'u32', 'u64', 'u8', 'f32', 'f64', 'bool', 'str', 'char',
      'String', 'Self'
    ]);
    const typeConstructors = new Set([
      'Result', 'Option', 'Vec', 'StringBuilder', 'List', 'Set'
    ]);
    const unitLiterals = new Set(['unit']);
    const voidLiterals = new Set(['void']);
    const constants = new Set([
      'true', 'false', 'Ok', 'Err', 'Some', 'None'
    ]);
    const builtins = new Set([
      'add', 'sub', 'mul', 'div', 'div_s', 'div_u', 'mod', 'mod_s', 'rem_s', 'rem_u',
      'eq', 'ne', 'lt', 'le', 'gt', 'ge', 'and', 'or', 'not', 'xor',
      'i32_add', 'i32_sub', 'i32_mul', 'i32_div_s', 'i32_div_u', 'i32_rem_s', 'i32_rem_u',
      'i32_and', 'i32_or', 'i32_xor', 'i32_shl', 'i32_shr_s', 'i32_shr_u',
      'i32_clz', 'i32_ctz', 'i32_popcnt', 'i32_eq', 'i32_ne', 'i32_lt_s', 'i32_lt_u',
      'i32_le_s', 'i32_le_u', 'i32_gt_s', 'i32_gt_u', 'i32_ge_s', 'i32_ge_u',
      'i64_add', 'i64_sub', 'i64_mul', 'i64_div_s', 'i64_div_u', 'i64_rem_s', 'i64_rem_u',
      'i64_and', 'i64_or', 'i64_xor', 'i64_shl', 'i64_shr_s', 'i64_shr_u',
      'i64_extend_i32_s', 'i64_extend_i32_u',
      'f32_add', 'f32_sub', 'f32_mul', 'f32_div', 'f32_sqrt', 'f32_abs', 'f32_neg',
      'f32_ceil', 'f32_floor', 'f32_trunc', 'f32_nearest', 'f32_min', 'f32_max',
      'f32_copysign', 'f32_eq', 'f32_ne', 'f32_lt', 'f32_le', 'f32_gt', 'f32_ge',
      'f64_add', 'f64_sub', 'f64_mul', 'f64_div', 'f64_sqrt', 'f64_abs', 'f64_neg',
      'f64_ceil', 'f64_floor', 'f64_trunc', 'f64_nearest', 'f64_min', 'f64_max',
      'f64_copysign', 'f64_eq', 'f64_ne', 'f64_lt', 'f64_le', 'f64_gt', 'f64_ge',
      'load', 'store', 'load_i32', 'store_i32', 'load_u8', 'store_u8',
      'alloc', 'dealloc', 'realloc',
      'print', 'println', 'print_i32', 'println_i32', 'read_line', 'read_all',
      'cast', 'bitcast', 'from_i32', 'to_i32',
      'assert', 'assert_eq_i32', 'test_checked', 'test_fail',
      'vec_new', 'vec_push', 'vec_get', 'vec_len', 'vec_cap', 'vec_is_empty',
      'vec_set', 'vec_pop', 'vec_clear', 'vec_free',
      'len', 'concat', 'concat3', 'str_eq', 'str_slice', 'str_trim', 'str_split_next',
      'str_starts_with', 'str_ends_with', 'string_builder_new', 'sb_append', 'sb_append_slice_result',
      'sb_append_i32', 'sb_build',
      'some', 'none', 'is_some', 'is_none', 'unwrap', 'unwrap_or', 'option_map',
      'ok', 'err', 'is_ok', 'is_err', 'unwrap_ok', 'unwrap_err', 'result_context',
      'hashmap_new', 'hashmap_insert', 'hashmap_get', 'hashmap_contains', 'hashmap_remove', 'hashmap_len', 'hashmap_free',
      'hashset_new', 'hashset_insert', 'hashset_contains', 'hashset_remove', 'hashset_len', 'hashset_free',
      'btreemap_new', 'btreemap_insert', 'btreemap_get', 'btreemap_contains', 'btreemap_remove', 'btreemap_len', 'btreemap_clear', 'btreemap_free',
      'btreeset_new', 'btreeset_insert', 'btreeset_contains', 'btreeset_remove', 'btreeset_len', 'btreeset_clear', 'btreeset_free',
      'list_nil', 'list_cons', 'list_head', 'list_tail', 'list_len', 'list_get', 'list_free', 'list_reverse',
      'stack_new', 'stack_push', 'stack_pop', 'stack_peek', 'stack_len', 'stack_is_empty', 'stack_clear', 'stack_free',
      'scanner_new', 'scanner_read_i32', 'scanner_read_f64', 'scanner_read_f32',
      'writer_new', 'writer_write_i32', 'writer_write_i64', 'writer_write_f64_ln',
      'writer_write_f32_ln', 'writer_write_str', 'writer_writeln', 'writer_flush', 'writer_free'
    ]);

    function hl(code) {
      const lines = String(code || '').split('\n');
      return lines.map(ln => {
        let out = '';
        let i = 0;
        while (i < ln.length) {
          if (ln[i] === '/' && ln[i + 1] === '/') {
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
            if (ln[i] === '0' && (ln[i + 1] === 'x' || ln[i + 1] === 'X')) {
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
            } else if (unitLiterals.has(tok)) {
              out += '<span class="nm-syn-literal-unit">' + esc(tok) + '</span>';
            } else if (voidLiterals.has(tok)) {
              out += '<span class="nm-syn-literal-void">' + esc(tok) + '</span>';
            } else if (typeConstructors.has(tok)) {
              out += '<span class="nm-syn-type-constructor">' + esc(tok) + '</span>';
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
      }).join('\n');
    }

    function esc(s) {
      return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
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

  // --- Initialization ---

  document.addEventListener('DOMContentLoaded', () => {
    highlightArticleNeplBlocks();

    for (const code of document.querySelectorAll('pre.nm-code > code')) {
      const hiddenNodes = Array.from(code.querySelectorAll('.nm-hidden'));
      if (hiddenNodes.length === 0) continue;

      let groups = [];
      let currentGroup = [];
      for (const node of hiddenNodes) {
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
      try {
        const bindings = await loadBindings();
        if (!running) return;
        const wasmBytes = compileRunnableSnippet(bindings, src.value);
        setStatus('running...', '');
        const blob = new Blob([makeWorkerScript()], { type: 'text/javascript' });
        worker = new Worker(URL.createObjectURL(blob));
        worker.onmessage = (ev) => {
          const msg = ev.data || {};
          if (msg.type === 'log') {
            console.log('[Tutorial Runner Worker]', msg.message);
          } else if (msg.type === 'stdout') {
            setStdoutText(stdoutText + String(msg.text || ''));
          } else if (msg.type === 'stdout_bytes') {
            const line = '[len=' + String(msg.len || 0) + '] ' + String(msg.bytesHex || '');
            stdoutHexLines.push(line);
            console.log('[Tutorial Runner] stdout bytes:', line);
          } else if (msg.type === 'done') {
            setRunningState(false);
            setStatus('done', 'ok');
            console.log('[Tutorial Runner] stdout:\n' + stdoutText);
            console.log('[Tutorial Runner] stdout bytes all:\n' + stdoutHexLines.join('\n'));
            worker && worker.terminate();
            worker = null;
          } else if (msg.type === 'error') {
            setRunningState(false);
            setStatus('runtime error', 'err');
            setStdoutText(stdoutText + '\n[error] ' + String(msg.message || ''));
            console.log('[Tutorial Runner] runtime error:', String(msg.message || ''));
            console.log('[Tutorial Runner] stdout (partial):\n' + stdoutText);
            console.log('[Tutorial Runner] stdout bytes (partial):\n' + stdoutHexLines.join('\n'));
            worker && worker.terminate();
            worker = null;
          }
        };
        if (!wasmBytes) {
          throw new Error('Compilation produced no output (wasmBytes is null)');
        }
        worker.postMessage({ wasmBytes, stdinText: stdin.value || '' });
      } catch (e) {
        setRunningState(false);
        setStatus('compile failed', 'err');
        setTimeout(() => {
          if (!running) setStatus('ready', 'ok');
        }, 3000);
        setStdoutText(stdoutText + '[compile error] ' + String((e && e.message) || e));
        console.log('[Tutorial Runner] compile failed:', String((e && e.message) || e));
        console.log('[Tutorial Runner] stdout (partial):\n' + stdoutText);
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
      const container = pre.parentElement;
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
      if (container.classList.contains('nm-code-content')) {
        container.appendChild(btn);
      } else {
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

    // --- Search UI Logic ---
    const searchKind = document.getElementById('doc-search-kind');
    const searchPath = document.getElementById('doc-search-path');
    const searchInput = document.getElementById('doc-search-input');
    const searchClear = document.getElementById('doc-search-clear');
    const searchResults = document.getElementById('doc-search-results');

    if (searchInput && searchResults && typeof NeplSearch !== 'undefined') {
      let activeIdx = -1;

      function getItems() {
        return searchResults.querySelectorAll('.search-result-item');
      }

      function setActive(idx) {
        const items = getItems();
        items.forEach((el, i) => {
          el.classList.toggle('active', i === idx);
        });
        activeIdx = idx;
        if (idx >= 0 && items[idx]) {
          items[idx].scrollIntoView({ block: 'nearest' });
        }
      }

      function renderSearchResults(query) {
        const q = query.trim();
        const kindVal = searchKind ? searchKind.value : 'all';
        const pathVal = searchPath ? searchPath.value.trim() : '';
        if (!q && kindVal === 'all' && !pathVal) {
          searchResults.classList.remove('open');
          searchResults.innerHTML = '';
          if (searchClear) searchClear.style.display = 'none';
          return;
        }
        if (searchClear && q) searchClear.style.display = 'block';
        else if (searchClear) searchClear.style.display = 'none';

        const hits = NeplSearch.searchIndex(q, __SEARCH_INDEX__, 15, { kind: kindVal, path: pathVal });
        activeIdx = -1;

        if (hits.length === 0) {
          searchResults.innerHTML = '<div class="search-no-result">該当なし</div>';
          searchResults.classList.add('open');
          return;
        }

        searchResults.innerHTML = hits.map((h, i) => {
          const titleEsc = h.title.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
          const snipEsc = h.snippet.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
          const pathEsc = (h.path || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
          const typeEsc = (h.type || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
          const typeHtml = typeEsc ? ' <span class="nm-type-sig">' + typeEsc + '</span>' : '';
          const badge = h.kind ? '<span class="nm-badge nm-badge-' + escapeHtml(h.kind) + '">' + escapeHtml(h.kind) + '</span> ' : '';
          return '<a class="search-result-item" href="' + __ROOT_PREFIX__ + h.url + '" data-idx="' + i + '">'
            + '<div class="search-result-title">' + badge + titleEsc + typeHtml + '</div>'
            + '<div class="search-result-path">Path: ' + pathEsc + '</div>'
            + '<div class="search-result-snippet">' + snipEsc + '</div>'
            + '</a>';
        }).join('');
        searchResults.classList.add('open');

        for (const item of searchResults.querySelectorAll('.search-result-item')) {
          item.addEventListener('click', () => {
            searchResults.classList.remove('open');
            searchInput.value = '';
            if (searchClear) searchClear.style.display = 'none';
            if (sidebar && window.innerWidth <= 768) {
              sidebar.classList.remove('mobile-open');
              sidebarOverlay.classList.remove('mobile-open');
            }
          });
        }
      }

      if (searchKind) {
        searchKind.addEventListener('change', () => {
          renderSearchResults(searchInput.value);
        });
      }

      if (searchPath) {
        searchPath.addEventListener('input', () => {
          renderSearchResults(searchInput.value);
        });
      }

      searchInput.addEventListener('input', () => {
        renderSearchResults(searchInput.value);
      });

      searchInput.addEventListener('keydown', (e) => {
        const items = getItems();
        if (e.key === 'ArrowDown') {
          e.preventDefault();
          setActive(Math.min(activeIdx + 1, items.length - 1));
        } else if (e.key === 'ArrowUp') {
          e.preventDefault();
          setActive(Math.max(activeIdx - 1, 0));
        } else if (e.key === 'Enter') {
          if (activeIdx >= 0 && items[activeIdx]) {
            items[activeIdx].click();
          }
        } else if (e.key === 'Escape') {
          searchResults.classList.remove('open');
          searchInput.blur();
        }
      });

      if (searchClear) {
        searchClear.addEventListener('click', () => {
          searchInput.value = '';
          renderSearchResults('');
          searchInput.focus();
        });
      }

      document.addEventListener('click', (e) => {
        if (!searchInput.contains(e.target) && !searchResults.contains(e.target)) {
          searchResults.classList.remove('open');
        }
      });
    }
  });

  // --- In-Page TOC Scroll Spy ---
  const inpageLinks = document.querySelectorAll('.inpage-toc-link');
  if (inpageLinks.length > 0) {
    const sections = document.querySelectorAll('section.nm-sec');
    const observerOptions = {
      root: null,
      rootMargin: '0px 0px -60% 0px',
      threshold: 0
    };

    let activeId = null;

    const observer = new IntersectionObserver((entries) => {
      let candidate = null;
      for (const entry of entries) {
        if (entry.isIntersecting) {
          candidate = entry.target.id;
        }
      }
      
      if (candidate && candidate !== activeId) {
        activeId = candidate;
        inpageLinks.forEach(link => {
          if (link.getAttribute('href') === '#' + activeId) {
            link.classList.add('active');
            // Ensure the active link is visible in the scrollable sidebar
            const parentToc = link.closest('.doc-inpage-toc');
            if (parentToc) {
               const linkRect = link.getBoundingClientRect();
               const parentRect = parentToc.getBoundingClientRect();
               if (linkRect.top < parentRect.top || linkRect.bottom > parentRect.bottom) {
                 link.scrollIntoView({ behavior: 'smooth', block: 'center' });
               }
            }
          } else {
            link.classList.remove('active');
          }
        });
      }
    }, observerOptions);

    sections.forEach(sec => observer.observe(sec));

    // Handle clicks on the top-level title (href="#") to scroll to top without polluting url hash
    inpageLinks.forEach(link => {
      link.addEventListener('click', (e) => {
        const h = link.getAttribute('href');
        if (h === '#') {
          e.preventDefault();
          window.scrollTo({ top: 0, behavior: 'smooth' });
          // optionally strip the hash using history
          if (window.location.hash) {
            history.pushState('', document.title, window.location.pathname + window.location.search);
          }
        }
      });
    });
  }
}

