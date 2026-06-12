2026-06-04 stdlib / examples Zenn audit

- subagent の `stdlib/core` + `stdlib/alloc/collections` 監査結果を統合し、既存 issue と重複しない root cause を追加する
- subagent の `stdlib/std` + `stdlib/platforms` 監査結果を統合し、raw boundary / host effect / platform detail 漏れを issue 化する
- subagent の `stdlib/neplg2` + `stdlib/neplg3` + `stdlib/nm` + `stdlib/kp` 監査結果を統合し、prefix range / diagnostic / doc parser 由来の問題を issue 化する
- subagent の `examples` + `features` + `tests` + GUI/TUI 監査結果を統合し、旧文法、ret-only test、TS/Rust simulation、GUI/TUI substrate 逸脱を issue 化する
- `remote/main` を定期的に取り込み、別 agent の cfg-test 相当通常テスト基盤が入った時点で監査 checklist と各 issue の検証方針を更新する

2026-06-01 GUI/TUI standard library

- `alloc/gui` の allocator-backed layout を flex / grid / scroll policy、text buffer node 対応へ拡張し、text line break / text hash based cache invalidation、pointer capture / gesture、stateful pointer routing と、Web / native / mobile raw keyboard normalization、terminal の Function key などの追加 ANSI / CSI sequence、途中入力 buffering を追加する
- GUI/TUI executable NEPLg2 code の括弧なし規約を source policy regression へ組み込み、stdlib implementation / doctest / `tests/stdlib/gui_*.n.md` / examples の回帰を自動検出する
- `GuiEffectBatch` の bounded checkpoint 実装を、`alloc` collection の所有権 contract が安定した段階で `Vec GuiEffect` へ置き換える
- Web Playground の stdout protocol fallback を正式な Wasm host import ABI へ置き換え、NEPL/Wasm が生成した `DrawCommand` stream を `neplGuiHost.beginFrame` / `pushCommand` / `endFrame` 相当へ直接渡す
- Web / native の formal presentation ABI に tile / bitmap / row / RLE payload を追加し、Mandelbrot などの true HD raster を stdout の大量 `fill_rect` stream ではなく bounded command transport で扱う
- Web Playground の `GuiWebEvent` action / pointer down-move-up-cancel / keyboard / single-scalar text input / window resized-close / timer checkpoint を、IME composition / multi-scalar text、window focus-unfocus policy、lifecycle variant、session id formalization へ拡張する
- Mandelbrot progressive rendering を NEPL app の update loop で処理する
- Paint example を直近 stroke slot の軽量 model から persistent canvas / stroke storage へ拡張する
- stdout fallback の timer request を正式 Wasm host import ABI と `std/gui` scheduler / timeslice contract へ移す
- `nepl-gui-native` の framebuffer renderer を、`std/gui::GuiHost` と `platforms/gui/native` の正式 `present` 実装へ寄せる
- 既存 `platforms/wasix/tui` の raw ANSI / TTY / line buffer API を `platforms/gui/terminal` backend detail へ段階移行する
- `features/tui` を互換 path として保ちながら、内部を `features/gui` + terminal backend へ差し替える
- embedded backend の dirty region generic capacity / compression、display adapter、optional `FlushTarget`、polling input を追加して no_alloc contract を実機風に検査する

2026-04-26 NEPLg2 Self-host

