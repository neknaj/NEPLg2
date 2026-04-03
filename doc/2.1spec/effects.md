# NEPLg2.1 副作用・Move・Borrow 仕様

最終更新: 2026-03-27

---

## 1. 仕様の三軸

この仕様は次の 3 軸を分離して扱う。

| 軸 | 記法 | 本質 |
|---|---|---|
| **Typing** | `Γ ⊢ e : τ` | 通常の型付け |
| **Effect** | `Γ ⊢ e ! ε` | 外界観測可能な副作用 |
| **Resource Usage** | `Γ ⊢ e ▷ σ` | move / borrow / drop / region に関わる資源使用 |

---

## 2. Pure / Impure

### 2.1 表記

Zenn #2 で確定した前置型記法に合わせ、関数型も次のように書く。

- `fn A B`: Pure な 1 引数関数
- `fn* A B`: Impure な 1 引数関数

多引数関数はカリー化して表す。

```nepl
let calc %fn i32 fn i32 i32 \a \b ...
let print_line %fn* str () \s ...
```

### 2.2 Pure とみなす操作

- 算術
- 比較
- 分岐
- データ構築
- 外界から観測できない内部メモリ操作

### 2.3 Impure とみなす操作

- 標準入出力
- ファイルシステム
- 環境変数
- 時刻
- 乱数

---

## 3. Move / Borrow / Copy / Clone

### 3.1 move の原則

- 値渡しはデフォルトで move
- `Copy` 型は暗黙複製
- 非 `Copy` 型は move 後に再利用不可

### 3.2 borrow の原則

- borrow は所有権を移さない
- borrow 中の可変性制約はコンパイラが検査する
- last use に基づいて borrow 範囲を終了させる

---

## 4. `let` / draft `set` と効果

Zenn #2 に従い、束縛の表層構文は `let <name> <expr>` / `let mut <name> <expr>` である。

`let` 式の型は `()` とする。

`set` は意味論上ここで扱うが、表層構文はまだ draft であり、`syntax.md` の保留事項にも含める。可変更新の purity 判定は次の条件に従う。

1. 更新対象が unique local state である
2. 参照が外へ escape しない
3. 共有 borrow が存在しない
4. raw identity が露出しない

---

## 5. 資源使用分類

| 型 | resource usage |
|---|---|
| `i32`, `u8`, `bool`, `f32`, `()` | `Unrestricted` |
| `str`, `List .T`, immutable struct | `Unrestricted` |
| `Slice .T` | `Unrestricted` |
| `ByteBuf`, `OwnedBuf .T`, `StringBuilder`, `VecBuilder .T` | `Owned` |
| `File`, `Socket`, `RegionToken`, `BuilderToken`, `CloseToken` | `Linear` |

---

## 6. trait との接続

- `Copy` は暗黙複製可能であることを示す
- `Clone` は明示複製
- `Drop` は scope exit / overwrite で drop 候補になる

`Linear` 型と `Copy` は両立しない。

---

## 7. 旧表記からの更新方針

この章では effect モデル自体は維持するが、表層構文は Zenn #1 / #2 の前置型記法・`()` 表記・`let` / lambda 構文へ合わせる。
