# NEPLg2.1 副作用・Move・Borrow 仕様

最終更新: 2026-03-16

---

## 1. 仕様の三軸

この仕様は次の 3 軸を分離して扱う。

| 軸 | 記法 | 本質 |
|---|---|---|
| **Typing** | `Γ ⊢ e : τ` | 通常の型付け |
| **Effect** | `Γ ⊢ e ! ε` | 外界観測可能な副作用 |
| **Resource Usage** | `Γ ⊢ e ▷ σ` | move / borrow / drop / region に関わる資源使用 |

`Result` を返すこと自体は impure を意味しない。move は effect と独立に判定する。

---

## 2. Pure / Impure 効果

### 2.1 判定基準

Pure / Impure は「外部環境に対する観測可能な副作用」で判定する。

型表現での区別:

- `%fn ... -> ...`: **Pure**（外部観測可能な副作用なし）
- `%fn* ... -> ...`: **Impure**（I/O・ファイルシステム・乱数等を含む）

```nepl
let calc %fn i32 i32 -> i32 \ a b :    // Pure
    ...

let print_line %fn* str -> unit \ s :  // Impure
    ...
```

### 2.2 Pure とみなせる操作

- 算術・比較・分岐・束縛・データ構築
- heap / 線形メモリ操作（`alloc/realloc/dealloc/load/store`）— compiler 内部では `InternalAlloc` 効果として区別するが、raw address が外部に漏れない限り surface では `Pure` に畳み込む

### 2.3 Impure とみなす操作

- 標準入力 / 標準出力
- ファイルシステム
- 環境変数・argv・時刻・乱数
- syscall / extern によるホスト依存 I/O

### 2.4 compiler 内部効果分類

| 内部効果 | surface への畳み込み |
|----------|---------------------|
| `Pure` | → `Pure` |
| `InternalAlloc` | → `Pure` |
| `ExternalIO` | → `Impure` |
| `Nondet` | → `Impure` |
| `Unsafe` | → `Impure` |

### 2.5 heap / 線形メモリを Pure にできる条件

1. メモリ状態はコンパイラ内部で線形資源として管理される。
2. 生ポインタ整数は公開 API に露出しない。
3. アドレス値の比較・算術など、実装依存の観測を禁止する。
4. 不正操作は未定義動作にせず `Result/Option` で返す。

### 2.6 entry 関数の効果

entry 関数も署名どおりに effect を判定する。entry だからといって強制 Impure に昇格しない。

---

## 3. Move / Borrow / Copy / Clone

### 3.1 move の原則

- 値渡し引数はデフォルトで move。
- `Copy` 型は move でなく複製として扱う。
- 非 Copy 型は move 後に再利用不可。

### 3.2 borrow の原則

- borrow は所有権を移さない一時参照として扱う。
- borrow 中の可変性制約はコンパイラが検査する。
- 解放済み領域への borrow は禁止する。

### 3.2.1 borrow のスコープ終端規則

borrow のライフタイムは**最後の使用点（last use）** で終了する（NLL: Non-Lexical Lifetimes）。ブロック終端を待たない。

| 状況 | borrow の終端 |
|------|--------------|
| 変数への borrow | その変数の最後の読み取り式の直後 |
| 関数引数への borrow | 呼び出し式が評価された直後 |
| 条件分岐 | 全アームで borrow が終了した点の最大値（保守的マージ） |
| ループ | borrow がループ先頭まで到達する可能性がある場合は全ループ期間に拡大 |

```nepl
let x 42
let b &x            // shared borrow 開始
let y deref b       // b の最後の使用
// ここで b のスコープが終了 → x は再び自由
let mut x 99        // OK: borrow は終了済み
```

borrow が終了する前に元の値を mutable access した場合は診断 5007/5008 を発行する。

### 3.3 Copy / Clone の原則

- `Copy`: 暗黙複製可能な値型のみ。リソース型は非 Copy。
- `Clone`: 明示的複製。コストや共有有無は型ごとに定義する。

### 3.4 変数状態の追跡

move / borrow check は各変数について以下の状態を追跡する:

