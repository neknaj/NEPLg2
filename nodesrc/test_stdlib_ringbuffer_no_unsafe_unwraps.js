#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/ringbuffer.nepl';
const modulePaths = [
    relPath,
    'stdlib/alloc/collections/ringbuffer/types.nepl',
    'stdlib/alloc/collections/ringbuffer/index.nepl',
    'stdlib/alloc/collections/ringbuffer/storage.nepl',
    'stdlib/alloc/collections/ringbuffer/api.nepl',
];

function implementationCode(file) {
    return legacyTypeSyntaxView(fs.readFileSync(path.join(repoRoot, file), 'utf8'));
}

const rootCode = implementationCode(relPath);
const code = modulePaths.map(implementationCode).join('\n');

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

assert.match(rootCode, /pub\s+#import\s+"\.\/ringbuffer\/types"\s+as\s+@merge/, 'RingBuffer root must re-export types from a submodule');
assert.match(rootCode, /pub\s+#import\s+"\.\/ringbuffer\/api"\s+as\s+@merge/, 'RingBuffer root must re-export API from a submodule');
assert.doesNotMatch(rootCode, /\b(?:struct|fn)\s+/, 'RingBuffer root must remain a public facade without implementation bodies');
assert.match(code, /struct\s+RingBuffer<\.T>:[\s\S]*len\s+<i32>[\s\S]*cap\s+<i32>[\s\S]*head\s+<i32>[\s\S]*items\s+<Vec<Option<\.T>>>/, 'RingBuffer must keep typed Vec<Option<T>> storage in its public owner struct');
assert.match(code, /struct\s+RingBufferPushError<\.T>:[\s\S]*buffer\s+<RingBuffer<\.T>>[\s\S]*diag\s+<Diag>/, 'RingBuffer push failure payload must carry the consumed buffer owner and diagnostic');
assert.match(code, /fn\s+ringbuffer_push_error_diag\s+<\.T>\s+<\(&RingBufferPushError<\.T>\)->Diag>[\s\S]*field::get_ref\s+e\s+"diag"/, 'RingBufferPushError diag access must borrow the error payload');
assert.match(code, /fn\s+ringbuffer_push_error_buffer\s+<\.T:\s*Copy>\s+<\(RingBufferPushError<\.T>\)->RingBuffer<\.T>>[\s\S]*field::get\s+e\s+"buffer"/, 'RingBufferPushError buffer extraction must move the returned owner and remain Copy-only while RingBuffer is Copy-only');
assert.match(code, /struct\s+RingBufferPop<\.T>:[\s\S]*buffer\s+<RingBuffer<\.T>>[\s\S]*item\s+<Option<\.T>>/, 'RingBuffer must expose an owner-preserving pop result');
assert.match(code, /fn\s+ringbuffer_pop_item\s+<\.T:\s*Copy>\s+<\(&RingBufferPop<\.T>\)->Option<\.T>>[\s\S]*field::get_ref\s+p\s+"item"/, 'RingBufferPop item access must be a public borrowed accessor');
assert.match(code, /fn\s+ringbuffer_pop_buffer\s+<\.T:\s*Copy>\s+<\(RingBufferPop<\.T>\)->RingBuffer<\.T>>[\s\S]*field::get\s+p\s+"buffer"/, 'RingBufferPop buffer extraction must be a public consuming accessor');
assert.match(code, /fn\s+ringbuffer_tail_index\s+<\(i32,i32,i32\)->i32>[\s\S]*rem_u\s+add\s+head\s+len\s+cap/, 'RingBuffer index helper must own tail calculation');
assert.match(code, /fn\s+ringbuffer_next_index\s+<\(i32,i32\)->i32>[\s\S]*rem_u\s+add\s+idx\s+1\s+cap/, 'RingBuffer index helper must own next index calculation');
assert.match(code, /fn\s+ringbuffer_item_at\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32\)->Option<\.T>>/, 'RingBuffer must read initialized slot state through Option<T>');
assert.match(code, /fn\s+ringbuffer_store_slot\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32,Option<\.T>\)\*>(?:\(\)|unit)>[\s\S]*vec::replace\s+items\s+idx\s+item/, 'RingBuffer must update slot state through Vec<Option<T>> replacement');
assert.match(code, /fn\s+ringbuffer_alloc_slots\s+<\.T:\s*Copy>[\s\S]*vec::filled\s+cap\s+none(?:<\.T>)?/, 'RingBuffer allocation must initialize every slot as None');
assert.match(code, /fn\s+push\s+<\.T:\s*Copy>\s+<\(RingBuffer<\.T>,\.T\)\*>Result<RingBuffer<\.T>,\s*RingBufferPushError<\.T>>>/, 'RingBuffer push must expose owner-preserving Result<RingBuffer<T>, RingBufferPushError<T>>');
assert.match(code, /fn\s+push\s+<\.T:\s*Copy>[\s\S]*Result::Err\s+d:[\s\S]*(?:Result::Err<RingBuffer<\.T>,\s*RingBufferPushError<\.T>>|Result::Err)\s+RingBufferPushError<\.T>\s+\(RingBuffer<\.T>\s+len0\s+cap0\s+head0\s+items\)\s+d/, 'RingBuffer push grow failure must return the consumed buffer owner in RingBufferPushError');
assert.doesNotMatch(code, /Result::Err\s+d:[\s\S]{0,120}vec::free\s+items[\s\S]{0,120}err<RingBuffer<\.T>,\s*Diag>\s+d/, 'RingBuffer push grow failure must not destroy the consumed owner and return Diag only');
assert.match(code, /fn\s+pop_front\s+<\.T:\s*Copy>\s+<\(RingBuffer<\.T>\)\*>RingBufferPop<\.T>>[\s\S]*ringbuffer_store_slot<\.T>\s+&items\s+head0\s+none(?:<\.T>)?[\s\S]*RingBufferPop<\.T>/, 'RingBuffer pop_front must clear the consumed slot and return the updated owner');
assert.match(code, /fn\s+free\s+<\.T:\s*Copy>\s+<\(RingBuffer<\.T>\)\*>(?:\(\)|unit)>[\s\S]*vec::free\s+field::get\s+rb\s+"items"/, 'RingBuffer.free must close the Copy-only Vec<Option<T>> owner through an impure owner-consuming boundary');
assert.doesNotMatch(code, /\bMemPtr\b|\balloc_ptr\b|\balloc_raw\b|\bdealloc_raw\b|\bload_i32\b|\bstore_i32\b|\bmem_ptr_addr\b|dealloc_ptr/, 'RingBuffer must not reintroduce raw header or raw element storage');

for (const testPath of [
    'stdlib/tests/ringbuffer.n.md',
    'tests/stdlib/ringbuffer_collections.n.md',
    'tests/stdlib/pipe_collections.n.md',
]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), 'utf8');
    assert.doesNotMatch(testSrc, /field::get(?:_ref)?\s+&?p[0-9]?\s+"(?:item|buffer)"/, `${testPath} must not project RingBufferPop fields directly`);
}

console.log('ringbuffer unsafe unwrap regression passed');
