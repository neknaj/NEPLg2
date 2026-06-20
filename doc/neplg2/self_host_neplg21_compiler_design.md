# NEPLg2.1 セルフホストコンパイラ設計

最終更新: 2026-06-14

## 位置づけ

この文書は、`stdlib/neplg2/` に実装する NEPLg2.1 セルフホストコンパイラの設計を定義する。

既存の `doc/neplg2/self_host_plan.md` は NEPLg2.0 時点の計画であり、現在の NEPLg2.1 表層構文、`void` marker、`#test`、compiler artifact、Resource IR 静的検査、compile-time performance 改良を十分に反映していない。したがって、今後の `stdlib/neplg2/` 実装はこの文書を正規の設計入口とする。

この設計は NEPLg3 仕様の実装ではない。NEPLg3 文書は今後も変更され得る参考資料であり、NEPLg2.1 セルフホストコンパイラは現行 Rust 実装の NEPLg2.1 を基準にする。

`doc/self_host.md` は NEPLg2.1 と NEPLg3 の総合入口である。NEPLg2.1 の詳細はこの文書、NEPLg3 の詳細は `doc/neplg3/` 以下の文書で扱う。

## 設計原則

セルフホストコンパイラは、Zenn の開発方針で示された試作段階の原則を満たす。特に、次を設計上の制約として扱う。

- 方針確認の authority は `https://zenn.dev/bem130/articles/1b352797de94e7` である。新しい issue、実装 slice、または設計変更に着手する前に、この記事を再確認する。
- 互換性維持より、正しい仕様境界と根本原因の修正を優先する。
- core は platform 非依存に保ち、filesystem、stdio、argv、環境変数、時計、乱数などは CLI / host boundary に閉じ込める。
- 静的検査を省略して高速化したように見せない。高速化は探索範囲、依存関係、cache key、事前検査済み artifact の設計で行う。
- 失敗は `Option` / `Result` / enum diagnostic として表現し、string 判定や sentinel 値へ依存しない。
- public API に型名付き重複関数を増やさず、型注釈、期待型、trait / generic 解決で選択する。
- prototype 段階では大きな再設計を許すが、後続実装へ隠れた技術的負債を渡さない。

## stage 進行と性能最適化の分類

セルフホストコンパイラの実装では、性能問題をすべて同じ優先度で扱わない。現在の stage で設計を誤ると後から取り返しがつかない最適化と、公開 contract を保ったまま後から差し替えられる最適化を分ける。

今の stage で固定するべき最適化は、後続の型、authority、artifact key、proof boundary、module DAG、public API、diagnostic code、owner / borrow / drop の意味論に影響するものである。これらは実装後に直すと、既存 artifact、source policy、issue、doctest、selfhost module 間の依存関係を広く壊す。したがって、多少時間がかかっても先に正しい境界を作る。

具体的には、次は今やっておく必要がある。

- source text、span、display name、diagnostic message、path suffix を semantic authority にしないこと。
- parser、type resolver、checker、HIR、Resource IR、backend、artifact reader / writer の authority を混ぜないこと。
- `TypeId` や arena-local id を永続 artifact key にせず、stable canonical key / fingerprint / public surface hash へ分けること。
- lookup miss を proof success に畳まないこと。complete witness、missing、unknown、duplicate、unsupported を typed enum で区別すること。
- `Result` / `Option` / enum error、owner recovery、drop / borrow / no-escape proof、private effect masking の境界を先に固定すること。
- cache を使う場合でも、cache key と invalidation boundary に必要な入力を先に型として固定すること。
- 後続 module から見える public data shape、facade boundary、source policy contract、doctest の成功 / fail-closed 条件を曖昧にしないこと。

一方で、公開 contract と authority を保ったまま内部実装だけを置き換えられる最適化は、一定の時間超過を許容して次の stage を進める。これは性能問題を無視するという意味ではない。現 stage では「後から置換できる形に閉じる」ことを完了条件にし、実測で支配的になった時点で独立 issue として扱う。

後からできる最適化の例は次である。

- `Vec` scan を sorted index、hash index、bucket table、merge cursor に置き換えること。
- O(n) lookup を module-local cache や same-session memo table に置き換えること。
- stage0 smoke の入力を分割して compile time を短くすること。ただし検査責務は削らない。
- public summary contract を変えずに nested traversal の重複計算を memoize すること。
- artifact reader / writer の linear validation を index-assisted validation に置き換えること。
- 同じ typed input / typed output を保ったまま、allocation 回数や temporary owner を減らすこと。

ただし、後からできる最適化として扱うには条件がある。現在の実装が安全性検査を省略していないこと、探索範囲と計算量を doc comment または issue に記録していること、置換時に守る contract が source policy で固定されていること、実測上の時間超過が stage 進行を妨げるほどではないことを確認する。

この分類により、compile time が一時的に長い module があっても、原因が「内部 table がまだ O(n) である」「stage0 fixture が大きい」「後で index 化できる linear validation である」なら、semantic stage を止めない。逆に、原因が「型証拠の不足を fallback success にしている」「cache key が不完全」「source spelling を authority にしている」「Resource proof を省いている」なら、速度に関係なくその stage で修正する。

## Zenn 方針 review gate

セルフホストコンパイラの実装は、各 slice ごとに Zenn 方針 review gate を通す。ここでの slice は、issue の 1 checkpoint、公開 API の追加、型や診断 enum の追加、Resource proof 境界の追加、または既存 module の責務変更を指す。

review gate は次の順で行う。

1. 作業開始時に Zenn 記事を再確認し、その slice で影響する方針を `note.n.md` または対応 issue に記録する。
2. 独立した subagent に、設計、実装、ドキュメントコメント、source policy、検証方針をレビューさせる。
3. Blocker は同じ slice 内で修正する。修正できないものは、原因、影響、完了条件を持つ issue として分離する。
4. Non-blocker は `note.n.md`、`todo.md`、または対応 issue に残し、次 slice の入口で再確認する。
5. commit 前に、今回の差分が Zenn 方針 review gate を通ったことを `note.n.md` の checkpoint に記録する。

subagent review は、単に承認を得るためではなく、実装者と独立した観点で次の観点を確認するために使う。

- 詳細な review 入力、確認項目、指摘分類、`note.n.md` 記録形式は `doc/neplg2/self_host_zenn_review_checklist.md` を正とする。
- review 依頼は `nodesrc/selfhost_zenn_review_packet.js` で生成した packet を土台にし、branch、base / head、committed / staged / unstaged / untracked に分けた変更 file list、accepted / fail-closed、検証欄、既存 warning、今回差分由来 warning、Zenn 記事 URL、`YYYY-MM-DD` または ISO-like date-time の Zenn 再確認日時、`AGENTS.md`、checklist、prompt authority を省かない。
- review response は `nodesrc/selfhost_zenn_review_response_check.js` で検査し、必須 section / field が欠けた返答、`policy/spec` と `implementation/test` のどちらかを欠いた返答、または弱い approve だけの返答を review 記録として扱わない。
- 失敗経路が `Result` / `Option` / enum error で表現され、表示文字列や sentinel 値で分岐していないこと。
- `match` の網羅性が効く形で variant が設計され、数値 tag や文字列 tag を公開 API にしていないこと。
- parser、checker、HIR、Resource IR、backend の authority が混ざっていないこと。
- pure core と host / CLI boundary が分離され、filesystem、stdio、clock、random などが core に漏れていないこと。
- 高速化が安全性検査の省略ではなく、探索範囲、依存関係、事前検査済み artifact、cache key の設計で行われていること。
- ドキュメントコメントが、目的、契約、戻り値や error variant の条件、計算量、制約、典型例、現状の実装詳細と将来も守る契約の区別を説明していること。
- コメントの量を減らすための行数制限や説明削減の検査が入っていないこと。コメントの丁寧さと契約の明確さを優先する。

source policy regression は、この review gate を支えるために追加する。検査は「コメントが多いこと」を拒否してはならない。検査すべきなのは、方針を守るための構造である。例えば、typed error enum、owner recovery、pure / impure boundary、module split、facade re-export、doc comment の必須項目、review checkpoint の記録を確認する。一方で、実装行数やドキュメントコメントの長さそのものを制限してはならない。

## NEPLg2.1 構文の前提

セルフホストコンパイラが受理する現在の正規構文は NEPLg2.1 である。

```neplg2
fn main %fn void unit \void:
    sub unit

fn sub %fn unit unit \a:
    a
```

構文上の重要な前提は次の通りである。

- 型注釈は `%T expr` であり、旧 `<T>` 注釈は正規構文ではない。
- 型式は prefix 表記であり、`Vec i32`、`Result i32 str`、`fn i32 fn i32 i32` のように書く。
- 関数型はカリー化された見た目を持つが、部分適用を導入しない。内部では必要に応じて多引数関数型へ正規化する。
- 0 引数関数は `fn void T` と `\void` で表す。
- `void` は型でも値でもなく、0 引数関数 marker だけである。`TypeExpr::Void`、HIR の void 値、Resource IR の void 値、runtime void は作らない。
- `unit` は unit 型かつ unit 値である。`fn unit T` は unit 値を 1 つ受け取る関数型である。
- NEPLg2.1 の正規セルフホスト source では、括弧 `(` `)` を grouping として使わない。現行 Rust parser が互換・回復用に受理する箇所は、self-host 側では移行診断または明示的な parity difference として扱う。
- generic postfix は正規構文ではない。`size_of %T` / `align_of %T` のような postfix-free layout query を正規 API とし、zero-argument user generic call の型証拠が不足する場合は type resolver の未解決 task として扱う。
- `#test` は test mode でのみ直後 1 statement を有効化する directive であり、通常 compile の public/runtime artifact に混ぜない。

旧構文を診断または移行支援として読むことはできるが、セルフホストコンパイラの内部設計は旧構文を authority にしない。

## Rust 実装から引き継ぐ境界

現行 Rust コンパイラは、概ね次の順で処理する。

```text
source / loader
    -> lexer / parser
    -> target/profile precheck
    -> typecheck
    -> .neplmeta public interface
    -> Resource IR 用 monomorphize
    -> entry reachable pruning
    -> Resource IR static check
    -> drop elaboration / drop insertion
    -> codegen 用 monomorphize
    -> Wasm / LLVM backend
```

セルフホストコンパイラでも、この authority 境界を維持する。

| 段階 | authority |
|---|---|
| loader | path resolution、prelude、import/include、module graph、SourceMap、public surface hash |
| parser | token / offside / directive / flat prefix expression / flat prefix type |
| type resolver | type constructor arity、kind、prefix type application boundary |
| type checker | overload、generic、trait、expected type、prefix call reduction、surface effect |
| HIR | typed source-level IR、drop insertion 前の backend 入力 |
| Resource IR | ownership、borrow、initialized state、drop plan、internal effect、proof summary |
| monomorphize | concrete function instance、trait call resolution、runtime helper reachability |
| backend | target-specific Wasm / LLVM emission |

`--check` 相当の処理も typecheck-only にしない。Rust 実装では `--check` が Resource IR static check と proof boundary まで通すため、セルフホスト側でも「安全性を確認した check」として同じ意味にする。

## ディレクトリ構造

`stdlib/neplg2/` は、compiler core と CLI boundary を明確に分ける。

```text
stdlib/neplg2/
    index.n.md
    README.md
    core/
        pipeline/
        options.nepl
        infra/
        syntax/
        module/
        resolve/
        ty/
        abstraction/
        check/
        hir/
        resource/
        proof/
        effect/
        memo/
        artifact/
        cache/
        incremental/
        mono/
        codegen/
        builtins/
    cli/
        args/
        driver.nepl
        file_io.nepl
        reporter/
```

`core/` は pure compiler core である。source text、module graph、diagnostic、typed HIR、Resource IR、artifact value を入力と出力として受け渡し、filesystem や stdio を直接扱わない。

`cli/` は WASI / host 依存の境界である。argv parsing、file read/write、stdout/stderr、exit code、artifact persistence はここに閉じ込める。

root module は facade と orchestration に留める。Rust 実装の大きな flat module をそのまま移植せず、proof、query、cache、diagnostic、backend の責務を小さい module に分ける。

既存の `core/pipeline.nepl`、`cli/args.nepl`、`cli/reporter.nepl`、`cli/driver.nepl` などの facade / import path は、可能な限り維持する。上の構造は「現在ある skeleton を壊すための置換案」ではなく、既存 facade を残したまま不足 module を追加し、古い NEPLg2.0 前提のコメントと責務だけを NEPLg2.1 設計へ寄せるための目標構造である。

## Public API

core の主要 API は値として設計する。

```text
SelfhostCoreCompileRequest:
    root_module
    vfs_snapshot
    artifact_snapshot
    session_cache_snapshot
    options

SelfhostCompileOptions:
    target
    profile
    test_mode
    entry
    no_prelude
    emit_set
    diagnostics_format
    cache_policy

SelfhostCoreCompileResult:
    diagnostics
    check_summary
    neplmeta
    neplproof
    neplobj
    wasm
    wat_comments
    llvm_ir
    timings
```

`SelfhostCoreCompileRequest` は host path や filesystem handle を持たない。`root_module` は logical module id であり、`vfs_snapshot`、`artifact_snapshot`、`session_cache_snapshot` は pure value として core に渡す。

永続化、host path の正規化、stdlib root の探索、artifact file の read/write、cache directory の管理は `cli/` の責務である。CLI は host の入力を core の snapshot value へ変換し、core の結果を filesystem / stdout / stderr へ戻す adapter として振る舞う。

`test_mode` は `profile` と分ける。`#test` は test build の source selection に影響するため、source key、public surface hash、artifact header の key material に入れる。

## Source と diagnostic

source location は byte offset を authority とする。line / column は表示用に変換する。

`SelfhostSourceSpan` は次を満たす。

- file id は VFS / SourceMap の stable path に紐づく。
- start と end は非負で、`start <= end` を満たす。
- text length を超える span は構築時に拒否する。
- diagnostic は span を持てない場合も typed code と context を持つ。

diagnostic code は階層 enum として扱い、human readable text と JSON stable string は reporter boundary でだけ作る。compiler 内部で diagnostic message string を解析しない。

## Lexer

lexer は offside rule と directive を最初の authority として扱う。

必須 token は次を含む。

- identifier、integer、string、doc comment、line comment
- `#entry`、`#target`、`#indent`、`#import`、`#include`、`#no_prelude`、`#test`
- `%`、`\`、`:`、`|>`、`.`、`,`、`@`
- `unit`、`void`、`fn`、`impure`
- `Indent`、`Dedent`、`Newline`、`Eof`

`void` は lexer で reserved keyword として分類する。通常 identifier として束縛しない。

旧 angle type syntax や旧 grouping token は、移行診断のために token として保持してもよい。ただし、NEPLg2.1 parser の正規 grammar では使わない。

## Parser

parser は call tree を完全確定しない。NEPLg2 は前置記法であり、呼び出し境界は callable 候補、arity、expected type、generic 解決、trait 解決に依存するためである。

parser の出力は次の形を基本にする。

```text
ParsedModule:
    directives
    item_list

ParsedExpr:
    PrefixList items
    Block
    If
    Match
    While
    Lambda
    Let

ParsedType:
    TypePrefixList items
    FunctionMarker
    TypeAscription
