const { assert } = require('./_shared');

module.exports = {
    id: 'parse_tree_profile_directive_shape',
    async run(api) {
        const source = `#entry main
#if[profile=debug]
fn only_debug <()->i32> ():
    1
fn main <()->i32> ():
    only_debug
`;

        const result = api.analyze_parse(source);
        assert.equal(result?.stage, 'parse', 'stage must be parse');
        assert.equal(!!result?.ok, true, 'parse should succeed');

        const items = result?.module?.root?.items;
        assert.ok(Array.isArray(items), 'root items should exist');
        assert.ok(items.length >= 4, 'expected at least entry/ifprofile/fn/fn');

        assert.equal(items[0]?.kind, 'Directive', 'first item should be directive');
        assert.equal(items[0]?.name, 'Entry', 'first directive should be Entry');

        assert.equal(items[1]?.kind, 'Directive', 'second item should be directive');
        assert.equal(items[1]?.name, 'IfProfile', 'second directive should be IfProfile');
        assert.ok(
            typeof items[1]?.debug === 'string' && items[1].debug.includes('profile: "debug"'),
            'IfProfile debug payload should include profile=debug'
        );

        assert.equal(items[2]?.kind, 'FnDef', 'third item should be gated function');
        assert.equal(items[2]?.name, 'only_debug', 'gated function name should match');

        assert.equal(items[3]?.kind, 'FnDef', 'fourth item should be main function');
        assert.equal(items[3]?.name, 'main', 'main function should exist');

        return { checked: 10, item_count: items.length };
    },
};
