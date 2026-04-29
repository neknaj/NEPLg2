#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/hashmap.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

function between(source, start, end) {
    const startIdx = source.indexOf(start);
    assert.notEqual(startIdx, -1, `missing section start: ${start}`);
    const endIdx = source.indexOf(end, startIdx + start.length);
    assert.notEqual(endIdx, -1, `missing section end: ${end}`);
    return source.slice(startIdx, endIdx);
}

const newSection = between(code, 'fn hashmap_new_with_capacity ', 'fn new ');
const rehashSection = between(code, 'fn hashmap_rehash_to ', 'fn hashmap_prepare_insert ');
const insertSection = between(code, 'fn insert ', 'fn get ');
const getSection = between(code, 'fn get ', 'fn contains ');
const containsSection = between(code, 'fn contains ', 'fn remove ');
const lenSection = between(code, 'fn len ', 'fn free ');
const freeSection = code.slice(code.indexOf('fn free '));

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
    /dealloc_ptr/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap or checked deallocation helpers in implementation code`);
}

assert.match(
    code,
    /struct\s+HashMap<\.K,\.V,\.H>:\s+hdr\s+<MemPtr<u8>>\s+entries\s+<MemPtr<u8>>\s+hasher\s+<\.H>/,
    'HashMap must carry header and entries owners as separate fields',
);

assert.doesNotMatch(
    code,
    /store_i32\s+add\s+hdr(?:_raw)?\s+8\s+(?:entries|mem_ptr_addr\s+entries)/,
    'HashMap must not hide the entries owner in the header raw cells',
);

assert.match(
    newSection,
    /let\s+hdr\s+<i32>\s+alloc_raw\s+12[\s\S]*dealloc_raw\s+mem_ptr_addr\s+entries\s+entry_bytes[\s\S]*store_i32\s+add\s+hdr\s+8\s+0[\s\S]*ok<HashMap<\.K,\.V,\.H>,\s*Diag>\s+HashMap\s+\(mem_ptr_wrap\s+hdr\)\s+entries\s+hasher/,
    'HashMap constructor must allocate a 12-byte metadata header, release entries on header failure, and return both owners',
);

assert.match(
    rehashSection,
    /let\s+hdr\s+<MemPtr<u8>>\s+field::get\s+hm\s+"hdr"[\s\S]*let\s+old_entries_mem\s+<MemPtr<u8>>\s+field::get\s+hm\s+"entries"[\s\S]*Result::Err\s+d:[\s\S]*dealloc_raw\s+old_entries\s+mul\s+old_cap\s+old_size[\s\S]*dealloc_raw\s+hdr_raw\s+12/,
    'HashMap rehash failure must close the consumed old entries and header owners',
);

assert.match(
    rehashSection,
    /Result::Ok\s+new_entries_mem:[\s\S]*dealloc_raw\s+old_entries\s+mul\s+old_cap\s+old_size[\s\S]*ok<HashMap<\.K,\.V,\.H>,\s*Diag>\s+HashMap\s+hdr\s+new_entries_mem\s+hasher/,
    'HashMap rehash success must release old entries and return the new entries owner',
);

assert.match(
    insertSection,
    /let\s+entries_mem\s+<MemPtr<u8>>\s+field::get\s+ready\s+"entries"[\s\S]*let\s+entries\s+<i32>\s+mem_ptr_addr\s+entries_mem[\s\S]*ok<HashMap<\.K,\.V,\.H>,\s*Diag>\s+HashMap\s+hdr\s+entries_mem\s+hasher/,
    'HashMap insert must move the entries owner from the consumed map into the returned map',
);

assert.match(
    getSection,
    /fn\s+get\s+<\.K:\s*HashKey&Copy,\.V:\s*Copy,\.H:\s*Hasher<\.K>&Copy>\s+<\(&HashMap<\.K,\.V,\.H>,\.K\)->Option<\.V>>/,
    'HashMap.get must be a borrow-based read API returning only Copy values',
);

assert.match(
    getSection,
    /let\s+hdr\s+<MemPtr<u8>>\s+\*field::get_ref\s+hm\s+"hdr"[\s\S]*let\s+entries_mem\s+<MemPtr<u8>>\s+\*field::get_ref\s+hm\s+"entries"/,
    'HashMap.get must inspect header and entries through field references',
);

assert.match(
    containsSection,
    /fn\s+contains\s+<\.K:\s*HashKey&Copy,\.V:\s*Copy,\.H:\s*Hasher<\.K>&Copy>\s+<\(&HashMap<\.K,\.V,\.H>,\.K\)->bool>/,
    'HashMap.contains must be a borrow-based read API',
);

assert.match(
    lenSection,
    /fn\s+len\s+<\.K:\s*HashKey&Copy,\.V:\s*Copy,\.H:\s*Hasher<\.K>&Copy>\s+<\(&HashMap<\.K,\.V,\.H>\)->i32>/,
    'HashMap.len must be a borrow-based read API',
);

assert.match(
    freeSection,
    /let\s+hdr\s+<MemPtr<u8>>\s+field::get\s+hm\s+"hdr"[\s\S]*let\s+entries_mem\s+<MemPtr<u8>>\s+field::get\s+hm\s+"entries"[\s\S]*dealloc_raw\s+entries\s+entry_bytes[\s\S]*dealloc_raw\s+hdr_raw\s+12/,
    'HashMap.free must explicitly release entries and header owners',
);

console.log('hashmap owner contract regression passed');
