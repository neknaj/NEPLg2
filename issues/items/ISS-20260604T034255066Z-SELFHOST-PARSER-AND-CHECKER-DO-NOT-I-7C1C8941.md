---
id: ISS-20260604T034255066Z-SELFHOST-PARSER-AND-CHECKER-DO-NOT-I-7C1C8941
title: "selfhost parser and checker do not implement full prefix expression and type range contracts"
area: selfhost
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-11
target: "stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/check/checker.nepl"
---

# ISS-20260604T034255066Z-SELFHOST-PARSER-AND-CHECKER-DO-NOT-I-7C1C8941: selfhost parser and checker do not implement full prefix expression and type range contracts

## 概要

Subagent audit found module_parser.nepl explicitly stating it is not a full expression parser, and checker.nepl still treating later stages as unimplemented. This conflicts with plan.md, where prefix expression ranges and type-stack reduction are central to NEPLg2, and with the Zenn policy of making static checks part of the core contract rather than surface simulation.

## 対象

- `stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/check/checker.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found module_parser.nepl explicitly stating it is not a full expression parser, and checker.nepl still treating later stages as unimplemented. This conflicts with plan.md, where prefix expression ranges and type-stack reduction are central to NEPLg2, and with the Zenn policy of making static checks part of the core contract rather than surface simulation.

## 影響

Selfhost compiler modules cannot validate the same prefix argument and % type ranges that the Rust compiler and Web highlighting now rely on, so stdlib/neplg2 tests can pass smoke cases while missing core language invariants.

## 修正方針

Implement or stage a real PrefixList/TypePrefixList parser boundary, connect checker range validation, and keep partial parser smoke paths marked as transitional rather than public compiler contract.

## 進捗

- 2026-06-05: `SelfhostSyntaxRange` を追加し、module declaration header が `%` type annotation range と lambda header range を typed evidence として保持するようにした。`module_parser/prefix_range.nepl` は token stream 上の flat range だけを切り、型木・式木・call boundary は parser では確定しない。
- 2026-06-05: module checker / proof solver は function 宣言で type annotation range と lambda header range が nonempty かつ header span 内にあることを検査するようにした。非 function 宣言では lambda/type range を受け付けない。
- 2026-06-05: `resolve/type_resolver` を追加し、parser の `%` annotation range から `%` marker を除いた flat type prefix item list を作る resolver input 境界を実装した。`void` は専用 marker item、`unit` は通常の named type item として分類し、まだ `TypeId` は生成しない。
- 2026-06-05: `resolve/type_resolver` の flat type prefix item list を TypeId 割当前の `resolved` tree へ縮約する reducer を追加した。`fn i32 fn i32 i32` は nonempty function type として flatten し、`fn void fn unit unit` は 0 引数 function が function を返す nested type として保持する。plan / validation / build は別 module に分割し、build 層が source string を再読しない境界にした。
- 2026-06-05: `resolved` tree root を `SelfhostTypeArena` へ投影する `project.nepl` を追加した。primitive / function type は arena-local `SelfhostTypeId` を得られるようになり、named type は type constructor table 未接続のため `UnsupportedNamedType` として fail-closed にした。
- 2026-06-05: `core/ty/ty` に `SelfhostNamedTypeId` と `SelfhostTypeRecord::Named` を追加し、`resolve/type_resolver/constructor.nepl` の constructor table から arity 0 named type を `SelfhostTypeArena` へ投影できるようにした。constructor table なし API は引き続き named type を拒否し、unknown named type / bare generic constructor は typed error で fail-closed にした。
- 2026-06-05: constructor-aware reducer が constructor table header に従って generic type argument list を再帰消費し、`SelfhostResolvedTypeNode::Applied` へ縮約するようにした。constructor-aware projection は `SelfhostTypeRecord::Applied` と arena type argument table へ投影し、constructor table なし projection は Applied node を `UnsupportedNamedType` として fail-closed にする。
- 2026-06-05: `core/ty/ty/key.nepl` を追加し、arena-local `SelfhostTypeId` を payload に入れない `SelfhostCanonicalTypeKeyArena` へ type record tree を投影できるようにした。primitive / named / applied / function key node と key argument table を持ち、projection は型 record / argument edge 数に対して O(n) である。同じ key arena 内の structural equality も追加した。
- 2026-06-05: `resolve/type_resolver/typeparam` を追加し、generic binder から作る `SelfhostTypeParameterEnv` と `SelfhostTypeParameterId` を named constructor identity から分離した。`selfhost_type_prefix_list_reduce_with_constructors_and_type_parameters` は `T` / `E` を `SelfhostResolvedTypeNode::Parameter` へ縮約し、constructor と parameter が同名の場合は `TypeParameterConstructorNameConflict` として fail-closed にする。
- 2026-06-05: `core/ty/ty` に `SelfhostTypeParameterBinding`、`SelfhostTypeRecord::Parameter`、`SelfhostCanonicalTypeKeyNode::Parameter` を追加した。type resolver projection は `SelfhostTypeParameterId` を現在 binder の `binder_depth = 0` と `parameter_index` へ正規化して TypeArena に保存し、canonical key equality は source spelling / span / arena-local `TypeId` ではなく binder identity で比較する。
- 2026-06-05: user-defined type constructor header を `SelfhostTypeConstructorKind` へ正規化し、負 arity、予約名、同一 table 内の重複名を `SelfhostTypeConstructorTableErrorKind` として登録時に拒否するようにした。constructor / type parameter lookup は `SelfhostTypeBoundPlan` に束縛し、validate と build が同じ lookup 結果を共有するため、reducer は raw `arity` や source span lookup を通常経路で繰り返さない。旧 constructor-aware validate/build helper は削除し、公開 API から bound plan を迂回できないようにした。
- 2026-06-05: constructor-aware projection でも `Applied` node の `SelfhostNamedTypeId` を constructor table で再検査し、constructor kind の型引数数と applied argument range が一致しない resolved tree を `GenericConstructorArgumentArityMismatch` として拒否するようにした。reducer 由来でない public resolved-tree constructor から不正な `SelfhostTypeRecord::Applied` が TypeArena に入る経路を閉じた。
- 2026-06-05: `syntax/ast/prefix_expr.nepl` を追加し、parser / focused test 由来の `SelfhostSyntaxRange` を pre-HIR の flat `SelfhostExprPrefixList` へ変換する入力境界を作った。`%` type annotation marker、lambda marker、`@` marker、literal、identifier、control form marker を token index / span 付き enum payload として保持し、call tree / HIR / TypeId / DefId allocation は行わない。`void` は expression start として拒否し、legacy grouping token 混入は typed build error にする。
- 2026-06-05: `module_parser/body_range.nepl` を追加し、declaration body block の envelope と first expression segment を `SelfhostSyntaxRange` として保持するようにした。parser は body を HIR / call tree へ落とさず、単純な function body では `declaration_body.first_expression` から `SelfhostExprPrefixList` を構築できることを focused doctest で確認した。複数式 body / nested block は envelope を後段 segmenter へ渡す設計にした。
- 2026-06-05: `syntax/parser/body_segmenter.nepl` を追加し、declaration body envelope を `ExpressionLine` / `BlockIntro` の typed segment list へ分解するようにした。`ExpressionLine.head` は `SelfhostExprPrefixList` の入力候補、`BlockIntro.body` は recursive segmenter の入力として分離し、nested body を flat prefix list に直接渡さない契約を source policy と doctest で固定した。
- 2026-06-05: `check/expr` を追加し、`SelfhostExprPrefixList` と callable candidate list を受け取る call reduction の初期境界を実装した。`SelfhostTypeExpectation` は expected type の由来と span を保持し、generic inference state / overload rejection / call reduction error は enum payload で分ける。現段階では named direct call の arity / expected result / no partial application / ambiguity / generic fail-closed を検査し、HIR lowering は行わない。
- 2026-06-05: `check/expr/body_line.nepl` を追加し、`SelfhostBodySegmentKind::ExpressionLine.head` から `SelfhostExprPrefixList` を作って `selfhost_call_reduce_prefix` へ渡す接続を実装した。`BlockIntro` は `NotExpressionLine`、prefix build failure は `PrefixBuildFailed`、call reduction failure は `CallReduceFailed` として分け、nested body を flat prefix list に直接渡さない契約を source policy と focused doctest で固定した。
- 2026-06-05: `resolve/type_resolver` に先頭 1 型式だけを縮約して `next_index` を返す `selfhost_type_prefix_list_reduce_prefix*` API を追加した。`TrailingItems` を返す full annotation reducer と分け、`%i32 add 1 2` のような ascription 入力で後続 expression token を誤って型式 trailing item として拒否しない境界にした。
- 2026-06-05: `check/expr/ascription.nepl` を追加し、`%T expr` を `SelfhostTypeExpectationSource::ExplicitAscription` と内側 `SelfhostSyntaxRange` へ投影する owner 付き入口を実装した。`body_line.nepl` には arena owner を受け取る `selfhost_check_expr_reduce_body_segment_with_arena` を追加し、`%` で始まる expression line は call reduction へ直接渡さず、ascription projection 後の内側 expression だけを縮約する。
- 2026-06-05: `stage1` smoke helper に `%i32 add 1 2` の固定 token fixture を追加した。lexer / parser の詳細ではなく、type resolver が返す型式消費境界と body line connector の owner 戻しを確認する fixture とした。
- 2026-06-05: `check/expr/candidate_collection.nepl` を追加し、`ExpressionLine.head` の identifier を `SelfhostNameScope` の function namespace で解決し、DefId に対応する `SelfhostCallableSignatureTable` record から call reducer 用 `SelfhostCallableCandidate` list を構築する初期境界を実装した。名前なしは空候補として reducer の `UnresolvedName` に集約し、DefId / signature 不整合は `PendingBinding` / `MissingSignature` として fail-closed にする。
- 2026-06-05: `check/expr/argument.nepl` を追加し、literal argument item から得られる型証拠を function parameter type と照合する初期境界を実装した。`UnitValue` / `IntLiteral` / `BoolLiteral` / `CharLiteral` / `StringLiteral` は primitive type evidence として扱い、`FloatLiteral`、`NamedValue`、nested call、block、lambda、`@ident`、ascription 付き argument など full expression checker が必要なものは source-less borrowed API では成功扱いせず fail-closed にする。`add true 1` のように arity と expected result だけでは見逃す direct call を拒否する focused smoke と source policy も追加した。
- 2026-06-05: `check/expr/body_line.nepl` の owner 付き ascription 入口で、`%T expr` の `ExplicitAscription` expectation と外側 context の expected type を照合するようにした。同じ `SelfhostTypeArena` 内で一致しない場合は、内側 call reduction へ進まず `AscriptionExpectedTypeConflict` を返す。error payload は arena 解放後も安全に読める source / span evidence だけを保持し、arena-local `SelfhostTypeId` は残さない。
- 2026-06-05: call reducer の raw `item_count - 1` argument count 依存をやめ、parameter index と prefix item cursor を分けた argument expression consume-width 境界へ移した。現 checkpoint では単一 literal item だけが `SelfhostExprArgumentMatch.next_index` を返して成功し、`%T literal` は source / token backed argument checker が未接続のため `UnsupportedArgumentExpression` として fail-closed にする。これにより `add %i32 1 2` 相当の flat prefix item 列を raw 4 argument と誤分類せず、後続の argument-scope ascription 検査へ接続できる。
- 2026-06-05: source / token backed の argument-scope ascription 検査を追加し、`selfhost_call_reduce_prefix_with_source` から `%T literal` を 1 つの argument expression として縮約できるようにした。`SelfhostExprArgumentOwnedMatch` / `SelfhostCallReduceOwnedResult` は `SelfhostTypeArena` owner を返すため、projection 後の expected type を arena-local id のまま安全に比較できる。source-less borrowed reducer は引き続き `%T literal` を成功扱いせず fail-closed にし、source と token を持つ入口だけが `add %i32 1 2` を 2 引数 direct call として受理し、`add %bool 1 2` を `ArgumentAscriptionExpectedTypeConflict` として拒否する。`%T` head の projection 自体が失敗した場合は `ArgumentAscriptionProjectionFailed` として span を保持し、empty-span の unsupported error へ潰さない。
- 2026-06-05: `check/expr/value_evidence.nepl` を追加し、source / token backed の `NamedValue` argument と `%T NamedValue` argument を DefId-linked な型証拠で検査するようにした。`SelfhostValueTypeEvidenceTable` は scope binding の `SelfhostDefId` と arena-local `SelfhostTypeId` を結び、名前の spelling だけでは成功させない。binding 欠落、DefId 未割当、値として扱えない binding kind、型証拠欠落を `NamedValue*` error として分け、call reducer 側では `ArgumentNamedValue*` error へ投影する。`add x 2` と `add %i32 x 2` は `x: i32` の証拠が登録済みの場合だけ受理し、binding だけでは `ArgumentNamedValueEvidenceMissing` として拒否する。
- 2026-06-05: source / token backed の nested named call argument 検査を追加した。`candidate_collection.nepl` は prefix 全体の先頭ではない `SelfhostExprPrefixItem` head からも function candidate を集め、`call_reduce.nepl` は外側 parameter の expected type を nested call の result expectation として使う。`add add 1 2 3` では内側 `add 1 2` が第1引数として消費され、返された `next_index` により最後の `3` が外側 call の第2引数として残る。候補 0 件のときだけ NamedValue value evidence へ fallback し、候補収集の DefId / signature 不整合は `ArgumentNestedCandidate*` として fail-closed にする。subagent review で指摘された shadowing 退行も修正し、最新の可視 binding が local / parameter などの non-function なら、古い同名 function を復活させず value evidence fallback へ進む。
- 2026-06-05: BlockIntro 専用の trailing block argument 境界を追加した。通常の `ExpressionLine` reducer は引き続き `BlockIntro` を `NotExpressionLine` として拒否する。一方、`selfhost_check_expr_reduce_block_intro_with_arena` は `BlockIntro.head` だけを prefix list にし、`BlockIntro.body` を `SelfhostTrailingBlockArgument` として source-backed call reducer へ渡す。この時点では block body result checker が未接続だったため、必要な parameter 位置では `TrailingBlockArgumentUnsupported`、parameter 消費後に余った block では `UnexpectedTrailingBlockArgument` として fail-closed にした。これにより、末尾 block argument 未対応を partial application / raw arity error へ潰さず、次 slice で `BlockResult` expectation に接続する API 境界を固定した。
- 2026-06-05: source / token backed の block body result checker を接続した。`check/expr/block_body.nepl` は `BlockIntro.body` を `selfhost_body_segment_list_from_envelope` で再帰分解し、単一 `ExpressionLine` の場合だけ prefix list と callable candidate list を構築する。call reducer は不足 parameter type を `SelfhostTypeExpectationSource::BlockResult` として nested expression reducer へ渡し、`add 1:\n    add 1 1` のような direct call body を 1 実引数式として消費できる。この時点では空 body、複数 segment、nested `BlockIntro`、prefix build failure、candidate collection failure を `TrailingBlockBody*` error として分け、余分な block は引き続き `UnexpectedTrailingBlockArgument` として拒否した。複数 segment の成功経路は 2026-06-06 の `BlockSequence` checked tree slice で接続済みである。
- 2026-06-05: block body result checkpoint の広域検証で、既存の `source_text_*` と `std/fs/path/normalize*` が `Vec` owner の更新・解放を行うのに pure function として公開されていることを検出した。所有者を作る・更新する・閉じる関数は `impure fn` として明示し、doc comment と source policy を effect 契約に合わせた。
- 2026-06-05: focused doctest を止めていた既存 effect 境界も修正した。`selfhost_diagnostics_push` / `selfhost_diagnostics_free` / `lex_stack_drop_top` は `Vec` owner の更新または解放を行うため `impure fn` に正規化し、`lex_stack_drop_top` は引き続き public `drop_last` API へ委譲して `Vec` 内部 storage layout へ依存しない。
- 2026-06-05: source / token backed の明示 function value argument 検査を追加した。Rust 実装と同じく正規表層構文は `@` token 直後の identifier であり、`@function name` keyword 形式ではない。`takes @add` のような argument は expected parameter type が function type で、対象名が callable signature table から monomorphic candidate として一意に解決され、candidate function type が expected type と同じ arena 内で構造一致する場合だけ成功する。`takes add` のような bare function name は partial application / implicit function value として扱わず拒否する。
- 2026-06-05: HIR expression model に `SelfhostHirExprPayload::FnValue` と `SelfhostHirFunctionValueIdentity` を追加した。function value identity は symbol だけではなく、`Option SelfhostDefId`、関数型 `SelfhostTypeId`、`SelfhostEffectKind`、`type_arg_count` を持つ。`def_id = None` や generic type argument 実体未接続の identity を accepted indirect call / `memo_call` へ流さないため、HIR の受け皿だけを先に typed payload として固定した。
- 2026-06-05: `SelfhostCallableCandidate` に `SelfhostDefId` を追加し、`SelfhostCallableSignature` から reducer 用 candidate へ変換するときに `signature.def_id` を落とさないようにした。stage0 / stage1 の手作り fixture も DefId を明示し、後続の HIR `FnValue` lowering が name string と function type だけへ退行しないよう source policy で固定した。
- 2026-06-05: `core/lower/hir/function_value.nepl` を追加し、DefId 付き `SelfhostCallableCandidate` から `SelfhostHirFunctionValueIdentity` と `SelfhostHirExprPayload::FnValue` expression record を作る境界を分離した。`candidate.def_id` は `Option::Some` として HIR identity に入れ、generic candidate は stable type-argument range / canonical key が無いため `GenericUnsupported` として拒否する。`check/expr` は HIR record を直接生成しない境界を保つ。
- 2026-06-05: `check/expr/argument_payload.nepl` を追加し、実引数式ごとの checked evidence を `SelfhostCheckedArgument` として保持するようにした。`unit` は `UnitValue`、scope と value evidence で照合済みの値参照は `NamedValue(SelfhostCheckedValueIdentity)`、`@ident` は `FunctionValue(candidate)` payload を持つ。nested named call は `NestedDirectCall(candidate)`、末尾 block result は `BlockResult` として summary を残す。`check/expr` は HIR を直接生成せず、後続の `lower/hir` がこの payload を消費する。
- 2026-06-05: `lower/hir/direct_call.nepl` を追加し、`SelfhostCallReduceResult::DirectCall` と `Vec SelfhostCheckedArgument` を消費して HIR child expression と parent call expression を作る初期 lowering を実装した。現 checkpoint では `UnitValue` を HIR `Unit` child、`NamedValue(identity)` を DefId-linked HIR `Var` child、`FunctionValue(candidate)` を HIR `FnValue` child にする。`TypedExpression` / `NestedDirectCall` / `BlockResult` は lowerable payload が不足しているため `UnsupportedArgumentKind` として fail-closed にする。callee は `candidate_index` から candidate table を読むだけで、prefix token や scope lookup を再実行しない。
- 2026-06-05: HIR expression model の `Var` payload を `str` から `SelfhostHirValueIdentity` へ変更した。`argument.nepl` は `name -> latest binding -> DefId -> value type evidence` の成功時に `SelfhostCheckedValueIdentity` を作り、`direct_call.nepl` はその payload だけを使って HIR `Var` child を作る。lowering は source token、scope lookup、value evidence lookup を再実行せず、DefId / 型 / binding kind を持つ variable identity を保持する。
- 2026-06-05: `check/expr/literal_payload.nepl` を追加し、source-backed checker が bool / 10 進 i32 / escape なし string literal の意味値を `SelfhostCheckedArgumentKind::BoolLiteral` / `I32Literal` / `StrLiteral` として保存するようにした。`lower/hir/direct_call.nepl` はこの payload だけを使って HIR `BoolLiteral` / `I32Literal` / `StrLiteral` child を作り、source token や literal lexeme を再読しない。hex integer は `ArgumentLiteralI32RadixUnsupported`、escape 付き string は `ArgumentLiteralStringEscapeUnsupported` として fail-closed にした。
- 2026-06-05: Zenn 方針と AGENTS.md のコメント方針に照らした subagent review を受け、`literal_payload.nepl` の helper doc comment を目的・契約・戻り値/エラー条件・計算量つきで補強した。`string::str_slice` fallback をやめ、`string::str_slice_result` の失敗を `StringSliceFailed` / `LiteralStringSliceFailed` / `ArgumentLiteralStringSliceFailed` として typed error に写すよう修正した。radix 判定は局所値へ分け、char literal は明示 branch で fail-closed payload に残す。
- 2026-06-05: char literal payload を `SelfhostCheckedArgumentKind::CharLiteral` として追加した。source-backed checker は simple char、simple escape、`\xHH`、`\u{...}` を semantic `char` へ decode し、malformed quote、未対応 escape、不正 scalar、複数 scalar を typed error として分ける。`lower/hir/direct_call.nepl` は現 Rust 実装と同じく `char_to_i32` で i32-backed HIR literal へ下ろし、source token や literal lexeme を再読しない。
- 2026-06-06: string literal payload の escape decode を追加した。Rust string literal と同じ `\n` / `\r` / `\t` / `\\` / `\"` / `\0` / `\xHH` だけを semantic `str` へ decode し、char 専用の `\b` / `\f` / `\'` / `\u{...}` は `StringEscapeUnsupported` として fail-closed にした。escape なしは `str_slice_result` fast path のまま、escape ありは `StringBuilder` owner path で `StringEscapeMalformed` / `StringBuildFailed` / `StringSliceFailed` を typed error として分けた。`SelfhostCheckedArgumentKind::StrLiteral` が decode 済み value を持つため、HIR lowering は source token を再読しない。
- 2026-06-06: numeric literal payload の Rust parity slice として、source-backed checker が接頭辞なし 10 進 `IntLiteral` と `0x` / `0X` 16 進 `IntLiteral` を semantic `i32` payload へ正規化するようにした。`SelfhostLiteralI32RadixPlan` が token-local lexeme から radix と digit body 範囲を分け、接頭辞除去後の body を `string::to_i32_radix` に渡す。空 hex body、無効 digit、decimal / hex overflow は `I32Invalid`、将来同一 token として渡る未対応 `0b` / `0o` は `I32RadixUnsupported` へ fail-closed にする。suffix は現行 Rust/selfhost lexer とも numeric token に含めないため checker が token 外へ後読みせず、別 item として後続 checker が扱う境界を source policy で固定した。subagent review `019e9b11-8ecf-77b1-82a4-f1150d214189` は suffix/defaulting を先行実装せず token-local numeric payload authority へ絞る判断を承認した。
- 2026-06-06: 負数 numeric literal の source-backed consume-width を追加した。`syntax/ast/prefix_expr.nepl` は `Minus` を `MinusMarker` として 1 token item のまま保持し、source-backed argument checker が `MinusMarker + IntLiteral/FloatLiteral` だけを expected type と照合して 2 item を 1 checked argument として消費する。operand 欠落、非 numeric operand、span 結合失敗、expected type mismatch は `NegativeLiteral*` / `ArgumentNegativeLiteral*` error として分ける。literal payload は joined span を診断範囲として使い、semantic lexeme は operand token spelling と `-` の連結で作るため、`- 1` の token 間 whitespace を数値 parser へ渡さない。HIR lowering は既存の literal checked payload を消費するため source token を再読しない。
- 2026-06-06: `check/expr/checked_tree_id.nepl` と `check/expr/checked_tree.nepl` を追加し、複合 expression の checked payload を owner 付き tree table へ保存するモデルを固定した。`SelfhostCheckedExprId` と `SelfhostCheckedArgumentRange` は sentinel を使わず typed id / typed range とし、範囲構築失敗は `SelfhostCheckedTreeRangeBuildError` で分類する。unchecked constructor は module private helper に落とし、外部入力や serialized boundary は `*_new_result` と bounded range constructor を通す。`SelfhostCheckedArgumentKind::CheckedExpr` は tree node id だけを持つ Copy payload とし、owner 付き Vec を argument enum へ入れない。tree builder は `DirectCall` argument range、`CheckedExpr` child id、`BlockResult` body id が追加前の既存 node だけを参照することを append 前に検査し、future id / self-reference / broken range は `StdErrorKind::InvalidOperation` として fail-closed にする。`lower/hir/direct_call.nepl` には `selfhost_hir_lower_checked_tree_expr` を追加し、checked tree の `DirectCall` node を selected candidate + checked argument range から HIR `Call` へ、`BlockResult` node を body expression id から HIR `Block` へ下ろす。lowering は source token、scope lookup、callable candidate collection を再実行せず、壊れた node / argument range は `CheckedTreeNodeMissing` / `CheckedTreeArgumentMissing`、fuel exhaustion は `CheckedTreeCycleDetected` として fail-closed にする。Zenn 方針に照らした subagent review `019e9b11-8ecf-77b1-82a4-f1150d214189` と `019e9b23-11f6-77f1-bf82-7aa52185f6e3` は、前回の DAG invariant blocker が閉じていることを確認し、この slice を approve した。
- 2026-06-06: source-backed call reducer が `SelfhostCheckedExprTree` を生成し、top-level direct call、nested direct call argument、trailing block result を同じ checked tree 内の node と root id へ接続するようにした。`SelfhostCallReduceOwnedResult` と `SelfhostExpressionLineCheckSuccess` は `checked_tree` と `root_expr` を owner payload として保持し、HIR lowering は `selfhost_hir_lower_expression_line_success_direct_call` から `selfhost_hir_lower_checked_tree_expr` へ進む。nested call と block result の外側 argument payload は `SelfhostCheckedArgumentKind::CheckedExpr` になり、source token、scope lookup、callable candidate collection を HIR lowering で再実行しない。checked tree build failure は subagent review の指摘に従い `InternalInvariant` へ潰さず、`CheckedTreeBuildInvalidOperation` として call reduction error enum に残す。
- 2026-06-06: 複数式 trailing block body を `BlockSequence` checked tree node へ接続した。`SelfhostCheckedExprTree` は block child root id 専用の `exprs` owner table を持ち、`BlockSequence` は `SelfhostCheckedExprRange` で source order の body root ids を参照する。node table の contiguous range は nested call や inner block node の挿入で body expression list と一致しないため、block sequence の authority には使わない。call reducer は複数 segment を同じ checked tree に順次追加し、最後の expression だけを外側 expected type の `BlockResult` expectation で検査する。非末尾 expression は `SelfhostTypeKind::Unit` を `selfhost_type_arena_find_kind` で検索し、`BlockSequenceDiscardedExpression` expectation として渡す。arena に unit type が無い場合は `TrailingBlockBodyUnitTypeMissing` にし、`SelfhostTypeId 0` のような fixture 前提へ戻さない。HIR lowering は `BlockSequence` range を fuel 付きで順に下ろし、HIR `Block` child range を作る。source token、scope lookup、callable candidate collection は lowering で再実行しない。
- 2026-06-06: trailing block body 内の nested `BlockIntro` を source-backed reducer へ接続した。`call_reduce.nepl` に segment dispatcher を追加し、`ExpressionLine` は従来の block body input builder、`BlockIntro` は `segment.head` だけの prefix list と `segment.body` の `SelfhostTrailingBlockArgument` に分けて、同じ `SelfhostCheckedExprTree` owner の中で再帰検査する。legacy single-input API は nested `BlockIntro` を引き続き `NestedBlockUnsupported` として拒否するが、通常成功経路は dispatcher を通る。stage1 には `TokenKind::Colon` を含む `add 1 add 1: add 1 1` fixture を追加し、`BlockIntro.body` を flat prefix list に直接渡さない契約を source policy で固定した。
- 2026-06-06: source-backed reducer に単一 pipe expression の direct-call 正規化を追加した。`1 |> add 2` は prefix list を1回走査して単一 `PipeOperator` を確認し、右辺 `add` を scope / callable signature table から DefId-linked target として収集し、左辺 `1` を target の第1 parameter expected type で検査して checked argument 0 番目へ保存する。右辺 suffix の `2` は既存 argument loop に第2 parameter から渡し、checked argument 1 番目として保存する。複数 pipe chain は `PipeUnsupportedMultiple`、`NamedValue` 以外の右辺 target は `PipeRightTargetUnsupported`、0引数 target は `PipeTargetRequiresInput` として fail-closed にし、source-less reducer は引き続き named-head-only にした。subagent review の指摘に従い、candidate collection error から pipe error への写像は `_` で潰さず全 enum variant を明示分岐にした。
- 2026-06-06: pipe の fail-closed 境界を stage1 の実行可能 smoke に追加した。`1 |> add 2 |> add 3` は `PipeUnsupportedMultiple`、`|> add 1` は `PipeMissingLeftOperand`、`1 |>` は `PipeMissingRightTarget`、`1 |> 2` と `1 |> %i32 add 2` は `PipeRightTargetUnsupported`、`1 2 |> add 3` は `PipeLeftSegmentNotSingleValue`、`1 |> answer` かつ `answer : fn void i32` は `PipeTargetRequiresInput` として、source-backed body-line reducer を実際に通した typed error を確認する。これは chain / ascribed target / lambda target を受理する実装ではなく、未対応形を fallback success へ流さない契約を固定する checkpoint である。
- 2026-06-11: source-backed reducer に ascribed pipe target を追加した。`1 |> %fn i32 fn i32 i32 add 2` は RHS の `%fn ... add 2` を source token range として `selfhost_expr_ascription_project_head_expectation` へ渡し、projection が返す callable type expectation と expression-first token から target prefix item を復元する。target は現 slice では `NamedValue` だけを受理し、候補が 1 件だけなら candidate callable type と ascribed function type を同じ arena 内で照合する。一致する場合は suffix cursor を `target_index + 1` にして `add 1 2` 相当の `DirectCall` checked tree へ進むため、`%fn ...` の型式 token を実引数として誤消費しない。projection 失敗は `PipeTargetAscriptionProjectionFailed`、型不一致は `PipeTargetAscriptionTypeMismatch` として typed enum error に分ける。
- 2026-06-11: subagent review `019eb4b9-e966-7240-8661-a0922b7bc942` は blocker なしで approve。non-blocker として、現 slice では candidate count が 1 件になった後で target ascription を照合するため、`%fn ... overloaded_name` による overload narrowing はまだ行わないことが確認された。この narrowing は generic inference / overload solver と同じ後続の advanced pipe target candidate narrowing として残す。
- 2026-06-11: ascribed pipe target の overload narrowing を追加した。`selfhost_call_reduce_pipe_candidates_with_source` は `target_ascription = Some` の場合、候補数だけで先に `PipeTargetAmbiguous` を返さず、projected function type と candidate callable type の構造一致で候補を絞る。一致 0 件は `PipeTargetAscriptionTypeMismatch`、一致 1 件はその candidate で direct call 正規化へ進み、同じ callable type に一致する候補が複数残る場合だけ `PipeTargetAmbiguous` にする。`target_ascription = None` の複数候補は従来通り曖昧なままにし、引数式を先読みする探索拡大はこの slice では行わない。stage1 smoke は `add : fn i32 i32` と `add : fn i32 fn i32 i32` の同名候補を置き、`1 |> %fn i32 fn i32 i32 add 2` が2引数版だけを選び、checked argument order を保つことを実行可能に確認する。さらに、注釈型に一致する overload が 0 件の case を `PipeTargetAscriptionTypeMismatch`、同じ callable type に複数一致する case を `PipeTargetAmbiguous` として source-backed reducer 経由で拒否する negative smoke も public pipe fail-closed 群へ接続した。
- 2026-06-11: pipe chain checked-tree checkpoint では、`left |> named_target suffix... |> named_target suffix...` を source-backed reducer の左結合 success path へ接続した。1段目は既存の pipe direct-call reducer で checked `DirectCall` node を作り、2段目以降は前段 root id を `SelfhostCheckedArgumentKind::CheckedExpr` として次段 target の第1引数へ渡す。各段の suffix は次の pipe operator より手前までに切るため、`1 |> add 2 |> add 3` の最初の `add` は `2` だけを消費し、2個目の pipe を raw trailing item として誤消費しない。stage1 smoke は最終段の checked argument 0 番目が `CheckedExpr [0, 4)`、1 番目が `I32Literal(3) [6, 7)` であることに加え、root `DirectCall` から前段 `DirectCall` node まで checked tree topology を辿って確認する。`1 |> add 2 |>` は前段成功後に `PipeMissingRightTarget`、`1 |> add 2 |> 3` は `PipeRightTargetUnsupported` として後段 fail-closed になることも public smoke へ追加した。source-less reducer は引き続き pipe を成功扱いしない。
- 2026-06-11: 注釈無し pipe target の単純な引数列 narrowing を追加した。`target_ascription = None` で候補が複数ある場合、source-backed owner reducer を候補ごとに投機実行せず、pipe 左辺と RHS suffix を borrowed probe で `Match` / `NoMatch` / `ProbeUnsupported` に分類して集計する。unsupported が混じる場合は `PipeTargetAmbiguous` として安全側に止め、一致 0 件は `PipeTargetNoApplicableCandidate`、一致複数は `PipeTargetAmbiguous`、一意一致だけを既存の checked-left finisher へ渡す。stage1 smoke は `1 |> add 2` で `fn i32 i32` と `fn i32 fn i32 i32` の同名候補から2引数版だけを選ぶ success case、適用候補 0 件 case、同じ callable type に複数一致する case を public pipe body-line から実行する。`UnsupportedArgumentExpression`、argument ascription projection failure、NamedValue evidence missing など source-backed 検査が必要な probe は candidate-specific mismatch にせず `ProbeUnsupported` へ分類する契約も source policy で固定した。
- 2026-06-11: 注釈無し pipe target の `ProbeUnsupported` runtime smoke を追加した。`1 |> add %i32 2` は、`add : fn i32 i32` では suffix 余剰により `NoMatch`、`add : fn i32 fn i32 i32` では `%i32 2` が source-backed argument checker を必要とするため `ProbeUnsupported` になる。この候補列を `PipeTargetAmbiguous` として拒否することを stage1 public smoke で確認し、borrowed probe が高度な suffix argument を不一致扱いして別候補を選ぶ退行を防いだ。これは advanced suffix expression の成功実装ではなく、source-backed checker を候補ごとに投機実行しない fail-closed 境界である。
- 2026-06-11: 注釈無し pipe target の唯一 source-backed 必須候補を source-backed finisher へ接続した。subagent review で、従来の `ProbeUnsupported` は `%i32 2` のような再検査可能な suffix と、generic 未解決・内部境界・OOM のような選んではいけない未対応を同じ variant に潰していると指摘されたため、`SelfhostPipeCandidateApplicability` を `SourceBackedRequired` と `SelectionBlockedUnsupported` に分離した。`Match` が 0 件、`SourceBackedRequired` が 1 件、`SelectionBlockedUnsupported` が 0 件の場合だけ、候補ごとの投機実行を広げず、その1候補だけを既存の checked-left / source-backed suffix checker へ渡す。これにより `1 |> add %i32 2` は `add : fn i32 i32` が suffix 余剰で `NoMatch`、`add : fn i32 fn i32 i32` だけが `%i32 2` の source-backed 検査を必要とする状況で2引数版 direct call として成功する。`Match` と `SourceBackedRequired` の混在、`SourceBackedRequired` 複数件、または `SelectionBlockedUnsupported` が1件以上ある場合は引き続き `PipeTargetAmbiguous` として fail-closed にする。stage1 smoke は checked argument order と `%i32 2` の内側 literal payload を確認し、HIR lowering が source token を再読しない契約を保つ。
- 2026-06-11: 注釈無し pipe target の唯一 source-backed named suffix を source-backed finisher へ接続した。`1 |> add x` は、borrowed probe では `x` の value evidence table を参照できないため2引数版 candidate が `SourceBackedRequired` になり、1引数版 candidate は suffix 余剰で `NoMatch` になる。`Match` 0 件、`SourceBackedRequired` 1 件、`SelectionBlockedUnsupported` 0 件という既存条件を満たす場合だけ、その1候補を source-backed finisher に渡し、`SelfhostValueTypeEvidenceTable` の DefId-linked `x: i32` evidence から `SelfhostCheckedArgumentKind::NamedValue` payload を作る。stage1 smoke は checked argument order が左辺 `I32Literal(1) [0, 1)`、suffix `NamedValue [3, 4)` であることを確認し、HIR lowering が source token / scope lookup / value evidence lookup を再実行しない契約を固定した。
- 2026-06-11: 注釈無し pipe target の唯一 source-backed nested suffix を source-backed finisher へ接続した。`1 |> add sum 2 3` は、borrowed probe では suffix の `sum 2 3` を checked tree node へ変換できないため2引数版 `add` candidate が `SourceBackedRequired` になり、1引数版 `add` candidate は suffix 余剰で `NoMatch` になる。`Match` 0 件、`SourceBackedRequired` 1 件、`SelectionBlockedUnsupported` 0 件という既存条件を満たす場合だけ、その1候補を source-backed finisher に渡し、`sum 2 3` を `SelfhostCheckedArgumentKind::CheckedExpr [3, 6)` として outer call の第2引数へ保存する。stage1 smoke は outer `add` の argument order だけでなく、checked tree 内の child `DirectCall` が `I32Literal(2) [4, 5)` と `I32Literal(3) [5, 6)` を持つことまで確認し、HIR lowering が source token / scope lookup / callable candidate collection を再実行しない契約を固定した。この checkpoint では inner callable を単一候補の `sum` に限定し、同名 overloaded inner call narrowing は次 checkpoint の対応対象として切り出した。
- 2026-06-11: source-backed nested argument の同名 final-range narrowing を追加した。`1 |> add add 2 3` は pipe suffix 由来の代表 smoke であり、outer target の `add` では既存の `Match` 0 件、`SourceBackedRequired` 1 件、`SelectionBlockedUnsupported` 0 件という条件だけで2引数版 outer candidate を finisher へ渡す。finisher 内で inner head `add` に複数候補が見つかった場合は、candidate ごとの owner / source-backed reducer 投機実行を行わず、borrowed matcher で `item_index..item_count` を完全消費し、かつ outer parameter の expected type と result type が一致する候補数だけを数える。0 件または複数件は `OverloadAmbiguous`、1 件だけならその候補で source-backed nested call finisher を 1 回実行する。stage1 smoke は outer argument order と child `DirectCall` の candidate name `add`、inner `I32Literal(2) [4, 5)` / `I32Literal(3) [5, 6)` を確認し、HIR lowering が source token / scope lookup / callable candidate collection を再実行しない契約を固定した。
- 残件: lambda / borrow / source-backed pipe suffix の block / lambda / inner call が source-backed argument 範囲全体を消費しない一般 nested overload boundary など複合式 success path、generic instantiation inference、trait solving、numeric suffix の言語仕様化、binary / octal radix の扱い、defaulting beyond current Rust fixed `i32` / `f32`、indirect call、`memo_call` Phase 1 境界、cross-arena serialized canonical key / fingerprint、nested generic binder depth と stable binder identity は未実装のため、この issue は open のまま維持する。

