const { assert, findFnDef, firstSymbolDebug, collectExprsFromBlock } = require('./_shared');

module.exports = {
    id: 'parse_tree_if_while_tuple',
    async run(api) {
        const source = `#entry main
#indent 4
#target wasm
fn main <()->i32> ():
    let mut i 0;
    while cond lt i 2 do set i add i 1;
    let t Tuple:
        1
        2
    if cond true then get t 1 else 0
`;

        const result = api.analyze_parse(source);
        assert.equal(result?.stage, 'parse', 'stage must be parse');
        assert.equal(!!result?.ok, true, 'parse should be ok');

        const mainFn = findFnDef(result, 'main');
        assert.ok(mainFn, 'main fn should exist in AST');

        const exprs = collectExprsFromBlock(mainFn?.body);
        assert.ok(exprs.length > 0, 'function body expressions should exist');

        const whileExpr = exprs.find((e) => firstSymbolDebug(e).startsWith('While('));
        assert.ok(whileExpr, 'while expression should exist');

        const ifExpr = exprs.find((e) => firstSymbolDebug(e).startsWith('If('));
        assert.ok(ifExpr, 'if expression should exist');

        const hasTupleItem = exprs.some(
            (e) => Array.isArray(e?.items) && e.items.some((it) => it?.kind === 'Tuple')
        );
        assert.ok(hasTupleItem, 'Tuple item should exist in parse tree');

        return { checked: 6, expr_count: exprs.length };
    },
};
