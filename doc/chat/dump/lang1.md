# ChatGPT 5.4 Thinking 20260316

> このファイルは当時の検討メモのダンプであり、現行仕様の正ではない。現在の仕様確認には `doc/neplg3/spec/` と Zenn #1 / #2 を参照すること。

今回は、NEPLg2 を「Rust 実装の compiler に付属するライブラリ群」としてではなく、**`stdlib/` の `.nepl` コードによって構成される自作言語プラットフォーム**として再定義し、そのうえで **最終到達点をセルフホスト**に固定した仕様書としてまとめます。判断の根拠は、Zenn 記事で明示された 4 原則、すなわちマルチプラットフォーム、強い静的検査、利用者自由度の最大化、具体先行の抽象化と、現行の stdlib reboot 文書にある安全 API 優先・trait による能力表現・`core -> alloc -> runtimes -> std -> features` の層分離・`stdlib` と `std` の区別、さらに `doc/self_host.md` にある「`/nepl-core` の Rust 製 compiler を使いながら `/stdlib/neplg2/src` に NEPLg2 製 compiler 本体を作る」という方針です。現行 `nepl-core` が `#![no_std]` を採り、source → lexer → parser → typecheck → codegen という bootstrap 用 pipeline を持つことも、この整理と整合します。 ([Zenn][1])

以下は、**現状説明ではなく、これから正とする提案仕様**です。モジュール名や論理 API 名の一部は、現行 repo にそのまま存在する実装名ではなく、NEPLg2 の設計思想に沿って固定した規範名です。

# NEPLg2 自作言語プラットフォーム仕様書 v1.0-draft

## 1. 文書の位置付け

本仕様は、NEPLg2 を **Language Platform** (`/ˈlæŋɡwɪdʒ ˈplætfɔːrm/`, 言語プラットフォーム) として定義する。ここでいう「言語プラットフォーム」とは、NEPLg2 本体だけでなく、JSON、CSV、XML、HTML、SVG、Markdown、LaTeX、NM、custommd、および将来のユーザー定義 **DSL** (`/ˌdiː ɛs ˈɛl/`, ドメイン固有言語 [ディーエスエル]) を、同じ共通基盤の上で記述・解析・検証・変換・整形・支援・実行できるようにする標準ライブラリ体系を指す。Zenn 記事では、NEPLg2 は tokenizer / parser / IR ライブラリを充実させ、独立文法を埋め込み、そのまま compiler や interpreter に成長させられる基盤を目指すと述べられているため、本仕様はその方向を正式化するものである。 ([Zenn][1])

本仕様において、利用者が触る主たる提供物は Rust crate ではなく、`stdlib/` 配下の `.nepl` モジュール群である。Rust 側は bootstrap compiler、最小 runtime、target/ABI adapter、外部 transport API の提供者であり、プラットフォームの主実装主体ではない。`doc/self_host.md` も、Rust 製 compiler を `/nepl-core` に置きつつ、NEPLg2 製 compiler 本体を `/stdlib/neplg2/src/core/` に作る方針を明示している。 ([GitHub][2])

## 2. 最上位目標

NEPLg2 自作言語プラットフォームの最上位目標は、次の 3 つである。

第一に、プラットフォーム依存部分を最小のホスト層へ隔離し、プラットフォーム非依存部分を極力 `.nepl` コードで共有することである。Zenn 記事では「プラットフォームに依存する部分を切り分けて適切に抽象化し、依存しない部分は全く共通のコードを用いるべきだ」と明言されている。 ([Zenn][1])

第二に、強い静的検査を備えた安全な標準基盤を構築することである。統合仕様では GC を導入せず、unsafe な操作は `Result/Option` で表し、Wasm と LLVM は同じ安全意味論を共有するが同じメモリレイアウトを共有する必要はない、とされている。これは platform 全体の安全設計原則として採用する。 ([GitHub][3])