## 検証

Add normal tests for prefix argument extent, %TypeExpr extent, nested block arguments, malformed prefix calls, and checker diagnostics once cfg-test-style tests are available.

2026-06-05 checkpoint:

- `node nodesrc/test_selfhost_parser_current_syntax_boundary.js`
- `node nodesrc/test_selfhost_module_parser_split_contract.js`
- `node nodesrc/test_selfhost_module_checker_split_contract.js`
- `node nodesrc/test_selfhost_proof_entry_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_parser.n.md -o tmp\selfhost-prefix-parser-tests.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_proof.n.md -o tmp\selfhost-prefix-proof-tests.json --no-tree -j 1 --assert-io --dist web\dist`

2026-06-06 source-backed pipe direct-call checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-pipe.json`: total=7, passed=7
- subagent review: first review flagged wildcard handling in pipe candidate-collection error mapping as a Zenn policy blocker; implementation was updated to explicit enum matching before verification.

2026-06-06 pipe fail-closed smoke checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-pipe-failclosed.json`: total=7, passed=7
- `node nodesrc/tests.js -i stdlib\neplg2\core\check -o tmp\selfhost-prefix-check-tests.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2 -o tmp\selfhost-prefix-boundary-stdlib-neplg2.json --no-tree -j 2 --assert-io --dist web\dist`
- `node nodesrc/run_source_policy_regressions.js --warn-only`（既存 5 warning のみ）
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`

2026-06-11 ascribed pipe target checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`: pass
- `node nodesrc/test_selfhost_hir_lowering_contract.js`: pass
- `node nodesrc/test_selfhost_documentation_contract.js`: pass
- `node nodesrc/test_selfhost_zenn_review_gate_contract.js`: pass
- `node nodesrc/test_source_policy_no_line_count_limits.js`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-ascribed-pipe-target.json`: total=7, passed=7
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
- subagent review `019eb4b9-e966-7240-8661-a0922b7bc942`: approve

2026-06-11 ascribed pipe target overload narrowing checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-pipe-target-overload-narrowing-initial.json`: total=7, passed=7
- subagent review `019eb4c8-30f7-7ca0-b591-772bdbdc316f`: blocker なし。`target_ascription = Some` の場合だけ callable type equality で候補を絞り、一致 0 / 1 / 複数を typed error / success / ambiguity に分ける方針を妥当と確認。
- subagent review `019eb4d0-46c1-7281-8e33-6f7c9d8b3a62`: blocker なし。non-blocker として、unused helper の削除と 0 件一致 / 重複一致の runtime negative smoke 追加を推奨。unused helper は削除し、negative smoke は `selfhost_check_expr_stage1_pipe_failclosed_body_line` へ接続した。

