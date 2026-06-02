const { assert } = require('./_shared');

function collectWarnings(result) {
    const diags = Array.isArray(result?.diagnostics) ? result.diagnostics : [];
    return diags.filter((d) => d?.severity === 'warning');
}

module.exports = {
    id: 'overload_shadow_diagnostics',
    async run(api) {
        const overloadSource = `#entry main
#indent 4
#import "core/math" as *

fn val_cast <(i32)->i32> (v):
    v

fn val_cast <(i32)->bool> (v):
    ne v 0

fn main %impure fn void i32 \\void:
    let v <i32> 10
    let res_i32 <i32> val_cast v
    let res_bool <bool> val_cast v
    if:
        res_bool
        then res_i32
        else 0
`;

        const overloadResult = api.analyze_name_resolution(overloadSource);
        assert.equal(!!overloadResult?.ok, true, 'name resolution should succeed for overload source');
        const overloadShadowDiags = Array.isArray(overloadResult?.shadow_diagnostics)
            ? overloadResult.shadow_diagnostics
            : [];
        const overloadShadowWarn = overloadShadowDiags.find(
            (d) =>
                d?.severity === 'warning' &&
                typeof d?.message === 'string' &&
                d.message.includes('redefined') &&
                d.message.includes('shadow')
        );
        assert.equal(
            overloadShadowWarn,
            undefined,
            'pure overload (different signatures) must not emit warning in name-resolution diagnostics'
        );

        const sameSigShadowSource = `#entry main
#indent 4
#target core

fn same <(i32)->i32> (x):
    x

fn same <(i32)->i32> (x):
    x

fn main %fn void i32 \\void:
    same 1
`;

        const sameSigResult = api.analyze_semantics(sameSigShadowSource);
        assert.equal(!!sameSigResult?.ok, true, 'same signature source should compile with warning');
        const sameSigWarnings = collectWarnings(sameSigResult);
        const sameSigShadowWarn = sameSigWarnings.find(
            (d) =>
                typeof d?.message === 'string' &&
                d.message.includes('same signature') &&
                d.message.includes('shadow')
        );
        assert.ok(
            sameSigShadowWarn,
            'same signature redefinition should emit shadow warning'
        );

        return {
            checked: 5,
            overload_shadow_diag_count: overloadShadowDiags.length,
            same_sig_warning_count: sameSigWarnings.length,
        };
    },
};
