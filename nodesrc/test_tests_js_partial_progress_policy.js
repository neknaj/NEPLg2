#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'nodesrc/tests.js';
const code = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

assert.match(code, /function\s+writeJsonFileAtomic\s*\(outAbs,\s*obj\)/, 'tests.js must write JSON through an atomic temp-file rename helper');
assert.doesNotMatch(code, /fs\.writeFileSync\(outAbs\s*,/, 'tests.js must not write output JSON directly to the final path');
assert.match(code, /top_issues:\s*pickTopIssues\(partialResults,\s*5\)/, 'partial JSON must include actionable top_issues');
assert.match(code, /const\s+recordWasmProgress\s*=\s*\(r\)\s*=>\s*\{[\s\S]*applyDoctestExpectations\(\{\s*\.\.\.r\s*\},\s*c,\s*\{\s*assertIo\s*\}\)[\s\S]*recordProgress/, 'wasm partial progress must record expectation-checked results');
assert.match(code, /const\s+shouldFlushFailure\s*=\s*result\?\.status\s*===\s*'fail'\s*\|\|\s*result\?\.status\s*===\s*'error'[\s\S]*shouldFlushFailure\s*\|\|/, 'failed and errored results must be flushed to partial JSON immediately');
assert.match(code, /effectiveProgressFlushEvery\(progressFlushEvery\(\),\s*expectedProgressResults\)/, 'small focused suites must reduce partial flush interval from the default');
assert.match(code, /msg\.kind\s*===\s*'progress'[\s\S]*active\.lastProgress\s*=\s*msg\.progress/, 'wasm worker phase progress must be retained for timeout diagnostics');
assert.match(code, /phase:\s*lastPhase\s*\|\|\s*'timeout'[\s\S]*last_phase:\s*lastPhase/, 'timeout result must expose the last known worker phase');
assert.match(code, /const\s+retiringWorkers\s*=\s*new\s+Set\(\)/, 'intentional worker termination must track retiring workers');
assert.match(code, /retiringWorkers\.add\(w\)/, 'timeout termination must mark the worker as retiring');
assert.match(code, /retiringWorkers\.has\(w\)/, 'worker exit handling must ignore intentionally retiring workers');
const wasmProgressCalls = code.match(/runAll\(wasmCases,\s*jobs,\s*distHint,\s*\{\s*onResult:\s*recordWasmProgress\s*\}\)/g) || [];
assert.equal(wasmProgressCalls.length, 2, 'wasm-only and all-runner wasm paths must both flush progress through expectation-checked progress');
assert.match(code, /id:\s*'nodesrc\/tests\/internal-error'[\s\S]*writePartialOutput\('error'\)/, 'internal harness errors must be recorded into the JSON output');

console.log('tests.js partial progress policy passed');