2026-06-11 pipe chain checked-tree checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`: pass
- `node nodesrc/test_selfhost_hir_lowering_contract.js`: pass
- `node nodesrc/test_selfhost_documentation_contract.js`: pass
- `node nodesrc/test_source_policy_no_line_count_limits.js`: pass
- `node nodesrc/test_selfhost_zenn_review_gate_contract.js`: pass
- `node nodesrc/selfhost_zenn_review_response_check.js --stdin --review-kind final --record note.n.md`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-pipe-chain.json`: total=7, passed=7
- `node nodesrc/run_source_policy_regressions.js --warn-only`: pass
- `node nodesrc/issues.js index --dir issues`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
- subagent review `019eb4f9-b13e-7202-bbb2-a5002e748277`: first post-implementation review found bare `Some` / `None` in topology helpers. Fixed to `Option::Some` / `Option::None`, added JS contract coverage for the qualified variants, and re-ran focused contract / stdlib smoke successfully. Post-fix review found no additional blocker and confirmed no owner cleanup leak or double-free concern in the added runner/helper paths.

2026-06-11 non-ascribed pipe target argument narrowing checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`: pass
- `node nodesrc/test_selfhost_hir_lowering_contract.js`: pass
- `node nodesrc/test_selfhost_documentation_contract.js`: pass
- `node nodesrc/test_source_policy_no_line_count_limits.js`: pass
- `node nodesrc/test_selfhost_zenn_review_gate_contract.js`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-pipe-target-arg-probe-final2.json`: total=7, passed=7
- `node nodesrc/run_source_policy_regressions.js --warn-only`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
- subagent review `019eb4b9-e966-7240-8661-a0922b7bc942`: blocker なし。non-blocker の summary doc comment 不一致は修正済み。runtime `ProbeUnsupported` smoke は source-backed advanced argument expression の次 slice 候補として残す。
- subagent review `019eb4c8-30f7-7ca0-b591-772bdbdc316f`: blocker なし。non-blocker の unsupported mapping contract 不足に従い、`UnsupportedArgumentExpression` / argument ascription projection failure / named value evidence missing が `ProbeUnsupported` へ行くことを JS contract で固定した。
- subagent review `019eb4c8-30f7-7ca0-b591-772bdbdc316f`: blocker あり。`ProbeUnsupported` が retryable source-backed suffix と generic/internal/OOM などの selection-blocked 状態を同一 variant にしていたため、`SourceBackedRequired` と `SelectionBlockedUnsupported` に分離して解消した。
- subagent re-review `019eb4c8-30f7-7ca0-b591-772bdbdc316f`: blocker なし。variant 分離、summary field 分離、selection-blocked の即時 `PipeTargetAmbiguous`、唯一 source-backed required candidate だけを既存 finisher へ渡す境界を確認済み。

