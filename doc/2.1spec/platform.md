# NEPLg2.1 プラットフォーム・ターゲット仕様

最終更新: 2026-03-16

---

## 1. 言語プラットフォームとしての位置づけ

NEPLg2.1 は単なるアプリケーション言語ではなく、様々な言語・DSL（JSON / Markdown / CSV / カスタム DSL）を構築・処理するための**言語プラットフォーム**として設計されている。

主な目標:
- Lexing / Parsing（ツールキットベース）
- 変換・フォーマット処理
- 統合言語アイランド（埋め込み DSL）
- 強力な静的検証

---

## 2. 実装レイヤ構成

### 2.1 Bootstrap Host（Rust）

- 場所: `/nepl-core-2.1`
- 責務: ブートストラップコンパイラパイプライン・ターゲット・バックエンド（Wasm/LLVM）・最小ランタイムの提供。

### 2.2 Platform Stdlib（NEPL）

- 場所: `/stdlib`（移行期間は `/stdlib-2.1` を並行開発）
- 責務: 実際のプラットフォーム実装・パーサツールキット・DSL 実装・**セルフホストコンパイラ**（`stdlib/neplg2`）。

---

## 3. ターゲット

| ターゲット | 特徴 |
|---|---|
| `wasm` | WASI import なしの純粋 WebAssembly。任意の WASM runtime で動作。 |
| `wasi` | WASI syscall 付き WebAssembly。WASI runtime で動作。 |
| `llvm` | LLVM 経由のネイティブバイナリ（POSIX 環境）。 |

---

## 4. 安全意味論の統一

コンパイラは Wasm であっても LLVM であっても**全く同じ安全意味論の検査**を実行する。

コンパイラがすべてのターゲットで静的に保証する安全性:
- moved value の再使用禁止
- borrowed place への不正 mutation 禁止
- freed resource の再使用禁止
- pure / impure の境界
- `str`・`List`・`OwnedBuf`・`File`・`Socket` の source semantics

各バックエンド（Wasm / LLVM）は、完全に検証された IR を受け取り、**物理的な命令とレイアウトの生成のみ**を行う。

---

## 5. プラットフォーム差異の吸収

ターゲット間の物理的な差異（ポインタ表現・アロケータ・ハンドル表現など）は、コンパイラ内部ではなく**標準ライブラリの条件付きコンパイル**で吸収する。

```nepl
#if target "wasm":
    // linear memory offset ベースの実装
#if target "llvm":
    // native pointer ベースの実装
```

`#if` は 2.1 では角括弧メタ構文ではなく、`#if <cond_expr>:` という前置ディレクティブとして扱う。`target "wasm"` や `profile "debug"` のような compile-time 条件式を後ろに置き、ブロック全体へ作用させる。

| 項目 | Wasm 向け実装 | LLVM 向け実装 |
|------|--------------|--------------|
| pointer 表現 | linear memory offset（`i32` にラップ） | native pointer |
| allocator | `core/mem` 内の bump + free_list 実装 | libc `malloc`/`free` の FFI |
| `str` header | `[len:i32][data...]`（linear memory 上） | native `{ptr, len}` 構造体 |
| file / socket | WASI API と `fd`（`i32`） | POSIX / OS native API |

---

## 6. コンパイラパイプライン

コンパイラの解析パス順:

1. surface typecheck
2. effect attribution
3. Resource IR 生成
4. ownership / borrow check
5. region inference
6. drop elaboration
7. target lowering（Wasm または LLVM）

---

## 7. プラットフォームライブラリ

プラットフォームとして提供するライブラリ:

- **構文処理**: PEG runtime・レイアウト対応パーサ・ストリーミングサポート
- **ドキュメント処理**: JSON / XML / HTML / Markdown / LaTeX の統一モデル
- **言語アイランド**: 言語の中に別言語を埋め込む first-class サポート
- **クエリ・変換**: セレクタベースのクエリと構造的書き換え

---

## 8. セルフホスト計画

最終目標は NEPLg2 コンパイラを NEPLg2 で書くこと（`stdlib/neplg2`）。

ブートストラップ互換性を維持する:

1. Rust host が `stdlib` をビルドする
2. Rust host がセルフホストコンパイラをビルドする
3. セルフホストコンパイラが自分自身と `stdlib` をビルドする

詳細は [../self_host.md](../self_host.md) を参照。
