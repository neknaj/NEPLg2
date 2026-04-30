#!/usr/bin/env node
// Editor update payload must preserve compiler diagnostic stable codes.

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const { pathToFileURL } = require('url');

async function main() {
  const repo = path.resolve(__dirname, '..');
  const build = process.platform === 'win32'
    ? spawnSync('cmd.exe', ['/d', '/s', '/c', 'npm --prefix web run build:ts'], {
        cwd: repo,
        stdio: 'inherit',
      })
    : spawnSync('npm', ['--prefix', 'web', 'run', 'build:ts'], {
        cwd: repo,
        stdio: 'inherit',
      });
  if (build.error) {
    throw build.error;
  }
  if (build.status !== 0) {
    throw new Error(`npm --prefix web run build:ts failed with status ${build.status}`);
  }

  const bridgePath = path.join(repo, 'web', 'dist_ts', 'editor-core', 'language-analysis.js');
  if (!fs.existsSync(bridgePath)) {
    throw new Error(`language analysis bridge not found: ${bridgePath}\nrun 'npm --prefix web run build:ts' first.`);
  }
  const bridge = await import(pathToFileURL(bridgePath).href);

  const text = 'fn main <()->i32> ():\n    missing_symbol\n';
  const snapshot = {
    semantics: {
      diagnostics: [{
        severity: 'error',
        code: 'resolve.identifier.undefined',
        code_message: 'undefined identifier',
        message: 'undefined identifier',
        span: {
          start: text.indexOf('missing_symbol'),
          end: text.indexOf('missing_symbol') + 'missing_symbol'.length,
          start_line: 1,
          start_col: 4,
          end_line: 1,
          end_col: 18,
        },
      }],
    },
  };

  const payload = bridge.buildEditorUpdatePayloadFromAnalysis(text, snapshot);
  assert.equal(payload.diagnostics.length, 1);
  assert.equal(payload.diagnostics[0].code, 'resolve.identifier.undefined');
  assert.equal(payload.diagnostics[0].codeMessage, 'undefined identifier');

  const remapped = bridge.remapEditorUpdatePayloadForTextChange(
    text,
    `// header\n${text}`,
    payload,
  );
  assert.equal(remapped.diagnostics.length, 1);
  assert.equal(remapped.diagnostics[0].code, 'resolve.identifier.undefined');
  assert.equal(remapped.diagnostics[0].codeMessage, 'undefined identifier');

  console.log('editor diagnostic code contract regression passed');
}

main().catch((error) => {
  console.error(error && error.stack ? error.stack : String(error));
  process.exit(1);
});
