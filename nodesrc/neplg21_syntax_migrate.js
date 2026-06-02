#!/usr/bin/env node
// NEPLg2.0 surface syntax to NEPLg2.1 notation migrator.
//
// This converter is intentionally syntax-aware only for balanced type
// annotations and lambda parameter lists. Generic call postfix removal still
// needs semantic review, so this script does not delete `name<T>` calls.

const fs = require("node:fs");
const path = require("node:path");

const DEFAULT_ROOTS = [
  "stdlib",
  "examples",
  "tests",
  "tutorials",
  "doc/examples",
  "nepl-core/tests/fixtures",
];

function parseArgs(argv) {
  const roots = [];
  let check = false;
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === "--check") {
      check = true;
      continue;
    }
    if (arg === "--root" && i + 1 < argv.length) {
      roots.push(argv[++i]);
      continue;
    }
    if (arg === "-h" || arg === "--help") {
      return { help: true, roots, check };
    }
    roots.push(arg);
  }
  return { help: false, roots: roots.length ? roots : DEFAULT_ROOTS, check };
}

function usage() {
  console.log("Usage: node nodesrc/neplg21_syntax_migrate.js [--check] [--root <path> ...]");
}

function listFiles(root) {
  const out = [];
  if (!fs.existsSync(root)) return out;
  const rootStat = fs.statSync(root);
  if (rootStat.isFile()) {
    const name = path.basename(root);
    return name.endsWith(".nepl") || name.endsWith(".n.md") ? [root] : [];
  }
  const stack = [root];
  while (stack.length) {
    const current = stack.pop();
    const entries = fs.readdirSync(current, { withFileTypes: true });
    for (const entry of entries) {
      const full = path.join(current, entry.name);
      if (entry.isDirectory()) {
        if (entry.name === ".git" || entry.name === "target" || entry.name === "node_modules") {
          continue;
        }
        stack.push(full);
      } else if (entry.isFile() && (entry.name.endsWith(".nepl") || entry.name.endsWith(".n.md"))) {
        out.push(full);
      }
    }
  }
  out.sort();
  return out;
}

function isIdentStart(ch) {
  return /[A-Za-z_]/.test(ch);
}

function isIdentContinue(ch) {
  return /[A-Za-z0-9_]/.test(ch);
}

function skipString(src, i, quote) {
  i++;
  while (i < src.length) {
    if (src[i] === "\\") {
      i += 2;
      continue;
    }
    if (src[i] === quote) {
      return i + 1;
    }
    i++;
  }
  return i;
}

function isUnescapedQuote(src, i, ch) {
  return ch === "\"" && src[i - 1] !== "\\" && !linePrefix(src, i).trimStart().startsWith("//");
}

function matching(src, start, open, close) {
  let depth = 0;
  for (let i = start; i < src.length; i++) {
    const ch = src[i];
    if (isUnescapedQuote(src, i, ch)) {
      i = skipString(src, i, ch) - 1;
      continue;
    }
    if (ch === open) {
      depth++;
      continue;
    }
    if (ch === close) {
      if (open === "<" && close === ">" && (src[i - 1] === "-" || src[i - 1] === "*")) {
        continue;
      }
      depth--;
      if (depth === 0) return i;
    }
  }
  return -1;
}

function splitTopLevel(src, sep) {
  const parts = [];
  let depthAngle = 0;
  let depthParen = 0;
  let start = 0;
  for (let i = 0; i < src.length; i++) {
    const ch = src[i];
    if (isUnescapedQuote(src, i, ch)) {
      i = skipString(src, i, ch) - 1;
      continue;
    }
    if (ch === "<") depthAngle++;
    else if (ch === ">") depthAngle = Math.max(0, depthAngle - 1);
    else if (ch === "(") depthParen++;
    else if (ch === ")") depthParen = Math.max(0, depthParen - 1);
    else if (ch === sep && depthAngle === 0 && depthParen === 0) {
      parts.push(src.slice(start, i).trim());
      start = i + 1;
    }
  }
  parts.push(src.slice(start).trim());
  return parts.filter((part) => part.length > 0);
}

function findTopLevelArrow(src) {
  let depthAngle = 0;
  let depthParen = 0;
  for (let i = 0; i < src.length; i++) {
    const ch = src[i];
    if (isUnescapedQuote(src, i, ch)) {
      i = skipString(src, i, ch) - 1;
      continue;
    }
    if (ch === "<") depthAngle++;
    else if (ch === ">") depthAngle = Math.max(0, depthAngle - 1);
    else if (ch === "(") depthParen++;
    else if (ch === ")") depthParen = Math.max(0, depthParen - 1);
    if (depthAngle === 0 && depthParen === 0) {
      if (src.startsWith("->", i)) return { index: i, effect: "fn", length: 2 };
      if (src.startsWith("*>", i)) return { index: i, effect: "impure fn", length: 2 };
    }
  }
  return null;
}

