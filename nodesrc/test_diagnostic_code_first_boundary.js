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

function diagnosticCodeEnumVariants(text, name) {
  const match = text.match(new RegExp(`\\npub enum ${name}\\s*\\{([\\s\\S]*?)\\n\\}`));
  assert(match, `nepl-core/src/diagnostic_codes.rs must define ${name}`);
  return [...match[1].matchAll(/^\s*([A-Z][A-Za-z0-9]*)(?:\(|,)/gm)].map((item) => item[1]);
}

function diagnosticCodeRegistryBlock(text) {
  const declaration = text.indexOf('pub const ALL_DIAGNOSTIC_CODES');
  assert(declaration >= 0, 'nepl-core/src/diagnostic_codes.rs must define ALL_DIAGNOSTIC_CODES');
  const open = text.indexOf('= &[', declaration);
  assert(open >= 0, 'ALL_DIAGNOSTIC_CODES must be initialized from a slice literal');

  let depth = 1;
  for (let index = open + '= &['.length; index < text.length; index += 1) {
    const char = text[index];
    if (char === '[') {
      depth += 1;
    } else if (char === ']') {
      depth -= 1;
      if (depth === 0) {
        return text.slice(open, index + 1);
      }
    }
  }
  assert.fail('could not find end of ALL_DIAGNOSTIC_CODES slice literal');
}

function countOccurrences(text, needle) {
  let count = 0;
  let index = text.indexOf(needle);
  while (index >= 0) {
    count += 1;
    index = text.indexOf(needle, index + needle.length);
  }
  return count;
}

function assertRegistryCoversEnum(registry, variantsByEnum, enumName, options = {}) {
  const ignored = new Set(options.ignore ?? []);
  for (const variant of variantsByEnum.get(enumName) ?? []) {
    if (ignored.has(variant)) {
      continue;
    }
    const reference = `${enumName}::${variant}`;
    assert.strictEqual(
      countOccurrences(registry, reference),
      1,
      `${reference} must appear exactly once in ALL_DIAGNOSTIC_CODES`,
    );
  }
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
assert.match(
  diagnosticCore,
  /pub\s+code:\s+DiagnosticCode\b/,
  'Diagnostic.code must be a mandatory DiagnosticCode, not an optional post-construction field',
);
assert.doesNotMatch(
  diagnosticCore,
  /Option\s*<\s*DiagnosticCode\s*>/,
  'Diagnostic must not permit code-less diagnostics through Option<DiagnosticCode>',
);

for (const { rel, text } of rustFiles) {
  const codeLessDiagnosticCall = /\bDiagnostic::(?:error|warning)\s*\(/g;
  for (let match = codeLessDiagnosticCall.exec(text); match; match = codeLessDiagnosticCall.exec(text)) {
    assert(
      false,
      `${rel} must not construct code-less diagnostics with ${match[0]}; use error_with_code/warning_with_code or a typed helper`,
    );
  }
}

const diagnosticCodes = rustFiles.find((item) => item.rel === 'nepl-core/src/diagnostic_codes.rs')?.text ?? '';
const diagnosticCodeEnumNames = [
  'LoaderDiagnosticCode',
  'LexerDiagnosticCode',
  'ParserDiagnosticCode',
  'ResolveDiagnosticCode',
  'TypeDiagnosticCode',
  'EffectDiagnosticCode',
  'ResourceMoveDiagnosticCode',
  'ResourceBorrowDiagnosticCode',
  'ResourceCellDiagnosticCode',
  'ResourceOwnerDiagnosticCode',
  'ResourceRawDiagnosticCode',
  'ResourceLowerDiagnosticCode',
  'BackendDiagnosticCode',
  'WasmDiagnosticCode',
  'LlvmDiagnosticCode',
];
const diagnosticCodeVariantsByEnum = new Map(
  diagnosticCodeEnumNames.map((name) => [name, diagnosticCodeEnumVariants(diagnosticCodes, name)]),
);
const diagnosticCodeRegistry = diagnosticCodeRegistryBlock(diagnosticCodes);

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

for (const enumName of diagnosticCodeEnumNames) {
  const ignore = enumName === 'BackendDiagnosticCode' ? ['Wasm', 'Llvm'] : [];
  assertRegistryCoversEnum(
    diagnosticCodeRegistry,
    diagnosticCodeVariantsByEnum,
    enumName,
    { ignore },
  );
}

const registryLeafReferences = diagnosticCodeRegistry.matchAll(/\b([A-Za-z][A-Za-z0-9]*DiagnosticCode)::([A-Z][A-Za-z0-9]*)\b/g);
const registryWrapperEnums = new Set(['DiagnosticCode', 'ResourceDiagnosticCode']);
for (const [, enumName, variant] of registryLeafReferences) {
  if (registryWrapperEnums.has(enumName)) {
    continue;
  }
  assert(
    diagnosticCodeVariantsByEnum.get(enumName)?.includes(variant),
    `ALL_DIAGNOSTIC_CODES references unknown diagnostic code variant ${enumName}::${variant}`,
  );
}

const web = rustFiles.find((item) => item.rel === 'nepl-web/src/lib.rs')?.text ?? '';
assert.match(
  web,
  /fn\s+loader_error\s*\([^)]*LoaderDiagnosticCode[\s\S]*Diagnostic::error_with_code\s*\(\s*DiagnosticCode::Loader\s*\(\s*code\s*\)/,
  'nepl-web loader diagnostics must use a LoaderDiagnosticCode helper',
);

console.log('diagnostic code-first boundary regression passed');
