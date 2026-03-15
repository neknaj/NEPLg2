# メモリ安全化 後方非互換移行計画

最終更新: 2026-03-15

> 目標仕様: `doc/purity_ownership_memory_spec.md`
> 本文書は、現在の unsafe 実装から統合仕様へ移行するための具体的な後方非互換変更計画を記述する。

## 0. 現状の問題

### 0.1 raw pointer の公開面積

| API | 使用元ファイル数 |
|-----|:----------:|
| `alloc_raw` | 23 |
| `dealloc_raw` | 21 |
| `mem_ptr_addr` | 29 |
| `mem_ptr_wrap` | 20 |

`MemPtr<T>` は導入済みだが、`mem_ptr_addr` で raw `i32` を取り出して直接 `load_i32` / `store_i32` する使い方が stdlib 全域に残っている。

### 0.2 compiler の effect 扱い

- builtins は `alloc/dealloc/realloc` を含め全て `Effect::Pure` で固定。
- `InternalAlloc` / `ExternalIO` の内部効果区別は未導入。
- `set` の purity は「局所なら pure」のまま。

### 0.3 List の unsafe

- `List<T>` は `struct List<.T>: ptr <i32>` — raw pointer を直接保有。
- public `free` が手動解放を要求し、呼び忘れでリーク、二重呼び出しで UAF。
- `tail` はノード列を共有するため、`free` と共有が矛盾する。

### 0.4 string の unsafe

- `alloc/string.nepl` は `RegionToken<u8>` 化済みだが、内部ヘルパは `mem_ptr_addr` で raw address に降りて `store_u8`/`load_u8` を直接呼ぶ。
- `str` 自体は runtime 側で raw `i32` として表現されており、ownership tracking がない。

---

## 1. Phase 0: 診断基盤の整備（非破壊）

### 目的

Phase 1 以降の破壊的変更を安全に進めるために、compiler 内部の効果分類と診断 ID を先に整備する。この段階では利用者向け API は変更しない。

### 変更

#### compiler

- `nepl-core/src/effect.rs`:
  `Effect` enum に `InternalAlloc`, `ExternalIO`, `Nondet`, `Unsafe` を追加する。
  surface への fold 関数 `to_surface_effect()` を追加する:
  `InternalAlloc → Pure`, `ExternalIO → Impure`, `Nondet → Impure`, `Unsafe → Impure`。

- `nepl-core/src/builtins.rs`:
  `alloc/dealloc/realloc/load/store` の builtins effect を `Effect::Pure` → `Effect::InternalAlloc` へ変更する。
  `Effect::InternalAlloc.to_surface_effect()` は `Pure` を返すため、既存コードの surface purity は変わらない。

- `nepl-core/src/diagnostics.rs`:
  memory safety 系の診断 ID 範囲を予約する（例: `5000-5099`）。
  - `5001`: moved value use
  - `5002`: raw address escape from safe boundary
  - `5003`: double free
  - `5004`: use after free

#### テスト

- `tests/compiler/effect_internal.n.md`: `InternalAlloc` が surface `Pure` に畳み込まれることの回帰テスト。

---

## 2. Phase 1: core/mem のポインタ隔離（破壊的）

### 目的

`mem_ptr_addr` / `mem_ptr_wrap` / `alloc_raw` / `dealloc_raw` / `realloc_raw` を公開面から除去し、safe API だけを残す。

### 破壊的変更

1. **`mem_ptr_addr` を non-public にする**
   - `MemPtr<T>` の `raw` field を compiler/runtime 内部のみに限定する。
   - 移行: すべての `mem_ptr_addr p` を `MemPtr<T>` のまま持ち回す設計に変更する。

2. **`mem_ptr_wrap` を non-public にする**
   - `alloc_ptr<T>` / `realloc_ptr<T>` / `region_ptr_at<T,U>` が代替。