2026-06-11 non-ascribed pipe target ProbeUnsupported smoke checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-pipe-probe-unsupported.json`: total=7, passed=7

2026-06-11 source-backed pipe named suffix checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`: pass
- `node nodesrc/test_selfhost_hir_lowering_contract.js`: pass
- `node nodesrc/test_selfhost_documentation_contract.js`: pass
- `node nodesrc/test_source_policy_no_line_count_limits.js`: pass
- `node nodesrc/test_selfhost_zenn_review_gate_contract.js`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-pipe-source-backed-named-suffix.json`: total=7, passed=7
- `node nodesrc/run_source_policy_regressions.js --warn-only`: pass
- `node nodesrc/issues.js index --dir issues`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
- subagent review `019eb594-fd38-7442-91ec-39bbfc618066`: blocker なし。`call_reduce.nepl` の finisher 条件を広げず、`1 |> add x` の `NamedValue` payload が source token 再読ではなく scope / value evidence 由来であることを確認済み。

2026-06-11 source-backed pipe nested suffix checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`: pass
- `node nodesrc/test_selfhost_hir_lowering_contract.js`: pass
- `node nodesrc/test_selfhost_documentation_contract.js`: pass
- `node nodesrc/test_source_policy_no_line_count_limits.js`: pass
- `node nodesrc/test_selfhost_zenn_review_gate_contract.js`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-pipe-source-backed-nested-suffix-final.json`: total=7, passed=7
- subagent design review `019eb594-fd38-7442-91ec-39bbfc618066`: 同名 `add` overload のまま `1 |> add add 2 3` を success にすると nested call overload narrowing まで含むため、この時点の slice では outer target を overloaded `add`、inner callable を単一候補 `sum` に分ける方針を推奨。これに従い、`1 |> add sum 2 3` で source-backed nested suffix の `CheckedExpr` topology を固定し、同名 overloaded inner call narrowing は後続 checkpoint で別途扱うことにした。
- subagent implementation review `019eb594-fd38-7442-91ec-39bbfc618066`: blocker なし。non-blocker として child call の callee が `sum` であることも確認するとより強いと指摘されたため、`selfhost_check_expr_stage1_pipe_suffix_nested_call_child_arguments_ok` で checked tree に保存済みの `call.candidate.name` を `string_search::str_eq` で確認するよう補強し、focused smoke を再実行した。

2026-06-05 NamedValue HIR identity checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/test_selfhost_hir_expr_payload.js`
- `node nodesrc/test_selfhost_hir_lowering_contract.js`
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-expr-named-value-payload.json`
- `node nodesrc/tests.js -i stdlib/neplg2/core/hir --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-hir-value-identity.json`
- `node nodesrc/tests.js -i stdlib/neplg2/core/lower --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-hir-named-value-lowering.json`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`

