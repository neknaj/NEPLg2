# stdlib / tests / tutorials NEPLg3 移行計画 draft

この文書は NEPLg3 向けの過去計画であり、2026-05-24 時点の NEPLg2.1 表層構文移行の正ではない。NEPLg2.1 では `stdlib/`、`tests/`、`tutorials/` を同一ディレクトリ内で更新する。現在の移行方針は [NEPLg2.1 surface syntax migration plan](../neplg2/neplg21_syntax_migration_plan.md) を参照する。

最終更新: 2026-03-17

---

## 1. 基本方針：並行ディレクトリ開発 → 一括切り替え

`nepl-core-g3` がコンパイラとして `nepl-core` と並行開発されるのと同じ方式を stdlib・tests・tutorials にも適用する。

```
現行（NEPLg2.0）          開発中（NEPLg3）
─────────────────         ─────────────────
stdlib/               ←→  stdlib-g3/
tests/                ←→  tests-g3/
tutorials/            ←→  tutorials-g3/
```

- **`stdlib/`・`tests/`・`tutorials/` は一切変更しない**。`nepl-core` でビルド・テストが通る状態を Stage 6 切り替えまで維持する。
- **`stdlib-g3/`・`tests-g3/`・`tutorials-g3/` を新規作成**し、`nepl-core-g3` でのみコンパイルする。
- **Stage 6 到達時に一括切り替え**（旧ディレクトリを archive してリネーム）。

### 1.1 ディレクトリ構成

```
/
├── stdlib/             # NEPLg2.0 stdlib（凍結。nepl-core でビルド）
├── stdlib-g3/          # NEPLg3 stdlib（新規。nepl-core-g3 でビルド）
│
├── tests/              # NEPLg2.0 テスト（凍結。nepl-core で実行）
├── tests-g3/           # NEPLg3 テスト（新規。nepl-core-g3 で実行）
│
├── tutorials/          # NEPLg2.0 チュートリアル（凍結）
├── tutorials-g3/       # NEPLg3 チュートリアル（新規。nepl-core-g3 で実行）
│
├── nepl-core/          # NEPLg2.0 コンパイラ（凍結）
└── nepl-core-g3/       # NEPLg3 コンパイラ（開発中）
```

### 1.2 Stage 6 切り替え手順

Stage 6（`nepl-core-g3` が全テスト通過）到達後:

```
1. stdlib-g3/ のテストが全通過することを確認
2. mv stdlib/        stdlib-archive/
   mv stdlib-g3/    stdlib/
3. mv tests/         tests-archive/
   mv tests-g3/     tests/
4. mv tutorials/     tutorials-archive/
   mv tutorials-g3/ tutorials/
5. nepl-core-g3 での CI が緑であることを確認
6. stdlib-archive/ tests-archive/ tutorials-archive/ を削除
```

切り替えは 1 コミット（ファイル移動のみ）で行い、容易に revert できる状態を保つ。

---

## 2. `stdlib-g3/` の構造

現行 `stdlib/` のディレクトリ構造を維持しつつ、ファイルヘッダと全構文を NEPLg3 形式で記述する。

```
stdlib-g3/
    core/
        traits/
            eq.nepl
            ord.nepl
            hash.nepl
            hash_key.nepl
            copy.nepl
            drop.nepl
            debug.nepl
            stringify.nepl
            serialize.nepl
            deserialize.nepl
        mem.nepl
        math.nepl
        option.nepl
        result.nepl
        cast.nepl
        field.nepl
        test.nepl
        rand/
            xorshift32.nepl
    std/
        io.nepl
        stdio.nepl
        streamio.nepl
        fs.nepl
        iotarget.nepl
        test.nepl
        prelude_base.nepl
        env/
            cliarg.nepl
    alloc/
        string.nepl
        io.nepl
        collections/
            vec.nepl
            stack.nepl
            queue.nepl
            deque.nepl
            list.nepl
            ringbuffer.nepl
            hashmap.nepl
            hashset.nepl
            btreemap.nepl
            btreeset.nepl
            binary_heap.nepl
            adjacency_matrix.nepl
            sparse_set.nepl
            bloom_filter.nepl
            counting_bloom_filter.nepl
            bitset.nepl
            fenwick.nepl
            segment_tree.nepl
            disjoint_set.nepl
        hash/
            fnv1a32.nepl
            hash32.nepl
            sha256.nepl
        encoding/
            json.nepl
        diag/
            diag.nepl
            error.nepl
    nm/
        README.n.md
        parser.nepl
        html_gen.nepl
    platforms/
        wasix/
            tui.nepl
    neplg3/          ← Stage 6 以降（doc/neplg3/impl/compiler_structure.md §4 参照）
```

### 2.1 各ファイルの先頭ヘッダ

`stdlib-g3/` の全ファイルは NEPLg3 モジュールヘッダで始まる。

```nepl
#module         // anchor ファイル（core/、std/、alloc/ 等のルート）
```

