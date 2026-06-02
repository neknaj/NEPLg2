# pipe + collections aliases

## pipe_list_alias_chain

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok]
    ##: [0] ok
    ##: [1] ok
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
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32 }

fn main %impure fn void i32 \void:
    let mut checks checks_new;
    let xs0 %List i32:
        unwrap_ok new
        |> push 3 |> uwok
        |> push 2 |> uwok
        |> push 1 |> uwok
    set checks checks_push checks check_eq_i32 3 len &xs0;
    free xs0;
    let xs1 %List i32:
        unwrap_ok new
        |> push 3 |> uwok
        |> push 2 |> uwok
        |> push 1 |> uwok
    match get &xs1 1:
        Option::Some v:
            set checks checks_push checks check_eq_i32 2 v
        Option::None:
            set checks checks_push checks Result::Err "pipe list get failed"
    free xs1;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## pipe_stack_alias_usage

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok]
    ##: [0] ok
    ##: [1] ok
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
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32 }

fn main %impure fn void i32 \void:
    let mut checks checks_new;
    let s0 %Stack i32:
        unwrap_ok new
        |> push 10 |> unwrap_ok
        |> push 20 |> unwrap_ok
    set checks checks_push checks check_eq_i32 2 len &s0;
    free s0;
    let s1 %Stack i32:
        unwrap_ok new
        |> push 10 |> unwrap_ok
        |> push 20 |> unwrap_ok
    let p pop s1;
    match p:
        Option::Some v:
            set checks checks_push checks check_eq_i32 20 v
        Option::None:
            set checks checks_push checks Result::Err "pipe stack pop failed"
    let shown checks_print_report checks;
    checks_exit_code shown
