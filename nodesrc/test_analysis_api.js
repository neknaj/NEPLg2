#!/usr/bin/env node
// nodesrc/test_analysis_api.js
// 目的:
// - nepl-web の解析 API（lex/parse/name_resolution/semantics）が期待どおりの形で
//   情報を返せることを確認する。

const { candidateDistDirs } = require('./util_paths');
const { loadCompilerFromCandidates } = require('./compiler_loader');

function fail(msg) {
    throw new Error(msg);
}

function findRefByName(result, name) {
    const refs = Array.isArray(result?.references) ? result.references : [];
    return refs.filter(r => r && r.name === name);
}

function getFnDef(parseResult, name) {
    const rootItems = parseResult?.module?.root?.items;
    if (!Array.isArray(rootItems)) return null;
    return rootItems.find((item) => item && item.kind === 'FnDef' && item.name === name) || null;
}

function getStmtExpr(stmt) {
    if (!stmt || typeof stmt !== 'object') return null;
    return stmt.expr && typeof stmt.expr === 'object' ? stmt.expr : null;
}

function sourceRange(source, needle, occurrence = 0) {
    let index = -1;
    let cursor = 0;
    for (let count = 0; count <= occurrence; count += 1) {
        index = source.indexOf(needle, cursor);
        if (index === -1) {
            fail(`missing source text: ${needle}`);
        }
        cursor = index + needle.length;
    }
    return { start: index, end: index + needle.length };
}

function tokenIndexForText(result, source, needle, occurrence = 0) {
    const expected = sourceRange(source, needle, occurrence);
    const tokens = Array.isArray(result?.tokens) ? result.tokens : [];
    const index = tokens.findIndex((token) =>
        Number(token?.span?.start) === expected.start &&
        Number(token?.span?.end) === expected.end
    );
    if (index < 0) {
        fail(`missing token for ${needle} at [${expected.start}, ${expected.end})`);
    }
    return index;
}

function tokenSemanticForIndex(result, tokenIndex) {
    const semantics = Array.isArray(result?.token_semantics) ? result.token_semantics : [];
    return semantics.find((item) => Number(item?.token_index) === tokenIndex) || null;
}

function tokenClassificationForIndex(result, tokenIndex) {
    const classifications = Array.isArray(result?.token_classifications) ? result.token_classifications : [];
    return classifications.find((item) => Number(item?.token_index) === tokenIndex) || null;
}

function firstSymbolDebug(expr) {
    if (!expr || !Array.isArray(expr.items) || expr.items.length === 0) return '';
    const first = expr.items[0];
    return String(first?.kind === 'Symbol' ? (first.debug || '') : '');
}

function assertItemKinds(expr, expectedKinds, label) {
    const items = Array.isArray(expr?.items) ? expr.items : [];
    const actualKinds = items.map((x) => x.kind);
    if (actualKinds.length !== expectedKinds.length) {
        fail(`${label}: expected item count ${expectedKinds.length}, got ${actualKinds.length} (${actualKinds.join(',')})`);
    }
    for (let i = 0; i < expectedKinds.length; i++) {
        if (actualKinds[i] !== expectedKinds[i]) {
            fail(`${label}: expected item[${i}]=${expectedKinds[i]}, got ${actualKinds[i]}`);
        }
    }
}

