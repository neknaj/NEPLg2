# typeannot.rs 由来の doctest

このファイルは Rust テスト `typeannot.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## test_type_annot_basic

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_type_annot_basic\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"basic literal annotation\" expected=\"123\" actual=\"123\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn main %impure fn unit i32 \unit:
    // 基本的なリテラルへの型注釈
    // 式 `123` は i32
    // `%i32` を前置しても値は変わらず、型がチェックされる
    let a %i32 123

    // 式の結果をそのまま検査する
    let report:
        test::test_report_new "test_type_annot_basic"
        |> test::test_report_push test::assert_eq_i32 "basic literal annotation" 123 a
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## test_type_annot_nested_expr

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_type_annot_nested_expr\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"nested expression annotation\" expected=\"60\" actual=\"60\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn main %impure fn unit i32 \unit:
    // 計算式全体への型注釈
    // add 10 20 は i32 を返す
    let a %i32 add 10 20

    // 部分式への型注釈も可能
    // `%i32 10` も `%i32 20` もただの i32 として振る舞う
    let b add %i32 10 %i32 20

    let actual %i32 add a b
    let report:
        test::test_report_new "test_type_annot_nested_expr"
        |> test::test_report_push test::assert_eq_i32 "nested expression annotation" 60 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## test_type_annot_on_let

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_type_annot_on_let\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"let expression annotation\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn main %impure fn unit i32 \unit:
    // plan.md 94行目の例: let mut neg %bool lt n 0
    // let 宣言の右辺式全体に対する型注釈

    let n 10

    // `%bool` は `lt n 0` という式にかかる
    let neg %bool lt n 0

    let actual %i32 if:
        neg
        then 1
        else 0
    let report:
        test::test_report_new "test_type_annot_on_let"
        |> test::test_report_push test::assert_eq_i32 "let expression annotation" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## test_type_annot_block

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_type_annot_block\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"block expression annotation\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn main %impure fn unit i32 \unit:
    // ブロック式全体への型注釈
    // ブロックの評価結果（最後の式の値）に対して型注釈がかかる

    let v %i32 block:
        let x 1
        let y 2
        add x y

    let report:
        test::test_report_new "test_type_annot_block"
        |> test::test_report_push test::assert_eq_i32 "block expression annotation" 3 v
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## test_type_annot_nested_annot

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_type_annot_nested_annot\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"nested redundant annotation\" expected=\"100\" actual=\"100\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn main %impure fn unit i32 \unit:
    // 型注釈を重ねることは仕様上可能だが冗長
    // 通常は 1 回の注釈を推奨

    let v %i32 %i32 100
    let report:
        test::test_report_new "test_type_annot_nested_annot"
        |> test::test_report_push test::assert_eq_i32 "nested redundant annotation" 100 v
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## test_type_annot_function_call

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_type_annot_function_call\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"function call result annotation\" expected=\"123\" actual=\"123\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn id %fn i32 i32 \x:
    x

fn main %impure fn unit i32 \unit:
    // 関数適用の結果に対する型注釈
    // id 123 は i32 を返すので %i32 で注釈可能

    let v %i32 id 123
    let report:
        test::test_report_new "test_type_annot_function_call"
        |> test::test_report_push test::assert_eq_i32 "function call result annotation" 123 v
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## test_type_annot_complex_expr

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_type_annot_complex_expr\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"complex expression annotation\" expected=\"10\" actual=\"10\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn main %impure fn unit i32 \unit:
    // 複雑な式の中での型注釈
    // add (mul %i32 2 3) (%i32 4)

    let left %i32 mul %i32 2 %i32 3
    let v %i32 add left %i32 4
    let report:
        test::test_report_new "test_type_annot_complex_expr"
        |> test::test_report_push test::assert_eq_i32 "complex expression annotation" 10 v
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## test_type_annot_if_expr

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_type_annot_if_expr\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"if expression annotation\" expected=\"10\" actual=\"10\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn main %impure fn unit i32 \unit:
    // if式全体、あるいは各ブランチへの型注釈

    let v %i32 if:
        %bool true
        then %i32 10
        else %i32 20
    let report:
        test::test_report_new "test_type_annot_if_expr"
        |> test::test_report_push test::assert_eq_i32 "if expression annotation" 10 v
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## test_type_annot_while_condition

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_type_annot_while_condition\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"while condition annotation\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn main %impure fn unit i32 \unit:
    let mut i 0
    let mut sum 0

    // while の条件式に型注釈
    while %bool lt i 3:
        do:
            set sum add sum i
            set i add i %i32 1

    let report:
        test::test_report_new "test_type_annot_while_condition"
        |> test::test_report_push test::assert_eq_i32 "while condition annotation" 3 sum
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## test_type_annot_generic_like

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_type_annot_generic_like\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic type annotation\" expected=\"42\" actual=\"42\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "core/option" as *
#import "std/test" as test

fn main %impure fn unit i32 \unit:
    // ジェネリック型に対する型注釈
    // Option<i32> 型の値を生成し、それに型注釈をつける

    let opt %Option i32 some<i32> 42

    let actual %i32 match opt:
        Option::Some v:
            v
        Option::None:
            0
    let report:
        test::test_report_new "test_type_annot_generic_like"
        |> test::test_report_push test::assert_eq_i32 "generic type annotation" 42 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## test_type_annot_deeply_nested

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_type_annot_deeply_nested\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"deeply nested annotation\" expected=\"6\" actual=\"6\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn main %impure fn unit i32 \unit:
    // 深くネストされた関数呼び出しと型注釈
    // add( add( %i321, %i322 ), %i323 )

    let ab %i32 add %i32 1 %i32 2
    let v %i32 add ab %i32 3
    let report:
        test::test_report_new "test_type_annot_deeply_nested"
        |> test::test_report_push test::assert_eq_i32 "deeply nested annotation" 6 v
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## test_type_annot_mixed_with_blocks

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_type_annot_mixed_with_blocks\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"mixed block annotation\" expected=\"30\" actual=\"30\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn main %impure fn unit i32 \unit:
    // ブロックとインラインの混在

    let v %i32 add: // 関数の引数で改行しているのは正しい インデントは各引数の先頭が+1で揃う
        %i32 block: // 型注釈付きの無名ブロックも正しい ブロックなので返り値はx
            let x 10
            x
        %i32 20
    let report:
        test::test_report_new "test_type_annot_mixed_with_blocks"
        |> test::test_report_push test::assert_eq_i32 "mixed block annotation" 30 v
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## test_type_annot_mixed_block_call_pipe

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_type_annot_mixed_block_call_pipe\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"mixed block call pipe annotation\" expected=\"7\" actual=\"7\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn main %impure fn unit i32 \unit:
    let v %i32 %i32 block:
        let base %i32 %i32 add 1 2
        base |> %i32 add %i32 4
    let report:
        test::test_report_new "test_type_annot_mixed_block_call_pipe"
        |> test::test_report_push test::assert_eq_i32 "mixed block call pipe annotation" 7 v
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## test_type_annot_mixed_function_literal_call

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_type_annot_mixed_function_literal_call\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"function literal annotation call\" expected=\"9\" actual=\"9\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn apply %fn i32 fn fn i32 i32 i32 \x\f:
    f x

fn main %impure fn unit i32 \unit:
    let f %fn i32 i32 \x:
        %i32 block:
            let y %i32 add x 2
            y
    let v %i32 %i32 apply %i32 7 f
    let report:
        test::test_report_new "test_type_annot_mixed_function_literal_call"
        |> test::test_report_push test::assert_eq_i32 "function literal annotation call" 9 v
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## test_type_annot_mixed_pipe_with_annotated_function_literal

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_type_annot_mixed_pipe_with_annotated_function_literal\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"pipe annotated function literal\" expected=\"8\" actual=\"8\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn main %impure fn unit i32 \unit:
    let plus3 %fn i32 i32 \x:
        %i32 add x 3
    let src %i32:
        %i32 block:
            4
    let v %i32 src |> %i32 plus3 |> %i32 add 1
    let report:
        test::test_report_new "test_type_annot_mixed_pipe_with_annotated_function_literal"
        |> test::test_report_push test::assert_eq_i32 "pipe annotated function literal" 8 v
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
