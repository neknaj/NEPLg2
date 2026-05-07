#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPaths = [
    'stdlib/alloc/collections/vec.nepl',
    'stdlib/alloc/collections/vec/types.nepl',
    'stdlib/alloc/collections/vec/storage.nepl',
    'stdlib/alloc/collections/vec/access.nepl',
    'stdlib/alloc/collections/vec/raw.nepl',
    'stdlib/alloc/collections/vec/sort.nepl',
    'stdlib/alloc/collections/vec/sort/common.nepl',
    'stdlib/alloc/collections/vec/sort/simple.nepl',
    'stdlib/alloc/collections/vec/sort/quick.nepl',
    'stdlib/alloc/collections/vec/sort/heap.nepl',
    'stdlib/alloc/collections/vec/sort/merge.nepl',
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
const vecStorageCode = codeByPath.get('stdlib/alloc/collections/vec/storage.nepl');
const vecAccessCode = codeByPath.get('stdlib/alloc/collections/vec/access.nepl');
const vecRawCode = codeByPath.get('stdlib/alloc/collections/vec/raw.nepl');
const vecCode = [vecTypesCode, vecStorageCode, vecAccessCode, vecRawCode, vecRootCode].join('\n');
const sortMergeCode = codeByPath.get('stdlib/alloc/collections/vec/sort/merge.nepl');
const loaderCode = fs.readFileSync(path.join(repoRoot, 'nepl-core/src/loader.rs'), 'utf8');

function between(code, start, end) {
    const startIdx = code.indexOf(start);
    assert.notEqual(startIdx, -1, `missing section start: ${start}`);
    const endIdx = code.indexOf(end, startIdx + start.length);
    assert.notEqual(endIdx, -1, `missing section end: ${end}`);
    return code.slice(startIdx, endIdx);
}

