# proptest.rs 由来の doctest

このファイルは Rust テスト `proptest.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## prop_add_commutative

以前は `skip` のままで、実質的に何も検証していませんでした。
本来 proptest は入力を自動生成して性質を広く検証しますが、ここでは機械的移植の制約上、
代表値（0、正数、負数）に対して「加算の可換性 add(a,b)=add(b,a)」を実行結果でチェックする形に置き換えます。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    // Rust の proptest は多数の入力を自動生成して「常に成り立つ性質」を検証するが、
    // ここでは .n.md の単体実行で扱える範囲として、代表的な入力を複数選んで可換性を確認する。
    let a1 add 0 0;
    let a2 add 0 0;
    if:
        eq a1 a2
        then:
            let b1 add 1 2;
            let b2 add 2 1;
            if:
                eq b1 b2
                then:
                    let c1 add -3 5;
                    let c2 add 5 -3;
                    if:
                        eq c1 c2
                        then:
                            0
                        else:
                            3
                else:
                    2
        else:
            1
```

## prop_sub_inverse

以前は `skip` のままで、実質的に何も検証していませんでした。
本来はランダム入力で広く検証すべき性質ですが、ここでは代表値で「減算の逆元性 (a-b)+b=a」を確認し、
失敗時は 0 以外のエラーコードを返すようにして `ret: 0` で合否判定します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    // (a - b) + b == a を、代表的な入力で確認する
    let a0 10;
    let b0 3;
    let t0 sub a0 b0;
    let x0 add t0 b0;

    if:
        eq x0 a0
        then:
            let a1 -7;
            let b1 5;
            let t1 sub a1 b1;
            let x1 add t1 b1;
            if:
                eq x1 a1
                then:
                    0
                else:
                    2
        else:
            1
```

## prop_fail_example

以前は `compile_fail,skip` となっており、しかも中身が 0 を返すだけだったため、
「失敗例」のテストとして機能していませんでした。
ここでは `skip` を外し、未定義シンボル参照で確実にコンパイルエラーになるケースを置いて `compile_fail` を検証します。

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
fn main <()->i32> ():
    // 故意に未定義シンボルを参照してコンパイルエラーになることを確認する
    unknown_symbol
```

