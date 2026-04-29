#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const driverRel = 'stdlib/neplg2/cli/driver.nepl';
const driverSrc = fs.readFileSync(path.join(repoRoot, driverRel), 'utf8');

for (const importPath of [
    'neplg2/cli/args',
    'neplg2/cli/reporter',
    'neplg2/core/module/loader',
    'neplg2/core/pipeline',
]) {
    assert.match(
        driverSrc,
        new RegExp(`#import\\s+"${importPath.replaceAll('/', '\\/')}"\\s+as\\s+\\*`),
        `cli/driver.nepl must import ${importPath} as an explicit boundary`,
    );
}

assert.match(
    driverSrc,
    /\bpub\s+struct\s+SelfhostCliDriverResult\b/,
    'driver must expose a result value that owns exit code and diagnostics',
);

assert.match(
    driverSrc,
    /\bselfhost_cli_options_to_compile_options\b/,
    'driver must use the CLI-to-core option conversion boundary',
);

assert.match(
    driverSrc,
    /\bselfhost_pipeline_load_root\b/,
    'driver must enter core through the pipeline boundary',
);

assert.match(
    driverSrc,
    /\bselfhost_cli_write_json_diagnostics_stdout\b/,
    'driver JSON output must go through cli/reporter',
);

assert.doesNotMatch(
    driverSrc,
    /#import\s+"std\/(?:fs|stdio|streamio|io)"/,
    'driver must not import filesystem or stdio directly',
);

console.log('selfhost CLI driver boundary regression passed');
