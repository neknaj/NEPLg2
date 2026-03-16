# stdlib 破壊的再設計案（後方互換なし）

最終更新: 2026-03-15

## 1. 目的

- stdlib 全体を「安全 API が標準、低レベル API は隔離」の方針で再構築する。
- `plan.md` の言語仕様、特に式指向・前置記法・パイプ合成・型注釈・effect 規則と矛盾しないライブラリ構成を確立する。
- `Copy` / `Clone` / move / effect / memory の規則を、compiler と stdlib の両方で同じ能力モデルに基づいて扱えるようにする。
- target 依存部分を adapter 層へ閉じ込め、利用者が触れる API 面では target 差分を最小化する。

## 2. 非目標

- 既存 stdlib API とのソース互換・バイナリ互換は提供しない。
- 段階的 deprecate は行わず、最終的には新構成へ一本化する。
- 暗黙変換・暗黙効果昇格・暗黙 import による利便性向上は行わない。
- compiler 内に「特定型は当然 Copy/Clone である」といった固定知識を持ち込まない。

## 3. 設計原則

### 3.1 値中心・式指向

- `if` / `match` / block が値を返す言語仕様に合わせ、ライブラリ API も「値を返す関数」を標準形とする。
- 更新系 API でも、可能な限り合成しやすい返り値設計を優先する。
- パイプ `|>` で自然に読めるよう、変換対象は原則第1引数に置く。

### 3.2 安全 API 優先

- 公開 API の戻り値は `Option` / `Result` / `Outcome` を用途に応じて使い分ける。
- 値が存在しないこと自体が想定内であり、診断や失敗理由を伴わない分岐には `Option` を使う。
- 単純な成功/失敗制御だけで十分な API は `Result` を使う。
- source code・span・補助診断を併せて返す必要があるリッチな API は `Outcome` を使う。
- `Result` と `Outcome` は対立する型ではなく、`Result` は診断を省略した簡易形、`Outcome` は診断付き標準形として設計する。
- `_raw` / `_safe` のような実装都合の命名は公開面から排除する。
- 低レベル API は内部実装または隔離層に閉じ込め、利用者に直接露出しない。

### 3.2.1 read/write 系命名の統一

- `io` / `streamio` と、その上に載る stdlib の入出力 API は、`read` / `write` / `writeln` / `flush` / `close` の bare 名を正とする。
- `scanner_read_*` / `writer_write_*` / `stream_write_*` のような prefix 付き別名は残さない。
- `read_i32` / `write_str` / `write_f64` のような型名 suffix 付き公開 API も残さない。
- 型ごとの差異は引数型・返り値型・trait 制約・オーバーロード解決で表し、命名規則で補わない。
- 後方互換 alias や移行期間用 wrapper は設けず、既存コードは新しい bare 名へ書き換える。

### 3.3 能力は trait で表す

- `Copy` / `Clone` / `Eq` / `Ord` / `Hash` / `Stringify` / `Debug` / `Parse` などの能力は trait で表す。
- どの型がどの能力を持つかは `.nepl` ソース上の宣言を唯一の根拠とし、compiler 内部に型ごとの固定表を持たない。
- trait 解決とオーバーロード解決は同じ型同値判定に基づいて処理する。

### 3.4 責務を層で分離する

- `core` / `alloc` は target 非依存の基盤層とする。
- `core` は heap 不要でほぼすべての target で共通に提供される最小基盤とする。
- `alloc` は heap 依存だが、heap さえあればどの target でも動く汎用ライブラリ層とする。
- `std` は各個別 target を束ね、device / OS 依存 API を標準的に扱えるようにする facade 層とする。
- `runtimes` は target ごとの差分と、それに付随する実装差分を吸収する adapter 層とする。
- `core` -> `alloc` -> `runtimes` -> `std` -> `features` の順に、動作条件は増え、互換性は狭くなるものとして設計する。
- したがって、より下位互換な層へ置ける機能を、より上位依存な層へ不用意に押し上げない。

### 3.5 `stdlib` と `std` は別物

- `stdlib` は、NEPLg2 の標準機能として配布・利用されるライブラリ群全体を指す。
- `std` はその一部であり、`stdlib/std` という入れ子構造をなす。
- `std` の責務は、各 target 固有の機能や `runtimes` が吸収する差分を束ね、利用者に共通の標準 API を提供することである。
- したがって、`core` / `alloc` / `nm` / `neplg2` / `kp` は stdlib に含まれていても `std` ではない。

