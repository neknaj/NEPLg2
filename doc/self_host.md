# NEPLg2 セルフホスト計画

最終更新: 2026-03-16

---

## 1. 目的

NEPLg2 コンパイラを NEPLg2 自身で書くこと（セルフホスト）を最終目標とする。
これは「言語プラットフォームの主実装を NEPLg2 コードで担う」という設計原則の完成形であり、
コンパイラの正しさとプラットフォームの表現力を同時に実証する。

詳細は [2.1spec/platform.md](./2.1spec/platform.md) §8 も参照。

---

## 2. 二層構造

| 層 | 場所 | 責務 |
|---|---|---|
| Bootstrap Host | `/nepl-core`（Rust） | ブートストラップコンパイラパイプライン・最小 runtime・target/ABI adapter |
| Self-host Compiler | `/stdlib/neplg2`（NEPLg2） | NEPLg2 製コンパイラ本体。セルフホスト達成後の主実装 |

`nepl-core` は足場であり、最終的な本体ではない。
プラットフォームの中心は `stdlib/neplg2` の NEPLg2 コードである。

---

## 3. ディレクトリ構成

```
stdlib/neplg2/
    cli/
        main.nepl      // CLI インターフェース（エントリポイント）
    src/
        core/          // コンパイラ本体
            lexer/
            parser/
            typecheck/
            resource_ir/
            codegen/
```

---

## 4. ブートストラップ手順

1. Rust 製 `nepl-core` が `stdlib/` をビルドする
2. Rust 製 `nepl-core` が `stdlib/neplg2` セルフホストコンパイラをビルドする
3. セルフホストコンパイラが自分自身と `stdlib/` をビルドする（Stage 2 検証）

Stage 2 の出力が Stage 1 と一致すること（reproducible build）を確認する。

---

## 5. 開発方針

- Rust 製 `nepl-core` を参照実装として使いながら、NEPLg2 でコンパイラ本体を実装する。
- 各モジュール（lexer・parser・typecheck・codegen）を段階的に移植し、テストで正しさを確認する。
- コンパイラ内部の安全性も NEPLg2.1 の型安全・メモリ安全規則に従う（`Result/Option` ベース、raw pointer 非公開）。
- `stdlib/neplg2` 自体が NEPLg2.1 仕様の最も高度な利用例となるよう設計する。

---

## 6. テスト

- `stdlib/neplg2/` 内の各モジュールにドキュメントテストを整備する。
- `tests/compiler/` の回帰テストをセルフホストコンパイラでも通すことを段階目標とする。
- Stage 2 ビルドの出力一致チェックをCIに追加する（セルフホスト達成後）。
