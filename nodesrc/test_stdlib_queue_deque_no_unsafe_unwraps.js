#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');
const relPaths = [
    'stdlib/alloc/collections/queue.nepl',
    'stdlib/alloc/collections/queue/types.nepl',
    'stdlib/alloc/collections/queue/index.nepl',
    'stdlib/alloc/collections/queue/storage.nepl',
    'stdlib/alloc/collections/queue/api.nepl',
    'stdlib/alloc/collections/deque.nepl',
    'stdlib/alloc/collections/deque/types.nepl',
    'stdlib/alloc/collections/deque/index.nepl',
    'stdlib/alloc/collections/deque/storage.nepl',
    'stdlib/alloc/collections/deque/api.nepl',
];

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

function implementationCode(relPath) {
    const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
    return legacyTypeSyntaxView(src);
}

for (const relPath of relPaths) {
    const code = implementationCode(relPath);
    for (const pattern of forbidden) {
        assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap helpers in implementation code`);
    }
}

const queueRoot = implementationCode('stdlib/alloc/collections/queue.nepl');
assert.match(queueRoot, /pub\s+#import\s+"\.\.\/?queue\/types"|pub\s+#import\s+"\.\/queue\/types"/, 'Queue root must re-export types from a submodule');
assert.match(queueRoot, /pub\s+#import\s+"\.\.\/?queue\/api"|pub\s+#import\s+"\.\/queue\/api"/, 'Queue root must re-export API from a submodule');
assert.doesNotMatch(queueRoot, /\b(?:struct|fn)\s+/, 'Queue root must remain a public facade without implementation bodies');

const queue = [
    'stdlib/alloc/collections/queue/types.nepl',
    'stdlib/alloc/collections/queue/index.nepl',
    'stdlib/alloc/collections/queue/storage.nepl',
    'stdlib/alloc/collections/queue/api.nepl',
].map(implementationCode).join('\n');
assert.match(queue, /struct\s+Queue<\.T>:[\s\S]*len\s+<i32>[\s\S]*cap\s+<i32>[\s\S]*head\s+<i32>[\s\S]*items\s+<Vec<Option<\.T>>>/, 'Queue must keep typed Vec<Option<T>> storage in its public owner struct');
assert.match(queue, /struct\s+QueuePushError<\.T>:[\s\S]*queue\s+<Queue<\.T>>[\s\S]*diag\s+<Diag>/, 'Queue push failure payload must carry the consumed queue owner and diagnostic');
assert.match(queue, /fn\s+queue_push_error_diag\s+<\.T>\s+<\(&QueuePushError<\.T>\)->Diag>[\s\S]*field::get_ref\s+e\s+"diag"/, 'QueuePushError diag access must borrow the error payload');
assert.match(queue, /fn\s+queue_push_error_queue\s+<\.T:\s*Copy>\s+<\(QueuePushError<\.T>\)->Queue<\.T>>[\s\S]*field::get\s+e\s+"queue"/, 'QueuePushError queue extraction must move the returned owner and remain Copy-only while Queue is Copy-only');
assert.match(queue, /struct\s+QueuePop<\.T>:[\s\S]*queue\s+<Queue<\.T>>[\s\S]*item\s+<Option<\.T>>/, 'Queue must expose an owner-preserving pop result');
assert.match(queue, /fn\s+queue_pop_item\s+<\.T:\s*Copy>\s+<\(&QueuePop<\.T>\)->Option<\.T>>[\s\S]*field::get_ref\s+p\s+"item"/, 'QueuePop item access must be a public borrowed accessor');
assert.match(queue, /fn\s+queue_pop_queue\s+<\.T:\s*Copy>\s+<\(QueuePop<\.T>\)->Queue<\.T>>[\s\S]*field::get\s+p\s+"queue"/, 'QueuePop queue extraction must be a public consuming accessor');
assert.match(queue, /fn\s+len\s+<\.T>\s+<\(&Queue<\.T>\)->i32>\s+\(q\):/, 'Queue.len must borrow the owner and not require Copy for metadata-only observation');
assert.match(queue, /fn\s+is_empty\s+<\.T>\s+<\(&Queue<\.T>\)->bool>\s+\(q\):/, 'Queue.is_empty must borrow the owner and not require Copy for metadata-only observation');
assert.match(queue, /fn\s+peek\s+<\.T:\s*Copy>\s+<\(&Queue<\.T>\)->Option<\.T>>\s+\(q\):/, 'Queue.peek must borrow the owner');
assert.doesNotMatch(queue, /fn\s+(?:len_ref|is_empty_ref|peek_ref)\b/, 'Queue must not keep duplicate *_ref observer surfaces');
assert.doesNotMatch(queue, /fn\s+(?:len|is_empty|peek)\s+<[^>]+>\s+<\(Queue<\.T>\)/, 'Queue observers must not consume the owner');
assert.match(queue, /fn\s+queue_item_at\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32\)->Option<\.T>>/, 'Queue must read initialized slot state through Option<T>');
assert.match(queue, /fn\s+queue_store_slot\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32,Option<\.T>\)\*>(?:\(\)|unit)>[\s\S]*vec::replace<Option<\.T>>/, 'Queue must update slot state through Vec<Option<T>> replacement');
assert.match(queue, /fn\s+queue_alloc_slots\s+<\.T:\s*Copy>[\s\S]*vec::filled<Option<\.T>>\s+cap\s+none(?:<\.T>)?/, 'Queue allocation must initialize every slot as None');
assert.match(queue, /fn\s+push\s+<\.T:\s*Copy>\s+<\(Queue<\.T>,\.T\)\*>Result<Queue<\.T>,\s*QueuePushError<\.T>>>/, 'Queue push must expose owner-preserving Result<Queue<T>, QueuePushError<T>>');
assert.match(queue, /fn\s+push\s+<\.T:\s*Copy>[\s\S]*Result::Err\s+d:[\s\S]*(?:Result::Err<Queue<\.T>,\s*QueuePushError<\.T>>|Result::Err)\s+QueuePushError<\.T>\s+\(Queue<\.T>\s+len0\s+cap0\s+head0\s+items\)\s+d/, 'Queue push grow failure must return the consumed queue owner in QueuePushError');
assert.doesNotMatch(queue, /Result::Err\s+d:[\s\S]{0,120}vec::free<Option<\.T>>\s+items[\s\S]{0,120}err<Queue<\.T>,\s*Diag>\s+d/, 'Queue push grow failure must not destroy the consumed owner and return Diag only');
assert.match(queue, /fn\s+pop_front\s+<\.T:\s*Copy>\s+<\(Queue<\.T>\)\*>QueuePop<\.T>>[\s\S]*queue_store_slot<\.T>\s+&items\s+head0\s+none(?:<\.T>)?[\s\S]*QueuePop<\.T>/, 'Queue pop_front must clear the consumed slot and return the updated owner');
assert.match(queue, /fn\s+free\s+<\.T:\s*Copy>\s+<\(Queue<\.T>\)->(?:\(\)|unit)>[\s\S]*vec::free<Option<\.T>>\s+field::get\s+q\s+"items"/, 'Queue.free must close the Copy-only Vec<Option<T>> owner');
assert.doesNotMatch(queue, /\bMemPtr\b|\balloc_ptr\b|\balloc_raw\b|\bdealloc_raw\b|\bload_i32\b|\bstore_i32\b|\bmem_ptr_addr\b/, 'Queue must not reintroduce raw header or raw element storage');

const dequeRoot = implementationCode('stdlib/alloc/collections/deque.nepl');
assert.match(dequeRoot, /pub\s+#import\s+"\.\/deque\/types"\s+as\s+@merge/, 'Deque root must re-export types from a submodule');
assert.match(dequeRoot, /pub\s+#import\s+"\.\/deque\/api"\s+as\s+@merge/, 'Deque root must re-export API from a submodule');
assert.doesNotMatch(dequeRoot, /\b(?:struct|fn)\s+/, 'Deque root must remain a public facade without implementation bodies');

const deque = [
    'stdlib/alloc/collections/deque/types.nepl',
    'stdlib/alloc/collections/deque/index.nepl',
    'stdlib/alloc/collections/deque/storage.nepl',
    'stdlib/alloc/collections/deque/api.nepl',
].map(implementationCode).join('\n');
assert.match(deque, /struct\s+Deque<\.T>:[\s\S]*len\s+<i32>[\s\S]*cap\s+<i32>[\s\S]*head\s+<i32>[\s\S]*items\s+<Vec<Option<\.T>>>/, 'Deque must keep typed Vec<Option<T>> storage in its public owner struct');
assert.match(deque, /struct\s+DequePushError<\.T>:[\s\S]*deque\s+<Deque<\.T>>[\s\S]*diag\s+<Diag>/, 'Deque push failure payload must carry the consumed deque owner and diagnostic');
assert.match(deque, /fn\s+deque_push_error_diag\s+<\.T>\s+<\(&DequePushError<\.T>\)->Diag>[\s\S]*field::get_ref\s+e\s+"diag"/, 'DequePushError diag access must borrow the error payload');
assert.match(deque, /fn\s+deque_push_error_deque\s+<\.T:\s*Copy>\s+<\(DequePushError<\.T>\)->Deque<\.T>>[\s\S]*field::get\s+e\s+"deque"/, 'DequePushError deque extraction must move the returned owner and remain Copy-only while Deque is Copy-only');
assert.match(deque, /struct\s+DequePop<\.T>:[\s\S]*deque\s+<Deque<\.T>>[\s\S]*item\s+<Option<\.T>>/, 'Deque must expose an owner-preserving pop result');
assert.match(deque, /fn\s+len\s+<\.T>\s+<\(&Deque<\.T>\)->i32>\s+\(dq\):/, 'Deque.len must borrow the owner and not require Copy for metadata-only observation');
assert.match(deque, /fn\s+cap\s+<\.T>\s+<\(&Deque<\.T>\)->i32>\s+\(dq\):/, 'Deque.cap must borrow the owner and not require Copy for metadata-only observation');
assert.match(deque, /fn\s+is_empty\s+<\.T>\s+<\(&Deque<\.T>\)->bool>\s+\(dq\):/, 'Deque.is_empty must borrow the owner and not require Copy for metadata-only observation');
assert.match(deque, /fn\s+peek_front\s+<\.T:\s*Copy>\s+<\(&Deque<\.T>\)->Option<\.T>>\s+\(dq\):/, 'Deque.peek_front must borrow the owner');
assert.match(deque, /fn\s+peek_back\s+<\.T:\s*Copy>\s+<\(&Deque<\.T>\)->Option<\.T>>\s+\(dq\):/, 'Deque.peek_back must borrow the owner');
assert.match(deque, /fn\s+deque_pop_item\s+<\.T:\s*Copy>\s+<\(&DequePop<\.T>\)->Option<\.T>>\s+\(p\):/, 'Deque pop item access must borrow the pop result');
assert.match(deque, /fn\s+deque_pop_deque\s+<\.T:\s*Copy>\s+<\(DequePop<\.T>\)->Deque<\.T>>\s+\(p\):/, 'Deque pop deque access must move the updated owner out of the pop result');
assert.doesNotMatch(deque, /fn\s+(?:len_ref|cap_ref|is_empty_ref|peek_front_ref|peek_back_ref)\b/, 'Deque must not keep duplicate *_ref observer surfaces');
assert.doesNotMatch(deque, /fn\s+(?:len|cap|is_empty|peek_front|peek_back)\s+<[^>]+>\s+<\(Deque<\.T>\)/, 'Deque observers must not consume the owner');
assert.match(deque, /fn\s+deque_item_at\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32\)->Option<\.T>>/, 'Deque must read initialized slot state through Option<T>');
assert.match(deque, /fn\s+deque_store_slot\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32,Option<\.T>\)\*>(?:\(\)|unit)>[\s\S]*vec::replace<Option<\.T>>/, 'Deque must update slot state through Vec<Option<T>> replacement');
assert.match(deque, /fn\s+deque_alloc_slots\s+<\.T:\s*Copy>[\s\S]*vec::filled<Option<\.T>>\s+cap\s+none(?:<\.T>)?/, 'Deque allocation must initialize every slot as None');
assert.match(deque, /fn\s+push_front\s+<\.T:\s*Copy>\s+<\(Deque<\.T>,\.T\)\*>Result<Deque<\.T>,\s*DequePushError<\.T>>>/, 'Deque push_front must expose owner-preserving Result<Deque<T>, DequePushError<T>>');
assert.match(deque, /fn\s+push_back\s+<\.T:\s*Copy>\s+<\(Deque<\.T>,\.T\)\*>Result<Deque<\.T>,\s*DequePushError<\.T>>>/, 'Deque push_back must expose owner-preserving Result<Deque<T>, DequePushError<T>>');
assert.match(deque, /fn\s+push_front\s+<\.T:\s*Copy>[\s\S]*Result::Err\s+d:[\s\S]*(?:Result::Err<Deque<\.T>,\s*DequePushError<\.T>>|Result::Err)\s+DequePushError<\.T>\s+\(Deque<\.T>\s+len0\s+cap0\s+head0\s+items\)\s+d/, 'Deque push_front grow failure must return the consumed deque owner in DequePushError');
assert.match(deque, /fn\s+push_back\s+<\.T:\s*Copy>[\s\S]*Result::Err\s+d:[\s\S]*(?:Result::Err<Deque<\.T>,\s*DequePushError<\.T>>|Result::Err)\s+DequePushError<\.T>\s+\(Deque<\.T>\s+len0\s+cap0\s+head0\s+items\)\s+d/, 'Deque push_back grow failure must return the consumed deque owner in DequePushError');
assert.doesNotMatch(deque, /Result::Err\s+d:[\s\S]{0,120}vec::free<Option<\.T>>\s+items[\s\S]{0,120}err<Deque<\.T>,\s*Diag>\s+d/, 'Deque push grow failure must not destroy the consumed owner and return Diag only');
assert.match(deque, /fn\s+push_front\s+<\.T:\s*Copy>[\s\S]*deque_prev_index[\s\S]*deque_store_slot<\.T>\s+&items\s+head1\s+some(?:<\.T>)?\s+item/, 'Deque push_front must write a typed Some slot at the new head');
assert.match(deque, /fn\s+push_back\s+<\.T:\s*Copy>[\s\S]*deque_tail_index[\s\S]*deque_store_slot<\.T>\s+&items\s+tail\s+some(?:<\.T>)?\s+item/, 'Deque push_back must write a typed Some slot at the tail');
assert.match(deque, /fn\s+pop_front\s+<\.T:\s*Copy>\s+<\(Deque<\.T>\)\*>DequePop<\.T>>[\s\S]*deque_store_slot<\.T>\s+&items\s+head0\s+none(?:<\.T>)?[\s\S]*DequePop<\.T>/, 'Deque pop_front must clear the consumed front slot and return the updated owner');
assert.match(deque, /fn\s+pop_back\s+<\.T:\s*Copy>\s+<\(Deque<\.T>\)\*>DequePop<\.T>>[\s\S]*deque_back_index[\s\S]*deque_store_slot<\.T>\s+&items\s+back\s+none(?:<\.T>)?[\s\S]*DequePop<\.T>/, 'Deque pop_back must clear the consumed back slot and return the updated owner');
assert.match(deque, /fn\s+free\s+<\.T:\s*Copy>\s+<\(Deque<\.T>\)->(?:\(\)|unit)>[\s\S]*vec::free<Option<\.T>>\s+field::get\s+dq\s+"items"/, 'Deque.free must close the Copy-only Vec<Option<T>> owner');
assert.doesNotMatch(deque, /\bMemPtr\b|\balloc_ptr\b|\balloc_raw\b|\bdealloc_raw\b|\bload_i32\b|\bstore_i32\b|\bmem_ptr_addr\b/, 'Deque must not reintroduce raw header or raw element storage');

for (const testPath of [
    'stdlib/tests/queue.n.md',
    'stdlib/tests/deque.n.md',
    'tests/stdlib/queue_collections.n.md',
    'tests/stdlib/deque_collections.n.md',
    'tests/stdlib/pipe_collections.n.md',
]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), 'utf8');
    assert.doesNotMatch(testSrc, /\b(?:len_ref|cap_ref|is_empty_ref|peek_ref|peek_front_ref|peek_back_ref)<i32>/, `${testPath} must not use removed Queue/Deque *_ref observers`);
    assert.doesNotMatch(testSrc, /\b(?:len|cap|is_empty|peek|peek_front|peek_back)<i32>\s+(?:q|dq)[0-9]?\b/, `${testPath} must not call Queue/Deque observers by value`);
    assert.doesNotMatch(testSrc, /\b(?:q|dq)[0-9]?\s+\|>\s+(?:peek|peek_front|peek_back)(?:<i32>)?\b/, `${testPath} must not pipe Queue/Deque owners into observers`);
    assert.doesNotMatch(testSrc, /field::get(?:_ref)?\s+&?p[0-9]?\s+"(?:item|queue)"/, `${testPath} must not project QueuePop fields directly`);
}

for (const testPath of [
    'stdlib/tests/queue.n.md',
    'tests/stdlib/queue_collections.n.md',
]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), 'utf8');
    assert.doesNotMatch(testSrc, /\b(?:new|with_capacity|push)<i32>/, `${testPath} must rely on Queue expected type or receiver evidence instead of explicit producer or mutator postfixes`);
}

console.log('queue/deque unsafe unwrap regression passed');