### 3.6 ドキュメントコメントも再構築対象

- stdlib reboot では API と実装だけでなく、ドキュメントコメントも再構築対象とする。
- `doc/stdlib_doc_comment_policy.md` を標準方針書とし、stdlib 内のすべてのファイルはこの方針に従ってドキュメントコメントを整備する。
- 関数、struct、enum、trait、ファイル冒頭コメントを含め、実装の責務・制約・使い方がコードの実体に対応して説明されていなければならない。
- ライブラリ `.nepl` 内の doctest は、利用者に対する使い方説明と、そのサンプルコードが正しいことの保証を目的とする。
- 実装の正しさ、回帰、エッジケースの検証は `tests/` に置き、ドキュメントコメント内の doctest と責務を分離する。
- テンプレート流用、ボイラープレート、機械生成的なコメントは採用しない。

## 4. 層構造

### 4.1 層の分類

- 基盤層
  - `core`
  - `alloc`
- 実装差分吸収層
  - `runtimes`
- 標準 API 層
  - `std`
- 外部連携機能層
  - `features`
- 暫定・実験層
  - `kp`
- 独立ライブラリ層
  - `nm`
  - `neplg2`

### 4.2 依存方向

依存は原則として次の方向にのみ流れる。

```text
alloc
  -> core

alloc/collections / alloc/text / alloc/io
  -> alloc
  -> core

std/streamio
  -> alloc/io
  -> alloc/text
  -> runtimes

std
  -> core
  -> alloc
  -> std/streamio
  -> runtimes

features
  -> core / alloc
  -> std
  -> runtimes

kp
  -> core / alloc
  -> alloc/collections / alloc/text / alloc/io
  -> std / std/streamio

nm / neplg2
  -> 必要な基盤層へ依存する独立ライブラリ
```

### 4.3 各層の意味

- 基盤層は、他のすべての層が依存する最小土台を提供する。
- `core` は heap 不要の最小土台を提供する。
- `alloc` は heap 依存の一般用途機能を提供する。
- 実装差分吸収層は、target 差分と `runtimes` が吸収すべき実装差分を閉じ込める。
- 標準 API 層は、OS / device 依存だが利用者が日常的に使う安定した入口を提供する。
- `std` は `runtimes` の上に載る facade であり、利用者は原則として `runtimes` を直接意識しない。
- 外部連携機能層は、標準機能としては弱いが、公式に提供する API / FFI / デバイス接続を担当する。
- 暫定・実験層は、設計が固まりきっていない高水準 API を先行投入する場所とする。
- 独立ライブラリ層は、stdlib に含むが一般利用者向け標準 API とは別の責務を持つ。
- `stdlib` はこれらすべてを含む全体名称であり、`std` はそのうち標準 facade を担う一層に過ぎない。
- 互換性は `core` が最も広く、次に `alloc`、その次に `runtimes`、さらに `std`、最後に `features` が最も狭い。
- `features` は `runtimes` と同一ではないが、外部 API / FFI / デバイス接続を扱うぶん、通常は最も強い前提条件を要求する。

## 5. 新しいパッケージ構成