3. **`alloc_raw` / `dealloc_raw` / `realloc_raw` を non-public にする**
   - 公開面には `alloc_ptr<T>` / `dealloc_ptr<T>` / `realloc_ptr<T>` と `alloc_region<T>` / `dealloc_region<T>` だけを残す。

4. **生 `i32` load/store を隠蔽する**
   - `load_i32(i32)` / `store_i32(i32,i32)` / `load_u8(i32)` / `store_u8(i32,i32)` を non-public にする。
   - `MemPtr<T>` オーバーロード版 `load_i32(MemPtr<i32>)` / `store_i32(MemPtr<i32>,i32)` のみ公開。

### 影響範囲と移行戦略

影響を受けるファイル群を依存の深い順に移行する:

#### 波 1: alloc/string（最重要、他の全 collection が依存）

- `string_alloc_region` / `string_finish` 系の内部ヘルパを `RegionToken<u8>` + `MemPtr<u8>` のみで完結させる。
- `concat`, `sb_build`, `str_slice`, `from_u128_radix`, `from_f64` の raw address 直接アクセスを `mem_ptr_add` + `store_u8(MemPtr<u8>,i32)` に置換。
- `str` 自身の表現は当面 raw `i32` のまま維持するが、`alloc/string` 外部からの直接操作は禁止。

#### 波 2: alloc/collections（Vec → Stack/Queue/Deque → その他）

- `Vec<T>`: `MemPtr<T>` field は導入済み。`mem_ptr_addr` 呼び出しを `mem_ptr_add` + typed load/store に置換。
- `Stack<T>`, `Queue<T>`, `Deque<T>`, `RingBuffer<T>`: 同様。
- `List<T>`: struct を `MemPtr<u8>` ベースに変更。`free` は残すが、`tail` による共有問題は Phase 3 で解決。
- `HashMap<K,V>`, `HashSet<K>`, `BTreeMap<K,V>`, `BTreeSet<K>`: 同様に typed pointer 化。
- `BitSet`, `AdjacencyMatrix`, `BloomFilter`, `CountingBloomFilter`: `MemPtr<u8>` header pointer 化。
- `Fenwick<T>`, `SegmentTree<T>`, `BinaryHeap<T>`, `DisjointSet`: 同様。
- `SparseSet`: raw `i32` header pointer を `MemPtr<u8>` に変更。

#### 波 3: alloc/diag

- `Diag` / `Diags` / `StdErrorKind` 内部の `Vec` 構築は波 2 で Vec が移行済みなら自動的に追従。
- `diag.nepl` / `error.nepl` 内の `alloc_raw` / `dealloc_raw` を `alloc_ptr<u8>` / `dealloc_ptr<u8>` に置換。

#### 波 4: std 層

- `std/streamio.nepl`, `std/stdio.nepl`, `std/fs.nepl`, `std/env/cliarg.nepl`: `mem_ptr_addr` / `mem_ptr_wrap` を除去。

#### 波 5: nm / kp / platforms

- `nm/parser.nepl`, `nm/html_gen.nepl`: Vec / string 経由の間接利用が主なので波 2 で大部分は解決。
- `kp/kpgraph.nepl`, `kp/kpsearch.nepl`, `kp/kpprefix.nepl`, `kp/kpfenwick.nepl`, `kp/kpdsu.nepl`: `alloc_raw` / `dealloc_raw` を typed API に置換。
- `platforms/wasix/tui.nepl`: 同様。

### 検証

- 各波ごとに `node nodesrc/tests.js -i stdlib -i tests --no-tree -o /tmp/tests-phaseN.json -j 15` で全テストを通す。
- `rg -n "alloc_raw|dealloc_raw|realloc_raw|mem_ptr_addr|mem_ptr_wrap" stdlib/` でゼロ件を確認する（core/mem 内部実装を除く）。

---

## 3. Phase 2: List の persistent 化と free 除去（破壊的）

### 目的

`List<T>` から手動 `free` を除去し、persistent list として設計する。

### 破壊的変更

