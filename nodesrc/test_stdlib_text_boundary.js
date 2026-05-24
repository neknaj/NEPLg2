#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { implementationLineCount } = require('./source_policy/stdlib_builder_owner');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
}

function codeOnly(src) {
    return legacyTypeSyntaxView(src);
}

const rootRelPath = 'stdlib/std/text.nepl';
const validateRelPath = 'stdlib/std/text/validate.nepl';
const decodeRelPath = 'stdlib/std/text/decode.nepl';
const convertRelPath = 'stdlib/std/text/convert.nepl';

const rootCode = codeOnly(read(rootRelPath));
const validateCode = codeOnly(read(validateRelPath));
const decodeCode = codeOnly(read(decodeRelPath));
const convertCode = codeOnly(read(convertRelPath));

assert.doesNotMatch(rootCode, /pub\s+#import\s+"\.\/text\/validate"\s+as\s+@merge/, 'std/text facade must not re-export raw validation helpers');
assert.doesNotMatch(rootCode, /pub\s+#import\s+"\.\/text\/decode"\s+as\s+@merge/, 'std/text facade must not re-export raw decode helpers');
assert.match(rootCode, /pub\s+#import\s+"\.\/text\/convert"\s+as\s+@merge/, 'std/text facade must re-export checked ByteBuf-to-str conversion');
assert.match(read(rootRelPath), /raw `MemPtr` helper/, 'std/text root must document explicit raw helper imports');

assert.doesNotMatch(rootCode, /\bfn\s+/, 'std/text facade must not keep implementation functions');
assert.doesNotMatch(rootCode, /\benum\s+/, 'std/text facade must not keep UTF-8 classifier enums');

for (const helper of [
    'TextUtf8LeadKind',
    'text_utf8_in_range',
    'text_utf8_is_continuation',
    'text_utf8_lead_kind',
    'text_utf8_byte_at_checked',
    'text_utf8_validate_mem',
]) {
    assert.match(validateCode, new RegExp(`\\b${helper}\\b`), `${helper} must stay in std/text/validate`);
    assert.doesNotMatch(decodeCode, new RegExp(`\\b(?:enum|fn)\\s+${helper}\\b`), `${helper} must not be defined in std/text/decode`);
    assert.doesNotMatch(convertCode, new RegExp(`\\b(?:enum|fn)\\s+${helper}\\b`), `${helper} must not be defined in std/text/convert`);
}

assert.doesNotMatch(
    validateCode,
    /\bpub\s+fn\s+text_utf8_validate_(?:two|three|four)\b/,
    'std/text sequence validators must stay private implementation details',
);
assert.doesNotMatch(
    validateCode,
    /\bfn\s+text_utf8_byte_at\b/,
    'std/text must not expose the old unchecked byte reader name',
);
assert.match(
    validateCode,
    /\bfn\s+text_utf8_byte_at_checked\b[\s\S]*or\s+lt\s+idx\s+0\s+le\s+byte_len\s+idx[\s\S]*let\s+ptr\s+<MemPtr<u8>>\s+mem_ptr_add\s+data\s+idx[\s\S]*match\s+load_u8\s+ptr:/,
    'std/text checked byte reader must carry byte_len and keep mem_ptr_add as call-head evidence',
);
assert.match(
    decodeCode,
    /\btext_utf8_byte_at_checked\s+data\s+byte_len\s+i\b/,
    'std/text decode must use the byte_len-carrying checked reader for the leading byte',
);

for (const helper of [
    'text_is_valid_scalar',
    'text_utf8_decode_next',
    'text_utf8_encode_char',
]) {
    assert.match(decodeCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must stay in std/text/decode`);
    assert.doesNotMatch(validateCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must not be defined in std/text/validate`);
    assert.doesNotMatch(convertCode, new RegExp(`\\bfn\\s+${helper}\\b`), `${helper} must not be defined in std/text/convert`);
}

assert.match(
    convertCode,
    /\bfn\s+text_bytebuf_to_utf8_str_result\b/,
    'text_bytebuf_to_utf8_str_result must stay in std/text/convert',
);
assert.doesNotMatch(
    validateCode,
    /\bfn\s+text_bytebuf_to_utf8_str_result\b/,
    'std/text/validate must not own ByteBuf-to-str conversion',
);
assert.doesNotMatch(
    decodeCode,
    /\bfn\s+text_bytebuf_to_utf8_str_result\b/,
    'std/text/decode must not own ByteBuf-to-str conversion',
);

assert.match(
    decodeCode,
    /#import\s+"\.\/validate"\s+as\s+\*/,
    'std/text/decode must reuse UTF-8 validation helpers from std/text/validate',
);
assert.match(
    convertCode,
    /#import\s+"\.\/validate"\s+as\s+\*/,
    'std/text/convert must validate via std/text/validate before converting to str',
);
assert.match(
    convertCode,
    /text_utf8_validate_mem[\s\S]*io_bytebuf_to_str_result/,
    'std/text/convert must validate bytes before io_bytebuf_to_str_result',
);

for (const [relPath, src, limit] of [
    [rootRelPath, read(rootRelPath), 80],
    [validateRelPath, read(validateRelPath), 240],
    [decodeRelPath, read(decodeRelPath), 200],
    [convertRelPath, read(convertRelPath), 100],
]) {
    const lines = implementationLineCount(src);
    assert(lines <= limit, `${relPath} has ${lines} lines; split limit is ${limit}`);
}

console.log('stdlib text boundary regression passed');
