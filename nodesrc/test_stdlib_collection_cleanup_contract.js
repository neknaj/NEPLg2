#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const collectionsRoot = path.join(repoRoot, 'stdlib/alloc/collections');

const inspected = [];
const ownerAccessorInspected = [];
const popOwnerAccessorInspected = [];
const fallibleOwnerConsumerInspected = [];
const ownerSurfaceInspected = [];
const emptyOwnerMetadataConstructorInspected = [];
const borrowedMetadataObserverInspected = [];
const borrowedCopyInvariantObserverInspected = [];
const borrowedPayloadCopyObserverInspected = [];
const borrowedStorageViewInspected = [];
const privateLifecycleProofHelperInspected = [];
const privateStorageReallocHelperInspected = [];
const privateProofBackedOwnerUpdateHelperInspected = [];
const privateProofBackedTailCleanupHelperInspected = [];
const ownerPreservingErrorRecoveryInspected = [];
const ownerPreservingPopRecoveryInspected = [];
const ownerPreservingPartitionRecoveryInspected = [];
const proofBackedPopMoveOutInspected = [];
const violations = [];

for (const relPath of walkNeplFiles(collectionsRoot)) {
    const source = readImplementation(relPath);
    const lines = source.split(/\r?\n/);
    for (let index = 0; index < lines.length; index += 1) {
        const signature = parseFunctionSignature(lines[index]);
        if (!signature || !isCleanupFunction(signature.name)) {
            const generics = parseGenericParameters(signature?.afterName ?? '');
            const typeSignature = parseTypeSignature(signature?.afterName ?? '');
            if (signature && generics.length > 0 && isOwnerReturningErrorAccessor(signature.name, typeSignature)) {
                ownerAccessorInspected.push(`${relPath}:${signature.name}`);
                const ownerPreservingErrorRecovery = classifyOwnerPreservingErrorRecovery(signature.name, typeSignature);
                if (ownerPreservingErrorRecovery) {
                    ownerPreservingErrorRecoveryInspected.push(`${relPath}:${signature.name}`);
                }
                const missingCopy = generics.filter((generic) => !/\bCopy\b/.test(generic.bound ?? ''));
                if (missingCopy.length > 0 && !ownerPreservingErrorRecovery) {
                    const names = missingCopy.map((generic) => `.${generic.name}`).join(', ');
                    violations.push(`${relPath}:${index + 1}: ${signature.name} owner-returning error accessor generic(s) ${names} must carry Copy until collection drop traversal exists`);
                }
            }
            if (signature && generics.length > 0 && isOwnerReturningPopAccessor(signature.name, typeSignature)) {
                popOwnerAccessorInspected.push(`${relPath}:${signature.name}`);
                const missingCopy = generics.filter((generic) => !/\bCopy\b/.test(generic.bound ?? ''));
                if (missingCopy.length > 0) {
                    const names = missingCopy.map((generic) => `.${generic.name}`).join(', ');
                    violations.push(`${relPath}:${index + 1}: ${signature.name} pop-result owner accessor generic(s) ${names} must carry Copy until collection drop traversal exists`);
                }
            }
            const fallibleOwnerConsumer = signature && generics.length > 0
                ? classifyFallibleOwnerConsumer(typeSignature)
                : null;
            if (fallibleOwnerConsumer) {
                fallibleOwnerConsumerInspected.push(`${relPath}:${signature.name}`);
                if (fallibleOwnerConsumer.errorKind === 'bare') {
                    violations.push(`${relPath}:${index + 1}: ${signature.name} consumes a generic collection owner but returns bare Diag/StdErrorKind on failure; use an owner-bearing error payload`);
                }
            }
            const ownerSurfaceCopyRequirement = signature && generics.length > 0
                ? collectionOwnerSurfaceCopyRequirement(typeSignature)
                : null;
            const privateStorageReallocHelper = signature && !signature.isPublic
                ? classifyPrivateStorageReallocHelper(source, signature.name, typeSignature)
                : null;
            const privateProofBackedOwnerUpdateHelper = signature && !signature.isPublic
                ? classifyPrivateProofBackedOwnerUpdateHelper(source, signature.name, typeSignature)
                : null;
            const privateProofBackedTailCleanupHelper = signature && !signature.isPublic
                ? classifyPrivateProofBackedTailCleanupHelper(source, signature.name, typeSignature)
                : null;
            if (privateStorageReallocHelper && !privateStorageReallocHelperInspected.includes(`${relPath}:${signature.name}`)) {
                privateStorageReallocHelperInspected.push(`${relPath}:${signature.name}`);
            }
            if (privateProofBackedOwnerUpdateHelper && !privateProofBackedOwnerUpdateHelperInspected.includes(`${relPath}:${signature.name}`)) {
                privateProofBackedOwnerUpdateHelperInspected.push(`${relPath}:${signature.name}`);
            }
            if (privateProofBackedTailCleanupHelper && !privateProofBackedTailCleanupHelperInspected.includes(`${relPath}:${signature.name}`)) {
                privateProofBackedTailCleanupHelperInspected.push(`${relPath}:${signature.name}`);
            }
            if (ownerSurfaceCopyRequirement && ownerSurfaceCopyRequirement.size > 0) {
                const ownerPreservingErrorRecovery = classifyOwnerPreservingErrorRecovery(signature.name, typeSignature);
                if (ownerPreservingErrorRecovery && !ownerPreservingErrorRecoveryInspected.includes(`${relPath}:${signature.name}`)) {
                    ownerPreservingErrorRecoveryInspected.push(`${relPath}:${signature.name}`);
                }
                const ownerPreservingPopRecovery = classifyOwnerPreservingPopRecovery(signature.name, typeSignature);
                if (ownerPreservingPopRecovery && !ownerPreservingPopRecoveryInspected.includes(`${relPath}:${signature.name}`)) {
                    ownerPreservingPopRecoveryInspected.push(`${relPath}:${signature.name}`);
                }
                const ownerPreservingPartitionRecovery = classifyOwnerPreservingPartitionRecovery(signature.name, typeSignature);
                if (ownerPreservingPartitionRecovery && !ownerPreservingPartitionRecoveryInspected.includes(`${relPath}:${signature.name}`)) {
                    ownerPreservingPartitionRecoveryInspected.push(`${relPath}:${signature.name}`);
                }
                const privateLifecycleProofHelper = !signature.isPublic
                    ? classifyPrivateCollectionLifecycleProofHelper(source, signature.name)
                    : null;
                const proofBackedPopMoveOut = classifyProofBackedPopMoveOutSurface(source, signature.name, typeSignature);
                if (proofBackedPopMoveOut && !proofBackedPopMoveOutInspected.includes(`${relPath}:${signature.name}`)) {
                    proofBackedPopMoveOutInspected.push(`${relPath}:${signature.name}`);
                }
                if (privateLifecycleProofHelper) {
                    privateLifecycleProofHelperInspected.push(`${relPath}:${signature.name}`);
                } else {
                    ownerSurfaceInspected.push(`${relPath}:${signature.name}`);
                }
                const emptyOwnerMetadataConstructor = classifyEmptyOwnerMetadataConstructor(source, signature.name, typeSignature);
                if (emptyOwnerMetadataConstructor) {
                    emptyOwnerMetadataConstructorInspected.push(`${relPath}:${signature.name}`);
                }
                const dropCapableOwnerSurface = classifyDropCapableOwnerSurface(source, signature.name, typeSignature);
                const missingCopy = generics.filter((generic) => ownerSurfaceCopyRequirement.has(generic.name) && !/\bCopy\b/.test(generic.bound ?? ''));
                if (missingCopy.length > 0) {
                    if (!emptyOwnerMetadataConstructor && !privateLifecycleProofHelper && !dropCapableOwnerSurface && !privateStorageReallocHelper && !privateProofBackedOwnerUpdateHelper && !privateProofBackedTailCleanupHelper && !ownerPreservingErrorRecovery && !ownerPreservingPopRecovery && !ownerPreservingPartitionRecovery && !proofBackedPopMoveOut) {
                        const names = missingCopy.map((generic) => `.${generic.name}`).join(', ');
                        violations.push(`${relPath}:${index + 1}: ${signature.name} owner-producing/updating generic collection surface ${names} must carry Copy or a structurally proven Drop cleanup/empty-allocation contract`);
                    }
                }
            }
            const borrowedMetadataObserverGenerics = signature && generics.length > 0
                ? borrowedCollectionOwnerMetadataObserverGenerics(signature.name, typeSignature)
                : null;
            if (borrowedMetadataObserverGenerics && borrowedMetadataObserverGenerics.size > 0) {
                borrowedMetadataObserverInspected.push(`${relPath}:${signature.name}`);
                const unnecessaryCopy = generics.filter((generic) => borrowedMetadataObserverGenerics.has(generic.name) && /\bCopy\b/.test(generic.bound ?? ''));
                if (unnecessaryCopy.length > 0) {
                    const names = unnecessaryCopy.map((generic) => `.${generic.name}`).join(', ');
                    violations.push(`${relPath}:${index + 1}: ${signature.name} borrowed metadata observer generic(s) ${names} must not carry Copy because it reads only header metadata`);
                }
            }
            const borrowedCopyInvariantObserverCopyRequirement = signature && generics.length > 0
                ? borrowedCollectionCopyInvariantObserverCopyRequirement(typeSignature)
                : null;
            if (borrowedCopyInvariantObserverCopyRequirement && borrowedCopyInvariantObserverCopyRequirement.size > 0) {
                borrowedCopyInvariantObserverInspected.push(`${relPath}:${signature.name}`);
                const missingCopy = generics.filter((generic) => borrowedCopyInvariantObserverCopyRequirement.has(generic.name) && !/\bCopy\b/.test(generic.bound ?? ''));
                if (missingCopy.length > 0) {
                    const names = missingCopy.map((generic) => `.${generic.name}`).join(', ');
                    violations.push(`${relPath}:${index + 1}: ${signature.name} borrowed Copy-invariant proof observer generic(s) ${names} must carry Copy because the proof authorizes Copy-only raw access`);
                }
            }
            const borrowedPayloadCopyObserverCopyRequirement = signature && generics.length > 0
                ? borrowedCollectionPayloadCopyObserverCopyRequirement(typeSignature)
                : null;
            if (borrowedPayloadCopyObserverCopyRequirement && borrowedPayloadCopyObserverCopyRequirement.size > 0) {
                borrowedPayloadCopyObserverInspected.push(`${relPath}:${signature.name}`);
                const missingCopy = generics.filter((generic) => borrowedPayloadCopyObserverCopyRequirement.has(generic.name) && !/\bCopy\b/.test(generic.bound ?? ''));
                if (missingCopy.length > 0) {
                    const names = missingCopy.map((generic) => `.${generic.name}`).join(', ');
                    violations.push(`${relPath}:${index + 1}: ${signature.name} borrowed payload-copying observer generic(s) ${names} must carry Copy until non-Copy slot copy-out is proven`);
                }
            }
            const borrowedStorageViewCopyRequirement = signature && generics.length > 0
                ? borrowedPayloadStorageViewCopyRequirement(typeSignature)
                : null;
            if (borrowedStorageViewCopyRequirement && borrowedStorageViewCopyRequirement.size > 0) {
                borrowedStorageViewInspected.push(`${relPath}:${signature.name}`);
                const missingCopy = generics.filter((generic) => borrowedStorageViewCopyRequirement.has(generic.name) && !/\bCopy\b/.test(generic.bound ?? ''));
                if (missingCopy.length > 0) {
                    const names = missingCopy.map((generic) => `.${generic.name}`).join(', ');
                    violations.push(`${relPath}:${index + 1}: ${signature.name} borrowed payload storage view generic(s) ${names} must carry Copy until collection drop traversal exists`);
                }
            }
            continue;
        }

        const generics = parseGenericParameters(signature.afterName);
        if (generics.length === 0) {
            continue;
        }
        inspected.push(`${relPath}:${signature.name}`);
        if (!signature.isPublic) {
            const privateLifecycleProofHelper = classifyPrivateCollectionLifecycleProofHelper(source, signature.name);
            if (privateLifecycleProofHelper) {
                privateLifecycleProofHelperInspected.push(`${relPath}:${signature.name}`);
            }
        }
        const privateLifecycleProofHelper = !signature.isPublic
            ? classifyPrivateCollectionLifecycleProofHelper(source, signature.name)
            : null;
        const storageReleaseOnlyLifecycleHelper = privateLifecycleProofHelper
            ? classifyStorageReleaseOnlyLifecycleHelper(source, signature.name)
            : null;
        const dropCapableCleanup = classifyDropCapableCleanup(source, signature.name);
        const missingCopy = generics.filter((generic) => !/\bCopy\b/.test(generic.bound ?? ''));
        if (missingCopy.length > 0) {
            if (!storageReleaseOnlyLifecycleHelper && !dropCapableCleanup) {
                const names = missingCopy.map((generic) => `.${generic.name}`).join(', ');
                violations.push(`${relPath}:${index + 1}: ${signature.name} cleanup generic(s) ${names} must carry Copy or a structurally proven Drop traversal/storage-release proof`);
            }
        }
    }
}

