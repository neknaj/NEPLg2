# tests/deque_collections.n.md

## deque_pipe_usage

[目的/もくてき]:
- `Deque` が pipe [記法/きほう]と `Result` / `Option` を[組/く]み[合/あ]わせた[基本的/きほんてき]な[使/つか]い[方/かた]で[利用/りよう]できることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `push_front`
- `push_back`
- `peek_front`
- `peek_back`
- `uwok`

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/deque" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let dq0 <Deque<i32>>:
        unwrap_ok<Deque<i32>, Diag> new<i32>
        |> push_back 7 |> uwok
        |> push_front 5 |> uwok
        |> push_back 9 |> uwok
    let ok0 <bool> eq len dq0 3;
    let dq1 <Deque<i32>>:
        unwrap_ok<Deque<i32>, Diag> new<i32>
        |> push_back 7 |> uwok
        |> push_front 5 |> uwok
        |> push_back 9 |> uwok
    let ok1 <bool> match dq1 |> peek_front:
        Option::Some v:
            eq v 5
        Option::None:
            false
    let dq2 <Deque<i32>>:
        unwrap_ok<Deque<i32>, Diag> new<i32>
        |> push_back 7 |> uwok
        |> push_front 5 |> uwok
        |> push_back 9 |> uwok
    let ok2 <bool> match dq2 |> peek_back:
        Option::Some v:
            eq v 9
        Option::None:
            false
    if and ok0 and ok1 ok2 1 0
```
