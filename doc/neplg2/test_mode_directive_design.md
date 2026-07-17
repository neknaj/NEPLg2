# NEPLg2 #test directive design

この文書は、Rust の `cfg(test)` に相当する NEPLg2 の test-only compile mode を定義する。

Rust では、通常 build では単体テスト用 code を compile 対象から外し、test build のときだけ compile する。NEPLg2 でも同じ目的のために、通常 compile と doctest / `nepl-cli test` compile を区別する `test_mode` 軸を持つ。

## 基本方針

`profile` は source semantics の debug / release 切り替えであり、test mode ではない。`debug` build でも通常 compile は test-only code を含めない。

test-only statement には `#test` を置く。

```neplg2
#test
fn helper %fn void i32 \void:
    1
```

この directive は既存の `#if[target=...]` / `#if[profile=...]` と同じく、直後の 1 statement だけに効く。複数の test-only item を置く場合は各 item の前に `#test` を置く。

依存 module の test-only item も必要な場合は、その import の末尾に `with tests` を付ける。

```neplg2
#import "support" as * with tests
```

test origin は entry source から始まり、`with tests` を持つ import と `#include` だけを推移的に辿る。通常 import と prelude は依存 module の production item だけを公開し、依存 module の `#test` itemを有効化しない。これにより、明示したtest helper dependencyだけを有効化し、同じmerged moduleにある無関係なtest overloadやtest implを混入させない。

`with tests` は通常 import のload、名前空間、visibility、re-exportを変更しない。test modeにおけるtest originの伝播だけを追加する。通常compileでは依存 module の `#test` itemは従来通り無効である。

`test_mode` が false の compile:

```text
#test の直後 1 statement を無効化する
```

`test_mode` が true の compile:

```text
#test の直後 1 statement を有効化する
```

## 起動点

test mode を true にする入口は次である。

- `nepl-cli test`
- `nepl-cli --test-mode`
- `nodesrc/run_test.js` の doctest compile
- `nodesrc/tests.js` の LLVM doctest compile
- `nepl-web` の `*_test_mode` compile API

通常の CLI compile、Web playground compile、prewarm compile は false のままにする。

## cache 境界

test mode は active statement 集合を変えるため、compile cache、Resource summary proof artifact の key に含める。

通常 compile と test compile が同じ artifact を再利用すると、通常 compile に test-only public item が混入するか、test compile が test helper を見失う。したがって、`test_mode` は target / profile と同じ invalidation boundary として扱う。

## 実装境界

- lexer / parser / AST に `DirTest` / `Directive::Test` を追加する。
- `target_gate` は target / profile / test mode を同じ active statement 判定に集約する。
- typecheck、raw body precheck、LLVM codegen は同じ active statement helper を使う。
- doctest runner と `nepl-cli test` は test mode を true にする。
- selfhost lexer/parser の token/directive enum も Rust 実装と同期する。
