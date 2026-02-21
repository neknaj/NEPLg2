const { assert } = require('./_shared');

module.exports = {
    id: 'name_resolution_shadowing',
    async run(api) {
        const source = `#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let x 1;
    let x 2;
    let add 10;
    add x 3
`;

        const result = api.analyze_name_resolution(source);
        assert.equal(result?.stage, 'name_resolution', 'stage must be name_resolution');
        assert.equal(!!result?.ok, true, 'name resolution should be ok');

        const defs = Array.isArray(result?.definitions) ? result.definitions : [];
        const refs = Array.isArray(result?.references) ? result.references : [];
        const xDefs = defs.filter((d) => d?.name === 'x');
        const xRefs = refs.filter((r) => r?.name === 'x');

        assert.ok(xDefs.length >= 2, 'shadowing requires at least two x definitions');
        assert.ok(xRefs.length >= 1, 'x reference should exist');

        const newestDefId = xDefs[xDefs.length - 1].id;
        const targetRef = xRefs[xRefs.length - 1];
        assert.equal(
            targetRef?.resolved_def_id,
            newestDefId,
            'x reference must resolve to nearest (newest) definition'
        );

        assert.ok(
            Array.isArray(targetRef?.candidate_def_ids),
            'candidate_def_ids should be available for debug/LSP'
        );
        assert.ok(
            targetRef?.resolved_def && typeof targetRef.resolved_def === 'object',
            'resolved_def object should be provided for LSP jump metadata'
        );
        assert.equal(
            targetRef?.resolved_def?.id,
            targetRef?.resolved_def_id,
            'resolved_def.id must match resolved_def_id'
        );
        assert.ok(
            Array.isArray(targetRef?.candidate_definitions),
            'candidate_definitions should be provided with detailed metadata'
        );
        assert.ok(
            targetRef.candidate_definitions.length >= targetRef.candidate_def_ids.length,
            'candidate_definitions should include each candidate id'
        );

        const shadows = Array.isArray(result?.shadows) ? result.shadows : [];
        const shadowDiags = Array.isArray(result?.shadow_diagnostics)
            ? result.shadow_diagnostics
            : [];
        assert.ok(shadows.length > 0, 'shadow events should be provided');

        const xShadow = shadows.find(
            (s) =>
                s?.name === 'x' &&
                s?.event_kind === 'definition_shadow' &&
                Array.isArray(s?.shadowed_def_ids) &&
                s.shadowed_def_ids.length > 0
        );
        assert.ok(xShadow, 'x shadow event should include hidden candidates');

        const importantWarn = shadowDiags.find(
            (s) =>
                s?.name === 'add' &&
                s?.severity === 'warning'
        );
        assert.ok(importantWarn, 'important stdlib-like symbol shadow warning should be present');

        return {
            checked: 14,
            def_count: defs.length,
            ref_count: refs.length,
            shadow_count: shadows.length,
        };
    },
};
