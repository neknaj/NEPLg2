# NEPLg2.1 セルフホストコンパイラ設計

最終更新: 2026-06-04

## 位置づけ

この文書は、`stdlib/neplg2/` に実装する NEPLg2.1 セルフホストコンパイラの設計を定義する。

既存の `doc/neplg2/self_host_plan.md` は NEPLg2.0 時点の計画であり、現在の NEPLg2.1 表層構文、`void` marker、`#test`、compiler artifact、Resource IR 静的検査、compile-time performance 改良を十分に反映していない。したがって、今後の `stdlib/neplg2/` 実装はこの文書を正規の設計入口とする。

この設計は NEPLg3 仕様の実装ではない。NEPLg3 文書は今後も変更され得る参考資料であり、NEPLg2.1 セルフホストコンパイラは現行 Rust 実装の NEPLg2.1 を基準にする。

`doc/self_host.md` は NEPLg2.1 と NEPLg3 の総合入口である。NEPLg2.1 の詳細はこの文書、NEPLg3 の詳細は `doc/neplg3/` 以下の文書で扱う。

## 設計原則

セルフホストコンパイラは、Zenn の開発方針で示された試作段階の原則を満たす。特に、次を設計上の制約として扱う。

- 互換性維持より、正しい仕様境界と根本原因の修正を優先する。
- core は platform 非依存に保ち、filesystem、stdio、argv、環境変数、時計、乱数などは CLI / host boundary に閉じ込める。
- 静的検査を省略して高速化したように見せない。高速化は探索範囲、依存関係、cache key、事前検査済み artifact の設計で行う。
- 失敗は `Option` / `Result` / enum diagnostic として表現し、string 判定や sentinel 値へ依存しない。
- public API に型名付き重複関数を増やさず、型注釈、期待型、trait / generic 解決で選択する。
- prototype 段階では大きな再設計を許すが、後続実装へ隠れた技術的負債を渡さない。

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
- `%`、`\`、`:`、`|>`、`.`、`,`、`@function`
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

現行 self-host 実装では、`stdlib/neplg2/core/syntax/ast/prefix_expr.nepl` が `SelfhostExprPrefixList` を提供する。これは `SelfhostSyntaxRange` から作る pre-HIR の flat expression item list であり、`%` type annotation marker、lambda marker、`@function` marker、literal、identifier、control form marker を token index と span 付きで保持する。`SelfhostExprPrefixList` は HIR ではなく、`SelfhostHirExprPayload::Call` のような解決済み call tree を作らない。body parser は `module_parser/body_range.nepl` で declaration body envelope と first expression segment を `SelfhostSyntaxRange` として切り出す。`parser/body_segmenter.nepl` は body envelope を top-level segment 列へ分解し、flat prefix expression にできる `ExpressionLine` と nested body を持つ `BlockIntro` を型で分ける。`BlockIntro.body` は recursive segmenter の入力であり、`SelfhostExprPrefixList` へ直接渡さない。

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
- `@function name` function value identity construction
- `memo_call` compiler-known primitive

現行 self-host 実装では、`stdlib/neplg2/core/check/expr/` が expression checker の初期境界を持つ。`SelfhostTypeExpectation` は expected type の `SelfhostTypeId` だけでなく、`ExplicitAscription` / `BlockResult` / `OuterConsumerArgument` の由来と span を保持する。`SelfhostCallableCandidate` は候補名、function type であるべき TypeId、effect、generic inference state、span を保持する。call reduction はこれらと `SelfhostExprPrefixList` を受け取り、HIR をまだ生成せず `SelfhostCallReduceResult::DirectCall` または `SelfhostCallReduceError` を返す。`check/expr/body_line.nepl` は `SelfhostBodySegmentKind::ExpressionLine.head` から prefix list を作ってこの境界へ渡す接続口である。`check/expr/candidate_collection.nepl` は prefix head の identifier spelling を token span から復元し、`SelfhostNameScope` の function namespace と `SelfhostCallableSignatureTable` の DefId-linked signature evidence から reducer 用 candidate vector を作る。`BlockIntro` は nested body envelope なので `NotExpressionLine` として拒否し、prefix list 生成失敗と call reduction 失敗は別の enum variant に保つ。

2026-06-05 checkpoint では、`%T expr` ascription を `SelfhostTypeExpectation::ExplicitAscription` へ変換する初期接続を追加した。`resolve/type_resolver/reduce.nepl` は flat type prefix list の先頭 1 type expression だけを reduce し、`SelfhostTypePrefixReducePrefixResult.next_index` によって expression tail の開始位置を caller へ返す。`check/expr/ascription.nepl` は `%` の直後から type prefix candidate を集め、type tree を `SelfhostTypeArena` へ projection し、残りの token range を inner expression として返す。`check/expr/body_line.nepl` は先頭 token が `%` の場合だけこの ascription projection を使い、`%` 自体を call head として call reducer へ渡さない。

2026-06-05 candidate collection checkpoint では、line head に対する最初の名前解決境界を追加した。`SelfhostCallableSignatureTable` は現在 `Vec` による insertion-order table で、DefId lookup は O(s) だが、public API は表現を隠している。これにより後続の `.neplmeta` interface artifact や prechecked signature index へ差し替えるとき、call reducer や parser の契約を変えずに lookup 実装だけを置き換えられる。名前が見つからない場合は空 candidate list を返し、`UnresolvedName` diagnostic は call reduction 側に集約する。binding があるのに DefId や signature が欠ける場合は `PendingBinding` / `MissingSignature` として fail-closed にする。

この境界は owner を明示する。ascription projection は `SelfhostTypeArenaAlloc` を保持する success payload を返し、caller は `into_arena` で arena を受け取るか `free` で破棄する。これは borrowed arena から projection result を取り出す設計では lifetime が表現できないためであり、Rust 実装で得た ownership boundary の知見を self-host stdlib 側へ反映したものである。

この初期境界は fail-closed である。先頭 item が named value で、候補が 1 つだけで、候補 type が function type で、引数数が完全一致し、expected result と候補 result が同じ arena 内で構造一致する場合だけ direct call plan を返す。候補が複数ある場合は、現段階では expected type による narrowing を行わず `OverloadAmbiguous` にする。generic inference state が `EvidenceMissing` / `Conflict` / `Unsupported` の場合は、それぞれ typed error に分ける。これにより、未完成の generic solver や overload solver が成功として後段へ流れない。

部分適用は許可しない。`add 1` が `fn i32 i32` を要求する文脈であっても、NEPLg2.1 の一般規則として関数値を暗黙生成しない。関数値が必要な場合は `@function name` や明示的な lambda を使う。

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

- 引数は明示的な `@function name` で得た non-capturing named pure function value に限定する。
- user が定義した同名の通常関数を `memo_call` primitive として扱わない。compiler-known 判定は stdlib の正確な module / symbol identity に限定する。
- `memo_call @function f arg` のような即時適用は Phase 1 では受理しない。まず `memo_call @function f` が memoized function value を返す境界を固定する。
- `func` は monomorphic な 1 引数 pure function とする。複数引数は tuple key 化を別段階にする。
- `K` と `V` は Copy または conservative MemoKey / MemoValue として認められる型に限定する。
- cache region は compiler が fresh に導入し、public type に現れない。
- cache lookup は owned / copied / cloned value を返し、cache 内部参照を返さない。
- hit/miss、size、clear、stats、address identity、allocation state を pure API として公開しない。

Resource IR では `MemoizedFunctionValue` と `PrivateCache` / `PrivateState` proof boundary を持つ。cache implementation correctness は trusted stdlib primitive と tests の責務とし、compiler は effect escape と public observation を検査する。

現行 Rust 実装は、`memo_call @function f` の typecheck / HIR 境界を先に持つ段階である。sealed backend cache representation と通常 compile path の private-cache mask proof は未完成のため、セルフホスト設計でも「backend private cache は fail-closed」「SourceCapability と private-cache mask proof は別 authority」として扱う。

Phase 2 では、`run_private` / `mask_private` に相当する一般 private region effect へ拡張する。

## HIR

HIR は型検査後、Resource IR lowering 前の typed source-level IR である。

HIR は次を持つ。

- expression kind
- type id
- surface effect
- source span
- function value identity
- indirect call
- memoized function value
- raw body marker

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
- `@function` と function value identity を Rust 実装に合わせる。
- indirect call と pure / impure call boundary を検査する。

Issue slice:

- argument type checking を含む prefix call reduction stack
- ascription expectation と outer expected type が衝突した場合の diagnostic 統合
- generic instantiation inference
- trait bound solving
- no partial application diagnostics
- `@function` identity and indirect call
- pure context effect diagnostics

Completed checkpoint:

- `ExpressionLine.head` から `SelfhostExprPrefixList` を作り `check/expr` へ渡す接続
- `%T expr` から `SelfhostTypeExpectation::ExplicitAscription` を作り、inner expression tail だけを call reducer へ渡す接続
- `ExpressionLine.head` の identifier を `SelfhostNameScope` と callable signature table へ通し、DefId-linked candidate list を call reducer へ渡す初期接続

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

- `@function` pure named function restriction
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

## 既存 issue との対応

現在の self-host 関連 issue は、この設計上では次の phase に属する。

| issue | status | phase | 設計への反映 |
|---|---|---|---|
| [SELFHOST-PARSER-AND-CHECKER-DO-NOT-IMPLEMENT-FULL-PREFIX...](../../issues/items/ISS-20260604T034255066Z-SELFHOST-PARSER-AND-CHECKER-DO-NOT-I-7C1C8941.md) | open | Phase 3 / Phase 5 / Phase 6 | 2026-06-05 checkpoint で declaration header の `%` type annotation range と lambda header range を typed evidence 化し、module checker / proof solver が function 宣言の range presence と containment を検査するようにした。続く checkpoint で `resolve/type_resolver` の flat type prefix item input、TypeId 割当前の resolved type tree reduction、primitive / function の `SelfhostTypeArena` projection、arity 0 named constructor lookup projection、constructor kind に基づく generic type application reduction / projection、`SelfhostTypeId` を payload に持たない canonical type key projection、generic type parameter environment と `Parameter` resolved node への reduction、binder-indexed type parameter の arena/key projection、constructor kind validation と bound plan、pre-HIR `SelfhostExprPrefixList`、declaration body envelope / first expression range 抽出、body envelope からの `ExpressionLine` / `BlockIntro` segmenter、`ExpressionLine.head` から `check/expr` call reduction 初期境界への接続、`%T expr` から `SelfhostTypeExpectation::ExplicitAscription` と inner expression tail を作る接続、line head の DefId-linked callable candidate collection を追加した。残件は argument type checking、ascription expectation と outer expected type の diagnostic 統合、generic instantiation inference、trait solving、`@function` / indirect call、cross-arena serialized canonical key / fingerprint、nested generic binder depth / stable binder identity。 |
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
