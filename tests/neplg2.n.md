# neplg2.rs 由来の doctest

このファイルは Rust テスト `neplg2.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## compiles_literal_main

このテストは以前、コンパイルが通るかだけを確認しており、実行結果（main の返り値）を検証していませんでした。
main は i32 を返すので、`ret:` を追加して「実際に評価した値が 1 であること」まで確認します。

neplg2:test
ret: 1
```neplg2
#entry main
fn main <() -> i32> ():
    #import "core/math" as *
    1
```

## compiles_add_block_expression

このテストは「ブロック式（block expression）」を関数呼び出しの引数として渡す挙動を確認したい内容ですが、以前はコンパイル確認のみで、
ブロックの評価結果が正しいか（= `add 1:` の下で計算した値が第2引数になり、合計が 6 になるか）を検証していませんでした。

具体的には、
・`add 1:` の直後の行でインデントを 1 段深くし（`#indent 4` なので 8 スペース相当）、そのブロックの最終式 `add 2 3` を第2引数として渡します。
・ブロック末尾に `;` を付けないことで「ブロック全体の値」が `add 2 3 = 5` となり、外側で `add 1 5 = 6` になります。

この期待値を `ret: 6` として明示しました。

neplg2:test
ret: 6
```neplg2
#entry main
#indent 4

#target wasm
#if[target=wasm]
fn add <(i32, i32) -> i32> (a, b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <() -> i32> ():
    #import "core/math" as *
    add 1:
        add 2 3
```

## set_type_mismatch_is_error

neplg2:test[compile_fail]
```neplg2

#entry main
fn main <() -> ()> ():
    let mut x <i32> 0;
    set x ();
```

## pure_cannot_call_impure

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

fn imp <(i32) *> i32> (x):
    #import "core/math" as *
    add x 1

fn pure <(i32) -> i32> (x):
    imp x

fn main <() -> i32> ():
    pure 1
```

## iftarget_non_wasm_is_skipped

以前はコンパイル確認のみでした。
`#if[target=other]` のブロックが非該当ターゲットではスキップされることを狙ったテストなので、
実行時には main が 1 を返せる（= スキップが有効で bad 定義が影響しない）ことを `ret: 1` で確認します。

neplg2:test
ret: 1
```neplg2
#entry main

#if[target=other]
fn bad <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    1
```

## ifprofile_debug_gate

以前はコンパイル確認のみでした。
`#if[profile=debug]` のゲートが有効なときに only_debug が定義され、main から呼べることを
`ret: 123` で確認します（= 実際に only_debug が評価される）。

neplg2:test
ret: 123
```neplg2
#entry main

#if[profile=debug]
fn only_debug <() -> i32> ():
    123

fn main <() -> i32> ():
    only_debug
```

## ifprofile_release_skips_in_debug

以前はコンパイル確認のみでした。
デバッグプロファイルでは `#if[profile=release]` 部分がスキップされ、未知シンボルを含む定義がコンパイルに影響しないことを狙っています。
実行時に main が 0 を返せることを `ret: 0` で確認します。

neplg2:test
ret: 0
```neplg2
#entry main

#if[profile=release]
fn only_release <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    0
```

## wasm_stack_mismatch_is_error

neplg2:test[compile_fail]
```neplg2

#entry main

#if[target=wasm]
fn add_one <(i32)->i32> (a):
    #wasm:
        local.get $a
        // missing value for add
        i32.add

fn main <() -> i32> ():
    #import "core/math" as *
    add_one 1
```

## wasi_allows_wasm_gate

以前はコンパイル確認のみでした。
このテストは `#if[target=wasm]` の分岐が有効な環境で only_wasm が定義され、main から呼べることを確認したいので、
ターゲットを明示的に `#target wasm` とし、返り値 `123` を `ret:` で検証します。

neplg2:test
ret: 123
```neplg2
#entry main

#target wasm
#if[target=wasm]
fn only_wasm <() -> i32> ():
    123

fn main <() -> i32> ():
    only_wasm
```

## wasm_skips_wasi_gate

以前はコンパイル確認のみでした。
`#if[target=wasi]` の定義が wasm ターゲットではスキップされ、未知シンボルを含む定義がコンパイルに影響しないことを狙っています。
ターゲットを `#target wasm` と明示し、main が 0 を返すことを `ret: 0` で確認します。

neplg2:test
ret: 0
```neplg2
#entry main

#target wasm
#if[target=wasi]
fn only_wasi <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    0
```

## iftarget_applies_to_next_single_expression_only

このテストは `#if[target=...]` が「直後の 1 式のみ」に適用される仕様を確認します。
1 つ目の `fn` 式は条件でスキップされますが、2 つ目は無条件で評価されるため、未定義識別子によりコンパイル失敗する必要があります。

