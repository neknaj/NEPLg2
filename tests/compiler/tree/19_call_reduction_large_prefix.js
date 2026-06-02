const { assert } = require('./_shared');

module.exports = {
    id: 'call_reduction_large_prefix_chain',
    async run(api) {
        const callCount = 1105;
        const chain = Array.from({ length: callCount }, () => 'inc').join(' ');
        const source = `#entry main
#indent 4
#target core

fn inc <(i32)->i32> (x):
    x

fn main %fn void i32 \\void:
    ${chain} 0
`;

        const result = api.analyze_semantics(source);
        assert.equal(result?.stage, 'semantics', 'stage must be semantics');
        assert.equal(!!result?.ok, true, 'large prefix chain should typecheck');

        return {
            checked: 2,
            call_count: callCount,
            expression_count: Array.isArray(result?.expressions) ? result.expressions.length : 0,
        };
    },
};