```

`%T expr` は expected type boundary として保持する。runtime operation ではない。

現行 self-host 実装では、`stdlib/neplg2/core/syntax/ast/prefix_expr.nepl` が `SelfhostExprPrefixList` を提供する。これは `SelfhostSyntaxRange` から作る pre-HIR の flat expression item list であり、`%` type annotation marker、lambda marker、`@` marker、literal、identifier、control form marker を token index と span 付きで保持する。`SelfhostExprPrefixList` は HIR ではなく、`SelfhostHirExprPayload::Call` のような解決済み call tree を作らない。body parser は `module_parser/body_range.nepl` で declaration body envelope と first expression segment を `SelfhostSyntaxRange` として切り出す。`parser/body_segmenter.nepl` は body envelope を top-level segment 列へ分解し、flat prefix expression にできる `ExpressionLine` と nested body を持つ `BlockIntro` を型で分ける。`BlockIntro.body` は recursive segmenter の入力であり、`SelfhostExprPrefixList` へ直接渡さない。

`fn void T` は parser/type resolver boundary で `params = []` へ正規化する。`void` は result type、type argument、expression、parameter name として受理しない。

`fn unit T` は `params = [unit]` として扱う。旧構文の `fn unit T \unit` は、0 引数関数のつもりであれば `fn void T \void` を提示する diagnostic にする。

## Type Resolver

prefix type syntax では、parser だけで型式境界を決めない。

```neplg2
%Result i32 str expr
%fn i32 fn i32 i32 add
```

type resolver は type constructor table と kind / arity 情報を使い、flat な type prefix list を型木へ縮約する。

必要な入力は次である。

- built-in type constructor table
- imported public type constructor surface
- local type declaration headers
- trait / associated type headers
- generic type parameter environment

resolver は `TypeId` を永続 key にしない。arena-local な `SelfhostTypeId` は session 内の高速参照であり、artifact や cache key には stable canonical type text / structural key を使う。

現行 self-host 実装では、まず structural canonical key を `SelfhostCanonicalTypeKeyArena` として作る。key arena は primitive / named / applied / function の key node table と key argument table を所有し、key node payload には arena-local `SelfhostTypeId` を入れない。projection diagnostic だけは、失敗位置を説明するために入力側の `SelfhostTypeId` を保持してよい。

named type の key は当面 `SelfhostNamedTypeId` を使う。これは constructor table snapshot 内の identity であり、永続 artifact の完全な nominal identity ではない。`.neplmeta` interface artifact が module path、surface hash、type constructor stable identity を提供した段階で、canonical named key payload をその stable identity へ拡張する。

generic postfix は type resolver の正規入力にしない。旧 `f<T>` は migration diagnostic と source-to-source migration の対象であり、通常 compile の成功経路では `%T expr`、expected type、receiver type、argument type、trait bound から型証拠を集める。

layout query は postfix-free API として扱う。

```neplg2
size_of %i32
align_of %Vec i32
```

zero-argument user generic call は、argument や receiver から型証拠を得られないため、expected type または明示 `%T` ascription が必要である。証拠が不足する場合は、曖昧な generic call diagnostic を出し、旧 postfix を内部的に復活させない。

## Type Checker

type checker は prefix call reduction を担当する。

主な処理は次である。

- expected type propagation
- `%T expr` ascription
- overload candidate collection
- generic type variable creation
- trait bound solving
- callable arity / effect / parameter matching
- indirect function value call
- no partial application enforcement
- `@ident` function value identity construction
- `memo_call` compiler-known primitive

現行 self-host 実装では、`stdlib/neplg2/core/check/expr/` が expression checker の初期境界を持つ。`SelfhostTypeExpectation` は expected type の `SelfhostTypeId` だけでなく、`ExplicitAscription` / `BlockResult` / `BlockSequenceDiscardedExpression` / `OuterConsumerArgument` の由来と span を保持する。`SelfhostCallableCandidate` は候補名、function type であるべき TypeId、effect、generic inference state、span を保持する。call reduction はこれらと `SelfhostExprPrefixList` を受け取り、HIR をまだ生成せず `SelfhostCallReduceResult::DirectCall` または `SelfhostCallReduceError` を返す。`check/expr/body_line.nepl` は `SelfhostBodySegmentKind::ExpressionLine.head` から prefix list を作ってこの境界へ渡す接続口である。`check/expr/candidate_collection.nepl` は prefix head の identifier spelling を token span から復元し、`SelfhostNameScope` の function namespace と `SelfhostCallableSignatureTable` の DefId-linked signature evidence から reducer 用 candidate vector を作る。通常の expression line 入口に渡された `BlockIntro` は nested body envelope なので `NotExpressionLine` として拒否し、prefix list 生成失敗と call reduction 失敗は別の enum variant に保つ。BlockIntro 専用入口では `head` だけを prefix list 化し、`body` は `SelfhostTrailingBlockArgument` として source-backed reducer へ渡す。`check/expr/block_body.nepl` はこの nested body envelope を body segmenter で再帰分解し、単一 `ExpressionLine` の prefix list と callable candidate list、または複数 top-level segment の segment list を構築する。call reducer 側の segment dispatcher は `ExpressionLine` を通常 prefix expression として検査し、nested `BlockIntro` は再び `head` だけを prefix list 化して `body` を trailing block evidence として渡す。単一 body は不足 parameter の expected type を `BlockResult` expectation として nested expression へ渡し、複数式 body は最後の expression だけを `BlockResult`、非末尾 expression を `BlockSequenceDiscardedExpression` 由来の `unit` expectation として検査する。成功時は末尾 block を `BlockResult` または `BlockSequence` checked tree node として 1 実引数式にする。空 body、prefix build failure、candidate collection failure、非末尾 expression 用の `unit` 型欠落は `TrailingBlockBody*` error として fail-closed にし、余分な block は `UnexpectedTrailingBlockArgument` として拒否する。legacy single-input API では nested `BlockIntro` を引き続き `NestedBlockUnsupported` として拒否し、通常の成功経路は segment dispatcher を通る。

2026-06-05 checkpoint では、`%T expr` ascription を `SelfhostTypeExpectation::ExplicitAscription` へ変換する初期接続を追加した。`resolve/type_resolver/reduce.nepl` は flat type prefix list の先頭 1 type expression だけを reduce し、`SelfhostTypePrefixReducePrefixResult.next_index` によって expression tail の開始位置を caller へ返す。`check/expr/ascription.nepl` は `%` の直後から type prefix candidate を集め、type tree を `SelfhostTypeArena` へ projection し、残りの token range を inner expression として返す。`check/expr/body_line.nepl` は先頭 token が `%` の場合だけこの ascription projection を使い、`%` 自体を call head として call reducer へ渡さない。

2026-06-05 candidate collection checkpoint では、line head に対する最初の名前解決境界を追加した。`SelfhostCallableSignatureTable` は現在 `Vec` による insertion-order table で、DefId lookup は O(s) だが、public API は表現を隠している。これにより後続の `.neplmeta` interface artifact や prechecked signature index へ差し替えるとき、call reducer や parser の契約を変えずに lookup 実装だけを置き換えられる。名前が見つからない場合は空 candidate list を返し、`UnresolvedName` diagnostic は call reduction 側に集約する。同名 function overload は最新の可視 binding が function である場合だけ候補列へ集め、後から local / parameter が同名で追加された場合は古い function を復活させない。binding があるのに DefId や signature が欠ける場合は `PendingBinding` / `MissingSignature` として fail-closed にする。

2026-06-05 argument type evidence checkpoint では、`check/expr/argument.nepl` を追加し、prefix item 1 個で型が一意に決まる literal argument だけを function parameter type と照合する境界を作った。`UnitValue`、`IntLiteral`、`FloatLiteral`、`BoolLiteral`、`CharLiteral`、`StringLiteral` はそれぞれ `unit`、`i32`、`f32`、`bool`、`char`、`str` の証拠を返す。`FloatLiteral` は Rust 実装の現 checkpoint と同じく `f32` 固定で扱い、`f64` defaulting や numeric suffix は後続の numeric literal 設計に残す。source-less borrowed 境界では、`NamedValue`、nested call、block、lambda、`@ident`、borrow、pipe、ascription 付き argument は full expression checker または source / token backed owner 境界が expected parameter type を使って縮約できるまで fail-closed にする。これは完全な argument expression checker ではなく、arity と expected result だけで `add true 1` のような direct call を通してしまう経路を閉じる初期 boundary である。

2026-06-05 argument expression cursor checkpoint では、call reducer が `item_count - 1` を実引数数として扱う旧方式をやめ、parameter index と prefix item cursor を分けた consume-width 走査へ移した。`SelfhostExprArgumentMatch.next_index` は 1 実引数式を消費した後の prefix item index を返す。この source-less borrowed API で成功するのは単一 literal item だけである。`%T literal` は言語仕様上 1 つの argument expression だが、`SelfhostExprPrefixList` は item kind、token index、span だけを持ち、`T` の source spelling や token buffer を持たないため、parameter expected type と明示 ascription type の一致を安全に検査できない。このため borrowed API の `%T literal` は `UnsupportedArgumentExpression` として拒否し、source / token backed owner API だけで解決する。重要なのは、`add %i32 1 2` のような入力を raw 4 argument と誤分類しないことである。

2026-06-05 source-backed argument-scope ascription checkpoint では、`SelfhostExprAscriptionHeadProjection`、`SelfhostExprArgumentOwnedMatch`、`SelfhostCallReduceOwnedResult`、`selfhost_call_reduce_prefix_with_source` を追加し、source と token buffer を持つ入口で `%T literal` を 1 つの argument expression として検査できるようにした。`%T literal` は ascription head から explicit expectation と literal tail の first token を投影し、argument cursor は type marker、type expression、literal tail をまとめて消費する。projection 後の `SelfhostTypeArena` は owner payload として caller へ返されるため、明示 ascription type と parameter expected type は arena-local `SelfhostTypeId` のまま構造一致を確認できる。`add %i32 1 2` は 2 引数 direct call として受理し、`add %bool 1 2` は `ArgumentAscriptionExpectedTypeConflict` として拒否する。`%T` head の projection 自体が失敗した場合は `ArgumentAscriptionProjectionFailed` として span を保持し、empty-span の unsupported error へ潰さない。nested call、block、lambda、borrow、pipe などは、引き続き full expression checker の残件として fail-closed にする。

2026-06-05 named value argument evidence checkpoint では、`check/expr/value_evidence.nepl` を追加し、argument position の `NamedValue` と `%T NamedValue` を DefId-linked な型証拠で検査するようにした。`SelfhostValueTypeEvidenceTable` は `SelfhostNameScope` の binding が持つ `SelfhostDefId` と `SelfhostTypeArena` 内の `SelfhostTypeId` を結び、source spelling から型を推測しない。source-backed argument checker は `name -> latest binding -> DefId -> value type evidence` の順に証拠を取得し、最新 binding が `Local` / `Param` / `Function` / `Builtin` 以外なら古い同名値へ戻らず `NamedValueUnsupportedBinding` として拒否する。binding が存在しない場合、DefId 未割当の場合、型証拠が未登録の場合もそれぞれ `NamedValueUnresolved` / `NamedValuePendingBinding` / `NamedValueEvidenceMissing` として分け、call reducer は `ArgumentNamedValue*` error へ投影する。これにより `add x 2` と `add %i32 x 2` は `x: i32` の証拠が登録済みの場合だけ受理し、binding だけでは成功しない。

2026-06-05 nested named call argument checkpoint では、source-backed call reducer が argument position の `NamedValue` を value evidence だけでなく callable candidate としても解釈できるようにした。`candidate_collection.nepl` は prefix 全体の先頭だけでなく、任意の `SelfhostExprPrefixItem` head から DefId-linked callable signature list を集める入口を持つ。外側 call の parameter expected type は nested call の result expectation として渡され、`add add 1 2 3` では内側 `add 1 2` だけが第1引数として消費され、最後の `3` は外側 call の第2引数として残る。候補が 0 件の場合だけ従来の NamedValue value evidence へ fallback し、function 候補があるのに signature 不整合や overload ambiguity がある場合は `ArgumentNestedCandidate*` / `OverloadAmbiguous` として fail-closed にする。候補 0 件には、最新の可視 binding が local / parameter として古い同名 function を shadow している場合も含む。この場合は `add add 2` の2個目の `add` のように、通常の value evidence で処理する。source-less borrowed reducer は引き続き nested call を成功扱いせず、source / token / scope / value evidence / signature table を持つ owner 入口だけがこの可変幅引数式を処理する。

2026-06-05 block body result checkpoint では、`BlockIntro` を flat prefix item に混ぜず、source-backed owner 入口へ optional trailing block evidence として渡す境界に、単一 expression line の block result checker を接続した。`add 1:` のように head prefix の引数が不足し、残る parameter を nested body が満たす位置では、block body envelope を `selfhost_body_segment_list_from_envelope` で再帰的に分解する。body が単一 expression line なら、その head から prefix list と candidate list を構築し、outer parameter type を `BlockResult` expectation として nested expression の source-backed reducer へ渡す。`add 1:\n    add 1 1` のような direct call body は outer call の第2引数として消費できる。`add 1 2:` のように parameter を消費し終えた後に block が残る場合は `UnexpectedTrailingBlockArgument` として拒否する。この checkpoint では空 block、複数式 block、nested block を `TrailingBlockBody*` error として残した。

2026-06-06 block body sequence checkpoint では、複数 top-level `ExpressionLine` を持つ trailing block argument を `BlockSequence` checked tree node へ接続した。`SelfhostCheckedExprTree` は node table と argument table に加え、block child root id 専用の `exprs` table を所有する。これは node table の contiguous range を block child list とみなす設計を避けるためである。nested direct call や block result は内部 node を追加するので、node id の数値範囲は top-level body expression の列と一致しない。call reducer は各 segment を source order で同じ checked tree へ追加し、非末尾 expression は arena 内の `unit` type を `BlockSequenceDiscardedExpression` expectation として検査する。`unit` type は `SelfhostTypeId 0` のような fixture 前提ではなく `selfhost_type_arena_find_kind` で検索する。最後の expression は外側 parameter の expected type を `BlockResult` expectation として受ける。HIR lowering は `BlockSequence` range を順に lowering して HIR `Block` の child range に変換し、source token、scope lookup、callable candidate collection を再実行しない。lambda / borrow / pipe chain / ascribed pipe target / indirect call は引き続き後続 slice の fail-closed 残件である。

2026-06-06 nested BlockIntro checkpoint では、trailing block body の segment dispatcher を追加し、block body 内にさらに `BlockIntro` が現れる場合も `head` と `body` を分離したまま再帰的に source-backed reducer へ渡すようにした。`ExpressionLine` は従来どおり `selfhost_block_body_result_input_from_segment` で prefix list と candidate vector を作る。nested `BlockIntro` は `segment.head` だけを prefix list 化し、`segment.body` は `SelfhostTrailingBlockArgument` として `selfhost_call_reduce_prefix_with_source_in_tree` へ渡す。これにより `add 1:` の body が `add 1: add 1 1` のような nested block であっても、内側 block result が同じ `SelfhostCheckedExprTree` に保存され、外側 call の不足 parameter を満たせる。`BlockIntro.body` を flat prefix list に直接渡す退行は source policy で禁止し、stage1 smoke で `TokenKind::Colon` を含む nested fixture を通して確認する。

2026-06-06 source-backed pipe direct-call checkpoint では、`1 |> add 2` のような単一 pipe expression を source-backed reducer で `add 1 2` 相当の `DirectCall` checked tree へ正規化する境界を追加した。pipe scan は prefix list を左から 1 回だけ見る O(n) の前処理であり、複数 pipe chain は `PipeUnsupportedMultiple` として fail-closed にする。右辺 target は現 checkpoint では `NamedValue` だけを許可し、scope と callable signature table から DefId-linked candidate を収集する。左辺は target の第1 parameter expected type で 1 実引数式として検査し、checked argument list の0番目へ保存する。右辺 suffix は既存の source-backed argument loop に第2 parameter から渡し、checked argument list の後続へ保存する。`SelfhostCheckedArgumentKind::I32Literal` payload と prefix item range を stage1 smoke で確認し、HIR lowering が source token や scope lookup を再実行しない契約を保つ。source-less reducer は pipe を成功扱いせず named-head-only の fail-closed 境界として残す。ascription 付き pipe target、lambda target、generic target、borrow を含む左辺、pipe chain の結合規則は後続 slice に残す。

2026-06-06 pipe fail-closed smoke checkpoint では、上記の受理範囲だけでなく、未対応または不正な pipe 形が typed error として閉じることを stage1 の実行可能 smoke へ追加した。`1 |> add 2 |> add 3` は `PipeUnsupportedMultiple`、`|> add 1` は `PipeMissingLeftOperand`、`1 |>` は `PipeMissingRightTarget`、`1 |> 2` と `1 |> %i32 add 2` は `PipeRightTargetUnsupported`、`1 2 |> add 3` は `PipeLeftSegmentNotSingleValue`、`1 |> answer` かつ `answer : fn void i32` は `PipeTargetRequiresInput` として確認する。これらは helper が手作り error を返すのではなく、同じ source-backed body-line reducer を通して観測する。現段階では pipe chain や non-`NamedValue` target の意味論を広げず、探索範囲を O(n) pipe scan と RHS target 候補収集の既存境界に保つ。次に pipe chain や ascribed target を受理する場合は、ここで固定した error がどの仕様変更で success path へ移るのかを issue と doc で明示してから変更する。

2026-06-11 ascribed pipe target checkpoint では、`left |> %fn ... named_target suffix...` を source-backed pipe reducer の成功経路へ接続した。`%T` は suffix argument ではなく target callable 自体への型注釈であるため、RHS の prefix item 範囲を source token range へ戻し、`selfhost_expr_ascription_project_head_expectation` で型式境界と expression-first token を投影する。projection 成功時は `SelfhostTypeExpectation` を arena owner と一緒に受け取り、target item 復元後に selected callable type と同じ arena 内で構造一致を確認する。`%fn i32 fn i32 i32 add` のように一致する場合だけ `add left suffix...` 相当の `DirectCall` checked tree へ進み、suffix cursor は `target_index + 1` から始める。これにより `%fn ...` の型式 token を実引数として誤消費しない。projection 失敗は `PipeTargetAscriptionProjectionFailed`、callable type 不一致は `PipeTargetAscriptionTypeMismatch` として分け、通常の `PipeRightTargetUnsupported` や argument mismatch へ潰さない。projection 失敗時の arena owner は projector が閉じ、call reducer は checked tree owner だけを閉じる。

2026-06-11 ascribed pipe target overload narrowing checkpoint では、上記の target ascription を単一候補の事後照合だけではなく、同名 overload 候補の探索空間削減にも使うようにした。`selfhost_call_reduce_pipe_candidates_with_source` は `target_ascription = Some` の場合、`candidate_count > 1` を先に曖昧性として返さず、明示された function type と各 candidate の callable type を同じ arena 内で構造比較する。一致 0 件は target は見つかったが注釈型に合う候補がないため `PipeTargetAscriptionTypeMismatch`、一致 1 件はその candidate で direct call 正規化、同じ callable type に複数候補が一致する場合は `PipeTargetAmbiguous` とする。`target_ascription = None` の複数候補は従来通り `PipeTargetAmbiguous` であり、引数式を先読みする narrowing や generic inference / trait solving はこの slice では行わない。stage1 smoke は `add : fn i32 i32` と `add : fn i32 fn i32 i32` を同じ scope に置き、`1 |> %fn i32 fn i32 i32 add 2` が2引数版だけを選ぶことを確認する。さらに、注釈型に一致する candidate が 0 件のときは `PipeTargetAscriptionTypeMismatch`、同じ callable type に複数 candidate が一致するときは `PipeTargetAmbiguous` になることを、source-backed reducer を通す negative smoke として public fail-closed 群へ入れた。lambda target、generic target、borrow を含む左辺、pipe chain の結合規則、引数式を使う advanced pipe target narrowing は引き続き後続 slice に残す。

2026-06-11 pipe chain checked-tree checkpoint では、`left |> named_target suffix... |> named_target suffix...` を source-backed reducer の左結合 success path へ接続した。最初の pipe segment は既存の `left |> named_target suffix...` と同じ規則で checked `DirectCall` node を作り、2段目以降は前段 root id を `SelfhostCheckedArgumentKind::CheckedExpr` として次段 target の第1引数へ渡す。これにより、chain の中間結果を source token へ戻して再解釈せず、HIR lowering は checked tree node と checked argument list を authority として使える。各段の右辺 suffix は次の pipe operator より手前で切るため、`1 |> add 2 |> add 3` の最初の `add` は `2` だけを suffix として消費し、2個目の pipe を raw trailing item として誤消費しない。source-less reducer は引き続き pipe を受けず、chain 対応は source / token / scope / signature evidence を持つ入口だけに閉じる。stage1 smoke は、最終段の第1引数が `CheckedExpr [0, 4)`、第2引数が `I32Literal(3) [6, 7)` であることに加え、root `DirectCall` から前段 `DirectCall` node へ checked tree topology を辿って確認する。さらに `1 |> add 2 |>` は前段成功後に `PipeMissingRightTarget`、`1 |> add 2 |> 3` は `PipeRightTargetUnsupported` として後段 fail-closed になることを確認する。lambda target、generic target、borrow を含む左辺、引数式先読みを伴う advanced pipe target narrowing は引き続き後続 slice に残す。

2026-06-11 non-ascribed pipe target ProbeUnsupported smoke checkpoint では、注釈無し pipe target の候補選択で borrowed probe が判定できない suffix argument を不一致として扱わないことを実行可能に固定した。`1 |> add %i32 2` は source-backed argument checker なら `%i32 2` として検査できるが、候補選択段階では checked tree owner や source-backed arena owner を候補ごとに消費しない。stage1 smoke は `add : fn i32 i32` と `add : fn i32 fn i32 i32` を同じ scope に置き、前者を suffix 余剰の `NoMatch`、後者を ascription suffix の `ProbeUnsupported` として集計し、unsupported が 1 件でも残る候補列を `PipeTargetAmbiguous` で止めることを確認する。これは advanced suffix expression の成功実装ではなく、source-backed checker を候補ごとに投機実行して探索空間を広げる前に、fail-closed 境界を明確にする checkpoint である。

2026-06-11 single source-backed required suffix checkpoint では、上記の fail-closed 境界を一段進めた。ただし、`ProbeUnsupported` という1 variant では、`%i32 2` のように既存 source-backed checker で再検査できる suffix と、generic 未解決・内部 invariant・OOM のように候補選択へ使ってはいけない未対応を区別できない。そのため `SelfhostPipeCandidateApplicability` を `SourceBackedRequired` と `SelectionBlockedUnsupported` に分離した。`Match` が 0 件、`SourceBackedRequired` が 1 件、`SelectionBlockedUnsupported` が 0 件の場合に限って、その候補を既存の source-backed finisher へ渡す。これにより `1 |> add %i32 2` は、1引数版 `add` が suffix 余剰で `NoMatch` になり、2引数版 `add` だけが `%i32 2` の source-backed 検査を必要とする場合に、2引数 direct call として成功する。`Match` と `SourceBackedRequired` が混在する場合、`SourceBackedRequired` が複数残る場合、または `SelectionBlockedUnsupported` が1件でもある場合は、別候補や将来のgeneric solver結果を隠す可能性があるため引き続き `PipeTargetAmbiguous` とする。stage1 smoke は checked argument 0 番目が pipe 左辺、1 番目が `%i32 2` の内側 literal payload であることを確認し、HIR lowering が source token を再読しなくても suffix argument order を保てる契約を固定する。

2026-06-11 source-backed named suffix checkpoint では、同じ一意 source-backed finisher 境界を `1 |> add x` にも適用した。`x` は source-less borrowed probe では value evidence table に到達できないため `SourceBackedRequired` になるが、候補列全体では1引数版 `add` が suffix 余剰で `NoMatch`、2引数版 `add` だけが `SourceBackedRequired` になる。`Match` 0 件、`SourceBackedRequired` 1 件、`SelectionBlockedUnsupported` 0 件という同じ条件を満たすため、その1候補だけを source-backed finisher へ渡し、`SelfhostValueTypeEvidenceTable` の DefId-linked `x: i32` evidence から `SelfhostCheckedArgumentKind::NamedValue` payload を作る。stage1 smoke は左辺 `I32Literal(1) [0, 1)` と suffix `NamedValue [3, 4)` の順序を確認し、HIR lowering が source token や scope lookup を再実行しない契約を保つ。これは候補ごとに source-backed owner reducer を投機実行する変更ではなく、既に一意化された `SourceBackedRequired` 候補だけを既存 finisher へ渡す範囲の拡張である。

2026-06-11 source-backed nested suffix checkpoint では、同じ一意 source-backed finisher 境界を `1 |> add sum 2 3` に適用した。outer target の `add` は `fn i32 i32` と `fn i32 fn i32 i32` の同名 overload とし、inner head の `sum` は単一候補の `fn i32 fn i32 i32` として scope / callable signature table に登録する。borrowed probe では1引数版 `add` が suffix 余剰で `NoMatch`、2引数版 `add` だけが nested suffix のため `SourceBackedRequired` になる。`Match` 0 件、`SourceBackedRequired` 1 件、`SelectionBlockedUnsupported` 0 件の場合だけ既存 finisher へ渡し、source-backed argument checker が `sum 2 3` を `SelfhostCheckedArgumentKind::CheckedExpr [3, 6)` として outer call の第2引数へ保存する。stage1 smoke は outer argument order に加え、checked tree の child `DirectCall` が candidate name `sum`、`I32Literal(2) [4, 5)`、`I32Literal(3) [5, 6)` を持つことまで辿って確認する。これにより、HIR lowering は suffix source token、scope lookup、callable candidate collection を再実行しない。inner head も同名 overloaded callable になる場合は、この時点では別 slice に残した。

2026-06-11 source-backed same-name nested final-range checkpoint では、直前に残した `1 |> add add 2 3` を、source-backed nested argument が渡された範囲の末尾まで完全消費できる場合の共通境界として接続した。pipe 経路では outer target の `add` 選択は既存の `Match` 0 件、`SourceBackedRequired` 1 件、`SelectionBlockedUnsupported` 0 件という条件を維持する。source-backed nested argument finisher 内で inner head `add` に複数候補が見つかった場合は、候補ごとの owner reducer 投機実行を行わず、borrowed argument matcher で `item_index..item_count` を完全消費でき、かつ外側 parameter の expected type と result type が一致する候補数を数える。0 件または複数件なら `OverloadAmbiguous` とし、1 件だけならその候補を使って source-backed nested call finisher を 1 回だけ実行する。stage1 smoke は pipe suffix 由来の代表例として outer call の左辺 `I32Literal(1) [0, 1)` と suffix `CheckedExpr [3, 6)`、child `DirectCall` の candidate name `add`、inner arguments `I32Literal(2) [4, 5)` と `I32Literal(3) [5, 6)` を確認する。これは source-backed argument 範囲全体を inner call が消費する形の接続である。

2026-06-11 source-backed nested overload boundary checkpoint では、final-range だけでは扱えなかった `outer add 1 2 3` と pipe 左辺の `add 1 2 |> use 3` を同じ抽象境界へ整理した。`SelfhostNestedArgumentBoundary` は `FinalRange` と `OuterContinuation` を持つ enum であり、pipe 左辺のように渡された範囲だけで単一値を作る場合は `FinalRange`、通常 call の引数位置で inner call の後続 item を outer call の残り parameter として検査できる場合は `OuterContinuation` を使う。`OuterContinuation` では borrowed probe が inner candidate の `next_index` を返し、その `next_index` から outer candidate の残り parameter が source-less matcher で満たせる候補だけを数える。選択後の source-backed finisher は 1 回だけ実行し、実際の `next_index` が probe と一致しなければ `InternalInvariant` として fail-closed にする。pipe 左辺では `FinalRange` を渡すため、`add 1 2 |> use 3` の `add` は pipe operator 直前までを完全消費する候補だけを選び、pipe target の suffix `3` を left segment の continuation として使わない。stage1 smoke は `outer add 1 2 3` の outer argument order、inner child `DirectCall`、duplicate overload の `OverloadAmbiguous`、pipe-left nested overload の checked tree topology を確認する。lambda suffix、pipe chain trailing block、borrow を含む左辺、generic / trait solver が必要な nested callable は引き続き fail-closed 残件として維持する。

2026-06-11 pipe trailing block candidate checkpoint では、単一 `left |> named_target:` の末尾 block を、pipe target の残り parameter を満たす source-backed argument として接続した。候補選択の borrowed probe は block body を候補ごとに投機検査しない。suffix token だけで全 parameter を満たす候補に trailing block が付いた場合は、block が余剰引数になるため `NoMatch` として扱う。一方、suffix token だけでは `PartialApplicationRejected` になる候補で trailing block が存在する場合は、block body の expected type 検査が必要な `SourceBackedRequired` として分類する。これにより `Match` 0 件、`SourceBackedRequired` 1 件、`SelectionBlockedUnsupported` 0 件という既存の一意 source-backed finisher 条件を崩さず、`1 |> add:` を `add 1 <block-result>` 相当の checked `DirectCall` へ正規化できる。stage1 smoke は block body が `CheckedExpr` argument として保存され、その参照先 checked tree node が `BlockResult` であること、同名 overload の2引数候補だけが選ばれること、1引数候補では余剰 block が `UnexpectedTrailingBlockArgument` として拒否されることを確認する。pipe chain の後続段に trailing block が付く形はこの checkpoint では success path として受理せず、専用 smoke で `UnexpectedTrailingBlockArgument` として fail-closed にする。lambda suffix、borrow を含む左辺、generic / trait solver が必要な候補は引き続き fail-closed 残件として維持する。

2026-06-11 pipe chain trailing block checkpoint では、`left |> named_target suffix... |> named_target:` の末尾 block を chain 全体の最終 pipe 段だけが消費する success path へ進めた。中間段は `segment_trailing = none` のまま前段 `DirectCall` root を `SelfhostCheckedArgumentKind::CheckedExpr` として次段へ渡し、`trailing_block` owner は `selfhost_call_reduce_pipe_chain_finish_or_continue` で保持したまま進める。最終段で次の pipe が無い場合だけ、既存の trailing block argument finisher が `BlockResult` node を checked tree に追加する。これにより `1 |> add 2 |> add:` は最終段の第1引数が前段 checked expression `[0, 4)`、第2引数が末尾 block result `[6, 6)` の `DirectCall` になる。stage1 smoke は root `DirectCall` から前段 `DirectCall` node と `BlockResult` node の両方を辿り、block body の `add 1 1` が body-side `DirectCall` node として保存されたことまで確認する。後段 target 欠落と literal target は引き続き pipe-chain fail-closed smoke で typed error として固定する。lambda suffix、borrow を含む左辺、generic / trait solver が必要な候補は引き続き後続 slice の残件として維持する。

2026-06-05 explicit function value argument checkpoint では、source / token backed owner 入口で明示 `@ident` argument を検査する境界を追加した。Rust 実装の正規表層構文は `@` token の直後に identifier を置く形であり、`@function name` という keyword 付き構文ではない。`@ident` は expected parameter type が function type であり、対象 identifier が callable signature table から monomorphic named function candidate として一意に得られ、その function type が同じ arena 内で expected function type と構造一致する場合だけ 1 実引数式として成功する。bare `ident` は expected type が function type であっても暗黙 function value や partial application として扱わない。generic candidate、overload ambiguity、missing signature、function type mismatch は typed error として fail-closed にする。この checkpoint は argument typecheck 境界であり、HIR の function value identity lowering、indirect call、`memo_call` primitive は後続 slice に残す。

2026-06-05 HIR function value identity model checkpoint では、`SelfhostHirExprPayload::FnValue` と `SelfhostHirFunctionValueIdentity` を追加した。関数値は `str` だけで保存しない。identity は symbol、`Option SelfhostDefId`、関数型の `SelfhostTypeId`、`SelfhostEffectKind`、`type_arg_count` を持つ。`def_id = Option::None` は direct-call 専用 artifact などから来た未解決 identity を表し、accepted `FnValue` lowering、indirect call、`memo_call` へは進めない。`type_arg_count = 0` の monomorphic identity だけを現 checkpoint の安全な入口とし、generic type argument の実体は stable type-argument range / canonical key を導入する後続 issue に残す。この checkpoint は HIR の受け皿であり、`SelfhostCallableCandidate` / signature lookup から DefId 付き identity を生成する lowering、CallIndirect、MemoizedFunctionValue、PrivateCache proof はまだ開かない。

2026-06-05 callable candidate DefId evidence checkpoint では、`SelfhostCallableCandidate` に `SelfhostDefId` を追加した。`SelfhostCallableSignature` はすでに DefId-linked signature evidence を持っていたが、reducer 用 candidate へ変換する時点で DefId を落としていた。このままでは `@ident` argument checking から HIR `FnValue` identity を作る後続 slice が name string と function type だけに戻ってしまう。修正後は `selfhost_callable_candidates_push_signature` が `signature.def_id` を candidate に渡し、stage0 / stage1 の手作り fixture も `selfhost_def_id_new` を明示して candidate を作る。これにより、実 lowering の前段で identity evidence を失わない。

2026-06-05 function value HIR lowering boundary checkpoint では、`stdlib/neplg2/core/lower/hir/function_value.nepl` を追加した。この module は `SelfhostCallableCandidate` から `SelfhostHirFunctionValueIdentity` を作り、必要なら `SelfhostHirExprPayload::FnValue` の expression record まで作る。`candidate.def_id` は `Option::Some` として identity に保存し、generic candidate は stable type-argument range / canonical key がまだ無いため `GenericUnsupported` として fail-closed にする。`check/expr` は引き続き HIR record を直接生成せず、`lower/hir` が checker evidence と HIR model の両方を知る接続層になる。これは `@ident` argument matching の成功 payload をまだ HIR tree へ保存する段階ではなく、候補 identity を HIR identity へ落とす規則を文字列名依存から分離する checkpoint である。次の slice で argument match payload / call reduction result にこの lowering evidence を接続する。

2026-06-05 checked argument payload checkpoint では、`check/expr/argument_payload.nepl` を追加し、1 つの実引数式を検査した結果を `SelfhostCheckedArgument` として保持する境界を作った。`UnitValue` は source 再読なしで HIR `Unit` へ下ろせる unit literal、`BoolLiteral` / `I32Literal` / `F32Literal` / `CharLiteral` / `StrLiteral` は source-backed literal payload checker が一度だけ解析した semantic value、`NamedValue` は scope と value type evidence で照合済みの DefId-linked value identity、`TypedExpression` は現 checkpoint で HIR child payload がまだ不足している通常式を表す。`FunctionValue` は `@ident` が選んだ `SelfhostCallableCandidate` と use-site span を保持する。`NestedDirectCall` は nested named call が direct call として縮約されたことを保持し、`BlockResult` は末尾 block body が外側 parameter の expected type を満たしたことを保持する。`SelfhostExprArgumentOwnedMatch`、`SelfhostCallReduceOwnedResult`、`SelfhostExpressionLineCheckSuccess` はこの checked argument payload を owner として運び、`next_index` だけに戻らない。`check/expr` は HIR を直接作らず、後続の `lower/hir` が checked evidence を HIR payload へ変換する。

2026-06-05 HIR direct call checked argument lowering checkpoint では、`lower/hir/direct_call.nepl` を追加した。この module は `SelfhostCallReduceResult::DirectCall`、`Vec SelfhostCheckedArgument`、candidate table を受け取り、HIR module に argument child expression と parent call expression を追加する。callee は `SelfhostReducedCall.candidate_index` で candidate table から得るため、prefix token spelling や scope lookup を lowering で再実行しない。現 checkpoint で HIR child にできる argument kind は `UnitValue`、`BoolLiteral`、`I32Literal`、`F32Literal`、`CharLiteral`、`StrLiteral`、`NamedValue(identity)`、`FunctionValue(candidate)`、`CheckedExpr` である。`UnitValue` は HIR `Unit`、bool / i32 / f32 / string は対応する HIR literal、`CharLiteral` は Rust 実装と同じ i32-backed literal、`NamedValue` は `SelfhostHirValueIdentity` を持つ HIR `Var`、`FunctionValue` は `lower/hir/function_value.nepl` の `selfhost_hir_expr_fn_value_from_candidate` を通る HIR `FnValue` になる。`CheckedExpr` は同じ `SelfhostCheckedExprTree` 内の `DirectCall` / `BlockResult` / `BlockSequence` node を fuel 付きで下ろす。tree node id を持たない legacy summary の `TypedExpression`、`NestedDirectCall`、`BlockResult` は、source-less borrowed 境界の fail-closed evidence として残し、通常 lowering authority に戻さない。`SelfhostExpressionLineCheckSuccess` を borrow する facade も用意したが、success owner は caller が閉じる。これにより、checked argument payload transport は HIR direct call の消費点まで接続された。

2026-06-11 HIR direct call callee identity checkpoint では、`SelfhostHirCallExpr` を表示名 `name` と child range だけの payload から、`SelfhostDefId`、callee の `SelfhostTypeId`、`SelfhostEffectKind`、`type_arg_count` も保持する typed callee identity payload へ拡張した。`lower/hir/direct_call.nepl` は `SelfhostCallableCandidate` から HIR `Call` を作るとき、source token、scope lookup、callable candidate collection を再実行せず、checked tree node または reducer-selected candidate に保存済みの DefId-linked evidence をそのまま HIR へ移す。`name` は diagnostic / dump / backend symbol 用の表示名であり、monomorphize、Resource IR、indirect call、`memo_call` が参照する authority ではない。現 checkpoint では stable type-argument range がまだ無いため `selfhost_hir_expr_call` は `type_arg_count` を引数に取らず 0 に固定し、`lower/hir/direct_call.nepl` は `SelfhostGenericInferenceState::NoneRequired` 以外の candidate を `GenericCallIdentityUnsupported` として拒否する。`Unique` を暗黙の 0 型引数として HIR に保存せず、generic instantiation inference は後続 slice に残す。この変更により、`FnValue` だけでなく通常 direct call も文字列名だけに戻らず、後続の `memo_call @f` や indirect call の identity 比較が同じ DefId / type / effect 境界を使える。

2026-06-11 memoized function value HIR boundary checkpoint では、`SelfhostHirExprPayload::MemoizedFunctionValue` を追加し、`memo_call @pure_named_func` Phase 1 の結果を通常の `FnValue` と区別できる HIR leaf として保存する受け皿を作った。payload は `str` や表示名ではなく `SelfhostHirFunctionValueIdentity` を使うため、DefId、function type、effect、type argument count を後続 Resource IR / backend へ渡せる。`lower/hir/function_value.nepl` は `def_id = Option::Some`、`type_arg_count = 0`、`effect = Pure` の identity だけを accepted とし、DefId 欠落、generic identity、impure function はそれぞれ typed error で fail-closed にする。この checkpoint は cache allocation、hit / miss state、`PrivateCache` effect、Pure mask、sealed backend representation を作らない。`memo_call` が stdlib の compiler-known primitive であることの source identity 判定、`MemoKey` / `MemoValue` の構造制約、private cache SourceCapability、Resource IR の non-escape proof、即時適用の扱いは後続 slice のまま維持する。

2026-06-12 MemoKey / MemoValue selfhost predicate checkpoint では、`stdlib/neplg2/core/ty/ty/memo_trait.nepl` を追加し、`memo_call` Phase 1 が使う key / value type の fail-closed 判定を `core/ty` に閉じて実装した。主 API は `Result unit SelfhostMemoTraitRejectKind` を返すため、診断と regression は `MissingTypeRecord`、`F32KeyUnsupported`、`FunctionUnsupported`、`NamedLayoutUnknown`、`AppliedLayoutUnknown`、`ParameterUnresolved` などの enum reason を失わない。bool helper は既存 gate 用の adapter に留める。

この checkpoint の `MemoKey` は `unit`、`bool`、`i32`、`u8`、`char` だけを受理し、`MemoValue` は同じ範囲に `f32` を追加して受理する。`f32` は value としては Copy 相当で返せるが、NaN、符号付き zero、正規化、hash consistency が固定されるまでは key として拒否する。`I64` / `F64` は selfhost 型モデルに primitive として載っているが、現 Rust Phase 1 predicate の accepted set には含めないため、明示的に unsupported として拒否する。

`Named` / `Applied` は Rust 実装の最終方針では structural aggregate として受理され得る。しかし selfhost の現行 `SelfhostTypeArena` は field layout、trait impl evidence、Drop/Copy evidence を持たないため、今回の predicate は aggregate を推測で受理しない。後続 slice で type constructor layout evidence と trait solving が入った段階で、同じ reject reason を置き換えて structural traversal へ拡張する。したがって、この checkpoint は private cache proof、backend cache representation、`MemoKey` / `MemoValue` trait solver、aggregate acceptance を完了させるものではなく、未証明型を pure memoization へ流さない typed boundary を固定する作業である。

2026-06-12 MemoKey / MemoValue aggregate evidence consumer checkpoint では、同じ module に `SelfhostMemoTraitEvidenceTable` と `selfhost_memo_key_type_result_with_evidence` / `selfhost_memo_value_type_result_with_evidence` を追加した。これは aggregate structural proof を計算する solver ではなく、後続の layout / trait solver が作った `Result unit SelfhostMemoTraitRejectKind` payload を `Named` / `Applied` predicate が消費するための session-local table である。

証拠付き入口でも primitive、function、parameter、missing record の判定は evidence で上書きできない。`Named` / `Applied` だけが evidence table を参照し、証拠が無ければ従来どおり `NamedLayoutUnknown` / `AppliedLayoutUnknown` に fail-closed する。table は `SelfhostTypeId` を key にするため同じ TypeArena session の中だけで使い、永続 artifact では canonical type key と solver policy hash を別途使う。これにより、aggregate acceptance の本体を始める前に「solver 証拠を受け取る場所」と「未証明 aggregate を拒否する場所」を分離した。

stage0 smoke は `Named` / primitive / missing record と `Applied` を別 helper に分け、公開 smoke では小さな summary を合成するだけにした。これは evidence policy の runtime 確認を保ちながら、単一の巨大な prefix expression / nested match が call reduction の探索範囲を増やすことを避けるためである。doctest では、証拠なし aggregate の拒否、Named / Applied の accepted evidence、Named / Applied の rejected evidence、primitive fake evidence の無効化、missing record fake evidence の無効化を確認する。

2026-06-12 MemoKey / MemoValue aggregate evidence producer gate checkpoint では、`stdlib/neplg2/core/ty/ty/memo_trait_producer.nepl` を追加した。この module は field layout solver や trait solver ではなく、後続 solver が作った `SelfhostMemoTraitAggregateProof` を `SelfhostMemoTraitEvidenceRecord` へ変換する producer gate である。`SelfhostMemoTraitAggregateProof` は `SelfhostTypeId` だけではなく、substitution 済み field layout summary、Copy / Drop / Eq / Hash proof status、cache escape hazard classification、key/value の `Result unit SelfhostMemoTraitRejectKind` を持つ。accepted record の候補になるのは `SelfhostTypeRecord::Named` と `SelfhostTypeRecord::Applied` だけであり、primitive、function、generic parameter、missing type record は `SelfhostMemoTraitEvidenceProduceRejectKind` として拒否する。さらに `Named` / `Applied` であっても、field layout missing、invalid field range、generic argument unsubstituted、cycle limit reached、operation proof missing / impure / unknown、cache reference escape、external handle、owner token、public mutable state、unknown hazard があれば accepted record へ変換しない。

2026-06-12 MemoKey / MemoValue canonical proof store checkpoint では、`stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl` を追加し、session-local evidence table の前段に canonical key indexed proof store を置いた。store record は `SelfhostTypeId` を保持せず、store が所有する `SelfhostCanonicalTypeKeyArena` の root key、solver policy identity、proof kind、TypeId を含まない stored aggregate proof payload だけを持つ。lookup は現在 arena の TypeId を一時 canonical key arena へ再投影し、cross-arena canonical equality と policy 一致を確認してから、stored proof を現在 TypeId の `SelfhostMemoTraitAggregateProof` へ戻す。canonical key は同じだが policy が違う record は stale proof として覚え、後続に期待 policy の record がないか探索を続ける。最後まで期待 policy がない場合だけ `PolicyMismatch` を返すため、store key が「canonical key + solver policy identity」の組であるという契約と一致する。その後、既存 producer gate を必ず通すため、store に primitive や stale proof が混ざっていても consumer table へ直接入らない。typed error は projection failure、missing proof、policy mismatch、proof kind mismatch、producer rejection を分け、`ProducerRejected` は payload まで比較する。この checkpoint でも named type の canonical key は `SelfhostNamedTypeId` を使う Phase 1 であり、module path / public surface hash / stable constructor identity を含む serialized nominal key は後続 artifact slice の責務として残す。

producer gate の入力と出力はどちらも `Result` / enum payload を保つ。`key_result` と `value_result` は `Result unit SelfhostMemoTraitRejectKind` のまま record に移し、`f32` を含む aggregate のように key は拒否し value は受理する非対称な証拠を表現できる。producer は session-local `SelfhostTypeId` を扱うが、永続 artifact の identity ではない。将来 `.neplmeta` / `.neplproof` から復元する proof store は canonical type key と solver policy hash で索引し、その結果を現在の arena へ投影した後、この producer gate へ渡す。この checkpoint は solver 本体を実装したものではなく、solver が満たすべき summary 契約と、summary 不足時の fail-closed 境界を固定したものである。

この checkpoint により、consumer 側だけではなく producer 側にも non-aggregate proof を通常経路で table へ入れない境界ができた。ただし、struct / enum field traversal、recursive aggregate の cycle boundary、`MemoKey` / `MemoValue` trait source identity、pure Eq / Hash / Clone / Drop proof はまだ未実装であり、この producer の入力を作る solver 側の後続 slice として扱う。

2026-06-12 MemoKey / MemoValue source definition table checkpoint では、`stdlib/neplg2/core/ty/ty/memo_trait_source.nepl` に `SelfhostMemoTraitDefinitionSourceTable` を追加し、current `MemoKey` / `MemoValue` source registry を個別 prepared helper の直結ではなく table-backed validator 経由にした。table は `MemoKeyTrait` と `MemoValueTrait` の record slot と duplicate flag を持ち、missing key、missing value、duplicate key、duplicate value、key source rejected、value source rejected を `SelfhostMemoTraitTrustedSourceRegistryErrorKind` の typed variant として区別する。registry error equality は payload 付き enum を明示 match で比較し、wildcard arm や整数 code の衝突に依存しない。

この checkpoint の authority は `selfhost_memo_trait_trusted_source_registry_current_result` と `selfhost_memo_trait_trusted_source_identity_set_current_result` であり、proof store はこの Result API だけを使う。per-kind の current source identity helper は Stage 0 の materializer smoke 用の private helper に閉じ、外部 caller が missing / duplicate 検査を通らない source identity を policy へ入れないようにした。current table producer はまだ prepared i32 fingerprint を 2 件だけ入れる Phase 1 であり、full trait definition table scanner、public surface hash、stable trait definition key、serialized source fingerprint は後続 slice に残す。

2026-06-12 MemoKey / MemoValue type constructor layout evidence checkpoint では、`stdlib/neplg2/core/ty/ty/memo_trait_layout.nepl` を追加した。この module は `SelfhostTypeRecord` に field layout を混ぜず、constructor identity keyed な `SelfhostMemoTraitLayoutEvidenceTable` と substitution 済み field `SelfhostTypeId` の table で layout evidence を持つ。accepted path は `SelfhostNamedTypeId`、constructor arity、field range、field type id を authority とし、source spelling、display name、diagnostic string、module path suffix は layout 判定に使わない。

`selfhost_memo_trait_layout_evidence_for_type_result` は `Named` と `Applied` を分ける。`Named` では arity 0 constructor layout だけを受け入れ、type parameter を持つ constructor layout は `NamedConstructorHasTypeParameters` として拒否する。`Applied` では constructor identity と applied argument count が layout arity と一致し、型引数に未解決 parameter が残っていないことを確認してから field range を検査する。field range が table 外、field owner mismatch、field index mismatch、field type missing、generic field leakage は `SelfhostMemoTraitLayoutEvidenceErrorKind` の typed error で fail-closed にする。成功した場合だけ `SelfhostMemoTraitAggregateFieldEvidence::Known(range)` を返すが、Copy / Drop / Eq / Hash / hazard proof はまだ別入力として producer gate が要求する。

この checkpoint は aggregate acceptance を完了させるものではない。enum / sum layout、recursive aggregate の cycle boundary、Copy / Drop / Eq / Hash pure evidence、cache hazard proof、stable nominal key / serialized canonical key fingerprint は後続 slice に残る。runtime doctest は typed Result domain の小さな smoke に留め、validator 本体の contract は source-policy regression で直接固定して、テスト用の巨大な prefix expression が compiler の探索範囲を広げないようにした。

2026-06-05 named value HIR identity checkpoint では、`SelfhostHirExprPayload::Var` を `str` payload から `SelfhostHirValueIdentity` payload へ変更した。value identity は diagnostic / dump 用の `symbol`、name resolver が割り当てた `SelfhostDefId`、checker が照合した `SelfhostTypeId`、`Local` / `Param` / `Builtin` などの `SelfhostDefKind` を持つ。`argument.nepl` は `name -> latest binding -> DefId -> value type evidence` の成功時に `SelfhostCheckedArgumentKind::NamedValue` を作り、`lower/hir/direct_call.nepl` はその payload だけを使って HIR `Var` child を作る。lowering は source token、scope lookup、value evidence lookup を再実行しない。数値、bool、char、string literal は実際の値 payload と literal parserを持つ。`NestedDirectCall` と `BlockResult` は内側の checked tree がまだ不足しているため、引き続き fail-closed にする。

2026-06-05 literal value payload checkpoint では、`check/expr/literal_payload.nepl` を追加し、source-backed argument checker が bool / i32 / f32 / char / string literal の意味値を checked payload として保存する境界を作った。`SelfhostCheckedArgumentKind::BoolLiteral`、`I32Literal`、`F32Literal`、`CharLiteral`、`StrLiteral` は HIR を import せず、source token から一度だけ解析した値だけを保持する。`lower/hir/direct_call.nepl` はこれらを `SelfhostHirExprPayload::BoolLiteral`、`I32Literal`、`F32Literal`、既存 Rust 実装と同じ i32-backed char literal、`StrLiteral` の child expression へ変換し、literal lexeme や source text を再読しない。i32 は 10 進、`0x` / `0X` 16 進、`0b` / `0B` 2 進、`0o` / `0O` 8 進形式を accepted とし、空本体、無効 digit、overflow は `ArgumentLiteralI32Invalid` として fail-closed にする。未知の alphabetic radix marker は `ArgumentLiteralI32RadixUnsupported` として分ける。f32 は Rust 実装の現 checkpoint と同じく `FloatLiteral` を `f32` payload へ下ろすが、不正 spelling は 0.0 に丸めず `ArgumentLiteralF32Invalid` として fail-closed にする。char は単一 scalar、simple escape、`\xHH`、`\u{...}` だけを accepted とし、malformed quote、未対応 escape、不正 scalar、複数 scalar を typed error として分ける。string は quote 除去と対応済み escape decode を行い、semantic value を payload に入れる。未対応 escape、malformed escape、quote 不整合、builder failure はそれぞれ typed error として拒否する。

2026-06-11 binary / octal i32 literal checkpoint では、self-host lexer の numeric token 境界も `0b` / `0B` と `0o` / `0O` を `IntLiteral` として切り出すように揃えた。`syntax/lexer/literal.nepl` は 2 進 digit と 8 進 digit の predicate と終端 scan を持ち、scan は prefix 直後から連続する有効 digit だけを O(n) で進める。checker 側の `SelfhostLiteralI32RadixPlan` は token-local lexeme から radix 2 / 8 / 10 / 16 と body range を返し、`string::to_i32_radix` に semantic body だけを渡す。`add 0b1010 0o12` は stage1 smoke で 2 つの `I32Literal(10)` checked argument になることを確認し、HIR lowering は従来どおり checked payload を読むだけで source token を再読しない。numeric suffix と `i32` / `f32` 固定を越える defaulting は別設計として残す。

2026-06-05 ascription expected conflict checkpoint では、`%T expr` が作る `ExplicitAscription` expectation と外側 context から渡る `SelfhostTypeExpectation` を `body_line.nepl` の owner 付き入口で照合するようにした。両者の `expected_type` が同じ `SelfhostTypeArena` 内で構造一致しない場合、内側 expression の call reduction へ進まず `SelfhostExpressionLineCheckError::AscriptionExpectedTypeConflict` を返す。これにより、`%i32 expr` が `bool` expected の位置にあるような入力を candidate / arity / argument type error へ誤分類しない。`SelfhostTypeExpectation.expected_type` は arena-local なので、失敗 payload は arena 解放後も安全に読める source / span evidence だけを保持する。型そのものの安定表示は canonical type key / diagnostic projection の後続 slice で接続する。

この境界は owner を明示する。ascription projection は `SelfhostTypeArenaAlloc` を保持する success payload を返し、caller は `into_arena` で arena を受け取るか `free` で破棄する。これは borrowed arena から projection result を取り出す設計では lifetime が表現できないためであり、Rust 実装で得た ownership boundary の知見を self-host stdlib 側へ反映したものである。

この初期境界は fail-closed である。先頭 item が named value で、候補が 1 つだけで、候補 type が function type で、parameter count ぶんの実引数式を prefix cursor が過不足なく消費し、各引数式の型証拠が parameter type と一致し、expected result と候補 result が同じ arena 内で構造一致する場合だけ direct call plan を返す。候補が複数ある場合は、現段階では expected type や argument type による overload narrowing を行わず `OverloadAmbiguous` にする。generic inference state が `EvidenceMissing` / `Conflict` / `Unsupported` の場合は、それぞれ typed error に分ける。これにより、未完成の generic solver や overload solver が成功として後段へ流れない。

部分適用は許可しない。`add 1` が `fn i32 i32` を要求する文脈であっても、NEPLg2.1 の一般規則として関数値を暗黙生成しない。関数値が必要な場合は `@ident` や明示的な lambda を使う。

zero-argument function は実引数なしで呼ぶ。

```neplg2
fn answer %fn void i32 \void:
    42

fn main %fn void i32 \void:
    answer
```

`answer void` は書かない。

## Effect と purity

表層 effect は当面 `Pure` / `Impure` の二値で維持する。ただし内部検査では、観測できない内部状態を区別する。

```text
InternalEffect:
    Pure
    PrivateAlloc region
    PrivateState region
    PrivateCache region
    UnsafeMemory
    ExternalIo
    Nondet
    PublicState
    Unknown
```

`Pure` は「内部 mutation が存在しない」ではなく、「外部から観測可能な effect が存在しない」という意味である。

`PrivateState` や `PrivateCache` はそのまま Pure ではない。fresh region に閉じ、戻り値、global、public field、raw pointer、stats API、hit/miss 観測へ escape しないことを証明した boundary の内側でだけ Pure に mask できる。

## memo_call

`memo_call` は、最初は compiler-known trusted stdlib primitive として実装する。

公開上の目標型は概念的には次である。この block は型システム上の構造を示す擬似表記であり、NEPLg2.1 source syntax そのものではない。

```text
memo_call:
    MemoKey K =>
    MemoValue V =>
    input = Function { params = [K], result = V, effect = Pure }
    result = Function { params = [K], result = V, effect = Pure }
```

Phase 1 では、実装範囲を保守的にする。

- 引数は明示的な `@ident` で得た non-capturing named pure function value に限定する。
- user が定義した同名の通常関数を `memo_call` primitive として扱わない。compiler-known 判定は stdlib source identity を確認した registry が発行する trusted `DefId` に限定し、candidate の表示名や token spelling を allowlist として扱わない。
- `memo_call @f arg` のような即時適用は Phase 1 では受理しない。まず `memo_call @f` が memoized function value を返す境界を固定する。
- `func` は monomorphic な 1 引数 pure function とする。複数引数は tuple key 化を別段階にする。
- `K` と `V` は Copy または conservative MemoKey / MemoValue として認められる型に限定する。
- cache region は compiler が fresh に導入し、public type に現れない。
- cache lookup は owned / copied / cloned value を返し、cache 内部参照を返さない。
- hit/miss、size、clear、stats、address identity、allocation state を pure API として公開しない。

Resource IR では `MemoizedFunctionValue` と `PrivateCache` / `PrivateState` proof boundary を持つ。cache implementation correctness は trusted stdlib primitive と tests の責務とし、compiler は effect escape と public observation を検査する。

現行 Rust 実装は、`memo_call @f` の typecheck / HIR 境界を先に持つ段階である。sealed backend cache representation と通常 compile path の private-cache mask proof は未完成のため、セルフホスト設計でも「backend private cache は fail-closed」「SourceCapability と private-cache mask proof は別 authority」として扱う。

2026-06-11 selfhost memo_call compiler-known identity gate checkpoint では、`lower/hir/memo_call.nepl` を追加し、`memo_call @func` として縮約済みの direct-call evidence から HIR `MemoizedFunctionValue` leaf を作る入口を分離した。accepted 判定は trusted `DefId` equality だけを使い、`candidate.name == "memo_call"` や source token lexeme を読み直さない。Phase 1 helper は `FunctionValue` checked argument 1 個だけを受理し、即時適用、通常値、result type mismatch、同名 user 関数を typed error で fail-closed にする。source function value の DefId / generic / Pure gate は既存 `lower/hir/function_value.nepl` に委譲し、private cache allocation、`MemoKey` / `MemoValue`、Resource IR `PrivateCache` proof、backend wrapper は後続 slice に残す。

2026-06-11 selfhost compiler-known primitive identity registry checkpoint では、上記の trusted identity を lowering-local な `SelfhostCompilerKnownMemoCallIdentity` から、`builtins/prelude/compiler_known.nepl` の共有 `SelfhostCompilerKnownPrimitiveIdentity` へ移した。identity は `SelfhostCompilerKnownPrimitiveKind::MemoCall`、監査用 `module_path` / `symbol`、resolver-local `DefId` を持つ。`lower/hir/memo_call.nepl` は `SelfhostCompilerKnownPrimitiveKind::MemoCall` と candidate `DefId` の両方が一致する場合だけ accepted path へ進む。これにより、typecheck、HIR lowering、Resource IR、backend が将来同じ primitive kind を参照できる。prelude registry は `lower/hir`、`hir`、`check/expr` に依存せず、source hash / policy hash materialization、`MemoKey` / `MemoValue`、private cache proof、sealed backend は引き続き後続 slice に残す。

2026-06-12 selfhost `MemoKey` / `MemoValue` stable source evidence checkpoint では、`core/check/module/memo_trait_source_fingerprint.nepl` を追加し、AST scanner が作る `signature_available = false` の候補 table と、typed public surface materializer が将来渡す stable fingerprint evidence を突き合わせる境界を作った。candidate table は `MemoKey` / `MemoValue` trait declaration が 1 件ずつ存在することと重複しないことだけを証明し、fingerprint payload には使わない。accepted source identity の材料は、別 evidence table が持つ module identity、stable trait definition key、normalized public signature fingerprint の 3 つがすべて存在し、かつ scanner placeholder と同じ `0` ではない場合だけ作られる。

この checkpoint は public surface hash の計算本体ではない。hash 計算、trait signature normalization、module identity の永続化、re-export を含む public surface materializer は後続 slice に残る。ただし、後続 materializer が入った後も、scanner の source spelling や span だけを trusted source identity へ昇格させず、欠落・重複・未確定 fingerprint を typed `Result` で fail-closed にする API 境界はここで固定した。

Phase 2 では、`run_private` / `mask_private` に相当する一般 private region effect へ拡張する。

## HIR

HIR は型検査後、Resource IR lowering 前の typed source-level IR である。

HIR は次を持つ。

- expression kind
- type id
- surface effect
- source span
- typed function value identity
- indirect call
- memoized function value
- raw body marker

function value identity は、表示用の symbol だけではなく、定義 id、関数型、effect、型引数情報をまとめた typed record として扱う。Rust 実装の `FunctionValueIdentity` と同じく、monomorphize、Resource IR summary、backend symbol、`memo_call` namespace が同じ対象を見ていることを確認する boundary である。self-host の現段階では generic type argument の実体をまだ保持しないため、0 以外の `type_arg_count` は accepted lowering ではなく fail-closed evidence として扱う。

HIR は永続 artifact にそのまま保存しない。`TypeId`、`Span`、`FileId`、temporary local id は session-local であり、cross-session cache key にしてはならない。

drop insertion は Resource IR proof 後に HIR へ戻す。drop の要否を parser/typechecker/codegen が個別判断しない。

## Resource IR

Resource IR は ownership、borrow、initialized state、drop plan、effect boundary の authority である。

Resource IR の検査は少なくとも次を含む。

- lowering coverage
- move / initialized state
- owner-backed aggregate obligation
- borrow and lifetime
- drop elaboration plan
- effect boundary
- raw memory / source capability boundary
- collection slot lifecycle
- private effect masking

静的検査は specialized checker の寄せ集めにしない。source capability、typed facts、resource place、effect op、owner obligation、summary dependency graph を共通の証明基盤に載せる。

## Proof Engine

`stdlib/neplg2/core/proof/` は、セルフホストコンパイラ全体の検査基盤である。

設計単位は次とする。

```text
Fact:
    Source
    Module
    Type
    Trait
    Effect
    Lifetime
    Owner
    Resource
    Artifact

Query:
    NameResolve
    TypeResolve
    TraitSolve
    CallReduce
    ResourceInitialized
    ResourceBorrow
    ResourceEffect
    ResourceDrop
    CacheReplay

ProofResult:
    proven
    disproven
    unknown
    diagnostics
    dependencies