neplg2:test[compile_fail]
```neplg2
#entry main
#target wasm

#if[target=wasi]
fn skipped <() -> i32> ():
    unknown_symbol_a

fn not_skipped <() -> i32> ():
    unknown_symbol_b

fn main <() -> i32> ():
    0
```

## iftarget_on_general_call_expression

`#if[target=...]` を通常の呼び出し式に適用できることを確認します。
先頭の未定義式だけがスキップされ、次の `add 2 3` は評価される必要があります。

neplg2:test
ret: 5
```neplg2
#entry main
#target wasm
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    #if[target=other]
    unknown_symbol
    add 2 3
```

## iftarget_on_let_expression

`#if[target=...]` を `let` 式に適用できることを確認します。
無効ターゲットの `let bad ...` だけがスキップされ、後続の `let ok ...` は評価される必要があります。

neplg2:test
ret: 7
```neplg2
#entry main
#target wasm
#indent 4

fn main <()->i32> ():
    #if[target=other]
    let bad <i32> unknown_symbol;
    let ok <i32> 7;
    ok
```

## iftarget_on_if_expression

`#if[target=...]` を `if` 式に適用できることを確認します。
無効ターゲットの `if` 式だけがスキップされ、後続の `if` 式は通常どおり評価される必要があります。

neplg2:test
ret: 9
```neplg2
#entry main
#target wasm
#indent 4

fn main <()->i32> ():
    #if[target=other]
    if true then 1 else unknown_symbol
    if true then 9 else 0
```

## import_and_prelude_directives_are_accepted

以前はコンパイル確認のみでした。
このテストの本質は各種ディレクティブ（#prelude / #no_prelude / #import など）のパースと受理ですが、
main が i32 を返すので `ret: 0` を付けて「実行まで通る」ことも確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#prelude std/prelude_base
#no_prelude
#import "core/math" as { add as plus, math::* }
#import "./part" as @merge

fn main <() -> i32> ():
    0
```

## string_literal_compiles

以前は `#extern "env" "print_str"` を呼ぶだけの内容で、実行環境（ホスト側に該当関数があるか）に依存するため、
返り値や出力を検証できていませんでした。

ここでは「文字列リテラル自体の扱い」を確実にテストするため、WASI の `std/stdio` による `print` を用いた実行可能な形に変更し、
`stdout:` で出力が "hello" になることまで確認します。

neplg2:test
stdout: "hello"
```neplg2
#target std
#entry main
#indent 4
#import "std/stdio" as *

fn main <()* >()> ():
    // 文字列リテラルが評価でき、標準出力へ書き出せることを確認する
    print "hello"
```

## pipe_injects_first_arg

以前はコンパイル確認のみでした。
パイプ演算子 `|>` が「左辺の値を右辺の関数呼び出しの第1引数へ注入する」ことを、実行結果で確認します。
この式は `add 2 3 = 5`、`add 1 5 = 6`、最後に `6 |> add 4` により `add 6 4 = 10` となるため、`ret: 10` を追加しました。

neplg2:test
ret: 10
```neplg2
#entry main
#indent 4

#target wasm
#if[target=wasm]
fn add <(i32,i32)->i32> (a,b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <()->i32> ():
    add 1 add 2 3 |> add 4
```

## pipe_requires_callable_target

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

fn main <()->i32> ():
    1 |> 2
```

## pipe_with_type_annotation_is_ok

以前は「型注釈つきパイプが構文上OK」かどうかのコンパイル確認のみでした。
実際に `1 |> <i32> add 4` が `add 1 4 = 5` になることまで確認するため、`ret: 5` を追加しました。

neplg2:test
ret: 5
```neplg2
#entry main
#indent 4

