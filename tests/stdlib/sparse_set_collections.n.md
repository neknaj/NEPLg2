# tests/sparse_set_collections.n.md

## sparse_set_pipe_usage

[目的/もくてき]:
- `SparseSet` が bare API と pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `insert`
- `remove`
- `contains`
- `len`
- `universe_len`
- `clear`
- `free`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/sparse_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let s0 <SparseSet>:
        unwrap_ok<SparseSet, Diag> new 12
        |> insert 1 |> uwok
        |> insert 5 |> uwok
        |> insert 9 |> uwok
        |> remove 5 |> uwok
    let ok0 <bool> unwrap_ok<bool, Diag> contains &s0 9;
    let ok1 <bool> not unwrap_ok<bool, Diag> contains &s0 5;
    let ok2 <bool> eq len &s0 2;
    let ok3 <bool> eq universe_len &s0 12;
    let s1 <SparseSet> clear s0;
    let ok4 <bool> eq len &s1 0;
    free s1
    if and and ok0 ok1 and ok2 and ok3 ok4 1 0
```

## sparse_set_clear_free_reallocates

[目的/もくてき]:
- `SparseSet` が insert/remove/clear 後に `free` しても trap せず、その後の[再確保/さいかくほ]が[正常/せいじょう]に[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `insert`
- `remove`
- `clear`
- `free`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/sparse_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let s_free <SparseSet>:
        unwrap_ok<SparseSet, Diag> new 12
        |> insert 1 |> uwok
        |> insert 5 |> uwok
        |> insert 9 |> uwok
        |> remove 5 |> uwok
        |> clear
    free s_free
    let s0 <SparseSet>:
        unwrap_ok<SparseSet, Diag> new 12
        |> insert 7 |> uwok
    let ok0 <bool> unwrap_ok<bool, Diag> contains &s0 7;
    free s0
    if ok0 1 0
```

## sparse_set_new_zero_is_empty

[目的/もくてき]:
- `new 0` が[空/から] domain の `SparseSet` として[成功/せいこう]し、`free` と[後続/こうぞく]の[再確保/さいかくほ]が[正常/せいじょう]に[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new 0`
- `universe_len`
- empty `contains` の範囲外 error
- `free`
- [再確保/さいかくほ]

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/sparse_set" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn main <()*>i32> ():
    let len_ok <bool> match new 0:
        Result::Err _e:
            false
        Result::Ok s:
            let ok <bool> eq universe_len &s 0;
            free s
            ok
    let contains_err_ok <bool> match new 0:
        Result::Err _e:
            false
        Result::Ok s:
            let r <Result<bool, Diag>> contains &s 0;
            free s
            match r:
                Result::Ok _v:
                    false
                Result::Err _e:
                    true
    let free_ok <bool> block:
        let empty <SparseSet> unwrap_ok<SparseSet, Diag> new 0
        free empty
        true
    let realloc_ok <bool> match new 2:
        Result::Err _e:
            false
        Result::Ok s0:
            match insert s0 1:
                Result::Err _e:
                    false
                Result::Ok s1:
                    let r <Result<bool, Diag>> contains &s1 1;
                    free s1
                    match r:
                        Result::Ok v:
                            v
                        Result::Err _e:
                            false
    if and and len_ok contains_err_ok and free_ok realloc_ok 1 0
```