for (const relPath of walkNeplFiles(collectionsRoot)) {
    const source = readImplementation(relPath);
    for (const line of source.split(/\r?\n/)) {
        const signature = parseFunctionSignature(line);
        if (!signature || signature.isPublic) {
            continue;
        }
        const privateLifecycleProofHelper = classifyPrivateCollectionLifecycleProofHelper(source, signature.name);
        if (privateLifecycleProofHelper) {
            const entry = `${relPath}:${signature.name}`;
            if (!privateLifecycleProofHelperInspected.includes(entry)) {
                privateLifecycleProofHelperInspected.push(entry);
            }
        }
    }
}

assert.ok(
    inspected.length >= 15,
    `collection cleanup policy must inspect the current generic cleanup surface, inspected only ${inspected.length}`,
);

for (const expected of [
    'stdlib/alloc/collections/vec/mutation/cleanup.nepl:clear',
    'stdlib/alloc/collections/vec/mutation/cleanup.nepl:free',
    'stdlib/alloc/collections/vec/storage/cleanup.nepl:vec_free_storage',
    'stdlib/alloc/collections/hashmap/api.nepl:free',
    'stdlib/alloc/collections/hashset/api.nepl:free',
    'stdlib/alloc/collections/queue/api.nepl:clear',
    'stdlib/alloc/collections/queue/api.nepl:free',
    'stdlib/alloc/collections/deque/api.nepl:clear',
    'stdlib/alloc/collections/deque/api.nepl:free',
]) {
    assert.ok(
        inspected.some((entry) => entry.includes(expected)),
        `collection cleanup policy did not inspect expected cleanup signature: ${expected}`,
    );
}

assert.ok(
    ownerAccessorInspected.length >= 14,
    `collection owner accessor policy must inspect the current generic error owner surface, inspected only ${ownerAccessorInspected.length}`,
);

assert.ok(
    ownerPreservingErrorRecoveryInspected.includes('stdlib/alloc/collections/vec/mutation/push.nepl:vec_push_error_rejected'),
    'collection owner accessor policy must allow Vec.push rejected owner recovery only when the accessor returns the Vec and rejected item together',
);
assert.ok(
    ownerPreservingErrorRecoveryInspected.includes('stdlib/alloc/collections/vec/mutation/push.nepl:vec_push_rejected_with'),
    'collection owner accessor policy must allow Vec.push rejected payload elimination only when Vec and rejected item are passed to the same callback',
);
assert.ok(
    ownerPreservingErrorRecoveryInspected.includes('stdlib/alloc/collections/vec/mutation/replace.nepl:vec_replace_error_rejected'),
    'collection owner accessor policy must allow Vec.replace_drop_old rejected owner recovery only when the accessor returns the Vec and rejected item together',
);
assert.ok(
    ownerPreservingErrorRecoveryInspected.includes('stdlib/alloc/collections/vec/mutation/replace.nepl:vec_replace_rejected_with'),
    'collection owner accessor policy must allow Vec.replace_drop_old rejected payload elimination only when Vec and rejected item are passed to the same callback',
);
assert.ok(
    ownerPreservingErrorRecoveryInspected.includes('stdlib/alloc/collections/vec/types.nepl:vec_transform_error_with'),
    'collection owner accessor policy must allow Vec transform error elimination only when Vec owner and error kind are passed to the same callback',
);