2026-06-05 literal value payload checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/test_selfhost_hir_lowering_contract.js`
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-expr-literal-payload.json`
- `node nodesrc/tests.js -i stdlib/neplg2/core/lower --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-hir-literal-lowering.json`

2026-06-06 string literal escape decode checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/test_selfhost_hir_lowering_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_call_reduce.n.md --no-tree --no-stdlib -j 1 --assert-io -o tmp/neplg2_call_reduce_string_escape_tests.json`
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-expr-string-escape.json`
- `node nodesrc/tests.js -i stdlib/neplg2/core/lower --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-hir-string-escape.json`

2026-06-06 numeric literal radix payload checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_call_reduce.n.md --no-tree --no-stdlib -j 1 --assert-io -o tmp/neplg2_call_reduce_numeric_literal_tests.json`

2026-06-06 negative numeric literal consume-width checkpoint:

- `node nodesrc/test_selfhost_expr_prefix_contract.js`
- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/test_selfhost_parser_tokenkind_match.js`
- `node nodesrc/test_selfhost_token_split_contract.js`
- `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- `node nodesrc/tests.js -i stdlib/neplg2/core/syntax/ast/prefix_expr.nepl --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-negative-prefix-expr.json`
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-expr-negative-literal.json`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_negative_literal.n.md --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-negative-literal-tests.json`

2026-06-06 checked tree HIR lowering checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/test_selfhost_hir_lowering_contract.js`
- `node nodesrc/test_selfhost_documentation_contract.js`
- `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- `node nodesrc/test_source_policy_no_line_count_limits.js`
- `node nodesrc/tests.js -i stdlib/neplg2/core/check/expr/checked_tree_id.nepl --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-checked-tree-id.json`
- `node nodesrc/tests.js -i stdlib/neplg2/core/check/expr/checked_tree.nepl --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-checked-tree.json`
- `node nodesrc/tests.js -i stdlib/neplg2/core/lower/hir/direct_call.nepl --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-hir-direct-call-tree.json`
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-expr-checked-tree.json`
- `node nodesrc/tests.js -i stdlib/neplg2/core/lower --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-hir-tree-lowering.json`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `node nodesrc/issues.js index --dir issues`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`

2026-06-06 checked tree reducer connection checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/test_selfhost_hir_lowering_contract.js`
- `node nodesrc/test_selfhost_documentation_contract.js`
- `node nodesrc/test_selfhost_zenn_review_gate_contract.js`
- `node nodesrc/tests.js -i stdlib/neplg2/core/lower/hir/direct_call.nepl --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-hir-direct-call-tree-reducer.json`
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-tree-reducer.json`
- `node nodesrc/tests.js -i stdlib/neplg2/core/lower --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-lower-tree-reducer.json`

2026-06-06 block body sequence checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/test_selfhost_hir_lowering_contract.js`
- `node nodesrc/test_selfhost_documentation_contract.js`
- `node nodesrc/test_source_policy_no_line_count_limits.js`
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-core-sequence.json`
- `node nodesrc/tests.js -i stdlib/neplg2/core/ty --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-ty-kind-find.json`
- `node nodesrc/tests.js -i stdlib/neplg2/core/lower/hir/direct_call.nepl --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-hir-direct-call-sequence.json`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`

