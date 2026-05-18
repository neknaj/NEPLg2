#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/list.nepl';
const typesPath = 'stdlib/alloc/collections/list/types.nepl';
const storagePath = 'stdlib/alloc/collections/list/storage.nepl';
const basicPath = 'stdlib/alloc/collections/list/basic.nepl';
const queryPath = 'stdlib/alloc/collections/list/query.nepl';
const transformPath = 'stdlib/alloc/collections/list/transform.nepl';

const rootCode = sourceWithoutComments(relPath);
const typesCode = sourceWithoutComments(typesPath);
const storageCode = sourceWithoutComments(storagePath);
const basicCode = sourceWithoutComments(basicPath);
const queryCode = sourceWithoutComments(queryPath);
const transformCode = sourceWithoutComments(transformPath);
const code = [rootCode, typesCode, storageCode, basicCode, queryCode, transformCode].join('\n');

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(code, pattern, 'List split modules must not use unsafe unwrap helpers in implementation code');
}

assert.doesNotMatch(rootCode, /\bfn\s+/, 'List root facade must not keep implementation bodies');
for (const submodule of ['types', 'basic', 'query', 'transform']) {
    assert.match(
        rootCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/list\\/${submodule}"\\s+as\\s+@merge`),
        `List root facade must re-export list/${submodule}`,
    );
}

assert.match(
    typesCode,
    /struct\s+List<\.T>:[\s\S]*items\s+<Vec<\.T>>/,
    'List must keep typed Vec<T> storage instead of a raw node pointer',
);
assert.match(
    typesCode,
    /#import\s+"alloc\/collections\/vec"\s+as\s+vec/,
    'List types must delegate storage ownership to the Vec abstraction',
);
assert.match(
    typesCode,
    /struct\s+ListPushError<\.T>:[\s\S]*list\s+<List<\.T>>[\s\S]*diag\s+<Diag>/,
    'List push failure payload must carry the consumed list owner and diagnostic',
);
assert.match(
    typesCode,
    /fn\s+list_push_error_diag\s+<\.T>\s+<\(&ListPushError<\.T>\)->Diag>[\s\S]*field::get_ref\s+e\s+"diag"/,
    'ListPushError diag access must borrow the error payload',
);
assert.match(
    typesCode,
    /fn\s+list_push_error_list\s+<\.T:\s*Copy>\s+<\(ListPushError<\.T>\)->List<\.T>>[\s\S]*field::get\s+e\s+"list"/,
    'ListPushError list extraction must move the returned owner and remain Copy-only while List is Copy-only',
);
assert.match(
    storageCode,
    /fn\s+list_physical_index\s+<\(i32,i32\)->i32>[\s\S]*sub\s+sub\s+len0\s+1\s+idx/,
    'List storage helper must own logical-to-physical index conversion',
);
assert.match(
    storageCode,
    /fn\s+list_free_items\s+<\.T:\s*Copy>\s+<\(List<\.T>\)->\(\)>[\s\S]*field::get\s+lst\s+"items"[\s\S]*vec::free<\.T>\s+items/,
    'List storage helper must close the Copy-only Vec<T> owner',
);
assert.match(
    storageCode,
    /fn\s+list_reverse_items\s+<\.T:\s*Copy>[\s\S]*vec::replace<\.T>\s+items\s+left\s+right_value[\s\S]*vec::replace<\.T>\s+items\s+right\s+left_value/,
    'List storage helper must own in-place reverse swaps',
);
assert.match(
    basicCode,
    /fn\s+cons\s+<\.T:\s*Copy>\s+<\(\.T,List<\.T>\)\*>Result<List<\.T>,\s*ListPushError<\.T>>>[\s\S]*vec::push<\.T>\s+items\s+head/,
    'List.cons must add the logical head through Vec.push and expose owner-preserving Result<List<T>, ListPushError<T>>',
);
assert.match(
    basicCode,
    /Result::Err\s+e:[\s\S]*vec::vec_push_error_vec<\.T>\s+e[\s\S]*Result::Err<List<\.T>,\s*ListPushError<\.T>>\s+ListPushError<\.T>\s+\(List<\.T>\s+returned_items\)\s+\(list_diag_from_vec_error\s+error\)/,
    'List.cons Vec.push failure must return the consumed List owner in ListPushError',
);
assert.doesNotMatch(
    basicCode,
    /Result::Err\s+e:[\s\S]{0,260}vec::free<\.T>\s+returned_items[\s\S]{0,120}diag_err<List<\.T>>/,
    'List.cons failure must not destroy the returned Vec owner and collapse the failure to Diag only',
);
assert.match(
    basicCode,
    /fn\s+tail\s+<\.T:\s*Copy>\s+<\(List<\.T>\)->Option<List<\.T>>>[\s\S]*vec::pop<\.T>\s+items[\s\S]*some<List<\.T>>\s+List<\.T>\s+next_items/,
    'List.tail must remove the logical head by returning the Vec.pop owner',
);
assert.match(
    basicCode,
    /fn\s+reverse\s+<\.T:\s*Copy>\s+<\(List<\.T>\)\*>List<\.T>>[\s\S]*list_reverse_items<\.T>\s+&items\s+0\s+sub\s+n\s+1[\s\S]*List<\.T>\s+items/,
    'List.reverse must reverse storage in place without raw node relinking',
);
assert.match(
    basicCode,
    /fn\s+free\s+<\.T:\s*Copy>\s+<\(List<\.T>\)->\(\)>[\s\S]*list_free_items<\.T>\s+lst/,
    'List.free must close the Copy-only Vec<T> owner',
);
assert.match(
    queryCode,
    /fn\s+len\s+<\.T>\s+<\(&List<\.T>\)->i32>\s+\(lst\):/,
    'List.len must borrow the owner',
);
assert.match(
    queryCode,
    /fn\s+get\s+<\.T:\s*Copy>\s+<\(&List<\.T>,i32\)->Option<\.T>>[\s\S]*list_physical_index\s+n\s+idx/,
    'List.get must borrow the owner and translate logical index to storage index',
);
assert.match(
    queryCode,
    /fn\s+head\s+<\.T:\s*Copy>\s+<\(&List<\.T>\)->Option<\.T>>/,
    'List.head must borrow the owner',
);
assert.match(
    queryCode,
    /fn\s+fold\s+<\.T:\s*Copy,\.U>\s+<\(&List<\.T>, \.U, \(\.U,\.T\)->\.U\)->\.U>/,
    'List.fold must borrow the owner',
);
assert.doesNotMatch(
    queryCode,
    /vec::free<\.T>\s+items/,
    'List borrowed observers must not close the owner storage',
);
assert.match(
    transformCode,
    /fn\s+map\s+<\.T:\s*Copy,\.U:\s*Copy>[\s\S]*vec::with_capacity<\.U>\s+n[\s\S]*vec::push<\.U>\s+out\s+mapped[\s\S]*vec::free<\.T>\s+items/,
    'List.map must build a typed Vec<U> output and close the input storage owner',
);
assert.match(
    transformCode,
    /fn\s+filter\s+<\.T:\s*Copy>[\s\S]*vec::with_capacity<\.T>\s+n[\s\S]*vec::push<\.T>\s+out\s+value[\s\S]*vec::free<\.T>\s+items/,
    'List.filter must build a typed Vec<T> output and close the input storage owner',
);
assert.doesNotMatch(
    code,
    /\bMemPtr\b|\balloc_ptr\b|\balloc_raw\b|\brealloc_ptr\b|\bdealloc_raw\b|\bload_i32\b|\bstore_i32\b|\bmem_ptr_addr\b|dealloc_ptr|\bptr\s+<i32>/,
    'List must not reintroduce raw node storage, raw headers, or raw pointer sentinels',
);

