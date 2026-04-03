// nodesrc/search.js
// 目的:
// - tutorial / stdlib の全文検索ロジック
// - Node.js (require) からも、HTML <script> inline 埋め込みでも同じコードで動作する
// - .n.md のルビ（漢字/よみ）を適切に扱い、読み仮名・漢字いずれでも検索できるようにする
//
// API:
//   searchIndex(query, index, maxResults) -> SearchResult[]
//   buildSearchEntry(title, url, bodyWords, id) -> SearchEntry
//
// SearchEntry 形式:
//   { id, title, url, body }
//   body: テキスト（漢字・よみ両方含む、スペース区切り）
//
// SearchResult 形式:
//   { id, title, url, snippet, score }

"use strict";

/**
 * クエリ文字列をトークンに分割する（スペース・全角スペースで分割、空を除く）
 * @param {string} query
 * @returns {string[]}
 */
function tokenizeQuery(query) {
  return (
    String(query || "")
      .toLowerCase()
      // 全角スペースも区切りとして扱う
      .replace(/\u3000/g, " ")
      .split(/\s+/)
      .filter(Boolean)
  );
}

/**
 * テキストを検索用に正規化する（小文字化・全角スペース除去）
 * @param {string} text
 * @returns {string}
 */
function normalizeText(text) {
  return String(text || "")
    .toLowerCase()
    .replace(/\u3000/g, " ");
}

/**
 * スニペットを生成する: 最初のヒット箇所の周囲 N 文字を返す
 * @param {string} body - 元の body テキスト
 * @param {string[]} tokens - クエリトークン
 * @param {number} radius - ヒット前後の文字数
 * @returns {string}
 */
function makeSnippet(body, tokens, radius) {
  radius = radius !== undefined ? radius : 60;
  const norm = normalizeText(body);
  // 最初にヒットするトークンの位置を探す
  let hitIdx = norm.length;
  for (const tok of tokens) {
    const idx = norm.indexOf(tok);
    if (idx >= 0 && idx < hitIdx) {
      hitIdx = idx;
    }
  }
  if (hitIdx === norm.length) {
    // ヒットなし: 先頭を返す
    return body.slice(0, radius * 2).replace(/\n/g, " ");
  }
  const start = Math.max(0, hitIdx - radius);
  const end = Math.min(body.length, hitIdx + radius);
  const snippet =
    (start > 0 ? "…" : "") +
    body.slice(start, end).replace(/\n/g, " ") +
    (end < body.length ? "…" : "");
  return snippet;
}

/**
 * エントリリストをクエリで全文検索する
 * @param {string} query - 検索クエリ（スペース区切りで AND 検索）
 * @param {SearchEntry[]} index - 検索インデックス
 * @param {number} [maxResults=20] - 最大件数
 * @returns {SearchResult[]}
 */
function searchIndex(query, index, maxResults, options) {
  maxResults = maxResults !== undefined ? maxResults : 20;
  options = options || {};
  const tokens = tokenizeQuery(query);
  if (tokens.length === 0 && (!options.kind || options.kind === "all"))
    return [];

  const results = [];

  for (const entry of index) {
    if (options.kind && options.kind !== "all" && entry.kind !== options.kind)
      continue;
    const normTitle = normalizeText(entry.title);
    const normBody = normalizeText(entry.body || "");

    let score = 0;
    let allHit = true;

    for (const tok of tokens) {
      const inTitle = normTitle.indexOf(tok) >= 0;
      const inBody = normBody.indexOf(tok) >= 0;
      if (!inTitle && !inBody) {
        allHit = false;
        break;
      }
      // タイトルヒットは高スコア、本文ヒットは低スコア
      if (inTitle) score += 10;
      if (inBody) {
        // 複数回ヒットのボーナス
        let pos = 0;
        let cnt = 0;
        while ((pos = normBody.indexOf(tok, pos)) >= 0) {
          cnt++;
          pos++;
        }
        score += Math.min(cnt, 5);
      }
    }

    if (!allHit) continue;

    if (options.path) {
      const normPath = normalizeText(entry.path || "");
      const normPathFilter = normalizeText(options.path);
      if (normPath.indexOf(normPathFilter) < 0) {
        continue;
      }
    }

    const snippet = makeSnippet(entry.body || "", tokens, 60);
    results.push({
      id: entry.id,
      title: entry.title,
      url: entry.url,
      path: entry.path,
      snippet,
      score,
      kind: entry.kind,
      type: entry.type,
    });
  }

  // スコア降順にソートして maxResults 件返す
  results.sort((a, b) => b.score - a.score);
  return results.slice(0, maxResults);
}

/**
 * ルビ付きテキスト（AST の inline[] から生成）から検索用本文を構築する
 * ルビは「漢字 よみ」の形で両方インデックスに含める
 * @param {object[]} inlines - parser.js の parseInlines() 結果
 * @returns {string}
 */