- `ISS-20260604T034255066Z-SELFHOST-PARSER-AND-CHECKER-DO-NOT-I-7C1C8941` の後続 slice として、lambda / borrow / source-backed pipe suffix の lambda などの複合式 success path、generic instantiation inference、trait solving、indirect call、`memo_call` の private cache proof / backend representation と `MemoKey` / `MemoValue` aggregate layout / trait evidence solver 入力を追加する。単一式 block、複数式 block sequence、nested `BlockIntro`、単一 `left |> named_target suffix...` pipe、単一 `left |> %fn ... named_target suffix...` pipe、ascribed pipe target による同名 overload narrowing、注釈無し pipe target の単純な引数列 narrowing、`Match` 0 件かつ `SourceBackedRequired` 1 件だけで `SelectionBlockedUnsupported` が無い `%T literal` / `NamedValue` / 単一候補 nested call、source-backed argument 範囲全体を完全消費する同名 overloaded nested call、通常 call 引数で outer continuation を使う同名 overloaded nested call、pipe left で final-range を使う overloaded nested call、単一 pipe trailing block argument、`left |> named_target suffix... |> named_target suffix...` の pipe chain、pipe chain trailing block argument、HIR `Call` の monomorphic DefId / callable type / effect 付き callee identity、selfhost `MemoizedFunctionValue` の HIR leaf payloadとpure monomorphic DefId identity gate、`memo_call @func` の shared compiler-known primitive identity gate、selfhost `MemoKey` / `MemoValue` primitive fail-closed predicate、aggregate evidence consumer table、field/proof/hazard summary 付き aggregate proof を consumer record へ変換する producer gate、canonical type key / solver policy indexed proof store は接続済みで、generic call は stable type-argument identity 追加まで `GenericCallIdentityUnsupported` で拒否し、generic memoized identity は `GenericUnsupported` で拒否する。pipe chain と nested suffix と trailing block の checked tree topology、後段 fail-closed smoke、ascription 一致 0 件 / 複数件、注釈無し target の一致 0 件 / 複数件、注釈無し target の source-backed required 混在 / 複数 source-backed required / selection-blocked fail-closed 境界を含む代表的な pipe smoke も固定済みなので、残る legacy summary variant は source-less / fail-closed 境界として扱い、通常 HIR lowering authority に戻さない
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
- `ISS-20260524T225852366Z-PER-PROGRAM-COMPILE-TIME-EXCEEDS-DEF-189918C5` に沿って、RPN cold base native の全体 wall-clock を 0.5 秒未満へ近づけるため、bundled stdlib `.neplmeta` / `.neplproof` preseed、typecheck/interface artifact、bootstrap proof generation 短縮、owner return summary stable mirror、`dealloc_raw` / `apply_op` / Stack owner flow の Resource proof template を実装へ落とす。2026-06-02 follow-up では persistent `.neplproof` codec、native disk proof cache、summary/pass snapshot preseed、owner obligation pass-level snapshot により RPN proof-backed `resource_static_check` median は約 `0.34-0.39s` まで下がった。さらに raw-alias 専用 dependency view と raw pointer / raw identity relevance filter により、測定条件を揃えた no-stage median は `1013.481ms -> 928.442ms` になった。最新 RPN import graph narrowing checkpoint では no-stage median `844.789ms`、stage median `loader_load=334.056ms`、`check_pipeline=502.638ms`、`resource_typecheck=115ms`、`resource_static_check=360ms` である。loader detail の module count は `115 -> 99` まで減ったが、0.5 秒未満にはまだ届かない。`loader_load` と dependency typecheck が残るため、次は `ISS-20260602T134118244Z-NATIVE-CHECK-SHOULD-USE-PRE-TYPECHEC-31F9C9CD` に沿って native CLI `--check` を `.neplmeta` / typed interface artifact に接続する。local condition memo / parameter condition 重複削減 / owner summary relevance filter / i32 scalar proof 無効化 / merge済みModule cache store clone除去 / one-shot native provider session cache / type arity hint BTreeMap index 化は RPN 実測で改善しなかったため、主経路にしない
- `ISS-20260531T025203216Z-PRIVATE-CACHE-EFFECT-MASKING-FOR-PUR-DF36DE4F` に沿って、`PrivateCache` / `PrivateState` internal effect を mask boundary なしでは `Pure` へ fold しない形で追加する
- `ISS-20260531T025408584Z-PRIVATE-STATE-MASKING-REQUIRES-RESOU-FCB116B4` に沿って、private region escape を Resource IR で拒否する proof domain を設計・実装する
- `ISS-20260531T035345811Z-SOURCECAPABILITY-NEEDS-PRIVATE-CACHE-5CC3FACF` に沿って、`PrivateCache` fresh region / non-escape proof と stdlib memo backend typecheck signature integration regression を実装する
- `ISS-20260531T035354039Z-MEMOKEY-AND-MEMOVALUE-NEED-STRUCTURA-592868B7` に沿って、MemoKey / MemoValue の structural purity rule を実装する。selfhost の primitive fail-closed predicate、aggregate evidence consumer table、field/proof/hazard summary 付き aggregate proof を consumer record へ変換する producer gate、canonical type key / typed solver policy indexed proof store、typed source definition record materializer、table-backed current source registry / missing duplicate validator、actual trait definition source scanner の fail-closed candidate table、type constructor layout evidence table / field range validator、scanner candidate table と stable public surface evidence を突き合わせる stable source record producer gate、typed module / public surface / trait signature seed から stable evidence table を作る Phase 1 producer、`SelfhostModuleAst` の public marker trait declaration から typed seed table を作る public surface seed materializer、local `MemoKey` / `MemoValue` marker trait pair の typed seed から module public surface hash を作る Phase 1 hash materializer、public marker trait signature shape evidence と seed 側の normalization boundary 接続、method-bearing trait body の parser-level method segment evidence、method name / type annotation / default body の standalone stable signature normalizer、method-bearing trait definition を token-aware public surface seed / hash pipeline へ通す facade-external internal gate、stable trait definition key producer、stable nominal key table / canonical type fingerprint sidecar projection、proof store の stable fingerprint sidecar / stable push / stable lookup / stable sidecar index / stable duplicate rejection / store-local stable identity boundary、serialized canonical key tree bytes codec、decoded single-record proof store append boundary、decoded batch preseed boundary、decoded `.neplproof` index table validation boundary、sorted artifact index candidate range contract、decoded record から sorted sidecar index を作る producer boundary は接続済み。
  次は re-export / import graph / public non-trait declaration を含む full public surface hash、Copy / Drop / Eq / Hash pure evidence の実計算、recursive aggregate / cycle boundary、`.neplproof` reader / serializer、永続 artifact 用 stable map / serialized index、generic instantiation 用 stable type argument identity を接続する。
  性能残件として、registry convenience path の candidate scanner + materializer 二重走査、facade-external token gate と seed module private token scan の重複を、次以降の同 issue slice または RPN cold base 高速化 issue へ切り分ける。hash materializer 単体の unsupported pre-scan + seed scan 二重走査は 2026-06-12 checkpoint で hash-owned single-pass scan へ移し、`trait_body_segmenter` の next-index recomputation は 2026-06-12 checkpoint で private build payload による single computation cursor へ移した。
