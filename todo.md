2026-02-10 trunk build復旧後の優先実装計画

現状把握
- `NO_COLOR=true trunk build` は成功する
- compiler artifact は `web/dist/nepl-web-*_bg.wasm` と `web/dist/nepl-web-*.js` に出力される
- `node nodesrc/tests.js -i tests -i tutorials -i stdlib -o /tmp/nmd-tests-after-trunk.json -j 4` は `total=326, errored=326`
- 主因は `nodesrc/util_paths.js` の探索順序で、存在確認だけで `dist/` を選び、artifact 未存在のまま `nodesrc/compiler_loader.js` が失敗すること

実装計画
1. dist探索の根本修正
- `candidateDistDirs` の「存在する最初のディレクトリ」を採用する方式をやめる
- `compiler_loader` 側で `nepl-web-*.js` と `*_bg.wasm` のペアが存在するディレクトリのみ採用する
- 複数候補がある場合は `web/dist` と `NEPL_DIST` を優先し、理由をエラーメッセージに出す

2. テスト導線の強化
- `nodesrc/tests.js` に `--dist` 指定時の検証ログを追加し、どの候補を採用したかをJSONに記録
- `--dist` 未指定時に候補全滅なら、探索した全パスをまとめて表示して調査時間を減らす

3. 回帰テストの追加
- `dist/` は存在するがartifactなし、`web/dist/` にartifactあり、という今回の再現ケースを固定テスト化
- `NEPL_DIST` 指定時の優先挙動をテスト化

4. 手順とCI整合
- `doc/testing.md` と workflow の実行例を、`trunk build` 後に `nodesrc/tests.js` が確実に同じ出力先を参照する書き方へ統一
- 必要なら workflow 側で `--dist web/dist` を明示

完了条件
- `trunk build` 直後に `node nodesrc/tests.js ...` を `--dist` 省略で実行しても `errored=0`
- 失敗が出る場合はテスト内容由来の `failed` のみになること

進捗 (2026-02-10)
- 1. dist探索の根本修正: 完了
- 2. テスト導線の強化: 完了（`resolved_dist_dirs` をJSON出力に追加、stdoutに `dist.resolved` を表示）
- 実測: `node nodesrc/tests.js -i tests -i tutorials -i stdlib -o /tmp/nmd-tests-after-fix.json -j 4` で `passed=250, failed=76, errored=0`

2026-02-10 コンパイラ再設計計画 (抜本改善)

現状計測
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests -o /tmp/tests-only.json -j 4`:
  - `total=309, passed=240, failed=69, errored=0`
  - 主要失敗傾向:
    - `expected compile_fail, but compiled successfully`（仕様検査漏れ）
    - `expression left extra values on the stack` / `return type does not match signature`（block/単行構文と型整合の境界）
- `nepl-core/src` 現状:
  - 巨大ファイル: `parser.rs`(114k), `typecheck.rs`(165k), `codegen_wasm.rs`(59k)
  - `module_graph.rs` / `resolve.rs` は存在するが `compile_wasm` 本流に未統合
  - ビルド時警告が多く、未使用経路・未整理コードが残存

根本課題
- 処理フローが段階分離されておらず、仕様追加時に parser/typecheck/codegen へ広く波及する。
- 名前解決が DefId 一貫になっておらず、文字列ベース解決が残る。
- parser/typecheck の肥大化により、plan.md 要件（単行 block / 条件式構文 / 将来 target 拡張）に追従しづらい。

再設計方針
- big-bang 置換は避け、既存パイプラインを維持しながら内部を段階置換する。
- コンパイラ本流を次の固定段階へ再編:
  1) ParseFront
  2) ModuleGraph
  3) Resolve(DefId)
  4) TypeCheck
  5) MIR + move/drop
  6) Backend(codegen)
- すべての段階で入力/出力データ構造を明示し、段階間の責務重複を禁止する。

フェーズ別計画
1. 安全網の先行整備
- `tests-only failed=69` をカテゴリ管理し、回帰検知のベースラインJSONを固定。
- 受け入れ条件:
  - 失敗件数/カテゴリを毎回比較可能
  - pass ケースの後退がない

2. Frontend 分割（parser再編）
- `parser.rs` を `directives / items / expr / block` に分割。
- plan.md の単行 block・if/while 引数改行規則を独立モジュール化。
- 受け入れ条件:
  - `tests/block_single_line.n.md`
  - `tests/block_if_semantics.n.md`
  で compile_fail/pass の判定が仕様どおりになる

3. ModuleGraph + Resolve の本流統合
- `compile_wasm` 入口に ModuleGraph 経路を接続（最初は feature flag 切替）。
- `resolve.rs` を DefId 出力まで完成し、typecheck 入力の文字列依存を削減。
- 受け入れ条件:
  - import/use/alias の曖昧性診断が安定
  - `tests/loader_cycle.n.md`, `tests/resolve.n.md` の期待が満たされる

4. TypeCheck 再編
- `typecheck.rs` を `constraint生成 / unify / effect検査 / block-stack検査` に分割。
- `<T>` の扱いを「関数適用的な旧処理」から「型 ascription」に明確化。
- 受け入れ条件:
  - `expected compile_fail` の逆転ケースを優先解消
  - `tests/typeannot.n.md`, `tests/functions.n.md`, `tests/generics.n.md` を重点改善

5. MIR/Pass 正規化
- move_check/drop_insertion を MIR パス化し、順序依存を削減。
- `passes` 層の単体テストを追加。
- 受け入れ条件:
  - `tests/move_check.n.md`, `tests/drop.n.md` の安定化

6. Backend 境界整理
- `codegen_wasm.rs` を MIR 入力前提で薄くし、target 拡張可能な backend trait を導入。
- target 再設計（wasm/wasip1/wasip2/wasix/nasm/c）をこの段階で吸収。
- 受け入れ条件:
  - 現行 wasm/wasi の互換維持
  - backend 追加時の変更範囲を最小化

次スプリントで着手する具体タスク
- A. `/tmp/tests-only.json` を元に fail 69 件の分類表を作成（仕様漏れ/実装バグ/診断差分）
- B. parser の block/if 周辺だけ先行分割して、単行 block 失敗群を先に潰す
- C. `compile_wasm` に ModuleGraph 経路の実験フラグを追加し、段階置換を開始
- D. `note.md` に毎回「失敗件数・主因・差分」を必ず記録

実装進捗 (2026-02-10 追加)
- フェーズ1（安全網）:
  - `nodesrc/analyze_tests_json.js` を追加し、`tests.js` 結果JSONの失敗理由をカテゴリ集計できるようにした
  - 現在の基準値:
    - `node nodesrc/tests.js -i tests -o /tmp/tests-only-after-phase2.json -j 4`
    - `total=309, passed=240, failed=69, errored=0`
    - 主要カテゴリ: `stack_extra_values=25`, `compile_fail_expectation_mismatch=10`, `indent_expected=7`
- フェーズ2（compilerフロー整理）:
  - `nepl-core/src/compiler.rs` を段階関数（typecheck/move_check/codegen）へ分割
  - 公開APIと主要型へ日本語docコメントを追加
  - 挙動は維持（tests件数は同一）


=== ここまで編集自由 ===
---
### 以下編集禁止

cast関連の実装中 fnのalias用法

<...> の中(型注釈や型引数として読む場所)で`::` PathSep を許可

複数行文字列リテラルの実装
plan.mdの文字列の項を参照

examples/nm.nepl, stdlib/nmの実装
ドキュメントコメントのパーサーとしても使えるよう、行頭の`//: `や`//:|`を扱うかのフラグを用意しておいて
parserでは、Resultを用い、エラーメッセージを適切に提供すること
stdlib/nm/README.n.mdを確認し、stdlib/nm/README.n.mdがhtmlに変換できるようにする

