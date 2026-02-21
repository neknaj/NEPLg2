const { assert } = require('./_shared');

function messages(result) {
    const diags = Array.isArray(result?.diagnostics) ? result.diagnostics : [];
    return diags.map((d) => String(d?.message || ''));
}

function hasReservedKeywordError(msgs, name) {
    return msgs.some(
        (m) => m.includes('reserved keyword') && m.includes(`'${name}'`)
    );
}

module.exports = {
    id: 'parse_tree_reserved_keyword_identifier_diagnostics',
    async run(api) {
        const letCond = `#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let cond 1;
    cond
`;
        const r1 = api.analyze_parse(letCond);
        assert.equal(r1?.stage, 'parse');
        const m1 = messages(r1);
        assert.ok(hasReservedKeywordError(m1, 'cond'), 'let cond should report reserved keyword error');

        const fnLet = `#entry main
#indent 4
#target wasm

fn let <()->i32> ():
    1

fn main <()->i32> ():
    let
`;
        const r2 = api.analyze_parse(fnLet);
        assert.equal(r2?.stage, 'parse');
        const m2 = messages(r2);
        assert.ok(hasReservedKeywordError(m2, 'let'), 'fn let should report reserved keyword error');

        const paramFn = `#entry main
#indent 4
#target wasm

fn id <(i32)->i32> (fn):
    fn

fn main <()->i32> ():
    id 1
`;
        const r3 = api.analyze_parse(paramFn);
        assert.equal(r3?.stage, 'parse');
        const m3 = messages(r3);
        assert.ok(hasReservedKeywordError(m3, 'fn'), 'parameter fn should report reserved keyword error');

        return { checked: 6, d1: m1.length, d2: m2.length, d3: m3.length };
    },
};
