#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const tutorialRuntimePath = path.join(repoRoot, "nodesrc", "static", "playground_runtime.js");
const workerPath = path.join(repoRoot, "web", "src", "runtime", "worker.ts");

const tutorialRuntime = fs.readFileSync(tutorialRuntimePath, "utf8");
const worker = fs.readFileSync(workerPath, "utf8");

assert.match(
    tutorialRuntime,
    /function\s+compilerApiForRun\s*\(\s*bindings\s*\)/,
    "tutorial runtime must centralize compiler session selection",
);
assert.match(
    tutorialRuntime,
    /new\s+bindings\.CompilerSession\s*\(\s*\)/,
    "tutorial runtime must create CompilerSession when the artifact exposes it",
);
assert.match(
    tutorialRuntime,
    /compilerApi\.compile_source_with_vfs_and_profile/,
    "tutorial runtime must compile snippets through the session API before legacy fallbacks",
);
assert.ok(
    tutorialRuntime.indexOf("compilerApi.compile_source_with_vfs_and_profile")
        < tutorialRuntime.indexOf("bindings.get_bundled_stdlib_vfs"),
    "tutorial runtime must not prefer the full stdlib VFS fallback over CompilerSession",
);

assert.match(
    worker,
    /let\s+compilerSession:\s*any\s*\|\s*null\s*=\s*null/,
    "playground worker must keep one CompilerSession per initialized compiler module",
);
assert.match(
    worker,
    /function\s+compilerApiForSession\s*\(\s*compilerModule:\s*any\s*\):\s*any/,
    "playground worker must centralize session selection",
);
assert.match(
    worker,
    /const\s+compilerApi\s*=\s*compilerApiForSession\(compilerModule\)/,
    "playground worker must use the session-aware API for compile requests",
);
assert.match(
    worker,
    /typeof\s+session\.compile_outputs_with_vfs\s*===\s*'function'/,
    "playground worker must only select a session that exposes the compile output API",
);
assert.match(
    worker,
    /request\.compileVfsData/,
    "playground worker must compile from the source-only VFS overlay",
);
assert.match(
    worker,
    /request\.runtimeVfsData/,
    "playground worker must keep the full runtime VFS for WASI execution",
);

console.log("playground compiler session policy passed");
