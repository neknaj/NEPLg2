# NEPLg2.0 → NEPLg3 メモリモデル移行

---

## 1. NEPLg2.0 の現状の問題

### 1.1 raw pointer の公開面積

| API | 使用元ファイル数（概算） |
|-----|:---:|
| `alloc_raw` | 23 |
| `dealloc_raw` | 21 |
| `mem_ptr_addr` | 29 |
| `mem_ptr_wrap` | 20 |

`MemPtr .T` は導入済みだが、`mem_ptr_addr` で raw `i32` を取り出して直接 `load_i32` / `store_i32` する使い方が stdlib 全域に残っている。

### 1.2 compiler の effect 扱い

- builtins は `alloc/dealloc/realloc` を含め全て `Effect::Pure` で固定。
- `InternalAlloc` / `ExternalIO` の内部効果区別は未導入。

### 1.3 `List .T` の unsafe

- `List .T` は raw pointer を直接保有。
- public `free` が手動解放を要求し、呼び忘れでリーク、二重呼び出しで UAF。
- `tail` はノード列を共有するため、`free` と共有が矛盾する。

### 1.4 `str` の unsafe

- `alloc/string.nepl` は内部ヘルパが `mem_ptr_addr` で raw address に降りて `store_u8`/`load_u8` を直接呼ぶ。
- `str` 自体は runtime 側で raw `i32` として表現されており、ownership tracking がない。

---

## 2. 移行計画

### Phase 0: 診断基盤の整備（非破壊）

目的: Phase 1 以降の破壊的変更を安全に進めるために、compiler 内部の効果分類と診断 ID を先に整備する。この段階では利用者向け API は変更しない。

- `Effect` enum に `InternalAlloc`, `ExternalIO`, `Nondet`, `Unsafe` を追加する。
- surface への fold 関数 `to_surface_effect()` を追加する（`InternalAlloc → Pure`, `ExternalIO → Impure`）。
- `alloc/dealloc/realloc/load/store` の effect を `Effect::Pure` → `Effect::InternalAlloc` へ変更。
- memory safety 系の診断 ID 範囲を予約（例: `5000-5099`）。

### Phase 1: core/mem のポインタ隔離（破壊的）

目的: `mem_ptr_addr` / `mem_ptr_wrap` / `alloc_raw` / `dealloc_raw` / `realloc_raw` を公開面から除去する。

破壊的変更:
1. `mem_ptr_addr` を non-public にする。
2. `mem_ptr_wrap` を non-public にする。
3. `alloc_raw` / `dealloc_raw` / `realloc_raw` を non-public にする。
4. 生 `i32` load/store（`load_i32(i32)` 等）を隠蔽する。`MemPtr .T` オーバーロード版のみ公開。

移行波:
- 波 1: `alloc/string`（最重要、他の全 collection が依存）
- 波 2: `alloc/collections`（Vec → Stack/Queue/Deque → その他）
- 波 3: `alloc/diag`
- 波 4: `std` 層

### Phase 2: `List .T` の再設計（破壊的）

- `List .T` を Region Inference 管理下に移行する。
- public `free` を廃止する。
- `tail` の共有問題を解消する（ノード単位の free をやめ、region 単位の bulk free へ移行）。

### Phase 3: `str` の所有権追跡（破壊的）

- `str` の ownership tracking を導入する。
- raw `i32` 表現から `MemPtr .T` ベースの内部表現へ移行する。
- safe user code への raw address 露出を完全に禁止する。

### Phase 4: Resource IR の導入

- typed HIR の後ろに Resource IR を配置する。
- ownership / borrow / region / drop の解析パスを整備する。

### Phase 5: stdlib 安全 API への統一

- すべての公開 API を `Result/Option` 前提の安全 API に統一する。
- `_raw` / `_safe` 接尾辞付き関数を削除する。
- bare 名を標準化する。

### Phase 6: テスト回帰の整備

- `tests/move_effect.n.md`: effect 検査回帰
- `tests/memory_safety.n.md`: OOB / UAF / double free の検出
- `compile_fail` テストに `diag_code` 固定検証を追加

---

## 3. 目標状態（NEPLg3）

| 項目 | NEPLg2.0 | NEPLg3 |
|------|----------|----------|
| `alloc/dealloc` | 公開 API | compiler/runtime 境界に隔離 |
| raw pointer | safe user code から見える | 完全に隠蔽 |
| `List .T` の解放 | `free` 手動呼び出し | Region Inference で自動 |
| `File`・`Socket` の解放 | 手動 `close` 呼び出しが必要 | Drop Elaboration で自動 drop |
| effect 判定 | `alloc` 系を `Pure` で固定 | `InternalAlloc` → `Pure` への fold |
| entry の effect | 強制 Impure | 署名どおり |

---

## 詳細仕様

| 変更カテゴリ | 詳細仕様 |
|---|---|
| メモリ管理（値の三分類・Region Inference・Drop Elaboration） | [neplg3/spec/memory.md](../neplg3/spec/memory.md) |
| 副作用システム（InternalAlloc・ExternalIO 分類） | [neplg3/spec/effects.md](../neplg3/spec/effects.md) |
| コンパイラ実装（Resource IR・M1–M6 マイルストーン） | [neplg3/spec/compiler.md](../neplg3/spec/compiler.md) |
| stdlib 設計（安全 API への統一） | [neplg3/spec/stdlib.md](../neplg3/spec/stdlib.md) |
