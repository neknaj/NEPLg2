const fs = require('fs');
const path = require('path');
const assert = require('assert');
const { readDiagSource } = require('./selfhost_diag_sources');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const root = path.resolve(__dirname, '..');

function read(rel) {
  return fs.readFileSync(path.join(root, rel), 'utf8');
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function declarationBody(src, declarationPattern, label) {
  const match = declarationPattern.exec(src);
  assert(match, `${label} declaration must exist`);
  const bodyStart = src.indexOf('\n', match.index) + 1;
  const rest = src.slice(bodyStart);
  const nextDeclaration = rest.search(/^(?:#import|\/\/:|(?:pub\s+)?(?:enum|impl|struct|fn)\s)/m);
  return nextDeclaration < 0 ? rest : rest.slice(0, nextDeclaration);
}

function enumVariants(enumName) {
  const body = declarationBody(
    diag,
    new RegExp(`^(?:pub\\s+)?enum\\s+${escapeRegExp(enumName)}:\\s*$`, 'm'),
    enumName,
  );
  return body
    .split(/\r?\n/)
    .map((line) => line.match(/^\s{4}([A-Za-z][A-Za-z0-9_]*)(?:\s|$)/)?.[1])
    .filter(Boolean);
}

function functionBody(fnName) {
  return declarationBody(
    diag,
    new RegExp(`^(?:pub\\s+)?fn\\s+${escapeRegExp(fnName)}\\b`, 'm'),
    fnName,
  );
}

function assertLeafCodeMapping({ enumName, functionName, prefix }) {
  const variants = enumVariants(enumName);
  assert(variants.length > 0, `${enumName} must declare diagnostic variants`);
  const body = functionBody(functionName);
  assert.match(body, /^\s{4}match\s+code:/m, `${functionName} must match on code`);
  assert.doesNotMatch(body, /^\s{8}_:/m, `${functionName} must not use a wildcard arm`);
  for (const variant of variants) {
    const references = body.match(
      new RegExp(`${escapeRegExp(enumName)}::${escapeRegExp(variant)}\\s*:`, 'g'),
    ) ?? [];
    assert.equal(
      references.length,
      1,
      `${functionName} must map ${enumName}::${variant} exactly once`,
    );
  }
  const stringCodes = [...body.matchAll(/"([^"]+)"/g)].map((m) => m[1]);
  assert(stringCodes.length > 0, `${functionName} must return stable string codes`);
  for (const code of stringCodes) {
    assert(
      code.startsWith(prefix),
      `${functionName} returned ${code}; expected ${prefix} prefix`,
    );
  }
}

const diag = legacyTypeSyntaxView(readDiagSource(root));
const reporter = [
  read('stdlib/neplg2/cli/reporter/render/single.nepl'),
  read('stdlib/neplg2/cli/reporter/render/collection.nepl'),
].map(legacyTypeSyntaxView).join('\n');
const lexer = legacyTypeSyntaxView(read('stdlib/neplg2/core/syntax/lexer.nepl'));
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
  /(?:pub\s+)?enum\s+SelfhostDiagnosticCode:[\s\S]*Loader\s+<SelfhostLoaderDiagnosticCode>[\s\S]*Lexer\s+<SelfhostLexerDiagnosticCode>[\s\S]*Parser\s+<SelfhostParserDiagnosticCode>[\s\S]*Resolve\s+<SelfhostResolveDiagnosticCode>[\s\S]*Checker\s+<SelfhostCheckerDiagnosticCode>[\s\S]*Cli\s+<SelfhostCliDiagnosticCode>/,
  'SelfhostDiagnosticCode must be a hierarchical enum',
);
assert.deepEqual(
  enumVariants('SelfhostDiagnosticCode'),
  ['Loader', 'Lexer', 'Parser', 'Resolve', 'Checker', 'Cli'],
  'SelfhostDiagnosticCode category additions must update the self-host diagnostic policy',
);
assert.match(diag, /code\s+<SelfhostDiagnosticCode>/, 'SelfhostDiagnostic.code must be typed');
assert.doesNotMatch(diag, /code\s+<str>/, 'SelfhostDiagnostic.code must not be a raw string');
assert.doesNotMatch(
  diag,
  /(?:pub\s+)?fn\s+selfhost_diag_(?:new|info|warning|error)\s+<\([^)]*str\s*,\s*str/,
  'selfhost diagnostic constructors must not accept raw string codes',
);
assert.match(
  diag,
  /(?:pub\s+)?fn\s+selfhost_diag_code_name\s+<\(SelfhostDiagnosticCode\)->str>[\s\S]*SelfhostDiagnosticCode::Loader[\s\S]*SelfhostDiagnosticCode::Lexer[\s\S]*SelfhostDiagnosticCode::Parser[\s\S]*SelfhostDiagnosticCode::Resolve[\s\S]*SelfhostDiagnosticCode::Checker[\s\S]*SelfhostDiagnosticCode::Cli/,
  'SelfhostDiagnosticCode string conversion must cover every category',
);
assert.doesNotMatch(
  diag.match(/(?:pub\s+)?fn\s+selfhost_diag_code_name[\s\S]*?(?=\n\/\/: selfhost_diag_label_new)/)?.[0] ?? '',
  /^\s*_:/m,
  'SelfhostDiagnosticCode string conversion must not use a wildcard arm',
);
[
  {
    enumName: 'SelfhostLoaderDiagnosticCode',
    functionName: 'selfhost_loader_diag_code_name',
    prefix: 'loader.',
  },
  {
    enumName: 'SelfhostLexerDiagnosticCode',
    functionName: 'selfhost_lexer_diag_code_name',
    prefix: 'lexer.',
  },
  {
    enumName: 'SelfhostParserDiagnosticCode',
    functionName: 'selfhost_parser_diag_code_name',
    prefix: 'parser.',
  },
  {
    enumName: 'SelfhostResolveDiagnosticCode',
    functionName: 'selfhost_resolve_diag_code_name',
    prefix: 'resolve.',
  },
  {
    enumName: 'SelfhostCheckerDiagnosticCode',
    functionName: 'selfhost_checker_diag_code_name',
    prefix: 'checker.',
  },
  {
    enumName: 'SelfhostCliDiagnosticCode',
    functionName: 'selfhost_cli_diag_code_name',
    prefix: 'cli.',
  },
].forEach(assertLeafCodeMapping);
assert.match(
  reporter,
  /selfhost_diag_code_name\s+\*field::get_ref\s+diag\s+"code"/,
  'reporter must render diagnostic codes through selfhost_diag_code_name',
);
assert.doesNotMatch(lexer, /\b(?:pub\s+)?enum\s+LexErrorCode:/, 'lexer must use the shared selfhost lexer diagnostic enum');
assert.doesNotMatch(lexer, /\blex_error_code_name\b/, 'lexer must not stringify codes before SelfhostDiagnostic');

for (const rel of neplg2Files) {
  const src = legacyTypeSyntaxView(read(rel));
  assert.doesNotMatch(
    src,
    /selfhost_diag_(?:new|info|warning|error)\s+"[^"]+"/,
    `${rel} must pass typed SelfhostDiagnosticCode values`,
  );
}

console.log('selfhost diagnostic code enum regression passed');