```text
stdlib/
    core/                # 純粋計算・基本型・基本 trait
        rand/            # target 非依存の乱数インタフェースと基本実装

    alloc/               # heap 依存だが target 非依存の汎用ライブラリ
        collections/     # Vec/Map/Set/Queue/Stack などの汎用データ構造
        text/            # str/String/Unicode/format/parse/文字列表現変換
        io/              # Reader/Writer/Seekable/Buffered などの低水準抽象
        diag/            # 診断値・診断構築補助
        encoding/        # json などの符号化/復号
        hash/            # HashMap/HashSet 向けのハッシュ実装

    std/                 # stdlib の一部。各個別 target を束ねる標準 API 面
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

## 6. 各パッケージの責務

### 6.1 `core`

- 純粋で target 非依存な基本機能だけを置く。
- 基本型、演算、基本 trait、`Result` / `Option` などの最小集合を担当する。
- OS / device / syscall / allocator 詳細に依存しない。
- heap を必要としないものを置く。
- `math.nepl` のように、厚い runtime wrapper を必要とせず、ほぼすべての runtime で共通に提供されるべき機能はここに置く。
- heap に依存しない計算ライブラリは原則としてすべて `core` に置く。
- `rand` も heap 不要な実装は `core/rand` に置き、heap を必要とする乱数アルゴリズムだけを将来的に `alloc/rand` へ分離する。

### 6.2 `alloc`

- `core` の上で、heap 依存だが target 非依存の汎用機能を提供する。
- allocator / 領域管理 / 診断補助 / encoding / hash をここへ置く。
- `MemPtr .T` / `RegionToken .T` は compiler/runtime 境界モジュールとして `alloc` 配下に置くが、safe user code からは抽象型 (`OwnedBuf .T`, `Slice .T` など) だけが見える。詳細は `doc/purity_ownership_memory_spec.md` を参照。
- 低レベル API はここに隔離し、上位層はできる限り安全 API だけを使う。
- `collections` / `text` / `io` のように、heap は使うが device や OS に依存しない層は `alloc` 配下に置く。

### 6.3 `alloc/collections`

- `Vec` / `Map` / `Set` / `Queue` / `Stack` など、一般用途のデータ構造を置く。
- device や OS に依存しないため `std` には入れない。
- allocator と trait 能力に基づき、move/effect/memory 規則と整合する API を提供する。

### 6.4 `alloc/text`

- `str` / `String` / Unicode / format / parse / 文字列表現変換を担当する。
- `bool/i32/i64/i128/...` と文字列の相互変換はここに置く。
- 標準入出力そのものは `text` の責務ではなく、`std/stdio` や `std/streamio` が利用する側とする。

### 6.5 `alloc/io`

- `Reader` / `Writer` / `Seekable` / `Buffered` など、I/O の低水準抽象を置く。
- syscall や descriptor などの具体実装は持たない。
- 「何が読めるか / 書けるか」の能力を trait として表現する。

### 6.6 `std/streamio`

- `alloc/io` の抽象を束ね、`read` / `write` / `flush` / `close` / event の高水準 API を提供する。
- `stdio`、`fs`、将来の socket / timer / process event / UI event などを、同じ stream 能力モデルで扱えるようにする。
- `streamio` は `std` 配下で、各個別 target の `stdio` / `fs` と `runtimes` 由来の差分を束ねる標準 stream facade とする。
- `kpread` / `kpwrite` の中核は最終的にこの層へ昇格させる。

### 6.7 `std`

- `core` / `alloc` ではないが、多くのプログラムで標準的に使う device / OS 依存 API を置く。
- `runtimes` / `std/streamio` を適切に包み、利用者向けに安定した標準 API 面を提供する。
- `std/stdio` は標準入出力、`std/fs` は標準ファイル操作、`std/env` は CLI 引数や環境取得を担当する。
- `std` は抽象化層ではなく facade であり、下位の複雑さを隠す。
- `std` は stdlib 全体そのものではなく、stdlib の中で target 依存 API を標準化して見せる役割だけを担う。

### 6.8 `features`

- `features` は、外部 API / FFI / デバイス接続のような、公式に提供する追加機能群を置く。
- `features` は `std` と異なり、「ほぼ全利用者が日常的に使う標準入口」ではない。
- GUI、HTTP、TUI、音声再生など、外界と接続する機能はここへ置く。
- 一方で、regex や audio buffer / audio processing のような純計算・データ処理は `features` ではなく `core` または `alloc` に置く。

### 6.9 `kp`

- 競技プログラミング向けの先行ライブラリを置く暫定層とする。
- ここは破壊的変更が頻発してよい前提で運用する。
- 十分に成熟し、責務が一般化できた機能は `std` または `alloc/collections` / `alloc/text` / `std/streamio` などへ昇格させる。
- 今回の `kpread` / `kpwrite` 由来の stream まとめ読み書き機能は、その昇格対象に当たる。

### 6.10 `nm`

- 拡張 markdown・doc comment・HTML 変換など、周辺ツールチェーン用の独立ライブラリとする。
- 一般的な標準 API 面ではないため `std` 配下へは入れない。

### 6.11 `neplg2`

- セルフホスト compiler 用ライブラリとする。
- 一般利用者向け標準 API とは別系統であり、`std` 配下へは入れない。

### 6.12 `runtimes`

- `runtimes` は target ごとの syscall / ABI / descriptor 差分を吸収する adapter 層。
- 各 target で挙動を共通化するために厚い wrapper が必要なものだけをここに置く。
- `math.nepl` のように厚い wrapper を必要とせず、runtime 差分をほぼ意識せずに提供できる機能は `runtimes` には置かない。
- wasip1 / wasip2 / wasix のような差分も `runtimes` に含めて扱う。
- 利用者は原則としてこれらを直接使わず、`std` や `std/streamio` を通して利用する。

## 7. trait と能力モデル

### 7.1 基本能力 trait

- `Eq`, `Ord`, `Hash`
- `Stringify`, `Debug`
- `Serialize .F`, `Deserialize .F`（フォーマット型 `.F` をパラメータに持つ）
- `Parse`
- `Default`, `Clone`, `Copy`
- `Add .U .R`, `Sub .U .R` などの演算 trait

### 7.2 表現系 trait の標準化

- stdlib 全体で、値の表現・記録・復元は trait を通して統一的に扱う。
- `stringify` / `debug` / `serialize` / `deserialize` / `parse` は、それぞれ別の責務を持つ能力として分離する。
- これらは個別モジュールごとに独自命名・独自シグネチャを乱立させず、共通 trait に実装を集約する。
- 代表的な役割分担は次の通りとする。
  - `Stringify`: 人間向けの安定した文字列表現を返す。
  - `Debug`: 開発時の調査用に、より詳細な表示を返す。
  - `Serialize .F`: 機械向けの外部表現 `.F` へ変換する。
  - `Deserialize .F`: 外部表現 `.F` から値を復元する。軽量 API では `Result .T StdErrorKind`、診断付き API では `Outcome .T StdErrorKind` を返してよい。
  - `Parse`: 人間が書いた入力を値として読む。戻り値形は `Result` / `Outcome` の使い分け規則に従う。
- `Stringify` と `Serialize` は同一視しない。
- `Debug` は `Stringify` の代替ではなく、詳細性と安定性の要件を分けて扱う。
- `Deserialize` と `Parse` も同一視しない。前者は機械表現からの復元、後者は人間入力の解析を担当する。
- `Parse` の詳細仕様、入力文法、エラー粒度、`Deserialize` との境界は別途標準化する。本仕様書では「trait 能力として統一的に扱う」ことだけを先に固定する。

### 7.3 Copy / Clone の扱い

- `Copy` / `Clone` は「組み込み型だから当然持つ能力」ではなく、`.nepl` ソース上で宣言される能力として扱う。
- compiler は「この型は Copy/Clone である」という固定知識をハードコードせず、trait 解決結果だけを move/effect 判定へ渡す。
- 将来的な derive 相当構文を導入する場合も、それは compiler 内固定表ではなく `.nepl` 側の明示宣言として扱う。

### 7.4 メモリ能力 trait

> メモリ管理の統合仕様は `doc/purity_ownership_memory_spec.md` を参照。値の 3 分類 (pure persistent value / unique mutable work state / linear capability) と region inference / drop elaboration の二段構えを前提とする。

- `RegionOwned`: 領域の所有権を保持する。
- `MemReadable .T`: `T` の読み取り能力を持つ。
- `MemWritable .T`: `T` の書き込み能力を持つ。
- `Allocator`: 領域確保・解放ポリシーを供給する。

### 7.5 I/O 能力 trait

- `Reader` / `Writer`
- `Seekable`
- `Buffered`
- `Stream`: `read/write/flush/close` を統一的に扱う。
- `EventSource .E` / `EventSink .E`: 将来的に stream 以外のイベントも同じ能力モデルへ乗せる。

## 8. diag の標準化

### 8.1 基本方針

- `diag` は、stdlib の失敗だけでなく、NEPLg2 全体のあらゆる診断情報を構造的に扱うための共通基盤とする。
- `diag` は compiler、stdlib、将来の selfhost compiler、追加ツール、DSL 実装系で共通利用できる形で標準化する。
- 現在の `error.nepl` は最終的に `diag` へ吸収し、stdlib の失敗表現は `Diag` を中心に再構成する。
- `Result` の `Ok/Err` と `Diag.kind` の `Log/Info/Warn/Error` は別軸として扱う。
- 新しい stdlib と compiler 基盤では、軽量な成功/失敗制御には `Result<T, E>` を使い、compiler・parser・json/html 変換などのリッチ診断を伴う処理には `Outcome<T, E>` を使う。
- `Result<T, E>` の既定の `E` は、`Diag` 全体ではなく標準化されたエラー kind を使う。
- debug 補助以外の公開 API で panic 相当挙動を標準にしない。

### 8.2 `StdErrorKind` の位置づけ

- `Result<T, E>` の既定の `E` には、`Diag` 全体ではなく `StdErrorKind` を使う。
- `StdErrorKind` は、軽量で比較しやすい標準エラー分類であり、stdlib 全体で共通に使う。
- `StdErrorKind` は `Diag.kind` のうち `Error` 系の標準分類と対応づける。
- `StdErrorKind` だけでは不足する詳細情報、warning/info/log、複数診断は `Outcome<T, E>` 側の `diags` で補う。

### 8.3 `Outcome<T, E>` の役割

- `Outcome<T, E>` は、`result` と `diags` に名前を与えてまとめて運ぶ構造体とする。
- 概念的には `(Result<T, E>, Diags)` と同等だが、field 名を持つ公開型として扱う。
- `Outcome<T, E>` は内部実装でも同じ構造体として扱い、tuple へ分解した ad-hoc 表現を標準にしない。
- `Outcome<T, E>` の役割は次の通りとする。
  - `result`: 処理そのものの成功/失敗を表す。
  - `diags`: 処理中に発生した `Log/Info/Warn/Error` を保持する。存在しない場合は `Option::None` としてよい。
- `Result` は制御フロー、`Diag` は報告情報であり、互いに独立した概念として設計する。
- これにより、
  - 成功したが warning を出す
  - 失敗したが info や log を併せて返す
  - compiler が値を返しつつ error diagnostic を保持する
  といったケースを自然に表現できる。
- `Outcome<T, E>` の field は必要最小限に留め、補助情報は `Option` を用いて任意化する。
- 最初の正式 field は `result` と `diags` とし、将来拡張が必要になった場合のみ optional field を追加する。

### 8.4 `Diags` の実体

- `Diags` は概念上の集合ではなく、正式な struct として定義する。
- `Diags` は複数の `Diag` を順序付きで保持するための入れ物とする。
- `Outcome<T, E>` の `diags` field は、この `Diags` struct を `Option` で包んだものとして扱う。
- `Result` を `Outcome` へ昇格するときは、診断がなければ `Option::None`、診断があれば `Option::Some diags` を使う。

### 8.5 `Result` と `Outcome` の使い分け

- `Result<T, E>` は、診断を伴わないか、伴っても呼び出し側が不要と判断できる軽量 API に用いる。
- `Outcome<T, E>` は、warning/info/log を失わずに返したい API、または compiler/analyzer/json/html のように値と診断群を同時に運びたい API に用いる。
- `Result<T, E>` は `Outcome<T, E>` へ損失なく昇格できるようにし、`Outcome` 側では `diags` を省略可能とする。
- stdlib の通常ライブラリでは `Option` / `Result` を標準とし、`Outcome` は例外的に rich diagnostic が必要な API に限定して使う。
- `Outcome` を使うかどうかは「診断群を持つべきか」に加えて、「source/span/help まで含めた rich reporting が本当に必要か」を基準に判断する。
- 診断を捨てて `Result` へ縮約する helper と、`Result` を `Outcome` へ包み上げる helper の両方を標準提供する。

### 8.6 `Diag` の責務

- `Diag` は単なるエラーメッセージ文字列ではなく、診断を構造的に運ぶ値とする。
- `Diag` は少なくとも次の情報を保持できるようにする。
  - `kind`
  - `message`
  - `span`
  - `notes`
  - `help`
  - `source`
- `span` は位置情報を持つ。存在しない場合は optional とする。
- `notes` は補足説明や原因候補を複数保持できるようにする。
- `help` は修正案・関連機能・参考情報を複数保持できるようにする。
- `source` は下位原因や内包した失敗を指せるようにし、失敗の連鎖を表現できるようにする。

### 8.7 `kind` の二層構造

- `kind` は NEPLg2 全体で共通に扱える標準分類と、各ライブラリ・各言語実装が定義できる独自分類の二層構造にする。
- 最上位には共通の大分類を持つ。
  - `Log`
  - `Info`
  - `Warn`
  - `Error`
- その下に、さらに階層化された kind を持てるようにする。
- 標準分類は ecosystem 全体で共通に解釈できる粗い粒度を提供する。
- 独自分類は、各ライブラリ・各ファイル・各 compiler・各 DSL が必要に応じて詳細化できるようにする。
- これにより、
  - 全ライブラリ横断では共通分類で集計・表示できる
  - 個別実装ではより詳細な分類で原因を表現できる
  という両立を目指す。
- `kind` は最終的に「実体は軽量整数、ソース上では階層化識別子」として扱える言語機能に接続する。
- 現在の enum/variant 機能ではこの表現を直接支えられないため、当面の仕様では `DiagKind` を構造化データとして設計し、将来的に専用の言語機能を追加する。

### 8.8 標準 kind と独自 kind

- 標準 kind では、NEPLg2 全体で共有すべき粗い分類だけを定める。
- 例えば compile/type/io/fs/alloc/invalid-state のような coarse-grained な分類を標準 kind として持てるようにする。
- 一方で、各ライブラリや各 compiler 実装は、それぞれ独自の詳細 kind を定めてよい。
- 例えば NEPLg2 標準では `CompileError` が粗すぎる場合でも、
  - parser 固有
  - typechecker 固有
  - DSL 固有
  の詳細 kind を独自に付けられるようにする。
- 独自 kind は標準 kind を置き換えるのではなく、その下位詳細として共存させる。

### 8.9 `help` と参照情報

- `help` は単なる短文ではなく、診断の解決案や参照先を表すための欄とする。
- `help` には、次のような情報を記述できるようにする。
  - 修正候補
  - 推奨 API
  - 関連するドキュメントへの参照
  - 将来的には doc / doccomment / symbol へのリンク
- doccomment の標準化自体は別仕様で扱うが、`Diag.help` はそれらを参照できる前提で設計する。

### 8.10 表示責務の分離

- `Diag` 自体は表示処理を内包しない。
- 表示は `Stringify` / `Debug` / `Serialize` と `std/stdio`、CLI/Web/LSP などの renderer の組み合わせで行う。
- したがって、現在の `error` / `diag` 実装にある表示専用 helper は、最終的に `Diag` の中核責務から分離する。

### 8.11 ecosystem 全体での位置づけ

- `diag` は stdlib 局所の例外表現ではなく、NEPLg2 の標準診断基盤とする。
- compiler 診断、stdlib の失敗、tooling、将来の selfhost compiler、DSL 実装系は、同じ `Diag` モデルの上で診断を構築できるようにする。
- `diag id`、`span`、`help`、`notes` は ecosystem 全体で再利用可能な共通資産とする。

## 9. 命名方針

- `_raw`, `_safe` 接尾辞は最終的に公開面から廃止する。
- 実装都合ではなく能力と意味に基づく名前を付ける。
- `to_xxx` は曖昧なので、
  - 失敗しない変換は `into_xxx`
  - 失敗する解析は `parse_xxx`
  に寄せる。
- ただし既存 API からの移行経路を考え、最終命名は各層ごとに段階的に揃える。

## 10. 個別論点

### 10.1 `kpread` / `kpwrite` の統合

- 現在の `kpread` / `kpwrite` は `kp` にあるが、責務としては「まとめて入出力する stream parser / formatter」であり、一般化可能である。
- そのため機能は `std/streamio` へ統合し、公開 API としての `kpread` / `kpwrite` は最終的に残さない。
- 競技プログラミング向けの説明やサンプルも、最終形では `std/streamio` の bare `read` / `write` / `writeln` / `flush` を直接使う形へ寄せる。
- `stdio` だけでなく `fs`、将来の他 stream 実装にも同じ parser / formatter を適用できる設計を目指す。

### 10.2 `std` と `streamio` の関係

- `alloc/io` は heap 依存だが target 非依存の低水準抽象を提供する。
- `std/streamio` は各 target の `stdio` / `fs` と `runtimes` 由来の差分を束ねる標準 stream facade である。
- `std` は `std/streamio` と `runtimes` を包み、利用者向けの安定した入口を提供する。

### 10.3 `features` と `std` の違い

- `std` は各 target の差分を束ねて共通 API を提供する標準 facade である。
- `features` は、標準機能としては弱いが、公式に提供する外部 API / FFI / デバイス接続機能を置く層である。
- `features` は `runtimes` そのものではなく、必要に応じて `std` や `runtimes` の上に載る追加機能群である。

### 10.4 `nm` / `neplg2` の独立性

- `nm` / `neplg2` は stdlib に含むが、言語の標準 API 面とは別の責務を持つ。
- したがって `std` や汎用層へ吸収せず、独立パッケージとして維持する。

## 11. `runtimes` と `std` の分離

- `runtimes/<target>` は target 別の最下層 adapter とする。
- `std` はそれらを直接見せず、安定した標準 API として包む。
- `std/streamio` は `runtimes` を束ねる標準 stream facade であり、利用者には target 差分を意識させない。

## 12. 移行の考え方

- 最終目標は新構成への全面移行であり、旧 API は最終的に完全削除する。
- ただし実装は段階的に進める必要があるため、移行途中では旧実装と新実装が一時的に併存しうる。
- その段階的移行手順そのものは `todo.md` に記述し、この仕様書には最終到達形だけを書く。

## 13. テスト方針

- `tests/` は階層的に整理し、少なくとも次の 2 系統へ分ける。
  - `tests/compiler/*`
    - stdlib との関係が薄く、compiler 自体の構文・型・名前解決・診断・codegen 前段検査などに誤りがないかを確認する。
  - `tests/stdlib/*`
    - stdlib の API、アルゴリズム、メモリ安全性、target facade、回帰ケースに誤りがないかを確認する。
- 既存の `tests/` 直下ファイルは、reboot の進行に合わせて上記構造へ段階的に移行する。
- compiler の仕様確認と stdlib の実装確認を同じ粒度で混在させない。
- trait 解決の曖昧性・重複 impl・能力不足は `compile_fail` で固定する。
- `alloc` / `alloc/collections` / `alloc/text` / `alloc/io` / `std/streamio` / `std` ごとに edge case を含む回帰を追加する。
- target 差分は adapter テストへ局所化し、共通挙動は同一ケースで検証する。
- `features` は外部 API / FFI / デバイス依存が強いため、各 feature ごとに integration test と adapter test を分けて検証する。
- `trunk build` + `nodesrc/cli.js` 系テスト + stdlib doctest をリリースゲートにする。

## 14. 期待効果

- 命名統一により API 学習コストを削減できる。
- trait 能力モデルにより、move/effect/memory と stdlib API の不整合を減らせる。
- target 依存責務の分離により、llvm/nasm/c/wasm の拡張が容易になる。
- `kp` の暫定実装から、成熟した機能を `std` / 汎用層へ順次昇格できる。
- compiler 側のハードコード依存を減らし、診断と意味論の一貫性を高められる。
- `Diag` を共通基盤にすることで、compiler・stdlib・selfhost・DSL 実装系が同じ診断モデルを共有できる。

## 15. 誤解しやすい点

- `core` / `alloc` / `std` の区別は「名前」や「利用頻度」ではなく、heap 依存性と target 依存性で決める。
- 計算ライブラリは、標準的によく使うから `std` に入るのではない。heap に依存しないなら原則として `core` に置く。
- `math` はその代表例であり、厚い runtime wrapper を必要としないため `core` に置く。
- `rand` も同様で、heap 不要な実装は `core/rand`、heap を必要とする実装だけを `alloc/rand` に置く。
- `std` は「よく使う汎用機能の置き場」ではなく、各 target と `runtimes` が吸収する差分を束ねて利用者向けの標準 API を提供する facade である。
- `stdlib` と `std` は同義ではない。`stdlib` はライブラリ全体、`std` はその内部にある target 依存標準 API 層である。
- `alloc` は heap を使うが target 非依存な汎用ライブラリの層であり、`collections` / `text` / `io` はこの基準で `alloc` 配下に置く。
- `std/streamio` は `alloc/io` の抽象を使って各 target の `stdio` / `fs` と `runtimes` 由来の差分を統一する層であり、`alloc/io` と役割が異なる。
- `diag` は stdlib の一部機能ではあるが、用途は stdlib 内部に閉じない。NEPLg2 全体の共通診断基盤として扱う。
- `core` / `alloc` / `runtimes` / `std` / `features` は、左ほど互換性が広く、右ほど要求条件が増える。配置判断では常により左へ置けないかを先に検討する。