function convertFunctionTypeParams(paramsText, effect) {
  const trimmed = paramsText.trim();
  if (trimmed === "" || trimmed === "()" || trimmed === "(())") return `${effect} void`;
  const parts = splitTopLevel(trimmed, ",");
  if (parts.length === 0) return `${effect} void`;
  return parts.map((part) => `${effect} ${convertTypeExpr(part)}`).join(" ");
}

function stripSingleOuterParens(src) {
  const trimmed = src.trim();
  if (trimmed === "()" || trimmed === "(())") return "unit";
  if (!trimmed.startsWith("(")) return trimmed;
  const end = matching(trimmed, 0, "(", ")");
  if (end === trimmed.length - 1) return trimmed.slice(1, -1).trim();
  return trimmed;
}

function convertTypeExpr(src) {
  let text = src.trim();
  if (text === "") return text;
  if (text.startsWith("&mut ")) return `&mut ${convertTypeExpr(text.slice(5))}`;
  if (text.startsWith("&")) return `&${convertTypeExpr(text.slice(1))}`;

  const arrow = findTopLevelArrow(text);
  if (arrow) {
    let left = text.slice(0, arrow.index).trim();
    const right = text.slice(arrow.index + arrow.length).trim();
    left = left === "()" || left === "(())" ? left : stripSingleOuterParens(left);
    const resultIsFunction = findTopLevelArrow(stripSingleOuterParens(right)) !== null;
    const result = convertTypeExpr(right);
    return `${convertFunctionTypeParams(left, arrow.effect)} ${resultIsFunction ? `(${result})` : result}`;
  }

  text = stripSingleOuterParens(text);
  if (text === "()" || text === "unit") return "unit";

  let out = "";
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (isUnescapedQuote(text, i, ch)) {
      const end = skipString(text, i, ch);
      out += text.slice(i, end);
      i = end - 1;
      continue;
    }
    if (isIdentStart(ch)) {
      const start = i;
      i++;
      while (i < text.length && (isIdentContinue(text[i]) || (text[i] === ":" && text[i + 1] === ":"))) {
        i += text[i] === ":" ? 2 : 1;
      }
      const name = text.slice(start, i);
      if (text[i] === "<") {
        const close = matching(text, i, "<", ">");
        if (close > i) {
          const args = splitTopLevel(text.slice(i + 1, close), ",").map(convertTypeExpr);
          out += `${name} ${args.join(" ")}`;
          i = close;
        } else {
          out += name;
          i--;
        }
      } else {
        out += name;
        i--;
      }
      continue;
    }
    out += ch;
  }
  return out.replace(/\s+/g, " ").trim();
}

function previousWord(src, idx) {
  let i = idx - 1;
  while (i >= 0 && /\s/.test(src[i])) i--;
  let end = i + 1;
  while (i >= 0 && /[A-Za-z0-9_]/.test(src[i])) i--;
  return src.slice(i + 1, end);
}

function linePrefix(src, idx) {
  const start = src.lastIndexOf("\n", idx - 1) + 1;
  return src.slice(start, idx);
}

function isTypeParamList(text) {
  const parts = splitTopLevel(text, ",");
  return parts.length > 0 && parts.every((part) => part.trim().startsWith("."));
}

function isLikelyStandaloneTypeExpr(text) {
  const trimmed = text.trim();
  if (trimmed === "") return false;
  if (trimmed === "()" || trimmed === "unit") return true;
  if (trimmed.startsWith("(") || trimmed.startsWith("&") || trimmed.startsWith(".")) return true;
  if (/^(i32|u32|i64|u64|i128|u128|u8|f32|f64|bool|char|never|str)\b/.test(trimmed)) return true;
  if (/^(fn|impure\s+fn)\b/.test(trimmed)) return true;
  return /^[A-Z]/.test(trimmed);
}

