#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/kp/kpgraph.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

const code = legacyTypeSyntaxView(src);
const lines = code.split(/\r?\n/);

function extractTopLevelFunction(name) {
    const start = lines.findIndex((line) => new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\b`).test(line));
    assert.notEqual(start, -1, `${name} must exist`);

    const rest = lines.slice(start + 1);
    const next = rest.findIndex((line) => /^(?:pub\s+)?(?:fn|struct|enum)\s+/.test(line));
    const end = next === -1 ? lines.length : start + 1 + next;
    return lines.slice(start, end).join('\n');
}

const bfs = extractTopLevelFunction('dense_graph_bfs_dist');

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

assert.match(code, /#import\s+"alloc\/collections\/vec"\s+as\s+v/, 'kpgraph must qualify implementation Vec allocation calls');
assert.doesNotMatch(code, /\b(?:KpI32PushRes|kp_i32_empty_vec|kp_push_i32|dense_graph_bfs_dist_raw)\b/, 'kpgraph must not keep legacy raw-BFS fallback helpers');
assert.match(code, /pub\s+fn\s+dense_graph_bfs_dist\s+<\(&DenseGraph,i32\)\*>Result<Vec<i32>,\s*Diag>>/, 'BFS must expose the typed DenseGraph owner API');
assert.doesNotMatch(bfs, /\bv::push<i32>\b/, 'BFS must not accumulate results through growable push failure sentinels');
assert.match(bfs, /match\s+v::filled<i32>\s+n\s+unvisited:[\s\S]*Result::Err\s+e:[\s\S]*let\s+d\s+<Diag>\s+dense_graph_diag_std_error\s+e[\s\S]*err\s+d/, 'BFS distance allocation failure must be reported as Result Err');
assert.match(bfs, /match\s+v::filled<i32>\s+n\s+0:[\s\S]*Result::Err\s+e:[\s\S]*v::free<i32>\s+dist[\s\S]*let\s+d\s+<Diag>\s+dense_graph_diag_std_error\s+e[\s\S]*err\s+d/, 'BFS queue allocation failure must free the distance Vec owner');
assert.match(bfs, /v::replace<i32>\s+&dist\s+start\s+0[\s\S]*v::replace<i32>\s+&queue\s+0\s+start/, 'BFS must initialize preallocated distance and queue Vec storage through Vec APIs');
assert.match(bfs, /match\s+v::get<i32>\s+&queue\s+head:[\s\S]*Option::None:[\s\S]*set\s+failed\s+true/, 'BFS queue reads must detect impossible Vec access failure');
assert.match(bfs, /match\s+dense_graph_has_edge\s+g\s+node\s+to:[\s\S]*Result::Err\s+_d:[\s\S]*set\s+failed\s+true/, 'BFS graph access failure must mark the traversal as failed');
assert.match(bfs, /v::free<i32>\s+queue[\s\S]*if:[\s\S]*failed[\s\S]*then:[\s\S]*v::free<i32>\s+dist[\s\S]*diag_err\s+dense_graph_diag_storage[\s\S]*else:[\s\S]*ok\s+dist/, 'BFS must close queue storage and either return dist owner or free it on invariant failure');

console.log('stdlib kpgraph unsafe unwrap regression passed');
