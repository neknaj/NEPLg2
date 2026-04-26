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
    let ok0 <bool> eq len_ref<i32> &q 2;
    let ok1 <bool> match q |> pop<i32>:
        Option::Some v:
            eq v 7
        Option::None:
            false
    if and ok0 ok1 1 0
```

## queue_grow_clear_and_free

[目的/もくてき]:
- capacity 1 からの `push` が grow 経路を[通/とお]り、old buffer cleanup と header 更新のあとも FIFO [順序/じゅんじょ]を[保/たも]つことを[確認/かくにん]します。
- `clear` と `free` が通常 cleanup として trap しないことを[確認/かくにん]します。

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
    let q0 <Queue<i32>> unwrap_ok<Queue<i32>, Diag> with_capacity<i32> 1;
    let q1 <Queue<i32>> unwrap_ok<Queue<i32>, Diag> push<i32> q0 10;
    let q2 <Queue<i32>> unwrap_ok<Queue<i32>, Diag> push<i32> q1 20;
    let ok_len <bool> eq len_ref<i32> &q2 2;
    let ok_peek <bool> match peek_ref<i32> &q2:
        Option::Some v:
            eq v 10
        Option::None:
            false
    let q3 <Queue<i32>> clear<i32> q2;
    let ok_clear <bool> is_empty_ref<i32> &q3;
    free<i32> q3;
    if and ok_len and ok_peek ok_clear 1 0
```
