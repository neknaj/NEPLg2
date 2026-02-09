// nodesrc/html_gen.js
// 目的: 拡張 Markdown（.n.md）や NEPL ドキュメントコメント（//:）から、最低限の静的 HTML を生成する。
//
// 制約:
// - 依存を増やさず、Node 標準ライブラリだけで完結させる。
// - Markdown の全機能は実装せず、チュートリアル用途で十分な範囲（見出し/段落/コードブロック）を優先する。

function escapeHtml(s) {
    return s
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
}

function applyRuby(s) {
    // ルビ記法: [漢字/よみ] -> <ruby>漢字<rt>よみ</rt></ruby>
    // 注意: ネストは扱わない（チュートリアル用途の簡易実装）
    return s.replace(/\[([^\[\]\/\n]+)\/([^\[\]\n]+)\]/g, (_m, k, r) => {
        return `<ruby>${escapeHtml(k)}<rt>${escapeHtml(r)}</rt></ruby>`;
    });
}

function renderCodeBlock(lines) {
    // "|" 行はデフォルトで非表示（クリックで表示）
    const rendered = lines.map((ln) => {
        if (ln.startsWith('|')) {
            const t = ln.slice(1).startsWith(' ') ? ln.slice(2) : ln.slice(1);
            return `<span class="nm-hidden">${escapeHtml(t)}</span>`;
        }
        return escapeHtml(ln);
    }).join('\n');

    return `<pre class="code"><code>${rendered}</code></pre>`;
}

function markdownToHtml(mdText) {
    const lines = mdText.replace(/\r\n/g, '\n').split('\n');

    const out = [];
    let inCode = false;
    let codeLang = '';
    let codeLines = [];

    let para = [];
    function flushPara() {
        if (para.length === 0) return;
        const text = para.join(' ');
        out.push(`<p>${applyRuby(escapeHtml(text))}</p>`);
        para = [];
    }

    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];

        // コードフェンス
        const mFence = raw.match(/^\s*```\s*(.*)\s*$/);
        if (mFence) {
            if (!inCode) {
                flushPara();
                inCode = true;
                codeLang = (mFence[1] || '').trim();
                codeLines = [];
            } else {
                // close
                out.push(renderCodeBlock(codeLines));
                inCode = false;
                codeLang = '';
                codeLines = [];
            }
            continue;
        }

        if (inCode) {
            codeLines.push(raw);
            continue;
        }

        // 見出し
        const mH = raw.match(/^(#{1,6})\s+(.*)$/);
        if (mH) {
            flushPara();
            const level = mH[1].length;
            const title = applyRuby(escapeHtml(mH[2].trim()));
            out.push(`<h${level}>${title}</h${level}>`);
            continue;
        }

        // 空行
        if (/^\s*$/.test(raw)) {
            flushPara();
            continue;
        }

        // 箇条書き（簡易）
        const mLi = raw.match(/^\s*[-*]\s+(.*)$/);
        if (mLi) {
            flushPara();
            // 連続する - を ul にまとめる
            const items = [];
            let j = i;
            while (j < lines.length) {
                const m = lines[j].match(/^\s*[-*]\s+(.*)$/);
                if (!m) break;
                items.push(`<li>${applyRuby(escapeHtml(m[1].trim()))}</li>`);
                j++;
            }
            out.push(`<ul>${items.join('')}</ul>`);
            i = j - 1;
            continue;
        }

        // 通常行は段落へ
        para.push(raw.trim());
    }

    flushPara();

    return out.join('\n');
}

function wrapHtml(title, bodyHtml) {
    const safeTitle = escapeHtml(title);
    return `<!doctype html>
<html lang="ja">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>${safeTitle}</title>
<style>
  :root { color-scheme: dark; }
  body { margin: 0; font-family: ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, sans-serif; line-height: 1.6; background: #0b0d10; color: #e6e6e6; }
  main { max-width: 980px; margin: 0 auto; padding: 24px 18px 60px; }
.nm-sec {padding: 0.5em;padding-left: 2em;margin: 1em;border-left: 3px solid var(--border);border-radius: 1em;}
  h1,h2,h3,h4 { line-height: 1.25; }
  a { color: #8ab4f8; }
  pre.code { background: #11151b; border: 1px solid #263244; border-radius: 12px; padding: 14px 14px; overflow-x: auto; }
  pre.code code { font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace; font-size: 13.5px; }
  .nm-hidden { display: none; }
  .nm-toggle { display: inline-block; margin: 6px 0 12px; padding: 6px 10px; border-radius: 10px; border: 1px solid #2f3f58; background: #0f141b; color: #d6d6d6; cursor: pointer; }
  ruby rt { font-size: 0.6em; opacity: 0.9; }
</style>
<script>
  function nmToggleHidden(btn) {
    const pre = btn.previousElementSibling;
    if (!pre) return;
    const nodes = pre.querySelectorAll('.nm-hidden');
    const currentlyHidden = nodes.length > 0 && nodes[0].style.display !== 'inline';
    for (const n of nodes) {
      n.style.display = currentlyHidden ? 'inline' : 'none';
    }
    btn.textContent = currentlyHidden ? '前置き( | 行)を隠す' : '前置き( | 行)を表示';
  }
  window.addEventListener('DOMContentLoaded', () => {
    for (const pre of document.querySelectorAll('pre.code')) {
      const hasHidden = pre.querySelector('.nm-hidden');
      if (!hasHidden) continue;
      // 初期状態は hidden
      for (const n of pre.querySelectorAll('.nm-hidden')) {
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
${bodyHtml}
</main>
</body>
</html>`;
}

module.exports = {
    markdownToHtml,
    wrapHtml,
};