```

query は deterministic input value だけを受け取り、副作用を持たない pure function として実装する。これにより、same-session cache、artifact replay、incremental recompilation の基盤にできる。

safety authority になる query では、`unknown` は成功ではない。`unknown` は追加診断で compile を止めるか、cache miss として full recompute へ戻す。未証明のまま Resource IR static check を通過させない。

proof 実装は、巨大な一括 issue にしない。次のように入力、出力、negative fixture が明確な slice へ分ける。

| proof slice | 入力 | 出力 | 必須 fixture |
|---|---|---|---|
| SourceSpan validation | source text、byte range | valid span または diagnostic | negative / inverted / out-of-range span |
| lowering coverage | typed HIR | Resource IR op coverage report | 未対応 HIR kind の compile_fail |
| initialized state | Resource IR、function body | move / init diagnostics、summary | branch 片側未初期化、move 後 use |
| owner obligation | owner-backed aggregate fact | owner summary、diagnostic | non-Copy owner の escape / double drop |
| borrow / lifetime | resource place、borrow op | lifetime proof、diagnostic | borrowed ref escape、mut alias |
| effect boundary | internal effect op、function effect | Pure / Impure fold、diagnostic | pure から impure call、unmasked private effect |
| drop plan | ownership summary | drop elaboration plan | early return、match arm、panic boundary |
| summary replay | `.neplproof` header、body hash | replay hit / miss report | stale proof、policy mismatch、type mismatch |

各 slice は Rust 実装と比較できる artifact を持つ。lexer token JSON、parser flat prefix AST、typed public signature hash、Resource IR proof summary、diagnostic JSON、stage timing JSON を段階ごとの比較対象にする。

## Artifact

セルフホストコンパイラは、Rust 実装で導入した役割分離を引き継ぐ。

| artifact | 役割 |
|---|---|
| `.neplmeta` | public interface / module surface。依存側の名前解決と型検査に必要な公開情報だけを持つ |
| `.neplproof` | Resource IR proof summary。source / policy / typed signature / dependency surface / function body hash に一致した場合だけ再利用する |
| `.neplobj` | backend body fragment。direct-call fragment など codegen 入力 cache を持つ |

artifact header は次を含む。

- schema version
- compiler identity hash
- syntax version
- target
- profile
- test mode
- stdlib content hash
- source cache key hash
- dependency public surface hash
- typed public signature hash
- source capability policy hash
- private effect policy hash

artifact は fail-closed にする。header 不一致、schema 不一致、policy 不一致、body hash 不一致、generic instantiation mismatch、span 再投影失敗は cache miss として通常 pipeline に戻す。

`.neplmeta` に function body、typed HIR、Resource IR、diagnostic span、`TypeId`、`FileId` を入れない。

artifact ごとの key material は同一ではない。

| artifact | 固有 key material |
|---|---|
| `.neplmeta` | public surface hash、typed public signature hash、module / import / export surface、syntax version、target/profile/test mode |
| `.neplproof` | Resource summary namespace、function body stable hash、dependency surface hash、source capability policy hash、private effect policy hash、generic type argument key |
| `.neplobj` | backend feature set、stable link symbol、selected body hash、generic instantiation hash、target ABI、referenced function/data symbol set |

`.neplmeta` は typecheck authority であり、dependency body authority ではない。materialized dependency の direct call が backend 入力に残る場合は、`.neplobj` fragment が backend plan で受理されるか、source fallback で dependency body を取得する。body がない依存を成功として codegen しない。

## Cache と incremental compile

性能目標は次である。

- 通常の 1 program compile を 0.5 秒未満へ近づける。
- literal 変更や式枝差し替えのような小変更では 0.1 秒未満、最終的には 10 ms 級の再コンパイルを目指す。
- stdlib は generic 具体型代入以外の検査を事前にほぼ終わらせ、依存側では interface / proof / object fragment を link / replay する。

cache に頼るだけではなく、cold base compile の計算量を下げる。

Rust 実装の RPN profiling では、Resource IR static check、initialized moves、owner obligation、raw init、function summary fixed point が主要な時間を使っている。セルフホスト側では次の設計を先に入れる。

- module graph と summary dependency graph を 1 度だけ構築する。
- summary kind ごとの relevant function set を先に計算する。
- entry reachable pruning を Resource IR 前に行う。
- direct call graph が曖昧な場合だけ conservative-all に fallback する。
- function body hash と dependency edge hash を分ける。
- public surface が変わらない body-only edit では downstream typecheck を無効化しない。
- expression subtree hash を持ち、局所式差し替えでは親 chain と影響先 summary だけを再計算する。
- proof query は input fact set の structural hash を key にする。

cache mode は次のように区別する。

| mode | 意味 | 設計上の扱い |
|---|---|---|
| cold base | proof / check cache を使わず、source から通常 pipeline を走らせる | 計算量削減の基準。空 cache を作るだけの固定費を入れない |
| proof-backed cold | disk `.neplproof` が存在する状態で初回 compile する | header / policy / body hash が一致した proof だけ preseed する |
| warm session | 同じ compiler session 内の loader / public surface / summary cache を再利用する | Web playground / LSP の小変更高速化に使う |
| exact check cache | 同一 source set の full check 成功だけを保存する | proof ではない。stale / malformed / options mismatch では通常 pipeline に戻る |

空の Resource summary cache を作るだけで cold base compile が遅くならないようにする。proof preseed は既存 artifact がある時だけ opt-in で行い、bootstrap / benchmark では cache 無効化の計測も標準にする。

## Loader と module graph

loader は path resolution と module graph の唯一の authority である。

処理対象は次である。

- root source
- default prelude
- `#no_prelude`
- `#import`
- `#include`
- `pub #import`
- `.neplmeta` materialized dependency

`include` は source merge boundary として扱い、included file の変更で includer を無効化する。`import` と `prelude` は interface boundary として扱い、`.neplmeta` だけで依存側の typecheck を進められるようにする。

module graph は cycle detection、reexport projection、export table、dependency public surface hash を提供する。

## Monomorphize

monomorphize は worklist 方式にする。

key は次を含む。

- original definition stable identity
- canonical type arguments
- trait impl selection
- target
- profile
- private effect policy
- source capability policy

mangle seed や表示名だけで instance を同一視しない。bucket 用 hash と full key equality を分ける。

Resource IR static check 前の monomorphize と、drop insertion 後の backend monomorphize を分ける。drop insertion によって backend 入力が変わるためである。

## Codegen

初期 target は Wasm / WASI を優先する。LLVM は host toolchain 依存が強いため、CLI boundary と backend adapter を分離してから扱う。

Wasm backend は次を生成する。

- type section
- import section
- function section
- table / element section
- memory section
- data section
- export section
- code section

`.neplobj` は完成済み Wasm module の連結単位ではなく、NEPLg2 backend が再構成できる function body fragment / signature / referenced symbol / data requirement の cache として扱う。

raw `#wasm` / `#llvmir` は call graph と effect の保守性を壊しやすい。Rust 実装と同じく、raw body が混じる場合は pruning や object reuse を保守的に制限する。

## Standard Library と test

`stdlib/neplg2/` 自体は NEPLg2.1 で書く。コメントは日本語の `.n.md` extended markdown とし、各 module の目的、algorithm、計算量、制約、diagnostic 方針を明記する。

test は 2 系統を併用する。

- `.n.md` の `neplg2:test` / `compile_fail` / `should_panic`
- source 中で直後 1 statement に作用する `#test`

`#test` は parser / loader / public surface / artifact key に影響するため、通常 compile の source と混同しない。

`#test` の検証契約は次である。

- 通常 mode では `#test` item が runtime reachability、public surface、`.neplmeta` export surface に入らない。
- test mode では `#test` item が test harness の entry candidate になり、通常 mode と artifact key が一致しない。
- `#test` の有無だけが異なる source は、通常 mode の public runtime surface が同一であることを確認する。
- test mode で生成した `.neplmeta` / `.neplproof` / `.neplobj` は通常 mode の compile に replay しない。

## Bootstrap

セルフホスト達成までの段階は次の通りである。

1. Rust compiler が `stdlib/neplg2/` の self-host compiler を compile する。
2. Rust compiler で作った self-host compiler が stdlib と小さな test program を compile する。
3. self-host compiler が自分自身を compile する。
4. Rust 出力と self-host 出力の public surface、diagnostic、artifact header、Wasm output を比較する。
5. reproducible build が安定したら、CI に bootstrap check を追加する。

Pass 比較では byte 完全一致だけを最初の条件にしない。Wasm section ordering、debug comment、symbol name が異なる可能性があるため、まずは semantic output と public artifact hash の一致を確認し、最終的に deterministic emission へ進める。

## 実装フェーズ

各 phase は複数 issue に分割する。issue は「入力」「出力」「negative fixture」「性能または cache 境界」「完了コマンド」を持つ粒度にする。

性能は Phase の最後にまとめて追加しない。loader、parser、type resolver、Resource IR、backend の各段階で、timing key、dependency key、relevance pruning、cache replay の受入条件を最初から定義する。

### Phase 0: 設計と文書の更新

- 古い NEPLg2.0 self-host 文書を historical plan として明示する。
- `stdlib/neplg2/README.md` と `index.n.md` を NEPLg2.1 前提へ更新する。
- `todo.md` には未着手の実装タスクだけを残し、完了報告は `note.n.md` に記録する。

Issue slice:

- design doc landing update
- stale NEPLg2.0 wording audit
- self-host issue inventory remap

### Phase 1: Infra validation

- `SelfhostSourceSpan` の non-negative / ordered / in-range validation を徹底する。
- integer range、type id、HIR id、resource id を raw i32 のまま公開しない。
- HIR child / parameter range と function type argument range は、0 件を `Empty` variant、1 件以上を checked nonempty payload として表す。外部入力から作る場合は typed error を返す checked constructor を使い、arena 内で直前の table 長と追加件数から証明済みの場所だけ `_unchecked` constructor を使う。
- diagnostic code enum と reporter boundary を固定する。

Issue slice:

- SourceSpan constructor validation
- TypeId / HirId / ResourceId newtype validation
- diagnostic code enum and JSON reporter parity
- source text line table and byte offset conversion

Performance acceptance:

- line / column lookup は line start table を再構築しない。
- diagnostic JSON 生成は reporter boundary のみに置き、compiler core の hot path で string formatting をしない。

### Phase 2: NEPLg2.1 lexer parity

- `%`、`\`、`void`、`unit`、`#test`、offside token を Rust 実装と揃える。
- `void` は reserved keyword として扱い、identifier として束縛しない。
- old token は migration diagnostic に必要な範囲だけ保持する。

Issue slice:

- keyword and directive classification
- offside Indent / Dedent parity
- raw body token boundary
- lexer token JSON parity fixture

Performance acceptance:

- tokenization は source を 1 pass で走査する。
- doc comment / normal comment の output cache key 正規化を lexer 結果から計算できるようにする。

### Phase 3: NEPLg2.1 parser parity

- expression と type は flat prefix list として保持し、parser で call boundary を決めない。
- `%T expr`、`\a\b:`、`\void:`、`fn void T`、`fn unit T` を正規に扱う。
- module declaration header では、`%` type annotation と lambda header を `SelfhostSyntaxRange` として保持する。これは最終的な型木・式木ではなく、後続 resolver / checker が kind / arity / expected type に基づいて境界を解くための flat token range evidence である。
- `syntax/ast/prefix_expr.nepl` は `SelfhostSyntaxRange` から `SelfhostExprPrefixList` を作る。expression prefix list は `%` marker を保持し、call boundary、expected type、overload、generic、trait、partial application の判断は Type checker へ渡す。
- `module_parser/body_range.nepl` は declaration body block の envelope と first expression segment を `SelfhostSyntaxRange` として保持する。複数式 body や nested block は envelope を後段 segmenter が扱い、`first_expression` は初期 call reduction 入力のための bounded expression range として使う。
- `syntax/parser/body_segmenter.nepl` は body envelope を `ExpressionLine` / `BlockIntro` の typed segment list へ分解する。`ExpressionLine.head` は `SelfhostExprPrefixList` の入力候補であり、`BlockIntro.body` は nested body envelope として再帰的に segmenter へ渡す。
- 旧 `()` grouping、angle type、generic postfix は正規 grammar から外し、必要なら migration diagnostic に限定する。

Issue slice:

- flat prefix expression parser
- flat prefix type parser
- lambda header and `void` marker parser
- `#test` item marker parser
- parser flat AST JSON parity fixture

Performance acceptance:

- parser は backtracking で call boundary を探索しない。
- no-progress guard を持ち、invalid source で無限 loop しない。
- source span と subtree hash を parse node に保持し、式枝差し替えの invalidation 境界に使えるようにする。

### Phase 4: Module interface

- loader、module graph、public surface、typed public signature、`.neplmeta` 生成を実装する。
- stdlib prelude / import を `.neplmeta` で満たせるようにする。
- `include` は source merge boundary、`import` / `prelude` は interface boundary として扱う。

Issue slice:

- logical module id and VFS snapshot
- import / include / prelude resolution
- cycle detection and reexport projection
- public surface hash
- typed public signature hash
- `.neplmeta` materialization and fail-closed fallback

Performance acceptance:

- stdlib module body を依存側 typecheck のたびに再parseしない。
- body-only edit では dependency public surface hash を変えない。
- loader timing と dependency aggregate hash を stage timing に出す。

### Phase 5: Type resolver

- kind-directed type application resolver を実装する。
- imported type constructor arity と local declaration header を prefix type reduction に使う。
- `void` を型として登録しない。
- parser の `%` annotation range は、`resolve/type_resolver` で `%` marker を除いた flat type prefix item list へ変換する。ここでは `TypeId` を生成せず、`fn` / `void` / named type などの token role と span/token index だけを保持する。
- type prefix item list は `resolved` tree へ縮約する。`resolved` tree は `TypeId` 割当前の arena-local node table であり、primitive / named type reference / generic type parameter / applied named type / function type node を保持する。
- `fn i32 fn i32 i32` のように result が nonempty function type の場合は、部分適用を導入せず multi-argument function type へ flatten する。`fn void fn unit unit` のように 0 引数 function が function を返す場合は、`void` marker の境界で flatten せず nested function type として保持する。
- reducer は source-dependent primitive detection と syntax validation を `reduce/plan.nepl`、owner table への build を `reduce/build.nepl`、共有 payload を `reduce/model.nepl` に分ける。constructor / type parameter lookup が必要な経路では `SelfhostTypeBoundPlan` を作り、validate / build は同じ束縛結果を共有する。旧 constructor-aware validate/build helper は公開 surface から外し、build 層は source string を読まず、plan が作った enum / bool / span payload だけを消費する。
- `project.nepl` は `resolved` tree root と `SelfhostTypeArena` を受け取り、primitive / function type を arena-local `SelfhostTypeId` へ投影する。constructor table なし API は named type / applied named type を `UnsupportedNamedType` として fail-closed にする。
- `constructor.nepl` は module surface / local declaration header から構築される named type constructor table の最小形を持つ。table 登録時に `SelfhostTypeConstructorKind` へ arity を正規化し、負 arity、予約名、同一 table 内の重複名を `SelfhostTypeConstructorTableErrorKind` として拒否する。table lookup は `source + span` から一時 name key を切り出し、arena へは `SelfhostNamedTypeId` だけを保存する。
- `project.nepl` の lookup 付き API は arity 0 の named constructor を `SelfhostTypeRecord::Named` へ投影する。unknown named type と bare generic constructor は typed error として fail-closed にし、既存の constructor table なし API は named を拒否し続ける。
- constructor-aware reducer は `SelfhostTypeConstructorKind` に従って `Box i32` / `Result i32 str` のような type argument list を再帰的に消費し、`SelfhostResolvedTypeNode::Applied` として保持する。型引数不足は projection まで送らず `GenericTypeArgumentMissing` として reducer で拒否する。
- constructor-aware projection は `Applied` node の constructor identity を constructor table で再検査し、constructor kind が要求する型引数数と resolved tree の argument range が一致する場合だけ、projected type argument `SelfhostTypeId` list を `SelfhostTypeRecord::Applied` として arena へ保存する。arena は source spelling ではなく identity と structural argument list だけを保持する。reducer が作った tree はすでに kind-checked だが、resolved tree constructor は public API でも作れるため、TypeArena へ入る直前の projection 境界でも `UnknownNamedType` / `GenericConstructorArgumentArityMismatch` として fail-closed にする。
- `ty/key.nepl` は `SelfhostTypeArena` の root `SelfhostTypeId` を `SelfhostCanonicalTypeKeyArena` へ投影する。canonical key node は `SelfhostTypeId` を持たず、primitive / named / type parameter / applied / function の構造と key argument range だけを保持する。projection は型 record と argument edge の数に対して O(n) であり、同じ key arena 内の structural equality を提供する。
- type-parameter-aware reducer は generic binder から作った `SelfhostTypeParameterEnv` を参照し、`T` / `E` のような名前を `Named` ではなく `SelfhostResolvedTypeNode::Parameter` として保持する。constructor table と parameter environment の両方に同じ名前がある場合は `TypeParameterConstructorNameConflict` として fail-closed にする。
- `SelfhostTypeArena` は type parameter を `SelfhostTypeRecord::Parameter` として保存する。payload は source name / span / resolver-local node id ではなく、`SelfhostTypeParameterBinding { binder_depth, parameter_index }` だけである。
- 現 checkpoint の projection は、1 つの generic binder environment から得た `SelfhostTypeParameterId` を `binder_depth = 0` の `SelfhostTypeParameterBinding` へ正規化する。nested binder depth と永続 artifact 用の stable binder identity は signature / interface artifact 実装時に追加する。

Issue slice:

- imported / local type constructor table construction
- imported type arity hint integration
- type resolver diagnostic parity

Performance acceptance:

- type constructor lookup は module surface から構築した table と `SelfhostTypeBoundPlan` を使い、prefix type ごとに import graph を再探索しない。constructor / type parameter lookup は validate と build の両方で繰り返さず、binding phase で 1 回に固定する。
- canonical type key を生成し、artifact / cache key へ `TypeId` を入れない。現 checkpoint では type parameter binding を含む structural key tree と equality までを実装済みであり、cross-arena serialized key / fingerprint は interface artifact 接続時に追加する。

### Phase 6: Type checker and higher-order functions

- prefix call reduction、expected type、overload、generic、trait solving、no partial application を実装する。
- `@ident` と function value identity を Rust 実装に合わせる。
- indirect call と pure / impure call boundary を検査する。

Issue slice:

- lambda / borrow / source-backed advanced pipe target argument expression success path を含む prefix call reduction stack。source-backed `left |> named_target suffix...`、`left |> %fn ... named_target suffix...`、`left |> named_target suffix... |> named_target suffix...` は checked tree 付き direct call として接続済みで、注釈無し pipe target の一意 source-backed suffix は `%T literal`、`NamedValue`、単一候補 nested call、source-backed nested argument 範囲全体を消費する同名 overloaded nested call、通常 call 引数の outer continuation 付き同名 overloaded nested call、pipe left の final-range overloaded nested call、単一 pipe trailing block argument、pipe chain trailing block argument まで checked argument payload へ接続済み。lambda suffix、borrow を含む左辺、generic / trait solver が必要な nested callable は後続 slice の残件として扱う
- generic instantiation inference
- trait bound solving
- no partial application diagnostics
- `@ident` identity and indirect call
- pure context effect diagnostics

Completed checkpoint:

- `ExpressionLine.head` から `SelfhostExprPrefixList` を作り `check/expr` へ渡す接続
- `%T expr` から `SelfhostTypeExpectation::ExplicitAscription` を作り、inner expression tail だけを call reducer へ渡す接続
- `ExpressionLine.head` の identifier を `SelfhostNameScope` と callable signature table へ通し、DefId-linked candidate list を call reducer へ渡す初期接続
- literal argument item の型証拠を function parameter type と照合し、未対応 argument expression を `UnsupportedArgumentExpression` で fail-closed にする初期接続
- `%T expr` の `ExplicitAscription` と outer expected type が衝突した場合に、内側 call reduction 前の typed error として拒否する接続
- call reducer の raw `item_count - 1` argument count 依存を外し、argument expression ごとの `next_index` cursor で prefix list を走査する接続
- source / token backed owner 入口で `%T literal` を 1 つの argument expression として消費し、ascription projection failure と明示 ascription type / parameter expected type の衝突を typed error として拒否する接続
- source / token backed owner 入口で `NamedValue` と `%T NamedValue` を scope binding と DefId-linked value evidence table へ通し、binding 欠落 / DefId 未割当 / unsupported binding kind / 型証拠欠落を別 error として拒否する接続
- source / token backed owner 入口で nested named call argument を callable signature table へ通し、内側 call の `next_index` を外側 argument cursor へ返す接続
- BlockIntro 専用 owner 入口で `head` prefix と `body` envelope を分け、単一 expression line の末尾 block argumentを `BlockResult`、複数 expression line の末尾 block argument を `BlockSequence` checked tree node として検査し、nested `BlockIntro` も同じ segment dispatcher で再帰的に source-backed reducer へ渡す接続。空 body / 入力構築失敗 / 非末尾式用 unit type 欠落は `TrailingBlockBody*` error として typed fail-closed にする
- source / token backed owner 入口で `left |> named_target suffix...`、`left |> %fn ... named_target suffix...`、`left |> named_target suffix... |> named_target suffix...` を `named_target left suffix...` 相当の `DirectCall` checked tree へ正規化し、左辺と右辺 suffix の checked argument payload 順序を保持する接続。ascribed target の同名 overload narrowing は callable type equality で 0 / 1 / 複数一致を分類する。pipe chain の2段目以降は前段 root を `CheckedExpr` argument として渡し、trailing block は中間段で消費せず最終段だけが `BlockResult` argument として受け取る。注釈無し pipe target は borrowed probe で `Match` / `NoMatch` / `SourceBackedRequired` / `SelectionBlockedUnsupported` を分類し、`Match` 0 件かつ `SourceBackedRequired` 1 件だけで `SelectionBlockedUnsupported` が無い場合だけ既存 source-backed finisher に進む。`%T literal`、`NamedValue`、単一候補 nested call suffix、source-backed nested argument 範囲全体を完全消費できる同名 overloaded nested call、通常 call の outer continuation 付き overloaded nested call、pipe left の final-range overloaded nested call、単一 pipe trailing block argument、pipe chain trailing block argument はこの境界で checked argument payload へ接続済みである。`Match` と `SourceBackedRequired` の混在、`SourceBackedRequired` 複数件、または `SelectionBlockedUnsupported` は `PipeTargetAmbiguous` に閉じる。source-less reducer と lambda target / borrow target / lambda suffix など複合式 success path は fail-closed 残件として維持する
- `@ident` function value argument、nested direct call argument、block result argument の checked payload を `SelfhostCheckedArgument` として call reduction / body line success 境界まで運び、`SelfhostCallableCandidate.def_id` を HIR lowering 前に落とさない接続
- `lower/hir/direct_call.nepl` で checked argument list を消費し、`FunctionValue(candidate)` を HIR `FnValue` child として parent direct call expression に接続する初期 lowering
- `unit` literal argument を `UnitValue` checked payload として保存し、source 再読なしで HIR unit expression child へ lower する接続
- `NamedValue` argument を `SelfhostCheckedValueIdentity` として保存し、source / scope 再読なしで DefId-linked HIR `Var` child へ lower する接続

Performance acceptance:

- callable candidate table を scope ごとに事前構築する。
- call reduction は open call state を bounded に保ち、全候補の再探索を繰り返さない。
- overload / trait solving は query cache key を持ち、同じ fact set の再解決を避ける。

### Phase 7: HIR and Resource IR lowering

- typed HIR を作り、Resource IR lowering coverage を検査する。
- HIR を永続 artifact にそのまま保存しない。
- Resource IR op と HIR kind の対応表を明示する。

Issue slice:

- HIR arena and stable debug projection
- Resource IR place / op model
- lowering coverage check
- function value and memoized function value lowering
- raw body conservative lowering boundary

Performance acceptance:

- entry reachable pruning を Resource IR static check 前に適用する。
- HIR subtree hash と Resource body hash を分ける。

### Phase 8: Resource proof slices

- initialized moves、owner obligation、borrow/lifetime、effect boundary、drop plan を proof query に載せる。
- Resource summary cache と fail-closed replay を実装する。

Issue slice:

- initialized state proof
- raw init summary proof
- owner obligation proof
- borrow / lifetime proof
- effect boundary proof
- drop elaboration proof
- summary dependency graph
- `.neplproof` replay and miss diagnostics

Performance acceptance:

- summary dependency graph を 1 度だけ構築する。
- proof slice ごとに relevant function set を計算する。
- irrelevant function は安全に空 summary として扱うか、linear lightweight checker へ回す。
- RPN cold base timing で Resource IR の階層内訳を出す。

### Phase 9: Private effect / memo_call

- compiler-known `memo_call` primitive を実装する。
- `PrivateCache` / `PrivateState` region と non-escape proof を Resource IR に追加する。
- Phase 1 では conservative MemoKey / MemoValue に限定し、後で一般 private region effect へ拡張する。

Issue slice:

- `@ident` pure named function restriction
- MemoKey / MemoValue conservative trait
- MemoizedFunctionValue HIR and Resource IR lowering
- PrivateCache exact boundary proof
- stale private cache escape negative fixture

Performance acceptance:

- memo cache proof は general Resource IR proof の relevant function set に載せる。
- private cache operation を pure と同一視せず、mask 可能性だけを cache する。

### Phase 10: Artifact and incremental compile

- `.neplmeta`、`.neplproof`、`.neplobj` を role-separated artifact として出す。
- expression subtree replacement cache と compiled output cache key を実装する。

Issue slice:

- `.neplmeta` writer / reader
- `.neplproof` writer / reader
- `.neplobj` direct-call fragment
- artifact schema version and policy hash
- expression subtree invalidation
- body-only edit reuse
- tiny edit benchmark

Performance acceptance:

- stale artifact は必ず miss になり、成功扱いしない。
- literal または式枝差し替えは、親 chain と影響先 summary だけを再計算する。
- tiny edit benchmark で 0.1 秒未満を目標値として記録する。

### 2026-06-12 memo trait `.neplproof` artifact schema checkpoint

`MemoKey` / `MemoValue` stored proof を `.neplproof` artifact へ載せる前段として、selfhost compiler 側に typed schema boundary を追加した。

この schema は codec 本体ではない。binary / text reader、serializer、canonical key tree payload encoding、disk / bundled artifact への保存は後続 slice に残す。今回固定したのは、reader が payload merge 前に検査すべき header、serialized proof record key、record payload、sidecar index entry、typed artifact error である。

Header は artifact schema、canonical payload schema、policy schema、record count、index count を持つ。Record key は store-local `SelfhostCanonicalTypeKeyId` ではなく、serialized canonical payload schema、canonical fingerprint、canonical payload hash、solver policy identity を持つ。Record payload は proof kind、stored aggregate proof、record payload hash をまとめる。Index entry は fingerprint、record ordinal、record payload hash だけを持つ候補 narrowing payload であり、proof acceptance authority ではない。

受理の authority は fingerprint 単独ではない。reader / preseed 側は、canonical payload schema、canonical fingerprint、canonical payload hash、policy、proof kind、stored proof payload、record payload hash をそれぞれ fail-closed に扱い、最終的には proof store preseed / lookup gate と producer gate を通す。source text、source span、path suffix、display name、diagnostic text、lexeme、store-local id、session-local TypeId は `.neplproof` schema の accepted authority にしない。

この checkpoint により、後続の `.neplproof` reader / serializer は proof store の store-local stable identity struct を直接永続化せず、serialized canonical key payload と typed policy / proof payload を分離して実装する。

### 2026-06-12 memo trait `.neplproof` decoded preseed materialization checkpoint

`.neplproof` artifact schema と proof store preseed decision の間に、decoded record を materialized canonical key と照合する bridge boundary を追加した。

この bridge は reader / serializer 本体ではない。Phase 1 では、caller が decoded canonical key payload から一時 `SelfhostCanonicalTypeKeyArena`、`SelfhostCanonicalTypeKeyId`、canonical fingerprint を作った後の接続点だけを固定する。canonical payload hash は bridge 内で materialized canonical key tree から再計算し、caller supplied な整数は accepted authority にしない。binary codec、serialized canonical key tree payload、persistent stable map、disk / bundled artifact I/O は後続 slice に残す。

bridge は decoded record key と record body を artifact schema boundary で再検査し、materialized key の存在、materialized canonical key tree から再計算した canonical payload hash、canonical fingerprint、solver policy をこの順で照合する。いずれかが壊れている場合は `SelfhostMemoTraitNeplProofPreseedErrorKind` の typed error として返し、proof store の decision へ進めない。

照合を通過した record だけが `selfhost_memo_trait_proof_store_preseed_decision_materialized_key` へ渡される。store relation は `AcceptMissing`、`ExistingMatching`、`RejectedConflict` の typed enum で返し、同一 stable identity / 同一 payload の再投入は skip 可能、同一 stable identity / 差分 payload は fail-closed conflict として扱う。stage0 smoke は empty store の `AcceptMissing` だけでなく、stable proof を seed した store に対する bridge 経由の `ExistingMatching` と `RejectedConflict` も実行で固定する。

永続 artifact の accepted authority は、serialized canonical key payload hash、canonical fingerprint、typed policy、proof kind、stored proof payload、record payload hash である。store-local `SelfhostCanonicalTypeKeyId`、`SelfhostMemoTraitProofStoreStableIdentity`、session-local `SelfhostTypeId`、source text、span、path suffix、display name、diagnostic text、lexeme は bridge API の入力 authority にしない。

### 2026-06-12 memo trait canonical key payload hash checkpoint

decoded preseed bridge の前提だった canonical payload hash を、caller supplied な整数から selfhost 側の typed producer へ移した。

`memo_trait_canonical_key_payload.nepl` は、materialized canonical key arena と stable nominal key table から `.neplproof` record key 用の `SelfhostMemoTraitCanonicalKeyPayloadHash` を作る。payload hash は payload schema、canonical node kind、primitive stable code、stable nominal key、argument order だけから決まる。source text、span、path、display name、diagnostic text、lexeme、store-local id、session-local `SelfhostTypeId` は authority にしない。

`Parameter` と `Function` は Phase 1 では fail-closed にする。generic type argument identity と higher-order function identity は別 issue の stable identity boundary が入るまで `.neplproof` payload hash の受理対象にしない。壊れた canonical key arena に対しては node 数と argument 数から作る traversal fuel により無制限再帰を止め、missing node、missing argument、invalid argument range、fuel exhaustion を typed error として返す。

`.neplproof` artifact schema は payload schema boundary と canonical fingerprint schema の両方を確認する。数値は Phase 1 で同じだが、payload schema function と fingerprint schema function は別 authority として扱う。decoded preseed bridge は `3003` のような固定 payload hash や public API caller から渡された raw hash を使わず、materialized canonical key から payload hash producer を呼ぶ。stage0 は producer の値で record key を作り、hash mismatch は record key 側だけを壊して、再計算値との不一致として確認する。

subagent review では、初期実装で public preseed API が `materialized_canonical_payload_hash %i32` を受け取っていた点が High として指摘された。修正後は public API に `&SelfhostMemoTraitStableNominalKeyTable` を渡し、bridge 内部で `selfhost_memo_trait_canonical_key_payload_hash_result` を呼ぶ。payload producer の `Applied` node hash にも payload schema version を混ぜ、module doc の「schema は hash 入力である」という契約と実装を一致させた。

この checkpoint 後も、binary reader / serializer、serialized canonical key tree bytes、decoded record から proof store へ append する永続 preseed loop、persistent stable map / serialized index は未実装である。

### 2026-06-12 memo trait canonical key payload bytes codec checkpoint

`.neplproof` reader / serializer 全体へ進む前段として、serialized canonical key tree payload の bytes codec を `memo_trait_canonical_key_payload_codec.nepl` に分離した。

codec は bytes を直接 proof store へ渡さない。header、fixed-width node table、argument table を decode し、stable nominal key material を `SelfhostMemoTraitStableNominalKeyTable` に照合して、現在の session 内の `SelfhostCanonicalTypeKeyArena` と root key を再構築する。serialized payload は payload schema、node kind、primitive stable code、stable nominal key material、argument order だけを authority とし、`SelfhostCanonicalTypeKeyId`、`SelfhostNamedTypeId`、`SelfhostTypeId`、proof store record index、source text、span、path、display name、diagnostic text、lexeme を永続 authority にしない。

decode は `SelfhostMemoTraitCanonicalKeyPayloadDecodeErrorKind` を返す。schema mismatch、unexpected end、trailing bytes、unknown node tag、unknown primitive code、negative count、count limit、root / argument target out of range、invalid argument range、unsupported parameter / function node、missing / duplicate / invalid nominal key、word high-bit、allocation failure、hash projection failure を typed enum として分ける。bool や display string へ潰さず、source policy でも enum error と `Result` API を固定した。

`selfhost_memo_trait_canonical_key_payload_decode_and_hash_result` は convenience API として、decoded arena/root を既存の `selfhost_memo_trait_canonical_key_payload_hash_result` へ渡して hash を再計算する。bytes 内または record key 内の hash を信用する受理経路は持たない。preseed の acceptance authority は引き続き materialized arena/root と typed policy / proof payload を照合する bridge 側に置く。

subagent review では、codec を「hash を読む境界」にせず「typed decoded canonical key payload から materialized arena/root へ戻す境界」にすること、artifact / preseed / proof store と責務を混ぜないこと、source text や store-local id を authority にしないことが Required として確認された。実装はこの指摘に従い、codec module は proof store、artifact、preseed、checker、HIR、Resource IR、backend を import しない。

この checkpoint 後も、`.neplproof` record reader / serializer、decoded record から proof store へ append する preseed loop、persistent stable map / serialized index、generic type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary は未実装である。

### 2026-06-12 memo trait `.neplproof` decoded batch preseed checkpoint

decoded canonical key payload codec と single-record append boundary の次段として、複数の materialized decoded `.neplproof` record を working proof store へ順に preseed する batch boundary を `memo_trait_proof_preseed.nepl` に追加した。

batch input は `SelfhostMemoTraitNeplProofDecodedBatchRecord` として、materialized canonical key id、期待する proof store policy、typed artifact record を名前付き field で持つ。reader / serializer、record bytes decode、serialized index、persistent stable map はこの boundary の責務に含めない。これにより、reader は「record 群を読む」責務、preseed boundary は「materialized record を検査して store へ投入する」責務に分かれる。

`selfhost_memo_trait_neplproof_decoded_record_batch_append` は入力 store owner を消費し、すべての record が `AcceptMissing` append または `ExistingMatching` skip として処理された場合だけ `Ok(store)` を返す。途中で `RejectedConflict`、record validation、fingerprint、payload hash、policy、store append のいずれかが失敗した場合は、single-record append boundary の contract に従って store owner を閉じ、batch 側は `record_ordinal` と typed nested error を持つ `SelfhostMemoTraitNeplProofPreseedBatchError` を返す。vector entry が取得できない場合も `RecordMissing` として store を閉じ、partial seeded store を成功値として返さない。

stage0 smoke は empty batch、同一 record 2 件による `ExistingMatching` skip、2 件目 conflict による ordinal 1 の `RejectedConflict`、2 件目 invalid record による ordinal 1 の `DecisionInvalid(ArtifactRecordInvalid(...))` を確認する。contract test は public batch API、ordinal 付き error、nested append error equality、入力順 loop、store cleanup、materialized fingerprint 再計算、計算量説明を固定する。

この checkpoint 後も、`.neplproof` record reader / serializer、persistent stable map / serialized index、generic type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash は未実装である。

### 2026-06-12 memo trait `.neplproof` decoded artifact batch projector checkpoint

decoded artifact owner と decoded batch append boundary の間に、`SelfhostMemoTraitNeplProofDecodedArtifact` と materialized canonical key id vector から `SelfhostMemoTraitNeplProofDecodedBatchRecord` vector を作る projector boundary を追加した。

この boundary は `.neplproof` reader / serializer 本体ではない。reader が record bytes を typed `SelfhostMemoTraitNeplProofRecord` vector へ decode し、canonical payload bytes を materialized key arena へ投影した後、record ordinal と materialized key id ordinal を 1 対 1 に対応付ける段階である。artifact owner の header / records / indexes invariant は `selfhost_memo_trait_neplproof_decoded_artifact_validate_result` で再検査し、`header.record_count` と materialized key id vector length が一致しなければ `MaterializedKeyCountMismatch` として fail-closed にする。件数が一致しても、各 `SelfhostCanonicalTypeKeyId` が caller-owned `SelfhostCanonicalTypeKeyArena` で読めなければ `MaterializedKeyMissing(record_ordinal)` として拒否する。

projector は `selfhost_memo_trait_neplproof_decoded_artifact_record_at_result` を使って record を copy-out し、同じ ordinal の materialized key id と期待 policy を束ねて batch record を作る。materialized key id の vector 欠落、arena 内欠落、record copy-out 失敗、output vector allocation / push failure は `SelfhostMemoTraitNeplProofDecodedBatchBuildErrorKind` の typed variant として返し、部分構築した output vector は boundary 内で閉じる。成功時の output vector owner は caller が batch append 後に閉じる。

この projector は proof acceptance authority ではない。artifact record key、materialized key id、expected policy を batch input へ束ねるだけであり、canonical payload hash、canonical fingerprint、policy、store relation、producer gate は後続の decoded batch append と proof store preseed decision が再検査する。source text、span、path、display name、diagnostic text、lexeme、store-local id、session-local `TypeId` はこの projector の authority にしない。

stage0 smoke は、1 record artifact と 1 key id vector から batch record vector を作る成功経路と、1 record artifact に空 key id vector を渡す件数不一致を確認する。source policy は decoded module import、typed builder error、artifact validation、key id count check、record/key ordinal pairing、partial output cleanup、stage0 smoke、reader / serializer 未導入を固定する。

この checkpoint 後も、`.neplproof` record reader / serializer、persistent stable map / serialized index、generic type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash は未実装である。

### 2026-06-12 memo trait `.neplproof` decoded index table validation checkpoint

decoded batch preseed の前段として、`.neplproof` reader が header / record vector / sidecar index vector を構築した直後に通す index table validation boundary を `memo_trait_proof_artifact.nepl` に追加した。

`selfhost_memo_trait_neplproof_index_table_result` は header の `record_count` / `index_count` と decoded vector length を照合し、record payload、index entry payload、index entry が指す record の fingerprint / record payload hash を再検査する。そのうえで、sidecar index table が record ordinal を一対一に覆っていることを確認する。同じ record ordinal へ複数 index entry が向く場合は `IndexRecordOrdinalDuplicate`、どの index entry からも覆われない record ordinal がある場合は `IndexRecordOrdinalMissing` として fail-closed にする。

この validation は proof acceptance authority ではない。index entry は候補 narrowing のための sidecar であり、ここで `Ok unit` になっても proof store へ投入してよい構造になっただけである。実際の受理は、後続の canonical payload decode、materialized canonical key hash 再計算、policy 照合、decoded batch preseed、proof store lookup、producer gate を通して決まる。source text、span、path、display name、diagnostic text、lexeme、store-local id、session-local `TypeId` は引き続き authority にしない。

現 stage の coverage 検査は decoded `Vec` の線形 scan を組み合わせるため O(n * m + m * m) である。これは reader / serializer 実装前に fail-closed contract と typed error を固定するための段階であり、persistent stable map / serialized index を追加するときは同じ error enum と不変条件を保ったまま O(n + m) へ移す。

stage0 smoke は accepted table、record count mismatch、index count mismatch、invalid record、invalid index entry、index-record mismatch、duplicate ordinal、missing coverage を public aggregate validator 経由で確認する。safe `Vec` から通常発生しない defensive missing entry は、duplicate scan と coverage scan の broken vector read を `IndexEntryMissing` に分類することで、duplicate や coverage missing と混ぜない。

この checkpoint 後も、`.neplproof` record reader / serializer、persistent stable map / serialized index、generic type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash は未実装である。

### 2026-06-12 memo trait `.neplproof` sorted index lookup contract checkpoint

decoded index table validation の次段として、`.neplproof` sidecar index の sorted order と candidate range lookup contract を `memo_trait_proof_index.nepl` に分離した。

この checkpoint は persistent map や binary reader / serializer 本体ではない。reader が構築した header / record vector / index vector に対して、header validation、decoded index table validation、sorted order check を通し、canonical fingerprint が一致する index entry 群の `SelfhostMemoTraitNeplProofIndexCandidateRange` だけを返す。range は index vector 内の開始 index と件数であり、proof payload や policy payload は返さない。

