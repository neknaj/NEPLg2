const { assert } = require('./_shared');

module.exports = {
    id: 'semantics_tree_types_and_links',
    async run(api) {
        const source = `#entry main
#indent 4
#target core

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
        const tokenHints = Array.isArray(result?.token_hints) ? result.token_hints : [];

        assert.ok(exprs.length > 0, 'expressions should exist');
        assert.ok(tokenSem.length > 0, 'token_semantics should exist');
        assert.ok(tokenRes.length > 0, 'token_resolution should exist');
        assert.ok(tokenHints.length > 0, 'token_hints should exist');

        const typedToken = tokenSem.find((t) => typeof t?.inferred_type === 'string' && t.inferred_type.length > 0);
        assert.ok(typedToken, 'at least one token should have inferred_type');

        const resolvedToken = tokenRes.find((t) => t?.resolved_def_id !== null && t?.resolved_def_id !== undefined);
        assert.ok(resolvedToken, 'at least one token should resolve to a definition');
        assert.ok(
            resolvedToken?.resolved_definition && resolvedToken.resolved_definition.span,
            'resolved_definition with span should be embedded'
        );
        assert.ok(
            Array.isArray(resolvedToken?.candidate_definitions),
            'candidate_definitions should be embedded as array'
        );
        const hintWithTypeAndRef = tokenHints.find(
            (t) =>
                typeof t?.inferred_type === 'string' &&
                t.inferred_type.length > 0 &&
                t?.resolved_def_id !== null &&
                t?.resolved_def_id !== undefined
        );
        assert.ok(hintWithTypeAndRef, 'token_hints should integrate type and resolution');

        const nr = result?.name_resolution;
        assert.ok(nr && Array.isArray(nr.definitions), 'name_resolution payload should be included');

        const shadowSource = `#entry main
#indent 4
#target core

fn main <()->i32> ():
    let outer_value 5;
    let result <i32> block:
        let outer_value 6;
        outer_value
    result
`;
        const shadowResult = api.analyze_semantics(shadowSource);
        assert.equal(!!shadowResult?.ok, true, 'shadowing source should still compile');
        const diagnostics = Array.isArray(shadowResult?.diagnostics) ? shadowResult.diagnostics : [];
        const shadowWarn = diagnostics.find(
            (d) =>
                d?.severity === 'warning' &&
                typeof d?.message === 'string' &&
                d?.code === 'resolve.shadow.outer_definition' &&
                d.message.includes('outer_value')
        );
        assert.ok(shadowWarn, 'outer definition shadow warning should be emitted by typecheck');

        return { checked: 14, expr_count: exprs.length, token_sem_count: tokenSem.length };
    },
};