for (const testPath of ['stdlib/tests/list.n.md', 'tests/stdlib/list_collections.n.md']) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), 'utf8');
    assert.match(testSrc, /\blen<i32>\s+&/, `${testPath} must exercise borrowed List.len`);
    assert.match(testSrc, /\bget<i32>\s+&/, `${testPath} must exercise borrowed List.get`);
    assert.doesNotMatch(testSrc, /\blen<i32>\s+(?!&)\w+/, `${testPath} must not call List.len by value`);
    assert.doesNotMatch(testSrc, /\bget<i32>\s+(?!&)\w+/, `${testPath} must not call List.get by value`);
    assert.match(testSrc, /\bfree<i32>\s+/, `${testPath} must explicitly free observed List owners`);
}

const pipeCollections = fs.readFileSync(path.join(repoRoot, 'tests/stdlib/pipe_collections.n.md'), 'utf8');
const pipeListSection = pipeCollections.match(/## pipe_list_alias_chain[\s\S]*?(?=\n## |$)/);
assert.ok(pipeListSection, 'pipe_collections must keep a List pipe fixture');
assert.match(pipeListSection[0], /\blen<i32>\s+&xs0\b/, 'pipe List fixture must borrow for len');
assert.match(pipeListSection[0], /\bget<i32>\s+&xs1\s+1\b/, 'pipe List fixture must borrow for get');
assert.match(pipeListSection[0], /\bfree<i32>\s+xs0\b/, 'pipe List fixture must free xs0 after observation');
assert.match(pipeListSection[0], /\bfree<i32>\s+xs1\b/, 'pipe List fixture must free xs1 after observation');
assert.doesNotMatch(pipeListSection[0], /\blen<i32>\s+xs\d+\b/, 'pipe List fixture must not call len by value');
assert.doesNotMatch(pipeListSection[0], /\bget<i32>\s+xs\d+\s+/, 'pipe List fixture must not call get by value');

console.log('list unsafe unwrap regression passed');

function sourceWithoutComments(file) {
    return fs.readFileSync(path.join(repoRoot, file), 'utf8')
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}
