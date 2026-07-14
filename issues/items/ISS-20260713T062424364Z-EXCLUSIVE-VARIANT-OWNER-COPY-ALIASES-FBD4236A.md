---
id: ISS-20260713T062424364Z-EXCLUSIVE-VARIANT-OWNER-COPY-ALIASES-FBD4236A
title: "Exclusive variant owner copy aliases sibling payloads"
area: RESOURCE
status: open
resolved: false
priority: P1
type: bug
created: 2026-07-13
updated: 2026-07-14
target: "nepl-core/src/resource/initialized_alias.rs, nepl-core/src/resource/owner_variant.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260713T062424364Z-EXCLUSIVE-VARIANT-OWNER-COPY-ALIASES-FBD4236A: Exclusive variant owner copy aliases sibling payloads

## 概要

Mutually exclusive nested enum owner return targets are unioned into one raw alias group, so an outer Result payload bind retires unselected sibling payloads as Moved.

## 対象

- `nepl-core/src/resource/initialized_alias.rs, nepl-core/src/resource/owner_variant.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `GuiSfntSimpleGlyphRenderStrokeSourceSegmentCursorTerminal` を `Result::Ok` から受け取った直後の inner match で、`Completed`、`LineSegment`、`QuadraticSegment` 配下の同形 owner place がすべて `Moved` と診断される。
- `copy_exclusive_applied_target` は排他的targetへ同じstorage identityをmaterializeする一方、`copy_alias_if_tracked_preserving_target`によりsibling targetを同じraw alias groupへunionしていた。
- outer `Result::Ok` payload bindの`retire_transferred_aliases`はinner variant選択前にalias groupを退役させるため、未選択siblingまで同時にmoveされた。
- `ISS-20260711T215752529Z-DEEP-RESULT-OWNER-RETURN-PROJECTION--1A86CA9D`を修正したcommit `19ce68712`のsummary canonicalizationは維持し、本件はそのruntime application側で導入された別regressionとして扱う。

## 問題

Mutually exclusive nested enum owner return targets are unioned into one raw alias group, so an outer Result payload bind retires unselected sibling payloads as Moved.

## 影響

Valid F5ku metric drain completion, line, and quadratic terminal branches fail with resource.owner.use_after_move and block the F5nxl production runtime gate.

## 修正方針

Copy owner/storage/scalar provenance to exclusive variant targets while keeping each sibling target in a disjoint raw-owner alias group.

`OwnerState`、`StorageId`、storage origin、scalar facts、raw view classificationは複製する。raw-marked sourceから作るtargetは独立singleton markとし、raw alias groupとraw-view originはsibling間でunionしない。retire側で診断だけを抑止せず、誤ったalias relation自体を作らない。

## 検証

Focused alias and owner-variant unit tests, production-shaped nested Result/terminal ResourceIR normal compile, F5nxl runtime fixture, trunk build, and normal GUI compile.

## 2026-07-13 診断状況

- raw alias groupを排他的targetごとに分離し、aggregate owner subtreeを同じstorage identityから複製する修正で、F5nxl production fixtureの診断は12件から2件まで減少した。
- aggregate identityはstorage集合だけでなくsource-relative projectionとLive / MaybeFreed / Reserved / Moved / Freed stateを保持し、同じstorage集合でもfield配置を入れ替えたaggregateを別identityとして扱う。
- 残る2件は`drain_to_complete`の`Result::Ok`配下で、`Completed / MetricPushed / StateUpdated`が共有する2個のexact owner leafに限定される。
- return availability snapshot時点ではsourceはtransferableだが、同一matchの後続ReturnValue適用時にはraw alias解決先が`Moved`になる。target順序変更、Moved target復活、宣言identityだけのfallback、aggregate/exact state snapshotはいずれもproduction fixtureを解消せず、診断拡散または無変化のため撤回した。
- 再開時は`apply_match_arm_returns`だけを緩和せず、先行ReturnValueの`move_owner_out`から`retire_transferred_aliases`までのraw alias canonical変更と、後続entryのsource解決をproduction-shaped recursive Result回帰へ縮約する。
- 最新trunk artifactで180秒まで許可したfocused fixtureでも2件を再確認した。診断対象は`checkpoint_drain_to_complete`がrecursive call resultをそのまま返す`Temporary(ResourceId(13))`のpath/raster Vec storage leafである。
- Vecの`buffer.storage.Owned`投影、CursorからPlanへのwrapper縮退、下位terminal/errorの再包装、completed invariant、recursive Checkpoint / BuildErrorを含む縮約回帰は正常終了した。Moved sourceをexclusive siblingのLive alias signatureへ寄せる案もproductionに効果がなく撤回したため、次はrecursive summary fixed-pointのparameter return source生成・canonicalizationを追跡する。
- wrapper constructorをhelper callへ変更した縮約回帰も正常終了した。Return terminatorでtemporary sourceだけmaterializeする変更はbranch/matchとの差を埋めるが、最新trunk productionの2診断を変えず撤回した。再開時はTemporary(ResourceId(13))に対応するpending return entryのsource / target / storage signatureを適用前後で観測し、再現回帰を先に赤くする。

- 2026-07-14: return result 自身・子孫の pending source move を最後の result move に一任する方向限定修正を検証した。詳細 resource IR 回帰と trunk build は通過したが、focused runtime fixture は Temporary(ResourceId(13)) field 2 / field 3 配下の use-after-move 2 件を維持したため撤回。次は pending return source、alias-resolved source、result の前後状態を対象関数だけで観測し、その形状を red regression に固定してから修正する。
- 2026-07-14: 対象関数限定traceでReturn terminatorの`move_result_effect_sources_out`と`materialize_result_owner_effects`はいずれもpending return entry 0件と確認した。残存Moved aliasはterminator内で新規生成されず、先行constructor/accessor summaryからTemporaryへ残存してfinal result moveに再訪される。traceは撤回済み。red regressionへimpure recursive checkpoint、opaque wrapper、sink sibling actual Vec、constructor/accessor summary境界を追加する。
- 2026-07-14: recursive縮約へconstructor summary chainを追加しても正常終了し、actual generic Vec版も181.93秒で正常終了した。ReturnValueのMoved descendantからavailable raw aliasを回収する限定修正はproduction診断を変えず撤回し、対象2 leafにはlive alias fallbackが無いことを確認した。selected result targetが先行summary適用時点でMovedになる生成地点の観測へ進む。
- 2026-07-14: Temporary targetの通常transfer traceは0件。outer match arm outputでは対象2 leafがarmによりLiveまたはMovedで、失敗armはarm.value transfer前からMovedだった。原因範囲を`apply_match_arm` / inactive payload retirement / recursive arm call-summary materializationへ縮小し、観測コードは撤回した。
- 2026-07-14: direct ownerがあるaliased descendantをcollectorで一律除外する修正はReturnValue 2件を残したままCompletedOwner error branchへMatchValue 6件を追加したため撤回。alias補完は排他的branchで必要であり、arm patternと段階を特定してから局所契約を修正する。
- 2026-07-14: stage traceでscrutineeは全段階Moved 0、match outputはpre/post apply/post opsがすべてMoved 21件だった。最初の不正state生成をouter match前の`drain_owner_step` call-result owner summary適用へ特定し、match instrumentationは撤回した。
- 2026-07-14: production同型の単一DrainError structへcursor/completed ownerをsibling Optionとして格納し、metricsを固定fieldにした縮約は正常終了した。cursor terminalを別impure Result helperへ分割してtail-forward summaryを追加しても正常終了したため、次はproduction helper内のborrow-only validation Result合流と、completion/noncompletion両branchからの同一cursor-step summary適用を順に縮約する。
- 2026-07-14: borrow-only validation合流とcompletion/noncompletion両経路を縮約へ追加しても正常終了した。実summaryでは通常parameter return sourceは空で、問題のsink leafはvariant projection returnだけに存在し、cursor recoveryとcompleted recoveryが同一`DrainError`内の別struct fieldへ合流していた。
- 2026-07-14: 同一sourceから非overlap struct siblingへのcopyを許すcompiler緩和はproduction fixtureを通したが、同時liveな2 fieldへunique ownerを複製できるためsubagent reviewでunsoundと判定し、完全に撤回した。compiler側はenum payloadで証明できる排他targetだけをcopyする契約を維持する。
- 2026-07-14: productionの`cursor: Option<Cursor>` / `completed_plan_owner: Option<PlanOwner>`という暗黙のexactly-one invariantを、private `DrainRecoveryOwner::{Cursor, Completed}` enumへ置換した。metricsは共通fieldに1個だけ保持し、freeはenum matchで選択authorityを1回、metricsを1回解放する。詳細ResourceIR回帰とF5nxl production runtime fixtureはこの型レベル排他表現で通過した。
