#!/usr/bin/env node
"use strict";

const path = require("path");
const fs = require("fs");
const { spawnSync } = require("child_process");
const { pathToFileURL } = require("url");

const repoRoot = path.resolve(__dirname, "..");

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

function assertEqual(actual, expected, message) {
    if (actual !== expected) {
        throw new Error(`${message}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
    }
}

function assertUint8Array(value, message) {
    assert(value instanceof Uint8Array, message);
}

function compiledModuleUrl(relPath) {
    return pathToFileURL(path.join(repoRoot, "web", "dist_ts", relPath)).href;
}

function ensureCompiledWebDistTs() {
    const requiredModule = path.join(repoRoot, "web", "dist_ts", "gui-font", "font-resource-vfs.js");
    if (fs.existsSync(requiredModule)) {
        return;
    }
    const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";
    const result = spawnSync(npmCommand, ["--prefix", "web", "run", "build:ts"], {
        cwd: repoRoot,
        stdio: "inherit",
    });
    if (result.status !== 0) {
        throw new Error(`build:ts failed with status ${result.status}`);
    }
    assert(fs.existsSync(requiredModule), "compiled font resource VFS module must exist after build:ts");
}

function binaryResponse(bytes, init) {
    return new Response(new Uint8Array(bytes).buffer, init);
}

function textResponse(text, init) {
    return new Response(text, init);
}

function responseForSuccess(url) {
    if (url.endsWith("HackGenConsoleNF-Regular.ttf")) {
        return binaryResponse([0, 1, 2, 3]);
    }
    if (url.endsWith("HackGen-LICENSE.txt")) {
        return textResponse("license");
    }
    return textResponse("missing", { status: 404 });
}

async function expectMountError(mountBundledGuiFontResources, vfs, expectedKind, fetchResource, options = {}) {
    const result = await mountBundledGuiFontResources(vfs, {
        fetch: fetchResource,
        ...options,
    });
    assert(!result.ok, `mount must fail with ${expectedKind}`);
    assertEqual(result.error.kind, expectedKind, `mount error kind`);
    return result.error;
}

class FailingReadOnlyVfs {
    constructor(base, pathToFail) {
        this.base = base;
        this.pathToFail = pathToFail;
    }

    writeFile(path, content, options) {
        return this.base.writeFile(path, content, options);
    }

    setReadOnly(path, readOnly) {
        if (path === this.pathToFail && readOnly) {
            throw new Error("forced read-only failure");
        }
        return this.base.setReadOnly(path, readOnly);
    }

    deleteFile(path) {
        return this.base.deleteFile(path);
    }
}

async function main() {
    ensureCompiledWebDistTs();
    const { VFS } = await import(compiledModuleUrl("runtime/vfs.js"));
    const { Shell } = await import(compiledModuleUrl("terminal/shell.js"));
    const {
        BUNDLED_GUI_FONT_RESOURCES,
        HACKGEN_CONSOLE_REGULAR_RESOURCE_PATH,
        bundledGuiFontResourcePaths,
        guiFontResourceVfsPath,
        mountBundledGuiFontResources,
        normalizeGuiFontResourcePath,
    } = await import(compiledModuleUrl("gui-font/font-resource-vfs.js"));

    const pathResult = normalizeGuiFontResourcePath(HACKGEN_CONSOLE_REGULAR_RESOURCE_PATH);
    assert(pathResult.ok, "canonical HackGen resource path must normalize");
    assertEqual(guiFontResourceVfsPath(pathResult.path), "/fonts/HackGenConsoleNF-Regular.ttf", "VFS path");
    assertEqual(bundledGuiFontResourcePaths()[0], "fonts/HackGenConsoleNF-Regular.ttf", "public bundled path");

    const successVfs = new VFS();
    successVfs.writeFile("/examples/app.nepl", "fn main:", { force: true });
    const success = await mountBundledGuiFontResources(successVfs, { fetch: async (url) => responseForSuccess(url) });
    assert(success.ok, "mount should succeed");
    assert(success.mountedPaths.includes("/fonts/HackGenConsoleNF-Regular.ttf"), "font path should be mounted");
    assert(success.mountedPaths.includes("/fonts/HackGen-LICENSE.txt"), "license path should be mounted");
    assert(successVfs.isReadOnly("/fonts/HackGenConsoleNF-Regular.ttf"), "font file should be read-only");
    assert(successVfs.isReadOnly("/fonts/HackGen-LICENSE.txt"), "license file should be read-only");
    assertUint8Array(successVfs.readFile("/fonts/HackGenConsoleNF-Regular.ttf"), "font file should be binary");
    assertEqual(successVfs.readFile("/fonts/HackGen-LICENSE.txt"), "license", "license text should be mounted");
    const compileOverlay = successVfs.serializeForCompile();
    assertEqual(compileOverlay["/examples/app.nepl"], "fn main:", "editable NEPL source should stay in compile overlay");
    assert(!Object.prototype.hasOwnProperty.call(compileOverlay, "/fonts/HackGenConsoleNF-Regular.ttf"), "font binary must not enter compile overlay");
    assert(!Object.prototype.hasOwnProperty.call(compileOverlay, "/fonts/HackGen-LICENSE.txt"), "read-only license must not enter compile overlay");

    await expectMountError(
        mountBundledGuiFontResources,
        new VFS(),
        "HttpError",
        async () => textResponse("not found", { status: 404 }),
    );

    await expectMountError(
        mountBundledGuiFontResources,
        new VFS(),
        "InvalidBytes",
        async (url) => url.endsWith(".ttf") ? binaryResponse([]) : textResponse("license"),
    );

    await expectMountError(
        mountBundledGuiFontResources,
        new VFS(),
        "InvalidText",
        async (url) => url.endsWith(".ttf") ? binaryResponse([1]) : textResponse(""),
    );

    const invalidPathError = await expectMountError(
        mountBundledGuiFontResources,
        new VFS(),
        "InvalidResourcePath",
        async (url) => responseForSuccess(url),
        {
            resources: [{
                resourcePath: "/fonts/Bad.ttf",
                vfsPath: "/fonts/Bad.ttf",
                sourceUrl: "./src/fonts/Bad.ttf",
                payloadKind: "binary",
            }],
        },
    );
    assertEqual(invalidPathError.reason, "Absolute", "absolute resource path must be rejected");

    const mismatchError = await expectMountError(
        mountBundledGuiFontResources,
        new VFS(),
        "InvalidResourcePath",
        async (url) => responseForSuccess(url),
        {
            resources: [{
                resourcePath: "fonts/Bad.ttf",
                vfsPath: "/other/Bad.ttf",
                sourceUrl: "./src/fonts/Bad.ttf",
                payloadKind: "binary",
            }],
        },
    );
    assertEqual(mismatchError.reason, "VfsPathMismatch", "mismatched VFS path must be rejected");

    const rollbackBase = new VFS();
    const failingVfs = new FailingReadOnlyVfs(rollbackBase, "/fonts/HackGen-LICENSE.txt");
    await expectMountError(
        mountBundledGuiFontResources,
        failingVfs,
        "VfsWriteFailed",
        async (url) => responseForSuccess(url),
        { resources: BUNDLED_GUI_FONT_RESOURCES },
    );
    assert(!rollbackBase.exists("/fonts/HackGenConsoleNF-Regular.ttf"), "rollback must remove staged font file");
    assert(!rollbackBase.exists("/fonts/HackGen-LICENSE.txt"), "rollback must remove current failed file");
    assert(!rollbackBase.isReadOnly("/fonts/HackGenConsoleNF-Regular.ttf"), "rollback must clear staged read-only flag");

    const shell = new Shell({ print() {}, write() {}, printError() {} }, new VFS(), {
        beforeWasmExecution: async () => "blocked by font resource preflight",
    });
    const originalWorker = globalThis.Worker;
    let workerConstructed = false;
    globalThis.Worker = class {
        constructor() {
            workerConstructed = true;
            throw new Error("worker should not be constructed");
        }
    };
    try {
        await shell.runWorkerProcess({
            type: "run-wasm",
            bin: new Uint8Array([0]),
            args: [],
            env: {},
            vfsData: {},
            sab: null,
            guiSab: null,
        });
        throw new Error("run-wasm preflight must reject");
    } catch (error) {
        assert(String(error && error.message || error).includes("blocked by font resource preflight"), "preflight message should reject run-wasm");
    } finally {
        globalThis.Worker = originalWorker;
        shell.dispose();
    }
    assert(!workerConstructed, "preflight must block before creating a Worker");

    console.log("web GUI font resource VFS behavior passed");
}

main().catch((error) => {
    console.error(error && (error.stack || error));
    process.exit(1);
});
