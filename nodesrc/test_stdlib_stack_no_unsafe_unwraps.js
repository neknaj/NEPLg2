#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/alloc/collections/stack.nepl';
const modulePaths = [
    relPath,
    'stdlib/alloc/collections/stack/types.nepl',
    'stdlib/alloc/collections/stack/storage.nepl',
    'stdlib/alloc/collections/stack/api.nepl',
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

assert.match(rootCode, /pub\s+#import\s+"\.\/stack\/types"\s+as\s+@merge/, 'Stack root must re-export types from a submodule');
assert.match(rootCode, /pub\s+#import\s+"\.\/stack\/api"\s+as\s+@merge/, 'Stack root must re-export API from a submodule');
assert.doesNotMatch(rootCode, /\b(?:struct|fn)\s+/, 'Stack root must remain a public facade without implementation bodies');
assert.match(code, /struct\s+Stack<\.T>:[\s\S]*len\s+<i32>[\s\S]*cap\s+<i32>[\s\S]*items\s+<Vec<Option<\.T>>>/, 'Stack must keep typed Vec<Option<T>> storage in its public owner struct');
assert.match(code, /struct\s+StackPushError<\.T>:[\s\S]*stack\s+<Stack<\.T>>[\s\S]*diag\s+<Diag>/, 'Stack push failure payload must carry the consumed stack owner and diagnostic');
assert.match(code, /fn\s+stack_push_error_diag\s+<\.T>\s+<\(&StackPushError<\.T>\)->Diag>[\s\S]*field::get_ref\s+e\s+"diag"/, 'StackPushError diag access must borrow the error payload');
assert.match(code, /fn\s+stack_push_error_stack\s+<\.T:\s*Copy>\s+<\(StackPushError<\.T>\)->Stack<\.T>>[\s\S]*field::get\s+e\s+"stack"/, 'StackPushError stack extraction must move the returned owner and remain Copy-only while Stack is Copy-only');
assert.match(code, /struct\s+StackPop<\.T>:[\s\S]*stack\s+<Stack<\.T>>[\s\S]*item\s+<Option<\.T>>/, 'Stack must expose an owner-preserving pop result');
assert.doesNotMatch(code, /#import\s+"alloc\/collections\/vec"\s+as\s+vec/, 'Stack implementation modules must not import the broad Vec root facade');
assert.match(code, /#import\s+"alloc\/collections\/vec\/types"\s+as\s+\*/, 'Stack implementation modules must import Vec types directly');
assert.match(code, /#import\s+"alloc\/collections\/vec\/query\/get"\s+as\s+vec_get/, 'Stack storage must import Vec get from its query module');
assert.match(code, /#import\s+"alloc\/collections\/vec\/mutation\/replace"\s+as\s+vec_replace/, 'Stack storage must import Vec replace from its mutation module');
assert.match(code, /#import\s+"alloc\/collections\/vec\/storage\/fill"\s+as\s+vec_fill/, 'Stack storage must import Vec filled construction from its storage module');
assert.match(code, /#import\s+"alloc\/collections\/vec\/mutation\/cleanup"\s+as\s+vec_cleanup/, 'Stack API must import Vec cleanup from its mutation module');
assert.match(code, /fn\s+stack_item_at\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32\)->Option<\.T>>/, 'Stack must read initialized slot state through Option<T>');
assert.match(code, /fn\s+stack_item_at\s+<\.T:\s*Copy>[\s\S]*vec_get::get\s+items\s+idx/, 'Stack must read slot state through the narrow Vec get module');
assert.match(code, /fn\s+stack_store_slot\s+<\.T:\s*Copy>\s+<\(&Vec<Option<\.T>>,i32,Option<\.T>\)\*>unit>[\s\S]*vec_replace::replace\s+items\s+idx\s+item/, 'Stack must update slot state through the narrow Vec replace module');
assert.match(code, /fn\s+stack_alloc_slots\s+<\.T:\s*Copy>[\s\S]*vec_fill::filled\s+cap\s+none/, 'Stack allocation must initialize every slot through the narrow Vec fill module');
assert.match(code, /fn\s+push\s+<\.T:\s*Copy>\s+<\(Stack<\.T>,\.T\)\*>Result<Stack<\.T>,\s*StackPushError<\.T>>>/, 'Stack push must expose owner-preserving Result<Stack<T>, StackPushError<T>>');
assert.match(code, /fn\s+push\s+<\.T:\s*Copy>[\s\S]*Result::Err\s+d:[\s\S]*(?:Result::Err<Stack<\.T>,\s*StackPushError<\.T>>|Result::Err)\s+StackPushError<\.T>\s+\(Stack<\.T>\s+len0\s+cap0\s+items\)\s+d/, 'Stack push grow failure must return the consumed stack owner in StackPushError');
assert.doesNotMatch(code, /Result::Err\s+d:[\s\S]{0,120}vec::free\s+items[\s\S]{0,120}err<Stack<\.T>,\s*Diag>\s+d/, 'Stack push grow failure must not destroy the consumed owner and return Diag only');
assert.match(code, /fn\s+pop_top\s+<\.T:\s*Copy>\s+<\(Stack<\.T>\)\*>StackPop<\.T>>[\s\S]*stack_store_slot<\.T>\s+&items\s+next_len\s+none[\s\S]*StackPop<\.T>/, 'Stack pop_top must clear the consumed slot and return the updated owner');
assert.match(code, /fn\s+stack_pop_item\s+<\.T:\s*Copy>\s+<\(&StackPop<\.T>\)->Option<\.T>>[\s\S]*field::get_ref\s+p\s+"item"/, 'StackPop item access must be a public borrowed accessor');
assert.match(code, /fn\s+stack_pop_stack\s+<\.T:\s*Copy>\s+<\(StackPop<\.T>\)->Stack<\.T>>[\s\S]*field::get\s+p\s+"stack"/, 'StackPop stack extraction must be a public consuming accessor');
assert.match(code, /fn\s+len\s+<\.T>\s+<\(&Stack<\.T>\)->i32>\s+\(stk\):/, 'Stack.len must borrow the owner and not require Copy for metadata-only observation');
assert.match(code, /fn\s+is_empty\s+<\.T>\s+<\(&Stack<\.T>\)->bool>\s+\(stk\):/, 'Stack.is_empty must borrow the owner and not require Copy for metadata-only observation');
assert.match(code, /fn\s+peek\s+<\.T:\s*Copy>\s+<\(&Stack<\.T>\)->Option<\.T>>\s+\(stk\):/, 'Stack.peek must borrow the owner');
assert.match(code, /fn\s+get\s+<\.T:\s*Copy>\s+<\(&Stack<\.T>,i32\)->Option<\.T>>\s+\(stk,\s*idx\):/, 'Stack.get must borrow the owner');
assert.doesNotMatch(code, /fn\s+(?:len_ref|is_empty_ref|peek_ref|get_ref)\b/, 'Stack must not keep duplicate *_ref observer surfaces');
assert.doesNotMatch(code, /fn\s+(?:len|is_empty|peek|get)\s+<[^>]+>\s+<\(Stack<\.T>\)/, 'Stack observers must not consume the owner');
assert.match(code, /fn\s+free\s+<\.T:\s*Copy>\s+<\(Stack<\.T>\)\*>unit>[\s\S]*let\s+items\s+<Vec<Option<\.T>>>\s+field::get\s+stk\s+"items"[\s\S]*vec_cleanup::free\s+items/, 'Stack.free must move the Copy-only Vec<Option<T>> owner into a typed local and close it through the narrow impure Vec cleanup module');
assert.doesNotMatch(code, /\bMemPtr\b|\balloc_ptr\b|\balloc_raw\b|\brealloc_ptr\b|\bdealloc_raw\b|\bload_i32\b|\bstore_i32\b|\bmem_ptr_addr\b|dealloc_ptr/, 'Stack must not reintroduce raw header or raw element storage');

for (const testPath of [
    'stdlib/tests/stack.n.md',
    'tests/stdlib/stack_collections.n.md',
]) {
    const testSrc = fs.readFileSync(path.join(repoRoot, testPath), 'utf8');
    assert.doesNotMatch(testSrc, /\b(?:len_ref|is_empty_ref|peek_ref|get_ref)<i32>/, `${testPath} must not use removed Stack *_ref observers`);
    assert.doesNotMatch(testSrc, /\b(?:new|push)<i32>/, `${testPath} must rely on Stack expected type or receiver evidence instead of explicit producer or mutator postfixes`);
    assert.doesNotMatch(testSrc, /\b(?:len|get|peek|pop|pop_top|free|clear)<i32>/, `${testPath} must rely on Stack receiver type evidence instead of explicit observer or cleanup postfixes`);
    assert.doesNotMatch(testSrc, /\b(?:len|is_empty|peek|get)(?:<i32>)?\s+s[0-9]?\b/, `${testPath} must not call Stack observers by value`);
    assert.doesNotMatch(testSrc, /\bs[0-9]?\s+\|>\s+(?:len|is_empty|peek|get)(?:<i32>)?\b/, `${testPath} must not pipe Stack owners into observers`);
    assert.match(testSrc, /\b(?:len|peek|get)\s+&s[0-9]?\b/, `${testPath} must exercise borrowed Stack observers through primary names`);
    assert.doesNotMatch(testSrc, /field::get(?:_ref)?\s+&?p[0-9]?\s+"(?:item|stack)"/, `${testPath} must not project StackPop fields directly`);
    assert.match(testSrc, /\bstack_pop_item\s+&p[0-9]?\b/, `${testPath} must exercise StackPop item accessor`);
    assert.match(testSrc, /\bstack_pop_stack\s+p[0-9]?\b/, `${testPath} must exercise StackPop stack accessor`);
    assert.doesNotMatch(testSrc, /unwrap_ok<Stack<i32>/, `${testPath} must rely on NEPLg2.1 type evidence instead of explicit Stack unwrap_ok postfix`);
}

for (const rel of [
    'examples/rpn.nepl',
    'examples/rpn_legacy.nepl',
    'examples/bf.nepl',
]) {
    const exampleSrc = fs.readFileSync(path.join(repoRoot, rel), 'utf8');
    assert.doesNotMatch(exampleSrc, /\bstk::(?:len_ref|is_empty_ref|peek_ref|get_ref)\b/, `${rel} must use primary borrowed Stack observer names`);
    assert.doesNotMatch(exampleSrc, /\bstk::(?:new|push)<i32>/, `${rel} must rely on NEPLg2.1 Stack expected type or receiver evidence instead of explicit producer or mutator postfixes`);
    assert.doesNotMatch(exampleSrc, /\bstk::pop_top<[^>]+>/, `${rel} must rely on Stack owner type evidence instead of explicit pop_top postfix`);
    assert.doesNotMatch(exampleSrc, /field::get(?:_ref)?\s+&?popped(?:_[ab])?\s+"(?:item|stack)"/, `${rel} must not project StackPop fields directly`);
    assert.match(exampleSrc, /\bstk::stack_pop_item\s+&popped(?:_[ab])?\b/, `${rel} must use StackPop item accessor`);
    assert.match(exampleSrc, /\bstk::stack_pop_stack\s+popped(?:_[ab])?\b/, `${rel} must use StackPop stack accessor`);
}

const pipeCollections = neplCodeBlocks(fs.readFileSync(path.join(repoRoot, 'tests/stdlib/pipe_collections.n.md'), 'utf8'));
const pipeStackSection = fs.readFileSync(path.join(repoRoot, 'tests/stdlib/pipe_collections.n.md'), 'utf8')
    .match(/## pipe_stack_alias_usage[\s\S]*?(?=\n## |$)/);
assert.ok(pipeStackSection, 'pipe_collections must keep a Stack pipe fixture');
assert.doesNotMatch(pipeStackSection[0], /\b(?:new|push)<i32>/, 'pipe Stack fixture must rely on expected type or receiver evidence instead of explicit producer or mutator postfixes');
assert.match(pipeCollections, /\blen\s+&s0\b/, 'pipe_collections stack case must borrow Stack.len');
assert.doesNotMatch(pipeCollections, /\blen\s+s0\b/, 'pipe_collections stack case must not consume Stack.len');

const overloadTests = fs.readFileSync(path.join(repoRoot, 'tests/compiler/overload.n.md'), 'utf8');
assert.doesNotMatch(overloadTests, /\blen_ref<i32>\s+&st\b/, 'overload tests must not use removed Stack.len_ref');
assert.doesNotMatch(overloadTests, /\b(?:len|free)<i32>\s+&?st\b/, 'overload tests must rely on Stack receiver type evidence instead of explicit observer or cleanup postfixes');
assert.doesNotMatch(overloadTests, /\bv::(?:len|free)<i32>\s+&?(?:vec|v|evens|rest)\b/, 'overload tests must rely on Vec receiver type evidence instead of explicit observer or cleanup postfixes');

console.log('stack unsafe unwrap regression passed');

function neplCodeBlocks(markdown) {
    return [...markdown.matchAll(/```neplg2\r?\n([\s\S]*?)```/g)]
        .map((match) => match[1])
        .join('\n');
}
