#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const reporterRel = 'stdlib/neplg2/cli/reporter.nepl';
const diagRel = 'stdlib/neplg2/core/infra/diag.nepl';
const reporterSrc = fs.readFileSync(path.join(repoRoot, reporterRel), 'utf8');
const diagSrc = fs.readFileSync(path.join(repoRoot, diagRel), 'utf8');

assert.match(
    reporterSrc,
    /#import\s+"neplg2\/core\/infra\/diag"\s+as\s+\*/,
    'cli/reporter.nepl must render the shared core diagnostic model',
);

assert.match(
    reporterSrc,
    /\bstdio_write_stderr_str_result\b/,
    'human diagnostics must be written through the Result-returning stderr API',
);

assert.match(
    reporterSrc,
    /\bstdio_write_str_result\b/,
    'machine diagnostic JSON must be written through the Result-returning stdout API',
);

assert.doesNotMatch(
    reporterSrc,
    /\bstdio_write_stderr_str(?!_result)\b/,
    'reporter must not use the lossy stderr write facade',
);

assert.doesNotMatch(
    reporterSrc,
    /\bstdio_write_str(?!_result)\b/,
    'reporter must not use the lossy stdout write facade',
);

assert.doesNotMatch(
    diagSrc,
    /#import\s+"std\/(?:stdio|streamio|io)"/,
    'core diagnostics must remain independent from stdio; only cli/reporter may write them',
);

console.log('selfhost CLI reporter boundary regression passed');
