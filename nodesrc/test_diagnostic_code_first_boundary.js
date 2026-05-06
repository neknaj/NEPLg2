#!/usr/bin/env node
// Diagnostic construction must stay code-first across Rust compiler boundaries.

const assert = require('assert');
const fs = require('fs');
const path = require('path');

const repo = path.resolve(__dirname, '..');

const rustSourceRoots = [
  'nepl-core/src',
  'nepl-language/src',
  'nepl-lsp/src',
  'nepl-web/src',
];

function toPosixPath(value) {
  return value.split(path.sep).join('/');
}

function walkRustFiles(rootRel) {
  const rootAbs = path.join(repo, rootRel);
  const entries = fs.readdirSync(rootAbs, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const childRel = `${rootRel}/${entry.name}`;
    if (entry.isDirectory()) {
      files.push(...walkRustFiles(childRel));
    } else if (entry.isFile() && entry.name.endsWith('.rs')) {
      files.push(toPosixPath(childRel));
    }
  }
  return files;
}

function diagnosticCodeImplBlocks(text) {
  const matches = [...text.matchAll(/\nimpl\s+([A-Za-z][A-Za-z0-9]*DiagnosticCode)\s*\{/g)];
  assert(matches.length > 0, 'nepl-core/src/diagnostic_codes.rs must define diagnostic code impls');

  return matches.map((match, index) => {
    const nextImpl = matches[index + 1]?.index ?? text.indexOf('\npub fn message', match.index);
    assert(nextImpl > match.index, `could not find end of ${match[1]} impl block`);
    return {
      name: match[1],
      text: text.slice(match.index, nextImpl),
    };
  });
}

const rustFiles = rustSourceRoots
  .flatMap(walkRustFiles)
  .sort()
  .map((rel) => ({ rel, text: fs.readFileSync(path.join(repo, rel), 'utf8') }));

for (const { rel, text } of rustFiles) {
  assert.doesNotMatch(
    text,
    /\.with_code\s*\(/,
    `${rel} must not attach diagnostic codes after construction`,
  );
  assert.doesNotMatch(
    text,
    /\bfn\s+with_code\b/,
    `${rel} must not define Diagnostic::with_code; diagnostics with codes must be constructed code-first`,
  );
}

const diagnosticCore = rustFiles.find((item) => item.rel === 'nepl-core/src/diagnostic.rs')?.text ?? '';
const diagnosticCoreTestModuleStart = diagnosticCore.indexOf('#[cfg(test)]');

for (const { rel, text } of rustFiles) {
  const codeLessDiagnosticCall = /\bDiagnostic::(?:error|warning)\s*\(/g;
  for (let match = codeLessDiagnosticCall.exec(text); match; match = codeLessDiagnosticCall.exec(text)) {
    const isDiagnosticCoreUnitTest =
      rel === 'nepl-core/src/diagnostic.rs'
      && diagnosticCoreTestModuleStart >= 0
      && match.index > diagnosticCoreTestModuleStart;
    assert(
      isDiagnosticCoreUnitTest,
      `${rel} must not construct code-less diagnostics with ${match[0]}; use error_with_code/warning_with_code or a typed helper`,
    );
  }
}

const diagnosticCodes = rustFiles.find((item) => item.rel === 'nepl-core/src/diagnostic_codes.rs')?.text ?? '';

for (const { name, text } of diagnosticCodeImplBlocks(diagnosticCodes)) {
  assert.match(
    text,
    /\b(?:pub\s+)?const\s+fn\s+as_str\s*\(\s*self\s*\)/,
    `${name} must expose an exhaustive as_str conversion`,
  );
  assert.match(
    text,
    /\b(?:pub\s+)?const\s+fn\s+message\s*\(\s*self\s*\)/,
    `${name} must expose an exhaustive message conversion`,
  );
  assert.doesNotMatch(
    text,
    /^\s*_\s*(?:if\b[^\n]*)?=>/m,
    `${name} must not use wildcard diagnostic code match arms; keep enum additions exhaustively checked`,
  );
}

const web = rustFiles.find((item) => item.rel === 'nepl-web/src/lib.rs')?.text ?? '';
assert.match(
  web,
  /fn\s+loader_error\s*\([^)]*LoaderDiagnosticCode[\s\S]*Diagnostic::error_with_code\s*\(\s*DiagnosticCode::Loader\s*\(\s*code\s*\)/,
  'nepl-web loader diagnostics must use a LoaderDiagnosticCode helper',
);

console.log('diagnostic code-first boundary regression passed');