for (const expected of [
    'stdlib/alloc/collections/vec/mutation/push.nepl:vec_push_error_vec',
    'stdlib/alloc/collections/vec/types.nepl:vec_transform_error_vec',
    'stdlib/alloc/collections/vec/sort/merge/api.nepl:vec_sort_merge_error_vec',
    'stdlib/alloc/collections/stack/types.nepl:stack_push_error_stack',
    'stdlib/alloc/collections/queue/types.nepl:queue_push_error_queue',
    'stdlib/alloc/collections/deque/types.nepl:deque_push_error_deque',
    'stdlib/alloc/collections/ringbuffer/types.nepl:ringbuffer_push_error_buffer',
    'stdlib/alloc/collections/binary_heap/types.nepl:binary_heap_push_error_heap',
    'stdlib/alloc/collections/list/types.nepl:list_push_error_list',
    'stdlib/alloc/collections/list/types.nepl:list_transform_error_list',
    'stdlib/alloc/collections/btreemap/types.nepl:btreemap_insert_error_owner',
    'stdlib/alloc/collections/btreeset/types.nepl:btreeset_insert_error_owner',
    'stdlib/alloc/collections/hashmap/types.nepl:hashmap_update_error_owner',
    'stdlib/alloc/collections/hashset/types.nepl:hashset_update_error_owner',
]) {
    assert.ok(
        ownerAccessorInspected.some((entry) => entry.includes(expected)),
        `collection owner accessor policy did not inspect expected error owner accessor: ${expected}`,
    );
}

assert.ok(
    privateStorageReallocHelperInspected.includes('stdlib/alloc/collections/vec/mutation/push.nepl:vec_realloc_region_or_keep'),
    'collection owner policy must classify Vec grow as a private storage-only RegionToken realloc helper instead of a public owner recovery surface',
);

assert.ok(
    privateProofBackedOwnerUpdateHelperInspected.includes('stdlib/alloc/collections/vec/mutation/push.nepl:vec_push_storage_checked'),
    'collection owner policy must classify Vec.push implementation as a private storage-invariant owner update helper instead of a public Copy-only surface',
);

assert.ok(
    privateProofBackedTailCleanupHelperInspected.includes('stdlib/alloc/collections/vec/mutation/pop.nepl:vec_drop_last_storage_checked_drop'),
    'collection owner policy must classify Vec.drop_last Drop implementation as a private single-slot cleanup helper instead of a public Copy-only surface',
);

assert.ok(
    proofBackedPopMoveOutInspected.includes('stdlib/alloc/collections/vec/mutation/pop.nepl:vec_pop_storage_checked')
        && proofBackedPopMoveOutInspected.includes('stdlib/alloc/collections/vec/mutation/pop.nepl:pop'),
    'collection owner policy must classify Vec.pop Drop move-out through storage invariant plus private MoveOut proof instead of requiring Copy',
);

assert.ok(
    ownerPreservingPopRecoveryInspected.includes('stdlib/alloc/collections/vec/types.nepl:vec_pop_with'),
    'collection pop owner recovery policy must allow Vec.pop only when Vec and Option<T> owners are passed to the same callback',
);
assert.ok(
    ownerPreservingPartitionRecoveryInspected.includes('stdlib/alloc/collections/vec/transform/filter/partition/view.nepl:vec_partition_with'),
    'collection partition owner recovery policy must allow VecPartition only when matched and rest Vec owners are passed to the same callback',
);

assert.ok(
    popOwnerAccessorInspected.length >= 6,
    `collection pop owner accessor policy must inspect the current generic pop owner surface, inspected only ${popOwnerAccessorInspected.length}`,
);

for (const expected of [
    'stdlib/alloc/collections/vec/types.nepl:vec_pop_vec',
    'stdlib/alloc/collections/stack/api.nepl:stack_pop_stack',
    'stdlib/alloc/collections/queue/types.nepl:queue_pop_queue',
    'stdlib/alloc/collections/deque/api.nepl:deque_pop_deque',
    'stdlib/alloc/collections/ringbuffer/types.nepl:ringbuffer_pop_buffer',
    'stdlib/alloc/collections/binary_heap/api/pop.nepl:binary_heap_pop_heap',
]) {
    assert.ok(
        popOwnerAccessorInspected.some((entry) => entry.includes(expected)),
        `collection pop owner accessor policy did not inspect expected pop owner accessor: ${expected}`,
    );
}

for (const expected of [
    'stdlib/alloc/collections/list/transform.nepl:map',
    'stdlib/alloc/collections/list/transform.nepl:filter',
]) {
    assert.ok(
        fallibleOwnerConsumerInspected.some((entry) => entry.includes(expected)),
        `collection fallible owner consumer policy did not inspect expected owner-consuming Result API: ${expected}`,
    );
}

assert.ok(
    ownerSurfaceInspected.length >= 45,
    `collection owner surface policy must inspect constructors, mutators, observers, and typed view APIs, inspected only ${ownerSurfaceInspected.length}`,
);

for (const expected of [
    'stdlib/alloc/collections/vec/storage/api.nepl:new',
    'stdlib/alloc/collections/vec/storage/api.nepl:with_capacity',
    'stdlib/alloc/collections/vec/storage/view.nepl:vec_empty',
    'stdlib/alloc/collections/vec/access/data.nepl:data_mem_view',
    'stdlib/alloc/collections/vec/query/get.nepl:get',
    'stdlib/alloc/collections/vec/mutation/push.nepl:push',
    'stdlib/alloc/collections/vec/mutation/replace.nepl:replace',
    'stdlib/alloc/collections/vec/mutation/pop.nepl:pop',
    'stdlib/alloc/collections/hashmap/api.nepl:new',
    'stdlib/alloc/collections/hashmap/api.nepl:insert',
    'stdlib/alloc/collections/hashmap/api.nepl:get',
    'stdlib/alloc/collections/hashmap/api.nepl:remove',
    'stdlib/alloc/collections/hashset/api.nepl:insert',
    'stdlib/alloc/collections/btreemap/api/insert.nepl:insert',
    'stdlib/alloc/collections/btreeset/api/insert.nepl:insert',
    'stdlib/alloc/collections/stack/api.nepl:push',
    'stdlib/alloc/collections/queue/api.nepl:push',
    'stdlib/alloc/collections/deque/api.nepl:push_front',
    'stdlib/alloc/collections/ringbuffer/api.nepl:push',
    'stdlib/alloc/collections/binary_heap/api/push.nepl:push',
    'stdlib/alloc/collections/list/basic.nepl:push',
]) {
    assert.ok(
        ownerSurfaceInspected.some((entry) => entry.includes(expected)),
        `collection owner surface policy did not inspect expected signature: ${expected}`,
    );
}

for (const expected of [
    'stdlib/alloc/collections/vec/mutation/push.nepl:vec_push_slot_store_initialized',
    'stdlib/alloc/collections/vec/mutation/cleanup.nepl:vec_cleanup_copy_initialized_prefix',
    'stdlib/alloc/collections/vec/mutation/cleanup.nepl:vec_cleanup_drop_initialized_prefix',
    'stdlib/alloc/collections/vec/mutation/cleanup.nepl:vec_cleanup_release_storage',
]) {
    assert.ok(
        privateLifecycleProofHelperInspected.some((entry) => entry.includes(expected)),
        `collection policy did not structurally classify expected private lifecycle proof helper: ${expected}`,
    );
}

assert.deepEqual(
    emptyOwnerMetadataConstructorInspected,
    ['stdlib/alloc/collections/vec/storage/view.nepl:vec_empty'],
    'only structurally proven zero-allocation Empty owner constructors may omit Copy on owner-producing generic collection surfaces',
);

assert.ok(
    borrowedMetadataObserverInspected.length >= 26,
    `collection borrowed metadata observer policy must inspect scalar metadata observer surfaces, inspected only ${borrowedMetadataObserverInspected.length}`,
);