第三に、最終的に **Self-hosting** (`/ˌsɛlf ˈhoʊstɪŋ/`, セルフホスト) を成立させることである。すなわち、NEPLg2 自作言語プラットフォームの主要部と、NEPLg2 compiler 本体は、最終的に `stdlib/neplg2` 側の `.nepl` 実装へ収束しなければならない。Rust 実装は bootstrap と検証のために残ってよいが、中心ではない。 ([GitHub][2])

## 3. 非目標

本仕様の非目標を明確に定義する。

第一に、旧 API との後方互換維持は本仕様の主目標ではない。統合仕様と `todo.md` は、旧 API 互換より最終構成への収束を優先すると明記している。 ([GitHub][3])

第二に、GC を標準メモリ管理戦略として導入しない。統合仕様は GC を非目標としている。 ([GitHub][3])

第三に、言語プラットフォームの便利機能を Rust 側に蓄積しない。便利ライブラリ、文書処理ライブラリ、埋め込み DSL ライブラリ、query/transform/format ライブラリは、原則として NEPLg2 で書かれなければならない。これは今回のユーザー指定を、仕様上の禁止事項へ昇格したものである。

## 4. 全体構造

### 4.1 二層構造

本プラットフォームは、**Bootstrap Host** (`/ˈbuːtˌstræp hoʊst/`, ブートストラップホスト) と **Platform Stdlib** (`/ˈplætfɔːrm ˈstændərd ˈlaɪbreri/`, プラットフォーム標準ライブラリ) の二層構造を取る。

Bootstrap Host は Rust 側である。ここには `nepl-core` を中核とする compiler pipeline、最小 runtime、target adapter、外部 API transport を置く。現行 `nepl-core` は `#![no_std]` で、source → indent-aware lexer → prefix + block parser → typecheck → codegen という pipeline を持っており、bootstrap 層として妥当である。 ([GitHub][4])

Platform Stdlib は `stdlib/` 側の `.nepl` 群である。ここに parser toolkit、PEG runtime、green-tree 風表層構文モデル、diag helper、query、rewrite、formatter、language island host、document renderer、execution harness、そして最終的な self-host compiler を置く。

### 4.2 中心は `stdlib/`

`stdlib_breaking_reboot.md` は、`core` / `alloc` / `runtimes` / `std` / `features` の層分離、`stdlib` と `std` の区別、安全 API 優先、能力は trait で表すことを設計原則としている。本仕様では、言語プラットフォームもこの層構造の中に配置し、`stdlib/` 全体を platform 本体とみなす。つまり `std` は利用者向け façade の一部にすぎず、`stdlib` 全体が標準機能の配布単位である。 ([GitHub][5])

## 5. ディレクトリ責務

### 5.1 `nepl-core` の責務

`nepl-core` は bootstrap compiler であり、最低限の lexer、parser、typecheck、lowering、backend、diagnostic infrastructure を提供する。ここは platform の最終本体ではなく、NEPL 側 platform をビルド・検証するための足場である。現行 `nepl-core` の public module 群はこの方向を示している。 ([GitHub][4])

### 5.2 `stdlib/` の責務

`stdlib/` は標準ライブラリ全体であり、platform 本体である。`stdlib_breaking_reboot.md` が定義する通り、`core` は heap 不要の最小基盤、`alloc` は heap 依存だが target 非依存の汎用基盤、`runtimes` は target 差分を吸収する adapter、`std` は標準 façade、`features` はより高水準で依存性の高い機能を置く層である。言語プラットフォーム用ライブラリもこの原則に従わなければならない。 ([GitHub][5])

### 5.3 `stdlib/neplg2` の責務

`stdlib/neplg2` は、一般 platform library の一部であると同時に、将来の self-host compiler source tree でもある。`doc/self_host.md` は `/stdlib/neplg2/cli/main.nepl` に interface を、`/stdlib/neplg2/src/core/` に compiler 本体を置く方針を明示している。したがって本仕様では、`stdlib/neplg2` を「NEPLg2 自身を再構築する標準機能」と位置付ける。 ([GitHub][2])

## 6. 公開 API の定義

### 6.1 API の意味

