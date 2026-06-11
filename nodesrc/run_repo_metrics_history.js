#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const repoRoot = path.resolve(__dirname, "..");
const SCHEMA = "neplg2-repo-metrics-history/v1";

function run(args, options = {}) {
    return spawnSync(args[0], args.slice(1), {
        cwd: options.cwd || repoRoot,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
        timeout: options.timeoutMs || 120000,
        maxBuffer: 64 * 1024 * 1024,
        shell: false,
    });
}

function requireOk(result, label) {
    if (result.status === 0 && !result.signal && !result.error) return result;
    const detail = [result.stderr, result.stdout, result.error && result.error.message]
        .filter(Boolean)
        .join("\n")
        .trim();
    throw new Error(`${label} failed${detail ? `\n${detail}` : ""}`);
}

function parseArgs(argv) {
    const args = {
        root: ".",
        ref: "HEAD",
        limit: 100,
        json: "",
        workRoot: path.join("tmp", "repo_metrics_history"),
        keepWorktrees: false,
        commandTimeoutMs: 120000,
    };
    for (let i = 0; i < argv.length; i++) {
        const a = argv[i];
        if (a === "--root" && i + 1 < argv.length) {
            args.root = argv[++i];
        } else if (a === "--ref" && i + 1 < argv.length) {
            args.ref = argv[++i];
        } else if (a === "--limit" && i + 1 < argv.length) {
            args.limit = Number(argv[++i]);
        } else if (a === "--json" && i + 1 < argv.length) {
            args.json = argv[++i];
        } else if (a === "--work-root" && i + 1 < argv.length) {
            args.workRoot = argv[++i];
        } else if (a === "--keep-worktrees") {
            args.keepWorktrees = true;
        } else if (a === "--command-timeout-ms" && i + 1 < argv.length) {
            args.commandTimeoutMs = Number(argv[++i]);
        } else if (a === "-h" || a === "--help") {
            return { ...args, help: true };
        } else {
            throw new Error(`unknown argument: ${a}`);
        }
    }
    if (!Number.isFinite(args.limit) || args.limit < 1) {
        throw new Error("--limit must be a positive number");
    }
    if (!Number.isFinite(args.commandTimeoutMs) || args.commandTimeoutMs < 1000) {
        throw new Error("--command-timeout-ms must be at least 1000");
    }
    return args;
}

function usage() {
    console.log("Usage: node nodesrc/run_repo_metrics_history.js --root <repo> --limit <n> --json <out.json>");
    console.log("");
    console.log("Samples approximately <n> commits evenly from all commits reachable from --ref.");
}

function ensureDir(dir) {
    fs.mkdirSync(dir, { recursive: true });
}

function writeJson(file, payload) {
    ensureDir(path.dirname(path.resolve(file)));
    fs.writeFileSync(file, `${JSON.stringify(payload, null, 2)}\n`, "utf8");
}

function safeName(text) {
    return String(text).replace(/[^A-Za-z0-9_.-]+/g, "_").replace(/^_+|_+$/g, "").slice(0, 80) || "ref";
}

function resolveGitRoot(root) {
    const result = requireOk(run(["git", "rev-parse", "--show-toplevel"], { cwd: root }), "git rev-parse");
    return path.resolve(result.stdout.trim());
}

function loadCommits(root, ref) {
    const result = requireOk(run([
        "git",
        "log",
        "--format=%H%x09%ct%x09%s",
        ref,
    ], { cwd: root }), "git log");
    return result.stdout
        .trim()
        .split(/\r?\n/)
        .filter(Boolean)
        .map((line) => {
            const parts = line.split("\t");
            return {
                commit: parts[0],
                committed_at: new Date((Number(parts[1]) || 0) * 1000).toISOString(),
                subject: parts.slice(2).join("\t"),
            };
        })
        .reverse();
}

function sampleCommits(commits, limit) {
    const target = Math.floor(limit);
    if (commits.length <= target) return commits;
    if (target === 1) return [commits[commits.length - 1]];

    const out = [];
    const last = commits.length - 1;
    let previousIndex = -1;
    for (let i = 0; i < target; i++) {
        const index = Math.round((i * last) / (target - 1));
        if (index === previousIndex) continue;
        out.push(commits[index]);
        previousIndex = index;
    }
    return out;
}

function sumRows(rows) {
    const totals = {
        files: 0,
        lines: 0,
        chars: 0,
        bytes: 0,
        blank: 0,
        source: 0,
        doc_comment: 0,
        document: 0,
        test: 0,
        comment: 0,
        other: 0,
        testCases: 0,
    };
    for (const row of rows || []) {
        for (const key of Object.keys(totals)) {
            totals[key] += Number(row[key] || 0);
        }
    }
    return totals;
}

function summarizeMetrics(metricsJson) {
    if (!metricsJson) return null;
    const byArea = Array.isArray(metricsJson.byArea) ? metricsJson.byArea : [];
    const byExtension = Array.isArray(metricsJson.byExtension) ? metricsJson.byExtension : [];
    const byContentKind = Array.isArray(metricsJson.byContentKind) ? metricsJson.byContentKind : [];
    return {
        totals: sumRows(byArea),
        by_area: Object.fromEntries(byArea.map((row) => [row.name, row])),
        by_extension: Object.fromEntries(byExtension.map((row) => [row.name, row])),
        by_content_kind: Object.fromEntries(byContentKind.map((row) => [row.name, row])),
    };
}

