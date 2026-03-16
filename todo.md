2026-03-09 stdlib reboot 実装計画

方針
- `plan.md` と `doc/stdlib_breaking_reboot.md` を正として実装する。
- ドキュメントコメント整備は `doc/stdlib_doc_comment_policy.md` を正として進める。
- 実装を変更した箇所のドキュメントコメントは、後回しにせず同じタイミングで必ず整備する。
- `todo.md` には、reboot 仕様を問題なく実装するための順序だけを書く。
- `tests/` の各テストケースには、そのケースの[目的/もくてき]と、何を[確/たし]かめるためのものかを日本語で丁寧に書く。
- `nodesrc/` のツールは、stdlib reboot の検証効率を上げるために適宜改良してよい。
- stdlib のドキュメントコメント内 `neplg2:test` は `nodesrc/tests.js` が `.nepl` 内の `//:` doctest として走査・実行する。stdlib 側の doctest を確認したいときは `node nodesrc/tests.js -i stdlib/... --no-tree -o ... -j 15` を基本形にする。
- `tests/compiler/*` と `tests/stdlib/*` の通常テストも `nodesrc/tests.js` で実行する。compiler 側だけを見るときは `-i tests/compiler/...`、stdlib 側だけを見るときは `-i tests/stdlib/...` を使い、移行中は範囲を絞って原因を切り分ける。
- doctest 1 件だけを最短で確認したいときは `nodesrc/run_test.js` に JSON を渡す focused 実行も使ってよい。
- stdlib / tutorials / tests に埋め込まれた doctest 1 件を直接確認したいときは `node nodesrc/run_doctest.js -i <file> -n <index>` を使って、該当ケースだけを再現する。
- stdlib 再構築は、依存の強い基盤から順に進める（diag/trait -> compiler 前提 -> core/mem -> alloc -> runtimes -> std -> features -> tutorials/tests）。
- compiler のバグを発見した場合は、library 側の迂回ではなく compiler 側を適切に根本から修正する。
- 間に合わせ修正を避け、旧 API の互換維持ではなく最終構成への収束を優先する。
- `io` / `streamio` と、それに準ずる stdlib の入出力 API は bare 名 `read` / `write` / `writeln` / `flush` / `close` を正とし、`scanner_read_*` / `writer_write_*` / `read_i32` / `write_str` のような prefix / suffix 付き命名は残さない。
- 入出力 API の型差はオーバーロードと trait で解決し、後方互換 alias は作らない。既存コードはすべて新しい bare 名へ書き換える。
- 実装完了項目はここから削除し、経過・差分・判断理由は `note.n.md` に記録する。

stdlib 再構築 本流

1. diag/trait 層の土台確定
- `Result` と `Outcome` を共通に扱う helper は導入済み。trait 抽象は associated type / trait generic 機能の整理後に再検討する。
- `Outcome` は読み取り helper を先に整備し、struct の多フィールド抽出を要する mutating helper は言語機能側の制約を確認しながら段階的に進める。
- `Copy` / `Clone` / `Stringify` / `Debug` / `Eq` / `Ord` / `Hash` / `Serialize` / `Deserialize` の stdlib trait 本体と `Result` / `Outcome` 共通 helper は配置済みなので、以後は associated type / trait generic 機能を前提に抽象化の整理を進める。
- `Diag.kind` を支える言語機能追加の計画と前段実装を進める。軽量実体を持ちながら階層識別子として扱える kind 表現を言語機能として生かし、compiler / selfhost / DSL 実装が共通 kind 体系を利用できるようにする。
- 完了条件:
  - trait 能力の責務が `core` / `alloc` / `std` の配置と一致する。
  - reboot 仕様に必要な kind 体系を支える実装方針が確定する。

