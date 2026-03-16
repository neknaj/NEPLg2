# move規則・pure/impure規則 統合仕様

最終更新: 2026-03-15

> 上位の統合仕様は `doc/purity_ownership_memory_spec.md` を参照。本文書は move/effect に焦点を当てた詳細規則として維持する。

## 0. 仕様の前提

この仕様は、次の3軸を分離して扱う。

- `Option/Result`: 欠損・失敗の表現
- `Pure/Impure`: 外部観測可能な副作用の表現
- `Move/Borrow/Copy/Clone`: 所有権と再利用可能性の表現

`Result` を返すこと自体は impure を意味しない。
move は effect と独立に判定する。

## 1. 目的

- GC なしで、コンパイラ管理のみでメモリ安全性を担保する。
- heap/線形メモリ操作を pure として扱える論理モデルを確定する。
- impure を I/O 系操作に限定し、effect 判定を明確化する。
- stdlib を `Result/Option` 前提の安全APIへ統一する。

## 2. Pure/Impure の意味

### 2.1 判定基準

Pure/Impure は「外部環境に対する観測可能な副作用」で判定する。

型表現では `%fn ... -> ...`（Pure）と `%fn* ... -> ...`（Impure）で区別する。
宣言例：

```nepl
let calc %fn i32 i32 -> i32 (a, b):       // Pure
let print_line %fn* str -> unit (s):       // Impure
```

- Pure:
  - 算術、比較、分岐、束縛、データ構築
  - heap/線形メモリ操作（`alloc/realloc/dealloc/load/store`）— compiler 内部では `InternalAlloc` 効果として区別するが、raw address が外部に漏れない限り surface では `Pure` に畳み込む
- Impure:
  - 標準入力/標準出力
  - ファイルシステム
  - 環境変数、argv、時刻、乱数
  - syscall/extern によるホスト依存I/O

compiler 内部では以下の効果分類を持つ:

| 内部効果 | surface への畳み込み |
|----------|---------------------|
| `Pure` | → `Pure` |
| `InternalAlloc` | → `Pure` |
| `ExternalIO` | → `Impure` |
| `Nondet` | → `Impure` |
| `Unsafe` | → `Impure` |

### 2.2 heap/線形メモリを Pure にできる条件

heap/線形メモリ操作を Pure とするため、以下を必須条件とする。

1. メモリ状態はコンパイラ内部で線形資源として管理される。
2. 生ポインタ整数は公開APIに露出しない。
3. アドレス値の比較・算術など、実装依存の観測を禁止する。
4. 不正操作は未定義動作にせず `Result/Option` で返す。

この条件下では、メモリ操作は「隠蔽された内部状態遷移」であり、I/O とは分離できる。

### 2.3 entry 関数の扱い

- entry を強制的に Impure へ昇格する特例は廃止する。
- entry も署名どおりに effect を判定する。

## 3. Move/Borrow/Copy/Clone

### 3.1 move の原則

- 値渡し引数はデフォルトで move。
- `Copy` 型は move でなく複製として扱う。
- 非Copy型は move 後に再利用不可。

### 3.2 borrow の原則

- borrow は所有権を移さない一時参照として扱う。
- borrow 中の可変性制約はコンパイラが検査する。
- 解放済み領域への borrow は禁止する。

### 3.3 Copy/Clone の原則

- `Copy`: 暗黙複製可能な値型のみ。
- `Clone`: 明示的複製。コストや共有有無は型ごとに定義する。
- リソース型（メモリトークン、I/Oハンドル）は非Copy。

### 3.4 変数状態の追跡

move check は少なくとも以下を追跡する。

- `Valid`
- `Moved`
- `PossiblyMoved`
- `BorrowedShared`
- `BorrowedUnique`

分岐合流とループで状態を保守的にマージする。

### 3.6 `set` の新規則

現在の「局所なら pure」は廃止する。`set` が pure である条件:

- 更新対象が unique local state である
- その状態への参照が外へ escape しない
- 共有 borrow が存在しない
- 更新の結果が観測可能な raw identity を漏らさない

