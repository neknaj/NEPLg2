# allocator coalescing

`core/mem` の allocator が[解放/かいほう]済みの[隣接/りんせつ] block を coalesce し、断片化した free list から[大/おお]きい[連続/れんぞく][領域/りょういき]を[再確保/さいかくほ]できることを[確認/かくにん]します。

## adjacent_blocks_merge_from_next

[目的/もくてき]:
- 後ろの block を先に free し、前の block を free したときに next 方向へ coalesce されることを[確認/かくにん]します。

neplg2:test
```neplg2
#target std
#entry main
#indent 4
#import "core/mem" as *
#import "core/math" as *

fn main <()*>i32> ():
    let a <i32> alloc_raw 8;
    let b <i32> alloc_raw 8;
    let c <i32> alloc_raw 8;
    dealloc_raw b 8;
    dealloc_raw a 8;
    let merged <i32> alloc_raw 24;
    let ok <bool> eq merged a;
    dealloc_raw c 8;
    dealloc_raw merged 24;
    if ok 0 1
```

## adjacent_blocks_merge_from_prev

[目的/もくてき]:
- 前の block を先に free し、後ろの block を free したときに prev 方向へ coalesce されることを[確認/かくにん]します。

neplg2:test
```neplg2
#target std
#entry main
#indent 4
#import "core/mem" as *
#import "core/math" as *

fn main <()*>i32> ():
    let a <i32> alloc_raw 8;
    let b <i32> alloc_raw 8;
    let c <i32> alloc_raw 8;
    dealloc_raw a 8;
    dealloc_raw b 8;
    let merged <i32> alloc_raw 24;
    let ok <bool> eq merged a;
    dealloc_raw c 8;
    dealloc_raw merged 24;
    if ok 0 1
```

## fragmentation_pattern_avoids_growth

[目的/もくてき]:
- page 末尾近くで 2 個の block を[連続/れんぞく]解放し、coalesce 後の[合計/ごうけい]サイズを使って大きい block を再確保できることを `mem_size` で[確認/かくにん]します。

neplg2:test
```neplg2
#target std
#entry main
#indent 4
#import "core/mem" as *
#import "core/math" as *

fn main <()*>i32> ():
    let pages0 <i32> mem_size;
    let cur_bytes <i32> mul pages0 65536;
    let heap0_raw <i32> load_i32 0;
    let heap0 <i32> align8 if lt heap0_raw 8 8 heap0_raw;
    let unit_size <i32> 1024;
    let unit_total <i32> align8 add unit_size 8;
    let merged_user_size <i32> sub mul unit_total 2 8;
    let filler_total <i32> sub sub cur_bytes heap0 mul unit_total 2;
    if:
        le filler_total 8
        then:
            1
        else:
            let filler_size <i32> sub filler_total 8;
            let filler <i32> alloc_raw filler_size;
            let a <i32> alloc_raw unit_size;
            let b <i32> alloc_raw unit_size;
            let pages_before <i32> mem_size;
            dealloc_raw b unit_size;
            dealloc_raw a unit_size;
            let merged <i32> alloc_raw merged_user_size;
            let pages_after <i32> mem_size;
            let ok_page <bool> eq pages_after pages_before;
            let ok_ptr <bool> eq merged a;
            dealloc_raw merged merged_user_size;
            dealloc_raw filler filler_size;
            if and ok_page ok_ptr 0 1
```
