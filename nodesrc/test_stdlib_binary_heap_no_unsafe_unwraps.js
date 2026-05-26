#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/binary_heap.nepl';
const typesPath = 'stdlib/alloc/collections/binary_heap/types.nepl';
const storagePath = 'stdlib/alloc/collections/binary_heap/storage.nepl';
const orderPath = 'stdlib/alloc/collections/binary_heap/order.nepl';
const apiPath = 'stdlib/alloc/collections/binary_heap/api.nepl';
const apiCreatePath = 'stdlib/alloc/collections/binary_heap/api/create.nepl';
const apiObserverPath = 'stdlib/alloc/collections/binary_heap/api/observer.nepl';
const apiPushPath = 'stdlib/alloc/collections/binary_heap/api/push.nepl';
const apiPopPath = 'stdlib/alloc/collections/binary_heap/api/pop.nepl';
const apiCleanupPath = 'stdlib/alloc/collections/binary_heap/api/cleanup.nepl';

const rootCode = sourceWithoutComments(relPath);
const typesCode = sourceWithoutComments(typesPath);
const storageCode = sourceWithoutComments(storagePath);
const orderCode = sourceWithoutComments(orderPath);
const apiCode = sourceWithoutComments(apiPath);
const apiCreateCode = sourceWithoutComments(apiCreatePath);
const apiObserverCode = sourceWithoutComments(apiObserverPath);
const apiPushCode = sourceWithoutComments(apiPushPath);
const apiPopCode = sourceWithoutComments(apiPopPath);
const apiCleanupCode = sourceWithoutComments(apiCleanupPath);
const code = [
    rootCode,
    typesCode,
    storageCode,
    orderCode,
    apiCode,
    apiCreateCode,
    apiObserverCode,
    apiPushCode,
    apiPopCode,
    apiCleanupCode,
].join('\n');

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(code, pattern, 'BinaryHeap split modules must not use unsafe unwrap helpers in implementation code');
}

