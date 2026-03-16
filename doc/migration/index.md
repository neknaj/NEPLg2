# stdlib / tests / tutorials NEPLg2.1 移行計画

最終更新: 2026-03-17

---

## 1. 移行の全体像

### 1.1 対象と規模

| 対象 | ファイル数 | 現在の状態 |
|---|---:|---|
| `stdlib/core/` | 18 | NEPLg2.0 構文。traits/ 以下 10 ファイル含む |
| `stdlib/std/` | 8 | NEPLg2.0 構文 |
| `stdlib/alloc/` | 50+ | NEPLg2.0 構文。collections/ 20 ファイル・string.nepl（74 KB）等 |
| `stdlib/nm/` | 3 | NEPLg2.0 構文 |
| `stdlib/neplg2/` | 6 | 暫定実装。NEPLg2.1 移行後に大幅再設計 |
| `stdlib/platforms/` | 1 | NEPLg2.0 構文 |
| `tests/compiler/` | 38 | NEPLg2.0 テストケース |
| `tests/stdlib/` | 38 | NEPLg2.0 テストケース |
| `tutorials/getting_started/` | 28 | NEPLg2.0 構文でのチュートリアル |

### 1.2 移行の制約

- **stdlib は NEPLg2.1 コンパイラが動作しないと移行できない**。現行 `nepl-core`（NEPLg2.0）は移行後の構文を解析できない。
- コンパイラの Stage 進行（`doc/2.1impl/compiler_structure.md §7`）と移行作業を同期させる。
- 移行中は `nepl-core`（NEPLg2.0）と `nepl-core-2.1` を並行運用し、既存テストを壊さない。

### 1.3 コンパイラ Stage との対応

| コンパイラ Stage | 達成内容 | 解禁される移行作業 |
|---|---|---|
| **Stage 1** | 字句解析・パーサ・モジュール骨格 | チュートリアル文書の書き換え（コンパイル不要） |
| **Stage 2** | 名前解決・型システム基礎 | `core/` の基本型・基本関数の移行検証 |
| **Stage 3** | 型検査・trait 検査・effect 検査 | `core/` 全体・`core/traits/` の移行 |
| **Stage 4** | Resource IR・ownership/borrow/region/drop | `alloc/` のメモリモデル移行（Phase 1–4） |
| **Stage 5** | 単相化・コード生成 | `std/` `alloc/` `nm/` の全移行・全テスト通過確認 |
| **Stage 6** | `nepl-core` → `nepl-core-2.1` 切り替え | `neplg2/` セルフホストコンパイラの本格移行 |

---

## 2. 構文変換クイックリファレンス

コードを移行する際の機械的な変換ルール（詳細は `doc/compare/syntax.md`）。

### 宣言キーワード

| NEPLg2.0 | NEPLg2.1 |
|---|---|
| `fn name <TypeParams> <Sig> (params):` | `let name [type_params] %fn ... -> ... \ params :` |
| `struct Name<.T>:` | `let Name struct .T:` |
| `enum Name<.T>:` | `let Name enum .T:` |
| `trait Name:` | `let Name trait:` |
| `impl Trait for Type:` | `let Type impl for Trait:` |

### 型記法

| NEPLg2.0 | NEPLg2.1 |
|---|---|
| `Vec<i32>` | `Vec i32` |
| `Option<.T>` | `Option .T` |
| `Result<i32, str>` | `Result i32 str` |
| `(A, B) -> C` | `fn A B -> C` |
| `(A, B) *> C` | `fn* A B -> C` |
| `()` （unit型）| `unit` |
| `<TypeExpr>` （型注釈）| `%TypeExpr` |

### 引数リスト・制御構造・モジュール

| NEPLg2.0 | NEPLg2.1 |
|---|---|
| `(a, b):` | `\ a b :` |
| `():` | `\ :` |
| `if cond then ... else ...` | `if cond : ... else : ...` |
| `while cond do ...` | `while cond : ...` |
| `#import "stdlib/std/streamio"` | `use std::streamio` |
| `#include "./editor_ops"` | `merge "./editor_ops"` |
| `#entry main` | `#entry` |
| `use path::*` | `use path as *` |

