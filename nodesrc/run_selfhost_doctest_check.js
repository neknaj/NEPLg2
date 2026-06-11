#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { parseFile } = require("./parser");
const { createRunner, runSingle } = require("./run_test");

function usage(exitCode) {
    console.log("Usage: node nodesrc/run_selfhost_doctest_check.js -i <dir_or_file> [-i ...] -o <out.json> [--dist <distDirHint>] [-j N] [--shard INDEX/TOTAL] [--batch-size N] [--max-cases N] [--failure-nonfatal]");
    process.exit(exitCode);
}

function parseArgs(argv) {
    const inputs = [];
    let outPath = "";
    let distHint = "";
    let jobs = 1;
    let shard = null;
    let batchSize = 20;
    let maxCases = 0;
    let failureNonfatal = false;
    for (let i = 0; i < argv.length; i++) {
        const arg = argv[i];
        if (arg === "-i" && i + 1 < argv.length) {
            inputs.push(argv[++i]);
            continue;
        }
        if (arg === "-o" && i + 1 < argv.length) {
            outPath = argv[++i];
            continue;
        }
        if (arg === "--dist" && i + 1 < argv.length) {
            distHint = argv[++i];
            continue;
        }
        if (arg === "-j" && i + 1 < argv.length) {
            jobs = Math.max(1, Number.parseInt(argv[++i], 10) || 1);
            continue;
        }
        if (arg === "--shard" && i + 1 < argv.length) {
            shard = parseShard(argv[++i]);
            continue;
        }
        if (arg === "--batch-size" && i + 1 < argv.length) {
            batchSize = Math.max(1, Number.parseInt(argv[++i], 10) || 1);
            continue;
        }
        if (arg === "--max-cases" && i + 1 < argv.length) {
            maxCases = Math.max(0, Number.parseInt(argv[++i], 10) || 0);
            continue;
        }
        if (arg === "--failure-nonfatal") {
            failureNonfatal = true;
            continue;
        }
        if (arg === "--timeout-nonfatal" || arg === "--no-tree" || arg === "--with-tree") {
            continue;
        }
        if (arg === "-h" || arg === "--help") {
            usage(0);
        }
        throw new Error(`unknown argument: ${arg}`);
    }
    if (inputs.length === 0 || !outPath) usage(2);
    return { inputs, outPath, distHint, jobs, shard, batchSize, maxCases, failureNonfatal };
}

function parseShard(raw) {
    const match = String(raw || "").match(/^([1-9][0-9]*)\/([1-9][0-9]*)$/);
    if (!match) throw new Error(`--shard must be INDEX/TOTAL, got: ${raw}`);
    const index = Number(match[1]);
    const total = Number(match[2]);
    if (index > total) throw new Error(`--shard index must be <= total, got: ${raw}`);
    return { index, total };
}

function isFile(filePath) {
    try {
        return fs.statSync(filePath).isFile();
    } catch {
        return false;
    }
}

function isDir(filePath) {
    try {
        return fs.statSync(filePath).isDirectory();
    } catch {
        return false;
    }
}

function walkFiles(root) {
    const out = [];
    function rec(cur) {
        const entries = fs.readdirSync(cur, { withFileTypes: true });
        for (const entry of entries) {
            const filePath = path.join(cur, entry.name);
            if (entry.isDirectory()) rec(filePath);
            else if (entry.isFile()) out.push(filePath);
        }
    }
    rec(root);
    return out;
}

function collectCases(inputPath) {
    const abs = path.resolve(inputPath);
    const files = [];
    if (isFile(abs)) {
        files.push(abs);
    } else if (isDir(abs)) {
        for (const filePath of walkFiles(abs)) {
            if (filePath.endsWith(".n.md") || filePath.endsWith(".nepl")) files.push(filePath);
        }
    }
    const cases = [];
    for (const filePath of files.sort()) {
        const parsed = parseFile(filePath);
        for (let i = 0; i < parsed.doctests.length; i++) {
            const doctest = parsed.doctests[i];
            cases.push({
                id: `${path.relative(process.cwd(), filePath)}::doctest#${i + 1}`,
                file: path.relative(process.cwd(), filePath),
                index: i + 1,
                source: doctest.code,
                tags: doctest.tags || [],
            });
        }
    }
    return cases;
}

function applyShard(cases, shard) {
    if (!shard) return cases;
    return cases.filter((_case, index) => (index % shard.total) === (shard.index - 1));
}

function hasTag(tags, name) {
    return Array.isArray(tags) && tags.includes(name);
}

function neplStringLiteral(value) {
    return JSON.stringify(String(value || ""));
}

