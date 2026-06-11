#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { pathToFileURL } = require("node:url");

async function main() {
    const repo = path.resolve(__dirname, "..");
    const workerPath = path.join(repo, "web", "dist_ts", "runtime", "worker.js");
    if (!fs.existsSync(workerPath)) {
        throw new Error(`runtime worker bridge not found: ${workerPath}\nrun 'npm --prefix web run build:ts' first.`);
    }

    const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "neplg2-worker-recovery-"));
    const compilerPath = path.join(tempDir, "fake-compiler.mjs");
    fs.writeFileSync(compilerPath, `
let initCount = 0;

export default async function initCompiler() {
    initCount += 1;
    if (initCount === 1) {
        throw new Error("transient wasm-bindgen initialization failure");
    }
}

export function compile_outputs_with_vfs() {
    return {
        wasm: new Uint8Array([0, 97, 115, 109]),
        wat: "(module ;; recovered)"
    };
}
`, "utf8");

    const previousSelf = global.self;
    const posted = [];
    global.self = {
        onmessage: null,
        postMessage(message) {
            posted.push(message);
        },
    };

    try {
        await import(`${pathToFileURL(workerPath).href}?init-recovery=${Date.now()}`);
        assert.equal(typeof global.self.onmessage, "function", "runtime worker module must install self.onmessage");
        const request = {
            type: "execute-neplg2",
            compilerMode: "rust",
            compiler: {
                moduleUrl: pathToFileURL(compilerPath).href,
                wasmUrl: pathToFileURL(path.join(tempDir, "fake_bg.wasm")).href,
            },
            entryPath: "/main.nepl",
            source: "print \"ok\"",
            compileVfsData: { "/main.nepl": "print \"ok\"" },
            runtimeVfsData: { "/main.nepl": "print \"ok\"" },
            emitValues: ["wasm", "wat"],
            attachSource: false,
            runAfterBuild: false,
            runArgs: [],
            env: {},
            sab: null,
        };

        await global.self.onmessage({ data: request });
        assert.equal(posted.length, 1, "failed initialization should produce a single worker error message");
        assert.equal(posted[0].type, "error");
        assert.equal(posted[0].phase, "compiler-init");
        assert.equal(posted[0].recoverable, false);
        assert.match(posted[0].message, /transient wasm-bindgen initialization failure/);

        posted.length = 0;
        await global.self.onmessage({ data: request });
        assert.equal(posted[0].type, "compile_result", "worker must retry initialization instead of reusing a rejected promise");
        assert.equal(posted[0].outputs.wat, "(module ;; recovered)");
        assert.deepEqual(Array.from(posted[0].outputs.wasm), [0, 97, 115, 109]);
        assert.deepEqual(posted[1], { type: "exit", code: 0 });
    } finally {
        global.self = previousSelf;
        fs.rmSync(tempDir, { recursive: true, force: true });
    }

    console.log("playground worker init recovery regression passed");
}

main().catch((error) => {
    console.error(error && error.stack ? error.stack : String(error));
    process.exit(1);
});
