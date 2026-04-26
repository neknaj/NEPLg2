# drop overwrite

`set` で `Drop` 型 local を上書きする時の drop elaboration 回帰です。
runtime の Drop 順序は Rust 側 integration test で固定し、この `.n.md` では nodesrc 経路でも旧値 drop 用の HIR 展開が compile / run できることを確認します。

## drop_set_overwrite

[目的/もくてき]:
- `set` で `Drop` 型 local を[上書/うわが]きしても compiler pipeline が旧値 drop と新値代入を正しく扱えることを確認します。

neplg2:test
ret: 0
```neplg2
#target wasm
#entry main
#indent 4
#no_prelude
#import "core/traits/drop" as *

struct Guard:
    dummy <i32>

impl Drop for Guard:
    fn drop <(&Guard)*>()> (self):
        ()

fn main <()->i32> ():
    let mut g <Guard> Guard 0;
    set g Guard 1;
    0
```
