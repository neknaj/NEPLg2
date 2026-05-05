#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/std/streamio.nepl';
const writerRelPath = 'stdlib/std/streamio/writer.nepl';
const bytesRelPath = 'stdlib/std/streamio/bytes.nepl';
const scannerRelPath = 'stdlib/std/streamio/scanner.nepl';
const scannerCursorRelPath = 'stdlib/std/streamio/scanner/cursor.nepl';
const scannerNumberRelPath = 'stdlib/std/streamio/scanner/number.nepl';
const scannerStateRelPath = 'stdlib/std/streamio/scanner/state.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
const writerSrc = fs.readFileSync(path.join(repoRoot, writerRelPath), 'utf8');
const bytesSrc = fs.readFileSync(path.join(repoRoot, bytesRelPath), 'utf8');
const scannerSrc = fs.readFileSync(path.join(repoRoot, scannerRelPath), 'utf8');
const scannerCursorSrc = fs.readFileSync(path.join(repoRoot, scannerCursorRelPath), 'utf8');
const scannerNumberSrc = fs.readFileSync(path.join(repoRoot, scannerNumberRelPath), 'utf8');
const scannerStateSrc = fs.readFileSync(path.join(repoRoot, scannerStateRelPath), 'utf8');

const facadeCode = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const writerCode = writerSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const bytesCode = bytesSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const scannerCode = scannerSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const scannerStateCode = scannerStateSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const scannerCursorCode = scannerCursorSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const scannerNumberCode = scannerNumberSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const code = `${facadeCode}\n${writerCode}\n${bytesCode}\n${scannerCode}\n${scannerCursorCode}\n${scannerNumberCode}\n${scannerStateCode}`;

assert.match(
    facadeCode,
    /pub\s+#import\s+"\.\/streamio\/writer"\s+as\s+\*/,
    `${relPath} must re-export the writer module`,
);
assert.match(
    facadeCode,
    /pub\s+#import\s+"\.\/streamio\/bytes"\s+as\s+\*/,
    `${relPath} must re-export the bytes module`,
);
assert.match(
    facadeCode,
    /pub\s+#import\s+"\.\/streamio\/scanner"\s+as\s+\*/,
    `${relPath} must re-export the scanner module`,
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
    assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap helpers in implementation code`);
}

assert.doesNotMatch(code, /trait\s+ScannerReadable/, 'StreamScanner read must use concrete overloads, not an unreachable generic trait default');
assert.match(code, /fn\s+read\s+<\(StreamScanner\)\*>str>\s+\(sc\):[\s\S]*scan_token_impl\s+sc/, 'StreamScanner must keep a str read overload');
assert.match(code, /fn\s+read\s+<\(StreamScanner\)\*>i32>\s+\(sc\):[\s\S]*scan_i32_impl\s+sc/, 'StreamScanner must keep an i32 read overload');
assert.match(code, /fn\s+read\s+<\(StreamScanner\)\*>f64>\s+\(sc\):[\s\S]*scan_f64_impl\s+sc/, 'StreamScanner must keep an f64 read overload');

assert.match(
    code,
    /fn\s+stream_scanner_load_header_result\s+<\(MemPtr<u8>,StreamScannerHeaderField\)->Result<i32,str>>\s+\(header,\s*field\):[\s\S]*le\s+raw\s+0[\s\S]*Result<i32,str>::Err\s+"streamio\.stream_scanner_load_header failed"[\s\S]*Result<i32,str>::Ok\s+load_i32\s+add\s+raw\s+stream_scanner_header_off\s+field/,
    'stream scanner header loads must return Result through the scanner header boundary instead of trapping',
);

assert.match(
    code,
    /fn\s+stream_scanner_store_header_result\s+<\(MemPtr<u8>,StreamScannerHeaderField,i32\)->Result<\(\),str>>\s+\(header,\s*field,\s*v\):[\s\S]*le\s+raw\s+0[\s\S]*Result<\(\),str>::Err\s+"streamio\.stream_scanner_store_header failed"[\s\S]*store_i32\s+add\s+raw\s+stream_scanner_header_off\s+field\s+v[\s\S]*Result<\(\),str>::Ok\s+\(\)/,
    'stream scanner header stores must return Result through the scanner header boundary instead of trapping',
);

assert.match(
    code,
    /fn\s+scanner_from_bytes\s+<\(ByteBuf\)\*>Result<StreamScanner,str>>\s+\(bytes\):[\s\S]*stream_scanner_store_header_result\s+header\s+StreamScannerHeaderField::BufPtr[\s\S]*Result::Err\s+_e:[\s\S]*io_bytebuf_free\s+bytes[\s\S]*stream_scanner_store_header_result\s+header\s+StreamScannerHeaderField::Len[\s\S]*stream_scanner_store_header_result\s+header\s+StreamScannerHeaderField::Pos/,
    'scanner_from_bytes must initialize scanner headers through Result-returning stores and clean up on failure',
);

assert.match(
    code,
    /fn\s+push_u8_impl\s+<\(StreamWriter,i32\)\*>StreamWriter>\s+\(w,\s*b\):[\s\S]*match\s+store_u8\s+mem_ptr_add\s+\*get_ref\s+&w1\s+"buf"\s+write_len\s+b:[\s\S]*Result::Ok\s+_:[\s\S]*StreamWriter\s+get\s+w1\s+"buf"\s+cap\s+add\s+write_len\s+1\s+target\s+@stream_writer_noncopy_marker[\s\S]*Result::Err\s+_e:[\s\S]*w1/,
    'push_u8_impl must only advance WriteLen after the byte store succeeds',
);

assert.match(
    code,
    /fn\s+append_str_impl\s+<\(StreamWriter,str\)\*>StreamWriter>\s+\(w,\s*s\):[\s\S]*while\s+lt\s+i\s+n:[\s\S]*string_byte_at_unchecked\s+s\s+i[\s\S]*set\s+ww\s+push_u8_impl\s+ww\s+ch/,
    'append_str_impl must stream through the alloc/string byte boundary and push_u8_impl',
);

assert.match(
    code,
    /fn\s+append_bytebuf_impl\s+<\(StreamWriter,ByteBuf\)\*>StreamWriter>\s+\(w,\s*bytes\):[\s\S]*while\s+and\s+eq\s+done\s+0\s+lt\s+i\s+n:[\s\S]*match\s+stream_writer_bytebuf_byte_at\s+&bytes\s+i:[\s\S]*Option::Some\s+ch:[\s\S]*set\s+ww\s+push_u8_impl\s+ww\s+ch[\s\S]*io_bytebuf_free\s+bytes/,
    'append_bytebuf_impl must stream through the borrowed ByteBuf byte boundary, preserve ownership cleanup, and avoid unsafe unwrap',
);

assert.doesNotMatch(code, /store_u8\s+mem_ptr_add\s+buf\s+off/, 'numeric writer helpers must not bypass push_u8_impl with direct buffer stores');
assert.match(code, /fn\s+append_u64_digits_impl\s+<\(StreamWriter,i64\)\*>StreamWriter>[\s\S]*push_u8_impl/, 'numeric writer helpers must funnel digits through push_u8_impl');

console.log('stdlib streamio unsafe unwrap regression passed');
