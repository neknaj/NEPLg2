const { parseInlines } = require("./parser");

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function escapeHtmlAttr(s) { return escapeHtml(s); }

function inlinesToPlainText(inlines) {
  if (!Array.isArray(inlines)) return "";
  return inlines
    .map((n) => {
      if (n.type === "text") return n.text;
      if (n.type === "code_inline") return n.text;
      if (n.type === "math") return n.text;
      if (n.type === "ruby") {
        return inlinesToPlainText(n.base) + " " + inlinesToPlainText(n.ruby);
      }
      if (n.type === "gloss") {
        const base = inlinesToPlainText(n.base);
        const notes = (n.notes || [])
          .map((x) => inlinesToPlainText(x))
          .join(" ");
        return base + " " + notes;
      }
      if (n.type === "link") return inlinesToPlainText(n.text);
      return "";
    })
    .join("")
    .replace(/\s+/g, " ")
    .trim();
}

function makeSlug(text, typeInfo) {
  let base = text
    .toLowerCase()
    .replace(/[\s\u3000]+/g, "-")
    .replace(/[^\w\u3040-\u309f\u30a0-\u30ff\u4e00-\u9fff\uac00-\ud7af\-]/g, "");
  if (typeInfo) {
    const safeType = typeInfo
      .replace(/\s+/g, "")
      .replace(/[<>()*]/g, "")
      .replace(/[,]/g, "-")
      .replace(/->/g, "-to-")
      .toLowerCase();
    base += "--" + safeType;
  }
  return base.slice(0, 100);
}

function inlinesToHtml(inlines) {
  const { renderInlines } = require("./html_gen");
  return renderInlines(inlines, { rewriteLinks: true });
}

function extractInPageToc(node, ancestorSlug = "") {
  let tocNodes = [];
  
  if (node.type === "document") {
    for (const child of node.children) {
      tocNodes.push(...extractInPageToc(child, ancestorSlug));
    }
  } else if (node.type === "section") {
    const titleText = inlinesToPlainText(node.heading);
    const slug = makeSlug(titleText, node.typeInfo);
    const fullId = ancestorSlug ? ancestorSlug + "-" + slug : slug;
    const titleHtml = inlinesToHtml(node.heading);
    
    // Level 1 heading gets a special "top page" anchor (empty href)
    tocNodes.push({
      id: node.level === 1 ? "" : fullId,
      level: node.level,
      titleHtml: titleHtml,
      typeInfo: node.typeInfo,
      kind: node.kind,
    });
    
    for (const child of node.children) {
      tocNodes.push(...extractInPageToc(child, fullId));
    }
  }
  
  return tocNodes;
}

function renderInPageTocHtml(tocNodes) {
  if (!tocNodes || tocNodes.length === 0) return "";

  const minLevel = Math.min(...tocNodes.map(n => n.level));
  
  const items = tocNodes.map((node, index) => {
     let depth = node.level - minLevel;
     if (depth < 0) depth = 0;
     if (depth > 6) depth = 6;
     
     let badgeHtml = "";
     if (node.kind) {
       badgeHtml = ` <span class="nm-badge nm-badge-${escapeHtmlAttr(node.kind)} inpage-toc-badge">${escapeHtml(node.kind)}</span>`;
     }
     
     const href = node.id ? `#${escapeHtmlAttr(node.id)}` : '#';
     
     return `<li><a class="inpage-toc-link depth-${depth}" href="${href}">${node.titleHtml}${badgeHtml}</a></li>`;
  }).join("\n");
  
  return `<ul class="inpage-toc-list">\n${items}\n</ul>`;
}

module.exports = {
  extractInPageToc,
  renderInPageTocHtml
};
