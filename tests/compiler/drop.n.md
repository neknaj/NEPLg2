# drop

`Drop` capability と auto drop 挿入の compiler 回帰です。
runtime の drop 順序は Rust 側 integration test で詳細に固定し、この `.n.md` では nodesrc 経路でも `Drop` を含む入力が正常に compile / run できることを確認します。

## drop_simple_let

[目的/もくてき]:
- `Drop` trait を source で宣言できることを確認します。
- scope end の auto drop が入っても `main` の返り値を壊さないことを確認します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"drop_simple_let\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"drop simple exit marker\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#no_prelude
#import "core/traits/drop" as *
#import "std/test" as *

struct Guard:
    dummy %i32

impl Drop for Guard:
    fn drop %impure fn &Guard unit \self:
        unit

fn main %impure fn unit i32 \unit:
    let g %Guard Guard 0;
    let report:
        test_report_new "drop_simple_let"
        |> test_report_push assert_eq_i32 "drop simple exit marker" 0 0
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## drop_nested_scopes

[目的/もくてき]:
- [入/い]れ[子/こ] scope の local に対して auto drop を[挿入/そうにゅう]しても、block [末尾/まつび]の return [値/ち]が[失/うしな]われないことを確認します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"drop_nested_scopes\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"nested branch result\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"nested drop exit marker\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#no_prelude
#import "core/traits/drop" as *
#import "std/test" as *

struct OuterGuard:
    dummy %i32
struct InnerGuard:
    dummy %i32

impl Drop for OuterGuard:
    fn drop %impure fn &OuterGuard unit \self:
        unit

impl Drop for InnerGuard:
    fn drop %impure fn &InnerGuard unit \self:
        unit

fn main %impure fn unit i32 \unit:
    let outer %OuterGuard OuterGuard 0;
    let branch %i32 if true:
        then:
            let inner %InnerGuard InnerGuard 0;
            1
        else:
            0
    let report:
        test_report_new "drop_nested_scopes"
        |> test_report_push assert_eq_i32 "nested branch result" 1 branch
        |> test_report_push assert_eq_i32 "nested drop exit marker" 0 0
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## drop_if_branch

[目的/もくてき]:
- `if` の[片方/かたほう]の branch だけに local `Drop` 型がある case でも compile / run できることを確認します。
- merge 後に `PossiblyMoved` / scope local の[扱/あつか]いが壊れていないことを確認します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"drop_if_branch\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"selected branch result\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"branch drop exit marker\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#no_prelude
#import "core/traits/drop" as *
#import "std/test" as *

struct TrueGuard:
    dummy %i32
struct FalseGuard:
    dummy %i32

impl Drop for TrueGuard:
    fn drop %impure fn &TrueGuard unit \self:
        unit

impl Drop for FalseGuard:
    fn drop %impure fn &FalseGuard unit \self:
        unit

fn main %impure fn unit i32 \unit:
    let flag %bool true;
    let branch %i32 if flag:
        then:
            let g %TrueGuard TrueGuard 0;
            1
        else:
            let h %FalseGuard FalseGuard 0;
            2
    let report:
        test_report_new "drop_if_branch"
        |> test_report_push assert_eq_i32 "selected branch result" 1 branch
        |> test_report_push assert_eq_i32 "branch drop exit marker" 0 0
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## drop_multiple_bindings_reverse_order

[目的/もくてき]:
- [同/おな]じ scope に[複数/ふくすう]の `Drop` 型 local があっても compile / run できることを確認します。
- reverse order drop に必要な epilogue [追加/ついか]が function return を壊さないことを確認します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"drop_multiple_bindings_reverse_order\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"multiple drop exit marker\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#no_prelude
#import "core/traits/drop" as *
#import "std/test" as *

struct GuardA:
    dummy %i32
struct GuardB:
    dummy %i32

impl Drop for GuardA:
    fn drop %impure fn &GuardA unit \self:
        unit

impl Drop for GuardB:
    fn drop %impure fn &GuardB unit \self:
        unit

fn main %impure fn unit i32 \unit:
    let a %GuardA GuardA 0;
    let b %GuardB GuardB 0;
    let report:
        test_report_new "drop_multiple_bindings_reverse_order"
        |> test_report_push assert_eq_i32 "multiple drop exit marker" 0 0
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
