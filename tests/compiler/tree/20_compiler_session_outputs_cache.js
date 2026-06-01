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

function neplMetaPreTypecheckProbeStats(session) {
    const s = stats(session);
    const keys = [
        'nepl_meta_artifact_store_pre_typecheck_probe_attempts',
        'nepl_meta_artifact_store_pre_typecheck_probe_projected',
        'nepl_meta_artifact_store_pre_typecheck_probe_missing_artifacts',
        'nepl_meta_artifact_store_pre_typecheck_probe_payload_rejects',
        'nepl_meta_artifact_store_pre_typecheck_probe_compatibility_rejects',
        'nepl_meta_artifact_store_pre_typecheck_probe_projection_rejects',
        'nepl_meta_artifact_store_pre_typecheck_probe_projected_entries',
        'nepl_meta_artifact_store_last_pre_typecheck_probe_reject_kind',
        'nepl_meta_artifact_store_last_pre_typecheck_probe_reject_code',
        'nepl_meta_artifact_store_last_pre_typecheck_probe_projection_blocker_reason_code',
        'nepl_meta_artifact_store_last_pre_typecheck_probe_projection_blocker_entry_kind_code',
        'nepl_meta_artifact_store_last_pre_typecheck_probe_projected_entries',
    ];
    for (const key of keys) {
        assert.ok(
            Object.prototype.hasOwnProperty.call(s, key),
            `.neplmeta pre-typecheck probe stats must expose ${key}`,
        );
    }
    return {
        attempts: Number(s.nepl_meta_artifact_store_pre_typecheck_probe_attempts || 0),
        projected: Number(s.nepl_meta_artifact_store_pre_typecheck_probe_projected || 0),
        missing: Number(s.nepl_meta_artifact_store_pre_typecheck_probe_missing_artifacts || 0),
        compatibilityRejects: Number(s.nepl_meta_artifact_store_pre_typecheck_probe_compatibility_rejects || 0),
        projectionRejects: Number(s.nepl_meta_artifact_store_pre_typecheck_probe_projection_rejects || 0),
        rejectKind: Number(s.nepl_meta_artifact_store_last_pre_typecheck_probe_reject_kind || 0),
        rejectCode: Number(s.nepl_meta_artifact_store_last_pre_typecheck_probe_reject_code || 0),
        projectionBlockerReasonCode: Number(s.nepl_meta_artifact_store_last_pre_typecheck_probe_projection_blocker_reason_code || 0),
        projectionBlockerEntryKindCode: Number(s.nepl_meta_artifact_store_last_pre_typecheck_probe_projection_blocker_entry_kind_code || 0),
    };
}

