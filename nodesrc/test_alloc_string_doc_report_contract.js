#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { parseFile } = require('./parser');

const repoRoot = path.resolve(__dirname, '..');

const contracts = [
    {
        rel: ['stdlib', 'alloc', 'string', 'find.nepl'],
        index: 0,
        name: 'string_find_doc',
        count: 6,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'integer.nepl'],
        index: 0,
        name: 'string_integer_facade_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'float.nepl'],
        index: 0,
        name: 'string_float_facade_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'search', 'byte_find.nepl'],
        index: 0,
        name: 'str_find_doc',
        count: 4,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'search', 'compare.nepl'],
        name: 'str_starts_with_at_doc',
        count: 6,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'integer', 'parse.nepl'],
        index: 0,
        name: 'string_integer_parse_doc',
        count: 6,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'float', 'parse.nepl'],
        index: 0,
        name: 'string_float_parse_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'integer', 'common', 'bool.nepl'],
        index: 0,
        name: 'string_from_bool_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'integer', 'common', 'bool.nepl'],
        index: 1,
        name: 'string_to_bool_doc',
        count: 3,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'builder', 'append.nepl'],
        name: 'sb_append_char_doc',
        count: 1,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'builder', 'append.nepl'],
        name: 'sb_append_ascii_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'builder', 'append.nepl'],
        name: 'sb_append_byte_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'builder', 'append.nepl'],
        name: 'sb_append_doc',
        count: 1,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'builder', 'build.nepl'],
        name: 'sb_build_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'builder', 'reserve.nepl'],
        name: 'string_builder_new_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'builder_ext.nepl'],
        name: 'sb_append_i32_doc',
        count: 1,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'char_offsets.nepl'],
        name: 'alloc_string_char_offsets_module_doc',
        count: 1,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'char_offsets.nepl'],
        name: 'alloc_string_char_offsets_step_byte_doc',
        count: 1,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'char_offsets.nepl'],
        name: 'alloc_string_char_offsets_step_width_doc',
        count: 3,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'char_offsets.nepl'],
        name: 'alloc_string_char_offsets_struct_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'char_offsets.nepl'],
        name: 'alloc_string_char_offsets_new_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'char_offsets.nepl'],
        name: 'alloc_string_char_offsets_result_doc',
        count: 5,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'concat.nepl'],
        name: 'alloc_string_concat_module_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'concat.nepl'],
        name: 'alloc_string_concat_result_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'concat.nepl'],
        name: 'alloc_string_concat_fallback_doc',
        count: 2,
    },
    {
        rel: ['stdlib', 'alloc', 'string', 'concat.nepl'],
        name: 'alloc_string_concat3_doc',
        count: 2,
    },
];

for (const { rel, index, name, count } of contracts) {
    const file = path.join(repoRoot, ...rel);
    const parsed = parseFile(file);
    const matchingDoctests = parsed.doctests.filter((doctest) =>
        new RegExp(`test_report_new "${name}"`).test(doctest.code),
    );
    assert.equal(
        matchingDoctests.length,
        1,
        `${rel.join('/')} must keep exactly one doc-comment doctest for ${name}`,
    );

    const source = fs.readFileSync(file, 'utf8');
    const doctest = matchingDoctests[0];
    assert.equal(doctest.ret, null, `${name} must not use ret as test-success metadata`);
    assert.equal(doctest.exit_code, 0, `${name} must pin exit_code: 0`);
    assert.match(
        doctest.stdout,
        new RegExp(`^test_report name="${name}" count=${count} failed=0\\n`),
        `${name} must pin canonical stdout report`,
    );
    assert.match(doctest.code, new RegExp(`test_report_new "${name}"`), `${name} must construct a named TestReport`);
    assert.match(doctest.code, /test_report_print_stdout report/, `${name} must print the report`);
    assert.match(doctest.code, /test_report_exit_code shown/, `${name} must derive exit code from the shown report`);
    assert.doesNotMatch(source, /\bchecks_exit_code\b/, `${rel.join('/')} must not hide report details behind checks_exit_code`);
    assert.doesNotMatch(source, /\bresult_exit_code\b/, `${rel.join('/')} must not hide report details behind result_exit_code`);
}

for (const [rel, snippets] of [
    [
        ['stdlib', 'alloc', 'string', 'builder', 'append.nepl'],
        [
            '空 builder fallback',
            '入力 builder owner',
            'UTF-8 妥当性はこの helper では検査しません',
            '既存 byte 数に応じた再確保 cost',
            'Result::Err',
        ],
    ],
    [
        ['stdlib', 'alloc', 'string', 'builder', 'build.nepl'],
        [
            '空文字列 fallback',
            '蓄積 byte 列を UTF-8 として検証',
            'sb_build_result',
            'O(total_bytes)',
        ],
    ],
    [
        ['stdlib', 'alloc', 'string', 'builder', 'reserve.nepl'],
        [
            'string_builder_new_result',
            '空 builder fallback',
            'error reason が必要な場合',
            'state access 自体は O(1)',
        ],
    ],
    [
        ['stdlib', 'alloc', 'string', 'builder_ext.nepl'],
        [
            'from_i32_radix v 10',
            '空 builder fallback',
            'O(d) + builder append',
        ],
    ],
    [
        ['stdlib', 'alloc', 'string', 'char_offsets.nepl'],
        [
            '公開 API に漏らしません',
            'Result::Err "string.char invalid slice range"',
            '`start_byte <= end_byte` や範囲内であることの検査は行いません',
            '`0 <= start_char <= end_char <= str_char_count(s)`',
            '`Result` payload を作らずに 1 回の走査',
            'char は UTF-8 scalar value 単位',
            'continuation byte や不正な leading byte から始まる位置は 0 を返します',
        ],
    ],
    [
        ['stdlib', 'alloc', 'string', 'concat.nepl'],
        [
            'root facade から allocation / raw copy の責務を分離します',
            '入力文字列は UTF-8 不変条件を満たす前提',
            'allocation failure や確保前の長さ不正の error reason が必要な処理では `concat_result` を使います',
            '`Result::Err` は `string_alloc_region` の確保失敗または確保前の長さ不正をそのまま返します',
            '途中まで構築された `str` owner は公開しません',
            'error reason を捨てて `""` を返します',
            'fallback は互換層のための仕様',
            '新規コードでは `concat_result` を使ってください',
            '`a + b` の中間文字列を 1 つ確保',
            '`a + b` の内容が 2 回目の連結時に再コピーされます',
            'string_alloc_region',
            'mem_copy',
        ],
    ],
]) {
    const source = fs.readFileSync(path.join(repoRoot, ...rel), 'utf8');
    for (const snippet of snippets) {
        assert.match(
            source,
            new RegExp(snippet.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')),
            `${rel.join('/')} must preserve StringBuilder fallback/owner contract snippet: ${snippet}`,
        );
    }
}

console.log('alloc string doc report contract passed');
