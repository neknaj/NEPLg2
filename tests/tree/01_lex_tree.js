const { assert } = require('./_shared');

module.exports = {
    id: 'lex_tree_keywords',
    async run(api) {
        const source = `#entry main
#indent 4
#target wasm
fn main <()->i32> ():
    if cond true then 1 else 2
`;

        const result = api.analyze_lex(source);
        assert.equal(result?.stage, 'lex', 'stage must be lex');
        assert.equal(!!result?.ok, true, 'lex should be ok');

        const tokens = Array.isArray(result?.tokens) ? result.tokens : [];
        const kinds = tokens.map((t) => t?.kind);

        assert.ok(kinds.includes('KwIf'), 'KwIf token is required');
        assert.ok(kinds.includes('KwCond'), 'KwCond token is required');
        assert.ok(kinds.includes('KwThen'), 'KwThen token is required');
        assert.ok(kinds.includes('KwElse'), 'KwElse token is required');

        const badIdent = tokens.find(
            (t) =>
                t?.kind === 'Ident' &&
                (t?.value === 'cond' || t?.value === 'then' || t?.value === 'else' || t?.value === 'do')
        );
        assert.equal(badIdent, undefined, 'layout keywords must not be tokenized as Ident');

        return { checked: 5, token_count: tokens.length };
    },
};
