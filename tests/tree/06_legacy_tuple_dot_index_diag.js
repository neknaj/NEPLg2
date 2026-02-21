const { assert } = require('./_shared');

module.exports = {
    id: 'legacy_tuple_dot_index_diagnostic',
    async run(api) {
        const source = `#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let t Tuple:
        1
        2
    t.0
`;

        const result = api.analyze_semantics(source);
        assert.equal(!!result?.ok, false, 'legacy dot-index must be rejected');

        const diagnostics = Array.isArray(result?.diagnostics) ? result.diagnostics : [];
        const legacyDiag = diagnostics.find(
            (d) =>
                d?.severity === 'error' &&
                typeof d?.message === 'string' &&
                d.message.includes("legacy tuple field access '.") &&
                d.message.includes("use 'get <tuple>")
        );
        assert.ok(legacyDiag, 'migration diagnostic for legacy tuple dot-index should be emitted');

        return {
            checked: 2,
            diagnostic_count: diagnostics.length,
        };
    },
};

