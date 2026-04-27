#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/std/streamio.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

const codeWithoutAbstractScanStub = code.replace(
    /trait\s+ScannerReadable:[\s\S]*?fn\s+scan\s+<\(StreamScanner\)\*>Self>\s+\(sc\):\s*#intrinsic\s+"unreachable"\s+<>\s+\(\)/,
    (match) => match.replace(/#intrinsic\s+"unreachable"\s+<>\s+\(\)/, 'ABSTRACT_SCANNER_READABLE_STUB'),
);

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(codeWithoutAbstractScanStub, pattern, `${relPath} must not use unsafe unwrap helpers in implementation code`);
}

assert.match(
    code,
    /fn\s+stream_scanner_load_header_result\s+<\(MemPtr<u8>,StreamScannerHeaderField\)->Result<i32,str>>\s+\(header,\s*field\):[\s\S]*match\s+load_i32\s+p:[\s\S]*Option::None:[\s\S]*Result<i32,str>::Err\s+"streamio\.stream_scanner_load_header failed"/,
    'stream scanner header loads must return Result instead of trapping',
);

assert.match(
    code,
    /fn\s+stream_scanner_store_header_result\s+<\(MemPtr<u8>,StreamScannerHeaderField,i32\)->Result<\(\),str>>\s+\(header,\s*field,\s*v\):[\s\S]*match\s+store_i32\s+p\s+v:[\s\S]*Result::Err\s+_e:[\s\S]*Result<\(\),str>::Err\s+"streamio\.stream_scanner_store_header failed"/,
    'stream scanner header stores must return Result instead of trapping',
);

assert.match(
    code,
    /fn\s+scanner_from_bytes\s+<\(ByteBuf\)\*>Result<StreamScanner,str>>\s+\(bytes\):[\s\S]*stream_scanner_store_header_result\s+header\s+StreamScannerHeaderField::BufPtr[\s\S]*Result::Err\s+_e:[\s\S]*io_bytebuf_free\s+bytes[\s\S]*stream_scanner_store_header_result\s+header\s+StreamScannerHeaderField::Len[\s\S]*stream_scanner_store_header_result\s+header\s+StreamScannerHeaderField::Pos/,
    'scanner_from_bytes must initialize scanner headers through Result-returning stores and clean up on failure',
);

assert.match(
    code,
    /fn\s+push_u8_impl\s+<\(StreamWriter,i32\)\*>StreamWriter>\s+\(w,\s*b\):[\s\S]*match\s+store_u8\s+mem_ptr_add\s+buf\s+write_len\s+b:[\s\S]*Result::Ok\s+_:[\s\S]*stream_writer_store_header\s+w_mem\s+StreamWriterHeaderField::WriteLen\s+add\s+write_len\s+1[\s\S]*Result::Err\s+_e:[\s\S]*\(\)[\s\S]*w1/,
    'push_u8_impl must only advance WriteLen after the byte store succeeds',
);

assert.match(
    code,
    /fn\s+append_str_impl\s+<\(StreamWriter,str\)\*>StreamWriter>\s+\(w,\s*s\):[\s\S]*while\s+and\s+eq\s+done\s+0\s+lt\s+i\s+n:[\s\S]*match\s+load_u8\s+mem_ptr_add\s+src\s+i:[\s\S]*Option::Some\s+ch:[\s\S]*set\s+ww\s+push_u8_impl\s+ww\s+ch/,
    'append_str_impl must stream through checked byte loads and push_u8_impl',
);

assert.match(
    code,
    /fn\s+append_bytebuf_impl\s+<\(StreamWriter,ByteBuf\)\*>StreamWriter>\s+\(w,\s*bytes\):[\s\S]*while\s+and\s+eq\s+done\s+0\s+lt\s+i\s+n:[\s\S]*match\s+load_u8\s+mem_ptr_add\s+src\s+i:[\s\S]*Option::Some\s+ch:[\s\S]*set\s+ww\s+push_u8_impl\s+ww\s+ch[\s\S]*io_bytebuf_free\s+bytes/,
    'append_bytebuf_impl must stream through checked byte loads, preserve ownership cleanup, and avoid unsafe unwrap',
);

assert.doesNotMatch(code, /store_u8\s+mem_ptr_add\s+buf\s+off/, 'numeric writer helpers must not bypass push_u8_impl with direct buffer stores');
assert.match(code, /fn\s+append_u64_digits_impl\s+<\(StreamWriter,i64\)\*>StreamWriter>[\s\S]*push_u8_impl/, 'numeric writer helpers must funnel digits through push_u8_impl');

console.log('stdlib streamio unsafe unwrap regression passed');