1. **`free<T>` を削除する。**
   - `List<T>` のメモリは将来 Region Inference で自動管理する。
   - 移行期間中は「リークするが安全」とする。非破壊な代替として、scope 脱出時に compiler が `__drop_list<T>` を自動挿入する仕組みを後で Phase 5 で追加する。

2. **`tail` の セマンティクスを正式に「構造共有」と確定する。**
   - `tail` は既にノードを共有しているが、`free` と排他だったため暗黙に壊れていた。
   - `free` 除去により、`tail` による共有が安全になる。

3. **`List<T>` の内部表現を `MemPtr<u8>` に変更する。**
   - `struct List<.T>: ptr <i32>` → `struct List<.T>: ptr <MemPtr<u8>>` （空リストは `MemPtr` の raw=0）。

### 影響範囲

- `stdlib/alloc/collections/list.nepl` の全関数。
- `free<T>` を呼んでいるテストと例（doctest 含む）。
- `tutorials/` 内で `free` を使っている箇所。

### 検証

- `node nodesrc/tests.js -i stdlib/alloc/collections/list.nepl -i stdlib/tests/list.n.md --no-stdlib --no-tree -o /tmp/tests-list-persistent.json -j 4`
- `rg -n "free<" stdlib/alloc/collections/list.nepl` でゼロ件。

---

## 4. Phase 3: IO の effect 宣言化（破壊的）

### 目的

IO 操作を文字列マーカーベースから `ExternalIO` 効果宣言へ移行する。

### 破壊的変更

1. **`std/stdio.nepl` の IO 関数を `ExternalIO` 効果付きに変更。**
   - compiler intrinsic の IO 系 (`fd_read`, `fd_write`, `args_get`, `clock_time_get` 等) を `Effect::ExternalIO` に変更。
   - surface fold で `Impure` になるため、既存の `*>` 関数は互換。`->` で定義されていた IO 関数があれば `*>` に変更。

2. **`std/fs.nepl`, `std/env/cliarg.nepl` も同様。**

### 影響範囲

- Pure 関数内で誤って IO を呼んでいた箇所がコンパイルエラーになる（これは意図通り）。

### 検証

- `node nodesrc/tests.js -i stdlib -i tests -i tutorials --no-tree -o /tmp/tests-io-effect.json -j 15`
- IO を含む test case が `*>` 前提で正しく通ることを確認。

---

## 5. Phase 4: set の purity 規則変更（破壊的）

### 目的

`set` の「局所なら pure」規則を廃止し、escape analysis ベースの新規則へ変更する。

### 破壊的変更

- compiler の `set` purity 判定を以下の条件に変更:
  1. 更新対象が unique local state である。
  2. その状態への参照が escape しない。
  3. 共有 borrow が存在しない。
  4. 更新結果が観測可能な raw identity を漏らさない。

### 影響範囲

- 変数を capture して closure に渡しつつ set している箇所がコンパイルエラーになる可能性。
- 影響の特定には全テスト実行が必要。

### 検証

- `node nodesrc/tests.js -i stdlib -i tests -i tutorials --no-tree -o /tmp/tests-set-purity.json -j 15`

---

## 6. Phase 5: Resource IR + Drop Elaboration の導入

### 目的

compiler に Resource IR を導入し、owned resource の自動 drop を実現する。

### 変更

1. **Resource IR パス:**
   - typed HIR → Resource IR に変換。
   - ownership / borrow / region / drop 情報を IR ノードに付与。
   - scope exit / overwrite 時に `drop` call を自動挿入。

2. **Drop 候補の対象型:**
   - 初期対象: `Vec<T>`, `Stack<T>`, `Queue<T>`, `Deque<T>`, `StringBuilder`, `ByteBuf`
   - `List<T>` は region inference 導入まで drop 対象にしない（構造共有があるため）。

3. **Drop trait:**
   - `Drop<T>` trait を `core/traits` に導入。
   - `impl Drop for Vec<T>` 等を stdlib に追加。

### 影響範囲