function neplMetaPreTypecheckEdgeProbeStats(session) {
    const s = stats(session);
    const keys = [
        'nepl_meta_artifact_store_pre_typecheck_edge_probe_attempts',
        'nepl_meta_artifact_store_pre_typecheck_edge_probe_projected',
        'nepl_meta_artifact_store_pre_typecheck_edge_probe_missing_artifacts',
        'nepl_meta_artifact_store_pre_typecheck_edge_probe_payload_rejects',
        'nepl_meta_artifact_store_pre_typecheck_edge_probe_compatibility_rejects',
        'nepl_meta_artifact_store_pre_typecheck_edge_probe_projection_rejects',
        'nepl_meta_artifact_store_pre_typecheck_edge_probe_projected_entries',
        'nepl_meta_artifact_store_last_pre_typecheck_edge_probe_reject_kind',
        'nepl_meta_artifact_store_last_pre_typecheck_edge_probe_reject_code',
        'nepl_meta_artifact_store_last_pre_typecheck_edge_probe_projection_blocker_reason_code',
        'nepl_meta_artifact_store_last_pre_typecheck_edge_probe_projection_blocker_entry_kind_code',
        'nepl_meta_artifact_store_last_pre_typecheck_edge_probe_projected_entries',
    ];
    for (const key of keys) {
        assert.ok(
            Object.prototype.hasOwnProperty.call(s, key),
            `.neplmeta pre-typecheck edge probe stats must expose ${key}`,
        );
    }
    return {
        attempts: Number(s.nepl_meta_artifact_store_pre_typecheck_edge_probe_attempts || 0),
        projected: Number(s.nepl_meta_artifact_store_pre_typecheck_edge_probe_projected || 0),
        missing: Number(s.nepl_meta_artifact_store_pre_typecheck_edge_probe_missing_artifacts || 0),
        compatibilityRejects: Number(s.nepl_meta_artifact_store_pre_typecheck_edge_probe_compatibility_rejects || 0),
        projectionRejects: Number(s.nepl_meta_artifact_store_pre_typecheck_edge_probe_projection_rejects || 0),
        rejectKind: Number(s.nepl_meta_artifact_store_last_pre_typecheck_edge_probe_reject_kind || 0),
        rejectCode: Number(s.nepl_meta_artifact_store_last_pre_typecheck_edge_probe_reject_code || 0),
        projectionBlockerReasonCode: Number(s.nepl_meta_artifact_store_last_pre_typecheck_edge_probe_projection_blocker_reason_code || 0),
        projectionBlockerEntryKindCode: Number(s.nepl_meta_artifact_store_last_pre_typecheck_edge_probe_projection_blocker_entry_kind_code || 0),
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
        assert.deepEqual(
            neplMetaPreTypecheckProbeStats(cacheSession),
            { attempts: 0, projected: 0, missing: 0, compatibilityRejects: 0, projectionRejects: 0, rejectKind: 0, rejectCode: 0, projectionBlockerReasonCode: 0, projectionBlockerEntryKindCode: 0 },
            'first compile must expose pre-typecheck probe stats without probing an empty artifact store',
        );
        assert.deepEqual(
            neplMetaPreTypecheckEdgeProbeStats(cacheSession),
            { attempts: 0, projected: 0, missing: 0, compatibilityRejects: 0, projectionRejects: 0, rejectKind: 0, rejectCode: 0, projectionBlockerReasonCode: 0, projectionBlockerEntryKindCode: 0 },
            'first compile must expose edge probe stats without probing an empty artifact store',
        );
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
        assert.ok(
            neplMetaStoreStats(cacheSession).stores > firstMetaStore.stores,
            'changed VFS content compile must refresh the root .neplmeta store and may populate stdlib dependency artifacts',
        );
        assert.notDeepEqual(
            wasmBytes(changedOutput),
            wasmBytes(firstOutput),
            'changed imported VFS content must change the compiled wasm instead of returning stale bytes',
        );
        assert.deepEqual(
            neplMetaPreTypecheckProbeStats(cacheSession),
            { attempts: 1, projected: 0, missing: 0, compatibilityRejects: 0, projectionRejects: 1, rejectKind: 4, rejectCode: 6, projectionBlockerReasonCode: 3, projectionBlockerEntryKindCode: 5 },
            'dependency body-only edit must still report a projection blocker instead of treating a root artifact hit as body-skip ready',
        );
        const dependencyEditEdgeProbe = neplMetaPreTypecheckEdgeProbeStats(cacheSession);
        assert.ok(
            dependencyEditEdgeProbe.attempts > 0,
            'dependency body-only edit must probe stdlib import/prelude edges after the root artifact store is populated',
        );
        assert.equal(
            dependencyEditEdgeProbe.missing,
            dependencyEditEdgeProbe.attempts,
            'edge probe must report missing target artifacts instead of treating dependency edges as reusable',
        );

        const stdlibDependencyArtifactSession = newSession(api);
        const stdlibDependencySourceOne = `#entry main
#import "std/prelude_base" as *
fn main %fn unit i32 \\unit:
    1
`;
        const stdlibDependencySourceTwo = `#entry main
#import "std/prelude_base" as *
fn main %fn unit i32 \\unit:
    2
`;
        const stdlibDependencySourceThree = `#entry main
#import "std/prelude_base" as *
fn main %fn unit i32 \\unit:
    3
`;
        stdlibDependencyArtifactSession.compile_outputs_with_vfs(
            '/virtual/stdlib_dependency_artifact.nepl',
            stdlibDependencySourceOne,
            {},
            ['wasm'],
            false,
        );
        assert.deepEqual(
            neplMetaPreTypecheckEdgeProbeStats(stdlibDependencyArtifactSession),
            { attempts: 0, projected: 0, missing: 0, compatibilityRejects: 0, projectionRejects: 0, rejectKind: 0, rejectCode: 0, projectionBlockerReasonCode: 0, projectionBlockerEntryKindCode: 0 },
            'the initial compile must not expand the empty .neplmeta store into dependency edge probes',
        );
        stdlibDependencyArtifactSession.compile_outputs_with_vfs(
            '/virtual/stdlib_dependency_artifact.nepl',
            stdlibDependencySourceTwo,
            {},
            ['wasm'],
            false,
        );
        const secondStdlibEdgeProbe = neplMetaPreTypecheckEdgeProbeStats(stdlibDependencyArtifactSession);
        const secondStdlibMetaStore = neplMetaStoreStats(stdlibDependencyArtifactSession);
        assert.ok(
            secondStdlibEdgeProbe.attempts > 0,
            'the first real edge probe must observe stdlib dependency artifact candidates',
        );
        assert.equal(
            secondStdlibEdgeProbe.missing,
            secondStdlibEdgeProbe.attempts,
            'before dependency artifacts are produced, every stdlib edge probe must fail by missing artifact',
        );
        assert.ok(
            secondStdlibMetaStore.entries > 1,
            'a successful compile with edge probes must populate .neplmeta artifacts for stdlib dependencies',
        );
        stdlibDependencyArtifactSession.compile_outputs_with_vfs(
            '/virtual/stdlib_dependency_artifact.nepl',
            stdlibDependencySourceThree,
            {},
            ['wasm'],
            false,
        );
        const thirdStdlibEdgeProbe = neplMetaPreTypecheckEdgeProbeStats(stdlibDependencyArtifactSession);
        assert.ok(
            thirdStdlibEdgeProbe.attempts > secondStdlibEdgeProbe.attempts,
            'a later body edit must keep probing the same stdlib dependency edges',
        );
        assert.equal(
            thirdStdlibEdgeProbe.missing,
            secondStdlibEdgeProbe.missing,
            'after dependency artifact production, later edge probes must not add more missing-artifact results',
        );
        assert.ok(
            thirdStdlibEdgeProbe.missing < thirdStdlibEdgeProbe.attempts,
            'stored stdlib dependency artifacts must move later edge probes beyond the missing-artifact boundary',
        );
        assert.ok(
            thirdStdlibEdgeProbe.projectionRejects > 0,
            'stored stdlib dependency artifacts must report explicit projection blockers until materializer support is added',
        );
        assert.equal(
            thirdStdlibEdgeProbe.projectionBlockerReasonCode,
            3,
            'stdlib prelude_base dependency artifacts now pass trait identity and stop at MissingNamedTypeIdentity, which is the next materializer root gap',
        );
        assert.equal(
            thirdStdlibEdgeProbe.projectionBlockerEntryKindCode,
            5,
            'the first stdlib edge blocker is an impl surface, so trait/impl materialization must be the next root fix',
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
        assert.deepEqual(
            neplMetaPreTypecheckProbeStats(sourceKeySession),
            { attempts: 1, projected: 0, missing: 0, compatibilityRejects: 1, projectionRejects: 0, rejectKind: 3, rejectCode: 6, projectionBlockerReasonCode: 0, projectionBlockerEntryKindCode: 0 },
            'root body token edit must reject the previous .neplmeta artifact by source key before typecheck reuse',
        );
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

        const includeSession = newSession(api);
        const includeSource = `#entry main
#no_prelude
#include "./included"
fn main %fn unit i32 \\unit:
    1
`;
        const includedSource = `fn included_value %fn unit i32 \\unit:
    1
`;
        includeSession.compile_outputs_with_vfs(
            '/virtual/include_root.nepl',
            includeSource,
            {
                '/virtual/included.nepl': includedSource,
            },
            ['wasm'],
            false,
        );
        includeSession.compile_outputs_with_vfs(
            '/virtual/include_root.nepl',
            `${includeSource}\n`,
            {
                '/virtual/included.nepl': includedSource,
            },
            ['wasm'],
            false,
        );
        assert.deepEqual(
            neplMetaPreTypecheckEdgeProbeStats(includeSession),
            { attempts: 0, projected: 0, missing: 0, compatibilityRejects: 0, projectionRejects: 0, rejectKind: 0, rejectCode: 0, projectionBlockerReasonCode: 0, projectionBlockerEntryKindCode: 0 },
            '#include must not be treated as a dependency artifact edge',
        );

        return { checked: 13 };
    },
};
