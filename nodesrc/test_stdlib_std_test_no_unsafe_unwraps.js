#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/std/test.nepl';
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
    /\btest_print_fail\b/,
    /\btest_checked\b/,
    /\bnoshadow\s+test_fail\b/,
    /\bprintln_color\b/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap helpers in implementation code`);
}

const obsoleteVecAccumulator = [
    /#import\s+"alloc\/collections\/vec"\s+as\s+v/,
    /\bVec<Result<\(\),str>>/,
    /\bchecks_empty_vec\b/,
    /\bchecks_has_err_loop\b/,
    /\bchecks_summary_loop\b/,
    /\bchecks_print_human_loop\b/,
    /\bload<Result<\(\),str>>/,
];

for (const pattern of obsoleteVecAccumulator) {
    assert.doesNotMatch(code, pattern, 'std/test must not reintroduce the raw Vec<Result<(),str>> accumulator');
}

assert.doesNotMatch(code, /struct\s+Checks:/, 'std/test must not keep the old Checks type as the primary report model');
assert.match(code, /enum\s+AssertionStatus:[\s\S]*Passed[\s\S]*Failed/, 'std/test must model assertion status as an enum');
assert.match(code, /enum\s+AssertionKind:[\s\S]*Bool[\s\S]*EqI32[\s\S]*NeBool[\s\S]*StrEq[\s\S]*OkI32[\s\S]*ErrI32[\s\S]*Manual/, 'std/test must model assertion kind as an enum');
assert.match(code, /struct\s+TestAssertion:[\s\S]*kind\s+<AssertionKind>[\s\S]*status\s+<AssertionStatus>[\s\S]*label\s+<str>[\s\S]*expected\s+<str>[\s\S]*actual\s+<str>[\s\S]*message\s+<str>/, 'std/test must keep structured assertion fields');
assert.match(code, /struct\s+TestReport:[\s\S]*name\s+<str>[\s\S]*count\s+<i32>[\s\S]*failed_count\s+<i32>[\s\S]*lines\s+<str>/, 'std/test must keep a structured TestReport accumulator');
assert.doesNotMatch(code, /impl\s+Copy\s+for\s+TestAssertion:/, 'TestAssertion must not shallow-copy report strings');
assert.doesNotMatch(code, /impl\s+Copy\s+for\s+TestReport:/, 'TestReport must not shallow-copy report strings');
assert.match(code, /fn\s+test_assertion_release\s+<\(TestAssertion\)->\(\)>[\s\S]*test_consume_str\s+get\s+a\s+"label"[\s\S]*test_consume_str\s+get\s+a\s+"message"/, 'TestAssertion must have an explicit owner terminal');
assert.match(code, /fn\s+test_report_release\s+<\(TestReport\)->\(\)>[\s\S]*test_consume_str\s+get\s+report\s+"name"[\s\S]*test_consume_str\s+get\s+report\s+"legacy_human"/, 'TestReport must have an explicit owner terminal');
assert.match(code, /fn\s+noshadow\s+assert_eq_i32\s+<\(str,i32,i32\)->TestAssertion>/, 'assert_eq_i32 must return a structured TestAssertion with a label');
assert.match(code, /fn\s+test_report_push\s+<\(TestReport,TestAssertion\)\*>TestReport>[\s\S]*match\s+\*get_ref\s+&assertion\s+"status":[\s\S]*AssertionStatus::Failed:[\s\S]*add\s+failed0\s+1[\s\S]*test_assertion_release\s+assertion/, 'test_report_push must observe assertions by reference and consume them once');
assert.match(code, /fn\s+test_report_render\s+<\(&TestReport\)->str>[\s\S]*json::json_quote_string\s+\*get_ref\s+report\s+"name"[\s\S]*concat\s+h5\s+\*get_ref\s+report\s+"lines"/, 'test_report_render must render the canonical stdout report by reference without printing');
assert.match(code, /fn\s+test_report_print_stdout\s+<\(TestReport\)\*>TestReport>[\s\S]*print\s+test_report_render\s+&report/, 'only the explicit report printer should emit stdout');
assert.match(code, /fn\s+test_report_exit_code\s+<\(TestReport\)->i32>[\s\S]*test_report_has_failure\s+&report[\s\S]*test_report_release\s+report[\s\S]*code/, 'test_report_exit_code must convert failed_count to 0/1 and consume the report');
assert.match(code, /fn\s+checks_push\s+<\(TestReport,Result<\(\),str>\)\*>TestReport>/, 'Result-based migration input must be converted through TestReport rather than raw Vec storage');

console.log('stdlib std/test unsafe unwrap regression passed');