ドキュメントコメントの整備
`//: `によるドキュメントコメントを追加
ドキュメントコメントあるとき、次の行には何らかの定義が来る
ドキュメントコメントはその定義に紐づけられる
`neplg2:test`によってテストを記述し、doctestコマンドでテストを実行できるようにする
`//:|`の行はドキュメントではデフォルト非表示にする testコードのimportなどの重要度が低い部分を隠すために使う
/AGENTS.mdや/examples/stdio.neplを参照

neplg2のドキュメントコメントは、stdlib/nmを使ってパースやAST構築、html変換などを行う
Wasmiを使ってRustのコンパイラと統合する

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

# targetの追加,再設計
現状: wasm か wasi
変更後: nasmを追加, wasip1 wasip2 wasix に変更
包含関係を上手く処理できるように注意すること
定義する側と、使用する側で、包含関係の判定処理が異なることなどに注意すること (定義する側(ライブラリ側)は依存を減らす「これさえあれば動く」、使用する側は依存できる先を増やす「これらのどこでも動く」)
```
if[target=wasm]
if[target=wasm&wasip1]
if[target=wasm&wasip1&wasip2]
if[target=wasm&wasip1&wasix]
if[target=nasm]
if[target=nasm|wasm]
if[target=nasm|(wasm&wasip1)]
if[target=nasm|(wasm&wasip1&wasip2)]
if[target=nasm|(wasm&wasip1&wasix)]
```
こんな感じ

NASM target, C targetの追加
stdlib/coreとstdlib/allocはNASMとCとWASMの全部に対応させる
stdlib/stdはNASMとCとWASM&WASIP1の全部に対応させる
WASIp2やWASIXが必要な機能はstdlib/platformsで扱う
また、今後のtarget追加があった時に柔軟に対応できるような設計とする

targetのエイリアスの追加

coreはnasm|c|wasm
stdはnasm|c|(wasm&wasip1)
```
if[target=core]
if[target=std]
```

tupleの書き方の変更
現行の`(a,b)`の記法は廃止して、他の書き方になじむよう
```
Tuple:
    a
    b
```
のような構文に変更
テストケースにある旧記法は新記法に置き換える
フィールドアクセスは廃止 (a.0, a.1 など)
field.neplのget,putによってアクセス

単行ブロック式の追加
plan.mdの単行ブロックの項を確認すること

パイプ演算子の改良,活用
パイプ演算子を改行して書けるようにする
標準ライブラリなどで、パイプ演算子を活用して書けるようにする
plan.mdのパイプ演算子の項を確認すること

stdlib/alloc/encoding/json.nepl
数値はf64として扱うように変更
serialize,parseの機能を追加
parserでは、Resultを用い、エラーメッセージを適切に提供すること

NEPLg2でセルフホストコンパイラを作る
stdlib/neplg2/
Rustの現実装のように、WASM依存のみでWASIに依存しないcoreと、stdやfsなどを扱うWASIに依存するcliに分けて実装する
