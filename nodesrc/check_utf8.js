#!/usr/bin/env node
/**
 * Verify that tracked text files are UTF-8 without BOM.
 *
 * Usage:
 *   node nodesrc/check_utf8.js
 *   node nodesrc/check_utf8.js tutorials/getting_started doc examples
 */

const fs = require("fs");
const path = require("path");
const cp = require("child_process");
const { TextDecoder } = require("util");

const TEXT_EXTENSIONS = new Set([
  ".md",
  ".n.md",
  ".nepl",
  ".ts",
  ".js",
  ".json",
  ".toml",
  ".yml",
  ".yaml",
  ".html",
  ".css",
  ".rs",
  ".txt",
]);

function normalizePath(p) {
  return p.replace(/\\/g, "/");
}

function isTextFile(relPath) {
  const lower = relPath.toLowerCase();
  if (lower.endsWith(".n.md")) return true;
  return TEXT_EXTENSIONS.has(path.extname(lower));
}

function isWithinRequestedRoots(relPath, roots) {
  if (roots.length === 0) return true;
  return roots.some((root) => relPath === root || relPath.startsWith(root + "/"));
}

function listTrackedFiles() {
  return cp
    .execSync("git ls-files", { encoding: "utf8" })
    .split(/\r?\n/)
    .filter(Boolean)
    .map(normalizePath);
}

function main() {
  const decoder = new TextDecoder("utf-8", { fatal: true });
  const requestedRoots = process.argv.slice(2).map(normalizePath);
  const files = listTrackedFiles().filter(
    (relPath) => isTextFile(relPath) && isWithinRequestedRoots(relPath, requestedRoots),
  );

  const invalidUtf8 = [];
  const utf8Bom = [];

  for (const relPath of files) {
    const buf = fs.readFileSync(relPath);
    const hasBom =
      buf.length >= 3 && buf[0] === 0xef && buf[1] === 0xbb && buf[2] === 0xbf;
    if (hasBom) utf8Bom.push(relPath);
    try {
      decoder.decode(buf);
    } catch {
      invalidUtf8.push(relPath);
    }
  }

  const ok = invalidUtf8.length === 0 && utf8Bom.length === 0;
  const summary = {
    roots: requestedRoots,
    scanned: files.length,
    invalidUtf8Count: invalidUtf8.length,
    utf8BomCount: utf8Bom.length,
    invalidUtf8,
    utf8Bom,
  };

  console.log(JSON.stringify(summary, null, 2));
  if (!ok) process.exit(1);
}

main();
