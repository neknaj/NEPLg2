# pipe + collections aliases

## pipe_list_alias_chain

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/list" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *

fn main <()*>i32> ():
    let xs0 <List<i32>>:
        unwrap_ok<List<i32>, Diag> new<i32>
        |> push<i32> 3 |> uwok
        |> push<i32> 2 |> uwok
        |> push<i32> 1 |> uwok
    let ok0 <bool> eq len<i32> &xs0 3;
    free<i32> xs0;
    let xs1 <List<i32>>:
        unwrap_ok<List<i32>, Diag> new<i32>
        |> push<i32> 3 |> uwok
        |> push<i32> 2 |> uwok
        |> push<i32> 1 |> uwok
    let ok1 <bool> match get<i32> &xs1 1:
        Option::Some v:
            eq v 2
        Option::None:
            false
    free<i32> xs1;
    if and ok0 ok1 1 0
```

## pipe_stack_alias_usage

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/field" as *
#import "core/result" as *

fn main <()*>i32> ():
    let s0 <Stack<i32>>:
        unwrap_ok<Stack<i32>, Diag> new<i32>
        |> push<i32> 10 |> unwrap_ok<Stack<i32>, Diag>
        |> push<i32> 20 |> unwrap_ok<Stack<i32>, Diag>
    let ok0 <bool> eq len<i32> &s0 2;
    free<i32> s0;
    let s1 <Stack<i32>>:
        unwrap_ok<Stack<i32>, Diag> new<i32>
        |> push<i32> 10 |> unwrap_ok<Stack<i32>, Diag>
        |> push<i32> 20 |> unwrap_ok<Stack<i32>, Diag>
    let p pop<i32> s1;
    let ok1 <bool> match p:
        Option::Some v:
            eq v 20
        Option::None:
            false
    if and ok0 ok1 1 0
```

## pipe_btreemap_usage

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "alloc/diag/error" as *
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32, check }
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *

fn must_map <(Result<BTreeMap<i32,i32>, Diag>)*>BTreeMap<i32,i32>> (r):
    match r:
        Result::Ok m:
            m
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let mut checks checks_new;
    let m0 <BTreeMap<i32,i32>>:
        new<i32,i32>
        |> must_map
        |> insert<i32,i32> 3 30
        |> must_map
        |> insert<i32,i32> 1 10
        |> must_map
    set checks checks_push checks check_eq_i32 2 len<i32,i32> &m0;
    free<i32,i32> m0;
    let m1 <BTreeMap<i32,i32>>:
        new<i32,i32>
        |> must_map
        |> insert<i32,i32> 3 30
        |> must_map
        |> insert<i32,i32> 1 10
        |> must_map
    match get<i32,i32> &m1 3:
        Option::Some v:
            set checks checks_push checks check_eq_i32 30 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "pipe btreemap get failed";
    free<i32,i32> m1;
    let m2 <BTreeMap<i32,i32>>:
        new<i32,i32>
        |> must_map
        |> insert<i32,i32> 3 30
        |> must_map
        |> insert<i32,i32> 1 10
        |> must_map
    set checks checks_push checks check contains<i32,i32> &m2 1;
    free<i32,i32> m2;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## pipe_btreeset_usage

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "alloc/diag/error" as *
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32, check }
#import "core/result" as *
#import "core/math" as *

fn must_set <(Result<BTreeSet<i32>, Diag>)*>BTreeSet<i32>> (r):
    match r:
        Result::Ok s:
            s
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let mut checks checks_new;
    let s0 <BTreeSet<i32>>:
        unwrap_ok<BTreeSet<i32>, Diag> new<i32>
        |> insert<i32> 5
        |> must_set
        |> insert<i32> 2
        |> must_set
    set checks checks_push checks check contains<i32> &s0 5;
    free<i32> s0;
    let s1 <BTreeSet<i32>>:
        unwrap_ok<BTreeSet<i32>, Diag> new<i32>
        |> insert<i32> 5
        |> must_set
        |> insert<i32> 2
        |> must_set
    set checks checks_push checks check_eq_i32 2 len<i32> &s1;
    free<i32> s1;
    let s2 <BTreeSet<i32>>:
        unwrap_ok<BTreeSet<i32>, Diag> new<i32>
        |> insert<i32> 5
        |> must_set
        |> insert<i32> 2
        |> must_set
        |> remove<i32> 5
    set checks checks_push checks check not contains<i32> &s2 5;
    free<i32> s2;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## pipe_hashmap_usage

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "core/traits/hash" as *
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32, check }
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *

