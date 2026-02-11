// nodesrc/html_gen.js
// 目的:
// - parser.js が生成した AST を静的 HTML に変換する
// - ルビ / Gloss / 見出しネスト / コードフェンス / 箇条書き / リンク等を、チュートリアル用途として十分な品質で出力する
//
// 方針:
// - 依存を増やさず、Node 標準ライブラリだけで完結させる
// - Markdown 全機能は実装しないが、ドキュメントとして破綻しやすい部分（リンク / ルビ / コード）を優先する

function escapeHtml(s) {
    return String(s)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
}

function escapeHtmlAttr(s) {
    // 属性値用（基本は escapeHtml と同等だが、明示しておく）
    return escapeHtml(s);
}

function rewriteDocLink(hrefRaw) {
    // 相対リンクの .n.md / .nepl を .html に直す
    // - http(s):// や mailto: などはそのまま
    // - #fragment のみもそのまま
    const href = String(hrefRaw || '').trim();
    if (href === '') return href;
    if (href.startsWith('#')) return href;
    if (/^(https?:|mailto:|data:|javascript:)/i.test(href)) return href;

    // URL は / を想定（\ が来た場合も一応 / に寄せる）
    let p = href.replace(/\\/g, '/');

    // hash / query
    let hash = '';
    const hashIdx = p.indexOf('#');
    if (hashIdx >= 0) {
        hash = p.slice(hashIdx);
        p = p.slice(0, hashIdx);
    }
    let query = '';
    const qIdx = p.indexOf('?');
    if (qIdx >= 0) {
        query = p.slice(qIdx);
        p = p.slice(0, qIdx);
    }

    const replaceExt = (ext, newExt) => {
        if (!p.toLowerCase().endsWith(ext)) return null;
        const parts = p.split('/');
        const file = parts.pop();
        const base = file.slice(0, file.length - ext.length);
        const newFile = base + newExt;
        return [...parts, newFile].join('/') + query + hash;
    };

    const r1 = replaceExt('.n.md', '.html');
    if (r1 !== null) return r1;
    const r2 = replaceExt('.nepl', '.html');
    if (r2 !== null) return r2;

    return p + query + hash;
}

function renderInlines(nodes, opt) {
    const o = opt || {};
    let out = '';

    for (const n of nodes) {
        if (n.type === 'text') {
            out += escapeHtml(n.text).replace(/\n/g, '<br/>');
            continue;
        }
        if (n.type === 'code_inline') {
            out += `<code class="nm-code-inline">${escapeHtml(n.text)}</code>`;
            continue;
        }
        if (n.type === 'math') {
            const cls = n.display ? 'math-display' : 'math-inline';
            out += `<span class="${cls}">${escapeHtml(n.text)}</span>`;
            continue;
        }
        if (n.type === 'ruby') {
            out += `<ruby class="nm-ruby"><rb>${renderInlines(n.base, o)}</rb><rt>${renderInlines(n.ruby, o)}</rt></ruby>`;
            continue;
        }
        if (n.type === 'gloss') {
            const base = renderInlines(n.base, o);
            const notes = (n.notes || []).map(x => `<span class="nm-gloss-note">${renderInlines(x, o)}</span>`).join('');
            out += `<ruby class="nm-gloss"><rb>${base}</rb><rt>${notes}</rt></ruby>`;
            continue;
        }
        if (n.type === 'link') {
            const href = o.rewriteLinks ? rewriteDocLink(n.href) : String(n.href || '');
            out += `<a href="${escapeHtmlAttr(href)}">${renderInlines(n.text || [], o)}</a>`;
            continue;
        }

        // 既知以外は安全側へ
        out += escapeHtml(JSON.stringify(n));
    }

    return out;
}

function renderCodeBlock(text) {
    const lines = String(text || '').split('\n');
    let rendered = '';

    for (let i = 0; i < lines.length; i++) {
        const ln0 = lines[i];
        const nl = (i < lines.length - 1) ? '\n' : '';

        if (ln0.startsWith('|')) {
            // "| " を剥がして hidden 扱い
            const t = ln0.slice(1).startsWith(' ') ? ln0.slice(2) : ln0.slice(1);
            rendered += `<span class="nm-hidden">${escapeHtml(t + nl)}</span>`;
        } else {
            rendered += escapeHtml(ln0 + nl);
        }
    }

    return rendered;
}

