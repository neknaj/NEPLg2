#!/usr/bin/env node
"use strict";

const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
}

function assertIncludes(text, needle, message) {
    assert(text.includes(needle), `${message}\nexpected to find:\n${needle}`);
}

function assertNotIncludes(text, needle, message) {
    assert(!text.includes(needle), `${message}\nunexpected text:\n${needle}`);
}

function jobBlock(name) {
    const startMarker = `\n    ${name}:\n`;
    const start = workflow.indexOf(startMarker);
    assert(start >= 0, `ci workflow must define ${name} job`);
    const bodyStart = start + startMarker.length;
    const nextJob = workflow.slice(bodyStart).search(/\n    [A-Za-z0-9_-]+:\n/);
    const end = nextJob >= 0 ? bodyStart + nextJob : workflow.length;
    return workflow.slice(start, end);
}

const workflow = read(".github/workflows/ci.yml");
const index = read("web/index.html");
const tests = read("web/tests.html");
const testinfo = read("web/testinfo.html");
const metrics = read("web/metrics.html");
const runner = read("nodesrc/run_repo_metrics.js");
const historyRunner = read("nodesrc/run_repo_metrics_history.js");
const selfhostRunner = read("nodesrc/run_selfhost_doctest_check.js");
const selfhostCompleter = read("nodesrc/complete_selfhost_doctest_artifact.js");
const buildJob = jobBlock("build");

assertIncludes(index, '<link data-trunk rel="copy-file" href="metrics.html" />', "Trunk must publish metrics.html at the Pages root");
assertIncludes(index, '<link data-trunk rel="copy-file" href="tests.html" />', "Trunk must keep publishing tests.html");
assertIncludes(index, '<link data-trunk rel="copy-file" href="testinfo.html" />', "Trunk must keep publishing testinfo.html");
assertIncludes(index, 'href="./metrics.html"', "Playground header should link to the metrics page");

assertIncludes(workflow, "Build repository metrics into dist", "CI must generate repo metrics for Pages");
assertIncludes(buildJob, "fetch-depth: 0", "metrics history generation needs full first-parent commit history in the build job");
assertIncludes(workflow, "Verify playground HTML in Pages dist", "CI must verify Trunk output before uploading Pages artifacts");
assertIncludes(workflow, "if [ -d web/dist ]; then", "CI may normalize Trunk versions that write under web/dist");
assertIncludes(workflow, "cp -a web/dist/. dist/", "CI should copy web/dist only after checking that it exists");
assertIncludes(workflow, "test -f dist/metrics.html", "CI must verify metrics.html is present in the Pages artifact root");
assertIncludes(workflow, "node nodesrc/run_repo_metrics.js --root . --json dist/metrics/repo_metrics.json --csv dist/metrics/repo_metrics.csv", "CI must run repo_metrics.ts through the wrapper");
assertIncludes(workflow, "node nodesrc/run_repo_metrics_history.js --root . --limit 100 --json dist/metrics/repo_metrics_history.json", "CI must publish approximately 100 sampled historical repo metrics for the Pages chart");
assertIncludes(workflow, "selfhost-doctest:", "CI must publish selfhost compiler check doctest artifacts");
assertIncludes(workflow, "Selfhost compiler check doctests", "CI must label selfhost doctests as compiler checks");
assertIncludes(workflow, '--timeout-marker "${timeout_marker}"', "selfhost timeout-nonfatal runs must emit a timeout marker");
assertIncludes(workflow, "node nodesrc/complete_selfhost_doctest_artifact.js", "selfhost timeout markers must be converted into doctest JSON artifacts");
assertIncludes(workflow, "selfhost-wasi-tests.json", "CI must publish selfhost WASI doctest check JSON");
assertIncludes(workflow, "selfhost-nmd-tests.json", "CI must publish selfhost .n.md doctest check JSON");
assertIncludes(workflow, "selfhost-tutorials-tests.json", "CI must publish selfhost tutorial doctest check JSON");
assertIncludes(workflow, "selfhost-examples-tests.json", "CI must publish selfhost example doctest check JSON");
assertIncludes(workflow, "selfhost-stdlib-tests.json", "CI must publish selfhost stdlib doctest check JSON");
assertIncludes(workflow, "selfhost-llvm-tests.json", "CI must publish selfhost LLVM doctest check JSON");
assertIncludes(workflow, '"run_url": "${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}"', "status.json should expose the Actions run URL");
assertIncludes(workflow, '"head_sha": "${{ github.sha }}"', "status.json should expose the source revision");
assertIncludes(workflow, "dist/tests/last-completed", "pending Pages artifacts must preserve the latest completed test JSON files");
assertIncludes(workflow, '"fallback_status_url": "./tests/last-completed/status.json"', "pending status must expose the last completed status location");
assertIncludes(workflow, '"fallback_results_prefix": "./tests/last-completed/"', "pending status must expose the last completed result prefix");
assertIncludes(workflow, "github.ref == 'refs/heads/main'", "Pages deployment jobs must be restricted to main");
assertIncludes(workflow, "!cancelled()", "Pages deployment jobs must not publish cancelled workflow runs");
assertIncludes(workflow, "needs.build.result == 'success'", "Final Pages bundle must require a successful build artifact");
assertIncludes(workflow, "Guard current main SHA before pending Pages artifact", "Pending Pages artifact must refuse stale main revisions");
assertIncludes(workflow, "Guard current main SHA before final Pages artifact", "Final Pages artifact must refuse stale main revisions");
assertIncludes(workflow, 'git ls-remote "${{ github.server_url }}/${{ github.repository }}.git" refs/heads/main', "Pages artifacts must compare github.sha with current remote main");
assertIncludes(workflow, "needs.pages-final-bundle.result == 'success'", "Final Pages deploy must not run when final artifact creation failed");

