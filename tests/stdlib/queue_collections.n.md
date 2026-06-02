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

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"queue_pipe_usage\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"queue len\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"pop follows FIFO\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/queue" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let q %Queue i32:
        unwrap_ok new
        |> push 7
        |> uwok
        |> push 8
        |> uwok
    let size %i32 len &q;
    let ok1 %bool match q |> pop:
        Option::Some v:
            eq v 7
        Option::None:
            false
    let report:
        test_report_new "queue_pipe_usage"
        |> test_report_push assert_eq_i32 "queue len" 2 size
        |> test_report_push assert "pop follows FIFO" ok1
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## queue_grow_clear_and_free

[目的/もくてき]:
- capacity 1 からの `push` が grow 経路を[通/とお]り、old buffer cleanup と header 更新のあとも FIFO [順序/じゅんじょ]を[保/たも]つことを[確認/かくにん]します。
- `clear` と `free` が通常 cleanup として trap しないことを[確認/かくにん]します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"queue_grow_clear_and_free\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"len after grow\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"peek preserves FIFO\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"clear empties queue\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/queue" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let q0 %Queue i32 unwrap_ok with_capacity 1;
    let q1 %Queue i32 unwrap_ok push q0 10;
    let q2 %Queue i32 unwrap_ok push q1 20;
    let size %i32 len &q2;
    let ok_peek %bool match peek &q2:
        Option::Some v:
            eq v 10
        Option::None:
            false
    let q3 %Queue i32 clear q2;
    let ok_clear %bool is_empty &q3;
    free q3;
    let report:
        test_report_new "queue_grow_clear_and_free"
        |> test_report_push assert_eq_i32 "len after grow" 2 size
        |> test_report_push assert "peek preserves FIFO" ok_peek
        |> test_report_push assert "clear empties queue" ok_clear
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
