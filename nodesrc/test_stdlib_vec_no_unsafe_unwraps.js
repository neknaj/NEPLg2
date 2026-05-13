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
    /#intrinsic\s+"unreachable"/,
    /dealloc_ptr/,
];

for (const [relPath, code] of codeByPath) {
    for (const pattern of forbidden) {
        assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap or checked deallocation helpers in implementation code`);
    }
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

function between(code, start, end) {
    const startIdx = code.indexOf(start);
    assert.notEqual(startIdx, -1, `missing section start: ${start}`);
    const endIdx = code.indexOf(end, startIdx + start.length);
    assert.notEqual(endIdx, -1, `missing section end: ${end}`);
    return code.slice(startIdx, endIdx);
}

const pushSection = between(vecCode, 'fn push ', 'fn replace ');
const withCapacitySection = between(vecCode, 'fn with_capacity ', 'fn filled ');
const popSection = between(vecCode, 'fn pop ', 'fn clear ');
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
assert.match(withCapacitySection, /if:\s+lt\s+cap\s+0\s+then:\s+Result::Err<Vec<\.T>,\s*StdErrorKind>\s+StdErrorKind::InvalidOperation[\s\S]*else:\s+vec_alloc_empty<\.T>\s+cap/, 'Vec.with_capacity must reject negative capacity before allocating owned storage');
for (const name of ['types', 'storage', 'access', 'raw', 'mutation', 'query', 'transform', 'sort']) {
    assert.match(vecRootCode, new RegExp(`pub\\s+#import\\s+"\\.\\/vec\\/${name}"\\s+as\\s+@merge`), `Vec root must merge re-export vec/${name}.nepl`);
}
assert.doesNotMatch(vecRootCode, /\b(?:fn|struct|enum|trait)\s+\w+\b/, 'Vec root must be a pure facade without implementation bodies');
assert.doesNotMatch(vecRootCode, /\bas\s+vec_/, 'Vec root must not keep private delegation aliases after becoming a merge facade');
for (const name of ['VecStorageState', 'Vec', 'VecDataLen', 'VecPop', 'VecPartition']) {
    assert.doesNotMatch(vecRootCode, new RegExp(`(?:enum|struct)\\s+${name}\\b`), `Vec root must not own ${name}; it belongs in vec/types.nepl`);
    assert.match(vecTypesCode, new RegExp(`(?:enum|struct)\\s+${name}\\b`), `vec/types.nepl must own ${name}`);
}
for (const name of ['vec_empty', 'vec_alloc_empty', 'vec_storage_mem_ptr', 'vec_free_storage', 'new', 'with_capacity', 'filled']) {
    assert.match(vecStorageCode, new RegExp(`fn\\s+${name}\\b`), `vec/storage.nepl must own ${name}`);
}
for (const name of ['view', 'alloc', 'cleanup', 'fill']) {
    assert.match(vecStorageRootCode, new RegExp(`pub\\s+#import\\s+"\\.\\/storage\\/${name}"\\s+as\\s+@merge`), `vec/storage.nepl must merge re-export storage/${name}.nepl`);
}
assert.doesNotMatch(vecStorageRootCode, /\b(?:fn|struct|enum|trait)\s+\w+\b/, 'vec/storage.nepl must be a pure facade without implementation bodies');
for (const name of ['vec_empty', 'vec_storage_mem_ptr']) {
    assert.match(vecStorageViewCode, new RegExp(`fn\\s+${name}\\b`), `vec/storage/view.nepl must own ${name}`);
}
for (const name of ['vec_alloc_empty', 'new', 'with_capacity']) {
    assert.match(vecStorageAllocCode, new RegExp(`fn\\s+${name}\\b`), `vec/storage/alloc.nepl must own ${name}`);
}
assert.match(vecStorageCleanupCode, /\bfn\s+vec_free_storage\b/, 'vec/storage/cleanup.nepl must own vec_free_storage');
assert.match(vecStorageFillCode, /\bfn\s+filled\b/, 'vec/storage/fill.nepl must own filled');
for (const name of ['len', 'cap', 'data_ptr', 'data_mem_ptr', 'data_len', 'is_empty']) {
    assert.match(vecAccessCode, new RegExp(`fn\\s+${name}\\b`), `vec/access.nepl must own ${name}`);
}
for (const name of ['header', 'data']) {
    assert.match(vecAccessRootCode, new RegExp(`pub\\s+#import\\s+"\\.\\/access\\/${name}"\\s+as\\s+@merge`), `vec/access.nepl must merge re-export access/${name}.nepl`);
}
assert.doesNotMatch(vecAccessRootCode, /\b(?:fn|struct|enum|trait)\s+\w+\b/, 'vec/access.nepl must be a pure facade without implementation bodies');
for (const name of ['len', 'cap', 'is_empty']) {
    assert.match(vecAccessHeaderCode, new RegExp(`fn\\s+${name}\\b`), `vec/access/header.nepl must own ${name}`);
}
for (const name of ['data_ptr', 'data_mem_ptr', 'data_len']) {
    assert.match(vecAccessDataCode, new RegExp(`fn\\s+${name}\\b`), `vec/access/data.nepl must own ${name}`);
}
for (const name of ['vec_read_at', 'vec_write_at']) {
    assert.match(vecRawCode, new RegExp(`fn\\s+${name}\\b`), `vec/raw facade closure must expose ${name}`);
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
    'stdlib/alloc/collections/vec/sort/common.nepl',
    'stdlib/alloc/collections/vec/sort/merge/api.nepl',
    'stdlib/alloc/collections/vec/sort/merge/buffer.nepl',
]) {
    assert.match(readImplementation(relPath), rawBoundaryEvidencePattern, `${relPath} must carry source-level raw memory boundary evidence`);
}
assert.match(vecCode, /struct\s+Vec<\.T>:[\s\S]*storage\s+<VecStorageState>[\s\S]*data\s+<MemPtr<\.T>>/, 'Vec must separate enum owner state from the raw pointer field so Resource IR can track memory cells');
assert.match(vecCode, /fn\s+vec_empty\s+<\.T>\s+<\(\)->Vec<\.T>>[\s\S]*Vec<\.T>\s+0\s+0\s+VecStorageState::Empty\s+mem_ptr_wrap\s+0/, 'Vec.empty must construct typed Empty storage');
assert.match(vecCode, /fn\s+vec_alloc_empty\s+<\.T>\s+<\(i32\)->Result<Vec<\.T>,\s*StdErrorKind>>[\s\S]*le\s+requested_cap\s+0[\s\S]*vec_empty<\.T>[\s\S]*VecStorageState::Owned\s+data/, 'Vec empty construction must use Empty for zero capacity and Owned for allocated storage');
assert.match(vecCode, /fn\s+vec_free_storage\s+<\.T>[\s\S]*match\s+storage:[\s\S]*VecStorageState::Empty:[\s\S]*\(\)[\s\S]*VecStorageState::Owned:[\s\S]*dealloc_raw\s+mem_ptr_addr\s+data\s+mul\s+cap\s+size_of<\.T>/, 'Vec.free must deallocate through exhaustive VecStorageState matching');
assert.match(withCapacitySection, /vec_alloc_empty<\.T>\s+cap/, 'Vec.with_capacity must delegate empty storage allocation to vec_alloc_empty');
assert.doesNotMatch(vecCode, /(?:->|Result<)\.Pair\b|Tuple:/, 'Vec must not return owner-carrying Vec pairs through anonymous .Pair/Tuple values');
assert.match(vecCode, /struct\s+VecPop<\.T>:[\s\S]*vec\s+<Vec<\.T>>[\s\S]*item\s+<Option<\.T>>/, 'Vec.pop result must be a named struct with an owned Vec field');
assert.match(vecCode, /struct\s+VecPartition<\.T>:[\s\S]*matched\s+<Vec<\.T>>[\s\S]*rest\s+<Vec<\.T>>/, 'Vec.partition result must be a named struct with both owned Vec fields');
for (const relPath of walkNeplFiles(path.join(repoRoot, 'stdlib'))) {
    assert.doesNotMatch(readImplementation(relPath), /\b(?:[a-zA-Z_][\w]*::)?Vec<[^>\n]+>\s+0\s+0\s+mem_ptr_wrap\s+0/, `${relPath} must use Vec.empty typed storage instead of raw null owner sentinel`);
}
assert.match(pushSection, /let\s+v_storage\s+<VecStorageState>\s+\*field::get_ref\s+&v\s+"storage"[\s\S]*let\s+v_data\s+<MemPtr<\.T>>\s+field::get\s+v\s+"data"/, 'Vec.push must read typed storage state before moving the data owner from the consumed input Vec');
assert.match(pushSection, /match\s+v_storage:[\s\S]*VecStorageState::Empty:[\s\S]*alloc_ptr<\.T>\s+new_bytes[\s\S]*VecStorageState::Owned\s+grown_data[\s\S]*VecStorageState::Owned:[\s\S]*realloc_ptr<\.T>\s+v_data\s+old_bytes\s+new_bytes/, 'Vec.push must use match over Empty/Owned storage when growing');
assert.match(popSection, /let\s+v_storage\s+<VecStorageState>\s+\*field::get_ref\s+&v\s+"storage"[\s\S]*let\s+v_data\s+<MemPtr<\.T>>\s+field::get\s+v\s+"data"/, 'Vec.pop must move the data owner and typed storage state into the returned Vec');
assert.match(popSection, /fn\s+pop\s+<\.T>\s+<\(Vec<\.T>\)->VecPop<\.T>>/, 'Vec.pop must return named VecPop so callers can free the returned Vec owner');
assert.match(clearSection, /let\s+v_storage\s+<VecStorageState>\s+\*field::get_ref\s+&v\s+"storage"[\s\S]*let\s+v_data\s+<MemPtr<\.T>>\s+field::get\s+v\s+"data"/, 'Vec.clear must explicitly move the data owner into the returned Vec with its storage state');
assert.match(freeSection, /let\s+v_storage\s+<VecStorageState>\s+\*field::get_ref\s+&v\s+"storage"[\s\S]*let\s+v_data\s+<MemPtr<\.T>>\s+field::get\s+v\s+"data"[\s\S]*vec_free_storage<\.T>\s+v_storage\s+v_data\s+v_cap/, 'Vec.free must explicitly move data before freeing through VecStorageState');
assert.match(mapSection, /let\s+out_storage\s+<VecStorageState>\s+\*field::get_ref\s+&out0\s+"storage"[\s\S]*let\s+out_data\s+<MemPtr<\.U>>\s+field::get\s+out0\s+"data"/, 'Vec.map must explicitly move the output data owner from the allocated Vec into the returned Vec');
assert.match(vecCode, /fn\s+push\s+<\.T>\s+<\(Vec<\.T>,\.T\)->Result<Vec<\.T>,\s*StdErrorKind>>\s+\(v,\s*item\):[\s\S]*match\s+realloc_ptr<\.T>\s+v_data\s+old_bytes\s+new_bytes:[\s\S]*Result::Err\s+_e:[\s\S]*dealloc_raw\s+mem_ptr_addr\s+v_data\s+old_bytes[\s\S]*Result::Err<Vec<\.T>,\s*StdErrorKind>\s+StdErrorKind::OutOfMemory/, 'Vec.push must release the consumed old buffer when grow fails');
assert.match(vecCode, /Result::Err\s+e:[\s\S]*vec_cleanup::free<\.T>\s+left0[\s\S]*vec_cleanup::free<\.T>\s+v[\s\S]*Result::Err<VecPartition<\.T>,\s*StdErrorKind>\s+e/, 'Vec.partition right allocation failure must free whole Vec owners instead of splitting storage fields at the call site');
assert.doesNotMatch(vecTransformFilterCode, /\bleft0_(?:cap|storage|data)\b/, 'Vec.partition must not reintroduce left0 storage field splitting for cleanup');
assert.match(vecCode, /fn\s+partition\s+<\.T:\s*Copy>\s+<\(Vec<\.T>,\s*\(\.T\)->bool\)->Result<VecPartition<\.T>,\s*StdErrorKind>>/, 'Vec.partition must return named VecPartition and require Copy elements for safe predicate scans');
assert.match(countSection, /fn\s+count\s+<\.T:\s+Copy>\s+<\(&Vec<\.T>,\s*\(\.T\)->bool\)->i32>/, 'Vec.count must be a borrowed observer so callers retain and free the Vec owner');
assert.match(foldSection, /fn\s+fold\s+<\.T:\s+Copy,\s*\.U>\s+<\(&Vec<\.T>,\s*\.U,\s*\(\.U,\.T\)->\.U\)->\.U>/, 'Vec.fold must borrow the Vec owner and copy elements into the reducer');
assert.match(reduceSection, /fn\s+reduce\s+<\.T:\s+Copy>\s+<\(&Vec<\.T>,\s*\(\.T,\.T\)->\.T\)->Option<\.T>>/, 'Vec.reduce must borrow the Vec owner and require Copy elements');
assert.match(findSection, /fn\s+find\s+<\.T:\s+Copy>\s+<\(&Vec<\.T>,\s*\(\.T\)->bool\)->Option<\.T>>/, 'Vec.find must borrow the Vec owner and require Copy elements');
assert.match(anySection, /fn\s+any\s+<\.T:\s+Copy>\s+<\(&Vec<\.T>,\s*\(\.T\)->bool\)->bool>/, 'Vec.any must borrow the Vec owner and require Copy elements');
assert.match(allSection, /fn\s+all\s+<\.T:\s+Copy>\s+<\(&Vec<\.T>,\s*\(\.T\)->bool\)->bool>/, 'Vec.all must borrow the Vec owner and require Copy elements');
assert.match(mapSection, /fn\s+map\s+<\.T:\s*Copy,\s*\.U:\s*Copy>/, 'Vec.map must require Copy input and output elements until non-Copy element drop traversal exists');
assert.match(vecTransformFilterCode, /fn\s+filter\s+<\.T:\s*Copy>/, 'Vec.filter must require Copy elements for predicate scans and output copy');
assert.match(vecTransformPrefixCode, /fn\s+take_while\s+<\.T:\s*Copy>/, 'Vec.take_while must require Copy elements for prefix scans and output copy');
assert.match(vecTransformPrefixCode, /fn\s+drop_while\s+<\.T:\s*Copy>/, 'Vec.drop_while must require Copy elements for prefix scans and output copy');
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
for (const name of ['buffer', 'range', 'api']) {
    assert.match(sortMergeRootCode, new RegExp(`pub\\s+#import\\s+"\\.\\/merge\\/${name}"\\s+as\\s+\\*`), `sort/merge.nepl must re-export merge/${name}.nepl`);
}
assert.doesNotMatch(sortMergeRootCode, /\b(?:fn|struct|enum|trait)\s+\w+\b/, 'sort/merge.nepl must be a pure facade without implementation bodies');
assert.match(sortMergeBufferCode, /fn\s+sort_buf_get\s+<\.T>/, 'sort/merge/buffer.nepl must own scratch buffer loads');
assert.match(sortMergeBufferCode, /fn\s+sort_buf_set\s+<\.T>/, 'sort/merge/buffer.nepl must own scratch buffer stores');
assert.match(sortMergeRangeCode, /fn\s+sort_merge_range_data\s+<\.T:\s+Ord>[\s\S]*sort_buf_set<\.T>[\s\S]*sort_buf_get<\.T>/, 'sort/merge/range.nepl must own range traversal and delegate scratch access');
assert.match(sortMergeApiCode, /fn\s+sort_merge\s+<\.T:\s+Ord>[\s\S]*dealloc_raw\s+mem_ptr_addr\s+buf\s+mul\s+n\s+size_of<\.T>[\s\S]*Result<\(\),\s*StdErrorKind>::Ok\s+\(\)/, 'sort_merge must release scratch buffer with raw owner cleanup');
assert.match(sortMergeApiCode, /fn\s+sort_merge_ret\s+<\.T:\s+Ord>[\s\S]*let\s+storage\s+<VecStorageState>\s+get\s+v\s+"storage"[\s\S]*let\s+data_ptr\s+<MemPtr<\.T>>\s+get\s+v\s+"data"[\s\S]*dealloc_raw\s+mem_ptr_addr\s+buf\s+mul\s+n\s+size_of<\.T>[\s\S]*Result<Vec<\.T>,\s*StdErrorKind>::Ok\s+Vec<\.T>\s+n\s+cap\s+storage\s+data_ptr/, 'sort_merge_ret must release scratch buffer and return the original Vec storage state and data owner');

console.log('vec unsafe unwrap regression passed');