const pushSection = between(vecCode, 'fn push ', 'fn get ');
const withCapacitySection = between(vecCode, 'fn with_capacity ', 'fn filled ');
const popSection = between(vecCode, 'fn pop ', 'fn clear ');
const clearSection = between(vecCode, 'fn clear ', 'fn vec_read_at ');
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
assert.match(vecRootCode, /pub\s+#import\s+"\.\/vec\/types"\s+as\s+\*/, 'Vec root must re-export the types module');
assert.match(vecRootCode, /#import\s+"\.\/vec\/storage"\s+as\s+vec_storage/, 'Vec root must delegate storage helpers to vec/storage.nepl');
assert.match(vecRootCode, /#import\s+"\.\/vec\/access"\s+as\s+vec_access/, 'Vec root must delegate observer helpers to vec/access.nepl');
assert.match(vecRootCode, /#import\s+"\.\/vec\/raw"\s+as\s+vec_raw/, 'Vec root must delegate raw storage helpers to vec/raw.nepl');
for (const name of ['VecStorageState', 'Vec', 'VecDataLen', 'VecPop', 'VecPartition']) {
    assert.doesNotMatch(vecRootCode, new RegExp(`(?:enum|struct)\\s+${name}\\b`), `Vec root must not own ${name}; it belongs in vec/types.nepl`);
    assert.match(vecTypesCode, new RegExp(`(?:enum|struct)\\s+${name}\\b`), `vec/types.nepl must own ${name}`);
}
for (const name of ['vec_empty', 'vec_alloc_empty', 'vec_storage_mem_ptr', 'vec_free_storage', 'new', 'with_capacity', 'filled']) {
    assert.match(vecStorageCode, new RegExp(`fn\\s+${name}\\b`), `vec/storage.nepl must own ${name}`);
}
for (const name of ['len', 'cap', 'data_ptr', 'data_mem_ptr', 'data_len', 'is_empty']) {
    assert.match(vecAccessCode, new RegExp(`fn\\s+${name}\\b`), `vec/access.nepl must own ${name}`);
}
for (const name of ['vec_read_at', 'vec_write_at', 'vec_fold_impl', 'vec_reduce_impl', 'vec_find_impl', 'vec_take_while_len_impl', 'vec_write_prefix_impl']) {
    assert.match(vecRawCode, new RegExp(`fn\\s+${name}\\b`), `vec/raw.nepl must own ${name}`);
}
for (const [name, target] of [
    ['vec_empty', 'vec_storage::vec_empty<\\.T>'],
    ['vec_alloc_empty', 'vec_storage::vec_alloc_empty<\\.T>\\s+requested_cap'],
    ['vec_storage_mem_ptr', 'vec_storage::vec_storage_mem_ptr<\\.T>\\s+storage\\s+data'],
    ['vec_free_storage', 'vec_storage::vec_free_storage<\\.T>\\s+storage\\s+data\\s+cap'],
    ['new', 'vec_storage::new<\\.T>'],
    ['with_capacity', 'vec_storage::with_capacity<\\.T>\\s+cap'],
    ['filled', 'vec_storage::filled<\\.T>\\s+n\\s+value'],
]) {
    assert.match(vecRootCode, new RegExp(`fn\\s+${name}\\b[\\s\\S]*?${target}`), `Vec root ${name} must be a thin storage facade wrapper`);
}
for (const [name, target] of [
    ['len', 'vec_access::len<\\.T>\\s+v'],
    ['cap', 'vec_access::cap<\\.T>\\s+v'],
    ['data_ptr', 'vec_access::data_ptr<\\.T>\\s+v'],
    ['data_mem_ptr', 'vec_access::data_mem_ptr<\\.T>\\s+v'],
    ['data_len', 'vec_access::data_len<\\.T>\\s+v'],
    ['is_empty', 'vec_access::is_empty<\\.T>\\s+v'],
]) {
    assert.match(vecRootCode, new RegExp(`fn\\s+${name}\\b[\\s\\S]*?${target}`), `Vec root ${name} must be a thin access facade wrapper`);
}
for (const [name, target] of [
    ['vec_read_at', 'vec_raw::vec_read_at<\\.T>\\s+data\\s+idx'],
    ['vec_write_at', 'vec_raw::vec_write_at<\\.T>\\s+data\\s+idx\\s+item'],
    ['vec_fold_impl', 'vec_raw::vec_fold_impl<\\.T,\\.U>\\s+data\\s+len\\s+idx\\s+acc\\s+f'],
    ['vec_reduce_impl', 'vec_raw::vec_reduce_impl<\\.T>\\s+data\\s+len\\s+idx\\s+acc\\s+f'],
    ['vec_find_impl', 'vec_raw::vec_find_impl<\\.T>\\s+data\\s+len\\s+idx\\s+p'],
    ['vec_take_while_len_impl', 'vec_raw::vec_take_while_len_impl<\\.T>\\s+data\\s+len\\s+idx\\s+p'],
    ['vec_write_prefix_impl', 'vec_raw::vec_write_prefix_impl<\\.T>\\s+src_data\\s+out_data\\s+src_from\\s+count'],
]) {
    assert.match(vecRootCode, new RegExp(`fn\\s+${name}\\b[\\s\\S]*?${target}`), `Vec root ${name} must be a thin raw helper facade wrapper`);
}
assert.match(vecCode, /enum\s+VecStorageState:[\s\S]*Empty[\s\S]*Owned/, 'Vec storage owner state must be represented by an enum');
for (const relPath of [
    /&\["alloc",\s*"collections",\s*"vec\.nepl"\]/,
    /&\["alloc",\s*"collections",\s*"vec",\s*"access\.nepl"\]/,
    /&\["alloc",\s*"collections",\s*"vec",\s*"raw\.nepl"\]/,
    /&\["alloc",\s*"collections",\s*"vec",\s*"storage\.nepl"\]/,
    /&\["alloc",\s*"collections",\s*"vec",\s*"types\.nepl"\]/,
    /&\["alloc",\s*"collections",\s*"vec",\s*"sort",\s*"common\.nepl"\]/,
    /&\["alloc",\s*"collections",\s*"vec",\s*"sort",\s*"merge\.nepl"\]/,
]) {
    assert.match(loaderCode, relPath, 'loader raw-memory boundary must include Vec root and exact submodule stdlib paths');
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
assert.match(vecCode, /vec_free_storage<\.T>\s+left0_storage\s+left0_data\s+left0_cap/, 'Vec.partition cleanup must use VecStorageState cleanup for the left buffer after right allocation failure');
assert.match(vecCode, /fn\s+partition\s+<\.T>\s+<\(Vec<\.T>,\s*\(\.T\)->bool\)->Result<VecPartition<\.T>,\s*StdErrorKind>>/, 'Vec.partition must return named VecPartition instead of anonymous owner pairs');
assert.match(countSection, /fn\s+count\s+<\.T:\s+Copy>\s+<\(&Vec<\.T>,\s*\(\.T\)->bool\)->i32>/, 'Vec.count must be a borrowed observer so callers retain and free the Vec owner');
assert.match(foldSection, /fn\s+fold\s+<\.T:\s+Copy,\s*\.U>\s+<\(&Vec<\.T>,\s*\.U,\s*\(\.U,\.T\)->\.U\)->\.U>/, 'Vec.fold must borrow the Vec owner and copy elements into the reducer');
assert.match(reduceSection, /fn\s+reduce\s+<\.T:\s+Copy>\s+<\(&Vec<\.T>,\s*\(\.T,\.T\)->\.T\)->Option<\.T>>/, 'Vec.reduce must borrow the Vec owner and require Copy elements');
assert.match(findSection, /fn\s+find\s+<\.T:\s+Copy>\s+<\(&Vec<\.T>,\s*\(\.T\)->bool\)->Option<\.T>>/, 'Vec.find must borrow the Vec owner and require Copy elements');
assert.match(anySection, /fn\s+any\s+<\.T:\s+Copy>\s+<\(&Vec<\.T>,\s*\(\.T\)->bool\)->bool>/, 'Vec.any must borrow the Vec owner and require Copy elements');
assert.match(allSection, /fn\s+all\s+<\.T:\s+Copy>\s+<\(&Vec<\.T>,\s*\(\.T\)->bool\)->bool>/, 'Vec.all must borrow the Vec owner and require Copy elements');
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
assert.match(sortMergeCode, /fn\s+sort_merge\s+<\.T:\s+Ord>[\s\S]*dealloc_raw\s+mem_ptr_addr\s+buf\s+mul\s+n\s+size_of<\.T>[\s\S]*Result<\(\),\s*StdErrorKind>::Ok\s+\(\)/, 'sort_merge must release scratch buffer with raw owner cleanup');
assert.match(sortMergeCode, /fn\s+sort_merge_ret\s+<\.T:\s+Ord>[\s\S]*let\s+storage\s+<VecStorageState>\s+get\s+v\s+"storage"[\s\S]*let\s+data_ptr\s+<MemPtr<\.T>>\s+get\s+v\s+"data"[\s\S]*dealloc_raw\s+mem_ptr_addr\s+buf\s+mul\s+n\s+size_of<\.T>[\s\S]*Result<Vec<\.T>,\s*StdErrorKind>::Ok\s+Vec<\.T>\s+n\s+cap\s+storage\s+data_ptr/, 'sort_merge_ret must release scratch buffer and return the original Vec storage state and data owner');

console.log('vec unsafe unwrap regression passed');