本仕様でいう **API** (`/ˌeɪ piː ˈaɪ/`, 公開境界 [エーピーアイ]) は、主として `.nepl` モジュールの public declaration、public trait、public type、public function の集合を指す。Rust の Web API、CLI API、LSP transport API は存在してよいが、それらは transport であって platform 本体仕様ではない。

### 6.2 `Outcome` と `Diag`

公開 API の戻り値は、安全 API 優先の原則に従い、`Option`、`Result`、`Outcome` を使い分けなければならない。とくに rich diagnostic が必要な場合は `Outcome` を標準形とし、`Result` は診断を省略した簡易形とみなす。`error.md` は `Diag` を `kind/message/span/notes/help/source` を持つ構造化診断値、`Outcome` を `Result` と `Diags` の組と定義している。std reboot 文書も同じ方針を採っているため、本仕様ではこれを platform 全体の共通通貨に固定する。 ([GitHub][6])

### 6.3 `.nepl` 上の trait を根拠にする

公開 API における能力表現は trait を用いなければならない。`stdlib_breaking_reboot.md` は、`Copy` / `Clone` / `Eq` / `Ord` / `Hash` / `Stringify` / `Debug` / `Parse` などの能力を trait で表し、どの型がどの能力を持つかは `.nepl` ソース上の宣言を唯一の根拠とし、compiler 内部に固定表を持たないと明記している。本仕様ではこの原則を platform API 全体へ拡張する。つまり、言語構築ライブラリの能力、document processor の能力、embedded DSL host の能力も、可能な限り `.nepl` 側 trait 宣言によって規定されなければならない。 ([GitHub][5])

## 7. 標準モジュール階層

以下の名前は提案仕様であり、現行 repo の物理パスと完全一致するとは限らない。ただし層分離は、この形を正とする。

### 7.1 `stdlib/core/lang/*`

ここには `span`, `source`, `diag`, `outcome`, `symbol`, `token_kind`, `lang_id`, `island_id`, `selector_id` のような heap 不要の基礎型を置く。`Diag` と `Outcome` はこの層の中心であり、すべての上位層が依存する。これにより、parser・formatter・query・runtime error を同じ診断形式で扱える。 ([GitHub][6])

### 7.2 `stdlib/alloc/lang/*`

ここには token buffer、tree node、arena、green tree、cursor、parser state、PEG runtime state、rewrite plan、query result table を置く。これらは便利ライブラリであり、Rust ではなく NEPLg2 で実装する。これは今回の前提を仕様へ固定したものである。

### 7.3 `stdlib/runtimes/lang/*`

ここには host call adapter、filesystem abstraction、sandbox policy、artifact cache、browser/native execution bridge を置く。低レベル API は利用者へ直接露出してはならず、`std` または `features` 側から安全に包まれなければならない。これは stdlib reboot の「低レベル API は内部実装または隔離層に閉じ込める」という原則に従う。 ([GitHub][5])

### 7.4 `stdlib/std/lang/*`

ここは通常利用者が import する façade である。`std/lang/json`, `std/lang/csv`, `std/lang/xml`, `std/lang/html`, `std/lang/svg`, `std/lang/md`, `std/lang/latex`, `std/lang/embed`, `std/lang/nepl` のような入口を置き、複雑な下位層を隠蔽する。

### 7.5 `stdlib/features/lang/*`

ここには高水準実装を置く。具体的には、各言語の parser/validator/formatter、selector/query engine、document transformer、embedded DSL host、renderer、実行支援などである。依存条件が強いものほど上位層へ置く、という stdlib reboot の方針をそのまま適用する。 ([GitHub][5])

## 8. 言語プラットフォームが提供すべきライブラリ

### 8.1 構文処理ライブラリ

プラットフォームは、Tokenizer Toolkit、Parser Toolkit、PEG Runtime、Layout/Block Parser、Streaming/Event Parser Support を提供しなければならない。Zenn 記事では tokenizer / parser / IR ライブラリの充実が明示されており、また NEPLg2 は前置記法・括弧なし・将来的な括弧との共存・埋め込み DSL を想定しているので、1 つの parser 方式に固定してはならない。 ([Zenn][1])