assertIncludes(runner, "repo_metrics.ts", "wrapper must keep repo_metrics.ts as the metrics authority");
assertIncludes(runner, "--experimental-strip-types", "wrapper should use native TS execution when available");
assertIncludes(runner, "node_modules", "wrapper should fall back to the repo TypeScript toolchain");
assertIncludes(historyRunner, "neplg2-repo-metrics-history/v1", "metrics history JSON must have a stable schema");
assertIncludes(historyRunner, "git worktree", "metrics history must inspect historical commits without moving the current checkout");
assertIncludes(historyRunner, "nodesrc\", \"run_repo_metrics.js", "metrics history must reuse the repo_metrics.ts wrapper as the metrics authority");
assertIncludes(historyRunner, "sampleCommits", "metrics history must sample across the complete reachable commit history instead of only loading recent commits");
assertIncludes(historyRunner, "total_commit_count", "metrics history must report the complete reachable commit count used for sampling context");
assertIncludes(historyRunner, "even-reachable-commits", "metrics history must declare the sampling strategy");
assertIncludes(historyRunner, "by_extension", "metrics history must preserve extension buckets for history presets");
assertIncludes(historyRunner, "by_content_kind", "metrics history must preserve content-kind buckets for history presets");
assertIncludes(selfhostRunner, "neplg2-selfhost-doctest/v1", "selfhost doctest checker JSON must have a stable schema");
assertIncludes(selfhostRunner, "selfhost_pipeline_check_loaded_root", "selfhost doctest checker must run the real selfhost compiler pipeline");
assertIncludes(selfhostRunner, "compiler_check", "selfhost doctest checker must label results as compiler checks instead of runtime doctests");
assertIncludes(selfhostRunner, "runtime_assertions: false", "selfhost doctest checker must not pretend to run Rust-style runtime assertions");
assertIncludes(selfhostCompleter, "selfhost-timeout:", "selfhost timeout completion must publish a result row instead of hiding missing JSON");
assertIncludes(selfhostCompleter, "summarize(payload.results)", "selfhost timeout completion must update report summary counts");

assertIncludes(metrics, "./metrics/repo_metrics.json", "metrics.html must load the generated metrics JSON by default");
assertIncludes(metrics, "./metrics/repo_metrics.csv", "metrics.html must link the generated metrics CSV");
assertIncludes(metrics, "./metrics/repo_metrics_history.json", "metrics.html must load the generated history JSON");
assertIncludes(metrics, "Repository History", "metrics.html must expose a repository history chart");
assertIncludes(metrics, "history-preset", "metrics.html must let users switch history graph presets");
assertIncludes(metrics, "By Content Kind", "metrics.html must expose a content-kind history graph preset");
assertIncludes(metrics, "By Extension", "metrics.html must expose an extension history graph preset");
assertIncludes(metrics, "topHistoryNames", "metrics.html must choose extension history series from the current sampled history");
assertIncludes(metrics, "renderHistoryChart", "metrics.html must render historical metrics as a chart");

