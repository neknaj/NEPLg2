#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPaths = [
    'stdlib/alloc/collections/vec.nepl',
    'stdlib/alloc/collections/vec/types.nepl',
    'stdlib/alloc/collections/vec/invariant.nepl',
    'stdlib/alloc/collections/vec/storage.nepl',
    'stdlib/alloc/collections/vec/storage/view.nepl',
    'stdlib/alloc/collections/vec/storage/api.nepl',
    'stdlib/alloc/collections/vec/storage/alloc.nepl',
    'stdlib/alloc/collections/vec/storage/cleanup.nepl',
    'stdlib/alloc/collections/vec/storage/fill.nepl',
    'stdlib/alloc/collections/vec/access.nepl',
    'stdlib/alloc/collections/vec/access/header.nepl',
    'stdlib/alloc/collections/vec/access/data.nepl',
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
    'stdlib/alloc/collections/vec/sort/quick.nepl',
    'stdlib/alloc/collections/vec/sort/heap.nepl',
    'stdlib/alloc/collections/vec/sort/merge.nepl',
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
const vecInvariantCode = codeByPath.get('stdlib/alloc/collections/vec/invariant.nepl');
const vecStorageRootCode = codeByPath.get('stdlib/alloc/collections/vec/storage.nepl');
const vecStorageViewCode = codeByPath.get('stdlib/alloc/collections/vec/storage/view.nepl');
const vecStorageApiCode = codeByPath.get('stdlib/alloc/collections/vec/storage/api.nepl');
const vecStorageAllocCode = codeByPath.get('stdlib/alloc/collections/vec/storage/alloc.nepl');
const vecStorageCleanupCode = codeByPath.get('stdlib/alloc/collections/vec/storage/cleanup.nepl');
const vecStorageCleanupSource = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/collections/vec/storage/cleanup.nepl'), 'utf8');
const vecStorageFillCode = codeByPath.get('stdlib/alloc/collections/vec/storage/fill.nepl');
const vecStorageCode = [vecStorageRootCode, vecStorageViewCode, vecStorageApiCode, vecStorageFillCode, vecStorageAllocCode, vecStorageCleanupCode].join('\n');
const vecAccessRootCode = codeByPath.get('stdlib/alloc/collections/vec/access.nepl');
const vecAccessHeaderCode = codeByPath.get('stdlib/alloc/collections/vec/access/header.nepl');
const vecAccessDataCode = codeByPath.get('stdlib/alloc/collections/vec/access/data.nepl');
const vecAccessCode = [vecAccessRootCode, vecAccessHeaderCode, vecAccessDataCode].join('\n');
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
const vecCode = [vecTypesCode, vecInvariantCode, vecStorageCode, vecAccessCode, vecTransformCode, vecQueryCode, vecMutationCode, vecRootCode].join('\n');
const sortMergeRootCode = codeByPath.get('stdlib/alloc/collections/vec/sort/merge.nepl');
const sortMergeApiCode = codeByPath.get('stdlib/alloc/collections/vec/sort/merge/api.nepl');
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

function publicFunctionSection(code, name) {
    const startPattern = new RegExp(`\\bpub\\s+fn\\s+${name}\\b`);
    const match = startPattern.exec(code);
    assert.notEqual(match, null, `missing public function: ${name}`);
    const startIdx = match.index;
    const rest = code.slice(startIdx + match[0].length);
    const nextMatch = /\n\s*pub\s+fn\s+\w+\b/.exec(rest);
    if (nextMatch === null) {
        return code.slice(startIdx);
    }
    return code.slice(startIdx, startIdx + match[0].length + nextMatch.index);
}

const pushSection = between(vecCode, 'fn push ', 'fn replace ');
const withCapacitySection = between(vecCode, 'fn with_capacity ', 'fn filled ');
const vecStorageViewSource = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/collections/vec/storage/view.nepl'), 'utf8');
const vecStorageApiSource = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/collections/vec/storage/api.nepl'), 'utf8');
const popSection = between(vecCode, 'fn pop ', 'fn clear ');
const popSource = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/collections/vec/mutation/pop.nepl'), 'utf8');
const vecAccessDataSource = fs.readFileSync(path.join(repoRoot, 'stdlib/alloc/collections/vec/access/data.nepl'), 'utf8');
const vecStdlibTestSource = fs.readFileSync(path.join(repoRoot, 'stdlib/tests/vec.n.md'), 'utf8');
const clearSection = between(vecCode, 'fn clear ', 'fn free ');
const mapSection = between(vecCode, 'fn map ', 'fn filter ');
const countSection = between(vecCode, 'fn count ', 'fn fold ');
const foldSection = between(vecCode, 'fn fold ', 'fn reduce ');
const reduceSection = between(vecCode, 'fn reduce ', 'fn find ');
const findSection = between(vecCode, 'fn find ', 'fn any ');
const anySection = between(vecCode, 'fn any ', 'fn all ');
const allSection = between(vecCode, 'fn all ', 'fn free ');
const freeSection = vecCode.slice(vecCode.indexOf('fn free '));
const ownedBufferSection = between(vecTypesCode, 'pub struct OwnedBuffer<.T>:', 'pub struct Vec<.T>:');
const vecStructSection = between(vecTypesCode, 'pub struct Vec<.T>:', 'pub struct VecPushError<.T>:');
const dataMemPtrUsageExample = between(vecAccessDataSource, '//: ### [使用例/しようれい]', '//: neplg2:test[compile_fail]');