2. compiler 前提の固定と診断・効果モデリング整備 (Migration Phase 0)
- codegen では診断を出さず、前段で完結させ、wasm/llvm の安全意味論・診断規則を共通化する。
- `Effect` enum を拡張し `InternalAlloc`, `ExternalIO`, `Nondet`, `Unsafe` を追加、`to_surface()` 畳み込みを実装する。
- builtins の `alloc/dealloc/realloc` や `load/store` の effect を surface `Pure` から外し、`Effect::InternalAlloc` に変更。
- memory safety 系診断 ID (5001-5008) を予約し、型分類子 `ValueCategory` の仕組みを導入する。
- 完了条件:
  - codegen 到達時は生成成功前提となり、wasm/llvm 共通診断化され、コンパイラ内部の効果・状態追跡の土台が整う。

3. Module System 実装と名前解決の刷新 (Migration Phase 0.5)
- ファイルとモジュールの直交化を parser / resolver レベルで実装する。
- `merge` 指令による同一モジュール内ソース合成を実装する。
- Anchor Part の概念を導入し、canonical なモジュールパス解決を強制する。
- 非 canonical パスでの `use` に対する警告と正規化、および曖昧な解決のコンパイルエラー化。
- 完了条件:
  - 複数の .nepl ファイルを 1 つの論理モジュールとして `merge` でき、anchor path を基点とした名前解決が安定して動作する。

4. core/mem のポインタ隔離と安全境界の構築 (Migration Phase 1)
- `_raw` 名依存や backend ごとの差分診断を共通検査へ寄せる。
- `MemPtr<T>` の raw address 露出を safe API から消し、`mem_ptr_addr` / `alloc_raw` などを compiler/runtime 境界に閉じ込める。
- 生 `i32` load/store 等を隠蔽し、型付きポインタでのアクセスへ置換する。
- Wasm/LLVM の表現分離（安全意味論は codegen 前に完結、target lowering は物理表現のみ担当し `#if[target="..."]` で吸収）。
- 完了条件:
  - `_raw` / `_safe` の公開命名が消え、公開面に生ポインタ前提 API が残らない。

5. alloc 層の新構成移行とコレクション安全化 (Migration Phase 1, Phase 2)
- `alloc/string` 等の主要確保経路を `RegionToken<u8>` + `MemPtr<u8>` な型付き領域へ分離 (`StringBuilder` / `ByteBuf` / `str` の分離)。
- 各種コレクション (`Vec`, `HashMap`, `Queue` 等) を `MemPtr<T>` / `RegionToken<T>` 前提の型付き API へ統一する。
- `List<T>` の public `free` を削除して persistent list に固定し、`tail` セマンティクスを正式に共有へ変更する (Phase 2)。
- `alloc/text` の文字列表現変換・数値変換・真偽値変換を trait 設計と整合させる。
- `alloc/io` に低水準抽象（Reader/Writer/Seekable/Buffered）を集約し、`alloc/encoding` / `alloc/hash` / `alloc/diag` の責務を再配置する。
- 完了条件:
  - `alloc` 層の公開 API が Wasm/LLVM 共通の新しいメモリモデルと整合する。

6. compiler 解析パスの強化と自動メモリ管理 (Migration Phase 4, Phase 5, Phase 6)
- `set` の purity 規則を「局所なら pure」から escape analysis ベースの新規則へ変更する (Phase 4)。
- `VarState` (Live/Moved/MaybeMoved 等) を用いた変数状態追跡・借用チェッカーを導入する (Phase 5)。
- Resource IR と ownership/borrow check pass を導入し、owned object (`File`, `Socket`, `ListBuilder<T>` など) に対し Drop Elaboration を実装する (Phase 5)。
- pure persistent value (`List<T>`, `str` 内部) に対する Region Inference の first version を実装する (Phase 6)。
- 完了条件:
  - OOB / UAF / double free / use-after-move / borrow conflict 等が Resource IR 上の compile error または `Result<Err>` として検出される。
  - 値の 3 分類 (pure persistent / unique mutable / linear capability) に基づいた GC レスの自動メモリ管理が稼働する。

7. runtimes 層を整理する
- target ごとの差分と厚い wrapper が必要な機能だけを `runtimes` に集める。
- `math` のような `core` へ置くべきものを `runtimes` に持ち込まない。
- wasip1 / wasip2 / wasix などの差分を `runtimes` 配下で整理する。
- 完了条件:
  - `runtimes` の責務が `std` や `features` と重複しない。
  - target 差分を `std` が包める状態になる。