assert.doesNotMatch(rootCode, /\bfn\s+/, 'BinaryHeap root facade must not keep implementation bodies');
for (const submodule of ['types', 'api']) {
    assert.match(
        rootCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/binary_heap\\/${submodule}"\\s+as\\s+@merge`),
        `BinaryHeap root facade must re-export binary_heap/${submodule}`,
    );
}
assert.doesNotMatch(apiCode, /\bfn\s+/, 'BinaryHeap api facade must not keep implementation bodies');
for (const submodule of ['create', 'observer', 'push', 'pop', 'cleanup']) {
    assert.match(
        apiCode,
        new RegExp(`pub\\s+#import\\s+"\\.\\/api\\/${submodule}"\\s+as\\s+@merge`),
        `BinaryHeap api facade must re-export api/${submodule}`,
    );
}
assert.match(typesCode, /struct\s+BinaryHeap<\.T>:[\s\S]*len\s+<i32>[\s\S]*cap\s+<i32>[\s\S]*items\s+<Vec<Option<\.T>>>/, 'BinaryHeap must keep typed Vec<Option<T>> storage in its public owner struct');
assert.match(typesCode, /struct\s+BinaryHeapPushError<\.T>:[\s\S]*heap\s+<BinaryHeap<\.T>>[\s\S]*diag\s+<Diag>/, 'BinaryHeap.push failure payload must carry the consumed heap owner and diagnostic');
assert.match(typesCode, /fn\s+binary_heap_push_error_diag\s+<\.T>\s+<\(&BinaryHeapPushError<\.T>\)->Diag>[\s\S]*field::get_ref\s+e\s+"diag"/, 'BinaryHeapPushError diag access must borrow the error payload');
assert.match(typesCode, /fn\s+binary_heap_push_error_heap\s+<\.T:\s*Copy>\s+<\(BinaryHeapPushError<\.T>\)->BinaryHeap<\.T>>[\s\S]*field::get\s+e\s+"heap"/, 'BinaryHeapPushError heap extraction must move the returned owner and remain Copy-only while BinaryHeap is Copy-only');
assert.match(typesCode, /struct\s+BinaryHeapPop<\.T>:[\s\S]*heap\s+<BinaryHeap<\.T>>[\s\S]*item\s+<Option<\.T>>/, 'BinaryHeap must expose an owner-preserving pop result');
assert.match(storageCode, /fn\s+heap_item_at\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32\)->Option<\.T>>/, 'BinaryHeap must read initialized slot state through Option<T>');
assert.match(storageCode, /fn\s+heap_store_slot\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32,Option<\.T>\)\*>(?:\(\)|unit)>[\s\S]*vec::replace<Option<\.T>>/, 'BinaryHeap must update slot state through Vec<Option<T>> replacement');
assert.match(storageCode, /fn\s+heap_alloc_slots\s+<\.T:\s*Copy>\s+<\(i32\)\*>Result<Vec<Option<\.T>>,\s*Diag>>[\s\S]*vec::filled<Option<\.T>>\s+cap\s+none(?:<\.T>)?/, 'BinaryHeap allocation must initialize every slot as None and report allocation failure as Diag');
assert.match(orderCode, /fn\s+heap_sift_up\s+<\.T:\s*Ord&Copy>/, 'BinaryHeap order module must own sift-up');
assert.match(orderCode, /fn\s+heap_sift_down\s+<\.T:\s*Ord&Copy>/, 'BinaryHeap order module must own sift-down');
assert.match(apiCreateCode, /fn\s+new\s+<\.T:\s*Copy>\s+<\(\)\*>Result<BinaryHeap<\.T>,\s*Diag>>/, 'BinaryHeap.new must expose allocation as an impure Result<BinaryHeap<T>, Diag>');
assert.match(apiCreateCode, /fn\s+with_capacity\s+<\.T:\s*Copy>\s+<\(i32\)\*>Result<BinaryHeap<\.T>,\s*Diag>>[\s\S]*heap_normalize_capacity\s+cap[\s\S]*heap_alloc_slots<\.T>\s+cap0/, 'BinaryHeap.with_capacity must own initial allocation');
assert.match(apiPushCode, /fn\s+push\s+<\.T:\s*Ord&Copy>\s+<\(BinaryHeap<\.T>,\.T\)\*>Result<BinaryHeap<\.T>,\s*BinaryHeapPushError<\.T>>>/, 'BinaryHeap.push must expose heap mutation as an owner-preserving Result<BinaryHeap<T>, BinaryHeapPushError<T>>');
assert.match(apiPushCode, /match\s+heap_alloc_slots<\.T>\s+grown_cap:[\s\S]*Result::Err\s+e:[\s\S]*(?:Result::Err<BinaryHeap<\.T>,\s*BinaryHeapPushError<\.T>>|Result::Err)\s+BinaryHeapPushError<\.T>\s+\(BinaryHeap<\.T>\s+len0\s+cap0\s+items\)\s+e/, 'BinaryHeap.push grow failure must return the consumed heap owner in BinaryHeapPushError');
assert.doesNotMatch(apiPushCode, /Result::Err\s+e:[\s\S]{0,120}vec::free<Option<\.T>>\s+items[\s\S]{0,120}err<BinaryHeap<\.T>,\s*Diag>\s+e/, 'BinaryHeap.push must not destroy the consumed heap owner and return Diag only on grow failure');
assert.match(apiObserverCode, /fn\s+len\s+<\.T>\s+<\(&BinaryHeap<\.T>\)->i32>\s+\(hp\):/, 'BinaryHeap.len must borrow the owner and not require Copy for metadata-only observation');
assert.match(apiObserverCode, /#import\s+"core\/math"\s+as\s+\*/, 'BinaryHeap observer module must own the math operators used by is_empty and peek');
assert.match(apiObserverCode, /fn\s+cap\s+<\.T>\s+<\(&BinaryHeap<\.T>\)->i32>\s+\(hp\):/, 'BinaryHeap.cap must borrow the owner and not require Copy for metadata-only observation');
assert.match(apiObserverCode, /fn\s+is_empty\s+<\.T>\s+<\(&BinaryHeap<\.T>\)->bool>\s+\(hp\):/, 'BinaryHeap.is_empty must borrow the owner and not require Copy for metadata-only observation');
assert.match(apiObserverCode, /fn\s+peek\s+<\.T:\s*Copy>\s+<\(&BinaryHeap<\.T>\)->Option<\.T>>\s+\(hp\):/, 'BinaryHeap.peek must borrow the owner');
assert.doesNotMatch(apiObserverCode, /fn\s+(?:len_ref|cap_ref|is_empty_ref|peek_ref)\b/, 'BinaryHeap must not keep duplicate *_ref observer surfaces');
assert.doesNotMatch(apiObserverCode, /fn\s+(?:len|cap|is_empty|peek)\s+<[^>]+>\s+<\(BinaryHeap<\.T>\)/, 'BinaryHeap observers must not consume the owner');
assert.match(apiPopCode, /fn\s+pop_max\s+<\.T:\s*Ord&Copy>\s+<\(BinaryHeap<\.T>\)\*>BinaryHeapPop<\.T>>/, 'BinaryHeap.pop_max must preserve the updated owner');
assert.match(apiPopCode, /fn\s+binary_heap_pop_item\s+<\.T:\s*Copy>\s+<\(&BinaryHeapPop<\.T>\)->Option<\.T>>[\s\S]*field::get_ref\s+p\s+"item"/, 'BinaryHeapPop item access must be a public borrowed accessor');
assert.match(apiPopCode, /fn\s+binary_heap_pop_heap\s+<\.T:\s*Copy>\s+<\(BinaryHeapPop<\.T>\)->BinaryHeap<\.T>>[\s\S]*field::get\s+p\s+"heap"/, 'BinaryHeapPop heap extraction must be a public consuming accessor');
assert.match(apiPopCode, /fn\s+pop\s+<\.T:\s*Ord&Copy>\s+<\(BinaryHeap<\.T>\)\*>Option<\.T>>[\s\S]*binary_heap_pop_item<\.T>\s+&p[\s\S]*free<\.T>\s+binary_heap_pop_heap<\.T>\s+p/, 'BinaryHeap.pop must clean up the updated heap owner through the public accessor');
assert.match(apiCleanupCode, /fn\s+free\s+<\.T:\s*Copy>\s+<\(BinaryHeap<\.T>\)->(?:\(\)|unit)>[\s\S]*vec::free<Option<\.T>>\s+field::get\s+hp\s+"items"/, 'BinaryHeap.free must close the Copy-only Vec<Option<T>> owner');
assert.doesNotMatch(code, /\bMemPtr\b|\balloc_ptr\b|\balloc_raw\b|\brealloc_ptr\b|\bdealloc_raw\b|\bload_i32\b|\bstore_i32\b|\bmem_ptr_addr\b|dealloc_ptr/, 'BinaryHeap must not reintroduce raw header or raw element storage');

const binaryHeapStdlibTests = fs.readFileSync(path.join(repoRoot, 'stdlib/tests/binary_heap.n.md'), 'utf8');
assert.match(binaryHeapStdlibTests, /\bbinary_heap_pop_item\s+&(?:popped|p[0-9])\b/, 'stdlib/tests/binary_heap.n.md must exercise BinaryHeapPop item accessor');
assert.match(binaryHeapStdlibTests, /\bbinary_heap_pop_heap\s+(?:popped|p[0-9])\b/, 'stdlib/tests/binary_heap.n.md must exercise BinaryHeapPop heap accessor');

for (const testPath of [
    'stdlib/tests/binary_heap.n.md',
    'tests/stdlib/binary_heap_collections.n.md',
]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), 'utf8');
    assert.doesNotMatch(testSrc, /\b(?:new|with_capacity|push)<i32>/, `${testPath} must rely on BinaryHeap expected type or receiver evidence instead of explicit producer or mutator postfixes`);
    assert.doesNotMatch(testSrc, /\b(?:len_ref|cap_ref|is_empty_ref|peek_ref)<i32>/, `${testPath} must not use removed BinaryHeap *_ref observers`);
    assert.doesNotMatch(testSrc, /\b(?:len|cap|is_empty|peek)(?:<i32>)?\s+hp[0-9]?\b/, `${testPath} must not call BinaryHeap observers by value`);
    assert.doesNotMatch(testSrc, /\bhp[0-9]?\s+\|>\s+peek(?:<i32>)?\b/, `${testPath} must not pipe BinaryHeap owners into peek`);
    assert.doesNotMatch(testSrc, /field::get(?:_ref)?\s+&?(?:popped|p[0-9])\s+"(?:item|heap)"/, `${testPath} must not project BinaryHeapPop fields directly`);
}

console.log('binary heap unsafe unwrap regression passed');

function sourceWithoutComments(file) {
    return legacyTypeSyntaxView(fs.readFileSync(path.join(repoRoot, file), 'utf8'));
}