function ansiToHtml(text) {
    const esc = escapeHtml(text);
    // 簡易的なANSIエスケープシーケンスのHTML変換
    // 色はCSS変数に依存させる
    let out = esc
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');

    // Reset
    out = out.replace(/\\x1b\[0m/g, '</span>');
    // Colors (Foreground)
    out = out.replace(/\\x1b\[31m/g, '<span style="color:var(--err)">'); // Red
    out = out.replace(/\\x1b\[32m/g, '<span style="color:var(--ok)">');  // Green
    out = out.replace(/\\x1b\[33m/g, '<span style="color:#e0af68">');    // Yellow
    out = out.replace(/\\x1b\[34m/g, '<span style="color:var(--accent)">'); // Blue
    out = out.replace(/\\x1b\[35m/g, '<span style="color:#bb9af7">');    // Magenta
    out = out.replace(/\\x1b\[36m/g, '<span style="color:#73daca">');    // Cyan
    out = out.replace(/\\x1b\[37m/g, '<span style="color:#c0caf5">');    // White
    out = out.replace(/\\x1b\[90m/g, '<span style="color:var(--muted)">'); // Gray
    out = out.replace(/\\x1b\[1m/g, '<span style="font-weight:bold">');  // Bold
    return out;
}

function decodeDoctestValue(raw) {
    const s = String(raw || '').trim();
    if (s.startsWith('"') && s.endsWith('"')) {
        try {
            return JSON.parse(s);
        } catch {
            return s.slice(1, -1);
        }
    }
    if (s.startsWith("'") && s.endsWith("'")) {
        return s.slice(1, -1);
    }
    return s
        .replace(/\\n/g, '\n')
        .replace(/\\r/g, '\r')
        .replace(/\\t/g, '\t');
}

function parseDoctestMeta(rawText) {
    const raw = String(rawText || '').replace(/\r\n/g, '\n');
    const firstNl = raw.indexOf('\n');
    const head = (firstNl >= 0 ? raw.slice(0, firstNl) : raw).trim();
    
    const match = head.match(/^\s*neplg2:test(?:\[(.*?)\])?\s*$/);
    if (!match) return null;
    const flagsStr = match[1] || '';
    const flags = flagsStr.split(',').map(s => s.trim()).filter(s => s.length > 0);
    
    const tail = firstNl >= 0 ? raw.slice(firstNl + 1) : '';
    const lines = tail.split('\n');
    const rows = [];
    for (let i = 0; i < lines.length; i++) {
        let ln = String(lines[i] || '').trim();
        if (!ln) continue;
        const m = ln.match(/^(stdin|stdout|ret)\s*:\s*([\s\S]*)$/);
        if (!m) continue;
        const key = m[1];
        let valueRaw = m[2] || '';

        const q = valueRaw.startsWith('"') ? '"' : (valueRaw.startsWith("'") ? "'" : '');
        if (q) {
            let esc = false;
            let closed = false;
            for (let p = 1; p < valueRaw.length; p++) {
                const ch = valueRaw[p];
                if (esc) {
                    esc = false;
                    continue;
                }
                if (ch === '\\') {
                    esc = true;
                    continue;
                }
                if (ch === q) {
                    closed = true;
                    break;
                }
            }
            while (!closed && i + 1 < lines.length) {
                i += 1;
                valueRaw += '\n' + lines[i];
                esc = false;
                for (let p = 1; p < valueRaw.length; p++) {
                    const ch = valueRaw[p];
                    if (esc) {
                        esc = false;
                        continue;
                    }
                    if (ch === '\\') {
                        esc = true;
                        continue;
                    }
                    if (ch === q) {
                        closed = true;
                        break;
                    }
                }
            }
        }
        rows.push({ key, value: decodeDoctestValue(valueRaw) });
    }

    return { head, flags, rows };
}

function renderDoctestBlock(meta, codeNode) {
    let out = `<div class="nm-code-container">`;
    
    // Header: Badges
    out += `<div class="nm-code-header">`;
    out += `<span class="nm-badge-main">TEST</span>`;
    for (const flag of meta.flags) {
        out += `<span class="nm-badge-flag">${escapeHtml(flag)}</span>`;
    }
    out += `</div>`; // header end

    // Code
    out += `<div class="nm-code-content">`;
    if (codeNode) {
        const cls = codeNode.lang ? `language-${escapeHtml(codeNode.lang)}` : '';
        const rendered = renderCodeBlock(codeNode.text || '');
        out += `<pre class="nm-code"><code class="${cls}">${rendered}</code></pre>`;
    }
    out += `</div>`; // content end

    // Footer: stdin/stdout/ret
    if (meta.rows.length > 0) {
        out += `<div class="nm-code-footer">`;
        for (const row of meta.rows) {
            const key = row.key;
            const value = row.value;
            if (key === 'ret') {
                out += `<div class="nm-doctest-row"><span class="nm-doctest-badge">${escapeHtml(key)}</span><code class="nm-doctest-inline">${escapeHtml(value)}</code></div>`;
            } else if (key === 'stdout' || key === 'stderr') {
                // ANSI escape support for stdout/stderr
                const htmlVal = ansiToHtml(value);
                out += `<div class="nm-doctest-row"><span class="nm-doctest-badge">${escapeHtml(key)}</span><pre class="nm-doctest-pre">${htmlVal}</pre></div>`;
            } else {
                out += `<div class="nm-doctest-row"><span class="nm-doctest-badge">${escapeHtml(key)}</span><pre class="nm-doctest-pre">${escapeHtml(value)}</pre></div>`;
            }
        }
        out += `</div>`; // footer end
    }

    out += `</div>`; // wrapper end
    return out;
}

function renderContainerChildren(children, o) {
    let out = '';
    let i = 0;
    while (i < children.length) {
        const child = children[i];
        // Check for doctest paragraph followed by code block
        if (child.type === 'paragraph' && child.inlines && child.inlines.length === 1 && child.inlines[0].type === 'text') {
            const meta = parseDoctestMeta(child.inlines[0].text);
            if (meta) {
                let codeNode = null;
                if (i + 1 < children.length && children[i + 1].type === 'code') {
                    codeNode = children[i + 1];
                    i++; // consume code block
                }
                out += renderDoctestBlock(meta, codeNode);
                i++;
                continue;
            }
        }

        out += renderNode(child, o);
        i++;
    }
    return out;
}

function renderNode(node, opt) {
    const o = opt || { rewriteLinks: true };

    if (node.type === 'document') {
        return renderContainerChildren(node.children, o);
    }
    if (node.type === 'section') {
        const tag = `h${Math.min(6, Math.max(1, node.level))}`;
        const head = renderInlines(node.heading, o);
        const body = renderContainerChildren(node.children, o);
        return `<section class="nm-sec level-${node.level}"><${tag}>${head}</${tag}>\n${body}\n</section>`;
    }
    if (node.type === 'paragraph') {
        return `<p>${renderInlines(node.inlines, o)}</p>`;
    }
    if (node.type === 'hr') {
        return '<hr/>';
    }
    if (node.type === 'list') {
        const items = (node.items || []).map(it => `<li>${renderInlines(it, o)}</li>`).join('\n');
        return `<ul>\n${items}\n</ul>`;
    }
    if (node.type === 'code') {
        const cls = node.lang ? `language-${escapeHtml(node.lang)}` : '';
        const rendered = renderCodeBlock(node.text || '');
        return `<pre class="nm-code"><code class="${cls}">${rendered}</code></pre>`;
    }

    return `<pre>${escapeHtml(JSON.stringify(node, null, 2))}</pre>`;
}

function wrapHtml(body, title, description) {
    const t = title || 'nm';
    const d = description || `${t} - NEPLg2 Getting Started tutorial`;
    return `<!doctype html>
<html lang="ja">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>${escapeHtml(t)}</title>
<meta name="description" content="${escapeHtmlAttr(d)}"/>
<style>
:root{
  --bg:#0b0f19;
  --fg:#e6edf3;
  --muted:#aab6c3;
  --card:#121a2a;
  --border:#23304a;
  --code:#0f1626;
  --accent:#7aa2f7;
}
html,body{background:var(--bg);color:var(--fg);font-family:system-ui,-apple-system,Segoe UI,Roboto,Helvetica,Arial;line-height:1.65;}
main{max-width:980px;margin:24px auto;padding:0 16px;}
a{color:var(--accent);}
hr{border:none;border-top:1px solid var(--border);margin:24px 0;}
.nm-sec{padding:0.5em;padding-left:2em;margin:1em;border-left:3px solid var(--border);border-radius:1em;}
h1,h2,h3,h4,h5,h6{margin:18px 0 10px;}
p{margin:10px 0;}
ul{margin:10px 0 10px 22px;}
.nm-code-container{border:1px solid var(--border);border-radius:12px;background:var(--card);margin:24px 0;overflow:hidden;}
.nm-code-header{display:flex;align-items:center;gap:8px;padding:8px 12px;background:rgba(255,255,255,0.03);border-bottom:1px solid var(--border);flex-wrap:wrap;}
.nm-badge-main{display:inline-block;padding:2px 8px;border-radius:6px;background:#7aa2f7;color:#1a202e;font-size:11px;font-weight:bold;letter-spacing:.05em;}
.nm-badge-flag{display:inline-block;padding:2px 8px;border-radius:6px;border:1px solid var(--border);background:rgba(0,0,0,0.2);color:var(--muted);font-size:11px;}
.nm-code-content{position:relative;}
.nm-code{background:var(--code);padding:12px;overflow:auto;margin:0;border:none;border-radius:0;}
.nm-code code{font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:13px;white-space:pre;}
.nm-code-inline{background:rgba(255,255,255,0.06);border:1px solid rgba(255,255,255,0.10);border-radius:8px;padding:1px 6px;}
.nm-gloss, .nm-ruby{ruby-position:over;}
.nm-gloss rt{font-size:0.72em;color:var(--muted);line-height:1.1;}
.nm-gloss-note{display:block;}
.math-inline{color:var(--muted);}
.math-display{display:block;padding:8px 10px;margin:8px 0;background:rgba(255,255,255,0.03);border:1px dashed var(--border);border-radius:10px;}
.nm-code-footer{padding:12px;border-top:1px solid var(--border);background:#0d1117;}
.nm-doctest-row{display:flex;align-items:flex-start;gap:8px;margin:4px 0;}
.nm-doctest-badge{display:inline-block;min-width:56px;text-align:center;padding:2px 8px;border-radius:999px;border:1px solid var(--border);background:rgba(255,255,255,0.03);color:var(--muted);font-size:11px;line-height:1.5;letter-spacing:.03em;}
.nm-doctest-pre{margin:0;padding:8px 10px;white-space:pre-wrap;word-break:break-word;background:rgba(255,255,255,0.03);border:1px solid var(--border);border-radius:8px;font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:12px;line-height:1.45;flex:1;}
.nm-doctest-inline{padding:2px 8px;border:1px solid var(--border);border-radius:8px;background:rgba(255,255,255,0.03);font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:12px;}
.nm-toggle{display:inline-block;margin:6px 0 12px;padding:6px 10px;border-radius:10px;border:1px solid #2f3f58;background:#0f141b;color:#d6d6d6;cursor:pointer;}
.nm-hidden{display:none;}
ruby rt{font-size:0.6em;opacity:0.95;}
</style>
<script>
// "|" 行（前置き）を隠す/表示する
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
  btn.textContent = show ? '主要部のみ表示' : '全て表示';
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
    btn.textContent = '全て表示';
    btn.onclick = () => nmToggleHidden(btn);
    pre.insertAdjacentElement('afterend', btn);
  }
});
</script>
</head>
<body>
<main>
${body}
</main>
</body>
</html>`;
}

function renderHtml(ast, opt) {
    const body = renderNode(ast, opt || { rewriteLinks: true });
    return wrapHtml(
        body,
        (opt && opt.title) ? opt.title : 'nm',
        (opt && opt.description) ? opt.description : undefined,
    );
}

module.exports = {
    renderHtml,
    renderNode,
    renderInlines,
    wrapHtml,
    rewriteDocLink,
};