sorted order は `(canonical_fingerprint.schema_version, canonical_fingerprint.root_hash, record_ordinal)` の昇順である。同じ fingerprint の entry は連続する collision group として扱い、複数候補があること自体は合法である。canonical equality、policy equality、proof kind、producer gate は proof store / preseed 側の authority として残す。つまり fingerprint hit は候補 narrowing であり、proof acceptance ではない。

実行 smoke では、通常の単一候補 lookup に加えて、同じ fingerprint を持つ 2 entry が `start_index = 0`、`candidate_count = 2` の合法な collision candidate range として返ることを確認する。これにより、collision group を破損や duplicate と誤分類せず、後段の canonical payload / policy / proof gate へ渡す境界を固定する。

破損は `SelfhostMemoTraitNeplProofSortedIndexErrorKind` で表す。header schema / count boundary は `HeaderInvalid(SelfhostMemoTraitNeplProofArtifactErrorKind)`、decoded table validation の拒否は `TableValidationRejected(SelfhostMemoTraitNeplProofIndexValidationErrorKind)` として nested typed payload を保つ。sorted order の破損は `FingerprintOrderInvalid` / `RecordOrdinalOrderInvalid`、候補不在は `CandidateMissing` として分ける。

source text、span、path suffix、display name、diagnostic text、lexeme は lookup key、sort key、tie-break authority にしない。proof store の stable sidecar index と artifact serialized index も混ぜず、artifact 側は record ordinal 候補、store 側は canonical equality / policy / producer gate という境界を保つ。

2026-06-12 follow-up で、現実装の lookup は decoded `Vec` 上の lower-bound binary search へ進めた。public lookup boundary は header validation、decoded table validation、sorted order check を省略せず、その後に half-open range `[low, high)` の binary search で `target <= entry.canonical_fingerprint` となる最初の index を求める。見つかった位置の fingerprint が target と一致する場合だけ、そこから同じ fingerprint の collision group を数える。candidate range lookup 自体は O(log m + c) である。c は同じ fingerprint の collision candidate 数である。

この follow-up でも range は proof acceptance ではない。binary search は candidate group の開始位置だけを縮小し、canonical payload decode、payload hash 再計算、policy、proof kind、decoded batch preseed、proof store lookup、producer gate は後続に残す。`IndexEntryMissing`、`CandidateMissing`、sorted order corruption の typed error も既存の `SelfhostMemoTraitNeplProofSortedIndexErrorKind` のまま保持する。

2026-06-12 follow-up で、decoded artifact owner 側に `selfhost_memo_trait_neplproof_decoded_artifact_candidate_record_at_result` を追加した。この boundary は `lookup_result` が返した `SelfhostMemoTraitNeplProofIndexCandidateRange`、caller が lookup に使った target fingerprint、range 内の相対 offset を受け取り、artifact invariant、range / offset、target fingerprint、index entry と record の対応を再検査したうえで `SelfhostMemoTraitNeplProofDecodedCandidateRecord` を返す。

candidate record は index entry と record の Copy payload pair であり、artifact owner 内部の storage location や参照を外へ出さない。`CandidateAccessInvalid(SelfhostMemoTraitNeplProofDecodedCandidateErrorKind)` は invalid range、offset out of range、projection-local index / record slot missing、target fingerprint mismatch、index-entry / record fingerprint mismatch、index-entry / record payload hash mismatch、unexpected artifact validator error を typed variant として保持する。これにより、後段の preseed / proof store / producer gate は vector slot を直接読み直さず、同じ fail-closed projection boundary を通って candidate record を受け取れる。

projection-local missing は既存の単体 accessor error である `RecordEntryMissing` / `IndexEntryMissing` へ混ぜない。`record_at_result` と `index_at_result` は artifact 全体から単体 slot を読む defensive accessor であり、candidate projection は lookup range、target fingerprint、index entry、record ordinal の関係も含めて検査する別責務だからである。

この follow-up でも candidate record は proof acceptance ではない。target fingerprint と record payload hash の整合を再確認しても、canonical payload decode、policy、proof kind、decoded batch preseed、proof store lookup、producer gate は後続の authority として残る。`memo_trait_proof_decoded.nepl` は引き続き proof store / preseed / source construction へ direct import せず、decoded artifact owner と candidate projection だけを担当する。

2026-06-12 follow-up で、preseed bridge 側に `selfhost_memo_trait_neplproof_decoded_batch_record_from_candidate_result` を追加した。この boundary は decoded artifact、materialized canonical key arena、materialized key id vector、期待する proof store policy、lookup target fingerprint、candidate range、candidate range 内 offset を受け取り、decoded artifact の candidate accessor を通して `SelfhostMemoTraitNeplProofDecodedBatchRecord` 1 件を作る。

この boundary の重要な contract は、candidate range 内 offset と record ordinal / materialized key id ordinal を混同しないことである。materialized key id vector の lookup には `candidate_offset` ではなく、candidate accessor が返した `index_entry.record_ordinal` を使う。件数不一致は `MaterializedKeyCountMismatch`、arena 内の key id 欠落は `MaterializedKeyMissing(record_ordinal)`、candidate accessor の拒否は `CandidateInvalid(SelfhostMemoTraitNeplProofDecodedBatchCandidateError { candidate_offset, kind })` として保持する。

実装では candidate record owner の nested field を直接深く読むのではなく、`field::get_ref` で `index_entry` と `record` を一段ずつ copy-out してから batch record を作る。これは Resource IR 上で partial field move と未初期化 cell を作らないための境界であり、候補投影の意味論ではなく、safe owner lifecycle を保つための実装 contract である。

この single-candidate projector も proof acceptance authority ではない。candidate accessor が target fingerprint と index / record payload hash の整合を確認しても、canonical payload hash、canonical fingerprint、policy、store relation、producer gate は decoded batch append と proof store preseed decision が再検査する。source policy は、candidate accessor 経由、`record_ordinal` による key id lookup、arena existence check、typed `CandidateInvalid` wrapping、`candidate_offset` を key id ordinal として使わないことを固定する。

この checkpoint 後も、`.neplproof` record reader / serializer、persistent stable map / serialized index の実体、generic type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash は未実装である。

### 2026-06-12 memo trait `.neplproof` candidate range preseed checkpoint

sorted sidecar index lookup が返した `SelfhostMemoTraitNeplProofIndexCandidateRange` を、working proof store へ流し込む preseed boundary を追加した。

`selfhost_memo_trait_neplproof_decoded_candidate_range_preseed` は proof store owner を消費し、stable nominal key table、materialized canonical key arena、decoded artifact、materialized key id vector、期待 policy、lookup target fingerprint、candidate range を借用または Copy payload として受け取る。各 candidate は range-local `candidate_offset` で走査し、既存の `selfhost_memo_trait_neplproof_decoded_batch_record_from_candidate_result` を通して `SelfhostMemoTraitNeplProofDecodedBatchRecord` へ変換する。その後、既存の `selfhost_memo_trait_neplproof_record_append_materialized_checked` へ渡し、canonical payload hash、canonical fingerprint、policy、store relation、proof store append の検査を再実行する。

この boundary は proof acceptance authority ではない。candidate range は探索範囲を `c` 件へ狭めるだけであり、受理は append boundary と proof store / producer gate に残る。`candidate_offset` は range-local offset であり、materialized key id vector の ordinal、artifact record ordinal、sorted index の absolute position として扱わない。key id lookup は既存 single-candidate projector が candidate accessor から得た `index_entry.record_ordinal` だけを使う。`expected_policy` は caller supplied の reuse boundary として保ち、record key から導出しない。

失敗は `SelfhostMemoTraitNeplProofDecodedCandidateRangePreseedError` に閉じ込める。candidate build が失敗した場合は、append がまだ store owner を受け取っていないため、この boundary が入力 store owner を閉じて `BuildInvalid` を返す。append が失敗した場合は、既存 append boundary が store owner を閉じる contract なので、range preseed 側は二重 close せず `AppendInvalid` を返す。どちらの error も failing `candidate_offset` と nested typed error を保持し、bool や diagnostic string へ潰さない。

既存の decoded preseed stage0 summary はすでに多くの結果 payload を持つため、この checkpoint では candidate range preseed の成功 / invalid range smoke を専用 helper として実行し、summary の field には追加しない。これにより、既存 smoke の戻り値契約を膨らませず、候補範囲 preseed が実行経路から外れないことだけを stage0 内で固定する。

この boundary も `.neplproof` reader / serializer ではない。record bytes decode、persistent stable map、serialized index、artifact file I/O、generic type argument stable identity は引き続き後続 slice の責務である。計算量は現時点では candidate 数 `c` に対して、candidate accessor 側の artifact validation を毎回通すため `O(c * artifact_validation + c * n * k + materialization)` である。cache 設計に頼る前に探索範囲を candidate group へ狭める段階だが、artifact validation の再実行は後続で shared validated view へ分離する余地がある。

### 2026-06-12 memo trait `.neplproof` sorted index producer checkpoint

sorted index lookup contract の次段として、decoded record vector から sorted sidecar index vector を作る producer boundary を同じ `memo_trait_proof_index.nepl` に追加した。

この checkpoint は `.neplproof` reader / serializer 本体ではない。reader または serializer がすでに decoded `SelfhostMemoTraitNeplProofRecord` vector を持っている状態から、record ordinal ごとに 1 つの `SelfhostMemoTraitNeplProofIndexEntry` を作り、既存の decoded table validation と sorted order validation を通した owner vector だけを返す。

producer は各 record を `selfhost_memo_trait_neplproof_record_key_result` と `selfhost_memo_trait_neplproof_record_result` へ再投入する。そのうえで `selfhost_memo_trait_neplproof_index_entry_result` で sidecar entry を作る。不正 record、placeholder hash、schema mismatch、index entry build rejection は `SelfhostMemoTraitNeplProofIndexProducerErrorKind` の typed variant として返し、bool や diagnostic string へ潰さない。record vector の defensive missing は `RecordEntryMissing`、bubble-back 中の produced index slot missing は `IndexEntryMissing` として分け、どの入力 owner が壊れたかを error kind から追えるようにする。

producer output は proof acceptance authority ではない。返す index は canonical fingerprint、record ordinal、record payload hash だけを持つ candidate narrowing table であり、canonical key bytes decode、payload hash 再計算、policy、proof kind、decoded batch preseed、proof store lookup、producer gate は後続の責務として残る。source text、span、path suffix、display name、diagnostic text、lexeme、`SelfhostTypeId`、`SelfhostNamedTypeId`、`SelfhostCanonicalTypeKeyId`、proof store stable identity は producer の sort key、lookup key、dedup key、error 分類に入れない。

producer implementation は proof store lookup / push / preseed、store-local stable identity、decoded batch append API を直接呼ばない。producer の責務は decoded record から sidecar index を作って既存 validator へ通すことまでであり、proof store へ投入するかどうかは reader / preseed bridge と proof store / producer gate が別段階で決める。

現 stage の producer は insertion sort 相当の bubble-back により `(canonical_fingerprint.schema_version, canonical_fingerprint.root_hash, record_ordinal)` の昇順へ整列する。計算量は record 数 n に対して O(n^2) である。これは decoded artifact 上の Phase 1 contract を先に固定するための実装であり、後続の binary writer、persistent stable map、serialized index では同じ `Result` / enum contract を保ったまま O(n log n) または O(n) の構築へ置き換える。

stage0 smoke は、意図的に unordered な record vector から first / second fingerprint の candidate range を作れること、same fingerprint の 2 record が collision group として `candidate_count = 2` になること、invalid record が `RecordInvalid(RecordPayloadHashPlaceholder)` として fail-closed に拒否されることを確認する。

この checkpoint 後も、`.neplproof` record reader / serializer、persistent stable map / serialized index の実体、generic type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash は未実装である。

### 2026-06-12 memo trait token-aware public surface seed scan core checkpoint

method-bearing `MemoKey` / `MemoValue` trait definition を扱う token-aware public surface scan を、seed / hash / token_gate から独立した `memo_trait_public_surface_token_seed_scan.nepl` へ分離した。

shared core は parser が作った `SelfhostModuleAst` と同じ source から得た `SelfhostToken` stream を受け取り、marker trait と method-bearing trait を同じ `SelfhostMemoTraitStableSourceSeedTable` へ正規化する。accepted authority は memo trait kind、public visibility、固定 declaration ordinal、signature shape normalizer または method signature normalizer が返す `normalized_signature_hash` に限定する。source text、span、syntax range、lexeme、path、display name、diagnostic text、source hash は seed / hash authority に混ぜない。

この module は `memo_trait_public_surface_seed`、`memo_trait_public_surface_hash`、`memo_trait_public_surface_token_gate` を import しない。逆に seed / hash / token_gate が shared core を direct import し、core 専用の `SelfhostMemoTraitPublicSurfaceTokenSeedScanErrorKind` を各 module の公開 error surface へ写像する。method normalizer の error payload は `MemoKeyMethodSignatureRejected` / `MemoValueMethodSignatureRejected` として core error に保持し、seed/hash/token_gate 側の mapping でも payload を落とさない。

`memo_trait_public_surface_token_gate.nepl` は互換 wrapper に縮小した。既存の item / module API は維持するが、declaration dispatch、method-bearing trait normalization、unsupported public surface rejection は shared core に委譲し、token_gate 側は seed error mapping だけを持つ。`memo_trait_public_surface_hash.nepl` は token_gate wrapper を経由せず shared core の item helper を直接呼ぶため、hash materializer 内の scan loop が同じ policy を使いながら wrapper 由来の循環依存を作らない。

この checkpoint により、前段 checkpoint で残っていた seed 側 private token scan と facade-external token gate の重複は解消された。`stdlib/neplg2/core/check/module.nepl` facade は shared core も token_gate も re-export しないため、安定公開面には AST-only public surface API を残し、token-aware path は内部 materializer / orchestrator の direct import に閉じる。

source policy は dedicated error enum、DAG、facade 非公開、method error payload preservation、hash の direct shared-core use、seed/token_gate の wrapper mapping、proof_store / HIR / Resource / backend への逆依存禁止、source identity / hash / span / path authority 禁止、行数制限や doc comment 長制限を入れないことを固定する。

この checkpoint 後も、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、`.neplproof` reader / serializer、永続 artifact 用 stable map / serialized index、generic instantiation 用 stable type argument identity は未実装である。

### 2026-06-12 memo trait public surface registry single-pass checkpoint

registry convenience path に残っていた candidate scanner と public surface hash materializer の module item stream 二重走査を、`memo_trait_public_surface_hash.nepl` 内の combined registry materializer へ移した。

`SelfhostMemoTraitPublicSurfaceHashRegistryScanState` は candidate source table と public surface seed table を別 field として保持する。candidate table は source spelling から `MemoKey` / `MemoValue` trait 候補を分類するためだけに使い、seed table は public surface hash の authority になる typed seed field だけを保持する。両者を同じ table に統合せず、成功時は `SelfhostMemoTraitPublicSurfaceHashRegistryMaterialization` として `candidates` と `materialization` に分けて返す。

AST-only registry path は `selfhost_memo_trait_public_surface_hash_registry_materialize_result` を通り、token-aware registry path は `selfhost_memo_trait_public_surface_hash_registry_materialize_with_tokens_result` を通る。どちらも module identity validation 後に module item stream を 1 回だけ走査し、candidate table と seed table を同じ item pass で更新する。standalone `selfhost_memo_trait_definition_source_table_scan_module_result`、`selfhost_memo_trait_public_surface_hash_materialize_result`、`selfhost_memo_trait_public_surface_hash_materialize_with_tokens_result` は、単体検査と互換境界として残す。

error taxonomy は combined path でも分けたままにした。candidate 分類の AST 不整合は `CandidateScanRejected(SelfhostMemoTraitDefinitionScanErrorKind)`、public surface seed / hash 側の拒否は `PublicSurfaceHashRejected(SelfhostMemoTraitPublicSurfaceHashErrorKind)`、stable evidence / registry validator の拒否は `SeedRegistryRejected(SelfhostMemoTraitStableSourceSeedRegistryErrorKind)` で返す。同じ trait item で candidate 分類と seed 分類の両方が失敗し得る場合は candidate 分類を先に評価し、malformed trait header は hash error ではなく `CandidateScanRejected` になる契約を doc comment と source policy で固定した。

accepted authority は従来どおり seed table、public surface hash materializer、stable evidence producer、trusted registry validator に置く。source spelling は candidate classification にだけ使い、source text、span、syntax range、lexeme、path、display name、diagnostic text は hash fold や accepted source identity に入れない。token-aware combined path は shared token-aware seed scan core を直接使い、token_gate wrapper や facade re-export 経由の循環を作らない。

計算量は registry convenience API 全体で module item 数 n と method-bearing trait body token 数 k に対して O(n + k) である。これにより、full public surface hash へ進む前の既知の探索範囲増加要因を 1 つ閉じた。

この checkpoint 後も、re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、`.neplproof` reader / serializer、永続 artifact 用 stable map / serialized index、generic instantiation 用 stable type argument identity は未実装である。

### 2026-06-12 memo trait `.neplproof` header reader codec checkpoint

`.neplproof` record reader / serializer へ進む前段として、永続 artifact が共有する 31-bit little-endian word codec と、`.neplproof` header prefix reader を追加した。

`memo_trait_artifact_word_codec.nepl` は 4 byte little-endian word decode と word append だけを担当する。Phase 1 artifact word は `0 <= word <= 0x7fffffff` の nonnegative 31-bit 値に限定し、4 byte 目の high bit が立つ入力は `WordHighBitUnsupported`、4 byte を読めない入力は `UnexpectedEnd` として分けて拒否する。writer は負の word を `NegativeWordUnsupported` として拒否し、vector push 失敗を `PushFailed(StdErrorKind)` として保持する。これにより canonical key payload codec と `.neplproof` reader が byte/word 規則を重複実装しない。

`memo_trait_canonical_key_payload_codec.nepl` は shared word codec を import し、byte offset / word index reader と word writer を委譲するようにした。canonical key payload codec は引き続き payload schema、kind、argument range、materialized canonical key tree への復元を担当し、低水準 byte-to-word authority は shared codec に閉じる。

`memo_trait_proof_reader.nepl` は `.neplproof` header prefix だけを読む。先頭 magic word `792013` を確認した後、artifact schema、canonical payload schema、policy schema、record count、index count の 5 word を読み、既存 `selfhost_memo_trait_neplproof_header_result` へ渡す。magic mismatch、word read failure、artifact header validation failure は `SelfhostMemoTraitNeplProofReaderErrorKind` の別 variant として保持し、schema mismatch や short input を bool / diagnostic string へ潰さない。

この reader は trailing bytes を拒否せず、record vector、sidecar index vector、canonical payload bytes table、proof store preseed、artifact file I/O へ進まない。したがって、この checkpoint は `.neplproof` 全体の受理ではない。proof acceptance authority は引き続き canonical payload decode、payload hash 再計算、policy、proof kind、decoded batch preseed、proof store lookup、producer gate に残る。source text、span、path suffix、display name、diagnostic text、lexeme、store-local id、session-local `TypeId` は header reader / shared word codec の authority にしない。

計算量は header prefix decode が固定 6 word の O(1)、word decode が O(1)、word append が償却 O(1) である。この checkpoint 後も、`.neplproof` record body reader / serializer、persistent stable map / serialized index の実体、generic type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash は未実装である。

### 2026-06-12 memo trait `.neplproof` fixed-width record-only reader checkpoint

`.neplproof` header reader の次段として、artifact schema version 1 の fixed-width record body を typed `SelfhostMemoTraitNeplProofRecord` vector へ decode し、`memo_trait_proof_decoded.nepl` の decoded artifact owner へ渡す record-only reader を追加した。

この reader は 31 word の record layout を schema として doc comment に固定する。offset `0..3` は canonical payload schema / fingerprint schema / fingerprint root hash / canonical payload hash、`4..11` は `MemoKey` / `MemoValue` source identity、`12..16` は solver policy、`17` は stored proof kind、`18..20` は aggregate field evidence、`21..24` は Copy / Drop / Eq / Hash proof status、`25` は hazard evidence、`26..29` は key/value result、`30` は record payload hash である。

reader は header の `record_count` を唯一の件数 authority とし、bytes length から record 件数を推定しない。`record_count` は外部入力なので `16384` 件を超えた場合は `RecordCountLimitExceeded` で拒否する。現 slice の public API は record-only stream を対象にするため、`index_count != record_count` を `IndexCountUnsupported` として拒否し、record table 直後に余分な word がある場合は `TrailingBytes` として fail-closed にする。serialized index section と canonical payload bytes section は後続の別 API と schema 境界で読む。

raw tag は上位へ流さず reader 内の dedicated helper で typed enum へ戻す。source kind、stored proof kind、field evidence、proof status、hazard、`Result unit SelfhostMemoTraitRejectKind` の tag/payload は、それぞれ未知 tag、`Ok` に残った reject payload、field range 不整合を typed `SelfhostMemoTraitNeplProofReaderErrorKind` として返す。proof record 自体は `selfhost_memo_trait_neplproof_record_result` を必ず通し、decoded artifact owner は `selfhost_memo_trait_neplproof_decoded_artifact_from_records` で作る。

binary reader は full canonical key producer を直接 import しない。serialized fingerprint parts は `memo_trait_proof_artifact.nepl` の `selfhost_memo_trait_neplproof_record_key_from_parts_result` へ委譲する。これにより、reader hot path は canonical key projection / source registry / proof store lookup へ広がらず、artifact schema validation boundary だけを使う。

この reader は proof acceptance authority ではない。generated sidecar index は decoded artifact owner の lookup narrowing 用であり、index hit、record payload hash、fingerprint hit のいずれかだけで proof を受理しない。canonical payload decode、payload hash 再計算、materialized key existence、policy、proof kind、decoded batch preseed、proof store lookup、producer gate は後続境界で再検査する。proof-store import は typed proof payload constructors のためだけに使い、lookup / push / preseed / stable materialization API は呼ばない。

性能面では、当初の深い nested match chain を record word vector loop と stage0 field-value projection loop へ置き換えた。record decode は record 数 n、固定 word 幅 w=31 に対して O(n * w)、つまり schema version 1 では O(n) である。stage0 fixture writer も field ordinal から値を返す関数と線形 push loop に分け、selfhost compiler の prefix/typecheck 探索空間を広げる大きな分岐式を避ける。

この checkpoint 後も、canonical payload bytes section reader、`.neplproof` serializer、persistent stable map / serialized index の実体、generic type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash は未実装である。

### 2026-06-13 memo trait `.neplproof` serializer checkpoint

`.neplproof` Phase 1 writer boundary として、header word、fixed-width record table entry、canonical payload bytes section entry を書く serializer を追加した。

serializer は writer であるが、proof acceptance authority ではない。header は `selfhost_memo_trait_neplproof_header_result`、record key/body は `selfhost_memo_trait_neplproof_record_key_result` と `selfhost_memo_trait_neplproof_record_result` に必ず戻し、既存 artifact schema validator を通過した typed payload だけを bytes へ投影する。payload hash、record payload hash、fingerprint、index hit は serializer の受理 authority ではなく、reader / payload reader / preseed / proof store / producer gate が後続で再検査する。

byte/word の低水準処理は `memo_trait_artifact_word_codec.nepl` の共有 31-bit word writer へ委譲する。serializer は独自の endian writer や bit operation を持たない。Phase 1 では serialized index table をまだ持たないため、header validation 後に `index_count == record_count` だけを受理し、それ以外は `IndexCountUnsupported` として fail-closed にする。これは将来の serialized index table を隠す暫定設計ではなく、index section 未実装時の明示的な schema boundary である。

record writer は reader が読む schema version 1 の 31 word layout と同じ順序で typed enum を artifact word / payload word へ戻す。source kind、stored proof kind、aggregate field evidence、Copy / Drop / Eq / Hash proof status、hazard、`Result unit SelfhostMemoTraitRejectKind` は各 helper で明示的に投影し、未知 ordinal は `RecordWordOrdinalInvalid` として閉じる。writer は `SelfhostTypeId`、`SelfhostCanonicalTypeKeyId`、proof store stable identity、source text、span、path suffix、display name、diagnostic text、lexeme を永続 authority にしない。

owner contract は、output `Vec u8` を serializer が消費し、validation 失敗や内部 ordinal failure では partial output owner を閉じる形にした。borrowed payload bytes は caller が所有し、serializer は payload bytes owner を閉じない。payload byte copy では `v::get` が `None` を返す impossible short read も typed `PayloadByteMissing` として扱い、output owner だけを閉じる。

stage0 smoke は serializer の public header / record / payload entry writer を経由して bytes を作り、同じ bytes を payload reader に渡して round-trip する。これにより writer と reader の layout drift、shared word codec drift、payload length prefix drift を実行例で固定する。source policy は reader / payload reader / serializer / preseed の順序、proof-store lookup / push / preseed へ進まないこと、line count / doc comment length cap を入れないことを固定する。

計算量は header write が固定 word 数の O(1)、record write が fixed width 31 word の O(1)、payload entry write が payload byte 長 b に対して O(b) である。FileSystem、source scan、module graph、proof store lookup、preseed、diagnostic rendering は行わない。cache に頼る前に serializer 自体の探索範囲を typed payload の線形投影へ閉じている。

この checkpoint 後も、persistent stable map / serialized index の実体、generic type argument identity、Copy / Drop / Eq / Hash pure evidence、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash は未実装である。

### 2026-06-13 memo trait `.neplproof` serialized index codec checkpoint

`.neplproof` の fixed-width record table と canonical payload section の間に、serialized sidecar index table を接続した。layout は header、record table、serialized index table、payload section の順であり、index entry は schema version 1 で 4 word、`canonical fingerprint schema`、`canonical fingerprint root hash`、`record ordinal`、`record payload hash` を持つ。

reader は既存の record-only API を互換境界として残し、新しい `selfhost_memo_trait_neplproof_reader_decoded_artifact_from_indexed_record_bytes_result` で serialized index table 付き prefix を読む。record-only API は引き続き `index_count == record_count` と trailing bytes を検査するが、indexed API は `index_count` 件の index entry を読み、`memo_trait_proof_decoded.nepl` の `selfhost_memo_trait_neplproof_decoded_artifact_from_record_and_index_tables` へ渡す。

serialized index entry の fingerprint scalar から typed fingerprint payload を作る責務は reader ではなく `memo_trait_proof_artifact.nepl` に置いた。`selfhost_memo_trait_neplproof_index_entry_from_parts_result` は record key の `*_from_parts_result` と同じ schema boundary であり、reader hot path が canonical key producer module へ直接依存しないようにする。これにより、DAG と authority が `artifact schema -> reader -> decoded artifact -> payload reader -> serializer/preseed` の方向に保たれる。

payload reader は payload section の開始位置を indexed prefix の byte count から計算する。ただし、この byte count は payload reader 側の unchecked arithmetic ではなく、reader 側の `selfhost_memo_trait_neplproof_reader_indexed_prefix_byte_count_result` を通して取得する。これにより、過大な `record_count` / `index_count` は prefix owner の確保や copy に進む前に fail-closed になり、reader 本体の input-size boundary と payload reader の offset 計算が同じ規則を共有する。indexed prefix 自体の decode は reader の indexed API へ委譲する。serializer は header と record table の後に public `selfhost_memo_trait_neplproof_serializer_index_entry_result` で serialized index entry を書き、その後に canonical payload entry を書く。stage0 は serializer が作った indexed bytes を payload reader へ渡す round-trip を実行する。

この index table は探索範囲を fingerprint group へ狭めるための artifact metadata であり、proof acceptance authority ではない。canonical payload hash、canonical fingerprint、policy、proof kind、stored proof payload、record payload hash、proof store relation、producer gate は後続の payload reader / preseed / proof store が再検査する。source text、span、path suffix、display name、diagnostic text、lexeme、session-local `SelfhostTypeId`、store-local `SelfhostCanonicalTypeKeyId`、fingerprint hit 単独、record payload hash 単独、index hit 単独は authority にしない。

計算量は、indexed prefix read が record 数 n と index 数 m に対して O(n + m)、index entry write が O(1)、payload section decode が payload byte 総量 b と canonical key tree 総 node/edge 数 k に対して O(b + k) である。FileSystem、source scan、module graph、proof store lookup、preseed、diagnostic rendering はこの codec path では行わない。

この checkpoint 後も、永続 artifact 用 stable map、generic type argument identity、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash は未実装である。

### 2026-06-13 memo trait `.neplproof` persistent stable map checkpoint

`memo_trait_proof_stable_map.nepl` を追加し、decoded `.neplproof` record vector から persistent stable map entry vector を作る producer と、canonical fingerprint + canonical payload hash による candidate range lookup boundary を定義した。

stable map entry は `canonical_fingerprint`、`canonical_payload_hash`、`policy`、`record_ordinal`、`record_payload_hash` を持つ。primary sort key は `(canonical_fingerprint.schema_version, canonical_fingerprint.root_hash, canonical_payload_hash)` であり、同じ key の entry は `record_ordinal` 昇順で連続する。`policy` は後続の policy equality のために entry に保持するが、sort key でも proof acceptance authority でもない。

lookup はまず `selfhost_memo_trait_neplproof_stable_map_order_result` で sorted order を検査し、その後 half-open lower-bound binary search と collision group count に進む。これにより、reader / preseed は artifact record 全体ではなく `O(log m + c)` の stable-key candidate group だけを後続検査へ渡せる。`c` は同じ canonical fingerprint と canonical payload hash を持つ candidate 数である。

producer は `selfhost_memo_trait_neplproof_stable_map_entry_from_record_result` を通して record key と record body を artifact validator へ再投入し、さらに stable map entry validator で ordinal / record payload hash 境界を確認する。不正 record、placeholder payload hash、schema mismatch は typed enum error として fail-closed に返す。Phase 1 producer は insertion sort 相当で O(n^2) だが、public contract は persistent binary writer / stable map codec が O(n log n) または O(n) の構築へ置換できる形にしている。

この stable map は proof acceptance ではない。map hit、fingerprint hit、canonical payload hash hit、record payload hash hit だけで proof を受理する経路は持たない。canonical payload decode、payload hash 再計算、policy equality、proof kind、proof store preseed / lookup、producer gate は後続の decoded artifact / preseed / proof store boundary が再検査する。`SelfhostCanonicalTypeKeyId`、`SelfhostTypeId`、`SelfhostNamedTypeId`、source text、span、path suffix、display name、diagnostic text、lexeme は stable map key、sort key、tie-break authority に入れない。

source policy は `nodesrc/test_selfhost_memo_trait_proof_stable_map_contract.js` で、facade re-export、source list 登録、typed entry / range / error enum、record validator への委譲、lower-bound lookup、producer output order validation、proof store / decoded / reader / serializer への逆依存禁止、session-local id / source authority 禁止、line count / doc comment length cap 禁止を固定する。

この checkpoint 後も、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash は未実装である。

### 2026-06-13 memo trait generic type argument identity checkpoint

`memo_trait_type_argument_identity.nepl` を追加し、generic instantiation の型引数列を session-local `SelfhostTypeId` vector ではなく、canonical key projection、stable nominal key table、canonical fingerprint、canonical payload hash から作る stable identity へ正規化する境界を定義した。

`SelfhostMemoTraitStableTypeArgumentIdentityEntry` は argument ordinal、canonical fingerprint、canonical payload hash を持つ。`SelfhostMemoTraitStableTypeArgumentIdentity` は entry vector と schema 付き aggregate hash を一緒に保持する。hash-only API も full identity producer を通ってから entry owner を閉じるため、hash-only path と entry-preserving path の authority は一致する。ただし aggregate hash は compact lookup key であり、最終的な同一性 authority ではない。後続 artifact は ordered entry vector と schema 付き hash を一緒に確認する。

この identity は proof acceptance authority ではない。MemoKey / MemoValue proof の受理、policy equality、producer gate、proof store lookup、decoded `.neplproof` candidate の検証は後続 boundary が再検査する。source text、span、path suffix、display name、diagnostic text、lexeme、session-local `SelfhostTypeId`、store-local `SelfhostCanonicalTypeKeyId` は accepted authority にしない。出力 struct にもそれらの local id を保存しない。

拒否理由は `SelfhostMemoTraitStableTypeArgumentIdentityErrorKind` に閉じた。type argument missing、canonical key projection failure、fingerprint rejection、payload rejection、entry push failure、identity hash placeholder を bool や diagnostic string に潰さず返す。stage0 smoke は empty accepted identity、single accepted identity、ordered two-argument accepted identity、argument order sensitivity、missing nominal key、duplicate nominal key、type parameter unsupported、function type unsupported を public producer 経由で確認する。

計算量は、型引数数を n、各 canonical key tree の大きさを k、stable nominal key table の record 数を m とすると最悪 O(n * k * m) である。FileSystem、source scan、module graph、proof store lookup、Resource IR、backend codegen はこの identity producer では行わない。

source policy は `nodesrc/test_selfhost_memo_trait_type_argument_identity_contract.js` で、facade re-export、source list 登録、canonical key / payload producer への委譲、typed error enum、entry / aggregate hash schema、argument order fold、stage0 fail-closed paths、checker / HIR / Resource IR / backend への逆依存禁止、source-display authority 禁止、line count / doc comment length cap 禁止を固定する。

この checkpoint 後も、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、re-export / import graph / public non-trait declaration を含む full public surface hash は未実装である。

2026-06-13 MemoKey / MemoValue operation proof table checkpoint では、`stdlib/neplg2/core/ty/ty/memo_trait_operation_proof.nepl` を追加し、Copy / Drop / Eq / Hash の operation proof status を session-local `SelfhostTypeId` table から `SelfhostMemoTraitAggregateProof` へ渡す境界を作った。この module は `SelfhostMemoTraitAggregateProofStatus` を bool や文字列に潰さず、`Proven`、`Missing`、`Impure`、`Unknown` のまま producer gate へ運ぶ。

`SelfhostMemoTraitOperationProofTable` は永続 artifact authority ではない。`.neplmeta`、`.neplproof`、cross-arena cache、serialized canonical key の代わりにはならず、現在の type arena session の中だけで使う。record 欠落は `Proven` に補完せず、Copy / Drop / Eq / Hash がすべて `Missing` の record として aggregate proof に渡す。producer gate は evidence record を作ったうえで、この status を `key_result` / `value_result` の `CopyProofMissing`、`EqProofImpure`、`HashProofUnknown` などの typed side rejection へ変換する。同じ TypeId の record が複数ある場合は first-wins で続行せず、`DuplicateRecord` として aggregate proof construction の前で拒否する。

この checkpoint は proof status の運搬と producer 接続であり、trait impl table、method body purity、Drop なし proof、Eq / Hash の純粋性検査、recursive field traversal、cycle boundary はまだ実装しない。型が `SelfhostTypeArena` に存在しない場合は `MissingTypeRecord` で拒否し、fake TypeId の operation proof record だけで accepted aggregate proof を作らない。source policy と doctest は duplicate record rejection、fake TypeId rejection、missing record fail-closed を実行経路として固定する。source policy はさらに facade re-export、source list 登録、checker / HIR / Resource IR / backend への逆依存禁止、proof store / artifact / codec への依存禁止、wildcard arm 禁止、line count / doc comment length cap 禁止を固定する。

2026-06-13 MemoKey / MemoValue recursive aggregate traversal checkpoint では、`stdlib/neplg2/core/ty/ty/memo_trait_recursive_aggregate.nepl` を追加し、aggregate field closure を cycle-safe に走査する境界を作った。

この module は accepted proof を作らない。`SelfhostMemoTraitRecursiveAggregateSummary` は root、到達 aggregate 数、field edge 数、最大 depth だけを保持し、Copy / Drop / Eq / Hash proof status、hazard proof、producer record、proof store record を保持しない。`SelfhostMemoTraitRecursiveAggregateErrorKind` は `MissingTypeRecord`、`TargetNotAggregate`、`LayoutRejected(SelfhostMemoTraitLayoutEvidenceErrorKind)`、`FieldRecordMissing`、`UnsupportedFieldType`、`CycleDetected`、`DepthLimitReached`、`StackPushFailed(StdErrorKind)` を typed enum として持ち、layout validator や Vec push の失敗を bool や diagnostic string に潰さない。

走査は `memo_trait_layout.nepl` の public layout validator だけを通って field range を取得する。primitive field は leaf として扱うが、root primitive は aggregate target ではないため拒否する。Named / Applied field は再帰的に visit し、Parameter field は `LayoutRejected(GenericArgumentUnsubstituted)`、Function field は `UnsupportedFieldType` として fail-closed にする。cycle 判定は現在の ancestry stack に限定し、sibling branch の再登場は global visited first-wins として拒否しない。`max_depth` は root depth 0 を含む fuel であり、`depth > max_depth` で `DepthLimitReached` を返す。

stage0 smoke は accepted root->primitive、nested accepted root->child->primitive、self-cycle、depth limit、primitive target、missing TypeId、parameter field rejection、function field rejection を public entry 経由で確認する。source policy は facade re-export、source list 登録、`layout < producer < recursive aggregate < operation proof` の順序、proof store / operation proof / HIR / Resource IR / backend / source text authority への依存禁止、typed error equality の wildcard 禁止、line count / doc comment length cap 禁止を固定する。

最適化の扱いは二段階に分ける。今固定したのは、後続 producer gate が依存する traversal contract、typed error taxonomy、cycle / depth / unsupported field の fail-closed 境界であり、これは後から崩すと accepted proof の意味が変わるため今必要な設計である。一方で、layout lookup の indexing、sibling branch の summary memoization、persistent artifact との共有、branch 間再走査の削減は summary と error の public 契約を変えずに後から差し替えられるため、ある程度の時間超過を理由にこの stage を止めず、次は producer gate / Copy-Drop-Eq-Hash pure evidence へ進める。

2026-06-13 MemoKey / MemoValue recursive producer input checkpoint では、`stdlib/neplg2/core/ty/ty/memo_trait_recursive_producer.nepl` を追加し、recursive aggregate traversal の成功を producer gate の実入力へ接続した。この boundary は `memo_trait_recursive_aggregate.nepl`、`memo_trait_layout.nepl`、`memo_trait_operation_proof.nepl`、`memo_trait_producer.nepl` を読むが、proof store、artifact reader / serializer、canonical key codec、HIR、Resource IR、backend へ依存しない。

重要な境界は、traversal summary を accepted proof として扱わないことである。`SelfhostMemoTraitRecursiveAggregateSummary` の `field_count` は closure 全体の edge 数であり、producer gate が読む root aggregate の direct field range とは一致しない場合がある。そのため `selfhost_memo_trait_recursive_producer_aggregate_proof_result` は、recursive traversal が `Ok` になった後で root の field evidence を `selfhost_memo_trait_layout_evidence_for_type_result` から再取得し、その field evidence と operation proof table の Copy / Drop / Eq / Hash status を `SelfhostMemoTraitAggregateProof` へ合流する。

