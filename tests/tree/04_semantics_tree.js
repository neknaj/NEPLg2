const { assert } = require('./_shared');

module.exports = {
    id: 'semantics_tree_types_and_links',
    async run(api) {
        const source = `#entry main
#indent 4
#target wasm

fn inc <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let a inc 41;
    a
`;

        const result = api.analyze_semantics(source);
        assert.equal(result?.stage, 'semantics', 'stage must be semantics');
        assert.equal(!!result?.ok, true, 'semantics should be ok');

        const exprs = Array.isArray(result?.expressions) ? result.expressions : [];
        const tokenSem = Array.isArray(result?.token_semantics) ? result.token_semantics : [];
        const tokenRes = Array.isArray(result?.token_resolution) ? result.token_resolution : [];

        assert.ok(exprs.length > 0, 'expressions should exist');
        assert.ok(tokenSem.length > 0, 'token_semantics should exist');
        assert.ok(tokenRes.length > 0, 'token_resolution should exist');

        const typedToken = tokenSem.find((t) => typeof t?.inferred_type === 'string' && t.inferred_type.length > 0);
        assert.ok(typedToken, 'at least one token should have inferred_type');

        const resolvedToken = tokenRes.find((t) => t?.resolved_def_id !== null && t?.resolved_def_id !== undefined);
        assert.ok(resolvedToken, 'at least one token should resolve to a definition');

        const nr = result?.name_resolution;
        assert.ok(nr && Array.isArray(nr.definitions), 'name_resolution payload should be included');

        return { checked: 7, expr_count: exprs.length, token_sem_count: tokenSem.length };
    },
};
