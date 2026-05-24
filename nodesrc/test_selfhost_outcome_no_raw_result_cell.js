#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/neplg2/core/infra/outcome.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

const code = legacyTypeSyntaxView(src);

assert.match(
    code,
    /struct\s+SelfhostOutcome<\.T,\s*\.E>:[\s\S]*\n\s+result\s+<Result<\.T,\s*\.E>>/,
    'SelfhostOutcome must own Result<T,E> directly instead of storing it in a raw pointer cell',
);

assert.doesNotMatch(
    code,
    /MemPtr<Result<\.T,\s*\.E>>/,
    'SelfhostOutcome must not store Result<T,E> in MemPtr<Result<T,E>>',
);

assert.doesNotMatch(
    code,
    /\b(?:load|store)<Result<\.T,\s*\.E>>/,
    'SelfhostOutcome must not raw-load or raw-store its stage Result<T,E>',
);

assert.doesNotMatch(
    code,
    /selfhost_outcome_dealloc_result_ptr/,
    'SelfhostOutcome must not need a result pointer deallocator',
);

console.log('selfhost outcome raw result cell regression passed');
