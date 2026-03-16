# NEPLg2.1 標準ライブラリ設計

最終更新: 2026-03-16

---

## 1. 設計原則

### 1.1 値中心・式指向

- `if` / `match` / block が値を返す言語仕様に合わせ、ライブラリ API も「値を返す関数」を標準形とする。
- パイプ `|>` で自然に読めるよう、変換対象は原則第 1 引数に置く。

### 1.2 安全 API 優先

- 公開 API の戻り値は `Option` / `Result` / `Outcome` を用途に応じて使い分ける。
- `_raw` / `_safe` のような実装都合の命名は公開面から排除する。
- 低レベル API は内部実装または隔離層に閉じ込め、利用者に直接露出しない。

### 1.3 bare 名の統一

- API 名には prefix/suffix を付けない（`get_opt`・`get_safe`・`read_i32` のような命名は採用しない）。
- `read` / `write` / `flush` / `close` / `get` / `push` / `pop` のような bare 名を正とする。
- 型ごとの差異は引数型・戻り値型・trait 制約・オーバーロード解決で表し、命名規則で補わない。

### 1.4 能力は trait で表す

- `Copy` / `Clone` / `Eq` / `Ord` / `Hash` / `Stringify` などの能力は trait で表す。
- どの型がどの能力を持つかは `.nepl` ソース上の宣言を唯一の根拠とし、compiler 内部に型ごとの固定表を持たない。

### 1.5 層構造と依存方向

- `core` → `alloc` → `runtimes` → `std` → `features` の順に、動作条件は増え、互換性は狭くなる。
- より下位互換な層へ置ける機能を、より上位依存な層へ不用意に押し上げない。

---

## 2. 層の分類と責務

### 2.1 `core`

- 純粋で target 非依存な基本機能だけを置く。
- heap を必要としないものを置く。
- OS / device / syscall / allocator 詳細に依存しない。
- 基本型・演算・基本 trait・`Result` / `Option` などの最小集合を担当する。
- `math.nepl` のように、ほぼすべての runtime で共通に提供されるべき計算ライブラリはここに置く。

### 2.2 `alloc`

- `core` の上で、heap 依存だが target 非依存の汎用機能を提供する。
- allocator / 領域管理 / 診断補助 / encoding / hash をここへ置く。
- `MemPtr .T` / `RegionToken .T` は compiler/runtime 境界モジュールとして `alloc` 配下に置くが、safe user code からは抽象型（`OwnedBuf .T`, `Slice .T` など）だけが見える。

### 2.3 `runtimes`

- target ごとの syscall / ABI / descriptor 差分を吸収する adapter 層。
- 利用者は原則としてこれらを直接使わず、`std` や `std/streamio` を通して利用する。
- wasm / wasi / llvm の差分もここに含めて扱う。

### 2.4 `std`

- 各個別 target を束ね、device / OS 依存 API を標準的に扱えるようにする facade 層。
- `runtimes` / `std/streamio` を適切に包み、利用者向けに安定した標準 API 面を提供する。
- `std` は stdlib 全体そのものではなく、stdlib の中で target 依存 API を標準化して見せる役割だけを担う。

### 2.5 `features`

- 外部 API / FFI / デバイス接続などの公式追加機能群。
- GUI / HTTP / TUI / 音声再生など、外界と接続する機能を置く。
- 一方で、regex や audio buffer など純計算・データ処理は `core` または `alloc` に置く。

**`alloc` と `features` の境界判断基準:**

| 判断基準 | `alloc` | `features` |
|---------|---------|-----------|
| ホスト syscall / FFI への依存 | なし | あり |
| target 非依存か | はい | 一般的にいいえ |
| Pure / Total 関数のみで構成できるか | 原則可 | Impure を含む |
| 典型例 | JSON パース・正規表現・Unicode 正規化・暗号ハッシュ | GUI・HTTP・TUI・音声・タイマー・プロセス管理 |

判断に迷う場合の原則: **「入出力なしに動作するか」が `alloc`、「外界と通信しなければ動作しないか」が `features`**。

### 2.6 `kp`（暫定・競技プログラミング層）

- 競技プログラミング向けの先行ライブラリを置く暫定層。
- 成熟し責務が一般化できた機能は `std` / `alloc/collections` / `alloc/text` 等へ昇格させる。

### 2.7 `nm`・`neplg2`（独立ライブラリ層）

- `nm`: 拡張 markdown・doc comment・HTML 変換など周辺ツールチェーン用。
- `neplg2`: セルフホスト compiler 用ライブラリ。

---

## 3. パッケージ構成

