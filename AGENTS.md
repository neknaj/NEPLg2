エラーは間に合わせの修正ではなく、原因を特定し、根本から修正すること  

# plan.md
plan.mdに作りたいものの説明が載っているので必ず確認してください  
人が書き換えるので変更しないでください  
変更が必要だと考える場合はnote.mdに記述してください  
# note.md
note.mdは自由に書き換えて構いません  
現在の実装状況、plan.mdと実装との差異などについてかならず記述し、更新してください  

# doc
doc/に適宜ドキュメントを作成してください  
これは自由に書き換えて構いません  
docは全体として一貫性のある書き方となるようにしてください  

# 開発方法
常にPlanを作成して計画的に進めること  
変更内容に関するコメントは付けないこと 「ここから変更」などのコメントは禁止 変更場所を説明するときはソースコードの外で、「○○を○○に変更しました」とテキストで説明すること  
コメント内容はコードのより抽象的な説明を行うこと 処理の流れや処理の目的が分かるように書くこと ドキュメントコメントを活用すること ある処理を変更した時には、それに伴って変更するべきコメントがないかよく確認すること  
不必要な変更を加えないこと エラーログなどがない限り適切に動作しているのだから、変更が必要な部分以外に変更を加えないこと 勝手に既存の機能を削除しないこと 改行やインデントを含むコーディングスタイルなども勝手に変えないこと  
実装を変更したとき、README.mdや/docを更新する必要がないか良く確認し、必要があれば更新すること  
エッジケースを想定しながら適切なテストを書くこと  
大きくなってきたファイルは適宜適切な形に分割すること  
ファイル名やファイル分割、クラス名や関数名など、適宜リファクタリングを行うこと  

# git commit
AGENTとして開発しているとき、git commitは適宜行ってください  
commitする前に、テストの通過を確認しなさい  

# テスト
trunk buildした後nodesrc/cli.jsのテストを実行し、outputのjsonを確認すること

## NEPL stdlib

stdlib/std/ に stdlib 各ファイルがあります

### コメントの付け方
NEPL stdlib には日本語でコメントを付けます
stdlib/nmでサポートする拡張markdownの形式で書きます
ファイルの先頭に、これは何に関する機能を集めたライブラリなのかを書きます
各関数の手前に、これは何をする関数で、どのようなアルゴリズムで実装されているかの説明を書きます
使用する上での注意点、計算量や制約や重要な仕様など、も丁寧に記述します

(例)
```neplg2
//: unwrap_err: Err の[中身/なかみ]を[取/と]り[出/だ]す（Ok なら[到達/とうたつ][不能/ふのう]）
//:
//: [目的/もくてき]:
//: - r が Err(e) なら e を[返/かえ]します。
//: - r が Ok(v) なら unreachable により「異常終了」します。
//:
//:
//: neplg2:test
//: ```neplg2
//:| #import "std/test" as *
//:| #import "core/result" as *
//: let r Result::Err "oops";
//: assert_str_eq "oops" unwrap_err r;
//: ```
//:
//: neplg2:test[should_panic]
//: ```neplg2
//:| #import "std/test" as *
//:| #import "core/result" as *
//: // Ok を渡すと unreachable が呼ばれ、落ちることを期待
//: let r Result::Ok 123;
//: unwrap_err r;
//: ```
//:
//: neplg2:test[compile_fail]
//: ```neplg2
//:| #import "std/test" as *
//:| #import "core/result" as *
//: // E が i32 の Result に文字列 Err を入れているので型エラーを期待
//: let r <Result<i32, i32>> Result::Err "text";
//: unwrap_err r;
//: ```
fn unwrap_err <.T, .E> <(Result<.T, .E>)->.E> (r):
    match r:
        Result::Ok v:
            #intrinsic "unreachable" <> ()
        Result::Err e:
            e
```

## NEPLg2 Compiler

/stdlib/neplg2/につくるセルフホストのコンパイラです
CLIはWASIです
CoreはWASI無しのただのWASMにします

/nepl-cli/src/main.rsではstdを使っているけど、/nepl-core/*.rsではno_stdなのと同じような感じでいきます

### コメントの付け方

NEPLg2 Compiler には日本語でコメントを付けます