function isLikelyTypeAnnotation(src, lt, gt) {
  const prev = lt > 0 ? src[lt - 1] : "";
  if (/[A-Za-z0-9_.>:]/.test(prev)) return false;

  const inner = src.slice(lt + 1, gt);
  if (!isLikelyStandaloneTypeExpr(inner)) return false;

  const before = previousWord(src, lt);
  if (["fn", "struct", "enum", "trait", "impl", "for"].includes(before)) return false;
  if (/#intrinsic\s+"[^"]+"\s*$/.test(linePrefix(src, lt))) return false;
  if (isTypeParamList(inner)) {
    const prefix = linePrefix(src, lt).trim();
    if (/(^|[^A-Za-z0-9_])(pub\s+)?(fn|struct|enum|trait)\s+[A-Za-z_][A-Za-z0-9_]*\s*$/.test(prefix)) {
      return false;
    }
    if (/^impl\s*$/.test(prefix)) return false;
  }
  let i = gt + 1;
  while (i < src.length && src[i] === " ") i++;
  return i >= src.length || ![";", ",", ")", "]"].includes(src[i]);
}

function convertAngleAnnotations(src) {
  let out = "";
  for (let i = 0; i < src.length; i++) {
    const ch = src[i];
    if (isUnescapedQuote(src, i, ch)) {
      const end = skipString(src, i, ch);
      out += src.slice(i, end);
      i = end - 1;
      continue;
    }
    if (ch === "<") {
      const gt = matching(src, i, "<", ">");
      if (gt > i && isLikelyTypeAnnotation(src, i, gt)) {
        out += `%${convertTypeExpr(src.slice(i + 1, gt))}`;
        i = gt;
        continue;
      }
    }
    out += ch;
  }
  return out;
}

function mapOutsideStrings(src, mapper) {
  let out = "";
  let start = 0;
  for (let i = 0; i < src.length; i++) {
    const ch = src[i];
    if (isUnescapedQuote(src, i, ch)) {
      const end = skipString(src, i, ch);
      out += mapper(src.slice(start, i));
      out += src.slice(i, end);
      start = end;
      i = end - 1;
    }
  }
  out += mapper(src.slice(start));
  return out;
}

function convertFunctionParamLists(src) {
  return mapOutsideStrings(src, (segment) =>
    segment.replace(/(^[ \t]*(?:(?:\/\/:\|?[ \t]*)?)(?:(?:pub[ \t]+)?fn|let)[^\r\n]*?%[^\r\n]*?)[ \t]*\(([^()\r\n]*)\)([ \t]*:)/gm, (_m, head, params, tail) => {
      const converted = params.trim() === ""
        ? "\\void"
        : splitTopLevel(params, ",").map((param) => `\\${param.trim()}`).join("");
      return `${head} ${converted}${tail}`;
    })
  );
}

function convertLegacyPercentEffectSignatures(src) {
  return mapOutsideStrings(src, (segment) =>
    segment.replace(/%unit\*((?:[A-Za-z_][A-Za-z0-9_:]*(?:<[^>\r\n]+>)?)|unit|i32|u8|f32|bool|char|never|str)>/g, (_m, result) =>
      `%impure fn void ${convertTypeExpr(result)}`
    )
  );
}

function convertMissingZeroArgLambdaMarker(src) {
  return mapOutsideStrings(src, (segment) =>
    segment.replace(
      /(^[ \t]*(?:(?:\/\/:\|?[ \t]*)?)(?:(?:pub[ \t]+)?fn|let)[^\r\n\\:]*?%(?:impure[ \t]+)?fn\s+void\s+(?!void\b)[^\r\n\\:]*?)([ \t]*:)/gm,
      "$1 \\void$2",
    )
  );
}

function convertLegacyUnitZeroArgMarkers(src) {
  return mapOutsideStrings(src, (segment) =>
    segment.replace(
      /(^[ \t]*(?:(?:\/\/:\|?[ \t]*)?)(?:(?:pub[ \t]+)?fn|let)[^\r\n]*?%)([^\r\n\\:]*?)\\unit([ \t]*:)/gm,
      (_m, head, typePart, tail) => `${head}${typePart.replace(/\b((?:impure\s+)?fn)\s+unit\b/g, "$1 void")}\\void${tail}`,
    )
  );
}

function isIntrinsicDirectiveLine(segment, idx) {
  return /#intrinsic\s+"[^"]+"\s*/.test(linePrefix(segment, idx));
}

function convertUnitSyntax(src) {
  return mapOutsideStrings(src, (segment) => {
    let out = "";
    for (let i = 0; i < segment.length; i++) {
      if (segment.startsWith("()", i) && !isIntrinsicDirectiveLine(segment, i)) {
        out += "unit";
        i++;
        continue;
      }
      out += segment[i];
    }
    return out;
  });
}

function restoreIntrinsicArgDelimiters(src) {
  return src.replace(/(#intrinsic\s+"[^"\r\n]+"\s+(?:<[^>\r\n]*>\s*)?)unit\b/g, "$1()");
}

function migrateText(src) {
  return convertLegacyUnitZeroArgMarkers(
    convertMissingZeroArgLambdaMarker(
      convertLegacyPercentEffectSignatures(
        restoreIntrinsicArgDelimiters(
          convertUnitSyntax(convertFunctionParamLists(convertAngleAnnotations(src)))
        )
      )
    )
  );
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) {
    usage();
    return;
  }

  const files = [...new Set(args.roots.flatMap(listFiles))];
  const changed = [];
  for (const file of files) {
    const src = fs.readFileSync(file, "utf8");
    const next = migrateText(src);
    if (next !== src) {
      changed.push(file);
      if (!args.check) fs.writeFileSync(file, next, "utf8");
    }
  }

  console.log(`${args.check ? "would update" : "updated"} ${changed.length} file(s)`);
  for (const file of changed.slice(0, 40)) {
    console.log(path.relative(process.cwd(), file).replace(/\\/g, "/"));
  }
  if (changed.length > 40) console.log(`... ${changed.length - 40} more`);
  if (args.check && changed.length > 0) process.exitCode = 1;
}

if (require.main === module) {
  main();
}

module.exports = {
  migrateText,
};