- 手動 `free` / `sb_free` 等を呼んでいる箇所は不要になる → 段階的に削除。
- 二重解放はコンパイルエラーになる。

### 検証

- drop 挿入の compile_fail テスト。
- 既存テスト全通し。

---

## 7. Phase 6: Region Inference の first version

### 目的

pure persistent value (`List<T>`, `str`, immutable tree) の region 単位 bulk free を実現する。

### 変更

1. **Region allocator の導入:**
   - `core/mem` に region bump allocator を追加。
   - persistent value の alloc は region allocator を使う。

2. **compiler の region 推論:**
   - pure value の生存期間を静的に推論。
   - scope 脱出点で region を一括解放。

3. **`List<T>` の Region 統合:**
   - Phase 2 で手動 free を除去した List を region 管理下に置く。

### 影響範囲

- 利用者側の変更はなし（内部最適化）。

### 検証

- region 解放のタイミングテスト。
- メモリ使用量の regression 確認。

---

## 8. 全体の移行順序まとめ

```text
Phase 0: 診断基盤               ← 非破壊、他の全 phase の前提
Phase 1: core/mem ポインタ隔離   ← 最大規模の破壊的変更
Phase 2: List persistent 化     ← Phase 1 完了後
Phase 3: IO effect 宣言化       ← Phase 0 完了後（Phase 1 と並行可）
Phase 4: set purity 変更         ← Phase 0 完了後（Phase 1 と並行可）
Phase 5: Resource IR + Drop     ← Phase 1, 2 完了後
Phase 6: Region Inference        ← Phase 5 完了後
```

Phase 0 → {Phase 1, Phase 3, Phase 4} → Phase 2 → Phase 5 → Phase 6

### 各 Phase の規模見積もり

| Phase | 変更ファイル数 | 破壊度 |
|:-----:|:---------:|:-----:|
| 0 | ~5 (compiler) | なし |
| 1 | ~30 (stdlib) + ~3 (compiler) | 大 |
| 2 | ~5 (stdlib) | 中 |
| 3 | ~5 (compiler) + ~5 (stdlib) | 小 |
| 4 | ~3 (compiler) | 小 |
| 5 | ~10 (compiler) + ~15 (stdlib) | 大 |
| 6 | ~5 (compiler) + ~3 (stdlib) | 中 |

## 9. 移行中の互換性方針

- 移行期間中は旧 API と新 API が一時的に併存しうる。
- ただし旧 API に `#[deprecated]` 相当の warning を出す仕組みは設けず、Phase 完了時に一括削除する。
- 各 Phase 完了時に `todo.md` を更新し、`note.n.md` に結果を記録する。
- テストが全通しすることを各 Phase の完了条件とする。

---

## 10. Compiler による所有権・線形性・純粋性検査の設計

### 10.0 現在の Compiler アーキテクチャ（起点）

本節の設計は現行 compiler の以下のデータ構造・パスを起点とする。

| 既存 | 場所 | 現状 |
|------|------|------|
| `Effect` enum | `ast.rs:12` | `Pure` / `Impure` の 2 値のみ |
| Effect 推論 | `effects.rs` | WASI syscall 名の文字列マーカー照合のみ |
| `Binding.moved` | `typecheck.rs` | field は存在するが dataflow 追跡なし |
| `TraitCapability` | `ast.rs:157` | `Copy`, `Clone`, `Drop` の 3 variant |
| `is_copy_impl_eligible` | `typecheck.rs:1111` | Copy impl の静的充足判定 |
| `register_drop_impl_target` | `typecheck.rs:1193` | Drop impl target の登録のみ、drop 挿入なし |
| `HirExprKind::Drop` | `hir.rs:145` | HIR ノードは定義済みだが codegen 以外で生成されない |
| `HirExprKind::Set` | `hir.rs:134` | `name` + `value` のみ、purity 検査ロジックなし |

### 10.1 値の 3 分類と型レベル分類子

統合仕様の値 3 分類を compiler 内部で識別するため、型に ValueCategory を付与する。

