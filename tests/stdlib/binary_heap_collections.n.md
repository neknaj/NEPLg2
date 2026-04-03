# tests/binary_heap_collections.n.md

## binary_heap_pipe_usage

[目的/もくてき]:
- `BinaryHeap` が bare API と `Result` / `Option` を[組/く]み[合/あ]わせた pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `push`
- `peek`
- `pop`
- `uwok`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/binary_heap" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let hp0 <BinaryHeap<i32>>:
        unwrap_ok<BinaryHeap<i32>, StdErrorKind> new<i32>
        |> push 3 |> uwok
        |> push 8 |> uwok
        |> push 5 |> uwok
    let ok0 <bool> match hp0 |> peek:
        Option::Some v:
            eq v 8
        Option::None:
            false
    let hp1 <BinaryHeap<i32>>:
        unwrap_ok<BinaryHeap<i32>, StdErrorKind> new<i32>
        |> push 3 |> uwok
        |> push 8 |> uwok
        |> push 5 |> uwok
    let ok1 <bool> match pop hp1:
        Option::Some v:
            eq v 8
        Option::None:
            false
    if and ok0 ok1 1 0
```
