# stdlib/deque.n.md

## deque_push_front_back

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
        |> push_back 10 |> uwok
        |> push_front 5 |> uwok
        |> push_back 20 |> uwok
    let ok0 <bool> eq len dq0 3;
    let dq1 <Deque<i32>>:
        unwrap_ok<Deque<i32>, Diag> new<i32>
        |> push_back 10 |> uwok
        |> push_front 5 |> uwok
    let ok1 <bool> match peek_front dq1:
        Option::Some v:
            eq v 5
        Option::None:
            false
    let dq2 <Deque<i32>>:
        unwrap_ok<Deque<i32>, Diag> new<i32>
        |> push_back 10 |> uwok
        |> push_back 20 |> uwok
    let ok2 <bool> match peek_back dq2:
        Option::Some v:
            eq v 20
        Option::None:
            false
    if and ok0 and ok1 ok2 1 0
```

## deque_pop_both_ends

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
    let dq_front <Deque<i32>>:
        unwrap_ok<Deque<i32>, Diag> new<i32>
        |> push_back 10 |> uwok
        |> push_back 20 |> uwok
    let ok0 <bool> match pop_front dq_front:
        Option::Some v:
            eq v 10
        Option::None:
            false
    let dq_back <Deque<i32>>:
        unwrap_ok<Deque<i32>, Diag> new<i32>
        |> push_back 10 |> uwok
        |> push_back 20 |> uwok
    let ok1 <bool> match pop_back dq_back:
        Option::Some v:
            eq v 20
        Option::None:
            false
    if and ok0 ok1 1 0
```
