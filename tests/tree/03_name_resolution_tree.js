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

        return { checked: 6, def_count: defs.length, ref_count: refs.length };
    },
};
