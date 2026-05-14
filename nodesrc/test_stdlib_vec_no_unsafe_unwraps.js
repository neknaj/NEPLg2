#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPaths = [
    'stdlib/alloc/collections/vec.nepl',
    'stdlib/alloc/collections/vec/types.nepl',
    'stdlib/alloc/collections/vec/storage.nepl',
    'stdlib/alloc/collections/vec/storage/view.nepl',
    'stdlib/alloc/collections/vec/storage/alloc.nepl',
    'stdlib/alloc/collections/vec/storage/cleanup.nepl',
    'stdlib/alloc/collections/vec/storage/fill.nepl',
    'stdlib/alloc/collections/vec/access.nepl',
    'stdlib/alloc/collections/vec/access/header.nepl',
    'stdlib/alloc/collections/vec/access/data.nepl',
    'stdlib/alloc/collections/vec/raw.nepl',
    'stdlib/alloc/collections/vec/raw/element.nepl',
    'stdlib/alloc/collections/vec/transform.nepl',
    'stdlib/alloc/collections/vec/transform/map.nepl',
    'stdlib/alloc/collections/vec/transform/filter.nepl',
    'stdlib/alloc/collections/vec/transform/prefix.nepl',
    'stdlib/alloc/collections/vec/query.nepl',
    'stdlib/alloc/collections/vec/query/get.nepl',
    'stdlib/alloc/collections/vec/query/aggregate.nepl',
    'stdlib/alloc/collections/vec/query/predicate.nepl',
    'stdlib/alloc/collections/vec/mutation.nepl',
    'stdlib/alloc/collections/vec/mutation/push.nepl',
    'stdlib/alloc/collections/vec/mutation/replace.nepl',
    'stdlib/alloc/collections/vec/mutation/pop.nepl',
    'stdlib/alloc/collections/vec/mutation/cleanup.nepl',
    'stdlib/alloc/collections/vec/sort.nepl',
    'stdlib/alloc/collections/vec/sort/common.nepl',
    'stdlib/alloc/collections/vec/sort/simple.nepl',
    'stdlib/alloc/collections/vec/sort/simple/insertion.nepl',
    'stdlib/alloc/collections/vec/sort/simple/selection.nepl',
    'stdlib/alloc/collections/vec/sort/simple/exchange.nepl',
    'stdlib/alloc/collections/vec/sort/simple/gap.nepl',
    'stdlib/alloc/collections/vec/sort/raw.nepl',
    'stdlib/alloc/collections/vec/sort/raw/access.nepl',
    'stdlib/alloc/collections/vec/sort/raw/quick.nepl',
    'stdlib/alloc/collections/vec/sort/raw/heap.nepl',
    'stdlib/alloc/collections/vec/sort/quick.nepl',
    'stdlib/alloc/collections/vec/sort/heap.nepl',
    'stdlib/alloc/collections/vec/sort/merge.nepl',
    'stdlib/alloc/collections/vec/sort/merge/buffer.nepl',
    'stdlib/alloc/collections/vec/sort/merge/range.nepl',
    'stdlib/alloc/collections/vec/sort/merge/api.nepl',
];

const codeByPath = new Map();

function readImplementation(relPath) {
    const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
    return src
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}

function walkNeplFiles(dir, out = []) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            walkNeplFiles(fullPath, out);
        } else if (entry.isFile() && entry.name.endsWith('.nepl')) {
            out.push(path.relative(repoRoot, fullPath).replace(/\\/g, '/'));
        }
    }
    return out;
}

for (const relPath of relPaths) {
    codeByPath.set(relPath, readImplementation(relPath));
}

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
];

