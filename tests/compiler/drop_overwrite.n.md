# drop overwrite

`set` で `Drop` 型 local を上書きする時の drop elaboration 回帰です。
runtime の Drop 順序は Rust 側 integration test で固定し、この `.n.md` では nodesrc 経路でも旧値 drop 用の HIR 展開が compile / run できることを確認します。

## drop_set_overwrite

[目的/もくてき]:
- `set` で `Drop` 型 local を[上書/うわが]きしても compiler pipeline が旧値 drop と新値代入を正しく扱えることを確認します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"drop_set_overwrite\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"drop overwrite exit marker\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#no_prelude
#import "core/traits/drop" as *
#import "std/test" as *

struct Guard:
    dummy %i32

impl Drop for Guard:
    fn drop %impure fn &Guard unit \self:
        unit

fn main %impure fn void i32 \void:
    let mut g %Guard Guard 0;
    set g Guard 1;
    let report:
        test_report_new "drop_set_overwrite"
        |> test_report_push assert_eq_i32 "drop overwrite exit marker" 0 0
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
