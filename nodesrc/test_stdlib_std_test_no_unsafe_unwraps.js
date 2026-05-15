#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { implementationLineCount } = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const modulePaths = {
    root: 'stdlib/std/test.nepl',
    types: 'stdlib/std/test/types.nepl',
    assertion: 'stdlib/std/test/assertion.nepl',
    report: 'stdlib/std/test/report.nepl',
};

function readModule(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
}

function stripComments(src) {
    return src
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}

const rootSrc = readModule(modulePaths.root);
const typesSrc = readModule(modulePaths.types);
const assertionSrc = readModule(modulePaths.assertion);
const reportSrc = readModule(modulePaths.report);

const rootCode = stripComments(rootSrc);
const typesCode = stripComments(typesSrc);
const assertionCode = stripComments(assertionSrc);
const reportCode = stripComments(reportSrc);
const code = [rootCode, typesCode, assertionCode, reportCode].join('\n');

const lineLimits = [
    [modulePaths.root, rootSrc, 80],
    [modulePaths.types, typesSrc, 220],
    [modulePaths.assertion, assertionSrc, 320],
    [modulePaths.report, reportSrc, 300],
];

for (const [relPath, src, maxLines] of lineLimits) {
    const lineCount = implementationLineCount(src);
    assert.ok(lineCount <= maxLines, `${relPath} must stay within its responsibility boundary (${lineCount}/${maxLines})`);
}

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
    assert.doesNotMatch(code, pattern, 'std/test modules must not use unsafe unwrap helpers in implementation code');
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

assert.match(rootCode, /pub\s+#import\s+"\.\/test\/types"\s+as\s+@merge/, 'std/test root must re-export test types through the facade export surface');
assert.match(rootCode, /pub\s+#import\s+"\.\/test\/assertion"\s+as\s+@merge/, 'std/test root must re-export assertion helpers through the facade export surface');
assert.match(rootCode, /pub\s+#import\s+"\.\/test\/report"\s+as\s+@merge/, 'std/test root must re-export report helpers through the facade export surface');
assert.doesNotMatch(rootCode, /\b(enum|struct|fn)\b/, 'std/test root must remain a facade without implementation bodies');

assert.match(typesCode, /enum\s+AssertionStatus:[\s\S]*Passed[\s\S]*Failed/, 'std/test/types must model assertion status as an enum');
assert.match(typesCode, /enum\s+AssertionKind:[\s\S]*Bool[\s\S]*EqI32[\s\S]*NeBool[\s\S]*StrEq[\s\S]*OkI32[\s\S]*ErrI32[\s\S]*Manual/, 'std/test/types must model assertion kind as an enum');
assert.match(typesCode, /struct\s+TestAssertion:[\s\S]*kind\s+<AssertionKind>[\s\S]*status\s+<AssertionStatus>[\s\S]*label\s+<str>[\s\S]*expected\s+<str>[\s\S]*actual\s+<str>[\s\S]*message\s+<str>/, 'std/test/types must keep structured assertion fields');
assert.match(typesCode, /struct\s+TestReport:[\s\S]*name\s+<str>[\s\S]*count\s+<i32>[\s\S]*failed_count\s+<i32>[\s\S]*lines\s+<str>/, 'std/test/types must keep a structured TestReport accumulator');
assert.doesNotMatch(typesCode, /impl\s+Copy\s+for\s+TestAssertion:/, 'TestAssertion must not shallow-copy report strings');
assert.doesNotMatch(typesCode, /impl\s+Copy\s+for\s+TestReport:/, 'TestReport must not shallow-copy report strings');
assert.match(typesCode, /fn\s+test_assertion_release\s+<\(TestAssertion\)->\(\)>[\s\S]*test_consume_str\s+get\s+a\s+"label"[\s\S]*test_consume_str\s+get\s+a\s+"message"/, 'TestAssertion must have an explicit owner terminal');
assert.match(typesCode, /fn\s+test_report_release\s+<\(TestReport\)->\(\)>[\s\S]*test_consume_str\s+get\s+report\s+"name"[\s\S]*test_consume_str\s+get\s+report\s+"legacy_human"/, 'TestReport must have an explicit owner terminal');

assert.match(assertionCode, /#import\s+"std\/test\/types"\s+as\s+\*/, 'std/test/assertion must depend on typed assertion data');
assert.match(assertionCode, /fn\s+noshadow\s+assert_eq_i32\s+<\(str,i32,i32\)->TestAssertion>/, 'assert_eq_i32 must return a structured TestAssertion with a label');

assert.match(reportCode, /#import\s+"std\/test\/types"\s+as\s+\*/, 'std/test/report must use typed report data');
assert.match(reportCode, /#import\s+"std\/test\/assertion"\s+as\s+\*/, 'std/test/report must convert migration Results through assertion helpers');
assert.match(reportCode, /fn\s+test_report_push\s+<\(TestReport,TestAssertion\)\*>TestReport>[\s\S]*match\s+\*get_ref\s+&assertion\s+"status":[\s\S]*AssertionStatus::Failed:[\s\S]*add\s+failed0\s+1[\s\S]*test_assertion_release\s+assertion/, 'test_report_push must observe assertions by reference and consume them once');
assert.match(reportCode, /fn\s+test_report_render\s+<\(&TestReport\)->str>[\s\S]*json::json_quote_string\s+\*get_ref\s+report\s+"name"[\s\S]*concat\s+h5\s+\*get_ref\s+report\s+"lines"/, 'test_report_render must render the canonical stdout report by reference without printing');
assert.match(reportCode, /fn\s+test_report_print_stdout\s+<\(TestReport\)\*>TestReport>[\s\S]*print\s+test_report_render\s+&report/, 'only the explicit report printer should emit stdout');
assert.match(reportCode, /fn\s+test_report_exit_code\s+<\(TestReport\)->i32>[\s\S]*test_report_has_failure\s+&report[\s\S]*test_report_release\s+report[\s\S]*code/, 'test_report_exit_code must convert failed_count to 0/1 and consume the report');
assert.match(reportCode, /fn\s+checks_push\s+<\(TestReport,Result<\(\),str>\)\*>TestReport>/, 'Result-based migration input must be converted through TestReport rather than raw Vec storage');

console.log('stdlib std/test unsafe unwrap regression passed');