`selfhost_memo_trait_recursive_producer_record_result` は最終的に既存 `selfhost_memo_trait_aggregate_proof_to_record` を呼ぶ。`SelfhostMemoTraitEvidenceRecord` を独自生成したり、consumer evidence table へ直接 push したりしない。失敗は `SelfhostMemoTraitRecursiveProducerErrorKind` に閉じ、`RecursiveRejected(SelfhostMemoTraitRecursiveAggregateErrorKind)`、`LayoutRejected(SelfhostMemoTraitLayoutEvidenceErrorKind)`、`OperationRejected(SelfhostMemoTraitOperationProofErrorKind)`、`ProducerRejected(SelfhostMemoTraitEvidenceProduceRejectKind)` を区別する。operation proof missing や hazard は producer-level error ではなく、record の key/value side payload に typed rejection として残る。

stage0 smoke は root の record 化、self-cycle、operation proof missing side payload、hazard side payload を public boundary 経由で確認する。source policy は、summary counter を producer field range に流用しないこと、producer gate を迂回しないこと、source text / span / path / display name / diagnostic / lexeme を authority にしないこと、行数 / doc comment 長制限を追加しないことを固定した。この checkpoint 後も、Copy / Drop / Eq / Hash pure evidence の実計算、full public surface hash、persistent stable map / serialized index、generic instantiation artifact 接続は後続 slice として残る。

2026-06-13 MemoKey / MemoValue operation solver checkpoint では、`stdlib/neplg2/core/ty/ty/memo_trait_operation_solver.nepl` を追加し、type arena と layout evidence から session-local operation proof table 1 件を作る境界を実装した。`memo_trait_operation_proof.nepl` は引き続き table transport と producer 合流だけを担当し、solver 本体は新 module に分離する。

public entry の `selfhost_memo_trait_operation_solver_table_for_type_result` は `SelfhostTypeArena`、`SelfhostMemoTraitLayoutEvidenceTable`、root `SelfhostTypeId`、`max_depth` を受け取る。最初に `selfhost_memo_trait_recursive_aggregate_result` を呼び、cycle、depth limit、function field、unsubstituted parameter field、missing type record を `SelfhostMemoTraitOperationSolverErrorKind::RecursiveRejected` として fail-closed にする。recursive gate が成功した後だけ operation table を確保し、layout validator の `Known(range)` から field type を読む。record 計算 helper は module 内部に閉じ、公開 caller が recursive gate を迂回して operation proof record を得る経路は作らない。

field type read は `memo_trait_layout.nepl` に追加した `selfhost_memo_trait_layout_field_type_at_result` だけを通る。この accessor は `Known(range)` を想定しつつも range と index を再検査し、`idx` を source position や table absolute index として扱わない。返す authority は arena-local `SelfhostTypeId` だけであり、field name、span、source text、diagnostic string を proof authority にしない。

primitive policy は enum match で固定した。`unit`、`bool`、`i32`、`u8`、`char` は Copy / Drop / Eq / Hash を `Proven` とする。`f32` は value primitive として Copy / Drop は `Proven` だが、MemoKey の Eq / Hash 仕様が未固定なので Eq / Hash を `Unknown` として producer gate で fail-closed にする。nested aggregate は同じ layout evidence table から子 aggregate record を再帰計算し、子の Copy / Drop / Eq / Hash status を親へ merge する。function、parameter、`str`、`i64`、`f64`、`never`、`error` はこの slice では推測で accepted にせず `Unknown` に畳む。

stage0 smoke は empty aggregate、i32 field aggregate、f32 field aggregate、missing layout、operation table lookup、nested i32 aggregate fold、nested f32 aggregate fold、nested child layout missing rejection、recursive cycle rejection を public boundary 経由で確認する。source policy は facade re-export、source list 登録、`operation_proof < operation_solver < recursive_producer` の順序、recursive gate 付き public entry、record lower helper の非公開、accepted evidence record / producer acceptance helper の非生成、layout field accessor 経由、global visited first-wins 禁止、proof store / artifact / reader / serializer / preseed / canonical key / checker / HIR / Resource IR / backend への逆依存禁止、source-derived authority 禁止、typed error equality の wildcard 禁止、line count / doc comment length cap 禁止を固定する。この checkpoint 後の残件は、trait impl table、method body purity、Drop なし proof、key/value 別 operation requirement 分離、full public surface hash、persistent stable map / serialized index、generic instantiation artifact 接続である。

2026-06-13 MemoKey / MemoValue public surface hash typed input checkpoint では、`stdlib/neplg2/core/check/module/memo_trait_public_surface_hash.nepl` に full public surface hash 用の ordered typed input boundary を追加した。

`SelfhostMemoTraitPublicSurfaceHashInputKind` は、現在受理できる local memo trait pair を `LocalMemoTrait(kind)` として保持し、後続 slice が渡す `DependencyModule`、`ReExport`、`PublicFunction`、`PublicStruct`、`PublicEnum`、`PublicImpl` を同じ enum 上に予約する。checker module はこれらを source path、lexeme、span から自分で解決しない。import graph、re-export projection、dependency public surface hash は loader / module graph の authority であり、この module は caller supplied typed input を fold するだけである。

`SelfhostMemoTraitPublicSurfaceHashInputItem` は `kind`、ordered `ordinal`、`visibility`、stable `payload_hash`、caller supplied `dependency_public_surface_hash` に authority を限定する。source text、syntax range、lexeme、path suffix、display name、diagnostic text は hash material ではない。`SelfhostMemoTraitPublicSurfaceHashInputTable` は Phase 1 では local `MemoKey` / `MemoValue` pair の fixed 2 slot table だが、後続の full public surface normalizer は同じ item schema を可変長 table に広げる。

既存の seed table folding は、seed table を直接 fold するのをやめ、`selfhost_memo_trait_public_surface_hash_input_table_from_seed_table_result` で ordered input table に変換してから `selfhost_memo_trait_public_surface_hash_input_table_result` を呼ぶ。full input schema code は `212203` に固定し、旧 seed 直結 helper と旧 seed 直結 domain code は残さない。

この checkpoint は full public surface normalizer ではない。public function / struct / enum / impl signature、dependency module identity、re-export chain、stable nominal key、serialized `.neplmeta` 入力は後続 slice の責務である。今回固定したのは、後続 normalizer が stable input を渡すための受け口と、source-derived authority へ戻らない hash folding contract である。

2026-06-13 MemoKey / MemoValue public surface input accumulator checkpoint では、上記の ordered typed input boundary を full public surface normalizer が使える accumulator API へ拡張した。

`SelfhostMemoTraitPublicSurfaceHashInputAccumulator` は、`count` と `folded_hash` だけを持つ fold state である。`selfhost_memo_trait_public_surface_hash_input_accumulator_push_result` は次に受け取る item の `ordinal` が `count + 1` と一致する場合だけ item hash を fold する。欠番、重複、順序入れ替えは `PublicSurfaceInputOrdinalMismatch` で拒否する。`selfhost_memo_trait_public_surface_hash_input_accumulator_finish_result` は item が 1 件もない input を `PublicSurfaceInputEmpty` として拒否し、schema code、item count、folded item hash から final public surface hash を作る。

dependency と re-export item には caller supplied typed dependency public surface hash を必須にした。`none` は `DependencyPublicSurfaceHashMissing`、`some 0` は `DependencyPublicSurfaceHashPlaceholder` で拒否する。逆に local memo trait item や public function / struct / enum / impl item に dependency public surface hash が付いた場合は `UnexpectedDependencyPublicSurfaceHash` として拒否する。これにより、import graph や re-export chain を hash module が source path から解決する経路を作らず、loader / module graph 側が検査した typed dependency hash だけを受け取る境界を固定した。

Stage0 smoke は、local `MemoKey` / `MemoValue`、dependency、re-export、public function、public struct、public enum、public impl を同じ accumulator schema で fold できることを確認する。また dependency hash missing、dependency hash placeholder、unexpected dependency hash、ordinal mismatch を typed enum error として確認する。既存の local memo trait pair adapter は fixed 2 slot table の互換 path として残るが、その fold 本体も accumulator API を通る。fixed 2 slot table を可変長 table へ置換することは、item schema と accumulator contract を変えずに後からできる。

この checkpoint でも、re-export / import graph / public non-trait declaration の stable payload を実際に作る normalizer は未実装である。typed item constructor と accumulator helper は module 内部に閉じ、stable facade API へは出さない。次 stage では loader / module graph authority が持つ typed dependency surface、re-export projection、public function / struct / enum / impl signature を `SelfhostMemoTraitPublicSurfaceHashInputItem` stream へ投影する producer を実装し、その producer 用に必要な公開境界を改めて設計する。trait impl table、method body purity、Drop なし proof は operation solver 側の後続 slice として扱う。

2026-06-13 MemoKey / MemoValue public surface normalizer producer checkpoint では、`stdlib/neplg2/core/check/module/memo_trait_public_surface_normalizer.nepl` を追加し、loader / module graph authority と full public surface input composition boundary の間に typed producer を置いた。

`memo_trait_public_surface_hash.nepl` には `selfhost_memo_trait_public_surface_hash_input_items_result` を module 内 fold gate として追加した。この関数は `&Vec SelfhostMemoTraitPublicSurfaceHashInputItem` を受け取り、既存 accumulator へ item を順に渡して finish する。hash module は引き続き loader、VFS、module graph、path map、diagnostic rendering を import しない。source text、span、path、alias、display name、diagnostic text は引数にも hash material にも入らない。この fold gate は、LocalMemoTrait item と normalizer 部分 stream を同じ full input stream へ合成する境界が完成するまで、facade public API としては公開しない。

normalizer module は `Vec SelfhostMemoTraitPublicSurfaceDependencyEvidence`、`Vec SelfhostMemoTraitPublicSurfaceReExportEvidence`、`Vec SelfhostMemoTraitPublicSurfacePublicDeclarationEvidence` を borrowed input として受け取る。dependency / re-export では `module_index` によって `SelfhostModuleGraph` 上の node 存在を確認するが、node の path は returned input item へ写さない。accepted input item に入るのは、kind、ordinal、public visibility、caller supplied stable payload hash、caller supplied dependency public surface hash だけである。実 module の import / re-export / public declaration 件数は固定ではないため、固定個数 helper は stage0 fixture に閉じ、facade public API は任意長 vector 境界にする。

拒否理由は `SelfhostMemoTraitPublicSurfaceNormalizerErrorKind` に閉じた。graph build / lookup failure、dependency public surface hash missing、dependency public surface hash placeholder、stable payload hash placeholder、hash fold rejection を bool や文字列へ潰さない。graph build 失敗は `SelfhostDiagnostic` payload を public error として保持せず、`SelfhostMemoTraitPublicSurfaceGraphRejectionKind` へ写して error と表示境界を分離する。hash fold rejection は後続 full hash fold boundary で `HashRejected(SelfhostMemoTraitPublicSurfaceHashErrorKind)` として payload を保持するため、ordinal mismatch など hash schema 側の拒否理由を失わない。

この checkpoint は public non-trait declaration の stable payload hash を作る本体でも、最終 public surface hash を確定する本体でもない。function signature、struct / enum layout header、impl header、re-export export table、stable nominal key、method body purity、Drop なし proof は後続 stage が別の typed evidence として計算する。今回固定したのは、後続 stage が作った stable payload を loader / graph authority と突き合わせて typed input stream へ渡す境界である。これは後から変えると public surface hash の authority が source text や path に戻りやすいため、試作段階の今決めるべき設計である。一方で stream owner の内部表現、graph lookup の index 化、stage0 fixture の整理は、公開 evidence / error / partial stream contract を変えずに後から最適化できる。

2026-06-13 MemoKey / MemoValue public surface seed and partial stream composer checkpoint では、`memo_trait_public_surface_hash.nepl` に `selfhost_memo_trait_public_surface_hash_from_seed_table_and_partial_items_result` を追加し、local `MemoKey` / `MemoValue` seed table と normalizer が作る partial input stream を同じ full public surface input stream へ合成する境界を作った。

この public API は `SelfhostMemoTraitStableSourceSeedTable` と `&Vec SelfhostMemoTraitPublicSurfaceHashInputItem` だけを受け取る。hash module は normalizer、loader、VFS、module graph、path map、diagnostic rendering を import しない。seed table は既存の `selfhost_memo_trait_public_surface_hash_input_table_from_seed_table_result` で LocalMemoTrait item へ変換し、partial stream は borrowed vector として読む。accepted hash material は item kind、ordinal、visibility、payload hash、dependency public surface hash に限定され、source text、span、path、alias、display name、diagnostic text は API 引数にも hash input にも入らない。

composer は partial stream を再採番しない。LocalMemoTrait の `MemoKey` / `MemoValue` item が ordinal 1 / 2 を持つため、partial stream の先頭は ordinal 3 でなければならない。target ordinal ごとに seed item と partial item から候補を 1 件だけ選び、欠番、重複、順序の取り違えは `PublicSurfaceInputOrdinalMismatch` で fail-closed にする。dependency public surface hash の欠落、placeholder、local item への余分な dependency hash は既存 input item validation が `DependencyPublicSurfaceHashMissing`、`DependencyPublicSurfaceHashPlaceholder`、`UnexpectedDependencyPublicSurfaceHash` として返す。

現実装は target ordinal ごとに partial stream を走査するため O(n^2) である。この計算量は公開契約ではなく現状の実装詳細であり、後で sorted index や merge cursor に置き換えられる。置換時も public item schema、borrowed partial stream API、typed error contract、source-derived authority を使わない境界は変えてはならない。この checkpoint 後の残件は、public function signature、struct / enum layout header、impl header、re-export export table、stable nominal key の実 stable payload producer、trait impl table、method body purity、Drop なし proofを接続することである。

2026-06-13 MemoKey / MemoValue public declaration payload producer checkpoint では、`memo_trait_public_surface_normalizer.nepl` に `SelfhostMemoTraitPublicSurfacePublicDeclarationPayloadInput` と内部 payload producer を追加し、public function / struct / enum / impl の stable payload hash を caller supplied raw hash ではなく typed component から作る境界へ進めた。

payload input は `kind`、`stable_declaration_key_hash`、`normalized_declaration_shape_hash` を持つ。`kind` は Function / Struct / Enum / Impl の payload domain を分けるために必ず fold へ入れる。`stable_declaration_key_hash` は stable definition key、stable nominal key、stable impl header key のような宣言 identity の typed hash であり、source span や display name ではない。`normalized_declaration_shape_hash` は function signature、struct / enum layout header、impl header のように依存側の型検査や layout 判定へ影響する公開 shape の typed hash である。

producer は `alloc/hash/hash32::mix` を使う fixed-size mixing だけを行い、source text、span、path、alias、display name、diagnostic text、HIR、Resource IR、backend artifact、proof store、serialized artifact を読まない。stable key が 0 の場合は `StableDeclarationKeyHashPlaceholder`、normalized shape が 0 の場合は `NormalizedDeclarationShapeHashPlaceholder`、derived payload が 0 の場合は `DerivedDeclarationPayloadHashPlaceholder` を返す。これにより、placeholder の原因を `StablePayloadHashPlaceholder` へ潰さず、後続の signature / layout / impl header producer がどの component を壊したかを typed enum で追える。

この adapter は module checker facade へ public export しない。現時点の安定した公開境界は、graph authority 付きの `selfhost_memo_trait_public_surface_normalizer_partial_input_items_result` と、hash module 側の seed + partial stream composer である。payload producer は normalizer 内で stage0 と後続 internal orchestration のために使う。上流の function signature / struct layout / enum layout / impl header producer を別 module として公開する段階では、facade public API と DAG を再確認してから安定境界を設計する。

ただし、現 public partial stream API は互換境界として `Vec SelfhostMemoTraitPublicSurfacePublicDeclarationEvidence` を受け取るため、payload provenance を型だけではまだ閉じ切っていない。caller は今回の内部 adapter、または同等の typed signature / layout / impl header producer が作った stable payload だけを渡す必要がある。次 slice では function signature、struct / enum layout header、impl header の実 producer を接続し、public declaration evidence を source-derived raw hash ではなく producer 由来に寄せる。

stage0 smoke は Function / Struct / Enum / Impl の declaration evidence をすべて payload producer 経由で作る。さらに stable key placeholder と normalized shape placeholder を別々の typed error として確認する。source policy は typed payload input、kind / schema fold、原因別 placeholder error、facade 非公開、source/span/path/diagnostic authority 禁止、line count / doc comment length cap 禁止を固定した。

計算量は declaration 1 件あたり O(1) である。vector producer に接続した場合、normalizer 全体は dependency / re-export / public declaration evidence 件数 k に対して O(k) のまま残る。stable nominal key table lookup、real function signature normalization、real struct / enum layout header normalization、real impl header normalization はまだ後続 stage の責務であり、今回の producer はそれらの output hash を安全に受ける境界である。

### Phase 11: Backend

- Wasm codegen を完成させる。
- `.neplobj` direct-call fragment cache を backend 入力 cache として使う。
- LLVM backend は CLI host adapter と分けて段階的に実装する。

Issue slice:

- Wasm type/import/function section
- memory/data/table/export section
- direct-call fragment replay
- Wasm validation and diagnostic
- LLVM IR emission boundary

Performance acceptance:

- backend は Resource proof を再実行しない。
- `.neplobj` fragment hit では function body lowering を再利用する。

### Phase 12: Bootstrap

- Rust compiler と self-host compiler の output parity test を追加する。
- self-host compiler で `stdlib/neplg2/` 自身を compile する。
- CI に bootstrap check を段階導入する。

Issue slice:

- lexer/parser Rust parity
- diagnostic JSON parity
- public surface hash parity
- Resource proof summary parity
- Wasm semantic output parity
- deterministic emission

Performance acceptance:

- RPN cold base compile を標準 benchmark とする。
- stage timing JSON を Discord report と CI artifact に含める。
- compile-time regression threshold を設け、cache hit だけで regression を隠さない。

### 2026-06-13 MemoKey / MemoValue key-value operation requirement checkpoint

`memo_trait_producer.nepl` の producer boundary を、record 全体の生成可否と `MemoKey` / `MemoValue` side の受理可否を分ける形へ更新した。`SelfhostMemoTraitEvidenceProduceRejectKind` は missing type、non-aggregate type、field layout 欠落、invalid range、generic argument unsubstituted、cycle limit のように evidence record を作れない構造的な失敗だけを表す。Copy / Drop / Eq / Hash proof と hazard は `SelfhostMemoTraitRejectKind` に追加した typed variant へ移し、`SelfhostMemoTraitEvidenceRecord.key_result` / `value_result` に保存する。

key side は Copy、Drop、Eq、Hash、hazard-free を要求する。value side は cache から owned / copied value として返せることを確認するため Copy、Drop、hazard-free を要求し、Eq / Hash は要求しない。これにより、Hash proof が unknown の aggregate でも `MemoValue` としては受理し、`MemoKey` としてだけ `HashProofUnknown` で拒否できる。hazard は key と value の両方に適用し、external handle、owner token、public mutable state、cache reference escape、unknown hazard を key にする経路も value と同じく拒否する。

producer stage0、operation proof stage0、recursive producer stage0 は、operation proof missing や hazard を producer-level `Err` としてではなく、生成された evidence record の side payload として検査するように更新した。これにより recursive producer は traversal / layout / operation table / recordization のどこで外側エラーになったかを保ちつつ、operation proof status の未証明は consumer predicate が読む `Result unit SelfhostMemoTraitRejectKind` として保持する。

この checkpoint では trait impl table、method body purity、Drop なし proof の実計算はまだ行わない。今回固定したのは、後続 solver が計算した operation status を key/value 別の受理条件へ畳み込む record boundary である。nested aggregate traversal の memoization、operation fold の重複削減、layout lookup index 化は、この side payload contract を変えずに後から最適化できる。

### 2026-06-13 MemoKey / MemoValue operation classifier checkpoint

`memo_trait_operation_classifier.nepl` を追加し、Copy / Drop / Eq / Hash の trait application shape を operation evidence producer が消費できる `SelfhostMemoTraitOperationClassifierEvidence` へ変換する境界を作った。

この classifier は、operation kind enum だけを信用しない。入力は operation trait source identity、type argument count、trait application shape hash であり、current trusted operation source registry に含まれる source identity であることと、source identity から再導出した shape hash が入力 shape hash と一致することを確認してから evidence を返す。source text、span、lexeme、display name、diagnostic、module path、HIR、Resource IR、backend artifact、proof store record は authority にしない。

`SelfhostMemoTraitOperationClassifierEvidence` は classifier module が所有し、operation evidence producer はそれを一方向に import して消費する。これにより、classifier から下流 producer への逆依存を避ける。producer 側の impl header trait application shape との再照合は残し、classifier は operation trait application の分類、producer は impl header / target type shape / method body purity / Drop evidence の整合性検査、という二段階の fail-closed boundary にする。

Phase 1 の current registry は compiler-known prepared fingerprint である。これは actual public surface materializer ではないため、後続では Copy / Drop / Eq / Hash trait definition source identity を public surface hash、stable trait definition key、normalized signature evidence から作る。type argument count は 0 だけを受理し、generic operation trait application は binder / argument identity が接続されるまで拒否する。

### 2026-06-13 MemoKey / MemoValue operation impl table checkpoint

`memo_trait_operation_impl_table.nepl` を追加し、public surface normalizer が作る trait impl 候補列と operation classifier / operation evidence producer の間に置く checker-layer 探索境界を作った。

この table は `SelfhostTypeId` と `SelfhostMemoTraitOperationEvidenceKind` を key として一意の `SelfhostMemoTraitOperationImplCandidate` を探す。同じ key の候補が複数ある場合は record order による first-wins にせず、`CandidateDuplicate` として fail-closed に拒否する。候補が無い場合は `CandidateMissing` であり、証拠欠落を `Proven` に見せない。`idx < len` なのに `Vec::get` が `None` を返す table 不整合は `CandidateReadFailed(idx)` として扱い、既に見つけた候補を成功へ畳まない。

candidate は typed public impl header input、typed trait application input、resolved target type shape evidence、method body evidence、Drop evidence を持つ。見つかった candidate は `selfhost_memo_trait_operation_classifier_evidence_result` で shape-bound classifier evidence に変換し、さらに `selfhost_memo_trait_operation_evidence_producer_status_result` と `selfhost_memo_trait_operation_evidence_producer_record_result` を通す。table 自身は operation evidence record を直接組み立てず、producer の public impl header / target shape / classifier shape / operation kind / method-drop evidence gate を迂回しない。

この checkpoint でも method body purity checker、Drop なし proof generator、Copy / Drop / Eq / Hash pure evidence の実計算、generic impl binder / bound detailed evidence、full public surface orchestration は未実装である。table はそれらが作った typed evidence を運ぶ探索境界であり、source text、span、lexeme、display name、diagnostic、module path、HIR、Resource IR、backend artifact、proof store、`.neplproof` reader / serializer、canonical key codec を authority にしない。

現実装の lookup は candidate 数 n に対して O(n) である。これは公開契約ではなく、後で sorted index や operation bucket へ置き換えられる。置換時も typed candidate schema、duplicate fail-closed、classifier / producer 経由、source-derived authority 非使用という契約は変えてはならない。

### 2026-06-13 MemoKey / MemoValue operation purity gate checkpoint

`memo_trait_operation_purity_gate.nepl` を追加し、method body / Drop 実装の typed effect fact を operation evidence producer が読む `SelfhostMemoTraitOperationMethodBodyEvidence` と `SelfhostMemoTraitOperationDropEvidence` へ写す checker-layer 境界を作った。

この gate は、実 method body checker や Drop resolver ではない。入力は `SelfhostEffectKind` と `SelfhostEffectEscapeState` を持つ typed check fact であり、source text、span、lexeme、display name、diagnostic text、module path、HIR、Resource IR、backend artifact、proof store record を authority にしない。実際の expression tree 解析、Drop impl 探索、generic binder / bound evidence、private cache effect masking、Resource IR escape proof、full public surface orchestration は後続 stage の責務である。

`SelfhostEffectKind::Pure` は method body evidence の `Pure` または Drop evidence の `PureDrop` へ写す。`InternalAlloc` は `SelfhostEffectEscapeState::NoEscapeProven` の場合だけ pure evidence へ畳み、`MayEscape` は `Impure` / `ImpureDrop`、`NotApplicable` は `Unknown` にする。`UnsafeMemory`、`ExternalIo`、`Nondet` は `Impure` / `ImpureDrop` であり、trusted private boundary や memo cache masking をこの gate が推測しない。

operation 別の matrix もここで固定した。`Eq` / `Hash` は method body evidence を必要とし、`NotRequired` は `MethodBodyEvidenceRequired` として拒否する。`Copy` / `Drop` に method body fact が渡された場合は `UnexpectedMethodBodyEvidence` として拒否する。`Drop` は Drop evidence を必要とし、`NotRequired` は `DropEvidenceRequired` として拒否する。`Copy` / `Eq` / `Hash` に Drop fact が渡された場合は `UnexpectedDropEvidence` として拒否する。`Missing` / `Unknown` / `Impure` は producer error に潰さず、producer / solver へ status として残す。

`memo_trait_operation_impl_table.nepl` には `selfhost_memo_trait_operation_impl_candidate_from_checks_result` を追加し、full orchestration が method / Drop check fact から candidate を作る場合に purity gate を必ず通る入口を用意した。既存の raw candidate constructor は typed evidence を束ねる低層 constructor として残るが、後続 orchestration は check fact から evidence enum を手で組み立てない。

この checkpoint 後の残件は、actual method body checker、Drop impl resolver、Copy / Drop / Eq / Hash pure evidence の実計算、generic impl binder / bound detailed evidence、full public surface orchestration、private cache / private state effect masking、prechecked artifact との接続である。lookup index 化や nested operation fold の重複削減は、今回固定した typed effect fact -> evidence enum の contract を変えずに後から最適化できる。

### 2026-06-13 MemoKey / MemoValue Drop impl resolver checkpoint

`memo_trait_operation_drop_impl_resolver.nepl` を追加し、complete な typed Drop impl fact table から `SelfhostMemoTraitOperationDropCheck` を作る checker-layer 境界を接続した。

この resolver は `SelfhostMemoTraitOperationDropImplSurfaceState` を `Complete` / `Missing` / `Unknown` に分ける。`Complete` の場合だけ fact table を走査し、対象 `SelfhostTypeId` の Drop impl fact が 0 件なら `DropImplAbsent`、1 件なら `DropImplPresent(effect, escape)`、2 件以上なら `RecordDuplicate` として fail-closed に拒否する。`Missing` / `Unknown` surface では lookup miss を no-drop proof にせず、それぞれ `Missing` / `Unknown` check を返す。

Drop impl fact は `SelfhostTypeId`、`SelfhostEffectKind`、`SelfhostEffectEscapeState` だけを authority にする。source text、span、lexeme、display name、diagnostic text、module path、HIR、Resource IR、backend artifact、proof store record は authority にしない。actual Drop body checker、Resource IR no-escape proof、generic impl binder、trait coherence、full public surface orchestration、private cache effect masking は後続 stage の責務である。

source policy は facade 非公開、`selfhost_ty_sources.js` 非登録、forbidden layer import 禁止、complete-only absent proof、Missing / Unknown surface preservation、duplicate fail-closed、owner recovery、line / doccomment length cap 禁止を固定した。

この checkpoint 後の残件は、actual method body checker、Drop body effect checker と Resource IR escape proof の実接続、Copy / Drop / Eq / Hash pure evidence の実計算、generic impl binder / bound detailed evidence、full public surface orchestration、private cache / private state effect masking、prechecked artifact 接続である。lookup index 化や Drop impl fact table の sorted index 化は、今回固定した complete surface state、duplicate fail-closed、typed fact authority の contract を変えずに後から行える最適化として扱う。

### 2026-06-13 MemoKey / MemoValue operation body check resolver checkpoint

`memo_trait_operation_body_check_resolver.nepl` を追加し、method body resolver と Drop impl resolver の結果を operation evidence producer 直前の `SelfhostMemoTraitOperationBodyChecks` へ束ねる checker-layer 境界を作った。

この resolver は証明生成層ではない。operation evidence record、producer input、aggregate proof status は作らず、operation kind に対する method body check と Drop impl check の組だけを返す。`Copy` は method body も Drop impl proof も不要なので両方を `NotRequired` にする。`Eq` / `Hash` は method body resolver から得た check と Drop `NotRequired` を組み合わせる。`Drop` は method `NotRequired` と Drop impl resolver から得た check を組み合わせる。この matrix は exhaustive match に閉じ、後続の full orchestration が手書きで `NotRequired` や `Missing` を混ぜる退行を防ぐ。

`Present` / `Missing` / `Unknown` / `DropImplAbsent` / `DropImplPresent` は既存 resolver から得る。この module が直接構築する check は operation 上不要な `NotRequired` だけである。`Missing` / `Unknown` は status check であり、resolver error ではない。method body resolver または Drop impl resolver が duplicate、table read failure、push failure などの構造的不整合を返した場合だけ、payload 付き wrapper error として fail-closed に伝播する。

accepted authority は session-local `SelfhostTypeId`、operation enum、typed effect check table、typed surface state に限定する。source text、span、lexeme、display name、diagnostic text、module path、HIR、Resource IR、backend artifact、proof store record、public surface hash は authority にしない。facade にはまだ re-export せず、`selfhost_ty_sources.js` にも登録しない。full public surface orchestration、actual expression method body effect summary producer、Drop body Resource IR proof、generic impl binder、private cache effect masking は後続 stage の責務である。

source policy は、facade 非公開、ty source 非登録、forbidden layer import 禁止、operation matrix、typed check pair、payload 付き wrapper error、status と structural error の分離、wildcard-free equality、line / doc comment length cap 禁止を固定した。

`SelfhostMemoTraitOperationBodyChecks` は resolver の返り値 payload 型として public にするが、この module 自体は facade-private に留める。generic pair constructor は private `fn` とし、repo 内の外部 module が `SelfhostMemoTraitOperationBodyChecks` を直接構築して operation matrix を迂回する退行は source policy で禁止した。stage0 smoke は `Missing` だけでなく method / Drop の `Unknown` pass-through も確認し、incomplete surface を accepted proof へ畳まない境界を実行例で固定する。

この checkpoint 後の残件は、actual expression method body effect summary producer、Drop body effect checker と Resource IR escape proof の実接続、Copy / Drop / Eq / Hash pure evidence の実計算、generic impl binder / bound detailed evidence、full public surface orchestration、private cache / private state effect masking、prechecked artifact 接続である。method body fact table lookup と Drop impl fact table lookup の sorted index 化は、今回固定した check pair contract を変えずに後から行える最適化として扱う。

### 2026-06-13 MemoKey / MemoValue method body effect checker checkpoint

`memo_trait_operation_method_body_effect_checker.nepl` を追加し、typed HIR expression root から method body の `SelfhostEffectKind` / `SelfhostEffectEscapeState` summary を作る checker-layer 境界を作った。

この module は method body fact、operation evidence record、aggregate proof status を作らない。`Call` payload に保存済みの effect enum と、`Block` / `If` / `Call` の child range だけを authority として読み、call name、source text、span、diagnostic text、module path から effect を再分類しない。`Error` expression は pure success へ畳まず typed error にし、missing root / malformed child range / fuel exhaustion も bool や表示文字列ではなく enum error として返す。

`InternalAlloc` はこの module では `Pure` に mask しない。HIR call payload から来た `InternalAlloc` は Resource IR no-escape proof をまだ持たないため `NotApplicable` escape のまま summary に残し、後続 purity gate が fail-closed に扱える状態を保つ。`UnsafeMemory` / `ExternalIo` / `Nondet` も同じく summary payload として保持し、表示用 severity や文字列 code に変換しない。

source policy は `nodesrc/test_selfhost_memo_trait_operation_method_body_effect_checker_contract.js` で固定した。facade 非公開、`selfhost_ty_sources.js` 非登録、Resource IR / backend / proof store / operation resolver / body check resolver / evidence producer への依存禁止、fact / evidence / check pair 構築禁止、HIR payload variant の明示 match、line count / doc comment length cap 禁止、unwrap / unreachable shortcut 禁止を確認する。

この checkpoint 後の残件は、summary を complete surface の method body fact table へ接続する orchestration、Drop body effect checker、Resource IR no-escape proof、Copy / Drop / Eq / Hash pure evidence の実計算、generic impl binder / bound detailed evidence、full public surface orchestration、private cache / private state effect masking、prechecked artifact 接続である。HIR traversal の explicit stack 化、subtree memoization、child range lookup index 化は summary / error contract を保って後からできる最適化として扱う。

### 2026-06-13 MemoKey / MemoValue method body fact producer checkpoint

`memo_trait_operation_method_body_fact_producer.nepl` を追加し、typed HIR method body effect summary と complete surface method body fact table が読む fact constructor を接続する checker-layer 境界を作った。

この producer は fact table owner を消費しない。HIR root から summary を作る場合は `memo_trait_operation_method_body_effect_checker` を呼び、summary から fact を作る場合は `memo_trait_operation_method_body_resolver` の `selfhost_memo_trait_operation_method_body_fact_new_result` へ `SelfhostTypeId`、operation kind、effect、escape をそのまま渡す。operation requirement matrix は resolver constructor が持つため、この producer は `Eq` / `Hash` を文字列や method name から推測しない。

error は `EffectCheckRejected` と `FactRejected` の nested typed payload に分けた。HIR root 欠落、fuel exhaustion、`Error` expression、malformed child range は effect checker 側 error として残り、`Copy` / `Drop` など method body fact を作ってはいけない operation は resolver 側 error として残る。どちらも bool や表示文字列へ潰さない。

この module は operation evidence record、method body evidence、Drop evidence、body check pair、aggregate proof status、Resource IR proof、backend artifact、public surface orchestration を作らない。table 追加、duplicate rejection、surface completeness の判断は既存 table push / resolver lookup と次の full orchestration に残す。これにより、effect checker 失敗時に table owner をどう回収するかという不要な所有権問題を作らず、summary から fact への変換を O(1) の pure boundary として扱える。

source policy は `nodesrc/test_selfhost_memo_trait_operation_method_body_fact_producer_contract.js` で固定した。facade 非公開、`selfhost_ty_sources.js` 非登録、forbidden layer import 禁止、fact table owner 消費禁止、resolver lookup 禁止、table / evidence / proof / body check 構築禁止、nested error payload 比較、line count / doc comment length cap 禁止、unwrap / unreachable shortcut 禁止を確認する。

この checkpoint 後の残件は、fact producer result を complete public surface impl candidate 群から method body fact table へ投入する orchestration、Drop body effect checker、Resource IR no-escape proof、Copy / Drop / Eq / Hash pure evidence の実計算、generic impl binder / bound detailed evidence、full public surface orchestration、private cache / private state effect masking、prechecked artifact 接続である。method body fact table lookup の sorted index 化、HIR traversal の explicit stack 化、subtree memoization は、今回固定した typed fact producer contract を保って後から行える最適化として扱う。

### 2026-06-13 MemoKey / MemoValue method body fact table builder checkpoint

`memo_trait_operation_method_body_fact_table_builder.nepl` を追加し、HIR method body root から作った fact producer result を `SelfhostMemoTraitOperationMethodBodyTable` owner へ投入する checker-layer 境界を作った。

この builder は `SelfhostMemoTraitOperationMethodBodyTable` owner を消費し、成功時だけ追加後の table owner を返す。fact producer が失敗した場合は table push へ進んでいないため、builder が未消費 table owner を `selfhost_memo_trait_operation_method_body_table_free` で閉じる。table push が失敗した場合は既存 resolver の `selfhost_memo_trait_operation_method_body_table_push` が `Vec` owner を回収して閉じるため、builder は古い table owner を二重解放しない。`Result::Err` を受け取った caller は、渡した table owner を再利用したり free したりしてはいけない。

error は `FactProducerRejected(SelfhostMemoTraitOperationMethodBodyFactProducerErrorKind)` と `TableRejected(SelfhostMemoTraitOperationMethodBodyResolverErrorKind)` に分けた。HIR effect checker、fact constructor、table push の拒否理由を bool や表示文字列へ潰さず、nested typed payload として保持する。

この module は table lookup、duplicate rejection、surface completeness decision、operation evidence record 作成、method body evidence 作成、Drop evidence 作成、Resource IR proof、backend artifact、proof store、public surface scanning を行わない。full public surface orchestration は、public impl candidate 群から HIR root を得る責務と、この builder を繰り返して complete method body fact table を作る責務を後続 stage で接続する。

source policy は `nodesrc/test_selfhost_memo_trait_operation_method_body_fact_table_builder_contract.js` で固定した。facade 非公開、`selfhost_ty_sources.js` 非登録、forbidden layer import 禁止、producer -> table push の順序、producer error branch の table free、table push error branch の二重解放禁止、resolver lookup 禁止、direct fact struct constructor bypass 禁止、body check / evidence / proof 作成禁止、line count / doc comment length cap 禁止、unwrap / unreachable shortcut 禁止を確認する。

この checkpoint 後の残件は、complete public surface impl candidate 群を走査してこの builder へ入力する full orchestration、Drop body effect checker、Resource IR no-escape proof、Copy / Drop / Eq / Hash pure evidence の実計算、generic impl binder / bound detailed evidence、private cache / private state effect masking、prechecked artifact 接続である。method body fact table lookup の sorted index 化、HIR traversal の explicit stack 化、subtree memoization は、今回固定した builder contract を保って後から行える最適化として扱う。

### 2026-06-13 MemoKey / MemoValue method body fact table inputs checkpoint

`memo_trait_operation_method_body_fact_table_inputs.nepl` を追加し、complete public surface scanner が後続 stage で作る typed method body root input 列を、既存 `memo_trait_operation_method_body_fact_table_builder` へ順番に渡す checker-layer batch boundary を作った。

`SelfhostMemoTraitOperationMethodBodyFactBuildInput` は `SelfhostTypeId`、`SelfhostMemoTraitOperationEvidenceKind`、`SelfhostHirExprId`、fuel だけを保持する。これは上流 scanner / classifier がすでに決定した typed input であり、この module は source text、span、lexeme、display name、diagnostic text、module path から operation や root を推測しない。input table は `Vec` owner table として別に持ち、batch build は input table を borrow で読むだけなので、caller が success / failure のあとで input table owner を閉じる。

output の `SelfhostMemoTraitOperationMethodBodyTable` は `selfhost_memo_trait_operation_method_body_fact_table_build_from_inputs_result` が消費する。全 input が成功した場合だけ完成後の table owner を返す。input read failure では、まだ builder に渡していない output table owner をこの module が閉じて `InputReadFailed(index)` を返す。builder rejection では builder が output table owner cleanup を完結させるため、この module は二重解放せず、`BuilderRejected(payload)` として失敗 index と nested builder error を保持する。NEPLg2.1 の enum variant payload は単一なので、`SelfhostMemoTraitOperationMethodBodyFactTableInputsBuilderRejected` struct に `index` と `error` を入れる。

この module は fact constructor、direct table push、duplicate lookup、surface completeness decision、method body evidence 作成、Drop evidence 作成、operation evidence record 作成、Resource IR proof、backend artifact、proof store、public surface scanning を行わない。full orchestration は、public impl candidate 群から typed input table を作る責務と、この input batch boundary を呼んで complete method body fact table を作る責務を後続 stage で接続する。

