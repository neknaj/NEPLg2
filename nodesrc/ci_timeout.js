#!/usr/bin/env node
"use strict";

const { spawn } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

function usage(exitCode) {
    console.log("Usage: node nodesrc/ci_timeout.js --minutes <n> [--label <name>] [--timeout-marker <path>] [--timeout-nonfatal] [--fail-on-timeout] -- <command> [args...]");
    process.exit(exitCode);
}

function parsePositiveNumber(raw, name) {
    const n = Number(raw);
    if (!Number.isFinite(n) || n <= 0) {
        throw new Error(`${name} must be a positive number`);
    }
    return n;
}

function parseArgs(argv) {
    let timeoutMs = 0;
    let label = "";
    let failOnTimeout = true;
    let timeoutMarker = "";
    let commandIndex = -1;

    for (let i = 0; i < argv.length; i++) {
        const a = argv[i];
        if (a === "--") {
            commandIndex = i + 1;
            break;
        }
        if (a === "--minutes" && i + 1 < argv.length) {
            timeoutMs = parsePositiveNumber(argv[++i], "--minutes") * 60 * 1000;
            continue;
        }
        if (a === "--timeout-ms" && i + 1 < argv.length) {
            timeoutMs = parsePositiveNumber(argv[++i], "--timeout-ms");
            continue;
        }
        if (a === "--label" && i + 1 < argv.length) {
            label = String(argv[++i]);
            continue;
        }
        if (a === "--timeout-marker" && i + 1 < argv.length) {
            timeoutMarker = String(argv[++i]);
            continue;
        }
        if (a === "--timeout-nonfatal") {
            failOnTimeout = false;
            continue;
        }
        if (a === "--fail-on-timeout") {
            failOnTimeout = true;
            continue;
        }
        if (a === "-h" || a === "--help") {
            usage(0);
        }
        throw new Error(`unknown argument: ${a}`);
    }

    if (timeoutMs <= 0) throw new Error("--minutes or --timeout-ms is required");
    if (commandIndex < 0 || commandIndex >= argv.length) throw new Error("command after -- is required");

    const command = argv[commandIndex];
    const args = argv.slice(commandIndex + 1);
    return {
        timeoutMs,
        label: label || command,
        failOnTimeout,
        timeoutMarker,
        command,
        args,
    };
}

function escapeGitHubAnnotationValue(value) {
    return String(value || "")
        .replace(/%/g, "%25")
        .replace(/\r/g, "%0D")
        .replace(/\n/g, "%0A")
        .replace(/:/g, "%3A");
}

function printTimeoutWarning(label, timeoutMs) {
    const seconds = Math.round(timeoutMs / 1000);
    const message = `${label} timed out after ${seconds}s`;
    if (process.env.GITHUB_ACTIONS === "true") {
        console.log(`::warning title=CI command timeout::${escapeGitHubAnnotationValue(message)}`);
    } else {
        console.warn(`[ci-timeout] warning: ${message}`);
    }
}

function writeTimeoutMarker(markerPath, label, timeoutMs) {
    if (!markerPath) return;
    const resolved = path.resolve(markerPath);
    fs.mkdirSync(path.dirname(resolved), { recursive: true });
    fs.writeFileSync(resolved, JSON.stringify({
        timed_out: true,
        label,
        timeout_ms: timeoutMs,
        generated_at: new Date().toISOString(),
    }, null, 2));
}

function killChild(child, signal) {
    if (!child || !child.pid) return;
    try {
        if (process.platform === "win32") {
            child.kill(signal);
        } else {
            process.kill(-child.pid, signal);
        }
    } catch {
        try { child.kill(signal); } catch {}
    }
}

function runWithTimeout(options) {
    return new Promise((resolve) => {
        const child = spawn(options.command, options.args, {
            stdio: "inherit",
            shell: false,
            detached: process.platform !== "win32",
        });
        let timedOut = false;
        let settled = false;

        const killTimer = setTimeout(() => {
            if (settled) return;
            timedOut = true;
            killChild(child, "SIGTERM");
            setTimeout(() => {
                if (!settled) {
                    killChild(child, "SIGKILL");
                }
            }, 5000).unref();
        }, options.timeoutMs);

        child.on("error", (error) => {
            if (settled) return;
            settled = true;
            clearTimeout(killTimer);
            console.error(String(error?.stack || error?.message || error));
            resolve(1);
        });

        child.on("exit", (code, signal) => {
            if (settled) return;
            settled = true;
            clearTimeout(killTimer);
            if (timedOut) {
                writeTimeoutMarker(options.timeoutMarker, options.label, options.timeoutMs);
                printTimeoutWarning(options.label, options.timeoutMs);
                resolve(options.failOnTimeout ? 124 : 0);
                return;
            }
            if (signal) {
                console.error(`[ci-timeout] ${options.label} terminated by signal ${signal}`);
                resolve(1);
                return;
            }
            resolve(Number.isInteger(code) ? code : 1);
        });
    });
}

async function main() {
    const options = parseArgs(process.argv.slice(2));
    const code = await runWithTimeout(options);
    process.exitCode = code;
}

if (require.main === module) {
    main().catch((error) => {
        console.error(String(error?.stack || error?.message || error));
        process.exit(1);
    });
}

module.exports = {
    parseArgs,
    runWithTimeout,
};