---

## 3. stdlib 移行計画

### 3.1 優先度と理由

```
依存方向（上が下に依存）:
  std/ alloc/ nm/ platforms/
        ↑
    alloc/string.nepl
        ↑
    core/mem.nepl
        ↑
    core/（option, result, cast, field）
        ↑
    core/traits/（eq, ord, hash, drop …）
```

移行は**下から上へ**（依存されるものから先に）行う。

### 3.2 Wave 1: `core/traits/` と `core/` の基本型（Stage 2 以降）

対象 10 + 7 ファイル。言語の根幹となるため最優先。

| ファイル | 作業内容 |
|---|---|
| `core/traits/eq.nepl` | `trait Eq:` → `let Eq trait:` / `fn eq` → `let eq %fn Self Self -> bool \ a b :` |
| `core/traits/ord.nepl` | 同上（`Ordering` enum の修飾形書き換えも必要） |
| `core/traits/hash.nepl` | 同上 |
| `core/traits/copy.nepl` | 同上 |
| `core/traits/drop.nepl` | 同上（Linear resource の Drop Elaboration と連携） |
| `core/traits/{debug,stringify,serialize,deserialize,hash_key}.nepl` | 同上パターン |
| `core/option.nepl` | `enum Option<.T>:` → `let Option enum .T:` / bare `Some`/`None` を修飾形または期待型付き bare 形へ |
| `core/result.nepl` | 同上（`Ok`/`Err` 修飾形） |
| `core/cast.nepl` | 関数シグネチャ書き換えのみ |
| `core/field.nepl` | 関数シグネチャ書き換えのみ |
| `core/test.nepl` | 関数シグネチャ書き換えのみ |

**注意事項**:
- enum バリアントの bare 参照（`Some x`, `None`, `Ok x`, `Err e`）は期待型が確定している場合のみ有効。確定しない箇所は `Option::Some x` 形式へ変更。
- `impl Eq for i32:` は `let i32 impl for Eq:` に変更。`Self` はメソッド内で使用可。

### 3.3 Wave 2: `core/mem.nepl` と raw pointer 隔離（Stage 3 + Memory Phase 1）

`core/mem.nepl` は最も難しい移行対象。

作業内容:
1. `mem_ptr_addr` / `mem_ptr_wrap` / `alloc_raw` / `dealloc_raw` を `private` に変更（構文変換と同時）
2. 公開 API を `MemPtr .T` ベースのオーバーロードに統一
3. `alloc_raw` → `InternalAlloc` effect 分類（コンパイラ対応後）

### 3.4 Wave 3: `alloc/string.nepl` と `alloc/collections/vec.nepl`（Stage 3–4）

最大ファイル（string.nepl 74 KB, vec.nepl 59 KB）。

- `string.nepl`: raw address 操作の隔離（Wave 2 完了後）+ 構文全体の書き換え
- `vec.nepl`: 構文書き換え + 内部の `mem_ptr_addr` 使用を隠蔽 API へ置換

`vec.nepl` が完了すると `Stack / Queue / Deque / ...` など vec に依存するコレクションが続けて移行できる。

### 3.5 Wave 4: `alloc/collections/` 残り（Stage 4）

以下の順（依存関係が少ないものから）:
1. `stack.nepl`, `queue.nepl`, `deque.nepl`, `ringbuffer.nepl`（vec 直接依存）
2. `list.nepl`（Region Inference 対応が必要。Memory Phase 2 待ち）
3. `hashmap.nepl`, `hashset.nepl`（Hash trait 対応が必要）
4. `btreemap.nepl`, `btreeset.nepl`, `binary_heap.nepl`（Ord trait 依存）
5. `disjoint_set.nepl`, `segment_tree.nepl`, `fenwick.nepl` 等

### 3.6 Wave 5: `std/` 層（Stage 4 以降）