```nepl
#part           // merge されるパートファイル（ほとんどの .nepl）
```

---

## 3. `tests-g3/` の構造

```
tests-g3/
    compiler/       # nepl-core-g3 の言語機能テスト（NEPLg3 構文）
    stdlib/         # stdlib-g3/ の動作テスト
```

テスト形式は現行 `tests/` と同じ `.n.md` ファイル。`nodesrc/tests.js` に `--stdlib stdlib-g3 --compiler nepl-core-g3` のようなオプションを追加して両方を CI で実行する。

---

## 4. `tutorials-g3/` の構造

```
tutorials-g3/
    getting_started/
        00_index.n.md
        01_hello_world.n.md
        ...（既存 28 ファイル対応）
        29_ownership_and_borrow.n.md     # NEPLg3 新規
        30_linear_resource.n.md          # NEPLg3 新規
        31_region_and_persistent.n.md    # NEPLg3 新規
        32_module_system.n.md            # NEPLg3 新規
        33_overload_and_redefinition.n.md # NEPLg3 新規
```

---

## 5. コンパイラ Stage と開発解禁範囲

| コンパイラ Stage | 達成内容 | `stdlib-g3/` 着手可能範囲 | `tests-g3/` 着手可能範囲 | `tutorials-g3/` 着手可能範囲 |
|---|---|---|---|---|
| **Stage 0**（現在） | 未着手 | — | — | 文書のみ（コンパイル不要）先行作成可 |
| **Stage 1** | 字句解析・パーサ・モジュール骨格 | 構文確認のみ（型検査なし） | コンパイルエラー系テスト | 01–04（hello world 〜 strings）文書確定 |
| **Stage 2** | 名前解決・型システム基礎 | `core/traits/`・`core/option`・`core/result`・`core/cast` | `functions`, `typeannot`, `if`, `resolve` | A グループ（01–04）通過確認 |
| **Stage 3** | 型検査・trait 検査・effect 検査 | `core/` 全体・`core/mem`（raw pointer 隔離） | `generics`, `overload`, `shadowing`, `move_check`, `trait_capability_copy` | B〜D グループ（05–09, 15, 17） |
| **Stage 4** | Resource IR・ownership/borrow/region/drop | `alloc/string`・`alloc/collections/` 全体・`std/` | `move_effect`, `drop`, `memory_safety` | E グループ（20–21）・新規 29–33 |
| **Stage 5** | 単相化・コード生成 | `nm/`・`platforms/`・`core/rand/` | `llvm_target`, `intrinsic`, 全 stdlib テスト | F グループ（22–27 競プロ）。全チュートリアル通過確認 |
| **Stage 6** | 切り替え完了 | `neplg3/` セルフホスト本格着手 | `neplg3.n.md` | — |

---

## 6. 開発順序（Wave）

### Wave 1: `core/traits/` と基本型（Stage 2）

対象: `stdlib-g3/core/traits/` 全 10 ファイル + `option.nepl`, `result.nepl`, `cast.nepl`, `field.nepl`, `test.nepl`

主な変換パターン:

| NEPLg2.0 | NEPLg3 |
|---|---|
| `trait Eq:` | `let Eq trait:` |
| `fn eq <(Self,Self)->bool> (a, b):` | `let eq %fn Self fn Self bool \a \b ...` |
| `impl Eq for i32:` | `let i32 impl for Eq:` |
| `enum Option<.T>:` | `let Option enum .T:` |
| `Ok <.T>` / `Err <.E>`（バリアント定義）| `Ok .T` / `Err .E` |
| bare `Some x` / `None`（期待型なし） | `Option::Some x` / `Option::None` |

### Wave 2: `core/mem.nepl` と raw pointer 隔離（Stage 3 + Memory Phase 1）

`stdlib-g3/core/mem.nepl` を**最初から** raw pointer 非公開 API で書く（移行でなく新規設計）。

- `mem_ptr_addr` / `mem_ptr_wrap` / `alloc_raw` / `dealloc_raw` は `private`
- 公開 API は `MemPtr .T` ベースのオーバーロードのみ
- `alloc/dealloc` の effect は `InternalAlloc` で宣言（コンパイラが `Pure` に fold）

### Wave 3: `alloc/string.nepl` と `alloc/collections/vec.nepl`（Stage 3–4）

`stdlib/` の既存コードを参照しながら `stdlib-g3/` に新規作成。
内部の raw address 操作は Wave 2 の隠蔽 API を使う。

### Wave 4: `alloc/collections/` 残り（Stage 4）

vec 完了後に順次追加。`list.nepl` は Memory Phase 2（Region Inference）完了後。

```
依存順: vec → stack/queue/deque/ringbuffer
            → hashmap/hashset（Hash trait 依存）
            → btreemap/btreeset/binary_heap（Ord trait 依存）
            → list（Region Inference 必須）
            → その他（fenwick, segment_tree …）
```

