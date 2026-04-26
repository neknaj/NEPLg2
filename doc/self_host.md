# NEPLg3 セルフホスト計画

最終更新: 2026-03-16

---

## 1. 目的

NEPLg3 コンパイラを NEPLg3 自身で書くこと（セルフホスト）を最終目標とする。
これは「言語プラットフォームの主実装を NEPLg3 コードで担う」という設計原則の完成形であり、
コンパイラの正しさとプラットフォームの表現力を同時に実証する。

詳細は [neplg3/spec/platform.md](./neplg3/spec/platform.md) §8 も参照。

---

## 2. 二層構造

| 層 | 場所 | 責務 |
|---|---|---|
| Bootstrap Host | `/nepl-core-g3`（Rust） | ブートストラップコンパイラパイプライン・最小 runtime・target/ABI adapter |
| Self-host Compiler | `/stdlib/neplg3`（NEPLg3） | NEPLg3 製コンパイラ本体。セルフホスト達成後の主実装 |

`nepl-core-g3` は足場であり、最終的な本体ではない。
プラットフォームの中心は `stdlib/neplg3` の NEPLg3 コードである。

---

## 3. ディレクトリ構成

```
stdlib/neplg3/
    cli/
        main.nepl      // CLI インターフェース（エントリポイント、WASI または LLVM ターゲット依存）
    core/              // コンパイラ本体（純粋 WASM、ターゲット非依存）
        syntax/
        module/
        resolve/
        ty/
        check/
        hir/
        resource/
        mono/
        codegen/
        builtins/
```

---

## 4. ブートストラップ手順

1. Rust 製 `nepl-core-g3` が `stdlib/` をビルドする（Pass 1）
2. Rust 製 `nepl-core-g3` が `stdlib/neplg3` セルフホストコンパイラをビルドする（Pass 1）
3. セルフホストコンパイラが自分自身と `stdlib/` をビルドする（Pass 2）

Pass 2 の出力が Pass 1 と一致すること（reproducible build）を確認する。

> **注意**: ここでの "Pass 1/2" はブートストラップのビルド回数を指す。`doc/neplg3/impl/compiler_structure.md §7` の "Stage 1–6"（nepl-core-g3 の開発段階）とは別の概念。

---

## 5. 開発方針

- Rust 製 `nepl-core-g3` を参照実装として使いながら、NEPLg3 でコンパイラ本体を実装する。
- 各モジュール（syntax・resolve・check・resource・codegen）を段階的に移植し、テストで正しさを確認する。
- コンパイラ内部の安全性も NEPLg3 の型安全・メモリ安全規則に従う（`Result/Option` ベース、raw pointer 非公開）。
- `stdlib/neplg3` 自体が NEPLg3 仕様の最も高度な利用例となるよう設計する。

---

## 6. テスト

- `stdlib/neplg3/` 内の各モジュールにドキュメントテストを整備する。
- `tests/compiler/` の回帰テストをセルフホストコンパイラでも通すことを段階目標とする。
- Stage 2 ビルドの出力一致チェックをCIに追加する（セルフホスト達成後）。