### 8.2 文書処理ライブラリ

プラットフォームは、JSON、CSV、XML、HTML、SVG、Markdown、LaTeX の parse / validate / query / transform / format を扱うライブラリを提供しなければならない。これらは compiler backend ではなく、構文を持つ言語全般の処理系として位置付けられる。

### 8.3 埋め込み DSL ライブラリ

プラットフォームは、**Language Island** (`/ˈlæŋɡwɪdʒ ˈaɪlənd/`, 言語アイランド) を first-class に扱う embedded DSL host を提供しなければならない。Markdown code fence、HTML `<script>`、LaTeX environment、NEPLg2 user-defined literal は、単なる文字列でなく、親言語中に埋め込まれた独立言語片として表現されるべきである。これは Zenn 記事の独立文法埋め込み構想を、platform 機能へ昇格したものである。 ([Zenn][1])

### 8.4 query / transform / format ライブラリ

プラットフォームは compile だけでなく、query / transform / format を第一級に扱わなければならない。とくに JSON/XML/HTML/Markdown/LaTeX 系では selector・rewrite・pretty print が本質だからである。これらも便利ライブラリに属する以上、原則 `.nepl` 実装とする。

## 9. 言語サービス仕様

### 9.1 現在の bootstrap 形

`editor_extensions.md` では、`nepl-web` は Web Playground 向け wasm API として維持し、editor extension 向けには別 Rust lib `nepl-language` を正とし、Zed / VSCode / 将来の WASIp1 Language Server は `nepl-language` を共通利用する、とされている。また editor 固有の薄い層だけを extension 側に置き、解析本体は compiler 実装を直接再利用し、将来的に extension 実装を NEPLg2 へ置き換える場合も薄い境界だけを置換すればよい、と明記されている。 ([GitHub][7])

### 9.2 本仕様での解釈

この方針はそのまま維持する。ただし、本仕様ではこれを bootstrap 段階と位置付ける。すなわち、現時点では `nepl-language` が Rust で token / diagnostic / semantic token / hover 向け情報を返してよいが、最終的にはその論理 API を `.nepl` 側 language service library と一対一対応させ、Rust 側は transport shell へ縮退させなければならない。`editor_extensions.md` 自体が「薄い境界だけを将来的に NEPLg2 実装へ置換する」方針を述べているので、本仕様はそれを強制条件として採用する。 ([GitHub][7])

## 10. セルフホスト要件

### 10.1 到達点

セルフホストとは、NEPLg2 で書かれた compiler が、NEPLg2 自身と `stdlib/` の主要部分を再コンパイルできる状態をいう。`doc/self_host.md` は、Rust 製 compiler を使いながら `stdlib/neplg2/src/core/` に NEPLg2 製 compiler 本体を作る、と明言しているため、本仕様でも self-host compiler source tree を `stdlib/neplg2` に置くことを正とする。 ([GitHub][2])

### 10.2 bootstrap 互換性

新しい stdlib 機能や platform 機能を追加するときは、少なくとも次の 3 段階を壊してはならない。

1 つ目は Rust bootstrap compiler が `stdlib/` をビルドできる段階。
2 つ目は Rust bootstrap compiler が self-host compiler source をビルドできる段階。
3 つ目は self-host compiler が自分自身と stdlib の主要部分を再ビルドできる段階である。

この **Bootstrap Compatibility** (`/ˈbuːtˌstræp kəmˌpætəˈbɪləti/`, ブートストラップ互換性) は、本仕様の必須要件とする。これは `todo.md` の「基盤から順に進める」「compiler のバグは library 側迂回でなく compiler 側を根本修正する」という方針とも一致する。 ([GitHub][8])

## 11. ドキュメント仕様

`stdlib_doc_comment_policy.md` は、ドキュメントコメントは手書きで、現在の実装と一致し、何をするかだけでなく、なぜその API にしたか、どのアルゴリズムか、どこが危険かまで書くことを要求している。本仕様では、これを language platform library 全体へ適用する。つまり、platform の一次仕様は別紙 PDF や Rust doc ではなく、`.nepl` ソースとその doc comment に置かれなければならない。ライブラリ仕様と実装は同期し、変更したコードに対応するコメントは同時に更新されなければならない。 ([GitHub][9])

