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
    let ok0 <bool> eq len<i32> &dq0 3;
    let ok1 <bool> match peek_front<i32> &dq0:
        Option::Some v:
            eq v 5
        Option::None:
            false
    let ok2 <bool> match peek_back<i32> &dq0:
        Option::Some v:
            eq v 9
        Option::None:
            false
    let result <i32> if and ok0 and ok1 ok2 1 0
    free<i32> dq0
    result
```

## deque_grow_clear_and_free

[目的/もくてき]:
- capacity 1 から `push_front` / `push_back` の grow 経路を[通/とお]り、old buffer cleanup と header 更新のあとも[両端/りょうたん]の[順序/じゅんじょ]を[保/たも]つことを[確認/かくにん]します。
- `clear` と `free` が通常 cleanup として trap しないことを[確認/かくにん]します。

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
    let dq0 <Deque<i32>> unwrap_ok<Deque<i32>, Diag> with_capacity<i32> 1;
    let dq1 <Deque<i32>> unwrap_ok<Deque<i32>, Diag> push_back<i32> dq0 10;
    let dq2 <Deque<i32>> unwrap_ok<Deque<i32>, Diag> push_front<i32> dq1 5;
    let dq3 <Deque<i32>> unwrap_ok<Deque<i32>, Diag> push_back<i32> dq2 20;
    let ok_len <bool> eq len<i32> &dq3 3;
    let ok_front <bool> match peek_front<i32> &dq3:
        Option::Some v:
            eq v 5
        Option::None:
            false
    let ok_back <bool> match peek_back<i32> &dq3:
        Option::Some v:
            eq v 20
        Option::None:
            false
    let dq4 <Deque<i32>> clear<i32> dq3;
    let ok_clear <bool> is_empty<i32> &dq4;
    free<i32> dq4;
    if and ok_len and ok_front and ok_back ok_clear 1 0
```