function selfhostBatchHarnessSource(cases) {
    const calls = cases.map((testCase, index) => {
        const sourceLiteral = neplStringLiteral(testCase.source);
        return `    let c${index} %i32 selfhost_check_source ${sourceLiteral}
    print_i32 c${index};
    print "\\n";`;
    }).join("\n");
    return `#entry main
#target std
#indent 4
#import "core/result" as *
#import "std/stdio" as *
#import "neplg2/core/module/loader" as *
#import "neplg2/core/options" as *
#import "neplg2/core/pipeline" as *

fn selfhost_check_source %impure fn str i32 \\source:
    match selfhost_vfs_new:
        Result::Ok vfs0:
            match selfhost_vfs_add vfs0 "main.nepl" source:
                Result::Ok vfs1:
                    let options %SelfhostCompileOptions selfhost_compile_options_default
                    let request %SelfhostCompileRequest selfhost_compile_request_new "main.nepl" options
                    match selfhost_pipeline_load_root &vfs1 request:
                        Result::Ok loaded:
                            match selfhost_pipeline_check_loaded_root &loaded:
                                Result::Ok _summary:
                                    selfhost_pipeline_loaded_root_free loaded
                                    selfhost_vfs_free vfs1
                                    0
                                Result::Err _diag:
                                    selfhost_pipeline_loaded_root_free loaded
                                    selfhost_vfs_free vfs1
                                    2
                        Result::Err _diag:
                            selfhost_vfs_free vfs1
                            2
                Result::Err _err:
                    3
        Result::Err _err:
            3

fn main %impure fn void i32 \\void:
${calls}
    0
`;
}

function summarize(results) {
    const summary = {
        total: results.length,
        passed: 0,
        failed: 0,
        errored: 0,
        skipped: 0,
        timed_out: 0,
        timeout_failed: 0,
        timeout_errored: 0,
        non_timeout_failed: 0,
        non_timeout_errored: 0,
    };
    for (const result of results) {
        if (result.skipped) summary.skipped += 1;
        const timeout = result && typeof result.timeout === "object" && result.timeout !== null;
        if (timeout) summary.timed_out += 1;
        if (result.status === "pass" || result.status === "passed" || result.status === "ok") {
            summary.passed += 1;
        } else if (result.status === "error") {
            summary.errored += 1;
            if (timeout) summary.timeout_errored += 1;
            else summary.non_timeout_errored += 1;
        } else {
            summary.failed += 1;
            if (timeout) summary.timeout_failed += 1;
            else summary.non_timeout_failed += 1;
        }
    }
    return summary;
}

function selfhostExitCode(runResult) {
    if (typeof runResult.return_value === "number") return runResult.return_value;
    if (typeof runResult.exit_code === "number") return runResult.exit_code;
    return null;
}

function convertRunResult(testCase, runResult) {
    const expectedReject = hasTag(testCase.tags, "compile_fail");
    const code = selfhostExitCode(runResult);
    const selfhostAccepted = code === 0;
    const selfhostRejected = code === 2;
    const harnessError = code === 3 || code === null;
    const ok = expectedReject ? selfhostRejected : selfhostAccepted;
    const status = harnessError ? "error" : ok ? "pass" : "fail";
    const error = ok
        ? null
        : harnessError
            ? `selfhost harness failed or returned an unknown code: ${code === null ? "null" : code}`
            : expectedReject
                ? "expected selfhost compiler rejection, but selfhost check accepted the source"
                : "expected selfhost compiler acceptance, but selfhost check rejected the source";
    return {
        ok,
        id: `${testCase.id}::selfhost-check`,
        file: testCase.file,
        index: testCase.index,
        tags: testCase.tags,
        status,
        phase: "selfhost_check",
        implementation: "selfhost",
        check_mode: "compiler_check",
        expected: expectedReject ? "reject" : "accept",
        selfhost_exit_code: code,
        error,
        compiler: runResult.compiler || null,
        timing: runResult.timing || null,
        duration_ms: runResult.duration_ms || null,
        harness_status: runResult.status || null,
        harness_phase: runResult.phase || null,
        harness_error: runResult.error || null,
    };
}

function skippedResult(testCase) {
    return {
        ok: true,
        id: `${testCase.id}::selfhost-check`,
        file: testCase.file,
        index: testCase.index,
        tags: testCase.tags,
        status: "pass",
        phase: "selfhost_skip",
        implementation: "selfhost",
        check_mode: "compiler_check",
        skipped: true,
        error: null,
    };
}

function harnessErrorResult(testCase, runResult) {
    return {
        ok: false,
        id: `${testCase.id}::selfhost-check`,
        file: testCase.file,
        index: testCase.index,
        tags: testCase.tags,
        status: "error",
        phase: "selfhost_harness",
        implementation: "selfhost",
        check_mode: "compiler_check",
        error: runResult.error || "selfhost harness did not run successfully",
        compiler: runResult.compiler || null,
        timing: runResult.timing || null,
        duration_ms: runResult.duration_ms || null,
        harness_status: runResult.status || null,
        harness_phase: runResult.phase || null,
    };
}

