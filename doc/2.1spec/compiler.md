# NEPLg2.1 コンパイラ実装設計

最終更新: 2026-03-16

---

## 1. 目的

- GC なしで、コンパイラ管理のみでメモリ安全性を確保する。
- heap / 線形メモリ操作を Pure として扱うための実装条件を定義する（内部効果 `InternalAlloc` として扱い、surface では `Pure` に畳み込む）。
- safe user code からは raw pointer を隠蔽し、安全 API のみを公開する。

---

## 2. 公開型と内部型

### 2.1 safe user code からは見せない型（compiler / runtime 境界型）

- `MemPtr .T`: 型付きメモリ参照。compiler / runtime 境界に閉じ込める。生アドレス整数は隠蔽する。
- `RegionToken`: 領域の所有権を表す線形トークン。`dealloc` で消費される。

### 2.2 safe 側に公開する抽象型

`str`, `ByteBuf`, `Slice .T`, `OwnedBuf .T`, `List .T`, `Vec .T`, `File`, `Socket`

### 2.3 不変条件

- `MemPtr .T` は有効な `RegionToken` と対応していること。
- 解放済み `RegionToken` からのアクセスは不可能。
- `offset + sizeof(U) <= size` を満たす場合のみ `load/store<U>` を許可。
- 二重解放・解放後アクセスは検出して拒否する。

---

## 3. 内部効果分類

メモリ操作（`alloc/realloc/dealloc/load/store`）は compiler 内部では `InternalAlloc` 効果を持つ。`InternalAlloc` は surface では `Pure` に畳み込まれる（raw address が外部に漏れないことが前提）。

| 内部効果 | surface への畳み込み |
|----------|---------------------|
| `Pure` | → `Pure` |
| `InternalAlloc` | → `Pure` |
| `ExternalIO` | → `Impure` |
| `Nondet` | → `Impure` |
| `Unsafe` | → `Impure` |

---

## 4. コンパイラが行う検査

### 4.1 型検査

- `load/store` などを `MemPtr .T` 受け取りに統一する。
- 生 `i32` ポインタ受け取りを公開 API から禁止する。
- fallible 操作を `Result/Option` で型に反映する。

### 4.2 move / borrow 検査

- `RegionToken` は非 Copy。
- `dealloc(token)` 後の token 再利用を禁止する。
- `MemPtr .T` の借用中は可変性制約を適用する。
- 分岐 / ループ合流で `MaybeMoved` を保守的に維持する。

### 4.3 境界 / 生存検査

- `load/store` の境界検査を挿入する。
- 解放後アクセスを `Result::Err` 経路へ分岐させる。
- 定数証明可能な安全アクセスは最適化で検査削除可能。

### 4.4 trait 制約検査

- `Copy` 実装可否を構造的に検査し、リソース所有型の `Copy` 実装を禁止する。
- `Clone` 実装は move 規則と矛盾しない複製規約を満たすことを要求する。
- メモリ系 trait（`MemReadable .T`, `MemWritable .T`, `RegionOwned`）の境界を満たさない呼び出しは型エラーにする。

---

## 5. Resource IR

typed HIR の後ろに **Resource IR**（資源 IR）を置く。CFG を持ち、以下を明示する:

```
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

この IR 上で次を診断する:

| 診断 | 内容 |
|------|------|
| 5001 | use-after-move |
| 5002 | raw address escape from safe boundary |
| 5003 | double free |
| 5004 | use after free |
| 5005 | linear value not consumed |
| 5006 | linear value may not be consumed on all paths |
| 5007 | borrow conflict: mutate while borrowed |
| 5008 | borrow conflict: access while uniquely borrowed |

---

## 6. 解析パス順

1. surface typecheck
2. effect attribution
3. Resource IR 生成
4. ownership / borrow check
5. region inference
6. drop elaboration
7. target lowering

---

## 7. Target Lowering

| target | 方式 |
|--------|------|
| Wasm | linear memory ベース |
| LLVM | native pointer / native allocator ベース |

共通化するのは「安全意味論」であって「レイアウト」ではない。

---

## 8. 段階導入計画

| Phase | 内容 |
|-------|------|
| 1 | `MemPtr .T` / `RegionToken` を compiler / runtime 境界に閉じ込める |
| 2 | builtins / effect 判定を `InternalAlloc` / `ExternalIO` 分類へ移行 |
| 3 | move check を token 消費対応へ拡張 |
| 4 | Resource IR を導入し、ownership / borrow / region / drop の解析パスを整備 |
| 5 | stdlib（`mem` / `std/streamio`）を安全 API へ統一 |
| 6 | tests に memory / effect 回帰を追加 |

---

## 9. API 設計指針

### 9.1 core/mem

- `_raw` 公開関数は廃止。
- `_safe` 接尾辞は廃止し、安全版を標準名へ統一。
- 失敗を `Result _ Diag` または `Option _` で返す。

### 9.2 std/streamio

- `Scanner` / `Writer` に所有権と領域情報を保持させる。
- ハンドル `i32` を外部 API へ露出しない。
- I/O 実行部のみ Impure として扱う。

### 9.3 trait ベース API

- `core/mem` の読み書き API は trait 境界で能力を表現する。
- stream I/O は `RegionOwned` を満たす型のみが解放操作を実行できるようにする。

---

## 10. テスト要件

- `tests/move_effect.n.md`: pure から I/O 呼び出しが拒否されること / pure からメモリ操作が許可されること
- `tests/memory_safety.n.md`: OOB / UAF / double free の検出
- `tests/overload.n.md`: type annotation と overload が move / effect と両立すること
- `compile_fail` テストでは `diag_id` で固定検証する
