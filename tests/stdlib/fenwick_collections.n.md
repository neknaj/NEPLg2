# tests/fenwick_collections.n.md

## fenwick_pipe_usage

[目的/もくてき]:
- `Fenwick` が bare API と `Result` を[組/く]み[合/あ]わせた pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `add`
- `sum_prefix`
- `sum_range`

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
    let fw_prefix <Fenwick>:
        unwrap_ok<Fenwick, Diag> new 6
        |> add 0 2 |> uwok
        |> add 2 5 |> uwok
        |> add 4 7 |> uwok
    let prefix5 <i32> unwrap_ok<i32, Diag> sum_prefix fw_prefix 5;
    let ok0 <bool> eq prefix5 14;
    let fw_range <Fenwick>:
        unwrap_ok<Fenwick, Diag> new 6
        |> add 0 2 |> uwok
        |> add 2 5 |> uwok
        |> add 4 7 |> uwok
    let range_2_5 <i32> unwrap_ok<i32, Diag> sum_range fw_range 2 5;
    let ok1 <bool> eq range_2_5 12;
    let ok <bool> and ok0 ok1;
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