assertIncludes(tests, "ansiToHtml", "tests.html must render ANSI diagnostic colors as HTML spans");
assertIncludes(tests, "parseDiagnosticError", "tests.html must parse compile_fail diagnostic mismatch details");
assertIncludes(tests, "resultTextFields", "tests.html must collect compiler output from result.error, compile_error, stderr, and related fields");
assertIncludes(tests, "hasAnsi", "tests.html must detect ANSI compiler output outside diagnostic mismatch reports");
assertIncludes(tests, "looksLikeCompilerOutput", "tests.html must identify ordinary compiler diagnostics before the raw fallback");
assertIncludes(tests, "renderTerminalOutput", "tests.html must route ordinary compiler output through ANSI-aware terminal rendering");
assertIncludes(tests, "Compiler output", "tests.html must split compiler output from mismatch metadata");
assertNotIncludes(tests, 'return `<pre>${escapeHtml(text || JSON.stringify(result, null, 2))}</pre>`;', "tests.html must not render ANSI compiler diagnostics through the raw escaped pre fallback");
assertIncludes(tests, "Selfhost compiler", "tests.html must show selfhost compiler check reports separately from Rust compiler reports");
assertIncludes(tests, "compiler check", "tests.html must not present selfhost checks as runtime doctests");
assertIncludes(tests, "loadReportSet", "tests.html must load current and last-completed report sets through one path");
assertIncludes(tests, "last-completed", "tests.html must display latest completed artifacts while the current run is pending");
assertIncludes(tests, "Test Hierarchy", "tests.html must expose hierarchical test grouping");
assertIncludes(tests, "buildHierarchy", "tests.html must aggregate reports into a hierarchy");
assertIncludes(tests, "rateView", "tests.html must show pass rates for reports and hierarchy nodes");
assertIncludes(tests, "reportJobState", "tests.html must prefer structured timeout report state over a nonfatal successful job pill");

for (const html of [testinfo]) {
    assertIncludes(html, "./tests/status.json", "CI result viewers must load the workflow status summary");
    assertIncludes(html, "./tests/tests-current.json", "CI result viewers must include wasi doctest JSON");
    assertIncludes(html, "./tests/nmd-tests.json", "CI result viewers must include nmd doctest JSON");
    assertIncludes(html, "./tests/tutorials-tests.json", "CI result viewers must include tutorials doctest JSON");
    assertIncludes(html, "./tests/examples-tests.json", "CI result viewers must include examples doctest JSON");
    assertIncludes(html, "./tests/stdlib-tests.json", "CI result viewers must include stdlib doctest JSON");
    assertIncludes(html, "./tests/tests-llvm.json", "CI result viewers must include LLVM doctest JSON");
    assertIncludes(html, "./tests/tests-dual-tests.json", "CI result viewers must include dual tests JSON");
    assertIncludes(html, "./tests/tests-dual-stdlib.json", "CI result viewers must include dual stdlib JSON");
    assertNotIncludes(html, 'value="./tests.json"', "CI result viewers must not default to stale tests.json");
    assertNotIncludes(html, 'value="./tests/cargo-test.log"', "CI result viewers must not depend on an unpublished cargo log");
}

for (const name of [
    "tests-current.json",
    "nmd-tests.json",
    "tutorials-tests.json",
    "examples-tests.json",
    "stdlib-tests.json",
    "tests-llvm.json",
    "selfhost-wasi-tests.json",
    "selfhost-nmd-tests.json",
    "selfhost-tutorials-tests.json",
    "selfhost-examples-tests.json",
    "selfhost-stdlib-tests.json",
    "selfhost-llvm-tests.json",
    "tests-dual-tests.json",
    "tests-dual-stdlib.json",
]) {
    assertIncludes(tests, name, `tests.html must know report artifact ${name}`);
}

console.log("Pages CI metrics contract regression passed");