### 3.5 trait の位置づけ

`trait` は effect と move の補助情報を型に付与する契約として扱う。

- `Copy` trait:
  - 暗黙複製可能な型のみ実装可。
  - リソース所有型（`RegionToken`, `Scanner`, `Writer`）には実装禁止。
- `Clone` trait:
  - 明示複製のみ許可。
  - 共有複製か独立複製かを型ごとに定義する。
- メモリ系 trait（導入予定）:
  - 例: `MemReadable<T>`, `MemWritable<T>`, `RegionOwned`
  - `load/store` や `dealloc` の呼び出し可能条件を型制約として表現する。

trait 実装可否は move check と整合して検査する。

## 4. メモリ安全モデル

### 4.1 公開型

- `MemPtr<T>`: 型付きメモリ参照
- `RegionToken`: 領域所有権トークン

`i32` 生ポインタは公開APIで禁止する。

### 4.2 不変条件

- `MemPtr<T>` は有効な `RegionToken` と対応していること。
- `dealloc` は `RegionToken` を消費し、以後再利用不可。
- 境界外アクセス、解放後アクセス、二重解放はコンパイラ/ランタイム検査で拒否。

### 4.3 失敗の表現

- fallible API は `Result<_, Diag>` を標準とする。
- optional API は `Option<_>` を用いる。
- 旧 `_safe` 接尾辞は廃止し、安全版をデフォルト命名に統一する。

## 5. #wasm / #llvmir と effect

- 生ターゲットブロックも effect 検査対象に含める。
- メモリアクセス命令は pure 文脈で許可可能。
- I/O 系命令を含む場合は impure 文脈を要求する。
- 判定は命令種別テーブルで一元管理する。

## 6. NEPLg2 既存仕様との整合

### 6.1 前置記法・式指向との整合

- 本仕様は前置記法を変更しない。
- `Pure/Impure` 判定は関数型 `%fn ... -> ...`（Pure）/ `%fn* ... -> ...`（Impure）で表現し、既存の式指向規則と整合する。
- 型注釈は `%TypeExpr` 記法を使う（`<>` 囲みは廃止）。型引数の曖昧性は `%Pair i32 str` のように明示する。型注釈の仕様は `doc/type_notation_spec.md` を参照。

### 6.2 オーバーロードとの整合

- 同名オーバーロードは既存の解決規則（引数型・戻り型・型引数）に従う。
- 暗黙castは行わない。必要な場合は明示 `cast` と型注釈で解決する。
- 現行実装の「同名オーバーロードは同一 effect を要求する」制約を維持する。
  - そのため、pure/impure を同名だけで分岐させるAPI設計は採用しない。
  - effect が異なる場合は別名関数か明示的な呼び分けを使う。

## 7. コンパイラ実装要件

1. builtins の `alloc/realloc/dealloc` に `InternalAlloc` 内部効果を導入する（surface は条件付き `Pure`）。
2. `entry` 強制 Impure 特例を削除する。
3. intrinsic effect 判定を `InternalAlloc` / `ExternalIO` / `Nondet` / `Unsafe` の宣言テーブルへ一元化する。
4. move check に `RegionToken` 消費規則を導入する。
5. `TypeCtx::is_copy` を構造型（tuple/struct/enum）まで拡張する。
6. 診断IDを move/effect/memory safety 系へ割り当てる。
7. Resource IR を導入し、ownership / borrow / region / drop の解析パスを整備する。

## 8. テスト要件

- `tests/move_effect.n.md`:
  - pure から I/O 呼び出しが拒否されること
  - pure からメモリ操作が許可されること
- `tests/memory_safety.n.md`:
  - OOB / UAF / double free の検出
- `tests/overload.n.md`:
  - type annotation と overload が move/effect と両立すること
  - 同名オーバーロードの effect 一致制約が維持されること

## 9. 非目標

- GC 導入は行わない。
- 暗黙castによる overload 解決は行わない。
- 旧APIとの後方互換は維持しない。
