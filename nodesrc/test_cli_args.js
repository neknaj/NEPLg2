#!/usr/bin/env node

const assert = require('node:assert/strict');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

const cliPath = path.join(__dirname, 'cli.js');

function runCli(args) {
    return spawnSync(process.execPath, [cliPath, ...args], {
        encoding: 'utf8',
        env: {
            ...process.env,
            NEPL_DISCORD_WEBHOOK_URL: '',
            DISCORD_WEBHOOK_URL: '',
        },
    });
}

{
    const result = runCli(['--playgroud-editor-tests']);
    assert.equal(result.status, 2);
    assert.match(result.stderr, /unknown argument: --playgroud-editor-tests/);
}

{
    const result = runCli(['-i']);
    assert.equal(result.status, 2);
    assert.match(result.stderr, /-i requires a value/);
}

{
    const result = runCli(['--help']);
    assert.equal(result.status, 0);
    assert.match(result.stdout, /Usage: node nodesrc\/cli\.js/);
}

console.log('cli argument tests passed');
