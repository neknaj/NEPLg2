2026-04-26 NEPLg2 Self-host

- `stdlib/doc-comment-boilerplate` branch で `ISS-20260426T060358681Z-STDLIB-DOC-COMMENTS-STILL-CONTAIN-GE-2D7384D1` に沿って boilerplate 化した stdlib doc comment を具体的な説明へ置き換える
- `nodesrc/selfhost-focused-tests` branch で `stdlib/neplg2` focused test の実行経路と JSON 確認を整備する

2026-04-26 NEPLg3 Migration

- `nepl-core-g3/` の Stage 1 着手内容を `doc/neplg3/impl/compiler_structure.md` に沿って実作業へ分解する
- `stdlib-g3/`、`tests-g3/`、`tutorials-g3/` の作成タイミングと CI job B の導入手順を具体化する
- `stdlib/neplg3/` の placeholder を実装単位へ分割し、最初の実行可能 doctest を追加する

2026-04-09 Playground

- terminal panel の shared terminal session / shared shell backend を設計する
- mobile / touch 環境での split / drag UI を調整する
- `tests/playground_editor/` に multi-file import / completion / fold / problem list 表示の fixture を追加する
- pointer 操作、fold click、scroll、completion UI の surface 回帰を CLI で検証できるようにする
- terminal worker protocol の compile progress / cancellation reason / stderr 表示を playground UI に反映する
- `tests/playground_editor/` 縺ｫ real-world source (複雑な型注釈 / nested block / multi-line string) 縺ｮ highlight fixture 繧定ｿｽ蜉縺励…urface 蝗槫ｸｰ繧ら判繧肴鋤縺医ｋ

2026-04-10 Tutorials

- `tutorials/getting_started/` 全体を `00_index.n.md` と同じ総ルビ方針へ統一し、章ごとの説明粒度・導入・まとめ・次章導線を整理する
- tutorial の doctest 群を章単位で見直し、学習内容に対して不足している実行例や回帰確認を追加する

2026-04-25 Review

- `RV-STDLIB-013` で stdlib collection doctest 群を所有型 API 移行後の実装に合わせ、`stdlib-test` を green に戻す
- `issues/index.md` の P1 Issue を修正順に分解し、compiler performance 計測 fixture と stdlib memory / I/O 回帰テストを追加する
- Issue を修正したら対応する `issues/items/*.md` の `resolved` / `status` / `updated` を更新し、`node nodesrc/issues.js index` と `check` を通してから確認結果を `note.n.md` に記録する

2026-05-31 Compiler performance / memoization purity

- `ISS-20260531T073211850Z-EXPRESSION-SUBTREE-INCREMENTAL-QUER-A91F3C2D` に沿って、リテラル置換を含む式枝差し替えを typed expression subtree query として扱い、warm `CompilerSession` で 0.1 秒以下にする
- `ISS-20260531T134245307Z-RPN-CODE-EDIT-STILL-SPENDS-SECONDS-AFTE-9C1F4F2D` に沿って、raw-init replay 後も残る RPN code edit の seconds-scale compile time を stage / function / summary kind ごとに分解し、次の cache 実装 issue へ切り分ける
- `ISS-20260601T174500000Z-OWNER-RETURN-SUMMARY-NEEDS-STABLE-CACH-8A7B61D2` に沿って、owner obligation pass cache 後も残る `compute_owner_return_summaries` の全関数固定費を stable mirror cache へ移す
- `ISS-20260524T225852366Z-PER-PROGRAM-COMPILE-TIME-EXCEEDS-DEF-189918C5` に沿って、RPN base compile `compile_ms=8931` / `resource_static_check=8318.313ms` を 0.5 秒未満へ近づけるため、stdlib prechecked artifact と Resource proof template の設計を実装へ落とす
- `ISS-20260531T025203216Z-PRIVATE-CACHE-EFFECT-MASKING-FOR-PUR-DF36DE4F` に沿って、`PrivateCache` / `PrivateState` internal effect を mask boundary なしでは `Pure` へ fold しない形で追加する
- `ISS-20260531T025408584Z-PRIVATE-STATE-MASKING-REQUIRES-RESOU-FCB116B4` に沿って、private region escape を Resource IR で拒否する proof domain を設計・実装する
- `ISS-20260531T035345811Z-SOURCECAPABILITY-NEEDS-PRIVATE-CACHE-5CC3FACF` に沿って、`PrivateCache` fresh region / non-escape proof と stdlib memo backend typecheck signature integration regression を実装する
- `ISS-20260531T035354039Z-MEMOKEY-AND-MEMOVALUE-NEED-STRUCTURA-592868B7` に沿って、MemoKey / MemoValue の structural purity rule を実装する
- `ISS-20260531T035402517Z-MEMOIZED-FUNCTION-VALUES-NEED-BACKEN-7B999CD7` に沿って、memoized function value の backend representation と identity observation ban を固定する
- `ISS-20260531T035410851Z-PRIVATE-EFFECTS-NEED-FOLD-AND-RESOUR-6DF550D2` に沿って、Private* effect の surface fold / diagnostics / Resource summary hash invalidation を接続する
- `ISS-20260531T060756264Z-MEMO-CALL-PHASE1-NEEDS-COMPILER-KNOW-2DB7C53C` に沿って、Phase 1 の `memo_call @pure_named_func` compiler-known primitive 境界を実装する
- `ISS-20260531T111205690Z-BINARY-INTERMEDIATE-ARTIFACTS-NEEDED-1C570649` に沿って、`.neplmeta` / `.neplobj` 相当の checked metadata と codegen fragment artifact を設計し、stdlib prechecked artifact と 0.1 秒 warm recompile の境界へ接続する
- `ISS-20260601T105003551Z-NEPLMETA-NOMINAL-TYPE-MATERIALIZER-NEEDED-5C9B2A10` に沿って、stable identity 付き `Named` / `Apply` を semantic impl target / trait application へ接続する
- `ISS-20260601T193116311Z-NEPLMETA-TRAIT-IMPL-MATERIALIZER-NEEDED-D3A0C2F1` と `ISS-20260531T111205690Z-BINARY-INTERMEDIATE-ARTIFACTS-NEEDED-1C570649` に沿って、source fallback / full compile 時に `resource_function_body_stable_hash` で `.neplobj` direct-call fragment payload を same-session object store へ保存し、Web / loader の import edge から `PublicInterfaceArtifactInputs` へ渡す