async function run() {
    const loaded = await loadCompilerFromCandidates(candidateDistDirs(''));
    const api = loaded.api;

    if (typeof api.analyze_lex !== 'function') fail('analyze_lex is missing');
    if (typeof api.analyze_parse !== 'function') fail('analyze_parse is missing');
    if (typeof api.analyze_name_resolution !== 'function') fail('analyze_name_resolution is missing');
    if (typeof api.analyze_semantics !== 'function') fail('analyze_semantics is missing');
    if (typeof api.analyze_semantics_with_vfs !== 'function') fail('analyze_semantics_with_vfs is missing');

    const cases = [
        {
            id: 'semantics_token_type_and_ranges',
            source: `#entry main
#indent 4
#target wasm
fn main <()->i32> ():
    1
`,
            checkSemantics(semResult) {
                const exprs = Array.isArray(semResult?.expressions) ? semResult.expressions : [];
                if (exprs.length === 0) fail('semantics_token_type_and_ranges: expressions missing');
                const lit = exprs.find(e => e && e.kind === 'LiteralI32');
                if (!lit) fail('semantics_token_type_and_ranges: literal expression missing');
                if (!Array.isArray(lit.argument_ranges) || lit.argument_ranges.length !== 0) {
                    fail('semantics_token_type_and_ranges: literal argument ranges should be empty');
                }
                const tokenSem = Array.isArray(semResult?.token_semantics) ? semResult.token_semantics : [];
                if (tokenSem.length === 0) fail('semantics_token_type_and_ranges: token_semantics missing');
                const hasTypedToken = tokenSem.some(t => t && typeof t.inferred_type === 'string' && t.inferred_type.length > 0);
                if (!hasTypedToken) fail('semantics_token_type_and_ranges: inferred_type not found on any token');
            },
        },
        {
            id: 'semantics_prefix_argument_range_aliases',
            source: `#no_prelude
fn id %fn i32 i32 \\x:
    x

fn main %fn void i32 \\void:
    id 41
`,
            checkSemantics(semResult, source) {
                const literalIndex = tokenIndexForText(semResult, source, '41');
                const literalSem = tokenSemanticForIndex(semResult, literalIndex);
                if (!literalSem) fail('semantics_prefix_argument_range_aliases: token semantics missing for 41');
                if (Number(literalSem.arg_index) !== 0) {
                    fail(`semantics_prefix_argument_range_aliases: expected arg_index 0, got ${literalSem.arg_index}`);
                }
                const exprSpan = literalSem.expr_span || literalSem.expression_range;
                const argSpan = literalSem.arg_span || literalSem.arg_range;
                if (!exprSpan || !argSpan) {
                    fail('semantics_prefix_argument_range_aliases: expr_span/arg_span aliases missing');
                }
                const expected = sourceRange(source, '41');
                if (Number(exprSpan.start) !== expected.start || Number(exprSpan.end) !== expected.end) {
                    fail('semantics_prefix_argument_range_aliases: expr_span should cover only the literal');
                }
                if (Number(argSpan.start) !== expected.start || Number(argSpan.end) !== expected.end) {
                    fail('semantics_prefix_argument_range_aliases: arg_span should cover only the first prefix argument');
                }
            },
        },
        {
            id: 'semantics_type_annotation_token_classifications',
            source: `#no_prelude
struct widget_state:
    value %i32

fn id %fn widget_state widget_state \\x:
    x

fn main %fn void widget_state \\void:
    %widget_state id
`,
            checkSemantics(semResult, source) {
                const ranges = Array.isArray(semResult?.syntax_ranges) ? semResult.syntax_ranges : [];
                if (!ranges.some((item) => item?.role === 'prefix_type_annotation')) {
                    fail('semantics_type_annotation_token_classifications: prefix_type_annotation range missing');
                }
                const percentIndex = tokenIndexForText(semResult, source, '%', 3);
                const typeIndex = tokenIndexForText(semResult, source, 'widget_state', 4);
                const percentClass = tokenClassificationForIndex(semResult, percentIndex);
                const typeClass = tokenClassificationForIndex(semResult, typeIndex);
                if (!percentClass || percentClass.category !== 'operator') {
                    fail('semantics_type_annotation_token_classifications: % should be classified as operator by syntax range');
                }
                if (!typeClass || typeClass.category !== 'type') {
                    fail('semantics_type_annotation_token_classifications: lower-case type after % should be classified as type');
                }
                if (typeClass.role !== 'prefix_type_annotation_inner') {
                    fail(`semantics_type_annotation_token_classifications: expected inner role, got ${typeClass.role}`);
                }
            },
        },
        {
            id: 'semantics_current_name_and_literal_classifications',
            source: `#no_prelude
fn keep %fn unit unit \\arg:
    arg

fn main %fn void unit \\void:
    let answer 1;
    keep unit
`,
            checkSemantics(semResult, source) {
                const callIndex = tokenIndexForText(semResult, source, 'keep', 1);
                const constantIndex = tokenIndexForText(semResult, source, 'answer');
                const argIndex = tokenIndexForText(semResult, source, 'arg', 1);
                const signatureVoidIndex = tokenIndexForText(semResult, source, 'void', 0);
                const lambdaVoidIndex = tokenIndexForText(semResult, source, 'void', 1);
                const mainReturnUnitIndex = tokenIndexForText(semResult, source, 'unit', 2);
                const valueUnitIndex = tokenIndexForText(semResult, source, 'unit', 3);
                const callClass = tokenClassificationForIndex(semResult, callIndex);
                const constantClass = tokenClassificationForIndex(semResult, constantIndex);
                const argClass = tokenClassificationForIndex(semResult, argIndex);
                const signatureVoidClass = tokenClassificationForIndex(semResult, signatureVoidIndex);
                const lambdaVoidClass = tokenClassificationForIndex(semResult, lambdaVoidIndex);
                const mainReturnUnitClass = tokenClassificationForIndex(semResult, mainReturnUnitIndex);
                const valueUnitClass = tokenClassificationForIndex(semResult, valueUnitIndex);
                if (!callClass || callClass.category !== 'function') {
                    fail('semantics_current_name_and_literal_classifications: function call should be classified as function');
                }
                if (!constantClass || constantClass.category !== 'constant') {
                    fail('semantics_current_name_and_literal_classifications: let name should be classified as constant');
                }
                if (!argClass || argClass.category !== 'variable') {
                    fail('semantics_current_name_and_literal_classifications: parameter reference should be classified as variable');
                }
                if (!signatureVoidClass || signatureVoidClass.category !== 'literal-void') {
                    fail('semantics_current_name_and_literal_classifications: signature void marker should be literal-void');
                }
                if (!lambdaVoidClass || lambdaVoidClass.category !== 'literal-void') {
                    fail('semantics_current_name_and_literal_classifications: lambda void marker should be literal-void');
                }
                if (!mainReturnUnitClass || mainReturnUnitClass.category !== 'literal-unit') {
                    fail('semantics_current_name_and_literal_classifications: return unit should be classified as literal-unit');
                }
                if (!valueUnitClass || valueUnitClass.category !== 'literal-unit') {
                    fail('semantics_current_name_and_literal_classifications: unit value should be literal-unit');
                }
            },
        },
        {
            id: 'semantics_path_namespace_member_classifications',
            source: `#no_prelude
enum Result:
    Ok %unit

fn main %fn void Result \\void:
    Result::Ok unit
`,
            checkSemantics(semResult, source) {
                const namespaceIndex = tokenIndexForText(semResult, source, 'Result', 2);
                const memberIndex = tokenIndexForText(semResult, source, 'Ok', 1);
                const namespaceClass = tokenClassificationForIndex(semResult, namespaceIndex);
                const memberClass = tokenClassificationForIndex(semResult, memberIndex);
                if (!namespaceClass || namespaceClass.category !== 'namespace') {
                    fail('semantics_path_namespace_member_classifications: path namespace should be classified as namespace');
                }
                if (!memberClass || memberClass.category !== 'constant') {
                    fail('semantics_path_namespace_member_classifications: path member should be classified as constant');
                }
            },
        },
        {
            id: 'shadowing_local_let',
            source: `#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let x 1;
    let x 2;
    add x 3
`,
            check(resolveResult) {
                const defs = Array.isArray(resolveResult.definitions) ? resolveResult.definitions : [];
                const xDefs = defs.filter(d => d.name === 'x');
                if (xDefs.length < 2) fail('expected at least two definitions for x');
                const refs = findRefByName(resolveResult, 'x');
                if (refs.length === 0) fail('expected a reference to x');
                const lastRef = refs[refs.length - 1];
                const newestDefId = xDefs[xDefs.length - 1].id;
                if (lastRef.resolved_def_id !== newestDefId) {
                    fail(`expected x to resolve to newest def ${newestDefId}, got ${lastRef.resolved_def_id}`);
                }
            },
        },
        {
            id: 'fn_alias_target_resolution',
            source: `#entry main
#indent 4
#target wasm
#import "core/math" as *

fn inc <(i32)->i32> (x):
    add x 1

fn plus @inc;

fn main <()->i32> ():
    plus 41
`,
            check(resolveResult) {
                const refs = findRefByName(resolveResult, 'inc');
                if (refs.length === 0) fail('expected alias target reference to inc');
                if (refs[0].resolved_def_id == null) fail('alias target inc should be resolved');
            },
        },
        {
            id: 'parse_if_inline_no_colon_blocks',
            source: `#entry main
#indent 4
#target wasm
fn main <()->i32> ():
    if cond true then 1 else 2
`,
            checkParse(parseResult) {
                const fn = getFnDef(parseResult, 'main');
                if (!fn) fail('parse_if_inline_no_colon_blocks: main not found');
                const bodyItems = Array.isArray(fn?.body?.items) ? fn.body.items : [];
                const ifStmt = bodyItems.find((stmt) => firstSymbolDebug(getStmtExpr(stmt)).startsWith('If('));
                if (!ifStmt) fail('parse_if_inline_no_colon_blocks: if stmt not found');
                const ifExpr = getStmtExpr(ifStmt);
                assertItemKinds(ifExpr, ['Symbol', 'Literal', 'Literal', 'Literal'], 'parse_if_inline_no_colon_blocks');
            },
        },
        {
            id: 'parse_if_colon_uses_block_for_cond_then_else',
            source: `#entry main
#indent 4
#target wasm
fn main <()->i32> ():
    if:
        cond:
            true
        then:
            1
        else:
            2
`,
            checkParse(parseResult) {
                const fn = getFnDef(parseResult, 'main');
                if (!fn) fail('parse_if_colon_uses_block_for_cond_then_else: main not found');
                const bodyItems = Array.isArray(fn?.body?.items) ? fn.body.items : [];
                const ifStmt = bodyItems.find((stmt) => firstSymbolDebug(getStmtExpr(stmt)).startsWith('If('));
                if (!ifStmt) fail('parse_if_colon_uses_block_for_cond_then_else: if stmt not found');
                const ifExpr = getStmtExpr(ifStmt);
                assertItemKinds(ifExpr, ['Symbol', 'Block', 'Block', 'Block'], 'parse_if_colon_uses_block_for_cond_then_else');
            },
        },
        {
            id: 'parse_while_inline_no_colon_blocks',
            source: `#entry main
#indent 4
#target wasm
fn main <()->i32> ():
    let mut i 0;
    while cond lt i 3 do set i add i 1;
    i
`,
            checkParse(parseResult) {
                const fn = getFnDef(parseResult, 'main');
                if (!fn) fail('parse_while_inline_no_colon_blocks: main not found');
                const bodyItems = Array.isArray(fn?.body?.items) ? fn.body.items : [];
                const whileStmt = bodyItems.find((stmt) => firstSymbolDebug(getStmtExpr(stmt)).startsWith('While('));
                if (!whileStmt) fail('parse_while_inline_no_colon_blocks: while stmt not found');
                const whileExpr = getStmtExpr(whileStmt);
                const kinds = Array.isArray(whileExpr?.items) ? whileExpr.items.map((x) => x.kind) : [];
                if (kinds.includes('Block')) {
                    fail(`parse_while_inline_no_colon_blocks: unexpected Block in inline while (${kinds.join(',')})`);
                }
            },
        },
        {
            id: 'parse_while_colon_uses_block_for_cond_do',
            source: `#entry main
#indent 4
#target wasm
fn main <()->i32> ():
    let mut i 0;
    while:
        cond lt i 3
        do:
            set i add i 1;
    i
`,
            checkParse(parseResult) {
                const fn = getFnDef(parseResult, 'main');
                if (!fn) fail('parse_while_colon_uses_block_for_cond_do: main not found');
                const bodyItems = Array.isArray(fn?.body?.items) ? fn.body.items : [];
                const whileStmt = bodyItems.find((stmt) => firstSymbolDebug(getStmtExpr(stmt)).startsWith('While('));
                if (!whileStmt) fail('parse_while_colon_uses_block_for_cond_do: while stmt not found');
                const whileExpr = getStmtExpr(whileStmt);
                assertItemKinds(whileExpr, ['Symbol', 'Block', 'Block'], 'parse_while_colon_uses_block_for_cond_do');
            },
        },
    ];

    const results = [];
    for (const c of cases) {
        const lex = api.analyze_lex(c.source);
        const parse = api.analyze_parse(c.source);
        const resolve = api.analyze_name_resolution(c.source);
        const semantics = api.analyze_semantics(c.source);
        if (!lex || lex.stage !== 'lex') fail(`${c.id}: lex stage mismatch`);
        if (!parse || parse.stage !== 'parse') fail(`${c.id}: parse stage mismatch`);
        if (!resolve || resolve.stage !== 'name_resolution') fail(`${c.id}: resolve stage mismatch`);
        if (!semantics || semantics.stage !== 'semantics') fail(`${c.id}: semantics stage mismatch`);
        if (!resolve.ok) fail(`${c.id}: resolve not ok`);
        if (!Array.isArray(semantics?.diagnostics)) fail(`${c.id}: semantics diagnostics missing`);
        if (typeof c.checkParse === 'function') c.checkParse(parse);
        if (typeof c.check === 'function') c.check(resolve);
        if (typeof c.checkSemantics === 'function') c.checkSemantics(semantics, c.source);
        results.push({ id: c.id, ok: true });
    }

    {
        const id = 'semantics_parse_error_keeps_compiler_token_classifications';
        const source = 'fn main %fn void unit \\void:\n    Result::Ok unit\n    %widget_state\n';
        const semantics = api.analyze_semantics(source);
        if (!semantics || semantics.stage !== 'semantics') fail(`${id}: semantics stage mismatch`);
        if (semantics.ok) fail(`${id}: expected parse/typecheck failure for incomplete source`);
        const fnIndex = tokenIndexForText(semantics, source, 'fn', 0);
        const voidIndex = tokenIndexForText(semantics, source, 'void', 0);
        const returnUnitIndex = tokenIndexForText(semantics, source, 'unit', 0);
        const groupIndex = tokenIndexForText(semantics, source, 'Result', 0);
        const memberIndex = tokenIndexForText(semantics, source, 'Ok', 0);
        const keywordClass = tokenClassificationForIndex(semantics, fnIndex);
        const voidClass = tokenClassificationForIndex(semantics, voidIndex);
        const unitClass = tokenClassificationForIndex(semantics, returnUnitIndex);
        const groupClass = tokenClassificationForIndex(semantics, groupIndex);
        const memberClass = tokenClassificationForIndex(semantics, memberIndex);
        if (!keywordClass || keywordClass.category !== 'keyword') {
            fail(`${id}: keyword classification missing after parse error`);
        }
        if (!voidClass || voidClass.category !== 'literal-void') {
            fail(`${id}: void marker classification missing after parse error`);
        }
        if (!unitClass || unitClass.category !== 'literal-unit') {
            fail(`${id}: unit classification missing after parse error`);
        }
        if (!groupClass || groupClass.category !== 'namespace') {
            fail(`${id}: namespace classification missing after parse error`);
        }
        if (!memberClass || memberClass.category !== 'constant') {
            fail(`${id}: path member classification missing after parse error`);
        }
        results.push({ id, ok: true });
    }

    {
        const id = 'semantics_vfs_loader_error_keeps_compiler_token_classifications';
        const source = `#no_prelude
#import "./missing_file" as modx

fn main %fn void unit \\void:
    modx::call unit
`;
        const semantics = api.analyze_semantics_with_vfs('/workspace/main.nepl', source, {});
        if (!semantics || semantics.stage !== 'semantics') fail(`${id}: semantics stage mismatch`);
        if (semantics.ok) fail(`${id}: expected loader failure for missing import`);
        const fnIndex = tokenIndexForText(semantics, source, 'fn', 0);
        const voidIndex = tokenIndexForText(semantics, source, 'void', 0);
        const unitIndex = tokenIndexForText(semantics, source, 'unit', 0);
        const groupIndex = tokenIndexForText(semantics, source, 'modx', 1);
        const memberIndex = tokenIndexForText(semantics, source, 'call', 0);
        const keywordClass = tokenClassificationForIndex(semantics, fnIndex);
        const voidClass = tokenClassificationForIndex(semantics, voidIndex);
        const unitClass = tokenClassificationForIndex(semantics, unitIndex);
        const groupClass = tokenClassificationForIndex(semantics, groupIndex);
        const memberClass = tokenClassificationForIndex(semantics, memberIndex);
        if (!keywordClass || keywordClass.category !== 'keyword') {
            fail(`${id}: keyword classification missing after loader error`);
        }
        if (!voidClass || voidClass.category !== 'literal-void') {
            fail(`${id}: void marker classification missing after loader error`);
        }
        if (!unitClass || unitClass.category !== 'literal-unit') {
            fail(`${id}: unit classification missing after loader error`);
        }
        if (!groupClass || groupClass.category !== 'namespace') {
            fail(`${id}: namespace classification missing after loader error`);
        }
        if (!memberClass || memberClass.category !== 'constant') {
            fail(`${id}: path member classification missing after loader error`);
        }
        results.push({ id, ok: true });
    }

    const vfsCases = [
        {
            id: 'semantics_vfs_qualified_function_call_keeps_namespace_dim',
            entryPath: '/workspace/main.nepl',
            source: `#no_prelude
#import "./group1" as group1

fn main %fn void unit \\void:
    group1::name unit
`,
            vfs: {
                '/workspace/group1.nepl': `#no_prelude
pub fn name %fn unit unit \\arg:
    arg
`,
            },
            checkSemantics(semResult, source) {
                const groupIndex = tokenIndexForText(semResult, source, 'group1', 2);
                const nameIndex = tokenIndexForText(semResult, source, 'name', 0);
                const groupClass = tokenClassificationForIndex(semResult, groupIndex);
                const nameClass = tokenClassificationForIndex(semResult, nameIndex);
                if (!groupClass || groupClass.category !== 'namespace') {
                    fail('semantics_vfs_qualified_function_call_keeps_namespace_dim: qualified call group should stay namespace');
                }
                if (!nameClass || nameClass.category !== 'function') {
                    fail('semantics_vfs_qualified_function_call_keeps_namespace_dim: qualified call terminal name should be function');
                }
            },
        },
    ];

    for (const c of vfsCases) {
        const semantics = api.analyze_semantics_with_vfs(c.entryPath, c.source, c.vfs);
        if (!semantics || semantics.stage !== 'semantics') fail(`${c.id}: semantics stage mismatch`);
        if (!semantics.ok) {
            const diagnostics = Array.isArray(semantics?.diagnostics) ? semantics.diagnostics : [];
            fail(`${c.id}: semantics not ok ${JSON.stringify(diagnostics)}`);
        }
        if (typeof c.checkSemantics === 'function') c.checkSemantics(semantics, c.source);
        results.push({ id: c.id, ok: true });
    }

    const out = {
        summary: {
            total: results.length,
            passed: results.length,
            failed: 0,
        },
        results,
    };
    console.log(JSON.stringify(out, null, 2));
}

run().catch((e) => {
    console.error(String(e?.stack || e?.message || e));
    process.exit(1);
});
