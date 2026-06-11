#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const repoRoot = path.resolve(__dirname, "..");
const repoMetricsTs = path.join(repoRoot, "repo_metrics.ts");

function run(cmd, args, options = {}) {
    return spawnSync(cmd, args, {
        cwd: repoRoot,
        encoding: "utf8",
        stdio: options.stdio || "pipe",
        shell: false,
    });
}

function printBuffered(proc) {
    if (proc.stdout) process.stdout.write(proc.stdout);
    if (proc.stderr) process.stderr.write(proc.stderr);
}

function supportsNativeStripTypes(proc) {
    if (proc.status === 0) return true;
    const stderr = String(proc.stderr || "");
    return !stderr.includes("bad option: --experimental-strip-types")
        && !stderr.includes("unknown option: --experimental-strip-types");
}

function runNativeStripTypes(args) {
    return run(process.execPath, ["--experimental-strip-types", repoMetricsTs, ...args]);
}

function tscJsPath() {
    return path.join(repoRoot, "web", "node_modules", "typescript", "bin", "tsc");
}

function compileRepoMetrics() {
    const outDir = path.join(repoRoot, "tmp", "repo_metrics_runner");
    fs.rmSync(outDir, { recursive: true, force: true });
    fs.mkdirSync(outDir, { recursive: true });
    const compiler = tscJsPath();
    if (!fs.existsSync(compiler)) {
        throw new Error("TypeScript compiler is missing. Run npm --prefix web ci before repo metrics generation.");
    }
    const proc = run(process.execPath, [
        compiler,
        repoMetricsTs,
        "--target", "ES2020",
        "--module", "commonjs",
        "--types", "node",
        "--typeRoots", path.join(repoRoot, "web", "node_modules", "@types"),
        "--esModuleInterop",
        "--skipLibCheck",
        "--outDir", outDir,
    ]);
    if (proc.status !== 0) {
        printBuffered(proc);
        throw new Error("failed to compile repo_metrics.ts");
    }
    return path.join(outDir, "repo_metrics.js");
}

function runCompiled(args) {
    const compiled = compileRepoMetrics();
    return run(process.execPath, [compiled, ...args], { stdio: "inherit" });
}

function main() {
    const args = process.argv.slice(2);
    if (process.env.NEPL_REPO_METRICS_FORCE_TSC === "1") {
        const compiled = runCompiled(args);
        return compiled.status || 0;
    }
    const native = runNativeStripTypes(args);
    if (native.status === 0 || supportsNativeStripTypes(native)) {
        printBuffered(native);
        return native.status || 0;
    }
    const compiled = runCompiled(args);
    return compiled.status || 0;
}

try {
    process.exitCode = main();
} catch (error) {
    console.error(String(error instanceof Error ? error.message : error));
    process.exitCode = 1;
}