| ファイル | 依存 | 注意 |
|---|---|---|
| `std/io.nepl` | core/ のみ | `StdErrorKind enum` の書き換え |
| `std/stdio.nepl` | io, streamio | `fn* unit -> unit` 形式へ |
| `std/streamio.nepl` | io | 最大ファイル（69 KB）。Linear resource（File）の Drop Elaboration 対応が必要 |
| `std/fs.nepl` | io, streamio | File / Result 中心の API |
| `std/test.nepl` | stdio, vec | テストフレームワーク書き換え |
| `std/env/cliarg.nepl` | core/ のみ | |

`File` / `Socket` 等の Linear resource は Stage 4（Resource IR）完了後に Drop Elaboration を活用して移行する。

### 3.7 Wave 6: `stdlib/nm/` と `stdlib/platforms/`（Stage 5）

- `nm/parser.nepl`（48 KB）: 構文書き換え
- `nm/html_gen.nepl`: 構文書き換え
- `platforms/wasix/tui.nepl`: 構文書き換え

### 3.8 Wave 7: `stdlib/neplg2/`（Stage 6）

セルフホストコンパイラ本体。`nepl-core-2.1` が安定してから本格着手（`doc/2.1impl/compiler_structure.md §7` 参照）。現行の `cli/main.nepl` と `core/{ast,parser,typecheck,diagnostic,span}.nepl` は NEPLg2.1 準拠で再設計する。

---

## 4. tests 移行計画

### 4.1 方針

- テストは**コンパイラが対応した段階でのみ追加・移行**する。
- 移行前のテスト（`.n.md` 内の NEPLg2.0 コードブロック）は `nepl-core` で動く状態を維持する。
- 移行後は `nepl-core-2.1` でのみ実行する。並行運用期間は両方のテストが存在してよい。

### 4.2 `tests/compiler/`（38 ファイル）

| 優先度 | ファイル群 | 移行タイミング |
|---|---|---|
| 1（基盤） | `functions.n.md`, `typeannot.n.md`, `if.n.md`, `pipe_operator.n.md` | Stage 2 |
| 2（名前解決） | `resolve.n.md`, `shadowing.n.md`, `generics.n.md`, `overload.n.md` | Stage 2–3 |
| 3（型・効果） | `move_check.n.md`, `move_effect.n.md`, `drop.n.md`, `trait_capability_copy.n.md` | Stage 3–4 |
| 4（コード生成） | `llvm_target.n.md`, `intrinsic.n.md`, `sizeof.n.md` | Stage 5 |
| 5（エラー診断） | `compile_fail_diag_location.n.md` | Stage 3 以降（diagnostics 安定後） |
| 6（セルフホスト） | `neplg2.n.md` | Stage 6 |

**各テストの書き換え内容**: コードフェンス内の NEPLg2.0 構文をセクション 2 のルールに従い変換。
`#import` → `use` / `fn` → `let ... %fn` / `struct/enum/trait/impl` → `let ... struct/enum/trait/impl` 等。

### 4.3 `tests/stdlib/`（38 ファイル）

対応する stdlib Wave が完了してから順次書き換える。

| Wave | 対象テストファイル |
|---|---|
| Wave 1 | `math.n.md`, `numerics.n.md`, `traits_hash.n.md`, `traits_order.n.md`, `traits_serde.n.md`, `traits_text.n.md` |
| Wave 3 | `string.n.md`, `sort.n.md` |
| Wave 4 | `vec.n.md`, `stack.n.md`, `queue.n.md`, `deque.n.md`, `binary_heap.n.md`, `bitset.n.md` 他コレクション群 |
| Wave 4 | `bloom_filter.n.md`, `fenwick.n.md`, `segment_tree.n.md`, `sparse_set.n.md`, `disjoint_set.n.md` 等 |
| Wave 5 | `io.n.md`, `fs.n.md`, `stdin.n.md`, `stdout.n.md`, `streamio.n.md` |
| Wave 6 | `nm.n.md` |
| 共通 | `memory_safety.n.md`（Phase 4 以降）, `proptest.n.md` |

---

## 5. tutorials 移行計画