2026-06-05 type resolver input checkpoint:

- `node nodesrc/test_selfhost_type_resolver_split_contract.js`
- `node nodesrc/test_selfhost_type_resolver_prefix_input.js`
- `node nodesrc/test_selfhost_ty_split_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_resolver.n.md -o tmp\selfhost-type-resolver-tests2.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2\core\resolve\type_resolver.nepl -o tmp\selfhost-type-resolver-facade-tests3.json --no-tree -j 1 --assert-io --dist web\dist`

2026-06-05 type resolver reduction checkpoint:

- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_resolver.n.md -o tmp\selfhost-type-resolver-reduce-tests-split.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/test_selfhost_type_resolver_split_contract.js`
- `node nodesrc/test_selfhost_type_resolver_prefix_input.js`
- `node nodesrc/tests.js -i stdlib\neplg2\core\resolve\type_resolver.nepl -o tmp\selfhost-type-resolver-facade-reduce-split.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2 -o tmp\selfhost-type-resolver-reduction-stdlib-neplg2-split.json --no-tree -j 2 --assert-io --dist web\dist`

2026-06-05 type resolver TypeId projection checkpoint:

- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_resolver.n.md -o tmp\selfhost-type-resolver-project-tests2.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/test_selfhost_type_resolver_split_contract.js`
- `node nodesrc/test_selfhost_type_resolver_prefix_input.js`
- `node nodesrc/tests.js -i stdlib\neplg2\core\resolve\type_resolver.nepl -o tmp\selfhost-type-resolver-project-facade2.json --no-tree -j 1 --assert-io --dist web\dist`

2026-06-05 nested named call argument checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_call_reduce.n.md --no-tree --no-stdlib -j 1 --assert-io -o tmp/neplg2_call_reduce_nested_argument_tests.json`
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-expr-nested-call-review.json`
- `node nodesrc/tests.js -i stdlib\neplg2 -o tmp\selfhost-type-resolver-project-stdlib-neplg2.json --no-tree -j 2 --assert-io --dist web\dist`