source policy は `nodesrc/test_selfhost_memo_trait_operation_method_body_fact_table_inputs_contract.js` で固定した。facade 非公開、`selfhost_ty_sources.js` 非登録、forbidden layer import 禁止、direct producer / fact constructor / table push bypass 禁止、resolver lookup 禁止、body check / evidence / proof 作成禁止、input read failure branch の output table free、builder rejection branch の二重解放禁止、input table owner を caller が保持する stage0 cleanup、line count / doc comment length cap 禁止、unwrap / unreachable shortcut 禁止を確認する。

この checkpoint 後の残件は、complete public surface impl candidate 群から method body fact build input table を作る scanner / full orchestration、Drop body effect checker、Resource IR no-escape proof、Copy / Drop / Eq / Hash pure evidence の実計算、generic impl binder / bound detailed evidence、private cache / private state effect masking、prechecked artifact 接続である。method body fact table lookup の sorted index 化、input table の sorted index 化、HIR traversal の explicit stack 化、subtree memoization は、今回固定した typed input batch contract を保って後から行える最適化として扱う。

### 2026-06-13 MemoKey / MemoValue method body fact input scan checkpoint

`memo_trait_operation_method_body_fact_input_scan.nepl` を追加し、complete public impl surface 由来の typed operation impl records から、Eq / Hash method body だけを `SelfhostMemoTraitOperationMethodBodyFactBuildInputTable` へ変換する scanner boundary を作った。

scan record は `SelfhostTypeId`、`SelfhostMemoTraitOperationEvidenceKind`、optional `SelfhostHirExprId`、fuel だけを持つ。accepted authority はこの typed field に限定し、source text、span、lexeme、display name、diagnostic text、module path、method name string を読まない。Eq / Hash は `some(root)` を要求して output build input に入り、Copy / Drop は `none` の場合だけ skip される。Eq / Hash の root 欠落と Copy / Drop の root 混入は、それぞれ index / operation payload 付き typed error として返す。

scan API は output build input table owner を内部で作り、成功時だけ caller へ返す。source record table は borrow で読むため caller が閉じる。失敗時は partial output owner を scanner が閉じるか、既存 output input table push boundary が閉じる。これにより、public impl candidate 列の走査と、builder batch boundary の owner lifecycle が分離される。

この scanner は HIR effect checker、fact producer、fact table builder、method body resolver lookup、Drop resolver、purity gate、operation impl candidate table、Resource IR proof、backend artifact、proof store、public surface hash を実行しない。actual public impl AST scanning、impl header / trait application materialization、Drop body effect checker、Resource IR no-escape proof、generic impl binder、private effect masking は後続 stage の責務として残す。

source policy は `nodesrc/test_selfhost_memo_trait_operation_method_body_fact_input_scan_contract.js` で、facade 非公開、forbidden layer import 禁止、operation matrix、cleanup、direct fact constructor bypass 禁止、line count / doc comment length cap 禁止を固定する。

この checkpoint 後の残件は、method body fact input scan と batch builder を実 public surface impl candidate materialization へ接続する full orchestration、Drop body effect checker、Resource IR no-escape proof、Copy / Drop / Eq / Hash pure evidence の実計算、generic impl binder / bound detailed evidence、private cache / private state effect masking、prechecked artifact 接続である。method body fact table lookup の sorted index 化、input table の sorted index 化、scanner source table の operation bucket 化、HIR traversal の explicit stack 化、subtree memoization は、今回固定した typed scan/input contract を保って後から行える最適化として扱う。

### 2026-06-13 MemoKey / MemoValue method body fact orchestrator checkpoint

`memo_trait_operation_method_body_fact_orchestrator.nepl` を追加し、typed scan record table から scan boundary と batch build boundary を順番に呼び、complete surface 用 `SelfhostMemoTraitOperationMethodBodyTable` owner を作る checker-layer orchestration boundary を作った。

この orchestrator は public impl AST materializer ではない。accepted authority は `SelfhostMemoTraitOperationMethodBodyFactInputScanRecordTable` の typed field、borrow された `SelfhostHirModule`、既存の input scan / batch build boundary だけである。source text、span、lexeme、display name、diagnostic text、module path、method name string、public surface hash から operation や HIR root を推測しない。また trait classifier、purity gate、operation impl table、Drop resolver、Resource IR proof、backend artifact、proof store も実行しない。

owner contract は、source scan record table を caller-owned borrow とし、scan 成功後の build input table owner は orchestrator が消費する形で固定した。batch build success、batch build rejection、output fact table allocation failure のどの場合でも build input table owner は orchestrator が閉じる。batch build rejection 時の output fact table owner は既存 batch boundary が cleanup 済みであるため、orchestrator は二重解放しない。success の場合だけ completed fact table owner を caller へ返す。

error は `InputScanRejected(SelfhostMemoTraitOperationMethodBodyFactInputScanErrorKind)`、`OutputTableAllocFailed(StdErrorKind)`、`BatchBuildRejected(SelfhostMemoTraitOperationMethodBodyFactTableInputsErrorKind)` に分けた。scan / allocation / batch build の失敗を bool や表示文字列へ潰さず、後続診断が必要な nested payload を保持する。

source policy は `nodesrc/test_selfhost_memo_trait_operation_method_body_fact_orchestrator_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、forbidden layer import 禁止、direct fact constructor / low-level builder / direct table push bypass 禁止、body check / evidence / aggregate proof / proof store 作成禁止、source-derived authority 禁止、owner cleanup、nested error equality、line count / doc comment length cap 禁止、unwrap / unreachable / fallback / first-wins 禁止を確認する。

subagent review では Bohr が、actual public surface materialization へ直接進む前に typed materialization input table を置くべきだと指摘した。今回の checkpoint は full public surface materializer ではなく、既存 scan / batch boundary を owner-safe に接続する前段である。次 slice は `memo_trait_operation_impl_candidate_builder.nepl` のような connector とし、public materializer が将来作る typed record table から method body fact table、body check resolver、operation impl candidate table までを source/path/display に触れず接続する。

この checkpoint 後の残件は、actual public impl candidate materializer が typed record table を作ってこの boundary へ渡す candidate builder / full public surface materialization、Drop body effect checker / Resource IR no-escape proof、Copy / Drop / Eq / Hash pure evidence の実計算、generic impl binder / bound detailed evidence、private cache / private state effect masking、prechecked artifact 接続である。method body fact table lookup の sorted index 化、method body fact build input table の sorted index 化、scan source table の bucket 化、HIR traversal の explicit stack 化 / subtree memoization / child range lookup index 化は、今回固定した typed input / owner / error contract を保って後から行える最適化として扱う。

### 2026-06-13 MemoKey / MemoValue operation impl candidate builder checkpoint

`memo_trait_operation_impl_candidate_builder.nepl` を追加し、actual public impl materializer が後続 stage で作る typed record table から、method body fact table、body check resolver、operation impl candidate table までを接続する checker-layer builder boundary を作った。

`SelfhostMemoTraitOperationImplCandidateBuilderInput` は、`SelfhostTypeId`、operation kind、typed public impl header input、typed trait application input、resolved target type shape evidence、optional method body root、fuel を保持する。accepted authority はこの typed field と borrow された `SelfhostHirModule` だけであり、source text、span、lexeme、display name、diagnostic text、module path、method name string、trait name string から operation や method root を推測しない。builder input table は caller-owned borrow とし、builder は success / failure のどちらでも input table と HIR module を閉じない。

builder はまず Drop preflight を実行し、Phase 1 では Drop operation input を `DropOperationUnsupportedUntilResourceProof(index, operation)` で明示的に拒否する。これは method body root が混入した Drop input でも同じであり、method fact scan や purity gate へ進めない。Drop input が無い場合だけ、input record を `SelfhostMemoTraitOperationMethodBodyFactInputScanRecordTable` へ写し、既存 `memo_trait_operation_method_body_fact_orchestrator` を通して complete method body fact table owner を作る。空 table から `NoDropRequired` を推測しないだけでなく、未証明の `Unknown` evidence もこの builder では作らない。Drop なし証拠と pure Drop proof は上流 Resource proof stage が明示的に作り、後続 slice で candidate 化する。

output table 作成では、同じ `SelfhostTypeId` と operation kind の candidate がすでに存在する場合に `CandidateDuplicate` として拒否し、record order による first-wins を避ける。Copy / Eq / Hash の candidate は `selfhost_memo_trait_operation_body_check_resolve_result` と `selfhost_memo_trait_operation_impl_candidate_from_checks_result` を順に通して作る。producer input や operation evidence record への変換は下流の impl table / producer API の責務であり、この builder では行わない。

source policy は `nodesrc/test_selfhost_memo_trait_operation_impl_candidate_builder_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、forbidden layer import 禁止、typed input payload、nested typed error、method fact orchestrator 経由、Drop unsupported typed error、producer input / evidence record / aggregate status 生成禁止、duplicate rejection、owner cleanup、line count / doc comment length cap 禁止を確認する。

この checkpoint 後の残件は、actual public impl candidate materializer / full public surface materialization、Drop body effect checker / Resource IR no-escape proof、Copy / Drop / Eq / Hash pure evidence の実計算、generic impl binder / bound detailed evidence、private cache / private state effect masking、prechecked artifact 接続である。operation impl table lookup の sorted index 化、method body fact table lookup の sorted index 化、method body fact build input table の sorted index 化、scan source table の operation bucket 化、HIR traversal の explicit stack 化 / subtree memoization / child range lookup index 化は、今回固定した typed input / owner / error contract を保って後から行える最適化として扱う。

### 2026-06-14 MemoKey / MemoValue public impl materializer checkpoint

`memo_trait_operation_public_impl_materializer.nepl` を追加し、actual public surface materializer が後続 stage で作る typed public impl record table から、既存 `memo_trait_operation_impl_candidate_builder` の builder input table と candidate table owner へ接続する checker-layer materializer boundary を作った。

`SelfhostMemoTraitOperationPublicImplMaterializerRecord` は、`SelfhostTypeId`、public impl header input へ写す module fingerprint / declaration ordinal / visibility / impl kind / target shape / trait application shape / type parameter count / bound count、classifier input へ渡す operation trait source identity / type argument count、optional HIR method body root、fuel だけを保持する。operation kind は record field として持たず、`selfhost_memo_trait_operation_classifier_evidence_result` が返す `classifier.operation` から builder input へ渡す。これにより、trait name string、method name string、source text、span、lexeme、display name、diagnostic text、module path から operation を推測しない契約を固定した。

materializer public API は `source` record table と `SelfhostHirModule` を borrow で読み、どちらも閉じない。内部で作る `SelfhostMemoTraitOperationImplCandidateBuilderInputTable` owner は、candidate builder success / failure のどちらでも materializer が閉じる。source read failure と classifier rejection では partial builder input table owner を materializer が閉じ、builder input push rejection では既存 builder input table push boundary が owner cleanup を完結するため二重解放しない。success の場合だけ output `SelfhostMemoTraitOperationImplTable` owner を caller へ返す。

この materializer は method body fact、Drop proof、operation evidence record、aggregate proof status を作らない。Drop record は classifier shape を確認したうえで builder input に写し、Resource proof 未接続の Phase 1 では既存 builder の `DropOperationUnsupportedUntilResourceProof` へ送る。空 table から `NoDropRequired` を合成せず、未証明の `Unknown` evidence もこの module では作らない。

source policy は `nodesrc/test_selfhost_memo_trait_operation_public_impl_materializer_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、Resource IR / backend / proof store / canonical key / evidence producer / purity gate / body check / method body / Drop resolver への直接 import 禁止、classifier-derived operation use、direct impl table push / producer input / evidence record / aggregate proof status 作成禁止、temporary builder input owner cleanup、line count / doc comment length cap 禁止を確認する。

この checkpoint 後の残件は、actual AST / typed HIR public impl scanner と full public surface materialization、Drop body effect checker / Resource IR no-escape proof、Copy / Drop / Eq / Hash pure evidence の実計算、generic impl binder / bound detailed evidence、private cache / private state effect masking、prechecked artifact 接続である。operation impl table lookup の sorted index 化、materializer record table の operation bucket 化、method body fact table lookup の sorted index 化、HIR traversal の explicit stack 化 / subtree memoization / child range lookup index 化は、今回固定した typed input / owner / error contract を保って後から行える最適化として扱う。

### 2026-06-14 MemoKey / MemoValue public impl scanner checkpoint

`memo_trait_public_impl_scanner.nepl` を追加し、`SelfhostModuleAst` の public `ImplDecl` と resolver / lowering stage が作った typed public impl record を 1-origin の public declaration ordinal で照合する checker-layer boundary を作った。

この scanner の authority は、AST 側では public declaration item の存在、visibility、declaration kind、public declaration ordinal に限定する。target type、trait application、operation trait source、method body root、fuel は resolver-owned typed record から受け取り、source text、span、lexeme、display name、diagnostic text、module path、method name string、trait name string、HIR traversal から復元しない。

accepted path では、typed record を `memo_trait_public_impl_header` で public impl header evidence へ通し、同じ record から `SelfhostMemoTraitOperationPublicImplMaterializerRecordTable` へ渡す materializer input を作る。これにより public surface normalizer 用の public declaration evidence と、operation materializer 用 typed record table を同じ association boundary から得る。operation kind は scanner では決めず、後続 materializer / classifier が shape-bound trusted source identity で決める。

error は `TypedRecordMissing`、`TypedRecordDuplicate`、`TypedRecordUnmatched`、`HeaderRejected`、AST alignment error、owner allocation / push error に分ける。module 全体の ordinal alignment は output owner allocation と header validation より先に preflight する。public impl に record が無い場合と、public impl ではない ordinal に record が向いた場合を stage0 fixture で分離し、first-wins や余剰 record の黙殺を禁止する。さらに余剰 record と invalid header が混在する場合は `TypedRecordUnmatched` を先に返す mixed regression を固定した。scanner output owner は success の場合だけ caller へ渡し、failure の partial output は scanner が閉じる。

source policy は `nodesrc/test_selfhost_memo_trait_public_impl_scanner_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、token seed scan 側の public impl unsupported 維持、forbidden layer import 禁止、source-derived authority 禁止、operation evidence / candidate / proof / method body / Drop body 作成禁止、stage0 smoke、line count / doc comment length cap 禁止を確認する。

この checkpoint 後の残件は、scanner output を full public surface composer へ接続して complete public surface state を作る orchestration、Drop body effect checker / Resource IR no-escape proof、Copy / Drop / Eq / Hash pure evidence の実計算、generic impl binder / bound detailed evidence、private cache / private state effect masking、prechecked artifact 接続である。public impl record lookup の ordinal index 化、materializer record table の operation bucket 化、method body fact table lookup の sorted index 化、HIR traversal の explicit stack 化 / subtree memoization / child range lookup index 化は、今回固定した association / owner / error contract を保って後から行える最適化として扱う。

### 2026-06-14 MemoKey / MemoValue public impl surface orchestrator checkpoint

`memo_trait_public_impl_surface_orchestrator.nepl` を追加し、public impl scanner output を full public surface hash と operation impl candidate table へ同じ boundary で接続した。

この orchestrator は scanner output の `public_declarations` を `memo_trait_public_surface_normalizer` へ渡し、caller supplied local `MemoKey` / `MemoValue` seed table と normalizer partial stream を `memo_trait_public_surface_hash` で full public surface hash へ畳む。その後、同じ scanner output の `operation_records` を `memo_trait_operation_public_impl_materializer` へ渡し、candidate builder / operation impl table へ接続する。public surface 側が拒否した場合は operation materializer へ進まない。

owner contract は、scanner output を borrow で消費する lower API と、AST / resolver records から scanner output owner を作る upper API に分けた。`from_scanner_output_result` は normalizer が返す partial item owner を hash success / hash rejection / materializer success / materializer rejection のすべてで閉じる。`from_ast_records_result` は scanner output owner を success / downstream rejection のどちらでも閉じ、scanner rejection では downstream stage へ進まない。

accepted authority は scanner output の typed field、caller supplied dependency / re-export evidence、caller supplied seed table、module graph、typed HIR module borrow、既存 normalizer / hash / materializer boundary に限定した。source text、span、lexeme、display name、diagnostic text、module path、method name string、trait name string から public surface、operation、HIR root を推測しない。Drop Resource proof、PrivateCache / PrivateState masking、operation evidence record、aggregate proof status、backend artifact、proof store はこの module の責務ではない。

stage0 doctest は compile time を抑えるため、accepted path だけを real scanner / normalizer / hash / materializer の owner path で実行する。scanner / normalizer / materializer の rejection branch は下位 module の stage0 と source policy に任せ、この module では synthetic typed `Result::Err` payload を使って wrapping contract を確認する。これは `resource_static_initialized_moves` の探索範囲を不必要に広げないための fixture 設計であり、public API や error payload の契約を弱めるものではない。

source policy は `nodesrc/test_selfhost_memo_trait_public_impl_surface_orchestrator_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、Resource IR / backend / proof store / canonical key / producer / purity / method body / Drop resolver import 禁止、normalizer -> hash -> materializer の順序、owner cleanup、source-derived authority 禁止、line count / doc comment length cap 禁止を確認する。

この checkpoint 後の残件は、complete public surface state から Copy / Drop / Eq / Hash pure evidence を実計算する aggregate solver 接続、Drop body effect checker / Resource IR no-escape proof、generic impl binder / bound detailed evidence、PrivateCache / PrivateState effect masking、prechecked artifact 接続である。operation impl table lookup の sorted index 化、materializer record table の operation bucket 化、public impl record lookup の ordinal index 化、normalizer / hash composer の sorted index / merge cursor 化、stage0 fixture のさらなる分割は、今回固定した typed input / owner / error contract を保って後から行える最適化として扱う。

### 2026-06-14 MemoKey / MemoValue public impl operation evidence connector checkpoint

`memo_trait_public_impl_operation_evidence_connector.nepl` を追加し、public impl surface state または operation impl table から root aggregate 用の `SelfhostMemoTraitOperationEvidenceTable` owner を作る checker-layer connector を接続した。これにより、public impl scanner / surface orchestrator / operation impl table が作った Copy / Drop / Eq / Hash 候補を、既存 operation evidence table と evidence 付き operation solver へ渡す境界ができた。

重要な点は、operation impl table の探索結果をこの connector が直接 final proof にしないことである。`CandidateMissing` だけは evidence table に何も push せず、後続の `selfhost_memo_trait_operation_evidence_record_for_type_or_missing_result` が `Missing` status へ畳む。`CandidateDuplicate`、table read / push failure、classifier rejection、producer rejection は typed `OperationImplRejected` として返し、Missing や Unknown に潰さない。Drop candidate が無い場合に `NoDropRequired` や `PureDrop` を合成せず、Drop なし proof / pure Drop proof は Resource proof stage が明示的に作った evidence だけを信用する。

surface state 入口では `public_surface_hash = 0` を拒否する。hash は complete proof ではないが、orchestrator 由来の transport state と手書きの空 struct literal を区別する最低限の境界として扱う。accepted authority は `SelfhostMemoTraitPublicImplSurfaceState` の hash と operation impl table、`SelfhostTypeId`、operation enum、既存 impl table / evidence table / solver API だけであり、source text、span、lexeme、display name、diagnostic text、module path、method name string、trait name string は authority にしない。

solver wrapper は `selfhost_memo_trait_operation_solver_table_for_type_with_operation_evidence_result` を呼ぶだけにした。field traversal、root evidence merge、layout validation、recursive aggregate gate は既存 solver の責務であり、この module は自前で Copy / Drop / Eq / Hash status を合成しない。temporary evidence table owner は solver success / failure のどちらでも connector が閉じる。

source policy は `nodesrc/test_selfhost_memo_trait_public_impl_operation_evidence_connector_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、explicit checker-layer import allow-list、Resource IR / backend / proof store / canonical key / method body / Drop resolver / candidate builder / materializer / scanner import 禁止、CandidateMissing のみ skip、duplicate / classifier / producer typed error、Drop proof 合成禁止、surface hash 0 rejection、solver 委譲、temporary evidence table cleanup、wildcard-free equality、行数 / doc comment 長制限禁止を確認する。

この checkpoint 後の残件は、Drop body effect checker / Resource IR no-escape proof、generic impl binder / bound detailed evidence、PrivateCache / PrivateState effect masking、prechecked artifact 接続である。operation impl table lookup の sorted index 化、operation 別 bucket 化、solver traversal の subtree memoization、stage0 fixture のさらなる分割は、今回固定した typed connector / owner cleanup / error contract を保って後からできる最適化として扱う。

## 2026-06-14 MemoKey / MemoValue Drop impl fact table builder checkpoint

`stdlib/neplg2/core/check/module/memo_trait_operation_drop_impl_fact_table_builder.nepl` を追加し、Drop impl body の typed HIR root から `SelfhostMemoTraitOperationDropImplFact` を作り、complete surface 用の `SelfhostMemoTraitOperationDropImplTable` owner へ投入する checker-layer boundary を作った。

この builder は既存 `memo_trait_operation_method_body_effect_checker` を、Drop impl body に対する typed HIR payload effect summary builder として再利用する。accepted authority は `SelfhostTypeId`、HIR root、fuel、HIR payload の `SelfhostEffectKind` / child range、effect summary の `SelfhostEffectEscapeState` に限定する。source text、span、lexeme、display name、diagnostic text、module path、method name string から Drop impl の存在や purity を推測しない。

owner 境界は destructive builder として固定した。`selfhost_memo_trait_operation_drop_impl_fact_table_builder_push_hir_root_result` は Drop impl fact table owner を消費し、effect checker が失敗した場合は未消費 table owner をこの module が閉じる。resolver table push が失敗した場合は既存 `selfhost_memo_trait_operation_drop_impl_table_push` が `Vec` owner cleanup を担当済みなので、この module は二重解放しない。`Result::Err` を受け取った caller は、渡した table owner を再利用してはいけない。

この checkpoint では Drop evidence、operation evidence record、aggregate proof status、Resource IR no-escape proof、PrivateCache / PrivateState masking、backend artifact、proof store、public surface scanning を作らない。`InternalAlloc` は effect checker が `NotApplicable` escape のまま fact table に保存するため、Resource IR no-escape proof なしに `PureDrop` へ畳まれない。Drop なし proof と pure Drop proof は、後続 Resource proof stage が明示的な typed evidence として作る。

source policy は `nodesrc/test_selfhost_memo_trait_operation_drop_impl_fact_table_builder_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、Resource IR / backend / proof store / canonical key / public surface / evidence producer / operation impl table / purity gate / classifier / materializer / scanner import 禁止、Drop evidence / aggregate proof 合成禁止、`DropImplAbsent` / `NoDropRequired` / `PureDrop` 合成禁止、effect checker -> fact -> resolver table push の順序、effect error branch の table free、table push error branch の二重解放禁止、summary effect / escape の mask なし保存、行数 / doc comment 長制限禁止を確認する。

この checkpoint 後の残件は、actual public impl scanner / materializer から Drop impl body root input をこの builder へ渡す orchestration、Resource IR no-escape proof、pure Drop evidence gate、generic impl binder / bound detailed evidence、PrivateCache / PrivateState effect masking、prechecked artifact 接続である。Drop impl fact table lookup の sorted index 化、HIR traversal の explicit stack 化 / subtree memoization / child range lookup index 化は、今回固定した typed input / owner / error contract を保って後からできる最適化として扱う。

## 2026-06-14 MemoKey / MemoValue public impl Drop fact orchestrator checkpoint

`stdlib/neplg2/core/check/module/memo_trait_operation_public_impl_drop_fact_orchestrator.nepl` を追加し、`SelfhostMemoTraitOperationPublicImplMaterializerRecordTable` から Drop impl fact table owner を作る checker-layer orchestration boundary を作った。

この boundary は public impl materializer record を source authority にしつつ、operation kind は `record.trait_source.operation` を直接信用せず、`SelfhostMemoTraitOperationTraitApplicationInput` を trusted operation classifier に通して得た classifier evidence から決める。classifier evidence が Drop の record だけ `method_body_root` を Drop body root として読み、non-Drop record は skip する。Eq / Hash record が method body root を持つことは正常な入力なので、この Drop 専用 boundary が non-Drop root を Drop root として扱う経路は作らない。

owner 境界は destructive orchestrator として固定した。entry API は fresh な `SelfhostMemoTraitOperationDropImplTable` owner を作り、Drop record を既存 `selfhost_memo_trait_operation_drop_impl_fact_table_builder_push_hir_root_result` へ渡す。source read failure、classifier rejection、Drop root 欠落では未消費 Drop table owner をこの module が閉じる。builder rejection では builder / resolver push boundary が owner cleanup を担当するため、この module は二重解放しない。`source` record table と `module` は borrow であり、この module は閉じない。

この checkpoint では Drop evidence、operation evidence record、aggregate proof status、Resource IR no-escape proof、PrivateCache / PrivateState masking、backend artifact、proof store、public surface hash を作らない。Drop record の body root 欠落は `RequiredDropBodyRootMissing(index)` として fail-closed に返し、`DropImplAbsent`、`NoDropRequired`、`PureDrop` へ畳まない。`InternalAlloc` などの effect は既存 Drop fact builder の summary contract に従って fact table へ保存され、Resource IR no-escape proof なしに pure Drop evidence へ昇格しない。

source policy は `nodesrc/test_selfhost_memo_trait_operation_public_impl_drop_fact_orchestrator_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、Resource IR / backend / proof store / canonical key / public surface / evidence producer / operation impl table / purity gate / body-check resolver / candidate builder / scanner import 禁止、classifier evidence authority、non-Drop skip、Drop root 必須、builder rejection no double-free、accepted table の resolver 経由確認、Drop proof 合成禁止、行数 / doc comment 長制限禁止を確認する。

この checkpoint 後の残件は、Resource IR no-escape proof、pure Drop evidence gate、generic impl binder / bound detailed evidence、PrivateCache / PrivateState effect masking、prechecked artifact 接続である。Drop materializer record table の operation bucket 化、Drop impl fact table lookup の sorted index 化、HIR traversal の explicit stack 化 / subtree memoization / child range lookup index 化は、今回固定した typed source authority / owner / error contract を保って後からできる最適化として扱う。

## 2026-06-14 MemoKey / MemoValue Drop no-escape proof gate checkpoint

`stdlib/neplg2/core/check/module/memo_trait_operation_drop_no_escape_gate.nepl` を追加し、Drop impl fact table に対して、後続 Resource IR stage が作る typed no-escape proof status を適用する checker-layer gate を作った。

この checkpoint では `SelfhostMemoTraitOperationDropImplFact` に `body_root %SelfhostHirExprId` を追加し、Drop body identity を fact table に保持するようにした。`body_root` は HIR payload を走査する authority ではなく、Resource proof が対象にした Drop body と fact table の Drop body を照合するための typed identity payload である。Drop impl resolver、Drop impl fact table builder、operation body check resolver はこの field を保持するように更新した。

2026-06-14 follow-up review では、`SelfhostHirExprId` が HIR module 内の expression table index であり、module をまたいだ安定 identity ではないことを確認した。そのため `SelfhostMemoTraitOperationDropImplFact` と `SelfhostMemoTraitOperationDropNoEscapeProofKey` に `body_module_fingerprint %i32` を追加し、Drop body root を「body module fingerprint + HIR root id」の組として扱うように強化した。これは Resource IR no-escape proof producer の実装ではなく、producer が将来作る proof status を誤った module の同じ root id へ再利用しないための typed origin boundary である。

no-escape proof key は `SelfhostTypeId`、Drop body module fingerprint、Drop body root、`SelfhostEffectKind`、`SelfhostEffectEscapeState` の組である。`SelfhostTypeId` だけ、または `SelfhostTypeId + Drop body root` だけで proof を再利用すると、同じ型に対する別 Drop body、別 module の同じ HIR root index、別 effect summary の proof を流用できてしまうため禁止する。source text、span、lexeme、display name、diagnostic text、module path、payload hash、public surface hash は accepted authority にしない。

`body_module_fingerprint == 0` は placeholder origin として拒否する。Drop impl fact table push は placeholder origin を持つ fact を受け取った時点で table owner を閉じて fail-closed にし、no-escape proof table push も placeholder origin を持つ proof record を保存しない。これにより、module-local HIR root id の衝突を避けるために追加した origin key が、未初期化または未 materialize の placeholder 値によって無効化されることを防ぐ。

gate が proof lookup を行うのは `InternalAlloc + NotApplicable` の fact だけである。matching proof status が `Proven` なら `NoEscapeProven` へ更新し、`Refuted` なら `MayEscape` へ更新する。`Missing` / `Unknown`、または matching proof が無い場合は `NotApplicable` のまま残し、後続 purity gate が fail-closed に扱えるようにする。入力 fact がこの gate より前に `InternalAlloc + NoEscapeProven` を持っていた場合は `UnexpectedPreProvenNoEscape` として拒否し、別 boundary が no-escape を合成する bypass を認めない。`Pure` / `UnsafeMemory` / `ExternalIo` / `Nondet` は proof table を見ずに pass-through し、observable effect を弱めない。

この module は Drop evidence、operation evidence record、aggregate proof status、proof store、PrivateCache / PrivateState masking、generic binder、prechecked artifact を作らない。最終的な `PureDrop` evidence は既存 `memo_trait_operation_purity_gate` が、`DropImplPresent(InternalAlloc, NoEscapeProven)` を入力にしたときだけ作る。したがって、今回の gate は Resource IR no-escape proof producer の代替ではなく、producer が作った typed status を消費する境界である。

source policy は `nodesrc/test_selfhost_memo_trait_operation_drop_no_escape_gate_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、proof key field、duplicate proof rejection、pre-proven input rejection、Missing / Unknown fail-closed、Drop evidence / aggregate proof / proof store 合成禁止、source-derived authority 禁止、Clone / Copy impl の typed-payload-only doc、line count / doc comment length cap 禁止を確認する。

この checkpoint 後の残件は、actual Resource IR no-escape proof producer、pure Drop evidence / operation evidence candidate への orchestration、generic impl binder / bound detailed evidence、PrivateCache / PrivateState effect masking、prechecked artifact 接続である。proof table の sorted index 化、type/module/root bucket 化、Drop impl fact table lookup の sorted index 化、HIR traversal explicit stack 化 / subtree memoization は、今回固定した key equality / origin equality / placeholder rejection / duplicate rejection / fail-closed status contract を保って後から行える最適化として扱う。

## 2026-06-14 MemoKey / MemoValue Drop candidate connector checkpoint

`stdlib/neplg2/core/check/module/memo_trait_operation_drop_candidate_connector.nepl` を追加し、typed public impl materializer record table、Drop impl fact orchestrator、Drop no-escape proof gate、purity gate、operation impl table を接続する checker-layer Drop candidate connector を作った。

この connector は Drop proof producer ではない。Resource IR no-escape proof table は caller が borrow で渡す typed status table であり、この module はその table を生成、永続化、proof store へ投入しない。entry API は output candidate table owner を消費し、materializer record table と HIR module と no-escape proof tableを borrow で読む。内部で作った raw Drop fact table と gated Drop fact table は connector が閉じる。Drop fact build rejection、no-escape gate rejection、classifier rejection、Drop resolver rejection、candidate rejection、duplicate candidate、table push rejection はすべて typed enum error に残し、bool や diagnostic string へ潰さない。

accepted authority は typed materializer record field、trusted operation classifier evidence、Drop fact orchestrator output、no-escape proof gate output、Drop resolver output、purity gate candidate conversion、operation impl table API に限定する。`record.trait_source.operation`、source text、span、lexeme、display name、diagnostic text、module path、method name string、trait name string は authority にしない。non-Drop record は同じ materializer table に混在する正常入力として skip し、Drop record だけを candidate へ進める。

この connector は `DropImplPresent` だけを purity gate へ渡す。Drop resolver が `DropImplAbsent`、`Missing`、`Unknown`、`NotRequired` を返した場合は fail-closed typed error にする。これは partial materializer record table や filtered Drop table の lookup miss から `NoDropRequired` を合成する退行を防ぐためである。`NoDropRequired` は complete public surface 全体の探索結果から作る別 boundary の責務であり、Drop record を読んでいるこの connector の責務ではない。

source policy は `nodesrc/test_selfhost_memo_trait_operation_drop_candidate_connector_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、Resource IR / backend / proof store / canonical key / public surface / scanner / method-body fact / body-check / candidate-builder / PrivateCache / PrivateState import 禁止、operation evidence record / aggregate proof / proof store 合成禁止、production path の `PureDrop` / `NoDropRequired` 直接合成禁止、classifier-derived Drop filter、duplicate candidate probe、DropImplPresent gate、owner cleanup、source-derived authority 禁止、行数 / doc comment 長制限禁止を確認する。

この checkpoint 後の残件は、actual Resource IR no-escape proof producer、complete public surface 由来の no-drop absence proof boundary、operation evidence connector への Drop candidate 統合、generic impl binder / bound detailed evidence、PrivateCache / PrivateState effect masking、prechecked artifact 接続である。materializer record table の operation bucket 化、proof table の sorted index 化、Drop impl fact table lookup の sorted index 化、operation impl table lookup の sorted index 化は、今回固定した typed authority / duplicate rejection / owner / error contract を保って後から行える最適化として扱う。

## 2026-06-14 MemoKey / MemoValue public impl surface Drop candidate connector checkpoint

`stdlib/neplg2/core/check/module/memo_trait_public_impl_surface_drop_candidate_connector.nepl` を追加し、public impl scanner output から full public surface hash と Drop candidate 増補済み operation impl table を同じ checker-layer boundary で作る wrapper を接続した。

この wrapper の目的は、caller が別々に作った `SelfhostMemoTraitPublicImplSurfaceState` と `SelfhostMemoTraitOperationPublicImplMaterializerRecordTable` を混ぜてしまう退行を防ぐことである。公開入口は scanner output、または AST + resolver record table から scanner output を作る entry に限定する。public surface hash は scanner output の `public_declarations` だけを normalizer / hash composer へ渡して作り、operation candidate は同じ scanner output の `operation_records` から作る。Drop candidate 追加も同じ `operation_records` borrow を下位 Drop candidate connector へ渡すため、hash 側と operation 側の origin が分離しない。

base materializer は Resource IR no-escape proof が未接続だった段階の境界なので、Drop record を渡すと fail-closed に拒否する。この wrapper は classifier evidence で `operation_records` を O(m) で走査し、non-Drop record だけを一時 table owner へ写して base materializer に渡す。Drop record は一時 table には入れず、original scanner output の record table を Drop candidate connector へ渡す。一時 table owner は success / materializer rejection の両方で閉じ、Drop candidate connector が受け取った output candidate table owner は下位 connector の owner contract に従って扱うため二重解放しない。

`public_surface_hash` は transport value であり、Drop proof lookup、candidate acceptance、record coverage の authority にしない。この module は Resource IR proof producer、complete no-drop absence proof、operation evidence record、aggregate proof status、proof store、PrivateCache / PrivateState masking、backend artifact、prechecked artifact を作らない。`NoDropRequired` は complete public surface 全体の探索結果から別 boundary が作る proof であり、この wrapper は `PureDrop` / `NoDropRequired` を直接合成しない。

source policy は `nodesrc/test_selfhost_memo_trait_public_impl_surface_drop_candidate_connector_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、checker-layer import allow-list、Resource IR / backend / proof store / canonical key / operation evidence connector / body-check / candidate-builder / method-body / Drop resolver / PrivateCache / PrivateState import 禁止、scanner-output same-origin、hash helper が public declarations だけを読むこと、non-Drop filter、Drop candidate append の順序、public surface hash を proof authority にしないこと、production path の proof / evidence / `PureDrop` / `NoDropRequired` 合成禁止、行数 / doc comment 長制限禁止を確認する。

この checkpoint 後の残件は、actual Resource IR no-escape proof producer、complete public surface 由来の no-drop absence proof boundary、Drop 増補済み surface state を full operation evidence pipeline へ使う上位 orchestration、generic impl binder / bound detailed evidence、PrivateCache / PrivateState effect masking、prechecked artifact 接続である。non-Drop filter の record bucket 化、proof table sorted index 化、operation impl table sorted index 化、stage0 fixture 分割は、今回固定した scanner-output origin / owner / error contract を保って後からできる最適化として扱う。

2026-06-14 follow-up では、Drop 増補済み surface state owner を既存 operation evidence / proof connector へ渡し、success / error の両方で state owner を閉じる上位 orchestration を追加した。この orchestration は proof producer ではなく、`NoDropRequired`、`PureDrop`、method body `Pure`、aggregate `Proven` を合成しない。actual Resource IR no-escape proof producer、complete public surface 由来の no-drop absence proof boundary、generic impl binder / bound detailed evidence、PrivateCache / PrivateState effect masking、prechecked artifact 接続は引き続き後続 slice の責務である。operation impl table lookup の sorted index 化、Drop proof table lookup の index 化、solver traversal memoization は、今回固定した typed authority / owner cleanup contract を保って後から置換できる最適化として扱う。

2026-06-14 Resource no-escape producer checkpoint では、`memo_trait_operation_drop_resource_no_escape_producer.nepl` を追加し、Resource IR 側が作った typed no-escape observation table を既存の `SelfhostMemoTraitOperationDropNoEscapeProofTable` へ変換する checker-layer 境界を接続した。入力 record は `SelfhostTypeId`、body module fingerprint、Drop body root、effect、escape、Resource status を持ち、producer は `InternalAlloc + NotApplicable` だけを受理する。`NoEscapeProven` は gate proof status の `Proven`、`MayEscape` は `Refuted`、`Missing` / `Unknown` は同名 status に写し、未証明 status を pure に mask しない。

この checkpoint は full Resource IR graph traversal ではない。source text、span、display name、module path、public surface hash、payload hash、HIR effect summary だけを authority にして proof を合成する経路は作らない。`body_module_fingerprint == 0`、`effect != InternalAlloc`、`escape != NotApplicable`、同一 key の duplicate observation は fail-closed に拒否する。input table push と producer loop の両方で validation を行うため、table push 以外で作られた malformed table も producer 境界で止まる。

source policy は `nodesrc/test_selfhost_memo_trait_operation_drop_resource_no_escape_producer_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、checker-layer import allow-list、Resource graph internals / backend / proof store / canonical key / public surface / evidence producer / impl table / scanner / materializer / purity gate / PrivateCache / PrivateState import 禁止、typed body identity fields、`InternalAlloc + NotApplicable` validation、duplicate rejection、status mapping、既存 no-escape proof table への一方向変換、`PureDrop` / `NoDropRequired` / aggregate proof / proof store 合成禁止、Result error の bool / string 化禁止、行数 / doc comment 長制限禁止を確認する。

この checkpoint 後の残件は、actual Resource IR graph traversal から `SelfhostMemoTraitOperationDropResourceNoEscapeRecord` table を作る evidence materializer、complete public surface 由来の no-drop absence proof boundary、generic impl binder / bound detailed evidence、PrivateCache / PrivateState effect masking、prechecked artifact 接続である。input observation table と output proof table の sorted index 化、duplicate scan の bucket 化、stage0 fixture 分割は、今回固定した typed key / owner cleanup / fail-closed contract を保って後からできる最適化として扱う。

