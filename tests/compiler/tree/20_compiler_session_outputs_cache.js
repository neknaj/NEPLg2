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

function neplObjDirectCallFragmentStoreStats(session) {
    const s = stats(session);
    const keys = [
        'nepl_obj_direct_call_fragment_store_entries',
        'nepl_obj_direct_call_fragment_store_stores',
        'nepl_obj_direct_call_fragment_store_duplicate_stores',
        'nepl_obj_direct_call_fragment_store_rejects',
        'nepl_obj_direct_call_fragment_store_lookup_attempts',
        'nepl_obj_direct_call_fragment_store_lookup_hits',
        'nepl_obj_direct_call_fragment_store_lookup_misses',
        'nepl_obj_direct_call_fragment_store_lookup_missing_source_policy',
        'nepl_obj_direct_call_fragment_store_lookup_context_rejects',
        'nepl_obj_direct_call_fragment_store_lookup_fragments_returned',
    ];
    for (const key of keys) {
        assert.ok(
            Object.prototype.hasOwnProperty.call(s, key),
            `.neplobj direct-call fragment store stats must expose ${key}`,
        );
    }
    return {
        entries: Number(s.nepl_obj_direct_call_fragment_store_entries || 0),
        stores: Number(s.nepl_obj_direct_call_fragment_store_stores || 0),
        duplicateStores: Number(s.nepl_obj_direct_call_fragment_store_duplicate_stores || 0),
        rejects: Number(s.nepl_obj_direct_call_fragment_store_rejects || 0),
        lookupAttempts: Number(s.nepl_obj_direct_call_fragment_store_lookup_attempts || 0),
        lookupHits: Number(s.nepl_obj_direct_call_fragment_store_lookup_hits || 0),
        lookupMisses: Number(s.nepl_obj_direct_call_fragment_store_lookup_misses || 0),
        lookupMissingSourcePolicy: Number(s.nepl_obj_direct_call_fragment_store_lookup_missing_source_policy || 0),
        lookupContextRejects: Number(s.nepl_obj_direct_call_fragment_store_lookup_context_rejects || 0),
        lookupFragmentsReturned: Number(s.nepl_obj_direct_call_fragment_store_lookup_fragments_returned || 0),
    };
}

function neplMetaMaterializedCompileStats(session) {
    const s = stats(session);
    const keys = [
        'nepl_meta_materialized_compile_attempts',
        'nepl_meta_materialized_compile_attempted_surfaces',
        'nepl_meta_materialized_compile_accepts',
        'nepl_meta_materialized_compile_source_fallbacks',
        'nepl_meta_materialized_compile_source_fallback_successes',
        'nepl_meta_materialized_compile_source_fallback_failures',
        'nepl_meta_materialized_compile_body_missing_fallbacks',
        'nepl_obj_candidate_body_missing_surfaces',
        'nepl_meta_body_missing_skip_entries',
        'nepl_meta_body_missing_skip_hits',
        'nepl_meta_body_missing_skip_stores',
        'nepl_meta_body_missing_skip_stale_entries',
        'nepl_meta_body_missing_skip_last_hits',
        'nepl_meta_body_missing_skip_last_stores',
        'nepl_meta_materialized_compile_last_outcome_code',
        'nepl_meta_materialized_compile_last_fallback_reason_code',
        'nepl_meta_materialized_compile_last_fallback_diagnostic_code',
        'nepl_meta_materialized_compile_last_attempted_surfaces',
        'nepl_obj_candidate_last_body_missing_surfaces',
    ];
    for (const key of keys) {
        assert.ok(
            Object.prototype.hasOwnProperty.call(s, key),
            `.neplmeta materialized compile stats must expose ${key}`,
        );
    }
    return {
        attempts: Number(s.nepl_meta_materialized_compile_attempts || 0),
        attemptedSurfaces: Number(s.nepl_meta_materialized_compile_attempted_surfaces || 0),
        accepts: Number(s.nepl_meta_materialized_compile_accepts || 0),
        fallbacks: Number(s.nepl_meta_materialized_compile_source_fallbacks || 0),
        fallbackSuccesses: Number(s.nepl_meta_materialized_compile_source_fallback_successes || 0),
        fallbackFailures: Number(s.nepl_meta_materialized_compile_source_fallback_failures || 0),
        bodyMissingFallbacks: Number(s.nepl_meta_materialized_compile_body_missing_fallbacks || 0),
        bodyMissingCandidateSurfaces: Number(s.nepl_obj_candidate_body_missing_surfaces || 0),
        bodyMissingSkipEntries: Number(s.nepl_meta_body_missing_skip_entries || 0),
        bodyMissingSkipHits: Number(s.nepl_meta_body_missing_skip_hits || 0),
        bodyMissingSkipStores: Number(s.nepl_meta_body_missing_skip_stores || 0),
        bodyMissingSkipStaleEntries: Number(s.nepl_meta_body_missing_skip_stale_entries || 0),
        lastBodyMissingSkipHits: Number(s.nepl_meta_body_missing_skip_last_hits || 0),
        lastBodyMissingSkipStores: Number(s.nepl_meta_body_missing_skip_last_stores || 0),
        lastOutcomeCode: Number(s.nepl_meta_materialized_compile_last_outcome_code || 0),
        lastFallbackReasonCode: Number(s.nepl_meta_materialized_compile_last_fallback_reason_code || 0),
        lastFallbackDiagnosticCode: String(s.nepl_meta_materialized_compile_last_fallback_diagnostic_code || ''),
        lastAttemptedSurfaces: Number(s.nepl_meta_materialized_compile_last_attempted_surfaces || 0),
        lastBodyMissingCandidateSurfaces: Number(s.nepl_obj_candidate_last_body_missing_surfaces || 0),
    };
}