fn must_hm <(Result<HashMap<i32,i32,DefaultHash32>, Diag>)*>HashMap<i32,i32,DefaultHash32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let mut checks checks_new;
    let hm0 <HashMap<i32,i32,DefaultHash32>>:
        must_hm new DefaultHash32
        |> insert 7 70
        |> must_hm
        |> insert 9 90
        |> must_hm
    set checks checks_push checks check_eq_i32 2 len &hm0;
    free hm0;
    let hm1 <HashMap<i32,i32,DefaultHash32>>:
        must_hm new DefaultHash32
        |> insert 7 70
        |> must_hm
        |> insert 9 90
        |> must_hm
    match get &hm1 9:
        Option::Some v:
            set checks checks_push checks check_eq_i32 90 v
        Option::None:
            set checks checks_push checks Result<(),str>::Err "pipe hashmap get failed";
    free hm1;
    let hm2 <HashMap<i32,i32,DefaultHash32>>:
        must_hm new DefaultHash32
        |> insert 7 70
        |> must_hm
        |> insert 9 90
        |> must_hm
    set checks checks_push checks check contains &hm2 7;
    free hm2;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## pipe_hashset_usage

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "core/traits/hash" as *
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32, check }
#import "alloc/diag/error" as *
#import "core/result" as *

fn must_hs <(Result<HashSet<i32,DefaultHash32>,Diag>)*>HashSet<i32,DefaultHash32>> (r):
    unwrap_ok<HashSet<i32,DefaultHash32>, Diag> r

fn new_hs <()*>Result<HashSet<i32,DefaultHash32>,Diag>> ():
    new DefaultHash32

fn main <()*>i32> ():
    let mut checks checks_new;
    let hs0 <HashSet<i32,DefaultHash32>>:
        new_hs
        |> must_hs
        |> insert 4
        |> must_hs
        |> insert 8
        |> must_hs
    set checks checks_push checks check_eq_i32 2 len &hs0;
    free hs0;
    let hs1 <HashSet<i32,DefaultHash32>>:
        new_hs
        |> must_hs
        |> insert 4
        |> must_hs
        |> insert 8
        |> must_hs
    set checks checks_push checks check contains &hs1 8;
    free hs1;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## pipe_ringbuffer_usage

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/ringbuffer" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let rb <RingBuffer<i32>>:
        unwrap_ok<RingBuffer<i32>, Diag> new<i32>
        |> push 11 |> uwok
        |> push 22 |> uwok
    let ok0 <bool> eq len<i32> &rb 2;
    free<i32> rb;
    let rb2 <RingBuffer<i32>>:
        unwrap_ok<RingBuffer<i32>, Diag> new<i32>
        |> push 11 |> uwok
        |> push 22 |> uwok
    let ok1 <bool> match peek<i32> &rb2:
        Option::Some v:
            eq v 11
        Option::None:
            false
    free<i32> rb2;
    if and ok0 ok1 1 0
```

## pipe_queue_usage

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
        |> push 3 |> uwok
        |> push 4 |> uwok
    let ok0 <bool> eq len<i32> &q 2;
    free<i32> q;
    let q2 <Queue<i32>>:
        unwrap_ok<Queue<i32>, Diag> new<i32>
        |> push 3 |> uwok
        |> push 4 |> uwok
    let ok1 <bool> match peek<i32> &q2:
        Option::Some v:
            eq v 3
        Option::None:
            false
    free<i32> q2;
    if and ok0 ok1 1 0
```
