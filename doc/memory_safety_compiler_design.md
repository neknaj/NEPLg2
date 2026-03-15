# メモリ安全コンパイラ設計

最終更新: 2026-03-15

> 統合仕様は `doc/purity_ownership_memory_spec.md` を参照。本文書はコンパイラ実装寄りの設計メモとして維持する。

## 1. 目的

- GC なしで、コンパイラ管理のみでメモリ安全性を確保する。
- heap/線形メモリ操作を pure として扱うための実装条件を定義する（内部効果 `InternalAlloc` として扱い、surface では `Pure` に畳み込む）。
- `mem` を `Result/Option` 前提の安全APIに統一する（`kpread`/`kpwrite` は削除済み、`std/streamio` へ統合）。

## 2. 公開モデル

### 2.1 公開型

> `MemPtr<T>` と `RegionToken` は safe user code からは直接見えない compiler/runtime 境界型とする。safe 側に公開するのは `OwnedBuf<T>`, `Slice<T>`, `List<T>`, `Vec<T>`, `str`, `ByteBuf` などの抽象型のみ。

- `MemPtr<T>`
  - 型付きメモリ参照。compiler/runtime 境界に閉じ込める。
  - 生アドレス整数は隠蔽する。
- `RegionToken`
  - 領域の所有権を表す線形トークン。
  - `dealloc` で消費される。

### 2.2 不変条件

- `MemPtr<T>` は有効な `RegionToken` と対応している。
- 解放済み `RegionToken` からのアクセスは不可能。
- `offset + sizeof(U) <= size` を満たす場合のみ `load/store<U>` を許可。
- 二重解放・解放後アクセスは検出して拒否する。

## 3. effect との整合

- メモリ操作（`alloc/realloc/dealloc/load/store`）は compiler 内部では `InternalAlloc` 効果を持つ。
- `InternalAlloc` は surface では `Pure` に畳み込まれる。ただし、raw address が外部に漏れないことが前提。
- I/O 操作（stdin/stdout/fs/env/time/random/syscall）は `ExternalIO` であり、surface では `Impure`。
- したがって、I/O を含まず raw address を漏らさないメモリ処理関数は `->` を保てる。

この整理は `doc/purity_ownership_memory_spec.md` の統合仕様を前提とする。

## 4. コンパイラで行う検査

### 4.1 型検査

- `load/store` などを `MemPtr<T>` 受け取りに統一する。
- 生 `i32` ポインタ受け取りを公開APIから禁止する。
- fallible 操作を `Result/Option` で型に反映する。

### 4.2 move/borrow 検査

- `RegionToken` は非Copy。
- `dealloc(token)` 後の token 再利用を禁止する。
- `MemPtr<T>` の借用中は可変性制約を適用する。
- 分岐/ループ合流で `PossiblyMoved` を保守的に維持する。

### 4.3 境界/生存検査

- `load/store` の境界検査を挿入する。
- 解放後アクセスを `Result::Err` 経路へ分岐させる。
- 定数証明可能な安全アクセスは最適化で検査削除可能。

### 4.4 trait 制約検査

- `Copy` 実装可否を構造的に検査し、リソース所有型の `Copy` 実装を禁止する。
- `Clone` 実装は move 規則と矛盾しない複製規約を満たすことを要求する。
- メモリ系 trait（`MemReadable<T>`, `MemWritable<T>`, `RegionOwned`）の境界を満たさない呼び出しは型エラーにする。

## 5. API 設計指針

### 5.1 core/mem

- `_raw` 公開関数は段階的に削除し最終的に廃止。
- `_safe` 接尾辞は廃止し、安全版を標準名へ統一。
- 失敗を `Result<_, Diag>` または `Option<_>` で返す。

### 5.2 std/streamio

- `Scanner` / `Writer` に所有権と領域情報を保持させる。
- ハンドル `i32` を外部APIへ露出しない。
- I/O 実行部のみ Impure として扱う。

### 5.3 trait ベース API

- `core/mem` の読み書きAPIは trait 境界で能力を表現する。
- stream I/O は `RegionOwned` を満たす型のみが解放操作を実行できるようにする。

## 6. 診断

少なくとも以下の診断カテゴリを持つ。

- メモリ型不一致
- 範囲外アクセス
- 解放後アクセス
- 二重解放
- moved 値使用
- pure 文脈での impure 呼び出し

compile_fail テストでは diag_id で固定検証する。

## 7. 二段構えの自動メモリ管理

### 7.1 Region Inference (領域推論)

- pure persistent value (`List<T>`, `str`, immutable tree など) は region 単位で bulk free する。
- source に region 構文は見せず、compiler が alloc/dealloc を自動挿入する。

### 7.2 Drop Elaboration (drop 展開)

- owned / linear resource (`File`, `Socket`, `OwnedBuf<T>`, `StringBuilder` など) は scope exit / overwrite 時に自動 drop。
- 初期化状態を dataflow で追い、条件付き drop を生成する。

## 8. 段階導入

1. `MemPtr<T>` / `RegionToken` を compiler/runtime 境界に閉じ込める。
2. builtins/effect 判定を `InternalAlloc` / `ExternalIO` 分類へ移行。
3. move check を token 消費対応へ拡張。
4. Resource IR を導入し、ownership / borrow / region / drop の解析パスを整備。
5. stdlib (`mem` / `std/streamio`) を安全APIへ統一。
6. tests に memory/effect 回帰を追加。

## 9. 非目標

- GC 導入は行わない。
- 未定義動作で隠す設計は採用しない。
- 旧ポインタAPIとの後方互換は維持しない。
