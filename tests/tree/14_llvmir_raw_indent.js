const { assert, findFnDef } = require('./_shared');

module.exports = {
    id: 'llvmir_raw_indent_parse_tree',
    async run(api) {
        const source = `#indent 4
#target llvm

#llvmir:
    ; module level
      ; deeper comment
    define i32 @m() {
    entry:
      ret i32 0
    }

fn f <()->i32> ():
    #llvmir:
        define i32 @f() {
        entry:
          ret i32 7
        }
`;

        const result = api.analyze_parse(source);
        assert.equal(result?.stage, 'parse', 'stage must be parse');
        assert.equal(!!result?.ok, true, 'parse should be ok for raw llvmir indentation');

        const items = result?.module?.root?.items || [];
        const topLlvm = items.find((it) => it?.kind === 'LlvmIr');
        assert.ok(topLlvm, 'top-level llvmir stmt should exist');
        assert.ok(String(topLlvm?.debug || '').includes('; deeper comment'), 'raw text should preserve deeper indentation line');

        const fdef = findFnDef(result, 'f');
        assert.ok(fdef, 'fn f should exist');
        const bodyDebug = String(fdef?.body || '');
        assert.ok(bodyDebug.includes('LlvmIr'), 'function body should be llvmir body');
        assert.ok(bodyDebug.includes('ret i32 7'), 'llvmir function body text should be preserved');

        return { checked: 7 };
    },
};