assert.doesNotMatch(vecCode, /\bfield::get\s+\w+\s+"(?:len|cap)"/, 'Vec implementation must read Copy len/cap header fields through field::get_ref so owner-consuming helpers do not move them');
assert.match(vecInvariantCode, /fn\s+vec_buffer_current_copy_invariant\s+<\.T>\s+<\(&OwnedBuffer<\.T>\)->bool>[\s\S]*let\s+len0\s+<i32>[\s\S]*let\s+initialized_len0\s+<i32>[\s\S]*let\s+cap0\s+<i32>[\s\S]*VecStorage::Empty:[\s\S]*eq\s+len0\s+0[\s\S]*eq\s+cap0\s+0[\s\S]*VecStorage::Owned\s+_region:[\s\S]*gt\s+cap0\s+0/, 'Vec invariant helper must prove len/initialized_len/cap/storage correlation with enum match before raw element access');
assert.match(vecInvariantCode, /fn\s+vec_current_copy_invariant\s+<\.T>\s+<\(&Vec<\.T>\)->bool>[\s\S]*vec_buffer_current_copy_invariant<\.T>/, 'Vec invariant helper must expose a Vec facade observer for raw access boundaries');
assert.match(withCapacitySection, /fn\s+with_capacity\s+<\.T:\s*Copy>[\s\S]*if:\s+lt\s+cap\s+0\s+then:\s+Result::Err<Vec<\.T>,\s*StdErrorKind>\s+StdErrorKind::InvalidOperation[\s\S]*else:\s+alloc::vec_alloc_empty<\.T>\s+cap/, 'Vec.with_capacity must reject negative capacity before allocating owned storage and remain Copy-only');
for (const name of ['types', 'storage', 'access', 'mutation', 'query', 'transform', 'sort']) {
    assert.match(vecRootCode, new RegExp(`pub\\s+#import\\s+"\\.\\/vec\\/${name}"\\s+as\\s+@merge`), `Vec root must merge re-export vec/${name}.nepl`);
}
assert.doesNotMatch(vecRootCode, /pub\s+#import\s+"\.\/vec\/raw"\s+as\s+@merge/, 'Vec root must not merge re-export unchecked vec/raw.nepl');
assert.doesNotMatch(vecRootCode, /\b(?:fn|struct|enum|trait)\s+\w+\b/, 'Vec root must be a pure facade without implementation bodies');
assert.doesNotMatch(vecRootCode, /\bas\s+vec_/, 'Vec root must not keep private delegation aliases after becoming a merge facade');
for (const name of ['VecStorage', 'OwnedBuffer', 'Vec', 'VecPushError', 'VecTransformError', 'VecPop', 'VecPartition']) {
    assert.doesNotMatch(vecRootCode, new RegExp(`(?:enum|struct)\\s+${name}\\b`), `Vec root must not own ${name}; it belongs in vec/types.nepl`);
    assert.match(vecTypesCode, new RegExp(`(?:enum|struct)\\s+${name}\\b`), `vec/types.nepl must own ${name}`);
}
for (const name of ['vec_empty', 'vec_alloc_empty', 'vec_free_storage', 'new', 'with_capacity', 'filled']) {
    assert.match(vecStorageCode, new RegExp(`fn\\s+${name}\\b`), `vec/storage.nepl must own ${name}`);
}
for (const name of ['view', 'api', 'fill']) {
    assert.match(vecStorageRootCode, new RegExp(`pub\\s+#import\\s+"\\.\\/storage\\/${name}"\\s+as\\s+@merge`), `vec/storage.nepl must merge re-export storage/${name}.nepl`);
}
for (const name of ['alloc', 'cleanup']) {
    assert.doesNotMatch(vecStorageRootCode, new RegExp(`pub\\s+#import\\s+"\\.\\/storage\\/${name}"\\s+as\\s+@merge`), `vec/storage.nepl must not re-export internal storage/${name}.nepl`);
}
assert.doesNotMatch(vecStorageRootCode, /\b(?:fn|struct|enum|trait)\s+\w+\b/, 'vec/storage.nepl must be a pure facade without implementation bodies');
for (const name of ['vec_empty']) {
    assert.match(vecStorageViewCode, new RegExp(`fn\\s+${name}\\b`), `vec/storage/view.nepl must own ${name}`);
}
assert.match(vecStorageViewCode, /pub\s+fn\s+vec_empty\s+<\.T:\s*Copy>\s+<\(\)->Vec<\.T>>/, 'Vec.empty typed constructor must remain public and Copy-only until OwnedBuffer initialized drop traversal exists');
assert.doesNotMatch(vecStorageViewCode, /pub\s+fn\s+vec_empty_region\b/, 'Vec empty RegionToken sentinel helper must remain private to storage/view.nepl');
assert.doesNotMatch(vecStorageViewCode, /\bfn\s+vec_storage_mem_ptr\b/, 'Vec storage MemPtr projection must not be exposed as a lower-level storage-state helper');
assert.match(vecStorageAllocCode, /\bfn\s+vec_alloc_empty\b/, 'vec/storage/alloc.nepl must own vec_alloc_empty');
for (const name of ['new', 'with_capacity']) {
    assert.doesNotMatch(vecStorageAllocCode, new RegExp(`fn\\s+${name}\\b`), `vec/storage/alloc.nepl must not own public ${name}`);
    assert.match(vecStorageApiCode, new RegExp(`fn\\s+${name}\\b`), `vec/storage/api.nepl must own public ${name}`);
}
assert.match(vecStorageApiCode, /alloc::vec_alloc_empty<\.T>\s+8/, 'Vec.new must delegate allocation through the explicit storage/alloc helper');
assert.match(vecStorageApiCode, /alloc::vec_alloc_empty<\.T>\s+cap/, 'Vec.with_capacity must delegate allocation through the explicit storage/alloc helper');
assert.match(vecStorageCleanupCode, /\bfn\s+vec_free_storage\b/, 'vec/storage/cleanup.nepl must own vec_free_storage');
assert.match(vecStorageCleanupCode, /fn\s+vec_free_storage\s+<\.T:\s*Copy>\s+<\(VecStorage<\.T>\)->\(\)>/, 'vec/storage/cleanup.nepl storage-only cleanup must take the owner-carrying storage enum and remain Copy-only until element drop traversal exists');
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
assert.match(vecAccessDataCode, /match\s+v_storage:[\s\S]*VecStorage::Empty:[\s\S]*mem_ptr_wrap\s+0[\s\S]*VecStorage::Owned\s+region:[\s\S]*region_ptr\s+region/, 'Vec.data_mem_ptr must observe the owner-carrying storage enum so lower-level helpers do not become public API');
assert.match(vecAccessDataSource, /diag_codes:\s*type\.trait_bound\.unsatisfied[\s\S]*data_mem_ptr<NonCopyPayload>/, 'Vec raw data observer must reject non-Copy payloads in doctests');
assert.doesNotMatch(dataMemPtrUsageExample, /\bcore\/mem\/internal\b|\bmem_ptr_addr\b/, 'Vec.data_mem_ptr usage example must not teach raw address observation through core/mem/internal');
assert.match(dataMemPtrUsageExample, /\blet\s+_data\s+<MemPtr<i32>>\s+data_mem_ptr<i32>\s+&v[\s\S]*\bfree<i32>\s+v/, 'Vec.data_mem_ptr usage example must show typed observer use while retaining the Vec owner for cleanup');
assert.doesNotMatch(vecStdlibTestSource, /\bcore\/mem\/internal\b|\bmem_ptr_addr\b|data pointer positive/, 'stdlib/tests/vec.n.md must validate public Vec behavior without observing raw backing addresses');
assert.match(vecStdlibTestSource, /with_capacity starts empty/, 'stdlib/tests/vec.n.md must cover allocation behavior through public Vec observers');
assert.equal(fs.existsSync(path.join(repoRoot, 'stdlib/alloc/collections/vec/raw.nepl')), false, 'vec/raw.nepl must not remain as an explicitly importable unchecked Vec facade');
assert.equal(fs.existsSync(path.join(repoRoot, 'stdlib/alloc/collections/vec/raw.n.md')), false, 'vec/raw.n.md must not replace the removed unchecked Vec facade');
assert.equal(fs.existsSync(path.join(repoRoot, 'stdlib/alloc/collections/vec/raw/element.nepl')), false, 'vec/raw/element.nepl must not remain as an explicitly importable unchecked Vec element helper');
assert.equal(fs.existsSync(path.join(repoRoot, 'stdlib/alloc/collections/vec/raw/element.n.md')), false, 'vec/raw/element.n.md must not replace the removed unchecked Vec element helper');
for (const relPath of [
    'stdlib/alloc/collections/vec/sort/raw.nepl',
    'stdlib/alloc/collections/vec/sort/raw.n.md',
    'stdlib/alloc/collections/vec/sort/raw/access.nepl',
    'stdlib/alloc/collections/vec/sort/raw/access.n.md',
    'stdlib/alloc/collections/vec/sort/raw/quick.nepl',
    'stdlib/alloc/collections/vec/sort/raw/quick.n.md',
    'stdlib/alloc/collections/vec/sort/raw/heap.nepl',
    'stdlib/alloc/collections/vec/sort/raw/heap.n.md',
]) {
    assert.equal(fs.existsSync(path.join(repoRoot, relPath)), false, `${relPath} must not remain as an explicitly importable unchecked Vec sort helper`);
}
assert.doesNotMatch(vecCode, /#import\s+"(?:\.\.\/raw|\.\/raw|alloc\/collections\/vec\/raw)"/, 'Vec implementation must not depend on an explicitly importable vec/raw unchecked helper facade');
assert.doesNotMatch(sortFamilyCode, /#import\s+"(?:\.\.\/raw|\.\/raw|alloc\/collections\/vec\/sort\/raw)(?:\/[^"]*)?"/, 'Vec sort implementation must not depend on an explicitly importable vec/sort/raw unchecked helper facade');
for (const name of ['vec_read_at', 'vec_write_at']) {
    assert.doesNotMatch(vecCode, new RegExp(`\\b(?:pub\\s+)?fn\\s+${name}\\b`), `${name} must not reappear as a shared Vec raw helper`);
    assert.doesNotMatch(vecRootCode, new RegExp(`\\b${name}\\b`), `Vec root facade must not expose ${name}`);
}
assert.doesNotMatch(vecTransformPrefixCode, /\bpub\s+fn\s+vec_(?:take_while_len|copy_range_to_raw)\b/, 'Vec prefix raw/boundary helpers must remain private implementation details');
assert.doesNotMatch(vecCode, /\b(?:pub\s+)?fn\s+vec_(?:get_read_at|push_write_at|replace_write_at|pop_read_at|map_write_at|filter_write_at|prefix_write_at)\b/, 'Vec must not add shared raw element helper declarations just to replace the removed vec/raw facade');
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
assert.match(vecCode, /enum\s+VecStorage<\.T>:[\s\S]*Empty[\s\S]*Owned\s+<RegionToken<\.T>>/, 'Vec storage owner state must bind the owned RegionToken to the Owned enum variant');
const rawBoundaryEvidencePattern = /\b(?:mem_ptr_wrap|mem_ptr_addr|mem_ptr_add|alloc_ptr|realloc_ptr|dealloc_ptr|alloc_region|alloc_region_bytes|dealloc_region|load<|store<|load_i32|store_i32|load_u8|store_u8|mem_copy|mem_move|alloc_raw|dealloc_raw|realloc_raw|mem_size|mem_grow|memset_u8|fill_u8|fill_i32|mem_fill)\b|#intrinsic\s+"(?:load|store|str_addr|str_from_addr_unchecked)"/;
for (const relPath of [
    'stdlib/alloc/collections/vec.nepl',
    'stdlib/alloc/collections/vec/access.nepl',
    'stdlib/alloc/collections/vec/access/header.nepl',
    'stdlib/alloc/collections/vec/storage.nepl',
    'stdlib/alloc/collections/vec/storage/api.nepl',
    'stdlib/alloc/collections/vec/mutation.nepl',
    'stdlib/alloc/collections/vec/mutation/cleanup.nepl',
    'stdlib/alloc/collections/vec/query.nepl',
    'stdlib/alloc/collections/vec/query/aggregate.nepl',
    'stdlib/alloc/collections/vec/query/predicate.nepl',
    'stdlib/alloc/collections/vec/transform.nepl',
    'stdlib/alloc/collections/vec/types.nepl',
    'stdlib/alloc/collections/vec/sort.nepl',
    'stdlib/alloc/collections/vec/sort/common.nepl',
    'stdlib/alloc/collections/vec/sort/merge.nepl',
]) {
    assert.doesNotMatch(readImplementation(relPath), rawBoundaryEvidencePattern, `${relPath} must not carry direct raw memory boundary evidence`);
}
for (const relPath of [
    'stdlib/alloc/collections/vec/access/data.nepl',
    'stdlib/alloc/collections/vec/mutation/push.nepl',
    'stdlib/alloc/collections/vec/mutation/pop.nepl',
    'stdlib/alloc/collections/vec/mutation/replace.nepl',
    'stdlib/alloc/collections/vec/query/get.nepl',
    'stdlib/alloc/collections/vec/storage/alloc.nepl',
    'stdlib/alloc/collections/vec/storage/cleanup.nepl',
    'stdlib/alloc/collections/vec/storage/fill.nepl',
    'stdlib/alloc/collections/vec/transform/filter.nepl',
    'stdlib/alloc/collections/vec/transform/map.nepl',
    'stdlib/alloc/collections/vec/transform/prefix.nepl',
    'stdlib/alloc/collections/vec/sort/quick.nepl',
    'stdlib/alloc/collections/vec/sort/heap.nepl',
    'stdlib/alloc/collections/vec/sort/simple/insertion.nepl',
    'stdlib/alloc/collections/vec/sort/simple/selection.nepl',
    'stdlib/alloc/collections/vec/sort/simple/exchange.nepl',
    'stdlib/alloc/collections/vec/sort/simple/gap.nepl',
    'stdlib/alloc/collections/vec/sort/merge/api.nepl',
]) {
    assert.match(readImplementation(relPath), rawBoundaryEvidencePattern, `${relPath} must carry source-level raw memory boundary evidence`);
}
assert.match(ownedBufferSection, /len\s+<i32>[\s\S]*initialized_len\s+<i32>[\s\S]*cap\s+<i32>[\s\S]*storage\s+<VecStorage<\.T>>/, 'OwnedBuffer must own len/initialized_len/cap/storage so live length, initialized prefix, and free obligation state are separated from the Vec facade');
assert.match(vecStructSection, /buffer\s+<OwnedBuffer<\.T>>/, 'Vec must be a facade over OwnedBuffer instead of storing backing storage directly');
assert.doesNotMatch(vecStructSection, /\b(?:len|initialized_len|cap|storage)\s+</, 'Vec facade must not reintroduce direct len/initialized_len/cap/storage fields');
assert.match(vecCode, /pub\s+fn\s+vec_empty\s+<\.T:\s*Copy>\s+<\(\)->Vec<\.T>>[\s\S]*Vec<\.T>\s+\(OwnedBuffer<\.T>\s+0\s+0\s+0\s+VecStorage<\.T>::Empty\)/, 'Vec.empty must construct typed Empty storage through OwnedBuffer without a zero-length RegionToken sentinel and remain Copy-only');
assert.doesNotMatch(vecCode, /OwnedBuffer<\.[TU]>\s+(?:v_len\s+v_cap|next_len\s+v_cap|src_len\s+out_cap|keep_len\s+out_cap|left_len\s+left_cap|right_len\s+right_cap|0\s+0\s+VecStorage<\.[TU]>::Empty|0\s+requested_cap|n\s+n\s+\(VecStorage)/, 'Vec implementation must not regress to the old three-field OwnedBuffer constructor shape');
assert.match(vecStorageViewSource, /OwnedBuffer<T>[\s\S]*vec_empty\s+<\.T:\s*Copy>/, 'Vec.empty documentation must explain the temporary Copy-only contract until OwnedBuffer initialized drop traversal exists');
assert.match(vecCode, /fn\s+vec_alloc_empty\s+<\.T:\s*Copy>\s+<\(i32\)->Result<Vec<\.T>,\s*StdErrorKind>>[\s\S]*le\s+requested_cap\s+0[\s\S]*vec_empty<\.T>[\s\S]*alloc_region<\.T>\s+requested_cap[\s\S]*VecStorage<\.T>::Owned\s+region/, 'Vec empty construction must use Empty for zero capacity and Owned(region) for allocated RegionToken storage');
assert.match(vecCode, /fn\s+new\s+<\.T:\s*Copy>\s+<\(\)->Result<Vec<\.T>,\s*StdErrorKind>>[\s\S]*vec_alloc_empty<\.T>\s+8/, 'Vec.new must remain Copy-only until non-Copy cleanup exists');
assert.match(vecStorageApiSource, /diag_codes:\s*type\.trait_bound\.unsatisfied[\s\S]*new<NonCopyPayload>[\s\S]*diag_codes:\s*type\.trait_bound\.unsatisfied[\s\S]*with_capacity<NonCopyPayload>/, 'Vec allocation constructors must reject non-Copy payloads in doctests');
assert.match(vecCode, /fn\s+vec_free_storage\s+<\.T:\s*Copy>[\s\S]*\(storage\):[\s\S]*match\s+storage:[\s\S]*VecStorage::Empty:[\s\S]*\(\)[\s\S]*VecStorage::Owned\s+region:[\s\S]*dealloc_region<\.T>\s+region/, 'Vec.free must consume the RegionToken owner only through the Owned storage variant');
assert.match(vecStorageCleanupSource, /VecStorage::Owned[\s\S]*Empty[\s\S]*RegionToken/, 'Vec.storage cleanup docs must explain that the owner token is structurally tied to the Owned variant');
assert.match(withCapacitySection, /alloc::vec_alloc_empty<\.T>\s+cap/, 'Vec.with_capacity must delegate empty storage allocation to vec_alloc_empty');
assert.doesNotMatch(vecCode, /(?:->|Result<)\.Pair\b|Tuple:/, 'Vec must not return owner-carrying Vec pairs through anonymous .Pair/Tuple values');
assert.doesNotMatch(vecCode, /\bVec<\.[TU]>\s+(?:[A-Za-z_]\w*|\d+)\s+(?:[A-Za-z_]\w*|\d+)\s+(?:VecStorage|[A-Za-z_]\w*)/, 'Vec constructors must pass a single OwnedBuffer payload instead of direct len/cap/storage fields');
assert.match(vecCode, /struct\s+VecPop<\.T>:[\s\S]*vec\s+<Vec<\.T>>[\s\S]*item\s+<Option<\.T>>/, 'Vec.pop result must be a named struct with an owned Vec field');
assert.match(vecCode, /struct\s+VecPartition<\.T>:[\s\S]*matched\s+<Vec<\.T>>[\s\S]*rest\s+<Vec<\.T>>/, 'Vec.partition result must be a named struct with both owned Vec fields');
for (const relPath of walkNeplFiles(path.join(repoRoot, 'stdlib'))) {
    assert.doesNotMatch(readImplementation(relPath), /\b(?:[a-zA-Z_][\w]*::)?Vec<[^>\n]+>\s+0\s+0\s+mem_ptr_wrap\s+0/, `${relPath} must use Vec.empty typed storage instead of raw null owner sentinel`);
}
assert.match(pushSection, /let\s+v_buffer\s+<OwnedBuffer<\.T>>\s+field::get\s+v\s+"buffer"[\s\S]*let\s+v_storage\s+<VecStorage<\.T>>\s+field::get\s+v_buffer\s+"storage"/, 'Vec.push must move the owner-carrying storage enum through OwnedBuffer from the consumed input Vec');
assert.match(pushSection, /let\s+v_initialized_len\s+<i32>\s+\*field::get_ref\s+v_buffer_ref\s+"initialized_len"/, 'Vec.push must read and preserve initialized_len separately from len on failure paths');
assert.match(pushSection, /vec_buffer_current_copy_invariant<\.T>\s+v_buffer_ref[\s\S]*not\s+v_invariant_ok[\s\S]*VecPushError<\.T>\s+\(Vec<\.T>\s+\(OwnedBuffer<\.T>\s+v_len\s+v_initialized_len\s+v_cap\s+v_storage\)\)\s+StdErrorKind::InvalidOperation/, 'Vec.push must reject malformed OwnedBuffer metadata before raw store or grow');
assert.match(pushSection, /fn\s+push\s+<\.T:\s*Copy>\s+<\(Vec<\.T>,\.T\)->Result<Vec<\.T>,\s*VecPushError<\.T>>>/, 'Vec.push must return an owner-preserving VecPushError payload on failure');
assert.match(vecCode, /fn\s+vec_next_capacity\s+<\.T>\s+<\(i32\)->Result<i32,\s*StdErrorKind>>[\s\S]*size_of<\.T>[\s\S]*max_alloc_payload_bytes[\s\S]*StdErrorKind::CapacityExceeded[\s\S]*half_limit[\s\S]*Result<i32,\s*StdErrorKind>::Ok\s+mul\s+cap0\s+2/, 'Vec.push growth must prove next capacity against element size and allocator payload bounds before doubling');
assert.match(pushSection, /match\s+vec_next_capacity<\.T>\s+v_cap:[\s\S]*Result::Err\s+grow_error:[\s\S]*VecPushError<\.T>\s+\(Vec<\.T>\s+\(OwnedBuffer<\.T>\s+v_len\s+v_initialized_len\s+v_cap\s+v_storage\)\)\s+grow_error[\s\S]*Result::Ok\s+grown_cap:/, 'Vec.push must compute grow capacity through the checked capacity helper and return the Vec owner on grow proof failure');
assert.match(pushSection, /match\s+v_storage:[\s\S]*VecStorage::Empty:[\s\S]*alloc_region<\.T>\s+grown_cap[\s\S]*Result::Err<Vec<\.T>,\s*VecPushError<\.T>>\s+VecPushError<\.T>\s+\(Vec<\.T>\s+\(OwnedBuffer<\.T>\s+v_len\s+v_initialized_len\s+v_cap\s+VecStorage<\.T>::Empty\)\)\s+StdErrorKind::OutOfMemory[\s\S]*VecStorage::Owned\s+v_region:[\s\S]*vec_realloc_region_or_keep<\.T>\s+v_region\s+grown_cap/, 'Vec.push must return the consumed Vec owner through VecPushError on Empty allocation failure and keep Owned grow transfer in RegionToken form');
assert.match(pushSection, /OwnedBuffer<\.T>\s+next_len\s+next_len\s+(?:grown_cap|v_cap)\s+\(VecStorage<\.T>::Owned/, 'Vec.push success paths must advance initialized_len with len for the current Copy-only contract');
assert.match(popSection, /let\s+v_data\s+<MemPtr<\.T>>\s+vec_data::data_mem_ptr<\.T>\s+&v[\s\S]*let\s+v_buffer\s+<OwnedBuffer<\.T>>\s+field::get\s+v\s+"buffer"[\s\S]*let\s+v_storage\s+<VecStorage<\.T>>\s+field::get\s+v_buffer\s+"storage"/, 'Vec.pop must borrow a data view before moving the owner-carrying storage enum into the returned Vec');
assert.match(popSection, /let\s+v_initialized_len\s+<i32>\s+\*field::get_ref\s+v_buffer_ref\s+"initialized_len"[\s\S]*OwnedBuffer<\.T>\s+v_len\s+v_initialized_len\s+v_cap\s+v_storage/, 'Vec.pop empty path must preserve initialized_len separately from len');
assert.match(popSection, /vec_buffer_current_copy_invariant<\.T>\s+v_buffer_ref[\s\S]*not\s+v_invariant_ok[\s\S]*VecPop<\.T>\s+\(Vec<\.T>\s+\(OwnedBuffer<\.T>\s+v_len\s+v_initialized_len\s+v_cap\s+v_storage\)\)\s+none<\.T>/, 'Vec.pop must not raw-load from malformed OwnedBuffer metadata');
assert.match(popSection, /OwnedBuffer<\.T>\s+next_len\s+next_len\s+v_cap/, 'Vec.pop/drop_last success paths must update initialized_len with len under the current Copy-only contract');
assert.match(popSection, /fn\s+pop\s+<\.T:\s*Copy>\s+<\(Vec<\.T>\)->VecPop<\.T>>/, 'Vec.pop must return named VecPop and remain Copy-only until initialized slot move state exists');
assert.match(vecCode, /fn\s+vec_pop_vec\s+<\.T:\s*Copy>\s+<\(VecPop<\.T>\)->Vec<\.T>>/, 'Vec.pop Vec accessor must remain Copy-only because it discards the popped Option<T> payload');
assert.match(popSource, /diag_codes:\s*type\.trait_bound\.unsatisfied[\s\S]*struct\s+NonCopyPayload:[\s\S]*pop<NonCopyPayload>/, 'Vec.pop must reject non-Copy payloads until OwnedBuffer initialized cell move-out exists');
assert.match(clearSection, /fn\s+clear\s+<\.T:\s*Copy>\s+<\(Vec<\.T>\)->Vec<\.T>>/, 'Vec.clear must remain Copy-only until initialized element drop traversal exists');
assert.match(clearSection, /let\s+v_buffer\s+<OwnedBuffer<\.T>>\s+field::get\s+v\s+"buffer"[\s\S]*let\s+v_storage\s+<VecStorage<\.T>>\s+field::get\s+v_buffer\s+"storage"/, 'Vec.clear must explicitly move the owner-carrying storage enum through OwnedBuffer into the returned Vec');
assert.match(freeSection, /fn\s+free\s+<\.T:\s*Copy>\s+<\(Vec<\.T>\)->\(\)>/, 'Vec.free must remain Copy-only until initialized element drop traversal exists');
assert.match(freeSection, /let\s+v_buffer\s+<OwnedBuffer<\.T>>\s+field::get\s+v\s+"buffer"[\s\S]*let\s+v_storage\s+<VecStorage<\.T>>\s+field::get\s+v_buffer\s+"storage"[\s\S]*vec_free_storage<\.T>\s+v_storage/, 'Vec.free must pass the owner-carrying storage enum to cleanup');
assert.match(mapSection, /let\s+out_data\s+<MemPtr<\.U>>\s+vec_data::data_mem_ptr<\.U>\s+&out0[\s\S]*let\s+out_buffer\s+<OwnedBuffer<\.U>>\s+field::get\s+out0\s+"buffer"[\s\S]*let\s+out_storage\s+<VecStorage<\.U>>\s+field::get\s+out_buffer\s+"storage"/, 'Vec.map must borrow the output data view before moving the output owner into the returned Vec');
assert.match(vecCode, /struct\s+VecPushError<\.T>:[\s\S]*vec\s+<Vec<\.T>>[\s\S]*error\s+<StdErrorKind>/, 'Vec.push failure payload must carry the consumed Vec owner and a copyable error kind');
assert.match(vecMutationPushCode, /fn\s+vec_push_error_vec\s+<\.T:\s*Copy>\s+<\(VecPushError<\.T>\)->Vec<\.T>>/, 'Vec.push error owner accessor must remain Copy-only until non-Copy Vec drop traversal exists');
assert.match(vecMutationPushCode, /fn\s+vec_realloc_region_error_region\s+<\.T:\s*Copy>\s+<\(VecReallocRegionError<\.T>\)->RegionToken<\.T>>/, 'Vec grow internal region recovery accessor must remain Copy-only with the current push contract');
assert.match(vecCode, /struct\s+VecTransformError<\.T>:[\s\S]*vec\s+<Vec<\.T>>[\s\S]*error\s+<StdErrorKind>[\s\S]*fn\s+vec_transform_error_vec\s+<\.T:\s*Copy>\s+<\(VecTransformError<\.T>\)->Vec<\.T>>/, 'Vec transform failure payload must carry the consumed input Vec owner and expose a Copy-only owner-moving accessor');
assert.match(vecCode, /fn\s+vec_realloc_region_or_keep\s+<\.T:\s*Copy>[\s\S]*le\s+new_cap\s+0[\s\S]*max_alloc_payload_bytes[\s\S]*gt\s+new_cap\s+max_count[\s\S]*match\s+realloc_region_bytes_keep<\.T>\s+region\s+new_bytes:[\s\S]*Result::Ok\s+grown_region:[\s\S]*Result::Ok<RegionToken<\.T>,\s*VecReallocRegionError<\.T>>\s+grown_region[\s\S]*Result::Err\s+e:[\s\S]*VecReallocRegionError<\.T>\s+\(region_realloc_error_region<\.T>\s+e\)\s+StdErrorKind::OutOfMemory/, 'Vec.push grow helper must prove capacity bounds and return the old RegionToken owner through core/mem realloc failure payload');
assert.match(vecQueryGetCode, /vec_current_copy_invariant<\.T>\s+v[\s\S]*then\s+none<\.T>[\s\S]*load<\.T>/, 'Vec.get must prove current Copy-only invariant before raw load');
assert.match(vecMutationReplaceCode, /vec_current_copy_invariant<\.T>\s+v[\s\S]*then\s+\(\)[\s\S]*store<\.T>/, 'Vec.replace must prove current Copy-only invariant before raw store');
assert.match(vecTransformMapCode, /vec_buffer_current_copy_invariant<\.T>\s+v_buffer_ref[\s\S]*VecTransformError<\.T>\s+v\s+StdErrorKind::InvalidOperation[\s\S]*store<\.U>/, 'Vec.map must reject malformed input invariant before constructing a supposedly fully initialized output');
assert.match(vecTransformFilterCode, /fn\s+filter[\s\S]*vec_buffer_current_copy_invariant<\.T>\s+v_buffer_ref[\s\S]*VecTransformError<\.T>\s+v\s+StdErrorKind::InvalidOperation[\s\S]*store<\.T>/, 'Vec.filter must reject malformed input invariant before output raw writes');
assert.match(vecTransformFilterCode, /fn\s+partition[\s\S]*vec_buffer_current_copy_invariant<\.T>\s+v_buffer_ref[\s\S]*VecTransformError<\.T>\s+v\s+StdErrorKind::InvalidOperation[\s\S]*store<\.T>/, 'Vec.partition must reject malformed input invariant before output raw writes');
assert.match(vecTransformPrefixCode, /fn\s+take_while[\s\S]*vec_buffer_current_copy_invariant<\.T>\s+v_buffer_ref[\s\S]*VecTransformError<\.T>\s+v\s+StdErrorKind::InvalidOperation[\s\S]*vec_copy_range_to_raw/, 'Vec.take_while must reject malformed input invariant before raw range copy');
assert.match(vecTransformPrefixCode, /fn\s+drop_while[\s\S]*vec_buffer_current_copy_invariant<\.T>\s+v_buffer_ref[\s\S]*VecTransformError<\.T>\s+v\s+StdErrorKind::InvalidOperation[\s\S]*vec_copy_range_to_raw/, 'Vec.drop_while must reject malformed input invariant before raw range copy');
for (const [name, section, neutral] of [
    ['count', countSection, /then\s+0/],
    ['fold', foldSection, /then\s+acc/],
    ['reduce', reduceSection, /then\s+none<\.T>/],
    ['find', findSection, /then\s+none<\.T>/],
    ['any', anySection, /then\s+false/],
    ['all', allSection, /then\s+false/],
]) {
    assert.match(section, /vec_buffer_current_copy_invariant<\.T>\s+v_buffer/, `Vec.${name} must prove OwnedBuffer invariant before using len as a scan bound`);
    assert.match(section, neutral, `Vec.${name} must return a neutral non-success result for malformed OwnedBuffer metadata`);
    assert(
        section.search(/vec_buffer_current_copy_invariant<\.T>\s+v_buffer/) < section.search(/let\s+src_len\s+<i32>/),
        `Vec.${name} must not read len as a scan bound before proving the Vec invariant`,
    );
}
for (const [name, code] of [
    ['sort_quick', codeByPath.get('stdlib/alloc/collections/vec/sort/quick.nepl')],
    ['sort_quick_ret', codeByPath.get('stdlib/alloc/collections/vec/sort/quick.nepl')],
    ['sort_heap', codeByPath.get('stdlib/alloc/collections/vec/sort/heap.nepl')],
    ['sort_heap_ret', codeByPath.get('stdlib/alloc/collections/vec/sort/heap.nepl')],
    ['sort_merge', sortMergeApiCode],
    ['sort_merge_ret', sortMergeApiCode],
    ['sort_insertion', codeByPath.get('stdlib/alloc/collections/vec/sort/simple/insertion.nepl')],
    ['sort_gnome', codeByPath.get('stdlib/alloc/collections/vec/sort/simple/insertion.nepl')],
    ['sort_selection', codeByPath.get('stdlib/alloc/collections/vec/sort/simple/selection.nepl')],
    ['sort_bubble', codeByPath.get('stdlib/alloc/collections/vec/sort/simple/exchange.nepl')],
    ['sort_cocktail', codeByPath.get('stdlib/alloc/collections/vec/sort/simple/exchange.nepl')],
    ['sort_shell', codeByPath.get('stdlib/alloc/collections/vec/sort/simple/gap.nepl')],
    ['sort_comb', codeByPath.get('stdlib/alloc/collections/vec/sort/simple/gap.nepl')],
]) {
    const section = publicFunctionSection(code, name);
    const invariantIdx = section.search(/vec_current_copy_invariant<\.T>/);
    const rawIdx = section.search(/\b(?:data_mem_ptr|load|store|alloc_region)<\.T>|sort_\w+_range_data<\.T>/);
    assert.notEqual(invariantIdx, -1, `${name} must prove Vec invariant before raw sort traversal`);
    assert.notEqual(rawIdx, -1, `${name} source-policy test must identify its raw traversal boundary`);
    assert(
        invariantIdx < rawIdx,
        `${name} must prove Vec invariant before deriving a raw data view, loading/storing elements, or allocating sort scratch space`,
    );
}
assert.doesNotMatch(pushSection, /\b(?:let\s+grown_cap\s+<i32>\s+if\s+eq\s+v_cap\s+0\s+1\s+mul\s+v_cap\s+2|\bmul\s+v_cap\s+2\b)/, 'Vec.push must not compute unchecked cap*2 in the hot path');
assert.doesNotMatch(vecCode, /fn\s+vec_realloc_region_or_keep\s+<\.T:\s*Copy>[\s\S]*\brealloc_ptr<\.T>\b/, 'Vec grow helper must not reimplement raw MemPtr realloc outside core/mem');
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
assert.doesNotMatch(vecQueryAggregateCode + '\n' + vecQueryPredicateCode + '\n' + vecTransformCode, /\b(?:load<|vec_\w+_read_at)\b[\s\S]{0,80}\b(?:p|f)\b|\b(?:p|f)\s+(?:load<|vec_\w+_read_at)\b/, 'Vec callback-facing query/transform helpers must not pass raw-loaded elements directly to callbacks');
assert.doesNotMatch(vecTransformCode, /\bvec_\w+_write_at<\.[TU]>[\s\S]{0,80}\bvec_\w+_read_at<\.T>/, 'Vec transform helpers must not pipe raw-loaded elements directly into output storage');
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
assert.doesNotMatch(codeByPath.get('stdlib/alloc/collections/vec/sort.nepl'), /\bMemPtr\b|sort_i32|sort_slice_quick|sort_quick_range_data|sort_heap_sift_down_data|sort_merge_range_data|sort_buf_/, 'canonical sort facade must not expose raw MemPtr helper names');
for (const relPath of [
    'stdlib/alloc/collections/vec/sort/merge/buffer.nepl',
    'stdlib/alloc/collections/vec/sort/merge/buffer.n.md',
    'stdlib/alloc/collections/vec/sort/merge/range.nepl',
    'stdlib/alloc/collections/vec/sort/merge/range.n.md',
]) {
    assert.equal(fs.existsSync(path.join(repoRoot, relPath)), false, `${relPath} must not remain as a directly importable raw merge helper`);
}
assert.match(sortMergeApiCode, /fn\s+sort_merge_buffer_get\s+<\.T:\s*Copy>/, 'sort/merge/api.nepl must own private Copy-only scratch buffer loads');
assert.match(sortMergeApiCode, /fn\s+sort_merge_buffer_set\s+<\.T:\s*Copy>\s+<\(MemPtr<\.T>,i32,\.T\)\*>\(\)>/, 'sort/merge/api.nepl must own private Copy-only scratch buffer stores as an impure write');
for (const name of ['sort_merge_buffer_get', 'sort_merge_buffer_set', 'sort_merge_range_data']) {
    assert.doesNotMatch(sortMergeApiCode, new RegExp(`\\bpub\\s+fn\\s+${name}\\b`), `${name} must remain private to merge/api.nepl`);
}
assert.doesNotMatch(sortFamilyCode, /fn\s+sort_\w+\s+<\.T>\s+</, 'Vec sort raw load/store helpers must not be unconstrained over T');
assert.doesNotMatch(sortFamilyCode, /fn\s+sort_\w+\s+<\.T:\s*Ord>\s+</, 'Vec sort algorithms must require Ord&Copy until non-Copy move/drop-aware sorting exists');
assert.doesNotMatch(sortFamilyCode, /\bfn\s+sort_i32\b/, 'Vec sort must not expose the raw sort_i32 adapter through any sort module');
for (const name of ['sort_get_unchecked', 'sort_set_unchecked', 'sort_get_unchecked_data', 'sort_set_unchecked_data', 'sort_swap_data', 'sort_swap', 'sort_slice_quick']) {
    assert.doesNotMatch(sortFamilyCode, new RegExp(`\\b(?:pub\\s+)?fn\\s+${name}\\b`), `${name} must not reappear as a shared/direct-importable raw sort helper`);
}
for (const name of ['sort_quick_get_data', 'sort_quick_set_data', 'sort_quick_swap_data', 'sort_quick_partition_data', 'sort_quick_range_data', 'sort_heap_get_data', 'sort_heap_set_data', 'sort_heap_swap_data', 'sort_heap_sift_down_data']) {
    assert.doesNotMatch(sortFamilyCode, new RegExp(`\\bpub\\s+fn\\s+${name}\\b`), `${name} must remain private to the owning sort implementation file`);
}
for (const [name, signature] of [
    ['sort_quick_set_data', /<\(MemPtr<\.T>,i32,\.T\)\*>\(\)>/],
    ['sort_quick_swap_data', /<\(MemPtr<\.T>,i32,i32\)\*>\(\)>/],
    ['sort_quick_partition_data', /<\(MemPtr<\.T>,i32,i32\)\*>i32>/],
    ['sort_quick_range_data', /<\(MemPtr<\.T>,i32,i32\)\*>\(\)>/],
    ['sort_quick', /<\(&Vec<\.T>\)\*>\(\)>/],
    ['sort_quick_ret', /<\(Vec<\.T>\)\*>Vec<\.T>>/],
    ['sort', /<\(&Vec<\.T>\)\*>\(\)>/],
    ['sort_heap_set_data', /<\(MemPtr<\.T>,i32,\.T\)\*>\(\)>/],
    ['sort_heap_swap_data', /<\(MemPtr<\.T>,i32,i32\)\*>\(\)>/],
    ['sort_heap_sift_down_data', /<\(MemPtr<\.T>,i32,i32\)\*>\(\)>/],
    ['sort_heap', /<\(&Vec<\.T>\)\*>\(\)>/],
    ['sort_heap_ret', /<\(Vec<\.T>\)\*>Vec<\.T>>/],
    ['sort_merge_buffer_set', /<\(MemPtr<\.T>,i32,\.T\)\*>\(\)>/],
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
assert.match(sortMergeApiCode, /fn\s+sort_merge_range_data\s+<\.T:\s+Ord&Copy>\s+<\(MemPtr<\.T>,MemPtr<\.T>,i32,i32\)\*>\(\)>[\s\S]*sort_merge_buffer_set<\.T>[\s\S]*sort_merge_buffer_get<\.T>/, 'sort/merge/api.nepl must own private Copy-only impure range traversal and delegate scratch access');
assert.match(sortMergeApiCode, /pub\s+struct\s+VecSortMergeError<\.T>:[\s\S]*vec\s+<Vec<\.T>>[\s\S]*error\s+<StdErrorKind>/, 'sort_merge_ret failure payload must carry the consumed Vec owner and a copyable error kind');
assert.match(sortMergeApiCode, /fn\s+vec_sort_merge_error_vec\s+<\.T:\s*Copy>\s+<\(VecSortMergeError<\.T>\)->Vec<\.T>>/, 'sort_merge_ret error owner accessor must remain Copy-only until non-Copy sort/drop traversal exists');
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
