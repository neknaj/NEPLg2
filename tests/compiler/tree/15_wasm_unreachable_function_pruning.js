const { assert } = require('./_shared');

module.exports = {
    id: 'wasm_unreachable_function_pruning',
    async run(api) {
        const source = `#entry main
#indent 4
#target core
#import "core/math" as *

fn dead %fn void i32 \\void:
    add 1 2

fn live %fn void i32 \\void:
    add 3 4

fn main %fn void i32 \\void:
    live
`;

        const out = api.compile_outputs(source, ['wat'], false);
        const wat = String(out?.wat || '');
        assert.ok(wat.length > 0, 'wat should be generated');
        assert.equal(wat.includes('live__void__i32__pure'), true, 'reachable function should be emitted');
        assert.equal(wat.includes('dead__void__i32__pure'), false, 'unreachable function should be pruned');

        return {
            checked: 3,
            wat_length: wat.length,
            has_live: wat.includes('live__void__i32__pure'),
            has_dead: wat.includes('dead__void__i32__pure'),
        };
    },
};

