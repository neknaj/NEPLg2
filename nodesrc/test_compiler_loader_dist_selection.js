#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const { findCompilerDistDir } = require("./compiler_loader");

function writeCompilerPair(dir, hash, date) {
    fs.mkdirSync(dir, { recursive: true });
    const jsPath = path.join(dir, `nepl-web-${hash}.js`);
    const wasmPath = path.join(dir, `nepl-web-${hash}_bg.wasm`);
    fs.writeFileSync(jsPath, "export function initSync() {}\n", "utf8");
    fs.writeFileSync(wasmPath, "");
    fs.utimesSync(jsPath, date, date);
    fs.utimesSync(wasmPath, date, date);
}

const root = fs.mkdtempSync(path.join(os.tmpdir(), "nepl-compiler-loader-dist-"));

try {
    const oldDist = path.join(root, "dist");
    const newDist = path.join(root, "web", "dist");
    writeCompilerPair(oldDist, "old", new Date("2026-01-01T00:00:00Z"));
    writeCompilerPair(newDist, "new", new Date("2026-02-01T00:00:00Z"));

    const selected = findCompilerDistDir([oldDist, newDist]);
    assert.ok(selected, "compiler dist should be selected");
    assert.equal(selected.distDir, newDist);
    assert.equal(selected.pair.jsFile, "nepl-web-new.js");
    assert.equal(selected.pair.wasmFile, "nepl-web-new_bg.wasm");
} finally {
    fs.rmSync(root, { recursive: true, force: true });
}

console.log("compiler loader dist selection regression passed");