#target wasm
#if[target=wasm]
fn add <(i32,i32)->i32> (a,b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <()->i32> ():
    1 |> <i32> add 4
```

## pipe_with_double_type_annotation_is_ok

以前は「型注釈が2回付いてもコンパイルできる」かの確認のみでした。
実際に計算が行われ `1 |> <i32> <i32> add 4 = 5` になることを `ret: 5` で確認します。

neplg2:test
ret: 5
```neplg2
#entry main
#indent 4

#target wasm
#if[target=wasm]
fn add <(i32,i32)->i32> (a,b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <()->i32> ():
    1 |> <i32> <i32> add 4
```

## pipe_target_missing_after_annotation_is_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

fn main <()->i32> ():
    1 |> <i32> 2
```

## wasi_import_rejected_on_wasm_target

以前は単なる `neplg2:test` で、失敗すべき条件（WASM ターゲットで WASI インポートを宣言する）を検証できていませんでした。
テスト名どおり「WASM ターゲットでは WASI インポートが拒否される」ことを確認したいので、
ターゲットを `#target wasm` と明示し、`compile_fail` として扱います。

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm
#extern "wasi_snapshot_preview1" "fd_write" fn fd_write <(i32,i32,i32,i32)->i32>
fn main <()->()> ():
    ()
```

## name_conflict_enum_fn_is_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

enum Foo:
    A

fn Foo <()->i32> ():
    0

fn main <()->i32> ():
    Foo
```

## wasm_cannot_use_stdio

以前は単なる `neplg2:test` で、実際には「コンパイルできてしまう」場合に検知できませんでした。
このテスト名は「WASM ターゲットでは std/stdio が使えない（= コンパイル時に拒否される）」ことを意図しているため、
ターゲットを `#target wasm` と明示し、`compile_fail` として検証します。

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target wasm
#import "std/stdio" as *

fn main <()->()> ():
    print "hi"
```

## run_add_returns_12

neplg2:test
ret: 12
```neplg2

#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    add 10 2
```

## match_option_some_returns_value

neplg2:test
ret: 5
```neplg2

#entry main
#indent 4
#import "core/option" as *

fn main <()* >i32> ():
    match some 5:
        Some v:
            v
        None:
            0
```

## list_get_out_of_bounds_err

以前はコンパイル確認のみでした。
`list_get` が範囲外アクセスで `None` を返す（= match の None 側に落ち、0 になる）ことを実行結果で確認するため、`ret: 0` を追加しました。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#import "alloc/collections/list" as *
#import "core/option" as *

fn main <()* >i32> ():
    let lst list_nil<i32>;
    let lst list_cons<i32> 1 lst;
    let r list_get<i32> lst 10;
    match r:
        Some v:
            v
        None:
            0
```

## non_exhaustive_match_is_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4
#import "core/option" as *

fn main <()->i32> ():
    match some 1:
        Some v:
            v
```

## target_directive_sets_default_to_wasi

以前は `compile_ok` で「コンパイルが通るか」だけでした。
このテストは実際に WASI として動作すること（少なくとも `std/stdio` が使えること）を確認したいので、`stdout: "ok"` を追加して実行結果まで検証します。

neplg2:test
stdout: "ok"
```neplg2

#target std
#entry main
#indent 4
#import "std/stdio" as *

fn main <()* >()> ():
    print "ok"
```

## duplicate_target_directive_is_error

以前は単なる `neplg2:test` で、ターゲット指定の重複が本当にエラーになるかを検証できていませんでした。
テスト名どおり「#target ディレクティブの重複はエラー」を確認したいので `compile_fail` に変更します。

neplg2:test[compile_fail]
```neplg2
#target wasm
#target std
#entry main
fn main <()->i32> ():
    0
```

## overloads_by_param_type_are_allowed

以前はコンパイル確認のみでした。
i32 と f32 のオーバーロードが解決され、main では i32 版 `id` が選ばれて 1 が返ることを `ret: 1` で確認します。

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4

fn id <(i32)->i32> (x):
    x

fn id <(f32)->f32> (x):
    x

fn main <()->i32> ():
    let tmp id 1.0;
    id 1
```

## overloads_with_different_arity_are_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

fn foo <(i32)->i32> (x):
    x

fn foo <(i32,i32)->i32> (a,b):
    a

fn main <()->i32> ():
    foo 1
```

## overloads_ambiguous_return_type_is_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

fn foo <(i32)->i32> (x):
    x

fn foo <(i32)->f32> (x):
    1.0

fn main <()->i32> ():
    foo 1
```

## trait_method_call_with_impl_compiles

以前はコンパイル確認のみでした。
trait の関連関数呼び出し `Show::show 1` が実行でき、1 を返すことを `ret: 1` で確認します。

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i32> (x):
        x

fn main <()->i32> ():
    Show::show 1
```

## trait_bound_satisfied_in_generic

以前はコンパイル確認のみでした。
型クラス制約（trait bound）を満たす型に対してジェネリック関数が正しく呼べることを、返り値 `5` で確認します。

neplg2:test
ret: 5
```neplg2
#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i32> (x):
        x

fn call_show <.T: Show> <(.T)->i32> (x):
    Show::show x

fn main <()->i32> ():
    call_show 5
```

## trait_bound_missing_impl_is_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

fn call_show <.T: Show> <(.T)->i32> (x):
    Show::show x

fn main <()->i32> ():
    call_show 1
```

## trait_method_arity_mismatch_is_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i32> (x):
        x

fn main <()->i32> ():
    Show::show 1 2
```

## unknown_trait_bound_is_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

fn call_show <.T: Missing> <(.T)->i32> (x):
    0

fn main <()->i32> ():
    0
```

## unreachable_does_not_force_never_in_generic

以前はコンパイル確認のみでした。
ジェネリック関数内の unreachable 分岐が型推論を壊さず、通常経路で値 `1` を返せることを `ret: 1` で確認します。

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4

fn pick <.T> <(.T)->.T> (x):
    if:
        true
        then:
            x
        else:
            #intrinsic "unreachable" <> ()

fn main <()->i32> ():
    pick 1
```
