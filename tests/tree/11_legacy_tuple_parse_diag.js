const { assert } = require('./_shared');

function collectMessages(parseResult) {
    const diags = Array.isArray(parseResult?.diagnostics) ? parseResult.diagnostics : [];
    return diags.map((d) => String(d?.message || ''));
}

module.exports = {
    id: 'parse_tree_legacy_tuple_diagnostics',
    async run(api) {
        const legacyTupleLiteral = `#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let t (1, true)
    0
`;

        const literalRes = api.analyze_parse(legacyTupleLiteral);
        assert.equal(literalRes?.stage, 'parse');
        const literalMessages = collectMessages(literalRes);
        assert.ok(
            literalMessages.some((m) => m.includes("legacy tuple literal '(...)' is removed")),
            'must report legacy tuple literal removal diagnostic'
        );

        const legacyTupleType = `#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let t <(i32,i32)> Tuple:
        1
        2
    0
`;

        const typeRes = api.analyze_parse(legacyTupleType);
        assert.equal(typeRes?.stage, 'parse');
        const typeMessages = collectMessages(typeRes);
        assert.ok(
            typeMessages.some((m) => m.includes("legacy tuple type '(T1, T2, ...)' is removed")),
            'must report legacy tuple type removal diagnostic'
        );

        return {
            checked: 6,
            literal_diag_count: literalMessages.length,
            type_diag_count: typeMessages.length,
        };
    },
};
