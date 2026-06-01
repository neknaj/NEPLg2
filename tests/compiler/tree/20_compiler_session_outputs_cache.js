const { assert } = require('./_shared');

function newSession(api) {
    if (typeof api.CompilerSession !== 'function') {
        throw new Error('CompilerSession API is not available');
    }
    return new api.CompilerSession();
}

function stats(session) {
    return JSON.parse(session.loader_cache_stats_json());
}

function cacheHits(session) {
    return Number(stats(session).compiled_output_cache_hits || 0);
}

function cacheStores(session) {
    return Number(stats(session).compiled_output_cache_stores || 0);
}

function neplMetaStoreStats(session) {
    const s = stats(session);
    return {
        entries: Number(s.nepl_meta_artifact_store_entries || 0),
        stores: Number(s.nepl_meta_artifact_store_stores || 0),
        rejects: Number(s.nepl_meta_artifact_store_rejects || 0),
    };
}

function neplMetaArtifactStats(session) {
    const s = stats(session);
    return {
        sourceKeyHash: String(s.nepl_meta_artifact_source_key_hash || 0),
        typedPublicSignatureHash: String(s.nepl_meta_artifact_typed_public_signature_hash || 0),
    };
}

function wasmBytes(outputs) {
    assert.ok(outputs && outputs.wasm instanceof Uint8Array, 'compile output must include wasm bytes');
    return Array.from(outputs.wasm);
}

module.exports = {
    id: 'compiler_session_outputs_cache_profile_and_vfs_key',
    async run(api) {
        const profileSession = newSession(api);
        const defaultDebugSource = `#entry main
#if[profile=release]
fn release_bad <()->i32> ():
    unknown_symbol

fn main <()->i32> ():
    0
`;
        const defaultDebugOutput = profileSession.compile_outputs_with_vfs(
            '/virtual/default_profile.nepl',
            defaultDebugSource,
            {},
            ['wasm'],
            false,
        );
        assert.ok(
            defaultDebugOutput.wasm instanceof Uint8Array,
            'compile_outputs_with_vfs must default source profile to debug even when the Rust artifact is release-built',
        );

        const cacheSession = newSession(api);
        const entrySource = `#entry main
#import "./dep" as *
fn main <()->i32> ():
    dep_value
`;
        const depOne = `pub fn dep_value <()->i32> ():
    1
`;
        const depTwo = `pub fn dep_value <()->i32> ():
    2
`;

        const firstOutput = cacheSession.compile_outputs_with_vfs(
            '/virtual/session_cache.nepl',
            entrySource,
            {
                '/virtual/dep.nepl': depOne,
                '/virtual/unused.nepl': '',
            },
            ['wasm'],
            false,
        );
        const firstStores = cacheStores(cacheSession);
        const firstMetaStore = neplMetaStoreStats(cacheSession);
        assert.equal(firstStores, 1, 'first compile must store one compiled output cache entry');
        assert.equal(firstMetaStore.entries, 1, 'first compile must store one .neplmeta artifact');
        assert.equal(firstMetaStore.stores, 1, 'first compile must count one .neplmeta store');
        assert.equal(firstMetaStore.rejects, 0, 'valid .neplmeta artifact must not be rejected');
        assert.notEqual(
            neplMetaArtifactStats(cacheSession).sourceKeyHash,
            '0',
            '.neplmeta stats must expose a non-zero source key hash for normal source-backed artifacts',
        );

        const orderOnlyOutput = cacheSession.compile_outputs_with_vfs(
            '/virtual/session_cache.nepl',
            entrySource,
            {
                '/virtual/unused.nepl': '',
                '/virtual/dep.nepl': depOne,
            },
            ['wasm'],
            false,
        );
        assert.equal(
            cacheHits(cacheSession),
            1,
            'VFS object key order must not prevent compiled-output cache reuse',
        );
        assert.equal(
            neplMetaStoreStats(cacheSession).stores,
            firstMetaStore.stores,
            'compiled-output cache hit must not count as a fresh .neplmeta store',
        );
        assert.deepEqual(
            wasmBytes(orderOnlyOutput),
            wasmBytes(firstOutput),
            'same source and same VFS content must return the same wasm bytes',
        );

        const changedOutput = cacheSession.compile_outputs_with_vfs(
            '/virtual/session_cache.nepl',
            entrySource,
            {
                '/virtual/dep.nepl': depTwo,
                '/virtual/unused.nepl': '',
            },
            ['wasm'],
            false,
        );
        assert.equal(
            cacheHits(cacheSession),
            1,
            'changed imported VFS content must not be treated as a compiled-output cache hit',
        );
        assert.equal(cacheStores(cacheSession), firstStores + 1, 'changed VFS content must store a new output');
        assert.equal(
            neplMetaStoreStats(cacheSession).stores,
            firstMetaStore.stores + 1,
            'changed VFS content compile must refresh the .neplmeta store',
        );
        assert.notDeepEqual(
            wasmBytes(changedOutput),
            wasmBytes(firstOutput),
            'changed imported VFS content must change the compiled wasm instead of returning stale bytes',
        );

        const stdlibOverlaySession = newSession(api);
        const stdlibOverlaySource = `#entry main
#import "std/prelude_base" as *
fn main %fn unit i32 \\unit:
    0
`;
        stdlibOverlaySession.compile_outputs_with_vfs(
            '/virtual/stdlib_overlay.nepl',
            stdlibOverlaySource,
            {
                '/stdlib/std/prelude_base.nepl': '',
            },
            ['wasm'],
            false,
        );
        assert.equal(
            neplMetaStoreStats(stdlibOverlaySession).entries,
            0,
            'stdlib overlay compile must not store .neplmeta artifact for normal bundled stdlib reuse',
        );
        assert.equal(
            neplMetaStoreStats(stdlibOverlaySession).stores,
            0,
            'stdlib overlay compile must not count a .neplmeta store',
        );

        const sourceKeySession = newSession(api);
        const sourceKeyOne = `#entry main
fn main <()->i32> ():
    1
`;
        const sourceKeyTwo = `#entry main
fn main <()->i32> ():
    2
`;
        sourceKeySession.compile_outputs_with_vfs(
            '/virtual/source_key.nepl',
            sourceKeyOne,
            {},
            ['wasm'],
            false,
        );
        const firstArtifactStats = neplMetaArtifactStats(sourceKeySession);
        sourceKeySession.compile_outputs_with_vfs(
            '/virtual/source_key.nepl',
            sourceKeyTwo,
            {},
            ['wasm'],
            false,
        );
        const secondArtifactStats = neplMetaArtifactStats(sourceKeySession);
        assert.equal(
            firstArtifactStats.typedPublicSignatureHash,
            secondArtifactStats.typedPublicSignatureHash,
            'body-only token edit must keep the same typed public signature hash',
        );
        assert.notEqual(
            firstArtifactStats.sourceKeyHash,
            secondArtifactStats.sourceKeyHash,
            'body-only token edit must change .neplmeta source key hash',
        );

        return { checked: 6 };
    },
};
