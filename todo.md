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

2. エントリ解決の厳密化
- entry 関数が「名前解決済み」かつ「codegen 対象として生成済み」であることを検証する。
- `_start` 欠落を実行時エラーではなく compile error で検出する。

3. functions 系テストの仕様整合
- `let` 関数糖衣（型注釈あり/なし）を仕様どおりに通す。
- `@` 関数参照と alias の組み合わせを安定化する。
- 関数リテラル系ケースは、non-capture 先行（table + `call_indirect`）で段階導入し、capture ありは closure conversion の設計後に実装する。

4. tests 全体の再分類と上流優先解消
- `node nodesrc/tests.js -i tests -o ...` の結果を stage 別に管理する。
- parser 起因の失敗群（stack/indent/unexpected token）を先に潰し、次に typecheck/codegen を進める。

5. ドキュメント運用
- 実装進捗・結果・失敗分析は `note.md` のみに記録する。
- `todo.md` は未完了タスクのみを保持し、完了項目は即時削除する。

6. VSCode/LSP API 拡張（phase 2）
- `analyze_name_resolution` を拡張し、`import` / `alias` / `use` を跨いだ同名識別子の解決結果（候補一覧・最終選択・定義位置）を返す。
- token 単位の型情報 API（`token -> inferred type / expression range / argument range`）を typecheck 後段で追加する。
- 定義ジャンプ API を同一ファイルだけでなく import 先ファイルまで解決できる形で実装する。
- Hover/Inlay Hint 用に「式範囲」「引数範囲」「推論型」「関連ドキュメントコメント」を返す API を追加する。

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