2026-06-05 trailing block argument checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_call_reduce.n.md --no-tree -o tmp/selfhost-call-reduce-block-argument.json -j 1`
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -o tmp/selfhost-check-block-argument.json -j 1`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
- `node nodesrc/run_source_policy_regressions.js --warn-only`（exit 0、既存の resource gate / diagnostic registry warning 2件あり）

2026-06-05 block body result checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/test_selfhost_body_segmenter_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_call_reduce.n.md --no-tree --no-stdlib -j 1 --assert-io -o tmp/neplg2_call_reduce_block_result_tests.json`
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-expr-block-result.json`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_text.n.md --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-text-effect.json`
- `node nodesrc/tests.js -i stdlib/neplg2/cli/file_io.nepl --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-file-io-effect.json`
- `node nodesrc/tests.js -i stdlib/std/fs/path.nepl --no-tree -j 1 --assert-io --dist web/dist -o tmp/fs-path-facade-effect.json`
- `node nodesrc/tests.js -i stdlib/neplg2 --no-tree -j 2 --assert-io --dist web/dist -o tmp/selfhost-stdlib-neplg2-block-result.json`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`

2026-06-05 type constructor lookup checkpoint:

- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_arena.n.md -o tmp\selfhost-type-arena-named-tests2.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_resolver.n.md -o tmp\selfhost-type-resolver-constructor-tests.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/test_selfhost_ty_split_contract.js`
- `node nodesrc/test_selfhost_type_resolver_split_contract.js`

2026-06-05 generic type application checkpoint:

- `node nodesrc/test_selfhost_type_record_payload.js`
- `node nodesrc/test_selfhost_type_arena_report_contract.js`
- `node nodesrc/test_selfhost_type_resolver_split_contract.js`
- `node nodesrc/test_selfhost_type_resolver_prefix_input.js`
- `node nodesrc/test_selfhost_type_resolver_generic_application_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_arena.n.md -o tmp\selfhost-type-arena-applied-tests.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_resolver.n.md -o tmp\selfhost-type-resolver-applied-shard-1-2.json --no-tree --shard 1/2 -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_resolver.n.md -o tmp\selfhost-type-resolver-applied-shard-2-2.json --no-tree --shard 2/2 -j 1 --assert-io --dist web\dist`

2026-06-05 type constructor kind validation checkpoint:

- `node nodesrc/test_selfhost_type_resolver_generic_application_contract.js`
- `node nodesrc/test_selfhost_type_resolver_type_parameters.js`
- `node nodesrc/test_selfhost_type_resolver_split_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_constructor_validation.n.md -o tmp\selfhost-type-constructor-validation-projection-final2.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_constructor_type_parameters.n.md -o tmp\selfhost-type-constructor-type-parameters-final6.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_constructor_projection.n.md -o tmp\selfhost-type-constructor-projection-final.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2\core\resolve\type_resolver.nepl -o tmp\selfhost-type-resolver-facade-kind-bound.json --no-tree -j 1 --assert-io --dist web\dist`

2026-06-05 prefix expression input checkpoint:

- `node nodesrc/test_selfhost_expr_prefix_contract.js`
- `node nodesrc/test_selfhost_parser_current_syntax_boundary.js`
- `node nodesrc/test_selfhost_type_resolver_prefix_input.js`
- `node nodesrc/test_selfhost_module_parser_split_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_expr_prefix.n.md -o tmp\selfhost-expr-prefix-tests.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2\core\syntax\ast\prefix_expr.nepl -o tmp\selfhost-expr-prefix-module-doctest.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2\core\syntax -o tmp\selfhost-syntax-focused.json --no-tree -j 2 --assert-io --dist web\dist`

2026-06-05 function body range checkpoint:

- `node nodesrc/test_selfhost_function_body_prefix_range_contract.js`
- `node nodesrc/test_selfhost_module_parser_split_contract.js`
- `node nodesrc/test_selfhost_parser_current_syntax_boundary.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_expr_prefix.n.md -o tmp\selfhost-expr-prefix-body-range.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_parser.n.md -o tmp\selfhost-parser-body-range.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_checker.n.md -o tmp\selfhost-checker-body-range.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2\core\syntax\parser\module_parser.nepl -o tmp\selfhost-module-parser-facade-body-range.json --no-tree -j 1 --assert-io --dist web\dist`

2026-06-05 body segmenter checkpoint:

- `node nodesrc/test_selfhost_body_segmenter_contract.js`
- `node nodesrc/test_selfhost_module_parser_split_contract.js`
- `node nodesrc/test_selfhost_function_body_prefix_range_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_body_segmenter.n.md -o tmp\selfhost-body-segmenter.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_expr_prefix.n.md -o tmp\selfhost-expr-prefix-after-segmenter.json --no-tree -j 1 --assert-io --dist web\dist`

2026-06-05 call reduction input checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/test_selfhost_module_checker_split_contract.js`
- `node nodesrc/test_selfhost_expr_prefix_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_call_reduce.n.md -o tmp\selfhost-call-reduce.json --no-tree -j 1 --assert-io --dist web\dist`

2026-06-05 literal argument type evidence checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_call_reduce.n.md --no-tree --no-stdlib -j 1 --assert-io -o tmp/neplg2_call_reduce_argument_type_tests.json`

2026-06-05 expression line call reduction connector checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/test_selfhost_body_segmenter_contract.js`
- `node nodesrc/test_selfhost_expr_prefix_contract.js`
- `node nodesrc/test_selfhost_module_checker_split_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_call_reduce.n.md -o tmp\selfhost-call-reduce-body-line-final.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2\core\check\expr.nepl -o tmp\selfhost-check-expr-facade-body-line-final.json --no-tree -j 1 --assert-io --dist web\dist`

2026-06-05 expression ascription expectation checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/test_selfhost_type_resolver_prefix_input.js`
- `node nodesrc/test_selfhost_type_resolver_split_contract.js`
- `node nodesrc/test_selfhost_type_resolver_generic_application_contract.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`（今回追加した selfhost / type resolver policy は pass。既存の `test_resource_gate_order.js` と `test_diagnostic_code_first_boundary.js` は warning）
- `node nodesrc/test_selfhost_string_helpers_boundary.js`
- `node nodesrc/test_selfhost_lexer_raw_mode_directive_enum.js`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_call_reduce.n.md --no-tree --no-stdlib -j 1 --assert-io -o tmp/neplg2_call_reduce_ascription_tests.json`（2/2 pass）
- `git diff --check`

2026-06-05 ascription outer expected conflict checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_call_reduce.n.md --no-tree --no-stdlib -j 1 --assert-io -o tmp/neplg2_call_reduce_ascription_conflict_tests.json`（2/2 pass）
- `node nodesrc/issues.js check --dir issues`
- `node nodesrc/run_source_policy_regressions.js --warn-only`（今回追加した selfhost policy は pass。既存の `test_resource_gate_order.js` と `test_diagnostic_code_first_boundary.js` は warning）
- `git diff --check`

2026-06-05 argument expression cursor checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_call_reduce.n.md --no-tree --no-stdlib -j 1 --assert-io -o tmp/neplg2_call_reduce_argument_cursor_tests.json`

2026-06-05 source-backed ascribed argument checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_call_reduce.n.md --no-tree --no-stdlib -j 1 --assert-io -o tmp/neplg2_call_reduce_ascribed_argument_source_tests.json`（2/2 pass）

2026-06-05 named value argument evidence checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_call_reduce.n.md --no-tree --no-stdlib -j 1 --assert-io -o tmp/neplg2_call_reduce_named_argument_tests.json`（2/2 pass）
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-expr-named-value-review.json`（3/3 pass）
- `node nodesrc/issues.js check --dir issues`
- `node nodesrc/run_source_policy_regressions.js --warn-only`（今回対象の selfhost policy は pass。既存の `test_resource_gate_order.js` と `test_diagnostic_code_first_boundary.js` は warning）
- `git diff --check`

2026-06-05 canonical type key checkpoint:

- `node nodesrc/test_selfhost_ty_split_contract.js`
- `node nodesrc/test_selfhost_type_record_payload.js`
- `node nodesrc/test_selfhost_type_arena_report_contract.js`
- `node nodesrc/test_selfhost_type_key_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_key.n.md -o tmp\selfhost-type-key-tests.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2\core\ty\ty.nepl -o tmp\selfhost-ty-key-smoke.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2 -o tmp\selfhost-canonical-type-key-stdlib-neplg2.json --no-tree -j 2 --assert-io --dist web\dist`
- `node nodesrc/run_source_policy_regressions.js --warn-only`（既存 5 warning のみ）
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`

