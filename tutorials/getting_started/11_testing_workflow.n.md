# テスト駆動で関数を固める

実装を進めるときは、最初に「壊したくない挙動」を `std/test` で固定してから実装を調整すると安全です。

この章では小さな関数を例に、入力ケースを増やして仕様を固定する流れを示します。

## 仕様を先にテストで固定する

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "std/test" as *

fn abs_i32 <(i32)->i32> (x):
    if lt x 0 then sub 0 x else x

fn main <()*> ()> ():
    assert_eq_i32 0 abs_i32 0
    assert_eq_i32 8 abs_i32 8
    assert_eq_i32 8 abs_i32 -8
    test_checked "abs_i32 cases"
```

## 失敗時の読みやすい出力

`test_checked` を使うと、どの塊が通ったかを小さく区切って確認できます。

neplg2:test[stdio, normalize_newlines, strip_ansi]
stdout: "Checked section-a\nChecked section-b\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "std/test" as *

fn main <()*> ()> ():
    test_checked "section-a";
    test_checked "section-b";
```

## テスト追加の実務手順

1. 先に最小ケースを 1 つ追加する（赤になることを確認）。
2. 実装を最小変更で通す。
3. 境界ケースを 1 つずつ増やし、失敗時に原因が切り分けられる粒度を保つ。

一度に大量ケースを追加すると、失敗原因の特定コストが急に上がるため、差分を小さく保つのが重要です。