```rust
// nepl-core/src/types.rs に追加
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueCategory {
    /// immutable, freely shareable, region-managed
    PurePersistent,
    /// mutable, single-owner, move semantics
    UniqueMutable,
    /// linear, consume-exactly-once (e.g. File, Socket)
    LinearCapability,
}
```

**分類規則:**

| 条件 | カテゴリ |
|------|---------|
| `impl Copy for T` 登録済み | `PurePersistent` |
| `impl Drop for T` 登録済み かつ Copy なし | `LinearCapability` |
| 上記以外（`let mut` で使われる型） | `UniqueMutable` |

Phase 0 では分類子の登録のみ行い、Phase 5 で実際の検査に利用する。

### 10.2 内部 Effect 拡張

Phase 0 で `ast.rs` の `Effect` enum を以下に拡張する:

```rust
pub enum Effect {
    Pure,
    Impure,
    // --- 以下 compiler 内部のみ ---
    InternalAlloc,
    ExternalIO,
    Nondet,
    Unsafe,
}

impl Effect {
    /// 利用者向け surface effect に畳み込む
    pub fn to_surface(&self) -> Effect {
        match self {
            Effect::InternalAlloc => Effect::Pure,
            Effect::ExternalIO => Effect::Impure,
            Effect::Nondet => Effect::Impure,
            Effect::Unsafe => Effect::Impure,
            other => *other,
        }
    }
}
```

**影響する既存コード:**

- `effects.rs`: `intrinsic_effect()` / `raw_body_effect()` — IO マーカー一致を `ExternalIO` に、それ以外で alloc 系なら `InternalAlloc` に分類する。
- `builtins.rs`: `alloc/dealloc/realloc/load/store` の `Effect::Pure` を `Effect::InternalAlloc` に変更する。
- `typecheck.rs`: 関数シグネチャの effect 判定で `to_surface()` を経由するようにする。これにより既存の surface purity は変わらない。
- `codegen_wasm.rs` / `codegen_llvm.rs`: effect を見て分岐している箇所は `to_surface()` で比較するよう変更する。

### 10.3 Move / Ownership 追跡

現在の `Binding.moved: bool` は単一フラグで、条件分岐やループを考慮しない。以下の設計でこれを拡張する。

#### 10.3.1 MoveState dataflow

```rust
// nepl-core/src/ownership.rs (新規ファイル)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarState {
    /// 初期化済み、使用可能
    Live,
    /// move 済み、再使用不可
    Moved,
    /// 条件分岐により不定（一方で move、もう一方で live）
    MaybeMoved,
    /// 未初期化
    Uninitialized,
    /// 共有 borrow 中（mutable access 禁止）
    BorrowedShared { borrower_count: u32 },
    /// 一意 borrow 中（他の全 access 禁止）
    BorrowedUnique,
}

/// 関数本体の各変数について状態を追跡する
pub struct OwnershipChecker {
    /// variable name → current state
    states: BTreeMap<String, VarState>,
}
```

**検査タイミング:** `check_function()` (`typecheck.rs:1990`) の後、HIR が完成した段階で `ownership_check(hir_function)` を走らせる。

**検査ルール:**

1. `HirExprKind::Var(name)`: 参照時に `states[name]` が `Moved` なら診断 5001 を出す。
2. `HirExprKind::Set { name, .. }`: `name` の型が `Copy` でない場合、旧値は move → drop 必要。
3. `HirExprKind::Call { args, .. }`: 引数として渡された non-Copy 変数を `Moved` に遷移させる。
4. `HirExprKind::If / Match`: 各分岐で独立に state を進め、join 点で不一致なら `MaybeMoved` にする。
5. `HirExprKind::While`: ループ本体を 2 回走査して fixed-point を取る。
6. `HirExprKind::AddrOf(expr)`: `expr` が変数なら `BorrowedShared` / `BorrowedUnique` に遷移させる。
7. borrow 中の変数への `set` → 診断 5007 "cannot mutate while borrowed"。
8. `BorrowedUnique` な変数の他アクセス → 診断 5008 "cannot access while uniquely borrowed"。