2026-06-05 generic type parameter environment checkpoint:

- `node nodesrc/test_selfhost_type_resolver_split_contract.js`
- `node nodesrc/test_selfhost_type_resolver_prefix_input.js`
- `node nodesrc/test_selfhost_type_resolver_generic_application_contract.js`
- `node nodesrc/test_selfhost_type_resolver_type_parameters.js`
- `node nodesrc/test_neplg21_vec_type_arity_imports.js`
- `node nodesrc/test_nepl_doc_report_metadata_policy.js`
- `node nodesrc/tests.js -i stdlib\neplg2\core\resolve\type_resolver\typeparam.nepl -o tmp\selfhost-typeparam-doctest.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i stdlib\neplg2\core\resolve\type_resolver.nepl -o tmp\selfhost-type-resolver-facade-smoke.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_resolver_type_parameters.n.md -o tmp\selfhost-type-resolver-type-parameters.json --no-tree -j 1 --assert-io --dist web\dist`

2026-06-05 binder-indexed type parameter arena/key checkpoint:

- `node nodesrc/test_selfhost_type_record_payload.js`
- `node nodesrc/test_selfhost_type_key_contract.js`
- `node nodesrc/test_selfhost_type_resolver_type_parameters.js`
- `node nodesrc/test_selfhost_type_arena_report_contract.js`
- `node nodesrc/test_selfhost_model_no_numeric_kind_tags.js`
- `node nodesrc/test_selfhost_ty_split_contract.js`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_arena.n.md -o tmp\selfhost-type-arena-parameter.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_key.n.md -o tmp\selfhost-type-key-parameter.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_resolver_type_parameters.n.md -o tmp\selfhost-type-resolver-type-parameters-projection.json --no-tree -j 1 --assert-io --dist web\dist`
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_proof.n.md -o tmp\selfhost-type-proof-parameter-kind.json --no-tree -j 1 --assert-io --dist web\dist`

2026-06-05 literal value payload HIR lowering checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/test_selfhost_hir_lowering_contract.js`
- `node nodesrc/test_selfhost_hir_expr_payload.js`
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-expr-literal-payload.json`（3/3 pass）
- `node nodesrc/tests.js -i stdlib/neplg2/core/lower --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-hir-literal-lowering.json`（2/2 pass）
- `node nodesrc/issues.js check --dir issues`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `git diff --check`

2026-06-05 Zenn policy compliance review checkpoint:

- `node nodesrc/test_selfhost_expr_call_reduce_contract.js`
- `node nodesrc/test_selfhost_hir_lowering_contract.js`
- `node nodesrc/test_selfhost_hir_expr_payload.js`
- `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-expr-zenn-comment-compliance.json`（3/3 pass）
- `node nodesrc/tests.js -i stdlib/neplg2/core/lower --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-hir-zenn-comment-compliance.json`（2/2 pass）
- `node nodesrc/issues.js check --dir issues`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `git diff --check`

2026-06-06 f32 literal checked payload and HIR lowering checkpoint:

- Rust 実装の現 checkpoint と同じく、selfhost `FloatLiteral` を `f32` 固定の literal として扱う境界を追加した。`f64` defaulting と numeric suffix 解決はこの issue の後続 slice に残す。
- `check/expr/argument.nepl` は `SelfhostExprPrefixItemKind::FloatLiteral` を `SelfhostTypeKind::F32` 証拠へ写像する。
- `check/expr/literal_payload.nepl` は `string::to_f32` で lexeme を一度だけ semantic `f32` へ変換し、失敗時は `F32Invalid` に写像する。Rust 側の `unwrap_or(0.0)` のような silent fallback は採用しない。
- `check/expr/argument_payload.nepl` に `SelfhostCheckedArgumentKind::F32Literal %f32` と constructor を追加し、HIR lowering が source token / lexeme を再読しない payload 境界を保った。
- `check/expr/model.nepl` / `call_reduce.nepl` に `LiteralF32Invalid` / `ArgumentLiteralF32Invalid` の写像を追加し、literal decode failure を unsupported expression へ潰さない。
- `hir/hir/expr.nepl` に `SelfhostHirExprKind::F32Literal`、`SelfhostHirExprPayload::F32Literal %f32`、`selfhost_hir_expr_f32_literal` を追加し、kind equality と child range accessor を明示分岐で更新した。
- `lower/hir/direct_call.nepl` は checked `F32Literal` payload を `SelfhostHirExprPayload::F32Literal` へ下ろす。lowering では float lexeme を解析しない。
- `doc/neplg2/self_host_neplg21_compiler_design.md` は、`FloatLiteral` 未対応という旧checkpoint説明を、現行の `f32` checked payload / HIR lowering 境界へ更新した。
- contract: `node nodesrc/test_selfhost_expr_call_reduce_contract.js` pass、`node nodesrc/test_selfhost_hir_expr_payload.js` pass、`node nodesrc/test_selfhost_hir_lowering_contract.js` pass、`node nodesrc/test_selfhost_checker_report_contract.js` pass、`node nodesrc/test_selfhost_zenn_review_gate_contract.js` pass、`node nodesrc/test_source_policy_no_line_count_limits.js` pass。
- focused module verification: `node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-expr-f32-literal.json` pass 5/5、`node nodesrc/tests.js -i stdlib/neplg2/core/lower --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-hir-f32-literal.json` pass 2/2。
- focused doctest verification: `node nodesrc/tests.js -i tests/stdlib/neplg2_call_reduce.n.md --no-tree --assert-io -j 1 -o tmp/selfhost-f32-literal-call-reduce.json` は total 6 中 5 pass。追加した `literal_f32_payload_decode` は pass したが、既存 `expression_line_segment_connects_to_call_reduction` 相当の doctest#5 が compile timeout 60s で errored。今回の f32 payload correctness ではなく、`check/expr` facade全体を読む既存 heavy compile path として継続監視する。
- 残件: numeric suffix language design、`NestedDirectCall` / `BlockResult` checked tree payload、既存 heavy doctest timeout の高速化 / focused test分割。

2026-06-06 block body sequence checked tree checkpoint:

- `SelfhostCheckedExprTree` に block body child root id 専用の `exprs %Vec SelfhostCheckedExprId` を追加し、`BlockSequence` は `SelfhostCheckedExprRange` でその範囲を参照する。node id の連続範囲は nested call / nested block node の挿入で body expression list と一致しないため、block sequence の authority には使わない。
- source-backed call reducer は trailing block body を一度だけ segment 化し、single body は `BlockResult`、multiple body は `BlockSequence` checked tree node として同じ tree に保存する。`MultipleSegments` は単一 input API の分類 signal として残るが、通常経路の複数式 body は dispatcher から `BlockSequence` に進む。
- 複数式 body では最後の expression だけを outer expected type の `BlockResult` expectation で検査し、非末尾 expression は `BlockSequenceDiscardedExpression` 由来の `unit` expectation で検査する。`unit` TypeId は複数式 block に入る時点で `selfhost_type_arena_find_kind` により一度だけ検索し、見つからない場合は `TrailingBlockBodyUnitTypeMissing` として fail-closed にする。
- HIR lowering は `BlockSequence` の expr range を fuel 付きで順に下ろし、HIR `Block` child range を作る。lowering で source token、scope lookup、callable candidate collection を再実行しない。
- stage1 public smoke は `unit; add 1 1` の success と `1; add 1 1` の non-unit rejection を `selfhost_check_expr_stage1_body_line` 経由で確認する。`tests/stdlib/neplg2_call_reduce.n.md` は既存 doctest#5 の compile timeout があるため、この slice の実行可能検証は focused stage1 smoke と source policy contract に置く。
- subagent review: `019e9b23-11f6-77f1-bf82-7aa52185f6e3` と `019e9b11-8ecf-77b1-82a4-f1150d214189` が blocker なし。後者の non-blocker 指摘に従い、非末尾 expression ごとの `unit` 型探索を dispatcher 側 1 回へ減らした。
- verification: `node nodesrc/test_selfhost_expr_call_reduce_contract.js` pass、`node nodesrc/test_selfhost_hir_lowering_contract.js` pass、`node nodesrc/test_selfhost_documentation_contract.js` pass、`node nodesrc/test_source_policy_no_line_count_limits.js` pass、`node nodesrc/test_selfhost_zenn_review_gate_contract.js` pass、`node nodesrc/tests.js -i stdlib/neplg2/core/check --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-check-core-sequence.json` pass 7/7、`node nodesrc/tests.js -i stdlib/neplg2/core/ty --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-ty-kind-find.json` pass 3/3、`node nodesrc/tests.js -i stdlib/neplg2/core/lower/hir/direct_call.nepl --no-tree -j 1 --assert-io --dist web/dist -o tmp/selfhost-hir-direct-call-sequence.json` pass 2/2。