function inlinesToSearchText(inlines) {
  if (!Array.isArray(inlines)) return "";
  return inlines
    .map((n) => {
      if (n.type === "text") return n.text;
      if (n.type === "code_inline") return n.text;
      if (n.type === "math") return n.text;
      if (n.type === "ruby") {
        // 漢字と読み仮名を両方含める（スペース区切り）
        const base = inlinesToSearchText(n.base);
        const ruby = inlinesToSearchText(n.ruby);
        return base + " " + ruby;
      }
      if (n.type === "gloss") {
        const base = inlinesToSearchText(n.base);
        const notes = (n.notes || [])
          .map((x) => inlinesToSearchText(x))
          .join(" ");
        return base + " " + notes;
      }
      if (n.type === "link") return inlinesToSearchText(n.text);
      return "";
    })
    .join("")
    .replace(/\s+/g, " ")
    .trim();
}

/**
 * AST の document ノードから検索エントリ配列を構築する
 * 各 section がエントリになる（本文はその section の段落等を連結）
 * @param {object} ast - parseNmdAst() の結果
 * @param {string} pageUrl - このページの URL (ex: "std/vec.html")
 * @param {string} pageTitle - このページのタイトル
 * @param {string} pagePath - このページのパス表示用 (ex: "alloc/collections/vec.nepl")
 * @returns {SearchEntry[]}
 */
function buildEntriesFromAst(ast, pageUrl, pageTitle, pagePath) {
  const entries = [];

  // スラグを生成する（URL fragment 用）
  function makeSlug(text, typeInfo) {
    let base = text
      .toLowerCase()
      .replace(/[\s\u3000]+/g, "-")
      .replace(
        /[^\w\u3040-\u309f\u30a0-\u30ff\u4e00-\u9fff\uac00-\ud7af\-]/g,
        "",
      );
    if (typeInfo) {
      // 型情報をスラグに追加（オーバーロード対策）
      // 特殊記号や空白を安全な文字に変換
      const safeType = typeInfo
        .replace(/\s+/g, "") // 全ての空白を削除
        .replace(/[<>()*]/g, "") // 不要な記号を削除
        .replace(/[,]/g, "-")
        .replace(/->/g, "-to-")
        .toLowerCase();
      base += "--" + safeType;
    }
    return base.slice(0, 100);
  }

  // 節から本文テキストを収集（子ノードの段落・コードブロック等）
  function gatherBody(node, depth) {
    depth = depth !== undefined ? depth : 0;
    const parts = [];
    const children = node.children || [];
    for (const child of children) {
      if (child.type === "paragraph") {
        parts.push(inlinesToSearchText(child.inlines));
      } else if (child.type === "code") {
        // コードブロックはそのままテキストとして含める（スペースで圧縮）
        if (child.text) {
          parts.push(child.text.replace(/\s+/g, " ").trim());
        }
      } else if (child.type === "list") {
        for (const item of child.items || []) {
          parts.push(inlinesToSearchText(item));
        }
      } else if (child.type === "section" && depth < 1) {
        // 直下のサブセクションの本文も含める（浅い深さのみ）
        parts.push(...gatherBody(child, depth + 1));
      }
    }
    return parts;
  }

  // ページ自体のエントリ（タイトルのみ、フォールバック用）
  const pageEntry = {
    id: "page-" + makeSlug(pageTitle || pageUrl),
    title: pageTitle || pageUrl,
    url: pageUrl,
    path: pagePath || pageUrl,
    body: "",
  };

  // document 直下の先頭段落を body にする
  const topParas = (ast.children || [])
    .filter((c) => c.type === "paragraph")
    .slice(0, 2);
  if (topParas.length > 0) {
    pageEntry.body = topParas
      .map((p) => inlinesToSearchText(p.inlines))
      .join(" ");
  }
  entries.push(pageEntry);

  // 各 section をエントリ化
  function visitSections(nodes, ancestorSlug) {
    for (const node of nodes) {
      if (node.type !== "section") {
        // section 以外 (paragraph, code, list 等) はここでは処理しない
        continue;
      }
      const headText = inlinesToSearchText(node.heading);
      const slug = makeSlug(headText, node.typeInfo);
      const id = ancestorSlug ? ancestorSlug + "-" + slug : slug;
      const url = pageUrl + "#" + id;
      const bodyParts = gatherBody(node, 0);
      const body = bodyParts.join(" ").replace(/\s+/g, " ").slice(0, 800);

      entries.push({
        id,
        title: headText,
        url,
        path: pagePath || pageUrl,
        body,
        kind: node.kind || null,
        type: node.typeInfo || null,
      });

      // 子 section も再帰処理
      visitSections(node.children || [], id);
    }
  }

  visitSections(ast.children || [], "");

  return entries;
}

// Node.js 環境ではモジュールとしてエクスポート、
// ブラウザ環境ではグローバルに公開
(function (root, factory) {
  if (typeof module !== "undefined" && module.exports) {
    // Node.js
    module.exports = factory();
  } else {
    // ブラウザ
    root.NeplSearch = factory();
  }
})(typeof globalThis !== "undefined" ? globalThis : this, function () {
  return {
    searchIndex,
    buildEntriesFromAst,
    inlinesToSearchText,
    tokenizeQuery,
    normalizeText,
    makeSnippet,
  };
});
