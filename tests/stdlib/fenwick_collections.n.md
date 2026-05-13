# tests/fenwick_collections.n.md

## fenwick_pipe_usage

[目的/もくてき]:
- `Fenwick` が bare API と `Result` を[組/く]み[合/あ]わせた pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `add`
- `len`
- `sum_prefix`
- `sum_range`
- `free`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/fenwick" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>i32> ():
    let fw <Fenwick>:
        unwrap_ok<Fenwick, Diag> new 6
        |> add 0 2 |> uwok
        |> add 2 5 |> uwok
        |> add 4 7 |> uwok
    let ok_len <bool> eq len &fw 6;
    let prefix5 <i32> unwrap_ok<i32, Diag> sum_prefix &fw 5;
    let ok0 <bool> eq prefix5 14;
    let range_2_5 <i32> unwrap_ok<i32, Diag> sum_range &fw 2 5;
    let ok1 <bool> eq range_2_5 12;
    free fw
    let ok <bool> and ok_len and ok0 ok1;
    if ok 1 0
```

## fenwick_free_releases_owned_storage

[目的/もくてき]:
- `Fenwick.free` が owner [管理/かんり]している 1-indexed `bit` [配列/はいれつ]を trap せず[解放/かいほう]し、その[後/あと]の[再確保/さいかくほ]で allocator が[継続/けいぞく]して[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `free`
- `new`
- `add`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/fenwick" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *

fn main <()*>i32> ():
    let fw0 <Fenwick>:
        unwrap_ok<Fenwick, Diag> new 6
        |> add 1 3 |> uwok
    free fw0
    let fw1 <Fenwick>:
        unwrap_ok<Fenwick, Diag> new 6
        |> add 2 5 |> uwok
    free fw1
    1
```

## fenwick_add_error_returns_owner

[目的/もくてき]:
- `add` の[範囲外/はんいがい] error が `Fenwick` owner を[失/うしな]わず、caller が[回収/かいしゅう]して `free` できることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `add`
- `add_error_tree`
- `free`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/fenwick" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *

fn main <()*>i32> ():
    let fw <Fenwick> unwrap_ok<Fenwick, Diag> new 4;
    match add fw 8 3:
        Result::Ok next:
            free next
            0
        Result::Err e:
            let recovered <Fenwick> add_error_tree e
            let ok <bool> eq len &recovered 4
            free recovered
            if ok 1 0
```

## fenwick_negative_length_rejected

[目的/もくてき]:
- `new` が[負/ふ]の length を allocator に[渡/わた]さず、typed `Diag` として[拒否/きょひ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `StdErrorKind::CapacityExceeded`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/fenwick" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/result" as *
#import "core/math" as *

fn main <()*>i32> ():
    let neg <i32> sub 0 1
    match new neg:
        Result::Ok fw:
            free fw
            0
        Result::Err d:
            let name <str> diag_std_error_kind_str d
            if str_eq name "CapacityExceeded" 1 0
```