8. std と std/streamio を再構築し、IO効果を適用する (Migration Phase 3)
- `std/io`, `std/stdio`, `std/fs`, `std/env/cliarg` 等の effect を文字列マーカーから宣言的 (`ExternalIO`) に変更する (Phase 3)。
- `std/streamio` を `alloc/io` 抽象の上に構築し、`runtimes` を直接見せない facade にする。
- `kpread` / `kpwrite` の機能を `std/streamio` へ統合し、公開 API としての `kpread` / `kpwrite` は最終的に撤去する。
- `io` / `streamio` / `kpread` / `kpwrite` を含む stdlib 全体の入出力 API から prefix / suffix 付き read/write 名を撤去し、bare overload へ統一する。
- `TUI` は `std` ではなく `features/tui` として扱う。
- 完了条件:
  - `std` が target 依存標準 API の facade として一貫し、I/O に `ExternalIO` 効果が正しく付与される。
  - `kpread` / `kpwrite` を使わずに `std/streamio` だけで同等の入出力が書ける。

9. features 層の残作業を整理する
- GUI / HTTP / 音声再生のような外部 API / FFI / デバイス接続を `features` へ配置する。
- regex や audio buffer/processing のような計算・データ処理を `core` / `alloc` へ戻す。
- `features` は `std` や `runtimes` の上に載る追加機能群として整理する。
- 完了条件:
  - `features` の責務が `std` と混ざらない。
  - TUI 以外の既存外部連携コードの配置方針も固定される。

10. tests / tutorials / docs を新 stdlib に追従させる
- `compile_fail` に `diag_id` を付ける。
- 診断位置検証の仕組みを追加する。
- tutorials を新ライブラリ構成と新 API に合わせて書き直す。
- `Part6` を含め、短く・安全で・ライブラリを活かした書き方へ改稿する。
- 完了条件:
  - 新構成で tests / tutorials / stdlib doctest が通る。
  - 旧ライブラリ前提の説明が消える。

stdlib 再構築と直接は関係しない別件

1. Web Playground / tests.html 強化
- 名前解決/型情報/式範囲/定義ジャンプ候補の表示を強化する。
- `web/tests.html` で AST / resolve / semantics を詳細表示できるようにする。

2. `examples/js_interpreter`
- stdlib 再構築後のライブラリを使って JavaScript インタプリタを整備する。
- Node.js 実行結果との同値性回帰を追加する。

---
### 以下LLM編集禁止 (人の指示領域)

# stdio, io
io,streamioは全て型名のprefix,suffixを用いず単にread,writeだけで扱えるようにする
ioのtargetをfsやstdioやnetworkやeventとすると、
`read target` や `write target data` の形式、或いはpipeを用いて `target |> read` や `target |> write data`の形式で書けるようにする
単発は前者、いくつか纏めて書き込むときは後者、
或いは pipeで処理してきたdataを `data |> write target` のようにもようにする
targetはそれぞれ型のオーバーロードで扱えるように適切に型を付け、traitで統一的に扱う
ioのtargetはEnumなどで引数で切り替えるようにして
streamioのtargetはそれぞれのstream

<...> の中(型注釈や型引数として読む場所)で`::` PathSep を許可
それに従って型の名前空間,importの挙動を適切に修正

examples/nm.nepl, stdlib/nmの実装 再開発
現状簡易実装であり、nodesrc/のものを用いている (nodesrcのものもより進んではいるが簡易実装)
stdlib/nm/README.n.mdを確認し、stdlib/nm/README.n.mdがhtmlに変換できるようにする
nodesrc/のhtml_gen.jsやhtml_gen_playground.jsの機能を参考に、glossとnestの扱いを改良し、
README.n.mdやその他の.n.mdや.neplを実際に変換し、先ずはnodesrc/のhtml_genの出力と比較しながら、
nodesrc/と一致してからはnodesrc/の実体をjsからstdlib/nmに乗り換え(現在のjs仮実装は反故)たあと、さらに実際の.n.md入力とhtml出力を吟味しながら未実装md構文などnmを改良する

