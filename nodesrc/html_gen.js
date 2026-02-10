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

function renderDoctestMetaParagraph(rawText) {
    const raw = String(rawText || '').replace(/\r\n/g, '\n');
    const firstNl = raw.indexOf('\n');
    const head = (firstNl >= 0 ? raw.slice(0, firstNl) : raw).trim();
    if (!/^\s*neplg2:test(?:\[[^\]]+\])?\s*$/.test(head)) return null;

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

    let out = `<div class="nm-doctest-block"><div class="nm-doctest-meta">${escapeHtml(head)}</div>`;
    for (const row of rows) {
        const key = row.key;
        const value = row.value;
        if (key === 'ret') {
            out += `<div class="nm-doctest-row"><span class="nm-doctest-badge">${escapeHtml(key)}</span><code class="nm-doctest-inline">${escapeHtml(value)}</code></div>`;
        } else {
            out += `<div class="nm-doctest-row"><span class="nm-doctest-badge">${escapeHtml(key)}</span><pre class="nm-doctest-pre">${escapeHtml(value)}</pre></div>`;
        }
    }
    out += '</div>';
    return out;
}

function renderNode(node, opt) {
    const o = opt || { rewriteLinks: true };

    if (node.type === 'document') {
        return node.children.map(ch => renderNode(ch, o)).join('\n');
    }
    if (node.type === 'section') {
        const tag = `h${Math.min(6, Math.max(1, node.level))}`;
        const head = renderInlines(node.heading, o);
        const body = node.children.map(ch => renderNode(ch, o)).join('\n');
        return `<section class="nm-sec level-${node.level}"><${tag}>${head}</${tag}>\n${body}\n</section>`;
    }
    if (node.type === 'paragraph') {
        // doctest メタ行の表示品質を上げる
        if (node.inlines && node.inlines.length === 1 && node.inlines[0].type === 'text') {
            const doctest = renderDoctestMetaParagraph(node.inlines[0].text);
            if (doctest) return doctest;
        }
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

function wrapHtml(body, title) {
    const t = title || 'nm';
    return `<!doctype html>
<html lang="ja">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>${escapeHtml(t)}</title>
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
.nm-code{background:var(--code);border:1px solid var(--border);border-radius:12px;padding:12px;overflow:auto;}
.nm-code code{font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:13px;white-space:pre;}
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
  btn.textContent = show ? '前置き( | 行)を隠す' : '前置き( | 行)を表示';
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
    return wrapHtml(body, (opt && opt.title) ? opt.title : 'nm');
}

module.exports = {
    renderHtml,
    renderNode,
    renderInlines,
    wrapHtml,
    rewriteDocLink,
};
