#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const reporterRel = 'stdlib/neplg2/cli/reporter.nepl';
const renderSingleRel = 'stdlib/neplg2/cli/reporter/render/single.nepl';
const renderCollectionRel = 'stdlib/neplg2/cli/reporter/render/collection.nepl';
const writeRel = 'stdlib/neplg2/cli/reporter/write.nepl';
const diagRel = 'stdlib/neplg2/core/infra/diag.nepl';
const reporterSrc = fs.readFileSync(path.join(repoRoot, reporterRel), 'utf8');
const renderSingleSrc = fs.readFileSync(path.join(repoRoot, renderSingleRel), 'utf8');
const renderCollectionSrc = fs.readFileSync(path.join(repoRoot, renderCollectionRel), 'utf8');
const writeSrc = fs.readFileSync(path.join(repoRoot, writeRel), 'utf8');
const diagSrc = fs.readFileSync(path.join(repoRoot, diagRel), 'utf8');

assert.match(
    reporterSrc,
    /pub\s+#import\s+"\.\/reporter\/render\/single"\s+as\s+\*/,
    'cli/reporter.nepl must re-export single diagnostic rendering',
);

assert.match(
    reporterSrc,
    /pub\s+#import\s+"\.\/reporter\/render\/collection"\s+as\s+\*/,
    'cli/reporter.nepl must re-export collection diagnostic rendering',
);

assert.match(
    reporterSrc,
    /pub\s+#import\s+"\.\/reporter\/write"\s+as\s+\*/,
    'cli/reporter.nepl must re-export stdio write boundary',
);

assert.doesNotMatch(
    reporterSrc,
    /\bfn\s+|#import\s+"std\/(?:stdio|streamio|io)"/,
    'cli/reporter.nepl must stay a facade without implementation bodies or direct stdio imports',
);

assert.match(
    renderSingleSrc,
    /#import\s+"neplg2\/core\/infra\/diag"\s+as\s+\*/,
    'single renderer must render the shared core diagnostic model',
);

assert.doesNotMatch(
    `${renderSingleSrc}\n${renderCollectionSrc}`,
    /#import\s+"std\/(?:stdio|streamio|io)"/,
    'render modules must not write stdout/stderr directly; reporter/write owns stdio',
);

assert.match(
    writeSrc,
    /\bstdio_write_stderr_str_result\b/,
    'human diagnostics must be written through the Result-returning stderr API',
);

assert.match(
    writeSrc,
    /\bstdio_write_str_result\b/,
    'machine diagnostic JSON must be written through the Result-returning stdout API',
);

assert.doesNotMatch(
    writeSrc,
    /\bstdio_write_stderr_str(?!_result)\b/,
    'reporter must not use the lossy stderr write facade',
);

assert.doesNotMatch(
    writeSrc,
    /\bstdio_write_str(?!_result)\b/,
    'reporter must not use the lossy stdout write facade',
);

assert.doesNotMatch(
    diagSrc,
    /#import\s+"std\/(?:stdio|streamio|io)"/,
    'core diagnostics must remain independent from stdio; only cli/reporter may write them',
);

console.log('selfhost CLI reporter boundary regression passed');
