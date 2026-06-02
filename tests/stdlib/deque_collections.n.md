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

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"deque_pipe_usage\" count=3 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"deque len\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"front item\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"back item\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/deque" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let dq0 %Deque i32:
        unwrap_ok new
        |> push_back 7 |> uwok
        |> push_front 5 |> uwok
        |> push_back 9 |> uwok
    let size %i32 len &dq0;
    let ok1 %bool match peek_front &dq0:
        Option::Some v:
            eq v 5
        Option::None:
            false
    let ok2 %bool match peek_back &dq0:
        Option::Some v:
            eq v 9
        Option::None:
            false
    free dq0
    let report:
        test_report_new "deque_pipe_usage"
        |> test_report_push assert_eq_i32 "deque len" 3 size
        |> test_report_push assert "front item" ok1
        |> test_report_push assert "back item" ok2
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## deque_grow_clear_and_free

[目的/もくてき]:
- capacity 1 から `push_front` / `push_back` の grow 経路を[通/とお]り、old buffer cleanup と header 更新のあとも[両端/りょうたん]の[順序/じゅんじょ]を[保/たも]つことを[確認/かくにん]します。
- `clear` と `free` が通常 cleanup として trap しないことを[確認/かくにん]します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"deque_grow_clear_and_free\" count=4 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"len after grow\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"front after grow\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"back after grow\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"clear empties deque\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/deque" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let dq0 %Deque i32 unwrap_ok with_capacity 1;
    let dq1 %Deque i32 unwrap_ok push_back dq0 10;
    let dq2 %Deque i32 unwrap_ok push_front dq1 5;
    let dq3 %Deque i32 unwrap_ok push_back dq2 20;
    let size %i32 len &dq3;
    let ok_front %bool match peek_front &dq3:
        Option::Some v:
            eq v 5
        Option::None:
            false
    let ok_back %bool match peek_back &dq3:
        Option::Some v:
            eq v 20
        Option::None:
            false
    let dq4 %Deque i32 clear dq3;
    let ok_clear %bool is_empty &dq4;
    free dq4;
    let report:
        test_report_new "deque_grow_clear_and_free"
        |> test_report_push assert_eq_i32 "len after grow" 3 size
        |> test_report_push assert "front after grow" ok_front
        |> test_report_push assert "back after grow" ok_back
        |> test_report_push assert "clear empties deque" ok_clear
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