- `Live`: 初期化済み、使用可能
- `Moved`: move 済み、再使用不可
- `MaybeMoved`: 条件分岐により不定
- `Uninitialized`: 未初期化
- `BorrowedShared`: 共有 borrow 中（mutable access 禁止）
- `BorrowedUnique`: 一意 borrow 中（他の全 access 禁止）

分岐合流とループで状態を保守的にマージする。

関連診断:

| 診断 ID | 内容 |
|---------|------|
| 5001 | use-after-move |
| 5007 | borrow conflict: mutate while borrowed |
| 5008 | borrow conflict: access while uniquely borrowed |
| 5005 | linear value not consumed |
| 5006 | linear value may not be consumed on all paths |

### 3.5 `set` の純粋性条件

`set`（可変更新）が Pure とみなされる条件:

1. 更新対象が unique local state である
2. その状態への参照が外へ escape しない
3. 共有 borrow が存在しない
4. 更新の結果が観測可能な raw identity を漏らさない

---

## 4. Ownership と Linearity の違い

- **Ownership**: 「誰がその資源の解放責任を持つか」「他に危険な別名が存在しないか」を扱う。
- **Linearity**: 「その値を何回使ってよいか」を扱う。

NEPLg2 での使い分け:

- 永続値の sharing → pure value semantics
- builder や mutable buffer の一意性 → ownership / uniqueness
- token や capability の「必ず 1 回使い切り」 → linearity

---

## 5. 値の resource usage 分類

| 型 | resource usage |
|----|----------------|
| `i32`, `u8`, `bool`, `f32`, `unit` | `Unrestricted` |
| `str`, `List .T`, immutable struct (`Pair`, `Triple` 等) | `Unrestricted`（region-managed） |
| `OwnedBuf .T`, `VecBuilder .T`, `File`, `Socket` | `Owned` |
| `RegionToken`, `BuilderToken`, `CloseToken` | `Linear` |

### 5.1 能力テーブル

| 能力 | 意味 |
|------|------|
| `Copy` | read が move でなく copy になる |
| `Clone` | 明示複製が許される |
| `Drop` | scope exit / overwrite / early return で drop 候補になる |
| `Linear` | implicit copy も implicit discard も不可 |
| `Owned` | implicit discard は drop elaboration でのみ可能 |

### 5.2 複合型の resource usage 合成則

複合型（struct・enum）の resource usage は、その構成要素の中で最も厳しい能力に引きずられる。

1. 要素に `Linear` 型が 1 つでも含まれる場合、複合型全体が `Linear`。
2. 要素に `Owned` 型が含まれ、かつ `Linear` が含まれない場合、複合型全体が `Owned`。
3. 全ての要素が `Unrestricted` である場合のみ、複合型全体が `Unrestricted`。

```
Pair i32 str       → Unrestricted
Pair File i32      → Linear  （File が Linear）
Pair OwnedBuf u8 str  → Owned
```

制約の伝播順序: `Linear` > `Owned` > `Unrestricted`

---

## 6. trait の位置づけ

- `Copy` trait: 暗黙複製可能な型のみ実装可。リソース所有型（`RegionToken`, `File` 等）には実装禁止。
- `Clone` trait: 明示複製のみ許可。共有複製か独立複製かを型ごとに定義する。
- メモリ系 trait（将来導入）: `MemReadable .T`, `MemWritable .T`, `RegionOwned` — `load/store` や `dealloc` の呼び出し可能条件を型制約として表現する。

trait 実装可否は move check と整合して検査する。

---

## 7. #wasm / #llvmir と effect

- 生ターゲットブロックも effect 検査対象に含める。
- メモリアクセス命令は pure 文脈で許可可能。
- I/O 系命令を含む場合は impure 文脈を要求する。
- 判定は命令種別テーブルで一元管理する。

---

## 8. オーバーロードと effect

同名オーバーロードは同一 effect を要求する。pure / impure を同名だけで分岐させる API 設計は採用しない。effect が異なる場合はモジュールを分けるか、別名関数を使う。