2026-06-14 follow-up checkpoint では、`memo_trait_operation_drop_resource_no_escape_materializer.nepl` を追加し、future Resource graph traversal が返す typed traversal summary table から既存 producer 用の `SelfhostMemoTraitOperationDropResourceNoEscapeTable` を作る checker-layer materializer boundary を接続した。入力 record は `SelfhostTypeId`、body module fingerprint、Drop body root、effect、escape、traversal status、traversal reason を持ち、materializer は `InternalAlloc + NotApplicable` だけを受理する。`AllTraversedPlacesPrivate` は `NoEscapeProven`、`EscapingPlaceObserved` は `MayEscape`、`ResourceGraphMissing` は `Missing`、`TraversalUnsupported` は `Unknown` に写し、missing / unsupported を pure に mask しない。

この checkpoint も full Resource IR graph traversal ではない。Resource graph owner、Resource proof API、proof table、proof store、public surface、backend artifact、PrivateCache / PrivateState masking には触れず、`SelfhostMemoTraitOperationDropResourceNoEscapeTable` construction だけを担当する。producer module は observation table 型と constructor / push を持つため import するが、source policy で proof conversion API、no-escape proof table API、proof status mapping API の参照を禁止した。`body_module_fingerprint == 0`、`effect != InternalAlloc`、`escape != NotApplicable`、同一 key の duplicate traversal summary は materializer 側でも fail-closed に拒否する。input table push と materializer loop の両方で validation を行うため、table push 以外で作られた malformed table も materializer 境界で止まる。

source policy は `nodesrc/test_selfhost_memo_trait_operation_drop_resource_no_escape_materializer_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、checker-layer import allow-list、Resource graph / Resource proof internals / backend / proof store / canonical key / public surface / evidence producer / impl table / scanner / purity gate / PrivateCache / PrivateState import 禁止、typed body identity fields、typed status / reason enum、`InternalAlloc + NotApplicable` validation、duplicate rejection、status mapping、既存 observation table push への一方向変換、proof table / Drop evidence / aggregate proof / proof store / PrivateCache / PrivateState / prechecked artifact 合成禁止、Result error の bool / string 化禁止、行数 / doc comment 長制限禁止を確認する。

この checkpoint 後の残件は、actual Resource IR graph traversal から `SelfhostMemoTraitOperationDropResourceNoEscapeTraversalRecord` table を作る scanner / evidence collector、complete public surface 由来の no-drop absence proof boundary、generic impl binder / bound detailed evidence、PrivateCache / PrivateState effect masking、prechecked artifact 接続である。traversal summary table、observation table、proof table の sorted index 化、duplicate scan の bucket 化、stage0 fixture 分割は、今回固定した typed key / owner cleanup / fail-closed contract を保って後からできる最適化として扱う。

2026-06-14 traversal collector checkpoint では、`memo_trait_operation_drop_resource_no_escape_traversal_collector.nepl` を追加し、future Resource graph walker が返す typed body / place / edge input から traversal summary table を作る checker-layer collector boundary を接続した。input model は `SelfhostDropResourceGraphBodyRecord`、`SelfhostDropResourceGraphPlaceRecord`、`SelfhostDropResourceGraphEdgeRecord` を持ち、body identity、graph id、effect、escape、graph completeness、place kind、edge kind を typed payload として運ぶ。source text、span、lexeme、display name、module path、public surface hash、payload hash、HIR effect summary だけから no-escape を推測しない。

collector は table output の前に preflight validation を行う。body / place / edge table 全体を検査し、placeholder fingerprint、不正 graph id、不正 place id、不正 edge endpoint id、`effect != InternalAlloc`、`escape != NotApplicable`、duplicate body / place / edge、orphan place / edge、同じ graph 内に存在しない edge endpoint を fail-closed に拒否する。`ClosedForDropBody` かつ place を 1 件以上持つ graph だけが private traversal summary fold へ進み、`ResourceGraphMissing` と `TraversalUnsupported` はそれぞれ同名 summary status として残る。return / public store / external handle place、return / public-store edge は `EscapingPlaceObserved` へ倒し、unsupported place / call boundary は `TraversalUnsupported` へ倒す。

この checkpoint も full Resource IR graph walker 本体ではない。Resource graph owner、Resource proof API、proof table、proof store、public surface、backend artifact、PrivateCache / PrivateState masking には触れない。source policy は `nodesrc/test_selfhost_memo_trait_operation_drop_resource_no_escape_traversal_collector_contract.js` で固定し、facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、explicit import allow-list、forbidden layer import、typed graph record fields、preflight validation、only `ClosedForDropBody` fold、escape sink / unsupported kind fail-closed、既存 traversal table push への一方向出力、proof table / Drop evidence / aggregate proof / proof store / PrivateCache / PrivateState / prechecked artifact 合成禁止、Result error の bool / string 化禁止、行数 / doc comment 長制限禁止を確認する。

この checkpoint 後の残件は、actual Resource IR graph walker が collector 入力の typed body / place / edge stream を生成する scanner、complete public surface 由来の no-drop absence proof boundary、generic impl binder / bound detailed evidence、PrivateCache / PrivateState effect masking、prechecked artifact 接続である。graph lookup index 化、traversal summary table、observation table、proof table の sorted index 化、duplicate scan の bucket 化、stage0 fixture 分割は、今回固定した typed graph input / owner cleanup / fail-closed contract を保って後からできる最適化として扱う。

2026-06-14 Resource graph input scanner checkpoint では、`memo_trait_operation_drop_resource_graph_input_scanner.nepl` を追加し、actual Resource IR graph walker が将来返す typed walker event table を collector 入力へ正規化する checker-layer scanner boundary を接続した。input model は body record、place event、edge event、unsupported event を別 table として持つ。これにより、place と edge を sentinel id 付きの汎用 record に潰さず、operation ordinal、place id、edge endpoint、unsupported reason を typed payload として検査できる。

scanner は input table 全体を preflight する。body は `SelfhostTypeId`、body module fingerprint、Drop body root、effect、escape、graph id、upstream completeness を持ち、`InternalAlloc + NotApplicable` 以外、placeholder fingerprint、不正 graph id、duplicate body を拒否する。place / edge / unsupported event は同じ body graph 内の `operation_ordinal` が一意でなければならず、所属 body が missing または upstream unsupported の場合も fail-closed に拒否する。closed body に unsupported event がある場合、collector input では `TraversalUnsupported` body として渡し、partial place / edge graph を no-escape proof へ使わない。

この checkpoint も full Resource IR graph walker 本体ではない。Rust 側の `ResourceOp` 走査、PlaceRoot / projection の完全分類、RawMemory / RawAddress / CollectionSlot / Call / IndirectCall / FunctionValue::Memoized / PrivateCache / PrivateState の詳細 graph lowering は後続で追加する。この scanner は、typed walker event stream が成立した場合に collector が読む `SelfhostDropResourceGraphInput` を owner-safe に作るだけであり、traversal summary、no-escape observation、proof table、Drop evidence、aggregate proof、proof store、PrivateCache / PrivateState masking、backend artifact、prechecked artifact を作らない。

source policy は `nodesrc/test_selfhost_memo_trait_operation_drop_resource_graph_input_scanner_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、explicit import allow-list、forbidden layer import、typed body/place/edge/unsupported event fields、preflight validation、unsupported event による `TraversalUnsupported` override、collector input constructor への一方向出力、proof table / Drop evidence / aggregate proof / proof store / PrivateCache mask / PrivateState mask / prechecked artifact 合成禁止、Result error の bool / string 化禁止、行数 / doc comment 長制限禁止を確認する。

この checkpoint 後の残件は、Rust Resource IR 相当の actual graph walker 本体、complete public surface 由来の no-drop absence proof boundary、generic impl binder / bound detailed evidence、PrivateCache / PrivateState effect masking、prechecked artifact 接続である。walker event operation ordinal index 化、graph lookup index 化、traversal summary / observation / proof table の sorted index 化、duplicate scan の bucket 化、stage0 fixture 分割は、今回固定した typed event authority / owner cleanup / fail-closed contract を保って後から行える最適化として扱う。

2026-06-14 Drop absence producer checkpoint では、`memo_trait_operation_drop_absence_producer.nepl` を追加し、complete public impl surface 由来の「Drop impl が存在しない」状態だけを `SelfhostMemoTraitOperationDropEvidence::NoDropRequired` へ写す checker-layer producer boundary を接続した。

この producer は fake impl candidate を作らない。Drop impl が存在しない場合には public impl header、trait application、method body、Drop body が存在しないため、`SelfhostMemoTraitOperationImplCandidate` や fake `SelfhostMemoTraitPublicImplHeaderInput` へ詰めると authority が壊れる。今回の出力は `SelfhostMemoTraitOperationDropEvidence` だけであり、operation evidence record、aggregate proof status、proof store、backend / prechecked artifact には接続しない。

accepted authority は `SelfhostMemoTraitOperationDropImplSurfaceState`、borrowed `SelfhostMemoTraitOperationDropImplTable`、`SelfhostTypeId`、既存 Drop impl resolver と purity gate の typed result だけである。`Complete` surface で resolver が `DropImplAbsent` を返した場合だけ、purity gate の Drop operation evidence result を通し、`NoDropRequired` だけを success として受理する。`DropImplPresent`、`Missing`、`Unknown`、`NotRequired`、duplicate / table read failure / push failure など resolver rejection、purity gate rejection、gate の unexpected evidence は typed error として fail-closed にする。

`public_surface_hash`、source text、span、lexeme、display name、diagnostic text、module path、payload hash、HIR body root、Resource IR graph、proof store record は no-drop absence proof の authority ではない。complete surface state は caller が渡す typed completion witness であり、空 table や lookup miss だけから `NoDropRequired` を作る設計ではない。generic impl / unresolved bound / unsupported pattern を complete absence と混同してはいけないため、後続の public surface orchestrator はそれらを `Unknown` または typed unsupported として surface state へ反映する必要がある。

source policy は `nodesrc/test_selfhost_memo_trait_operation_drop_absence_producer_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、checker-layer import allow-list、Resource IR / backend / proof store / canonical key / public surface / scanner / materializer / method-body / Drop resource / PrivateCache / PrivateState import 禁止、operation evidence record / aggregate proof / proof store / private effect / backend / prechecked artifact 合成禁止、DropImplAbsent だけの success、present / missing / unknown / not-required fail-closed、resolver / purity gate typed error preservation、bool / string error 禁止、行数 / doc comment 長制限禁止を確認する。

subagent review では、`NoDropRequired` を fake impl candidate で表現しないこと、complete public surface witness と `DropImplAbsent` を必須 authority にすること、`Missing` / `Unknown` / `DropImplPresent` / partial surface / duplicate を fail-closed にすること、sorted index や lookup cache は後続最適化に回すことが Required / Non-blocker として確認された。今回の producer は evidence table 増補や operation proof table 接続までは行わず、absence-only evidence boundary として閉じる。

2026-06-14 Drop absence evidence connector checkpoint では、`memo_trait_operation_drop_absence_evidence_connector.nepl` を追加し、complete public impl surface 由来の no-drop absence evidence を既存 `SelfhostMemoTraitOperationEvidenceTable` owner へ増補する checker-layer connector boundary を接続した。

この connector は owned operation evidence table、`SelfhostMemoTraitOperationDropImplSurfaceState`、borrowed `SelfhostMemoTraitOperationDropImplTable`、`SelfhostTypeId` を受け取り、追加前に同じ TypeId / Drop operation の既存 record を lookup する。既存 Drop record がある場合は `DropEvidenceAlreadyPresent(status)`、duplicate がある場合は `EvidenceTableRejected(DuplicateRecord)` として fail-closed にし、record order による first-wins を許さない。preflight / absence producer error の経路では connector が table owner を閉じ、push failure は evidence table push boundary が owner を消費するため二重解放しない。

accepted authority は `memo_trait_operation_drop_absence_producer` の `Ok(NoDropRequired)` と operation evidence table の typed API だけである。fake `SelfhostMemoTraitOperationImplCandidate`、fake public impl header、public surface hash authority、source text / span / display / path authority、Resource IR graph、proof store、aggregate proof、PrivateCache / PrivateState、backend artifact、prechecked artifact は使わない。`NoDropRequired` だけを `SelfhostMemoTraitOperationEvidenceKind::Drop` / `SelfhostMemoTraitAggregateProofStatus::Proven` に写し、`PureDrop` は Drop body effect / Resource no-escape proof 側の責務としてこの module では合成しない。

source policy は `nodesrc/test_selfhost_memo_trait_operation_drop_absence_evidence_connector_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、minimal checker-layer import allow-list、fake candidate / fake header / impl table / public surface / Resource / backend / proof store / PrivateCache / PrivateState / prechecked import 禁止、existing Drop / duplicate preflight rejection、producer error owner cleanup、`NoDropRequired -> Drop/Proven` 変換、non-`NoDropRequired` fail-closed、typed error、wildcard-free equality、行数 / doc comment 長制限禁止を確認する。

subagent review では、absence を既存 public impl operation connector の `CandidateMissing` path に混ぜないこと、`DropImplPresent` を成功扱いにしないこと、owner cleanup と duplicate rejection を connector contract と source policy に固定すること、fake candidate / fake public impl header / public surface hash authority を禁止することが Required として確認された。実装はこの指摘に従い、absence producer と operation evidence table の間だけを接続する小さい module にした。

この checkpoint 後の残件は、Rust Resource IR 相当の actual graph walker 本体、generic impl binder / bound detailed evidence、PrivateCache / PrivateState effect masking、prechecked artifact 接続である。Drop impl fact table lookup の sorted index 化、surface completion witness の origin index 化、absence lookup cache、stage0 fixture 分割、operation evidence table lookup の index 化は、今回固定した typed authority / fail-closed / owner cleanup contract を保って後から行える最適化として扱う。

## 2026-06-14 MemoKey / MemoValue generic impl binder evidence checkpoint

`stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_binder.nepl` を追加し、public trait impl header に含まれる generic parameter binder と bound の詳細 evidence を検査する checker-layer boundary を作った。

この boundary は、`memo_trait_public_impl_header.nepl` が現時点で `type_parameter_count` / `type_parameter_bound_count` だけを受け取り、generic impl を fail-closed にしている理由を解消するための前段である。count だけでは異なる binder / bound environment を同一視するため、今回の module は parameter ordinal、`SelfhostTypeParameterBinding`、stable symbol hash、bound range、bound 側の trait application shape hash、trait type argument count を typed table として受ける。

accepted authority は caller supplied の `SelfhostMemoTraitPublicImplGenericParameterTable` と `SelfhostMemoTraitPublicImplGenericBoundTable` の typed field だけである。parameter table length と expected type parameter count、bound table length と expected bound count は一致しなければならない。parameter ordinal は 1-origin table order と一致し、`SelfhostTypeParameterBinding` は Phase 1 では `binder_depth = 0`、`parameter_index = table index` でなければならない。stable symbol hash 0、negative range、range overlap / hole、bound parameter mismatch、bound ordinal mismatch、missing / placeholder trait application shape hash、negative trait type argument count は typed error として fail-closed にする。

この checkpoint は generic impl の semantic acceptance ではない。trait solving 成功、operation impl candidate、operation evidence record、aggregate proof status、public surface hash authority、Resource IR proof、PrivateCache / PrivateState masking、backend artifact、prechecked artifact は作らない。出力は schema version、type parameter count、bound count、nonzero shape hash を持つ `SelfhostMemoTraitPublicImplGenericBinderEvidence` であり、後続の public impl header / materializer 接続がこの hash を header shape へ含めるかどうかを別 boundary で判断する。

source policy は `nodesrc/test_selfhost_memo_trait_public_impl_generic_binder_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、HIR / Resource IR / backend / proof store / operation classifier / candidate builder / evidence producer / public impl header / PrivateCache / PrivateState / prechecked import 禁止、count-only generic acceptance 禁止、source text / span / display / path / diagnostic / public surface hash authority 禁止、typed table length / ordinal / binding / range / bound shape 検査、impl candidate / operation evidence / aggregate proof / private effect 合成禁止、行数 / doc comment 長制限禁止を確認する。

subagent review では、count だけで `GenericImplUnsupported` / `TypeParameterBoundUnsupported` を解除する設計、operation candidate や operation evidence へ直接流す設計、public surface hash を generic binder authority にする設計は Blocker と確認された。実装はこの指摘に従い、generic binder / bound detail table の validation と shape evidence だけで閉じる。`SelfhostTypeParameterBinding` を authority に含めるべきという Required も反映し、source name ではなく binder depth / parameter index を検査する。

この checkpoint 後の残件は、generic binder evidence hash を public impl header shape / materializer record へ接続する boundary、trait bound solving、generic impl coherence、Rust Resource IR 相当の actual graph walker 本体、PrivateCache / PrivateState effect masking、prechecked artifact 接続である。generic binder table の sorted index 化、bound lookup cache、stage0 fixture 分割は、今回固定した typed authority / fail-closed / shape hash contract を保って後から行える最適化として扱う。

## 2026-06-14 MemoKey / MemoValue generic binder header evidence checkpoint

`memo_trait_public_impl_header.nepl` に `SelfhostMemoTraitPublicImplHeaderGenericBinderEvidence` を追加し、monomorphic header と detailed generic binder evidence を enum で分ける boundary を接続した。count だけで generic impl を accepted に戻すのではなく、`SelfhostMemoTraitPublicImplGenericBinderEvidence` の parameter count、bound count、nonzero shape hash を検査してから public impl header shape hash へ折り込む。

既存の count-only `selfhost_memo_trait_public_impl_header_shape_hash_result` は fail-closed API として残す。`type_parameter_count > 0` は `GenericImplUnsupported`、`type_parameter_bound_count > 0` は `TypeParameterBoundUnsupported` のまま拒否する。generic-aware path は `selfhost_memo_trait_public_impl_header_shape_hash_with_generic_binder_result` だけであり、monomorphic input は `Monomorphic`、generic input は `Detailed(evidence)` を明示的に渡す。

`memo_trait_operation_public_impl_materializer.nepl` の record も同じ generic binder evidence mode を保持するようにした。materializer は header validation を classifier / builder input 生成より先に通す。`Detailed` record は generic binder hash が妥当でも、generic instantiation、bound solving、substituted target type shape evidence が未接続であるため、operation impl candidate を作る前に `GenericImplInstantiationUnsupported` で拒否する。これにより、generic binder hash の有無と operation proof の成立を混同しない。

`memo_trait_operation_public_impl_drop_fact_orchestrator.nepl` と `memo_trait_operation_drop_candidate_connector.nepl` も materializer record の generic binder evidence mode を保持する。Drop fact orchestrator は public impl header validation を classifier より前に行い、detailed generic record を Drop fact table へ投入する前に拒否する。Drop candidate connector は record から header input を再構成するときに `record.generic_binder_evidence` を使い、monomorphic evidence へ潰さない。

この checkpoint の accepted authority は、public impl header input の typed field、detailed generic binder evidence、trusted trait operation classifier input、typed method body root identity だけである。source text、span、lexeme、display name、diagnostic text、module path、public surface hash、HIR 再探索、Resource IR、backend artifact、proof store record は generic binder header acceptance の authority ではない。

subagent review では、generic count / bound count だけで materializer が `SelfhostMemoTraitOperationImplCandidate` を作ること、raw `i32` hash だけを generic binder evidence として扱うこと、generic instantiation / bound solving なしに operation proof へ進めることが Blocker と確認された。実装は typed enum evidence を record に持たせ、count-only path を fail-closed のまま残し、detailed generic record を operation candidate 化前で止める形にした。

この checkpoint 後の残件は、generic impl instantiation、trait bound solving、substituted target type shape evidence、generic coherence、Rust Resource IR 相当の actual graph walker 本体、PrivateCache / PrivateState effect masking、prechecked artifact 接続である。materializer record table の operation bucket 化、generic binder table の sorted index 化、bound lookup cache、stage0 fixture 分割は、今回固定した typed evidence / fail-closed / owner cleanup contract を保って後から行える最適化として扱う。

## 2026-06-14 MemoKey / MemoValue generic impl instantiation gate checkpoint

`memo_trait_public_impl_generic_instantiation.nepl` を追加し、detailed generic binder evidence を持つ public trait impl を operation candidate へ進める前に必要な generic instantiation evidence contract を固定した。

この gate は actual substitution engine でも trait solver でもない。入力は `SelfhostMemoTraitPublicImplGenericInstantiationInput` の typed field だけであり、generic binder evidence、type argument count、schema 付き stable type argument identity hash、substitution shape evidence、bound solving status を分けて受け取る。source text、span、lexeme、display name、diagnostic text、module path、public surface hash、HIR、Resource IR、backend artifact、proof store record は accepted instantiation hash material に入らない。

`generic_binder_evidence.type_parameter_count` と `type_argument_count` は一致しなければならない。stable type argument identity は `memo_trait_type_argument_identity` が作った `schema_version` と `identity_hash` の両方を検査し、hash value だけを authority にしない。substituted target type shape と substituted trait application shape は raw `Option i32` では受け取らず、`memo_trait_public_impl_generic_substitution_shape` の evidence からだけ取り出す。

bound solving は `NoBounds`、`AllSolved(evidence)`、`Unsolved(first_unsolved_bound_ordinal)` の enum として扱う。bound count が 0 の場合は `NoBounds` だけを受理し、bound count が 1 以上の場合は `AllSolved` だけを受理する。`AllSolved` evidence は schema version、solved bound count、solver policy hash、proof shape hash を検査し、count mismatch や placeholder hash を typed error として fail-closed にする。`Unsolved` は success へ弱めず、未解決 ordinal を payload として保持する。

この checkpoint でも materializer の `GenericImplInstantiationUnsupported` は維持する。今回の evidence success は「generic impl candidate acceptance」ではなく、後続で materializer record に接続するための前段 contract である。operation classifier、method body purity、Drop no-escape、MemoKey / MemoValue aggregate proof、PrivateCache / PrivateState masking、prechecked artifact acceptance は別 stage が別 evidence として検査する。

source policy は `nodesrc/test_selfhost_memo_trait_public_impl_generic_instantiation_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、stable type argument identity / generic binder evidence import、HIR / Resource IR / backend / proof store / operation classifier / candidate builder / public impl header / private effect / prechecked artifact import 禁止、typed input / evidence / error enum、bound status consistency、payload-aware error equality、materializer fail-closed 維持、source-derived authority 禁止、行数 / doc comment 長制限禁止を確認する。

この checkpoint 後の残件は、actual type substitution engine、trait bound solver、generic coherence、generic instantiation evidence を materializer accepted path へ接続する boundary、PrivateCache / PrivateState effect masking、prechecked artifact 接続である。instantiation gate の内部 hash 計算は O(1) であり、solver table lookup、type substitution cache、bound lookup cache、stage0 fixture 分割は今回固定した typed authority / fail-closed contract を保って後から行える最適化として扱う。

## 2026-06-14 MemoKey / MemoValue generic substitution shape producer checkpoint

`memo_trait_public_impl_generic_substitution_shape.nepl` を追加し、generic impl instantiation gate が要求する substituted target type shape / substituted trait application shape を、任意の nonzero hash ではなく typed substitution shape evidence として扱うための前段境界を固定した。

この producer は actual type substitution engine ではない。入力は `SelfhostMemoTraitPublicImplGenericSubstitutionShapeInput` の typed field だけであり、generic binder evidence、type argument count、schema 付き stable type argument identity hash、pre-substitution target type shape hash、pre-substitution trait application shape hash、substitution trace shape hash、substituted target type shape hash、substituted trait application shape hash を分けて受け取る。source text、span、lexeme、display name、diagnostic text、module path、public surface hash、HIR、Resource IR、backend artifact、proof store record は accepted substitution shape material に入らない。

この producer は generic substitution 専用であるため、`generic_binder_evidence.type_parameter_count` が 0 以下の input は `GenericParameterCountMissing` として拒否する。monomorphic impl は既存 header / materializer の monomorphic path が担当し、generic substitution trace を持つものとして扱わない。`type_argument_count` は非負で、binder evidence の type parameter count と一致しなければならない。type argument identity は schema version と identity hash の両方を検査し、pre-substitution shape、trace shape、substituted shape はすべて `some(nonzero)` だけを受理する。

success evidence は schema、type parameter count、type argument count、type parameter bound count、generic binder shape hash、type argument identity hash、pre-substitution shape、substitution trace shape、substituted shape、combined substitution shape hash を保持する。後続の actual substitution engine は、この trace shape と output shape が実際の typed substitution result から来たことをさらに強く証明する。今回の checkpoint では materializer の `GenericImplInstantiationUnsupported` は維持し、operation candidate acceptance を広げない。

`substitution_trace_shape_hash` は stage0 では target type と trait application の substitution trace を束ねる単一 summary として扱う。actual substitution engine が typed trace table を持つ段階では、schema version を上げたうえで target substitution trace と trait application substitution trace を別 field に分けられる。この checkpoint は、trace が source text や display name から作られず、typed substitution step summary から来る必要がある、という authority 境界を先に固定する。