for (const expected of [
    'stdlib/alloc/collections/vec/access/header.nepl:len',
    'stdlib/alloc/collections/vec/access/header.nepl:cap',
    'stdlib/alloc/collections/vec/access/header.nepl:is_empty',
    'stdlib/alloc/collections/vec/transform/filter/partition/view.nepl:vec_partition_matched_len',
    'stdlib/alloc/collections/vec/transform/filter/partition/view.nepl:vec_partition_rest_len',
    'stdlib/alloc/collections/stack/api.nepl:len',
    'stdlib/alloc/collections/stack/api.nepl:is_empty',
    'stdlib/alloc/collections/queue/api.nepl:len',
    'stdlib/alloc/collections/queue/api.nepl:is_empty',
    'stdlib/alloc/collections/deque/api.nepl:len',
    'stdlib/alloc/collections/deque/api.nepl:cap',
    'stdlib/alloc/collections/deque/api.nepl:is_empty',
    'stdlib/alloc/collections/ringbuffer/api.nepl:len',
    'stdlib/alloc/collections/ringbuffer/api.nepl:cap',
    'stdlib/alloc/collections/ringbuffer/api.nepl:is_empty',
    'stdlib/alloc/collections/binary_heap/api/observer.nepl:len',
    'stdlib/alloc/collections/binary_heap/api/observer.nepl:cap',
    'stdlib/alloc/collections/binary_heap/api/observer.nepl:is_empty',
    'stdlib/alloc/collections/list/query.nepl:len',
    'stdlib/alloc/collections/list/query.nepl:is_empty',
    'stdlib/alloc/collections/btreemap/api/observer.nepl:len',
    'stdlib/alloc/collections/btreeset/api/observer.nepl:len',
    'stdlib/alloc/collections/hashmap/api.nepl:len',
    'stdlib/alloc/collections/hashset/api.nepl:len',
    'stdlib/alloc/collections/bloom_filter/api.nepl:len',
    'stdlib/alloc/collections/counting_bloom_filter/api.nepl:len',
]) {
    assert.ok(
        borrowedMetadataObserverInspected.some((entry) => entry.includes(expected)),
        `collection borrowed metadata observer policy did not inspect expected signature: ${expected}`,
    );
}

assert.ok(
    borrowedCopyInvariantObserverInspected.length >= 2,
    `collection borrowed Copy-invariant proof observer policy must inspect raw-access proof surfaces, inspected only ${borrowedCopyInvariantObserverInspected.length}`,
);

for (const expected of [
    'stdlib/alloc/collections/vec/invariant.nepl:vec_buffer_current_copy_invariant',
    'stdlib/alloc/collections/vec/invariant.nepl:vec_current_copy_invariant',
]) {
    assert.ok(
        borrowedCopyInvariantObserverInspected.some((entry) => entry.includes(expected)),
        `collection borrowed Copy-invariant proof observer policy did not inspect expected signature: ${expected}`,
    );
}

assert.ok(
    borrowedPayloadCopyObserverInspected.length >= 20,
    `collection borrowed payload-copying observer policy must inspect Option<T> observer surfaces, inspected only ${borrowedPayloadCopyObserverInspected.length}`,
);

for (const expected of [
    'stdlib/alloc/collections/vec/query/get.nepl:get',
    'stdlib/alloc/collections/vec/query/aggregate.nepl:reduce',
    'stdlib/alloc/collections/vec/query/predicate.nepl:find',
    'stdlib/alloc/collections/vec/types.nepl:vec_pop_item',
    'stdlib/alloc/collections/stack/api.nepl:peek',
    'stdlib/alloc/collections/queue/api.nepl:peek',
    'stdlib/alloc/collections/deque/api.nepl:peek_front',
    'stdlib/alloc/collections/ringbuffer/api.nepl:peek',
    'stdlib/alloc/collections/binary_heap/api/observer.nepl:peek',
    'stdlib/alloc/collections/list/query.nepl:get',
    'stdlib/alloc/collections/btreemap/api/observer.nepl:get',
]) {
    assert.ok(
        borrowedPayloadCopyObserverInspected.some((entry) => entry.includes(expected)),
        `collection borrowed payload-copying observer policy did not inspect expected signature: ${expected}`,
    );
}

assert.ok(
    borrowedStorageViewInspected.length >= 6,
    `collection borrowed storage view policy must inspect payload-bearing Vec<Option<T>> storage views, inspected only ${borrowedStorageViewInspected.length}`,
);

for (const expected of [
    'stdlib/alloc/collections/btreemap/storage.nepl:btreemap_storage_keys',
    'stdlib/alloc/collections/btreemap/storage.nepl:btreemap_storage_values',
    'stdlib/alloc/collections/btreeset/storage.nepl:btreeset_storage_keys',
    'stdlib/alloc/collections/hashmap/storage.nepl:hashmap_storage_keys',
    'stdlib/alloc/collections/hashmap/storage.nepl:hashmap_storage_values',
    'stdlib/alloc/collections/hashset/storage.nepl:hashset_storage_keys',
]) {
    assert.ok(
        borrowedStorageViewInspected.some((entry) => entry.includes(expected)),
        `collection borrowed storage view policy did not inspect expected signature: ${expected}`,
    );
}

assert.deepEqual(violations, [], `generic collection cleanup, owner recovery, owner-producing APIs, borrowed Copy-invariant proof observers, borrowed payload-copying observers, borrowed payload storage views, and public lifecycle surfaces must remain Copy-only:\n${violations.join('\n')}`);

console.log('stdlib collection cleanup contract regression passed');

function walkNeplFiles(dir, out = []) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            walkNeplFiles(fullPath, out);
        } else if (entry.isFile() && entry.name.endsWith('.nepl')) {
            out.push(path.relative(repoRoot, fullPath).replace(/\\/g, '/'));
        }
    }
    return out.sort();
}

function readImplementation(relPath) {
    return fs
        .readFileSync(path.join(repoRoot, relPath), 'utf8')
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}