## LSP関連
テキストエディタなどで使用するための情報を、NEPLコンパイラが出力できるようにする
tokenごとに、型の情報や式の範囲、引数の範囲、定義ジャンプのジャンプ先などの情報を取得できるようにする
オーバーフローで表示するドキュメントコメントの内容も取得できるようにする
エラーや警告などの位置も取得できるようにする
定義ジャンプなど、importされている場合はそのファイルにジャンプできるよう、ファイルを跨いだ情報を提供する

### エラー回復など
1つのエラーを検出したら直ちに終了するのではなく、できる限り多くのエラーを報告するモダンなコンパイラを目指します
インデントの仕方に強い制約があるため、インデントの情報などを使用することができるはずです
例えばインデントズレなどを検出することができるかもしれません
結果をキャッシュしておきインクリメンタルに更新できるよう設計

### VSCode拡張機能
WASIp1を用いたLanguage Serverを提供する
Semantic Highlightingを提供する
Testing APIやCodeLensを利用(ドキュメントコメント内のテストの実行ボタン)
Hoverでドキュメントコメントや型を表示
Inlay Hints を提供 (式の型や括弧を表示する)

### Zed拡張機能
VSCodeと同じような機能を提供する

#### 行単位
単行ifや単行block式などに対して括弧を表示
let直後の式や単行ifや単行block式などに対して型注釈(前置)を表示
(例)
```
let a if true then add sub 5 3 1 else block let b sub 6 add 1 2; add b 2 // ソースコード
let a <i32> if (true) then (add sub 5 3 1) else (<i32> block let b <i32> sub 6 add 1 2; add b 2) // Inlay Hint 表示
```

#### 関数単位
`fn add (a,b)`
が定義されていたとして、
```
add add 1 2 add 2 3
```
みたいなコードで、一つ目のaddにカーソルがあるとき、
```
<i32> ad|d a:(<i32> add 1 2) b:(<i32> add 2 3)
```
こんな風に表示 Inlay Hint, a,bにInlayHintLabelPart, offUnlessPressed

# targetの再設計
現状targetでruntimesとfeaturesがぐちゃぐちゃに扱われている
FFI,APIの仕組みも整理しながら、適切にtargetを再設計する

# tupleの書き方の変更
現状、
```
Tuple:
    a
    b
```
2要素と3要素のTupleにPairとTripleの標準名を与え、そのように書き直す
```
Pair a b
Triple a b c
```

# 型の前置記法化
現状、型は前置記法となっていない
適切に再設計し、他の記法と同様に前置記法化を進める

# patternの設計
letやmatch caseでpatternが使えるようにする
structやtupleの分解などが行えるようにする
型の記法と構文にを持たせる
コード例
```
let p Pair 1 2
let Pair a b p // pを(a,b)に分解
```

# 言語処理系

stdlib/alloc/encoding/json.nepl
数値はf64として扱うように変更
serialize,parseの機能を追加
parserでは、Resultを用い、エラーメッセージを適切に提供すること

NEPLg2でセルフホストコンパイラを作る
stdlib/neplg2/
- 2-layer 言語プラットフォーム構造（Bootstrap Rust Host + Platform Stdlib）の完成。
- Rust の現実装のように、WASM依存のみでWASIに依存しないcoreと、stdやfsなどを扱うWASIに依存するcliに分けて実装する。
- 言語島（Embedded DSL）や構文処理ツールキットを stdlib に整備する。
- 完了条件:
  - 自作コンパイラが自身のソースコードをコンパイルし、Rust ホスト版と同一の Wasm を出力できる。


# 新tutorial作成計画

現在のgetting_startedの1種類しかない暫定のtutorialを
1. プログラミング初心者向けのプログラミング解説
2. プログラミング経験者向けのNEPLg2解説
3. 競技プログラミング経験者向けのNEPLg2での競技プログラミングの書き方解説
この3本に分けて完全に整備しなおす
nmを用いて総ルビで作成する
tutorial作成中にライブラリの不備や不足が見つかったら適切に改良する

doc/new_tutorial_plan.mdを参照