### 5.1 方針

チュートリアルはコンパイル対象（実行可能コードを含む）と説明文の混在。

- **Stage 1 完了後**: 説明文・コード例の**文書書き換え**（コンパイル検証なし）を先行して行える。
- **Stage 3 完了後**: 実際に `nepl-core-2.1` でコンパイル・実行して正しさを確認する。

### 5.2 書き換え優先順

| グループ | ファイル | 書き換え難度 | ポイント |
|---|---|---|---|
| A（超基本） | 01–04（hello world〜strings） | 低 | `fn` → `let ... %fn`、`#import` → `use`、`#entry main` → `#entry` |
| B（型・パターン） | 05–06（Option, Result）、15（match） | 中 | enum バリアント修飾形、`Ok/Err/Some/None` の扱い |
| C（制御構造） | 07–08（while, if layouts） | 中 | `then`/`do` 補助マーカー削除、`:` + インデント統一 |
| D（モジュール） | 09, 17（import, namespace） | 中 | `#import` → `use`、`use path::*` → `use path as *` |
| E（型システム） | 20–21（generics, trait bounds） | 高 | `where` 節、juxtaposition 型適用、`%fn` 記法 |
| F（競プロ） | 22–27 | 高 | 大量のコード変換。標準ライブラリ移行後に対応 |

### 5.3 新規チュートリアルの追加

NEPLg2.1 では以下の新概念を説明するチュートリアルが必要（Stage 3 以降に新規作成）:

- **所有権と借用**: `&expr` / `&mut expr` 記法、move セマンティクス
- **Linear resource**: `File`/`Socket` の正しい使い方、Drop Elaboration の効果
- **Region と純粋永続値**: `List .T` の使い方、manual free なし
- **`noshadow let`**: オーバーロードとシャドウイングの使い分け
- **モジュールシステム**: `#module` / `#part` / `merge` の使い方

---

## 6. メモリモデル移行（Memory Phase 0–6）

`doc/compare/memory_model.md` の Phase 計画をコンパイラ Stage に対応付ける。

| Memory Phase | コンパイラ Stage | 内容 |
|---|---|---|
| **Phase 0** | Stage 2 | compiler に `InternalAlloc`/`ExternalIO` 分類を追加（stdlib 変更なし） |
| **Phase 1** | Stage 3 | `core/mem.nepl` の raw pointer 公開面を隔離（Wave 2 と同期） |
| **Phase 2** | Stage 4 | `List .T` を Region Inference 管理下へ。public `free` 廃止 |
| **Phase 3** | Stage 4 | `str` の ownership tracking 導入 |
| **Phase 4** | Stage 4 | Resource IR の実装と `alloc/` への適用 |
| **Phase 5** | Stage 5 | 全公開 API を `Result/Option` 安全 API に統一。`_raw`/`_safe` 接尾辞廃止 |
| **Phase 6** | Stage 5 | テスト回帰整備。`memory_safety.n.md` の compile_fail テスト追加 |

---

## 7. 進捗管理

### 7.1 ステータスラベル

| ラベル | 意味 |
|---|---|
| `waiting` | 対応コンパイラ Stage 未完了で着手不可 |
| `ready` | 着手可能（コンパイラ準備完了） |
| `in-progress` | 作業中 |
| `needs-test` | コード移行済み、テスト確認待ち |
| `done` | 移行完了・テスト通過確認済み |

### 7.2 テスト戦略

移行後の各モジュールは以下の 2 段階で検証する:

1. **構文確認**: `nepl-core-2.1` でコンパイルエラーなし
2. **動作確認**: `nodesrc/tests.js` で既存 stdlib テストが通過（`--runner wasm` または `--runner all`）

`nepl-core`（NEPLg2.0）での既存テスト通過を移行完了まで維持し、両コンパイラで CI を実行する。

### 7.3 現状まとめ

現時点（2026-03-17）では `nepl-core-2.1` は未着手（Stage 0）。
`doc/2.1impl/compiler_structure.md` の Stage 1 着手が移行全体の起点となる。
