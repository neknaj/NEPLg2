# NEPLg2.1 メモリ管理仕様

最終更新: 2026-03-27

---

## 1. 設計目標

1. GC を使わずにメモリ安全を確保する。
2. 内部メモリ操作を surface では Pure に畳み込めるようにする。
3. 安全な API を `Result` / `Option` 前提で公開する。
4. Wasm と LLVM で安全意味論を共有する。

---

## 2. 値の三分類

### 2.1 Pure persistent value

例: `str`, `List .T`, immutable struct

### 2.2 Unique mutable work state

例: `ByteBuf`, `OwnedBuf .T`, `VecBuilder .T`, `StringBuilder`

### 2.3 Linear capability / external resource

例: `File`, `Socket`, `RegionToken`

---

## 3. 自動メモリ管理

### 3.1 Region Inference

Pure persistent value を region 単位で解放する。

### 3.2 Drop Elaboration

Owned / Linear resource に対して scope exit などで自動 drop を挿入する。

---

## 4. 文字列とバイト列

| 型 | 意味 |
|---|---|
| `str` | UTF-8 保証された immutable text |
| `ByteBuf` | 任意バイト列 |
| `StringBuilder` | 文字列構築用の一意可変状態 |

### 4.1 `StringBuilder` API 例

表層構文は Zenn #2 の let / lambda / `()` 表記に合わせる。

```nepl
let new %fn () StringBuilder \() ...
let push_str %fn StringBuilder fn str StringBuilder \builder \s ...
let finish %fn StringBuilder str \builder ...
```

### 4.2 変換

```nepl
let to_str %fn ByteBuf Result str Utf8Error \buf ...
let to_bytes %fn str ByteBuf \s ...
```

---

## 5. IO 資源

IO は Impure であり、所有権付き handle を消費しながら進める設計を維持する。

```nepl
let Mode enum:
    Read
    Write
    Append

let open %fn* Path fn Mode Result File IoError \path \mode ...
let read %fn* File Result Pair File str IoError \file ...
let write %fn* File fn str Result File IoError \file \text ...
let flush %fn* File Result File IoError \file ...
let close %fn* File Result () IoError \file ...
```

`close` の成功値は `()` とする。

失敗時に handle を返す別 API が必要なら、戻り値側に `Pair File IoError` を含む別シグネチャを定義する。

```nepl
let try_read %fn* File Result Pair File str Pair File IoError \file ...
```

---

## 6. Phase 8 との接続

依存型導入後も、型に現れる値は不変でなければならない。このため、Pure persistent value と region 管理の設計はそのまま Phase 8 の基盤になる。

```nepl
let get .T .len .idx %fn Vec .T .len fn .idx .T
    ...
```

具体的な依存型シグネチャは [phase8.md](./phase8.md) に従う。
