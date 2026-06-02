const { assert } = require('./_shared');

function messages(result) {
    const diags = Array.isArray(result?.diagnostics) ? result.diagnostics : [];
    return diags.map((d) => String(d?.message || ''));
}

function countReserved(msgs, kw) {
    return msgs.filter((m) => m.includes('reserved keyword') && m.includes(`'${kw}'`)).length;
}

module.exports = {
    id: 'parse_tree_multi_error_recovery_reserved_keywords',
    async run(api) {
        const source = `#entry main
#indent 4
#target core

fn main %fn void i32 \\void:
    let cond 1;
    let then 2;
    let else 3;
    0
`;

        const result = api.analyze_parse(source);
        assert.equal(result?.stage, 'parse');

        const msgs = messages(result);
        const cCond = countReserved(msgs, 'cond');
        const cThen = countReserved(msgs, 'then');
        const cElse = countReserved(msgs, 'else');

        assert.ok(cCond >= 1, 'must diagnose reserved cond');
        assert.ok(cThen >= 1, 'must diagnose reserved then');
        assert.ok(cElse >= 1, 'must diagnose reserved else');

        return { checked: 5, diagnostics: msgs.length, cond: cCond, then: cThen, else: cElse };
    },
};
