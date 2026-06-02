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
#target core

fn main %fn void i32 \\void:
    let outer_value 1;
    let result <i32> block:
        let outer_value 2;
        outer_value
    result
`;
        const warnResult = api.analyze_name_resolution(warnSource);
        assert.equal(!!warnResult?.ok, true, 'name resolution should succeed for warnSource');
        const warned = warningNames(warnResult);
        assert.ok(
            warned.includes('outer_value'),
            "shadowing an actual outer symbol 'outer_value' must emit warning"
        );

        const noWarnSource = `#entry main
#indent 4
#target core

fn main %fn void i32 \\void:
    let local_value 10;
    local_value
`;
        const noWarnResult = api.analyze_name_resolution(noWarnSource);
        assert.equal(!!noWarnResult?.ok, true, 'name resolution should succeed for noWarnSource');
        const noWarned = warningNames(noWarnResult);
        assert.ok(
            !noWarned.includes('local_value'),
            "defining a fresh local symbol must not emit shadow warning"
        );

        if (typeof api.analyze_name_resolution_with_options === 'function') {
            const suppressedResult = api.analyze_name_resolution_with_options(
                warnSource,
                { warn_shadow: false }
            );
            assert.equal(
                !!suppressedResult?.ok,
                true,
                'name resolution should succeed for suppressed warnSource'
            );
            const suppressedWarnings = warningNames(suppressedResult);
            assert.ok(
                !suppressedWarnings.includes('outer_value'),
                "shadow warning for 'outer_value' must be suppressible by option"
            );
        }

        return {
            checked: 5,
            warn_count: warned.length,
            no_warn_count: noWarned.length,
        };
    },
};
