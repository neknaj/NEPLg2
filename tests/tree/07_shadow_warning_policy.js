const { assert } = require('./_shared');

function warningNames(result) {
    const diags = Array.isArray(result?.shadow_diagnostics) ? result.shadow_diagnostics : [];
    return diags
        .filter((d) => d?.severity === 'warning')
        .map((d) => String(d?.name || ''));
}

module.exports = {
    id: 'shadow_warning_policy',
    async run(api) {
        const warnSource = `#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let print 1;
    add print 2
`;
        const warnResult = api.analyze_name_resolution(warnSource);
        assert.equal(!!warnResult?.ok, true, 'name resolution should succeed for warnSource');
        const warned = warningNames(warnResult);
        assert.ok(
            warned.includes('print'),
            "shadowing important stdlib-like symbol 'print' must emit warning"
        );

        const noWarnSource = `#entry main
#indent 4
#target wasm
#import "core/cast" as *

fn cast_i32 <(i32)->i32> (x):
    cast x

fn main <()->i32> ():
    let cast 10;
    add cast 1
`;
        const noWarnResult = api.analyze_name_resolution(noWarnSource);
        assert.equal(!!noWarnResult?.ok, true, 'name resolution should succeed for noWarnSource');
        const noWarned = warningNames(noWarnResult);
        assert.ok(
            !noWarned.includes('cast'),
            "shadowing non-important symbol 'cast' must not emit important-shadow warning"
        );

        if (typeof api.analyze_name_resolution_with_options === 'function') {
            const suppressedResult = api.analyze_name_resolution_with_options(
                warnSource,
                { warn_important_shadow: false }
            );
            assert.equal(
                !!suppressedResult?.ok,
                true,
                'name resolution should succeed for suppressed warnSource'
            );
            const suppressedWarnings = warningNames(suppressedResult);
            assert.ok(
                !suppressedWarnings.includes('print'),
                "important shadow warning for 'print' must be suppressible by option"
            );
        }

        return {
            checked: 5,
            warn_count: warned.length,
            no_warn_count: noWarned.length,
        };
    },
};
