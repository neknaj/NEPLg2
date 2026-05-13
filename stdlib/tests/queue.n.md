# stdlib/queue.n.md

## queue_push_pop

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/queue" as *
#import "alloc/diag/error" as *
#import "core/field" as field
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *

fn main <()*>i32> ():
    let q0 <Queue<i32>>:
        unwrap_ok<Queue<i32>, Diag> new<i32>
        |> push<i32> 1
        |> unwrap_ok<Queue<i32>, Diag>
        |> push<i32> 2
        |> unwrap_ok<Queue<i32>, Diag>
    let ok0 <bool> eq len<i32> &q0 2;
    free<i32> q0;
    let q1 <Queue<i32>>:
        unwrap_ok<Queue<i32>, Diag> new<i32>
        |> push<i32> 1
        |> unwrap_ok<Queue<i32>, Diag>
        |> push<i32> 2
        |> unwrap_ok<Queue<i32>, Diag>
    let ok1 <bool> match peek<i32> &q1:
        Option::Some v:
            eq v 1
        Option::None:
            false
    free<i32> q1;
    let q2 <Queue<i32>>:
        unwrap_ok<Queue<i32>, Diag> new<i32>
        |> push<i32> 5
        |> unwrap_ok<Queue<i32>, Diag>
    let ok2 <bool> match pop<i32> q2:
        Option::Some v:
            eq v 5
        Option::None:
            false
    let q3 <Queue<i32>>:
        unwrap_ok<Queue<i32>, Diag> new<i32>
        |> push<i32> 7
        |> unwrap_ok<Queue<i32>, Diag>
        |> push<i32> 8
        |> unwrap_ok<Queue<i32>, Diag>
    let p0 <QueuePop<i32>> pop_front<i32> q3
    let ok3 <bool> match *field::get_ref &p0 "item":
        Option::Some v:
            eq v 7
        Option::None:
            false
    let q4 <Queue<i32>> field::get p0 "queue"
    let ok4 <bool>:
        match peek<i32> &q4:
            Option::Some v:
                and eq len<i32> &q4 1 eq v 8
            Option::None:
                false
    free<i32> q4;
    if and and ok0 ok1 and ok2 and ok3 ok4 1 0
```

## queue_pop_empty

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/queue" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let q <Queue<i32>> unwrap_ok<Queue<i32>, Diag> new<i32>;
    match pop<i32> q:
        Option::Some _:
            0
        Option::None:
            1
```
