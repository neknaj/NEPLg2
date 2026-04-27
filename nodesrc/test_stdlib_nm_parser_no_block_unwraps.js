#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/nm/parser.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

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

const parseStart = code.indexOf('pub fn parse_markdown <(str)->Document> (input):');
const parseEnd = code.indexOf('fn json_escape <(str)->str> (s):', parseStart);
assert.notEqual(parseStart, -1, 'parse_markdown must exist');
assert.notEqual(parseEnd, -1, 'parse_markdown section boundary must exist');
const parseMarkdown = code.slice(parseStart, parseEnd);

assert.match(code, /struct\s+NodePushRes:[\s\S]*items\s+<Vec<Node>>[\s\S]*ok\s+<bool>/, 'Node push result must carry Vec and status');
assert.match(code, /fn\s+nm_node_empty_vec\s+<\(\)->Vec<Node>>\s+\(\):\s+v::Vec<Node>\s+0\s+0\s+mem_ptr_wrap\s+0/, 'block allocation failure must use an empty Node Vec sentinel');
assert.match(code, /fn\s+nm_push_node\s+<\(Vec<Node>, Node\)->NodePushRes>\s+\(items,\s*item\):[\s\S]*match\s+v::push<Node>\s+items\s+item:[\s\S]*Result::Err\s+_e:[\s\S]*NodePushRes\s+nm_node_empty_vec\s+false/, 'Node pushes must convert grow failure to ok=false');
assert.match(code, /fn\s+nest_stack_push_from_hdr_result\s+<\(MemPtr<u8>,NestSection\)->Result<\(\),str>>\s+\(hdr,\s*item\):[\s\S]*match\s+realloc_ptr<NestSection>[\s\S]*Result::Err\s+_e:[\s\S]*Result<\(\),str>::Err\s+"nm\.parser nest stack grow failed"/, 'nest section stack grow must return Result instead of trapping');
assert.match(parseMarkdown, /match\s+v::new<Node>:[\s\S]*Result::Err\s+_e:[\s\S]*set\s+failed\s+true/, 'parse_markdown must handle root Vec allocation failure');
assert.match(parseMarkdown, /let\s+pushed_root\s+<NodePushRes>\s+nm_push_node\s+root\s+Node::Hr/, 'horizontal rule accumulation must go through checked Node push');
assert.match(parseMarkdown, /let\s+pushed_kids\s+<NodePushRes>\s+nm_push_node\s+kids\s+Node::Paragraph\s+pr_inlines/, 'section child accumulation must go through checked Node push');
assert.match(parseMarkdown, /while\s+and\s+lt\s+i\s+nlines\s+not\s+failed:/, 'parse_markdown must stop scanning after allocation failure');

console.log('stdlib nm parser block unsafe unwrap regression passed');