function parseFunctionSignature(line) {
    const match = line.match(/^\s*(pub\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(.*)$/);
    if (!match) {
        return null;
    }
    return { isPublic: Boolean(match[1]), name: match[2], afterName: match[3].trimStart() };
}

function isCleanupFunction(name) {
    return /(?:^|_)(?:clear|free)(?:_|$)/.test(name) || /\bcleanup\b/.test(name);
}

function isOwnerReturningErrorAccessor(name, typeSignature) {
    if (!typeSignature || !/(?:^|_)error_/.test(name)) {
        return false;
    }

    const functionType = parseUnaryFunctionType(typeSignature);
    if (!functionType) {
        return false;
    }

    return !functionType.parameter.trimStart().startsWith('&')
        && /\b[A-Za-z_][A-Za-z0-9_]*Error</.test(functionType.parameter)
        && /<\./.test(functionType.returnType);
}

function isOwnerReturningPopAccessor(name, typeSignature) {
    if (!typeSignature) {
        return false;
    }

    const functionType = parseUnaryFunctionType(typeSignature);
    if (!functionType) {
        return false;
    }

    return !functionType.parameter.trimStart().startsWith('&')
        && /\b[A-Za-z_][A-Za-z0-9_]*Pop</.test(functionType.parameter)
        && /<\./.test(functionType.returnType);
}

function classifyOwnerPreservingErrorRecovery(name, typeSignature) {
    if (!typeSignature) {
        return null;
    }
    if (name === 'vec_transform_error_with') {
        return classifyOwnerPreservingVecTransformErrorEliminator(typeSignature);
    }
    if (/_rejected_with$/.test(name)) {
        return classifyOwnerPreservingRejectedEliminator(typeSignature);
    }
    if (!/_error_rejected$/.test(name)) {
        return null;
    }

    const functionType = parseUnaryFunctionType(typeSignature);
    if (!functionType) {
        return null;
    }

    const parameter = functionType.parameter.trim();
    const returnType = functionType.returnType.trim();
    if (parameter.startsWith('&')) {
        return null;
    }
    if (!/\b[A-Za-z_][A-Za-z0-9_]*Error<\./.test(parameter)) {
        return null;
    }
    if (!/\b[A-Za-z_][A-Za-z0-9_]*Rejected<\./.test(returnType)) {
        return null;
    }

    return { kind: 'owner-preserving-error-recovery' };
}

function classifyOwnerPreservingRejectedEliminator(typeSignature) {
    const functionType = parseFunctionType(typeSignature);
    if (!functionType || functionType.parameters.length !== 2) {
        return null;
    }

    const rejected = functionType.parameters[0].trim();
    const rejectedMatch = rejected.match(/^[A-Za-z_][A-Za-z0-9_]*Rejected<\.(\w+)>$/);
    if (!rejectedMatch) {
        return null;
    }

    const callback = functionType.parameters[1].replace(/\s+/g, '');
    const returnType = functionType.returnType.replace(/\s+/g, '');
    const genericName = rejectedMatch[1];
    if (callback !== `(Vec<.${genericName}>,.${genericName})*>${returnType}`) {
        return null;
    }

    return { kind: 'owner-preserving-rejected-eliminator' };
}

function classifyOwnerPreservingVecTransformErrorEliminator(typeSignature) {
    const functionType = parseFunctionType(typeSignature);
    if (!functionType || functionType.parameters.length !== 2) {
        return null;
    }

    const errorPayload = functionType.parameters[0].trim();
    const errorMatch = errorPayload.match(/^VecTransformError<\.(\w+)>$/);
    if (!errorMatch) {
        return null;
    }

    const genericName = errorMatch[1];
    const callback = functionType.parameters[1].replace(/\s+/g, '');
    const returnType = functionType.returnType.replace(/\s+/g, '');
    if (callback !== `(Vec<.${genericName}>,StdErrorKind)*>${returnType}`) {
        return null;
    }

    return { kind: 'owner-preserving-transform-error-eliminator' };
}

function classifyOwnerPreservingPopRecovery(name, typeSignature) {
    if (name !== 'vec_pop_with') {
        return null;
    }

    const functionType = parseFunctionType(typeSignature);
    if (!functionType || functionType.parameters.length !== 2) {
        return null;
    }

    const popResult = functionType.parameters[0].trim();
    const popMatch = popResult.match(/^VecPop<\.(\w+)>$/);
    if (!popMatch) {
        return null;
    }

    const genericName = popMatch[1];
    const callback = functionType.parameters[1].replace(/\s+/g, '');
    const returnType = functionType.returnType.replace(/\s+/g, '');
    if (callback !== `(Vec<.${genericName}>,Option<.${genericName}>)*>${returnType}`) {
        return null;
    }

    return { kind: 'owner-preserving-pop-recovery' };
}

function classifyOwnerPreservingPartitionRecovery(name, typeSignature) {
    if (name !== 'vec_partition_with') {
        return null;
    }

    const functionType = parseFunctionType(typeSignature);
    if (!functionType || functionType.parameters.length !== 2) {
        return null;
    }

    const partitionResult = functionType.parameters[0].trim();
    const partitionMatch = partitionResult.match(/^VecPartition<\.(\w+)>$/);
    if (!partitionMatch) {
        return null;
    }

    const genericName = partitionMatch[1];
    const callback = functionType.parameters[1].replace(/\s+/g, '');
    const returnType = functionType.returnType.replace(/\s+/g, '');
    if (callback !== `(Vec<.${genericName}>,Vec<.${genericName}>)*>${returnType}`) {
        return null;
    }

    return { kind: 'owner-preserving-partition-recovery' };
}

function classifyFallibleOwnerConsumer(typeSignature) {
    if (!typeSignature) {
        return null;
    }

    const functionType = parseFunctionType(typeSignature);
    if (!functionType || functionType.parameters.length === 0) {
        return null;
    }

    const firstParameter = functionType.parameters[0].trim();
    if (firstParameter.startsWith('&') || !/\b[A-Z][A-Za-z0-9_]*<\./.test(firstParameter)) {
        return null;
    }

    const errorType = resultErrorType(functionType.returnType);
    if (!errorType) {
        return null;
    }

    return {
        errorKind: /^(?:Diag|StdErrorKind)$/.test(errorType.trim()) ? 'bare' : 'owner-bearing',
    };
}

function collectionOwnerSurfaceCopyRequirement(typeSignature) {
    if (!typeSignature) {
        return null;
    }

    const functionType = parseFunctionType(typeSignature);
    if (!functionType) {
        return null;
    }

    const required = new Set();
    const ownerGenericNames = new Set();
    for (const text of [...functionType.parameters, functionType.returnType]) {
        for (const name of ownerAggregateGenericNames(text)) {
            ownerGenericNames.add(name);
        }
    }

    if (ownerGenericNames.size === 0) {
        return null;
    }

    const returnType = functionType.returnType.trim();
    if (!returnType.startsWith('&')) {
        for (const name of ownerAggregateGenericNames(returnType)) {
            required.add(name);
        }

        const returnGenericNames = genericNames(returnType);
        if (setIntersects(ownerGenericNames, returnGenericNames)) {
            for (const name of returnGenericNames) {
                if (ownerGenericNames.has(name)) {
                    required.add(name);
                }
            }
        }
    }

    let exposesPayloadThroughByValueInput = false;
    for (const parameter of functionType.parameters) {
        const trimmed = parameter.trim();
        if (trimmed.startsWith('&')) {
            continue;
        }

        const parameterOwnerNames = ownerAggregateGenericNames(trimmed);
        for (const name of parameterOwnerNames) {
            required.add(name);
        }

        const parameterGenericNames = genericNames(trimmed);
        if (setIntersects(ownerGenericNames, parameterGenericNames)) {
            exposesPayloadThroughByValueInput = true;
            for (const name of parameterGenericNames) {
                if (ownerGenericNames.has(name)) {
                    required.add(name);
                }
            }
        }
    }

    if (exposesPayloadThroughByValueInput) {
        for (const name of ownerGenericNames) {
            required.add(name);
        }
    }

    return required;
}

function classifyEmptyOwnerMetadataConstructor(source, functionName, typeSignature) {
    if (!typeSignature) {
        return null;
    }

    const functionType = parseFunctionType(typeSignature);
    if (!functionType || functionType.parameters.length !== 0) {
        return null;
    }

    const returnType = functionType.returnType.trim();
    if (returnType !== 'Vec<.T>') {
        return null;
    }

    const section = implementationFunctionSection(source, functionName);
    if (!section) {
        return null;
    }

    const constructsEmptyOwnedBuffer = /Vec<\.T>\s+\(OwnedBuffer<\.T>\s+0\s+0\s+0\s+VecStorage<\.T>::Empty\)/.test(section);
    const touchesRuntimeStorage = /\b(?:alloc_region|alloc_region_bytes|dealloc_region|realloc_region|mem_ptr_wrap|field::get|field::get_ref|load<|store<)\b|VecStorage<\.T>::Owned/.test(section);
    if (!constructsEmptyOwnedBuffer || touchesRuntimeStorage) {
        return null;
    }

    return { kind: 'empty-owner-metadata-constructor' };
}

function classifyDropCapableOwnerSurface(source, functionName, typeSignature) {
    if (!typeSignature) {
        return null;
    }

    const functionType = parseFunctionType(typeSignature);
    if (!functionType) {
        return null;
    }

    const returnType = functionType.returnType.trim();
    if (!/\b(?:Vec|OwnedBuffer|VecStorage)<\.T>/.test(returnType)) {
        return null;
    }

    const section = implementationFunctionSections(source, functionName)
        .find((candidate) => /<\.\w+:\s*Drop>/.test(candidate));
    if (!section) {
        return null;
    }

    const proofBackedOwnerUpdate = classifyProofBackedOwnerUpdateSurface(source, section, functionType);
    if (proofBackedOwnerUpdate) {
        return proofBackedOwnerUpdate;
    }
    const proofBackedReplaceDropOld = classifyProofBackedReplaceDropOldSurface(source, section, functionType);
    if (proofBackedReplaceDropOld) {
        return proofBackedReplaceDropOld;
    }
    const proofBackedTailDrop = classifyProofBackedTailDropSurface(source, section, functionType);
    if (proofBackedTailDrop) {
        return proofBackedTailDrop;
    }

    const writesPayloadSlots = /\bstore<\.T>|\bload<\.T>|\bmem_copy\b|\bmem_move\b|\bcollection_slot_initialize_empty\b|\bcollection_slot_move_out\b/.test(section);
    if (writesPayloadSlots) {
        return null;
    }

    const delegatesEmptyAllocation = /\balloc::vec_alloc_empty<\.T>/.test(section);
    const constructsZeroInitializedAllocation = /\balloc_region<\.T>[\s\S]*OwnedBuffer<\.T>\s+0\s+0\s+\w+\s+\(VecStorage<\.T>::Owned\s+\w+\)/.test(section);
    if (!delegatesEmptyAllocation && !constructsZeroInitializedAllocation) {
        return null;
    }

    return { kind: 'drop-capable-owner-surface' };
}

function classifyProofBackedOwnerUpdateSurface(source, section, functionType) {
    if (functionType.parameters.length !== 2) {
        return null;
    }

    const [ownerParam, itemParam] = functionType.parameters.map((parameter) => parameter.trim());
    const returnType = functionType.returnType.trim();
    const match = ownerParam.match(/^Vec<\.(\w+)>$/);
    if (!match || itemParam !== `.${match[1]}`) {
        return null;
    }
    if (returnType !== `Result<Vec<.${match[1]}>, VecPushError<.${match[1]}>>`) {
        return null;
    }
    if (/\b(?:region_ptr|mem_ptr_addr|mem_ptr_add|store<|load<|mem_copy|mem_move)\b|#intrinsic\s+"collection_slot_/.test(section)) {
        return null;
    }

    const delegateMatch = section.match(/\b([A-Za-z_][A-Za-z0-9_]*)<\.\w+>\s+v\s+item\b/);
    if (!delegateMatch) {
        return null;
    }
    const helper = classifyPrivateProofBackedOwnerUpdateHelper(source, delegateMatch[1]);
    if (!helper) {
        return null;
    }
    return { kind: 'drop-capable-proof-backed-owner-surface' };
}

function classifyProofBackedReplaceDropOldSurface(source, section, functionType) {
    if (functionType.parameters.length !== 3) {
        return null;
    }

    const [ownerParam, indexParam, itemParam] = functionType.parameters.map((parameter) => parameter.trim());
    const returnType = functionType.returnType.trim();
    const match = ownerParam.match(/^Vec<\.(\w+)>$/);
    if (!match || indexParam !== 'i32' || itemParam !== `.${match[1]}`) {
        return null;
    }
    if (returnType !== `Result<Vec<.${match[1]}>, VecReplaceError<.${match[1]}>>`) {
        return null;
    }
    if (/\b(?:region_ptr|mem_ptr_addr|mem_ptr_add|store<|load<|mem_copy|mem_move)\b|#intrinsic\s+"collection_slot_/.test(section)) {
        return null;
    }
    if (!new RegExp(`\\bVecStorageInvariant\\b[\\s\\S]*\\bvec_buffer_current_storage_invariant<\\.${match[1]}>`).test(section)) {
        return null;
    }
    if (!new RegExp(`\\bVecReplaceRejected<\\.${match[1]}>[\\s\\S]*\\bitem\\b`).test(section)) {
        return null;
    }

    const helperNames = [...section.matchAll(/\b([A-Za-z_][A-Za-z0-9_]*)<\.\w+>\s+&[A-Za-z_][A-Za-z0-9_]*\s+byte_off\s+item\b/g)]
        .map((helperMatch) => helperMatch[1]);
    if (helperNames.length === 0) {
        return null;
    }
    if (!helperNames.every((helperName) => classifyPrivateCollectionLifecycleProofHelper(source, helperName))) {
        return null;
    }

    return { kind: 'drop-capable-proof-backed-replace-drop-old-surface' };
}

function classifyProofBackedPopMoveOutSurface(source, functionName, typeSignature) {
    if (!typeSignature) {
        return null;
    }

    const functionType = parseFunctionType(typeSignature);
    if (!functionType || functionType.parameters.length !== 1) {
        return null;
    }

    const ownerParam = functionType.parameters[0].trim();
    const match = ownerParam.match(/^Vec<\.(\w+)>$/);
    if (!match || functionType.returnType.trim() !== `VecPop<.${match[1]}>`) {
        return null;
    }

    const sections = implementationFunctionSections(source, functionName);
    if (sections.some((section) => classifyProofBackedPopMoveOutSection(source, section, match[1], new Set()))) {
        return { kind: 'proof-backed-pop-move-out-surface' };
    }

    return null;
}

function classifyProofBackedPopMoveOutSection(source, section, genericName, seen) {
    if (/\b(?:region_ptr|mem_ptr_addr|mem_ptr_add|store<|load<|mem_copy|mem_move)\b|#intrinsic\s+"collection_slot_/.test(section)) {
        return null;
    }

    const delegate = section.match(/\b([A-Za-z_][A-Za-z0-9_]*)<\.\w+>\s+v\b/);
    if (delegate) {
        const delegateName = delegate[1];
        if (seen.has(delegateName)) {
            return null;
        }
        seen.add(delegateName);
        return implementationFunctionSections(source, delegateName)
            .some((candidate) => classifyProofBackedPopMoveOutSection(source, candidate, genericName, seen));
    }

    if (!new RegExp(`\\bVecStorageInvariant\\b[\\s\\S]*\\bvec_buffer_current_storage_invariant<\\.${genericName}>`).test(section)) {
        return null;
    }
    if (!new RegExp(`\\bVecPop<\\.${genericName}>[\\s\\S]*\\bnone<\\.${genericName}>[\\s\\S]*\\bVecPop<\\.${genericName}>[\\s\\S]*\\bsome<\\.${genericName}>\\s+item`).test(section)) {
        return null;
    }

    const helperNames = [...section.matchAll(/\b([A-Za-z_][A-Za-z0-9_]*)<\.\w+>\s+&[A-Za-z_][A-Za-z0-9_]*\s+byte_off\b/g)]
        .map((helperMatch) => helperMatch[1]);
    if (helperNames.length === 0) {
        return null;
    }
    if (!helperNames.every((helperName) => classifyPrivateCollectionLifecycleProofHelper(source, helperName))) {
        return null;
    }

    return { kind: 'proof-backed-pop-move-out-section' };
}

function classifyProofBackedTailDropSurface(source, section, functionType) {
    if (functionType.parameters.length !== 1) {
        return null;
    }

    const ownerParam = functionType.parameters[0].trim();
    const returnType = functionType.returnType.trim();
    const match = ownerParam.match(/^Vec<\.(\w+)>$/);
    if (!match || returnType !== `Vec<.${match[1]}>`) {
        return null;
    }
    if (/\b(?:region_ptr|mem_ptr_addr|mem_ptr_add|store<|load<|mem_copy|mem_move)\b|#intrinsic\s+"collection_slot_/.test(section)) {
        return null;
    }

    const delegateMatch = section.match(/\b([A-Za-z_][A-Za-z0-9_]*)<\.\w+>\s+v\b/);
    if (!delegateMatch) {
        return null;
    }
    const helper = classifyPrivateProofBackedTailCleanupHelper(source, delegateMatch[1]);
    if (!helper) {
        return null;
    }
    return { kind: 'drop-capable-proof-backed-tail-cleanup-surface' };
}

function classifyPrivateProofBackedOwnerUpdateHelper(source, functionName, typeSignature = null) {
    const section = implementationFunctionSection(source, functionName);
    if (!section || /^\s*pub\s+fn\b/.test(section)) {
        return null;
    }

    const signature = typeSignature ?? parseTypeSignature(section.match(/^\s*fn\s+[A-Za-z_][A-Za-z0-9_]*\s*(.*)$/m)?.[1] ?? '');
    const functionType = signature ? parseFunctionType(signature) : null;
    if (!functionType || functionType.parameters.length !== 2) {
        return null;
    }

    const [ownerParam, itemParam] = functionType.parameters.map((parameter) => parameter.trim());
    const returnType = functionType.returnType.trim();
    const payloadMatch = ownerParam.match(/^Vec<\.(\w+)>$/);
    if (!payloadMatch || itemParam !== `.${payloadMatch[1]}`) {
        return null;
    }
    const payload = payloadMatch[1];
    if (returnType !== `Result<Vec<.${payload}>, VecPushError<.${payload}>>`) {
        return null;
    }
    if (!new RegExp(`\\bVecStorageInvariant\\b[\\s\\S]*\\bvec_buffer_current_storage_invariant<\\.${payload}>`).test(section)) {
        return null;
    }
    if (!new RegExp(`\\bVecStorageInvariant::Invalid\\b[\\s\\S]*\\bVecStorageInvariant::Valid\\b`).test(section)) {
        return null;
    }
    if (/\bVecCopyInvariant\b|\bvec_buffer_current_copy_invariant\b/.test(section)) {
        return null;
    }
    if (/\b(?:region_ptr|mem_ptr_addr|mem_ptr_add|store<|load<|mem_copy|mem_move)\b|#intrinsic\s+"collection_slot_/.test(section)) {
        return null;
    }
    if (!new RegExp(`\\bVecPushRejected<\\.${payload}>[\\s\\S]*\\bitem\\b`).test(section)) {
        return null;
    }

    const slotHelperNames = [...section.matchAll(/\b([A-Za-z_][A-Za-z0-9_]*)<\.\w+>\s+&[A-Za-z_][A-Za-z0-9_]*\s+byte_off\s+item\b/g)]
        .map((match) => match[1]);
    if (slotHelperNames.length === 0) {
        return null;
    }
    if (!slotHelperNames.every((helperName) => classifyPrivateCollectionLifecycleProofHelper(source, helperName))) {
        return null;
    }

    return { kind: 'private-proof-backed-owner-update-helper' };
}

function classifyPrivateProofBackedTailCleanupHelper(source, functionName, typeSignature = null) {
    const section = implementationFunctionSection(source, functionName);
    if (!section || /^\s*pub\s+fn\b/.test(section)) {
        return null;
    }

    const signature = typeSignature ?? parseTypeSignature(section.match(/^\s*fn\s+[A-Za-z_][A-Za-z0-9_]*\s*(.*)$/m)?.[1] ?? '');
    const functionType = signature ? parseFunctionType(signature) : null;
    if (!functionType || functionType.parameters.length !== 1) {
        return null;
    }

    const ownerParam = functionType.parameters[0].trim();
    const returnType = functionType.returnType.trim();
    const payloadMatch = ownerParam.match(/^Vec<\.(\w+)>$/);
    if (!payloadMatch || returnType !== `Vec<.${payloadMatch[1]}>`) {
        return null;
    }
    const payload = payloadMatch[1];
    if (!new RegExp(`\\bVecStorageInvariant\\b[\\s\\S]*\\bvec_buffer_current_storage_invariant<\\.${payload}>`).test(section)) {
        return null;
    }
    if (!new RegExp(`\\bVecStorageInvariant::Invalid\\b[\\s\\S]*\\bVecStorageInvariant::Valid\\b`).test(section)) {
        return null;
    }
    if (/\bVecCopyInvariant\b|\bvec_buffer_current_copy_invariant\b/.test(section)) {
        return null;
    }
    if (/\b(?:region_ptr|mem_ptr_addr|mem_ptr_add|store<|load<|mem_copy|mem_move)\b|#intrinsic\s+"collection_slot_/.test(section)) {
        return null;
    }
    if (!/\bsub\s+v_len\s+1\b[\s\S]*\bOwnedBuffer<\.\w+>\s+next_len\s+next_len\s+v_cap\b/.test(section)) {
        return null;
    }

    const slotHelperNames = [...section.matchAll(/\b([A-Za-z_][A-Za-z0-9_]*)<\.\w+>\s+&[A-Za-z_][A-Za-z0-9_]*\s+byte_off\b/g)]
        .map((match) => match[1]);
    if (slotHelperNames.length === 0) {
        return null;
    }
    if (!slotHelperNames.every((helperName) => classifyPrivateCollectionLifecycleProofHelper(source, helperName))) {
        return null;
    }

    return { kind: 'private-proof-backed-tail-cleanup-helper' };
}

function classifyPrivateStorageReallocHelper(source, functionName, typeSignature) {
    const section = implementationFunctionSection(source, functionName);
    if (!section || /^\s*pub\s+fn\b/.test(section)) {
        return null;
    }

    const functionType = parseFunctionType(typeSignature);
    if (!functionType || functionType.parameters.length !== 2) {
        return null;
    }

    const [storageParam, sizeParam] = functionType.parameters.map((parameter) => parameter.trim());
    const returnType = functionType.returnType.trim();
    if (!/^RegionToken<\.\w+>$/.test(storageParam) || sizeParam !== 'i32') {
        return null;
    }
    if (!/^Result<RegionToken<\.\w+>,\s*\w+<\.\w+>>$/.test(returnType)) {
        return null;
    }
    if (!/\brealloc_region_bytes_keep<\.\w+>/.test(section)) {
        return null;
    }
    if (/\b(?:load<|store<|mem_copy|mem_move)\b|#intrinsic\s+"collection_slot_/.test(section)) {
        return null;
    }

    return { kind: 'private-storage-realloc-helper' };
}

function classifyDropCapableCleanup(source, functionName) {
    const section = implementationFunctionSections(source, functionName)
        .find((candidate) => /<\.\w+:\s*Drop>/.test(candidate));
    if (!section) {
        return null;
    }

    const hasActualDropTraversal = /\bwhile\b[\s\S]*\bload<[^>]+>[\s\S]*\bDrop::drop\b[\s\S]*#intrinsic\s+"collection_slot_drop_traversal"/.test(section);
    const delegatesToDropTraversalHelper = /\bvec_cleanup_drop_initialized_prefix<\.\w+>/.test(section);
    if (!hasActualDropTraversal && !delegatesToDropTraversalHelper) {
        return null;
    }

    return { kind: 'drop-capable-cleanup' };
}

function classifyStorageReleaseOnlyLifecycleHelper(source, functionName) {
    const section = implementationFunctionSection(source, functionName);
    if (!section || /^\s*pub\s+fn\b/.test(section)) {
        return null;
    }

    const markers = [...section.matchAll(/#intrinsic\s+"(collection_slot_[^"]+)"/g)].map((match) => match[1]);
    if (markers.length !== 1 || markers[0] !== 'collection_slot_storage_dealloc') {
        return null;
    }

    if (!/\b(?:dealloc_region|dealloc_raw|allocator::dealloc_raw|allocator::dealloc_region)\b/.test(section)) {
        return null;
    }

    return { kind: 'storage-release-only-lifecycle-helper' };
}

function classifyPrivateCollectionLifecycleProofHelper(source, functionName) {
    const section = implementationFunctionSection(source, functionName);
    if (!section || /^\s*pub\s+fn\b/.test(section)) {
        return null;
    }

    const markers = [...section.matchAll(/#intrinsic\s+"(collection_slot_[^"]+)"/g)].map((match) => match[1]);
    if (markers.length === 0) {
        return null;
    }

    if (!markers.every((marker) => markerHasLocalRawLifecycleEvidence(section, marker))) {
        return null;
    }

    return { kind: 'private-collection-lifecycle-proof-helper' };
}

function markerHasLocalRawLifecycleEvidence(section, marker) {
    switch (marker) {
        case 'collection_slot_initialize_empty':
            return /\bstore<[^>]+>/.test(section);
        case 'collection_slot_move_out':
        case 'collection_slot_borrow_read':
            return /\bload<[^>]+>/.test(section);
        case 'collection_slot_drop_initialized':
            return /\bload<[^>]+>[\s\S]*\bDrop::drop\b/.test(section);
        case 'collection_slot_drop_traversal':
            return /\bwhile\b[\s\S]*\bload<[^>]+>/.test(section)
                && (/\bDrop::drop\b/.test(section) || /<\.\w+:\s*Copy>/.test(section));
        case 'collection_slot_storage_dealloc':
            return /\b(?:dealloc_region|dealloc_raw|allocator::dealloc_raw|allocator::dealloc_region)\b/.test(section);
        case 'collection_slot_storage_relocate':
            return /\b(?:realloc_region|realloc_raw|allocator::realloc_raw|allocator::realloc_region)\b/.test(section);
        case 'collection_slot_replace_return_old':
            return /\bload<[^>]+>[\s\S]*\bstore<[^>]+>/.test(section);
        case 'collection_slot_replace_drop_old':
            return /\bload<[^>]+>[\s\S]*\bstore<[^>]+>/.test(section)
                && (/\bDrop::drop\b/.test(section) || /<\.\w+:\s*Copy>/.test(section));
        default:
            return false;
    }
}

function implementationFunctionSection(source, functionName) {
    return implementationFunctionSections(source, functionName)[0] ?? null;
}

function implementationFunctionSections(source, functionName) {
    const startPattern = new RegExp(`(?:^|\\n)\\s*(?:pub\\s+)?fn\\s+${functionName}\\b`);
    const sections = [];
    let searchFrom = 0;
    while (searchFrom < source.length) {
        const startMatch = startPattern.exec(source.slice(searchFrom));
        if (!startMatch) {
            break;
        }
        const startIndex = searchFrom + startMatch.index;
        const restStart = startIndex + startMatch[0].length;
        const rest = source.slice(restStart);
        const nextMatch = /\n\s*(?:pub\s+)?fn\s+[A-Za-z_][A-Za-z0-9_]*\b/.exec(rest);
        const endIndex = nextMatch ? restStart + nextMatch.index : source.length;
        sections.push(source.slice(startIndex, endIndex));
        searchFrom = endIndex;
    }
    return sections;
}

function borrowedPayloadStorageViewCopyRequirement(typeSignature) {
    if (!typeSignature) {
        return null;
    }

    const functionType = parseFunctionType(typeSignature);
    if (!functionType) {
        return null;
    }

    const returnType = functionType.returnType.trim();
    if (!/^&Vec<Option<\./.test(returnType)) {
        return null;
    }

    return genericNames(returnType);
}

function borrowedCollectionOwnerMetadataObserverGenerics(functionName, typeSignature) {
    if (!typeSignature) {
        return null;
    }

    const functionType = parseFunctionType(typeSignature);
    if (!functionType || !isMetadataObserverName(functionName) || !isMetadataObserverReturn(functionType.returnType)) {
        return null;
    }

    const observed = new Set();
    for (const parameter of functionType.parameters) {
        const trimmed = parameter.trim();
        if (!trimmed.startsWith('&')) {
            continue;
        }
        for (const name of ownerAggregateGenericNames(trimmed)) {
            observed.add(name);
        }
    }
    return observed;
}

function borrowedCollectionPayloadCopyObserverCopyRequirement(typeSignature) {
    if (!typeSignature) {
        return null;
    }

    const functionType = parseFunctionType(typeSignature);
    if (!functionType || !isPayloadCopyObserverReturn(functionType.returnType)) {
        return null;
    }

    const required = new Set();
    for (const parameter of functionType.parameters) {
        const trimmed = parameter.trim();
        if (!trimmed.startsWith('&')) {
            continue;
        }
        for (const name of ownerAggregateGenericNames(trimmed)) {
            required.add(name);
        }
    }
    return required;
}

function borrowedCollectionCopyInvariantObserverCopyRequirement(typeSignature) {
    if (!typeSignature) {
        return null;
    }

    const functionType = parseFunctionType(typeSignature);
    if (!functionType || functionType.returnType.trim() !== 'VecCopyInvariant') {
        return null;
    }

    const required = new Set();
    for (const parameter of functionType.parameters) {
        const trimmed = parameter.trim();
        if (!trimmed.startsWith('&')) {
            continue;
        }
        for (const name of ownerAggregateGenericNames(trimmed)) {
            required.add(name);
        }
    }
    return required;
}

function isMetadataObserverReturn(returnType) {
    const trimmed = returnType.trim();
    return trimmed === 'i32'
        || trimmed === 'bool';
}

function isMetadataObserverName(name) {
    return [
        'len',
        'cap',
        'is_empty',
        'vec_partition_matched_len',
        'vec_partition_rest_len',
    ].includes(name);
}

function isPayloadCopyObserverReturn(returnType) {
    return /^Option<\./.test(returnType.trim());
}

function ownerAggregateGenericNames(text) {
    const names = new Set();
    const aggregatePattern = /\b([A-Z][A-Za-z0-9_]*)\s*</g;
    let match;
    while ((match = aggregatePattern.exec(text)) !== null) {
        const typeName = match[1];
        if (!isOwnerAggregateName(typeName)) {
            continue;
        }

        const angleStart = text.indexOf('<', match.index + typeName.length);
        const angleEnd = topLevelAngleEnd(text.slice(angleStart));
        if (angleStart === -1 || angleEnd === -1) {
            continue;
        }

        const inner = text.slice(angleStart + 1, angleStart + angleEnd);
        for (const name of genericNames(inner)) {
            names.add(name);
        }
    }
    return names;
}

function isOwnerAggregateName(typeName) {
    return [
        'Vec',
        'OwnedBuffer',
        'VecStorage',
        'VecDataView',
        'VecPushRejected',
        'VecPartition',
        'Stack',
        'Queue',
        'Deque',
        'RingBuffer',
        'BinaryHeap',
        'BTreeMap',
        'BTreeMapStorage',
        'BTreeSet',
        'BTreeSetStorage',
        'HashMap',
        'HashMapStorage',
        'HashSet',
        'HashSetStorage',
        'List',
        'BloomFilter',
        'CountingBloomFilter',
        'RegionToken',
    ].includes(typeName) || /(?:Error|Pop)$/.test(typeName);
}

function genericNames(text) {
    const names = new Set();
    const genericPattern = /\.(\w+)/g;
    let match;
    while ((match = genericPattern.exec(text)) !== null) {
        names.add(match[1]);
    }
    return names;
}

function setIntersects(left, right) {
    for (const value of left) {
        if (right.has(value)) {
            return true;
        }
    }
    return false;
}

function parseGenericParameters(afterName) {
    if (!afterName.startsWith('<') || afterName.startsWith('<(')) {
        return [];
    }

    const end = topLevelAngleEnd(afterName);
    if (end === -1) {
        return [];
    }

    const genericText = afterName.slice(1, end);
    const typeSignature = afterName.slice(end + 1).trimStart();
    if (!typeSignature.startsWith('<')) {
        return [];
    }

    return splitTopLevel(genericText, ',')
        .map((entry) => {
            const match = entry.trim().match(/^\.(\w+)(?:\s*:\s*(.+))?$/);
            if (!match) {
                return null;
            }
            return { name: match[1], bound: match[2]?.trim() };
        })
        .filter((entry) => entry !== null);
}

function parseTypeSignature(afterName) {
    let rest = afterName.trimStart();
    if (!rest.startsWith('<')) {
        return null;
    }
    if (!rest.startsWith('<(')) {
        const genericEnd = topLevelAngleEnd(rest);
        if (genericEnd === -1) {
            return null;
        }
        rest = rest.slice(genericEnd + 1).trimStart();
    }
    if (!rest.startsWith('<')) {
        return null;
    }
    const typeEnd = topLevelAngleEnd(rest);
    if (typeEnd === -1) {
        return null;
    }
    return rest.slice(0, typeEnd + 1);
}

function parseUnaryFunctionType(typeSignature) {
    const functionType = parseFunctionType(typeSignature);
    if (!functionType || functionType.parameters.length !== 1) {
        return null;
    }
    return { parameter: functionType.parameters[0], returnType: functionType.returnType };
}

function parseFunctionType(typeSignature) {
    if (!typeSignature.startsWith('<(') || !typeSignature.endsWith('>')) {
        return null;
    }

    const body = typeSignature.slice(1, -1);
    let angleDepth = 0;
    let parenDepth = 0;
    for (let i = 0; i < body.length; i += 1) {
        const ch = body[i];
        if (ch === '<') {
            angleDepth += 1;
        } else if (ch === '>' && body[i - 1] !== '-' && body[i - 1] !== '*') {
            angleDepth -= 1;
        } else if (ch === '(' && angleDepth === 0) {
            parenDepth += 1;
        } else if (ch === ')' && angleDepth === 0) {
            parenDepth -= 1;
            const separator = body.slice(i + 1, i + 3);
            if (parenDepth === 0 && (separator === '->' || separator === '*>')) {
                const parameterText = body.slice(1, i).trim();
                const returnType = body.slice(i + 3).trim();
                return { parameters: splitTopLevelTuple(parameterText, ','), returnType };
            }
        }
    }
    return null;
}

function topLevelAngleEnd(text) {
    let depth = 0;
    for (let i = 0; i < text.length; i += 1) {
        const ch = text[i];
        if (ch === '<') {
            depth += 1;
        } else if (ch === '>' && text[i - 1] !== '-' && text[i - 1] !== '*') {
            depth -= 1;
            if (depth === 0) {
                return i;
            }
        }
    }
    return -1;
}

function splitTopLevel(text, delimiter) {
    const parts = [];
    let depth = 0;
    let start = 0;
    for (let i = 0; i < text.length; i += 1) {
        const ch = text[i];
        if (ch === '<') {
            depth += 1;
        } else if (ch === '>' && text[i - 1] !== '-' && text[i - 1] !== '*') {
            depth -= 1;
        } else if (ch === delimiter && depth === 0) {
            parts.push(text.slice(start, i));
            start = i + 1;
        }
    }
    parts.push(text.slice(start));
    return parts;
}

function splitTopLevelTuple(text, delimiter) {
    if (text === '') {
        return [];
    }

    const parts = [];
    let angleDepth = 0;
    let parenDepth = 0;
    let start = 0;
    for (let i = 0; i < text.length; i += 1) {
        const ch = text[i];
        if (ch === '<') {
            angleDepth += 1;
        } else if (ch === '>' && text[i - 1] !== '-' && text[i - 1] !== '*') {
            angleDepth -= 1;
        } else if (ch === '(' && angleDepth === 0) {
            parenDepth += 1;
        } else if (ch === ')' && angleDepth === 0) {
            parenDepth -= 1;
        } else if (ch === delimiter && angleDepth === 0 && parenDepth === 0) {
            parts.push(text.slice(start, i).trim());
            start = i + 1;
        }
    }
    parts.push(text.slice(start).trim());
    return parts;
}

function resultErrorType(returnType) {
    const trimmed = returnType.trim();
    if (!trimmed.startsWith('Result<') || !trimmed.endsWith('>')) {
        return null;
    }

    const inner = trimmed.slice('Result<'.length, -1);
    const parts = splitTopLevel(inner, ',');
    if (parts.length !== 2) {
        return null;
    }
    return parts[1].trim();
}