source policy は `nodesrc/test_selfhost_memo_trait_public_impl_generic_substitution_shape_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、stable type argument identity / generic binder evidence import、HIR / Resource IR / backend / proof store / operation classifier / candidate builder / public impl header / private effect / prechecked artifact import 禁止、typed input / evidence / error enum、monomorphic rejection、payload-aware error equality、materializer fail-closed 維持、source-derived authority 禁止、行数 / doc comment 長制限禁止を確認する。

この checkpoint 後の残件は、actual type substitution engine、trait bound solver、generic coherence、generic instantiation evidence を materializer accepted path へ接続する boundary、PrivateCache / PrivateState effect masking、prechecked artifact 接続である。substitution trace lookup cache、type argument substitution cache、bound lookup cache、stage0 fixture 分割は、今回固定した typed authority / fail-closed contract を保って後から行える最適化として扱う。

## 2026-06-14 MemoKey / MemoValue generic instantiation consumes substitution shape evidence checkpoint

`memo_trait_public_impl_generic_instantiation.nepl` を更新し、generic instantiation input から raw `substituted_target_type_shape_hash %Option i32` と raw `substituted_trait_application_shape_hash %Option i32` を除去した。現在の input は `SelfhostMemoTraitPublicImplGenericSubstitutionShapeEvidence` を受け取り、instantiation root hash へ混ぜる substitution material も `substitution_shape_hash` と evidence 内の substituted target / trait application shape から作る。

この follow-up は actual substitution engine ではない。ただし、substitution shape producer が作ったと主張する evidence を instantiation gate が無条件に信用しないことを固定する。instantiation gate は schema version、substitution shape root hash、type parameter count、type argument count、type parameter bound count、generic binder shape hash、schema 付き type argument identity、pre-substitution target / trait application shape、substitution trace shape、substituted target / trait application shape を再検査する。これにより、producer を迂回して任意の nonzero output shape hash を渡す API は閉じられる。

pre-substitution target / trait application shape が public impl header の original shape と一致するかどうかは、この gate ではなく materializer accepted path へ接続する後続 connector の責務である。instantiation gate は public value として構築可能な substitution evidence を再検査し、binder / type argument identity / substitution trace / substituted output shape / bound solving を同じ instantiation evidence record に束ねるところまでを担当する。

accepted instantiation evidence は `type_argument_count` と `substitution_shape_hash` を保持するようになった。後続の materializer accepted path は、単に target / trait shape hash だけを読むのではなく、binder evidence、type argument identity、substitution evidence、bound solving evidence を同じ instantiation evidence record として確認できる。

source policy は `nodesrc/test_selfhost_memo_trait_public_impl_generic_instantiation_contract.js` を更新して固定した。substitution shape evidence import、raw `Option i32` input 禁止、old raw target / trait helper path 禁止、substitution evidence 再検査、payload-aware count mismatch equality、materializer fail-closed 維持、source-derived authority 禁止、行数 / doc comment 長制限禁止を確認する。

この checkpoint でも materializer の `GenericImplInstantiationUnsupported` は維持する。actual type substitution engine、trait bound solver、generic coherence、generic instantiation evidence を materializer accepted path へ接続する boundary、PrivateCache / PrivateState effect masking、prechecked artifact 接続は未実装である。substitution trace lookup cache、type argument substitution cache、bound lookup cache、stage0 fixture 分割は、今回固定した evidence boundary を保てるため後からできる最適化として扱う。

## 2026-06-14 MemoKey / MemoValue generic substitution trace evidence checkpoint

`memo_trait_public_impl_generic_substitution_trace.nepl` を追加し、generic substitution shape producer が raw `substitution_trace_shape_hash` を直接受け取る境界を閉じた。現在の substitution shape input は `SelfhostMemoTraitPublicImplGenericSubstitutionTraceEvidence` を受け取り、shape producer 側でも schema、binder shape hash、type parameter count、type argument count、bound count、schema 付き type argument identity、trace record count、trace shape hash を再検査する。

この trace evidence producer は actual type substitution engine ではない。accepted authority は `SelfhostMemoTraitPublicImplGenericBinderEvidence`、`SelfhostMemoTraitPublicImplGenericParameterTable`、`SelfhostMemoTraitPublicImplGenericSubstitutionTraceTable`、`SelfhostMemoTraitStableTypeArgumentIdentity` の typed field だけである。producer は identity owner の entry vector から aggregate hash を再計算し、保存されている schema 付き identity hash と一致した場合だけ受理する。さらに trace record 側の copied entry と identity owner 側の同じ ordinal の entry を、ordinal、canonical fingerprint schema / root hash、canonical payload schema / payload hash まで照合する。parameter ordinal、`SelfhostTypeParameterBinding`、stable symbol hash、argument ordinal、stable type argument identity entry の schema / hash / ordinal を O(n) で検査し、source text、span、lexeme、display name、diagnostic text、module path、public surface hash、HIR、Resource IR、backend artifact、proof store record は trace authority にしない。

substitution shape producer は、trace evidence の binder hash / type argument identity / count が自身の binder evidence と type argument identity に一致する場合だけ、pre-substitution shape と substituted output shape を同じ evidence record に束ねる。これにより、任意の nonzero trace hash を入れて generic instantiation root hash に混ぜる経路を閉じる。ただし output shape が actual typed substitution traversal から来たことの証明、trait bound solving、generic coherence、materializer accepted path への接続は後続 stage の責務として残す。

source policy は `nodesrc/test_selfhost_memo_trait_public_impl_generic_substitution_trace_contract.js` と `nodesrc/test_selfhost_memo_trait_public_impl_generic_substitution_shape_contract.js` で固定した。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、HIR / Resource IR / backend / proof store / operation classifier / candidate builder / public impl header / PrivateCache / PrivateState / prechecked artifact import 禁止、typed table / evidence / error enum、raw trace option input 禁止、identity entry vector 由来の aggregate hash 再検査、trace entry と identity owner entry の一致検査、payload-aware error equality、materializer fail-closed 維持、source-derived authority 禁止、行数 / doc comment 長制限禁止を確認する。

この checkpoint でも materializer の `GenericImplInstantiationUnsupported` は維持する。trace stage0 smoke は機能的には `NEPL_TEST_CASE_TIMEOUT_MS=120000` の focused 検証で pass しているが、通常 60 秒 timeout では resource static check が支配的になるため doctest は skip にしている。trace record table の sorted index 化、stage0 fixture 分割、type argument substitution cache、bound lookup cache、resource static check の探索範囲削減は、今回固定した typed trace evidence contract を保って後から行える最適化として扱う。

## 2026-06-14 MemoKey / MemoValue generic binder same-origin table hash checkpoint

`memo_trait_public_impl_generic_binder.nepl` を更新し、accepted binder evidence に `parameter_table_shape_hash` と `bound_table_shape_hash` を保持するようにした。これにより、後続の trace / shape / instantiation gate は、root の `generic_binder_shape_hash` だけではなく、binder producer が検査した parameter table と bound table の形も同じ evidence record から再確認できる。

この checkpoint の目的は、別由来の `SelfhostMemoTraitPublicImplGenericBinderEvidence` を、たまたま count や root hash が合うだけで substitution trace / substitution shape / instantiation evidence へ混ぜないことである。binder module は producer と同じ parameter table hash / bound table hash / root shape hash を再計算する same-origin gate を提供し、trace evidence producer は `SelfhostMemoTraitPublicImplGenericParameterTable` と `SelfhostMemoTraitPublicImplGenericBoundTable` を受け取って binder evidence と照合する。mismatch は typed enum error として fail-closed に扱い、source text、display name、public surface hash、HIR、Resource IR、backend artifact、proof store record は authority にしない。

`memo_trait_public_impl_generic_substitution_trace.nepl` は trace evidence に `generic_parameter_table_shape_hash` と `generic_bound_table_shape_hash` を保存する。`memo_trait_public_impl_generic_substitution_shape.nepl` は trace evidence の table hash と binder evidence の table hash を再検査し、`memo_trait_public_impl_generic_instantiation.nepl` も substitution shape evidence の table hash を binder evidence と照合する。これにより、binder evidence、trace evidence、substitution shape evidence、instantiation evidence が同じ generic parameter / bound table 由来であることを段階ごとに確認できる。

`memo_trait_public_impl_header.nepl` は detailed binder evidence を header shape hash に混ぜる前に、parameter table shape hash と bound table shape hash の placeholder も明示的に拒否する。`memo_trait_public_impl_generic_instantiation.nepl` の accepted instantiation evidence も `generic_parameter_table_shape_hash` と `generic_bound_table_shape_hash` を保持し、後続 materializer accepted path が instantiation evidence だけを読んでも same-origin 情報を失わないようにした。

この checkpoint でも actual type substitution engine、trait bound solver、generic coherence、materializer accepted path は開かない。materializer は引き続き detailed generic record を `GenericImplInstantiationUnsupported` で止める。table hash の sorted index 化、bound lookup cache、stage0 fixture 分割、trace lookup cache は今回固定した same-origin contract を保って後から行える最適化として扱う。

## 2026-06-14 Selfhost core type substitution traversal checkpoint

`stdlib/neplg2/core/ty/ty/substitution.nepl` を追加し、selfhost compiler が generic substitution を hash 合成ではなく `SelfhostTypeArena` 上の typed traversal として扱うための core 境界を作った。

この module は checker-layer generic materializer ではない。入力は `SelfhostTypeArena`、borrowed `SelfhostTypeSubstitutionBindingTable`、root `SelfhostTypeId` であり、output は更新済み arena、typed substitution step table、`SelfhostTypeSubstitutionEvidence` を束ねた owner result である。`Primitive` / `Named` は保持し、`Parameter` は binding table に一致する場合だけ replacement TypeId へ写す。`Applied` と `Function` は child TypeId を左から再帰的に置換し、置換後の child list から新しい type record を arena に追加する。

accepted authority は arena 内の `SelfhostTypeRecord`、binding table 内の `SelfhostTypeParameterBinding` と replacement TypeId、schema 付き substitution step stream だけである。source text、span、display name、diagnostic text、module path、public surface hash、HIR、Resource IR、backend artifact、proof store、PrivateCache / PrivateState、prechecked artifact は import せず、substitution evidence の authority にしない。

失敗は `SelfhostTypeSubstitutionErrorKind` の enum data として保持する。invalid binding、duplicate binding、binding record read failure、missing replacement type、missing source type record、missing applied / function child、rebuild failure、traversal fuel exhaustion、step stream hash placeholder、evidence hash mismatch はそれぞれ別 variant で fail-closed にする。error display や bool flag へ潰さない。error equality は wildcard fallback を使わず、variant code と payload slot を網羅 match で正規化して比較する。

所有権境界は Resource IR の static check を authority として設計した。`selfhost_type_substitution_push_step`、`selfhost_type_substitution_push_arg_result`、`selfhost_type_arena_add_applied_named`、`selfhost_type_arena_add_function` は失敗時に一部 owner を消費するため、caller 側では消費済み owner を再利用せず、`*_after_step_table_consumed`、`*_after_args_consumed`、`*_after_arena_builder_consumed` helper で残っている owner だけを閉じる。

現在の計算量は、visited type node / edge 数を `n`、binding count を `b`、rebuild child count 合計を `a` とすると、binding duplicate validation が `O(b*b)`、parameter lookup が `O(n*b)`、applied / function rebuild が `O(a)` である。合計の上界は `O(b*b + n*b + a)` である。これは正しい authority boundary を先に固定するための単純な `Vec` table 実装であり、binding table の sorted index 化、duplicate validation index 化、replacement lookup cache、substitution result memo、stage0 fixture 分割は、この typed traversal / owner cleanup / evidence schema contract を保てるため後続最適化として扱う。

subagent review では、`idx < len` の binding table read が `None` になった場合を success にしていると構造破損を fail-closed にできないこと、`ty.nepl` facade export と source list の登録が一致していないこと、source policy regression runner に個別 contract が載っていないこと、error equality の wildcard fallback が variant 追加漏れを隠すことが指摘された。同じ checkpoint 内で `BindingRecordReadFailed`、`selfhost_ty_sources.js` 登録、source policy regression 登録、wildcard-free equality policy を追加した。

この checkpoint 後の残件は、checker-layer generic substitution shape producer へこの core substitution evidence を接続し、pre-substitution target / trait application shape から substituted target / trait application shape を実際の typed traversal output 由来にすることである。trait bound solver、generic coherence、generic instantiation evidence の materializer accepted path 接続、PrivateCache / PrivateState effect masking、prechecked artifact 接続はまだ開かない。

## 2026-06-14 MemoKey / MemoValue generic substitution canonical projection checkpoint

`stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_substitution_projection.nepl` を追加し、generic substitution shape evidence に保存された target / trait application の output `SelfhostTypeId` を canonical type key 由来の stable projection evidence へ写す checker-layer 境界を作った。

この producer は `SelfhostMemoTraitPublicImplGenericSubstitutionShapeEvidence` の schema、root hash placeholder、field 由来 root hash を再検査してから、`target_substitution_output_type_id` と `trait_application_substitution_output_type_id` を別々に canonical projection へ通す。projection は既存の `selfhost_canonical_type_key_project_from_arena`、`selfhost_memo_trait_canonical_type_fingerprint_result`、`selfhost_memo_trait_canonical_key_payload_hash_result` だけに委譲し、この module は canonical key の独自実装を持たない。成功時の evidence は source-local TypeId、schema 付き fingerprint、schema 付き payload hash、final shape hash を target / trait application それぞれに保持する。

accepted authority は `SelfhostTypeArena`、`SelfhostMemoTraitStableNominalKeyTable`、substitution shape evidence の typed field に限定する。source text、span、display name、diagnostic text、module path、public surface hash、HIR、Resource IR、backend artifact、proof store、PrivateCache / PrivateState、prechecked artifact は authority にしない。positive index でも caller が渡した arena に record が無い TypeId は `ProjectionRejected(MissingTypeRecord)` として拒否し、unresolved type parameter は canonical fingerprint の `TypeParameterUnsupported` として拒否する。ただし、この public API 単独では public constructible な substitution evidence の TypeId が substitution traversal と同じ arena owner から来たことまでは証明しない。同一 arena provenance は上流の owner boundary の precondition であり、materializer accepted path へ接続する後続 connector が再確認する。target 側と trait application 側は別 wrapper error に包み、payload を比較する equality で固定した。

`projection_shape_hash` は session-local TypeId linkage も含む checker-layer root hash であり、永続 artifact の単独 authority ではない。永続 authority として扱う材料は、target / trait application それぞれの canonical fingerprint、canonical payload hash、final shape hash を schema 付き field として保持する部分である。target と trait application の hash material には別 domain tag を混ぜ、同じ canonical type の入れ替えや順序依存だけに頼る退行を防ぐ。

この checkpoint でも materializer の `GenericImplInstantiationUnsupported` は維持する。projection evidence は generic impl candidate acceptance ではなく、後続の trait bound solver、generic coherence、instantiation evidence connector、materializer accepted path が読む typed material である。stable nominal key table lookup の index 化、stage0 fixture 分割、projection result memo は、今回固定した schema / payload / fail-closed contract を保てるため後続最適化として扱う。

## 2026-06-15 MemoKey / MemoValue generic instantiation projection connector checkpoint

`stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_instantiation_projection_connector.nepl` を追加し、generic substitution shape evidence、generic instantiation evidence、canonical projection evidence を同じ checker-layer boundary で照合する connector を作った。

この connector は materializer accepted path ではない。入力は expected pre-substitution target / trait application shape、`SelfhostMemoTraitPublicImplGenericSubstitutionShapeEvidence`、`SelfhostMemoTraitPublicImplGenericInstantiationEvidence`、`SelfhostMemoTraitPublicImplGenericSubstitutionProjectionEvidence` に限定する。HIR、Resource IR、backend artifact、proof store、public impl materializer、PrivateCache / PrivateState、prechecked artifact は import せず、generic detailed record を operation candidate に変換しない。

connector は substitution evidence の schema / root hash、instantiation evidence の schema / root hash、projection evidence の schema / root hash をそれぞれ field から再計算する。instantiation schema は `selfhost_memo_trait_public_impl_generic_instantiation_schema_version` と exact match しなければならず、canonical fingerprint / payload schema も `selfhost_memo_trait_canonical_type_fingerprint_schema_version` と `selfhost_memo_trait_canonical_key_payload_schema_version` に一致しなければならない。さらに instantiation evidence が参照する substitution hash、target / trait application の substitution output `SelfhostTypeId`、substituted shape hash、projection evidence の source TypeId、final shape hash、canonical fingerprint / payload hash を相互に照合する。pre-substitution shape は caller が渡した expected target / trait application shape と substitution evidence の field が一致する場合だけ受理する。これにより、projection producer と instantiation gate を個別に通しただけの public constructible record を materializer の根拠として混ぜる経路を閉じる。

`SelfhostMemoTraitPublicImplGenericInstantiationEvidence` には `substitution_trace_shape_hash` を保存する field を追加した。instantiation evidence の root hash は従来から substitution trace material を混ぜていたが、accepted evidence record には trace hash が残っていなかったため、後続 connector が evidence field だけから root hash を再計算できなかった。この field は accepted instantiation evidence を永続 authority にするためではなく、same-session checker-layer evidence の再検査材料を落とさないための boundary である。

success evidence は substitution evidence、instantiation evidence、projection evidence、expected pre-shape、target / trait application の canonical material を束ねる。これは後続の trait bound solver、generic coherence、materializer accepted path が読む前段 material であり、まだ generic impl candidate acceptance ではない。trait bound solver、generic coherence、materializer accepted path、PrivateCache / PrivateState effect masking、prechecked artifact 接続は次の slice に残す。

source policy は `nodesrc/test_selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_contract.js` で固定する。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、forbidden layer import 禁止、materializer fail-closed 維持、substitution / instantiation / projection の root hash 再計算、instantiation / canonical material の exact schema match、cross-evidence field mismatch、canonical material placeholder rejection、全 payload variant の payload-aware error equality、private helper doc comment、行数 / doc comment 長制限禁止を確認する。

stage0 doctest は、意味論上の代表 smoke を持つ。ただし現行 selfhost owner-bearing fixture は Resource static check の探索時間が支配的で、60 秒 default timeout では compile timeout になり得る。これは connector の意味論を緩める理由にはせず、source policy と timeout-nonfatal focused run で semantic boundary を確認し、Resource static check の探索範囲削減は `ISS-20260614T130656620Z-SELFHOST-SUBSTITUTION-SHAPE-DOCTEST--405AF02E` 側の性能残件として扱う。

## 2026-06-15 MemoKey / MemoValue generic concrete coherence checkpoint

`stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_concrete_coherence.nepl` を追加し、generic instantiation projection connector evidence を materializer accepted path へ渡す前に、同じ concrete target / trait application へ複数の generic impl が到達していないことを検査する checker-layer 境界を作った。

この coherence boundary は materializer accepted path ではない。入力は `SelfhostMemoTraitPublicImplGenericInstantiationProjectionConnectorEvidence` と、caller が owner として持つ `SelfhostMemoTraitPublicImplGenericConcreteCoherenceRecordTable` に限定する。HIR、Resource IR、backend artifact、proof store、public impl materializer、PrivateCache / PrivateState、prechecked artifact は import せず、generic detailed record を operation candidate に変換しない。

coherence key は、target の canonical fingerprint / canonical payload hash / final shape と、trait application の canonical fingerprint / canonical payload hash / final shape の組である。`connector_shape_hash` だけ、final shape だけ、あるいは表示名や source text を authority にしない。connector evidence は public struct として構築可能なので、schema、connector hash、instantiation / substitution / projection / final shape、canonical fingerprint / payload schema と hash をこの module でも再検査する。`selfhost_memo_trait_public_impl_generic_instantiation_projection_connector_schema_version` は downstream exact schema check のため public にした。

現段階の coherence checker は full overlap solver ではない。同じ declaration / connector / canonical material が再投入された場合は `DuplicateExactMatch` として拒否し、異なる declaration または異なる connector が同じ concrete target / trait application key に到達した場合は `OverlapUnsupported` として fail-closed にする。generic parameter を含む blanket impl 同士の包含関係や unification は後続 solver の責務として残す。

record table は owner `Vec` を持つため shallow `Clone` / `Copy` を提供しない。record と evidence は fixed-size の schema 付き summary なので Copy 可能である。現状の table scan は O(n) 時間・O(1) 追加空間であり、sorted index、bucket 化、merge cursor は今回固定した canonical key / typed error / materializer fail-closed contract を保って後から置換できる最適化として扱う。

source policy は `nodesrc/test_selfhost_memo_trait_public_impl_generic_concrete_coherence_contract.js` で固定する。facade 非公開、`nodesrc/selfhost_ty_sources.js` 非登録、forbidden layer import 禁止、materializer fail-closed 維持、connector schema exact match、canonical material schema exact match、duplicate / overlap の payload-aware error equality、owner table の shallow Clone / Copy 禁止、行数 / doc comment 長制限禁止を確認する。

stage0 doctest は empty table accepted、connector schema placeholder、target final shape placeholder、exact duplicate、concrete overlap を確認する。focused doctest は pass しており、既存の connector / source policy も維持されている。この checkpoint 後も、generic instantiation / bound solver / projection connector / coherence evidence を materializer accepted path へ束ねる boundary、PrivateCache / PrivateState effect masking、prechecked artifact 接続は未実装である。

## 2026-06-15 MemoKey / MemoValue generic materializer connector checkpoint

`stdlib/neplg2/core/check/module/memo_trait_public_impl_generic_materializer_connector.nepl` を追加し、generic public impl record と generic evidence 群を materializer accepted path へ渡す前の checker-layer preflight boundary を固定した。

この connector は operation candidate を作らない。入力は `SelfhostMemoTraitOperationPublicImplMaterializerRecord`、`SelfhostMemoTraitPublicImplGenericBoundSolvingStatus`、`SelfhostMemoTraitPublicImplGenericInstantiationEvidence`、`SelfhostMemoTraitPublicImplGenericInstantiationProjectionConnectorEvidence`、`SelfhostMemoTraitPublicImplGenericConcreteCoherenceEvidence` であり、accepted authority はこれらの typed field に限定する。HIR traversal result、Resource IR、backend artifact、proof store、PrivateCache / PrivateState、prechecked artifact、source text、display name、diagnostic text は accepted material に入らない。

この boundary の中心は、public constructible な evidence を後続 materializer がそのまま信用しないことである。record 側の `Detailed` generic binder evidence と instantiation evidence の parameter count / bound count / table hash / binder shape を照合し、materializer record の declaration ordinal と original target / trait shape を projection connector evidence と照合する。bound solving status は `NoBounds` / `AllSolved` / `Unsolved` を connector でも再検査する。`AllSolved` evidence は `selfhost_memo_trait_public_impl_generic_bound_solving_evidence_schema_version` と一致する schema だけを受理し、そこから導出した bound solution hash が instantiation evidence の `bound_solution_shape_hash` と一致しなければ拒否する。

projection connector evidence と concrete coherence evidence は schema、instantiation shape、connector shape、canonical target fingerprint / payload、canonical trait application fingerprint / payload、target final shape、trait application final shape を相互に照合する。さらに projection connector evidence と instantiation evidence の substitution shape hash、substituted target shape、substituted trait application shape も照合する。成功時の `SelfhostMemoTraitPublicImplGenericMaterializerConnectorEvidence` は、record identity、generic binder material、bound solution hash、pre-substitution shape、instantiation / connector / coherence root、substituted target / trait shape、canonical target / trait material、final target / trait shape、materializer connector root hash を保持する。これは後続 candidate builder が読むための typed preflight summary であり、generic impl の accepted candidate そのものではない。

この checkpoint でも `memo_trait_operation_public_impl_materializer.nepl` は detailed generic record を `GenericImplInstantiationUnsupported` で拒否する。generic accepted path を実際に開くには、今回の connector evidence を operation classifier、method body purity、Drop no-escape、MemoKey / MemoValue aggregate proof と同じ candidate construction boundary へ接続する必要がある。したがって、現時点では facade へ re-export せず、`nodesrc/selfhost_ty_sources.js` にも登録しない。

source policy は `nodesrc/test_selfhost_memo_trait_public_impl_generic_materializer_connector_contract.js` で固定する。facade 非公開、selfhost source list 非登録、materializer fail-closed 維持、forbidden layer import 禁止、typed evidence field、bound status 再検査、bound solving evidence schema exact match、stage0 unsolved-bound / schema-mismatch rejection、行数 / doc comment 長制限禁止を確認する。focused doctest は 240 秒 timeout で pass しているが、標準 60 秒 timeout は現行 Resource static check の探索空間で超え得る。schema exact match 追加後の最新実測 wall time は約 156 秒である。これは後からできる fixture 分割 / Resource 探索削減の性能課題であり、typed authority boundary を緩める理由にはしない。

## 2026-06-15 MemoKey / MemoValue generic materializer accepted path checkpoint

`stdlib/neplg2/core/check/module/memo_trait_operation_public_impl_materializer.nepl` の generic-aware accepted path を、generic materializer connector input へ接続した。connector table を渡さない既存 API は従来どおり `Detailed` generic record を `GenericImplInstantiationUnsupported` で拒否する。一方で connector input table を渡す API では、matching declaration ordinal の input を探し、materializer record と input の bound solving status / instantiation evidence / projection connector evidence / coherence evidence を `selfhost_memo_trait_public_impl_generic_materializer_connector_result` に渡して bridge evidence をその場で作る。その後、defensive `evidence_recheck_result`、record identity、module fingerprint、pre-substitution target shape、pre-substitution trait application shape の再照合を通った場合だけ、concrete target TypeId と final target / trait shape を candidate builder input へ渡す。

この accepted path は raw final evidence vector や public constructible final evidence table を materializer が受け取らない。`memo_trait_public_impl_generic_materializer_connector.nepl` の table は `SelfhostMemoTraitPublicImplGenericMaterializerConnectorInputTable` であり、final evidence ではなく connector result を実行するための typed input だけを保持する。したがって public struct constructor で input table を直接作られても、materializer accepted path は table 中の final evidence を信用する余地を持たず、source record を持つ地点で connector result を再計算する。materializer 側の recheck は producer provenance の代替ではなく、connector result 生成後の field consistency check である。

この checkpoint でも operation candidate acceptance の最終責務は分離している。generic accepted path は classifier と public impl header producer と candidate builder に接続するが、method body purity、Drop no-escape proof、MemoKey / MemoValue aggregate proof、PrivateCache / PrivateState masking、proof store、backend artifact、prechecked artifact は直接 import しない。これらは既存の method body fact / Drop proof / operation evidence connector / aggregate solver / private effect boundary の後続 stage で検査される。

source policy は `nodesrc/test_selfhost_memo_trait_operation_public_impl_materializer_contract.js` と `nodesrc/test_selfhost_memo_trait_public_impl_generic_materializer_connector_contract.js` で固定する。materializer が shared record module を使い import cycle を作らないこと、materializer が raw connector evidence table を所有・push・受理しないこと、connector module が final evidence table を公開しないこと、materializer が matching input から `connector_result` を実行すること、concrete target TypeId が producer root と recheck root の両方に含まれること、Resource / backend / proof store / PrivateCache / prechecked layer を直接 import しないこと、行数 / doc comment 長制限を追加しないことを確認する。focused doctest は materializer 約 191 秒、generic materializer connector 約 161 秒の wall time で pass した。これは owner-bearing stage0 fixture と現行 Resource static check の探索空間による性能課題であり、semantic accepted path の誤りとは分けて扱う。table lookup の sorted index 化、operation bucket 化、stage0 fixture 分割は、この typed input / connector-result boundary を保ったまま後から行える最適化である。

## 2026-06-15 MemoKey / MemoValue generic surface orchestration checkpoint

`memo_trait_public_impl_surface_orchestrator.nepl` に、generic connector input table を受け取る surface orchestration 入口を追加した。既存の `selfhost_memo_trait_public_impl_surface_from_scanner_output_result` と `selfhost_memo_trait_public_impl_surface_from_ast_records_result` は monomorphic / generic fail-closed 互換 path として残す。新しい `*_with_generic_connectors_result` は、scanner output の public declaration evidence から public surface normalizer と full public surface hash composer を既存順序で通し、operation record table だけを `selfhost_memo_trait_operation_public_impl_materializer_candidate_table_from_records_with_generics_result` へ渡す。

この checkpoint の目的は、generic materializer accepted path を operation evidence / aggregate proof pipeline が読む `SelfhostMemoTraitPublicImplSurfaceState` へ接続することである。`SelfhostMemoTraitPublicImplSurfaceState` は従来どおり public surface hash と operation impl table owner を束ねる transport owner state であり、complete proof ではない。generic connector input は public surface hash の材料ではなく、operation materializer 内で detailed generic record を concrete builder input へ変換するためだけに使う。hash 生成前に connector input を読むことも、connector input から public surface hash を作ることも許さない。

この入口も raw final evidence table は受け取らない。materializer は connector input table から matching declaration ordinal を探し、materializer record と input の typed evidence 群を使って `connector_result` をその場で実行する。その結果だけが concrete target TypeId と final target / trait shape を持つ builder input へ進む。したがって、surface orchestrator は final evidence を信用するのではなく、generic-aware materializer boundary を呼び出す上位接続だけを担当する。

source policy は `nodesrc/test_selfhost_memo_trait_public_impl_surface_orchestrator_contract.js` を更新して固定した。generic connector import は `memo_trait_public_impl_generic_materializer_connector` の input table type に限定し、Resource / backend / proof store / operation evidence producer / purity gate / method body / Drop resolver / candidate builder の直接 import 禁止を維持する。さらに generic scanner-output entry が public surface normalization / hash order を変えないこと、connector input を materializer にだけ渡すこと、partial input items と scanner output owner cleanup を保つこと、行数 / doc comment 長制限を追加しないことを確認する。

この checkpoint 後の残件は、PrivateCache / PrivateState effect masking、prechecked artifact 接続、memo_call backend representation である。generic connector lookup の sorted index 化、operation bucket 化、stage0 fixture 分割、Resource initialized-state 探索範囲削減は、今回固定した transport owner state / generic-aware materializer boundary を保ったまま後から実装できる最適化として扱う。

## 2026-06-15 memo_call backend request manifest checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_request.nepl` を追加し、HIR の `SelfhostHirExprPayload::MemoizedFunctionValue` を codegen / backend が読む typed request manifest へ変換する境界を作った。

この boundary は private cache backend 実装ではない。Wasm / LLVM bytes、cache allocation、hit / miss logic、cache namespace 永続化、PrivateCache / PrivateState Resource proof、prechecked artifact は作らない。今回固定したのは、通常の `FnValue` と `memo_call @func` 由来の `MemoizedFunctionValue` を backend input で同化しないための typed manifest である。

accepted input は `SelfhostHirExprPayload::MemoizedFunctionValue(identity)` だけである。`FnValue` は `FnValueUnsupported`、`Call` は `CallUnsupported`、その他の non-memo payload は `NonMemoizedExpressionUnsupported` または `UnsupportedExprKind` として fail-closed にする。accepted identity は `DefId` を持ち、monomorphic で、`SelfhostEffectKind::Pure` であり、さらに `SelfhostHirExpr.ty` と identity の `function_ty` が一致していなければならない。これにより synthetic HIR が function identity と expression type を不整合にした場合も backend request へ流れない。

request record は `SelfhostMemoCallBackendRequestKind::MemoCall`、`source_function_def_id`、`function_ty`、`source_effect`、`type_arg_count` を authority field として持つ。`diagnostic_symbol` と `diagnostic_span` は診断 / dump / source map 用 metadata であり、accepted 判定、proof authority、cache namespace、永続 artifact key には使わない。`SelfhostDefId`、`SelfhostTypeId`、`SelfhostSourceSpan` は現時点では session-local identity であるため、将来 `.neplobj` / `.neplproof` / `.neplmeta` へ渡すときは canonical type key、public surface hash、compiler policy hash、stable source identity と組み合わせた別 boundary が必要である。

source policy は `nodesrc/test_selfhost_memo_call_backend_request_contract.js` で固定する。Resource IR、proof store、memo trait proof layer、PrivateCache / PrivateState、prechecked artifact、backend bytes、lower/hir、checker、compiler-known primitive registry を import しないこと、`"memo_call"` 文字列や candidate display name で acceptance しないこと、`expr.ty == identity.function_ty` を検査すること、error を enum variant に分けること、diagnostic metadata を identity matcher が読まないこと、行数 / doc comment 長制限を追加しないことを確認する。

この checkpoint 後の残件は、memoized function value の sealed backend representation、private cache region / no-escape proof、identity observation ban、`MemoKey` / `MemoValue` aggregate proof と backend request の接続、prechecked artifact / `.neplobj` への stable key 投影である。backend request table の index 化や request stream compaction は、今回固定した typed manifest contract を保てるため後続最適化として扱う。

## 2026-06-15 memo_call backend request table checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_request_table.nepl` を追加し、borrowed `SelfhostHirModule` の root expression subtree から `MemoizedFunctionValue` leaf だけを typed backend request stream へ集める境界を作った。

table entry は `memoized_expr_id` と `SelfhostMemoCallBackendRequest` を持つ。同じ `DefId` / `SelfhostTypeId` を指す memoized leaf が複数ある場合でも、後続の PrivateCache materialization は leaf occurrence ごとに別 cache を持つ必要があるため、collector は dedupe しない。`memoized_expr_id` は session-local occurrence metadata であり、永続 artifact key、diagnostic span、cache namespace の authority ではない。

collector は `SelfhostHirExprPayload::MemoizedFunctionValue` branch に入った場合だけ `selfhost_memo_call_backend_request_from_hir_expr_result` を呼ぶ。通常の `FnValue`、literal、`Var` は backend cache materialization を要求しないため正常に無視する。一方で、HIR `Error` payload、root expression 欠落、child range 不正、child id 欠落、child expression 欠落、memoized payload rejection は typed enum error として fail-closed にする。

child range は `selfhost_hir_child_range_new_bounded_result` で module child table 長に対して事前検証する。`selfhost_hir_module_get_child` の `Option::None` だけに頼ると、unchecked range 由来の範囲不正と child id 欠落が同じ失敗に潰れるためである。traversal fuel は再帰深さではなく訪問 expression 総数の予算として `SelfhostMemoCallBackendRequestTraversalState` に保持し、sibling 間で残量を thread する。これにより、壊れた HIR graph が横に広い場合でも探索量を上限付きにできる。

`SelfhostMemoCallBackendRequestTable` は owner buffer であり `Clone` / `Copy` を実装しない。push は collector 内部 API に閉じ、外部 caller が forged request entry を直接 table へ投入できないようにする。成功時は caller が table owner を閉じ、失敗時は collector が部分 table owner を閉じる。push 失敗では `Vec` が返した owner を閉じ、`StdErrorKind` を `RequestPushFailed` に保存する。

source policy は `nodesrc/test_selfhost_memo_call_backend_request_table_contract.js` で固定する。Resource IR、proof store、memo trait proof layer、PrivateCache / PrivateState、prechecked artifact、backend bytes、lower/check/compiler-known registry を import しないこと、`"memo_call"` 文字列、candidate display name、diagnostic symbol / span を authority にしないこと、child range bounded validation、global traversal fuel、HIR Error fail-closed、occurrence id 付き table entry、private push、owner cleanup、行数 / doc comment 長制限を追加しないことを確認する。

この checkpoint 後も、sealed memoized backend representation、PrivateCache / PrivateState effect masking、Resource no-escape proof、identity observation ban、prechecked artifact / `.neplobj` stable key 投影は未実装である。request table の sorted index 化、stream compaction、explicit stack traversal、subtree memoization、stage0 fixture 分割は、今回固定した occurrence identity / fail-closed / owner cleanup contract を保てるため後続最適化として扱う。

## 2026-06-15 memo_call backend preflight checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_preflight.nepl` を追加し、memoized backend request table を caller supplied authority として受け取らず、borrowed `SelfhostHirModule` と root expression から内部で request table を作って再照合する境界を作った。

この checkpoint は sealed backend representation ではなく、sealed backend representation へ進む前の fail-closed preflight である。request table は owner buffer だが `pub struct` なので、後続 stage が request table だけを受け取る API にすると forged table や stale table を accepted path へ流せる。preflight は public entrypoint を `SelfhostHirModule` borrow、root `SelfhostHirExprId`、traversal fuel に限定し、`selfhost_memo_call_backend_request_table_from_hir_root_result` で table を内部構築する。

構築した各 entry は `memoized_expr_id` から HIR expression を引き直し、`selfhost_memo_call_backend_request_from_hir_expr_result` を再実行した結果と `request_kind`、`source_function_def_id`、`function_ty`、`source_effect`、`type_arg_count` を照合する。`diagnostic_symbol`、`diagnostic_span`、関数表示名、`"memo_call"` 文字列は authority にしない。recheck で HIR expression が欠落した場合、builder が拒否した場合、typed field が一致しない場合は、それぞれ `RequestExpressionMissing`、`RequestRecheckRejected`、`RequestIdentityMismatch` として分ける。

request が 0 件なら backend materialization は不要なので `SelfhostMemoCallBackendPreflightSummary` を返す。request が 1 件以上ある場合、現段階では `PrivateCacheProofUnavailable(expr_id)` を返して backend accepted path を開かない。subagent review では `ProofDeferred` や public constructible `Ready` evidence を `Ok` の plan table に入れると後続 backend が proof 未接続のまま実行可能 plan と誤認する、という blocker が示された。今回の実装では実行可能 plan table を作らず、non-empty request を typed error として止めることでこの指摘に対応した。

source policy は `nodesrc/test_selfhost_memo_call_backend_preflight_contract.js` で固定する。public API が caller supplied request table を取らないこと、Resource IR / proof store / memo trait proof layer / PrivateCache / PrivateState / prechecked artifact / Wasm / LLVM bytes / artifact IO を import しないこと、HIR payload 再照合、typed field comparison、non-empty request の `PrivateCacheProofUnavailable`、owner table cleanup、`ProofDeferred` / `StableKeyReady` 相当の成功 path 禁止、行数 / doc comment 長制限を追加しないことを確認する。

この checkpoint 後の残件は、PrivateCache / PrivateState effect masking、Resource no-escape proof、identity observation ban、MemoKey / MemoValue aggregate proof と backend request の接続、prechecked artifact / `.neplobj` stable request key 投影である。preflight table recheck loop の sorted index 化や stage0 fixture 分割は、今回固定した HIR-root authority と fail-closed proof boundary を保てるため後続最適化として扱う。

## 2026-06-15 memo_call backend private-cache request-evidence gate checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` を追加し、memoized backend request を request occurrence evidence と照合する gate を作った。

この checkpoint は sealed backend representation でも、PrivateCache / PrivateState effect masking でも、Resource no-escape proof producer でもない。ここで固定したのは、backend accepted path へ進む前に「どの HIR root の、どの memoized occurrence に対する request-evidence なのか」を typed key で確認し、missing / refuted / duplicate / orphan proof を fail-closed にする境界である。

accepted gate 本体は module-private である。内部 entrypoint は borrowed `SelfhostHirModule`、root `SelfhostHirExprId`、traversal fuel、`body_module_fingerprint`、borrowed proof table だけを受け取る。caller supplied request table は受け取らず、内部で `selfhost_memo_call_backend_request_table_from_hir_root_result` を実行して request table を作る。その後、各 entry の `memoized_expr_id` から HIR expression を引き直し、`selfhost_memo_call_backend_request_from_hir_expr_result` を再実行して、request manifest の typed field と一致するかを確認する。

proof key は `memoized_expr_id`、`source_function_def_id`、`function_ty`、`root_expr_id`、`body_module_fingerprint`、`request_kind`、`source_effect`、`type_arg_count`、`proof_kind`、`proof_schema_version` を持つ。`DefId` / `TypeId` だけでは同じ関数を別 occurrence で memoize した場合や、別 root / 別 module の HIR id を取り違える危険があるため、request occurrence と root origin を key に含める。`proof_kind` は現段階では `RequestOccurrenceGateEvidence` だけであり、PrivateCache region no-escape や identity-observation proof と混同しないために入れている。

proof status は `RequestEvidenceProven` と `RequestEvidenceRefuted` である。`RequestEvidenceProven` はこの gate に必要な request occurrence evidence が一致したことだけを表し、PrivateCache effect fold、cache algorithm correctness、sealed representation、Resource no-escape proof の完了を意味しない。成功時も backend bytes や cache operation plan は作らず、`SelfhostMemoCallBackendPrivateCacheProofGateSummary` という non-executable summary だけを返す。

proof record、proof table、proof table push、accepted gate 本体は module-private にした。NEPL の public struct は constructor / field payload の直組み可能性を持つため、push だけを private にしても producer-owned 契約にはならない。後続で Resource proof producer を接続する場合は、その producer-owned boundary が同じ module-private table writer を所有し、同じ key / status contract を満たす必要がある。

subagent review では、`Proven` という広すぎる status 名が PrivateCache proof 完了と誤読されること、public push があると trust bypass になること、public proof table / record constructor / table field が残ると直組みされた evidence を accepted gate へ渡せること、proof key に request kind / source effect / type argument count / proof kind / schema version が必要なこと、proof table に current root の request と対応しない orphan record がある場合は拒否すべきことが blocker / required として指摘された。実装では status 名を `RequestEvidenceProven` / `RequestEvidenceRefuted` に変更し、key field を増やし、proof record / table / writer / accepted gate を module-private にし、orphan proof rejection を追加した。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で固定する。Resource IR、proof store、memo trait proof layer、PrivateCache / PrivateState implementation、prechecked artifact、Wasm / LLVM bytes、checker、compiler-known registry、artifact IO を import しないこと、caller supplied request table を public API authority にしないこと、diagnostic symbol / span / source text / `"memo_call"` 文字列を authority にしないこと、missing / refuted proof を successful plan として返さないこと、proof record / table / writer / accepted gate を module-private に保つこと、orphan proof を拒否すること、行数 / doc comment 長制限を追加しないことを確認する。

この checkpoint 後の残件は、producer-owned Resource proof boundary、PrivateCache / PrivateState effect masking、cache hit / miss / size / clear / raw identity の observation ban、sealed backend representation、MemoKey / MemoValue aggregate proof と backend request の接続、prechecked artifact / `.neplobj` / `.neplproof` stable key 投影である。現状の proof lookup は request 数を `m`、proof record 数を `p` とすると O(m * p) であり、sorted index 化、root / fingerprint bucket 化、artifact sidecar index 化は後続で行える。ただし exact key、duplicate rejection、orphan rejection、non-executable summary という semantic contract は今の stage で固定した。

## 2026-06-20 memo_call backend Resource observation producer stage0 checkpoint

`stdlib/neplg2/core/codegen/memo_call_backend_private_cache_proof_gate.nepl` に、Resource observation 由来の private-cache proof status を request-evidence proof table へ変換する module-private producer stage0 を追加した。

この checkpoint は actual Resource IR graph walker でも、PrivateCache / PrivateState effect masking でも、sealed cache backend representation でもない。ここで固定したのは、Resource 側の observation status を `PrivateCacheNoEscapeProven`、`PrivateCacheMayEscape`、`PrivateCacheMissing`、`PrivateCacheUnknown` に分け、未証明や未判定を成功扱いにしない変換境界である。`PrivateCacheNoEscapeProven` だけが既存の `RequestEvidenceProven` へ進み、`PrivateCacheMayEscape` は `RequestEvidenceRefuted` として既存 gate の `ProofRefuted` に流れる。`PrivateCacheMissing` と `PrivateCacheUnknown` は producer error のまま返し、request-evidence proof table へ保存しない。

Resource proof record / table / status は module-private である。public API に `SelfhostMemoCallBackendPrivateCacheResourceProofTable` や `SelfhostMemoCallBackendPrivateCacheResourceProofRecord` を渡す経路は作らない。これは、caller が forged Resource observation table を直接 accepted gate へ渡す trust bypass を作らないためである。stage0 fixture は同じ module 内で private writer を使い、外部 module へ accepted path を公開しないまま、NoEscape / MayEscape / Missing / Unknown / duplicate の代表ケースを確認する。

producer は Resource observation table を一度 `SelfhostMemoCallBackendPrivateCacheProofTable` へ変換し、その後で既存の module-private request-evidence gate を呼ぶ。既存 gate は引き続き HIR root から request table を内部再構築し、request entry を HIR payload と再照合するため、Resource observation table は caller supplied request table の代替 authority にならない。既存 gate が拒否した場合は `RequestEvidenceGateRejected` として包み、Resource producer 側の table 構築失敗、duplicate、Missing、Unknown と分けて診断できるようにした。

source policy は `nodesrc/test_selfhost_memo_call_backend_private_cache_proof_gate_contract.js` で更新した。private request-evidence 型と private Resource proof 型が public function signature や `pub struct` / `pub enum` に露出しないこと、`public entrypoint` という誤った説明が残らないこと、Missing / Unknown が `Result::Ok` へ畳まれないこと、MayEscape が refuted request-evidence として fail-closed に進むこと、行数や doc comment 量の制限を追加しないことを固定している。

この checkpoint 後も、actual Resource IR graph walker からこの Resource observation table を作る境界、private cache region の fresh / non-escape proof、cache hit / miss / size / clear / raw identity の observation ban、PrivateCache / PrivateState internal effect の surface fold、sealed backend representation、prechecked `.neplobj` / `.neplproof` stable key 投影は未完了である。Resource proof table lookup の sorted index 化や root / fingerprint bucket 化は後からできる最適化であり、今回固定した private type exposure ban、fail-closed status fold、HIR-root recheck contract を保つ限り後続で置換できる。

## 既存 issue との対応

現在の self-host 関連 issue は、この設計上では次の phase に属する。

| issue | status | phase | 設計への反映 |
|---|---|---|---|
| [SELFHOST-PARSER-AND-CHECKER-DO-NOT-IMPLEMENT-FULL-PREFIX...](../../issues/items/ISS-20260604T034255066Z-SELFHOST-PARSER-AND-CHECKER-DO-NOT-I-7C1C8941.md) | open | Phase 3 / Phase 5 / Phase 6 / Phase 7 | 2026-06-05 checkpoint で declaration header、type prefix reduction、generic type parameter environment、pre-HIR `SelfhostExprPrefixList`、body segmenter、`check/expr` call reduction、argument-scope ascription、DefId-linked callable candidate collection、literal argument type evidence、`NamedValue` / `%T NamedValue` value evidence、nested named call argument、明示 `@ident` function value argument、HIR `FnValue` identity、checked argument payload transport を接続した。さらに `UnitValue` checked payload を HIR `Unit` child、`NamedValue` checked payload を DefId-linked HIR `Var` child、`FunctionValue(candidate)` を HIR `FnValue` child、bool / i32 / f32 / char / string literal payload を対応する HIR literal child へ消費する direct call lowering を追加した。string escape decode、10 進 / 16 進 / 2 進 / 8 進 i32 payload、f32 payload、負数 numeric literal の `MinusMarker + IntLiteral/FloatLiteral` consume-width は source-backed checker 側で意味値へ正規化済みであり、lowering は source token を再読しない。2026-06-06 checkpoint では `SelfhostCheckedExprTree` が `DirectCall` / `BlockResult` / `BlockSequence` node を所有し、単一式 block、複数式 block sequence、nested `BlockIntro` block result を HIR `Block` child range へ下ろす。さらに単一 `left |> named_target suffix...` pipe を source-backed reducerで `DirectCall` checked tree へ正規化し、左辺と右辺 suffix の checked argument payload 順序を stage1 smoke で固定した。2026-06-11 checkpoint では `left |> %fn ... named_target suffix...` の target ascription を projection owner と型一致検査つきで同じ direct-call checked tree へ接続し、同名 overload narrowing も callable type equality で 0 / 1 / 複数一致に分類した。さらに `left |> named_target suffix... |> named_target suffix...` を左結合 checked tree として接続し、前段 root を `CheckedExpr` argument として次段へ渡す stage1 smoke を追加した。今回の follow-up では、注釈無し pipe target の複数候補を source-backed owner reducer の投機実行ではなく、左辺 / suffix の borrowed probe で `Match` / `NoMatch` / `SourceBackedRequired` / `SelectionBlockedUnsupported` に分類する narrowing へ進めた。一意一致だけを既存 finisher へ渡し、0 件は `PipeTargetNoApplicableCandidate`、複数一致、source-backed required 混在、selection-blocked unsupported は `PipeTargetAmbiguous` で止める。さらに `Match` が 0 件、`SourceBackedRequired` が 1 件、`SelectionBlockedUnsupported` が 0 件なら、その候補だけを既存 source-backed finisher へ渡すため、`1 |> add %i32 2` と `1 |> add x` は2引数版 direct call として成功する。`x` は `SelfhostValueTypeEvidenceTable` の DefId-linked evidence で `NamedValue` payload になる。`1 |> add sum 2 3` は inner `sum` を単一候補 nested callとして checked tree の child `DirectCall` へ接続し、outer call には `CheckedExpr` payload を渡す。`1 |> add add 2 3` は source-backed nested argument の final-range narrowing として、inner `add` の候補を borrowed complete consumption と expected result で一意化し、child `DirectCall` へ接続した。さらに `outer add 1 2 3` は outer continuation 付き nested overload narrowing として、inner `add` の消費後 `next_index` から outer `outer` の残り parameter を借用検査して一意化する。`add 1 2 |> use 3` は pipe left の `FinalRange` として、pipe target suffix を left segment の continuation に使わない。単一 `left |> named_target:` と `left |> named_target suffix... |> named_target:` は trailing block を最終 callable の残り parameter の `BlockResult` checked argument として扱う。binary / octal literal follow-up では、self-host lexer が `0b` / `0B` と `0o` / `0O` を `IntLiteral` token として切り、source-backed checker が radix 2 / 8 として semantic `i32` payload へ正規化する。HIR call identity follow-up では、`SelfhostHirCallExpr` が表示名だけでなく DefId、callee function type、effect、monomorphic type-arg count を保持し、direct-call lowering と checked-tree lowering が selected `SelfhostCallableCandidate` から typed callee identity を作るようにした。`Unique` などの generic candidate は stable type-argument identity が無いため `GenericCallIdentityUnsupported` で拒否する。今回の memoized function value HIR boundary follow-up では、`SelfhostHirExprPayload::MemoizedFunctionValue` を typed function identity payload で追加し、DefId あり / monomorphic / Pure の identity だけを accepted にした。DefId 欠落、generic identity、impure identity は typed error で fail-closed にする。残件は lambda / borrow / source-backed pipe suffix の lambda など複合式 success path、generic instantiation inference、trait solving、numeric suffix / defaulting beyond current fixed `i32` / `f32`、indirect call、`memo_call` compiler-known source identity 判定 / `MemoKey` / `MemoValue` / private cache proof / backend representation、cross-arena serialized canonical key / fingerprint、nested generic binder depth / stable binder identity。 |
| [SELFHOST-TYPE-AND-HIR-RANGES-ALLOW-INVALID...](../../issues/items/ISS-20260604T034255467Z-SELFHOST-TYPE-AND-HIR-RANGES-ALLOW-I-A4509F7E.md) | fixed | Phase 1 | HIR child / parameter range と function type argument range の checked constructor と defensive equality として反映 |
| [SELFHOST-SOURCESPAN-CAN-REPRESENT-NEGATIVE...](../../issues/items/ISS-20260604T034255819Z-SELFHOST-SOURCESPAN-CAN-REPRESENT-NE-644AA655.md) | open | Phase 1 | SourceSpan validation proof slice として反映 |
| [SELFHOST-PARSER-MIXES-CURRENT-PERCENT-SYNTAX-WITH-LEGACY...](../../issues/items/ISS-20260604T034256529Z-SELFHOST-PARSER-MIXES-CURRENT-PERCEN-3647B103.md) | open | Phase 2 / Phase 3 | 正規構文と migration diagnostic の分離として反映 |
| [SELFHOST-PARSER-TOKEN-ROLE-CLASSIFICATION...](../../issues/items/ISS-20260604T034256890Z-SELFHOST-PARSER-TOKEN-ROLE-CLASSIFIC-913AF123.md) | open | Phase 2 | token role classification の単一 authority として反映 |
| [SELFHOST-PARSER-LOOP-EXPOSES-LONG-STATE...](../../issues/items/ISS-20260604T034257629Z-SELFHOST-PARSER-LOOP-EXPOSES-LONG-ST-D57B1E8E.md) | open | Phase 3 | parser state transition と no-progress guard として反映 |

新規 issue は、この文書の phase を基準に小さく分割する。特に parser、type resolver、Resource IR、memo_call、incremental cache は一つの巨大 issue にまとめない。

## 完了条件

セルフホスト設計としての完了条件は次である。

- NEPLg2.1 syntax が旧 NEPLg2.0 syntax と混同されていない。
- Rust 実装の authority boundary が設計へ反映されている。
- Resource IR 静的検査が self-host compiler の safety authority として位置づけられている。
- compile performance 改良が cache だけでなく探索範囲と計算量の削減として設計されている。
- `.neplmeta`、`.neplproof`、`.neplobj` の役割が分離されている。
- `memo_call` と private effect が Pure / Impure の表層二値と矛盾しない。
- `stdlib/neplg2/` の既存 skeleton を活かしつつ、古い NEPLg2.0 記述を NEPLg2.1 へ更新する道筋が明確である。
