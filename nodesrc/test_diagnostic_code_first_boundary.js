#!/usr/bin/env node
// Diagnostic construction must stay code-first across Rust compiler boundaries.

const assert = require('assert');
const fs = require('fs');
const path = require('path');

const repo = path.resolve(__dirname, '..');

const rustFiles = [
  'nepl-core/src/diagnostic.rs',
  'nepl-language/src/lib.rs',
  'nepl-lsp/src/main.rs',
  'nepl-web/src/lib.rs',
].map((rel) => ({ rel, text: fs.readFileSync(path.join(repo, rel), 'utf8') }));

for (const { rel, text } of rustFiles) {
  assert.doesNotMatch(
    text,
    /\.with_code\s*\(/,
    `${rel} must not attach diagnostic codes after construction`,
  );
}

const diagnosticCore = rustFiles.find((item) => item.rel === 'nepl-core/src/diagnostic.rs')?.text ?? '';
assert.doesNotMatch(
  diagnosticCore,
  /\bfn\s+with_code\b/,
  'Diagnostic::with_code API must not exist; diagnostics with codes must be constructed code-first',
);

const web = rustFiles.find((item) => item.rel === 'nepl-web/src/lib.rs')?.text ?? '';
assert.match(
  web,
  /fn\s+loader_error\s*\([^)]*LoaderDiagnosticCode[\s\S]*Diagnostic::error_with_code\s*\(\s*DiagnosticCode::Loader\s*\(\s*code\s*\)/,
  'nepl-web loader diagnostics must use a LoaderDiagnosticCode helper',
);

console.log('diagnostic code-first boundary regression passed');
