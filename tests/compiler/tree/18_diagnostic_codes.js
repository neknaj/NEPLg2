const { assert } = require('./_shared');

module.exports = {
    id: 'diagnostic_codes_for_target_loader_parser_typecheck',
    async run(api) {
        const badTarget = `#entry main
#indent 4
#target wasi2

fn main <()->i32> ():
    0
`;
        const sem = api.analyze_semantics(badTarget);
        const diagnostics = Array.isArray(sem?.diagnostics) ? sem.diagnostics : [];
        const unknownTarget = diagnostics.find(
            (d) =>
                d?.severity === 'error' &&
                d?.message === 'unknown target in #target' &&
                d?.code === 'loader.target.unknown'
        );
        assert.ok(unknownTarget, 'unknown target diagnostic should include loader.target.unknown');
        assert.equal(
            unknownTarget?.code_message,
            'unknown target in #target',
            'code_message should resolve from diagnostic table'
        );

        const missing = api.analyze_name_resolution_with_vfs(
            '/virtual/missing.nepl',
            '#entry main\n#indent 4\n#target core\n#import \"missing/module\" as *\nfn main <()->i32> (): 0\n',
            {},
            { warn_important_shadow: true }
        );
        assert.equal(!!missing?.ok, false, 'missing module should fail');
        const missingDs = Array.isArray(missing?.diagnostics) ? missing.diagnostics : [];
        const loaderDiag = missingDs.find((d) => d?.code === 'loader.source.failure');
        assert.ok(loaderDiag, 'loader diagnostic should include loader.source.failure');

        const parseBad = `#entry main
#indent 4
#target core

fn main <()->i32> ():
    let
`;
        const parseRes = api.analyze_semantics(parseBad);
        const parseDs = Array.isArray(parseRes?.diagnostics) ? parseRes.diagnostics : [];
        assert.ok(
            parseDs.some((d) => d?.code === 'parser.identifier.expected'),
            'parse diagnostics should include parser.identifier.expected'
        );

        const undefVar = `#entry main
#indent 4
#target core
fn main <()->i32> ():
    unknown_symbol
`;
        const undefRes = api.analyze_semantics(undefVar);
        const undefDs = Array.isArray(undefRes?.diagnostics) ? undefRes.diagnostics : [];
        assert.ok(
            undefDs.some((d) => d?.code === 'resolve.identifier.undefined'),
            'typecheck diagnostics should include resolve.identifier.undefined'
        );

        const overloadAmb = `#entry main
#indent 4
#target core

fn cast <(i32)->i32> (x): x
fn cast <(i32)->f32> (x): <f32> cast x
fn main <()->i32> ():
    let y cast 1
    0
`;
        const overloadRes = api.analyze_semantics(overloadAmb);
        const overloadDs = Array.isArray(overloadRes?.diagnostics) ? overloadRes.diagnostics : [];
        assert.ok(
            overloadDs.some((d) => d?.code === 'type.overload.ambiguous'),
            'overload diagnostics should include type.overload.ambiguous'
        );

        const lexBad = `#entry main
#indent xx
#target core
fn main <()->i32> ():
    $
`;
        const lexRes = api.analyze_lex(lexBad);
        const lexDs = Array.isArray(lexRes?.diagnostics) ? lexRes.diagnostics : [];
        assert.ok(
            lexDs.some((d) => d?.code === 'lexer.indent.argument_invalid'),
            'lexer diagnostics should include lexer.indent.argument_invalid'
        );
        assert.ok(
            lexDs.some((d) => d?.code === 'lexer.token.unknown'),
            'lexer diagnostics should include lexer.token.unknown'
        );

        return {
            checked: 10,
            diagnostics_count:
                diagnostics.length +
                missingDs.length +
                parseDs.length +
                undefDs.length +
                overloadDs.length +
                lexDs.length,
        };
    },
};
