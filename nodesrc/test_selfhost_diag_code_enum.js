const fs = require('fs');
const path = require('path');
const assert = require('assert');

const root = path.resolve(__dirname, '..');

function read(rel) {
  return fs.readFileSync(path.join(root, rel), 'utf8');
}

const diag = read('stdlib/neplg2/core/infra/diag.nepl');
const reporter = read('stdlib/neplg2/cli/reporter.nepl');
const lexer = read('stdlib/neplg2/core/syntax/lexer.nepl');
const neplg2Files = [];

function walk(dir) {
  for (const entry of fs.readdirSync(path.join(root, dir), { withFileTypes: true })) {
    const rel = path.join(dir, entry.name).replace(/\\/g, '/');
    if (entry.isDirectory()) {
      walk(rel);
    } else if (entry.isFile() && rel.endsWith('.nepl')) {
      neplg2Files.push(rel);
    }
  }
}

walk('stdlib/neplg2');

assert.match(
  diag,
  /enum\s+SelfhostDiagnosticCode:[\s\S]*Loader\s+<SelfhostLoaderDiagnosticCode>[\s\S]*Lexer\s+<SelfhostLexerDiagnosticCode>[\s\S]*Parser\s+<SelfhostParserDiagnosticCode>[\s\S]*Resolve\s+<SelfhostResolveDiagnosticCode>[\s\S]*Cli\s+<SelfhostCliDiagnosticCode>/,
  'SelfhostDiagnosticCode must be a hierarchical enum',
);
assert.match(diag, /code\s+<SelfhostDiagnosticCode>/, 'SelfhostDiagnostic.code must be typed');
assert.doesNotMatch(diag, /code\s+<str>/, 'SelfhostDiagnostic.code must not be a raw string');
assert.doesNotMatch(
  diag,
  /fn\s+selfhost_diag_(?:new|info|warning|error)\s+<\([^)]*str\s*,\s*str/,
  'selfhost diagnostic constructors must not accept raw string codes',
);
assert.match(
  diag,
  /fn\s+selfhost_diag_code_name\s+<\(SelfhostDiagnosticCode\)->str>[\s\S]*SelfhostDiagnosticCode::Loader[\s\S]*SelfhostDiagnosticCode::Lexer[\s\S]*SelfhostDiagnosticCode::Parser[\s\S]*SelfhostDiagnosticCode::Resolve[\s\S]*SelfhostDiagnosticCode::Cli/,
  'SelfhostDiagnosticCode string conversion must cover every category',
);
assert.doesNotMatch(
  diag.match(/fn\s+selfhost_diag_code_name[\s\S]*?(?=\n\/\/: selfhost_diag_label_new)/)?.[0] ?? '',
  /^\s*_:/m,
  'SelfhostDiagnosticCode string conversion must not use a wildcard arm',
);
assert.match(
  reporter,
  /selfhost_diag_code_name\s+field::get\s+diag\s+"code"/,
  'reporter must render diagnostic codes through selfhost_diag_code_name',
);
assert.doesNotMatch(lexer, /\benum\s+LexErrorCode:/, 'lexer must use the shared selfhost lexer diagnostic enum');
assert.doesNotMatch(lexer, /\blex_error_code_name\b/, 'lexer must not stringify codes before SelfhostDiagnostic');

for (const rel of neplg2Files) {
  const src = read(rel);
  assert.doesNotMatch(
    src,
    /selfhost_diag_(?:new|info|warning|error)\s+"[^"]+"/,
    `${rel} must pass typed SelfhostDiagnosticCode values`,
  );
}

console.log('selfhost diagnostic code enum regression passed');
