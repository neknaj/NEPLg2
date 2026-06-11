#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");

function usage(exitCode) {
    console.log("Usage: node nodesrc/complete_selfhost_doctest_artifact.js --marker <timeout.json> --json <out.json> --suite-id <id> --suite-label <label> [--matrix-inputs <inputs>]");
    process.exit(exitCode);
}

function parseArgs(argv) {
    const out = {
        markerPath: "",
        jsonPath: "",
        suiteId: "",
        suiteLabel: "",
        matrixInputs: "",
    };
    for (let i = 0; i < argv.length; i++) {
        const arg = argv[i];
        if (arg === "--marker" && i + 1 < argv.length) {
            out.markerPath = argv[++i];
            continue;
        }
        if (arg === "--json" && i + 1 < argv.length) {
            out.jsonPath = argv[++i];
            continue;
        }
        if (arg === "--suite-id" && i + 1 < argv.length) {
            out.suiteId = argv[++i];
            continue;
        }
        if (arg === "--suite-label" && i + 1 < argv.length) {
            out.suiteLabel = argv[++i];
            continue;
        }
        if (arg === "--matrix-inputs" && i + 1 < argv.length) {
            out.matrixInputs = argv[++i];
            continue;
        }
        if (arg === "-h" || arg === "--help") usage(0);
        throw new Error(`unknown argument: ${arg}`);
    }
    if (!out.markerPath || !out.jsonPath || !out.suiteId || !out.suiteLabel) usage(2);
    return out;
}

function readJson(filePath) {
    return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function writeJson(filePath, payload) {
    fs.mkdirSync(path.dirname(path.resolve(filePath)), { recursive: true });
    fs.writeFileSync(filePath, JSON.stringify(payload, null, 2));
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
        const timeout = result && typeof result.timeout === "object" && result.timeout !== null;
        if (result && result.skipped) summary.skipped += 1;
        if (timeout) summary.timed_out += 1;
        if (result && (result.status === "pass" || result.status === "passed" || result.status === "ok")) {
            summary.passed += 1;
        } else if (result && result.status === "error") {
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

function basePayload(options) {
    return {
        schema: "neplg2-selfhost-doctest/v1",
        generated_at: new Date().toISOString(),
        implementation: "selfhost",
        check_mode: "compiler_check",
        runtime_assertions: false,
        jobs: 1,
        scan: {
            inputs: options.matrixInputs ? [options.matrixInputs] : [],
            shard: null,
            batch_size: null,
            max_cases: null,
        },
        summary: summarize([]),
        duration_ms: null,
        results: [],
    };
}

function timeoutResult(options, marker) {
    return {
        ok: false,
        id: `selfhost-timeout:${options.suiteId}`,
        file: options.matrixInputs || options.suiteId,
        index: null,
        tags: [],
        status: "error",
        phase: "selfhost_timeout",
        implementation: "selfhost",
        check_mode: "compiler_check",
        runtime_assertions: false,
        expected: "complete selfhost compiler check",
        error: `${options.suiteLabel} timed out before a complete selfhost compiler-check report was produced.`,
        timeout: {
            timed_out: true,
            label: marker.label || options.suiteLabel,
            timeout_ms: marker.timeout_ms || null,
            generated_at: marker.generated_at || null,
        },
    };
}

function completeArtifact(options) {
    const markerExists = fs.existsSync(options.markerPath);
    const jsonExists = fs.existsSync(options.jsonPath);
    if (!markerExists) {
        if (!jsonExists) {
            throw new Error(`selfhost doctest JSON was not produced: ${options.jsonPath}`);
        }
        return false;
    }

    const marker = readJson(options.markerPath);
    const payload = jsonExists ? readJson(options.jsonPath) : basePayload(options);
    if (!Array.isArray(payload.results)) payload.results = [];
    const id = `selfhost-timeout:${options.suiteId}`;
    if (!payload.results.some((result) => result && result.id === id)) {
        payload.results.push(timeoutResult(options, marker));
    }
    payload.schema = payload.schema || "neplg2-selfhost-doctest/v1";
    payload.implementation = payload.implementation || "selfhost";
    payload.check_mode = payload.check_mode || "compiler_check";
    payload.runtime_assertions = false;
    payload.timed_out = true;
    payload.timeout = marker;
    payload.summary = summarize(payload.results);
    payload.completed_by = "complete_selfhost_doctest_artifact";
    writeJson(options.jsonPath, payload);
    return true;
}

if (require.main === module) {
    try {
        const completed = completeArtifact(parseArgs(process.argv.slice(2)));
        if (completed) console.log("[selfhost-doctest] wrote timeout report JSON");
    } catch (error) {
        console.error(String(error && error.stack ? error.stack : error));
        process.exit(1);
    }
}

module.exports = {
    completeArtifact,
    timeoutResult,
};
