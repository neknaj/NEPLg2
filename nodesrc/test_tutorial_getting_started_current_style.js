#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { parseFile } = require('./parser.js');

const repoRoot = path.resolve(__dirname, '..');
const tutorialDir = path.join(repoRoot, 'tutorials', 'getting_started');

const expectedFiles = [
    '00_index.n.md',
    '01_hello_world.n.md',
    '02_test_harness.n.md',
    '03_values_and_types.n.md',
    '04_prefix_calls.n.md',
    '05_functions_and_blocks.n.md',
    '06_if_and_match.n.md',
    '07_option.n.md',
    '08_result.n.md',
    '09_validation_project.n.md',
    '10_string_and_text.n.md',
    '11_bytebuf_and_text_io.n.md',
    '12_char_and_ascii.n.md',
    '13_vec_basics.n.md',
    '14_collection_reads.n.md',
    '15_move_and_borrow.n.md',
    '16_drop_and_cleanup.n.md',
    '17_imports_and_modules.n.md',
    '18_generics.n.md',
    '19_traits_and_bounds.n.md',
    '20_namespace_and_methods.n.md',
    '21_project_fizzbuzz.n.md',
    '22_project_parser_small.n.md',
    '23_project_config_validator.n.md',
    '24_project_byte_output.n.md',
    '90_competitive_programming_intro.n.md',
    '91_sort_search_prefixsum.n.md',
    '92_graph_bfs_dp.n.md',
    '95_target_and_wasi_notes.n.md',
    '99_migration_notes.n.md',
];

const removedFiles = [
    '02_numbers_and_variables.n.md',
    '02b_type_conversion_and_textual_conversion.n.md',
    '03_functions.n.md',
    '04_strings_and_stdio.n.md',
    '05_option.n.md',
    '06_result.n.md',
    '07_while_and_block.n.md',
    '08_if_layouts.n.md',
    '09_import_and_structure.n.md',
    '10_project_fizzbuzz.n.md',
    '11_testing_workflow.n.md',
    '12_pure_function_pipeline.n.md',
    '13_type_driven_error_modeling.n.md',
    '14_refactor_with_properties.n.md',
    '15_match_patterns.n.md',
    '16_debug_and_ansi.n.md',
    '17_namespace_and_alias.n.md',
    '18_recursion_and_termination.n.md',
    '19_pipe_operator.n.md',
    '20_generics_basics.n.md',
    '21_trait_bounds_basics.n.md',
    '22_competitive_io_and_arith.n.md',
    '23_competitive_sort_and_search.n.md',
    '24_competitive_dp_basics.n.md',
    '25_competitive_prefixsum_twopointers.n.md',
    '26_competitive_graph_bfs.n.md',
    '27_competitive_algorithms_catalog.n.md',
];

for (const name of expectedFiles) {
    assert.equal(fs.existsSync(path.join(tutorialDir, name)), true, `${name} must exist`);
}

for (const name of removedFiles) {
    assert.equal(fs.existsSync(path.join(tutorialDir, name)), false, `${name} must not remain in getting_started`);
}

const files = fs.readdirSync(tutorialDir)
    .filter((name) => name.endsWith('.n.md'))
    .sort();

assert.deepEqual(files, expectedFiles, 'getting_started chapter list must match the current tutorial plan');

function extractNeplBlocks(text) {
    const blocks = [];
    const re = /```neplg2\r?\n([\s\S]*?)```/g;
    let m;
    while ((m = re.exec(text)) !== null) {
        blocks.push(m[1]);
    }
    return blocks;
}

const forbiddenCodePatterns = [
    ['old spaced impure unit signature', /fn\s+main\s+<\(\)\*>\s+\(\)>\s+\(\)/],
    ['raw allocator in tutorial example', /\balloc_raw\b/],
    ['raw memory pointer in tutorial example', /\bMemPtr\b/],
    ['panic helper unwrap_ok in tutorial example', /\bunwrap_ok\b|\buwok\b/],
    ['panic helper unwrap_err in tutorial example', /\bunwrap_err\b|\buwerr\b/],
    ['direct test panic helper in tutorial example', /\btest_fail\b|\btest_checked\b/],
];

let doctestCount = 0;
for (const name of files) {
    const rel = `tutorials/getting_started/${name}`;
    const filePath = path.join(tutorialDir, name);
    const text = fs.readFileSync(filePath, 'utf8');
    const blocks = extractNeplBlocks(text);
    doctestCount += blocks.length;
    for (const block of blocks) {
        for (const [label, pattern] of forbiddenCodePatterns) {
            assert.doesNotMatch(block, pattern, `${rel} code block must not contain ${label}`);
        }
    }

    const parsed = parseFile(filePath);
    parsed.doctests.forEach((doctest, index) => {
        if (!/#import\s+"std\/test"\s+as\b/.test(doctest.code)) {
            return;
        }

        const label = `${rel} doctest#${index + 1}`;
        assert.deepEqual(
            doctest.tags,
            ['stdio', 'normalize_newlines'],
            `${label} must opt into normalized stdout execution`,
        );
        assert.equal(doctest.ret, null, `${label} must not use ret: as an exit-code substitute`);
        assert.equal(doctest.exit_code, 0, `${label} must pin exit_code: 0`);
        assert.match(
            doctest.stdout || '',
            /^Checked \[[a-z,]+\]\n(?:\[\d+\] [a-z]+\n)+$/,
            `${label} must pin the deterministic std/test stdout report`,
        );
        assert.match(
            doctest.code,
            /checks_print_report[\s\S]*checks_exit_code/,
            `${label} must print the report before deriving the exit code`,
        );
    });
}

assert.ok(doctestCount >= 20, 'current tutorial must keep runnable examples across the main track');

const indexText = fs.readFileSync(path.join(tutorialDir, '00_index.n.md'), 'utf8');
const linkRe = /\[[^\]]+\]\(([^)]+\.n\.md)\)/g;
let linkMatch;
while ((linkMatch = linkRe.exec(indexText)) !== null) {
    const target = linkMatch[1];
    assert.equal(fs.existsSync(path.join(tutorialDir, target)), true, `index link target must exist: ${target}`);
}

const charChapter = fs.readFileSync(path.join(tutorialDir, '12_char_and_ascii.n.md'), 'utf8');
assert.match(charChapter, /let\s+a\s+<char>\s+'A'/, 'char chapter must show char literal syntax');
assert.match(charChapter, /str_char_at_result/, 'char chapter must connect char with string APIs');

const genericsChapter = fs.readFileSync(path.join(tutorialDir, '18_generics.n.md'), 'utf8');
const genericsBlocks = extractNeplBlocks(genericsChapter).join('\n');
assert.match(genericsBlocks, /#import\s+"core\/traits\/copy"\s+as\s+\*/, 'generics chapter must import Copy capability explicitly');
assert.match(genericsBlocks, /fn\s+identity\s+<\.T:\s*Copy>\s+<\(\.T\)->\.T>/, 'generics chapter must show Copy-bound generic values');
assert.match(genericsBlocks, /Option<i32>/, 'generics chapter must keep a generic Option example');
assert.match(genericsBlocks, /Result<i32,str>/, 'generics chapter must keep a generic Result example');
assert.doesNotMatch(genericsBlocks, /fn\s+or_default\s+<\.T>\b/, 'generics chapter must not reintroduce unconstrained owner-generic helper examples');
assert.doesNotMatch(genericsBlocks, /identity\s+"nepl"|check_str_eq\s+"nepl"\s+identity/, 'generics chapter must not demonstrate owner string identity through quiet checks');

console.log('getting_started current tutorial style regression passed');
