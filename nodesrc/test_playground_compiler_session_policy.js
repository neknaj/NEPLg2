#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const tutorialRuntimePath = path.join(repoRoot, "nodesrc", "static", "playground_runtime.js");
const workerPath = path.join(repoRoot, "web", "src", "runtime", "worker.ts");
const shellPath = path.join(repoRoot, "web", "src", "terminal", "shell.ts");

const tutorialRuntime = fs.readFileSync(tutorialRuntimePath, "utf8");
const worker = fs.readFileSync(workerPath, "utf8");
const shell = fs.readFileSync(shellPath, "utf8");

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
    /class\s+CompilerInitializationError\s+extends\s+Error/,
    "playground worker must distinguish compiler asset initialization failure from user compile errors",
);
assert.match(
    worker,
    /function\s+resetCompilerInitializationState\s*\(\s*\)[\s\S]*compilerInitPromise\s*=\s*null[\s\S]*compilerSession\s*=\s*null[\s\S]*compilerSessionChecked\s*=\s*false/,
    "playground worker must clear init promise and session state after compiler initialization failure",
);
assert.match(
    worker,
    /compilerInitPromise\s*=\s*\(async\s*\(\)\s*=>[\s\S]*?\}\)\(\)\.catch\(\(error\)\s*=>\s*\{[\s\S]*resetCompilerInitializationState\(\);[\s\S]*throw\s+new\s+CompilerInitializationError\(error\);/,
    "playground worker must not cache a rejected compiler initialization promise",
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
    /request\.compilerMode\s*\|\|\s*'rust'/,
    "playground worker must default compile requests to the Rust compiler mode",
);
assert.match(
    worker,
    /mode\s*===\s*'selfhost'/,
    "playground worker must keep selfhost compiler mode on a separate explicit path",
);
assert.match(
    worker,
    /typeof\s+session\.compile_outputs_with_vfs\s*===\s*'function'/,
    "playground worker must only select a session that exposes the compile output API",
);
assert.match(
    worker,
    /isCompilerInitFailure[\s\S]*phase\s*=\s*isCompilerInitFailure[\s\S]*\?\s*'compiler-init'/,
    "playground worker must report compiler initialization failure as a non-compile phase",
);
assert.match(
    worker,
    /recoverable:\s*phase\s*===\s*'compile'/,
    "playground worker must keep only user compile failures recoverable for the persistent worker",
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
assert.match(
    shell,
    /private\s+compilerWorker:\s*Worker\s*\|\s*null/,
    "playground shell must retain a compiler worker across compile requests",
);
assert.match(
    shell,
    /compilerWorkerForSession\(request\.compiler\)/,
    "playground shell must route execute-neplg2 through the persistent compiler worker",
);
assert.match(
    shell,
    /keepWorkerAlive:\s*true/,
    "playground shell must keep the compiler worker alive after successful compile requests",
);
assert.match(
    shell,
    /resolveCompilerMode\(parsed\.flags\)/,
    "playground shell must resolve compiler mode before building worker requests",
);
assert.match(
    shell,
    /compilerMode,\s*\n\s*compiler,/,
    "playground shell must send compiler mode through the worker protocol",
);
assert.match(
    shell,
    /worker\s*===\s*this\.compilerWorker\s*&&\s*!message\.recoverable[\s\S]*finish\(true\)/,
    "playground shell must terminate the persistent compiler worker after a worker-level error",
);
assert.match(
    shell,
    /phase\?:\s*'compile'\s*\|\s*'runtime'\s*\|\s*'worker'\s*\|\s*'compiler-init'/,
    "playground shell must accept explicit compiler initialization failure classification",
);

console.log("playground compiler session policy passed");
