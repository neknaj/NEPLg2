#!/usr/bin/env node
// nodesrc/analyze_tests_json.js
// 目的:
// - nodesrc/tests.js が出力した JSON を読み取り、失敗/エラーの主因をカテゴリ別に集計する。
//
// 使い方:
//   node nodesrc/analyze_tests_json.js /tmp/tests-only.json

const fs = require('node:fs');
const path = require('node:path');

function parseArgs(argv) {
    if (argv.length < 1 || argv.includes('-h') || argv.includes('--help')) {
        return { help: true, input: '' };
    }
    return { help: false, input: argv[0] };
}

function stripAnsi(s) {
    return String(s || '').replace(/\x1b\[[0-9;]*m/g, '');
}

function classifyReason(errorText) {
    const s = stripAnsi(errorText);
    if (s.includes('expected compile_fail, but compiled successfully')) {
        return 'compile_fail_expectation_mismatch';
    }
    if (s.includes('expression left extra values on the stack')) {
        return 'stack_extra_values';
    }
    if (s.includes('return type does not match signature')) {
        return 'return_type_mismatch';
    }
    if (s.includes('entry function is missing or ambiguous')) {
        return 'entry_missing_or_ambiguous';
    }
    if (s.includes('parenthesized expressions are not supported')) {
        return 'old_parenthesized_expression_syntax';
    }
    if (s.includes('unexpected token in expression')) {
        return 'unexpected_token';
    }
    if (s.includes('expected Indent, found')) {
        return 'indent_expected';
    }
    return 'other';
}

function inc(map, key) {
    map.set(key, (map.get(key) || 0) + 1);
}

function topNEntries(map, n) {
    return Array.from(map.entries())
        .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
        .slice(0, n)
        .map(([name, count]) => ({ name, count }));
}

function main() {
    const { help, input } = parseArgs(process.argv.slice(2));
    if (help || !input) {
        console.log('Usage: node nodesrc/analyze_tests_json.js <tests-result.json>');
        process.exit(help ? 0 : 2);
    }

    const abs = path.resolve(input);
    const raw = fs.readFileSync(abs, 'utf-8');
    const j = JSON.parse(raw);
    const results = Array.isArray(j.results) ? j.results : [];

    const statusCounts = new Map();
    const reasonCounts = new Map();

    for (const r of results) {
        const status = String(r.status || 'unknown');
        inc(statusCounts, status);
        if (status === 'fail' || status === 'error') {
            inc(reasonCounts, classifyReason(r.error || ''));
        }
    }

    const out = {
        input: abs,
        summary: j.summary || null,
        by_status: topNEntries(statusCounts, 20),
        fail_error_reasons: topNEntries(reasonCounts, 50),
    };

    console.log(JSON.stringify(out, null, 2));
}

if (require.main === module) {
    try {
        main();
    } catch (e) {
        console.error(String(e?.stack || e?.message || e));
        process.exit(1);
    }
}