function isInside(parent, child) {
    const rel = path.relative(path.resolve(parent), path.resolve(child));
    return rel === "" || (!!rel && !rel.startsWith("..") && !path.isAbsolute(rel));
}

function removeWorktree(repo, worktreeRoot, worktreePath) {
    const resolved = path.resolve(worktreePath);
    const root = path.resolve(worktreeRoot);
    if (!isInside(root, resolved)) {
        throw new Error(`refusing to remove worktree outside ${root}: ${resolved}`);
    }
    run(["git", "worktree", "remove", "--force", resolved], { cwd: repo, timeoutMs: 120000 });
    if (fs.existsSync(resolved)) {
        fs.rmSync(resolved, { recursive: true, force: true });
    }
}

function collectOne(repo, args, commitInfo, index, runId, worktreeRoot, artifactsRoot) {
    const short = commitInfo.commit.slice(0, 12);
    const name = `${String(index + 1).padStart(2, "0")}-${short}`;
    const worktreePath = path.join(worktreeRoot, `${runId}-${name}`);
    const metricsOut = path.join(artifactsRoot, `${name}.json`);
    const commands = [];
    let metrics = null;
    let metricsError = null;

    try {
        const add = run(["git", "worktree", "add", "--detach", worktreePath, commitInfo.commit], {
            cwd: repo,
            timeoutMs: 120000,
        });
        commands.push({ kind: "git_worktree_add", status: add.status, signal: add.signal || null });
        requireOk(add, `git worktree add ${short}`);

        const metricsRun = run([
            process.execPath,
            path.join(repoRoot, "nodesrc", "run_repo_metrics.js"),
            "--root",
            worktreePath,
            "--mode",
            "git",
            "--json",
            metricsOut,
        ], {
            cwd: repo,
            timeoutMs: args.commandTimeoutMs,
        });
        commands.push({
            kind: "repo_metrics",
            status: metricsRun.status,
            signal: metricsRun.signal || null,
        });
        if (metricsRun.status === 0 && fs.existsSync(metricsOut)) {
            metrics = summarizeMetrics(JSON.parse(fs.readFileSync(metricsOut, "utf8")));
        } else {
            metricsError = [metricsRun.stderr, metricsRun.stdout, metricsRun.error && metricsRun.error.message]
                .filter(Boolean)
                .join("\n")
                .trim() || `repo metrics exited with ${metricsRun.status}`;
        }
    } catch (error) {
        metricsError = String(error && error.message ? error.message : error);
    } finally {
        if (!args.keepWorktrees && fs.existsSync(worktreePath)) {
            removeWorktree(repo, worktreeRoot, worktreePath);
        }
    }

    return {
        commit: commitInfo.commit,
        short,
        committed_at: commitInfo.committed_at,
        subject: commitInfo.subject,
        metrics,
        metrics_error: metricsError,
        commands,
    };
}

function buildDeltas(revisions) {
    const out = [];
    for (let i = 1; i < revisions.length; i++) {
        const prev = revisions[i - 1];
        const cur = revisions[i];
        const pt = prev.metrics && prev.metrics.totals;
        const ct = cur.metrics && cur.metrics.totals;
        if (!pt || !ct) {
            out.push({ commit: cur.commit, previous_commit: prev.commit, metrics: null });
            continue;
        }
        const metrics = {};
        for (const key of ["files", "lines", "source", "doc_comment", "document", "test", "testCases", "bytes"]) {
            metrics[key] = Number(ct[key] || 0) - Number(pt[key] || 0);
        }
        out.push({ commit: cur.commit, previous_commit: prev.commit, metrics });
    }
    return out;
}

function main() {
    const args = parseArgs(process.argv.slice(2));
    if (args.help || !args.json) {
        usage();
        return args.help ? 0 : 2;
    }

    const root = resolveGitRoot(path.resolve(args.root));
    const allCommits = loadCommits(root, args.ref);
    const commits = sampleCommits(allCommits, args.limit);
    const runId = `${Date.now()}-${process.pid}-${Math.random().toString(16).slice(2)}`;
    const workRoot = path.resolve(root, args.workRoot);
    const worktreeRoot = path.join(workRoot, "worktrees");
    const artifactsRoot = path.join(workRoot, "artifacts", safeName(runId));
    ensureDir(worktreeRoot);
    ensureDir(artifactsRoot);

    const revisions = commits.map((commit, index) => (
        collectOne(root, args, commit, index, runId, worktreeRoot, artifactsRoot)
    ));
    const payload = {
        schema: SCHEMA,
        generated_at: new Date().toISOString(),
        ref: args.ref,
        limit: args.limit,
        total_commit_count: allCommits.length,
        sampling: "even-reachable-commits",
        revision_count: revisions.length,
        revisions,
        deltas: buildDeltas(revisions),
    };
    writeJson(args.json, payload);
    console.log(JSON.stringify({
        schema: payload.schema,
        revisions: payload.revision_count,
        total_commit_count: payload.total_commit_count,
        sampling: payload.sampling,
        json: args.json,
        errors: revisions.filter((r) => r.metrics_error).length,
    }, null, 2));
    return 0;
}

try {
    process.exitCode = main();
} catch (error) {
    console.error(String(error && error.stack ? error.stack : error));
    process.exitCode = 1;
}