```text
stdlib/
    core/                # 純粋計算・基本型・基本 trait
        rand/

    alloc/               # heap 依存だが target 非依存の汎用ライブラリ
        collections/     # Vec/Map/Set/Queue/Stack などのデータ構造
        text/            # str/String/Unicode/format/parse
        io/              # Reader/Writer/Seekable/Buffered などの低水準抽象
        diag/            # 診断値・診断構築補助
        encoding/        # json などの符号化/復号
        hash/            # HashMap/HashSet 向けのハッシュ実装

    std/                 # 各個別 target を束ねる標準 API 面
        streamio/        # fs/stdio などを束ねる高水準 stream 抽象
        stdio/
        fs/
        env/

    features/            # 外部 API / FFI / デバイス接続などの公式追加機能
        gui/
        http/
        tui/
        audio/

    kp/                  # 競技プログラミング向け暫定ライブラリ
    nm/                  # 拡張 markdown・doc comment・HTML 変換
    neplg2/              # セルフホスト compiler 用ライブラリ
        core/
        cli/

    runtimes/            # target 別 adapter 実装
        wasi/
        wasm/
        nasm/
        c/

    tests/               # stdlib 専用 fixture / test support

    prelude/
        core.nepl
        std.nepl
```

---

## 4. 各 alloc サブパッケージの責務

### 4.1 `alloc/collections`

- `Vec` / `Map` / `Set` / `Queue` / `Stack` など一般用途のデータ構造。
- device や OS に依存しないため `std` には入れない。
- allocator と trait 能力に基づき、move / effect / memory 規則と整合する API を提供する。

### 4.2 `alloc/text`

- `str` / `String` / Unicode / format / parse / 文字列表現変換を担当する。
- `bool/i32/i64/i128/...` と文字列の相互変換はここに置く。
- 標準入出力そのものは `text` の責務ではなく、`std/stdio` や `std/streamio` が利用する側。

### 4.3 `alloc/io`

- `Reader` / `Writer` / `Seekable` / `Buffered` など I/O の低水準抽象。
- syscall や descriptor などの具体実装は持たない。
- 「何が読めるか / 書けるか」の能力を trait として表現する。

### 4.4 `std/streamio`

- `alloc/io` の抽象を束ね、`read` / `write` / `flush` / `close` の高水準 API を提供する。
- `stdio`・`fs`・将来の socket / timer / process event / UI event などを、同じ stream 能力モデルで扱えるようにする。

---

## 5. エラー・診断モデル

詳細は [errors.md](./errors.md) を参照。

| 型 | 用途 |
|---|---|
| `Option .T` | 値がある/ない（診断なし） |
| `Result .T .E` | 成功/失敗（軽量、診断省略）。既定 `.E` は `StdErrorKind` |
| `Outcome .T .E` | 成功/失敗 + 診断群（`Diag`）の組み合わせ |
| `Diag` | 構造化診断値（kind / message / span / notes / help / source） |

### 5.1 使い分け基準

- `Option`: 値が存在しないことが想定内で、診断や失敗理由を伴わない分岐。
- `Result`: 単純な成功/失敗制御で十分。診断は不要か呼び出し側が不要と判断できる。
- `Outcome`: warning/info/log を失わずに返したい API。compiler・parser・json 変換など。

---

## 6. 標準 trait 一覧

| Trait | 意味 |
|-------|------|
| `Eq` | 等値比較 |
| `Ord` | 全順序比較 |
| `Hash` | ハッシュ計算 |
| `Stringify` | 人間可読文字列変換 |
| `Debug` | デバッグ表現（詳細・安定性要件が `Stringify` と異なる） |
| `Serialize .F` | 指定フォーマット `.F` への直列化 |
| `Deserialize .F` | 指定フォーマット `.F` からの逆直列化 |
| `Parse` | 人間が書いた入力のパース |
| `Default` | デフォルト値生成 |
| `Clone` | 明示的複製 |
| `Copy` | 暗黙複製（`Clone` の特殊ケース） |
| `Add .U .R` | 加算演算 |
| `Drop` | scope exit での自動 drop |
| `RegionOwned` | region 管理下の所有者 |
| `MemReadable .T` | メモリから `.T` を読む能力（将来導入） |
| `MemWritable .T` | メモリへ `.T` を書く能力（将来導入） |
| `Allocator` | 領域確保・解放ポリシー |
| `Reader` | バイト列の読み出し |
| `Writer` | バイト列の書き込み |
| `Seekable` | シーク操作 |
| `Buffered` | バッファリング |
| `EventSource .E` | イベント発行源 |
| `EventSink .E` | イベント消費先 |

---

## 7. ドキュメントコメントポリシー

- stdlib reboot では API と実装だけでなく、ドキュメントコメントも再構築対象とする。
- 関数・struct・enum・trait・ファイル冒頭コメントを含め、実装の責務・制約・使い方がコードの実体に対応して説明されていなければならない。
- ライブラリ `.nepl` 内の doctest は、利用者に対する使い方説明と、そのサンプルコードが正しいことの保証を目的とする。
- 実装の正しさ・回帰・エッジケースの検証は `tests/` に置き、ドキュメントコメント内の doctest と責務を分離する。
- テンプレート流用・ボイラープレート・機械生成的なコメントは採用しない。