**導入 Phase:** Phase 5（Resource IR 導入時）。Phase 0–4 では `states` を構築するだけで検査はしない（dry-run mode）。

#### 10.3.2 Linear Capability の消費検査

`ValueCategory::LinearCapability` の変数は正確に 1 回消費されなければならない。

- scope 終端で `Live` の linear 変数が残っている → 診断 5005 "linear value not consumed"
- `Moved` + 再使用 → 診断 5001 (moved value use)
- `MaybeMoved` で scope 終端 → 診断 5006 "linear value may not be consumed on all paths"

### 10.4 set の Escape Analysis

現在の `set` は `HirExprKind::Set { name, value }` として HIR に入り、purity の特別扱いはない。統合仕様の新規則を実装する。

#### 10.4.1 分析パス

`typecheck.rs` の `check_function` 内に escape analysis パスを追加する。

```rust
// nepl-core/src/escape_analysis.rs (新規ファイル)

/// 変数が「局所 unique」であるかを判定する
pub fn is_locally_unique(func: &HirFunction, var_name: &str) -> bool {
    // 以下の条件を全て満たすこと:
    // 1. var_name が mutable local である
    // 2. var_name が closure に capture されていない
    // 3. var_name が関数の返り値として leak しない
    // 4. var_name への参照 (&var_name) が取られていない
    // 5. var_name が別の変数に alias されていない
}
```

**判定ロジック:**

1. HIR を走査し、各 `Set { name, .. }` について `name` の流れを追う。
2. `name` が `HirExprKind::Call` の引数に渡されている場合:
   - callee が `Pure` → OK（値コピーなので escape しない）
   - callee が `Impure` かつ callee の引数型が non-Copy → escape する
3. `name` が `HirExprKind::AddrOf` の対象 → escape する（参照が取られている）
4. `name` が closure の captures に含まれている → escape する

**Phase 4 での使用:** `set` が pure 関数内にある場合、`is_locally_unique` が true のときのみ許可する。false なら診断を出す。

### 10.5 Drop Elaboration（Phase 5 詳細設計）

#### 10.4.2 Resource IR の命令

統合仕様 (§11.1) に合わせ、Resource IR は最低限以下の命令を持つ:

```text
move x -> y
borrow_shared x -> b
borrow_unique x -> b
region_new ρ
region_alloc ρ, n
region_end ρ
drop x
io_open path
io_write h, data
io_close h
```

この IR 上で use-after-move / double free / use-after-free / borrow conflict / leaked linear token / unclosed external resource を診断する。

#### 10.5.1 パイプライン位置

```text
Source → parse → typecheck (HIR生成) → ownership_check → drop_elaborate → monomorphize → codegen
```

drop_elaborate は ownership_check の直後、monomorphize の前に入る。

#### 10.5.2 drop 挿入アルゴリズム

```rust
// nepl-core/src/drop_elaborate.rs (新規ファイル)

/// HIR を走査し、scope exit / overwrite 時に Drop::drop 呼び出しを挿入する
pub fn elaborate_drops(module: &mut HirModule) {
    for func in &mut module.functions {
        let mut inserter = DropInserter::new();
        inserter.visit_body(&mut func.body);
    }
}
```

**挿入規則:**

1. **scope exit:** `HirBlock` の最終行の後ろに、そのブロックで定義された non-Copy ローカル変数の `HirExprKind::Drop { name }` を逆順に挿入する。

2. **set overwrite:** `HirExprKind::Set { name, value }` の直前に、旧値の `Drop` を挿入する（`name` の型が `impl Drop` の場合のみ）。

3. **条件付き drop:** `If` / `Match` の各分岐で異なる変数が live → join 点に条件付き drop を挿入する。