## 12. 実装順序

実装順序は `todo.md` の stdlib reboot 順序に従う。すなわち、`diag/trait -> compiler 前提 -> core/mem -> alloc -> runtimes -> std -> features -> tutorials/tests` を正とする。platform library もこの順序に従い、まず `core/lang` と `alloc/lang` の基礎型・基礎アルゴリズムを整え、その後 runtime adapter、façade、各言語 feature、最後に tutorial と test を整備する。移行中に compiler の不備が見つかった場合は、library 側の回避ではなく compiler 側を修正する。これは `todo.md` の明示方針である。 ([GitHub][8])

## 13. 最終規定

NEPLg2 自作言語プラットフォームは、「Rust で実装された compiler に付属する補助ライブラリ群」ではない。**Rust bootstrap compiler に支えられながら、`stdlib/` の `.nepl` コードとして言語構築基盤そのものを育て、最終的には `stdlib/neplg2` の self-host compiler を中心に自己記述化していくシステム**である。Zenn 記事の 4 原則、stdlib reboot の層分離・安全 API・trait 主体設計、`Diag/Outcome` の共通診断モデル、editor の薄い shell 方針、そして self-host 指示は、この読み方で最もきれいに整合する。 ([Zenn][1])

この仕様で固定すると、今後の判断基準は明確になる。Rust 側に新機能を足すか迷ったら「それは bootstrap と adapter に不可避か」を問う。`.nepl` 側の API を設計するときは「safe surface か」「trait で能力を表せるか」「`Outcome` を返すべきか」を問う。各言語ライブラリを置くときは「`core/alloc/runtimes/std/features` のどこが正しい責務位置か」を問う。そして最後に、「この追加は self-host compiler へ移せるか」を常に問う。これが、NEPLg2 の設計思想に最も整合した、自作言語プラットフォーム仕様書としての固め方です。

[1]: https://zenn.dev/bem130/articles/8b6fe66d473e81 "Bemの自作プログラミング言語の歴史,設計哲学,設計思想 — NEPLg2はどのように生まれ、これからどうなるのか"
[2]: https://raw.githubusercontent.com/neknaj/NEPLg2/b6f9995c40b486247c75892c2cda0ba4cfaa45c6/doc/self_host.md "raw.githubusercontent.com"
[3]: https://raw.githubusercontent.com/neknaj/NEPLg2/b6f9995c40b486247c75892c2cda0ba4cfaa45c6/doc/purity_ownership_memory_spec.md "raw.githubusercontent.com"
[4]: https://raw.githubusercontent.com/neknaj/NEPLg2/b6f9995c40b486247c75892c2cda0ba4cfaa45c6/nepl-core/src/lib.rs "raw.githubusercontent.com"
[5]: https://raw.githubusercontent.com/neknaj/NEPLg2/b6f9995c40b486247c75892c2cda0ba4cfaa45c6/doc/stdlib_breaking_reboot.md "raw.githubusercontent.com"
[6]: https://raw.githubusercontent.com/neknaj/NEPLg2/b6f9995c40b486247c75892c2cda0ba4cfaa45c6/doc/error.md "raw.githubusercontent.com"
[7]: https://github.com/neknaj/NEPLg2/blob/b6f9995c40b486247c75892c2cda0ba4cfaa45c6/doc/editor_extensions.md "NEPLg2/doc/editor_extensions.md at b6f9995c40b486247c75892c2cda0ba4cfaa45c6 · neknaj/NEPLg2 · GitHub"
[8]: https://raw.githubusercontent.com/neknaj/NEPLg2/b6f9995c40b486247c75892c2cda0ba4cfaa45c6/todo.md "raw.githubusercontent.com"
[9]: https://raw.githubusercontent.com/neknaj/NEPLg2/b6f9995c40b486247c75892c2cda0ba4cfaa45c6/doc/stdlib_doc_comment_policy.md "raw.githubusercontent.com"
