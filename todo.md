2026-02-22 今後の実装計画（未完了のみ）

方針
- plan.md の仕様を唯一の基準として、上流（lexer/parser）から順に修正する。
- 間に合わせ修正を避け、原因が同一の失敗はまとめて解消する。
- 実装進捗・結果・失敗分析は `note.md` に記録し、`todo.md` は未完了のみを保持する。

1. 名前解決・名前空間の再設計（最優先）
- `typecheck` の環境を `ValueNs`（変数）と `CallableNs`（関数/alias）に分離する。
- `let`/`fn` の巻き上げを plan.md 準拠で統一する（`mut` なし `let` と `fn` のみ）。
- nested `fn`/`let` を呼び出し可能にする経路を確立する（`tests/functions.n.md`）。
- 関数値（`@fn` / 関数を値として渡すケース）を HIR で明示表現し、解決規則を固定する。

2. 高階関数・call_indirect
- 関数値呼び出し (`func val`) の `_unknown` フォールバックを廃止する。
- WASM table + `call_indirect` で non-capture 高階関数を動作させる。
- capture あり関数値は closure conversion の設計を確定して段階導入する。

3. シャドーイング運用の完成
- オーバーロードとシャドーの判定を最終仕様で固定する（同一シグネチャのみ shadow warning）。
- 重要 stdlib 記号に対する warning 抑制ルール（設定/フラグ）を設計する。
- `noshadow` の適用範囲を stdlib で段階拡大し、運用ルールを文書化する。

4. sort/generics と Vec 読み取り設計
- `tests/sort.n.md` を起点に `stdlib/alloc/sort.nepl` の move-check 問題を根本解消する。
- `sort_*` API と move 規則の整合を見直し、必要なら API/実装/テストを再設計する。
- `Vec` の read-only 経路（`vec_len`/`vec_data_ptr`/slice 風 API）を再設計する。

5. LSP/API 拡張（phase 2）
- `analyze_name_resolution` を拡張し、import/alias/use 跨ぎの解決候補と最終選択を返す。
- token 単位の型情報 API に定義ジャンプ情報（import 先含む）を統合する。
- Hover/Inlay Hint 向けに式範囲・引数範囲・推論型・関連 doc comment を返す API を追加する。

6. 診断体系の再整理
- エラーをテーブルで一元管理する（短い数値ID + 詳細メッセージ）。
- 診断生成側は `ErrorId` を返し、表示層で `id -> 本文` を解決する構造に整理する。
- LSP/API から `id` と展開済み本文の両方を取得できるようにする。

7. Web Playground / tests.html 強化
- VSCode 拡張予定の情報（名前解決/型情報/式範囲/定義ジャンプ候補）を Playground で表示する。
- `web/tests.html` の詳細展開時にソースと解析結果（AST/resolve/semantics）を併記する。

8. `examples/js_interpreter` 実装（言語仕様固定後）
- `examples/js_interpreter` に JavaScript インタプリタを実装する。
- 言語仕様は変更せず、stdlib の再設計・改良のみで不足を埋める。
- Node.js 実行結果との同値性回帰テストを追加する。

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
