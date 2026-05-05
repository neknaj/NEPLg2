const { assert, path } = require('./_shared');
const { loadStdlibVfsFromFs } = require('../../../nodesrc/stdlib_vfs_cache');

function compileWithStdlibProfile(api, source, profile) {
    if (typeof api.compile_source_with_vfs_stdlib_and_profile !== 'function') {
        throw new Error('compile_source_with_vfs_stdlib_and_profile API is not available');
    }
    const stdlibVfs = loadStdlibVfsFromFs(path.resolve(process.cwd(), 'stdlib'), { missing: 'empty' });
    return api.compile_source_with_vfs_stdlib_and_profile(
        '/tests/compiler/tree/stdio_release_debug_noop.nepl',
        source,
        {},
        stdlibVfs,
        profile
    );
}

module.exports = {
    id: 'stdio_release_debug_noop_symbols',
    async run(api) {
        const source = `#entry main
#indent 4
#target std

#import "std/stdio" as *

fn main <()*>i32> ():
    debug "hidden"
    debug_color AnsiStyle::Red "hidden"
    debugln "hidden"
    debugln_color AnsiStyle::Red "hidden"
    0
`;

        const wasm = compileWithStdlibProfile(api, source, 'release');
        assert.ok(wasm instanceof Uint8Array, 'release profile must expose debug no-op stdio symbols');

        return { checked: 1 };
    },
};