for (const [relPath, code] of codeByPath) {
    for (const pattern of forbidden) {
        assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap or checked deallocation helpers in implementation code`);
    }
    assert.deepEqual(
        unexpectedUnreachableLines(code),
        [],
        `${relPath} may only use unreachable for typed dealloc_ptr owner-cleanup invariants`,
    );
}

const vecRootCode = codeByPath.get('stdlib/alloc/collections/vec.nepl');
const vecTypesCode = codeByPath.get('stdlib/alloc/collections/vec/types.nepl');
const vecStorageRootCode = codeByPath.get('stdlib/alloc/collections/vec/storage.nepl');
const vecStorageViewCode = codeByPath.get('stdlib/alloc/collections/vec/storage/view.nepl');
const vecStorageAllocCode = codeByPath.get('stdlib/alloc/collections/vec/storage/alloc.nepl');
const vecStorageCleanupCode = codeByPath.get('stdlib/alloc/collections/vec/storage/cleanup.nepl');
const vecStorageFillCode = codeByPath.get('stdlib/alloc/collections/vec/storage/fill.nepl');
const vecStorageCode = [vecStorageRootCode, vecStorageViewCode, vecStorageAllocCode, vecStorageCleanupCode, vecStorageFillCode].join('\n');
const vecAccessRootCode = codeByPath.get('stdlib/alloc/collections/vec/access.nepl');
const vecAccessHeaderCode = codeByPath.get('stdlib/alloc/collections/vec/access/header.nepl');
const vecAccessDataCode = codeByPath.get('stdlib/alloc/collections/vec/access/data.nepl');
const vecAccessCode = [vecAccessRootCode, vecAccessHeaderCode, vecAccessDataCode].join('\n');
const vecRawRootCode = codeByPath.get('stdlib/alloc/collections/vec/raw.nepl');
const vecRawElementCode = codeByPath.get('stdlib/alloc/collections/vec/raw/element.nepl');
const vecRawCode = [vecRawRootCode, vecRawElementCode].join('\n');
const vecTransformRootCode = codeByPath.get('stdlib/alloc/collections/vec/transform.nepl');
const vecTransformMapCode = codeByPath.get('stdlib/alloc/collections/vec/transform/map.nepl');
const vecTransformFilterCode = codeByPath.get('stdlib/alloc/collections/vec/transform/filter.nepl');
const vecTransformPrefixCode = codeByPath.get('stdlib/alloc/collections/vec/transform/prefix.nepl');
const vecTransformCode = [vecTransformRootCode, vecTransformMapCode, vecTransformFilterCode, vecTransformPrefixCode].join('\n');
const vecQueryRootCode = codeByPath.get('stdlib/alloc/collections/vec/query.nepl');
const vecQueryGetCode = codeByPath.get('stdlib/alloc/collections/vec/query/get.nepl');
const vecQueryAggregateCode = codeByPath.get('stdlib/alloc/collections/vec/query/aggregate.nepl');
const vecQueryPredicateCode = codeByPath.get('stdlib/alloc/collections/vec/query/predicate.nepl');
const vecQueryCode = [vecQueryRootCode, vecQueryGetCode, vecQueryAggregateCode, vecQueryPredicateCode].join('\n');
const vecMutationRootCode = codeByPath.get('stdlib/alloc/collections/vec/mutation.nepl');
const vecMutationPushCode = codeByPath.get('stdlib/alloc/collections/vec/mutation/push.nepl');
const vecMutationReplaceCode = codeByPath.get('stdlib/alloc/collections/vec/mutation/replace.nepl');
const vecMutationPopCode = codeByPath.get('stdlib/alloc/collections/vec/mutation/pop.nepl');
const vecMutationCleanupCode = codeByPath.get('stdlib/alloc/collections/vec/mutation/cleanup.nepl');
const vecMutationCode = [vecMutationRootCode, vecMutationPushCode, vecMutationReplaceCode, vecMutationPopCode, vecMutationCleanupCode].join('\n');
const vecCode = [vecTypesCode, vecStorageCode, vecAccessCode, vecRawCode, vecTransformCode, vecQueryCode, vecMutationCode, vecRootCode].join('\n');
const sortMergeRootCode = codeByPath.get('stdlib/alloc/collections/vec/sort/merge.nepl');
const sortMergeBufferCode = codeByPath.get('stdlib/alloc/collections/vec/sort/merge/buffer.nepl');
const sortMergeRangeCode = codeByPath.get('stdlib/alloc/collections/vec/sort/merge/range.nepl');
const sortMergeApiCode = codeByPath.get('stdlib/alloc/collections/vec/sort/merge/api.nepl');
const sortRawRootCode = codeByPath.get('stdlib/alloc/collections/vec/sort/raw.nepl');
const sortRawAccessCode = codeByPath.get('stdlib/alloc/collections/vec/sort/raw/access.nepl');
const sortRawQuickCode = codeByPath.get('stdlib/alloc/collections/vec/sort/raw/quick.nepl');
const sortRawHeapCode = codeByPath.get('stdlib/alloc/collections/vec/sort/raw/heap.nepl');
const sortFamilyCode = relPaths
    .filter((relPath) => relPath.startsWith('stdlib/alloc/collections/vec/sort/'))
    .map((relPath) => codeByPath.get(relPath))
    .join('\n');

function between(code, start, end) {
    const startIdx = code.indexOf(start);
    assert.notEqual(startIdx, -1, `missing section start: ${start}`);
    const endIdx = code.indexOf(end, startIdx + start.length);
    assert.notEqual(endIdx, -1, `missing section end: ${end}`);
    return code.slice(startIdx, endIdx);
}

const pushSection = between(vecCode, 'fn push ', 'fn replace ');
const withCapacitySection = between(vecCode, 'fn with_capacity ', 'fn filled ');
const vecStorageAllocSource = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/collections/vec/storage/alloc.nepl'), 'utf8');
const popSection = between(vecCode, 'fn pop ', 'fn clear ');
const popSource = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/collections/vec/mutation/pop.nepl'), 'utf8');
const vecRawElementSource = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/collections/vec/raw/element.nepl'), 'utf8');
const vecAccessDataSource = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/collections/vec/access/data.nepl'), 'utf8');
const clearSection = between(vecCode, 'fn clear ', 'fn free ');
const mapSection = between(vecCode, 'fn map ', 'fn filter ');
const countSection = between(vecCode, 'fn count ', 'fn fold ');
const foldSection = between(vecCode, 'fn fold ', 'fn reduce ');
const reduceSection = between(vecCode, 'fn reduce ', 'fn find ');
const findSection = between(vecCode, 'fn find ', 'fn any ');
const anySection = between(vecCode, 'fn any ', 'fn all ');
const allSection = between(vecCode, 'fn all ', 'fn free ');
const freeSection = vecCode.slice(vecCode.indexOf('fn free '));

assert.doesNotMatch(vecCode, /\bfield::get\s+\w+\s+"(?:len|cap)"/, 'Vec implementation must read Copy len/cap header fields through field::get_ref so owner-consuming helpers do not move them');
assert.match(withCapacitySection, /fn\s+with_capacity\s+<\.T:\s*Copy>[\s\S]*if:\s+lt\s+cap\s+0\s+then:\s+Result::Err<Vec<\.T>,\s*StdErrorKind>\s+StdErrorKind::InvalidOperation[\s\S]*else:\s+vec_alloc_empty<\.T>\s+cap/, 'Vec.with_capacity must reject negative capacity before allocating owned storage and remain Copy-only');
for (const name of ['types', 'storage', 'access', 'mutation', 'query', 'transform', 'sort']) {
    assert.match(vecRootCode, new RegExp(`pub\\s+#import\\s+"\\.\\/vec\\/${name}"\\s+as\\s+@merge`), `Vec root must merge re-export vec/${name}.nepl`);
}
assert.doesNotMatch(vecRootCode, /pub\s+#import\s+"\.\/vec\/raw"\s+as\s+@merge/, 'Vec root must not merge re-export unchecked vec/raw.nepl');
assert.doesNotMatch(vecRootCode, /\b(?:fn|struct|enum|trait)\s+\w+\b/, 'Vec root must be a pure facade without implementation bodies');
assert.doesNotMatch(vecRootCode, /\bas\s+vec_/, 'Vec root must not keep private delegation aliases after becoming a merge facade');
for (const name of ['VecStorageState', 'Vec', 'VecPushError', 'VecTransformError', 'VecPop', 'VecPartition']) {
    assert.doesNotMatch(vecRootCode, new RegExp(`(?:enum|struct)\\s+${name}\\b`), `Vec root must not own ${name}; it belongs in vec/types.nepl`);
    assert.match(vecTypesCode, new RegExp(`(?:enum|struct)\\s+${name}\\b`), `vec/types.nepl must own ${name}`);
}
for (const name of ['vec_empty', 'vec_alloc_empty', 'vec_free_storage', 'new', 'with_capacity', 'filled']) {
    assert.match(vecStorageCode, new RegExp(`fn\\s+${name}\\b`), `vec/storage.nepl must own ${name}`);
}
for (const name of ['view', 'alloc', 'cleanup', 'fill']) {
    assert.match(vecStorageRootCode, new RegExp(`pub\\s+#import\\s+"\\.\\/storage\\/${name}"\\s+as\\s+@merge`), `vec/storage.nepl must merge re-export storage/${name}.nepl`);
}
assert.doesNotMatch(vecStorageRootCode, /\b(?:fn|struct|enum|trait)\s+\w+\b/, 'vec/storage.nepl must be a pure facade without implementation bodies');
for (const name of ['vec_empty']) {
    assert.match(vecStorageViewCode, new RegExp(`fn\\s+${name}\\b`), `vec/storage/view.nepl must own ${name}`);
}
assert.match(vecStorageViewCode, /pub\s+fn\s+vec_empty\s+<\.T>\s+<\(\)->Vec<\.T>>/, 'Vec.empty typed constructor must remain public');
assert.doesNotMatch(vecStorageViewCode, /pub\s+fn\s+vec_empty_region\b/, 'Vec empty RegionToken sentinel helper must remain private to storage/view.nepl');
assert.doesNotMatch(vecStorageViewCode, /\bfn\s+vec_storage_mem_ptr\b/, 'Vec storage MemPtr projection must not be exposed as a lower-level storage-state helper');
for (const name of ['vec_alloc_empty', 'new', 'with_capacity']) {
    assert.match(vecStorageAllocCode, new RegExp(`fn\\s+${name}\\b`), `vec/storage/alloc.nepl must own ${name}`);
}
assert.match(vecStorageCleanupCode, /\bfn\s+vec_free_storage\b/, 'vec/storage/cleanup.nepl must own vec_free_storage');
assert.match(vecStorageCleanupCode, /fn\s+vec_free_storage\s+<\.T:\s*Copy>\s+<\(VecStorageState,RegionToken<\.T>\)->\(\)>/, 'vec/storage/cleanup.nepl storage-only cleanup must take storage state and RegionToken and remain Copy-only until element drop traversal exists');
assert.match(vecStorageFillCode, /\bfn\s+filled\b/, 'vec/storage/fill.nepl must own filled');
for (const name of ['len', 'cap', 'data_mem_ptr', 'is_empty']) {
    assert.match(vecAccessCode, new RegExp(`fn\\s+${name}\\b`), `vec/access.nepl must own ${name}`);
}
for (const name of ['header', 'data']) {
    assert.match(vecAccessRootCode, new RegExp(`pub\\s+#import\\s+"\\.\\/access\\/${name}"\\s+as\\s+@merge`), `vec/access.nepl must merge re-export access/${name}.nepl`);
}
assert.doesNotMatch(vecAccessRootCode, /\b(?:fn|struct|enum|trait)\s+\w+\b/, 'vec/access.nepl must be a pure facade without implementation bodies');
for (const name of ['len', 'cap', 'is_empty']) {
    assert.match(vecAccessHeaderCode, new RegExp(`fn\\s+${name}\\b`), `vec/access/header.nepl must own ${name}`);
}
for (const name of ['data_mem_ptr']) {
    assert.match(vecAccessDataCode, new RegExp(`fn\\s+${name}\\b`), `vec/access/data.nepl must own ${name}`);
}
assert.doesNotMatch(vecAccessDataCode, /\bfn\s+data_ptr\b/, 'Vec.data_ptr must not reappear as a public raw i32 storage observer');
assert.match(vecAccessDataCode, /fn\s+data_mem_ptr\s+<\.T:\s*Copy>\s+<\(&Vec<\.T>\)->MemPtr<\.T>>/, 'Vec.data_mem_ptr must remain Copy-only because it exposes raw storage identity');
assert.match(vecAccessDataCode, /match\s+v_storage:[\s\S]*VecStorageState::Empty:[\s\S]*mem_ptr_wrap\s+0[\s\S]*VecStorageState::Owned:[\s\S]*region_ptr\s+v_region/, 'Vec.data_mem_ptr must own the storage-state projection so lower-level helpers do not become public API');
assert.match(vecAccessDataSource, /diag_codes:\s*type\.trait_bound\.unsatisfied[\s\S]*data_mem_ptr<NonCopyPayload>/, 'Vec raw data observer must reject non-Copy payloads in doctests');
for (const name of ['vec_read_at', 'vec_write_at']) {
    assert.match(vecRawCode, new RegExp(`fn\\s+${name}\\b`), `vec/raw facade closure must expose ${name}`);
    assert.doesNotMatch(vecRootCode, new RegExp(`\\b${name}\\b`), `Vec root facade must not expose ${name}; import alloc/collections/vec/raw explicitly`);
}
for (const name of ['element']) {
    assert.match(vecRawRootCode, new RegExp(`pub\\s+#import\\s+"\\.\\/raw\\/${name}"\\s+as\\s+@merge`), `vec/raw.nepl must merge re-export raw/${name}.nepl`);
}
for (const name of ['aggregate', 'predicate', 'prefix']) {
    assert.doesNotMatch(vecRawRootCode, new RegExp(`\\.\\/raw\\/${name}`), `vec/raw.nepl must not re-export raw ${name} callback helpers`);
    assert.equal(fs.existsSync(path.join(repoRoot, 'stdlib/alloc/collections/vec/raw', `${name}.nepl`)), false, `vec/raw/${name}.nepl must not keep unsafe raw callback helpers`);
}
assert.doesNotMatch(vecRawRootCode, /\b(?:fn|struct|enum|trait)\s+\w+\b/, 'vec/raw.nepl must be a pure facade without implementation bodies');
for (const name of ['vec_read_at', 'vec_write_at']) {
    assert.match(vecRawElementCode, new RegExp(`fn\\s+${name}\\b`), `vec/raw/element.nepl must own ${name}`);
}
assert.match(vecRawElementCode, /fn\s+vec_read_at\s+<\.T:\s*Copy>\s+<\(MemPtr<\.T>,\s*i32\)->\.T>/, 'Vec raw read helper must remain Copy-only until initialized move-out state exists');
assert.match(vecRawElementCode, /fn\s+vec_write_at\s+<\.T:\s*Copy>\s+<\(MemPtr<\.T>,\s*i32,\s*\.T\)->\(\)>/, 'Vec raw write helper must remain Copy-only until overwrite/drop state exists');
assert.match(vecRawElementSource, /diag_codes:\s*type\.trait_bound\.unsatisfied[\s\S]*vec_read_at<NonCopyPayload>[\s\S]*diag_codes:\s*type\.trait_bound\.unsatisfied[\s\S]*vec_write_at<NonCopyPayload>/, 'Vec raw element helpers must reject non-Copy payloads in doctests');
for (const name of ['map', 'filter', 'partition', 'take_while', 'drop_while']) {
    assert.match(vecTransformCode, new RegExp(`fn\\s+${name}\\b`), `vec/transform facade closure must expose ${name}`);
}
for (const name of ['map', 'filter', 'prefix']) {
    assert.match(vecTransformRootCode, new RegExp(`pub\\s+#import\\s+"\\.\\/transform\\/${name}"\\s+as\\s+@merge`), `vec/transform.nepl must merge re-export transform/${name}.nepl`);
}
assert.doesNotMatch(vecTransformRootCode, /\b(?:fn|struct|enum|trait)\s+\w+\b/, 'vec/transform.nepl must be a pure facade without implementation bodies');
assert.match(vecTransformMapCode, /\bfn\s+map\b/, 'vec/transform/map.nepl must own map');
for (const name of ['filter', 'partition']) {
    assert.match(vecTransformFilterCode, new RegExp(`fn\\s+${name}\\b`), `vec/transform/filter.nepl must own ${name}`);
}
for (const name of ['take_while', 'drop_while']) {
    assert.match(vecTransformPrefixCode, new RegExp(`fn\\s+${name}\\b`), `vec/transform/prefix.nepl must own ${name}`);
}
for (const name of ['get', 'count', 'fold', 'reduce', 'find', 'any', 'all']) {
    assert.match(vecQueryCode, new RegExp(`fn\\s+${name}\\b`), `vec/query facade closure must expose ${name}`);
}
for (const name of ['get', 'aggregate', 'predicate']) {
    assert.match(vecQueryRootCode, new RegExp(`pub\\s+#import\\s+"\\.\\/query\\/${name}"\\s+as\\s+@merge`), `vec/query.nepl must merge re-export query/${name}.nepl`);
}
assert.doesNotMatch(vecQueryRootCode, /\b(?:fn|struct|enum|trait)\s+\w+\b/, 'vec/query.nepl must be a pure facade without implementation bodies');
assert.match(vecQueryGetCode, /\bfn\s+get\b/, 'vec/query/get.nepl must own get');
for (const name of ['count', 'fold', 'reduce']) {
    assert.match(vecQueryAggregateCode, new RegExp(`fn\\s+${name}\\b`), `vec/query/aggregate.nepl must own ${name}`);
}
for (const name of ['find', 'any', 'all']) {
    assert.match(vecQueryPredicateCode, new RegExp(`fn\\s+${name}\\b`), `vec/query/predicate.nepl must own ${name}`);
}
for (const name of ['push', 'replace', 'pop', 'clear', 'free']) {
    assert.match(vecMutationCode, new RegExp(`fn\\s+${name}\\b`), `vec/mutation.nepl must own ${name}`);
}
for (const name of ['push', 'replace', 'pop', 'cleanup']) {
    assert.match(vecMutationRootCode, new RegExp(`pub\\s+#import\\s+"\\.\\/mutation\\/${name}"\\s+as\\s+@merge`), `vec/mutation.nepl must merge re-export mutation/${name}.nepl`);
}
assert.doesNotMatch(vecMutationRootCode, /\b(?:fn|struct|enum|trait)\s+\w+\b/, 'vec/mutation.nepl must be a pure facade without implementation bodies');
assert.match(vecMutationPushCode, /\bfn\s+push\b/, 'vec/mutation/push.nepl must own push');
assert.match(vecMutationReplaceCode, /\bfn\s+replace\b/, 'vec/mutation/replace.nepl must own replace');
assert.match(vecMutationPopCode, /\bfn\s+pop\b/, 'vec/mutation/pop.nepl must own pop');
for (const name of ['clear', 'free']) {
    assert.match(vecMutationCleanupCode, new RegExp(`fn\\s+${name}\\b`), `vec/mutation/cleanup.nepl must own ${name}`);
}
assert.match(vecCode, /enum\s+VecStorageState:[\s\S]*Empty[\s\S]*Owned/, 'Vec storage owner state must be represented by an enum');
const rawBoundaryEvidencePattern = /\b(?:mem_ptr_wrap|mem_ptr_addr|mem_ptr_add|alloc_ptr|realloc_ptr|dealloc_ptr|alloc_region|alloc_region_bytes|dealloc_region|load<|store<|load_i32|store_i32|load_u8|store_u8|mem_copy|mem_move|alloc_raw|dealloc_raw|realloc_raw|mem_size|mem_grow|memset_u8|fill_u8|fill_i32|mem_fill)\b|#intrinsic\s+"(?:load|store|str_addr|str_from_addr_unchecked)"/;
for (const relPath of [
    'stdlib/alloc/collections/vec.nepl',
    'stdlib/alloc/collections/vec/access.nepl',
    'stdlib/alloc/collections/vec/access/header.nepl',
    'stdlib/alloc/collections/vec/storage.nepl',
    'stdlib/alloc/collections/vec/mutation.nepl',
    'stdlib/alloc/collections/vec/mutation/cleanup.nepl',
    'stdlib/alloc/collections/vec/mutation/pop.nepl',
    'stdlib/alloc/collections/vec/mutation/replace.nepl',
    'stdlib/alloc/collections/vec/query.nepl',
    'stdlib/alloc/collections/vec/query/aggregate.nepl',
    'stdlib/alloc/collections/vec/query/get.nepl',
    'stdlib/alloc/collections/vec/query/predicate.nepl',
    'stdlib/alloc/collections/vec/raw.nepl',
    'stdlib/alloc/collections/vec/transform.nepl',
    'stdlib/alloc/collections/vec/transform/filter.nepl',
    'stdlib/alloc/collections/vec/transform/map.nepl',
    'stdlib/alloc/collections/vec/transform/prefix.nepl',
    'stdlib/alloc/collections/vec/types.nepl',
    'stdlib/alloc/collections/vec/sort.nepl',
    'stdlib/alloc/collections/vec/sort/common.nepl',
    'stdlib/alloc/collections/vec/sort/quick.nepl',
    'stdlib/alloc/collections/vec/sort/heap.nepl',
    'stdlib/alloc/collections/vec/sort/merge.nepl',
]) {
    assert.doesNotMatch(readImplementation(relPath), rawBoundaryEvidencePattern, `${relPath} must not carry direct raw memory boundary evidence`);
}
for (const relPath of [
    'stdlib/alloc/collections/vec/access/data.nepl',
    'stdlib/alloc/collections/vec/mutation/push.nepl',
    'stdlib/alloc/collections/vec/raw/element.nepl',
    'stdlib/alloc/collections/vec/storage/alloc.nepl',
    'stdlib/alloc/collections/vec/storage/cleanup.nepl',
    'stdlib/alloc/collections/vec/storage/fill.nepl',
    'stdlib/alloc/collections/vec/storage/view.nepl',
    'stdlib/alloc/collections/vec/sort/raw/access.nepl',
    'stdlib/alloc/collections/vec/sort/merge/api.nepl',
    'stdlib/alloc/collections/vec/sort/merge/buffer.nepl',
]) {
    assert.match(readImplementation(relPath), rawBoundaryEvidencePattern, `${relPath} must carry source-level raw memory boundary evidence`);
}
assert.match(vecCode, /struct\s+Vec<\.T>:[\s\S]*storage\s+<VecStorageState>[\s\S]*region\s+<RegionToken<\.T>>/, 'Vec must keep free obligation in a RegionToken field and not in a raw MemPtr field');
assert.match(vecCode, /fn\s+vec_empty_region\s+<\.T>\s+<\(\)->RegionToken<\.T>>[\s\S]*region_new\s+ptr\s+0[\s\S]*pub\s+fn\s+vec_empty\s+<\.T>\s+<\(\)->Vec<\.T>>[\s\S]*Vec<\.T>\s+0\s+0\s+VecStorageState::Empty\s+vec_empty_region<\.T>/, 'Vec.empty must construct typed Empty storage with a private zero-length RegionToken sentinel');
assert.match(vecCode, /fn\s+vec_alloc_empty\s+<\.T:\s*Copy>\s+<\(i32\)->Result<Vec<\.T>,\s*StdErrorKind>>[\s\S]*le\s+requested_cap\s+0[\s\S]*vec_empty<\.T>[\s\S]*alloc_region<\.T>\s+requested_cap[\s\S]*VecStorageState::Owned\s+region/, 'Vec empty construction must use Empty for zero capacity, Owned for allocated RegionToken storage, and remain Copy-only');
assert.match(vecCode, /fn\s+new\s+<\.T:\s*Copy>\s+<\(\)->Result<Vec<\.T>,\s*StdErrorKind>>[\s\S]*vec_alloc_empty<\.T>\s+8/, 'Vec.new must remain Copy-only until non-Copy cleanup exists');
assert.match(vecStorageAllocSource, /diag_codes:\s*type\.trait_bound\.unsatisfied[\s\S]*new<NonCopyPayload>[\s\S]*diag_codes:\s*type\.trait_bound\.unsatisfied[\s\S]*with_capacity<NonCopyPayload>/, 'Vec allocation constructors must reject non-Copy payloads in doctests');
assert.match(vecCode, /fn\s+vec_free_storage\s+<\.T:\s*Copy>[\s\S]*match\s+storage:[\s\S]*VecStorageState::Empty:[\s\S]*\(\)[\s\S]*VecStorageState::Owned:[\s\S]*match\s+dealloc_region<\.T>\s+region:[\s\S]*Result::Ok\s+_:[\s\S]*\(\)[\s\S]*Result::Err\s+_:[\s\S]*\(\)/, 'Vec.free must deallocate only Owned storage and must not send the Empty zero-size sentinel through dealloc_region');
assert.match(withCapacitySection, /vec_alloc_empty<\.T>\s+cap/, 'Vec.with_capacity must delegate empty storage allocation to vec_alloc_empty');
assert.doesNotMatch(vecCode, /(?:->|Result<)\.Pair\b|Tuple:/, 'Vec must not return owner-carrying Vec pairs through anonymous .Pair/Tuple values');
assert.match(vecCode, /struct\s+VecPop<\.T>:[\s\S]*vec\s+<Vec<\.T>>[\s\S]*item\s+<Option<\.T>>/, 'Vec.pop result must be a named struct with an owned Vec field');
assert.match(vecCode, /struct\s+VecPartition<\.T>:[\s\S]*matched\s+<Vec<\.T>>[\s\S]*rest\s+<Vec<\.T>>/, 'Vec.partition result must be a named struct with both owned Vec fields');
for (const relPath of walkNeplFiles(path.join(repoRoot, 'stdlib'))) {
    assert.doesNotMatch(readImplementation(relPath), /\b(?:[a-zA-Z_][\w]*::)?Vec<[^>\n]+>\s+0\s+0\s+mem_ptr_wrap\s+0/, `${relPath} must use Vec.empty typed storage instead of raw null owner sentinel`);
}
assert.match(pushSection, /let\s+v_storage\s+<VecStorageState>\s+\*field::get_ref\s+&v\s+"storage"[\s\S]*let\s+v_region\s+<RegionToken<\.T>>\s+field::get\s+v\s+"region"/, 'Vec.push must read typed storage state before moving the RegionToken owner from the consumed input Vec');
assert.match(pushSection, /fn\s+push\s+<\.T:\s*Copy>\s+<\(Vec<\.T>,\.T\)->Result<Vec<\.T>,\s*VecPushError<\.T>>>/, 'Vec.push must return an owner-preserving VecPushError payload on failure');
assert.match(pushSection, /match\s+v_storage:[\s\S]*VecStorageState::Empty:[\s\S]*alloc_region<\.T>\s+grown_cap[\s\S]*Result::Err<Vec<\.T>,\s*VecPushError<\.T>>\s+VecPushError<\.T>\s+\(Vec<\.T>\s+v_len\s+v_cap\s+v_storage\s+v_region\)\s+StdErrorKind::OutOfMemory[\s\S]*VecStorageState::Owned:[\s\S]*vec_realloc_region_or_keep<\.T>\s+v_region\s+grown_cap/, 'Vec.push must return the consumed Vec owner through VecPushError on Empty allocation failure and keep Owned grow transfer in RegionToken form');
assert.match(popSection, /let\s+v_storage\s+<VecStorageState>\s+\*field::get_ref\s+&v\s+"storage"[\s\S]*let\s+v_data\s+<MemPtr<\.T>>\s+vec_data::data_mem_ptr<\.T>\s+&v[\s\S]*let\s+v_region\s+<RegionToken<\.T>>\s+field::get\s+v\s+"region"/, 'Vec.pop must borrow a data view before moving the RegionToken owner into the returned Vec');
assert.match(popSection, /fn\s+pop\s+<\.T:\s*Copy>\s+<\(Vec<\.T>\)->VecPop<\.T>>/, 'Vec.pop must return named VecPop and remain Copy-only until initialized slot move state exists');
assert.match(popSource, /diag_codes:\s*type\.trait_bound\.unsatisfied[\s\S]*struct\s+NonCopyPayload:[\s\S]*pop<NonCopyPayload>/, 'Vec.pop must reject non-Copy payloads until OwnedBuffer initialized cell move-out exists');
assert.match(clearSection, /fn\s+clear\s+<\.T:\s*Copy>\s+<\(Vec<\.T>\)->Vec<\.T>>/, 'Vec.clear must remain Copy-only until initialized element drop traversal exists');
assert.match(clearSection, /let\s+v_storage\s+<VecStorageState>\s+\*field::get_ref\s+&v\s+"storage"[\s\S]*let\s+v_region\s+<RegionToken<\.T>>\s+field::get\s+v\s+"region"/, 'Vec.clear must explicitly move the RegionToken owner into the returned Vec with its storage state');
assert.match(freeSection, /fn\s+free\s+<\.T:\s*Copy>\s+<\(Vec<\.T>\)->\(\)>/, 'Vec.free must remain Copy-only until initialized element drop traversal exists');
assert.match(freeSection, /let\s+v_storage\s+<VecStorageState>\s+\*field::get_ref\s+&v\s+"storage"[\s\S]*vec_free_storage<\.T>\s+v_storage\s+field::get\s+v\s+"region"/, 'Vec.free must pass storage state with the RegionToken owner so Empty cleanup is not treated as an owned dealloc');
assert.match(mapSection, /let\s+out_storage\s+<VecStorageState>\s+\*field::get_ref\s+&out0\s+"storage"[\s\S]*let\s+out_data\s+<MemPtr<\.U>>\s+vec_data::data_mem_ptr<\.U>\s+&out0/, 'Vec.map must borrow the output data view from RegionToken before moving the output owner into the returned Vec');
assert.match(vecCode, /struct\s+VecPushError<\.T>:[\s\S]*vec\s+<Vec<\.T>>[\s\S]*error\s+<StdErrorKind>/, 'Vec.push failure payload must carry the consumed Vec owner and a copyable error kind');
assert.match(vecCode, /struct\s+VecTransformError<\.T>:[\s\S]*vec\s+<Vec<\.T>>[\s\S]*error\s+<StdErrorKind>[\s\S]*fn\s+vec_transform_error_vec\s+<\.T>\s+<\(VecTransformError<\.T>\)->Vec<\.T>>/, 'Vec transform failure payload must carry the consumed input Vec owner and expose an owner-moving accessor');
assert.match(vecCode, /fn\s+vec_realloc_region_or_keep\s+<\.T:\s*Copy>[\s\S]*match\s+realloc_ptr<\.T>\s+old_ptr\s+old_bytes\s+new_bytes:[\s\S]*Result::Ok\s+grown_ptr:[\s\S]*region_new\s+grown_ptr\s+new_bytes[\s\S]*Result::Err\s+_e:[\s\S]*VecReallocRegionError<\.T>\s+region\s+StdErrorKind::OutOfMemory/, 'Vec.push grow helper must return the old RegionToken owner on realloc failure instead of hiding cleanup inside implementation discipline');
assert.doesNotMatch(pushSection, /\bdealloc_region<\.T>\s+v_region\b|\bvec_realloc_region_or_free\b/, 'Vec.push must not consume and free the input Vec owner on grow failure');
assert.match(mapSection, /fn\s+map\s+<\.T:\s*Copy,\s*\.U:\s*Copy>\s+<\(Vec<\.T>,\s*\(\.T\)->\.U\)->Result<Vec<\.U>,\s*VecTransformError<\.T>>>[\s\S]*Result::Err<Vec<\.U>,\s*VecTransformError<\.T>>\s+VecTransformError<\.T>\s+v\s+e/, 'Vec.map must return the consumed input Vec owner on output allocation failure');
assert.match(vecTransformFilterCode, /fn\s+filter\s+<\.T:\s*Copy>\s+<\(Vec<\.T>,\s*\(\.T\)->bool\)->Result<Vec<\.T>,\s*VecTransformError<\.T>>>[\s\S]*Result::Err<Vec<\.T>,\s*VecTransformError<\.T>>\s+VecTransformError<\.T>\s+v\s+e/, 'Vec.filter must return the consumed input Vec owner on output allocation failure');
assert.match(vecTransformFilterCode, /Result::Err\s+e:[\s\S]*vec_cleanup::free<\.T>\s+left0[\s\S]*Result::Err<VecPartition<\.T>,\s*VecTransformError<\.T>>\s+VecTransformError<\.T>\s+v\s+e/, 'Vec.partition right allocation failure must free partial output and return the consumed input Vec owner');
assert.doesNotMatch(vecTransformFilterCode, /\bleft0_(?:cap|storage|data|region)\b/, 'Vec.partition must not reintroduce left0 storage field splitting for cleanup');
assert.match(vecCode, /fn\s+partition\s+<\.T:\s*Copy>\s+<\(Vec<\.T>,\s*\(\.T\)->bool\)->Result<VecPartition<\.T>,\s*VecTransformError<\.T>>>/, 'Vec.partition must return named VecPartition and return input owner on failure');
assert.match(countSection, /fn\s+count\s+<\.T:\s+Copy>\s+<\(&Vec<\.T>,\s*\(\.T\)->bool\)->i32>/, 'Vec.count must be a borrowed observer so callers retain and free the Vec owner');
assert.match(foldSection, /fn\s+fold\s+<\.T:\s+Copy,\s*\.U>\s+<\(&Vec<\.T>,\s*\.U,\s*\(\.U,\.T\)->\.U\)->\.U>/, 'Vec.fold must borrow the Vec owner and copy elements into the reducer');
assert.match(reduceSection, /fn\s+reduce\s+<\.T:\s+Copy>\s+<\(&Vec<\.T>,\s*\(\.T,\.T\)->\.T\)->Option<\.T>>/, 'Vec.reduce must borrow the Vec owner and require Copy elements');
assert.match(findSection, /fn\s+find\s+<\.T:\s+Copy>\s+<\(&Vec<\.T>,\s*\(\.T\)->bool\)->Option<\.T>>/, 'Vec.find must borrow the Vec owner and require Copy elements');
assert.match(anySection, /fn\s+any\s+<\.T:\s+Copy>\s+<\(&Vec<\.T>,\s*\(\.T\)->bool\)->bool>/, 'Vec.any must borrow the Vec owner and require Copy elements');
assert.match(allSection, /fn\s+all\s+<\.T:\s+Copy>\s+<\(&Vec<\.T>,\s*\(\.T\)->bool\)->bool>/, 'Vec.all must borrow the Vec owner and require Copy elements');
assert.match(mapSection, /fn\s+map\s+<\.T:\s*Copy,\s*\.U:\s*Copy>/, 'Vec.map must require Copy input and output elements until non-Copy element drop traversal exists');
assert.match(vecTransformFilterCode, /fn\s+filter\s+<\.T:\s*Copy>/, 'Vec.filter must require Copy elements for predicate scans and output copy');
assert.match(vecTransformPrefixCode, /fn\s+take_while\s+<\.T:\s*Copy>\s+<\(Vec<\.T>,\s*\(\.T\)->bool\)->Result<Vec<\.T>,\s*VecTransformError<\.T>>>[\s\S]*Result::Err<Vec<\.T>,\s*VecTransformError<\.T>>\s+VecTransformError<\.T>\s+v\s+e/, 'Vec.take_while must require Copy elements and return input owner on output allocation failure');
assert.match(vecTransformPrefixCode, /fn\s+drop_while\s+<\.T:\s*Copy>\s+<\(Vec<\.T>,\s*\(\.T\)->bool\)->Result<Vec<\.T>,\s*VecTransformError<\.T>>>[\s\S]*Result::Err<Vec<\.T>,\s*VecTransformError<\.T>>\s+VecTransformError<\.T>\s+v\s+e/, 'Vec.drop_while must require Copy elements and return input owner on output allocation failure');
assert.doesNotMatch(vecQueryCode, /vec_raw::vec_read_at<\.T>[\s\S]{0,80}\b(?:p|f)\b|\b(?:p|f)\s+vec_raw::vec_read_at<\.T>/, 'Vec query helpers must not pass raw-loaded elements directly to callbacks');
assert.doesNotMatch(vecTransformCode, /(?:p|f)\s+vec_raw::vec_read_at<\.T>|vec_raw::vec_write_at<\.[TU]>[\s\S]{0,80}vec_raw::vec_read_at<\.T>/, 'Vec transform helpers must not pass raw-loaded elements directly to callbacks or output storage');
for (const [name, section] of [
    ['count', countSection],
    ['fold', foldSection],
    ['reduce', reduceSection],
    ['find', findSection],
    ['any', anySection],
    ['all', allSection],
]) {
    assert.doesNotMatch(section, /field::get_ref\s+&v\s+"data"/, `Vec.${name} must not consume Vec just to read backing storage`);
}
assert.match(sortMergeRootCode, /pub\s+#import\s+"\.\/merge\/api"\s+as\s+\*/, 'sort/merge.nepl must re-export the public merge API');
assert.doesNotMatch(sortMergeRootCode, /pub\s+#import\s+"\.\/merge\/(?:buffer|range)"\s+as\s+\*/, 'sort/merge.nepl must not re-export raw merge buffer/range helpers');
assert.doesNotMatch(sortMergeRootCode, /\b(?:fn|struct|enum|trait)\s+\w+\b/, 'sort/merge.nepl must be a pure facade without implementation bodies');
for (const name of ['access', 'quick', 'heap']) {
    assert.match(sortRawRootCode, new RegExp(`pub\\s+#import\\s+"\\.\\/raw\\/${name}"\\s+as\\s+\\*`), `sort/raw.nepl must re-export raw/${name}.nepl`);
}
assert.doesNotMatch(sortRawRootCode, /\b(?:fn|struct|enum|trait)\s+\w+\b/, 'sort/raw.nepl must be a pure raw facade without implementation bodies');
assert.doesNotMatch(codeByPath.get('stdlib/alloc/collections/vec/sort.nepl'), /\bMemPtr\b|sort_i32|sort_slice_quick|sort_quick_range_data|sort_heap_sift_down_data|sort_merge_range_data|sort_buf_/, 'canonical sort facade must not expose raw MemPtr helper names');
assert.match(sortMergeBufferCode, /fn\s+sort_buf_get\s+<\.T:\s*Copy>/, 'sort/merge/buffer.nepl must own Copy-only scratch buffer loads');
assert.match(sortMergeBufferCode, /fn\s+sort_buf_set\s+<\.T:\s*Copy>\s+<\(MemPtr<\.T>,i32,\.T\)\*>\(\)>/, 'sort/merge/buffer.nepl must own Copy-only scratch buffer stores as an impure write');
assert.doesNotMatch(sortFamilyCode, /fn\s+sort_\w+\s+<\.T>\s+</, 'Vec sort raw load/store helpers must not be unconstrained over T');
assert.doesNotMatch(sortFamilyCode, /fn\s+sort_\w+\s+<\.T:\s*Ord>\s+</, 'Vec sort algorithms must require Ord&Copy until non-Copy move/drop-aware sorting exists');
assert.doesNotMatch(sortFamilyCode, /\bfn\s+sort_i32\b/, 'Vec sort must not expose the raw sort_i32 adapter through any sort module');
for (const [name, signature] of [
    ['sort_set_unchecked', /<\(&Vec<\.T>,i32,\.T\)\*>\(\)>/],
    ['sort_set_unchecked_data', /<\(MemPtr<\.T>,i32,\.T\)\*>\(\)>/],
    ['sort_swap_data', /<\(MemPtr<\.T>,i32,i32\)\*>\(\)>/],
    ['sort_swap', /<\(&Vec<\.T>,i32,i32\)\*>\(\)>/],
    ['sort_quick_partition_data', /<\(MemPtr<\.T>,i32,i32\)\*>i32>/],
    ['sort_quick_range_data', /<\(MemPtr<\.T>,i32,i32\)\*>\(\)>/],
    ['sort_quick', /<\(&Vec<\.T>\)\*>\(\)>/],
    ['sort_slice_quick', /<\(MemPtr<\.T>,i32\)\*>\(\)>/],
    ['sort_quick_ret', /<\(Vec<\.T>\)\*>Vec<\.T>>/],
    ['sort', /<\(&Vec<\.T>\)\*>\(\)>/],
    ['sort_heap_sift_down_data', /<\(MemPtr<\.T>,i32,i32\)\*>\(\)>/],
    ['sort_heap', /<\(&Vec<\.T>\)\*>\(\)>/],
    ['sort_heap_ret', /<\(Vec<\.T>\)\*>Vec<\.T>>/],
    ['sort_merge_range_data', /<\(MemPtr<\.T>,MemPtr<\.T>,i32,i32\)\*>\(\)>/],
    ['sort_bubble', /<\(&Vec<\.T>\)\*>\(\)>/],
    ['sort_cocktail', /<\(&Vec<\.T>\)\*>\(\)>/],
    ['sort_shell', /<\(&Vec<\.T>\)\*>\(\)>/],
    ['sort_comb', /<\(&Vec<\.T>\)\*>\(\)>/],
    ['sort_insertion', /<\(&Vec<\.T>\)\*>\(\)>/],
    ['sort_gnome', /<\(&Vec<\.T>\)\*>\(\)>/],
    ['sort_selection', /<\(&Vec<\.T>\)\*>\(\)>/],
]) {
    assert.match(sortFamilyCode, new RegExp(`fn\\s+${name}\\s+[^\\n]*${signature.source}`), `${name} mutates Vec/raw storage and must carry an impure effect signature`);
}
assert.match(sortMergeRangeCode, /fn\s+sort_merge_range_data\s+<\.T:\s+Ord&Copy>\s+<\(MemPtr<\.T>,MemPtr<\.T>,i32,i32\)\*>\(\)>[\s\S]*sort_buf_set<\.T>[\s\S]*sort_buf_get<\.T>/, 'sort/merge/range.nepl must own Copy-only impure range traversal and delegate scratch access');
assert.match(sortMergeApiCode, /pub\s+struct\s+VecSortMergeError<\.T>:[\s\S]*vec\s+<Vec<\.T>>[\s\S]*error\s+<StdErrorKind>/, 'sort_merge_ret failure payload must carry the consumed Vec owner and a copyable error kind');
assert.match(sortMergeApiCode, /fn\s+sort_merge\s+<\.T:\s+Ord&Copy>[\s\S]*match\s+alloc_region<\.T>\s+n:[\s\S]*let\s+buf\s+<MemPtr<\.T>>\s+region_ptr\s+&buf_region[\s\S]*match\s+dealloc_region<\.T>\s+buf_region:[\s\S]*Result::Ok\s+_:[\s\S]*Result<\(\),\s*StdErrorKind>::Ok\s+\(\)[\s\S]*Result::Err\s+_:[\s\S]*Result<\(\),\s*StdErrorKind>::Err\s+StdErrorKind::InvalidOperation/, 'sort_merge must keep scratch ownership in RegionToken and report cleanup failure without unreachable');
assert.match(sortMergeApiCode, /fn\s+sort_merge_ret\s+<\.T:\s+Ord&Copy>\s+<\(Vec<\.T>\)\*>Result<Vec<\.T>,\s*VecSortMergeError<\.T>>>[\s\S]*let\s+n\s+<i32>\s+len<\.T>\s+&v[\s\S]*match\s+alloc_region<\.T>\s+n:[\s\S]*Result<Vec<\.T>,\s*VecSortMergeError<\.T>>::Err\s+VecSortMergeError<\.T>\s+v\s+StdErrorKind::OutOfMemory[\s\S]*let\s+buf\s+<MemPtr<\.T>>\s+region_ptr\s+&buf_region[\s\S]*match\s+dealloc_region<\.T>\s+buf_region:[\s\S]*Result<Vec<\.T>,\s*VecSortMergeError<\.T>>::Ok\s+v[\s\S]*Result<Vec<\.T>,\s*VecSortMergeError<\.T>>::Err\s+VecSortMergeError<\.T>\s+v\s+StdErrorKind::InvalidOperation/, 'sort_merge_ret must return the consumed Vec owner on all error paths');
assert.doesNotMatch(sortMergeApiCode, /#intrinsic\s+"unreachable"/, 'sort/merge/api.nepl must not use unreachable for scratch cleanup');

console.log('vec unsafe unwrap regression passed');

function unexpectedUnreachableLines(code) {
    const lines = code.split(/\r?\n/);
    const unexpected = [];
    for (let i = 0; i < lines.length; i += 1) {
        if (!/#intrinsic\s+"unreachable"/.test(lines[i])) continue;
        const window = lines.slice(Math.max(0, i - 5), i + 1).join('\n');
        if (/\bmatch\s+dealloc_(?:ptr|region)<[^>]+>\s+[^\n]+:[\s\S]*\bResult::Err\s+_\w*:/.test(window)) continue;
        unexpected.push(`${i + 1}: ${lines[i].trim()}`);
    }
    return unexpected;
}