const MATERIALIZED_FUNCTION_BODY_MISSING_REASON_CODE = 1;

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
        assert.deepEqual(
            neplMetaMaterializedCompileStats(cacheSession),
            {
                attempts: 0,
                attemptedSurfaces: 0,
                accepts: 0,
                fallbacks: 0,
                fallbackSuccesses: 0,
                fallbackFailures: 0,
                bodyMissingFallbacks: 0,
                bodyMissingCandidateSurfaces: 0,
                bodyMissingSkipEntries: 0,
                bodyMissingSkipHits: 0,
                bodyMissingSkipStores: 0,
                bodyMissingSkipStaleEntries: 0,
                lastOutcomeCode: 0,
                lastFallbackReasonCode: 0,
                lastFallbackDiagnosticCode: '',
                lastAttemptedSurfaces: 0,
                lastBodyMissingCandidateSurfaces: 0,
                lastBodyMissingSkipHits: 0,
                lastBodyMissingSkipStores: 0,
            },
            'first compile must expose materialized compile stats without attempting body skip',
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
            neplMetaMaterializedCompileStats(cacheSession),
            {
                attempts: 0,
                attemptedSurfaces: 0,
                accepts: 0,
                fallbacks: 0,
                fallbackSuccesses: 0,
                fallbackFailures: 0,
                bodyMissingFallbacks: 0,
                bodyMissingCandidateSurfaces: 0,
                bodyMissingSkipEntries: 0,
                bodyMissingSkipHits: 0,
                bodyMissingSkipStores: 0,
                bodyMissingSkipStaleEntries: 0,
                lastOutcomeCode: 0,
                lastFallbackReasonCode: 0,
                lastFallbackDiagnosticCode: '',
                lastAttemptedSurfaces: 0,
                lastBodyMissingCandidateSurfaces: 0,
                lastBodyMissingSkipHits: 0,
                lastBodyMissingSkipStores: 0,
            },
            'compiled-output cache hit must not reuse the previous materialized compile observation as a fresh attempt',
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
            { attempts: 1, projected: 1, missing: 0, compatibilityRejects: 0, projectionRejects: 0, rejectKind: 0, rejectCode: 0, projectionBlockerReasonCode: 0, projectionBlockerEntryKindCode: 0 },
            'dependency body-only edit may project the root artifact surface but must still compile against changed dependency content',
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
        const stdlibDependencySourceFour = `#entry main
#import "std/prelude_base" as *
fn main %fn unit i32 \\unit:
    4
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
            'stored stdlib dependency artifacts must report explicit projection rejects until materializer support is added',
        );
        assert.ok(
            thirdStdlibEdgeProbe.projected > 0,
            'backend scalar and local int128 capability cleanup must let at least one stored stdlib artifact project successfully',
        );
        assert.equal(
            thirdStdlibEdgeProbe.rejectCode,
            0,
            'non-callable export projection support lets the last stdlib edge probe end in a successful projection',
        );
        assert.equal(
            thirdStdlibEdgeProbe.projectionBlockerReasonCode,
            0,
            'successful projection must not report stale public surface blocker details',
        );
        assert.equal(
            thirdStdlibEdgeProbe.projectionBlockerEntryKindCode,
            0,
            'successful projection must not report stale public surface blocker entry details',
        );
        const thirdMaterializedCompile = neplMetaMaterializedCompileStats(stdlibDependencyArtifactSession);
        assert.ok(
            thirdMaterializedCompile.attempts > 0,
            'projection success must be counted separately from materialized compile attempts',
        );
        assert.ok(
            thirdMaterializedCompile.accepts + thirdMaterializedCompile.fallbacks > 0,
            'metadata-only dependency body skip must either accept or report a typed source fallback',
        );
        assert.ok(
            thirdMaterializedCompile.lastAttemptedSurfaces > 0,
            'last materialized compile attempt must report how many surfaces entered typecheck',
        );
        if (thirdMaterializedCompile.fallbacks > 0) {
            assert.equal(
                thirdMaterializedCompile.fallbackSuccesses,
                thirdMaterializedCompile.fallbacks,
                'source fallback after materialized compile attempt must preserve successful compile behavior',
            );
            assert.equal(
                thirdMaterializedCompile.fallbackFailures,
                0,
                'materialized attempt fallback must not hide a failing source compile in this regression',
            );
            assert.equal(
                thirdMaterializedCompile.lastOutcomeCode,
                2,
                'last materialized compile fallback must end in source fallback success',
            );
            assert.ok(
                thirdMaterializedCompile.lastFallbackReasonCode > 0,
                'source fallback must expose a typed reason code instead of relying on error text parsing',
            );
            assert.match(
                thirdMaterializedCompile.lastFallbackDiagnosticCode,
                /^[a-z0-9_.]+$/,
                'source fallback must expose the primary compiler diagnostic code that caused fallback',
            );
            if (
                thirdMaterializedCompile.lastFallbackReasonCode
                === MATERIALIZED_FUNCTION_BODY_MISSING_REASON_CODE
            ) {
                assert.ok(
                    thirdMaterializedCompile.bodyMissingCandidateSurfaces
                        >= thirdMaterializedCompile.lastAttemptedSurfaces,
                    '.neplobj candidate stats must count body-missing materialized surfaces, not only compile attempts',
                );
                assert.equal(
                    thirdMaterializedCompile.lastBodyMissingCandidateSurfaces,
                    thirdMaterializedCompile.lastAttemptedSurfaces,
                    'last .neplobj candidate surface count must match the body-missing materialized compile attempt',
                );
            } else {
                assert.equal(
                    thirdMaterializedCompile.lastBodyMissingCandidateSurfaces,
                    0,
                    'non-body-missing materialized fallback must not be counted as a .neplobj body candidate',
                );
            }
        } else {
            assert.ok(
                thirdMaterializedCompile.accepts > 0,
                'a metadata-only compile with no selected dependency body should be accepted',
            );
            assert.equal(
                thirdMaterializedCompile.lastOutcomeCode,
                1,
                'last materialized compile attempt without selected dependency body should be accepted',
            );
            assert.equal(
                thirdMaterializedCompile.lastFallbackReasonCode,
                0,
                'accepted materialized compile must not retain a stale fallback reason',
            );
            assert.equal(
                thirdMaterializedCompile.lastFallbackDiagnosticCode,
                '',
                'accepted materialized compile must not retain a stale fallback diagnostic code',
            );
            assert.equal(
                thirdMaterializedCompile.lastBodyMissingCandidateSurfaces,
                0,
                'accepted materialized compile must not be counted as a .neplobj body candidate',
            );
        }
        const beforeFourthMaterializedCompile = neplMetaMaterializedCompileStats(stdlibDependencyArtifactSession);
        if (beforeFourthMaterializedCompile.bodyMissingSkipStores > 0) {
            stdlibDependencyArtifactSession.compile_outputs_with_vfs(
                '/virtual/stdlib_dependency_artifact.nepl',
                stdlibDependencySourceFour,
                {},
                ['wasm'],
                false,
            );
            const fourthMaterializedCompile = neplMetaMaterializedCompileStats(stdlibDependencyArtifactSession);
            assert.ok(
                fourthMaterializedCompile.bodyMissingSkipHits > beforeFourthMaterializedCompile.bodyMissingSkipHits,
                'a repeated body edit must skip known body-missing .neplmeta edges until .neplobj can satisfy them',
            );
            assert.equal(
                fourthMaterializedCompile.attempts,
                beforeFourthMaterializedCompile.attempts,
                'body-missing skip cache must avoid a repeated materialized compile attempt for the same dependency body boundary',
            );
            assert.equal(
                fourthMaterializedCompile.fallbacks,
                beforeFourthMaterializedCompile.fallbacks,
                'body-missing skip cache must avoid a repeated source fallback caused only by missing .neplobj body',
            );
            assert.ok(
                fourthMaterializedCompile.bodyMissingSkipEntries > 0,
                'body-missing skip cache must keep source-hash scoped entries for later .neplobj replacement',
            );
            assert.equal(
                fourthMaterializedCompile.lastOutcomeCode,
                0,
                'a skipped materialized attempt must reset last attempt outcome instead of replaying stale fallback state',
            );
        }

        const neplObjStoreSession = newSession(api);
        function charDependencyBodySource(value) {
            return `#entry main
#import "core/char" as *
fn main %fn unit i32 \\unit:
    char_utf8_cont_byte ${Number(value) | 0}
`;
        }
        neplObjStoreSession.compile_outputs_with_vfs(
            '/virtual/neplobj_char_dependency.nepl',
            charDependencyBodySource(1),
            {},
            ['wasm'],
            false,
        );
        assert.deepEqual(
            neplObjDirectCallFragmentStoreStats(neplObjStoreSession),
            {
                entries: 0,
                stores: 0,
                duplicateStores: 0,
                rejects: 0,
                lookupAttempts: 0,
                lookupHits: 0,
                lookupMisses: 0,
                lookupMissingSourcePolicy: 0,
                lookupContextRejects: 0,
                lookupFragmentsReturned: 0,
            },
            'initial compile must expose an empty .neplobj direct-call fragment store',
        );
        neplObjStoreSession.compile_outputs_with_vfs(
            '/virtual/neplobj_char_dependency.nepl',
            charDependencyBodySource(2),
            {},
            ['wasm'],
            false,
        );
        assert.equal(
            neplObjDirectCallFragmentStoreStats(neplObjStoreSession).stores,
            0,
            'dependency artifacts must be projected before a source fallback can export .neplobj fragments',
        );
        neplObjStoreSession.compile_outputs_with_vfs(
            '/virtual/neplobj_char_dependency.nepl',
            charDependencyBodySource(3),
            {},
            ['wasm'],
            false,
        );
        const neplObjCandidateMaterialized = neplMetaMaterializedCompileStats(neplObjStoreSession);
        const neplObjCandidateStore = neplObjDirectCallFragmentStoreStats(neplObjStoreSession);
        assert.ok(
            neplObjCandidateMaterialized.bodyMissingFallbacks > 0,
            'body-missing materialized fallback must occur before the first .neplobj fragment can be stored',
        );
        assert.ok(
            neplObjCandidateMaterialized.bodyMissingSkipStores > 0,
            'body-missing fallback must still store negative skip entries for edges without object fragments',
        );
        assert.ok(
            neplObjCandidateStore.stores > 0,
            'source fallback must export at least one direct-call .neplobj fragment into the same-session store',
        );
        assert.equal(
            neplObjCandidateStore.lookupHits,
            0,
            'the compile that creates the first direct-call fragment must not count it as an earlier lookup hit',
        );
        neplObjStoreSession.compile_outputs_with_vfs(
            '/virtual/neplobj_char_dependency.nepl',
            charDependencyBodySource(4),
            {},
            ['wasm'],
            false,
        );
        const neplObjHitMaterialized = neplMetaMaterializedCompileStats(neplObjStoreSession);
        const neplObjHitStore = neplObjDirectCallFragmentStoreStats(neplObjStoreSession);
        assert.ok(
            neplObjHitStore.lookupHits > neplObjCandidateStore.lookupHits,
            'later body edit must look up and hit the same-session direct-call .neplobj fragment store',
        );
        assert.ok(
            neplObjHitStore.lookupFragmentsReturned > neplObjCandidateStore.lookupFragmentsReturned,
            'direct-call .neplobj store hit must return a fragment to PublicInterfaceArtifactInputs',
        );
        assert.equal(
            neplObjHitMaterialized.bodyMissingFallbacks,
            neplObjCandidateMaterialized.bodyMissingFallbacks,
            'object fragment availability must prevent another materialized function-body-missing fallback',
        );
        assert.equal(
            neplObjHitMaterialized.lastBodyMissingCandidateSurfaces,
            0,
            'the last materialized fallback after object lookup must not be counted as a body-missing .neplobj candidate',
        );
        assert.ok(
            neplObjHitMaterialized.attempts > neplObjCandidateMaterialized.attempts,
            'object candidate availability must keep the materialized compile probe open instead of hiding it behind the body-missing skip cache',
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

        return { checked: 14 };
    },
};
