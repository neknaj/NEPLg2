2026-02-10 今後の実装計画

方針
- plan.md の仕様を唯一の基準として、上流（lexer/parser）から順に修正する。
- 間に合わせ修正を避け、原因が同一の失敗はまとめて解消する。
- 実装順序は「名前空間の再設計」を最優先とし、Value と Callable の責務を分離する。

直近の実装タスク（未完了のみ）
1. 名前空間再設計（最優先）
- `typecheck` の環境を `ValueNs`（変数）と `CallableNs`（関数/alias）に分離する。
- `let`/`fn` の巻き上げを plan.md 準拠で統一する（`mut` なし `let` と `fn` のみ）。
- nested `fn`/`let` を呼び出し可能にする経路を確立する（少なくとも `tests/functions.n.md` の `double` / `add_y` を通す）。
- 関数値（`@fn` / 関数を値として渡すケース）を HIR で明示表現し、`CallableNs` と整合する解決規則にする。
- ローカル束縛と import/alias の同名衝突で、ローカル側が常に優先されるシャドーイング規則を実装・固定する（`kpread` の `len` 系再発防止）。
- シャドーイングはエラーにせず合法のまま維持しつつ、`print` / `println` / `add` など重要 stdlib 記号 warning の無効化フラグと抑制ルール（プロジェクト単位設定）を設計する。
- 宣言修飾子として「シャドー不可（上書き不可）」を導入する（対象は immutable の `let` / `fn` のみ。`let mut` には適用しない）。
- stdlib 側では `result` など基盤 API の immutable 定義にシャドー不可修飾を付与し、同名再定義時は compile error にする。

2. エントリ解決の厳密化
- entry 関数が「名前解決済み」かつ「codegen 対象として生成済み」であることを検証する。
- `_start` 欠落を実行時エラーではなく compile error で検出する。

3. functions 系テストの仕様整合
- 関数値呼び出し (`func val`) の `_unknown` フォールバックを廃止し、WASM table + `call_indirect` で非キャプチャ高階関数を動作させる。
- 関数リテラル系ケースは、non-capture 先行（table + `call_indirect`）で段階導入し、capture ありは closure conversion の設計後に実装する。

4. sort/generics 連携不具合の調査
- `tests/sort.n.md` を起点に、`stdlib/alloc/sort.nepl` の move-check 失敗（`use of moved value`）を解消する。
- `sort_*` API (`(Vec<T>)->()`) と move 規則の整合を見直し、必要なら API/実装/テストを一貫して再設計する。
- `stdlib/alloc/sort.nepl` がジェネリクス（`<.T: Ord>`）で動作する経路を `tests/generics.n.md` と合わせて確認する。
- `Vec` の読み取り API（`vec_get` など）の所有権設計を見直し、反復参照で move-check に詰まらない read-only 経路を追加する（`vec_len`/`vec_data_ptr`/slice 風 API を含む再設計）。

4.5. StringBuilder の根本再設計（高階関数対応の次フェーズ）
- `stdlib/alloc/string.nepl` の `StringBuilder` を Rust/Go の方式を参考に再実装する（連結反復ではなく、可変バッファに append して最後に 1 回だけ str を構築）。
- `sb_append` / `sb_append_i32` / `sb_build` の計算量を見直し、build が O(n) になる設計へ変更する。
- 現行で「処理が終わらない」再現ケースを `tests/string.n.md` に追加し、再実装後に回帰テストとして固定する。

5. tests 全体の再分類と上流優先解消
- `node nodesrc/tests.js -i tests -o ...` の結果を stage 別に管理する。
- parser 起因の失敗群（stack/indent/unexpected token）を先に潰し、次に typecheck/codegen を進める。
- 最新分類（2026-02-10）: `total=413, passed=404, failed=9`

6. ドキュメント運用
- 実装進捗・結果・失敗分析は `note.md` のみに記録する。
- `todo.md` は未完了タスクのみを保持し、完了項目は即時削除する。

7. VSCode/LSP API 拡張（phase 2）
- `analyze_name_resolution` を拡張し、`import` / `alias` / `use` を跨いだ同名識別子の解決結果（候補一覧・最終選択・定義位置）を返す。
- token 単位の型情報 API（`token -> inferred type / expression range / argument range`）の既存出力へ、import 先を含む定義ジャンプ情報を統合する。
- 定義ジャンプ API を同一ファイルだけでなく import 先ファイルまで解決できる形で実装する。
- Hover/Inlay Hint 用に「式範囲」「引数範囲」「推論型」「関連ドキュメントコメント」を返す API を追加する。

8. 高階関数実装後のコンパイラエラー再整理
- エラーをテーブルで一元管理する（短い数値ID + 詳細メッセージ本文）。
- 診断生成側は原則 `ErrorId` を返し、表示層で `id -> 本文` を解決する構造に整理する。
- 既存エラー文言の重複を統合し、同一原因に同一IDを付与する。
- LSP/API からは `id` と展開済み本文の両方を取得できるようにする。

9. 高階関数対応完了後の Web Playground 改良
- VSCode 拡張機能で提供予定の情報（名前解決、型情報、式範囲、引数範囲、定義ジャンプ候補）を Playground 上でも表示できる UI/API を追加する。
- `web/tests.html` でテスト詳細展開時に、該当ソースと解析 API の詳細（AST/resolve/semantics）を併記できるようにする。

10. `examples/nm.nepl` / `stdlib/nm/parser` のデバッグ
- `examples/nm.nepl` を起点に、`stdlib/nm/parser` の不具合再現手順を固定する。
- `nodesrc/analyze_source.js --stage lex|parse` とコンパイラ診断を併用し、lexer/parser/typecheck のどこで崩れているかを切り分ける。
- 既存修正で自然治癒しているかを再検証し、未解決なら最小再現テストを `tests/` に追加してから根本修正する。
- `tests/nm.n.md` を基準に、`nm/parser` の Vec/str 所有権処理を data/len 直接アクセスへ置換して move-check を根本解消する。
- `nm/parser` の型名衝突（構造体名と enum variant 名）を整理し、名前解決の曖昧さを排除する。
- `examples/nm.nepl` は `std/env/cliarg` を使った引数処理で `--ast`/`--html` の双方を回帰テスト化する。

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

NASM target, LLVM IR target, C targetの追加
stdlib/coreとstdlib/allocはNASMとLLVMとCとWASMの全部に対応させる
stdlib/stdはNASMとLLVMとCとWASM&WASIP1の全部に対応させる
WASIp2やWASIXが必要な機能はstdlib/platformsで扱う
また、今後のtarget追加があった時に柔軟に対応できるような設計とする

targetのエイリアスの追加

coreはnasm|llvm|c|wasm
stdはnasm|llvm|c|(wasm&wasip1)
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
