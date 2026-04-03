# tests/queue_collections.n.md

## queue_pipe_usage

[目的/もくてき]:
- `Queue` が `RingBuffer` の[上/うえ]に[構築/こうちく]された FIFO として、pipe [記法/きほう]で[自然/しぜん]に[使/つか]えることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `push`
- `len`
- `pop`
- `uwok`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/queue" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let q <Queue<i32>>:
        unwrap_ok<Queue<i32>, Diag> new<i32>
        |> push<i32> 7
        |> uwok
        |> push<i32> 8
        |> uwok
    let ok0 <bool> eq len<i32> q 2;
    let q2 <Queue<i32>>:
        unwrap_ok<Queue<i32>, Diag> new<i32>
        |> push<i32> 7
        |> uwok
        |> push<i32> 8
        |> uwok
    let ok1 <bool> match q2 |> pop<i32>:
        Option::Some v:
            eq v 7
        Option::None:
            false
    if and ok0 ok1 1 0
```
