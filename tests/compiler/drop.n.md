# drop

`Drop` capability と auto drop 挿入の compiler 回帰です。  
runtime の drop 順序は Rust 側 integration test で詳細に固定し、この `.n.md` では nodesrc 経路でも `Drop` を含む入力が正常に compile / run できることを確認します。

## drop_simple_let

[目的/もくてき]:
- `Drop` trait を source で宣言できることを確認します。
- scope end の auto drop が入っても `main` の返り値を壊さないことを確認します。

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
    let g <Guard> Guard 0;
    0
```

## drop_nested_scopes

[目的/もくてき]:
- [入/い]れ[子/こ] scope の local に対して auto drop を[挿入/そうにゅう]しても、block [末尾/まつび]の return [値/ち]が[失/うしな]われないことを確認します。

neplg2:test
ret: 0
```neplg2
#target wasm
#entry main
#indent 4
#no_prelude
#import "core/traits/drop" as *

struct OuterGuard:
    dummy <i32>
struct InnerGuard:
    dummy <i32>

impl Drop for OuterGuard:
    fn drop <(&OuterGuard)*>()> (self):
        ()

impl Drop for InnerGuard:
    fn drop <(&InnerGuard)*>()> (self):
        ()

fn main <()->i32> ():
    let outer <OuterGuard> OuterGuard 0;
    let _ <i32> if true:
        then:
            let inner <InnerGuard> InnerGuard 0;
            1
        else:
            0
    0
```

## drop_if_branch

[目的/もくてき]:
- `if` の[片方/かたほう]の branch だけに local `Drop` 型がある case でも compile / run できることを確認します。
- merge 後に `PossiblyMoved` / scope local の[扱/あつか]いが壊れていないことを確認します。

neplg2:test
ret: 0
```neplg2
#target wasm
#entry main
#indent 4
#no_prelude
#import "core/traits/drop" as *

struct TrueGuard:
    dummy <i32>
struct FalseGuard:
    dummy <i32>

impl Drop for TrueGuard:
    fn drop <(&TrueGuard)*>()> (self):
        ()

impl Drop for FalseGuard:
    fn drop <(&FalseGuard)*>()> (self):
        ()

fn main <()->i32> ():
    let flag <bool> true;
    let _ <i32> if flag:
        then:
            let g <TrueGuard> TrueGuard 0;
            1
        else:
            let h <FalseGuard> FalseGuard 0;
            2
    0
```

## drop_multiple_bindings_reverse_order

[目的/もくてき]:
- [同/おな]じ scope に[複数/ふくすう]の `Drop` 型 local があっても compile / run できることを確認します。
- reverse order drop に必要な epilogue [追加/ついか]が function return を壊さないことを確認します。

neplg2:test
ret: 0
```neplg2
#target wasm
#entry main
#indent 4
#no_prelude
#import "core/traits/drop" as *

struct GuardA:
    dummy <i32>
struct GuardB:
    dummy <i32>

impl Drop for GuardA:
    fn drop <(&GuardA)*>()> (self):
        ()

impl Drop for GuardB:
    fn drop <(&GuardB)*>()> (self):
        ()

fn main <()->i32> ():
    let a <GuardA> GuardA 0;
    let b <GuardB> GuardB 0;
    0
```
