# tests/ringbuffer_collections.n.md

## ringbuffer_pipe_usage

[目的/もくてき]:
- `RingBuffer` が pipe [記法/きほう]と `Result` / `Option` を[組/く]み[合/あ]わせた[基本的/きほんてき]な[使/つか]い[方/かた]で[利用/りよう]できることを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `new`
- `push`
- `len`
- `free`
- `pop`
- `uwok`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"ringbuffer_pipe_usage\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"ringbuffer len\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"pop follows FIFO\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/ringbuffer" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let rb %RingBuffer i32:
        unwrap_ok new
        |> push 4
        |> uwok
        |> push 9
        |> uwok
    let size %i32 len &rb;
    free rb
    let rb2 %RingBuffer i32:
        unwrap_ok new
        |> push 4
        |> uwok
        |> push 9
        |> uwok
    let ok1 %bool match rb2 |> pop:
        Option::Some v:
            eq v 4
        Option::None:
            false
    let report:
        test_report_new "ringbuffer_pipe_usage"
        |> test_report_push assert_eq_i32 "ringbuffer len" 2 size
        |> test_report_push assert "pop follows FIFO" ok1
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## ringbuffer_grow_clear_free

[目的/もくてき]:
- `RingBuffer` が capacity 1 から grow した後でも、header 更新、`clear`、`free` が trap せず[動作/どうさ]することを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `with_capacity`
- `push`
- grow
- `clear`
- `free`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"ringbuffer_grow_clear_free\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"clear empties ringbuffer\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/ringbuffer" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let rb_clear %RingBuffer i32:
        unwrap_ok with_capacity 1
        |> push 4 |> uwok
        |> push 9 |> uwok
        |> clear
    let ok0 %bool is_empty &rb_clear;
    free rb_clear
    let rb0 %RingBuffer i32:
        unwrap_ok with_capacity 1
        |> push 4 |> uwok
        |> push 9 |> uwok
        |> clear
    free rb0
    let rb1 %RingBuffer i32:
        unwrap_ok with_capacity 1
        |> push 12 |> uwok
    free rb1
    let report:
        test_report_new "ringbuffer_grow_clear_free"
        |> test_report_push assert "clear empties ringbuffer" ok0
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