4. **既存の `HirExprKind::Drop` との整合:** 既に codegen が `Drop` ノードを生成する箇所があるため、二重挿入を防ぐフラグ `drop_already_handled` を追加する。

#### 10.5.3 Drop trait と codegen

```text
trait Drop:
    #[capability: drop]
    fn drop <(Self)*>()> (self):
        ...
```

codegen は `HirExprKind::Drop { name }` を見たとき:
1. `name` の型に `impl Drop` があれば → trait method call を生成。
2. なければ → no-op（Copy 型など）。

### 10.6 Region Inference（Phase 6 詳細設計）

#### 10.6.1 Region 表現

```rust
// nepl-core/src/region.rs (新規ファイル)
pub type RegionId = u32;

#[derive(Debug, Clone)]
pub struct RegionInfo {
    pub id: RegionId,
    /// この region に属する allocation の一覧
    pub allocs: Vec<AllocSite>,
    /// region の生存期間が終わる scope
    pub end_scope: ScopeId,
}
```

#### 10.6.2 推論アルゴリズム

1. pure persistent value の alloc site を収集する。
2. 各 alloc site の生存期間を、最後の use point まで伝播する。
3. 同じ scope で終わる alloc site を同一 region にまとめる。
4. scope exit 時に region 一括解放コードを挿入する。

**制約:** closure で capture された値は外側 scope の region に属する。

#### 10.6.3 codegen

alloc の呼び出しを:
- `alloc_raw size` → `region_bump_alloc region_id size`

scope exit で:
- `region_free_all region_id`

### 10.7 Compiler パス全体の最終的な順序

```text
Source
 ↓ parse
AST
 ↓ name_resolve / target_precheck
 ↓ typecheck (型推論 + HIR 生成 + effect 判定)
HIR (typed)
 ↓ value_category_assign     ← Phase 0 で追加
 ↓ escape_analysis            ← Phase 4 で追加（set purity 用）
 ↓ ownership_check            ← Phase 5 で追加（move / linear 検査）
 ↓ drop_elaborate             ← Phase 5 で追加
 ↓ region_inference           ← Phase 6 で追加
HIR (elaborated)
 ↓ monomorphize
Mono HIR
 ↓ codegen_wasm / codegen_llvm
Output
```

各パスは独立したファイル（`ownership.rs`, `escape_analysis.rs`, `drop_elaborate.rs`, `region.rs`）に分離し、`compiler.rs` のパイプラインから順番に呼び出す。

### 10.8 ランタイム差異の吸収（Wasm / LLVM）

統合仕様 (§12) に従い、安全意味論と物理レイアウトを分離する。

#### 揃えるもの（Resource IR より上で保証）

- moved value の再使用禁止
- borrowed place への不正 mutation 禁止
- freed resource の再使用禁止
- pure / impure の境界
- `str`, `List`, `OwnedBuf`, `File`, `Socket` の source semantics

これらは全て Resource IR ～ drop elaboration のパスで検査完了するため、codegen に入る前に保証される。target lowering は safety-proven な IR を受け取るだけでよい。

#### 揃えないもの（target lowering で分岐）

| 項目 | Wasm | LLVM |
|------|------|------|
| pointer 表現 | linear memory offset (`i32`) | native pointer |
| allocator | `core/mem` の bump+free_list | libc malloc / custom |
| `str` header | `[len:i32][data...]` in linear memory | native `{ptr, len}` |
| file/socket handle | WASI `fd` (`i32`) | POSIX fd or OS handle |
| region bulk free | linear memory range free | native free |

#### 実装方針

- `codegen_wasm.rs` と `codegen_llvm.rs` は elaborated HIR を入力とし、target 固有のメモリ表現のみを担う。
- 現行の `codegen_llvm.rs` が `TypeExpr::Reference` を `LlTy::I32` に寄せている箇所は、Phase 2 (Wasm/LLVM 表現分離) で native pointer として扱うよう変更する。
- 安全意味論の検査を codegen に持ち込まない。診断は全て Resource IR パスで完結させる。
