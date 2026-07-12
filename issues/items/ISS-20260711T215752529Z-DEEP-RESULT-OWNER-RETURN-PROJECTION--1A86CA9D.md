---
id: ISS-20260711T215752529Z-DEEP-RESULT-OWNER-RETURN-PROJECTION--1A86CA9D
title: "Deep Result owner return projection reuses moved payload"
area: RESOURCE
status: open
resolved: false
priority: P1
type: bug
created: 2026-07-11
updated: 2026-07-12
target: nepl-core/src/resource/owner_return_apply_projection.rs
---

# ISS-20260711T215752529Z-DEEP-RESULT-OWNER-RETURN-PROJECTION--1A86CA9D: Deep Result owner return projection reuses moved payload

## 概要

A caller that consumes a deeply nested move-only registered owner through Result<BudgetStep, StepError> is rejected at the call expression with resource.owner.use_after_move on ReturnValue projections even though every branch consumes or frees the owner exactly once. F5nxj registered path command sink runtime reproduction fails first at writer_step_budget(recovered_writer, 0).

## 対象

- `nepl-core/src/resource/owner_return_apply_projection.rs`

## 根拠

- GUI production chainでは`writer_step_budget(owner, 1)`の`Result<BudgetStep, StepError>`適用時に、複数の独立deep owner leafが順番に`ReturnValue` transferされ、2件目以降がchecker内部で`Moved`として拒否される。
- outer/inner enum pathをpayload bindへ遅延する試作は単純な`Result<Step, E>`では元の誤報を消したが、GUIのstruct fieldと複数内部owner enumを含む大きなprojection graphでは誤報が残った。
- moved sourceを単にskipする方法は、別variantが選択された場合にownerを返せなくなる。outer variant名だけの平坦化や最初のleaf採用では正当なowner transferを証明できない。
- 同一`OwnerProjectionSource`だけを排他的deep targetの共通prefixへaggregate化する試作はunit対照を通過したがproduction誤報は残った。その後のsource追跡では、異なるleaf同士を同一owning authorityへ収束させる明示的raw alias contractは存在せず、sourceとwriterおよび内部Vecは独立allocationだった。raw viewはnon-owningでありauthority group根拠にできない。
- callee `StorageId`ごとのabstract group schemaも試作したが、production parameter seedはdeep owner leafごとに新しいStorageIdを割り当てるため、StorageId partitionはaggregate authority relationを表さなかった。人工的に同じStorageIdを与えるunitだけではproduction契約を証明できない。
- `resource_ir_owner_check_returns_deep_multi_owner_aggregate_through_result`はdistinct-authority topologyの通過基準を固定した。`resource_ir_owner_check_routes_distinct_deep_owners_across_result_variants`は2個の独立deep ownerをOkと2種類のowner-bearing Errへ返す排他的path対照をowner checkとnormal compileで固定した。この限定的な深度・3 return variantだけでは再現せず、次候補は実owner chain固有のprojection形状またはsummary適用規模である。
- 同じreturn topologyでowner-token直上に内部enumを置いても通過した。`resource_ir_owner_check_scales_distinct_deep_result_projection_leaves`は独立deep owner leafを1、2、4、8、16、32個へ増やして全件通過し、このtopologyでは32 leaf以下の単純な数閾値を除外した。次はproductionのVec/OwnedBuffer storage enum、相関する複数metadata field、追加のsummary extent mappingを段階的に加える。
- 探索fixtureでは実stdlib `Vec<i32>`型の2 owner parameter fieldを同じ3 return variantへ通し、owner checkとnormal compileを通過した。この限定topologyではVec/OwnedBuffer storage enumと通常metadataだけで再現しなかった。この環境のfocused実行は約166秒だったため恒久suiteには残さず、結果だけを境界証拠として記録した。
- `deep_distinct_owner_variant_summary_preserves_path_conditioned_mapping`はsummaryを直接検査し、2個のexact parameter sourceが共通のOk/direct Err/nested Err 3-path subsetを保ち、root/projection unconditional return、Maybe/Fresh/Unknown、target collapseへ劣化しないことを固定した。後続拡張ではsourceだけにSourceOnly Errを追加した非対称7対応も同じ不変条件で検査する。
- root関数のowner-summary callee closureだけを固定点計算するtest-only入口を追加し、cheap fixtureでfull固定点と同一summaryになることを固定した。production `writer_step_budget`へ適用しても64MiB stackで5分超となり、closure filteringだけでは不十分だった。高コストprobeは残さず、次はdirect-call構成要素をsynthetic wrapperへ段階移植する。
- cheap fixtureをproduction budget形状へ拡張し、2本のdirect Ok、下位Result委譲、writer authorityの明示dealloc後にsourceだけを返すSourceOnly Errを同時に検査した。summaryはsource 4 path／writer 3 pathの7 path-conditioned mappingを保持し、call-site owner checkとnormal compileも通過した。direct return mergeと非対称cleanupだけでは再現せず、次はlower source/writer内部projectionを段階移植する。
- 一回限りの実stdlib `Vec<i32>`探索でも、source/writer Vec、budget direct Ok、lower委譲、writer `free`、SourceOnly Errを組み合わせてowner checkが約68秒で通過した。高コストな非再現fixtureは残さず、次はproduction同様のsource 3 Vec／writer 2 Vec wrapperをsyntheticに検査する。
- 一回限りのsource 3 Vec／writer 2 Vec探索も同じbudget／cleanup形状で約52秒で通過した。高コストfixtureは残さず、production同数のVec allocationだけでは再現せず単独原因ではないことを確認した。`owner_summary_raw_i32_leaf`も再確認し、owner tokenを含むaggregateは`OwnerTokenOnly`となり通常のi32 metadataをleafへ加えないため、この仮説も棄却した。次はproduction型のenum alternativeとreturn projection graphを列挙し、同じ投影形状だけをcheap synthetic fixtureへ移植する。
- production moduleだけをimportして型投影を列挙するprobeは通常stackでoverflowし、64 MiBでもtypecheck完了前に90秒を超えたため撤去した。代わりにsource 3／writer 2の独立deep authority、direct Ok、下位Result委譲、AlreadyCompleted、nested SourceReadFailed、writer cleanup後のWriterPushFailed source-only returnを同時に持つcheap retained回帰を追加し、Resource owner checkとnormal compileが通過した。5 fieldそれぞれの二重moveが`OwnerUnavailable`になるnegative controlも固定した。production同数leafと外側variant topologyの組合せだけでも再現しないため、次はsource/writer内部enum projectionの宣言差を一段ずつ移植する。
- 5-owner回帰のleaf直上を非generic enumからproduction `VecStorage<T>`同型の`Apply<LeafState<T>> -> Ready payload -> Apply<RegionToken<T>>`へ変更した。positive全return pathと5 fieldのnegative controlは引き続き通過し、generic enumの型引数置換も単独原因ではないと確認した。次はsource側2 Vecのowner wrapperにauthorityとは別fieldのsibling phase enumを一段だけ追加する。
- source側2 authorityを同じwrapperへまとめ、owner-free siblingとしてPendingContour scalar／ActiveContour aggregate payload／Completedのphase enumを追加した。`OwnerTokenOnly`ではこのsibling自体はowner leaf投影から除外されるため、positive全return pathと5 leaf negative controlの通過が固定するのはsource authority suffixへ追加されたwrapper prefix一段であり、phase enumの影響ではない。次はowner-bearing suffixのwrapper深度を段階的に増やしてproduction chain深度との境界を検査する。
- owner-bearing suffix wrapper深度を1／2／4／8／16／32層へ増やすretained回帰を追加した。各深度でOk／direct Err／nested Errの正当なreturnが通り、同じdeep projected sourceの二重moveは`OwnerUnavailable`になり、32層はnormal compileも通過した。単一leaf suffix深度32以下では再現しないため、次はproduction同様の5 leaf非対称return topologyへ深度を組み合わせる。

## 問題

A caller that consumes a deeply nested move-only registered owner through Result<BudgetStep, StepError> is rejected at the call expression with resource.owner.use_after_move on ReturnValue projections even though every branch consumes or frees the owner exactly once. F5nxj registered path command sink runtime reproduction fails first at writer_step_budget(recovered_writer, 0).

## 影響

Valid production owner chains cannot be exercised by runtime fixtures; F5nxj integration is blocked despite normal compile and source-policy gates passing.

## 修正方針

GUI owner chainから型を段階的に削り、どのowner leafまたはprojection expansionで最初に誤報するかを二分する。独立allocationをflat group化せず、owner summary内のsource suffixとreturn projectionの対応を直接検査する。適用側変更は、最小再現、distinct-authority control、owner-bearing Err path control、genuine use-after-move診断を同時に満たす場合だけ採用する。

## 検証

Run the minimized Resource IR regression and tests/stdlib/gui_font_registered_face.n.md with the F5nxj controlled 8-command runtime contract: read retry, zero/negative budget, partial seal, eight writes, terminal completion, checked seal, and cleanup-only push failure.