async function runBatch(batch, preloaded, batchIndex) {
    const active = batch.filter((testCase) => !(hasTag(testCase.tags, "skip") || hasTag(testCase.tags, "skip_selfhost")));
    const skipped = new Map();
    for (const testCase of batch) {
        if (!active.includes(testCase)) skipped.set(testCase.id, skippedResult(testCase));
    }
    if (active.length === 0) return batch.map((testCase) => skipped.get(testCase.id));
    const harness = selfhostBatchHarnessSource(active);
    const runResult = await runSingle({
        id: `selfhost-batch#${batchIndex + 1}`,
        file: active[0] ? active[0].file : "nodesrc/run_selfhost_doctest_check.js",
        source: harness,
        tags: [],
        forceStdlibVfs: true,
    }, preloaded);
    const byId = new Map(skipped);
    if (runResult.status !== "pass") {
        for (const testCase of active) byId.set(testCase.id, harnessErrorResult(testCase, runResult));
        return batch.map((testCase) => byId.get(testCase.id));
    }
    const codes = String(runResult.stdout || "")
        .split(/\r?\n/)
        .filter((line) => line.trim().length > 0)
        .map((line) => Number.parseInt(line.trim(), 10));
    for (let i = 0; i < active.length; i++) {
        const testCase = active[i];
        const code = Number.isFinite(codes[i]) ? codes[i] : null;
        byId.set(testCase.id, convertRunResult(testCase, {
            ...runResult,
            return_value: code,
            exit_code: code,
        }));
    }
    return batch.map((testCase) => byId.get(testCase.id));
}

async function runCase(testCase, preloaded) {
    if (hasTag(testCase.tags, "skip") || hasTag(testCase.tags, "skip_selfhost")) {
        return skippedResult(testCase);
    }
    const harness = selfhostBatchHarnessSource([testCase]);
    const runResult = await runSingle({
        id: `${testCase.id}::selfhost-harness`,
        file: testCase.file,
        source: harness,
        tags: [],
        forceStdlibVfs: true,
    }, preloaded);
    if (runResult.status !== "pass") {
        return harnessErrorResult(testCase, runResult);
    }
    const code = Number.parseInt(String(runResult.stdout || "").trim(), 10);
    return convertRunResult(testCase, {
        ...runResult,
        return_value: Number.isFinite(code) ? code : null,
        exit_code: Number.isFinite(code) ? code : null,
    });
}

async function mapConcurrent(items, limit, fn) {
    const results = new Array(items.length);
    let next = 0;
    async function worker() {
        while (true) {
            const index = next++;
            if (index >= items.length) return;
            results[index] = await fn(items[index], index);
        }
    }
    const workers = [];
    for (let i = 0; i < Math.max(1, limit); i++) workers.push(worker());
    await Promise.all(workers);
    return results;
}

async function main() {
    const options = parseArgs(process.argv.slice(2));
    let cases = [];
    for (const input of options.inputs) cases.push(...collectCases(input));
    const allCasesBeforeShard = cases.length;
    cases = applyShard(cases, options.shard);
    if (options.maxCases > 0) cases = cases.slice(0, options.maxCases);
    const preloaded = await createRunner(options.distHint || "");
    const startedAt = Date.now();
    const batches = [];
    for (let i = 0; i < cases.length; i += options.batchSize) {
        batches.push(cases.slice(i, i + options.batchSize));
    }
    const batchResults = await mapConcurrent(batches, options.jobs, async (batch, index) => {
        const result = await runBatch(batch, preloaded, index);
        const done = Math.min(cases.length, (index + 1) * options.batchSize);
        if ((index + 1) % 5 === 0 || index + 1 === batches.length) {
            console.log(`[selfhost-doctest] ${done}/${cases.length}`);
        }
        return result;
    });
    const results = batchResults.flat();
    const payload = {
        schema: "neplg2-selfhost-doctest/v1",
        generated_at: new Date().toISOString(),
        implementation: "selfhost",
        check_mode: "compiler_check",
        runtime_assertions: false,
        jobs: options.jobs,
        scan: {
            inputs: options.inputs.map((input) => path.relative(process.cwd(), path.resolve(input))),
            shard: options.shard ? { ...options.shard, all_cases_before_shard: allCasesBeforeShard, cases_after_shard: cases.length } : null,
            batch_size: options.batchSize,
            max_cases: options.maxCases || null,
        },
        summary: summarize(results),
        duration_ms: Date.now() - startedAt,
        results,
    };
    fs.mkdirSync(path.dirname(path.resolve(options.outPath)), { recursive: true });
    fs.writeFileSync(options.outPath, JSON.stringify(payload, null, 2));
    if (!options.failureNonfatal && (payload.summary.failed > 0 || payload.summary.errored > 0)) {
        process.exitCode = 1;
    }
}

if (require.main === module) {
    main().catch((error) => {
        console.error(String(error && error.stack ? error.stack : error));
        process.exit(1);
    });
}

module.exports = {
    collectCases,
    selfhostBatchHarnessSource,
    summarize,
};