### Wave 5: `std/` 層（Stage 4）

`File`/`Socket` 等の Linear resource に Drop Elaboration が効く Stage 4 完了後に着手。

| ファイル | 主要変更点 |
|---|---|
| `std/io.nepl` | `StdErrorKind enum` 修飾形 / 関数シグネチャ |
| `std/stdio.nepl` | `fn* () ()` を `%...` で前置した型注釈付き lambda へ書き換え |
| `std/streamio.nepl` | `File` を Linear resource として扱う。Drop Elaboration を前提に手動 close 不要化 |
| `std/fs.nepl` | `Result` / `Option` ベース API の徹底 |
| `std/test.nepl` | テストフレームワーク全体の書き換え |

### Wave 6: `nm/` と `platforms/`（Stage 5）

`nm/parser.nepl`（48 KB）・`nm/html_gen.nepl`・`platforms/wasix/tui.nepl` を新規作成。

### Wave 7: `neplg3/`（Stage 6）

`doc/neplg3/impl/compiler_structure.md §4` の設計に従いセルフホストコンパイラを新規実装。

---

## 7. メモリモデル移行（Memory Phase 0–6）と Wave の対応

| Memory Phase | コンパイラ Stage | Wave | 内容 |
|---|---|---|---|
| **Phase 0** | Stage 2 | — | compiler 内に `InternalAlloc`/`ExternalIO` 分類追加。stdlib 変更なし |
| **Phase 1** | Stage 3 | Wave 2 | `stdlib-g3/core/mem.nepl` を最初から raw pointer 非公開で設計 |
| **Phase 2** | Stage 4 | Wave 4 | `stdlib-g3/alloc/collections/list.nepl` を Region Inference 前提で設計 |
| **Phase 3** | Stage 4 | Wave 3 | `stdlib-g3/alloc/string.nepl` の ownership tracking |
| **Phase 4** | Stage 4 | Wave 3–5 | Resource IR を前提とした `alloc/` 全体の設計 |
| **Phase 5** | Stage 5 | Wave 3–6 | 全公開 API を `Result/Option` 安全 API に統一。`_raw`/`_safe` 接尾辞なし |
| **Phase 6** | Stage 5 | — | `tests-g3/stdlib/memory_safety.n.md` compile_fail テスト追加 |

---

## 8. CI 戦略

### 並行運用期間（Stage 1–5）

```
CI job A: nepl-core + stdlib/ + tests/ （NEPLg2.0。常時グリーンを維持）
CI job B: nepl-core-g3 + stdlib-g3/ + tests-g3/ （NEPLg3。Wave 進捗で増加）
```

Job B は Wave ごとに対象ファイルを追加していく。Job A は Stage 6 切り替えまで壊さない。

### 切り替え後（Stage 6）

Job A を廃止し、Job B のみで CI を継続。旧ディレクトリ（`stdlib-archive/` 等）は 1 リリースサイクル後に削除。

---

## 9. 構文変換クイックリファレンス

（詳細は `doc/compare/syntax.md`。ここでは `stdlib-g3/` 作業中に頻出するパターンのみ）

| NEPLg2.0 | NEPLg3 |
|---|---|
| `fn name <TypeParams> <Sig> (params):` | `let name <expr>` |
| `struct Name<.T>:` / `enum Name<.T>:` | `let Name struct .T:` / `let Name enum .T:` |
| `trait Name:` / `impl Trait for Type:` | `let Name trait:` / `let Type impl for Trait:` |
| `Vec<i32>` / `Option<.T>` / `Result<T, E>` | `Vec i32` / `Option .T` / `Result .T .E` |
| `(A) -> B` / `(A) *> B` | `fn A B` / `fn* A B` |
| `(A, B) -> C` | `fn A fn B C` |
| `()` （unit） | `()` |
| `<TypeExpr>` （型注釈） | `%TypeExpr <expr>` |
| `(a, b):` / `():` | `\a \b ...` / `\()` |
| `if cond then ... else ...` | `if cond a b` または `if cond then a else b` |
| `while cond do ...` | `while cond : ...` |
| `#import "stdlib/std/streamio"` | `use std::streamio` |
| `#include "./path"` | `merge "./path"` |
| `#entry main` | `#entry` |
| `use path::*` | `use path as *` |

---

## 10. 現状と起点

現時点（2026-03-17）では `nepl-core-g3` は未着手（Stage 0）。
`stdlib-g3/`・`tests-g3/`・`tutorials-g3/` のディレクトリはまだ存在しない。

**起点となるアクション:**

1. `nepl-core-g3/` の Stage 1 着手（`doc/neplg3/impl/compiler_structure.md §7`）
2. `tutorials-g3/getting_started/` の文書先行作成（コンパイル不要。Stage 0 から可能）
3. Stage 1 完了後に `stdlib-g3/` ディレクトリを作成し Wave 1 を開始