```

## pipe_btreemap_usage

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
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

fn must_map %impure fn Result BTreeMap i32 i32 BTreeMapInsertError i32 i32 BTreeMap i32 i32 \r:
    match r:
        Result::Ok m:
            m
        Result::Err e:
            let _d %Diag btreemap_insert_error_diag &e
            btreemap_insert_error_owner e

fn main %impure fn void i32 \void:
    let mut checks checks_new;
    let m0 %BTreeMap i32 i32:
        unwrap_ok new
        |> insert 3 30
        |> must_map
        |> insert 1 10
        |> must_map
    set checks checks_push checks check_eq_i32 2 len &m0;
    free m0;
    let m1 %BTreeMap i32 i32:
        unwrap_ok new
        |> insert 3 30
        |> must_map
        |> insert 1 10
        |> must_map
    match get &m1 3:
        Option::Some v:
            set checks checks_push checks check_eq_i32 30 v
        Option::None:
            set checks checks_push checks Result::Err "pipe btreemap get failed";
    free m1;
    let m2 %BTreeMap i32 i32:
        unwrap_ok new
        |> insert 3 30
        |> must_map
        |> insert 1 10
        |> must_map
    set checks checks_push checks check contains &m2 1;
    free m2;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## pipe_btreeset_usage

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *
#import "alloc/diag/error" as *
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32, check }
#import "core/result" as *
#import "core/math" as *

fn must_set %impure fn Result BTreeSet i32 BTreeSetInsertError i32 BTreeSet i32 \r:
    match r:
        Result::Ok s:
            s
        Result::Err e:
            let _d %Diag btreeset_insert_error_diag &e
            btreeset_insert_error_owner e

fn main %impure fn void i32 \void:
    let mut checks checks_new;
    let s0 %BTreeSet i32:
        unwrap_ok new
        |> insert 5
        |> must_set
        |> insert 2
        |> must_set
    set checks checks_push checks check contains &s0 5;
    free s0;
    let s1 %BTreeSet i32:
        unwrap_ok new
        |> insert 5
        |> must_set
        |> insert 2
        |> must_set
    set checks checks_push checks check_eq_i32 2 len &s1;
    free s1;
    let s2 %BTreeSet i32:
        unwrap_ok new
        |> insert 5
        |> must_set
        |> insert 2
        |> must_set
        |> remove 5
    set checks checks_push checks check not contains &s2 5;
    free s2;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## pipe_hashmap_usage

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
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

fn must_hm %impure fn Result HashMap i32 i32 DefaultHash32 Diag HashMap i32 i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hm %impure fn Result HashMap i32 i32 DefaultHash32 HashMapUpdateError i32 i32 DefaultHash32 HashMap i32 i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err e:
            let hm %HashMap i32 i32 DefaultHash32 hashmap_update_error_owner e;
            free hm;
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let mut checks checks_new;
    let hm0 %HashMap i32 i32 DefaultHash32:
        must_hm new DefaultHash32
        |> insert 7 70
        |> must_hm
        |> insert 9 90
        |> must_hm
    set checks checks_push checks check_eq_i32 2 len &hm0;
    free hm0;
    let hm1 %HashMap i32 i32 DefaultHash32:
        must_hm new DefaultHash32
        |> insert 7 70
        |> must_hm
        |> insert 9 90
        |> must_hm
    match get &hm1 9:
        Option::Some v:
            set checks checks_push checks check_eq_i32 90 v
        Option::None:
            set checks checks_push checks Result::Err "pipe hashmap get failed";
    free hm1;
    let hm2 %HashMap i32 i32 DefaultHash32:
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

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok]
    ##: [0] ok
    ##: [1] ok
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "core/traits/hash" as *
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32, check }
#import "alloc/diag/error" as *
#import "core/result" as *

fn must_hs %impure fn Result HashSet i32 DefaultHash32 Diag HashSet i32 DefaultHash32 \r:
    unwrap_ok r

fn must_hs %impure fn Result HashSet i32 DefaultHash32 HashSetUpdateError i32 DefaultHash32 HashSet i32 DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err e:
            let hs %HashSet i32 DefaultHash32 hashset_update_error_owner e;
            free hs;
            #intrinsic "unreachable" <> ()

fn new_hs %impure fn void Result HashSet i32 DefaultHash32 Diag \void:
    new DefaultHash32

fn main %impure fn void i32 \void:
    let mut checks checks_new;
    let hs0 %HashSet i32 DefaultHash32:
        new_hs
        |> must_hs
        |> insert 4
        |> must_hs
        |> insert 8
        |> must_hs
    set checks checks_push checks check_eq_i32 2 len &hs0;
    free hs0;
    let hs1 %HashSet i32 DefaultHash32:
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

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok]
    ##: [0] ok
    ##: [1] ok
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/ringbuffer" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32 }

fn main %impure fn void i32 \void:
    let mut checks checks_new;
    let rb %RingBuffer i32:
        unwrap_ok new
        |> push 11 |> uwok
        |> push 22 |> uwok
    set checks checks_push checks check_eq_i32 2 len &rb;
    free rb;
    let rb2 %RingBuffer i32:
        unwrap_ok new
        |> push 11 |> uwok
        |> push 22 |> uwok
    match peek &rb2:
        Option::Some v:
            set checks checks_push checks check_eq_i32 11 v
        Option::None:
            set checks checks_push checks Result::Err "pipe ringbuffer peek failed"
    free rb2;
    let shown checks_print_report checks;
    checks_exit_code shown
```

## pipe_queue_usage

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok]
    ##: [0] ok
    ##: [1] ok
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/queue" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as { checks_new, checks_push, checks_print_report, checks_exit_code, check_eq_i32 }

fn main %impure fn void i32 \void:
    let mut checks checks_new;
    let q %Queue i32:
        unwrap_ok new
        |> push 3 |> uwok
        |> push 4 |> uwok
    set checks checks_push checks check_eq_i32 2 len &q;
    free q;
    let q2 %Queue i32:
        unwrap_ok new
        |> push 3 |> uwok
        |> push 4 |> uwok
    match peek &q2:
        Option::Some v:
            set checks checks_push checks check_eq_i32 3 v
        Option::None:
            set checks checks_push checks Result::Err "pipe queue peek failed"
    free q2;
    let shown checks_print_report checks;
    checks_exit_code shown
```