- `ISS-20260531T035402517Z-MEMOIZED-FUNCTION-VALUES-NEED-BACKEN-7B999CD7` に沿って、memoized function value の backend representation と identity observation ban を固定する
- `ISS-20260531T035410851Z-PRIVATE-EFFECTS-NEED-FOLD-AND-RESOUR-6DF550D2` に沿って、Private* effect の surface fold / diagnostics / Resource summary hash invalidation を接続する
- `ISS-20260531T111205690Z-BINARY-INTERMEDIATE-ARTIFACTS-NEEDED-1C570649` に沿って、`.neplmeta` / `.neplobj` 相当の checked metadata と codegen fragment artifact を設計し、stdlib prechecked artifact と 0.1 秒 warm recompile の境界へ接続する。direct-call relocation producer、same-session store、raw wasm leaf fragment、relocation dependency closure、persistent `.neplproof` codec / native disk proof cache は実装済み。RPN loader/process-directives 計測では `.neplproof` read ではなく loader / dependency body merge / typecheck が支配的だったため、次は bundled stdlib `.neplmeta` または materialized typed public surface から native CLI `--check` の依存先 environment を構成する。ただし現行 prepare path は selected materialized callable body 欠落を拒否するため、check 専用 public interface + Resource proof summary 境界または `.neplobj` body fragment 併用を先に設計する。その後に generic instantiation hash、string/data relocation、raw LLVM body、function value、memoized function value、PrivateCache proof をそれぞれ別境界として設計する
- `ISS-20260601T105003551Z-NEPLMETA-NOMINAL-TYPE-MATERIALIZER-NEEDED-5C9B2A10` に沿って、stable identity 付き `Named` / `Apply` を semantic impl target / trait application へ接続する
- `ISS-20260601T193116311Z-NEPLMETA-TRAIT-IMPL-MATERIALIZER-NEEDED-D3A0C2F1` に沿って、trait impl materializer が依存 body skip 後も impl resolution を fail-closed に復元できる stable surface を追加する
