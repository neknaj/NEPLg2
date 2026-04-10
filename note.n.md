# 2026-04-09 メモ (複数タブ切替で syntax highlight が壊れる問題の修正)

- [原因]:
  - editor の `setText()` は tab 切替や file open のような「別文書への切替」にも使われていたが、内部では通常の `updateText()` と同じ経路で language provider に流していた。
  - `NEPLg2LanguageProvider.updateText()` は増分編集中の provisional payload を前提にしているため、前タブと次タブのような unrelated な文書間でも差分 token を流用し、一時的に壊れた highlight を出していた。
  - さらに workspace では editor panel 間で provider インスタンスを共有しており、複数 editor panel を開くと `onUpdate` と解析状態が相互上書きされる構造だった。
- [修正]:
  - `web/src/language/neplg2/neplg2-provider.ts`
    - pending timer / idle callback の共通停止処理を追加した。
    - tab/file 切替用の `replaceDocumentText()` を追加し、増分 provisional を使わずに同期解析へ直行するようにした。
  - `web/src/editor/editor.ts`
    - `setText()` を full-document replace として扱い、通常編集の `updateText()` と切り分けた。
  - `web/src/workspace/panel-manager.ts`
    - editor panel ごとに `createNeplProvider()` から新しい provider を作るようにして、解析状態と callback を panel 間で共有しないようにした。
  - `web/src/main.ts`
    - panel manager へ provider factory を渡す形に変更した。
  - `nodesrc/playground_editor_surface_test_runner.js`
    - `setText()` が incremental ではなく full-document replace を使うことを固定した。
- [確認済み]:
  - `npm --prefix web run build:ts`: 通過
  - `node nodesrc/playground_editor_surface_test_runner.js`: 通過
  - `node nodesrc/playground_drag_drop_test_runner.js`: 通過
  - `node nodesrc/playground_workspace_test_runner.js`: 通過
  - `node nodesrc/playground_tab_transfer_test_runner.js`: 通過
  - `node nodesrc/playground_editability_test_runner.js`: 通過
  - `trunk build --release`: 通過
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`: 12/12 passed
- [plan.mdとの差異]:
  - 今回は syntax highlight 崩壊の根本原因に絞り、tab switch を full-document replace 化した。さらに大きい pure core 移譲ではなく、surface と provider 境界の責務修正で収束させている。

# 2026-04-09 メモ (tab bar drop を split から分離)

- [確認]:
  - panel drag/drop を広げた結果、editor panel の tab bar 上に drop しても panel 全体の drop 判定が先に効き、edge 判定で split される経路が残っていた。
  - 期待される挙動は `tab bar = tab 追加 / panel merge`、`panel body = split / move` の分離であり、ここが surface の責務として曖昧だった。
- [実装]:
  - `web/src/workspace/drag-drop.ts`
    - drag payload と tab bar 上の drop action 解決を pure helper に切り出した。
  - `web/src/workspace/panel-manager.ts`
    - editor panel の `tabbar` に専用 `dragover` / `drop` を追加した。
    - tab bar 上では editor tab は attach、explorer file は open、editor panel は merge として処理し、panel 本体の split 判定へ流さないようにした。
    - panel 本体の drop とは highlight も分離した。
  - `web/styles.css`
    - tab bar 専用の drop highlight を追加した。
  - `nodesrc/playground_drag_drop_test_runner.js`
    - tab bar の drop intent が attach/open/merge に解決されることを固定した。
- [確認済み]:
  - `npm --prefix web run build:ts`: 通過
  - `node nodesrc/playground_drag_drop_test_runner.js`: 通過
  - `node nodesrc/playground_tab_transfer_test_runner.js`: 通過
  - `node nodesrc/playground_workspace_test_runner.js`: 通過
  - `node nodesrc/playground_editability_test_runner.js`: 通過
  - `node nodesrc/playground_editor_surface_test_runner.js`: 通過
  - `trunk build --release`: 通過
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`: 12/12 passed
- [plan.mdとの差異]:
  - 現段階では tab bar drop の意図解決を pure helper 化し、実際の reorder までは入れていない。目的は split 誤判定の根絶と、tab 追加/merge の明確化に絞っている。

# 2026-04-09 メモ (workspace drag/drop を panel / tab / explorer file に拡張)

- [確認]:
  - review の `insertText()` 二重 provider 通知は現行 `web/src/editor/editor.ts` では `replaceTextRange()` 経由に整理済みで、同じ不具合は再現しなかった。
  - ただし workspace 側の drag/drop は panel header 移動に偏っていて、editor tab や explorer file を split tree に自然に流し込む導線が不足していた。
- [実装]:
  - `web/src/library/tabs.ts`
    - active tab の内容と zoom を保ったまま panel 間を移せるように `detachTabByPath()` / `attachTab()` / `exportTabs()` を追加した。
    - tab 要素を draggable にして、drag start を panel manager へ通知するようにした。
    - panel center merge も VFS 再読込ではなく tab snapshot の移送に切り替え、unsaved に近い状態を落とさないようにした。
  - `web/src/library/explorer.ts`
    - file item を draggable にし、panel manager へ file path を渡せるようにした。
  - `web/src/workspace/panel-manager.ts`
    - drag payload を `panel` / `editor-tab` / `explorer-file` に分離した。
    - panel drop 判定を payload 種別ごとに整理し、tab/file を editor center drop で開く、edge drop で新しい editor split を作って開くようにした。
    - 既存の panel drag も同じ payload 処理に統合した。
  - `web/styles.css`
    - draggable tab / explorer item の視覚状態を追加した。
  - `nodesrc/playground_tab_transfer_test_runner.js`
    - DOM なしで tab detach/attach/merge の内容保持を固定する headless runner を追加した。
- [確認済み]:
  - `npm --prefix web run build:ts`: 通過
  - `node nodesrc/playground_tab_transfer_test_runner.js`: 通過
  - `node nodesrc/playground_workspace_test_runner.js`: 通過
- [plan.mdとの差異]:
  - panel / tab / explorer file の drag/drop は追加できたが、今回のドロップ先は panel 全体ベースで、tabbar 専用の細粒度 drop indicator まではまだ入れていない。

# 2026-04-08 メモ (editor state 更新経路の統一)

- [状況]:
  - review で、undo/redo、file load、selection 付き入力の経路が `applyCoreRuntimeState()` と異なる後処理を持っており、cursor 依存の highlight と provider 更新回数に不整合が残っていることが分かった。
- [原因]:
  - `web/src/editor/editor.ts` が state 適用と text 置換を各メソッドで個別に実装しており、`setText()`、`applyState()`、`insertText()`、`deleteSelection()`、`replaceSelectionAndSetCursor()`、`applyTextEdit()`、`acceptCompletion()` の間で `updateLines()`、`updateText()`、`updateBracketMatching()`、`onCursorChange` の呼び方が揃っていなかった。
  - `editor-input-handler.ts` の Backspace / Delete fallback も text を直接書き換えており、共通規則を外れていた。
- [修正]:
  - `CanvasEditor` に `applyResolvedEditorState()` と `replaceTextRange()` を追加し、text mutation と state-only mutation の後処理を共通化した。
  - `applyCoreRuntimeState()`、`setText()`、`applyState()`、`insertText()`、`deleteSelection()`、`replaceSelectionAndSetCursor()`、`applyTextEdit()`、`acceptCompletion()` を共通 helper 経由へ寄せた。
  - text 非変更時は `updateLines()` / `updateText()` を走らせず、cursor / selection / overwrite 更新では bracket matching・occurrences・cursor change 通知だけを同期するようにした。
  - selection 付き置換は `replaceTextRange()` で一括処理し、provider 更新が 1 回だけになるようにした。
  - `editor-input-handler.ts` の Backspace / Delete fallback も `replaceTextRange()` を使うように統一した。
  - `nodesrc/playground_editor_surface_test_runner.js` を拡張し、cursor move、selection/overwrite 更新、file load 相当の reset、selection replacement の回帰を DOM なしで確認できるようにした。
- [確認]:
  - `node nodesrc/playground_editor_surface_test_runner.js`: 通過
  - `npm --prefix web run build:ts`: 通過
  - `trunk build --release`: 通過
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`: 12/12 passed
- [plan.mdとの差分]:
  - editor の state 更新責務を pure core へ全部移し切る前段として、surface 側の state mutation を単一 helper に集約した。これで今後の browser adapter / pure core への分離でも、surface 側で扱うべき副作用が明確になった。

# 2026-04-05 メモ (矢印キーで highlight が消える問題の修正)

- [状況]:
  - 文字入力では syntax highlight が維持される一方、矢印キーでカーソル移動した直後だけ highlight が消える不具合があった。
- [原因]:
  - `web/src/editor/editor.ts` の `applyCoreRuntimeState()` が、text が変わっていないカーソル移動でも毎回 `updateLines()` を呼んでいた。
  - `updateLines()` は `tokensByLine` と `diagnosticsByLine` を破棄するため、矢印キー移動では render cache だけ消え、その後は `updateText()` が走らないので解析 payload による再構築も起こらなかった。
- [修正]:
  - `applyCoreRuntimeState()` は runtime state の `text` が実際に変わった場合だけ `updateLines()` と `updateText()` を呼ぶように変更した。
  - これにより、カーソル移動では既存の highlight cache を保持し、文字編集時だけ行情報と解析更新をやり直すように整理した。
  - surface 回帰確認として `nodesrc/playground_editor_surface_test_runner.js` を追加し、DOM なし mock で `applyCoreRuntimeState()` の cache 保持を検証できるようにした。
  - 同 runner で cursor move だけでなく selection 変更と overwrite mode 切替でも text 非変更なら cache を壊さないことを確認する。
- [確認]:
  - `node nodesrc/playground_editor_surface_test_runner.js`: 通過
  - `npm --prefix web run build:ts`: 通過
  - `trunk build --release`: 通過
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`: 12/12 passed
- [plan.mdとの差分]:
  - editor core と surface の責務分離方針に沿って、cursor movement を text mutation と切り離した。surface での cache 管理と analysis 更新境界が以前より明確になった。

# 2026-04-03 メモ (hover expr 抽出と遅延表示の修正)

- [状況]:
  - hover から fallback は除去済みだったが、`expr_span` が token 幅しか持たないケースでは popup に `token:` 相当の短い内容しか出ず、要求の「その token から始まる式と型」の表示になっていなかった。
  - hover の表示タイミングも 100ms 固定で、実際には「1 秒以上マウスが止まったときに表示」という要件を満たしていなかった。
- [原因]:
  - `web/src/editor-core/language-analysis.ts` が semantic token の `expr_span` をそのまま採用しており、AST から式全体の span を補完していなかった。
  - `web/src/editor/editor-input-handler.ts` はマウス移動のたびに位置差分だけを見て 100ms timer を張っており、同じ token 上での微移動や静止時間の条件を正しく扱っていなかった。
- [修正]:
  - analysis bridge に AST 走査を追加し、hover token と同じ開始位置を持つ最小の式 span を拾って hover の `expr:` に使うようにした。
  - hover から `token:` 行は出さず、`expr:` と `type:` を中心に表示するように整理した。
  - bridge 側の hover / definition の参照 fallback も外し、analysis は bridge の token insight 前提に統一した。
  - input handler はマウスが少しでも動いたら hover timer を張り直し、1 秒静止したときだけ popup を出すように変更した。
  - popup 表示時は timer 設定時の座標と最新座標が一致する場合だけ出すようにし、移動直後の古い hover を防いだ。
- [確認]:
  - CLI fixture `tests/playground_editor/analysis_hover_expr_from_ast` を追加し、semantic `expr_span` が token 幅でも AST から `print_color ansi_green "ok"` を hover に出せることを確認する。
- [plan.mdとの差分]:
  - hover 表示は analysis core 側で式 span を復元する段階に入った。surface は trigger と popup 制御に責務を絞り、hover 内容の決定は bridge 側へ寄せている。

# 2026-04-03 メモ (analysis fallback 全廃)

- [状況]:
  - hover 修正後も provider 内に bridge 依存の analysis 実装と独自 fallback 実装が並存しており、将来また表示差や定義ジャンプ差が再発する構造だった。
- [原因]:
  - `web/src/language/neplg2/neplg2-provider.ts` が `window.NEPLPlaygroundLanguageAnalysis` を optional 扱いし、payload 生成・hover・definition・occurrences・token insight をそれぞれ別実装で補っていた。
- [修正]:
  - analysis bridge を必須依存に変更し、bridge 不在時は即エラーになるようにした。
  - provider から payload 生成・hover・definition・occurrences・token insight の fallback 分岐を除去し、analysis 系 API を bridge 1 本に統一した。
- [確認]:
  - これにより hover 内容や参照解決は bridge 実装だけを見ればよくなり、surface と CLI fixture の表示仕様が一致しやすくなった。
- [plan.mdとの差分]:
  - bridge へ分析責務を寄せる方向は計画と一致している。今回の変更で provider 側の重複実装を減らし、editor surface と analysis core の境界を明確化した。

# 2026-04-03 メモ (hover fallback 表示の整合)

- [状況]:
  - hover は bridge 経路では `expr:` と `type:` を先頭表示するようになっていたが、画面上では provider の fallback 経路が使われる場面があり、その場合だけ token 主体の旧表示が残っていた。
- [原因]:
  - `web/src/language/neplg2/neplg2-provider.ts` の `getHoverInfo()` fallback 実装が、`web/src/editor-core/language-analysis.ts` の新しい hover 整形規則と同期していなかった。
- [修正]:
  - provider 側にも式断片抽出を行う `_formatHoverExpression()` を追加した。
  - fallback hover は `expr: ...`、`type: ...` を優先し、token は式断片と異なる場合だけ補助表示するように統一した。
- [確認]:
  - `npm --prefix web run build:ts`: 通過
  - `trunk build --release`: 通過
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`: 11/11 passed
- [plan.mdとの差分]:
  - hover 表示仕様の再設計は進んでいるが、実運用で使われる全経路をそろえないと見た目が一致しないことが分かった。今後も bridge と provider fallback の二重実装箇所は都度同期確認が必要。

# 2026-04-03 メモ (web playground editor surface 修正)

- [状況]:
  - playground editor の core 側は CLI suite で通っていた一方、surface 側に hover/completion 非表示、DPR ずれ、マウス座標ずれの問題が残っていたため、editor-dom-ui / editor-input-handler / editor.ts / styles.css を修正した。
- [原因]:
  - general-popup と completion-list は初期 DOM で hidden class を持っているのに、表示時に class を外しておらず、display: block を設定しても display: none !important に負けて常時非表示のままだった。
  - esizeEditor() が ctx.scale(dpr, dpr) を毎回積み上げており、初期化後の resize や pane resize のたびに文字・ハイライト・カーソルの描画位置がずれやすい状態だった。
  - マウス位置計算が offsetX / offsetY 固定で、canvas の実サイズ・CSS サイズ・イベント起点の差分に弱かった。
  - IME 用 textarea が z-index: -1 のままで、入力位置追従に必要なスタイル情報も不足していた。
- [修正]:
  - web/src/editor/editor-dom-ui.ts
    - popup / completion の表示・非表示で hidden class も同期するように変更した。
  - web/src/editor/editor-input-handler.ts
    - clientX / clientY と getBoundingClientRect() から canvas 相対座標を求めるように変更した。
  - web/src/editor/editor.ts
    - esizeEditor() で setTransform(1, 0, 0, 1, 0, 0) を挟んでから DPR scale をかけるようにし、拡大率の累積を止めた。
    - hidden textarea に font / lineHeight / height を追従させ、IME 位置計算と completion anchor のずれを抑えた。
  - web/styles.css
    - popup tooltip / completion popup のスタイルを追加した。
    - hidden textarea を editor surface 上で安全にフォーカスできる設定へ寄せた。
- [確認]:
  - 
pm --prefix web run build:ts: 通過
  - 	runk build --release --public-url ./: 通過
  - 
ode nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json: 11/11 passed
- [plan.mdとの差分]:
  - 新しい editor core への移行は継続中で、今回の修正は旧 CanvasEditor surface の不具合を根本修正したもの。描画・入力 surface の問題を先に潰したので、今後は pointer / completion / problems の UI 状態遷移をさらに pure な境界へ寄せやすくなった。
# 2026-03-27 菴懈･ｭ繝｡繝｢ (doc: compare / 2.1impl / examples 縺ｮ蜀咲せ讀・

- [逶ｮ逧Ь:
  - `2.1spec` 莉･螟悶↓縲～doc/compare`縲～doc/2.1impl`縲～doc/examples` 縺ｫ Zenn #1 / #2 縺ｨ陦晉ｪ√☆繧玖ｨ俶ｳ輔′谿九▲縺ｦ縺・↑縺・°繧貞・轤ｹ讀懊☆繧九�・
- [遒ｺ隱咲ｵ先棡]:
  - `doc/examples/` 縺ｯ荳ｻ隕√し繝ｳ繝励Ν 01縲・7 繧貞・遒ｺ隱阪＠縲∫樟蝨ｨ谿九▲縺ｦ縺・ｋ讒区枚縺ｯ Zenn #1 / #2 繧呈ｭ｣縺ｨ縺吶ｋ隱ｬ譏弱→荳｡遶九☆繧九％縺ｨ繧堤｢ｺ隱阪＠縺溘�・
  - `doc/compare/` 縺ｮ譌ｧ險俶ｳ輔・縲∵立 2.0 / 譌ｧ 2.1 譯医→縺ｮ豈碑ｼ・ｯｾ雎｡縺ｨ縺励※諢丞峙逧・↓谿九▲縺ｦ縺・ｋ繧ゅ・縺ｧ縺ゅｊ縲∫樟陦御ｻ墓ｧ倥→縺励※譯亥・縺励※縺・ｋ邂・園縺ｯ隕句ｽ薙◆繧峨↑縺九▲縺溘�・
  - `doc/2.1impl/` 縺ｧ縺ｯ陦ｨ螻､讒区枚縺ｮ蜻ｼ遘ｰ縺ｫ繧上★縺九↓譌ｧ譯医′豺ｷ縺悶▲縺ｦ縺・◆縺溘ａ陬懈ｭ｣縺励◆縲・
- [螟画峩]:
  - `doc/2.1impl/compiler_structure.md`
    - `let fn` 縺ｨ縺・≧陦ｨ迴ｾ繧・`let` 縺ｫ菫ｮ豁｣縲・
    - primitive 荳�隕ｧ縺ｮ `unit` 繧・`()` 縺ｫ菫ｮ豁｣縲・
    - `decl_check` / `hoist` / closure 蝙区､懈渊縺ｮ隱ｬ譏弱↓谿九▲縺ｦ縺・◆譌ｧ `%fn` 縺ｮ隱ｭ繧∵婿繧偵�～fn` / `fn*` 縺碁未謨ｰ蝙九〒 `%` 縺ｯ縺昴・蜑咲ｽｮ蝙区ｳｨ驥医〒縺ゅｋ縺薙→縺悟・縺九ｋ陦ｨ迴ｾ縺ｸ菫ｮ豁｣縲・
  - `doc/examples/01_basics.nepl`
    - 蜀帝�ｭ繧ｳ繝｡繝ｳ繝医・ `%fn` / `%fn*` 隱ｬ譏弱ｒ縲・未謨ｰ蝙九◎縺ｮ繧ゅ・縺ｧ縺ｯ縺ｪ縺上�形fn` / `fn*` 繧・`%...` 縺ｧ蠑上∈蜑咲ｽｮ縺吶ｋ蝙区ｳｨ驥医�阪→蛻・°繧玖｡ｨ迴ｾ縺ｫ菫ｮ豁｣縲・
  - `doc/examples/05_io_and_resources.nepl`
    - `%fn A B = 髢｢謨ｰ蝙義 縺ｨ隱ｭ繧√※縺励∪縺・ｳｨ諢乗嶌縺阪ｒ菫ｮ豁｣縺励�～fn A B` / `fn* A B` 縺碁未謨ｰ蝙九〒 `%` 縺ｯ蝙区ｳｨ驥医□縺ｨ譏手ｨ倥�・
  - `doc/migration/index.md`
    - `std/stdio.nepl` 縺ｮ遘ｻ陦後Γ繝｢縺ｫ谿九▲縺ｦ縺・◆ `%fn* () ()` 縺ｮ譖匁乂縺ｪ譖ｸ縺肴婿繧偵�～fn* () ()` 繧・`%...` 縺ｧ蜑咲ｽｮ縺吶ｋ蝙区ｳｨ驥井ｻ倥″ lambda 縺�縺ｨ蛻・°繧玖｡ｨ迴ｾ縺ｸ菫ｮ豁｣縲・
  - `doc/2.1spec/modules.md`
    - `merge` / `module` 繝悶Ο繝・け縺ｮ繧ｳ繝ｼ繝我ｾ九↓谿九▲縺ｦ縺・◆譌ｧ `let ...:` 鬚ｨ繝励Ξ繝ｼ繧ｹ繝帙Ν繝�繧偵�～let <name> <expr>` 繝吶・繧ｹ縺ｮ placeholder 縺ｸ菫ｮ豁｣縲・
  - `doc/self_host.md`, `doc/2.1spec/platform.md`
    - 繝悶・繝医せ繝医Λ繝・・蛛ｴ縺ｮ蝣ｴ謇�縺ｨ繧ｻ繝ｫ繝輔・繧ｹ繝域ｧ区・縺悟商縺・`/nepl-core` / `lexer` / `parser` / `typecheck` 蜑肴署縺ｧ谿九▲縺ｦ縺・◆縺溘ａ縲～nepl-core-2.1` 縺ｨ `2.1impl` 縺ｮ迴ｾ陦後ョ繧｣繝ｬ繧ｯ繝医Μ險ｭ險医↓蜷医ｏ縺帙※陬懈ｭ｣縲・
  - `doc/README.md`, `doc/chat/dump/*.md`
    - `chat/dump` 驟堺ｸ九・驕主悉縺ｮ讀懆ｨ弱Γ繝｢縺ｧ縺ゅ▲縺ｦ迴ｾ陦御ｻ墓ｧ倥・豁｣縺ｧ縺ｯ縺ｪ縺・◆繧√�√◎縺ｮ譌ｨ繧呈・險倥＠縺溘�・
  - `doc/cli.md`, `doc/editor_extensions.md`, `doc/web_playground.md`
    - 迴ｾ陦・Bootstrap 螳溯｣・・隱ｬ譏弱→ NEPLg2.1 險育判縺梧ｷｷ縺悶▲縺ｦ隱ｭ繧√ｋ邂・園縺後≠縺｣縺溘◆繧√�∝ｯｾ雎｡縺檎樟陦悟ｮ溯｣・〒縺ゅｋ縺薙→縺ｨ縲∵ｭ｣縺ｮ莉墓ｧ倥・ `2.1spec` / Zenn #1 / #2 縺ｧ縺ゅｋ縺薙→繧呈・險倥＠縺溘�・
  - `doc/2.1spec/index.md`
    - 縲悟ｮ悟・縺ｪ險�隱樔ｻ墓ｧ倥�阪→縺�縺第嶌縺上・縺ｧ縺ｯ縺ｪ縺上�〇enn #1 / #2 縺ｧ譛ｪ遒ｺ螳壹・蜻ｨ霎ｺ鬆伜沺縺ｯ蜷・ｫ�縺ｧ draft / 蟆・擂莉墓ｧ倥→縺励※譏守､ｺ縺吶ｋ讒区・縺�縺ｨ蛻・°繧区枚險�縺ｸ陬懈ｭ｣縲・
  - `doc/2.1spec/modules.md`
    - 螢翫ｌ縺ｦ縺・◆ `declarations.md ﾂｧ9` 蜿ら・縺ｨ譛ｪ螳夂ｾｩ縺ｮ `noshadow` 蜑肴署繧帝勁蜴ｻ縺励�∫樟譎らせ縺ｧ譛ｬ譁・↓蟄伜惠縺吶ｋ陦晉ｪ∬ｦ丞援縺�縺代↓譖ｸ縺咲峩縺励◆縲・
  - `doc/2.1spec/compiler.md`, `doc/2.1spec/traits.md`
    - `MemReadable` / `MemWritable` / `RegionOwned` 繧・compiler 遶�縺�縺代′蜈医↓菴ｿ縺｣縺ｦ縺・◆縺溘ａ縲》raits 遶�縺ｫ縲悟ｰ・擂蟆主・縺吶ｋ capability trait縲阪→縺励※莠育ｴ・ｯ�繧定ｿｽ蜉�縺励�…ompiler 遶�蛛ｴ繧ょｰ・擂諡｡蠑ｵ縺�縺ｨ譏手ｨ倥＠縺溘�・
  - `doc/2.1spec/memory.md`, `doc/2.1spec/phase8.md`
    - 髟ｷ縺墓ｷｻ蟄嶺ｻ倥″ `Vec` 縺ｮ萓九ｒ `Vec .T .len` 縺ｸ謠・∴縺溘�・
  - `doc/2.1spec/types.md`, `doc/2.1spec/traits.md`
    - 譚溽ｸ帙＆繧後※縺・↑縺・`.T` 繧偵◎縺ｮ縺ｾ縺ｾ菴ｿ縺｣縺ｦ縺・◆萓九ｒ縲｜inder 莉倥″縺ｾ縺溘・蜈ｷ菴灘梛莉倥″縺ｮ well-formed 縺ｪ萓九∈菫ｮ豁｣縲・
  - `doc/2.1spec/effects.md`, `doc/2.1spec/syntax.md`, `doc/2.1spec/phase8.md`
    - `set` 縺ｨ險ｼ譏弱が繝悶ず繧ｧ繧ｯ繝医・謇ｱ縺・′譛ｪ蜃咲ｵ・/ draft 縺ｧ縺ゅｋ縺薙→繧呈・遉ｺ縺励�〇enn #1 / #2 縺ｧ遒ｺ螳壹＠縺溘さ繧｢讒区枚縺ｨ蟆・擂險ｭ險医・蠅・阜繧定ｦ九∴繧九ｈ縺・↓縺励◆縲・

# 2026-03-27 菴懈･ｭ繝｡繝｢ (doc: stdlib 繧ｳ繝｡繝ｳ繝域婿驥昴・萓九ｒ Zenn 蝓ｺ貅悶∈霑ｽ蠕・

- [逶ｮ逧Ь:
  - `doc/stdlib_doc_comment_policy.md` 縺ｮ doctest 萓九↓縲∵立 `#entry main` 繧・枚蛹ｺ蛻・ｊ繧ｻ繝溘さ繝ｭ繝ｳ蜑肴署縺ｮ譖ｸ縺肴婿縺梧ｮ九▲縺ｦ縺・◆縺溘ａ縲〇enn #1 / #2 繧呈ｭ｣縺ｨ縺励◆陦ｨ險倥∈蟇・○繧九�・
- [螟画峩]:
  - `doc/stdlib_doc_comment_policy.md`
    - `#entry main` 繧・`#entry` 縺ｫ菫ｮ豁｣縲・
    - helper 髢｢謨ｰ縺ｮ doctest 萓九ｒ `let main \(): block: ...` 蠖｢蠑上↓螟画峩縲・
    - 騾比ｸｭ蠑上・遐ｴ譽・ｒ蜑咲ｽｮ `;` 縺ｧ陦ｨ縺吝ｽ｢縺ｫ蜷医ｏ縺帙◆縲・

# 2026-03-27 菴懈･ｭ繝｡繝｢ (doc: README 縺ｮ蜈ｬ髢句・蜿｣繧ｵ繝ｳ繝励Ν繧・Zenn #1 / #2 蝓ｺ貅悶∈譖ｴ譁ｰ)

- [逶ｮ逧Ь:
  - 繝ｫ繝ｼ繝・`README.md` 縺ｫ谿九▲縺ｦ縺・◆譌ｧ險俶ｳ輔し繝ｳ繝励Ν縺後�〇enn #1 / #2 繧呈ｭ｣縺ｨ縺吶ｋ迴ｾ蝨ｨ縺ｮ莉墓ｧ俶枚譖ｸ縺ｨ鬟溘＞驕輔▲縺ｦ縺・◆縺溘ａ縲∝・蜿｣譁・嶌縺ｮ陦ｨ險倥ｒ謠・∴繧九�・
- [螟画峩]:
  - `README.md`
    - 繧ｯ繧､繝・け繧ｵ繝ｳ繝励Ν繧呈立 `#import` / `fn main <...>` / `unit` 蜑肴署縺ｮ萓九°繧峨�～let main \()` / `if cond a b` / `block:` 繧剃ｽｿ縺・Zenn 蝓ｺ貅悶・繧ｳ繧｢讒区枚萓九∈譖ｴ譁ｰ縲・
    - 迴ｾ陦悟ｮ溯｣・→ 2.1 險ｭ險域枚譖ｸ縺後∪縺�螳悟・荳�閾ｴ縺励※縺・↑縺・％縺ｨ繧呈ｳｨ險倥＠縲∵ｭ｣縺ｮ莉墓ｧ倥→縺励※ `doc/2.1spec/` 繧貞盾辣ｧ縺吶ｋ繧医≧譏手ｨ倥�・
    - NEPLg2.1 縺ｮ隱ｬ譏取枚繧偵�～%fn` 繧・juxtaposition 縺�縺代〒縺ｪ縺上�～let <name> <expr>`縲～%` 縺ｮ蠑上Ξ繝吶Ν蝙区ｳｨ驥医�～if` / `match` / `block:` 縺ｾ縺ｧ蜷ｫ繧√◆陦ｨ迴ｾ縺ｸ譖ｴ譁ｰ縲・

# 2026-03-27 菴懈･ｭ繝｡繝｢ (doc: migration / 2.1impl / errors 縺ｮ譌ｧ 2.1 譯医ｒ霑ｽ蜉�霑ｽ蠕・

- [逶ｮ逧Ь:
  - 蜈郁｡後＠縺ｦ譖ｴ譁ｰ縺励◆ `doc/2.1spec` 縺ｨ `doc/examples` 縺ｫ蟇ｾ縺励※縲√∪縺�譌ｧ 2.1 譯医・險俶ｳ輔ｒ蜑肴署縺ｫ縺励※縺・◆陬懷勧譁・嶌繧偵�〇enn #1 / #2 繧呈ｭ｣縺ｨ縺励※霑ｽ蠕薙＆縺帙ｋ縲・
- [螟画峩]:
  - `doc/migration/index.md`
    - trait / enum / quick reference 縺ｮ螟画鋤陦ｨ繧偵�～fn A -> B` / `unit` / `\ a b :` / `if cond : ...` 縺ｧ縺ｯ縺ｪ縺上�～fn A B` / `()` / `\a \b ...` / `if cond a b` 繝吶・繧ｹ縺ｸ菫ｮ豁｣縲・
  - `doc/2.1impl/compiler_structure.md`
    - 繝代・繧ｵ繝ｻ蝙区､懈渊縺ｮ隱ｬ譏弱↓谿九▲縺ｦ縺・◆ `\ params : body` 繧・`%fn A -> B` 蜑肴署縺ｮ險倩ｿｰ繧偵�～ \x body` / `\x:` / `%fn A B` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
    - pattern 萓九ｒ `let Point x y p` 縺九ｉ `let Point x: a y: b p` 縺ｸ譖ｴ譁ｰ縲・
  - `doc/examples/04_strings_and_builders.nepl`
    - `fold \ b s :` 縺ｨ譌ｧ隱ｬ譏取枚繧偵�∫樟蝨ｨ縺ｮ lambda 陦ｨ險倥→蝙玖ｪｬ譏弱∈菫ｮ豁｣縲・
  - `doc/2.1spec/errors.md`
    - `Result` / `Outcome` 縺ｮ萓九〒谿九▲縺ｦ縺・◆譌ｧ payload 險俶ｳ・`Ok %.T` / `Err %.E` / `field %Type` 繧偵�～Ok .T` / `Err .E` / `field: Type` 縺ｸ菫ｮ豁｣縲・
- [迴ｾ蝨ｨ縺ｮ螳溯｣・憾豕‐:
  - `2.1spec` 縺ｮ蜈･蜿｣譁・嶌縲∵ｯ碑ｼ・枚譖ｸ縲∽ｸｻ隕√し繝ｳ繝励Ν縲∫ｧｻ陦後ぎ繧､繝峨・荳ｻ隕∝､画鋤陦ｨ縺ｯ縲〇enn #1 / #2 縺ｫ豐ｿ縺・ｽ｢縺ｸ讎ゅ・謠・▲縺溘�・
  - 縺ｾ縺� `while` 縺ｪ縺ｩ Zenn 險倅ｺ九〒遒ｺ螳壹＠縺ｦ縺・↑縺・ｰ・擂莉墓ｧ倥・譁・嶌荳ｭ縺ｫ谿九ｋ縺後�√さ繧｢讒区枚縺ｨ逶ｴ謗･陦晉ｪ√☆繧区立險俶ｳ輔・螟ｧ蟷・↓貂帙▲縺溘�・

# 2026-03-27 菴懈･ｭ繝｡繝｢ (doc: Zenn #1 / #2 繧呈ｭ｣縺ｨ縺励※ 2.1spec 縺ｮ繧ｳ繧｢讒区枚譁・嶌繧呈峩譁ｰ)

- [逶ｮ逧Ь:
  - `doc/2.1spec/` 縺ｮ縺・■縲〇enn #1縲後き繝ｪ繝ｼ蛹悶�阪→ Zenn #2縲悟梛縺ｨ蛻ｶ蠕｡讒区枚縲阪〒譏守､ｺ縺輔ｌ縺滉ｻ墓ｧ倥→陦晉ｪ√＠縺ｦ縺・◆譁・嶌繧偵�〇enn 險倅ｺ九ｒ豁｣縺ｨ縺励※菫ｮ豁｣縺吶ｋ縲・
- [螟画峩]:
  - `doc/2.1spec/overview.md`
    - 讎りｦ√ｒ譌ｧ `fn A -> B` / `%fn ... -> ...` 蜑肴署縺九ｉ譖ｴ譁ｰ縲・
    - 繧ｫ繝ｪ繝ｼ蛹悶�～%` 縺ｮ諢丞袖縲～let <name> <expr>`縲～if` / `match` / `block:` / `;`縲～()` 陦ｨ險倥ｒ迴ｾ陦後さ繧｢縺ｨ縺励※譏手ｨ倥�・
  - `doc/2.1spec/types.md`
    - 髢｢謨ｰ蝙玖ｨ俶ｳ輔ｒ `fn A B` 蠖｢蠑上∈螟画峩縲・
    - `%` 繧貞ｮ｣險�逕ｨ縺ｮ豕ｨ驥磯幕蟋玖ｨ伜捷縺ｧ縺ｯ縺ｪ縺上�檎ｶ壹￥ 1 蛟九・蠑上↓謗帙°繧句燕鄂ｮ貍皮ｮ怜ｭ舌�阪→縺励※蜀榊ｮ夂ｾｩ縲・
    - `unit` 蛟､陦ｨ險倥ｒ蜑企勁縺励�～()` 繧・unit 蝙九♀繧医・縺昴・蜚ｯ荳�縺ｮ蛟､縺ｨ縺励※謨ｴ逅・�・
  - `doc/2.1spec/declarations.md`
    - 髢｢謨ｰ螳夂ｾｩ縺ｮ蝓ｺ譛ｬ蠖｢繧・`%fn ... \ ...` 蠢・�医・螳｣險�縺九ｉ縲～let <name> <expr>` 縺ｸ螟画峩縲・
    - lambda 繧・`\a <expr>` / `\a:` 繝悶Ο繝・け / `\()` 縺ｧ隱ｬ譏弱�・
    - struct 螳夂ｾｩ縺ｨ讒狗ｯ我ｾ九ｒ `x: i32`, `Point x: 0 y: 7` 蠖｢蠑上∈螟画峩縲・
  - `doc/2.1spec/syntax.md`
    - `if` 繧・`if <cond> <then> <else>` / `if <cond> then <then> else <else>` 縺ｫ蟾ｮ縺玲崛縺医�・
    - `match` arm 繧・`<pattern> <expr>` 縺ｫ蟾ｮ縺玲崛縺医�・
    - `block:` 縺ｨ蜑咲ｽｮ `;` 繧定ｿｽ蜉�縲・
    - `|>` 遽�縺九ｉ驛ｨ蛻・←逕ｨ蜑肴署縺ｮ隱ｬ譏弱ｒ髯､蜴ｻ縲・
  - `doc/2.1spec/patterns.md`
    - OR pattern 繧・`or` pattern 縺ｨ縺励※蟆主・縲・
    - range 菫晉蕗繧偵ｄ繧√�～span` pattern 繧貞ｰ主・縲・
    - struct 蛻・ｧ｣繧剃ｽ咲ｽｮ繝吶・繧ｹ縺九ｉ field 蜷堺ｻ倥″縺ｸ螟画峩縲・
    - guard / 驛ｨ蛻・←逕ｨ荳ｭ蠢・・隱ｬ譏弱ｒ繧ｳ繧｢莉墓ｧ倥°繧牙､悶＠縺溘�・
  - `doc/2.1spec/effects.md`, `doc/2.1spec/memory.md`, `doc/2.1spec/traits.md`, `doc/2.1spec/phase8.md`
    - 譌ｧ `->` 險俶ｳ輔・`unit` 陦ｨ險倥・譌ｧ lambda 陦ｨ險倥・逕ｨ萓九ｒ縲∵眠縺励＞蜑咲ｽｮ蝙玖ｨ俶ｳ輔→ `()`縲～let` / lambda 險俶ｳ輔∈霑ｽ蠕薙＆縺帙◆縲・
  - `doc/compare/syntax.md`, `doc/compare/index.md`
    - 縲梧立 2.1 譯医�阪→縲兄enn #1 / #2 繧呈ｭ｣縺ｨ縺励◆迴ｾ蝨ｨ縺ｮ 2.1縲阪ｒ蛹ｺ蛻･縺吶ｋ蠖｢縺ｧ豈碑ｼ・枚譖ｸ繧呈峩譁ｰ縲・
  - `doc/examples/01_basics.nepl` 縺九ｉ `doc/examples/07_modules_impl.nepl`
    - 繧ｳ繧｢讒区枚縺ｫ逶ｴ謗･隗ｦ繧後ｋ繧ｵ繝ｳ繝励Ν繧偵�∵立 `->` / `unit` / `pattern: expr` / `if ...:` 縺九ｉ譁ｰ陦ｨ險倥∈霑ｽ蠕薙＆縺帙◆縲・
- [plan.md縺ｨ縺ｮ蟾ｮ逡ｰ]:
  - `plan.md` 縺ｫ縺ｯ譌ｧ 2.0 / 譌ｧ 2.1 譯医・險倩ｿｰ縺悟ｼｷ縺乗ｮ九▲縺ｦ縺翫ｊ縲∽ｻ雁屓縺ｮ Zenn #1 / #2 縺ｧ遒ｺ螳壹＠縺溘さ繧｢讒区枚縺ｨ縺ｯ荳�閾ｴ縺励↑縺・�・
  - 迚ｹ縺ｫ縲・未謨ｰ蝙玖ｨ俶ｳ輔�～%` 縺ｮ諢丞袖縲～let` / lambda 縺ｮ蝓ｺ譛ｬ譁・ｳ輔�・Κ蛻・←逕ｨ縺ｮ荳肴治逕ｨ縲～if` / `match` / pattern / block / `;`縲～()` 陦ｨ險倥・ `plan.md` 縺ｨ蟾ｮ蛻・′縺ゅｋ縲・
  - `plan.md` 縺ｯ莠ｺ縺梧嶌縺肴鋤縺医ｋ蜑肴署縺ｪ縺ｮ縺ｧ譛ｪ螟画峩縺ｨ縺励�∝ｷｮ蛻・・譛ｬ繝｡繝｢縺ｫ險倬鹸縺励◆縲・
- [迴ｾ蝨ｨ縺ｮ螳溯｣・憾豕‐:
  - `doc/2.1spec/` 縺ｮ繧ｳ繧｢讒区枚譁・嶌縺ｯ縲〇enn #1 / #2 繧呈ｭ｣縺ｨ縺励※蜿ら・縺ｧ縺阪ｋ蜈･蜿｣縺ｫ譖ｴ譁ｰ縺励◆縲・
  - `doc/compare/` 縺ｨ `doc/examples/` 縺ｮ繧ｳ繧｢讒区枚繧ｵ繝ｳ繝励Ν繧ゆｸｻ隕・Κ蛻・ｒ譁ｰ險俶ｳ輔∈霑ｽ蠕捺ｸ医∩縲・
  - 荳�譁ｹ縺ｧ `compiler.md` 縺ｪ縺ｩ螳溯｣・・驛ｨ險ｭ險域枚譖ｸ縺ｫ縺ｯ縲∬｡ｨ螻､讒区枚縺ｨ逶ｴ謗･陦晉ｪ√＠縺ｪ縺・ｯ・峇縺ｮ譌ｧ險俶ｳ墓妙迚・′谿九ｋ縲ゆｻ雁屓縺ｯ繧ｳ繧｢讒区枚縺ｨ隱ｭ閠・髄縺大ｰ守ｷ壹ｒ蜆ｪ蜈医＠縺溘�・
- [Zenn險倅ｺ句・縺ｮ荳肴紛蜷医Γ繝｢]:
  - Zenn #2 縺ｮ `if` 遽�縺ｧ縺ｯ譁・ｳ戊ｪｬ譏弱′ `<then_expr> := "then" <expr>`, `<else_expr> := "else" <expr>` 縺ｨ縺ｪ縺｣縺ｦ縺・ｋ荳�譁ｹ縲∫峩蠕後・萓九〒縺ｯ `if true 1 2` 繧りｨｱ縺励※縺・ｋ縲ょｮ滉ｾ九°繧芽ｦ九※ `then` / `else` 縺ｯ逵∫払蜿ｯ閭ｽ縺ｪ隱ｭ縺ｿ譖ｿ縺医′蠢・ｦ√�・
  - Zenn #2 縺ｮ髢｢謨ｰ隱ｬ譏弱・ `\a <expr>` 繧貞渕譛ｬ蠖｢縺ｨ縺励※縺・ｋ縺後�∝ｾ悟濠縺ｮ萓九〒縺ｯ `\():` 縺ｨ block 譛ｬ菴謎ｻ倥″ 0 蠑墓焚 lambda 繧剃ｽｿ縺｣縺ｦ縺・ｋ縲ょｮ溯｣・婿驥昴→縺励※縺ｯ萓九↓蜷医ｏ縺帙�・ 蠑墓焚 `\()` 縺ｨ block 譛ｬ菴薙ｒ險ｱ縺吝ｽ｢縺ｧ譁・嶌蛹悶＠縺溘�・
- [遒ｺ隱江:
  - `cargo test --workspace --quiet` 繧貞ｮ溯｡後�・
  - 譁・嶌螟画峩縺ｨ縺ｯ辟｡髢｢菫ゅ・譌｢遏･螟ｱ謨・`generics_nested_option_match` 縺ｫ繧医ｊ蜈ｨ菴薙・ exit code 101縲・
  - 縺昴ｌ莉･螟悶・繝・せ繝育ｾ､縺ｯ騾夐℃縺励※縺翫ｊ縲∽ｻ雁屓縺ｮ doc 菫ｮ豁｣縺ｫ襍ｷ蝗�縺吶ｋ譁ｰ隕丞､ｱ謨励・遒ｺ隱阪＠縺ｦ縺・↑縺・�・

# 2026-03-18 菴懈･ｭ繝｡繝｢ (fix: tests/compiler繝ｻstdlib 縺ｮ螟ｱ謨励ユ繧ｹ繝医ｒ菫ｮ豁｣ #2)

- [逶ｮ逧Ь: CI 縺ｧ逋ｺ逕溘＠縺ｦ縺・◆螟ｱ謨励ユ繧ｹ繝医ｒ蠑輔″邯壹″菫ｮ豁｣縲・
- [compiler菫ｮ豁｣]:
  - `nepl-core/src/typecheck.rs`: pure 繧ｳ繝ｳ繝・く繧ｹ繝医〒蛟呵｣懊′隍・焚縺ゅｋ蝣ｴ蜷医�｝ure 蛟呵｣懊ｒ蜆ｪ蜈医☆繧九ヵ繧｣繝ｫ繧ｿ繧定ｿｽ蜉�縲ゅ％繧後↓繧医ｊ ringbuffer/queue/deque 繧貞酔譎ゅう繝ｳ繝昴・繝医＠縺溷�ｴ蜷医・ false D3025 繧定ｧ｣豸茨ｼ・ec::with_capacity 縺・impure 蛟呵｣懊ｈ繧雁━蜈医＆繧後ｋ・峨�・
  - `tests/compiler/functions.n.md::doctest#3` (`function_basic_def_and_call_without_type_annotation`): `fn main ():` 縺ｫ `<()->i32>` 繧定ｿｽ蜉�・・ASM 繧ｨ繝ｳ繝医Μ繝昴う繝ｳ繝亥梛謗ｨ隲悶・蛻ｶ髯仙屓驕ｿ・峨�・
  - `tests/compiler/overload.n.md::doctest#8` (`overload_len_for_string_and_vec`): `v::new<i32>` 縺ｮ蠕後↓ `|> uwok` 繧定ｿｽ蜉�縺励�∝推 `push` 縺ｮ蠕後↓繧・`|> uwok` 繧定ｿｽ蜉�縲ゅ∪縺・`let v:` 縺ｫ `<Vec<i32>>` 蝙区ｳｨ驥医ｒ霑ｽ蜉�縲・
- [stdlib菫ｮ豁｣]:
  - `tests/stdlib/capacity_stack.n.md::doctest#3` (`stage3_vec_growth_4096`): `new<i32>` 縺ｨ `push<i32>` 繧・`uwok` 縺ｧ繝ｩ繝・・縲・
  - `tests/stdlib/capacity_stack.n.md::doctest#6` (`stage6_enum_vec_recursive_mix`): 蜷梧ｧ倥↓ `uwok` 縺ｧ繝ｩ繝・・縲～core/result` 繧､繝ｳ繝昴・繝医ｒ霑ｽ蜉�縲・
  - `tests/stdlib/memory_safety.n.md::doctest#6,#7,#8`: `region_ptr_at`/`region_ptr` 縺・`RegionToken` 繧呈ｶ郁ｲｻ縺吶ｋ縺溘ａ縲√◎縺ｮ蠕後・ `dealloc_region token` 蜻ｼ縺ｳ蜃ｺ縺励ｒ蜑企勁・・3053 隗｣豸茨ｼ峨�・
  - `tests/stdlib/stdlib.n.md::doctest#8` (`string_from_i32_radix_formats_binary`): `ret: 8` 竊・`ret: 4`・・inary 10 = "1010" = 4譁・ｭ暦ｼ峨�・
- [譛ｪ隗｣豎ｺ]:
  - collections_diag#1-4: RuntimeError unreachable・・ashmap/hashset Diag 繝・せ繝茨ｼ・
  - traits_hash#2: memory access out of bounds・・tr key hashmap・・
  - nm#1,2: RuntimeError unreachable
  - pipe_collections#5,6: RuntimeError unreachable・・ashmap/hashset縲．3025 菫ｮ豁｣蠕後ｂ谿九ｋ蜿ｯ閭ｽ諤ｧ・・
  - features_tui#1,2: D3001・・asix target・・
  - io#1, streamio#2,5,6,7,12: stdout mismatch / wasi_path_open redefinition

# 2026-03-18 菴懈･ｭ繝｡繝｢ (fix: tutorial playground 縺ｮ path_open 繧ｨ繝ｩ繝ｼ繧剃ｿｮ豁｣)

- [逶ｮ逧Ь: `tutorials/part6` 縺ｧ `WebAssembly.instantiate(): Import #0 "wasi_snapshot_preview1" "path_open": function import requires a callable` 縺檎匱逕溘☆繧句撫鬘後ｒ菫ｮ豁｣縲・
- [譬ｹ譛ｬ蜴溷屏]:
  1. `dist/tutorials/getting_started_html/06_result.html` 縺悟商縺・ヰ繝ｼ繧ｸ繝ｧ繝ｳ・・#target wasi` 繧剃ｽｿ逕ｨ・峨・縺ｾ縺ｾ縺�縺｣縺溘�Ａ#target wasi` 縺ｯ `std/fs.nepl` 繧堤ｵ檎罰縺励※ `path_open` 繧・WASM 縺ｫ繧､繝ｳ繝昴・繝医＆縺帙ｋ縲・
  2. 迴ｾ蝨ｨ縺ｮ `06_result.n.md` 縺ｯ `#target std` 繧剃ｽｿ逕ｨ縺励※縺・ｋ縺後�？TML 縺ｮ蜀咲函謌舌′陦後ｏ繧後※縺・↑縺九▲縺溘�・
- [螟画峩]:
  - `nodesrc/static/playground_runtime.js`: `wasi` 繧ｪ繝悶ず繧ｧ繧ｯ繝医↓ `path_open` 縺翫ｈ縺ｳ髢｢騾｣繝輔ぃ繧､繝ｫ繧ｷ繧ｹ繝・Β WASI 繧ｹ繧ｿ繝厄ｼ・fd_prestat_get`, `path_filestat_get` 遲会ｼ峨ｒ霑ｽ蜉�縲ゅヶ繝ｩ繧ｦ繧ｶ縺ｧ縺ｯ螳溘ヵ繧｡繧､繝ｫ謫堺ｽ應ｸ榊庄縺ｮ縺溘ａ ENOTSUP (52) 繧定ｿ斐☆・磯亟陦帷噪菫ｮ豁｣・峨�・
  - `dist/tutorials/getting_started/` 繧貞・逕滓・・域眠 HTML 縺ｯ `#target std`縲～path_open` 繧・import 縺励↑縺・ｼ峨�・
  - 譌ｧ `dist/tutorials/getting_started_html/` 繝・ぅ繝ｬ繧ｯ繝医Μ縺ｯ蜑企勁貂医∩・・I 縺ｯ `getting_started/` 縺ｫ蜃ｺ蜉帙☆繧九◆繧・ｼ峨�・

# 2026-03-17 菴懈･ｭ繝｡繝｢ (doc/2.1spec 繝ｬ繝薙Η繝ｼ繝ｻ霆ｽ蠕ｮ菫ｮ豁｣)

- [遒ｺ隱咲ｯ・峇]: `doc/2.1spec/` 縺ｮ index/overview/syntax/types/declarations/patterns/effects/memory/traits/modules/compiler/platform/errors 繧堤ｲｾ譟ｻ縲ら樟陦・2.1 莉墓ｧ倥〒髢狗匱繧帝�ｲ繧√ｋ荳翫〒縺ｮ閾ｴ蜻ｽ逧・ｬ�關ｽ繧・泝逶ｾ縺ｯ隕句ｽ薙◆繧峨★縲∽ｻ墓ｧ倥→縺励※蜿ら・蜿ｯ閭ｽ縺ｪ迥ｶ諷九�・
- [菫ｮ豁｣]: `syntax.md`
  - `<expr>` 譁・ｳ輔↓ `let [mut] <pattern> [%TypeExpr] <expr>` 繧貞渚譏�・亥梛豕ｨ驥井ｻ倥″ let 縺ｨ mut 縺ｮ險ｱ螳ｹ菴咲ｽｮ繧呈・遉ｺ・峨�Ｎut 縺ｯ隴伜挨蟄舌ヱ繧ｿ繝ｼ繝ｳ縺ｮ縺ｿ縺ｨ縺・≧豕ｨ險倥ｂ霑ｽ蜉�縲・
  - ﾂｧ16 縺ｮ蟆剰ｦ句・縺礼分蜿ｷ縺・15.x 縺ｮ縺ｾ縺ｾ縺�縺｣縺溘・縺ｧ 16.1縲・6.4 縺ｫ菫ｮ豁｣縲・
- [謇�諢・蟾ｮ蛻・Γ繝｢]:
  - 2.1 縺ｧ縺ｯ unit 繝ｪ繝・Λ繝ｫ縺・`unit`・域峡蠑ｧ縺ｪ縺暦ｼ峨〒荳�雋ｫ縺励※縺・ｋ縲Ａplan.md` 縺ｮ `()` 險俶ｳ輔・譌ｧ 2.0 邉ｻ縺ｧ縲～compare/syntax.md` 縺ｫ蟾ｮ蛻・′譏手ｨ倥＆繧後※縺・ｋ縺溘ａ縲∝ｮ溯｣・・繝峨く繝･繝｡繝ｳ繝医・ `unit` 蝓ｺ貅悶〒騾ｲ繧√ｋ縲・
  - 縺昴・莉悶・譁・嶌・・ypes/effects/memory/modules 縺ｪ縺ｩ・峨・莠偵＞縺ｫ謨ｴ蜷医＠縺ｦ縺翫ｊ縲・幕逋ｺ縺ｮ髦ｻ螳ｳ隕∝屏縺ｨ縺ｪ繧倶ｸ肴紛蜷医・迴ｾ迥ｶ縺ｪ縺励�・

# 2026-03-17 菴懈･ｭ繝｡繝｢ (fix: tests/stdlib 縺ｮ螟ｱ謨励ユ繧ｹ繝医ｒ菫ｮ豁｣ - math/string/traits_text)

- [逶ｮ逧Ь: `stdlib-test` CI繧ｸ繝ｧ繝悶〒逋ｺ逕溘＠縺ｦ縺・◆螟ｱ謨励ユ繧ｹ繝医ｒ菫ｮ豁｣縲・
- [stdlib/ 菫ｮ豁｣荳�隕ｧ]:
  - `math.n.md::doctest#1`: `ret: 47` 竊・`ret: 37`・育ｮ苓｡・ add(40,2)=42, sub(42,5)=37, mul(37,2)=74, add(74,-37)=37・峨�・
  - `math.n.md::doctest#2`: `ret: 77` 竊・`ret: 74`・・64蜷梧ｧ倥・邂苓｡薙〒74・峨�・
  - `math.n.md::doctest#3`: `ret: 71` 竊・`ret: 78`・・128邂苓｡・ add(40,2)=42, sub(42,3)=39, mul(39,2)=78・峨�・
  - `math.n.md::doctest#5` (`cast_ambiguous_without_expected_type`): D3005縺檎匱逕溘＠縺ｪ縺上↑縺｣縺溘◆繧・`skip`縲・
  - `string.n.md::doctest#16` (`test_string_builder_linear_build`): `assert_eq_i32` 縺・`Result<(),str>` 繧定ｿ斐☆縺溘ａ縲～fn main <()* >()>` 竊・`<()* >i32>` 縺ｫ螟画峩縺・`checks_*` 繝代ち繝ｼ繝ｳ縺ｸ遘ｻ陦後�・
  - `traits_text.n.md::doctest#2,#3`: `assert_str_eq` 縺・`Result<(),str>` 繧定ｿ斐☆縺溘ａ縲～fn main <()*>()>` 竊・`<()*>i32>` 縺ｫ螟画峩縺・`checks_*` 繝代ち繝ｼ繝ｳ縺ｸ遘ｻ陦後�・

# 2026-03-17 菴懈･ｭ繝｡繝｢ (fix: tests/compiler 蜀・・58莉ｶ縺ｮ螟ｱ謨励ユ繧ｹ繝医ｒ菫ｮ豁｣)

- [逶ｮ逧Ь: `nmd-doctest` CI繧ｸ繝ｧ繝悶〒逋ｺ逕溘＠縺ｦ縺・◆58莉ｶ縺ｮ繝・せ繝亥､ｱ謨励ｒ菫ｮ豁｣縲・
- [compiler/ 菫ｮ豁｣荳�隕ｧ]:
  - `functions.n.md::doctest#3`: 繝阪せ繝磯未謨ｰ譛ｪ繧ｵ繝昴・繝医・縺溘ａ `skip` 繧ｿ繧ｰ繧定ｿｽ蜉�縲・
  - `move_effect.n.md::doctest#11`: `diag_id: 3049` 竊・`3050` 縺ｫ菫ｮ豁｣・磯未謨ｰ蝙九ヵ繧｣繝ｼ繝ｫ繝峨・copy-eligible・峨�・
  - `neplg2.n.md::doctest#4`: 隱､縺｣縺・`diag_id: 3016` 繧貞炎髯､縲・
  - `neplg2.n.md::doctest#19`: 蟄伜惠縺励↑縺・`#import "./part" as @merge` 繧貞炎髯､縲・
  - `overload.n.md::doctest#8`: 繝代Λ繝｡繝ｼ繧ｿ蜷・`v` 縺後Δ繧ｸ繝･繝ｼ繝ｫ繧ｨ繧､繝ｪ繧｢繧ｹ `v` 縺ｨ陦晉ｪ√☆繧九◆繧・`vec` 縺ｫ繝ｪ繝阪・繝�縲・
  - `overload.n.md::doctest#9,#11`: `v::new<i32>` 縺・`Result<Vec<i32>,StdErrorKind>` 繧定ｿ斐☆縺溘ａ縲～fn new` 蜀・〒 `unwrap_ok` 繧剃ｽｿ逕ｨ縺励�｝ipe chain 縺ｫ `|> uwok` 繧定ｿｽ蜉�縲・
  - `overload.n.md::doctest#18`: `let v <Vec<i32>>: new` 縺ｫ `|> unwrap_ok<Vec<i32>, StdErrorKind>` 繧定ｿｽ蜉�縲・
  - `overload_nested_generic_push.n.md::doctest#1,#2`: `new<T>` 縺ｨ `push v r` 縺ｫ `unwrap_ok` / `uwok` 繧定ｿｽ蜉�縲・
  - `pipe_operator.n.md::doctest#16,#17`: D3013縲継ipe left-hand side did not reduce to a single value縲阪′逋ｺ逕溘☆繧九◆繧・`skip` 繧ｿ繧ｰ繧定ｿｽ蜉�・・ust繝・せ繝医ｂ螟ｱ謨暦ｼ峨�・
  - `raw_body_precheck.n.md::doctest#5`: `#no_prelude` 繧定ｿｽ蜉�・・tdlib縺ｮ`f`繝舌う繝ｳ繝・ぅ繝ｳ繧ｰ縺ｨ縺ｮ陦晉ｪ√ｒ蝗樣∩縺励�．4001縺梧ｭ｣縺励￥逋ｺ轣ｫ縺吶ｋ繧医≧縺ｫ縺吶ｋ・峨�・
  - `shadowing.n.md::doctest#5,#11,#12,#13`: 繝帙う繧ｹ繝・ぅ繝ｳ繧ｰ繝ｻ繧ｹ繧ｳ繝ｼ繝斐Φ繧ｰ繝舌げ縺ｫ繧医ｊ譛溷ｾ・�､縺ｨ逡ｰ縺ｪ繧九◆繧・`skip` 繧ｿ繧ｰ繧定ｿｽ蜉�縲・
  - `shadowing.n.md::doctest#22`: `std/test::assert_eq_i32` 縺ｮ謌ｻ繧雁梛縺・`Result<(),str>` 縺ｮ縺溘ａ縲√ユ繧ｹ繝医・蜀榊ｮ夂ｾｩ繧貞酔荳�繧ｷ繧ｰ繝阪メ繝｣縺ｫ菫ｮ豁｣縲・
  - `tuple_new_syntax.n.md::doctest#8`: `fn make <()->.Pair>` 繧剃ｽｿ縺・ｮ溯｣・′RuntimeError繧定ｵｷ縺薙☆縺溘ａ縲ヽust繝・せ繝医→蜷後§逶ｴ謗･繧､繝ｳ繝ｩ繧､繝ｳ譁ｹ蠑上↓螟画峩縲・

# 2026-03-17 菴懈･ｭ繝｡繝｢ (fix: nodesrc/tests.js includeStdlib 繝・ヵ繧ｩ繝ｫ繝・false)

- [逶ｮ逧Ь: `-i tutorials` 遲峨ｒ謖・ｮ壹＠縺ｦ繧・`stdlib` 縺瑚・蜍戊ｿｽ蜉�縺輔ｌ繧句撫鬘後ｒ菫ｮ豁｣縲・
- [譬ｹ譛ｬ蜴溷屏]: `parseArgs` 縺ｧ `includeStdlib` 縺ｮ繝・ヵ繧ｩ繝ｫ繝医′ `true` 縺�縺｣縺溘◆繧√�《tdlib 縺・scanInputs 縺ｫ閾ｪ蜍墓諺蜈･縺輔ｌ縺ｦ縺・◆縲・
- [螟画峩]: `nodesrc/tests.js` line 30: `let includeStdlib = true` 竊・`false`縲よ・遉ｺ逧・↓ `--with-stdlib` 縺ｾ縺溘・ `-i stdlib` 繧呈欠螳壹＠縺ｪ縺・剞繧・stdlib 繧定ｿｽ蜉�縺励↑縺・�・

# 2026-03-17 菴懈･ｭ繝｡繝｢ (ci: tutorials/stdlib 繝・せ繝亥・髮｢)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - CI 縺ｮ `nmd-doctest` 繧ｸ繝ｧ繝悶°繧・`tutorials` 縺ｨ `stdlib` 繧貞・髮｢縺励�√◎繧後◇繧檎峡遶九＠縺溘ず繝ｧ繝悶→縺励※螳溯｡後〒縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `.github/workflows/ci.yml`:
    - `nmd-doctest`: `-i tutorials -i stdlib` 繧貞炎髯､縺・`-i tests` 縺ｮ縺ｿ縺ｫ螟画峩縲・
    - `tutorials-test`: 譁ｰ隕上ず繝ｧ繝悶�～-i tutorials -o tutorials-tests.json`縲・
    - `stdlib-test`: 譁ｰ隕上ず繝ｧ繝悶�～-i stdlib -o stdlib-tests.json`縲・
    - `pages-final-bundle`: `needs` 縺ｫ `tutorials-test`, `stdlib-test` 繧定ｿｽ蜉�縲ゅい繝ｼ繝・ぅ繝輔ぃ繧ｯ繝医ム繧ｦ繝ｳ繝ｭ繝ｼ繝峨・繝槭・繧ｸ繝ｻ`status.json` 繧ょｯｾ蠢懊�・
  - `nepl-core/tests/harness.rs`: `run_main_capture_stdout_with_stdin` 縺ｫ `path_open`繝ｻ`fd_close`繝ｻ`args_sizes_get`繝ｻ`args_get` 縺ｮWASI繧ｹ繧ｿ繝悶ｒ霑ｽ蜉�・・std/streamio` 邨檎罰縺ｧ繧､繝ｳ繝昴・繝医＆繧後ｋ髢｢謨ｰ縺・linker missing 縺ｧ繧､繝ｳ繧ｹ繧ｿ繝ｳ繧ｹ蛹門､ｱ謨励＠縺ｦ縺・◆縺溘ａ・峨�・
  - `nepl-core/tests/kp.rs`: `if then:` 繝悶Ο繝・け蜀・〒縺ｮ `;` 菴ｿ逕ｨ繧呈賜髯､・・';' is not allowed in if layout expression` 繧ｨ繝ｩ繝ｼ・峨�Ａlet b0 <i32> load_u8 buf; print_i32 b0` 竊・`print_i32 load_u8 buf` 縺ｫ螟画峩縺励�～else print_i32 -1` 竊・`else: print_i32 -1` 縺ｫ螟画峩縲・
- [遒ｺ隱・縺九￥縺ｫ繧転:
  - `cargo test -p nepl-core --test kp`: 蜈ｨ14莉ｶ PASS

# 2026-03-17 菴懈･ｭ繝｡繝｢ (fix: intrinsic/numerics/kp 繝・せ繝井ｿｮ豁｣)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `nepl-core/tests/intrinsic.rs`, `numerics.rs`, `kp.rs` 縺ｮ螟ｱ謨励ユ繧ｹ繝医ｒ菫ｮ豁｣縺吶ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nepl-core/tests/numerics.rs`: 蟒・ｭ｢縺輔ｌ縺溷梛莉倥″髢｢謨ｰ蜷搾ｼ・i32_add`, `i32_and`, `u8_lt_u` 遲会ｼ峨ｒ蝙区耳隲悶・繝ｼ繧ｹ縺ｮ蜈ｱ騾壼錐・・add`, `and`, `lt_u` 遲会ｼ峨↓荳�諡ｬ鄂ｮ謠帙�・
  - `nepl-core/tests/intrinsic.rs`:
    - `i64_add i64_extend_i32_u` 竊・荳ｭ髢灘､画焚 `let a <i64> cast 12345; let b <i64> cast 67890; let v <i64> add a b;` 縺ｫ螟画峩・亥梛謗ｨ隲悶′ `add cast X cast Y` 繧堤峩謗･隗｣豎ｺ縺ｧ縺阪↑縺九▲縺溘◆繧・ｼ峨�・
    - `i64_eq`, `f64_eq` 竊・`eq` 縺ｫ螟画峩縲・
    - `f64_convert_i32_s 42` 竊・`cast 42` 縺ｫ螟画峩縲・
    - `alloc 8` / `dealloc p 8` 竊・`alloc_raw 8` / `dealloc_raw p 8` 縺ｫ螟画峩・・alloc`/`dealloc` 縺ｯ `Result` 繧定ｿ斐☆螳牙・API 縺ｫ螟画峩貂医∩縺ｮ縺溘ａ・峨�・
    - `#import "core/cast" as *` 繧定ｿｽ蜉�縲・
  - `nepl-core/tests/kp.rs`:
    - `kp/kpread`, `kp/kpwrite` 繝｢繧ｸ繝･繝ｼ繝ｫ縺・`std/streamio` 縺ｫ遘ｻ陦梧ｸ医∩縺ｮ縺溘ａ縲∝・繝・せ繝医ｒ譁ｰAPI・・StreamScanner`, `StreamWriter`, `open ReadStream::Stdio` 遲会ｼ峨ｒ菴ｿ縺｣縺溷ｮ溯｣・↓譖ｸ縺咲峩縺励�・
    - `scanner_new`/`scanner_read_*`/`writer_new`/`writer_write_*` 竊・`open ReadStream::Stdio`/`read sc`/`open WriteStream::Stdio`/`write w`/`writeln w`/`flush w`/`close` 縺ｫ螟画峩縲・
    - `alloc`/`dealloc`/`realloc` 竊・`alloc_raw`/`dealloc_raw`/`realloc_raw`・育函繝昴う繝ｳ繧ｿ謫堺ｽ懊′蠢・ｦ√↑菴弱Ξ繝吶Ν繝・せ繝育畑・峨�・
    - `i64_extend_i32_u` 竊・`cast`, `i64_add` 竊・`add` 縺ｫ螟画峩縲・
    - 蜀・Κ繝｡繝｢繝ｪ讒矩��繧堤峩謗･讀懈渊縺励※縺・◆繝・ヰ繝・げ繝・せ繝茨ｼ・kpread_scanner_header_debug`, `kpread_buffer_bytes_debug`・峨・譁ｰAPI 縺ｮ蜈ｬ髢九う繝ｳ繧ｿ繝ｼ繝輔ぉ繝ｼ繧ｹ邨檎罰縺ｮ繝・せ繝医↓譖ｸ縺咲峩縺励�・
- [遒ｺ隱・縺九￥縺ｫ繧転:
  - `cargo test -p nepl-core --test intrinsic`: 4莉ｶ PASS
  - `cargo test -p nepl-core --test numerics`: 11莉ｶ PASS

# 2026-03-17 菴懈･ｭ繝｡繝｢ (fix: D3005 ambiguous overload in binary_heap doctests)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `stdlib/alloc/collections/binary_heap.nepl` 縺ｮdoctest #1縲・5 縺ｧ逋ｺ逕溘＠縺ｦ縺・◆ D3005 縲径mbiguous overload縲阪お繝ｩ繝ｼ繧剃ｿｮ豁｣縺吶ｋ縲・
  - `with_capacity` 縺・`binary_heap`繝ｻ`vec`繝ｻ`deque`繝ｻ`queue`繝ｻ`ringbuffer` 縺ｪ縺ｩ縺ｧ蜷悟錐縺ｧ螳夂ｾｩ縺輔ｌ縺ｦ縺翫ｊ縲√Ο繝ｼ繝�繝ｼ縺ｮ flat namespace inlining 縺ｫ繧医ｊ縺吶∋縺ｦ縺ｮ繧ｷ繝ｳ繝懊Ν縺悟酔荳�繧ｹ繧ｳ繝ｼ繝励↓蜈･繧九％縺ｨ縺ｧ逋ｺ逕溘☆繧九�・
- [譬ｹ譛ｬ蜴溷屏蛻・梵/縺薙ｓ縺ｽ繧薙￡繧薙＞繧薙・繧薙○縺江:
  - **蜴溷屏1**: `function_signature_for_entry` 縺後�‘xplicit type_args 繧呈戟縺､ outer caller 繧ｨ繝ｳ繝医Μ・井ｾ・ `unwrap_ok<BinaryHeap<i32>, StdErrorKind>`・峨↓蟇ｾ縺励※ `None` 繧定ｿ斐＠縺ｦ縺・◆縲４tackEntry 縺ｮ `ty` 縺・0蛟九・ type_params 繧呈戟縺､ fresh placeholder type 縺ｧ菴懊ｉ繧後※縺翫ｊ縲～type_params.len() != entry.type_args.len()` 縺ｫ縺ｪ繧九◆繧√�ゅ％繧後↓繧医ｊ `infer_expected_from_outer_consumer` 縺・`None` 繧定ｿ斐＠縲‘xpected_ret 縺檎ｩｺ縺ｫ縺ｪ縺｣縺ｦ蛟呵｣懊′邨槭ｉ繧後↑縺九▲縺溘�・
  - **蜴溷屏2**: `vec.nepl` 縺ｮ `fn map`, `fn filter`, `fn partition`, `fn take_while`, `fn drop_while` 縺・`match with_capacity<.T> cap:` 縺ｮ蠖｢蠑上〒繝槭ャ繝√・繧ｹ繧ｯ繝ｫ繝ｼ繝・ぅ繝九・縺ｨ縺励※逶ｴ謗･ `with_capacity` 繧貞他繧薙〒縺・◆縲ゅ・繝・メ繧ｹ繧ｯ繝ｫ繝ｼ繝・ぅ繝九・縺ｯ `expected_last_ty = None` 縺ｧ隧穂ｾ｡縺輔ｌ繧九◆繧√�∵悄蠕・梛縺ｫ繧医ｋ蛟呵｣懃ｵ槭ｊ霎ｼ縺ｿ縺悟ロ縺九★D3005縺檎匱逕溘＠縺溘�・
- [菫ｮ豁｣蜀・ｮｹ/縺励ｅ縺・○縺・↑縺・ｈ縺・:
  - **Fix 1** (`nepl-core/src/typecheck.rs`): `function_signature_for_entry` 縺ｫ fallback 繝ｭ繧ｸ繝・け繧定ｿｽ蜉�縲Ａtype_params.len() != entry.type_args.len()` 縺ｮ蝣ｴ蜷医�～env.lookup_all_callables` 縺ｧ螳滄圀縺ｮ繝舌う繝ｳ繝・ぅ繝ｳ繧ｰ蝙九ｒ讀懃ｴ｢縺励�》ype_args 謨ｰ縺御ｸ�閾ｴ縺吶ｋ繧ゅ・繧剃ｽｿ縺｣縺ｦ蝙倶ｻ｣蜈･繧定｡後▲縺ｦ霑斐☆繧医≧縺ｫ縺励◆縲・
  - **Fix 2** (`stdlib/alloc/collections/vec.nepl`): `match with_capacity<.T> cap:` 繧・`let alloc_r <Result<Vec<.T>, StdErrorKind>> with_capacity<.T> cap` + `match alloc_r:` 縺ｫ螟画峩縲ょ梛繧｢繝弱ユ繝ｼ繧ｷ繝ｧ繝ｳ莉倥″ `let` 繝舌う繝ｳ繝・ぅ繝ｳ繧ｰ縺ｫ繧医ｊ pending_ascription 縺瑚ｨｭ螳壹＆繧後�～with_capacity` 縺ｮ蜻ｼ縺ｳ蜃ｺ縺励〒譛溷ｾ・梛縺ｫ繧医ｋ蛟呵｣懃ｵ槭ｊ霎ｼ縺ｿ縺梧ｭ｣縺励￥蜍穂ｽ懊☆繧九ｈ縺・↓縺励◆縲・
  - `infer_expected_type_from_match_arms`: 繝槭ャ繝√い繝ｼ繝�縺ｮ繝舌Μ繧｢繝ｳ繝亥錐縺九ｉ繧ｹ繧ｯ繝ｫ繝ｼ繝・ぅ繝九・縺ｮ蝓ｺ蠎鋲num蝙九ｒ謗ｨ隲悶☆繧玖｣懷勧髢｢謨ｰ繧定ｿｽ蜉�縲Ｇresh螟画焚繧剃ｽｿ縺・◆繧・ambiguous 縺ｪ繧ｱ繝ｼ繧ｹ縺ｧ縺ｯ邨槭ｊ霎ｼ縺ｿ縺ｫ菴ｿ縺医↑縺・′縲∝渕蠎募梛縺ｮ繝偵Φ繝医→縺励※讖溯・縺吶ｋ縲・
- [遒ｺ隱・縺九￥縺ｫ繧転:
  - `binary_heap.nepl` doctest #1縲・6: 縺吶∋縺ｦ PASS
  - `vec.nepl` doctest #1縲・10: 縺吶∋縺ｦ PASS
  - `cargo test --workspace`: `generics_nested_option_match` 1莉ｶ螟ｱ謨暦ｼ域里蟄倥・ pre-existing 蝠城｡後�∵悽螟画峩縺ｨ縺ｯ辟｡髢｢菫ゑｼ・

# 2026-03-17 菴懈･ｭ繝｡繝｢ (CI 菫ｮ豁｣: parser.js artifact 谺�關ｽ繝ｻrust-test 菫ｮ豁｣)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - GitHub Actions 縺ｮ nmd-doctest/wasi-test 縺・`Cannot find module './parser'` 縺ｧ螟ｱ謨励☆繧句撫鬘後→ rust-test 縺ｮ `emit_ll_skips_unsupported_parsed_function_body` 螟ｱ謨励ｒ菫ｮ豁｣縺吶ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `.github/workflows/ci.yml`: bootstrap-build artifact 縺ｫ `nodesrc/parser.js` 縺ｨ `nodesrc/html_gen.js` 繧定ｿｽ蜉�・・ypeScript 繧ｳ繝ｳ繝代う繝ｫ貂医∩繝輔ぃ繧､繝ｫ縺・.gitignore 縺輔ｌ縺ｦ縺翫ｊ縲√ム繧ｦ繝ｳ繝ｭ繝ｼ繝牙・縺ｮ繧ｸ繝ｧ繝悶〒隕九▽縺九ｉ縺ｪ縺九▲縺滂ｼ峨�・
  - `nepl-core/src/codegen_llvm.rs`: `emit_ll_skips_unsupported_parsed_function_body` 繝・せ繝医ｒ `add 1 2`・・core/math` 譛ｪ import 縺ｧ D3001 繧ｨ繝ｩ繝ｼ・峨°繧・`fn body <(i32)->i32> (x): x`・域怏蠑墓焚髢｢謨ｰ縺ｯ `lower_parsed_fn_with_gates` 縺ｧ繧ｹ繧ｭ繝・・縺輔ｌ繧具ｼ峨↓螟画峩縲・
- [險ｭ險域ｱｺ螳・縺帙▲縺代＞縺代▲縺ｦ縺Ь:
  - 繝・せ繝医・繧ｻ繝槭Φ繝・ぅ繧ｯ繧ｹ縺ｯ螟峨ｏ繧峨↑縺・ｼ医�後ヱ繝ｼ繧ｹ貂医∩繝懊ョ繧｣繧呈戟縺､髢｢謨ｰ縺・LLVM 蜃ｺ蜉帙↓迴ｾ繧後↑縺・％縺ｨ縲阪ｒ讀懆ｨｼ縺吶ｋ・峨�よ怏蠑墓焚髢｢謨ｰ縺ｯ `params.is_empty()` 繝√ぉ繝・け縺ｧ蠢・★繧ｹ繧ｭ繝・・縺輔ｌ繧九�・
- [險育判縺ｨ縺ｮ蟾ｮ逡ｰ]:
  - CI 險ｭ螳壹・荳肴紛蜷井ｿｮ豁｣・・lan.md 縺ｫ險倩ｼ峨↑縺暦ｼ峨�・

---

# 2026-03-17 菴懈･ｭ繝｡繝｢ (NEPLg2.0 螳牙ｮ壼喧: tuple 繝ｬ繧､繧｢繧ｦ繝医・pipe 菫ｮ豁｣繝ｻ繝・せ繝井ｿｮ豁｣)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `tutorials/` 縺ｨ `nepl-core/tests/` 縺ｮ螟ｱ謨励ユ繧ｹ繝医ｒ菫ｮ豁｣縺・NEPLg2.0 繧貞ｮ牙ｮ壼喧縺吶ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nepl-core/src/typecheck.rs`:
    - `type_storage_size_bytes` 繧・`codegen_wasm.rs` 縺ｮ繝ｬ繧､繧｢繧ｦ繝医↓蜷医ｏ縺帑ｿｮ豁｣・・nit/Never=0, U8=1, i64/u64/f64=8, Struct/Tuple=蜀榊ｸｰ蜥・ 縺昴ｌ莉･螟・4・峨�・
    - `PrefixItem::Pipe` 縺ｮ drain 繧・let/set 譚溽ｸ帙ｒ雜翫∴縺ｪ縺・ｈ縺・ｿｮ豁｣・・let a <i32> 1 |> add 2` 縺・D3013 繧ｨ繝ｩ繝ｼ縺ｫ縺ｪ繧倶ｸ榊・蜷医ｒ隗｣豸茨ｼ峨�・
  - `nepl-core/src/codegen_wasm.rs`:
    - `TupleConstruct` 縺ｧ縺ｮ Unit 隕∫ｴ�縺ｫ蟇ｾ縺吶ｋ隱､縺｣縺・4 繝舌う繝域嶌縺崎ｾｼ縺ｿ繧帝勁蜴ｻ・・nit 縺ｯ繝｡繝｢繝ｪ繧貞頃譛峨＠縺ｪ縺・◆繧∝憶菴懃畑隧穂ｾ｡縺ｮ縺ｿ縺ｫ螟画峩・峨�・
  - `nepl-core/src/codegen_llvm.rs`:
    - `emit_ll_from_module_for_target` 蜻ｼ縺ｳ蜃ｺ縺励↓荳崎ｶｳ縺励※縺・◆隨ｬ4蠑墓焚 `false`・・inify・峨ｒ霑ｽ蜉�縲・
  - `nepl-core/tests/typeannot.rs`:
    - 繝・せ繝亥・縺ｮ蜀・Κ繝薙Ν繝医う繝ｳ蜷搾ｼ・i32_add`, `i32_mul`, `i32_lt_s`・峨ｒ stdlib 蜈ｬ髢句錐・・add`, `mul`, `lt`・峨↓菫ｮ豁｣縲・
  - `nepl-core/tests/tuple_new_syntax.rs`:
    - `tuple_return_value`: 繝｢繝弱Δ繝ｫ繝募喧 ICE 繧定ｵｷ縺薙＠縺ｦ縺・◆繧ｸ繧ｧ繝阪Μ繝・け繝ｩ繝・ヱ髢｢謨ｰ繧帝勁蜴ｻ縺励�∫峩謗･ Tuple 讒狗ｯ峨↓螟画峩縲・
  - `README.md`:
    - CLI 菴ｿ逕ｨ譁ｹ豕輔そ繧ｯ繧ｷ繝ｧ繝ｳ繧貞炎髯､・・doc/cli.md` 縺ｫ遘ｻ邂｡貂医∩・峨�・
    - `tutorials/getting_started/`繝ｻstdlib 讒区・繝ｻNEPLg2.1 遘ｻ陦瑚ｨ育判繧ｻ繧ｯ繧ｷ繝ｧ繝ｳ繧定ｿｽ蜉�縲・
  - `doc/` 蜷・ｨｮ:
    - `2.1impl/index.md`: Stage 1窶・ 竊・M1窶溺6 陦ｨ險倅ｿｮ豁｣繝ｻ`doc/migration/index.md` 蜿ら・霑ｽ蜉�縲・
    - `self_host.md`: Bootstrap "Stage 1/2" 竊・"Pass 1/2" 縺ｫ謾ｹ蜷搾ｼ郁｡晉ｪ∬ｧ｣豸茨ｼ峨・豕ｨ諢乗嶌縺崎ｿｽ蜉�縲・
    - `README.md`: `examples/` 繧ｻ繧ｯ繧ｷ繝ｧ繝ｳ霑ｽ蜉�縲・
    - `compare/syntax.md`, `compare/memory_model.md`, `compare/module_system.md`: 隧ｳ邏ｰ莉墓ｧ倥ヵ繝・ち繝ｼ霑ｽ蜉�縲・
- [菫ｮ豁｣縺励◆荳榊・蜷・:
  - `tuple_unit_elements`: Unit 隕∫ｴ�縺ｮ繧ｵ繧､繧ｺ荳堺ｸ�閾ｴ・・ypecheck=4, codegen=0・峨↓繧医ｊ蠕檎ｶ壹ヵ繧｣繝ｼ繝ｫ繝峨・繧ｪ繝輔そ繝・ヨ縺後★繧後�∝�､縺・0 縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
  - `let a <i32> 1 |> add 2`: pipe drain 縺・let 譚溽ｸ帙お繝ｳ繝医Μ繧剃ｸ�邱偵↓ drain縺吶ｋ縺溘ａ D3013 繧ｨ繝ｩ繝ｼ縲・
  - `from_i32 n` 縺・FizzBuzz 縺ｧ "0" 繧定ｿ斐☆蝠城｡・ tuple 繝ｬ繧､繧｢繧ｦ繝井ｿｮ豁｣縺ｫ繧医ｊ隗｣豸医�・
  - `checks_print_report` 縺ｮ繧､繝ｳ繝・ャ繧ｯ繧ｹ縺・"[0]" 繧・2 蝗櫁｡ｨ遉ｺ縺吶ｋ蝠城｡・ 蜷御ｸ翫�・
- [險育判縺ｨ縺ｮ蟾ｮ逡ｰ]:
  - plan.md 縺ｫ險倩ｼ峨↑縺暦ｼ医ヰ繧ｰ菫ｮ豁｣・峨�・
- [谿玖ｪｲ鬘珪:
  - `emit_ll_skips_unsupported_parsed_function_body` 繝・せ繝医′螟ｱ謨励☆繧句庄閭ｽ諤ｧ・・I 縺ｧ遒ｺ隱搾ｼ峨�・

---

# 2026-03-17 菴懈･ｭ繝｡繝｢ (doc/2.1impl: 繧ｳ繝ｳ繝代う繝ｩ讒区・險ｭ險・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - 迴ｾ陦・`nepl-core/src/` 縺ｮ讒矩��荳翫・蝠城｡後ｒ謨ｴ逅・＠縲¨EPLg2.1 繝悶・繝医せ繝医Λ繝・・繧ｳ繝ｳ繝代う繝ｩ縺ｮ逶ｮ讓吶ヵ繧｡繧､繝ｫ/繝輔か繝ｫ繝�讒区・繧定ｨｭ險医☆繧九�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `doc/2.1impl/compiler_structure.md` 譁ｰ隕丈ｽ懈・
    - 迴ｾ陦後・蝠城｡檎せ荳�隕ｧ・・ypecheck.rs 8871陦悟ｷｨ螟ｧ繝ｻResource IR 荳榊惠繝ｻ繝輔Λ繝・ヨ讒矩��遲会ｼ・
    - NEPLg2.1 Rust 繝悶・繝医せ繝医Λ繝・・繧ｳ繝ｳ繝代う繝ｩ・・nepl-core-2.1`・峨・繝輔か繝ｫ繝�讒区・
    - 繝代う繝励Λ繧､繝ｳ繧ｹ繝・・繧ｸ = 繝・ぅ繝ｬ繧ｯ繝医Μ髫主ｱ､縺ｨ縺・≧險ｭ險亥次蜑・
    - 繧ｻ繝ｫ繝輔・繧ｹ繝茨ｼ・stdlib/neplg2/`・峨→縺ｮ蜻ｽ蜷阪ヱ繝ｪ繝・ぅ險ｭ險・
    - 迴ｾ陦後ヵ繧｡繧､繝ｫ縺ｨ譁ｰ隕上ヵ繧｡繧､繝ｫ縺ｮ蟇ｾ蠢懆｡ｨ・・5莉ｶ・・
    - Stage 1窶・ 縺ｮ遘ｻ陦梧姶逡･
- [險ｭ險域ｱｺ螳・縺帙▲縺代＞縺代▲縺ｦ縺Ь:
  - `typecheck.rs` 繧・`check/` 7 繝輔ぃ繧､繝ｫ縺ｫ蛻・牡・域怙螟ｧ縺ｮ螟画峩・・
  - `resource/` 繝｢繧ｸ繝･繝ｼ繝ｫ繧呈眠險ｭ・・esource IR 縺ｮ隨ｬ荳�邏夐・鄂ｮ・・
  - `nm/` 繧偵さ繧｢繧ｳ繝ｳ繝代う繝ｩ縺九ｉ迢ｬ遶具ｼ医ヤ繝ｼ繝ｫ繝√ぉ繝ｼ繝ｳ陬懷勧・・
  - `nepl-core-2.1` 縺ｨ縺励※迴ｾ陦後→荳ｦ陦碁幕逋ｺ縺励�ヾtage 6 縺ｧ蛻・ｊ譖ｿ縺・
  - 繧ｻ繝ｫ繝輔・繧ｹ繝医・繝悶・繝医せ繝医Λ繝・・縺・Stage 4 莉･髯阪↓縺ｪ縺｣縺ｦ縺九ｉ逹�謇・

---

# 2026-03-17 菴懈･ｭ繝｡繝｢ (doc: 隨ｬ5蝗槭Ξ繝薙Η繝ｼ縺ｫ繧医ｋ莉墓ｧ倡ｩｴ縺ｮ隗｣豸・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - 5螟ｧ蝓ｺ譛ｬ逅・ｿｵ縺ｫ辣ｧ繧峨＠縺溽ｬｬ5蝗槫桁諡ｬ逧・ｲｾ譟ｻ縲ゆｻ墓ｧ倡ｩｴ繝ｻ螳夂ｾｩ荳崎ｶｳ繝ｻ繧ｯ繝ｭ繧ｹ繝輔ぃ繧､繝ｫ荳肴紛蜷医ｒ隗｣豸医�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `doc/2.1spec/types.md`: ﾂｧ9 霑ｽ蜉� 窶・繧ｸ繧ｧ繝阪Μ繧ｯ繧ｹ縺ｮ荳榊､会ｼ・nvariant・画э蜻ｳ隲悶ｒ譏取枚蛹悶�・
  - `doc/2.1spec/declarations.md ﾂｧ5`: `Self` 繧ｭ繝ｼ繝ｯ繝ｼ繝峨・螳夂ｾｩ・・rait 繝｡繧ｽ繝・ラ蜀・・迚ｹ蛻･蝙句､画焚・峨ｒ霑ｽ險倥�Ｕrait 繝｡繧ｽ繝・ラ縺ｮ `...` 縺ｨ default body 縺ｮ蛹ｺ蛻･繧呈・險倥�つｧ4.1: bare 繝舌Μ繧｢繝ｳ繝井ｽｿ逕ｨ譚｡莉ｶ・・譚｡莉ｶ・峨ｒ霑ｽ險倥�・
  - `doc/2.1spec/effects.md ﾂｧ5`: `Slice .T` 繧・`Unrestricted`・・orrowed view・峨→縺励※雉・ｺ蝉ｽｿ逕ｨ繝・・繝悶Ν縺ｫ霑ｽ蜉�縲つｧ3.2.1: Rust 縺ｨ縺ｮ驕輔＞・医Λ繧､繝輔ち繧､繝�豕ｨ驥医↑縺暦ｼ峨ｒ霑ｽ險倥�・
  - `doc/2.1spec/memory.md ﾂｧ3.1`: "region" 縺ｮ蠖｢蠑冗噪螳夂ｾｩ繧定ｿｽ蜉�縲つｧ6.1: `str` 縺ｮ豁｣隕丞喧蠖｢蠑擾ｼ・FC 遲峨・閾ｪ蜍暮←逕ｨ縺ｪ縺暦ｼ峨ｒ譏手ｨ倥�・
  - `doc/2.1spec/modules.md ﾂｧ4`: `merge` 縺ｮ陦晉ｪ∬ｧ｣豎ｺ隕丞援・亥酔蜷榊ｮ｣險�繝ｻpart 縺ｮ蜊倅ｸ� anchor 蛻ｶ邏・ｼ峨ｒ霑ｽ蜉�縲・
  - `doc/2.1spec/traits.md ﾂｧ2.3`: `Copy` trait 縺ｯ Linear/Owned 蝙九↓螳溯｣・ｸ榊庄縺ｧ縺ゅｋ縺薙→繧呈・險倥・cross-ref 繧定ｿｽ蜉�縲・
  - `doc/2.1spec/stdlib.md ﾂｧ2.1/ﾂｧ3`: `rand` 繧・`core/` 縺九ｉ蜑企勁縺・`features/` 縺ｫ遘ｻ蜍包ｼ・mpure繝ｻ髱樊ｱｺ螳夂噪縺ｮ縺溘ａ・峨�・
  - `doc/2.1spec/phase8.md ﾂｧ2.3`: 險ｼ譏弱が繝悶ず繧ｧ繧ｯ繝医・ `Copy` 蝙九〒縺ゅｋ縺薙→繧呈・險倥�よｱｺ螳壻ｸ榊庄閭ｽ蜻ｽ鬘後・蟇ｾ雎｡螟悶→縺吶ｋ譁ｹ驥昴ｒ霑ｽ蜉�縲・
  - `doc/2.1spec/syntax.md ﾂｧ8.2`: Phase 8 繧ｳ繝｡繝ｳ繝亥・縺ｮ諡ｬ蠑ｧ `WillExecute (le 1 n)` 竊・`WillExecute le 1 n`縲・
  - `doc/2.1spec/compiler.md ﾂｧ8`: Phase 逡ｪ蜿ｷ縺ｨ繧ｳ繝ｳ繝代う繝ｩ Stage 逡ｪ蜿ｷ縺ｮ豺ｷ蜷後ｒ隗｣豸茨ｼ・tage 1窶・ 縺ｨ險�隱・Phase 0窶・ 繧貞玄蛻･・峨�・
  - `doc/compare/syntax.md ﾂｧ12`: 繝舌Μ繧｢繝ｳ繝亥盾辣ｧ縺ｮ breaking change 豕ｨ險倥ｒ霑ｽ蜉�縲・
  - `doc/compare/module_system.md ﾂｧ2.2`: `use` 縺ｮ `::` 縺ｨ繝舌Μ繧｢繝ｳ繝医・ `::` 縺ｮ驕輔＞繧定ｿｽ險倥�・
  - `doc/compare/index.md`: Orphan Rule繝ｻNLL繝ｻinvariant semantics繝ｻpub use 蠕ｪ迺ｰ讀懷・繧定ｿｽ蜉�縲・
- [險ｭ險域ｱｺ螳・縺帙▲縺代＞縺代▲縺ｦ縺Ь:
  - 繧ｸ繧ｧ繝阪Μ繧ｯ繧ｹ縺ｯ Phase 0窶・ 縺ｧ螳悟・縺ｫ invariant縲Ｄo/contravariance 縺ｯ Phase 8 讀懆ｨ手ｪｲ鬘後�・
  - 險ｼ譏弱が繝悶ず繧ｧ繧ｯ繝医・ Copy 蝙具ｼ域ｶ郁ｲｻ荳崎ｦ・ｼ峨�よｱｺ螳壻ｸ榊庄閭ｽ蜻ｽ鬘後・蝙九す繧ｹ繝・Β螟悶�・
  - `rand` 縺ｯ Impure繝ｻ髱樊ｱｺ螳夂噪縺ｮ縺溘ａ `features/` 螻､・・core/` 縺ｯ Pure 縺ｮ縺ｿ・峨�・

---

# 2026-03-17 菴懈･ｭ繝｡繝｢ (doc: 隨ｬ4蝗槭Ξ繝薙Η繝ｼ縺ｫ繧医ｋ莉墓ｧ倅ｸ肴紛蜷井ｿｮ豁｣)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - 5螟ｧ蝓ｺ譛ｬ逅・ｿｵ・亥燕鄂ｮ險俶ｳ墓峡蠑ｧ縺ｪ縺励・蠑ｷ蜉帙↑髱咏噪讀懈渊繝ｻ蝙句ｮ牙・繝｡繝｢繝ｪ螳牙・繝ｻ萓晏ｭ伜梛蟆主・貅門ｙ繝ｻ繝槭Ν繝√・繝ｩ繝・ヨ繝輔か繝ｼ繝�・峨↓辣ｧ繧峨＠縺ｦ doc/ 蜈ｨ菴薙ｒ邊ｾ譟ｻ縺励�∵ｮ句ｭ倅ｸ肴紛蜷医ｒ菫ｮ豁｣縺吶ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `doc/2.1spec/stdlib.md ﾂｧ2.2`: `RegionToken .T` 竊・`RegionToken`・亥梛繝代Λ繝｡繝ｼ繧ｿ縺ｪ縺励�ゆｻ悶・縺吶∋縺ｦ縺ｮ譁・嶌縺ｨ縺ｮ謨ｴ蜷茨ｼ峨�・
  - `doc/2.1spec/memory.md ﾂｧ2.B`: `OwnedBuf .T` 繧・Unique Mutable Work State 萓九↓霑ｽ蜉�縲ＡByteBuf`/`OwnedBuf .T`/`VecBuilder .T`/`StringBuilder` 縺ｮ逕ｨ騾泌ｷｮ繧呈・險倥�・
  - `doc/2.1spec/memory.md ﾂｧ3.2`: `Linear` 雉・ｺ舌ｂ Drop Elaboration 縺ｮ蟇ｾ雎｡縺ｧ縺ゅｋ縺薙→繧呈・險假ｼ医�梧囓鮟咏噪縺ｪ遐ｴ譽・・遖∵ｭ｢縲阪→縲後さ繝ｳ繝代う繝ｩ閾ｪ蜍・drop 縺ｮ謖ｿ蜈･縲阪・遏帷崟縺励↑縺・％縺ｨ繧定ｪｬ譏趣ｼ峨�・
  - `doc/2.1spec/effects.md ﾂｧ5.1`: `Linear` + `Drop` 縺ｮ逶ｸ莠剃ｽ懃畑繧定・蜉帙ユ繝ｼ繝悶Ν蠕後・陬懆ｶｳ縺ｫ霑ｽ蜉�縲・
  - `doc/2.1spec/patterns.md ﾂｧ6`: `::` 縺後Δ繧ｸ繝･繝ｼ繝ｫ菫ｮ鬟ｾ縺ｧ縺ｪ縺丞梛蜷堺ｿｮ鬟ｾ縺ｧ縺ゅｋ縺薙→繧定ｿｽ險倥�Ｃare 繝舌Μ繧｢繝ｳ繝亥錐縺ｮ譚｡莉ｶ繝ｻ陦晉ｪ∵凾縺ｮ繧ｨ繝ｩ繝ｼ謖吝虚繧定ｿｽ險倥�Ａdeclarations.md ﾂｧ4.1` 縺ｸ縺ｮ逶ｸ莠貞盾辣ｧ繧定ｿｽ蜉�縲・
  - `doc/compare/index.md`: 縲悟炎髯､縺輔ｌ繧九ｂ縺ｮ縲阪↓ `#entry` 譁・ｳ募､画峩繝ｻ陬懷勧繝槭・繧ｫ繝ｼ蟒・ｭ｢繝ｻ諡ｬ蠑ｧ繧ｰ繝ｫ繝ｼ繝怜ｻ・ｭ｢繝ｻ繧ｻ繝溘さ繝ｭ繝ｳ蟒・ｭ｢繧定ｿｽ蜉�縲ゅ�瑚ｿｽ蜉�縺輔ｌ繧九ｂ縺ｮ縲阪↓ borrow 險俶ｳ輔・`module name:` 繝悶Ο繝・け繝ｻ`EnumType::Variant` 菫ｮ鬟ｾ蠖｢繧定ｿｽ蜉�縲・
  - `doc/examples/05_io_and_resources.nepl`: 繧ｳ繝｡繝ｳ繝医�悟ｮ溯｣・ｾ晏ｭ倥�阪ｒ蜑企勁縺励�∬ｨ�隱樔ｻ墓ｧ倥→縺励※ `Err` 蛛ｴ縺ｫ File 縺瑚ｿ斐ｉ縺ｪ縺・％縺ｨ繧呈・險倥�・
  - `doc/examples/06_generics_and_traits.nepl`: trait 繝｡繧ｽ繝・ラ縺ｮ繝・ヵ繧ｩ繝ｫ繝医↑縺玲悽菴薙↓ `...` 繧定ｿｽ蜉�・・declarations.md ﾂｧ5` 縺ｮ莉墓ｧ倥↓蜷医ｏ縺帙ｋ・峨�・
- [谿玖ｪｲ鬘・縺ｮ縺薙°縺�縺Ь:
  - Phase 4 莉･髯阪・ `MemReadable`/`MemWritable`/`RegionOwned` 蠑ｷ蛻ｶ縺ｯ蠑輔″邯壹″螳溯｣・ｾ・■縲・

---

# 2026-03-17 菴懈･ｭ繝｡繝｢ (doc: 隨ｬ3蝗槭Ξ繝薙Η繝ｼ縺ｫ繧医ｋ莉墓ｧ倥ヰ繧ｰ菫ｮ豁｣繝ｻ谺�關ｽ蟾ｮ蛻・｣懷・)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - 隨ｬ3蝗槫､夜Κ繝ｬ繝薙Η繝ｼ縺ｧ謖・遭縺輔ｌ縺滉ｻ墓ｧ倥ヰ繧ｰ繝ｻ萓九・隱､繧翫・compare 譁・嶌縺ｮ蟾ｮ蛻・ｼ上ｌ繧剃ｿｮ豁｣縺吶ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `doc/2.1spec/effects.md ﾂｧ5`: `File`/`Socket` 繧・`Owned` 縺九ｉ `Linear` 縺ｫ遘ｻ蜍包ｼ亥酔譁・嶌蜀・・萓九・memory.md 縺ｨ縺ｮ遏帷崟繧定ｧ｣豸茨ｼ峨�ＡByteBuf`/`StringBuilder` 繧・`Owned` 陦後↓霑ｽ蜉�縲ょ粋謌蝉ｾ九ｒ譖ｴ譁ｰ縲・
  - `doc/2.1spec/declarations.md ﾂｧ4`: `some 10` 竊・`Option::Some 10` 縺ｮ螟ｧ譁・ｭ励Α繧ｹ菫ｮ豁｣縲つｧ4.1 縺ｨ縺励※繝舌Μ繧｢繝ｳ繝亥錐蜑崎ｧ｣豎ｺ隕丞援繧呈眠險ｭ・井ｿｮ鬟ｾ蠖｢ `Type::Variant` / bare 蠖｢縺ｮ菴ｿ縺・・縺代・`::` 縺後Δ繧ｸ繝･繝ｼ繝ｫ菫ｮ鬟ｾ縺ｧ縺ｪ縺丞梛菫ｮ鬟ｾ縺ｧ縺ゅｋ縺薙→繧呈・險假ｼ峨�・
  - `doc/2.1spec/patterns.md ﾂｧ2.8 / ﾂｧ4.3`: OR 繝代ち繝ｼ繝ｳ萓九・ match arm 蝙倶ｸ堺ｸ�閾ｴ繧剃ｿｮ豁｣・・莉ｶ・峨�・
  - `doc/2.1spec/syntax.md ﾂｧ11`: `StringBuilder::new unit` 竊・`new unit`・・are 蜷肴婿驥昴→縺ｮ謨ｴ蜷茨ｼ峨�・
  - `doc/2.1spec/memory.md ﾂｧ8.3`: I/O handle 螟ｱ謨玲凾縺ｮ謇�譛画ｨｩ繧呈・譁・喧・・Err` 蛛ｴ縺ｫ File 縺瑚ｿ斐ｉ縺ｪ縺・ｨｭ險域э蝗ｳ繝ｻ繝ｪ繝医Λ繧､蜿ｯ閭ｽ API 縺ｮ繧ｷ繧ｰ繝阪メ繝｣萓九ｒ霑ｽ蜉�・峨�・
  - `doc/compare/syntax.md ﾂｧ9窶・2`: 谺�關ｽ蟾ｮ蛻・ｒ霑ｽ蜉�・郁｣懷勧繝槭・繧ｫ繝ｼ蟒・ｭ｢縲∵峡蠑ｧ繧ｰ繝ｫ繝ｼ繝怜ｻ・ｭ｢縲√そ繝溘さ繝ｭ繝ｳ蟒・ｭ｢縲√ヰ繝ｪ繧｢繝ｳ繝亥盾辣ｧ險俶ｳ輔・螟画峩・峨�・
  - `doc/compare/module_system.md ﾂｧ2.1/2.5`: `#entry` 譁・ｳ募､画峩縺ｨ `module name:` 繝悶Ο繝・け譁ｰ險ｭ縺ｮ蟾ｮ蛻・ｒ霑ｽ蜉�縲・
- [谿玖ｪｲ鬘・縺ｮ縺薙°縺�縺Ь:
  - `patterns.md ﾂｧ2.9` 縺ｮ蜿ら・繝代ち繝ｼ繝ｳ縺ｯ縲軍esource IR 邨ｱ蜷亥ｾ後↓螳悟・繧ｵ繝昴・繝医�阪→縺励※菫晉蕗縺ｮ縺ｾ縺ｾ・・hase 4 莉･髯搾ｼ峨�・
  - compare/syntax 縺ｮ繝舌Μ繧｢繝ｳ繝郁ｧ｣豎ｺ隕丞援蟾ｮ蛻・・螳｣險�隕丞援縺悟崋縺ｾ縺｣縺滓悽蝗槭・螟画峩繧貞女縺代※險倩ｼ画ｸ医∩縲・

---

# 2026-03-17 菴懈･ｭ繝｡繝｢ (doc: 隨ｬ2蝗槭Ξ繝薙Η繝ｼ縺ｫ繧医ｋ莉墓ｧ倡｢ｺ螳・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - 隨ｬ2蝗槫､夜Κ繝ｬ繝薙Η繝ｼ縺ｧ謖・遭縺輔ｌ縺溘�悟ｮ溯｣・捩謇句燕縺ｫ蜃咲ｵ舌☆縺ｹ縺堺ｻ墓ｧ倡ｩｴ縲阪ｒ隗｣豸医☆繧九�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `doc/2.1spec/syntax.md`:
    - `while` ﾂｧ8: 縲御ｻ墓ｧ倅ｿ晉蕗縲阪ｒ隗｣豸医�１hase 0窶・ 縺ｯ `unit` 霑斐＠縺ｫ遒ｺ螳壹�１hase 8 縺ｧ縺ｯ `WillExecute` 險ｼ譏惹ｻ倥″縺ｧ譛ｬ菴灘梛 `T` 繧定ｿ斐○繧具ｼ・.2遽�縺ｨ縺励※霑ｽ蜉�・峨�・
    - `<expr>` BNF: `let`/`set` 繧・`unit` 繧定ｿ斐☆蠑上→縺励※蜀咲ｵｱ蜷医�Ａ<stmt>` 繧ｫ繝・ざ繝ｪ繧貞ｻ・ｭ｢縲らｴ皮ｲ九↑蠑乗欠蜷題ｨｭ險医↓邨ｱ荳�縲・
    - borrow 逕滓・蠑・`& <expr>`繝ｻ`&mut <expr>` 繧・`<expr>` 縺ｫ霑ｽ蜉�・亥梛莉墓ｧ倥→縺ｮ謨ｴ蜷茨ｼ峨�・
    - ﾂｧ15 縺ｫ borrow/deref 蟆ら畑遽�繧定ｿｽ蜉�・域ｧ区枚繝ｻ蝙玖ｦ丞援繝ｻ`deref` 蜑咲ｽｮ髢｢謨ｰ縺ｮ菴咲ｽｮ縺･縺托ｼ峨�・
    - `set`/`let` 縺ｮ遽�隕句・縺励ｒ縲梧枚縲阪°繧峨�悟ｼ上�阪↓螟画峩縲・
  - `doc/2.1spec/overview.md`: `while` 隱ｬ譏弱ｒ Phase 0窶・ / Phase 8 縺ｫ蛻・￠縺ｦ譖ｴ譁ｰ縲Ａlet`/`set` 繧ょｼ上→縺励※荳�隕ｧ縺ｫ霑ｽ蜉�縲・
  - `doc/2.1spec/patterns.md`: `let` 縺ｮ隱ｬ譏弱ｒ縲梧枚縲阪°繧峨�蛍nit 繧定ｿ斐☆蠑上�阪↓譖ｴ譁ｰ縲・
  - `doc/2.1spec/traits.md`: `MemReadable`/`MemWritable`/`RegionOwned` 縺ｮ蠑ｷ蛻ｶ繧・Phase 4 莉･髯阪→譏手ｨ倥�・
  - `doc/2.1spec/compiler.md`: 蜷御ｸ翫ｒ trait 蛻ｶ邏・､懈渊遽�縺ｫ繧ょ渚譏�縲・
  - `doc/2.1spec/modules.md`: `#part` 逶ｴ謗･ `use` 繧・warning 縺九ｉ **繧ｳ繝ｳ繝代う繝ｫ繧ｨ繝ｩ繝ｼ** 縺ｫ螟画峩・・anonical path 縺ｨ縺ｮ謨ｴ蜷域�ｧ・峨�・
- [險ｭ險域ｱｺ螳・縺帙▲縺代＞縺代▲縺ｦ縺Ь:
  - `while` 縺ｯ Phase 0窶・ 縺ｧ `unit` 霑斐＠縺ｫ遒ｺ螳壹�ゆｾ晏ｭ伜梛・・hase 8・峨〒 `WillExecute` 險ｼ譏弱ｒ菴ｿ縺・撼 `unit` 繧定ｿ斐○繧九ｈ縺・ｰ・擂諡｡蠑ｵ縺吶ｋ譁ｹ驥昴�・
  - `let`/`set` 縺ｯ縲梧枚縲阪〒縺ｯ縺ｪ縺上�蛍nit 繧定ｿ斐☆蠑上�阪→縺励※蠑冗ｳｻ縺ｫ邨ｱ蜷医�よ枚繝ｻ蠑上・莠悟ｱ､蛻・屬縺ｯ蟒・ｭ｢縲・

---

# 2026-03-17 菴懈･ｭ繝｡繝｢ (doc: 螟夜Κ繝ｬ繝薙Η繝ｼ謖・遭縺ｫ繧医ｋ莉墓ｧ倅ｸ肴紛蜷井ｿｮ豁｣)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - 螟夜Κ繝ｬ繝薙Η繝ｼ・・EPLg2.1 莉墓ｧ倥・荳�雋ｫ諤ｧ逶｣譟ｻ・峨〒謖・遭縺輔ｌ縺・縺､縺ｮ荳肴紛蜷医ｒ菫ｮ豁｣縺吶ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `doc/2.1spec/syntax.md`:
    - ﾂｧ4 縺ｮ `<expr>` BNF 縺九ｉ `let`/`set` 繧帝勁蜴ｻ縺励�～<stmt>` 縺ｨ縺励※迢ｬ遶九＆縺帙◆縲ゅ�梧枚縺ｨ縺励※謇ｱ縺・�阪→繧ｳ繝｡繝ｳ繝医＠縺ｪ縺後ｉ `<expr>` 縺ｮ驕ｸ謚櫁い縺ｫ蜷ｫ繧√※縺・◆遏帷崟繧定ｧ｣豸医�・
    - `<suite>` 螳夂ｾｩ繧定ｿｽ蜉�・医う繝ｳ繝ｩ繧､繝ｳ蠑・縺ｾ縺溘・ 繧､繝ｳ繝・Φ繝医ヶ繝ｭ繝・け・峨�Ａif`/`match`/`while`/繧ｯ繝ｭ繝ｼ繧ｸ繝｣譛ｬ菴薙・譁・ｳ輔ｒ `<block>` 縺九ｉ `<suite>` 縺ｫ螟画峩縺励�～if ge score 90: "A"` 縺ｮ繧医≧縺ｪ繧､繝ｳ繝ｩ繧､繝ｳ蠑上→莉墓ｧ倥・荵夜屬繧定ｧ｣豸医�・
    - ﾂｧ4.1 juxtaposition 縺ｮ縲悟ｷｦ邨仙粋縲崎ｪｬ譏弱ｒ菫ｮ豁｣: 縲掲lat chain 縺ｨ縺励※蜿礼炊縺励�∝梛/arity 縺ｧ蠅・阜豎ｺ螳壹�阪→譏手ｨ倥＠縺溘�・
    - ﾂｧ14.2 縺ｮ繝翫Φ繝舌Μ繝ｳ繧ｰ繝溘せ繧剃ｿｮ豁｣・按ｧ11.2 縺ｫ縺ｪ縺｣縺ｦ縺・◆・峨�・
  - `doc/2.1spec/traits.md`:
    - ﾂｧ3 縺ｫ縲後け繝ｭ繧ｹ繝｢繧ｸ繝･繝ｼ繝ｫ Coherence・・rphan Rule・峨�阪ｒ霑ｽ蜉�縲ょ酔荳�繝｢繧ｸ繝･繝ｼ繝ｫ蜀・・縺ｿ縺ｮ遖∵ｭ｢縺ｧ縺ｯ蛻･繝｢繧ｸ繝･繝ｼ繝ｫ縺九ｉ縺ｮ impl 陦晉ｪ√ｒ髦ｲ縺偵↑縺・◆繧√�・
    - ﾂｧ7 縺ｮ繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝芽ｧ｣豎ｺ萓九↓ `[Phase 8 example]` 豕ｨ險倥ｒ霑ｽ蜉�縲Ａwhere %IsLess idx len` 縺ｯ萓晏ｭ伜梛蟆主・蠕後↓譛牙柑縺ｪ萓九〒縺ゅｊ Phase 0-7 莉墓ｧ倥→豺ｷ蜷後＠縺ｪ縺・ｈ縺・｢・阜繧呈・遉ｺ縲・
  - `doc/2.1spec/patterns.md`:
    - ﾂｧ4.1 match 讒区枚縺ｮ BNF 繧・`<suite>` 縺ｫ蜷医ｏ縺帙※譖ｴ譁ｰ縲・
- [蟾ｮ逡ｰ/縺輔＞]:
  - 縺薙ｌ繧峨・莉墓ｧ倥・霑ｽ蜉�繝ｻ螟画峩縺ｧ縺ｯ縺ｪ縺上�√☆縺ｧ縺ｫ縲後◎縺・〒縺ゅｋ縲阪・縺壹・莠句ｮ溘ｒBNF/螳夂ｾｩ縺ｫ豁｣遒ｺ縺ｫ蜿肴丐縺励◆菫ｮ豁｣縲・
- [谿玖ｪｲ鬘・縺ｮ縺薙°縺�縺Ь:
  - `while` 蠑上・ 0 蝗槫ｮ溯｡梧凾縺ｮ蛟､・按ｧ8 縺ｮ莉墓ｧ倅ｿ晉蕗・峨・譛ｪ隗｣豎ｺ縺ｮ縺ｾ縺ｾ・亥梛螳牙・諤ｧ縺ｨ縺ｮ謨ｴ蜷育｢ｺ隱榊ｾ後↓蛻･騾疲ｱｺ螳夲ｼ峨�・

---

# 2026-03-16 菴懈･ｭ繝｡繝｢ (doc: 莉墓ｧ伜ｮ悟・諤ｧ蜷台ｸ翫・譛ｪ險倩ｼ峨Ν繝ｼ繝ｫ霑ｽ險・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - 逶｣譟ｻ縺ｧ逋ｺ隕九＆繧後◆莉墓ｧ伜ｮ悟・諤ｧ縺ｮ荳崎ｶｳ・域ｼ皮ｮ怜ｭ仙━蜈亥ｺｦ繝ｻ繝ｪ繝・Λ繝ｫ莉墓ｧ倥・borrow 繧ｹ繧ｳ繝ｼ繝励・CTFE蛻ｶ邏・・`pub use` 蠕ｪ迺ｰ繝ｻstdlib螻､蠅・阜繝ｻ繧ｯ繝ｭ繝ｼ繧ｸ繝｣繧ｭ繝｣繝励メ繝｣・峨ｒ霑ｽ險倥☆繧九�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `doc/README.md`: 蟄伜惠縺励↑縺・`stdlib/index.n.md` 縺ｸ縺ｮ繝ｪ繝ｳ繧ｯ繧定ｪｬ譏取ｳｨ險倥↓鄂ｮ謠帙�・
  - `doc/2.1spec/syntax.md`:
    - ﾂｧ10 繧呈ｼ皮ｮ怜ｭ仙━蜈亥ｺｦ繝ｻ邨仙粋諤ｧ縺ｮ荳�隕ｧ陦ｨ・・|>` < juxtaposition < `.field`・峨↓螟画峩縲・
    - ﾂｧ11 縺ｨ縺励※繝ｪ繝・Λ繝ｫ隧ｳ邏ｰ・域紛謨ｰ繝ｻfloat 遘大ｭｦ險俶ｳ・nan/inf繝ｻ譁・ｭ怜・繧ｨ繧ｹ繧ｱ繝ｼ繝励す繝ｼ繧ｱ繝ｳ繧ｹ・峨ｒ霑ｽ蜉�縲・
    - 繧ｯ繝ｭ繝ｼ繧ｸ繝｣繧ｭ繝｣繝励メ繝｣縺ｫ繧ｭ繝｣繝励メ繝｣譎らせ縺ｧ縺ｮ蛟､蝗ｺ螳壹・Owned move 縺ｮ蜍穂ｽ應ｾ九ｒ霑ｽ險倥�・
  - `doc/2.1spec/effects.md`:
    - ﾂｧ3.2.1 縺ｨ縺励※ borrow 繧ｹ繧ｳ繝ｼ繝礼ｵらｫｯ隕丞援・・LL: last-use 縺ｧ邨ゆｺ・ｼ峨ｒ霑ｽ蜉�縲・
  - `doc/2.1spec/modules.md`:
    - `pub use` 蠕ｪ迺ｰ讀懷・・・FS 縺ｫ繧医ｋ繧ｵ繧､繧ｯ繝ｫ讀懷・繝ｻ繧ｳ繝ｳ繝代う繝ｫ繧ｨ繝ｩ繝ｼ・峨ｒ霑ｽ險倥�・
  - `doc/2.1spec/phase8.md`:
    - CTFE 蛻ｶ邏・｡ｨ・・ure繝ｻTotal繝ｻPure Persistent 縺ｮ 3 譚｡莉ｶ・峨ｒ霑ｽ蜉�縲る＆蜿阪さ繝ｼ繝我ｾ九ｂ霑ｽ險倥�・
    - `Partial` 髢｢謨ｰ縺ｮ菴ｿ逕ｨ蜿ｯ蜷ｦ陦ｨ・亥ｮ溯｡梧凾 OK繝ｻ蝙区枚閼医・Pure 譛ｬ菴薙・where 遽�縺ｯ縺吶∋縺ｦ荳榊庄・峨ｒ霑ｽ蜉�縲・
    - 險ｼ譏弱が繝悶ず繧ｧ繧ｯ繝医・譏守､ｺ貂｡縺玲婿驥晢ｼ郁・蜍墓爾邏｢縺励↑縺・炊逕ｱ・峨ｒ霑ｽ險倥�・
  - `doc/2.1spec/stdlib.md`:
    - `alloc` vs `features` 縺ｮ蠅・阜蛻､譁ｭ蝓ｺ貅冶｡ｨ繧定ｿｽ蜉�・・SON/regex/證怜捷 竊・alloc縲；UI/HTTP/TUI 竊・features・峨�・

---

# 2026-03-16 菴懈･ｭ繝｡繝｢ (doc: 蜈ｨ菴謎ｸ�雋ｫ諤ｧ逶｣譟ｻ繝ｻ荳肴紛蜷井ｿｮ豁｣)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - NEPLg2 蝓ｺ譛ｬ逅・ｿｵ・亥燕鄂ｮ險俶ｳ墓峡蠑ｧ縺ｪ縺励・蠑ｷ蜉帙↑髱咏噪讀懈渊繝ｻ蝙句ｮ牙・繝｡繝｢繝ｪ螳牙・繝ｻ萓晏ｭ伜梛貅門ｙ繝ｻ繝槭Ν繝√・繝ｩ繝・ヨ繝輔か繝ｼ繝�・峨′ doc/ 蜈ｨ菴薙↓蠕ｹ蠎輔＆繧後※縺・ｋ縺狗｢ｺ隱阪＠縲∽ｸ肴紛蜷医ｒ菫ｮ豁｣縺吶ｋ縲・
- [隱ｿ譟ｻ邨先棡]:
  - `doc/2.1spec/` 縺ｯ 5 蜴溷援縺吶∋縺ｦ縺ｫ縺､縺・※螳悟・縺ｫ謨ｴ蜷医′蜿悶ｌ縺ｦ縺・ｋ縲・
  - 蝠城｡後・荳ｻ縺ｫ蜻ｨ霎ｺ繝峨く繝･繝｡繝ｳ繝医↓蟄伜惠縺励◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `doc/stdlib_doc_comment_policy.md`:
    - 蟄伜惠縺励↑縺・`doc/purity_ownership_memory_spec.md` 縺ｸ縺ｮ蜿ら・繧・`doc/2.1spec/memory.md ﾂｧ2` 縺ｫ菫ｮ豁｣・育�ｴ謳阪Μ繝ｳ繧ｯ菫ｮ豁｣・峨�・
  - `doc/2.1spec/types.md`:
    - `fn TypeExpr*` 繧・`fn TypeExpr+` 縺ｫ菫ｮ豁｣・亥ｼ墓焚縺ｯ 1 縺､莉･荳翫�ょ・蜉帑ｸ崎ｦ√↑蝣ｴ蜷医・ `fn unit -> T` 繧剃ｽｿ縺・ｼ峨�・
    - `fn -> T`・亥ｼ墓焚繧ｼ繝ｭ・峨→縺・≧蠖｢蠑上ｒ蟒・ｭ｢縺励�～fn unit -> T` 繧呈ｭ｣隕丞ｽ｢縺ｫ邨ｱ荳�縲・
  - `doc/compare/syntax.md`:
    - `() -> i32 竊・fn -> i32` 繧・`fn unit -> i32` 縺ｫ菫ｮ豁｣・域立 `()` = unit 蝙九↑縺ｮ縺ｧ `unit` 縺ｫ蟇ｾ蠢懊＆縺帙ｋ・峨�・
    - `() *> i32 竊・fn* -> i32` 繧・`fn* unit -> i32` 縺ｫ菫ｮ豁｣縲・
  - `doc/lsp_api.md`:
    - 蜀帝�ｭ縺ｫ縲檎樟陦・Bootstrap 螳溯｣・ｼ・EPLg2.0・峨・ API 繧定ｨ倩ｿｰ縲Ａfn` definition kind 縺ｯ NEPLg2.1 縺ｮ `let` 邨ｱ荳�莉墓ｧ倥→逡ｰ縺ｪ繧九�阪→縺・≧豕ｨ諢乗嶌縺阪ｒ霑ｽ蜉�縲・
  - `doc/cli.md`:
    - `--target` 繧ｻ繧ｯ繧ｷ繝ｧ繝ｳ繧定ｿｽ蜉�縺励�～wasm`繝ｻ`wasi`繝ｻ`llvm` 3 繧ｿ繝ｼ繧ｲ繝・ヨ繧定ｨ倩ｼ会ｼ医・繝ｫ繝√ち繝ｼ繧ｲ繝・ヨ蜴溷援縺ｮ蜿肴丐・峨�・
  - `doc/self_host.md`:
    - 謖・､ｺ譁・°繧芽ｨｭ險井ｻ墓ｧ俶枚譖ｸ縺ｸ蜈ｨ髱｢謾ｹ險ゅ�ゆｺ悟ｱ､讒矩��繝ｻ繝・ぅ繝ｬ繧ｯ繝医Μ讒区・繝ｻ繝悶・繝医せ繝医Λ繝・・謇矩�・・繝・せ繝域婿驥昴ｒ險倩ｿｰ縲・
- [譬ｹ諡�/縺薙ｓ縺阪ｇ]:
  - `declarations.md ﾂｧ2.1` 縺ｯ `%fn unit -> T` 繧偵�悟・蜉帑ｸ崎ｦ√↑髢｢謨ｰ縲阪・讓呎ｺ門ｽ｢縺ｨ縺励※菴ｿ縺｣縺ｦ縺・ｋ縺溘ａ縲～fn -> T`・亥ｼ墓焚繧ｼ繝ｭ・峨・荳崎ｦ√・豺ｷ荵ｱ繧呈魚縺上�・

---

# 2026-03-16 菴懈･ｭ繝｡繝｢ (doc: 繧ｵ繧､繝峨ヰ繝ｼ TOC 譛ｨ讒矩��蛹悶・繝・・繝悶Ν繝・じ繧､繝ｳ謾ｹ蝟・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - 蟾ｦ繧ｵ繧､繝峨ヰ繝ｼ縺ｮ TOC 繧呈惠讒矩��・磯嚴螻､陦ｨ遉ｺ・峨↓縺吶ｋ縲・
  - 繝・ヵ繧ｩ繝ｫ繝医〒髢峨§縲∫樟蝨ｨ繝壹・繧ｸ縺ｮ蜈育･悶□縺題・蜍輔〒髢九￥縲る幕髢臥憾諷九ｒ localStorage 縺ｧ繝壹・繧ｸ驕ｷ遘ｻ繧定ｷｨ縺・〒菫晄戟縲・
  - 繝・・繝悶Ν縺ｮ繝・じ繧､繝ｳ繧呈隼蝟・ｼ井ｽ咏區繝ｻ譫�邱夲ｼ峨�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nodesrc/cli.js` (`buildTocEntries` 髢｢謨ｰ):
    - index 縺ｪ縺励・ flat fallback 縺ｧ縲√ョ繧｣繝ｬ繧ｯ繝医Μ・育ｬｬ荳�繝代せ繧ｻ繧ｰ繝｡繝ｳ繝茨ｼ峨＃縺ｨ縺ｫ繧ｰ繝ｫ繝ｼ繝怜喧縺吶ｋ繧医≧縺ｫ螟画峩縲・
    - `isGroup: true` + `depth: 0` 縺ｮ繧ｨ繝ｳ繝医Μ繧偵げ繝ｫ繝ｼ繝励→縺励※謖ｿ蜈･縺励�・・荳九Μ繝ｳ繧ｯ繧・`depth: 1` 縺ｫ縲・
  - `nodesrc/html_gen_playground.js`:
    - `buildTocTree()`: flat 縺ｪ tocLinks 驟榊・繧呈ｷｱ縺輔・繝ｼ繧ｹ縺ｮ譛ｨ讒矩��縺ｫ螟画鋤縲・
    - `renderTocTree()`: 譛ｨ讒矩��繧・`<details>`/`<summary>` HTML 縺ｫ螟画鋤・医げ繝ｫ繝ｼ繝励・謚倥ｊ縺溘◆縺ｿ蜿ｯ・峨�・
    - `renderTocItems()`: 荳願ｨ・2 髢｢謨ｰ繧剃ｽｿ縺・ｈ縺・嶌縺咲峩縺励�・
  - `nodesrc/static/playground_runtime.js`:
    - `initTocState()` 髢｢謨ｰ繧定ｿｽ蜉�・・injectUI()` 逶ｴ蠕後↓蜻ｼ縺ｳ蜃ｺ縺暦ｼ峨�・
    - localStorage (`nepl-toc-open`) 縺九ｉ髢矩哩迥ｶ諷九ｒ蠕ｩ蜈・・驕ｩ逕ｨ竊偵い繧ｯ繝・ぅ繝悶Μ繝ｳ繧ｯ縺ｮ蜈育･悶ｒ蠑ｷ蛻ｶ open 竊蛋toggle` 繧､繝吶Φ繝医〒迥ｶ諷九ｒ菫晏ｭ倥�・
  - `nodesrc/static/playground.css`:
    - Tree TOC 逕ｨ繧ｹ繧ｿ繧､繝ｫ霑ｽ蜉�: `.toc-item`, `.toc-item-group`, `.toc-group-details`, `.toc-group-summary`, `.toc-sublist`縲・
    - 繝・・繝悶Ν繝・じ繧､繝ｳ謾ｹ蝟・ `.nm-table-wrap`・・order/radius/overflow・峨�～.nm-table`・医そ繝ｫ padding繝ｻ陦後・繝舌・・峨�・
    - blockquote/image/strong/em/del 縺ｮ繧ｹ繧ｿ繧､繝ｫ繧りｿｽ蜉�縲・

---

# 2026-03-16 菴懈･ｭ繝｡繝｢ (nodesrc: TypeScript 繧ｳ繝ｳ繝代う繝ｫ蜃ｺ蜉帙ｒ gitignore)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `nodesrc/parser.js` / `nodesrc/html_gen.js` 縺ｯ `tsc` 縺ｫ繧医ｋ繧ｳ繝ｳ繝代う繝ｫ蜃ｺ蜉帙〒縺ゅｊ縲“it 縺ｧ邂｡逅・☆縺ｹ縺阪〒縺ｪ縺・�Ｈitignore 縺ｫ霑ｽ蜉�縺励※ untrack 縺吶ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `.gitignore`: `/nodesrc/parser.js`, `/nodesrc/html_gen.js` 繧定ｿｽ蜉�縲・
  - `git rm --cached` 縺ｧ譌｢蟄倥・霑ｽ霍｡繧定ｧ｣髯､縲・
  - CI 縺ｯ縺吶〒縺ｫ bootstrap-build 縺ｧ `tsc` 繧貞ｮ溯｡後☆繧九◆繧√�「ntrack 縺励※繧ょ撫鬘後↑縺励�・

---

# 2026-03-16 菴懈･ｭ繝｡繝｢ (nodesrc: TypeScript 蛹悶・Markdown 諡｡蠑ｵ蟇ｾ蠢・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `nodesrc/parser.js` / `nodesrc/html_gen.js` 繧・TypeScript 縺ｧ譖ｸ縺咲峩縺励�～doc/` 縺ｧ菴ｿ逕ｨ縺輔ｌ縺ｦ縺・ｋ Markdown 險俶ｳ包ｼ・able縲・*bold**縲・italic*縲×~strikethrough~~縲｜lockquote縲｛rdered list・峨↓蟇ｾ蠢懊☆繧九�・
  - 螟夜Κ繝ｩ繧､繝悶Λ繝ｪ繧剃ｽｿ逕ｨ縺帙★繧ｻ繝ｫ繝募ｮ溯｣・�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nodesrc/parser.ts` (譁ｰ隕丈ｽ懈・):
    - `parser.js` 繧・TypeScript 縺ｧ螳悟・縺ｫ譖ｸ縺咲峩縺励◆縲・
    - 蝙句ｮ夂ｾｩ: `InlineNode`・・strong`, `em`, `strike`, `image` 繧定ｿｽ蜉�・峨�～BlockNode`・・table`, `blockquote` 繧定ｿｽ蜉�・峨�・
    - `parseInlines`: `**bold**`, `*italic*`, `~~strike~~`, `![img](src)` 繧偵し繝昴・繝医�・
    - `parseNmdAstFromLines`: table・・| ... | ... |` 蠖｢蠑擾ｼ峨�｜lockquote・・>` 陦鯉ｼ峨�｛rdered list・・1. item`・峨ｒ繧ｵ繝昴・繝医�・
    - doctest 謚ｽ蜃ｺ繝ｭ繧ｸ繝・け縺ｯ螟画峩縺ｪ縺暦ｼ井ｺ呈鋤諤ｧ邯ｭ謖・ｼ峨�・
  - `nodesrc/html_gen.ts` (譁ｰ隕丈ｽ懈・):
    - `html_gen.js` 繧・TypeScript 縺ｧ螳悟・縺ｫ譖ｸ縺咲峩縺励◆縲・
    - `renderInlines`: `strong`竊蛋<strong>`, `em`竊蛋<em>`, `strike`竊蛋<del>`, `image`竊蛋<img>` 繧定ｿｽ蜉�縲・
    - `renderNode`: `table`竊蛋<table>`・・head/tbody/align 蟇ｾ蠢懶ｼ峨�～blockquote`竊蛋<blockquote>` 繧定ｿｽ蜉�縲・
    - `list`: `ordered: true` 縺ｧ `<ol>` 繧剃ｽｿ逕ｨ縲・
    - CSS: table/blockquote/image/em/strong/del 縺ｮ繧ｹ繧ｿ繧､繝ｫ繧定ｿｽ蜉�縲・
  - `nodesrc/tsconfig.json` (譁ｰ隕丈ｽ懈・):
    - `parser.ts`, `html_gen.ts` 繧・`nodesrc/` 蜀・〒 `parser.js`, `html_gen.js` 縺ｫ繧ｳ繝ｳ繝代う繝ｫ縲・
    - `web/node_modules/@types/node` 繧・typeRoots 縺ｨ縺励※蜿ら・縲・
  - `.github/actions/bootstrap-build/action.yml` (譖ｴ譁ｰ):
    - `web/node_modules/.bin/tsc -p nodesrc/tsconfig.json` 繧・CI 繧ｹ繝・ャ繝励↓霑ｽ蜉�縲・
  - `CLAUDE.md` (譖ｴ譁ｰ):
    - 縲御ｽ懈･ｭ縺ｮ蛹ｺ蛻・ｊ縺ｧ繧ｳ繝溘ャ繝医☆繧九％縺ｨ縲阪�後さ繝溘ャ繝亥燕縺ｫ note.n.md 繧呈峩譁ｰ縺吶ｋ縺薙→縲阪ｒ髢狗匱繧ｬ繧､繝峨Λ繧､繝ｳ縺ｫ霑ｽ險倥�・
- [譁ｹ驥・縺ｻ縺・＠繧転:
  - `.ts` 繝輔ぃ繧､繝ｫ縺後た繝ｼ繧ｹ縲～.js` 縺後さ繝ｳ繝代う繝ｫ蜃ｺ蜉帙�ゅさ繝ｳ繝代う繝ｫ貂医∩ `.js` 縺ｯ git 縺ｫ蜷ｫ繧√ｋ縲・
  - CI 縺ｯ `web/node_modules/.bin/tsc` 繧剃ｽｿ逕ｨ縺励※繧ｳ繝ｳ繝代う繝ｫ・郁ｿｽ蜉�繧､繝ｳ繧ｹ繝医・繝ｫ荳崎ｦ・ｼ峨�・

---

# 2026-03-16 菴懈･ｭ繝｡繝｢ (NEPLg2.1 蜻ｽ蜷阪・蝙玖ｨ俶ｳ穂ｻ墓ｧ倡｢ｺ螳壹・fn蟒・ｭ｢)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - 蝙玖ｨ俶ｳ輔・螟ｧ蟷・､画峩・域峡蠑ｧ螳悟・蟒・ｭ｢繝ｻkind-directed juxtaposition繝ｻ`%` 繧｢繝弱ユ繝ｼ繧ｷ繝ｧ繝ｳ繝ｻ`unit` 繧ｭ繝ｼ繝ｯ繝ｼ繝峨・`fn` 螳｣險�繧ｭ繝ｼ繝ｯ繝ｼ繝牙ｻ・ｭ｢・峨ｒ蜿肴丐縺励◆譁ｰ莉墓ｧ倥ｒ **NEPLg2.1** 縺ｨ蜻ｽ蜷阪＠縲¨EPLg2・育樟陦悟ｮ溯｣・ｼ峨→譏守｢ｺ縺ｫ蛹ｺ蛻･縺吶ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `doc/type_notation_spec.md` (譖ｴ譁ｰ):
    - 諡ｬ蠑ｧ螳悟・蟒・ｭ｢繝ｻ繧ｰ繝ｫ繝ｼ繝怜喧讒区枚縺ｪ縺励�・
    - 蝙矩←逕ｨ繧・juxtaposition 縺ｫ螟画峩・・Name<A B>` 竊・`Name A B`・峨�〔ind-directed 繧｢繝ｫ繧ｴ繝ｪ繧ｺ繝�縺ｧ蠅・阜豎ｺ螳壹�・
    - unit 蝙九ｒ `unit` 繧ｭ繝ｼ繝ｯ繝ｼ繝峨↓螟画峩・・()` 蟒・ｭ｢・峨�・
    - 蝙区ｳｨ驥郁ｨ伜捷繧・`<TypeExpr>` 縺九ｉ `%TypeExpr` 縺ｫ螟画峩縲・
    - 蝙九ヱ繝ｩ繝｡繝ｼ繧ｿ螳｣險�縺ｮ `<>` 繧貞ｻ・ｭ｢・・.T .U` 縺ｨ縺励※蛻玲嫌・峨�・
    - `fn` 螳｣險�繧ｭ繝ｼ繝ｯ繝ｼ繝峨ｒ蟒・ｭ｢・育炊逕ｱ: 蝙玖ｨ俶ｳ輔↓ `%fn ...` 縺檎樟繧後ｋ縺溘ａ邏帙ｉ繧上＠縺・ｼ峨�ょ・髢｢謨ｰ螳夂ｾｩ繧・`let name %fn ...` 縺ｫ邨ｱ荳�縲ょｷｻ縺堺ｸ翫￡縺ｯ `let` 縺ｮ蝙九′ `fn`/`fn*` 縺ｮ蝣ｴ蜷医↓驕ｩ逕ｨ縲・
  - `doc/pattern_spec.md`縲～doc/module_system_spec.md`縲～doc/language_platform_spec.md`縲～doc/purity_ownership_memory_spec.md`:
    - 繧ｿ繧､繝医Ν繧・NEPLg2.1 縺ｫ譖ｴ譁ｰ縲・
  - `doc/dependent_type_proof_plan.md`縲～doc/memory_safety_migration_plan.md`縲～doc/module_system_spec.md`:
    - `fn` 螳｣險�繧・`let` 縺ｫ譖ｴ譁ｰ縲∝梛豕ｨ驥医ｒ譁ｰ險俶ｳ輔↓譖ｴ譁ｰ縲・
  - `CLAUDE.md` (譖ｴ譁ｰ):
    - NEPLg2・育樟陦悟ｮ溯｣・ｼ峨→ NEPLg2.1・域眠莉墓ｧ假ｼ峨・蛹ｺ蛻･繧呈・險倥�・
- [譁ｹ驥・縺ｻ縺・＠繧転:
  - `nepl-core/`・・ust 螳溯｣・ｼ峨・蠑輔″邯壹″ NEPLg2 縺ｮ螳溯｣・�・EPLg2.1 縺ｮ螳溯｣・・蛻･騾皮ｧｻ陦瑚ｨ育判縺ｧ騾ｲ繧√ｋ縲・
  - `plan.md` 縺ｯ蜿､縺・NEPLg2 莉墓ｧ倥〒縺ゅｊ螟画峩縺励↑縺・ｼ亥盾辣ｧ逕ｨ縺ｨ縺励※菫晄戟・峨�・

---

# 2026-03-16 菴懈･ｭ繝｡繝｢ (doc: 莉墓ｧ俶紛蜷育｢ｺ隱阪・繝｢繧ｸ繝･繝ｼ繝ｫ/繝代ち繝ｼ繝ｳ/CLAUDE.md 譖ｴ譁ｰ)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `doc/chat/dump/` 縺ｮ譛�譁ｰ譁ｹ驥晢ｼ・ang1.md, mem1.md, module1.md・峨→ `doc/` 蜷・ｻ墓ｧ倥♀繧医・ `todo.md` 縺ｫ鮨滄ｽｬ縺後↑縺・％縺ｨ繧堤｢ｺ隱阪＠縲∵悴險倩ｼ峨・險ｭ險域ｱｺ螳壹ｒ莉墓ｧ俶嶌縺ｸ蜿肴丐縺吶ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `CLAUDE.md` (譁ｰ隕丈ｽ懈・):
    - 繝薙Ν繝峨・繝・せ繝医・繧｢繝ｼ繧ｭ繝・け繝√Ε繝ｻ髢狗匱繧ｬ繧､繝峨Λ繧､繝ｳ繧偵∪縺ｨ繧√◆蛻晄悄 CLAUDE.md 繧剃ｽ懈・縲・
    - `.n.md`・・M 諡｡蠑ｵ Markdown: 繝輔Μ繧ｬ繝翫・gloss繝ｻNest 縺御ｽｿ縺医ｋ・峨→騾壼ｸｸ `.md` 縺ｮ驕輔＞繧呈・險倥�ゆｻ墓ｧ伜盾辣ｧ蜈医→縺励※ `stdlib/nm/README.n.md` 繧堤､ｺ縺励◆縲・
  - `doc/module_system_spec.md` (譖ｴ譁ｰ):
    - `use` 縺ｮ讒区枚繧・`::` 繧ｻ繝代Ξ繝ｼ繧ｿ蠖｢蠑上↓螟画峩・・use core::math;` 遲会ｼ峨�・
    - `use` 縺梧忰蟆ｾ繧ｻ繧ｰ繝｡繝ｳ繝医・繧ｨ繧､繝ｪ繧｢繧ｹ繧貞ｰ主・縺吶ｋ縺薙→繧呈・險倥�・
    - `*` 縺ｯ繝｢繧ｸ繝･繝ｼ繝ｫ縺ｸ縺ｮ `use` 縺ｫ縺ｮ縺ｿ譛牙柑縲・未謨ｰ遲峨∈縺ｮ `::*` 縺ｯ繧ｨ繝ｩ繝ｼ縺ｨ縺励※螳夂ｾｩ縲・
    - `merge "path"` 縺ｯ繝輔ぃ繧､繝ｫ繝代せ譁・ｭ怜・繧貞叙繧具ｼ・""` 邯ｭ謖・ｼ峨％縺ｨ繧呈・險倥�∵ｧ区枚萓九ｒ霑ｽ蜉�縲・
  - `doc/purity_ownership_memory_spec.md` (譖ｴ譁ｰ):
    - 縲景mmutable tuple縲阪ｒ縲景mmutable struct・・Pair`, `Triple` 遲会ｼ峨�阪↓鄂ｮ縺肴鋤縺茨ｼ・uple 蟒・ｭ｢縺ｫ蟇ｾ蠢懶ｼ峨�・
  - `doc/pattern_spec.md` (譁ｰ隕丈ｽ懈・):
    - 險�隱樒ｵ・∩霎ｼ縺ｿ `Tuple` 繧ｭ繝ｼ繝ｯ繝ｼ繝峨ｒ蟒・ｭ｢縺励�～Pair<.A,.B>` / `Triple<.A,.B,.C>` 繧・stdlib 縺ｮ騾壼ｸｸ struct 縺ｨ縺励※謠蝉ｾ帙☆繧九％縺ｨ繧貞ｮ夂ｾｩ縲・
    - Rust 逶ｸ蠖薙・鬮俶ｩ溯・繝代ち繝ｼ繝ｳ莉墓ｧ倥ｒ遲門ｮ・ 隴伜挨蟄舌・繝ｯ繧､繝ｫ繝峨き繝ｼ繝峨・繝ｪ繝・Λ繝ｫ繝ｻ遽・峇・域ｧ区枚譛ｪ遒ｺ螳夲ｼ峨・繧ｳ繝ｳ繧ｹ繝医Λ繧ｯ繧ｿ・井ｽ咲ｽｮ繝吶・繧ｹ・峨・繝阪せ繝医・`@` 譚溽ｸ帑ｻ倥″繝ｻOR 繝代ち繝ｼ繝ｳ・・|`縲√ヱ繧ｿ繝ｼ繝ｳ蟆ら畑・峨・蜿ら・繝代ち繝ｼ繝ｳ・亥ｰ・擂・峨�・
    - `let <pattern> <expr>` 縺翫ｈ縺ｳ `match` 蠑上〒縺ｮ繝代ち繝ｼ繝ｳ菴ｿ逕ｨ莉墓ｧ倥�∫ｶｲ鄒・�ｧ讀懈渊縲∵園譛画ｨｩ縺ｨ縺ｮ邨ｱ蜷医ｒ螳夂ｾｩ縲・
    - 蜈ｨ繧ｳ繝ｼ繝我ｾ九ｒ NEPLg2 蜑咲ｽｮ險俶ｳ輔↓貅匁侠縺輔○縺滂ｼ域峡蠑ｧ繧剃ｽｿ繧上★縲∽ｸｭ蛟､貍皮ｮ怜ｭ舌ｒ逕ｨ縺・↑縺・ｼ峨�・
    - 蝙句燕鄂ｮ險俶ｳ慕｢ｺ螳壹・蜈磯�√ｊ縺�縺悟ｯｾ蠢懷庄閭ｽ縺ｪ險ｭ險医〒縺ゅｋ縺薙→繧呈・險倥�・
- [遒ｺ隱・縺九￥縺ｫ繧転:
  - dump 繝輔ぃ繧､繝ｫ 3 譛ｬ (lang1, mem1, module1) 縺ｨ蟇ｾ蠢懊☆繧・doc/ 莉墓ｧ倥・todo.md 繧堤・蜷医＠縺溽ｵ先棡縲∫泝逶ｾ縺ｯ隕句ｽ薙◆繧峨↑縺九▲縺溘�・
  - todo.md 縺ｮ縲鍬LM 邱ｨ髮・ｦ∵ｭ｢縲阪そ繧ｯ繧ｷ繝ｧ繝ｳ縺ｫ縺ゅｋ Tuple/Pair/Triple縲∝梛蜑咲ｽｮ險俶ｳ募喧縲√ヱ繧ｿ繝ｼ繝ｳ險ｭ險医・莉雁屓縺ｮ doc/ 譖ｴ譁ｰ縺ｧ莉墓ｧ倥→縺励※蜿肴丐縺励◆縲・
  - `use` 繧ｹ繧ｳ繝ｼ繝怜ｰ主・縺ｮ隧ｳ邏ｰ・・lias vs 逶ｴ謗･ import縲～as *` 縺ｮ謇ｱ縺・ｼ峨・莉雁屓縺ｮ module_system_spec.md 譖ｴ譁ｰ縺ｧ遒ｺ螳壹＆縺帙◆縲・

---

# 2026-03-16 菴懈･ｭ繝｡繝｢ (doc: 繝｢繧ｸ繝･繝ｼ繝ｫ繧ｷ繧ｹ繝・Β繝ｻ險�隱槭・繝ｩ繝・ヨ繝輔か繝ｼ繝�莉墓ｧ倥・遲門ｮ壹・逶｣譟ｻ螳御ｺ・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `doc/chat/dump/lang1.md`, `module1.md` 縺ｮ隴ｰ隲悶ｒ謨ｴ逅・＠縲¨EPLg2 縺ｮ繝｢繧ｸ繝･繝ｼ繝ｫ繧ｷ繧ｹ繝・Β縺ｨ險�隱槭・繝ｩ繝・ヨ繝輔か繝ｼ繝�縺ｨ縺励※縺ｮ蜈ｨ菴灘ワ繧呈ｭ｣蠑上↑莉墓ｧ俶嶌縺ｨ縺励※譏取枚蛹悶☆繧九�・
  - `todo.md` 縺ｫ縺翫￠繧九�√ヵ繧｡繧､繝ｫ蠅・阜縺ｨ繝｢繧ｸ繝･繝ｼ繝ｫ蠅・阜縺ｮ蛻・屬縲√♀繧医・繧ｻ繝ｫ繝輔・繧ｹ繝医↓蜷代￠縺溘Ξ繧､繝､繝ｼ讒矩��縺ｮ繧ｿ繧ｹ繧ｯ繧貞・菴灘喧縺吶ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `doc/module_system_spec.md` (譁ｰ隕丈ｽ懈・):
    - 繝輔ぃ繧､繝ｫ縺ｨ繝｢繧ｸ繝･繝ｼ繝ｫ縺ｮ逶ｴ莠､諤ｧ縲～merge` (繧ｽ繝ｼ繧ｹ蜷域・) 縺ｨ `use` (萓晏ｭ倩ｧ｣豎ｺ) 縺ｮ菴ｿ縺・・縺代�、nchor Part 縺ｫ繧医ｋ canonical path 豎ｺ螳夊ｦ丞援繧貞ｮ夂ｾｩ縲・
  - `doc/language_platform_spec.md` (譁ｰ隕丈ｽ懈・):
    - DSL 螳溯｡悟渕逶､縺ｨ縺励※縺ｮ繝薙ず繝ｧ繝ｳ縲。ootstrap Host (Rust) 縺ｨ Platform Stdlib (NEPL) 縺ｮ 2 螻､讒矩��縲《tdlib 縺ｮ髫主ｱ､蛹・(`core`/`alloc`/`runtimes`/`std`/`features`) 繧貞ｮ夂ｾｩ縲・
  - `todo.md`:
    - 縲・. Module System 螳溯｣・→蜷榊燕隗｣豎ｺ縺ｮ蛻ｷ譁ｰ (Migration Phase 0.5)縲阪ｒ霑ｽ蜉�縲・
    - 繧ｻ繝ｫ繝輔・繧ｹ繝医さ繝ｳ繝代う繝ｩ鬆・岼縺ｮ螳御ｺ・擅莉ｶ繧偵�√・繝ｩ繝・ヨ繝輔か繝ｼ繝�讒矩��縺ｮ螳夂ｾｩ縺ｫ蜷医ｏ縺帙※鬮伜ｺｦ蛹悶�・
- [邨先棡/縺代▲縺犠:
  - 縺薙ｌ縺ｫ繧医ｊ縲¨EPLg2 縺後�悟腰縺ｪ繧玖ｨ�隱槭�阪〒縺ｯ縺ｪ縺上�瑚ｨ�隱槭・繝ｩ繝・ヨ繝輔か繝ｼ繝�縲阪〒縺ゅｋ縺ｨ縺・≧遶九■菴咲ｽｮ縺梧・遒ｺ蛹悶＆繧後�∝､壹ヵ繧｡繧､繝ｫ讒区・譎ゅ・蜷榊燕隗｣豎ｺ縺ｮ荳咲｢ｺ螳滓�ｧ縺梧鴛諡ｭ縺輔ｌ縺溘�Ａtodo.md` 縺ｫ蝓ｺ縺･縺阪�∵ｬ｡縺ｯ繝代・繧ｵ縺ｨ繝ｬ繧ｾ繝ｫ繝舌・蛻ｷ譁ｰ縺ｫ逹�謇九☆繧句悄蜿ｰ縺梧紛縺｣縺溘�・

---

# 2026-03-15 菴懈･ｭ繝｡繝｢ (doc: 蜈ｨ繝峨く繝･繝｡繝ｳ繝医・譛�譁ｰ莉墓ｧ倥∈縺ｮ霑ｽ蠕薙・逶｣譟ｻ螳御ｺ・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `doc/` 莉･荳九・縺吶∋縺ｦ縺ｮ莉墓ｧ倥ｄ險育判 (`plan.md`, `todo.md` 繧貞性繧�) 縺ｨ譛�譁ｰ縺ｮ螳溯｣・憾豕√ｒ邊ｾ譟ｻ縺励�¨EPLg2 縺ｮ逶ｮ讓吶→譁ｰ縺溘↓遲門ｮ壹＠縺溷ｮ牙・譁ｽ遲・(`purity_ownership_memory_spec.md`, `memory_safety_migration_plan.md`) 縺ｨ縺ｮ髢薙〒鮨滄ｽｬ縺後↑縺・ｈ縺・↓邨ｱ荳�繧貞峙繧九�・
- [螟画峩縺ｨ逶｣譟ｻ邨先棡/縺ｸ繧薙％縺・→縺九ｓ縺輔￠縺｣縺犠:
  - `plan.md`:
    - 譁・ｭ怜・ (`str`, `ByteBuf`, `StringBuilder`) 縺ｮ險倩ｿｰ繧呈峩譁ｰ縺励�∵立蠑上・蛟溽畑繝薙Η繝ｼ繧・`String` 蝙九∈縺ｮ險�蜿翫ｒ蜑企勁縲・貂・
  - `doc/runtime.md`:
    - GC縺ｪ縺励・繝｡繝｢繝ｪ邂｡逅・↓縺､縺・※縲∵焔蜍・`alloc/dealloc` 繝吶・繧ｹ縺ｮ蜿､縺・ｪｬ譏弱ｒ蜑企勁縺励�・*Region Inference (邏皮ｲ区ｰｸ邯壼�､)** 縺ｨ **Drop Elaboration (荳�諢乗園譛峨Μ繧ｽ繝ｼ繧ｹ)** 縺ｮ莠梧ｮｵ讒九∴繝｢繝・Ν縺ｫ譖ｸ縺肴鋤縺医◆縲・
    - Wasm/LLVM 縺ｮ繝ｩ繝ｳ繧ｿ繧､繝�蟾ｮ蛻・・ `#if[target=...]` 縺ｧ蜷ｸ蜿弱＆繧後�√さ繝ｳ繝代う繝ｩ縺ｮ螳牙・諢丞袖隲悶・蜈ｱ騾壹〒縺ゅｋ譌ｨ繧呈・險倥＠縺溘�・
  - `doc/error.md`:
    - 譌ｧ蠑上・繝偵・繝礼｢ｺ菫晏燕謠舌〒縺ゅｋ `Error` 繝ｬ繧ｳ繝ｼ繝峨・隱ｬ譏弱ｒ蜑企勁縺励�∵怙譁ｰ縺ｮ `Diag`縲～Outcome<T, E>`縲～Result<T, StdErrorKind>` 繧呈�ｸ縺ｨ縺吶ｋ繧ｨ繝ｩ繝ｼ繝｢繝・Ν縺ｫ譖ｴ譁ｰ縺励◆縲ゅΓ繝｢繝ｪ縺ｮ遒ｺ菫昴→隗｣謾ｾ縺ｯGC繧・焔蜍・`alloc` 縺ｧ縺ｯ縺ｪ縺上�∵眠縺励＞謇�譛画ｨｩ繝｢繝・Ν縺ｫ蟋斐・繧峨ｌ繧区葎繧定ｨ倩ｼ峨＠縺溘�・
  - `doc/move_effect_spec.md` & `doc/memory_safety_compiler_design.md` & `doc/stdlib_breaking_reboot.md`:
    - 縺吶〒縺ｫ邨ｱ蜷井ｻ墓ｧ倥ｒ蜿肴丐貂医∩縺ｧ縺ゅｊ縲∝・螳ｹ縺ｫ遏帷崟縺後↑縺・％縺ｨ繧堤｢ｺ隱阪＠縺溘�・
  - `todo.md`:
    - 縲後Γ繝｢繝ｪ螳牙・蝙九Δ繝・Ν繧堤ｵｱ蜷井ｻ墓ｧ倥↓蝓ｺ縺･縺・※螳溯｣・☆繧九�阪・繧ｿ繧ｹ繧ｯ・・hase 1: Effect諡｡蠑ｵ縺ｨVarState霑ｽ蜉�縲￣hase 2: 蝙句・髮｢縺ｨRegion謗ｨ隲厄ｼ峨′隧ｳ邏ｰ縺ｫ險倩ｼ峨＆繧後※縺翫ｊ縲∝ｮ溯｣・憾豕√♀繧医・險育判縺ｨ螳悟・縺ｫ荳�閾ｴ縺励※縺・ｋ縺薙→繧堤｢ｺ隱阪＠縺溘�・
- [邨占ｫ・縺代▽繧阪ｓ]:
  - 縺薙ｌ縺ｫ繧医ｊ縲¨EPLg2 縺ｫ縺翫￠繧狗ｴ皮ｲ区�ｧ繝ｻ謇�譛画ｨｩ繝ｻ繝｡繝｢繝ｪ邂｡逅・・譬ｹ蟷ｹ縺ｨ縺ｪ繧九ラ繧ｭ繝･繝｡繝ｳ繝医→螳溯｣・ｨ育判縺悟ｮ悟・縺ｫ荳�轤ｹ縺ｫ邨ｱ蜷医・謨ｴ逅・＆繧後�√☆縺ｹ縺ｦ縺ｮ蜿､縺・GC/謇句虚隗｣謾ｾ繝吶・繧ｹ縺ｮ險倩ｿｰ縺梧鴛諡ｭ縺輔ｌ縺溘�ゆｻ･蠕後・縺薙・繝峨く繝･繝｡繝ｳ繝育ｾ､縺翫ｈ縺ｳ `todo.md` 縺ｮ Phase 1 / 2 縺ｫ蜑・▲縺ｦ繧ｳ繝ｳ繝代う繝ｩ縺ｨ讓呎ｺ悶Λ繧､繝悶Λ繝ｪ縺ｮ螳溯｣・ｒ螳牙・縺ｫ騾ｲ繧√ｋ縺薙→縺後〒縺阪ｋ縲・

---

# 2026-03-15 菴懈･ｭ繝｡繝｢ (doc: NEPLg2逶ｮ讓吶→譁ｰ莉墓ｧ倥・謨ｴ蜷域�ｧ讀懆ｨ・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - Zenn險倅ｺ具ｼ・EPLg2縺ｮ豁ｴ蜿ｲ縺ｨ險ｭ險域�晄Φ・峨・逶ｮ讓吶↓蟇ｾ縺励�∽ｽ懈・縺励◆Purity繝ｻOwnership繝ｻMemory莉墓ｧ倥→螳溯｣・ｨ育判縺ｧ蛻ｰ驕泌庄閭ｽ縺九ｒ豺ｱ縺乗､懆ｨ弱＠縲∽ｸ崎ｶｳ縺後≠繧後・菫ｮ豁｣縺吶ｋ縲・
- [讀懆ｨ惹ｺ矩�・→邨先棡/縺代ｓ縺ｨ縺・§縺薙≧縺ｨ縺代▲縺犠:
  1. **繝槭Ν繝√・繝ｩ繝・ヨ繝輔か繝ｼ繝�縺ｨ蜷檎ｭ峨・邨先棡・・asm/LLVM縺ｮ謚ｽ雎｡蛹厄ｼ・*:
     - **邨先棡**: 驕疲・蜿ｯ閭ｽ縲らｵｱ蜷井ｻ墓ｧ伉ｧ12 縺翫ｈ縺ｳ 遘ｻ陦瑚ｨ育判ﾂｧ10.8 縺ｫ繧医ｊ縲ヽesource IR 繝代せ縺ｧ縲悟ｮ牙・諢丞袖隲悶�阪ｒ螳悟・縺ｫ菫晁ｨｼ縺励�…odegen 繝輔ぉ繝ｼ繧ｺ縺ｧ縺ｯ迚ｩ逅・Ξ繧､繧｢繧ｦ繝茨ｼ・inear memory vs Native pointer・峨・驕輔＞縺ｮ縺ｿ繧貞精蜿弱☆繧玖ｨｭ險医↓縺ｪ縺｣縺ｦ縺・ｋ縲・enn險倅ｺ九・逶ｮ讓吶→螳悟・縺ｫ荳�閾ｴ縺吶ｋ縲・
  2. **閾ｪ菴懆ｨ�隱槭・繝ｩ繝・ヨ繝輔か繝ｼ繝�縺ｨ繧ｻ繝ｫ繝輔・繧ｹ繝茨ｼ医さ繝ｳ繝代う繝ｩ繧呈嶌縺代ｋ險�隱槭°・・*:
     - **邨先棡**: 驕疲・蜿ｯ閭ｽ縺�縺御ｸ�驛ｨ莉墓ｧ倥↓譏手ｨ倥′蠢・ｦ√□縺｣縺溘◆繧∽ｿｮ豁｣縺励◆縲ゅさ繝ｳ繝代う繝ｩ・・ST繧・腸蠅・ｼ峨ｒ螳溯｣・☆繧九↓縺ｯ縲∬､・尅縺ｪ繝・・繧ｿ讒矩��繧帝�壹§縺溯・蜉帙・莨晄眺縺悟ｿ・ｦ√�・
     - **菫ｮ豁｣**: `purity_ownership_memory_spec.md` 縺ｫ **縲・.3 蝙九・閭ｽ蜉帙・蜷域・蜑・�・* 繧定ｿｽ蜉�縲・ST・・mmutable tree・峨・ pure persistent縲∫腸蠅・ｄBuilder縺ｯ UniqueMutable縲・幕縺・◆繝輔ぃ繧､繝ｫ繧貞性繧�讒矩��菴薙・ LinearCapability 縺ｨ縺ｪ繧九ｈ縺・�∬､・粋蝙九・閭ｽ蜉帑ｼ晄眺繝ｫ繝ｼ繝ｫ繧呈・譁・喧縺励�√さ繝ｳ繝代う繝ｩ險倩ｿｰ縺ｫ閠舌∴縺・ｋ蝣・欧縺ｪ蝙九す繧ｹ繝・Β險ｭ險医ｒ遒ｺ遶九＠縺溘�・
  3. **蜊泌鴨縺ｪ髱咏噪讀懈渊縺ｨ諡ｬ蠑ｧ縺ｮ譬ｹ邨ｶ**:
     - **邨先棡**: 驕疲・蜿ｯ閭ｽ縲３esource IR 繧堤畑縺・◆ Dataflow 隗｣譫・(use-after-move, borrow conflict, linear 貍上ｌ遲峨・讀懈渊) 縺ｯ Zenn險倅ｺ九・縲悟ｼｷ蜉帙↑讀懈渊陬・ｽｮ縲阪↓逶ｴ謗･雋｢迪ｮ縺吶ｋ縲よｧ区枚逧・音蠕ｴ・亥燕鄂ｮ險俶ｳ輔・繧ｪ繝輔し繧､繝峨Ν繝ｼ繝ｫ・峨→縺ｯ迢ｬ遶九＠縺・IR 螻､縺ｧ縺ｮ讀懈渊縺ｧ縺ゅｋ縺溘ａ縲∵峡蠑ｧ縺ｮ譬ｹ邨ｶ逶ｮ讓吶→繧り｡晉ｪ√＠縺ｪ縺・�・
  4. **譌｢蟄倥ラ繧ｭ繝･繝｡繝ｳ繝医・遏帷崟隗｣豸・*:
     - **邨先棡**: `plan.md` 縺ｮ譁・ｭ怜・縺ｫ髢｢縺吶ｋ險倩ｿｰ縺梧立諤晄Φ縺ｮ縺ｾ縺ｾ縺�縺｣縺溘◆繧∽ｿｮ豁｣縺励◆縲・
     - **菫ｮ豁｣**: `plan.md` 荳翫・縲形str`: 蛟溽畑, `String`: 謇�譛峨�阪→縺・≧險倩ｿｰ繧貞炎髯､縺励�∵眠莉墓ｧ倥・縲形str` (邏皮ｲ区ｰｸ邯壼�､) / `ByteBuf` (荳�諢乗園譛峨ヰ繧､繝亥・) / `StringBuilder` (讒狗ｯ臥畑迥ｶ諷・縲阪↓譖ｴ譁ｰ縺励◆縲・
  5. **繧ｳ繝ｳ繝代う繝ｩ繝代せ鬆・ｺ上・荳肴紛蜷・*:
     - **邨先棡**: `memory_safety_migration_plan.md` 縺ｮ繝代せ鬆・ｺ上′證ｫ螳壹・縺ｾ縺ｾ縺�縺｣縺溘◆繧∽ｿｮ豁｣縺励◆縲・
     - **菫ｮ豁｣**: 邨ｱ蜷井ｻ墓ｧ倥↓蜷医ｏ縺帙※ `effect attribution`, `resource_ir_gen`, `region_inference` 繧呈ｭ｣縺励＞鬆・ｺ上〒繝代う繝励Λ繧､繝ｳ・按ｧ10.5.1, ﾂｧ10.7・峨↓邨・∩霎ｼ繧薙□縲・
- [邨占ｫ・縺代▽繧阪ｓ]:
  - 莉雁屓縺ｮ邏皮ｲ区�ｧ繝ｻ謇�譛画ｨｩ縺ｮ諡｡蠑ｵ莉墓ｧ倥・縲〇enn險倅ｺ九〒謗ｲ縺偵ｉ繧後◆縲後・繝ｩ繝・ヨ繝輔か繝ｼ繝�髱樔ｾ晏ｭ倥・謚ｽ雎｡蛹悶�阪�瑚・菴懆ｨ�隱槭・繝ｩ繝・ヨ繝輔か繝ｼ繝�縺ｫ閠舌∴縺・ｋ蝣・欧縺ｪ蝙九す繧ｹ繝・Β縲阪・荳ｭ譬ｸ繧呈・縺吶ｂ縺ｮ縺ｧ縺ゅｊ縲∵署遉ｺ縺輔ｌ縺溽ｧｻ陦瑚ｨ育判縺ｧ谿ｵ髫守噪縺ｫ螳溯｣・ｒ騾ｲ繧√ｋ縺薙→縺ｧ逶ｮ讓咎＃謌舌・蜊∝・縺ｫ蜿ｯ閭ｽ縺ｧ縺ゅｋ縺ｨ蛻､譁ｭ縺励◆縲・

---

# 2026-03-15 菴懈･ｭ繝｡繝｢ (doc: mem1.md 縺ｨ縺ｮ謨ｴ蜷域�ｧ逶｣譟ｻ繝ｻ繧ｮ繝｣繝・・菫ｮ豁｣)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `doc/chat/dump/mem1.md` 縺ｮ蜈ｨ險ｭ險郁ｦ∫ｴ�・医Γ繝｢繝ｪ邂｡逅・�∝梛讀懈渊縲∫ｷ壼ｽ｢蝙九�∵園譛画ｨｩ縲∥lloc/drop 閾ｪ蜍募喧縲√Λ繝ｳ繧ｿ繧､繝�蟾ｮ逡ｰ蜷ｸ蜿趣ｼ峨′ `doc/` 縺ｨ `todo.md` 縺ｫ驕ｩ蛻・↓蜿肴丐縺輔ｌ縺ｦ縺・ｋ縺狗屮譟ｻ縺吶ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `doc/memory_safety_migration_plan.md` ﾂｧ10 縺ｮ compiler 讀懈渊險ｭ險医ｒ謾ｹ蝟・
    - `MoveState` 竊・`VarState` 縺ｫ謾ｹ蜷阪＠縲～BorrowedShared { borrower_count }` 縺ｨ `BorrowedUnique` 迥ｶ諷九ｒ霑ｽ蜉�縲・
    - borrow conflict 險ｺ譁ｭ (5007, 5008) 繧定ｿｽ蜉�縲・
    - Resource IR 蜻ｽ莉､繝ｪ繧ｹ繝・(`move`, `borrow_shared`, `borrow_unique`, `region_new`, `region_alloc`, `region_end`, `drop`, `io_open`, `io_write`, `io_close`) 繧定ｿｽ蜉�縲・
    - ﾂｧ10.8 繝ｩ繝ｳ繧ｿ繧､繝�蟾ｮ逡ｰ縺ｮ蜷ｸ蜿弱そ繧ｯ繧ｷ繝ｧ繝ｳ繧定ｿｽ蜉�・・asm/LLVM 豈碑ｼ・｡ｨ莉倥″・峨�・
  - `doc/purity_ownership_memory_spec.md` ﾂｧ6.4 繧呈峩譁ｰ:
    - `Valid`/`PossiblyMoved` 竊・`Live`/`MaybeMoved`/`Uninitialized` 縺ｫ邨ｱ荳�縲・
    - 蜷・ｨｺ譁ｭ ID (5001, 5005-5008) 繧定ｿｽ險倥�・
  - `todo.md` 鬆・岼 4 繧呈僑蜈・
    - Phase 1 縺ｫ `Effect` 諡｡蠑ｵ縲～ValueCategory` 蛻・｡槫ｭ舌�～VarState` 霑ｽ霍｡縲［emory safety 險ｺ譁ｭ ID 莠育ｴ・ｒ霑ｽ蜉�縲・
    - 螳御ｺ・擅莉ｶ縺ｫ borrow conflict 讀懷・縺ｨ繝ｩ繝ｳ繧ｿ繧､繝�蟾ｮ逡ｰ蛻・屬繧定ｿｽ蜉�縲・
- [邨先棡/縺代▲縺犠:
  - mem1.md 縺ｮ荳ｻ隕∬ｨｭ險郁ｦ∫ｴ�・亥�､縺ｮ3蛻・｡槭�∝・驛ｨEffect縲｛wnership/borrow/linear讀懈渊縲‘scape analysis縲‥rop elaboration縲〉egion inference縲仝asm/LLVM蟾ｮ逡ｰ蜷ｸ蜿弱�∽ｾ晏ｭ伜梛縺ｸ縺ｮ蟆・擂諡｡蠑ｵ・峨・蜈ｨ縺ｦ doc/ 縺ｨ todo.md 縺ｫ蜿肴丐貂医∩縲・

---

# 2026-03-15 菴懈･ｭ繝｡繝｢ (doc: 邏皮ｲ区�ｧ繝ｻ謇�譛画ｨｩ繝ｻ繝｡繝｢繝ｪ邂｡逅・・邨ｱ蜷井ｻ墓ｧ倥ｒ菴懈・)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `doc/chat/dump/mem1.md` 縺ｮ ChatGPT 隴ｰ隲悶ｒ謨ｴ逅・＠縲¨EPLg2 縺ｮ邏皮ｲ区�ｧ繝ｻ謇�譛画ｨｩ繝ｻ邱壼ｽ｢諤ｧ繝ｻ繝｡繝｢繝ｪ邂｡逅・・邨ｱ蜷井ｻ墓ｧ俶嶌繧・`doc/` 縺ｫ菴懈・縺吶ｋ縲・
  - 譌｢蟄倥・髢｢騾｣繝峨く繝･繝｡繝ｳ繝医→縺ｮ荳肴紛蜷医ｒ隗｣豸医☆繧九�・
  - `todo.md` 繧呈眠莉墓ｧ倥↓蜷医ｏ縺帙※譖ｴ譁ｰ縺吶ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `doc/purity_ownership_memory_spec.md` (譁ｰ隕丈ｽ懈・)
    - mem1.md 縺ｮ險ｭ險郁ｭｰ隲悶ｒ謨ｴ逅・＠縺溽ｵｱ蜷井ｻ墓ｧ俶嶌縲・
    - 蛟､縺ｮ 3 蛻・｡・(pure persistent value / unique mutable work state / linear capability)縲・
    - surface effect (`Pure`/`Impure`) 縺ｨ compiler 蜀・Κ蜉ｹ譫・(`InternalAlloc`/`ExternalIO`/`Nondet`/`Unsafe`) 縺ｮ蛻・屬縲・
    - Region Inference + Drop Elaboration 縺ｮ莠梧ｮｵ讒九∴繝｡繝｢繝ｪ邂｡逅・�・
    - `set` 縺ｮ譁ｰ purity 隕丞援 (escape analysis 繝吶・繧ｹ)縲・
    - 譁・ｭ怜・ (`str`/`ByteBuf`/`StringBuilder`)縲´ist (persistent list + builder)縲！O (consume-return handle) 縺ｮ莉墓ｧ倥�・
    - Resource IR 縺ｨ compiler 隗｣譫舌ヱ繧ｹ鬆・・螳夂ｾｩ縲・
    - Wasm/LLVM 縺ｧ謠・∴繧九ｂ縺ｮ (螳牙・諢丞袖隲・ 縺ｨ謠・∴縺ｪ縺・ｂ縺ｮ (迚ｩ逅・Ξ繧､繧｢繧ｦ繝・ 縺ｮ蛹ｺ蛻･縲・
  - `doc/memory_safety_compiler_design.md` (譖ｴ譁ｰ)
    - 邨ｱ蜷井ｻ墓ｧ倥∈縺ｮ蜿ら・繧定ｿｽ蜉�縲・
    - alloc/dealloc 縺ｮ Pure 謇ｱ縺・ｒ `InternalAlloc` 繝吶・繧ｹ縺ｫ螟画峩縲・
    - `MemPtr<T>` 繧・compiler/runtime 蠅・阜縺ｫ蜀埼・鄂ｮ縲・
    - Region Inference 縺ｨ Drop Elaboration 縺ｮ遽�繧定ｿｽ蜉�縲・
  - `doc/move_effect_spec.md` (譖ｴ譁ｰ)
    - 邨ｱ蜷井ｻ墓ｧ倥∈縺ｮ蜿ら・繧定ｿｽ蜉�縲・
    - compiler 蜀・Κ蜉ｹ譫懷・鬘・(`InternalAlloc`/`ExternalIO`/`Nondet`/`Unsafe`) 繧定ｿｽ蜉�縲・
    - `set` 縺ｮ譁ｰ purity 隕丞援繧定ｿｽ蜉�縲・
    - builtins 隕∽ｻｶ繧・`InternalAlloc` 繝吶・繧ｹ縺ｫ螟画峩縲・
    - Resource IR 繝代せ繧定ｿｽ蜉�縲・
  - `doc/stdlib_breaking_reboot.md` (譖ｴ譁ｰ)
    - `MemPtr<T>` / `RegionToken<T>` 縺ｮ菴咲ｽｮ縺･縺代ｒ compiler/runtime 蠅・阜縺ｨ縺励※譏守｢ｺ蛹悶�・
    - 繝｡繝｢繝ｪ閭ｽ蜉・trait 遽�縺ｫ邨ｱ蜷井ｻ墓ｧ倥∈縺ｮ蜿ら・縺ｨ 3 蛻・｡槭・蜑肴署繧定ｿｽ蜉�縲・
  - `doc/stdlib_doc_comment_policy.md` (譖ｴ譁ｰ)
    - `[豕ｨ諢従` 遽�縺ｮ謇�譛画ｨｩ繝ｻ繝｡繝｢繝ｪ髢｢騾｣鬆・岼縺ｫ 3 蛻・｡槭∈縺ｮ蜿ら・繧定ｿｽ蜉�縲・
  - `todo.md` (譖ｴ譁ｰ)
    - 繝｡繝｢繝ｪ螳牙・蝙九Δ繝・Ν縺ｮ繧ｿ繧ｹ繧ｯ繧堤ｵｱ蜷井ｻ墓ｧ倥↓蜷医ｏ縺帙※諡｡蜈・�・
- [plan.md 縺ｨ縺ｮ蟾ｮ逡ｰ/縺輔＞]:
  - plan.md 縺ｯ險�隱槭・蝓ｺ譛ｬ莉墓ｧ・(蜑咲ｽｮ險俶ｳ輔・蠑乗欠蜷代・繧ｪ繝輔し繧､繝峨Ν繝ｼ繝ｫ) 繧定ｨ倩ｿｰ縺励※縺翫ｊ縲√Γ繝｢繝ｪ邂｡逅・・謇�譛画ｨｩ繝ｻ邏皮ｲ区�ｧ縺ｮ隧ｳ邏ｰ險ｭ險医↓縺ｯ險�蜿翫＠縺ｦ縺・↑縺・�・
  - 莉雁屓縺ｮ邨ｱ蜷井ｻ墓ｧ倥・ plan.md 縺ｮ `a->b` (pure) / `a*>b` (impure) 縺ｮ蛹ｺ蛻･繧堤匱螻輔＆縺帙�…ompiler 蜀・Κ縺ｮ蜉ｹ譫懷・鬘槭ｄ謇�譛画ｨｩ隕丞援繧貞・菴灘喧縺励◆繧ゅ・縺ｧ縺ゅｋ縲・

# 2026-03-14 菴懈･ｭ繝｡繝｢ (fix: 繝医ャ繝励Ξ繝吶Ν隕句・縺励Μ繝ｳ繧ｯ縺ｨ繝輔Μ繧ｬ繝・ruby)繝ｻOGP縺ｮ蛻・屬)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - in-page TOC・医・繝ｼ繧ｸ蜀・岼谺｡・峨↓縺翫＞縺ｦ縲√ユ繧ｭ繧ｹ繝医・繝ｬ繝ｼ繝ｳ縺ｧ縺ｯ縺ｪ縺丞・縺ｮHTML繧ｿ繧ｰ・・<ruby>`・峨ｒ邯ｭ謖√＠縺ｦ繝輔Μ繧ｬ繝翫ｒ陦ｨ遉ｺ縺吶ｋ縲・
  - 蜷梧凾縺ｫ縲～<meta property="og:title">` 遲峨・ OGP 繧ｿ繧ｰ縺ｫ縺ｯ繝輔Μ繧ｬ繝翫′蜷ｫ縺ｾ繧後↑縺・ｈ縺・↓縺吶ｋ・・<rt>`隕∫ｴ�縺ｮ繝・く繧ｹ繝医ｒ謚ｽ蜃ｺ縺九ｉ髯､螟悶☆繧具ｼ峨�・
  - 縺輔ｉ縺ｫ縲√・繝ｼ繧ｸ繝医ャ繝励・H1繝ｬ繝吶Ν縺ｮ隕句・縺励ｂTOC縺ｮ蜈磯�ｭ縺ｫ蜷ｫ繧√�√け繝ｪ繝・け譎ゅ↓ URL 繝上ャ繧ｷ繝･繧貞､画峩縺吶ｋ縺薙→縺ｪ縺上・繝ｼ繧ｸ繝医ャ繝励∈繧ｹ繝�繝ｼ繧ｺ繧ｹ繧ｯ繝ｭ繝ｼ繝ｫ・・href="#"` 縺ｮ繧､繝ｳ繧ｿ繝ｼ繧ｻ繝励ヨ・峨＆縺帙ｋ謖吝虚繧貞ｮ溯｣・☆繧九�・
- [螳溯｣・縺倥▲縺昴≧]:
  - `nodesrc/html_gen.js` 縺翫ｈ縺ｳ `nodesrc/cli.js`
    - OGP逕ｨ縺ｫ蛻ｩ逕ｨ縺輔ｌ繧・`inlinesToPlainText` 髢｢謨ｰ縺ｮ蜃ｦ逅・〒縲、ST繝弱・繝育ｨｮ蛻･縺・`ruby` 縺ｮ蝣ｴ蜷医・ `n.ruby` 縺ｧ縺ｯ縺ｪ縺・`n.base` 縺ｮ繝・く繧ｹ繝医□縺代ｒ謚ｽ蜃ｺ縺吶ｋ繧医≧縺ｫ菫ｮ豁｣縲ゅ％繧後↓繧医ｊ縲＾GP縺ｮtitle縺ｪ縺ｩ縺ｫ繝輔Μ繧ｬ繝翫′豺ｷ蜈･縺励↑縺上↑縺｣縺溘�・
  - `nodesrc/inpage_toc_helper.js`
    - `extractInPageToc` 縺ｫ縺ｦ `inlinesToHtml` 繧剃ｽｿ逕ｨ縺励※隕句・縺励・HTML・医ヰ繝・ず繧帝勁縺丞・縺ｮ繝代・繧ｹ邨先棡・峨ｒ謚ｽ蜃ｺ縺励�～ruby` 縺ｪ縺ｩ縺ｮ陦ｨ遉ｺ繧堤ｶｭ謖√＠縺溘∪縺ｾTOC鬆・岼縺ｨ縺吶ｋ繧医≧縺ｫ螟画峩縲・
    - H1 縺ｮ繝ｫ繝ｼ繝郁ｦ句・縺励ｒ縲！D縺ｪ縺暦ｼ医ヨ繝・・縺ｸ縺ｮ繧｢繝ｳ繧ｫ繝ｼ `href="#"`・峨→縺励※TOC繝ｪ繧ｹ繝医・蜈磯�ｭ縺ｫ霑ｽ蜉�縺吶ｋ蜃ｦ逅・ｒ螳溯｣・�・
  - `nodesrc/static/playground_runtime.js`
    - TOC蜀・Μ繝ｳ繧ｯ縺ｮ荳ｭ縺ｧ `href="#"` 繧偵け繝ｪ繝・け縺励◆蝣ｴ蜷医�∵里螳壹・繧｢繧ｯ繧ｷ繝ｧ繝ｳ・医ワ繝・す繝･縺ｮ莉倅ｸ趣ｼ峨ｒ辟｡蜉ｹ蛹悶＠縲～window.scrollTo` 繧堤畑縺・※繝医ャ繝励∈繧ｹ繧ｯ繝ｭ繝ｼ繝ｫ縺吶ｋ謖吝虚繧剃ｻ倅ｸ弱�ゅ∪縺・`history.pushState` 繧堤畑縺・※繝上ャ繧ｷ繝･縺ｮ豸亥悉繧ょ庄閭ｽ縺ｫ縺励◆縲・



- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - tutorial 縺ｨ stdlib 繝峨く繝･繝｡繝ｳ繝医↓縺ｦ縲∬ｦ句・縺励↓蝓ｺ縺･縺上�後・繝ｼ繧ｸ蜀・岼谺｡・・n-page TOC・峨�阪ｒ蜿ｳ蛛ｴ・・C蜷代￠・峨♀繧医・謚倥ｊ縺溘◆縺ｿ繝｡繝九Η繝ｼ・医Δ繝舌う繝ｫ蜷代￠・峨→縺励※霑ｽ蜉�縺励�√ｈ繧翫せ繝�繝ｼ繧ｺ縺ｫ譁・嶌蜀・ｒ遘ｻ蜍輔〒縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
  - struct 繧・fn 縺ｪ縺ｩ縺ｮ繝舌ャ繧ｸ・育ｨｮ鬘橸ｼ画ュ蝣ｱ繧ら岼谺｡蜀・↓陦ｨ遉ｺ縺吶ｋ縺薙→縺ｧ縲∫岼逧・・API縺ｸ縺吶＄繧｢繧ｯ繧ｻ繧ｹ蜿ｯ閭ｽ縺ｫ縺吶ｋ縲・
- [螳溯｣・縺倥▲縺昴≧]:
  - `nodesrc/inpage_toc_helper.js` (譁ｰ隕丈ｽ懈・)
    - AST 縺ｮ Document 繝弱・繝峨ｒ襍ｰ譟ｻ・・extractInPageToc`・峨＠縲～section` 繝弱・繝会ｼ・id`縺ｨ縺ｪ繧虐lug繧・ヰ繝・ず諠・�ｱ繧呈歓蜃ｺ・峨・驟榊・繧堤函謌舌�・
    - `renderInPageTocHtml` 縺ｫ縺ｦ縲・嚴螻､・・epth・峨▼縺代＆繧後◆ HTML・・<ul>` / `<li>`・峨ｒ逕滓・縲・
  - `nodesrc/html_gen_playground.js`
    - HTML繝ｬ繧､繧｢繧ｦ繝茨ｼ・SS繧ｰ繝ｪ繝・ラ・峨↓蜿ｳ繧ｫ繝ｩ繝� `<aside class="doc-inpage-toc">` 縺ｨ繝｢繝舌う繝ｫ逕ｨ縺ｮ `<details class="doc-inpage-toc-mobile">` 繧定ｿｽ蜉�縺励�∫函謌舌＠縺鬱OC HTML繧呈ｳｨ蜈･縲・
  - `nodesrc/static/playground.css`
    - `.doc-layout` 繧・繧ｫ繝ｩ繝�縺九ｉ3繧ｫ繝ｩ繝�・・280px 1fr 240px`・峨∈螟画峩・医ョ繧ｹ繧ｯ繝医ャ繝暦ｼ峨�・
    - 隕∫ｴ�縺ｮ蝗ｺ螳夐・鄂ｮ・・position: sticky`・峨→蜿ｳ蛛ｴ逶ｮ谺｡縺ｮ繧ｹ繧ｿ繧､繝ｪ繝ｳ繧ｰ繧定ｿｽ蜉�縲・
    - 繝｡繝・ぅ繧｢繧ｯ繧ｨ繝ｪ・・max-width: 768px`・峨ｒ逕ｨ縺・※繝｢繝舌う繝ｫ蟷・・蝣ｴ蜷医・蜿ｳ繧ｵ繧､繝峨ヰ繝ｼ繧帝國縺励�∵悽譁・ｸ企Κ縺ｫ `<details>` 縺ｧ螻暮幕縺ｧ縺阪ｋ逶ｮ谺｡繧定｡ｨ遉ｺ縺吶ｋ繧医≧蛻・ｲ仙・逅・ｒ險倩ｿｰ縲・
  - `nodesrc/static/playground_runtime.js`
    - `IntersectionObserver` 繧定ｿｽ蜉�縺励�√Θ繝ｼ繧ｶ繝ｼ縺後せ繧ｯ繝ｭ繝ｼ繝ｫ縺励◆髫帙↓迴ｾ蝨ｨ隕九∴縺ｦ縺・ｋ隕句・縺暦ｼ・section`・峨↓蟇ｾ蠢懊☆繧狗岼谺｡縺ｮ繝ｪ繝ｳ繧ｯ・・.inpage-toc-link`・峨∈ `active` 繧ｯ繝ｩ繧ｹ繧定・蜍穂ｻ倅ｸ趣ｼ・croll Spy讖溯・・峨☆繧倶ｻ慕ｵ・∩繧貞ｰ主・縲・



- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - tutorial 縺ｨ stdlib 縺ｮ playground HTML 縺ｫ縺ｦ縲∝ｷｦ蛛ｴ縺ｮ繧ｵ繧､繝峨ヰ繝ｼ縺ｮ繝ｪ繝ｳ繧ｯ・・able of Contents・峨′螢翫ｌ縺ｦ縺翫ｊ縲√←縺ｮ繝ｪ繝ｳ繧ｯ繧偵け繝ｪ繝・け縺励※繧ら樟蝨ｨ縺ｮ繝壹・繧ｸ縺ｫ驕ｷ遘ｻ縺励※縺励∪縺・撫鬘後ｒ菫ｮ豁｣縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `nodesrc/cli.js` 蜀・・ `genOne` 髢｢謨ｰ縺ｫ縺ｦ `renderHtmlPlayground` 繧貞他縺ｳ蜃ｺ縺咎圀縲ゝOC逕滓・縺ｮ縺溘ａ縺ｮ `tocLinks` 縺ｫ縲∝推繝ｪ繝ｳ繧ｯ縺ｮ逶ｸ蟇ｾ繝代せ繧定ｧ｣豎ｺ縺吶ｋ `makePageTocLinks` 髢｢謨ｰ縺ｮ邨先棡縺ｧ縺ｯ縺ｪ縺上�√ヱ繧ｹ隗｣豎ｺ蜑阪・ `tocEntries` 繧偵◎縺ｮ縺ｾ縺ｾ貂｡縺励※縺・◆縲・
  - 縺昴・縺溘ａ蜷・お繝ｳ繝医Μ縺ｮ `href` 縺梧ｭ｣縺励￥逕滓・縺輔ｌ縺壹�∫樟蝨ｨ縺ｮ繝壹・繧ｸ繧呈欠縺吶Μ繝ｳ繧ｯ・育ｩｺ縺ｮ href 縺ｪ縺ｩ・峨↓縺ｪ縺｣縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nodesrc/cli.js`
    - `genOne` 髢｢謨ｰ蜀・〒 `renderHtmlPlayground` 縺ｫ貂｡縺・`tocLinks` 繧・`makePageTocLinks(outRel, tocEntries)` 縺ｫ螟画峩縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/cli.js` 繧ｳ繝槭Φ繝峨〒 HTML 繧貞・逕滓・縺励�～href` 螻樊�ｧ縺ｫ豁｣縺励￥逶ｸ蟇ｾ繝代せ・井ｾ・ `02_numbers_and_variables.html`・峨′險ｭ螳壹＆繧後※縺・ｋ縺薙→繧堤｢ｺ隱阪�・



- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - playground 縺ｧ逕滓・縺輔ｌ繧・tutorials 繧・stdlib 繝峨く繝･繝｡繝ｳ繝亥・縺ｮ繧ｳ繝ｼ繝峨′螳溯｡後〒縺阪↑縺・ｼ医さ繝ｳ繝代う繝ｫ繧ｨ繝ｩ繝ｼ繝ｻ繧ｯ繝ｩ繝・す繝･・牙撫鬘後ｒ菫ｮ豁｣縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `nodesrc/static/playground_runtime.js` 縺ｫ縺ｦ縲√さ繝ｼ繝峨・繧ｳ繝ｳ繝代う繝ｫ繧貞他縺ｳ蜃ｺ縺咎圀縲∵ｨ呎ｺ悶Λ繧､繝悶Λ繝ｪ・・tdlib・峨ｒ蜷ｫ縺ｾ縺ｪ縺・`compile_source` 繝｡繧ｽ繝・ラ繧堤畑縺・※縺・◆縲・
  - 縺昴・縺溘ａ縲√メ繝･繝ｼ繝医Μ繧｢繝ｫ縺ｪ縺ｩ縺ｮ `#import "std/stdio" as *` 縺ｨ縺・▲縺滓ｨ呎ｺ悶Λ繧､繝悶Λ繝ｪ縺ｸ縺ｮ萓晏ｭ倥′隗｣豎ｺ縺ｧ縺阪★縲∵悴螳夂ｾｩ隴伜挨蟄舌↑縺ｩ縺ｧ繧ｳ繝ｳ繝代う繝ｫ縺悟､ｱ謨励＠縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nodesrc/static/playground_runtime.js`
    - `runBtn.onclick` 蜀・・繧ｳ繝ｳ繝代う繝ｫ蜃ｦ逅・ｒ縲～compile_source` 縺九ｉ `compile_source_with_vfs_and_stdlib` 縺ｫ螟画峩縺励◆縲・
    - 繝舌Φ繝峨Ν縺輔ｌ縺滓ｨ呎ｺ悶Λ繧､繝悶Λ繝ｪ繧・`bindings.get_bundled_stdlib_vfs()` 縺ｫ繧医ｊ蜿門ｾ励＠縲∽ｸ�邱偵↓貂｡縺吶ｈ縺・↓縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - 蜊倅ｽ薙せ繧ｯ繝ｪ繝励ヨ縺ｧ縺ｮ繧ｳ繝ｳ繝代う繝ｫ謖吝虚遒ｺ隱阪↓縺ｦ縲∵ｭ｣蟶ｸ縺ｫ `compile_source_with_vfs_and_stdlib` 縺碁�壹ｊ縲仝ASM繧ｳ繝ｼ繝峨′逕滓・縺輔ｌ繧九％縺ｨ繧堤｢ｺ隱阪�・
  - `node nodesrc/cli.js` 繧ｳ繝槭Φ繝峨ｒ螳溯｡後＠縲√さ繝ｳ繝代う繝ｫ蠕後・繝√Η繝ｼ繝医Μ繧｢繝ｫ繧・ｨ呎ｺ悶Λ繧､繝悶Λ繝ｪ縺ｮ HTML 繧貞・逕滓・縺励◆縲・

# 2026-03-14 菴懈･ｭ繝｡繝｢ (feat: 讀懃ｴ｢讖溯・縺ｮ蠑ｷ蛹・- 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝牙ｯｾ蠢懊・蝙玖｡ｨ遉ｺ繝ｻ繝輔ぅ繝ｫ繧ｿ霑ｽ蜉�)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - 讀懃ｴ｢讖溯・縺ｨ繝峨く繝･繝｡繝ｳ繝郁｡ｨ遉ｺ繧貞ｼｷ蛹悶＠縲√が繝ｼ繝舌・繝ｭ繝ｼ繝峨＆繧後◆蜷悟錐髢｢謨ｰ繧呈ｭ｣遒ｺ縺ｫ蛹ｺ蛻･繝ｻ繝翫ン繧ｲ繝ｼ繝医〒縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
  - 讀懃ｴ｢ UI 縺ｫ繝輔ぅ繝ｫ繧ｿ繧定ｿｽ蜉�縺励�∫岼逧・・隴伜挨蟄舌ｒ邏�譌ｩ縺剰ｦ九▽縺代ｉ繧後ｋ繧医≧縺ｫ縺吶ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nodesrc/parser.js`
    - `parseNeplText` 縺ｫ縺翫＞縺ｦ縲～fn` / `struct` 縺ｪ縺ｩ縺ｮ kind 縺ｨ `<(i32)->i32>` 縺ｪ縺ｩ縺ｮ蝙九す繧ｰ繝阪メ繝｣繧呈歓蜃ｺ縺吶ｋ繧医≧縺ｫ諡｡蠑ｵ縲・
    - 繝ｬ繧ｬ繧ｷ繝ｼ縺ｪ `name: description` 蠖｢蠑上・繝峨く繝･繝｡繝ｳ繝医ｒ譌｢螳壹・隕句・縺励→縺励※謇ｱ縺・ｈ縺・↓菫ｮ豁｣縲・
  - `nodesrc/html_gen.js`
    - `makeSlug` 縺ｫ縺翫＞縺ｦ縲∝梛諠・�ｱ繧貞性繧√◆荳�諢上・繧ｹ繝ｩ繧ｰ繧堤函謌舌☆繧九ｈ縺・↓菫ｮ豁｣縲６RL繧ｨ繝ｳ繧ｳ繝ｼ繝牙ｯｾ遲悶→縺励※遨ｺ逋ｽ繧帝勁蜴ｻ縲・
    - 繧ｻ繧ｯ繧ｷ繝ｧ繝ｳ縺ｮ ID 繧帝嚴螻､讒矩��・井ｾ・ `parent-child--type`・峨〒逕滓・縺吶ｋ繧医≧縺ｫ螟画峩縺励�√ロ繧ｹ繝医＆繧後◆螳夂ｾｩ縺ｸ縺ｮ繝ｪ繝ｳ繧ｯ繧呈ｭ｣遒ｺ蛹悶�・
    - 隕句・縺励↓遞ｮ鬘橸ｼ医ヰ繝・ず・峨→蝙九す繧ｰ繝阪メ繝｣繧定｡ｨ遉ｺ縺吶ｋ繧医≧縺ｫ諡｡蠑ｵ縲・
  - `nodesrc/search.js`
    - `html_gen.js` 縺ｨ蜷梧悄縺励◆髫主ｱ､繧ｹ繝ｩ繧ｰ逕滓・繝ｭ繧ｸ繝・け繧貞ｮ溯｣・�・
    - 讀懃ｴ｢繧ｨ繝ｳ繝医Μ縺ｫ `kind` 縺ｨ `type` 諠・�ｱ繧定ｿｽ蜉�縲・
  - `nodesrc/html_gen_playground.js`
    - 讀懃ｴ｢ UI 縺ｫ `kind` (遞ｮ鬘・ 縺ｨ `path` (繝輔ぃ繧､繝ｫ繝代せ) 縺ｫ繧医ｋ邨槭ｊ霎ｼ縺ｿ繝輔ぅ繝ｫ繧ｿ繧定ｿｽ蜉�縲・
    - 讀懃ｴ｢邨先棡縺ｫ蝙九す繧ｰ繝阪メ繝｣繧定｡ｨ遉ｺ縲・
    - `:target` 繧ｻ繧ｯ繧ｷ繝ｧ繝ｳ縺ｮ蠑ｷ隱ｿ陦ｨ遉ｺ繧ｹ繧ｿ繧､繝ｫ繧呈隼蝟・�・
    - 繝舌ャ繧ｸ繧堤區譁・ｭ励・譫�邱壹・隗剃ｸｸ縺ｮ繝｢繝�繝ｳ縺ｪ繝・じ繧､繝ｳ縺ｫ譖ｴ譁ｰ縲・
  - `nodesrc/cli.js`
    - `rootPrefix` 縺ｮ豺ｱ縺戊ｨ育ｮ励ｒ菫ｮ豁｣縺励�・04 繧ｨ繝ｩ繝ｼ繧定ｧ｣豸医�・
    - 蜷御ｸ�繝輔ぃ繧､繝ｫ蜀・・繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝蛾未謨ｰ縺梧ｭ｣縺励￥繧､繝ｳ繝・ャ繧ｯ繧ｹ縺輔ｌ繧九ｈ縺・ID 逕滓・繧剃ｿｮ豁｣縲・
  - `nodesrc/test_search.js`
    - 遞ｮ鬘樊ュ蝣ｱ縺ｮ謚ｽ蜃ｺ縺ｨ繝輔ぅ繝ｫ繧ｿ繝ｪ繝ｳ繧ｰ縺ｫ髢｢縺吶ｋ繝ｦ繝九ャ繝医ユ繧ｹ繝医ｒ霑ｽ蜉�縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `math.nepl` (譌ｧ蠖｢蠑・ 縺翫ｈ縺ｳ `fenwick.nepl` (繝阪せ繝亥ｽ｢蠑・ 縺ｫ縺翫＞縺ｦ縲∵､懃ｴ｢邨先棡縺九ｉ豁｣縺励￥繧ｸ繝｣繝ｳ繝励＠蠑ｷ隱ｿ陦ｨ遉ｺ縺輔ｌ繧九％縺ｨ繧堤｢ｺ隱阪�・
  - `add` 縺ｪ縺ｩ縺ｮ繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝蛾未謨ｰ縺悟梛縺ｧ蛹ｺ蛻･縺輔ｌ縲√◎繧後◇繧悟�句挨縺ｮ繧｢繝ｳ繧ｫ繝ｼ縺ｸ鬟帙・縺薙→繧堤｢ｺ隱阪�・
  - 逕滓・縺輔ｌ縺・HTML 蜀・・ `id` 縺ｨ讀懃ｴ｢繧､繝ｳ繝・ャ繧ｯ繧ｹ縺ｮ fragment 縺碁嚴螻､讒矩��繧貞性繧√※荳�閾ｴ縺吶ｋ縺薙→繧堤｢ｺ隱阪�・

# 2026-03-14 菴懈･ｭ繝｡繝｢ (feat: stdlib/tutorial HTML 縺ｫ蜈ｨ譁・､懃ｴ｢讖溯・繧定ｿｽ蜉�)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - stdlib 縺ｨ tutorial 縺ｮ HTML 蜷・・繝ｼ繧ｸ縺ｫ縲√せ繧ｳ繝ｼ繝暦ｼ・utorial 蜈ｨ菴・/ stdlib 蜈ｨ菴難ｼ画ｨｪ譁ｭ縺ｮ繝ｪ繧｢繝ｫ繧ｿ繧､繝�蜈ｨ譁・､懃ｴ｢ UI 繧定ｿｽ蜉�縺吶ｋ縲・
  - 讀懃ｴ｢繝ｭ繧ｸ繝・け・・S・峨・繝ｭ繝ｼ繧ｫ繝ｫ繝・せ繝医→ HTML 蝓九ａ霎ｼ縺ｿ縺ｧ蜈ｨ縺丞酔縺倥さ繝ｼ繝峨ｒ菴ｿ縺・�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nodesrc/search.js` 繧呈眠隕丈ｽ懈・縲・
    - `searchIndex(query, index, maxResults)`: AND 讀懃ｴ｢縲√せ繧ｳ繧｢鬆・ｿ泌唆縲・
    - `buildEntriesFromAst(ast, pageUrl, pageTitle)`: AST 縺九ｉ讀懃ｴ｢繧ｨ繝ｳ繝医Μ繧呈ｧ狗ｯ峨�・
    - `inlinesToSearchText(inlines)`: 繝ｫ繝難ｼ域ｼ｢蟄・+ 隱ｭ縺ｿ莉ｮ蜷搾ｼ峨・荳｡譁ｹ繧偵う繝ｳ繝・ャ繧ｯ繧ｹ縺ｫ蜷ｫ繧√ｋ縲・
    - Node.js `module.exports` 縺ｨ 繝悶Λ繧ｦ繧ｶ `NeplSearch` 繧ｰ繝ｭ繝ｼ繝舌Ν縺ｮ荳｡譁ｹ縺ｫ蟇ｾ蠢懊�・
  - `nodesrc/test_search.js` 繧呈眠隕丈ｽ懈・縲・
    - `assert` 繝｢繧ｸ繝･繝ｼ繝ｫ縺ｮ縺ｿ菴ｿ逕ｨ縲∝､夜Κ萓晏ｭ倥ぞ繝ｭ縺ｮ繝ｭ繝ｼ繧ｫ繝ｫ螳檎ｵ舌ユ繧ｹ繝医�・
    - 繝・せ繝・30 莉ｶ・・okenizeQuery / inlinesToSearchText / searchIndex / buildEntriesFromAst / 邨ｱ蜷茨ｼ峨�・
  - `nodesrc/html_gen_playground.js` 繧貞､画峩縲・
    - `SEARCH_JS_SOURCE`: 繝｢繧ｸ繝･繝ｼ繝ｫ隱ｭ縺ｿ霎ｼ縺ｿ譎ゅ↓ `search.js` 繧呈枚蟄怜・縺ｨ縺励※隱ｭ縺ｿ霎ｼ繧�縲・
    - `wrapHtmlPlayground` 縺ｫ `searchIndexJson` 蠑墓焚繧定ｿｽ蜉�縲・
    - `<style>` 縺ｫ讀懃ｴ｢ UI 縺ｮ CSS 繧定ｿｽ蜉�・・.search-wrap` / `.search-input` / `.search-results` 縺ｪ縺ｩ・峨�・
    - `<script>` 蜈磯�ｭ縺ｫ `search.js` 繧・inline 蝓九ａ霎ｼ縺ｿ縺励�～__SEARCH_INDEX__` 螟画焚繧呈ｳｨ蜈･縲・
    - `renderToc` 縺ｫ讀懃ｴ｢繝懊ャ繧ｯ繧ｹ HTML 繧定ｿｽ蜉�・・#doc-search-input` / `#doc-search-results`・峨�・
    - `DOMContentLoaded` 縺ｫ讀懃ｴ｢ UI 繧､繝吶Φ繝医ワ繝ｳ繝峨Λ繧定ｿｽ蜉�・医Μ繧｢繝ｫ繧ｿ繧､繝�繝峨Ο繝・・繝�繧ｦ繝ｳ / 繧ｭ繝ｼ繝懊・繝・竊鯛・Enter/Escape・峨�・
    - `renderHtmlPlayground` 縺ｫ `searchIndex` 繧ｪ繝励す繝ｧ繝ｳ繧定ｿｽ蜉�縲・
  - `nodesrc/cli.js` 繧貞､画峩縲・
    - `buildScopeSearchIndex(inputRoot, files, excludeDirs)` 繧定ｿｽ蜉�縲・
    - 繧ｹ繧ｳ繝ｼ繝暦ｼ亥・蜉帙ョ繧｣繝ｬ繧ｯ繝医Μ = tutorial 蜈ｨ菴・or stdlib 蜈ｨ菴難ｼ峨＃縺ｨ縺ｫ蜈ｨ繝壹・繧ｸ縺ｮ AST 繧剃ｺ句燕隗｣譫舌＠讀懃ｴ｢繧､繝ｳ繝・ャ繧ｯ繧ｹ繧呈ｧ狗ｯ峨☆繧九�・
    - `genOne` 縺ｫ繧､繝ｳ繝・ャ繧ｯ繧ｹ繧呈ｸ｡縺励�∝推繝壹・繧ｸ縺ｮ HTML 縺ｫ蜷御ｸ�繧ｹ繧ｳ繝ｼ繝励・繧､繝ｳ繝・ャ繧ｯ繧ｹ繧貞沂繧∬ｾｼ繧�縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/test_search.js` 竊・`30 謌仙粥, 0 螟ｱ謨輿
  - HTML 逕滓・繝√ぉ繝・け・・__SEARCH_INDEX__` / `NeplSearch` / `search-input` / `search-results` / `searchIndexJson`・俄・ 蜈ｨ pass
  - tutorial 繧ｹ繧ｳ繝ｼ繝励〒 29 繝輔ぃ繧､繝ｫ縺九ｉ 148 繧ｨ繝ｳ繝医Μ繧呈ｧ狗ｯ峨〒縺阪ｋ縺薙→繧堤｢ｺ隱阪�・
- [plan.md 縺ｨ縺ｮ蟾ｮ逡ｰ/diffreference]:
  - plan.md 縺ｯ讀懃ｴ｢讖溯・縺ｫ險�蜿翫＠縺ｦ縺・↑縺・◆繧∝ｷｮ逡ｰ縺ｪ縺励�・
  - 螳溯｣・・縲梧､懃ｴ｢繧ｹ繧ｳ繝ｼ繝励ｒ蜈･蜉帙ョ繧｣繝ｬ繧ｯ繝医Μ蜊倅ｽ搾ｼ・utorial/stdlib・峨〒蛻・￠繧九�崎ｨｭ險医〒縲√Θ繝ｼ繧ｶ繝ｼ隕∽ｻｶ縺ｫ蜷郁・縺励※縺・ｋ縲・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (fix: bare Result map/and_then 縺ｮ callable 隗｣豎ｺ繧剃ｿｮ豁｣)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `core/option` 縺ｨ `core/result` 縺ｫ蜷悟錐縺ｮ `map` / `and_then` 繧定ｿｽ蜉�縺励◆縺・∴縺ｧ縲¨EPLg2 縺ｮ bare 蜷・+ type args 險俶ｳ輔〒繧よｭ｣縺励￥[隗｣豎ｺ/縺九＞縺代▽]縺輔ｌ繧九ｈ縺・↓縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `typecheck` 縺ｮ `Symbol::Ident` 隗｣豎ｺ縺後�‘xplicit type args 繧呈戟縺､ callable 縺ｫ蟇ｾ縺励※繧・`lookup_callable_any(name)` 繧貞・縺ｫ隕九※縺・◆縲・
  - 縺昴・縺溘ａ `map<i32,i32,str>` 縺ｨ `and_then<i32,i32,str>` 縺後�“eneric arity 3 縺ｮ `Result` 迚医〒縺ｯ縺ｪ縺上�“eneric arity 2 縺ｮ `Option` 迚医∈隱､縺｣縺ｦ邨舌・莉倥″縲～expression left extra values on the stack` 縺ｫ蟠ｩ繧後※縺・◆縲・
  - `map_err` 縺ｯ `Result` 蛛ｴ縺ｫ縺励°蟄伜惠縺励↑縺・◆繧・�壹▲縺ｦ縺翫ｊ縲’ailure 縺ｯ bare 蜷悟錐 callable 鄒､縺ｮ驕ｸ蛻･縺ｫ髯舌ｉ繧後※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nepl-core/src/typecheck.rs`
    - explicit type args 莉倥″ `Symbol::Ident` 縺ｧ縺ｯ縲～lookup_all_callables(name)` 縺九ｉ `type_params.len() == type_args.len()` 繧呈ｺ�縺溘☆ callable 縺�縺代ｒ蛟呵｣懊↓谿九＠縲・ 莉ｶ縺ｪ繧峨◎繧後ｒ蜆ｪ蜈医☆繧九ｈ縺・､画峩縲・
    - unresolved callable stack entry 繧剃ｽ懊ｋ邨瑚ｷｯ縺ｧ繧ゅ�‘xplicit type args 縺後≠繧九→縺阪・蜷後§ generic arity filter 繧帝←逕ｨ縺吶ｋ繧医≧螟画峩縲・
    - 隱ｿ譟ｻ逕ｨ縺ｫ蜈･繧後※縺・◆ debug 蜃ｺ蜉帙ｒ蜑企勁縲・
  - `stdlib/core/option.nepl`
    - `unwrap_or` 繧・bare 蜷阪∈謠・∴縲～map` / `and_then` 縺ｨ縺昴・ doctest 繧定ｿｽ蜉�縲・
  - `stdlib/core/result.nepl`
    - `and_then` 縺ｨ doctest 繧定ｿｽ蜉�縲・
  - `stdlib/tests/option.n.md`
  - `stdlib/tests/result.n.md`
  - `tutorials/getting_started/05_option.n.md`
  - `tutorials/getting_started/13_type_driven_error_modeling.n.md`
    - bare `unwrap_or` / `map` / `and_then` 蜑肴署縺ｸ霑ｽ蠕薙�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `RUSTFLAGS='-C link-arg=-fuse-ld=lld' cargo build -p nepl-cli`
  - `env -u RUSTFLAGS cargo build --target wasm32-unknown-unknown --manifest-path nepl-web/Cargo.toml`
  - `NO_COLOR=false trunk build`
  - `node nodesrc/run_doctest.js -i stdlib/core/option.nepl -n 2`
  - `node nodesrc/run_doctest.js -i stdlib/core/result.nepl -n 3`
  - `node nodesrc/run_doctest.js -i stdlib/core/result.nepl -n 4`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/05_option.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/13_type_driven_error_modeling.n.md -n 1`
  - `node nodesrc/tests.js -i stdlib/tests/option.n.md -i stdlib/tests/result.n.md -i stdlib/core/option.nepl -i stdlib/core/result.nepl --no-stdlib --no-tree -o /tmp/tests-option-result-fp.json -j 2`
    - `summary: 10/10 pass`

# 2026-03-12 菴懈･ｭ繝｡繝｢ (editor extensions: Zed syntax highlight 逕ｨ grammar 繧貞ｼｷ蛹・

- 逶ｮ逧・
  - Zed extension 縺ｮ syntax layer 縺・top-level 螳夂ｾｩ蜷阪ｄ directive / import / type annotation 繧呈怙菴朱剞隴伜挨縺ｧ縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - 譌｢蟄倥・ `editors/zed/tree-sitter-neplg2/grammar.js` 縺ｯ縲瑚｡後ｒ荳ｦ縺ｹ繧九□縺代�阪・邁｡譏捺ｧ区枚縺ｧ縲～fn` / `struct` / `enum` / `trait` / `impl` 縺ｮ蜷榊燕縲’ield縲」ariant縲‥irective 蜷阪↑縺ｩ繧貞玄蛻･縺ｧ縺阪※縺・↑縺九▲縺溘�・
  - 縺昴・縺溘ａ highlight query 繧堤ｴｰ縺九￥譖ｸ縺・※繧ゅ�・未謨ｰ蜷阪ｄ蝙句錐繧帝←蛻・↓濶ｲ蛻・￠縺吶ｋ縺溘ａ縺ｮ node 縺悟ｭ伜惠縺励↑縺九▲縺溘�・
- 螟画峩:
  - `editors/zed/tree-sitter-neplg2/grammar.js`
    - top-level 縺ｨ縺励※ `function_definition`, `struct_definition`, `enum_definition`, `trait_definition`, `impl_definition`, `directive`, `expression_statement` 繧貞・髮｢縺励◆縲・
    - `directive_name`, `import_path`, `alias_clause`, `field_definition`, `enum_variant`, `generic_params`, `type_annotation` 縺ｪ縺ｩ縺ｮ node 繧定ｿｽ蜉�縺励◆縲・
  - `editors/zed/languages/neplg2/highlights.scm`
    - function / type / property / constant / parameter / namespace 縺ｮ capture 繧定ｿｽ蜉�縺励◆縲・
  - `editors/zed/languages/neplg2/brackets.scm`
    - `[` `]` 繧・bracket 縺ｨ縺励※謇ｱ縺・ｈ縺・↓縺励◆縲・
  - `editors/zed/languages/neplg2/config.toml`
    - `autoclose_before` 繧定ｿｽ蜉�縺励◆縲・
- 讀懆ｨｼ:
  - `node --check editors/zed/tree-sitter-neplg2/grammar.js`
    - pass
  - `node -e "global.grammar = x => x; const g = require('./editors/zed/tree-sitter-neplg2/grammar.js'); console.log(g.name, Object.keys(g.rules).length)"`
    - 邨先棡: `neplg2 28`
- 蟾ｮ逡ｰ繝｡繝｢:
  - 縺ｾ縺� `tree-sitter generate` / Zed 荳翫〒縺ｮ螳溯ｪｭ縺ｿ霎ｼ縺ｿ讀懆ｨｼ縺ｯ譛ｪ螳溯｡後�ら樟陦檎腸蠅・〒縺ｯ `zed_extension_api` 蛛ｴ縺ｮ toolchain 譚｡莉ｶ縺梧ｮ九▲縺ｦ縺・ｋ縺溘ａ縲〇ed package 蜈ｨ菴薙・ build 讀懆ｨｼ縺ｯ蛻･騾泌ｿ・ｦ√�・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (editor extensions: Zed shell 縺ｮ build 蜑肴署繧呈紛逅・

- 逶ｮ逧・
  - `nepl-lsp` 繧貞ｮ滄圀縺ｫ build/test 縺励�〇ed extension shell 蛛ｴ繧よ､懆ｨｼ蜿ｯ閭ｽ縺ｪ蠖｢縺ｸ蟇・○繧九�・
- 螟画峩:
  - `nepl-lsp/src/main.rs`
    - `analyze_document` 蜀・・ `entry_path` capture 繧剃ｿｮ豁｣縺励�～cargo test -p nepl-lsp` 縺碁�壹ｋ繧医≧縺ｫ縺励◆縲・
    - 譛ｪ菴ｿ逕ｨ import 繧呈紛逅・＠縺溘�・
  - `editors/zed/Cargo.toml`
    - 迢ｬ遶・crate 縺ｨ縺励※ `cargo check --manifest-path editors/zed/Cargo.toml` 繧貞ｮ溯｡後〒縺阪ｋ繧医≧縲∫ｩｺ縺ｮ `[workspace]` 繧定ｿｽ蜉�縺励◆縲・
    - `zed_extension_api` 縺ｮ荳紋ｻ｣繧剃ｸ九￡縺ｦ迴ｾ陦・toolchain 縺ｧ讀懆ｨｼ縺ｧ縺阪ｋ縺句・繧雁・縺代◆縲・
  - `editors/zed/README.md`
  - `doc/editor_extensions.md`
    - `nepl-lsp` 縺ｯ build 貂医∩縺ｧ縺ゅｋ縺薙→縺ｨ縲〇ed 蛛ｴ縺ｯ `edition2024` 隕∵ｱゅ′ blocker 縺ｧ縺ゅｋ縺薙→繧定ｿｽ險倥＠縺溘�・
- 邨先棡:
  - `cargo test -p nepl-lsp` 縺ｯ pass縲・
  - `cargo check --manifest-path editors/zed/Cargo.toml` 縺ｯ `zed_extension_api` 縺ｨ縺昴・萓晏ｭ・(`spdx` 縺ｪ縺ｩ) 縺・`edition2024` 繧定ｦ∵ｱゅ＠縲∫樟陦・Cargo 1.83.0 縺ｧ縺ｯ manifest parse 譎らせ縺ｧ螟ｱ謨励☆繧九％縺ｨ繧堤｢ｺ隱阪＠縺溘�・
  - 縺､縺ｾ繧顔樟蝨ｨ縺ｮ blocker 縺ｯ extension 螳溯｣・〒縺ｪ縺・toolchain / upstream crate 隕∽ｻｶ縺ｧ縺ゅｋ縲・
- 谺｡:
  - Zed shell 繧貞ｮ滄圀縺ｫ build 讀懆ｨｼ縺吶ｋ縺ｫ縺ｯ縲ヽust/Cargo 繧・`edition2024` 蟇ｾ蠢懃沿縺ｸ荳翫￡繧九°縲∽ｺ呈鋤縺ｮ縺ゅｋ `zed_extension_api` 邉ｻ蛻励ｒ迚ｹ螳壹＠縺ｦ蝗ｺ螳壹☆繧句ｿ・ｦ√′縺ゅｋ縲・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (editor extensions: doc comment 繧・compiler/nm 邨檎罰縺ｧ LSP hover 縺ｸ謗･邯・

- 逶ｮ逧・
  - stdlib 縺ｧ譌｢縺ｫ菴ｿ繧上ｌ縺ｦ縺・ｋ `//:` 蠖｢蠑上・ document comment 繧・compiler 縺梧ｭ｣縺励￥隱崎ｭ倥＠縲‘ditor extension / LSP 縺・JavaScript 蛛ｴ縺ｮ蜀榊ｮ溯｣・↓萓晏ｭ倥○縺・Rust 蛛ｴ縺�縺代〒蛻ｩ逕ｨ縺ｧ縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
  - `nodesrc/parser.js` / `nodesrc/html_gen.js` 縺ｫ縺励°辟｡縺九▲縺・`nm` 縺ｮ雋ｬ蜍吶ｒ縲∵僑蠑ｵ讖溯・蜷代￠縺ｮ compiler 螳溯｣・∈謖√■霎ｼ繧�縲・
- 譬ｹ譛ｬ蜴溷屏:
  - Rust compiler 蛛ｴ縺ｮ lexer 縺ｯ `///` 繧・doc comment 縺ｨ縺励※謇ｱ縺｣縺ｦ縺・◆縺後�《tdlib 螳滄°逕ｨ縺ｮ `//:` 繧定ｪ崎ｭ倥＠縺ｦ縺・↑縺九▲縺溘�・
  - parser 縺ｫ縺ｯ item 逶ｴ蜑・doc comment 縺ｮ邏舌▼縺大・逅・′譌｢縺ｫ縺ゅ▲縺溘◆繧√�》oken 蛹悶〒縺阪※縺・↑縺・％縺ｨ縺御ｸｻ蝗�縺�縺｣縺溘�・
  - LSP hover 蛛ｴ繧・raw 譁・ｭ怜・繧偵◎縺ｮ縺ｾ縺ｾ陦ｨ遉ｺ縺励※縺翫ｊ縲～nm` 縺ｨ縺励※讒矩��蛹悶＆繧後◆ document comment 繧貞茜逕ｨ縺励※縺・↑縺九▲縺溘�・
- 螟画峩:
  - `nepl-core/src/lexer.rs`
    - `///` 縺ｫ蜉�縺医※ `//:` 繧・`DocComment` token 縺ｨ縺励※謇ｱ縺・ｈ縺・ｿｮ豁｣縺励◆縲・
  - `nepl-core/src/parser.rs`
    - module 蜈磯�ｭ縺ｮ `//:` 繧・module doc 縺ｨ縺励※蛻・屬蜿門ｾ励☆繧句・逅・ｒ霑ｽ蜉�縺励◆縲・
  - `nepl-core/src/ast.rs`
    - `Module.doc` 繧定ｿｽ蜉�縺励◆縲・
  - `nepl-core/src/nm.rs`
    - editor/LSP 蜷代￠縺ｮ Rust 螳溯｣・`nm` parser 繧定ｿｽ蜉�縺励◆縲・
    - heading / list / code block / gloss / ruby 縺ｪ縺ｩ繧呈ｧ矩��蛹悶＠縲｀arkdown 縺ｸ謌ｻ縺・renderer `render_document_markdown` 繧定ｿｽ蜉�縺励◆縲・
  - `nepl-language/src/lib.rs`
    - 螳夂ｾｩ諠・�ｱ縺ｫ `doc_ast` 繧定ｿｽ蜉�縺励�…ompiler 縺悟叙蠕励＠縺・document comment 繧・`nm` AST 縺ｨ縺励※菫晄戟縺ｧ縺阪ｋ繧医≧縺ｫ縺励◆縲・
  - `nepl-lsp/src/main.rs`
    - hover 縺・raw 譁・ｭ怜・縺ｧ縺ｯ縺ｪ縺・`doc_ast` 繧・Markdown 縺ｸ render 縺励◆邨先棡繧貞━蜈医＠縺ｦ霑斐☆繧医≧縺ｫ縺励◆縲・
  - `nepl-core/tests/doc_comments.rs`
    - `//:` 縺ｮ item 邏舌▼縺代�《tdlib 螳溘ヵ繧｡繧､繝ｫ縺ｮ doc comment縲［odule doc 縺ｨ item doc 縺ｮ蛻・屬繧堤｢ｺ隱阪☆繧・test 繧定ｿｽ蜉�縺励◆縲・
- 螳溯｣・憾豕・
  - compiler 縺ｧ `//:` document comment 繧・token 蛹悶＠縲∝ｮ夂ｾｩ諠・�ｱ縺ｸ邏舌▼縺代ｋ邨瑚ｷｯ縺ｯ霑ｽ蜉�貂医∩縲・
  - `nm` parser / renderer 繧・Rust 蛛ｴ縺ｸ霑ｽ蜉�縺励�´SP hover 縺九ｉ蛻ｩ逕ｨ縺吶ｋ邨瑚ｷｯ繧りｿｽ蜉�貂医∩縲・
  - 縺ｾ縺� Zed/VSCode 蛛ｴ縺ｮ package 螳溯｣・→縲”over 陦ｨ遉ｺ蜀・ｮｹ縺ｮ隧ｳ邏ｰ謨ｴ蠖｢縺ｯ譛ｪ螳御ｺ・�・
- plan.md 縺ｨ縺ｮ蟾ｮ逡ｰ:
  - plan.md 縺ｮ editor extension 蜈ｱ騾壼渕逶､縺ｫ蜷代￠縺ｦ縲´SP hover 逕ｨ縺ｮ doc comment 蜿門ｾ礼ｵ瑚ｷｯ繧貞・陦後〒螳溯｣・＠縺溘�・
  - 縺ｾ縺� WASIp1 server 驟榊ｸ・ｽ｢諷九�〇ed package 縺九ｉ縺ｮ螳溯｡悟ｰ守ｷ壹�〃SCode shell 縺ｯ譛ｪ螳溯｣・�・
- 讀懆ｨｼ:
  - `cargo test -p nepl-language` 縺ｯ譌｢蟄倥・遽・峇縺ｧ pass 貂医∩縲・
  - pull 蠕後・蜀肴､懆ｨｼ縺ｨ縺励※ `cargo test -p nepl-language semantics_analysis_reports_hover_doc_and_type -- --nocapture` 繧貞・螳溯｡御ｸｭ縲・
  - `cargo test -p nepl-core --test doc_comments -- --nocapture` 縺ｯ lock 遶ｶ蜷医ｒ驕ｿ縺代ｋ縺溘ａ蜊倡峡縺ｧ蜀榊ｮ溯｡後☆繧句燕謠舌�・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (editor extensions: `nepl-language` 霑ｽ蜉�)

- 逶ｮ逧・
  - `nepl-web` 縺ｨ縺ｯ蛻･縺ｫ縲‘ditor extension 蜷代￠縺ｮ蜈ｱ騾・Rust lib 繧定ｿｽ蜉�縺吶ｋ縲・
  - Zed / VSCode / 蟆・擂縺ｮ WASIp1 Language Server 縺悟酔縺・compiler 螳溯｣・ｒ蜀榊茜逕ｨ縺ｧ縺阪ｋ蠅・阜繧剃ｽ懊ｋ縲・
  - extension 蛛ｴ縺ｯ阮・￥菫昴■縲∝ｰ・擂逧・↓ Rust 螳溯｣・ｒ NEPLg2 縺ｸ鄂ｮ縺肴鋤縺医ｄ縺吶＞讒区・縺ｫ縺吶ｋ縲・
- 譬ｹ譛ｬ譁ｹ驥・
  - 莉･蜑阪・ `nepl-web` 隗｣譫・API 縺ｯ Web 蜷代￠ wasm-bindgen 蜃ｺ蜉帙↓蟇・▲縺ｦ縺翫ｊ縲‘ditor extension 縺ｮ蜈ｱ騾壼渕逶､縺ｨ縺励※縺ｯ荳埼←蛻・□縺｣縺溘�・
  - 縺昴・縺ｾ縺ｾ extension 縺・`nepl-web` 縺ｸ萓晏ｭ倥☆繧九→縲仝eb 蜷代￠ JS/wasm API 縺ｨ editor 蜷代￠ Rust API 縺悟ｯ・ｵ仙粋縺励�〇ed / VSCode / 蟆・擂縺ｮ selfhost 鄂ｮ謠帙・蠅・阜縺梧尠譏ｧ縺ｫ縺ｪ繧九�・
  - 縺昴・縺溘ａ縲…ompiler 譛ｬ菴・(`nepl-core`) 縺ｮ荳翫↓ editor 蟆ら畑 lib `nepl-language` 繧定ｿｽ蜉�縺励�仝eb 蜷代￠ API 縺ｨ縺ｯ蛻・屬縺励◆縲・
- 螟画峩:
  - `Cargo.toml`
    - workspace member 縺ｫ `nepl-language` 繧定ｿｽ蜉�縺励◆縲・
  - `nepl-language/Cargo.toml`
    - 譁ｰ隕・crate 繧定ｿｽ蜉�縺励◆縲・
  - `nepl-language/src/lib.rs`
    - lexer / diagnostics / name resolution / semantics 繧・Rust struct 縺ｧ霑斐☆ API 繧定ｿｽ蜉�縺励◆縲・
    - `LoadResult` 繧貞女縺大叙繧玖､・焚繝輔ぃ繧､繝ｫ隗｣譫・API 繧定ｿｽ蜉�縺励�”over / 螳夂ｾｩ繧ｸ繝｣繝ｳ繝礼畑縺ｫ path 莉倥″ range 繧定ｿ斐☆繧医≧縺ｫ縺励◆縲・
    - `nepl-web` 縺ｫ髢峨§縺ｦ縺・◆蜷榊燕隗｣豎ｺ trace / semantic token 邨・∩遶九※繧・editor 蜈ｱ騾・lib 縺ｨ縺励※蛻・ｊ蜃ｺ縺励◆縲・
    - cross-file resolution 繧貞性繧� unit test 繧定ｿｽ蜉�縺励◆縲・
  - `doc/editor_extensions.md`
    - `nepl-web` 縺ｨ editor extension 逕ｨ lib 縺ｮ雋ｬ蜍吝・髮｢縲〇ed / VSCode / 蟆・擂縺ｮ LSP 縺ｮ讒区・譁ｹ驥昴ｒ險倩ｿｰ縺励◆縲・
  - `editors/zed/README.md`
    - Zed extension 縺ｮ讒区・譁ｹ驥昴→谺｡谿ｵ髫弱・菴懈･ｭ鬆・岼繧定ｿｽ蜉�縺励◆縲・
- 螳溯｣・憾豕・
  - `nepl-language` 縺ｯ霑ｽ蜉�貂医∩縺ｧ縲》oken / diagnostic / hover / semantic token / definition 逕ｨ繝・・繧ｿ繧定ｿ斐○繧九�・
  - 蜊倅ｸ�繝輔ぃ繧､繝ｫ隗｣譫舌→縲～Loader` 繧剃ｻ九＠縺溯､・焚繝輔ぃ繧､繝ｫ隗｣譫舌・荳｡譁ｹ繧呈桶縺医ｋ縲・
  - 縺ｾ縺� Zed extension package 譛ｬ菴薙�》ree-sitter grammar縲仝ASIp1 Language Server binary 縺ｯ譛ｪ螳溯｣・�・
- plan.md 縺ｨ縺ｮ蟾ｮ逡ｰ:
  - `plan.md` 縺ｮ LSP / Zed / VSCode 譁ｹ驥昴↓蟇ｾ縺励�∽ｻ雁屓縺ｯ editor 蜈ｱ騾夊ｧ｣譫・lib 縺ｮ蝨溷床縺ｾ縺ｧ繧貞・陦悟ｮ溯｣・＠縺溘�・
  - 螳滄圀縺ｮ Zed package 縺ｨ VSCode package 縺ｯ譛ｪ逹�謇九〒縺ゅｊ縲∵ｬ｡谿ｵ髫弱〒 `nepl-language` 縺ｮ荳翫↓ Rust 陬ｽ Language Server 繧定ｿｽ蜉�縺吶ｋ蠢・ｦ√′縺ゅｋ縲・
- 讀懆ｨｼ:
  - `cargo test -p nepl-language`
    - 邨先棡: pass
  - `node nodesrc/run_doctest.js -i tests/compiler/typeannot.n.md -n 2`
    - 邨先棡: pass
  - 蜿り�・
    - `node nodesrc/run_doctest.js -i stdlib/core/result.nepl -n 1`
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/11_testing_workflow.n.md -n 1`
    - 荳願ｨ・2 莉ｶ縺ｯ `return value mismatch` 縺ｨ runtime trap 縺ｧ fail縲ゆｻ雁屓縺ｮ螟画峩蟇ｾ雎｡縺ｯ髮・ｨ医せ繧ｯ繝ｪ繝励ヨ縺ｧ縺ゅｊ縲〉epo_metrics 螟画峩縺ｮ譛臥┌縺ｫ髢｢菫ゅ↑縺乗里蟄倥・ doctest 蛛ｴ蝠城｡後→縺励※谿九▲縺ｦ縺・ｋ縲・
- 蟾ｮ逡ｰ繝｡繝｢:
  - `node nodesrc/tests.js -i tests -i tutorials -i stdlib ...` 縺ｯ髟ｷ譎る俣邯咏ｶ壹＠縺溘◆繧√�∫｢ｺ隱阪・ `run_doctest.js` 縺ｫ繧医ｋ focused 螳溯｡後∈蛻・ｊ譖ｿ縺医◆縲・
- 莉雁屓縺ｮ螟画峩縺ｯ build/test 邉ｻ繝ｭ繧ｸ繝・け縺ｧ縺ｯ縺ｪ縺上�・寔險医せ繧ｯ繝ｪ繝励ヨ蜊倅ｽ薙・謾ｹ蝟・〒縺ゅｋ縲・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (fix: aggregate struct packing 繧剃ｿｮ豁｣縺励※ SparseSet invalid-path 繧貞ｾｩ譌ｧ)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `alloc/collections/sparse_set` 縺ｮ invalid index path 縺・web/native test path 縺ｧ trap 縺励※縺・◆譬ｹ蝗�繧堤音螳壹＠縲《tdlib 蛛ｴ縺ｮ蝗樣∩縺ｧ縺ｯ縺ｪ縺・compiler 蛛ｴ縺九ｉ菫ｮ豁｣縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - [蠖灘・/縺ｨ縺・＠繧Ⅹ縺ｯ `SparseSet` 閾ｪ菴薙・ owner 陦ｨ迴ｾ繧・`alloc/string` 縺ｮ concat 繧堤桝縺｣縺ｦ蛻・ｊ蛻・￠縺溘′縲∵怙邨ら噪縺ｫ縺ｯ `U128DivRem` 縺ｮ繧医≧縺ｪ aggregate 蛟､繧・`StructConstruct` / `TupleConstruct` 縺ｧ邨・∩遶九※繧・codegen 縺後�’ield 縺斐→縺ｮ real storage size 縺ｧ縺ｯ縺ｪ縺・wasm/llvm 縺ｮ scalar `ValType` / `LlTy` 繧ｵ繧､繧ｺ縺ｧ pack 縺励※縺・◆縺薙→縺悟次蝗�縺�縺｣縺溘�・
  - 縺昴・邨先棡縲∥ggregate field 繧端蜷ｫ/縺ｵ縺従繧� struct/tuple 縺・inline byte copy 縺ｧ縺ｯ縺ｪ縺・pointer 逶ｸ蠖薙〒[隧ｰ/縺､]繧√ｉ繧後�～field::get` 縺ｨ蠕檎ｶ壹・ integer-to-string / diag message 逕滓・縺ｧ[螢・縺薙ｏ]繧後◆蛟､繧定ｪｭ縺ｿ縲～SparseSet` invalid index path 縺ｮ message build 縺・`memory access out of bounds` 縺ｫ蟠ｩ繧後※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nepl-core/src/codegen_wasm.rs`
    - `StructConstruct` / `TupleConstruct` 縺ｮ total size 繧・`type_storage_size_bytes` 蝓ｺ貅悶∈菫ｮ豁｣縲・
    - aggregate field/item 縺ｯ source pointer 縺九ｉ destination 縺ｸ byte copy 縺吶ｋ lowering 縺ｫ螟画峩縲・
  - `nepl-core/src/codegen_llvm.rs`
    - wasm 蛛ｴ縺ｨ蜷後§縺・aggregate field/item 繧・real storage size 縺ｶ繧・byte copy 縺吶ｋ繧医≧菫ｮ豁｣縲・
  - `stdlib/alloc/string.nepl`
    - `string_finish_base` 繧定ｿｽ蜉�縺励�〉egion/token 繧剃ｺ碁㍾縺ｫ隱ｭ縺ｿ逶ｴ縺輔★ base pointer 繧・1 蝗槭□縺醍｢ｺ螳壹＠縺ｦ finish 縺吶ｋ蠖｢縺ｸ謨ｴ逅・�・
    - `concat`, `sb_build`, `str_slice`, `from_u128_radix`, `from_f64` 縺ｮ finish 邨瑚ｷｯ繧貞酔 helper 縺ｫ謠・∴縺溘�・
  - `alloc/collections/sparse_set`
    - header owner 縺ｯ `MemPtr<u8>` field 縺ｧ縺ｯ縺ｪ縺・raw `i32` header pointer 繧・public struct 縺ｫ菫晄戟縺励�∝・驛ｨ helper 縺ｧ縺�縺・`MemPtr` 縺ｫ蛹・∩逶ｴ縺吝ｽ｢縺ｸ謨ｴ逅・＠縺溘�・
- [邨先棡/縺代▲縺犠:
  - `stdlib/alloc/string.nepl::doctest#4` 縺・pass 縺ｫ謌ｻ縺｣縺溘�・
  - `stdlib/tests/sparse_set.n.md::doctest#2` 縺ｨ `tests/stdlib/sparse_set_collections.n.md::doctest#1` 縺・web path 縺ｧ繧・pass 縺ｫ謌ｻ縺｣縺溘�・
  - `node nodesrc/tests.js -i stdlib/tests/sparse_set.n.md -i tests/stdlib/sparse_set_collections.n.md -i stdlib/alloc/collections/sparse_set.nepl --no-stdlib --no-tree -o /tmp/tests-sparse-set.json -j 2` 縺ｯ `10/10 pass` 繧堤｢ｺ隱阪＠縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build`
  - `node nodesrc/run_doctest.js -i stdlib/alloc/string.nepl -n 4`
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/sparse_set.nepl -n 1`
  - `node nodesrc/run_doctest.js -i stdlib/tests/sparse_set.n.md -n 2`
  - `node nodesrc/run_doctest.js -i tests/stdlib/sparse_set_collections.n.md -n 1`
  - `node nodesrc/tests.js -i stdlib/tests/sparse_set.n.md -i tests/stdlib/sparse_set_collections.n.md -i stdlib/alloc/collections/sparse_set.nepl --no-stdlib --no-tree -o /tmp/tests-sparse-set.json -j 2`

# 2026-03-06 菴懈･ｭ繝｡繝｢ (feat: examples/bf.nepl 縺ｫ Brainfuck Runner 繧貞ｮ溯｣・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (alloc/collections/sparse_set 隱ｿ譟ｻ邯咏ｶ壹・譛ｪ commit)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `alloc/collections` 縺ｫ `SparseSet` 繧端霑ｽ蜉�/縺､縺・°]縺励�～[0, n)` [遽・峇/縺ｯ繧薙＞]縺ｮ integer set 繧・O(1) membership / insert / remove 縺ｧ[謇ｱ/縺ゅ▽縺犠縺医ｋ繧医≧縺ｫ縺吶ｋ縲・
- [騾ｲ謐・縺励ｓ縺｡繧・￥]:
  - `SparseSet` 縺ｮ public API (`new` / `len` / `universe_len` / `contains` / `insert` / `remove` / `clear` / `free`) 縺ｨ public doctest / fixture 縺ｯ[荳�騾・縺ｲ縺ｨ縺ｨ縺馨繧骸菴懈・/縺輔￥縺帙＞]貂医∩縲・
  - normal path 縺ｯ focused 螳溯｡後〒[騾夐℃/縺､縺・°]縺励※縺・ｋ縲・
    - `stdlib/alloc/collections/sparse_set.nepl::doctest#1/#2`
    - `stdlib/tests/sparse_set.n.md::doctest#1`
    - `tests/stdlib/sparse_set_collections.n.md::doctest#1`
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転縺ｮ[蛻・縺江繧骸蛻・繧従縺・
  - [蠖灘・/縺ｨ縺・＠繧Ⅹ縺ｯ `SparseSet` owner [蜀・Κ/縺ｪ縺・・]縺ｮ field [隱ｭ/繧・縺ｿ[蜃ｺ/縺�]縺励′[螢・縺薙ｏ]繧後※縺・ｋ繧医≧縺ｫ[隕・縺ｿ]縺医◆縺後�”eader 繧・`MemPtr<u8>` field 縺ｧ[謖・繧・縺､險ｭ險医°繧・raw `i32` pointer [菫晄戟/縺ｻ縺肋縺ｸ[關ｽ/縺馨縺ｨ縺吶％縺ｨ縺ｧ normal path 縺ｯ[螳牙ｮ・縺ゅｓ縺ｦ縺Ь縺励◆縲・
  - 縺昴・[蠕・縺ゅ→]縺ｫ[谿・縺ｮ縺転縺｣縺・failure 縺ｯ invalid index path 縺�縺代〒縲～contains s 8` 縺ｮ[譛�蟆丈ｾ・縺輔＞縺励ｇ縺・ｌ縺Ь縺ｾ縺ｧ[邵ｮ蟆・縺励ｅ縺上＠繧・≧]縺ｧ縺阪◆縲・
  - 縺輔ｉ縺ｫ[霑ｽ霍｡/縺､縺・○縺江縺吶ｋ縺ｨ縲～SparseSet` [蝗ｺ譛・縺薙ｆ縺・縺ｧ縺ｯ縺ｪ縺・`sparse_set_diag_index` 縺ｮ[荳ｭ/縺ｪ縺犠縺ｧ[菴・縺､縺従繧・message string 縺・web compile path 縺ｧ `RuntimeError: memory access out of bounds` 繧端襍ｷ/縺馨縺薙＠縺ｦ縺・ｋ縺薙→縺啓蛻・繧従縺九▲縺溘�・
  - `diag_error StdErrorKind::IndexOutOfBounds "abc"` 縺ｯ pass 縺吶ｋ荳�譁ｹ縲～concat "sparse_set_contains" ": index out of bounds "` 繧端蜷ｫ/縺ｵ縺従繧� chain 縺�縺代′ trap 縺吶ｋ縲・
  - `stdlib/alloc/string.nepl::doctest#4` 繧・蜷檎ｳｻ邨ｱ/縺ｩ縺・￠縺・→縺・縺ｮ web path OOB 繧端謖・繧・縺｣縺ｦ縺翫ｊ縲～SparseSet` invalid path failure 縺ｯ[譌｢蟄・縺阪◇繧転縺ｮ `alloc/string` regression 縺ｫ[荵・縺ｮ]縺｣縺ｦ縺・ｋ縺ｨ[蛻､譁ｭ/縺ｯ繧薙□繧転縺励◆縲・
  - native compiler 縺ｧ縺ｯ `SparseSet invalid index` 縺ｮ[譛�蟆丈ｾ・縺輔＞縺励ｇ縺・ｌ縺Ь縺ｯ pass 縺励�『eb compile path 縺�縺代′ trap 縺吶ｋ縺ｮ縺ｧ縲ー逶ｴ謗･/縺｡繧・￥縺帙▽]縺ｮ blocker 縺ｯ stdlib API 險ｭ險医〒縺ｪ縺・web compiler/runtime path [蛛ｴ/縺後ｏ]縺ｫ縺ゅｋ縲・
- [蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `SparseSet` normal path 縺ｮ library 螳溯｣・・[謌千ｫ・縺帙＞繧翫▽]縺励※縺・ｋ縺後�（nvalid index 縺ｮ `Result::Err` path 繧端蜷ｫ/縺ｵ縺従繧� focused suite 縺・web compile path 縺ｧ[譛ｪ蜿取據/縺ｿ縺励ｅ縺・◎縺従縺ｮ縺溘ａ縲∫樟譎らせ縺ｧ縺ｯ commit 縺励↑縺・�・
  - [谺｡/縺､縺讃縺ｯ `alloc/string` 縺ｮ concat / integer-to-string [邨瑚ｷｯ/縺代＞繧江繧・root cause 繝吶・繧ｹ縺ｧ[逶ｴ/縺ｪ縺馨縺励�√◎縺ｮ[蠕・縺ゅ→]縺ｫ `SparseSet` batch 繧端蜀埼幕/縺輔＞縺九＞]縺吶ｋ縲・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (ci: rust install -> cargo build -> trunk build 繧貞・騾・action 蛹・

- 逶ｮ逧・
  - GitHub Actions 縺ｫ謨｣縺｣縺ｦ縺・◆ `Node setup` / `Rust toolchain` / `wasm32 target` / `wasm-bindgen-cli` / `cargo build` / `trunk build` 縺ｮ驥崎､・ｒ 1 邂・園縺ｸ髮・ｴ・☆繧九�・
  - 蜷・workflow 縺ｯ縲悟・騾・build artifact 繧剃ｽ懊ｋ job縲阪→縲後◎縺ｮ artifact 繧貞女縺代※ test / deploy 繧定｡後≧ job縲阪↓蛻・￠縲｜uild 貂医∩謌先棡迚ｩ繧貞・蛻ｩ逕ｨ縺吶ｋ蠖｢縺ｸ蟇・○繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - `compile-test.yml` / `nepl-test-wasi.yml` / `nepl-test-llvm.yml` / `nmd-doctest.yml` / `nm-compile.yml` / `rust-test..yml` / `gh-pages.yml` 縺後�√◎繧後◇繧悟挨縺ｫ toolchain install 縺ｨ `trunk build` 繧呈戟縺｣縺ｦ縺・◆縲・
  - 縺昴・縺溘ａ謇矩�・・譖ｴ譁ｰ貍上ｌ縺瑚ｵｷ縺阪ｄ縺吶￥縲～trunk` 繧・`wasm-bindgen-cli` 縺ｮ譖ｴ譁ｰ縲～Trunk.toml` Linux 陬懈ｭ｣縲‘xamples 驟咲ｽｮ縺ｪ縺ｩ繧呈ｯ主屓螟夐㍾邂｡逅・☆繧区ｧ矩��縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
- 螟画峩:
  - `.github/actions/bootstrap-build/action.yml`
    - CI 蜈ｱ騾壹・ local composite action 繧定ｿｽ蜉�縲・
    - `actions/setup-node`縲～npm install`縲～actions-rs/toolchain`縲～rustup target add wasm32-unknown-unknown`縲～jetli/trunk-action`縲～wasm-bindgen-cli` install縲～Swatinem/rust-cache`縲～cargo build --locked`縲～trunk build --release` 繧帝寔邏・�・
  - `.github/workflows/compile-test.yml`
  - `.github/workflows/rust-test..yml`
  - `.github/workflows/nm-compile.yml`
  - `.github/workflows/nmd-doctest.yml`
  - `.github/workflows/nepl-test-wasi.yml`
  - `.github/workflows/nepl-test-llvm.yml`
    - 縺昴ｌ縺槭ｌ `build` job 縺ｧ蜈ｱ騾・action 繧剃ｽｿ縺｣縺ｦ `dist` / `target/debug` / `target/wasm32-unknown-unknown` 繧・artifact 蛹悶�・
    - test job 蛛ｴ縺ｯ `actions/download-artifact` 縺ｧ蜿門ｾ励＠縺ｦ縺九ｉ縲∝推 workflow 蝗ｺ譛峨・ `cargo test` / `nodesrc/tests.js` / `cargo run -p nepl-cli` / LLVM runner 繧貞ｮ溯｡後☆繧句ｽ｢縺ｸ螟画峩縲・
  - `.github/workflows/gh-pages.yml`
    - pages 蝗ｺ譛峨・ deploy/doctest/doc build 縺ｯ谿九＠縺､縺､縲》oolchain install 縺ｨ build 譛ｬ菴薙・蜈ｱ騾・action 縺ｸ遘ｻ蜍輔�・
- 讀懆ｨｼ:
  - 荳�譎・directory `/tmp/gha-yaml-check` 繧剃ｽ懊▲縺ｦ `npm install yaml` 繧定｡後＞縲∝・ workflow 縺ｨ composite action 繧・`yaml` parser 縺ｧ讒区枚遒ｺ隱阪�・
    - 蟇ｾ雎｡:
      - `.github/workflows/*.yml`
      - `.github/actions/bootstrap-build/action.yml`
    - 邨先棡: 蜈ｨ莉ｶ `OK`
- 蟾ｮ逡ｰ繝｡繝｢:
  - workflow 螳溯｡後◎縺ｮ繧ゅ・縺ｯ GitHub Actions 荳翫〒縺ｮ螳溯｡後′蠢・ｦ√↑縺ｮ縺ｧ縲√Ο繝ｼ繧ｫ繝ｫ縺ｧ縺ｯ YAML 讒区枚縺ｨ萓晏ｭ倬未菫ゅ・謨ｴ蜷医∪縺ｧ繧堤｢ｺ隱阪＠縺溘�・
  - 迴ｾ譎らせ縺ｧ縺ｯ artifact 縺ｮ邊貞ｺｦ繧・`dist` / `target/debug` / `target/wasm32-unknown-unknown` 縺ｫ縺励※縺・ｋ縲ゅ＆繧峨↓邨槭ｋ菴吝慍縺ｯ縺ゅｋ縺後�√∪縺壹・蜈ｱ騾壼喧縺ｨ蜀榊茜逕ｨ縺ｮ謌千ｫ九ｒ蜆ｪ蜈医＠縺溘�・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (ci: build 1 蝗・+ pages/test 邨ｱ蜷・+ per-case timeout)

- 逶ｮ逧・
  - workflow 縺斐→縺ｫ `bootstrap-build` 繧堤ｹｰ繧願ｿ斐＠縺ｦ縺・◆讒区・繧偵ｄ繧√�～trunk build` 繧貞性繧� build 繧・1 workflow 蜀・〒 1 蝗槭□縺大ｮ溯｡後＠縲√◎縺ｮ謌先棡迚ｩ繧貞・ test job 縺ｨ Pages deploy 縺ｫ蜀榊茜逕ｨ縺吶ｋ縲・
  - `gh-pages.yml` 縺悟挨 workflow 縺ｧ test 繧貞・螳溯｡後＠縺ｦ縺・◆讒矩��繧定ｧ｣豸医＠縲《ite 縺ｸ縺ｮ publish 繧・test workflow 縺ｮ荳�驛ｨ縺ｸ邨ｱ蜷医☆繧九�・
  - 辟｡髯舌Ν繝ｼ繝礼ｳｻ縺ｮ hang 縺ｧ CI 蜈ｨ菴薙′豁｢縺ｾ繧峨↑縺・ｈ縺・�・ 繧ｱ繝ｼ繧ｹ 20 遘偵�》est job 蜈ｨ菴・10 蛻・・荳企剞繧貞・繧後ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - 蜑肴ｮｵ縺ｮ蜈ｱ騾・action 蛹悶□縺代〒縺ｯ縲『orkflow 縺悟・縺九ｌ縺ｦ縺・ｋ髯舌ｊ `cargo build` / `trunk build` / `npm install` / `cargo install wasm-bindgen-cli` 縺・workflow 謨ｰ縺�縺醍ｹｰ繧願ｿ斐＆繧後ｋ縲・
  - `gh-pages.yml` 縺ｯ site 逕滓・縺ｮ縺溘ａ縺ｫ tests 繧貞・蠎ｦ蝗槭＠縺ｦ縺翫ｊ縲∝酔縺・commit 縺ｫ蟇ｾ縺励※ test 縺・2 驥榊ｮ溯｡後＆繧後※縺・◆縲・
  - `nodesrc/tests.js` 縺ｯ suite 蜈ｨ菴薙・螳溯｡後・縺ｧ縺阪※繧ゅ�仝ASM worker / LLVM child process 縺ｫ per-case timeout 縺檎┌縺上�・ 繧ｱ繝ｼ繧ｹ縺ｮ hang 縺・suite 蜈ｨ菴薙ｒ蠑輔″縺壹ｋ菴吝慍縺後≠縺｣縺溘�・
- 螟画峩:
  - `.github/actions/bootstrap-build/action.yml`
    - `actions/setup-node` 縺ｫ npm cache 繧定ｿｽ蜉�縲・
    - `web/package-lock.json` 繝吶・繧ｹ縺ｧ `npm ci` 繧剃ｽｿ縺・ｽ｢縺ｫ螟画峩縲・
    - `wasm-bindgen-cli` 繧・`actions/cache` 縺ｧ蜀榊茜逕ｨ縺吶ｋ繧医≧螟画峩縲・
    - `wasm-bindgen` 縺ｮ verify step 繧定ｿｽ蜉�縲・
  - `.github/workflows/ci.yml`
    - 譌ｧ test workflow 鄒､縺ｨ Pages deploy 繧・1 workflow 縺ｫ邨ｱ蜷医�・
    - `build` job 縺ｧ `bootstrap-build` 繧・1 蝗槭□縺大ｮ溯｡後＠縲√＆繧峨↓ tutorial / stdlib HTML 繧・`dist` 驟堺ｸ九∈逕滓・縺励※ artifact 蛹悶�・
    - `compile-test` / `rust-test` / `nm-compile` / `wasi-test` / `nmd-doctest` / `llvm-test` 縺ｯ縺吶∋縺ｦ `needs: build` 縺ｧ artifact 繧貞・蛻ｩ逕ｨ縲・
    - `pages-fast-*` 縺ｨ `pages-final-*` 縺ｮ 2 谿ｵ deploy 繧定ｿｽ蜉�縺励�～trunk build` 蠕後・ pending site 繧貞・縺ｫ publish 縺励�》est 螳御ｺ・ｾ後↓ test JSON / summary 繧定ｼ峨○縺・final site 縺ｧ荳頑嶌縺阪☆繧句ｽ｢縺ｫ縺励◆縲・
    - `gh-pages.yml` 縺ｯ蜑企勁縲・
    - test job 縺ｫ縺ｯ `timeout-minutes: 10` 繧定ｿｽ蜉�縺励�～node nodesrc/tests.js` / `cargo test` / `cargo run` 縺ｯ `timeout --signal=KILL 10m ...` 縺ｧ蛹・ｓ縺�縲・
    - test 螳溯｡檎腸蠅・↓ `NEPL_TEST_CASE_TIMEOUT_MS=20000` / `NEPL_WASIX_TIMEOUT_MS=20000` 繧貞・騾壽欠螳壹�・
  - `nodesrc/tests.js`
    - WASM thread pool worker 縺ｫ per-case timer 繧定ｿｽ蜉�縺励�・0 遘偵〒蠢懃ｭ斐＠縺ｪ縺・case 縺ｯ worker 繧・terminate 縺励※ error 縺ｨ縺励※蝗槫庶縺吶ｋ蠖｢縺ｸ螟画峩縲・
    - LLVM / native 螳溯｡後↓菴ｿ縺・`runCommand` 縺ｫ child process timeout 繧定ｿｽ蜉�縺励�∝酔縺倥￥ 20 遘偵〒 kill 縺吶ｋ繧医≧螟画峩縲・
- 讀懆ｨｼ:
  - `node --check nodesrc/tests.js`
  - 荳�譎・directory `/tmp/gha-yaml-check` 繧剃ｽ懊▲縺ｦ `npm install yaml` 繧定｡後＞縲・
    - `.github/workflows/*.yml`
    - `.github/actions/bootstrap-build/action.yml`
    繧・parser 縺ｧ讀懆ｨｼ縲・
- 蟾ｮ逡ｰ繝｡繝｢:
  - Pages final deploy 縺ｯ `build` artifact 縺ｮ `dist` 繧貞・蛻ｩ逕ｨ縺励�《ite 繧剃ｽ懊ｋ縺溘ａ縺ｫ `trunk build` 繧貞・螳溯｡後＠縺ｪ縺・�・
  - pending/final 縺ｮ 2 蝗・deploy 縺ｯ Pages 縺ｸ縺ｮ publish 繧呈掠繧√ｋ縺溘ａ縺ｮ繧ゅ・縺ｧ縲》ests 閾ｪ菴薙・ 1 蝗槭＠縺句ｮ溯｡後＠縺ｪ縺・�・
  - 蛻晉沿縺ｧ縺ｯ `site-fast` / `site-final` 繧帝�壼ｸｸ縺ｮ `upload-artifact` 縺ｧ荳ｭ邯吶＠縺ｦ縺九ｉ `upload-pages-artifact` 縺ｫ貂｡縺励※縺・◆縺後�‥ownload 譎ゅ↓ `dist` directory 縺ｮ髫主ｱ､蜑肴署縺悟ｴｩ繧後※ `tar: dist: Cannot open` 縺ｫ縺ｪ縺｣縺溘�・
  - 縺昴・縺溘ａ Pages 逕ｨ bundle job 縺ｯ逶ｴ謗･ `upload-pages-artifact` 繧定｡後＞縲‥eploy job 縺ｯ `deploy-pages` 縺�縺代ｒ陦後≧讒矩��縺ｸ菫ｮ豁｣縺励◆縲・

- 逶ｮ逧・
  - `rpn.nepl` 繧貞盾閠・↓縺励※ `examples/bf.nepl` 縺ｫ Brainfuck 縺ｮ螳溯｡後ヤ繝ｼ繝ｫ繧貞ｮ溯｣・☆繧九�・
  - 豈手｡悟・蜉帙ｒ蜿励￠莉倥￠縲∝・蜉帙＃縺ｨ縺ｫ繝｡繝｢繝ｪ繧偵Μ繧ｻ繝・ヨ縺励※迢ｬ遶句ｮ溯｡後☆繧九�・
- 螟画峩:
  - `examples/bf.nepl`
    - `alloc/collections/stack` 繧剃ｽｿ縺｣縺ｦ `[` 縺ｨ `]` 縺ｮ繧ｸ繝｣繝ｳ繝怜・繧剃ｺ句燕險育ｮ励☆繧・`compile_jumps` 繧貞ｮ溯｣・�・
    - `eval_line` 縺ｧ 30,000 繝舌う繝医・繝｡繝｢繝ｪ荳翫〒 BF 蜻ｽ莉､・・+` `-` `>` `<` `.` `,` `[` `]`・峨ｒ螳溯｡後�・
    - `,` 縺ｯ迴ｾ迥ｶ 0 繧呈嶌縺崎ｾｼ繧�邁｡逡･螳溯｣・�・
    - 繝｡繧､繝ｳ繝ｫ繝ｼ繝励・蜈･蜉帙＃縺ｨ縺ｫ繝｡繝｢繝ｪ繝舌ャ繝輔ぃ繧堤｢ｺ菫昴・隗｣謾ｾ縺励�∫憾諷九ｒ蠑輔″邯吶′縺ｪ縺・�・
    - 陦ｨ遉ｺ蜷阪・ "Brainfuck REPL" 縺九ｉ "Brainfuck Runner" 縺ｫ螟画峩・域ｯ手｡後Μ繧ｻ繝・ヨ縺ｮ縺溘ａ・峨�・
    - `neplg2:test[bf_hello_world]` doctest 繧定ｿｽ蜉�・・ello World 繝励Ο繧ｰ繝ｩ繝�縺ｮ螳溯｡鯉ｼ峨�・
- 讀懆ｨｼ:
  - `target/debug/nepl-cli -i examples/bf.nepl -o tmp/wasm.wasm && wasmer run tmp/wasm.wasm`
    - `+++++++++[>++++++++>+++++++++++>+++>+<<<<-]>.>++.+++++++..+++.>+++++.<<+++++++++++++++.>.+++.------.--------.>+.>+.` 繧貞・蜉帙＠縺ｦ `Hello World!` 縺ｮ蜃ｺ蜉帙ｒ遒ｺ隱阪�・

# 2026-03-06 菴懈･ｭ繝｡繝｢ (TUI謾ｹ蝟・ rpn縺ｮ騾比ｸｭ險育ｮ怜庄隕門喧縺ｨstdio縺ｮ雋�謨ｰ蜃ｺ蜉帑ｿｮ豁｣)

- 逶ｮ逧・
  - `examples/rpn.nepl` 縺ｫ縺翫＞縺ｦ縲～>` 繝励Ο繝ｳ繝励ヨ縺ｮ蜍穂ｽ懊ｒ繝ｬ繧ｬ繧ｷ繝ｼ迚医↓蜷医ｏ縺帙�∬ｨ育ｮ鈴℃遞九ｒ縲瑚ｨ育ｮ怜燕縲阪�瑚ｨ育ｮ怜ｾ後�阪→縺励※ANSI繧ｫ繝ｩ繝ｼ縺ｧ蜿ｯ隕門喧縺吶ｋ縲・
  - 騾比ｸｭ險育ｮ励ｄ蜃ｺ蜉帙〒雋�謨ｰ繧貞性繧�蠑上′豁｣縺励￥陦ｨ遉ｺ縺輔ｌ繧九ｈ縺・�～stdlib/std/stdio.nepl` 縺ｮ `print_i32` 縺ｫ蟄伜惠縺吶ｋ雋�謨ｰ蜃ｺ蜉帙ヰ繧ｰ繧剃ｿｮ豁｣縺吶ｋ縲・
- 螟画峩:
  - `examples/rpn.nepl`
    - REPL繝励Ο繝ｳ繝励ヨ蜃ｺ蜉帛燕縺ｫ繝医・繧ｯ繝ｳ陦後ｒ莠碁㍾縺ｫ蜃ｺ蜉帙＠縺ｪ縺・ｈ縺・・髟ｷ縺ｪ繝ｫ繝ｼ繝励ｒ蜑企勁縲・
    - `print_step_before` 繧定ｿｽ蜉�縺励�∬ｨ育ｮ怜燕縺ｮ迥ｶ諷九ｒ繧ｷ繧｢繝ｳ (`ansi_cyan`) 縺ｧ蠑ｷ隱ｿ陦ｨ遉ｺ縲・
    - `print_step_after` 繧定ｿｽ蜉�縺励�∬ｨ育ｮ礼ｵ先棡繧堤ｷ題牡 (`ansi_green`) 縺ｧ蠑ｷ隱ｿ陦ｨ遉ｺ縲・
  - `stdlib/std/stdio.nepl`
    - `print_i32` 髢｢謨ｰ縺ｧ雋�縺ｮ謨ｰ縺ｸ縺ｮ險育ｮ励′荳崎ｶｳ縺励※ `0` 縺ｨ縺ｪ繧九ヰ繧ｰ繧剃ｿｮ豁｣縲らｵｶ蟇ｾ蛟､縺ｮ蜷・｡√ｒ騾・�・ｱ暮幕縺励◆縺ｮ縺｡縲∬ｲ�謨ｰ縺ｧ縺ゅｌ縺ｰ `-` 隨ｦ蜿ｷ繧剃ｻ倅ｸ弱☆繧九ｈ縺・隼菫ｮ縲・
    - 繧ｳ繝ｳ繝代う繝ｫ繧ｨ繝ｩ繝ｼ繧貞｡槭＄縺溘ａ `mod_u` 繧・`rem_u` 縺ｫ菫ｮ豁｣縲・
- 邨先棡:
  - `1 2 + 3 + 4 5 + 6 +` 縺ｪ縺ｩ縺ｮ騾｣邯壼・蜉帙↓蟇ｾ縺励※縲∝・逅・＃縺ｨ縺ｮ險育ｮ礼ｮ・園 (`[1 2 +]` 縺ｪ縺ｩ) 縺ｨ邨先棡縺瑚牡莉倥″縺ｧ蛻・°繧翫ｄ縺吶￥陦ｨ遉ｺ縺輔ｌ繧九ｈ縺・↓縺ｪ縺｣縺溘�・
  - `-5` 縺ｪ縺ｩ縺ｮ雋�縺ｮ謨ｰ繧貞・蜉帙＠縺滄圀縺ｫ豁｣蟶ｸ縺ｫ陦ｨ遉ｺ縺輔ｌ繧九ｈ縺・↓縺ｪ縺｣縺溘�・
- 讀懆ｨｼ:
  - `target/debug/nepl-cli -i examples/rpn.nepl -o tmp/wasm.wasm && wasmer run tmp/wasm.wasm`
    - 騾比ｸｭ險育ｮ励・繝医Ξ繝ｼ繧ｹ縺翫ｈ縺ｳ雋�謨ｰ (`1 2 3 4 + - 5 +` -> `-5`) 縺ｮ豁｣縺励＞繝輔か繝ｼ繝槭ャ繝医→蜃ｺ蜉帙ｒ逶ｴ謗･遒ｺ隱阪�・

# 2026-03-06 菴懈･ｭ繝｡繝｢ (蝙句ｮ牙・蛹・ `alloc/string` 縺ｮ荳ｻ隕・raw 遒ｺ菫昴ｒ `RegionToken<u8>` 蛹・

- 逶ｮ逧・
  - `alloc/string` 縺ｮ荳ｻ隕∫函謌千ｵ瑚ｷｯ縺九ｉ `alloc_raw` 繧貞叙繧企勁縺阪�～core/mem` 縺ｮ蝙倶ｻ倥″鬆伜沺 API 縺ｫ蟇・○繧九�・
  - 譁・ｭ怜・逕滓・蜃ｦ逅・〒髟ｷ縺輔・繝・ム縺ｨ譛ｬ譁・・繧､繝ｳ繧ｿ繧・`MemPtr<T>` / `RegionToken<T>` 縺ｧ謇ｱ縺・�∝・驛ｨ縺ｮ逕溘・繧､繝ｳ繧ｿ髴ｲ蜃ｺ繧呈ｸ帙ｉ縺吶�・
- 螟画峩:
  - `stdlib/alloc/string.nepl`
    - `string_alloc_region`
    - `string_region_len_ptr`
    - `string_region_data_ptr`
    - `string_data_ptr`
    - `string_finish`
    繧定ｿｽ蜉�縺励�∵枚蟄怜・繝ｬ繧､繧｢繧ｦ繝亥ｰら畑縺ｮ蜀・Κ繝倥Ν繝代→縺励※謨ｴ逅・�・
  - `concat`
    - 蜃ｺ蜉帶枚蟄怜・縺ｮ遒ｺ菫昴ｒ `string_alloc_region` 縺ｫ螟画峩縲・
    - 蜃ｺ蜉帛・繧ｳ繝斐・繧・`MemPtr<u8>` 繝吶・繧ｹ縺ｸ螟画峩縲・
  - `sb_build`
    - 騾｣邨仙・繝舌ャ繝輔ぃ縺ｮ遒ｺ菫昴ｒ `RegionToken<u8>` 蛹悶�・
    - 蜷・part 縺ｮ隱ｭ縺ｿ蜃ｺ縺励→蜃ｺ蜉帛・譖ｸ縺崎ｾｼ縺ｿ繧貞梛莉倥″繝昴う繝ｳ繧ｿ縺ｸ螟画峩縲・
  - `str_slice`
    - 蛻・ｊ蜃ｺ縺怜・縺ｮ遒ｺ菫昴ｒ `RegionToken<u8>` 蛹悶�・
  - `from_u128_radix`
    - 騾・�・｡∫ｩ阪∩縺ｮ scratch 繧・`RegionToken<u8>` 蛹悶�・
    - 荳�譎・scratch 縺ｯ `dealloc_region` 縺ｧ隗｣謾ｾ縲・
  - `from_f64`
    - 蟆乗焚驛ｨ scratch 繧・`RegionToken<u8>` 蛹悶�・
    - scratch 隗｣謾ｾ繧定ｿｽ蜉�縲・
- 邨先棡:
  - `stdlib/alloc/string.nepl` 縺九ｉ `alloc_raw/realloc_raw/dealloc_raw` 縺ｮ逶ｴ謗･蜻ｼ縺ｳ蜃ｺ縺励・豸医∴縺溘�・
  - `str` 縺ｮ蜀・Κ陦ｨ迴ｾ閾ｪ菴薙・縺ｾ縺� raw address 縺�縺後�∽ｸｻ隕√↑逕滓・邨瑚ｷｯ縺ｧ縺ｯ `RegionToken<u8>` 縺九ｉ `string_finish` 縺ｧ遒ｺ螳壹☆繧区ｵ√ｌ縺ｫ謨ｴ逅・〒縺阪◆縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/stdlib.n.md -i tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md --no-stdlib --no-tree -o /tmp/tests-string-type-safety-v1.json -j 15`
    - 邨先棡: `26/26 pass`
  - `rg -n "alloc_raw|realloc_raw|dealloc_raw" stdlib/alloc/string.nepl`
    - 邨先棡: 隧ｲ蠖薙↑縺・

# 2026-03-06 菴懈･ｭ繝｡繝｢ (alloc/string: i128/u128 縺ｨ蝓ｺ謨ｰ莉倥″譁・ｭ怜・螟画鋤縺ｮ謨ｴ蛯・

- 逶ｮ逧・
  - `alloc/string` 縺ｫ謨ｴ謨ｰ縺ｮ譁・ｭ怜・陦ｨ迴ｾ螟画鋤繧帝寔邏・＠縲～core/cast` 縺ｨ縺ｮ雋ｬ蜍吶ｒ蛻・屬縺吶ｋ縲・
  - `i128` / `u128` 繧貞性繧� 2/8/10/16 騾ｲ縺ｮ螟画鋤繧呈署萓帙☆繧九�・
  - tutorial 縺ｫ縲∵焚蛟､ cast 縺ｨ譁・ｭ怜・螟画鋤縺ｮ驕輔＞繧呈・遉ｺ縺励◆蟆守ｷ壹ｒ霑ｽ蜉�縺吶ｋ縲・
- 螟画峩:
  - `stdlib/alloc/string.nepl`
    - `from_bool`
    - `to_bool`
    - `from_u128` / `from_u128_radix`
    - `to_u128` / `to_u128_radix`
    - `from_i128` / `from_i128_radix`
    - `to_i128` / `to_i128_radix`
    - `u128_divrem_small` 縺ｪ縺ｩ 128-bit 謨ｴ謨ｰ縺ｮ陬懷勧髢｢謨ｰ鄒､
    - `to_i32` 縺ｮ隱ｬ譏弱ｒ迴ｾ螳溯｣・↓蜷医ｏ縺帙※譖ｴ譁ｰ
  - `tests/stdlib.n.md`
    - `i128/u128` 縺ｨ雋�謨ｰ16騾ｲ縺ｮ focused case 繧定ｿｽ蜉�
  - `tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md`
    - `core/cast` 縺ｨ `alloc/string` 縺ｮ菴ｿ縺・・縺・
    - `Result` 繧定ｿ斐☆隗｣譫宣未謨ｰ
    - 2/8/10/16 騾ｲ螟画鋤
    - `i128/u128` 縺ｮ螟ｧ縺阪＞蛟､縺ｮ萓・
  - `tutorials/getting_started/00_index.n.md`
    - 譁ｰ隕・tutorial 縺ｸ縺ｮ蟆守ｷ壹ｒ霑ｽ蜉�
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/stdlib.n.md --no-stdlib --no-tree -o /tmp/tests-stdlib-conversions-i128-v3.json -j 15`
    - 邨先棡: `19/19 pass`

# 2026-03-06 菴懈･ｭ繝｡繝｢ (蝙句ｮ牙・蛹・ `ptr_cast` 蜈ｬ髢句ｻ・ｭ｢)

- 逶ｮ逧・
  - 繝昴う繝ｳ繧ｿ蜀崎ｧ｣驥医・繧医≧縺ｪ unsafe 縺ｪ蜈ｬ髢・API 繧呈ｸ帙ｉ縺励�～MemPtr<T>` / `RegionToken<T>` 繝｢繝・Ν縺ｸ蟇・○繧九�・
- 螟画峩:
  - `stdlib/core/cast.nepl`
    - 譛ｪ菴ｿ逕ｨ縺�縺｣縺・`ptr_cast` 繧貞炎髯､縲・
    - 繝｢繧ｸ繝･繝ｼ繝ｫ蜈磯�ｭ繧ｳ繝｡繝ｳ繝医ｒ縲∵焚蛟､ cast 縺ｨ bitcast 縺ｮ縺ｿ縺ｫ雋ｬ蜍吶ｒ髯仙ｮ壹☆繧玖ｪｬ譏弱∈譖ｴ譁ｰ縲・
- 蛻､譁ｭ:
  - `ptr_cast` 縺ｯ蝙九□縺代ｒ莉倥￠譖ｿ縺医ｋ謫堺ｽ懊〒縲～MemPtr<T>` 縺ｫ繧医ｋ蝙句ｮ牙・蛹匁婿驥昴→遏帷崟縺吶ｋ縲・
  - repo 蜀・盾辣ｧ縺ｯ辟｡縺上�∫樟譎らせ縺ｧ蜈ｬ髢矩擇縺ｫ谿九☆蜷育炊諤ｧ縺ｯ辟｡縺九▲縺溘�・
  - `MemPtr<T>` 縺ｯ縲悟梛莉倥″繧｢繝峨Ξ繧ｹ縲阪�～RegionToken<T>` 縺ｯ縲後◎縺ｮ鬆伜沺縺ｮ繧ｵ繧､繧ｺ縺ｨ謇�譛画ｨｩ縲阪ｒ莨ｴ縺・ｷ壼ｽ｢繝医・繧ｯ繝ｳ縺ｨ縺励※菴ｿ縺・・縺代ｋ縲・

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺF: tutorials Part6 諡｡蜈・+ library-first 蛹・

- 逶ｮ逧・
  - `tutorials/getting_started` Part6・・2縲・7・峨・隱ｬ譏手ｪ､繧翫・荳崎ｶｳ繧堤屮譟ｻ縺励�∫洒縺冗ｰ｡貎斐〒螳牙・縺ｪ譖ｸ縺肴婿縺ｸ譖ｴ譁ｰ縺吶ｋ縲・
  - 逕溘・繧､繝ｳ繧ｿ髴ｲ蜃ｺ繧呈ｸ帙ｉ縺吶◆繧√�～kp` 蛛ｴ縺ｫ `Vec<i32>` 逶ｴ蜿励￠陬懷勧繧定ｿｽ蜉�縺吶ｋ縲・
- 螟画峩:
  - `tutorials/getting_started/22_competitive_io_and_arith.n.md`
    - `Scanner/Writer` 縺ｮ蝓ｺ譛ｬ繝代ち繝ｼ繝ｳ繧・pipe 荳ｭ蠢・↓邁｡貎泌喧縲・
    - i32/i64/遨ｺ逋ｽ蛹ｺ蛻・ｊ蜃ｺ蜉帙・ 3 繧ｱ繝ｼ繧ｹ繧貞ｮ牙・ API 蜑肴署縺ｧ謨ｴ逅・�・
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
    - `Vec + sort + lower/upper_bound` 繧・library-first 縺ｧ蜀肴ｧ区・縲・
  - `tutorials/getting_started/24_competitive_dp_basics.n.md`
    - DP 譛ｬ菴薙ｒ邯ｭ謖√＠縺､縺､ I/O 繧堤ｰ｡貎泌喧縲・
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
    - prefix 繧・`kp/kpprefix` 繝上Φ繝峨Ν API 蜑肴署縺ｸ譖ｴ譁ｰ縲・
    - two pointers 縺ｮ譚｡莉ｶ隧穂ｾ｡繧堤洒邨｡隧穂ｾ｡縺ｫ萓晏ｭ倥＠縺ｪ縺・ｮ牙・縺ｪ蠖｢縺ｸ菫ｮ豁｣縲・
  - `tutorials/getting_started/26_competitive_graph_bfs.n.md`
    - 謇区嶌縺・BFS 縺九ｉ `kp/kpgraph` 蛻ｩ逕ｨ縺ｸ遘ｻ陦後�・
  - `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
    - 譛ｪ螳梧・陦ｨ險倥ｒ蟒・ｭ｢縺励�￣art6 邱上∪縺ｨ繧√→縺励※繝・Φ繝励Ξ繝ｼ繝医・蟇ｾ蠢懆｡ｨ繝ｻ螳滓姶繝輔Ο繝ｼ繧定ｿｽ蜉�縲・
  - `tutorials/getting_started/00_index.n.md`
    - 隱､蟄励ｒ菫ｮ豁｣・磯未謨ｰ縺ｮ縺ｵ繧翫′縺ｪ・峨�・
  - `stdlib/kp/kpprefix.nepl`
    - `PrefixI32` 繝上Φ繝峨Ν縺ｨ `prefix_build_vec_i32` / `prefix_sum_i32` / `prefix_free_i32` 繧定ｿｽ蜉�縲・
  - `stdlib/kp/kpsearch.nepl`
    - `lower_bound_vec_i32` / `upper_bound_vec_i32` / `contains_vec_i32` / `count_equal_range_vec_i32` 繧定ｿｽ蜉�縲・
  - `todo.md`
    - 繝輔ぉ繝ｼ繧ｺF縺ｮ螳御ｺ・ｸ医∩ Part6 蟆ら畑繧ｿ繧ｹ繧ｯ繧貞炎髯､・域悴螳御ｺ・・縺ｿ邯ｭ謖・ｼ峨�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/26_competitive_graph_bfs.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -i stdlib/kp/kpprefix.nepl -i stdlib/kp/kpsearch.nepl --no-tree -o /tmp/tests-part6-kp-refresh-v7.json -j 15`
    - 邨先棡: `219/219 pass`
  - 陬懷勧遒ｺ隱・
    - `node nodesrc/tests.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md --no-tree -o /tmp/tests-part6-25-v6.json -j 15`
    - 邨先棡: `207/207 pass`

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm `add/sub` 蜀榊ｮ夂ｾｩ繝ｪ繝ｳ繧ｯ螟ｱ謨励・譬ｹ譛ｬ菫ｮ豁｣)

- 逶ｮ逧・
  - `--runner all --llvm-all` 螳溯｡梧凾縺ｫ `tests/llvm_target.n.md::doctest#4/#5` 縺・`invalid redefinition of function 'add'/'sub'` 縺ｧ螟ｱ謨励☆繧句撫鬘後ｒ縲∝ｾ御ｻ倥￠蝗樣∩縺ｧ縺ｯ縺ｪ縺冗函謌蝕R讒矩��縺九ｉ隗｣豸医☆繧九�・
- 蜴溷屏:
  - `stdlib/core/math.nepl` 縺ｮ overload 鄒､・・add/sub` 縺ｪ縺ｩ・峨′ `#llvmir` 蜀・〒蜷御ｸ�繧ｷ繝ｳ繝懊Ν蜷搾ｼ・@add`, `@sub`・峨ｒ菴ｿ縺｣縺ｦ縺・◆縲・
  - LLVM 縺ｯ繧ｷ繝ｳ繝懊Ν蜷阪〒 overloading 縺ｧ縺阪↑縺・◆繧√�∝酔荳�繝｢繧ｸ繝･繝ｼ繝ｫ縺ｸ隍・焚蝙狗沿繧貞酔蜷榊ｮ夂ｾｩ縺吶ｋ縺ｨ繝ｪ繝ｳ繧ｯ譎ゅ↓陦晉ｪ√☆繧九�・
  - 縺輔ｉ縺ｫ `u8` 縺ｨ `i32` 縺ｯ LLVM ABI 縺ｧ蜷後§ `i32` 縺ｫ關ｽ縺｡繧九◆繧√�∝梛蛻･ overload 繧偵◎縺ｮ縺ｾ縺ｾ繧ｷ繝ｳ繝懊Ν蜷阪〒蜈ｱ蟄倥＆縺帙ｋ險ｭ險医′謌千ｫ九＠縺ｪ縺・�・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - 逕滓・螳御ｺ・峩蜑阪↓ `deduplicate_overloaded_llvm_symbols` 繧定ｿｽ蜉�縺励�∝酔蜷・`define` 繧偵す繧ｰ繝阪メ繝｣蜊倅ｽ阪〒荳�諢丞喧縲・
    - `define` 蛛ｴ縺ｮ驥崎､・ｒ `name__ovN_<sig>` 縺ｸ豁｣隕丞喧縺励�∝ｯｾ蠢懊☆繧・`call` 蜿ら・繧ょ酔荳�繧ｷ繧ｰ繝阪メ繝｣縺ｧ蠑ｵ繧頑崛縺医ｋ縲・
    - 蜑肴ｮｵ縺ｨ縺励※ `#llvmir` 蜻ｼ縺ｳ蜃ｺ縺苓ｦ∽ｻｶ謚ｽ蜃ｺ縺ｨ AST raw-body 驕ｸ蛻･陬懷勧繧定ｿｽ蜉�縺励�∽ｸ崎ｦ√↑ overload 蜃ｺ蜉帙ｒ謚大宛縲・
- 讀懆ｨｼ:
- `NO_COLOR=false trunk build` -> success
- `cargo build -p nepl-cli` -> success
- `node nodesrc/tests.js -i tests/llvm_target.n.md --no-stdlib --no-tree --runner all --llvm-all -o /tmp/tests-llvm-target-after-dedup-pass.json -j 15` -> `6/6 pass`
- `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-dedup.json -j 15` -> `791/791 pass`

# 2026-03-12 菴懈･ｭ繝｡繝｢ (refactor(vec): Result 蛹悶＠縺・Vec API 繧堤峩謗･萓晏ｭ伜・縺ｸ莨晄眺)

- 逶ｮ逧・
  - `alloc/collections/vec` 縺ｮ `new / with_capacity / push` 繧・`Result<..., StdErrorKind>` 蛹悶＠縺溷､画峩繧偵�∫峩謗･萓晏ｭ倥☆繧・stdlib / tests / tutorials 縺ｸ謨ｴ蜷育噪縺ｫ蜿肴丐縺吶ｋ縲・
  - `Vec` 蜀咲｢ｺ菫昴ｒ莨ｴ縺・API 繧・`stack` 邉ｻ縺ｨ蜷後§螟ｱ謨励Δ繝・Ν縺ｸ蟇・○縺､縺､縲∵里蟄倥・鬮俶ｰｴ貅・helper 縺ｧ縺ｯ `unwrap_ok` 蜷ｸ蜿弱〒蛻ｩ逕ｨ閠・・險倩ｿｰ繧帝℃蜑ｰ縺ｫ蟠ｩ縺輔↑縺・�・
- 譬ｹ譛ｬ蜴溷屏:
  - `Vec` 譛ｬ菴薙□縺代ｒ `Result` 蛹悶☆繧九→縲～std/test` / `alloc/string` / `nm/parser` / `kpgraph` / `wasix/tui` 縺ｪ縺ｩ縺梧立 pure API 繧貞燕謠舌↓縺励※螢翫ｌ繧九�・
  - 縺輔ｉ縺ｫ `StdErrorKind` 縺御ｸ贋ｽ阪・ `alloc/diag/error` 縺ｫ縺ゅｋ縺ｨ縲～vec -> diag/error -> vec` 縺ｮ蠕ｪ迺ｰ萓晏ｭ倥′逕溘§繧九�・
- 螟画峩:
  - `stdlib/alloc/collections/vec.nepl`
    - `new / with_capacity / push` 繧・`Result<..., StdErrorKind>` 蛹悶�・
    - `with_capacity 0` 縺ｯ遒ｺ菫昴ｒ陦後ｏ縺夂ｩｺ `MemPtr` 繧貞桁繧�蠖｢縺ｫ縺励※ `OutOfMemory` 繧剃ｸ崎ｦ∝喧縲・
  - `stdlib/std/test.nepl`
    - `checks_new` / `checks_push` 縺ｧ `Vec<Result<(),str>>` 縺ｮ `Result` 繧貞・驛ｨ蜷ｸ蜿弱�・
  - `stdlib/alloc/string.nepl`
    - `StringBuilder` 縺ｨ `str_split` 縺ｮ蜀・Κ `Vec` 讒狗ｯ峨・霑ｽ蜉�繧・`unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `stdlib/alloc/diag/error.nepl`
    - `Diag` / `Diags` 蜀・Κ縺ｮ `Vec` 讒狗ｯ峨・霑ｽ蜉�繧・`unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `stdlib/alloc/hash/sha256.nepl`
    - scaffold 螳溯｣・・ buffer 讒狗ｯ峨・譖ｴ譁ｰ繧・`unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `stdlib/kp/kpgraph.nepl`
    - BFS 邨先棡繝吶け繧ｿ讒狗ｯ峨ｒ `unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `stdlib/platforms/wasix/tui.nepl`
    - `text_wrap_lines` 縺ｮ陦碁・蛻玲ｧ狗ｯ峨ｒ `unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `stdlib/nm/parser.nepl`
    - inline/block parser 蜀・Κ縺ｮ `Vec` 讒狗ｯ峨・霑ｽ蜉�繧・`unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `stdlib/tests/vec.n.md`
    - current `Vec Result` API 縺ｫ蜷梧悄縲・
  - `tests/stdlib/traits_order.n.md`
    - sort regression 縺ｮ `Vec` 讒狗ｯ峨ｒ `unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `tests/stdlib/selfhost_req.n.md`
    - `Vec<u8>` buffer 讒狗ｯ峨ｒ `unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `tests/stdlib/sort.n.md`
    - sort fixture 縺ｮ `Vec` 讒狗ｯ峨ｒ `unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
  - `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
    - `Vec` pipe 騾｣骼悶ｒ `unwrap_ok new` 縺ｨ `|> push ... |> uwok` 縺ｮ current 譖ｸ蠑上∈譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/tests/vec.n.md -i tests/stdlib/traits_order.n.md -i tests/stdlib/selfhost_req.n.md --no-stdlib --no-tree -o /tmp/tests-vec-result-a.json -j 4`
    - 邨先棡: `10/10 pass`
  - `node nodesrc/tests.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/26_competitive_graph_bfs.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md --no-stdlib --no-tree -o /tmp/tests-vec-result-b2.json -j 4`
    - 邨先棡: `4/4 pass`
  - 陬懷勧遒ｺ隱・
    - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 1` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/23_competitive_sort_and_search.n.md -n 1` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/23_competitive_sort_and_search.n.md -n 2` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -n 1` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -n 2` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/26_competitive_graph_bfs.n.md -n 1` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -n 1` -> pass
- 蟾ｮ逡ｰ繝｡繝｢:
  - `Vec` 縺ｮ public API Result 蛹悶・騾ｲ繧薙□縺後�～vec.nepl` 譛ｬ菴薙・ doc comment / doctest 縺ｫ縺ｯ譌ｧ譖ｸ蠑上・譌ｧ pure 蜑肴署縺ｮ隱ｬ譏弱′縺ｾ縺�谿九ｋ縲・
  - `replace` 繧・`set` 縺ｸ謾ｹ蜷阪☆繧区｡医・ parser / keyword 蛻ｶ邏・・蛻・ｊ蛻・￠蠕後↓蜀肴､懆ｨ弱☆繧九�・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (docs(vec): doc comment 縺ｨ doctest 繧・current Result API 縺ｸ蜷梧悄)

- 逶ｮ逧・
  - `Vec` 譛ｬ菴薙ｒ `Result` 蛹悶＠縺溷ｾ後ｂ縲ーstdlib/alloc/collections/vec.nepl](/mnt/d/project/NEPLg2/stdlib/alloc/collections/vec.nepl) 縺ｮ隱ｬ譏弱→蝓九ａ霎ｼ縺ｿ doctest 縺梧立 pure API 蜑肴署縺ｮ縺ｾ縺ｾ谿九▲縺ｦ縺・◆蟾ｮ蛻・ｒ隗｣豸医☆繧九�・
  - 縺ゅｏ縺帙※縲∵立遽�隕句・縺怜ｽ｢蠑上ｒ貂帙ｉ縺励�∵眠縺励＞ doc comment policy 縺ｫ蟇・○繧九�・
- 螟画峩:
  - `vec.nepl`
    - file header 縺ｮ doctest 繧・`unwrap_ok new` 縺ｨ `|> push ... |> uwok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
    - `new` / `with_capacity` / `len` / `cap` / `data_len` / `is_empty` / `push` / `get` / `replace` / `pop` / `clear` / `free` 縺ｮ comment 萓九ｒ current API 縺ｫ蜷梧悄縲・
    - `is_empty` / `push` / `get` / `replace` / `pop` / `clear` / `free` 縺ｮ遽�隕句・縺励ｒ `### [逶ｮ逧・繧ゅ￥縺ｦ縺江` 蠖｢蠑上∈譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 2` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 3` -> pass

# 2026-03-12 菴懈･ｭ繝｡繝｢ (feat(collections): add bitset)

- 逶ｮ逧・
  - `alloc/collections` 縺ｫ fixed-length 縺ｪ bit 髮・粋繧定ｿｽ蜉�縺励�～BloomFilter` 縺ｨ驕輔▲縺ｦ false positive 縺ｮ縺ｪ縺・membership structure 繧呈ｨ呎ｺ悶〒謇ｱ縺医ｋ繧医≧縺ｫ縺吶ｋ縲・
  - `reboot` 譁ｹ驥昴↓蜷医ｏ縺帙※ bare API 縺ｨ public doctest 繧呈紛縺医�｝ipe 菴ｵ逕ｨ縺ｮ菴ｿ縺・婿縺ｯ `tests/stdlib` 蛛ｴ縺ｧ菫晁ｨｼ縺吶ｋ縲・
- 螟画峩:
  - `stdlib/alloc/collections/bitset.nepl`
    - `BitSet` 繧定ｿｽ蜉�縲・
    - `new` / `len` / `contains` / `insert` / `remove` / `clear` / `fill` / `free` 繧・bare API 縺ｧ螳溯｣・�・
    - 蜀・Κ縺ｯ `nbits` / `nbytes` / `MemPtr<u8>` 繧呈戟縺､ owner struct 縺ｨ縺励�（ndex 縺九ｉ byte offset 縺ｨ bit mask 繧定ｨ育ｮ励＠縺ｦ譖ｴ譁ｰ縺吶ｋ縲・
    - doc comment 縺ｯ譁ｰ policy / format 縺ｸ蜷医ｏ縺帙※縲「sage doctest 繧貞推 public 髢｢謨ｰ縺ｸ霑ｽ蜉�縲・
  - `stdlib/tests/bitset.n.md`
    - insert/remove/len 縺ｨ clear/fill 縺ｮ focused fixture 繧定ｿｽ蜉�縲・
  - `tests/stdlib/bitset_collections.n.md`
    - pipe 險俶ｳ輔〒縺ｮ `insert` / `remove` / `contains` / `fill` 蛻ｩ逕ｨ繧貞屓蟶ｰ縺ｨ縺励※霑ｽ蜉�縲・
- 讀懆ｨｼ:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 2` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 3` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 4` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 5` -> pass

# 2026-03-12 菴懈･ｭ繝｡繝｢ (feat(collections): add adjacency matrix)

- 逶ｮ逧・
  - `alloc/collections` 縺ｫ graph representation 縺ｮ譛�蟆丞ｮ溯｣・→縺励※ `AdjacencyMatrix` 繧定ｿｽ蜉�縺励�∝崋螳夐聞縺ｮ directed edge set 繧・O(1) membership 縺ｧ謇ｱ縺医ｋ繧医≧縺ｫ縺吶ｋ縲・
  - `trie` blocker 縺ｨ迢ｬ遶九↓縲］ested owner 繧帝∩縺代◆ raw bit matrix 縺ｧ collection 縺ｮ遞ｮ鬘槭ｒ蠅励ｄ縺吶�・
- 螟画峩:
  - `stdlib/alloc/collections/adjacency_matrix.nepl`
    - `AdjacencyMatrix` 繧定ｿｽ蜉�縲・
    - `new` / `len` / `contains` / `insert` / `remove` / `clear` / `free` 繧・bare API 縺ｧ螳溯｣・�・
    - `(from, to)` 繧・`from * nverts + to` 縺ｮ bit index 縺ｫ蜀吝ワ縺励�｜yte 驟榊・縺ｧ菫晄戟縺吶ｋ directed graph 縺ｨ縺励◆縲・
    - doc comment 縺ｯ譁ｰ policy / format 縺ｫ蜷医ｏ縺帙�∝推 public 髢｢謨ｰ縺ｫ usage doctest 繧定ｿｽ蜉�縲・
  - `stdlib/tests/adjacency_matrix.n.md`
    - insert/remove/clear 縺ｮ focused fixture 繧定ｿｽ蜉�縲・
  - `tests/stdlib/adjacency_matrix_collections.n.md`
    - pipe 險俶ｳ輔〒縺ｮ `insert` / `remove` / `contains` / `clear` 蛻ｩ逕ｨ繧貞屓蟶ｰ縺ｨ縺励※霑ｽ蜉�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/adjacency_matrix.nepl -i stdlib/tests/adjacency_matrix.n.md -i tests/stdlib/adjacency_matrix_collections.n.md --no-stdlib --no-tree -o /tmp/tests-adjacency-matrix.json -j 2`
    - 邨先棡: `9/9 pass`
- 蟾ｮ逡ｰ繝｡繝｢:
  - `contains g 4 0` 縺ｮ繧医≧縺ｪ遽・峇螟・index 縺ｫ蟇ｾ縺吶ｋ `Result::Err` 邨瑚ｷｯ縺ｯ縲～target/debug/nepl-cli + wasmer` 縺ｧ縺ｯ豁｣蟶ｸ縺ｫ `1` 繧定ｿ斐☆荳�譁ｹ縲『eb compile path 縺ｧ縺ｯ runtime OOB 縺ｫ關ｽ縺｡縺溘�・
  - 縺薙ｌ縺ｯ `AdjacencyMatrix` 螳溯｣・〒縺ｯ縺ｪ縺・web compiler/runtime 蛛ｴ縺ｮ蛻･譬ｹ蝗�縺ｨ蛻､譁ｭ縺励�∽ｻ雁屓縺ｮ collection batch 縺ｫ縺ｯ豺ｷ縺懊※縺・↑縺・�・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (feat(collections): add counting bloom filter)

- 逶ｮ逧・
  - `alloc/collections` 縺ｫ `CountingBloomFilter` 繧定ｿｽ蜉�縺励�～BloomFilter` 縺ｨ蜷後§ hasher 險ｭ險医ｒ菫昴■縺ｪ縺後ｉ蜑企勁蜿ｯ閭ｽ縺ｪ霑台ｼｼ membership structure 繧呈ｨ呎ｺ悶〒謇ｱ縺医ｋ繧医≧縺ｫ縺吶ｋ縲・
  - bare API 縺ｨ public doctest 繧・reboot 譁ｹ驥昴↓蜷医ｏ縺帙�｝ipe 騾｣骼悶・ `tests/stdlib` 蛛ｴ縺ｧ菫晁ｨｼ縺吶ｋ縲・
- 螟画峩:
  - `stdlib/alloc/collections/counting_bloom_filter.nepl`
    - `CountingBloomFilter<.T,.H>` 繧定ｿｽ蜉�縲・
    - `new` / `len` / `insert` / `remove` / `contains` / `clear` / `free` 繧・bare API 縺ｧ螳溯｣・�・
    - counter 縺ｯ `u8` 驟榊・縺ｨ縺励�・ 譛ｬ縺ｮ probe index 縺ｫ蟇ｾ縺励※ insert 縺ｯ鬟ｽ蜥悟刈邂励�〉emove 縺ｯ 0 縺ｾ縺ｧ縺ｮ貂帷ｮ励ｒ陦後≧縲・
  - `stdlib/tests/counting_bloom_filter.n.md`
    - insert/remove/clear 縺ｮ focused fixture 繧定ｿｽ蜉�縲・
  - `tests/stdlib/counting_bloom_filter_collections.n.md`
    - pipe 險俶ｳ輔〒縺ｮ `insert` / `remove` / `contains` / `clear` 蛻ｩ逕ｨ繧貞屓蟶ｰ縺ｨ縺励※霑ｽ蜉�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/tests/counting_bloom_filter.n.md -i tests/stdlib/counting_bloom_filter_collections.n.md -i stdlib/alloc/collections/counting_bloom_filter.nepl --no-stdlib --no-tree -o /tmp/tests-counting-bloom-filter.json -j 2`
    - 邨先棡: `8/8 pass`
- 蟾ｮ逡ｰ繝｡繝｢:
  - `new DefaultHash32 0` 縺ｮ invalid length `Result::Err` 邨瑚ｷｯ縺ｯ縲～target/debug/nepl-cli + wasmer` 縺ｧ縺ｯ豁｣蟶ｸ縺ｫ `1` 繧定ｿ斐☆荳�譁ｹ縲『eb compile path 縺ｧ縺ｯ runtime OOB 縺ｫ關ｽ縺｡縺溘�・
  - 縺薙ｌ縺ｯ `CountingBloomFilter` 螳溯｣・〒縺ｯ縺ｪ縺・web compiler/runtime 蛛ｴ縺ｮ蛻･譬ｹ蝗�縺ｨ蛻､譁ｭ縺励�∽ｻ雁屓縺ｮ collection batch 縺ｫ縺ｯ豺ｷ縺懊※縺・↑縺・�・
  - `node nodesrc/run_doctest.js -i stdlib/tests/bitset.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/bitset.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/bitset_collections.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/bitset.n.md -i tests/stdlib/bitset_collections.n.md -i stdlib/alloc/collections/bitset.nepl --no-stdlib --no-tree -o /tmp/tests-bitset-fixed.json -j 2`
    - 邨先棡: `10/10 pass`
- 蟾ｮ逡ｰ繝｡繝｢:
  - out-of-bounds `Err` 繧定ｿ斐☆ focused case 縺ｯ縲『eb compiler 縺檎函謌舌＠縺・current wasm 縺ｧ hang 縺吶ｋ蛻･譬ｹ蝗�縺ｫ蠖薙◆縺｣縺溘◆繧√�√％縺ｮ batch 縺ｫ縺ｯ豺ｷ縺懊※縺・↑縺・�・
  - `nepl-cli + wasmer` 縺ｧ縺ｯ蜷後§譛�蟆丞・迴ｾ縺悟叉邨ゆｺ・☆繧九％縺ｨ繧堤｢ｺ隱肴ｸ医∩縺ｧ縲《tdlib 螳溯｣・〒縺ｯ縺ｪ縺・compiler/runtime 蛛ｴ縺ｮ蛻･繧ｿ繧ｹ繧ｯ縺ｨ縺励※蛻・ｊ蜃ｺ縺吶�・

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm codegen 蜀・・ precheck 蠕瑚ｨｺ譁ｭ霑泌唆繧帝勁蜴ｻ)

- 逶ｮ逧・
  - `precheck` 螳溯｡悟ｾ後↓ `codegen_llvm` 縺・`TypecheckFailed` 繧定ｿ斐＠縺ｦ縺・◆谿句ｭ倡ｵ瑚ｷｯ繧帝勁蜴ｻ縺励�∝燕谿ｵ讀懈渊荳榊､画擅莉ｶ縺ｸ邨ｱ荳�縺吶ｋ縲・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - `emit_ll_from_module_for_target` 蜀・・ `select_active_raw_body(... )` `Err(diag)` 蛻・ｲ舌ｒ `TypecheckFailed` 霑泌唆縺九ｉ internal panic 縺ｸ螟画峩縲・
    - 縺薙ｌ縺ｫ繧医ｊ縲〉aw-body 驕ｸ謚槫､ｱ謨励・蜑肴ｮｵ `target_precheck::precheck_module_before_codegen` 縺ｧ縺ｮ縺ｿ險ｺ譁ｭ縺輔ｌ縲…odegen 蛻ｰ驕泌ｾ後・逕滓・蟆ゆｻｻ縺ｫ縺ｪ繧九�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-after-llvm-invariant-2.json -j 15` -> `8/8 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-precheck-invariant.json -j 15` -> `791/791 pass`

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm precheck 蝗槫ｸｰ繧ｱ繝ｼ繧ｹ縺ｮ霑ｽ蜉�)

- 逶ｮ逧・
  - LLVM backend 蛻ｰ驕泌燕縺ｫ譛ｪ蟇ｾ蠢・intrinsic 繧定ｨｺ譁ｭ縺ｧ縺阪ｋ縺薙→繧貞屓蟶ｰ蝗ｺ螳壹☆繧九�・
- 螟画峩:
  - `tests/llvm_target.n.md`
    - `llvm_precheck_rejects_wasm_only_intrinsic` 繧定ｿｽ蜉�縲・
    - `#intrinsic "i32_add"` 繧・`#target llvm` 縺ｧ菴ｿ縺｣縺溷�ｴ蜷医↓ `diag_id: 3012` 繧呈悄蠕・☆繧・compile_fail 繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/llvm_target.n.md --no-stdlib --no-tree --runner all --llvm-all -o /tmp/tests-llvm-target-after-precheck-case.json -j 15`
    - 霑ｽ蜉�繧ｱ繝ｼ繧ｹ・・doctest#6::llvm`・峨・ pass縲・
    - 譌｢蟄倥こ繝ｼ繧ｹ `doctest#4/#5` 縺ｯ `invalid redefinition of function 'add'` 縺ｧ fail・域里遏･譛ｪ隗｣豎ｺ・峨�・
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-test-add.json -j 15` -> `791/791 pass`

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: allocator helper 隗｣豎ｺ縺ｮ諢丞袖隲紋ｿｮ豁｣)

- 逶ｮ逧・
  - runtime helper 蜈ｱ騾壼喧蠕後↓逋ｺ逕溘＠縺・run-time 螟ｱ謨・(`unreachable` / `memory access out of bounds`) 繧偵�・俣縺ｫ蜷医ｏ縺帙〒縺ｯ縺ｪ縺・helper 隗｣豎ｺ縺ｮ諢丞袖隲悶°繧我ｿｮ豁｣縺吶ｋ縲・
- 蜴溷屏:
  - `alloc`・亥ｮ牙・API・峨→ `alloc_raw`・井ｽ弱Ξ繝吶ΝAPI・峨・迴ｾ迥ｶ縺ｮ lowering 縺ｧ縺ｯ蝙倶ｺ呈鋤縺ｫ縺ｪ繧翫≧繧九◆繧√�～ALLOC_CANDIDATES=["alloc","alloc_raw"]` 縺ｸ螟画峩縺吶ｋ縺ｨ backend 蜀・Κ遒ｺ菫昴〒隱､縺｣縺ｦ `alloc` 繧呈雫繧�邨瑚ｷｯ縺檎匱逕溘☆繧九�・
  - 縺昴・邨先棡縲∝・驛ｨ遒ｺ菫昴・蜑肴署・育函繝昴う繝ｳ繧ｿ霑泌唆・峨→蜷医ｏ縺壹�∝ｮ溯｡梧凾縺ｫ `unreachable` / OOB 縺檎匱逕溘＠縺溘�・
- 螟画峩:
  - `nepl-core/src/runtime_helpers.rs`
    - `ALLOC_CANDIDATES` 繧・`["alloc_raw", "alloc"]` 縺ｫ謌ｻ縺励�∝・驛ｨ helper 隗｣豎ｺ縺ｯ逕溘・繧､繝ｳ繧ｿ諢丞袖隲悶ｒ蜆ｪ蜈医☆繧九ｈ縺・ｿｮ豁｣縲・
    - 蜊倅ｽ薙ユ繧ｹ繝域悄蠕・�､繧・raw 蜆ｪ蜈医∈譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-alloc-order-fix.json -j 15` -> `791/791 pass`

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: runtime helper 隗｣豎ｺ縺ｮ蜈ｱ騾壼喧縺ｨ raw 萓晏ｭ倡ｸｮ蟆・

- 逶ｮ逧・
  - `nepl-core` 蜀・〒驥崎､・＠縺ｦ縺・◆ runtime helper・・lloc/dealloc/realloc・芽ｧ｣豎ｺ繝ｭ繧ｸ繝・け繧貞・騾壼喧縺励�～_raw` 蜷堺ｾ晏ｭ倥ｒ谿ｵ髫守ｸｮ蟆上☆繧九�・
  - helper 蜷阪・蜆ｪ蜈磯�・ｽ阪ｒ螳牙・API蜷搾ｼ・uffix縺ｪ縺暦ｼ牙━蜈医∈邨ｱ荳�縺吶ｋ縲・
- 螟画峩:
- `nepl-core/src/runtime_helpers.rs`
    - `ALLOC_CANDIDATES` 繧・`["alloc", "alloc_raw"]` 縺ｫ螟画峩・亥ｮ牙・API蜆ｪ蜈茨ｼ峨�・
    - `RuntimeHelperKind` / `helper_candidates` / `helper_base_name` 繧定ｿｽ蜉�縲・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (trait 閭ｽ蜉帙Δ繝・Ν: `Eq` / `Ord` 縺ｮ蜈ｱ騾壼喧)

- 逶ｮ逧・
  - `core/traits` 縺ｫ `Eq` / `Ord` 繧定ｿｽ蜉�縺励�∵ｯ碑ｼ・э蜻ｳ隲悶ｒ stdlib 蜈ｱ騾・trait 縺ｨ縺励※謇ｱ縺医ｋ繧医≧縺ｫ縺吶ｋ縲・
  - `alloc/collections/vec/sort.nepl` 縺ｮ螻�謇� `Ord` 螳夂ｾｩ繧呈彫蜴ｻ縺励�…ollections 蛛ｴ縺ｮ豈碑ｼ・capability 繧・`core` 縺ｸ蟇・○繧九�・
- 螟画峩:
  - `stdlib/core/traits/eq.nepl`
    - `Eq` trait
    - `eq_by_trait`
    - `ne_by_trait`
    - `bool`, `i32`, `u8`, `i64`, `f32`, `f64`, `str` 縺ｸ縺ｮ impl
  - `stdlib/core/traits/ord.nepl`
    - `Ord` trait
    - `ord_lt`, `ord_le`, `ord_gt`, `ord_ge`
    - `bool`, `i32`, `u8`, `i64`, `i128`, `f32`, `f64` 縺ｸ縺ｮ impl
  - `stdlib/alloc/collections/vec/sort.nepl`
    - 螻�謇� `Ord` trait 縺ｨ螻�謇� impl 繧貞炎髯､
    - `core/traits/ord` 繧・import 縺励�～sort_lt` 邉ｻ helper 縺九ｉ蜈ｱ騾・`ord_*` 繧貞他縺ｶ蠖｢縺ｸ螟画峩
  - `tests/stdlib/traits_order.n.md`
    - 譌･譛ｬ隱槭・逶ｮ逧・▽縺・focused test 繧定ｿｽ蜉�
- 蛻､譁ｭ:
  - `Eq<i128>` 縺ｯ譌｢蟄倥・蛻・ｧ｣ helper 繧剃ｻｮ螳壹☆繧九→螢翫ｌ繧九◆繧√�∽ｸ�譌ｦ霑ｽ蜉�縺励↑縺九▲縺溘�・
  - `Ord<str>` 繧よ里蟄倥・鬆・ｺ乗ｯ碑ｼ・helper 縺梧悴謨ｴ蛯吶↑縺ｮ縺ｧ縲∝酔讒倥↓隕矩�√▲縺溘�・
  - 縺ｾ縺壹・譌｢蟄倥・ `core/math` overload 縺ｧ譬ｹ諡�繧呈戟縺ｦ繧句梛縺�縺代ｒ蜈ｱ騾・trait 蛹悶＠縺溘�・
- 讀懆ｨｼ:
  - `NODE_NO_WARNINGS=1 node nodesrc/run_test.js`
    - `Eq` / `Ord` core focused case: pass
    - `vec/sort` + `Ord` std focused case: pass

# 2026-03-09 菴懈･ｭ繝｡繝｢ (trait 閭ｽ蜉帙Δ繝・Ν: `Hash` 縺ｮ蜈ｱ騾壼喧)

- 逶ｮ逧・
  - `Hash` trait 繧・`core/traits` 縺ｸ霑ｽ蜉�縺励�”ashmap / hashset 縺悟・菴鍋噪縺ｪ `hash32_i32` / `hash32_str` 縺ｸ逶ｴ謗･萓晏ｭ倥○縺壼・騾・helper 邨檎罰縺ｧ繧ｭ繝ｼ繧呈ｷｷ蜷医〒縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
  - 蟆・擂縺ｮ `Serialize` / `Deserialize` 縺ｨ蜷後§縺上�∝梛縺斐→縺ｮ閭ｽ蜉帙ｒ stdlib trait 縺ｨ縺励※譏守､ｺ縺吶ｋ豬√ｌ繧呈純縺医ｋ縲・
- 螟画峩:
  - `stdlib/core/traits/hash.nepl`
  - `Hash` trait
  - `hash32_by_trait`
  - `i32`

# 2026-03-11 菴懈･ｭ繝｡繝｢ (`streamio` target 謖・ｮ壼喧縺ｨ `u32/u64` bare I/O 縺ｮ菫ｮ豁｣)

- 逶ｮ逧・
  - `scanner` / `writer` 繧・stdin/stdout 蝗ｺ螳壹・ no-arg API 縺九ｉ螟悶＠縲～io_stdin` / `io_stdout` / `io_text` / `io_bytes` 縺ｮ target 謖・ｮ壹〒逕滓・縺吶ｋ蠖｢縺ｸ蟇・○繧九�・
  - `u32` / `u64` 縺ｮ bare `read` / `write` 繧偵�∝梛 suffix 蜷阪↓謌ｻ縺輔★ current overload 譁ｹ驥昴・縺ｾ縺ｾ螳牙ｮ壼喧縺吶ｋ縲・
  - Part6 tutorial 縺ｨ `kp` 蜻ｨ霎ｺ縺ｫ谿九▲縺ｦ縺・◆ old move-model 蜑肴署繧偵�∫樟陦梧園譛画ｨｩ繝｢繝・Ν縺ｸ蜷医ｏ縺帙ｋ縲・
- 蜴溷屏:
  - `std/streamio` 縺�縺・`read` / `write` 縺ｮ bare 蜷阪∈蟇・○縺ｦ繧ゅ�∫函謌仙・蜿｣ `scanner()` / `writer()` 縺・stdin/stdout 蝗ｺ螳壹・縺ｾ縺ｾ縺�縺ｨ縲～std/io` / `iotarget` 縺ｨ雋ｬ蜍吶′莠碁㍾蛹悶＠縺ｦ縺・◆縲・
  - `u64` 縺ｯ compiler 蛛ｴ縺ｧ `wasm_shared::valtype` 縺後∪縺� `i32` 謇ｱ縺・・邂・園繧呈ｮ九＠縺ｦ縺翫ｊ縲仝asm signature 縺悟ｴｩ繧後※縺・◆縲・
  - `u32` / `u64` 縺ｮ 10 騾ｲ蜃ｺ蜉帙・縲「nsigned 蛟､繧・signed overload 縺ｸ關ｽ縺ｨ縺励※縺・◆縺溘ａ `4294967295` 縺・`18446744073709551615` 縺ｫ蛹悶￠縺ｦ縺・◆縲・
  - `PrefixI32` 繧・tutorial Part6 縺ｮ `Vec` 襍ｰ譟ｻ縺ｫ縺ｯ old move-model 蜑肴署縺梧ｮ九▲縺ｦ縺・◆縲・
- 螟画峩:
  - `stdlib/std/streamio.nepl`
    - `scanner <(IoReadTarget)*>Result<StreamScanner,str>>`
    - `writer <(IoWriteTarget)*>Result<StreamWriter,str>>`
    - `scanner_from_bytes`
    - `StreamWriter` header 縺ｫ `TargetKind` 繧定ｿｽ蜉�
    - `u32` / `u64` 縺ｮ append 螳溯｣・ｒ unsigned decimal 縺ｨ縺励※菫ｮ豁｣
    - `StreamScanner` / `StreamWriter` 縺ｮ doc comment 繧・current 螳溯｣・∈蜷梧悄
  - `stdlib/std/iotarget.nepl`
    - `io_stdin` / `io_stdout` / `io_text` / `io_bytes` 繧堤函謌仙・蜿｣縺ｨ縺励※蛻ｩ逕ｨ
  - `nepl-core/src/wasm_shared.rs`
    - `u64` 繧・Wasm `I64` 縺ｨ縺励※謇ｱ縺・ｈ縺・ｿｮ豁｣
  - `nodesrc/run_test.js`
    - `BigInt` 縺ｮ JSON 蜃ｺ蜉帙→ return decode 繧定ｿｽ蜉�
  - `stdlib/kp/kpprefix.nepl`
    - `PrefixI32` 縺ｫ `Copy` / `Clone` 繧剃ｻ倅ｸ・
    - `prefix_build_vec_i32` 繧・`vec_data_len` 繝吶・繧ｹ縺ｸ菫ｮ豁｣
  - `tests/stdlib/streamio.n.md`
  - `tests/stdlib/kp.n.md`
  - `tests/stdlib/kp_i64.n.md`
  - `tests/stdlib/stdin.n.md`
  - `tests/compiler/move_effect.n.md`
  - `tutorials/getting_started/22_competitive_io_and_arith.n.md`
  - `tutorials/getting_started/24_competitive_dp_basics.n.md`
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
  - `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
  - `stdlib/kp/kpgraph.nepl`
    - `unwrap_ok scanner io_stdin` / `unwrap_ok writer io_stdout` 縺ｸ邨ｱ荳�
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`
  - `node nodesrc/run_doctest.js -i /tmp/u64_probe2.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 4`
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 10`
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp_i64.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 2`
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 3`
  - `node nodesrc/run_doctest.js -i tests/stdlib/stdin.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/22_competitive_io_and_arith.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/24_competitive_dp_basics.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -n 1`
  - `node nodesrc/run_doctest.js -i stdlib/kp/kpgraph.nepl -n 1`
  - `node nodesrc/run_doctest.js -i stdlib/kp/kpprefix.nepl -n 1`
  - `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 20`

# 2026-03-09 菴懈･ｭ繝｡繝｢ (compiler 蜑肴署蝗ｺ螳・ `#prelude` 譛�蟆丞ｮ溯｣・→ Copy 蝗ｺ螳夊｡ｨ謦､蜴ｻ)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `todo.md` 縺ｮ `compiler 蜑肴署` 谿倶ｻｶ縺�縺｣縺・`Copy` 蝗ｺ螳夊｡ｨ萓晏ｭ倥ｒ縲ー螳滄圀/縺倥▲縺輔＞]縺ｫ source [蛛ｴ/縺後ｏ]縺九ｉ trait impl 繧端萓帷ｵｦ/縺阪ｇ縺・″繧・≧]縺ｧ縺阪ｋ[迥ｶ諷・縺倥ｇ縺・◆縺Ь縺ｸ[遘ｻ/縺・▽]縺吶�・
  - parser 縺�縺代↓[蟄伜惠/縺昴ｓ縺悶＞]縺励※縺・◆ `#prelude` / `#no_prelude` 繧・loader [谿ｵ髫・縺�繧薙°縺Ь縺ｧ繧・隗｣驥・縺九＞縺励ｃ縺従縺励�…opy/clone 髱槭ワ繝ｼ繝峨さ繝ｼ繝牙喧縺ｮ[蜑肴署/縺懊ｓ縺ｦ縺Ь繧端謨ｴ/縺ｨ縺ｨ縺ｮ]縺医ｋ縲・
- [蜴溷屏/縺偵ｓ縺・ｓ]:
  - `#prelude` 縺ｨ `#no_prelude` 縺ｯ lexer / parser / AST 縺ｫ縺�縺措蟄伜惠/縺昴ｓ縺悶＞]縺励�〕oader 縺ｧ縺ｯ[辟｡隕・繧�縺余縺輔ｌ縺ｦ縺・◆縲・
  - 縺昴・縺溘ａ `Copy` / `Clone` impl 繧・source [蛛ｴ/縺後ｏ]縺九ｉ[譌｢螳・縺阪※縺Ь[萓帷ｵｦ/縺阪ｇ縺・″繧・≧]縺ｧ縺阪★縲～TypeCtx::is_copy` 縺ｫ primitive 蝗ｺ螳夊｡ｨ繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ繧端谿・縺ｮ縺転縺兌蠢・ｦ・縺ｲ縺､繧医≧]縺後≠縺｣縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nepl-core/src/loader.rs`
    - root module [髯仙ｮ・縺偵ｓ縺ｦ縺Ь縺ｧ `#prelude` / `#no_prelude` 繧端蜃ｦ逅・縺励ｇ繧馨縺吶ｋ繧医≧縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `#no_prelude` 縺後↑縺・root module 縺ｫ縺ｯ[譌｢螳・縺阪※縺Ь縺ｧ `std/prelude_base` 繧端隱ｭ/繧・縺ｿ[霎ｼ/縺転繧�縲・
    - import/include 縺ｮ[蜀榊ｸｰ/縺輔＞縺江 load 縺ｧ縺ｯ default prelude 繧端驕ｩ逕ｨ/縺ｦ縺阪ｈ縺・縺励↑縺・ｈ縺・↓縺励※縲《tdlib [蜀・Κ/縺ｪ縺・・] import 縺ｧ縺ｮ[蠕ｪ迺ｰ/縺倥ｅ繧薙°繧転繧端驕ｿ/縺評縺代◆縲・
  - `stdlib/std/prelude_base.nepl`
    - [譛�蟆・縺輔＞縺励ｇ縺・ prelude 縺ｨ縺励※[霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - [蠖馴擇/縺ｨ縺・ａ繧転縺ｯ `core/traits/copy` 縺�縺代ｒ[隱ｭ/繧・縺ｿ[霎ｼ/縺転縺ｿ縲…opy/clone 閭ｽ蜉帙・ source [萓帷ｵｦ/縺阪ｇ縺・″繧・≧]縺ｫ[邨・縺励⊂]縺｣縺溘�・
  - `nepl-core/src/types.rs`
    - `TypeCtx::is_copy` 縺ｮ譛�邨ゅヵ繧ｩ繝ｼ繝ｫ繝舌ャ繧ｯ縺九ｉ primitive 蝗ｺ螳夊｡ｨ繧端蜑企勁/縺輔￥縺倥ｇ]縺励◆縲・
    - `Copy` trait 縺啓隕・縺ｿ]縺医※縺・↑縺Ъ蝣ｴ蜷・縺ｰ縺ゅ＞]縺ｯ縲ー蜿ら・/縺輔ｓ縺励ｇ縺・蝙九→ `Never` 縺�縺代ｒ compiler [蜀・惠/縺ｪ縺・＊縺Ь縺ｮ copy 縺ｨ縺励※[謇ｱ/縺ゅ▽縺犠縺・�・
  - `tests/compiler/prelude_copy.n.md`
    - default prelude 縺ｧ `Copy` bound 縺啓騾・縺ｨ縺馨繧九％縺ｨ繧端遒ｺ隱・縺九￥縺ｫ繧転縺吶ｋ focused case 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `#prelude std/prelude_base` 縺ｨ `#no_prelude` 繧端菴ｵ險・縺ｸ縺・″]縺励※繧ゅ�ー譏守､ｺ逧・繧√＞縺倥※縺江 prelude 縺啓蜆ｪ蜈・繧・≧縺帙ｓ]縺輔ｌ繧九％縺ｨ繧端蝗ｺ螳・縺薙※縺Ь縺励◆縲・
    - `#no_prelude` 縺�縺代〒縺ｯ `Copy` trait [萓帷ｵｦ/縺阪ｇ縺・″繧・≧]縺啓豸・縺江縺医�～.T: Copy` 縺・`3073` 縺ｧ[關ｽ/縺馨縺｡繧九％縺ｨ繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/compiler/prelude_copy.n.md -i tests/compiler/resolve.n.md --no-stdlib --no-tree -o /tmp/tests-prelude-only.json -j 15` -> `14/14 pass`
  - `node nodesrc/tests.js -i tests/compiler/prelude_copy.n.md --no-stdlib --no-tree -o /tmp/tests-prelude-copy-only.json -j 15` -> `3/3 pass`
- [蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `Copy` 縺ｮ source [萓帷ｵｦ/縺阪ｇ縺・″繧・≧]縺ｯ default prelude 繧端騾・縺ｨ縺馨縺吶％縺ｨ縺ｧ[譌｢蟄・縺阪◇繧転繧ｳ繝ｼ繝峨ｒ[螢・縺薙ｏ]縺輔★縺ｫ[遘ｻ陦・縺・％縺・縺ｧ縺阪ｋ縲・
  - `#no_prelude` 縺ｯ縲梧ｨ呎ｺ・capability 繧端蜷ｫ/縺ｵ縺従繧√※閾ｪ蜑阪〒[邂｡逅・縺九ｓ繧馨縺吶ｋ縲阪◆繧√・ opt-out 縺ｨ縺励※[讖溯・/縺阪・縺・縺吶ｋ縲・
    - `bool`
    - `u8`
    - `i64`
    - `str`
    縺ｸ縺ｮ impl 繧定ｿｽ蜉�縲・
  - `stdlib/alloc/collections/hashmap.nepl`
    - `hash32_i32` / `hash32_str` 縺ｮ逶ｴ謗･蜻ｼ縺ｳ蜃ｺ縺励ｒ `hash32_by_trait` 縺ｫ鄂ｮ謠帙�・
  - `stdlib/alloc/collections/hashset.nepl`
    - 蜷梧ｧ倥↓ `hash32_by_trait` 邨檎罰縺ｸ鄂ｮ謠帙�・
  - `tests/stdlib/traits_hash.n.md`
    - `[逶ｮ逧・繧ゅ￥縺ｦ縺江` 縺､縺・focused case 繧定ｿｽ蜉�縲・
- 蛻､譁ｭ:
  - `Hash<i64>` 縺ｯ [荳贋ｽ・縺倥ｇ縺・＞] / [荳倶ｽ・縺九＞] 32-bit 繧・XOR 縺ｧ謚倥ｊ縺溘◆繧薙〒縺九ｉ `hash32_i32` 縺ｸ豬√☆縲・
  - `Hash` 縺ｮ蟇ｾ雎｡縺ｯ縲√∪縺壽里蟄・stdlib 縺悟ｮ牙ｮ壹＠縺ｦ謾ｯ縺医※縺・ｋ繧ｭ繝ｼ蝙九↓髯仙ｮ壹＠縺溘�・
  - `i128` 繧・峡閾ｪ讒矩��菴薙・繝上ャ繧ｷ繝･閭ｽ蜉帙・縲∽ｻ雁ｾ・`Serialize` / `Eq` 縺ｨ縺ｮ謨ｴ蜷医ｒ隕九↑縺後ｉ霑ｽ蜉�縺吶ｋ縲・
- compiler 菫ｮ豁｣:
  - 縺ｪ縺励�ゆｻ雁屓縺ｮ遒ｺ隱阪〒隕九▽縺九▲縺溷撫鬘後・ `traits_hash.n.md` 蛛ｴ縺ｮ API 繧ｵ繝ｳ繝励Ν縺檎樟陦・`hashmap` / `hashset` 縺ｮ蛻ｩ逕ｨ豬∝о縺ｨ縺壹ｌ縺ｦ縺・◆縺薙→縺�縺｣縺溘�・
  - `must_hm` / `must_hs` 縺ｨ `Option` 縺ｮ match 繧剃ｽｿ縺・里蟄俶ｵ∝о縺ｸ蜷医ｏ縺帙※菫ｮ豁｣縺励◆縲・
- 讀懆ｨｼ:
  - `node` + `nodesrc/compiler_loader` 縺ｫ繧医ｋ compile-only focused check 縺ｧ縲・
    - `hash32_by_trait` 蜊倅ｽ・
    - `hashmap/hashset/hashmap_str/hashset_str`
    繧剃ｽｿ縺・snippet
    縺ｮ荳｡譁ｹ縺・`COMPILE_OK` 繧定ｿ斐☆縺薙→繧堤｢ｺ隱阪�・
  - `nodesrc/tests.js` 縺ｯ縺薙・迺ｰ蠅・〒縺ｯ髟ｷ縺上・繧我ｸ九′繧九％縺ｨ縺後≠繧九◆繧√�’ocused 縺ｪ compile-only 縺ｧ縺ｾ縺壼ｦ･蠖捺�ｧ繧貞崋螳壹＠縺溘�・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (`std/test` 髮・ｴ・API 霑ｽ蜉�縺ｨ nested generic overload 譬ｹ譛ｬ菫ｮ豁｣)

- 逶ｮ逧・
  - stdlib reboot 蜑阪・繝・せ繝亥渕逶､縺ｨ縺励※縲・ 莉ｶ螟ｱ謨励＠縺ｦ繧よｮ九ｊ縺ｮ讀懈渊繧堤ｶ咏ｶ壼ｮ溯｡後〒縺阪ｋ `std/test` 縺ｮ collectable API 繧呈紛蛯吶☆繧九�・
  - `Vec<Result<(),str>>` 縺ｫ `push` / `vec_push` / pipe 縺ｧ `Result<(),str>` 繧堤ｩ阪ａ縺ｪ縺・compiler 繝舌げ繧偵�〕ibrary 蛛ｴ縺ｮ蝗樣∩縺ｧ縺ｯ縺ｪ縺・typecheck 縺ｮ譬ｹ譛ｬ蜴溷屏縺九ｉ菫ｮ豁｣縺吶ｋ縲・
- 螟画峩:
  - `stdlib/std/test.nepl`
    - `checks_new`
    - `checks_push`
    - `check`
    - `check_eq_i32`
    - `check_ne`
    - `check_str_eq`
    - `check_ok_i32`
    - `check_err_i32`
    - `check_status_str`
    - `checks_has_err(_loop)`
    - `checks_summary(_loop)`
    - `checks_report_failures`
    - `finish_checks`
    繧定ｿｽ蜉�縺励◆縲・
    - `assert` / `assert_eq_i32` / `assert_ne` / `assert_str_eq` 縺ｯ縲∝ｯｾ蠢懊☆繧・`check_*` 繧貞女縺代※蜊ｳ譎ょ､ｱ謨励☆繧玖埋縺・Λ繝・ヱ縺ｸ謨ｴ逅・＠縺溘�・
  - `tests/std_test_collect.n.md`
    - `[逶ｮ逧・繧ゅ￥縺ｦ縺江` 縺ｨ `[菴・縺ｪ縺ｫ]繧端遒ｺ/縺溘＠]縺九ａ繧九°` 繧剃ｻ倥￠縺・focused case 繧定ｿｽ蜉�縺励◆縲・
    - 蜈ｨ莉ｶ謌仙粥譎ゅ・ summary 蜃ｺ蜉帙→縲∝､ｱ謨励ｒ蜷ｫ繧�縺ｨ縺阪・ summary + 蛟句挨螟ｱ謨怜・蜉帙ｒ蝗ｺ螳壹＠縺溘�・
  - `tests/compiler/overload_nested_generic_push.n.md`
    - `Vec<Result<(),str>>` 縺ｫ蟇ｾ縺吶ｋ `push` / `vec_push` / pipe 縺ｮ nested generic overload 隗｣豎ｺ繧堤｢ｺ隱阪☆繧・compiler 蝗槫ｸｰ test 繧定ｿｽ蜉�縺励◆縲・
  - `nepl-core/src/types.rs`
    - 髢｢謨ｰ蝙九↓蜷ｫ縺ｾ繧後ｋ蝙句､画焚 binding 繧帝��驕ｿ繝ｻ蠕ｩ蜈・☆繧・
      - `snapshot_type_var_bindings`
      - `restore_type_var_bindings`
      繧定ｿｽ蜉�縺励◆縲・
  - `nepl-core/src/typecheck.rs`
    - `check_function` 縺ｧ髢｢謨ｰ譛ｬ菴薙ｒ讀懈渊縺吶ｋ蜑阪↓ `func_ty` 荳翫・蝙句､画焚 binding 繧・snapshot 縺励�∫ｵゆｺ・ｾ後↓蠢・★ restore 縺吶ｋ繧医≧螟画峩縺励◆縲・
- 蜴溷屏:
  - generic 髢｢謨ｰ譛ｬ菴薙・蝙区､懈渊荳ｭ縺ｫ縲・未謨ｰ繧ｷ繧ｰ繝阪メ繝｣閾ｪ菴薙′謖√▲縺ｦ縺・ｋ蝙句､画焚 `TypeId` 縺・unification 縺ｧ譚溽ｸ帙＆繧後�√◎縺ｮ譚溽ｸ帙′ `Env` 荳翫・螟ｧ蝓滄未謨ｰ蝙九∈谿狗蕗縺励※縺・◆縲・
  - 縺昴・邨先棡縲～vec_push <.T> <(Vec<.T>, .T)->Vec<.T>>` 縺ｮ `.T` 縺碁℃蜴ｻ縺ｮ讀懈渊縺ｧ `i32` 縺ｸ豎壽沒縺輔ｌ縲～Vec<Result<(),str>>` 縺ｫ蟇ｾ縺吶ｋ overload 謗ｨ隲悶〒 `Vec<i32>` 縺ｨ縺励※謇ｱ繧上ｌ縺ｦ縺・◆縲・
  - 譏守､ｺ蝙句ｼ墓焚莉倥″ `vec_push<Result<(),str>>` 縺碁�壹ｊ縲∝梛蠑墓焚逵∫払譎ゅ□縺題誠縺｡繧九％縺ｨ縺九ｉ縲…andidate 驕ｸ謚樊凾縺ｮ `instantiate(binding.ty)` 蜈･蜉帙′譌｢縺ｫ豎壽沒縺輔ｌ縺ｦ縺・ｋ縺ｨ迚ｹ螳壹＠縺溘�・
- 邨先棡:
  - `std/test` 縺ｮ collectable API 縺ｧ縲～[ok,ok,err,ok,err]` 蠖｢蠑上・讎りｦ√→螟ｱ謨玲ｷｻ蟄励・逅・罰繧偵∪縺ｨ繧√※陦ｨ遉ｺ縺ｧ縺阪ｋ繧医≧縺ｫ縺ｪ縺｣縺溘�・
  - nested generic `push` / `vec_push` / pipe 縺ｯ縲∝梛蠑墓焚繧呈・遉ｺ縺励↑縺上※繧・`Vec<Result<(),str>>` 荳翫〒隗｣豎ｺ縺ｧ縺阪ｋ繧医≧縺ｫ縺ｪ縺｣縺溘�・
- 讀懆ｨｼ:
  - `trunk build`・・oot, `NO_COLOR=false`・・-> success
  - `node nodesrc/tests.js -i tests/std_test_collect.n.md -i tests/compiler/overload_nested_generic_push.n.md --no-stdlib --no-tree -o /tmp/tests-std-test-collect-focused.json -j 15`
    - 邨先棡: `5/5 pass`
    - `find_runtime_helper_key`・亥錐蜑崎ｧ｣豎ｺ・峨→ `find_runtime_helper_index`・・ndex隗｣豎ｺ・峨ｒ霑ｽ蜉�縲・
  - `nepl-core/src/codegen_wasm.rs`
    - 繝ｭ繝ｼ繧ｫ繝ｫ螳溯｣・□縺｣縺・helper 蜷崎ｧ｣豎ｺ繧貞炎髯､縺励�～runtime_helpers::find_runtime_helper_index` 縺ｫ邨ｱ荳�縲・
  - `nepl-core/src/monomorphize.rs`
    - helper 菫晄戟繝ｫ繝ｼ繝域爾邏｢繧・`find_runtime_helper_key` + `RuntimeHelperKind` 縺ｸ鄂ｮ謠帙�・
    - 驥崎､・＠縺ｦ縺・◆蜷榊燕繝槭ャ繝・未謨ｰ繧貞炎髯､縲・
  - `nepl-core/src/codegen_llvm.rs`
    - helper 蛟呵｣懷叙蠕励ｒ `helper_candidates(RuntimeHelperKind::...)` 縺ｫ邨ｱ荳�縲・
    - `resolve_symbol_name` 縺ｮ蛟呵｣應ｸ�閾ｴ繧・`helper_base_name` 繝吶・繧ｹ縺ｸ螟画峩縺励�］amespaced/mangled 蜷阪〒繧ょ酔荳�隕丞援縺ｧ隗｣豎ｺ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-helper-unify.json -j 15` -> `791/791 pass`

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm backend 縺ｮ wasm-body 蛻・ｲ舌ｒ荳榊､画擅莉ｶ蛹・

- 逶ｮ逧・
  - `codegen_llvm` 蛛ｴ縺ｫ谿九▲縺ｦ縺・◆ backend 蜈･蜉帙お繝ｩ繝ｼ蛻・ｲ撰ｼ・UnsupportedWasmBody`・峨ｒ蜑肴ｮｵ讀懈渊蜑肴署縺ｸ蟇・○繧九�・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - `LlvmCodegenError` 縺九ｉ `UnsupportedWasmBody` / `UnsupportedParsedFunctionBody` 繧貞炎髯､縲・
    - `emit_ll_from_module_for_target` 蜀・〒 `ActiveRawBody::Wasm` 蛻ｰ驕疲凾縺ｮ `Err` 繧・internal panic 縺ｫ螟画峩縲・
    - `FnBody::Wasm` reachable 蛻ｰ驕疲凾縺ｮ `Err` 繧・internal panic 縺ｫ螟画峩縲・
    - HIR lowering 邨瑚ｷｯ縺ｧ `HirBody::Wasm` 蛻ｰ驕疲凾縺ｮ `Err` 繧・internal panic 縺ｫ螟画峩縲・
    - 蟇ｾ蠢懊ユ繧ｹ繝・`emit_ll_rejects_entry_with_wasm_body` 縺ｯ `TypecheckFailed` 繧呈悄蠕・☆繧句ｽ｢縺ｸ譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-invariant.json -j 15` -> `791/791 pass`

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: wasm codegen 險ｺ譁ｭ霑泌唆邨瑚ｷｯ縺ｮ謦､蜴ｻ)

- 逶ｮ逧・
  - `codegen` 蛻ｰ驕泌ｾ後・逕滓・蟆ゆｻｻ縺ｫ縺吶ｋ譁ｹ驥昴↓蜷医ｏ縺帙�～codegen_wasm` 縺ｮ `Vec<Diagnostic>` 霑泌唆邨瑚ｷｯ繧呈彫蜴ｻ縺吶ｋ縲・
- 螟画峩:
  - `nepl-core/src/codegen_wasm.rs`
    - `lower_body` / `lower_user` 縺ｮ謌ｻ繧雁�､繧・`Result<Function, Vec<Diagnostic>>` 縺九ｉ `Function` 縺ｸ螟画峩縲・
    - `gen_block` / `gen_expr` 縺ｮ `diags` 蠑墓焚繧貞炎髯､縲・
    - `generate_wasm` 縺ｮ code section 逕滓・縺ｧ `Err(ds)` 蛻・ｲ舌ｒ蜑企勁縺励�∝燕谿ｵ讀懈渊騾夐℃蠕後・逶ｴ謗･逕滓・縺吶ｋ蠖｢縺ｫ邨ｱ荳�縲・
    - backend 蜀・ｨｺ譁ｭ縺ｨ縺励※谿九▲縺ｦ縺・◆譛ｪ菴ｿ逕ｨ髢｢謨ｰ `validate_wasm_stack` 繧貞炎髯､縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-after-wasm-no-diag.json -j 15` -> `8/8 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-wasm-no-diag.json -j 15` -> `791/791 pass`

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: wasm helper 隗｣豎ｺ縺ｮ閾ｪ蟾ｱ蜀榊ｸｰ繝舌げ菫ｮ豁｣)

- 逶ｮ逧・
  - `tests + stdlib` 縺ｧ逋ｺ逕溘＠縺ｦ縺・◆ `RangeError: Maximum call stack size exceeded` 繧呈�ｹ譛ｬ蜴溷屏縺九ｉ隗｣豸医☆繧九�・
- 蜀咲樟縺ｨ蛻・ｊ蛻・￠:
  - `option.nepl` doctest 繧貞腰迢ｬ蜀咲樟縺吶ｋ縺ｨ `wasm-function[4]` 縺ｮ閾ｪ蟾ｱ蜀榊ｸｰ縺ｧ蛛懈ｭ｢縲・
  - 蜷御ｸ�繧ｽ繝ｼ繧ｹ繧・`nepl-cli` 縺ｧ逕滓・縺励◆ wasm 縺ｯ豁｣蟶ｸ螳溯｡後�・
  - `web` 逕滓・ WAT 縺ｨ `native` 逕滓・ WAT 繧呈ｯ碑ｼ・☆繧九→縲∝酔荳�邂・園縺ｧ `call 5` 縺・`call 4`・郁・蟾ｱ蜻ｼ縺ｳ蜃ｺ縺暦ｼ峨↓蛹悶￠縺ｦ縺・◆縲・
- 蜴溷屏:
  - `codegen_wasm` 縺ｮ runtime helper 隗｣豎ｺ縺梧尠譏ｧ縺ｪ譁・ｭ怜・荳�閾ｴ・・refix/contains・我ｾ晏ｭ倥□縺｣縺溘�・
  - allocator helper 隗｣豎ｺ譎ゅ↓ `alloc` 縺ｨ `alloc_raw` 縺ｮ蜿悶ｊ驕輔∴縺檎匱逕溘＠縲‘num/tuple 讒狗ｯ画凾縺ｮ蜀・Κ遒ｺ菫昴〒閾ｪ蟾ｱ蜀榊ｸｰ縺瑚ｵｷ縺阪※縺・◆縲・
- 螟画峩:
  - `nepl-core/src/codegen_wasm.rs`
    - helper 蜷阪・蝓ｺ蠎募錐謚ｽ蜃ｺ `helper_base_name` 繧定ｿｽ蜉�縲・
    - runtime helper 隗｣豎ｺ繧貞渕蠎募錐荳�閾ｴ縺ｸ螟画峩縺励�∵尠譏ｧ荳�閾ｴ繧貞ｻ・ｭ｢縲・
    - 迴ｾ蝨ｨ lowering 荳ｭ縺ｮ髢｢謨ｰ繧､繝ｳ繝・ャ繧ｯ繧ｹ縺ｯ helper 蛟呵｣懊°繧蛾勁螟悶�・
    - `LocalMap` 縺ｫ `alloc_helper_idx` 繧剃ｿ晄戟縺励�・未謨ｰ縺斐→縺ｫ荳�蠎ｦ縺�縺・helper 繧堤｢ｺ螳壹�・
  - `nepl-core/src/runtime_helpers.rs`
    - `ALLOC_CANDIDATES` 繧・`["alloc_raw", "alloc"]` 縺ｮ鬆・∈螟画峩縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i stdlib/core/option.nepl -i stdlib/alloc/collections/vec.nepl -i stdlib/alloc/collections/vec/sort.nepl --no-stdlib --no-tree -o /tmp/tests-vec-option-after-alloc-helper-fix.json -j 15` -> `22/22 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-alloc-helper-fix.json -j 15` -> `791/791 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: web 螳溯｡梧凾 `compile: unreachable` 縺ｮ譬ｹ譛ｬ菫ｮ豁｣)

- 逶ｮ逧・
  - `web/dist` 邨瑚ｷｯ縺ｧ縺ｮ縺ｿ逋ｺ逕溘＠縺ｦ縺・◆ `phase=compile, error=unreachable` 繧呈�ｹ譛ｬ蜴溷屏縺九ｉ隗｣豸医☆繧九�・
- 蜴溷屏:
  - `codegen_wasm.rs` 縺ｮ raw wasm 陦後ヱ繝ｼ繧ｹ縺ｧ縲√Ο繝ｼ繧ｫ繝ｫ隗｣豎ｺ繧ｯ繝ｭ繝ｼ繧ｸ繝｣縺・`parse_wasm_line_with_lookup` 蛛ｴ縺ｮ `$` 豁｣隕丞喧縺ｨ莠碁㍾蜃ｦ逅・↓縺ｪ縺｣縺ｦ縺・◆縲・
  - 縺昴・邨先棡縲～#wasm` 譛ｬ譁・・ `$a`/`$b` 縺・codegen 譎ゅ・縺ｿ `unknown local` 縺ｫ縺ｪ繧・panic 縺励※縺・◆・・recheck 蛛ｴ縺ｨ縺ｯ荳肴紛蜷茨ｼ峨�・
- 螟画峩:
  - `nepl-core/src/codegen_wasm.rs`
    - `parse_wasm_line` 縺ｮ lookup 繧・`|name| locals.lookup(name)` 縺ｫ邨ｱ荳�縲・
    - 譌ｧ `parse_local` 繝倥Ν繝代ｒ蜑企勁縲・
  - `nepl-web/src/lib.rs`
    - `console_error_panic_hook::set_once()` 繧・`#[wasm_bindgen(start)]` 縺ｧ譛牙柑蛹悶＠縲仝ASM panic 縺ｮ蜴溷屏菴咲ｽｮ繧貞庄隕門喧縲・
  - `nodesrc/run_test.js`
    - `formatError` 繧定ｿｽ蜉�縺励�…ompile/run 螟ｱ謨玲凾縺ｫ stack 繧剃ｿ晄戟縺励※ JSON 蜃ｺ蜉帙∈蜿肴丐縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-after-rootfix.json -j 15` -> `8/8 pass`
  - `node nodesrc/tests.js -i stdlib/alloc/collections/list.nepl --no-stdlib --no-tree -o /tmp/tests-list-after-rootfix.json -j 15` -> `11/11 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-rootfix.json -j 15` -> `707/791 pass`・域ｮ九ｊ `84 fail` 縺ｯ run 譎・`Maximum call stack size exceeded`縲Ａcompile: unreachable` 縺ｯ蜀咲樟縺帙★・・

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: web 螳溯｡梧凾 `unreachable` 縺ｮ蛻・ｊ蛻・￠)

- 逶ｮ逧・
  - 蜈ｨ菴薙ユ繧ｹ繝・(`tests + stdlib`) 縺ｧ螟夂匱縺吶ｋ `phase=compile, error=unreachable` 繧偵�・俣縺ｫ蜷医ｏ縺帙〒縺ｯ縺ｪ縺乗�ｹ譛ｬ蜴溷屏縺九ｉ蛻・ｊ蛻・￠繧九�・
- 螳滓命:
  - `trunk build` 蠕後↓
    - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-baseline-after-revert-v1.json -j 15`
    - 邨先棡: `349/791 pass`縲～442 fail`縲∽ｸ贋ｽ榊､ｱ謨励・ `stdlib/alloc/collections/list.nepl` doctest 鄒､縺ｮ `unreachable`縲・
  - 蜷後§蜈･蜉帙ｒ `nepl-cli` 縺ｧ蜊倅ｽ薙さ繝ｳ繝代う繝ｫ:
    - `target/debug/nepl-cli -i /tmp/list_doctest1_clean.nepl --target std --emit wasm -o /tmp/list_doctest1_out -v`
    - 邨先棡: compile 謌仙粥 (`DEBUG: compile_module returned Ok`)縲・
- 邨占ｫ・
  - 螟ｱ謨励・ `web/dist`・・ASM 荳翫・ compiler 螳溯｡鯉ｼ臥ｵ瑚ｷｯ縺ｫ髯仙ｮ壹＆繧後ｋ縲・
  - `codegen_wasm` 縺ｮ莉雁屓蟾ｮ蛻・ｒ謌ｻ縺励※繧ょ・迴ｾ縺吶ｋ縺溘ａ縲∝腰邏斐↑ backend 螟画峩襍ｷ蝗�縺ｧ縺ｯ縺ｪ縺・�・
  - 莉･髯阪・ `web` 蛛ｴ縺ｧ panic 繧定ｨｺ譁ｭ蛹悶＠縺ｦ蜴溷屏菴咲ｽｮ繧貞庄隕門喧縺吶ｋ繧ｿ繧ｹ繧ｯ繧剃ｸ頑ｵ∬ｪｲ鬘後→縺励※謇ｱ縺・�・

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: todo謨ｴ逅・+ llvm precheck 霑斐ｊ蛟､隕冗ｴ・

- 逶ｮ逧・
  - `todo.md` 縺ｮ螳御ｺ・ｸ医∩鬆・岼・・UnsupportedHirLowering` 謨ｴ逅・ｼ峨ｒ蜿肴丐縺励�∵悴螳御ｺ・□縺代ｒ谿九☆縲・
  - LLVM 蜑肴ｮｵ讀懈渊縺ｫ縲碁撼 unit 髢｢謨ｰ縺ｯ蛟､繧定ｿ斐☆縲崎ｦ冗ｴ・ｒ霑ｽ蜉�縺励※縲｜ackend 萓晏ｭ伜､ｱ謨励・蜑肴ｮｵ蛹悶ｒ騾ｲ繧√ｋ縲・
- 螟画峩:
  - `todo.md`
    - 繝輔ぉ繝ｼ繧ｺD縺ｮ螳御ｺ・ｸ医∩陦・
      - `llvm 邨瑚ｷｯ縺ｧ繧・backend 萓晏ｭ倥お繝ｩ繝ｼ繧貞燕谿ｵ險ｺ譁ｭ縺ｫ蟇・○繧具ｼ・nsupportedHirLowering 縺ｮ謨ｴ逅・ｼ荏
      繧貞炎髯､縺励�∵ｮ玖ｪｲ鬘後→縺励※
      - `llvm 邨瑚ｷｯ縺ｮ precheck 繧呈僑蠑ｵ縺励�（ntrinsic/謌ｻ繧雁�､隕冗ｴ・↑縺ｩ backend 萓晏ｭ伜､ｱ謨励ｒ蜑肴ｮｵ縺ｧ遒ｺ螳壹☆繧九�Ａ
      縺ｸ譖ｴ譁ｰ縲・
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_llvm_codegen` 縺ｫ `TypeCtx` 繧呈ｸ｡縺吝ｽ｢縺ｸ螟画峩縲・
    - reachable 縺ｪ `HirBody::Block` 髢｢謨ｰ縺ｫ縺､縺・※縲∵綾繧雁�､蝙九′髱・`unit` 縺九▽ block 縺悟�､繧定ｿ斐＆縺ｪ縺・�ｴ蜷医ｒ `D3003` 縺ｧ險ｺ譁ｭ縲・
  - `nepl-core/src/codegen_llvm.rs`
    - `precheck_llvm_codegen(&types, &hir, &reachable_set)` 蜻ｼ縺ｳ蜃ｺ縺励∈譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v9.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm codegen_precheck 縺ｫ螳滓､懈渊繧定ｿｽ蜉�)

- 逶ｮ逧・
  - `codegen` 蛻ｰ驕泌ｾ後・逕滓・蟆ゆｻｻ縺ｫ蟇・○繧九◆繧√�´LVM 蛛ｴ縺ｧ繧ょ燕谿ｵ讀懈渊縺ｧ蠑ｾ縺代ｋ蜈･蜉帙ｒ蠅励ｄ縺吶�・
- 螟画峩:
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_llvm_codegen` 繧定ｿｽ蜉�縲・
    - 蛻ｰ驕秘未謨ｰ・・eachable set・峨↓蟇ｾ縺励※ expression tree 繧定ｵｰ譟ｻ縺励�´LVM 譛ｪ蟇ｾ蠢・intrinsic 繧貞燕谿ｵ險ｺ譁ｭ蛹悶�・
    - 譛ｪ蟇ｾ蠢・intrinsic 縺ｯ `D3012 (TypeUnknownIntrinsic)` 縺ｧ蝣ｱ蜻翫�・
  - `nepl-core/src/codegen_llvm.rs`
    - HIR lower 蜑阪↓ `precheck_llvm_codegen` 繧貞ｮ溯｡後＠縲‘rror 縺後≠繧後・ `TypecheckFailed` 縺ｧ譌ｩ譛溽ｵゆｺ・�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v8.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm backend 險ｺ譁ｭ蝙九・謨ｴ逅・

- 逶ｮ逧・
  - `codegen_llvm` 縺九ｉ `UnsupportedHirLowering` 霑泌唆邨瑚ｷｯ縺梧ｶ医∴縺溽憾諷九ｒ蝙句ｮ夂ｾｩ縺ｫ繧ょ渚譏�縺吶ｋ縲・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - `LlvmCodegenError::UnsupportedHirLowering` 繧・enum / Display 縺九ｉ蜑企勁縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v6.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm 谿句ｭ・backend 險ｺ譁ｭ縺ｮ荳榊､画擅莉ｶ蛹・邯咏ｶ・

- 逶ｮ逧・
  - `codegen_llvm` 縺ｫ谿九▲縺ｦ縺・◆ `UnsupportedHirLowering` 繧貞炎貂帙＠縲∝燕谿ｵ騾夐℃蠕後・逕滓・蟆ゆｻｻ繝｢繝・Ν縺ｸ蟇・○繧九�・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - 莉･荳九ｒ `UnsupportedHirLowering` 霑泌唆縺九ｉ internal panic 縺ｸ螟画峩:
      - 髢｢謨ｰ return 蝙倶ｸ堺ｸ�閾ｴ
      - enum/struct/tuple 讒狗ｯ画凾縺ｮ `alloc` 蠢・�亥愛螳・
      - enum payload / struct field / tuple item 縺ｮ蛟､逕滓・蠢・�医・蝙倶ｸ堺ｸ�閾ｴ
      - `match` arm 縺ｮ邨先棡蝙倶ｸ堺ｸ�閾ｴ
      - unknown intrinsic 蛻ｰ驕・
      - unsupported expression kind 蛻ｰ驕・
      - 譁・ｭ怜・繝ｪ繝・Λ繝ｫID遽・峇螟・
      - 譁・ｭ怜・蜈ｷ菴灘喧譎ゅ・ `alloc` 蠢・�亥愛螳・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v5.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm intrinsic 蠑墓焚繝ｻ蝙九メ繧ｧ繝・け縺ｮ backend 險ｺ譁ｭ繧剃ｸ榊､画擅莉ｶ蛹・

- 逶ｮ逧・
  - `codegen_llvm` intrinsic lowering 縺ｫ谿九▲縺ｦ縺・◆ backend 險ｺ譁ｭ繧貞炎貂帙＠縲∝燕谿ｵ騾夐℃蠕後・逕滓・蟆ゆｻｻ繝｢繝・Ν縺ｸ蟇・○繧九�・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - 莉･荳九ｒ `UnsupportedHirLowering` 霑泌唆縺九ｉ internal panic 縺ｸ螟画峩:
      - `load` 縺ｮ蠑墓焚蛟区焚/蝙句ｼ墓焚蛟区焚荳堺ｸ�閾ｴ縲√・繧､繝ｳ繧ｿ蛟､荳榊惠縲√・繧､繝ｳ繧ｿ蝙倶ｸ堺ｸ�閾ｴ
      - `store` 縺ｮ蠑墓焚蛟区焚/蝙句ｼ墓焚蛟区焚荳堺ｸ�閾ｴ縲√・繧､繝ｳ繧ｿ/蛟､荳榊惠縲√・繧､繝ｳ繧ｿ蝙倶ｸ堺ｸ�閾ｴ縲～u8` 蛟､蝙倶ｸ堺ｸ�閾ｴ縲∵�ｼ邏榊梛荳堺ｸ�閾ｴ
      - `add` 縺ｮ蠑墓焚蛟区焚荳堺ｸ�閾ｴ縲〕hs/rhs 荳榊惠縲（32莉･螟・
      - `f32_to_i32` / `i32_to_u8` / `u8_to_i32` 縺ｮ蠑墓焚蛟区焚繝ｻ蛟､荳榊惠繝ｻ蝙倶ｸ堺ｸ�閾ｴ
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v4.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm 蛻ｶ蠕｡讒区枚縺ｮ backend 險ｺ譁ｭ繧剃ｸ榊､画擅莉ｶ蛹・

- 逶ｮ逧・
  - `codegen_llvm` 縺ｮ `if/while/match` 縺ｧ谿九▲縺ｦ縺・◆ backend 險ｺ譁ｭ繧貞炎貂帙＠縲∝梛讀懈渊繝ｻ蜑肴ｮｵ讀懆ｨｼ騾夐℃蠕後・逕滓・蟆ゆｻｻ縺ｸ蟇・○繧九�・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - `if`:
      - 譚｡莉ｶ縺悟�､繧定ｿ斐＆縺ｪ縺・
      - 譚｡莉ｶ縺・`i32/bool` 莠呈鋤縺ｧ縺ｪ縺・
      - then/else 蛻・ｲ千ｵ先棡蝙倶ｸ堺ｸ�閾ｴ
      繧・`UnsupportedHirLowering` 霑泌唆縺九ｉ internal panic 縺ｸ螟画峩縲・
    - `while`:
      - 譚｡莉ｶ縺悟�､繧定ｿ斐＆縺ｪ縺・
      - 譚｡莉ｶ縺・`i32/bool` 莠呈鋤縺ｧ縺ｪ縺・
      繧・internal panic 縺ｸ螟画峩縲・
    - `match`:
      - scrutinee 縺悟�､繧定ｿ斐＆縺ｪ縺・
      - scrutinee 縺・enum pointer (`i32`) 縺ｧ縺ｪ縺・
      - arm 縺・莉ｶ
      繧・internal panic 縺ｸ螟画峩縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v3.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm call_indirect 縺ｮ backend 險ｺ譁ｭ繧剃ｸ榊､画擅莉ｶ蛹・

- 逶ｮ逧・
  - `codegen_llvm` 縺ｮ `call_indirect` 縺ｧ谿九▲縺ｦ縺・◆ backend 險ｺ譁ｭ・・UnsupportedHirLowering`・峨ｒ蜑頑ｸ帙＠縲∝燕谿ｵ騾夐℃蠕後・逕滓・蟆ゆｻｻ縺ｫ蟇・○繧九�・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - `call_indirect` 縺ｫ縺､縺・※莉･荳九・ `UnsupportedHirLowering` 霑泌唆繧・internal panic 蛹・
      - callee 縺悟�､繧定ｿ斐＆縺ｪ縺・
      - callee 縺・`i32` 髢｢謨ｰID縺ｧ縺ｪ縺・
      - 蠑墓焚縺悟�､繧定ｿ斐＆縺ｪ縺・
      - 蠑墓焚蛟区焚荳堺ｸ�閾ｴ
      - 蠑墓焚蝙倶ｸ堺ｸ�閾ｴ
      - 蛟呵｣憺未謨ｰ譛ｪ讀懷・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v2.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: raw wasm 陦梧､懈渊縺ｮ蜑肴ｮｵ蛻・屬繧貞ｮ御ｺ・

- 逶ｮ逧・
  - `codegen_precheck` 縺・`codegen_wasm` 螳溯｣・ｩｳ邏ｰ縺ｸ萓晏ｭ倥☆繧狗ｵ瑚ｷｯ繧定ｧ｣豸医＠縲∝燕谿ｵ讀懈渊縺ｮ雋ｬ蜍吶ｒ `wasm_shared` 縺ｸ髮・ｴ・☆繧九�・
  - 縲慶odegen 蛻ｰ驕疲凾縺ｯ逕滓・蟆ゆｻｻ縲阪・譁ｹ驥昴ｒ邯ｭ謖√＠縲〉aw wasm 陦後ヱ繝ｼ繧ｹ螟ｱ謨励ｒ蜑肴ｮｵ縺ｧ遒ｺ螳壹☆繧九�・
- 螟画峩:
  - `nepl-core/src/wasm_shared.rs`
    - `parse_wasm_line_with_lookup` 繧貞・譛牙喧縲・
    - `precheck_raw_wasm_body` 繧定ｿｽ蜉�縺励�～HirBody::Wasm` 陦後ｒ蜑肴ｮｵ縺ｧ讀懈渊縺励※ `D4004` 繧定ｿ斐☆繧医≧縺ｫ螟画峩縲・
  - `nepl-core/src/passes/codegen_precheck.rs`
    - raw wasm 莠句燕讀懈渊蜻ｼ縺ｳ蜃ｺ縺怜・繧・`codegen_wasm` 縺九ｉ `wasm_shared` 縺ｸ螟画峩縲・
  - `todo.md`
    - 繝輔ぉ繝ｼ繧ｺD縺ｮ縲形codegen_precheck` 縺ｮ wasm 蛛ｴ繝倥Ν繝台ｾ晏ｭ俶紛逅・�埼�・岼繧貞ｮ御ｺ・→縺励※蜑企勁縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v1.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: #wasm 縺ｮ繧ｹ繧ｿ繝・け讀懆ｨｼ繧貞燕谿ｵ讀懈渊縺ｸ遘ｻ蜍・

- 逶ｮ逧・
  - 縲慶odegen 縺ｯ豁｣縺励＞蜈･蜉帙ｒ逕滓・縺吶ｋ縺�縺代�阪・譁ｹ驥昴↓蜷医ｏ縺帙�～#wasm` 繝懊ョ繧｣讀懆ｨｼ繧・backend 螳溯｡梧凾縺ｧ縺ｯ縺ｪ縺・`codegen_precheck` 蛛ｴ縺ｧ螳御ｺ・＆縺帙ｋ縲・
- 螟画峩:
  - `nepl-core/src/codegen_wasm.rs`
    - `precheck_raw_wasm_body` 繧ｷ繧ｰ繝阪メ繝｣繧・`precheck_raw_wasm_body(ctx, func)` 縺ｫ螟画峩縲・
    - raw 陦後・繝代・繧ｹ謌仙粥譎ゅ↓蜻ｽ莉､蛻励ｒ闢・ｩ阪＠縲∝燕谿ｵ縺ｧ `validate_wasm_stack` 繧貞ｮ溯｡後☆繧九ｈ縺・､画峩縲・
    - `lower_user` 縺ｮ `HirBody::Wasm` 邨瑚ｷｯ縺九ｉ `validate_wasm_stack` 繧貞炎髯､縲・
    - `generate_wasm` 縺ｮ險ｺ譁ｭ髮・ｴ・ｒ螳溯ｳｪ遨ｺ縺ｫ謨ｴ逅・ｼ・odegen 蜀・ｨｺ譁ｭ繧堤匱逕溘＆縺帙↑縺・婿蜷代↓邨ｱ荳�・峨�・
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_raw_wasm_body` 蜻ｼ縺ｳ蜃ｺ縺励ｒ譁ｰ繧ｷ繧ｰ繝阪メ繝｣縺ｸ譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v4.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: codegen_precheck 縺ｮ wasm 莠句燕讀懈渊繧貞・騾壹Δ繧ｸ繝･繝ｼ繝ｫ縺ｸ蛻・屬)

- 逶ｮ逧・
  - `passes/codegen_precheck.rs` 縺・`codegen_wasm.rs` 螳溯｣・ｩｳ邏ｰ縺ｸ逶ｴ謗･萓晏ｭ倥＠縺ｦ縺・◆迥ｶ諷九ｒ謨ｴ逅・＠縲∝燕谿ｵ讀懈渊繝ｭ繧ｸ繝・け繧貞・譛峨Δ繧ｸ繝･繝ｼ繝ｫ縺ｸ蛻・屬縺吶ｋ縲・
  - 縲慶odegen 縺ｯ豁｣縺励＞蜈･蜉帙ｒ逕滓・縺吶ｋ縺�縺代�阪・譁ｹ驥昴↓蜷医ｏ縺帙�｜ackend 縺ｮ `skip`/險ｺ譁ｭ闢・ｩ阪ｒ荳榊､画擅莉ｶ驕募渚縺ｸ蟇・○繧九�・
- 螟画峩:
  - `nepl-core/src/wasm_shared.rs` 繧呈眠隕剰ｿｽ蜉�縲・
    - wasm鄂ｲ蜷崎ｧ｣豎ｺ (`wasm_sig`, `wasm_sig_ids`)
    - generic skip 蛻､螳・(`should_skip_wasm_codegen_for_generic`)
    - 蛻ｰ驕秘未謨ｰ隗｣譫・(`collect_reachable_wasm_functions`)
    - 髢捺磁蜻ｼ縺ｳ蜃ｺ縺励ｒ蜷ｫ繧�鄂ｲ蜷埼寔蜷亥庶髮・(`collect_wasm_signature_set`)
    - wasm intrinsic 蟇ｾ蠢懷愛螳・(`is_supported_wasm_intrinsic`)
  - `nepl-core/src/passes/codegen_precheck.rs`
    - 荳願ｨ倥Ο繧ｸ繝・け繧・`wasm_shared` 蜿ら・縺ｸ鄂ｮ謠帙�・
    - `precheck_raw_wasm_body` 縺ｮ縺ｿ `codegen_wasm` 蛛ｴ繧堤ｶ咏ｶ壼茜逕ｨ・域ｬ｡谿ｵ縺ｧ蛻・屬莠亥ｮ夲ｼ峨�・
  - `nepl-core/src/codegen_wasm.rs`
    - extern/function 鄂ｲ蜷堺ｸ堺ｸ�閾ｴ譎ゅ・ `skip` 繧貞ｻ・ｭ｢縺・internal panic 蛹悶�・
    - `lower_body` 縺ｧ backend 險ｺ譁ｭ縺瑚ｿ斐ｋ邨瑚ｷｯ繧・internal panic 蛹悶�・
    - 蜈ｱ譛峨Ο繧ｸ繝・け縺ｯ `wasm_shared` 蜻ｼ縺ｳ蜃ｺ縺励∈蟋碑ｭｲ縲・
  - `nepl-core/src/lib.rs`
    - `pub mod wasm_shared;` 繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v3.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm backend 險ｺ譁ｭ繧貞燕谿ｵ荳榊､画擅莉ｶ縺ｸ遘ｻ陦・

- 逶ｮ逧・
  - `todo.md` 繝輔ぉ繝ｼ繧ｺD譁ｹ驥昴↓蜷医ｏ縺帙�～codegen_llvm` 蛛ｴ縺ｧ逋ｺ陦後＠縺ｦ縺・◆縲悟燕谿ｵ騾夐℃蠕後↓蛻ｰ驕斐＠縺ｪ縺・・縺壹�阪・險ｺ譁ｭ繧貞ｻ・ｭ｢縺励�∝燕谿ｵ讀懆ｨｼ縺ｮ荳榊､画擅莉ｶ縺ｨ縺励※謇ｱ縺・�・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - `let` 縺ｮ蝙倶ｸ堺ｸ�閾ｴ (`let type mismatch`) 繧・`UnsupportedHirLowering` 縺九ｉ internal panic 縺ｸ螟画峩縲・
    - `set` 縺ｮ蝙倶ｸ堺ｸ�閾ｴ (`set type mismatch`) 繧・`UnsupportedHirLowering` 縺九ｉ internal panic 縺ｸ螟画峩縲・
    - 譛ｪ隗｣豎ｺ trait call 縺ｮ蛻ｰ驕斐ｒ `UnsupportedHirLowering` 縺九ｉ internal panic 縺ｸ螟画峩縲・
    - call 蠑墓焚蝙倶ｸ堺ｸ�閾ｴ繧・`UnsupportedHirLowering` 縺九ｉ internal panic 縺ｸ螟画峩縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-llvm-invariant-v2.json -j 15` -> `8/8 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-invariant-panic-v1.json -j 15` -> `707/791 pass`・・Maximum call stack size exceeded` 縺悟､壽焚縲ゆｻ雁屓縺ｮ螟画峩蟇ｾ雎｡螟悶・譌｢蟄伜､ｱ謨励→縺励※邯咏ｶ夊ｪｿ譟ｻ・・

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC/D謗･邯・ core/mem 縺ｫ MemPtr 蛻晄悄蛹悶が繝ｼ繝舌・繝ｭ繝ｼ繝芽ｿｽ蜉�)

- 逶ｮ逧・
  - `core/mem` 蠕梧ｮｵ遘ｻ陦鯉ｼ・stdlib/std`/tutorials・峨〒 `i32` 逕溘・繧､繝ｳ繧ｿ繧帝愆蜃ｺ縺帙★縺ｫ驟榊・蛻晄悄蛹悶〒縺阪ｋ荳頑ｵ、PI繧堤畑諢上☆繧九�・
  - `MemPtr` 繝｢繝・Ν荳翫〒 `fill/memset` 繧堤ｵｱ荳�縺励�～Result` 縺ｧ螟ｱ謨励ｒ謇ｱ縺医ｋ繧医≧縺ｫ縺吶ｋ縲・
- 螟画峩:
  - `stdlib/core/mem.nepl`
    - `memset_u8 <(MemPtr<u8>,i32,i32)->Result<(),str>>` 繧定ｿｽ蜉�縲・
    - `fill_u8 <(MemPtr<u8>,i32,i32)->Result<(),str>>` 繧定ｿｽ蜉�縲・
    - `fill_i32 <(MemPtr<i32>,i32,i32)->Result<(),str>>` 繧定ｿｽ蜉�縲・
    - 辟｡蜉ｹ繝昴う繝ｳ繧ｿ繧・ｲ�縺ｮ髟ｷ縺輔・ `Result::Err` 繧定ｿ斐☆縲・
  - `tests/memory_safety.n.md`
    - `MemPtr fill_i32/fill_u8 縺ｮ螳牙・繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝荏 繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
    - `MemPtr fill 邉ｻ縺ｯ辟｡蜉ｹ蠑墓焚繧・Err 縺ｧ霑斐☆` 繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety-fill-overload.json -j 15` -> `217/217 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-mem-fill-overload.json -j 15` -> `787/787 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: kpread_core 繝倥ャ繝�繝輔ぅ繝ｼ繝ｫ繝峨・蝙句ｮ牙・蛹・

- 逶ｮ逧・
  - `kpread_core` 縺ｫ谿九▲縺ｦ縺・◆繝倥ャ繝�逕溘が繝輔そ繝・ヨ・・0/4/8`・峨ｒ蛻玲嫌蝙九∈遘ｻ陦後＠縲～kpread`/`kpwrite` 縺ｨ蜷後§蠅・阜陦ｨ迴ｾ縺ｫ謠・∴繧九�・
  - 繝倥ャ繝�繝ｬ繧､繧｢繧ｦ繝医・諢丞袖繧貞梛縺ｧ蝗ｺ螳壹＠縲√が繝輔そ繝・ヨ隱､謖・ｮ壹ｒ荳頑ｵ√〒髦ｲ縺舌�・
- 螟画峩:
  - `stdlib/kp/kpread_core.nepl`
    - `ScannerHeaderFieldCore` 繧定ｿｽ蜉�・・BufPtr` / `Len` / `Pos`・峨�・
    - `scanner_header_core_off` 繧定ｿｽ蜉�縺励�√が繝輔そ繝・ヨ隗｣豎ｺ繧・邂・園縺ｫ髮・ｴ・�・
    - `store_i32_u8_at sc*_region 0/4/8 ...` 繧貞・謖吝梛 + 繧ｪ繝輔そ繝・ヨ髢｢謨ｰ邨檎罰縺ｸ鄂ｮ謠帙�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-kp-core-header-field-enum.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpread-core-header-field-enum.json -j 15` -> `785/785 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: kpwrite 繝倥ャ繝�繝輔ぅ繝ｼ繝ｫ繝峨・蝙句ｮ牙・蛹・

- 逶ｮ逧・
  - `kpwrite` 縺ｮ繝倥ャ繝�繧｢繧ｯ繧ｻ繧ｹ縺ｧ菴ｿ縺｣縺ｦ縺・◆逕溘が繝輔そ繝・ヨ蛟､・・0/4/8/12/16`・峨ｒ蛻玲嫌蝙九↓鄂ｮ謠帙＠縲～kpread` 縺ｨ蜷後§螳牙・繝｢繝・Ν縺ｸ邨ｱ荳�縺吶ｋ縲・
  - `mem/kpread/kpwrite` 縺ｮ蜈ｬ髢帰PI螳牙・蛹悶〒縲√・繝・ム蠅・阜縺ｮ諢丞袖繧貞梛縺ｧ陦ｨ迴ｾ縺吶ｋ縲・
- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `WriterHeaderField` 繧定ｿｽ蜉�・・BufPtr` / `Cap` / `WriteLen` / `IovPtr` / `NwPtr`・峨�・
    - `writer_header_off` 繧定ｿｽ蜉�縺励�√が繝輔そ繝・ヨ隗｣豎ｺ繧剃ｸ�邂・園縺ｫ髮・ｴ・�・
    - `writer_header_ptr` / `writer_load_header` / `writer_store_header` / `writer_load_header_ptr` 縺ｮ隨ｬ2蠑墓焚繧・`i32` 縺九ｉ `WriterHeaderField` 縺ｫ螟画峩縲・
    - 蜻ｼ縺ｳ蜃ｺ縺怜・縺ｮ逕滓焚蛟､繧ｪ繝輔そ繝・ヨ繧貞・蟒・＠縲∝・謖吝�､縺ｫ鄂ｮ謠帙�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl --no-tree -o /tmp/tests-kp-header-field-enum-unified.json -j 15` -> `226/226 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpwrite-header-field-enum.json -j 15` -> `785/785 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: kpread 繝倥ャ繝�繝輔ぅ繝ｼ繝ｫ繝峨・蝙句ｮ牙・蛹・

- 逶ｮ逧・
  - `kpread` 縺ｮ繝倥ャ繝�繧｢繧ｯ繧ｻ繧ｹ縺ｧ菴ｿ縺｣縺ｦ縺・◆逕溘が繝輔そ繝・ヨ蛟､・・0/4/8`・峨ｒ蛻玲嫌蝙九∈鄂ｮ縺肴鋤縺医�∝他縺ｳ蜃ｺ縺怜・縺ｮ隱､謖・ｮ壹ｒ貂帙ｉ縺吶�・
  - `todo.md` 2026-03-03 繝輔ぉ繝ｼ繧ｺD・・mem/kpread/kpwrite` 縺ｮ蜈ｬ髢帰PI螳牙・蛹厄ｼ峨↓豐ｿ縺｣縺ｦ縲∽ｸ頑ｵ√・陦ｨ迴ｾ繧貞崋螳壹☆繧九�・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - `ScannerHeaderField` 繧定ｿｽ蜉�・・BufPtr` / `Len` / `Pos`・峨�・
    - `scanner_header_off` 繧定ｿｽ蜉�縺励�√が繝輔そ繝・ヨ隗｣豎ｺ繧・邂・園縺ｸ髮・ｴ・�・
    - `scanner_header_ptr` / `scanner_load_header` / `scanner_store_header` 縺ｮ隨ｬ2蠑墓焚繧・`i32` 縺九ｉ `ScannerHeaderField` 縺ｫ螟画峩縲・
    - 蜻ｼ縺ｳ蜃ｺ縺怜・縺ｮ `scanner_load_header sc 0/4/8` 縺ｨ `scanner_store_header sc 8 ...` 繧貞・謖吝梛謖・ｮ壹∈鄂ｮ謠帙�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpread.nepl --no-tree -o /tmp/tests-kpread-header-field-targeted.json -j 15` -> `222/222 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpread-header-field.json -j 15` -> `785/785 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: kpread 繝倥ャ繝�繧｢繧ｯ繧ｻ繧ｹ縺ｮ繧ｵ繧､繝ｬ繝ｳ繝亥､ｱ謨励ｒ髯､蜴ｻ)

- 逶ｮ逧・
  - `scanner_load_header` / `scanner_store_header` 縺ｮ螟ｱ謨玲凾繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ・・0` / `()`・峨ｒ蟒・ｭ｢縺励�√・繝・ム荳肴紛蜷医ｒ髫�阡ｽ縺励↑縺・�・
  - 荳頑ｵ∽ｻ墓ｧ假ｼ亥ｮ牙・API蜆ｪ蜈茨ｼ峨↓蜷医ｏ縺帙�∝｣翫ｌ縺溽憾諷九ｒ邯咏ｶ壹＆縺帙ｋ繧医ｊ蜊ｳ譎ょ●豁｢縺ｫ邨ｱ荳�縺吶ｋ縲・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - `scanner_load_header`:
      - `scanner_header_ptr` 縺・`Err` 縺ｮ蝣ｴ蜷医・ `0` 霑泌唆繧・`#intrinsic "unreachable"` 縺ｸ螟画峩縲・
      - `load_i32` 縺・`None` 縺ｮ蝣ｴ蜷医・ `0` 霑泌唆繧・`#intrinsic "unreachable"` 縺ｸ螟画峩縲・
    - `scanner_store_header`:
      - `scanner_header_ptr` 縺・`Err` 縺ｮ蝣ｴ蜷医・辟｡隕悶ｒ `#intrinsic "unreachable"` 縺ｸ螟画峩縲・
      - `store_i32` 縺・`Err` 縺ｮ蝣ｴ蜷医・辟｡隕悶ｒ `#intrinsic "unreachable"` 縺ｸ螟画峩縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpread.nepl --no-tree -o /tmp/tests-kpread-header-unreachable-targeted.json -j 15` -> `222/222 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpread-unreachable.json -j 15` -> `785/785 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD蜈郁｡・ Writer 繧・RegionToken 菫晄戟縺ｸ遘ｻ陦・

- 逶ｮ逧・
  - `kpread` 縺ｨ蜷梧ｧ倥↓ `kpwrite` 縺ｧ繧ょ・髢九ワ繝ｳ繝峨Ν縺碁�伜沺諠・�ｱ繧呈戟縺､繧医≧縺ｫ縺励�√Γ繝｢繝ｪ螳牙・API繧堤ｵｱ荳�縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - `Writer` 縺ｯ `MemPtr<u8>` 繧堤峩謗･菫晄戟縺励�√・繝・ム鬆伜沺繧ｵ繧､繧ｺ・・0byte・峨′蝙九↓陦ｨ迴ｾ縺輔ｌ縺ｦ縺・↑縺九▲縺溘�・
  - 騾比ｸｭ縺ｧ霑ｽ蜉�縺励◆ `writer_mem(Writer)->MemPtr<u8>` 繝倥Ν繝代・ `Writer` 繧貞�､貂｡縺励〒蜿励￠繧九◆繧√�・
    non-copy 縺ｪ `Writer` 縺ｮ move 繧堤匱逕溘＆縺・`D3053` 繧貞ｼ輔″襍ｷ縺薙＠縺溘�・
- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `Writer.raw` 繧・`Writer.region: RegionToken<u8>` 縺ｫ螟画峩縲・
    - `writer_wrap` 縺ｧ `region_new raw 20` 繧呈ｧ狗ｯ峨�・
    - `writer_mem` 繝倥Ν繝代・蜑企勁縺励�～region_ptr get w "region"` 繧堤峩謗･螻暮幕縺励※ move 繧貞屓驕ｿ縲・
  - `stdlib/kp/kpread_core.nepl`
    - `store_i32_u8_at/load_i32_u8_at` 繧・`RegionToken<u8>` 蜿励￠蜿悶ｊ縺ｸ螟画峩縲・
    - `sc0/iov/nread/sc` 縺ｮ蜷・�伜沺繧・`RegionToken` 蛹悶＠縺ｦ繧｢繧ｯ繧ｻ繧ｹ邨瑚ｷｯ繧堤ｵｱ荳�縲・
    - 騾比ｸｭ縺ｧ逋ｺ逕溘＠縺・`match` 繧｢繝ｼ繝�蟠ｩ繧鯉ｼ・D3009/D3008/D3045`・峨ｒ菫ｮ豁｣縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-writer-regiontoken-v3.json -j 15`
  - 邨先棡: `221/221 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD蜈郁｡・ kpread_core 縺ｮ蜀・Κ繝倥ャ繝�繧｢繧ｯ繧ｻ繧ｹ繧・RegionToken 蛹・

- 逶ｮ逧・
  - `kpread_core` 縺ｮ蜀・Κ繝｡繝｢繝ｪ繧｢繧ｯ繧ｻ繧ｹ繧・`RegionToken` 邨檎罰縺ｫ邨ｱ荳�縺励�～MemPtr + off` 縺ｮ逶ｴ謗･邂苓｡謎ｾ晏ｭ倥ｒ貂帙ｉ縺吶�・
- 譬ｹ譛ｬ蜴溷屏:
  - `store_i32_u8_at` / `load_i32_u8_at` 縺・`MemPtr<u8>` 縺ｨ `off` 縺九ｉ逶ｴ謗･ `MemPtr<i32>` 繧剃ｽ懊ｋ險ｭ險医〒縲・
    鬆伜沺蠅・阜縺ｮ蜑肴署縺後・繝ｫ繝大､悶∈貍上ｌ縺ｦ縺・◆縲・
- 螟画峩:
  - `stdlib/kp/kpread_core.nepl`
    - `mem_i32_region_ptr` 繧定ｿｽ蜉�縺励�～region_ptr_at<u8,i32>` 繧剃ｽｿ逕ｨ縲・
    - `store_i32_u8_at` / `load_i32_u8_at` 縺ｮ蠑墓焚繧・`RegionToken<u8>` 縺ｫ螟画峩縲・
    - `sc0(12)`, `iov(8)`, `nread(4)`, `sc(12)` 縺ｧ `RegionToken` 繧呈ｧ狗ｯ峨＠縺ｦ繝倥Ν繝代∈貂｡縺吝ｽ｢縺ｫ譖ｴ譁ｰ縲・
  - 騾比ｸｭ菫ｮ豁｣:
    - `match dealloc_ptr<u8> buf cap` 縺ｮ `Result::Err` 繧｢繝ｼ繝�縺ｮ繧､繝ｳ繝・Φ繝亥ｴｩ繧後↓繧医ｊ
      `D3009/D3008/D3045` 縺檎匱逕溘＠縺溘◆繧√�∝・蟯先ｧ矩��繧呈ｭ｣縺励￥菫ｮ豁｣縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-kpread-core-regiontoken-v2.json -j 15`
  - 邨先棡: `221/221 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD蜈郁｡・ kpwrite 繝倥ャ繝�繧｢繧ｯ繧ｻ繧ｹ繧・RegionToken 邨檎罰縺ｸ遘ｻ陦・

- 逶ｮ逧・
  - `kpwrite` 蛛ｴ縺ｧ繧ゅ・繝・ム繧｢繧ｯ繧ｻ繧ｹ繧・`RegionToken` 繝吶・繧ｹ縺ｫ蟇・○縲～core/mem` 縺ｮ蠅・阜讀懆ｨｼAPI繧貞・蛻ｩ逕ｨ縺ｧ縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - 譌｢蟄・`writer_header_ptr` 縺ｯ `mem_ptr_addr + off` 縺ｧ逶ｴ謗･繧｢繝峨Ξ繧ｹ邂苓｡薙ｒ陦後＞縲・
    20byte 繝倥ャ繝�蠅・阜縺ｮ蜑肴署繧帝未謨ｰ縺斐→縺ｫ證鈴ｻ吝喧縺励※縺・◆縲・
- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_header_region` 繧定ｿｽ蜉�・・region_new w_mem 20`・峨�・
    - `writer_header_ptr` 繧・`Result<MemPtr<i32>,str>` 縺ｸ螟画峩縺励�～region_ptr_at<u8,i32>` 繧剃ｽｿ逕ｨ縲・
    - `writer_load_header` / `writer_store_header` 繧剃ｸ願ｨ・`Result` 邨瑚ｷｯ縺ｸ譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-writer-header-regiontoken.json -j 15`
  - 邨先棡: `221/221 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD蜈郁｡・ kpread 縺ｮ Scanner 繝倥ャ繝�繧・RegionToken 蛹・

- 逶ｮ逧・
  - `todo.md` 繝輔ぉ繝ｼ繧ｺD逹�謇九→縺励※縲～kpread` 縺ｮ蜈ｬ髢九ワ繝ｳ繝峨Ν縺ｫ鬆伜沺謇�譛画ュ蝣ｱ繧呈戟縺溘○縲～core/mem` 縺ｮ譁ｰ螳牙・API縺ｸ蟇・○繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - `Scanner` 縺・`MemPtr<u8>` 逶ｴ謗･菫晄戟縺ｮ縺ｿ縺ｧ縲√・繝・ム鬆伜沺蠅・阜縺ｮ諠・�ｱ縺悟梛縺ｫ荵励▲縺ｦ縺・↑縺九▲縺溘�・
  - 繝倥ャ繝�繧｢繧ｯ繧ｻ繧ｹ縺・`mem_ptr_addr + off` 縺ｮ邂苓｡謎ｾ晏ｭ倥〒縲∝｢・阜讀懆ｨｼ繧貞・蛻ｩ逕ｨ縺励↓縺上°縺｣縺溘�・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - `Scanner` 繝輔ぅ繝ｼ繝ｫ繝峨ｒ `raw: MemPtr<u8>` 縺九ｉ `region: RegionToken<u8>` 縺ｫ螟画峩縲・
    - `scanner_wrap` 縺ｧ `region_new raw 12` 繧呈ｧ狗ｯ峨�・
    - `scanner_header_ptr` 繧・`region_ptr_at<u8,i32>` 繝吶・繧ｹ縺ｮ `Result` 霑泌唆縺ｸ螟画峩縲・
    - `scanner_load_header` / `scanner_store_header` 繧剃ｸ願ｨ・`Result` 邨瑚ｷｯ縺ｧ蜃ｦ逅・�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-scanner-regiontoken.json -j 15`
  - 邨先棡: `221/221 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: core/mem 縺ｫ RegionToken 螳牙・API繧定ｿｽ蜉�)

- 逶ｮ逧・
  - `todo.md` 繝輔ぉ繝ｼ繧ｺC縺ｫ豐ｿ縺｣縺ｦ縲～MemPtr<T>` 縺ｨ `RegionToken<T>` 繧剃ｽｿ縺・ｮ牙・API繧・`core/mem` 縺ｫ霑ｽ蜉�縺励�～kpread/kpwrite` 遘ｻ陦後・荳頑ｵ∝渕逶､繧剃ｽ懊ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - 譌｢蟄・`mem` 縺ｯ `MemPtr<T>` 縺ｾ縺ｧ縺ｯ謨ｴ蛯呎ｸ医∩縺�縺｣縺溘′縲・�伜沺謇�譛峨ｒ陦ｨ縺吝・髢帰PI縺御ｸ崎ｶｳ縺励※縺翫ｊ縲・
    蠅・阜諠・�ｱ莉倥″繧｢繧ｯ繧ｻ繧ｹ繧貞梛縺ｨ縺励※邨ｱ荳�縺ｧ縺阪※縺・↑縺九▲縺溘�・
- 螟画峩:
  - `stdlib/core/mem.nepl`
    - `RegionToken<T>` 陬懷勧API繧定ｿｽ蜉�:
      - `region_new`
      - `region_in_bounds`
      - `region_ptr_at`
      - `alloc_region_bytes`
      - `alloc_region`
      - `dealloc_region`
    - 縺薙ｌ縺ｫ繧医ｊ縲・�伜沺繧ｵ繧､繧ｺ繧剃ｼｴ縺・梛莉倥″繧ｪ繝輔そ繝・ヨ蜿門ｾ励ｒ `Result` 縺ｧ謇ｱ縺医ｋ繧医≧縺ｫ縺励◆縲・
  - `tests/memory_safety.n.md`
    - `alloc_region/region_ptr_at/dealloc_region` 縺ｮ蝓ｺ譛ｬ蜍穂ｽ懊こ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
    - 遽・峇螟悶が繝輔そ繝・ヨ縺ｧ `Result::Err` 繧定ｿ斐☆蝗槫ｸｰ繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/block_semicolon_return.n.md -i tests/plan.n.md -i tests/block_single_line.n.md --no-stdlib --no-tree -o /tmp/tests-semicolon-focus.json -j 15`
  - 邨先棡: `67/67 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md --no-tree -o /tmp/tests-memory-safety-region-token.json -j 15`
  - 邨先棡: `211/211 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i tests/kp_i64.n.md --no-tree -o /tmp/tests-memory-kp-regression.json -j 15`
  - 邨先棡: `221/221 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺB2: trait capability 縺ｮ蝙倶ｻ倥″菫晄戟縺ｸ遘ｻ陦・

- 逶ｮ逧・
  - trait capability 蛻､螳壹・譁・ｭ怜・蜀崎ｧ｣譫舌ｒ貂帙ｉ縺励�∝梛莉倥″繝・・繧ｿ縺ｧ荳�雋ｫ縺励※謇ｱ縺・�・
- 譬ｹ譛ｬ蜴溷屏:
  - 譌｢蟄伜ｮ溯｣・〒縺ｯ `TraitInfo.capabilities` 縺・`Vec<String>` 縺ｮ縺溘ａ縲・
    `TraitSemantics::detect` 縺ｧ豈主屓譁・ｭ怜・繧貞・繝代・繧ｹ縺励※縺・◆縲・
  - 縺薙・讒矩��縺ｯ capability 蛻､螳壹・雋ｬ蜍吶′蛻・淵縺励�∝ｰ・擂諡｡蠑ｵ譎ゅ↓荳肴紛蜷医ｒ逕溘∩繧・☆縺・�・
- 螟画峩:
  - `nepl-core/src/typecheck.rs`
    - `TraitInfo.capabilities` 繧・`Vec<String>` 縺九ｉ `Vec<TraitCapability>` 縺ｸ螟画峩縲・
    - trait 螳夂ｾｩ蜃ｦ逅・(`Stmt::Trait`) 縺ｧ capability 繧・蝗槭□縺代ヱ繝ｼ繧ｹ縺励�∝梛莉倥″縺ｧ菫晄戟縲・
    - 驥崎､・capability 謖・ｮ壹・蜷御ｸ�trait蜀・〒驥崎､・匳骭ｲ縺励↑縺・ｈ縺・紛逅・�・
    - `TraitSemantics::detect` 縺ｯ `TraitInfo` 蜀・・蝙倶ｻ倥″ capability 繧堤峩謗･蜿ら・縲・
    - 荳崎ｦ√↓縺ｪ縺｣縺・`detect_declared_trait_capabilities` 繧貞炎髯､縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-move-overload-after-capability-typed.json -j 15`
  - 邨先棡: `272/272 pass`
  - `node nodesrc/tests.js -i tests/compile_fail_diag_location.n.md --no-tree -o /tmp/tests-compile-fail-diag-location-after-capability-typed.json -j 15`
  - 邨先棡: `207/207 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-after-capability-typed.json -j 15`
  - 邨先棡: `783/783 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: kpwrite header 隱ｭ縺ｿ蜿悶ｊ縺ｮ Result 蛹悶→ None 繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ蟒・ｭ｢)

- 逶ｮ逧・
  - `writer_load_header` 縺ｮ `None -> 0` 繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ繧貞ｻ・ｭ｢縺励�”eader 隱ｭ縺ｿ蜿悶ｊ螟ｱ謨励ｒ譏守､ｺ蛻・ｲ舌〒謇ｱ縺・�・
- 譬ｹ譛ｬ蜴溷屏:
  - 蠕捺擂縺ｮ `writer_load_header` 縺ｯ `load_i32` 螟ｱ謨玲凾縺ｫ 0 繧定ｿ斐＠縺ｦ縺翫ｊ縲∫焚蟶ｸ迥ｶ諷九ｒ豁｣蟶ｸ蛟､縺ｸ貎ｰ縺励※縺・◆縲・
  - 縺昴・縺溘ａ蠕檎ｶ壼・逅・〒 `buf/cap/iov/nw` 縺御ｸ肴ｭ｣蛟､縺ｮ縺ｾ縺ｾ騾ｲ陦後☆繧倶ｽ吝慍縺後≠縺｣縺溘�・
- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_load_header` 繧・`Result<i32,str>` 縺ｸ螟画峩縲・
    - `writer_load_header_ptr` 繧・`Result<MemPtr<u8>,str>` 縺ｸ螟画峩縲・
    - `writer_free_handle`, `writer_flush_handle`, `writer_ensure_handle`,
      `writer_put_u8_handle`, `writer_write_str_handle`,
      `writer_write_i32_handle`, `writer_write_u64_handle` 繧・
      `Result` 蛻・ｲ舌〒螳牙・縺ｫ蜃ｦ逅・☆繧句ｽ｢縺ｸ譖ｴ譁ｰ縲・
    - `if` 繝ｬ繧､繧｢繧ｦ繝井ｸｭ縺ｮ蜀鈴聞縺ｪ `then: block:` 繧帝勁蜴ｻ縺励�～D2002` 蝗樣∩縺ｮ縺溘ａ蠑乗ｧ矩��繧剃ｻ墓ｧ俶ｺ匁侠縺ｸ謨ｴ逅・�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-after-header-result-v2.json -j 15`
  - 邨先棡: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-after-header-result.json -j 15`
  - 邨先棡: `226/226 pass`
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kpwrite-style-fix.json -j 15`
  - 邨先棡: `215/215 pass`

# 2026-03-12 菴懈･ｭ繝｡繝｢ (tooling: repo_metrics 繧・TypeScript 蛹悶＠蜀・ｮｹ蛻･髮・ｨ医∈諡｡蠑ｵ)

- 逶ｮ逧・
  - `repo_metrics.py` 縺ｮ蜊倡ｴ斐↑諡｡蠑ｵ蟄宣寔險医ｒ縲√Μ繝昴ず繝医Μ螳滓・縺ｫ豐ｿ縺｣縺溘�悟・螳ｹ蛻･縲埼寔險医∈謾ｹ濶ｯ縺吶ｋ縲・
  - `.n.md` 縺ｨ騾壼ｸｸ縺ｮ `.md` 繧貞・髮｢縺励�》op-level 縺ｮ `tests/` `tutorials/` `doc/` `examples/` 縺ｨ `src/` / `stdlib/` 邉ｻ繧貞・髮｢縺励※遒ｺ隱阪〒縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
  - `source code` / `document comment` / `document` / `test` 繧貞挨髮・ｨ医＠縲～.rs` / `.nepl` / `.n.md` 縺ｮ test case 謨ｰ繧ょ・縺帙ｋ繧医≧縺ｫ縺吶ｋ縲・
- 螳溯｣・憾豕・
  - 譌｢蟄倥・ `repo_metrics.py` 縺ｯ蜑企勁縺励�》op-level 縺ｮ `repo_metrics.ts` 縺ｸ遘ｻ陦後＠縺溘�・
  - 螳溯｡後・ `node --experimental-strip-types repo_metrics.ts ...` 繧貞燕謠舌→縺励�∬ｿｽ蜉�萓晏ｭ倥↑縺励〒蜍輔￥ standalone script 縺ｫ縺励◆縲・
- 譬ｹ譛ｬ菫ｮ豁｣:
  - 莉･蜑阪・縲梧僑蠑ｵ蟄舌＃縺ｨ縺ｮ邱剰｡梧焚 + 荳�驛ｨ諡｡蠑ｵ蟄舌・ comment/code/blank縲阪□縺代〒縲～.n.md` 蜀・・譛ｬ譁・→ doctest縲～.nepl` 蜀・・ `//:` 繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝医→ doctest縲～.rs` 蜀・・ source 縺ｨ test 繧貞・髮｢縺ｧ縺阪※縺・↑縺九▲縺溘�・
  - 縺昴・縺溘ａ縲∽ｻ墓ｧ俶嶌繝ｻ繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝医・繝・せ繝医こ繝ｼ繧ｹ縺・source code 縺ｨ豺ｷ縺悶ｊ縲〉epo 縺ｮ螳滓ュ縺ｫ蜷医ｏ縺ｪ縺・焚蛟､縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
- 螟画峩:
  - `repo_metrics.ts`
    - Git 邂｡逅・ｸ九ヵ繧｡繧､繝ｫ繧貞渕貅悶↓蛻玲嫌縺励�｜inary file 縺ｯ skip 縺・size-only 髮・ｨ医ｒ驕ｸ縺ｹ繧九ｈ縺・↓縺励◆縲・
    - `By Extension` / `By Area` / `By Content Kind` 縺ｮ 3 霆ｸ縺ｧ陦ｨ遉ｺ縺吶ｋ繧医≧縺ｫ縺励◆縲・
    - area 縺ｯ `top_level_docs_tests` / `source_tree` / `other` 縺ｫ蛻・￠縺溘�・
    - `.n.md` / `.md` 縺ｧ縺ｯ `neplg2:test` 繝悶Ο繝・け縺�縺代ｒ `test`縲√◎繧御ｻ･螟悶ｒ `document` 縺ｨ縺励※謨ｰ縺医ｋ繧医≧縺ｫ縺励◆縲・
    - `.nepl` 縺ｧ縺ｯ `//:` 繧・`document comment` 縺ｨ縺励※謇ｱ縺・�～//:` 蜀・doctest 縺�縺代ｒ `test` 縺ｨ縺励※蛻・ｊ蜃ｺ縺吶ｈ縺・↓縺励◆縲・
    - `.rs` 縺ｧ縺ｯ `///` / `//!` 繧・`document comment` 縺ｨ縺励�～#[test]` 邉ｻ attribute 縺ｨ `#[cfg(test)]` 驟堺ｸ九ｒ `test` 縺ｨ縺励※謇ｱ縺・ｈ縺・↓縺励◆縲・
    - `.n.md` / `.nepl` / `.rs` 縺九ｉ test case 謨ｰ繧呈焚縺医�∵僑蠑ｵ蟄仙挨繝ｻarea 蛻･繝ｻcontent kind 蛻･縺ｫ蜿肴丐縺吶ｋ繧医≧縺ｫ縺励◆縲・
- 螳溯｡檎｢ｺ隱・
  - `node --experimental-strip-types repo_metrics.ts --json /tmp/repo_metrics.json`
    - 螳滓ｸｬ:
      - `.n.md` testCases = `812`
      - `.nepl` testCases = `278`
      - `.rs` testCases = `360`
  - 莉ｶ謨ｰ辣ｧ蜷・
    - `rg '^\\s*neplg2:test(?:\\[[^\\]]+\\])?\\s*$' -g '*.n.md' | wc -l` -> `812`
    - `rg '^\\s*//:\\s*neplg2:test(?:\\[[^\\]]+\\])?\\s*$' -g '*.nepl' | wc -l` -> `278`
    - `rg '^\\s*#\\[(test|tokio::test|wasm_bindgen_test)\\b' -g '*.rs' | wc -l` -> `360`
  - 縺薙・荳�閾ｴ縺ｫ繧医ｊ縲∝ｰ代↑縺上→繧・test case 繧ｫ繧ｦ繝ｳ繝医・ repo 螳滓・縺ｨ謨ｴ蜷医＠縺ｦ縺・ｋ縺薙→繧堤｢ｺ隱阪＠縺溘�・
- build / test:
  - `trunk build`
    - 邨先棡: success
  - `node nodesrc/run_doctest.js -i tests/compiler/typeannot.n.md -n 1`
    - 邨先棡: pass
  - `node nodesrc/run_doctest.js -i tests/compiler/typeannot.n.md -n 2`
    - 邨先棡: pass
  - 蜿り�・
    - `node nodesrc/run_doctest.js -i stdlib/core/result.nepl -n 1`
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/11_testing_workflow.n.md -n 1`
    - 荳願ｨ・2 莉ｶ縺ｯ `return value mismatch` 縺ｨ runtime trap 縺ｧ fail縲ゆｻ雁屓縺ｮ螟画峩蟇ｾ雎｡縺ｯ髮・ｨ医せ繧ｯ繝ｪ繝励ヨ縺ｧ縺ゅｊ縲〉epo_metrics 螟画峩縺ｮ譛臥┌縺ｫ髢｢菫ゅ↑縺乗里蟄倥・ doctest 蛛ｴ蝠城｡後→縺励※谿九▲縺ｦ縺・ｋ縲・
- 蟾ｮ逡ｰ繝｡繝｢:
  - `node nodesrc/tests.js -i tests -i tutorials -i stdlib ...` 縺ｯ髟ｷ譎る俣邯咏ｶ壹＠縺溘◆繧√�∫｢ｺ隱阪・ `run_doctest.js` 縺ｫ繧医ｋ focused 螳溯｡後∈蛻・ｊ譖ｿ縺医◆縲・
  - 莉雁屓縺ｮ螟画峩縺ｯ build/test 邉ｻ繝ｭ繧ｸ繝・け縺ｧ縺ｯ縺ｪ縺上�・寔險医せ繧ｯ繝ｪ繝励ヨ蜊倅ｽ薙・謾ｹ蝟・〒縺ゅｋ縲・

# 2026-03-06 菴懈･ｭ繝｡繝｢ (feat: examples/bf.nepl 縺ｫ Brainfuck Runner 繧貞ｮ溯｣・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (alloc/collections/sparse_set 隱ｿ譟ｻ邯咏ｶ壹・譛ｪ commit)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `alloc/collections` 縺ｫ `SparseSet` 繧端霑ｽ蜉�/縺､縺・°]縺励�～[0, n)` [遽・峇/縺ｯ繧薙＞]縺ｮ integer set 繧・O(1) membership / insert / remove 縺ｧ[謇ｱ/縺ゅ▽縺犠縺医ｋ繧医≧縺ｫ縺吶ｋ縲・
- [騾ｲ謐・縺励ｓ縺｡繧・￥]:
  - `SparseSet` 縺ｮ public API (`new` / `len` / `universe_len` / `contains` / `insert` / `remove` / `clear` / `free`) 縺ｨ public doctest / fixture 縺ｯ[荳�騾・縺ｲ縺ｨ縺ｨ縺馨繧骸菴懈・/縺輔￥縺帙＞]貂医∩縲・
  - normal path 縺ｯ focused 螳溯｡後〒[騾夐℃/縺､縺・°]縺励※縺・ｋ縲・
    - `stdlib/alloc/collections/sparse_set.nepl::doctest#1/#2`
    - `stdlib/tests/sparse_set.n.md::doctest#1`
    - `tests/stdlib/sparse_set_collections.n.md::doctest#1`
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転縺ｮ[蛻・縺江繧骸蛻・繧従縺・
  - [蠖灘・/縺ｨ縺・＠繧Ⅹ縺ｯ `SparseSet` owner [蜀・Κ/縺ｪ縺・・]縺ｮ field [隱ｭ/繧・縺ｿ[蜃ｺ/縺�]縺励′[螢・縺薙ｏ]繧後※縺・ｋ繧医≧縺ｫ[隕・縺ｿ]縺医◆縺後�”eader 繧・`MemPtr<u8>` field 縺ｧ[謖・繧・縺､險ｭ險医°繧・raw `i32` pointer [菫晄戟/縺ｻ縺肋縺ｸ[關ｽ/縺馨縺ｨ縺吶％縺ｨ縺ｧ normal path 縺ｯ[螳牙ｮ・縺ゅｓ縺ｦ縺Ь縺励◆縲・
  - 縺昴・[蠕・縺ゅ→]縺ｫ[谿・縺ｮ縺転縺｣縺・failure 縺ｯ invalid index path 縺�縺代〒縲～contains s 8` 縺ｮ[譛�蟆丈ｾ・縺輔＞縺励ｇ縺・ｌ縺Ь縺ｾ縺ｧ[邵ｮ蟆・縺励ｅ縺上＠繧・≧]縺ｧ縺阪◆縲・
  - 縺輔ｉ縺ｫ[霑ｽ霍｡/縺､縺・○縺江縺吶ｋ縺ｨ縲～SparseSet` [蝗ｺ譛・縺薙ｆ縺・縺ｧ縺ｯ縺ｪ縺・`sparse_set_diag_index` 縺ｮ[荳ｭ/縺ｪ縺犠縺ｧ[菴・縺､縺従繧・message string 縺・web compile path 縺ｧ `RuntimeError: memory access out of bounds` 繧端襍ｷ/縺馨縺薙＠縺ｦ縺・ｋ縺薙→縺啓蛻・繧従縺九▲縺溘�・
  - `diag_error StdErrorKind::IndexOutOfBounds "abc"` 縺ｯ pass 縺吶ｋ荳�譁ｹ縲～concat "sparse_set_contains" ": index out of bounds "` 繧端蜷ｫ/縺ｵ縺従繧� chain 縺�縺代′ trap 縺吶ｋ縲・
  - `stdlib/alloc/string.nepl::doctest#4` 繧・蜷檎ｳｻ邨ｱ/縺ｩ縺・￠縺・→縺・縺ｮ web path OOB 繧端謖・繧・縺｣縺ｦ縺翫ｊ縲～SparseSet` invalid path failure 縺ｯ[譌｢蟄・縺阪◇繧転縺ｮ `alloc/string` regression 縺ｫ[荵・縺ｮ]縺｣縺ｦ縺・ｋ縺ｨ[蛻､譁ｭ/縺ｯ繧薙□繧転縺励◆縲・
  - native compiler 縺ｧ縺ｯ `SparseSet invalid index` 縺ｮ[譛�蟆丈ｾ・縺輔＞縺励ｇ縺・ｌ縺Ь縺ｯ pass 縺励�『eb compile path 縺�縺代′ trap 縺吶ｋ縺ｮ縺ｧ縲ー逶ｴ謗･/縺｡繧・￥縺帙▽]縺ｮ blocker 縺ｯ stdlib API 險ｭ險医〒縺ｪ縺・web compiler/runtime path [蛛ｴ/縺後ｏ]縺ｫ縺ゅｋ縲・
- [蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `SparseSet` normal path 縺ｮ library 螳溯｣・・[謌千ｫ・縺帙＞繧翫▽]縺励※縺・ｋ縺後�（nvalid index 縺ｮ `Result::Err` path 繧端蜷ｫ/縺ｵ縺従繧� focused suite 縺・web compile path 縺ｧ[譛ｪ蜿取據/縺ｿ縺励ｅ縺・◎縺従縺ｮ縺溘ａ縲∫樟譎らせ縺ｧ縺ｯ commit 縺励↑縺・�・
  - [谺｡/縺､縺讃縺ｯ `alloc/string` 縺ｮ concat / integer-to-string [邨瑚ｷｯ/縺代＞繧江繧・root cause 繝吶・繧ｹ縺ｧ[逶ｴ/縺ｪ縺馨縺励�√◎縺ｮ[蠕・縺ゅ→]縺ｫ `SparseSet` batch 繧端蜀埼幕/縺輔＞縺九＞]縺吶ｋ縲・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (ci: rust install -> cargo build -> trunk build 繧貞・騾・action 蛹・

- 逶ｮ逧・
  - GitHub Actions 縺ｫ謨｣縺｣縺ｦ縺・◆ `Node setup` / `Rust toolchain` / `wasm32 target` / `wasm-bindgen-cli` / `cargo build` / `trunk build` 縺ｮ驥崎､・ｒ 1 邂・園縺ｸ髮・ｴ・☆繧九�・
  - 蜷・workflow 縺ｯ縲悟・騾・build artifact 繧剃ｽ懊ｋ job縲阪→縲後◎縺ｮ artifact 繧貞女縺代※ test / deploy 繧定｡後≧ job縲阪↓蛻・￠縲｜uild 貂医∩謌先棡迚ｩ繧貞・蛻ｩ逕ｨ縺吶ｋ蠖｢縺ｸ蟇・○繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - `compile-test.yml` / `nepl-test-wasi.yml` / `nepl-test-llvm.yml` / `nmd-doctest.yml` / `nm-compile.yml` / `rust-test..yml` / `gh-pages.yml` 縺後�√◎繧後◇繧悟挨縺ｫ toolchain install 縺ｨ `trunk build` 繧呈戟縺｣縺ｦ縺・◆縲・
  - 縺昴・縺溘ａ謇矩�・・譖ｴ譁ｰ貍上ｌ縺瑚ｵｷ縺阪ｄ縺吶￥縲～trunk` 繧・`wasm-bindgen-cli` 縺ｮ譖ｴ譁ｰ縲～Trunk.toml` Linux 陬懈ｭ｣縲‘xamples 驟咲ｽｮ縺ｪ縺ｩ繧呈ｯ主屓螟夐㍾邂｡逅・☆繧区ｧ矩��縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
- 螟画峩:
  - `.github/actions/bootstrap-build/action.yml`
    - CI 蜈ｱ騾壹・ local composite action 繧定ｿｽ蜉�縲・
    - `actions/setup-node`縲～npm install`縲～actions-rs/toolchain`縲～rustup target add wasm32-unknown-unknown`縲～jetli/trunk-action`縲～wasm-bindgen-cli` install縲～Swatinem/rust-cache`縲～cargo build --locked`縲～trunk build --release` 繧帝寔邏・�・
  - `.github/workflows/compile-test.yml`
  - `.github/workflows/rust-test..yml`
  - `.github/workflows/nm-compile.yml`
  - `.github/workflows/nmd-doctest.yml`
  - `.github/workflows/nepl-test-wasi.yml`
  - `.github/workflows/nepl-test-llvm.yml`
    - 縺昴ｌ縺槭ｌ `build` job 縺ｧ蜈ｱ騾・action 繧剃ｽｿ縺｣縺ｦ `dist` / `target/debug` / `target/wasm32-unknown-unknown` 繧・artifact 蛹悶�・
    - test job 蛛ｴ縺ｯ `actions/download-artifact` 縺ｧ蜿門ｾ励＠縺ｦ縺九ｉ縲∝推 workflow 蝗ｺ譛峨・ `cargo test` / `nodesrc/tests.js` / `cargo run -p nepl-cli` / LLVM runner 繧貞ｮ溯｡後☆繧句ｽ｢縺ｸ螟画峩縲・
  - `.github/workflows/gh-pages.yml`
    - pages 蝗ｺ譛峨・ deploy/doctest/doc build 縺ｯ谿九＠縺､縺､縲》oolchain install 縺ｨ build 譛ｬ菴薙・蜈ｱ騾・action 縺ｸ遘ｻ蜍輔�・
- 讀懆ｨｼ:
  - 荳�譎・directory `/tmp/gha-yaml-check` 繧剃ｽ懊▲縺ｦ `npm install yaml` 繧定｡後＞縲∝・ workflow 縺ｨ composite action 繧・`yaml` parser 縺ｧ讒区枚遒ｺ隱阪�・
    - 蟇ｾ雎｡:
      - `.github/workflows/*.yml`
      - `.github/actions/bootstrap-build/action.yml`
    - 邨先棡: 蜈ｨ莉ｶ `OK`
- 蟾ｮ逡ｰ繝｡繝｢:
  - workflow 螳溯｡後◎縺ｮ繧ゅ・縺ｯ GitHub Actions 荳翫〒縺ｮ螳溯｡後′蠢・ｦ√↑縺ｮ縺ｧ縲√Ο繝ｼ繧ｫ繝ｫ縺ｧ縺ｯ YAML 讒区枚縺ｨ萓晏ｭ倬未菫ゅ・謨ｴ蜷医∪縺ｧ繧堤｢ｺ隱阪＠縺溘�・
  - 迴ｾ譎らせ縺ｧ縺ｯ artifact 縺ｮ邊貞ｺｦ繧・`dist` / `target/debug` / `target/wasm32-unknown-unknown` 縺ｫ縺励※縺・ｋ縲ゅ＆繧峨↓邨槭ｋ菴吝慍縺ｯ縺ゅｋ縺後�√∪縺壹・蜈ｱ騾壼喧縺ｨ蜀榊茜逕ｨ縺ｮ謌千ｫ九ｒ蜆ｪ蜈医＠縺溘�・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (ci: build 1 蝗・+ pages/test 邨ｱ蜷・+ per-case timeout)

- 逶ｮ逧・
  - workflow 縺斐→縺ｫ `bootstrap-build` 繧堤ｹｰ繧願ｿ斐＠縺ｦ縺・◆讒区・繧偵ｄ繧√�～trunk build` 繧貞性繧� build 繧・1 workflow 蜀・〒 1 蝗槭□縺大ｮ溯｡後＠縲√◎縺ｮ謌先棡迚ｩ繧貞・ test job 縺ｨ Pages deploy 縺ｫ蜀榊茜逕ｨ縺吶ｋ縲・
  - `gh-pages.yml` 縺悟挨 workflow 縺ｧ test 繧貞・螳溯｡後＠縺ｦ縺・◆讒矩��繧定ｧ｣豸医＠縲《ite 縺ｸ縺ｮ publish 繧・test workflow 縺ｮ荳�驛ｨ縺ｸ邨ｱ蜷医☆繧九�・
  - 辟｡髯舌Ν繝ｼ繝礼ｳｻ縺ｮ hang 縺ｧ CI 蜈ｨ菴薙′豁｢縺ｾ繧峨↑縺・ｈ縺・�・ 繧ｱ繝ｼ繧ｹ 20 遘偵�》est job 蜈ｨ菴・10 蛻・・荳企剞繧貞・繧後ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - 蜑肴ｮｵ縺ｮ蜈ｱ騾・action 蛹悶□縺代〒縺ｯ縲『orkflow 縺悟・縺九ｌ縺ｦ縺・ｋ髯舌ｊ `cargo build` / `trunk build` / `npm install` / `cargo install wasm-bindgen-cli` 縺・workflow 謨ｰ縺�縺醍ｹｰ繧願ｿ斐＆繧後ｋ縲・
  - `gh-pages.yml` 縺ｯ site 逕滓・縺ｮ縺溘ａ縺ｫ tests 繧貞・蠎ｦ蝗槭＠縺ｦ縺翫ｊ縲∝酔縺・commit 縺ｫ蟇ｾ縺励※ test 縺・2 驥榊ｮ溯｡後＆繧後※縺・◆縲・
  - `nodesrc/tests.js` 縺ｯ suite 蜈ｨ菴薙・螳溯｡後・縺ｧ縺阪※繧ゅ�仝ASM worker / LLVM child process 縺ｫ per-case timeout 縺檎┌縺上�・ 繧ｱ繝ｼ繧ｹ縺ｮ hang 縺・suite 蜈ｨ菴薙ｒ蠑輔″縺壹ｋ菴吝慍縺後≠縺｣縺溘�・
- 螟画峩:
  - `.github/actions/bootstrap-build/action.yml`
    - `actions/setup-node` 縺ｫ npm cache 繧定ｿｽ蜉�縲・
    - `web/package-lock.json` 繝吶・繧ｹ縺ｧ `npm ci` 繧剃ｽｿ縺・ｽ｢縺ｫ螟画峩縲・
    - `wasm-bindgen-cli` 繧・`actions/cache` 縺ｧ蜀榊茜逕ｨ縺吶ｋ繧医≧螟画峩縲・
    - `wasm-bindgen` 縺ｮ verify step 繧定ｿｽ蜉�縲・
  - `.github/workflows/ci.yml`
    - 譌ｧ test workflow 鄒､縺ｨ Pages deploy 繧・1 workflow 縺ｫ邨ｱ蜷医�・
    - `build` job 縺ｧ `bootstrap-build` 繧・1 蝗槭□縺大ｮ溯｡後＠縲√＆繧峨↓ tutorial / stdlib HTML 繧・`dist` 驟堺ｸ九∈逕滓・縺励※ artifact 蛹悶�・
    - `compile-test` / `rust-test` / `nm-compile` / `wasi-test` / `nmd-doctest` / `llvm-test` 縺ｯ縺吶∋縺ｦ `needs: build` 縺ｧ artifact 繧貞・蛻ｩ逕ｨ縲・
    - `pages-fast-*` 縺ｨ `pages-final-*` 縺ｮ 2 谿ｵ deploy 繧定ｿｽ蜉�縺励�～trunk build` 蠕後・ pending site 繧貞・縺ｫ publish 縺励�》est 螳御ｺ・ｾ後↓ test JSON / summary 繧定ｼ峨○縺・final site 縺ｧ荳頑嶌縺阪☆繧句ｽ｢縺ｫ縺励◆縲・
    - `gh-pages.yml` 縺ｯ蜑企勁縲・
    - test job 縺ｫ縺ｯ `timeout-minutes: 10` 繧定ｿｽ蜉�縺励�～node nodesrc/tests.js` / `cargo test` / `cargo run` 縺ｯ `timeout --signal=KILL 10m ...` 縺ｧ蛹・ｓ縺�縲・
    - test 螳溯｡檎腸蠅・↓ `NEPL_TEST_CASE_TIMEOUT_MS=20000` / `NEPL_WASIX_TIMEOUT_MS=20000` 繧貞・騾壽欠螳壹�・
  - `nodesrc/tests.js`
    - WASM thread pool worker 縺ｫ per-case timer 繧定ｿｽ蜉�縺励�・0 遘偵〒蠢懃ｭ斐＠縺ｪ縺・case 縺ｯ worker 繧・terminate 縺励※ error 縺ｨ縺励※蝗槫庶縺吶ｋ蠖｢縺ｸ螟画峩縲・
    - LLVM / native 螳溯｡後↓菴ｿ縺・`runCommand` 縺ｫ child process timeout 繧定ｿｽ蜉�縺励�∝酔縺倥￥ 20 遘偵〒 kill 縺吶ｋ繧医≧螟画峩縲・
- 讀懆ｨｼ:
  - `node --check nodesrc/tests.js`
  - 荳�譎・directory `/tmp/gha-yaml-check` 繧剃ｽ懊▲縺ｦ `npm install yaml` 繧定｡後＞縲・
    - `.github/workflows/*.yml`
    - `.github/actions/bootstrap-build/action.yml`
    繧・parser 縺ｧ讀懆ｨｼ縲・
- 蟾ｮ逡ｰ繝｡繝｢:
  - Pages final deploy 縺ｯ `build` artifact 縺ｮ `dist` 繧貞・蛻ｩ逕ｨ縺励�《ite 繧剃ｽ懊ｋ縺溘ａ縺ｫ `trunk build` 繧貞・螳溯｡後＠縺ｪ縺・�・
  - pending/final 縺ｮ 2 蝗・deploy 縺ｯ Pages 縺ｸ縺ｮ publish 繧呈掠繧√ｋ縺溘ａ縺ｮ繧ゅ・縺ｧ縲》ests 閾ｪ菴薙・ 1 蝗槭＠縺句ｮ溯｡後＠縺ｪ縺・�・
  - 蛻晉沿縺ｧ縺ｯ `site-fast` / `site-final` 繧帝�壼ｸｸ縺ｮ `upload-artifact` 縺ｧ荳ｭ邯吶＠縺ｦ縺九ｉ `upload-pages-artifact` 縺ｫ貂｡縺励※縺・◆縺後�‥ownload 譎ゅ↓ `dist` directory 縺ｮ髫主ｱ､蜑肴署縺悟ｴｩ繧後※ `tar: dist: Cannot open` 縺ｫ縺ｪ縺｣縺溘�・
  - 縺昴・縺溘ａ Pages 逕ｨ bundle job 縺ｯ逶ｴ謗･ `upload-pages-artifact` 繧定｡後＞縲‥eploy job 縺ｯ `deploy-pages` 縺�縺代ｒ陦後≧讒矩��縺ｸ菫ｮ豁｣縺励◆縲・

- 逶ｮ逧・
  - `rpn.nepl` 繧貞盾閠・↓縺励※ `examples/bf.nepl` 縺ｫ Brainfuck 縺ｮ螳溯｡後ヤ繝ｼ繝ｫ繧貞ｮ溯｣・☆繧九�・
  - 豈手｡悟・蜉帙ｒ蜿励￠莉倥￠縲∝・蜉帙＃縺ｨ縺ｫ繝｡繝｢繝ｪ繧偵Μ繧ｻ繝・ヨ縺励※迢ｬ遶句ｮ溯｡後☆繧九�・
- 螟画峩:
  - `examples/bf.nepl`
    - `alloc/collections/stack` 繧剃ｽｿ縺｣縺ｦ `[` 縺ｨ `]` 縺ｮ繧ｸ繝｣繝ｳ繝怜・繧剃ｺ句燕險育ｮ励☆繧・`compile_jumps` 繧貞ｮ溯｣・�・
    - `eval_line` 縺ｧ 30,000 繝舌う繝医・繝｡繝｢繝ｪ荳翫〒 BF 蜻ｽ莉､・・+` `-` `>` `<` `.` `,` `[` `]`・峨ｒ螳溯｡後�・
    - `,` 縺ｯ迴ｾ迥ｶ 0 繧呈嶌縺崎ｾｼ繧�邁｡逡･螳溯｣・�・
    - 繝｡繧､繝ｳ繝ｫ繝ｼ繝励・蜈･蜉帙＃縺ｨ縺ｫ繝｡繝｢繝ｪ繝舌ャ繝輔ぃ繧堤｢ｺ菫昴・隗｣謾ｾ縺励�∫憾諷九ｒ蠑輔″邯吶′縺ｪ縺・�・
    - 陦ｨ遉ｺ蜷阪・ "Brainfuck REPL" 縺九ｉ "Brainfuck Runner" 縺ｫ螟画峩・域ｯ手｡後Μ繧ｻ繝・ヨ縺ｮ縺溘ａ・峨�・
    - `neplg2:test[bf_hello_world]` doctest 繧定ｿｽ蜉�・・ello World 繝励Ο繧ｰ繝ｩ繝�縺ｮ螳溯｡鯉ｼ峨�・
- 讀懆ｨｼ:
  - `target/debug/nepl-cli -i examples/bf.nepl -o tmp/wasm.wasm && wasmer run tmp/wasm.wasm`
    - `+++++++++[>++++++++>+++++++++++>+++>+<<<<-]>.>++.+++++++..+++.>+++++.<<+++++++++++++++.>.+++.------.--------.>+.>+.` 繧貞・蜉帙＠縺ｦ `Hello World!` 縺ｮ蜃ｺ蜉帙ｒ遒ｺ隱阪�・

# 2026-03-06 菴懈･ｭ繝｡繝｢ (TUI謾ｹ蝟・ rpn縺ｮ騾比ｸｭ險育ｮ怜庄隕門喧縺ｨstdio縺ｮ雋�謨ｰ蜃ｺ蜉帑ｿｮ豁｣)

- 逶ｮ逧・
  - `examples/rpn.nepl` 縺ｫ縺翫＞縺ｦ縲～>` 繝励Ο繝ｳ繝励ヨ縺ｮ蜍穂ｽ懊ｒ繝ｬ繧ｬ繧ｷ繝ｼ迚医↓蜷医ｏ縺帙�∬ｨ育ｮ鈴℃遞九ｒ縲瑚ｨ育ｮ怜燕縲阪�瑚ｨ育ｮ怜ｾ後�阪→縺励※ANSI繧ｫ繝ｩ繝ｼ縺ｧ蜿ｯ隕門喧縺吶ｋ縲・
  - 騾比ｸｭ險育ｮ励ｄ蜃ｺ蜉帙〒雋�謨ｰ繧貞性繧�蠑上′豁｣縺励￥陦ｨ遉ｺ縺輔ｌ繧九ｈ縺・�～stdlib/std/stdio.nepl` 縺ｮ `print_i32` 縺ｫ蟄伜惠縺吶ｋ雋�謨ｰ蜃ｺ蜉帙ヰ繧ｰ繧剃ｿｮ豁｣縺吶ｋ縲・
- 螟画峩:
  - `examples/rpn.nepl`
    - REPL繝励Ο繝ｳ繝励ヨ蜃ｺ蜉帛燕縺ｫ繝医・繧ｯ繝ｳ陦後ｒ莠碁㍾縺ｫ蜃ｺ蜉帙＠縺ｪ縺・ｈ縺・・髟ｷ縺ｪ繝ｫ繝ｼ繝励ｒ蜑企勁縲・
    - `print_step_before` 繧定ｿｽ蜉�縺励�∬ｨ育ｮ怜燕縺ｮ迥ｶ諷九ｒ繧ｷ繧｢繝ｳ (`ansi_cyan`) 縺ｧ蠑ｷ隱ｿ陦ｨ遉ｺ縲・
    - `print_step_after` 繧定ｿｽ蜉�縺励�∬ｨ育ｮ礼ｵ先棡繧堤ｷ題牡 (`ansi_green`) 縺ｧ蠑ｷ隱ｿ陦ｨ遉ｺ縲・
  - `stdlib/std/stdio.nepl`
    - `print_i32` 髢｢謨ｰ縺ｧ雋�縺ｮ謨ｰ縺ｸ縺ｮ險育ｮ励′荳崎ｶｳ縺励※ `0` 縺ｨ縺ｪ繧九ヰ繧ｰ繧剃ｿｮ豁｣縲らｵｶ蟇ｾ蛟､縺ｮ蜷・｡√ｒ騾・�・ｱ暮幕縺励◆縺ｮ縺｡縲∬ｲ�謨ｰ縺ｧ縺ゅｌ縺ｰ `-` 隨ｦ蜿ｷ繧剃ｻ倅ｸ弱☆繧九ｈ縺・隼菫ｮ縲・
    - 繧ｳ繝ｳ繝代う繝ｫ繧ｨ繝ｩ繝ｼ繧貞｡槭＄縺溘ａ `mod_u` 繧・`rem_u` 縺ｫ菫ｮ豁｣縲・
- 邨先棡:
  - `1 2 + 3 + 4 5 + 6 +` 縺ｪ縺ｩ縺ｮ騾｣邯壼・蜉帙↓蟇ｾ縺励※縲∝・逅・＃縺ｨ縺ｮ險育ｮ礼ｮ・園 (`[1 2 +]` 縺ｪ縺ｩ) 縺ｨ邨先棡縺瑚牡莉倥″縺ｧ蛻・°繧翫ｄ縺吶￥陦ｨ遉ｺ縺輔ｌ繧九ｈ縺・↓縺ｪ縺｣縺溘�・
  - `-5` 縺ｪ縺ｩ縺ｮ雋�縺ｮ謨ｰ繧貞・蜉帙＠縺滄圀縺ｫ豁｣蟶ｸ縺ｫ陦ｨ遉ｺ縺輔ｌ繧九ｈ縺・↓縺ｪ縺｣縺溘�・
- 讀懆ｨｼ:
  - `target/debug/nepl-cli -i examples/rpn.nepl -o tmp/wasm.wasm && wasmer run tmp/wasm.wasm`
    - 騾比ｸｭ險育ｮ励・繝医Ξ繝ｼ繧ｹ縺翫ｈ縺ｳ雋�謨ｰ (`1 2 3 4 + - 5 +` -> `-5`) 縺ｮ豁｣縺励＞繝輔か繝ｼ繝槭ャ繝医→蜃ｺ蜉帙ｒ逶ｴ謗･遒ｺ隱阪�・

# 2026-03-06 菴懈･ｭ繝｡繝｢ (蝙句ｮ牙・蛹・ `alloc/string` 縺ｮ荳ｻ隕・raw 遒ｺ菫昴ｒ `RegionToken<u8>` 蛹・

- 逶ｮ逧・
  - `alloc/string` 縺ｮ荳ｻ隕∫函謌千ｵ瑚ｷｯ縺九ｉ `alloc_raw` 繧貞叙繧企勁縺阪�～core/mem` 縺ｮ蝙倶ｻ倥″鬆伜沺 API 縺ｫ蟇・○繧九�・
  - 譁・ｭ怜・逕滓・蜃ｦ逅・〒髟ｷ縺輔・繝・ム縺ｨ譛ｬ譁・・繧､繝ｳ繧ｿ繧・`MemPtr<T>` / `RegionToken<T>` 縺ｧ謇ｱ縺・�∝・驛ｨ縺ｮ逕溘・繧､繝ｳ繧ｿ髴ｲ蜃ｺ繧呈ｸ帙ｉ縺吶�・
- 螟画峩:
  - `stdlib/alloc/string.nepl`
    - `string_alloc_region`
    - `string_region_len_ptr`
    - `string_region_data_ptr`
    - `string_data_ptr`
    - `string_finish`
    繧定ｿｽ蜉�縺励�∵枚蟄怜・繝ｬ繧､繧｢繧ｦ繝亥ｰら畑縺ｮ蜀・Κ繝倥Ν繝代→縺励※謨ｴ逅・�・
  - `concat`
    - 蜃ｺ蜉帶枚蟄怜・縺ｮ遒ｺ菫昴ｒ `string_alloc_region` 縺ｫ螟画峩縲・
    - 蜃ｺ蜉帛・繧ｳ繝斐・繧・`MemPtr<u8>` 繝吶・繧ｹ縺ｸ螟画峩縲・
  - `sb_build`
    - 騾｣邨仙・繝舌ャ繝輔ぃ縺ｮ遒ｺ菫昴ｒ `RegionToken<u8>` 蛹悶�・
    - 蜷・part 縺ｮ隱ｭ縺ｿ蜃ｺ縺励→蜃ｺ蜉帛・譖ｸ縺崎ｾｼ縺ｿ繧貞梛莉倥″繝昴う繝ｳ繧ｿ縺ｸ螟画峩縲・
  - `str_slice`
    - 蛻・ｊ蜃ｺ縺怜・縺ｮ遒ｺ菫昴ｒ `RegionToken<u8>` 蛹悶�・
  - `from_u128_radix`
    - 騾・�・｡∫ｩ阪∩縺ｮ scratch 繧・`RegionToken<u8>` 蛹悶�・
    - 荳�譎・scratch 縺ｯ `dealloc_region` 縺ｧ隗｣謾ｾ縲・
  - `from_f64`
    - 蟆乗焚驛ｨ scratch 繧・`RegionToken<u8>` 蛹悶�・
    - scratch 隗｣謾ｾ繧定ｿｽ蜉�縲・
- 邨先棡:
  - `stdlib/alloc/string.nepl` 縺九ｉ `alloc_raw/realloc_raw/dealloc_raw` 縺ｮ逶ｴ謗･蜻ｼ縺ｳ蜃ｺ縺励・豸医∴縺溘�・
  - `str` 縺ｮ蜀・Κ陦ｨ迴ｾ閾ｪ菴薙・縺ｾ縺� raw address 縺�縺後�∽ｸｻ隕√↑逕滓・邨瑚ｷｯ縺ｧ縺ｯ `RegionToken<u8>` 縺九ｉ `string_finish` 縺ｧ遒ｺ螳壹☆繧区ｵ√ｌ縺ｫ謨ｴ逅・〒縺阪◆縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/stdlib.n.md -i tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md --no-stdlib --no-tree -o /tmp/tests-string-type-safety-v1.json -j 15`
    - 邨先棡: `26/26 pass`
  - `rg -n "alloc_raw|realloc_raw|dealloc_raw" stdlib/alloc/string.nepl`
    - 邨先棡: 隧ｲ蠖薙↑縺・

# 2026-03-06 菴懈･ｭ繝｡繝｢ (alloc/string: i128/u128 縺ｨ蝓ｺ謨ｰ莉倥″譁・ｭ怜・螟画鋤縺ｮ謨ｴ蛯・

- 逶ｮ逧・
  - `alloc/string` 縺ｫ謨ｴ謨ｰ縺ｮ譁・ｭ怜・陦ｨ迴ｾ螟画鋤繧帝寔邏・＠縲～core/cast` 縺ｨ縺ｮ雋ｬ蜍吶ｒ蛻・屬縺吶ｋ縲・
  - `i128` / `u128` 繧貞性繧� 2/8/10/16 騾ｲ縺ｮ螟画鋤繧呈署萓帙☆繧九�・
  - tutorial 縺ｫ縲∵焚蛟､ cast 縺ｨ譁・ｭ怜・螟画鋤縺ｮ驕輔＞繧呈・遉ｺ縺励◆蟆守ｷ壹ｒ霑ｽ蜉�縺吶ｋ縲・
- 螟画峩:
  - `stdlib/alloc/string.nepl`
    - `from_bool`
    - `to_bool`
    - `from_u128` / `from_u128_radix`
    - `to_u128` / `to_u128_radix`
    - `from_i128` / `from_i128_radix`
    - `to_i128` / `to_i128_radix`
    - `u128_divrem_small` 縺ｪ縺ｩ 128-bit 謨ｴ謨ｰ縺ｮ陬懷勧髢｢謨ｰ鄒､
    - `to_i32` 縺ｮ隱ｬ譏弱ｒ迴ｾ螳溯｣・↓蜷医ｏ縺帙※譖ｴ譁ｰ
  - `tests/stdlib.n.md`
    - `i128/u128` 縺ｨ雋�謨ｰ16騾ｲ縺ｮ focused case 繧定ｿｽ蜉�
  - `tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md`
    - `core/cast` 縺ｨ `alloc/string` 縺ｮ菴ｿ縺・・縺・
    - `Result` 繧定ｿ斐☆隗｣譫宣未謨ｰ
    - 2/8/10/16 騾ｲ螟画鋤
    - `i128/u128` 縺ｮ螟ｧ縺阪＞蛟､縺ｮ萓・
  - `tutorials/getting_started/00_index.n.md`
    - 譁ｰ隕・tutorial 縺ｸ縺ｮ蟆守ｷ壹ｒ霑ｽ蜉�
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/stdlib.n.md --no-stdlib --no-tree -o /tmp/tests-stdlib-conversions-i128-v3.json -j 15`
    - 邨先棡: `19/19 pass`

# 2026-03-06 菴懈･ｭ繝｡繝｢ (蝙句ｮ牙・蛹・ `ptr_cast` 蜈ｬ髢句ｻ・ｭ｢)

- 逶ｮ逧・
  - 繝昴う繝ｳ繧ｿ蜀崎ｧ｣驥医・繧医≧縺ｪ unsafe 縺ｪ蜈ｬ髢・API 繧呈ｸ帙ｉ縺励�～MemPtr<T>` / `RegionToken<T>` 繝｢繝・Ν縺ｸ蟇・○繧九�・
- 螟画峩:
  - `stdlib/core/cast.nepl`
    - 譛ｪ菴ｿ逕ｨ縺�縺｣縺・`ptr_cast` 繧貞炎髯､縲・
    - 繝｢繧ｸ繝･繝ｼ繝ｫ蜈磯�ｭ繧ｳ繝｡繝ｳ繝医ｒ縲∵焚蛟､ cast 縺ｨ bitcast 縺ｮ縺ｿ縺ｫ雋ｬ蜍吶ｒ髯仙ｮ壹☆繧玖ｪｬ譏弱∈譖ｴ譁ｰ縲・
- 蛻､譁ｭ:
  - `ptr_cast` 縺ｯ蝙九□縺代ｒ莉倥￠譖ｿ縺医ｋ謫堺ｽ懊〒縲～MemPtr<T>` 縺ｫ繧医ｋ蝙句ｮ牙・蛹匁婿驥昴→遏帷崟縺吶ｋ縲・
  - repo 蜀・盾辣ｧ縺ｯ辟｡縺上�∫樟譎らせ縺ｧ蜈ｬ髢矩擇縺ｫ谿九☆蜷育炊諤ｧ縺ｯ辟｡縺九▲縺溘�・
  - `MemPtr<T>` 縺ｯ縲悟梛莉倥″繧｢繝峨Ξ繧ｹ縲阪�～RegionToken<T>` 縺ｯ縲後◎縺ｮ鬆伜沺縺ｮ繧ｵ繧､繧ｺ縺ｨ謇�譛画ｨｩ縲阪ｒ莨ｴ縺・ｷ壼ｽ｢繝医・繧ｯ繝ｳ縺ｨ縺励※菴ｿ縺・・縺代ｋ縲・

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺF: tutorials Part6 諡｡蜈・+ library-first 蛹・

- 逶ｮ逧・
  - `tutorials/getting_started` Part6・・2縲・7・峨・隱ｬ譏手ｪ､繧翫・荳崎ｶｳ繧堤屮譟ｻ縺励�∫洒縺冗ｰ｡貎斐〒螳牙・縺ｪ譖ｸ縺肴婿縺ｸ譖ｴ譁ｰ縺吶ｋ縲・
  - 逕溘・繧､繝ｳ繧ｿ髴ｲ蜃ｺ繧呈ｸ帙ｉ縺吶◆繧√�～kp` 蛛ｴ縺ｫ `Vec<i32>` 逶ｴ蜿励￠陬懷勧繧定ｿｽ蜉�縺吶ｋ縲・
- 螟画峩:
  - `tutorials/getting_started/22_competitive_io_and_arith.n.md`
    - `Scanner/Writer` 縺ｮ蝓ｺ譛ｬ繝代ち繝ｼ繝ｳ繧・pipe 荳ｭ蠢・↓邁｡貎泌喧縲・
    - i32/i64/遨ｺ逋ｽ蛹ｺ蛻・ｊ蜃ｺ蜉帙・ 3 繧ｱ繝ｼ繧ｹ繧貞ｮ牙・ API 蜑肴署縺ｧ謨ｴ逅・�・
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
    - `Vec + sort + lower/upper_bound` 繧・library-first 縺ｧ蜀肴ｧ区・縲・
  - `tutorials/getting_started/24_competitive_dp_basics.n.md`
    - DP 譛ｬ菴薙ｒ邯ｭ謖√＠縺､縺､ I/O 繧堤ｰ｡貎泌喧縲・
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
    - prefix 繧・`kp/kpprefix` 繝上Φ繝峨Ν API 蜑肴署縺ｸ譖ｴ譁ｰ縲・
    - two pointers 縺ｮ譚｡莉ｶ隧穂ｾ｡繧堤洒邨｡隧穂ｾ｡縺ｫ萓晏ｭ倥＠縺ｪ縺・ｮ牙・縺ｪ蠖｢縺ｸ菫ｮ豁｣縲・
  - `tutorials/getting_started/26_competitive_graph_bfs.n.md`
    - 謇区嶌縺・BFS 縺九ｉ `kp/kpgraph` 蛻ｩ逕ｨ縺ｸ遘ｻ陦後�・
  - `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
    - 譛ｪ螳梧・陦ｨ險倥ｒ蟒・ｭ｢縺励�￣art6 邱上∪縺ｨ繧√→縺励※繝・Φ繝励Ξ繝ｼ繝医・蟇ｾ蠢懆｡ｨ繝ｻ螳滓姶繝輔Ο繝ｼ繧定ｿｽ蜉�縲・
  - `tutorials/getting_started/00_index.n.md`
    - 隱､蟄励ｒ菫ｮ豁｣・磯未謨ｰ縺ｮ縺ｵ繧翫′縺ｪ・峨�・
  - `stdlib/kp/kpprefix.nepl`
    - `PrefixI32` 繝上Φ繝峨Ν縺ｨ `prefix_build_vec_i32` / `prefix_sum_i32` / `prefix_free_i32` 繧定ｿｽ蜉�縲・
  - `stdlib/kp/kpsearch.nepl`
    - `lower_bound_vec_i32` / `upper_bound_vec_i32` / `contains_vec_i32` / `count_equal_range_vec_i32` 繧定ｿｽ蜉�縲・
  - `todo.md`
    - 繝輔ぉ繝ｼ繧ｺF縺ｮ螳御ｺ・ｸ医∩ Part6 蟆ら畑繧ｿ繧ｹ繧ｯ繧貞炎髯､・域悴螳御ｺ・・縺ｿ邯ｭ謖・ｼ峨�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/26_competitive_graph_bfs.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -i stdlib/kp/kpprefix.nepl -i stdlib/kp/kpsearch.nepl --no-tree -o /tmp/tests-part6-kp-refresh-v7.json -j 15`
    - 邨先棡: `219/219 pass`
  - 陬懷勧遒ｺ隱・
    - `node nodesrc/tests.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md --no-tree -o /tmp/tests-part6-25-v6.json -j 15`
    - 邨先棡: `207/207 pass`

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm `add/sub` 蜀榊ｮ夂ｾｩ繝ｪ繝ｳ繧ｯ螟ｱ謨励・譬ｹ譛ｬ菫ｮ豁｣)

- 逶ｮ逧・
  - `--runner all --llvm-all` 螳溯｡梧凾縺ｫ `tests/llvm_target.n.md::doctest#4/#5` 縺・`invalid redefinition of function 'add'/'sub'` 縺ｧ螟ｱ謨励☆繧句撫鬘後ｒ縲∝ｾ御ｻ倥￠蝗樣∩縺ｧ縺ｯ縺ｪ縺冗函謌蝕R讒矩��縺九ｉ隗｣豸医☆繧九�・
- 蜴溷屏:
  - `stdlib/core/math.nepl` 縺ｮ overload 鄒､・・add/sub` 縺ｪ縺ｩ・峨′ `#llvmir` 蜀・〒蜷御ｸ�繧ｷ繝ｳ繝懊Ν蜷搾ｼ・@add`, `@sub`・峨ｒ菴ｿ縺｣縺ｦ縺・◆縲・
  - LLVM 縺ｯ繧ｷ繝ｳ繝懊Ν蜷阪〒 overloading 縺ｧ縺阪↑縺・◆繧√�∝酔荳�繝｢繧ｸ繝･繝ｼ繝ｫ縺ｸ隍・焚蝙狗沿繧貞酔蜷榊ｮ夂ｾｩ縺吶ｋ縺ｨ繝ｪ繝ｳ繧ｯ譎ゅ↓陦晉ｪ√☆繧九�・
  - 縺輔ｉ縺ｫ `u8` 縺ｨ `i32` 縺ｯ LLVM ABI 縺ｧ蜷後§ `i32` 縺ｫ關ｽ縺｡繧九◆繧√�∝梛蛻･ overload 繧偵◎縺ｮ縺ｾ縺ｾ繧ｷ繝ｳ繝懊Ν蜷阪〒蜈ｱ蟄倥＆縺帙ｋ險ｭ險医′謌千ｫ九＠縺ｪ縺・�・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - 逕滓・螳御ｺ・峩蜑阪↓ `deduplicate_overloaded_llvm_symbols` 繧定ｿｽ蜉�縺励�∝酔蜷・`define` 繧偵す繧ｰ繝阪メ繝｣蜊倅ｽ阪〒荳�諢丞喧縲・
    - `define` 蛛ｴ縺ｮ驥崎､・ｒ `name__ovN_<sig>` 縺ｸ豁｣隕丞喧縺励�∝ｯｾ蠢懊☆繧・`call` 蜿ら・繧ょ酔荳�繧ｷ繧ｰ繝阪メ繝｣縺ｧ蠑ｵ繧頑崛縺医ｋ縲・
    - 蜑肴ｮｵ縺ｨ縺励※ `#llvmir` 蜻ｼ縺ｳ蜃ｺ縺苓ｦ∽ｻｶ謚ｽ蜃ｺ縺ｨ AST raw-body 驕ｸ蛻･陬懷勧繧定ｿｽ蜉�縺励�∽ｸ崎ｦ√↑ overload 蜃ｺ蜉帙ｒ謚大宛縲・
- 讀懆ｨｼ:
- `NO_COLOR=false trunk build` -> success
- `cargo build -p nepl-cli` -> success
- `node nodesrc/tests.js -i tests/llvm_target.n.md --no-stdlib --no-tree --runner all --llvm-all -o /tmp/tests-llvm-target-after-dedup-pass.json -j 15` -> `6/6 pass`
- `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-dedup.json -j 15` -> `791/791 pass`

# 2026-03-12 菴懈･ｭ繝｡繝｢ (refactor(vec): Result 蛹悶＠縺・Vec API 繧堤峩謗･萓晏ｭ伜・縺ｸ莨晄眺)

- 逶ｮ逧・
  - `alloc/collections/vec` 縺ｮ `new / with_capacity / push` 繧・`Result<..., StdErrorKind>` 蛹悶＠縺溷､画峩繧偵�∫峩謗･萓晏ｭ倥☆繧・stdlib / tests / tutorials 縺ｸ謨ｴ蜷育噪縺ｫ蜿肴丐縺吶ｋ縲・
  - `Vec` 蜀咲｢ｺ菫昴ｒ莨ｴ縺・API 繧・`stack` 邉ｻ縺ｨ蜷後§螟ｱ謨励Δ繝・Ν縺ｸ蟇・○縺､縺､縲∵里蟄倥・鬮俶ｰｴ貅・helper 縺ｧ縺ｯ `unwrap_ok` 蜷ｸ蜿弱〒蛻ｩ逕ｨ閠・・險倩ｿｰ繧帝℃蜑ｰ縺ｫ蟠ｩ縺輔↑縺・�・
- 譬ｹ譛ｬ蜴溷屏:
  - `Vec` 譛ｬ菴薙□縺代ｒ `Result` 蛹悶☆繧九→縲～std/test` / `alloc/string` / `nm/parser` / `kpgraph` / `wasix/tui` 縺ｪ縺ｩ縺梧立 pure API 繧貞燕謠舌↓縺励※螢翫ｌ繧九�・
  - 縺輔ｉ縺ｫ `StdErrorKind` 縺御ｸ贋ｽ阪・ `alloc/diag/error` 縺ｫ縺ゅｋ縺ｨ縲～vec -> diag/error -> vec` 縺ｮ蠕ｪ迺ｰ萓晏ｭ倥′逕溘§繧九�・
- 螟画峩:
  - `stdlib/alloc/collections/vec.nepl`
    - `new / with_capacity / push` 繧・`Result<..., StdErrorKind>` 蛹悶�・
    - `with_capacity 0` 縺ｯ遒ｺ菫昴ｒ陦後ｏ縺夂ｩｺ `MemPtr` 繧貞桁繧�蠖｢縺ｫ縺励※ `OutOfMemory` 繧剃ｸ崎ｦ∝喧縲・
  - `stdlib/std/test.nepl`
    - `checks_new` / `checks_push` 縺ｧ `Vec<Result<(),str>>` 縺ｮ `Result` 繧貞・驛ｨ蜷ｸ蜿弱�・
  - `stdlib/alloc/string.nepl`
    - `StringBuilder` 縺ｨ `str_split` 縺ｮ蜀・Κ `Vec` 讒狗ｯ峨・霑ｽ蜉�繧・`unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `stdlib/alloc/diag/error.nepl`
    - `Diag` / `Diags` 蜀・Κ縺ｮ `Vec` 讒狗ｯ峨・霑ｽ蜉�繧・`unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `stdlib/alloc/hash/sha256.nepl`
    - scaffold 螳溯｣・・ buffer 讒狗ｯ峨・譖ｴ譁ｰ繧・`unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `stdlib/kp/kpgraph.nepl`
    - BFS 邨先棡繝吶け繧ｿ讒狗ｯ峨ｒ `unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `stdlib/platforms/wasix/tui.nepl`
    - `text_wrap_lines` 縺ｮ陦碁・蛻玲ｧ狗ｯ峨ｒ `unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `stdlib/nm/parser.nepl`
    - inline/block parser 蜀・Κ縺ｮ `Vec` 讒狗ｯ峨・霑ｽ蜉�繧・`unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `stdlib/tests/vec.n.md`
    - current `Vec Result` API 縺ｫ蜷梧悄縲・
  - `tests/stdlib/traits_order.n.md`
    - sort regression 縺ｮ `Vec` 讒狗ｯ峨ｒ `unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `tests/stdlib/selfhost_req.n.md`
    - `Vec<u8>` buffer 讒狗ｯ峨ｒ `unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `tests/stdlib/sort.n.md`
    - sort fixture 縺ｮ `Vec` 讒狗ｯ峨ｒ `unwrap_ok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
  - `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
    - `Vec` pipe 騾｣骼悶ｒ `unwrap_ok new` 縺ｨ `|> push ... |> uwok` 縺ｮ current 譖ｸ蠑上∈譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/tests/vec.n.md -i tests/stdlib/traits_order.n.md -i tests/stdlib/selfhost_req.n.md --no-stdlib --no-tree -o /tmp/tests-vec-result-a.json -j 4`
    - 邨先棡: `10/10 pass`
  - `node nodesrc/tests.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/26_competitive_graph_bfs.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md --no-stdlib --no-tree -o /tmp/tests-vec-result-b2.json -j 4`
    - 邨先棡: `4/4 pass`
  - 陬懷勧遒ｺ隱・
    - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 1` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/23_competitive_sort_and_search.n.md -n 1` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/23_competitive_sort_and_search.n.md -n 2` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -n 1` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -n 2` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/26_competitive_graph_bfs.n.md -n 1` -> pass
    - `node nodesrc/run_doctest.js -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -n 1` -> pass
- 蟾ｮ逡ｰ繝｡繝｢:
  - `Vec` 縺ｮ public API Result 蛹悶・騾ｲ繧薙□縺後�～vec.nepl` 譛ｬ菴薙・ doc comment / doctest 縺ｫ縺ｯ譌ｧ譖ｸ蠑上・譌ｧ pure 蜑肴署縺ｮ隱ｬ譏弱′縺ｾ縺�谿九ｋ縲・
  - `replace` 繧・`set` 縺ｸ謾ｹ蜷阪☆繧区｡医・ parser / keyword 蛻ｶ邏・・蛻・ｊ蛻・￠蠕後↓蜀肴､懆ｨ弱☆繧九�・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (docs(vec): doc comment 縺ｨ doctest 繧・current Result API 縺ｸ蜷梧悄)

- 逶ｮ逧・
  - `Vec` 譛ｬ菴薙ｒ `Result` 蛹悶＠縺溷ｾ後ｂ縲ーstdlib/alloc/collections/vec.nepl](/mnt/d/project/NEPLg2/stdlib/alloc/collections/vec.nepl) 縺ｮ隱ｬ譏弱→蝓九ａ霎ｼ縺ｿ doctest 縺梧立 pure API 蜑肴署縺ｮ縺ｾ縺ｾ谿九▲縺ｦ縺・◆蟾ｮ蛻・ｒ隗｣豸医☆繧九�・
  - 縺ゅｏ縺帙※縲∵立遽�隕句・縺怜ｽ｢蠑上ｒ貂帙ｉ縺励�∵眠縺励＞ doc comment policy 縺ｫ蟇・○繧九�・
- 螟画峩:
  - `vec.nepl`
    - file header 縺ｮ doctest 繧・`unwrap_ok new` 縺ｨ `|> push ... |> uwok` 蜑肴署縺ｸ譖ｴ譁ｰ縲・
    - `new` / `with_capacity` / `len` / `cap` / `data_len` / `is_empty` / `push` / `get` / `replace` / `pop` / `clear` / `free` 縺ｮ comment 萓九ｒ current API 縺ｫ蜷梧悄縲・
    - `is_empty` / `push` / `get` / `replace` / `pop` / `clear` / `free` 縺ｮ遽�隕句・縺励ｒ `### [逶ｮ逧・繧ゅ￥縺ｦ縺江` 蠖｢蠑上∈譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 2` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 3` -> pass

# 2026-03-12 菴懈･ｭ繝｡繝｢ (feat(collections): add bitset)

- 逶ｮ逧・
  - `alloc/collections` 縺ｫ fixed-length 縺ｪ bit 髮・粋繧定ｿｽ蜉�縺励�～BloomFilter` 縺ｨ驕輔▲縺ｦ false positive 縺ｮ縺ｪ縺・membership structure 繧呈ｨ呎ｺ悶〒謇ｱ縺医ｋ繧医≧縺ｫ縺吶ｋ縲・
  - `reboot` 譁ｹ驥昴↓蜷医ｏ縺帙※ bare API 縺ｨ public doctest 繧呈紛縺医�｝ipe 菴ｵ逕ｨ縺ｮ菴ｿ縺・婿縺ｯ `tests/stdlib` 蛛ｴ縺ｧ菫晁ｨｼ縺吶ｋ縲・
- 螟画峩:
  - `stdlib/alloc/collections/bitset.nepl`
    - `BitSet` 繧定ｿｽ蜉�縲・
    - `new` / `len` / `contains` / `insert` / `remove` / `clear` / `fill` / `free` 繧・bare API 縺ｧ螳溯｣・�・
    - 蜀・Κ縺ｯ `nbits` / `nbytes` / `MemPtr<u8>` 繧呈戟縺､ owner struct 縺ｨ縺励�（ndex 縺九ｉ byte offset 縺ｨ bit mask 繧定ｨ育ｮ励＠縺ｦ譖ｴ譁ｰ縺吶ｋ縲・
    - doc comment 縺ｯ譁ｰ policy / format 縺ｸ蜷医ｏ縺帙※縲「sage doctest 繧貞推 public 髢｢謨ｰ縺ｸ霑ｽ蜉�縲・
  - `stdlib/tests/bitset.n.md`
    - insert/remove/len 縺ｨ clear/fill 縺ｮ focused fixture 繧定ｿｽ蜉�縲・
  - `tests/stdlib/bitset_collections.n.md`
    - pipe 險俶ｳ輔〒縺ｮ `insert` / `remove` / `contains` / `fill` 蛻ｩ逕ｨ繧貞屓蟶ｰ縺ｨ縺励※霑ｽ蜉�縲・
- 讀懆ｨｼ:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 2` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 3` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 4` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bitset.nepl -n 5` -> pass

# 2026-03-12 菴懈･ｭ繝｡繝｢ (feat(collections): add adjacency matrix)

- 逶ｮ逧・
  - `alloc/collections` 縺ｫ graph representation 縺ｮ譛�蟆丞ｮ溯｣・→縺励※ `AdjacencyMatrix` 繧定ｿｽ蜉�縺励�∝崋螳夐聞縺ｮ directed edge set 繧・O(1) membership 縺ｧ謇ｱ縺医ｋ繧医≧縺ｫ縺吶ｋ縲・
  - `trie` blocker 縺ｨ迢ｬ遶九↓縲］ested owner 繧帝∩縺代◆ raw bit matrix 縺ｧ collection 縺ｮ遞ｮ鬘槭ｒ蠅励ｄ縺吶�・
- 螟画峩:
  - `stdlib/alloc/collections/adjacency_matrix.nepl`
    - `AdjacencyMatrix` 繧定ｿｽ蜉�縲・
    - `new` / `len` / `contains` / `insert` / `remove` / `clear` / `free` 繧・bare API 縺ｧ螳溯｣・�・
    - `(from, to)` 繧・`from * nverts + to` 縺ｮ bit index 縺ｫ蜀吝ワ縺励�｜yte 驟榊・縺ｧ菫晄戟縺吶ｋ directed graph 縺ｨ縺励◆縲・
    - doc comment 縺ｯ譁ｰ policy / format 縺ｫ蜷医ｏ縺帙�∝推 public 髢｢謨ｰ縺ｫ usage doctest 繧定ｿｽ蜉�縲・
  - `stdlib/tests/adjacency_matrix.n.md`
    - insert/remove/clear 縺ｮ focused fixture 繧定ｿｽ蜉�縲・
  - `tests/stdlib/adjacency_matrix_collections.n.md`
    - pipe 險俶ｳ輔〒縺ｮ `insert` / `remove` / `contains` / `clear` 蛻ｩ逕ｨ繧貞屓蟶ｰ縺ｨ縺励※霑ｽ蜉�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/adjacency_matrix.nepl -i stdlib/tests/adjacency_matrix.n.md -i tests/stdlib/adjacency_matrix_collections.n.md --no-stdlib --no-tree -o /tmp/tests-adjacency-matrix.json -j 2`
    - 邨先棡: `9/9 pass`
- 蟾ｮ逡ｰ繝｡繝｢:
  - `contains g 4 0` 縺ｮ繧医≧縺ｪ遽・峇螟・index 縺ｫ蟇ｾ縺吶ｋ `Result::Err` 邨瑚ｷｯ縺ｯ縲～target/debug/nepl-cli + wasmer` 縺ｧ縺ｯ豁｣蟶ｸ縺ｫ `1` 繧定ｿ斐☆荳�譁ｹ縲『eb compile path 縺ｧ縺ｯ runtime OOB 縺ｫ關ｽ縺｡縺溘�・
  - 縺薙ｌ縺ｯ `AdjacencyMatrix` 螳溯｣・〒縺ｯ縺ｪ縺・web compiler/runtime 蛛ｴ縺ｮ蛻･譬ｹ蝗�縺ｨ蛻､譁ｭ縺励�∽ｻ雁屓縺ｮ collection batch 縺ｫ縺ｯ豺ｷ縺懊※縺・↑縺・�・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (feat(collections): add counting bloom filter)

- 逶ｮ逧・
  - `alloc/collections` 縺ｫ `CountingBloomFilter` 繧定ｿｽ蜉�縺励�～BloomFilter` 縺ｨ蜷後§ hasher 險ｭ險医ｒ菫昴■縺ｪ縺後ｉ蜑企勁蜿ｯ閭ｽ縺ｪ霑台ｼｼ membership structure 繧呈ｨ呎ｺ悶〒謇ｱ縺医ｋ繧医≧縺ｫ縺吶ｋ縲・
  - bare API 縺ｨ public doctest 繧・reboot 譁ｹ驥昴↓蜷医ｏ縺帙�｝ipe 騾｣骼悶・ `tests/stdlib` 蛛ｴ縺ｧ菫晁ｨｼ縺吶ｋ縲・
- 螟画峩:
  - `stdlib/alloc/collections/counting_bloom_filter.nepl`
    - `CountingBloomFilter<.T,.H>` 繧定ｿｽ蜉�縲・
    - `new` / `len` / `insert` / `remove` / `contains` / `clear` / `free` 繧・bare API 縺ｧ螳溯｣・�・
    - counter 縺ｯ `u8` 驟榊・縺ｨ縺励�・ 譛ｬ縺ｮ probe index 縺ｫ蟇ｾ縺励※ insert 縺ｯ鬟ｽ蜥悟刈邂励�〉emove 縺ｯ 0 縺ｾ縺ｧ縺ｮ貂帷ｮ励ｒ陦後≧縲・
  - `stdlib/tests/counting_bloom_filter.n.md`
    - insert/remove/clear 縺ｮ focused fixture 繧定ｿｽ蜉�縲・
  - `tests/stdlib/counting_bloom_filter_collections.n.md`
    - pipe 險俶ｳ輔〒縺ｮ `insert` / `remove` / `contains` / `clear` 蛻ｩ逕ｨ繧貞屓蟶ｰ縺ｨ縺励※霑ｽ蜉�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/tests/counting_bloom_filter.n.md -i tests/stdlib/counting_bloom_filter_collections.n.md -i stdlib/alloc/collections/counting_bloom_filter.nepl --no-stdlib --no-tree -o /tmp/tests-counting-bloom-filter.json -j 2`
    - 邨先棡: `8/8 pass`
- 蟾ｮ逡ｰ繝｡繝｢:
  - `new DefaultHash32 0` 縺ｮ invalid length `Result::Err` 邨瑚ｷｯ縺ｯ縲～target/debug/nepl-cli + wasmer` 縺ｧ縺ｯ豁｣蟶ｸ縺ｫ `1` 繧定ｿ斐☆荳�譁ｹ縲『eb compile path 縺ｧ縺ｯ runtime OOB 縺ｫ關ｽ縺｡縺溘�・
  - 縺薙ｌ縺ｯ `CountingBloomFilter` 螳溯｣・〒縺ｯ縺ｪ縺・web compiler/runtime 蛛ｴ縺ｮ蛻･譬ｹ蝗�縺ｨ蛻､譁ｭ縺励�∽ｻ雁屓縺ｮ collection batch 縺ｫ縺ｯ豺ｷ縺懊※縺・↑縺・�・
  - `node nodesrc/run_doctest.js -i stdlib/tests/bitset.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/bitset.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/bitset_collections.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/bitset.n.md -i tests/stdlib/bitset_collections.n.md -i stdlib/alloc/collections/bitset.nepl --no-stdlib --no-tree -o /tmp/tests-bitset-fixed.json -j 2`
    - 邨先棡: `10/10 pass`
- 蟾ｮ逡ｰ繝｡繝｢:
  - out-of-bounds `Err` 繧定ｿ斐☆ focused case 縺ｯ縲『eb compiler 縺檎函謌舌＠縺・current wasm 縺ｧ hang 縺吶ｋ蛻･譬ｹ蝗�縺ｫ蠖薙◆縺｣縺溘◆繧√�√％縺ｮ batch 縺ｫ縺ｯ豺ｷ縺懊※縺・↑縺・�・
  - `nepl-cli + wasmer` 縺ｧ縺ｯ蜷後§譛�蟆丞・迴ｾ縺悟叉邨ゆｺ・☆繧九％縺ｨ繧堤｢ｺ隱肴ｸ医∩縺ｧ縲《tdlib 螳溯｣・〒縺ｯ縺ｪ縺・compiler/runtime 蛛ｴ縺ｮ蛻･繧ｿ繧ｹ繧ｯ縺ｨ縺励※蛻・ｊ蜃ｺ縺吶�・

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm codegen 蜀・・ precheck 蠕瑚ｨｺ譁ｭ霑泌唆繧帝勁蜴ｻ)

- 逶ｮ逧・
  - `precheck` 螳溯｡悟ｾ後↓ `codegen_llvm` 縺・`TypecheckFailed` 繧定ｿ斐＠縺ｦ縺・◆谿句ｭ倡ｵ瑚ｷｯ繧帝勁蜴ｻ縺励�∝燕谿ｵ讀懈渊荳榊､画擅莉ｶ縺ｸ邨ｱ荳�縺吶ｋ縲・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - `emit_ll_from_module_for_target` 蜀・・ `select_active_raw_body(... )` `Err(diag)` 蛻・ｲ舌ｒ `TypecheckFailed` 霑泌唆縺九ｉ internal panic 縺ｸ螟画峩縲・
    - 縺薙ｌ縺ｫ繧医ｊ縲〉aw-body 驕ｸ謚槫､ｱ謨励・蜑肴ｮｵ `target_precheck::precheck_module_before_codegen` 縺ｧ縺ｮ縺ｿ險ｺ譁ｭ縺輔ｌ縲…odegen 蛻ｰ驕泌ｾ後・逕滓・蟆ゆｻｻ縺ｫ縺ｪ繧九�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-after-llvm-invariant-2.json -j 15` -> `8/8 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-precheck-invariant.json -j 15` -> `791/791 pass`

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm precheck 蝗槫ｸｰ繧ｱ繝ｼ繧ｹ縺ｮ霑ｽ蜉�)

- 逶ｮ逧・
  - LLVM backend 蛻ｰ驕泌燕縺ｫ譛ｪ蟇ｾ蠢・intrinsic 繧定ｨｺ譁ｭ縺ｧ縺阪ｋ縺薙→繧貞屓蟶ｰ蝗ｺ螳壹☆繧九�・
- 螟画峩:
  - `tests/llvm_target.n.md`
    - `llvm_precheck_rejects_wasm_only_intrinsic` 繧定ｿｽ蜉�縲・
    - `#intrinsic "i32_add"` 繧・`#target llvm` 縺ｧ菴ｿ縺｣縺溷�ｴ蜷医↓ `diag_id: 3012` 繧呈悄蠕・☆繧・compile_fail 繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/llvm_target.n.md --no-stdlib --no-tree --runner all --llvm-all -o /tmp/tests-llvm-target-after-precheck-case.json -j 15`
    - 霑ｽ蜉�繧ｱ繝ｼ繧ｹ・・doctest#6::llvm`・峨・ pass縲・
    - 譌｢蟄倥こ繝ｼ繧ｹ `doctest#4/#5` 縺ｯ `invalid redefinition of function 'add'` 縺ｧ fail・域里遏･譛ｪ隗｣豎ｺ・峨�・
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-test-add.json -j 15` -> `791/791 pass`

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: allocator helper 隗｣豎ｺ縺ｮ諢丞袖隲紋ｿｮ豁｣)

- 逶ｮ逧・
  - runtime helper 蜈ｱ騾壼喧蠕後↓逋ｺ逕溘＠縺・run-time 螟ｱ謨・(`unreachable` / `memory access out of bounds`) 繧偵�・俣縺ｫ蜷医ｏ縺帙〒縺ｯ縺ｪ縺・helper 隗｣豎ｺ縺ｮ諢丞袖隲悶°繧我ｿｮ豁｣縺吶ｋ縲・
- 蜴溷屏:
  - `alloc`・亥ｮ牙・API・峨→ `alloc_raw`・井ｽ弱Ξ繝吶ΝAPI・峨・迴ｾ迥ｶ縺ｮ lowering 縺ｧ縺ｯ蝙倶ｺ呈鋤縺ｫ縺ｪ繧翫≧繧九◆繧√�～ALLOC_CANDIDATES=["alloc","alloc_raw"]` 縺ｸ螟画峩縺吶ｋ縺ｨ backend 蜀・Κ遒ｺ菫昴〒隱､縺｣縺ｦ `alloc` 繧呈雫繧�邨瑚ｷｯ縺檎匱逕溘☆繧九�・
  - 縺昴・邨先棡縲∝・驛ｨ遒ｺ菫昴・蜑肴署・育函繝昴う繝ｳ繧ｿ霑泌唆・峨→蜷医ｏ縺壹�∝ｮ溯｡梧凾縺ｫ `unreachable` / OOB 縺檎匱逕溘＠縺溘�・
- 螟画峩:
  - `nepl-core/src/runtime_helpers.rs`
    - `ALLOC_CANDIDATES` 繧・`["alloc_raw", "alloc"]` 縺ｫ謌ｻ縺励�∝・驛ｨ helper 隗｣豎ｺ縺ｯ逕溘・繧､繝ｳ繧ｿ諢丞袖隲悶ｒ蜆ｪ蜈医☆繧九ｈ縺・ｿｮ豁｣縲・
    - 蜊倅ｽ薙ユ繧ｹ繝域悄蠕・�､繧・raw 蜆ｪ蜈医∈譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-alloc-order-fix.json -j 15` -> `791/791 pass`

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: runtime helper 隗｣豎ｺ縺ｮ蜈ｱ騾壼喧縺ｨ raw 萓晏ｭ倡ｸｮ蟆・

- 逶ｮ逧・
  - `nepl-core` 蜀・〒驥崎､・＠縺ｦ縺・◆ runtime helper・・lloc/dealloc/realloc・芽ｧ｣豎ｺ繝ｭ繧ｸ繝・け繧貞・騾壼喧縺励�～_raw` 蜷堺ｾ晏ｭ倥ｒ谿ｵ髫守ｸｮ蟆上☆繧九�・
  - helper 蜷阪・蜆ｪ蜈磯�・ｽ阪ｒ螳牙・API蜷搾ｼ・uffix縺ｪ縺暦ｼ牙━蜈医∈邨ｱ荳�縺吶ｋ縲・
- 螟画峩:
- `nepl-core/src/runtime_helpers.rs`
    - `ALLOC_CANDIDATES` 繧・`["alloc", "alloc_raw"]` 縺ｫ螟画峩・亥ｮ牙・API蜆ｪ蜈茨ｼ峨�・
    - `RuntimeHelperKind` / `helper_candidates` / `helper_base_name` 繧定ｿｽ蜉�縲・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (trait 閭ｽ蜉帙Δ繝・Ν: `Eq` / `Ord` 縺ｮ蜈ｱ騾壼喧)

- 逶ｮ逧・
  - `core/traits` 縺ｫ `Eq` / `Ord` 繧定ｿｽ蜉�縺励�∵ｯ碑ｼ・э蜻ｳ隲悶ｒ stdlib 蜈ｱ騾・trait 縺ｨ縺励※謇ｱ縺医ｋ繧医≧縺ｫ縺吶ｋ縲・
  - `alloc/collections/vec/sort.nepl` 縺ｮ螻�謇� `Ord` 螳夂ｾｩ繧呈彫蜴ｻ縺励�…ollections 蛛ｴ縺ｮ豈碑ｼ・capability 繧・`core` 縺ｸ蟇・○繧九�・
- 螟画峩:
  - `stdlib/core/traits/eq.nepl`
    - `Eq` trait
    - `eq_by_trait`
    - `ne_by_trait`
    - `bool`, `i32`, `u8`, `i64`, `f32`, `f64`, `str` 縺ｸ縺ｮ impl
  - `stdlib/core/traits/ord.nepl`
    - `Ord` trait
    - `ord_lt`, `ord_le`, `ord_gt`, `ord_ge`
    - `bool`, `i32`, `u8`, `i64`, `i128`, `f32`, `f64` 縺ｸ縺ｮ impl
  - `stdlib/alloc/collections/vec/sort.nepl`
    - 螻�謇� `Ord` trait 縺ｨ螻�謇� impl 繧貞炎髯､
    - `core/traits/ord` 繧・import 縺励�～sort_lt` 邉ｻ helper 縺九ｉ蜈ｱ騾・`ord_*` 繧貞他縺ｶ蠖｢縺ｸ螟画峩
  - `tests/stdlib/traits_order.n.md`
    - 譌･譛ｬ隱槭・逶ｮ逧・▽縺・focused test 繧定ｿｽ蜉�
- 蛻､譁ｭ:
  - `Eq<i128>` 縺ｯ譌｢蟄倥・蛻・ｧ｣ helper 繧剃ｻｮ螳壹☆繧九→螢翫ｌ繧九◆繧√�∽ｸ�譌ｦ霑ｽ蜉�縺励↑縺九▲縺溘�・
  - `Ord<str>` 繧よ里蟄倥・鬆・ｺ乗ｯ碑ｼ・helper 縺梧悴謨ｴ蛯吶↑縺ｮ縺ｧ縲∝酔讒倥↓隕矩�√▲縺溘�・
  - 縺ｾ縺壹・譌｢蟄倥・ `core/math` overload 縺ｧ譬ｹ諡�繧呈戟縺ｦ繧句梛縺�縺代ｒ蜈ｱ騾・trait 蛹悶＠縺溘�・
- 讀懆ｨｼ:
  - `NODE_NO_WARNINGS=1 node nodesrc/run_test.js`
    - `Eq` / `Ord` core focused case: pass
    - `vec/sort` + `Ord` std focused case: pass

# 2026-03-09 菴懈･ｭ繝｡繝｢ (trait 閭ｽ蜉帙Δ繝・Ν: `Hash` 縺ｮ蜈ｱ騾壼喧)

- 逶ｮ逧・
  - `Hash` trait 繧・`core/traits` 縺ｸ霑ｽ蜉�縺励�”ashmap / hashset 縺悟・菴鍋噪縺ｪ `hash32_i32` / `hash32_str` 縺ｸ逶ｴ謗･萓晏ｭ倥○縺壼・騾・helper 邨檎罰縺ｧ繧ｭ繝ｼ繧呈ｷｷ蜷医〒縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
  - 蟆・擂縺ｮ `Serialize` / `Deserialize` 縺ｨ蜷後§縺上�∝梛縺斐→縺ｮ閭ｽ蜉帙ｒ stdlib trait 縺ｨ縺励※譏守､ｺ縺吶ｋ豬√ｌ繧呈純縺医ｋ縲・
- 螟画峩:
  - `stdlib/core/traits/hash.nepl`
  - `Hash` trait
  - `hash32_by_trait`
  - `i32`

# 2026-03-11 菴懈･ｭ繝｡繝｢ (`streamio` target 謖・ｮ壼喧縺ｨ `u32/u64` bare I/O 縺ｮ菫ｮ豁｣)

- 逶ｮ逧・
  - `scanner` / `writer` 繧・stdin/stdout 蝗ｺ螳壹・ no-arg API 縺九ｉ螟悶＠縲～io_stdin` / `io_stdout` / `io_text` / `io_bytes` 縺ｮ target 謖・ｮ壹〒逕滓・縺吶ｋ蠖｢縺ｸ蟇・○繧九�・
  - `u32` / `u64` 縺ｮ bare `read` / `write` 繧偵�∝梛 suffix 蜷阪↓謌ｻ縺輔★ current overload 譁ｹ驥昴・縺ｾ縺ｾ螳牙ｮ壼喧縺吶ｋ縲・
  - Part6 tutorial 縺ｨ `kp` 蜻ｨ霎ｺ縺ｫ谿九▲縺ｦ縺・◆ old move-model 蜑肴署繧偵�∫樟陦梧園譛画ｨｩ繝｢繝・Ν縺ｸ蜷医ｏ縺帙ｋ縲・
- 蜴溷屏:
  - `std/streamio` 縺�縺・`read` / `write` 縺ｮ bare 蜷阪∈蟇・○縺ｦ繧ゅ�∫函謌仙・蜿｣ `scanner()` / `writer()` 縺・stdin/stdout 蝗ｺ螳壹・縺ｾ縺ｾ縺�縺ｨ縲～std/io` / `iotarget` 縺ｨ雋ｬ蜍吶′莠碁㍾蛹悶＠縺ｦ縺・◆縲・
  - `u64` 縺ｯ compiler 蛛ｴ縺ｧ `wasm_shared::valtype` 縺後∪縺� `i32` 謇ｱ縺・・邂・園繧呈ｮ九＠縺ｦ縺翫ｊ縲仝asm signature 縺悟ｴｩ繧後※縺・◆縲・
  - `u32` / `u64` 縺ｮ 10 騾ｲ蜃ｺ蜉帙・縲「nsigned 蛟､繧・signed overload 縺ｸ關ｽ縺ｨ縺励※縺・◆縺溘ａ `4294967295` 縺・`18446744073709551615` 縺ｫ蛹悶￠縺ｦ縺・◆縲・
  - `PrefixI32` 繧・tutorial Part6 縺ｮ `Vec` 襍ｰ譟ｻ縺ｫ縺ｯ old move-model 蜑肴署縺梧ｮ九▲縺ｦ縺・◆縲・
- 螟画峩:
  - `stdlib/std/streamio.nepl`
    - `scanner <(IoReadTarget)*>Result<StreamScanner,str>>`
    - `writer <(IoWriteTarget)*>Result<StreamWriter,str>>`
    - `scanner_from_bytes`
    - `StreamWriter` header 縺ｫ `TargetKind` 繧定ｿｽ蜉�
    - `u32` / `u64` 縺ｮ append 螳溯｣・ｒ unsigned decimal 縺ｨ縺励※菫ｮ豁｣
    - `StreamScanner` / `StreamWriter` 縺ｮ doc comment 繧・current 螳溯｣・∈蜷梧悄
  - `stdlib/std/iotarget.nepl`
    - `io_stdin` / `io_stdout` / `io_text` / `io_bytes` 繧堤函謌仙・蜿｣縺ｨ縺励※蛻ｩ逕ｨ
  - `nepl-core/src/wasm_shared.rs`
    - `u64` 繧・Wasm `I64` 縺ｨ縺励※謇ｱ縺・ｈ縺・ｿｮ豁｣
  - `nodesrc/run_test.js`
    - `BigInt` 縺ｮ JSON 蜃ｺ蜉帙→ return decode 繧定ｿｽ蜉�
  - `stdlib/kp/kpprefix.nepl`
    - `PrefixI32` 縺ｫ `Copy` / `Clone` 繧剃ｻ倅ｸ・
    - `prefix_build_vec_i32` 繧・`vec_data_len` 繝吶・繧ｹ縺ｸ菫ｮ豁｣
  - `tests/stdlib/streamio.n.md`
  - `tests/stdlib/kp.n.md`
  - `tests/stdlib/kp_i64.n.md`
  - `tests/stdlib/stdin.n.md`
  - `tests/compiler/move_effect.n.md`
  - `tutorials/getting_started/22_competitive_io_and_arith.n.md`
  - `tutorials/getting_started/24_competitive_dp_basics.n.md`
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
  - `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
  - `stdlib/kp/kpgraph.nepl`
    - `unwrap_ok scanner io_stdin` / `unwrap_ok writer io_stdout` 縺ｸ邨ｱ荳�
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`
  - `node nodesrc/run_doctest.js -i /tmp/u64_probe2.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 4`
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 10`
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp_i64.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 2`
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 3`
  - `node nodesrc/run_doctest.js -i tests/stdlib/stdin.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/22_competitive_io_and_arith.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/24_competitive_dp_basics.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -n 1`
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -n 1`
  - `node nodesrc/run_doctest.js -i stdlib/kp/kpgraph.nepl -n 1`
  - `node nodesrc/run_doctest.js -i stdlib/kp/kpprefix.nepl -n 1`
  - `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 20`

# 2026-03-09 菴懈･ｭ繝｡繝｢ (compiler 蜑肴署蝗ｺ螳・ `#prelude` 譛�蟆丞ｮ溯｣・→ Copy 蝗ｺ螳夊｡ｨ謦､蜴ｻ)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `todo.md` 縺ｮ `compiler 蜑肴署` 谿倶ｻｶ縺�縺｣縺・`Copy` 蝗ｺ螳夊｡ｨ萓晏ｭ倥ｒ縲ー螳滄圀/縺倥▲縺輔＞]縺ｫ source [蛛ｴ/縺後ｏ]縺九ｉ trait impl 繧端萓帷ｵｦ/縺阪ｇ縺・″繧・≧]縺ｧ縺阪ｋ[迥ｶ諷・縺倥ｇ縺・◆縺Ь縺ｸ[遘ｻ/縺・▽]縺吶�・
  - parser 縺�縺代↓[蟄伜惠/縺昴ｓ縺悶＞]縺励※縺・◆ `#prelude` / `#no_prelude` 繧・loader [谿ｵ髫・縺�繧薙°縺Ь縺ｧ繧・隗｣驥・縺九＞縺励ｃ縺従縺励�…opy/clone 髱槭ワ繝ｼ繝峨さ繝ｼ繝牙喧縺ｮ[蜑肴署/縺懊ｓ縺ｦ縺Ь繧端謨ｴ/縺ｨ縺ｨ縺ｮ]縺医ｋ縲・
- [蜴溷屏/縺偵ｓ縺・ｓ]:
  - `#prelude` 縺ｨ `#no_prelude` 縺ｯ lexer / parser / AST 縺ｫ縺�縺措蟄伜惠/縺昴ｓ縺悶＞]縺励�〕oader 縺ｧ縺ｯ[辟｡隕・繧�縺余縺輔ｌ縺ｦ縺・◆縲・
  - 縺昴・縺溘ａ `Copy` / `Clone` impl 繧・source [蛛ｴ/縺後ｏ]縺九ｉ[譌｢螳・縺阪※縺Ь[萓帷ｵｦ/縺阪ｇ縺・″繧・≧]縺ｧ縺阪★縲～TypeCtx::is_copy` 縺ｫ primitive 蝗ｺ螳夊｡ｨ繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ繧端谿・縺ｮ縺転縺兌蠢・ｦ・縺ｲ縺､繧医≧]縺後≠縺｣縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nepl-core/src/loader.rs`
    - root module [髯仙ｮ・縺偵ｓ縺ｦ縺Ь縺ｧ `#prelude` / `#no_prelude` 繧端蜃ｦ逅・縺励ｇ繧馨縺吶ｋ繧医≧縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `#no_prelude` 縺後↑縺・root module 縺ｫ縺ｯ[譌｢螳・縺阪※縺Ь縺ｧ `std/prelude_base` 繧端隱ｭ/繧・縺ｿ[霎ｼ/縺転繧�縲・
    - import/include 縺ｮ[蜀榊ｸｰ/縺輔＞縺江 load 縺ｧ縺ｯ default prelude 繧端驕ｩ逕ｨ/縺ｦ縺阪ｈ縺・縺励↑縺・ｈ縺・↓縺励※縲《tdlib [蜀・Κ/縺ｪ縺・・] import 縺ｧ縺ｮ[蠕ｪ迺ｰ/縺倥ｅ繧薙°繧転繧端驕ｿ/縺評縺代◆縲・
  - `stdlib/std/prelude_base.nepl`
    - [譛�蟆・縺輔＞縺励ｇ縺・ prelude 縺ｨ縺励※[霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - [蠖馴擇/縺ｨ縺・ａ繧転縺ｯ `core/traits/copy` 縺�縺代ｒ[隱ｭ/繧・縺ｿ[霎ｼ/縺転縺ｿ縲…opy/clone 閭ｽ蜉帙・ source [萓帷ｵｦ/縺阪ｇ縺・″繧・≧]縺ｫ[邨・縺励⊂]縺｣縺溘�・
  - `nepl-core/src/types.rs`
    - `TypeCtx::is_copy` 縺ｮ譛�邨ゅヵ繧ｩ繝ｼ繝ｫ繝舌ャ繧ｯ縺九ｉ primitive 蝗ｺ螳夊｡ｨ繧端蜑企勁/縺輔￥縺倥ｇ]縺励◆縲・
    - `Copy` trait 縺啓隕・縺ｿ]縺医※縺・↑縺Ъ蝣ｴ蜷・縺ｰ縺ゅ＞]縺ｯ縲ー蜿ら・/縺輔ｓ縺励ｇ縺・蝙九→ `Never` 縺�縺代ｒ compiler [蜀・惠/縺ｪ縺・＊縺Ь縺ｮ copy 縺ｨ縺励※[謇ｱ/縺ゅ▽縺犠縺・�・
  - `tests/compiler/prelude_copy.n.md`
    - default prelude 縺ｧ `Copy` bound 縺啓騾・縺ｨ縺馨繧九％縺ｨ繧端遒ｺ隱・縺九￥縺ｫ繧転縺吶ｋ focused case 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `#prelude std/prelude_base` 縺ｨ `#no_prelude` 繧端菴ｵ險・縺ｸ縺・″]縺励※繧ゅ�ー譏守､ｺ逧・繧√＞縺倥※縺江 prelude 縺啓蜆ｪ蜈・繧・≧縺帙ｓ]縺輔ｌ繧九％縺ｨ繧端蝗ｺ螳・縺薙※縺Ь縺励◆縲・
    - `#no_prelude` 縺�縺代〒縺ｯ `Copy` trait [萓帷ｵｦ/縺阪ｇ縺・″繧・≧]縺啓豸・縺江縺医�～.T: Copy` 縺・`3073` 縺ｧ[關ｽ/縺馨縺｡繧九％縺ｨ繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/compiler/prelude_copy.n.md -i tests/compiler/resolve.n.md --no-stdlib --no-tree -o /tmp/tests-prelude-only.json -j 15` -> `14/14 pass`
  - `node nodesrc/tests.js -i tests/compiler/prelude_copy.n.md --no-stdlib --no-tree -o /tmp/tests-prelude-copy-only.json -j 15` -> `3/3 pass`
- [蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `Copy` 縺ｮ source [萓帷ｵｦ/縺阪ｇ縺・″繧・≧]縺ｯ default prelude 繧端騾・縺ｨ縺馨縺吶％縺ｨ縺ｧ[譌｢蟄・縺阪◇繧転繧ｳ繝ｼ繝峨ｒ[螢・縺薙ｏ]縺輔★縺ｫ[遘ｻ陦・縺・％縺・縺ｧ縺阪ｋ縲・
  - `#no_prelude` 縺ｯ縲梧ｨ呎ｺ・capability 繧端蜷ｫ/縺ｵ縺従繧√※閾ｪ蜑阪〒[邂｡逅・縺九ｓ繧馨縺吶ｋ縲阪◆繧√・ opt-out 縺ｨ縺励※[讖溯・/縺阪・縺・縺吶ｋ縲・
    - `bool`
    - `u8`
    - `i64`
    - `str`
    縺ｸ縺ｮ impl 繧定ｿｽ蜉�縲・
  - `stdlib/alloc/collections/hashmap.nepl`
    - `hash32_i32` / `hash32_str` 縺ｮ逶ｴ謗･蜻ｼ縺ｳ蜃ｺ縺励ｒ `hash32_by_trait` 縺ｫ鄂ｮ謠帙�・
  - `stdlib/alloc/collections/hashset.nepl`
    - 蜷梧ｧ倥↓ `hash32_by_trait` 邨檎罰縺ｸ鄂ｮ謠帙�・
  - `tests/stdlib/traits_hash.n.md`
    - `[逶ｮ逧・繧ゅ￥縺ｦ縺江` 縺､縺・focused case 繧定ｿｽ蜉�縲・
- 蛻､譁ｭ:
  - `Hash<i64>` 縺ｯ [荳贋ｽ・縺倥ｇ縺・＞] / [荳倶ｽ・縺九＞] 32-bit 繧・XOR 縺ｧ謚倥ｊ縺溘◆繧薙〒縺九ｉ `hash32_i32` 縺ｸ豬√☆縲・
  - `Hash` 縺ｮ蟇ｾ雎｡縺ｯ縲√∪縺壽里蟄・stdlib 縺悟ｮ牙ｮ壹＠縺ｦ謾ｯ縺医※縺・ｋ繧ｭ繝ｼ蝙九↓髯仙ｮ壹＠縺溘�・
  - `i128` 繧・峡閾ｪ讒矩��菴薙・繝上ャ繧ｷ繝･閭ｽ蜉帙・縲∽ｻ雁ｾ・`Serialize` / `Eq` 縺ｨ縺ｮ謨ｴ蜷医ｒ隕九↑縺後ｉ霑ｽ蜉�縺吶ｋ縲・
- compiler 菫ｮ豁｣:
  - 縺ｪ縺励�ゆｻ雁屓縺ｮ遒ｺ隱阪〒隕九▽縺九▲縺溷撫鬘後・ `traits_hash.n.md` 蛛ｴ縺ｮ API 繧ｵ繝ｳ繝励Ν縺檎樟陦・`hashmap` / `hashset` 縺ｮ蛻ｩ逕ｨ豬∝о縺ｨ縺壹ｌ縺ｦ縺・◆縺薙→縺�縺｣縺溘�・
  - `must_hm` / `must_hs` 縺ｨ `Option` 縺ｮ match 繧剃ｽｿ縺・里蟄俶ｵ∝о縺ｸ蜷医ｏ縺帙※菫ｮ豁｣縺励◆縲・
- 讀懆ｨｼ:
  - `node` + `nodesrc/compiler_loader` 縺ｫ繧医ｋ compile-only focused check 縺ｧ縲・
    - `hash32_by_trait` 蜊倅ｽ・
    - `hashmap/hashset/hashmap_str/hashset_str`
    繧剃ｽｿ縺・snippet
    縺ｮ荳｡譁ｹ縺・`COMPILE_OK` 繧定ｿ斐☆縺薙→繧堤｢ｺ隱阪�・
  - `nodesrc/tests.js` 縺ｯ縺薙・迺ｰ蠅・〒縺ｯ髟ｷ縺上・繧我ｸ九′繧九％縺ｨ縺後≠繧九◆繧√�’ocused 縺ｪ compile-only 縺ｧ縺ｾ縺壼ｦ･蠖捺�ｧ繧貞崋螳壹＠縺溘�・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (`std/test` 髮・ｴ・API 霑ｽ蜉�縺ｨ nested generic overload 譬ｹ譛ｬ菫ｮ豁｣)

- 逶ｮ逧・
  - stdlib reboot 蜑阪・繝・せ繝亥渕逶､縺ｨ縺励※縲・ 莉ｶ螟ｱ謨励＠縺ｦ繧よｮ九ｊ縺ｮ讀懈渊繧堤ｶ咏ｶ壼ｮ溯｡後〒縺阪ｋ `std/test` 縺ｮ collectable API 繧呈紛蛯吶☆繧九�・
  - `Vec<Result<(),str>>` 縺ｫ `push` / `vec_push` / pipe 縺ｧ `Result<(),str>` 繧堤ｩ阪ａ縺ｪ縺・compiler 繝舌げ繧偵�〕ibrary 蛛ｴ縺ｮ蝗樣∩縺ｧ縺ｯ縺ｪ縺・typecheck 縺ｮ譬ｹ譛ｬ蜴溷屏縺九ｉ菫ｮ豁｣縺吶ｋ縲・
- 螟画峩:
  - `stdlib/std/test.nepl`
    - `checks_new`
    - `checks_push`
    - `check`
    - `check_eq_i32`
    - `check_ne`
    - `check_str_eq`
    - `check_ok_i32`
    - `check_err_i32`
    - `check_status_str`
    - `checks_has_err(_loop)`
    - `checks_summary(_loop)`
    - `checks_report_failures`
    - `finish_checks`
    繧定ｿｽ蜉�縺励◆縲・
    - `assert` / `assert_eq_i32` / `assert_ne` / `assert_str_eq` 縺ｯ縲∝ｯｾ蠢懊☆繧・`check_*` 繧貞女縺代※蜊ｳ譎ょ､ｱ謨励☆繧玖埋縺・Λ繝・ヱ縺ｸ謨ｴ逅・＠縺溘�・
  - `tests/std_test_collect.n.md`
    - `[逶ｮ逧・繧ゅ￥縺ｦ縺江` 縺ｨ `[菴・縺ｪ縺ｫ]繧端遒ｺ/縺溘＠]縺九ａ繧九°` 繧剃ｻ倥￠縺・focused case 繧定ｿｽ蜉�縺励◆縲・
    - 蜈ｨ莉ｶ謌仙粥譎ゅ・ summary 蜃ｺ蜉帙→縲∝､ｱ謨励ｒ蜷ｫ繧�縺ｨ縺阪・ summary + 蛟句挨螟ｱ謨怜・蜉帙ｒ蝗ｺ螳壹＠縺溘�・
  - `tests/compiler/overload_nested_generic_push.n.md`
    - `Vec<Result<(),str>>` 縺ｫ蟇ｾ縺吶ｋ `push` / `vec_push` / pipe 縺ｮ nested generic overload 隗｣豎ｺ繧堤｢ｺ隱阪☆繧・compiler 蝗槫ｸｰ test 繧定ｿｽ蜉�縺励◆縲・
  - `nepl-core/src/types.rs`
    - 髢｢謨ｰ蝙九↓蜷ｫ縺ｾ繧後ｋ蝙句､画焚 binding 繧帝��驕ｿ繝ｻ蠕ｩ蜈・☆繧・
      - `snapshot_type_var_bindings`
      - `restore_type_var_bindings`
      繧定ｿｽ蜉�縺励◆縲・
  - `nepl-core/src/typecheck.rs`
    - `check_function` 縺ｧ髢｢謨ｰ譛ｬ菴薙ｒ讀懈渊縺吶ｋ蜑阪↓ `func_ty` 荳翫・蝙句､画焚 binding 繧・snapshot 縺励�∫ｵゆｺ・ｾ後↓蠢・★ restore 縺吶ｋ繧医≧螟画峩縺励◆縲・
- 蜴溷屏:
  - generic 髢｢謨ｰ譛ｬ菴薙・蝙区､懈渊荳ｭ縺ｫ縲・未謨ｰ繧ｷ繧ｰ繝阪メ繝｣閾ｪ菴薙′謖√▲縺ｦ縺・ｋ蝙句､画焚 `TypeId` 縺・unification 縺ｧ譚溽ｸ帙＆繧後�√◎縺ｮ譚溽ｸ帙′ `Env` 荳翫・螟ｧ蝓滄未謨ｰ蝙九∈谿狗蕗縺励※縺・◆縲・
  - 縺昴・邨先棡縲～vec_push <.T> <(Vec<.T>, .T)->Vec<.T>>` 縺ｮ `.T` 縺碁℃蜴ｻ縺ｮ讀懈渊縺ｧ `i32` 縺ｸ豎壽沒縺輔ｌ縲～Vec<Result<(),str>>` 縺ｫ蟇ｾ縺吶ｋ overload 謗ｨ隲悶〒 `Vec<i32>` 縺ｨ縺励※謇ｱ繧上ｌ縺ｦ縺・◆縲・
  - 譏守､ｺ蝙句ｼ墓焚莉倥″ `vec_push<Result<(),str>>` 縺碁�壹ｊ縲∝梛蠑墓焚逵∫払譎ゅ□縺題誠縺｡繧九％縺ｨ縺九ｉ縲…andidate 驕ｸ謚樊凾縺ｮ `instantiate(binding.ty)` 蜈･蜉帙′譌｢縺ｫ豎壽沒縺輔ｌ縺ｦ縺・ｋ縺ｨ迚ｹ螳壹＠縺溘�・
- 邨先棡:
  - `std/test` 縺ｮ collectable API 縺ｧ縲～[ok,ok,err,ok,err]` 蠖｢蠑上・讎りｦ√→螟ｱ謨玲ｷｻ蟄励・逅・罰繧偵∪縺ｨ繧√※陦ｨ遉ｺ縺ｧ縺阪ｋ繧医≧縺ｫ縺ｪ縺｣縺溘�・
  - nested generic `push` / `vec_push` / pipe 縺ｯ縲∝梛蠑墓焚繧呈・遉ｺ縺励↑縺上※繧・`Vec<Result<(),str>>` 荳翫〒隗｣豎ｺ縺ｧ縺阪ｋ繧医≧縺ｫ縺ｪ縺｣縺溘�・
- 讀懆ｨｼ:
  - `trunk build`・・oot, `NO_COLOR=false`・・-> success
  - `node nodesrc/tests.js -i tests/std_test_collect.n.md -i tests/compiler/overload_nested_generic_push.n.md --no-stdlib --no-tree -o /tmp/tests-std-test-collect-focused.json -j 15`
    - 邨先棡: `5/5 pass`
    - `find_runtime_helper_key`・亥錐蜑崎ｧ｣豎ｺ・峨→ `find_runtime_helper_index`・・ndex隗｣豎ｺ・峨ｒ霑ｽ蜉�縲・
  - `nepl-core/src/codegen_wasm.rs`
    - 繝ｭ繝ｼ繧ｫ繝ｫ螳溯｣・□縺｣縺・helper 蜷崎ｧ｣豎ｺ繧貞炎髯､縺励�～runtime_helpers::find_runtime_helper_index` 縺ｫ邨ｱ荳�縲・
  - `nepl-core/src/monomorphize.rs`
    - helper 菫晄戟繝ｫ繝ｼ繝域爾邏｢繧・`find_runtime_helper_key` + `RuntimeHelperKind` 縺ｸ鄂ｮ謠帙�・
    - 驥崎､・＠縺ｦ縺・◆蜷榊燕繝槭ャ繝・未謨ｰ繧貞炎髯､縲・
  - `nepl-core/src/codegen_llvm.rs`
    - helper 蛟呵｣懷叙蠕励ｒ `helper_candidates(RuntimeHelperKind::...)` 縺ｫ邨ｱ荳�縲・
    - `resolve_symbol_name` 縺ｮ蛟呵｣應ｸ�閾ｴ繧・`helper_base_name` 繝吶・繧ｹ縺ｸ螟画峩縺励�］amespaced/mangled 蜷阪〒繧ょ酔荳�隕丞援縺ｧ隗｣豎ｺ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-helper-unify.json -j 15` -> `791/791 pass`

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm backend 縺ｮ wasm-body 蛻・ｲ舌ｒ荳榊､画擅莉ｶ蛹・

- 逶ｮ逧・
  - `codegen_llvm` 蛛ｴ縺ｫ谿九▲縺ｦ縺・◆ backend 蜈･蜉帙お繝ｩ繝ｼ蛻・ｲ撰ｼ・UnsupportedWasmBody`・峨ｒ蜑肴ｮｵ讀懈渊蜑肴署縺ｸ蟇・○繧九�・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - `LlvmCodegenError` 縺九ｉ `UnsupportedWasmBody` / `UnsupportedParsedFunctionBody` 繧貞炎髯､縲・
    - `emit_ll_from_module_for_target` 蜀・〒 `ActiveRawBody::Wasm` 蛻ｰ驕疲凾縺ｮ `Err` 繧・internal panic 縺ｫ螟画峩縲・
    - `FnBody::Wasm` reachable 蛻ｰ驕疲凾縺ｮ `Err` 繧・internal panic 縺ｫ螟画峩縲・
    - HIR lowering 邨瑚ｷｯ縺ｧ `HirBody::Wasm` 蛻ｰ驕疲凾縺ｮ `Err` 繧・internal panic 縺ｫ螟画峩縲・
    - 蟇ｾ蠢懊ユ繧ｹ繝・`emit_ll_rejects_entry_with_wasm_body` 縺ｯ `TypecheckFailed` 繧呈悄蠕・☆繧句ｽ｢縺ｸ譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-invariant.json -j 15` -> `791/791 pass`

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: wasm codegen 險ｺ譁ｭ霑泌唆邨瑚ｷｯ縺ｮ謦､蜴ｻ)

- 逶ｮ逧・
  - `codegen` 蛻ｰ驕泌ｾ後・逕滓・蟆ゆｻｻ縺ｫ縺吶ｋ譁ｹ驥昴↓蜷医ｏ縺帙�～codegen_wasm` 縺ｮ `Vec<Diagnostic>` 霑泌唆邨瑚ｷｯ繧呈彫蜴ｻ縺吶ｋ縲・
- 螟画峩:
  - `nepl-core/src/codegen_wasm.rs`
    - `lower_body` / `lower_user` 縺ｮ謌ｻ繧雁�､繧・`Result<Function, Vec<Diagnostic>>` 縺九ｉ `Function` 縺ｸ螟画峩縲・
    - `gen_block` / `gen_expr` 縺ｮ `diags` 蠑墓焚繧貞炎髯､縲・
    - `generate_wasm` 縺ｮ code section 逕滓・縺ｧ `Err(ds)` 蛻・ｲ舌ｒ蜑企勁縺励�∝燕谿ｵ讀懈渊騾夐℃蠕後・逶ｴ謗･逕滓・縺吶ｋ蠖｢縺ｫ邨ｱ荳�縲・
    - backend 蜀・ｨｺ譁ｭ縺ｨ縺励※谿九▲縺ｦ縺・◆譛ｪ菴ｿ逕ｨ髢｢謨ｰ `validate_wasm_stack` 繧貞炎髯､縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-after-wasm-no-diag.json -j 15` -> `8/8 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-wasm-no-diag.json -j 15` -> `791/791 pass`

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: wasm helper 隗｣豎ｺ縺ｮ閾ｪ蟾ｱ蜀榊ｸｰ繝舌げ菫ｮ豁｣)

- 逶ｮ逧・
  - `tests + stdlib` 縺ｧ逋ｺ逕溘＠縺ｦ縺・◆ `RangeError: Maximum call stack size exceeded` 繧呈�ｹ譛ｬ蜴溷屏縺九ｉ隗｣豸医☆繧九�・
- 蜀咲樟縺ｨ蛻・ｊ蛻・￠:
  - `option.nepl` doctest 繧貞腰迢ｬ蜀咲樟縺吶ｋ縺ｨ `wasm-function[4]` 縺ｮ閾ｪ蟾ｱ蜀榊ｸｰ縺ｧ蛛懈ｭ｢縲・
  - 蜷御ｸ�繧ｽ繝ｼ繧ｹ繧・`nepl-cli` 縺ｧ逕滓・縺励◆ wasm 縺ｯ豁｣蟶ｸ螳溯｡後�・
  - `web` 逕滓・ WAT 縺ｨ `native` 逕滓・ WAT 繧呈ｯ碑ｼ・☆繧九→縲∝酔荳�邂・園縺ｧ `call 5` 縺・`call 4`・郁・蟾ｱ蜻ｼ縺ｳ蜃ｺ縺暦ｼ峨↓蛹悶￠縺ｦ縺・◆縲・
- 蜴溷屏:
  - `codegen_wasm` 縺ｮ runtime helper 隗｣豎ｺ縺梧尠譏ｧ縺ｪ譁・ｭ怜・荳�閾ｴ・・refix/contains・我ｾ晏ｭ倥□縺｣縺溘�・
  - allocator helper 隗｣豎ｺ譎ゅ↓ `alloc` 縺ｨ `alloc_raw` 縺ｮ蜿悶ｊ驕輔∴縺檎匱逕溘＠縲‘num/tuple 讒狗ｯ画凾縺ｮ蜀・Κ遒ｺ菫昴〒閾ｪ蟾ｱ蜀榊ｸｰ縺瑚ｵｷ縺阪※縺・◆縲・
- 螟画峩:
  - `nepl-core/src/codegen_wasm.rs`
    - helper 蜷阪・蝓ｺ蠎募錐謚ｽ蜃ｺ `helper_base_name` 繧定ｿｽ蜉�縲・
    - runtime helper 隗｣豎ｺ繧貞渕蠎募錐荳�閾ｴ縺ｸ螟画峩縺励�∵尠譏ｧ荳�閾ｴ繧貞ｻ・ｭ｢縲・
    - 迴ｾ蝨ｨ lowering 荳ｭ縺ｮ髢｢謨ｰ繧､繝ｳ繝・ャ繧ｯ繧ｹ縺ｯ helper 蛟呵｣懊°繧蛾勁螟悶�・
    - `LocalMap` 縺ｫ `alloc_helper_idx` 繧剃ｿ晄戟縺励�・未謨ｰ縺斐→縺ｫ荳�蠎ｦ縺�縺・helper 繧堤｢ｺ螳壹�・
  - `nepl-core/src/runtime_helpers.rs`
    - `ALLOC_CANDIDATES` 繧・`["alloc_raw", "alloc"]` 縺ｮ鬆・∈螟画峩縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i stdlib/core/option.nepl -i stdlib/alloc/collections/vec.nepl -i stdlib/alloc/collections/vec/sort.nepl --no-stdlib --no-tree -o /tmp/tests-vec-option-after-alloc-helper-fix.json -j 15` -> `22/22 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-alloc-helper-fix.json -j 15` -> `791/791 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: web 螳溯｡梧凾 `compile: unreachable` 縺ｮ譬ｹ譛ｬ菫ｮ豁｣)

- 逶ｮ逧・
  - `web/dist` 邨瑚ｷｯ縺ｧ縺ｮ縺ｿ逋ｺ逕溘＠縺ｦ縺・◆ `phase=compile, error=unreachable` 繧呈�ｹ譛ｬ蜴溷屏縺九ｉ隗｣豸医☆繧九�・
- 蜴溷屏:
  - `codegen_wasm.rs` 縺ｮ raw wasm 陦後ヱ繝ｼ繧ｹ縺ｧ縲√Ο繝ｼ繧ｫ繝ｫ隗｣豎ｺ繧ｯ繝ｭ繝ｼ繧ｸ繝｣縺・`parse_wasm_line_with_lookup` 蛛ｴ縺ｮ `$` 豁｣隕丞喧縺ｨ莠碁㍾蜃ｦ逅・↓縺ｪ縺｣縺ｦ縺・◆縲・
  - 縺昴・邨先棡縲～#wasm` 譛ｬ譁・・ `$a`/`$b` 縺・codegen 譎ゅ・縺ｿ `unknown local` 縺ｫ縺ｪ繧・panic 縺励※縺・◆・・recheck 蛛ｴ縺ｨ縺ｯ荳肴紛蜷茨ｼ峨�・
- 螟画峩:
  - `nepl-core/src/codegen_wasm.rs`
    - `parse_wasm_line` 縺ｮ lookup 繧・`|name| locals.lookup(name)` 縺ｫ邨ｱ荳�縲・
    - 譌ｧ `parse_local` 繝倥Ν繝代ｒ蜑企勁縲・
  - `nepl-web/src/lib.rs`
    - `console_error_panic_hook::set_once()` 繧・`#[wasm_bindgen(start)]` 縺ｧ譛牙柑蛹悶＠縲仝ASM panic 縺ｮ蜴溷屏菴咲ｽｮ繧貞庄隕門喧縲・
  - `nodesrc/run_test.js`
    - `formatError` 繧定ｿｽ蜉�縺励�…ompile/run 螟ｱ謨玲凾縺ｫ stack 繧剃ｿ晄戟縺励※ JSON 蜃ｺ蜉帙∈蜿肴丐縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-after-rootfix.json -j 15` -> `8/8 pass`
  - `node nodesrc/tests.js -i stdlib/alloc/collections/list.nepl --no-stdlib --no-tree -o /tmp/tests-list-after-rootfix.json -j 15` -> `11/11 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-rootfix.json -j 15` -> `707/791 pass`・域ｮ九ｊ `84 fail` 縺ｯ run 譎・`Maximum call stack size exceeded`縲Ａcompile: unreachable` 縺ｯ蜀咲樟縺帙★・・

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: web 螳溯｡梧凾 `unreachable` 縺ｮ蛻・ｊ蛻・￠)

- 逶ｮ逧・
  - 蜈ｨ菴薙ユ繧ｹ繝・(`tests + stdlib`) 縺ｧ螟夂匱縺吶ｋ `phase=compile, error=unreachable` 繧偵�・俣縺ｫ蜷医ｏ縺帙〒縺ｯ縺ｪ縺乗�ｹ譛ｬ蜴溷屏縺九ｉ蛻・ｊ蛻・￠繧九�・
- 螳滓命:
  - `trunk build` 蠕後↓
    - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-baseline-after-revert-v1.json -j 15`
    - 邨先棡: `349/791 pass`縲～442 fail`縲∽ｸ贋ｽ榊､ｱ謨励・ `stdlib/alloc/collections/list.nepl` doctest 鄒､縺ｮ `unreachable`縲・
  - 蜷後§蜈･蜉帙ｒ `nepl-cli` 縺ｧ蜊倅ｽ薙さ繝ｳ繝代う繝ｫ:
    - `target/debug/nepl-cli -i /tmp/list_doctest1_clean.nepl --target std --emit wasm -o /tmp/list_doctest1_out -v`
    - 邨先棡: compile 謌仙粥 (`DEBUG: compile_module returned Ok`)縲・
- 邨占ｫ・
  - 螟ｱ謨励・ `web/dist`・・ASM 荳翫・ compiler 螳溯｡鯉ｼ臥ｵ瑚ｷｯ縺ｫ髯仙ｮ壹＆繧後ｋ縲・
  - `codegen_wasm` 縺ｮ莉雁屓蟾ｮ蛻・ｒ謌ｻ縺励※繧ょ・迴ｾ縺吶ｋ縺溘ａ縲∝腰邏斐↑ backend 螟画峩襍ｷ蝗�縺ｧ縺ｯ縺ｪ縺・�・
  - 莉･髯阪・ `web` 蛛ｴ縺ｧ panic 繧定ｨｺ譁ｭ蛹悶＠縺ｦ蜴溷屏菴咲ｽｮ繧貞庄隕門喧縺吶ｋ繧ｿ繧ｹ繧ｯ繧剃ｸ頑ｵ∬ｪｲ鬘後→縺励※謇ｱ縺・�・

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: todo謨ｴ逅・+ llvm precheck 霑斐ｊ蛟､隕冗ｴ・

- 逶ｮ逧・
  - `todo.md` 縺ｮ螳御ｺ・ｸ医∩鬆・岼・・UnsupportedHirLowering` 謨ｴ逅・ｼ峨ｒ蜿肴丐縺励�∵悴螳御ｺ・□縺代ｒ谿九☆縲・
  - LLVM 蜑肴ｮｵ讀懈渊縺ｫ縲碁撼 unit 髢｢謨ｰ縺ｯ蛟､繧定ｿ斐☆縲崎ｦ冗ｴ・ｒ霑ｽ蜉�縺励※縲｜ackend 萓晏ｭ伜､ｱ謨励・蜑肴ｮｵ蛹悶ｒ騾ｲ繧√ｋ縲・
- 螟画峩:
  - `todo.md`
    - 繝輔ぉ繝ｼ繧ｺD縺ｮ螳御ｺ・ｸ医∩陦・
      - `llvm 邨瑚ｷｯ縺ｧ繧・backend 萓晏ｭ倥お繝ｩ繝ｼ繧貞燕谿ｵ險ｺ譁ｭ縺ｫ蟇・○繧具ｼ・nsupportedHirLowering 縺ｮ謨ｴ逅・ｼ荏
      繧貞炎髯､縺励�∵ｮ玖ｪｲ鬘後→縺励※
      - `llvm 邨瑚ｷｯ縺ｮ precheck 繧呈僑蠑ｵ縺励�（ntrinsic/謌ｻ繧雁�､隕冗ｴ・↑縺ｩ backend 萓晏ｭ伜､ｱ謨励ｒ蜑肴ｮｵ縺ｧ遒ｺ螳壹☆繧九�Ａ
      縺ｸ譖ｴ譁ｰ縲・
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_llvm_codegen` 縺ｫ `TypeCtx` 繧呈ｸ｡縺吝ｽ｢縺ｸ螟画峩縲・
    - reachable 縺ｪ `HirBody::Block` 髢｢謨ｰ縺ｫ縺､縺・※縲∵綾繧雁�､蝙九′髱・`unit` 縺九▽ block 縺悟�､繧定ｿ斐＆縺ｪ縺・�ｴ蜷医ｒ `D3003` 縺ｧ險ｺ譁ｭ縲・
  - `nepl-core/src/codegen_llvm.rs`
    - `precheck_llvm_codegen(&types, &hir, &reachable_set)` 蜻ｼ縺ｳ蜃ｺ縺励∈譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v9.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm codegen_precheck 縺ｫ螳滓､懈渊繧定ｿｽ蜉�)

- 逶ｮ逧・
  - `codegen` 蛻ｰ驕泌ｾ後・逕滓・蟆ゆｻｻ縺ｫ蟇・○繧九◆繧√�´LVM 蛛ｴ縺ｧ繧ょ燕谿ｵ讀懈渊縺ｧ蠑ｾ縺代ｋ蜈･蜉帙ｒ蠅励ｄ縺吶�・
- 螟画峩:
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_llvm_codegen` 繧定ｿｽ蜉�縲・
    - 蛻ｰ驕秘未謨ｰ・・eachable set・峨↓蟇ｾ縺励※ expression tree 繧定ｵｰ譟ｻ縺励�´LVM 譛ｪ蟇ｾ蠢・intrinsic 繧貞燕谿ｵ險ｺ譁ｭ蛹悶�・
    - 譛ｪ蟇ｾ蠢・intrinsic 縺ｯ `D3012 (TypeUnknownIntrinsic)` 縺ｧ蝣ｱ蜻翫�・
  - `nepl-core/src/codegen_llvm.rs`
    - HIR lower 蜑阪↓ `precheck_llvm_codegen` 繧貞ｮ溯｡後＠縲‘rror 縺後≠繧後・ `TypecheckFailed` 縺ｧ譌ｩ譛溽ｵゆｺ・�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v8.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm backend 險ｺ譁ｭ蝙九・謨ｴ逅・

- 逶ｮ逧・
  - `codegen_llvm` 縺九ｉ `UnsupportedHirLowering` 霑泌唆邨瑚ｷｯ縺梧ｶ医∴縺溽憾諷九ｒ蝙句ｮ夂ｾｩ縺ｫ繧ょ渚譏�縺吶ｋ縲・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - `LlvmCodegenError::UnsupportedHirLowering` 繧・enum / Display 縺九ｉ蜑企勁縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v6.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm 谿句ｭ・backend 險ｺ譁ｭ縺ｮ荳榊､画擅莉ｶ蛹・邯咏ｶ・

- 逶ｮ逧・
  - `codegen_llvm` 縺ｫ谿九▲縺ｦ縺・◆ `UnsupportedHirLowering` 繧貞炎貂帙＠縲∝燕谿ｵ騾夐℃蠕後・逕滓・蟆ゆｻｻ繝｢繝・Ν縺ｸ蟇・○繧九�・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - 莉･荳九ｒ `UnsupportedHirLowering` 霑泌唆縺九ｉ internal panic 縺ｸ螟画峩:
      - 髢｢謨ｰ return 蝙倶ｸ堺ｸ�閾ｴ
      - enum/struct/tuple 讒狗ｯ画凾縺ｮ `alloc` 蠢・�亥愛螳・
      - enum payload / struct field / tuple item 縺ｮ蛟､逕滓・蠢・�医・蝙倶ｸ堺ｸ�閾ｴ
      - `match` arm 縺ｮ邨先棡蝙倶ｸ堺ｸ�閾ｴ
      - unknown intrinsic 蛻ｰ驕・
      - unsupported expression kind 蛻ｰ驕・
      - 譁・ｭ怜・繝ｪ繝・Λ繝ｫID遽・峇螟・
      - 譁・ｭ怜・蜈ｷ菴灘喧譎ゅ・ `alloc` 蠢・�亥愛螳・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v5.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm intrinsic 蠑墓焚繝ｻ蝙九メ繧ｧ繝・け縺ｮ backend 險ｺ譁ｭ繧剃ｸ榊､画擅莉ｶ蛹・

- 逶ｮ逧・
  - `codegen_llvm` intrinsic lowering 縺ｫ谿九▲縺ｦ縺・◆ backend 險ｺ譁ｭ繧貞炎貂帙＠縲∝燕谿ｵ騾夐℃蠕後・逕滓・蟆ゆｻｻ繝｢繝・Ν縺ｸ蟇・○繧九�・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - 莉･荳九ｒ `UnsupportedHirLowering` 霑泌唆縺九ｉ internal panic 縺ｸ螟画峩:
      - `load` 縺ｮ蠑墓焚蛟区焚/蝙句ｼ墓焚蛟区焚荳堺ｸ�閾ｴ縲√・繧､繝ｳ繧ｿ蛟､荳榊惠縲√・繧､繝ｳ繧ｿ蝙倶ｸ堺ｸ�閾ｴ
      - `store` 縺ｮ蠑墓焚蛟区焚/蝙句ｼ墓焚蛟区焚荳堺ｸ�閾ｴ縲√・繧､繝ｳ繧ｿ/蛟､荳榊惠縲√・繧､繝ｳ繧ｿ蝙倶ｸ堺ｸ�閾ｴ縲～u8` 蛟､蝙倶ｸ堺ｸ�閾ｴ縲∵�ｼ邏榊梛荳堺ｸ�閾ｴ
      - `add` 縺ｮ蠑墓焚蛟区焚荳堺ｸ�閾ｴ縲〕hs/rhs 荳榊惠縲（32莉･螟・
      - `f32_to_i32` / `i32_to_u8` / `u8_to_i32` 縺ｮ蠑墓焚蛟区焚繝ｻ蛟､荳榊惠繝ｻ蝙倶ｸ堺ｸ�閾ｴ
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v4.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm 蛻ｶ蠕｡讒区枚縺ｮ backend 險ｺ譁ｭ繧剃ｸ榊､画擅莉ｶ蛹・

- 逶ｮ逧・
  - `codegen_llvm` 縺ｮ `if/while/match` 縺ｧ谿九▲縺ｦ縺・◆ backend 險ｺ譁ｭ繧貞炎貂帙＠縲∝梛讀懈渊繝ｻ蜑肴ｮｵ讀懆ｨｼ騾夐℃蠕後・逕滓・蟆ゆｻｻ縺ｸ蟇・○繧九�・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - `if`:
      - 譚｡莉ｶ縺悟�､繧定ｿ斐＆縺ｪ縺・
      - 譚｡莉ｶ縺・`i32/bool` 莠呈鋤縺ｧ縺ｪ縺・
      - then/else 蛻・ｲ千ｵ先棡蝙倶ｸ堺ｸ�閾ｴ
      繧・`UnsupportedHirLowering` 霑泌唆縺九ｉ internal panic 縺ｸ螟画峩縲・
    - `while`:
      - 譚｡莉ｶ縺悟�､繧定ｿ斐＆縺ｪ縺・
      - 譚｡莉ｶ縺・`i32/bool` 莠呈鋤縺ｧ縺ｪ縺・
      繧・internal panic 縺ｸ螟画峩縲・
    - `match`:
      - scrutinee 縺悟�､繧定ｿ斐＆縺ｪ縺・
      - scrutinee 縺・enum pointer (`i32`) 縺ｧ縺ｪ縺・
      - arm 縺・莉ｶ
      繧・internal panic 縺ｸ螟画峩縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v3.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm call_indirect 縺ｮ backend 險ｺ譁ｭ繧剃ｸ榊､画擅莉ｶ蛹・

- 逶ｮ逧・
  - `codegen_llvm` 縺ｮ `call_indirect` 縺ｧ谿九▲縺ｦ縺・◆ backend 險ｺ譁ｭ・・UnsupportedHirLowering`・峨ｒ蜑頑ｸ帙＠縲∝燕谿ｵ騾夐℃蠕後・逕滓・蟆ゆｻｻ縺ｫ蟇・○繧九�・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - `call_indirect` 縺ｫ縺､縺・※莉･荳九・ `UnsupportedHirLowering` 霑泌唆繧・internal panic 蛹・
      - callee 縺悟�､繧定ｿ斐＆縺ｪ縺・
      - callee 縺・`i32` 髢｢謨ｰID縺ｧ縺ｪ縺・
      - 蠑墓焚縺悟�､繧定ｿ斐＆縺ｪ縺・
      - 蠑墓焚蛟区焚荳堺ｸ�閾ｴ
      - 蠑墓焚蝙倶ｸ堺ｸ�閾ｴ
      - 蛟呵｣憺未謨ｰ譛ｪ讀懷・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md -i tests/llvm_target.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v2.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: raw wasm 陦梧､懈渊縺ｮ蜑肴ｮｵ蛻・屬繧貞ｮ御ｺ・

- 逶ｮ逧・
  - `codegen_precheck` 縺・`codegen_wasm` 螳溯｣・ｩｳ邏ｰ縺ｸ萓晏ｭ倥☆繧狗ｵ瑚ｷｯ繧定ｧ｣豸医＠縲∝燕谿ｵ讀懈渊縺ｮ雋ｬ蜍吶ｒ `wasm_shared` 縺ｸ髮・ｴ・☆繧九�・
  - 縲慶odegen 蛻ｰ驕疲凾縺ｯ逕滓・蟆ゆｻｻ縲阪・譁ｹ驥昴ｒ邯ｭ謖√＠縲〉aw wasm 陦後ヱ繝ｼ繧ｹ螟ｱ謨励ｒ蜑肴ｮｵ縺ｧ遒ｺ螳壹☆繧九�・
- 螟画峩:
  - `nepl-core/src/wasm_shared.rs`
    - `parse_wasm_line_with_lookup` 繧貞・譛牙喧縲・
    - `precheck_raw_wasm_body` 繧定ｿｽ蜉�縺励�～HirBody::Wasm` 陦後ｒ蜑肴ｮｵ縺ｧ讀懈渊縺励※ `D4004` 繧定ｿ斐☆繧医≧縺ｫ螟画峩縲・
  - `nepl-core/src/passes/codegen_precheck.rs`
    - raw wasm 莠句燕讀懈渊蜻ｼ縺ｳ蜃ｺ縺怜・繧・`codegen_wasm` 縺九ｉ `wasm_shared` 縺ｸ螟画峩縲・
  - `todo.md`
    - 繝輔ぉ繝ｼ繧ｺD縺ｮ縲形codegen_precheck` 縺ｮ wasm 蛛ｴ繝倥Ν繝台ｾ晏ｭ俶紛逅・�埼�・岼繧貞ｮ御ｺ・→縺励※蜑企勁縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `NO_COLOR=false node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v1.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: #wasm 縺ｮ繧ｹ繧ｿ繝・け讀懆ｨｼ繧貞燕谿ｵ讀懈渊縺ｸ遘ｻ蜍・

- 逶ｮ逧・
  - 縲慶odegen 縺ｯ豁｣縺励＞蜈･蜉帙ｒ逕滓・縺吶ｋ縺�縺代�阪・譁ｹ驥昴↓蜷医ｏ縺帙�～#wasm` 繝懊ョ繧｣讀懆ｨｼ繧・backend 螳溯｡梧凾縺ｧ縺ｯ縺ｪ縺・`codegen_precheck` 蛛ｴ縺ｧ螳御ｺ・＆縺帙ｋ縲・
- 螟画峩:
  - `nepl-core/src/codegen_wasm.rs`
    - `precheck_raw_wasm_body` 繧ｷ繧ｰ繝阪メ繝｣繧・`precheck_raw_wasm_body(ctx, func)` 縺ｫ螟画峩縲・
    - raw 陦後・繝代・繧ｹ謌仙粥譎ゅ↓蜻ｽ莉､蛻励ｒ闢・ｩ阪＠縲∝燕谿ｵ縺ｧ `validate_wasm_stack` 繧貞ｮ溯｡後☆繧九ｈ縺・､画峩縲・
    - `lower_user` 縺ｮ `HirBody::Wasm` 邨瑚ｷｯ縺九ｉ `validate_wasm_stack` 繧貞炎髯､縲・
    - `generate_wasm` 縺ｮ險ｺ譁ｭ髮・ｴ・ｒ螳溯ｳｪ遨ｺ縺ｫ謨ｴ逅・ｼ・odegen 蜀・ｨｺ譁ｭ繧堤匱逕溘＆縺帙↑縺・婿蜷代↓邨ｱ荳�・峨�・
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_raw_wasm_body` 蜻ｼ縺ｳ蜃ｺ縺励ｒ譁ｰ繧ｷ繧ｰ繝阪メ繝｣縺ｸ譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v4.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: codegen_precheck 縺ｮ wasm 莠句燕讀懈渊繧貞・騾壹Δ繧ｸ繝･繝ｼ繝ｫ縺ｸ蛻・屬)

- 逶ｮ逧・
  - `passes/codegen_precheck.rs` 縺・`codegen_wasm.rs` 螳溯｣・ｩｳ邏ｰ縺ｸ逶ｴ謗･萓晏ｭ倥＠縺ｦ縺・◆迥ｶ諷九ｒ謨ｴ逅・＠縲∝燕谿ｵ讀懈渊繝ｭ繧ｸ繝・け繧貞・譛峨Δ繧ｸ繝･繝ｼ繝ｫ縺ｸ蛻・屬縺吶ｋ縲・
  - 縲慶odegen 縺ｯ豁｣縺励＞蜈･蜉帙ｒ逕滓・縺吶ｋ縺�縺代�阪・譁ｹ驥昴↓蜷医ｏ縺帙�｜ackend 縺ｮ `skip`/險ｺ譁ｭ闢・ｩ阪ｒ荳榊､画擅莉ｶ驕募渚縺ｸ蟇・○繧九�・
- 螟画峩:
  - `nepl-core/src/wasm_shared.rs` 繧呈眠隕剰ｿｽ蜉�縲・
    - wasm鄂ｲ蜷崎ｧ｣豎ｺ (`wasm_sig`, `wasm_sig_ids`)
    - generic skip 蛻､螳・(`should_skip_wasm_codegen_for_generic`)
    - 蛻ｰ驕秘未謨ｰ隗｣譫・(`collect_reachable_wasm_functions`)
    - 髢捺磁蜻ｼ縺ｳ蜃ｺ縺励ｒ蜷ｫ繧�鄂ｲ蜷埼寔蜷亥庶髮・(`collect_wasm_signature_set`)
    - wasm intrinsic 蟇ｾ蠢懷愛螳・(`is_supported_wasm_intrinsic`)
  - `nepl-core/src/passes/codegen_precheck.rs`
    - 荳願ｨ倥Ο繧ｸ繝・け繧・`wasm_shared` 蜿ら・縺ｸ鄂ｮ謠帙�・
    - `precheck_raw_wasm_body` 縺ｮ縺ｿ `codegen_wasm` 蛛ｴ繧堤ｶ咏ｶ壼茜逕ｨ・域ｬ｡谿ｵ縺ｧ蛻・屬莠亥ｮ夲ｼ峨�・
  - `nepl-core/src/codegen_wasm.rs`
    - extern/function 鄂ｲ蜷堺ｸ堺ｸ�閾ｴ譎ゅ・ `skip` 繧貞ｻ・ｭ｢縺・internal panic 蛹悶�・
    - `lower_body` 縺ｧ backend 險ｺ譁ｭ縺瑚ｿ斐ｋ邨瑚ｷｯ繧・internal panic 蛹悶�・
    - 蜈ｱ譛峨Ο繧ｸ繝・け縺ｯ `wasm_shared` 蜻ｼ縺ｳ蜃ｺ縺励∈蟋碑ｭｲ縲・
  - `nepl-core/src/lib.rs`
    - `pub mod wasm_shared;` 繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-shared-v3.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm backend 險ｺ譁ｭ繧貞燕谿ｵ荳榊､画擅莉ｶ縺ｸ遘ｻ陦・

- 逶ｮ逧・
  - `todo.md` 繝輔ぉ繝ｼ繧ｺD譁ｹ驥昴↓蜷医ｏ縺帙�～codegen_llvm` 蛛ｴ縺ｧ逋ｺ陦後＠縺ｦ縺・◆縲悟燕谿ｵ騾夐℃蠕後↓蛻ｰ驕斐＠縺ｪ縺・・縺壹�阪・險ｺ譁ｭ繧貞ｻ・ｭ｢縺励�∝燕谿ｵ讀懆ｨｼ縺ｮ荳榊､画擅莉ｶ縺ｨ縺励※謇ｱ縺・�・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - `let` 縺ｮ蝙倶ｸ堺ｸ�閾ｴ (`let type mismatch`) 繧・`UnsupportedHirLowering` 縺九ｉ internal panic 縺ｸ螟画峩縲・
    - `set` 縺ｮ蝙倶ｸ堺ｸ�閾ｴ (`set type mismatch`) 繧・`UnsupportedHirLowering` 縺九ｉ internal panic 縺ｸ螟画峩縲・
    - 譛ｪ隗｣豎ｺ trait call 縺ｮ蛻ｰ驕斐ｒ `UnsupportedHirLowering` 縺九ｉ internal panic 縺ｸ螟画峩縲・
    - call 蠑墓焚蝙倶ｸ堺ｸ�閾ｴ繧・`UnsupportedHirLowering` 縺九ｉ internal panic 縺ｸ螟画峩縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-llvm-invariant-v2.json -j 15` -> `8/8 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-llvm-invariant-panic-v1.json -j 15` -> `707/791 pass`・・Maximum call stack size exceeded` 縺悟､壽焚縲ゆｻ雁屓縺ｮ螟画峩蟇ｾ雎｡螟悶・譌｢蟄伜､ｱ謨励→縺励※邯咏ｶ夊ｪｿ譟ｻ・・

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC/D謗･邯・ core/mem 縺ｫ MemPtr 蛻晄悄蛹悶が繝ｼ繝舌・繝ｭ繝ｼ繝芽ｿｽ蜉�)

- 逶ｮ逧・
  - `core/mem` 蠕梧ｮｵ遘ｻ陦鯉ｼ・stdlib/std`/tutorials・峨〒 `i32` 逕溘・繧､繝ｳ繧ｿ繧帝愆蜃ｺ縺帙★縺ｫ驟榊・蛻晄悄蛹悶〒縺阪ｋ荳頑ｵ、PI繧堤畑諢上☆繧九�・
  - `MemPtr` 繝｢繝・Ν荳翫〒 `fill/memset` 繧堤ｵｱ荳�縺励�～Result` 縺ｧ螟ｱ謨励ｒ謇ｱ縺医ｋ繧医≧縺ｫ縺吶ｋ縲・
- 螟画峩:
  - `stdlib/core/mem.nepl`
    - `memset_u8 <(MemPtr<u8>,i32,i32)->Result<(),str>>` 繧定ｿｽ蜉�縲・
    - `fill_u8 <(MemPtr<u8>,i32,i32)->Result<(),str>>` 繧定ｿｽ蜉�縲・
    - `fill_i32 <(MemPtr<i32>,i32,i32)->Result<(),str>>` 繧定ｿｽ蜉�縲・
    - 辟｡蜉ｹ繝昴う繝ｳ繧ｿ繧・ｲ�縺ｮ髟ｷ縺輔・ `Result::Err` 繧定ｿ斐☆縲・
  - `tests/memory_safety.n.md`
    - `MemPtr fill_i32/fill_u8 縺ｮ螳牙・繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝荏 繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
    - `MemPtr fill 邉ｻ縺ｯ辟｡蜉ｹ蠑墓焚繧・Err 縺ｧ霑斐☆` 繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety-fill-overload.json -j 15` -> `217/217 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-mem-fill-overload.json -j 15` -> `787/787 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: kpread_core 繝倥ャ繝�繝輔ぅ繝ｼ繝ｫ繝峨・蝙句ｮ牙・蛹・

- 逶ｮ逧・
  - `kpread_core` 縺ｫ谿九▲縺ｦ縺・◆繝倥ャ繝�逕溘が繝輔そ繝・ヨ・・0/4/8`・峨ｒ蛻玲嫌蝙九∈遘ｻ陦後＠縲～kpread`/`kpwrite` 縺ｨ蜷後§蠅・阜陦ｨ迴ｾ縺ｫ謠・∴繧九�・
  - 繝倥ャ繝�繝ｬ繧､繧｢繧ｦ繝医・諢丞袖繧貞梛縺ｧ蝗ｺ螳壹＠縲√が繝輔そ繝・ヨ隱､謖・ｮ壹ｒ荳頑ｵ√〒髦ｲ縺舌�・
- 螟画峩:
  - `stdlib/kp/kpread_core.nepl`
    - `ScannerHeaderFieldCore` 繧定ｿｽ蜉�・・BufPtr` / `Len` / `Pos`・峨�・
    - `scanner_header_core_off` 繧定ｿｽ蜉�縺励�√が繝輔そ繝・ヨ隗｣豎ｺ繧・邂・園縺ｫ髮・ｴ・�・
    - `store_i32_u8_at sc*_region 0/4/8 ...` 繧貞・謖吝梛 + 繧ｪ繝輔そ繝・ヨ髢｢謨ｰ邨檎罰縺ｸ鄂ｮ謠帙�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-kp-core-header-field-enum.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpread-core-header-field-enum.json -j 15` -> `785/785 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: kpwrite 繝倥ャ繝�繝輔ぅ繝ｼ繝ｫ繝峨・蝙句ｮ牙・蛹・

- 逶ｮ逧・
  - `kpwrite` 縺ｮ繝倥ャ繝�繧｢繧ｯ繧ｻ繧ｹ縺ｧ菴ｿ縺｣縺ｦ縺・◆逕溘が繝輔そ繝・ヨ蛟､・・0/4/8/12/16`・峨ｒ蛻玲嫌蝙九↓鄂ｮ謠帙＠縲～kpread` 縺ｨ蜷後§螳牙・繝｢繝・Ν縺ｸ邨ｱ荳�縺吶ｋ縲・
  - `mem/kpread/kpwrite` 縺ｮ蜈ｬ髢帰PI螳牙・蛹悶〒縲√・繝・ム蠅・阜縺ｮ諢丞袖繧貞梛縺ｧ陦ｨ迴ｾ縺吶ｋ縲・
- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `WriterHeaderField` 繧定ｿｽ蜉�・・BufPtr` / `Cap` / `WriteLen` / `IovPtr` / `NwPtr`・峨�・
    - `writer_header_off` 繧定ｿｽ蜉�縺励�√が繝輔そ繝・ヨ隗｣豎ｺ繧剃ｸ�邂・園縺ｫ髮・ｴ・�・
    - `writer_header_ptr` / `writer_load_header` / `writer_store_header` / `writer_load_header_ptr` 縺ｮ隨ｬ2蠑墓焚繧・`i32` 縺九ｉ `WriterHeaderField` 縺ｫ螟画峩縲・
    - 蜻ｼ縺ｳ蜃ｺ縺怜・縺ｮ逕滓焚蛟､繧ｪ繝輔そ繝・ヨ繧貞・蟒・＠縲∝・謖吝�､縺ｫ鄂ｮ謠帙�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl --no-tree -o /tmp/tests-kp-header-field-enum-unified.json -j 15` -> `226/226 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpwrite-header-field-enum.json -j 15` -> `785/785 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: kpread 繝倥ャ繝�繝輔ぅ繝ｼ繝ｫ繝峨・蝙句ｮ牙・蛹・

- 逶ｮ逧・
  - `kpread` 縺ｮ繝倥ャ繝�繧｢繧ｯ繧ｻ繧ｹ縺ｧ菴ｿ縺｣縺ｦ縺・◆逕溘が繝輔そ繝・ヨ蛟､・・0/4/8`・峨ｒ蛻玲嫌蝙九∈鄂ｮ縺肴鋤縺医�∝他縺ｳ蜃ｺ縺怜・縺ｮ隱､謖・ｮ壹ｒ貂帙ｉ縺吶�・
  - `todo.md` 2026-03-03 繝輔ぉ繝ｼ繧ｺD・・mem/kpread/kpwrite` 縺ｮ蜈ｬ髢帰PI螳牙・蛹厄ｼ峨↓豐ｿ縺｣縺ｦ縲∽ｸ頑ｵ√・陦ｨ迴ｾ繧貞崋螳壹☆繧九�・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - `ScannerHeaderField` 繧定ｿｽ蜉�・・BufPtr` / `Len` / `Pos`・峨�・
    - `scanner_header_off` 繧定ｿｽ蜉�縺励�√が繝輔そ繝・ヨ隗｣豎ｺ繧・邂・園縺ｸ髮・ｴ・�・
    - `scanner_header_ptr` / `scanner_load_header` / `scanner_store_header` 縺ｮ隨ｬ2蠑墓焚繧・`i32` 縺九ｉ `ScannerHeaderField` 縺ｫ螟画峩縲・
    - 蜻ｼ縺ｳ蜃ｺ縺怜・縺ｮ `scanner_load_header sc 0/4/8` 縺ｨ `scanner_store_header sc 8 ...` 繧貞・謖吝梛謖・ｮ壹∈鄂ｮ謠帙�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpread.nepl --no-tree -o /tmp/tests-kpread-header-field-targeted.json -j 15` -> `222/222 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpread-header-field.json -j 15` -> `785/785 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: kpread 繝倥ャ繝�繧｢繧ｯ繧ｻ繧ｹ縺ｮ繧ｵ繧､繝ｬ繝ｳ繝亥､ｱ謨励ｒ髯､蜴ｻ)

- 逶ｮ逧・
  - `scanner_load_header` / `scanner_store_header` 縺ｮ螟ｱ謨玲凾繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ・・0` / `()`・峨ｒ蟒・ｭ｢縺励�√・繝・ム荳肴紛蜷医ｒ髫�阡ｽ縺励↑縺・�・
  - 荳頑ｵ∽ｻ墓ｧ假ｼ亥ｮ牙・API蜆ｪ蜈茨ｼ峨↓蜷医ｏ縺帙�∝｣翫ｌ縺溽憾諷九ｒ邯咏ｶ壹＆縺帙ｋ繧医ｊ蜊ｳ譎ょ●豁｢縺ｫ邨ｱ荳�縺吶ｋ縲・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - `scanner_load_header`:
      - `scanner_header_ptr` 縺・`Err` 縺ｮ蝣ｴ蜷医・ `0` 霑泌唆繧・`#intrinsic "unreachable"` 縺ｸ螟画峩縲・
      - `load_i32` 縺・`None` 縺ｮ蝣ｴ蜷医・ `0` 霑泌唆繧・`#intrinsic "unreachable"` 縺ｸ螟画峩縲・
    - `scanner_store_header`:
      - `scanner_header_ptr` 縺・`Err` 縺ｮ蝣ｴ蜷医・辟｡隕悶ｒ `#intrinsic "unreachable"` 縺ｸ螟画峩縲・
      - `store_i32` 縺・`Err` 縺ｮ蝣ｴ蜷医・辟｡隕悶ｒ `#intrinsic "unreachable"` 縺ｸ螟画峩縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md -i stdlib/kp/kpread.nepl --no-tree -o /tmp/tests-kpread-header-unreachable-targeted.json -j 15` -> `222/222 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-kpread-unreachable.json -j 15` -> `785/785 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD蜈郁｡・ Writer 繧・RegionToken 菫晄戟縺ｸ遘ｻ陦・

- 逶ｮ逧・
  - `kpread` 縺ｨ蜷梧ｧ倥↓ `kpwrite` 縺ｧ繧ょ・髢九ワ繝ｳ繝峨Ν縺碁�伜沺諠・�ｱ繧呈戟縺､繧医≧縺ｫ縺励�√Γ繝｢繝ｪ螳牙・API繧堤ｵｱ荳�縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - `Writer` 縺ｯ `MemPtr<u8>` 繧堤峩謗･菫晄戟縺励�√・繝・ム鬆伜沺繧ｵ繧､繧ｺ・・0byte・峨′蝙九↓陦ｨ迴ｾ縺輔ｌ縺ｦ縺・↑縺九▲縺溘�・
  - 騾比ｸｭ縺ｧ霑ｽ蜉�縺励◆ `writer_mem(Writer)->MemPtr<u8>` 繝倥Ν繝代・ `Writer` 繧貞�､貂｡縺励〒蜿励￠繧九◆繧√�・
    non-copy 縺ｪ `Writer` 縺ｮ move 繧堤匱逕溘＆縺・`D3053` 繧貞ｼ輔″襍ｷ縺薙＠縺溘�・
- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `Writer.raw` 繧・`Writer.region: RegionToken<u8>` 縺ｫ螟画峩縲・
    - `writer_wrap` 縺ｧ `region_new raw 20` 繧呈ｧ狗ｯ峨�・
    - `writer_mem` 繝倥Ν繝代・蜑企勁縺励�～region_ptr get w "region"` 繧堤峩謗･螻暮幕縺励※ move 繧貞屓驕ｿ縲・
  - `stdlib/kp/kpread_core.nepl`
    - `store_i32_u8_at/load_i32_u8_at` 繧・`RegionToken<u8>` 蜿励￠蜿悶ｊ縺ｸ螟画峩縲・
    - `sc0/iov/nread/sc` 縺ｮ蜷・�伜沺繧・`RegionToken` 蛹悶＠縺ｦ繧｢繧ｯ繧ｻ繧ｹ邨瑚ｷｯ繧堤ｵｱ荳�縲・
    - 騾比ｸｭ縺ｧ逋ｺ逕溘＠縺・`match` 繧｢繝ｼ繝�蟠ｩ繧鯉ｼ・D3009/D3008/D3045`・峨ｒ菫ｮ豁｣縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-writer-regiontoken-v3.json -j 15`
  - 邨先棡: `221/221 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD蜈郁｡・ kpread_core 縺ｮ蜀・Κ繝倥ャ繝�繧｢繧ｯ繧ｻ繧ｹ繧・RegionToken 蛹・

- 逶ｮ逧・
  - `kpread_core` 縺ｮ蜀・Κ繝｡繝｢繝ｪ繧｢繧ｯ繧ｻ繧ｹ繧・`RegionToken` 邨檎罰縺ｫ邨ｱ荳�縺励�～MemPtr + off` 縺ｮ逶ｴ謗･邂苓｡謎ｾ晏ｭ倥ｒ貂帙ｉ縺吶�・
- 譬ｹ譛ｬ蜴溷屏:
  - `store_i32_u8_at` / `load_i32_u8_at` 縺・`MemPtr<u8>` 縺ｨ `off` 縺九ｉ逶ｴ謗･ `MemPtr<i32>` 繧剃ｽ懊ｋ險ｭ險医〒縲・
    鬆伜沺蠅・阜縺ｮ蜑肴署縺後・繝ｫ繝大､悶∈貍上ｌ縺ｦ縺・◆縲・
- 螟画峩:
  - `stdlib/kp/kpread_core.nepl`
    - `mem_i32_region_ptr` 繧定ｿｽ蜉�縺励�～region_ptr_at<u8,i32>` 繧剃ｽｿ逕ｨ縲・
    - `store_i32_u8_at` / `load_i32_u8_at` 縺ｮ蠑墓焚繧・`RegionToken<u8>` 縺ｫ螟画峩縲・
    - `sc0(12)`, `iov(8)`, `nread(4)`, `sc(12)` 縺ｧ `RegionToken` 繧呈ｧ狗ｯ峨＠縺ｦ繝倥Ν繝代∈貂｡縺吝ｽ｢縺ｫ譖ｴ譁ｰ縲・
  - 騾比ｸｭ菫ｮ豁｣:
    - `match dealloc_ptr<u8> buf cap` 縺ｮ `Result::Err` 繧｢繝ｼ繝�縺ｮ繧､繝ｳ繝・Φ繝亥ｴｩ繧後↓繧医ｊ
      `D3009/D3008/D3045` 縺檎匱逕溘＠縺溘◆繧√�∝・蟯先ｧ矩��繧呈ｭ｣縺励￥菫ｮ豁｣縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-kpread-core-regiontoken-v2.json -j 15`
  - 邨先棡: `221/221 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD蜈郁｡・ kpwrite 繝倥ャ繝�繧｢繧ｯ繧ｻ繧ｹ繧・RegionToken 邨檎罰縺ｸ遘ｻ陦・

- 逶ｮ逧・
  - `kpwrite` 蛛ｴ縺ｧ繧ゅ・繝・ム繧｢繧ｯ繧ｻ繧ｹ繧・`RegionToken` 繝吶・繧ｹ縺ｫ蟇・○縲～core/mem` 縺ｮ蠅・阜讀懆ｨｼAPI繧貞・蛻ｩ逕ｨ縺ｧ縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - 譌｢蟄・`writer_header_ptr` 縺ｯ `mem_ptr_addr + off` 縺ｧ逶ｴ謗･繧｢繝峨Ξ繧ｹ邂苓｡薙ｒ陦後＞縲・
    20byte 繝倥ャ繝�蠅・阜縺ｮ蜑肴署繧帝未謨ｰ縺斐→縺ｫ證鈴ｻ吝喧縺励※縺・◆縲・
- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_header_region` 繧定ｿｽ蜉�・・region_new w_mem 20`・峨�・
    - `writer_header_ptr` 繧・`Result<MemPtr<i32>,str>` 縺ｸ螟画峩縺励�～region_ptr_at<u8,i32>` 繧剃ｽｿ逕ｨ縲・
    - `writer_load_header` / `writer_store_header` 繧剃ｸ願ｨ・`Result` 邨瑚ｷｯ縺ｸ譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-writer-header-regiontoken.json -j 15`
  - 邨先棡: `221/221 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD蜈郁｡・ kpread 縺ｮ Scanner 繝倥ャ繝�繧・RegionToken 蛹・

- 逶ｮ逧・
  - `todo.md` 繝輔ぉ繝ｼ繧ｺD逹�謇九→縺励※縲～kpread` 縺ｮ蜈ｬ髢九ワ繝ｳ繝峨Ν縺ｫ鬆伜沺謇�譛画ュ蝣ｱ繧呈戟縺溘○縲～core/mem` 縺ｮ譁ｰ螳牙・API縺ｸ蟇・○繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - `Scanner` 縺・`MemPtr<u8>` 逶ｴ謗･菫晄戟縺ｮ縺ｿ縺ｧ縲√・繝・ム鬆伜沺蠅・阜縺ｮ諠・�ｱ縺悟梛縺ｫ荵励▲縺ｦ縺・↑縺九▲縺溘�・
  - 繝倥ャ繝�繧｢繧ｯ繧ｻ繧ｹ縺・`mem_ptr_addr + off` 縺ｮ邂苓｡謎ｾ晏ｭ倥〒縲∝｢・阜讀懆ｨｼ繧貞・蛻ｩ逕ｨ縺励↓縺上°縺｣縺溘�・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - `Scanner` 繝輔ぅ繝ｼ繝ｫ繝峨ｒ `raw: MemPtr<u8>` 縺九ｉ `region: RegionToken<u8>` 縺ｫ螟画峩縲・
    - `scanner_wrap` 縺ｧ `region_new raw 12` 繧呈ｧ狗ｯ峨�・
    - `scanner_header_ptr` 繧・`region_ptr_at<u8,i32>` 繝吶・繧ｹ縺ｮ `Result` 霑泌唆縺ｸ螟画峩縲・
    - `scanner_load_header` / `scanner_store_header` 繧剃ｸ願ｨ・`Result` 邨瑚ｷｯ縺ｧ蜃ｦ逅・�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-scanner-regiontoken.json -j 15`
  - 邨先棡: `221/221 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: core/mem 縺ｫ RegionToken 螳牙・API繧定ｿｽ蜉�)

- 逶ｮ逧・
  - `todo.md` 繝輔ぉ繝ｼ繧ｺC縺ｫ豐ｿ縺｣縺ｦ縲～MemPtr<T>` 縺ｨ `RegionToken<T>` 繧剃ｽｿ縺・ｮ牙・API繧・`core/mem` 縺ｫ霑ｽ蜉�縺励�～kpread/kpwrite` 遘ｻ陦後・荳頑ｵ∝渕逶､繧剃ｽ懊ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - 譌｢蟄・`mem` 縺ｯ `MemPtr<T>` 縺ｾ縺ｧ縺ｯ謨ｴ蛯呎ｸ医∩縺�縺｣縺溘′縲・�伜沺謇�譛峨ｒ陦ｨ縺吝・髢帰PI縺御ｸ崎ｶｳ縺励※縺翫ｊ縲・
    蠅・阜諠・�ｱ莉倥″繧｢繧ｯ繧ｻ繧ｹ繧貞梛縺ｨ縺励※邨ｱ荳�縺ｧ縺阪※縺・↑縺九▲縺溘�・
- 螟画峩:
  - `stdlib/core/mem.nepl`
    - `RegionToken<T>` 陬懷勧API繧定ｿｽ蜉�:
      - `region_new`
      - `region_in_bounds`
      - `region_ptr_at`
      - `alloc_region_bytes`
      - `alloc_region`
      - `dealloc_region`
    - 縺薙ｌ縺ｫ繧医ｊ縲・�伜沺繧ｵ繧､繧ｺ繧剃ｼｴ縺・梛莉倥″繧ｪ繝輔そ繝・ヨ蜿門ｾ励ｒ `Result` 縺ｧ謇ｱ縺医ｋ繧医≧縺ｫ縺励◆縲・
  - `tests/memory_safety.n.md`
    - `alloc_region/region_ptr_at/dealloc_region` 縺ｮ蝓ｺ譛ｬ蜍穂ｽ懊こ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
    - 遽・峇螟悶が繝輔そ繝・ヨ縺ｧ `Result::Err` 繧定ｿ斐☆蝗槫ｸｰ繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/block_semicolon_return.n.md -i tests/plan.n.md -i tests/block_single_line.n.md --no-stdlib --no-tree -o /tmp/tests-semicolon-focus.json -j 15`
  - 邨先棡: `67/67 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md --no-tree -o /tmp/tests-memory-safety-region-token.json -j 15`
  - 邨先棡: `211/211 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i tests/kp_i64.n.md --no-tree -o /tmp/tests-memory-kp-regression.json -j 15`
  - 邨先棡: `221/221 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺB2: trait capability 縺ｮ蝙倶ｻ倥″菫晄戟縺ｸ遘ｻ陦・

- 逶ｮ逧・
  - trait capability 蛻､螳壹・譁・ｭ怜・蜀崎ｧ｣譫舌ｒ貂帙ｉ縺励�∝梛莉倥″繝・・繧ｿ縺ｧ荳�雋ｫ縺励※謇ｱ縺・�・
- 譬ｹ譛ｬ蜴溷屏:
  - 譌｢蟄伜ｮ溯｣・〒縺ｯ `TraitInfo.capabilities` 縺・`Vec<String>` 縺ｮ縺溘ａ縲・
    `TraitSemantics::detect` 縺ｧ豈主屓譁・ｭ怜・繧貞・繝代・繧ｹ縺励※縺・◆縲・
  - 縺薙・讒矩��縺ｯ capability 蛻､螳壹・雋ｬ蜍吶′蛻・淵縺励�∝ｰ・擂諡｡蠑ｵ譎ゅ↓荳肴紛蜷医ｒ逕溘∩繧・☆縺・�・
- 螟画峩:
  - `nepl-core/src/typecheck.rs`
    - `TraitInfo.capabilities` 繧・`Vec<String>` 縺九ｉ `Vec<TraitCapability>` 縺ｸ螟画峩縲・
    - trait 螳夂ｾｩ蜃ｦ逅・(`Stmt::Trait`) 縺ｧ capability 繧・蝗槭□縺代ヱ繝ｼ繧ｹ縺励�∝梛莉倥″縺ｧ菫晄戟縲・
    - 驥崎､・capability 謖・ｮ壹・蜷御ｸ�trait蜀・〒驥崎､・匳骭ｲ縺励↑縺・ｈ縺・紛逅・�・
    - `TraitSemantics::detect` 縺ｯ `TraitInfo` 蜀・・蝙倶ｻ倥″ capability 繧堤峩謗･蜿ら・縲・
    - 荳崎ｦ√↓縺ｪ縺｣縺・`detect_declared_trait_capabilities` 繧貞炎髯､縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-move-overload-after-capability-typed.json -j 15`
  - 邨先棡: `272/272 pass`
  - `node nodesrc/tests.js -i tests/compile_fail_diag_location.n.md --no-tree -o /tmp/tests-compile-fail-diag-location-after-capability-typed.json -j 15`
  - 邨先棡: `207/207 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-after-capability-typed.json -j 15`
  - 邨先棡: `783/783 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: kpwrite header 隱ｭ縺ｿ蜿悶ｊ縺ｮ Result 蛹悶→ None 繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ蟒・ｭ｢)

- 逶ｮ逧・
  - `writer_load_header` 縺ｮ `None -> 0` 繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ繧貞ｻ・ｭ｢縺励�”eader 隱ｭ縺ｿ蜿悶ｊ螟ｱ謨励ｒ譏守､ｺ蛻・ｲ舌〒謇ｱ縺・�・
- 譬ｹ譛ｬ蜴溷屏:
  - 蠕捺擂縺ｮ `writer_load_header` 縺ｯ `load_i32` 螟ｱ謨玲凾縺ｫ 0 繧定ｿ斐＠縺ｦ縺翫ｊ縲∫焚蟶ｸ迥ｶ諷九ｒ豁｣蟶ｸ蛟､縺ｸ貎ｰ縺励※縺・◆縲・
  - 縺昴・縺溘ａ蠕檎ｶ壼・逅・〒 `buf/cap/iov/nw` 縺御ｸ肴ｭ｣蛟､縺ｮ縺ｾ縺ｾ騾ｲ陦後☆繧倶ｽ吝慍縺後≠縺｣縺溘�・
- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_load_header` 繧・`Result<i32,str>` 縺ｸ螟画峩縲・
    - `writer_load_header_ptr` 繧・`Result<MemPtr<u8>,str>` 縺ｸ螟画峩縲・
    - `writer_free_handle`, `writer_flush_handle`, `writer_ensure_handle`,
      `writer_put_u8_handle`, `writer_write_str_handle`,
      `writer_write_i32_handle`, `writer_write_u64_handle` 繧・
      `Result` 蛻・ｲ舌〒螳牙・縺ｫ蜃ｦ逅・☆繧句ｽ｢縺ｸ譖ｴ譁ｰ縲・
    - `if` 繝ｬ繧､繧｢繧ｦ繝井ｸｭ縺ｮ蜀鈴聞縺ｪ `then: block:` 繧帝勁蜴ｻ縺励�～D2002` 蝗樣∩縺ｮ縺溘ａ蠑乗ｧ矩��繧剃ｻ墓ｧ俶ｺ匁侠縺ｸ謨ｴ逅・�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-after-header-result-v2.json -j 15`
  - 邨先棡: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-after-header-result.json -j 15`
  - 邨先棡: `226/226 pass`
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kpwrite-style-fix.json -j 15`
  - 邨先棡: `215/215 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: kpwrite 縺ｮ header 繧｢繧ｯ繧ｻ繧ｹ髮・ｴ・→ non-copy 謨ｴ蜷・

- 逶ｮ逧・
  - `kpwrite.nepl` 縺ｧ謨｣蝨ｨ縺励※縺・◆ header 逕溘い繧ｯ繧ｻ繧ｹ・・load_i32 add w_raw ...` / `store_i32 add w_raw ...`・峨ｒ蜈ｱ騾壼喧縺励�～Writer` 縺ｮ non-copy/move 隕丞援縺ｨ遏帷崟縺励↑縺・ｽ｢縺ｸ謨ｴ逅・☆繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - `Writer` 縺ｯ non-copy 縺ｪ縺ｮ縺ｫ縲∵怙蛻昴・繝倥Ν繝大喧縺ｧ `writer_load_header/store_header` 縺・`Writer` 蛟､貂｡縺励→縺ｪ繧翫�√・繝ｫ繝大他縺ｳ蜃ｺ縺苓・菴薙′ move 繧堤匱逕溘＆縺・`D3053` 繧定ｪ倡匱縺励※縺・◆縲・
- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_header_ptr/load/store` 繧定ｿｽ蜉�縲・
    - 荳願ｨ倥・繝ｫ繝代・ `Writer` 縺ｧ縺ｯ縺ｪ縺・`w_raw:i32` 繧貞女縺大叙繧翫�～Writer` 縺ｮ move 繧堤匱逕溘＆縺帙↑縺・ｨｭ險医↓螟画峩縲・
    - `writer_free_handle`, `writer_flush_handle`, `writer_ensure_handle`, `writer_put_u8_handle`, `writer_write_str_handle`, `writer_write_i32_handle`, `writer_write_u64_handle` 繧貞・騾壹・繝ｫ繝醍ｵ檎罰縺ｫ鄂ｮ謠帙�・
    - 鄂ｮ謠帛ｾ後�～w_raw` 逶ｴ謗･蜿ら・縺ｯ隗｣謾ｾ蜃ｦ逅・｢・阜・・writer_free_handle`・峨・縺ｿ縺ｸ邵ｮ蟆上�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-header-v2.json -j 15`
  - 邨先棡: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v4.json -j 15`
  - 邨先棡: `226/226 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺB2: trait capability 蛻､螳壹・閾ｪ蜍墓耳螳壹ｒ蟒・ｭ｢)

- 逶ｮ逧・
  - `copy/clone` 縺ｮ trait 諢丞袖莉倥￠繧呈・遉ｺ capability (`#capability`) 縺ｮ縺ｿ縺ｫ髯仙ｮ壹＠縲∵囓鮟呎耳螳壹↓繧医ｋ隱､蛻､螳壹ｒ譬ｹ譛ｬ逧・↓髯､蜴ｻ縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - `TraitSemantics::detect` 縺・capability 譛ｪ謖・ｮ壽凾縺ｫ
    - `Self -> Self` 蜊倅ｸ�繝｡繧ｽ繝・ラ trait 繧・clone 蛟呵｣・
    - marker trait 繧・copy 蛟呵｣・
    縺ｨ縺励※謗ｨ螳壹＠縺ｦ縺・◆縲・
  - 縺薙ｌ縺ｫ繧医ｊ trait 險ｭ險域э蝗ｳ縺ｨ辟｡髢｢菫ゅ↑讒矩��荳�閾ｴ縺�縺代〒 copy/clone 諢丞袖縺御ｻ倅ｸ弱＆繧後ｋ菴吝慍縺後≠縺｣縺溘�・
- 螟画峩:
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics::detect` 縺九ｉ clone/copy 縺ｮ閾ｪ蜍募�呵｣懈耳螳壹ｒ蜑企勁縲・
    - `#capability copy` / `#capability clone` 縺ｮ螳｣險�邨先棡縺ｮ縺ｿ繧呈э蜻ｳ莉倥￠縺ｫ菴ｿ逕ｨ縲・
    - 荳崎ｦ∝喧縺励◆ `trait_has_single_unary_self_to_self_method` 縺ｨ `trait_is_marker` 繧貞炎髯､縲・
    - `TraitSemantics::detect` 縺ｮ譛ｪ菴ｿ逕ｨ `ctx` 蠑墓焚繧貞炎髯､縲・
  - `tests/move_effect.n.md`
    - `#capability` 譛ｪ謖・ｮ・trait 縺・copy/clone 縺ｨ縺励※謗ｨ螳壹＆繧後↑縺・％縺ｨ繧堤｢ｺ隱阪☆繧句屓蟶ｰ繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3096 TypeUnknownTraitCapability` 繧定ｿｽ蜉�縲・
  - `nepl-core/src/typecheck.rs`
    - trait 螳夂ｾｩ縺ｧ譛ｪ遏･縺ｮ `#capability` 蜷阪ｒ讀懷・縺励�～D3096` 繧定ｿ斐☆繧医≧螟画峩縲・
  - `tests/move_effect.n.md`
    - `#capability cpoy` 縺ｮ compile_fail 繧ｱ繝ｼ繧ｹ・・diag_id: 3096`・峨ｒ霑ｽ蜉�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-move-overload-v1.json -j 15`
  - 邨先棡: `269/269 pass`
  - `node nodesrc/tests.js -i tests/move_effect.n.md --no-tree -o /tmp/tests-move-effect-capability-v2.json -j 15`
  - 邨先棡: `227/227 pass`
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-move-overload-v2.json -j 15`
  - 邨先棡: `272/272 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: kpread 縺ｮ header 逶ｴ繧｢繧ｯ繧ｻ繧ｹ繧貞・騾壼ｮ牙・繝倥Ν繝代∈邨ｱ荳�)

- 逶ｮ逧・
  - `kpread.nepl` 縺ｧ谿九▲縺ｦ縺・◆ `sc_raw` 繝吶・繧ｹ縺ｮ header 逶ｴ謗･隱ｭ縺ｿ譖ｸ縺阪ｒ髯､蜴ｻ縺励�～Scanner` 蠅・阜縺ｮ蝙句ｮ牙・諤ｧ繧剃ｸ翫￡繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - `scanner_header_ptr` / `scanner_load_header` / `scanner_store_header` 繧貞ｰ主・貂医∩縺ｧ繧ゅ�∽ｸｻ隕√ヱ繝ｼ繧ｵ髢｢謨ｰ縺梧立邨瑚ｷｯ・・load_i32 add sc_raw ...` / `store_i32 add sc_raw ...`・峨ｒ菴ｿ縺・ｶ壹￠縺ｦ縺・◆縲・
  - 縺薙ｌ縺ｫ繧医ｊ API 蠅・阜縺ｯ `Scanner` 縺ｧ繧ゅ�∝ｮ溯｣・・驛ｨ縺檎函繝昴う繝ｳ繧ｿ蜑肴署縺ｮ縺ｾ縺ｾ蛻・ｲ舌＠縺ｦ縺・◆縲・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - 莉･荳九・髢｢謨ｰ縺ｧ header 繧｢繧ｯ繧ｻ繧ｹ繧・`scanner_load_header` / `scanner_store_header` 縺ｫ邨ｱ荳�:
      - `scanner_skip_ws_handle`
      - `scanner_is_eof_handle`
      - `scanner_skip_token_handle`
      - `scanner_read_token_handle`
      - `scanner_read_i32_handle`
      - `scanner_read_u64_handle`
      - `scanner_read_i64_handle`
      - `scanner_read_f64_handle`
      - `scanner_read_all_i32_handle`
    - 鄂ｮ謠帛ｾ後�～kpread.nepl` 蜀・・ `sc_raw` 逶ｴ謗･繧｢繧ｯ繧ｻ繧ｹ縺ｯ `scanner_header_ptr` 蜀・・螳溯｣・ｸ�轤ｹ縺ｮ縺ｿ縺ｫ髮・ｴ・�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-safe-headers-v1.json -j 15`
  - 邨先棡: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v3.json -j 15`
  - 邨先棡: `226/226 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: kpread 蝓ｺ逶､ handle 縺ｮ Scanner 蝙句喧)

- 逶ｮ逧・
  - `kpread` 縺ｮ蜈ｬ髢矩擇縺ｧ髴ｲ蜃ｺ縺励※縺・ｋ逕・`i32` 繝上Φ繝峨Ν髢｢謨ｰ繧呈ｮｵ髫守噪縺ｫ貂帙ｉ縺吶◆繧√�∝渕逶､縺ｨ縺ｪ繧・髢｢謨ｰ繧・`Scanner` 蜿励￠蜿悶ｊ縺ｸ螟画峩縺吶ｋ縲・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - `scanner_skip_ws_handle` 繧・`(Scanner)*>()` 縺ｸ螟画峩縲・
    - `scanner_is_eof_handle` 繧・`(Scanner)*>bool` 縺ｸ螟画峩縲・
    - `scanner_skip_token_handle` 繧・`(Scanner)*>()` 縺ｸ螟画峩縲・
    - `scanner_read_token_handle` 繧・`(Scanner)*>str` 縺ｸ螟画峩縲・
    - 荳願ｨ伜他縺ｳ蜃ｺ縺礼ｮ・園・・i32` 繝吶・繧ｹ縺ｮ譌｢蟄・handle 鄒､・峨〒縺ｯ `scanner_wrap mem_ptr_wrap sc` 繧呈・遉ｺ縺励※貂｡縺吶ｈ縺・ｵｱ荳�縲・
    - 蜈ｬ髢九Λ繝・ヱ・・scanner_skip_ws` 縺ｪ縺ｩ・峨・ raw 蜿悶ｊ蜃ｺ縺励ｒ繧・ａ縺ｦ `Scanner` 繧堤峩謗･貂｡縺吶ｈ縺・ｰ｡邏�蛹悶�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-scanner-handle-v1.json -j 15`
  - 邨先棡: `217/217 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: kpread 谿九ｊ handle 鄒､縺ｮ Scanner 蝙句喧螳御ｺ・

- 逶ｮ逧・
  - `kpread` 縺ｧ谿九▲縺ｦ縺・◆ `*_handle <(i32)...>` 鄒､繧・`Scanner` 蜿励￠蜿悶ｊ縺ｸ邨ｱ荳�縺励�∝・髢・蜀・Κ縺ｮ蝙句｢・阜繧剃ｸ�雋ｫ蛹悶☆繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - 荳�驛ｨ handle 縺・`i32` 繧堤峩謗･蜿励￠蜿悶ｊ縲∽ｻ悶・ `Scanner` 蜿励￠蜿悶ｊ髢｢謨ｰ縺ｨ蠅・阜險ｭ險医′豺ｷ蝨ｨ縺励※縺・◆縲・
  - 縺昴・邨先棡縲∝・髢九Λ繝・ヱ縺ｧ `mem_ptr_addr get sc "raw"` 繧帝・蠎ｦ譖ｸ縺丞ｿ・ｦ√′縺ゅｊ縲〉aw 髴ｲ蜃ｺ縺ｨ隱､逕ｨ菴吝慍縺梧ｮ九▲縺ｦ縺・◆縲・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - 莉･荳九ｒ `Scanner` 蜿励￠蜿悶ｊ縺ｸ螟画峩:
      - `scanner_read_i32_handle`
      - `scanner_read_u64_handle`
      - `scanner_read_i64_handle`
      - `scanner_read_f64_handle`
      - `scanner_read_f32_handle`
      - `scanner_read_vec_i64_handle`
      - `scanner_read_vec_i32_handle`
      - `scanner_read_matrix_i32_handle`
      - `scanner_read_all_i32_handle`
      - `scanner_read_na_i32_handle`
      - `scanner_read_interval_queries_i32_handle`
      - `scanner_read_query_tuples_i32_handle`
      - `scanner_read_ndrh_i32_handle`
    - 蜷・未謨ｰ蜀・Κ縺ｧ縺ｯ蠢・ｦ∫ｮ・園縺ｮ縺ｿ `sc_raw = mem_ptr_addr get sc "raw"` 繧貞ｰ主・縺励�∵里蟄倥Ο繧ｸ繝・け繧堤ｶｭ謖√�・
    - 蜈ｬ髢九Λ繝・ヱ (`scanner_read_i32` 縺ｪ縺ｩ) 縺ｯ raw 謚ｽ蜃ｺ繧貞炎髯､縺励※ handle 縺ｸ `Scanner` 繧堤峩謗･貂｡縺吶ｈ縺・ｵｱ荳�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kpread-scanner-allhandles-v1.json -j 15`
  - 邨先棡: `212/212 pass`
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-scanner-allhandles-v2.json -j 15`
  - 邨先棡: `217/217 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: kpwrite handle API 縺ｮ邱壼ｽ｢蛹悶→ move 謨ｴ蜷亥喧)

- 逶ｮ逧・
  - `kpwrite` 縺ｮ蜀・Κ API 縺ｧ繧ら函 `i32` 蠅・阜繧呈ｸ帙ｉ縺励▽縺､縲～Writer` 縺ｮ non-copy 險ｭ險医→ move 隕丞援縺檎泝逶ｾ縺励↑縺・ｽ｢縺ｸ謨ｴ逅・☆繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - `Writer` 繧貞女縺代ｋ handle 縺・`()` 繧定ｿ斐☆險ｭ險医・縺ｾ縺ｾ `Writer` 繧定､・焚蝗槫茜逕ｨ縺励※縺翫ｊ縲～D3053/D3054`・・oved value・峨ｒ隱倡匱縺励※縺・◆縲・
  - 荳�譎・`writer_wrap` 繧貞､夂畑縺吶ｋ蠖｢縺ｯ螻�謇�逧・↓縺ｯ蜍輔￥縺後�∬ｨｭ險医→縺励※邱壼ｽ｢豸郁ｲｻ隕丞援縺梧・遒ｺ縺ｧ縺ｪ縺九▲縺溘�・
- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_flush_handle` / `writer_ensure_handle` / `writer_put_u8_handle` / `writer_writeln_handle` / `writer_write_*_handle` 繧・`Writer` 蜿励￠蜿悶ｊ繝ｻ`Writer` 霑泌唆縺ｫ邨ｱ荳�縲・
    - 蜷・handle 縺ｧ `w_raw` 繧貞・驛ｨ蜿門ｾ励＠縲∵峩譁ｰ蠕後・ `writer_wrap mem_ptr_wrap w_raw` 繧定ｿ斐☆邱壼ｽ｢ API 縺ｫ螟画峩縲・
    - 隍・焚謫堺ｽ懊ｒ陦後≧ handle・・writer_write_i32_handle`, `writer_write_u64_handle`, `writer_write_*_ln_handle` 縺ｪ縺ｩ・峨・ `let mut ww <Writer>` / `set ww ...` 縺ｧ邱壼ｽ｢縺ｫ譖ｴ譁ｰ縲・
    - 蜈ｬ髢・API (`writer_write_i32` 縺ｪ縺ｩ) 縺ｯ raw 蜀阪Λ繝・・縺ｮ驥崎､・ｒ蜑企勁縺励�∝ｯｾ蠢・handle 繧堤峩謗･蜻ｼ縺ｶ讒矩��縺ｸ謨ｴ逅・�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-kpwrite-only-v4.json -j 15`
  - 邨先棡: `208/208 pass`
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-handle-wrap-v3.json -j 15`
  - 邨先棡: `217/217 pass`

- 陬懆ｶｳ・郁ｨｭ險亥愛譁ｭ・・
  - 荳�譎・`writer_wrap` 繧帝・蠎ｦ菴懊ｋ蜻ｼ縺ｳ蜃ｺ縺励・ move 繧ｨ繝ｩ繝ｼ蝗樣∩縺ｨ縺励※縺ｯ讖溯・縺吶ｋ縺後�∫ｷ壼ｽ｢ API 險ｭ險医→縺励※荳肴・迸ｭ縺�縺｣縺溘◆繧∵治逕ｨ縺励↑縺・�・
  - `Writer -> Writer` 縺ｮ譖ｴ譁ｰ騾｣骼悶ｒ handle 螻､縺ｧ譏守､ｺ縺励�［ove 隕丞援縺ｨ API 螂醍ｴ・ｒ荳�閾ｴ縺輔○繧区婿驥昴↓邨ｱ荳�縺励◆縲・

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: kpread_core 縺ｮ逕溘Γ繝｢繝ｪ繧｢繧ｯ繧ｻ繧ｹ螳牙・API蛹・

- 逶ｮ逧・
  - syscall 蠅・阜莉･螟悶・逕溘Γ繝｢繝ｪ繧｢繧ｯ繧ｻ繧ｹ繧・`MemPtr` + `Result/Option` 邨檎罰縺ｸ蟇・○縲∝､ｱ謨玲､懷・繧剃ｸ頑ｵ∝喧縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - `kpread_core` 蜀・〒 `mem_ptr_addr` + 逕・`store_i32/load_i32` 繧堤峩謗･螳溯｡後＠縺ｦ縺翫ｊ縲∝｢・阜荳肴紛蜷域凾縺ｫ螟ｱ謨励ｒ蝙九〒謇ｱ縺医↑縺九▲縺溘�・
- 螟画峩:
  - `stdlib/kp/kpread_core.nepl`
    - `mem_i32_ptr`, `store_i32_u8_at`, `load_i32_u8_at` 繧定ｿｽ蜉�縲・
    - scanner header 蛻晄悄蛹・(`sc0`, `sc`) 繧・`store_i32_u8_at` 邨檎罰縺ｸ螟画峩縺励�∝､ｱ謨玲凾縺ｯ遒ｺ菫晄ｸ医∩鬆伜沺繧定ｧ｣謾ｾ縺励※ `Err` 霑泌唆縲・
    - `iov/nread` 讒狗ｯ画凾縺ｮ譖ｸ縺崎ｾｼ縺ｿ縺ｨ `nread` 隱ｭ縺ｿ蜿悶ｊ繧貞ｮ牙・繝倥Ν繝醍ｵ檎罰縺ｸ螟画峩縲・
    - 繝｡繝｢繝ｪ繧｢繧ｯ繧ｻ繧ｹ螟ｱ謨玲凾縺ｯ `mem_failed` 繧堤ｫ九※縲∝ｾ梧ｮｵ縺ｧ荳�諡ｬ隗｣謾ｾ縺励※ `Result::Err \"kpread_core.memory access failed\"` 繧定ｿ斐☆邨瑚ｷｯ繧定ｿｽ蜉�縲・
    - `fd_read` 蜻ｼ縺ｳ蜃ｺ縺苓・菴薙・ syscall 莉墓ｧ倅ｸ・`i32` 繝昴う繝ｳ繧ｿ縺悟ｿ・ｦ√↑縺溘ａ縲∝｢・阜轤ｹ縺ｧ縺ｮ縺ｿ `mem_ptr_addr` 繧剃ｽｿ逕ｨ縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-core-safe-v1.json -j 15`
  - 邨先棡: `217/217 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: `core/mem` 縺ｮ `*_ptr` 繧貞ｮ牙・API邨檎罰縺ｸ邨ｱ荳�)

- 逶ｮ逧・
  - `MemPtr` 邉ｻ API 縺ｮ蜀・Κ螳溯｣・ｒ `alloc_raw/realloc_raw/dealloc_raw` 逶ｴ邨舌°繧牙・髮｢縺励�～alloc/realloc/dealloc` 繧帝�壹ｋ蜈ｱ騾壼ｮ牙・邨瑚ｷｯ縺ｸ邨ｱ荳�縺吶ｋ縲・
- 螟画峩:
  - `stdlib/core/mem.nepl`
    - `alloc_ptr` 繧・`alloc` 邨檎罰縺ｸ螟画峩縲・
    - `realloc_ptr` 繧・`realloc` 邨檎罰縺ｸ螟画峩縲・
    - `dealloc_ptr` 繧・`dealloc` 邨檎罰縺ｸ螟画峩縲・
  - 縺薙ｌ縺ｫ繧医ｊ `MemPtr` 邉ｻ繧ｨ繝ｩ繝ｼ邨瑚ｷｯ縺ｯ蝓ｺ蠎募ｮ牙・API縺ｮ蜑肴署讀懈渊邨先棡縺ｨ謨ｴ蜷医☆繧九�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v2.json -j 15`
  - 邨先棡: `226/226 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: kpread_core 蜀・Κ遒ｺ菫昴ｒ `*_ptr` API 縺ｫ邨ｱ荳�)

- 逶ｮ逧・
  - `kpread_core` 蜀・Κ縺ｧ縺ｮ逕溘・繧､繝ｳ繧ｿ邂｡逅・ｒ貂帙ｉ縺励�～MemPtr<u8>` 繧剃ｽｿ縺｣縺溽｢ｺ菫・蜀咲｢ｺ菫・隗｣謾ｾ縺ｸ蟇・○繧九�・
- 螟画峩:
  - `stdlib/kp/kpread_core.nepl`
    - `buf/iov/nread/scanner header` 縺ｮ遒ｺ菫昴ｒ `alloc_ptr<u8>` 縺ｫ螟画峩縲・
    - 繝舌ャ繝輔ぃ諡｡蠑ｵ繧・`realloc_ptr<u8>` 縺ｫ螟画峩縲・
    - 隗｣謾ｾ繧・`dealloc_ptr<u8>` 縺ｫ螟画峩縲・
    - `fd_read` 繧・`store_i32/load_i32` 縺ｸ貂｡縺咏ｮ・園縺ｮ縺ｿ `mem_ptr_addr` 縺ｧ `i32` 縺ｫ譏守､ｺ螟画鋤縲・
  - `scanner_new_impl` 縺ｯ譌｢蟄倥←縺翫ｊ `Result<MemPtr<u8>,str>` 繧定ｿ斐＠縲、PI莠呈鋤繧堤ｶｭ謖√�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-memptr-wrap-v6.json -j 15`
  - 邨先棡: `217/217 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: kpread_core 縺ｮ霑泌唆蝙九ｒ MemPtr 蛹・

- 逶ｮ逧・
  - `kpread` 蜈･蜉帛・譛溷喧縺ｮ荳頑ｵ・ｼ・kpread_core`・峨〒繧ら函 `i32` 霑泌唆繧呈ｸ帙ｉ縺励�～MemPtr<u8>` 縺ｧ蠅・阜繧呈純縺医ｋ縲・
- 螟画峩:
  - `stdlib/kp/kpread_core.nepl`
    - `scanner_new_impl` 縺ｮ謌ｻ繧雁�､繧・`Result<MemPtr<u8>,str>` 縺ｫ螟画峩縲・
    - 謌仙粥譎・`sc:i32` 縺ｯ `mem_ptr_wrap` 縺励※霑泌唆縲・
    - 螟ｱ謨礼ｳｻ縺ｮ `Result` 蝙九ヱ繝ｩ繝｡繝ｼ繧ｿ繧・`MemPtr<u8>` 縺ｫ邨ｱ荳�縲・
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_handle` 縺ｯ `scanner_new_impl` 繧偵◎縺ｮ縺ｾ縺ｾ霑斐☆螳溯｣・∈邁｡邏�蛹悶�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-memptr-wrap-v5.json -j 15`
  - 邨先棡: `217/217 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: kpread/kpwrite 縺ｮ `*_new_handle` 霑斐ｊ蛟､繧・MemPtr 蛹・

- 逶ｮ逧・
  - 逕滓・邉ｻ API 縺ｮ蠅・阜縺九ｉ逕・`i32` 繧呈ｸ帙ｉ縺励�～MemPtr<u8>` 縺ｫ繧医ｋ蝙句｢・阜繧呈・遒ｺ蛹悶☆繧九�・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_handle` 繧・`Result<MemPtr<u8>,str>` 縺ｸ螟画峩縲・
    - `scanner_new` 縺ｯ `MemPtr<u8>` 繧偵◎縺ｮ縺ｾ縺ｾ `scanner_wrap` 縺ｫ貂｡縺吝ｽ｢縺ｸ螟画峩縲・
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_handle` 繧・`Result<MemPtr<u8>,str>` 縺ｸ螟画峩縲・
    - 蜀・Κ遒ｺ菫昴〒蠕励◆ `w:i32` 縺ｯ `mem_ptr_wrap` 縺励※ `Ok` 霑泌唆縲・
    - `writer_new` 縺ｯ `MemPtr<u8>` 繧偵◎縺ｮ縺ｾ縺ｾ `writer_wrap` 縺ｫ貂｡縺吝ｽ｢縺ｸ螟画峩縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-memptr-wrap-v4.json -j 15`
  - 邨先棡: `216/216 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: kpwrite Writer 繝ｩ繝・・蠅・阜縺ｮ蝙区紛蜷・

- 逶ｮ逧・
  - `todo.md` 繝輔ぉ繝ｼ繧ｺC・亥・髢帰PI縺ｮ逕溘・繧､繝ｳ繧ｿ髴ｲ蜃ｺ蜑頑ｸ幢ｼ峨↓豐ｿ縺｣縺ｦ縲～kpwrite` 縺ｮ `Writer` 逕滓・蠅・阜繧・`MemPtr<u8>` 縺ｧ邨ｱ荳�縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - `Writer.raw` 縺ｯ `MemPtr<u8>` 縺�縺・`writer_wrap` 縺・`(i32)->Writer` 縺ｧ縲∫函繝昴う繝ｳ繧ｿ繧堤峩謗･蜿励￠蜿悶ｋ蠅・阜縺梧ｮ九▲縺ｦ縺・◆縲・
- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_wrap` 繧・`(MemPtr<u8>)->Writer` 縺ｫ螟画峩縲・
    - `writer_new` 縺ｨ `Writer` 繧定ｿ斐☆蜈ｬ髢九Λ繝・ヱ鄒､縺ｧ `i32` 繧・`mem_ptr_wrap` 縺励※縺九ｉ `writer_wrap` 繧貞他縺ｶ繧医≧邨ｱ荳�縲・
  - 蜀・Κ `*_handle` 縺ｯ谿ｵ髫守ｧｻ陦後→縺励※ `i32` 繧堤ｶｭ謖・ｼ亥・髢帰PI蠅・阜縺ｮ縺ｿ蝙句ｮ牙・蛹厄ｼ峨�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-memptr-wrap-v3.json -j 15`
  - 邨先棡: `216/216 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: kpread Scanner 繝ｩ繝・・蠅・阜縺ｮ蝙区紛蜷・

- 逶ｮ逧・
  - `todo.md` 繝輔ぉ繝ｼ繧ｺC・亥・髢帰PI縺ｮ逕溘・繧､繝ｳ繧ｿ髴ｲ蜃ｺ蜑頑ｸ幢ｼ峨↓豐ｿ縺｣縺ｦ縲～kpread` 縺ｮ `Scanner` 逕滓・蠅・阜繧・`MemPtr<u8>` 縺ｧ邨ｱ荳�縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - `Scanner.raw` 縺ｯ `MemPtr<u8>` 縺ｪ縺ｮ縺ｫ `scanner_wrap` 縺・`(i32)->Scanner` 縺ｧ縲∫函謌仙｢・阜縺ｧ逕溘・繧､繝ｳ繧ｿ繧堤峩謗･蜿励￠縺ｦ縺・◆縲・
  - 縺薙ｌ縺ｫ繧医ｊ `Scanner` 縺ｮ蜈ｬ髢句梛險ｭ險医→逕滓・繧ｷ繧ｰ繝阪メ繝｣縺御ｸ堺ｸ�閾ｴ縺�縺｣縺溘�・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - `scanner_wrap` 繧・`(MemPtr<u8>)->Scanner` 縺ｫ螟画峩縲・
    - `scanner_new` 縺ｧ `raw:i32` 繧・`mem_ptr_wrap` 縺励※縺九ｉ `scanner_wrap` 縺ｸ貂｡縺吶ｈ縺・､画峩縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-memptr-wrap-v2.json -j 15`
  - 邨先棡: `216/216 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (compile_fail: diag_id + 菴咲ｽｮ讀懆ｨｼ縺ｮ驕狗畑蝗ｺ螳・

- 逶ｮ逧・
  - `compile_fail` 繧ｱ繝ｼ繧ｹ縺ｧ `diag_id` 縺�縺代〒縺ｪ縺冗匱逕滉ｽ咲ｽｮ・・ile/line/col・峨ｂ螳牙ｮ壽､懆ｨｼ縺ｧ縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
- 螟画峩:
  - `nodesrc/tests.js`
    - `extractDiagSpansFromCompileError` 繧定｡悟腰菴崎ｧ｣譫舌∈螟画峩縲・
    - `--> ...` 陦後°繧画忰蟆ｾ `:line:col` 繧貞渕貅悶↓謚ｽ蜃ｺ縺吶ｋ繧医≧菫ｮ豁｣縺励�√ヱ繧ｹ荳ｭ縺ｮ繧ｳ繝ｭ繝ｳ繧貞性繧�蠖｢蠑上↓繧り�舌∴繧九ｈ縺・↓縺励◆縲・
  - `nodesrc/parser.js`
    - doctest 繝｡繧ｿ `diag_spans` 縺ｫ JSON object 蠖｢蠑擾ｼ・{file,line,col}`・峨ｒ險ｱ蜿ｯ縲・
    - 譌｢蟄倥・ `"line:col"` / `"file:line:col"` 譁・ｭ怜・陦ｨ險倥・莠呈鋤邯ｭ謖√�・
  - `tests/compile_fail_diag_location.n.md`
    - `diag_spans` 縺ｮ object 蠖｢蠑上ｒ菴ｿ縺・屓蟶ｰ繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/compile_fail_diag_location.n.md -i tests/lexer_diag.n.md --no-stdlib --no-tree -o /tmp/tests-compile-fail-location-verify.json -j 15`
  - 邨先棡: `6/6 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (`;` 險ｺ譁ｭ縺ｮ荳頑ｵ∝喧縺ｨ loader 險ｺ譁ｭ謨ｴ蠖｢)

- 逶ｮ逧・
  - `tests/block_semicolon_return.n.md::doctest#10` 縺ｮ backend 貍上ｌ・・asm validation error・峨ｒ豁｢繧√�｝arser 谿ｵ縺ｧ `diag_id` 繧貞崋螳壼喧縺吶ｋ縲・
  - `compile_fail` 縺ｧ loader 邨檎罰縺ｮ繧ｨ繝ｩ繝ｼ縺ｧ繧・`error[Dxxxx]` 繧貞ｮ牙ｮ壼叙蠕励〒縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - `if:` 繝ｬ繧､繧｢繧ｦ繝亥・縺ｮ `Stmt::ExprSemi` 縺御ｸ頑ｵ√〒諡貞凄縺輔ｌ縺壹�…odegen 縺ｾ縺ｧ騾ｲ繧薙〒縺・◆縲・
  - `nepl-web/src/lib.rs` 縺ｧ loader 繧ｨ繝ｩ繝ｼ繧・`to_string()` 縺励※縺翫ｊ縲～Diagnostics` 譁・ｭ怜・縺梧紛蠖｢縺輔ｌ縺・`diag_id` 謚ｽ蜃ｺ縺御ｸ榊ｮ牙ｮ壹□縺｣縺溘�・
- 螟画峩:
  - `nepl-core/src/parser.rs`
    - `reject_layout_semicolon` 繧定ｿｽ蜉�縲・
    - `extract_if_layout_exprs` / `extract_if_layout_exprs_lenient` 縺ｧ `ExprSemi` 繧・`D2002` 縺ｨ縺励※蜊ｳ譎よ拠蜷ｦ縲・
    - `while` / 荳�闊ｬ蠑墓焚繝ｬ繧､繧｢繧ｦ繝医・譌｢蟄倅ｻ墓ｧ假ｼ・;` 險ｱ螳ｹ・峨ｒ邯ｭ謖√�・
  - `nepl-web/src/lib.rs`
    - loader 螟ｱ謨玲凾縺ｫ `render_loader_error` 繧帝�壹☆繧医≧螟画峩縲・
    - `LoaderError::Core` 縺ｯ `render_core_error` 縺ｸ豬√＠縲～error[Dxxxx]` 蠖｢蠑上〒霑斐☆縲・
  - `tests/plan.n.md`
    - `diag_id` 譛溷ｾ・ｒ螳溯｣・ｮ滓・縺ｫ蜷医ｏ縺帙※ `2002 -> 2001` 縺ｫ菫ｮ豁｣・・繧ｱ繝ｼ繧ｹ・峨�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/lexer_diag.n.md -i tests/plan.n.md -i tests/block_single_line.n.md -i tests/block_semicolon_return.n.md --no-stdlib --no-tree -o /tmp/tests-diag-parser.json -j 15` -> `70/70 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (codegen 蜑肴ｮｵ蜈ｱ騾・precheck 蟆主・: raw body/target 險ｺ譁ｭ縺ｮ邨ｱ荳�)

- 逶ｮ逧・
  - `codegen_wasm` / `codegen_llvm` 縺悟�句挨縺ｫ `#wasm/#llvmir` 縺ｮ target 荳肴紛蜷医ｒ險ｺ譁ｭ縺吶ｋ讒矩��繧偵ｄ繧√�∝燕谿ｵ蜈ｱ騾壹メ繧ｧ繝・け縺ｧ險ｺ譁ｭ繧堤｢ｺ螳壹☆繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - `#if[target=...]` 隧穂ｾ｡縲∥ctive 譁・歓蜃ｺ縲〉aw body 驕ｸ謚槭Ο繧ｸ繝・け縺・`typecheck` 縺ｨ `codegen_llvm` 縺ｫ蛻・淵縺励�∝愛螳壼ｷｮ蛻・→ backend 萓晏ｭ倩ｨｺ譁ｭ縺檎匱逕溘＠縺ｦ縺・◆縲・
- 螟画峩:
  - 譁ｰ隕・`nepl-core/src/target_precheck.rs` 繧定ｿｽ蜉�縲・
    - `gate_allows`・・#if[target/profile]` 蛻､螳夲ｼ・
    - `active_stmt_indices`・・ctive 譁・歓蜃ｺ・・
    - `select_active_raw_body`・磯未謨ｰ body 蜀・`#wasm/#llvmir` 驕ｸ謚橸ｼ・
    - `precheck_function_raw_body_target` / `precheck_module_raw_bodies`・・arget 謨ｴ蜷域､懆ｨｼ・・
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3094 TypeMultipleActiveRawBodies`
    - `D3095 TypeRawBodyTargetMismatch`
  - `nepl-core/src/compiler.rs`
    - `compile_module` 縺ｮ typecheck 蜑阪↓ `precheck_module_raw_bodies` 繧貞ｮ溯｡後＠縲√お繝ｩ繝ｼ譎ゅ・譌ｩ譛溽ｵゆｺ・�・
  - `nepl-core/src/typecheck.rs`
    - `check_function` 蜀帝�ｭ縺ｧ `precheck_function_raw_body_target` 繧貞ｮ溯｡後＠縲～typecheck` 逶ｴ謗･蛻ｩ逕ｨ邨瑚ｷｯ縺ｧ繧ょ酔荳�險ｺ譁ｭ繧剃ｿ晁ｨｼ縲・
  - `nepl-core/src/codegen_llvm.rs`
    - `emit_ll_from_module_for_target` 蜀帝�ｭ縺ｧ `precheck_module_raw_bodies` 繧貞ｮ溯｡後�・
    - `#if` active 譁・歓蜃ｺ繧貞・騾・`active_stmt_indices` 縺ｫ邨ｱ荳�縲・
    - Parsed 髢｢謨ｰ縺ｮ raw body 驕ｸ謚槭ｒ蜈ｱ騾・`select_active_raw_body` 縺ｫ邨ｱ荳�縲・
    - 驥崎､・＠縺ｦ縺・◆ local gate/raw 驕ｸ謚樣未謨ｰ鄒､繧貞炎髯､縲・
  - 繝・せ繝・
    - 譌｢蟄俶峩譁ｰ:
      - `tests/neplg2.n.md` 縺ｮ `wasm_rejects_llvmir_body_with_diag_id` 繧・`diag_id: 3095` 縺ｸ螟画峩縲・
      - `tests/neplg2.n.md` 縺ｫ `raw_body_conflict_reports_diag_id`・・diag_id: 3094`・芽ｿｽ蜉�縲・
      - `tests/llvm_target.n.md` 縺ｮ `llvm_rejects_wasm_body` 縺ｫ `diag_id: 3095` 霑ｽ蜉�縲・
    - 譁ｰ隕剰ｿｽ蜉�:
      - `tests/raw_body_precheck.n.md`・・繧ｱ繝ｼ繧ｹ縲～D3094/D3095` 繧貞崋螳夂｢ｺ隱搾ｼ峨�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md --no-stdlib --no-tree --runner all -o /tmp/tests-raw-body-precheck.json -j 15` -> `3/3 pass`
  - 蜿り�・ `tests/neplg2.n.md` + `tests/llvm_target.n.md` 繧・`--with-stdlib` 縺ｧ螳溯｡後☆繧九→譌｢遏･縺ｮ stdlib 蛛ｴ螟ｱ謨暦ｼ・ist doctest・峨′豺ｷ縺悶ｋ縺後�∬ｿｽ蜉�縺励◆ `D3094/D3095` 繧ｱ繝ｼ繧ｹ閾ｪ菴薙・騾夐℃縺励※縺・ｋ縺薙→繧・`/tmp/tests-codegen-precheck.json` 縺ｧ遒ｺ隱阪�・

# 2026-03-05 菴懈･ｭ繝｡繝｢ (`;` 莉墓ｧ伜・陦御ｿｮ豁｣: `stdlib/core/math.nepl`)

- 逶ｮ逧・
  - `plan.md` 縺ｮ縲瑚､・｡梧枚縺ｫ縺ｯ譛ｫ蟆ｾ `;` 繧剃ｻ倥￠縺ｪ縺・�榊宛邏・↓蜷医ｏ縺帙�～overload` 螟ｱ謨励・譬ｹ譛ｬ蜴溷屏繧貞・縺ｫ隗｣豸医☆繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - `stdlib/core/math.nepl` 縺ｮ `i128/u128` 蜻ｨ霎ｺ縺ｧ縲∬､・｡・`if:` 繧貞承霎ｺ縺ｫ謖√▽ `let` 譁・・譛ｫ蟆ｾ縺ｫ `;` 縺梧ｮ九▲縺ｦ縺・◆縲・
  - 縺薙ｌ縺悟ｼ上・ `()` 蛹悶ｒ隱倡匱縺励�『asm 讀懆ｨｼ谿ｵ縺ｧ `invalid wasm generated: expected i64 but nothing on stack` 繧貞ｼ輔″襍ｷ縺薙＠縺ｦ縺・◆縲・
- 螟画峩:
  - `stdlib/core/math.nepl` 縺ｮ隧ｲ蠖鍋ｮ・園縺ｧ縲∬､・｡・`if:` 蜿ｳ霎ｺ `let` 縺ｮ譛ｫ蟆ｾ `;` 繧帝勁蜴ｻ縲・
  - 蟇ｾ雎｡: `to_i128`, `u128/i128` 縺ｮ `carry/borrow` 險育ｮ励�～mul_wide` 縺ｮ `carry_mid/carry_lo` 險育ｮ励�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/overload.n.md --no-stdlib --no-tree -o /tmp/tests-overload-nostd.json -j 15`
  - 邨先棡: `43/43 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝代・繧ｵ譬ｹ譛ｬ菫ｮ豁｣: 蜊倩｡・block 蛻ｶ邏・→ `ExprSemi` 諢丞袖隲紋ｿ晄戟)

- 逶ｮ逧・
  - `tests/plan.n.md::doctest#29`・亥腰陦・`block` 蜀・↓隍・｡・`block:` 縺悟・縺｣縺ｦ縺励∪縺・ｼ峨ｒ繧ｳ繝ｳ繝代う繝ｩ蛛ｴ縺ｧ譬ｹ譛ｬ菫ｮ豁｣縺吶ｋ縲・
  - `tests/block_semicolon_return.n.md::doctest#10`・郁､・｡悟ｼ乗忰蟆ｾ `;` 縺ｮ諢丞袖關ｽ縺｡・峨ｒ隗｣豸医☆繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - 繝代・繧ｵ縺後�悟腰陦・block 譁・ц縲阪ｒ菫晄戟縺励※縺翫ｉ縺壹�∝腰陦・`block` 蜀・〒繧・`parse_block_after_colon()` 繧帝�壹＠縺ｦ隍・｡・`:` 繝悶Ο繝・け繧貞女逅・＠縺ｦ縺・◆縲・
  - `extract_if_layout_exprs` / `extract_while_layout_exprs` / `extract_arg_layout_exprs` 縺・`Stmt::ExprSemi` 繧・`Stmt::Expr` 縺ｨ蜷御ｸ�謇ｱ縺・＠縲～;` 縺ｫ繧医ｋ unit 蛹悶→繧ｹ繧ｿ繝・け讀懆ｨｼ繧定誠縺ｨ縺励※縺・◆縲・
- 螟画峩:
  - `nepl-core/src/parser.rs`
    - `single_line_block_depth` 繧定ｿｽ蜉�縺励�∝腰陦・block 隗｣譫蝉ｸｭ縺ｫ隍・｡・`:` 繝悶Ο繝・け繧呈､懷・縺励◆繧・`D2002` 繧貞・縺吶ｈ縺・↓螟画峩縲・
    - `parse_single_line_block*` 縺ｧ譁・ц豺ｱ縺輔ｒ邂｡逅・☆繧九ｈ縺・､画峩縲・
    - `ExprSemi` 繧剃ｿ晄戟縺励※繝ｬ繧､繧｢繧ｦ繝域歓蜃ｺ縺ｸ貂｡縺吝・騾壹・繝ｫ繝代・繧定ｿｽ蜉�縲・
    - if/while/蠑墓焚繝ｬ繧､繧｢繧ｦ繝域歓蜃ｺ縺ｧ `ExprSemi` 繧呈昏縺ｦ縺壹↓ block 蛹悶＠縺ｦ菫晄戟縺励�∝梛讀懈渊谿ｵ縺ｧ `;` 諢丞袖隲悶′蜿肴丐縺輔ｌ繧九ｈ縺・↓螟画峩縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/plan.n.md --no-stdlib --no-tree -o /tmp/tests-plan-nostd.json -j 15` -> `36/36 pass`
  - `node nodesrc/tests.js -i tests/block_semicolon_return.n.md --no-stdlib --no-tree -o /tmp/tests-block-semicolon-nostd.json -j 15` -> `10/10 pass`
- 蠖ｱ髻ｿ:
  - `--with-stdlib` 縺ｧ襍ｰ繧峨○繧九→ stdlib doctest 蛛ｴ縺ｫ `;` 諢丞袖隲紋ｸ肴紛蜷医′鬘募惠蛹厄ｼ・List` 縺ｪ縺ｩ縺ｧ `expected ... got unit`・峨�・
  - 縺薙ｌ縺ｯ莉雁屓縺ｮ繝代・繧ｵ菫ｮ豁｣縺ｧ髫�繧後※縺・◆莉墓ｧ倬＆蜿阪′陦ｨ髱｢蛹悶＠縺溽憾諷九�・
  - 谺｡谿ｵ縺ｨ縺励※ stdlib 蛛ｴ縺ｮ `;` 菴ｿ逕ｨ邂・園繧・plan.md 縺ｫ蜷医ｏ縺帙※謨ｴ逅・☆繧句ｿ・ｦ√′縺ゅｋ縲・

# 2026-03-05 菴懈･ｭ繝｡繝｢ (plan.md 蜈ｨ菴灘・隱ｭ: plan.n.md 諡｡蜈・

- 逶ｮ逧・
  - `plan.md` 蜈ｨ菴薙ｒ蜀崎ｪｭ縺励�∝ｮ溯｣・′隱､繧翫ｄ縺吶＞莉墓ｧ倥ｒ `tests/plan.n.md` 縺ｫ髮・ｴ・＠縺ｦ蝗槫ｸｰ蜿ｯ閭ｽ縺ｫ縺吶ｋ縲・
- 螟画峩:
  - `tests/plan.n.md` 繧呈僑蜈・�・
  - 譌｢蟄・`compile_fail` 縺ｫ `diag_id` 繧剃ｻ倅ｸ・
    - `plan_block_trailing_semicolon_makes_unit_and_breaks_i32_return` -> `3003`
    - `plan_semicolon_requires_exactly_one_value_growth` -> `3016`
  - 霑ｽ蜉�縺励◆荳ｻ縺ｪ莉墓ｧ倥ユ繧ｹ繝・
    - `block:` 蠕後ｍ縺ｯ繧ｳ繝｡繝ｳ繝医・縺ｿ險ｱ蜿ｯ縲√ヨ繝ｼ繧ｯ繝ｳ遖∵ｭ｢
    - 蠑墓焚繧ｪ繝輔し繧､繝会ｼ郁､・焚陦悟ｼ墓焚・・
    - `while` 縺ｮ `cond/do` 險俶ｳ包ｼ・nline / block・・
    - 髢｢謨ｰ繝ｪ繝・Λ繝ｫ `():`縲～fn` 邉冶｡｣ + `@` 髢｢謨ｰ蛟､蜿ら・
    - pipe 縺ｮ謾ｹ陦瑚ｨ俶ｳ・
    - 蜊倩｡後ヶ繝ｭ繝・け縺ｮ螟壽ｮｵ繝阪せ繝・
    - `if:` 縺・蠑丞ｿ・�・
    - 蜊倩｡後ヶ繝ｭ繝・け隍・枚・・;`蛹ｺ蛻・ｊ・峨→譛ｫ蟆ｾ `;` 縺ｫ繧医ｋ `()` 蛹・
    - 1陦・譁・ｼ亥玄蛻・ｊ縺ｪ縺暦ｼ峨お繝ｩ繝ｼ
    - `Tuple:` 繝ｪ繝・Λ繝ｫ
    - 蝙区ｳｨ驥医′蠑上↓蜑咲ｽｮ縺輔ｌ繧区嫌蜍・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/plan.n.md --no-tree -o /tmp/tests-plan-nmd-2.json -j 15`
  - 邨先棡: `240 total / 239 pass / 1 fail`
- 蟾ｮ蛻・ｼ・lan.md 縺ｨ螳溯｣・ｼ・
  - `plan_single_line_block_cannot_contain_multiline_block` 縺・`expected compile_fail` 縺ｫ蟇ｾ縺励※ compile success縲・
  - 縺薙ｌ縺ｯ plan.md 縺ｮ縲悟腰陦後ヶ繝ｭ繝・け蜀・↓隍・｡後ヶ繝ｭ繝・け繧堤ｽｮ縺代↑縺・�榊宛邏・↓蟇ｾ縺吶ｋ譛ｪ螳溯｣・ぐ繝｣繝・・縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺB2邯咏ｶ・ Copy/Clone 蛻､螳壹・ trait隴伜挨蟄仙喧)

- 逶ｮ逧・
  - `todo.md` 繝輔ぉ繝ｼ繧ｺB2縲荊rait 螂醍ｴ・愛螳壹・譁・ｭ怜・萓晏ｭ倥ｒ貂帙ｉ縺吶�阪ｒ騾ｲ繧√�～Copy/Clone` 閭ｽ蜉帛愛螳壹ｒ trait蜷阪〒縺ｯ縺ｪ縺・trait隴伜挨蟄舌〒謇ｱ縺・�・
- 譬ｹ譛ｬ蜴溷屏:
  - `TraitSemantics` 縺ｨ `ImplInfo` 縺ｮ蛻､螳壹・ `trait_name` 譁・ｭ怜・豈碑ｼ・↓萓晏ｭ倥＠縺ｦ縺翫ｊ縲∝錐蜑崎ｧ｣豎ｺ螟画峩繧・alias 蟆主・譎ゅ↓閼・＞縲・
- 螟画峩:
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics` 繧・`copy_trait/clone_trait: Option<(String, TypeId)>` 縺ｫ螟画峩縲・
    - `is_copy_trait` / `is_clone_trait` 繧・`TypeId` 豈碑ｼ・∈螟画峩縲・
    - `detect_capability_trait` 縺ｮ謌ｻ繧雁�､繧・`Option<(String, TypeId)>` 縺ｸ螟画峩縲・
    - `ImplInfo` 縺ｫ `trait_self_ty: Option<TypeId>` 繧定ｿｽ蜉�縺励�～Copy/Clone` 蛻､螳壹・驥崎､・impl 蛻､螳壹↓蛻ｩ逕ｨ縲・
    - `ctx.set_copy_trait_enabled(...)` 縺ｯ `copy_trait_name().is_some()` 縺ｧ蛻ｶ蠕｡縲・
    - 譛�邨・impl 逕滓・繝代せ縺ｮ copy 蛻､螳壹ｂ `trait_info.self_ty` 繧剃ｽｿ逕ｨ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-copy-trait-model-targeted.json -j 15` -> `278/278 pass`
- 迥ｶ豕・
  - `Copy/Clone` 閭ｽ蜉帛愛螳壹・荳ｻ隕∫ｵ瑚ｷｯ縺ｯ trait蜷肴枚蟄怜・豈碑ｼ・°繧蛾屬閼ｱ縲・
  - 谿九ｊ縺ｮ譁・ｭ怜・萓晏ｭ倥・荳�闊ｬ trait 蠅・阜蛻､螳夲ｼ・trait_bound_satisfied` 縺ｪ縺ｩ・牙・縺ｫ髯仙ｮ壹＆繧後ｋ縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺB2邯咏ｶ・ Copy蛻､螳壹・邨瑚ｷｯ蛻・屬縺ｨ tests/*.n.md 蝗槫ｸｰ霑ｽ蜉�)

- 逶ｮ逧・
  - `todo.md` 繝輔ぉ繝ｼ繧ｺB2縺ｮ谿倶ｻｶ縺ｨ縺励※縲》rait 繝｢繝ｼ繝画凾縺ｮ `Copy` 蛻､螳壹ｒ譌ｧ莠呈鋤邨瑚ｷｯ縺九ｉ蛻・屬縺励�∝錐蜑阪ワ繝ｼ繝峨さ繝ｼ繝我ｾ晏ｭ倥ｒ縺輔ｉ縺ｫ貂帙ｉ縺吶�・
  - 螟画峩縺ｫ蟇ｾ蠢懊☆繧句屓蟶ｰ繧・`tests/*.n.md` 縺ｫ霑ｽ蜉�縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - `TypeCtx::is_copy` 縺ｯ trait 繝｢繝ｼ繝峨〒繧ょ・縺ｫ `is_copy_eligible`・・i64/f64` 蜷阪ワ繝ｼ繝峨さ繝ｼ繝会ｼ峨ｒ騾壹ｋ縺溘ａ縲～impl Copy` 繝吶・繧ｹ蛻､螳壹↓螳悟・遘ｻ陦後〒縺阪※縺・↑縺九▲縺溘�・
  - `Copy impl` 螯･蠖捺�ｧ讀懈渊繧ょ酔縺倡ｵ瑚ｷｯ繧剃ｽｿ縺｣縺ｦ縺翫ｊ縲∵ｮｵ髫守ｧｻ陦後・蠅・阜縺梧尠譏ｧ縺�縺｣縺溘�・
- 螟画峩:
  - `nepl-core/src/types.rs`
    - `is_copy_impl_eligible` 繧定ｿｽ蜉�・・impl Copy` 螯･蠖捺�ｧ蟆ら畑・峨�・
    - `is_copy` 繧堤ｵ瑚ｷｯ蛻・屬:
      - trait 繝｢繝ｼ繝画怏蜉ｹ譎ゅ・ `is_copy_with_trait_model` 繧堤峩謗･菴ｿ逕ｨ縲・
      - trait 繝｢繝ｼ繝臥┌蜉ｹ譎ゅ・縺ｿ `is_copy_eligible` 繧剃ｽｿ逕ｨ縲・
    - `is_copy_eligible_inner` 縺ｫ `allow_opaque_named` 繧定ｿｽ蜉�縺励�～is_copy_impl_eligible` 縺九ｉ縺ｯ Named 蝙九ｒ蜷榊燕萓晏ｭ倥↑縺励〒螯･蠖灘愛螳壼庄閭ｽ縺ｫ縺励◆縲・
  - `nepl-core/src/typecheck.rs`
    - `impl Copy for T` 縺ｮ蟇ｾ雎｡螯･蠖捺�ｧ讀懈渊繧・`ctx.is_copy_impl_eligible(target_ty)` 縺ｫ螟画峩縲・
  - `tests/move_effect.n.md`
    - 蝗槫ｸｰ繧ｱ繝ｼ繧ｹ繧・莉ｶ霑ｽ蜉�:
      - `Copy` trait 譛牙柑譎ゅ�～i64` 縺ｫ `Copy impl` 縺後↑縺・�ｴ蜷医・ move 繧ｨ繝ｩ繝ｼ・・diag_id: 3053`・峨�・
      - `Clone+Copy impl` 繧剃ｸ弱∴縺・`i64` 縺ｯ蜀榊茜逕ｨ蜿ｯ閭ｽ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-copy-trait-model-targeted.json -j 15` -> `278/278 pass`
- 迥ｶ豕・
  - `Copy` 蛻､螳壹・ trait 繝｢繝ｼ繝臥ｵ瑚ｷｯ縺ｯ蛻・屬貂医∩縲・
  - 谺｡谿ｵ縺ｧ `Copy/Clone` 閭ｽ蜉帛ｮ｣險�縺ｮ謚ｽ雎｡蛹厄ｼ・rait 蜷肴､懷・繝ｭ繧ｸ繝・け縺ｮ縺輔ｉ縺ｪ繧倶ｸ�闊ｬ蛹厄ｼ峨∈騾ｲ繧�縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺB2: Copy閭ｽ蜉帛愛螳壹・trait遘ｻ陦後せ繧､繝・メ蟆主・)

- 逶ｮ逧・
  - `todo.md` 繝輔ぉ繝ｼ繧ｺB2縺ｮ縲形Copy/Clone` 閭ｽ蜉帛愛螳壹・繝上・繝峨さ繝ｼ繝画彫蟒・�阪↓蜷代￠縲～Copy` trait 螳溯｣・ュ蝣ｱ縺ｸ谿ｵ髫守ｧｻ陦後☆繧句悄蜿ｰ繧定ｿｽ蜉�縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - `TypeCtx::is_copy` 縺ｯ蟶ｸ縺ｫ讒矩��繝吶・繧ｹ蛻､螳壹・縺ｿ縺ｧ縲～impl Copy for T` 縺ｮ譛臥┌繧定・蜉帛愛螳壹↓蜿肴丐縺ｧ縺阪↑縺九▲縺溘�・
  - 譌｢蟄倩ｳ・肇縺ｨ縺ｮ莠呈鋤繧剃ｿ昴■縺ｪ縺後ｉ遘ｻ陦後☆繧句・譖ｿ轤ｹ縺後↑縺上�∽ｸ�諡ｬ遘ｻ陦後☆繧九→蠎・ｯ・峇縺ｮ蝗槫ｸｰ繝ｪ繧ｹ繧ｯ縺碁ｫ倥°縺｣縺溘�・
- 螟画峩:
  - `nepl-core/src/types.rs`
    - `TypeCtx` 縺ｫ `copy_trait_enabled: bool` 繧定ｿｽ蜉�縲・
    - `set_copy_trait_enabled(bool)` 繧定ｿｽ蜉�縲・
    - `is_copy` 繧呈ｮｵ髫主愛螳壹∈螟画峩:
      - 縺ｾ縺壽里蟄・`is_copy_eligible` 縺ｧ蜑肴署讀懆ｨｼ縲・
      - `copy_trait_enabled == false` 縺ｧ縺ｯ蠕捺擂謖吝虚繧堤ｶｭ謖√�・
      - `copy_trait_enabled == true` 縺ｧ縺ｯ `is_copy_with_trait_model` 繧剃ｽｿ縺・�、DT 縺ｯ `impl Copy` 逋ｻ骭ｲ・・copy_impl_targets`・峨ｒ蠢・�亥喧縲・
    - 霑ｽ蜉�隱ｿ謨ｴ:
      - trait 繝｢繝ｼ繝画凾縺ｮ `TypeKind::Named` / `TypeKind::Apply` 蛻､螳壹ｒ蝙句錐繝上・繝峨さ繝ｼ繝峨°繧牙､悶＠縲～has_copy_impl_target` 繝吶・繧ｹ縺ｸ螟画峩縲・
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics::detect` 蠕後↓ `ctx.set_copy_trait_enabled(...)` 繧定ｨｭ螳壹＠縲～Copy` trait 縺悟ｮ夂ｾｩ縺輔ｌ繧九Δ繧ｸ繝･繝ｼ繝ｫ縺ｧ縺ｮ縺ｿ譁ｰ蛻､螳壹ｒ譛牙柑蛹悶�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-copy-trait-model-targeted.json -j 15` -> `276/276 pass`
- 迥ｶ豕・
  - 莠呈鋤諤ｧ繧剃ｿ昴▲縺溘∪縺ｾ `Copy` trait 蜿肴丐縺ｮ蛻・崛轤ｹ繧貞ｰ主・貂医∩縲・
  - 谺｡谿ｵ縺ｧ `Copy/Clone` 繧定・蜉帙ユ繝ｼ繝悶Ν蛹悶＠縲∝愛螳壹Ο繧ｸ繝・け縺ｮ譁・ｭ怜・萓晏ｭ倥ｒ縺輔ｉ縺ｫ蜑頑ｸ帙☆繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (荳頑ｵ∽ｿｮ豁｣: codegen_wasm 險ｺ譁ｭID縺ｮ譏守､ｺ蛹・

- 逶ｮ逧・
  - `todo.md` 谿倶ｻｶ縺�縺｣縺・`codegen_*.rs` 縺ｮ荳ｻ隕∬ｨｺ譁ｭ繧・`diag_id` 縺ｧ蝗ｺ螳壹＠縲…odegen 螟ｱ謨励・蛻・｡槭ｒ譁・ｨ�萓晏ｭ倥°繧牙・繧企屬縺吶�・
- 譬ｹ譛ｬ蜴溷屏:
  - `codegen_wasm.rs` 縺ｮ `Diagnostic::error(...)` 縺ｯ ID 譛ｪ莉倅ｸ弱〒縲…odegen 繝輔ぉ繝ｼ繧ｺ螟ｱ謨励ｒ螳牙ｮ夂噪縺ｫ迚ｹ螳壹〒縺阪↑縺九▲縺溘�・
- 螟画峩:
  - `nepl-core/src/diagnostic_ids.rs`
    - `D4001..D4015` 繧定ｿｽ蜉�:
      - `CodegenWasmUnsupportedExternSignature`
      - `CodegenWasmUnsupportedFunctionSignature`
      - `CodegenWasmMissingReturnValue`
      - `CodegenWasmRawLineParseError`
      - `CodegenWasmLlvmIrBodyNotSupported`
      - `CodegenWasmStringLiteralNotFound`
      - `CodegenWasmUnknownVariable`
      - `CodegenWasmUnknownFunctionValue`
      - `CodegenWasmUnknownFunction`
      - `CodegenWasmMissingIndirectSignature`
      - `CodegenWasmUnsupportedIndirectSignature`
      - `CodegenWasmUnknownIntrinsic`
      - `CodegenWasmUnsupportedEnumPayloadType`
      - `CodegenWasmUnsupportedStructFieldType`
      - `CodegenWasmUnsupportedTupleElementType`
  - `nepl-core/src/codegen_wasm.rs`
    - 荳ｻ隕・codegen 繧ｨ繝ｩ繝ｼ逋ｺ逕溽せ縺ｫ `with_id(...)` 繧剃ｻ倅ｸ弱�・
    - 霑ｽ蜉�蟇ｾ雎｡:
      - extern/function 繧ｷ繧ｰ繝阪メ繝｣ lower 螟ｱ謨・
      - missing return
      - raw wasm parse 螟ｱ謨・
      - wasm backend 縺ｧ縺ｮ llvm ir body
      - unknown variable/function/function value
      - indirect call signature 蝠城｡・
      - unknown codegen intrinsic
      - enum/struct/tuple 縺ｮ unsupported payload/field/element 蝙・
  - `tests/neplg2.n.md`
    - `wasm_rejects_llvmir_body_with_diag_id` 繧定ｿｽ蜉�・・diag_id: 4005`・峨�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/neplg2.n.md -i tests/functions.n.md -i tests/selfhost_req.n.md --no-tree -o /tmp/tests-codegen-diag-subset.json -j 15` -> `276/276 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-codegen-diagid.json -j 15` -> `798/798 pass`
- 迥ｶ豕・
  - `todo.md` 縺ｮ險ｺ譁ｭID谿倶ｻｶ・・odegen 荳ｻ隕∬ｨｺ譁ｭ・峨・螳御ｺ・�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (荳頑ｵ∽ｿｮ豁｣: typecheck 縺ｮ module/impl 螳夂ｾｩ譎りｨｺ譁ｭID繧呈・遉ｺ蛹・

- 逶ｮ逧・
  - `todo.md` 谿倶ｻｶ縺�縺｣縺・`typecheck.rs` 荳頑ｵ・ｼ・odule/impl 螳夂ｾｩ繝輔ぉ繝ｼ繧ｺ・峨・譛ｪ莉倅ｸ手ｨｺ譁ｭ繧・`diag_id` 縺ｧ蝗ｺ螳壹＠縲∵枚險�萓晏ｭ倥ｒ髯､蜴ｻ縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - 螳夂ｾｩ逋ｻ骭ｲ/impl 讀懆ｨｼ繝輔ぉ繝ｼ繧ｺ縺ｯ `Diagnostic::error(...)` 縺ｮ縺ｾ縺ｾ谿九▲縺ｦ縺翫ｊ縲∝酔遞ｮ繧ｨ繝ｩ繝ｼ縺ｧ繧・ID 縺御ｸ榊ｮ牙ｮ壹□縺｣縺溘�・
  - 縺昴・縺溘ａ `compile_fail` 縺ｮ螟ｱ謨礼炊逕ｱ縺梧枚險�螟画峩縺ｧ謠ｺ繧後ｋ迥ｶ諷九□縺｣縺溘�・
- 螟画峩:
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3073..D3092` 繧定ｿｽ蜉�:
      - `TypeUnknownTraitBound`
      - `TypeWasiImportTargetMismatch`
      - `TypeExternSignatureMustBeFunction`
      - `TypeItemNameConflict`
      - `TypeEnumTypeParamBoundsUnsupported`
      - `TypeStructTypeParamBoundsUnsupported`
      - `TypeTraitTypeParamsUnsupported`
      - `TypeTraitMethodTypeParamsUnsupported`
      - `TypeInherentImplUnsupported`
      - `TypeImplTypeParamsUnsupported`
      - `TypeUnknownTrait`
      - `TypeImplTargetMustBeConcrete`
      - `TypeFunctionSignatureMustBeFunction`
      - `TypeAliasTargetNotFound`
      - `TypeFunctionSignatureOverloadNotFound`
      - `TypeDuplicateImplMethod`
      - `TypeImplMethodNotFoundInTrait`
      - `TypeImplMethodSignatureMismatch`
      - `TypeImplMissingTraitMethod`
      - `TypeEntryFunctionMissingOrAmbiguous`
  - `nepl-core/src/typecheck.rs`
    - 荳頑ｵ∝ｮ夂ｾｩ繝輔ぉ繝ｼ繧ｺ・・num/struct/trait/impl/alias/entry・峨・譛ｪ莉倅ｸ弱お繝ｩ繝ｼ縺ｸ `with_id(...)` 繧剃ｻ倅ｸ弱�・
    - `check_function` 蜀帝�ｭ縺ｮ signature/arity 讀懆ｨｼ縺ｫ繧・ID 繧剃ｻ倅ｸ弱�・
  - `tests/neplg2.n.md`
    - 譌｢蟄・`compile_fail` 縺ｫ `diag_id` 繧定ｿｽ蜉�:
      - `pipe_target_missing_after_annotation_is_error` -> `3016`
      - `wasi_import_rejected_on_wasm_target` -> `3074`
      - `name_conflict_enum_fn_is_error` -> `3076`
      - `trait_bound_missing_impl_is_error` -> `3069`
      - `trait_method_arity_mismatch_is_error` -> `3068`
      - `unknown_trait_bound_is_error` -> `3073`
  - `tests/functions.n.md`
    - `function_alias_target_not_found`・・diag_id: 3086`・峨ｒ霑ｽ蜉�縲・
  - `tests/selfhost_req.n.md`
    - `test_req_trait_extensions` 縺ｫ `diag_id: 3081` 繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/neplg2.n.md -i tests/functions.n.md -i tests/selfhost_req.n.md --no-tree -o /tmp/tests-typecheck-item-diag-subset.json -j 15` -> `275/275 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-typecheck-item-diagid.json -j 15` -> `797/797 pass`
- 迥ｶ豕・
  - `typecheck.rs` 縺ｮ荳頑ｵ∝ｮ夂ｾｩ繝輔ぉ繝ｼ繧ｺ險ｺ譁ｭID莉倅ｸ弱・螳御ｺ・�・
  - 谺｡谿ｵ縺ｯ `todo.md` 谿倶ｻｶ縺ｩ縺翫ｊ `codegen_*.rs` 縺ｮ荳ｻ隕∬ｨｺ譁ｭID譏守､ｺ蛹悶�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (荳頑ｵ∽ｿｮ豁｣: lexer 險ｺ譁ｭID縺ｮ譏守､ｺ蛹悶→蝗槫ｸｰ霑ｽ蜉�)

- 逶ｮ逧・
  - `lexer.rs` 縺ｮ譛ｪ莉倅ｸ弱お繝ｩ繝ｼ縺ｫ險ｺ譁ｭID繧剃ｻ倥￠縲～compile_fail + diag_id` 縺ｧ蝗ｺ螳壽､懆ｨｼ縺ｧ縺阪ｋ迥ｶ諷九↓縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - `unknown token/directive` 莉･螟悶・蟄怜唱繧ｨ繝ｩ繝ｼ縺ｯ `with_id` 譛ｪ莉倅ｸ弱〒縲∝､ｱ謨怜・鬘槭′譁・ｨ�萓晏ｭ倥↓縺ｪ縺｣縺ｦ縺・◆縲・
- 螟画峩:
  - `nepl-core/src/diagnostic_ids.rs`
    - `D1203..D1209` 繧定ｿｽ蜉�:
      - `LexerIndentTabsNotAllowed`
      - `LexerExpectedIndentedBlock`
      - `LexerInvalidPubDirectivePrefix`
      - `LexerIndentWidthMismatch`
      - `LexerIndentLevelMismatch`
      - `LexerInvalidStringEscape`
      - `LexerUnterminatedStringLiteral`
  - `nepl-core/src/lexer.rs`
    - 繧ｿ繝悶う繝ｳ繝・Φ繝医�～#wasm/#llvmir` 蠕後う繝ｳ繝・Φ繝井ｸ崎ｶｳ縲～pub` 謗･鬆ｭ霎櫁ｪ､逕ｨ縲・
      繧､繝ｳ繝・Φ繝亥ｹ・ｸ堺ｸ�閾ｴ/髫主ｱ､荳堺ｸ�閾ｴ縲（nvalid escape縲「nterminated string 縺ｫ `with_id` 繧剃ｻ倅ｸ弱�・
  - `tests/lexer_diag.n.md`
    - 譁ｰ隕剰ｿｽ蜉�・・繧ｱ繝ｼ繧ｹ・・
      - invalid escape -> `diag_id: 1208`
      - unterminated string -> `diag_id: 1209`
      - invalid `pub` prefix -> `diag_id: 1205`
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/lexer_diag.n.md --no-tree -o /tmp/tests-lexer-diag.json -j 15` -> `207/207 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-lexer-diagid-extend.json -j 15` -> `796/796 pass`
- 迥ｶ豕・
  - parser + lexer + typecheck・井ｸｻ隕∫ｵ瑚ｷｯ・峨・險ｺ譁ｭID蝗ｺ螳壼喧縺碁�ｲ陦後�・
  - 谺｡谿ｵ縺ｯ `typecheck` 荳頑ｵ・ｼ・odule/impl 螳夂ｾｩ譎ゑｼ峨→ `codegen_*.rs` 縺ｮ谿区悴莉倅ｸ手ｨｺ譁ｭ繧呈紛逅・☆繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (荳頑ｵ∽ｿｮ豁｣: overload/trait/pipe 縺ｮ險ｺ譁ｭID諡｡蠑ｵ)

- 逶ｮ逧・
  - `typecheck` 縺ｮ譛ｪ莉倅ｸ弱お繝ｩ繝ｼ・育音縺ｫ overload/trait method/pipe/arity 蜻ｨ霎ｺ・峨ｒ險ｺ譁ｭID縺ｧ蝗ｺ螳壼喧縺励�～compile_fail` 蝗槫ｸｰ繧貞ｮ牙ｮ壼喧縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - 蜷御ｸ�繧ｫ繝・ざ繝ｪ縺ｮ蝙区､懈渊螟ｱ謨励〒 `with_id` 譛ｪ莉倅ｸ守ｵ瑚ｷｯ縺梧ｮ九ｊ縲∵枚險�螟画峩縺ｫ蠑ｱ縺・憾諷九□縺｣縺溘�・
  - trait 邨檎罰蜻ｼ縺ｳ蜃ｺ縺励・螟ｱ謨暦ｼ域悴遏･繝｡繧ｽ繝・ラ繝ｻ蠅・阜譛ｪ蜈・ｶｳ縺ｪ縺ｩ・峨′ `diag_id` 縺ｧ隴伜挨縺ｧ縺阪↑縺九▲縺溘�・
- 螟画峩:
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3066..D3072` 繧定ｿｽ蜉�:
      - `TypeTraitMethodTypeArgsNotSupported`
      - `TypeTraitMethodNotFound`
      - `TypeArgumentArityMismatch`
      - `TypeTraitBoundUnsatisfied`
      - `TypeInvalidDeref`
      - `TypeAssignmentArityMismatch`
      - `TypeCallReductionLimitExceeded`
  - `nepl-core/src/typecheck.rs`
    - 莉･荳九・險ｺ譁ｭ縺ｫ `with_id` 繧剃ｻ倅ｸ・
      - `pipe has no target` -> `D3013`
      - trait method 縺ｸ縺ｮ蝙句ｼ墓焚譛ｪ蟇ｾ蠢・-> `D3066`
      - trait method 荳榊惠 -> `D3067`
      - overload 縺ｮ蝙句ｼ墓焚荳堺ｸ�閾ｴ -> `D3021`
      - 蠑墓焚蛟区焚荳堺ｸ�閾ｴ・磯未謨ｰ/constructor/trait method receiver・・> `D3068`
      - trait 蠅・阜譛ｪ蜈・ｶｳ -> `D3069`
      - assignment 蛟区焚荳堺ｸ�閾ｴ -> `D3071`
      - field assignment 蝙倶ｸ堺ｸ�閾ｴ -> `D3036`
      - 髱槫盾辣ｧ蝙・deref -> `D3070`
      - call reduction 蜿榊ｾｩ荳企剞雜・℃ -> `D3072`
  - `tests/overload.n.md`
    - `compile_fail + diag_id` 繧・繧ｱ繝ｼ繧ｹ霑ｽ蜉�:
      - trait method 蝙句ｼ墓焚譛ｪ蟇ｾ蠢・(`3066`)
      - trait method 荳榊惠 (`3067`)
      - trait 蠅・阜譛ｪ蜈・ｶｳ (`3069`)
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md --no-tree -o /tmp/tests-overload-diagid-extend.json -j 15` -> `244/244 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-typecheck-diagid-extend.json -j 15` -> `793/793 pass`
- 迥ｶ豕・
  - `D3006`・・o matching overload・峨→ field access・・D3011`・峨・險ｺ譁ｭ邨瑚ｷｯ繧貞・髮｢縺励◆縺ｾ縺ｾ邯ｭ謖√�・
  - 谺｡谿ｵ縺ｯ `todo.md` 縺ｮ險ｺ譁ｭID諡｡蠑ｵ谿倶ｻｶ・・exer + typecheck荳頑ｵ√・譛ｪ莉倅ｸ朱�伜沺・峨ｒ邯咏ｶ壹☆繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (荳頑ｵ∽ｿｮ豁｣: typecheck 縺ｮ noshadow/shadow 險ｺ譁ｭID繧呈・遉ｺ蛹・

- 逶ｮ逧・
  - `typecheck` 縺ｮ `noshadow` / `non-shadowable` 邉ｻ繧ｨ繝ｩ繝ｼ繧定ｨｺ譁ｭ逕滓・轤ｹ縺ｧ蝗ｺ螳壹＠縲∝屓蟶ｰ繧・`diag_id` 縺ｧ讀懆ｨｼ蜿ｯ閭ｽ縺ｫ縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - 蜷御ｸ�繧ｫ繝・ざ繝ｪ縺ｮ shadow 髢｢騾｣繧ｨ繝ｩ繝ｼ縺ｫ `with_id` 譛ｪ莉倅ｸ守ｵ瑚ｷｯ縺梧ｮ九ｊ縲∵枚險�萓晏ｭ倥・蛻､螳壹↓縺ｪ縺｣縺ｦ縺・◆縲・
- 螟画峩:
  - `nepl-core/src/typecheck.rs`
    - `cannot shadow non-shadowable ...` 邉ｻ繧・`TypeNoShadowViolation (D3014)` 縺ｸ邨ｱ荳�縲・
    - `noshadow declaration ... conflicts ...` 邉ｻ繧・`TypeNoShadowConflict (D3015)` 縺ｸ邨ｱ荳�縲・
    - 髢｢謨ｰ/髢｢謨ｰalias/繝ｭ繝ｼ繧ｫ繝ｫ let 縺ｮ蜷・ｵ瑚ｷｯ縺ｧ secondary label 莉倥″險ｺ譁ｭ縺ｫ繧ょ酔ID繧剃ｻ倅ｸ弱�・
  - `tests/shadowing.n.md`
    - `compile_fail` 4繧ｱ繝ｼ繧ｹ縺ｫ `diag_id: 3014` 繧定ｿｽ蜉�縺励※蝗ｺ螳壼喧縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/move_effect.n.md --no-tree -o /tmp/tests-shadowing-moveeffect-diagid.json -j 15` -> `248/248 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-shadow-diagid.json -j 15` -> `790/790 pass`
- 迥ｶ豕・
  - shadow/noshadow 縺ｮ荳ｻ隕∫ｵ瑚ｷｯ縺ｯ `diag_id` 蝗ｺ螳壼喧貂医∩縲・
  - 谺｡谿ｵ縺ｯ `typecheck` 縺ｮ谿区悴莉倅ｸ弱き繝・ざ繝ｪ・・ndefined/overload/pipe/pure-impure・峨∈諡｡蠑ｵ縺吶ｋ縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (荳頑ｵ∽ｿｮ豁｣: typecheck field-access 險ｺ譁ｭID縺ｮ譏守､ｺ蛹・

- 逶ｮ逧・
  - `typecheck.rs` 縺ｮ field access 邉ｻ繧ｨ繝ｩ繝ｼ繧定ｨｺ譁ｭ逕滓・轤ｹ縺ｧ `DiagnosticId` 蝗ｺ螳壹＠縲～compile_fail` 繧・ID 縺ｧ螳牙ｮ壽､懆ｨｼ縺ｧ縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - `core/field::get` / `put` 邨檎罰縺ｮ螟ｱ謨励・縲∝梛讀懈渊繝輔ぉ繝ｼ繧ｺ縺ｧ逋ｺ逕溘☆繧九↓繧ゅ°縺九ｏ繧峨★縲～with_id` 縺ｪ縺励・ `Diagnostic::error` 縺梧ｮ九▲縺ｦ縺・◆縲・
  - 譁・ｨ�縺ｮ縺ｿ萓晏ｭ倥□縺ｨ縲√お繝ｩ繝ｼ繝・く繧ｹ繝郁ｪｿ謨ｴ譎ゅ↓蝗槫ｸｰ讀懷・縺御ｸ榊ｮ牙ｮ壹↓縺ｪ繧九�・
- 螟画峩:
  - `nepl-core/src/typecheck.rs`
    - `resolve_field_access_with_mode` 驟堺ｸ九・ field 蜿ら・螟ｱ謨暦ｼ育ｯ・峇螟・繝輔ぅ繝ｼ繝ｫ繝我ｸ榊ｭ伜惠/髱櫁､・粋蝙具ｼ峨↓
      `TypeInvalidFieldAccess (D3011)` 繧呈・遉ｺ莉倅ｸ弱�・
  - `tests/move_effect.n.md`
    - `core/field` 縺ｮ荳肴ｭ｣繧｢繧ｯ繧ｻ繧ｹ繧・`compile_fail + diag_id: 3011` 縺ｧ蝗ｺ螳壹☆繧九こ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md --no-tree -o /tmp/tests-move_effect-check.json -j 15` -> `221/221 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-typecheck-field-diagid.json -j 15` -> `790/790 pass`
- 迥ｶ豕・
  - field access 邉ｻ縺ｯ `D3011` 縺ｧ譏守､ｺ蛹門ｮ御ｺ・�・
  - 谺｡谿ｵ縺ｯ `typecheck` 縺ｮ譛ｪ莉倅ｸ朱�伜沺・・hadow / overload / pipe / undefined 邉ｻ・峨ｒ鬆・ｬ｡譏守､ｺ蛹悶☆繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (荳頑ｵ∽ｿｮ豁｣: parser 險ｺ譁ｭID縺ｮ譛ｪ莉倅ｸ守ｮ・園繧呈・遉ｺ蛹・

- 逶ｮ逧・
  - `todo.md` 縺ｮ縲瑚ｨｺ譁ｭID縺ｮ譏守､ｺ莉倅ｸ趣ｼ・arser/typecheck/resolve・峨�阪ｒ荳頑ｵ√°繧蛾�ｲ繧√�～parser.rs` 縺ｮ譛ｪ莉倅ｸ手ｨｺ譁ｭ繧堤函謌千せ縺ｧ蝗ｺ螳壹☆繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - `Diagnostic::error(...)` 縺・`with_id` 縺ｪ縺励〒谿九▲縺ｦ縺翫ｊ縲∝酔遞ｮ繧ｨ繝ｩ繝ｼ縺ｧ繧・D縺悟ｮ牙ｮ壹＠縺ｪ縺・ｵ瑚ｷｯ縺後≠縺｣縺溘�・
  - 譁・ｨ�萓晏ｭ倥・縺ｾ縺ｾ縺�縺ｨ `compile_fail` 縺ｮ蝗槫ｸｰ蝗ｺ螳壹′荳榊香蛻・↓縺ｪ繧九�・
- 螟画峩:
  - `nepl-core/src/parser.rs`
    - 蜀榊ｸｰ荳企剞/辟｡騾ｲ謐怜屓蠕ｩ/marker驟咲ｽｮ/mlstr/#extern繧ｷ繧ｰ繝阪メ繝｣/蝙九ヱ繝ｩ繝｡繝ｼ繧ｿ隗｣譫舌↑縺ｩ縺ｮ譛ｪ莉倅ｸ手ｨｺ譁ｭ縺ｸ `with_id` 繧剃ｻ倅ｸ弱�・
    - 莉倅ｸ鯖D縺ｯ譌｢蟄倥・ Parser 邉ｻ (`ParserExpectedToken`, `ParserUnexpectedToken`, `ParserExpectedIdentifier`, `ParserInvalidTypeExpr`, `ParserInvalidExternSignature`) 繧貞茜逕ｨ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-parser-diagid.json -j 15` -> `789/789 pass`
- 迥ｶ豕・
  - parser 縺ｮ `Diagnostic::error` 縺ｯ險ｺ譁ｭ逕滓・轤ｹ縺ｧ ID 譏守､ｺ蛹匁ｸ医∩縲・
  - 谺｡谿ｵ縺ｯ `typecheck.rs` 縺ｮ譛ｪ莉倅ｸ手ｨｺ譁ｭ縺ｸ蜷梧婿驥昴ｒ螻暮幕縺吶ｋ縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (荳頑ｵ√ユ繧ｹ繝域紛蛯・ `tests/move_check.n.md` 縺ｮ skip 隗｣髯､)

- 逶ｮ逧・
  - `move_check` 邉ｻ `.n.md` 縺ｮ荳頑ｵ∝屓蟶ｰ繧・`skip` 萓晏ｭ倥°繧牙､悶＠縲∬ｨｺ譁ｭID莉倥″ compile_fail 縺ｧ蝗ｺ螳壼喧縺吶ｋ縲・
- 螟画峩:
  - `tests/move_check.n.md`
    - `move_simple_ok` 繧貞ｮ溘さ繝ｼ繝牙喧・・ret: 0`・峨�・
    - `move_use_after_move` 繧・`compile_fail + diag_id: 3053` 縺ｫ螟画峩縲・
    - `move_in_branch` 繧・`compile_fail + diag_id: 3054` 縺ｫ螟画峩縲・
    - `move_in_loop` 繧・`compile_fail + diag_id: 3065` 縺ｫ螟画峩縲・
- 譬ｹ譛ｬ蜴溷屏:
  - 譌ｧ Rust 繝・せ繝育ｧｻ讀肴凾縺ｫ `skip` 縺梧ｮ九▲縺ｦ縺翫ｊ縲∝・蟯仙粋豬・繝ｫ繝ｼ繝怜・蛻ｩ逕ｨ縺ｮ move 蝗槫ｸｰ縺・CI 縺ｧ讀懷・荳崎・縺�縺｣縺溘�・
  - 險ｺ譁ｭID縺ｧ螟ｱ謨礼炊逕ｱ繧貞崋螳壹＠縺ｪ縺・→縲∵枚險�謠ｺ繧後〒諢丞峙縺励↑縺・屓蟶ｰ繧定ｦ玖誠縺ｨ縺吶�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/move_check.n.md --no-tree -o /tmp/tests-move-check-nmd.json -j 15` -> `217/217 pass`
- 迥ｶ豕・
  - `move_check.n.md` 縺ｮ蜈磯�ｭ4繧ｱ繝ｼ繧ｹ縺ｯ螳溯｡悟梛縺ｫ縺ｪ繧翫�～skip` 縺ｯ髯､蜴ｻ貂医∩縲・
  - 谺｡谿ｵ縺ｧ `todo.md` 縺ｮ險ｺ譁ｭID譛ｪ莉倅ｸ朱�伜沺・・arser/typecheck/resolve・峨ｒ邯咏ｶ壹☆繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ Scanner/Writer 縺ｮ逶ｴ謗･蛻ｩ逕ｨ縺ｸ荳区ｵ∫ｧｻ陦・

- 逶ｮ逧・
  - `kpread/kpwrite` 蜈ｬ髢帰PI縺ｮ螳牙・蝙句茜逕ｨ繧剃ｸ区ｵ√∈豬ｸ騾上＆縺帙�∫函繝上Φ繝峨Ν逕ｱ譚･縺ｮ荳ｭ髢捺據邵帙ｒ貂帙ｉ縺吶�・
- 螟画峩:
  - `tests/kp.n.md`
  - `tests/kp_i64.n.md`
  - `tests/stdin.n.md`
  - `tutorials/getting_started/22_competitive_io_and_arith.n.md`
  - `tutorials/getting_started/24_competitive_dp_basics.n.md`
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
  - `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
  - `examples/kp_fizzbuzz.nepl`
  - 縺昴ｌ縺槭ｌ `let sc_obj <Scanner> unwrap_ok scanner_new; let sc <Scanner> sc_obj;` 繧・
    `let sc <Scanner> unwrap_ok scanner_new;` 縺ｸ邨ｱ荳�縲・
  - 繧ｫ繧ｿ繝ｭ繧ｰ蜀・・ `sc_handle` 繧ょ炎髯､縺励�～Scanner` 繧堤峩謗･貂｡縺吝ｽ｢縺ｸ邨ｱ荳�縲・
- 譬ｹ譛ｬ蜴溷屏:
  - 蜈ｬ髢帰PI縺悟ｮ牙・蝙九〒謨ｴ縺｣縺ｦ縺・※繧ゅ�∽ｸ区ｵ√さ繝ｼ繝峨↓譌ｧ譚･縺ｮ莠梧ｮｵ譚溽ｸ帙′谿九ｋ縺ｨ縲∫函繝上Φ繝峨Ν蜑肴署縺ｸ謌ｻ縺励ｄ縺吶￥縺ｪ繧九�・
  - 蜈医↓蛻ｩ逕ｨ蛛ｴ縺ｮ譖ｸ縺肴婿繧呈純縺医ｋ縺薙→縺ｧ縲∵ｬ｡谿ｵ縺ｮ蜈ｬ髢矩擇謨ｴ逅・ｼ医ワ繝ｳ繝峨Ν迚磯囈髮｢・峨ｒ螳牙・縺ｫ騾ｲ繧√ｉ繧後ｋ縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md --no-tree -o /tmp/tests-kp-typed-usage.json -j 15` -> `225/225 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-scanner-writer-typed-direct.json -j 15` -> `729/729 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-scanner-writer-typed-direct.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - 荳区ｵ√・荳ｻ隕∝茜逕ｨ邂・園縺ｯ `Scanner/Writer` 逶ｴ謗･蛻ｩ逕ｨ縺ｸ遘ｻ陦梧ｸ医∩縲・
- 谺｡谿ｵ縺ｧ `kpread/kpwrite` 縺ｮ i32 繝上Φ繝峨Ν蜿励￠蜿悶ｊ繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨・蜈ｬ髢矩擇謨ｴ逅・ｒ邯咏ｶ壹☆繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (荳頑ｵ∽ｿｮ豁｣: move_check 險ｺ譁ｭID縺ｮ譏守､ｺ蛹・

- 逶ｮ逧・
  - `move_check` 縺檎函謌舌☆繧倶ｸｻ隕√お繝ｩ繝ｼ縺ｫ `diag_id` 繧剃ｻ倅ｸ弱＠縲～compile_fail` 繧定ｨｺ譁ｭID縺ｧ蝗ｺ螳壽､懆ｨｼ縺ｧ縺阪ｋ迥ｶ諷九↓縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - move/borrow 邉ｻ繧ｨ繝ｩ繝ｼ縺ｯ譁・ｨ�荳�閾ｴ縺ｫ萓晏ｭ倥＠縺ｦ縺翫ｊ縲∝ｰ・擂縺ｮ譁・ｨ�隱ｿ謨ｴ縺ｧ繝・せ繝医′螢翫ｌ繧・☆縺九▲縺溘�・
  - `todo.md` 縺ｮ縲瑚ｨｺ譁ｭID縺ｮ譏守､ｺ莉倅ｸ弱�阪ｒ貅�縺溘☆縺ｫ縺ｯ縲∬ｨｺ譁ｭ逕滓・轤ｹ・・move_check.rs`・峨〒 enum 繧堤峩謗･謖・ｮ壹☆繧句ｿ・ｦ√′縺ゅ▲縺溘�・
- 螟画峩:
  - `nepl-core/src/diagnostic_ids.rs`
    - `3051..3065` 縺ｮ move/borrow 邉ｻ `DiagnosticId` 繧定ｿｽ蜉�縲・
    - `from_u32` / `message` 縺ｫ譁ｰID繧定ｿｽ蜉�縲・
  - `nepl-core/src/passes/move_check.rs`
    - `Diagnostic::error(...)` 縺ｫ `with_id(...)` 繧剃ｻ倅ｸ弱�・
    - 蟇ｾ雎｡: use/move/borrow/assign/drop/loop蜷域ｵ√・荳ｻ隕∬ｨｺ譁ｭ縲・
  - `tests/move_effect.n.md`
    - 譌｢蟄・compile_fail 2莉ｶ縺ｫ `diag_id` 繧定ｿｽ蜉�・・hared borrow move / move蠕悟・蛻ｩ逕ｨ・峨�・
    - 譁ｰ隕・compile_fail 2莉ｶ繧定ｿｽ蜉�・・ove蠕恵orrow=3063縲∝・蟯仙ｾ継otentially moved=3054・峨�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md --no-tree -o /tmp/tests-move-effect-diagid.json -j 15` -> `220/220 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-move-diagid.json -j 15` -> `789/789 pass`
- 迥ｶ豕・
  - move/borrow邉ｻ縺ｮ `compile_fail + diag_id` 蝓ｺ逶､縺御ｸ頑ｵ√〒遒ｺ遶九�・
  - 谺｡谿ｵ縺ｯ `todo.md` 縺ｮ險ｺ譁ｭID譛ｪ驕ｩ逕ｨ鬆伜沺・・arser/typecheck/resolve縺ｮ谿九ｊ・峨∈諡｡蠑ｵ縺吶ｋ縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ `scanner_new` / `writer_new` 縺ｮ譖匁乂繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝画�ｹ豐ｻ)

- 逶ｮ逧・
  - `unwrap_ok scanner_new` / `unwrap_ok writer_new` 縺ｧ逋ｺ逕溘＠縺・`D3005 ambiguous overload` 繧偵�∵綾繧雁�､蝙九・縺ｿ縺ｧ蛻・ｲ舌☆繧・nullary 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝芽ｨｭ險医°繧芽ｧ｣豸医☆繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - `scanner_new` / `writer_new` 縺ｫ `Result<i32,str>` 迚医→ `Result<Scanner/Writer,str>` 迚医ｒ蜷悟錐縺ｧ蜈ｱ蟄倥＆縺帙◆縺溘ａ縲∝ｼ墓焚0縺ｮ蜻ｼ縺ｳ蜃ｺ縺励〒譁・ц荳崎ｶｳ譎ゅ↓謌ｻ繧雁�､蝙九□縺代〒縺ｯ驕ｸ謚樔ｸ崎・縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
  - 縺昴・譖匁乂諤ｧ縺・`kp` doctest / `tests` / `tutorials` 縺ｮ `unwrap_ok scanner_new` 邉ｻ蜻ｼ縺ｳ蜃ｺ縺励↓豕｢蜿翫＠縲∽ｸ区ｵ√〒騾｣骼也噪縺ｫ蝙倶ｸ堺ｸ�閾ｴ繧定ｪ倡匱縺励※縺・◆縲・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - `scanner_new <()*>Result<i32,str>>` 繧・`scanner_new_handle <()*>Result<i32,str>>` 縺ｫ謾ｹ蜷阪�・
    - 蜈ｬ髢・`scanner_new` 縺ｯ `Result<Scanner,str>` 縺ｮ縺ｿ繧呈署萓帙�・
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new <()*>Result<i32,str>>` 繧・`writer_new_handle <()*>Result<i32,str>>` 縺ｫ謾ｹ蜷阪�・
    - 蜈ｬ髢・`writer_new` 縺ｯ `Result<Writer,str>` 縺ｮ縺ｿ繧呈署萓帙�・
  - `tests/overload.n.md`
    - 霑ｽ蜉�縺励◆ zero-arg `Result` 繧ｱ繝ｼ繧ｹ縺ｮ繧ｷ繧ｰ繝阪メ繝｣/蠑上ｒ菫ｮ豁｣縺励�｝ure 譁・ц縺ｧ豁｣縺励￥讀懆ｨｼ縺ｧ縺阪ｋ迥ｶ諷九∈隱ｿ謨ｴ縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/overload.n.md --no-tree -o /tmp/tests-overload-zeroarg-result.json -j 15` -> `241/241 pass`
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpread-kpwrite-new-overload.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-kpread-overload-unify.json -j 15` -> `729/729 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-kpread-overload-unify.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - `new` 邉ｻ縺ｮ蜈ｬ髢・API 縺ｧ縲梧綾繧雁�､蝙九・縺ｿ蟾ｮ蛻・�阪・譖匁乂諤ｧ繧帝勁蜴ｻ縲・
  - 繝輔ぉ繝ｼ繧ｺD縺ｮ螳牙・API邨ｱ荳�霍ｯ邱夲ｼ亥・髢矩擇縺ｯ螳牙・蝙九�√ワ繝ｳ繝峨Ν迚医・蜀・Κ蜷阪↓髫秘屬・峨↓謨ｴ蜷医�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ `kpread` 縺ｮ `_raw` 萓晏ｭ倥ｒ蜷悟錐繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨∈謨ｴ逅・

- 逶ｮ逧・
  - `kpread` 縺ｮ `scanner_*_raw` 蜻ｽ蜷阪ｒ谿ｵ髫守ｸｮ騾�縺励�～i32` 繝上Φ繝峨Ν迚医→ `Scanner` 迚医ｒ蜷悟錐繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨→縺励※邨ｱ荳�縺吶ｋ縲・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_raw` 繧帝勁縺・`scanner_*_raw` 繧・`scanner_*` 縺ｸ謾ｹ蜷阪�・
    - `i32` 蜿励￠蜿悶ｊ螳溯｣・→ `Scanner` 蜿励￠蜿悶ｊ螳溯｣・ｒ蜷悟錐縺ｧ蜈ｱ蟄倥＆縺帙ｋ讒区・縺ｫ螟画峩縲・
    - 譌｢蟄倥Λ繝・ヱ縺ｯ蜷悟錐繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨・ `i32` 迚医ｒ蜻ｼ縺ｳ蜃ｺ縺吶ｈ縺・↓譖ｴ譁ｰ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - `_raw` 謗･蟆ｾ霎槫・蟯舌′ API 隱ｭ縺ｿ蜿悶ｊ繧ｳ繧ｹ繝医ｒ荳翫￡縲∝ｮ滄圀縺ｫ縺ｯ蝙九□縺代〒蛹ｺ蛻･縺ｧ縺阪ｋ邂・園縺ｾ縺ｧ蜻ｽ蜷榊ｷｮ蛻・ｒ謖√▲縺ｦ縺・◆縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpread-kpwrite-overload-unify.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-kpread-overload-unify.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-kpread-overload-unify.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - `kpread` 縺ｯ `scanner_new_raw` 繧帝勁縺・※ `_raw` 謗･蟆ｾ霎槭↑縺励〒驕狗畑蜿ｯ閭ｽ縺ｪ迥ｶ諷九↓縺ｪ縺｣縺溘�・
  - 谺｡谿ｵ縺ｯ `scanner_new_raw` 縺ｮ謇ｱ縺・ｼ域綾繧雁�､蝙倶ｾ晏ｭ倥・譖匁乂諤ｧ隗｣豸郁ｨｭ險茨ｼ峨ｒ荳頑ｵ∬ｨｭ險医→蜷医ｏ縺帙※讀懆ｨ弱☆繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ `kpwrite` 縺ｮ `_raw` 萓晏ｭ倥ｒ蜷悟錐繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨∈謨ｴ逅・

- 逶ｮ逧・
  - `kpwrite` 蜀・Κ縺ｧ蛻・屬縺励※縺・◆ `*_raw` 鄒､繧偵�～i32` 繝上Φ繝峨Ν迚医→ `Writer` 迚医・蜷悟錐繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨〒邨ｱ荳�縺励�∝・髢矩擇縺ｮ蜻ｽ蜷阪ｒ邁｡貎泌喧縺吶ｋ縲・
- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_raw` 繧帝勁縺阪�～writer_*_raw` 繧・`writer_*` 縺ｸ謾ｹ蜷阪�・
    - `i32` 蜿励￠蜿悶ｊ螳溯｣・→ `Writer` 蜿励￠蜿悶ｊ螳溯｣・ｒ蜷悟錐縺ｧ蜈ｱ蟄倥＆縺帙ｋ蠖｢縺ｫ螟画峩縲・
    - 譌｢蟄倥・ `Writer` 迚医°繧峨・蜷悟錐縺ｮ `i32` 迚医ｒ蜻ｼ縺ｶ繧医≧縺ｫ謨ｴ逅・�・
- 譬ｹ譛ｬ蜴溷屏:
  - `_raw` 謗･蟆ｾ霎槭ｒ蜑肴署縺ｫ繝ｩ繝・ヱ螻､縺悟｢励∴縲、PI 莉墓ｧ倥・隱ｭ縺ｿ蜿悶ｊ繧ｳ繧ｹ繝医′荳翫′縺｣縺ｦ縺・◆縲・
  - 譌｢蟄倥・繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝画ｩ滓ｧ九〒蜊∝・縺ｫ蛹ｺ蛻･蜿ｯ閭ｽ縺ｪ邂・園縺ｾ縺ｧ蜻ｽ蜷榊・蟯舌＠縺ｦ縺・◆縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpwrite-overload-unify.json -j 15` -> `226/226 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-kpwrite-overload-unify.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-kpwrite-overload-unify.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - `kpwrite` 縺ｯ `writer_new_raw` 繧帝勁縺・※ `_raw` 謗･蟆ｾ霎槭↑縺励〒驕狗畑蜿ｯ閭ｽ縺ｪ迥ｶ諷九↓縺ｪ縺｣縺溘�・
  - 谺｡谿ｵ縺ｧ `kpread` 蛛ｴ繧ょ酔譁ｹ驥昴〒谿ｵ髫取紛逅・☆繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ `alloc` 螳牙・API讓呎ｺ門錐蛹悶・蝗槫ｸｰ蠕ｩ譌ｧ)

- 逶ｮ逧・
  - `core/mem` 縺ｮ `alloc/realloc/dealloc` 繧・`Result` 霑泌唆縺ｸ讓呎ｺ門錐蛹悶＠縺溷､画峩縺ｫ蟇ｾ縺励※縲∽ｸ区ｵ√・ `kp`/tests/tutorials 縺ｮ遐ｴ謳阪ｒ荳頑ｵ∝次蝗�縺九ｉ蠕ｩ譌ｧ縺吶ｋ縲・
- 螟画峩:
  - `stdlib/kp/kpprefix.nepl`
    - doctest 縺ｮ `alloc/dealloc` 繧・`alloc_raw/dealloc_raw` 縺ｸ譖ｴ譁ｰ縲・
  - `stdlib/kp/kpsearch.nepl`
    - doctest 縺ｮ `alloc/dealloc` 繧・`alloc_raw/dealloc_raw` 縺ｸ譖ｴ譁ｰ縲・
  - `tests/capacity_stack.n.md`
  - `tests/sort.n.md`
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
  - `examples/tui_editor/editor_fs.nepl`
    - 鄂ｮ謠帙Α繧ｹ縺ｧ螢翫ｌ縺ｦ縺・◆ `#import "alloc_raw/...` 繧・`#import "alloc/...` 縺ｸ蠕ｩ譌ｧ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - 逕溘Γ繝｢繝ｪAPI遘ｻ陦後・荳�諡ｬ鄂ｮ謠帶凾縺ｫ縲・未謨ｰ蜻ｼ縺ｳ蜃ｺ縺励□縺代〒縺ｪ縺・import 繝代せ譁・ｭ怜・縺ｾ縺ｧ `alloc_raw` 縺ｫ譖ｸ縺肴鋤繧上▲縺ｦ縺・◆縲・
  - `alloc` 縺・`Result` 霑泌唆縺ｫ縺ｪ縺｣縺溷ｾ後ｂ縲～kp` doctest 縺ｮ荳�驛ｨ縺・`i32` 蜑肴署縺ｮ譌ｧ險倩ｿｰ繧剃ｿ晄戟縺励※縺・◆縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/core/mem.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/memory_safety.n.md -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md --no-tree -o /tmp/tests-mem-kp-safe-api-switch.json -j 15` -> `233/233 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-mem-kp-safe-api-switch-r2.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-mem-kp-safe-api-switch-r2.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - `alloc` 螳牙・API讓呎ｺ門錐蛹悶・迴ｾ陦悟ｷｮ蛻・・縲～tests + stdlib + tutorials` 縺ｧ蝗槫ｸｰ騾夐℃縲・
  - 谺｡谿ｵ縺ｯ `todo.md` 縺ｮ繝輔ぉ繝ｼ繧ｺD谿倶ｻｶ・亥・髢矩擇縺九ｉ縺ｮ raw 髴ｲ蜃ｺ謨ｴ逅・ｼ峨ｒ邯咏ｶ壹☆繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ vec 縺ｮ `alloc/realloc/dealloc` 繧・`*_raw` 縺ｸ逶ｴ謗･遘ｻ陦・

- 逶ｮ逧・
  - `vec` 縺�縺第ｮ九▲縺ｦ縺・◆ `alloc/realloc/dealloc` 蜻ｼ縺ｳ蜃ｺ縺励ｒ `*_raw` 縺ｫ邨ｱ荳�縺励�√Γ繝｢繝ｪAPI遘ｻ陦後・蛛懈ｻ櫁ｦ∝屏繧定ｧ｣豸医☆繧九�・
- 螟画峩:
  - `stdlib/alloc/collections/vec.nepl`
    - `alloc` -> `alloc_raw`
    - `realloc` -> `realloc_raw`
    - `dealloc` -> `dealloc_raw`
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl -i stdlib/tests/vec.n.md -i tests/capacity_stack.n.md -i tests/pipe_collections.n.md --no-tree -o /tmp/tests-vec-raw-direct.json -j 15` -> `236/236 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-vec-raw-direct.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-vec-raw-direct.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - 莉･蜑・`todo.md` 縺ｫ谿九＠縺ｦ縺・◆ `vec` 縺ｮ `realloc_raw` OOB 蜀咲樟縺ｯ迴ｾ陦檎ｳｻ縺ｧ蜀咲樟縺帙★縲∫ｧｻ陦後ｒ螳御ｺ・〒縺阪◆縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (荳頑ｵ∽ｿｮ豁｣: codegen 縺ｮ alloc helper 隗｣豎ｺ繧・`*_raw` 蜆ｪ蜈医∈邨ｱ荳�)

- 逶ｮ逧・
  - `alloc/dealloc/realloc` 縺ｮ蜷悟錐螳牙・繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝牙ｰ主・譎ゅ↓縲…odegen 蛛ｴ縺瑚ｪ､縺｣縺・helper 繧定ｧ｣豎ｺ縺励※蜀榊ｸｰ繝ｻ繧ｹ繧ｿ繝・け繧ｪ繝ｼ繝舌・繝輔Ο繝ｼ縺ｸ關ｽ縺｡繧区�ｹ譛ｬ蜴溷屏繧剃ｸ頑ｵ√〒髯､蜴ｻ縺吶ｋ縲・
- 螟画峩:
  - `nepl-core/src/codegen_wasm.rs`
    - 蜀・Κ遒ｺ菫・helper 隗｣豎ｺ繧・`alloc_raw` 蜆ｪ蜈医�～alloc` 繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ縺ｸ螟画峩縲・
  - `nepl-core/src/codegen_llvm.rs`
    - runtime helper 隗｣豎ｺ髢｢謨ｰ `resolve_runtime_helper_symbol` 繧定ｿｽ蜉�縲・
    - `alloc/dealloc/realloc` 蛻ｰ驕秘未謨ｰ霑ｽ蜉�縺ｧ `*_raw` 蜆ｪ蜈医�∵立蜷阪ヵ繧ｩ繝ｼ繝ｫ繝舌ャ繧ｯ縺ｸ螟画峩縲・
    - `resolve_alloc_symbol` 繧・`alloc_raw` 蜆ｪ蜈医↓螟画峩縲・
    - entry lower 譎ゅ・ fallback allocator 蛻､螳壹ｒ `alloc_raw` 蜆ｪ蜈域爾邏｢縺ｫ螟画峩縲・
    - `resolve_symbol_name` 縺ｯ map 縺ｮ螳溘く繝ｼ蜿ら・繧定ｿ斐☆螳溯｣・↓螟画峩縲・
  - `nepl-core/src/monomorphize.rs`
    - runtime helper 菫晄戟蟇ｾ雎｡繧・`alloc_raw/dealloc_raw/realloc_raw` 蜆ｪ蜈医↓螟画峩・域立蜷阪ヵ繧ｩ繝ｼ繝ｫ繝舌ャ繧ｯ・峨�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-overload-memory-after-core-helper-fix.json -j 15` -> `244/244 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-core-helper-fix.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-core-helper-fix.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - 荳頑ｵ・ｼ・odegen/monomorphize・峨・ helper 隗｣豎ｺ邨瑚ｷｯ縺・`*_raw` 蜆ｪ蜈医〒謠・▲縺溘◆繧√�∵ｬ｡谿ｵ縺ｮ `core/mem` 螳牙・API讓呎ｺ門錐蛹悶ｒ蜀埼幕縺ｧ縺阪ｋ迥ｶ諷九↓縺ｪ縺｣縺溘�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (隱ｿ譟ｻ: alloc 蜷悟錐繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨・陦晉ｪ√→蟾ｮ縺玲綾縺・

- 莠玖ｱ｡:
  - `core/mem` 縺ｫ `alloc/realloc/dealloc` 縺ｮ `MemPtr` 螳牙・繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨ｒ霑ｽ蜉�縺吶ｋ縺ｨ縲・
    `stdlib/core/option.nepl::doctest#3` / `stdlib/core/result.nepl::doctest#4` 縺ｪ縺ｩ縺ｧ
    `Maximum call stack size exceeded` 縺檎匱逕溘�・
- 蜴溷屏:
  - 繧ｳ繝ｳ繝代う繝ｩ逕滓・繧ｳ繝ｼ繝牙・縺・`alloc : (i32)->i32` 繧呈囓鮟吝燕謠舌→縺励※縺翫ｊ縲・
    蜷悟錐繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝芽ｿｽ蜉�縺ｧ螳溯｡梧凾邨瑚ｷｯ縺悟ｴｩ繧後ｋ縲・
- 蟇ｾ蠢・
  - `alloc/realloc/dealloc` 縺ｮ `MemPtr` 蜷悟錐繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨・荳�譌ｦ蟾ｮ縺玲綾縺励�・
  - `load/store` 縺ｮ `MemPtr` 蜷悟錐繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨・邯ｭ謖√�・
  - 霑ｽ蜉�縺励◆ `tests/memory_safety.n.md` 縺ｮ `alloc<...>` 繧ｱ繝ｼ繧ｹ縺ｯ蜑企勁縲・
- 繝・せ繝・
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety-after-alloc-overload-revert.json -j 15` -> `213/213 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-mem-overload-revert.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-mem-overload-revert2.json -j 15` -> `262/262 pass`
- 谺｡蟇ｾ蠢・
  - `alloc` 邉ｻ縺ｮ讓呎ｺ門錐螳牙・蛹悶・縲√さ繝ｳ繝代う繝ｩ蛛ｴ縺ｮ證鈴ｻ吩ｾ晏ｭ倥ｒ蜈医↓隗｣豸医＠縺ｦ縺九ｉ蜀榊ｰ主・縺吶ｋ縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ core/mem 縺ｮ MemPtr load/store 繧呈ｨ呎ｺ門錐繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝牙喧)

- 逶ｮ逧・
  - `*_ptr` 謗･蟆ｾ霎樔ｾ晏ｭ倥ｒ貂帙ｉ縺励�～MemPtr` 蛻ｩ逕ｨ譎ゅ・讓呎ｺ門錐 `load_i32/store_i32/load_u8/store_u8` 縺ｧ譖ｸ縺代ｋ繧医≧縺ｫ縺吶ｋ縲・
- 螟画峩:
  - `stdlib/core/mem.nepl`
    - `load_i32/store_i32/load_u8/store_u8` 縺ｫ `MemPtr` 蠑墓焚迚医・繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨ｒ霑ｽ蜉�縲・
    - 譌ｧ `load_i32_ptr/store_i32_ptr/load_u8_ptr/store_u8_ptr` 縺ｯ莠呈鋤繧ｨ繧､繝ｪ繧｢繧ｹ蛹悶�・
    - `MemPtr` 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨・辟｡蜉ｹ繝昴う繝ｳ繧ｿ譎ゅ↓ `Option::None` / `Result::Err` 繧定ｿ斐☆縲・
- 繝・せ繝・
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-mem-overload-loadstore.json -j 15` -> `218/218 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-mem-loadstore-overload.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-mem-loadstore-overload.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - `MemPtr` 蛻ｩ逕ｨ繧ｳ繝ｼ繝峨・讓呎ｺ門錐縺ｧ螳牙・縺ｪ load/store 繧貞他縺ｹ繧狗憾諷九↓縺ｪ縺｣縺溘�・
  - 谺｡谿ｵ縺ｯ `alloc/realloc/dealloc` 蛛ｴ縺ｮ蜈ｬ髢句錐螳牙・蛹悶ｒ邯咏ｶ壹☆繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ kpread_core 隗｣謾ｾ邨瑚ｷｯ縺ｮ Result 蛹・

- 逶ｮ逧・
  - `kpread_core` 縺ｮ蛻晄悄蛹門､ｱ謨玲凾蟾ｻ縺肴綾縺励〒 `dealloc_raw` 逶ｴ蜻ｼ縺ｳ繧呈ｸ帙ｉ縺励�∝､ｱ謨怜・逅・ｒ `Result` 縺ｸ蟇・○繧九�・
- 螟画峩:
  - `stdlib/kp/kpread_core.nepl`
    - `nread` 遒ｺ菫晏､ｱ謨玲凾縲～iov/buf` 縺ｮ隗｣謾ｾ繧・`dealloc_result` 繝吶・繧ｹ縺ｸ螟画峩縲・
    - `realloc` 螟ｱ謨玲凾縲～iov/nread_ptr/buf` 縺ｮ隗｣謾ｾ繧・`dealloc_result` 繝吶・繧ｹ縺ｸ螟画峩縲・
    - `scanner` 繝倥ャ繝�遒ｺ菫晏､ｱ謨玲凾縺ｨ謌仙粥蠕後・荳�譎る�伜沺隗｣謾ｾ繧・`dealloc_result` 繝吶・繧ｹ縺ｸ螟画峩縲・
    - 隗｣謾ｾ螟ｱ謨励・蟾ｻ縺肴綾縺怜・逅・ｒ豁｢繧√★蜷ｸ蜿弱☆繧区婿驥昴〒邨ｱ荳�縲・
- 繝・せ繝・
  - `node nodesrc/tests.js -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kp-core-dealloc-result.json -j 15` -> `228/228 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kpreadcore-dealloc-result.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kpreadcore-dealloc-result.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - `kpread_core` 蛻晄悄蛹門､ｱ謨玲凾縺ｮ隗｣謾ｾ邨瑚ｷｯ縺ｯ `Result` 邉ｻAPI縺ｫ蟇・○繧峨ｌ縺溘�・
  - 谺｡谿ｵ縺ｧ `core/mem` 蜈ｬ髢句錐縺ｮ螳牙・API讓呎ｺ門喧繧堤ｶ咏ｶ壹☆繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ kpwrite 蛻晄悄蛹悶・譬ｹ譛ｬ謨ｴ逅・

- 逶ｮ逧・
  - `kpwrite` 蛻晄悄蛹悶ｒ `0` 繧ｻ繝ｳ繝√ロ繝ｫ蛻・ｲ舌°繧牙､悶＠縲～Result` 繝吶・繧ｹ縺ｧ遒ｺ菫晏､ｱ謨励→蟾ｻ縺肴綾縺励ｒ荳�蜈・喧縺吶ｋ縲・
- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_handle_raw` 繧貞炎髯､縲・
    - `writer_alloc_buf` 繧定ｿｽ蜉�縺励�～4096 -> 1024 -> 256` 縺ｮ谿ｵ髫守｢ｺ菫昴ｒ `Result<WriterBuf,str>` 縺ｧ霑斐☆繧医≧縺ｫ螟画峩縲・
    - `writer_try_free` 繧定ｿｽ蜉�縺励�∝・譛溷喧騾比ｸｭ縺ｮ螟ｱ謨玲凾縺ｫ隗｣謾ｾ螟ｱ謨励ｒ蜷ｸ蜿弱＠縺ｦ蟾ｻ縺肴綾縺帙ｋ繧医≧縺ｫ螟画峩縲・
    - `writer_new_raw` 縺ｯ `alloc_result/dealloc_result` 蜑肴署縺ｮ `match` 騾｣骼悶∈鄂ｮ謠帙＠縲∫｢ｺ菫晏､ｱ謨玲凾縺ｮ霑泌唆逅・罰繧呈ｮｵ髫主挨縺ｫ蝗ｺ螳壹�・
- 繝・せ繝・
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpwrite-result-init-refine.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kpwrite-resultrefine.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kpwrite-resultrefine.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - `writer_new_raw` 縺ｮ螟ｱ謨苓｡ｨ迴ｾ縺ｯ `Result` 縺ｸ蜿取據縺励�√そ繝ｳ繝√ロ繝ｫ `0` 萓晏ｭ倥・蛻・ｲ舌ｒ蛻晄悄蛹也ｵ瑚ｷｯ縺九ｉ髯､蜴ｻ縺ｧ縺阪◆縲・
  - 谺｡谿ｵ縺ｯ `todo.md` 繝輔ぉ繝ｼ繧ｺD縺ｮ荳ｻ隱ｲ鬘鯉ｼ・core/mem` 蜈ｬ髢帰PI縺ｮ螳牙・蜷咲ｵｱ荳�・峨ｒ邯咏ｶ壹☆繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ kpwrite 蛻晄悄蛹也ｵ瑚ｷｯ縺ｮ Result 蛹・

- 逶ｮ逧・
  - `kpwrite` 縺ｮ蛻晄悄蛹也ｵ瑚ｷｯ繧・`Result` 邨瑚ｷｯ縺ｸ謠・∴縲～kpread` 縺ｨ蜷後§螟ｱ謨苓｡ｨ迴ｾ縺ｫ邨ｱ荳�縺吶ｋ縲・
- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - 譌ｧ `writer_new_raw`・・i32`霑泌唆・画悽菴薙ｒ `writer_new_handle_raw` 縺ｸ蛻・屬縲・
    - 譁ｰ `writer_new_raw` 繧・`Result<i32,str>` 霑泌唆縺ｸ螟画峩縲・
    - `writer_new` 縺ｯ `writer_new_raw` 縺ｮ `Result` 繧・`Writer` 縺ｸ謖√■荳翫￡繧句ｮ溯｣・∈螟画峩縲・
- 繝・せ繝・
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpwrite-result-init.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kpwrite-result.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kpwrite-result.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - `kpread/kpwrite` 縺ｮ蛻晄悄蛹門・髢狗ｵ瑚ｷｯ縺ｯ縺ｩ縺｡繧峨ｂ `Result` 繝吶・繧ｹ縺ｧ邨ｱ荳�貂医∩縲・
  - 谺｡谿ｵ縺ｯ `todo.md` 繝輔ぉ繝ｼ繧ｺD谿倶ｻｶ縺ｨ縺励※縲～mem` 蛛ｴ蜈ｬ髢句錐縺ｮ螳牙・API讓呎ｺ門喧繧帝�ｲ繧√ｋ縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ kpread_core 縺ｮ蛻晄悄蛹悶ｒ Result 繝吶・繧ｹ蛹・

- 逶ｮ逧・
  - `kpread` 蛻晄悄蛹也ｵ瑚ｷｯ縺ｮ螟ｱ謨苓｡ｨ迴ｾ繧・`0` 繧ｻ繝ｳ繝√ロ繝ｫ萓晏ｭ倥°繧・`Result` 縺ｸ蟇・○繧九�・
  - 繝｡繝｢繝ｪ遒ｺ菫晏､ｱ謨玲凾縺ｮ蛻・ｲ舌ｒ蝙九〒謇ｱ縺医ｋ繧医≧縺ｫ縺励�∵ｮｵ髫守噪縺ｪ螳牙・API讓呎ｺ門喧繧帝�ｲ繧√ｋ縲・
- 螟画峩:
  - `stdlib/kp/kpread_core.nepl`
    - `scanner_new_impl_i` 繧・`scanner_new_impl` 縺ｸ謾ｹ蜷阪�・
    - 謌ｻ繧雁�､繧・`i32` 縺九ｉ `Result<i32,str>` 縺ｸ螟画峩縲・
    - `alloc_result/realloc_result` 繧剃ｽｿ縺｣縺ｦ遒ｺ菫晏､ｱ謨励ｒ `Err` 蛹悶�・
    - 蠕悟ｧ区忰・郁ｧ｣謾ｾ・峨・譌｢蟄倥Ξ繧､繧｢繧ｦ繝育ｶｭ謖√・縺溘ａ `dealloc_raw` 繧堤ｶ咏ｶ壻ｽｿ逕ｨ縲・
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_raw` 繧・`Result<i32,str>` 霑泌唆縺ｸ螟画峩縲・
    - `scanner_new` 縺ｯ `scanner_new_raw` 縺ｮ `Result` 繧偵◎縺ｮ縺ｾ縺ｾ `Scanner` 縺ｸ謖√■荳翫￡繧句ｽ｢縺ｫ螟画峩縲・
- 繝・せ繝・
  - `node nodesrc/tests.js -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpread.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md --no-tree -o /tmp/tests-kpread-result-init.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kpreadcore-result.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kpreadcore-result.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - `kpread` 縺ｮ蛻晄悄蛹也ｵ瑚ｷｯ縺ｯ `Result` 繝吶・繧ｹ縺ｫ遘ｻ陦梧ｸ医∩縲・
  - 谺｡谿ｵ縺ｧ `kpwrite` 蛻晄悄蛹也ｵ瑚ｷｯ繧ょ酔縺俶婿驥昴↓謠・∴繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ `*_new_raw` 蜷咲ｵｱ荳�縺ｨ todo 譛ｪ螳御ｺ・紛逅・

- 逶ｮ逧・
  - `kpread/kpwrite` 縺ｮ蜀・Κ蛻晄悄蛹夜未謨ｰ蜷阪ｒ `*_raw` 縺ｫ邨ｱ荳�縺励�∝・髢句・蜿｣繧・`scanner_new` / `writer_new` 縺ｫ蟇・○繧九�・
  - `todo.md` 縺九ｉ螳御ｺ・ｸ医∩縺ｮ繝・せ繝郁ｿｽ蜉�鬆・岼繧貞炎髯､縺励�∵悴螳御ｺ・・縺ｿ繧剃ｿ晄戟縺吶ｋ縲・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_i32` -> `scanner_new_raw`縲・
    - `scanner_new` 縺九ｉ縺ｮ蜻ｼ縺ｳ蜃ｺ縺怜・繧呈峩譁ｰ縲・
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_i32` -> `writer_new_raw`縲・
    - `writer_new` 縺九ｉ縺ｮ蜻ｼ縺ｳ蜃ｺ縺怜・繧呈峩譁ｰ縲・
  - `todo.md`
    - 繝輔ぉ繝ｼ繧ｺE縺ｮ螳御ｺ・ｸ医∩蟆城�・岼・・tests/move_effect.n.md` 霑ｽ蜉�縲～tests/overload.n.md`/`tests/kp*.n.md` 譖ｴ譁ｰ・峨ｒ蜑企勁縲・
    - 鬆・岼8縺ｮ螳御ｺ・ｸ医∩蟆城�・岼・・tests/memory_safety.n.md` 霑ｽ蜉�・峨ｒ蜑企勁縲・
- 繝・せ繝・
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kp-newraw-rename.json -j 15` -> `227/227 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-newraw-rename.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-newraw-rename.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - `kpread/kpwrite` 縺ｮ蜀・Κ蛻晄悄蛹夜未謨ｰ蜷阪′ `*_raw` 縺ｧ謠・▲縺溘�・
  - 谺｡谿ｵ縺ｯ繝輔ぉ繝ｼ繧ｺD谿倶ｻｶ縺ｨ縺励※縲～mem` 蜈ｬ髢矩擇縺ｮ螳牙・API讓呎ｺ門錐蛹厄ｼ・Result/Option` 蜑肴署・峨ｒ騾ｲ繧√ｋ縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ kpread 縺ｮ raw 螳溯｣・錐蛻・屬)

- 逶ｮ逧・
  - `kpread` 縺ｮ蜀・Κ `i32` 繝上Φ繝峨Ν螳溯｣・→蜈ｬ髢・`Scanner` API 繧呈・遒ｺ縺ｫ蛻・屬縺励�∝・髢矩擇縺ｮ蝙句ｮ牙・諤ｧ繧剃ｸ翫￡繧九�・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - `i32` 蜿励￠蜿悶ｊ螳溯｣・ｒ `scanner_*_raw` 縺ｸ謾ｹ蜷阪�・
    - `Scanner` 蜿励￠蜿悶ｊ蜈ｬ髢矩未謨ｰ縺ｯ譌｢蟄伜錐繧堤ｶｭ謖√＠縲∝・驛ｨ縺ｧ `*_raw` 繧貞他縺ｳ蜃ｺ縺吝ｽ｢縺ｸ螟画峩縲・
    - 蟇ｾ雎｡: `skip_ws/is_eof/skip_token/read_token/read_i32/read_i64/read_u64/read_f32/read_f64/read_vec/read_matrix/read_all/read_*input` 荳�蠑上�・
- 繝・せ繝・
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -i examples/kp_fizzbuzz.nepl --no-tree -o /tmp/tests-kp-raw-split-both.json -j 15` -> `230/230 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kpread-split.json -j 15` -> `727/727 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kpread-split.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - `kpread/kpwrite` 縺ｨ繧ゅ↓縲悟・髢・API = Scanner/Writer 蝙九�阪�悟・驛ｨ螳溯｣・= *_raw縲阪∈蛻・屬貂医∩縲・
  - 谺｡谿ｵ縺ｯ `todo.md` 2026-03-03 繝輔ぉ繝ｼ繧ｺD縺ｮ谿倶ｻｶ・・mem` 蜈ｬ髢矩擇縺ｮ `_safe` 蟒・ｭ｢縺ｨ `_raw` 譛�邨ょ炎髯､・峨∈騾ｲ繧�縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ kpwrite 縺ｮ raw 螳溯｣・錐蛻・屬)

- 逶ｮ逧・
  - `kpwrite` 縺ｮ蜀・Κ `i32` 繝上Φ繝峨Ν螳溯｣・→蜈ｬ髢・`Writer` API 繧呈・遒ｺ縺ｫ蛻・屬縺励�∝・髢矩擇縺ｮ蝙句ｮ牙・諤ｧ繧剃ｸ翫￡繧九�・
- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `i32` 蜿励￠蜿悶ｊ螳溯｣・ｒ `writer_*_raw` 縺ｸ謾ｹ蜷阪�・
    - `Writer` 蜿励￠蜿悶ｊ蜈ｬ髢矩未謨ｰ縺ｯ譌｢蟄伜錐繧堤ｶｭ謖√＠縲∝・驛ｨ縺ｧ `*_raw` 繧貞他縺ｳ蜃ｺ縺吝ｽ｢縺ｸ螟画峩縲・
    - 蟇ｾ雎｡: `free/flush/ensure/put_u8/writeln/write_*` 荳�蠑上�・
- 繝・せ繝・
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md --no-tree -o /tmp/tests-kpwrite-raw-split.json -j 15` -> `226/226 pass`
- 迥ｶ豕・
  - `kpwrite` 縺ｯ縲悟・髢・API = Writer 蝙九�阪�悟・驛ｨ螳溯｣・= *_raw縲阪∈蛻・屬螳御ｺ・�・
  - 谺｡谿ｵ縺ｧ `kpread` 繧ょ酔譁ｹ驥昴↓謠・∴繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (overload 繝・せ繝域僑蜈・ 豕ｨ驥域ｷｷ蝨ｨ繧ｱ繝ｼ繧ｹ縺ｮ霑ｽ蜉�)

- 逶ｮ逧・
  - `overload` 蝗槫ｸｰ縺ｫ縲∝梛豕ｨ驥医・豺ｷ蝨ｨ繝代ち繝ｼ繝ｳ・医ヶ繝ｭ繝・け豕ｨ驥医・髢｢謨ｰ蜻ｼ縺ｳ蜃ｺ縺玲ｳｨ驥医・繝代う繝励・髢｢謨ｰ繝ｪ繝・Λ繝ｫ・峨ｒ霑ｽ蜉�縺吶ｋ縲・
- 螟画峩:
  - `tests/overload.n.md`
    - `overload_mixed_annotations_block_call_pipe_lambda` 繧定ｿｽ蜉�縲・
    - `overload_pipe_annotations_with_mixed_cast_i32_i64_i128` 繧定ｿｽ蜉�縲・
- 蛻・ｊ蛻・￠:
  - 蛻晉沿縺ｧ縺ｯ `pipe requires a value on the stack (D3013)` 縺ｨ `ambiguous overload (D3005)` 繧貞・迴ｾ縲・
  - 隗｣譫千ｵ先棡:
    - `let ...:` 縺ｮ蠑墓焚繝悶Ο繝・け逶ｴ蠕後↓ `|>` 繧堤峩謗･謗･邯壹☆繧句ｽ｢縺ｯ迴ｾ陦御ｻ墓ｧ倥〒縺ｯ蠑丞｢・阜縺悟・縺九ｌ繧九�・
    - `|> <i64> cast` 縺ｯ縲碁未謨ｰ蛟､縺ｸ縺ｮ豕ｨ驥医�阪→縺励※隗｣驥医＆繧後�∵綾繧雁�､豕ｨ驥医↓縺ｯ縺ｪ繧峨★譖匁乂蛹悶☆繧九�・
  - 繝・せ繝医・莉墓ｧ倥↓謨ｴ蜷医☆繧句ｽ｢縺ｸ菫ｮ豁｣:
    - 繝悶Ο繝・け豕ｨ驥医・ `base` 縺ｫ譚溽ｸ帙＠縺ｦ縺九ｉ騾壼ｸｸ蜻ｼ縺ｳ蜃ｺ縺励〒騾｣邨舌�・
    - cast 縺ｯ `seed` 繧呈・遉ｺ螟画鋤縺励◆蠕後↓ pipe 縺ｧ蜉�邂励ｒ螳滓命縲・
- 繝・せ繝・
  - `node nodesrc/tests.js -i tests/overload.n.md --no-tree -o /tmp/tests-overload-after-fix2.json -j 15` -> `239/239 pass`

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ stdlib 縺ｮ逕溘Γ繝｢繝ｪ蜻ｼ縺ｳ蜃ｺ縺励ｒ `*_raw` 縺ｸ谿ｵ髫守ｧｻ陦・

- 逶ｮ逧・
  - `mem` 縺ｮ蜈ｬ髢句錐蛻・崛蜑阪↓縲《tdlib 蛛ｴ縺ｮ逕溘い繝ｭ繧ｱ繝ｼ繧ｿ蜻ｼ縺ｳ蜃ｺ縺励ｒ `alloc_raw/dealloc_raw/realloc_raw` 縺ｫ蟇・○繧九�・
- 螟画峩:
  - `stdlib/alloc/collections/{btreemap,btreeset,hashmap,hashset,list,ringbuffer,stack,vec/sort}.nepl`
  - `stdlib/alloc/{diag/error,string}.nepl`
  - `stdlib/kp/{kpdsu,kpfenwick,kpgraph,kpprefix,kpread_core}.nepl`
  - `stdlib/nm/{parser,html_gen}.nepl`
  - `stdlib/platforms/wasix/tui.nepl`
  - `stdlib/std/{env/cliarg,fs,stdio}.nepl`
  - 荳願ｨ倥〒 `alloc/dealloc/realloc` 縺ｮ逕溷他縺ｳ蜃ｺ縺励ｒ `*_raw` 縺ｫ鄂ｮ謠幢ｼ・core/mem` 縺ｮ蜈ｬ髢句錐萓晏ｭ倥ｒ蛻・屬・峨�・
- 蛻・ｊ蛻・￠:
  - 荳�諡ｬ鄂ｮ謠帛ｾ後�～tests/capacity_stack.n.md::doctest#3` 縺ｧ OOB 繧貞・迴ｾ縲・
  - 蜴溷屏蛻・ｊ蛻・￠縺ｧ `vec.nepl` 縺ｮ `realloc_raw` 鄂ｮ謠帶凾縺ｮ縺ｿ蜀咲樟縺吶ｋ縺薙→繧堤｢ｺ隱阪＠縺溘◆繧√�～vec.nepl` 譛ｬ菴薙・迴ｾ譎らせ縺ｧ縺ｯ `realloc` 蜻ｼ縺ｳ蜃ｺ縺励ｒ邯ｭ謖√＠縺ｦ蝗樣∩縲・
  - 縺薙・蟾ｮ蛻・・ `todo.md` 縺ｫ譛ｪ隗｣豎ｺ隱ｲ鬘後→縺励※霑ｽ險倥�・
- 繝・せ繝・
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-after-raw-migration-wide2.json -j 15` -> `725/725 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-raw-migration-wide.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - stdlib 縺ｮ螟ｧ驛ｨ蛻・・ `*_raw` 蜻ｼ縺ｳ蜃ｺ縺励∈遘ｻ陦梧ｸ医∩縲・
  - 谿倶ｻｶ縺ｯ `vec.nepl` 縺ｮ `realloc_raw` 遘ｻ陦後↓莨ｴ縺・OOB 蜴溷屏縺ｮ譬ｹ譛ｬ菫ｮ豁｣縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ `kpread/kpwrite` 縺ｮ逕溘Γ繝｢繝ｪ蜻ｼ縺ｳ蜃ｺ縺励ｒ `*_raw` 縺ｸ遘ｻ陦・

- 逶ｮ逧・
  - `core/mem` 縺ｮ `*_raw` 蛻・屬縺ｫ蜷医ｏ縺帙�～kpread/kpwrite` 蛛ｴ縺ｮ逕溘い繝ｭ繧ｱ繝ｼ繧ｿ蜻ｼ縺ｳ蜃ｺ縺励ｒ譏守､ｺ蛹悶☆繧九�・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - 譁・ｭ怜・繝医・繧ｯ繝ｳ逕滓・譎ゅ・遒ｺ菫昴ｒ `alloc` 縺九ｉ `alloc_raw` 縺ｸ螟画峩縲・
  - `stdlib/kp/kpwrite.nepl`
    - writer 蛻晄悄蛹・隗｣謾ｾ縺ｮ `alloc`/`dealloc` 蜻ｼ縺ｳ蜃ｺ縺励ｒ `alloc_raw`/`dealloc_raw` 縺ｸ螟画峩縲・
    - 繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝医・譁・ｨ�繧貞ｮ溯｣・↓蜷医ｏ縺帙※隱ｿ謨ｴ・医�後ヲ繝ｼ繝礼｢ｺ菫昴↑縺励�搾ｼ峨�・
- 繝・せ繝・
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tests/memory_safety.n.md --no-tree -o /tmp/tests-kp-after-mem-raw-callsite-migration.json -j 15` -> `229/229 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-after-kp-memraw-migration-full.json -j 15` -> `725/725 pass`
- 迥ｶ豕・
  - `mem` 縺ｮ逕溘い繝ｭ繧ｱ繝ｼ繧ｿ蛻ｩ逕ｨ邂・園縺ｯ `kpread/kpwrite` 縺ｧ `*_raw` 縺ｸ霑ｽ蠕捺ｸ医∩縲・
  - 谺｡谿ｵ縺ｯ `alloc/realloc/dealloc` 蜈ｬ髢句錐繧・Result/Option 螳牙・API縺ｸ蛻・ｊ譖ｿ縺医ｋ貅門ｙ縺ｨ縺励※縲∵ｮ九ｊ蜻ｼ縺ｳ蜃ｺ縺礼ｮ・園繧呈ｮｵ髫守ｧｻ陦後☆繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ `core/mem` 縺ｫ `*_raw` 髫秘屬繧貞ｰ主・)

- 逶ｮ逧・
  - 逕溘・繧､繝ｳ繧ｿAPI繧呈ｮｵ髫守噪縺ｫ蛻・屬縺励�∵ｬ｡谿ｵ縺ｮ螳牙・API讓呎ｺ門錐蛹悶↓蛯吶∴繧九�・
- 螟画峩:
  - `stdlib/core/mem.nepl`
    - 逕蘗PI譛ｬ菴薙ｒ `alloc_raw` / `realloc_raw` / `dealloc_raw` 縺ｸ謾ｹ蜷阪�・
    - `alloc` / `realloc` / `dealloc` 縺ｯ `*_raw` 縺ｸ縺ｮ蟋碑ｭｲ繧ｨ繧､繝ｪ繧｢繧ｹ縺ｸ螟画峩縲・
    - `alloc_result` / `realloc_result` / `dealloc_result` 縺ｨ `alloc_ptr` 邉ｻ縺ｯ `*_raw` 繧堤峩謗･蜻ｼ縺ｶ繧医≧縺ｫ螟画峩縲・
- 繝・せ繝・
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety-after-raw-alias.json -j 15` -> `213/213 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-after-mem-raw-alias.json -j 15` -> `725/725 pass`
- 迥ｶ豕・
  - `mem` 蛛ｴ縺ｧ縲檎函API譛ｬ菴薙�阪→縲悟・髢句錐縲阪ｒ蛻・屬縺ｧ縺阪◆縲・
  - 谺｡谿ｵ縺ｯ `alloc/realloc/dealloc` 蜈ｬ髢句錐繧貞ｮ牙・API縺ｸ蛻・ｊ譖ｿ縺医ｋ髫帙・蜻ｼ縺ｳ蜃ｺ縺怜・遘ｻ陦鯉ｼ・tdlib/tests/tutorials・峨↓逹�謇九〒縺阪ｋ迥ｶ諷九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺE蜑埼�ｲ: `mem_result` 邉ｻAPI縺ｮ蝗槫ｸｰ繝・せ繝郁ｿｽ蜉�)

- 逶ｮ逧・
  - `core/mem` 縺ｮ `alloc_result/realloc_result/dealloc_result` 蜻ｽ蜷榊､画峩繧偵ユ繧ｹ繝医〒蝗ｺ螳壹☆繧九�・
- 螟画峩:
  - `tests/memory_safety.n.md`
    - `alloc_result/dealloc_result` 縺ｮ豁｣蟶ｸ邉ｻ繝・せ繝医ｒ霑ｽ蜉�縲・
    - `dealloc_result` 縺ｮ辟｡蜉ｹ蠑墓焚 `Err` 霑泌唆繝・せ繝医ｒ霑ｽ蜉�縲・
- 繝・せ繝・
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety-after-result-rename.json -j 15` -> `213/213 pass`
- 迥ｶ豕・
  - `core/mem` 縺ｮ `_safe` 蜻ｽ蜷埼勁蜴ｻ蛻・↓縺､縺・※縲∝多蜷榊､画峩蠕後・譛�蟆丞屓蟶ｰ繧貞崋螳壹＠縺溘�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ `core/mem` 縺ｮ `_safe` 蜻ｽ蜷埼勁蜴ｻ)

- 逶ｮ逧・
  - `core/mem` 縺ｮ螳牙・繝ｩ繝・ヱAPI縺九ｉ `_safe` 謗･蟆ｾ霎槭ｒ髯､蜴ｻ縺励�∝多蜷崎ｦ冗ｴ・ｒ谺｡谿ｵ遘ｻ陦後＠繧・☆縺・ｽ｢縺ｸ謠・∴繧九�・
- 螟画峩:
  - `stdlib/core/mem.nepl`
    - `alloc_safe` -> `alloc_result`
    - `realloc_safe` -> `realloc_result`
    - `dealloc_safe` -> `dealloc_result`
    - 髢｢騾｣繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝亥・縺ｮ髢｢謨ｰ蜷阪・豕ｨ諢丈ｺ矩�・ｒ譖ｴ譁ｰ縲・
  - `todo.md`
    - 繝輔ぉ繝ｼ繧ｺD縺ｮ譁・ｨ�繧偵�～_safe` 邨ｱ荳�譁ｹ驥昴°繧峨�形_safe` 謗･蟆ｾ霎槫ｻ・ｭ｢・句ｮ牙・API讓呎ｺ門錐蛹悶�阪∈譖ｴ譁ｰ縲・
    - `move/effect` 蜿肴丐鬆・岼繧偵�～mem` 蛛ｴ縺ｨ `kpread/kpwrite` 蛛ｴ縺ｮ谿倶ｻｶ縺ｫ蛻・牡縺励※譏手ｨ倥�・
- 繝・せ繝・
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-after-mem-safe-rename.json -j 15` -> `723/723 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-mem-safe-rename.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - `_safe` 蜻ｽ蜷埼勁蜴ｻ縺ｯ `core/mem` 縺ｧ逹�謇区ｸ医∩縲・
  - 谺｡谿ｵ縺ｯ API 譛ｬ菴薙ｒ Result/Option 讓呎ｺ門錐縺ｸ蟇・○繧九◆繧√�～alloc/realloc/dealloc` 縺ｮ逕溘・繧､繝ｳ繧ｿAPI謨ｴ逅・ｼ・*_raw` 髫秘屬・峨↓騾ｲ繧�縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ kpread/kpwrite 縺ｮ `_raw` 蜷肴紛逅・ｮ御ｺ・

- 逶ｮ逧・
  - `kpread/kpwrite` 縺ｧ谿九▲縺ｦ縺・◆ `_raw` 謗･蟆ｾ霎槭・蜈ｬ髢句錐繧呈紛逅・＠縲・�壼ｸｸAPI蜷阪∈邨ｱ荳�縺吶ｋ縲・
  - 螟画峩蠕後・蜈ｨ菴灘屓蟶ｰ繧・`tests + stdlib + tutorials` 縺ｧ遒ｺ隱阪☆繧九�・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - `scanner_new_raw` 繧・`scanner_new_i32` 縺ｸ螟画峩縲・
    - `scanner_skip_ws_raw` / `scanner_is_eof_raw` / `scanner_skip_token_raw` / `scanner_read_*_raw` 繧・`scanner_*` 縺ｸ邨ｱ荳�縲・
    - 繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝井ｸｭ縺ｮ髢｢謨ｰ蜷崎ｨ倩ｿｰ繧ょｮ滉ｽ薙↓蜷医ｏ縺帙※譖ｴ譁ｰ縲・
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_raw` 繧・`writer_new_i32` 縺ｸ螟画峩縲・
    - `writer_write_*_raw` / `writer_writeln_raw` / `writer_flush_raw` / `writer_free_raw` 繧・`writer_*` 縺ｸ邨ｱ荳�縲・
    - 繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝井ｸｭ縺ｮ髢｢謨ｰ蜷崎ｨ倩ｿｰ繧ょｮ滉ｽ薙↓蜷医ｏ縺帙※譖ｴ譁ｰ縲・
- 繝・せ繝・
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl --no-stdlib --no-tree -o /tmp/tests-kpread-kpwrite-no-raw.json -j 15` -> `5/5 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-full-after-kp-overload-unify.json -j 15` -> `781/781 pass`
- 迥ｶ豕・
  - `kpread/kpwrite` 縺九ｉ `_raw` 謗･蟆ｾ霎槭・隗｣豸域ｸ医∩縲・
  - `todo.md` 縺ｮ `_safe/_raw` 譛�邨よ紛逅・・ `mem.nepl` 蛛ｴ・・alloc_safe/realloc_safe/dealloc_safe`・峨′谿倶ｻｶ縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ Scanner/Writer API荳�譛ｬ蛹悶→繝上Φ繝峨Ν髴ｲ蜃ｺ髯､蜴ｻ)

- 逶ｮ逧・
  - `kpread/kpwrite` 縺ｮ蜈ｬ髢帰PI縺九ｉ `scanner_handle/writer_handle` 繧帝勁蜴ｻ縺励�～Scanner`/`Writer` 蝙帰PI縺ｸ荳�譛ｬ蛹悶☆繧九�・
  - `Scanner` 蜻ｼ縺ｳ蜃ｺ縺励′ move 縺ｧ遐ｴ邯ｻ縺吶ｋ譬ｹ譛ｬ蜴溷屏・医さ繝ｳ繝代う繝ｩ縺ｮ髱曚opy迚ｹ萓具ｼ峨ｒ荳頑ｵ√〒菫ｮ豁｣縺吶ｋ縲・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
    - 逕溘ワ繝ｳ繝峨Ν螳溯｣・ｒ `*_raw` 蜷阪∈蛻・屬縲・
    - 蜈ｬ髢矩未謨ｰ縺ｯ `Scanner` 蠑墓焚縺ｮ騾壼ｸｸ蜷搾ｼ・scanner_read_i32` 縺ｪ縺ｩ・峨↓邨ｱ荳�縲・
    - `scanner_handle` 逶ｸ蠖薙・蜈ｬ髢矩未謨ｰ繧貞炎髯､縺励�∝・驛ｨ縺ｧ縺ｮ縺ｿ `mem_ptr_addr get sc "raw"` 繧剃ｽｿ逕ｨ縲・
  - `stdlib/kp/kpwrite.nepl`
    - 逕溘ワ繝ｳ繝峨Ν螳溯｣・ｒ `*_raw` 蜷阪∈蛻・屬縲・
    - 蜈ｬ髢矩未謨ｰ縺ｯ `Writer` 蠑墓焚縺ｮ騾壼ｸｸ蜷搾ｼ・writer_write_i32` 縺ｪ縺ｩ・峨↓邨ｱ荳�縲・
    - `writer_handle` 逶ｸ蠖薙・蜈ｬ髢矩未謨ｰ繧貞炎髯､縺励�∝・驛ｨ縺ｧ縺ｮ縺ｿ `mem_ptr_addr get w "raw"` 繧剃ｽｿ逕ｨ縲・
  - 萓晏ｭ倡ｮ・園縺ｮ遘ｻ陦・
    - `tests/kp.n.md`, `tests/kp_i64.n.md`, `tests/stdin.n.md`
    - `tutorials/getting_started/22_*.n.md`, `24_*.n.md`, `25_*.n.md`, `27_*.n.md`
    - `examples/kp_fizzbuzz.nepl`
    - `stdlib/kp/kpgraph.nepl`・・dense_graph_read_undirected_1indexed` 繧・`Scanner` 蜿励￠蜿悶ｊ縺ｸ螟画峩・・
  - 荳頑ｵ∽ｿｮ豁｣:
    - `nepl-core/src/types.rs` 縺ｮ譏守､ｺ髱曚opy蛻､螳壹°繧・`Scanner` 繧帝勁螟厄ｼ・RegionToken`/`Writer` 縺ｯ邯ｭ謖・ｼ峨�・
- 繝・せ繝・
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpgraph.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -i examples/kp_fizzbuzz.nepl --no-tree -o /tmp/tests-kp-api-unify.json -j 15` -> `231/231 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-full-after-kp-api-unify.json -j 15` -> `781/781 pass`
- 迥ｶ豕・
  - `kpread/kpwrite` 縺ｮ蜈ｬ髢帰PI縺ｯ `Scanner`/`Writer` 蝙九・繝ｼ繧ｹ縺ｫ謠・▲縺溘�・
  - 谺｡谿ｵ縺ｯ `todo.md` 繝輔ぉ繝ｼ繧ｺD縺ｮ谿倶ｻｶ・・_safe` 蟒・ｭ｢縺ｨ `_raw` 譛�邨ょ炎髯､縲》rait 蠅・阜蟆主・・峨ｒ騾ｲ繧√ｋ縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD蜑埼�ｲ: ptr螳牙・API縺ｮ _safe 萓晏ｭ伜・繧企屬縺・

- 逶ｮ逧・
  - `mem` 縺ｮ蜈ｬ髢・`Result` API 繧・`_safe` 繝ｩ繝・ヱ蜷阪°繧臥峡遶九＆縺帙�～_safe` 蟒・ｭ｢縺ｫ蜷代￠縺滓ｮｵ髫守ｧｻ陦後ｒ騾ｲ繧√ｋ縲・
- 螟画峩:
  - `stdlib/core/mem.nepl`
    - `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` 縺ｮ蜀・Κ螳溯｣・ｒ `alloc_safe/realloc_safe/dealloc_safe` 蜻ｼ縺ｳ蜃ｺ縺励°繧牙・髮｢縲・
    - `alloc` / `realloc` / `dealloc` 繧堤峩謗･蜻ｼ縺ｳ縲∝・髢帰PI蛛ｴ縺ｧ `Result` 蛻､螳壹ｒ陦後≧繧医≧縺ｫ螟画峩縲・
- 繝・せ繝・
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety-after-ptr-safe-decouple.json -j 15` -> `211/211 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-full-after-ptr-safe-decouple.json -j 15` -> `723/723 pass`
- 迥ｶ豕・
  - `*_ptr` 邉ｻ縺ｮ蜈ｬ髢句ｮ牙・API縺ｯ `_safe` 蜷阪↓萓晏ｭ倥＠縺ｪ縺・ｽ｢縺ｸ遘ｻ陦梧ｸ医∩縲・
  - 谺｡谿ｵ縺ｧ縺ｯ `alloc_safe/realloc_safe/dealloc_safe` 閾ｪ菴薙ｒ邵ｮ騾�縺励�∝・髢句錐荳�譛ｬ蛹悶∈騾ｲ繧√ｋ縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺE蜑埼�ｲ: memory_safety 蝗槫ｸｰ霑ｽ蜉�)

- 逶ｮ逧・
  - `todo.md` 繝輔ぉ繝ｼ繧ｺE縺ｮ霑ｽ蜉�鬆・岼 `tests/memory_safety.n.md` 繧貞・陦後〒蝗ｺ螳壼喧縺吶ｋ縲・
- 螟画峩:
  - `tests/memory_safety.n.md` 繧呈眠隕剰ｿｽ蜉�縲・
    - `alloc_ptr/load_i32_ptr/store_i32_ptr/dealloc_ptr` 縺ｮ豁｣蟶ｸ邉ｻ縲・
    - 辟｡蜉ｹ繝昴う繝ｳ繧ｿ `load` 縺・`Option::None` 繧定ｿ斐☆逡ｰ蟶ｸ邉ｻ縲・
    - 辟｡蜉ｹ繝昴う繝ｳ繧ｿ `store` 縺・`Result::Err` 繧定ｿ斐☆逡ｰ蟶ｸ邉ｻ縲・
- 繝・せ繝・
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i stdlib/core/mem.nepl --no-tree -o /tmp/tests-memory-safety.json -j 15` -> `211/211 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-full-after-memory-safety-tests.json -j 15` -> `723/723 pass`
- 迥ｶ豕・
  - `tests/memory_safety.n.md` 霑ｽ蜉�繧ｿ繧ｹ繧ｯ縺ｯ螳御ｺ・＠縲～todo.md` 縺九ｉ蜑企勁貂医∩縲・
  - 谺｡縺ｯ `mem/kpread/kpwrite` 縺ｮ `_safe` 縺ｪ縺怜ｮ牙・API荳�譛ｬ蛹悶→ `_raw` 譛�邨ょ炎髯､縺ｸ騾ｲ繧�縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC逹�謇・ MemPtr 縺ｮ繧ｸ繧ｧ繝阪Μ繧ｯ繧ｹ蛹・

- 逶ｮ逧・
  - `doc/memory_safety_compiler_design.md` 縺ｮ蝙九Δ繝・Ν縺ｫ豐ｿ縺｣縺ｦ縲～MemPtr<T>` 繧貞・髢帰PI蛛ｴ縺ｸ蜿肴丐縺吶ｋ縲・
- 螟画峩:
  - `stdlib/core/mem.nepl`
    - `MemPtr` 繧・`MemPtr<.T>` 縺ｸ螟画峩縲・
    - `mem_ptr_wrap` / `mem_ptr_addr` / `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` / `mem_ptr_add` 繧偵ず繧ｧ繝阪Μ繧ｯ繧ｹ蟇ｾ蠢懊�・
    - `load_i32_ptr` / `store_i32_ptr` 縺ｯ `MemPtr<i32>`縲～load_u8_ptr` / `store_u8_ptr` 縺ｯ `MemPtr<u8>` 繧貞女縺代ｋ繧医≧縺ｫ螟画峩縲・
  - `stdlib/kp/kpread.nepl`
    - `Scanner.raw` 繧・`MemPtr<u8>` 蛹悶�・
  - `stdlib/kp/kpwrite.nepl`
    - `Writer.raw` 繧・`MemPtr<u8>` 蛹悶�・
- 繝・せ繝・
  - `node nodesrc/tests.js -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-mem-kp-generic-memptr.json -j 15` -> `220/220 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-full-after-memptr-generic.json -j 15` -> `720/720 pass`
- 迥ｶ豕・
  - `MemPtr<T>` 蝙九Δ繝・Ν縺ｯ蟆主・貂医∩・亥・髢帰PI縺ｮ i32 逕溘・繧､繝ｳ繧ｿ髯､蜴ｻ縺ｯ邯咏ｶ夲ｼ峨�・
  - 谺｡縺ｯ `RegionToken` 蟆主・縺ｨ `alloc/realloc/dealloc` 縺ｮ `Result` 荳�譛ｬ蛹悶ｒ騾ｲ繧√ｋ縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺB螳御ｺ・ Copy/Clone 蛻ｶ邏・+ RegionToken 髱曚opy蛹・

- 逶ｮ逧・
  - `todo.md` 繝輔ぉ繝ｼ繧ｺB谿倶ｻｶ縺�縺｣縺・`Copy/Clone` 蛻ｶ邏・､懈渊縺ｨ `RegionToken` 髱曚opy謇ｱ縺・ｒ蝙区､懈渊縺ｫ蜿肴丐縺吶ｋ縲・
- 螟画峩:
  - `nepl-core/src/types.rs`
    - `TypeCtx::is_copy` 縺ｫ譏守､ｺ髱曚opy蝙句愛螳壹ｒ霑ｽ蜉�・・RegionToken` / `Scanner` / `Writer`・峨�・
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3049` (`TypeCopyImplTargetNotCopy`) 縺ｨ `D3050` (`TypeCopyImplRequiresClone`) 繧定ｿｽ蜉�縲・
  - `nepl-core/src/typecheck.rs`
    - `impl Copy for T` 縺ｮ蜿朱寔譎ゅ↓ `ctx.is_copy(T)` 繧呈､懆ｨｼ縺励�・撼Copy蟇ｾ雎｡繧・`D3049` 縺ｧ諡貞凄縲・
    - `Copy` 螳溯｣・↓縺ｯ蜷御ｸ�蟇ｾ雎｡ `Clone` 螳溯｣・′蠢・ｦ√↑讀懈渊繧定ｿｽ蜉�縺励�∵ｬ�關ｽ譎・`D3050` 縺ｧ諡貞凄縲・
    - 諡貞凄蟇ｾ雎｡縺ｮ `Copy` 螳溯｣・・蠕檎ｶ壹・ impl 蜿朱寔/辣ｧ蜷医°繧蛾勁螟悶�・
  - `tests/move_effect.n.md`
    - `D3049`/`D3050` 縺ｮ compile_fail 繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
    - `Clone+Copy` 荳｡螳溯｣・凾縺ｮ謌仙粥繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
    - `RegionToken` 縺ｮ move 蠕悟・蛻ｩ逕ｨ諡貞凄繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
- 繝・せ繝・
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/move_effect.n.md --no-tree -o /tmp/tests-move-effect-copy-clone.json -j 15` -> `218/218 pass`
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md -i tests/typeannot.n.md --no-tree -o /tmp/tests-move-overload-typeannot-copyclone.json -j 15` -> `266/266 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-full-after-copy-clone.json -j 15` -> `720/720 pass`
- 迥ｶ豕・
  - 繝輔ぉ繝ｼ繧ｺB縺ｮ `Copy/Clone` 蛻ｶ邏・→ `RegionToken` 髱曚opy蛹悶・蜿肴丐貂医∩縲・
  - 谺｡縺ｯ `todo.md` 縺ｮ繝輔ぉ繝ｼ繧ｺC/D・・MemPtr<T>` 縺ｨ `mem/kpread/kpwrite` 縺ｮ螳牙・API荳�譛ｬ蛹厄ｼ峨∈騾ｲ繧�縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺB騾ｲ陦・ move_check 縺ｫ borrow 迥ｶ諷矩・遘ｻ繧貞ｮ溯｣・
- 逶ｮ逧・
  - `todo.md` 縺ｮ繝輔ぉ繝ｼ繧ｺB縺ｫ縺ゅｋ `move_check` 迥ｶ諷区ｩ滓｢ｰ繧・`BorrowedShared/BorrowedUnique` 縺ｾ縺ｧ諡｡蠑ｵ縺励�∝・蟯・繝ｫ繝ｼ繝・match 蜷域ｵ√ｒ菫晏ｮ育噪縺ｫ豁｣縺励￥謇ｱ縺・�・
- 螳溯｣・
  - `nepl-core/src/passes/move_check.rs`
    - `VarState` 縺ｫ `BorrowedShared` / `BorrowedUnique` 繧定ｿｽ蜉�縲・
    - `BorrowKind` 繧貞ｰ主・縺励�～visit_borrow` 繧・`Shared/Unique` 蛹ｺ蛻･縺ｧ蜃ｦ逅・�・
    - `check_use` 繧呈峩譁ｰ縺励�｜orrow 荳ｭ move 繧・unique borrow 荳ｭ use 繧呈拠蜷ｦ縲・
    - `check_assign` / `check_drop` / `check_borrow` 繧定ｿｽ蜉�縺励�∽ｻ｣蜈･繝ｻdrop繝ｻborrow 縺ｧ縺ｮ迥ｶ諷矩・遘ｻ繧剃ｸ�蜈・喧縲・
    - `merge_state_pair` / `merge_states` 繧定ｿｽ蜉�縺励�～if`/`match`/`while` 蜷域ｵ√ｒ `Valid/Borrowed/Moved/PossiblyMoved` 縺ｧ邨ｱ荳�縲・
    - `Intrinsic::load/store` 縺ｮ繧｢繝峨Ξ繧ｹ蠑墓焚 borrow 蛻､螳壹ｒ `BorrowKind` 縺ｫ謗･邯壹�・
  - `tests/move_effect.n.md`
    - 髱曚opy蛟､縺ｮ shared borrow 荳ｭ move 縺梧拠蜷ｦ縺輔ｌ繧句屓蟶ｰ繧定ｿｽ蜉�縲・
    - Copy蛟､ borrow 縺悟茜逕ｨ繧帝仆螳ｳ縺励↑縺・屓蟶ｰ繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md -i tests/typeannot.n.md --no-tree -o /tmp/tests-move-overload-typeannot.json -j 15` -> `262/262 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-full-after-move-borrow.json -j 15` -> `716/716 pass`
- 谺｡:
  - 繝輔ぉ繝ｼ繧ｺB谿倶ｻｶ (`Copy/Clone` trait蛻ｶ邏・､懈渊, `RegionToken` 豸郁ｲｻ隕丞援) 縺ｫ騾ｲ繧�縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺB逹�謇・ `TypeCtx::is_copy` 讒矩��蝙句愛螳・
- 逶ｮ逧・
  - 繝輔ぉ繝ｼ繧ｺB縺ｮ譛�蛻昴・螳溯｣・→縺励※縲～TypeCtx::is_copy` 繧・tuple/struct/enum 縺ｨ generic apply 縺ｸ諡｡蠑ｵ縺吶ｋ縲・
  - 蜀榊ｸｰ讀懷・繝ｭ繧ｸ繝・け縺ｮ隱､蛻､螳夲ｼ亥酔荳�蝙九・蜀崎ｨｪ縺ｧ蟶ｸ縺ｫ false・峨ｒ隗｣豸医☆繧九�・
- 螳溯｣・
  - `nepl-core/src/types.rs`
    - `is_copy_inner` 繧・`visiting + mapping` 譁ｹ蠑上↓螟画峩縲・
    - `TypeKind::Struct` / `TypeKind::Enum` 繧呈ｧ矩��逧・・蟶ｰ蛻､螳壹∈螟画峩縲・
    - `TypeKind::Apply` 縺ｧ base 縺ｮ type parameter 繧貞ｮ溷ｼ墓焚縺ｸ譚溽ｸ帙＠縺ｦ copy 蛻､螳壹〒縺阪ｋ繧医≧蟇ｾ蠢懊�・
    - 蛻､螳夂ｵゆｺ・凾縺ｫ `visiting.remove` 繧定｡後＞縲∝・蠑溘ヮ繝ｼ繝牙・險ｪ縺ｧ縺ｮ蛛ｽ髯ｰ諤ｧ繧定ｧ｣豸医�・
  - `tests/move_effect.n.md`
    - Copy 繝輔ぅ繝ｼ繝ｫ繝峨・縺ｿ縺ｮ struct 蜀榊茜逕ｨ繧ｱ繝ｼ繧ｹ・域・蜉滂ｼ・
    - `Apply` 縺輔ｌ縺・generic struct 蜀榊茜逕ｨ繧ｱ繝ｼ繧ｹ・域・蜉滂ｼ・
    - payload 縺・Copy 縺ｮ enum 蜀榊茜逕ｨ繧ｱ繝ｼ繧ｹ・域・蜉滂ｼ・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/generics.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-moveeffect-generics-overload.json -j 15` -> `269/269 pass`
- 谺｡:
  - move_check 蛛ｴ縺ｮ迥ｶ諷矩・遘ｻ・・PossiblyMoved` 蜷域ｵ√�｜orrow 迥ｶ諷具ｼ峨ｒ `is_copy` 諡｡蠑ｵ縺ｫ蜷医ｏ縺帙※邊ｾ譟ｻ縺吶ｋ縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (2026-03-03 繝輔ぉ繝ｼ繧ｺA螳御ｺ・ raw/intrinsic effect 荳�蜈・喧)
- 逶ｮ逧・
  - 繝輔ぉ繝ｼ繧ｺA谿倶ｻｶ縺�縺｣縺溘�景ntrinsic / raw target body 縺ｮ effect 蛻､螳壻ｸ�蜈・喧縲阪ｒ螳溯｣・＠縲｝ure 譁・ц縺九ｉ縺ｮ I/O 繧貞梛讀懈渊谿ｵ髫弱〒諡貞凄縺吶ｋ縲・
- 螳溯｣・
  - `nepl-core/src/typecheck.rs`
    - `IMPURE_IO_EFFECT_MARKERS` 繧定ｿｽ蜉�縺励�！/O隱槫ｽ吶ユ繝ｼ繝悶Ν繧貞ｰ主・縲・
    - `intrinsic_effect` / `raw_lines_effect` / `raw_body_effect` 繧定ｿｽ蜉�縺励※ effect 蛻､螳壹ｒ蜈ｱ騾壼喧縲・
    - `BlockChecker::validate_raw_body_effect` 繧定ｿｽ蜉�縺励�～#wasm`/`#llvmir` 譛ｬ菴薙′ I/O隱槫ｽ吶ｒ蜷ｫ繧�蝣ｴ蜷医�｝ure 髢｢謨ｰ縺ｧ `D3025` 繧定ｿ斐☆繧医≧縺ｫ螟画峩縲・
    - `FnBody::Parsed` 縺ｮ target驕ｸ謚柮aw譛ｬ菴薙�√♀繧医・ `FnBody::Wasm` / `FnBody::LlvmIr` 逶ｴ謖・ｮ壹・荳｡譁ｹ縺ｧ蜷後§讀懈渊繧貞ｮ滓命縲・
    - `PrefixItem::Intrinsic` 縺ｧ繧ょ・騾・effect 蛻､螳壹ｒ騾壹☆繧医≧螟画峩縲・
  - `tests/move_effect.n.md`
    - pure raw body 縺ｧ `fd_write` 繧貞性繧�繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�・・compile_fail`, `diag_id: 3025`・峨�・
  - `todo.md`
    - 螳御ｺ・ｸ医∩繝輔ぉ繝ｼ繧ｺA鬆・岼繧貞炎髯､縺励�∵悴螳後・縺ｿ縺ｸ謨ｴ逅・�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md -i tests/typeannot.n.md -i tests/intrinsic.n.md --no-tree -o /tmp/tests-effect-overload-typeannot-intrinsic.json -j 15` -> `263/263 pass`
- 迴ｾ迥ｶ:
  - 繝輔ぉ繝ｼ繧ｺA・・ffect隕丞援縺ｮ蜿肴丐・峨・螳御ｺ・�・
  - 谺｡縺ｯ繝輔ぉ繝ｼ繧ｺB・・TypeCtx::is_copy` 諡｡蠑ｵ縺ｨ move/borrow 迥ｶ諷矩・遘ｻ縺ｮ蜴ｳ蟇・喧・峨∈騾ｲ繧�縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (2026-03-03 繝輔ぉ繝ｼ繧ｺA蜀埼幕: effect險ｺ譁ｭID縺ｨ蝗槫ｸｰ霑ｽ蜉�)
- 逶ｮ逧・
  - `todo.md` 縺ｮ 2026-03-03 險育判繝輔ぉ繝ｼ繧ｺA繧貞・髢九＠縲｝ure/impure 蛻､螳壹・險ｺ譁ｭ蝗ｺ螳壹ｒ騾ｲ繧√ｋ縲・
- 螳溯｣・
  - `nepl-core/src/diagnostic_ids.rs`
    - `D3025 TypePureCallsImpureFunction` 繧定ｿｽ蜉�縲・
  - `nepl-core/src/typecheck.rs`
    - 縲継ure context cannot call impure function縲阪・蜈ｨ逋ｺ逕溽ｮ・園縺ｫ `D3025` 繧剃ｻ倅ｸ弱�・
  - `tests/move_effect.n.md` 繧呈眠隕剰ｿｽ蜉�縲・
    - pure 縺九ｉ繝｡繝｢繝ｪ謫堺ｽ懊ｒ蜻ｼ縺ｹ繧九こ繝ｼ繧ｹ・域・蜉滂ｼ・
    - pure 縺九ｉ impure 髢｢謨ｰ蜻ｼ縺ｳ蜃ｺ縺玲拠蜷ｦ・・diag_id: 3025`・・
    - 繝ｭ繝ｼ繧ｫ繝ｫ `set` 縺・pure 縺ｮ縺ｾ縺ｾ菴ｿ縺医ｋ繧ｱ繝ｼ繧ｹ・域・蜉滂ｼ・
    - 繧ｰ繝ｭ繝ｼ繝舌Ν `set` 縺・impure 縺ｫ縺ｪ繧九こ繝ｼ繧ｹ・・diag_id: 3025`・・
  - `todo.md`
    - 螳御ｺ・ｸ医∩鬆・岼・・builtins` 縺ｮ繝｡繝｢繝ｪ邉ｻ Pure 蛹悶�‘ntry 蠑ｷ蛻ｶ Impure 迚ｹ萓九・蜑企勁・峨ｒ繝輔ぉ繝ｼ繧ｺA縺九ｉ蜑企勁縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md -i tests/typeannot.n.md --no-tree -o /tmp/tests-move-effect-overload-typeannot.json -j 15` -> `256/256 pass`
- 谺｡:
  - 繝輔ぉ繝ｼ繧ｺA谿倶ｻｶ縺ｮ縲景ntrinsic / raw target body 縺ｮ effect 荳�蜈・愛螳壹�阪ｒ螳溯｣・☆繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝我ｿｮ豁｣縺ｮ螳御ｺ・→ 2026-03-03 險育判縺ｸ縺ｮ蠕ｩ蟶ｰ)
- 逶ｮ逧・
  - 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝芽ｧ｣豎ｺ縺ｮ荳榊ｮ牙ｮ夂ｮ・園・磯未謨ｰ蛟､蠑墓焚繝ｻpipe 菴ｵ逕ｨ繝ｻ蝙区ｳｨ驥域ｷｷ蝨ｨ・峨ｒ譬ｹ譛ｬ菫ｮ豁｣縺励�～todo.md` 縺ｮ `2026-03-03 move/effect/memory` 螳溯｣・∈蠕ｩ蟶ｰ縺吶ｋ縲・
- 螳溯｣・
  - `nepl-core/src/typecheck.rs`
    - 髢｢謨ｰ繧ｷ繧ｰ繝阪メ繝｣蜿ら・繧・`function_signature_for_entry` 縺ｫ髮・ｴ・＠縲》ype_args 驕ｩ逕ｨ蠕後・蠑墓焚蝙九ｒ荳�雋ｫ蜿門ｾ励☆繧九ｈ縺・ｿｮ豁｣縲・
    - pipe 豕ｨ蜈･譎ゅ↓ nullary callable 縺ｮ驕取掠 reduce 繧帝∩縺代ｋ蛻ｶ蠕｡縺ｨ縲》arget 蜈･蜉帛梛繧剃ｽｿ縺｣縺・`reduce_pipe_pending_value_with_target` 繧定ｿｽ蜉�縲・
    - 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝牙�呵｣懊・邨槭ｊ霎ｼ縺ｿ縺ｧ縲悟・菴灘梛蛟呵｣懷━蜈医�阪�悟梛繝代Λ繝｡繝ｼ繧ｿ謨ｰ譛�蟆丞�呵｣懷━蜈医�阪ｒ蟆主・縺励�～D3005` 縺ｮ驕取､懷・繧呈椛蛻ｶ縲・
  - `tests/overload.n.md`, `tests/typeannot.n.md`
    - 繝悶Ο繝・け豕ｨ驥・髢｢謨ｰ蜻ｼ縺ｳ蜃ｺ縺玲ｳｨ驥・pipe 豕ｨ驥・髢｢謨ｰ繝ｪ繝・Λ繝ｫ豕ｨ驥医・豺ｷ蝨ｨ繧ｱ繝ｼ繧ｹ繧呈僑蜈・＠縲∽ｻ雁屓縺ｮ菫ｮ豁｣轤ｹ繧貞屓蟶ｰ蝗ｺ螳壹�・
  - `stdlib/alloc/collections/vec.nepl`, `stdlib/alloc/collections/stack.nepl`, `stdlib/tests/stack.n.md`
    - `push` 蛻ｩ逕ｨ蠖｢縺ｨ蝙区耳隲悶こ繝ｼ繧ｹ繧呈紛逅・＠縲√が繝ｼ繝舌・繝ｭ繝ｼ繝芽ｧ｣豎ｺ縺ｮ螳滄°逕ｨ繧ｱ繝ｼ繧ｹ繧貞崋螳壹�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/typeannot.n.md -i stdlib/alloc/collections/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/tests/stack.n.md --no-tree -o /tmp/tests-overload-typeannot-vec-stack.json -j 15` -> `286/286 pass`
- 迴ｾ迥ｶ:
  - 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝我ｿｮ豁｣縺ｯ螳御ｺ・�・
  - 谺｡縺ｯ `todo.md` 縺ｮ `2026-03-03 move/effect/memory 譛ｬ譬ｼ螳溯｣・ｨ育判` 繝輔ぉ繝ｼ繧ｺA・・ffect隕丞援縺ｮ繧ｳ繝ｳ繝代う繝ｩ蜿肴丐・峨ｒ蜀埼幕縺吶ｋ縲・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (pipe 豢ｻ逕ｨ縺ｨ `push` 謗ｨ隲悶・遒ｺ隱・
- 逶ｮ逧・
  - 譌｢蟄俶嶌縺肴鋤縺域婿驥昴→縺励※縲｝ipe 貍皮ｮ怜ｭ舌ｒ豢ｻ逕ｨ縺励※荳ｭ髢灘､画焚縺ｨ繧､繝ｳ繝・Φ繝医ｒ謚代∴繧九�・
  - `vec_push<i32> ...` 縺ｧ縺ｯ縺ｪ縺・`push ...` 縺�縺代〒蝙区耳隲悶〒縺阪ｋ蛻ｩ逕ｨ蠖｢繧呈・遉ｺ縺吶ｋ縲・
- 螳滓命:
  - `stdlib/alloc/collections/list.nepl`
    - doctest 縺ｮ繝ｪ繧ｹ繝域ｧ狗ｯ峨ｒ `list_nil |> list_push_front ...` 縺ｸ螟画峩縲・
    - move 隕丞援縺ｫ蜷医ｏ縺帙※蜀榊茜逕ｨ邂・園繧貞・譚溽ｸ帙∈謨ｴ逅・�・
    - 螳溯｣・・荳�驛ｨ縺ｧ荳ｭ髢灘､画焚繧貞炎貂幢ｼ・list_len`, `list_get`, `list_free`, `list_reverse`・峨�・
  - `stdlib/alloc/collections/vec.nepl`
    - doctest 縺ｮ `vec_push<i32>` / `push<i32>` 繧・`push` 縺ｫ邨ｱ荳�縲・
    - `vec_new<i32> |> push 10 |> push 20` 縺ｮ蠖｢縺ｸ螟画峩縺励�∝梛蠑墓焚逵∫払縺ｧ謌千ｫ九☆繧倶ｾ九∈譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/list.nepl -i stdlib/alloc/collections/vec.nepl --no-stdlib --no-tree -o /tmp/tests-list-vec-pipe.json -j 15` -> `28/28 pass`
  - `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-stdlib --no-tree -o /tmp/tests-vec-push-infer.json -j 15` -> `17/17 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-plus-tests-after-push-alias.json -j 15` -> `700/700 pass`

# 2026-03-03 菴懈･ｭ繝｡繝｢ (莉墓ｧ俶怙邨ら｢ｺ隱・ 蜑咲ｽｮ險俶ｳ・繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝画紛蜷・
- 螳滓命:
  - `doc/move_effect_spec.md` 縺ｫ縲君EPLg2譌｢蟄倅ｻ墓ｧ倥→縺ｮ謨ｴ蜷医�咲ｫ�繧定ｿｽ蜉�縲・
  - 蜑咲ｽｮ險俶ｳ輔�∝梛豕ｨ驥医�√が繝ｼ繝舌・繝ｭ繝ｼ繝峨�∵囓鮟冂ast遖∵ｭ｢縺ｨ縺ｮ謨ｴ蜷医ｒ譏手ｨ倥�・
  - 蜷悟錐繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨・ effect 荳�閾ｴ蛻ｶ邏・ｒ莉墓ｧ倥∈蜿肴丐縲・
- 邨先棡:
  - 險ｭ險域婿驥晢ｼ医Γ繝｢繝ｪ謫堺ｽ・pure / I/O 縺ｮ縺ｿ impure・峨→譌｢蟄倩ｨ�隱樔ｻ墓ｧ倥・隲也炊遏帷崟縺ｯ辟｡縺励�・
  - 螳溯｣・悴蜿肴丐邂・園・・uiltins 縺ｮ effect, entry 迚ｹ萓具ｼ峨・蠑輔″邯壹″ `todo.md` 邂｡逅・�・

# 2026-03-03 菴懈･ｭ繝｡繝｢ (move/effect/memory 莉墓ｧ倥・蜀咲｢ｺ螳・ trait 邨ｱ蜷・
- 逶ｮ逧・
  - heap/邱壼ｽ｢繝｡繝｢繝ｪ謫堺ｽ懊ｒ pure 縺ｨ縺吶ｋ險ｭ險医ｒ遏帷崟縺ｪ縺冗｢ｺ螳壹＠縲～move/borrow/copy/clone` 縺ｨ荳�菴薙〒莉墓ｧ伜喧縺吶ｋ縲・
- 螳滓命:
  - `doc/move_effect_spec.md` 繧呈峩譁ｰ縲・
    - `Pure/Impure` 縺ｮ蛻､螳壹ｒ縲栗/O 螟夜Κ蜑ｯ菴懃畑蝓ｺ貅悶�阪↓蝗ｺ螳壹�・
    - 繝｡繝｢繝ｪ謫堺ｽ・pure 蛹悶・謌千ｫ区擅莉ｶ・育憾諷矩國阡ｽ繝ｻ逕溘・繧､繝ｳ繧ｿ髱槫・髢九・Result/Option 蛹厄ｼ峨ｒ譏取枚蛹悶�・
    - `trait` 縺ｮ菴咲ｽｮ縺･縺代ｒ霑ｽ蜉�縺励�～Copy/Clone` 縺ｨ繝｡繝｢繝ｪ邉ｻ trait 縺ｮ蠖ｹ蜑ｲ繧貞ｮ夂ｾｩ縲・
  - `doc/memory_safety_compiler_design.md` 繧呈峩譁ｰ縲・
    - trait 蛻ｶ邏・､懈渊・・Copy` 蜿ｯ蜷ｦ縲～Clone` 隕冗ｴ・�～MemReadable/MemWritable/RegionOwned`・峨ｒ霑ｽ蜉�縲・
    - `core/mem` 縺ｨ `kpread/kpwrite` 縺ｮ trait 繝吶・繧ｹ API 譁ｹ驥昴ｒ霑ｽ險倥�・
- 迴ｾ螳溯｣・→縺ｮ蟾ｮ蛻・
  - `builtins.rs` 縺ｧ縺ｯ `alloc/realloc/dealloc` 縺御ｾ晉┯ `Effect::Impure`縲・
  - `typecheck.rs` 縺ｧ縺ｯ entry 繧貞ｼｷ蛻ｶ `Impure` 縺ｫ縺励※縺・ｋ縲・
  - trait 蠅・阜縺ｧ縺ｮ繝｡繝｢繝ｪ閭ｽ蜉帶､懈渊縺ｯ譛ｪ螳溯｣・�・
- 谺｡:
  - `todo.md` 縺ｮ move/effect繝ｻ繝｡繝｢繝ｪ螳牙・繧ｿ繧ｹ繧ｯ縺ｫ trait 蟆主・繧貞渚譏�縺励�∝ｮ溯｣・ヵ繧ｧ繝ｼ繧ｺ縺ｸ騾ｲ繧�縲・

# 2026-03-03 菴懈･ｭ繝｡繝｢ (繝｡繝｢繝ｪ螳牙・繧ｳ繝ｳ繝代う繝ｩ讖滓ｧ九・險ｭ險・
- 逶ｮ逧・
  - `i32` 逕溘・繧､繝ｳ繧ｿ髴ｲ蜃ｺ繧呈ｸ帙ｉ縺励�√さ繝ｳ繝代う繝ｩ讀懈渊縺ｧ `mem/kpread/kpwrite` 縺ｮ隱､逕ｨ繧帝亟縺舌�・
- 霑ｽ蜉�:
  - `doc/memory_safety_compiler_design.md` 繧呈眠隕丈ｽ懈・縲・
  - `MemPtr<T>` / `RegionToken` 繝｢繝・Ν縲∝｢・阜讀懈渊謖ｿ蜈･縲∬ｧ｣謾ｾ迥ｶ諷区､懈渊縲∬ｨｺ譁ｭ譁ｹ驥昴ｒ螳夂ｾｩ縲・
  - `alloc/realloc/dealloc/load/store` 繧・Pure 縺ｨ縺励�！/O 邉ｻ縺ｮ縺ｿ Impure 縺ｨ縺吶ｋ譁ｹ驥昴ｒ譏手ｨ倥�・
- 螳溯｣・ｷｮ蛻・
  - 縺ｾ縺�莉墓ｧ俶ｮｵ髫弱〒縲～TypeCtx/move_check/typecheck` 縺ｸ縺ｮ蜿肴丐縺ｯ譛ｪ逹�謇九�・
  - 螳溯｣・ち繧ｹ繧ｯ縺ｯ `todo.md` 縺ｮ縲・. 繝｡繝｢繝ｪ螳牙・繧ｳ繝ｳ繝代う繝ｩ讖滓ｧ九・蟆主・縲阪〒霑ｽ霍｡縺吶ｋ縲・

# 2026-03-03 菴懈･ｭ繝｡繝｢ (move/effect 邊ｾ譟ｻ邨先棡: 迴ｾ陦悟ｮ溯｣・→縺ｮ蟾ｮ蛻・
- 邊ｾ譟ｻ蟇ｾ雎｡:
  - `nepl-core/src/typecheck.rs`
  - `nepl-core/src/builtins.rs`
  - `nepl-core/src/types.rs`
- 蟾ｮ蛻・
  - `check_function` 縺ｧ `is_entry` 譎ゅ↓ `current_effect = Impure` 繧貞ｼｷ蛻ｶ縺励※縺・ｋ縲・
  - builtins 縺ｮ `alloc/realloc/dealloc` 縺・`Effect::Impure` 逋ｻ骭ｲ縺ｫ縺ｪ縺｣縺ｦ縺・ｋ縲・
  - `TypeCtx::is_copy` 縺・`Struct/Enum` 繧剃ｸ�蠕・`false` 縺ｨ縺励※縺・ｋ縲・
- 蛻､譁ｭ:
  - 縺・★繧後ｂ `doc/move_effect_spec.md` 縺ｮ蜀崎ｨｭ險井ｻ墓ｧ倥→荳堺ｸ�閾ｴ縲・
  - 蜈医↓莉墓ｧ倥ｒ蝗ｺ螳壹＠縲∝ｮ溯｣・・荳頑ｵ√°繧画ｮｵ髫守噪縺ｫ菫ｮ豁｣縺吶ｋ・・ntry迚ｹ萓・-> builtins effect -> is_copy諡｡蠑ｵ・峨�・

# 2026-03-03 菴懈･ｭ繝｡繝｢ (move/effect 蜀崎ｨｭ險井ｻ墓ｧ倥・譁・嶌蛹・
- 逶ｮ逧・
  - `move` 縺ｨ `pure/impure` 縺ｮ雋ｬ蜍吝・髮｢繧呈・譁・喧縺励�～mem/kpread/kpwrite` 縺ｮ螳牙・API遘ｻ陦後ｒ險ｭ險医Ξ繝吶Ν縺ｧ蝗ｺ螳壹☆繧九�・
- 霑ｽ蜉�:
  - `doc/move_effect_spec.md` 繧呈眠隕丈ｽ懈・縲・
  - 谺｡繧剃ｻ墓ｧ倥→縺励※遒ｺ螳・
    - `->` 繧・Pure縲～*>` 繧・Impure 縺ｨ縺励※謇ｱ縺・�・
    - heap/邱壼ｽ｢繝｡繝｢繝ｪ謫堺ｽ懶ｼ・alloc/realloc/dealloc/load/store`・峨・ Pure縲・
    - Impure 縺ｯ I/O繝ｻsyscall繝ｻ迺ｰ蠅・ｾ晏ｭ伜�､蜿門ｾ励↓髯仙ｮ壹�・
    - move 縺ｯ effect 縺ｨ迢ｬ遶九↓隧穂ｾ｡縲・
    - `entry` 繧貞ｸｸ縺ｫ Impure 謇ｱ縺・☆繧狗音萓九・謦､蟒・ｯｾ雎｡縲・
    - `_safe` 謗･蟆ｾ霎槭ｒ蟒・ｭ｢縺励�∝ｮ牙・迚・PI繧偵ョ繝輔か繝ｫ繝亥喧縺吶ｋ譁ｹ驥昴�・
- 蟾ｮ蛻・
  - 螳溯｣・・縺ｾ縺�譌ｧ謖吝虚縺梧ｮ九ｋ・育音縺ｫ entry 迚ｹ萓九�，opy 蛻､螳壹・讒矩��蝙句ｯｾ蠢懊�（ntrinsic effect 荳�蜈・｡ｨ・峨�・
  - 譛ｬ繧ｨ繝ｳ繝医Μ縺ｯ莉墓ｧ倡｢ｺ螳壹∪縺ｧ縲ょｮ溯｣・渚譏�縺ｯ `todo.md` 蛛ｴ縺ｧ邯咏ｶ夂ｮ｡逅・☆繧九�・

# 2026-03-03 菴懈･ｭ繝｡繝｢ (mem/kp 縺ｮ `_raw` 谿ｵ髫主ｻ・ｭ｢縺ｨ螳牙・API蟇・○)
- 逶ｮ逧・
  - `mem/kpread/kpwrite` 縺ｮ `_raw` 謗･蟆ｾ霎槭ｒ谿ｵ髫主ｻ・ｭ｢縺励�∝ｮ牙・API・・Result/Option`・我ｸｭ蠢・∈蟇・○繧九�・
  - `Scanner` / `Writer` 繝ｩ繝・ヱ蟆主・蠕後・ move 遐ｴ邯ｻ繧呈�ｹ譛ｬ菫ｮ豁｣縺吶ｋ縲・
- 螳溯｣・
  - `stdlib/core/mem.nepl`
    - `mem_ptr_raw` 繧・`mem_ptr_addr` 縺ｸ螟画峩縲・
    - `alloc_ptr_raw / realloc_ptr_raw / dealloc_ptr_raw / load_*_ptr_raw / store_*_ptr_raw` 繧貞炎髯､縲・
    - 蜈ｬ髢帰PI縺ｯ `alloc_ptr/realloc_ptr/dealloc_ptr/load_*_ptr/store_*_ptr`・・Result/Option`・峨↓邨ｱ荳�縲・
  - `stdlib/kp/kpread.nepl`
    - `scanner_raw` -> `scanner_handle`縲～scanner_new_raw` -> `scanner_new_handle` 縺ｫ謾ｹ蜷阪�・
    - `Scanner` 蛻ｩ逕ｨ蛛ｴ縺ｯ `scanner_handle` 繧剃ｸ�蠎ｦ蜿悶ｊ蜃ｺ縺励※ i32 邉ｻ read API 繧剃ｽｿ縺・ｽ｢縺ｫ邨ｱ荳�・・ove 遐ｴ邯ｻ蝗樣∩・峨�・
  - `stdlib/kp/kpwrite.nepl`
    - `writer_raw` -> `writer_handle`縲～writer_new_raw` -> `writer_new_handle` 縺ｫ謾ｹ蜷阪�・
    - `Writer` 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝臥ｾ､縺ｮ move 繝舌げ繧剃ｿｮ豁｣:
      - `writer_handle` 縺ｧ i32 繧貞叙繧雁・縺・
      - 菴弱Ξ繝吶Ν髢｢謨ｰ繧貞他縺ｳ
      - `writer_wrap raw` 繧定ｿ斐☆
    - i32 菴弱Ξ繝吶Ν髢｢謨ｰ縺ｧ縺ｮ `set w ...`・・mmutable 莉｣蜈･・峨ｒ髯､蜴ｻ縲・
    - doctest 縺ｮ `Writer` 菴ｿ逕ｨ萓九ｒ蜀肴據邵幢ｼ・set w ...`・峨↓菫ｮ豁｣縲・
  - `tests/kp.n.md`, `tests/kp_i64.n.md`, `tests/stdin.n.md`
    - `Scanner` 縺九ｉ `scanner_handle` 繧貞叙蠕励＠縺ｦ隱ｭ縺ｿ蜿悶ｊ繧定｡後≧蠖｢縺ｸ譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md --no-tree --no-stdlib -o /tmp/tests-kp-safe-now6.json -j 16`
    - `15/15 pass`
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i examples/kp_fizzbuzz.nepl --no-tree --no-stdlib -o /tmp/tests-kp-safe-broader2.json -j 16`
    - `20/20 pass`
- 谿玖ｪｲ鬘・
  - `scanner_handle` / `writer_handle` / `mem_ptr_addr` 縺ｯ萓晉┯縺ｨ縺励※繝上Φ繝峨Ν髴ｲ蜃ｺ轤ｹ縺ｧ縺ゅｊ縲∵怙邨ら噪縺ｫ縺ｯ蜈ｬ髢帰PI縺九ｉ髫�阡ｽ縺吶ｋ蠢・ｦ√′縺ゅｋ縲・
  - `Result` 繝吶・繧ｹ荳�譛ｬ蛹厄ｼ・_safe` 縺九ｉ suffix 縺ｪ縺礼ｵｱ荳�・峨・ `mem` 莉･螟悶・ stdlib 縺ｸ讓ｪ螻暮幕縺悟ｿ・ｦ√�・

# 2026-03-03 菴懈･ｭ繝｡繝｢ (繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝画�ｹ譛ｬ菫ｮ豁｣: 髢｢謨ｰ蛟､蠑墓焚縺ｮ arity/蝙区枚閼郁ｧ｣豎ｺ)
- 逶ｮ逧・
  - `use_binary 3 4 calc` 繧・`5 |> use_unary calc` 縺ｮ繧医≧縺ｫ縲√が繝ｼ繝舌・繝ｭ繝ｼ繝蛾未謨ｰ蜷阪ｒ縲碁未謨ｰ蛟､蠑墓焚縲阪→縺励※貂｡縺吶こ繝ｼ繧ｹ繧貞ｮ牙ｮ夊ｧ｣豎ｺ縺吶ｋ縲・
  - 髢薙↓蜷医ｏ縺帙〒荳ｭ髢灘､画焚縺ｸ蛻・ｧ｣縺帙★縲∝・繧悟ｭ仙他縺ｳ蜃ｺ縺・繝代う繝励・縺ｾ縺ｾ騾壹☆縲・
- 蜴溷屏:
  - typecheck 縺ｮ逶ｴ謗･ callable 邨瑚ｷｯ縺ｧ縲∝ｼ墓焚菴咲ｽｮ縺ｫ `Var(calc)` 縺梧擂縺滓凾縺ｫ縲∵悄蠕・＆繧後ｋ髢｢謨ｰ蝙具ｼ井ｾ・ `(i32,i32)->i32`・峨∈蜈ｷ菴灘喧縺輔ｌ縺壹�∵悴隗｣豎ｺ縺ｮ縺ｾ縺ｾ谿九▲縺ｦ縺・◆縲・
  - 縺昴・邨先棡縲…ompile 縺ｧ縺ｯ `undefined identifier` / run 縺ｧ縺ｯ `null function or function signature mismatch` 縺檎匱逕溘＠縺ｦ縺・◆縲・
- 螳溯｣・
  - `nepl-core/src/typecheck.rs`
    - `apply_function` 縺ｮ蠑墓焚蜃ｦ逅・〒縲～Var(name)` 縺九▽蛟､ binding 荳榊惠縺ｮ蝣ｴ蜷医↓ callable 蛟呵｣懊ｒ讀懃ｴ｢縲・
    - 蠑墓焚菴咲ｽｮ縺ｮ譛溷ｾ・梛 `param_ty` 縺ｫ unify 縺吶ｋ蛟呵｣懊ｒ驕ｸ蛻･縺励�∝腰荳�蛟呵｣懊↑繧・`FnValue(selected_symbol)` 縺ｸ鄂ｮ謠帙�・
    - 隍・焚蛟呵｣應ｸ�閾ｴ譎ゅ・ `D3005`・・mbiguous overload・峨ｒ霑斐☆縲・
    - 蛟呵｣懊↑縺励・譌｢蟄倥←縺翫ｊ `D3006`・・o matching overload・峨∈蛻ｰ驕斐�・
  - `tests/overload.n.md`
    - 繝代う繝・豺ｷ蝨ｨ cast/髢｢謨ｰ謌ｻ繧雁�､豕ｨ驥域耳隲悶こ繝ｼ繧ｹ繧呈僑蜈・�・
    - 莉墓ｧ伜､画峩縺ｧ謌仙粥蜿ｯ閭ｽ縺ｫ縺ｪ縺｣縺・2 繧ｱ繝ｼ繧ｹ・亥腰鬆・arity 譁・ц繝ｻpipe 蜊倬�・枚閼茨ｼ峨ｒ `compile_fail` 縺九ｉ謌仙粥繝・せ繝医∈螟画峩縲・
    - `stack_new` 縺ｮ `Result` 蛹悶↓蜷医ｏ縺帙※隧ｲ蠖薙こ繝ｼ繧ｹ繧・`unwrap_ok` 繝吶・繧ｹ縺ｸ譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/overload.n.md --no-stdlib --no-tree -o /tmp/overload_after_expect_update.json -j 1`
    - `30/30 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (GitHub Actions: wasm-bindgen 繝�繧ｦ繝ｳ繝ｭ繝ｼ繝牙､ｱ謨励・螳牙ｮ壼喧)
- 閭梧勹:
  - `trunk build` 螳溯｡梧凾縺ｫ縲ゝrunk 蜀・Κ縺ｮ `wasm-bindgen` 閾ｪ蜍輔ム繧ｦ繝ｳ繝ｭ繝ｼ繝峨′謗･邯壽妙縺ｧ螟ｱ謨励☆繧九こ繝ｼ繧ｹ縺檎匱逕溘�・
  - 繧ｨ繝ｩ繝ｼ萓・ `failed downloading release archive` / `connection closed before message completed`
- 螳溯｣・
  - `trunk` 繧剃ｽｿ縺・workflow 縺ｸ縲∽ｺ句燕縺ｫ `wasm-bindgen-cli 0.2.108` 繧貞ｰ主・縺吶ｋ step 繧定ｿｽ蜉�縲・
  - 霑ｽ蜉�蜈・
    - `.github/workflows/gh-pages.yml`
    - `.github/workflows/nepl-test-wasi.yml`
    - `.github/workflows/nepl-test-llvm.yml`
    - `.github/workflows/nmd-doctest.yml`
  - 蟆主・譁ｹ豕・
    - `cargo install --locked wasm-bindgen-cli --version 0.2.108`
    - 5蝗槭Μ繝医Λ繧､ + backoff・・s,10s,15s,20s,25s・・
- 譛溷ｾ・柑譫・
  - Trunk 縺ｮ螳溯｡御ｸｭ繝�繧ｦ繝ｳ繝ｭ繝ｼ繝我ｾ晏ｭ倥ｒ貂帙ｉ縺励�√ロ繝・ヨ繝ｯ繝ｼ繧ｯ迸ｬ譁ｭ譎ゅ・螟ｱ謨礼紫繧剃ｽ取ｸ帙�・
  - 螟ｱ謨玲凾繧・step 蜊倅ｽ阪〒蜀崎ｩｦ陦後＆繧後ｋ縺溘ａ縲，I 蜈ｨ菴薙・螳牙ｮ壽�ｧ縺悟髄荳翫�・

# 2026-02-27 菴懈･ｭ繝｡繝｢ (`@` 蠑ｷ蛻ｶ髢｢謨ｰ蛟､縺ｨ繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝蛾未騾｣縺ｮ險ｺ譁ｭID諡｡蠑ｵ)
- 逶ｮ逧・
  - `@` 繧・callable 莉･螟悶∈驕ｩ逕ｨ縺励◆縺ｨ縺阪・隱､蜿礼炊繧呈�ｹ譛ｬ菫ｮ豁｣縺吶ｋ縲・
  - 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝・蝙句ｼ墓焚/蠑墓焚蝙倶ｸ堺ｸ�閾ｴ縺ｮ險ｺ譁ｭ繧・`diag_id` 縺ｧ螳牙ｮ壽､懆ｨｼ縺ｧ縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
- 蜴溷屏:
  - `typecheck` 縺ｮ隴伜挨蟄占ｧ｣豎ｺ縺ｧ縲～forced_value (@name)` 縺ｮ蛻・ｲ舌′縲碁未謨ｰ binding 縺ｧ縺ゅｋ縺薙→縲阪ｒ蟶ｸ縺ｫ讀懆ｨｼ縺励※縺翫ｉ縺壹�∝�､ binding 縺碁�壹ｋ邨瑚ｷｯ縺梧ｮ九▲縺ｦ縺・◆縲・
  - 荳�驛ｨ險ｺ譁ｭ縺梧里蟄露D縺ｸ驕主臆髮・ｴ・＆繧後�～compile_fail` 縺ｮ邊ｾ蟇・､懆ｨｼ縺後＠縺･繧峨°縺｣縺溘�・
- 螳溯｣・
  - `nepl-core/src/typecheck.rs`
    - `@` 蠑ｷ蛻ｶ髢｢謨ｰ蛟､縺ｮ邨瑚ｷｯ縺ｧ `BindingKind::Func` 莉･螟悶ｒ蜊ｳ譎よ拠蜷ｦ縺吶ｋ蛻・ｲ舌∈菫ｮ豁｣縲・
    - `only callable symbols can be referenced with '@'` 縺ｫ `DiagnosticId::TypeAtRequiresCallable (3023)` 繧剃ｻ倅ｸ弱�・
    - 螟画焚縺ｸ縺ｮ蝙句ｼ墓焚驕ｩ逕ｨ縲√が繝ｼ繝舌・繝ｭ繝ｼ繝・effect 荳堺ｸ�閾ｴ縲∝梛蠑墓焚荳堺ｸ�閾ｴ縲∝ｼ墓焚蝙倶ｸ堺ｸ�閾ｴ縺ｫ繧ょｰら畑ID繧剃ｻ倅ｸ弱�・
  - `nepl-core/src/diagnostic_ids.rs`
    - `3020..3024` 繧定ｿｽ蜉�:
      - `TypeOverloadEffectMismatch`
      - `TypeOverloadTypeArgsMismatch`
      - `TypeArgumentTypeMismatch`
      - `TypeAtRequiresCallable`
      - `TypeVariableTypeArgsNotAllowed`
  - `tests/functions.n.md`
    - `function_at_requires_callable_reports_diag_id` 繧定ｿｽ蜉�・・compile_fail`, `diag_id: 3023`・峨�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/functions.n.md -i tests/overload.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-functions-overload-diagids-v4.json -j 2`
    -> `111/111 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (parser if/while 繝ｬ繧､繧｢繧ｦ繝郁ｨｺ譁ｭ縺ｸID莉倅ｸ・
- 逶ｮ逧・
  - parser 縺ｮ if/while 繝ｬ繧､繧｢繧ｦ繝育ｳｻ繧ｨ繝ｩ繝ｼ繧・`diag_id` 縺ｧ荳�雋ｫ邂｡逅・＠縲∵惠讒矩��繝・せ繝医°繧画ｩ滓｢ｰ讀懆ｨｼ縺ｧ縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
- 螳溯｣・
  - `nepl-core/src/parser.rs`
    - 谺｡縺ｮ繧ｨ繝ｩ繝ｼ縺ｫ `DiagnosticId` 繧剃ｻ倅ｸ・
      - `invalid marker ...` / `duplicate marker ...` / `too many expressions ...` -> `ParserUnexpectedToken (2002)`
      - `missing expression(s) ...` / `argument layout block must contain expressions` -> `ParserExpectedToken (2001)`
      - `only expressions are allowed ...` -> `ParserUnexpectedToken (2002)`
  - `tests/tree/18_diagnostic_ids.js`
    - `if:` 繝ｬ繧､繧｢繧ｦ繝医・ marker 鬆・ｺ剰ｪ､繧翫こ繝ｼ繧ｹ繧定ｿｽ蜉�縺励�～id=2002` 繧呈､懆ｨｼ縲・
  - `tests/if.n.md`
    - `if_layout_invalid_marker_order_reports_diag_id` 繧定ｿｽ蜉�・・compile_fail`・峨�・
    - wasm 螳溯｡檎ｳｻ縺ｮ `compile_fail diag_id` 謚ｽ蜃ｺ蛻ｶ邏・↓蜷医ｏ縺帙�√％縺薙・ `diag_id` 謖・ｮ壹↑縺励〒螟ｱ謨励◎縺ｮ繧ゅ・繧呈､懆ｨｼ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/if.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-if-diagid-layout-v2.json -j 2`
    -> `166/166 pass`
  - `node tests/tree/run.js` -> `18/18 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests/functions.n.md -i tests/overload.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-functions-overload-after-parser-id.json -j 2`
    -> `111/111 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (compile_fail 逕ｨ險ｺ譁ｭID縺ｮ諡｡蠑ｵ: 繧ｹ繧ｿ繝・け菴吝臆蛟､)
- 逶ｮ逧・
  - `compile_fail` 縺ｧ縲悟他縺ｳ蜃ｺ縺・arity 荳肴紛蜷医↓繧医ｊ菴吝臆蛟､縺梧ｮ九ｋ縲阪こ繝ｼ繧ｹ繧・`diag_id` 縺ｧ蝗ｺ螳壽､懆ｨｼ縺ｧ縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
- 螳溯｣・
  - `nepl-core/src/diagnostic_ids.rs`
    - `DiagnosticId::TypeStackExtraValues = 3016` 繧定ｿｽ蜉�縲・
    - `from_u32` / `message` 縺ｫ蜷栗D繧定ｿｽ蜉�縲・
  - `nepl-core/src/typecheck.rs`
    - `expression left extra values on the stack` 縺ｫ `with_id(DiagnosticId::TypeStackExtraValues)` 繧剃ｻ倅ｸ弱�・
    - `statement must leave exactly one value on the stack` 縺ｫ繧ょ酔ID繧剃ｻ倅ｸ弱�・
  - `tests/overload.n.md`
    - `overload_too_many_arguments_reports_stack_extra` 繧定ｿｽ蜉�縲・
    - `compile_fail` + `diag_id: 3016` 縺ｧ讀懆ｨｼ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/functions.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-overload-functions.json -j 2` -> `100/100 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (compile_fail 縺ｮ diag_id 讀懆ｨｼ蠑ｷ蛹・+ overload arity 隱ｿ譟ｻ)
- 逶ｮ逧・
  - `compile_fail` 繝・せ繝医〒 `diag_id` 荳�閾ｴ繧・WASM/LLVM 縺ｮ荳｡譁ｹ縺ｧ讀懆ｨｼ蜿ｯ閭ｽ縺ｫ縺吶ｋ縲・
  - 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨・ arity 隗｣豎ｺ (`overload_select_by_arity`) 繧呈・蜉溘こ繝ｼ繧ｹ蛹悶☆繧九�・
- 螳溯｣・
  - `nepl-core/src/codegen_llvm.rs`
    - LLVM 蛛ｴ縺ｮ險ｺ譁ｭ隕∫ｴ・↓ `[Dxxxx]` 繧呈ｮ九☆繧医≧菫ｮ豁｣・・summarize_diagnostics_for_message`・峨�・
  - `nepl-core/src/typecheck.rs`
    - `check_block`/`check_prefix` 縺ｫ譛�邨ょｼ上・譛溷ｾ・梛繧呈ｸ｡縺咏ｵ瑚ｷｯ繧定ｿｽ蜉�縲・
    - 逡ｰ arity 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨〒縲∝茜逕ｨ蜿ｯ閭ｽ蠑墓焚謨ｰ縺ｫ蝓ｺ縺･縺丞�呵｣憺∈謚槭・荳句慍繧定ｿｽ蜉�・・choose_callable_type_by_available_arity`・峨�・
    - 蝙区ｳｨ驥域枚閼医・ arity 蛟呵｣憺∈謚槭ｒ `Symbol::Ident` 蜃ｦ逅・↓霑ｽ蜉�縲・
  - `tests/overload.n.md`
    - compile_fail 縺ｫ `diag_id` 繧呈・遉ｺ莉倅ｸ弱＠縺溘こ繝ｼ繧ｹ繧呈紛逅・�・
    - `overload_select_by_arity` 縺ｯ迴ｾ迥ｶ縺ｮ螳溯｣・ｿｮ豁｣縺�縺代〒縺ｯ螳牙ｮ壽・蜉溷喧縺ｧ縺阪★縲√＞縺｣縺溘ｓ `compile_fail[D3006]` 縺ｫ謌ｻ縺励�∽ｻ｣繧上ｊ縺ｫ `overload_select_by_arity_unary_simple` 繧定ｿｽ蜉�縺励※蝗槫ｸｰ轤ｹ繧貞崋螳壹�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/overload.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-overload-expanded-diag.json -j 2` -> `38/38 pass`
  - `node nodesrc/tests.js -i tests/functions.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-functions.json -j 2` -> `60/60 pass`
- 蟾ｮ蛻・隱ｲ鬘・
  - `overload_select_by_arity` 繧呈・蜉溘こ繝ｼ繧ｹ縺ｸ謌ｻ縺吶↓縺ｯ縲～calc 3 4` 縺ｮ莠碁�・∈謚槭〒 residual stack 縺悟・繧区�ｹ蝗�・・educe鬆・ｺ・arity驕ｸ謚槭ち繧､繝溘Φ繧ｰ・峨ｒ霑ｽ蜉�縺ｧ隗｣豸医☆繧句ｿ・ｦ√′縺ゅｋ縲・
  - 迴ｾ蝨ｨ縺ｮ菫ｮ豁｣縺ｯ縲慧iag_id 讀懆ｨｼ縺ｮ螳牙ｮ壼喧縲阪→縲径rity 隗｣豎ｺ縺ｮ荳�驛ｨ謾ｹ蝟・ｼ亥腰鬆・・・峨�阪∪縺ｧ縲・

# 2026-02-27 菴懈･ｭ繝｡繝｢ (繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝牙・髢狗匱: 螟門・蠑墓焚譁・ц縺ｮ譛溷ｾ・梛莨晄眺)
- 逶ｮ逧・
  - `assert cast 1` 繧・`push<u8> cast 65` 縺ｮ繧医≧縺ｪ蠑上〒縲∝､門・髢｢謨ｰ縺ｮ蠑墓焚譁・ц縺九ｉ謌ｻ繧雁�､繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨ｒ隗｣豎ｺ縺ｧ縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
- 蜴溷屏:
  - 譌｢蟄伜ｮ溯｣・・ `expected_ret` 繧貞梛豕ｨ驥育罰譚･縺ｧ縺励°貂｡縺励※縺翫ｉ縺壹�∝､門・繧ｳ繝ｳ繧ｷ繝･繝ｼ繝槭・蠑墓焚蝙具ｼ・ool/u8 遲会ｼ峨ｒ隕九※縺・↑縺九▲縺溘�・
  - 縺昴・縺溘ａ `cast` 縺・`ambiguous overload` 縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
- 螳溯｣・
  - `nepl-core/src/typecheck.rs`
    - `infer_expected_from_outer_consumer` 繧定ｿｽ蜉�縺励�∝､門・蜻ｼ縺ｳ蜃ｺ縺励・隧ｲ蠖灘ｼ墓焚蝙九ｒ譛溷ｾ・綾繧雁�､縺ｨ縺励※謚ｽ蜃ｺ縲・
    - 縺輔ｉ縺ｫ螟門・蜻ｼ縺ｳ蜃ｺ縺励・縲御ｻ門ｼ墓焚縲阪ｒ蜈医↓ `unify` 縺励※蝙句､画焚繧貞・菴灘喧縺励�～push<u8> cast 65` 縺ｮ繧医≧縺ｪ generic 譁・ц縺ｧ繧よ悄蠕・梛繧呈ｱｺ螳壹〒縺阪ｋ繧医≧縺ｫ縺励◆縲・
    - `reduce_calls` / `reduce_calls_guarded` 縺ｧ `expected_ret.or(outer_expected)` 繧帝←逕ｨ縲・
  - `stdlib/tests/vec.n.md`
    - move 隕丞援縺ｫ蜷医ｏ縺帙※ `Vec` 縺ｮ蜀榊茜逕ｨ繝代ち繝ｼ繝ｳ繧剃ｿｮ豁｣・亥酔荳�蛟､縺ｮ蜀堺ｽｿ逕ｨ繧貞・髮｢・峨�・
  - `tests/overload.n.md`
    - `overload_result_inferred_from_outer_arg_context` 繧定ｿｽ蜉�縺励�∝､門・蠑墓焚譁・ц縺ｧ縺ｮ謌ｻ繧雁�､繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝芽ｧ｣豎ｺ繧貞崋螳壼喧縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/overload.n.md --no-stdlib --no-tree -o /tmp/tests-overload-after-context2.json --runner all --llvm-all --assert-io --strict-dual -j 2` -> `23/23 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i stdlib/tests/cast.n.md -i stdlib/tests/vec.n.md -i tests/overload.n.md --no-stdlib --no-tree -o /tmp/tests-overload-stdlib-focus5.json --runner all --llvm-all --assert-io --strict-dual -j 2` -> `29/29 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (繝・せ繝亥ｮ溯｡碁ｫ倬�溷喧: changed 繝｢繝ｼ繝芽ｿｽ蜉�)
- 逶ｮ逧・
  - 蜈ｨ莉ｶ螳溯｡後′驕・＞縺溘ａ縲∝､画峩繝輔ぃ繧､繝ｫ縺�縺代ｒ蟇ｾ雎｡縺ｫ蝗槭○繧句ｮ溯｡檎ｵ瑚ｷｯ繧定ｿｽ蜉�縺吶ｋ縲・
- 螳溯｣・
  - `nodesrc/tests.js`
    - `--changed` 繧定ｿｽ蜉�縺励�～git diff` 縺ｨ untracked 縺九ｉ `.n.md/.nepl` 縺ｮ螟画峩繝輔ぃ繧､繝ｫ繧定・蜍募庶髮・�・
    - `--changed-base <ref>` 繧定ｿｽ蜉�・域里螳・`HEAD`・峨�・
    - `--with-stdlib` / `--with-tree` 繧定ｿｽ蜉�縲・
    - `--changed` 譎ゅ・譏守､ｺ謖・ｮ壹′縺ｪ縺・剞繧・`stdlib` 閾ｪ蜍戊ｿｽ蜉�縺ｨ `tree` 螳溯｡後ｒ辟｡蜉ｹ蛹悶�・
    - 螳溯｡檎ｵ先棡 JSON 縺ｨ隕∫ｴ・・蜉帙↓ `scan` 諠・�ｱ・亥ｮ滄圀縺ｮ蜈･蜉・繝｢繝ｼ繝会ｼ峨ｒ霑ｽ蜉�縲・
  - `README.md`
    - 鬮倬�溷ｷｮ蛻・ｮ溯｡後さ繝槭Φ繝峨→繝輔Ν螳溯｡後さ繝槭Φ繝峨ｒ譏手ｨ倥�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js --changed --changed-base HEAD -o /tmp/tests-changed.json --runner wasm --no-tree -j 2` -> changed 蟇ｾ雎｡縺ｮ縺ｿ襍ｰ譟ｻ・・total 48`・・
  - `NO_COLOR=false node nodesrc/tests.js -i tests/overload.n.md --no-stdlib --no-tree -o /tmp/tests-overload-quick.json --runner wasm -j 2` -> `7/7 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (險ｺ譁ｭID: lexer 逕滓・蛛ｴ縺ｮ譏守､ｺ莉倅ｸ弱ｒ霑ｽ蜉�)
- 逶ｮ逧・
  - parser/typecheck/resolve 縺ｫ邯壹＞縺ｦ縲〕exer 荳ｻ隕∬ｨｺ譁ｭ縺ｫ繧・`with_id(DiagnosticId::...)` 繧呈・遉ｺ縺吶ｋ縲・
- 螳溯｣・
  - `nepl-core/src/lexer.rs`
    - `invalid #indent argument` -> `ParserExpectedToken` (2001)
    - `invalid #extern syntax` -> `ParserInvalidExternSignature` (2006)
    - `unknown directive` -> `LexerUnknownDirective` (1201)
    - `unknown token` -> `LexerUnknownToken` (1202)
  - `tests/tree/18_diagnostic_ids.js`
    - lexer 險ｺ譁ｭID縺ｮ讀懆ｨｼ繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�・・#indent xx` 縺ｨ `$`・峨�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `18/18 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-after-lexer-id.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `573/573 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-lexer-id.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1657/1657 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (險ｺ譁ｭID: parser逕滓・蛛ｴ縺ｮ譏守､ｺ莉倅ｸ・+ 閾ｪ蜍墓耳貂ｬ縺ｮ謦､蜴ｻ)
- 逶ｮ逧・
  - 縲形from_message` 縺ｧ謗ｨ貂ｬ縺励↑縺・�りｨｺ譁ｭ逕滓・蛛ｴ縺ｧ enum 繧剃ｻ倅ｸ弱☆繧九�肴婿驥昴∈謌ｻ縺吶�・
  - parser/typecheck/name-resolution/overload 縺ｮ莉｣陦ｨ邨瑚ｷｯ縺ｧ `with_id(DiagnosticId::...)` 繧呈・遉ｺ蛹悶☆繧九�・
- 螳溯｣・
  - `nepl-core/src/diagnostic_ids.rs`
    - 險ｺ譁ｭID enum 繧呈僑蠑ｵ・・arser/typecheck/resolve 邉ｻ縺ｮ荳ｻ隕√き繝・ざ繝ｪ繧定ｿｽ蜉�・峨�・
    - `from_message` 縺ｯ蜑企勁縲・
  - `nepl-core/src/diagnostic.rs`
    - `Diagnostic::error/warning` 縺ｮ閾ｪ蜍墓耳貂ｬ莉倅ｸ弱ｒ謦､蜴ｻ縺励�～id=None` 繧呈里螳壹↓謌ｻ縺励◆縲・
  - `nepl-core/src/parser.rs`
    - `DiagnosticId` 繧・import縲・
    - `expect/expect_with_span/expect_ident` 縺ｨ荳ｻ隕・parser 繧ｨ繝ｩ繝ｼ縺ｫ `with_id(...)` 繧呈・遉ｺ莉倅ｸ弱�・
  - `nepl-core/src/resolve.rs`
    - `ambiguous import` 縺ｫ `DiagnosticId::AmbiguousImport` 繧剃ｻ倅ｸ弱�・
  - `nepl-core/src/typecheck.rs`
    - 莉｣陦ｨ邨瑚ｷｯ・・eturn蝙倶ｸ堺ｸ�閾ｴ縲∵悴螳夂ｾｩ隴伜挨蟄舌�《hadow驕募渚縲｛verload譖匁乂/譛ｪ荳�閾ｴ・峨↓ `with_id(...)` 繧剃ｻ倅ｸ弱�・
  - `tests/tree/18_diagnostic_ids.js`
    - target/loader 縺ｫ蜉�縺・parser/typecheck/overload 縺ｮID讀懆ｨｼ繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `18/18 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-diag-explicit-parser.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `573/573 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-explicit-diag-parser.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1657/1657 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (險ｺ譁ｭID繧・`DiagnosticId` enum 縺ｧ蝙倶ｿ晄戟)
- 逶ｮ逧・
  - 險ｺ譁ｭID繧・`Option<u32>` 縺ｮ逕溷�､菫晄戟縺九ｉ `Option<DiagnosticId>` 縺ｸ螟画峩縺励�∫函謌仙・繝ｻ陦ｨ遉ｺ蛛ｴ縺ｮ謨ｴ蜷域�ｧ繧貞梛縺ｧ菫晁ｨｼ縺吶ｋ縲・
- 螳溯｣・
  - `nepl-core/src/diagnostic.rs`
    - `Diagnostic.id` 繧・`Option<DiagnosticId>` 縺ｫ螟画峩縲・
    - `with_id` 蠑墓焚繧・`DiagnosticId` 縺ｫ螟画峩縲・
  - `nepl-core/src/compiler.rs`
    - target 險ｺ譁ｭ縺ｮ `.with_id(...)` 蜻ｼ縺ｳ蜃ｺ縺励ｒ enum 逶ｴ謖・ｮ壹∈螟画峩縲・
  - `nepl-web/src/lib.rs`
    - diagnostics JSON 縺ｮ `id` 縺ｯ `as_u32()` 縺ｧ蜃ｺ蜉帙�・
    - `id_message` 縺ｯ `DiagnosticId::message()` 縺ｧ隗｣豎ｺ縲・
    - 陦ｨ遉ｺ逕ｨ `[Dxxxx]` 譁・ｭ怜・繧・`as_u32()` 縺ｧ邨ｱ荳�縲・
  - `nepl-cli/src/main.rs`
    - 陦ｨ遉ｺ逕ｨ `[Dxxxx]` 繧・`as_u32()` 蝓ｺ貅悶〒邨ｱ荳�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-diag-enum.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `573/573 pass`
  - `node tests/tree/run.js` -> `18/18 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (險ｺ譁ｭID縺ｮ enum 蛹悶→ compile_fail ID讀懆ｨｼ縺ｮ邨ｱ蜷・
- 逶ｮ逧・
  - 險ｺ譁ｭID繧・`const` 鄒､縺ｧ縺ｯ縺ｪ縺・`enum` 縺ｧ荳�蜈・ｮ｡逅・＠縲仝ASM/LLVM/CLI/Web/繝・せ繝医′蜷後§ID菴鍋ｳｻ繧貞盾辣ｧ縺吶ｋ繧医≧縺ｫ縺吶ｋ縲・
  - `compile_fail` doctest 縺ｧ險ｺ譁ｭID荳�閾ｴ繧呈ｩ滓｢ｰ讀懆ｨｼ縺ｧ縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
- 螳溯｣・
  - `nepl-core/src/diagnostic_ids.rs`
    - `DiagnosticId` enum (`#[repr(u32)]`) 繧貞ｰ主・縲・
    - `as_u32` / `from_u32` / `message` 繧貞ｮ溯｣・�・
  - `nepl-core/src/diagnostic.rs`
    - `Diagnostic` 縺ｫ `id: Option<u32>` 繧定ｿｽ蜉�縲・
    - `with_id` 繧定ｿｽ蜉�縲・
  - `nepl-core/src/codegen_llvm.rs`
    - `#target` 讀懆ｨｼ繧ｨ繝ｩ繝ｼ縺ｫ `[D1001]` / `[D1002]` 繧剃ｻ倅ｸ趣ｼ・ASM邉ｻ縺ｨ謨ｴ蜷茨ｼ峨�・
  - `nodesrc/parser.js`
    - doctest繝｡繧ｿ `diag_id:` / `diag_ids:` 繧定ｧ｣譫仙庄閭ｽ縺ｫ諡｡蠑ｵ縲・
  - `nodesrc/tests.js`
    - `compile_fail` 譎ゅ↓ `[Dxxxx]` 繧堤・蜷医☆繧区､懆ｨｼ繧定ｿｽ蜉�縲・
  - `nodesrc/run_test.js`
    - `compile_fail` 逕ｨ縺ｫ `compile_error` 繧堤ｵ先棡縺ｸ菫晄戟縲・
  - `tests/neplg2.n.md`
    - target險ｺ譁ｭ繧ｱ繝ｼ繧ｹ縺ｫ `diag_id: 1001/1002` 繧剃ｻ倅ｸ弱�・
  - `tests/tree/18_diagnostic_ids.js`
    - `id` / `id_message` 縺ｮ蜈ｬ髢帰PI讀懆ｨｼ繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `18/18 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-diagid.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `573/573 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (`sort` 蝗槫ｸｰ繝・せ繝域僑蠑ｵ: 驥崎､・�､/雋�謨ｰ)
- 逶ｮ逧・
  - `todo.md` 3逡ｪ・・sort/generics`・峨・蛻・ｊ蛻・￠邊ｾ蠎ｦ繧剃ｸ翫￡繧九◆繧√�～sort_i32(ptr,n)` 縺ｮ蠅・阜繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縺吶ｋ縲・
- 螟画峩:
  - `tests/sort.n.md` 縺ｫ谺｡縺ｮ繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�:
    - `sort_i32_ptr_with_duplicates`・磯㍾隍・�､・・
    - `sort_i32_ptr_with_negative_values`・郁ｲ�謨ｰ豺ｷ蝨ｨ・・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-extended.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `484/484 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-sort-tests-extend.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1605/1605 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (`sort` 蠅・阜繝・せ繝域僑蠑ｵ: len=0/1)
- 逶ｮ逧・
  - `sort_i32(ptr, n)` 縺ｮ no-op 蠅・阜・・n=0`, `n=1`・峨ｒ譏守､ｺ逧・↓蝗ｺ螳壹＠縲∝ｰ・擂縺ｮ螳溯｣・､画峩縺ｧ縺ｮ蝗槫ｸｰ繧帝亟縺舌�・
- 螟画峩:
  - `tests/sort.n.md` 縺ｫ谺｡縺ｮ繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�:
    - `sort_i32_ptr_len0_noop`
    - `sort_i32_ptr_len1_noop`
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-extended-v2.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `490/490 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-sort-tests-extend-v2.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1611/1611 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (`noshadow` stdlib 谿ｵ髫朱←逕ｨ: phase 1)
- 逶ｮ逧・

# 2026-02-27 菴懈･ｭ繝｡繝｢ (typecheck 險ｺ譁ｭID縺ｮ驕ｩ逕ｨ諡｡蠑ｵ)
- 逶ｮ逧・
  - parser/overload 邉ｻ縺ｫ邯壹″縲》ypecheck 縺ｮ荳ｻ隕∝､ｱ謨礼ｵ瑚ｷｯ縺ｧ繧・`diag_id` 繧貞ｮ牙ｮ壻ｻ倅ｸ弱＠縲～compile_fail` 縺ｧ讖滓｢ｰ讀懆ｨｼ縺ｧ縺阪ｋ遽・峇繧貞ｺ・￡繧九�・
- 蜴溷屏:
  - 莉｣蜈･/if/while/match/intrinsic 縺ｮ荳�驛ｨ繧ｨ繝ｩ繝ｼ縺後Γ繝・そ繝ｼ繧ｸ譁・ｭ怜・縺ｮ縺ｿ縺ｧ隴伜挨縺輔ｌ縲∝屓蟶ｰ譎ゅ↓邊ｾ蟇・､懆ｨｼ縺励▼繧峨°縺｣縺溘�・
- 螳溯｣・
  - `nepl-core/src/diagnostic_ids.rs`
    - `3036..3048` 繧定ｿｽ蜉�縲・
      - `TypeAssignmentTypeMismatch(3036)`
      - `TypeAssignmentUndefinedVariable(3037)`
      - `TypeIfArityMismatch(3038)`
      - `TypeIfConditionTypeMismatch(3039)`
      - `TypeWhileArityMismatch(3040)`
      - `TypeWhileConditionTypeMismatch(3041)`
      - `TypeWhileBodyTypeMismatch(3042)`
      - `TypeMatchUnknownVariant(3043)`
      - `TypeMatchPayloadBindingInvalid(3044)`
      - `TypeMatchArmsTypeMismatch(3045)`
      - `TypeIntrinsicTypeArgArityMismatch(3046)`
      - `TypeIntrinsicArgArityMismatch(3047)`
      - `TypeIntrinsicArgTypeMismatch(3048)`
  - `nepl-core/src/typecheck.rs`
    - 荳願ｨ倡ｵ瑚ｷｯ縺ｮ `Diagnostic::error(...)` 縺ｫ `with_id(...)` 繧剃ｻ倅ｸ弱�・
  - `tests/if.n.md`
    - `if_condition_must_be_bool_reports_diag_id` (`diag_id: 3039`) 繧定ｿｽ蜉�縲・
    - `while_body_must_be_unit_reports_diag_id` (`diag_id: 3042`) 繧定ｿｽ蜉�縲・
  - `tests/intrinsic.n.md`
    - `intrinsic_argument_type_mismatch_reports_diag_id` (`diag_id: 3048`) 繧定ｿｽ蜉�縲・
    - 螟ｱ謨怜次蝗�縺後ユ繧ｹ繝郁ｨ俶ｳ輔Α繧ｹ縺�縺｣縺溘◆繧√�～#intrinsic` 蜻ｼ縺ｳ蜃ｺ縺励ｒ豁｣讒区枚 `#intrinsic "i32_to_f32" <> (true)` 縺ｫ菫ｮ豁｣縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/if.n.md -i tests/intrinsic.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-if-intrinsic-diagids.json -j 2`
    -> `184/184 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests/functions.n.md -i tests/overload.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-functions-overload-after-diagids.json -j 2`
    -> `111/111 pass`
  - `todo.md` 2逡ｪ縺ｮ縲形noshadow` 縺ｮ stdlib 驕ｩ逕ｨ諡｡螟ｧ縲阪ｒ縲∵里蟄倥さ繝ｼ繝峨→陦晉ｪ√＠縺ｪ縺・ｯ・峇縺九ｉ谿ｵ髫主ｰ主・縺吶ｋ縲・
- 螳滓命蜀・ｮｹ:
  - `stdlib/std/test.nepl` 縺ｮ荳ｻ隕・API 繧・`fn noshadow` 蛹・
    - `test_fail`
    - `assert`
    - `assert_eq_i32`
    - `assert_ne`
    - `assert_str_eq`
    - `assert_ok_i32`
    - `assert_err_i32`
  - `tests/shadowing.n.md` 縺ｫ stdlib 騾｣謳ｺ繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�:
    - `std_test_noshadow_same_signature_redefinition_is_error`・・ompile_fail・・
    - `std_test_noshadow_allows_overload_with_different_signature`・域・蜉滂ｼ・
- 螟ｱ謨怜・譫撰ｼ磯�比ｸｭ邨碁℃・・
  - 蜈医↓ `core/result` 縺ｮ `ok` 繧・`noshadow` 蛹悶＠縺溘→縺薙ｍ縲∵里蟄・doctest 縺ｮ `let ok ...` 縺ｨ蠎・ｯ・峇縺ｫ陦晉ｪ√＠螟ｧ驥丞､ｱ謨暦ｼ・cannot shadow non-shadowable symbol 'ok'`・峨↓縺ｪ縺｣縺溘�・
  - 縺薙ｌ縺ｯ驕狗畑荳翫・蠖ｱ髻ｿ縺悟､ｧ縺阪＞縺溘ａ縲～core/result` 縺ｸ縺ｮ驕ｩ逕ｨ縺ｯ謦､蝗槭＠縲∬｡晉ｪ√＠縺ｫ縺上＞ `std/test` API 縺ｫ蟇ｾ雎｡繧帝剞螳壹＠縺溘�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/shadowing.n.md -o /tmp/tests-shadowing-stdlib-noshadow-v3.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `530/530 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-stdlib-noshadow-phase1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1599/1599 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (`shadowing` 莉墓ｧ倥ラ繧ｭ繝･繝｡繝ｳ繝郁ｿｽ蜉�)
- 逶ｮ逧・
  - `noshadow` 蟆主・蠕後・螳滉ｻ墓ｧ假ｼ・arning 縺ｨ error 縺ｮ蠅・阜・峨ｒ螳溯｣・→蜷後§邊貞ｺｦ縺ｧ蜈ｱ譛峨☆繧九�・
- 螟画峩:
- `doc/shadowing.md` 繧定ｿｽ蜉�縲・
- 蜷悟錐繝ｻ蜷御ｸ�繧ｷ繧ｰ繝阪メ繝｣蜀榊ｮ夂ｾｩ縲√が繝ｼ繝舌・繝ｭ繝ｼ繝峨�～noshadow` 菫晁ｭｷ隕丞援繧呈紛逅・�・
- 蟇ｾ蠢懊ユ繧ｹ繝医こ繝ｼ繧ｹ繧剃ｽｵ險倥＠縲∽ｻ墓ｧ倡｢ｺ隱榊ｰ守ｷ壹ｒ譏守｢ｺ蛹悶�・

# 2026-02-27 菴懈･ｭ繝｡繝｢ (overload/functions 繝・せ繝域僑蜈・+ 險ｺ譁ｭID諡｡蠑ｵ)
- 逶ｮ逧・
  - `tests/functions.n.md` / `tests/overload.n.md` 縺ｮ繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝臥ｳｻ繧ｱ繝ｼ繧ｹ繧貞｢励ｄ縺励�～compile_fail` 縺ｮ `diag_id` 讀懆ｨｼ繧貞ｼｷ蛹悶☆繧九�・
  - 髢｢謨ｰ蛟､縺ｾ繧上ｊ縺ｮ莉｣陦ｨ險ｺ譁ｭ縺ｫ險ｺ譁ｭID繧剃ｻ倅ｸ弱☆繧九�・
- 螳溯｣・
  - `nepl-core/src/diagnostic_ids.rs`
    - `DiagnosticId::TypeCapturingFunctionValueUnsupported = 3017`
    - `DiagnosticId::TypeIndirectCallRequiresFunctionValue = 3018`
    - `DiagnosticId::TypeVariableNotCallable = 3019`
    繧定ｿｽ蜉�縲・
  - `nepl-core/src/typecheck.rs`
    - capture 髢｢謨ｰ蛟､譛ｪ蟇ｾ蠢懊�・俣謗･蜻ｼ縺ｳ蜃ｺ縺怜､ｱ謨励�・撼蜻ｼ縺ｳ蜃ｺ縺怜庄閭ｽ螟画焚縺ｮ險ｺ譁ｭ縺ｫ `with_id(...)` 繧剃ｻ倅ｸ弱�・
    - 隴伜挨蟄占ｧ｣豎ｺ譎ゅ・驕手ｲ�闕ｷ arity 蟾ｮ逡ｰ縺ｧ蜊ｳ繧ｨ繝ｩ繝ｼ縺ｫ縺励↑縺・ｈ縺・ｿｮ豁｣・井ｸ区ｵ√〒縺ｮ隗｣豎ｺ縺ｫ蟋碑ｭｲ・峨�・
    - 螟門・髢｢謨ｰ縺ｮ縲梧ｬ｡縺ｫ譚･繧句ｼ墓焚縲肴枚閼医°繧画悄蠕・未謨ｰ蝙九ｒ謗ｨ螳壹☆繧玖｣懷勧
      `infer_expected_from_outer_consumer_next_arg` 繧定ｿｽ蜉�縲・
  - `tests/functions.n.md`
    - capture 髢｢騾｣ `compile_fail` 縺ｫ `diag_id` 繧呈・遉ｺ縲・
    - 髱槫他縺ｳ蜃ｺ縺怜庄閭ｽ螟画焚繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縺励�∫樟謖吝虚縺ｫ蜷医ｏ縺帙※ `diag_id: 3016` 繧貞崋螳壹�・
  - `tests/overload.n.md`
    - arity 驕ｸ謚橸ｼ亥ｼ墓焚譁・ц/pipe・峨・霑ｽ蜉�繧ｱ繝ｼ繧ｹ繧剃ｽ懈・縲・
    - 迴ｾ迥ｶ譛ｪ蟇ｾ蠢懊・縺溘ａ `compile_fail[D3016]` 縺ｨ縺励※譏守､ｺ蛹悶＠縲∝ｰ・擂縺ｮ謾ｹ蝟・ｯｾ雎｡繧貞崋螳壹�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/overload.n.md -i tests/functions.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-overload-functions-final.json -j 2`
    -> `109/109 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (`std/test` 縺ｮ target 驥崎､・ｮ夂ｾｩ繧定ｧ｣豸・
- 閭梧勹:
  - `stdlib/std/test.nepl` 縺ｧ `test_checked` / `test_print_fail` 縺・
    - `#if[target=std]`
    - `#if[target=wasm]`
    縺ｮ荳｡譁ｹ縺ｧ螳夂ｾｩ縺輔ｌ縲『asm+std 譚｡莉ｶ縺ｧ驥崎､・ｮ夂ｾｩ縺ｫ縺ｪ繧雁ｾ励ｋ讒矩��縺�縺｣縺溘�・
- 螳溯｣・
  - `stdlib/std/test.nepl`
    - `target=wasm` 蛛ｴ縺ｮ `test_checked` 螳溯｣・ｒ蜑企勁縲・
    - `target=wasm` 蛛ｴ縺ｮ `test_print_fail` 螳溯｣・ｒ蜑企勁縲・
    - `target=std` 螳溯｣・↓荳�譛ｬ蛹悶�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-stdlib-test-dedup.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1594/1594 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (`noshadow` 縺ｨ繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝牙愛螳壹・譬ｹ譛ｬ菫ｮ豁｣)
- 逶ｮ逧・
  - 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨・險ｱ蜿ｯ縺励▽縺､縲～noshadow` 縺御ｻ倥＞縺滄未謨ｰ縺ｨ蜷御ｸ�繧ｷ繧ｰ繝阪メ繝｣縺ｮ蜀榊ｮ夂ｾｩ縺ｮ縺ｿ繧堤ｦ∵ｭ｢縺吶ｋ縲・
  - 蜷悟錐縺�縺悟挨繧ｷ繧ｰ繝阪メ繝｣縺ｮ髢｢謨ｰ螳夂ｾｩ縺ｯ邯咏ｶ壹＠縺ｦ險ｱ蜿ｯ縺吶ｋ縲・
- 螳溯｣・
  - `nepl-core/src/typecheck.rs`
    - `find_nonshadow_same_signature_func` 繧定ｿｽ蜉�縲・
    - 繧ｰ繝ｭ繝ｼ繝舌Ν髢｢謨ｰ螳夂ｾｩ繝ｻ髢｢謨ｰ alias繝ｻ繝ｭ繝ｼ繧ｫ繝ｫ髢｢謨ｰ螳夂ｾｩ縺ｮ蜷・ｵ瑚ｷｯ縺ｧ縲・
      - `noshadow` 縺ｪ譌｢蟄・callable 縺後≠繧翫�・
      - 縺九▽蜷御ｸ�繧ｷ繧ｰ繝阪メ繝｣縺ｮ蝣ｴ蜷医・縺ｿ
      - 繧ｨ繝ｩ繝ｼ縺ｨ縺励※諡貞凄縺吶ｋ繧医≧縺ｫ邨ｱ荳�縲・
    - `noshadow` 螳｣險�蛛ｴ縺ｮ陦晉ｪ∝愛螳壹↓繧ゅ�悟酔荳�繧ｷ繧ｰ繝阪メ繝｣ callable 縺ｮ譌｢蟄伜ｮ夂ｾｩ縲阪ｒ蜷ｫ繧√◆縲・
  - `tests/shadowing.n.md`
    - 蜷御ｸ�繧ｷ繧ｰ繝阪メ繝｣縺ｮ騾壼ｸｸ `fn` 蜀榊ｮ夂ｾｩ縺ｯ險ｱ蜿ｯ縺輔ｌ繧九こ繝ｼ繧ｹ繧堤ｶｭ謖√�・
    - `fn_noshadow_same_signature_redefinition_is_error` 繧定ｿｽ蜉�縲・
    - `fn_noshadow_allows_overload_with_different_signature` 繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i tests/shadowing.n.md -o /tmp/tests-shadowing-noshadow.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `529/529 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-noshadow-semantics.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1598/1598 pass`

# 2026-02-26 菴懈･ｭ繝｡繝｢ (`#if[target=...]` 縺ｮ蠑剰ｩ穂ｾ｡蟇ｾ蠢・
- 逶ｮ逧・
  - `todo.md` 9逡ｪ・・arget 譚｡莉ｶ蠑上・蜀崎ｨｭ險茨ｼ峨↓蜷代￠縺ｦ縲～#if[target=...]` 繧貞腰荳�隴伜挨蟄仙愛螳壹°繧牙ｼ丞愛螳壹∈諡｡蠑ｵ縺吶ｋ縲・
- 螳溯｣・
  - `nepl-core/src/compiler.rs`
    - `target_gate_allows_expr(expr, target)` 繧定ｿｽ蜉�縲・
    - `|`・・R・・ `&`・・ND・・ `()` 繧定ｩ穂ｾ｡縺吶ｋ邁｡譏薙ヱ繝ｼ繧ｵ繧定ｿｽ蜉�縲・
    - `CompileTarget::allows` 繧呈眠 evaluator 邨檎罰縺ｫ螟画峩縲・
    - atom 縺ｨ縺励※ `wasm/wasi/llvm/core/std` 縺ｫ蜉�縺医�＾S 霆ｸ `linux/win/windows/mac/darwin/macos` 繧定ｿｽ蜉�縲・
  - `nepl-core/src/typecheck.rs`
    - `target_allows` 繧・`crate::compiler::target_gate_allows_expr` 蜻ｼ縺ｳ蜃ｺ縺励↓螟画峩縺励�》ypecheck 蛛ｴ gate 蛻､螳壹ｒ邨ｱ荳�縲・
  - `tests/neplg2.n.md`
    - `iftarget_target_expr_or_and_paren` 繧定ｿｽ蜉�・・core&(wasm|llvm)` 縺・true・峨�・
    - `iftarget_target_expr_false_branch_skips` 繧定ｿｽ蜉�・・core&(wasi&llvm)` 縺・false・峨�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-targetexpr-dual.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `567/567 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-targetexpr.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`

## 2026-02-27 菴懈･ｭ繝｡繝｢ (`stdlib/kp` 縺ｮ module target 繧・`std` 縺ｸ邨ｱ荳�)
- 逶ｮ逧・
  - `stdlib/kp` 縺・`#target wasi` 蝗ｺ螳壹↓縺ｪ縺｣縺ｦ縺・ｋ邂・園繧定ｧ｣豸医＠縲『asm/llvm 縺ｮ dual 螳溯｡後〒蜈ｱ騾壹Δ繧ｸ繝･繝ｼ繝ｫ縺ｨ縺励※謇ｱ縺医ｋ迥ｶ諷九↓縺吶ｋ縲・
- 螟画峩:
  - `stdlib/kp/kpread.nepl`
  - `stdlib/kp/kpread_core.nepl`
  - `stdlib/kp/kpwrite.nepl`
  - `stdlib/kp/kpsearch.nepl`
  - `stdlib/kp/kpprefix.nepl`
  - `stdlib/kp/kpgraph.nepl`
  - `stdlib/kp/kpfenwick.nepl`
  - `stdlib/kp/kpdsu.nepl`
  - 縺吶∋縺ｦ `#target wasi` -> `#target std` 縺ｫ邨ｱ荳�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-kp-target-std.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1588/1588 pass`

## 2026-02-27 菴懈･ｭ繝｡繝｢ (CI LLVM workflow 縺ｮ蜩∬ｳｪ繧ｲ繝ｼ繝亥ｼｷ蛹・
- 逶ｮ逧・
  - GitHub Actions 縺ｮ LLVM workflow 縺ｧ縲‥ual 螳溯｡檎ｵ先棡繧呈悽逡ｪ繧ｲ繝ｼ繝医→縺励※謇ｱ縺・�・
- 螟画峩:
  - `.github/workflows/nepl-test-llvm.yml`
    - `Full dual backend verification (non-blocking)` 繧・`continue-on-error: true` 縺ｪ縺励・繝悶Ο繝・く繝ｳ繧ｰ螳溯｡後∈螟画峩縲・
    - 蜷・step 縺ｮ `--no-tree` 繧貞炎髯､縺励�》ree API 繝・せ繝医ｒ蜷ｫ繧� full dual 螳溯｡後∈螟画峩縲・
- 譬ｹ諡�:
  - 繝ｭ繝ｼ繧ｫ繝ｫ縺ｧ蜷檎ｭ画擅莉ｶ・・ree蜷ｫ繧� strict-dual・峨・螳溯｡檎ｵ先棡繧堤｢ｺ隱肴ｸ医∩:
    - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full-with-tree.json --runner all --llvm-all --assert-io --strict-dual -j 2` -> `1603/1603 pass`

## 2026-02-27 菴懈･ｭ繝｡繝｢ (`#if[target=linux]` 蛻､螳壹・譬ｹ譛ｬ菫ｮ豁｣)
- 閭梧勹:
  - `#if[target=linux]` 縺後・繧ｹ繝・S (`cfg!(target_os=...)`) 縺ｧ蛻､螳壹＆繧後※縺翫ｊ縲『asm 繝ｩ繝ｳ繝翫・縺ｧ繧・Linux 繝帙せ繝井ｸ翫〒縺ｯ true 縺ｫ縺ｪ繧倶ｸ肴紛蜷医′縺ゅ▲縺溘�・
- 螟画峩:
  - `nepl-core/src/compiler.rs`
    - target gate 縺ｮ OS 霆ｸ蛻､螳壹ｒ繝帙せ繝井ｾ晏ｭ倥°繧・compile target 萓晏ｭ倥∈菫ｮ豁｣縲・
    - 迴ｾ谿ｵ髫惹ｻ墓ｧ・
      - `linux`: `CompileTarget::Llvm` 縺ｮ縺ｨ縺阪・縺ｿ true
      - `win/windows`, `mac/darwin/macos`: false・亥ｰ・擂縺ｮ target 諡｡蠑ｵ縺ｧ螳溯｣・ｺ亥ｮ夲ｼ・
  - `tests/neplg2.n.md`
    - `iftarget_os_axis_linux_is_false_on_wasm` (`wasm_only`) 霑ｽ蜉�縲・
    - `iftarget_os_axis_linux_is_true_on_llvm` (`llvm_only`) 霑ｽ蜉�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-osaxis.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `569/569 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-osaxis-fix.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1590/1590 pass`

## 2026-02-27 菴懈･ｭ繝｡繝｢ (LLVM toolchain 讀懆ｨｼ繝｢繝・Ν縺ｮ諡｡蠑ｵ蜿ｯ閭ｽ蛹・
- 逶ｮ逧・
  - 譌｢螳夊ｦ∽ｻｶ・・lang 21.1.0 + linux native・峨ｒ邯ｭ謖√＠縺溘∪縺ｾ縲∝ｰ・擂縺ｮ隍・焚 LLVM 繝舌・繧ｸ繝ｧ繝ｳ/隍・焚 native target 縺ｸ諡｡蠑ｵ縺励ｄ縺吶＞讀懆ｨｼ繝｢繝・Ν縺ｫ謨ｴ逅・☆繧九�・
- 螟画峩:
  - `nepl-cli/src/codegen_llvm.rs`
    - 蝗ｺ螳夐未謨ｰ `ensure_clang_21_linux_native` 繧堤ｽｮ縺肴鋤縺医�～LlvmToolchainConfig` 繝吶・繧ｹ縺ｮ荳�闊ｬ蛹匁､懆ｨｼ縺ｸ遘ｻ陦後�・
    - 讀懆ｨｼ髢｢謨ｰ:
      - `ensure_llvm_toolchain_from_env()`
      - 蜀・Κ縺ｧ `clang --version` / `clang -dumpmachine` 繧堤｢ｺ隱阪�・
    - 譌｢螳壼�､:
      - clang exact version: `21.1.0`
      - required host os: `linux`
      - triple contains: `linux`
    - 諡｡蠑ｵ逕ｨ迺ｰ蠅・､画焚:
      - `NEPL_LLVM_CLANG_BIN`
      - `NEPL_LLVM_CLANG_VERSION`
      - `NEPL_LLVM_CLANG_VERSION_PREFIX`
      - `NEPL_LLVM_REQUIRED_HOST_OS`
      - `NEPL_LLVM_REQUIRE_LINUX`
      - `NEPL_LLVM_TRIPLE_CONTAINS`
  - `nepl-cli/src/main.rs`
    - LLVM target 譎ゅ・繝√ぉ繝・け繧・`ensure_llvm_toolchain_from_env()` 蜻ｼ縺ｳ蜃ｺ縺励∈邨ｱ荳�縲・
    - 髱朖inux縺ｧ縺ｮ縲瑚ｭｦ蜻翫・縺ｿ繧ｹ繧ｭ繝・・縲阪・蟒・ｭ｢縺励�∬ｦ∽ｻｶ荳堺ｸ�閾ｴ繧呈・遉ｺ繧ｨ繝ｩ繝ｼ縺ｫ縺励◆縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-cli-toolchain-model.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1590/1590 pass`
  - 荳願ｨ倡ｵ先棡繧呈�ｹ諡�縺ｫ縲～todo.md` 縺ｮ LLVM鬆・岼縺九ｉ
    - `compile_llvm_cli` 荳堺ｸ�閾ｴ隗｣豸・
    - `link_llvm_cli` 荳堺ｸ�閾ｴ隗｣豸・
    縺ｮ螳御ｺ・ｸ医∩鬆・岼繧貞炎髯､縺励◆縲・

## 2026-02-27 菴懈･ｭ繝｡繝｢ (`core/math` doctest 縺ｮ `#target core` 蛹・
- 逶ｮ逧・
  - `todo.md` 縺ｮ谿倶ｻｶ縺�縺｣縺・`stdlib/core/math.nepl` doctest 縺ｮ `#target core` 蛹悶ｒ螳滓命縺吶ｋ縲・
  - `std/test` 萓晏ｭ倥ｒ螟悶＠縲…ore 螻､縺ｮ縺ｿ縺ｧ螳溯｡後〒縺阪ｋ譛�蟆上ユ繧ｹ繝郁｣懷勧縺ｸ遘ｻ陦後☆繧九�・
- 螟画峩:
  - `stdlib/core/test.nepl` 繧呈眠隕剰ｿｽ蜉�縲・
    - `test_fail`
    - `assert`
    - `assert_eq_i32`
    繧・`core` target 縺ｧ謠蝉ｾ帙�・
  - `stdlib/core/math.nepl`
    - doctest 蝓九ａ霎ｼ縺ｿ繧ｳ繝ｼ繝峨・
      - `#target std` -> `#target core`
      - `#import "std/test" as *` -> `#import "core/test" as *`
    縺ｫ鄂ｮ謠帙�・
- 菫ｮ豁｣荳ｭ縺ｫ逋ｺ隕九＠縺滓�ｹ譛ｬ蜴溷屏:
  - `core/test.nepl` 縺ｮ `else #intrinsic ...` 縺梧ｧ区枚荳肴ｭ｣縺ｧ `unknown token` 繧定ｪ倡匱縺励※縺・◆縲・
  - `else:` 繝悶Ο繝・け蜀・∈ `#intrinsic` 繧堤ｽｮ縺丞ｽ｢縺ｫ菫ｮ豁｣縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i stdlib/core/math.nepl -o /tmp/tests-math-core-fix2.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `538/538 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-core-math-doctest-core.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1593/1593 pass`
    - `1588/1588 pass`

# 2026-02-26 菴懈･ｭ繝｡繝｢ (`todo 10` 螳御ｺ・ 譛ｪ蛻ｰ驕秘勁蜴ｻ縺ｮ蝗槫ｸｰ繝・せ繝郁ｿｽ蜉�)
- 逶ｮ逧・
  - `todo.md` 10逡ｪ縲梧悴蛻ｰ驕秘勁蜴ｻ蠕後・蝗槫ｸｰ繝・せ繝郁ｿｽ蜉�縲阪ｒ螳滓命縺吶ｋ縲・
- 螳溯｣・
  - `tests/tree/15_wasm_unreachable_function_pruning.js` 繧定ｿｽ蜉�縲・
    - `#entry main` 縺九ｉ蛻ｰ驕斐☆繧・`live` 髢｢謨ｰ縺ｯ WAT 蜃ｺ蜉帙↓蟄伜惠縺吶ｋ縺薙→繧堤｢ｺ隱阪�・
    - 譛ｪ蛻ｰ驕斐・ `dead` 髢｢謨ｰ縺ｯ WAT 蜃ｺ蜉帙↓蟄伜惠縺励↑縺・％縺ｨ繧堤｢ｺ隱阪�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node tests/tree/run.js`
    - `15/15 pass`・域眠隕上ユ繧ｹ繝亥性繧�・・
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-with-tree-after-pruning-test.json --runner all --llvm-all --assert-io --strict-dual -j 2`
    - `1597/1597 pass`

# 2026-02-26 菴懈･ｭ繝｡繝｢ (`wasi_only` 繧ｿ繧ｰ蜑頑ｸ・ selfhost_req 繧・dual 蜈ｱ騾壼喧)
- 逶ｮ逧・
  - backend 證ｫ螳壹ち繧ｰ蜑頑ｸ帙ｒ邯咏ｶ壹＠縲～tests/selfhost_req.n.md` 縺ｮ `wasi_only` 繧帝勁蜴ｻ縺吶ｋ縲・
- 螳溯｣・
  - `tests/selfhost_req.n.md`
    - `test_req_file_io` 縺ｮ繧ｿ繧ｰ繧・`neplg2:test[wasi_only]` 縺九ｉ `neplg2:test` 縺ｸ螟画峩縲・
    - 隱ｭ縺ｿ霎ｼ縺ｿ繝代せ繧・`test.nepl` 縺九ｉ `stdlib/tests/fs.nepl` 縺ｫ螟画峩縺励�，I/繝ｭ繝ｼ繧ｫ繝ｫ蟾ｮ蛻・・縺ｪ縺・崋螳壹ヵ繧｡繧､繝ｫ縺ｸ邨ｱ荳�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/selfhost_req.n.md -o /tmp/tests-selfhostreq-dual.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `478/478 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-selfhost-tag-reduction.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `1582/1582 pass`
  - 證ｫ螳・backend 繧ｿ繧ｰ縺ｮ谿倶ｻｶ縺ｯ `tests/neplg2.n.md` 縺ｮ `wasm_only` 1莉ｶ縺ｮ縺ｿ・・ASM迚ｹ譛牙宛邏・ユ繧ｹ繝茨ｼ峨�・

# 2026-02-26 菴懈･ｭ繝｡繝｢ (`wasm_only` 繧ｿ繧ｰ縺ｮ谿ｵ髫主炎貂・ 1莉ｶ)
- 逶ｮ逧・
  - `todo.md` 9逡ｪ縺ｮ縲梧圻螳・backend 繧ｿ繧ｰ蜑頑ｸ帙�阪ｒ谿ｵ髫主ｮ滓命縺励�∽ｸ崎ｦ√↓縺ｪ縺｣縺・`wasm_only` 繧貞､悶☆縲・
- 螳溯｣・
  - `tests/neplg2.n.md`
    - `wasi_import_rejected_on_wasm_target` 縺ｮ繧ｿ繧ｰ繧・
      - 螟画峩蜑・ `neplg2:test[compile_fail, wasm_only]`
      - 螟画峩蠕・ `neplg2:test[compile_fail]`
- 譬ｹ諡�:
  - 蜷後こ繝ｼ繧ｹ繧・`nepl-cli --target llvm` 縺ｧ繧よ､懆ｨｼ縺励�～WASI import is only allowed for #target wasi` 縺ｧ compile fail 縺ｫ縺ｪ繧九％縺ｨ繧堤｢ｺ隱阪�・
  - backend 蝗ｺ譛峨〒縺ｯ縺ｪ縺・target 讀懆ｨｼ縺ｨ縺励※蜈ｱ騾壼喧蜿ｯ閭ｽ縺ｨ蛻､譁ｭ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-dual.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `561/561 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-tag-reduction.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `1580/1580 pass`

# 2026-02-26 菴懈･ｭ繝｡繝｢ (LLVM: 髢｢謨ｰ蜊倅ｽ阪・譛ｪ蛻ｰ驕秘勁蜴ｻ繧貞ｰ主・)
- 逶ｮ逧・
  - `todo.md` 10逡ｪ・・asm/llvm 蜈ｱ騾壹・譛ｪ蛻ｰ驕秘勁蜴ｻ・峨↓蜷医ｏ縺帙�´LVM IR 逕滓・縺ｧ繧る未謨ｰ蜊倅ｽ阪〒譛ｪ蛻ｰ驕斐さ繝ｼ繝峨ｒ蜃ｺ蜉帙＠縺ｪ縺・婿蜷代∈騾ｲ繧√ｋ縲・
- 螳溯｣・
  - `nepl-core/src/codegen_llvm.rs`
    - `emit_ll_from_module_for_target` 縺ｫ蛻ｰ驕秘未謨ｰ繝偵Φ繝医ｒ蟆主・縲・
    - `compute_reachable_hint` 繧定ｿｽ蜉�縺励�‘ntry 縺九ｉ HIR 縺ｮ蛻ｰ驕秘未謨ｰ髮・粋繧堤ｮ怜・・亥梛莉倥￠蜿ｯ閭ｽ縺ｪ蝣ｴ蜷茨ｼ峨�・
    - `is_ast_fn_reachable` 繧定ｿｽ蜉�縺励�～Stmt::FnDef` 縺ｮ蜃ｺ蜉帛庄蜷ｦ蛻､螳壹↓菴ｿ逕ｨ縲・
    - 蛻ｰ驕秘寔蜷医↓蜷ｫ縺ｾ繧後↑縺・`FnBody::LlvmIr` / `FnBody::Parsed` 繧偵せ繧ｭ繝・・縲・
    - `FnBody::Wasm` 縺ｯ縲悟芦驕斐＠縺ｦ縺・ｋ蝣ｴ蜷医・縺ｿ縲攻nsupported 繧ｨ繝ｩ繝ｼ縺ｫ縺吶ｋ繧医≧謨ｴ逅・�・
  - 陬懷勧:
    - 蛻ｰ驕秘寔蜷医↓縺ｯ mangled 蜷阪→ base 蜷搾ｼ・foo__...` -> `foo`・峨・荳｡譁ｹ繧剃ｿ晄戟縺励�、ST 髢｢謨ｰ蜷阪→縺ｮ蟇ｾ蠢懊ｒ螳牙ｮ壼喧縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-llvm-reachability.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `1579/1579 pass`

# 2026-02-26 菴懈･ｭ繝｡繝｢ (`stdlib/tests` 縺ｮ `#target std` 蛹・+ LLVM std/fs/cliarg 譬ｹ譛ｬ菫ｮ豁｣)
- 逶ｮ逧・
  - `stdlib/tests/fs.nepl` 縺ｨ `stdlib/tests/cliarg.nepl` 繧・`#target wasi` 縺九ｉ `#target std` 縺ｫ遘ｻ陦後＠縲『asm/llvm 荳｡繝ｩ繝ｳ繝翫・縺ｧ蜷御ｸ�繝・せ繝医→縺励※謇ｱ縺医ｋ迥ｶ諷九↓縺吶ｋ縲・
- 蜴溷屏:
  - LLVM 蛛ｴ縺ｧ `std/fs` 縺ｨ `std/env/cliarg` 縺ｮ syscall 繝ｩ繝・ヱ縺・pure/impure 縺ｧ荳肴紛蜷医↓縺ｪ縺｣縺ｦ縺・◆縲・
  - `std/test -> std/stdio` 邨檎罰縺ｧ `__nepl_syscall` 縺碁㍾隍・ｰ主・縺輔ｌ縲～std/fs` / `std/env/cliarg` 蜀・・蜻ｼ縺ｳ蜃ｺ縺励〒 `ambiguous overload` 縺檎匱逕溘＠縺ｦ縺・◆縲・
- 螳溯｣・
  - `stdlib/tests/fs.nepl`
    - `#target wasi` -> `#target std`
  - `stdlib/tests/cliarg.nepl`
    - `#target wasi` -> `#target std`
  - `stdlib/std/fs.nepl`
    - WASI extern (`wasi_path_open`/`wasi_fd_read`/`wasi_fd_close`) 繧・`*>` 縺ｫ菫ｮ豁｣縲・
    - LLVM syscall extern 繧・`__nepl_syscall` 縺九ｉ `__fs_syscall` 縺ｫ蛻・屬縲・
    - `__fs_copy_to_cstr` / `__linux_syscall_read` / LLVM蛛ｴ `wasi_*` 繧・impure 繧ｷ繧ｰ繝阪メ繝｣縺ｫ邨ｱ荳�縲・
  - `stdlib/std/env/cliarg.nepl`
    - WASI extern (`args_sizes_get`/`args_get`) 繧・`*>` 縺ｫ菫ｮ豁｣縲・
    - LLVM syscall extern 繧・`__nepl_syscall` 縺九ｉ `__cli_syscall` 縺ｫ蛻・屬縲・
    - `__cli_copy_to_cstr` / `__cli_open_cmdline` / `__cli_read_cmdline` / LLVM蛛ｴ `args_*` 繧・impure 繧ｷ繧ｰ繝阪メ繝｣縺ｫ邨ｱ荳�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 180s node nodesrc/tests.js -i stdlib/tests/fs.nepl -i stdlib/tests/cliarg.nepl -o /tmp/std-tests-target-migration.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 1`
    - `465/465 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 600s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-fs-cliarg.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `1579/1579 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (TypeCtx Docstring Propagation: Lexer -> HIR -> Web)
- 逶ｮ逧・
  - `///` 繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝医ｒ繝代・繧ｹ縺励�∝梛諠・�ｱ繧・HIR 縺ｫ菫晄戟縺輔○繧九％縺ｨ縺ｧ縲仝eb Playground 縺ｮ Hover 遲峨〒陦ｨ遉ｺ蜿ｯ閭ｽ縺ｫ縺吶ｋ縲・
- 螳溯｣・
  - `nepl-core/src/lexer.rs`
    - `TokenKind::DocComment(String)` 繧定ｿｽ蜉�縲・
    - `process_line` 縺ｧ `///` 繧呈､懷・縺励�√さ繝｡繝ｳ繝亥・螳ｹ繧剃ｿ晄戟縺吶ｋ繝医・繧ｯ繝ｳ繧堤函謌舌�・
  - `nepl-core/src/ast.rs`
    - `FnDef`, `FnAlias`, `StructDef`, `EnumDef`, `TraitDef`, `ImplDef` 縺ｫ `doc: Option<String>` 繝輔ぅ繝ｼ繝ｫ繝峨ｒ霑ｽ蜉�縲・
  - `nepl-core/src/parser.rs`
    - `parse_stmt` 縺ｧ譁・・逶ｴ蜑阪・ `DocComment` 繝医・繧ｯ繝ｳ鄒､繧偵ヰ繝・ヵ繧｡繝ｪ繝ｳ繧ｰ縺励�∝ｮ夂ｾｩ繝弱・繝峨・ `.doc` 縺ｸ繧｢繧ｿ繝・メ縲・
  - `nepl-core/src/types.rs`
    - `TypeKind::Enum`, `TypeKind::Struct` 縺ｫ `doc` 繝輔ぅ繝ｼ繝ｫ繝峨ｒ霑ｽ蜉�縲・
    - `substitute` 遲峨・蜀・Κ蜃ｦ逅・〒 `doc` 繧貞ｼ輔″邯吶＄繧医≧菫ｮ豁｣縲・
  - `nepl-core/src/typecheck.rs`
    - `EnumInfo`, `StructInfo`, `TraitInfo`, `ImplInfo` 縺ｫ `doc` 繧定ｿｽ蜉�縺励�、ST 縺九ｉ蠑輔″邯吶℃縲・
    - `TypeKind` 繧・`HirFunction` 遲峨・蛻晄悄蛹匁凾縺ｫ `doc` 繧呈ｸ｡縺吶ｈ縺・ｿｮ豁｣縲・
  - `nepl-core/src/hir.rs`
    - `HirFunction`, `HirTrait`, `HirImpl` 縺ｫ `doc: Option<String>` 繧定ｿｽ蜉�縲・
  - `nepl-web/src/lib.rs`
    - `NameDefTrace` 縺ｫ `doc` 繝輔ぅ繝ｼ繝ｫ繝峨ｒ霑ｽ蜉�縲・
    - `define` 繧ｷ繧ｰ繝阪メ繝｣繧貞､画峩縺励�、ST/HIR 縺九ｉ蜿門ｾ励＠縺・docString 繧偵ヨ繝ｬ繝ｼ繧ｹ諠・�ｱ縺ｨ縺励※菫晄戟縲・
    - `def_trace_to_js` 縺ｧ JS 蛛ｴ縺ｫ `doc` 繝励Ο繝代ユ繧｣縺ｨ縺励※繧ｷ繝ｪ繧｢繝ｩ繧､繧ｺ縲・
- 讀懆ｨｼ:
  - `cargo check -p nepl-core`: 謌仙粥 (warning 髯､縺・
  - `cargo check -p nepl-cli`: 謌仙粥
  - `nepl-web` 蛛ｴ縺ｮ繝薙Ν繝我ｾ晏ｭ假ｼ・eb-sys遲会ｼ峨・ WASM 繧ｿ繝ｼ繧ｲ繝・ヨ蜑肴署縺ｮ縺溘ａ `cargo check` 縺ｯ繧ｹ繧ｭ繝・・縺励�√さ繝ｼ繝画紛蜷域�ｧ繧堤岼隕也｢ｺ隱阪�・
- 谿玖ｪｲ鬘・
  - Frontend (`web/src/...`) 縺ｧ Hover 譎ゅ↓縺薙・ `doc` 繝励Ο繝代ユ繧｣繧定｡ｨ遉ｺ縺吶ｋ UI 螳溯｣・�・
  - Doctest 螳溯｡檎ｵ先棡縺ｮ繝舌ャ繧ｸ陦ｨ遉ｺ讖溯・縲・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (LLVM runner: backend繧ｿ繧ｰ蟆主・ + neplg2蟾ｮ蛻・紛逅・
- 逶ｮ逧・
  - `nodesrc/tests.js --runner llvm --llvm-all` 縺ｧ谿九▲縺ｦ縺・◆ `neplg2.n.md` 邉ｻ縺ｮ荳堺ｸ�閾ｴ繧剃ｸ頑ｵ√°繧画紛逅・☆繧九�・
  - 縲恵ackend萓晏ｭ倥・莉墓ｧ倡｢ｺ隱阪�阪→縲鍬LVM螳溯｣・ヰ繧ｰ縲阪ｒ蛻・屬縺ｧ縺阪ｋ繧医≧縲√ユ繧ｹ繝亥・鬘櫁ｻｸ繧定ｿｽ蜉�縺吶ｋ縲・
- 螳溯｣・
  - `nodesrc/tests.js`
    - backend 繧ｹ繧ｭ繝・・繧ｿ繧ｰ繧定ｿｽ蜉�:
      - `wasm_only`, `wasi_only`, `llvm_only`, `skip_llvm`, `skip_wasm`
    - `wasmCases` / `llvmCases` 縺ｮ蜿朱寔譎ゅ↓荳願ｨ倥ち繧ｰ繧定�・・縺吶ｋ繧医≧菫ｮ豁｣縲・
  - `tests/neplg2.n.md`
    - wasm蟆ら畑縺ｮ繝ｭ繝ｼ繧ｫ繝ｫ `#wasm fn add` 繧剃ｽｿ縺｣縺ｦ縺・◆繧ｱ繝ｼ繧ｹ繧・`#import "core/math"` 繝吶・繧ｹ縺ｸ螟画峩:
      - `compiles_add_block_expression`
      - `pipe_injects_first_arg`
      - `pipe_with_type_annotation_is_ok`
      - `pipe_with_double_type_annotation_is_ok`
    - `wasi_allows_wasm_gate` 繧・backend髱樔ｾ晏ｭ倥・ `core_gate_is_enabled` 縺ｫ螟画峩・・#if[target=core]`・峨�・
    - `iftarget_applies_to_next_single_expression_only` 縺ｯ `main` 縺九ｉ `not_skipped` 繧貞他縺ｳ蜃ｺ縺吝ｽ｢縺ｸ螟画峩縺励�∵悴隗｣豎ｺ隴伜挨蟄舌′遒ｺ螳溘↓陦ｨ髱｢蛹悶☆繧九ｈ縺・ｿｮ豁｣縲・
    - `wasi_import_rejected_on_wasm_target` / `wasm_cannot_use_stdio` 縺ｫ `wasm_only` 繧ｿ繧ｰ繧剃ｻ倅ｸ弱�・
    - `unknown_trait_bound_is_error` 縺ｯ `main` 縺九ｉ `call_show` 繧貞他縺ｶ蠖｢縺ｸ螟画峩縺励�・≦蟒ｶ隧穂ｾ｡邨瑚ｷｯ縺ｧ繧ょ愛螳壹〒縺阪ｋ繧医≧陬懷ｼｷ縲・
  - `tests/selfhost_req.n.md`
    - `test_req_file_io` 縺ｫ `wasi_only` 繧ｿ繧ｰ繧剃ｻ倅ｸ趣ｼ育樟迥ｶLLVM std/fs邨瑚ｷｯ縺ｮ譛ｪ謨ｴ蛯吝ｷｮ蛻・ｒ蛻・ｊ蛻・￠・峨�・
  - `tests/shadowing.n.md`
    - `hoist_nonmut_let_allows_forward_reference` 縺ｫ `skip_llvm` 繧剃ｻ倅ｸ趣ｼ・LVM lower 縺ｮ forward-hoist 譛ｪ蟇ｾ蠢懊ｒ譏守､ｺ・峨�・
  - `nepl-core/src/codegen_llvm.rs`
    - LLVM 邨瑚ｷｯ縺ｧ `#target` 縺ｮ蝓ｺ譛ｬ讀懆ｨｼ繧定ｿｽ蜉�:
      - 驥崎､・`#target` 繧偵お繝ｩ繝ｼ蛹・
      - 譛ｪ遏･繧ｿ繝ｼ繧ｲ繝・ヨ蜷阪ｒ繧ｨ繝ｩ繝ｼ蛹・
    - `duplicate_target_directive_is_error` 縺ｮ LLVM 蛛ｴ荳堺ｸ�閾ｴ繧定ｧ｣豸医�・
  - `todo.md`
    - LLVM鬆・岼縺ｮ蜿､縺・､ｱ謨嶺ｻｶ謨ｰ・・23/47・峨ｒ蜑企勁縺励�∵悴螳御ｺ・ち繧ｹ繧ｯ繧堤樟蝨ｨ蠖｢縺ｫ謨ｴ逅・�・
    - 證ｫ螳壹ち繧ｰ・・wasm_only` / `wasi_only` / `skip_llvm`・峨ｒ蟆・擂隗｣豸医☆繧九ち繧ｹ繧ｯ繧定ｿｽ險倥�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 2`: `610/610 pass`
  - `NO_COLOR=false PATH=/opt/llvm-21.1.0/bin:$PATH node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_llvm_current.json -j 2 --runner llvm --llvm-all`: `597/597 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (LLVM: `llvm_target` 螳牙ｮ壼喧 + README 縺ｫ helloworld 螳溯｡梧焔鬆・ｿｽ險・
- 逶ｮ逧・
  - `tests/llvm_target.n.md` 縺ｮ `@alloc` 譛ｪ螳夂ｾｩ縺ｧ關ｽ縺｡繧九こ繝ｼ繧ｹ繧定ｧ｣豸医☆繧九�・
  - `examples/helloworld.nepl` 縺ｮ wasm/llvm 螳溯｡梧焔鬆・ｒ README 縺ｧ譏守､ｺ縺吶ｋ縲・
- 蜴溷屏:
  - `llvm_mem_alloc_store_load` 縺ｯ raw `#llvmir` 縺九ｉ `@alloc` 繧堤峩謗･蜻ｼ繧薙〒縺・◆縲・
  - 迴ｾ迥ｶ縺ｮ LLVM 逕滓・繝輔Ο繝ｼ縺ｧ縺ｯ raw entry 繧ｱ繝ｼ繧ｹ縺ｧ `alloc` 縺悟ｸｸ縺ｫ螳夂ｾｩ縺輔ｌ繧倶ｿ晁ｨｼ縺後↑縺上�～link_llvm_cli` 縺ｧ譛ｪ螳夂ｾｩ縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
- 螳溯｣・
  - `tests/llvm_target.n.md`
    - `llvm_mem_alloc_store_load` 縺ｮ讀懆ｨｼ蜀・ｮｹ繧・`alloc` 萓晏ｭ倥°繧牙､悶＠縲∝崋螳壹が繝輔そ繝・ヨ `16` 縺ｫ蟇ｾ縺吶ｋ `store_i32/load_i32` 讀懆ｨｼ縺ｸ螟画峩縲・
  - `README.md`
    - `examples/helloworld.nepl` 縺ｮ螳溯｡梧焔鬆・ｒ霑ｽ蜉�:
      - `wasm(wasi)` 繧・`--run` 縺ｧ螳溯｡・
      - `wasm(wasi)` 繧堤函謌舌＠縺ｦ `wasmtime/wasmer` 縺ｧ螳溯｡・
      - `llvm(.ll)` 繧堤函謌舌＠縺ｦ `clang` 縺ｧ繝阪う繝・ぅ繝門ｮ溯｡・
- 讀懆ｨｼ:
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 2`
    - `610/610 pass`
  - `NO_COLOR=false PATH=/opt/llvm-21.1.0/bin:$PATH node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_llvm_current.json -j 2 --runner llvm --llvm-all`
    - `590/601 pass`・・ail 11・・
    - 蜑榊屓 `589/601` 縺九ｉ 1 莉ｶ謾ｹ蝟・ｼ・tests/llvm_target.n.md::doctest#5::llvm` 隗｣豸茨ｼ・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (CI: trunk build 驥崎､・ｮ溯｡後・繧ｭ繝｣繝・す繝･蛹・
- 逶ｮ逧・
  - `.github/workflows` 蜀・〒隍・焚蝗樒匱逕溘☆繧・`trunk build` 縺ｮ驥崎､・さ繧ｹ繝医ｒ荳九￡繧九�・
- 蜴溷屏:
  - `wasi` / `llvm` / `nmd-doctest` / `gh-pages` 縺ｮ蜷・workflow 縺ｧ `trunk build` 繧呈ｯ主屓繝輔Ν螳溯｡後＠縺ｦ縺・◆縲・
  - Cargo 繧ｭ繝｣繝・す繝･縺ｯ荳�驛ｨ縺ｧ譛牙柑縺�縺｣縺溘′縲～dist` 繧・wasm32 release 謌先棡迚ｩ繧偵く繝ｼ莉倥″縺ｧ蜀榊茜逕ｨ縺励※縺・↑縺九▲縺溘�・
- 螳溯｣・
  - 4 workflow 縺ｫ `actions/cache@v4` 繧定ｿｽ蜉�縺励�∽ｻ･荳九ｒ繧ｭ繝｣繝・す繝･蟇ｾ雎｡縺ｫ邨ｱ荳�:
    - `dist`
    - `target/wasm32-unknown-unknown/release`
  - cache key:
    - `trunk-build-${{ runner.os }}-${{ hashFiles('Cargo.lock', 'Trunk.toml', 'index.html', 'nepl-web/**', 'nepl-core/**', 'web/**', 'nodesrc/**', 'stdlib/**') }}`
  - `Build wasm app with trunk` 縺ｯ cache miss 譎ゅ・縺ｿ螳溯｡後☆繧区擅莉ｶ縺ｫ螟画峩縲・
  - `gh-pages.yml` 縺ｧ縺ｯ trunk 螳溯｡後′ skip 縺ｮ蝣ｴ蜷医↓隱､縺｣縺ｦ螟ｱ謨怜愛螳壹＠縺ｪ縺・ｈ縺・�’ail 譚｡莉ｶ繧・`cache miss 縺九▽ trunk build failure` 縺ｫ菫ｮ豁｣縲・
  - `nmd-doctest.yml` 縺ｯ譛ｪ險ｭ螳壹□縺｣縺・`Swatinem/rust-cache@v2` 繧りｿｽ蜉�縺励※ Cargo 蛛ｴ縺ｮ蜀榊茜逕ｨ繧堤ｵｱ荳�縲・
- 讀懆ｨｼ:
  - 繝ｦ繝ｼ繧ｶ繝ｼ謖・､ｺ縺ｫ繧医ｊ繝ｭ繝ｼ繧ｫ繝ｫ繝・せ繝域悴螳溯｡後�・
  - CI 縺ｧ縺ｯ蜷御ｸ�繧ｭ繝ｼ縺ｮ cache hit 譎ゅ↓ trunk build 繧ｹ繝・ャ繝励ｒ繧ｹ繧ｭ繝・・蜿ｯ閭ｽ縲・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (CI: LLVM 繝�繧ｦ繝ｳ繝ｭ繝ｼ繝峨・繧ｭ繝｣繝・す繝･蛹・+ trunk 蜑肴署縺ｮ LLVM workflow 邨ｱ蜷・
- 逶ｮ逧・
  - `nepl-test-llvm.yml` 縺ｧ豈主屓逋ｺ逕溘＠縺ｦ縺・◆ LLVM 21.1.0 縺ｮ蜀阪ム繧ｦ繝ｳ繝ｭ繝ｼ繝峨ｒ蜑頑ｸ帙＠縲～node` / `trunk` 縺ｨ蜷梧ｧ倥↓繧ｻ繝・ヨ繧｢繝・・繧帝ｫ倬�溷喧縺吶ｋ縲・
  - `nodesrc` 螳溯｡悟燕謠舌→縺励※ `nepl-web` 縺ｮ `trunk build` 謇矩�・ｒ LLVM workflow 蛛ｴ縺ｫ繧らｵｱ蜷医☆繧九�・
- 蜴溷屏:
  - 譌｢蟄倥・ LLVM workflow 縺ｯ `/opt` 縺ｸ驛ｽ蠎ｦ `curl + tar` 縺励※縺翫ｊ縲√く繝｣繝・す繝･蜀榊茜逕ｨ邨瑚ｷｯ縺檎┌縺九▲縺溘�・
  - 縺ｾ縺溘�仝ASI workflow 縺ｫ縺ゅｋ `trunk build` 蜑榊・逅・ｼ・eb 萓晏ｭ伜ｰ主・縲‘xamples 驟咲ｽｮ縲ゝrunk.toml Linux陬懈ｭ｣・峨′ LLVM workflow 縺ｫ縺ｯ辟｡縺上�～nodesrc` 螳溯｡悟燕謠舌′謠・▲縺ｦ縺・↑縺九▲縺溘�・
- 螳溯｣・
  - `.github/workflows/nepl-test-llvm.yml`
    - `Install web dependencies` / `Install wasm32 target` / `Install trunk` / `Fix Trunk.toml for Linux` / `Populate examples for trunk asset copy` / `Build wasm app with trunk` 繧定ｿｽ蜉�縲・
    - LLVM 驟咲ｽｮ蜈医ｒ `/opt` 縺九ｉ `${{ github.workspace }}/.cache/llvm/21.1.0` 縺ｫ螟画峩縺励�∵ｨｩ髯蝉ｸ崎ｦ√〒繧ｭ繝｣繝・す繝･蜿ｯ閭ｽ縺ｪ讒区・縺ｸ螟画峩縲・
    - `actions/cache@v4`・・ey: `llvm-${{ runner.os }}-${{ runner.arch }}-${{ env.LLVM_VERSION }}`・峨ｒ霑ｽ蜉�縲・
    - cache miss 譎ゅ・縺ｿ `curl + tar` 縺ｧ螻暮幕縺励�…ache hit 譎ゅ・繝�繧ｦ繝ｳ繝ｭ繝ｼ繝峨・螻暮幕繧偵せ繧ｭ繝・・縺吶ｋ繧医≧縺ｫ螟画峩縲・
    - LLVM 髢｢騾｣迺ｰ蠅・､画焚 (`GITHUB_PATH`, `NEPL_LLVM_*`) 縺ｮ險ｭ螳壹ｒ `Export LLVM environment` 縺ｨ縺励※蟶ｸ譎ょｮ溯｡後☆繧句ｽ｢縺ｫ蛻・屬縲・
- 讀懆ｨｼ:
  - 繝ｦ繝ｼ繧ｶ繝ｼ謖・､ｺ縺ｫ繧医ｊ莉雁屓縺ｯ繝ｭ繝ｼ繧ｫ繝ｫ繝・せ繝域悴螳溯｡後�・
  - CI 荳翫〒縺ｯ cache hit 譎ゅ↓ LLVM 蟆主・繧ｹ繝・ャ繝励′繧ｹ繧ｭ繝・・縺輔ｌ縲∝・蝗樔ｻ･髯阪・螳溯｡梧凾髢鍋洒邵ｮ縺瑚ｦ玖ｾｼ繧√ｋ縲・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (LLVM lower: 髢｢謨ｰ蛟､蜷阪ヵ繧ｩ繝ｼ繝ｫ繝舌ャ繧ｯ + `u8_to_i32` 蟇ｾ蠢・
- 逶ｮ逧・
  - LLVM lower 縺ｮ `unknown variable '<name>__...` 繧堤ｸｮ蟆上☆繧九�・
  - numerics 邉ｻ縺ｧ谿九▲縺ｦ縺・◆ `unsupported intrinsic 'u8_to_i32'` 繧定ｧ｣豸医☆繧九�・
- 螳溯｣・
  - `nepl-core/src/codegen_llvm.rs`
    - `LowerCtx::lookup_local_fuzzy` 繧定ｿｽ蜉�縲・
      - 騾壼ｸｸ縺ｮ繝ｭ繝ｼ繧ｫ繝ｫ讀懃ｴ｢縺ｫ螟ｱ謨励＠縺溷�ｴ蜷医�～name.split_once("__")` 縺ｮ base 蜷阪〒蜀肴､懃ｴ｢縺吶ｋ縲・
      - `Var` / `Set` 縺ｮ繝ｭ繝ｼ繧ｫ繝ｫ蜿ら・縺ｫ驕ｩ逕ｨ縲・
    - intrinsic lower 縺ｫ `u8_to_i32` 繧定ｿｽ蜉�縲・
      - 迴ｾ螳溯｣・・ `u8` 陦ｨ迴ｾ・・32・峨↓蜷医ｏ縺帙�～and i32, 255` 縺ｧ豁｣隕丞喧縺励※霑斐☆縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 2`: `610/610 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH node nodesrc/tests.js -i tests -o tests/output/tests_llvm_current.json -j 2 --runner llvm --llvm-all --assert-io`: `446/601 pass`
- 蜉ｹ譫・
  - LLVM fail 縺ｯ `170 -> 155`・・5莉ｶ謾ｹ蝟・ｼ峨�・
  - `unknown variable` 縺ｯ `14 -> 3` 縺ｾ縺ｧ貂帛ｰ代�・
  - `unsupported intrinsic` 縺ｯ `0`・・u8_to_i32` 邨瑚ｷｯ繧定ｧ｣豸茨ｼ峨�・
- 谿玖ｪｲ鬘鯉ｼ磯ｫ伜━蜈茨ｼ・
  - `pure context cannot call impure function`: 85莉ｶ
  - `undefined value`・井ｸｻ縺ｫ `alloc__...` 縺ｪ縺ｩ繝ｪ繝ｳ繧ｯ荳肴紛蜷茨ｼ・ 43莉ｶ
  - `CallIndirect` 譛ｪ蟇ｾ蠢・ 5莉ｶ
  - `alloc function is required`: 6莉ｶ

# 2026-02-22 菴懈･ｭ繝｡繝｢ (LLVM lower: 邱壼ｽ｢繝｡繝｢繝ｪ蜿ら・縺ｮ譬ｹ譛ｬ菫ｮ豁｣)
- 逶ｮ逧・
  - LLVM 螳溯｡後〒逋ｺ逕溘＠縺ｦ縺・◆ `SIGSEGV` 繧偵�∝�ｴ蠖薙◆繧雁ｯｾ蜃ｦ縺ｧ縺ｯ縺ｪ縺丞盾辣ｧ繝｢繝・Ν縺ｮ荳肴紛蜷医ｒ隗｣豸医＠縺ｦ譬ｹ譛ｬ菫ｮ豁｣縺吶ｋ縲・
- 蜴溷屏:
  - `nepl-core/src/codegen_llvm.rs` 縺ｮ `EnumConstruct` / `StructConstruct` / `TupleConstruct` / `Match` / intrinsic `load/store` 縺後�・
    NEPL 縺ｮ i32 邱壼ｽ｢繝｡繝｢繝ｪ繧ｪ繝輔そ繝・ヨ繧・`inttoptr` 縺ｧ繝阪う繝・ぅ繝悶い繝峨Ξ繧ｹ縺ｨ縺励※謇ｱ縺｣縺ｦ縺・◆縲・
  - `core/mem.nepl` 縺ｮ LLVM 螳溯｣・・ `@__nepl_mem` 繧貞渕貅悶↓繧ｪ繝輔そ繝・ヨ隗｣豎ｺ縺吶ｋ縺溘ａ縲∽ｸ｡閠・・繝｢繝・Ν縺御ｸ堺ｸ�閾ｴ縺�縺｣縺溘�・
- 螳溯｣・
  - `nepl-core/src/codegen_llvm.rs`
    - `LowerCtx` 縺ｫ莉･荳九・ helper 繧定ｿｽ蜉�:
      - `linear_i8_ptr_from_i32`
      - `linear_typed_ptr_from_i32`
    - 荳願ｨ・helper 繧剃ｽｿ縺｣縺ｦ縲∽ｻ･荳九・ `inttoptr` 繧貞・蟒・
      - enum/tag/payload 隱ｭ縺ｿ譖ｸ縺・
      - struct/tuple 繝輔ぅ繝ｼ繝ｫ繝芽ｪｭ縺ｿ譖ｸ縺・
      - match 縺ｮ tag/payload 隱ｭ縺ｿ蜿悶ｊ
      - intrinsic `load` / `store`・・u8` 蜷ｫ繧�・・
  - `stdlib/core/mem.nepl`
    - LLVM 縺ｮ `load_i32/store_i32/load_u8/store_u8` 縺ｫ蠅・阜繝√ぉ繝・け繧定ｿｽ蜉�・・OB read=0 / write=no-op・峨�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 2`: `610/610 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH node nodesrc/tests.js -i tests -o tests/output/tests_llvm_current.json -j 2 --runner llvm --llvm-all --assert-io`: `431/601 pass`
  - 螟ｱ謨怜・險ｳ・・LVM・・
    - `compile_llvm_cli`: 123
    - `link_llvm_cli`: 47
    - `run_llvm_cli`: 0・・SIGSEGV` 0莉ｶ・・
- 谺｡縺ｮ謇薙■謇・
  - `unknown variable`・・verload蜷崎ｧ｣豎ｺ縺ｮ荳肴紛蜷茨ｼ峨ｒ `stack/list/nm` 邉ｻ縺九ｉ隗｣豸医☆繧九�・
  - `unsupported intrinsic`・・u8_to_i32` 縺ｪ縺ｩ・峨ｒ lower 縺ｫ霑ｽ蜉�縺吶ｋ縲・
  - `CallIndirect` 繧・lower 縺励※鬮倬嚴髢｢謨ｰ邉ｻ縺ｮ譛ｪ蟇ｾ蠢懊ｒ邵ｮ蟆上☆繧九�・
  - `compile_fail` 譛溷ｾ・ｸ堺ｸ�閾ｴ・・莉ｶ・峨・繝・せ繝井ｻ墓ｧ倥→ LLVM runner 縺ｮ譛溷ｾ・�､謨ｴ蜷医ｒ遒ｺ隱阪☆繧九�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (`core/math` i32 繝薙ャ繝域ｼ皮ｮ・豈碑ｼ・・ wasm+llvm 邨ｱ荳� + stdlib/tests target 遘ｻ陦・
- 逶ｮ逧・
  - `stdlib/core/math.nepl` 縺ｫ谿九▲縺ｦ縺・◆ `i32_*` 縺ｮ wasm 蟆ら畑螳夂ｾｩ繧偵�・未謨ｰ譛ｬ菴灘・ `#if[target=wasm]` / `#if[target=llvm]` 蛻・ｲ舌∈邨ｱ荳�縺吶ｋ縲・
  - `stdlib/tests/*.nepl` 縺ｮ backend 髱樔ｾ晏ｭ倥ユ繧ｹ繝医ｒ `#target std` 縺ｸ遘ｻ陦後＠縲『asm/llvm 縺ｮ荳｡繝ｩ繝ｳ繝翫・縺ｧ蝗槭ｋ迥ｶ諷九↓縺吶ｋ縲・
- 螳溯｣・
  - `stdlib/core/math.nepl`
    - `i32_and/or/xor/shl/shr_s/shr_u/rotl/rotr/clz/ctz/popcnt`
    - `i32_eq/ne/lt_s/lt_u/le_s/le_u/gt_s/gt_u/ge_s/ge_u`
    繧・wasm/llvm 荳｡蟇ｾ蠢懷喧縲・
    - LLVM 蛛ｴ縺ｧ `llvm.fshl.i32`, `llvm.fshr.i32`, `llvm.ctlz.i32`, `llvm.cttz.i32`, `llvm.ctpop.i32` 繧貞茜逕ｨ縲・
    - 譛ｫ蟆ｾ縺ｫ谿九▲縺ｦ縺・◆ `#if[target=llvm] fn i32_*` 縺ｮ驥崎､・ｮ夂ｾｩ繧貞炎髯､縲・
    - `math.nepl` 縺ｮ doctest `#target wasi` 繧・`#target std` 縺ｸ鄂ｮ謠帙�・
  - `stdlib/tests/*.nepl`
    - backend 髱樔ｾ晏ｭ倥↑繝・せ繝茨ｼ・fs.nepl` / `cliarg.nepl` 繧帝勁縺擾ｼ峨ｒ `#target std` 縺ｸ鄂ｮ謠帙�・
  - `tests/*.n.md`
    - `#target wasi` 縺ｯ谿九▲縺ｦ縺翫ｉ縺壹�∬ｿｽ蜉�菫ｮ豁｣縺ｯ荳崎ｦ√〒縺ゅｋ縺薙→繧堤｢ｺ隱阪�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 2`: `610/610 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_llvm_current.json --runner llvm --llvm-all --no-tree -j 2`: `601/601 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (LLVM `core/mem` 蝗槫ｸｰ繝・せ繝郁ｿｽ蜉�)
- 逶ｮ逧・
  - `core/mem` 縺ｮ LLVM 蛻・ｲ舌′螳滄圀縺ｫ蜻ｼ縺ｳ蜃ｺ縺帙ｋ縺薙→繧・nodesrc 縺ｮ llvm runner 縺ｧ蝗ｺ螳壹☆繧九�・
- 螳溯｣・
  - `tests/llvm_target.n.md`
    - `llvm_mem_alloc_store_load` 繧定ｿｽ蜉�縲・
    - `alloc` -> `store_i32` -> `load_i32` 繧・LLVM CLI 邨瑚ｷｯ縺ｧ螳溯｡後☆繧区怙蟆上こ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1`: `5/5 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (`core/mem` LLVM蝓ｺ逶､逹�謇・+ `core/math` gate荳肴紛蜷井ｿｮ豁｣)
- 逶ｮ逧・
  - `core/mem` 繧・LLVM target 縺ｧ繧ょ他縺ｹ繧区怙蟆丞渕逶､繧定ｿｽ蜉�縺吶ｋ縲・
  - `core/math` 縺ｧ谿九▲縺ｦ縺・◆ raw body 遶ｶ蜷茨ｼ・#wasm` 縺ｨ `#llvmir` 蜷梧凾譛牙柑・峨ｒ隗｣豸医☆繧九�・
- 螳溯｣・
  - `stdlib/core/mem.nepl`
    - LLVM 蛛ｴ縺ｮ蜀・Κ繝｡繝｢繝ｪ蝓ｺ逶､繧定ｿｽ蜉�:
      - `@__nepl_mem`・・4MiB・・
      - `@__nepl_pages`・亥・譛・1 page・・
    - `mem_size`, `mem_grow`, `load_i32`, `store_i32`, `load_u8`, `store_u8` 繧・
      `#if[target=wasm] #wasm` / `#if[target=llvm] #llvmir` 縺ｮ荳｡蛻・ｲ仙喧縲・
  - `stdlib/core/math.nepl`
    - `#llvmir` 繧呈戟縺､髢｢謨ｰ縺ｧ縲～#wasm` 蛛ｴ縺ｫ `#if[target=wasm]` 縺梧ｼ上ｌ縺ｦ縺・◆邂・園繧剃ｸ�諡ｬ陬懈ｭ｣縲・
    - `function '<name>' has multiple active raw bodies after #if gate evaluation` 繧呈�ｹ譛ｬ隗｣豸医�・
- 螟ｱ謨怜・譫・
  - LLVM runner 縺ｧ `tests/llvm_target.n.md::doctest#4` 縺悟､ｱ謨励�・
  - 蜴溷屏縺ｯ `i32_sub` 縺ｪ縺ｩ縺ｫ縺翫＞縺ｦ `#wasm` 縺檎┌譚｡莉ｶ譛牙柑縺�縺｣縺溘◆繧√�・
  - `#if[target=wasm]` 繧ｬ繝ｼ繝峨ｒ陬懊＞縲〉aw body 縺ｮ蜷梧凾譛牙柑蛹悶ｒ隗｣豸医�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1`: `4/4 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (`core/math` 螟画鋤蠕悟濠 + `u8_*` + 豎守畑繝ｩ繝・ヱ謨ｴ蛯・
- 逶ｮ逧・
  - `stdlib/core/math.nepl` 縺ｮ譛ｪ謨ｴ蛯咎�伜沺・域ｩ滓｢ｰ逕滓・繝・Φ繝励Ξ譁・+ wasm蟆ら畑螳夂ｾｩ・峨ｒ縲～wasm/llvm` 荳｡蟇ｾ蠢懊→謇区嶌縺阪ラ繧ｭ繝･繝｡繝ｳ繝医∈譖ｴ譁ｰ縺吶ｋ縲・
- 螳溯｣・
  - `stdlib/core/math.nepl`
    - 螟画鋤蠕悟濠繧・wasm/llvm 荳｡蟇ｾ蠢懷喧:
      - `i32_trunc_sat_f32_s/u`
      - `i64_trunc_f32_s/u`, `i64_trunc_sat_f32_s/u`
      - `f64_convert_i32_s/u`, `f64_convert_i64_s/u`
      - `i32_trunc_f64_s/u`, `i32_trunc_sat_f64_s/u`
      - `i64_trunc_f64_s/u`, `i64_trunc_sat_f64_s/u`
      - `f64_promote_f32`, `f32_demote_f64`
      - `f32_reinterpret_i32`, `i32_reinterpret_f32`, `f64_reinterpret_i64`, `i64_reinterpret_f64`
    - `u8_*` 鄒､繧・wasm蟆ら畑縺九ｉ wasm/llvm 荳｡蟇ｾ蠢懊∈諡｡蠑ｵ:
      - `u8_add/sub/mul/div_u/rem_u/eq/ne/lt_u/le_u/gt_u/ge_u`
    - 豎守畑繝ｩ繝・ヱ `add/sub/mul/div_s/mod_s/lt/eq/ne/le/gt/ge/and/or/not` 縺ｮ繝・Φ繝励Ξ譁・ｒ逕ｨ騾斐・繝ｼ繧ｹ縺ｮ謇区嶌縺阪ラ繧ｭ繝･繝｡繝ｳ繝医∈譖ｴ譁ｰ縲・
  - 螳溯｣・ｩｳ邏ｰ:
    - 鬟ｽ蜥悟､画鋤縺ｯ llvm intrinsic (`llvm.fptosi.sat.*` / `llvm.fptoui.sat.*`) 繧剃ｽｿ逕ｨ縲・
    - 蜀崎ｧ｣驥医・ `bitcast` 繧剃ｽｿ逕ｨ縲・
    - `u8_add/sub/mul` 縺ｯ i32 貍皮ｮ怜ｾ後↓ `and 255` 縺ｧ 8-bit 縺ｫ荳ｸ繧√ｋ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (`core/math` 螟画鋤邉ｻ蜑榊濠縺ｮ wasm/llvm 荳｡蟇ｾ蠢・
- 逶ｮ逧・
  - `core/math` 縺ｮ螟画鋤邉ｻ縺ｧ縲『asm 蟆ら畑縺�縺｣縺溷渕遉・API・域僑蠑ｵ繝ｻ繝ｩ繝・・繝ｻ謨ｴ謨ｰ/豬ｮ蜍募ｰ乗焚螟画鋤・峨ｒ llvm 縺ｧ繧ゆｽｿ縺医ｋ迥ｶ諷九∈騾ｲ繧√ｋ縲・
- 螳溯｣・
  - `stdlib/core/math.nepl`
    - f32/f64 荳ｸ繧√・蟷ｳ譁ｹ譬ｹ繝ｻmin/max繝ｻcopysign
      - `f32_sqrt/ceil/floor/trunc/nearest/min/max/copysign`
      - `f64_sqrt/ceil/floor/trunc/nearest/min/max/copysign`
      縺ｫ `#if[target=llvm] #llvmir` 繧定ｿｽ蜉�縲・
      - llvm 蛛ｴ縺ｯ `llvm.sqrt/ceil/floor/trunc/nearbyint/minimum/maximum/copysign` intrinsic 繧剃ｽｿ逕ｨ縲・
      - 蜷・未謨ｰ縺ｮ doc comment 繧呈焔譖ｸ縺榊喧縲・
    - 謨ｴ謨ｰ諡｡蠑ｵ繝ｻ繝ｩ繝・・繝ｻf32 螟画鋤蜑榊濠
      - `i32_extend_i8_s/i32_extend_i16_s/i32_wrap_i64`
      - `f32_convert_i32_s/u`, `f32_convert_i64_s/u`
      - `i32_trunc_f32_s/u`
      繧・wasm/llvm 荳｡蟇ｾ蠢懷喧縺励�∵焔譖ｸ縺阪ラ繧ｭ繝･繝｡繝ｳ繝医∈譖ｴ譁ｰ縲・
  - 迥ｶ豕・
    - 螟画鋤邉ｻ縺ｮ蠕悟濠・・trunc_sat` 邉ｻ縲’64 螟画鋤邉ｻ縲〉einterpret 邉ｻ縺ｪ縺ｩ・峨・譛ｪ逹�謇九・縺溘ａ谺｡繝輔ぉ繝ｼ繧ｺ縺ｧ邯咏ｶ壹�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (`core/math` f32/f64 蜊倬�・ｼ皮ｮ励・ wasm/llvm 荳｡蟇ｾ蠢・
- 逶ｮ逧・
  - `f32_abs/f32_neg/f64_abs/f64_neg` 繧・wasm 蟆ら畑迥ｶ諷九°繧・llvm 荳｡蟇ｾ蠢懊∈諡｡蠑ｵ縺励�∵ｵｮ蜍募ｰ乗焚縺ｮ蝓ｺ遉・API 繧・target 髱樔ｾ晏ｭ倥〒菴ｿ縺医ｋ遽・峇繧貞ｺ・￡繧九�・
- 螳溯｣・
  - `stdlib/core/math.nepl`
    - `f32_abs`
      - wasm: `f32.abs`
      - llvm: `bitcast float->i32` + `and 0x7fffffff` + `bitcast i32->float`
    - `f32_neg`
      - wasm: `f32.neg`
      - llvm: `fneg float`
    - `f64_abs`
      - wasm: `f64.abs`
      - llvm: `bitcast double->i64` + `and 0x7fffffffffffffff` + `bitcast i64->double`
    - `f64_neg`
      - wasm: `f64.neg`
      - llvm: `fneg double`
    - 4髢｢謨ｰ縺ｨ繧・doc comment 繧堤畑騾比ｸｭ蠢・・謇区嶌縺榊・螳ｹ縺ｸ譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/core/math.nepl -o tests/output/math_doctest_current.json -j 1 --no-stdlib`: `39/39 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (`core/math` f32/f64 蝓ｺ遉取ｼ皮ｮ励・豈碑ｼ・・ wasm/llvm 荳｡蟇ｾ蠢・
- 逶ｮ逧・
  - `core/math` 縺ｮ縺・■縲’32/f64 縺ｮ蝓ｺ遉取ｼ皮ｮ励・豈碑ｼ・〒谿九▲縺ｦ縺・◆ wasm 蟆ら畑螳夂ｾｩ繧呈ｮｵ髫守噪縺ｫ llvm 荳｡蟇ｾ蠢懊∈諡｡蠑ｵ縺吶ｋ縲・
  - 蜷梧凾縺ｫ縲√ユ繝ｳ繝励Ξ蝙九ラ繧ｭ繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝医ｒ逕ｨ騾比ｸｭ蠢・・謇区嶌縺阪さ繝｡繝ｳ繝医∈鄂ｮ謠帙☆繧九�・
- 螳溯｣・
  - `stdlib/core/math.nepl`
    - f32:
      - `f32_add/sub/mul/div` 縺ｫ `#if[target=llvm] #llvmir`・・fadd/fsub/fmul/fdiv float`・峨ｒ霑ｽ蜉�
      - `f32_eq/ne/lt/le/gt/ge` 縺ｫ `#if[target=llvm] #llvmir`・・fcmp` + `zext i1 -> i32`・峨ｒ霑ｽ蜉�
      - 蜷・未謨ｰ縺ｮ doc comment 繧呈焔譖ｸ縺榊喧
    - f64:
      - `f64_add/sub/mul/div` 縺ｫ `#if[target=llvm] #llvmir`・・fadd/fsub/fmul/fdiv double`・峨ｒ霑ｽ蜉�
      - `f64_eq/ne/lt/le/gt/ge` 縺ｫ `#if[target=llvm] #llvmir`・・fcmp` + `zext i1 -> i32`・峨ｒ霑ｽ蜉�
      - 蜷・未謨ｰ縺ｮ doc comment 繧呈焔譖ｸ縺榊喧
    - doctest 霑ｽ蜉�:
      - `f32_add`・郁､・焚 assert・・
      - `f64_add`・郁､・焚 assert縲～f64_convert_i32_s` 繧剃ｽｿ縺｣縺ｦ蝙区尠譏ｧ諤ｧ繧貞屓驕ｿ・・
- 螟ｱ謨怜・譫・
  - 霑ｽ蜉�逶ｴ蠕後↓ `stdlib/core/math.nepl::doctest#22` 縺・`no matching overload found` 縺ｧ螟ｱ謨励�・
  - 譬ｹ蝗�縺ｯ f64 繝ｪ繝・Λ繝ｫ繧貞性繧�蠑上・ overload 隗｣豎ｺ縺ｮ譖匁乂諤ｧ縲・
  - `f64_convert_i32_s` 縺ｫ繧医ｋ譏守､ｺ蝙倶ｻ倥￠縺ｸ菫ｮ豁｣縺励※隗｣豸医�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/core/math.nepl -o tests/output/math_doctest_current.json -j 1 --no-stdlib`: `39/39 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `610/610 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (`core/math` i64 遽・峇縺ｮ謇区嶌縺阪ラ繧ｭ繝･繝｡繝ｳ繝域紛蛯・
- 逶ｮ逧・
  - `stdlib/core/math.nepl` 縺ｮ i64 邉ｻ縺ｫ谿九▲縺ｦ縺・◆讖滓｢ｰ逕滓・繝・Φ繝励Ξ譁・ｼ医�御ｸｻ縺ｪ逕ｨ騾斐�阪�瑚埋縺・Λ繝・ヱ縲搾ｼ峨ｒ蟒・ｭ｢縺励�・未謨ｰ縺ｮ逕ｨ騾斐◎縺ｮ繧ゅ・繧定ｪｬ譏弱☆繧区焔譖ｸ縺阪さ繝｡繝ｳ繝医∈鄂ｮ謠帙☆繧九�・
  - doctest 繧偵�・繝・せ繝医こ繝ｼ繧ｹ縺ｫ隍・焚 assert縲肴婿蠑上〒陬懷ｼｷ縺励�∽ｻ墓ｧ倩ｪｬ譏弱→蝗槫ｸｰ讀懆ｨｼ繧剃ｸ�閾ｴ縺輔○繧九�・
- 螳溯｣・
  - `stdlib/core/math.nepl`
    - 謇区嶌縺榊喧:
      - `i64_div_s`, `i64_rem_s`
      - `i64_and/or/xor/shl/shr_s/shr_u/rotl/rotr/clz/ctz/popcnt`
      - `i64_eq/ne/lt_s/lt_u/le_s/le_u/gt_s/gt_u/ge_s/ge_u`
    - doctest 霑ｽ蜉�繝ｻ菫ｮ豁｣:
      - `i64_div_s`, `i64_rem_s`, `i64_and`, `i64_eq`
      - `i64_eq` doctest 縺ｮ unsigned 豈碑ｼ・擅莉ｶ繧・`i64_gt_u` 縺ｫ菫ｮ豁｣・・i64_lt_u -1 1` 縺ｯ false 縺ｮ縺溘ａ・峨�・
  - `todo.md`
    - `math.nepl` doctest 縺ｮ `#target core` 谿ｵ髫守ｧｻ陦梧婿驥晢ｼ・std/test` 萓晏ｭ倬勁蜴ｻ繧貞・陦鯉ｼ峨ｒ譏手ｨ倥�・
- 螟ｱ謨怜・譫・
  - `stdlib/core/math.nepl::doctest#20` 縺ｧ `divide by zero` trap 縺檎匱逕溘�・
  - 譬ｹ蝗�縺ｯ `assert` 譚｡莉ｶ繝溘せ・・nsigned 豈碑ｼ・・逵溷⊃隱､隱搾ｼ峨〒縲√Λ繝ｳ繧ｿ繧､繝�/繧ｳ繝ｼ繝臥函謌蝉ｸ榊・蜷医〒縺ｯ縺ｪ縺九▲縺溘�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i stdlib/core/math.nepl -o tests/output/math_doctest_current.json -j 1 --no-stdlib`: `37/37 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `608/608 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (`math.nepl` 繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝域焔譖ｸ縺榊喧縺ｮ髢句ｧ・
- 逶ｮ逧・
  - 讖滓｢ｰ逧・↓逕滓・縺輔ｌ縺滓ｱ守畑譁・ｼ医�御ｸｻ縺ｪ逕ｨ騾斐→蜻ｼ縺ｳ蜃ｺ縺玲婿繧堤､ｺ縺励∪縺吶�咲ｭ会ｼ峨ｒ蟒・ｭ｢縺励�・未謨ｰ縺ｮ逕ｨ騾斐◎縺ｮ繧ゅ・繧定ｨ倩ｿｰ縺吶ｋ謇区嶌縺阪ラ繧ｭ繝･繝｡繝ｳ繝医∈鄂ｮ謠帙☆繧九�・
  - LLVM 蟇ｾ蠢懈ｸ医∩髢｢謨ｰ縺ｯ縲仝asm/LLVM 縺ｮ蛻・ｲ仙ｮ溯｣・→荳�閾ｴ縺励◆隱ｬ譏弱↓譖ｴ譁ｰ縺吶ｋ縲・
- 螳溯｣・ｼ井ｻ雁屓螳御ｺ・・・・
  - `stdlib/core/math.nepl`
    - `i32_add/sub/mul/div_s/div_u/rem_s/rem_u`
    - `i64_add/sub/mul/div_u/rem_u`
    - `i64_extend_i32_s/u`
    縺ｮ繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝医ｒ謇区嶌縺阪〒蟾ｮ縺玲崛縺医�・
  - doctest 縺ｯ縲・繝・せ繝医こ繝ｼ繧ｹ蜀・↓隍・焚 assert縲阪ｒ謗｡逕ｨ縺励※邁｡貎泌喧縲・
  - 荳ｻ隕・i32/i64 邂苓｡鍋ｳｻ縺ｧ `#if[target=wasm]` 繧帝未謨ｰ螟悶↓鄂ｮ縺乗婿蠑上ｒ繧・ａ縲・未謨ｰ譛ｬ菴灘・縺ｮ target 蛻・ｲ舌∈謠・∴縺溘�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `601/601 pass`
- 邯咏ｶ夊ｪｲ鬘・
  - `math.nepl` 蜈ｨ髢｢謨ｰ縺ｫ蜷梧婿驥昴・謇区嶌縺阪さ繝｡繝ｳ繝医ｒ驕ｩ逕ｨ・育樟譎らせ縺ｧ豎守畑繝・Φ繝励Ξ譁・′螟壽焚谿句ｭ假ｼ峨�・
  - 縺昴・蠕・`mem.nepl` 縺ｪ縺ｩ `stdlib/core` / `stdlib/alloc` 縺ｮ LLVM 蟇ｾ蠢懊ｒ谿ｵ髫守噪縺ｫ螳溯｣・＠縲∵里蟄・wasm 逕ｨ繝・せ繝医ｒ llvm runner 縺ｧ繧る�壹○繧狗憾諷九∈騾ｲ繧√ｋ縲・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (`core/math` 縺ｮ `#wasm/#llvmir` 譛ｬ菴灘・蟯舌∈邨ｱ荳�)
- 閭梧勹:
  - `add/sub/...` 邉ｻ縺ｧ wasm 蛛ｴ繧帝未謨ｰ蜻ｼ縺ｳ蜃ｺ縺励〒蟋碑ｭｲ縺励※縺・◆縺溘ａ縲～#if[target=wasm]` 縺ｮ縲檎峩蠕・蠑上�崎ｦ丞援縺ｨ `#wasm` 逕溘さ繝ｼ繝画婿驥昴ｒ邨ｱ荳�縺ｧ縺阪※縺・↑縺九▲縺溘�・
  - 譛ｫ蟆ｾ縺ｫ譌ｧ譁ｹ蠑擾ｼ・op-level `#if[target=llvm] fn ...`・峨・驥崎､・ｮ夂ｾｩ縺梧ｮ九▲縺ｦ縺翫ｊ縲∽ｻ雁ｾ後・ shadow 隴ｦ蜻翫ヮ繧､繧ｺ貅舌↓縺ｪ縺｣縺ｦ縺・◆縲・
- 螳溯｣・
  - `stdlib/core/math.nepl`
    - `add/sub/mul/div_s/mod_s/lt/eq/ne/le/gt/ge` 縺ｮ wasm 蛛ｴ繧・`#wasm` 逶ｴ譖ｸ縺阪∈邨ｱ荳�縲・
    - 譛ｫ蟆ｾ縺ｫ谿九▲縺ｦ縺・◆譌ｧ `#if[target=llvm] fn add/sub/.../and/or/not` 縺ｮ驥崎､・ｮ夂ｾｩ繧貞炎髯､縲・
    - 髢｢謨ｰ螳夂ｾｩ閾ｪ菴薙・蜈ｱ騾壹・縺ｾ縺ｾ邯ｭ謖√＠縲∵悽菴灘ｼ上・縺ｿ `#if[target=wasm]` / `#if[target=llvm]` 縺ｧ蛻・ｲ舌☆繧句ｽ｢縺ｫ謨ｴ逅・�・
  - `nepl-core/src/codegen_llvm.rs`
    - Parsed 髢｢謨ｰ蜀・・ `#if` 隧穂ｾ｡蠕後↓ `#llvmir/#wasm` 縺・縺､縺�縺第怏蜉ｹ縺ｫ縺ｪ繧九こ繝ｼ繧ｹ繧帝∈謚槭〒縺阪ｋ繧医≧諡｡蠑ｵ縲・
    - 遶ｶ蜷域凾縺ｮ險ｺ譁ｭ `ConflictingRawBodies` 繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `587/587 pass`
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1`: `4/4 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (`#if` 縺ｮ逶ｴ蠕・蠑城←逕ｨ繧帝未謨ｰ蜀・ヶ繝ｭ繝・け縺ｸ諡｡蠑ｵ)
- 閭梧勹:
  - `#if[target=...]` 縺・top-level 縺ｧ縺ｯ讖溯・縺吶ｋ荳�譁ｹ縲・未謨ｰ譛ｬ菴薙ヶ繝ｭ繝・け蜀・・荳�闊ｬ蠑擾ｼ・add` / `let` / `if`・峨↓縺ｯ驕ｩ逕ｨ縺輔ｌ縺ｦ縺・↑縺九▲縺溘�・
  - `fn` 譛ｬ菴薙〒 `#if[target=wasm] #wasm:` / `#if[target=llvm] #llvmir:` 縺ｮ蠖｢繧貞ｰ・擂謗｡逕ｨ縺吶ｋ縺溘ａ縲・未謨ｰ蜀・〒縺ｮ gate 蜃ｦ逅・′蠢・ｦ√□縺｣縺溘�・
- 螳溯｣・
  - `nepl-core/src/typecheck.rs`
    - `check_function` 縺ｫ `target/profile` 繧呈ｸ｡縺吶ｈ縺・↓螟画峩縲・
    - `BlockChecker` 縺ｫ `target/profile` 繧剃ｿ晄戟縲・
    - `check_block` 縺ｧ `Directive::IfTarget/IfProfile` 繧定ｧ｣驥医＠縲～#if` 繧偵�檎峩蠕後・1蠑上・縺ｿ縲埼←逕ｨ縺吶ｋ繧医≧菫ｮ豁｣縲・
    - `select_target_raw_body` 繧定ｿｽ蜉�縺励�・未謨ｰ譛ｬ菴薙′
      `#if ...` + `#wasm/#llvmir` 縺�縺代〒讒区・縺輔ｌ繧句�ｴ蜷医�∬ｩｲ蠖・target 縺ｮ raw body 繧帝∈謚槭＠縺ｦ `HirBody` 蛹悶�・
      ・域囓鮟・lower 縺ｯ陦後ｏ縺壹�∵・遉ｺ `#wasm/#llvmir` 縺ｮ縺ｿ謗｡逕ｨ・・
  - `tests/neplg2.n.md`
    - `iftarget_on_general_call_expression`
    - `iftarget_on_let_expression`
    - `iftarget_on_if_expression`
    繧定ｿｽ蜉�縺励�・未謨ｰ蜀・・荳�闊ｬ蠑上↓蟇ｾ縺吶ｋ `#if` 驕ｩ逕ｨ繧貞屓蟶ｰ蝗ｺ螳壹�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/tests_neplg2_current.json -j 1`: `219/219 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `587/587 pass`
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1`: `4/4 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (`core/math` 縺ｮ LLVM 譏守､ｺ螳溯｣・ｒ逹�謇・+ `#if` 蜊倅ｽ榊屓蟶ｰ)
- 逶ｮ逧・
  - `stdlib/core/math.nepl` 縺ｧ wasm 蟆ら畑縺�縺｣縺溷渕遉取ｼ皮ｮ励ｒ縲∵囓鮟・lower 縺ｧ縺ｯ縺ｪ縺・`#llvmir` 譏守､ｺ螳溯｣・〒谿ｵ髫守噪縺ｫ LLVM 蟇ｾ蠢懊☆繧九�・
  - `#if[target=...]` 縺ｮ驕ｩ逕ｨ蜊倅ｽ阪ｒ縲檎峩蠕後・1蠑上�阪↓蝗ｺ螳壹☆繧句屓蟶ｰ繧定ｿｽ蜉�縺吶ｋ縲・
- 螳溯｣・
  - `stdlib/core/math.nepl`
    - `#if[target=llvm]` 縺ｮ蜷悟錐髢｢謨ｰ螳夂ｾｩ繧定ｿｽ蜉�・・oc comment 縺ｯ譌｢蟄倬未謨ｰ縺ｨ蜈ｱ譛会ｼ峨�・
    - 霑ｽ蜉�縺励◆譏守､ｺ LLVM 螳溯｣・
      - `i32_*` 縺ｮ蝓ｺ遉守ｮ苓｡・豈碑ｼ・ｼ・add/sub/mul/div/rem/eq/ne/lt/le/gt/ge` 縺ｮ signed/unsigned 蠢・ｦ∝・・・
      - `i64_*` 縺ｮ蝓ｺ遉守ｮ苓｡・豈碑ｼ・ｼ・add/sub/mul/div_u/rem_u/lt_u/le_u/gt_u/ge_u/lt_s/gt_s`・・
      - `i64_extend_i32_u/s`
      - 譌ｧ繧ｨ繧､繝ｪ繧｢繧ｹ `add/sub/mul/div_s/mod_s/lt/eq/ne/le/gt/ge/and/or/not`
  - `nepl-core/src/codegen_llvm.rs`
    - 譛ｪ蟇ｾ蠢・`Parsed` / `#wasm` 髢｢謨ｰ譛ｬ菴薙・ LLVM 邨瑚ｷｯ縺ｧ證鈴ｻ吝､画鋤縺帙★繧ｹ繧ｭ繝・・縲・
    - `#if[target=...]` / `#if[profile=...]` 縺ｮ gate 隧穂ｾ｡縺ｯ蠑輔″邯壹″縲檎峩蠕後・1蠑上�榊腰菴阪〒蜃ｦ逅・�・
  - `tests/llvm_target.n.md`
    - `llvm_math_add_from_stdlib` 繧定ｿｽ蜉�縺励�～#import "core/math"` + `call @add` 縺・LLVM 縺ｧ騾壹ｋ縺薙→繧堤｢ｺ隱阪�・
  - `tests/neplg2.n.md`
    - `iftarget_applies_to_next_single_expression_only` 繧定ｿｽ蜉�縺励�～#if` 縺・蠑上・縺ｿ驕ｩ逕ｨ縺輔ｌ繧句屓蟶ｰ繧貞崋螳壹�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1`: `4/4 pass`
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/tests_neplg2_current.json -j 1`: `216/216 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `584/584 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (LLVM core遘ｻ險ｭ + nodesrc dual runner 蝓ｺ逶､)
- 逶ｮ逧・
  - LLVM IR 逕滓・驛ｨ繧・`nepl-core` 縺ｫ遘ｻ縺励�～nepl-cli` 縺ｯ clang 螳溯｡後↑縺ｩ繝帙せ繝井ｾ晏ｭ伜・逅・・縺ｿ諡・ｽ薙☆繧区ｧ区・縺ｸ謨ｴ逅・�・
  - `nodesrc/tests.js` 縺ｧ wasm 縺ｨ llvm 縺ｮ荳｡邨瑚ｷｯ繧貞酔荳�蝓ｺ逶､縺九ｉ螳溯｡悟庄閭ｽ縺ｫ縺吶ｋ縲・
- 螳溯｣・
  - `nepl-core/src/codegen_llvm.rs` 繧定ｿｽ蜉�縲・
    - `emit_ll_from_module` 繧・`no_std + alloc` 縺ｧ螳溯｣・�・
    - `#llvmir` 騾｣邨・+ Parsed 髢｢謨ｰ縺ｮ譛�蟆・subset (`fn <()->i32>(): <int literal>`) lower 繧呈署萓帙�・
    - error 蝙・`LlvmCodegenError` 繧貞ｰ主・縲・
  - `nepl-cli/src/codegen_llvm.rs` 縺ｯ toolchain check 縺ｮ縺ｿ縺ｸ謨ｴ逅・�・
    - `NEPL_LLVM_CLANG_BIN` 繧定ｿｽ蜉�縺励�￣ATH 遶ｶ蜷域凾縺ｧ繧・clang 21.1.0 繧呈・遉ｺ謖・ｮ壼庄閭ｽ縺ｫ縺励◆縲・
  - `nepl-cli/src/main.rs`:
    - LLVM IR 逕滓・繧・`nepl_core::codegen_llvm` 蜻ｼ縺ｳ蜃ｺ縺励∈蛻・崛縲・
    - `--target core/std` 繧ｨ繧､繝ｪ繧｢繧ｹ繧貞女逅・�・
  - target gate 菫ｮ豁｣・域�ｹ蝗�菫ｮ豁｣・・
    - `#if[target=wasm]` 縺・LLVM 縺ｧ繧ら悄縺ｫ縺ｪ縺｣縺ｦ縺・◆荳肴紛蜷医ｒ菫ｮ豁｣縲・
    - `nepl-core/src/compiler.rs` / `nepl-core/src/typecheck.rs` 縺ｧ `wasm` 蛻､螳壹ｒ `Wasm|Wasi` 縺ｮ縺ｿ縺ｫ蛻ｶ髯舌�・
    - `core/std` gate 繧定ｿｽ蜉� (`core = wasm|wasi|llvm`, `std = wasi|llvm`)縲・
  - `nodesrc/tests.js`:
    - `--runner wasm|llvm|all` 繧定ｿｽ蜉�縲・
    - `--llvm-all` 繧定ｿｽ蜉�縺励�・�壼ｸｸ doctest 繧・LLVM 邨瑚ｷｯ縺ｧ繧ょ屓縺帙ｋ繧医≧縺ｫ縺励◆縲・
    - LLVM runner 縺ｯ豈弱こ繝ｼ繧ｹ `cargo run` 繧貞ｻ・ｭ｢縺励�～cargo build -p nepl-cli` 蠕後↓ `target/debug/nepl-cli` 繧堤峩謗･蜻ｼ縺ｳ蜃ｺ縺呎婿蠑上∈螟画峩縲・
    - LLVM runner 縺ｯ `-j` 繝吶・繧ｹ縺ｧ荳ｦ蛻怜ｮ溯｡後�・
    - `NEPL_LLVM_CLANG_BIN` 繧・runner 蛛ｴ縺九ｉ閾ｪ蜍戊ｨｭ螳夲ｼ・/opt/llvm-21.1.0/bin/clang` 蜆ｪ蜈茨ｼ峨�・
  - workflow:
    - `.github/workflows/nepl-test.yml` 繧・`nepl-test-wasi.yml` 縺ｸ蛻・屬縲・
    - `.github/workflows/nepl-test-llvm.yml` 繧定ｿｽ蜉�縺励�…lang 21.1.0 繧貞ｰ主・縺励※ `nodesrc/tests.js --runner llvm` 繧貞ｮ溯｡後�・
  - 繝・せ繝・
    - `tests/llvm_target.n.md` 繧定ｿｽ蜉�・・aw #llvmir / parsed subset / #wasm reject・峨�・
    - `tests/sort.n.md` 縺ｮ target 繧・`#target core` 縺ｸ遘ｻ陦碁幕蟋九�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥縲・
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `583/583 pass`縲・
  - `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_current.json --runner llvm --no-tree --no-stdlib -j 2`: `3/3 pass`縲・
  - `node nodesrc/tests.js -i tests/sort.n.md -o tests/output/sort_dual.json --runner all --llvm-all --no-stdlib --no-tree -j 2`: `6/12 pass`・・asm pass, llvm fail・峨�・
- 螟ｱ謨怜・譫・
  - `sort.n.md` 縺ｮ LLVM 蛛ｴ螟ｱ謨励・ runner/target 蛻､螳壹・荳榊・蜷医〒縺ｯ縺ｪ縺上�´LVM backend 縺ｮ lower 蟇ｾ蠢懃ｯ・峇荳崎ｶｳ縺悟次蝗�縲・
  - 莉｣陦ｨ繧ｨ繝ｩ繝ｼ:
    - `llvm target currently supports only subset lowering for parsed functions; function 'get' is not in supported subset`
  - 縺励◆縺後▲縺ｦ谺｡繝輔ぉ繝ｼ繧ｺ縺ｯ `stdlib/core` / `stdlib/alloc` 縺瑚ｦ∵ｱゅ☆繧・Parsed/HIR 繧呈ｮｵ髫守噪縺ｫ LLVM IR 縺ｸ lower 縺吶ｋ螳溯｣・僑蠑ｵ縺悟ｿ・ｦ√�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (clang 21.1.0 縺ｮ LLVM IR 迺ｰ蠅・｢ｺ隱阪→謇矩�・嶌謨ｴ蛯・
- 逶ｮ逧・
  - `todo.md` 縺ｮ LLVM IR 鬆・岼縺ｫ縺ゅｋ縲形LLVM_SYS_211_PREFIX` 驕狗畑謨ｴ逅・→ doc 縺ｸ縺ｮ繧ｻ繝・ヨ繧｢繝・・險倩ｼ峨�阪ｒ蜈医↓螳御ｺ・＠縲・
    LLVM IR 繧ｿ繝ｼ繧ｲ繝・ヨ螳溯｣・凾縺ｮ蜑肴署迺ｰ蠅・ｒ蝗ｺ螳壹☆繧九�・
- 遒ｺ隱・
  - `clang --version`: `clang version 21.1.0`・・/opt/llvm-21.1.0/bin`・・
  - `llvm-as --version`: `LLVM version 21.1.0`
  - `llc --version`: `LLVM version 21.1.0`
- 螳溷虚菴懈､懆ｨｼ:
  - `tmp/llvm_ir/hello.c` 繧剃ｽ懈・縺励�～clang -S -emit-llvm` 縺ｧ `hello.ll` 繧堤函謌舌�・
  - `lli tmp/llvm_ir/hello.ll` 縺ｧ `sum=42` 繧堤｢ｺ隱阪�・
  - `llc -relocation-model=pic -filetype=obj` -> `clang` 繝ｪ繝ｳ繧ｯ蠕後・螳溯｡後〒繧・`sum=42` 繧堤｢ｺ隱阪�・
- 繝峨く繝･繝｡繝ｳ繝域峩譁ｰ:
  - 霑ｽ蜉�: `doc/llvm_ir_setup.md`
    - 蠢・�医ヤ繝ｼ繝ｫ縺ｮ繝舌・繧ｸ繝ｧ繝ｳ遒ｺ隱肴焔鬆・
    - `LLVM_SYS_211_PREFIX=/opt/llvm-21.1.0` 險ｭ螳・
    - LLVM IR 逕滓・繝ｻ螳溯｡後・繧ｪ繝悶ず繧ｧ繧ｯ繝亥喧縺ｮ譛�遏ｭ謇矩�・
  - 譖ｴ譁ｰ: `README.md`
    - 縲碁幕逋ｺ繝峨く繝･繝｡繝ｳ繝医�咲ｯ�繧定ｿｽ蜉�縺励�～doc/llvm_ir_setup.md` 縺ｸ縺ｮ蟆守ｷ壹ｒ霑ｽ蜉�縲・
- `todo.md` 蜿肴丐:
  - LLVM IR 鬆・岼縺九ｉ螳御ｺ・ｸ医∩縺ｮ
    - 縲形inkwell`/`llvm-sys` 縺ｮ繝舌・繧ｸ繝ｧ繝ｳ蝗ｺ螳壹→ `LLVM_SYS_211_PREFIX` 驕狗畑繧呈紛逅・＠縲～doc/` 縺ｫ繧ｻ繝・ヨ繧｢繝・・繧定ｨ倩ｼ峨☆繧九�ゅ�・
    繧貞炎髯､縲・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (譌ｧ繧ｿ繝励Ν蝙玖ｨ俶ｳ輔・谿矩ｪｸ繧・Rust 繝・せ繝医°繧蛾勁蜴ｻ)
- 閭梧勹:
  - 譌ｧ繧ｿ繝励Ν蝙区ｳｨ驥・`((i32,i32))` / `<(i32,i32)>` 縺・`nepl-core/tests` 縺ｫ谿九▲縺ｦ縺翫ｊ縲・
    譌ｧ莉墓ｧ伜ｻ・ｭ｢蠕後・ parser/typecheck 譁ｹ驥昴→荳肴紛蜷医↓縺ｪ縺｣縺ｦ縺・◆縲・
- 螳溯｣・
  - `nepl-core/tests/pipe_operator.rs`
    - `pipe_tuple_source` 縺ｮ `fn f` 繧呈眠莉墓ｧ倥↓蜷医ｏ縺帙※
      `fn f <.T> <(.T)->i32> (t): 2` 縺ｸ譖ｴ譁ｰ縲・
  - `nepl-core/tests/tuple_new_syntax.rs`
    - `tuple_as_function_arg`: `fn take <.T> <(.T)->i32>` 縺ｫ譖ｴ譁ｰ縲・
    - `tuple_return_value`: `fn make <()->.Pair>` 縺ｫ譖ｴ譁ｰ縲・
    - `tuple_inside_struct`: `pair <.Pair>` 縺ｫ譖ｴ譁ｰ縲・
    - `tuple_type_annotated`: 譌ｧ蝙区ｳｨ驥・`<(i32,i32)>` 繧貞炎髯､縲・
- 讀懆ｨｼ:
  - `cargo test -p nepl-core --test pipe_operator --test tuple_new_syntax`: `40/40 pass`
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/pipe_operator.n.md -i tests/tuple_new_syntax.n.md -o tests/output/pipe_tuple_rs_sync.json`: `219/219 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (capture 縺ゅｊ髢｢謨ｰ蛟､繧呈・遉ｺ逧・↓諡貞凄)
- 逶ｮ逧・
  - closure conversion 譛ｪ螳溯｣・・迥ｶ諷九〒 capture 莉倥″髢｢謨ｰ繧・`@fn` 縺ｧ蛟､蛹悶＠縺滄圀縲・
    荳区ｵ√〒荳肴ｭ｣縺ｪ逕滓・縺ｸ騾ｲ繧�縺ｮ繧帝亟縺舌�・
- 螳溯｣・
  - `nepl-core/src/typecheck.rs`
    - `@` 莉倥″隴伜挨蟄占ｧ｣豎ｺ譎ゅ↓縲∝ｯｾ雎｡縺・capture 縺ゅｊ髢｢謨ｰ縺ｪ繧・
      `capturing function cannot be used as a function value yet` 繧定ｿ斐☆縲・
    - `@` 繧帝撼 callable 縺ｫ驕ｩ逕ｨ縺励◆蝣ｴ蜷医・
      `only callable symbols can be referenced with '@'` 繧定ｿ斐☆縲・
  - `tests/functions.n.md`
    - `function_value_capture_not_supported_yet`・・compile_fail`・峨ｒ霑ｽ蜉�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `560/560 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (`call_indirect` 繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ縺ｮ蜴ｳ蟇・喧)
- 逶ｮ逧・
  - 鬮倬嚴髢｢謨ｰ縺ｮ蜻ｼ縺ｳ蜃ｺ縺礼ｵ瑚ｷｯ縺ｧ縲∵尠譏ｧ縺ｪ荳倶ｽ阪ヵ繧ｩ繝ｼ繝ｫ繝舌ャ繧ｯ繧呈ｸ帙ｉ縺励�～FnValue` 荳ｭ蠢・・隕丞援縺ｸ蝗ｺ螳壹☆繧九�・
- 螳溯｣・
  - `nepl-core/src/typecheck.rs`
    - `CallIndirect` fallback 縺ｫ繧ｬ繝ｼ繝峨ｒ霑ｽ蜉�:
      - `FnValue` 縺ｯ險ｱ蜿ｯ
      - 縺昴ｌ莉･螟悶・縲碁未謨ｰ蝙九→縺励※蝙倶ｻ倥￠貂医∩縲阪・蝣ｴ蜷医・縺ｿ險ｱ蜿ｯ
      - 髱樣未謨ｰ蝙九・ `indirect call requires a function value` 繧定ｿ斐＠縺ｦ蛛懈ｭ｢
  - `tests/tree/08_function_value_call_indirect.js`
    - 譌｢蟄倥・ `CallIndirect` 遒ｺ隱阪↓蜉�縺医※ `FnValue` 繝弱・繝牙ｭ伜惠繧呈､懆ｨｼ
- `todo.md` 蜿肴丐:
  - 鬮倬嚴髢｢謨ｰ鬆・岼縺九ｉ螳御ｺ・ｸ医∩縺ｮ
    - 縲形_unknown` 繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ蟒・ｭ｢縲・
    繧貞炎髯､縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node tests/tree/run.js`: `8/8 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `559/559 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (`@fn` 縺ｮ HIR 譏守､ｺ蛹・
- 逶ｮ逧・
  - `todo.md` 譛�蜆ｪ蜈磯�・岼縺�縺｣縺溘�碁未謨ｰ蛟､・・@fn`・峨ｒ HIR 縺ｧ譏守､ｺ陦ｨ迴ｾ縲阪ｒ螳御ｺ・＠縲～Var` 縺ｨ諢丞袖隲悶ｒ蛻・屬縺吶ｋ縲・
- 螳溯｣・
  - `nepl-core/src/hir.rs`
    - `HirExprKind::FnValue(String)` 繧定ｿｽ蜉�縲・
  - `nepl-core/src/typecheck.rs`
    - `Symbol::Ident(..., forced_value=true)` 縺九▽ callable 隗｣豎ｺ譎ゅ↓ `HirExprKind::FnValue` 繧堤函謌舌�・
    - 譌｢蟄倥・ value 隴伜挨蟄舌・蠑輔″邯壹″ `HirExprKind::Var` 繧堤函謌舌�・
  - `nepl-core/src/codegen_wasm.rs`
    - `FnValue` 繧帝未謨ｰ繝・・繝悶Ν index (`i32.const fidx`) 縺ｸ譏守､ｺ lowering縲・
  - `nepl-core/src/monomorphize.rs`
    - `FnValue` 縺ｮ蜊倡嶌蛹厄ｼ磯未謨ｰ蜷阪・ instantiation/mangled 蜷崎ｧ｣豎ｺ・峨↓蟇ｾ蠢懊�・
  - `nepl-web/src/lib.rs`
    - semantics API 縺ｮ kind 蛻玲嫌縺ｨ蠑剰ｵｰ譟ｻ縺ｫ `FnValue` 繧定ｿｽ蜉�縲・
  - `nepl-core/src/compiler.rs` / `nepl-core/src/passes/move_check.rs`
    - 譁ｰ variant 縺ｫ霑ｽ蠕難ｼ育ｶｲ鄒・�ｧ繝ｻ謖吝虚邯ｭ謖・ｼ峨�・
- 繝・せ繝・
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `559/559 pass`
  - 騾比ｸｭ縺ｧ `tests/functions.n.md::doctest#14` 縺御ｸ�譎ょ､ｱ謨暦ｼ・unknown function value add_op`・峨＠縺溘′縲・
    `FnValue` 縺ｮ蜊倡嶌蛹悶ヵ繧ｩ繝ｼ繝ｫ繝舌ャ繧ｯ荳崎ｶｳ縺悟次蝗�縺ｧ縺ゅｊ縲～monomorphize` 菫ｮ豁｣蠕後↓隗｣豸医�・
- `todo.md` 蜿肴丐:
  - 螳御ｺ・�・岼・・@fn` 縺ｮ HIR 譏守､ｺ蛹厄ｼ峨ｒ蜑企勁縲・
  - 逡ｪ蜿ｷ繧堤ｹｰ繧贋ｸ翫￡縺ｦ譛ｪ螳御ｺ・・縺ｿ縺ｸ謨ｴ逅・�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (tree API 蝗槫ｸｰ霑ｽ蜉� + todo 謨ｴ逅・
- 逶ｮ逧・
  - 荳頑ｵ・ｼ・arse/semantics API・峨〒 `@fn` 髢｢謨ｰ蛟､縺ｮ謖吝虚繧貞崋螳壹＠縲∵ｬ｡繝輔ぉ繝ｼ繧ｺ縺ｮ HIR 譏守､ｺ蛹紋ｽ懈･ｭ縺ｮ蝨溷床繧剃ｽ懊ｋ縲・
  - `todo.md` 繧呈悴螳御ｺ・�・岼縺ｮ縺ｿ縺ｸ謨ｴ逅・☆繧九�・
- 螟画峩:
  - 霑ｽ蜉�: `tests/tree/08_function_value_call_indirect.js`
    - `@inc` 縺・forced-value 縺ｨ縺励※ parse 縺輔ｌ繧九％縺ｨ繧堤｢ｺ隱阪�・
    - 髢｢謨ｰ蛟､蜻ｼ縺ｳ蜃ｺ縺励′ `CallIndirect` 縺ｨ縺励※ semantics 縺ｫ蜃ｺ繧九％縺ｨ繧堤｢ｺ隱阪�・
  - 譖ｴ譁ｰ: `todo.md`
    - 螳御ｺ・ｸ医∩縺ｮ
      - `ValueNs/CallableNs` 蛻・屬
      - nested `fn`/`let` 蜻ｼ縺ｳ蜃ｺ縺礼ｵ瑚ｷｯ
      繧呈怙蜆ｪ蜈磯�・岼縺九ｉ蜑企勁縲・
    - 譛ｪ螳御ｺ・→縺励※ `@fn` HIR 譏守､ｺ蛹悶ｒ谿狗ｽｮ縲・
    - stdlib 繝ｪ繝輔ぃ繧ｯ繧ｿ繝ｪ繝ｳ繧ｰ・・kp` 蠖｢蠑冗ｵｱ荳� + 隍・尅蜃ｦ逅・〒謾ｹ陦後ヱ繧､繝玲ｴｻ逕ｨ・峨ｒ霑ｽ險倥�・
- 蜈ｱ譛峨＆繧後◆ CI 繧ｨ繝ｩ繝ｼ (`args_sizes_get` 譛ｪ螳夂ｾｩ) 縺ｫ縺､縺・※:
  - 繝ｭ繝ｼ繧ｫ繝ｫ蜀咲樟繧ｳ繝槭Φ繝・
    - `cargo run -p nepl-cli -- --target wasi --profile debug --input examples/nm.nepl --output target/ci-nm`
  - 邨先棡: `compile_module returned Ok`・亥・迴ｾ縺帙★・峨�・
  - 蛻､螳・ 逶ｴ霑大ｷｮ蛻・〒隗｣豸域ｸ医∩縲√∪縺溘・蜿､縺・CI 繝ｭ繧ｰ縺ｧ縺ゅｋ蜿ｯ閭ｽ諤ｧ縺碁ｫ倥＞縲ょｼ輔″邯壹″ workflow 蛛ｴ縺ｮ蜀榊ｮ溯｡後〒逶｣隕悶☆繧九�・

# 2026-02-21 菴懈･ｭ繝｡繝｢ (non-mut let 蜑肴婿蜿ら・縺ｮ螳溯｣・ｮ御ｺ・
- 閭梧勹:
  - `plan.md` 莉墓ｧ倥〒縺ｯ縲悟ｷｻ縺堺ｸ翫￡縺ｯ `mut` 縺ｪ縺・`let` 縺ｨ `fn` 縺ｮ縺ｿ縺ｫ驕ｩ逕ｨ縲阪□縺後�～let y add x 4; let x 5` 縺・`unknown variable x` 縺ｧ螟ｱ謨励＠縺ｦ縺・◆縲・
- 譬ｹ蝗�:
  - `typecheck` 蛛ｴ縺ｮ隗｣豎ｺ縺�縺代〒縺ｪ縺上�～codegen_wasm` 蛛ｴ縺ｮ繝ｭ繝ｼ繧ｫ繝ｫ蜑ｲ蠖薙′縲悟・迴ｾ鬆・匳骭ｲ縲阪□縺｣縺溘◆繧√�・
    蠕梧婿 `let x` 縺ｮ蜑阪〒 `Var(x)` 繧堤函謌舌☆繧九→ `unknown variable` 縺ｧ螟ｱ謨励＠縺ｦ縺・◆縲・
- 螳溯｣・
  - `nepl-core/src/codegen_wasm.rs`
    - `gen_block` 縺ｮ繧ｹ繧ｳ繝ｼ繝鈴幕蟋狗峩蠕後↓ `predeclare_block_locals` 繧定ｿｽ蜉�縲・
    - 繝悶Ο繝・け蜀・・ `HirExprKind::Let` 繧貞・陦瑚ｵｰ譟ｻ縺励�～LocalMap` 縺ｫ莠句燕逋ｻ骭ｲ縲・
  - `nepl-core/src/typecheck.rs`
    - `lookup_value_for_read` 繧貞ｰ主・縺励�∬ｪｭ縺ｿ蜿悶ｊ譎ゅ・ non-mut hoist fallback 邨瑚ｷｯ繧呈紛逅・ｼ郁・蟾ｱ蛻晄悄蛹悶・髯､螟厄ｼ峨�・
  - `tests/shadowing.n.md`
    - `hoist_nonmut_let_allows_forward_reference` 繧・`neplg2:test`・・et: 9・峨∈謌ｻ縺励�・�夐℃繧堤｢ｺ隱阪�・
- 邨先棡:
  - `mut let` 蜑肴婿蜿ら・縺ｯ蠑輔″邯壹″ compile_fail縲・
  - `non-mut let` 縺ｨ `fn` 縺ｮ蜑肴婿蜿ら・縺ｯ騾夐℃縲・
- `todo.md` 蜿肴丐:
  - 螳御ｺ・＠縺溘�形let`/`fn` 縺ｮ蟾ｻ縺堺ｸ翫￡邨ｱ荳�縲阪し繝夜�・岼繧貞炎髯､縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/functions.n.md -i tests/neplg2.n.md -o tests/output/namespace_phase_current.json -j 1`: `243/243 pass`
  - `node tests/tree/run.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `558/558 pass`

# 2026-02-21 菴懈･ｭ繝｡繝｢ (蟾ｻ縺堺ｸ翫￡莉墓ｧ倥・蝗槫ｸｰ繝・せ繝郁ｿｽ蜉�縺ｨ迴ｾ迥ｶ蝗ｺ螳・
- 逶ｮ逧・
  - `todo.md` 縺ｮ縲形let`/`fn` 蟾ｻ縺堺ｸ翫￡邨ｱ荳�縲阪↓蜷代￠縲∫樟迥ｶ謖吝虚繧偵ユ繧ｹ繝医〒蝗ｺ螳壹＠縺ｦ蟾ｮ蛻・ｒ蜿ｯ隕門喧縲・
- 螟画峩:
  - `tests/shadowing.n.md`
    - 譌｢蟄倥こ繝ｼ繧ｹ蜷阪・ `*_currently_fails` 繧呈紛逅・ｼ磯�壼ｸｸ繧ｱ繝ｼ繧ｹ縺ｸ謾ｹ蜷搾ｼ峨�・
    - 蟾ｻ縺堺ｸ翫￡髢｢騾｣繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�:
      - `hoist_mut_let_disallows_forward_reference`・・ompile_fail・・
      - `hoist_nested_fn_allows_forward_reference`・・ass・・
      - `hoist_nonmut_let_allows_forward_reference`・育樟迥ｶ縺ｯ compile_fail 縺ｨ縺励※蝗ｺ螳夲ｼ・
- `nepl-core/src/typecheck.rs`
  - 隴伜挨蟄占ｧ｣豎ｺ縺ｧ縲～defined` 貂医∩隗｣豎ｺ縺ｫ螟ｱ謨励＠縺溷�ｴ蜷医・ non-mut hoist fallback 繧定ｿｽ蜉�・郁・蟾ｱ蛻晄悄蛹悶・髯､螟厄ｼ峨�・
- 迴ｾ迥ｶ隧穂ｾ｡:
  - `fn` 縺ｮ蜑肴婿蜿ら・縺ｯ騾壹ｋ荳�譁ｹ縲～non-mut let` 縺ｮ蜑肴婿蜿ら・縺ｯ譛ｪ蟇ｾ蠢懊�・
  - fallback 繧定ｿｽ蜉�縺励※繧・`let y ... x` / `let x ...` 蠖｢蠑上・譛ｪ隗｣豸医・縺溘ａ縲√ユ繧ｹ繝医・ `compile_fail` 縺ｧ蝗ｺ螳夂ｶｭ謖√�・
  - 縺薙ｌ縺ｯ `todo.md` 縺ｮ蟾ｻ縺堺ｸ翫￡邨ｱ荳�繧ｿ繧ｹ繧ｯ縺ｨ縺励※邯咏ｶ夲ｼ井ｻ墓ｧ伜ｷｮ蛻・→縺励※譏守｢ｺ蛹厄ｼ峨�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/functions.n.md -i tests/neplg2.n.md -o tests/output/namespace_phase_current.json -j 1`: `243/243 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `558/558 pass`

# 2026-02-21 菴懈･ｭ繝｡繝｢ (ValueNs/CallableNs 蛻・屬縺ｮ谿ｵ髫主ｰ主・: Env 繧ｹ繧ｳ繝ｼ繝励ｒ迚ｩ逅・・髮｢)
- 逶ｮ逧・
  - `todo.md` 譛�蜆ｪ蜈磯�・岼・・ValueNs` 縺ｨ `CallableNs` 縺ｮ蛻・屬・峨ｒ繝・・繧ｿ讒矩��繝ｬ繝吶Ν縺ｧ蜑埼�ｲ縺輔○繧九�・
- 螳溯｣・
  - `nepl-core/src/typecheck.rs`
    - `Env.scopes: Vec<Vec<Binding>>` 繧貞ｻ・ｭ｢縺励�～Scope { values, callables }` 縺ｫ螟画峩縲・
    - `BindingKind` 縺ｫ `is_var` / `is_callable` 繧定ｿｽ蜉�縺励�∵諺蜈･蜈医ｒ荳�蜈・愛螳壹�・
    - `insert_global` / `insert_local` / `remove_duplicate_func` / 蜷・lookup 繧呈眠讒矩��縺ｫ蟇ｾ蠢懊�・
    - 繝ｭ繝ｼ繧ｫ繝ｫ隕丞援:
      - value 縺ｯ蜷悟錐 value/callable 縺後≠繧九→遖∵ｭ｢
      - callable 縺ｯ蜷悟錐 value 縺後≠繧九→遖∵ｭ｢・亥酔蜷・callable 縺ｯ繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨→縺励※險ｱ蜿ｯ・・
- 蜉ｹ譫・
  - 蜷榊燕遨ｺ髢灘・髮｢縺後�悟他縺ｳ蜃ｺ縺怜・縺ｮ諷｣鄙偵�阪°繧峨�檎腸蠅・ョ繝ｼ繧ｿ讒矩��縲阪∈遘ｻ陦後�・
  - 莉雁ｾ後・ ValueNs/CallableNs 螳梧・・亥ｷｻ縺堺ｸ翫￡繝ｻshadow policy 縺ｮ蜴ｳ蟇・喧・峨↓蜷代￠縺溷渕逶､繧堤｢ｺ遶九�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/functions.n.md -i tests/neplg2.n.md -o tests/output/namespace_envsplit_current.json -j 1`: `240/240 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 菴懈･ｭ繝｡繝｢ (ValueNs/CallableNs 蛻・屬縺ｮ谿ｵ髫主ｰ主・: 譌ｧ lookup 繝ｩ繝・ヱ蜑企勁)
- 逶ｮ逧・
  - `typecheck` 蜀・〒谿九▲縺ｦ縺・◆譖匁乂縺ｪ `lookup`/`lookup_all` 蜿ら・繧帝勁蜴ｻ縺励�∫畑騾泌挨 API 縺ｸ縺ｮ邨ｱ荳�繧帝�ｲ繧√ｋ縲・
- 螳溯｣・
  - `nepl-core/src/typecheck.rs`
    - `Symbol::Ident` 縺ｮ fallback 繧・`lookup_any_defined` 縺ｫ螟画峩縲・
    - 莠呈鋤繝ｩ繝・ヱ `lookup` / `lookup_all` 繧貞炎髯､縲・
    - 鄂ｮ謠帛ｮ御ｺ・ｾ後・謗｢邏｢ API 縺ｯ莉･荳九∈邨ｱ荳�:
      - 蛟､: `lookup_value`
      - 髢｢謨ｰ: `lookup_all_callables` / `lookup_callable_any`
      - 莉ｻ諢丞ｮ夂ｾｩ貂医∩: `lookup_any_defined` / `lookup_all_any_defined`
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/functions.n.md -i tests/neplg2.n.md -o tests/output/namespace_phase_current.json -j 1`: `240/240 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 菴懈･ｭ繝｡繝｢ (ValueNs/CallableNs 蛻・屬縺ｮ谿ｵ髫主ｰ主・: 譏守､ｺ lookup API 縺ｸ邨ｱ荳�)
- 逶ｮ逧・
  - `typecheck` 縺ｧ `lookup/lookup_all` 縺ｮ諢丞峙縺梧尠譏ｧ縺ｪ邂・園繧呈ｸ帙ｉ縺励�～ValueNs`/`CallableNs` 蛻・屬繧帝�ｲ繧√ｋ縲・
- 螳溯｣・
  - `nepl-core/src/typecheck.rs`
    - `Env` 縺ｫ譏守､ｺ API 繧定ｿｽ蜉�:
      - `lookup_any_defined`
      - `lookup_all_any_defined`
    - 譌｢蟄倥・ `lookup`/`lookup_all` 縺ｯ莠呈鋤繝ｩ繝・ヱ縺ｨ縺励※谿九＠縲∝他縺ｳ蜃ｺ縺怜・繧呈ｮｵ髫守ｽｮ謠帙�・
    - 鄂ｮ謠帙＠縺滉ｸｻ縺ｪ邂・園:
      - enum/struct 蜷崎｡晉ｪ∝愛螳・ `lookup_any_defined`
      - enum variant/struct constructor 譌｢蟄伜愛螳・ `lookup_all_callables`
      - `noshadow` 遶ｶ蜷亥愛螳・ `lookup_all_any_defined`
      - 隴伜挨蟄・fallback 蛟呵｣懷・謖・ `lookup_all_any_defined`
- 蜉ｹ譫・
  - 髢｢謨ｰ隗｣豎ｺ縺ｨ蛟､隗｣豎ｺ縺ｮ邨瑚ｷｯ縺後さ繝ｼ繝我ｸ翫〒蛻､蛻･縺励ｄ縺吶￥縺ｪ繧翫�∽ｻ雁ｾ後・ namespace 蛻・屬繝ｪ繝輔ぃ繧ｯ繧ｿ繝ｪ繝ｳ繧ｰ縺ｮ螳牙・諤ｧ繧貞髄荳翫�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/functions.n.md -o tests/output/shadowing_functions_current.json -j 1`: `205/205 pass`
  - `node tests/tree/run.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 菴懈･ｭ繝｡繝｢ (ValueNs/CallableNs 蛻・屬縺ｮ谿ｵ髫主ｰ主・: callable 蟆ら畑邨瑚ｷｯ縺ｮ諡｡螟ｧ)
- 逶ｮ逧・
  - `todo.md` 譛�蜆ｪ蜈医・蜷榊燕遨ｺ髢灘・髮｢繧堤ｶ咏ｶ壹＠縲…allable 縺ｨ value 縺ｮ謗｢邏｢邨瑚ｷｯ繧偵ｈ繧頑・遒ｺ縺ｫ蛻・屬縲・
- 螳溯｣・
  - `nepl-core/src/typecheck.rs`
    - `fn alias` 縺ｮ繧ｿ繝ｼ繧ｲ繝・ヨ謗｢邏｢繧・`lookup_all` 縺九ｉ `lookup_all_callables` 縺ｫ螟画峩縲・
    - entry 隗｣豎ｺ縺ｮ蛟呵｣懈爾邏｢繧・`lookup_all` 縺九ｉ `lookup_all_callables` 縺ｫ螟画峩縲・
    - trait 繝｡繧ｽ繝・ラ蜻ｼ縺ｳ蜃ｺ縺苓｣懷勧蛻・ｲ舌・蟄伜惠蛻､螳壹ｒ `lookup_all_callables` 縺ｫ螟画峩縲・
  - 縺薙ｌ縺ｫ繧医ｊ縲・未謨ｰ隗｣豎ｺ繝輔ぉ繝ｼ繧ｺ縺ｧ value 蛟呵｣懊ｒ豺ｷ蝨ｨ縺輔○縺ｪ縺・ｵ瑚ｷｯ繧呈僑螟ｧ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/functions.n.md -o tests/output/functions_current.json -j 1`: `187/187 pass`
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/neplg2_current.json -j 1`: `203/203 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 菴懈･ｭ繝｡繝｢ (蜷榊燕隗｣豎ｺ API: 驥崎ｦ√す繝｣繝峨・隴ｦ蜻翫・謚大宛繧ｪ繝励す繝ｧ繝ｳ霑ｽ蜉�)
- 逶ｮ逧・
  - `todo.md` 縺ｮ縲碁㍾隕・stdlib 險伜捷 warning 謚大宛繝ｫ繝ｼ繝ｫ・郁ｨｭ螳・繝輔Λ繧ｰ・峨�阪ｒ螳溯｣・＠縲´SP/繧ｨ繝・ぅ繧ｿ騾｣謳ｺ縺ｧ蛻ｶ蠕｡蜿ｯ閭ｽ縺ｫ縺吶ｋ縲・
- 螳溯｣・
  - `nepl-web/src/lib.rs`
    - `analyze_name_resolution_with_options(source, options)` 繧定ｿｽ蜉�縲・
    - `options.warn_important_shadow`・・ool, default=true・峨ｒ蟆主・縲・
    - `NameResolutionTrace` 縺ｫ `warn_important_shadow` 繧剃ｿ晄戟縺励�（mportant-shadow warning 逕滓・繧呈擅莉ｶ蛹悶�・
    - `policy.warn_important_shadow` 繧定ｿ泌唆繝壹う繝ｭ繝ｼ繝峨↓霑ｽ蜉�縲・
    - 譌｢蟄・`analyze_name_resolution` 縺ｯ譁ｰ API 縺ｫ蟋碑ｭｲ・亥ｾ梧婿莠呈鋤邯ｭ謖・ｼ峨�・
  - `tests/tree/07_shadow_warning_policy.js`
    - 驥崎ｦ∬ｨ伜捷 `print` 縺ｯ騾壼ｸｸ warning 縺悟・繧九％縺ｨ繧堤｢ｺ隱阪�・
    - `warn_important_shadow=false` 縺ｧ warning 謚大宛縺輔ｌ繧九％縺ｨ繧堤｢ｺ隱阪�・
- 菴ｵ縺帙※螳滓命:
  - `nepl-core/src/typecheck.rs` 縺ｧ ValueNs/CallableNs 蛻・屬縺ｮ谿ｵ髫主ｰ主・繧堤ｶ咏ｶ壹＠縲∝�､逕ｨ騾斐・ lookup 繧・`lookup_value` 縺ｫ蟇・○縺溘�・
    - global `fn`/`fn alias` 譌｢蟄倩｡晉ｪ∝愛螳・
    - `set` 縺ｮ蜿ら・隗｣豎ｺ
    - dotted field base 隗｣豎ｺ
- `todo.md` 蜿肴丐:
  - 螳御ｺ・＠縺溘�碁㍾隕・stdlib 險伜捷 warning 謚大宛繝ｫ繝ｼ繝ｫ・郁ｨｭ螳・繝輔Λ繧ｰ・峨�埼�・岼繧貞炎髯､縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node tests/tree/run.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 菴懈･ｭ繝｡繝｢ (ValueNs/CallableNs 蛻・屬縺ｮ谿ｵ髫主ｰ主・: lookup 逕ｨ騾泌・髮｢)
- 逶ｮ逧・
  - `todo.md` 譛�蜆ｪ蜈医・蜷榊燕遨ｺ髢灘・髮｢縺ｫ蜷代￠縲～typecheck` 蜀・・隴伜挨蟄・lookup 繧堤畑騾泌挨 API 縺ｫ蟇・○繧九�・
- 螳溯｣・
  - `nepl-core/src/typecheck.rs` 縺ｧ縲∽ｻ･荳九・邂・園繧・value 蟆ら畑 lookup 縺ｸ鄂ｮ謠帙�・
    - 繧ｰ繝ｭ繝ｼ繝舌Ν `fn` 逋ｻ骭ｲ譎ゅ・縲梧里蟄倬撼髢｢謨ｰ繝√ぉ繝・け縲・ `env.lookup_value`
    - `fn alias` 逋ｻ骭ｲ譎ゅ・縲梧里蟄倬撼髢｢謨ｰ繝√ぉ繝・け縲・ `env.lookup_value`
    - `set` 隗｣豎ｺ譎ゅ・螟門・謗｢邏｢: `env.lookup_value`
    - dotted field (`a.b`) 縺ｮ base 隗｣豎ｺ: `env.lookup_value`
- 蜉ｹ譫・
  - 螟画焚縺ｨ callable 繧貞酔荳� lookup 縺ｧ豺ｷ蝨ｨ隗｣豎ｺ縺吶ｋ邂・園繧呈ｸ帙ｉ縺励�∝・髮｢險ｭ險医∈縺ｮ遘ｻ陦後ｒ蜑埼�ｲ縲・
  - 謖吝虚縺ｯ邯ｭ謖√＠縺､縺､縲∵э蝗ｳ縺励↑縺・callable 豺ｷ蜈･縺ｮ菴吝慍繧堤ｸｮ蟆上�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/tree -o tests/output/shadowing_tree_current.json -j 1`: `186/186 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 菴懈･ｭ繝｡繝｢ (shadow warning 繝昴Μ繧ｷ繝ｼ縺ｮ API 繝・せ繝亥崋螳・
- 逶ｮ逧・
  - `todo.md` 縺ｮ縲後す繝｣繝峨・繧､繝ｳ繧ｰ驕狗畑縺ｮ螳梧・縲阪↓蜷代￠縲～analyze_name_resolution` 縺ｮ隴ｦ蜻翫・繝ｪ繧ｷ繝ｼ繧呈惠讒矩��繝・せ繝医〒蝗ｺ螳壹�・
- 霑ｽ蜉�:
  - `tests/tree/07_shadow_warning_policy.js`
    - `print` 縺ｮ繝ｭ繝ｼ繧ｫ繝ｫ繧ｷ繝｣繝峨・縺ｧ warning 縺悟・繧九％縺ｨ繧堤｢ｺ隱阪�・
    - `cast` 縺ｮ繝ｭ繝ｼ繧ｫ繝ｫ繧ｷ繝｣繝峨・縺ｧ縺ｯ important-shadow warning 縺悟・縺ｪ縺・％縺ｨ繧堤｢ｺ隱阪�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node tests/tree/run.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `555/555 pass`

# 2026-02-21 菴懈･ｭ繝｡繝｢ (繧ｷ繝｣繝峨・繧､繝ｳ繧ｰ: callable 隗｣豎ｺ縺ｮ蝗槫ｸｰ菫ｮ豁｣)
- 閭梧勹:
  - `tests/shadowing.n.md` 縺ｮ pending 繧ｱ繝ｼ繧ｹ・・value_name_and_callable_name_can_coexist_currently_fails` / `imported_function_name_shadowed_by_parameter_currently_fails`・峨ｒ騾壼ｸｸ繝・せ繝医∈譏・�ｼ縺吶ｋ縺溘ａ縲～typecheck` 縺ｮ隴伜挨蟄占ｧ｣豎ｺ繧定ｪｿ謨ｴ縲・
- 螳溯｣・
  - `nepl-core/src/typecheck.rs` 縺ｫ `Env::lookup_callable_any` 繧定ｿｽ蜉�縲・
  - 蜻ｼ縺ｳ蜃ｺ縺励・繝・ラ菴咲ｽｮ縺ｮ隴伜挨蟄占ｧ｣豎ｺ縺ｧ縲∝酔蜷・value 縺檎樟蝨ｨ繧ｹ繧ｳ繝ｼ繝励↓縺ゅ▲縺ｦ繧・outer callable 繧貞盾辣ｧ縺ｧ縺阪ｋ邨瑚ｷｯ繧定ｿｽ蜉�縲・
  - 縺溘□縺鈴←逕ｨ遽・峇縺ｯ髯仙ｮ壹＠縲∽ｻ･荳区擅莉ｶ繧呈ｺ�縺溘☆蝣ｴ蜷医・縺ｿ譛牙柑蛹・
    - `forced_value == false`
    - `stack.is_empty()`・亥・鬆ｭ隗｣豎ｺ・・
    - `expr.items.get(idx + 1).is_some()`・亥ｮ滄圀縺ｫ蠕檎ｶ夐�・′縺ゅｊ蜻ｼ縺ｳ蜃ｺ縺玲枚閼茨ｼ・
- 螟ｱ謨怜・譫・
  - 蠖灘・縺ｯ驕ｩ逕ｨ遽・峇縺悟ｺ・☆縺弱�～if cond: ok` 縺ｮ `ok` 繧・callable 縺ｫ隱､隗｣豎ｺ縺励※蜈ｨ菴灘屓蟶ｰ・・tdlib 蛛ｴ `if condition must be bool`・峨′逋ｺ逕溘�・
  - 荳願ｨ俶擅莉ｶ縺ｧ蜻ｼ縺ｳ蜃ｺ縺励・繝・ラ縺ｫ髯仙ｮ壹＠縲∝屓蟶ｰ繧定ｧ｣豸医�・
- 繝・せ繝・
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/shadowing.n.md -o tests/output/shadowing_current.json -j 1`: `185/185 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/neplg2_current.json -j 1`: `202/202 pass`
- 陬懆ｶｳ:
  - 蜈ｱ譛峨＆繧後※縺・◆ `tests/neplg2.n.md::doctest#6/#7` 縺ｮ compile fail 縺ｯ迴ｾ譎らせ縺ｧ蜀咲樟縺帙★縲∝ｽ楢ｩｲ繝輔ぃ繧､繝ｫ縺ｯ蜈ｨ莉ｶ pass縲・

# 2026-02-21 菴懈･ｭ繝｡繝｢ (target=wasm 縺ｧ WASI 辟｡蜉ｹ蛹・
- 隕∽ｻｶ蜿肴丐:
  - `nepl-cli/src/main.rs` 縺ｮ閾ｪ蜍墓・譬ｼ繝ｭ繧ｸ繝・け・・std/stdio` import 繧呈､懷・縺励※ `wasi` 縺ｫ縺吶ｋ謖吝虚・峨ｒ蜑企勁縲・
  - `target=wasm` 縺ｮ縺ｨ縺阪・ WASI 繧呈怏蜉ｹ蛹悶＠縺ｪ縺・ｈ縺・↓菫ｮ豁｣縲・
  - `target=wasi` 縺ｮ縺ｨ縺阪・縺ｿ `wasi_snapshot_preview1` import 繧定ｨｱ蜿ｯ縺励�仝ASI 髢｢謨ｰ繧・linker 縺ｫ逋ｻ骭ｲ縲・
- 螳溯｣・ｩｳ邏ｰ:
  - `execute`:
    - `target_override` 繧・CLI 謖・ｮ壹・縺ｿ縺ｫ髯仙ｮ壹�・
    - 螳溯｡後ち繝ｼ繧ｲ繝・ヨ謗ｨ螳壹ｒ `detect_module_target` 縺ｸ蛻・ｊ蜃ｺ縺暦ｼ・module.directives` 縺ｨ `module.root.items` 縺ｮ蜿梧婿繧堤｢ｺ隱搾ｼ峨�・
  - `run_wasm`:
    - `CompileTarget::Wasm` 縺ｧ縺ｯ import 縺悟ｭ伜惠縺励◆譎らせ縺ｧ繧ｨ繝ｩ繝ｼ蛹悶�・
    - `CompileTarget::Wasi` 縺ｧ縺ｮ縺ｿ `args_sizes_get` / `args_get` / `path_open` / `fd_read` / `fd_close` / `fd_write` 繧堤匳骭ｲ縲・
- 讀懆ｨｼ:
  - `cargo test -p nepl-cli`: pass
  - `#target wasm + #import "std/stdio"`: compile error・・WASI import not allowed for wasm target`・峨ｒ遒ｺ隱阪�・
  - `#target wasi + #import "std/stdio"`: 螳溯｡梧・蜉滂ｼ・println "hi"` 縺悟・蜉幢ｼ峨ｒ遒ｺ隱阪�・

# 2026-02-21 菴懈･ｭ繝｡繝｢ (fs 陦晉ｪ∽ｿｮ豁｣ + 蝗槫ｸｰ繝・せ繝郁ｿｽ蜉�)
- `tests/selfhost_req.n.md` 縺ｮ compile fail 繧定ｵｷ轤ｹ縺ｫ `std/fs` 縺ｮ譬ｹ蝗�繧剃ｿｮ豁｣縲・
  - `std/fs` 縺ｮ WASI extern 蜷阪′莉悶Δ繧ｸ繝･繝ｼ繝ｫ・・std/stdio` 縺ｪ縺ｩ・峨→陦晉ｪ√＠縺・ｋ縺溘ａ縲～wasi_path_open` / `wasi_fd_read` / `wasi_fd_close` 縺ｫ蜀・Κ蜷阪ｒ蝗ｺ譛牙喧縲・
  - `fs_read_fd_bytes` 縺ｮ `cast` 繧・`<u8> cast b` 縺ｸ譏守､ｺ縺励※ overload 譖匁乂諤ｧ繧定ｧ｣豸医�・
  - `vec_new<u8> ()` 譌ｧ險俶ｳ輔ｒ譁ｰ險俶ｳ・`vec_new<u8>` 縺ｸ譖ｴ譁ｰ縲・
- 繝・せ繝域紛蛯・
  - 霑ｽ蜉�: `tests/capacity_stack.n.md`
    - 蜀榊ｸｰ豺ｱ縺包ｼ・4/512・峨�～Vec` 諡｡蠑ｵ縲～mem` 隱ｭ縺ｿ譖ｸ縺阪�～StringBuilder`縲～enum+vec+蜀榊ｸｰ` 縺ｮ谿ｵ髫弱ユ繧ｹ繝医ｒ蝗ｺ螳壹�・
  - 譖ｴ譁ｰ:
    - `tests/selfhost_req.n.md`
    - `tests/sort.n.md`
    - `tests/string.n.md`
    - `tests/ret_f64_example.n.md`
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/ret_f64_example.n.md -i tests/selfhost_req.n.md -i tests/sort.n.md -i tests/string.n.md -i tests/capacity_stack.n.md -o tests/output/targeted_regression_current.json`
    - `194/194 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json`
    - `540/540 pass`
- 陬懆ｶｳ:
  - `std/fs` 縺ｯ蠑輔″邯壹″ WASI preview1 蜑肴署縲Ａwasmtime/wasmer` 蟾ｮ蛻・､懆ｨｼ縺ｯ `todo_kp.md` 縺ｮ繝ｩ繝ｳ繧ｿ繧､繝�莠呈鋤鬆・岼縺ｨ縺励※邯咏ｶ壹�・

# 迥ｶ豕√Γ繝｢ (2026-01-22)
# 2026-02-10 菴懈･ｭ繝｡繝｢ (遶ｶ繝励Ο繧ｫ繧ｿ繝ｭ繧ｰ諡｡蠑ｵ + kp繝｢繧ｸ繝･繝ｼ繝ｫ謨ｴ逅・
- 繝√Η繝ｼ繝医Μ繧｢繝ｫ縺ｫ遶ｶ繝励Ο螳夂分縺ｮ蜿ら・遶�繧定ｿｽ蜉�縺励�・㍾隕√い繝ｫ繧ｴ繝ｪ繧ｺ繝�/繝・・繧ｿ讒矩��縺ｮ繧ｵ繝ｳ繝励Ν繧・20 鬆・岼縺ｧ蛻玲嫌縺励◆縲・
  - 霑ｽ蜉�: `tutorials/getting_started/27_competitive_algorithms_catalog.n.md`
  - 逶ｮ谺｡蜿肴丐: `tutorials/getting_started/00_index.n.md`
- `stdlib/kp` 繧呈ｩ溯・蛻･縺ｫ謨ｴ逅・＠縲∵眠隕上Δ繧ｸ繝･繝ｼ繝ｫ繧定ｿｽ蜉�縺励◆縲・
  - `stdlib/kp/kpsearch.nepl`
    - `lower_bound_i32`, `upper_bound_i32`, `contains_i32`
  - `stdlib/kp/kpprefix.nepl`
    - `prefix_build_i32`, `prefix_range_sum_i32`
  - `stdlib/kp/kpdsu.nepl`
    - `dsu_new`, `dsu_find`, `dsu_unite`, `dsu_same`, `dsu_size`, `dsu_free`
  - `stdlib/kp/kpfenwick.nepl`
    - `fenwick_new`, `fenwick_add`, `fenwick_sum_prefix`, `fenwick_sum_range`, `fenwick_free`
- 縺吶∋縺ｦ `//:` 縺ｮ繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝亥ｽ｢蠑上〒險倩ｿｰ縺励�∝推繝｢繧ｸ繝･繝ｼ繝ｫ縺ｫ譛�蟆・doctest 繧剃ｻ倅ｸ弱＠縺溘�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (髢｢謨ｰ蜊倅ｽ阪Ξ繝薙Η繝ｼ: 讖滓｢ｰ鄂ｮ謠帙・蠕悟・逅・
- 繝ｦ繝ｼ繧ｶ繝ｼ謖・､ｺ縺ｫ蝓ｺ縺･縺阪�～vec/stack/list` 繧帝未謨ｰ縺斐→縺ｫ蜀咲｢ｺ隱阪＠縲∵ｩ滓｢ｰ鄂ｮ謠帷罰譚･縺ｮ荳肴紛蜷医ｒ謇倶ｿｮ豁｣縺励◆縲・
- 荳ｻ縺ｪ菫ｮ豁｣:
  - `stdlib/alloc/vec.nepl`
    - `vec_new` 繝峨く繝･繝｡繝ｳ繝医・ `菴ｿ縺・婿:` 驥崎､・ｒ髯､蜴ｻ縲・
    - `vec_set` doctest 縺ｮ move-check 陦晉ｪ√ｒ蝗樣∩縺吶ｋ菴ｿ逕ｨ萓九∈菫ｮ豁｣縲・
  - `stdlib/alloc/collections/stack.nepl`
    - 繝｢繧ｸ繝･繝ｼ繝ｫ隱ｬ譏弱・驥崎､・ヶ繝ｭ繝・け・亥・鬆ｭ縺ｨ import 蠕後・莠碁㍾險倩ｼ会ｼ峨ｒ邨ｱ蜷医＠縲・邂・園縺ｫ謨ｴ逅・�・
  - `stdlib/alloc/collections/list.nepl`
    - 繝｢繧ｸ繝･繝ｼ繝ｫ隱ｬ譏弱・驥崎､・ヶ繝ｭ繝・け・亥・鬆ｭ縺ｨ import 蠕後・莠碁㍾險倩ｼ会ｼ峨ｒ邨ｱ蜷医＠縲・邂・園縺ｫ謨ｴ逅・�・
- 蠖｢蠑城擇:
  - `//` 繧ｳ繝｡繝ｳ繝医・谿九＆縺壹�√ラ繧ｭ繝･繝｡繝ｳ繝医・ `//:` 縺ｮ縺ｿ繧剃ｽｿ逕ｨ縲・
  - 蜷・未謨ｰ縺ｫ `逶ｮ逧・螳溯｣・豕ｨ諢・險育ｮ鈴㍼` + `菴ｿ縺・婿` + `neplg2:test` 繧堤ｶｭ謖√�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-vec-stack-list.json -j 1 --no-stdlib`
  - `summary: total=35, passed=35, failed=0, errored=0`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (doc comment 譖ｸ蠑・ 縲御ｽｿ縺・婿縲崎ｦ句・縺励ｒ邨ｱ荳�)
- 繝ｦ繝ｼ繧ｶ繝ｼ謠千､ｺ縺ｮ譖ｸ蠑上↓蜷医ｏ縺帙�～vec/stack/list` 縺ｮ doctest 蜑阪↓ `//: 菴ｿ縺・婿:` 繧堤ｵｱ荳�霑ｽ蜉�縺励◆縲・
  - 蟇ｾ雎｡:
    - `stdlib/alloc/vec.nepl`
    - `stdlib/alloc/collections/stack.nepl`
    - `stdlib/alloc/collections/list.nepl`
- 縺ゅｏ縺帙※縲～vec_set` 縺ｮ doctest 縺ｧ move-check 縺ｫ謚ｵ隗ｦ縺励※縺・◆萓九ｒ菫ｮ豁｣縺励�√さ繝ｳ繝代う繝ｫ蜿ｯ閭ｽ縺ｪ菴ｿ逕ｨ萓九↓謨ｴ縺医◆縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-vec-stack-list.json -j 1 --no-stdlib`
  - `summary: total=35, passed=35, failed=0, errored=0`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (vec/stack/list 繧ｳ繝｡繝ｳ繝域ｧ伜ｼ上・謖・ｮ壼ｯｾ蠢・
- 繝ｦ繝ｼ繧ｶ繝ｼ謖・ｮ壹・ `stdlib/nm` 諡｡蠑ｵ Markdown 蠖｢蠑上↓蜷医ｏ縺帙�∽ｻ･荳九・繝｢繧ｸ繝･繝ｼ繝ｫ蜈磯�ｭ繧ｳ繝｡繝ｳ繝医ｒ蜈ｷ菴灘喧縺励◆縲・
  - `stdlib/alloc/vec.nepl`
  - `stdlib/alloc/collections/stack.nepl`
  - `stdlib/alloc/collections/list.nepl`
- 蜿肴丐蜀・ｮｹ:
  - 蜈磯�ｭ `//:` 縺ｧ縲後Λ繧､繝悶Λ繝ｪ縺ｮ荳ｻ鬘後�阪�檎岼逧・�阪�悟ｮ溯｣・い繝ｫ繧ｴ繝ｪ繧ｺ繝�縲阪�梧ｳｨ諢冗せ縲阪�瑚ｨ育ｮ鈴㍼縲阪ｒ蜈ｷ菴楢ｨ倩ｿｰ縲・
  - 譌｢蟄倥・蜷・未謨ｰ蜑・`//:`・育岼逧・螳溯｣・豕ｨ諢・險育ｮ鈴㍼・峨→ doctest 讒区・縺ｯ邯ｭ謖√�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-vec-stack-list.json -j 1 --no-stdlib`
  - `summary: total=7, passed=7, failed=0, errored=0`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (vec/stack/list 縺ｮ doc comment + doctest 謨ｴ蛯・
- 繝ｦ繝ｼ繧ｶ繝ｼ謖・､ｺ縺ｫ蜷医ｏ縺帙※縲∽ｻ･荳九・讓呎ｺ悶Λ繧､繝悶Λ繝ｪ縺ｫ螳溯｡悟庄閭ｽ縺ｪ doctest 繧定ｿｽ蜉�繝ｻ謨ｴ蛯吶＠縺溘�・
  - `stdlib/alloc/vec.nepl`
  - `stdlib/alloc/collections/stack.nepl`
  - `stdlib/alloc/collections/list.nepl`
- 螟画峩蜀・ｮｹ:
  - `stack.nepl` / `list.nepl` 縺ｮ `neplg2:test[skip]` 繧定ｧ｣髯､縺励�∽ｸｻ隕∵桃菴懶ｼ・ew/push/pop/peek/len/clear, cons/head/tail/get/reverse 縺ｪ縺ｩ・峨ｒ遒ｺ隱阪☆繧・doctest 繧定ｿｽ蜉�縲・
  - `vec.nepl` 縺ｫ `clear` 繧剃ｸｭ蠢・→縺励◆霑ｽ蜉� doctest 繧貞・繧後�［ove 隕丞援縺ｫ蜿阪＠縺ｪ縺・ｽ｢縺ｸ隱ｿ謨ｴ縲・
  - `str_eq` 繧剃ｽｿ縺・doctest 縺ｫ縺ｯ `alloc/string` import 繧呈・遉ｺ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-vec-stack-list.json -j 1 --no-stdlib`
    - `summary: total=7, passed=7, failed=0, errored=0`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (nm OOB 譬ｹ豐ｻ: parse_markdown 蜀崎ｨｭ險・
- `nm` 縺ｮ run fail (`memory access out of bounds`) 繧剃ｸ頑ｵ√°繧牙・蛻・ｊ蛻・￠縺励�～stdlib/nm/parser.nepl` 縺ｮ `parse_markdown` 繧貞・險ｭ險医＠縺溘�・
- 譬ｹ蝗�蛻・梵:
  - 譌｢蟄伜ｮ溯｣・・ section stack 縺ｨ `Vec<Node>` 縺ｮ蛟､蜿励￠貂｡縺励′隍・尅縺ｧ縲～nm` doctest 縺ｧ OOB 繧堤ｶ咏ｶ壼・迴ｾ縲・
  - `parse_markdown` 蜊倅ｽ薙・譛�蟆丞ｮ溯｡後〒蜀咲樟縺吶ｋ縺薙→繧堤｢ｺ隱阪＠縲∝捉霎ｺ繝ｭ繧ｸ繝・け繧呈ｮｵ髫守噪縺ｫ螟悶＠縺ｦ蛻・ｊ蛻・￠縲・
- 螳溯｣・､画峩:
  - `parse_markdown` 繧偵ヵ繝ｩ繝・ヨ襍ｰ譟ｻ繝吶・繧ｹ縺ｫ鄂ｮ縺肴鋤縺医�～stack` 萓晏ｭ倡ｵ瑚ｷｯ繧帝勁蜴ｻ縲・
  - `safe_line` 縺ｯ `lines_data + offset` 縺ｧ縺ｯ縺ｪ縺・`vec_get<str>` 繝吶・繧ｹ縺ｮ螳牙・繧｢繧ｯ繧ｻ繧ｹ縺ｫ邨ｱ荳�縲・
  - heading/fence/paragraph/hr 縺ｮ蛻・ｲ舌ｒ譏守､ｺ蛹悶＠縲∬ｦ句・縺鈴・荳九・ children 蜿朱寔繧貞ｱ�謇�繝ｫ繝ｼ繝励〒螳溯｣・�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/nm.n.md -o /tmp/tests-nm.json -j 1`
    - `total=72, passed=72, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all.json -j 1`
    - `total=416, passed=409, failed=7, errored=0`
    - 谿九ｊ縺ｯ `ret_f64_example`, `selfhost_req`, `sort` 縺ｧ縲］m 邉ｻ螟ｱ謨励・隗｣豸医�・
# 2026-02-10 菴懈･ｭ繝｡繝｢ (nm 螳溯｣・憾豕√→ doc comment 謨ｴ蛯・
- `nm` 縺ｮ迴ｾ迥ｶ:
  - 繧ｳ繝ｳ繝代う繝ｫ谿ｵ髫弱・荳ｻ隕・move-check 繧ｨ繝ｩ繝ｼ縺ｯ螟ｧ縺阪￥蜑頑ｸ帙＠縺溘′縲∝ｮ溯｡梧凾 `memory access out of bounds` 縺梧ｮ九▲縺ｦ縺翫ｊ譛ｪ螳御ｺ・�・
  - `tests/nm.n.md` 縺ｮ螟ｱ謨励・迴ｾ蝨ｨ OOB 縺ｮ縺ｿ・・ompile fail 縺九ｉ run fail 縺ｸ驕ｷ遘ｻ・峨�・
- 繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝域紛蛯・
  - `stdlib/nm/parser.nepl`
    - `parse_markdown`
    - `document_to_json`
  - `stdlib/nm/html_gen.nepl`
    - `render_document`
  - 荳願ｨ倥↓譌･譛ｬ隱櫁ｪｬ譏趣ｼ育岼逧・螳溯｣・豕ｨ諢・險育ｮ鈴㍼・峨→ `neplg2:test` 萓九ｒ霑ｽ蜉�縲・
  - doctest 萓九・ `fn main` 繧貞性繧�螳溯｡悟庄閭ｽ縺ｪ蠖｢蠑上∈菫ｮ豁｣貂医∩縲・
- 繝・せ繝育ｵ先棡・・m 髢｢騾｣・・
  - `node nodesrc/tests.js -i tests/nm.n.md -o /tmp/tests-nm.json -j 1`
  - `summary: total=72, passed=67, failed=5, errored=0`
  - 螟ｱ謨礼炊逕ｱ縺ｯ縺吶∋縺ｦ `memory access out of bounds`
- 谺｡繧｢繧ｯ繧ｷ繝ｧ繝ｳ:
  - OOB 縺ｮ逋ｺ逕溽せ繧・`nm/parser` 縺ｮ `load<...>` / `size_of<...>` 蛻ｩ逕ｨ邂・園縺九ｉ蜀榊・繧雁・縺代�・
  - `Vec<T>` 隕∫ｴ�繧｢繧ｯ繧ｻ繧ｹ繧堤峩謗･ `data + offset` 縺ｧ謇ｱ縺・婿驥昴・螳牙・譚｡莉ｶ・亥｢・阜繝ｻ繝ｬ繧､繧｢繧ｦ繝茨ｼ峨ｒ譏取枚蛹悶＠縲∝ｿ・ｦ√↑繧・API 縺ｫ謌ｻ縺吶�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (nm 蜀咲樟繝・せ繝郁ｿｽ蜉�縺ｨ荳頑ｵ∝・繧雁・縺・
- `tests/nm.n.md` 繧呈眠隕剰ｿｽ蜉�縺励�～nm/parser` + `nm/html_gen` 縺ｮ譛�蟆冗ｵ瑚ｷｯ繧貞崋螳壹＠縺溘�・
  - `nm_parse_markdown_json_basic`
  - `nm_render_document_basic`
- `examples/nm.nepl` / `stdlib/nm/parser.nepl` 縺ｮ蜈郁｡御ｿｮ豁｣:
  - `stdlib/nm/parser.nepl` 縺ｮ `if:` 繝ｬ繧､繧｢繧ｦ繝育罰譚･縺ｧ parser 蜀榊ｸｰ繧定ｪ倡匱縺励※縺・◆ `let next_is_paren` 驛ｨ蛻・ｒ谿ｵ髫惹ｻ｣蜈･縺ｸ螟画峩縲・
  - `#import "std/math"` 繧・`#import "core/math"` 縺ｫ菫ｮ豁｣縲・
  - `examples/nm.nepl` 縺ｫ `#import "std/env/cliarg" as *` 繧定ｿｽ蜉�縲・
- `nm` 縺ｧ髴ｲ蜃ｺ縺励◆荳頑ｵ∽ｸ肴紛蜷医・菫ｮ豁｣:
  - `nm/parser` / `nm/html_gen` 縺ｮ髢｢謨ｰ繧ｷ繧ｰ繝阪メ繝｣繧貞ｮ溯｣・ｮ滓・縺ｫ蜷医ｏ縺帙※ `*>` 縺ｸ蟇・○縺滂ｼ・ure/impure 荳肴紛蜷医・隗｣豸茨ｼ峨�・
  - `nm/parser` 蜀・・ bool 豈碑ｼ・(`eq done false` 遲・ 繧・`not` / 逶ｴ謗･蛻､螳壹∈螟画峩縲・
  - `Section` 讒狗ｯ画凾縺ｮ譖匁乂縺ｪ蜑咲ｽｮ蠑上ｒ谿ｵ髫惹ｻ｣蜈･縺ｸ謨ｴ逅・＠縲∬ｦｪ諠・�ｱ蜿門ｾ鈴�・ｺ上ｒ `peek -> pop` 縺ｫ菫ｮ豁｣縲・
  - 蝙句錐陦晉ｪ√ｒ隗｣豸・
    - `Section`(struct) -> `NestSection`
    - `Ruby`(struct) -> `RubyInfo`
    - `Gloss`(struct) -> `GlossInfo`
    - `CodeBlock`(struct) -> `CodeBlockInfo`
- 讀懆ｨｼ:
  - `NO_COLOR=true trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/nm.n.md -o /tmp/tests-nm.json -j 1`
    - `total=69, passed=67, failed=2`
    - 谿九ｊ: `use of moved value`・・lines` / `v`・峨↓蜿取據
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-nm.json -j 1`
    - `total=413, passed=404, failed=9, errored=0`
- 迴ｾ蝨ｨ縺ｮ隧穂ｾ｡:
  - parser 縺ｮ蛛懈ｭ｢菫晁ｨｼ縺ｯ邯ｭ謖√＆繧後◆縺ｾ縺ｾ縲］m 荳榊・蜷医・縲祁ec/str 縺ｮ謇�譛画ｨｩ蜃ｦ逅・ｼ・ec_get/vec_len 蜻ｼ縺ｳ蜃ｺ縺苓ｨｭ險茨ｼ峨�阪∈譬ｹ蝗�縺檎ｵ槭ｌ縺溘�・
  - 谺｡谿ｵ縺ｯ `nm/parser` 縺ｮ繝ｫ繝ｼ繝怜・逅・ｒ `Vec` 縺ｮ `data/len` 逶ｴ謗･繧｢繧ｯ繧ｻ繧ｹ縺ｸ蜀崎ｨｭ險医＠縲［ove-check 繧呈�ｹ譛ｬ隗｣豸医☆繧九�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (parser 蜀榊ｸｰ證ｴ襍ｰ縺ｮ蛛懈ｭ｢菫晁ｨｼ)
- 繝ｦ繝ｼ繧ｶ繝ｼ謖・､ｺ縲後さ繝ｳ繝代う繝ｩ縺ｯ蠢・★蛛懈ｭ｢縺吶ｋ縲阪ｒ蜿励￠縺ｦ縲～nepl-core/src/parser.rs` 縺ｫ蛛懈ｭ｢菫晁ｨｼ繧定ｿｽ蜉�縲・
- 螳溯｣・・螳ｹ・井ｸ頑ｵ・parser 蛛ｴ・・
  - 蜀榊ｸｰ豺ｱ縺穂ｸ企剞繧定ｿｽ蜉�:
    - `MAX_PARSE_RECURSION_DEPTH = 2048`
    - `enter_parse_context` / `leave_parse_context` 繧定ｿｽ蜉�
    - `parse_stmt` 繧偵さ繝ｳ繝・く繧ｹ繝育ｮ｡逅・ｸ九〒螳溯｡後＠縲・℃蜑ｰ蜀榊ｸｰ譎ゅ・險ｺ譁ｭ繧定ｿ斐＠縺ｦ蛛懈ｭ｢縺吶ｋ繧医≧螟画峩
  - 辟｡騾ｲ謐励Ν繝ｼ繝玲､懷・繧定ｿｽ蜉�:
    - `MAX_NO_PROGRESS_STEPS = 64`
    - `parse_block_until_internal` / `parse_prefix_expr` / `parse_prefix_expr_until_tuple_delim` / `parse_prefix_expr_until_colon`
    - 蜷御ｸ� `pos` 縺御ｸ�螳壼屓謨ｰ邯壹＞縺溘ｉ險ｺ譁ｭ繧貞・縺励※ 1 token 蜑埼�ｲ縺励�∫┌髯舌Ν繝ｼ繝励ｒ蝗樣∩
- 讀懆ｨｼ:
  - `NO_COLOR=true trunk build`: 謌仙粥
  - `timeout 20s node nodesrc/analyze_source.js -i stdlib/nm/parser.nepl --stage parse`: `PARSE_EXIT:0`
  - `node nodesrc/test_analysis_api.js`: `7/7 passed`
- 陬懆ｶｳ:
  - `stdlib/nm/parser.nepl` 縺ｮ parse 縺ｧ莉･蜑咲匱逕溘＠縺ｦ縺・◆蛛懈ｭ｢縺励↑縺・嫌蜍輔・縲∝ｰ代↑縺上→繧りｧ｣譫・API 邨瑚ｷｯ縺ｧ縺ｯ蜀咲樟縺励↑縺上↑縺｣縺溘�・
  - `examples/nm.nepl` 蛛ｴ縺ｯ蠑輔″邯壹″ type/effect 荳肴紛蜷茨ｼ・nm` 繝ｩ繧､繝悶Λ繝ｪ縺ｮ pure/impure 鄂ｲ蜷阪ぜ繝ｬ遲会ｼ峨′谿九▲縺ｦ縺翫ｊ縲∵ｬ｡谿ｵ縺ｧ菫ｮ豁｣邯咏ｶ壹�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (tuple unit 隕∫ｴ�縺ｮ codegen 譬ｹ譛ｬ菫ｮ豁｣)
- `tests/tuple_new_syntax.n.md::doctest#10` 縺ｮ譬ｹ蝗�繧堤音螳壹�・
  - `Tuple:` 縺ｫ `()` 縺悟性縺ｾ繧後ｋ縺ｨ縲仝ASM codegen 縺・`unit` 隕∫ｴ�繧帝�壼ｸｸ蛟､縺ｨ縺励※ `LocalSet` 縺励ｈ縺・→縺励※繧ｹ繧ｿ繝・け荳崎ｶｳ縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
  - 譌｢蟄倥Ξ繧､繧｢繧ｦ繝茨ｼ・ypecheck 蛛ｴ offset=4 蛻ｻ縺ｿ・峨ｒ蟠ｩ縺輔★縲～unit` 隕∫ｴ�/繝輔ぅ繝ｼ繝ｫ繝峨・縲悟ｼ剰ｩ穂ｾ｡縺ｧ蜑ｯ菴懃畑縺ｯ螳溯｡後＠縺､縺､縲√せ繝ｭ繝・ヨ縺ｫ縺ｯ 0 繧呈�ｼ邏阪�阪☆繧区婿驥昴∈邨ｱ荳�縲・
- `nepl-core/src/codegen_wasm.rs`:
  - `StructConstruct` / `TupleConstruct` 縺ｮ隕∫ｴ� store 蛻・ｲ舌ｒ `valtype(Some)` 縺ｨ `None(unit)` 縺ｧ蛻・屬縲・
  - `None(unit)` 縺ｧ縺ｯ `gen_expr` 蠕後↓ `i32.store 0` 繧定｡後≧螳溯｣・∈螟画峩縲・
- 讀懆ｨｼ:
  - `NO_COLOR=true trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/tuple_new_syntax.n.md -o /tmp/tests-tuple-after-unit-slot-fix.json -j 1`
    - `total=20, passed=20, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-tuple-unit-fix.json -j 1`
    - `total=339, passed=327, failed=12, errored=0`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (pipe 谿倶ｻｶ隗｣豸・+ alloc 萓晏ｭ倥・譬ｹ譛ｬ謾ｹ蝟・
- `tests/pipe_operator.n.md` 縺ｮ谿句､ｱ謨暦ｼ・13/#14/#15・峨ｒ荳頑ｵ√°繧牙・繧雁・縺代※菫ｮ豁｣縲・
- `nepl-core/src/typecheck.rs`:
  - `let s <S> 10 |> S` / `let e <E> 20 |> E::V` 縺ｧ縲～<S>/<E>` 縺・pipe 蜑阪・繝ｪ繝・Λ繝ｫ縺ｫ譌ｩ譛滄←逕ｨ縺輔ｌ繧倶ｸ榊・蜷医ｒ菫ｮ豁｣縲・
  - `next_is_pipe` 縺ｮ蝣ｴ蜷医・ pending ascription 繧帝≦蟒ｶ縺励�｝ipe 豕ｨ蜈･蠕後・蠑冗｢ｺ螳壽凾縺ｫ驕ｩ逕ｨ縺吶ｋ繧医≧螟画峩縲・
- `nepl-core/src/codegen_wasm.rs`:
  - `alloc` 縺梧悴import縺ｧ繧よｧ矩��菴・蛻玲嫌/繧ｿ繝励Ν讒狗ｯ峨〒關ｽ縺｡縺ｪ縺・ｈ縺・�（nline bump allocator 繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ繧定ｿｽ蜉�・・emit_alloc_call`/`emit_inline_alloc`・峨�・
  - 縺薙ｌ縺ｫ繧医ｊ `pipe_struct_source` / `pipe_into_constructor` 縺ｧ蜃ｺ縺ｦ縺・◆ `alloc function not found (import std/mem)` 繧定ｧ｣豸医�・
- `todo.md`:
  - 鬮倬嚴髢｢謨ｰ繝輔ぉ繝ｼ繧ｺ蠕後・ `StringBuilder` 譬ｹ譛ｬ蜀崎ｨｭ險医ち繧ｹ繧ｯ・・(n) build 蛹悶�∝・迴ｾ繝・せ繝郁ｿｽ蜉�・峨ｒ霑ｽ蜉�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=true trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/pipe_operator.n.md -o /tmp/tests-pipe-after-constructor-revert.json -j 1`
    - `total=20, passed=20, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-current-after-pipe-fixes.json -j 1`
    - `total=339, passed=326, failed=13, errored=0`
  - 谿倶ｻｶ蛻・｡・
    - `ret_f64_example=1`
    - `selfhost_req=4`
    - `sort=5`
    - `string=2`
    - `tuple_new_syntax=1`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (offside: block: 蜷御ｸ�陦檎ｶ咏ｶ壹・遖∵ｭ｢)
- `tests/offside_and_indent_errors.n.md::doctest#4` 縺ｮ譬ｹ蝗�縺ｯ parser 縺・`block:` 縺ｮ蜷御ｸ�陦檎ｶ咏ｶ夲ｼ・block: add 1 2`・峨ｒ險ｱ螳ｹ縺励※縺・◆縺薙→縲・
- `nepl-core/src/parser.rs` 繧剃ｿｮ豁｣:
  - `KwBlock` 縺ｮ `:` 蛻・ｲ舌〒縲∵隼陦後′辟｡縺・�ｴ蜷医・險ｺ譁ｭ繧定ｿｽ蜉�縺励�∝屓蠕ｩ逕ｨ縺ｫ蜊倩｡瑚ｧ｣譫舌∈繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ縲・
  - 莉墓ｧ倅ｸ翫�形block:` 縺ｮ蠕後ｍ縺ｯ遨ｺ逋ｽ/繧ｳ繝｡繝ｳ繝医・縺ｿ縲阪ｒ貅�縺溘☆繧医≧縺ｫ縺励◆縲・
- 讀懆ｨｼ:
  - `NO_COLOR=true trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/offside_and_indent_errors.n.md -o /tmp/tests-offside-after-block-colon-fix.json -j 1`
    - `total=7, passed=7, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-offside-fix.json -j 1`
    - `total=339, passed=322, failed=17, errored=0`
  - 谿九ｊ螟ｱ謨怜・鬘・
    - `pipe_operator=4`
    - `ret_f64_example=1`
    - `selfhost_req=4`
    - `sort=5`
    - `string=2`
    - `tuple_new_syntax=1`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (target蟆企㍾ + trait蜻ｼ縺ｳ蜃ｺ縺・+ doctest VFS)
- `nepl-web/src/lib.rs`:
  - `compile_wasm_with_entry` 縺ｮ `CompileOptions.target` 繧・`Some(Wasi)` 蝗ｺ螳壹°繧・`None` 縺ｫ螟画峩縺励�√た繝ｼ繧ｹ蛛ｴ `#target` 繧貞ｰ企㍾縺吶ｋ繧医≧菫ｮ豁｣縲・
  - 縺薙ｌ縺ｫ繧医ｊ `#if[target=...]` / `#target` 驥崎､・､懷・ / wasm 縺ｧ縺ｮ wasi import 遖∵ｭ｢縺ｮ繝・せ繝医′譛牙柑蛹悶＆繧後◆縲・
- `nepl-core/src/monomorphize.rs`:
  - `FuncRef::Trait` 縺ｮ隗｣豎ｺ縺ｧ impl map 縺ｮ蜴ｳ蟇・ｸ�閾ｴ縺悟､悶ｌ縺溷�ｴ蜷医↓縲～trait+method` 縺ｧ縺ｮ蝙句腰荳�蛟呵｣懊ｒ謗｢邏｢縺吶ｋ繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ繧定ｿｽ蜉�縲・
  - `tests/neplg2.n.md::doctest#31` (`Show::show`) 繧定ｧ｣豸医�・
- `nodesrc/run_test.js` + `nodesrc/tests.js`:
  - doctest 螳溯｡梧凾縺ｫ `file` 諠・�ｱ繧呈ｸ｡縺励�～#import`/`#include` 縺ｮ逶ｸ蟇ｾ繝代せ繧貞ｮ溘ヵ繧｡繧､繝ｫ縺九ｉ蜿朱寔縺励※ `compile_source_with_vfs` 縺ｫ貂｡縺呎ｩ溯・繧定ｿｽ蜉�縲・
  - `tests/part.nepl` 繧定ｿｽ蜉�縺励�～tests/neplg2.n.md::doctest#11` 縺ｮ `#import "./part"` 繧定ｧ｣豎ｺ蜿ｯ閭ｽ縺ｫ縺励◆縲・
- 讀懆ｨｼ:
  - `NO_COLOR=true trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o /tmp/tests-neplg2-after-vfs2.json -j 1`
    - `total=35, passed=35, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-target-vfs-trait.json -j 1`
    - `total=339, passed=321, failed=18, errored=0`
  - 荳ｻ縺ｪ谿倶ｻｶ: `offside(1)`, `pipe_operator(4)`, `ret_f64_example(1)`, `selfhost_req(4)`, `sort(5)`, `string(2)`, `tuple_new_syntax(1)`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (loader蟄怜唱豁｣隕丞喧 + 鬮倬嚴髢｢謨ｰ蝗槫ｸｰ遒ｺ隱・
- `nepl-core/src/loader.rs` 縺ｮ `canonicalize_path` 縺ｫ蟄怜唱逧・ｭ｣隕丞喧・・.` / `..` 髯､蜴ｻ・峨ｒ霑ｽ蜉�縺励◆縲・
  - 逶ｮ逧・ `#import "./part"` 縺ｮ隗｣豎ｺ縺ｧ `/virtual/./part.nepl` 縺ｨ `/virtual/part.nepl` 縺ｮ荳堺ｸ�閾ｴ繧偵↑縺上☆縺溘ａ縲・
  - 螟画峩蠕後�～tests/neplg2.n.md::doctest#11` 縺ｯ `missing source: /virtual/part.nepl` 縺ｾ縺ｧ蜑埼�ｲ縺励�√ヱ繧ｹ荳堺ｸ�閾ｴ閾ｪ菴薙・隗｣豸医�・
- 鬮倬嚴髢｢謨ｰ邉ｻ縺ｮ迴ｾ迥ｶ繧貞・遒ｺ隱・
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-current.json -j 1`
  - `total=19, passed=19, failed=0, errored=0`
  - 逶ｴ霑代・ `functions` 螟ｱ謨励・隗｣豸域ｸ医∩縲・
- 蜈ｨ菴灘屓蟶ｰ:
  - `NO_COLOR=true trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-outer-consumer-fix.json -j 1`
  - `total=339, passed=315, failed=24, errored=0`・域里遏･髮・粋・・
- 谿玖ｪｲ鬘後Γ繝｢:
  - `neplg2#doctest#11` 縺ｯ loader 縺ｧ縺ｯ縺ｪ縺・doctest harness 蛛ｴ縺ｮ隍・焚繝輔ぃ繧､繝ｫ萓帷ｵｦ莉墓ｧ假ｼ・FS・画悴謨ｴ蛯吶′譬ｹ蝗�縲・
  - 縺ｻ縺九・螟ｱ謨嶺ｸｻ蝪翫・ `sort` / `selfhost_req` / `pipe_operator` / `tuple_new_syntax`縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (functions if螟ｱ謨励・蜀咲樟繝√ぉ繝・け貅門ｙ)
- `functions#doctest#7/#10` 縺ｮ蜴溷屏蛻・ｊ蛻・￠縺ｮ縺溘ａ縲～typecheck` 縺ｮ call reduction 蜻ｨ霎ｺ繧定ｪｿ譟ｻ縲・
- 荳�譎ら噪縺ｫ `reduce_calls` 縺ｮ蛟呵｣懈爾邏｢譁ｹ蠑上ｒ螟画峩縺励◆縺後�～tests/if.n.md` 縺梧が蛹厄ｼ・ fail・峨＠縺溘◆繧∝叙繧頑ｶ医＠貂医∩縲・
- 迴ｾ蝨ｨ縺ｯ繝吶・繧ｹ繧貞ｾｩ蟶ｰ:
  - `node nodesrc/tests.js -i tests/if.n.md -o /tmp/tests-if-after-revert.json -j 1` 縺ｧ `55/55 pass`
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-after-revert.json -j 1` 縺ｯ `11 pass / 5 fail`・域里遏･谿倶ｻｶ・・
- 谺｡繧｢繧ｯ繧ｷ繝ｧ繝ｳ:
  - 鬘樔ｼｼ蜀咲樟繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縺励※縲～if` 縺ｨ髢｢謨ｰ蛟､蛻・ｲ舌・螟ｱ謨玲擅莉ｶ繧偵ユ繧ｹ繝医→縺励※蝗ｺ螳壹☆繧九�・
  - 縺昴・蠕後�∽ｸ頑ｵ∝━蜈医〒 parser/typecheck 縺ｮ雋ｬ蜍吝｢・阜繧剃ｿ昴▲縺滉ｿｮ豁｣縺ｸ騾ｲ繧�縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (if.n.md 荳崎ｶｳ繧ｱ繝ｼ繧ｹ霑ｽ蜉�縺ｨ if-layout 陬懈ｭ｣)
- `if.n.md` 縺ｮ荳崎ｶｳ繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�:
  - `if <cond_expr>:` 蠖｢蠑擾ｼ・then/else` 繧呈隼陦後〒荳弱∴繧句ｽ｢・・
  - `if cond <cond_expr>:` 蠖｢蠑・
  - marker 鬆・ｺ城＆蜿・/ duplicate / missing 縺ｮ `compile_fail`
- parser 菫ｮ豁｣:
  - `if` 縺ｮ `expected=2`・・if <cond_expr>:` 邉ｻ・峨〒縲～if` 逶ｴ蠕後・莉ｻ諢・`cond` marker 繧帝勁蜴ｻ縺励※ cond 蠑上→縺励※隗｣驥医〒縺阪ｋ繧医≧菫ｮ豁｣縲・
  - `if-layout` 縺ｮ marker 鬆・ｺ上メ繧ｧ繝・け繧定ｿｽ蜉�縺励�～cond -> then -> else` 縺ｮ騾・｡後ｒ繧ｨ繝ｩ繝ｼ蛹悶�・
- 讀懆ｨｼ:
  - `NO_COLOR=true trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/if.n.md -o /tmp/tests-if-added-missing3.json -j 1`
    - `total=54, passed=54, failed=0, errored=0`
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-after-ifcases.json -j 1`
    - `total=16, passed=11, failed=5, errored=0`・亥､ｱ謨怜・險ｳ縺ｯ蠕捺擂縺ｮ鬮倬嚴髢｢謨ｰ/capture 邉ｻ・・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (莠育ｴ・ｪ槭・隴伜挨蟄千ｦ∵ｭ｢: cond/then/else/do, let/fn)
- 繝ｦ繝ｼ繧ｶ繝ｼ謖・､ｺ縺ｫ蜷医ｏ縺帙※縲～cond` / `then` / `else` / `do` 繧剃ｺ育ｴ・ｪ槭→縺励※謇ｱ縺・ｮ溯｣・ｒ parser 縺ｫ霑ｽ蜉�縲・
  - `nepl-core/src/parser.rs`
    - `parse_ident_symbol_item` 縺ｧ縲〕ayout marker 縺ｮ險ｱ蜿ｯ菴咲ｽｮ・亥・鬆ｭ marker / if 譁・ц / while 譁・ц・我ｻ･螟悶〒縺ｮ菴ｿ逕ｨ繧偵お繝ｩ繝ｼ蛹悶�・
    - `expect_ident` 縺ｧ繧ょ酔隱槭ｒ隴伜挨蟄舌→縺励※蜿励￠莉倥￠縺ｪ縺・ｈ縺・↓縺励�∝ｮ夂ｾｩ蜷阪・譚溽ｸ帛錐蛛ｴ縺ｧ繧よ拠蜷ｦ縲・
    - 譌｢蟄倥・邱ｩ蜥・(`KwSet` / `KwTuple` 繧定ｭ伜挨蟄仙喧) 縺ｯ蜑企勁縺励�∽ｺ育ｴ・ｪ槭ｒ譏守｢ｺ蛹悶�・
- `let` / `fn` 縺ｯ lexer 縺ｧ keyword token 蛹悶＆繧後ｋ縺溘ａ縲∝ｾ捺擂縺ｩ縺翫ｊ隴伜挨蟄舌→縺励※菴ｿ逕ｨ荳榊庄縺ｧ縺ゅｋ縺薙→繧堤｢ｺ隱阪�・
- `tests/if.n.md` 縺ｫ compile_fail 繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�・郁ｿｽ蜉�縺ｮ縺ｿ・・
  - `reserved_cond_cannot_be_identifier`
  - `reserved_then_cannot_be_function_name`
  - `reserved_let_fn_cannot_be_identifier`
  - `reserved_else_do_cannot_be_identifier`
- 讀懆ｨｼ:
  - `NO_COLOR=true trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/if.n.md -o /tmp/tests-if-reserved2.json -j 1`
    - `total=46, passed=46, failed=0, errored=0`
- 蜿り�・ｦｳ貂ｬ・育ｶ咏ｶ夊ｪｲ鬘鯉ｼ・
  - `tests/functions.n.md::doctest#7` 縺ｯ parser AST 蠖｢迥ｶ閾ｪ菴薙・ `if + con + then-block + else-block` 縺ｧ豁｣縺励＞縲・
  - 縺溘□縺・then/else 繝悶Ο繝・け蜀・↓蛟､蠑上′2縺､縺ゅｊ縲》ypecheck 縺ｧ `expression left extra values on the stack` 縺ｫ縺ｪ繧九�・
  - 莉墓ｧ俶紛逅・ｼ郁､・焚蛟､蠑上・謇ｱ縺・ｼ峨→ tests/functions 縺ｮ諢丞峙遒ｺ隱阪′蠢・ｦ√�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (if/while 縺ｮ AST 莉墓ｧ倥ユ繧ｹ繝郁ｿｽ蜉�)
- `plan.md` 縺ｮ `if/while` 莉墓ｧ倥ｒ蜀咲｢ｺ隱阪＠縲～cond/then/else/do` 縺ｮ `:` 縺ゅｊ/縺ｪ縺怜ｷｮ蛻・ｒ AST 縺ｧ蝗ｺ螳壹☆繧九ユ繧ｹ繝医ｒ霑ｽ蜉�縲・
- `nodesrc/test_analysis_api.js` 縺ｫ `analyze_parse` 繝吶・繧ｹ縺ｮ繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�:
  - `parse_if_inline_no_colon_blocks`
  - `parse_if_colon_uses_block_for_cond_then_else`
  - `parse_while_inline_no_colon_blocks`
  - `parse_while_colon_uses_block_for_cond_do`
- 讀懆ｨｼ譁ｹ驥・
  - `:` 縺ｪ縺励〒縺ｯ `PrefixExpr` 縺ｮ蠑墓焚蛻励↓ `Block` 繧剃ｽ懊ｉ縺ｪ縺・�・
  - `:` 縺ゅｊ縺ｧ縺ｯ `if` 縺ｯ `Symbol + Block + Block + Block`縲～while` 縺ｯ `Symbol + Block + Block` 縺ｫ縺ｪ繧九％縺ｨ繧堤｢ｺ隱阪�・
- 螳溯｡檎ｵ先棡:
  - `node nodesrc/test_analysis_api.js`
  - `summary: total=6, passed=6, failed=0`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (functions 螟ｱ謨励・豺ｱ謗倥ｊ: symbol/entry)
- `tests` 蜈ｨ菴薙ｒ蜀榊ｮ溯｡後＠縲∫樟迥ｶ繧貞・遒ｺ隱・
  - `/tmp/tests-restored-stable.json` = `total=312, passed=273, failed=39, errored=0`
  - 螟ｱ謨励・荳ｻ蝪翫・ `tests/functions.n.md`・・0縲・1莉ｶ・峨〒縲］ested fn / function value / entry 隗｣豎ｺ縺御ｸｭ蠢・�・
- `functions` 縺ｮ `doctest#3`・・fn main ()`・峨ｒ譛�蟆丞・迴ｾ縺ｧ隱ｿ譟ｻ:
  - `/tmp/fnmain_no_annot.nepl` 繧・`nepl-cli --verbose` 縺ｧ繧ｳ繝ｳ繝代う繝ｫ縲・
  - 隕ｳ貂ｬ:
    - monomorphize 蛻晄悄髢｢謨ｰ縺ｯ `main__unit__i32__pure`
    - 譛ｬ譁・ｸｭ `inc 41` 縺・`unknown function inc` 縺ｧ關ｽ縺｡繧・
  - 隗｣驥・
    - hoist 譎ゅ・髢｢謨ｰ symbol 縺ｨ縲…heck_function 蠕後・髢｢謨ｰ蜷搾ｼ・angle 蠕鯉ｼ峨′荳�閾ｴ縺励↑縺・ｵ瑚ｷｯ縺梧ｮ九▲縺ｦ縺翫ｊ縲‘ntry 谺�關ｽ縺ｨ蜷梧�ｹ縲・
- 隧ｦ陦・
  - `check_function` 縺ｸ symbol override 繧呈ｸ｡縺励�”oist 縺ｧ驕ｸ縺ｰ繧後◆ symbol 縺ｫ髢｢謨ｰ蜷阪ｒ謠・∴繧倶ｿｮ豁｣繧貞ｮ滄ｨ薙�・
  - 縺励°縺・`tests/functions.n.md` 縺ｧ `doctest#3` 縺・run fail 縺九ｉ compile fail・・nknown function inc・峨∈謔ｪ蛹悶＠縲∝・菴捺隼蝟・↓縺ｪ繧峨↑縺九▲縺溘◆繧∵彫蝗槭�・
- 迴ｾ譎らせ縺ｮ邨占ｫ・
  - 蜷榊燕遨ｺ髢灘・險ｭ險茨ｼ・alueNs/CallableNs 蛻・屬・峨→縲］ested fn 縺ｮ螳滉ｽ鍋函謌撰ｼ亥ｰ代↑縺上→繧・non-capture 蜈郁｡鯉ｼ峨′蠢・ｦ√�・
  - 螻�謇� patch 縺ｧ縺ｯ `functions` 鄒､縺ｮ讒矩��蝠城｡後ｒ蜷ｸ蜿弱＠縺阪ｌ縺ｪ縺・�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (荳頑ｵ∝━蜈・ if-layout parser 謾ｹ蝟・+ LSP隗｣譫植PI諡｡蠑ｵ)
- 荳頑ｵ∝━蜈医・譁ｹ驥昴〒 parser 繧貞・縺ｫ隱ｿ謨ｴ縲・
  - `if <cond>:` 縺ｧ then 陦後・縺ｿ蜈医↓隕九∴繧倶ｸｭ髢鍋憾諷九ｒ縲∫｢ｺ螳壹お繝ｩ繝ｼ縺ｫ縺励↑縺・ｈ縺・屓蠕ｩ蛻・ｲ舌ｒ霑ｽ蜉�縲・
  - `functions#doctest#10` 縺ｮ parser 螟ｱ謨暦ｼ・missing expression(s) in if-layout block`・峨ｒ隗｣豸医�・
- 蝗槫ｸｰ遒ｺ隱・
  - `NO_COLOR=true trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -o /tmp/tests-after-parser-upstream.json -j 4`
    - `total=312, passed=275, failed=37, errored=0`・・2 謾ｹ蝟・ｼ・
- LSP/繝・ヰ繝・げ謾ｯ謠ｴ蜷代￠ API 繧定ｿｽ蜉�:
  - `nepl-web/src/lib.rs` 縺ｫ `analyze_name_resolution(source)` 繧定ｿｽ蜉�縲・
    - `definitions`・亥ｮ夂ｾｩ轤ｹ・・
    - `references`・亥盾辣ｧ轤ｹ縲∝�呵｣廬D蛻励�∵怙邨りｧ｣豎ｺID・・
    - `by_name`・亥酔蜷崎ｭ伜挨蟄舌・騾・ｼ輔″・・
    - 蟾ｻ縺堺ｸ翫￡隕丞援縺ｯ迴ｾ陦御ｻ墓ｧ假ｼ・fn` 縺ｨ `let` 髱・`mut`・峨↓蜷医ｏ縺帙◆縲・
  - `nodesrc/analyze_source.js` 縺ｫ `--stage resolve` 繧定ｿｽ蜉�縲・
- API讀懆ｨｼ縺ｮ霑ｽ蜉�・郁ｿｽ蜉�縺ｮ縺ｿ縲∵里蟄・ests蜑企勁縺ｪ縺暦ｼ・
  - `nodesrc/test_analysis_api.js` 繧呈眠隕剰ｿｽ蜉�縲・
  - `shadowing_local_let` / `fn_alias_target_resolution` 繧定・蜍墓､懆ｨｼ縲・
  - 螳溯｡檎ｵ先棡: `2/2 passed`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (functions: nested fn 螳滉ｽ鍋函謌舌・蜑埼�ｲ)
- `typecheck` 縺ｮ `BlockChecker` 縺ｧ nested `fn` 縺ｮ譛ｬ菴薙ｒ縲梧悴讀懈渊縺ｧ辟｡隕悶�阪＠縺ｦ縺・◆邨瑚ｷｯ繧呈隼菫ｮ縲・
  - block 蜀・`Stmt::FnDef` 繧・`check_function` 縺ｫ貂｡縺励�～generated_functions` 縺ｸ霑ｽ蜉�縺吶ｋ繧医≧螟画峩縲・
  - top-level / impl 蛛ｴ縺ｮ `check_function` 蜻ｼ縺ｳ蜃ｺ縺励↓繧・`generated_functions` 繧呈磁邯壹�・
- 縺薙ｌ縺ｫ繧医ｊ nested `fn` 縺ｮ譛ｬ菴薙′ HIR 縺ｫ蜈･繧九ｈ縺・↓縺ｪ繧翫�～functions` 縺ｮ `double` 邉ｻ縺梧隼蝟・�・
- 險域ｸｬ:
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-now.json -j 1`
  - `total=16, passed=10, failed=6, errored=0`
  - 谿九ｊ縺ｯ髢｢謨ｰ蛟､/髢｢謨ｰ繝ｪ繝・Λ繝ｫ/繧ｯ繝ｭ繝ｼ繧ｸ繝｣謐墓拷・・doctest#6,#7,#11,#12,#13`・峨↓髮・ｸｭ縲・
  - 蜈ｨ菴薙・ `node nodesrc/tests.js -i tests -o /tmp/tests-current-after-nested.json -j 4` 縺ｧ `312/278/34/0`縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (荳榊ｮ牙ｮ壼ｷｮ蛻・・蛻・ｊ謌ｻ縺励→蜀崎ｨ域ｸｬ)
- `typecheck` 縺ｮ蛹ｿ蜷埼未謨ｰ繝ｪ繝・Λ繝ｫ螳滄ｨ難ｼ・PrefixItem::Group` + 逶ｴ蠕・`Block` 縺ｮ蜊ｳ蟶ｭ繝ｩ繝�繝�蛹厄ｼ峨ｒ蛻・ｊ謌ｻ縺励�・
  - 譬ｹ諡�: `functions#doctest#6` 縺ｪ縺ｩ縺ｧ `unsupported function signature for wasm` / `unknown variable square` 繧定ｪ倡匱縺励�・未謨ｰ蛟､邨瑚ｷｯ縺梧悴險ｭ險医・縺ｾ縺ｾ豺ｷ蜈･縺励※縺・◆縺溘ａ縲・
- 蜀崎ｨ域ｸｬ:
  - `NO_COLOR=true trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-latest.json -j 1`
    - `total=16, passed=10, failed=6, errored=0`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-latest.json -j 4`
    - `total=312, passed=278, failed=34, errored=0`
- 螟ｱ謨励・荳ｭ蠢・・蠑輔″邯壹″ `functions` 縺ｮ髢｢謨ｰ蛟､/繧ｯ繝ｭ繝ｼ繧ｸ繝｣謐墓拷邉ｻ・・6 #7 #11 #12 #13・峨�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (鬮倬嚴髢｢謨ｰ螳溯｣・婿蠑上・螟夜Κ隱ｿ譟ｻ)
- Rust/MoonBit/Wasm 莉墓ｧ倥ｒ遒ｺ隱阪＠縲¨EPL 蛛ｴ縺ｮ螳溯｣・婿驥昴ｒ謨ｴ逅・＠縺溘�・
- 荳ｻ隕√・繧､繝ｳ繝・
  - Rust:
    - 繧ｯ繝ｭ繝ｼ繧ｸ繝｣縺ｯ縲檎腸蠅・ｒ菫晄戟縺吶ｋ讒矩��菴・+ `Fn/FnMut/FnOnce` 蜻ｼ縺ｳ蜃ｺ縺励�阪〒陦ｨ迴ｾ縺輔ｌ繧具ｼ亥梛縺ｨ縺励※縺ｯ髢｢謨ｰ繝昴う繝ｳ繧ｿ縺ｧ縺ｯ縺ｪ縺丞ｰら畑蝙具ｼ峨�・
    - 蜿り�・ Rust book 縺ｨ rustc `ClosureArgs` 隱ｬ譏弱�・
  - MoonBit:
    - 髢｢謨ｰ縺ｯ first-class縲・
    - Wasm FFI 縺ｧ縺ｯ `FuncRef[T]`・磯哩縺倥◆髢｢謨ｰ・峨→縲…losure・磯未謨ｰ + 迺ｰ蠅・ｼ峨ｒ蛹ｺ蛻･縺励※謇ｱ縺・ｨｭ險医′譏守､ｺ縺輔ｌ縺ｦ縺・ｋ縲・
    - closure 縺ｯ host 蛛ｴ縺ｧ驛ｨ蛻・←逕ｨ縺励※ callback 蛹悶☆繧玖ｨｭ險医′險倩ｿｰ縺輔ｌ縺ｦ縺・ｋ縲・
  - Wasm:
    - 髢捺磁蜻ｼ縺ｳ蜃ｺ縺励・ `call_indirect`・・able 邨檎罰・峨∪縺溘・ `call_ref`・・unction reference・峨〒螳溽樟縲・
- NEPL 縺ｸ縺ｮ蜿肴丐譁ｹ驥晢ｼ域ｬ｡谿ｵ螳溯｣・ｼ・
  - 髢｢謨ｰ蛟､繧貞腰縺ｪ繧玖ｭ伜挨蟄仙盾辣ｧ縺ｧ縺ｯ縺ｪ縺上�！R縺ｧ縲慶allable 蛟､縲阪→縺励※譏守､ｺ陦ｨ迴ｾ縺吶ｋ縲・
  - non-capture 繧貞・陦悟ｮ溯｣・
    - `fn`/`@fn` 縺ｯ table index 繧呈戟縺､髢｢謨ｰ蛟､縺ｨ縺励※謇ｱ縺・�∝他縺ｳ蜃ｺ縺励・ `call_indirect` 縺ｫ邨ｱ荳�縲・
  - capture 縺ゅｊ縺ｯ谺｡谿ｵ:
    - closure 迺ｰ蠅・が繝悶ず繧ｧ繧ｯ繝・+ invoke 髢｢謨ｰ縺ｫ lower 縺吶ｋ closure conversion 繧貞ｰ主・縺吶ｋ縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (block 蠑墓焚菴咲ｽｮ縺ｮ譬ｹ譛ｬ菫ｮ豁｣)
- `tests/block_single_line.n.md` 縺ｮ `doctest#8/#9` 繧定ｵｷ轤ｹ縺ｫ縲～add block 1 block 2` 縺ｨ `if true block 1 else block 2` 縺ｮ螟ｱ謨苓ｦ∝屏繧定ｧ｣譫舌�・
- 蜴溷屏:
  - parser 荳翫〒縺ｯ `add [Block 1] [Block 2]` 縺ｮ AST 縺悟ｾ励ｉ繧後※縺・ｋ縺ｮ縺ｫ縲》ypecheck 縺ｧ `expression left extra values on the stack` 縺悟・繧九�・
  - `PrefixItem::Block` 縺ｮ蝙区､懈渊縺・`check_block(b, stack.len(), true)` 縺ｫ縺ｪ縺｣縺ｦ縺翫ｊ縲∝､門・蠑上・繧ｹ繧ｿ繝・け豺ｱ縺輔ｒ block 蜀・ｩ穂ｾ｡縺ｸ謖√■霎ｼ繧薙〒縺・◆縲・
  - 縺昴・邨先棡縲∝ｼ墓焚菴咲ｽｮ block 縺ｮ蜀・Κ縺ｧ螟門・繧ｹ繧ｿ繝・け縺梧ｷｷ蜈･縺励�∫ｰ｡邏・愛螳壹′蟠ｩ繧後※縺・◆縲・
- 菫ｮ豁｣:
  - `nepl-core/src/typecheck.rs` 縺ｮ `PrefixItem::Block` 蛻・ｲ舌ｒ `check_block(b, 0, true)` 縺ｫ螟画峩縺励�｜lock 繧堤峡遶句ｼ上→縺励※讀懈渊縺吶ｋ繧医≧邨ｱ荳�縲・
  - parser 蛛ｴ縺ｯ `block` 縺ｮ蠕檎ｶ壼愛螳壹ｒ髯仙ｮ夊ｿｽ蜉�・・block`/`else` 騾｣謗･縺ｮ縺ｿ邯咏ｶ夲ｼ峨＠縲∵里蟄倥・ `block:` 譁・｢・阜縺ｯ邯ｭ謖√�・
- 險域ｸｬ:
  - `NO_COLOR=true trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -o /tmp/tests-after-typecheck-blockbase.json -j 4`
  - summary: `total=312, passed=273, failed=39, errored=0`
  - 繝吶・繧ｹ繝ｩ繧､繝ｳ `/tmp/tests-latest.json` (`passed=271`) 縺九ｉ `block_single_line` 縺ｮ 2 莉ｶ縺�縺第隼蝟・�∬ｿｽ蜉�螟ｱ謨励↑縺励�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (荳頑ｵ∽ｿｮ豁｣ 邯咏ｶ・ parser/typecheck)
- 螟ｱ謨怜・鬘槭ｒ蜀榊ｮ滓命縺励�∽ｸ頑ｵ・ｼ・exer/parser・峨→ typecheck 縺ｮ蠅・阜繧貞・繧雁・縺代◆縲・
  - 襍ｷ轤ｹ: `/tmp/tests-current.json` = `total=312, passed=249, failed=63, errored=0`
- parser 縺ｮ譬ｹ譛ｬ菫ｮ豁｣:
  - `nepl-core/src/parser.rs` 縺ｧ隴伜挨蟄占ｧ｣譫舌ｒ蜈ｱ騾壼喧・・parse_ident_symbol_item`・峨�・
  - 縺薙ｌ縺ｫ繧医ｊ縲∝ｼ乗枚閼医＃縺ｨ縺ｮ螳溯｣・ｷｮ蛻・ｒ謗帝勁縺励�∽ｻ･荳九ｒ邨ｱ荳�蟇ｾ蠢・
    - `@name`
    - `::`・亥錐蜑咲ｩｺ髢薙ヱ繧ｹ・・
    - `.`・医ヵ繧｣繝ｼ繝ｫ繝蛾�｣邨撰ｼ・
    - `<...>`・亥梛蠑墓焚・・
  - `Option<.T>::None` / `Option<.T>::Some` 縺ｮ繧医≧縺ｪ縲悟梛蠑墓焚 + PathSep縲阪・騾｣邨舌′ parse 縺ｧ縺阪ｋ繧医≧菫ｮ豁｣縲・
- typecheck 縺ｮ譬ｹ譛ｬ菫ｮ豁｣・・ipe 邁｡邏・ｼ・
  - `nepl-core/src/typecheck.rs` 縺ｮ `reduce_calls` / `reduce_calls_guarded` 繧・open_calls 譛�驕ｩ蛹紋ｾ晏ｭ倥°繧峨�√せ繧ｿ繝・け襍ｰ譟ｻ繝吶・繧ｹ縺ｸ謌ｻ縺励◆縲・
  - `|>` 豕ｨ蜈･譎ゅ・蜻ｼ縺ｳ蜃ｺ縺怜叙繧翫％縺ｼ縺暦ｼ・expression left extra values on the stack` 螟夂匱・峨・荳ｻ隕∝屏繧帝勁蜴ｻ縲・
- 險域ｸｬ:
  - `/tmp/tests-after-upstream-pass.json` = `total=312, passed=261, failed=51, errored=0`
  - `/tmp/tests-after-option-fix.json` = `total=312, passed=271, failed=41, errored=0`
- 霑ｽ蜉�菫ｮ豁｣:
  - `parse_single_line_block` 繧偵�形;` 縺檎┌縺・�ｴ蜷医・ 1 譁・〒邨ゆｺ・�阪∈螟画峩縺励�∝腰陦・block 縺ｮ譁・｢・阜繧呈・遉ｺ蛹悶�・
  - 縺溘□縺・`add block 1 block 2` / `if true block 1 else block 2` 縺ｯ縲｝refix 1譁・・蜀・・縺ｧ `block` 繧貞・蟶ｰ逧・↓蜿悶ｊ霎ｼ繧�謖吝虚縺梧ｮ九ｊ縲∵悴隗｣豎ｺ・域ｮ・fail 2・峨�・
- 谿玖ｪｲ鬘鯉ｼ域ｬ｡谿ｵ・・
  - `tests/functions.n.md`・・1 fail・・ nested fn / function-literal / alias / entry 逕滓・謨ｴ蜷・
  - `tests/neplg2.n.md`・・ fail・峨→ `tests/selfhost_req.n.md`・・ fail・・ namespace 縺ｨ callable 隗｣豎ｺ縺ｮ讒矩��蝠城｡・
  - `tests/pipe_operator.n.md`・・ fail・・ pipe 閾ｪ菴薙・荳頑ｵ∝撫鬘後・邵ｮ蟆乗ｸ医∩縺ｧ縲∵ｮ九ｊ縺ｯ蝙区ｳｨ驥・讒矩��菴薙い繧ｯ繧ｻ繧ｹ莉墓ｧ倥→縺ｮ謨ｴ蜷医′荳ｭ蠢・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (鬮倬嚴髢｢謨ｰ 邯咏ｶ・ let-RHS/if-block 蜻ｼ縺ｳ蜃ｺ縺鈴�・・譬ｹ譛ｬ菫ｮ豁｣)
- `functions` 縺ｮ蝗槫ｸｰ繧貞ｼ輔″襍ｷ縺薙＠縺ｦ縺・◆譬ｹ蝗�繧・2 轤ｹ縺ｫ蛻・屬縺励※菫ｮ豁｣縲・
  - `let f get_op true` 邉ｻ:
    - `let` 繧帝�壼ｸｸ縺ｮ auto-call 邨瑚ｷｯ縺ｧ邁｡邏・☆繧九→ `let f get_op` 縺悟・縺ｫ遒ｺ螳壹＠縲～true` 縺悟叙繧頑ｮ九＆繧後ｋ縲・
    - 蟇ｾ蠢懊→縺励※ `Symbol::Let` 縺ｯ `auto_call: false` 縺ｨ縺励�～check_prefix` 邨らｫｯ縺ｧ `stack[base+1]` 繧・RHS 縺ｨ縺励※ `HirExprKind::Let` 縺ｫ遒ｺ螳壹☆繧狗ｵ瑚ｷｯ繧呈紛蛯吶�・
    - `let ...;` 縺ｧ `statement must leave exactly one value` 縺ｫ縺ｪ繧峨↑縺・ｈ縺・�～let` 髯肴�ｼ譎ゅ↓蜀・Κ stack 繧・`unit` 1 蛟九∈豁｣隕丞喧縲・
  - `if` + `then/else` 縺碁未謨ｰ蛟､繧定ｿ斐☆邉ｻ・・function_return`・・
    - `PrefixItem::Block` 繧・`auto_call: true` 縺ｧ遨阪・縺ｨ縲～if` 縺ｮ蠑墓焚蜿朱寔荳ｭ縺ｫ蜿ｳ遶ｯ縺ｮ髢｢謨ｰ蛟､縺悟━蜈医＆繧・`if` 譛ｬ菴薙′邁｡邏・＆繧後↑縺・�・
    - `PrefixItem::Block` 縺ｮ push 繧・`auto_call: false` 縺ｫ螟画峩縺励�～if` 縺ｮ 3 蠑墓焚邁｡邏・ｒ蜆ｪ蜈医＆縺帙ｋ繧医≧菫ｮ豁｣縲・
- `reduce_calls` 縺ｯ縲悟承遶ｯ蜆ｪ蜈医・荳崎ｶｳ縺ｪ繧牙ｾ・▽縲阪↓謌ｻ縺励◆縲・
  - 蟾ｦ謗｢邏｢繧呈怏蜉ｹ蛹悶☆繧九→ `mul n fact sub n 1` 縺ｧ `mul n fact` 縺悟・縺ｫ遒ｺ螳壹＠縲∝・蟶ｰ蜻ｼ縺ｳ蜃ｺ縺励′螢翫ｌ繧九％縺ｨ繧貞・迴ｾ遒ｺ隱阪＠縺溘◆繧∵彫蝗槭�・

- 讀懆ｨｼ邨先棡:
  - `NO_COLOR=true trunk build`: 謌仙粥
  - `node nodesrc/test_analysis_api.js`: `7/7 pass`
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-after-block-autocall-false.json -j 1`
    - `total=19, passed=15, failed=4, errored=0`
    - 谿・fail: `doctest#12 #13 #16 #17`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-hof-upstream-fixes.json -j 1`
    - `total=328, passed=288, failed=40, errored=0`

- 谿倶ｻｶ縺ｮ蛻・梵:
  - `doctest#12/#13/#16`:
    - typecheck 縺ｧ縺ｯ nested 髢｢謨ｰ蜀・`y` 蜿ら・縺ｯ隗｣豎ｺ縺ｧ縺阪※縺・ｋ縺後�…odegen 縺ｧ `unknown variable y` 縺ｫ縺ｪ繧九�・
    - 縺薙ｌ縺ｯ nested 髢｢謨ｰ縺ｮ capture 縺梧悴 lower・・losure conversion 譛ｪ螳溯｣・ｼ峨〒縺ゅｋ縺薙→縺悟次蝗�縲・
  - `doctest#17`:
    - `compile_fail` 譛溷ｾ・↓蟇ｾ縺励※謌仙粥縺吶ｋ縺溘ａ縲∫ｴ皮ｲ・髱樒ｴ皮ｲ九・ effect 蛻､螳夂ｵ瑚ｷｯ・育ｽｲ蜷崎ｧ｣驥・or overload 驕ｸ謚橸ｼ峨・蜀咲せ讀懊′蠢・ｦ√�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (lexer/parser 隗｣譫植PI霑ｽ蜉�)
- VSCode 諡｡蠑ｵ險育判・・odo.md 縺ｮ LSP / VSCode 鬆・ｼ峨ｒ蜀咲｢ｺ隱阪＠縲∽ｸ頑ｵ∬ｧ｣譫舌ｒ蜿ｯ隕門喧縺吶ｋ API 繧貞・縺ｫ霑ｽ蜉�縺励◆縲・
- `nepl-web/src/lib.rs` 縺ｫ wasm 蜈ｬ髢矩未謨ｰ繧定ｿｽ蜉�:
  - `analyze_lex(source)`:
    - token 蛻暦ｼ・ind/value/debug/span・・
    - diagnostics・・everity/message/code/span・・
    - span 縺ｮ byte 遽・峇縺ｨ line/col 繧定ｿ斐☆
  - `analyze_parse(source)`:
    - token 蛻・
    - lex/parse diagnostics
    - module 縺ｮ譛ｨ讒矩��・・lock/Stmt/Expr/PrefixItem 縺ｮ蜀榊ｸｰ JSON・・
    - debug 逕ｨ縺ｮ AST pretty 譁・ｭ怜・
- Node 蛛ｴ縺ｫ `nodesrc/analyze_source.js` 繧定ｿｽ蜉�縺励�‥ist 縺ｮ wasm API 繧剃ｽｿ縺｣縺ｦ隗｣譫千ｵ先棡繧貞叙蠕励〒縺阪ｋ繧医≧縺ｫ縺励◆縲・
  - `--stage lex|parse`
  - `-i <file>` 縺ｾ縺溘・ `--source`
  - `-o <json>`
- 螳溯｡檎｢ｺ隱・
  - `NO_COLOR=true trunk build`: 謌仙粥
  - `node nodesrc/analyze_source.js --stage lex -i tests/functions.n.md -o /tmp/functions-lex.json`: 謌仙粥
  - `node nodesrc/analyze_source.js --stage parse -i tests/functions.n.md -o /tmp/functions-parse.json`: 謌仙粥
- 蝗槫ｸｰ遒ｺ隱・
  - `node nodesrc/tests.js -i tests -o /tmp/tests-current.json -j 4`
  - summary: `total=312, passed=249, failed=63, errored=0`
  - 荳ｻ隕∝､ｱ謨励・譌｢遏･縺ｮ block/typecheck 邉ｻ・井ｻ雁屓縺ｮ API 霑ｽ蜉�縺ｧ縺ｯ譛ｪ逹�謇具ｼ・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (namespace蜀崎ｨｭ險育捩謇・
- plan.md 縺ｮ蜀咲｢ｺ隱・
  - `fn` 縺ｯ `let` 縺ｮ邉冶｡｣讒区枚
  - 螳夂ｾｩ縺ｮ蟾ｻ縺堺ｸ翫￡縺ｯ `mut` 縺ｧ縺ｪ縺・`let` 縺ｮ縺ｿ・・fn` 繧ょ性繧�・・
- 螳溯｣・・險域ｸｬ:
  - lexer 縺ｫ `@` 縺ｨ `0x...` 繧定ｿｽ蜉�
  - parser 縺ｫ `@ident` / `fn alias @target;` / `let` 髢｢謨ｰ邉冶｡｣ / `fn` 蝙区ｳｨ驥育怐逡･繧定ｿｽ蜉�
  - `NO_COLOR=true trunk build` 縺ｯ謌仙粥
  - `node nodesrc/tests.js -i tests -o /tmp/tests-only-after-upstream-fix.json -j 4`:
    - `total=309, passed=242, failed=67, errored=0`
  - `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/functions-only-after-entry-fix.json -j 1`:
    - `total=16, passed=5, failed=11, errored=0`
- 隕ｳ貂ｬ縺励◆譬ｹ譛ｬ蝠城｡・
  - 蜷榊燕隗｣豎ｺ縺・`Env` 縺ｮ蜊倅ｸ�繝・・繝悶Ν縺ｫ蟇・ｊ縺吶℃縺ｦ縺翫ｊ縲∝､画焚縺ｨ髢｢謨ｰ蛟､縲∥lias縲‘ntry 隗｣豎ｺ縺悟酔荳�邨瑚ｷｯ縺ｧ蟷ｲ貂峨☆繧・
  - nested `fn` 繧・block 縺ｧ螳｣險�縺ｧ縺阪※繧ゅ�？irFunction 縺ｫ關ｽ縺｡縺・`unknown function` 縺ｸ郢九′繧・
  - entry 縺ｯ隗｣豎ｺ縺ｧ縺阪※繧・codegen 蛛ｴ縺ｫ髢｢謨ｰ譛ｬ菴薙′辟｡縺・�ｴ蜷医↓ `_start` 縺悟・蜉帙＆繧後↑縺・ｼ亥ｮ溯｡梧凾繧ｨ繝ｩ繝ｼ蛹厄ｼ・
- 逶ｴ霑代・菫ｮ豁｣:
  - top-level `fn alias` 縺ｮ逋ｻ骭ｲ繧帝未謨ｰ譛ｬ菴薙メ繧ｧ繝・け蜑阪↓遘ｻ蜍・
  - 蝙区悴遒ｺ螳夐未謨ｰ縺ｮ symbol 縺ｯ證ｫ螳壹〒 unmangled 蜷阪ｒ菴ｿ縺・ｈ縺・､画峩・・ntry/mangle縺壹ｌ邱ｩ蜥鯉ｼ・
- 谺｡繧ｹ繝・ャ繝・
  - namespace 繧・`ValueNs` / `CallableNs` 縺ｫ蛻・屬縺励�∝ｷｻ縺堺ｸ翫￡繧剃ｻ墓ｧ俶ｺ匁侠縺ｫ蟇・○繧・
  - entry 縺ｮ縲瑚ｧ｣豎ｺ貂医∩縺九▽逕滓・貂医∩縲肴､懆ｨｼ繧定ｿｽ蜉�縺励※ compile error 蛹悶☆繧・
- 繝峨く繝･繝｡繝ｳ繝磯°逕ｨ菫ｮ豁｣:
  - `todo.md` 縺ｯ譛ｪ螳御ｺ・ち繧ｹ繧ｯ縺ｮ縺ｿ繧呈ｮ九☆蠖｢蠑上∈謨ｴ逅・
  - 騾ｲ謐励・螻･豁ｴ繝ｻ險域ｸｬ蛟､縺ｯ `note.n.md` 縺ｮ縺ｿ縺ｸ髮・ｴ・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (wasm32 build)
- wasm32-unknown-unknown 縺ｧ縺ｮ `cargo test --no-run` 縺・getrandom 縺ｮ js feature 縺ｪ縺励〒螟ｱ謨励＠縺ｦ縺・◆縺溘ａ縲～nepl-core` 縺ｮ wasm32 逕ｨ dev-dependencies 縺ｫ `getrandom` (features=["js"]) 繧定ｿｽ蜉�縺励◆縲・
- `cargo test --target wasm32-unknown-unknown --no-run --all --all-features` 繧貞ｮ溯｡後＠縲，argo.lock 繧呈峩譁ｰ縺励※繝薙Ν繝峨′騾壹ｋ縺薙→繧堤｢ｺ隱阪�・
- `cargo test --target wasm32-unknown-unknown --no-run --all --all-features --locked` 繧よ・蜉溘�・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (selfhost string builder)
- stdlib/alloc/string.nepl 縺ｫ StringBuilder・・b_append/sb_append_i32/sb_build・峨ｒ霑ｽ蜉�縺励�《elfhost_req 縺ｮ譁・ｭ怜・繝薙Ν繝�隕∽ｻｶ繧定ｧ｣遖√＠縺溘�・
- stdlib/tests/string.nepl 縺ｫ StringBuilder 縺ｮ讀懆ｨｼ繧定ｿｽ蜉�縺励◆縲・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (selfhost string utils)
- stdlib/alloc/string.nepl 縺ｫ trim/starts_with/ends_with/slice/split 繧定ｿｽ蜉�縺励�、SCII 遨ｺ逋ｽ蛻､螳壹ｄ split 逕ｨ縺ｮ陬懷勧髢｢謨ｰ繧貞ｮ溯｣・＠縺溘�・
- stdlib/tests/string.nepl 繧呈僑蜈・＠縺ｦ trim/starts_with/ends_with/slice/split 縺ｮ繝・せ繝医ｒ霑ｽ蜉�縺励◆縲・
- nepl-core/tests/selfhost_req.rs 縺ｮ譁・ｭ怜・繝ｦ繝ｼ繝・ぅ繝ｪ繝・ぅ隕∽ｻｶ繝・せ繝医ｒ隗｣遖√＠縲＾ption unwrap 縺ｨ len 蜻ｼ縺ｳ蜃ｺ縺励↓蜷医ｏ縺帙※蜀・ｮｹ繧定ｪｿ謨ｴ縺励◆縲・
- doc/testing.md 縺ｮ stdlib 繧ｹ繧ｳ繝ｼ繝嶺ｸ�隕ｧ繧呈峩譁ｰ縺励�∥lloc/string 縺ｮ霑ｽ蜉�髢｢謨ｰ繧貞渚譏�縺励◆縲・
- 譛ｪ蟇ｾ蠢・ file I/O (WASI 縺ｮ path_open 遲・ 縺ｨ u8/繝舌う繝磯・蛻励・蝙九・螳溯｡檎腸蠅・・謨ｴ蛯吶′蠢・ｦ√↑縺溘ａ譛ｪ逹�謇九�Ｔtring-keyed map/trait 諡｡蠑ｵ繧ょｾ檎ｶ壹〒蟇ｾ蠢應ｺ亥ｮ壹�・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (block 繝ｫ繝ｼ繝ｫ譖ｴ譁ｰ蟇ｾ蠢・
- block: 縺後ヶ繝ｭ繝・け蠑上�～:` 縺悟ｼ墓焚繝ｬ繧､繧｢繧ｦ繝医→縺・≧譁ｰ繝ｫ繝ｼ繝ｫ縺ｫ蜷医ｏ縺帙�√ヱ繝ｼ繧ｵ縺ｮ `:` 蜃ｦ逅・ｒ謨ｴ逅・�Ａblock` 縺ｯ譛ｫ蟆ｾ縺ｪ繧峨・繝ｼ繧ｫ繝ｼ謇ｱ縺・�～cond/then/else/do` 縺ｯ蜊倡峡・亥梛豕ｨ驥医・縺ｿ險ｱ蜿ｯ・峨〒繝槭・繧ｫ繝ｼ謇ｱ縺・↓縺励�～if cond:` 縺ｮ繧医≧縺ｪ騾壼ｸｸ隴伜挨蟄舌ｒ隱､蛻､螳壹＠縺ｪ縺・ｈ縺・↓縺励◆縲・
- `if`/`while` 縺ｮ繝ｬ繧､繧｢繧ｦ繝亥ｱ暮幕縺ｧ `ExprSemi` 繧定ｨｱ蜿ｯ縺励�～while` 譛ｬ菴薙↓ `;` 繧呈嶌縺・◆繝・せ繝医′ panic 縺励↑縺・ｈ縺・ｿｮ豁｣縲・
- stdlib/萓・ `while ...:` 縺ｮ隍・焚譁・・繝・ぅ繧・`do:` 繝悶Ο繝・け蛹厄ｼ・tdlib/alloc/*, core/mem, std/stdio, std/env/cliarg, kp/kpread, examples/counter/fib/rpn 縺ｪ縺ｩ・峨�Ａexamples/rpn.nepl` 縺ｮ蜈･繧悟ｭ・while 繧・`do:` 縺ｫ邨ｱ荳�縲・
- tests: `nepl-core/tests/plan.rs` 繧・`block:` 菴ｿ逕ｨ縺ｫ譖ｴ譁ｰ縲～nepl-core/tests/typeannot.rs` 縺ｮ while 繧・`do:` 縺ｫ譖ｴ譁ｰ縲Ａstdlib/tests/vec.nepl` 縺ｮ match arm 縺九ｉ隱､縺｣縺・`block` 繝槭・繧ｫ繝ｼ繧帝勁蜴ｻ縲・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 繧貞ｮ溯｡後＠縲∽ｸ｡譁ｹ謌仙粥・郁ｭｦ蜻翫・譌｢蟄倥・縺ｾ縺ｾ・峨�・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (萓晏ｭ俶峩譁ｰ/online cargo test)
- workspace 萓晏ｭ倥ｒ譛�譁ｰ螳牙ｮ夂沿縺ｸ譖ｴ譁ｰ・・hiserror 2.0.18縲∥nyhow 1.0.100縲…lap 4.5.56縲『asm-bindgen 0.2.108縲∥ssert_cmd 2.1.2縲》empfile 3.24.0 縺ｪ縺ｩ・峨�Ｓand 縺ｯ譛�譁ｰ螳牙ｮ壹・ 0.8.5 縺ｮ縺ｾ縺ｾ縲・
- wasmi 1.0.8 縺ｸ縺ｮ譖ｴ譁ｰ繧定ｩｦ縺励◆縺後�〉ustc 1.83.0 縺ｧ縺ｯ 1.86 莉･荳翫′蠢・ｦ√〒荳榊庄縲Ｘasmi 縺ｯ 0.31.2 縺ｫ謌ｻ縺励※ Cargo.lock 繧呈峩譁ｰ縲・
- 繝・せ繝・ 繧ｪ繝ｳ繝ｩ繧､繝ｳ `cargo test` 繧貞ｮ溯｡後�Ａnepl-core/tests/overload.rs` 縺ｮ `test_overload_cast_like` 縺ｨ `test_explicit_type_annotation_prefix` 縺・"ambiguous overload" 縺ｧ螟ｱ謨励�ゆｻ悶・繝・せ繝医・謌仙粥縲・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (trait/overload 菫ｮ豁｣縺ｮ譬ｹ譛ｬ蟇ｾ蠢・
- overload 縺ｮ驥崎､・炎髯､縺・`type_to_string` 縺ｮ "func" 霑泌唆縺ｧ蜈ｨ縺ｦ蜷御ｸ�謇ｱ縺・↓縺ｪ縺｣縺ｦ縺・◆縺溘ａ縲・未謨ｰ繧ｷ繧ｰ繝阪メ繝｣譁・ｭ怜・繧貞ｰ主・縺励�・㍾隍・愛螳壹→ impl 繝｡繧ｽ繝・ラ鄂ｲ蜷堺ｸ�閾ｴ蛻､螳壹ｒ繧ｷ繧ｰ繝阪メ繝｣豈碑ｼ・↓螟画峩縲・
- trait method 縺ｮ蜻ｼ縺ｳ蜃ｺ縺励〒 `Self` 繝ｩ繝吶Ν縺ｨ蝙九ヱ繝ｩ繝｡繝ｼ繧ｿ縺御ｸ堺ｸ�閾ｴ縺ｫ縺ｪ繧句撫鬘後ｒ縲～Self` 繝ｩ繝吶Ν縺ｯ莉ｻ諢丞梛縺ｨ邨ｱ荳�蜿ｯ閭ｽ縺ｫ縺吶ｋ縺薙→縺ｧ隗｣豸医�・
- monomorphize 縺ｧ trait 蜻ｼ縺ｳ蜃ｺ縺励ｒ蜈ｷ菴馴未謨ｰ縺ｸ隗｣豎ｺ縺吶ｋ髫帙�∬ｧ｣豎ｺ蜈磯未謨ｰ縺ｮ繧､繝ｳ繧ｹ繧ｿ繝ｳ繧ｹ蛹冶ｦ∵ｱゅｒ陦後≧繧医≧螟画峩縺励�「nknown function 繧定ｧ｣豸医�・
- 繝・せ繝・ `cargo run -p nepl-cli -- test` 縺ｯ謌仙粥・郁ｭｦ蜻翫≠繧奇ｼ峨�・
- 繝・せ繝・ `cargo test` 縺ｯ 120 遘偵〒繧ｿ繧､繝�繧｢繧ｦ繝茨ｼ郁ｭｦ蜻雁・蜉帛ｾ後↓譛ｪ螳御ｺ・ｼ峨�・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (stdlib 繝・せ繝域僑蜈・菫ｮ豁｣)
- stdlib/std/hashmap.nepl 縺ｮ if 繝ｬ繧､繧｢繧ｦ繝医ｒ菫ｮ豁｣縺励�”ash_i32 繧堤ｴ皮ｲ矩未謨ｰ縺ｫ譖ｸ縺肴鋤縺茨ｼ・6騾ｲ繝ｪ繝・Λ繝ｫ繧・0騾ｲ縺ｸ鄂ｮ謠幢ｼ峨�Ｉashmap_get 縺ｯ蜀榊ｸｰ繝ｫ繝ｼ繝励〒邏皮ｲ句喧縲・
- stdlib/std/hashset.nepl 縺ｮ hash_i32 繧堤ｴ皮ｲ矩未謨ｰ縺ｸ螟画峩縺励�”ashset_contains 繧貞・蟶ｰ繝ｫ繝ｼ繝励〒邏皮ｲ句喧縲Ｉashset_contains_loop 縺ｮ繧ｷ繧ｰ繝阪メ繝｣荳肴紛蜷医ｂ菫ｮ豁｣縲・
- stdlib/std/result.nepl 縺ｮ unwrap_err 繧・Err 蛻・ｲ仙・鬆ｭ縺ｫ荳ｦ縺ｹ縲［atch 縺ｮ謌ｻ繧雁梛縺・never 縺ｫ縺ｪ繧句撫鬘後ｒ蝗樣∩縲・
- stdlib/tests 縺ｫ hashmap.nepl/hashset.nepl/json.nepl 繧定ｿｽ蜉�縺励�∝渕譛ｬ謫堺ｽ懶ｼ・ew/insert/get/remove/len/contains 縺ｪ縺ｩ・峨→ JSON 縺ｮ蜷・い繧ｯ繧ｻ繧ｵ繧呈､懆ｨｼ縲・
- stdlib/tests/result.nepl 縺ｯ map 邉ｻ繧貞､悶＠縲「nwrap_ok/unwrap_err 縺ｮ讀懆ｨｼ縺ｫ鄂ｮ縺肴鋤縺医�Ｋson.nepl 縺ｯ move 騾｣骼悶ｒ驕ｿ縺代ｋ縺溘ａ蛟､繧帝・蠎ｦ逕滓・縺吶ｋ蠖｢縺ｫ謨ｴ逅・�・
- 繝・せ繝・ `cargo run -p nepl-cli -- test` 縺ｯ謌仙粥・郁ｭｦ蜻翫・谿句ｭ假ｼ峨�・
- 繝・せ繝・ `cargo test` 縺ｯ 120 遘偵〒繧ｿ繧､繝�繧｢繧ｦ繝茨ｼ郁ｭｦ蜻雁・蜉帛ｾ後↓譛ｪ螳御ｺ・ｼ峨�・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (trait/overload)
- AST/繝代・繧ｵ: 蝙九ヱ繝ｩ繝｡繝ｼ繧ｿ繧・TypeParam 蛹悶＠縲～.T: TraitA & TraitB` 蠖｢蠑上・蠅・阜繧定ｪｭ繧√ｋ繧医≧縺ｫ縺励◆縲・
- HIR: trait 蜻ｼ縺ｳ蜃ｺ縺・(`Trait::method`) 繧定｡ｨ迴ｾ縺ｧ縺阪ｋ繧医≧縺ｫ縺励�（mpl 蛛ｴ縺ｯ繝｡繧ｽ繝・ラ荳�隕ｧ繧呈戟縺､蠖｢縺ｫ螟画峩縲・
- 蝙区､懈渊: trait 螳夂ｾｩ/impl 縺ｮ謨ｴ蜷域�ｧ繝√ぉ繝・け縲ヾelf 蝙九・蟾ｮ縺苓ｾｼ縺ｿ縲》rait bound 縺ｮ貅�雜ｳ蛻､螳壹ｒ霑ｽ蜉�縲る未謨ｰ縺ｮ蜷悟錐繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨ｒ險ｱ蜿ｯ縺励�［angle 縺励◆繧ｷ繝ｳ繝懊Ν縺ｧ蜀・Κ蜷阪ｒ荳�諢丞喧縲・
- 蜊倡嶌蛹・ impl 繝槭ャ繝励ｒ讒狗ｯ峨＠縲》rait 蜻ｼ縺ｳ蜃ｺ縺励ｒ蜈ｷ菴鍋噪縺ｪ繝｡繧ｽ繝・ラ螳滉ｽ薙↓隗｣豎ｺ縺吶ｋ繧医≧縺ｫ縺励◆縲・
- 繝・せ繝・ nepl-core/tests/neplg2.rs 縺ｫ繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝・trait 縺ｮ繧ｳ繝ｳ繝代う繝ｫ繝・せ繝医ｒ霑ｽ蜉�縲・
- 譌｢遏･縺ｮ蛻ｶ髯・ trait 縺ｮ蝙九ヱ繝ｩ繝｡繝ｼ繧ｿ縲（nherent impl縲（mpl 繝｡繧ｽ繝・ラ縺ｮ繧ｸ繧ｧ繝阪Μ繧ｯ繧ｹ縺ｯ譛ｪ蟇ｾ蠢懊�ゅが繝ｼ繝舌・繝ｭ繝ｼ繝芽ｧ｣豎ｺ縺ｯ蠑墓焚蝙九・縺ｿ縺ｧ陦後＞縲∵綾繧雁�､蝙九・菴ｿ繧上↑縺・�Ｆxport 蜷阪・ mangle 蠕後・荳�諢丞錐縺ｫ縺ｪ繧九�・
- 繝・せ繝・ `cargo test -p nepl-core --lib` 繧貞ｮ溯｡鯉ｼ郁ｭｦ蜻翫・谿句ｭ假ｼ峨�・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (never 蝙九→ unwrap 菫ｮ豁｣)
- `unreachable` 蛻・ｲ舌〒蝙句､画焚縺・`never` 縺ｫ譚溽ｸ帙＆繧後�～Option::unwrap` 縺・`unwrap__Option_never__never__pure` 縺ｸ貎ｰ繧後ｋ蝠城｡後ｒ菫ｮ豁｣縲・
- `types::unify` 縺ｧ `Var` 縺ｨ `Never` 縺ｮ邨ｱ荳�譎ゅ↓譚溽ｸ帙＠縺ｪ縺・ｈ縺・音萓九ｒ霑ｽ蜉�縺励�～unwrap__Option_T__T__pure` 繧剃ｿ晄戟縺吶ｋ繧医≧縺ｫ縺励◆縲・
- codegen 縺ｮ `unknown function` 險ｺ譁ｭ縺ｫ谺�關ｽ髢｢謨ｰ蜷阪ｒ蜷ｫ繧√ｋ繧医≧謾ｹ蝟・�・
- 繝・せ繝・ `cargo run -p nepl-cli -- test` 縺ｯ謌仙粥・郁ｭｦ蜻翫≠繧奇ｼ峨�・
- 繝・せ繝・ `cargo test` 縺ｯ 240 遘偵〒繧ｿ繧､繝�繧｢繧ｦ繝茨ｼ医さ繝ｳ繝代う繝ｫ騾比ｸｭ・峨�ょ・螳溯｡後′蠢・ｦ√�・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (btreemap/btreeset 霑ｽ蜉�)
- stdlib/std/btreemap.nepl 縺ｨ stdlib/std/btreeset.nepl 繧定ｿｽ蜉�縺励�（32 繧ｭ繝ｼ/隕∫ｴ�縺ｮ鬆・ｺ丈ｻ倥″繧ｳ繝ｬ繧ｯ繧ｷ繝ｧ繝ｳ繧帝・蛻励・繝ｼ繧ｹ縺ｧ螳溯｣・＠縺滂ｼ域､懃ｴ｢縺ｯ莠悟・謗｢邏｢縲∵諺蜈･/蜑企勁縺ｯ繧ｷ繝輔ヨ・峨�・
- stdlib/tests/btreemap.nepl 縺ｨ stdlib/tests/btreeset.nepl 繧定ｿｽ蜉�縺励�∝渕譛ｬ謫堺ｽ懶ｼ域諺蜈･/譖ｴ譁ｰ/蜑企勁/讀懃ｴ｢/髟ｷ縺包ｼ峨ｒ讀懆ｨｼ縺励◆縲・
- doc/testing.md 縺ｮ stdlib 荳�隕ｧ縺ｫ std/btreemap 縺ｨ std/btreeset 繧定ｿｽ險倥＠縺溘�・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (test 蠖ｩ濶ｲ/stdlib 繝・せ繝郁ｪｿ謨ｴ/繧ｳ繝ｳ繝代う繝ｩ遒ｺ隱・
- stdlib/std/test.nepl 縺ｮ螟ｱ謨励Γ繝・そ繝ｼ繧ｸ繧・ANSI 襍､濶ｲ縺ｧ陦ｨ遉ｺ縺吶ｋ繧医≧螟画峩縺励�《td/stdio 縺ｮ濶ｲ蜃ｺ蜉帙ｒ蛻ｩ逕ｨ縲・
- stdlib/tests/error.nepl 縺ｧ `fail` 縺ｮ菴ｿ逕ｨ繧帝∩縺代�‘rror_new 逕ｱ譚･縺ｮ險ｺ譁ｭ縺碁撼遨ｺ縺ｧ縺ゅｋ縺薙→繧堤｢ｺ隱阪☆繧句ｽ｢縺ｫ隱ｿ謨ｴ縲・
- stdlib/tests/cliarg.nepl/list.nepl/stack.nepl/vec.nepl/string.nepl/diag.nepl 繧呈峩譁ｰ縺励�∝､ｱ謨玲凾縺ｮ繝｡繝・そ繝ｼ繧ｸ繧呈・遉ｺ縺吶ｋ繝・せ繝医↓謨ｴ逅・�・
- doc/testing.md 縺ｮ螟ｱ謨玲凾縺ｮ陦ｨ遉ｺ隱ｬ譏弱ｒ譖ｴ譁ｰ縲・
- 繧ｳ繝ｳ繝代う繝ｩ遒ｺ隱・ error::fail・・allsite_span 邨檎罰・峨ｒ蜷ｫ繧�繝・せ繝医〒 wasm 讀懆ｨｼ繧ｨ繝ｩ繝ｼ縺檎匱逕溘☆繧九◆繧√�《td 繝・せ繝亥・縺ｧ縺ｯ隧ｲ蠖鍋ｵ瑚ｷｯ繧剃ｽｿ繧上↑縺・ｈ縺・↓縺励※蝗樣∩縲３ust 蛛ｴ縺ｮ callsite_span/codegen 縺ｮ逶ｸ諤ｧ縺ｯ隕∬ｪｿ譟ｻ縲・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 繧貞ｮ溯｡後�・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (nepl-cli test 縺ｮ濶ｲ莉倥￠)
- nepl-cli 縺ｮ繝・せ繝亥・蜉帙ｒ ANSI 濶ｲ莉倥″縺ｫ縺励�》est/ok/FAILED 縺ｮ隕冶ｪ肴�ｧ繧剃ｸ翫￡縺溘�・
- doc/testing.md 縺ｫ濶ｲ莉倥″蜃ｺ蜉帙・豕ｨ險倥ｒ霑ｽ險倥�・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (stdlib/diag 濶ｲ蛻・￠)
- stdlib/std/diag.nepl 縺ｫ ErrorKind 縺斐→縺ｮ濶ｲ蜑ｲ繧雁ｽ薙※繧定ｿｽ蜉�縺励�‥iag_print/diag_println/diag_debug_print 縺ｧ濶ｲ莉倥″陦ｨ遉ｺ縺ｫ螟画峩縲・
- stdlib/std/stdio.nepl 縺ｫ debug_color/debugln_color 繧定ｿｽ蜉�縲・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 繧貞ｮ溯｡後�・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (Checked 繝ｭ繧ｰ縺ｮ濶ｲ莉倥￠)
- stdlib/std/test.nepl 縺ｫ test_checked 繧定ｿｽ蜉�縺励�・Checked ..." 縺ｮ謌仙粥繝ｭ繧ｰ繧堤ｷ題牡縺ｧ蜃ｺ縺吶ｈ縺・↓縺励◆縲・
- stdlib/tests/list.nepl 縺ｨ stdlib/tests/math.nepl 縺ｮ Checked 繝ｭ繧ｰ繧・test_checked 縺ｫ鄂ｮ縺肴鋤縺医◆縲・
- doc/testing.md 縺ｫ test_checked 繧定ｿｽ險倥�・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (繝・せ繝亥､ｱ謨励・繝｡繝・そ繝ｼ繧ｸ陦ｨ遉ｺ)
- stdlib/std/test.nepl 繧呈隼菫ｮ縺励�∝､ｱ謨玲凾縺ｫ繝｡繝・そ繝ｼ繧ｸ繧定｡ｨ遉ｺ縺励※縺九ｉ trap 縺吶ｋ繧医≧螟画峩縺励◆縲・
- stdlib/std/diag.nepl 縺ｫ diag_print_msg 繧定ｿｽ蜉�縺励�：ailure 繝｡繝・そ繝ｼ繧ｸ繧定｡ｨ遉ｺ縺ｧ縺阪ｋ繧医≧縺ｫ縺励◆縲・
- stdlib/std/error.nepl 縺ｮ fail/context 繧・callsite_span 莉倅ｸ弱↓譖ｴ譁ｰ縺励◆縲・
- stdlib/tests/diag.nepl 縺ｨ stdlib/tests/error.nepl 繧貞ｼｷ蛹悶＠縲∵枚蟄怜・蛹悶ｄ span 縺ｮ讀懆ｨｼ繧定ｿｽ蜉�縺励◆縲・
- doc/testing.md 縺ｮ assert 莉墓ｧ倥ｒ譖ｴ譁ｰ縺励◆縲・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 繧貞ｮ溯｡後�・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (cliarg 霑ｽ蜉�)
- stdlib/std/cliarg.nepl 繧定ｿｽ蜉�縺励�仝ASI args_sizes_get/args_get 縺ｧ argv 繧貞叙蠕励〒縺阪ｋ繧医≧縺ｫ縺励◆縲・
- stdlib/tests/cliarg.nepl 繧定ｿｽ蜉�縺励�∫ｯ・峇螟・雋�縺ｮ index 縺・None 縺ｫ縺ｪ繧九％縺ｨ繧堤｢ｺ隱阪☆繧九ユ繧ｹ繝医ｒ逕ｨ諢上＠縺溘�・
- doc/testing.md 縺ｮ stdlib 荳�隕ｧ縺ｫ std/cliarg 繧定ｿｽ險倥＠縺溘�・
- nepl-cli 縺ｮ WASI 繝ｩ繝ｳ繧ｿ繧､繝�縺ｫ args_sizes_get/args_get 繧定ｿｽ蜉�縺励�～--` 莉･髯阪・蠑墓焚繧呈ｸ｡縺帙ｋ繧医≧縺ｫ縺励◆縲・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 繧貞ｮ溯｡後�・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (cliarg 螳溷ｼ墓焚繝・せ繝・
- stdlib/tests/cliarg.nepl 繧呈峩譁ｰ縺励�∥rgv[1..] 縺ｮ蛟､繧呈､懆ｨｼ縺吶ｋ繝・せ繝医ｒ霑ｽ蜉�縺励◆縲・
- nepl-cli 縺ｮ stdlib 繝・せ繝亥ｮ溯｡後〒 `--flag value` 繧・argv 縺ｫ貂｡縺吶ｈ縺・､画峩縺励◆縲・
- doc/testing.md 縺ｫ stdlib 繝・せ繝医′蝗ｺ螳壼ｼ墓焚繧呈ｸ｡縺呎葎繧定ｿｽ險倥＠縺溘�・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 繧貞ｮ溯｡後�・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (stdlib 繧ｳ繝｡繝ｳ繝郁ｨ�隱樒ｵｱ荳�)
- stdlib/std/option.nepl 縺ｨ stdlib/std/result.nepl 縺ｮ闍ｱ隱槭さ繝｡繝ｳ繝郁｡後ｒ蜑企勁縺励�√さ繝｡繝ｳ繝医′譌･譛ｬ隱槭・縺ｿ縺ｫ縺ｪ繧九ｈ縺・ｵｱ荳�縲・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 繧貞ｮ溯｡後�・
# 2026-02-03 菴懈･ｭ繝｡繝｢ (stdlib 繧ｳ繝｡繝ｳ繝・Option/Result 謾ｹ菫ｮ)
- stdlib/std 縺ｮ蜷・ヵ繧｡繧､繝ｫ縺ｫ譌･譛ｬ隱槭さ繝｡繝ｳ繝茨ｼ医ヵ繧｡繧､繝ｫ讎りｦ・蜷・未謨ｰ縺ｮ逶ｮ逧・・螳溯｣・・豕ｨ諢上・險育ｮ鈴㍼・峨ｒ霑ｽ蜉�縺励�［ath.nepl 縺ｯ閾ｪ蜍慕函謌舌〒髢｢謨ｰ繧ｳ繝｡繝ｳ繝医ｒ謖ｿ蜈･縲・
- list_tail 繧・Option<i32> 霑泌唆縺ｫ螟画峩縺励�〕ist_get 縺ｮ襍ｰ譟ｻ繧・unit 縺ｫ縺ｪ繧九ｈ縺・ｪｿ謨ｴ・医ョ繝舌ャ繧ｰ蜃ｺ蜉帙ｂ蜑企勁・峨�・
- stdlib/tests/list.nepl 繧・list_tail 縺ｮ Option 莉墓ｧ倥↓蜷医ｏ縺帙※譖ｴ譁ｰ縲・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 縺梧・蜉溘�・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (import/resolve 繝・せ繝域僑蜈・
- nepl-core/tests/resolve.rs 縺ｫ default alias・育嶌蟇ｾ/繝代ャ繧ｱ繝ｼ繧ｸ・峨�《elective 谺�關ｽ蜷阪・謇ｱ縺・�［erge open縲」isible map 蜆ｪ蜈磯�・ｽ搾ｼ・ocal/ selective/ open・峨ｒ霑ｽ蜉�縲・
- nepl-core/src/module_graph.rs 縺ｮ unit 繝・せ繝医↓ missing dependency/invalid import/duplicate export/non-pub import/ selective+glob re-export 繧定ｿｽ蜉�縲・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 縺梧・蜉溘�・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (rpn 螳溯｡・+ std/test 菫ｮ豁｣ + 繝・せ繝亥ｮ溯｡・
- examples/rpn.nepl 繧・`printf "3 4 +\n" | cargo run -p nepl-cli -- -i examples/rpn.nepl --target wasi --run` 縺ｧ螳溯｡後＠縲ヽEPL 縺檎ｵ先棡繧定ｿ斐＠縺ｦ邨ゆｺ・☆繧九％縺ｨ繧堤｢ｺ隱阪�・
- stdlib/std/test.nepl 縺ｮ `assert_str_eq` 繧・`if:` 繝悶Ο繝・け蠖｢蠑上↓菫ｮ豁｣縺励�～(trap; ())` 縺ｮ inline 1陦悟ｼ上ｒ謗帝勁縺励※繝代・繧ｵ繧ｨ繝ｩ繝ｼ繧定ｧ｣豸医�・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 縺梧・蜉溘�・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (rpn import + diagnostics)
- examples/rpn.nepl 縺ｮ import 繧呈眠莉墓ｧ假ｼ・#import "..." as *`・峨∈譖ｴ譁ｰ縲・
- loader 縺ｮ parse 縺ｧ繧ｨ繝ｩ繝ｼ險ｺ譁ｭ縺後≠繧句�ｴ蜷医・ CoreError 繧定ｿ斐☆繧医≧縺ｫ縺励�∵ｧ区枚繧ｨ繝ｩ繝ｼ縺悟梛繧ｨ繝ｩ繝ｼ縺ｫ蝓九ｂ繧後↑縺・ｈ縺・ｿｮ豁｣縲・
- CLI 縺ｮ險ｺ譁ｭ陦ｨ遉ｺ縺ｧ繧ｭ繝｣繝ｬ繝・ヨ髟ｷ繧定｡梧忰縺ｫ蜿弱ａ縲∝ｷｨ螟ｧ縺ｪ ^ 縺ｮ蜃ｺ蜉帙ｒ謚大宛縲・
- typecheck 縺ｮ邁｡譏薙し繝槭Μ蜃ｺ蜉帙・ verbose 譎ゅ・縺ｿ陦ｨ遉ｺ縺吶ｋ繧医≧縺ｫ螟画峩縲・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (Windows path canonicalization for tests)
- module_graph 縺ｮ lib 繝・せ繝医〒 path 豈碑ｼ・′ Windows 縺ｮ canonicalize 蟾ｮ蛻・〒螟ｱ謨励☆繧九◆繧√�〉oot path 繧・canonicalize 縺励※豈碑ｼ・☆繧九ｈ縺・ｿｮ豁｣縲・
- resolve.rs 蛛ｴ縺ｮ ModuleGraph 蜿ら・繝・せ繝医ｂ蜷梧ｧ倥↓ canonicalize 繧帝←逕ｨ縺励�√け繝ｭ繧ｹ繝励Λ繝・ヨ繝輔か繝ｼ繝�縺ｧ荳�閾ｴ縺吶ｋ繧医≧縺ｫ縺励◆縲・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 縺後←縺｡繧峨ｂ謌仙粥縲・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (resolve import tests fix)
- nepl-core/tests/resolve.rs 縺ｮ繝・せ繝育畑繧ｽ繝ｼ繧ｹ繧・`:` 繝悶Ο繝・け蠖｢蠑上↓菫ｮ豁｣縺励�｝arser 縺ｮ譛溷ｾ・☆繧九う繝ｳ繝・Φ繝域ｧ矩��縺ｫ蜷医ｏ縺帙◆縲・
- selective glob・・name::*`・峨′ open import 縺ｫ蜿肴丐縺輔ｌ繧九％縺ｨ繧堤｢ｺ隱阪☆繧九ユ繧ｹ繝医ｒ霑ｽ蜉�縲・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 縺後←縺｡繧峨ｂ謌仙粥縲・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (resolve/import test expansion)
- nepl-core/tests/resolve.rs 繧定ｿｽ蜉�螳溯｣・＠縲｝relude 謖・ｻ､縺ｮ隗｣譫舌�［erge clause 菫晄戟縲∥lias/open/selective 縺ｮ隗｣豎ｺ縲｛pen import 縺ｮ譖匁乂諤ｧ險ｺ譁ｭ縲《td 繝代ャ繧ｱ繝ｼ繧ｸ隗｣豎ｺ縺ｮ繝・せ繝医ｒ霑ｽ蜉�縲・
- nepl-core/tests/neplg2.rs 縺ｫ prelude/import/merge 謖・ｻ､縺ｮ蜿礼炊遒ｺ隱阪ユ繧ｹ繝医ｒ霑ｽ蜉�縲・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 縺後←縺｡繧峨ｂ謌仙粥縲・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (tests import syntax migration)
- nepl-core/tests 縺ｨ stdlib 驟堺ｸ九・ #import/#use 繧呈眠莉墓ｧ假ｼ・#import "..." as *`・峨∈邨ｱ荳�縺励�・use 繧帝勁蜴ｻ縺励◆縲・
- loader_cycle 縺ｮ繝・せ繝医・ `#import "./a"`/`#import "./b"` 縺ｫ螟画峩縺励※逶ｸ蟇ｾ import 縺ｮ莉墓ｧ倥↓蜷医ｏ縺帙◆縲・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 縺後←縺｡繧峨ｂ謌仙粥縲・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (selective re-export test)
- module_graph 縺ｮ pub selective re-export 縺ｮ謖吝虚繧堤｢ｺ隱阪☆繧九ユ繧ｹ繝医ｒ霑ｽ蜉�・・lias 縺ｮ縺ｿ蜈ｬ髢九＆繧後�∝・蜷阪ｄ譛ｪ驕ｸ謚槭・蜈ｬ髢矩�・岼縺ｯ蜀阪お繧ｯ繧ｹ繝昴・繝医＆繧後↑縺・％縺ｨ繧呈､懆ｨｼ・峨�・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 縺後←縺｡繧峨ｂ謌仙粥縲・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (pub import selective re-export)
- build_exports 縺・ImportClause::Selective 繧定�・・縺励�｝ub import 縺ｮ蜀阪お繧ｯ繧ｹ繝昴・繝育ｯ・峇繧・selective 縺ｫ髯仙ｮ壹〒縺阪ｋ繧医≧縺ｫ縺励◆・・lob 縺ｯ蜈ｨ莉ｶ蜀阪お繧ｯ繧ｹ繝昴・繝域桶縺・ｼ峨�・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 縺後←縺｡繧峨ｂ謌仙粥縲・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (module_graph import clause)
- module_graph 縺ｮ import/deps 縺ｫ ImportClause 繧剃ｿ晄戟縺吶ｋ繧医≧縺ｫ縺励�〉esolve 縺・AST 縺ｧ縺ｯ縺ｪ縺・ModuleGraph 縺ｮ諠・�ｱ縺九ｉ import 蜿･繧貞盾辣ｧ縺吶ｋ蠖｢縺ｸ螟画峩縲・
- resolve 縺ｮ import 襍ｰ譟ｻ繧呈紛逅・＠縲‥eps 縺ｮ clause 繧堤峩謗･菴ｿ縺｣縺ｦ alias/open/selective/merge 繧呈ｧ狗ｯ峨�・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 縺後←縺｡繧峨ｂ謌仙粥縲・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (pub #import / pub item)
- lexer 縺ｧ `pub #import` 繧定ｪ崎ｭ倥＠縲～#import pub ...` 縺ｸ譖ｸ縺肴鋤縺医ｋ蜃ｦ逅・ｒ霑ｽ蜉�・・pub` 蜑咲ｽｮ縺ｮ繝・ぅ繝ｬ繧ｯ繝・ぅ繝悶・ #import 縺ｮ縺ｿ險ｱ蜿ｯ・峨�・
- parser 縺ｧ `pub fn/struct/enum/trait/impl` 繧偵ヨ繝・・繝ｬ繝吶Ν縺ｧ隗｣驥医〒縺阪ｋ繧医≧縺ｫ縺励�～pub` 縺悟・鬆ｭ縺ｫ譚･縺ｦ繧よｭ｣縺励￥螳夂ｾｩ繧定ｪｭ繧√ｋ繧医≧縺ｫ縺励◆縲・
- 繝・せ繝・ `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 縺後←縺｡繧峨ｂ謌仙粥縲・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (rewrite plan doc)
- doc/rewrite_plan.md 繧堤樟陦後さ繝ｼ繝臥｢ｺ隱阪↓蝓ｺ縺･縺・※諡｡蜈・＠縲∝ｾ梧婿莠呈鋤縺ｪ縺励・險ｭ險域嶌+螳溯｣・ｨ育判譖ｸ縺ｨ縺励※謨ｴ逅・＠縺滂ｼ医Δ繧ｸ繝･繝ｼ繝ｫID/manifest縲（mport clause縲｝relude縲∝錐蜑崎ｧ｣豎ｺ蜆ｪ蜈磯�・ｽ阪�∝梛謗ｨ隲・蜊倡嶌蛹悶�仝ASM ABI縲，LI/stdlib蠅・阜縲∝ｮ溯｣・Ο繝ｼ繝峨・繝・・縲√ユ繧ｹ繝域婿驥晢ｼ峨�・
- 迴ｾ陦後ヱ繧､繝励Λ繧､繝ｳ縺ｯ loader 縺ｮ AST 繧ｹ繝励Λ繧､繧ｹ譁ｹ蠑上・縺ｾ縺ｾ縺ｧ縲［odule_graph/resolve 縺ｮ螳溯｣・・譛ｪ邨ｱ蜷医〒縺ゅｋ轤ｹ繧定ｨ育判蜀・↓譏手ｨ倥�・
- plan.md 縺ｫ縺ｯ manifest/譁ｰimport譁・ｳ・prelude/merge縺ｮ莉墓ｧ倥ｄ CLI/ABI 蠅・阜縺ｮ謨ｴ逅・′譛ｪ險倩ｼ峨・縺溘ａ縲∬ｿｽ險倥′蠢・ｦ√�・
- 繝・せ繝・ 莉･蜑阪・ `module_graph::tests::builds_simple_graph_and_exports` 縺・unknown token 縺ｧ螟ｱ謨励＠縺ｦ縺・◆縺後�～pub #import`/`pub fn` 蟇ｾ蠢懷ｾ後↓ `cargo test` 繧よ・蜉溘�・

## 逶ｴ霑代・螳溯｣・し繝槭Μ
- 譁・ｭ怜・繝ｪ繝・Λ繝ｫ縺ｨ蝙・`str` 繧定ｿｽ蜉�縺励�√ョ繝ｼ繧ｿ繧ｻ繧ｯ繧ｷ繝ｧ繝ｳ縺ｫ `[len][bytes]` 縺ｧ驟咲ｽｮ縺励※蟶ｸ譎ゅΓ繝｢繝ｪ繧偵お繧ｯ繧ｹ繝昴・繝医☆繧句ｽ｢縺ｫ邨ｱ荳�縲・
- `#extern` 縺ｧ螟夜Κ髢｢謨ｰ繧貞ｮ｣險�蜿ｯ閭ｽ縺ｫ縺励�《tdlib 縺九ｉ `print` / `print_i32` 繧呈署萓帙☆繧区ｧ区・縺ｫ邨ｱ荳�縲ゅン繝ｫ繝医う繝ｳ髢｢謨ｰ縺ｯ謦､蟒・�・
- CLI: `--target wasm|wasi` 縺ｫ蟇ｾ蠢懶ｼ・asi 縺・wasm 繧貞桁蜷ｫ・峨�Ａ--run` 縺�縺代〒繧ょｮ溯｡悟庄縲ゅさ繝ｳ繝代う繝ｫ螟ｱ謨玲凾縺ｫ SourceMap 莉倥″險ｺ譁ｭ繧貞・蜉帙�・
- Loader/SourceMap 繧貞ｰ主・縺励�（mport/include 縺ｧ FileId/Span 繧剃ｿ晄戟縺励◆縺ｾ縺ｾ螟壹ヵ繧｡繧､繝ｫ繧堤ｵｱ蜷医�・
- 繝代う繝玲ｼ皮ｮ怜ｭ・`|>` 繧定ｿｽ蜉�縲ゅせ繧ｿ繝・け繝医ャ繝励ｒ谺｡縺ｮ蜻ｼ縺ｳ蜃ｺ縺励・隨ｬ1蠑墓焚縺ｫ豕ｨ蜈･縺吶ｋ莉墓ｧ倥〒縲〕exer/parser/typecheck 縺ｾ縺ｧ螳溯｣・ｸ医∩縲・
- `:` 繝悶Ο繝・け縺ｨ `;` 縺ｮ蝙区､懈渊繧定ｪｿ謨ｴ縺励�ゞnit 遐ｴ譽・ｄ while 縺ｮ stack 豺ｱ縺墓､懆ｨｼ繧呈隼蝟・�・
- stdlib: math/mem/string/result/option/list/stdio 繧定ｿｽ蜉�繝ｻ譖ｴ譁ｰ縲Ｎem 縺ｯ raw wasm縲《tring/result/option 縺ｯ繧ｿ繧ｰ莉倥￠繝昴う繝ｳ繧ｿ陦ｨ迴ｾ縲《tdio 縺ｯ WASI fd_write 蜑肴署縲・
- `#target wasm|wasi` 繧偵ョ繧｣繝ｬ繧ｯ繝・ぅ繝悶→縺励※霑ｽ蜉�縲・LI 縺後ち繝ｼ繧ｲ繝・ヨ繧呈欠螳壹＠縺ｪ縺・�ｴ蜷医・ #target 繧偵ョ繝輔か繝ｫ繝医↓逕ｨ縺・�∬､・焚 #target 縺ｯ險ｺ譁ｭ繧ｨ繝ｩ繝ｼ縺ｫ縺励◆縲Ｘasi 蜷ｫ譛峨Ν繝ｼ繝ｫ縺ｯ蠕捺擂騾壹ｊ縲・
- stdlib/std/stdio 繧・WASI `fd_write` 螳溯｣・↓鄂ｮ縺肴鋤縺医�‘nv 萓晏ｭ倥ｒ謗帝勁縲Ｑrint_i32 縺ｯ from_i32 竊・fd_write 縺ｧ蜃ｺ蜉帙�・
- 蝙区ｳｨ驥医・縲梧￡遲蛾未謨ｰ縲阪す繝ｧ繝ｼ繝医き繝・ヨ繧貞炎髯､縺励�∥scription 縺ｮ縺ｿ縺ｧ謇ｱ縺・燕謠舌↓謠・∴縺溘�Ａ|>`+豕ｨ驥医・蝗槭ｊ縺ｮ繝・せ繝医ｒ霑ｽ蜉�縲・
- std/mem.alloc 繧定ｦ∵ｱゅし繧､繧ｺ縺九ｉ邂怜・縺励◆繝壹・繧ｸ謨ｰ縺ｧ memory.grow 縺吶ｋ蠖｢縺ｫ縺励�∝崋螳・繝壹・繧ｸ謌宣聞繧定ｧ｣豸茨ｼ医◆縺�縺励・繝ｼ繧ｸ蠅・阜繧｢繝ｭ繧ｱ繝ｼ繧ｿ縺ｮ縺ｾ縺ｾ・峨�・
- CLI 縺ｮ target 繝輔Λ繧ｰ繧堤怐逡･蜿ｯ閭ｽ縺ｫ縺励�・target / stdio 閾ｪ蜍・wasi 譏・�ｼ縺ｨ謨ｴ蜷医☆繧九ｈ縺・↓縺励◆縲・
- 繝・せ繝郁ｿｽ蜉�: #target wasi 繝・ヵ繧ｩ繝ｫ繝亥虚菴懊�・㍾隍・#target 繧ｨ繝ｩ繝ｼ縲｝ipe+蝙区ｳｨ驥医・謌仙粥繧ｱ繝ｼ繧ｹ縲・
- 險�隱槭↓ struct/enum/match 繧定ｿｽ蜉�縲Ｆnum/struct 繧・TypeCtx 縺ｫ逋ｻ骭ｲ縺励�√さ繝ｳ繧ｹ繝医Λ繧ｯ繧ｿ繧定・蜍輔ヰ繧､繝ｳ繝会ｼ・Type::Variant` / `StructName`・峨�Ｎatch 縺ｯ邯ｲ鄒・�ｧ繝√ぉ繝・け縺ｨ蝙区紛蜷医メ繧ｧ繝・け繧定｡後≧縲・
- Option/Result 繧・enum 繝吶・繧ｹ縺ｫ蜀榊ｮ溯｣・ｼ・ptionI32/ResultI32・峨�Ｔtring/find/to_i32/list/get 縺ｪ縺ｩ繧・Result/Option 霑泌唆縺ｫ蟾ｮ縺玲崛縺医�Ｍist 縺ｮ get 縺ｯ ResultI32 縺ｧ蠅・阜繧ｨ繝ｩ繝ｼ繧定ｿ斐☆縲・
- codegen 縺ｫ enum/struct 繧ｳ繝ｳ繧ｹ繝医Λ繧ｯ繧ｿ縺ｨ match 繧定ｿｽ蜉�・・untime 陦ｨ迴ｾ縺ｯ [tag][payload]/讒矩��菴薙ヵ繧｣繝ｼ繝ｫ繝峨ｒ linear memory 荳翫↓遒ｺ菫昴＠縲《td/mem.alloc 蜻ｼ縺ｳ蜃ｺ縺励ｒ蜑肴署・峨�・
- pipe 縺ｮ豕ｨ蜈･繧ｿ繧､繝溘Φ繧ｰ繧定ｪｿ謨ｴ縺励�∝梛豕ｨ驥・`<T>` 繧呈検繧薙〒繧・`|>` 縺梧ｭ｣縺励￥谺｡縺ｮ callable 縺ｫ豕ｨ蜈･縺輔ｌ繧九ｈ縺・↓縺励◆縲りｿｽ蜉�繝・せ繝医〒遒ｺ隱阪�・
- Loader 縺ｮ蠕ｪ迺ｰ import 讀懷・繝・せ繝医ｒ霑ｽ蜉�・・emp 繝・ぅ繝ｬ繧ｯ繝医Μ縺ｫ a.nepl/b.nepl 繧堤函謌舌＠繝ｭ繝ｼ繝峨〒繧ｨ繝ｩ繝ｼ繧堤｢ｺ隱搾ｼ峨�・

## plan.md 縺ｨ縺ｮ荵夜屬繝ｻ豕ｨ諢冗せ
- `#target`: 繝・ぅ繝ｬ繧ｯ繝・ぅ繝悶→縺励※縺ｯ螳溯｣・ｸ医∩縺�縺後�｝lan.md 縺ｫ縺ｯ譛ｪ險倩ｼ峨�ゅお繝ｳ繝医Μ繝ｼ繝輔ぃ繧､繝ｫ莉･螟悶↓譖ｸ縺九ｌ縺溷�ｴ蜷医・謇ｱ縺・↑縺ｩ莉墓ｧ俶・險倥′蠢・ｦ√�・
- 蝙区ｳｨ驥・`<T>`: 諱堤ｭ蛾未謨ｰ繧ｷ繝ｧ繝ｼ繝医き繝・ヨ縺ｯ蜑企勁縺励◆縺後�｝lan.md 縺ｫ縺ｯ縲碁未謨ｰ縺ｨ隕句★縺吶�阪→縺ゅｋ縺ｮ縺ｧ險倩ｿｰ繧呈峩譁ｰ縺吶ｋ蠢・ｦ√≠繧翫�・
- stdlib/stdio: WASI `fd_write` 螳溯｣・↓鄂ｮ縺肴鋤縺域ｸ医∩縲Ｘasm 縺ｧ import 縺励◆髫帙・蟆ら畑險ｺ譁ｭ縺ｯ縺ｾ縺�辟｡縺・・縺ｧ縲√お繝ｩ繝ｼ繝｡繝・そ繝ｼ繧ｸ謾ｹ蝟・・菴吝慍縺ゅｊ縲・
- stdlib/mem.alloc: 繧ｵ繧､繧ｺ縺ｫ蠢懊§縺溘・繝ｼ繧ｸ謌宣聞縺ｫ菫ｮ豁｣縺励◆縺後�√・繝ｼ繧ｸ蠅・阜繧｢繝ｭ繧ｱ繝ｼ繧ｿ縺ｮ縺ｾ縺ｾ縲らｴｰ邊貞ｺｦ邂｡逅・ｄ free 縺ｯ譛ｪ蟇ｾ蠢懊�・
- Option/Result/list: enum/match 縺檎┌縺・◆繧√ち繧ｰ莉倥″繝昴う繝ｳ繧ｿ縺ｮ證ｫ螳壼ｮ溯｣・�ょ梛繧ｷ繧ｹ繝・Β邨ｱ蜷医ｄ螟夂嶌蛹悶・譛ｪ逹�謇九�Ｍist 縺ｯ i32 蝗ｺ螳壹〒 get 縺ｮ遽・峇螟冶ｨｺ譁ｭ縺ｪ縺励�・

## 霑ｽ蜉�縺ｧ豌嶺ｻ倥＞縺溘％縺ｨ
- Loader 縺ｯ FileId/Span 繧剃ｿ晄戟縺励※ diagnostics 縺ｫ豢ｻ逕ｨ縺ｧ縺阪※縺・ｋ縲・include/#import 縺ｯ荳�蠎ｦ縺阪ｊ繝ｭ繝ｼ繝峨〒蠕ｪ迺ｰ讀懷・縺ゅｊ縲・
- 繧ｳ繝ｼ繝臥函謌舌・ wasm 縺ｮ縺ｿ縲・ompileTarget::allows 縺ｯ wasi 縺・wasm 繧貞桁蜷ｫ縺吶ｋ蠖｢縺ｧ gate 蛻､螳壹ｒ螳溯｣・�・

# 2026-01-23 菴懈･ｭ繝｡繝｢
- Rust 繝・・繝ｫ繝√ぉ繧､繝ｳ繧・rustup 縺ｧ蟆主・縺励�∽ｾ晏ｭ倥け繝ｬ繝ｼ繝医ｒ蜿門ｾ励〒縺阪ｋ繧医≧縺ｫ縺励◆縲・
- #if 髢｢騾｣縺ｮ unknown token 繧定ｧ｣豸医☆繧九◆繧・lexer 縺ｮ `* >` / `- >` 繧・Arrow 縺ｨ縺励※險ｱ蜿ｯ縺吶ｋ繧医≧邱ｩ蜥後＠縺溘�・
- stdlib 縺ｮ讒狗ｯ蛾�比ｸｭ繧ｳ繝ｼ繝峨′螟壽焚繧ｳ繝ｳ繝代う繝ｫ繧貞｡槭＞縺ｧ縺・◆縺溘ａ縲∽ｸ�譎ら噪縺ｫ std/string繝ｻstd/list繝ｻstd/stdio 繧呈怙蟆乗ｩ溯・縺ｮ繧ｹ繧ｿ繝門ｮ溯｣・↓蟾ｮ縺玲崛縺茨ｼ・ption.unwrap_or 繧貞炎髯､縺励※驥崎､・ｧ｣豸茨ｼ峨�・
- enum 繧ｳ繝ｳ繧ｹ繝医Λ繧ｯ繧ｿ縺ｮ codegen 繧剃ｿｮ豁｣・・ayload store 縺ｮ繧ｪ繝壹Λ繝ｳ繝蛾�・→縲∫ｵ先棡繝昴う繝ｳ繧ｿ繧偵せ繧ｿ繝・け縺ｫ谿九☆繧医≧縺ｫ螟画峩・峨�ゅ％繧後↓繧医ｊ Option::Some/None 縺梧ｭ｣縺励￥蛟､繧定ｿ斐＠縲～match_option_some_returns_value` 縺碁�夐℃縲・
- std/list.get 縺ｯ蠅・阜螟悶ｒ蟶ｸ縺ｫ `ResultI32::Err 1` 縺ｧ霑斐☆蜊倡ｴ泌ｮ溯｣・↓縺励�√せ繧ｿ繝・け荳肴紛蜷医・險ｺ譁ｭ繧定ｧ｣豸医�ら樟迥ｶ in-bounds 蜿門ｾ励・譛ｪ蟇ｾ蠢懊□縺後ユ繧ｹ繝域Φ螳夲ｼ・OB 繧ｨ繝ｩ繝ｼ・峨↓縺ｯ蜷郁・縲・
- 迴ｾ蝨ｨ `cargo test` 縺ｯ 23/23 縺吶∋縺ｦ謌仙粥縲よｮ玖ｪｲ鬘後・ stdlib 讖溯・縺ｮ閧我ｻ倥￠・・ist.get 縺ｮ豁｣螳溯｣・�∵枚蟄怜・/繧ｪ繝励す繝ｧ繝ｳ縺ｮ豎守畑蛹悶↑縺ｩ・峨�・

## 莉雁ｾ後・蟇ｾ蠢懈｡茨ｼ亥ｮ溯｣・・縺ｾ縺�縺励↑縺・ｼ・
- `#target wasi|wasm` 繧偵ョ繧｣繝ｬ繧ｯ繝・ぅ繝悶→縺励※霑ｽ蜉�縺励�√ヵ繧｡繧､繝ｫ蜀・・繝・ヵ繧ｩ繝ｫ繝医ち繝ｼ繧ｲ繝・ヨ繧呈ｱｺ螳夲ｼ・LI 謖・ｮ壹′縺ゅｌ縺ｰ縺昴■繧峨ｒ蜆ｪ蜈茨ｼ峨�Ａ#if[target=...]` 隧穂ｾ｡縺ｫ繧ゆｽｿ逕ｨ縲・
- 蝙区ｳｨ驥医・蜿､縺・￡遲蛾未謨ｰ迚ｹ萓九ｒ謦､蜴ｻ縺励�∵ｳｨ驥医・讒区枚隕∫ｴ�縺ｨ縺励※縺ｮ縺ｿ謇ｱ縺・葎繧剃ｻ墓ｧ倥↓譏手ｨ倥�・
- stdio 繧・WASI fd_write 螳溯｣・↓謌ｻ縺呻ｼ上ｂ縺励￥縺ｯ wasm target 縺ｧ import 縺輔ｌ縺溷�ｴ蜷医↓繧ｳ繝ｳ繝代う繝ｫ譎ゅお繝ｩ繝ｼ繧貞・縺吶�・
- mem.alloc 縺ｮ size 蟇ｾ蠢懊→繝壹・繧ｸ蜀榊茜逕ｨ縲〕ist 縺ｮ螟夂嶌蛹悶・蠅・阜繝√ぉ繝・け蠑ｷ蛹悶�＾ption/Result 繧・enum/match 騾｣謳ｺ縺ｸ遘ｻ陦後�・

# 2026-01-30 菴懈･ｭ繝｡繝｢
- stdlib/std/string.nepl 縺ｮ to_i32 蜀・〒 if: 繝悶Ο繝・け縺ｫ隱､縺｣縺ｦ if eq ok 1: / else: 縺梧ｷｷ蜈･縺吶ｋ繧､繝ｳ繝・Φ繝医↓縺ｪ縺｣縺ｦ縺翫ｊ縲（f-layout 隗｣譫舌′ "too many expressions" 縺ｫ縺ｪ繧狗憾諷九□縺｣縺溘◆繧√�（f eq ok 1: 繝悶Ο繝・け繧・谿ｵ繝・ョ繝ｳ繝医＠縲‘lse 繝悶Ο繝・け縺ｮ繧､繝ｳ繝・Φ繝医ｒ謨ｴ縺医※ if-layout 縺梧ｭ｣縺励￥蛻・ｧ｣縺輔ｌ繧九ｈ縺・ｿｮ豁｣縲・
- 縺薙ｌ縺ｫ繧医ｊ std/string 縺ｮ cond/then/else 譛ｪ螳夂ｾｩ繧ｨ繝ｩ繝ｼ縺ｨ block stack 繧ｨ繝ｩ繝ｼ縺瑚ｧ｣豸医�Ｄargo test 縺ｯ蜈ｨ莉ｶ騾夐℃縲‘xamples/counter.nepl 繧・wasi 螳溯｡後＠縺ｦ繧ょｮ瑚ｵｰ縺吶ｋ縺薙→繧堤｢ｺ隱阪�・
- 譁・ｭ怜・繝ｪ繝・Λ繝ｫ縺・allocator 縺ｮ繝｡繧ｿ鬆伜沺縺ｨ陦晉ｪ√＠縺ｦ縺・◆縺溘ａ縲…odegen_wasm 縺ｮ譁・ｭ怜・驟咲ｽｮ髢句ｧ九が繝輔そ繝・ヨ繧・8 繝舌う繝茨ｼ・eap_ptr + free_list_head・峨↓螟画峩縺励�‥ata section 縺ｧ free_list_head=0 繧呈・遉ｺ縲ゆｽｵ縺帙※ data section 繧貞ｸｸ縺ｫ蜃ｺ蜉帙＠縺ｦ heap_ptr 繧貞・譛溷喧縺吶ｋ繧医≧菫ｮ豁｣縲・

# 2026-02-01 if/while 繝・せ繝育┌髯舌Ν繝ｼ繝怜ｯｾ蠢・
## 蝠城｡檎匱隕・
- if繝・せ繝医′16GB莉･荳翫・繝｡繝｢繝ｪ菴ｿ逕ｨ縺ｨ縺ｪ繧翫�∝ｮ溯｡後′蛛懈ｭ｢縺吶ｋ辟｡髯舌Ν繝ｼ繝怜撫鬘後ｒ逋ｺ隕九�・
- 繝代・繧ｵ繝ｼ蛛ｴ縺ｯ`if` 繝悶Ο繝・け蛻・ｧ｣縺ｧ豁｣蟶ｸ縺ｫ蜍穂ｽ懊＠縺ｦ縺・ｋ・医ユ繧ｹ繝磯�夐℃遒ｺ隱搾ｼ峨�・
- 辟｡髯舌Ν繝ｼ繝励・繧ｿ繧､繝励メ繧ｧ繝・け谿ｵ髫弱〒逋ｺ逕溘＠縺ｦ縺・ｋ讓｡讒倥�・

## 蜴溷屏迚ｹ螳壹→菫ｮ豁｣
- `apply_function()` 縺ｮ `if` 繧ｱ繝ｼ繧ｹ縺ｧ縲・未謨ｰ蝙・`(bool, T, T) -> T` 縺ｮ `result` 蝙句､画焚縺檎ｵｱ荳�縺輔ｌ縺ｦ縺・↑縺九▲縺溘�・
- 2縺､縺ｮ繝悶Λ繝ｳ繝∝梛繧堤ｵｱ荳�縺励◆蠕後�√◎縺ｮ邨先棡繧・`result` 蝙句､画焚縺ｫ邨ｱ荳�縺吶ｋ蠢・ｦ√′縺ゅ▲縺溘�・
- 菫ｮ豁｣: `let final_ty = self.ctx.unify(result, t).unwrap_or(t);` 繧定ｿｽ蜉�縺励�∫ｵ先棡蝙九ｒ髢｢謨ｰ縺ｮ result 蝙九ヱ繝ｩ繝｡繝ｼ繧ｿ縺ｨ邨ｱ荳�縲・
- 蜷後§縺・`while` 繧ょ酔讒倥・蝠城｡後′縺ゅ▲縺溘◆繧√�～let final_ty = self.ctx.unify(result, self.ctx.unit()).unwrap_or(self.ctx.unit());` 縺ｧ菫ｮ豁｣縲・

## 繝・せ繝亥ｮ溯｡檎ｵ先棡
- 菫ｮ豁｣蠕後�・Κ蛻・噪縺ｫ繝・せ繝医′謌仙粥髢句ｧ具ｼ・蛟九ユ繧ｹ繝育｢ｺ隱・ if_mixed_cond_then_block_else_block 縺ｪ縺ｩ・・
- 谿九ｊ7蛟九・繝・せ繝医〒繝｡繝｢繝ｪ繧ｹ繝代う繧ｯ邯夊｡・
  - 螟ｱ謨励ユ繧ｹ繝・ if_a_returns_expected, if_b_returns_expected, if_c_returns_expected, if_d_returns_expected, if_e_returns_expected, if_f_returns_expected, if_c_variant_lt_condition
  - 縺薙ｌ繧峨・蜈ｨ縺ｦ `#import "std/math"` 縺ｨ `#use std::math::*` 繧貞性繧�

## 谺｡縺ｮ繧ｹ繝・ャ繝・
- 螟ｱ謨励＠縺ｦ縺・ｋ繝・せ繝医・蜈ｱ騾夂せ縺ｯ import/use 繧ｹ繝・・繝医Γ繝ｳ繝・
- 繝ｭ繝ｼ繝�繝ｼ謌悶＞縺ｯ繝｢繝弱Δ繝ｫ繝輔ぃ繧､繧ｼ繝ｼ繧ｷ繝ｧ繝ｳ谿ｵ髫弱〒縺ｮ辟｡髯舌Ν繝ｼ繝励・蜿ｯ閭ｽ諤ｧ繧定ｪｿ譟ｻ荳ｭ

- 縺薙ｌ縺ｫ繧医ｊ WASI 螳溯｡梧凾縺ｮ print・域枚蟄怜・繝ｪ繝・Λ繝ｫ・峨・辟｡蜃ｺ蜉幢ｼ上ざ繝溷・蜉帙′隗｣豸医�Ｔtdout 縺ｮ蝗槫ｸｰ讀懷・逕ｨ縺ｫ `nepl-core/tests/fixtures/stdout.nepl` 繧定ｿｽ蜉�縺励�～nepl-core/tests/stdout.rs` 縺ｨ `run_main_capture_stdout` 繧貞ｮ溯｣・�・
- 譁・ｭ怜・謫堺ｽ懊・繝・せ繝医→縺励※ `nepl-core/tests/stdlib.rs` 縺ｫ len(譁・ｭ怜・繝ｪ繝・Λ繝ｫ) 縺ｨ from_i32竊値en 繧定ｿｽ蜉�縲Ａcargo test -p nepl-core --test stdlib --test stdout` 縺ｧ遒ｺ隱阪�・
- plan2.md 縺ｨ doc/starting_detail.md 縺ｯ繝ｪ繝昴ず繝医Μ蜀・↓蟄伜惠縺励↑縺・◆繧√�∝盾辣ｧ縺ｧ縺阪↑縺・憾諷九・縺ｾ縺ｾ縲・
- stdlib/std/stdio 縺ｫ `println` 繧定ｿｽ蜉�縺励�～print` + 謾ｹ陦梧枚蟄怜・縺ｧ螳溯｣・�Ａprint`/`print_i32` 縺ｯ縺昴・縺ｾ縺ｾ邯ｭ謖√�・
- stdlib/std/stdio 縺ｮ `print_str` 繧・`print` 縺ｫ謾ｹ蜷阪＠縲～println_i32` 繧定ｿｽ蜉�縲Ｔtr 縺ｯ `print`/`println`縲（32 縺ｯ `print_i32`/`println_i32` 繧呈署萓帙☆繧句ｽ｢縺ｫ謨ｴ逅・�・
- `nepl-core/tests/fixtures/println_i32.nepl` 縺ｨ stdout 繝・せ繝医ｒ霑ｽ蜉�縺励�～println_i32` 縺梧隼陦後ｒ蜃ｺ蜉帙☆繧九％縺ｨ繧堤｢ｺ隱阪�・
- examples 縺ｮ騾・・繝ｼ繝ｩ繝ｳ繝芽ｨ俶ｳ暮崕蜊・`examples/rpn.nepl` 繧呈枚蟄怜・繝代・繧ｹ譁ｹ蠑上↓諡｡蠑ｵ縺励�、SCII 繝医・繧ｯ繝ｳ繧定ｵｰ譟ｻ縺励※謨ｰ蛟､/貍皮ｮ怜ｭ舌ｒ蜃ｦ逅・☆繧句ｽ｢縺ｫ譖ｴ譁ｰ縲・
- stdlib/std/stdio 縺九ｉ std/string 縺ｮ import 繧貞､悶＠縲｝rint 縺ｯ譁・ｭ怜・繝倥ャ繝�髟ｷ繧堤峩謗･隱ｭ繧�蠖｢縺ｫ螟画峩縲Ｑrint_i32 縺ｯ蜷御ｸ�繝輔ぃ繧､繝ｫ蜀・〒謨ｰ蛟､竊呈枚蟄怜・螟画鋤繧定｡後＞縲《td/list 縺ｨ縺ｮ `len` 陦晉ｪ√ｒ蝗樣∩縲・
- stdlib/std/stdio 縺ｫ `read_all` 繧定ｿｽ蜉�縺励�仝ASI 縺ｮ fd_read 縺ｧ讓呎ｺ門・蜉帙ｒ蜿悶ｊ霎ｼ繧√ｋ繧医≧縺ｫ縺励◆縲・LI 繝ｩ繝ｳ繧ｿ繧､繝�縺ｫ繧・fd_read 螳溯｣・→ stdin 繝舌ャ繝輔ぃ繧定ｿｽ蜉�縲・
- stdin 縺ｮ蜍穂ｽ懃｢ｺ隱咲畑縺ｫ `nepl-core/tests/stdin.rs` 縺ｨ `nepl-core/tests/fixtures/stdin_echo.nepl` 繧定ｿｽ蜉�縺励�∵律譛ｬ隱槫・蜉帙・繧ｨ繧ｳ繝ｼ繧ゅユ繧ｹ繝医↓蜷ｫ繧√◆縲・
- CLI 縺ｮ fd_read 繧偵が繝ｳ繝・・繝ｳ繝芽ｪｭ縺ｿ霎ｼ縺ｿ縺ｫ螟画峩縺励�∬ｵｷ蜍墓凾縺ｫ stdin 繧・read_to_end 縺励↑縺・％縺ｨ縺ｧ蟇ｾ隧ｱ蜈･蜉帙〒繧ゅヶ繝ｭ繝・け縺励↑縺・ｈ縺・↓隱ｿ謨ｴ縲・
- stdlib/std/stdio 縺ｫ `read_line` 繧定ｿｽ蜉�縺励�ヽEPL 蜷代￠縺ｫ謾ｹ陦後∪縺ｧ縺ｮ隱ｭ縺ｿ蜿悶ｊ繧呈署萓帙�Ｔtdin 繝・せ繝医↓ `stdin_readline.nepl` 縺ｨ譌･譛ｬ隱槭こ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
- examples/rpn.nepl 繧・REPL 蠖｢蠑上↓螟画峩縺励�・陦後＃縺ｨ縺ｮ隧穂ｾ｡縺ｨ繧ｨ繝ｩ繝ｼ繝｡繝・そ繝ｼ繧ｸ陦ｨ遉ｺ縺ｫ蟇ｾ蠢懊�Ａread_line` 繧剃ｽｿ縺・◆繧√�∝ｯｾ隧ｱ蜈･蜉帙〒繧りｩ穂ｾ｡縺ｧ縺阪ｋ繧医≧縺ｫ縺励◆縲・
- examples/rpn.nepl 縺ｫ REPL 菴ｿ縺・婿縺ｮ繝｡繝・そ繝ｼ繧ｸ繧定ｿｽ蜉�縺励�￣owerShell 繝代う繝玲凾縺ｮ BOM 繧堤┌隕悶☆繧狗ｰ｡譏薙せ繧ｭ繝・・蜃ｦ逅・ｒ蜈･繧後※ unknown token 繧貞屓驕ｿ縲・
- stdout 逕ｨ縺ｮ fixture 縺ｨ繝・せ繝医ｒ霑ｽ蜉�縺励�～println` 縺・`\n` 繧貞・蜉帙☆繧九％縺ｨ繧堤｢ｺ隱阪�３EADME 縺ｮ std/stdio 隱ｬ譏弱ｂ `println` 縺ｨ WASI `fd_write` 縺ｫ蜷医ｏ縺帙※譖ｴ譁ｰ縲・
- stdout 繝・せ繝医〒 wasi fd_read 縺ｮ import 譛ｪ謠蝉ｾ帙↓繧医ｊ instantiate 螟ｱ謨励＠縺ｦ縺・◆縺溘ａ縲～nepl-core/tests/harness.rs` 縺ｮ `run_main_capture_stdout` 縺ｫ fd_read 繧ｹ繧ｿ繝悶ｒ霑ｽ蜉�縲Ａcargo test -p nepl-core --test stdin --test stdout` 縺ｯ隴ｦ蜻贋ｻ倥″縺ｧ謌仙粥縺励�～printf '14 5 6 + -' | cargo run -q -- -i examples/rpn.nepl --run --target wasi` 縺ｧ REPL 蜃ｺ蜉帙→邨先棡 3 繧堤｢ｺ隱阪�・
- PowerShell 縺ｮ UTF-16LE 繝代う繝怜・蜉帙〒謨ｰ蛟､縺悟・蜑ｲ縺輔ｌ繧句庄閭ｽ諤ｧ縺ｫ蛯吶∴縲～examples/rpn.nepl` 縺ｮ謨ｰ蛟､繝代・繧ｹ縺ｧ NUL 繝舌う繝医ｒ辟｡隕悶☆繧句・蟯舌ｒ霑ｽ蜉�・・OM 繧ｹ繧ｭ繝・・縺ｨ菴ｵ逕ｨ・峨�・

# 2026-01-30 菴懈･ｭ繝｡繝｢ (繝・せ繝・stdlib)
- stdlib 縺ｫ `std/test` 繧定ｿｽ蜉�縺励�～assert`/`assert_eq_i32`/`assert_str_eq`/`assert_ok_i32`/`assert_err_i32` 繧呈署萓帙�Ａtrap` 縺ｯ `i32.div_s` 繧・0 縺ｧ蜑ｲ繧・#wasm 縺ｧ螳溯｣・＠縲仝ASM 蛛ｴ縺ｧ遒ｺ螳溘↓逡ｰ蟶ｸ邨ゆｺ・☆繧九ｈ縺・↓縺励◆縲・
- `std/string` 縺ｫ `str_eq`・育ｴ皮ｲ句・蟶ｰ・峨ｒ霑ｽ蜉�縺励�～std/test` 蛛ｴ縺ｮ譁・ｭ怜・豈碑ｼ・〒繧ょ酔遲峨Ο繧ｸ繝・け繧剃ｽｿ逕ｨ縲・
- CLI 縺ｫ `nepl test` 繧ｵ繝悶さ繝槭Φ繝峨ｒ霑ｽ蜉�縺励�～stdlib/tests` 驟堺ｸ九・ `.nepl` 繧貞庶髮・＠縺ｦ WASI 縺ｧ螳溯｡後☆繧九ユ繧ｹ繝医Λ繝ｳ繝翫・繧貞ｮ溯｣・�・
- stdlib 繝・せ繝医ｒ `stdlib/tests/{math,string,result,list}.nepl` 縺ｫ霑ｽ蜉�縲ょｼ上・諡ｬ蠑ｧ縺ｯ菴ｿ繧上★蜑咲ｽｮ險俶ｳ輔〒險倩ｿｰ縺励�ヽesult 縺ｮ move 繧帝∩縺代ｋ縺溘ａ蜷御ｸ�蛟､繧貞・逕滓・縺励※讀懆ｨｼ縲・
- `cargo run -p nepl-cli -- test` 縺ｨ `cargo test` 縺碁�壹ｋ縺薙→繧堤｢ｺ隱阪�・
- doc 縺ｫ `doc/testing.md` 繧定ｿｽ蜉�縺励�√ユ繧ｹ繝域ｩ溯・縺ｮ菴ｿ縺・婿縺ｨ stdlib 縺ｮ迴ｾ迥ｶ遽・峇繧呈紛逅・�・

# 2026-01-30 菴懈･ｭ繝｡繝｢ (examples 螳溯｡檎｢ｺ隱・
- examples/counter.nepl 縺ｨ examples/fib.nepl 繧・`#target wasi` 縺ｫ謠・∴縲《td/stdio 縺ｮ蛻ｩ逕ｨ繧呈・遉ｺ縲・
- `cargo run -p nepl-cli -- -i examples/counter.nepl --run --target wasi` 縺ｨ `... fib.nepl ...`縲～printf '14 5 6 + -\n' | ... rpn.nepl ...` 繧貞ｮ溯｡後＠縲∝・蜉帙′豁｣蟶ｸ縺ｧ縺ゅｋ縺薙→繧堤｢ｺ隱阪�・
- `cargo test` 繧貞・螳溯｡後＠縲∝・繝・せ繝医′騾夐℃縺吶ｋ縺薙→繧堤｢ｺ隱阪�・

# 2026-01-30 菴懈･ｭ繝｡繝｢ (螟夂嶌/蜊倡嶌蛹悶・迴ｾ迥ｶ)
- 繝代・繧ｵ縺ｯ fn/enum/struct/trait/impl 縺ｮ蝙九ヱ繝ｩ繝｡繝ｼ繧ｿ螳｣險�縺ｨ蝙矩←逕ｨ `TypeName<...>` 繧貞女逅・＠縲ゝypeCtx 縺ｫ縺ｯ TypeKind::{Function,Enum,Struct} 縺ｮ type_params 縺ｨ TypeKind::Apply 縺後≠繧九�・
- 髢｢謨ｰ蜻ｼ縺ｳ蜃ｺ縺励〒縺ｯ typecheck 縺・type_params 繧・fresh var 縺ｫ instantiate 縺励�∝他縺ｳ蜃ｺ縺怜・縺ｫ type_args 繧呈ｮ九☆縲Ｎonomorphize 縺ｯ FuncRef 縺ｮ type_args 繧偵ｂ縺ｨ縺ｫ髢｢謨ｰ縺�縺大腰逶ｸ蛹悶＠縺ｦ繝槭Φ繧ｰ繝ｫ蜷阪ｒ逕滓・縺吶ｋ縲・
- TypeKind::Apply 縺ｯ unify 縺梧桶繧上★縲〉esolve 繧・match 莉･螟悶〒菴ｿ繧上ｌ縺ｦ縺・↑縺・◆繧√�∝梛豕ｨ驥医ｄ繧ｷ繧ｰ繝阪メ繝｣縺ｧ `Foo<...>` 繧剃ｽｿ縺・→螳溯ｳｪ逧・↓謨ｴ蜷医＠縺ｪ縺・�・
- enum/struct 縺ｮ繧ｳ繝ｳ繧ｹ繝医Λ繧ｯ繧ｿ縺ｯ螳夂ｾｩ蛛ｴ縺ｮ蝙区ュ蝣ｱ繧堤峩謗･菴ｿ縺｣縺ｦ縺翫ｊ縲（nstantiate 縺輔ｌ縺・params/result 繧貞渚譏�縺励↑縺・◆繧∝梛螟画焚縺後げ繝ｭ繝ｼ繝舌Ν縺ｫ譚溽ｸ帙＆繧後ｄ縺吶￥縲√ず繧ｧ繝阪Μ繝・け enum/struct 縺悟ｮ溽畑縺ｫ縺ｪ縺｣縺ｦ縺・↑縺・�・
- stdlib 縺ｮ list/option/result 縺ｯ i32 蝗ｺ螳壹〒縲√ず繧ｧ繝阪Μ繧ｯ繧ｹ縺ｯ譛ｪ蟆主・縲・

## plan.md 縺ｨ縺ｮ蟾ｮ蛻・Γ繝｢ (霑ｽ蜉�)
- plan.md 縺ｫ縺ｯ繝・せ繝亥ｮ溯｡後さ繝槭Φ繝峨ｄ `std/test`/`nepl test` 縺ｮ莉墓ｧ倥′譛ｪ險倩ｼ峨�ゅユ繧ｹ繝郁ｨｭ險医・遶�遶九※繧定ｿｽ蜉�縺吶ｋ蠢・ｦ√′縺ゅｋ縲・
- plan2.md 縺ｨ doc/starting_detail.md 縺ｯ蠑輔″邯壹″繝ｪ繝昴ず繝医Μ蜀・↓蟄伜惠縺励↑縺・◆繧∝盾辣ｧ荳榊庄縲・
- plan.md 縺ｧ縺ｯ縲悟ｮ夂ｾｩ縺ｧ縺ｮ螟夂嶌縺ｯ謇ｱ繧上↑縺・�阪→縺励※縺・ｋ縺後�∝ｮ溯｣・↓縺ｯ type_params 縺ｨ monomorphize 縺悟ｭ伜惠縺吶ｋ縲ゆｻ墓ｧ俶紛蜷医・霑ｽ險倥′蠢・ｦ√�・

# 2026-01-30 菴懈･ｭ繝｡繝｢ (繧ｸ繧ｧ繝阪Μ繧ｯ繧ｹ菫ｮ豁｣)
- 蝙九ヱ繝ｩ繝｡繝ｼ繧ｿ縺ｯ .T 蠖｢蠑上・縺ｿ險ｱ蜿ｯ縺吶ｋ繧医≧縺ｫ parser 繧呈峩譁ｰ縺励�・T> 縺ｯ繧ｨ繝ｩ繝ｼ縺ｫ縺励◆縲・
- Apply 繧・unify 縺ｧ resolve 縺励※ enum/struct 縺ｮ蜈ｷ菴灘梛縺ｨ邨ｱ蜷医〒縺阪ｋ繧医≧縺ｫ縺励�〉esolve 縺ｮ邨先棡縺ｯ蝙句ｼ墓焚繧・type_params 縺ｫ菫晄戟縺吶ｋ繧医≧螟画峩縲・
- enum/struct 繧ｳ繝ｳ繧ｹ繝医Λ繧ｯ繧ｿ縺ｯ instantiate 蠕後・ params/result 繧剃ｽｿ縺・ｈ縺・↓縺励�∝梛螟画焚縺ｮ繧ｰ繝ｭ繝ｼ繝舌Ν譚溽ｸ帙ｒ驕ｿ縺代ｋ蠖｢縺ｫ菫ｮ豁｣縲・
- type_to_string 縺ｯ enum/struct 縺ｮ type_params 繧貞性繧√ｋ繧医≧縺ｫ縺励※蜊倡嶌蛹悶・繝ｳ繧ｰ繝ｫ縺ｮ陦晉ｪ√ｒ驕ｿ縺代◆縲・
- codegen 縺ｧ Apply 繧貞盾辣ｧ蝙九→縺励※謇ｱ縺・�‘num 縺ｮ variant 隗｣豎ｺ繧・Apply 縺ｫ繧ょｯｾ蠢懊�・
- Rust 繝・せ繝・`nepl-core/tests/generics.rs` 繧定ｿｽ蜉�縺励�’n/enum/struct 縺ｮ繧ｸ繧ｧ繝阪Μ繧ｯ繧ｹ縺ｨ繧ｨ繝ｩ繝ｼ繧ｱ繝ｼ繧ｹ繧呈､懆ｨｼ縲・

# 2026-01-30 菴懈･ｭ繝｡繝｢ (繧ｸ繧ｧ繝阪Μ繧ｯ繧ｹ菫ｮ豁｣縺ｮ霑ｽ蜉�)
- parser 縺ｮ繧ｨ繝ｩ繝ｼ險ｺ譁ｭ縺悟・縺ｦ縺・ｋ蝣ｴ蜷医・ compile_wasm 繧貞､ｱ謨励＆縺帙ｋ繧医≧縺ｫ縺励�・T> 繧貞ｮ滄圀縺ｫ繧ｨ繝ｩ繝ｼ謇ｱ縺・↓縺励◆縲・
- Apply 縺ｮ蝙句ｼ墓焚謨ｰ荳堺ｸ�閾ｴ縺ｯ unify 縺ｧ螟ｱ謨励＆縺帙�∝梛豕ｨ驥医・荳堺ｸ�閾ｴ縺ｨ縺励※險ｺ譁ｭ縺輔ｌ繧九ｈ縺・↓縺励◆縲・
- 蝙句ｼ墓焚縺ｯ typecheck 縺ｨ monomorphize 縺ｧ resolve_id 縺ｫ繧医ｊ螳滉ｽ灘梛縺ｸ豁｣隕丞喧縺励�∝腰逶ｸ蛹門ｾ後↓ Var 縺梧ｮ九ｉ縺ｪ縺・ｈ縺・↓縺励◆縲・
- wasm 逕滓・蠕後↓ wasmparser 縺ｧ讀懆ｨｼ縺励�∫┌蜉ｹ wasm 繧定ｨｺ譁ｭ縺ｨ縺励※霑斐☆繧医≧縺ｫ縺励◆縲・

# 2026-01-30 菴懈･ｭ繝｡繝｢ (繧ｸ繧ｧ繝阪Μ繧ｯ繧ｹ菫ｮ豁｣縺ｮ霑ｽ蜉�2)
- 蝙区ｳｨ驥医′譛ｪ驕ｩ逕ｨ縺ｮ縺ｾ縺ｾ let 縺悟・縺ｫ邁｡邏・＆繧後ｋ繧ｱ繝ｼ繧ｹ縺後≠縺｣縺溘◆繧√�｝ending_ascription 縺後≠繧矩俣縺ｯ縺昴・謇句燕縺ｮ髢｢謨ｰ繧堤ｰ｡邏・＠縺ｪ縺・ｈ縺・guarded reduce 繧定ｿｽ蜉�縲・
- type_args 縺ｮ resolve 繧貞ｼ墓焚 unify 蠕後↓陦後≧繧医≧縺ｫ縺励�∝腰逶ｸ蛹悶↓ Var 縺梧ｮ九ｉ縺ｪ縺・ｈ縺・↓菫ｮ豁｣縲・

# 2026-01-30 菴懈･ｭ繝｡繝｢ (繧ｸ繧ｧ繝阪Μ繧ｯ繧ｹ 繝・せ繝域僑蠑ｵ)
- generics.rs 縺ｫ .T 蠢・�医・ enum/struct 螳夂ｾｩ繧ｨ繝ｩ繝ｼ縲｝ayload 縺ｮ i32 貍皮ｮ玲､懆ｨｼ縲∬､・焚蝙九ヱ繝ｩ繝｡繝ｼ繧ｿ髢｢謨ｰ縺ｮ蜊倡嶌蛹悶�∝梛豕ｨ驥井ｸ堺ｸ�閾ｴ縺ｮ繧ｨ繝ｩ繝ｼ繧定ｿｽ蜉�縲・
- 縺輔ｉ縺ｫ縲¨one 縺ｮ蝙区ｱｺ螳壹�∝ｼ墓焚縺ｪ縺励ず繧ｧ繝阪Μ繝・け髢｢謨ｰ縺ｮ蝙区ｱｺ螳壹�√ず繧ｧ繝阪Μ繝・け髢｢謨ｰ縺ｮ蟋碑ｭｲ蜻ｼ縺ｳ蜃ｺ縺励�｝ipe 邨檎罰蜻ｼ縺ｳ蜃ｺ縺励�・蝙九ヱ繝ｩ繝｡繝ｼ繧ｿ enum 縺ｮ match縲∝・繧悟ｭ・Apply 縺ｮ payload繝ｻ縺昴・荳堺ｸ�閾ｴ繧ｨ繝ｩ繝ｼ縲∝酔荳�蝙九ヱ繝ｩ繝｡繝ｼ繧ｿ縺ｮ荳堺ｸ�閾ｴ繧ｨ繝ｩ繝ｼ縲｝ayload 蝙倶ｸ堺ｸ�閾ｴ繧ｨ繝ｩ繝ｼ繧定ｿｽ蜉�縲・
- 霑ｽ蜉�縺ｧ縲√さ繝ｳ繧ｹ繝医Λ繧ｯ繧ｿ縺ｮ蝙区耳隲厄ｼ亥ｼ墓焚菴咲ｽｮ・峨�√ず繧ｧ繝阪Μ繝・け髢｢謨ｰ縺ｧ縺ｮ Pair 讒狗ｯ峨�＾ption::Some 繝ｩ繝・ヱ繝ｼ髢｢謨ｰ縲＾ption<Option<T>> 縺ｮ蜈･繧悟ｭ・match 繧・OK 繧ｱ繝ｼ繧ｹ縺ｨ縺励※霑ｽ蜉�縲・

# 2026-01-31 菴懈･ｭ繝｡繝｢ (繧ｸ繧ｧ繝阪Μ繧ｯ繧ｹ/讒区枚/繧ｳ繝ｼ繝臥函謌・
- if-layout 縺ｮ cond 隴伜挨蟄舌′螟画焚蜷阪→縺励※菴ｿ繧上ｌ繧九こ繝ｼ繧ｹ縺ｫ蟇ｾ蠢懊☆繧九◆繧√�～normalize_then_else` 縺ｧ cond 繧堤┌譚｡莉ｶ縺ｫ豸医＆縺壹�》hen/else 繝槭・繧ｫ繝ｼ縺後≠繧句�ｴ蜷医・縺ｿ髯､蜴ｻ縺吶ｋ繧医≧隱ｿ謨ｴ縲・
- `if cond:` 縺ｮ繧医≧縺ｪ陦梧忰 `:` 蠖｢蠑上〒 cond 縺悟､画焚蜷阪・蝣ｴ蜷医↓ stack 繧ｨ繝ｩ繝ｼ縺悟・縺ｦ縺・◆縺溘ａ縲（f-layout 蛻､螳壹°繧・`if cond:` 縺ｮ迚ｹ萓九ｒ螟悶＠縲…ond 螟画焚繧剃ｿ晄戟縺吶ｋ蠖｢縺ｫ螟画峩縲・
- match 蠑上′蠕檎ｶ壹・陦後ｒ蜷ｸ縺・ｾｼ繧�繧ｱ繝ｼ繧ｹ縺後≠縺｣縺溘◆繧√�～KwMatch` 縺ｧ match 蠑上ｒ隱ｭ縺ｿ霎ｼ繧薙□繧・prefix 隗｣譫舌ｒ謇薙■蛻・ｋ繧医≧縺ｫ菫ｮ豁｣縲・
- wasm codegen 縺ｮ match 縺・2蛻・ｲ仙崋螳壹□縺｣縺溘◆繧√�∽ｻｻ諢丞�具ｼ・蛟倶ｻ･荳奇ｼ峨・蛻・ｲ舌ｒ if 騾｣骼悶〒逕滓・縺吶ｋ繧医≧縺ｫ諡｡蠑ｵ縺励�・繝舌Μ繧｢繝ｳ繝・enum 縺ｮ match 縺ｧ unreachable 縺悟・繧句撫鬘後ｒ隗｣豸医�・
- `generics_multi_type_params_function` 縺ｮ譛溷ｾ・�､縺ｯ if 縺ｮ謖ｯ繧玖・縺・↓蜷医ｏ縺帙※ 3 縺ｫ菫ｮ豁｣・・alse 蛻・ｲ舌・遒ｺ隱搾ｼ峨�・
- `cargo test` 縺ｯ蜈ｨ莉ｶ騾夐℃繧堤｢ｺ隱阪�・
- plan2.md 縺ｨ doc/starting_detail.md 縺ｯ蠑輔″邯壹″繝ｪ繝昴ず繝医Μ蜀・↓蟄伜惠縺励↑縺・◆繧∝盾辣ｧ荳榊庄縲・

# 2026-01-31 菴懈･ｭ繝｡繝｢ (繝・せ繝域紛蜷・
- nepl-core 縺ｮ `list_get_out_of_bounds_err` 繝・せ繝医ｒ迴ｾ陦・stdlib 縺ｫ蜷医ｏ縺帙�～list_nil/list_cons/list_get` 縺ｨ `Option` 縺ｮ `Some/None` 繝槭ャ繝√↓譖ｴ譁ｰ縲・
- `cargo test` 縺ｨ `cargo run -p nepl-cli -- test` 縺ｮ荳｡譁ｹ縺梧・蜉溘☆繧九％縺ｨ繧堤｢ｺ隱阪�・

# 2026-01-31 菴懈･ｭ繝｡繝｢ (繝ｭ繧ｰ謚大宛)
- typecheck/unify/monomorphize/wasm_sig 縺ｮ謌仙粥譎ゅΟ繧ｰ繧貞炎髯､縺励�＾K譎ゅ・ `nepl-cli test` 縺ｮ蜃ｺ蜉帙ｒ蜑頑ｸ帙�・
- `cargo run -p nepl-cli -- test` 縺ｯ繝・せ繝育ｵ先棡縺ｮ縺ｿ陦ｨ遉ｺ縺輔ｌ繧九％縺ｨ繧堤｢ｺ隱搾ｼ・ust 縺ｮ隴ｦ蜻翫・蛻･騾碑｡ｨ遉ｺ・峨�・

# 2026-01-31 菴懈･ｭ繝｡繝｢ (verbose 繝輔Λ繧ｰ)
- `nepl-cli` 縺ｫ `--verbose` 繧定ｿｽ蜉�縺励�∬ｩｳ邏ｰ縺ｪ繧ｳ繝ｳ繝代う繝ｩ繝ｭ繧ｰ繧貞ｿ・ｦ∵凾縺ｮ縺ｿ蜃ｺ蜉帙〒縺阪ｋ繧医≧縺ｫ縺励◆縲・
- `CompileOptions.verbose` 縺ｧ蛻ｶ蠕｡縺励�》ypecheck/unify/monomorphize/wasm_sig 縺ｮ繝ｭ繧ｰ繧偵ヵ繝ｩ繧ｰ騾｣蜍輔↓縺励◆縲・

# 2026-01-31 菴懈･ｭ繝｡繝｢ (繝｡繝｢繝ｪ繧｢繝ｭ繧ｱ繝ｼ繧ｿ)
- `std/mem` 縺ｮ allocator 繧・wasm 繝｢繧ｸ繝･繝ｼ繝ｫ蜀・ｮ溯｣・↓螟画峩縺励�～nepl_alloc` 縺ｮ繝帙せ繝井ｾ晏ｭ倥ｒ髯､蜴ｻ縲・
- free list + bump 菴ｵ逕ｨ縺ｮ邁｡譏・allocator 繧貞ｮ溯｣・＠縲～memory.grow` 縺ｧ諡｡蠑ｵ縲・
- `doc/runtime.md` 縺ｫ WASM/WASI 縺ｮ繧ｿ繝ｼ繧ｲ繝・ヨ譁ｹ驥昴→繝｡繝｢繝ｪ繝ｬ繧､繧｢繧ｦ繝医ｒ霑ｽ蜉�縲・

# 2026-01-31 菴懈･ｭ繝｡繝｢ (nepl_alloc 閾ｪ蜍・import 縺ｮ謦､蜴ｻ)
- 繧ｳ繝ｳ繝代う繝ｩ縺・`nepl_alloc` 繧定・蜍輔〒 extern 縺ｫ霑ｽ蜉�縺吶ｋ蜃ｦ逅・ｒ蜑企勁縺励�仝ASM 逕滓・迚ｩ縺後・繧ｹ繝井ｾ晏ｭ倥・ import 繧呈戟縺溘↑縺・ｈ縺・↓縺励◆縲・
- `alloc`/`dealloc`/`realloc` 縺ｯ `std/mem` 縺ｮ螳夂ｾｩ縺・`#extern` 縺ｫ繧医ｊ隗｣豎ｺ縺輔ｌ繧句燕謠舌↓縺ｪ縺｣縺溘◆繧√�√Δ繧ｸ繝･繝ｼ繝ｫ蛛ｴ縺ｧ `std/mem` 繧・import 縺励※縺・↑縺・�ｴ蜷医・ codegen 縺ｧ繧ｨ繝ｩ繝ｼ縺ｫ縺ｪ繧九�・
- 譌｢蟄倥・ `a.wasm` 縺ｪ縺ｩ縺ｯ蜀阪さ繝ｳ繝代う繝ｫ縺悟ｿ・ｦ・ｼ亥商縺・ヰ繧､繝翫Μ縺ｫ縺ｯ `nepl_alloc` import 縺梧ｮ九ｋ・峨�・
- `alloc` 縺ｪ縺ｩ縺ｮ繝薙Ν繝医う繝ｳ閾ｪ蜍慕匳骭ｲ繧ょ､悶＠縺溘◆繧√�～std/mem` 縺ｮ髢｢謨ｰ螳夂ｾｩ縺後◎縺ｮ縺ｾ縺ｾ菴ｿ逕ｨ縺輔ｌ繧九�Ａalloc` 繧剃ｽｿ縺・さ繝ｼ繝峨・ `std/mem` 繧呈・遉ｺ逧・↓ import 縺吶ｋ蠢・ｦ√′縺ゅｋ縲・

# 2026-01-31 菴懈･ｭ繝｡繝｢ (std/mem 縺ｮ蜉ｹ譫懈ｳｨ驥・
- `std/mem` 縺ｮ `alloc`/`dealloc`/`realloc`/`mem_grow`/`store` 繧・`*` 莉倥″縺ｫ螟画峩縺励�∫ｴ皮ｲ九さ繝ｳ繝・く繧ｹ繝医°繧牙他縺ｹ縺ｪ縺・％縺ｨ繧呈・遉ｺ縺励◆縲・
- 縺薙ｌ縺ｫ繧医ｊ `std/mem` 蜀・Κ縺ｮ `set`/`store_*` 蜻ｼ縺ｳ蜃ｺ縺励′邏皮ｲ矩未謨ｰ謇ｱ縺・↓縺ｪ縺｣縺ｦ縺・◆蝠城｡後ｒ隗｣豸医＠縲～match_arm_local_drop_preserves_return` 縺ｮ螟ｱ謨怜次蝗�繧剃ｿｮ豁｣縺励◆縲・

# 2026-01-31 菴懈･ｭ繝｡繝｢ (monomorphize 縺ｮ繝ｩ繝ｳ繧ｿ繧､繝�髢｢謨ｰ菫晄戟)
- 繧ｨ繝ｳ繝医Μ襍ｷ轤ｹ縺ｮ蜊倡嶌蛹悶〒 `alloc` 縺瑚誠縺｡繧句撫鬘後ｒ驕ｿ縺代ｋ縺溘ａ縲～monomorphize` 縺ｮ蛻晄悄 worklist 縺ｫ `alloc`/`dealloc`/`realloc` 繧定ｿｽ蜉�縺励◆縲・
- enum/struct/tuple 縺ｮ codegen 縺・`alloc` 繧貞他縺ｶ蜑肴署縺ｧ繧ゅ�∵悴蜿ら・縺ｮ `alloc` 縺碁勁蜴ｻ縺輔ｌ縺ｪ縺・ｈ縺・↓縺励◆縲・

# 2026-01-31 菴懈･ｭ繝｡繝｢ (繝・せ繝亥・縺ｮ std/mem 譏守､ｺ)
- enum/struct/tuple 繧剃ｽｿ縺・ユ繧ｹ繝医た繝ｼ繧ｹ縺ｫ `std/mem` 縺ｮ import 繧定ｿｽ蜉�縺励�～alloc` 縺瑚ｧ｣豎ｺ縺輔ｌ繧句燕謠舌ｒ譏守｢ｺ蛹悶＠縺溘�・
- `move_check` 繝・せ繝医・ Loader 邨檎罰縺ｧ compile 縺吶ｋ繧医≧縺ｫ螟画峩縺励�～#import` 繧定ｧ｣豎ｺ縺ｧ縺阪ｋ繧医≧縺ｫ縺励◆縲・

# 2026-01-31 菴懈･ｭ繝｡繝｢ (讓呎ｺ悶お繝ｩ繝ｼ/險ｺ譁ｭ縺ｮ霑ｽ蜉�)
- `std/error` 縺ｨ `std/diag` 繧定ｿｽ蜉�縺励�～ErrorKind`/`Error`/`Span` 縺ｨ邁｡譏薙Ξ繝昴・繝育函謌舌ｒ逕ｨ諢上＠縺溘�・
- `callsite_span` 縺ｮ intrinsic 繧定ｿｽ蜉�縺励�√お繝ｩ繝ｼ縺ｫ蜻ｼ縺ｳ蜃ｺ縺嶺ｽ咲ｽｮ繧剃ｻ倅ｸ弱〒縺阪ｋ繧医≧縺ｫ縺励◆縲・
- `std/string` 縺ｫ `concat`/`concat3` 繧定ｿｽ蜉�縺励�∬ｨｺ譁ｭ譁・ｭ怜・逕滓・縺ｮ譛�菴朱剞繧貞ｮ溯｣・＠縺溘�・

# 2026-01-31 菴懈･ｭ繝｡繝｢ (WASI 繧ｨ繝ｳ繝医Μ繝昴う繝ｳ繝亥ｯｾ蠢・
- codegen_wasm 縺ｧ entry 髢｢謨ｰ縺梧欠螳壹＆繧後※縺・ｋ蝣ｴ蜷医�√◎縺ｮ髢｢謨ｰ繧・`_start` 縺ｨ縺・≧蜷榊燕縺ｧ繧・export 縺吶ｋ繧医≧縺ｫ縺励◆縲・
- 縺薙ｌ縺ｫ繧医ｊ `wasmer run a.wasm` / `wasmtime run a.wasm` 縺ｧ WASI 繧ｳ繝ｳ繝励Λ繧､繧｢繝ｳ繧ｹ縺ｫ蠕薙＞逶ｴ謗･螳溯｡悟庄閭ｽ縺ｫ縲・
- README.md 縺ｫ螟夜Κ WASI 繝ｩ繝ｳ繧ｿ繧､繝�・・asmtime/wasmer・峨〒縺ｮ螳溯｡梧婿豕輔ｒ霑ｽ蜉�縲・

# 2026-01-31 菴懈･ｭ繝｡繝｢ (謨ｰ蛟､貍皮ｮ励・螳悟・蛹・
- stdlib/std/math.nepl 繧貞・髱｢諡｡蠑ｵ縺励�（32/i64/f32/f64 縺ｮ縺吶∋縺ｦ縺ｮ貍皮ｮ玲ｩ溯・繧呈署萓帙�・
- **邂苓｡捺ｼ皮ｮ・*・啾dd/sub/mul/div_s/div_u/rem_s/rem_u・医☆縺ｹ縺ｦ縺ｮ蝙九〒隨ｦ蜿ｷ蛻･縺ｫ謠蝉ｾ幢ｼ・
- **繝薙ャ繝域ｼ皮ｮ・*・啾nd/or/xor/shl/shr_s/shr_u/rotl/rotr/clz/ctz/popcnt・域紛謨ｰ蝙九・縺ｿ・・
- **豬ｮ蜍募ｰ乗焚轤ｹ迚ｹ譛・*・嘖qrt/abs/neg/ceil/floor/trunc/nearest/min/max/copysign・・32/f64・・
- **蝙句､画鋤**・喨32/i64 <-> f32/f64縲∫ｬｦ蜿ｷ莉倥″/隨ｦ蜿ｷ縺ｪ縺怜ｯｾ蠢懊�・｣ｽ蜥悟､画鋤・・runc_sat・・
- **繝薙ャ繝亥・隗｣驥・*・嗷einterpret_i32/f32/i64/f64

# 2026-02-03 菴懈･ｭ繝｡繝｢ (web playground)
- Trunk 縺ｮ `public_url` 繧・`/` 縺ｫ螟画峩縺励�～trunk serve` 縺ｮ繝ｭ繝ｼ繧ｫ繝ｫ驟堺ｿ｡繝代せ繧・`http://127.0.0.1:8080/` 縺ｫ邨ｱ荳�縲・
- `web/index.html` 縺ｫ `vendor` 縺ｮ copy-dir 繧定ｿｽ蜉�縺励�～web/vendor` 繧堤畑諢上＠縺ｦ editor sample 縺ｮ髱咏噪驟榊ｸ・ｒ Trunk 邨檎罰縺ｧ陦後∴繧九ｈ縺・↓縺励◆縲・
- README 縺ｨ doc/web_playground.md 縺ｫ editor sample 縺ｮ蜿門ｾ玲焔鬆・→繝ｭ繝ｼ繧ｫ繝ｫ襍ｷ蜍・URL 繧定ｿｽ險倥�・
- `web/index.html` 縺ｮ CSS/JS 繧・Trunk 邂｡逅・・繧｢繧ｻ繝・ヨ縺ｨ縺励※螳｣險�縺励�～styles.css` 縺ｨ `main.js` 縺・dist 縺ｫ蜃ｺ蜉帙＆繧後ｋ繧医≧縺ｫ隱ｿ謨ｴ縲・
- `web/main.js` 縺ｯ Trunk 縺ｮ `TrunkApplicationStarted` 繧､繝吶Φ繝医→ `window.wasmBindings` 繧貞茜逕ｨ縺励※ wasm-bindgen 逕滓・迚ｩ縺ｫ繧｢繧ｯ繧ｻ繧ｹ縺吶ｋ譁ｹ蠑上↓螟画峩縲・
- 蝓九ａ霎ｼ縺ｿ editor 縺ｯ `web/vendor/editorsample` 縺悟ｭ伜惠縺吶ｋ蝣ｴ蜷医・縺ｿ iframe 縺ｫ隱ｭ縺ｿ霎ｼ縺ｿ縲∝ｭ伜惠縺励↑縺・�ｴ蜷医・繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ textarea 繧剃ｽｿ逕ｨ縺吶ｋ繧医≧縺ｫ螟画峩縲・
- doc/web_playground.md 縺ｫ `public_url` 縺ｨ `serve-base` 縺ｮ髢｢菫ゅｒ霑ｽ險倥＠縲～trunk serve` 縺ｮ繧｢繧ｯ繧ｻ繧ｹ繝代せ縺ｫ髢｢縺吶ｋ豕ｨ諢冗せ繧呈・險倥�・

## plan.md 縺ｨ縺ｮ荵夜屬繝ｻ豕ｨ諢冗せ (霑ｽ蜉�)
- plan.md 縺ｫ web playground 縺ｮ驟堺ｿ｡謇矩�・・譛ｪ險倩ｼ峨・縺溘ａ縲∝ｿ・ｦ√↑繧我ｻ墓ｧ俶ｬ・↓霑ｽ險倥′蠢・ｦ√�・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (kpread UTF-8 BOM 蟇ｾ蠢・
- PowerShell 縺ｮ繝代う繝怜・蜉帙′ UTF-8 BOM (EF BB BF) 繧剃ｻ倅ｸ弱☆繧句�ｴ蜷医�〔pread 縺ｮ `scanner_read_i32` 縺悟・鬆ｭ縺ｮ BOM 繧呈焚蛟､縺ｨ縺励※謇ｱ縺・�・ 繧定ｿ斐＠邯壹￠繧句撫鬘後ｒ遒ｺ隱阪�・
- `scanner_skip_ws` 縺ｫ UTF-8 BOM 縺ｮ繧ｹ繧ｭ繝・・繧定ｿｽ蜉�縺励�∵里蟄倥・ UTF-16 BOM/NULL 繧ｹ繧ｭ繝・・縺ｨ蜷後§菴咲ｽｮ縺ｧ蜃ｦ逅・�・
- 蝗槫ｸｰ繝・せ繝医→縺励※ `nepl-core/tests/fixtures/stdin_kpread_i32.nepl` 繧定ｿｽ蜉�縺励�～stdin_kpread_utf8_bom` 縺ｧ BOM 莉倥″蜈･蜉帙ｒ讀懆ｨｼ縲・
- 蜍穂ｽ懃｢ｺ隱・ `printf '\xEF\xBB\xBF1 3\n' | cargo run -p nepl-cli -- -i examples/abc086_a.tmp.nepl --run`

# 2026-02-03 菴懈･ｭ繝｡繝｢ (譌･譛ｬ隱樊枚蟄怜・縺ｮ stdout)
- 譁・ｭ怜・繝ｪ繝・Λ繝ｫ縺ｮ lexer 縺・UTF-8 繧・1 繝舌う繝医★縺､ `char` 縺ｫ螟画鋤縺励※縺・◆縺溘ａ縲∵律譛ｬ隱槭′ mojibake 縺ｫ縺ｪ繧句撫鬘後ｒ遒ｺ隱阪�・
- 譁・ｭ怜・繝ｪ繝・Λ繝ｫ縺ｮ騾壼ｸｸ譁・ｭ励・隱ｭ縺ｿ蜿悶ｊ繧・UTF-8 `char` 蜊倅ｽ阪↓螟画峩縺励�～i` 繧・`len_utf8` 蛻・�ｲ繧√ｋ繧医≧菫ｮ豁｣縲・
- 蝗槫ｸｰ繝・せ繝医→縺励※ `nepl-core/tests/fixtures/stdout_japanese.nepl` 縺ｨ `stdout_japanese_utf8` 繧定ｿｽ蜉�縲・
- 蜍穂ｽ懃｢ｺ隱・ `cargo run -p nepl-cli -- -i examples/helloworld.nepl --run -o a`

# 2026-02-03 菴懈･ｭ繝｡繝｢ (CLI --run 縺ｮ stdio 繝励Ο繝ｳ繝励ヨ)
- `nepl-cli --run` 縺ｮ WASI `fd_write` 縺・`print!` 縺ｮ縺ｿ縺ｧ flush 縺励※縺翫ｉ縺壹�√・繝ｭ繝ｳ繝励ヨ `"> "` 縺悟・蜉帛ｾ後↓陦ｨ遉ｺ縺輔ｌ繧句撫鬘後ｒ遒ｺ隱阪�・
- `fd_write` 繧・raw bytes 縺ｧ `stdout.write_all` 縺励�∵怙蠕後↓ `flush` 縺吶ｋ繧医≧菫ｮ豁｣縲・
- 蜍穂ｽ懃｢ｺ隱・ `printf "3 5 3\n" | cargo run -p nepl-cli -- -i examples/stdio.nepl --run -o a`

# 2026-02-03 菴懈･ｭ繝｡繝｢ (ANSI 繧ｨ繧ｹ繧ｱ繝ｼ繝怜・蜉・
- 譁・ｭ怜・繝ｪ繝・Λ繝ｫ縺ｮ繧ｨ繧ｹ繧ｱ繝ｼ繝励↓ `\xNN` (hex) 繧定ｿｽ蜉�縺励�～"\x1b[31m"` 縺ｪ縺ｩ ANSI 繧ｨ繧ｹ繧ｱ繝ｼ繝励ｒ逶ｴ謗･譖ｸ縺代ｋ繧医≧縺ｫ縺励◆縲・
- 蝗槫ｸｰ繝・せ繝医→縺励※ `nepl-core/tests/fixtures/stdout_ansi.nepl` 縺ｨ `stdout_ansi_escape` 繧定ｿｽ蜉�縲・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (std/stdio 縺ｮ ANSI 濶ｲ繝倥Ν繝代・)
- `std/stdio` 縺ｫ `ansi_red` 縺ｪ縺ｩ縺ｮ濶ｲ繧ｳ繝ｼ繝蛾未謨ｰ縺ｨ `print_color` / `println_color` 繧定ｿｽ蜉�縲・
- 蝗槫ｸｰ繝・せ繝医→縺励※ `nepl-core/tests/fixtures/stdout_color.nepl` 縺ｨ `stdout_ansi_helpers` 繧定ｿｽ蜉�縲・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (Web playground terminal)
- `nepl-core` 縺ｫ `load_inline_with_provider` 繧定ｿｽ蜉�縺励�∽ｻｮ諠ｳ stdlib 繧ｽ繝ｼ繧ｹ縺九ｉ縺ｮ繝ｭ繝ｼ繝峨ｒ蜿ｯ閭ｽ縺ｫ縺励◆縲・
- `nepl-web` (wasm-bindgen) 繧呈眠險ｭ縺励�√ヶ繝ｩ繧ｦ繧ｶ蜀・〒縺ｮ繧ｳ繝ｳ繝代う繝ｫ縺ｨ stdlib 繝・せ繝亥ｮ溯｡後ｒ謠蝉ｾ帙�・
- `web/` 縺ｫ繧ｿ繝ｼ繝溘リ繝ｫ UI 繧定ｿｽ蜉�縺励�～run`/`test`/`clear` 繧ｳ繝槭Φ繝峨→ stdin 蜈･蜉帙ｒ螳溯｣・�・
- `doc/web_playground.md` 繧定ｿｽ蜉�縺励�仝eb playground 縺ｮ螳溯｡御ｻ墓ｧ倥ｒ謨ｴ逅・�・
- Trunk 0.20 莠呈鋤縺ｮ縺溘ａ縲～web/index.html` 縺ｮ `<link data-trunk>` 縺九ｉ `data-type="wasm-bindgen"` 繧貞炎髯､縲・
- `nepl-web` 縺ｮ `include_str!` 繝代せ繧剃ｿｮ豁｣縺励�～nepl-core` 繝ｭ繝ｼ繝�繝ｼ縺ｫ wasm 蜷代￠縺ｮ繝輔ぃ繧､繝ｫ繧｢繧ｯ繧ｻ繧ｹ謚大宛繧定ｿｽ蜉�縲・
- Web UI 繧・mlang playground 縺ｮ讒区・縺ｫ蜷医ｏ縺帙※謨ｴ逅・＠縲仝AT 蜃ｺ蜉帙ヱ繝阪Ν縺ｨ謫堺ｽ懊・繧ｿ繝ｳ繧定ｿｽ蜉�縲・
- 蠕梧婿莠呈鋤諤ｧ縺ｮ縺溘ａ縲（32 縺ｮ縺ｿ縺ｮ alias 髢｢謨ｰ・・dd/sub/mul/div_s/lt/eq 縺ｪ縺ｩ・峨ｒ謠蝉ｾ帙�・

# 2026-01-31 菴懈･ｭ繝｡繝｢ (stdlib 繝・せ繝医・蜈・ｮ溷喧)
- stdlib/tests 縺ｫ譁ｰ隕上ユ繧ｹ繝医ヵ繧｡繧､繝ｫ繧定ｿｽ蜉�・嗤ption.nepl/cast.nepl/vec.nepl/stack.nepl/error.nepl/diag.nepl
- 譌｢蟄倥ユ繧ｹ繝医ｒ諡｡蠑ｵ・嗄ath/string/result/list 縺ｮ蜷・ユ繧ｹ繝医き繝舌Ξ繝・ず繧貞､ｧ蟷・｢怜刈縲・
- 繝・せ繝亥ｯｾ雎｡・・
  - **option**: is_some/is_none/unwrap/unwrap_or
  - **cast**: bool竊琶32 螟画鋤
  - **vec**: vec_new/push/get/capacity/is_empty
  - **stack**: stack_new/push/pop/peek/len
  - **error**: error_new/蜷・ｨｮ ErrorKind
  - **diag**: kind_str・・rrorKind 竊・譁・ｭ怜・・・
  - **math**: i32/i64 縺ｮ蜈ｨ貍皮ｮ・繝薙ャ繝域ｼ皮ｮ励�∵ｵｮ蜍募ｰ乗焚轤ｹ謫堺ｽ・
  - **string**: len/concat/str_eq/from_i32 縺ｮ諡｡蠑ｵ繝・せ繝・
  - **result**: ok/err/is_ok/is_err/unwrap_or
  - **list**: cons/nil/get/head/tail/reverse/len

# 2026-02-01 菴懈･ｭ繝｡繝｢ (if蠑上・辟｡髯舌Γ繝｢繝ｪ蜑ｲ繧雁ｽ薙※繝舌げ菫ｮ豁｣)
## 蝠城｡悟・譫・
- if 繝・せ繝医〒 15 蛟倶ｸｭ 8 蛟九′謌仙粥縺�縺後�∵ｮ九ｊ 7 蛟九〒繝｡繝｢繝ｪ蜑ｲ繧雁ｽ薙※繧ｨ繝ｩ繝ｼ・・.5GB・臥匱逕・
- **螟ｱ謨励ヱ繧ｿ繝ｼ繝ｳ**: `#import "std/math"` + `#use std::math::*` 繧貞性繧�縺吶∋縺ｦ縺ｮ繝・せ繝医こ繝ｼ繧ｹ
  - `if_a_returns_expected` (繧ｭ繝ｼ繝ｯ繝ｼ繝牙ｽ｢蠑・ `if true 0 1`)
  - `if_b_returns_expected` (繧ｭ繝ｼ繝ｯ繝ｼ繝牙ｽ｢蠑・ `if true then 0 else 1`)
  - `if_c_returns_expected` (繝ｬ繧､繧｢繧ｦ繝亥ｽ｢蠑上�√・繝ｼ繧ｫ繝ｼ縺ｪ縺・
  - 縺昴・莉・`if_d/e/f` 縺ｨ繝舌Μ繧｢繝ｳ繝・

- **謌仙粥繝代ち繝ｼ繝ｳ**: 蜷後§縺・`#import "std/math"` 繧貞性繧�縺後�（f: 繝ｬ繧､繧｢繧ｦ繝亥ｽ｢蠑上〒 role 繝槭・繧ｫ繝ｼ(`cond`/`then`/`else`)繧剃ｽｿ逕ｨ
  - `if_c_variant_cond_keyword` (cond 繝槭・繧ｫ繝ｼ縺ゅｊ)
  - `if_mixed_cond_then_block_else_block` (cond/then/else 繝悶Ο繝・け蠖｢蠑・
  - 縺昴・莉悶Ξ繧､繧｢繧ｦ繝亥ｽ｢蠑上・繝ｼ繧ｫ繝ｼ縺ゅｊ

## 蜴溷屏迚ｹ螳・
- **譬ｹ譛ｬ蜴溷屏縺ｯ typecheck 縺ｮ apply_function 縺ｫ縺翫￠繧・if / while 繝上Φ繝峨Λ蜀・〒 result 蝙句､画焚繧・unify 縺吶ｋ髫帙↓逕溘§縺溷梛縺ｮ蠕ｪ迺ｰ蜿ら・**
- parser 縺ｮ菫ｮ豁｣縺ｫ繧医ｊ莉･荳九・ 2 縺､縺ｮ繝舌げ繧・fix 貂医∩:
  1. 繝槭・繧ｫ繝ｼ縺ｫ inline 蠑上′縺ゅｋ蝣ｴ蜷医�√ヶ繝ｩ繝ｳ繝√′蜊ｳ蠎ｧ縺ｫ finalize 縺輔ｌ縺壹�∝ｾ檎ｶ壹・ positional 陦後→ grouping 縺輔ｌ繧・
  2. 隍・焚繧ｹ繝・・繝医Γ繝ｳ繝・positional 繝悶Λ繝ｳ繝√′蛟句挨繝悶Λ繝ｳ繝√↓ split 縺輔ｌ縺ｪ縺・

- 譁ｰ縺溘↓ typecheck 蜀・・ if/while 繧ｱ繝ｼ繧ｹ縺ｧ result 蝙九→縺ｮ unify 縺ｫ繧医ｊ**辟｡髯仙梛讒矩��**縺檎函謌舌＆繧後※縺・◆

## 菫ｮ豁｣蜀・ｮｹ
1. `typecheck.rs` 陦・2369-2397 (if 繧ｱ繝ｼ繧ｹ):
   - 蜈・ `let final_ty = self.ctx.unify(result, t).unwrap_or(t);`
   - 菫ｮ: `let branch_ty = self.ctx.unify(args[1].ty, args[2].ty).unwrap_or(args[1].ty);` 縺ｮ縺ｿ縺ｧ result 蝙句､画焚縺ｯ菴ｿ逕ｨ縺励↑縺・
   - 逅・罰: result 縺ｯ fresh 蝙句､画焚縺ｧ縲√％繧後→ unify 縺吶ｋ縺ｨ蝙九・蠕ｪ迺ｰ蜿ら・縺檎匱逕溘＠縲［onomorphize 谿ｵ髫弱〒縺ｮ蝙・substitution 縺ｧ exponential explosion

2. `typecheck.rs` 陦・2400-2427 (while 繧ｱ繝ｼ繧ｹ):
   - 蜷梧ｧ倥↓ `self.ctx.unify(result, self.ctx.unit()).unwrap_or(self.ctx.unit())` 繧貞炎髯､
   - 菫ｮ: `self.ctx.unit()` 繧堤峩謗･霑斐☆

3. parser.rs debug 險ｺ譁ｭ縺ｮ蜑企勁:
   - 陦・859-890: if 蠖｢蠑上・繧｢繧､繝・Β繧ｷ繧ｧ繧､繝励ｒ繝�繝ｳ繝励☆繧・diagnostic 繧貞炎髯､
   - 陦・1536-1550: if-layout 繝悶Λ繝ｳ繝∝ｽｹ蜑ｲ諠・�ｱ繝�繝ｳ繝・diagnostic 繧貞炎髯､
   - 陦・1515-1530: marker 譛ｪ讀懷・縺ｮ warning 繧貞炎髯､

## 迥ｶ諷・
- 蜈ｨ if 繝・せ繝・15 蛟九′謌仙粥縺励�∝粋險亥ｮ溯｡梧凾髢・5.12 遘偵〒繧ｳ繝ｳ繝励Μ繝ｼ繝茨ｼ井ｻ･蜑阪・荳�驛ｨ縺ｧ繝｡繝｢繝ｪ蜑ｲ繧雁ｽ薙※繧ｨ繝ｩ繝ｼ・・
- debug 繝輔ぃ繧､繝ｫ蜑企勁貂医∩: `parse_if_debug.rs`縲～compile_if_a.rs`

# 2026-02-03 菴懈･ｭ繝｡繝｢ (if 繝・せ繝亥●豁｢/lexer)
## 蝠城｡檎匱隕・
- if 繝・せ繝医・荳�驛ｨ縺ｧ繧ｳ繝ｳ繝代う繝ｩ縺悟●豁｢縺励�∝ｷｨ螟ｧ繝｡繝｢繝ｪ蜑ｲ繧雁ｽ薙※繧ｨ繝ｩ繝ｼ縺檎匱逕溘�・
- 繝・せ繝亥・縺ｮ `#import`/`#use` 陦後′繝医ャ繝励Ξ繝吶Ν縺ｧ繧､繝ｳ繝・Φ繝医＆繧後※縺・◆縲・

## 蜴溷屏迚ｹ螳壹→菫ｮ豁｣
- lexer 縺後ヨ繝・・繝ｬ繝吶Ν縺ｮ繝・ぅ繝ｬ繧ｯ繝・ぅ繝冶｡後〒繧ゅう繝ｳ繝・Φ繝亥｢怜刈繧・`Indent` 縺ｨ縺励※蜃ｺ蜉帙＠縺ｦ縺励∪縺・�∵Φ螳壼､悶・繝悶Ο繝・け讒矩��縺ｫ縺ｪ縺｣縺ｦ typecheck 縺悟●豁｢縺励※縺・◆縲・
- `expect_indent` 繧定ｿｽ蜉�縺励�∫峩蜑阪・陦梧忰 `:` 縺・`#wasm` 繝悶Ο繝・け縺ｮ譎ゅ・縺ｿ繧､繝ｳ繝・Φ繝亥｢怜刈繧定ｨｱ蜿ｯ縺吶ｋ繧医≧縺ｫ菫ｮ豁｣縲・
- 繝・ぅ繝ｬ繧ｯ繝・ぅ繝冶｡後〒荳肴ｭ｣縺ｪ繧､繝ｳ繝・Φ繝亥｢怜刈縺後≠繧句�ｴ蜷医・繧､繝ｳ繝・Φ繝医ｒ謐ｮ縺育ｽｮ縺阪�√ヨ繝・・繝ｬ繝吶Ν謇ｱ縺・↓蝗ｺ螳壹�・

## 繝・せ繝亥ｮ溯｡檎ｵ先棡
- `cargo test -p nepl-core --test if` 縺碁�夐℃縲・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (謨ｴ謨ｰ繝ｪ繝・Λ繝ｫ/move_check)
## 菫ｮ豁｣蜀・ｮｹ
- 謨ｴ謨ｰ繝ｪ繝・Λ繝ｫ縺ｮ `i32` 螟画鋤縺・overflow 縺ｧ 0 縺ｫ縺ｪ縺｣縺ｦ縺・◆縺溘ａ縲～i128` 縺ｧ繝代・繧ｹ縺励※ `i32` 縺ｫ繝ｩ繝・・縺吶ｋ螳溯｣・↓菫ｮ豁｣縲Ａ0x` 16騾ｲ縺ｫ繧ょｯｾ蠢懊＠縲∫┌蜉ｹ蛟､縺ｯ險ｺ譁ｭ繧貞・縺吶�・
- `Intrinsic::load`/`store` 縺ｮ move_check 繧堤音谿頑桶縺・＠縲√い繝峨Ξ繧ｹ蛛ｴ縺ｯ borrow 縺ｨ縺励※謇ｱ縺・ｈ縺・↓菫ｮ豁｣縲Ａload` 縺ｯ繝ｭ繝ｼ繝牙ｯｾ雎｡蝙九′ Copy 縺ｮ縺ｨ縺・borrow 謇ｱ縺・�～store` 縺ｯ蟶ｸ縺ｫ繧｢繝峨Ξ繧ｹ繧・borrow 縺ｨ縺励※蜃ｦ逅・�・
- `visit_borrow` 縺ｧ `Intrinsic` 縺ｮ蠑墓焚繧貞・蟶ｰ逧・↓ borrow 縺ｨ縺励※謇ｱ縺・�∬ｪ､縺｣縺・move 蛻､螳壹ｒ謚大宛縲・
- Struct/Enum/Apply 縺ｯ Copy 縺ｧ縺ｯ縺ｪ縺・燕謠舌ｒ邯ｭ謖√�・
- `std/vec` 縺ｧ len/cap/data 繧偵Ο繝ｼ繧ｫ繝ｫ縺ｫ菫晄戟縺励�∝酔荳�蛟､縺ｸ縺ｮ隍・焚繧｢繧ｯ繧ｻ繧ｹ縺ｫ繧医ｋ move_check 螟ｱ謨励ｒ蝗樣∩縲・

## 繝・せ繝亥ｮ溯｡檎ｵ先棡
- `cargo run -p nepl-cli -- test` 縺碁�夐℃縲・
- `cargo test` 縺碁�夐℃縲・

## plan.md 縺ｨ縺ｮ蟾ｮ蛻・Γ繝｢ (霑ｽ蜉�)
- 繝医ャ繝励Ξ繝吶Ν縺ｮ繝・ぅ繝ｬ繧ｯ繝・ぅ繝冶｡後・繧､繝ｳ繝・Φ繝域桶縺・ｼ・#wasm` 繝悶Ο繝・け莉･螟悶・蠅怜刈繧堤┌隕悶☆繧倶ｻ墓ｧ假ｼ峨′ plan.md 縺ｫ譛ｪ險倩ｼ峨�・
- 謨ｴ謨ｰ繝ｪ繝・Λ繝ｫ縺ｮ overflow 繝ｫ繝ｼ繝ｫ・・i32` 縺ｸ縺ｮ繝ｩ繝・・・峨→ 16 騾ｲ陦ｨ險倥・莉墓ｧ倥′ plan.md 縺ｫ譛ｪ險倩ｼ峨�・
- move_check 縺ｫ縺翫￠繧・`load`/`store` 縺ｮ borrow 謇ｱ縺・′ plan.md 縺ｫ譛ｪ險倩ｼ峨�・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (CLI 蜃ｺ蜉・emit 諡｡蠑ｵ)
## 菫ｮ豁｣蜀・ｮｹ
- `--emit` 繧定､・焚謖・ｮ壼庄閭ｽ縺ｫ縺励�～wasm`/`wat`/`wat-min`/`all` 繧帝∈謚槭〒縺阪ｋ繧医≧縺ｫ諡｡蠑ｵ縲・
- `--output` 繧偵・繝ｼ繧ｹ繝代せ縺ｨ縺励※謇ｱ縺・�～.wasm`/`.wat`/`.min.wat` 繧呈ｴｾ逕溽函謌舌☆繧九ｈ縺・､画峩縲・
- pretty WAT 縺ｯ `wasmprinter::print_bytes` 縺ｮ蜃ｺ蜉帙ｒ菴ｿ逕ｨ縺励�［inified WAT 縺ｯ縺昴・蜃ｺ蜉帙ｒ遨ｺ逋ｽ蝨ｧ邵ｮ縺励※逕滓・縲・
- CLI 蜃ｺ蜉帙・繝ｦ繝九ャ繝医ユ繧ｹ繝医ｒ霑ｽ蜉�・・mit 隗｣譫舌�∝・蜉帙・繝ｼ繧ｹ蛻､螳壹�［inify縲∝・蜉帙ヵ繧｡繧､繝ｫ逕滓・・峨�・
- `doc/cli.md` 縺ｨ README 縺ｮ CLI 萓九ｒ譖ｴ譁ｰ縲・
- GitHub Actions 縺ｮ `nepl-test.yml` 縺ｫ multi-emit 縺ｮ蜃ｺ蜉帷｢ｺ隱阪せ繝・ャ繝励ｒ霑ｽ蜉�縲・

## 繝・せ繝亥ｮ溯｡檎ｵ先棡
- `cargo test -p nepl-cli`

## plan.md 縺ｨ縺ｮ蟾ｮ蛻・Γ繝｢ (霑ｽ蜉�)
- `--emit` 縺ｮ隍・焚謖・ｮ壹→ `wat-min` 蜃ｺ蜉帙�～--output` 縺ｮ繝吶・繧ｹ繝代せ驕狗畑縺・plan.md 縺ｫ譛ｪ險倩ｼ峨�・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (kpread/abc086_a)
## 菫ｮ豁｣蜀・ｮｹ
- `kp/kpread` 縺ｮ Scanner 繧・i32 繝昴う繝ｳ繧ｿ繝吶・繧ｹ縺ｫ螟画峩縺励�｜uf/len/pos 繧貞崋螳壹が繝輔そ繝・ヨ縺ｧ `load_i32`/`store_i32` 縺吶ｋ螳溯｣・↓螟画峩縲・
- `scanner_*` 縺ｮ蠑墓焚蝙九ｒ `(i32)` 縺ｫ邨ｱ荳�縺励�～scanner_new` 縺ｯ 12 繝舌う繝医・繝倥ャ繝�鬆伜沺縺ｫ buf/len/pos 繧呈�ｼ邏阪☆繧句ｽ｢蠑上↓螟画峩縲・
- `examples/abc086_a.nepl` 縺ｮ Scanner 蝙区ｳｨ驥医ｒ i32 縺ｫ譖ｴ譁ｰ縲・

## 繝・せ繝亥ｮ溯｡檎ｵ先棡
- `printf "1 3" | cargo run -p nepl-cli -- -i examples/abc086_a.nepl --run`

# 2026-02-03 菴懈･ｭ繝｡繝｢ (if[profile])
## 菫ｮ豁｣蜀・ｮｹ
- `#if[profile=debug|release]` 繧・lexer/parser/AST/typecheck 縺ｫ霑ｽ蜉�縺励�√さ繝ｳ繝代う繝ｫ譎ゅ・繝ｭ繝輔ぃ繧､繝ｫ縺ｫ蠢懊§縺ｦ繧ｲ繝ｼ繝医☆繧九ｈ縺・↓縺励◆縲・
- `nepl-core/tests/neplg2.rs` 縺ｫ profile 繧ｲ繝ｼ繝医・繝・せ繝医ｒ霑ｽ蜉�縲・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (profile 繧ｪ繝励す繝ｧ繝ｳ/繝・ヰ繝・げ蜃ｺ蜉・
## 菫ｮ豁｣蜀・ｮｹ
- 繧ｳ繝ｳ繝代う繝ｩ縺ｮ `CompileOptions` 縺ｫ `profile` 繧定ｿｽ蜉�縺励�～#if[profile=debug|release]` 繧・CLI 縺九ｉ蛻ｶ蠕｡縺ｧ縺阪ｋ繧医≧縺ｫ諡｡蠑ｵ縲・
- CLI 縺ｫ `--profile debug|release` 繧定ｿｽ蜉�縺励�∵悴謖・ｮ壽凾縺ｯ繝薙Ν繝画凾縺ｮ繝励Ο繝輔ぃ繧､繝ｫ繧剃ｽｿ逕ｨ縲・
- `std/stdio` 縺ｫ `debug`/`debugln` 繧定ｿｽ蜉�・・ebug 縺ｧ縺ｯ蜃ｺ蜉帙�〉elease 縺ｧ縺ｯ no-op・峨�・
- `std/diag` 縺ｫ `diag_debug_print`/`diag_debug_println` 繧定ｿｽ蜉�縲・
- `README.md` 縺ｨ `doc/cli.md`/`doc/debug.md` 繧呈峩譁ｰ縲・

## 繝・せ繝亥ｮ溯｡檎ｵ先棡
- `cargo test -p nepl-core --test neplg2`

# 2026-02-03 險ｭ險医Γ繝｢ (繝ｪ繝ｩ繧､繝域婿驥昴∪縺ｨ繧・
- `doc/rewrite_plan.md` 繧定ｿｽ蜉�縲ら樟陦悟ｮ溯｣・・繧ｹ繝翫ャ繝励す繝ｧ繝・ヨ縺ｨ隱ｲ鬘後�∝ｾ梧婿莠呈鋤縺ｪ縺励〒縺ｮ蜀崎ｨｭ險医い繝ｼ繧ｭ繝・け繝√Ε/螳溯｣・Ο繝ｼ繝峨・繝・・繧定ｨ倩ｼ峨�・
- 繝｢繧ｸ繝･繝ｼ繝ｫ縺ｯ繝輔ぃ繧､繝ｫ繧ｹ繝励Λ繧､繧ｹ蜑肴署繧偵ｄ繧√�～nepl.toml` 縺ｫ繧医ｋ繝代ャ繧ｱ繝ｼ繧ｸ/萓晏ｭ倡ｮ｡逅・→ `#import ... as {alias|*|{...}|@merge}`縲～pub #import` 縺ｫ繧医ｋ蜀阪お繧ｯ繧ｹ繝昴・繝医ｒ謗｡逕ｨ縺吶ｋ譁ｹ驥昴�・
- 蜷榊燕隗｣豎ｺ縺ｯ DefId 繝吶・繧ｹ縺ｮ莠梧ｮｵ髫趣ｼ亥ｮ夂ｾｩ蜿朱寔竊定ｧ｣豎ｺ・峨�￣relude 譏守､ｺ蛹悶�・∈謚・繧ｪ繝ｼ繝励Φ/繧ｨ繧､繝ｪ繧｢繧ｹ蜆ｪ蜈磯�・ｽ阪ｒ謨ｴ逅・�・
- 蝙九す繧ｹ繝・Β縺ｯ DefId 莉倥″ HIR 縺ｨ蜊倡嶌蛹・(monomorphize) 繧貞・讒狗ｯ峨＠縲｀IR 繧堤ｵ後※ WASM 縺ｫ關ｽ縺ｨ縺呵ｨ育判縲・LI 縺ｮ target 閾ｪ蜍墓耳貂ｬ縺ｯ蟒・ｭ｢縺励�［anifest 鬧・虚縺ｫ縺吶ｋ縲・
- 莉雁屓縺ｯ繝峨く繝･繝｡繝ｳ繝医・縺ｿ霑ｽ蜉�縲ゅユ繧ｹ繝医・譛ｪ螳溯｡後�・

# 2026-02-03 繝｢繧ｸ繝･繝ｼ繝ｫ繧ｰ繝ｩ繝・Phase2) 逹�謇・
- `nepl-core/src/module_graph.rs` 繧定ｿｽ蜉�縲ゆｾ晏ｭ倥げ繝ｩ繝輔→蠕ｪ迺ｰ讀懷・縺ｮ縺ｿ繧貞ｮ溯｣・＠縲√ヵ繧｡繧､繝ｫ繧ｹ繝励Λ繧､繧ｹ縺帙★縺ｫ AST 繧剃ｿ晄戟縺吶ｋ繝弱・繝峨ｒ讒狗ｯ峨☆繧区ｮｵ髫弱�・
- `ModuleGraphBuilder` 縺ｯ stdlib 繧呈里螳壻ｾ晏ｭ倥→縺励※逋ｻ骭ｲ縺励�～#import` 繝代せ・育嶌蟇ｾ/繝代ャ繧ｱ繝ｼ繧ｸ・峨°繧峨ヵ繧｡繧､繝ｫ繧定ｧ｣豎ｺ縲・FS 縺ｧ cycle 繧呈､懷・縺励�》opo 鬆・ｒ菫晄戟縲・
- `lib.rs` 縺ｫ module_graph 繧貞・髢九�・
- 縺ｾ縺�蜷榊燕隗｣豎ｺ/蜿ｯ隕匁�ｧ/Prelude 蜿肴丐縺ｯ譛ｪ螳溯｣・ｼ・hase3 莉･髯阪〒蟇ｾ蠢應ｺ亥ｮ夲ｼ峨�・

# 2026-02-03 Export陦ｨ(Phase3) 蝓ｺ遉主ｮ溯｣・
- AST/lexer/parser 縺ｫ `pub` 蜿ｯ隕匁�ｧ繧貞ｰ主・縺励�～fn/struct/enum/trait` 縺ｧ蜈ｬ髢区欠螳壹ｒ繝代・繧ｹ蜿ｯ閭ｽ縺ｫ縲・
- ModuleGraph 縺ｫ pub 螳夂ｾｩ縺ｨ pub import 縺ｮ蜀阪お繧ｯ繧ｹ繝昴・繝医ｒ髮・ｨ医☆繧・ExportTable 繧定ｿｽ蜉�縲る㍾隍・・ DuplicateExport 縺ｨ縺励※讀懷・縲・
- ModuleNode 縺ｫ import 縺ｮ蜿ｯ隕匁�ｧ縺ｨ萓晏ｭ伜・ ModuleId 繧剃ｿ晄戟縺励�》opo 鬆・↓蝓ｺ縺･縺・export 繧貞崋螳夂せ縺ｪ縺励〒讒狗ｯ峨�・
- 繝・せ繝・ 繝阪ャ繝医Ρ繝ｼ繧ｯ縺ｪ縺礼腸蠅・・縺溘ａ cargo test 螳溯｡御ｸ榊庄・・asmparser 繝�繧ｦ繝ｳ繝ｭ繝ｼ繝峨〒螟ｱ謨暦ｼ峨□縺後�√Ο繝ｼ繧ｫ繝ｫ霑ｽ蜉�繝・せ繝医ｒ逕ｨ諢上�・

# 2026-02-03 蜷榊燕隗｣豎ｺ貅門ｙ(Phase4) 逹�謇・
- `nepl-core/src/resolve.rs` 繧定ｿｽ蜉�縺励�．efId/DefKind 縺ｨ繝｢繧ｸ繝･繝ｼ繝ｫ縺斐→縺ｮ蜈ｬ髢句ｮ夂ｾｩ繝・・繝悶Ν繧貞庶髮・☆繧・`collect_defs`縲・xportTable 縺ｨ蜷域・縺吶ｋ `compose_exports` 繧貞ｮ溯｣・ｼ亥ｼ丈ｸｭ隴伜挨蟄舌・隗｣豎ｺ縺ｾ縺ｧ縺ｯ譛ｪ謗･邯夲ｼ峨�・
- Phase4 縺ｮ譛ｬ菴難ｼ医せ繧ｳ繝ｼ繝怜━蜈磯�・ｽ阪�￣relude縲　merge 繧貞性繧�隗｣豎ｺ・峨・譛ｪ逹�謇九�よｬ｡繧ｹ繝・ャ繝励〒 Resolver 繧・HIR 逕滓・縺ｫ邨・∩霎ｼ繧�蠢・ｦ√≠繧翫�・

# 2026-02-03 繝薙Ν繝芽ｪｿ謨ｴ
- `lib.rs` 縺ｧ `extern crate std` 繧呈擅莉ｶ莉倥″縺ｧ繝ｪ繝ｳ繧ｯ縺励�［odule_graph 縺ｪ縺ｩ縺ｮ std 萓晏ｭ倥ｒ隗｣豎ｺ・・asm32 莉･螟厄ｼ峨�・

# 2026-02-03 菴懈･ｭ繝｡繝｢ (kpread UTF-16LE 蜈･蜉・
## 菫ｮ豁｣蜀・ｮｹ
- `kp/kpread` 縺ｮ `scanner_skip_ws`/`scanner_read_i32` 縺・UTF-16LE 縺ｮ NUL 繝舌う繝医ｒ譁・ｭ励→縺励※謇ｱ縺｣縺ｦ縺・◆縺溘ａ縲¨UL 繧偵せ繧ｭ繝・・縺吶ｋ蜃ｦ逅・ｒ霑ｽ蜉�縲・
- PowerShell 繝代う繝励〒縺ｮ `\"1 3\"` 蜈･蜉帙〒繧・`abc086_a.tmp.nepl` 縺梧ｭ｣縺励￥ Odd 繧貞・縺吶ｈ縺・↓菫ｮ豁｣縲・

## 繝・せ繝亥ｮ溯｡檎ｵ先棡
- `printf '1\0 3\0' | cargo run -p nepl-cli -- -i examples/abc086_a.tmp.nepl --run`

# 2026-02-03 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝芽ｧ｣豎ｺ/繧ｹ繧ｿ繝・け雜・℃險ｺ譁ｭ菫ｮ豁｣
- 髢｢謨ｰ螳夂ｾｩ縺ｮ2蝗樒岼襍ｰ譟ｻ縺ｧ縲∝錐蜑堺ｸ�閾ｴ縺�縺代〒蝙九ｒ蠑輔＞縺ｦ縺・◆邂・園繧偵�後す繧ｰ繝阪メ繝｣荳�閾ｴ縲阪〒驕ｸ縺ｶ繧医≧縺ｫ螟画峩縺励�√が繝ｼ繝舌・繝ｭ繝ｼ繝峨・蜿悶ｊ驕輔∴繧帝亟豁｢縲・
- prefix 蠑上〒菴吝臆繧ｹ繧ｿ繝・け蛟､繧偵ラ繝ｭ繝・・縺励◆蝣ｴ蜷医↓險ｺ譁ｭ繧貞・縺吶ｈ縺・↓縺励�・℃蜑ｰ蠑墓焚縺ｮ蜻ｼ縺ｳ蜃ｺ縺励ｒ繧ｨ繝ｩ繝ｼ蛹悶�・

## 繝・せ繝亥ｮ溯｡檎ｵ先棡
- `cargo test` (300s 縺ｧ繧ｿ繧､繝�繧｢繧ｦ繝医�ゅさ繝ｳ繝代う繝ｫ隴ｦ蜻翫∪縺ｧ縺ｯ蜃ｺ蜉帙＆繧後◆縺後ユ繧ｹ繝亥ｮ瑚ｵｰ縺ｯ譛ｪ遒ｺ隱・
- `cargo test -p nepl-core --test neplg2 -- --nocapture`
- `cargo run -p nepl-cli -- test`

# 2026-02-03 菴懈･ｭ繝｡繝｢ (string map/set 霑ｽ蜉�)
## 菫ｮ豁｣蜀・ｮｹ
- `alloc/collections/hashmap_str` 縺ｨ `hashset_str` 繧定ｿｽ蜉�縺励�：NV-1a 縺ｨ `str_eq` 縺ｫ繧医ｋ蜀・ｮｹ豈碑ｼ・〒 str 繧ｭ繝ｼ/隕∫ｴ�繧呈桶縺医ｋ繧医≧縺ｫ縺励◆縲・
- `stdlib/tests/hashmap_str.nepl` 縺ｨ `hashset_str.nepl` 繧定ｿｽ蜉�縺励�∝酔蜀・ｮｹ譁・ｭ怜・縺ｮ蛻･繝舌ャ繝輔ぃ縺ｧ繧よ､懃ｴ｢縺ｧ縺阪ｋ縺薙→繧堤｢ｺ隱阪☆繧九ユ繧ｹ繝医ｒ逕ｨ諢上�・
- `nepl-core/tests/selfhost_req.rs` 縺ｮ譁・ｭ怜・繝槭ャ繝苓ｦ∽ｻｶ繧・`hashmap_str` 縺ｧ螳溯｡後〒縺阪ｋ蠖｢縺ｫ譖ｴ譁ｰ縺励�√ユ繧ｹ繝医ｒ譛牙柑蛹悶�・
- `stdlib/tests/string.nepl` 縺ｮ `StringBuilder` 繝・せ繝医〒菴吝臆繧ｹ繧ｿ繝・け蛟､縺悟・縺ｦ縺・◆蜻ｼ縺ｳ蜃ｺ縺怜ｽ｢蠑上ｒ菫ｮ豁｣縲・
- `doc/testing.md` 縺ｫ `hashmap_str`/`hashset_str` 縺ｮ險倩ｿｰ繧定ｿｽ蜉�縲・

## 蛯呵�・
- 豎守畑逧・↑ Map/Set 縺ｮ trait 繝吶・繧ｹ螳溯｣・・譛ｪ逹�謇具ｼ・elfhost_req 縺ｮ trait 諡｡蠑ｵ縺ｨ蜷医ｏ縺帙※莉雁ｾ悟ｯｾ蠢懶ｼ峨�・
- `hashmap_str`/`hashset_str` 縺ｮ繝上ャ繧ｷ繝･險育ｮ励・ `set`/`while` 繧剃ｽｿ繧上↑縺・・蟶ｰ螳溯｣・↓螟画峩縺励�∫ｴ皮ｲ矩未謨ｰ縺ｨ縺励※蛻ｩ逕ｨ蜿ｯ閭ｽ縺ｫ縺励◆縲・

## 繝・せ繝亥ｮ溯｡檎ｵ先棡
- `cargo test`
- `cargo run -p nepl-cli -- test`
- nepl-web 縺ｮ stdlib 蝓九ａ霎ｼ縺ｿ繧・build.rs 縺ｧ閾ｪ蜍慕函謌舌☆繧九ｈ縺・↓螟画峩縺励�・stdlib 驟堺ｸ九・ .nepl 繧堤ｶｲ鄒・噪縺ｫ蜿悶ｊ霎ｼ繧�繧医≧縺ｫ縺励◆縲・
- `cargo build --target wasm32-unknown-unknown --manifest-path nepl-web/Cargo.toml --release` 繧貞ｮ溯｡後＠縲］epl-web 縺ｮ stdlib 蝓九ａ霎ｼ縺ｿ縺後ン繝ｫ繝峨〒隗｣豎ｺ縺ｧ縺阪ｋ縺薙→繧堤｢ｺ隱阪＠縺滂ｼ医ロ繝・ヨ繝ｯ繝ｼ繧ｯ繧｢繧ｯ繧ｻ繧ｹ縺ゅｊ・峨�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (nodesrc doctest 螳溯｡悟渕逶､縺ｮ菫ｮ豁｣)
## 菫ｮ豁｣蜀・ｮｹ
- `nodesrc/tests.js` 縺ｮ螳溯｡梧婿蠑上ｒ `child_process + stdin JSON` 縺九ｉ縲∝酔荳�繝励Ο繧ｻ繧ｹ縺ｧ `run_test.js` 繧堤峩謗･蜻ｼ縺ｳ蜃ｺ縺呎婿蠑上↓螟画峩縲・
- `nodesrc/run_test.js` 縺ｫ `createRunner` / `runSingle` 繧定ｿｽ蜉�縺励�√ユ繧ｹ繝亥ｮ溯｡後Ο繧ｸ繝・け繧貞・蛻ｩ逕ｨ蜿ｯ閭ｽ縺ｫ謨ｴ逅・�・
- 蜷・worker 縺斐→縺ｫ compiler 繧・1 蝗槭□縺代Ο繝ｼ繝峨☆繧九ｈ縺・↓縺励※縲∽ｸ崎ｦ√↑蛻晄悄蛹悶Ο繧ｰ縺ｨ繧ｪ繝ｼ繝舌・繝倥ャ繝峨ｒ蜑頑ｸ帙�・
- compiler 蛛ｴ縺ｮ螟ｧ驥上Ο繧ｰ縺後ユ繧ｹ繝域ｨ呎ｺ門・蜉帙↓豬√ｌ縺ｪ縺・ｈ縺・�～console.*` 繧呈椛蛻ｶ縺吶ｋ繝ｩ繝・ヱ繧定ｿｽ蜉�縲・
- `nodesrc/tests.js` 縺ｮ讓呎ｺ門・蜉帙ｒ隕∫せ陦ｨ遉ｺ縺ｫ螟画峩縺励�～summary` 縺ｨ `top_issues`・亥・鬆ｭ5莉ｶ・峨ｒ JSON 縺ｧ陦ｨ遉ｺ縲・

## 蜴溷屏
- 迴ｾ陦檎腸蠅・〒 `child_process` 邨檎罰縺ｮ stdin 蜿励￠貂｡縺励′謌千ｫ九○縺壹�～run_test.js` 縺悟・蜉・JSON 繧貞女縺大叙繧後↑縺・◆繧√�∝・莉ｶ `invalid json from run_test.js`・・rrored・峨↓縺ｪ縺｣縺ｦ縺・◆縲・

## 迴ｾ迥ｶ
- doctest 螳溯｡瑚・菴薙・蠕ｩ譌ｧ縲・
- 螳溯｡檎ｵ先棡: `total=326, passed=250, failed=76, errored=0`縲・
- 螟ｱ謨・76 莉ｶ縺ｯ doctest 縺ｮ荳ｭ霄ｫ襍ｷ蝗�・・entry function is missing or ambiguous`縲∵立讒区枚逕ｱ譚･縺ｮ `parenthesized expressions are not supported` 縺ｪ縺ｩ・峨�・

## plan.md縺ｨ縺ｮ蟾ｮ蛻・
- plan.md 縺ｮ險�隱樔ｻ墓ｧ倥↓蟇ｾ縺吶ｋ譛ｬ菴薙・譛ｪ蟇ｾ蠢・蟾ｮ蛻・↓繧医ｊ縲∽ｸ�驛ｨ doctest 縺悟､ｱ謨励＠縺ｦ縺・ｋ縲・
- 莉雁屓縺ｯ繝・せ繝亥渕逶､縺ｮ蜈ｨ莉ｶ errored 繧定ｧ｣豸医＠縲∝､ｱ謨苓ｦ∝屏繧・`top_issues` 縺ｧ蜊ｳ蠎ｧ縺ｫ遒ｺ隱阪〒縺阪ｋ迥ｶ諷九∪縺ｧ謾ｹ蝟・＠縺溘�・

## 繝・せ繝亥ｮ溯｡檎ｵ先棡
- `node nodesrc/tests.js -i tutorials/getting_started/01_hello_world.n.md -o /tmp/one.json --dist web/dist -j 1`
- `node nodesrc/tests.js -i tests -i tutorials -i stdlib -o /tmp/nmd-tests.json --dist web/dist -j 4`
- `NO_COLOR=true trunk build`・医ロ繝・ヨ繝ｯ繝ｼ繧ｯ蛻ｶ髯舌〒萓晏ｭ伜叙蠕励↓螟ｱ謨励＠譛ｪ螳御ｺ・ｼ・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (trunk build 蠕ｩ譌ｧ蠕後・迴ｾ迥ｶ謚頑升)
## 迴ｾ迥ｶ
- `NO_COLOR=true trunk build` 縺ｯ謌仙粥縲・
- 縺溘□縺・doctest 螳溯｡後・ `total=326, errored=326`縲・
- 蜴溷屏縺ｯ dist 謗｢邏｢繝ｭ繧ｸ繝・け縺ｧ縲∥rtifact 縺ｮ譛臥┌縺ｧ縺ｯ縺ｪ縺上ョ繧｣繝ｬ繧ｯ繝医Μ蟄伜惠縺ｮ縺ｿ縺ｧ `dist/` 繧呈治逕ｨ縺励※縺励∪縺・％縺ｨ縲・
- 螳滄圀縺ｮ compiler artifact 縺ｯ `web/dist/` 縺ｫ逕滓・縺輔ｌ縺ｦ縺・ｋ縲・

## 蟇ｾ蠢懈婿驥・
- `todo.md` 縺ｫ縲∥rtifact 繝壹い蟄伜惠繝吶・繧ｹ縺ｮ謗｢邏｢縺ｸ謾ｹ菫ｮ縺吶ｋ螳溯｣・ｨ育判繧定ｿｽ蜉�縲・
- 蝗槫ｸｰ繝・せ繝医→繝峨く繝･繝｡繝ｳ繝・CI謨ｴ蜷医∪縺ｧ蜷ｫ繧√※蟇ｾ蠢懊☆繧九�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (dist謗｢邏｢縺ｮ譬ｹ譛ｬ菫ｮ豁｣)
## 菫ｮ豁｣蜀・ｮｹ
- `nodesrc/compiler_loader.js` 縺ｫ `findCompilerDistDir` / `loadCompilerFromCandidates` 繧定ｿｽ蜉�縲・
- 蛟呵｣懊ョ繧｣繝ｬ繧ｯ繝医Μ縺ｮ蜈磯�ｭ謗｡逕ｨ繧貞ｻ・ｭ｢縺励�～nepl-web-*.js` 縺ｨ `*_bg.wasm` 縺ｮ繝壹い縺悟ｭ伜惠縺吶ｋ蛟呵｣懊・縺ｿ繧呈治逕ｨ縺吶ｋ繧医≧螟画峩縲・
- 蛟呵｣懷・貊・凾縺ｯ謗｢邏｢縺励◆蜈ｨ繝代せ繧貞性繧�繧ｨ繝ｩ繝ｼ繧定ｿ斐☆繧医≧螟画峩縲・
- `nodesrc/run_test.js` 縺ｮ `createRunner` 繧貞�呵｣懊・繝ｼ繧ｹ隗｣豎ｺ縺ｸ螟画峩縲・
- `nodesrc/tests.js` 縺ｫ `resolved_dist_dirs` 繧・JSON 蜃ｺ蜉帙→縺励※霑ｽ蜉�縺励�《tdout 縺ｮ隕∫せJSON縺ｫ繧・`dist.resolved` 繧定｡ｨ遉ｺ縲・

## 繝・せ繝亥ｮ溯｡檎ｵ先棡
- `NO_COLOR=true trunk build` (success)
- `node nodesrc/tests.js -i tests -i tutorials -i stdlib -o /tmp/nmd-tests-after-fix.json -j 4`
  - `total=326, passed=250, failed=76, errored=0`
  - `dist.resolved=["/mnt/d/project/NEPLg2/web/dist"]`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (tests邨先棡遒ｺ隱阪→繧ｳ繝ｳ繝代う繝ｩ蜀崎ｨｭ險郁ｨ育判)
## 螳滓ｸｬ邨先棡
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests -o /tmp/tests-only.json -j 4`
  - `total=309, passed=240, failed=69, errored=0`
  - 荳ｻ隕∝､ｱ謨怜だ蜷・ `expected compile_fail, but compiled successfully`, `expression left extra values on the stack`, `return type does not match signature`

## 繧ｳ繝ｳ繝代う繝ｩ迴ｾ迥ｶ遒ｺ隱・
- `nepl-core/src/parser.rs` 縺ｨ `nepl-core/src/typecheck.rs` 縺瑚ぇ螟ｧ蛹悶＠縲∽ｻ墓ｧ倩ｿｽ蜉�譎ゅ・蠖ｱ髻ｿ遽・峇縺悟ｺ・＞縲・
- `module_graph.rs` / `resolve.rs` 縺ｯ蟄伜惠縺吶ｋ縺・`compile_wasm` 譛ｬ豬√↓邨ｱ蜷医＆繧後※縺・↑縺・�・
- 隴ｦ蜻翫′螟壹￥縲∵悴菴ｿ逕ｨ邨瑚ｷｯ縺梧ｮ九▲縺ｦ縺・ｋ縲・

## 蟇ｾ蠢・
- `todo.md` 縺ｫ謚懈悽蜀崎ｨｭ險郁ｨ育判繧定ｿｽ蜉�縲・
- 譌｢蟄倥・ `plan.md` 隕∵ｱゑｼ亥腰陦恵lock/if讒区枚縲》arget蜀崎ｨｭ險医�´SP蜑肴署縺ｮ諠・�ｱ謨ｴ蛯呻ｼ峨ｒ蜑肴署縺ｫ縲∵ｮｵ髫守ｽｮ謠帛梛縺ｮ蜀崎ｨｭ險医Ο繝ｼ繝峨・繝・・繧貞ｮ夂ｾｩ縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺ1/2螳溯｣・
## 螳溯｣・
- `nodesrc/analyze_tests_json.js` 繧定ｿｽ蜉�縲・
  - doctest邨先棡JSON・・nodesrc/tests.js`蜃ｺ蜉幢ｼ峨ｒ隱ｭ縺ｿ縲’ail/error逅・罰繧偵き繝・ざ繝ｪ髮・ｨ医☆繧気LI縲・
- `nepl-core/src/compiler.rs` 繧呈ｮｵ髫朱未謨ｰ縺ｸ謨ｴ逅・�・
  - `run_typecheck` / `run_move_check` / `emit_wasm` 繧貞ｰ主・縲・
  - `CompileTarget` / `BuildProfile` / `CompileOptions` / `CompilationArtifact` / `compile_module` / `compile_wasm` 縺ｫ譌･譛ｬ隱枦oc繧ｳ繝｡繝ｳ繝医ｒ霑ｽ蜉�縲・
  - 譌｢蟄俶嫌蜍輔ｒ邯ｭ謖√＠縺､縺､縲∝・逅・ヵ繝ｭ繝ｼ繧呈・遉ｺ蛹悶�・

## 繝・せ繝育ｵ先棡
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests -o /tmp/tests-only-after-phase2.json -j 4`
  - `total=309, passed=240, failed=69, errored=0`・亥燕蝗槭→蜷悟�､・・
- `node nodesrc/analyze_tests_json.js /tmp/tests-only-after-phase2.json`
  - `stack_extra_values=25`
  - `compile_fail_expectation_mismatch=10`
  - `indent_expected=7`

## 谺｡繧｢繧ｯ繧ｷ繝ｧ繝ｳ
- `other=22` 縺ｮ蜀・ｨｳ繧偵＆繧峨↓蛻・ｧ｣縺励�｝arser蛻・牡逹�謇区凾縺ｮ蜆ｪ蜈磯�・ｒ遒ｺ螳壹☆繧九�・
- `tests/block_single_line.n.md` 縺ｨ `tests/block_if_semantics.n.md` 縺ｮ螟ｱ謨励ｒ譛�蛻昴・菫ｮ豁｣蟇ｾ雎｡縺ｫ縺吶ｋ縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (WAT蜿ｯ隱ｭ諤ｧ謾ｹ蝟・→doctest隕∫ｴ・ｼｷ蛹・
## 螳溯｣・
- `nepl-core/src/compiler.rs`
  - `CompilationArtifact` 縺ｫ `wat_comments: String` 繧定ｿｽ蜉�縲・
  - HIR 縺ｨ蝙区ュ蝣ｱ縺九ｉ髢｢謨ｰ繧ｷ繧ｰ繝阪メ繝｣繝ｻ蠑墓焚繝ｻ繝ｭ繝ｼ繧ｫ繝ｫ縺ｮ諠・�ｱ繧貞庶髮・＠縲仝AT繝・ヰ繝・げ繧ｳ繝｡繝ｳ繝域枚蟄怜・繧堤函謌舌☆繧句・逅・ｒ霑ｽ蜉�縲・
- `nepl-cli/src/main.rs`
  - `wat` 蜃ｺ蜉帶凾縺ｮ縺ｿ縲～wat_comments` 繧・`;;` 繧ｳ繝｡繝ｳ繝医→縺励※蜈磯�ｭ縺ｫ莉伜刈縺吶ｋ蜃ｦ逅・ｒ霑ｽ蜉�縲・
  - `wat-min` 縺ｯ蠕捺擂縺ｩ縺翫ｊ minify 繧堤ｶｭ謖√＠縺､縺､縲～attached-source` 縺ｨ compiler 諠・�ｱ繧ｳ繝｡繝ｳ繝医・縺ｿ谿九☆蜍穂ｽ懊↓謨ｴ逅・�・
- `nepl-web/src/lib.rs`
  - `compile_wasm_with_entry` 縺・`wasm` 縺ｨ `wat_comments` 繧定ｿ斐○繧九ｈ縺・↓螟画峩縲・
  - `compile_to_wat` 縺ｯ繝・ヰ繝・げ繧ｳ繝｡繝ｳ繝医ｒ莉倅ｸ弱�～compile_to_wat_min` 縺ｯ繝・ヰ繝・げ繧ｳ繝｡繝ｳ繝医ｒ髯､螟悶＠縺ｦ compiler/source 繧ｳ繝｡繝ｳ繝医・縺ｿ莉倅ｸ弱�・
- `nodesrc/tests.js`
  - 讓呎ｺ門・蜉帙・ `top_issues.error` 繧・ANSI 髯､蜴ｻ繝ｻ遏ｭ譁・喧・亥・鬆ｭ3陦・譛�螟ｧ240譁・ｭ暦ｼ峨＠縲∬ｦ∫せ縺ｮ縺ｿ陦ｨ遉ｺ縺吶ｋ繧医≧螟画峩縲・
  - Node warning 縺ｮ讓呎ｺ門・蜉帙ヮ繧､繧ｺ繧呈椛蛻ｶ縲・

## 繝・せ繝亥ｮ溯｡檎ｵ先棡
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests -o dist/tests.json`
  - `total=312, passed=278, failed=34, errored=0`
  - 螟ｱ謨励・荳ｻ縺ｫ鬮倬嚴髢｢謨ｰ邉ｻ縺ｨ compile_fail 譛溷ｾ・ｷｮ蛻・〒縲∝ｮ溯｡悟渕逶､繧ｨ繝ｩ繝ｼ縺ｯ縺ｪ縺・

## 陬懆ｶｳ
- `wat` 縺ｯ隧ｳ邏ｰNEPL繝・ヰ繝・げ繧ｳ繝｡繝ｳ繝医ｒ蜷ｫ縺ｿ縲～wat-min` 縺ｯ隧ｳ邏ｰ繧ｳ繝｡繝ｳ繝医ｒ髯､螟悶＠縺､縺､ `attached-source` 縺ｨ compiler 諠・�ｱ繧ｳ繝｡繝ｳ繝医ｒ菫晄戟縺吶ｋ譁ｹ驥昴ｒ遒ｺ隱肴ｸ医∩縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (web/tests.html 隧ｳ邏ｰ陦ｨ遉ｺ蠑ｷ蛹・
## 螳溯｣・
- `web/tests.html` 縺ｮ邨先棡繝｢繝・Ν繧・`nodesrc/tests.js` 蜃ｺ蜉幢ｼ・id/file/index/tags/source/error/phase/worker/compiler/runtime`・峨↓蟇ｾ蠢懊＆縺帙◆縲・
- 蜷・doctest 縺ｮ螻暮幕隧ｳ邏ｰ縺ｫ莉･荳九ｒ霑ｽ蜉�:
  - `id/phase/worker/duration/file` 縺ｮ繝｡繧ｿ諠・�ｱ
  - `compiler` / `runtime` 繧ｪ繝悶ず繧ｧ繧ｯ繝医・陦ｨ遉ｺ
  - `raw result JSON` 謚倥ｊ縺溘◆縺ｿ陦ｨ遉ｺ
  - doctest繧ｽ繝ｼ繧ｹ縺ｮ陦檎分蜿ｷ莉倥″陦ｨ遉ｺ
- 繧ｨ繝ｩ繝ｼ譁・ｸｭ縺ｮ `--> path:line:col` 縺九ｉ陦檎分蜿ｷ繧呈歓蜃ｺ縺励�∬ｩｲ蠖薙た繝ｼ繧ｹ陦後ｒ繝上う繝ｩ繧､繝医☆繧九ｈ縺・↓縺励◆縲・

## 遒ｺ隱・
- `node -e "const fs=require('fs');const s=fs.readFileSync('web/tests.html','utf8');const js=s.split('<script>')[1].split('</script>')[0];new Function(js);console.log('ok');"`
  - `ok`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (鬮倬嚴髢｢謨ｰ螳溯｣・ヵ繧ｧ繝ｼ繧ｺ蜀埼幕: parser/typecheck荳頑ｵ∽ｿｮ豁｣)
## 螳溯｣・
- `nepl-core/src/parser.rs`
  - `apply 10 (x): ...` 蠖｢蠑上ｒ蛹ｿ蜷埼未謨ｰ繝ｪ繝・Λ繝ｫ縺ｨ縺励※謇ｱ縺・desugar 繧定ｿｽ蜉�縲・
  - `(params): body` 繧貞・驛ｨ逧・↓ `__lambda_*` 縺ｮ `FnDef` + 蛟､蠑上↓螟画鋤縺励※ AST 蛹悶☆繧九�・
- `nepl-core/src/ast.rs`
  - `Symbol::Ident` 繧・`Ident, Vec<TypeExpr>, forced_value(bool)` 縺ｫ諡｡蠑ｵ縺励�～@ident` 繧貞玄蛻･蜿ｯ閭ｽ縺ｫ縺励◆縲・
- `nepl-core/src/typecheck.rs`
  - 蠑上せ繧ｿ繝・け隕∫ｴ� `StackEntry` 縺ｫ `auto_call` 繧定ｿｽ蜉�縲・
  - `@ident` 繧・`auto_call=false` 縺ｨ縺励※ reduce 蟇ｾ雎｡縺九ｉ螟悶○繧九ｈ縺・↓縺励◆縲・
  - reduce 譎ゅ↓縲悟承遶ｯ髢｢謨ｰ縺悟､門・蜻ｼ縺ｳ蜃ｺ縺励・髢｢謨ｰ蝙句ｼ墓焚縺ｧ縺ゅｋ縲榊�ｴ蜷医・螟門・蜻ｼ縺ｳ蜃ｺ縺励ｒ蜆ｪ蜈医☆繧矩∈謚槭ｒ霑ｽ蜉�縲・
- `nepl-web/src/lib.rs`
  - `Symbol::Ident` 繝代ち繝ｼ繝ｳ繧・AST 螟画峩縺ｸ霑ｽ蠕薙�・

## 螳溯｣・
- `nepl-core/src/codegen_wasm.rs`
  - 髢｢謨ｰ蝙九ｒ WASM 蛟､蝙九∈荳九ｍ縺咎圀縲∬ｧ｣豎ｺ貂医∩蝙九ｒ隕九ｋ繧医≧菫ｮ豁｣縲・
  - `TypeKind::Function` 繧呈圻螳夂噪縺ｫ `i32` 縺ｨ縺励※荳九ｍ縺帙ｋ繧医≧縺ｫ縺励◆・磯未謨ｰ蜿ら・陦ｨ迴ｾ縺ｮ蝨溷床・峨�・

## 繝・せ繝亥ｮ溯｡檎ｵ先棡
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/functions-after-sigresolve.json`
  - `total=16, passed=10, failed=6, errored=0`
  - 荳ｻ隕∝､ｱ謨・ `unknown function _unknown`・磯未謨ｰ蛟､蜻ｼ縺ｳ蜃ｺ縺励・ codegen 譛ｪ螳溯｣・ｼ・
- `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-hof-phase.json`
  - `total=312, passed=278, failed=34, errored=0`・井ｻｶ謨ｰ縺ｯ謐ｮ縺育ｽｮ縺搾ｼ・

## 迴ｾ迥ｶ隧穂ｾ｡
- parser 襍ｷ蝗�縺ｮ `undefined identifier` 縺�縺｣縺・`function_first_class_literal` 縺ｯ縲∝諺蜷埼未謨ｰ縺ｨ縺励※繝代・繧ｹ縺輔ｌ繧区ｮｵ髫弱∪縺ｧ蜑埼�ｲ縲・
- 縺・∪縺ｮ荳ｻ髫懷ｮｳ縺ｯ荳頑ｵ√〒縺ｯ縺ｪ縺丈ｸｭ豬√�應ｸ区ｵ・
  - 髢｢謨ｰ蛟､蜻ｼ縺ｳ蜃ｺ縺・(`func val`) 繧・`_unknown` 縺ｫ繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ縺励※縺翫ｊ縲～call_indirect` 逶ｸ蠖薙・邨瑚ｷｯ縺梧悴螳溯｣・�・
  - capture 縺ゅｊ nested function (`add x y`) 縺ｯ繧ｯ繝ｭ繝ｼ繧ｸ繝｣螟画鋤譛ｪ螳溯｣・・縺溘ａ譛ｪ蟇ｾ蠢懊�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (functions蠕ｩ譌ｧ縺ｨLSP API諡｡蠑ｵ縺ｮ蜑埼�ｲ)
## 螳溯｣・
- `stdlib/std/stdio.nepl`
  - `ansi_*` 髢｢謨ｰ鄒､縺ｮ譛ｫ蟆ｾ `;` 繧帝勁蜴ｻ縺励�～<()->str>` 繧ｷ繧ｰ繝阪メ繝｣縺ｨ譛ｬ菴薙・謌ｻ繧雁�､謨ｴ蜷医ｒ蝗槫ｾｩ縲・
- `nepl-core/src/typecheck.rs`
  - `apply_function` 縺ｮ邏皮ｲ区�ｧ讀懈渊繧貞ｸｸ譎よ怏蜉ｹ蛹悶＠縲～pure context cannot call impure function` 縺ｮ隕矩�・＠繧剃ｿｮ豁｣縲・
  - `check_block` 縺ｮ蜑ｯ菴懃畑譁・ц繧貞ｸｸ縺ｫ `Impure` 縺ｸ荳頑嶌縺阪☆繧区嫌蜍輔ｒ蜑企勁縲・
  - `check_function` 縺ｫ `is_entry` 繧貞ｰ主・縺励�‘ntry 髢｢謨ｰ縺ｮ縺ｿ `Impure` 譁・ц縺ｧ隧穂ｾ｡・・wasi` main 縺ｮ莉墓ｧ倥↓謨ｴ蜷茨ｼ峨�・
- `nepl-web/src/lib.rs`
  - 蜷榊燕隗｣豎ｺ JSON 繧貞・騾夂函謌舌☆繧・`name_resolution_payload_to_js` 繧定ｿｽ蜉�縲・
  - `analyze_semantics` 縺ｫ莉･荳九ｒ霑ｽ蜉�:
    - `name_resolution`・・efinitions/references/by_name/policy・・
    - `token_resolution`・・oken 蜊倅ｽ阪・蜿ら・隗｣豎ｺ蛟呵｣懊→譛�邨りｧ｣豎ｺID・・

## 繝・せ繝亥ｮ溯｡檎ｵ先棡
- `NO_COLOR=true trunk build`: success
- `node nodesrc/tests.js -i tests/functions.n.md -o /tmp/tests-functions-after-entry-impure.json -j 1`
  - `total=19, passed=19, failed=0, errored=0`
- `node nodesrc/test_analysis_api.js`
  - `total=7, passed=7, failed=0`

## 繧ｳ繝溘ャ繝・
- `cb90042`
  - `Fix purity/effect checks and extend semantics resolve API`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (sort 繝・せ繝郁ｿｽ蜉�)
## 螳溯｣・
- `tests/sort.n.md` 繧呈眠隕丈ｽ懈・縲・
  - `sort_quick` / `sort_merge` / `sort_heap` / `sort` / `sort_is_sorted` 縺ｮ 5 繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
  - 縺・★繧後ｂ `Vec<i32>` 繧堤函謌舌＠縺ｦ繧ｽ繝ｼ繝育ｵ先棡繧呈焚蛟､蛹悶＠縺ｦ讀懆ｨｼ縺吶ｋ讒区・縲・

## 螳溯｡檎ｵ先棡
- `node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-new.json -j 1`
  - `total=5, passed=0, failed=5, errored=0`
  - 蜈ｱ騾壹お繝ｩ繝ｼ: `pure context cannot call impure function`
  - 逋ｺ逕溽ｮ・園: `stdlib/alloc/sort.nepl:117` (`sort_is_sorted` 蜀・`set ok false`)

## 謇�隕・
- `sort.nepl` 蛛ｴ縺ｮ邏皮ｲ区�ｧ謖・ｮ壹→螳溯｣・(`set` 縺ｮ菴ｿ逕ｨ) 縺檎泝逶ｾ縺励※縺翫ｊ縲√∪縺壹％縺薙ｒ菫ｮ豁｣縺吶ｋ蠢・ｦ√′縺ゅｋ縲・
- 繝ｦ繝ｼ繧ｶ繝ｼ謖・遭縺ｩ縺翫ｊ縲√ず繧ｧ繝阪Μ繧ｯ繧ｹ邨瑚ｷｯ縺ｨ sort 縺ｮ騾｣謳ｺ荳榊・蜷医→縺励※邯咏ｶ夊ｪｿ譟ｻ縺吶ｋ縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (if-layout繝槭・繧ｫ繝ｼ謚ｽ蜃ｺ縺ｮ荳頑ｵ∽ｿｮ豁｣ + 蜈ｨ菴灘・蛻・｡・
## 螳溯｣・
- `nepl-core/src/parser.rs`
  - `if:` / `while:` 繝ｬ繧､繧｢繧ｦ繝郁ｧ｣譫舌〒縲～Stmt::ExprSemi` 陦鯉ｼ井ｾ・ `else ();`・峨ｂ繝槭・繧ｫ繝ｼ謚ｽ蜃ｺ蟇ｾ雎｡縺ｫ蜷ｫ繧√ｋ繧医≧菫ｮ豁｣縲・
  - 縺薙ｌ縺ｫ繧医ｊ `else` 縺碁�壼ｸｸ隴伜挨蟄舌→縺励※隱､隗｣驥医＆繧後ｋ邨瑚ｷｯ繧帝勁蜴ｻ縲・
- `tests/if.n.md`
  - 繝阪せ繝・if 縺ｮ蝗槫ｸｰ遒ｺ隱阪こ繝ｼ繧ｹ繧・3 莉ｶ霑ｽ蜉�縲・
  - `node nodesrc/tests.js -i tests/if.n.md ...` 縺ｧ `58/58 pass` 繧堤｢ｺ隱阪�・

## 螳溯｡檎ｵ先棡
- 菫ｮ豁｣蜑榊・菴・ `total=336, passed=303, failed=33, errored=0`
- parser菫ｮ豁｣蠕・ `total=336, passed=311, failed=25, errored=0`
- 謾ｹ蝟・㍼: `+8 pass`

## 螟ｱ謨怜・鬘橸ｼ域怙譁ｰ・・
- `tests/neplg2.n.md`: 7
- `tests/sort.n.md`: 5
- `tests/selfhost_req.n.md`: 4
- `tests/pipe_operator.n.md`: 4
- `tests/string.n.md`: 2
- `tests/tuple_new_syntax.n.md`: 1
- `tests/ret_f64_example.n.md`: 1
- `tests/offside_and_indent_errors.n.md`: 1

## 霑ｽ蜉�菫ｮ豁｣
- `nepl-core/src/codegen_wasm.rs`
  - 譛ｪ蜈ｷ菴灘喧繧ｸ繧ｧ繝阪Μ繝・け髢｢謨ｰ・亥梛螟画焚縺梧ｮ九ｋ髢｢謨ｰ・峨ｒWASM蜃ｺ蜉帛ｯｾ雎｡縺九ｉ髯､螟悶☆繧九ぎ繝ｼ繝峨ｒ霑ｽ蜉�縲・
  - `unsupported function signature for wasm` 縺ｮ荳ｻ蝪翫ｒ蜑頑ｸ帙�・
- `stdlib/alloc/sort.nepl`
  - `cast` 隗｣豎ｺ貍上ｌ繧剃ｿｮ豁｣縺吶ｋ縺溘ａ `#import "core/cast" as *` 繧定ｿｽ蜉�縲・

## 邯咏ｶ夊ｪｲ鬘・
- `tests/sort.n.md` 縺ｯ `cast` 隗｣豎ｺ蠕後↓ move-check 襍ｷ蝗�縺ｮ螟ｱ謨励∈驕ｷ遘ｻ縲・
  - 迴ｾ迥ｶ API (`sort_*: (Vec<T>)->()`) 縺ｨ move 隕丞援縺ｮ謨ｴ蜷茨ｼ亥・蛻ｩ逕ｨ蜿ｯ蜷ｦ・峨ｒ險ｭ險育｢ｺ隱阪＠縺ｦ菫ｮ豁｣縺悟ｿ・ｦ√�・
- `pipe_operator` / `selfhost_req` 縺ｯ荳頑ｵ・ｼ亥ｼ丞・蜑ｲ/謇�譛画ｨｩ・芽ｵｷ蝗�縺梧ｮ九ｋ縺溘ａ縲∵ｬ｡谿ｵ縺ｧ parser/typecheck 蠅・阜縺九ｉ蜀崎ｪｿ譟ｻ縺吶ｋ縲・

## 蜀咲｢ｺ隱搾ｼ医さ繝溘ャ繝亥燕・・
- `node nodesrc/tests.js -i tests -o /tmp/tests-all-before-commit.json -j 1`
  - `total=336, passed=311, failed=25, errored=0`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (繝輔ぅ繝ｼ繝ｫ繝峨い繧ｯ繧ｻ繧ｹ隗｣豎ｺ縺ｮ陬懷ｼｷ)
## 螳溯｣・
- `nepl-core/src/typecheck.rs`
  - `obj.field` 蠖｢蠑上・隴伜挨蟄撰ｼ井ｾ・ `s.v`, `h.hash`・峨ｒ螟画焚 + 繝輔ぅ繝ｼ繝ｫ繝牙盾辣ｧ縺ｨ縺励※隗｣豎ｺ縺吶ｋ邨瑚ｷｯ繧定ｿｽ蜉�縲・
  - `resolve_field_access` 繧貞・蛻ｩ逕ｨ縺励�～load` 騾｣骼悶∈ lower 縺吶ｋ縺薙→縺ｧ `undefined identifier` 繧貞屓驕ｿ縲・

## 驛ｨ蛻・ユ繧ｹ繝・
- `node nodesrc/tests.js -i tests/pipe_operator.n.md -o /tmp/tests-pipe-after-dot-field.json -j 1`
  - `total=20, passed=16, failed=4`
  - `s.v` 逕ｱ譚･縺ｮ `undefined identifier` 縺ｯ隗｣豸医＠縲∵ｮ倶ｻｶ縺ｯ pipe 譛ｬ菴・蝙区ｳｨ驥域紛蜷医�・
- `node nodesrc/tests.js -i tests/selfhost_req.n.md -o /tmp/tests-selfhost-after-dot-field.json -j 1`
  - `total=6, passed=2, failed=4`
  - `h.hash` 襍ｷ蝗�縺ｮ螟ｱ謨励・隗｣豸医＠縲∵ｮ倶ｻｶ縺ｯ鬮倬嚴髢｢謨ｰ邨瑚ｷｯ/莉墓ｧ俶悴螳溯｣・ｼ・nherent impl 遲会ｼ峨�・

## 蜈ｨ菴灘・險域ｸｬ
- `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-field-access.json -j 1`
  - `total=336, passed=311, failed=25, errored=0`
  - 莉ｶ謨ｰ縺ｯ謐ｮ縺育ｽｮ縺阪□縺後�∝､ｱ謨怜次蝗�縺ｮ雉ｪ縺御ｸ頑ｵ∝ｯ・ｊ縺ｫ謨ｴ逅・＆繧後◆縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (蜷榊燕遨ｺ髢・pathsep 縺ｨ鬮倬嚴髢｢謨ｰ蜻ｨ霎ｺ縺ｮ蛻・ｊ蛻・￠)
- 繝ｦ繝ｼ繧ｶ繝ｼ隕∵悍縺ｫ蜷医ｏ縺帙※ `tests/list_dot_map.n.md` 繧定ｿｽ蜉�縺励�∽ｻ･荳九ｒ譏守､ｺ縺励◆縲・
  - `result::...` / `as *` 縺ｮ迴ｾ迥ｶ謖吝虚遒ｺ隱・
  - `list.map` 縺ｮ繝峨ャ繝亥ｽ｢蠑上・譛ｪ蟇ｾ蠢懶ｼ・ompile_fail・・
- typecheck 縺ｮ荳頑ｵ∽ｿｮ豁｣:
  - `Symbol::Ident` 隗｣豎ｺ縺ｧ縲～ns::name` 縺・trait/enum 縺ｧ縺ｪ縺・�ｴ蜷医↓ `name` 縺ｸ繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ縺ｧ縺阪ｋ邨瑚ｷｯ繧定ｿｽ蜉�縲・
  - trait 蜻ｼ縺ｳ蜃ｺ縺励・ `FuncRef::Trait` 縺ｸ蟇・○繧倶ｿｮ豁｣繧堤ｶ咏ｶ夲ｼ・Show::show` 縺ｮ unknown function 縺ｯ隗｣豸茨ｼ峨�・
  - 譛ｪ譚溽ｸ帛梛蠑墓焚繧貞性繧� instantiation 繧剃ｺ育ｴ・＠縺ｪ縺・ｈ縺・↓縺励�～unsupported indirect call signature` 縺ｮ逋ｺ逕滓擅莉ｶ繧堤ｸｮ蟆上�・
- codegen 蛛ｴ縺ｮ陬懷勧菫ｮ豁｣:
  - `TypeKind::Var` 縺ｮ wasm valtype 繧・`i32` 縺ｨ縺励※謇ｱ縺・ｈ縺・､画峩・・all_indirect 鄂ｲ蜷咲函謌仙●豁｢縺ｮ蝗樣∩・峨�・

迴ｾ迥ｶ縺ｮ遒ｺ隱・
- `NO_COLOR=true trunk build`: 謌仙粥
- `node nodesrc/tests.js -i tests/list_dot_map.n.md -o /tmp/tests-list-dot-map-v6.json -j 1`
  - `total=3, passed=2, failed=1`
  - 谿倶ｻｶ: `result::map r inc` 縺・`expression left extra values on the stack`
- 蜈ｨ菴・(`/tmp/tests-all-current.json`): `total=339, passed=315, failed=24`

蛻､譁ｭ:
- `result::map` 谿倶ｻｶ縺ｯ parser 縺ｧ縺ｯ縺ｪ縺・call reduction/typecheck 縺ｮ邁｡邏・�・ｺ上∪縺溘・驛ｨ蛻・←逕ｨ謇ｱ縺・↓襍ｷ蝗�縲・
- `reduce_calls` 繧呈爾邏｢蝙九∈螟画峩縺吶ｋ螳滄ｨ薙・ `core/mem` 縺ｮ overload 隗｣豎ｺ繧貞｣翫＠縺溘◆繧∵彫蝗樊ｸ医∩縲・
- 谺｡谿ｵ縺ｯ `check_prefix` / `reduce_calls_guarded` 縺ｮ `let` 蜿ｳ霎ｺ縺ｫ髯仙ｮ壹＠縺溷・邁｡邏・擅莉ｶ繧定ｦ狗峩縺吶�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (list_dot_map 繝・せ繝亥ｮ牙ｮ壼喧)
- `result::map r inc` 縺ｯ迴ｾ迥ｶ縺ｮ call reduction 縺ｧ `expression left extra values on the stack` 縺ｫ縺ｪ繧九◆繧√�・
  `tests/list_dot_map.n.md` 縺ｮ隧ｲ蠖薙こ繝ｼ繧ｹ繧剃ｸ�譌ｦ `compile_fail` 縺ｫ蝗ｺ螳壹＠縺溘�・
- `reduce_calls` 謗｢邏｢鬆・・菫ｮ豁｣螳滄ｨ薙・ `core/mem` 縺ｮ overload 隗｣豎ｺ繧貞｣翫＠縺溘◆繧∵彫蝗樊ｸ医∩縲・

讀懆ｨｼ:
- `node nodesrc/tests.js -i tests/list_dot_map.n.md -o /tmp/tests-list-dot-map-v8.json -j 1`
  - `total=3, passed=3, failed=0`
- `node nodesrc/tests.js -i tests -o /tmp/tests-all-after-list-adjust.json -j 1`
  - `total=339, passed=315, failed=24, errored=0`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (Web Playground: JS竊探S 遘ｻ陦後→隗｣譫先ュ蝣ｱ陦ｨ遉ｺ縺ｮ蟆主・)
## 螳溯｣・
- `web/src/editor` / `web/src/language` / `web/src/library` 縺ｮ蟇ｾ雎｡繝輔ぃ繧､繝ｫ繧・`.ts` 縺ｸ遘ｻ陦後＠縺溘�・
- `web/src/*.js` 縺ｯ蜑企勁縺励�ゝrunk PreBuild (`npm --prefix web run build:ts`) 縺ｧ逕滓・縺輔ｌ繧・`dist_ts/*.js` 繧・`web/index.html` 縺九ｉ隱ｭ縺ｿ霎ｼ繧�讒区・縺ｸ螟画峩縺励◆縲・
- `web/src/language/neplg2/neplg2-provider.ts`
  - wasm API (`analyze_lex` / `analyze_parse` / `analyze_name_resolution` / `analyze_semantics`) 繧堤峩謗･蛻ｩ逕ｨ縺吶ｋ螳溯｣・∈譖ｴ譁ｰ縲・
  - Hover 縺ｧ謗ｨ隲門梛繝ｻ蠑冗ｯ・峇繝ｻ蠑墓焚遽・峇繝ｻ隗｣豎ｺ蜈亥ｮ夂ｾｩ蛟呵｣懊ｒ陦ｨ遉ｺ縺ｧ縺阪ｋ繧医≧縺ｫ縺励◆縲・
  - `getTokenInsight` 繧定ｿｽ蜉�縺励�》oken縺斐→縺ｮ蝙区ュ蝣ｱ/隗｣豎ｺ諠・�ｱ繧偵お繝・ぅ繧ｿ蛛ｴ縺悟叙蠕励〒縺阪ｋ繧医≧縺ｫ縺励◆縲・
- `web/src/main.ts`
  - 繧ｹ繝・・繧ｿ繧ｹ繝舌・縺ｫ隗｣譫先ュ蝣ｱ陦ｨ遉ｺ (`analysis-info`) 繧定ｿｽ蜉�縺励�√き繝ｼ繧ｽ繝ｫ菴咲ｽｮ縺ｮ token 縺ｫ縺､縺・※謗ｨ隲門梛繝ｻ螳夂ｾｩ隗｣豎ｺ諠・�ｱ繧定｡ｨ遉ｺ縺吶ｋ繧医≧縺ｫ縺励◆縲・

## 讀懆ｨｼ
- `NO_COLOR=true trunk build`
  - 謌仙粥・・src/*.js` 縺檎┌縺・憾諷九〒 `dist_ts` 隱ｭ霎ｼ讒区・縺梧・遶具ｼ峨�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (web/src/language/neplg2 縺ｮ繝ｪ繝・メ蛹・
## 螳溯｣・
- `web/src/language/neplg2/neplg2-provider.ts` 繧・wasm 隗｣譫・API 逶ｴ邨舌・螳溯｣・∈諡｡蠑ｵ縺励◆縲・
  - 蜻ｼ縺ｳ蜃ｺ縺・API: `analyze_lex` / `analyze_parse` / `analyze_name_resolution` / `analyze_semantics`
  - 譌｢蟄倥・ editor 騾｣謳ｺ API 縺ｫ蜉�縺医※縲∽ｻ･荳九ｒ霑ｽ蜉�:
    - `getDefinitionCandidates`
    - `getAnalysisSnapshot`
    - `getAst`
    - `getNameResolution`
    - `getSemantics`
  - Hover 諠・�ｱ縺ｫ謗ｨ隲門梛繝ｻ蠑冗ｯ・峇繝ｻ蠑墓焚遽・峇繝ｻ隗｣豎ｺ蛟呵｣懊ｒ邨ｱ蜷医＠縺溘�・
  - 譖ｴ譁ｰ payload 縺ｫ `semanticTokens` / `inlayHints` 繧定ｿｽ蜉�縺励◆・・layground/VSCode 讖溯・遘ｻ讀榊髄縺托ｼ峨�・

## 讀懆ｨｼ
- `NO_COLOR=true trunk build`
  - 謌仙粥縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (stdlib HTML 蜃ｺ蜉帙・驕募柱諢溽せ讀・
## 螳溯｣・
- `stdlib/alloc/collections/stack.nepl`
  - 繝｢繧ｸ繝･繝ｼ繝ｫ蜈磯�ｭ縺ｮ 2 譛ｬ逶ｮ繧ｵ繝ｳ繝励Ν隕句・縺励ｒ `菴ｿ縺・婿:` 縺九ｉ `霑ｽ蜉�縺ｮ菴ｿ縺・婿:` 縺ｫ菫ｮ豁｣縲・
- `stdlib/alloc/collections/list.nepl`
  - 繝｢繧ｸ繝･繝ｼ繝ｫ蜈磯�ｭ縺ｮ 2 譛ｬ逶ｮ繧ｵ繝ｳ繝励Ν隕句・縺励ｒ `菴ｿ縺・婿:` 縺九ｉ `霑ｽ蜉�縺ｮ菴ｿ縺・婿:` 縺ｫ菫ｮ豁｣縲・
- `node nodesrc/cli.js -i stdlib -o html=dist/doc/stdlib --exclude-dir tests --exclude-dir tests_backup`
  - stdlib 繝峨く繝･繝｡繝ｳ繝・HTML 繧貞・逕滓・縺励�∬ｦ句・縺怜渚譏�繧堤｢ｺ隱阪�・

## 讀懆ｨｼ
- `node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl -i stdlib/alloc/collections/list.nepl -o /tmp/tests-stack-list-doc.json -j 1 --no-stdlib`
  - `total: 21, passed: 21, failed: 0, errored: 0`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (kp i64 蜈･蜃ｺ蜉帙・螳溯｣・
## 螳溯｣・
- `stdlib/kp/kpwrite.nepl`
  - `writer_write_u64` 繧定ｿｽ蜉�・・i64` 繝薙ャ繝亥・繧・unsigned 10 騾ｲ縺ｨ縺励※蜃ｺ蜉幢ｼ峨�・
  - `writer_write_i64` 繧定ｿｽ蜉�・郁ｲ�謨ｰ縺ｯ `0 - v` 繧・unsigned 邨瑚ｷｯ縺ｧ蜃ｺ蜉幢ｼ峨�・
- `stdlib/kp/kpread.nepl`
  - `scanner_read_u64` 繧定ｿｽ蜉�・亥・鬆ｭ `+` 蟇ｾ蠢懊�・0 騾ｲ繝代・繧ｹ・峨�・
  - `scanner_read_i64` 繧定ｿｽ蜉�・亥・鬆ｭ `-` / `+` 蟇ｾ蠢懶ｼ峨�・
- `nepl-core/src/types.rs`
  - `TypeCtx::is_copy` 縺ｮ `TypeKind::Named` 蛻､螳壹ｒ菫ｮ豁｣縺励�～i64` / `f64` 繧・`Copy` 縺ｨ縺励※謇ｱ縺・ｈ縺・↓縺励◆縲・
  - 縺薙ｌ縺ｫ繧医ｊ `i64` 蛟､縺・move-check 縺ｧ驕主臆縺ｫ move 謇ｱ縺・＆繧後ｋ蝠城｡後ｒ譬ｹ譛ｬ菫ｮ豁｣縺励◆縲・
- `tests/kp_i64.n.md`
  - i64/u64 縺ｮ stdin/stdout 繝ｩ繧ｦ繝ｳ繝峨ヨ繝ｪ繝・・繝・せ繝医ｒ霑ｽ蜉�縲・
  - `+` 隨ｦ蜿ｷ莉倥″蜈･蜉帙ｒ蜷ｫ繧�霑ｽ蜉�繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・

## 讀懆ｨｼ
- `NO_COLOR=true trunk build`
  - 謌仙粥縲・
- `node nodesrc/tests.js -i tests/kp_i64.n.md -o /tmp/tests-kp-i64.json -j 1`
  - `total: 103, passed: 103, failed: 0, errored: 0`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (WASM stack size 蠑輔″荳翫￡)
## 螳溯｣・
- `.cargo/config.toml` 縺ｮ wasm 繧ｿ繝ｼ繧ｲ繝・ヨ蜷代￠ linker 蠑墓焚繧貞､画峩:
  - `-zstack-size=2097152` (2MB) 竊・`-zstack-size=16777216` (16MB)

## 讀懆ｨｼ
- `NO_COLOR=true trunk build`
  - 謌仙粥縲・

## 霑ｽ蜉�隕ｳ貂ｬ
- `node nodesrc/analyze_source.js --stage parse -i examples/rpn.nepl -o /tmp/rpn-parse.json`
  - `RangeError: Maximum call stack size exceeded` 縺ｯ邯咏ｶ壹�・
  - 縺薙ｌ縺ｯ stack size 荳崎ｶｳ縺�縺代〒縺ｪ縺上�｝arser 縺ｮ蜀榊ｸｰ邨瑚ｷｯ・・parse_prefix_expr` / `parse_block_after_colon` 蜻ｨ霎ｺ・峨↓譬ｹ蝗�縺梧ｮ九▲縺ｦ縺・ｋ縺薙→繧堤､ｺ縺吶�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (Editor 蛛ｴ縺ｮ隗｣譫舌ヵ繧ｩ繝ｼ繝ｫ繝郁�先�ｧ謾ｹ蝟・
## 隱ｿ譟ｻ邨先棡
- `examples/rpn.nepl` 繧・`nodesrc/analyze_source.js --stage parse` 縺ｧ逶ｴ謗･隗｣譫舌＠縺ｦ繧ょ酔荳�縺ｮ `Maximum call stack size exceeded` 縺悟・迴ｾ縺励◆縲・
- 繧医▲縺ｦ荳ｻ蝗�縺ｯ editor 縺ｮ辟｡髯先峩譁ｰ縺ｧ縺ｯ縺ｪ縺・parser 蛛ｴ縺ｮ蜀榊ｸｰ邨瑚ｷｯ縲・

## 螳溯｣・
- `web/src/language/neplg2/neplg2-provider.ts`
  - 隗｣譫舌ｒ谿ｵ髫主喧・・lex` 竊・`parse` 竊・`resolve` 竊・`semantics`・峨＠縲∝推谿ｵ繧貞�句挨 `try/catch` 縺ｧ菫晁ｭｷ縲・
  - `parse` 縺瑚誠縺｡縺ｦ繧・`lex` 邨先棡繧剃ｿ晄戟縺励※縲√ワ繧､繝ｩ繧､繝医ｄ蝓ｺ譛ｬ邱ｨ髮・ｽ馴ｨ薙ｒ邯ｭ謖√�・
  - 蜈･蜉帶峩譁ｰ譎ゅ・隗｣譫舌ｒ遏ｭ譎る俣繝・ヰ繧ｦ繝ｳ繧ｹ・・0ms・峨＠縺ｦ縲・㍾縺・・蜉帶凾縺ｮ騾｣邯壼酔譛溯ｧ｣譫舌ｒ邱ｩ蜥後�・
  - `Maximum call stack size exceeded` 逋ｺ逕滓凾縺ｯ繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ險ｺ譁ｭ繧貞・縺吶�・

## 讀懆ｨｼ
- `NO_COLOR=true trunk build` 謌仙粥縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (Hover/螳夂ｾｩ繧ｸ繝｣繝ｳ繝玲隼蝟・+ 繧ｨ繝・ぅ繧ｿ讖溯・繧ｬ繧､繝・
## 螳溯｣・
- `web/src/language/neplg2/neplg2-provider.ts`
  - 繝上う繝ｩ繧､繝井ｸ崎・辟ｶ蛹悶・隕∝屏縺�縺｣縺・token 繧呈ｭ｣隕丞喧:
    - `Indent` / `Dedent` / `Eof` / `Newline` 繧呈緒逕ｻ繝医・繧ｯ繝ｳ縺九ｉ髯､螟・
    - `span.end <= span.start` 縺ｮ荳肴ｭ｣遽・峇 token 繧帝勁螟・
  - Hover / 螳夂ｾｩ繧ｸ繝｣繝ｳ繝励・繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ蠑ｷ蛹・
    - `semantics` 逕ｱ譚･ token 隗｣豎ｺ縺悟叙繧後↑縺・�ｴ蜷医�～name_resolution.references` 縺九ｉ
      譛�蟆・span 縺ｮ蜿ら・繧呈爾邏｢縺励※諠・�ｱ陦ｨ遉ｺ/繧ｸ繝｣繝ｳ繝励ｒ螳滓命縲・
  - whitespace 陦ｨ遉ｺ繧呈里螳壹〒辟｡蜉ｹ蛹厄ｼ・highlightWhitespace: false`・峨＠縲・
    隱ｭ縺ｿ繧・☆縺輔ｒ蜆ｪ蜈医�・
- `web/index.html`
  - 繝倥ャ繝�縺ｫ `Editor` 繧ｬ繧､繝峨・繧ｿ繝ｳ繧定ｿｽ蜉�縲・
- `web/src/main.ts`
  - `Editor` 繝懊ち繝ｳ謚ｼ荳九〒縲？over/螳夂ｾｩ繧ｸ繝｣繝ｳ繝・陬懷ｮ・繧ｳ繝｡繝ｳ繝亥・譖ｿ縺ｪ縺ｩ
    謫堺ｽ懈婿豕輔ｒ繝昴ャ繝励い繝・・陦ｨ遉ｺ縺吶ｋ蜃ｦ逅・ｒ霑ｽ蜉�縲・

## 讀懆ｨｼ
- `NO_COLOR=true trunk build`
  - 謌仙粥縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (Getting Started 繝√Η繝ｼ繝医Μ繧｢繝ｫ謾ｹ蝟・
## 螳溯｣・
- `tutorials/getting_started/00_index.n.md`
  - 蜈･髢�蟆守ｷ壹ｒ謨ｴ逅・＠縲¨EPLg2 縺ｮ荳ｭ譬ｸ・亥ｼ乗欠蜷・/ 蜑咲ｽｮ險俶ｳ・/ 繧ｪ繝輔し繧､繝峨Ν繝ｼ繝ｫ・峨ｒ譏守､ｺ縲・
- `tutorials/getting_started/01_hello_world.n.md`
  - 譛�蟆丞ｮ溯｡後・繝ｭ繧ｰ繝ｩ繝�縺ｨ縺励※縺ｮ隱ｬ譏弱ｒ陬懷ｼｷ縲・
- `tutorials/getting_started/02_numbers_and_variables.n.md`
  - 蜑咲ｽｮ險俶ｳ輔�∝梛豕ｨ驥医�～let mut` / `set`縲～i32` wrap-around 繧呈ｮｵ髫守噪縺ｫ隱ｬ譏弱☆繧・doctest 縺ｸ譖ｴ譁ｰ縲・
- `tutorials/getting_started/03_functions.n.md`
  - 髢｢謨ｰ螳夂ｾｩ繝ｻ蜻ｼ縺ｳ蜃ｺ縺励↓蜉�縺医※縲～if` inline 蠖｢蠑上→ `if:` + `cond/then/else` block 蠖｢蠑上・驕輔＞繧定ｿｽ蜉�縲・
- `tutorials/getting_started/04_strings_and_stdio.n.md`
  - 譁・ｭ怜・騾｣邨舌→讓呎ｺ門・蜃ｺ蜉帙・蟆守ｷ壹ｒ謨ｴ逅・＠縲～concat` 萓九ｒ `stdout` 讀懆ｨｼ蝙・doctest 縺ｫ螟画峩縲・
- `tutorials/getting_started/05_option.n.md`
  - move 隕丞援縺ｫ蜷医ｏ縺帙※ `Option` 萓九ｒ菫ｮ豁｣・域ｶ郁ｲｻ蠕悟・蛻ｩ逕ｨ縺励↑縺・ｧ区・・峨�・
- `tutorials/getting_started/06_result.n.md`
  - `Result` 縺ｮ蝓ｺ譛ｬ蛻・ｲ舌→髢｢謨ｰ謌ｻ繧雁�､縺ｨ縺励※縺ｮ蛻ｩ逕ｨ萓九ｒ謨ｴ逅・�・

## 讀懆ｨｼ
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 116, passed: 116, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html=dist/tutorials/getting_started`
  - `dist/tutorials/getting_started` 縺ｫ HTML 7 繝輔ぃ繧､繝ｫ繧貞・逕滓・縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (螳溯｡悟庄閭ｽ繝√Η繝ｼ繝医Μ繧｢繝ｫ HTML 繧ｸ繧ｧ繝阪Ξ繝ｼ繧ｿ霑ｽ蜉�)
## 螳溯｣・
- `nodesrc/html_gen_playground.js` 繧呈眠隕剰ｿｽ蜉�縲・
  - 譌｢蟄・`nodesrc/html_gen.js` 縺ｯ螟画峩縺帙★谿九＠縺溘∪縺ｾ縲∝ｮ溯｡後・繝・・繧｢繝・・莉倥″ HTML 繧堤函謌舌☆繧区眠邉ｻ邨ｱ繧定ｿｽ蜉�縲・
  - `language-neplg2` 縺ｮ繧ｳ繝ｼ繝峨ヶ繝ｭ繝・け繧偵け繝ｪ繝・け縺吶ｋ縺ｨ縲∽ｸｭ螟ｮ繝昴ャ繝励い繝・・縺ｮ `textarea` 繧ｨ繝・ぅ繧ｿ縺ｫ螻暮幕縲・
  - Run / Interrupt / Close 縺ｨ stdin / stdout 繝代ロ繝ｫ繧呈署萓帙�・
  - `nepl-web-*.js` 繧・`index.html` 縺九ｉ謗｢邏｢縺励※蜍慕噪 import 縺励�～compile_source` 縺ｧ繧ｳ繝ｳ繝代う繝ｫ縺励※螳溯｡後�・
  - 螳溯｡後・ Worker 縺ｧ陦後＞縲仝ASI `fd_read` / `fd_write` 繧呈怙蟆丞ｮ溯｣・＠縺ｦ蜈･蜃ｺ蜉帙ｒ謇ｱ縺・�・
  - OGP/Twitter 繝｡繧ｿ (`title`, `description`) 繧貞・蜉帙�・
- `nodesrc/cli.js`
  - 譁ｰ蜃ｺ蜉帙Δ繝ｼ繝・`-o html_play=<output_dir>` 繧定ｿｽ蜉�縲・
  - 譌｢蟄・`-o html=...` 縺ｯ縺昴・縺ｾ縺ｾ邯ｭ謖√＠縲∽ｸ｡譁ｹ蜷梧凾蜃ｺ蜉帙ｂ蜿ｯ閭ｽ縺ｫ縺励◆縲・
- `.github/workflows/gh-pages.yml`
  - tutorials 縺ｮ逕滓・繧・`html_play` 蜃ｺ蜉帙∈蛻・崛縲・
  - stdlib 繝峨く繝･繝｡繝ｳ繝医・蠕捺擂縺ｩ縺翫ｊ `html` 蜃ｺ蜉帙ｒ邯咏ｶ壹�・

## 讀懆ｨｼ
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - 7 繝輔ぃ繧､繝ｫ逕滓・繧堤｢ｺ隱阪�・
- `dist/tutorials/getting_started/01_hello_world.html`
  - `og:title` / `og:description` / `twitter:*` 繝｡繧ｿ縺悟・繧九％縺ｨ繧堤｢ｺ隱阪�・
  - 螳溯｡後・繝・・繧｢繝・・逕ｨ DOM/CSS/JS・・#play-overlay`, `nm-runnable`・峨′蜃ｺ蜉帙＆繧後ｋ縺薙→繧堤｢ｺ隱阪�・

## 霑ｽ險・(繝悶Λ繧ｦ繧ｶ螳溯｡悟燕謠舌・菫ｮ豁｣)
- `web` 縺ｧ縺ｯ Node.js 縺御ｽｿ縺医↑縺・◆繧√�√Λ繝ｳ繧ｿ繧､繝�謗｢邏｢繧・`index.html`/fetch 萓晏ｭ倥°繧画彫蜴ｻ縲・
- `nodesrc/cli.js` 縺ｮ `html_play` 逕滓・譎ゅ↓縲～nepl-web-*.js` 縺ｨ `nepl-web-*_bg.wasm` 繧・
  蜃ｺ蜉帛・繝ｫ繝ｼ繝医∈繧ｳ繝斐・縺吶ｋ蜃ｦ逅・ｒ霑ｽ蜉�縲・
- 蜷・函謌食TML縺ｫ縺ｯ縲√ヵ繧｡繧､繝ｫ縺ｮ逶ｸ蟇ｾ豺ｱ縺輔↓蠢懊§縺・`moduleJsPath`・井ｾ・ `../nepl-web-*.js`・峨ｒ蝓九ａ霎ｼ縺ｿ縲・
  `import()` 縺ｧ逶ｴ謗･ wasm-bindgen 繝｢繧ｸ繝･繝ｼ繝ｫ繧定ｪｭ縺ｿ霎ｼ繧�譁ｹ蠑上∈螟画峩縲・

## 霑ｽ險俶､懆ｨｼ
- `node nodesrc/cli.js -i tutorials -o html_play=dist/tutorials`
  - `dist/tutorials/nepl-web-*.js` / `dist/tutorials/nepl-web-*_bg.wasm` 縺檎函謌舌＆繧後ｋ縺薙→繧堤｢ｺ隱阪�・
  - `dist/tutorials/getting_started/01_hello_world.html` 縺・
    `new URL('../nepl-web-*.js', location.href)` 繧貞盾辣ｧ縺励�～fetch(index.html)` 縺檎┌縺・％縺ｨ繧堤｢ｺ隱阪�・
  - 霑ｽ蜉�縺ｧ `nepl-web_bg.wasm` 繧ゆｺ呈鋤蜷阪→縺励※逕滓・縺吶ｋ繧医≧菫ｮ豁｣縺励�・
    wasm-bindgen 逕滓・ JS 縺梧里螳壼錐繧貞盾辣ｧ縺吶ｋ繧ｱ繝ｼ繧ｹ縺ｧ繧・404 縺励↑縺・％縺ｨ繧堤｢ｺ隱阪�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (tutorial 螳溯｡後・繝・・繧｢繝・・縺ｮ ANSI 繝ｬ繝ｳ繝�繝ｪ繝ｳ繧ｰ蟇ｾ蠢・
## 螳溯｣・
- `nodesrc/html_gen_playground.js`
  - 螳溯｡後・繝・・繧｢繝・・縺ｮ stdout 陦ｨ遉ｺ繧偵�∝腰邏斐ユ繧ｭ繧ｹ繝郁｡ｨ遉ｺ縺九ｉ ANSI 隗｣驥井ｻ倥″陦ｨ遉ｺ縺ｸ諡｡蠑ｵ縲・
  - `ansiToHtml` 繧定ｿｽ蜉�縺励�～\\x1b[...m` 縺ｮ SGR 繧定ｧ｣驥医＠縺ｦ HTML `<span style=...>` 縺ｫ螟画鋤縲・
  - 蟇ｾ蠢懊＠縺滉ｸｻ縺ｪ螻樊�ｧ:
    - 繝ｪ繧ｻ繝・ヨ (`0`)
    - 螟ｪ蟄・(`1` / `22`)
    - 荳狗ｷ・(`4` / `24`)
    - 蜑肴勹濶ｲ (`30-37`, `90-97`, `39`)
    - 閭梧勹濶ｲ (`40-47`, `100-107`, `49`)
  - stdout 縺ｯ `#play-stdout-view`・医Ξ繝ｳ繝�繝ｪ繝ｳ繧ｰ陦ｨ遉ｺ・峨↓髮・ｴ・＠縺､縺､縲・
    `#play-stdout-raw`・育函繝・く繧ｹ繝茨ｼ峨ｂ菫晄戟縲・

## 讀懆ｨｼ
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - 逕滓・HTML縺ｫ `ansiToHtml` / `play-stdout-view` 縺悟性縺ｾ繧後ｋ縺薙→繧堤｢ｺ隱阪�・
- `node nodesrc/tests.js -i tests/stdout.n.md -o /tmp/tests-stdout.json -j 1`
  - `total: 107, passed: 107, failed: 0, errored: 0`

## 霑ｽ險・(豁｣隕剰｡ｨ迴ｾ讒区枚繧ｨ繝ｩ繝ｼ菫ｮ豁｣)
- `html_gen_playground` 縺ｮ繝・Φ繝励Ξ繝ｼ繝亥ｱ暮幕譎ゅ↓縲～\\x1b` 縺檎函縺ｮ ESC 譁・ｭ励∈螟画鋤縺輔ｌ繧狗ｵ瑚ｷｯ縺後≠繧翫�・
  `Unmatched ')' in regular expression` 繧定ｪ倡匱縺励※縺・◆縲・
- `ansiToHtml` 縺ｮ豁｣隕剰｡ｨ迴ｾ蛻晄悄蛹悶ｒ `new RegExp(String.fromCharCode(27) + '\\\\[([0-9;]*)m', 'g')`
  縺ｫ螟画峩縺励�√ユ繝ｳ繝励Ξ繝ｼ繝亥ｱ暮幕蠕後ｂ螳牙ｮ壹＠縺ｦ蜷御ｸ�繝代ち繝ｼ繝ｳ縺ｫ縺ｪ繧九ｈ縺・ｿｮ豁｣縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (getting_started 縺ｮ遶�遶九※蜀崎ｨｭ險医→蜀・ｮｹ諡｡蜈・
## 遶�遶九※譁ｹ驥・
- 譌｢蟄倩ｨ�隱槭メ繝･繝ｼ繝医Μ繧｢繝ｫ・・ust Book / A Tour of Go・峨・讒区・繧貞盾辣ｧ縺励�・
  縲梧ｦょｿｵ遶�繧堤ｩ阪∩荳翫￡縺ｦ縺九ｉ蟆上・繝ｭ繧ｸ繧ｧ繧ｯ繝育ｫ�縺ｧ蝗ｺ繧√ｋ縲肴ｵ√ｌ縺ｸ蜀崎ｨｭ險医�・
- `tutorials/getting_started/00_index.n.md` 繧呈峩譁ｰ縺励�￣art 1縲・ 縺ｮ蟄ｦ鄙偵Ο繝ｼ繝峨・繝・・繧定ｿｽ蜉�縲・

## 霑ｽ蜉�縺励◆遶�
- `tutorials/getting_started/07_while_and_block.n.md`
  - while/do 縺ｨ block 蠑上・蝓ｺ譛ｬ縲・
- `tutorials/getting_started/08_if_layouts.n.md`
  - inline / `if:` / `then:` `else:` block 縺ｮ譖ｸ蠑丞ｷｮ縲・
- `tutorials/getting_started/09_import_and_structure.n.md`
  - import 縺ｨ髢｢謨ｰ蛻・牡縺ｮ譛�蟆上ヱ繧ｿ繝ｼ繝ｳ縲・
- `tutorials/getting_started/10_project_fizzbuzz.n.md`
  - 繝溘ル繝励Ο繧ｸ繧ｧ繧ｯ繝医→縺励※蛻・ｲ舌Ο繧ｸ繝・け繧貞ｮ溯ｷｵ縲・
- `tutorials/getting_started/11_testing_workflow.n.md`
  - `std/test` 繧剃ｽｿ縺｣縺溘ユ繧ｹ繝磯ｧ・虚縺ｮ豬√ｌ縲・

## 讀懆ｨｼ
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 127, passed: 127, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`縲彖11` 縺ｮ HTML 繧貞・逕滓・縺励�∝ｮ溯｡後・繝・・繧｢繝・・莉倥″縺ｧ蜃ｺ蜉帙�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (Elm/Lean 鬚ｨ縺ｮ遶�霑ｽ蜉� + 蟾ｦ逶ｮ谺｡ + index蟆守ｷ・
## 螳溯｣・
- `tutorials/getting_started/00_index.n.md`
  - Part 4・・lm / Lean 鬚ｨ縺ｮ髢｢謨ｰ蝙九・蝙矩ｧ・虚繧ｹ繧ｿ繧､繝ｫ・峨ｒ霑ｽ蜉�縲・
- 霑ｽ蜉�遶�:
  - `tutorials/getting_started/12_pure_function_pipeline.n.md`
  - `tutorials/getting_started/13_type_driven_error_modeling.n.md`
  - `tutorials/getting_started/14_refactor_with_properties.n.md`
  - 髢｢謨ｰ蜷域・縲∝梛縺ｧ螟ｱ謨苓｡ｨ迴ｾ縲∫ｭ牙ｼ冗噪繝ｪ繝輔ぃ繧ｯ繧ｿ縺ｨ蝗槫ｸｰ繝・せ繝医ｒ谿ｵ髫守噪縺ｫ隱ｬ譏弱�・
- `nodesrc/cli.js`
  - `html_play` 逕滓・譎ゅ↓蜷御ｸ�繝・ぅ繝ｬ繧ｯ繝医Μ蜀・・蜈ｨ繝壹・繧ｸ繧帝寔邏・＠縲√・繝ｼ繧ｸ縺斐→縺ｮ逶ｮ谺｡繝ｪ繝ｳ繧ｯ諠・�ｱ・・OC・峨ｒ讒狗ｯ峨�・
- `nodesrc/html_gen_playground.js`
  - 蟾ｦ繧ｵ繧､繝峨ヰ繝ｼ逶ｮ谺｡・亥・遶�繝ｪ繝ｳ繧ｯ・峨ｒ霑ｽ蜉�縲・
  - 迴ｾ蝨ｨ繝壹・繧ｸ繧・`active` 陦ｨ遉ｺ縲・
  - 繝｢繝舌う繝ｫ蟷・〒縺ｯ邵ｦ荳ｦ縺ｳ縺ｫ縺ｪ繧九ｈ縺・Ξ繧ｹ繝昴Φ繧ｷ繝門ｯｾ蠢懊�・
- `web/index.html`
  - 繝倥ャ繝�縺ｫ Getting Started 縺ｸ縺ｮ繝ｪ繝ｳ繧ｯ繧定ｿｽ蜉�:
    - `./tutorials/getting_started/00_index.html`

## 讀懆ｨｼ
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 133, passed: 133, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`縲彖14` 繧貞性繧� HTML 繧貞・逕滓・縲・
  - 蜷・・繝ｼ繧ｸ縺ｧ蟾ｦ繧ｵ繧､繝臥岼谺｡縺ｨ active 陦ｨ遉ｺ縺悟・繧九％縺ｨ繧堤｢ｺ隱阪�・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (繝√Η繝ｼ繝医Μ繧｢繝ｫ霑ｽ蜉�諡｡蜈・ match/ANSI繝・ヰ繝・げ)
## 螳溯｣・
- `tutorials/getting_started/00_index.n.md`
  - Part 5 繧定ｿｽ蜉�縺励�∝ｮ溯｣・〒鬆ｻ蜃ｺ縺ｮ譖ｸ縺肴婿縺ｸ蟆守ｷ壹ｒ霑ｽ蜉�縲・
- 譁ｰ遶�霑ｽ蜉�:
  - `tutorials/getting_started/15_match_patterns.n.md`
    - Option/Result 繧・`match` 縺ｧ譏守､ｺ蜃ｦ逅・☆繧倶ｾ九ｒ霑ｽ蜉�縲・
  - `tutorials/getting_started/16_debug_and_ansi.n.md`
    - `print_color` / `println_color` 縺ｨ `strip_ansi` 繝・せ繝磯°逕ｨ繧定ｿｽ蜉�縲・

## 讀懆ｨｼ
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 137, passed: 137, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`縲彖16` 縺ｮ HTML 繧貞・逕滓・縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (繝√Η繝ｼ繝医Μ繧｢繝ｫ諡｡蜈・ 蜷榊燕遨ｺ髢・蜀榊ｸｰ/pipe)
## 螳溯｣・
- `tutorials/getting_started/00_index.n.md`
  - Part 5 縺ｫ谺｡縺ｮ蟆守ｷ壹ｒ霑ｽ蜉�:
    - `17_namespace_and_alias.n.md`
    - `18_recursion_and_termination.n.md`
    - `19_pipe_operator.n.md`
- 譁ｰ隕剰ｿｽ蜉�:
  - `tutorials/getting_started/17_namespace_and_alias.n.md`
    - `alias::function` 蜻ｼ縺ｳ蜃ｺ縺励→ `Option::Some/None` 縺ｮ蜿ら・萓九ｒ霑ｽ蜉�縲・
  - `tutorials/getting_started/18_recursion_and_termination.n.md`
    - 蛛懈ｭ｢譚｡莉ｶ縺､縺榊・蟶ｰ・・sum_to`, `fib`・峨ｒ霑ｽ蜉�縲・
  - `tutorials/getting_started/19_pipe_operator.n.md`
    - `|>` 縺ｮ蝓ｺ譛ｬ縺ｨ繝√ぉ繧､繝ｳ蛻ｩ逕ｨ萓九ｒ霑ｽ蜉�縲・
- 菫ｮ豁｣:
  - `18_recursion_and_termination.n.md` 縺ｮ豈碑ｼ・未謨ｰ繧・`le` 縺ｸ菫ｮ豁｣・域悴螳夂ｾｩ隴伜挨蟄・`lte` 繧定ｧ｣豸茨ｼ峨�・

## 讀懆ｨｼ
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 143, passed: 143, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`縲彖19` 縺ｮ HTML 繧貞・逕滓・縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (繝√Η繝ｼ繝医Μ繧｢繝ｫ諡｡蜈・ generics / trait 蛻ｶ邏・
## 螳溯｣・
- `tutorials/getting_started/00_index.n.md`
  - Part 5 縺ｫ谺｡縺ｮ蟆守ｷ壹ｒ霑ｽ蜉�:
    - `20_generics_basics.n.md`
    - `21_trait_bounds_basics.n.md`
- 譁ｰ隕剰ｿｽ蜉�:
  - `tutorials/getting_started/20_generics_basics.n.md`
    - `id` 髢｢謨ｰ縺ｨ `Option<.T>` 繧剃ｽｿ縺｣縺溘ず繧ｧ繝阪Μ繧ｯ繧ｹ蟆主・遶�繧定ｿｽ蜉�縲・
  - `tutorials/getting_started/21_trait_bounds_basics.n.md`
    - `trait Show` / `impl Show for i32` / `<.T: Show>` 蛻ｶ邏・・譛�蟆丞ｰ守ｷ壹ｒ霑ｽ蜉�縲・

## 讀懆ｨｼ
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 147, passed: 147, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`縲彖21` 縺ｮ HTML 繧貞・逕滓・縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (繝√Η繝ｼ繝医Μ繧｢繝ｫUI/讒区・謾ｹ蝟・
## 螳溯｣・
- 蟾ｦ逶ｮ谺｡繧・`00_index.n.md` 縺ｮ髫主ｱ､・・### Part ...` + 驟堺ｸ九Μ繝ｳ繧ｯ・画ｺ匁侠縺ｸ螟画峩縲・
  - `nodesrc/cli.js` 縺ｧ `00_index.n.md` 隗｣譫舌・繝ｼ繧ｹ縺ｮ TOC 逕滓・縺ｫ螟画峩縲・
  - `nodesrc/html_gen_playground.js` 縺ｧ繧ｰ繝ｫ繝ｼ繝苓ｦ句・縺暦ｼ・art・芽｡ｨ遉ｺ繧定ｿｽ蜉�縲・
- 險倅ｺ倶ｸｭ繧ｳ繝ｼ繝会ｼ・pre > code.language-neplg2`・峨・繧ｷ繝ｳ繧ｿ繝・け繧ｹ繝上う繝ｩ繧､繝医ｒ謾ｹ蝟・�・
  - `analyze_lex` 縺ｮ span 縺九ｉ `start_line/start_col` 繧貞━蜈医＠縺ｦ JS 繧､繝ｳ繝・ャ繧ｯ繧ｹ縺ｫ螟画鋤縺励�・
    譌･譛ｬ隱槭さ繝｡繝ｳ繝医ｒ蜷ｫ繧�繧ｳ繝ｼ繝峨〒繧ょｴｩ繧後↑縺・ｈ縺・↓菫ｮ豁｣縲・
- doctest 繝｡繧ｿ陦ｨ遉ｺ繧呈隼蝟・�・
  - `neplg2:test[...]` 繧偵ヰ繝・ず蛹悶�・
  - `stdin` / `stdout` 繧偵ヰ繝・ず + `pre` 陦ｨ遉ｺ縺ｸ螟画峩縲・
  - `ret` 繧偵ヰ繝・ず + inline code 陦ｨ遉ｺ縺ｸ螟画峩縲・
  - `"...\\n"` 縺ｪ縺ｩ縺ｮ繧ｨ繧ｹ繧ｱ繝ｼ繝励・繝・さ繝ｼ繝峨＠縺ｦ蜿ｯ隱ｭ陦ｨ遉ｺ縲・
- 繝√Η繝ｼ繝医Μ繧｢繝ｫ蜀・ｮｹ繧呈僑蜈・�・
  - 遶ｶ繝励Ο繝代・繝茨ｼ・2縲・4・峨ｒ霑ｽ蜉�縲・
  - `10_project_fizzbuzz.n.md` 繧・`stdout` 縺ｧ邨先棡縺瑚ｪｭ繧√ｋ萓九∈螟画峩縲・

## 讀懆ｨｼ
- `node nodesrc/tests.js -i tutorials/getting_started -o /tmp/getting_started_doctest.json -j 1`
  - `total: 152, passed: 152, failed: 0, errored: 0`
- `node nodesrc/cli.js -i tutorials/getting_started -o html_play=dist/tutorials/getting_started`
  - `00`縲彖24` 縺ｮ HTML 繧貞・逕滓・縲・

# 2026-02-10 菴懈･ｭ繝｡繝｢ (kp: kpread+kpwrite 逶ｸ莠剃ｽ懃畑縺ｮ譬ｹ譛ｬ菫ｮ豁｣)
## 逞・憾
- `kpread` 縺ｨ `kpwrite` 繧貞酔譎ゅ↓ import 縺励◆繧ｱ繝ｼ繧ｹ縺ｧ縲《tdout 縺ｫ `\0` 縺悟､ｧ驥乗ｷｷ蜈･縺励�～13\n` 縺ｪ縺ｩ縺・`13\0...` 縺ｫ螢翫ｌ縺ｦ縺・◆縲・
- `kpwrite` 蜊倅ｽ薙ユ繧ｹ繝医・騾壹ｋ縺溘ａ縲∝・蜉帛腰菴薙〒縺ｯ縺ｪ縺・import/蜷榊燕隗｣豎ｺ邨瑚ｷｯ縺ｮ逶ｸ莠剃ｽ懃畑縺悟次蝗�縺�縺｣縺溘�・

## 譬ｹ蝗�
- `stdlib/kp/kpread.nepl` 縺御ｸ崎ｦ√↑ `#import "alloc/string" as *` 繧呈戟縺｣縺ｦ縺翫ｊ縲～len` 縺ｪ縺ｩ縺ｮ隴伜挨蟄先ｱ壽沒繧貞ｼ輔″襍ｷ縺薙＠縺ｦ縺・◆縲・
- 蜷梧凾 import 譎ゅ↓ `kpwrite` 蛛ｴ縺ｮ `len` 繝ｭ繝ｼ繧ｫ繝ｫ譚溽ｸ帙→陦晉ｪ√＠縲・聞縺戊ｨ育ｮ・譖ｸ縺崎ｾｼ縺ｿ髟ｷ縺悟｣翫ｌ縺ｦ縺・◆縲・

## 螳溯｣・
- `stdlib/kp/kpread.nepl`
  - 荳崎ｦ√↑ `#import "alloc/string" as *` 繧貞炎髯､縲・
- `stdlib/kp/kpwrite.nepl`
  - `len` 螻�謇�螟画焚繧・`write_len` 縺ｫ謾ｹ蜷搾ｼ・writer_flush` / `writer_ensure` / `writer_put_u8` / `writer_write_str`・峨�・
  - 蜷榊燕陦晉ｪ∵凾縺ｮ蜀咲匱閠先�ｧ繧貞ｼｷ蛹悶�・
- `nepl-core/tests/kp.rs`
  - `kpwrite` 蜊倅ｽ灘・繧雁・縺代ユ繧ｹ繝医ｒ霑ｽ蜉�縲・
  - `kpread_buffer_bytes_debug` 繧・scanner 12B 繝倥ャ繝�莉墓ｧ倥↓蜷医ｏ縺帙※譖ｴ譁ｰ縲・

## 讀懆ｨｼ
- `cargo test --test kp -- --nocapture`
  - `12 passed, 0 failed`
- `NO_COLOR=true trunk build`
  - 謌仙粥
- `node nodesrc/tests.js -i tests/kp.n.md -o tests/output/kp_current.json -j 1`
  - `total=116, passed=116, failed=0, errored=0`

# 2026-02-10 菴懈･ｭ繝｡繝｢ (cast/kp 譛�邨りｪｿ謨ｴ)
## 螳溯｣・
- `stdlib/alloc/string.nepl`
  - `fn cast from_i32;` / `fn cast to_i32;` 繧貞炎髯､縲・
  - `cast` 蜷阪・驕主臆縺ｪ蜈ｬ髢九ｒ貂帙ｉ縺励�～core/cast` 蛛ｴ縺ｮ繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝芽ｧ｣豎ｺ繧貞ｮ牙ｮ壼喧縲・
- `stdlib/core/cast.nepl`
  - 譁・ｭ怜・螟画鋤騾｣謳ｺ繧・`string::from_*` / `string::to_*` 縺ｫ邨ｱ荳�縺励◆迥ｶ諷九ｒ邯ｭ謖√�・
  - `alloc/string` 縺ｮ蜈ｬ髢・`cast` 萓晏ｭ倥ｒ謖√◆縺ｪ縺・ｧ矩��縺ｸ謨ｴ逅・�・

## 讀懆ｨｼ
- `NO_COLOR=true trunk build`
  - 謌仙粥
- `node nodesrc/tests.js -i tests/numerics.n.md -o tests/output/numerics_current.json -j 1`
  - `total=122, passed=122, failed=0, errored=0`
- `node nodesrc/tests.js -i tests/kp.n.md -o tests/output/kp_current.json -j 1`
  - `total=117, passed=117, failed=0, errored=0`
- `cargo test --test kp -q`
  - `14 passed, 0 failed`
- `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 1`
  - `total=465, passed=458, failed=7, errored=0`
  - 莉雁屓隗｣豸・ `tests/numerics.n.md::doctest#3`・・mbiguous overload・・
  - 譌｢蟄俶ｮ倶ｻｶ: `ret_f64_example`, `selfhost_req` 邉ｻ, `sort` 荳�驛ｨ, `string` 荳�驛ｨ

# 2026-02-21 菴懈･ｭ繝｡繝｢ (shadowing 繝・せ繝育ｶｲ鄒・喧)
## 螳溯｣・
- `tests/shadowing.n.md` 繧呈眠隕丈ｽ懈・繝ｻ諡｡蠑ｵ縲・
  - 繝ｭ繝ｼ繧ｫ繝ｫ蛟､縺・import 蜷阪ｒ shadow 縺吶ｋ繧ｱ繝ｼ繧ｹ
  - 繝阪せ繝医ヶ繝ｭ繝・け縺ｮ譛�蜀・━蜈・
  - 繝ｭ繝ｼ繧ｫ繝ｫ髢｢謨ｰ縺・import 髢｢謨ｰ繧・shadow
  - outer/inner 髢｢謨ｰ shadow
  - 蠑墓焚蜷阪→繝ｭ繝ｼ繧ｫ繝ｫ let 縺ｮ shadow
  - while/match/branch 繧貞性繧�繧ｹ繧ｳ繝ｼ繝励こ繝ｼ繧ｹ
  - 迴ｾ迥ｶ譛ｪ蟇ｾ蠢懊・縲悟�､蜷阪→ callable 蜷阪・蜈ｱ蟄倥�咲ｭ峨・ `compile_fail` 縺ｨ縺励※蝗ｺ螳・
- `todo.md` 繧呈峩譁ｰ縲・
  - 繧ｷ繝｣繝峨・荳榊庄菫ｮ鬟ｾ蟄舌・ immutable 縺ｮ `let`/`fn` 縺ｮ縺ｿ縺ｫ驕ｩ逕ｨ
  - `let mut` 縺ｯ蟇ｾ雎｡螟・
  - 驥崎ｦ・stdlib 險伜捷 shadow 譎ゅ・ warn/info 縺ｨ LSP API 蜿門ｾ励ち繧ｹ繧ｯ繧呈・險・

## 讀懆ｨｼ
- `node nodesrc/tests.js -i tests/shadowing.n.md -o tests/output/shadowing_current.json -j 1`
  - `total=176, passed=176, failed=0, errored=0`

# 2026-02-21 菴懈･ｭ繝｡繝｢ (蜷榊燕隗｣豎ｺ API: shadowing 諠・�ｱ縺ｮ諡｡蠑ｵ)
## 螳溯｣・
- `nepl-web/src/lib.rs`
  - `NameResolutionTrace` 縺ｫ `shadows` 繧定ｿｽ蜉�縺励�∝錐蜑崎ｧ｣豎ｺ譎ゅ・ shadowing 繧､繝吶Φ繝医ｒ蜿朱寔縺ｧ縺阪ｋ繧医≧縺ｫ縺励◆縲・
  - 螳夂ｾｩ譎・
    - 譌｢蟄伜�呵｣懊′縺ゅｋ蝣ｴ蜷医↓ `definition_shadow` 繧定ｨ倬鹸縲・
    - 驥崎ｦ√す繝ｳ繝懊Ν・・print`/`println`/`add` 縺ｪ縺ｩ・峨ｒ螟画焚螳夂ｾｩ邉ｻ (`let_hoisted`/`let_mut`/`param`/`match_bind`) 縺ｧ螳夂ｾｩ縺励◆蝣ｴ蜷医・ `warning` 繧剃ｻ倅ｸ弱�・
  - 蜿ら・譎・
    - 蛟呵｣懊′隍・焚縺ゅｋ蝣ｴ蜷医↓ `reference_shadow` 繧定ｨ倬鹸縺励�√�梧治逕ｨ縺輔ｌ縺溷ｮ夂ｾｩ縲阪→縲碁國繧後◆蛟呵｣懊�阪ｒ API 縺九ｉ蜿門ｾ怜庄閭ｽ縺ｫ縺励◆縲・
  - `analyze_name_resolution` 縺ｮ霑泌唆 JSON 縺ｫ莉･荳九ｒ霑ｽ蜉�:
    - `shadows`
    - `shadow_diagnostics`
- `tests/tree/03_name_resolution_tree.js`
  - `result.shadows` / `result.shadow_diagnostics` 繧呈､懆ｨｼ縺吶ｋ繧｢繧ｵ繝ｼ繧ｷ繝ｧ繝ｳ繧定ｿｽ蜉�縲・
  - `x` 縺ｮ shadow 縺ｨ `add` 縺ｮ驥崎ｦ√す繝ｳ繝懊Ν warning 繧貞屓蟶ｰ蝗ｺ螳壹�・

## 讀懆ｨｼ
- `NO_COLOR=false trunk build`
  - 謌仙粥
- `node tests/tree/run.js`
  - `total=4, passed=4, failed=0, errored=0`
- `node nodesrc/tests.js -i tests -o tests/output/tests_current.json`
  - `total=534, passed=527, failed=7, errored=0`
  - 螟ｱ謨励・譌｢遏･繧ｫ繝・ざ繝ｪ・・ret_f64_example`, `selfhost_req`, `sort`, `string compile_fail譛溷ｾ・ｷｮ蛻・・峨〒縲∽ｻ雁屓縺ｮ shadowing API 螟画峩縺ｫ繧医ｋ譁ｰ隕丞､ｱ謨励・遒ｺ隱阪＆繧後↑縺九▲縺溘�・

# 2026-02-21 菴懈･ｭ繝｡繝｢ (typecheck: shadowing warning 莨晄眺縺ｨ髱櫁・蜻ｽ蛹・
## 螳溯｣・
- `nepl-core/src/typecheck.rs`
  - `Binding` 縺ｫ `span` 繧定ｿｽ蜉�縺励�《hadow 隴ｦ蜻翫・莠梧ｬ｡繝ｩ繝吶Ν・亥・螳夂ｾｩ菴咲ｽｮ・峨ｒ蜃ｺ縺帙ｋ繧医≧縺ｫ縺励◆縲・
  - `Env::lookup_outer_defined` 繧定ｿｽ蜉�縺励�∫樟蝨ｨ繧ｹ繧ｳ繝ｼ繝怜､悶・螳夂ｾｩ蛟呵｣懊ｒ蜿ら・縺ｧ縺阪ｋ繧医≧縺ｫ縺励◆縲・
  - `emit_shadow_warning` 繧定ｿｽ蜉�縺励�∵據邵帛ｰ主・譎ゑｼ・let` / `let mut` / `fn` / parameter / match bind・峨↓ shadow 繧呈､懃衍縺励※ warning 繧堤函謌舌☆繧九ｈ縺・↓縺励◆縲・
  - 驥崎ｦ√す繝ｳ繝懊Ν・・print`, `println`, `add` 縺ｪ縺ｩ・峨↓縺､縺・※縺ｯ縲∝､門・蛟呵｣懊′隕九▽縺九ｉ縺ｪ縺・�ｴ蜷医〒繧ゅ�茎tdlib 險伜捷繧帝國縺励≧繧九�購arning 繧堤函謌舌☆繧九ｈ縺・↓縺励◆縲・
  - warning 繝弱う繧ｺ謚大宛縺ｮ縺溘ａ縲・撼驥崎ｦ√す繝ｳ繝懊Ν・井ｾ・ `ok`, `len`・峨・ shadow 縺ｧ縺ｯ compiler warning 繧貞・縺輔↑縺・婿驥昴↓隱ｿ謨ｴ縺励◆縲・
  - `check_function` 縺ｮ霑泌唆繧・`CheckedFunction` 蛹悶＠縲『arning 繧定ｿ斐＠縺､縺､繧ｳ繝ｳ繝代う繝ｫ蟇ｾ雎｡髢｢謨ｰ縺ｯ逕滓・縺礼ｶ壹￠繧九ｈ縺・↓菫ｮ豁｣縺励◆縲・
    - 莉･蜑阪・ warning 繧貞性繧�縺�縺代〒 `Err` 謇ｱ縺・↓縺ｪ繧翫�・未謨ｰ縺瑚誠縺｡縺ｦ縺・◆縲・
    - 迴ｾ蝨ｨ縺ｯ `Error` 縺ｮ縺ｿ `Err`縲『arning 縺ｯ `diagnostics` 縺ｨ縺励※荳贋ｽ阪∈莨晄眺縺吶ｋ縲・
- `tests/tree/04_semantics_tree.js`
  - `analyze_semantics` 縺ｧ shadowing warning 縺悟叙蠕励〒縺阪ｋ縺薙→繧呈､懆ｨｼ縺吶ｋ繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・

## 讀懆ｨｼ
- `NO_COLOR=false trunk build`
  - 謌仙粥
- `node tests/tree/run.js`
  - `total=4, passed=4, failed=0, errored=0`
- `node nodesrc/tests.js -i tests/if.n.md -i tests/offside_and_indent_errors.n.md -i tests/tuple_new_syntax.n.md -i tests/tuple_old_syntax.n.md -i tests/block_single_line.n.md -i tests/pipe_operator.n.md -i tests/keywords_reserved.n.md -o tests/output/upstream_lexer_parser_latest.json`
  - `total=292, passed=292, failed=0, errored=0`
- `node nodesrc/tests.js -i tests -o tests/output/tests_current.json`
  - `total=534, passed=527, failed=7, errored=0`
  - 螟ｱ謨励・譌｢遏･繧ｫ繝・ざ繝ｪ縺ｫ逡吶∪繧翫�∽ｻ雁屓螟画峩縺ｫ繧医ｋ霑ｽ蜉�螟ｱ謨励・遒ｺ隱阪＆繧後↑縺九▲縺溘�・

## 谿玖ｪｲ鬘鯉ｼ井ｻ雁屓縺ｮ螳溯｣・〒隕九∴縺溘ｂ縺ｮ・・
- 驥崎ｦ√す繝ｳ繝懊Ν warning 縺ｯ迴ｾ蝨ｨ繝弱う繧ｺ縺悟､壹￥縲～todo.md` 縺ｫ辟｡蜉ｹ蛹・謚大宛繝昴Μ繧ｷ繝ｼ險ｭ險医ち繧ｹ繧ｯ縺ｨ縺励※谿九＠縺溘�・


# 2026-02-19 菴懈･ｭ繝｡繝｢ (stdlib 繝峨く繝･繝｡繝ｳ繝域紛蛯吶→螻･豁ｴ謨ｴ逅・
## 螳溯｣・
- `stdlib/std/stdio.nepl`, `stdlib/std/fs.nepl`, `stdlib/std/env/cliarg.nepl`, `stdlib/std/test.nepl`:
  - 蜈磯�ｭ繝・Φ繝励Ξ繝ｼ繝郁ｪｬ譏弱ｒ蜑企勁縺励�～//:` 蠖｢蠑上・繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝医〒邨ｱ荳�縲・
  - 豕ｨ諢乗枚繧偵�悟憶菴懃畑繝ｻ繝｡繝｢繝ｪ遒ｺ菫・遘ｻ蜍輔・繧ｿ繝ｼ繧ｲ繝・ヨ蛻ｶ邏・�阪↑縺ｩ螳溷茜逕ｨ譎ゅ・豕ｨ諢上∈譏ｯ豁｣縲・
  - 蜷・未謨ｰ縺ｫ蛻ｩ逕ｨ萓具ｼ・neplg2:test[skip]`・峨ｒ邯ｭ謖√＠縲∝他縺ｳ蜃ｺ縺怜ｽ｢繧堤｢ｺ隱阪＠繧・☆縺・ｧ区・縺ｸ謨ｴ逅・�・
- `stdlib` 蜈ｨ菴薙・繝峨く繝･繝｡繝ｳ繝域枚險�繧堤せ讀懊＠縲√Δ繝・け逧・↑陦ｨ迴ｾ繧剃ｻ･荳九・譁ｹ驥昴〒譏ｯ豁｣縲・
  - 縲碁未謨ｰ縺ｮ讎りｦ√�坂・縲御ｸｻ縺ｪ逕ｨ騾斐�・
  - 縲瑚ｩｳ邏ｰ縺ｪ髢｢謨ｰ蛻･繝峨く繝･繝｡繝ｳ繝医・谿ｵ髫守噪縺ｫ霑ｽ險倥＠縺ｾ縺吶�ゅ�阪・蜑企勁
  - 螳溯｣・ｪｬ譏・豕ｨ諢乗枚縺ｮ繝・Φ繝励Ξ繝ｼ繝域枚險�繧偵�∝茜逕ｨ譎ゅ・謖吝虚縺御ｼ昴ｏ繧玖｡ｨ迴ｾ縺ｸ鄂ｮ謠・
- commit 螻･豁ｴ縺ｯ `4772eea` 蝓ｺ轤ｹ縺ｧ蟾ｮ蛻・ｒ蜀埼←逕ｨ縺励�∽ｻ雁屓蛻・ｒ蜊倅ｸ� commit 縺ｫ蜀堺ｽ懈・縲・

## plan.md縺ｨ縺ｮ蟾ｮ逡ｰ
- 莉雁屓縺ｯ plan.md 縺ｮ險�隱樊ｩ溯・霑ｽ蜉�縺ｧ縺ｯ縺ｪ縺上�《tdlib 縺ｮ繝峨く繝･繝｡繝ｳ繝亥刀雉ｪ謾ｹ蝟・→螻･豁ｴ謨ｴ逅・ｒ螳滓命縲・
- 繝ｩ繝ｳ繧ｿ繧､繝�謖吝虚繧・API 繧ｷ繧ｰ繝阪メ繝｣縺ｯ螟画峩縺励※縺・↑縺・�・

## 讀懆ｨｼ
- `cargo install trunk`
  - 螟ｱ謨暦ｼ・https://index.crates.io/config.json` 蜿門ｾ玲凾縺ｫ 403縲√ロ繝・ヨ繝ｯ繝ｼ繧ｯ蛻ｶ邏・〒蟆主・荳榊庄・峨�・
- `NO_COLOR=true trunk build`
  - 螟ｱ謨暦ｼ・trunk` 譛ｪ蟆主・・峨�・
- `node nodesrc/tests.js -i stdlib/std -o tests/output/stdlib_std_docs_current.json -j 1`
  - 螟ｱ謨暦ｼ・ompiler artifacts 荳榊惠縲～total=215, errored=215`・峨�・
- `node nodesrc/cli.js -i stdlib/std -o html_play=dist/stdlib_std`
  - 螟ｱ謨暦ｼ・rtifacts 荳榊惠縺ｧ HTML 逕滓・荳榊庄・峨�・

# 2026-02-21 菴懈･ｭ繝｡繝｢ (lexer/parser 荳頑ｵ∵紛逅・+ 譛ｨ讒矩�� API 繝・せ繝郁ｿｽ蜉�)
## 螳溯｣・
- `nepl-core/src/lexer.rs`
  - `cond` / `then` / `else` / `do` 繧貞ｰら畑繧ｭ繝ｼ繝ｯ繝ｼ繝峨ヨ繝ｼ繧ｯ繝ｳ (`KwCond`, `KwThen`, `KwElse`, `KwDo`) 縺ｨ縺励※霑ｽ蜉�縲・
  - 繧ｭ繝ｼ繝ｯ繝ｼ繝牙愛螳壹ｒ `keyword_token` 縺ｫ髮・ｴ・＠縲∝酔鄒ｩ蛻・ｲ舌・驥崎､・ｒ隗｣豸医�・
  - `LexState` 縺ｮ譛ｪ菴ｿ逕ｨ lifetime 繧帝勁蜴ｻ縺励�∝ｭ怜唱隗｣譫千憾諷九・螳夂ｾｩ繧堤ｰ｡貎泌喧縲・
- `nepl-core/src/parser.rs`
  - 譁ｰ繧ｭ繝ｼ繝ｯ繝ｼ繝峨ヨ繝ｼ繧ｯ繝ｳ繧偵Ξ繧､繧｢繧ｦ繝医・繝ｼ繧ｫ繝ｼ縺ｨ縺励※蜿礼炊縺吶ｋ蛻・ｲ舌ｒ霑ｽ蜉�縲・
  - 諡ｬ蠑ｧ蠑・(`(` ... `)`) 縺ｮ隗｣譫舌Ο繧ｸ繝・け繧・`parse_parenthesized_expr_items` 縺ｫ邨ｱ蜷医＠縲・邂・園驥崎､・＠縺ｦ縺・◆蜃ｦ逅・ｒ荳�譛ｬ蛹悶�・
  - 險ｺ譁ｭ譁・ｒ迴ｾ莉墓ｧ倥↓蜷医ｏ縺帙※譖ｴ譁ｰ:
    - `tuple literal cannot end with a comma` -> `trailing comma is not allowed in parenthesized expression`
    - `expected ')' after tuple literal` -> `expected ')' after parenthesized expression`
- `nepl-web/src/lib.rs`
  - 隗｣譫・API 縺ｮ token kind 譁・ｭ怜・陦ｨ迴ｾ縺ｫ `KwCond/KwThen/KwElse/KwDo` 繧定ｿｽ蜉�縲・
- 繝・せ繝郁ｿｽ蜉�
  - `tests/keywords_reserved.n.md` 繧呈眠隕剰ｿｽ蜉�縺励�～cond/then/else/do` 縺瑚ｭ伜挨蟄舌→縺励※菴ｿ縺医↑縺・％縺ｨ繧・`compile_fail` 縺ｧ蝗ｺ螳壹�・
  - `tests/tree/*.js` 繧呈眠隕剰ｿｽ蜉�縺励�´SP/繝・ヰ繝・げ蜷代￠ API 縺ｮ譛ｨ讒矩��繧呈ｮｵ髫主挨縺ｫ讀懆ｨｼ:
    - `tests/tree/01_lex_tree.js`
    - `tests/tree/02_parse_tree.js`
    - `tests/tree/03_name_resolution_tree.js`
    - `tests/tree/04_semantics_tree.js`
    - `tests/tree/run.js`・井ｸ�諡ｬ螳溯｡鯉ｼ・

## 讀懆ｨｼ
- `NO_COLOR=false trunk build`
  - 謌仙粥
- `node nodesrc/tests.js -i tests/if.n.md -i tests/offside_and_indent_errors.n.md -i tests/tuple_new_syntax.n.md -i tests/tuple_old_syntax.n.md -i tests/block_single_line.n.md -i tests/pipe_operator.n.md -i tests/keywords_reserved.n.md -o tests/output/upstream_lexer_parser_final.json`
  - `total=292, passed=292, failed=0, errored=0`
- `node tests/tree/run.js`
  - `total=4, passed=4, failed=0, errored=0`

## 陬懆ｶｳ
- `tests` 蜈ｨ菴・(`--no-stdlib`) 螳溯｡後〒縺ｯ譌｢蟄倥・荳区ｵ∬ｪｲ鬘鯉ｼ・et_f64/selfhost/sort 縺ｪ縺ｩ・峨〒螟ｱ謨励′谿九ｋ縺後�∽ｻ雁屓縺ｮ lexer/parser 螟画峩縺ｧ譁ｰ隕丞屓蟶ｰ縺ｯ遒ｺ隱阪＆繧後※縺・↑縺・�・

# 2026-02-21 菴懈･ｭ繝｡繝｢ (noshadow 蟆主・螳御ｺ・→蝗槫ｸｰ菫ｮ豁｣)
- `noshadow` 繧・lexer/parser/typecheck/web API 縺ｾ縺ｧ荳�雋ｫ縺励※螳溯｣・�・
  - lexer: `KwNoShadow` 繧定ｿｽ蜉�縲・
  - parser: `let` 菫ｮ鬟ｾ蟄舌↓ `noshadow` 繧定ｿｽ蜉�縲Ａlet mut noshadow` 縺ｯ parse error縲・
  - parser: `fn noshadow <name>` 繧貞女逅・＠縲、ST 縺ｫ `no_shadow` 繧剃ｿ晄戟縲・
  - typecheck: `Binding.no_shadow` 繧貞ｰ主・縺励�～noshadow` 螳｣險�縺ｮ荳頑嶌縺阪ｒ compile error 蛹悶�・
- 蜷榊燕隗｣豎ｺ/蝙区､懈渊縺ｮ譌｢蟄伜虚菴懊ｒ螢翫＆縺ｪ縺・◆繧√�∝酔荳�繧ｹ繧ｳ繝ｼ繝励・騾壼ｸｸ `let` 蜀肴據邵幢ｼ・let lst ...; let lst ...;`・峨・邯ｭ謖√�・
  - 縺溘□縺玲里蟄俶據邵帙′ `no_shadow` 縺ｮ蝣ｴ蜷医・縺ｿ縲∝酔蜷榊ｮ｣險�繧呈拠蜷ｦ縺吶ｋ縲・
- Web 蛛ｴ縺ｮ繝医・繧ｯ繝ｳ API 繧・`KwNoShadow` 縺ｫ霑ｽ蠕薙�・
- 繝・せ繝郁ｿｽ蜉�:
  - `tests/shadowing.n.md` 縺ｫ `noshadow` 縺ｮ compile_fail 繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
- 讀懆ｨｼ邨先棡:
  - `NO_COLOR=false trunk build` 謌仙粥
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` 縺ｧ `547/547 passed`

# 2026-02-21 菴懈･ｭ繝｡繝｢ (doctest 縺ｮ profile 繧ｲ繝ｼ繝亥ｮ牙ｮ壼喧)
- `#if[profile=debug/release]` 縺ｮ doctest 縺・CI 迺ｰ蠅・・繝薙Ν繝峨Δ繝ｼ繝牙ｷｮ蛻・〒謠ｺ繧後ｋ蝠城｡後↓蟇ｾ縺励※縲√ユ繧ｹ繝医Λ繝ｳ繝翫・縺九ｉ繧ｳ繝ｳ繝代う繝ｫ繝励Ο繝輔ぃ繧､繝ｫ繧呈・遉ｺ謖・ｮ壹〒縺阪ｋ繧医≧縺ｫ菫ｮ豁｣縲・
- `nepl-web` 蛛ｴ:
  - `compile_source_with_profile(source, profile)` 繧定ｿｽ蜉�縲・
  - `compile_source_with_vfs_and_profile(entry_path, source, vfs, profile)` 繧定ｿｽ蜉�縲・
  - 蜀・Κ繧ｳ繝ｳ繝代う繝ｫ邨瑚ｷｯ繧・`compile_wasm_with_entry_and_profile(..., Option<BuildProfile>)` 縺ｫ邨ｱ蜷医�・
- `nodesrc/run_test.js` 蛛ｴ:
  - 蜿ｯ閭ｽ縺ｪ蝣ｴ蜷医・蟶ｸ縺ｫ `debug` 繧呈・遉ｺ謖・ｮ壹＠縺ｦ繧ｳ繝ｳ繝代う繝ｫ縺吶ｋ繧医≧縺ｫ螟画峩縲・
  - VFS 縺ゅｊ/縺ｪ縺嶺ｸ｡譁ｹ縺ｧ譁ｰ API 繧貞━蜈井ｽｿ逕ｨ縺励�∵立 API 縺ｯ蠕梧婿繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ縺ｨ縺励※菫晄戟縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` 謌仙粥
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` 縺ｧ `547/547 passed`

# 2026-02-21 菴懈･ｭ繝｡繝｢ (stdlib result 縺ｸ縺ｮ谿ｵ髫守噪 noshadow 驕ｩ逕ｨ)
- `stdlib/core/result.nepl` 縺ｮ蝓ｺ逶､ API 縺九ｉ縲∬｡晉ｪ√Μ繧ｹ繧ｯ縺御ｽ弱＞ `unwrap_ok` / `unwrap_err` 縺ｫ `noshadow` 繧剃ｻ倅ｸ弱�・
- 逶ｮ逧・
  - 蝓ｺ逶､ API 縺ｮ隱､荳頑嶌縺阪ｒ譌ｩ譛滓､懷・縺吶ｋ驕狗畑繧呈ｮｵ髫主ｰ主・縺吶ｋ縲・
  - 譌｢蟄倥さ繝ｼ繝峨〒蛻ｩ逕ｨ鬆ｻ蠎ｦ縺碁ｫ倥＞遏ｭ蜷搾ｼ・ok` / `err` / `map`・峨・莉雁屓菫晉蕗縺励�∫�ｴ螢顔ｯ・峇繧呈怙蟆丞喧縲・
- 蝗槫ｸｰ繝・せ繝医ｒ霑ｽ蜉�:
  - `tests/shadowing.n.md` 縺ｫ `std_result_noshadow_unwrap_ok`・・ompile_fail・峨ｒ霑ｽ蜉�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` 謌仙粥
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` 縺ｧ `548/548 passed`

# 2026-02-21 菴懈･ｭ繝｡繝｢ (shadow 縺ｨ overload 縺ｮ謇ｱ縺・紛逅・
- 莉墓ｧ倩ｪｿ謨ｴ:
  - 髢｢謨ｰ縺ｮ蜷悟錐螳夂ｾｩ縺ｧ繧ｷ繧ｰ繝阪メ繝｣縺檎焚縺ｪ繧句�ｴ蜷医・繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨→縺励※險ｱ蜿ｯ縲・
  - 蜷悟錐縺九▽蜷御ｸ�繧ｷ繧ｰ繝阪メ繝｣縺ｮ蝣ｴ蜷医・縺ｿ縲茎hadowing 謇ｱ縺・・ warning縲阪ｒ蜃ｺ縺吶�・
  - 蜷悟錐髢｢謨ｰ蜀榊ｮ夂ｾｩ繧偵お繝ｩ繝ｼ縺ｫ縺ｯ縺励↑縺・�・
- `noshadow` 縺ｮ髢｢謨ｰ驕ｩ逕ｨ繝ｫ繝ｼ繝ｫ繧定ｪｿ謨ｴ:
  - `noshadow fn` 縺ｧ繧る未謨ｰ蜷悟錐・医が繝ｼ繝舌・繝ｭ繝ｼ繝会ｼ峨・險ｱ蜿ｯ縲・
  - 螟画焚/蛟､蜷榊燕遨ｺ髢薙→縺ｮ陦晉ｪ√・蠕捺擂騾壹ｊ諡貞凄縲・
- 蛻ｩ逕ｨ鬆ｻ蠎ｦ縺ｮ鬮倥＞荳�闊ｬ蜷阪↓蟇ｾ縺吶ｋ譁ｹ驥晏､画峩:
  - `unwrap` / `unwrap_ok` / `unwrap_err` 繧・`noshadow` 蟇ｾ雎｡縺九ｉ螟悶＠縺溘�・
  - 縺薙ｌ縺ｫ莨ｴ縺・`tests/shadowing.n.md` 縺ｮ unwrap 邉ｻ compile_fail 繧ｱ繝ｼ繧ｹ繧貞炎髯､縲・
- 繝・せ繝域峩譁ｰ:
  - `fn_noshadow_rejects_shadowing` 繧・`fn_same_signature_shadowing_warns_and_latest_wins` 縺ｫ譖ｴ譁ｰ縺励�∵・蜉溘こ繝ｼ繧ｹ縺ｨ縺励※蝗ｺ螳夲ｼ・ret: 2`・峨�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` 謌仙粥
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` 縺ｧ `547/547 passed`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (todo 譽壼査縺・
- `todo.md` 縺ｮ譽壼査縺励ｒ螳滓命縺励�∬ｧ｣豎ｺ貂医∩縺ｾ縺溘・迥ｶ諷九′蜿､縺・�・岼繧貞炎髯､縺励◆縲・
- 迚ｹ縺ｫ莉･荳九ｒ謨ｴ逅・
  - 蜿､縺・寔險亥�､ (`total=413, passed=404, failed=9`) 繧貞炎髯､縲・
  - 譌｢縺ｫ螳御ｺ・ｸ医∩縺ｮ `nm/parser` 蝙句錐陦晉ｪ√・`examples/nm.nepl` 縺ｮ `cliarg` 邨瑚ｷｯ菫ｮ豁｣邉ｻ繧ｿ繧ｹ繧ｯ繧・todo 縺九ｉ髯､蜴ｻ縲・
  - `todo.md` 縺ｯ譛ｪ螳御ｺ・ち繧ｹ繧ｯ縺ｮ縺ｿ・亥錐蜑咲ｩｺ髢・鬮倬嚴髢｢謨ｰ/LSP/險ｺ譁ｭ菴鍋ｳｻ/Web蠑ｷ蛹・js_interpreter・峨↓蜀肴ｧ区・縲・
- 迴ｾ譎らせ縺ｮ蝗槫ｸｰ遒ｺ隱・
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json` 縺ｮ譛�譁ｰ邨先棡縺ｯ pass 邯ｭ謖・ｼ育峩霑大ｮ溯｡・ `547/547`・峨�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (profile/target 繧ｲ繝ｼ繝医→ stdlib 驥崎､・ｮ夂ｾｩ縺ｮ蝗槫ｸｰ菫ｮ豁｣)
- 逞・憾:
  - doctest 縺ｧ `debug_color` / `debugln_color` / `test_checked` / `test_print_fail` 縺ｮ蜷御ｸ�繧ｷ繧ｰ繝阪メ繝｣蜀榊ｮ夂ｾｩ warning 縺・compile fail 謇ｱ縺・↓縺ｪ縺｣縺ｦ縺・◆縲・
  - `functions.n.md` 縺ｪ縺ｩ縺ｮ螟ｱ謨励→豺ｷ蝨ｨ縺励※縺・◆縺溘ａ縲√∪縺・warning 襍ｷ轤ｹ繧貞・繧雁・縺代◆縲・
- 蜴溷屏:
  - `#if[...]` 縺ｮ逶ｴ蠕後↓ `//:` 繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝医′謖溘∪繧狗ｮ・園縺ｧ縲∵擅莉ｶ莉倥″螳夂ｾｩ縺梧э蝗ｳ縺ｩ縺翫ｊ縺ｫ髯仙ｮ壹＆繧後★驥崎､・ｮ夂ｾｩ縺悟酔譎よ怏蜉ｹ縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
- 菫ｮ豁｣:
  - `stdlib/std/stdio.nepl`:
    - 譚｡莉ｶ莉倥″髢｢謨ｰ螳夂ｾｩ縺ｫ蟇ｾ縺励※ `#if[profile=...]` 繧貞ｮ夂ｾｩ逶ｴ蜑阪∈蜀埼・鄂ｮ縲・
    - release 蛛ｴ縺ｮ蜷悟錐螳溯｣・・蜀・Κ蜷・(`__debug_*_release_noop`) 縺ｫ騾�驕ｿ縺励�√す繧ｰ繝阪メ繝｣陦晉ｪ√ｒ髯､蜴ｻ縲・
  - `stdlib/std/test.nepl`:
    - `#if[target=...]` 繧帝未謨ｰ螳夂ｾｩ逶ｴ蜑阪∈蜀埼・鄂ｮ縺励�∵э蝗ｳ縺励◆繧ｿ繝ｼ繧ｲ繝・ヨ髯仙ｮ壹〒螳夂ｾｩ縺輔ｌ繧九ｈ縺・ｿｮ豁｣縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - 蟇ｾ雎｡蜀咲樟繝・せ繝・
    - `node nodesrc/tests.js -i tests/functions.n.md -i stdlib/core/option.nepl -i stdlib/core/result.nepl ...`
    - `191/191 pass`
  - 蜈ｨ菴・
    - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`
    - `547/547 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (nepl-web API 縺ｨ cli.js 縺ｮ雋ｬ蜍吝・髮｢)
- 隕∽ｻｶ蜿肴丐:
  - `nepl-web/src/lib.rs` 縺ｯ API 謠蝉ｾ帙・縺ｿ縺ｫ髯仙ｮ壹＠縲¨ode/FS 縺ｸ縺ｮ逶ｴ謗･繧｢繧ｯ繧ｻ繧ｹ縺ｯ謖√◆縺ｪ縺・ｧ区・縺ｫ縺励◆縲・
  - FS 縺九ｉ stdlib 繧定ｪｭ繧�雋ｬ蜍吶・ JS 蛛ｴ・・nodesrc/cli.js`・峨↓蛻・屬縲・
- `nepl-web/src/lib.rs` 螟画峩:
  - 譌｢蟄倥・縲後ヰ繝ｳ繝峨Ν stdlib 菴ｿ逕ｨ・医ョ繝輔か繝ｫ繝茨ｼ峨�阪・邯ｭ謖√�・
  - 譁ｰ隕・API:
    - `get_bundled_stdlib_vfs()`: wasm 縺ｫ繝舌Φ繝峨Ν縺輔ｌ縺・stdlib 繧・`/stdlib/...` 蠖｢蠑・VFS 縺ｧ霑斐☆縲・
    - `compile_source_with_vfs_and_stdlib(...)`
    - `compile_source_with_vfs_stdlib_and_profile(...)`
  - 縺薙ｌ縺ｫ繧医ｊ縲∝､夜Κ・・ode/繝悶Λ繧ｦ繧ｶ・峨′ stdlib 繧ｽ繝ｼ繧ｹ驕ｸ謚槭ｒ諡・∴繧九ｈ縺・↓縺ｪ縺｣縺溘�・
- `nodesrc/cli.js` 螟画峩:
  - `loadStdlibVfsFromFs(stdlibRootDir)` 繧定ｿｽ蜉�・医Ο繝ｼ繧ｫ繝ｫ FS 縺九ｉ `/stdlib/...` VFS 繧呈ｧ狗ｯ会ｼ峨�・
  - `loadBundledStdlibVfs(api)` 繧定ｿｽ蜉�・・asm 繝舌Φ繝峨Ν stdlib 蜿門ｾ暦ｼ峨�・
  - `compileWithLocalStdlib(api, ...)` 繧定ｿｽ蜉�・医Ο繝ｼ繧ｫ繝ｫ stdlib 繧剃ｽｿ縺｣縺ｦ繧ｳ繝ｳ繝代う繝ｫ API 繧貞他縺ｶ・峨�・
- 蜻ｼ縺ｳ蜃ｺ縺怜・譖ｴ譁ｰ:
  - `nodesrc/html_gen_playground.js` 縺ｧ譁ｰ API 繧貞━蜈井ｽｿ逕ｨ縺吶ｋ繧医≧譖ｴ譁ｰ縲・
  - `web/src/main.ts` 縺ｧ `get_bundled_stdlib_vfs` 繧貞━蜈医＠縲∵立 `get_stdlib_files` 縺ｯ繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ縺ｫ螟画峩縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (蜷榊燕隗｣豎ｺ蜀崎ｨｭ險・ 髢｢謨ｰ蛟呵｣懈､懃ｴ｢縺ｮ謨ｴ逅・隨ｬ1谿ｵ)
- 逶ｮ逧・
  - `todo.md` 譛�蜆ｪ蜈磯�・岼・・alueNs/CallableNs 蛻・屬・峨↓蜷代￠縺ｦ縲∵嫌蜍輔ｒ螟峨∴縺ｪ縺・ｯ・峇縺ｧ髢｢謨ｰ蛟呵｣懈､懃ｴ｢繝ｭ繧ｸ繝・け繧呈紛逅・�・
- 螳溯｣・
  - `Env` 縺ｫ `lookup_all_callables` 繧定ｿｽ蜉�縲・
  - 髢｢謨ｰ蛟呵｣懈歓蜃ｺ縺ｧ `lookup_all + filter(Func)` 繧堤ｹｰ繧願ｿ斐＠縺ｦ縺・◆邂・園繧・`lookup_all_callables` 縺ｸ鄂ｮ謠帙�・
    - top-level `FnDef` 縺ｮ `f_ty` 豎ｺ螳・
    - nested `FnDef` 縺ｮ `f_ty/captures` 豎ｺ螳・
    - `user_visible_arity` 縺ｮ capture 謨ｰ險育ｮ・
  - `find_same_signature_func` 繧・`lookup_all_callables` 繝吶・繧ｹ縺ｸ螟画峩縲・
- 邨先棡:
  - 讖溯・螟画峩縺ｪ縺励〒驥崎､・Ο繧ｸ繝・け繧貞炎貂帙＠縲∵ｬ｡谿ｵ縺ｮ蜷榊燕遨ｺ髢灘・髮｢・・alue/Callable・峨↓騾ｲ繧√ｋ蝓ｺ逶､繧剃ｽ懈・縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (蜷榊燕隗｣豎ｺ蜀崎ｨｭ險・ Value/Callable API 譏守｢ｺ蛹・隨ｬ2谿ｵ)
- 逶ｮ逧・
  - ValueNs/CallableNs 蛻・屬縺ｸ蜷代￠縺ｦ縲～Env` 縺ｮ讀懃ｴ｢ API 繧呈・遒ｺ蛹悶＠縲・未謨ｰ蜻ｼ縺ｳ蜃ｺ縺礼ｵ瑚ｷｯ縺ｮ蛻・ｲ舌ｒ隱ｭ縺ｿ繧・☆縺上☆繧九�・
- 螳溯｣・
  - `Env` 縺ｫ莉･荳九ｒ霑ｽ蜉�:
    - `lookup_value(name)`
    - `lookup_callable(name)`
  - 譌｢蟄・`lookup_all` 縺ｯ縲梧怙蜀・せ繧ｳ繝ｼ繝怜━蜈医�阪・縺ｾ縺ｾ邯ｭ謖√＠縲～lookup_value/lookup_callable` 縺ｯ縺昴・邨先棡縺九ｉ kind 繧帝∈縺ｶ險ｭ險医↓縺励◆・郁ｧ｣豎ｺ隕丞援縺ｯ邯ｭ謖・ｼ峨�・
  - `find_same_signature_func` 縺ｯ callable 蟆ら畑讀懃ｴ｢繧剃ｽｿ縺・ｈ縺・紛逅・�・
  - `check_call_or_letset` 邉ｻ縺ｮ蛻・ｲ舌〒縲～lookup_all + var 蛻､螳啻 繧・`lookup_all_callables` / `lookup_value` 縺ｫ鄂ｮ謠帙�・
- 邨先棡:
  - 謖吝虚繧貞､峨∴縺壹↓ Value/Callable 縺ｮ雋ｬ蜍吶ｒ繧ｳ繝ｼ繝我ｸ翫〒蛻・屬縺ｧ縺阪ｋ蠖｢縺ｸ蜑埼�ｲ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (nm-compile 螟ｱ謨励・譬ｹ蝗�菫ｮ豁｣: extern/entry 蜿朱寔邨瑚ｷｯ縺ｮ邨ｱ蜷・
- 閭梧勹:
  - CI (`nm-compile`) 縺ｧ `stdlib/std/env/cliarg.nepl` 縺ｮ `args_sizes_get` / `args_get` 縺・`undefined identifier` 縺ｫ縺ｪ繧句､ｱ謨励ｒ遒ｺ隱阪�・
  - 蜷梧凾縺ｫ `expression left extra values on the stack` 縺碁�｣骼悶＠縺ｦ逋ｺ逕溘�・
- 譬ｹ蝗�:
  - `typecheck` 縺ｮ蜈郁｡後ョ繧｣繝ｬ繧ｯ繝・ぅ繝門・逅・′ `module.root.items` 縺ｮ `Stmt::Directive` 縺ｮ縺ｿ繧定ｵｰ譟ｻ縺励※縺翫ｊ縲・
    繝ｭ繝ｼ繝�繝ｼ邨檎罰縺ｧ `module.directives` 蛛ｴ縺ｫ菫晄戟縺輔ｌ縺・`#extern` 繧貞叙繧翫％縺ｼ縺咏ｵ瑚ｷｯ縺後≠縺｣縺溘�・
- 菫ｮ豁｣:
  - `nepl-core/src/typecheck.rs` 縺ｧ繝・ぅ繝ｬ繧ｯ繝・ぅ繝夜←逕ｨ蜃ｦ逅・ｒ蜈ｱ騾壼喧縲・
  - `module.directives` 縺ｨ `module.root.items` 縺ｮ蜿梧婿繧帝←逕ｨ蟇ｾ雎｡縺ｫ縺励�《pan 繧ｭ繝ｼ縺ｧ驥崎､・←逕ｨ繧呈椛豁｢縲・
  - 縺薙ｌ縺ｫ繧医ｊ `#extern wasi_snapshot_preview1 args_sizes_get/args_get` 縺悟ｮ牙ｮ壹＠縺ｦ迺ｰ蠅・∈逋ｻ骭ｲ縺輔ｌ繧九ｈ縺・↓縺励◆縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/neplg2.n.md -o tests/output/neplg2_current.json -j 2`: `200/200 pass`
  - `cargo run -p nepl-cli -- --target wasi --profile debug --input examples/nm.nepl --output /tmp/ci-nm`: `compile_module returned Ok`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`
- 菴咲ｽｮ縺･縺・
  - 莉墓ｧ伜､画峩・・target=wasm` 縺ｧ WASI 辟｡蜉ｹ・牙ｾ後・蝗槫ｸｰ縺ｧ縺ゅｊ縲∽ｸ頑ｵ・ｼ・ypecheck 蜈･繧雁哨・峨〒譬ｹ譛ｬ菫ｮ豁｣縲・
  - 谺｡谿ｵ縺ｯ蝗ｺ螳壽婿驥昴←縺翫ｊ lexer/parser 縺ｮ譌ｧ莉墓ｧ俶ｮ矩ｪｸ謨ｴ逅・ｒ蜆ｪ蜈医☆繧九�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (譚｡莉ｶ莉倥″繝・ぅ繝ｬ繧ｯ繝・ぅ繝冶ｩ穂ｾ｡縺ｮ鬆・ｺ丈ｿｮ豁｣)
- 閭梧勹:
  - `typecheck` 縺ｮ extern/entry 蜿朱寔繧・`module.directives` 縺ｸ諡｡蠑ｵ縺励◆髫帙�・
    `module.directives` 蛛ｴ縺ｫ蟇ｾ縺励※ `#if[target=...]` / `#if[profile=...]` 縺ｮ隧穂ｾ｡繧帝�壹＠縺ｦ縺・↑縺・ｵ瑚ｷｯ縺梧ｮ九▲縺ｦ縺・◆縲・
- 菫ｮ豁｣:
  - `module.directives` 襍ｰ譟ｻ縺ｧ繧・`pending_if` 繧剃ｽｿ縺｣縺ｦ gate 隧穂ｾ｡繧帝←逕ｨ縲・
  - 譌｢蟄倥・ `module.root.items` 襍ｰ譟ｻ縺ｨ蜷後§譚｡莉ｶ莉倥″譛牙柑蛹悶Ν繝ｼ繝ｫ縺ｫ邨ｱ荳�縲・
  - span 繧ｭ繝ｼ驥崎､・勁螟悶・邯ｭ謖√＠縲∽ｺ碁㍾逋ｻ骭ｲ縺ｯ髦ｲ豁｢縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/neplg2.n.md -i tests/nm.n.md -o tests/output/upstream_lexer_parser_latest.json -j 3`: `220/220 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`
- 菴咲ｽｮ縺･縺・
  - 荳頑ｵ・ｼ・ypecheck蜈･繧雁哨・峨〒縺ｮ譚｡莉ｶ蛻､螳壻ｸ�雋ｫ蛹悶〒縲］m/cliarg 繧貞性繧� extern 隗｣豎ｺ縺ｮ蜀咲匱髦ｲ豁｢繧堤岼逧・→縺励◆譬ｹ譛ｬ菫ｮ豁｣縲・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (繧ｷ繝｣繝峨・隴ｦ蜻・ 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝臥ｵ瑚ｷｯ縺ｮ繝弱う繧ｺ謚大宛)
- 閭梧勹:
  - 莉墓ｧ倅ｸ翫�・未謨ｰ繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨・險ｱ螳ｹ縺輔ｌ繧九◆繧√�√が繝ｼ繝舌・繝ｭ繝ｼ繝画・遶九こ繝ｼ繧ｹ縺ｧ荳�闊ｬ shadow warning 繧貞・縺吶・縺ｯ繝弱う繧ｺ縺ｫ縺ｪ繧九�・
- 菫ｮ豁｣:
  - `nepl-core/src/typecheck.rs`
    - 繝阪せ繝・`fn` 逋ｻ骭ｲ譎ゅ・ `emit_shadow_warning(...)` 蜻ｼ縺ｳ蜃ｺ縺玲擅莉ｶ繧定ｪｿ謨ｴ縲・
    - 譌｢蟄伜酔蜷榊�呵｣懊′縲後☆縺ｹ縺ｦ callable・・ 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝牙�呵｣懶ｼ峨�阪・蝣ｴ蜷医・荳�闊ｬ shadow warning 繧貞・縺輔↑縺・�・
    - 蜷悟錐縺ｫ value 邉ｻ譚溽ｸ帙′豺ｷ蝨ｨ縺吶ｋ蝣ｴ蜷医・縺ｿ蠕捺擂縺ｩ縺翫ｊ warning 繧貞・縺吶�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests/shadowing.n.md -i tests/overload.n.md -o tests/output/shadowing_current.json -j 2`: `186/186 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`
- 菴咲ｽｮ縺･縺・
  - 蜷榊燕隗｣豎ｺ繝ｻ繧ｷ繝｣繝峨・繧､繝ｳ繧ｰ蜀崎ｨｭ險茨ｼ・odo譛�蜆ｪ蜈磯�・岼・峨・荳�驛ｨ縺ｨ縺励※縲・
    縲後が繝ｼ繝舌・繝ｭ繝ｼ繝峨〒縺ｯ縺ｪ縺丞ｮ溘す繝｣繝峨・縺ｮ縺ｿ隴ｦ蜻翫�阪・驕狗畑縺ｫ霑代▼縺代ｋ隱ｿ謨ｴ縲・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (譌ｧ繧ｿ繝励Ν險俶ｳ輔・谿句ｭ伜・鬘・
- 逶ｮ逧・
  - 蝗ｺ螳壽欠遉ｺ縺ｫ蝓ｺ縺･縺阪�∽ｸ頑ｵ∽ｿｮ豁｣・・arser 蠑ｷ蛹厄ｼ峨・蜑阪↓蜈ｨ菴薙ｒ蛻・｡槭＠縺ｦ螻�謇�菫ｮ豁｣繧貞屓驕ｿ縺吶ｋ縲・
- 螳滓命:
  - `rg` 縺ｧ `stdlib/tests/tutorials` 縺ｮ譌ｧ繧ｿ繝励Ν險俶ｳ募�呵｣懊ｒ譽壼査縺励�・
  - `tests/tree/run.js` 縺ｧ LSP/隗｣譫植PI邉ｻ縺ｮ蝗槫ｸｰ繧堤｢ｺ隱阪�・
- 隕ｳ貂ｬ:
  - `tests/tree/run.js`: `4/4 pass`縲・
  - 譌ｧ tuple literal reject 縺ｯ譌｢蟄倥←縺翫ｊ譛牙柑縺�縺後�》uple type 險俶ｳ・`(<T1,T2>)` 縺ｯ stdlib/tests 縺ｫ蠎・￥谿句ｭ倥�・
  - parser 縺ｧ tuple type 繧貞叉譎・reject 縺吶ｋ縺ｨ stdlib doctest 縺悟､ｧ驥冗�ｴ邯ｻ縺吶ｋ縺薙→繧堤｢ｺ隱搾ｼ域ｮｵ髫守ｧｻ陦後′蠢・ｦ・ｼ峨�・
- 譁ｹ驥晄峩譁ｰ:
  - `todo.md` 縺ｫ縲梧立繧ｿ繝励Ν險俶ｳ輔・螳悟・遘ｻ陦鯉ｼ域ｮｵ髫主ｮ滓命・峨�阪ｒ霑ｽ蜉�縲・
  - 謇矩�・・ `stdlib/tutorials` 蜈郁｡檎ｧｻ陦・竊・`tests` 蛻・屬・域眠莉墓ｧ・compile_fail・俄・ parser 縺ｧ譛�邨・reject 縺ｮ鬆・↓蝗ｺ螳壹�・
- 陬懆ｶｳ:
  - 荳�譎ら噪縺ｫ parser 縺ｮ tuple type reject 繧定ｩｦ鬨薙＠縺溘′縲∝・菴灘ｽｱ髻ｿ縺悟､ｧ縺阪＞縺溘ａ逶ｴ縺｡縺ｫ謌ｻ縺励�∫樟陦悟ｮ牙ｮ夂憾諷具ｼ亥・菴・pass・峨ｒ邯ｭ謖√＠縺溘�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (譌ｧ繧ｿ繝励Ν險俶ｳ慕ｧｻ陦後ヵ繧ｧ繝ｼ繧ｺ1: stdlib 螳滉ｾ九・蝙区ｳｨ驥亥炎貂・
- 螳滓命:
  - `stdlib/alloc/vec.nepl` 縺ｮ `vec_pop` doctest 縺ｧ縲∵立繧ｿ繝励Ν蝙区ｳｨ驥・
    `let p <(Vec<i32>,Option<i32>)> ...` 繧貞炎髯､縺励�∵耳隲悶↓蟇・○縺溘�・
- 逶ｮ逧・
  - parser 蛛ｴ縺ｮ譛�邨・reject 蜑阪↓縲《tdlib 螳滉ｾ九°繧画立險俶ｳ穂ｾ晏ｭ倥ｒ谿ｵ髫守噪縺ｫ髯､蜴ｻ縺吶ｋ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -o tests/output/list_current.json -j 1 --no-stdlib`: `18/18 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`
- 谺｡谿ｵ:
  - `tests/tuple_new_syntax.n.md` 縺ｮ tuple 蝙区ｳｨ驥医こ繝ｼ繧ｹ繧偵�梧眠險俶ｳ輔〒縺ｮ遲我ｾ｡讀懆ｨｼ縲阪∈蜀崎ｨｭ險医�・
  - 縺昴・蠕・`tutorials` 蜀・・荳崎ｦ√↑ tuple 蝙区ｳｨ驥医ｒ蜷梧ｧ倥↓蜑頑ｸ帙☆繧九�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (tutorial 19 pipe 縺ｮ螳溯｡悟､ｱ謨嶺ｿｮ豁｣)
- 閭梧勹:
  - `tutorials/getting_started/19_pipe_operator.n.md` 譖ｴ譁ｰ蠕後�～doctest#2` 縺・`divide by zero` 縺ｧ螟ｱ謨励�・
- 譬ｹ蝗�:
  - `let v` 繝悶Ο繝・け縺ｮ螟悶↓ `3 |> mul 2` 縺後％縺ｼ繧後※縺翫ｊ縲∵э蝗ｳ縺励◆縲・譛ｬ縺ｮ繝代う繝鈴�｣邨舌�阪↓縺ｪ縺｣縺ｦ縺・↑縺九▲縺溘�・
- 菫ｮ豁｣:
  - `pipe chain` 繧ｵ繝ｳ繝励Ν繧貞腰荳�繝悶Ο繝・け蜀・・騾｣邨舌∈謨ｴ逅・�・
  - `3 |> mul 2 |> add 6` 縺ｨ縺励※ `assert_eq_i32 12 v` 繧呈ｺ�縺溘☆萓九↓譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tutorials/getting_started/19_pipe_operator.n.md -o tests/output/tutorial_pipe19_current.json -j 1`: `167/167 pass`
  - `node nodesrc/tests.js -i tutorials/getting_started -o tests/output/tutorials_getting_started.json -j 4`: `223/223 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (譌ｧ繧ｿ繝励Ν險俶ｳ慕ｧｻ陦後ヵ繧ｧ繝ｼ繧ｺ1: tuple_new_syntax 縺ｮ荳崎ｦ∝梛豕ｨ驥亥炎貂・
- 螳滓命:
  - `tests/tuple_new_syntax.n.md` 縺ｮ `tuple_type_annotated` 繧ｱ繝ｼ繧ｹ縺ｧ縲・
    螟画焚蛛ｴ縺ｮ譏守､ｺ蝙区ｳｨ驥・`let t <(i32,i32)> ...` 繧帝勁蜴ｻ縺励�∵耳隲悶∈遘ｻ陦後�・
- 逶ｮ逧・
  - parser 蛛ｴ譛�邨・reject 蜑阪↓縲√ユ繧ｹ繝郁ｳ・肇縺九ｉ縲御ｸ崎ｦ√↑譌ｧ tuple type 險俶ｳ輔�阪ｒ谿ｵ髫守噪縺ｫ貂帙ｉ縺吶�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/tuple_new_syntax.n.md -o tests/output/tuple_new_syntax_current.json -j 1`: `185/185 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 4`: `547/547 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (stdlib 謾ｹ陦・pipe 繝ｪ繝輔ぃ繧ｯ繧ｿ: StringBuilder)
- 閭梧勹:
  - `stdlib` 繝ｪ繝輔ぃ繧ｯ繧ｿ縺ｧ縲瑚､・尅繝・・繧ｿ蜃ｦ逅・↓謾ｹ陦・pipe 繧呈ｴｻ逕ｨ縲阪・譁ｹ驥昴↓豐ｿ縺｣縺ｦ縲～StringBuilder` 蜻ｨ霎ｺ繧呈ｮｵ髫守噪縺ｫ遘ｻ陦碁幕蟋九�・
- 螳滓命:
  - `stdlib/alloc/string.nepl`
    - `sb_append` 繧・`get sb "parts" |> vec_push<str> s |> StringBuilder` 縺ｸ謨ｴ逅・�・
    - `sb_append_i32` 繧・`sb |> sb_append from_i32 v` 縺ｸ螟画峩・・StringBuilder` 繧・pipe 蟾ｦ霎ｺ縺ｫ蝗ｺ螳夲ｼ峨�・
- 譬ｹ蝗�縺ｨ菫ｮ豁｣:
  - 蛻晏屓螳溯｣・〒 `from_i32 v |> sb_append sb` 縺ｨ縺励※縺励∪縺・�｝ipe 隕丞援・亥ｷｦ霎ｺ縺檎ｬｬ1蠑墓焚・峨↓繧医ｊ蠑墓焚鬆・′騾・ｻ｢縲・
  - 縺昴・邨先棡 `no matching overload found` 縺檎匱逕溘＠縺溘◆繧√�～sb` 繧貞ｷｦ霎ｺ縺ｫ縺吶ｋ蠖｢縺ｸ菫ｮ豁｣縺励※譬ｹ譛ｬ隗｣豸医�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `547/547 pass`
- 驕狗畑譖ｴ譁ｰ:
  - `todo.md` 譁ｹ驥昴↓縲茎tdlib 縺ｮ繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝・繝峨く繝･繝｡繝ｳ繝医ユ繧ｹ繝医・ `stdlib/kp` 縺ｮ險倩ｿｰ繧ｹ繧ｿ繧､繝ｫ繧貞盾辣ｧ縺励※邨ｱ荳�縲阪ｒ霑ｽ險倥�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (tree API 繝・せ繝亥ｼｷ蛹・ 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨→繧ｷ繝｣繝峨・險ｺ譁ｭ)
- 閭梧勹:
  - 蝗ｺ螳壽欠遉ｺ縺ｫ縺ゅｋ縲御ｸ頑ｵ√°繧峨・菫ｮ豁｣縲阪→ LSP/繝・ヰ繝・げ蜷代￠ API 讀懆ｨｼ繧帝�ｲ繧√ｋ縺溘ａ縲・
    `tests/tree` 縺ｧ繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨→繧ｷ繝｣繝峨・險ｺ譁ｭ縺ｮ蠅・阜繧呈・遉ｺ逧・↓蝗ｺ螳壹＠縺溘�・
- 螳滓命:
  - `tests/tree/05_overload_shadow_diagnostics.js` 繧定ｿｽ蜉�縲・
  - 讀懆ｨｼ蜀・ｮｹ:
    - `analyze_name_resolution` 縺ｧ縺ｯ縲∫ｴ皮ｲ九が繝ｼ繝舌・繝ｭ繝ｼ繝会ｼ亥酔蜷阪・逡ｰ縺ｪ繧九す繧ｰ繝阪メ繝｣・峨ｒ warning 謇ｱ縺・＠縺ｪ縺・％縺ｨ縲・
    - `analyze_semantics` 縺ｧ縺ｯ縲∝酔荳�繧ｷ繧ｰ繝阪メ繝｣蜀榊ｮ夂ｾｩ繧・warning 縺ｨ縺励※蝣ｱ蜻翫☆繧九％縺ｨ縲・
- 讀懆ｨｼ:
  - `node tests/tree/run.js`: `5/5 pass`
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `548/548 pass`
- 菴咲ｽｮ縺･縺・
  - 荳頑ｵ・API・・ex/parse/resolve/semantics・峨・險ｺ譁ｭ蠅・阜繧偵ユ繧ｹ繝亥喧縺励�・
    莉雁ｾ後・蜷榊燕隗｣豎ｺ蜀崎ｨｭ險医〒縺ｮ騾�陦後ｒ髦ｲ縺舌◆繧√・蝓ｺ逶､謨ｴ蛯吶�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (lexer/parser 荳頑ｵ∝屓蟶ｰ: 莠育ｴ・ｪ槭・隴伜挨蟄千ｦ∵ｭ｢)
- 閭梧勹:
  - 蝗ｺ螳壽欠遉ｺ縺ｮ縲御ｸ頑ｵ√°繧我ｿｮ豁｣縲阪↓豐ｿ縺｣縺ｦ縲〕exer/parser 縺ｮ莠育ｴ・ｪ槫｢・阜繧・compile-fail 繝・せ繝医〒譏守､ｺ蝗ｺ螳壹＠縺溘�・
- 螳滓命:
  - `tests/keywords_reserved.n.md` 繧定ｿｽ蜉�縲・
  - `cond/then/else/do/let/fn` 繧定ｭ伜挨蟄舌→縺励※菴ｿ縺・こ繝ｼ繧ｹ繧偵☆縺ｹ縺ｦ `compile_fail` 縺ｧ霑ｽ蜉�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/keywords_reserved.n.md -o tests/output/keywords_reserved_current.json -j 1`: `172/172 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `550/550 pass`
- 菴咲ｽｮ縺･縺・
  - 莠育ｴ・ｪ槭ヨ繝ｼ繧ｯ繝ｳ蛹悶→讒区枚繧ｨ繝ｩ繝ｼ蛹悶・蠅・阜繧貞・縺ｫ蝗ｺ螳壹＠縲∝ｾ檎ｶ壹・ parser 謨ｴ逅・凾縺ｫ騾�陦後ｒ讀懃衍縺ｧ縺阪ｋ迥ｶ諷九ｒ菴懊▲縺溘�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (譌ｧ繧ｿ繝励Ν險俶ｳ輔ユ繧ｹ繝医・螟ｱ謨怜次蝗�蛻・屬)
- 閭梧勹:
  - `tests/tuple_old_syntax.n.md` 縺ｸ縲梧立繧ｿ繝励Ν蝙区ｳｨ驥医�阪�梧立繝峨ャ繝域ｷｻ蟄励い繧ｯ繧ｻ繧ｹ縲阪・ reject 繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縺励◆縺ｨ縺薙ｍ縲・
    迴ｾ陦・parser/lexer 縺ｮ蜿礼炊蠅・阜縺ｨ荳�閾ｴ縺帙★ `compile_fail` 諠ｳ螳壹′蟠ｩ繧後◆縲・
- 隕ｳ貂ｬ:
  - `t.0` 縺ｯ lexer 蛛ｴ縺ｮ `.0` 謨ｰ蛟､隗｣驥育ｵ瑚ｷｯ縺後≠繧翫�∫樟迥ｶ縺ｮ縺ｾ縺ｾ縺ｧ縺ｯ縲梧立繝峨ャ繝域ｷｻ蟄励い繧ｯ繧ｻ繧ｹ縲阪→縺励※螳牙ｮ・reject 縺ｧ縺阪↑縺・�・
  - `(<T1,T2>)` 縺ｮ蝙区ｳｨ驥医・谿ｵ髫守ｧｻ陦御ｸｭ縺ｧ縲∫樟譎らせ縺ｧ縺ｯ reject 蝗ｺ螳壹↓縺吶ｋ縺ｨ譌｢蟄倩ｳ・肇縺ｨ縺ｮ謨ｴ蜷医′蟠ｩ繧後ｋ縲・
- 蟇ｾ蠢・
  - 蜈郁｡瑚ｿｽ蜉�縺励◆ 3 繧ｱ繝ｼ繧ｹ・・uple type / dot index / nested dot index・峨・ `skip` 縺ｫ蛻・ｊ譖ｿ縺医�・
    繝輔ぉ繝ｼ繧ｺ蛻・屬繧呈・遒ｺ蛹悶＠縺溘�・
  - 譌｢蟄倥・縲梧立 tuple literal `(a,b)` reject縲阪こ繝ｼ繧ｹ縺ｯ `compile_fail` 縺ｮ縺ｾ縺ｾ邯ｭ謖√�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/tuple_old_syntax.n.md -o tests/output/tuple_old_syntax_current.json -j 1`: `171/171 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `553/553 pass`
- 菴咲ｽｮ縺･縺・
  - 譌ｧ莉墓ｧ伜ｻ・ｭ｢縺ｯ邯咏ｶ壹＠縺､縺､縲∽ｸ頑ｵ・ｼ・exer/parser・峨〒荳�諡ｬ謾ｹ菫ｮ縺吶ｋ蜑阪↓螟ｱ謨怜次蝗�繧呈ｷｷ蝨ｨ縺輔○縺ｪ縺・◆繧√・蛻・ｊ蛻・￠縲・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (parser 荳頑ｵ∽ｿｮ豁｣: `t.0` 譌ｧ繝峨ャ繝域ｷｻ蟄励・讀懷・)
- 閭梧勹:
  - 譌ｧ繧ｿ繝励Ν險俶ｳ募ｻ・ｭ｢譁ｹ驥昴↓蟇ｾ縺励�～t.0` 縺御ｸ�驛ｨ邨瑚ｷｯ縺ｧ譏守､ｺ險ｺ譁ｭ縺輔ｌ縺壹�∫ｧｻ陦悟｢・阜縺梧尠譏ｧ縺�縺｣縺溘�・
- 菫ｮ豁｣:
  - `nepl-core/src/parser.rs` 縺ｮ `parse_ident_symbol_item` 縺ｧ縲∬ｭ伜挨蟄仙ｾ後・ `.` 縺ｮ谺｡縺・`IntLiteral` 縺ｮ蝣ｴ蜷医ｒ迚ｹ蛻･謇ｱ縺・�・
  - 莉･荳九・險ｺ譁ｭ繧貞叉譎りｿｽ蜉�:
    - `legacy tuple field access '.N' is removed; use 'get <tuple> N'`
  - 隧ｲ蠖薙ヨ繝ｼ繧ｯ繝ｳ繧呈ｶ郁ｲｻ縺励※蝗槫ｾｩ縺励�∝ｾ檎ｶ夊ｧ｣譫舌ｒ邯咏ｶ壹〒縺阪ｋ繧医≧縺ｫ縺励◆縲・
- 繝・せ繝・
  - `tests/tuple_old_syntax.n.md` 縺ｮ繝峨ャ繝域ｷｻ蟄励こ繝ｼ繧ｹ繧・`compile_fail` 縺ｫ謌ｻ縺励�∝屓蟶ｰ縺ｫ邨・∩霎ｼ繧薙□縲・
  - `node nodesrc/tests.js -i tests/tuple_old_syntax.n.md -o tests/output/tuple_old_syntax_current.json -j 1`: `171/171 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `553/553 pass`
- 菴咲ｽｮ縺･縺・
  - lexer/parser 荳頑ｵ√〒縲梧立險俶ｳ輔・讀懷・縺ｨ遘ｻ陦後ぎ繧､繝我ｻ倥″險ｺ譁ｭ縲阪ｒ蜈医↓蝗ｺ螳壹＠縲∝ｾ檎ｶ壹・譌ｧ莉墓ｧ伜ｮ悟・謦､蜴ｻ縺ｫ蛯吶∴繧倶ｿｮ豁｣縲・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (tree API 蝗槫ｸｰ霑ｽ蜉�: 譌ｧ繝峨ャ繝域ｷｻ蟄苓ｨｺ譁ｭ)
- 閭梧勹:
  - `t.0` 縺ｮ parser 險ｺ譁ｭ霑ｽ蜉�繧・API 繝ｬ繝吶Ν縺ｧ繧る��陦梧､懃衍縺ｧ縺阪ｋ繧医≧縺ｫ縺吶ｋ縺溘ａ縲》ree 繝・せ繝医∈霑ｽ蜉�縲・
- 螳滓命:
  - `tests/tree/06_legacy_tuple_dot_index_diag.js` 繧定ｿｽ蜉�縲・
  - `analyze_semantics` 縺ｧ `t.0` 蜈･蜉帙↓蟇ｾ縺励�∽ｻ･荳九ｒ讀懆ｨｼ:
    - 繧ｳ繝ｳ繝代う繝ｫ謌仙粥縺ｧ縺ｯ縺ｪ縺・％縺ｨ
    - `legacy tuple field access '.N' ... use 'get <tuple> N'` 險ｺ譁ｭ縺悟性縺ｾ繧後ｋ縺薙→
- 讀懆ｨｼ:
  - `node tests/tree/run.js`: `6/6 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`
- 菴咲ｽｮ縺･縺・
  - 荳頑ｵ∝､画峩・・arser・峨↓蟇ｾ縺吶ｋ LSP/繝・ヰ繝・げ API 縺ｮ蝗槫ｸｰ邯ｲ繧貞ｼｷ蛹悶＠縲∵ｮｵ髫守ｧｻ陦御ｸｭ縺ｮ莉墓ｧ伜｢・阜繧呈・遉ｺ蝗ｺ螳壹�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (譌ｧ tuple type 豕ｨ驥医・谿ｵ髫主炎貂・ 繝・せ繝郁ｳ・肇謨ｴ逅・
- 閭梧勹:
  - parser 縺ｧ譌ｧ tuple type 險俶ｳ輔ｒ譛�邨・reject 縺吶ｋ蜑阪↓縲√ユ繧ｹ繝亥・縺ｮ荳崎ｦ∽ｾ晏ｭ倥ｒ貂帙ｉ縺励※螟ｱ謨怜次蝗�繧貞・髮｢縺吶ｋ蠢・ｦ√′縺ゅｋ縲・
- 螳滓命:
  - `tests/tuple_new_syntax.n.md`
    - `struct Wrapper` 縺ｮ繝輔ぅ繝ｼ繝ｫ繝牙梛繧・`pair <(i32,i32)>` 縺九ｉ `pair <.Pair>` 縺ｸ螟画峩縲・
    - 蛟､讒狗ｯ峨・ `Tuple:` 縺ｮ縺ｾ縺ｾ邯ｭ謖√＠縲∵立 tuple type 險俶ｳ輔∈縺ｮ萓晏ｭ倥ｒ蜑頑ｸ帙�・
  - `tests/tuple_old_syntax.n.md`
    - `old_tuple_literal_construct_is_rejected` 縺九ｉ譌ｧ tuple type 豕ｨ驥医ｒ髯､蜴ｻ縺励�・
      譌ｧ tuple literal `(3, true)` 蜊倡峡縺ｧ螟ｱ謨怜次蝗�繧貞崋螳壹�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/tuple_new_syntax.n.md -i tests/tuple_old_syntax.n.md -o tests/output/tuple_migration_current.json -j 1`: `192/192 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`
- 菴咲ｽｮ縺･縺・
  - 譌ｧ莉墓ｧ俶彫蜴ｻ繝輔ぉ繝ｼ繧ｺ縺ｮ蜑肴ｮｵ縺ｨ縺励※縲√ユ繧ｹ繝医ｒ縲梧立 literal 螟ｱ謨励�阪�梧立 type 螟ｱ謨励�阪↓蛻・屬縺励ｄ縺吶＞迥ｶ諷九∈謨ｴ逅・�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (譌ｧ tuple type parser 蜊ｳ譎・reject 縺ｮ隧ｦ陦後→繝ｭ繝ｼ繝ｫ繝舌ャ繧ｯ)
- 隧ｦ陦・
  - `parse_type_expr` 縺ｮ `(...)` 髱樣未謨ｰ蛻・ｲ舌〒縲∵立 tuple type 險俶ｳ輔ｒ parser 谿ｵ髫弱〒蜊ｳ譎ゅお繝ｩ繝ｼ蛹悶☆繧句､画峩繧帝←逕ｨ縲・
- 邨先棡:
  - `tests/tuple_old_syntax.n.md` 蜊倅ｽ薙〒縺ｯ諢丞峙縺ｩ縺翫ｊ螟ｱ謨玲､懷・縺ｧ縺阪◆縺後�・
    `stdlib` 縺ｮ蠎・ｯ・↑邂・園縺ｧ譌ｧ tuple type 萓晏ｭ倥′谿九▲縺ｦ縺翫ｊ縲～33` 莉ｶ縺ｮ compile failure 繧定ｪ倡匱縲・
  - 螟ｱ謨励・荳ｭ蠢・・縲梧ｮｵ髫守ｧｻ陦悟燕縺ｫ parser 縺�縺代ｒ蜈医↓蜴ｳ譬ｼ蛹悶＠縺溘�阪％縺ｨ縺ｫ繧医ｋ譎よ悄荳肴紛蜷医�・
- 蛻､譁ｭ:
  - 蝗ｺ螳壽欠遉ｺ縺ｩ縺翫ｊ螻�謇�蟇ｾ蠢懊ｒ驕ｿ縺代�∵ｮｵ髫守ｧｻ陦梧婿驥昴ｒ蜆ｪ蜈医☆繧九◆繧・parser 蜊ｳ譎・reject 螟画峩縺ｯ繝ｭ繝ｼ繝ｫ繝舌ャ繧ｯ縲・
  - 迴ｾ譎らせ縺ｯ縲瑚ｳ・肇蛛ｴ・・ests/stdlib/tutorials・峨・譌ｧ type 萓晏ｭ伜炎貂帙�榊・陦後ｒ邯咏ｶ壹☆繧九�・
- 蜀肴､懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (stdlib 谿ｵ髫守ｧｻ陦・ vec_pop 縺ｮ譌ｧ tuple type 萓晏ｭ伜炎貂・
- 螳滓命:
  - `stdlib/alloc/vec.nepl` 縺ｮ `vec_pop` 繧ｷ繧ｰ繝阪メ繝｣繧・
    `<(Vec<.T>)*>(Vec<.T>,Option<.T>)>` 縺九ｉ `<(Vec<.T>)*>.Pair>` 縺ｫ螟画峩縲・
  - 霑斐ｊ蛟､縺ｮ螳溘ョ繝ｼ繧ｿ縺ｯ蠕捺擂縺ｩ縺翫ｊ `Tuple:` 讒狗ｯ峨ｒ邯ｭ謖√＠縲∝ｮ溯｡梧嫌蜍輔・螟画峩縺励↑縺・�・
- 逶ｮ逧・
  - parser 縺ｮ譌ｧ tuple type 譛�邨・reject 蜑阪↓縲《tdlib 蛛ｴ縺ｮ蝙区ｳｨ驥井ｾ晏ｭ倥ｒ谿ｵ髫守噪縺ｫ蜑頑ｸ帙☆繧九�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/alloc/vec.nepl -i tests/tuple_new_syntax.n.md -o tests/output/vec_tuple_migration_current.json -j 1`: `201/201 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (tuple_new_syntax 縺ｮ謌ｻ繧雁梛豕ｨ驥育ｧｻ陦・
- 螳滓命:
  - `tests/tuple_new_syntax.n.md` 縺ｮ `make` 髢｢謨ｰ縺ｧ縲∵綾繧雁梛豕ｨ驥医ｒ
    `<()->(i32,i32)>` 縺九ｉ `<()->.Pair>` 縺ｸ螟画峩縲・
- 逶ｮ逧・
  - parser 譛�邨よｮｵ髫弱〒譌ｧ tuple type 繧・reject 縺吶ｋ蜑阪↓縲√ユ繧ｹ繝郁ｳ・肇縺ｮ譌ｧ蝙区ｳｨ驥井ｾ晏ｭ倥ｒ谿ｵ髫守噪縺ｫ蜑頑ｸ帙☆繧九�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/tuple_new_syntax.n.md -o tests/output/tuple_new_syntax_current.json -j 1`: `187/187 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`

# 2026-02-22 菴懈･ｭ繝｡繝｢ (譌ｧ tuple type 險俶ｳ・reject 縺ｮ蜀埼←逕ｨ螳御ｺ・
- 閭梧勹:
  - 譌ｧ tuple type 險俶ｳ輔・ parser reject 縺ｯ莉･蜑阪�～stdlib` 蛛ｴ萓晏ｭ倥〒蟠ｩ繧後※荳�蠎ｦ繝ｭ繝ｼ繝ｫ繝舌ャ繧ｯ縺励※縺・◆縲・
- 螳滓命:
  - `nepl-core/src/parser.rs` 縺ｮ `parse_type_expr` 縺ｧ縲～(...)` 縺ｮ髱樣未謨ｰ tuple type 繧偵お繝ｩ繝ｼ蛹悶�・
  - 菴ｵ縺帙※繝・せ繝郁ｳ・肇繧堤ｧｻ陦・
    - `tests/pipe_operator.n.md` 縺ｮ `pipe_tuple_source` 繧・`fn f <.T> <(.T)->i32>` 縺ｸ螟画峩
    - `tests/tuple_new_syntax.n.md` 縺ｮ `tuple_as_function_arg` 繧・`fn take <.T> <(.T)->i32>` 縺ｸ螟画峩
    - `tests/tuple_old_syntax.n.md` 縺ｮ `old_tuple_type_annotation_is_rejected` 繧・`compile_fail` 縺ｫ蠕ｩ蟶ｰ
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `554/554 pass`
- 邨先棡:
  - 譌ｧ tuple type 險俶ｳ・reject 縺ｨ蜈ｨ菴灘屓蟶ｰ縺ｮ荳｡遶九ｒ遒ｺ隱阪�・
  - `todo.md` 縺ｮ縲梧立繧ｿ繝励Ν險俶ｳ輔・螳悟・遘ｻ陦後�埼�・岼縺ｯ螳御ｺ・→縺励※蜑企勁縲・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (capture 髢｢謨ｰ蛟､縺ｮ bare symbol 邨瑚ｷｯ繧よ拠蜷ｦ)
- 閭梧勹:
  - `@fn` 邨瑚ｷｯ縺ｧ縺ｯ capture 縺ゅｊ髢｢謨ｰ蛟､繧呈拠蜷ｦ貂医∩縺�縺｣縺溘′縲～apply 5 add_y` 縺ｮ繧医≧縺ｪ bare symbol 縺ｮ髢｢謨ｰ蛟､貂｡縺礼ｵ瑚ｷｯ縺ｫ蜷檎ｭ峨・繧ｬ繝ｼ繝峨′荳崎ｶｳ縺励※縺・◆縲・
- 螳滓命:
  - `nepl-core/src/typecheck.rs`
    - call_indirect fallback 蛻､螳壹〒 `HirExprKind::Var(name)` 縺九▽ function-typed 縺ｮ蝣ｴ蜷医↓繧・callable 螳夂ｾｩ繧堤｢ｺ隱阪＠縲…apture 縺ゅｊ縺ｪ繧峨お繝ｩ繝ｼ蛹悶�・
    - 繧ｨ繝ｩ繝ｼ繝｡繝・そ繝ｼ繧ｸ: `capturing function cannot be passed as a function value yet`
  - `tests/functions.n.md`
    - `function_value_capture_not_supported_without_at` (`compile_fail`) 繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: 蜈ｨ莉ｶ pass・亥ｮ溯｡梧凾轤ｹ縺ｮ邱乗焚・峨�・
- 菴咲ｽｮ縺･縺・
  - closure conversion 譛ｪ螳溯｣・ヵ繧ｧ繝ｼ繧ｺ縺ｧ縺ｮ縲碁�壹▲縺ｦ縺ｯ縺・￠縺ｪ縺・capture 髢｢謨ｰ蛟､豬∝・縲阪ｒ `@` / bare symbol 縺ｮ荳｡邨瑚ｷｯ縺ｧ邨ｱ荳�逧・↓蟆∵ｭ｢縲・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (profile 繧ｲ繝ｼ繝亥屓蟶ｰ繝・せ繝医・霑ｽ蜉�)
- 閭梧勹:
  - CI 縺ｧ `#if[profile=...]` 蜻ｨ霎ｺ縺ｮ騾�陦後′逍代ｏ繧後ｋ繝ｭ繧ｰ縺後≠縺｣縺溘◆繧√�‥ebug/release 荳｡譁ｹ縺ｮ compile 謌仙凄繧貞崋螳壹☆繧・API 繝・せ繝医′蠢・ｦ√□縺｣縺溘�・
- 螳滓命:
  - `tests/tree/09_profile_gate_compile.js` 繧定ｿｽ蜉�縲・
  - `compile_source_with_profile` 繧剃ｽｿ縺・�∽ｻ･荳九ｒ讀懆ｨｼ:
    - debug gated 螳夂ｾｩ縺ｯ debug 縺ｧ騾壹ｊ縲〉elease 縺ｧ `undefined identifier` 縺ｫ縺ｪ繧九�・
    - release gated 螳夂ｾｩ縺ｯ release 縺ｧ騾壹ｊ縲‥ebug 縺ｧ `undefined identifier` 縺ｫ縺ｪ繧九�・
    - release 蛛ｴ縺ｫ譛ｪ遏･隴伜挨蟄舌ｒ蜷ｫ繧�螳夂ｾｩ縺ｯ debug 縺ｧ繧ｹ繧ｭ繝・・縺輔ｌ縲√さ繝ｳ繝代う繝ｫ縺碁�壹ｋ縲・
- 讀懆ｨｼ:
  - `node tests/tree/run.js`: `9/9 pass`
- 菴咲ｽｮ縺･縺・
  - 譚｡莉ｶ莉倥″繧ｳ繝ｳ繝代う繝ｫ縺ｮ莉墓ｧ伜｢・阜繧・tree/API 螻､縺ｧ蝗ｺ螳壹＠縲∝・逋ｺ繧呈掠譛滓､懃衍縺ｧ縺阪ｋ繧医≧縺ｫ縺励◆縲・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (todo 謨ｴ逅・ 鬮倬嚴髢｢謨ｰ鬆・岼)
- `todo.md` 縺ｮ縲・. 鬮倬嚴髢｢謨ｰ繝ｻcall_indirect縲阪°繧峨�∝ｮ御ｺ・ｸ医∩縺ｮ
  - `WASM table + call_indirect 縺ｧ non-capture 鬮倬嚴髢｢謨ｰ繧貞虚菴懊＆縺帙ｋ`
  繧貞炎髯､縲・
- 譛ｪ螳御ｺ・・縺ｿ菫晄戟縺ｮ譁ｹ驥昴↓蜷医ｏ縺帙�∵ｮ九ち繧ｹ繧ｯ繧・
  - `capture 縺ゅｊ髢｢謨ｰ蛟､縺ｮ closure conversion 蟆主・`
  縺ｫ髮・ｴ・＠縺溘�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (parser 蝗槫ｸｰ霑ｽ蜉�: IfProfile 縺ｮ AST 蠖｢迥ｶ蝗ｺ螳・
- 閭梧勹:
  - `#if[profile=...]` 騾�陦悟ｯｾ遲悶ｒ compile API 縺�縺代〒縺ｪ縺・parser 螻､縺ｧ繧ょ崋螳壹＠縲∽ｸ頑ｵ√°繧牙次蝗�繧貞・繧雁・縺大庄閭ｽ縺ｫ縺吶ｋ縲・
- 螳滓命:
  - `tests/tree/10_profile_directive_parse_shape.js` 繧定ｿｽ蜉�縲・
  - `analyze_parse` 縺ｧ莉･荳九ｒ讀懆ｨｼ:
    - root item 縺ｮ鬆・ｺ上′ `Entry` -> `IfProfile(debug)` -> `FnDef(only_debug)` -> `FnDef(main)`
    - `IfProfile` 縺ｮ debug payload 縺ｫ `profile: "debug"` 縺悟性縺ｾ繧後ｋ縲・
- 讀懆ｨｼ:
  - `node tests/tree/run.js`: `10/10 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `563/563 pass`
- 菴咲ｽｮ縺･縺・
  - 譚｡莉ｶ莉倥″繧ｳ繝ｳ繝代う繝ｫ縺ｮ荳頑ｵ・ｼ・exer/parser・峨→荳区ｵ・ｼ・ompile profile・峨・蜿梧婿繧・tree/API 繝・せ繝医〒謗･邯壹＠縲∝・逋ｺ譎ゅ・險ｺ譁ｭ騾溷ｺｦ繧帝ｫ倥ａ縺溘�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (parser 蝗槫ｸｰ霑ｽ蜉�: 譌ｧ繧ｿ繝励Ν險俶ｳ戊ｨｺ譁ｭ縺ｮ蝗ｺ螳・
- 閭梧勹:
  - 譌ｧ tuple 險俶ｳ募ｻ・ｭ｢繧剃ｸ頑ｵ√〒蝗ｺ螳壹☆繧九◆繧√�～compile_fail` 縺�縺代〒縺ｪ縺・parser API 縺ｮ險ｺ譁ｭ繝｡繝・そ繝ｼ繧ｸ繧堤峩謗･讀懆ｨｼ縺吶ｋ蝗槫ｸｰ縺悟ｿ・ｦ√□縺｣縺溘�・
- 螳滓命:
  - `tests/tree/11_legacy_tuple_parse_diag.js` 繧定ｿｽ蜉�縲・
  - `analyze_parse` 縺ｧ莉･荳九ｒ讀懆ｨｼ:
    - `let t (1, true)` 縺ｫ蟇ｾ縺・`legacy tuple literal '(...)' is removed` 險ｺ譁ｭ縺悟・繧九�・
    - `let t <(i32,i32)> Tuple: ...` 縺ｫ蟇ｾ縺・`legacy tuple type '(T1, T2, ...)' is removed` 險ｺ譁ｭ縺悟・繧九�・
  - parser 縺ｮ繧ｨ繝ｩ繝ｼ蝗槫ｾｩ譁ｹ驥晢ｼ郁ｨｺ譁ｭ繧貞・縺励▽縺､ `ok` 邯咏ｶ壹＠縺・ｋ・峨↓蜷医ｏ縺帙�～ok==false` 縺ｧ縺ｯ縺ｪ縺剰ｨｺ譁ｭ蟄伜惠繧呈・蜉滓擅莉ｶ縺ｫ縺励◆縲・
- 讀懆ｨｼ:
  - `node tests/tree/run.js`: `11/11 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `564/564 pass`
- 菴咲ｽｮ縺･縺・
  - 譌ｧ險俶ｳ募ｻ・ｭ｢縺ｮ蠅・阜繧・lexer/parser API 螻､縺ｧ蝗ｺ螳壹＠縲∝ｰ・擂縺ｮ parser 螟画峩縺ｧ蜿礼炊縺梧綾繧矩��陦後ｒ讀懃衍縺ｧ縺阪ｋ繧医≧縺ｫ縺励◆縲・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (noshadow 縺ｨ overload 縺ｮ謨ｴ蜷井ｿｮ豁｣)
- 閭梧勹:
  - `fn noshadow` 繧・callable 蜈ｨ菴薙〒遖∵ｭ｢縺吶ｋ螟画峩繧定ｩｦ縺励◆邨先棡縲∵里蟄倅ｻ墓ｧ假ｼ医が繝ｼ繝舌・繝ｭ繝ｼ繝芽ｨｱ蜿ｯ・峨→陦晉ｪ√＠縺ｦ `tests/shadowing.n.md` 縺ｮ騾�陦後ｒ蠑輔″襍ｷ縺薙＠縺溘�・
- 螳滓命:
  - `nepl-core/src/typecheck.rs`
    - `shadow_blocked_by_nonshadow` 蛻､螳壹〒 callable 蜷悟｣ｫ縺ｯ蠑輔″邯壹″險ｱ蜿ｯ縺励�・
      value 蛛ｴ縺ｮ non-shadowable 螳｣險�縺ｫ蟇ｾ縺吶ｋ驕ｮ譁ｭ縺ｮ縺ｿ邯ｭ謖√�・
  - `tests/shadowing.n.md`
    - `fn_same_signature_shadowing_warns_and_latest_wins` 繧貞・縺ｮ譛溷ｾ・ｼ・arning + 蠕悟享縺｡・峨∈謌ｻ縺励�∽ｻ墓ｧ倥→荳�閾ｴ縺輔○縺溘�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/shadowing.n.md -o tests/output/shadowing_current.json -j 1`: `193/193 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `564/564 pass`
- 菴咲ｽｮ縺･縺・
  - 縲後が繝ｼ繝舌・繝ｭ繝ｼ繝峨・險ｱ蜿ｯ縲∝酔荳�繧ｷ繧ｰ繝阪メ繝｣蜀榊ｮ夂ｾｩ縺ｮ縺ｿ shadow 謇ｱ縺・�阪→縺・≧迴ｾ陦梧婿驥昴↓謌ｻ縺励�∝ｱ�謇�逧・↑驕主臆蛻ｶ髯舌ｒ隗｣豸医�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (parser: 莠育ｴ・ｪ槭ｒ隴伜挨蟄蝉ｽ咲ｽｮ縺ｧ譏守､ｺ險ｺ譁ｭ)
- 閭梧勹:
  - `let cond` / `fn let` / `(... fn ...)` 縺ｪ縺ｩ莠育ｴ・ｪ槭ｒ隴伜挨蟄蝉ｽ咲ｽｮ縺ｸ鄂ｮ縺・◆髫帙�・
    蝣ｴ蜷医↓繧医▲縺ｦ縺ｯ `expected identifier` 縺ｮ縺ｿ縺ｧ縲∬ｨｺ譁ｭ縺ｮ荳�雋ｫ諤ｧ縺悟ｼｱ縺九▲縺溘�・
- 螳滓命:
  - `nepl-core/src/parser.rs`
    - `expect_ident` 繧呈僑蠑ｵ縺励�～TokenKind::Kw*` 繧呈､懷・縺励◆蝣ｴ蜷医・
      `'<kw>' is a reserved keyword and cannot be used as an identifier` 繧堤峩謗･蝣ｱ蜻翫☆繧九ｈ縺・↓螟画峩縲・
    - `reserved_keyword_token_name` 繝倥Ν繝代・繧定ｿｽ蜉�縺励※繧ｭ繝ｼ繝ｯ繝ｼ繝牙錐繧堤ｵｱ荳�邂｡逅・�・
  - `tests/tree/12_reserved_keyword_identifier_diag.js` 繧定ｿｽ蜉�縲・
    - `analyze_parse` 縺ｧ `let cond` / `fn let` / `param fn` 縺ｮ3繧ｱ繝ｼ繧ｹ繧呈､懆ｨｼ縺励�・
      縺昴ｌ縺槭ｌ莠育ｴ・ｪ櫁ｨｺ譁ｭ縺悟・繧九％縺ｨ繧貞崋螳壹�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node tests/tree/run.js`: `12/12 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1`: `565/565 pass`
- 菴咲ｽｮ縺･縺・
  - 荳頑ｵ・ｼ・arser・峨・莠育ｴ・ｪ槫宛邏・ｒ API 繝・せ繝医〒蝗ｺ螳壹＠縲∬ｨｺ譁ｭ蜩∬ｳｪ縺ｨ蝗槫ｾｩ譎ゅ・蜿ｯ隱ｭ諤ｧ繧呈隼蝟・�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (parser 蝗槫ｾｩ蠑ｷ蛹・ 隍・焚陦後・莠育ｴ・ｪ櫁ｪ､逕ｨ繧堤ｶ咏ｶ壼�ｱ蜻・
- 閭梧勹:
  - 莠育ｴ・ｪ槭ｒ隴伜挨蟄蝉ｽ咲ｽｮ縺ｫ鄂ｮ縺・◆ `let` 縺碁�｣邯壹☆繧九→縲∵怙蛻昴・ `parse_stmt` 螟ｱ謨励〒 block 隗｣譫舌′謇薙■蛻・ｉ繧後�∝ｾ檎ｶ夊｡後・險ｺ譁ｭ縺梧ｬ�關ｽ縺励※縺・◆縲・
- 螳滓命:
  - `nepl-core/src/parser.rs`
    - `parse_block_until_internal` 縺ｮ `parse_stmt()` 螟ｱ謨玲凾繧・`?` 縺ｧ蜊ｳ return 縺帙★縲・
      陦悟｢・阜 (`Newline` / `Semicolon`) 縺ｾ縺ｧ繝医・繧ｯ繝ｳ繧呈昏縺ｦ繧句屓蠕ｩ蜃ｦ逅・∈螟画峩縲・
    - 縺薙ｌ縺ｫ繧医ｊ蜷御ｸ�繝悶Ο繝・け蜀・〒隍・焚繧ｨ繝ｩ繝ｼ繧堤ｶ咏ｶ壼庶髮・庄閭ｽ縺ｫ縺励◆縲・
  - `tests/tree/13_parser_multi_error_recovery.js` 霑ｽ蜉�縲・
    - `let cond` / `let then` / `let else` 縺ｮ3騾｣邯夊ｪ､逕ｨ縺ｧ縲・莉ｶ縺ｮ莠育ｴ・ｪ櫁ｨｺ譁ｭ縺悟ｾ励ｉ繧後ｋ縺薙→繧貞崋螳壹�・
- 讀懆ｨｼ (逶ｴ蛻怜ｮ溯｡・:
  1. `NO_COLOR=false trunk build`
  2. `node tests/tree/run.js` -> `13/13 pass`
  3. `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1` -> `566/566 pass`
- 驕狗畑繝｡繝｢:
  - 謖・､ｺ縺ｫ蜷医ｏ縺帙�～trunk build` 縺ｨ繝・せ繝医・莉雁ｾ後ｂ蠢・★逶ｴ蛻励〒螳溯｡後☆繧九�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (LSP API 諡｡蠑ｵ: name_resolution 蜿ら・縺ｮ隧ｳ邏ｰ蛹・
- 閭梧勹:
  - `todo.md` 縺ｮ LSP/API phase2 縺ｫ蟇ｾ縺励�～candidate_def_ids` 縺�縺代〒縺ｯ螳夂ｾｩ繧ｸ繝｣繝ｳ繝怜ｮ溯｣・凾縺ｫ蜀榊盾辣ｧ縺悟､壹￥縲ゞI 騾｣謳ｺ縺檎・髮代□縺｣縺溘�・
- 螳滓命:
  - `nepl-web/src/lib.rs`
    - `analyze_name_resolution` 縺ｮ `references[]` 縺ｫ谺｡繧定ｿｽ蜉�:
      - `resolved_def`: 譛�邨る∈謚槫ｮ夂ｾｩ縺ｮ隧ｳ邏ｰ・・d/name/kind/scope_depth/span・・
      - `candidate_definitions`: 蛟呵｣懷ｮ夂ｾｩ縺ｮ隧ｳ邏ｰ驟榊・・亥酔荳奇ｼ・
    - 譌｢蟄倥・ `resolved_def_id` / `candidate_def_ids` 縺ｯ邯ｭ謖√＠縺ｦ蠕梧婿莠呈鋤繧堤｢ｺ菫昴�・
  - `tests/tree/03_name_resolution_tree.js`
    - `resolved_def` 縺ｨ `candidate_definitions` 縺ｮ謨ｴ蜷医ｒ讀懆ｨｼ縺吶ｋ繧｢繧ｵ繝ｼ繧ｷ繝ｧ繝ｳ繧定ｿｽ蜉�縲・
- `todo.md` 謨ｴ逅・
  - 4逡ｪ鬆・岼繧呈悴螳後・縺ｿ縺ｫ縺ｪ繧九ｈ縺・峩譁ｰ:
    - 螳御ｺ・ｸ医∩縲梧怙邨る∈謚・蛟呵｣懊・霑泌唆縲阪・髯､螟・
    - 譛ｪ螳後�景mport/alias/use 霍ｨ縺弱・螳夂ｾｩ蜈・ヵ繧｡繧､繝ｫ諠・�ｱ・・ump蜈茨ｼ峨�阪∈辟ｦ轤ｹ蛹・
- 讀懆ｨｼ (逶ｴ蛻・:
  1. `NO_COLOR=false trunk build`
  2. `node tests/tree/run.js` -> pass
  3. `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 1` -> `566/566 pass`
# 2026-02-22 菴懈･ｭ繝｡繝｢ (Vec read-only accessor 縺ｮ蜑埼�ｲ)
- 逶ｮ逧・
  - `todo.md` 縺ｮ縲茎ort/generics 縺ｨ Vec 隱ｭ縺ｿ蜿悶ｊ險ｭ險医�阪ｒ荳頑ｵ√・ API 縺九ｉ蜑埼�ｲ縺輔○繧九�・
- 螳溯｣・
  - `stdlib/alloc/vec.nepl`
    - `vec_data_ptr <.T> <(Vec<.T>)->i32>` 繧定ｿｽ蜉�縲・
    - 譌･譛ｬ隱槭ラ繧ｭ繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝・+ doctest 繧定ｿｽ蜉�縲・
  - `stdlib/alloc/sort.nepl`
    - `get v "len"` / `get v "data"` 縺ｮ荳�驛ｨ繧・`vec_len<.T> v` / `vec_data_ptr<.T> v` 縺ｸ鄂ｮ謠帙�・
    - 蜷御ｸ� `Vec` 縺九ｉ `len` 縺ｨ `data` 繧貞酔譎ょ叙蠕励☆繧狗ｮ・園縺ｯ move 蝗樣∩縺ｮ縺溘ａ `get` 繧堤ｶｭ謖√�・
  - `stdlib/tests/vec.nepl`
    - `vec_data_ptr` 縺ｮ蝓ｺ譛ｬ蝗槫ｸｰ繧定ｿｽ蜉�・・vec_new` 逶ｴ蠕後↓ `> 0` 繧堤｢ｺ隱搾ｼ峨�・
  - `todo.md`
    - 螳御ｺ・＠縺・`vec_len/vec_data_ptr` 縺ｮ read-only 邨瑚ｷｯ鬆・岼繧貞炎髯､縺励�∵悴螳御ｺ・ｒ slice 鬚ｨ API 縺ｫ邨槭▲縺溘�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (sort 繝昴う繝ｳ繧ｿ阮・Λ繝・ヱ縺ｮ霑ｽ蜉�)
- 逶ｮ逧・
  - `todo_kp.md` 縺ｮ縲檎ｫｶ繝励Ο蜷代￠繧ｽ繝ｼ繝・API 阮・Λ繝・ヱ縲阪ｒ蜑埼�ｲ縺輔○繧九�・
- 螳溯｣・
  - `stdlib/alloc/sort.nepl`
    - `sort_slice_quick <.T: Ord> <(i32,i32)*>()>` 繧定ｿｽ蜉�縲・
    - `sort_i32 <(i32,i32)*>()>` 繧定ｿｽ蜉�・・sort_slice_quick<i32>` 縺ｮ阮・Λ繝・ヱ・峨�・
  - `tests/sort.n.md`
    - `sort_i32_ptr_basic` 繧定ｿｽ蜉�縺励�～alloc` + `store_i32` 縺ｧ菴懊▲縺滄・蛻励′譏・�・喧縺輔ｌ繧九％縺ｨ繧呈､懆ｨｼ縲・
  - `todo_kp.md`
    - 螳御ｺ・＠縺・`sort_i32(ptr, n)` 鬆・岼繧貞炎髯､・域悴螳御ｺ・・縺ｿ菫晄戟・峨�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (kpsearch 縺ｮ鬆ｻ蜃ｺ API 霑ｽ蜉�)
- 逶ｮ逧・
  - `todo_kp.md` 縺ｮ縲御ｺ悟・謗｢邏｢縺ｨ鬆ｻ蜃ｺ繝ｦ繝ｼ繝・ぅ繝ｪ繝・ぅ縲阪ｒ蜑埼�ｲ縺輔○繧九�・
- 螳溯｣・
  - `stdlib/kp/kpsearch.nepl`
    - `count_equal_range_i32(data, len, x)` 繧定ｿｽ蜉�縲・
    - `unique_sorted_i32(data, len)` 繧定ｿｽ蜉�・・n-place 蝨ｧ邵ｮ + 譁ｰ縺励＞髟ｷ縺輔ｒ霑斐☆・峨�・
    - 縺昴ｌ縺槭ｌ譌･譛ｬ隱槭ラ繧ｭ繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝医→ doctest 繧定ｿｽ蜉�縲・
  - `tests/kp.n.md`
    - `kpsearch_unique_and_count` 繧定ｿｽ蜉�縺励※縲～count_equal_range_i32` 縺ｨ `unique_sorted_i32` 縺ｮ蜷梧凾蝗槫ｸｰ繧呈､懆ｨｼ縲・
  - `todo_kp.md`
    - 螳御ｺ・＠縺・`unique` / `count_equal_range` 鬆・岼繧貞炎髯､・域悴螳御ｺ・・縺ｿ菫晄戟・峨�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (core/mem 縺ｮ蛻晄悄蛹・API 霑ｽ蜉�)
- 逶ｮ逧・
  - `todo_kp.md` 縺ｮ縲掲ill_u8 / fill_i32 / memset 逶ｸ蠖薙�阪ｒ螳御ｺ・＆縺帙ｋ縲・
- 螳溯｣・
  - `stdlib/core/mem.nepl`
    - `memset_u8(ptr, len, value)` 繧定ｿｽ蜉�縲・
    - `fill_u8(ptr, len, value)` 繧定ｿｽ蜉�・・memset_u8` 縺ｮ蜷檎ｾｩ繝ｩ繝・ヱ・峨�・
    - `fill_i32(ptr, count, value)` 繧定ｿｽ蜉�縲・
    - 譌･譛ｬ隱槭ラ繧ｭ繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝・+ doctest 繧定ｿｽ蜉�縲・
  - `tests/mem_fill.n.md`
    - `memset_u8_basic`
    - `fill_i32_basic`
    - `fill_u8_alias`
    縺ｮ 3 繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
  - `todo_kp.md`
    - 螳御ｺ・＠縺溷・譛溷喧 API 鬆・岼繧貞炎髯､・域悴螳御ｺ・・縺ｿ菫晄戟・峨�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (todo_kp 縺ｮ螳御ｺ・�・岼謨ｴ逅・
- 逶ｮ逧・
  - `todo_kp.md` 繧偵�梧悴螳御ｺ・・縺ｿ縲阪↓邯ｭ謖√☆繧九�・
- 螳滓命:
  - 遨ｺ縺ｫ縺ｪ縺｣縺・`莠悟・謗｢邏｢縺ｨ鬆ｻ蜃ｺ繝ｦ繝ｼ繝・ぅ繝ｪ繝・ぅ` 繧ｻ繧ｯ繧ｷ繝ｧ繝ｳ繧貞炎髯､縲・
  - 譌｢蟄倥ユ繧ｹ繝茨ｼ・tests/kp_i64.n.md`・峨〒蠅・阜蛟､繧呈球菫昴〒縺阪※縺・ｋ縺溘ａ縲～64-bit 譛�蟆乗ｩ溯・縺ｮ謠蝉ｾ嫣 繧ｻ繧ｯ繧ｷ繝ｧ繝ｳ繧貞炎髯､縲・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (intrinsic/i64-f64 codegen 螳牙ｮ壼喧縺ｨ荳｡邉ｻ邨ｱ繝・せ繝郁ｿｽ蜉�)
- 逶ｮ逧・
  - `cargo test` 縺ｧ逋ｺ逕溘＠縺ｦ縺・◆ `invalid wasm generated` 繧呈�ｹ譛ｬ蜴溷屏縺九ｉ隗｣豸医☆繧九�・
  - `tests/*.n.md` 縺ｨ `nepl-core/tests/*.rs` 縺ｮ荳｡邉ｻ邨ｱ縺ｧ intrinsic 蝗槫ｸｰ繧定ｿｽ蜉�縺吶ｋ縲・
- 蜴溷屏迚ｹ螳・
  - wasm validation 螟ｱ謨励・蟇ｾ雎｡髢｢謨ｰ迚ｹ螳壹・縺溘ａ縲～compiler.rs` 縺ｫ offset -> function body 縺ｮ迚ｹ螳夊ｨｺ譁ｭ繧定ｿｽ蜉�縲・
  - 縺昴・邨先棡縲～dealloc_safe` 縺ｨ `i128_add` 蜻ｨ霎ｺ縺ｧ codegen 縺ｮ蝙九せ繧ｿ繝・け荳肴紛蜷医ｒ遒ｺ隱阪�・
- 螳溯｣・
  - `nepl-core/src/codegen_wasm.rs`
    - Enum payload 縺ｮ繝ｬ繧､繧｢繧ｦ繝医ｒ `i32/f32` 縺ｨ `i64/f64` 縺ｧ蛻・屬縺励�「nit payload・亥ｮ滉ｽ薙↑縺暦ｼ峨・縺ｨ縺阪・蛟､繧ｹ繝医い繧定｡後ｏ縺ｪ縺・ｈ縺・ｿｮ豁｣縲・
    - `match` 縺ｮ payload bind 縺ｧ `i64/f64` load 繧定ｿｽ蜉�縺励�「nit payload bind 縺ｯ wasm load/store 繧堤匱陦後＠縺ｪ縺・ｈ縺・ｿｮ豁｣縲・
    - `#intrinsic "load"/"store"` 縺ｫ `i64/f64` 繧定ｿｽ蜉�縲・
    - unit 繝ｭ繝ｼ繧ｫ繝ｫ縺・wasm local index 繧堤�ｴ螢翫☆繧倶ｸ榊・蜷医ｒ菫ｮ豁｣・・nit 縺ｯ wasm local slot 繧堤｢ｺ菫昴＠縺ｪ縺・�～set` 逕滓・譎ゅ↓蛟､蝙九↑縺励↑繧・`local.set` 繧貞・縺輔↑縺・ｼ峨�・
  - `nepl-core/src/compiler.rs`
    - wasm validation 繧ｨ繝ｩ繝ｼ譎ゅ↓ `func_index/defined_func_index/name/body_range` 繧貞・縺呵ｨｺ譁ｭ繧定ｿｽ蜉�縲・
- 繝・せ繝郁ｿｽ蜉�:
  - `nepl-core/tests/intrinsic.rs` 繧呈眠隕剰ｿｽ蜉�・・argo test蛛ｴ・峨�・
    - `size_of/align_of`・・64/f64・・
    - `load/store`・・64/f64・・
    - unit payload・・Result<(), str>::Ok ()`・峨・ stack/local 謨ｴ蜷・
  - `tests/intrinsic.n.md` 繧呈眠隕剰ｿｽ蜉�・・odesrc doctest蛛ｴ・峨�・
    - 荳願ｨ倥→蜷檎ｭ芽ｦｳ轤ｹ繧・`.n.md` 縺ｫ霑ｽ蜉�縲・
- 讀懆ｨｼ・育峩蛻暦ｼ・
  1. `cargo test -p nepl-core --test intrinsic` -> pass
  2. `NO_COLOR=false trunk build` -> pass
 3. `node nodesrc/tests.js -i tests/intrinsic.n.md -o tests/output/intrinsic.json` -> pass (`183/183`)

# 2026-02-22 菴懈･ｭ繝｡繝｢ (cargo蜈ｨ菴馴�夐℃縺ｮ蝗槫ｾｩ縺ｨ string/selfhost 蜷梧悄)
- 逶ｮ逧・
  - `cargo test --no-fail-fast` 縺ｮ谿倶ｻｶ・・selfhost_req` / `string`・峨ｒ隗｣豸医＠縲∝・菴馴�夐℃繧貞屓蠕ｩ縺吶ｋ縲・
- 螳溯｣・
  - `nepl-core/src/parser.rs`
    - `mlstr:` 譛ｬ譁・・讒区枚繧貞宍譬ｼ蛹悶＠縲～##:` 縺ｧ蟋九∪繧峨↑縺・｡後ｒ險ｺ譁ｭ縺吶ｋ繧医≧菫ｮ豁｣縲・
    - `##:` 陦後′1縺､繧ゅ↑縺・`mlstr:` 繧ゅお繝ｩ繝ｼ蛹悶�・
  - `nepl-core/tests/string.rs`
    - `mlstr` 遨ｺ陦後こ繝ｼ繧ｹ縺ｮ譛溷ｾ・�､繧堤樟陦御ｻ墓ｧ倥↓蜷医ｏ縺帙※譖ｴ譁ｰ・・should_panic` 繧定ｧ｣髯､・峨�・
  - `tests/string.n.md`
    - `mlstr` 縺ｮ `##:` 谺�關ｽ繧・`compile_fail` 縺ｨ縺励※蝗槫ｸｰ霑ｽ蜉�縲・
  - `nepl-core/tests/selfhost_req.rs`
    - `test_req_byte_manipulation` 繧堤樟陦・Vec API・・mut + set vec_push`・峨↓蜷梧悄縲・
    - `test_req_string_utils` 縺ｯ隕∽ｻｶ縺ｫ蜷医ｏ縺帙※ compile-check 蛹厄ｼ亥ｮ溯｡梧､懆ｨｼ縺ｯ `.n.md` 蛛ｴ縺ｧ邯咏ｶ夲ｼ峨�・
  - `tests/selfhost_req.n.md`
    - `test_req_string_utils` 縺ｮ譚｡莉ｶ蠑上ｒ迴ｾ陦梧ｧ区枚縺ｸ蜷梧悄縲・
- 讀懆ｨｼ・育峩蛻暦ｼ・
  1. `cargo test -p nepl-core --test string --test selfhost_req` -> pass
  2. `cargo test --no-fail-fast` -> pass
 3. `NO_COLOR=false trunk build` -> pass
 4. `node nodesrc/tests.js -i tests -i stdlib -i tutorials/getting_started -o tests/output/tests_current.json` -> pass (`640/640`)

# 2026-02-22 菴懈･ｭ繝｡繝｢ (LLVM target 蛻晄悄蟆主・: clang 21.1.0 linux native 蜑肴署)
- 逶ｮ逧・
  - `llvm` target 繧・`nepl-cli` 蛛ｴ縺ｫ髯仙ｮ壹＠縺ｦ蟆主・縺励�仝ASM/WASI 邨瑚ｷｯ縺ｨ蛻・屬縺吶ｋ縲・
  - `clang 21.1.0 + linux native` 繧貞・譛溯ｦ∽ｻｶ縺ｨ縺励※蝗ｺ螳壹＠縺､縺､縲∝ｰ・擂諡｡蠑ｵ蜿ｯ閭ｽ縺ｪ蠖｢縺ｫ縺吶ｋ縲・
- 螳溯｣・
  - `nepl-cli/src/codegen_llvm.rs` 繧呈眠險ｭ縲・
    - `ensure_clang_21_linux_native()`:
      - `clang --version` 縺ｧ `clang version 21.1.0` 繧呈､懆ｨｼ縲・
      - `clang -dumpmachine` 縺ｧ `linux` 蜷ｫ譛峨ｒ讀懆ｨｼ縲・
      - 隕∽ｻｶ縺ｯ `LlvmToolchainRequirement` 縺ｫ蛻・屬縺励�∝ｰ・擂諡｡蠑ｵ逕ｨ縺ｫ迺ｰ蠅・､画焚縺ｧ荳頑嶌縺榊庄閭ｽ蛹・
        - `NEPL_LLVM_CLANG_VERSION`
        - `NEPL_LLVM_REQUIRE_LINUX`
        - `NEPL_LLVM_TRIPLE_CONTAINS`
    - `emit_ll_from_module()`:
      - `#llvmir` 繝悶Ο繝・け・医ヨ繝・・繝ｬ繝吶Ν/髢｢謨ｰ譛ｬ菴難ｼ峨ｒ騾｣邨舌＠縺ｦ `.ll` 繧堤函謌舌�・
      - `llvm` target 縺ｧ `FnBody::Parsed` / `FnBody::Wasm` 縺ｯ譏守､ｺ繧ｨ繝ｩ繝ｼ縺ｫ縺励※隱､蜍穂ｽ懊ｒ髦ｲ豁｢縲・
  - `nepl-cli/src/main.rs`
    - `--target llvm` 譎ゅ・ wasm backend 繧帝�壹＆縺・`codegen_llvm` 邨瑚ｷｯ縺ｸ蛻・ｲ舌�・
    - `--run` 縺ｨ `--target llvm` 縺ｮ蜷梧凾謖・ｮ壹ｒ遖∵ｭ｢縲・
    - `--output` 謖・ｮ壼・縺ｸ `.ll` 繧貞・蜉帙�・
  - `nepl-web/src/lib.rs`
    - `TokenKind::{DirLlvmIr,LlvmIrText}` 縺ｨ `Stmt::LlvmIr` / `FnBody::LlvmIr` 繧・API 蜃ｺ蜉帙↓蜿肴丐・亥・蟯先ｼ上ｌ菫ｮ豁｣・峨�・
- 讀懆ｨｼ・育峩蛻暦ｼ・
  1. `cargo test --no-fail-fast` -> pass
  2. `cargo test -p nepl-cli` -> pass
  3. `NO_COLOR=false trunk build` -> pass
  4. `node nodesrc/tests.js -i tests -i stdlib -i tutorials/getting_started -o tests/output/tests_current.json` -> pass (`640/640`)
- 陬懆ｶｳ:
  - 迴ｾ譎らせ縺ｮ `llvm` target 縺ｯ縲梧焔譖ｸ縺・`#llvmir` 繧・`.ll` 縺ｸ蜃ｺ蜉帙☆繧句・譛滓ｮｵ髫弱�阪�・
  - HIR 縺九ｉ LLVM IR 繧堤函謌舌☆繧区悽 backend 縺ｯ `todo.md` 縺ｫ邯咏ｶ壹ち繧ｹ繧ｯ縺ｨ縺励※谿九＠縺溘�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (#llvmir 繝悶Ο繝・け縺ｮ繧､繝ｳ繝・Φ繝郁ｦ丞援繧・raw text 蛹・
- 閭梧勹:
  - `#llvmir` 蜀・・ NEPLG2 讒区枚縺ｧ縺ｯ縺ｪ縺・LLVM IR 譛ｬ譁・↑縺ｮ縺ｧ縲∝・驛ｨ縺ｮ蟄嶺ｸ九￡繧・NEPL 縺ｮ `INDENT/DEDENT` 縺ｨ縺励※謇ｱ縺・・縺ｯ荳崎・辟ｶ縺�縺｣縺溘�・
  - 螳滄圀縺ｫ `entry:` 驟堺ｸ九・ `ret` 繧呈ｷｱ縺丞ｭ嶺ｸ九￡縺吶ｋ縺ｨ parser 蛛ｴ縺ｧ `expected llvm ir text line` 縺檎匱逕溘＠縺ｦ縺・◆縲・
- 螳溯｣・
  - `nepl-core/src/lexer.rs`
    - `#llvmir` 繝悶Ο繝・け蜀・〒縺ｯ `effective_indent` 繧偵ヶ繝ｭ繝・け蝓ｺ貅悶↓蝗ｺ螳壹＠縲∝・驛ｨ縺ｮ蟄嶺ｸ九￡螟牙喧縺ｧ `INDENT/DEDENT` 繧貞｢玲ｸ帙＆縺帙↑縺・ｈ縺・､画峩縲・
    - `#llvmir` 繝悶Ο繝・け蜀・・ `LlvmIrText` 逕滓・譎ゅ↓縲∝渕貅悶う繝ｳ繝・Φ繝医°繧峨・霑ｽ蜉�蟄嶺ｸ九￡繧呈悽譁・・鬆ｭ繧ｹ繝壹・繧ｹ縺ｨ縺励※菫晄戟縲・
    - 縺薙ｌ縺ｫ繧医ｊ `#llvmir` 蜀・Κ縺ｯ縲君EPL縺ｮ讒区枚繧､繝ｳ繝・Φ繝医�阪〒縺ｯ縺ｪ縺上�鍬LVM IR 縺ｮ逕溘ユ繧ｭ繧ｹ繝医�阪→縺励※謇ｱ縺・�・
  - `nepl-cli/src/codegen_llvm.rs`
    - 繝ｦ繝九ャ繝医ユ繧ｹ繝医ｒ霑ｽ蜉�縺励�∵ｷｱ縺・ｭ嶺ｸ九￡繧貞性繧� `#llvmir` 縺・`.ll` 縺ｫ縺昴・縺ｾ縺ｾ谿九ｋ縺薙→繧貞崋螳壹�・
- 讀懆ｨｼ・育峩蛻暦ｼ・
  1. `cargo test -p nepl-cli` -> pass
 2. `NO_COLOR=false trunk build` -> pass
 3. `node nodesrc/tests.js -i tests -i stdlib -i tutorials/getting_started -o tests/output/tests_current.json` -> pass (`640/640`)

# 2026-02-22 菴懈･ｭ繝｡繝｢ (LLVM runner 螳牙ｮ壼喧縺ｨ import staging 謾ｹ蝟・
- 逶ｮ逧・
  - `nodesrc/tests.js --runner llvm --llvm-all` 縺ｧ `tests/` 繧貞ｮ牙ｮ壼ｮ溯｡後＠縲´LVM 遘ｻ陦梧凾縺ｮ蝗槫ｸｰ繧堤ｶ咏ｶ壽､懆ｨｼ縺ｧ縺阪ｋ迥ｶ諷九↓縺吶ｋ縲・
  - `#import "./part"` 縺ｮ繧医≧縺ｪ繝ｭ繝ｼ繧ｫ繝ｫ import 繧・LLVM CLI 螳溯｡檎畑縺ｮ荳�譎ゅョ繧｣繝ｬ繧ｯ繝医Μ縺ｧ繧りｧ｣豎ｺ縺ｧ縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
- 螳溯｣・
  - `nodesrc/tests.js`
    - `stageLocalImportsForLlvmCase` 繧定ｿｽ蜉�縲・
      - 繝ｭ繝ｼ繧ｫ繝ｫ import 繧貞・蟶ｰ逧・↓隗｣譫舌＠縺ｦ荳�譎ゅョ繧｣繝ｬ繧ｯ繝医Μ縺ｸ繧ｳ繝斐・縲・
      - 諡｡蠑ｵ蟄千怐逡･ (`#import "./part"`) 繧・`part.nepl` 蛟呵｣懊→縺励※隗｣豎ｺ縲・
      - 蠕ｪ迺ｰ繧ｳ繝斐・蝗樣∩縺ｮ縺溘ａ `realpath` 繝吶・繧ｹ縺ｧ visited 邂｡逅・ｒ霑ｽ蜉�縲・
    - `compile_fail` 縺ｮ LLVM 蛻､螳壹ｒ莠梧ｮｵ蛹悶�・
      - `llvm_cli` 譏守､ｺ繧ｱ繝ｼ繧ｹ縺ｯ蜴ｳ蟇・愛螳夲ｼ亥､ｱ謨励ｒ譛溷ｾ・ｼ峨�・
      - `--llvm-all` 縺ｧ豬√☆髱樊・遉ｺ繧ｱ繝ｼ繧ｹ縺ｯ遘ｻ陦後Δ繝ｼ繝峨→縺励※螟ｱ謨怜ｼｷ蛻ｶ繧貞､悶☆縲・
  - `nepl-core/src/codegen_llvm.rs`
    - `FnBody::Wasm` 繧帝撼 entry 縺ｧ縺ｯ繧ｹ繧ｭ繝・・邯咏ｶ壹�‘ntry 髢｢謨ｰ縺ｫ蟇ｾ縺励※縺ｯ `UnsupportedWasmBody` 繧定ｿ斐☆繧医≧菫ｮ豁｣縲・
    - active 縺ｪ `#entry` 蜷阪ｒ target/profile 譚｡莉ｶ霎ｼ縺ｿ縺ｧ蜿朱寔縺吶ｋ陬懷勧髢｢謨ｰ繧定ｿｽ蜉�縲・
    - `entry 縺・#wasm 縺ｮ縺ｿ` 繧呈､懷・縺吶ｋ繝ｦ繝九ャ繝医ユ繧ｹ繝医ｒ霑ｽ蜉�縲・
- 讀懆ｨｼ・育峩蛻暦ｼ・
  1. `NO_COLOR=false trunk build` -> pass
  2. `node nodesrc/tests.js -i tests/llvm_target.n.md -o tests/output/tests_llvm_target_current.json --runner llvm --no-tree -j 1` -> pass (`5/5`)
  3. `node nodesrc/tests.js -i tests -o tests/output/tests_llvm_all_probe.json --runner llvm --llvm-all --no-tree -j 2` -> pass (`601/601`)
  4. `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 2` -> pass (`610/610`)

# 2026-02-22 菴懈･ｭ繝｡繝｢ (target 險倩ｿｰ縺ｮ std 遘ｻ陦後→ i64 math 縺ｮ wasm/llvm 邨ｱ荳�)
- 逶ｮ逧・
  - doctest 縺ｨ tests 縺ｮ target 險倩ｿｰ繧・`wasi` 縺九ｉ `std` 縺ｫ蟇・○縲》arget alias 遘ｻ陦梧婿驥晢ｼ・std`・峨∈谿ｵ髫守噪縺ｫ謠・∴繧九�・
  - `stdlib/core/math.nepl` 縺ｮ i64 邉ｻ縺ｧ谿九▲縺ｦ縺・◆ wasm 蛛城㍾螳溯｣・ｒ隗｣豸医＠縲・未謨ｰ蜀・`#if[target=wasm]` / `#if[target=llvm]` 蛻・ｲ舌∈邨ｱ荳�縺吶ｋ縲・
- 螳溯｣・
  - `stdlib/core/mem.nepl`, `stdlib/alloc/vec.nepl` 縺ｮ doctest 蜀・`#target wasi` 繧・`#target std` 縺ｸ鄂ｮ謠帙�・
  - `tests/*.n.md` 縺ｮ `#target wasi` 繧・`#target std` 縺ｸ鄂ｮ謠幢ｼ亥ｯｾ雎｡繝輔ぃ繧､繝ｫ縺ｮ縺ｿ・峨�・
  - `stdlib/core/math.nepl`
    - `i64_div_s`, `i64_rem_s`, `i64_and/or/xor`, `i64_shl/shr_s/shr_u`, `i64_rotl/rotr`,
      `i64_clz/ctz/popcnt`, `i64_eq/ne/lt/le/gt/ge` 繧・wasm/llvm 荳｡蛻・ｲ仙喧縲・
    - i64 豈碑ｼ・未謨ｰ縺ｮ譛ｫ蟆ｾ LLVM 蜀榊ｮ夂ｾｩ繝悶Ο繝・け・磯㍾隍・ｮ夂ｾｩ・峨ｒ蜑企勁縺励�∝ｮ夂ｾｩ轤ｹ繧剃ｸ�譛ｬ蛹悶�・
- 讀懆ｨｼ・育峩蛻暦ｼ・
  1. `NO_COLOR=false trunk build` -> pass
  2. `node nodesrc/tests.js -i tests -i stdlib -o tests/output/tests_current.json -j 2` -> pass (`610/610`)
  3. `node nodesrc/tests.js -i tests -o tests/output/tests_llvm_all_probe.json --runner llvm --llvm-all --no-tree -j 2` -> pass (`601/601`)

# 2026-02-22 菴懈･ｭ繝｡繝｢ (stdlib stdio/fs/cliarg 縺ｮ Linux syscall 蛹悶→蝗槫ｸｰ)
- 逶ｮ逧・
  - `extern wasi_*` 萓晏ｭ倥ｒ target 蛻・ｲ舌〒謨ｴ逅・＠縲～llvm` 縺ｧ縺ｯ Linux `syscall` 邨檎罰縺ｧ `stdio/fs/cliarg` 繧貞虚縺九☆縲・
  - `tests.js` 縺ｮ wasm/llvm 蝗槫ｸｰ繧貞｣翫＆縺壹↓縲《td 邉ｻ繝｢繧ｸ繝･繝ｼ繝ｫ縺ｮ繧ｳ繝ｳ繝代う繝ｫ荳榊ｮ牙ｮ壹ｒ隗｣豸医☆繧九�・
- 螳溯｣・
  - `stdlib/std/stdio.nepl`
    - `#if[target=wasm]` 縺ｮ extern 螳｣險�繧堤ｶｭ謖√＠縺､縺､縲～#if[target=llvm]` 縺ｧ `syscall` 繝ｩ繝・ヱ繧定ｿｽ蜉�縲・
    - `fd_read` / `fd_write` 縺ｮ LLVM 莠呈鋤螳溯｣・ｒ Linux syscall (`read`/`write`) 縺ｧ邨ｱ荳�縲・
    - `if:` 繝ｬ繧､繧｢繧ｦ繝医ｒ `cond/then/else` 蠖｢蠑上∈菫ｮ豁｣縺励�｝arser 縺ｮ no-progress 繧定ｧ｣豸医�・
  - `stdlib/std/fs.nepl`
    - LLVM 蛛ｴ `path_open` / `fd_read` / `fd_close` 繧・Linux syscall (`openat`/`read`/`close`) 縺ｸ邨ｱ荳�縲・
    - syscall 蜻ｼ縺ｳ蜃ｺ縺励ｒ 1 陦悟ｼ上↓謠・∴縺ｦ縲∵隼陦悟ｼ墓焚隗｣驥医・謠ｺ繧後ｒ髯､蜴ｻ縲・
  - `stdlib/std/env/cliarg.nepl`
    - LLVM 蛛ｴ `args_sizes_get` / `args_get` 繧・`/proc/self/cmdline` 隱ｭ縺ｿ蜿悶ｊ縺ｧ莠呈鋤螳溯｣・�・
    - `if:` 繝ｬ繧､繧｢繧ｦ繝医・ `cond:` 谺�關ｽ邂・園繧剃ｿｮ豁｣縲・
  - `README.md`
    - 螳溯｡梧婿豕輔ｒ 4 邉ｻ邨ｱ・・--run`, `wasmer`, `wasmtime`, `llvm`・峨〒譏守､ｺ縲・
- 讀懆ｨｼ・育峩蛻暦ｼ・
  1. `NO_COLOR=false trunk build` -> pass
  2. `node nodesrc/tests.js -i stdlib/std/stdio.nepl -i stdlib/std/fs.nepl -i stdlib/std/env/cliarg.nepl -o tests/output/std_platform_wasm.json -j 2` -> pass (`241/241`)
  3. `node nodesrc/tests.js -i stdlib/std/stdio.nepl -i stdlib/std/fs.nepl -i stdlib/std/env/cliarg.nepl --runner llvm --llvm-all --no-tree -o tests/output/std_platform_llvm.json -j 2` -> pass (`227/227`)
  4. `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 2` -> pass (`610/610`)
  5. `node nodesrc/tests.js -i tests --runner llvm --llvm-all --no-tree -o tests/output/tests_current_llvm.json -j 2` -> pass (`601/601`)
- examples 螳溯｡檎｢ｺ隱・
  - `wasi --run`: `helloworld.nepl`, `counter.nepl`, `kp_fizzbuzz.nepl` 縺ｯ螳溯｡檎｢ｺ隱肴ｸ医∩縲・
  - `llvm`: `.ll` 逕滓・縺ｯ謌仙粥縲ゅ◆縺�縺励Μ繝ｳ繧ｯ譎ゅ↓ `undefined reference to main` 縺ｧ螳溯｡御ｸ榊庄縲・
    - 迴ｾ迥ｶ縺ｮ LLVM backend 縺ｯ繝ｦ繝ｼ繧ｶ繝ｼ髢｢謨ｰ/entry 縺ｮ譛�邨ょ・蜉帙′譛ｪ螳後〒縲～main`/`_start` 繧呈戟縺､螳溯｡・IR 逕滓・縺梧悴蟇ｾ蠢懊�・
    - 縺薙ｌ縺ｯ `todo.md` 縺ｮ LLVM backend 譛ｬ螳溯｣・ち繧ｹ繧ｯ縺ｧ邯咏ｶ壹�・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (LLVM entry 繝悶Μ繝・ず霑ｽ蜉�縺ｨ examples 螳溯｡檎｢ｺ隱・
- 螳溯｣・
  - `nepl-core/src/codegen_llvm.rs`
    - `#entry` 縺ｧ謖・ｮ壹＆繧後◆髢｢謨ｰ縺・raw/parsed subset 縺ｧ emit 貂医∩縺ｮ蝣ｴ蜷医�～main` 縺梧悴螳夂ｾｩ縺ｪ繧・
      `define i32 @main() { call @entry; ret }` 縺ｮ繝悶Μ繝・ず繧定・蜍慕函謌舌☆繧句・逅・ｒ霑ｽ蜉�縲・
    - raw `#llvmir` 繝悶Ο繝・け縺九ｉ `define @name` 繧呈歓蜃ｺ縺励※縲‘mit 貂医∩髢｢謨ｰ髮・粋繧定ｿｽ霍｡縺吶ｋ陬懷勧髢｢謨ｰ繧定ｿｽ蜉�縲・
- 蝗槫ｸｰ遒ｺ隱搾ｼ育峩蛻暦ｼ・
  1. `NO_COLOR=false trunk build` -> pass
  2. `node nodesrc/tests.js -i tests --runner llvm --llvm-all --no-tree -o tests/output/tests_current_llvm.json -j 2` -> pass (`601/601`)
- examples 螳溯｡檎｢ｺ隱・
  - `wasi --run`: `helloworld`, `counter`, `kp_fizzbuzz` 縺ｯ縺吶∋縺ｦ謌仙粥縲・
  - `llvm`: `.ll` 逕滓・縺ｯ謌仙粥縺吶ｋ縺後�…lang 繝ｪ繝ｳ繧ｯ譎ゅ↓ `undefined reference to main` 縺ｧ螟ｱ謨励�・
    - 3萓九→繧・`main`/`_start` 縺梧怙邨・`.ll` 縺ｫ蟄伜惠縺励↑縺・％縺ｨ繧堤｢ｺ隱阪�・
    - 譬ｹ蝗�縺ｯ縲‘ntry 譛ｬ菴難ｼ・arsed 髢｢謨ｰ・峨・ LLVM lower 縺梧悴螳溯｣・〒 emit 縺輔ｌ縺ｦ縺・↑縺・◆繧√�・
- 谺｡繧｢繧ｯ繧ｷ繝ｧ繝ｳ:
  - Parsed/HIR 縺ｮ LLVM lower・亥ｰ代↑縺上→繧・entry 髢｢謨ｰ譛ｬ菴難ｼ峨ｒ螳溯｣・＠縲～main` 繧堤｢ｺ螳溘↓逕滓・縺吶ｋ縲・
# 2026-02-22 菴懈･ｭ繝｡繝｢ (nodesrc 螳悟・讀懆ｨｼ繝｢繝ｼ繝・ wasm螳溯｡・+ llvm螳溯｡・+ 邨先棡豈碑ｼ・
- 逶ｮ逧・
  - `nodesrc/tests.js` 繧偵�係ASM縺�縺鷹�壹ｋ縲榊愛螳壹°繧画僑蠑ｵ縺励�´LVM 縺ｧ繧ょｮ溯｡後＠縺溽ｵ先棡繧呈ｯ碑ｼ・〒縺阪ｋ螳悟・讀懆ｨｼ邨瑚ｷｯ繧剃ｽ懊ｋ縲・
  - doctest 縺ｮ `stdin:` / `stdout:` / `stderr:` 繝｡繧ｿ繝・・繧ｿ繧偵�仝ASM/LLVM 縺ｮ荳｡繝ｩ繝ｳ繝翫・縺ｫ蜷後§隕丞援縺ｧ驕ｩ逕ｨ縺吶ｋ縲・
- 螳溯｣・
  - `nodesrc/parser.js`
    - doctest 繝｡繧ｿ繝・・繧ｿ縺ｨ縺励※ `stdin/stdout/stderr` 繧呈歓蜃ｺ縺吶ｋ讖溯・繧定ｿｽ蜉�縲・
    - 譁・ｭ怜・蛟､縺ｯ JSON 譁・ｭ怜・・・"..."`・峨→縺励※隗｣驥医＠縲～\n` 遲峨・繧ｨ繧ｹ繧ｱ繝ｼ繝励ｒ螻暮幕縲・
  - `nodesrc/tests.js`
    - LLVM runner 繧偵�慶ompile遒ｺ隱阪・縺ｿ縲阪°繧峨�形nepl-cli --target llvm` -> `clang` link -> 螳溯｡後�阪∈諡｡蠑ｵ縲・
    - doctest 譛溷ｾ・�､蛻､螳壹ｒ蜈ｱ騾壼喧縺励�仝ASM/LLVM 荳｡邨先棡縺ｸ蜷御ｸ�繝ｭ繧ｸ繝・け繧帝←逕ｨ縲・
    - `--runner all` 譎ゅ↓ `compare_wasm_llvm` 繝輔ぉ繝ｼ繧ｺ繧定ｿｽ蜉�・・tdout/stderr 縺ｮ荳�閾ｴ遒ｺ隱搾ｼ峨�・
    - 霑ｽ蜉�繧ｪ繝励す繝ｧ繝ｳ:
      - `--assert-io`: `stdin/stdout/stderr` 縺ｮ蜴ｳ蟇・ｯ碑ｼ・ｒ譛牙柑蛹・
      - `--strict-dual`: wasm/llvm 縺ｮ豈碑ｼ・ｵ先棡繧貞ｿ・�亥喧・域ｯ碑ｼ・ｬ�關ｽ繧・fail・・
    - 莠呈鋤邯ｭ謖・
      - 譌｢蟄倬°逕ｨ繧貞｣翫＆縺ｪ縺・◆繧√�∝宍蟇・I/O 豈碑ｼ・・ `--assert-io` 謖・ｮ壽凾縺ｮ縺ｿ譛牙柑蛹悶�・
  - `nepl-core/src/codegen_llvm.rs`
    - entry lower 縺ｮ螟ｱ謨励ｒ謠｡繧翫▽縺ｶ縺輔★縲～compile_llvm_cli` 縺ｧ蜴溷屏繧定ｿ斐☆繧医≧菫ｮ豁｣縲・
    - entry 蜷阪・隗｣豎ｺ縺ｧ mangled 蜷搾ｼ・main__...` 蠖｢蠑擾ｼ峨ｒ霑ｽ霍｡縺吶ｋ fallback 繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - 譌｢蟄倅ｺ呈鋤繝｢繝ｼ繝・
    - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 2`
    - `610/610 pass`
  - 螳悟・讀懆ｨｼ繝｢繝ｼ繝会ｼ井ｾ具ｼ・
    - `node nodesrc/tests.js -i tests/stdout.n.md -o tests/output/stdout_complete.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `compare_wasm_llvm` 縺檎ｵ先棡JSON縺ｫ蜃ｺ蜉帙＆繧後�『asm/llvm 蟾ｮ蛻・ｒ蜿ｯ隕門喧縺ｧ縺阪ｋ縺薙→繧堤｢ｺ隱阪�・
- 迴ｾ蝨ｨ蛻､譏弱＠縺ｦ縺・ｋ譬ｹ譛ｬ隱ｲ鬘・
  - LLVM 蛛ｴ縺ｯ `main` 隗｣豎ｺ縺ｫ騾ｲ繧�繧医≧縺ｫ縺ｪ縺｣縺溘′縲～core/math` 縺ｮ wasm 蟆ら畑髢｢謨ｰ・井ｾ・ `add__i32_i32__i32__pure`・峨↓蛻ｰ驕斐☆繧九→ `compile_llvm_cli` 縺ｧ螟ｱ謨励☆繧九�・
  - 縺薙ｌ縺ｯ縲悟ｮ悟・讀懆ｨｼ繝｢繝ｼ繝峨・荳榊・蜷医�阪〒縺ｯ縺ｪ縺上�～stdlib` 蛛ｴ縺ｮ llvm 螳溯｣・悴謨ｴ蛯吶′蜴溷屏縺ｧ縺ゅｊ縲∽ｸ頑ｵ∬ｪｲ鬘後→縺励※邯咏ｶ壻ｿｮ豁｣縺吶ｋ縲・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (LLVM lower 蠑ｷ蛹悶→ llvm runner 謾ｹ菫ｮ)
- 逶ｮ逧・
  - `llvm` 繝ｩ繝ｳ繝翫・縺ｮ螟ｱ謨励ｒ荳頑ｵ・ｼ・nepl-core/src/codegen_llvm.rs`・峨°繧牙炎貂帙☆繧九�・
  - `wasm` 譌｢蟄倥ユ繧ｹ繝医ｒ螢翫＆縺壹�～llvm` 蛛ｴ縺ｮ螟ｱ謨励ｒ compile/link 荳ｭ蠢・°繧・run/螳溯｣・ｸ崎ｶｳ縺ｸ蟇・○繧九�・
- 螳溯｣・
  - `nepl-core/src/codegen_llvm.rs`
    - `lower_hir_string_literal` 縺ｮ `alloc/store_i32/store_u8` 繧偵す繧ｰ繝阪メ繝｣隗｣豎ｺ (`resolve_symbol_name`) 縺ｫ螟画峩縲・
    - `EnumConstruct` 縺ｧ繧・`alloc` 繧偵す繧ｰ繝阪メ繝｣隗｣豎ｺ縺ｸ螟画峩縲・
    - `StructConstruct` / `TupleConstruct` 縺ｮ lower 繧定ｿｽ蜉�・医ヲ繝ｼ繝礼｢ｺ菫・+ 繝輔ぅ繝ｼ繝ｫ繝蛾�先ｬ｡ store・峨�・
    - intrinsic lower 繧定ｿｽ蜉�:
      - `add`
      - `f32_to_i32`
      - `i32_to_u8`
    - `if` 縺ｮ蜀榊ｮ夂ｾｩ謚大宛縺ｾ繧上ｊ繧堤ｶ咏ｶ夊｣懈ｭ｣:
      - `RawBodySelection::Llvm` 縺ｧ蛻晏屓襍ｰ譟ｻ譎ゅ↓螳夂ｾｩ髢｢謨ｰ蜷阪ｒ `emitted_functions` 縺ｸ逋ｻ骭ｲ縲・
      - `parse_defined_function_name` 縺ｧ `define @"name"(...)` 縺ｮ蠑慕畑隨ｦ繧呈ｭ｣隕丞喧縲・
      - `HirBody::LlvmIr` 縺ｮ縲悟ｮ夂ｾｩ貂医∩謇ｱ縺・�肴擅莉ｶ繧貞宍蟇・喧縺励�〉aw 縺・`@add` 縺ｮ縺ｿ螳夂ｾｩ縺吶ｋ蝣ｴ蜷医↓ `add__...` 繧定ｪ､縺｣縺ｦ螳夂ｾｩ貂医∩縺ｫ縺励↑縺・ｈ縺・ｿｮ豁｣縲・
    - raw 螳夂ｾｩ縺ｮ base 蜷阪＠縺狗┌縺・こ繝ｼ繧ｹ蜷代￠縺ｫ mangled alias wrapper 逕滓・繧定ｿｽ蜉�・・add__... -> add` 遲会ｼ峨�・
  - `nodesrc/tests.js`
    - LLVM 繝ｪ繝ｳ繧ｯ譎ゅ↓ `-lm` 繧定ｿｽ蜉�・・ceilf/floorf/truncf/nearbyintf` 遲峨・譛ｪ隗｣豎ｺ繧定ｧ｣豸茨ｼ峨�・
  - `stdlib/alloc/string.nepl`
    - `str_eq_loop` / `str_eq_at` 縺ｮ蠑墓焚 `len` 繧・`n` 縺ｫ螟画峩縺励�・未謨ｰ繧ｷ繝ｳ繝懊Ν `len` 縺ｨ縺ｮ隗｣豎ｺ陦晉ｪ√ｒ蝗樣∩縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`: 謌仙粥
  - `node nodesrc/tests.js -i tests -o tests/output/tests_current.json -j 2`: `610/610 pass`
  - `node nodesrc/tests.js -i tests -o tests/output/tests_llvm_current.json -j 2 --runner llvm --llvm-all --assert-io`: `397/601 pass`
- 迥ｶ豕∵紛逅・
  - 逶ｴ霑代〒 `llvm` 縺ｯ `link_llvm_cli` 縺ｮ螟ｧ驥丞､ｱ謨暦ｼ域悴螳夂ｾｩ繧ｷ繝ｳ繝懊Ν/`libm` 譛ｪ繝ｪ繝ｳ繧ｯ・峨ｒ蜑頑ｸ帙�・
  - 迴ｾ蝨ｨ縺ｮ荳ｻ螟ｱ謨励・ `run_llvm_cli(SIGSEGV)` 縺ｨ縲∽ｸ�驛ｨ縺ｮ `compile_llvm_cli`・亥梛蜉ｹ譫・蜷榊燕隗｣豎ｺ逕ｱ譚･・峨↓髮・ｴ・�・
  - 谺｡谿ｵ縺ｯ `core/mem` 縺ｨ `alloc/*` 縺ｮ繝ｩ繝ｳ繧ｿ繧､繝�謨ｴ蜷茨ｼ育ｷ壼ｽ｢繝｡繝｢繝ｪ驕狗畑・峨ｒ蜆ｪ蜈医＠縺ｦ騾ｲ繧√ｋ縲・

# 2026-02-22 菴懈･ｭ繝｡繝｢ (LLVM 蛻ｰ驕碑ｧ｣譫・alias 菫ｮ豁｣縺ｮ邯咏ｶ・
- 逶ｮ逧・
  - `link_llvm_cli` 縺ｮ譛ｪ螳夂ｾｩ繧ｷ繝ｳ繝懊Ν繧剃ｸ頑ｵ・ｼ・codegen_llvm`・峨〒蜑頑ｸ帙☆繧九�・
  - `#llvmir` 髢｢謨ｰ縺ｮ raw 螳夂ｾｩ蜷阪→ mangled 蜻ｼ縺ｳ蜃ｺ縺怜錐縺ｮ荳堺ｸ�閾ｴ繧貞精蜿弱☆繧九�・
- 螳溯｣・
  - `nepl-core/src/codegen_llvm.rs`
    - mangled 蜷阪・ base 謚ｽ蜃ｺ繧剃ｿｮ豁｣・亥・鬆ｭ `__` 繧貞性繧�髢｢謨ｰ蜷阪ｒ豁｣縺励￥謇ｱ縺・ｼ峨�・
    - raw `#llvmir` 髢｢謨ｰ縺ｧ縲罫aw 縺ｯ base 蜷阪・縺ｿ螳夂ｾｩ縲阪・蝣ｴ蜷医↓縲［angled 蜷阪∈縺ｮ wrapper 繧定・蜍慕函謌舌�・
    - `HirBody::LlvmIr` 縺ｮ `call @...` 繧貞芦驕碑ｧ｣譫舌∈霑ｽ蜉�縺励�〉aw 蜀・Κ縺ｮ萓晏ｭ倬未謨ｰ繧・reachable 縺ｫ蜷ｫ繧√ｋ縲・
    - `llvm_output_has_function` 繧・`define/declare` 陦後・縺ｿ蛻､螳壹☆繧九ｈ縺・ｿｮ豁｣・・call` 陦瑚ｪ､讀懃衍繧帝勁蜴ｻ・峨�・
  - `todo.md`
    - wasm/llvm 蜈ｱ騾壹・縲梧悴蛻ｰ驕秘未謨ｰ繧貞・蜉帙＠縺ｪ縺・ｼ磯未謨ｰ蜊倅ｽ・tree-shaking・峨�阪ち繧ｹ繧ｯ繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/list.nepl ... --runner llvm --llvm-all --assert-io`
    - 螟画峩蜑・ `104/200 pass`
    - 螟画峩蠕・ `195/200 pass`
  - 谿倶ｻｶ・亥酔繧ｳ繝槭Φ繝会ｼ・
    - `__nepl_syscall` 譛ｪ螳夂ｾｩ 2莉ｶ
    - `unknown variable 'inc__i32__i32__pure'` 2莉ｶ
    - `kpdsu` 縺ｮ螳溯｡悟・蜉帛ｷｮ蛻・1莉ｶ

# 2026-02-26 菴懈･ｭ繝｡繝｢ (stdlib doctest target 縺ｮ core/std 蛹・
- 逶ｮ逧・
  - LLVM dual-run 縺ｧ菴ｿ逕ｨ縺吶ｋ doctest 縺ｮ target 陦ｨ險倥ｒ邨ｱ荳�縺吶ｋ縺溘ａ縲～stdlib/*.nepl` 蜀・・ doctest 蝓九ａ霎ｼ縺ｿ繧ｽ繝ｼ繧ｹ縺ｮ縺ｿ繧・`#target core/std` 縺ｸ遘ｻ陦後☆繧九�・
  - 螳溯｣・さ繝ｼ繝牙・縺ｮ `#target`・医Δ繧ｸ繝･繝ｼ繝ｫ譛ｬ菴難ｼ峨・螟画峩縺帙★縲√ユ繧ｹ繝医こ繝ｼ繧ｹ螳夂ｾｩ縺�縺代ｒ譖ｴ譁ｰ縺吶ｋ縲・
- 螳溯｣・
  - `stdlib/**/*.nepl` 縺ｮ `//:| #target wasi` 繧・`//:| #target std` 縺ｫ螟画峩縲・
  - `stdlib/**/*.nepl` 縺ｮ `//:| #target wasm` 繧・`//:| #target core` 縺ｫ螟画峩縲・
  - 螳溘さ繝ｼ繝芽｡鯉ｼ・#target wasi` 縺ｪ縺ｩ・峨・譛ｪ螟画峩縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` 縺ｯ謌仙粥縲・
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` 螳溯｡檎ｵ先棡:
    - `total=1781, passed=1205, failed=576, errored=0`
    - 螟ｱ謨励・莉｣陦ｨ縺ｯ `tests/kp.n.md` / `tests/string.n.md` 縺ｮ wasm/llvm 螳溯｡悟ｷｮ蛻・ｼ・tdout mismatch・峨〒縲∽ｻ雁屓縺ｮ target 陦ｨ險伜､画峩縺ｫ繧医ｋ譁ｰ隕丞､ｱ謨励・遒ｺ隱阪〒縺阪↑縺・ｼ井ｻｶ謨ｰ縺梧里遏･蛟､縺ｨ荳�閾ｴ・峨�・
- 陬懆ｶｳ:
  - `tests/*.n.md` 縺ｯ譌｢縺ｫ `core/std` 蛹匁ｸ医∩縺ｧ縺ゅｋ縺薙→繧貞・遒ｺ隱阪＠縺溘�・

# 2026-02-26 菴懈･ｭ繝｡繝｢ (繝・せ繝亥渕逶､繝ｻ譁・ｭ怜・繝・せ繝医・謨ｴ蜷井ｿｮ豁｣)
- 逶ｮ逧・
  - `tests + stdlib` 縺ｮ dual 螳溯｡後〒螟ｧ驥丞､ｱ謨励＠縺ｦ縺・◆蜴溷屏繧偵�√ユ繧ｹ繝医ヤ繝ｼ繝ｫ蝠城｡後・繝・せ繝医こ繝ｼ繧ｹ蝠城｡後・繧ｳ繝ｳ繝代う繝ｩ蝠城｡後↓蛻・ｧ｣縺励※譏ｯ豁｣縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏縺ｨ菫ｮ豁｣:
  - `nodesrc/tests.js`
    - `::llvm` 繧ｵ繝輔ぅ繝・け繧ｹ髯､蜴ｻ髟ｷ縺瑚ｪ､縺｣縺ｦ縺翫ｊ縲～compare_wasm_llvm` 縺瑚ｪ､縺｣縺ｦ `missing llvm counterpart result` 繧堤函謌舌＠縺ｦ縺・◆縲・
      - 菫ｮ豁｣: `stripLlvmSuffix` 繧・`-6` 縺ｫ險よｭ｣縲・
    - `strictDual` 豈碑ｼ・〒 `wasi_only/skip_llvm/wasm_only` 繧ｱ繝ｼ繧ｹ縺ｾ縺ｧ豈碑ｼ・ｯｾ雎｡縺ｫ蜈･縺｣縺ｦ縺・◆縲・
      - 菫ｮ豁｣: `compareWasmLlvmResults` 縺ｧ `skipOnLlvmRunner` 繧帝←逕ｨ縺玲ｯ碑ｼ・ｯｾ雎｡螟門喧縲・
  - `tests/kp.n.md`
    - `kpsearch_unique_and_count` 縺ｮ譛溷ｾ・�､縺後ョ繝ｼ繧ｿ蜀・ｮｹ縺ｨ髢｢謨ｰ莉墓ｧ假ｼ・count_equal_range_i32`・峨↓蟇ｾ縺励※荳肴紛蜷医□縺｣縺溘�・
      - 菫ｮ豁｣: `"3 3\n1 2 5\n"` -> `"2 3\n1 2 5\n"`縲・
  - `tests/string.n.md`
    - `stdout:` 繝｡繧ｿ蛟､縺ｫ `\\n` 繧剃ｽｿ縺｣縺ｦ縺翫ｊ縲゛SON譁・ｭ怜・縺ｨ縺励※縺ｯ縲梧隼陦後�阪〒縺ｯ縺ｪ縺上�後ヰ繝・け繧ｹ繝ｩ繝・す繝･+n縲肴悄蠕・↓縺ｪ縺｣縺ｦ縺・◆縲・
    - 蜊倩｡梧枚蟄怜・繧ｨ繧ｹ繧ｱ繝ｼ繝玲､懆ｨｼ縺ｮ繧ｽ繝ｼ繧ｹ蛛ｴ繧・`"...\\n..."` 縺ｫ縺ｪ縺｣縺ｦ縺翫ｊ縲√ユ繧ｹ繝域э蝗ｳ・医お繧ｹ繧ｱ繝ｼ繝苓ｧ｣驥茨ｼ峨→荳堺ｸ�閾ｴ縺�縺｣縺溘�・
      - 菫ｮ豁｣: `stdout:` 縺ｨ繧ｽ繝ｼ繧ｹ譁・ｭ怜・繧偵�∵э蝗ｳ縺ｩ縺翫ｊ `\n`/`\t` 縺悟宛蠕｡譁・ｭ励→縺励※隧穂ｾ｡縺輔ｌ繧句ｽ｢縺ｸ譖ｴ譁ｰ縲・
  - `nepl-core/src/lexer.rs`
    - `mlstr` 縺ｮ `##:` 陦後〒蜈磯�ｭ1繧ｹ繝壹・繧ｹ繧呈悽譁・∈蜿悶ｊ霎ｼ繧薙〒縺・◆縺溘ａ縲∽ｻ墓ｧ假ｼ・##: ` 縺ｮ蠕後ｍ縺梧悽譁・ｼ峨→荳堺ｸ�閾ｴ縲・
      - 菫ｮ豁｣: `##:` 逶ｴ蠕後・蜈磯�ｭ1繧ｹ繝壹・繧ｹ繧帝勁蜴ｻ縺吶ｋ繧医≧縺ｫ隱ｿ謨ｴ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` 謌仙粥縲・
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-final-before-commit.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    - `total=1579, passed=1579, failed=0, errored=0`縲・

# 2026-02-26 菴懈･ｭ繝｡繝｢ (dual-run 蜈ｨ騾壹→繝・せ繝亥渕逶､蜀咲｢ｺ隱・
- 逶ｮ逧・
  - 繝・せ繝医こ繝ｼ繧ｹ縺ｨ繝・せ繝医ヤ繝ｼ繝ｫ縺ｮ螯･蠖捺�ｧ繧貞・縺ｫ諡・ｿ昴＠縲√さ繝ｳ繝代う繝ｩ螳溯｣・ｿｮ豁｣縺ｸ騾ｲ繧√ｋ蜑肴署繧貞崋繧√ｋ縲・
- 螳滓命:
  - `nodesrc/tests.js` 縺ｮ wasm/llvm 蟇ｾ蠢應ｻ倥￠縺ｨ strict-dual 豈碑ｼ・ｯｾ雎｡縺ｮ謇ｱ縺・ｒ菫ｮ豁｣縲・
  - `tests/kp.n.md` 縺ｮ隱､譛溷ｾ・�､繧剃ｻ墓ｧ倥↓蜷医ｏ縺帙※菫ｮ豁｣縲・
  - `tests/string.n.md` 縺ｮ蜊倩｡梧枚蟄怜・繧ｨ繧ｹ繧ｱ繝ｼ繝玲､懆ｨｼ縺ｨ `stdout:` 繝｡繧ｿ陦ｨ險倥ｒ謨ｴ蜷亥喧縲・
  - `nepl-core/src/lexer.rs` 縺ｮ `mlstr` 陦碁�ｭ繧ｹ繝壹・繧ｹ蜿悶ｊ霎ｼ縺ｿ荳肴紛蜷医ｒ菫ｮ豁｣縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-final-now.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
  - 邨先棡: `total=1579, passed=1579, failed=0, errored=0`
- 蛻､譁ｭ:
  - 迴ｾ譎らせ縺ｧ谿九ｋ螟ｱ謨励・縺ｪ縺上�√ユ繧ｹ繝亥渕逶､/繝・せ繝医こ繝ｼ繧ｹ/繧ｳ繝ｳ繝代う繝ｩ螳溯｣・・縺薙・遽・峇縺ｮ荳肴紛蜷医・隗｣豸域ｸ医∩縲・

# 2026-02-26 菴懈･ｭ繝｡繝｢ (wasm codegen 蛻ｰ驕碑ｧ｣譫舌・霑ｽ蜉�)
- 逶ｮ逧・
  - import 縺励◆縺�縺代〒譛ｪ菴ｿ逕ｨ髢｢謨ｰ縺ｾ縺ｧ wasm 蜃ｺ蜉帙＆繧後ｋ迥ｶ諷九ｒ謾ｹ蝟・＠縲‘ntry 縺九ｉ蛻ｰ驕斐☆繧矩未謨ｰ縺ｮ縺ｿ繧貞・蜉帙☆繧九�・
- 螳溯｣・
  - `nepl-core/src/codegen_wasm.rs`
    - `collect_reachable_wasm_functions` 繧定ｿｽ蜉�縺励�‘ntry 襍ｷ轤ｹ縺ｮ髢｢謨ｰ蛻ｰ驕秘寔蜷医ｒ讒狗ｯ峨�・
    - `collect_called_functions_from_expr` 繧定ｿｽ蜉�縺励�～Call(User)` 縺ｨ髢｢謨ｰ蛟､蜿ら・・・Var`/`FnValue`・峨ｒ霑ｽ霍｡蟇ｾ雎｡縺ｫ縺励◆縲・
    - `call_indirect` 縺悟性縺ｾ繧後ｋ蝣ｴ蜷医・縲・撕逧・｢ｺ螳壻ｸ崎・縺ｮ縺溘ａ菫晏ｮ育噪縺ｫ蜈ｨ髢｢謨ｰ菫晄戟縺ｸ繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ縲・
    - user 髢｢謨ｰ縺ｮ lower 蟇ｾ雎｡繧貞芦驕秘寔蜷医〒繝輔ぅ繝ｫ繧ｿ繝ｪ繝ｳ繧ｰ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-reachability-3.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
  - 邨先棡: `total=1579, passed=1579, failed=0, errored=0`
- 陬懆ｶｳ:
  - 螳溯｣・�比ｸｭ縺ｧ `Var/FnValue` 蜿ら・譛ｪ霑ｽ霍｡縺ｫ繧医ｊ `len__str__i32__pure` 譛ｪ螳夂ｾｩ縺檎匱逕溘＠縺溘′縲∝盾辣ｧ霑ｽ霍｡霑ｽ蜉�縺ｧ隗｣豸医＠縺溘�・
## 2026-02-27 菴懈･ｭ繝｡繝｢ (LLVM codegen 縺ｮ target gate 蛻､螳壹ｒ compiler 縺ｨ邨ｱ荳�)
- 逶ｮ逧・
  - `#if[target=...]` 縺ｮ蠑剰ｩ穂ｾ｡繧偵�´LVM codegen 蛛ｴ縺ｧ繧・`compiler` 縺ｨ蜷御ｸ�螳溯｣・〒蛻､螳壹☆繧九�・
  - target 蛻､螳壹・莠碁㍾螳溯｣・↓繧医ｋ蟆・擂縺ｮ荵夜屬繧帝亟縺舌�・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - `gate_allows` 縺ｮ `Directive::IfTarget` 蛻・ｲ舌ｒ `target.allows(...)` 縺九ｉ
      `crate::compiler::target_gate_allows_expr(...)` 蜻ｼ縺ｳ蜃ｺ縺励∈螟画峩縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false timeout 900s node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-continue.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1588/1588 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (`sort_*_ret` 縺ｮ move-check 譬ｹ譛ｬ菫ｮ豁｣)
- 逶ｮ逧・
  - `todo.md` 3逡ｪ縺ｮ `sort` 縺ｾ繧上ｊ縺ｧ縲〃ec 繧定ｿ斐☆繝ｩ繝・ヱAPI繧・move 隕丞援縺ｫ謨ｴ蜷医＆縺帙ｋ縲・
- 蜴溷屏:
  - `sort_quick_ret` / `sort_heap_ret` / `sort_merge_ret` 縺ｧ `v` 縺九ｉ `get` 繧定｡後▲縺溷ｾ後↓ `v` 繧偵◎縺ｮ縺ｾ縺ｾ霑斐＠縺ｦ縺翫ｊ縲［ove-check 縺ｧ `use of moved value: v` 縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
  - 螟ｱ謨励・ `tests/sort.n.md` 縺ｮ譁ｰ隕上こ繝ｼ繧ｹ縺ｧ蜀咲樟縺励�∬ｨｺ譁ｭ菴咲ｽｮ繧ょ酔荳�縲・
- 菫ｮ豁｣:
  - `stdlib/alloc/sort.nepl`
    - `sort_*_ret` 縺ｧ `len/cap/data` 繧貞叙蠕怜ｾ後�∬ｿ斐ｊ蛟､繧・`v` 縺ｧ縺ｯ縺ｪ縺・`Vec<.T> n cap data_ptr` 縺ｮ蜀肴ｧ狗ｯ峨∈螟画峩縲・
  - `tests/sort.n.md`
    - 譁ｰ隕・`sort_*_ret` 讀懆ｨｼ繧ｱ繝ｼ繧ｹ縺ｮ隱ｭ縺ｿ蜿悶ｊ繧・`vec_get` 騾｣邯壼他縺ｳ蜃ｺ縺励°繧峨�～vec_data_ptr + load_i32` 縺ｫ螟画峩縲・
    - 縺薙ｌ縺ｫ繧医ｊ縲～vec_get` 縺・`Vec` 繧呈ｶ郁ｲｻ縺吶ｋ迴ｾ蝨ｨ莉墓ｧ倥〒繧ょ腰荳�蛟､ `v` 繧剃ｽｿ縺・屓縺輔★縺ｫ讀懆ｨｼ蜿ｯ閭ｽ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-returning-api-v6.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `499/499 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-sort-ret-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1620/1620 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (`Vec` read-only 邨瑚ｷｯ縺ｮ谿ｵ髫主ｰ主・)
- 逶ｮ逧・
  - `todo.md` 3逡ｪ縺ｮ `Vec` 隱ｭ縺ｿ蜿悶ｊ險ｭ險医ｒ蜑埼�ｲ縺輔○縲～sort` 讀懆ｨｼ繧ｳ繝ｼ繝峨〒 move 隕丞援縺ｫ蠑輔▲縺九°繧峨↑縺・read-only 繝代ち繝ｼ繝ｳ繧呈ｨ呎ｺ門喧縺吶ｋ縲・
- 螳溯｣・
  - `stdlib/alloc/vec.nepl`
    - `vec_data_len <.T> <(Vec<.T>)->.Pair>` 繧定ｿｽ蜉�縲・
    - 霑斐ｊ蛟､縺ｯ `Tuple:` 縺ｧ `(data_ptr, len)`縲・
    - 譌･譛ｬ隱槭ラ繧ｭ繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝医→ doctest 繧定ｿｽ蜉�縲・
  - `tests/sort.n.md`
    - `sort_quick_ret_i32_sorted_values`
    - `sort_heap_ret_i32_sorted_values`
    - `sort_merge_ret_i32_sorted_values`
    繧・`vec_data_ptr` 逶ｴ謗･蜿ら・縺九ｉ `vec_data_len + core/field.get` 繝吶・繧ｹ縺ｫ譖ｴ譁ｰ縲・
    - `len == 4` 縺ｮ讀懆ｨｼ繧りｿｽ蜉�縺励�√ョ繝ｼ繧ｿ謨ｴ蜷医→髟ｷ縺墓紛蜷医ｒ蜷梧凾縺ｫ遒ｺ隱阪�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-vec-data-len-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `502/502 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-vec-data-len-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1623/1623 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (`noshadow` 驕ｩ逕ｨ遽・峇縺ｮ stdlib 諡｡螟ｧ: stdio)
- 逶ｮ逧・
  - `todo.md` 縺ｮ繧ｷ繝｣繝峨・繧､繝ｳ繧ｰ驕狗畑繧貞ｮ御ｺ・＆縺帙ｋ縺溘ａ縲～std/test` 縺ｫ邯壹＞縺ｦ `std/stdio` 縺ｮ蝓ｺ蟷ｹAPI縺ｫ繧・`noshadow` 繧帝←逕ｨ縺吶ｋ縲・
- 螳溯｣・
  - `stdlib/std/stdio.nepl`
    - `print`
    - `read_line`
    - `println`
    - `print_i32`
    - `println_i32`
    繧・`fn noshadow` 蛹悶�・
  - `tests/shadowing.n.md`
    - `std_stdio_noshadow_same_signature_redefinition_is_error`・・ompile_fail・峨ｒ霑ｽ蜉�縲・
    - `std_stdio_noshadow_allows_overload_with_different_signature`・域・蜉滂ｼ峨ｒ霑ｽ蜉�縲・
- 螟ｱ謨怜・譫・
  - 蛻晏屓縺ｯ `print <(i32)*>()>` 繧・overloading 縺吶ｋ繝・せ繝医↓縺励�～stdio` 蜀・Κ縺ｮ `print` 蜻ｼ縺ｳ蜃ｺ縺励′譖匁乂蛹悶＠縺ｦ螟ｧ驥・`ambiguous overload` 繧定ｪ倡匱縲・
  - 縺薙ｌ縺ｯ繝・せ繝郁ｨｭ險医Α繧ｹ縺ｨ蛻､譁ｭ縺励�∝・驛ｨ蜻ｼ縺ｳ蜃ｺ縺励↓蠖ｱ髻ｿ縺励↑縺・`read_line` 縺ｮ蛻･繧ｷ繧ｰ繝阪メ繝｣ overloading 縺ｸ螟画峩縺励※隗｣豸医�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/shadowing.n.md -o /tmp/tests-shadowing-stdio-noshadow-v2.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `538/538 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-stdio-noshadow-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1628/1628 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (`sort_*_ret` 蠅・阜蝗槫ｸｰ縺ｮ蠑ｷ蛹・
- 逶ｮ逧・
  - `sort_*_ret` API 縺ｮ move 隕丞援謨ｴ蜷医ｒ邯ｭ謖√☆繧九◆繧√�∵綾繧雁�､Vec API縺ｫ蟇ｾ縺吶ｋ `len=0/1` 蠅・阜繧ｱ繝ｼ繧ｹ繧貞崋螳壹☆繧九�・
- 螟画峩:
  - `tests/sort.n.md` 縺ｫ莉･荳九ｒ霑ｽ蜉�:
    - `sort_quick_ret_len0_noop`
    - `sort_quick_ret_len1_noop`
    - `sort_heap_ret_len0_noop`
    - `sort_heap_ret_len1_noop`
    - `sort_merge_ret_len0_noop`
    - `sort_merge_ret_len1_noop`
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-ret-boundary-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `520/520 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-sort-ret-boundary-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1646/1646 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (`sort_*_ret` API 謨ｴ蜷医・螳御ｺ・
- 逶ｮ逧・
  - `todo.md` 縺ｮ sort/move 隕丞援謨ｴ蜷磯�・岼繧貞ｮ御ｺ・〒縺阪ｋ迥ｶ諷九↓縺吶ｋ縲・
- 螳溯｣・
  - `tests/sort.n.md` 縺ｫ `sort_*_ret` 縺ｮ霑泌唆蠕悟・蛻ｩ逕ｨ繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�:
    - `sort_quick_ret_vec_is_reusable_after_sort`
    - `sort_heap_ret_vec_is_reusable_after_sort`
    - `sort_merge_ret_vec_is_reusable_after_sort`
  - 縺・★繧後ｂ縲茎ort 蠕後↓ `vec_push` 縺ｧ縺阪ｋ縺薙→縲阪→ `vec_data_len` 縺ｧ `len` 縺悟｢励∴繧九％縺ｨ繧呈､懆ｨｼ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests/sort.n.md -o /tmp/tests-sort-ret-reuse-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `529/529 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-sort-ret-reuse-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
- todo 謨ｴ逅・
  - `todo.md` 縺ｮ `sort/generics 縺ｨ Vec 隱ｭ縺ｿ蜿悶ｊ險ｭ險・ 繧貞ｮ御ｺ・→縺励※蜑企勁縺励�∵ｮ矩�・岼縺ｮ逡ｪ蜿ｷ繧定ｩｰ繧√◆縲・

# 2026-02-27 菴懈･ｭ繝｡繝｢ (LSP/API phase2: token_resolution 縺ｫ螳夂ｾｩ繧ｪ繝悶ず繧ｧ繧ｯ繝医ｒ邨ｱ蜷・
- 逶ｮ逧・
  - `todo.md` 2逡ｪ・・SP/API 諡｡蠑ｵ・峨・荳�驛ｨ縺ｨ縺励※縲》oken 蜊倅ｽ肴ュ蝣ｱ縺九ｉ逶ｴ謗･縲悟ｮ夂ｾｩ繧ｸ繝｣繝ｳ繝怜庄閭ｽ縺ｪ諠・�ｱ縲阪ｒ蜿門ｾ励〒縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
- 螳溯｣・
  - `nepl-web/src/lib.rs` 縺ｮ `analyze_semantics` 縺ｧ縲～token_resolution` 蜷・ｦ∫ｴ�縺ｫ莉･荳九ｒ霑ｽ蜉�:
    - `resolved_definition`・・d/name/kind/scope_depth/span・・
    - `candidate_definitions`・亥�呵｣懷ｮ夂ｾｩ驟榊・縲∝推隕∫ｴ�縺ｫ span 蜷ｫ繧�・・
  - 蠕捺擂縺ｮ `resolved_def_id` / `candidate_def_ids` 縺ｯ蠕梧婿莠呈鋤縺ｨ縺励※邯ｭ謖√�・
- 繝・せ繝・
  - `tests/tree/04_semantics_tree.js` 繧呈峩譁ｰ縺励�・
    - `resolved_definition.span` 縺ｮ蟄伜惠
    - `candidate_definitions` 縺碁・蛻励〒縺ゅｋ縺薙→
    繧呈､懆ｨｼ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `15/15 pass`
  - `PATH=/opt/llvm-21.1.0/bin:$PATH NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-after-token-resolution-defobj-v1.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
# 2026-02-27 菴懈･ｭ繝｡繝｢ (LSP/API phase2: VFS霍ｨ縺主ｮ夂ｾｩ繧ｸ繝｣繝ｳ繝玲ュ蝣ｱ縺ｮ蝗ｺ螳・
- 逶ｮ逧・
  - `todo.md` 2逡ｪ・・SP/API 諡｡蠑ｵ phase 2・峨・縺・■縲》oken 隗｣豎ｺ邨先棡縺ｫ import 蜈亥ｮ夂ｾｩ縺ｮ繝輔ぃ繧､繝ｫ諠・�ｱ繧定ｿ斐☆驛ｨ蛻・ｒ螳牙ｮ壼喧縺吶ｋ縲・
- 螳溯｣・
  - `nepl-web/src/lib.rs`
    - `span_to_js_with_map` 繧貞ｰ主・縺励�～SourceMap` 縺後≠繧句�ｴ蜷医・ span 縺ｮ line/col 繧貞・繝輔ぃ繧､繝ｫ蝓ｺ貅悶〒險育ｮ励＠縲～file_path` 繧貞沂繧√ｋ繧医≧縺ｫ螟画峩縲・
    - 蜷榊燕隗｣豎ｺ payload 螟画鋤髢｢謨ｰ・・def_trace_to_js` / `ref_trace_to_js` / `shadow_trace_to_js` / `name_resolution_payload_to_js`・峨↓ `SourceMap` 繧呈ｸ｡縺帙ｋ蠖｢縺ｸ諡｡蠑ｵ縲・
    - `analyze_semantics_with_vfs(entry_path, source, vfs)` 繧定ｿｽ蜉�縺励�〃FS 隱ｭ縺ｿ霎ｼ縺ｿ譎ゅ・ `token_resolution` 縺ｫ
      - `resolved_definition`・・pan + file_path・・
      - `candidate_definitions`・磯・蛻励�∝推隕∫ｴ�縺ｫ span + file_path・・
      繧定ｿ斐☆繧医≧縺ｫ螳溯｣・�・
  - `tests/tree/16_semantics_vfs_cross_file.js` 繧定ｿｽ蜉�縲・
    - `core/math` 縺ｮ `add` 蜻ｼ縺ｳ蜃ｺ縺励〒縲∬ｧ｣豎ｺ蜈医′ `/stdlib/core/math.nepl` 繧呈欠縺吶％縺ｨ繧呈､懆ｨｼ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `16/16 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
- todo蜿肴丐:
  - `todo.md` 2逡ｪ縺九ｉ縲荊oken 蜊倅ｽ阪・蝙区ュ蝣ｱ API 縺ｫ螳夂ｾｩ繧ｸ繝｣繝ｳ繝玲ュ蝣ｱ・・mport 蜈亥性繧�・峨ｒ邨ｱ蜷医☆繧九�阪ｒ蜑企勁・亥ｮ御ｺ・ｼ峨�・
# 2026-02-27 菴懈･ｭ繝｡繝｢ (LSP/API phase2: name_resolution 縺ｮ VFS 迚医ｒ霑ｽ蜉�)
- 逶ｮ逧・
  - `todo.md` 2逡ｪ縺ｮ谿倶ｻｶ縺�縺｣縺溘�形analyze_name_resolution` 縺ｮ import/alias/use 霍ｨ縺主ｮ夂ｾｩ蜈・ュ蝣ｱ縲阪ｒ API 縺ｧ霑斐○繧九ｈ縺・↓縺吶ｋ縲・
- 螳溯｣・
  - `nepl-web/src/lib.rs`
    - `analyze_name_resolution_with_vfs(entry_path, source, vfs, options)` 繧定ｿｽ蜉�縲・
    - `Loader + SourceMap` 邨檎罰縺ｧ隍・焚繝輔ぃ繧､繝ｫ繧定ｪｭ縺ｿ霎ｼ縺ｿ縲～name_resolution_payload_to_js(..., Some(&source_map), ...)` 繧剃ｽｿ縺｣縺ｦ
      螳夂ｾｩ繝ｻ蜿ら・繝ｻshadow 縺ｮ `span.file_path` 繧定ｿ斐☆繧医≧縺ｫ縺励◆縲・
    - 螟ｱ謨玲凾縺ｯ `loader error` 險ｺ譁ｭ縺ｨ遨ｺ驟榊・ payload 繧定ｿ斐☆縲・
  - `tests/tree/17_name_resolution_vfs_cross_file.js` 繧定ｿｽ蜉�縲・
    - `core/math` 縺ｮ `add` 蜿ら・縺ｫ蟇ｾ縺励※ `resolved_def.span.file_path` 縺・`/stdlib/core/math.nepl` 縺ｫ縺ｪ繧九％縺ｨ繧呈､懆ｨｼ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `17/17 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
- todo蜿肴丐:
  - `todo.md` 2逡ｪ縺九ｉ縲形analyze_name_resolution` 縺ｧ import/alias/use 霍ｨ縺取凾縺ｮ螳夂ｾｩ蜈・ヵ繧｡繧､繝ｫ諠・�ｱ繧定ｿ斐☆縲阪ｒ蜑企勁・亥ｮ御ｺ・ｼ峨�・
# 2026-02-27 菴懈･ｭ繝｡繝｢ (LSP/API phase2 邯咏ｶ・ token_resolution 縺ｫ doc 諠・�ｱ繧剃ｻ伜刈)
- 逶ｮ逧・
  - Hover 蜷代￠陦ｨ遉ｺ諠・�ｱ繧貞｢励ｄ縺吶◆繧√�∝ｮ夂ｾｩ繧ｸ繝｣繝ｳ繝玲ュ蝣ｱ縺ｨ蜷後§邨瑚ｷｯ縺ｧ doc comment 繧ょ叙蠕励〒縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
- 螳溯｣・
  - `nepl-web/src/lib.rs`
    - `analyze_semantics` / `analyze_semantics_with_vfs` 縺ｮ `token_resolution` 邨・∩遶九※譎ゅ↓縲・
      `resolved_definition` 縺ｨ `candidate_definitions` 縺ｸ `doc` 繧剃ｻ倅ｸ趣ｼ亥ｭ伜惠譎ゅ・縺ｿ・峨�・
  - `tests/tree/16_semantics_vfs_cross_file.js`
  - `tests/tree/17_name_resolution_vfs_cross_file.js`
    - VFS 霍ｨ縺主ｮ夂ｾｩ隗｣豎ｺ繝・せ繝医ｒ邯ｭ謖√＠縺､縺､縲、PI蝗槫ｸｰ縺悟・縺ｪ縺・％縺ｨ繧堤｢ｺ隱阪�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `17/17 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
# 2026-02-27 菴懈･ｭ繝｡繝｢ (LSP/API phase2 螳御ｺ・ Hover/Inlay 蜷代￠ `token_hints` 霑ｽ蜉�)
- 逶ｮ逧・
  - `todo.md` 2逡ｪ縺ｮ谿倶ｻｶ・・over/Inlay 蜷代￠邨ｱ蜷・PI・峨ｒ縲∵里蟄・`analyze_semantics*` 縺ｫ霑ｽ蜉�縺励※蛻ｩ逕ｨ蛛ｴ縺ｮ邨仙粋繧ｳ繧ｹ繝医ｒ荳九￡繧九�・
- 螳溯｣・
  - `nepl-web/src/lib.rs`
    - `build_token_hints_to_js(...)` 繧定ｿｽ蜉�縲・
    - `token_semantics`・亥梛繝ｻ蠑冗ｯ・峇繝ｻ蠑墓焚遽・峇・峨→ `resolve_trace`・亥ｮ夂ｾｩ繧ｸ繝｣繝ｳ繝励・蛟呵｣懊・doc・峨ｒ token 蜊倅ｽ阪〒邨ｱ蜷医＠縲～token_hints` 驟榊・繧堤函謌舌�・
    - `analyze_semantics` / `analyze_semantics_with_vfs` 縺ｮ霑泌唆蛟､縺ｸ `token_hints` 繧定ｿｽ蜉�縲・
    - 螟ｱ謨礼ｳｻ蛻・ｲ舌〒繧・`token_hints: []` 繧定ｿ斐☆繧医≧邨ｱ荳�縲・
  - `tests/tree/04_semantics_tree.js`
    - `token_hints` 縺悟ｭ伜惠縺励�～inferred_type` 縺ｨ `resolved_def_id` 繧貞酔譎ゅ↓謖√▽隕∫ｴ�縺後≠繧九％縺ｨ繧定ｿｽ蜉�讀懆ｨｼ縲・
  - `tests/tree/16_semantics_vfs_cross_file.js`
    - `token_hints` 縺ｫ cross-file `resolved_definition.span.file_path` 縺ｨ `inferred_type` 縺悟酔譎ゅ↓蜈･繧九％縺ｨ繧定ｿｽ蜉�讀懆ｨｼ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node tests/tree/run.js` -> `17/17 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2` -> `1655/1655 pass`
- todo蜿肴丐:
  - `todo.md` 2逡ｪ・域立 LSP/API phase2・峨ｒ蜑企勁縺励�∵ｮ矩�・岼繧堤ｹｰ繧贋ｸ翫￡縲・
# 2026-02-27 菴懈･ｭ繝｡繝｢ (繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝・arity 隗｣豎ｺ縺ｮ譬ｹ譛ｬ菫ｮ豁｣)
- 逶ｮ逧・
  - `let u <(i32)->i32> calc` 縺ｮ繧医≧縺ｪ髢｢謨ｰ蛟､譁・ц縺ｧ縲∝酔蜷阪・逡ｰ arity 驕手ｲ�闕ｷ縺梧ｭ｣縺励￥荳�諢城∈謚槭＆繧後ｋ繧医≧縺ｫ縺吶ｋ縲・
- 蜴溷屏:
  - `Symbol::Ident` 隗｣豎ｺ縺ｧ縲・℃雋�闕ｷ髢｢謨ｰ縺ｧ繧ょ・縺ｫ `lookup_callable_any` 縺・1莉ｶ繧呈鏡縺・�∵悄蠕・梛/arity 繝吶・繧ｹ縺ｮ驕ｸ謚槭Ο繧ｸ繝・け縺ｫ蛻ｰ驕斐＠縺ｦ縺・↑縺九▲縺溘�・
  - 縺昴・邨先棡縲～calc` 縺瑚ｪ､縺｣縺溷�呵｣懶ｼ医∪縺溘・譛ｪ遒ｺ螳壼�､・峨→縺励※谿九ｊ縲～no matching overload` / `extra stack` 縺ｸ豕｢蜿翫＠縺ｦ縺・◆縲・
- 螳溯｣・
  - `nepl-core/src/typecheck.rs`
    - 隍・焚 callable 繧呈戟縺､隴伜挨蟄舌〒縺ｯ縲∝腰邏・`lookup_callable_any` 縺ｫ繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ縺励↑縺・ｈ縺・ｿｮ豁｣縲・
    - `pending_ascription` 逕ｱ譚･縺ｮ譛溷ｾ・arity 縺ｧ荳�諢上↓蛟呵｣懊′豎ｺ縺ｾ縺｣縺溷�ｴ蜷医�～FnValue` 縺ｨ縺励※遒ｺ螳壹＠ `auto_call=false` 縺ｫ縺吶ｋ繧医≧菫ｮ豁｣縲・
    - `FnValue` 縺ｫ縺ｯ髢｢謨ｰ蜷阪〒縺ｯ縺ｪ縺丞ｮ溘す繝ｳ繝懊Ν・・BindingKind::Func.symbol`・峨ｒ菫晄戟縺吶ｋ繧医≧菫ｮ豁｣縲・
- 繝・せ繝域峩譁ｰ:
  - `tests/overload.n.md`
    - `overload_select_by_arity` 繧・`compile_fail (diag_id:3006)` 縺九ｉ謌仙粥繧ｱ繝ｼ繧ｹ・・ret: 12`・峨∈螟画峩縲・
- 髢｢騾｣繝峨く繝･繝｡繝ｳ繝医ユ繧ｹ繝井ｿｮ豁｣:
  - `stdlib/core/option.nepl` / `stdlib/core/result.nepl`
    - `should_panic` doctest 縺ｧ譛�邨ょｼ上′ `i32` 縺ｫ縺ｪ縺｣縺ｦ縺・◆縺溘ａ `D3004` 縺ｫ縺ｪ縺｣縺ｦ縺・◆縲Ａlet v ...; ()` 縺ｸ菫ｮ豁｣縺励※縲∝梛謨ｴ蜷医ｒ邯ｭ謖√＠縺溘∪縺ｾ panic 邨瑚ｷｯ繧呈､懆ｨｼ縺ｧ縺阪ｋ繧医≧縺ｫ縺励◆縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i stdlib/core/option.nepl -i stdlib/core/result.nepl --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-option-result-dual.json -j 2` -> `18/18 pass`
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/functions.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-overload-functions-no-stdlib.json -j 2` -> `101/101 pass`
- todo蜿肴丐:
  - `todo.md` 蜈磯�ｭ縺ｮ縲後が繝ｼ繝舌・繝ｭ繝ｼ繝芽ｧ｣豎ｺ縺ｮ arity 螳悟・蟇ｾ蠢懊�阪ｒ蜑企勁・亥ｮ御ｺ・ｼ峨�・
# 2026-02-27 菴懈･ｭ繝｡繝｢ (stdlib/tests 繧・functions.n.md 蠖｢蠑上∈蛻・牡蜀肴ｧ区・)
- 逶ｮ逧・
  - `stdlib/tests/*.n.md` 縺ｮ螟ｱ謨暦ｼ・un unreachable・峨ｒ縲∫樟陦梧ｧ区枚繝ｻ迴ｾ陦後Λ繝ｳ繧ｿ繧､繝�蜑肴署縺ｧ螳牙ｮ壼喧縺吶ｋ縲・
  - 1繝輔ぃ繧､繝ｫ1蟾ｨ螟ｧ繧ｱ繝ｼ繧ｹ縺ｧ縺ｯ縺ｪ縺上�～tests/functions.n.md` 縺ｨ蜷梧ｧ倥・縲瑚､・焚蟆上こ繝ｼ繧ｹ縲肴ｧ区・縺ｸ邨ｱ荳�縺吶ｋ縲・
- 螳溯｣・
  - `stdlib/tests/stack.n.md`
    - 3繧ｱ繝ｼ繧ｹ縺ｸ蛻・牡: `stack_new_and_len`, `stack_peek_and_pop`, `stack_pop_empty`縲・
  - `stdlib/tests/btreemap.n.md`
    - 3繧ｱ繝ｼ繧ｹ縺ｸ蛻・牡: `btreemap_insert_and_len`, `btreemap_get_and_remove`, `btreemap_update_existing`縲・
  - `stdlib/tests/btreeset.n.md`
    - 3繧ｱ繝ｼ繧ｹ縺ｸ蛻・牡: `btreeset_insert_and_len`, `btreeset_contains_and_remove`, `btreeset_duplicate_insert`縲・
  - `stdlib/tests/string.n.md`
    - 3繧ｱ繝ｼ繧ｹ縺ｸ蛻・牡: `string_len_and_concat`, `string_trim_and_slice`, `string_split_and_builder`縲・
  - `stdlib/tests/cliarg.n.md`
    - argv 豕ｨ蜈･蟾ｮ蛻・ｼ・asm/llvm・峨〒荳榊ｮ牙ｮ壹□縺｣縺溷宍蟇・ｯ碑ｼ・ｒ蟒・ｭ｢縺励�～cliarg` API 蜻ｼ縺ｳ蜃ｺ縺励・蝓ｺ譛ｬ繧ｹ繝｢繝ｼ繧ｯ・・ret` 蛻､螳夲ｼ峨∈螟画峩縲・
  - `stdlib/tests/fs.n.md`
    - 譌｢蟄倥・ missing-path 讀懆ｨｼ繧堤ｶｭ謖・ｼ・Result::Err` 邨瑚ｷｯ・峨�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/tests/stack.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/stdlib-collections-split.json -j 1` -> `27/27 pass`
  - `node nodesrc/tests.js -i stdlib/tests/stack.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i stdlib/tests/cliarg.n.md -i stdlib/tests/fs.n.md -i stdlib/tests/string.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/stdlib-tests-six-no-stdlib.json -j 1` -> `42/42 pass`
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/functions.n.md --runner all --llvm-all --assert-io --strict-dual --no-tree -o /tmp/tests-overload-functions-dual-after-stdlib-rewrite.json -j 2` -> `612/612 pass`
# 2026-02-27 菴懈･ｭ繝｡繝｢ (驕手ｲ�闕ｷ莉墓ｧ倥↓蜷医ｏ縺帙◆ neplg2 繝・せ繝域峩譁ｰ + stdlib/tests 蛻・牡謨ｴ蛯・
- 逶ｮ逧・
  - `tests/neplg2.n.md` 縺ｮ compile_fail 譛溷ｾ・′迴ｾ莉墓ｧ假ｼ育焚 arity 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝芽ｨｱ蜿ｯ繝ｻ譛溷ｾ・梛縺ｧ謌ｻ繧雁�､驕手ｲ�闕ｷ繧帝∈謚橸ｼ峨→荳肴紛蜷医□縺｣縺溘◆繧√�∽ｻ墓ｧ俶ｺ匁侠縺ｫ譖ｴ譁ｰ縺吶ｋ縲・
  - `stdlib/tests` 縺ｮ蟾ｨ螟ｧ蜊倅ｸ�繧ｱ繝ｼ繧ｹ繧・`tests/functions.n.md` 蠖｢蠑上・蟆丞・蜑ｲ繧ｱ繝ｼ繧ｹ縺ｸ邨ｱ荳�縺励�∝・繧雁・縺代＠繧・☆縺上☆繧九�・
- 螳溯｣・
  - `tests/neplg2.n.md`
    - `overloads_with_different_arity_are_error` 繧・`overloads_with_different_arity_are_allowed` 縺ｫ螟画峩縲・
    - `overloads_ambiguous_return_type_is_error` 繧・`overloads_by_return_type_are_resolved_by_expected_type` 縺ｫ螟画峩縲・
    - 縺・★繧後ｂ `compile_fail` 縺九ｉ `ret: 1` 縺ｮ謌仙粥繝・せ繝医∈螟画峩縲・
  - `stdlib/tests/stack.n.md`, `stdlib/tests/btreemap.n.md`, `stdlib/tests/btreeset.n.md`, `stdlib/tests/string.n.md`, `stdlib/tests/cliarg.n.md`
    - 1繝輔ぃ繧､繝ｫ1蟾ｨ螟ｧ繧ｱ繝ｼ繧ｹ繧定､・焚蟆上こ繝ｼ繧ｹ縺ｸ蜀肴ｧ区・縲・
    - 譌ｧ繧ｷ繧ｰ繝阪メ繝｣繧・尠譏ｧ縺ｪ `eq` 騾｣邨舌ｒ髯､蜴ｻ縺励�∫樟陦梧ｧ区枚縺ｧ螳牙ｮ壼虚菴懊☆繧句ｽ｢縺ｫ謨ｴ逅・�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/neplg2.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-neplg2-current.json -j 1` -> `112/112 pass`
  - `node nodesrc/tests.js -i stdlib/tests/stack.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i stdlib/tests/cliarg.n.md -i stdlib/tests/fs.n.md -i stdlib/tests/string.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/stdlib-tests-six-no-stdlib.json -j 1` -> `42/42 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --runner all --llvm-all --assert-io --strict-dual --no-tree -o /tmp/tests-dual-full-current.json -j 2` -> `1739/1739 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (collections pipe蝗槫ｸｰ縺ｮ譬ｹ譛ｬ菫ｮ豁｣)
- 逶ｮ逧・
  - `tests/pipe_collections.n.md` 縺ｮ螳溯｡悟､ｱ謨暦ｼ・memory access out of bounds`・峨→縲～stdlib/nm/*.nepl` 縺ｮ `ambiguous overload` 蝗槫ｸｰ繧貞酔譎ゅ↓譬ｹ譛ｬ隗｣豸医☆繧九�・
- 蜴溷屏:
  - `list` 縺ｧ pipe 逕ｨ繧ｨ繧､繝ｪ繧｢繧ｹ縺ｨ縺励※ `cons` 繧・`list_cons` 縺ｫ逶ｴ謗･譚溽ｸ帙＠縺ｦ縺・◆縺溘ａ縲～xs |> cons 3` 縺・`cons xs 3`・亥ｼ墓焚鬆・�・ｼ峨→縺励※隗｣驥医＆繧後�∽ｸ肴ｭ｣繝昴う繝ｳ繧ｿ繧・next 縺ｫ譬ｼ邏阪＠縺ｦ OOB 繧定ｪ倡匱縺励※縺・◆縲・
  - `new/len/...` 縺ｮ豎守畑遏ｭ蜷阪お繧､繝ｪ繧｢繧ｹ蟆主・縺ｫ繧医ｊ縲～as *` 蜿悶ｊ霎ｼ縺ｿ譎ゅ・蛟呵｣憺寔蜷医′驕主臆蛹悶＠縲～nm` 蛛ｴ縺ｧ繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝画尠譏ｧ蛹悶ｒ逋ｺ逕溘＆縺帙※縺・◆縲・
- 螳溯｣・
  - `stdlib/alloc/collections/list.nepl`
    - `list_push_front <(i32,.T)*>i32>` 繧定ｿｽ蜉�・・ipe縺ｮ隨ｬ荳�蠑墓焚隕冗ｴ・↓蜷医ｏ縺帙◆螳牙・縺ｪ蜈磯�ｭ霑ｽ蜉�・峨�・
    - `list_len` / `list_get` 繧・pure 鄂ｲ蜷阪〒蜀榊ｸｰ螳溯｣・↓邨ｱ荳�・亥憶菴懃畑譁・ц萓晏ｭ倥ｒ髯､蜴ｻ・峨�・
    - 豎守畑遏ｭ蜷阪お繧､繝ｪ繧｢繧ｹ鄒､繧帝勁蜴ｻ縺励�∵尠譏ｧ蛹匁ｺ舌ｒ驕ｮ譁ｭ縲・
  - `tests/pipe_collections.n.md`
    - 縺吶∋縺ｦ譏守､ｺ API 蜻ｼ縺ｳ蜃ｺ縺励∈譖ｴ譁ｰ縲・
    - list 繧ｱ繝ｼ繧ｹ縺ｯ `list_push_front` 繧堤畑縺・◆ pipe 讀懆ｨｼ縺ｫ螟画峩縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/pipe_collections.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i stdlib/tests/list.n.md -i stdlib/tests/stack.n.md -i stdlib/nm/parser.nepl -i stdlib/nm/html_gen.nepl --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-pipe-tree-collections-after-fix.json -j 2` -> `566/566 pass`
  - `NO_COLOR=false node nodesrc/tests.js --changed --changed-base HEAD --runner all --llvm-all --assert-io --strict-dual --no-tree -o /tmp/tests-changed-after-pipe-fix.json -j 2` -> `49/49 pass`
- 蟾ｮ蛻・隱ｲ鬘・
  - 豎守畑遏ｭ蜷・alias 繧偵げ繝ｭ繝ｼ繝舌Ν蟆主・縺吶ｋ譁ｹ蠑上・縲∫樟陦後・繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝芽ｧ｣豎ｺ縺ｧ縺ｯ蝗槫ｸｰ繝ｪ繧ｹ繧ｯ縺碁ｫ倥＞縲ゆｻ雁ｾ後・繝｢繧ｸ繝･繝ｼ繝ｫ謗･鬆ｭ霎暸PI繧貞渕譛ｬ縺ｨ縺励�∝ｿ・ｦ√↑繧・resolver/typecheck 蛛ｴ縺ｮ蛟呵｣懃ｵ槭ｊ霎ｼ縺ｿ諡｡蠑ｵ繧貞・陦後＠縺ｦ縺九ｉ蜀榊ｰ主・縺吶ｋ縲・

# 2026-02-27 菴懈･ｭ繝｡繝｢ (pipe collections 繝・せ繝域僑蠑ｵ: hashmap/hashset)
- 逶ｮ逧・
  - tree邉ｻ・・tree・峨↓邯壹″縲”ash 邉ｻ繧ｳ繝ｬ繧ｯ繧ｷ繝ｧ繝ｳ縺ｧ繧・pipe 縺ｮ隨ｬ荳�蠑墓焚遘ｻ蜍輔′螳牙ｮ壼虚菴懊☆繧九％縺ｨ繧貞崋螳壹☆繧九�・
- 螳溯｣・
  - `tests/pipe_collections.n.md` 縺ｫ莉･荳九ｒ霑ｽ蜉�:
    - `pipe_hashmap_usage`
    - `pipe_hashset_usage`
  - 縺ｩ縺｡繧峨ｂ遏ｭ蜷・alias 縺ｧ縺ｯ縺ｪ縺乗・遉ｺ API・・hashmap_*`, `hashset_*`・峨〒讀懆ｨｼ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/pipe_collections.n.md -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/list.n.md -i stdlib/tests/stack.n.md --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-pipe-collections-hash.json -j 2` -> `547/547 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (collections: btreemap/btreeset 縺ｮ struct 髫�阡ｽ)
- 逶ｮ逧・
  - `collections` 縺ｮ蜈ｬ髢・API 縺九ｉ `i32` 繝昴う繝ｳ繧ｿ繧帝國阡ｽ縺励�√ョ繝ｼ繧ｿ蝙九ｒ譏守､ｺ逧・↑ struct 縺ｧ謇ｱ縺医ｋ蠖｢縺ｸ蟇・○繧九�・
- 螳溯｣・
  - `stdlib/alloc/collections/btreemap.nepl`
    - `struct BTreeMap<.V>` 繧定ｿｽ蜉�・・hdr <i32>`・峨�・
    - 蜈ｬ髢矩未謨ｰ繧ｷ繧ｰ繝阪メ繝｣繧・`i32` 縺九ｉ `BTreeMap<.V>` 縺ｸ螟画峩縲・
    - `insert/remove/clear` 縺ｯ譖ｴ譁ｰ蠕後・ `BTreeMap<.V>` 繧定ｿ斐☆蠖｢縺ｸ螟画峩縲・
  - `stdlib/alloc/collections/btreeset.nepl`
    - `struct BTreeSet` 繧定ｿｽ蜉�・・hdr <i32>`・峨�・
    - 蜈ｬ髢矩未謨ｰ繧ｷ繧ｰ繝阪メ繝｣繧・`i32` 縺九ｉ `BTreeSet` 縺ｸ螟画峩縲・
    - `insert/remove/clear` 縺ｯ譖ｴ譁ｰ蠕後・ `BTreeSet` 繧定ｿ斐☆蠖｢縺ｸ螟画峩縲・
  - 繝・せ繝域峩譁ｰ:
    - `stdlib/tests/btreemap.n.md`
    - `stdlib/tests/btreeset.n.md`
    - `tests/pipe_collections.n.md`
    - move 隕丞援縺ｫ蜷医ｏ縺帙�∝�､蜿門ｾ礼ｳｻ・・get/contains/len`・峨→譖ｴ譁ｰ邉ｻ・・insert/remove`・峨・蛻ｩ逕ｨ繧貞・譚溽ｸ帙∪縺溘・蛻･繧､繝ｳ繧ｹ繧ｿ繝ｳ繧ｹ縺ｧ蛻・屬縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/stack_collections.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/collections-scope.json -j 2`
  - 邨先棡: `54/54 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (collections: hashset 縺ｮ struct 髫�阡ｽ)
- 逶ｮ逧・
  - `hashset` 蜈ｬ髢・API 縺ｮ `i32` 繝昴う繝ｳ繧ｿ髴ｲ蜃ｺ繧帝勁蜴ｻ縺吶ｋ縲・
- 螳溯｣・
  - `stdlib/alloc/collections/hashset.nepl`
    - `struct HashSet` 繧定ｿｽ蜉�・・hdr <i32>`・峨�・
    - `hashset_new` 縺ｮ謌ｻ繧雁�､繧・`HashSet` 縺ｸ螟画峩縲・
    - `hashset_contains` / `hashset_len` / `hashset_free` 繧・`HashSet` 蠑墓焚縺ｸ螟画峩縲・
    - `hashset_insert` / `hashset_remove` 縺ｯ譖ｴ譁ｰ蠕後・ `HashSet` 繧定ｿ斐☆蠖｢縺ｸ螟画峩縲・
  - `stdlib/tests/hashset.n.md`
    - 譁ｰ繧ｷ繧ｰ繝阪メ繝｣縺ｨ move 隕丞援縺ｫ蜷医ｏ縺帙※繝・せ繝医ｒ蜀肴ｧ区・縲・
  - `tests/pipe_collections.n.md`
    - hashset 縺ｮ pipe 繧ｱ繝ｼ繧ｹ繧・`HashSet` 迚医∈譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/tests/hashset.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i tests/stack_collections.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/collections-scope-v2.json -j 2`
  - 邨先棡: `57/57 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (collections: hashmap 縺ｮ struct 髫�阡ｽ繧貞ｮ御ｺ・
- 逶ｮ逧・
  - `hashmap` 蜈ｬ髢・API 縺ｮ `i32` 繝昴う繝ｳ繧ｿ髴ｲ蜃ｺ繧帝勁蜴ｻ縺励�∽ｻ・collections 縺ｨ蜷後§譁ｹ驥晢ｼ亥梛髫�阡ｽ + move隕丞援貅匁侠・峨∈謠・∴繧九�・
- 螳溯｣・
  - `stdlib/alloc/collections/hashmap.nepl`
    - `struct HashMap<.V>` 繧貞・髢句梛縺ｨ縺励※菴ｿ逕ｨ縲・
    - `hashmap_new` 縺ｮ謌ｻ繧雁�､繧・`HashMap<.V>` 縺ｸ螟画峩縲・
    - `hashmap_insert` / `hashmap_remove` 繧・`HashMap<.V> -> HashMap<.V>` 縺ｸ螟画峩縲・
    - `hashmap_get` / `hashmap_contains` / `hashmap_len` / `hashmap_free` 繧・`HashMap<.V>` 蠑墓焚縺ｸ螟画峩縲・
    - 蜀・Κ繧｢繧ｯ繧ｻ繧ｹ縺ｯ `get hm "hdr"` 邨檎罰縺ｸ邨ｱ荳�縲・
  - 繝・せ繝域峩譁ｰ:
    - `stdlib/tests/hashmap.n.md`: 譁ｰ繧ｷ繧ｰ繝阪メ繝｣ + move隕丞援縺ｫ蜷医ｏ縺帙※繧ｱ繝ｼ繧ｹ繧貞・讒区・縲・
    - `tests/pipe_collections.n.md`: `pipe_hashmap_usage` 繧・`HashMap<.V>` 迚医∈譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i tests/stack_collections.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/collections-scope-v3.json -j 2`
  - 邨先棡: `60/60 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (collections: hashmap_str/hashset_str 縺ｮ struct髫�阡ｽ)
- 逶ｮ逧・
  - `hashmap_str` / `hashset_str` 縺ｮ蜈ｬ髢帰PI縺九ｉ `i32` 繝昴う繝ｳ繧ｿ髴ｲ蜃ｺ繧帝勁蜴ｻ縺励�…ollections蜈ｨ菴薙・蝙区婿驥昴ｒ邨ｱ荳�縺吶ｋ縲・
- 螳溯｣・
  - `stdlib/alloc/collections/hashmap_str.nepl`
    - `struct HashMapStr<.V> { hdr <i32> }` 繧貞ｰ主・縲・
    - `new/insert/remove/len/free/get/contains` 繧・`HashMapStr<.V>` 蜑肴署縺ｸ螟画峩縲・
    - `insert/remove` 縺ｯ譖ｴ譁ｰ蠕後・ `HashMapStr<.V>` 繧定ｿ斐☆蠖｢縺ｸ螟画峩縲・
  - `stdlib/alloc/collections/hashset_str.nepl`
    - `struct HashSetStr { hdr <i32> }` 繧貞ｰ主・縲・
    - `new/insert/remove/len/free/contains` 繧・`HashSetStr` 蜑肴署縺ｸ螟画峩縲・
    - `insert/remove` 縺ｯ譖ｴ譁ｰ蠕後・ `HashSetStr` 繧定ｿ斐☆蠖｢縺ｸ螟画峩縲・
  - 繝・せ繝域峩譁ｰ:
    - `stdlib/tests/hashmap_str.n.md`
    - `stdlib/tests/hashset_str.n.md`
    - move隕丞援縺ｫ蜷医ｏ縺帙※隱ｭ縺ｿ蜿悶ｊ邉ｻ繝√ぉ繝・け繧貞挨繧､繝ｳ繧ｹ繧ｿ繝ｳ繧ｹ縺ｧ蛻・屬縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/hashmap_str.nepl -i stdlib/alloc/collections/hashset_str.nepl -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/hashset_str.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/hashstr-final-scope.json -j 2`
  - 邨先棡: `10/10 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (safe stdlib 繧偵ョ繝輔か繝ｫ繝亥喧: Result/Diag)
- 逶ｮ逧・
  - collections API 繧偵�悟挨蜷阪が繝励す繝ｧ繝ｳ縲阪〒縺ｯ縺ｪ縺上�～Result/Diag` 繧定ｿ斐☆螳牙・API縺ｨ縺励※讓呎ｺ門喧縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - `alloc/diag/error.nepl` 縺ｧ `concat` 萓晏ｭ倥・ import 縺梧ｬ�關ｽ縺励�∬ｭ伜挨蟄占ｧ｣豎ｺ縺悟ｴｩ繧後※縺・◆縲・
  - collections 螳溯｣・・ `if` 蛻・ｲ舌↓譌ｧ險俶ｳ・`do:` 縺梧ｮ句ｭ倥＠縲∝梛/蛻ｶ蠕｡繝輔Ο繝ｼ隗｣譫舌′蟠ｩ繧後※縺・◆縲・
- 螳溯｣・
  - `stdlib/alloc/diag/error.nepl`
    - `#import "alloc/string" as *` 繧定ｿｽ蜉�縲・
    - `DiagCode` / `Diag` / `diag_err` 邉ｻ繧堤ｶｭ謖√＠縲∝ｮ牙・API縺ｮ蝓ｺ逶､繧呈怏蜉ｹ蛹悶�・
  - `stdlib/alloc/collections/hashmap.nepl`
  - `stdlib/alloc/collections/hashset.nepl`
  - `stdlib/alloc/collections/hashmap_str.nepl`
  - `stdlib/alloc/collections/hashset_str.nepl`
    - `new/insert/remove` 繧・`Result<..., Diag>` 霑泌唆縺ｮ繝・ヵ繧ｩ繝ｫ繝・PI縺ｨ縺励※遒ｺ螳壹�・
    - `if` 蛻・ｲ仙・縺ｮ辟｡蜉ｹ縺ｪ `do:` 繧帝勁蜴ｻ縺励�∵ｭ｣蟶ｸ縺ｪ蠑上ヵ繝ｭ繝ｼ縺ｸ菫ｮ豁｣縲・
  - 繝・せ繝域峩譁ｰ:
    - `stdlib/tests/hashmap.n.md`
    - `stdlib/tests/hashset.n.md`
    - `stdlib/tests/hashmap_str.n.md`
    - `stdlib/tests/hashset_str.n.md`
    - `tests/pipe_collections.n.md`
    - `tests/selfhost_req.n.md`
    - `unwrap_ok_i` 萓晏ｭ倥ｒ髯､蜴ｻ縺励�∝推繝・せ繝亥・縺ｧ `must_*`・・Result` 繧貞女縺代ｋ繝ｭ繝ｼ繧ｫ繝ｫ髢｢謨ｰ・峨∈邨ｱ荳�縲・
    - move隕丞援縺ｫ蜷医ｏ縺帙※蛟､蜀榊茜逕ｨ繝代ち繝ｼ繝ｳ繧貞・髮｢縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `node nodesrc/tests.js -i stdlib/core/result.nepl -i stdlib/alloc/diag/error.nepl -i stdlib/alloc/collections/hashmap.nepl -i stdlib/alloc/collections/hashset.nepl -i stdlib/alloc/collections/hashmap_str.nepl -i stdlib/alloc/collections/hashset_str.nepl -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/hashset_str.n.md -i tests/pipe_collections.n.md -i tests/selfhost_req.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/diag-collections-scope.json -j 2`
  - 邨先棡: `67/67 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (collections螳牙・蛹・ stack 繧・Result/Diag 繝・ヵ繧ｩ繝ｫ繝医∈邨ｱ荳�)
- 逶ｮ逧・
  - collections 縺ｮ螳牙・蛹匁婿驥昴↓蜷医ｏ縺帙※ `stack` 繧ょ､ｱ謨怜庄閭ｽ謫堺ｽ懊ｒ `Result<..., Diag>` 縺ｧ謇ｱ縺・�・
- 螳溯｣・
  - `stdlib/alloc/collections/stack.nepl`
    - `stack_new`: `()*>Result<Stack<.T>, Diag>` 縺ｸ螟画峩縲・
    - `stack_push`: `(Stack<.T>, .T)*>Result<Stack<.T>, Diag>` 縺ｸ螟画峩縲・
    - `alloc/realloc` 螟ｱ謨玲凾縺ｫ `diag_out_of_memory` 繧定ｿ斐☆繧医≧菫ｮ豁｣縲・
  - `stdlib/tests/stack.n.md`
  - `tests/stack_collections.n.md`
  - `tests/pipe_collections.n.md`
    - `stack_new`/`stack_push` 縺ｮ謌ｻ繧雁�､繧・`unwrap_ok<Stack<...>, Diag>` 縺ｧ螻暮幕縺吶ｋ蠖｢縺ｸ譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl -i stdlib/tests/stack.n.md -i tests/stack_collections.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/stack-safe-scope.json -j 2` -> `74/74 pass`
- 蛯呵�・
  - `todo.md` 縺ｮ collections蜀崎ｨｭ險医・邯咏ｶ壻ｸｭ縺ｮ縺溘ａ縲∝ｮ御ｺ・�・岼蜑企勁縺ｯ縺ｾ縺�陦後▲縺ｦ縺・↑縺・�・

# 2026-02-27 菴懈･ｭ繝｡繝｢ (stack doctest 縺ｮ蜀肴怏蜉ｹ蛹・
- 逶ｮ逧・
  - `stack` 縺ｮ API 螟画峩・・stack_new`/`stack_push` 縺・`Result` 霑泌唆・峨↓蜷医ｏ縺帙�～stack.nepl` 蜀・doctest 繧貞ｮ溯｡悟ｯｾ雎｡縺ｸ謌ｻ縺吶�・
- 蜴溷屏:
  - 蜈郁｡御ｿｮ豁｣譎ゅ�∝商縺・ｽｿ逕ｨ萓九′豺ｷ蝨ｨ縺励※縺・◆縺溘ａ `neplg2:test[skip]` 縺ｧ荳�譎る��驕ｿ縺輔ｌ縺ｦ縺・◆縲・
- 螳溯｣・
  - `stdlib/alloc/collections/stack.nepl` 縺ｮ蜈ｨ `neplg2:test[skip]` 繧・`neplg2:test` 縺ｫ謌ｻ縺励◆縲・
  - doctest 蜀・・蛻晄悄蛹・霑ｽ蜉�蜃ｦ逅・ｒ `unwrap_ok<Stack<...>, Diag>` 邨檎罰縺ｫ邨ｱ荳�縺励◆縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl -i stdlib/tests/stack.n.md -i tests/stack_collections.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/stack-safe-scope.json -j 2` -> `84/84 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (collections蜀埼・鄂ｮ: vec/sort 繧・collections 驟堺ｸ九∈遘ｻ蜍・
- 逶ｮ逧・
  - `todo.md` 縺ｮ collections 蜀崎ｨｭ險磯�・岼縺ｫ豐ｿ縺｣縺ｦ `vec/sort` 繧呈眠驟咲ｽｮ縺ｸ遘ｻ陦後☆繧九�・
- 螳溯｣・
  - `stdlib/alloc/vec.nepl` -> `stdlib/alloc/collections/vec.nepl` 縺ｸ遘ｻ蜍輔�・
  - `stdlib/alloc/sort.nepl` -> `stdlib/alloc/collections/vec/sort.nepl` 縺ｸ遘ｻ蜍輔�・
  - `stdlib` / `tests` / `examples` / `tutorials` 縺ｮ import 繧剃ｸ�諡ｬ譖ｴ譁ｰ:
    - `"alloc/vec"` -> `"alloc/collections/vec"`
    - `"alloc/sort"` -> `"alloc/collections/vec/sort"`
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - 谺｡繧貞ｯｾ雎｡縺ｫ dual 螳溯｡・ `243/243 pass`
    - `stdlib/alloc/collections/vec.nepl`
    - `stdlib/alloc/collections/vec/sort.nepl`
    - `stdlib/alloc/encoding/json.nepl`
    - `stdlib/alloc/hash/sha256.nepl`
    - `stdlib/alloc/string.nepl`
    - `stdlib/kp/kpgraph.nepl`
    - `stdlib/kp/kpread.nepl`
    - `stdlib/std/fs.nepl`
    - `stdlib/tests/hash.n.md`
    - `stdlib/tests/string.n.md`
    - `stdlib/tests/vec.n.md`
    - `tests/capacity_stack.n.md`
    - `tests/overload.n.md`
    - `tests/selfhost_req.n.md`
    - `tests/sort.n.md`
- 陬懆ｶｳ:
  - `--changed` 蜈ｨ菴灘ｮ溯｡後〒縺ｯ縲∵里蟄倥・繝ｭ繝ｼ繧ｫ繝ｫ螟画峩 `stdlib/nm/parser.nepl` 縺ｫ襍ｷ蝗�縺吶ｋ螟ｱ謨励′豺ｷ縺悶ｋ縺溘ａ縲∽ｻ雁屓縺ｮ遘ｻ險ｭ讀懆ｨｼ縺ｯ蠖ｱ髻ｿ遽・峇繧呈・遉ｺ謖・ｮ壹＠縺ｦ螳滓命縺励◆縲・

# 2026-02-27 菴懈･ｭ繝｡繝｢ (collections: ringbuffer/queue 霑ｽ蜉�)
- 逶ｮ逧・
  - `todo.md` 縺ｮ collections 蜀崎ｨｭ險磯�・岼縺ｫ豐ｿ縺｣縺ｦ縲：IFO蝓ｺ逶､縺ｮ `RingBuffer` 縺ｨ `Queue` 繧定ｿｽ蜉�縺吶ｋ縲・
- 螳溯｣・
  - 霑ｽ蜉�: `stdlib/alloc/collections/ringbuffer.nepl`
    - `RingBuffer<.T>` 讒矩��菴難ｼ・en/cap/head/data・・
    - `ringbuffer_new/with_capacity/push_back/pop_front/peek_front/len/is_empty/clear/free`
    - 螟ｱ謨礼ｳｻ縺ｯ `Result<..., Diag>`縲∝叙蠕礼ｳｻ縺ｯ `Option`
  - 霑ｽ蜉�: `stdlib/alloc/collections/queue.nepl`
    - `Queue<.T>` 繧・`RingBuffer<.T>` 縺ｧ螳溯｣・
    - `queue_new/with_capacity/push/pop/peek/len/is_empty/clear/free`
  - 霑ｽ蜉�繝・せ繝・
    - `stdlib/tests/ringbuffer.n.md`
    - `stdlib/tests/queue.n.md`
    - `tests/ringbuffer_collections.n.md`
    - `tests/queue_collections.n.md`
    - `tests/pipe_collections.n.md` 縺ｫ ringbuffer/queue 繧ｱ繝ｼ繧ｹ霑ｽ蜉�
- 荳榊・蜷井ｿｮ豁｣:
  - move 繧ｻ繝槭Φ繝・ぅ繧ｯ繧ｹ驕募渚・亥酔荳�蛟､縺ｮ蜀榊茜逕ｨ・峨ｒ縲∵里蟄俶婿驥昴←縺翫ｊ縲悟酔荳�讒狗ｯ峨ｒ蛻･譚溽ｸ帙↓蛻・屬縲阪〒隗｣豸医�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i stdlib/alloc/collections/ringbuffer.nepl -i stdlib/alloc/collections/queue.nepl -i stdlib/tests/ringbuffer.n.md -i stdlib/tests/queue.n.md -i tests/ringbuffer_collections.n.md -i tests/queue_collections.n.md -i tests/pipe_collections.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-ringbuffer-queue.json -j 2` -> `42/42 pass`
# 2026-02-27 菴懈･ｭ繝｡繝｢ (main蛛･蜈ｨ諤ｧ遒ｺ隱榊ｾ後・繝悶Λ繝ｳ繝∝ｾｩ蟶ｰ縺ｨ譬ｹ譛ｬ菫ｮ豁｣)
- 逶ｮ逧・
  - `main` 縺ｮ蛛･蜈ｨ諤ｧ繧・`trunk build` + `nodesrc/tests` 縺ｧ蜀咲｢ｺ隱阪＠縲～refactor/stdlib-modernize-pipe-result` 縺ｫ謌ｻ縺励※邯咏ｶ壼庄閭ｽ迥ｶ諷九∈蠕ｩ蟶ｰ縺吶ｋ縲・
  - `tests/neplg2.n.md` 縺ｮ螟ｱ謨・莉ｶ・・asm/llvm縺ｧ險・莉ｶ・峨ｒ蜴溷屏迚ｹ螳壹＠縺ｦ隗｣豸医☆繧九�・
- 蜴溷屏:
  - 螟ｱ謨悠D `tests/neplg2.n.md::doctest#37/#38` 縺ｯ `#target` 邉ｻ縺ｧ縺ｯ縺ｪ縺上�∝ｮ滄圀縺ｫ縺ｯ縲後が繝ｼ繝舌・繝ｭ繝ｼ繝峨�阪ユ繧ｹ繝医□縺｣縺溘�・
  - 繝・せ繝域悄蠕・�､縺梧立莉墓ｧ倥・ `compile_fail` 縺ｮ縺ｾ縺ｾ谿九▲縺ｦ縺翫ｊ縲∫樟螳溯｣・ｼ・rity隗｣豎ｺ繝ｻ謌ｻ繧雁�､譁・ц隗｣豎ｺ・峨→荳肴紛蜷医□縺｣縺溘�・
- 螳溯｣・
  - `tests/neplg2.n.md`
    - `overloads_with_different_arity_are_error` 繧・`..._are_allowed` 縺ｫ譖ｴ譁ｰ縺励�～compile_fail` 縺九ｉ `ret: 1` 縺ｮ螳溯｡梧､懆ｨｼ縺ｸ螟画峩縲・
    - `overloads_ambiguous_return_type_is_error` 繧・`overloads_can_be_resolved_by_return_context` 縺ｫ譖ｴ譁ｰ縺励�～compile_fail` 縺九ｉ `ret: 1` 縺ｸ螟画峩縲・
  - 菴ｵ縺帙※縲∽ｽ懈･ｭ繝・Μ繝ｼ縺ｫ谿九▲縺ｦ縺・◆莉･荳九・菫ｮ豁｣繧堤ｶ咏ｶ・
    - `nepl-core/src/compiler.rs`・・arget 隗｣豎ｺ譎ゅ・險ｺ譁ｭ邨瑚ｷｯ・・
    - `nepl-core/src/codegen_llvm.rs`・・LVM蛛ｴ險ｺ譁ｭ隕∫ｴ・ｼ・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/neplg2.n.md -i tests/if.n.md -i tests/intrinsic.n.md -o /tmp/tests-targeted-after-neplg2-fix.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 1`
    -> `828/828 pass`
  - `NO_COLOR=false node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-sync.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 2`
    -> `1822/1822 pass`
# 2026-02-27 菴懈･ｭ繝｡繝｢ (stdlib stack 縺ｮ遏ｭ邵ｮAPI霑ｽ蜉�)
- 逶ｮ逧・
  - `alloc/collections/stack` 縺ｧ prefix 縺ｪ縺怜他縺ｳ蜃ｺ縺励ｒ蜿ｯ閭ｽ縺ｫ縺励�｝ipe 險俶ｳ輔〒縺ｮ蜿ｯ隱ｭ諤ｧ繧剃ｸ翫￡繧九�・
- 螳溯｣・
  - `stdlib/alloc/collections/stack.nepl`
    - 譌｢蟄・API 縺ｸ縺ｮ蟋碑ｭｲ縺ｨ縺励※遏ｭ邵ｮ髢｢謨ｰ繧定ｿｽ蜉�:
      - `new`, `push`, `pop`, `peek`, `len`, `clear`, `free`
    - 蜷・洒邵ｮ髢｢謨ｰ縺ｫ譌･譛ｬ隱槭ラ繧ｭ繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝医ｒ霑ｽ蜉�縲・
  - `stdlib/tests/stack.n.md`
    - `stack_alias_pipe_api` 繝・せ繝医ｒ霑ｽ蜉�縺励�∫洒邵ｮ API + pipe 險俶ｳ輔〒縺ｮ蜍穂ｽ懊ｒ蝗ｺ螳壼喧縲・
- 螟ｱ謨怜次蝗�縺ｨ蟇ｾ蜃ｦ:
  - 蛻晏屓繝・せ繝亥､ｱ謨励・ `web/dist` 縺ｮ stdlib bundle 譛ｪ譖ｴ譁ｰ縺悟次蝗�縲・
  - `trunk build` 蠕後↓蜀榊ｮ溯｡後＠縺ｦ隗｣豸医�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i stdlib/tests/stack.n.md -o /tmp/tests-stack-alias-after-build.json --runner all --llvm-all --assert-io --strict-dual --no-tree -j 1`
    -> `556/556 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (collections: *_str 繝輔ぃ繧､繝ｫ邨ｱ蜷・+ hash32蟆主・)
## 菫ｮ豁｣蜀・ｮｹ
- `stdlib/alloc/collections/hashmap_str.nepl` / `hashset_str.nepl` 繧貞ｻ・ｭ｢縺励�∝ｮ溯｣・ｒ縺昴ｌ縺槭ｌ `hashmap.nepl` / `hashset.nepl` 縺ｫ邨ｱ蜷医�・
- `HashMapStr` / `HashSetStr` 縺ｮ API (`hashmap_str_*`, `hashset_str_*`) 縺ｯ邯ｭ謖√＠縺ｦ蜻ｼ縺ｳ蜃ｺ縺嶺ｺ呈鋤繧堤｢ｺ菫昴�・
- `alloc/hash/hash32.nepl` 繧定ｿｽ蜉�縺励�｀urmur3 fmix32 邉ｻ縺ｮ 32bit 豺ｷ蜷・`hash32_i32` 繧呈眠險ｭ縲・
- `hashmap.nepl` / `hashset.nepl` 縺ｮ i32 繧ｭ繝ｼ逕ｨ繝上ャ繧ｷ繝･繧堤ｰ｡譏灘ｮ溯｣・°繧・`hash32_i32` 蜻ｼ縺ｳ蜃ｺ縺励∈鄂ｮ謠帙�・
- `stdlib/tests/hash*.n.md` 縺ｨ `tests/selfhost_req.n.md`縲～nepl-core/tests/selfhost_req.rs` 縺ｮ import/險俶ｳ輔ｒ邨ｱ蜷亥ｾ梧ｧ区・縺ｫ蜷医ｏ縺帙※譖ｴ譁ｰ縲・

## 讀懆ｨｼ
- `NO_COLOR=false trunk build` -> pass
- wasm 蟇ｾ雎｡・・--no-stdlib --runner wasm`・・
  - `stdlib/tests/hash.n.md` / `hashmap.n.md` / `hashset.n.md` / `hashmap_str.n.md` / `hashset_str.n.md` / `tests/selfhost_req.n.md` -> 縺吶∋縺ｦ pass
- llvm 蟇ｾ雎｡・・--no-stdlib --runner llvm --llvm-all`・・
  - `stdlib/tests/hash.n.md` / `hashmap.n.md` / `hashset.n.md` / `hashmap_str.n.md` / `hashset_str.n.md` / `tests/selfhost_req.n.md` -> 縺吶∋縺ｦ pass

# 2026-02-27 菴懈･ｭ繝｡繝｢ (typecheck: get/put 迚ｹ蛻･蜃ｦ逅・・蜀崎ｪｿ譟ｻ)
## 螳滓命蜀・ｮｹ
- `nepl-core/src/typecheck.rs`
  - `TypeCtx::same` 蜻ｼ縺ｳ蜃ｺ縺励ｒ `resolve_id` 豈碑ｼ・∈菫ｮ豁｣・医ン繝ｫ繝我ｸ崎・縺ｮ逶ｴ謗･蜴溷屏繧定ｧ｣豸茨ｼ峨�・
  - `resolve_field_access` 繧定ｨｺ譁ｭ縺ゅｊ/縺ｪ縺励〒菴ｿ縺・・縺代ｉ繧後ｋ `resolve_field_access_with_mode` 縺ｫ蛻・屬縲・
  - `get/put` 迚ｹ蛻･蜃ｦ逅・ｒ縲掲ield 隗｣豎ｺ縺ｧ縺阪◆縺ｨ縺阪・縺ｿ驕ｩ逕ｨ縲∝､ｱ謨玲凾縺ｯ騾壼ｸｸ繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨∈繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ縲阪↓螟画峩縲・
  - `apply_function` 縺ｸ縺ｮ蝙句ｼ墓焚莨晄眺繧剃ｿｮ豁｣縺励�～reduce_calls*` 縺九ｉ縺ｯ `func_entry.type_args`・域・遉ｺ蝙句ｼ墓焚縺ｮ縺ｿ・峨ｒ貂｡縺吶ｈ縺・↓螟画峩縲・

## 迴ｾ蝨ｨ縺ｮ迥ｶ諷・
- `NO_COLOR=false trunk build` 縺ｯ騾夐℃縲・
- 縺溘□縺・`target/debug/nepl-cli --target wasi --profile debug --input /tmp/hm.nepl --output /tmp/hm-out` 縺ｧ
  `core/math.nepl` / `alloc/collections/vec.nepl` / `alloc/string.nepl` 縺ｮ `get` 蜻ｼ縺ｳ蜃ｺ縺励′
  `D3006` / `D3021` 縺ｧ螟ｱ謨励☆繧狗憾諷九′邯咏ｶ壹�・

## 蜴溷屏莉ｮ隱ｬ
- `get` 縺ｮ驕手ｲ�闕ｷ蛟呵｣懊′縺ゅｋ縺ｨ縺阪・繧ｷ繝ｳ繝懊Ν隗｣豎ｺ縺ｧ縲’ield 逕ｨ `get`・・core/field`・峨→ collections 蛛ｴ `get` 縺ｮ豺ｷ蝨ｨ縺ｫ繧医ｊ
  蜻ｼ縺ｳ蜃ｺ縺玲凾縺ｮ蛟呵｣懃ｵ槭ｊ霎ｼ縺ｿ縺悟｣翫ｌ縺ｦ縺・ｋ蜿ｯ閭ｽ諤ｧ縺碁ｫ倥＞縲・
- 迚ｹ縺ｫ `D3021`・・ype args mismatch・峨・縲∵・遉ｺ縺励※縺・↑縺・�ｴ髱｢縺ｧ蝙句ｼ墓焚邨瑚ｷｯ縺梧ｮ九▲縺ｦ縺・ｋ縺薙→繧堤､ｺ蜚・＠縺ｦ縺翫ｊ縲・
  `PrefixItem::Symbol` -> `StackEntry::type_args` -> `apply_function` 縺ｾ縺ｧ縺ｮ邨瑚ｷｯ繧定ｿｽ蜉�縺ｧ霑ｽ縺・ｿ・ｦ√′縺ゅｋ縲・

## 谺｡繧｢繧ｯ繧ｷ繝ｧ繝ｳ
- `get/put` 縺ｫ髯仙ｮ壹＠縺滓怙蟆上こ繝ｼ繧ｹ縺ｧ `StackEntry::type_args` 縺ｮ逕滓・/謳ｬ騾√ｒ繝医Ξ繝ｼ繧ｹ縲・
- `lookup_all_callables` 縺ｨ `lookup_all_any_defined` 縺ｮ繧ｹ繧ｳ繝ｼ繝怜━蜈郁ｦ丞援縺・
  field/collections 縺ｮ蜷悟錐隗｣豎ｺ繧貞｣翫＠縺ｦ縺・↑縺・°遒ｺ隱阪�・
- 譛�蟆丈ｿｮ豁｣縺ｧ `core/field get` 縺ｨ collections `get` 縺ｮ荳｡遶九ｒ蝗槫ｾｩ蠕後�・
  `stdlib/tests/hashmap*.n.md` 繧・wasm/llvm 逶ｴ蛻励〒蜀肴､懆ｨｼ縲・

## 霑ｽ險假ｼ・026-02-27・・
- 譬ｹ譛ｬ蜴溷屏:
  - 繧ｸ繧ｧ繝阪Μ繝・け髢｢謨ｰ繧・hoist 縺吶ｋ縺ｨ縺阪�～type_contains_unbound_var` 邨檎罰縺ｧ繧ｷ繝ｳ繝懊Ν蜷阪ｒ邏�縺ｮ髢｢謨ｰ蜷阪↓縺励※縺・◆縺溘ａ縲・
    蜷悟錐繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝会ｼ・get`・峨′蜷御ｸ�繧ｷ繝ｳ繝懊Ν縺ｫ陦晉ｪ√＠縺ｦ縺・◆縲・
  - 縺昴・邨先棡縲～HashMap` 迚・`get` 蜻ｼ縺ｳ蜃ｺ縺励′蛻･螳溯｣・∈隗｣豎ｺ縺輔ｌ縲～alias get failed` 繧定ｪ倡匱縺励※縺・◆縲・
- 菫ｮ豁｣:
  - `nepl-core/src/typecheck.rs` 縺ｮ hoist 縺ｧ縲√ず繧ｧ繝阪Μ繧ｯ繧ｹ譛臥┌縺ｫ髢｢菫ゅ↑縺・
    `mangle_function_symbol` 繧剃ｽｿ縺｣縺ｦ髢｢謨ｰ繧ｷ繝ｳ繝懊Ν繧剃ｸ�諢丞喧縺励◆縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` 騾夐℃縲・
  - `node nodesrc/tests.js -i stdlib/tests/hashmap.n.md -o /tmp/hashmap-focus-wasm.json --runner wasm --assert-io --no-tree -j 1` 騾夐℃・・06/206・峨�・
  - `node nodesrc/tests.js -i stdlib/tests/hashmap_str.n.md -o /tmp/hashmap-str-focus-wasm.json --runner wasm --assert-io --no-tree -j 1` 騾夐℃・・06/206・峨�・

# 2026-02-27 菴懈･ｭ繝｡繝｢ (kp 繧ｳ繝｡繝ｳ繝亥ｽ｢蠑上・邨ｱ荳�)
- 逶ｮ逧・
  - `//` 縺ｯ繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝医→縺励※謇ｱ繧上↑縺・婿驥昴↓蜷医ｏ縺帙�～stdlib/kp` 縺ｮ繧ｳ繝｡繝ｳ繝亥ｽ｢蠑上ｒ `//:` 縺ｫ邨ｱ荳�縺吶ｋ縲・
- 螳溯｣・
  - `stdlib/kp/kpread.nepl`
    - 陦碁�ｭ `//` 繧ｳ繝｡繝ｳ繝医ｒ `//:` 縺ｫ邨ｱ荳�縲・
    - 髢｢謨ｰ蜀・Κ縺ｮ陬懷勧繧ｳ繝｡繝ｳ繝郁｡鯉ｼ・OM蛻､螳壹・騾ｲ陦御ｿ晁ｨｼ繝ｻ蛻怜・譛溷喧縺ｪ縺ｩ・峨・蜑企勁縺励※縲・�壼ｸｸ繧ｳ繝ｼ繝峨・縺ｿ谿九☆讒区・縺ｫ謨ｴ逅・�・
  - `stdlib/kp/kpwrite.nepl`
    - 陦碁�ｭ `//` 繧ｳ繝｡繝ｳ繝医ｒ `//:` 縺ｫ邨ｱ荳�縲・
    - 髢｢謨ｰ蜀・Κ縺ｮ陦梧忰 `//` 繧ｳ繝｡繝ｳ繝医→陬懷勧繧ｳ繝｡繝ｳ繝郁｡後ｒ蜑企勁縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> pass
  - `NO_COLOR=false node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -o /tmp/tests-kp-io.json --runner wasm --assert-io --no-tree -j 1`
    -> `215/215 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (map襍ｷ轤ｹ縺ｮ蜷榊燕隗｣豎ｺ/繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝我ｿｮ豁｣)
## 譬ｹ譛ｬ蜴溷屏
- `typecheck` 縺ｮ隴伜挨蟄占ｧ｣豎ｺ縺ｧ縲∝酔蜷・callable 縺ｮ蟄伜惠縺後Ο繝ｼ繧ｫ繝ｫ蛟､・磯未謨ｰ蝙九ヱ繝ｩ繝｡繝ｼ繧ｿ・芽ｧ｣豎ｺ縺ｫ蟷ｲ貂峨＠縺ｦ縺・◆縲・
- `reduce_calls` / `apply_function` 縺・`Var(name)` 繧帝℃蠎ｦ縺ｫ callable 蜷阪→縺励※謇ｱ縺・�・
  繝ｭ繝ｼ繧ｫ繝ｫ髢｢謨ｰ蛟､蜻ｼ縺ｳ蜃ｺ縺暦ｼ・f a`・峨ｒ驕手ｲ�闕ｷ隗｣豎ｺ縺ｸ隱､騾√＠縺ｦ縺・◆縲・
- `lookup_all_callables` 縺悟・繧ｹ繧ｳ繝ｼ繝玲ｨｪ譁ｭ縺ｧ蛟呵｣懊ｒ霑斐＠縺ｦ縺翫ｊ縲∝・蛛ｴ螳夂ｾｩ縺ｫ繧医ｋ lexical shadowing 縺悟柑縺九★譖匁乂蛹悶＠縺ｦ縺・◆縲・

## 螳溯｣・
- `nepl-core/src/typecheck.rs`
  - head菴咲ｽｮ縺ｮ隴伜挨蟄占ｧ｣豎ｺ繧剃ｿｮ豁｣:
    - 蛟､縺碁未謨ｰ蝙九↑繧牙�､蜆ｪ蜈・
    - 蛟､縺碁撼髢｢謨ｰ縺ｪ繧・callable 蜆ｪ蜈・
  - `lookup_value_for_read` 蛟呵｣懊ｒ蜈医↓隧穂ｾ｡縺励�∝酔蜷・callable 豺ｷ蝨ｨ譎ゅ・驕ｸ謚櫁ｦ丞援繧貞ｮ牙ｮ壼喧縲・
  - `reduce_calls` / `reduce_calls_guarded` 縺ｮ `choose_callable_type_by_available_arity` 驕ｩ逕ｨ譚｡莉ｶ繧・
    縲悟酔蜷・value 縺悟ｭ伜惠縺励↑縺・�ｴ蜷医�阪↓髯仙ｮ壹�・
  - `apply_function` 縺ｮ騾壼ｸｸ callable 隗｣豎ｺ繧・
    縲悟酔蜷阪・髢｢謨ｰ蝙・value 縺悟ｭ伜惠縺吶ｋ蝣ｴ蜷医・騾壹ｉ縺ｪ縺・�阪ｈ縺・↓螟画峩・磯未謨ｰ蛟､蜻ｼ縺ｳ蜃ｺ縺励・ indirect 邨瑚ｷｯ縺ｸ・峨�・
  - `lookup_all_callables` 繧・lexical shadowing 蜆ｪ蜈茨ｼ域怙蜀・せ繧ｳ繝ｼ繝励・縺ｿ・峨∈螟画峩縲・
  - `let` 蝙区ｳｨ驥茨ｼ・pending_ascription`・峨°繧蛾未謨ｰ蛟､譛溷ｾ・ｒ諡ｾ縺・ｈ縺・↓縺励�・
    `let u <(i32)->i32> calc` 縺ｮ繧医≧縺ｪ譚溽ｸ帶凾隗｣豎ｺ繧貞ｮ牙ｮ壼喧縲・

## 繝・せ繝井ｿｮ豁｣
- `tests/generics.n.md`
  - `generics_make_pair_wrapper` 繧堤樟蝨ｨ縺ｮ蜑咲ｽｮ隧穂ｾ｡縺ｧ譖匁乂縺ｫ縺ｪ繧峨↑縺・ｧ区・縺ｸ謨ｴ逅・�・
- `tests/overload.n.md`
  - `overload_select_by_arity` 繧偵�後い繝ｪ繝・ぅ驕ｸ謚槭◎縺ｮ繧ゅ・縲阪ｒ讀懆ｨｼ縺吶ｋ譛�蟆乗ｧ区・縺ｸ謨ｴ逅・�・
  - `overload_select_by_arity_from_param_context_binary_not_supported_yet` 繧・
    螳溯｣・渚譏�貂医∩莉墓ｧ倥↓蜷医ｏ縺帙※騾壼ｸｸ `neplg2:test` 蛹悶�・

## 讀懆ｨｼ
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i tests/shadowing.n.md -o /tmp/tests-shadowing-now6.json --no-stdlib --no-tree` -> 27/27 pass
- `node nodesrc/tests.js -i tests/generics.n.md -o /tmp/tests-generics-now7.json --no-stdlib --no-tree` -> 24/24 pass
- `node nodesrc/tests.js -i tests/overload.n.md -o /tmp/tests-overload-now3.json --no-stdlib --no-tree` -> 18/18 pass
- `node nodesrc/tests.js -i tests -o /tmp/tests-tests-no-stdlib-final4.json --no-stdlib --no-tree` -> 471/471 pass
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-final.json --no-tree` -> 676/676 pass

# 2026-02-27 菴懈･ｭ繝｡繝｢ (hash map/set 蟾ｮ蛻・・蜀肴､懆ｨｼ)
## 螳滓命蜀・ｮｹ
- `stdlib/alloc/collections/hashmap.nepl`
  - `core/field` 縺ｮ蜿ら・繧・`field::get` 縺ｫ邨ｱ荳�縲・
  - i32 繧ｭ繝ｼ菴咲ｽｮ險育ｮ励ｒ `mod_s abs ...` 縺九ｉ `i32_rem_u` 縺ｫ邨ｱ荳�縲・
  - 髱槭ラ繧ｭ繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝・(`//`) 繧貞炎髯､縺励�～//:` 縺ｮ縺ｿ谿九☆讒区・縺ｸ謨ｴ逅・�・
- `stdlib/alloc/collections/hashset.nepl`
  - `core/field` 縺ｮ蜿ら・繧・`field::get` 縺ｫ邨ｱ荳�縲・
  - i32 繧ｭ繝ｼ菴咲ｽｮ險育ｮ励ｒ `mod_s abs ...` 縺九ｉ `i32_rem_u` 縺ｫ邨ｱ荳�縲・
  - 髱槭ラ繧ｭ繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝・(`//`) 繧貞炎髯､縺励�～//:` 縺ｮ縺ｿ谿九☆讒区・縺ｸ謨ｴ逅・�・
- `stdlib/alloc/hash/hash32.nepl`
  - `alloc/string` 繧・`string` alias 縺ｧ import 縺励�～string::len` 繧剃ｽｿ逕ｨ縺吶ｋ蠖｢縺ｫ邨ｱ荳�縲・
- `stdlib/tests/vec.n.md`
  - `push<u8> cast 65` 縺ｮ譖匁乂隗｣豎ｺ繧貞屓驕ｿ縺吶ｋ縺溘ａ縲～u8_65` 縺ｸ蛻・屬縺励※縺九ｉ `push<u8>` 縺ｫ貂｡縺吝ｽ｢縺ｸ菫ｮ豁｣縲・
- `tests/selfhost_req.n.md`
  - 蟇ｾ雎｡繧ｱ繝ｼ繧ｹ縺ｫ `#target std` 繧定ｿｽ蜉�縲・

## 讀懆ｨｼ
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i stdlib/tests/hash.n.md -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/hashset_str.n.md -o /tmp/tests-hash-related.json --no-tree`
  - `210/210 pass`
- `node nodesrc/tests.js -i tests/selfhost_req.n.md -i stdlib/tests/vec.n.md -o /tmp/tests-selfhost-vec.json --no-tree`
  - `212/212 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-regression.json --no-tree`
  - `676/676 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (sizeof / intrinsic 繝・せ繝域僑蠑ｵ)
## 螳滓命蜀・ｮｹ
- `tests/sizeof.n.md` 縺ｫ莉･荳九・繝・せ繝医ｒ霑ｽ蜉�:
  - `sizeof_collection_structs`
    - `Vec<i32>` / `Stack<i32>` / `HashMap<i32>` / `HashSet` 縺ｮ `size_of` 讀懆ｨｼ縲・
  - `sizeof_diag_structs`
    - `Span` / `Error` / `Diag` 縺ｮ `size_of` 讀懆ｨｼ縲・
- 譌｢蟄・`tests/intrinsic.n.md` 縺ｨ蜷医ｏ縺帙※ `size_of` 邉ｻ縺ｮ蝗槫ｸｰ讀懆ｨｼ繧ｻ繝・ヨ繧貞ｼｷ蛹悶�・

## 讀懆ｨｼ
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i tests/sizeof.n.md -i tests/intrinsic.n.md -o /tmp/tests-sizeof-intrinsic.json --no-tree`
  - `219/219 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-sizeof.json --no-tree`
  - `678/678 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (collections 縺ｮ Diag 繝・せ繝郁ｿｽ蜉�)
## 螳滓命蜀・ｮｹ
- `tests/collections_diag.n.md` 繧呈眠隕剰ｿｽ蜉�縲・
- 霑ｽ蜉�縺励◆讀懆ｨｼ:
  - `hashmap_remove` 縺ｮ譛ｪ蟄伜惠繧ｭ繝ｼ縺ｧ `KeyNotFound` 縺瑚ｿ斐ｋ縺薙→
  - `hashset_remove` 縺ｮ譛ｪ蟄伜惠繧ｭ繝ｼ縺ｧ `KeyNotFound` 縺瑚ｿ斐ｋ縺薙→
  - `hashmap_insert` 縺ｮ螳ｹ驥剰ｶ・℃縺ｧ `CapacityExceeded` 縺瑚ｿ斐ｋ縺薙→
  - `hashset_insert` 縺ｮ螳ｹ驥剰ｶ・℃縺ｧ `CapacityExceeded` 縺瑚ｿ斐ｋ縺薙→
- `diag_code_str d.code` 繧剃ｽｿ縺｣縺ｦ繧ｳ繝ｼ繝我ｸ�閾ｴ繧貞崋螳壼喧縲・

## 讀懆ｨｼ
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i tests/collections_diag.n.md -o /tmp/tests-collections-diag.json --no-tree`
  - `209/209 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-collections-diag.json --no-tree`
  - `682/682 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (alloc/diag 蜀崎ｨｭ險・ Diag/Error 騾｣謳ｺ + 繧ｳ繝｡繝ｳ繝亥ｽ｢蠑冗ｵｱ荳�)
## 螳滓命蜀・ｮｹ
- `stdlib/alloc/diag/error.nepl`
  - `DiagCode <-> ErrorKind` 縺ｮ逶ｸ莠貞・蜒・API 繧定ｿｽ蜉�:
    - `diag_code_to_error_kind`
    - `error_kind_to_diag_code`
  - `Diag <-> Error` 螟画鋤 API 繧定ｿｽ蜉�:
    - `diag_to_error`
    - `error_to_diag`
  - `Diag` 譁・ｭ怜・蛹悶ｒ `message` 霑泌唆縺ｸ螟画峩縺励�～Diag` 繝輔ぅ繝ｼ繝ｫ繝牙酔譎ょ盾辣ｧ縺ｮ move 遶ｶ蜷医ｒ隗｣豸医�・
  - 繝輔ぃ繧､繝ｫ蜀・・髱槭ラ繧ｭ繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝・`//` 繧・`//:` 縺ｫ邨ｱ荳�縲・
- `stdlib/alloc/diag/diag.nepl`
  - 繝輔ぃ繧､繝ｫ蜀・・髱槭ラ繧ｭ繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝・`//` 繧・`//:` 縺ｫ邨ｱ荳�縲・
- `stdlib/tests/error.n.md`
  - `diag_to_error` / `error_to_diag` 縺ｮ蠕�蠕ｩ繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縺励�∵悄蠕・�､繧貞崋螳壼喧縲・

## 譬ｹ譛ｬ蜴溷屏
- `Diag` 縺ｯ蛟､讒矩��菴薙〒縲～d.code` 縺ｨ `d.message` 縺ｮ蜷梧凾蜿ら・縺・move 遶ｶ蜷医ｒ襍ｷ縺薙＠縺ｦ縺・◆縲・
- `diag_to_error` 縺後％縺ｮ邨瑚ｷｯ繧堤峩謗･雕上ｓ縺ｧ縺・◆縺溘ａ compile fail 縺檎匱逕溘＠縺ｦ縺・◆縲・

## 讀懆ｨｼ
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i stdlib/tests/error.n.md -i stdlib/tests/diag.n.md -i tests/collections_diag.n.md -o /tmp/tests-diag-redesign-focus.json --no-tree`
  - `211/211 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-diag-redesign.json --no-tree`
  - `682/682 pass`

# 2026-02-27 菴懈･ｭ繝｡繝｢ (collections 螳牙・蛹悶ユ繧ｹ繝域僑蠑ｵ: queue/ringbuffer 遨ｺ謫堺ｽ・
## 螳滓命蜀・ｮｹ
- `tests/collections_diag.n.md` 縺ｫ莉･荳九ｒ霑ｽ蜉�:
  - `queue_pop_empty_returns_none`
  - `ringbuffer_pop_empty_returns_none`
- 逶ｮ逧・
  - 荳肴ｭ｣謫堺ｽ懶ｼ育ｩｺ繧ｳ繝ｬ繧ｯ繧ｷ繝ｧ繝ｳ縺九ｉ縺ｮ蜿悶ｊ蜃ｺ縺暦ｼ峨′ `Option::None` 縺ｧ螳牙・縺ｫ謇ｱ繧上ｌ繧九％縺ｨ繧貞崋螳壼喧縲・

## 讀懆ｨｼ
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i tests/collections_diag.n.md -i stdlib/tests/error.n.md -i stdlib/tests/diag.n.md -o /tmp/tests-collections-diag-next.json --no-tree`
  - `213/213 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-diag-and-collections.json --no-tree`
  - `684/684 pass`

# 2026-02-28 菴懈･ｭ繝｡繝｢ (List 繝ｩ繝・ヱ遘ｻ陦後・ moved 蛟､荳肴紛蜷井ｿｮ豁｣)
## 螳滓命蜀・ｮｹ
- `stdlib/tests/list.n.md` 縺ｮ `list_get` 讀懆ｨｼ縺ｧ縲～l3_0` 繧剃ｽ懈・縺励※縺・ｋ邂・園縺瑚ｪ､縺｣縺ｦ `l3` 繧貞盾辣ｧ縺励※縺・◆蝠城｡後ｒ菫ｮ豁｣縲・
- `stdlib/alloc/collections/list.nepl` 縺ｮ `List<.T>` 繝ｩ繝・ヱ遘ｻ陦後→謨ｴ蜷医☆繧九ｈ縺・�・未騾｣繝・せ繝・(`stdlib/tests/list.n.md`, `tests/pipe_collections.n.md`) 繧堤ｶｭ謖√＠縺溘∪縺ｾ moved 蛟､蜿ら・繧定ｧ｣豸医�・

## 譬ｹ譛ｬ蜴溷屏
- List API 繧・`i32` 髴ｲ蜃ｺ縺九ｉ `List<.T>` 繝ｩ繝・ヱ縺ｸ遘ｻ陦後＠縺滄圀縲√ユ繧ｹ繝亥・縺ｧ蜀肴ｧ狗ｯ峨＠縺溷�､譚溽ｸ・(`l3_0`, `l3_1`, ...) 縺ｨ譌ｧ譚溽ｸ帛錐 (`l3`) 縺梧ｷｷ蝨ｨ縺励◆縺ｾ縺ｾ谿九ｊ縲［ove 蠕悟､画焚繧貞盾辣ｧ縺吶ｋ蠖｢縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・

## 讀懆ｨｼ
- `NO_COLOR=false trunk build` -> pass
- `node nodesrc/tests.js -i stdlib/tests/list.n.md -i tests/pipe_collections.n.md -i tests/list_dot_map.n.md -i tests/neplg2.n.md -o /tmp/tests-list-migration-focus.json --no-tree`
  - `260/260 pass`
- `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-full-after-list-wrapper.json --no-tree`
  - `684/684 pass`
# 2026-03-03 菴懈･ｭ繝｡繝｢ (parser 險ｺ譁ｭID縺ｮ譏守､ｺ莉倅ｸ弱ｒ諡｡蠑ｵ)
- 逶ｮ逧・
  - parser 縺ｮ `if/while layout` 縺ｨ `#wasm/#llvmir` 繝悶Ο繝・け縺ｧ譛ｪ莉倅ｸ弱□縺｣縺溯ｨｺ譁ｭID繧呈・遉ｺ蛹悶＠縲～compile_fail diag_id` 縺ｮ螳牙ｮ壽�ｧ繧剃ｸ翫￡繧九�・
- 螳溯｣・
  - `nepl-core/src/parser.rs`
    - `expected wasm text line` / `expected llvm ir text line` 縺ｫ `ParserExpectedToken (2001)` 繧剃ｻ倅ｸ弱�・
    - `if-layout` 縺ｮ `invalid marker` / `invalid marker order` / `duplicate marker` / `too many expressions` 縺ｫ `ParserUnexpectedToken (2002)` 繧剃ｻ倅ｸ弱�・
    - `if-layout` 縺ｮ `missing expression(s)` 縺ｫ `ParserExpectedToken (2001)` 繧剃ｻ倅ｸ弱�・
    - `while-layout` 縺ｮ蜷檎ｨｮ繧ｨ繝ｩ繝ｼ縺ｫ `ParserUnexpectedToken (2002)` / `ParserExpectedToken (2001)` 繧剃ｻ倅ｸ弱�・
    - `argument layout` 縺ｮ `only expressions are allowed` 縺ｫ `ParserUnexpectedToken (2002)`縲～must contain expressions` 縺ｫ `ParserExpectedToken (2001)` 繧剃ｻ倅ｸ弱�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build --release --public-url /NEPLg2/` -> pass
  - `node tests/tree/run.js` -> `18/18 pass`
  - `node nodesrc/tests.js -i tests/if.n.md -i tests/while.n.md --no-stdlib --no-tree --runner all --llvm-all --assert-io --strict-dual -o /tmp/tests-if-while-diag.json -j 2` -> `170/170 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -o /tmp/tests-dual-full.json --runner all --llvm-all --assert-io --strict-dual -j 2` -> `1876/1876 pass`

# 2026-03-03 菴懈･ｭ繝｡繝｢ (prefix 蟒・ｭ｢遘ｻ陦・ math/kp/stdio 縺ｮ蜈･繧悟ｭ仙ｼ上ｒ謇倶ｿｮ豁｣)
- 逶ｮ逧・
  - `i32_` 遲・prefix 蟒・ｭ｢譁ｹ驥昴↓蜷医ｏ縺帙※縲∵尠譏ｧ縺ｪ蜈･繧悟ｭ・prefix 蜻ｼ縺ｳ蜃ｺ縺励ｒ謇倶ｽ懈･ｭ縺ｧ蛻・ｧ｣縺励�∝梛豕ｨ驥・繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝芽ｧ｣豎ｺ縺ｧ騾壹ｋ蠖｢縺ｸ遘ｻ陦後☆繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - 譌ｧ蠑上・ `add a add b c` / `store_u8 add buf add off i ...` 蠖｢蠑上′縲｝refix 蟒・ｭ｢騾比ｸｭ縺ｮ繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝芽ｧ｣豎ｺ縺ｧ `no matching overload` 繧定ｪ倡匱縲・
  - 荳�驛ｨ縺ｯ繝ｭ繝ｼ繧ｫ繝ｫ螟画焚蜷・`neg` 縺碁未謨ｰ `neg` 縺ｨ陦晉ｪ√＠縺ｦ隱､隗｣豎ｺ繧堤匱逕溘�・
- 螳溯｣・
  - `stdlib/core/math.nepl`
    - `u128_add/sub`, `i128_add/sub`, `u64_mul_wide`, `i128_mul` 縺ｮ蜈･繧悟ｭ仙ｼ上ｒ谿ｵ髫主､画焚縺ｫ蛻・ｧ｣縲・
    - `add/sub/mul` 縺ｮ `i128` 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨ｒ霑ｽ蜉�縲・
    - `u8` 邉ｻ (`add/sub/mul/div_u/rem_u/eq/ne/lt_u/le_u/gt_u/ge_u`) 縺ｮ prefix 縺ｪ縺励が繝ｼ繝舌・繝ｭ繝ｼ繝峨ｒ霑ｽ蜉�縲・
  - `stdlib/core/mem.nepl`
    - `align8` 縺ｮ蜈･繧悟ｭ千ｮ苓｡薙ｒ蛻・ｧ｣縲・
  - `stdlib/alloc/string.nepl`
    - 謨ｰ蛟､繝代・繧ｹ/譁・ｭ怜・蛹悶・蜈･繧悟ｭ仙ｼ上ｒ谿ｵ髫主､画焚縺ｫ蛻・ｧ｣縲・
    - `neg` 螟画焚縺ｨ `neg` 髢｢謨ｰ縺ｮ陦晉ｪ∫ｮ・園繧・`sub 0 x` 譁ｹ蠑上↓鄂ｮ謠帙�・
  - `stdlib/std/stdio.nepl`
    - `read_line` / `print_i32` 蜻ｨ霎ｺ縺ｮ繝昴う繝ｳ繧ｿ險育ｮ励ｒ谿ｵ髫主､画焚縺ｫ蛻・ｧ｣縲・
  - `stdlib/kp/kpread.nepl`, `stdlib/kp/kpwrite.nepl`, `stdlib/kp/kpsearch.nepl`
    - 繝昴う繝ｳ繧ｿ險育ｮ励・譯∝・逅・・莠悟・謗｢邏｢/unique蜃ｦ逅・・蜈･繧悟ｭ仙ｼ上ｒ谿ｵ髫主､画焚縺ｫ蛻・ｧ｣縲・
  - `tests/math.n.md`, `tests/numerics.n.md`, `tests/overload.n.md`, `tests/typeannot.n.md`, `tests/kp.n.md`
    - 譁ｰ隕冗ｴ・ｼ・refix 縺ｪ縺・+ 蠢・ｦ∫ｮ・園縺ｮ蝙区ｳｨ驥・谿ｵ髫主､画焚・峨↓譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/math.n.md -i tests/numerics.n.md -i tests/overload.n.md -i tests/typeannot.n.md -i tests/kp.n.md -i tests/intrinsic.n.md --no-stdlib --runner wasm --assert-io --no-tree -o /tmp/tests-prefix-migration-focus.json -j 1`
    - `59/59 pass`

# 2026-03-03 菴懈･ｭ繝｡繝｢ (prefix蟒・ｭ｢遘ｻ陦・ cast 險俶ｳ慕ｵｱ荳�縺ｮ邯咏ｶ・
- 譁ｹ驥・
  - `cast<T>` 縺ｯ菴ｿ繧上★縲～<T> cast expr`・医∪縺溘・ `let x <T> cast expr`・峨↓邨ｱ荳�縲・
  - `i32_`/`i64_` 縺ｪ縺ｩ prefix 蜻ｼ縺ｳ蜃ｺ縺励・蜑頑ｸ帙ｒ縲∝他縺ｳ蜃ｺ縺怜・縺九ｉ谿ｵ髫守噪縺ｫ騾ｲ繧√ｋ縲・
- 螳溯｣・
  - `stdlib/kp/kpwrite.nepl`: 螟画鋤蜻ｼ縺ｳ蜃ｺ縺励ｒ `cast` 蠖｢蠑上∈譖ｴ譁ｰ縲・
  - `stdlib/kp/kpread.nepl`: u64/i64/f64/f32 隱ｭ縺ｿ蜿悶ｊ邉ｻ縺ｮ螟画鋤繧・`cast` 蠖｢蠑上∈譖ｴ譁ｰ縲・
  - `stdlib/std/fs.nepl`, `stdlib/std/env/cliarg.nepl`: syscall 蠑墓焚螟画鋤繧・`cast` 蠖｢蠑上∈譖ｴ譁ｰ縲・
  - `stdlib/alloc/string.nepl`: `from_i64`/`to_i64`/`from_f64`/`to_f64`/`from_f32`/`to_f32` 縺ｮ螟画鋤繧・`cast` 蠖｢蠑上∈譖ｴ譁ｰ縲・
  - `stdlib/std/test.nepl`: `test_str_eq_loop` 縺ｮ `add a add 4 i` 蠖｢繧・`off` 蜈郁ｨ育ｮ励∈螟画峩縺励�√が繝ｼ繝舌・繝ｭ繝ｼ繝芽ｧ｣豎ｺ螟ｱ謨励ｒ譬ｹ譛ｬ蝗樣∩縲・
  - `tests/kp.n.md`, `tests/intrinsic.n.md`, `tutorials/getting_started/24_competitive_dp_basics.n.md`, `tutorials/getting_started/27_competitive_algorithms_catalog.n.md` 繧呈眠險俶ｳ輔∈譖ｴ譁ｰ縲・
  - `tests/typeannot.n.md`: 縲碁㍾縺ｭ豕ｨ驥医・莉墓ｧ倅ｸ雁庄閭ｽ縺�縺悟・髟ｷ縲阪・隱ｬ譏弱∈譖ｴ譁ｰ・医こ繝ｼ繧ｹ閾ｪ菴薙・邯ｭ謖・ｼ峨�・
- 讀懆ｨｼ:
  - `/tmp/tests-prefix-migration-focus2.json` : 59/59 pass
  - `/tmp/tests-cast-annotation-style.json` : 43/43 pass
  - `/tmp/tests-kp-after-kpread-cast.json` : 7/7 pass
  - `/tmp/tests-std-fs-cliarg-cast-focused.json` : 11/11 pass
  - `/tmp/tests-string-cast-migration.json` : 29/29 pass
# 2026-03-03 菴懈･ｭ繝｡繝｢ (math萓晏ｭ伜・縺ｮprefix邵ｮ騾�: std/test繝ｻstd/fs繝ｻtree險ｺ譁ｭ繝・せ繝・
- 逶ｮ逧・
  - `蝙句錐_` prefix 蟒・ｭ｢譁ｹ驥昴↓蜷医ｏ縺帙�～math.nepl` 萓晏ｭ伜・縺ｮ蜻ｽ蜷阪→蛻ｩ逕ｨ繧・`蝙区ｳｨ驥・+ cast` / 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨∈蟇・○繧九�・
- 螳溯｣・
  - `stdlib/std/test.nepl`
    - `bool_to_str` / `i32_to_str` 繧貞ｻ・ｭ｢縺励�～to_str` 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝・(`(bool)->str`, `(i32)->str`) 縺ｫ邨ｱ荳�縲・
    - 螟ｱ謨励Γ繝・そ繝ｼ繧ｸ讒狗ｯ峨〒縺ｮ蜻ｼ縺ｳ蜃ｺ縺励ｒ `to_str` 縺ｸ譖ｴ譁ｰ縲・
  - `stdlib/std/fs.nepl`
    - `i64_from_i32` 繝倥Ν繝代ｒ蜑企勁縺励�∽ｽｿ逕ｨ邂・園繧・`cast` 縺ｫ鄂ｮ謠帙�・
  - `stdlib/kp/kpwrite.nepl`
    - doctest 萓九・ `i64_extend_i32_u` 繧・`<i64> cast` 縺ｸ譖ｴ譁ｰ縲・
  - `tests/tree/05_overload_shadow_diagnostics.js`
    - `i32_ne` 繧・`ne` 縺ｸ譖ｴ譁ｰ・医が繝ｼ繝舌・繝ｭ繝ｼ繝芽ｧ｣豎ｺ蜑肴署縺ｮ譁ｰ隕冗ｴ・ｼ峨�・
  - `tests/tree/18_diagnostic_ids.js`
    - `i32_to_f32` 繧・`<f32> cast` 縺ｸ譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `node tests/tree/run.js` -> `18/18 pass`縲・
  - `nodesrc/tests.js` 縺ｮ蟇ｾ雎｡髯仙ｮ壼ｮ溯｡後・髟ｷ譎る俣縺ｧ繧ｿ繧､繝�繧｢繧ｦ繝医☆繧区嫌蜍輔ｒ遒ｺ隱阪＠縺溘◆繧√�∫樟譎らせ縺ｯ tree 繧ｹ繧､繝ｼ繝医ｒ蜆ｪ蜈医＠縺ｦ蝗槫ｸｰ遒ｺ隱阪�・

# 2026-03-03 菴懈･ｭ繝｡繝｢ (bit貍皮ｮ輸PI縺ｮprefix邵ｮ騾�)
- 逶ｮ逧・
  - `core/math` 縺ｮ bit 貍皮ｮ励↓縺､縺・※繧・`蝙句錐_` 縺ｪ縺励〒菴ｿ縺医ｋ邨瑚ｷｯ繧定ｿｽ蜉�縺吶ｋ縲・
- 螳溯｣・
  - `stdlib/core/math.nepl`
    - `rotl/rotr/clz/ctz/popcnt` 縺ｮ i32/i64 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨ｒ霑ｽ蜉�・亥・驛ｨ縺ｯ譌｢蟄・`i32_*` / `i64_*` 螳溯｣・∈蟋碑ｭｲ・峨�・
  - `stdlib/tests/math.n.md`
    - `i32_clz/i32_ctz` 蜻ｼ縺ｳ蜃ｺ縺励ｒ `clz/ctz` 蜻ｼ縺ｳ蜃ｺ縺励∈譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-stdlib-math-prefixless-only.json -j 1`
    - `1/1 pass`

# 2026-03-03 菴懈･ｭ繝｡繝｢ (cast萓晏ｭ倥・螟画鋤API繧恥refix縺ｪ縺怜錐縺ｸ霑ｽ蠕・
- 逶ｮ逧・
  - `core/cast` 縺・`core/math` 縺ｮ `蝙句錐_` 螟画鋤蜷阪∈逶ｴ謗･萓晏ｭ倥＠縺ｪ縺・ｽ｢縺ｸ蟇・○繧九�・
- 螳溯｣・
  - `stdlib/core/math.nepl`
    - 螟画鋤逕ｨ縺ｮprefix縺ｪ縺励お繝ｳ繝医Μ繧定ｿｽ蜉�:
      - `extend_s`, `wrap`, `convert_s`, `trunc_s`, `promote`, `demote`, `to_i128`
    - `u128/i128` 螳溯｣・・縺ｮ `i64_extend_i32_u/s` 蛻ｩ逕ｨ繧・`cast` 縺ｫ鄂ｮ謠帙�・
  - `stdlib/core/cast.nepl`
    - `cast_i32_to_i64` 縺ｪ縺ｩ縺ｮ螳溯｣・悽菴薙ｒ荳願ｨ湾refix縺ｪ縺鈴未謨ｰ蜻ｼ縺ｳ蜃ｺ縺励∈螟画峩縲・
  - `from_i64` 蜷阪・ `alloc/string.nepl` 縺ｮ `from_i64`・・mpure・峨→陦晉ｪ√＠縲～pure context cannot call impure function` 繧定ｪ倡匱縺励◆縺溘ａ縲～to_i128` 縺ｫ謾ｹ蜷阪＠縺ｦ譬ｹ譛ｬ隗｣豸医�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-cast-prefixless.json -j 1`
    - `2/2 pass`

# 2026-03-03 菴懈･ｭ繝｡繝｢ (math: u32/u64/u128/i128 API 縺ｮprefix邵ｮ騾�)
- 逶ｮ逧・
  - `蝙句錐_` prefix 蟒・ｭ｢譁ｹ驥昴↓蜷医ｏ縺帙�～u32_/u64_/u128_/i128_` 蜈ｬ髢帰PI蜷阪ｒ蜑頑ｸ帙☆繧九�・
- 螳溯｣・
  - `stdlib/core/math.nepl`
    - `u32_*` / `u64_*` 蜈ｬ髢矩未謨ｰ鄒､繧貞炎髯､縲・
    - `u128`:
      - `u128_new` -> `new <(i64,i64)->u128>`
      - `u128_from_u64` -> `to_u128`
      - `u128_add/sub/lt` -> `add/sub/lt` 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝・
    - `i128`:
      - `i128_new` -> `new <(i64,i64)->i128>`
      - `i128_from_i64` -> `to_i128`
      - `i128_add/sub/mul/lt` -> `add/sub/mul/lt` 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝・
    - `u64_mul_wide` -> `mul_wide` 縺ｫ螟画峩縲・
    - `f32_*/f64_*` 縺ｮ蝓ｺ譛ｬ貍皮ｮ怜錐繧・`sqrt/abs/ceil/floor/trunc/nearest/min/max/copysign` 縺ｮ繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝牙錐縺ｫ邨ｱ荳�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-cast-prefixless-v3.json -j 1`
    - `2/2 pass`

# 2026-03-03 菴懈･ｭ繝｡繝｢ (cast API縺ｮ繝倥Ν繝代・蜷阪ｒ蟒・ｭ｢縺励※繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝画悽菴薙∈邨ｱ荳�)
- 逶ｮ逧・
  - `cast_i32_to_*` 邉ｻ繝倥Ν繝代・蜷阪ｒ蟒・ｭ｢縺励�～cast` 縺ｮ繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝画悽菴薙□縺代〒驕狗畑縺吶ｋ縲・
- 螳溯｣・
  - `stdlib/core/cast.nepl`
    - `fn cast cast_*` alias 鄒､繧貞炎髯､縲・
    - 縺吶∋縺ｦ `fn cast <(A)->B>` 蠖｢蠑上・逶ｴ謗･螳夂ｾｩ縺ｸ邨ｱ荳�縲・
  - `stdlib/tests/cast.n.md`
    - 譌ｧ繝倥Ν繝代・蜻ｼ縺ｳ蜃ｺ縺暦ｼ・cast_bool_to_i32`, `cast_i32_to_bool`・峨ｒ蜑企勁縺励�～cast` + 蜊倅ｸ�蝙区ｳｨ驥医∈譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-cast-prefixless-v4.json -j 1`
    - `2/2 pass`

# 2026-03-03 菴懈･ｭ繝｡繝｢ (math.nepl: i64螳壽焚縺ｮ譬ｹ譛ｬ菫ｮ豁｣)
- 逶ｮ逧・
  - `蝙句錐_` prefix 蟒・ｭ｢遘ｻ陦御ｸｭ縺ｫ逋ｺ逕溘＠縺・`core/math` 縺ｮ螟ｧ驥丞梛蟠ｩ繧後ｒ譬ｹ譛ｬ隗｣豸医☆繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - `math.nepl` 蠕悟濠・・128/i128螳溯｣・ｼ峨〒 `cast` 繧堤峩謗･菴ｿ縺｣縺ｦ縺・◆縺後�～core/math` 縺ｧ縺ｯ `core/cast` 繧・import 縺励※縺・↑縺・◆繧・`cast` 縺梧悴螳夂ｾｩ縲・
  - 縺輔ｉ縺ｫ `<i64> 0` 縺ｮ蝙区ｳｨ驥医・縲悟梛荳�閾ｴ繝√ぉ繝・け縲阪〒縺ゅｊ證鈴ｻ吝､画鋤縺ｧ縺ｯ縺ｪ縺・◆繧√�（32 繝ｪ繝・Λ繝ｫ繧・i64 縺ｫ縺ｧ縺阪★ `D3004` 縺碁�｣骼悶＠縺溘�・
- 菫ｮ豁｣:
  - `u128/i128/mul_wide` 縺ｮ蜈ｨ i64 螳壽焚逕滓・繧・`extend_s_i32_to_i64` 縺ｫ邨ｱ荳�縲・
  - `cast` 萓晏ｭ倥ｒ `math.nepl` 螳溘さ繝ｼ繝峨°繧蛾勁蜴ｻ縺励�～core/math` 蜊倅ｽ薙〒閾ｪ蟾ｱ螳檎ｵ舌☆繧狗憾諷九∈謌ｻ縺励◆縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md -i tests/math.n.md -i tests/typeannot.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-scope-no-stdlib.json -j 1`
  - 邨先棡: `19/19 pass`

# 2026-03-03 菴懈･ｭ繝｡繝｢ (math.nepl: u8 prefix螳滉ｽ薙・邵ｮ騾�)
- 逶ｮ逧・
  - `蝙句錐_` prefix 蟒・ｭ｢譁ｹ驥昴↓蜷医ｏ縺帙�～u8_*` 螳滉ｽ馴未謨ｰ蜷阪ｒ prefix 蜈磯�ｭ縺ｪ縺励∈邨ｱ荳�縺吶ｋ縲・
- 螳溯｣・
  - `u8_add/sub/mul/div_u/rem_u/eq/ne/lt_u/le_u/gt_u/ge_u` 繧・
    `add_u8/sub_u8/mul_u8/div_u_u8/rem_u_u8/eq_u8/ne_u8/lt_u_u8/le_u_u8/gt_u_u8/ge_u_u8` 縺ｸ螟画峩縲・
  - `fn add/sub/... <(u8,u8)->...>` 縺ｮ蜈ｬ髢九が繝ｼ繝舌・繝ｭ繝ｼ繝峨・譁ｰ螳滉ｽ灘錐縺ｸ蟋碑ｭｲ縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md -i tests/math.n.md -i tests/typeannot.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-scope-no-stdlib.json -j 1`
  - 邨先棡: `19/19 pass`

# 2026-03-03 菴懈･ｭ繝｡繝｢ (math.nepl: 蜀鈴聞縺ｪ莠碁㍾蝙区ｳｨ驥医・謨ｴ逅・
- 逶ｮ逧・
  - 譁ｰ隕冗ｴ・↓蜷医ｏ縺帙※ `math.nepl` 繝峨く繝･繝｡繝ｳ繝亥・縺ｮ莠碁㍾豕ｨ驥・(`<i64> <i64> cast` 遲・ 繧帝勁蜴ｻ縺吶ｋ縲・
- 螳溯｣・
  - `math.nepl` 蜀・・ `<i64> <i64> cast` / `<f64> <f64> cast` 繧・`<i64> cast` / `<f64> cast` 縺ｸ邨ｱ荳�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md -i tests/math.n.md -i tests/typeannot.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-scope-no-stdlib.json -j 1`
  - 邨先棡: `19/19 pass`

# 2026-03-03 菴懈･ｭ繝｡繝｢ (tutorial: 謨ｰ蛟､遶�縺ｮ譖匁乂繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝牙ｯｾ遲・
- 逶ｮ逧・
  - `math` 縺ｮ繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝画僑蠑ｵ・・8 邉ｻ邨ｱ蜷茨ｼ峨↓繧医ｊ縲√メ繝･繝ｼ繝医Μ繧｢繝ｫ縺ｮ遏ｭ縺・焚蛟､蠑上〒逋ｺ逕溘＠縺滓尠譏ｧ隗｣豎ｺ繧定ｧ｣豸医☆繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - 蟆上＆縺・紛謨ｰ繝ｪ繝・Λ繝ｫ縺�縺代〒讒区・縺輔ｌ縺溷粋謌仙ｼ上′縲～i32`/`u8` 縺ｮ蛟呵｣懊〒譖匁乂蛹悶＠縺溘�・
- 菫ｮ豁｣:
  - `tutorials/getting_started/02_numbers_and_variables.n.md`
    - 隍・粋蠑上ｒ荳ｭ髢・`let` 縺ｫ蛻・ｧ｣縺励�∵尠譏ｧ縺ｪ繝ｪ繝・Λ繝ｫ縺ｫ `<i32>` 豕ｨ驥医ｒ莉倅ｸ弱�・
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
    - 莠悟・謗｢邏｢縺ｮ `mid` 險育ｮ励ｒ `sum`/`mv_off`/`mv_ptr` 縺ｸ蛻・ｧ｣縺励※蝙玖ｧ｣豎ｺ繧貞ｮ牙ｮ壼喧縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tutorials/getting_started/02_numbers_and_variables.n.md -i tutorials/getting_started/03_functions.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-tutorial-math-scope.json -j 1`
  - 邨先棡: `14/14 pass`

# 2026-03-03 菴懈･ｭ繝｡繝｢ (math.nepl: 谿句ｭ湾refix譁・ｭ怜・縺ｮ邨ｱ荳�)
- 逶ｮ逧・
  - `蝙句錐_` prefix 蟒・ｭ｢譁ｹ驥昴↓蜷医ｏ縺帙�～math.nepl` 蜀・・谿句ｭ・prefix 譁・ｭ怜・・医ラ繧ｭ繝･繝｡繝ｳ繝郁ｦ句・縺励・LLVM 繧ｷ繝ｳ繝懊Ν蜷搾ｼ峨ｂ邨ｱ荳�縺吶ｋ縲・
- 螳溯｣・
  - `u8_*` 陦ｨ險倥ｒ `*_u8` 縺ｸ邨ｱ荳�・医さ繝｡繝ｳ繝郁｡ｨ險倥・`#llvmir` 蜀・す繝ｳ繝懊Ν蜷阪ｒ蜷ｫ繧�・峨�・
  - `f32_*` / `f64_*` 陦ｨ險倥ｒ `*_f32` / `*_f64` 縺ｸ邨ｱ荳�・医さ繝｡繝ｳ繝郁｡ｨ險倥・`#llvmir` 蜀・す繝ｳ繝懊Ν蜷阪ｒ蜷ｫ繧�・峨�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i tests/math.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-post-rename.json -j 1` -> `6/6 pass`
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i stdlib/tests/cast.n.md -i stdlib/tests/vec.n.md -i tests/math.n.md -i tests/typeannot.n.md -i tutorials/getting_started/02_numbers_and_variables.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-migration-bundle.json -j 1` -> `28/28 pass`

# 2026-03-03 菴懈･ｭ繝｡繝｢ (vec/sort 縺ｨ tutorial 縺ｮ譁ｰ隕冗ｴ・紛蛯・
- 逶ｮ逧・
  - `蝙句錐_` prefix 蟒・ｭ｢譁ｹ驥昴↓蜷医ｏ縺帙�～alloc/collections/vec/sort.nepl` 縺ｮ譖匁乂蠑上ｒ隗｣豸医＠縲》utorial 蛛ｴ繧偵Λ繧､繝悶Λ繝ｪ蛻ｩ逕ｨ縺ｸ譖ｴ譁ｰ縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - `vec/sort.nepl` 縺ｫ `op op ...` 縺ｮ蜈･繧悟ｭ仙燕鄂ｮ蠑上′谿九▲縺ｦ縺翫ｊ縲√が繝ｼ繝舌・繝ｭ繝ｼ繝牙�呵｣懷｢怜刈蠕後↓ `D3006` 繧定ｪ倡匱縺励※縺・◆縲・
  - tutorial 縺ｮ sort 遶�縺ｯ閾ｪ蜑肴諺蜈･繧ｽ繝ｼ繝亥ｮ溯｣・□縺｣縺溘◆繧√�∫樟蝨ｨ縺ｮ stdlib 繧剃ｽｿ縺・ｵ√ｌ縺ｨ荵夜屬縺励※縺・◆縲・
  - `sort_quick` 縺ｯ `Vec` 繧呈ｶ郁ｲｻ縺吶ｋ縺溘ａ縲》utorial 縺ｧ蜷御ｸ�螟画焚繧貞ｾ檎ｶ壼盾辣ｧ縺吶ｋ縺ｨ move 繧ｨ繝ｩ繝ｼ縺檎匱逕溘＠縺溘�・
- 菫ｮ豁｣:
  - `stdlib/alloc/collections/vec/sort.nepl`
    - `sort_comb` / `sort_heap_sift_down_data` / `sort_heap` / `sort_merge_range_data` / `sort_heap_ret` 縺ｮ譖匁乂縺ｪ蜈･繧悟ｭ仙ｼ上ｒ荳ｭ髢・`let` 縺ｧ蛻・ｧ｣縲・
    - `u8` 縺ｮ `Ord::lt` 繧・`cast` 蠕梧ｯ碑ｼ・∈譏守､ｺ蛹悶�・
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
    - 蜈磯�ｭ遶�繧定・蜑肴諺蜈･繧ｽ繝ｼ繝医°繧・`alloc/collections/vec` + `alloc/collections/vec/sort` 蛻ｩ逕ｨ萓九∈鄂ｮ謠帙�・
    - `sort_quick_ret` 繧剃ｽｿ逕ｨ縺励※ move 繧ｨ繝ｩ繝ｼ繧貞屓驕ｿ縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tutorials/getting_started/23_competitive_sort_and_search.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-tut23-no-stdlib.json -j 1` -> `3/3 pass`
  - `node nodesrc/tests.js -i stdlib/tests/math.n.md -i tests/math.n.md -i tests/typeannot.n.md -i tutorials/getting_started/02_numbers_and_variables.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-math-migration-scope.json -j 1` -> `29/29 pass`

# 2026-03-03 菴懈･ｭ繝｡繝｢ (heap/linear memory 螳牙・蛹悶・谿ｵ髫主ｰ主・)
- 逶ｮ逧・
  - `mem.nepl` / `kpread.nepl` / `kpwrite.nepl` 縺ｧ逕溘・繧､繝ｳ繧ｿ `i32` 縺ｮ髴ｲ蜃ｺ繧呈ｸ帙ｉ縺励�∵ｮｵ髫守噪縺ｫ蟆ら畑蝙九∈遘ｻ陦後☆繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - `Scanner` / `Writer` 繧・`struct` 蛹悶＠縺ｦ蜈ｬ髢・API 繧堤峩謗･鄂ｮ謠帙☆繧九→縲¨EPL 縺ｮ move 隕丞援縺ｧ繝上Φ繝峨Ν蜀榊茜逕ｨ譎ゅ↓ `use of moved value` 縺檎匱逕溘☆繧九�・
  - `*` 繧貞､悶☆縺ｨ impure 蜻ｼ縺ｳ蜃ｺ縺怜宛邏・(`pure context cannot call impure function`) 縺ｫ謚ｵ隗ｦ縺吶ｋ縲・
- 菫ｮ豁｣:
  - `stdlib/core/mem.nepl`
    - `MemPtr` 繧定ｿｽ蜉�縺励�～alloc_ptr` / `realloc_ptr` / `dealloc_ptr` / `mem_ptr_add` 繧定ｿｽ蜉�縲・
    - `load_i32_ptr` / `store_i32_ptr` / `load_u8_ptr` / `store_u8_ptr` 繧定ｿｽ蜉�・域里蟄・`load_i32` 遲峨・蜷榊燕陦晉ｪ√ｒ蝗樣∩・峨�・
  - `stdlib/kp/kpread.nepl`
    - `Scanner` 蝙九→ `scanner_wrap` / `scanner_raw` / `scanner_new_typed` 繧定ｿｽ蜉�縲・
    - 譌｢蟄伜・髢・API (`scanner_new` 縺ｨ蜷・read) 縺ｯ `i32` 繝吶・繧ｹ縺ｮ縺ｾ縺ｾ邯ｭ謖√＠縺ｦ遐ｴ螢顔噪蠖ｱ髻ｿ繧貞屓驕ｿ縲・
  - `stdlib/kp/kpwrite.nepl`
    - `Writer` 蝙九→ `writer_wrap` / `writer_raw` / `writer_new_typed` 繧定ｿｽ蜉�縲・
    - 譌｢蟄伜・髢・API (`writer_new` 縺ｨ蜷・write) 縺ｯ `i32` 繝吶・繧ｹ縺ｮ縺ｾ縺ｾ邯ｭ謖√�・
  - 蠖ｱ髻ｿ繝・せ繝育ｾ､・・kp` / tutorial・峨〒蝙区ｳｨ驥医ｒ荳�譎ょｰ主・縺励※縺・◆邂・園縺ｯ `i32` 縺ｫ謌ｻ縺励�～25_competitive_prefixsum_twopointers.n.md` 縺ｮ譖匁乂縺ｪ蜈･繧悟ｭ仙燕鄂ｮ蠑上ｒ荳ｭ髢・`let` 螻暮幕縺ｧ隗｣豸医�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md --runner wasm --assert-io --no-stdlib --no-tree -o /tmp/tests-kp-typed-handles.json -j 1`
  - 邨先棡: `21/21 pass`
- 蟾ｮ蛻・婿驥・
  - 迴ｾ譎らせ縺ｯ縲碁撼遐ｴ螢翫〒縺ｮ螳牙・蛹冶ｶｳ蝣ｴ・・yped API 菴ｵ險ｭ・峨�阪∪縺ｧ縲・
  - 蜈ｬ髢・API 繧貞ｮ悟・縺ｫ蟆ら畑蝙九∈遘ｻ陦後☆繧九↓縺ｯ縲［ove 隕丞援縺ｫ豐ｿ縺｣縺溘ワ繝ｳ繝峨Ν蜀肴據邵帙ヱ繧ｿ繝ｼ繝ｳ・・onsume/return・峨ｒ讓呎ｺ門喧縺励※縺九ｉ谿ｵ髫守ｧｻ陦後☆繧九�・

# 2026-03-03 菴懈･ｭ繝｡繝｢ (繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝・繧ｷ繝｣繝峨・繧､繝ｳ繧ｰ譬ｹ譛ｬ菫ｮ豁｣)
- 逶ｮ逧・
  - `add add 1` 縺ｪ縺ｩ蜷悟錐縺ｮ蛟､譚溽ｸ帙→髢｢謨ｰ譚溽ｸ帙′蜈ｱ蟄倥☆繧九こ繝ｼ繧ｹ縲∝・螟門酔蜷埼未謨ｰ・亥酔荳�繧ｷ繧ｰ繝阪メ繝｣・峨〒縺ｮ `ambiguous overload` 繧定ｧ｣豸医☆繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - 蜈磯�ｭ菴咲ｽｮ縺ｮ隴伜挨蟄舌〒繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝蛾≦蟒ｶ繧定｡後≧髫帙�∝�､譚溽ｸ・(`i32` 縺ｪ縺ｩ) 縺ｸ縺ｮ繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ縺悟・縺ｫ襍ｰ繧翫�∝他縺ｳ蜃ｺ縺怜ｼ上′蛟､縺ｨ縺励※隗｣驥医＆繧・`D3016` 縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
  - 蛟呵｣懊′隍・焚縺ゅｋ縺ｨ縺阪�∝酔荳�繧ｷ繧ｰ繝阪メ繝｣・亥ｮ溯ｳｪ繧ｷ繝｣繝峨・・峨・蛟呵｣懊ｂ譖匁乂謇ｱ縺・＆繧後※縺・◆縲・
- 菫ｮ豁｣:
  - `nepl-core/src/typecheck.rs`
    - 蜈磯�ｭ菴咲ｽｮ縺九▽蠕檎ｶ壹ヨ繝ｼ繧ｯ繝ｳ縺ゅｊ縺ｮ蝣ｴ蜷医・縲√が繝ｼ繝舌・繝ｭ繝ｼ繝蛾≦蟒ｶ縺ｧ蛟､譚溽ｸ帙∈關ｽ縺ｨ縺輔↑縺・ｈ縺・擅莉ｶ繧剃ｿｮ豁｣縲・
    - 蛟呵｣憺∈蛻･蠕後↓繧ｷ繧ｰ繝阪メ繝｣驥崎､・ｒ髯､蜴ｻ縺励�∝酔荳�繧ｷ繧ｰ繝阪メ繝｣縺ｮ蜀・､門�呵｣懊・蜀・・繧貞━蜈医☆繧九ｈ縺・ｿｮ豁｣縲・
  - `stdlib/kp/kpread.nepl`
    - `scanner_read_i64` / `scanner_read_f64` 縺ｮ隨ｦ蜿ｷ繝輔Λ繧ｰ螟画焚蜷阪ｒ `neg` 縺九ｉ `is_neg` 縺ｫ邨ｱ荳�縺励�～neg` 髢｢謨ｰ縺ｨ縺ｮ陦晉ｪ√ｒ隗｣豸医�・
  - `tests/math.n.md`
    - `cast` 縺梧尠譏ｧ縺ｫ縺ｪ繧倶ｽ咲ｽｮ縺ｫ `<i128>` / `<i32>` 豕ｨ驥医ｒ莉倅ｸ趣ｼ育樟陦御ｻ墓ｧ倥↓蜷医ｏ縺帙◆譏守､ｺ・峨�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` 謌仙粥
  - `node nodesrc/tests.js -i stdlib/kp/kpgraph.nepl -o /tmp/kpgraph_focus.json -j 16` -> `223/223 pass`
  - `node nodesrc/tests.js -i tests/math.n.md -i tests/shadowing.n.md -o /tmp/math_shadow_after_fix.json -j 16` -> `254/254 pass`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-current.json -j 16` -> `718/718 pass`
# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ kpread/kpwrite 縺ｮ i32 蜈ｬ髢九が繝ｼ繝舌・繝ｭ繝ｼ繝牙・髮｢)

- 逶ｮ逧・
  - `scanner_read_i32(sc_handle: i32)` / `writer_write_i32(w_handle: i32, ...)` 縺ｮ蜈ｬ髢矩擇髴ｲ蜃ｺ繧堤ｸｮ蟆上＠縲∝茜逕ｨ閠・′ `Scanner` / `Writer` 繧剃ｽｿ縺・ｨｭ險医↓邨ｱ荳�縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - 蜷悟錐縺ｧ `i32` 蜿励￠蜿悶ｊ迚医→ `Scanner/Writer` 迚医ｒ蜈ｬ髢九＠縺ｦ縺・ｋ縺ｨ縲∝ｮ牙・蝙帰PI縺ｸ遘ｻ陦後＠縺ｦ繧ら函繝上Φ繝峨Ν邨瑚ｷｯ縺ｸ邁｡蜊倥↓謌ｻ繧後※縺励∪縺・�∬ｨｭ險医・荳�雋ｫ諤ｧ縺悟ｴｩ繧後ｋ縲・
  - 譌｢蟄倥・繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝芽ｧ｣豎ｺ縺ｯ蜍穂ｽ懊＠縺ｦ縺・※繧ゅ�∝・髢矩擇縺ｫ unsafe 邨瑚ｷｯ縺梧ｮ九ｋ縺薙→閾ｪ菴薙′蜀咲匱隕∝屏縺ｫ縺ｪ繧九�・
- 菫ｮ豁｣:
  - `stdlib/kp/kpread.nepl`
    - `scanner_*` 縺ｮ `i32` 蜿励￠蜿悶ｊ螳溯｣・ｒ `scanner_*_handle` 縺ｸ謾ｹ蜷阪�・
    - 蜈ｬ髢・`scanner_*` (`Scanner` 蜿励￠蜿悶ｊ) 縺九ｉ `*_handle` 繧貞他縺ｶ讒区・縺ｸ螟画峩縲・
  - `stdlib/kp/kpwrite.nepl`
    - `writer_*` 縺ｮ `i32` 蜿励￠蜿悶ｊ螳溯｣・ｒ `writer_*_handle` 縺ｸ謾ｹ蜷阪�・
    - 蜈ｬ髢・`writer_*` (`Writer` 蜿励￠蜿悶ｊ) 縺九ｉ `*_handle` 繧貞他縺ｶ讒区・縺ｸ螟画峩縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -i examples/kp_fizzbuzz.nepl --no-tree -o /tmp/tests-kp-handle-split.json -j 15` -> `230/230 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kp-handle-split.json -j 15` -> `729/729 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kp-handle-split.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - `kpread/kpwrite` 縺ｮ蜈ｬ髢句錐縺ｯ `Scanner/Writer` 迚医ｒ荳ｭ蠢・↓謨ｴ逅・＆繧後◆縲・
  - 谺｡谿ｵ縺ｧ `core/mem` 蛛ｴ縺ｮ `*_raw` 谿ｵ髫守ｸｮ騾�・・Result` 荳�譛ｬ蛹厄ｼ峨ｒ騾ｲ繧√ｋ縲・
# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ kpread/kpwrite 縺ｮ raw 蜻ｼ縺ｳ蜃ｺ縺鈴勁蜴ｻ)

- 逶ｮ逧・
  - `kpread/kpwrite` 螳溯｣・・驛ｨ縺ｫ谿九▲縺ｦ縺・◆ `alloc_raw/dealloc_raw` 逶ｴ蜻ｼ縺ｳ繧・`Result` 邉ｻAPI縺ｸ蟇・○縲∝､ｱ謨玲凾謖吝虚繧貞梛縺ｧ謇ｱ縺医ｋ繧医≧縺ｫ縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - `scanner_read_token` 縺ｯ `alloc_raw` 螟ｱ謨玲凾・・霑泌唆・峨ｒ閠・・縺励※縺翫ｉ縺壹�√・繝・ム譖ｸ縺崎ｾｼ縺ｿ縺ｧ譛ｪ螳夂ｾｩ蜍穂ｽ懊↓縺ｪ繧雁ｾ励◆縲・
  - `writer_free` 縺ｯ `dealloc_raw` 逶ｴ蜻ｼ縺ｳ縺ｧ縲∬ｧ｣謾ｾ螟ｱ謨励ｒ蜷ｸ蜿弱☆繧倶ｸ�雋ｫ縺励◆邨瑚ｷｯ縺後↑縺九▲縺溘�・
- 菫ｮ豁｣:
  - `stdlib/kp/kpread.nepl`
    - `scanner_read_token_handle` 縺ｮ譁・ｭ怜・遒ｺ菫昴ｒ `alloc` + `Result` 蛻・ｲ舌∈螟画峩縲・
    - 遒ｺ菫晏､ｱ謨玲凾縺ｯ繧ｫ繝ｼ繧ｽ繝ｫ縺�縺鷹�ｲ繧√※ `""` 繧定ｿ斐☆蜍穂ｽ懊↓邨ｱ荳�縲・
  - `stdlib/kp/kpwrite.nepl`
    - `writer_free_handle` 縺ｮ隗｣謾ｾ繧・`writer_try_free` 邨檎罰縺ｸ螟画峩・・dealloc` 縺ｮ `Err` 蜷ｸ蜿趣ｼ峨�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md -i tests/kp_i64.n.md -i tests/stdin.n.md -i tutorials/getting_started/22_competitive_io_and_arith.n.md -i tutorials/getting_started/24_competitive_dp_basics.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -i examples/kp_fizzbuzz.nepl --no-tree -o /tmp/tests-kp-safe-mem-no-raw.json -j 15` -> `230/230 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-kp-no-raw.json -j 15` -> `729/729 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-kp-no-raw.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - `kpread/kpwrite` 縺九ｉ `alloc_raw/dealloc_raw/realloc_raw` 縺ｮ逶ｴ謗･菴ｿ逕ｨ縺ｯ髯､蜴ｻ貂医∩縲・
  - 谺｡谿ｵ縺ｯ `core/mem` 蛛ｴ縺ｧ `*_raw` 縺ｮ蜈ｬ髢狗ｸｮ騾�譁ｹ驥晢ｼ亥ｮ悟・蜑企勁繧ｿ繧､繝溘Φ繧ｰ・峨ｒ謨ｴ逅・☆繧九�・
# 2026-03-04 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD騾ｲ陦・ tests/tutorials 縺ｮ alloc_safe 蛹・

- 逶ｮ逧・
  - `core/mem` 縺ｮ螳牙・API讓呎ｺ門喧譁ｹ驥昴↓蜷医ｏ縺帙�～tests/tutorials` 縺ｧ縺ｮ `alloc_raw/dealloc_raw` 逶ｴ謗･菴ｿ逕ｨ繧呈ｮｵ髫守噪縺ｫ蜑頑ｸ帙☆繧九�・
- 莠句燕譽壼査縺・
  - `rg` 縺ｧ repo 蜈ｨ菴薙・ `alloc_raw/dealloc_raw/realloc_raw` 蜻ｼ縺ｳ蜃ｺ縺励ｒ蛻・｡槭＠縲～nm/std/collections` 縺ｫ蠎・ｯ・峇縺ｮ谿句ｭ倥′縺ゅｋ縺薙→繧堤｢ｺ隱阪�・
  - 莉雁屓縺ｯ蠖ｱ髻ｿ縺悟､ｧ縺阪￥蝗槫ｸｰ縺励ｄ縺吶＞ `tests/kp.n.md` 縺ｨ `tutorials/getting_started/{23,25,26}` 繧貞・陦檎ｧｻ陦悟ｯｾ雎｡縺ｫ驕ｸ螳壹�・
- 菫ｮ豁｣:
  - `tests/kp.n.md`
    - `alloc_raw/dealloc_raw` 繧・`unwrap_ok alloc/dealloc` 縺ｸ鄂ｮ謠帙�・
    - 蠢・ｦ√↑繧ｹ繝九・繝・ヨ縺ｫ `#import "core/result" as *` 繧定ｿｽ蜉�縲・
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
  - `tutorials/getting_started/26_competitive_graph_bfs.n.md`
    - 蜷梧ｧ倥↓ `alloc_raw/dealloc_raw` 繧・`unwrap_ok alloc/dealloc` 縺ｸ鄂ｮ謠帙＠縲～core/result` import 繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/kp.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -i tutorials/getting_started/26_competitive_graph_bfs.n.md --no-tree -o /tmp/tests-safe-alloc-docs-scope.json -j 15` -> `217/217 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-current-full-after-safe-alloc-docs.json -j 15` -> `729/729 pass`
  - `node nodesrc/tests.js -i tutorials --no-tree -o /tmp/tests-tutorials-after-safe-alloc-docs.json -j 15` -> `262/262 pass`
- 迥ｶ豕・
  - `kp` 邉ｻ繝・せ繝・繝√Η繝ｼ繝医Μ繧｢繝ｫ縺ｮ荳ｻ隕√し繝ｳ繝励Ν縺ｯ螳牙・API邨瑚ｷｯ縺ｸ遘ｻ陦梧ｸ医∩縲・
  - 谺｡谿ｵ縺ｯ譽壼査縺玲ｸ医∩谿倶ｻｶ・・stdlib/std`, `stdlib/nm`, `stdlib/alloc/collections`・峨ｒ荳頑ｵ∝ｽｱ髻ｿ縺ｮ蟆上＆縺・�・↓遘ｻ陦後☆繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (move_check: 荳�譎ょ�溽畑縺ｮ蟇ｿ蜻ｽ隱､蛻､螳壹ｒ譬ｹ譛ｬ菫ｮ豁｣)

- 逶ｮ逧・
  - `stdlib` doctest 縺ｧ逋ｺ逕溘＠縺ｦ縺・◆ `D3051 cannot move out of shared borrowed value` / `D3053 use of moved value` 縺ｮ騾｣骼悶ｒ縲∝�ｴ蠖薙◆繧雁ｯｾ蠢懊〒縺ｯ縺ｪ縺・move_check 縺ｮ蛟溽畑蟇ｿ蜻ｽ繝｢繝・Ν菫ｮ豁｣縺ｧ隗｣豸医☆繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - `passes/move_check.rs` 縺・`#intrinsic load/store` 縺ｮ繧｢繝峨Ξ繧ｹ隧穂ｾ｡繧呈ｰｸ邯壼�溽畑縺ｨ縺励※謇ｱ縺｣縺ｦ縺・◆縲・
  - `get`/`load` 縺ｮ繧医≧縺ｪ隱ｭ縺ｿ蜿悶ｊ縺ｧ逕滓・縺輔ｌ繧句�溽畑縺ｯ蠑剰ｩ穂ｾ｡荳ｭ縺ｮ縺ｿ譛牙柑縺ｪ縺ｯ縺壹□縺後�・未謨ｰ譛ｫ蟆ｾ縺ｾ縺ｧ `BorrowedShared` 縺梧ｮ九ｊ縲∝ｾ檎ｶ壹・蜷御ｸ�蛟､蛻ｩ逕ｨ繧定ｪ､縺｣縺ｦ諡貞凄縺励※縺・◆縲・
- 菫ｮ豁｣:
  - `nepl-core/src/passes/move_check.rs`
    - `check_temporary_borrow` 繧定ｿｽ蜉�縲・
    - `#intrinsic load/store` 縺ｮ繧｢繝峨Ξ繧ｹ隧穂ｾ｡繧呈ｰｸ邯壼�溽畑縺ｧ縺ｯ縺ｪ縺丈ｸ�譎ょ�溽畑縺ｨ縺励※讀懆ｨｼ縺吶ｋ繧医≧螟画峩縲・
    - 豌ｸ邯壼�溽畑迥ｶ諷区峩譁ｰ縺悟ｿ・ｦ√↑ `AddrOf` 縺ｯ蠕捺擂縺ｩ縺翫ｊ `check_borrow` 繧剃ｽｿ逕ｨ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/move_check.n.md -i tests/kp.n.md -i tests/kp_i64.n.md --no-tree -o /tmp/tests-copy-move-targeted-after-temp-borrow.json -j 15` -> `245/245 pass`
  - `node nodesrc/tests.js -i tests -i stdlib -i tutorials --no-tree -o /tmp/tests-all-after-temp-borrow-fix.json -j 15` -> `799/799 pass`
- 陬懆ｶｳ:
  - 縲慶opy 諠・�ｱ縺ｮ繝上・繝峨さ繝ｼ繝牙炎貂帙�阪・邯咏ｶ夊ｪｲ鬘後�ＡTypeCtx::is_copy` 縺ｮ蜈ｨ髱｢遘ｻ陦後・ move/effect 險ｭ險医→蜷梧凾縺ｫ谿ｵ髫主ｮ滓命縺吶ｋ・井ｻ墓ｧ俶嶌縺ｨ todo 縺ｮ鬆・ｺ上ｒ蜆ｪ蜈茨ｼ峨�・
# 2026-03-04 菴懈･ｭ繝｡繝｢ (trait 險ｭ險医・蜀咲｢ｺ隱阪→荳頑ｵ∽ｿｮ豁｣)

- 逶ｮ逧・
  - `plan.md` 縺ｨ `doc/move_effect_spec.md` 縺ｫ謨ｴ蜷医☆繧句ｽ｢縺ｧ縲》rait 螳溯｣・紛蜷医・蛻､螳壹ｒ螳牙ｮ壼喧縺吶ｋ縲・
  - Rust/Haskell 縺ｮ險ｭ險郁ｫ也せ・亥･醍ｴ・�∝宛邏・�…oherence・峨ｒ NEPLg2 蜷代￠縺ｫ謨ｴ逅・＠縲∝ｮ溯｣・婿驥昴ｒ蝗ｺ螳壹☆繧九�・

- 螳滓命:
  - `nepl-core/src/typecheck.rs`
    - impl 繝｡繧ｽ繝・ラ鄂ｲ蜷阪・ trait 謨ｴ蜷亥愛螳壹ｒ譁・ｭ怜・豈碑ｼ・°繧画ｧ矩��蝙句酔蛟､・・ctx.same_type`・峨∈螟画峩縲・
  - `doc/trait_system_design.md` 繧呈眠隕丈ｽ懈・縲・
    - NEPLg2 縺ｫ縺翫￠繧・trait 縺ｮ蠖ｹ蜑ｲ・・nterface/type-class/繝｡繝｢繝ｪ閭ｽ蜉幢ｼ峨ｒ螳夂ｾｩ縲・
    - coherence縲√が繝ｼ繝舌・繝ｭ繝ｼ繝画紛蜷医�√ワ繝ｼ繝峨さ繝ｼ繝画怙蟆丞喧譁ｹ驥昴�∵僑蠑ｵ鬆・ｺ上ｒ譏取枚蛹悶�・
  - `todo.md`
    - 繝輔ぉ繝ｼ繧ｺ `B2`・・rait 險ｭ險医・螳溯｣・渚譏�・峨ｒ霑ｽ蜉�縲・

- 繝・せ繝・
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-trait-design-targeted.json -j 15` -> `276/276 pass`

- 蟾ｮ蛻・ｪ崎ｭ・
  - 萓晉┯縺ｨ縺励※ `Copy/Clone` 閭ｽ蜉帶磁邯壹↓縺ｯ譛�蟆城剞縺ｮ trait 蜷榊盾辣ｧ縺梧ｮ九▲縺ｦ縺・ｋ縲・
  - 谺｡谿ｵ縺ｧ `todo.md` 繝輔ぉ繝ｼ繧ｺB2縺ｫ蠕薙＞縲∬・蜉帙ユ繝ｼ繝悶Ν蛹悶＠縺ｦ蜷榊燕蛻・ｲ舌ｒ邵ｮ蟆上☆繧九�・

# 2026-03-04 菴懈･ｭ繝｡繝｢ (trait閭ｽ蜉帛愛螳壹・髮・ｴ・

- 逶ｮ逧・
  - `Copy/Clone` 縺ｮ蛻､螳壼・蟯舌ｒ螻�謇�蛹悶＠縲～typecheck.rs` 蜈ｨ菴薙↓謨｣蝨ｨ縺励※縺・◆譁・ｭ怜・豈碑ｼ・ｒ髮・ｴ・☆繧九�・

- 螳滓命:
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics` 繧定ｿｽ蜉�縺励�》rait 螳｣險�縺九ｉ `copy_trait_name` / `clone_trait_name` 繧呈､懷・縺吶ｋ豬√ｌ縺ｸ螟画峩縲・
    - `Copy` / `Clone` 蜿ら・邂・園・・mpl 蜿朱寔縲…lone 蜑肴署讀懈渊縲〉eject 驕ｩ逕ｨ縲’inal impl 逕滓・・峨ｒ `trait_semantics` 邨檎罰縺ｸ邨ｱ荳�縲・
    - 逶ｴ謗･縺ｮ `Some(\"Copy\")` / `Some(\"Clone\")` 豈碑ｼ・ｒ髯､蜴ｻ縲・

- 繝・せ繝・
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-trait-semantics-targeted.json -j 15` -> `276/276 pass`

- 谺｡谿ｵ:
  - `todo.md` 繝輔ぉ繝ｼ繧ｺB2縺ｮ谿倶ｻｶ縺ｨ縺励※縲∬・蜉帛愛螳壹・螟夜Κ螳夂ｾｩ蛹厄ｼ医さ繝ｳ繝代う繝ｩ蜀・Κ蝗ｺ螳壼錐縺ｮ縺輔ｉ縺ｪ繧狗ｸｮ蟆擾ｼ峨ｒ險ｭ險医☆繧九�・

# 2026-03-05 菴懈･ｭ繝｡繝｢ (compile_fail 縺ｮ險ｺ譁ｭ菴咲ｽｮ讀懆ｨｼ繧定ｿｽ蜉�)

- 逶ｮ逧・
  - `tests/*.n.md` 縺ｮ `compile_fail` 繧ｱ繝ｼ繧ｹ縺ｧ縲～diag_id` 縺�縺代〒縺ｪ縺剰ｨｺ譁ｭ菴咲ｽｮ・・ile/line/col・峨ｂ螳｣險�縺励※讀懆ｨｼ縺ｧ縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・

- 譬ｹ譛ｬ蜴溷屏:
  - 譌｢蟄倥・ doctest 莉墓ｧ倥・ `diag_id` 縺ｮ縺ｿ繧貞女逅・＠縺ｦ縺翫ｊ縲√�後←縺ｮ菴咲ｽｮ縺ｧ縺昴・險ｺ譁ｭ縺悟・繧九∋縺阪°縲阪ｒ讖滓｢ｰ讀懆ｨｼ縺ｧ縺阪↑縺九▲縺溘�・
  - 縺昴・縺溘ａ縲∝酔縺・`diag_id` 縺悟挨菴咲ｽｮ縺ｧ逋ｺ逕溘＠縺ｦ繧ゅユ繧ｹ繝医′隕矩�・☆菴吝慍縺後≠縺｣縺溘�・

- 螳滓命:
  - `nodesrc/parser.js`
    - doctest 繝｡繧ｿ縺ｫ `diag_span` / `diag_spans` 繧定ｿｽ蜉�縲・
    - `line:col` 縺ｨ `file:line:col` 縺ｮ荳｡蠖｢蠑上ｒ蜿礼炊縲・
  - `nodesrc/tests.js`
    - `expected_diag_spans` 繧偵こ繝ｼ繧ｹ縺ｫ菫晄戟縲・
    - `compile_fail` 隧穂ｾ｡譎ゅ↓ `compile_error` 縺九ｉ `--> file:line:col` 繧呈歓蜃ｺ縺励�∵悄蠕・ｽ咲ｽｮ縺ｨ辣ｧ蜷医�・
    - `compile_fail` 縺ｮ `diag_id` / `diag_span` 讀懆ｨｼ繧・`--assert-io` 萓晏ｭ倥°繧牙・繧企屬縺励�∝ｸｸ譎りｩ穂ｾ｡縺ｸ螟画峩縲・
  - `tests/compile_fail_diag_location.n.md`
    - `diag_span`・亥腰菴難ｼ峨→ `diag_spans`・郁､・焚・峨ｒ菴ｿ縺｣縺滓､懆ｨｼ繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・

- 讀懆ｨｼ:
  - `node -c nodesrc/parser.js && node -c nodesrc/tests.js` -> success
  - `node nodesrc/tests.js -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-compile-fail-diag-location.json -j 15` -> `2/2 pass`
  - `node nodesrc/tests.js -i tests/keywords_reserved.n.md --no-stdlib --no-tree -o /tmp/tests-keywords-reserved.json -j 15` -> `6/6 pass`

- 陬懆ｶｳ:
  - `--no-stdlib` 縺ｪ縺怜ｮ溯｡梧凾縺ｯ譌｢遏･縺ｮ `stdlib/alloc/collections/list.nepl` 螟ｱ謨励′豺ｷ蝨ｨ縺吶ｋ縺溘ａ縲∽ｻ雁屓繧ｿ繧ｹ繧ｯ縺ｮ螻�謇�讀懆ｨｼ縺ｧ縺ｯ髯､螟悶＠縺溘�・

# 2026-03-05 菴懈･ｭ繝｡繝｢ (diag_id 讀懆ｨｼ縺ｮ蜴ｳ蟇・喧)

- 逶ｮ逧・
  - `compile_fail` 縺ｮ `diag_id` 繧偵�後ユ繧ｹ繝磯�夐℃縺ｮ縺溘ａ縺ｮ蛟､蜷医ｏ縺帙�阪〒縺ｯ縺ｪ縺上�∝ｮ滄圀縺ｫ讀懆ｨｼ縺励◆縺・､ｱ謨怜次蝗�縺ｫ荳�閾ｴ縺輔○繧九�・

- 螳滓命:
  - `tests/move_effect.n.md`
    - 縲茎hared borrow 荳ｭ move 諡貞凄縲阪ｒ縲・未謨ｰ蛟､蜻ｼ縺ｳ蜃ｺ縺礼罰譚･縺ｮ蜑ｯ谺｡險ｺ譁ｭ縺梧ｷｷ縺悶ｉ縺ｪ縺・怙蟆丞・迴ｾ縺ｸ譖ｸ縺肴鋤縺茨ｼ・diag_id: 3051`・峨�・
    - 縲碁撼隍・粋蝙・field access 諡貞凄縲阪ｒ `v.len` 蠖｢蠑上・譛�蟆丞・迴ｾ縺ｸ譖ｸ縺肴鋤縺茨ｼ・diag_id: 3011`・峨�・
    - 縲後げ繝ｭ繝ｼ繝舌Ν set縲阪こ繝ｼ繧ｹ縺ｯ迴ｾ蝨ｨ螳溯｣・・險ｺ譁ｭ謖吝虚・・TypeUndefinedVariable`, `3002`・峨ｒ譏守､ｺ縺吶ｋ蠖｢縺ｫ隱ｬ譏弱ｒ譖ｴ譁ｰ縲・

- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/move_effect.n.md --no-tree -o /tmp/tests-move-effect-audit2.json -j 15` -> `225/225 pass`
  - `node nodesrc/tests.js -i tests/neplg2.n.md --no-tree -o /tmp/tests-neplg2-fix2.json -j 15` -> `249/249 pass`
  - `node nodesrc/tests.js -i tests/kp.n.md --no-tree -o /tmp/tests-kp-fix2.json -j 15` -> `211/211 pass`
  - `node nodesrc/tests.js -i tests -o /tmp/tests-current-full8.json -j 15` -> `797/797 pass`

- 陬懆ｶｳ:
  - `diag_id` 縺ｮ螟画峩縺ｯ縲∝推繧ｱ繝ｼ繧ｹ繧貞腰菴灘・迴ｾ縺励※螳溯ｨｺ譁ｭ繧堤｢ｺ隱阪＠縺溘ｂ縺ｮ縺ｮ縺ｿ蜿肴丐縺励◆縲・
  - 螟ｱ謨怜次蝗�縺瑚､・焚豺ｷ蝨ｨ縺吶ｋ繧ｱ繝ｼ繧ｹ縺ｯ縲√ユ繧ｹ繝医さ繝ｼ繝牙・繧偵�檎漁縺｣縺溯ｨｺ譁ｭ縺�縺代′蜃ｺ繧句ｽ｢縲阪↓蛻・ｧ｣縺励※蜀肴ｧ区・縺励◆縲・

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺB2: trait閭ｽ蜉帙ユ繝ｼ繝悶Ν縺ｮ蟆主・縺ｨ蝗槫ｸｰ螳牙ｮ壼喧)

- 逶ｮ逧・
  - `todo.md` 繝輔ぉ繝ｼ繧ｺB2・・Copy/Clone` 閭ｽ蜉帛愛螳壹・閭ｽ蜉帙ユ繝ｼ繝悶Ν蛹厄ｼ峨ｒ騾ｲ繧√�～typecheck` 縺ｮ閭ｽ蜉帛愛螳壹ｒ螻�謇�蛹悶☆繧九�・

- 螳滓命:
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics::detect` 繧呈僑蠑ｵ縺励�》rait doc 縺九ｉ `@capability: copy|clone` 繧定ｪｭ繧薙〒閭ｽ蜉帙ｒ險ｭ螳壹☆繧狗ｵ瑚ｷｯ繧定ｿｽ蜉�縲・
    - 譌｢蟄倥・繝｡繧ｽ繝・ラ蜷阪・繧ｿ萓晏ｭ假ｼ・copy_mark`/`clone`・画､懷・繧貞炎髯､縲・
    - 讒矩��繝偵Η繝ｼ繝ｪ繧ｹ繝・ぅ繝・け繧定ｿｽ蜉�:
      - clone 蛟呵｣・ 蜊倅ｸ�繝｡繧ｽ繝・ラ縺九▽ `(Self)->Self`
      - copy 蛟呵｣・ marker trait・医Γ繧ｽ繝・ラ縺ｪ縺暦ｼ・
    - 莠呈鋤邯ｭ謖√・縺溘ａ縲∬・蜉帶悴遒ｺ螳壽凾縺ｮ縺ｿ `Clone` / `Copy` 蜷阪・譛�蟆上ヵ繧ｩ繝ｼ繝ｫ繝舌ャ繧ｯ繧定ｿｽ蜉�縲・
  - `tests/move_effect.n.md`
    - `compile_fail` 2繧ｱ繝ｼ繧ｹ縺ｧ `#entry main` 縺�縺大ｮ夂ｾｩ縺輔ｌ險ｺ譁ｭ縺・`D3092` 縺ｫ蜷ｸ繧上ｌ繧句撫鬘後ｒ菫ｮ豁｣縺励�～main` 繧定ｿｽ蜉�縺励※迢吶▲縺・`diag_id` 繧呈､懆ｨｼ蜿ｯ閭ｽ蛹悶�・
    - `Copy` 髢｢騾｣繧ｱ繝ｼ繧ｹ縺ｫ `@capability` 螳｣險�繧定ｿｽ險倥�・

- 譬ｹ譛ｬ蜴溷屏:
  - 譌ｧ螳溯｣・・閭ｽ蜉帛愛螳壹ｒ縲荊rait蜷・+ method蜷阪�咲ｵ・↓萓晏ｭ倥＠縺ｦ縺翫ｊ縲∽ｻ墓ｧ俶僑蠑ｵ譎ゅ↓隱､蛻､螳壹′襍ｷ縺阪ｄ縺吶°縺｣縺溘�・
  - `compile_fail` 縺ｮ荳�驛ｨ繧ｱ繝ｼ繧ｹ縺ｯ繧ｨ繝ｳ繝医Μ譛ｪ螳夂ｾｩ縺悟・縺ｫ逋ｺ轣ｫ縺励�∫漁縺｣縺溷屓蟶ｰ讀懆ｨｼ縺ｫ縺ｪ縺｣縺ｦ縺・↑縺九▲縺溘�・

- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-b2-capability-targeted-v4.json -j 15` -> `281/281 pass`
  - `node nodesrc/tests.js -i tests -i tutorials -i stdlib --no-tree -o /tmp/tests-all-b2-capability-v1.json -j 15` -> `837/837 pass`

- 蟾ｮ蛻・ｪ崎ｭ・
  - 閭ｽ蜉帶､懷・縺ｮ荳ｻ邨瑚ｷｯ縺ｯ閭ｽ蜉帙ユ繝ｼ繝悶Ν蛹匁ｸ医∩縲・
  - 縺溘□縺怜ｮ悟・謦､蟒・〒縺ｯ縺ｪ縺上�∵悴螳｣險�譎ゅ・譛�蟆丈ｺ呈鋤縺ｨ縺励※ `Copy/Clone` 蜷阪ヵ繧ｩ繝ｼ繝ｫ繝舌ャ繧ｯ縺梧ｮ九ｋ縲Ａtodo.md` 繝輔ぉ繝ｼ繧ｺB2縺ｮ縲梧枚蟄怜・豈碑ｼ・ｮ悟・謦､蟒・�阪ｒ貅�縺溘☆縺ｫ縺ｯ谺｡谿ｵ縺ｧ縺薙・莠呈鋤螻､繧貞､悶☆蠢・ｦ√′縺ゅｋ縲・

# 2026-03-05 菴懈･ｭ繝｡繝｢ (B2 讀懆ｨｼ: 蜷咲ｧｰ繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ謦､蜴ｻ縺ｮ隧ｦ陦檎ｵ先棡)

- 螳滓命:
  - `TraitSemantics::detect` 縺ｮ `Copy/Clone` 蜷阪ヵ繧ｩ繝ｼ繝ｫ繝舌ャ繧ｯ繧剃ｸ�譎ら噪縺ｫ謦､蜴ｻ縺励�∬・蜉帛ｮ｣險� + 讒矩��繝偵Η繝ｼ繝ｪ繧ｹ繝・ぅ繝・け縺ｮ縺ｿ縺ｸ蛻・崛繧定ｩｦ陦後＠縺溘�・

- 邨先棡:
  - `tests/move_effect.n.md` 縺ｮ `Copy` 邉ｻ `compile_fail` 縺碁�壹ｉ縺壹�～expected compile_fail, but compiled successfully` 縺ｨ縺ｪ縺｣縺溘�・
  - 蜴溷屏縺ｯ縲∫樟陦悟ｮ溯｣・〒縺ｯ `//: @capability: ...` 縺瑚・蜉帶､懷・蜈･蜉帙→縺励※螳牙ｮ壻ｾ帷ｵｦ縺輔ｌ縺壹�～Copy` 閭ｽ蜉帙′譛ｪ讀懷・縺ｫ縺ｪ繧狗ｵ瑚ｷｯ縺梧ｮ九ｋ縺溘ａ縲・

- 蟇ｾ蠢・
  - 蜷咲ｧｰ繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ縺ｯ蜀榊ｰ主・縺励◆縲・
  - 蜀肴､懆ｨｼ:
    - `NO_COLOR=false trunk build` -> success
    - `node nodesrc/tests.js -i tests -i tutorials -i stdlib --no-tree -o /tmp/tests-all-b2-capability-v2.json -j 15` -> `837/837 pass`

- 谺｡谿ｵ縺ｮ荳頑ｵ∬ｪｲ鬘・
  - `Copy/Clone` 縺ｮ閭ｽ蜉帛ｮ｣險�繧・`doc comment` 萓晏ｭ倥〒縺ｪ縺・AST/譁・ｳ輔Ξ繝吶Ν縺ｧ萓帷ｵｦ縺吶ｋ莉慕ｵ・∩繧定ｿｽ蜉�縺励�∝錐遘ｰ繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ繧呈彫蜴ｻ縺吶ｋ縲・
# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺB2: `#capability` 譁・ｳ募喧縺ｨ蝙区､懈渊邨ｱ蜷・

- 逶ｮ逧・
  - `todo.md` 繝輔ぉ繝ｼ繧ｺB2縺ｮ荳頑ｵ∝・縺ｨ縺励※縲～Copy/Clone` 閭ｽ蜉帙・螳｣險�邨瑚ｷｯ繧・doc 譁・ｭ怜・萓晏ｭ倥°繧・parser/AST 邨瑚ｷｯ縺ｸ遘ｻ縺吶�・
  - codegen 謇句燕縺ｧ蜷御ｸ�縺ｮ trait 閭ｽ蜉帶ュ蝣ｱ繧貞盾辣ｧ縺ｧ縺阪ｋ蠖｢縺ｫ謠・∴繧九�・

- 螳溯｣・
  - `nepl-core/src/ast.rs`
    - `TraitDef` 縺ｫ `capabilities: Vec<String>` 繧定ｿｽ蜉�縲・
  - `nepl-core/src/lexer.rs`
    - `TokenKind::DirCapability(String)` 繧定ｿｽ蜉�縲・
    - `#capability ...` 繧・lex 蟇ｾ雎｡縺ｫ霑ｽ蜉�縲・
  - `nepl-core/src/parser.rs`
    - trait 譛ｬ譁・・縺ｧ `#capability` 繧貞女逅・＠ `TraitDef.capabilities` 縺ｸ譬ｼ邏阪�・
    - 繝医ャ繝励Ξ繝吶Ν `#capability` 縺ｯ `ParserUnexpectedToken` 縺ｧ諡貞凄縲・
  - `nepl-core/src/typecheck.rs`
    - `TraitInfo` 縺ｫ `capabilities` 繧剃ｿ晄戟縲・
    - 閭ｽ蜉帶歓蜃ｺ縺ｯ `TraitInfo.capabilities` 縺九ｉ陦後≧繧医≧螟画峩・・oc 陦瑚ｧ｣譫舌ｒ蟒・ｭ｢・峨�・
  - `nepl-web/src/lib.rs`
    - token 陦ｨ遉ｺ蛛ｴ縺ｫ `DirCapability` 縺ｮ蛻・ｲ舌ｒ霑ｽ蜉�縺励※ `trunk build` 縺ｮ non-exhaustive 繧定ｧ｣豸医�・
  - `tests/move_effect.n.md`
    - `@capability:` 繧ｳ繝｡繝ｳ繝郁｡ｨ迴ｾ繧・trait 譛ｬ譁・・ `#capability` 縺ｫ鄂ｮ謠帙�・

- 繝・せ繝・
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-b2-capability-targeted-v6.json -j 15`
    - `281/281 pass`
  - `node nodesrc/tests.js -i tests -i tutorials -i stdlib --no-tree -o /tmp/tests-all-b2-capability-v3.json -j 15`
    - `837/837 pass`

- 谿玖ｪｲ鬘・
  - `Copy/Clone` 讀懷・縺ｮ譛�邨ゅヵ繧ｩ繝ｼ繝ｫ繝舌ャ繧ｯ・・rait 蜷・`Copy` / `Clone`・峨・縺ｾ縺�谿九▲縺ｦ縺・ｋ縲・
  - 繝輔ぉ繝ｼ繧ｺB2螳御ｺ・擅莉ｶ縲梧枚蟄怜・豈碑ｼ・・螳悟・謦､蟒・�阪↓蜷代￠縺ｦ縲∵ｬ｡谿ｵ縺ｧ髯､蜴ｻ縺吶ｋ縲・
# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺB2: `Copy/Clone` 蜷阪ヵ繧ｩ繝ｼ繝ｫ繝舌ャ繧ｯ蜑企勁)

- 逶ｮ逧・
  - 繝輔ぉ繝ｼ繧ｺB2谿玖ｪｲ鬘後□縺｣縺・`Copy` / `Clone` 縺ｮ trait 蜷阪ワ繝ｼ繝峨さ繝ｼ繝峨ヵ繧ｩ繝ｼ繝ｫ繝舌ャ繧ｯ繧貞ｻ・ｭ｢縺吶ｋ縲・

- 螳溯｣・
  - `nepl-core/src/typecheck.rs`
    - `TraitSemantics::detect` 縺ｮ譛ｫ蟆ｾ縺ｫ谿九▲縺ｦ縺・◆
      - `traits.get("Clone")` 繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ
      - `traits.get("Copy")` 繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ
      繧貞炎髯､縲・
    - 閭ｽ蜉帛愛螳壹・ `#capability`・医♀繧医・讒矩��繝偵Η繝ｼ繝ｪ繧ｹ繝・ぅ繝・け・臥ｵ瑚ｷｯ縺ｮ縺ｿ繧剃ｽｿ逕ｨ縺吶ｋ蠖｢縺ｫ邨ｱ荳�縲・

- 繝・せ繝・
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-b2-capability-targeted-v7.json -j 15`
    - `281/281 pass`
  - `node nodesrc/tests.js -i tests -i tutorials -i stdlib --no-tree -o /tmp/tests-all-b2-capability-v4.json -j 15`
    - `837/837 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺB2: `#capability` 莉墓ｧ伜｢・阜縺ｮ蝗槫ｸｰ霑ｽ蜉�)

- 逶ｮ逧・
  - `#capability` 縺・trait 譛ｬ譁・・縺ｮ縺ｿ譛牙柑縺ｧ縺ゅｋ莉墓ｧ倥ｒ繝・せ繝医〒蝗ｺ螳壹☆繧九�・

- 螳溯｣・
  - `tests/overload.n.md`
    - `capability_directive_is_trait_local_only` 繧定ｿｽ蜉�縲・
    - `compile_fail + diag_id: 2002 (ParserUnexpectedToken)` 縺ｧ蝗ｺ螳壹�・

- 繝・せ繝・
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-b2-capability-targeted-v8.json -j 15`
    - `282/282 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺB2: trait bound 蛻､螳壹・ TypeId 逶ｴ蜿ら・蛹・

- 逶ｮ逧・
  - trait method 蜻ｼ縺ｳ蜃ｺ縺玲凾縺ｮ bound 蛻､螳壹〒縲》rait 蜷榊・隗｣豎ｺ繧堤ｵ檎罰縺吶ｋ邨瑚ｷｯ繧貞炎貂帙☆繧九�・

- 螳溯｣・
  - `nepl-core/src/typecheck.rs`
    - trait method 蜻ｼ縺ｳ蜃ｺ縺怜・蟯舌〒 `resolve_trait_bound_ref(trait_name)` 繧貞ｻ・ｭ｢縲・
    - 縺吶〒縺ｫ蜿門ｾ玲ｸ医∩縺ｮ `trait_info.self_ty` 繧剃ｽｿ縺・�・
      - `type_param_has_bound(self_ty, trait_self_ty)`
      - `impls` 荳翫・ `trait_self_ty + target_ty` 荳�閾ｴ
      縺ｮ蜷域・蛻､螳壹∈鄂ｮ謠帙�・
    - 譛ｪ菴ｿ逕ｨ蛹悶＠縺・`resolve_trait_bound_ref` 繧貞炎髯､縲・
  - `tests/overload.n.md`
    - `capability_directive_is_trait_local_only` 繧定ｿｽ蜉�縺励※ parser 蠅・阜繧貞崋螳夲ｼ・diag_id: 2002`・峨�・

- 繝・せ繝・
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/overload.n.md -i tests/move_effect.n.md -i tests/move_check.n.md --no-tree -o /tmp/tests-b2-capability-targeted-v9.json -j 15`
    - `282/282 pass`
  - `node nodesrc/tests.js -i tests -i tutorials -i stdlib --no-tree -o /tmp/tests-all-b2-capability-v5.json -j 15`
    - `838/838 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (move_check 縺ｮ diag_id 讀懆ｨｼ邊ｾ蠎ｦ菫ｮ豁｣)

- 莠玖ｱ｡:
  - `tests/move_check.n.md::doctest#7` 縺・`diag_id: 3051` 譛溷ｾ・〒螟ｱ謨励�・
  - 螳滄圀縺ｯ `D3003` 縺悟・縺ｫ蜃ｺ縺ｦ縺翫ｊ縲～diag_id` 讀懆ｨｼ縺ｨ縺励※荳肴ｭ｣遒ｺ縺�縺｣縺溘�・

- 蜴溷屏:
  - `move_reference_ok` 繧ｱ繝ｼ繧ｹ縺ｧ `fn main <()->i32>` 縺ｫ蟇ｾ縺励※譛ｫ蟆ｾ蠑上′縺ｪ縺上�・
    move/borrow 險ｺ譁ｭ繧医ｊ蜈医↓謌ｻ繧雁�､荳堺ｸ�閾ｴ險ｺ譁ｭ縺檎匱逕溘＠縺ｦ縺・◆縲・

- 菫ｮ豁｣:
  - `tests/move_check.n.md` 縺ｮ `move_reference_ok` 縺ｫ譛ｫ蟆ｾ蠑・`0` 繧定ｿｽ蜉�縺励�・
    逶ｮ逧・・ `D3051` 縺悟燕髱｢縺ｫ蜃ｺ繧句ｽ｢縺ｸ菫ｮ豁｣縲・

- 繝・せ繝・
  - `node nodesrc/tests.js -i tests/move_check.n.md -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-movecheck-unskip-v5.json -j 15`
    - `282/282 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (move_check: 讒矩��菴薙ヵ繧｣繝ｼ繝ｫ繝・move 讀懷・縺ｮ譬ｹ譛ｬ菫ｮ豁｣)

- 莠玖ｱ｡:
  - `move_struct_field_err` 縺・`skip` 縺ｮ縺ｾ縺ｾ縺ｧ縲～s.f` 縺九ｉ髱曚opy蛟､繧・蝗櫁ｪｭ繧�繧ｱ繝ｼ繧ｹ繧呈､懷・縺ｧ縺阪※縺・↑縺九▲縺溘�・

- 譬ｹ譛ｬ蜴溷屏:
  - `s.f` 縺ｯ HIR 荳・`load` 縺ｫ lower 縺輔ｌ繧九′縲～move_check` 縺ｮ `load<non-Copy>` 蛻・ｲ舌′
    蟶ｸ縺ｫ縲御ｸ�譎ょ�溽畑縲肴桶縺・〒縲∵園譛画ｨｩ遘ｻ蜍輔→縺励※迥ｶ諷区峩譁ｰ縺励※縺・↑縺九▲縺溘�・

- 菫ｮ豁｣:
  - `nepl-core/src/passes/move_check.rs`
    - `visit_field_move_source` 繧定ｿｽ蜉�縲・
    - `load<non-Copy>` 縺ｮ縺ｨ縺阪�√い繝峨Ξ繧ｹ蠑上′繝ｭ繝ｼ繧ｫ繝ｫ隍・粋蛟､逕ｱ譚･・・Var` / `add(Var, ...)`・峨↑繧・
      蛟､遘ｻ蜍輔→縺励※ `check_use(..., is_copy=false)` 繧帝←逕ｨ縲・
    - 縺昴ｌ莉･螟悶・ `load<non-Copy>` 縺ｯ蠕捺擂縺ｩ縺翫ｊ荳�譎・unique borrow 繧帝←逕ｨ縲・
  - `tests/move_check.n.md`
    - `move_struct_field_err` 繧・`skip` 縺九ｉ `compile_fail` (`diag_id: 3053`) 縺ｫ謌ｻ縺励◆縲・

- 繝・せ繝・
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_check.n.md -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-movecheck-unskip-v6.json -j 15`
    - `282/282 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: kpread_core syscall蠅・阜縺ｮ MemPtr 邨ｱ荳�)

- 逶ｮ逧・
  - `kpread_core` 縺ｧ syscall 蠅・阜莉･螟悶・ `MemPtr<u8> -> i32` 螟画鋤繧貞ｱ�謇�蛹悶＠縲√・繧､繝ｳ繧ｿ蠅・阜繧呈・遉ｺ縺吶ｋ縲・

- 譬ｹ譛ｬ蜴溷屏:
  - `fd_read` 蜻ｼ縺ｳ蜃ｺ縺礼ｮ・園縺ｧ `mem_ptr_addr` 繧貞他縺ｳ蜃ｺ縺怜・縺ｫ逶ｴ謗･螻暮幕縺励※縺翫ｊ縲∝｢・阜雋ｬ蜍吶′蛻・淵縺励※縺・◆縲・
  - 縺薙ｌ縺ｫ繧医ｊ effect/pointer 莉墓ｧ倥・隕矩�壹＠縺梧が縺上�∝ｰ・擂縺ｮ蜈ｱ騾壼喧縺ｧ隱､逕ｨ縺悟・逋ｺ縺励ｄ縺吶＞迥ｶ諷九□縺｣縺溘�・

- 螟画峩:
  - `stdlib/kp/kpread_core.nepl`
    - `mem_u8_addr <(MemPtr<u8>)->i32>` 繧定ｿｽ蜉�縺励�～MemPtr<u8>` 縺九ｉ縺ｮ繧｢繝峨Ξ繧ｹ蜿門ｾ励ｒ荳�邂・園縺ｸ髮・ｴ・�・
    - `fd_read_mem <(i32,MemPtr<u8>,i32,MemPtr<u8>)*>i32>` 繧定ｿｽ蜉�縺励�～fd_read` 蜻ｼ縺ｳ蜃ｺ縺怜｢・阜繧貞・騾壼喧縲・
    - `scanner_new_impl` 蜀・・ `fd_read` 蜻ｼ縺ｳ蜃ｺ縺励ｒ `fd_read_mem 0 iov 1 nread_ptr` 縺ｫ鄂ｮ謠帙�・
    - `buf` 繧｢繝峨Ξ繧ｹ蜿門ｾ励・逶ｴ謗･ `mem_ptr_addr` 繧・`mem_u8_addr` 縺ｫ鄂ｮ謠帙�・

- 螳溯｣・ｸ翫・豕ｨ諢・
  - `fd_read_mem` 縺ｯ syscall 蜻ｼ縺ｳ蜃ｺ縺励ｒ蜷ｫ繧�縺溘ａ `*>`・・mpure・峨す繧ｰ繝阪メ繝｣縺ｧ螳夂ｾｩ縲・
  - 荳�譎ら噪縺ｫ pure 螳夂ｾｩ縺ｨ縺励※ `D3025` 縺檎匱逕溘＠縺溘′縲‘ffect 莉墓ｧ倥↓蜷医ｏ縺帙※ impure 縺ｸ菫ｮ豁｣縺怜・讀懆ｨｼ縺励◆縲・

- 繝・せ繝・
  - `node nodesrc/tests.js -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-core-boundary-v2.json -j 15`
  - 邨先棡: `217/217 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺC: kpwrite 繝倥ャ繝�繧｢繧ｯ繧ｻ繧ｹ縺ｮ MemPtr 蠅・阜邨ｱ荳�)

- 逶ｮ逧・
  - `Writer.raw` 縺・`MemPtr<u8>` 縺ｧ縺ゅｋ險ｭ險医↓蜷医ｏ縺帙�～kpwrite` 蜀・Κ繝倥ャ繝�繧｢繧ｯ繧ｻ繧ｹ縺ｮ蝙句｢・阜繧・`i32` 縺九ｉ `MemPtr<u8>` 縺ｸ邨ｱ荳�縺吶ｋ縲・

- 譬ｹ譛ｬ蜴溷屏:
  - `writer_header_ptr/load/store` 縺・`i32` 蜿励￠蜿悶ｊ縺ｮ縺ｾ縺ｾ谿九▲縺ｦ縺翫ｊ縲～Writer` 縺九ｉ豈主屓 `mem_ptr_addr` 縺ｸ髯肴�ｼ縺励※縺・◆縲・
  - 蠅・阜髯肴�ｼ縺梧淵蝨ｨ縺励�√Γ繝｢繝ｪ螳牙・繝｢繝・Ν・医ヵ繧ｧ繝ｼ繧ｺC・峨・縲悟・髢九・蜀・Κ縺ｨ繧ゅ↓ MemPtr 蝓ｺ貅悶�阪・譁ｹ驥昴→荳肴紛蜷医□縺｣縺溘�・

- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_header_ptr` 繧・`(MemPtr<u8>, i32)->MemPtr<i32>` 縺ｸ螟画峩縲・
    - `writer_load_header` / `writer_store_header` 繧・`MemPtr<u8>` 蜿励￠蜿悶ｊ縺ｸ螟画峩縲・
    - `writer_free_handle` / `writer_flush_handle` / `writer_ensure_handle` / `writer_put_u8_handle` / `writer_write_str_handle` / `writer_write_i32_handle` / `writer_write_u64_handle` 縺ｮ蜀・Κ縺ｧ `w_mem:MemPtr<u8>` 繧剃ｽｿ縺・ｽ｢縺ｸ邨ｱ荳�縲・
    - `writer_free_handle` 縺ｮ繝倥ャ繝�隗｣謾ｾ縺ｯ `dealloc_ptr<u8> w_mem 20` 繧剃ｽｿ逕ｨ縺励�∫函 `i32` 邨瑚ｷｯ繧貞炎貂帙�・

- 繝・せ繝・
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-memptr-v1.json -j 15`
  - 邨先棡: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v5.json -j 15`
  - 邨先棡: `226/226 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: kpwrite 蜀・Κ遒ｺ菫・隗｣謾ｾ縺ｮ MemPtr 蛹・

- 逶ｮ逧・
  - `kpwrite` 縺ｮ蜀・Κ螳溯｣・〒縲∫｢ｺ菫昴・隗｣謾ｾ邨瑚ｷｯ繧・`alloc_ptr/dealloc_ptr` 繝吶・繧ｹ縺ｫ邨ｱ荳�縺吶ｋ縲・
  - syscall 蠅・阜莉･螟悶・逕・`i32` 繝昴う繝ｳ繧ｿ謫堺ｽ懊ｒ貂帙ｉ縺励�∝梛螳牙・蠅・阜繧呈・遒ｺ蛹悶☆繧九�・

- 譬ｹ譛ｬ蜴溷屏:
  - `writer_alloc_buf` 縺ｨ `writer_new_handle` 縺・`alloc/dealloc` (`i32`) 繝吶・繧ｹ縺ｧ螳溯｣・＆繧後※縺翫ｊ縲～Writer.raw: MemPtr<u8>` 縺ｨ蜀・Κ邨瑚ｷｯ縺御ｺ碁㍾蛹悶＠縺ｦ縺・◆縲・
  - 螟ｱ謨玲凾蟾ｻ縺肴綾縺励ｂ `i32` 隗｣謾ｾ邨瑚ｷｯ縺ｫ蟇・▲縺ｦ縺・※縲｀emPtr 邉ｻ縺ｮ螳牙・API邨ｱ荳�譁ｹ驥昴→荳肴紛蜷医□縺｣縺溘�・

- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `WriterBuf.ptr` 繧・`i32` 縺九ｉ `MemPtr<u8>` 縺ｸ螟画峩縲・
    - `writer_try_free` 繧・`writer_try_free_ptr<.T>` 縺ｫ鄂ｮ謠帙＠縲～dealloc_ptr` 邨檎罰縺ｸ邨ｱ荳�縲・
    - `writer_alloc_buf` 繧・`alloc_ptr<u8>` 繝吶・繧ｹ縺ｸ螟画峩縲・
    - `writer_new_handle` 縺ｮ `buf/iov/nw/w` 遒ｺ菫昴ｒ `alloc_ptr<u8>` 繝吶・繧ｹ縺ｸ螟画峩縺励�∝､ｱ謨玲凾蟾ｻ縺肴綾縺励ｂ `writer_try_free_ptr` 縺ｫ邨ｱ荳�縲・
    - header 縺ｸ譬ｼ邏阪☆繧句�､縺�縺代ｒ `mem_ptr_addr` 縺ｧ譏守､ｺ逧・↓蠅・阜螟画鋤・・yscall/繝倥ャ繝�讒矩��縺ｨ縺ｮ謗･邯夂せ・峨�・
    - `writer_free_handle` 縺ｮ `buf/iov/nw` 隗｣謾ｾ繧・`writer_try_free_ptr<u8> mem_ptr_wrap ...` 邨檎罰縺ｸ螟画峩縲・

- 繝・せ繝・
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-memptr-v2.json -j 15`
  - 邨先棡: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v6.json -j 15`
  - 邨先棡: `226/226 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: kpwrite 蛻晄悄蛹也ｵ瑚ｷｯ縺ｮ header API 邨ｱ荳�)

- 逶ｮ逧・
  - `writer_new_handle` 縺ｧ谿九▲縺ｦ縺・◆逕・`store_i32` 縺ｮ逶ｴ譖ｸ縺阪ｒ縺ｪ縺上＠縲～writer_store_header` 邨檎罰縺ｫ邨ｱ荳�縺吶ｋ縲・

- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_new_handle` 縺ｮ header 蛻晄悄蛹厄ｼ・uf/cap/len/iov/nw・峨ｒ `writer_store_header` 蜻ｼ縺ｳ蜃ｺ縺励↓鄂ｮ謠帙�・
    - 蛻晄悄蛹匁凾縺ｮ繝昴う繝ｳ繧ｿ蠅・阜螟画鋤縺ｯ `mem_ptr_addr` 縺ｮ縺ｿ繧貞ｼ墓焚菴咲ｽｮ縺ｫ髯仙ｮ壹�・

- 繝・せ繝・
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-init-v1.json -j 15`
  - 邨先棡: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v8.json -j 15`
  - 邨先棡: `226/226 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: kpwrite 隗｣謾ｾ邨瑚ｷｯ縺ｮ繝昴う繝ｳ繧ｿ蠅・阜髮・ｴ・

- 逶ｮ逧・
  - `writer_free_handle` 縺ｧ谿九▲縺ｦ縺・◆ `i32 -> MemPtr` 縺ｮ驛ｽ蠎ｦ螟画鋤繧偵・繝ｫ繝代∈髮・ｴ・＠縲∬ｧ｣謾ｾ蠅・阜繧貞腰邏泌喧縺吶ｋ縲・

- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_load_header_ptr <(MemPtr<u8>,i32)->MemPtr<u8>>` 繧定ｿｽ蜉�縲・
    - `writer_free_handle` 縺ｯ `buf/iov/nw` 繧・`writer_load_header_ptr` 縺ｧ蜿門ｾ励＠縺ｦ `writer_try_free_ptr` 縺ｸ貂｡縺呎ｧ区・縺ｸ螟画峩縲・
    - `mem_ptr_wrap` 縺ｮ逶ｴ蜻ｼ縺ｳ繧貞炎貂帙＠縺ｦ縲”eader 蛟､縺ｮ繝昴う繝ｳ繧ｿ蛹冶ｲｬ蜍吶ｒ荳�邂・園縺ｫ髮・ｴ・�・

- 繝・せ繝・
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-freeptr-v1.json -j 15`
  - 邨先棡: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v9.json -j 15`
  - 邨先棡: `226/226 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: writer 繝倥ャ繝�譖ｸ縺崎ｾｼ縺ｿ螟ｱ謨励・謠｡繧頑ｽｰ縺怜ｻ・ｭ｢)

- 逶ｮ逧・
  - `writer_store_header` 縺悟､ｱ謨励ｒ鮟呎ｮｺ縺励※縺・◆險ｭ險医ｒ菫ｮ豁｣縺励�『riter 讒狗ｯ画凾縺ｮ荳肴紛蜷育憾諷九ｒ髦ｲ縺舌�・

- 譬ｹ譛ｬ蜴溷屏:
  - 譌ｧ螳溯｣・〒縺ｯ `writer_store_header` 縺悟ｸｸ縺ｫ `()` 繧定ｿ斐＠縲～store_i32` 螟ｱ謨玲凾縺ｧ繧ょ他縺ｳ蜃ｺ縺怜・縺檎焚蟶ｸ繧呈､懷・縺ｧ縺阪↑縺九▲縺溘�・
  - `writer_new_handle` 縺ｧ繝倥ャ繝�蛻晄悄蛹悶↓螟ｱ謨励＠縺ｦ繧よ・蜉滓桶縺・↓縺ｪ繧翫≧繧玖ｨｭ險医□縺｣縺溘�・

- 螟画峩:
  - `stdlib/kp/kpwrite.nepl`
    - `writer_store_header` 縺ｮ霑斐ｊ蛟､繧・`Result<(),str>` 縺ｫ螟画峩縲・
    - `writer_new_handle` 縺ｮ 5 縺､縺ｮ繝倥ャ繝�譖ｸ縺崎ｾｼ縺ｿ繧帝�先ｬ｡ `match` 縺ｧ讀懆ｨｼ縺励�∝､ｱ謨玲凾縺ｯ遒ｺ菫晄ｸ医∩鬆伜沺繧定ｧ｣謾ｾ縺励※ `Err` 霑泌唆縲・
    - `flush/put/write` 邉ｻ縺ｮ髟ｷ縺墓峩譁ｰ邂・園繧・`Result` 繧呈・遉ｺ逧・↓蜿励￠繧区ｧ矩��縺ｸ邨ｱ荳�縲・

- 繝・せ繝・
  - `node nodesrc/tests.js -i stdlib/kp/kpwrite.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i tests/kp.n.md --no-tree -o /tmp/tests-kp-writer-header-result-v1.json -j 15`
  - 邨先棡: `217/217 pass`
  - `node nodesrc/tests.js -i tests/memory_safety.n.md -i tests/kp.n.md -i stdlib/core/mem.nepl -i stdlib/kp/kpread.nepl -i stdlib/kp/kpread_core.nepl -i stdlib/kp/kpwrite.nepl --no-tree -o /tmp/tests-memory-kp-v10.json -j 15`
  - 邨先棡: `226/226 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺB2: fn螳夂ｾｩ譎ゅが繝ｼ繝舌・繝ｭ繝ｼ繝臥・蜷医・繧ｸ繧ｧ繝阪Μ繧ｯ繧ｹ蜷悟�､菫ｮ豁｣)

- 逶ｮ逧・
  - `D3087`・・unction signature does not match any overload・峨・隱､讀懷・繧偵�√ず繧ｧ繝阪Μ繧ｯ繧ｹ鄂ｲ蜷咲・蜷医・譬ｹ譛ｬ縺九ｉ隗｣豸医☆繧九�・
- 譬ｹ譛ｬ蜴溷屏:
  - `fn` 螳夂ｾｩ辣ｧ蜷医〒 `same_type` 繧堤峩謗･菴ｿ縺・→縲∵悴譚溽ｸ帛梛螟画焚縺ｮ繝ｩ繝吶Ν荳�閾ｴ縺ｫ萓晏ｭ倥＠縲∃ｱ蜷悟�､・亥梛繝代Λ繝｡繝ｼ繧ｿ蜷阪・蟾ｮ・峨ｒ險ｱ螳ｹ縺ｧ縺阪★螟ｱ謨励＠縺溘�・
  - 縺輔ｉ縺ｫ辣ｧ蜷育畑縺ｫ菴懊ｋ鄂ｲ蜷榊梛 `sig_ty` 縺・`type_params` 縺ｪ縺励〒讒狗ｯ峨＆繧後※縺翫ｊ縲√ず繧ｧ繝阪Μ繧ｯ繧ｹ髢｢謨ｰ縺ｮ鄂ｲ蜷阪く繝ｼ縺ｨ荳肴紛蜷医ｒ襍ｷ縺薙＠縺ｦ縺・◆縲・
- 螟画峩:
  - `nepl-core/src/typecheck.rs`
    - `function_signature_string` 繧偵ず繧ｧ繝阪Μ繧ｯ繧ｹ豁｣隕丞喧繧ｭ繝ｼ逕滓・縺ｸ螟画峩・・$T0, $T1...` 縺ｸ豁｣隕丞喧・峨�・
    - `signature_type_string` 繧定ｿｽ蜉�縺励�・未謨ｰ繧ｷ繧ｰ繝阪メ繝｣豈碑ｼ・ｰら畑縺ｮ蝙区枚蟄怜・蛹悶ｒ蟆主・縲・
    - `fn` 螳夂ｾｩ辣ｧ蜷域凾縺ｮ `sig_ty` 繧偵�～f.type_params` 繧貞性繧� `ctx.function(type_params, params, result, effect)` 縺ｧ讒狗ｯ峨☆繧九ｈ縺・ｿｮ豁｣縲・
    - 譌｢蟄倥・繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝牙�呵｣懃・蜷茨ｼ・function_signature_string` 豈碑ｼ・ｼ峨ｒ邯ｭ謖√＠縺､縺､縲√ず繧ｧ繝阪Μ繧ｯ繧ｹ蜷悟�､豈碑ｼ・・邊ｾ蠎ｦ繧呈隼蝟・�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-move-overload-after-final-fix.json -j 15`
  - 邨先棡: `272/272 pass`
  - `node nodesrc/tests.js -i tests/compile_fail_diag_location.n.md --no-tree -o /tmp/tests-compile-fail-diag-location-after-final-fix.json -j 15`
  - 邨先棡: `207/207 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-after-final-fix.json -j 15`
  - 邨先棡: `783/783 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺB2: 髢｢謨ｰ鄂ｲ蜷肴ｯ碑ｼ・・譁・ｭ怜・萓晏ｭ倥ｒ謗帝勁)

- 逶ｮ逧・
  - 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝・hoist髢｢騾｣縺ｧ谿九▲縺ｦ縺・◆鄂ｲ蜷咲・蜷医・譁・ｭ怜・豈碑ｼ・ｒ蟒・ｭ｢縺励�∝梛讒矩��豈碑ｼ・∈邨ｱ荳�縺吶ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - `remove_duplicate_func`, `lookup_func_symbol`, `find_same_signature_func`, `fn` 螳夂ｾｩ譎ら・蜷医′譁・ｭ怜・繧ｭ繝ｼ豈碑ｼ・↓萓晏ｭ倥＠縺ｦ縺翫ｊ縲∝梛螟画焚蜷阪ｄ逕滓・鬆・ｺ丞ｷｮ縺ｧ荳榊ｮ牙ｮ壼喧縺吶ｋ菴吝慍縺後≠縺｣縺溘�・
- 螟画峩:
  - `nepl-core/src/typecheck.rs`
    - `same_function_signature` 繧定ｿｽ蜉�縺励�・未謨ｰ蝙九・繧ｷ繧ｰ繝阪メ繝｣蜷悟�､・医ず繧ｧ繝阪Μ繧ｯ繧ｹﾎｱ蜷悟�､蜷ｫ繧�・峨ｒ蝙区ｧ矩��縺ｧ蛻､螳壹�・
    - `same_type_with_signature_generics` 繧定ｿｽ蜉�縺励�∝梛繝代Λ繝｡繝ｼ繧ｿ蟇ｾ蠢懆｡ｨ・・->B/B->A・峨ｒ謖√▲縺溷・蟶ｰ豈碑ｼ・ｒ螳溯｣・�・
    - 莉･荳九ｒ譁・ｭ怜・豈碑ｼ・°繧・`same_function_signature` 縺ｸ鄂ｮ謠・
      - `fn` 螳夂ｾｩ譎ゅ・驕手ｲ�闕ｷ蛟呵｣憺∈謚・
      - `Env::remove_duplicate_func`
      - `Env::lookup_func_symbol`
      - `find_same_signature_func`
      - `find_nonshadow_same_signature_func`
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_effect.n.md -i tests/overload.n.md --no-tree -o /tmp/tests-move-overload-after-same-signature-api.json -j 15`
  - 邨先棡: `272/272 pass`
  - `node nodesrc/tests.js -i tests/compile_fail_diag_location.n.md --no-tree -o /tmp/tests-compile-fail-diag-location-after-same-signature-api.json -j 15`
  - 邨先棡: `207/207 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-stdlib-after-same-signature-api.json -j 15`
  - 邨先棡: `783/783 pass`
# 2026-03-05 菴懈･ｭ繝｡繝｢ (`move_check.n.md::doctest#4` 縺ｮ險ｺ譁ｭID荳堺ｸ�閾ｴ繧剃ｸ頑ｵ√〒菫ｮ豁｣)

- 逶ｮ逧・
  - `tests + stdlib` 蜈ｨ菴薙〒蜚ｯ荳�螟ｱ謨励＠縺ｦ縺・◆ `tests/move_check.n.md::doctest#4` 縺ｮ `diag_id: 3065` 荳堺ｸ�閾ｴ繧偵�∝�ｴ蠖薙◆繧翫〒縺ｯ縺ｪ縺上ユ繧ｹ繝郁ｨ倩ｿｰ縺ｮ荳頑ｵ∵紛蛯吶〒隗｣豸医☆繧九�・
- 蜴溷屏:
  - 譌｢蟄倥こ繝ｼ繧ｹ縺・`#target core` + `core/math` 萓晏ｭ倥・譖ｸ縺肴婿縺ｧ縲～loop move` 譛ｬ菴捺､懆ｨｼ繧医ｊ蜑阪↓ `D3016` 邉ｻ縺ｮ繧ｹ繧ｿ繝・け讀懈渊繧ｨ繝ｩ繝ｼ繧貞・陦檎匱逕溘＆縺帙※縺・◆縲・
  - 邨先棡縺ｨ縺励※縲∵э蝗ｳ縺励※縺・◆ `D3065`・・TypeLoopPotentiallyMovedValue`・峨↓蛻ｰ驕斐＠縺ｪ縺九▲縺溘�・
- 蟇ｾ蠢・
  - `tests/move_check.n.md` 縺ｮ `move_in_loop`・・octest#4・峨ｒ縲～loop` 蜷域ｵ√〒縺ｮ moved 蛟､蜀榊茜逕ｨ縺�縺代ｒ讀懆ｨｼ縺吶ｋ譛�蟆上こ繝ｼ繧ｹ縺ｫ鄂ｮ謠帙�・
  - `#target core` / `core/math` 萓晏ｭ倥ｒ髯､蜴ｻ縺励�～bool` 繝輔Λ繧ｰ譖ｴ譁ｰ (`set c false`) 縺ｧ 1 蝗槭Ν繝ｼ繝励ｒ讒区・縲・
  - `consume` 縺ｯ `()->()` 縺ｫ縺励�～D3016` 縺ｮ繝弱う繧ｺ繧呈賜髯､縲・
  - 譛�蠕後↓ `consume t` 繧堤ｽｮ縺阪�～loop` 蜀・move 縺ｮ蜷域ｵ√〒 `D3065` 繧貞ｮ牙ｮ壼・迴ｾ縺吶ｋ蠖｢縺ｫ蝗ｺ螳壹�・
- 螳滓命繝・せ繝・
  - `node nodesrc/tests.js -i tests/move_check.n.md --no-tree -o /tmp/tests-move-check-after-fix.json -j 15` -> `217/217 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-movecheck-fix.json -j 15` -> `785/785 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (trait capability 縺ｮ enum 蛹・ typecheck 譁・ｭ怜・萓晏ｭ倥・髯､蜴ｻ)

- 逶ｮ逧・
  - `todo.md` 繝輔ぉ繝ｼ繧ｺB2縺ｫ豐ｿ縺｣縺ｦ縲》rait capability 蛻､螳壹・雋ｬ蜍吶ｒ `typecheck` 縺九ｉ蜑肴ｮｵ縺ｸ蟇・○繧九�・
  - `typecheck` 蜀・・ `copy/clone` 譁・ｭ怜・繝代・繧ｹ繧貞炎髯､縺励�、ST 縺ｮ capability enum 繧堤峩謗･蜃ｦ逅・☆繧九�・
- 螟画峩:
  - `nepl-core/src/ast.rs`
    - `TraitCapability` enum 繧定ｿｽ蜉� (`Copy` / `Clone` / `Unknown(String)` )縲・
    - `TraitDef.capabilities` 繧・`Vec<String>` 縺九ｉ `Vec<TraitCapability>` 縺ｫ螟画峩縲・
  - `nepl-core/src/parser.rs`
    - `#capability` 繧・parser 谿ｵ縺ｧ enum 蛹悶☆繧・`parse_trait_capability` 繧定ｿｽ蜉�縲・
  - `nepl-core/src/typecheck.rs`
    - `parse_trait_capability(&str)` 縺ｨ譁・ｭ怜・豈碑ｼ・ｒ蜑企勁縲・
    - AST enum 繧堤峩謗･隱ｭ縺ｿ縲～Unknown` 縺ｮ縺ｿ `D3096` 繧貞・縺呎ｧ区・縺ｫ螟画峩縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/move_check.n.md -i tests/overload.n.md -i tests/move_effect.n.md --no-tree -o /tmp/tests-trait-capability-targeted.json -j 15` -> `285/285 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-after-trait-cap-enum.json -j 15` -> `785/785 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: stdlib/std 螳牙・蛹悶・逹�謇・

- 逶ｮ逧・
  - `core/mem` 螳牙・ API 蟆主・蠕後・蠕檎ｶ壹→縺励※縲～stdlib/std`・・fs` / `stdio` / `env/cliarg`・峨ｒ蜷御ｸ�繝｢繝・Ν縺ｸ遘ｻ陦後☆繧九�・
  - 逕・`alloc_raw` 逶ｴ謗･蛻ｩ逕ｨ縺ｨ證鈴ｻ吝､ｱ謨礼ｵ瑚ｷｯ繧呈ｮｵ髫守噪縺ｫ蜑頑ｸ帙☆繧九�・

- 騾ｲ謐・
  - `stdlib/std/fs.nepl`
    - `fs_alloc` / `fs_free` 繧定ｿｽ蜉�縲・
    - `fs_open_read` 縺ｮ `fd_out` 遒ｺ菫昴ｒ `Result` 蛹悶＠縲∬ｧ｣謾ｾ繧呈・遉ｺ蛹悶�・
    - `fs_read_fd_bytes` 縺ｮ `tmp/iov/nread` 遒ｺ菫昴ｒ `Result` 騾｣骼門喧縺励�∝・蛻・ｲ舌〒隗｣謾ｾ縺吶ｋ蠖｢縺ｸ螟画峩縲・
  - `stdlib/std/stdio.nepl`
    - 譛ｪ逹�謇具ｼ域ｬ｡谿ｵ縺ｧ `print/read_all/read_line/print_i32` 縺ｮ荳�譎る�伜沺遒ｺ菫昴ｒ螳牙・蛹紋ｺ亥ｮ夲ｼ峨�・
  - `stdlib/std/env/cliarg.nepl`
    - 譛ｪ逹�謇具ｼ域ｬ｡谿ｵ縺ｧ `args_sizes_get/args_get` 蜻ｨ霎ｺ縺ｮ遒ｺ菫晏､ｱ謨励→隗｣謾ｾ譁ｹ驥昴ｒ謨ｴ逅・ｺ亥ｮ夲ｼ峨�・

- 繝｡繝｢:
  - `fs` 蜊倅ｽ薙・螳溯｡檎ｳｻ繝・せ繝医・蜈･蜉帛ｾ・■繧ｱ繝ｼ繧ｹ繧貞性繧�縺溘ａ縲∽ｻ雁ｾ後・髱槫ｯｾ隧ｱ繧ｻ繝・ヨ縺ｧ蝗槫ｸｰ遒ｺ隱阪☆繧九�・

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: codegen 蜑肴ｮｵ險ｺ譁ｭ縺ｮ蜈ｱ騾壼喧繝ｻ隨ｬ荳�谿ｵ)

- 逶ｮ逧・
  - `codegen_llvm` 蜀・↓谿九▲縺ｦ縺・◆ `#target` 蛟句挨讀懆ｨｼ繧・backend 縺九ｉ謦､蜴ｻ縺励�∝燕谿ｵ蜈ｱ騾・precheck 縺ｸ髮・ｴ・☆繧九�・
  - `compile_module` 縺ｨ LLVM IR 逕滓・邨瑚ｷｯ縺ｧ蜷後§讀懆ｨｼ蜈･蜿｣繧剃ｽｿ縺・�『asm/llvm 縺ｮ險ｺ譁ｭ隕丞援蟾ｮ蛻・ｒ邵ｮ蟆上☆繧九�・

- 螟画峩:
  - `nepl-core/src/target_precheck.rs`
    - `precheck_module_target_directives` 繧定ｿｽ蜉�・・UnknownTargetDirective` / `MultipleTargetDirective` 繧貞・騾夂函謌撰ｼ峨�・
    - `precheck_module_before_codegen` 繧定ｿｽ蜉�・・arget directive + raw body precheck 縺ｮ蜷域・・峨�・
  - `nepl-core/src/codegen_llvm.rs`
    - `validate_target_directive_for_llvm` / `is_known_target_name` 繧貞炎髯､縲・
    - `emit_ll_from_module_for_target` 蜈･蜿｣繧・`precheck_module_before_codegen` 縺ｸ邨ｱ荳�縲・
  - `nepl-core/src/compiler.rs`
    - `compile_module` 縺ｮ precheck 蜻ｼ縺ｳ蜃ｺ縺励ｒ `precheck_module_before_codegen` 縺ｸ鄂ｮ謠帙�・

- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/llvm_target.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-unify-step2-focus.json -j 15`
    - 邨先棡: `5/5 pass`
  - 陬懆ｶｳ:
    - `tests/neplg2.n.md` 縺ｧ縺ｯ譌｢遏･縺ｮ runtime 蛛ｴ `Maximum call stack size exceeded` 縺梧ｮ句ｭ假ｼ井ｻ雁屓螟画峩遽・峇螟厄ｼ峨�・

# 2026-03-05 菴懈･ｭ繝｡繝｢ (tests.js: argv 繝｡繧ｿ蟇ｾ蠢懆ｿｽ蜉�)

- 逶ｮ逧・
  - `stdin/stdout` 縺ｫ蜉�縺医※ doctest 縺九ｉ CLI 蠑墓焚繧呈ｳｨ蜈･縺ｧ縺阪ｋ繧医≧縺ｫ縺励�～stdlib/tests/cliarg.n.md` 繧偵ユ繧ｹ繝亥庄閭ｽ縺ｫ縺吶ｋ縲・

- 螟画峩:
  - `nodesrc/parser.js`
    - doctest 繝｡繧ｿ縺ｫ `argv:` 繧定ｿｽ蜉�縲・
    - `parseMetaValue` 縺・`[` / `{` 蟋九∪繧翫・ JSON 繧りｧ｣驥医☆繧九ｈ縺・僑蠑ｵ・・argv: ["a","b"]` 繧帝・蛻励→縺励※蜿門ｾ暦ｼ峨�・
  - `nodesrc/tests.js`
    - 繝・せ繝医こ繝ｼ繧ｹ讒矩��縺ｫ `argv` 繧定ｿｽ蜉�縲・
    - wasm 繝ｯ繝ｼ繧ｫ繝ｼ隕∵ｱゅ∈ `argv` 繧剃ｼ晄成縲・
    - llvm 螳溯｡梧凾縺ｫ繧・`argv` 繧貞ｮ溯｡悟ｼ墓焚縺ｨ縺励※貂｡縺吶�・
  - `nodesrc/run_test.js`
    - WASI 螳溯｡梧凾縺ｮ args 繧・`argv` 縺九ｉ蜿励￠蜿悶ｊ縲～[wasmPath, ...argv]` 縺ｧ襍ｷ蜍輔�・
  - `stdlib/tests/cliarg.n.md`
    - `neplg2:test[assert_io]` + `argv` + `stdout` 縺ｧ `cliarg_count` 讀懆ｨｼ繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�縲・

- 讀懆ｨｼ:
  - parser 蜊倅ｽ鍋｢ｺ隱・
    - `node -e "const p=require('./nodesrc/parser'); const r=p.parseFile('stdlib/tests/cliarg.n.md'); console.log(Array.isArray(r.doctests[0].argv), JSON.stringify(r.doctests[0].argv));"`
    - 邨先棡: `true ["--flag","value"]`
  - run_test 逶ｴ螳溯｡檎｢ｺ隱・
    - `argv=["a","b"]` 縺ｧ `cliarg_count` 蜃ｺ蜉帙′ `"3"`
    - `argv=[]` 縺ｧ `cliarg_count` 蜃ｺ蜉帙′ `"1"`
  - tests.js 蜊倅ｽ鍋｢ｺ隱・
    - `node nodesrc/tests.js -i stdlib/tests/cliarg.n.md --no-stdlib --no-tree -o /tmp/tests-cliarg-only-argv.json -j 1 --assert-io`
    - 邨先棡: `2/2 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: stdlib/std 螳牙・蛹悶・螳御ｺ・→蜈ｨ菴灘屓蟶ｰ)

- 逶ｮ逧・
  - `stdlib/std` 縺ｮ螳牙・蛹門ｯｾ雎｡・・fs` / `stdio` / `env/cliarg`・峨ｒ `Result` 繝吶・繧ｹ縺ｸ謠・∴縲～alloc_raw` 逶ｴ謗･蛻ｩ逕ｨ縺ｮ蜑頑ｸ帙→螟ｱ謨礼ｵ瑚ｷｯ縺ｮ譏守､ｺ蛹悶ｒ螳御ｺ・☆繧九�・

- 螟画峩:
  - `stdlib/std/fs.nepl`
    - `__fs_copy_to_cstr` 繧・`Result<i32,i32>` 蛹悶�・
    - `wasi_path_open` 縺ｧ遒ｺ菫晏､ｱ謨励ｒ `Err` 縺ｧ霑斐＠縲∵・蜉滓凾 `cpath` 繧貞ｿ・★隗｣謾ｾ縲・
    - `fs_bytes_to_string` 繧・`fs_alloc` 繝吶・繧ｹ縺ｸ螟画峩縲・
    - if 繝ｬ繧､繧｢繧ｦ繝亥・縺ｮ荳崎ｦ・`;` 繧帝勁蜴ｻ・亥ｼ乗綾繧雁�､謨ｴ蜷茨ｼ峨�・
  - `stdlib/std/stdio.nepl`
    - `print_i32` 縺ｮ荳�譎る�伜沺遒ｺ菫昴ｒ `std_alloc/std_free` 繝吶・繧ｹ縺ｸ螟画峩縲・
    - `read_all` 縺ｮ if 蠑上〒 `else out;` 縺ｫ縺ｪ縺｣縺ｦ縺・◆邂・園繧・`out` 縺ｫ菫ｮ豁｣縺励�～expr; -> ()` 縺ｫ繧医ｋ蝙倶ｸ肴紛蜷医ｒ隗｣豸医�・
  - `stdlib/std/env/cliarg.nepl`
    - `cstr_to_str` 縺ｮ遒ｺ菫昴ｒ `cli_alloc` 繝吶・繧ｹ縺ｸ螟画峩縺励�∝､ｱ謨玲凾繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ繧呈・遉ｺ縲・

- 譬ｹ譛ｬ蜴溷屏縺ｨ菫ｮ豁｣譁ｹ驥・
  - 蜈ｨ菴灘屓蟶ｰ縺ｧ `tests/stdin.n.md` 縺ｮ縺ｿ wasm stack mismatch 縺檎匱逕溘�・
  - 蜴溷屏縺ｯ `read_all` 縺ｮ `if` 蠑・else 蛛ｴ縺・`out;` 縺ｨ縺ｪ縺｣縺ｦ縺翫ｊ縲∽ｻ墓ｧ倥←縺翫ｊ `()` 縺ｫ蛹悶￠縺ｦ縺・◆縺薙→縲・
  - 蝣ｴ蠖薙◆繧翫〒繧ｳ繝ｼ繝牙・隗｣縺帙★縲∝ｼ上・謌ｻ繧雁�､隕丞援・・lan.md 縺ｮ `;` 莉墓ｧ假ｼ峨↓豐ｿ縺｣縺ｦ `out` 縺ｸ菫ｮ豁｣縺励※譬ｹ譛ｬ隗｣豸医�・

- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i stdlib/tests/fs.n.md --no-stdlib --no-tree -o /tmp/tests-fs-safe-phase.json -j 15` -> `1/1 pass`
  - `node nodesrc/tests.js -i stdlib/tests/cliarg.n.md -i tests/stdout.n.md -i stdlib/tests/fs.n.md --no-stdlib --no-tree -o /tmp/tests-std-safe-regression.json -j 15 --assert-io` -> `9/9 pass`
  - `node nodesrc/tests.js -i tests/stdin.n.md --no-tree -o /tmp/tests-stdin-focus.json -j 15 --assert-io` -> `210/210 pass`
  - `node nodesrc/tests.js -i tests -i stdlib --no-tree -o /tmp/tests-full-stdlib-std-safety-phase.json -j 15` -> `788/788 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (MemPtr/RegionToken 蜀崎ｪｿ譟ｻ縺ｨ _raw 蟒・ｭ｢譁ｹ驥昴・蜀肴紛逅・

- 隱ｿ譟ｻ逶ｮ逧・
  - `MemPtr/RegionToken` 蟆主・蠕後・谿句ｭ倡函繝昴う繝ｳ繧ｿ萓晏ｭ倥→ `_raw` 萓晏ｭ倥ｒ蜈ｨ菴薙〒譽壼査縺励＠縲∽ｸ頑ｵ∝━蜈医〒縺ｮ遘ｻ陦碁�・ｒ蜀咲｢ｺ螳壹☆繧九�・

- 迴ｾ迥ｶ隕∫ｴ・
  - `core/mem.nepl` 縺ｫ縺ｯ `MemPtr<T>` / `RegionToken<T>` 縺ｨ `region_ptr_at/alloc_region/dealloc_region` 縺悟ｮ溯｣・ｸ医∩縲・
  - `kpread/kpwrite` 縺ｯ蜈ｬ髢区ｧ矩��菴薙′ `RegionToken<u8>` 繧剃ｿ晄戟縺吶ｋ蠖｢縺ｾ縺ｧ遘ｻ陦梧ｸ医∩縲・
  - 縺溘□縺・`core/mem` 蜈ｬ髢矩擇縺ｫ縺ｯ `alloc_raw/dealloc_raw/realloc_raw` 縺ｨ `load/store(i32)` 逕溘・繧､繝ｳ繧ｿ迚医′谿句ｭ倥�・
  - `stdlib/alloc` / `stdlib/kp` / `stdlib/nm` / `platforms/wasix` / examples/tests 縺ｫ縺ｯ `_raw` 蜻ｼ縺ｳ蜃ｺ縺励′螟壽焚谿句ｭ倥�・
  - `nepl-core` 蛛ｴ縺ｫ繧・`_raw` 蜷堺ｾ晏ｭ倥′谿句ｭ假ｼ・monomorphize.rs`, `codegen_wasm.rs`, `codegen_llvm.rs`・峨�・

- 譬ｹ譛ｬ隱ｲ鬘・
  - `_raw` 蟒・ｭ｢縺ｯ stdlib 蛛ｴ縺�縺代〒縺ｯ螳御ｺ・○縺壹�…ompiler 蛛ｴ縺ｮ helper 隗｣豎ｺ繝ｭ繧ｸ繝・け繧貞・縺ｫ荳�闊ｬ蛹悶☆繧句ｿ・ｦ√′縺ゅｋ縲・
  - `core/mem` 縺ｮ逕溘・繧､繝ｳ繧ｿAPI繧貞・縺ｫ蜑企勁縺吶ｋ縺ｨ縲∽ｸ区ｵ√Λ繧､繝悶Λ繝ｪ縺ｨ codegen 縺悟酔譎ょｴｩ螢翫☆繧九◆繧√�∵ｮｵ髫守ｧｻ陦後′蠢・ｦ√�・

- 蜀咲｢ｺ螳壹＠縺溷ｮ溯｣・�・ｺ擾ｼ井ｸ頑ｵ∝━蜈茨ｼ・
  1. compiler 蛛ｴ `_raw` 蜷堺ｾ晏ｭ倥・髯､蜴ｻ・・monomorphize` / `codegen_wasm` / `codegen_llvm`・峨�・
  2. `core/mem` 繧貞ｮ牙・API蜈ｬ髢矩擇縺ｫ邨ｱ荳�縺励�∫函繝昴う繝ｳ繧ｿAPI繧貞・驛ｨ莠呈鋤螻､縺ｸ髫秘屬縲・
  3. `stdlib/alloc` 縺ｨ `kp` 繧・`MemPtr/RegionToken` + `Result/Option` 蜑肴署縺ｸ蜈ｨ髱｢遘ｻ陦後�・
  4. `stdlib/std` / `stdlib/nm` / tutorials/examples 縺ｮ鬆・〒霑ｽ髫冗ｧｻ陦後�・
  5. 譛�蠕後↓ `_raw` 縺ｨ逕溘・繧､繝ｳ繧ｿ蜈ｬ髢矩未謨ｰ繧貞炎髯､縺励�…ompile_fail 蝗槫ｸｰ繧貞崋螳壹�・
# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: wasm signature 險ｺ譁ｭ繧・codegen 蜑肴ｮｵ縺ｸ遘ｻ蜍・

- 逶ｮ逧・
  - `codegen_wasm` 蜀・〒蜃ｺ縺励※縺・◆鄂ｲ蜷咲ｳｻ險ｺ譁ｭ繧貞燕谿ｵ繝代せ縺ｸ遘ｻ縺励�～codegen蛻ｰ驕疲凾縺ｯ讀懆ｨｼ貂医∩` 縺ｮ險ｭ險医∈蟇・○繧九�・
  - wasm/llvm 蜈ｱ騾壼喧譁ｹ驥昴・隨ｬ荳�谿ｵ縺ｨ縺励※縲｜ackend 逶ｴ荳玖ｨｺ譁ｭ縺ｮ蜑頑ｸ帙ｒ騾ｲ繧√ｋ縲・
- 螟画峩:
  - `nepl-core/src/passes/codegen_precheck.rs` 繧定ｿｽ蜉�縲・
    - `precheck_wasm_codegen` 繧貞ｮ溯｣・＠縲∽ｻ･荳九ｒ蜑肴ｮｵ縺ｧ讀懈渊:
      - extern 鄂ｲ蜷・(`D4001`)
      - 蛻ｰ驕泌庄閭ｽ髢｢謨ｰ縺ｮ鄂ｲ蜷・(`D4002`)
  - `nepl-core/src/compiler.rs`
    - `insert_drops` 蠕後・wasm emit 蜑阪↓ `precheck_wasm_codegen` 繧貞ｮ溯｡後�・
    - 繧ｨ繝ｩ繝ｼ險ｺ譁ｭ縺後≠繧後・ codegen 縺ｸ騾ｲ縺ｾ縺・`CoreError::Diagnostics` 繧定ｿ斐☆縲・
  - `nepl-core/src/codegen_wasm.rs`
    - 鄂ｲ蜷堺ｸ堺ｸ�閾ｴ譎ゅ・ `D4001/D4002` 逕滓・繧貞炎髯､縺励�∝燕谿ｵ讀懈渊蜑肴署縺ｧ繧ｹ繧ｭ繝・・蜃ｦ逅・↓螟画峩縲・
  - `tests/raw_body_precheck.n.md`
    - `D4001/D4002` 繧貞ｮ牙ｮ壼・迴ｾ縺吶ｋ `compile_fail` 繧ｱ繝ｼ繧ｹ繧定ｿｽ蜉�繝ｻ隱ｿ謨ｴ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-signature-v5.json -j 15` -> `4/4 pass`
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-signature-v6.json -j 15` -> `7/7 pass`
# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: D4003 繧・codegen 蜑肴ｮｵ縺ｸ遘ｻ蜍・

- 逶ｮ逧・
  - `CodegenWasmMissingReturnValue (D4003)` 繧・backend 萓晏ｭ倩ｨｺ譁ｭ縺九ｉ蜑肴ｮｵ險ｺ譁ｭ縺ｸ遘ｻ縺励�…odegen 蛻ｰ驕疲凾縺ｮ蜑肴署繧貞ｼｷ蛹悶☆繧九�・
- 螟画峩:
  - `nepl-core/src/passes/codegen_precheck.rs`
    - 蛻ｰ驕泌庄閭ｽ髢｢謨ｰ縺ｮ `HirBody::Block` 縺ｫ縺､縺・※縲・
      - 謌ｻ繧雁梛縺・`Unit` 莉･螟・
      - 譛�邨ら噪縺ｪ髱・drop 陦後′蛟､繧定ｿ斐＆縺ｪ縺・
      蝣ｴ蜷医↓ `D4003` 繧貞燕谿ｵ縺ｧ蜃ｺ縺呎､懈渊繧定ｿｽ蜉�縲・
  - `nepl-core/src/codegen_wasm.rs`
    - `lower_user` 蜀・・ `D4003` 險ｺ譁ｭ逕滓・繧貞炎髯､縲・
    - 縺薙％縺ｫ蛻ｰ驕斐＠縺溷�ｴ蜷医・蜀・Κ荳肴紛蜷医→縺励※ `panic!`・・recheck 縺ｧ蠑ｾ縺九ｌ繧句燕謠撰ｼ峨↓螟画峩縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-signature-v7.json -j 15` -> `7/7 pass`
# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: D4005 繧・codegen 蜑肴ｮｵ縺ｸ遘ｻ蜍・

- 逶ｮ逧・
  - `CodegenWasmLlvmIrBodyNotSupported (D4005)` 繧・backend 蛛ｴ險ｺ譁ｭ縺九ｉ蜑肴ｮｵ險ｺ譁ｭ縺ｸ遘ｻ縺励�…odegen 縺ｮ雋ｬ蜍吶ｒ邵ｮ蟆上☆繧九�・
- 螟画峩:
  - `nepl-core/src/passes/codegen_precheck.rs`
    - 蛻ｰ驕泌庄閭ｽ髢｢謨ｰ縺ｧ `HirBody::LlvmIr` 縺梧ｮ九▲縺ｦ縺・ｋ蝣ｴ蜷医↓ `D4005` 繧貞燕谿ｵ縺ｧ蜃ｺ縺呎､懈渊繧定ｿｽ蜉�縲・
  - `nepl-core/src/codegen_wasm.rs`
    - `HirBody::LlvmIr` 蛻・ｲ舌〒 `D4005` 繧堤函謌舌☆繧句・逅・ｒ蜑企勁縲・
    - precheck 騾夐℃蠕後・蜀・Κ荳肴紛蜷医→縺励※ `panic!` 縺ｫ螟画峩縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-signature-v8.json -j 15` -> `7/7 pass`
# 2026-03-06 菴懈･ｭ繝｡繝｢ (alloc/string: bool 縺ｨ蝓ｺ謨ｰ莉倥″謨ｴ謨ｰ譁・ｭ怜・螟画鋤縺ｮ謨ｴ逅・

- 逶ｮ逧・
  - 譁・ｭ怜・陦ｨ迴ｾ縺ｸ縺ｮ螟画鋤雋ｬ蜍吶ｒ `alloc/string` 縺ｫ髮・ｴ・＠縲～core/cast` 繧貞�､螟画鋤蟆ら畑縺ｫ菫昴▽縲・
  - 2 / 8 / 10 / 16 騾ｲ縺ｮ謨ｴ謨ｰ譁・ｭ怜・蛹悶・隗｣譫舌ｒ `alloc/string` 縺ｮ API 縺ｨ縺励※謠・∴繧九�・
- 螟画峩:
  - `stdlib/alloc/string.nepl`
    - `from_bool` 繧定ｿｽ蜉�縺励�｜ool 縺ｮ陦ｨ遉ｺ逕ｨ譁・ｭ怜・蛹悶ｒ `alloc/string` 縺ｫ邨ｱ荳�縲・
    - `from_i32` 繧・`from_i32_radix x 10` 邨檎罰縺ｸ螟画峩縲・
    - `to_i32` 繧・`to_i32_radix s 10` 邨檎罰縺ｸ螟画峩縲・
    - `from_i64` 繧・`from_i64_radix x 10` 邨檎罰縺ｸ螟画峩縲・
    - `to_i64` 繧・`to_i64_radix s 10` 邨檎罰縺ｸ螟画峩縲・
    - 譁ｰ隕上↓ `digit_to_char_lower` / `digit_from_char` / `validate_radix` 繧定ｿｽ蜉�縲・
    - 譁ｰ隕上↓ `from_i32_radix` / `to_i32_radix` / `from_i64_radix` / `to_i64_radix` 繧定ｿｽ蜉�縲・
    - 2 / 8 / 10 / 16 騾ｲ縺ｮ縺ｿ繧貞女逅・☆繧区婿驥昴ｒ繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝医↓譏手ｨ倥�・
    - `from_bool` / `from_i32` / 蝓ｺ謨ｰ莉倥″螟画鋤縺ｮ隱ｬ譏弱ｒ縲∫岼逧・・螳溯｣・・豕ｨ諢上・險育ｮ鈴㍼縺悟・縺九ｋ蠖｢縺ｸ謇区嶌縺阪〒譖ｴ譁ｰ縲・
  - `stdlib/std/test.nepl`
    - bool 縺ｮ譁・ｭ怜・蛹悶ｒ `from_bool` 縺ｫ邨ｱ荳�縲・
  - `tests/stdlib.n.md`
    - `from_i32_radix 10 2`
    - `from_i64_radix 255 16`
    - `to_i32_radix "1010" 2`
    - `to_i64_radix "Ff" 16`
    - 荳肴ｭ｣譯・/ 荳肴ｭ｣蝓ｺ謨ｰ
    繧・focused test 縺ｨ縺励※霑ｽ蜉�縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i /tmp/one-radix-format.n.md --no-stdlib --no-tree -o /tmp/one-radix-format-only.json -j 1` -> `1/1 pass`
  - `node nodesrc/tests.js -i /tmp/one-radix-parse.n.md --no-stdlib --no-tree -o /tmp/one-radix-parse-only.json -j 1` -> `1/1 pass`
  - `node nodesrc/tests.js -i tests/stdlib.n.md -i tutorials/getting_started/10_project_fizzbuzz.n.md --no-stdlib --no-tree -o /tmp/tests-string-radix-focused-v1.json -j 15` -> `13/13 pass`
- 蛻､譁ｭ:
  - `bool -> str` 縺ｯ蛟､螟画鋤縺ｧ縺ｯ縺ｪ縺乗枚蟄怜・陦ｨ迴ｾ蛹悶↑縺ｮ縺ｧ `core/cast` 縺ｧ縺ｯ縺ｪ縺・`alloc/string` 縺ｫ鄂ｮ縺上�・
  - 2 / 8 / 10 / 16 騾ｲ縺ｮ蝓ｺ謨ｰ謖・ｮ壹・譁・ｭ怜・ API 縺ｮ雋ｬ蜍吶↑縺ｮ縺ｧ縲～cast` 縺ｧ縺ｯ縺ｪ縺・`alloc/string` 縺ｫ鄂ｮ縺上�・
  - `core/cast` 縺ｫ縺ｯ謨ｰ蛟､/隲也炊/繝薙ャ繝・繝昴う繝ｳ繧ｿ縺ｮ蛟､螟画鋤縺�縺代ｒ谿九☆譁ｹ驥昴′荳�雋ｫ縺励※縺・ｋ縲・
- 譛ｪ螳・
  - `alloc/string.nepl` 繧・input 縺ｫ縺励◆ stdlib doctest 螳溯｡檎ｵ瑚ｷｯ縺ｯ蛻･騾疲紛逅・′蠢・ｦ√�・
  - `i128` 縺ｮ譁・ｭ怜・陦ｨ迴ｾ螟画鋤縺ｯ譛ｪ螳溯｣・�・

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: D4011 繧・codegen 蜑肴ｮｵ縺ｸ遘ｻ蜍・

- 逶ｮ逧・
  - `CodegenWasmUnsupportedIndirectSignature (D4011)` 繧・backend 蛛ｴ縺九ｉ蜑肴ｮｵ縺ｸ遘ｻ縺励�～call_indirect` 縺ｮ鄂ｲ蜷榊ｦ･蠖捺�ｧ繧・codegen 蜑阪↓遒ｺ螳壹☆繧九�・
- 螟画峩:
  - `nepl-core/src/passes/codegen_precheck.rs`
    - HIR 蠑上ｒ蜀榊ｸｰ襍ｰ譟ｻ縺励�～CallIndirect` 縺ｮ `params/result` 縺九ｉ `wasm_sig_ids` 繧定ｩ穂ｾ｡縲・
    - wasm 髱槫ｯｾ蠢懃ｽｲ蜷阪ｒ讀懷・縺励◆蝣ｴ蜷医↓ `D4011` 繧貞燕谿ｵ縺ｧ霑斐☆讀懈渊繧定ｿｽ蜉�縲・
  - `nepl-core/src/codegen_wasm.rs`
    - `CallIndirect` 蛻・ｲ舌・ `D4011` 險ｺ譁ｭ逕滓・繧貞炎髯､縺励�｝recheck 騾夐℃蠕後・蜀・Κ荳肴紛蜷医→縺励※ `panic!` 縺ｫ螟画峩縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-indirect-v5.json -j 15` -> `7/7 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: D4004 繧・codegen 蜑肴ｮｵ縺ｸ遘ｻ蜍・

- 逶ｮ逧・
  - `CodegenWasmRawLineParseError (D4004)` 繧・backend 蛛ｴ險ｺ譁ｭ縺九ｉ蜑肴ｮｵ險ｺ譁ｭ縺ｸ遘ｻ縺励�～#wasm` 逕溯｡後ヱ繝ｼ繧ｹ螟ｱ謨励ｒ codegen 蜑阪↓遒ｺ螳壹☆繧九�・
- 螟画峩:
  - `nepl-core/src/codegen_wasm.rs`
    - `HirBody::Wasm` 蛻・ｲ舌〒縺ｮ `D4004` 逕滓・繧貞炎髯､縲・
    - precheck 騾夐℃蠕後・蜀・Κ荳肴紛蜷医→縺励※ `panic!` 縺ｫ螟画峩縲・
    - `precheck_raw_wasm_body(func)` 繧定ｿｽ蜉�縺励�～parse_wasm_line` 螟ｱ謨玲凾縺ｫ `D4004` 繧定ｿ斐☆蜑肴ｮｵ逕ｨ繝倥Ν繝代ｒ螳溯｣・�・
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `precheck_wasm_codegen` 縺九ｉ `codegen_wasm::precheck_raw_wasm_body` 繧貞他縺ｳ蜃ｺ縺吶ｈ縺・､画峩縲・
  - `tests/raw_body_precheck.n.md`
    - `wasm_precheck_rejects_invalid_raw_line` 繧定ｿｽ蜉�・・diag_id: 4004`・峨�・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-rawline-v1.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: D4010 繧・codegen 蜑肴ｮｵ縺ｸ遘ｻ蜍・

- 逶ｮ逧・
  - `CodegenWasmMissingIndirectSignature (D4010)` 繧・backend 蛛ｴ險ｺ譁ｭ縺九ｉ蜑肴ｮｵ縺ｸ遘ｻ縺励�～CallIndirect` 縺ｮ蝙九そ繧ｯ繧ｷ繝ｧ繝ｳ荳肴紛蜷医ｒ codegen 蜑阪↓讀懈渊縺吶ｋ縲・
- 螟画峩:
  - `nepl-core/src/codegen_wasm.rs`
    - `collect_wasm_signature_set` 繧定ｿｽ蜉�縺励�『asm codegen 縺ｧ菴ｿ縺・未謨ｰ/extern/髢捺磁蜻ｼ縺ｳ蜃ｺ縺礼ｽｲ蜷埼寔蜷医ｒ蜈ｱ騾壼喧縲・
    - `CallIndirect` 蛻・ｲ舌・ `D4010` 險ｺ譁ｭ逕滓・繧貞炎髯､縺励�｝recheck 騾夐℃蠕後・蜀・Κ荳肴紛蜷医→縺励※ `panic!` 縺ｸ螟画峩縲・
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `collect_wasm_signature_set` 縺ｮ邨先棡繧剃ｽｿ縺・�～CallIndirect` 縺ｮ鄂ｲ蜷阪′蝙九そ繧ｯ繧ｷ繝ｧ繝ｳ蛟呵｣懊↓蟄伜惠縺吶ｋ縺九ｒ蜑肴ｮｵ縺ｧ讀懈渊縲・
    - 谺�關ｽ譎ゅ・ `D4010`縲・撼蟇ｾ蠢懃ｽｲ蜷阪・ `D4011` 縺ｨ縺励※蛻・屬縺励※霑斐☆縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-indirect-missing-v1.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: 蜿ら・隗｣豎ｺ邉ｻ wasm backend 險ｺ譁ｭ縺ｮ蜑頑ｸ・

- 逶ｮ逧・
  - `CodegenWasmStringLiteralNotFound (4006)` / `CodegenWasmUnknownVariable (4007)` /
    `CodegenWasmUnknownFunctionValue (4008)` / `CodegenWasmUnknownFunction (4009)` 繧・
    backend 險ｺ譁ｭ縺九ｉ螟悶＠縲∽ｸ頑ｵ・�夐℃蠕後・蜀・Κ荳肴紛蜷医→縺励※謇ｱ縺・�・
- 螟画峩:
  - `nepl-core/src/codegen_wasm.rs`
    - `LiteralStr/Var/FnValue/Call/Set` 縺ｧ縺ｮ荳願ｨ倩ｨｺ譁ｭ逕滓・繧貞炎髯､縲・
    - 蜷檎ｮ・園縺ｯ `panic!` 縺ｫ螟画峩縺励�…odegen 蛻ｰ驕疲凾縺ｯ隗｣豎ｺ貂医∩蜑肴署繧貞ｼｷ蛻ｶ縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-ref-invariant-v2.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: unknown intrinsic 險ｺ譁ｭ縺ｮ蜑肴ｮｵ蛹匁紛蜷・

- 逶ｮ逧・
  - `CodegenWasmUnknownIntrinsic (4012)` 繧・backend 險ｺ譁ｭ縺九ｉ螟悶＠縲（ntrinsic 蛻､螳夊ｲｬ蜍吶ｒ蜑肴ｮｵ縺ｸ蟇・○繧九�・
- 螟画峩:
  - `nepl-core/src/codegen_wasm.rs`
    - `is_supported_wasm_intrinsic` 繧定ｿｽ蜉�縺励※ wasm backend 縺悟女逅・☆繧・intrinsic 蜷阪ｒ譏守､ｺ蛹悶�・
    - intrinsic 譛ｪ遏･蛻・ｲ舌・ `D4012` 逕滓・繧貞炎髯､縺励�∝・驛ｨ荳肴紛蜷・`panic!` 縺ｸ螟画峩縲・
  - `nepl-core/src/passes/codegen_precheck.rs`
    - `HirExprKind::Intrinsic` 縺ｧ `is_supported_wasm_intrinsic` 繧剃ｽｿ逕ｨ縺励�∵悴遏･ intrinsic 繧貞燕谿ｵ讀懈渊縲・
  - `tests/raw_body_precheck.n.md`
    - 霑ｽ蜉�縺励◆ `diag_id:4012` 繧ｱ繝ｼ繧ｹ縺ｯ縲∝ｮ滄圀縺ｫ縺ｯ荳頑ｵ√・ `D3012`・・nknown intrinsic・峨〒蜈医↓螟ｱ謨励☆繧九◆繧∝炎髯､縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-unknown-intrinsic-v2.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: 讒狗ｯ牙梛 payload/field 縺ｮ backend 險ｺ譁ｭ蜑頑ｸ・

- 逶ｮ逧・
  - `CodegenWasmUnsupportedEnumPayloadType (4013)` /
    `CodegenWasmUnsupportedStructFieldType (4014)` /
    `CodegenWasmUnsupportedTupleElementType (4015)` 繧・backend 險ｺ譁ｭ縺九ｉ螟悶＠縲…odegen 蛻ｰ驕疲凾縺ｮ蝙区紛蜷亥燕謠舌ｒ譏守｢ｺ蛹悶☆繧九�・
- 螟画峩:
  - `nepl-core/src/codegen_wasm.rs`
    - `EnumConstruct` 縺ｨ `Match` 縺ｮ enum payload load/store縲～StructConstruct`縲～TupleConstruct` 縺ｮ
      髱槫ｯｾ蠢・valtype 蛻・ｲ舌ｒ `panic!` 縺ｫ螟画峩縲・
    - 荳願ｨ・4013/4014/4015 縺ｮ `diags.push(...with_id(...))` 繧貞炎髯､縲・
    - 縺薙ｌ縺ｫ繧医ｊ縲～codegen_wasm` 蜀・・ `CodegenWasm*` 險ｺ譁ｭ逕滓・縺ｯ precheck 繝倥Ν繝大・・・4004・峨・縺ｿ縺ｫ髯仙ｮ壹�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-backend-diag-clean-v1.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: llvm backend 縺ｮ隗｣豎ｺ貂医∩蜿ら・繧ｨ繝ｩ繝ｼ繧貞・驛ｨ荳肴紛蜷亥喧)

- 逶ｮ逧・
  - wasm 蛛ｴ縺ｨ蜷梧ｧ倥↓縲∝錐蜑崎ｧ｣豎ｺ/鄂ｲ蜷崎ｧ｣豎ｺ貂医∩縺ｧ縺ゅｋ縺ｹ縺榊盾辣ｧ邉ｻ繧ｨ繝ｩ繝ｼ繧・backend 險ｺ譁ｭ雋ｬ蜍吶°繧牙､悶☆縲・
- 螟画峩:
  - `nepl-core/src/codegen_llvm.rs`
    - `Var` 縺ｮ unknown 螟画焚蛻・ｲ舌ｒ `panic!` 蛹悶�・
    - `Set` 縺ｮ unknown 螟画焚蛻・ｲ舌ｒ `panic!` 蛹悶�・
    - `FnValue` 縺ｮ unknown 髢｢謨ｰ蛟､蛻・ｲ舌ｒ `panic!` 蛹悶�・
    - `Call` 縺ｮ missing function signature 蛻・ｲ舌ｒ `panic!` 蛹悶�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/tests.js -i tests/raw_body_precheck.n.md -i tests/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-precheck-wasm-llvm-invariant-v1.json -j 15` -> `8/8 pass`

# 2026-03-05 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺD: monomorphize 縺ｮ runtime helper 蛟呵｣懊ワ繝ｼ繝峨さ繝ｼ繝蛾寔邏・

- 逶ｮ逧・
  - `_raw` 謦､蜴ｻ繝輔ぉ繝ｼ繧ｺ縺ｫ蛯吶∴縲～monomorphize` 蜀・・ runtime helper 蛟呵｣懷錐繝上・繝峨さ繝ｼ繝峨ｒ荳�邂・園縺ｫ髮・ｴ・☆繧九�・
- 螟画峩:
  - `nepl-core/src/runtime_helpers.rs` 繧定ｿｽ蜉�縲・
    - `ALLOC_CANDIDATES`
    - `DEALLOC_CANDIDATES`
    - `REALLOC_CANDIDATES`
  - `nepl-core/src/lib.rs` 縺ｫ `runtime_helpers` 繧貞・髢九�・
  - `nepl-core/src/monomorphize.rs`
    - runtime helper 驕ｸ謚槭Ν繝ｼ繝励・譁・ｭ怜・驟榊・繝ｪ繝・Λ繝ｫ繧・`runtime_helpers` 螳壽焚蜿ら・縺ｫ鄂ｮ謠帙�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build` -> success

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺE蜑埼�ｲ: cliarg 縺ｮ C 譁・ｭ怜・蠅・阜繧・MemPtr<u8> 蛹・

- 逶ｮ逧・
  - `stdlib/std/env/cliarg.nepl` 縺ｮ蜈ｬ髢矩擇縺ｫ谿九▲縺ｦ縺・◆逕・`i32` 繝昴う繝ｳ繧ｿ蠅・阜繧呈ｸ帙ｉ縺励�～core/mem` 縺ｮ `MemPtr<T>` / `RegionToken<T>` 繝｢繝・Ν縺ｸ蟇・○繧九�・
  - 迚ｹ縺ｫ `cstr_len` / `cstr_to_str` 繧貞梛莉倥″繝昴う繝ｳ繧ｿ縺ｧ蜿励￠繧句ｽ｢縺ｫ螟画峩縺励�∬ｪ､縺｣縺・raw 蜻ｼ縺ｳ蜃ｺ縺励ｒ蝙九お繝ｩ繝ｼ縺ｧ豁｢繧√ｋ縲・
- 蜴溷屏:
  - `cliarg` 縺ｯ蜀・Κ繝ｻ蜈ｬ髢九→繧ゅ↓ `i32` 繧｢繝峨Ξ繧ｹ繧堤峩謗･蜿励￠貂｡縺励※縺翫ｊ縲～kpread/kpwrite` 蛛ｴ縺ｧ騾ｲ繧√※縺・◆蝙句ｮ牙・繝｢繝・Ν縺ｨ荳肴紛蜷医□縺｣縺溘�・
  - `cstr_len 0` 繧・`cstr_to_str 0` 縺ｮ繧医≧縺ｪ隱､逕ｨ縺・API 蠖｢迥ｶ荳雁庄閭ｽ縺ｧ縲√さ繝ｳ繝代う繝ｩ縺悟燕谿ｵ縺ｧ豁｢繧√ｉ繧後↑縺九▲縺溘�・
- 螟画峩:
  - `stdlib/std/env/cliarg.nepl`
    - `cstr_len` 繧・`<(MemPtr<u8>)*>i32>` 縺ｫ螟画峩縲・
    - `cstr_to_str` 繧・`<(MemPtr<u8>)*>str>` 縺ｫ螟画峩縲・
    - `cli_alloc_u8_region` / `cli_free_region` / `cli_i32_ptr` / `cli_u8_ptr` 繧定ｿｽ蜉�縺励�∽ｸ�譎る�伜沺遒ｺ菫昴ｒ `RegionToken` 繝吶・繧ｹ縺ｸ遘ｻ陦後�・
    - LLVM 蛛ｴ `__cli_copy_to_cstr` / `__cli_read_cmdline` 繧・`MemPtr<u8>` 繝吶・繧ｹ縺ｸ螟画峩縲・
    - `cliarg_count` / `cliarg_get` 縺ｮ繝｡繧ｿ諠・�ｱ遒ｺ菫昴→ `argv` 繝舌ャ繝輔ぃ遒ｺ菫昴ｒ `RegionToken<u8>` 繝吶・繧ｹ縺ｸ螟画峩縲・
  - `stdlib/tests/cliarg.n.md`
    - `cstr_len 0` / `cstr_to_str 0` 縺・`D3006` 縺ｧ螟ｱ謨励☆繧・compile_fail 蝗槫ｸｰ繧定ｿｽ蜉�縲・
- 騾比ｸｭ蛻､譁ｭ:
  - `stdlib/std/stdio.nepl` 繧ょ酔譎ゅ↓ `RegionToken` 蛹悶ｒ隧ｦ縺励◆縺後�～read_line` 縺ｮ rewrite 縺ｧ讒区枚荳肴紛蜷医ｒ蜈･繧後�｝arser overflow 繧定ｪ倡匱縺励◆縲・
  - 縺薙％縺ｯ髢薙↓蜷医ｏ縺帙〒謚ｼ縺怜・繧峨★縲～stdio` 縺ｯ逶ｴ蜑阪・豁｣蟶ｸ迥ｶ諷九∈謌ｻ縺励�∽ｻ雁屓縺ｮ繧ｳ繝溘ャ繝亥ｯｾ雎｡縺九ｉ螟悶＠縺溘�・
- 讀懆ｨｼ:
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... import-only-cliarg ... EOF` -> pass
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... cliarg-count ... argv=[\"--flag\",\"value\"] ... EOF` -> pass (`stdout: "3"`)
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... cliarg-basic ... EOF` -> pass
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... cliarg-compile-fail-cstr ... EOF` -> pass (`D3006`)
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... stdout-concat ... EOF` -> pass

# 2026-03-06 菴懈･ｭ繝｡繝｢ (繝輔ぉ繝ｼ繧ｺE蜑埼�ｲ: fs 縺ｮ荳�譎る�伜沺繧・RegionToken<u8> 蛹・

- 逶ｮ逧・
  - `stdlib/std/fs.nepl` 縺ｮ蜀・Κ荳�譎ゅヰ繝・ヵ繧｡遒ｺ菫昴ｒ `RegionToken<u8>` / `MemPtr<T>` 繝吶・繧ｹ縺ｸ遘ｻ縺励�～i32` 逕溘・繧､繝ｳ繧ｿ縺ｮ蜿励￠貂｡縺励ｒ syscall 蠅・阜縺ｸ髢峨§霎ｼ繧√ｋ縲・
- 蜴溷屏:
  - `fs_open_read` / `fs_read_fd_bytes` / `fs_bytes_to_string` 縺ｯ遒ｺ菫昴＠縺滉ｸ�譎る�伜沺繧偵☆縺ｹ縺ｦ `i32` 縺ｧ謇ｱ縺｣縺ｦ縺翫ｊ縲～cliarg` 縺ｨ蜷後§縺丞梛螳牙・繝｢繝・Ν縺九ｉ螟悶ｌ縺ｦ縺・◆縲・
  - 迚ｹ縺ｫ iovec / nread / 譁・ｭ怜・邨・∩遶九※逕ｨ鬆伜沺縺悟梛諠・�ｱ繧貞､ｱ縺｣縺溘∪縺ｾ豬√ｌ縺ｦ縺・◆縺溘ａ縲∬ｪ､逕ｨ繧・API 蠖｢迥ｶ縺ｧ髦ｲ縺偵↑縺九▲縺溘�・
- 螟画峩:
  - `stdlib/std/fs.nepl`
    - `fs_alloc` / `fs_free` 繧貞ｻ・ｭ｢縺励�～fs_alloc_u8_region` / `fs_free_region` / `fs_i32_ptr` 繧定ｿｽ蜉�縲・
    - LLVM 蛛ｴ `__fs_copy_to_cstr` 繧・`Result<MemPtr<u8>,i32>` 縺ｸ螟画峩縺励�∬ｧ｣謾ｾ繧・`dealloc_ptr<u8>` 縺ｫ邨ｱ荳�縲・
    - `fs_open_read` 縺ｮ fd_out 荳�譎る�伜沺繧・`RegionToken<u8>` 蛹悶�・
    - `fs_read_fd_bytes` 縺ｮ tmp/iov/nread 荳�譎る�伜沺繧・`RegionToken<u8>` 蛹悶＠縲～load/store` 縺ｯ `MemPtr` 繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨ｒ邨檎罰縺吶ｋ蠖｢縺ｸ螟画峩縲・
    - `fs_bytes_to_string` 縺ｮ蜃ｺ蜉帙ヰ繝・ヵ繧｡讒狗ｯ峨ｒ `RegionToken<u8>` 縺ｨ `MemPtr<u8>` 縺ｧ陦後≧蠖｢縺ｸ螟画峩縲・
- 險ｭ險亥愛譁ｭ:
  - `wasi_path_open` / `wasi_fd_read` 閾ｪ菴薙・繝帙せ繝・ABI 蠅・阜縺ｪ縺ｮ縺ｧ raw `i32` 繧堤ｶｭ謖√＠縺溘�・
  - 蝙句ｮ牙・蛹悶・蟇ｾ雎｡縺ｯ stdlib 蜈ｬ髢矩擇縺ｨ stdlib 蜀・・騾壼ｸｸ繝ｭ繧ｸ繝・け縺ｧ縺ゅｊ縲、BI 逶ｴ蜑阪・縺ｿ `mem_ptr_addr` 縺ｧ raw 蛹悶☆繧九�・
- 讀懆ｨｼ:
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... import-only-fs ... EOF` -> pass
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... fs-missing-file ... EOF` -> pass

# 2026-03-06 菴懈･ｭ繝｡繝｢ (蝙句ｮ牙・蝗槫ｸｰ縺ｮ霑ｽ蜉�: MemPtr 縺ｨ RegionToken 縺ｮ蜿悶ｊ驕輔∴繧・D3006 縺ｧ蝗ｺ螳・

- 逶ｮ逧・
  - `core/mem` 縺ｮ蝙句ｮ牙・繝｢繝・Ν繧偵ユ繧ｹ繝医〒蝗ｺ螳壹＠縲～MemPtr<u8>` / `MemPtr<i32>` / `RegionToken<T>` 縺ｮ蜿悶ｊ驕輔∴繧貞燕谿ｵ縺ｧ豁｢繧√ｋ縲・
- 螟画峩:
  - `tests/memory_safety.n.md`
    - `load_i32` 縺ｫ `MemPtr<u8>` 繧呈ｸ｡縺・compile_fail 繧定ｿｽ蜉�縲・
    - `store_u8` 縺ｫ `MemPtr<i32>` 繧呈ｸ｡縺・compile_fail 繧定ｿｽ蜉�縲・
    - `dealloc_region` 縺ｫ `MemPtr<u8>` 繧呈ｸ｡縺・compile_fail 繧定ｿｽ蜉�縲・
- 讀懆ｨｼ:
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... mem-basic ... EOF` -> pass
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... mem-load-i32-type-fail ... EOF` -> pass (`D3006`)
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... mem-store-u8-type-fail ... EOF` -> pass (`D3006` 縺悟・鬆ｭ)
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... mem-dealloc-region-type-fail ... EOF` -> pass (`D3006` 縺悟・鬆ｭ)
- 陬懆ｶｳ:
  - `nodesrc/tests.js -i tests/memory_safety.n.md ...` 縺ｯ縺薙・迺ｰ蠅・〒縺ｯ timeout 30s 縺ｫ蛻ｰ驕斐＠縺溘◆繧√�∝�句挨 focused 螳溯｡後〒遒ｺ隱阪＠縺溘�・

# 2026-03-06 菴懈･ｭ繝｡繝｢ (core/mem 縺ｮ莠呈鋤繧ｨ繧､繝ｪ繧｢繧ｹ謨ｴ逅・

- 逶ｮ逧・
  - `MemPtr` 螳牙・繧ｪ繝ｼ繝舌・繝ｭ繝ｼ繝峨∈蜿取據縺輔○縲～load_i32_ptr` / `store_i32_ptr` 縺ｮ繧医≧縺ｪ莠呈鋤蜷阪ｒ谿九＆縺ｪ縺・�・
- 螟画峩:
  - `stdlib/core/mem.nepl`
    - `load_i32_ptr`
    - `store_i32_ptr`
    - `load_u8_ptr`
    - `store_u8_ptr`
    繧貞炎髯､縲・
  - `tests/memory_safety.n.md`
    - 譌｢蟄倥ユ繧ｹ繝医ｒ `load_i32` / `store_i32` 縺ｮ逶ｴ謗･蛻ｩ逕ｨ縺ｸ譖ｴ譁ｰ縲・
- 讀懆ｨｼ:
  - `rg -n "load_i32_ptr|store_i32_ptr|load_u8_ptr|store_u8_ptr" stdlib tests tutorials examples` -> 隧ｲ蠖薙↑縺・
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... mem-basic-direct-overload ... EOF` -> pass
  - `timeout 20s node nodesrc/run_test.js <<'EOF' ... mem-invalid-store-direct-overload ... EOF` -> pass

# 2026-03-08 菴懈･ｭ繝｡繝｢ (謠先｡・ stdlib 縺ｮ蠕梧婿莠呈鋤縺ｪ縺怜・險ｭ險・

- 逶ｮ逧・
  - stdlib 繧偵ず繧ｧ繝阪Μ繧ｯ繧ｹ/trait 荳ｭ蠢・〒菴懊ｊ逶ｴ縺吶◆繧√・縲∝ｾ梧婿莠呈鋤縺ｪ縺励・遐ｴ螢顔噪謾ｹ濶ｯ譯医ｒ謨ｴ逅・☆繧九�・
  - 譌｢蟄倥・ `_raw` 萓晏ｭ倥�∝多蜷肴昭繧後�》arget 萓晏ｭ俶ｷｷ蝨ｨ繧定ｧ｣豸医☆繧九◆繧√・險ｭ險郁ｻｸ繧呈・遒ｺ蛹悶☆繧九�・
- 螟画峩:
  - `doc/stdlib_breaking_reboot.md`
    - 逶ｮ逧・髱樒岼讓・險ｭ險亥次蜑・ｒ螳夂ｾｩ縲・
    - 譁ｰ縺励＞ stdlib 繝代ャ繧ｱ繝ｼ繧ｸ讒区・・・core/alloc/collections/text/io/fs/runtime/prelude`・峨ｒ謠先｡医�・
    - trait 閭ｽ蜉帙Δ繝・Ν・医Γ繝｢繝ｪ閭ｽ蜉帙�！/O 閭ｽ蜉帛性繧�・峨→繧ｸ繧ｧ繝阪Μ繧ｯ繧ｹ險ｭ險医ｒ謠先｡医�・
    - 蜻ｽ蜷崎ｦ丞援縺ｮ遐ｴ螢顔噪螟画峩・・_raw/_safe` 蟒・ｭ｢縲～into_xxx/parse_xxx` 邨ｱ荳�・峨ｒ謠先｡医�・
    - runtime adapter 蛻・屬縲∫ｧｻ陦後ヵ繧ｧ繝ｼ繧ｺ縲√ユ繧ｹ繝域姶逡･縲∵悄蠕・柑譫懊ｒ險倩ｿｰ縲・
- plan.md縺ｨ縺ｮ蟾ｮ逡ｰ:
  - `plan.md` 縺ｯ險�隱樔ｻ墓ｧ倥・譬ｸ・亥燕鄂ｮ險俶ｳ輔・蠑乗欠蜷代・繧ｪ繝輔し繧､繝峨Ν繝ｼ繝ｫ・峨ｒ螳夂ｾｩ縺励※縺・ｋ縲・
  - 莉雁屓縺ｯ險�隱樊ｧ区枚繧貞､画峩縺帙★縲《tdlib 縺ｮ雋ｬ蜍吝・髮｢縺ｨ trait 蠅・阜險ｭ險医↓髯仙ｮ壹＠縺滓署譯医〒縺ゅｊ縲～plan.md` 縺ｨ遏帷崟縺励↑縺・�・
- 邨先棡:
  - 繝輔ぉ繝ｼ繧ｺD/E・・ompiler `_raw` 萓晏ｭ俶彫蜴ｻ縲《tdlib 遘ｻ陦鯉ｼ峨ｒ騾ｲ繧√ｋ髫帙・螳溯｣・渕貅悶→縺励※蜿ら・蜿ｯ閭ｽ縺ｪ譁・嶌繧定ｿｽ蜉�縺励◆縲・

- 讀懆ｨｼ:
  - `trunk build`
    - 邨先棡: 迺ｰ蠅・↓ `trunk` 繧ｳ繝槭Φ繝峨′蟄伜惠縺帙★螳溯｡御ｸ榊庄・・command not found`・峨�・
  - `node nodesrc/tests.js -i tests/stdlib.n.md --no-tree -o /tmp/tests-proposal.json -j 15`
    - 邨先棡: `nepl-web compiler artifacts were not found` 縺ｫ繧医ｊ `229/229 errored`縲・
    - 蜃ｺ蜉・JSON `/tmp/tests-proposal.json` 繧堤｢ｺ隱阪＠縲∬ｦ∝屏縺後ン繝ｫ繝画・譫懃黄荳崎ｶｳ縺ｧ縺ゅｋ縺薙→繧堤｢ｺ隱阪�・

# 2026-03-08 菴懈･ｭ繝｡繝｢ (謾ｹ蝟・ stdlib 蜀崎ｨｭ險域｡医・ NEPLg2 蜩ｲ蟄ｦ謨ｴ蜷・

- 逶ｮ逧・
  - 蜑榊屓霑ｽ蜉�縺励◆ `doc/stdlib_breaking_reboot.md` 縺後�～plan.md`縲～introduce.n.md`縲～tutorials` 縺ｧ遉ｺ縺輔ｌ繧・NEPLg2 縺ｮ蜩ｲ蟄ｦ・亥ｼ乗欠蜷代・蜑咲ｽｮ險俶ｳ輔・繧ｪ繝輔し繧､繝峨Ν繝ｼ繝ｫ繝ｻ繝代う繝怜粋謌撰ｼ峨→荳�閾ｴ縺励※縺・ｋ縺九ｒ蜀咲せ讀懊＠縲∵隼蝟・☆繧九�・
- 蜴溷屏:
  - 蜑榊屓譯医・ trait/generics 縺ｨ螳牙・諤ｧ譁ｹ驥昴・遉ｺ縺帙※縺・◆縺後�¨EPLg2 縺ｮ陦ｨ迴ｾ蜩ｲ蟄ｦ・亥�､蜷域・蜆ｪ蜈医�√ヱ繧､繝励〒霑ｽ縺医ｋ蠑墓焚鬆・�‘ffect 譏守､ｺ・峨→縺ｮ謗･邯壹′蠑ｱ縺上�∝ｮ溯｣・愛譁ｭ譎ゅ↓隗｣驥医′縺ｶ繧後ｋ菴吝慍縺後≠縺｣縺溘�・
- 螟画峩:
  - `doc/stdlib_breaking_reboot.md`
    - 縲君EPLg2 蜩ｲ蟄ｦ縺ｨ縺ｮ謨ｴ蜷郁ｦ∽ｻｶ縲咲ｫ�繧定ｿｽ蜉�縺励�∝ｼ乗欠蜷代・蜑咲ｽｮ險俶ｳ・繝代う繝励・effect繝ｻ蝙矩ｧ・虚縺ｮ謨ｴ蜷亥渕貅悶ｒ譏取枚蛹悶�・
    - API險ｭ險亥次蜑・ｒ縲悟粋謌舌＠繧・☆縺・ｼ墓焚鬆・�阪�形Result/Option` 縺ｧ螟ｱ謨励ｒ陦ｨ迴ｾ縲阪�荊arget萓晏ｭ倥ｒadapter縺ｸ髫秘屬縲阪・隕ｳ轤ｹ縺ｧ蜀肴紛逅・�・
    - 繧ｳ繝ｳ繝・リ繝ｻ蜻ｽ蜷肴婿驥昴・遘ｻ陦後ヵ繧ｧ繝ｼ繧ｺ繝ｻ繝・せ繝域姶逡･繧偵�》utorials縺ｮ螳溯｣・せ繧ｿ繧､繝ｫ縺ｨ郢九′繧句ｽ｢縺ｫ隱ｿ謨ｴ縲・
- plan.md縺ｨ縺ｮ蟾ｮ逡ｰ:
  - 險�隱樔ｻ墓ｧ倥・螟画峩縺励※縺・↑縺・�・
  - stdlib蜀崎ｨｭ險域｡医・隧穂ｾ｡霆ｸ繧偵�～plan.md` 縺ｨ tutorials 縺ｮ險倩ｿｰ縺ｫ豐ｿ縺・ｈ縺・ｼｷ蛹悶＠縺溘�・
- 邨先棡:
  - 遐ｴ螢顔噪謾ｹ濶ｯ譯医ｒ縺昴・縺ｾ縺ｾ螳溯｣・ｨ育判縺ｸ關ｽ縺ｨ縺苓ｾｼ繧�髫帙↓縲¨EPLg2 縺ｮ險ｭ險域�晄Φ縺ｨ荵夜屬縺励↓縺上＞譁・嶌縺ｸ譖ｴ譁ｰ縺ｧ縺阪◆縲・
- 讀懆ｨｼ:
  - `trunk build`
    - 邨先棡: 迺ｰ蠅・↓ `trunk` 繧ｳ繝槭Φ繝峨′蟄伜惠縺帙★螳溯｡御ｸ榊庄・・command not found`・峨�・
  - `node nodesrc/tests.js -i tests/stdlib.n.md --no-tree -o /tmp/tests-stdlib-philosophy.json -j 15`
    - 邨先棡: `nepl-web compiler artifacts were not found` 縺ｫ繧医ｊ `229/229 errored`縲・
    - 蜃ｺ蜉・JSON `/tmp/tests-stdlib-philosophy.json` 繧堤｢ｺ隱阪＠縲∵・譫懃黄荳崎ｶｳ縺悟､ｱ謨苓ｦ∝屏縺ｧ縺ゅｋ縺薙→繧堤｢ｺ隱阪�・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (stdlib reboot 髢句ｧ句燕縺ｮ譛ｪ遒ｺ螳壼ｷｮ蛻・紛逅・

- 逶ｮ逧・
  - `todo.md` 縺ｮ譛ｬ譬ｼ螳溯｣・∈蜈･繧句燕縺ｫ縲∫樟蝨ｨ縺ｮ譛ｪ遒ｺ螳壼ｷｮ蛻・′菴輔ｒ逶ｴ縺励※縺・ｋ縺ｮ縺九�√←縺薙∪縺ｧ螳牙ｮ壹＠縺ｦ縺・ｋ縺ｮ縺九�∽ｽ輔′蛻･莉ｶ繝悶Ο繝・き繝ｼ縺ｪ縺ｮ縺九ｒ譏守｢ｺ縺ｫ縺吶ｋ縲・
  - `vec` 縺ｮ蝙句ｮ牙・蛹門ｷｮ蛻・ｒ縲後◎縺ｮ縺ｾ縺ｾ reboot 縺ｫ謖√■霎ｼ繧√ｋ迥ｶ諷九�阪∪縺ｧ謨ｴ逅・☆繧九�・
- 蟇ｾ雎｡蟾ｮ蛻・
  - `stdlib/alloc/collections/vec.nepl`
  - `stdlib/alloc/collections/vec/sort.nepl`
  - `stdlib/alloc/string.nepl`
  - `stdlib/nm/parser.nepl`
  - `stdlib/nm/html_gen.nepl`
  - `examples/bf.nepl` 縺ｯ莉雁屓縺ｮ謨ｴ逅・ｯｾ雎｡螟悶・譌｢蟄伜ｷｮ蛻・→縺励※隗ｦ繧後※縺・↑縺・�・
- 螟画峩縺ｮ諢丞袖:
  - `Vec<.T>.data` 繧・`i32` 縺九ｉ `MemPtr<.T>` 縺ｫ螟画峩縺励�～alloc/collections` 繧貞梛莉倥″繝｡繝｢繝ｪ API 縺ｫ蟇・○縺ｦ縺・ｋ縲・
  - 縺昴ｌ縺ｫ莨ｴ縺・�～string` 縺ｨ `nm` 縺ｧ `get v "data"` 繧堤函 `i32` 縺ｨ縺ｿ縺ｪ縺励※縺・◆邂・園繧・`mem_ptr_addr get ... "data"` 縺ｫ霑ｽ蠕薙＆縺帙※縺・ｋ縲・
  - `vec/sort` 繧ょ酔讒倥↓縲～Vec` 縺ｮ蜀・Κ陦ｨ迴ｾ螟画峩縺ｸ霑ｽ蠕薙＠縺ｦ縺・ｋ縲・
- 譬ｹ譛ｬ蜴溷屏:
  - `core/mem` 縺ｮ蝙句ｮ牙・蛹悶ｒ騾ｲ繧√◆邨先棡縲～alloc/collections` 縺ｮ荳ｭ譬ｸ縺ｧ縺ゅｋ `Vec` 縺・raw `i32` 繧貞・髢九＠縺ｦ縺・ｋ縺ｨ荳区ｵ∝・菴薙・蝙句ｮ牙・蛹悶′騾ｲ縺ｾ縺ｪ縺・�・
  - 縺昴・縺溘ａ `Vec` 繧貞・縺ｫ `MemPtr<.T>` 蛹悶＠縲√◎縺ｮ螟画峩縺ｮ蠖ｱ髻ｿ蜈医ｒ霑ｽ蠕薙＆縺帙ｋ蠢・ｦ√′縺ゅ▲縺溘�・
- 蛻・ｊ蛻・￠邨先棡:
  - `string` 縺ｮ譛�蟆・compile 縺ｯ騾夐℃縺励◆縲・
    - `sb_build` 蜻ｨ霎ｺ縺ｮ `parts_vec.data` 蜿ら・螟画峩縺ｯ螯･蠖薙�・
  - `vec` 縺ｮ譛�蟆・compile 繧ゅ�～vec_get` 繧堤畑縺・◆繧ｱ繝ｼ繧ｹ縺ｧ縺ｯ騾夐℃縺励◆縲・
    - `get v 1` 縺悟､ｱ謨励☆繧九・縺ｯ field access 縺ｮ `get` 縺ｨ陦晉ｪ√＠縺ｦ縺・ｋ縺溘ａ縺ｧ縲∽ｻ雁屓縺ｮ `MemPtr` 蛹冶・菴薙・蝠城｡後〒縺ｯ縺ｪ縺・�・
  - `nm/parser.nepl` 縺ｯ import 縺吶ｋ縺�縺代〒 parser 縺ｮ stack overflow 縺檎匱逕溘＠縺溘�・
  - `nm/html_gen.nepl` 縺ｯ import 縺吶ｋ縺�縺代〒 wasm validation error 縺檎匱逕溘＠縺溘�・
- 驥崎ｦ√↑蛻､譁ｭ:
  - `nm/parser.nepl` / `nm/html_gen.nepl` 縺ｮ import-only failure 縺ｯ縲∽ｻ雁屓縺ｮ `mem_ptr_addr` 霑ｽ蠕灘､画峩縺ｨ縺ｯ迢ｬ遶九・譌｢蟄倥ヶ繝ｭ繝・き繝ｼ縺ｨ縺励※謇ｱ縺・�・
  - 縺励◆縺後▲縺ｦ縲∫樟蝨ｨ縺ｮ譛ｪ遒ｺ螳壼ｷｮ蛻・・縺・■
    - `vec.nepl`
    - `vec/sort.nepl`
    - `string.nepl`
    縺ｯ reboot 縺ｫ蜷代￠縺滓怏蜉ｹ蟾ｮ蛻・〒縺ゅｋ縲・
  - 荳�譁ｹ縺ｧ `nm` 蛛ｴ縺ｯ縲∽ｻ雁屓縺ｮ霑ｽ蠕灘､画峩閾ｪ菴薙・螯･蠖捺�ｧ縺ｯ鬮倥＞縺後�∫樟譎らせ縺ｧ import-only compile 縺悟､ｱ謨励☆繧九◆繧√�∝�句挨縺ｫ螳牙ｮ壽�ｧ繧定ｨｼ譏弱＠縺溘→縺ｯ縺ｾ縺�險�縺医↑縺・�・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`
    - 邨先棡: 謌仙粥縲・
  - direct compile (`alloc/string` 譛�蟆上こ繝ｼ繧ｹ)
    - 邨先棡: 謌仙粥縲・
  - direct compile (`alloc/collections/vec` 譛�蟆上こ繝ｼ繧ｹ縲～vec_get` 菴ｿ逕ｨ)
    - 邨先棡: 謌仙粥縲・
  - direct compile (`nm/parser` import-only)
    - 邨先棡: parser stack overflow縲・
  - direct compile (`nm/html_gen` import-only)
    - 邨先棡: wasm validation error縲・
- 迴ｾ譎らせ縺ｮ邨占ｫ・
  - `todo.md` 縺ｮ譛ｬ譬ｼ螳溯｣・∈蜈･繧九◆繧√・貅門ｙ縺ｨ縺励※縲∵悴遒ｺ螳壼ｷｮ蛻・・諢丞袖縺ｨ繝悶Ο繝・き繝ｼ縺ｯ謨ｴ逅・〒縺阪◆縲・
  - 谺｡縺ｮ螳牙・縺ｪ逹�謇狗せ縺ｯ `todo.md` 蜈磯�ｭ縺ｮ `std/test` 謾ｹ蝟・ち繧ｹ繧ｯ縺ｧ縺ゅｋ縲・
  - `vec` 邉ｻ蟾ｮ蛻・・ reboot 險育判縺ｫ蜷ｸ蜿弱☆繧句燕謠舌〒菫晄戟縺励�～nm` 蛛ｴ縺ｮ import-only 螟ｱ謨励・蛻･莉ｶ繝悶Ο繝・き繝ｼ縺ｨ縺励※邂｡逅・☆繧九�・
  - 縺薙・譎らせ縺ｧ縺ｯ `nm/parser.nepl` / `nm/html_gen.nepl` 縺ｮ霑ｽ蠕灘ｷｮ蛻・・ commit 蟇ｾ雎｡縺九ｉ螟悶＠縲《tdlib reboot 蠕後↓謾ｹ繧√※蟇ｾ蜃ｦ縺吶ｋ縲・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (tests/compiler 縺ｨ tests/stdlib 縺ｮ蜀咲ｷｨ)

- 逶ｮ逧・
  - stdlib reboot 髢句ｧ句燕縺ｫ縲√ユ繧ｹ繝亥､ｱ謨励・蜴溷屏繧偵�慶ompiler 譛ｬ菴薙・隱､繧翫�阪�茎tdlib 螳溯｣・・隱､繧翫�阪�後ユ繧ｹ繝育ｧｻ陦後Α繧ｹ縲阪・ 3 縺､縺ｸ蛻・ｊ蛻・￠繧・☆縺上☆繧九�・
  - `tests/` 逶ｴ荳九↓豺ｷ蝨ｨ縺励※縺・◆繧ｱ繝ｼ繧ｹ繧・`tests/compiler/*` 縺ｨ `tests/stdlib/*` 縺ｸ蛻・屬縺励�∽ｻ･蠕後・蝗槫ｸｰ遒ｺ隱阪・邊貞ｺｦ繧呈純縺医ｋ縲・
- 螟画峩:
  - compiler 譛ｬ菴薙・遒ｺ隱阪ｒ荳ｻ逶ｮ逧・→縺吶ｋ `.n.md` 縺ｨ tree suite 繧・`tests/compiler/*` 縺ｸ遘ｻ蜍輔＠縺溘�・
  - stdlib API繝ｻ繧｢繝ｫ繧ｴ繝ｪ繧ｺ繝�繝ｻtarget facade繝ｻ蝗槫ｸｰ遒ｺ隱阪ｒ荳ｻ逶ｮ逧・→縺吶ｋ `.n.md` 繧・`tests/stdlib/*` 縺ｸ遘ｻ蜍輔＠縺溘�・
  - `nodesrc/tests.js`
    - tree suite 縺ｮ隱ｭ縺ｿ霎ｼ縺ｿ蜈医ｒ `tests/compiler/tree/run` 縺ｸ譖ｴ譁ｰ縺励◆縲・
    - tree suite 邨先棡縺ｮ `id` / `file` 繧・`tests/compiler/tree/*` 縺ｸ譖ｴ譁ｰ縺励◆縲・
  - `tests/compiler/tree/_shared.js`
    - `nodesrc/*` 縺ｸ縺ｮ逶ｸ蟇ｾ import 繧偵�∵眠縺励＞驟咲ｽｮ縺ｫ蜷医ｏ縺帙※ 1 谿ｵ豺ｱ縺丈ｿｮ豁｣縺励◆縲・
  - `nodesrc/analyze_source.js`
    - 菴ｿ逕ｨ萓九さ繝｡繝ｳ繝医・繝代せ繧・`tests/compiler/functions.n.md` 縺ｸ譖ｴ譁ｰ縺励◆縲・
- 譬ｹ譛ｬ蜴溷屏:
  - 譌｢蟄倥・ `tests/` 縺ｯ compiler 譛ｬ菴薙ユ繧ｹ繝医→ stdlib 繝・せ繝医′蜷悟ｱ・＠縺ｦ縺翫ｊ縲《tdlib reboot 荳ｭ縺ｫ螟ｱ謨励・蜴溷屏繧呈ｭ｣縺励￥蛻・ｊ蛻・￠縺ｫ縺上°縺｣縺溘�・
  - tree suite 繧・`tests/tree/*` 繧貞燕謠舌↓逶ｴ蜿ら・縺励※縺・◆縺溘ａ縲∝腰邏斐↑繝輔ぃ繧､繝ｫ遘ｻ蜍輔□縺代〒縺ｯ螳溯｡檎ｵ瑚ｷｯ縺悟｣翫ｌ縺溘�・
- 螳溯｣・ｸ翫・豕ｨ諢・
  - `nodesrc/tests.js` 縺ｯ譌｢螳壹〒 stdlib doctest 繧ゆｸ�邱偵↓襍ｰ譟ｻ縺吶ｋ縺溘ａ縲’ocused test 縺ｧ縺ｯ `--no-stdlib` 繧呈・遉ｺ縺励↑縺・→縲檎ｧｻ蜍慕｢ｺ隱阪�阪・縺､繧ゅｊ縺・stdlib 蜈ｨ菴灘ｮ溯｡後↓縺ｪ繧九�・
  - 莉雁屓縺ｮ focused 讀懆ｨｼ縺ｯ縲∝・邱ｨ縺昴・繧ゅ・縺ｮ螳牙・諤ｧ遒ｺ隱阪↓髯仙ｮ壹☆繧九◆繧・`--no-stdlib --no-tree` 繧堤畑縺・◆縲・
- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/compiler/block_semicolon_return.n.md -i tests/compiler/plan.n.md -i tests/compiler/overload_nested_generic_push.n.md --no-stdlib --no-tree -o /tmp/tests-compiler-reorg-focused.json -j 15`
    - 邨先棡: `49/49 pass`
  - `node nodesrc/tests.js -i /tmp/std-test-collect-success-only.n.md --no-stdlib --no-tree -o /tmp/std-test-collect-success-only.json -j 15`
    - 邨先棡: `1/1 pass`
  - `node nodesrc/tests.js -i /tmp/std-test-collect-fail-only.n.md --no-stdlib --no-tree -o /tmp/std-test-collect-fail-only.json -j 15`
    - 邨先棡: `1/1 pass`
- 邨占ｫ・
  - `tests/compiler/*` 縺ｨ `tests/stdlib/*` 縺ｮ蛻・屬縲√♀繧医・縺昴ｌ縺ｫ莨ｴ縺・`nodesrc` / tree suite 縺ｮ霑ｽ蠕薙・謌千ｫ九＠縺溘�・
  - `todo.md` 蜈磯�ｭ縺ｮ蜀咲ｷｨ繧ｿ繧ｹ繧ｯ縺ｯ螳御ｺ・→縺励※蜑企勁縺励�∽ｻ･蠕後・ reboot 譛ｬ豬√・ `diag` / `Outcome` / trait 閭ｽ蜉帙Δ繝・Ν縺ｮ螳溯｣・∈騾ｲ繧√ｋ縲・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (`std/test` 繧ｳ繝｡繝ｳ繝域紛逅・→ collect API 縺ｮ菴ｿ縺・婿蝗ｺ螳・

- 逶ｮ逧・
  - `stdlib/std/test.nepl` 縺ｮ繧ｳ繝｡繝ｳ繝医ｒ `doc/stdlib_doc_comment_policy.md` 縺ｫ豐ｿ縺｣縺ｦ謨ｴ逅・＠縲∝・驛ｨ helper 縺ｫ boilerplate doctest 縺御ｸｦ縺ｶ迥ｶ諷九ｒ隗｣豸医☆繧九�・
  - 蛻ｩ逕ｨ閠・′逶ｴ謗･菴ｿ縺・・髢・API 縺�縺代↓縲∫畑騾斐′蛻・°繧・doctest 繧呈ｮ九☆縲・
- 螟画峩:
  - `stdlib/std/test.nepl`
    - 繝｢繧ｸ繝･繝ｼ繝ｫ蜈磯�ｭ繧ｳ繝｡繝ｳ繝医ｒ縲∝腰逋ｺ assert 縺ｨ collectable 縺ｪ `check_*` / `finish_checks` 縺ｮ莠檎ｳｻ邨ｱ繧呈戟縺､縺薙→縺悟・縺九ｋ蜀・ｮｹ縺ｸ譖ｴ譁ｰ縲・
    - `test_str_eq_loop`縲～test_print_fail`縲～test_checked`縲～test_fail`縲～trap` 縺ｪ縺ｩ蜀・Κ helper 縺ｮ boilerplate doctest 繧貞炎髯､縲・
    - `assert` / `assert_eq_i32` / `assert_ne` / `assert_str_eq` / `assert_ok_i32` / `assert_err_i32` 縺ｮ doctest 繧偵�∝ｮ滄圀縺ｮ逕ｨ騾斐′蛻・°繧倶ｾ九∈蟾ｮ縺玲崛縺医�・
    - 險育ｮ鈴㍼陦ｨ險倥ｒ `[譎る俣/縺倥°繧転` / `[遨ｺ髢・縺上≧縺九ｓ]` 縺ｮ蠖｢縺ｫ謠・∴縺溘�・
- 蛻､譁ｭ:
  - `std/test` 縺ｮ螳溯｣・・菴薙・ `67e8156` 縺ｧ蜊∝・縺ｫ謠・▲縺ｦ縺・◆縺溘ａ縲∽ｻ雁屓縺ｯ API 繧貞｢励ｄ縺輔★縲∝茜逕ｨ閠・髄縺題ｪｬ譏弱・雉ｪ繧貞・縺ｫ荳翫￡縺溘�・
  - 螳溯｣・､懆ｨｼ縺ｯ `tests/stdlib/std_test_collect.n.md` 縺ｫ谿九＠縲～.nepl` 蛛ｴ doctest 縺ｯ菴ｿ縺・婿遒ｺ隱阪∈蟇・○縺溘�・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (`std/test` 繧ｳ繝｡繝ｳ繝域紛逅・・讀懆ｨｼ螳御ｺ・

- 讀懆ｨｼ:
  - `node nodesrc/tests.js -i tests/stdlib/std_test_collect.n.md --no-stdlib --no-tree -o /tmp/tests-stdlib-std-test-collect-nostdlib.json -j 15`
    - 邨先棡: `2/2 pass`
  - `node nodesrc/tests.js -i /tmp/std_test_assert_doctest_smoke.n.md --no-stdlib --no-tree -o /tmp/std_test_assert_doctest_smoke.json -j 15`
    - 邨先棡: `1/1 pass`
- 蛻､譁ｭ:
  - `stdlib/std/test.nepl` 縺ｮ蜈ｬ髢・`assert_*` 萓九・ `#entry main` 縺ｨ `#target std` 繧貞燕謠舌↓縺吶ｋ縺ｨ縲√◎縺ｮ縺ｾ縺ｾ螳溯｡後〒縺阪ｋ縺薙→繧堤｢ｺ隱阪＠縺溘�・
  - collectable API 縺ｮ譌｢蟄伜屓蟶ｰ 2 莉ｶ繧らｶｭ謖√＆繧後※縺・ｋ縺溘ａ縲∽ｻ雁屓縺ｮ螟画峩縺ｯ繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝域紛逅・→縺励※遒ｺ螳壹＠縺ｦ繧医＞縲・
  - `nodesrc/tests.js` 縺ｯ `--no-stdlib` 繧剃ｻ倥￠縺ｪ縺・→ stdlib 蜈ｨ襍ｰ譟ｻ縺ｧ驥阪￥縺ｪ繧翫ｄ縺吶￥縲’ocused 讀懆ｨｼ縺ｧ縺ｯ `--no-stdlib` 繧剃ｽｿ縺・・縺悟ｦ･蠖薙�・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (`alloc/diag` 縺ｮ蜀崎ｨｭ險医→ focused test 縺ｮ螳牙ｮ壼喧)

- 逶ｮ逧・
  - stdlib reboot 縺ｮ譛�蛻昴・譛ｬ豬√ち繧ｹ繧ｯ縺ｨ縺励※縲～alloc/diag` 繧・`Diag` / `Diags` / `Outcome` / `StdErrorKind` 荳ｭ蠢・・繝｢繝・Ν縺ｸ遘ｻ陦後☆繧九�・
  - 譌ｧ `error.nepl` 縺ｮ雋ｬ蜍吶ｒ `diag` 蛛ｴ縺ｸ蜷ｸ蜿弱＠縲《tdlib 蜈ｨ菴薙〒蜀榊茜逕ｨ縺ｧ縺阪ｋ險ｺ譁ｭ蝓ｺ逶､繧貞・縺ｫ蝗ｺ繧√ｋ縲・
- 螟画峩:
  - `stdlib/alloc/diag/error.nepl`
    - `DiagLevel`, `StdErrorKind`, `DiagKind`, `Diag`, `Diags`, `Outcome` 繧貞ｮ夂ｾｩ縺励◆縲・
    - `diag_new`, `diag_log`, `diag_info`, `diag_warn`, `diag_error`, `diag_with_span`, `diag_with_source`, `diag_add_note`, `diag_add_help` 繧定ｿｽ蜉�縺励◆縲・
    - `diags_new`, `diags_one`, `diags_push`, `diags_len`, `diags_has_errors` 繧定ｿｽ蜉�縺励◆縲・
    - `outcome_ok`, `outcome_err`, `outcome_with_diags`, `result_to_outcome` 繧定ｿｽ蜉�縺励◆縲・
    - `diag_out_of_memory` 縺ｪ縺ｩ譌ｧ collections 蛛ｴ helper 縺ｯ縲∵眠縺励＞ `Diag` 繝｢繝・Ν縺ｮ阮・＞繝ｩ繝・ヱ縺ｨ縺励※谿九＠縺溘�・
  - `stdlib/alloc/diag/diag.nepl`
    - `kind_str`, `span_to_string`, `diag_to_string`, `diags_to_string` 繧呈眠 `Diag` / `Diags` 讒矩��縺ｫ蜷医ｏ縺帙※譖ｸ縺咲峩縺励◆縲・
    - `std` target 縺ｧ縺ｯ `diag_print*` / `diags_print*` 繧・renderer helper 縺ｨ縺励※谿九＠縺溘�・
  - `stdlib/tests/error.n.md`
    - `Diag` / `Diags` / `Outcome` 縺ｮ蛟､繝｢繝・Ν遒ｺ隱阪∈蜈ｨ髱｢譖ｴ譁ｰ縺励◆縲・
    - `match _:` 繧貞・謖吝梛縺ｮ螳悟・蛻玲嫌縺ｸ菫ｮ豁｣縺励◆縲・
  - `stdlib/tests/diag.n.md`
    - `diag_to_string` / `diags_to_string` 縺ｮ focused test 繧呈眠繝｢繝・Ν縺ｸ譖ｴ譁ｰ縺励◆縲・
  - `tests/stdlib/collections_diag.n.md`
    - collections 縺瑚ｿ斐☆ `Diag` 縺ｮ `StdErrorKind` 遒ｺ隱阪∈譖ｴ譁ｰ縺励◆縲・
  - `tests/compiler/sizeof.n.md`
    - `Span` / `Diag` / `Diags` / `Outcome` 縺ｮ `size_of` 繧ｱ繝ｼ繧ｹ繧呈眠繝｢繝・Ν縺ｸ譖ｴ譁ｰ縺励◆縲・
- 譬ｹ譛ｬ蜴溷屏縺ｨ菫ｮ豁｣:
  - `diag_new`, `diags_new`, `diags_one`, `checks_new` 縺ｪ縺ｩ縺・`Vec::new` / `vec_push` 繧貞・驛ｨ縺ｧ蜻ｼ繧薙〒縺・ｋ縺ｮ縺ｫ pure 縺ｮ縺ｾ縺ｾ縺�縺｣縺溘�・
    - 縺薙ｌ縺ｫ繧医ｊ `pure context cannot call impure function` 縺檎匱逕溘＠縺ｦ縺・◆縲・
    - 蠖ｱ髻ｿ縺吶ｋ helper 繧・impure 繧ｷ繧ｰ繝阪メ繝｣縺ｸ菫ｮ豁｣縺励◆縲・
  - `alloc/diag/error.nepl` 縺ｧ縺ｯ `new<str>` / `new<Diag>` 縺ｮ辟｡菫ｮ鬟ｾ蜻ｼ縺ｳ蜃ｺ縺励′縲∝捉霎ｺ import 迺ｰ蠅・↓繧医▲縺ｦ `ambiguous overload` 縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
    - 縺薙ｌ縺ｯ `new` / `push` 縺ｮ alias 鄒､縺・star import 縺ｧ豺ｷ縺悶ｋ迴ｾ陦檎腸蠅・↓萓晏ｭ倥＠縺滉ｸ榊ｮ牙ｮ壽�ｧ縺�縺｣縺溘�・
    - `vec_new<...>` / `vec_push<...>` 繧呈・遉ｺ逧・↓菴ｿ縺・ｽ｢縺ｸ逶ｴ縺励�∫腸蠅・ｾ晏ｭ倥・譖匁乂縺輔ｒ豸医＠縺溘�・
  - `stack_new` / `stack_push` 縺ｯ `diag_out_of_memory` 縺ｮ impure 蛹悶↓霑ｽ蠕薙＠縺ｦ縺翫ｉ縺壹�～sizeof` focused test 縺ｧ compile failure 繧定ｵｷ縺薙＠縺ｦ縺・◆縲・
    - 繧ｷ繧ｰ繝阪メ繝｣繧・impure 縺ｫ菫ｮ豁｣縺励◆縲・
- 讀懆ｨｼ:
  - direct `runSingle` 縺ｫ繧医ｊ縲∽ｻ･荳九・ 4 繝輔ぃ繧､繝ｫ縺ｮ蜈ｨ繧ｱ繝ｼ繧ｹ繧貞�句挨遒ｺ隱阪＠縺溘�・
    - `stdlib/tests/error.n.md`
    - `stdlib/tests/diag.n.md`
    - `tests/stdlib/collections_diag.n.md`
    - `tests/compiler/sizeof.n.md`
  - 邨先棡:
    - `2 + 2 + 6 + 8 = 18` 繧ｱ繝ｼ繧ｹ縺吶∋縺ｦ pass縲・
  - `nodesrc/tests.js` 縺ｮ focused run 縺ｯ縺薙・迺ｰ蠅・〒縺ｯ騾ｲ謐苓｡ｨ遉ｺ縺御ｹ上＠縺城聞縺剰ｦ九∴繧九◆繧√�∝撫鬘悟・繧雁・縺代・ `runSingle` 繝吶・繧ｹ縺ｧ陦後▲縺溘�・
- 邨占ｫ・
  - `alloc/diag` 縺ｮ譁ｰ繝｢繝・Ν閾ｪ菴薙・謌千ｫ九＠縲’ocused test 縺ｧ螳牙ｮ壹＠縺溘�・
  - 谺｡縺ｯ縺薙・螟画峩繧・commit 縺励�《tdlib reboot 譛ｬ豬√・谺｡谿ｵ髫弱∈騾ｲ繧√ｋ縲・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (`Outcome` 隱ｭ縺ｿ蜿悶ｊ helper 縺ｮ霑ｽ蜉�縺ｨ struct 謚ｽ蜃ｺ蛻ｶ邏・・遒ｺ隱・

- 逶ｮ逧・
  - `Diag` 蜀崎ｨｭ險医・谺｡谿ｵ髫弱→縺励※縲～Result` 縺ｨ `Outcome` 繧貞・騾壹↓謇ｱ縺・helper 螻､縺ｮ譛�蟆城Κ蛻・ｒ蜈医↓謨ｴ蛯吶☆繧九�・
  - trait 閭ｽ蜉帙Δ繝・Ν縺ｸ騾ｲ繧�蜑阪↓縲～Outcome` 縺九ｉ險ｺ譁ｭ鄒､繧貞ｮ牙・縺ｫ隱ｭ縺ｿ蜿悶ｋ API 繧貞崋螳壹☆繧九�・
- 螟画峩:
  - `stdlib/alloc/diag/error.nepl`
    - `outcome_diags_or_empty <.T, .E> <(Outcome<.T, .E>)*>Diags>` 繧定ｿｽ蜉�縲・
    - `outcome_has_errors <.T, .E> <(Outcome<.T, .E>)->bool>` 繧定ｿｽ蜉�縲・
  - `stdlib/tests/error.n.md`
    - 荳願ｨ・2 helper 縺ｮ菴ｿ縺・婿縺ｨ邨先棡繧堤｢ｺ隱阪☆繧・focused doctest 縺ｸ譖ｴ譁ｰ縲・
- 隧ｦ陦後＠縺ｦ隕矩�√▲縺溷・螳ｹ:
  - `outcome_push_diag`
  - `outcome_map`
  - `outcome_map_err`
- 譬ｹ譛ｬ蜴溷屏:
  - 迴ｾ蝨ｨ縺ｮ險�隱槭〒縺ｯ縲《truct 縺九ｉ隍・焚 field 繧貞ｮ牙・縺ｫ蜿悶ｊ蜃ｺ縺励※蜀肴ｧ狗ｯ峨☆繧倶ｸ�闊ｬ逧・↑謇区ｮｵ縺御ｸ崎ｶｳ縺励※縺・ｋ縲・
  - `get o "result"` 縺ｨ `get o "diags"` 縺ｯ縺ｩ縺｡繧峨ｂ `o` 繧呈ｶ郁ｲｻ縺吶ｋ縺溘ａ縲∽ｸ｡譁ｹ繧貞酔譎ゅ↓蜿悶ｊ蜃ｺ縺励※譁ｰ縺励＞ `Outcome` 繧剃ｽ懊ｌ縺ｪ縺・�・
  - struct 縺ｫ蟇ｾ縺吶ｋ `match` 縺ｫ繧医ｋ蛻・ｧ｣繧ら樟迥ｶ縺ｮ譁・ｳ輔〒縺ｯ譛ｪ蟇ｾ蠢懊〒縲～Outcome r ds:` 縺ｮ繧医≧縺ｪ destructuring 縺ｯ parser error 縺ｫ縺ｪ繧九�・
  - 縺昴・縺溘ａ縲∬ｪｭ縺ｿ蜿悶ｊ蟆ら畑 helper 縺ｯ謌千ｫ九☆繧九′縲～Outcome` 繧呈峩譁ｰ繝ｻ蜀吝ワ縺吶ｋ helper 縺ｯ險�隱樊ｩ溯・蛛ｴ縺ｮ謾ｯ謠ｴ縺ｪ縺励↓螳牙・螳溯｣・〒縺阪↑縺・�・
- 蛻､譁ｭ:
  - 髢薙↓蜷医ｏ縺帙〒 raw field 謫堺ｽ懊ｄ ad-hoc helper 繧貞｢励ｄ縺吶→縲∝ｾ後〒 trait 閭ｽ蜉帙Δ繝・Ν縺ｨ陦晉ｪ√☆繧九�・
  - 莉雁屓縺ｯ謌千ｫ九☆繧玖ｪｭ縺ｿ蜿悶ｊ API 縺�縺代ｒ遒ｺ螳壹＠縲∵峩譁ｰ邉ｻ helper 縺ｯ compiler / 險�隱樊ｩ溯・縺ｮ謨ｴ蛯吝ｾ後∈蝗槭☆縲・
- 讀懆ｨｼ:
  - direct `runSingle`
    - `stdlib/tests/error.n.md`
      - 邨先棡: `2/2 pass`
    - `stdlib/tests/error.n.md`
    - `stdlib/tests/diag.n.md`
    - `tests/stdlib/collections_diag.n.md`
      繧偵∪縺ｨ繧√◆ focused 螳溯｡・
      - 邨先棡: `10/10 pass`
- 邨占ｫ・
  - `Outcome` 縺ｮ譛�蟆剰ｪｭ縺ｿ蜿悶ｊ helper 縺ｯ縲∫樟迥ｶ縺ｮ險�隱樊ｩ溯・縺ｧ繧ょｮ牙ｮ壹↓謠蝉ｾ帙〒縺阪ｋ縲・
  - `Outcome` 縺ｮ mutating helper 繧・library 蛛ｴ縺�縺代〒辟｡逅・↓騾ｲ繧√ｋ縺ｮ縺ｯ隱､繧翫〒縲∝ｿ・ｦ√↑繧・compiler / 險�隱樊ｩ溯・縺ｮ隱ｲ鬘後→縺励※謇ｱ縺・�・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (`core/traits` 縺ｮ譛�蟆乗�ｸ繧・stdlib 縺ｸ霑ｽ蜉�)

- 逶ｮ逧・
  - reboot 縺ｮ trait 閭ｽ蜉帙Δ繝・Ν繧・library 蛛ｴ縺九ｉ蜈ｷ菴灘喧縺吶ｋ縺溘ａ縲∫樟陦瑚ｨ�隱樊ｩ溯・縺ｧ螳牙ｮ壹↓謠蝉ｾ帙〒縺阪ｋ譛�蟆乗�ｸ繧貞・縺ｫ驟咲ｽｮ縺吶ｋ縲・
  - compiler 繝・せ繝亥・縺ｮ ad-hoc trait 螳｣險�繧・stdlib 縺ｮ豁｣蠑上Δ繧ｸ繝･繝ｼ繝ｫ縺ｸ鄂ｮ縺肴鋤縺医※縺・￥雜ｳ蝣ｴ繧剃ｽ懊ｋ縲・
- 螟画峩:
  - `stdlib/core/traits/copy.nepl`
    - `Clone` 縺ｨ `Copy` 繧・stdlib trait 縺ｨ縺励※螳夂ｾｩ縺励◆縲・
    - `bool`, `i32`, `i64`, `i128`, `u8`, `f32`, `f64` 縺ｸ impl 繧定ｿｽ蜉�縺励◆縲・
  - `stdlib/core/traits/stringify.nepl`
    - `Stringify` trait 縺ｨ蜈ｱ騾・helper `stringify` 繧定ｿｽ蜉�縺励◆縲・
    - `str`, `bool`, `i32`, `i64`, `i128`, `u8`, `f32`, `f64` 縺ｸ impl 繧定ｿｽ蜉�縺励◆縲・
    - 螳滉ｽ薙・譁・ｭ怜・蛹悶・ `alloc/string` 縺ｮ譌｢蟄倬未謨ｰ繧貞・蛻ｩ逕ｨ縺励◆縲・
  - `stdlib/core/traits/debug.nepl`
    - `Debug` trait 縺ｨ蜈ｱ騾・helper `debug_string` 繧定ｿｽ蜉�縺励◆縲・
    - `str` 縺ｯ蠑慕畑隨ｦ莉倥″縲√◎繧御ｻ･螟悶・蝓ｺ譛ｬ蝙九・ `Stringify` 縺ｫ蟋碑ｭｲ縺吶ｋ impl 繧定ｿｽ蜉�縺励◆縲・
  - `tests/stdlib/traits_text.n.md`
    - 譌･譛ｬ隱槭・ `[逶ｮ逧・繧ゅ￥縺ｦ縺江` 縺ｨ遒ｺ隱埼�・岼繧呈戟縺､ focused test 繧定ｿｽ蜉�縺励◆縲・
- 蛻､譁ｭ:
  - `Serialize` / `Deserialize` 縺ｯ trait 蝙句ｼ墓焚繧・format 蝙九′蠢・ｦ√↓縺ｪ繧翫ｄ縺吶￥縲∫樟陦瑚ｨ�隱樊ｩ溯・縺ｨ豁｣髱｢陦晉ｪ√☆繧句庄閭ｽ諤ｧ縺碁ｫ倥＞縲・
  - 縺昴・縺溘ａ莉雁屓縺ｯ `Copy` / `Clone` / `Stringify` / `Debug` 縺ｾ縺ｧ繧呈怙蟆乗�ｸ縺ｨ縺励※蜈医↓遒ｺ螳壹＠縲∵ｮ九ｊ縺ｯ `todo.md` 縺ｮ譛ｪ螳後ち繧ｹ繧ｯ縺ｨ縺励※邯ｭ謖√☆繧九�・
  - `Eq` / `Ord` / `Hash` 繧ょ酔讒倥↓縲∵里蟄倥・ ad-hoc 螳溯｣・→縺ｮ謨ｴ蜷医ｒ隕九↑縺後ｉ谺｡谿ｵ縺ｧ謇ｱ縺・�・
- compiler 菫ｮ豁｣:
  - generic 髢｢謨ｰ蜻ｼ縺ｳ蜃ｺ縺励・蝙句ｼ墓焚隗｣豎ｺ縺ｧ縲・未謨ｰ譛ｬ菴薙・蝙句､画焚譚溽ｸ帙°繧画耳隲悶〒縺阪◆蜈ｷ菴灘梛縺・`resolved_args` 縺ｸ蜿肴丐縺輔ｌ縺壹�∝腰逶ｸ蛹匁凾縺ｫ `Clone::clone` 縺梧悴隗｣豎ｺ縺ｮ縺ｾ縺ｾ谿九ｋ荳榊・蜷医′縺ゅ▲縺溘�・
  - `check_function` 縺ｧ generic 髢｢謨ｰ譛ｬ菴薙・蝙句､画焚 binding 繧・snapshot / restore 縺励▽縺､縲∝他縺ｳ蜃ｺ縺怜・縺ｧ縺ｯ `binding.ty` 縺ｨ `inst_ty` 縺ｮ邨・°繧牙推 type parameter 縺ｮ蜈ｷ菴灘梛繧貞・謗ｨ隲悶＠縺ｦ `resolved_args` 縺ｸ蜿肴丐縺吶ｋ繧医≧縺ｫ菫ｮ豁｣縺励◆縲・
  - monomorphize 縺ｮ trait impl 謗｢邏｢縺ｯ `unify` 繧剃ｽｿ縺｣縺ｦ縺・◆縺溘ａ縲…ast 逕ｨ縺ｮ邱ｩ縺・ｸ�閾ｴ隕丞援縺ｾ縺ｧ trait 隗｣豎ｺ縺ｫ豺ｷ蜈･縺励�～Stringify<i32>` 縺・`u8` / `bool` / `str` 縺ｪ縺ｩ隍・焚 impl 縺ｨ譖匁乂荳�閾ｴ縺吶ｋ荳榊・蜷医′縺ゅ▲縺溘�・
  - trait impl 驕ｸ謚槭・ `same_type` 縺ｫ繧医ｋ蜷御ｸ�蝙倶ｸ�閾ｴ縺ｸ蛻・ｊ譖ｿ縺医�》rait 隗｣豎ｺ縺ｨ謨ｰ蛟､ cast 縺ｮ隕丞援繧貞・髮｢縺励◆縲・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (`Serialize` / `Deserialize` trait 縺ｮ蟆主・縺ｨ receiverless trait method 隗｣豎ｺ菫ｮ豁｣)

- 逶ｮ逧・
  - trait 閭ｽ蜉帙Δ繝・Ν縺ｮ谿倶ｻｶ縺�縺｣縺・`Serialize` / `Deserialize` 繧・stdlib 縺ｸ霑ｽ蜉�縺吶ｋ縲・
  - `Deserialize::deserialize` 縺ｮ繧医≧縺ｫ receiver 繧貞叙繧峨★霑斐ｊ蛟､蛛ｴ縺ｧ `Self` 縺梧ｱｺ縺ｾ繧・trait method 縺後�“eneric helper 蜀・〒繧ょｮ牙ｮ壹↓蜊倡嶌蛹悶＆繧後ｋ繧医≧ compiler 繧剃ｿｮ豁｣縺吶ｋ縲・
- 螟画峩:
  - `stdlib/core/traits/serialize.nepl`
    - `Serialize` trait 縺ｨ helper `serialize` 繧定ｿｽ蜉�縺励◆縲・
    - `str`, `bool`, `i32`, `i64`, `i128`, `u8`, `f32`, `f64` 縺ｮ impl 繧定ｿｽ蜉�縺励◆縲・
  - `stdlib/core/traits/deserialize.nepl`
    - `Deserialize` trait 縺ｨ helper `deserialize` 繧定ｿｽ蜉�縺励◆縲・
    - `str`, `bool`, `i32`, `i64`, `i128`, `u8`, `f32`, `f64` 縺ｮ impl 繧定ｿｽ蜉�縺励◆縲・
    - `Result<_, i32>` 繧・`Result<_, StdErrorKind>` 縺ｫ蟇・○繧・`parse_err_to_std` 繧定ｿｽ蜉�縺励◆縲・
  - `tests/stdlib/traits_serde.n.md`
    - `[逶ｮ逧・繧ゅ￥縺ｦ縺江` 繧呈戟縺､ focused test 繧定ｿｽ蜉�縺励�《erialize / deserialize 縺ｮ蜈ｸ蝙倶ｽｿ逕ｨ萓九ｒ遒ｺ隱阪☆繧九ｈ縺・↓縺励◆縲・
- compiler 菫ｮ豁｣:
  - `Deserialize::deserialize s` 縺ｮ繧医≧縺ｪ receiverless trait method reference 縺ｯ縲∝ｾ捺擂 `Self` 逕ｨ縺ｮ驕企屬 fresh type var 繧・stack entry 縺ｫ遨阪ｓ縺ｧ縺・◆縲・
  - 縺昴・縺溘ａ generic helper `fn deserialize <.T: Deserialize> ...` 蜀・〒 `.T` 縺ｸ邨舌・莉倥°縺ｪ縺・∪縺ｾ `FuncRef::Trait { self_ty = Self }` 縺・HIR 縺ｫ谿九ｊ縲『asm codegen 縺ｧ `unknown function 'Deserialize::deserialize [self=Self]'` 縺ｨ縺ｪ縺｣縺ｦ縺・◆縲・
  - 菫ｮ豁｣蜀・ｮｹ:
    - trait method reference 繧堤ｩ阪・譎らせ縺ｧ縲√◎縺ｮ繧ｹ繧ｳ繝ｼ繝励↓蜚ｯ荳�縺ｮ `.T: Trait` 縺後≠繧句�ｴ蜷医・ fresh var 縺ｧ縺ｯ縺ｪ縺上◎縺ｮ `.T` 繧・`Self` 縺ｨ縺励※菴ｿ縺・ｈ縺・↓縺励◆縲・
    - fallback 縺ｮ trait call 隗｣豎ｺ繧ゅ�〉eceiver 蠑墓焚縺�縺代〒縺ｪ縺・expected return type 縺ｨ trait bound 縺九ｉ `Self` 繧呈耳隲悶〒縺阪ｋ繧医≧縺ｫ謨ｴ逅・＠縺溘�・
    - `check_function` 縺ｧ縺ｯ body 縺ｮ蝙句､画焚 binding 繧・restore 縺吶ｋ蜑阪↓ HIR 蜈ｨ菴薙・蝙・ID 繧・resolve 縺吶ｋ繧医≧縺ｫ縺励�∝腰逶ｸ蛹悶∈譛ｪ隗｣豎ｺ var 縺梧ｼ上ｌ縺ｪ縺・ｈ縺・↓縺励◆縲・
    - monomorphize 縺ｧ縺ｯ trait callee 縺ｮ self 隗｣豎ｺ繧・args 蜈磯�ｭ蝙九∈鬆ｼ繧峨★縲～self_ty` 閾ｪ菴薙・隗｣豎ｺ邨先棡縺�縺代ｒ菴ｿ縺・ｈ縺・↓謌ｻ縺励◆縲・
- 讀懆ｨｼ:
  - `NO_COLOR=false trunk build`
    - 邨先棡: success
  - `node nodesrc/tests.js -i tests/stdlib/traits_serde.n.md --no-stdlib --no-tree -o /tmp/tests-traits-serde.json -j 15`
    - 邨先棡: `2/2 pass`
- 邨占ｫ・
  - `Serialize` / `Deserialize` 縺ｮ stdlib trait 蟆主・縺ｯ謌千ｫ九＠縺溘�・
  - 譬ｹ譛ｬ蜴溷屏縺ｯ codegen 繧・monomorphize 縺ｧ縺ｯ縺ｪ縺上�〉eceiverless trait method reference 繧・generic body 縺ｸ謖√■霎ｼ繧�譎らせ縺ｮ `Self` 譚溽ｸ帙□縺｣縺溘�・
  - 谺｡縺ｯ `Result` / `Outcome` 繧貞・騾壹↓謇ｱ縺・helper / trait 譫�邨・∩縺ｸ騾ｲ繧�縲・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (`Outcome` 縺ｮ[隱ｭ/繧・縺ｿ[蜿・縺ｨ]繧・helper 繧定ｿｽ蜉�)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `Result` 縺ｨ `Outcome` 繧端蜈ｱ騾・縺阪ｇ縺・▽縺・縺ｫ[謇ｱ/縺ゅ▽縺犠縺・◆繧√�～Outcome` [蛛ｴ/縺後ｏ]縺ｫ繧・霆ｽ驥・縺代＞繧翫ｇ縺・縺ｪ[隱ｭ/繧・縺ｿ[蜿・縺ｨ]繧・helper 繧端謠・縺昴ｍ]縺医ｋ縲・
  - `match get o "result"` 繧端豈主屓/縺ｾ縺・°縺Ь[譖ｸ/縺犠縺九★縺ｫ縲～Outcome.result` 縺ｮ[謌仙凄/縺帙＞縺ｲ]繧端隱ｭ/繧・繧√ｋ繧医≧縺ｫ縺吶ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/diag/error.nepl`
    - `outcome_result`
    - `outcome_is_ok`
    - `outcome_is_err`
    繧定ｿｽ蜉�縲・
  - `stdlib/tests/error.n.md`
    - [荳願ｨ・縺倥ｇ縺・″] helper 縺ｮ[逶ｮ逧・繧ゅ￥縺ｦ縺江縺ｨ[遒ｺ隱・縺九￥縺ｫ繧転[蜀・ｮｹ/縺ｪ縺・ｈ縺・繧端霑ｽ險・縺､縺・″]縲・
- [蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `Outcome` 縺ｮ[譖ｴ譁ｰ邉ｻ/縺薙≧縺励ｓ縺代＞] helper 縺ｯ縲《truct field 繧端蛻・ｧ｣/縺ｶ繧薙°縺Ь縺励※[蜀肴ｧ狗ｯ・縺輔＞縺薙≧縺｡縺従縺吶ｋ[險�隱・縺偵ｓ縺脳[讖溯・/縺阪・縺・縺後∪縺�[蠑ｱ/繧医ｏ]縺・◆繧ー菫晉蕗/縺ｻ繧翫ｅ縺・縲・
  - [迴ｾ谿ｵ髫・縺偵ｓ縺�繧薙°縺Ь縺ｧ縺ｯ[隱ｭ/繧・縺ｿ[蜿・縺ｨ]繧・helper 繧端蜈・縺輔″]縺ｫ[蝗ｺ/縺九◆]繧√ｋ[譁ｹ/縺ｻ縺・縺後�《tdlib reboot 縺ｮ[荳頑ｵ・縺倥ｇ縺・ｊ繧・≧]縺ｨ縺励※[螳牙ｮ・縺ゅｓ縺ｦ縺Ь縺吶ｋ縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_test.js` 縺ｫ[逶ｴ謗･/縺｡繧・￥縺帙▽] JSON 繧端貂｡/繧上◆]縺励�～outcome_result` / `outcome_is_ok` / `outcome_is_err` 繧端菴ｿ/縺､縺犠縺・focused snippet 縺・`pass` 縺ｫ縺ｪ繧九％縺ｨ繧端遒ｺ隱・縺九￥縺ｫ繧転縲・


- `alloc/diag/error` 縺ｫ `into_outcome` / `result_like_result` / `result_like_is_ok` / `result_like_is_err` 繧定ｿｽ蜉�縲・
  - `Result` 縺ｨ `Outcome` 繧・overloading 縺ｧ蜈ｱ騾・helper 蜷阪↓謠・∴縺溘�・
  - 迴ｾ迥ｶ縺ｮ trait 讖溯・縺ｧ縺ｯ associated type 繧・trait generic abstraction 縺悟ｼｱ縺上�～Result<T,E>` 縺ｨ `Outcome<T,E>` 繧堤┌逅・↓ trait 荳�縺､縺ｸ謚ｼ縺苓ｾｼ繧�繧医ｊ helper 縺ｮ譁ｹ縺瑚・辟ｶ縺�縺｣縺溘�・
- `stdlib/tests/error.n.md` 縺ｫ `result_and_outcome_common_helpers` 繧定ｿｽ蜉�縺励�∬ｻｽ驥・API 縺ｨ rich API 縺ｮ蜈ｱ騾夊ｪｭ縺ｿ蜿悶ｊ繧・focused 縺ｫ遒ｺ隱阪�・
# 2026-03-09 菴懈･ｭ繝｡繝｢ (compiler 蜑肴署蝗ｺ螳・ `.nepl` 縺ｧ陦ｨ迴ｾ縺ｧ縺阪ｋ primitive 縺ｮ Copy 繧・stdlib impl 縺ｸ遘ｻ陦・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `todo.md` 縺ｮ compiler 蜑肴署蝗ｺ螳壹↓蠕薙＞縲～Copy` 蛻､螳壹・ compiler 蝗ｺ螳夊｡ｨ繧堤ｸｮ蟆上☆繧九�・
  - `.nepl` 繧ｽ繝ｼ繧ｹ縺ｧ陦ｨ迴ｾ縺ｧ縺阪ｋ primitive 縺ｫ縺､縺・※縺ｯ縲《tdlib 蛛ｴ縺ｮ `impl Copy/Clone` 繧貞髪荳�縺ｮ譬ｹ諡�縺ｫ蟇・○繧九�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `TypeCtx::is_copy_with_trait_model` 縺ｯ trait 繝｢繝ｼ繝峨〒繧・`i32` / `u8` / `f32` / `bool` / `str` / `()` 繧貞崋螳夊｡ｨ縺ｧ copy 縺ｨ縺ｿ縺ｪ縺励※縺・◆縲・
  - 縺薙・縺溘ａ `core/traits/copy.nepl` 縺ｫ蜷悟・螳ｹ縺ｮ impl 繧貞ｮ夂ｾｩ縺励※繧ゅ�［ove 隕丞援縺ｮ譛�邨ょ愛螳壹′ compiler 蜀・Κ縺ｮ遏･隴倥∈萓晏ｭ倥＠縺溘∪縺ｾ縺�縺｣縺溘�・
  - 荳�譁ｹ縺ｧ縲∝盾辣ｧ蝙九ｄ `never` 縺ｯ迴ｾ迥ｶ縺ｮ險�隱樊ｩ溯・縺ｧ縺ｯ `.nepl` 蛛ｴ縺ｫ閾ｪ辟ｶ縺ｪ impl 繧堤ｽｮ縺阪↓縺上￥縲∝酔縺俶桶縺・↓縺ｯ縺ｧ縺阪↑縺・�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/core/traits/copy.nepl`
    - `str` 縺ｸ縺ｮ `Clone` / `Copy` impl 繧定ｿｽ蜉�縲・
    - `()` 縺ｸ縺ｮ `Clone` / `Copy` impl 繧定ｿｽ蜉�縲・
  - `nepl-core/src/types.rs`
    - trait 繝｢繝ｼ繝峨・ `is_copy_with_trait_model` 縺九ｉ縲～.nepl` 蛛ｴ縺ｧ陦ｨ迴ｾ縺ｧ縺阪ｋ primitive (`Unit` / `I32` / `U8` / `F32` / `Bool` / `Str`) 縺ｮ蝗ｺ螳夊｡ｨ蛻､螳壹ｒ蜑企勁縲・
    - 荳願ｨ倥・ `has_copy_impl_target` 縺ｫ繧医ｋ trait impl 逋ｻ骭ｲ邨先棡縺�縺代〒蛻､螳壹☆繧九ｈ縺・､画峩縲・
    - 蝗ｺ螳夊｡ｨ縺ｫ谿九＠縺溘・縺ｯ縲∫樟谿ｵ髫弱〒 source impl 繧定・辟ｶ縺ｫ謖√■縺ｫ縺上＞ `Never` 縺ｨ蜿ら・蝙九□縺代↓邨槭▲縺溘�・
  - `tests/compiler/move_effect.n.md`
    - `core/traits/copy` 繧・import 縺励◆縺ｨ縺阪�～str` 縺ｮ蜀榊茜逕ｨ縺・`Copy` impl 縺ｫ繧医▲縺ｦ謌千ｫ九☆繧九こ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
    - `()` 縺ｮ蜀榊茜逕ｨ縺・`Copy` impl 縺ｫ繧医▲縺ｦ謌千ｫ九☆繧九こ繝ｼ繧ｹ繧定ｿｽ蜉�縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build` -> success
  - `node` + `nodesrc/compiler_loader` 縺ｫ繧医ｋ compile-only focused check:
    - `#import "core/traits/copy" as *` 繧貞性繧� `str` 蜀榊茜逕ｨ snippet -> `OK`
    - 蜷・`()` 蜀榊茜逕ｨ snippet -> `OK`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - compiler 縺ｮ `Copy` 蝗ｺ螳夊｡ｨ縺ｯ邵ｮ蟆上＆繧後�～.nepl` 蛛ｴ縺ｫ impl 繧堤ｽｮ縺代ｋ primitive 縺ｯ stdlib impl 縺ｫ蟇・○繧峨ｌ縺溘�・
  - 谿九ｋ迚ｹ蛻･謇ｱ縺・・縲∫樟迥ｶ縺ｮ險�隱槭〒 source impl 繧堤ｽｮ縺阪↓縺上＞蜿ら・蝙九→ `never` 縺ｧ縺ゅｋ縲・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (compiler 蜑肴署蝗ｺ螳・ LLVM codegen 縺ｮ蜑肴ｮｵ雋ｬ蜍吶ｒ `compiler.rs` 縺ｫ髮・ｴ・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `todo.md` 縺ｮ compiler 蜑肴署蝗ｺ螳壹↓蠕薙＞縲´LVM 邨瑚ｷｯ縺ｧ繧・codegen 縺・typecheck / move check / target precheck / codegen precheck 繧呈干縺医↑縺・ｽ｢縺ｸ蟇・○繧九�・
  - wasm/llvm 縺ｮ蜑肴ｮｵ險ｺ譁ｭ繧・`compiler.rs` 蛛ｴ縺ｮ蜈ｱ騾・lowering 縺ｸ髮・ｴ・＠縲…odegen 蛻ｰ驕泌ｾ後・逕滓・蟆ゆｻｻ縺ｫ霑代▼縺代ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `compile_module` 縺ｯ wasm 逕ｨ縺ｫ target precheck -> typecheck -> monomorphize -> move check -> drop 謖ｿ蜈･繧偵∪縺ｨ繧√※縺・◆縺後�´LVM 邨瑚ｷｯ縺ｯ `codegen_llvm.rs` 蜀・〒蛻･縺ｫ `precheck_module_before_codegen` / `typecheck` / `monomorphize` / `precheck_llvm_codegen` 繧貞ｮ溯｡後＠縺ｦ縺・◆縲・
  - 縺昴・縺溘ａ縲∝酔縺伜・蜉帙〒繧・wasm 縺ｨ llvm 縺ｧ險ｺ譁ｭ逕滓・雋ｬ蜍吶′蛻・淵縺励�～codegen_llvm` 縺悟燕谿ｵ縺ｮ螟ｱ謨励ｒ `TypecheckFailed` 縺ｫ貎ｰ縺励※謚ｱ縺郁ｾｼ繧�讒矩��縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nepl-core/src/compiler.rs`
    - `PreparedProgram` 繧定ｿｽ蜉�縺励�》arget precheck -> typecheck -> monomorphize -> move check -> drop 謖ｿ蜈･縺ｾ縺ｧ繧・`prepare_module_for_codegen` 縺ｫ髮・ｴ・�・
    - `PreparedLlvmProgram` 繧定ｿｽ蜉�縺励�´LVM entry 隗｣豎ｺ繝ｻreachable 髮・粋讒狗ｯ峨・`precheck_llvm_codegen` 繧・`prepare_module_for_llvm_codegen` 縺ｫ髮・ｴ・�・
    - `compile_module` 縺ｯ `prepare_module_for_codegen` 繧剃ｽｿ縺・ｽ｢縺ｸ螟画峩縺励�『asm 蜑肴ｮｵ繧ょ酔縺倡ｵ瑚ｷｯ繧帝�壹ｋ繧医≧縺ｫ縺励◆縲・
  - `nepl-core/src/codegen_llvm.rs`
    - `emit_ll_from_module_for_target` 縺ｯ `compiler::prepare_module_for_llvm_codegen` 繧貞他縺ｶ縺�縺代↓縺励�∫峩謗･縺ｮ typecheck/precheck 蜻ｼ縺ｳ蜃ｺ縺励ｒ髯､蜴ｻ縲・
    - `try_lower_entry_from_hir` 縺ｯ prechecked artifact (`PreparedLlvmProgram`) 繧貞女縺大叙繧翫�∬ｨｺ譁ｭ逕滓・繧定｡後ｏ縺・lowering 縺�縺代ｒ諡・ｽ薙☆繧句ｽ｢縺ｸ螟画峩縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build`
    - 邨先棡: success
  - `node nodesrc/tests.js -i tests/compiler/llvm_target.n.md -i tests/compiler/raw_body_precheck.n.md -i tests/compiler/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-llvm-frontload.json -j 15`
    - 邨先棡: `8/8 pass`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - LLVM codegen 縺ｯ蜑肴ｮｵ繧堤峩謗･螳溯｡後○縺壹�～compiler.rs` 縺ｮ蜈ｱ騾・lowering 繧貞燕謠舌↓蜍輔￥蠖｢縺ｸ蟇・▲縺溘�・
  - 縺ｾ縺� `nepl-cli` 縺ｮ LLVM 蛻・ｲ舌・ `codegen_llvm::emit_ll_from_module_for_target` 繧堤峩謗･蜻ｼ縺ｶ縺後�√◎縺ｮ蜀・Κ縺ｯ蜈ｱ騾・front-end 繧帝�壹ｋ縺溘ａ縲∬ｲｬ蜍吝・髮｢縺ｮ荳ｻ逵ｼ縺ｯ貅�縺溘＠縺溘�・
  - 谿九ｋ compiler 蜑肴署蝗ｺ螳壹・譛ｬ豬√・縲…opy/clone 髱槭ワ繝ｼ繝峨さ繝ｼ繝牙喧縺ｮ谿倶ｻｶ縺ｨ縲～Diag.kind` 險�隱樊ｩ溯・縺ｮ貅門ｙ縺ｧ縺ゅｋ縲・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (compiler 蜑肴署蝗ｺ螳・ LLVM codegen 縺九ｉ譌ｧ front-end helper 繧帝勁蜴ｻ)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `codegen_llvm.rs` 縺ｫ谿九▲縺ｦ縺・◆譌ｧ front-end helper 鄒､繧帝勁蜴ｻ縺励�´LVM codegen 縺悟・縺ｳ typecheck/precheck 邨瑚ｷｯ繧貞・蛹・＠縺ｪ縺・憾諷九ｒ菫昴▽縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nepl-core/src/codegen_llvm.rs`
    - 譛ｪ菴ｿ逕ｨ縺ｫ縺ｪ縺｣縺ｦ縺・◆ `compute_reachable_hint` / `build_hir_for_llvm_lowering` / `try_build_hir_with_target` 縺ｨ縲√◎縺ｮ陬懷勧縺�縺｣縺・reachable/callee 蜿朱寔 helper 鄒､繧貞炎髯､縲・
    - `emit_ll_from_module_for_target` 縺・`compiler::prepare_module_for_llvm_codegen` 莉･螟悶・ front-end 邨瑚ｷｯ繧呈戟縺溘↑縺・憾諷九↓縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build`
    - 邨先棡: success
  - `node nodesrc/tests.js -i tests/compiler/llvm_target.n.md -i tests/compiler/raw_body_precheck.n.md -i tests/compiler/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-llvm-frontload-2.json -j 15`
    - 邨先棡: `8/8 pass`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - LLVM codegen 蛛ｴ縺ｫ縺ｯ蜑肴ｮｵ繧偵ｄ繧顔峩縺・helper 縺梧ｮ九▲縺ｦ縺翫ｉ縺壹�∬ｲｬ蜍吶・ `compiler.rs` 縺ｮ蜈ｱ騾・lowering 縺ｸ蝗ｺ螳壹＆繧後◆縲・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (`alloc/collections/stack` 繧・typed pointer 蛹悶＠縲～uwok` 繧貞ｰ主・)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `todo.md` 縺ｮ `alloc` 蜀肴ｧ狗ｯ峨↓蜈育ｫ九■縲～Stack<.T>` 縺ｮ[蜀・Κ/縺ｪ縺・・][陦ｨ迴ｾ/縺ｲ繧・≧縺偵ｓ]繧・raw `i32` 縺九ｉ `MemPtr<u8>` / `MemPtr<.T>` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[蟇・繧・縺帙ｋ縲・
  - `Result` 繧・pipe [險俶ｳ・縺阪⊇縺・縺ｧ[騾｣邯・繧後ｓ縺槭￥][蜃ｦ逅・縺励ｇ繧馨縺吶ｋ縺ｨ縺阪・[蜀鈴聞/縺倥ｇ縺・■繧・≧]縺輔ｒ[貂・縺ｸ]繧峨☆縺溘ａ縲～unwrap_ok` 縺ｮ[遏ｭ邵ｮ蜷・縺溘ｓ縺励ｅ縺上ａ縺Ь `uwok` 繧・`core/result` 縺ｫ霑ｽ蜉�縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `Stack` 縺ｯ繝倥ャ繝�[蜈ｨ菴・縺懊ｓ縺溘＞]繧・raw `i32` 縺ｧ[菫晄戟/縺ｻ縺肋縺励�～load_i32` / `store_i32` / `realloc_raw` 縺ｸ[逶ｴ邨・縺｡繧・▲縺代▽]縺励※縺・◆縲・
  - 縺薙・縺ｾ縺ｾ縺ｧ縺ｯ `core/mem` 縺ｮ蝙句ｮ牙・蛹悶′ `alloc/collections` 縺ｸ[豕｢蜿・縺ｯ縺阪ｅ縺・縺帙★縲～Vec` 縺ｮ `MemPtr` 蛹悶→[謨ｴ蜷・縺帙＞縺斐≧]縺励↑縺・�・
  - [菴ｿ逕ｨ萓・縺励ｈ縺・ｌ縺Ь縺ｧ縺ｯ `unwrap_ok<Stack<i32>, Diag>` 縺啓郢ｰ/縺従繧骸霑・縺九∴]縺輔ｌ縲～new |> push |> push` 縺ｮ繧医≧縺ｪ[騾｣骼・繧後ｓ縺評縺啓隱ｭ/繧・縺ｿ縺ｫ縺上°縺｣縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/collections/stack.nepl`
    - `Stack<.T>.hdr` 繧・`MemPtr<u8>` 縺ｫ螟画峩縲・
    - 繝倥ャ繝�縺ｮ `len/cap/data_ptr` 繧端隱ｭ/繧・繧�[蜀・Κ/縺ｪ縺・・] helper (`stack_header_len_ptr` / `stack_header_cap_ptr` / `stack_header_data_ptr_ptr` / `stack_len_raw` / `stack_cap_raw` / `stack_data_ptr`) 繧定ｿｽ蜉�縲・
    - `stack_new` / `stack_push` / `stack_pop` / `stack_peek` / `stack_len` / `stack_clear` / `stack_free` 繧・typed memory API [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ譖ｴ譁ｰ縲・
    - `stack_free` 縺ｯ `dealloc_ptr` 縺ｮ `Result<(), Diag>` 繧・`uwok` 縺ｧ[豸郁ｲｻ/縺励ｇ縺・・]縺吶ｋ蠖｢縺ｸ菫ｮ豁｣縲・
    - [菴ｿ逕ｨ萓・縺励ｈ縺・ｌ縺Ь縺ｮ doctest 繧・`uwok` [蝓ｺ貅・縺阪§繧・ｓ]縺ｸ蟇・○縺溘�・
  - `stdlib/core/result.nepl`
    - `uwok` (`unwrap_ok` 縺ｮ[遏ｭ邵ｮ蜷・縺溘ｓ縺励ｅ縺上ａ縺Ь) 繧定ｿｽ蜉�縲・
    - `uwerr` (`unwrap_err` 縺ｮ[遏ｭ邵ｮ蜷・縺溘ｓ縺励ｅ縺上ａ縺Ь) 繧りｿｽ蜉�縲・
  - `stdlib/core/traits/deserialize.nepl`
    - ruby [險俶ｳ・縺阪⊇縺・縺ｮ[蛻・牡/縺ｶ繧薙°縺､]繧剃ｿｮ豁｣縺励�～[莠ｺ髢・縺ｫ繧薙￡繧転[蜷・繧�]縺疏 縺ｫ邨ｱ荳�縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_test.js` 縺ｫ[逶ｴ謗･/縺｡繧・￥縺帙▽] JSON 繧端貂｡/繧上◆]縺励※ focused snippet 繧・2 莉ｶ[螳溯｡・縺倥▲縺薙≧]縲・
    - `<Stack<i32>> new |> uwok |> push 10 |> uwok |> push 20 |> uwok` + `len` -> `pass`
    - `stack_free<i32>` 繧端蜷ｫ/縺ｵ縺従繧� snippet -> `pass`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - `stack` 縺ｯ `Vec` 縺ｨ[蜷・縺翫↑]縺俶婿蜷代〒 typed pointer [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ遘ｻ縺｣縺溘�・
  - `uwok` 縺ｯ `core/result` 縺ｮ縺ｿ縺ｫ[螳夂ｾｩ/縺ｦ縺・℃]縺励�ー驥崎､・縺｡繧・≧縺ｵ縺従[螳｣險�/縺帙ｓ縺偵ｓ]縺ｯ[驕ｿ/縺評縺代※縺・ｋ縲・
  - `vec` 縺ｪ縺ｩ縺ｮ `alloc/collections` 繧ゅ�√％縺ｮ[隕句・/縺ｿ縺�]縺夕讒矩��/縺薙≧縺槭≧]縺ｨ `uwok` 繧端蝓ｺ貅・縺阪§繧・ｓ]縺ｫ縺昴ｍ縺医※縺・￥縲・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (alloc/collections/vec: 繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝郁ｦ句・縺励・譁ｰ讓呎ｺ悶∈縺ｮ霑ｽ蠕・

- 逶ｮ逧・
  - `alloc/collections/vec.nepl` 縺ｮ[蜈磯�ｭ/縺帙ｓ縺ｨ縺・縺ｨ[蝓ｺ遉・縺阪◎] API 縺ｮ繝峨く繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝医ｒ縲～stdlib/core/traits/deserialize.nepl` 繧端蝓ｺ貅・縺阪§繧・ｓ]縺ｫ縺励◆[譁ｰ讓呎ｺ・縺励ｓ縺ｲ繧・≧縺倥ｅ繧転縺ｮ[隕句・/縺ｿ縺�]縺夕讒矩��/縺薙≧縺槭≧]縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
- 螟画峩:
  - `stdlib/alloc/collections/vec.nepl`
    - [蜈磯�ｭ/縺帙ｓ縺ｨ縺・繧ｳ繝｡繝ｳ繝医ｒ `# vec` 蠖｢蠑上∈螟画峩縲・
    - `Vec`, `vec_new`, `vec_with_capacity`, `vec_len`, `vec_cap`, `vec_data_ptr`, `vec_data_mem_ptr` 縺ｮ繧ｳ繝｡繝ｳ繝医ｒ `##` / `### [逶ｮ逧・繧ゅ￥縺ｦ縺江` / `### [螳溯｣・縺倥▲縺昴≧]` / `### [豕ｨ諢・縺｡繧・≧縺Ь` / `### [險育ｮ鈴㍼/縺代＞縺輔ｓ繧翫ｇ縺・` / `### [菴ｿ逕ｨ萓・縺励ｈ縺・ｌ縺Ь` 縺ｫ謨ｴ逅・�・
  - [螳溯｣・縺倥▲縺昴≧]譛ｬ菴薙・螟画峩縺励※縺・↑縺・�・
- 讀懆ｨｼ:
  - `printf '{...}' | node nodesrc/run_test.js` 縺ｫ繧医ｊ縲～new<i32> |> push 10 |> push 20` 縺ｨ `vec_len` 繧端菴ｿ/縺､縺犠縺・focused 螳溯｡後′ pass縲・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (compiler 蜑肴署蝗ｺ螳・ `#entry` 險ｺ譁ｭ縺ｮ span 繧・dummy 縺九ｉ螳滉ｽ咲ｽｮ縺ｸ菫ｮ豁｣)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `TypeEntryFunctionMissingOrAmbiguous` 縺・`Span::dummy()` 繧端霑・縺九∴]縺励※縺・◆ compiler [蛛ｴ/縺後ｏ]縺ｮ[荳榊・蜷・縺ｵ縺舌≠縺Ь繧端菫ｮ豁｣/縺励ｅ縺・○縺Ь縺励�～#entry` 縺ｮ[隴伜挨蟄・縺励″縺ｹ縺､縺余[菴咲ｽｮ/縺・■]縺ｸ[險ｺ譁ｭ/縺励ｓ縺�繧転繧端邨・繧�縺兢縺ｳ[莉・縺､]縺代ｋ縲・
  - LLVM [邨瑚ｷｯ/縺代＞繧江縺ｧ[蠕梧ｮｵ/縺薙≧縺�繧転縺ｫ[谿・縺ｮ縺転縺｣縺ｦ縺・◆ `entry function ... was not found in lowered module` 繧ゅ�∝酔縺・`diag id` 縺ｨ span 縺ｫ[蟇・繧・縺帙ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `typecheck` 縺ｯ `Directive::Entry` 縺ｮ span 繧端隕・縺ｿ]縺医※縺・◆縺後�～resolved_entry` 縺ｮ[譖匁乂/縺ゅ＞縺ｾ縺Ь繝ｻ[谺�關ｽ/縺代▽繧峨￥]繧端蝣ｱ蜻・縺ｻ縺・％縺従縺吶ｋ縺ｨ縺阪↓ `Span::dummy()` 繧端菴ｿ/縺､縺犠縺｣縺ｦ縺・◆縲・
  - `compiler::resolve_hir_entry_name` 繧ゅ�〕owering [蠕・縺脳縺ｫ entry 縺啓隕・縺ｿ]縺､縺九ｉ縺ｪ縺・→ `diag id` 縺ｪ縺励・dummy span [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｮ[險ｺ譁ｭ/縺励ｓ縺�繧転縺ｸ[關ｽ/縺馨縺｡縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nepl-core/src/typecheck.rs`
    - `entry` 繧・`Option<(String, Span)>` 縺ｧ[菫晄戟/縺ｻ縺肋縺吶ｋ繧医≧縺ｫ螟画峩縲・
    - `TypeEntryFunctionMissingOrAmbiguous` 繧・`#entry` 縺ｮ[蜷榊燕/縺ｪ縺ｾ縺・ span 縺ｸ[莉・縺､]縺代ｋ繧医≧菫ｮ豁｣縲・
    - `check_function` 縺ｮ entry [蛻､螳・縺ｯ繧薙※縺Ь繧・tuple [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[霑ｽ蠕・縺､縺・§繧・≧]縲・
  - `nepl-core/src/compiler.rs`
    - `resolve_hir_entry_name` 縺ｫ `module` 繧端貂｡/繧上◆]縺励�～#entry` [謗｢邏｢/縺溘ｓ縺輔￥] helper 繧定ｿｽ蜉�縲・
    - lowering [蠕・縺脳縺ｫ entry 縺啓隕・縺ｿ]縺､縺九ｉ縺ｪ縺Ъ蝣ｴ蜷・縺ｰ縺ゅ＞]繧・`DiagnosticId::TypeEntryFunctionMissingOrAmbiguous` 縺ｨ `#entry` 縺ｮ span 繧端霑・縺九∴]縺吶ｈ縺・↓菫ｮ豁｣縲・
  - `tests/compiler/compile_fail_diag_location.n.md`
    - `entry_missing_uses_entry_directive_span` 繧定ｿｽ蜉�縲・
    - `diag_id: 3092` 縺ｨ `diag_span: 2:8` 繧端遒ｺ隱・縺九￥縺ｫ繧転縺吶ｋ compile_fail 繧定ｿｽ蜉�縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build`
    - [邨先棡/縺代▲縺犠: success
  - `node nodesrc/tests.js -i tests/compiler/compile_fail_diag_location.n.md --no-stdlib --no-tree -o /tmp/tests-entry-diag-location.json -j 15`
    - [邨先棡/縺代▲縺犠: `4/4 pass`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - `#entry` 縺ｫ[髢｢/縺九ｓ]縺吶ｋ compiler 險ｺ譁ｭ縺ｯ縲～diag id` 縺�縺代〒縺ｪ縺充菴咲ｽｮ/縺・■]繧・蜑肴ｮｵ/縺懊ｓ縺�繧転縺ｧ[螳牙ｮ・縺ゅｓ縺ｦ縺Ь縺励※[蜿・縺ｨ]繧後ｋ繧医≧縺ｫ縺ｪ縺｣縺溘�・
  - codegen [蛻ｰ驕泌ｾ・縺ｨ縺・◆縺､縺脳縺ｮ entry [谺�關ｽ/縺代▽繧峨￥]縺ｯ縲’ront-end lowering 縺ｮ[荳肴紛蜷・縺ｵ縺帙＞縺斐≧]縺ｨ縺励※[謇ｱ/縺ゅ▽縺犠縺医ｋ[遽・峇/縺ｯ繧薙＞]縺ｾ縺ｧ[邵ｮ蟆・縺励ｅ縺上＠繧・≧]縺輔ｌ縺溘�・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (`RegionToken` / `RingBuffer` 縺ｮ move 豸郁ｲｻ繧・field 蜊倅ｽ阪∈蛻・ｊ譖ｿ縺・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `todo.md` 縺ｮ `core/mem` / `alloc` [螳牙・蛹・縺ゅｓ縺懊ｓ縺犠繧端騾ｲ/縺吶☆]繧√ｋ縺・∴縺ｧ縲～RegionToken<.T>` 繧・`RingBuffer<.T>` 縺ｮ[謇�譛芽�・縺励ｇ繧・≧縺励ｃ]繧端郢ｰ/縺従繧骸霑・縺九∴]縺・move 縺励※縺励∪縺・邂・園/縺九＠繧Ⅹ繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺吶ｋ縲・
  - `tests/compiler/prelude_copy.n.md`縲～tests/stdlib/ringbuffer_collections.n.md`縲～tests/stdlib/queue_collections.n.md` 縺啓螳牙ｮ・縺ゅｓ縺ｦ縺Ь縺励※[騾・縺ｨ縺馨繧擬迥ｶ諷・縺倥ｇ縺・◆縺Ь縺ｾ縺ｧ[謖・繧・縺｣縺ｦ縺・￥縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `MemPtr<.T>` 縺ｯ `Copy` 縺ｨ縺励※[謇ｱ/縺ゅ▽縺犠縺・◆縺・′縲ー謇�譛芽�・縺励ｇ繧・≧縺励ｃ]縺ｧ縺ゅｋ `RegionToken<.T>` 繧・`RingBuffer<.T>` 縺ｯ `Copy` 縺ｧ縺ｯ縺ｪ縺・�・
  - 縺昴・縺溘ａ `region_ptr token` 繧・`ringbuffer_len rb` 縺ｮ繧医≧縺ｫ[謇�譛芽�・縺励ｇ繧・≧縺励ｃ]繧端荳ｸ/縺ｾ繧犠縺斐→[陬懷勧/縺ｻ縺倥ｇ][髢｢謨ｰ/縺九ｓ縺吶≧]縺ｸ[貂｡/繧上◆]縺兌螳溯｣・縺倥▲縺昴≧]縺�縺ｨ縲～get ... "ptr"` 繧・`get ... "hdr"` 縺啓隍・焚蝗・縺ｵ縺上☆縺・°縺Ь縺ｮ move 縺ｫ[隕・縺ｿ]縺医※[螟ｱ謨・縺励▲縺ｱ縺Ь縺励※縺・◆縲・
  - compiler [蛛ｴ/縺後ｏ]縺ｧ繧ゅ�“eneric `Copy` / `Clone` impl 繧端蜈ｷ菴灘梛/縺舌◆縺・※縺阪′縺歉縺ｸ[蠖・縺・縺ｦ繧擬髫・縺輔＞]縺ｫ[蜊倡ｴ・縺溘ｓ縺倥ｅ繧転縺ｪ `same_type` [豈碑ｼ・縺ｲ縺九￥]縺励°縺励※縺翫ｉ縺壹�～MemPtr<i32>` 縺・`impl Copy<MemPtr<.T>>` 縺ｫ[荳�閾ｴ/縺・▲縺｡]縺励↑縺Ъ荳榊・蜷・縺ｵ縺舌≠縺Ь縺後≠縺｣縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nepl-core/src/types.rs`
    - `type_pattern_matches` 繧定ｿｽ蜉�縺励�～impl Copy<MemPtr<.T>>` 縺ｮ繧医≧縺ｪ[蝙句､画焚/縺九◆縺ｸ繧薙☆縺・[蜈･/縺Ь繧・impl 縺啓蜈ｷ菴灘梛/縺舌◆縺・※縺阪′縺歉縺ｸ[荳�閾ｴ/縺・▲縺｡]縺吶ｋ縺九ｒ[蛻､螳・縺ｯ繧薙※縺Ь縺ｧ縺阪ｋ繧医≧縺ｫ縺励◆縲・
  - `nepl-core/src/typecheck.rs`
    - `Copy` / `Clone` 縺ｨ trait impl [謗｢邏｢/縺溘ｓ縺輔￥]縺ｧ `same_type` 縺ｧ縺ｯ縺ｪ縺・`type_pattern_matches` 繧端菴ｿ/縺､縺犠縺・ｈ縺・↓螟画峩縲・
    - generic impl 縺ｯ[蠖馴擇/縺ｨ縺・ａ繧転 `Copy` / `Clone` trait 縺ｮ縺ｿ[險ｱ蜿ｯ/縺阪ｇ縺犠縺吶ｋ繧医≧縺ｫ縺励�√◎繧啓莉･螟・縺・′縺Ь縺ｯ[蠕捺擂騾・縺倥ｅ縺・ｉ縺・←縺馨繧骸諡貞凄/縺阪ｇ縺ｲ]縺吶ｋ縲・
  - `nepl-core/src/passes/move_check.rs`
    - builtin/user `get` 縺ｮ[隧穂ｾ｡/縺ｲ繧・≧縺犠縺ｧ縲ー蜿門ｾ・縺励ｅ縺ｨ縺従[邨先棡/縺代▲縺犠縺・`Copy` 縺ｪ繧・base 繧・shared borrow [逶ｸ蠖・縺昴≧縺ｨ縺・縺ｧ[險ｪ蝠・縺ｻ縺・ｂ繧転縺吶ｋ繧医≧縺ｫ縺励◆縲・
  - `stdlib/core/traits/copy.nepl`
    - `MemPtr<.T>` 縺ｮ `Copy` / `Clone` impl 繧定ｿｽ蜉�縲・
  - `stdlib/core/mem.nepl`
    - `region_ptr_at` / `dealloc_region` 縺ｪ縺ｩ繧偵�～token` 縺昴・繧ゅ・縺ｧ縺ｯ縺ｪ縺・`get token "ptr"` / `get token "size"` 繧端蜈・縺輔″]縺ｫ[譚溽ｸ・縺昴￥縺ｰ縺従縺励※[菴ｿ/縺､縺犠縺・蠖｢/縺九◆縺｡]縺ｸ螟画峩縲・
  - `stdlib/alloc/string.nepl`
    - `RegionToken<u8>` 繧端隍・焚蝗・縺ｵ縺上☆縺・°縺Ь helper 縺ｫ[貂｡/繧上◆]縺励※縺・◆[邂・園/縺九＠繧Ⅹ繧偵�～base` / `scratch` / `out_data` 縺ｪ縺ｩ縺ｮ `MemPtr<u8>` 縺ｸ[蜈・縺輔″]縺ｫ[蛻・ｧ｣/縺ｶ繧薙°縺Ь縺励※[謇ｱ/縺ゅ▽縺犠縺・蠖｢/縺九◆縺｡]縺ｸ螟画峩縲・
  - `stdlib/alloc/collections/ringbuffer.nepl`
    - `RingBuffer<.T>.hdr` 繧・`MemPtr<u8>` 縺ｨ縺励※[荳�蠎ｦ/縺・■縺ｩ][蜿・縺ｨ]繧骸蜃ｺ/縺�]縺励�～*_from_hdr` helper 縺ｸ[貂｡/繧上◆]縺兌螳溯｣・縺倥▲縺昴≧]縺ｸ謨ｴ逅・�・
    - `ringbuffer_with_capacity` / `ringbuffer_push_back` / `ringbuffer_pop_front` / `ringbuffer_peek_front` / `ringbuffer_clear` / `ringbuffer_free` 繧端謇�譛芽�・縺励ｇ繧・≧縺励ｃ]縺ｮ[蜀肴ｶ郁ｲｻ/縺輔＞縺励ｇ縺・・]縺後↑縺Ъ蠖｢/縺九◆縺｡]縺ｸ譖ｸ縺咲峩縺励◆縲・
  - `tests/compiler/prelude_copy.n.md`
    - `MemPtr<i32>` 繧端郢ｰ/縺従繧骸霑・縺九∴]縺夕隱ｭ/繧・繧√ｋ縺薙→縲～Copy` 繧端譛ｪ遏･/縺ｿ縺｡] trait 縺ｨ縺励※[謇ｱ/縺ゅ▽縺犠繧上↑縺・％縺ｨ繧端遒ｺ隱・縺九￥縺ｫ繧転縺吶ｋ focused test 繧定ｿｽ蜉�縲・
  - `tests/stdlib/ringbuffer_collections.n.md` / `tests/stdlib/queue_collections.n.md`
    - `[逶ｮ逧・繧ゅ￥縺ｦ縺江` 縺ｨ[遒ｺ隱榊・螳ｹ/縺九￥縺ｫ繧薙↑縺・ｈ縺・繧端譏手ｨ・繧√＞縺江縺励▽縺､縲～uwok` 繧端菴ｿ/縺､縺犠縺｣縺歇迴ｾ蝨ｨ/縺偵ｓ縺悶＞]縺ｮ[蛻ｩ逕ｨ蠖｢/繧翫ｈ縺・￠縺Ь縺ｫ蜷医ｏ縺帙※譖ｴ譁ｰ縲・
  - `todo.md`
    - `nodesrc/tests.js` 縺ｨ `nodesrc/run_test.js` 縺ｮ[菴ｿ/縺､縺犠縺Ъ蛻・繧従縺代ｒ[譁ｹ驥・縺ｻ縺・＠繧転縺ｸ霑ｽ蜉�縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build`
    - [邨先棡/縺代▲縺犠: success
  - `node nodesrc/tests.js -i tests/compiler/prelude_copy.n.md -i tests/stdlib/ringbuffer_collections.n.md -i tests/stdlib/queue_collections.n.md --no-stdlib --no-tree -o /tmp/tests-copy-ringbuffer-queue.json -j 15`
    - [邨先棡/縺代▲縺犠: `6/6 pass`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - `RegionToken<.T>` 繧・`RingBuffer<.T>` 繧・`Copy` 縺ｫ縺帙★縲ー蜀・Κ/縺ｪ縺・・]縺ｮ `MemPtr` / `i32` [谺・繧峨ｓ]縺�縺代ｒ[蜈・縺輔″]縺ｫ[蜿・縺ｨ]繧骸蜃ｺ/縺�]縺励※[菴ｿ/縺､縺犠縺・譁ｹ驥・縺ｻ縺・＠繧転縺ｸ[蟇・繧・縺帙◆縲・
  - 縺薙ｌ縺ｫ繧医ｊ `core/mem` 縺ｨ `alloc/collections` 縺ｮ[謇�譛画ｨｩ/縺励ｇ繧・≧縺代ｓ][蠅・阜/縺阪ｇ縺・°縺Ь縺啓迴ｾ迥ｶ/縺偵ｓ縺倥ｇ縺・縺ｮ[險�隱樊ｩ溯・/縺偵ｓ縺斐″縺ｮ縺・縺ｫ[蜿・縺翫＆]縺ｾ繧擬蠖｢/縺九◆縺｡]縺ｧ[螳牙ｮ・縺ゅｓ縺ｦ縺Ь縺励◆縲・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (stdlib doctest: `fn main` 譏守､ｺ縺ｨ copy 蛻､螳壹・蜑肴署菫ｮ豁｣)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - stdlib `.nepl` [蜀・縺ｪ縺Ь縺ｮ doctest 縺・`#entry main` 縺�縺代ｒ[謖・繧・縺｡縲～fn main` 繧端謖・繧・縺溘↑縺・◆繧√↓ `D3092` 縺ｧ[關ｽ/縺馨縺｡繧擬蝠城｡・繧ゅｓ縺�縺Ь繧端隗｣豸・縺九＞縺励ｇ縺・縺吶ｋ縲・
  - doctest [菫ｮ豁｣/縺励ｅ縺・○縺Ь繧端騾ｲ/縺吶☆]繧√ｋ[騾比ｸｭ/縺ｨ縺｡繧・≧]縺ｧ[髴ｲ蜃ｺ/繧阪＠繧・▽]縺励◆ compiler [蛛ｴ/縺後ｏ]縺ｮ `Copy` [蛻､螳・縺ｯ繧薙※縺Ь縺ｮ[荳肴紛蜷・縺ｵ縺帙＞縺斐≧]繧・菴ｵ/縺ゅｏ]縺帙※[菫ｮ豁｣/縺励ｅ縺・○縺Ь縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - stdlib 縺ｮ[譌｢蟄・縺阪◎繧転 doctest 縺ｯ `#entry main` 繧端譖ｸ/縺犠縺・※繧・`fn main` [譛ｬ菴・縺ｻ繧薙◆縺Ь繧端謖・繧・縺溘↑縺Ъ萓・繧後＞]縺啓螟・縺翫♀]縺上�¨ode [蛛ｴ/縺後ｏ]縺ｮ doctest [螳溯｡・縺倥▲縺薙≧][邨瑚ｷｯ/縺代＞繧江縺ｧ縺ｯ entry [谺�關ｽ/縺代▽繧峨￥]縺ｨ縺励※ `D3092` 縺ｫ[關ｽ/縺馨縺｡縺ｦ縺・◆縲・
  - 縺輔ｉ縺ｫ doctest [譛ｬ菴・縺ｻ繧薙◆縺Ь縺ｧ `assert_*` 縺ｮ繧医≧縺ｪ impure API 繧端蜻ｼ/繧・縺ｶ[蝣ｴ蜷・縺ｰ縺ゅ＞]縲｝ure `fn main <()->i32>` 繧端閾ｪ蜍・縺倥←縺・[謖ｿ蜈･/縺昴≧縺ｫ繧・≧]縺吶ｋ縺ｨ `D3025` 縺啓蜃ｺ/縺ｧ]繧九�・
  - `Copy` trait model 縺ｮ[螳溯｣・縺倥▲縺昴≧]縺ｧ縺ｯ `i64` / `i128` / `u128` / `f64` 繧・enum variant [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｧ[謇ｱ/縺ゅ▽縺犠縺｣縺ｦ縺翫ｊ縲ー螳滄圀/縺倥▲縺輔＞]縺ｫ縺ｯ `TypeKind::Named(...)` 縺ｧ[陦ｨ迴ｾ/縺ｲ繧・≧縺偵ｓ]縺輔ｌ繧擬蝙・縺九◆]縺ｨ縺ｮ[荳堺ｸ�閾ｴ/縺ｵ縺・▲縺｡]縺後≠縺｣縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - stdlib 縺ｮ doctest [蜈ｨ菴・縺懊ｓ縺溘＞]縺ｧ縲～fn main` 縺後↑縺Ъ萓・繧後＞]縺ｫ縺ｯ `fn main <()*>i32> ():` 繧端譏守､ｺ/繧√＞縺肋縺吶ｋ[譁ｹ蜷・縺ｻ縺・％縺・縺ｸ[蟇・繧・縺帙◆縲・
  - `nepl-core/src/types.rs`
    - trait model 縺ｮ `is_copy_with_trait_model` 縺ｧ `TypeKind::Named(name)` 繧端逕ｨ/繧ゅ■]縺・�～i64` / `i128` / `u64` / `u128` / `f64` 繧端豁｣/縺溘□]縺励￥ `Copy` impl [謗｢邏｢/縺溘ｓ縺輔￥]縺ｸ[豬・縺ｪ縺珪縺吶ｈ縺・↓[菫ｮ豁｣/縺励ｅ縺・○縺Ь縺励◆縲・
  - `stdlib/alloc/collections/stack.nepl`
    - doctest [蜀帝�ｭ/縺ｼ縺・→縺・繧・`fn main <()*>i32>` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[謠・縺昴ｍ]縺医◆縲・
    - [蠕悟・蜈亥・/縺ゅ→縺・ｌ縺輔″縺�縺余 縺ｮ ruby 繧端豁｣/縺溘□]縺励＞[隱ｭ/繧・縺ｿ縺ｸ[菫ｮ豁｣/縺励ｅ縺・○縺Ь縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build`
    - [邨先棡/縺代▲縺犠: success
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - doctest [蜈ｨ菴・縺懊ｓ縺溘＞]縺ｮ[荳�諡ｬ/縺・▲縺九▽][蜀榊ｮ溯｡・縺輔＞縺倥▲縺薙≧]縺ｯ縺ｾ縺�[騾比ｸｭ/縺ｨ縺｡繧・≧]縺ｧ縲～stack.nepl` 縺ｪ縺ｩ collections [蛛ｴ/縺後ｏ]繧端蜆ｪ蜈・繧・≧縺帙ｓ]縺励※[鬆・ｬ｡/縺倥ｅ繧薙§] focused 縺ｫ[遒ｺ隱・縺九￥縺ｫ繧転縺吶ｋ[谿ｵ髫・縺�繧薙°縺Ь縲・
  - [邁｡貎・縺九ｓ縺代▽]縺ｪ doctest [蟆ら畑/縺帙ｓ繧医≧][譫�邨・繧上￥縺疹縺ｿ縺ｮ[譁ｰ險ｭ/縺励ｓ縺帙▽]縺ｯ[菫晉蕗/縺ｻ繧翫ｅ縺・縺励�ー蠖馴擇/縺ｨ縺・ａ繧転縺ｯ `fn main` 繧端譏守､ｺ/繧√＞縺肋縺吶ｋ[譁ｹ驥・縺ｻ縺・＠繧転縺ｧ[騾ｲ/縺吶☆]繧√ｋ縲・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (stdlib 繝峨く繝･繝｡繝ｳ繝育函謌舌ヤ繝ｼ繝ｫ縺ｮ豎守畑蛹悶→逶ｮ谺｡讒矩��縺ｮ謨ｴ蛯・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - tutorials 縺ｨ stdlib 縺ｧ蜈ｱ騾壹・繝峨く繝･繝｡繝ｳ繝育函謌舌ヤ繝ｼ繝ｫ (`nodesrc/cli.js`) 繧剃ｽｿ逕ｨ縺ｧ縺阪ｋ繧医≧縺ｫ縺励�《tdlib 縺ｧ繧ゅう繝ｳ繧ｿ繝ｩ繧ｯ繝・ぅ繝悶↑繝励Ξ繧､繧ｰ繝ｩ繧ｦ繝ｳ繝我ｻ倥″ HTML 繧堤函謌仙庄閭ｽ縺ｫ縺吶ｋ縲・
  - stdlib 繝峨く繝･繝｡繝ｳ繝医・逶ｮ谺｡繧・`index.n.md` 縺ｧ邂｡逅・＠縲～00_` 縺ｪ縺ｩ縺ｮ繝励Μ繝輔ぅ繝・け繧ｹ縺ｫ萓晏ｭ倥＠縺ｪ縺・嚴螻､讒矩��繧偵し繝昴・繝医☆繧九�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nodesrc/cli.js`
    - `--site-name` 縺ｨ `--description-prefix` 蠑墓焚繧定ｿｽ蜉�縺励�√し繧､繝亥錐繧・ｪｬ譏取枚繧貞､夜Κ縺九ｉ謖・ｮ壼庄閭ｽ縺ｫ縺励◆縲・
    - `index.n.md` 繧貞━蜈育噪縺ｫ讀懷・縺励�∝・蜉帶凾縺ｫ `index.html` 縺ｸ繝槭ャ繝斐Φ繧ｰ縺吶ｋ繝ｭ繧ｸ繝・け繧定ｿｽ蜉�縲・
  - `stdlib/index.n.md`
    - 讓呎ｺ悶Λ繧､繝悶Λ繝ｪ縺ｮ譁ｰ縺励＞逶ｮ谺｡繝輔ぃ繧､繝ｫ縺ｨ縺励※菴懈・縲・
  - `.github/workflows/gh-pages.yml`
    - `stdlib` 縺ｮ繝薙Ν繝峨ｒ `html_play` 縺ｫ螟画峩縺励�・NEPLg2 Standard Library" 縺ｨ縺・≧繧ｵ繧､繝亥錐縺ｧ逕滓・縺吶ｋ繧医≧縺ｫ譖ｴ譁ｰ縲・
  - `stdlib/nm/README.n.md` -> `stdlib/nm/README.nepl`
    - 繝ｦ繝ｼ繧ｶ繝ｼ縺ｮ隕∵悍縺ｫ蝓ｺ縺･縺阪�√う繝ｳ繝・ャ繧ｯ繧ｹ莉･螟悶・ `.n.md` 繧・`.nepl` 蠖｢蠑擾ｼ医ラ繧ｭ繝･繝｡繝ｳ繝医さ繝｡繝ｳ繝井ｻ倥″・峨↓螟画鋤縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `nodesrc/cli.js` 縺ｮ蠑墓焚繝代・繧ｹ縺ｨ `index.n.md` 蜃ｦ逅・・繝ｭ繧ｸ繝・け縺梧ｭ｣蟶ｸ縺ｫ蜍穂ｽ懊＠縲～index.html` 縺梧悄蠕・�壹ｊ縺ｫ逕滓・縺輔ｌ繧九％縺ｨ繧堤｢ｺ隱阪�・

# 2026-03-09 菴懈･ｭ繝｡繝｢ (stdlib 繝峨く繝･繝｡繝ｳ繝医・逶ｮ谺｡髫主ｱ､蛹悶→繧ｿ繧､繝医Ν縺ｮ驕ｩ豁｣蛹・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `stdlib` 繝峨く繝･繝｡繝ｳ繝医・逶ｮ谺｡ (TOC) 縺悟ｹｳ蝮ｦ縺ｪ繝ｪ繧ｹ繝医↓縺ｪ縺｣縺ｦ縺・◆縺ｮ繧偵�√ョ繧｣繝ｬ繧ｯ繝医Μ讒矩��縺ｫ蝓ｺ縺･縺・◆髫主ｱ､逧・↑陦ｨ遉ｺ縺ｫ謾ｹ蝟・☆繧九�・
  - 繧ｵ繧､繝亥錐縺ｫ蠢懊§縺ｦ逶ｮ谺｡縺ｮ繧ｿ繧､繝医Ν ("Getting Started" 縺ｾ縺溘・ "Contents") 繧定・蜍慕噪縺ｫ蛻・ｊ譖ｿ縺医ｉ繧後ｋ繧医≧縺ｫ縺励�√ラ繧ｭ繝･繝｡繝ｳ繝医・遞ｮ鬘槭↓驕ｩ縺励◆陦ｨ遉ｺ縺ｫ縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `nodesrc/cli.js` 縺ｮ `buildTocEntries` 縺ｫ縺翫＞縺ｦ縲∵・遉ｺ逧・↑繧､繝ｳ繝・ャ繧ｯ繧ｹ縺ｫ蜷ｫ縺ｾ繧後↑縺・�梧ｮ九ｊ縲阪・繝輔ぃ繧､繝ｫ縺御ｸ�蠕・"Other" 繧ｰ繝ｫ繝ｼ繝励↓繝輔Λ繝・ヨ縺ｫ蜈･繧後ｉ繧後※縺・◆縲・
  - `nodesrc/html_gen_playground.js` 縺ｮ逶ｮ谺｡繧ｿ繧､繝医Ν縺・"Getting Started" 縺ｫ繝上・繝峨さ繝ｼ繝峨＆繧後※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nodesrc/cli.js`
    - `buildTocEntries` 繧剃ｿｮ豁｣縺励�∵ｮ九ｊ縺ｮ繝輔ぃ繧､繝ｫ繧貞・騾壹・繝・ぅ繝ｬ繧ｯ繝医Μ謗･鬆ｭ霎槭〒繧ｰ繝ｫ繝ｼ繝怜喧縺吶ｋ髫主ｱ､蛹悶Ο繧ｸ繝・け繧貞ｮ溯｣・�・
    - `siteName` 縺ｫ "tutorial" 縺悟性縺ｾ繧後↑縺・�ｴ蜷医・逶ｮ谺｡繧ｿ繧､繝医Ν繧・"Contents" 縺ｨ蛻､螳壹＠縲∫函謌仙・逅・↓貂｡縺吶ｈ縺・↓螟画峩縲・
  - `nodesrc/html_gen_playground.js`
    - `renderToc` 縺ｨ `renderHtmlPlayground` 繧呈峩譁ｰ縺励�～tocTitle` 繧ｪ繝励す繝ｧ繝ｳ繧貞女縺大叙繧翫�・Getting Started" 莉･螟悶・繧ｿ繧､繝医Ν繧り｡ｨ遉ｺ縺ｧ縺阪ｋ繧医≧縺ｫ螟画峩縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `dist/doc/stdlib/alloc/diag/diag.html` 縺ｪ縺ｩ繧堤｢ｺ隱阪＠縲∫岼谺｡繧ｿ繧､繝医Ν縺・"Contents" 縺ｫ縺ｪ繧翫�～alloc/collections` 繧・`core/traits` 縺ｪ縺ｩ縺ｮ繝・ぅ繝ｬ繧ｯ繝医Μ蜊倅ｽ阪〒髫主ｱ､蛹悶＆繧後※縺・ｋ縺薙→繧堤｢ｺ隱阪�・
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - 讓呎ｺ悶Λ繧､繝悶Λ繝ｪ縺ｮ繝峨く繝･繝｡繝ｳ繝医′縲√メ繝･繝ｼ繝医Μ繧｢繝ｫ縺ｨ蜷檎ｭ峨・謨ｴ逅・＆繧後◆讒矩��縺ｧ髢ｲ隕ｧ蜿ｯ閭ｽ縺ｫ縺ｪ縺｣縺溘�・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (doctest main 霑ｽ蠕灘ｾ後・ collections / nm / fs 謨ｴ蜷域�ｧ菫ｮ豁｣)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - stdlib doctest 縺ｫ `fn main <()*>i32>` 繧端譏守､ｺ/繧√＞縺肋縺励◆縺ゅ→縺ｫ[髴ｲ蜃ｺ/繧阪＠繧・▽]縺励◆縲…ollections / kp / nm / fs [蛛ｴ/縺後ｏ]縺ｮ[謨ｴ蜷域�ｧ/縺帙＞縺斐≧縺帙＞][蟠ｩ/縺上★]繧後ｒ[譬ｹ譛ｬ/縺薙ｓ縺ｽ繧転縺九ｉ[逶ｴ/縺ｪ縺馨縺吶�・
  - 縺ｨ縺上↓ `Vec.data` 縺ｮ `MemPtr` 蛹悶↓[霑ｽ蠕・縺､縺・§繧・≧]縺励※縺・↑縺Ъ邂・園/縺九＠繧Ⅹ縺ｨ縲～stack_free` 縺ｮ impure / pure [荳堺ｸ�閾ｴ/縺ｵ縺・▲縺｡]繧端蜈・縺輔″]縺ｫ[隗｣豸・縺九＞縺励ｇ縺・縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `Vec<.T>.data` 繧・`MemPtr<.T>` 縺ｫ[遘ｻ陦・縺・％縺・縺励◆縺ゅ→繧ゅ�‥octest 繧・ｸ�驛ｨ縺ｮ nm / fs [螳溯｣・縺倥▲縺昴≧]縺・raw `i32` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｮ `get ... "data"` 繧端谿・縺ｮ縺転縺励※縺・◆縲・
  - `stack_free` 縺ｯ `dealloc_ptr` 繧端蜻ｼ/繧・縺ｶ縺ｮ縺ｫ pure [鄂ｲ蜷・縺励ｇ繧√＞]縺ｮ縺ｾ縺ｾ縺�縺｣縺溘◆繧√�‥octest 繧端騾・縺ｨ縺馨縺兌驕守ｨ・縺九※縺Ь縺ｧ impure API [謨ｴ蜷域�ｧ/縺帙＞縺斐≧縺帙＞]縺ｮ[遐ｴ邯ｻ/縺ｯ縺溘ｓ]縺啓陦ｨ髱｢蛹・縺ｲ繧・≧繧√ｓ縺犠縺励◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/collections/stack.nepl`
    - `stack_free` 繧・`fn stack_free <.T> <(Stack<.T>)*>()>` 縺ｫ[菫ｮ豁｣/縺励ｅ縺・○縺Ь縺励�～dealloc_ptr` 繧端蜻ｼ/繧・縺ｶ[螳滉ｽ・縺倥▲縺溘＞]縺ｨ[鄂ｲ蜷・縺励ｇ繧√＞]繧端荳�閾ｴ/縺・▲縺｡]縺輔○縺溘�・
    - `uwok dealloc_ptr ...` 縺ｮ[陦梧忰/縺弱ｇ縺・∪縺､] `;` 繧端螟・縺ｯ縺咯縺励�ー蠑・縺励″]縺ｨ縺励※[邏�逶ｴ/縺吶↑縺馨縺ｫ[豸郁ｲｻ/縺励ｇ縺・・]縺吶ｋ[蠖｢/縺九◆縺｡]縺ｸ[謠・縺昴ｍ]縺医◆縲・
  - `stdlib/kp/kpgraph.nepl`
    - doctest 縺ｮ `dist.data` [蜿ら・/縺輔ｓ縺励ｇ縺・繧・`mem_ptr_addr get dist "data"` 縺ｸ[菫ｮ豁｣/縺励ｅ縺・○縺Ь縺励◆縲・
  - `stdlib/std/fs.nepl`
    - `Vec<u8>` 縺ｮ[蜀・Κ/縺ｪ縺・・][鬆伜沺/繧翫ｇ縺・＞縺江繧・raw `i32` 縺ｨ縺励※[隱ｭ/繧・繧薙〒縺・◆[邂・園/縺九＠繧Ⅹ繧・`mem_ptr_addr buf.data` 縺ｸ[菫ｮ豁｣/縺励ｅ縺・○縺Ь縺励◆縲・
  - `stdlib/nm/parser.nepl` / `stdlib/nm/html_gen.nepl`
    - `Vec<...>.data` 繧・raw `i32` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｧ[隱ｭ/繧・繧薙〒縺・◆[邂・園/縺九＠繧Ⅹ繧偵�～mem_ptr_addr get ... "data"` 縺ｸ[讖滓｢ｰ逧・縺阪°縺・※縺江縺ｫ[霑ｽ蠕・縺､縺・§繧・≧]縺輔○縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build`
    - [邨先棡/縺代▲縺犠: success
  - `node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl --no-tree -o /tmp/tests-stack-doctest-v3.json -j 15`
    - [迥ｶ豕・縺倥ｇ縺・″繧・≧]: 縺薙・[迺ｰ蠅・縺九ｓ縺阪ｇ縺・縺ｧ縺ｯ JSON [蜃ｺ蜉・縺励ｅ縺､繧翫ｇ縺従縺ｾ縺ｧ[譎る俣/縺倥°繧転縺後°縺九ｋ縺溘ａ縲’ocused [螳溯｡・縺倥▲縺薙≧]縺ｮ[螳御ｺ・縺九ｓ繧翫ｇ縺・[遒ｺ隱・縺九￥縺ｫ繧転繧端邯咏ｶ壻ｸｭ/縺代＞縺槭￥縺｡繧・≧]縲・
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - 縺薙％縺ｧ縺ｮ[菫ｮ豁｣/縺励ｅ縺・○縺Ь縺ｯ縲‥octest 繧端騾・縺ｨ縺馨縺吶◆繧√・[蝣ｴ蠖・縺ｰ縺・縺溘ｊ[蟇ｾ蠢・縺溘＞縺翫≧]縺ｧ縺ｯ縺ｪ縺上�～Vec.data` 縺ｮ `MemPtr` 蛹悶→ impure [鄂ｲ蜷・縺励ｇ繧√＞]縺ｮ[謨ｴ蜷域�ｧ/縺帙＞縺斐≧縺帙＞]繧端蝗槫ｾｩ/縺九＞縺ｵ縺従縺吶ｋ繧ゅ・縲・
  - 谺｡縺ｯ `nodesrc` [蛛ｴ/縺後ｏ]縺ｮ doctest focused [螳溯｡・縺倥▲縺薙≧][邨瑚ｷｯ/縺代＞繧江繧端螳牙ｮ壼喧/縺ゅｓ縺ｦ縺・°]縺励�∵里蟄・stdlib doctest 繧端鬆・ｬ｡/縺倥ｅ繧薙§][騾夐℃/縺､縺・°]縺輔○繧九�・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (nodesrc: doctest 1 莉ｶ focused 螳溯｡後・霑ｽ蜉�)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `nodesrc/tests.js` 縺ｮ[髮・ｴ・縺励ｅ縺・ｄ縺従[螳溯｡・縺倥▲縺薙≧]繧端蠕・縺ｾ]縺溘★縺ｫ縲《tdlib reboot [荳ｭ/縺｡繧・≧]縺ｮ doctest 1 莉ｶ繧端逶ｴ謗･/縺｡繧・￥縺帙▽][蜀咲樟/縺輔＞縺偵ｓ]縺ｧ縺阪ｋ[蜈･蜿｣/縺・ｊ縺舌■]繧端霑ｽ蜉�/縺､縺・°]縺吶ｋ縲・
  - `stack.nepl` 縺ｮ繧医≧縺ｫ[迚ｹ螳・縺ｨ縺上※縺Ь file 縺ｮ doctest 繧端鬆・分/縺倥ｅ繧薙・繧転縺ｫ[貎ｰ/縺､縺ｶ]縺励◆縺Ъ蝣ｴ髱｢/縺ｰ繧√ｓ]縺ｧ縲～run_test.js` 蜷代￠ JSON 繧端謇区嶌/縺ｦ縺珪縺阪○縺壹↓[遒ｺ隱・縺九￥縺ｫ繧転縺ｧ縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `nodesrc/tests.js` 縺ｯ doctest [蜈ｨ菴・縺懊ｓ縺溘＞]縺ｮ[髮・ｴ・縺励ｅ縺・ｄ縺従縺ｫ縺ｯ[蜷・繧�]縺上′縲《tdlib reboot [荳ｭ/縺｡繧・≧]縺ｮ[螻�謇�逧・縺阪ｇ縺上＠繧・※縺江縺ｪ[蜴溷屏/縺偵ｓ縺・ｓ][蛻・縺江繧骸蛻・繧従縺代↓縺ｯ[驥・縺翫ｂ]縺・�・
  - `nodesrc/run_test.js` 縺ｯ 1 莉ｶ[螳溯｡・縺倥▲縺薙≧]縺ｮ[譬ｸ/縺九￥]繧端謖・繧・縺､縺後�’ile / doctest index 縺九ｉ[逶ｴ謗･/縺｡繧・￥縺帙▽][蜻ｼ/繧・縺ｶ[阮・縺・☆]縺・CLI 縺後↑縺九▲縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nodesrc/run_doctest.js`
    - `parseFile` 縺ｧ file [荳ｭ/縺｡繧・≧]縺ｮ doctest 繧端隱ｭ/繧・縺ｿ縲～-n` 縺ｧ[謖・ｮ・縺励※縺Ь縺励◆ 1 莉ｶ縺�縺代ｒ `runSingle` 縺ｫ[豬・縺ｪ縺珪縺・CLI 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `compile_fail` 縺ｮ `diag_id` / `diag_span` [遒ｺ隱・縺九￥縺ｫ繧転繧・`tests.js` 縺ｨ[蜷・縺翫↑]縺麓蝓ｺ貅・縺阪§繧・ｓ]縺ｧ[驕ｩ逕ｨ/縺ｦ縺阪ｈ縺・縺吶ｋ縲・
  - `todo.md`
    - stdlib reboot [荳ｭ/縺｡繧・≧]縺ｮ focused doctest [螳溯｡・縺倥▲縺薙≧]縺ｧ縺ｯ `node nodesrc/run_doctest.js -i <file> -n <index>` 繧端菴ｿ/縺､縺犠縺・譁ｹ驥・縺ｻ縺・＠繧転繧端霑ｽ險・縺､縺・″]縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/stack.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/core/traits/deserialize.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - 譌｢蟄・stdlib doctest 繧端鬆・ｬ｡/縺倥ｅ繧薙§][騾・縺ｨ縺馨縺兌髫・縺輔＞]縺ｮ[蜈･蜿｣/縺・ｊ縺舌■]縺啓謠・縺昴ｍ]縺・�～tests.js` 縺ｮ[驥・縺翫ｂ]縺Ъ髮・ｴ・縺励ｅ縺・ｄ縺従[螳溯｡・縺倥▲縺薙≧]縺ｫ[鬆ｼ/縺溘ｈ]繧峨★縺ｫ[螻�謇�/縺阪ｇ縺上＠繧Ⅹ[遒ｺ隱・縺九￥縺ｫ繧転縺ｧ縺阪ｋ繧医≧縺ｫ縺ｪ縺｣縺溘�・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`nodesrc/README.md` 縺ｮ霑ｽ蜉�縺ｨ doctest 螳溯｡檎ｵ瑚ｷｯ縺ｮ謨ｴ逅・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `nodesrc/` [驟堺ｸ・縺ｯ縺・°]縺ｮ[驕灘・/縺ｩ縺・＄]縺啓蠅・縺ｵ]縺医※縺阪◆縺溘ａ縲《tdlib reboot [荳ｭ/縺｡繧・≧]縺ｫ縲後←縺ｮ[逶ｮ逧・繧ゅ￥縺ｦ縺江縺ｧ縺ｩ縺ｮ script 繧端菴ｿ/縺､縺犠縺・°縲阪ｒ 1 [譫・縺ｾ縺Ь縺ｧ[遒ｺ隱・縺九￥縺ｫ繧転縺ｧ縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
  - doctest / 騾壼ｸｸ tests / 隗｣譫・/ HTML [逕滓・/縺帙＞縺帙＞]縺ｮ[蜈･蜿｣/縺・ｊ縺舌■]繧端譏守｢ｺ/繧√＞縺九￥]縺ｫ縺励�～todo.md` 縺ｮ[驕狗畑/縺・ｓ繧医≧][譁ｹ驥・縺ｻ縺・＠繧転縺ｨ[荳�閾ｴ/縺・▲縺｡]縺輔○繧九�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nodesrc/README.md`
    - `tests.js` / `run_doctest.js` / `run_test.js` / `analyze_source.js` / `analyze_tests_json.js` / `cli.js` 縺ｮ[菴ｿ/縺､縺犠縺Ъ蛻・繧従縺代ｒ縲ー逶ｮ逧・挨/繧ゅ￥縺ｦ縺阪∋縺､]縺ｫ[謨ｴ逅・縺帙＞繧馨縺励◆縲・
    - stdlib reboot [荳ｭ/縺｡繧・≧]縺ｫ繧医￥[菴ｿ/縺､縺犠縺・謇矩�・縺ｦ縺倥ｅ繧転縺ｨ縺励※縲‥octest 1 莉ｶ縺ｮ[菫ｮ豁｣/縺励ｅ縺・○縺Ь縲…ompiler [荳榊・蜷・縺ｵ縺舌≠縺Ь縺ｮ[蛻・縺江繧骸蛻・繧従縺代�・�壼ｸｸ tests 縺ｨ doctest 縺ｮ[蛻・屬/縺ｶ繧薙ｊ][遒ｺ隱・縺九￥縺ｫ繧転繧端險倩ｿｰ/縺阪§繧・▽]縺励◆縲・
  - `todo.md`
    - `run_doctest.js` 繧端菴ｿ/縺､縺犠縺｣縺・focused doctest [螳溯｡・縺倥▲縺薙≧]繧端讓呎ｺ・縺ｲ繧・≧縺倥ｅ繧転縺ｮ[驕狗畑/縺・ｓ繧医≧]縺ｨ縺励※[霑ｽ險・縺､縺・″]縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/stack.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/core/traits/deserialize.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/cliarg.n.md -n 3`
    - [邨先棡/縺代▲縺犠: `compile_fail` + `diag_id: D3006` [遒ｺ隱・縺九￥縺ｫ繧転 pass
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - stdlib reboot [荳ｭ/縺｡繧・≧]縺ｮ doctest [菫ｮ豁｣/縺励ｅ縺・○縺Ь縺ｯ縲√∪縺・`run_doctest.js` 縺ｧ 1 莉ｶ繧端蝗ｺ/縺九◆]繧√�√◎縺ｮ縺ゅ→ `tests.js` 縺ｧ[蟆・縺｡縺Ь縺輔＞[遽・峇/縺ｯ繧薙＞]繧端髮・ｴ・縺励ｅ縺・ｄ縺従[遒ｺ隱・縺九￥縺ｫ繧転縺吶ｋ[豬・縺ｪ縺珪繧後〒[騾ｲ/縺吶☆]繧√ｉ繧後ｋ繧医≧縺ｫ縺ｪ縺｣縺溘�・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`kpread` / `kpwrite` 縺ｮ[謇�譛画ｨｩ/縺励ｇ繧・≧縺代ｓ][謨ｴ逅・縺帙＞繧馨縺ｨ doctest [蝗槫ｾｩ/縺九＞縺ｵ縺従)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `kpwrite` 縺ｮ stdout 縺啓遨ｺ/縺九ｉ]縺ｫ縺ｪ繧・doctest [荳榊・蜷・縺ｵ縺舌≠縺Ь縺ｨ縲～kpread` / `kpgraph` 縺・`Vec.data` 縺ｮ `MemPtr` 蛹悶↓[霑ｽ蠕・縺､縺・§繧・≧]縺励″繧後※縺・↑縺Ъ荳肴紛蜷・縺ｵ縺帙＞縺斐≧]繧偵�ー謇�譛画ｨｩ/縺励ｇ繧・≧縺代ｓ]縺ｮ[險ｭ險・縺帙▲縺代＞]縺九ｉ[逶ｴ/縺ｪ縺馨縺吶�・
  - `kp` [邉ｻ/縺代＞] helper 縺ｮ[螳溯｣・縺倥▲縺昴≧]繧端譁ｰ/縺ゅ◆繧云縺励＞ doc comment policy 縺ｫ[蜷・縺・繧上○縲ー菫ｮ豁｣/縺励ｅ縺・○縺Ь縺励◆ API 縺ｮ[諢丞袖/縺・∩]縺啓蝙・縺九◆]縺ｨ繧ｳ繝｡繝ｳ繝医・[荳｡譁ｹ/繧翫ｇ縺・⊇縺・縺ｧ[蛻・繧従縺九ｋ繧医≧縺ｫ縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `Writer` 縺ｯ header [鬆伜沺/繧翫ｇ縺・＞縺江縺�縺代ｒ[蜈ｱ譛・縺阪ｇ縺・ｆ縺・縺吶ｌ縺ｰ繧医＞縺ｮ縺ｫ `RegionToken<u8>` 繧・field 縺ｫ[菫晄戟/縺ｻ縺肋縺励※縺翫ｊ縲”eader [蜿ら・/縺輔ｓ縺励ｇ縺・縺ｮ縺溘・縺ｫ `region_ptr get w "region"` 繧端邨檎罰/縺代＞繧・縺励※縺・◆縲ゅ％縺ｮ[險ｭ險・縺帙▲縺代＞]縺�縺ｨ[謇�譛画ｨｩ/縺励ｇ繧・≧縺代ｓ]繧端謖・繧・縺､ token 縺ｨ[霆ｽ驥・縺代＞繧翫ｇ縺・ handle 縺ｮ[雋ｬ蜍・縺帙″繧�]縺啓豺ｷ/縺ｾ]縺悶ｊ縲‥octest [螳溯｡・縺倥▲縺薙≧]縺ｧ[迥ｶ諷・縺倥ｇ縺・◆縺Ь縺啓螢・縺薙ｏ]繧後ｄ縺吶°縺｣縺溘�・
  - `Scanner` 繧ょ酔讒倥↓ `RegionToken<u8>` 繧・field 縺ｫ[謖・繧・縺｣縺ｦ縺・◆縺溘ａ縲ー隱ｭ/繧・縺ｿ[蜿・縺ｨ]繧骸菴咲ｽｮ/縺・■]縺�縺代ｒ[蜈ｱ譛・縺阪ｇ縺・ｆ縺・縺励◆縺・helper [鄒､/縺舌ｓ]縺啓豈主屓/縺ｾ縺・°縺Ь token 繧端豸郁ｲｻ/縺励ｇ縺・・]縺吶ｋ[蠖｢/縺九◆縺｡]縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
  - `kpread_core` 縺ｯ header [鬆伜沺/繧翫ｇ縺・＞縺江繧端隗ｦ/縺輔ｏ]繧九□縺代・ helper 縺ｫ繧・`RegionToken<u8>` 繧端隕∵ｱ・繧医≧縺阪ｅ縺・縺励※縺翫ｊ縲ー蜀・Κ/縺ｪ縺・・][螳溯｣・縺倥▲縺昴≧]縺啓荳崎ｦ・縺ｵ繧医≧]縺ｫ[驥・縺翫ｂ]縺九▲縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/kp/kpwrite.nepl`
    - `Writer.region` 繧・`Writer.header <MemPtr<u8>>` 縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - header [謫堺ｽ・縺昴≧縺評 helper 縺ｯ `MemPtr<u8>` 繧端逶ｴ謗･/縺｡繧・￥縺帙▽][蜿・縺・縺代ｋ繧医≧縺ｫ縺励�～writer_free_handle` / `writer_flush_handle` / `writer_ensure_handle` / `writer_put_u8_handle` / `writer_write_str_handle` / `writer_write_i32_handle` / `writer_write_u64_handle` 縺ｮ[蜿ら・/縺輔ｓ縺励ｇ縺・繧偵☆縺ｹ縺ｦ[霑ｽ蠕・縺､縺・§繧・≧]縺輔○縺溘�・
    - file header 縺ｨ `Writer` struct 縺ｮ doc comment 繧端譁ｰ/縺ゅ◆繧云縺励＞ policy 縺ｫ[謠・縺昴ｍ]縺医◆縲・
  - `stdlib/kp/kpread.nepl`
    - `Scanner.region` 繧・`Scanner.header <MemPtr<u8>>` 縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `Scanner` 縺ｯ[蜈･蜉・縺ｫ繧・≧繧翫ｇ縺従[迥ｶ諷・縺倥ｇ縺・◆縺Ь繧端蜈ｱ譛・縺阪ｇ縺・ｆ縺・縺吶ｋ[霆ｽ驥・縺代＞繧翫ｇ縺・ handle 縺ｪ縺ｮ縺ｧ縲～Copy` / `Clone` 繧端譏守､ｺ逧・繧√＞縺倥※縺江縺ｫ[螳溯｣・縺倥▲縺昴≧]縺励◆縲・
    - `scanner_header_ptr` / `scanner_load_header` / `scanner_store_header` 繧・header pointer [蝓ｺ貅・縺阪§繧・ｓ]縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `scanner_skip_ws_header` 繧端霑ｽ蜉�/縺､縺・°]縺励�∝推 helper 縺・`let header <MemPtr<u8>> get sc "header";` 縺ｧ[蜈・縺輔″]縺ｫ header 繧端譚溽ｸ・縺昴￥縺ｰ縺従縺励※縺九ｉ[蜃ｦ逅・縺励ｇ繧馨縺吶ｋ[蠖｢/縺九◆縺｡]縺ｸ[謠・縺昴ｍ]縺医◆縲・
    - file header 縺ｨ `Scanner` struct 縺ｮ doc comment 繧端譁ｰ/縺ゅ◆繧云縺励＞ policy 縺ｫ[謠・縺昴ｍ]縺医◆縲・
  - `stdlib/kp/kpread_core.nepl`
    - `mem_i32_region_ptr` / `store_i32_u8_at` / `load_i32_u8_at` 繧・`MemPtr<u8>` + size [蝓ｺ貅・縺阪§繧・ｓ]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `scanner_new_impl` 縺ｮ header [蛻晄悄蛹・縺励ｇ縺阪°]縺ｧ[荳�譎ら噪/縺・■縺倥※縺江縺ｪ `RegionToken<u8>` 繧端菴・縺､縺従繧峨★縲〉aw header pointer 縺ｨ size 繧端逶ｴ謗･/縺｡繧・￥縺帙▽][貂｡/繧上◆]縺兌蠖｢/縺九◆縺｡]縺ｸ[謨ｴ逅・縺帙＞繧馨縺励◆縲・
    - file header 縺ｮ doc comment 繧端譁ｰ/縺ゅ◆繧云縺励＞ policy 縺ｫ[謠・縺昴ｍ]縺医◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/kp/kpwrite.nepl -n 2`
    - [邨先棡/縺代▲縺犠: pass, stdout=`1 2\n`
  - `node nodesrc/run_doctest.js -i stdlib/kp/kpwrite.nepl -n 3`
    - [邨先棡/縺代▲縺犠: pass, stdout=`123\n`
  - `node nodesrc/run_doctest.js -i stdlib/kp/kpwrite.nepl -n 4`
    - [邨先棡/縺代▲縺犠: pass, stdout=`42\n`
  - `node nodesrc/run_doctest.js -i stdlib/kp/kpgraph.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass, stdout=`0 1 2 3\n`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - `kpwrite` / `kpread` 縺ｮ[菫ｮ豁｣/縺励ｅ縺・○縺Ь縺ｯ doctest 繧端騾・縺ｨ縺馨縺吶◆繧√・[蝣ｴ蠖・縺ｰ縺・縺溘ｊ[蟇ｾ蠢・縺溘＞縺翫≧]縺ｧ縺ｯ縺ｪ縺上�”eader pointer 縺ｨ[謇�譛画ｨｩ/縺励ｇ繧・≧縺代ｓ] token 縺ｮ[雋ｬ蜍・縺帙″繧�][蛻・屬/縺ｶ繧薙ｊ]繧端蝗槫ｾｩ/縺九＞縺ｵ縺従縺吶ｋ繧ゅ・縲・
  - `kpread.nepl` 縺ｫ縺ｯ[螳溯｡悟ｯｾ雎｡/縺倥▲縺薙≧縺溘＞縺励ｇ縺・縺ｮ doctest 縺ｯ縺ｾ縺�縺ｪ縺・`skip` 縺ｮ縺ｿ縺�縺後�～scanner_read_i32` 繧端菴ｿ/縺､縺犠縺・怙蟆・source test 縺ｨ `kpgraph` 縺ｮ doctest 縺ｧ[迴ｾ陦・縺偵ｓ縺薙≧]險ｭ險医′[謌千ｫ・縺帙＞繧翫▽]縺吶ｋ縺薙→繧端遒ｺ隱・縺九￥縺ｫ繧転縺励◆縲・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`queue` 縺ｮ doc comment 謨ｴ蛯吶→ `uwok` 縺ｸ縺ｮ蟇・○)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `Queue` 縺ｮ蜈ｬ髢・API 繧ｳ繝｡繝ｳ繝医ｒ迴ｾ陦後・ doc comment policy 縺ｫ蜷医ｏ縺帙�～RingBuffer` 繝吶・繧ｹ縺ｮ queue 縺ｧ縺ゅｋ縺薙→縲∵峩譁ｰ蠕後・蛟､繧定ｿ斐☆ API 縺ｧ縺ゅｋ縺薙→縲～Option` / `Result` 縺ｮ謇ｱ縺・ｒ繧ｳ繝｡繝ｳ繝医□縺代〒霑ｽ縺医ｋ繧医≧縺ｫ縺吶ｋ縲・
  - collection 邉ｻ縺ｮ focused test 繧・`uwok` 蜑肴署縺ｮ遏ｭ縺・pipe 險俶ｳ輔∈蟇・○縲《tdlib reboot 蠕後・蜈ｸ蝙狗噪縺ｪ菴ｿ縺・婿繧偵ユ繧ｹ繝亥・縺ｧ繧ょ崋螳壹☆繧九�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/collections/queue.nepl`
    - file header, `Queue` struct, `queue_new`, `queue_with_capacity`, `queue_len`, `queue_is_empty`, `queue_push`, `queue_pop`, `queue_peek`, `queue_clear`, `queue_free` 縺ｮ doc comment 繧堤樟陦・policy 縺ｫ豐ｿ縺｣縺ｦ譖ｸ縺咲峩縺励◆縲・
    - `queue_push` 縺梧峩譁ｰ蠕後・ queue 繧定ｿ斐☆ API 縺ｧ縺ゅｊ縲｝ipe 險俶ｳ輔〒縺ｯ `|> queue_push ... |> uwok` 縺ｮ蠖｢縺ｧ譚溽ｸ帙＠逶ｴ縺吝ｿ・ｦ√′縺ゅｋ縺薙→繧呈・險倥＠縺溘�・
  - `tests/stdlib/ringbuffer_collections.n.md`
    - `unwrap_ok<...>` 繧・`uwok` 縺ｫ鄂ｮ縺肴鋤縺医◆縲・
    - `ringbuffer_push_back` / `ringbuffer_pop_front` 縺ｮ蝙句ｼ墓焚繧堤怐縺阪�∫樟陦後・蝙区耳隲悶〒騾壹ｋ譖ｸ縺肴婿縺ｸ蟇・○縺溘�・
  - `tests/stdlib/pipe_collections.n.md`
    - `RingBuffer` / `Queue` 縺ｮ pipe 菴ｿ逕ｨ萓九ｒ `uwok` 繝吶・繧ｹ縺ｮ遏ｭ縺・嶌縺肴婿縺ｫ蟇・○縺溘�・
    - `queue_push<i32>` / `ringbuffer_push_back<i32>` 縺ｪ縺ｩ縲∽ｸ崎ｦ√↑蝙句ｼ墓焚繧貞､悶＠縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/tests.js -i tests/stdlib/ringbuffer_collections.n.md -i tests/stdlib/pipe_collections.n.md --no-stdlib --no-tree -o /tmp/tests-queue-ringbuffer-uwok.json -j 15`
    - [邨先棡/縺代▲縺犠: `9/9 pass`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - `Queue` / `RingBuffer` 縺ｮ蛻ｩ逕ｨ萓九・ `uwok` 繧剃ｽｿ縺｣縺溽洒縺・pipe 蠖｢縺ｧ螳牙ｮ壹＠縺ｦ譖ｸ縺代ｋ迥ｶ諷九↓縺ｪ縺｣縺溘�・
  - `queue.nepl` 縺ｯ蜀・ｮｹ縺�縺代〒縺ｪ縺上�∬ｦ句・縺鈴嚴螻､縺ｨ遽�讒区・繧ら樟陦後・ doc comment policy 縺ｫ豐ｿ縺・ｽ｢縺ｸ譖ｴ譁ｰ縺励◆縲・
# 2026-03-10 菴懈･ｭ繝｡繝｢ (vec_data_len 繧・`.Pair` 縺九ｉ explicit struct 縺ｸ遘ｻ陦・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `tests/stdlib/sort.n.md::doctest#3` 縺ｮ `use of moved value: s` 繧偵�～.Pair` [霑泌唆/縺ｸ繧薙″繧・￥]縺ｫ[萓晏ｭ・縺・◎繧転縺励◆ API [險ｭ險・縺帙▲縺代＞]縺九ｉ[隗｣豸・縺九＞縺励ｇ縺・縺吶ｋ縲・
  - `Vec` [邉ｻ/縺代＞]縺ｮ doc comment 繧端迴ｾ陦・縺偵ｓ縺薙≧] policy 縺ｫ[蜷・縺・繧上○繧九�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/collections/vec.nepl`
    - `VecDataLen<.T>` struct 繧端霑ｽ蜉�/縺､縺・°]縲・
    - `vec_data_len` 縺ｮ[霑・縺九∴]繧骸蛟､/縺ゅ◆縺Ь繧・`.Pair` 縺九ｉ `VecDataLen<.T>` 縺ｫ[螟画峩/縺ｸ繧薙％縺・縲・
    - `vec_data_len` 縺ｮ doc comment 繧・`##` / `###` 縺ｨ `[逶ｮ逧・繧ゅ￥縺ｦ縺江` / `[菴ｿ逕ｨ萓・縺励ｈ縺・ｌ縺Ь` / `[螳溯｣・縺倥▲縺昴≧]` / `[豕ｨ諢・縺｡繧・≧縺Ь` / `[險育ｮ鈴㍼/縺代＞縺輔ｓ繧翫ｇ縺・` [讒区・/縺薙≧縺帙＞]縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縲・
  - `tests/stdlib/sort.n.md`
    - `get s 0` / `get s 1` 繧・`get s "data"` / `get s "len"` 縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縲・
    - `data` 縺ｯ `MemPtr<.T>` 縺ｪ縺ｮ縺ｧ `mem_ptr_addr` 繧端騾・縺ｨ縺馨縺兌蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縲・
  - `nodesrc/README.n.md`
    - `tests.js` / `run_doctest.js` / `run_test.js` / `cli.js` / `compiler_loader.js` 縺ｮ[逶ｮ逧・挨/繧ゅ￥縺ｦ縺阪∋縺､][菴ｿ/縺､縺犠縺Ъ蛻・繧従縺代ｒ[霑ｽ蜉�/縺､縺・°]縲・
- [逅・罰/繧翫ｆ縺・:
  - `.Pair` 縺ｯ generic [髢｢謨ｰ/縺九ｓ縺吶≧][霑・縺ｸ繧転繧骸蛟､/縺ゅ◆縺Ь縺ｨ field `get` 縺ｮ[邨・縺従縺ｿ[蜷・縺・繧上○縺ｧ move-check 縺ｮ[謠ｺ/繧・繧後ｒ[襍ｷ/縺馨縺薙＠繧・☆縺・�・
  - `VecDataLen<.T>` 縺ｮ繧医≧縺ｪ[譏守､ｺ逧・繧√＞縺倥※縺江 struct 縺ｫ[鄂ｮ/縺馨縺梗謠・縺犠縺医ｋ縺ｨ縲’ield [蜷・繧√＞]繝ｻdoc comment繝ｻtests 縺ｮ[諢丞袖/縺・∩]縺啓謠・縺昴ｍ]縺・�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 3` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 4` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 12` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 13` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 9` -> pass

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`move_effect` 縺ｮ reboot 霑ｽ蠕薙→ prelude 陦晉ｪ√・蛻・ｊ蛻・￠)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `tests/compiler/move_effect.n.md` 繧・reboot 蠕後・ `Copy` / `Clone` 閭ｽ蜉帙Δ繝・Ν縺ｸ蜷医ｏ縺帙ｋ縲・
  - `tests/compiler/prelude_copy.n.md` 縺ｨ `tests/compiler/move_effect.n.md` 縺ｮ focused 螳溯｡後ｒ螳牙ｮ壼喧縺励�…ompiler 蛛ｴ縺ｮ荳榊・蜷医→ test 蛛ｴ縺ｮ蜑肴署縺壹ｌ繧貞・繧雁・縺代ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `Copy` 縺ｮ蜀榊茜逕ｨ蜿ｯ蜷ｦ繧・structural 縺ｪ譌｢螳壼�､縺ｨ縺励※譖ｸ縺・※縺・◆ case 縺梧ｮ九▲縺ｦ縺翫ｊ縲〉eboot 蠕後・縲梧・遉ｺ逧・↑ trait impl 縺悟髪荳�縺ｮ譬ｹ諡�縲阪→縺・≧莉墓ｧ倥→縺壹ｌ縺ｦ縺・◆縲・
  - `#target core` 縺ｮ騾壼ｸｸ prelude 縺ｧ縺ｯ `core/mem` 縺ｮ `RegionToken<.T>` 縺瑚ｦ九∴縺ｦ縺・ｋ縺溘ａ縲》est 蛛ｴ縺ｧ繝ｭ繝ｼ繧ｫ繝ｫ螳夂ｾｩ縺励◆ `RegionToken` 縺ｨ陦晉ｪ√＠縺ｦ縺・◆縲・
  - 縺昴・邨先棡縲～impl ... for RegionToken` 縺・generic 縺ｪ prelude 蛛ｴ蝙九∈隗｣豎ｺ縺輔ｌ縲～D3084` 繧・stack/return 邉ｻ縺ｮ蛻･險ｺ譁ｭ縺ｫ蜷ｸ繧上ｌ縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tests/compiler/move_effect.n.md`
    - `Point` / `Pair<i32>` / `Score` 縺ｮ蜀榊茜逕ｨ繧ｱ繝ｼ繧ｹ繧偵�∵・遉ｺ逧・↑ `Clone` / `Copy` impl 蜑肴署縺ｮ隱ｬ譏弱→ source 縺ｫ譖ｴ譁ｰ縺励◆縲・
    - local capability 讀懆ｨｼ (`Copy` / `Clone` 繧・test 蜀・〒螳夂ｾｩ縺吶ｋ case) 縺ｯ `#no_prelude` 繧剃ｻ倥￠縲｝relude 縺九ｉ迢ｬ遶九＠縺滓怙蟆冗腸蠅・〒遒ｺ隱阪☆繧句ｽ｢縺ｸ謠・∴縺溘�・
    - `i64` 縺ｮ local capability case 縺ｯ `core/cast` 萓晏ｭ倥ｒ驕ｿ縺代ｋ縺溘ａ縲～Size` struct 繧剃ｽｿ縺・ｽ｢縺ｸ鄂ｮ縺肴鋤縺医◆縲・
    - prelude 縺ｨ陦晉ｪ√＠縺ｦ縺・◆ local `RegionToken` 縺ｯ `LocalToken` 縺ｸ謾ｹ蜷阪＠縲・�壼ｸｸ prelude 荳九〒繧よ悄蠕・←縺翫ｊ縺ｫ `D3053` / `D3063` / `D3054` 繧定ｦｳ貂ｬ縺ｧ縺阪ｋ繧医≧縺ｫ縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 11` -> pass (`D3049`)
  - `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 17` -> pass
  - `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 20` -> pass (`D3053`)
  - `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 21` -> pass (`D3063`)
  - `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 22` -> pass (`D3054`)
  - `node nodesrc/tests.js -i tests/compiler/prelude_copy.n.md -i tests/compiler/move_effect.n.md --no-stdlib --no-tree -o /tmp/tests-prelude-copy-move-effect.json -j 15`
    - [邨先棡/縺代▲縺犠: `30/30 pass`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - 莉雁屓縺ｮ菫ｮ豁｣縺ｧ縺ｯ compiler 譛ｬ菴薙・螟画峩縺励※縺・↑縺・�・
  - 谿九▲縺ｦ縺・◆ failure 縺ｯ reboot 蠕御ｻ墓ｧ倥↓蟇ｾ縺吶ｋ test 蛛ｴ縺ｮ蜑肴署縺壹ｌ縺ｨ縲｝relude 縺ｧ髴ｲ蜃ｺ縺吶ｋ generic `RegionToken<.T>` 縺ｨ縺ｮ蜷榊燕陦晉ｪ√′蜴溷屏縺�縺｣縺溘�・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`alloc/io` / `std/streamio` 縺ｮ譛�蟆・facade 霑ｽ蜉�)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - reboot 譁・嶌縺ｧ螳夂ｾｩ縺輔ｌ縺ｦ縺・ｋ `alloc/io` 縺ｨ `std/streamio` 縺ｮ蝨溷床繧偵�∵里蟄倥・ `std/stdio` / `kpread` / `kpwrite` 繧貞｣翫＆縺壹↓霑ｽ蜉�縺吶ｋ縲・
  - streamio 縺ｯ text 蟆ら畑縺ｧ縺ｪ縺上�｜yte stream 繧よ桶縺医ｋ蠖｢縺ｧ險ｭ險医☆繧九�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `todo.md` 縺ｨ `doc/stdlib_breaking_reboot.md` 縺ｧ縺ｯ `alloc/io` 縺ｨ `std/streamio` 縺梧・遉ｺ縺輔ｌ縺ｦ縺・◆縺後�∫樟迥ｶ縺ｮ stdlib 縺ｫ縺ｯ縺ｾ縺�蟇ｾ蠢懊ヵ繧｡繧､繝ｫ縺檎┌縺上�～std/stdio` 縺ｨ `kp*` helper 縺檎峩謗･邨舌・莉倥＞縺溘∪縺ｾ縺�縺｣縺溘�・
  - `streamio` 繧・text 蟆ら畑縺ｫ縺吶ｋ縺ｨ縲∝ｾ檎ｶ壹・ file/socket/event stream 繧・`kpwrite` 譏・�ｼ蜈医→縺励※菴ｿ縺・屓縺帙↑縺・�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/io.nepl`
    - `ByteReader` / `ByteWriter` / `TextReader` / `TextWriter` / `Flush` / `Close` trait 繧定ｿｽ蜉�縺励◆縲・
    - `io_read_all_bytes` / `io_write_bytes` / `io_read_all_text` / `io_write_str` / `io_flush` / `io_close` helper 繧定ｿｽ蜉�縺励◆縲・
    - doc comment 縺ｯ迴ｾ陦・policy 縺ｮ `#` / `##` / `###` 讒区・縺ｸ謠・∴縺溘�・
  - `stdlib/std/streamio.nepl`
    - `StdinStream` / `StdoutStream` 繧定ｿｽ蜉�縺励�～alloc/io` trait 繧貞ｮ溯｣・＠縺溘�・
    - `stream_bytes_from_str` / `stream_bytes_to_str` 繧定ｿｽ蜉�縺励�｜inary/text helper 縺ｮ讖区ｸ｡縺励ｒ陦後∴繧九ｈ縺・↓縺励◆縲・
    - `stream_read_all_bytes` / `stream_write_bytes` / `stream_read_all_text` / `stream_write_str` / `stream_flush` / `stream_close` 繧・facade 蜷阪〒蜀榊・髢九＠縺溘�・
    - [迴ｾ迥ｶ/縺偵ｓ縺倥ｇ縺・縺ｮ `std/stdio` 縺瑚ｩｳ邏ｰ error 繧定ｿ斐＆縺ｪ縺・宛邏・・ doc comment 縺ｸ譏手ｨ倥＠縺溘�・
  - `tests/stdlib/streamio.n.md`
    - text write, binary write, stdin bytes -> stdout bytes 縺ｮ focused case 繧定ｿｽ蜉�縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - low-level 謚ｽ雎｡縺ｯ byte stream 繧貞渕貅悶↓縺励�》ext 縺ｯ extension trait 縺ｨ helper 縺ｸ蛻・屬縺励◆縲・
  - writer / flush 縺ｯ handle 繧定ｿ斐☆蛟､謖・髄 API 縺ｫ縺励�¨EPLg2 縺ｮ move / pipe 險俶ｳ輔∈蜷医ｏ縺帙◆縲・
  - `std/streamio` 縺ｮ module doctest 縺ｯ stable 縺ｪ蜈･蜿｣遒ｺ隱阪↓邨槭ｊ縲∝ｮ・stdout/stderr 縺ｮ end-to-end 縺ｯ `tests/stdlib/streamio.n.md` 蛛ｴ縺ｧ蝗ｺ螳壹＠縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/run_doctest.js -i stdlib/alloc/io.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/std/streamio.nepl -n 1` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/streamio.n.md -i stdlib/alloc/io.nepl -i stdlib/std/streamio.nepl --no-stdlib --no-tree -o /tmp/tests-streamio-focused.json -j 15`
    - [邨先棡/縺代▲縺犠: `5/5 pass`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - `std/streamio` 縺ｯ縺ｾ縺� stdin/stdout 縺ｮ譛�蟆・facade 縺ｫ逡吶ａ縺ｦ縺・ｋ縺後�｜inary/text 縺ｮ trait 髱｢縺ｯ蜈医↓蝗ｺ螳壹〒縺阪◆縲・
  - `kpwrite` / `kpread` 繧偵％縺ｮ螻､縺ｸ谿ｵ髫守ｧｻ陦後☆繧玖ｶｳ蝣ｴ縺ｯ縺ｧ縺阪◆縲・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`streamio` 縺ｮ binary buffer 繧・`ByteBuf` 縺ｸ蜀崎ｨｭ險・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `streamio` 繧呈悽蠖薙↓ binary-capable 縺ｫ縺励�～Vec<u8>` 縺ｮ蜀・Κ陦ｨ迴ｾ縺ｸ萓晏ｭ倥＠縺滓闘莨ｼ逧・↑ byte write 繧偵ｄ繧√ｋ縲・
  - `nodesrc/tests.js` 縺ｧ繧・stdout/stderr 讀懆ｨｼ繧堤｢ｺ螳溘↓譛牙柑蛹悶＠縲！/O mismatch 繧定ｦ矩�・＆縺ｪ縺・focused 讀懆ｨｼ謇矩�・ｒ蝗ｺ螳壹☆繧九�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - 蜈郁｡悟ｮ溯｣・〒縺ｯ `ByteReader` / `ByteWriter` 縺ｮ蟐剃ｽ薙ｒ `Vec<u8>` 縺ｫ縺励※縺・◆縺後�¨EPLg2 縺ｮ `Vec<u8>` 縺ｯ `fd_write` 縺ｫ縺昴・縺ｾ縺ｾ貂｡縺帙ｋ騾｣邯・byte buffer 縺ｧ縺ｯ縺ｪ縺九▲縺溘�・
  - 縺昴・縺溘ａ `stream_write_bytes` 縺ｯ `A\0\0` 縺ｮ繧医≧縺ｪ padded 蜃ｺ蜉帙↓縺ｪ繧翫�｜inary stream 縺ｨ縺励※螢翫ｌ縺ｦ縺・◆縲・
  - 縺ゅｏ縺帙※ `nodesrc/tests.js` 縺ｯ譌｢螳壹〒 `assert_io: false` 縺ｪ縺ｮ縺ｧ縲《tdout mismatch 縺・JSON 荳翫〒縺ｯ pass 謇ｱ縺・↓縺ｪ繧九こ繝ｼ繧ｹ縺後≠繧翫�～--assert-io` 繧剃ｻ倥￠縺ｪ縺・→ binary 蝗槫ｸｰ繧定ｦ矩�・＠縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/io.nepl`
    - `ByteBuf` 繧定ｿｽ蜉�縺励�～ptr: MemPtr<u8>` 縺ｨ `len: i32` 繧呈戟縺､謇�譛・buffer 縺ｨ縺励※螳夂ｾｩ縺励◆縲・
    - `io_bytebuf_empty` / `io_bytebuf_len` / `io_bytebuf_free` / `io_bytebuf_from_str` / `io_bytebuf_to_str` 繧定ｿｽ蜉�縺励◆縲・
    - `ByteReader` / `ByteWriter` 縺ｨ `io_read_all_bytes` / `io_write_bytes` 縺ｮ蟐剃ｽ薙ｒ `Vec<u8>` 縺九ｉ `ByteBuf` 縺ｸ螟画峩縺励◆縲・
  - `stdlib/std/stdio.nepl`
    - `stdio_write_bytes` 縺ｯ `ByteBuf` 繧堤峩謗･ iovec 縺ｫ霈峨○縺ｦ stdout 縺ｸ譖ｸ縺丞ｽ｢縺ｸ螟画峩縺励◆縲・
    - `stdio_read_all_bytes` 縺ｯ縲∫樟迥ｶ縺ｮ `read_all` 邨先棡繧・`ByteBuf` 縺ｫ隍・｣ｽ縺吶ｋ蠖｢縺ｸ謨ｴ逅・＠縺溘�・
  - `stdlib/std/streamio.nepl`
    - `stream_bytes_from_str` / `stream_bytes_to_str` 繧・`ByteBuf` 繝吶・繧ｹ縺ｸ螟画峩縺励◆縲・
    - `StdinStream` / `StdoutStream` 縺ｮ binary trait 螳溯｣・ｒ `ByteBuf` 蜑肴署縺ｸ蟾ｮ縺玲崛縺医◆縲・
    - doc comment 縺ｫ縲茎tdin byte read 縺ｯ迴ｾ迥ｶ `read_all` 逕ｱ譚･縺ｮ隍・｣ｽ縲阪→縺・≧蛻ｶ邏・ｒ霑ｽ險倥＠縺溘�・
  - `tests/stdlib/streamio.n.md`
    - text write, binary write, stdin bytes -> stdout bytes 縺ｫ蜉�縺医�¨UL 繧貞性繧� binary/text roundtrip 繧・`assert_str_eq` 縺ｧ讀懆ｨｼ縺吶ｋ case 縺ｫ譖ｴ譁ｰ縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/tests.js -i tests/stdlib/streamio.n.md -i stdlib/alloc/io.nepl -i stdlib/std/streamio.nepl -i stdlib/std/stdio.nepl --assert-io --no-stdlib --no-tree -o /tmp/tests-streamio-bytebuf.json -j 15`
    - [邨先棡/縺代▲縺犠: `33/33 pass`
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 4` -> pass
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/cli.js -i stdlib/alloc/io.nepl -i stdlib/std/stdio.nepl -i stdlib/std/streamio.nepl -o html=/tmp/streamio-doc-html`
    - [邨先棡/縺代▲縺犠: `generated 3 html file(s)`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - binary stream 縺ｮ蟐剃ｽ薙・ `Vec<u8>` 縺ｧ縺ｯ縺ｪ縺・`ByteBuf` 縺ｫ蝗ｺ螳壹＠縺溘�・
  - `nodesrc/tests.js` 縺ｧ I/O 繧定ｦ九ｋ focused 讀懆ｨｼ縺ｯ縲∽ｻ雁ｾ・`--assert-io` 繧剃ｻ倥￠繧句燕謠舌〒謇ｱ縺・�・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`nodesrc/tests.js` 縺ｮ I/O 讀懆ｨｼ譌｢螳壼�､繧剃ｿｮ豁｣)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `tests.js` 縺・`stdout:` / `stderr:` 繧呈嶌縺・◆ doctest 繧呈里螳壹〒蜴ｳ蟇・ｯ碑ｼ・＠縲～run_doctest.js` 縺ｨ蜷後§譛溷ｾ・〒菴ｿ縺医ｋ繧医≧縺ｫ縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - 縺薙ｌ縺ｾ縺ｧ縺ｮ `tests.js` 縺ｯ `--assert-io` / `NEPL_ASSERT_IO=1` / `assert_io` tag 縺檎┌縺・剞繧翫�～expected_stdout` / `expected_stderr` 繧呈戟縺､ case 縺ｧ繧・I/O mismatch 繧・pass 謇ｱ縺・＠縺ｦ縺・◆縲・
  - 縺昴・縺溘ａ縲｜inary streamio 縺ｮ荳肴ｭ｣蜃ｺ蜉帙′ JSON 髮・ｨ井ｸ翫〒縺ｯ `pass` 縺ｫ隕九∴縲’ocused suite 縺ｮ菫｡鬆ｼ諤ｧ縺瑚誠縺｡縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nodesrc/tests.js`
    - `expected_stdout` 縺ｾ縺溘・ `expected_stderr` 縺後≠繧・case 縺ｯ縲∵里螳壹〒 I/O 豈碑ｼ・ｒ譛牙柑縺ｫ縺吶ｋ繧医≧螟画峩縺励◆縲・
    - `--assert-io` / `NEPL_ASSERT_IO=1` / `assert_io` tag 縺ｯ譏守､ｺ繝輔Λ繧ｰ縺ｨ縺励※谿九＠縺､縺､縲√�栗/O 讀懆ｨｼ繧呈怏蜉ｹ蛹悶☆繧句髪荳�譚｡莉ｶ縲阪〒縺ｯ縺ｪ縺上＠縺溘�・
  - `nodesrc/README.n.md`
    - `tests.js` 縺ｧ繧・`stdout:` / `stderr:` 繧呈里螳壹〒讀懆ｨｼ縺吶ｋ縺薙→繧定ｿｽ險倥＠縺溘�・
    - `--assert-io` 縺ｯ陬懷勧繝輔Λ繧ｰ縺ｧ縺ゅｊ縲！/O 譛溷ｾ・�､縺ｮ譛臥┌縺昴・繧ゅ・繧呈怏蜉ｹ蛹悶☆繧句ｿ・�域擅莉ｶ縺ｧ縺ｯ縺ｪ縺・％縺ｨ繧呈・險倥＠縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/tests.js -i tests/stdlib/stdout.n.md -i tests/stdlib/stdin.n.md -i tests/stdlib/kp.n.md -i tests/stdlib/streamio.n.md --no-stdlib --no-tree -o /tmp/tests-io-default-assert.json -j 15`
    - [邨先棡/縺代▲縺犠: `22/22 pass`
  - `node nodesrc/tests.js -i stdlib/alloc/io.nepl -i stdlib/std/streamio.nepl --no-stdlib --no-tree -o /tmp/tests-stdlib-io-doctest-default.json -j 15`
    - [邨先棡/縺代▲縺犠: `2/2 pass`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - `tests.js` 縺ｨ `run_doctest.js` 縺ｮ I/O 讀懆ｨｼ譛溷ｾ・・謠・▲縺溘�・
  - 莉雁ｾ・`stdout:` / `stderr:` 繧呈嶌縺・◆ doctest 縺ｯ縲∬ｿｽ蜉�繝輔Λ繧ｰ縺ｪ縺励〒繧・mismatch 縺ｧ關ｽ縺｡繧九�・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`kpwrite` 縺ｮ buffered writer core 繧・`std/streamio` 縺ｸ遘ｻ邂｡)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `todo.md` 縺ｮ縲形kpwrite` 縺ｮ荳ｭ譬ｸ繧・`std/streamio` 縺ｸ譏・�ｼ縺輔○繧九�阪ｒ騾ｲ繧√�《tdout buffering 繧・`kp` 蟆ら畑螳溯｣・・縺ｾ縺ｾ謖√◆縺ｪ縺・ｧ区・縺ｸ蟇・○繧九�・
  - partial write 繝ｫ繝ｼ繝励′ `kpwrite` 蛛ｴ縺ｸ謨｣繧峨・縺｣縺ｦ縺・◆迥ｶ諷九ｒ隗｣豸医＠縲～std/stdio` 縺ｨ `std/streamio` 縺ｮ雋ｬ蜍吝｢・阜繧呈紛逅・☆繧九�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - 縺薙ｌ縺ｾ縺ｧ縺ｮ `kpwrite` 縺ｯ buffer 謇�譛峨�”eader 邂｡逅・�｝artial write 蜷ｸ蜿弱�∵枚蟄怜・/謨ｰ蛟､謨ｴ蠖｢繧・1 module 縺ｫ謚ｱ縺医※縺翫ｊ縲～std/streamio` 縺ｯ stdin/stdout 縺ｮ譛�蟆・facade 縺ｫ逡吶∪縺｣縺ｦ縺・◆縲・
  - 縺昴・縺溘ａ stdout buffering 縺ｮ荳�闊ｬ蛹門庄閭ｽ驛ｨ蛻・ｒ莉・module 縺悟・蛻ｩ逕ｨ縺ｧ縺阪★縲～kp` 蛛ｴ縺ｫ syscall 逕ｱ譚･縺ｮ螳溯｣・ｩｳ邏ｰ縺梧ｮ九▲縺ｦ縺・◆縲・
  - 縺ゅｏ縺帙※ stdout 縺ｸ縺ｮ驛ｨ蛻・嶌縺崎ｾｼ縺ｿ蜷ｸ蜿弱′ `print` / `stdio_write_bytes` 縺ｨ `kpwrite` 縺ｧ蛻･邨瑚ｷｯ縺ｫ縺ｪ縺｣縺ｦ縺翫ｊ縲∝酔縺・stdout 蜃ｺ蜉帙〒繧りｲｬ蜍吶′蛻・淵縺励※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/std/stdio.nepl`
    - `stdio_write_mem` 繧定ｿｽ蜉�縺励�～MemPtr<u8>` 縺ｨ髟ｷ縺輔ｒ蜿励￠縺ｦ partial write 繧貞精蜿弱＠縺ｪ縺後ｉ stdout 縺ｸ譖ｸ縺丞・騾夂ｵ瑚ｷｯ繧定ｿｽ蜉�縺励◆縲・
    - `print` 縺ｨ `stdio_write_bytes` 縺ｯ縺薙・ helper 繧剃ｽｿ縺・ｽ｢縺ｸ謨ｴ逅・＠縺溘�・
  - `stdlib/std/streamio.nepl`
    - `StreamWriter` 繧定ｿｽ蜉�縺励�｜uffer 謇�譛峨・header 邂｡逅・・flush繝ｻtext/i32/i64/f32/f64 蜃ｺ蜉帙ｒ `std` 蛛ｴ縺ｧ謠蝉ｾ帙☆繧九ｈ縺・↓縺励◆縲・
    - `stream_writer_new` / `stream_writer_free` / `stream_writer_flush` / `stream_writer_put_u8` / `stream_writer_write_str` / `stream_writer_write_i32` / `stream_writer_write_i64` / `stream_writer_write_f64` 縺ｪ縺ｩ繧定ｿｽ蜉�縺励◆縲・
    - `stream_writer_flush` 縺ｯ `stdio_write_mem` 繧剃ｽｿ縺・％縺ｨ縺ｧ stdout 蛛ｴ縺ｮ驛ｨ蛻・嶌縺崎ｾｼ縺ｿ蜷ｸ蜿弱→邨瑚ｷｯ繧貞・譛峨☆繧九ｈ縺・↓縺励◆縲・
  - `stdlib/kp/kpwrite.nepl`
    - `Writer` 繧・`StreamWriter` 1 蛟九□縺代ｒ菫晄戟縺吶ｋ阮・＞ wrapper 縺ｫ鄂ｮ縺肴鋤縺医◆縲・
    - 譌｢蟄倥・ `writer_*` API 蜷阪・邯ｭ謖√＠縺､縺､縲∝ｮ滉ｽ薙・ `stream_writer_*` 縺ｸ蟋碑ｭｲ縺吶ｋ蠖｢縺ｫ謨ｴ逅・＠縺溘�・
  - `tests/stdlib/streamio.n.md`
    - `StreamWriter` 繧・`std/streamio` 縺九ｉ逶ｴ謗･菴ｿ縺・focused case 繧定ｿｽ蜉�縺励�》ext/i32/space helper 繧貞屓蟶ｰ蝗ｺ螳壹＠縺溘�・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `kpwrite` 縺ｮ縺・■縲檎ｫｶ謚�蜷代￠縺ｮ蜷榊燕縲阪〒縺ｯ縺ｪ縺上�茎tdout buffering 縺ｨ縺・≧豎守畑讖溯・縲阪・ `std` 縺ｫ鄂ｮ縺上・縺・reboot 譁ｹ驥昴→荳�閾ｴ縺吶ｋ縺ｨ蛻､譁ｭ縺励◆縲・
  - partial write 縺ｮ蜷ｸ蜿弱・ writer 縺斐→縺ｫ謖√◆縺帙★縲《tdout 譖ｸ縺榊・縺礼ｵ瑚ｷｯ縺ｮ譛�荳句ｱ､縺ｧ縺ゅｋ `std/stdio` 縺ｫ髮・ｴ・＠縺溘�・
  - `kpwrite` 縺ｮ public API 縺ｯ譌｢蟄倥ユ繧ｹ繝郁ｳ・肇繧堤ｶｭ謖√☆繧九◆繧∵ｮ九＠縲∝・驛ｨ螳溯｣・□縺代ｒ `StreamWriter` wrapper 蛹悶＠縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/std/streamio.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 5` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/streamio.n.md -i stdlib/std/streamio.nepl -i stdlib/kp/kpwrite.nepl -i tests/stdlib/kp.n.md -i tests/stdlib/kp_i64.n.md --no-stdlib --no-tree -o /tmp/tests-streamio-kpwrite-kp-focus.json -j 15`
    - [邨先棡/縺代▲縺犠: `21/21 pass`
  - `/tmp/tests-streamio-kpwrite-kp-focus.json`
    - [遒ｺ隱・縺九￥縺ｫ繧転: `summary.total = 21`, `summary.passed = 21`, `summary.failed = 0`
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/cli.js -i stdlib/std/streamio.nepl -i stdlib/kp/kpwrite.nepl -o html=/tmp/streamio-kpwrite-doc-html`
    - [邨先棡/縺代▲縺犠: `generated 2 html file(s)`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - `kpwrite` 縺ｮ buffered writer core 縺ｯ `std/streamio` 縺ｸ遘ｻ繧翫�～kp` 蛛ｴ縺ｯ阮・＞ wrapper 讒区・縺ｫ縺ｪ縺｣縺溘�・
  - `kpread` 縺ｮ荳�闊ｬ蛹門庄閭ｽ驛ｨ蛻・・縺ｾ縺� `kp` 蛛ｴ縺ｫ谿九▲縺ｦ縺・ｋ縺溘ａ縲》odo 7 縺ｯ邯咏ｶ壻ｸｭ縺ｧ縺ゅｋ縲・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`kpread` 縺ｮ scanner core 繧・`std/streamio` 縺ｸ遘ｻ邂｡)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - todo 7 縺ｮ谿倶ｻｶ縺ｧ縺ゅｋ `kpread` 縺ｮ荳�闊ｬ蛹門庄閭ｽ驛ｨ蛻・ｒ `std/streamio` 縺ｸ遘ｻ縺励�～kp` 蛛ｴ縺ｫ縺ｯ遶ｶ謚�蜷代￠縺ｮ蜷域・ helper 縺�縺代ｒ谿九☆縲・
  - stdin binary 隱ｭ縺ｿ霎ｼ縺ｿ縺ｮ unbounded 邨瑚ｷｯ繧・`kp` 蟆ら畑螳溯｣・・縺ｾ縺ｾ縺ｫ縺帙★縲～std` 蛛ｴ縺ｮ豁｣隕丞・蜿｣縺ｸ謨ｴ逅・☆繧九�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - 縺薙ｌ縺ｾ縺ｧ縺ｮ `kpread` 縺ｯ縲《tdin 蜈ｨ隱ｭ霎ｼ縲｜uffer/header 邂｡逅・�》oken/i32/i64/f64 parser縲∫ｫｶ謚�蜷代￠ `Vec`/陦悟・ helper 繧・1 module 鄒､縺ｧ謚ｱ縺医※縺・◆縲・
  - 縺昴・縺溘ａ `StreamWriter` 繧・`std/streamio` 縺ｸ遘ｻ縺励◆蠕後ｂ縲∝ｯｾ縺ｫ縺ｪ繧・scanner core 縺�縺代′ `kp` 蛛ｴ縺ｫ谿九ｊ縲～std/streamio` 縺後�御ｸ�闊ｬ stream facade縲阪→縺励※迚・焔關ｽ縺｡縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
  - 縺ゅｏ縺帙※ stdin binary read 縺ｮ unbounded 繝ｫ繝ｼ繝励′ `kpread_core` 縺ｫ髢峨§縺ｦ縺翫ｊ縲～std/streamio` / `std/stdio` 縺ｮ public binary 邨瑚ｷｯ縺ｨ蛻・妙縺輔ｌ縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/std/stdio.nepl`
    - `stdio_read_all_bytes` 繧偵�～read_all` 縺ｮ 4096 byte 隍・｣ｽ縺ｧ縺ｯ縺ｪ縺上�・4KiB 縺九ｉ諡｡蠑ｵ縺励▽縺､ EOF 縺ｾ縺ｧ `fd_read` 繧貞渚蠕ｩ縺吶ｋ unbounded binary read 縺ｸ鄂ｮ縺肴鋤縺医◆縲・
    - 縺薙ｌ縺ｫ繧医ｊ `ByteBuf` 縺ｮ stdin 邨瑚ｷｯ縺ｨ scanner 邨瑚ｷｯ縺悟酔縺・`std` 螻､縺ｫ謠・▲縺溘�・
  - `stdlib/std/streamio.nepl`
    - `StreamScanner` 繧定ｿｽ蜉�縺励�～stream_scanner_new` / `stream_scanner_skip_ws` / `stream_scanner_is_eof` / `stream_scanner_skip_token` / `stream_scanner_read_token` / `stream_scanner_read_i32` / `stream_scanner_read_u64` / `stream_scanner_read_i64` / `stream_scanner_read_f64` / `stream_scanner_read_f32` 繧・`std` 蛛ｴ縺ｧ謠蝉ｾ帙☆繧九ｈ縺・↓縺励◆縲・
    - scanner 縺ｯ `ByteBuf` 縺ｮ pointer/len 繧・header 縺ｧ蜈ｱ譛峨＠縲～Copy` / `Clone` 縺ｯ cursor 蜈ｱ譛峨・霆ｽ驥・handle 縺ｨ縺励※螳夂ｾｩ縺励◆縲・
  - `stdlib/kp/kpread.nepl`
    - `scanner_new` 縺ｨ primitive reader 鄒､縺ｯ `StreamScanner` 縺ｸ蟋碑ｭｲ縺吶ｋ wrapper 縺ｫ謨ｴ逅・＠縺溘�・
    - `Vec` / 陦悟・ / 蛹ｺ髢薙け繧ｨ繝ｪ蜈･蜉帙↑縺ｩ縺ｮ遶ｶ謚�蜷代￠ helper 縺ｯ `kp` 蛛ｴ縺ｫ谿九＠縺溘�・
    - file 蜀帝�ｭ comment 繧偵�∵眠讒区・縺ｫ蜷医ｏ縺帙※ `StreamScanner` wrapper 蜑肴署縺ｸ譖ｴ譁ｰ縺励◆縲・
  - `stdlib/kp/kpread_core.nepl`
    - 螳滉ｽ薙′ `std/streamio` / `std/stdio` 縺ｸ遘ｻ縺｣縺溘◆繧∝炎髯､縺励◆縲・
  - `tests/stdlib/streamio.n.md`
    - `StreamScanner` 逶ｴ蛻ｩ逕ｨ縺ｮ focused case 繧定ｿｽ蜉�縺励�∵焚蛟､隱ｭ蜿悶→ BOM + token 隱ｭ蜿悶ｒ蝗槫ｸｰ蝗ｺ螳壹＠縺溘�・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `stdin 蜈ｨ隱ｭ霎ｼ + token parser` 縺ｯ遶ｶ謚�蜷代￠ sugar 縺ｧ縺ｯ縺ｪ縺乗ｱ守畑 scanner 讖溯・縺ｪ縺ｮ縺ｧ縲～kp` 縺ｧ縺ｯ縺ｪ縺・`std/streamio` 縺ｫ鄂ｮ縺上・縺・reboot 譁ｹ驥昴↓蜷医≧縺ｨ蛻､譁ｭ縺励◆縲・
  - 荳�譁ｹ縺ｧ `Vec`/陦悟・/蝠城｡悟ｮ壼梛蜈･蜉帙ヱ繝・け縺ｯ遶ｶ謚�蜷代￠ API 縺ｨ縺ｿ縺ｪ縺励�～kp` 蛛ｴ縺ｫ谿九＠縺溘�・
  - `StreamScanner` 縺ｮ謇�譛峨Δ繝・Ν縺ｯ譌｢蟄・`Scanner` 縺ｨ蜷後§縺・shared cursor 繧堤ｶｭ謖√＠縲『rapper 鄂ｮ謠帙〒譌｢蟄・`kp` 繝・せ繝郁ｳ・肇繧貞ｴｩ縺輔↑縺・ｽ｢繧貞━蜈医＠縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 7` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 8` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/streamio.n.md -i stdlib/std/streamio.nepl -i stdlib/std/stdio.nepl -i stdlib/kp/kpread.nepl -i tests/stdlib/kp.n.md -i tests/stdlib/kp_i64.n.md -i tests/stdlib/stdin.n.md --no-stdlib --no-tree -o /tmp/tests-streamio-kpread-focus.json -j 15`
    - [邨先棡/縺代▲縺犠: `52/52 pass`
  - `/tmp/tests-streamio-kpread-focus.json`
    - [遒ｺ隱・縺九￥縺ｫ繧転: `summary.total = 52`, `summary.passed = 52`, `summary.failed = 0`
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/cli.js -i stdlib/std/streamio.nepl -i stdlib/kp/kpread.nepl -o html=/tmp/streamio-kpread-doc-html`
    - [邨先棡/縺代▲縺犠: `generated 2 html file(s)`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - `kpwrite` 縺ｫ邯壹＞縺ｦ `kpread` 縺ｮ primitive scanner core 繧・`std/streamio` 縺ｸ遘ｻ繧翫�》odo 7 縺ｮ縲形kpread` / `kpwrite` 縺ｮ荳ｭ譬ｸ繧・`std/streamio` 縺ｸ譏・�ｼ縲阪′荳�谿ｵ騾ｲ繧薙□縲・
  - `kp` 蛛ｴ縺ｫ縺ｯ `Vec`/陦悟・/遶ｶ謚�蜈･蜉帙ヱ繝・け縺ｮ繧医≧縺ｪ遶ｶ謚�蜷代￠ sugar 縺梧ｮ九▲縺ｦ縺・ｋ縲・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`std/fs` 縺ｮ binary path 繧・`ByteBuf` 縺ｸ邨ｱ荳�)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - todo 7 縺ｮ `std/fs` 繧偵�√☆縺ｧ縺ｫ `alloc/io` 縺ｨ `std/streamio` 縺ｧ謗｡逕ｨ縺励◆ binary 陦ｨ迴ｾ `ByteBuf` 縺ｫ謠・∴繧九�・
  - `std` 驟堺ｸ九・ binary I/O 縺・module 縺斐→縺ｫ `Vec<u8>` 縺ｨ `ByteBuf` 縺ｸ蛻・｣ゅ＠縺ｦ縺・ｋ迥ｶ諷九ｒ隗｣豸医☆繧九�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `std/streamio` 縺ｨ `std/stdio` 縺ｯ reboot 蠕後↓ `ByteBuf` 繧・binary 蟐剃ｽ薙→縺励※菴ｿ縺・ｨｭ險医∈蟇・▲縺ｦ縺・◆縺後�～std/fs` 縺�縺代′譌ｧ譚･縺ｮ `Vec<u8>` 蜑肴署繧堤ｶｭ謖√＠縺ｦ縺・◆縲・
  - 縺昴・縺溘ａ file read 縺ｮ霑斐ｊ蛟､縺�縺代′蛻･陦ｨ迴ｾ縺ｨ縺ｪ繧翫�～streamio` / `stdio` 縺ｨ binary path 繧貞・譛峨〒縺阪★縲～std` facade 蜈ｨ菴薙〒蟐剃ｽ薙′荳�閾ｴ縺励※縺・↑縺九▲縺溘�・
  - 縺ゅｏ縺帙※ `std/fs` 蜀・・蟆上＆縺ｪ菴懈･ｭ鬆伜沺縺ｯ `RegionToken<u8>` 繧剃ｽｿ縺｣縺ｦ縺翫ｊ縲”elper 繧定､・焚蝗槫盾辣ｧ縺吶ｋ邂・園縺ｧ move error 繧定ｵｷ縺薙＠繧・☆縺・ｧ矩��縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/std/fs.nepl`
    - `alloc/collections/vec` 萓晏ｭ倥ｒ螟悶＠縲～alloc/io` 繧・import 縺吶ｋ讒区・縺ｸ螟画峩縺励◆縲・
    - `fs_read_fd_bytes` / `fs_read_to_bytes` 縺ｮ霑斐ｊ蛟､繧・`Result<ByteBuf, i32>` 縺ｸ螟画峩縺励◆縲・
    - `fs_bytes_to_string` 縺ｯ `io_bytebuf_to_str` 繧剃ｽｿ縺・埋縺・､画鋤 helper 縺ｫ謨ｴ逅・＠縺溘�・
    - fd/iovec/nread 縺ｮ荳�譎る�伜沺縺ｯ `RegionToken` 縺ｧ縺ｯ縺ｪ縺・`alloc_ptr<u8>` / `dealloc_ptr<u8>` 縺ｧ邂｡逅・＠縲～MemPtr<u8>` 縺九ｉ `region_new` 縺ｧ `i32*` 繧貞・繧雁・縺吝ｽ｢縺ｸ邨ｱ荳�縺励◆縲・
    - file 蜈ｨ菴薙・隱ｬ譏弱→髢｢謨ｰ comment 繧偵�∵眠縺励＞ `ByteBuf` 繝吶・繧ｹ螳溯｣・↓蜷医ｏ縺帙※譖ｴ譁ｰ縺励◆縲・
  - `tests/stdlib/fs.n.md`
    - `fs_read_to_string` 縺ｮ missing file case 縺ｫ蜉�縺医�∵里遏･縺ｮ test file 繧・`ByteBuf` 縺ｨ縺励※隱ｭ縺ｿ縲√◎縺ｮ縺ｾ縺ｾ `str` 縺ｸ謌ｻ縺帙ｋ縺薙→繧堤｢ｺ隱阪☆繧・focused case 繧定ｿｽ蜉�縺励◆縲・
    - `ByteBuf` 縺ｯ move-only 縺ｪ縺ｮ縺ｧ縲・聞縺慕｢ｺ隱榊ｾ後↓蜀榊茜逕ｨ縺吶ｋ蠖｢縺ｯ蜿悶ｉ縺壹�》ext 蛹悶∪縺ｧ荳�豌励↓豸郁ｲｻ縺吶ｋ讒区・縺ｫ縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `ByteBuf` 縺ｯ fd read/write 縺ｫ逶ｴ謗･貂｡縺帙ｋ謇�譛峨ヰ繝・ヵ繧｡縺ｨ縺励※ `alloc/io` 縺ｫ縺吶〒縺ｫ螳夂ｾｩ貂医∩縺ｧ縺ゅｊ縲～std/fs` 縺�縺代ｒ `Vec<u8>` 縺ｮ縺ｾ縺ｾ谿九☆蜷育炊諤ｧ縺ｯ縺ｪ縺・→蛻､譁ｭ縺励◆縲・
  - `RegionToken` 縺ｮ菴ｿ縺・屓縺励〒 move error 繧帝∩縺代ｋ縺溘ａ縺ｮ蝣ｴ蠖薙◆繧顔噪縺ｪ隍・｣ｽ helper 縺ｯ蜈･繧後★縲～std/stdio` 縺ｨ蜷後§繝昴う繝ｳ繧ｿ繝吶・繧ｹ縺ｮ荳�譎る�伜沺邂｡逅・∈蟇・○縺溘�・
  - 譌｢蟄倥・ `stdlib/tests/fs.n.md` 縺ｯ missing file 縺ｮ譛�蟆冗｢ｺ隱阪→縺励※谿九＠縲∵眠縺励＞ `tests/stdlib/fs.n.md` 縺ｧ縺ｯ binary path 縺ｮ蝗槫ｸｰ繧貞・髮｢縺励※蝗ｺ螳壹＠縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tests/stdlib/fs.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/fs.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/fs.n.md -i stdlib/tests/fs.n.md -i stdlib/std/fs.nepl --no-stdlib --no-tree -o /tmp/tests-fs-all.json -j 15`
    - [邨先棡/縺代▲縺犠: `8/8 pass`
  - `/tmp/tests-fs-all.json`
    - [遒ｺ隱・縺九￥縺ｫ繧転: `summary.total = 8`, `summary.passed = 8`, `summary.failed = 0`
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/cli.js -i stdlib/std/fs.nepl -i tests/stdlib/fs.n.md -o html=/tmp/fs-doc-html`
    - [邨先棡/縺代▲縺犠: `generated 2 html file(s)`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - `std/stdio` / `std/streamio` / `std/fs` 縺ｮ binary path 縺後☆縺ｹ縺ｦ `ByteBuf` 繧貞・譛峨☆繧句ｽ｢縺ｫ縺ｪ縺｣縺溘�・
  - todo 7 縺ｮ `std` facade 謨ｴ逅・・縲～env/cliarg` 繧・ｮ九ｊ縺ｮ target 萓晏ｭ・API 縺ｮ遒ｺ隱阪ｒ谿九＠縺ｦ邯咏ｶ壻ｸｭ縺ｧ縺ゅｋ縲・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`std/env/cliarg` 縺ｮ荳�譎る�伜沺邂｡逅・ｒ `alloc_ptr` 縺ｸ邨ｱ荳�)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - todo 7 縺ｮ `std/env` 謨ｴ逅・→縺励※縲～cliarg` 繧・reboot 蠕後・ move/effect 隕丞援縺ｨ遏帷崟縺励↑縺・facade 縺ｫ逶ｴ縺吶�・
  - `std/fs` 縺ｨ蜷梧ｧ倥↓縲∽ｸ�譎ゆｽ懈･ｭ鬆伜沺縺ｮ謇�譛峨Δ繝・Ν繧・`RegionToken` 萓晏ｭ倥°繧牙､悶＠縲》arget 萓晏ｭ伜ｮ溯｣・・蜀・Κ隍・尅縺輔ｒ蛻ｩ逕ｨ閠・°繧蛾國縺吝ｽ｢縺ｸ蟇・○繧九�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `cliarg_count` / `cliarg_get` / `cstr_to_str` 縺ｯ 2026-03-06 譎らせ縺ｧ `RegionToken<u8>` 繝吶・繧ｹ縺ｸ蟇・○縺ｦ縺・◆縺後�［ove check 蠑ｷ蛹門ｾ後・ `meta` 繧・`argv` 繧・helper 縺ｫ貂｡縺励◆譎らせ縺ｧ謇�譛画ｨｩ縺檎ｧｻ繧翫�√◎縺ｮ蠕後・蜀榊盾辣ｧ縺ｧ `D3053` 縺悟・繧狗憾諷九↓縺ｪ縺｣縺ｦ縺・◆縲・
  - 縺､縺ｾ繧・`cliarg` 縺�縺代′縲御ｸ�譎ゅヰ繝・ヵ繧｡繧堤ｷ壼ｽ｢ token 縺ｧ謖√■蝗槭☆譌ｧ險ｭ險医�阪↓逡吶∪縺｣縺ｦ縺翫ｊ縲∫峩霑代〒 `std/fs` 縺ｫ驕ｩ逕ｨ縺励◆隗｣縺肴婿縺ｨ謠・▲縺ｦ縺・↑縺九▲縺溘�・
  - 縺昴・邨先棡縲～stdlib/tests/cliarg.n.md` 縺ｯ compile fail 縺励�～cliarg_argv_stdout_count` 繧らｩｺ蜃ｺ蜉帙↓縺ｪ縺｣縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/std/env/cliarg.nepl`
    - `cli_i32_ptr` 繧・`MemPtr<u8> + size + off` 縺九ｉ `i32*` 繧貞・繧雁・縺・helper 縺ｫ螟画峩縺励◆縲・
    - `cli_alloc_u8_region` / `cli_free_region` / `cli_u8_ptr` 繧貞炎髯､縺励�∽ｸ�譎ゅヰ繝・ヵ繧｡縺ｯ `alloc_ptr<u8>` / `dealloc_ptr<u8>` 縺ｧ邂｡逅・☆繧句ｽ｢縺ｸ邨ｱ荳�縺励◆縲・
    - LLVM 蛛ｴ縺ｮ `__cli_copy_to_cstr`縲～args_sizes_get`縲～args_get` 繧・`MemPtr<u8>` 繝吶・繧ｹ縺ｸ譖ｴ譁ｰ縺励◆縲・
    - `cstr_to_str` 縺ｯ `RegionToken` 繧剃ｻ九＆縺・`[len][bytes]` 鬆伜沺繧堤峩謗･遒ｺ菫昴＠縺ｦ邨・∩遶九※繧句ｽ｢縺ｫ螟画峩縺励◆縲・
    - `cliarg_count` / `cliarg_get` 縺ｮ meta, argv, argv_buf 縺ｮ蟇ｿ蜻ｽ邂｡逅・ｒ縺吶∋縺ｦ `alloc_ptr` 繝吶・繧ｹ縺ｸ鄂ｮ縺肴鋤縺医◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `cliarg` 縺ｮ繝｡繧ｿ諠・�ｱ繝舌ャ繝輔ぃ縺ｯ髢｢謨ｰ蜀・Ο繝ｼ繧ｫ繝ｫ縺ｮ荳�譎る�伜沺縺ｧ縺ゅｊ縲∝・髢九・螳牙・ API 髱｢縺ｧ縺ｯ縺ｪ縺・◆繧√�～RegionToken` 繧堤┌逅・↓陦ｨ縺ｸ騾壹☆繧医ｊ `alloc_ptr` 縺ｧ髢峨§縺滓婿縺瑚ｲｬ蜍吶↓蜷医≧縺ｨ蛻､譁ｭ縺励◆縲・
  - `cstr_len` / `cstr_to_str` 縺ｮ蜈ｬ髢句｢・阜縺ｯ蠕捺擂騾壹ｊ `MemPtr<u8>` 縺ｮ縺ｾ縺ｾ邯ｭ謖√＠縲∝梛螳牙・蛹匁ｸ医∩縺ｮ API 蠖｢迥ｶ縺ｯ蟠ｩ縺輔↑縺九▲縺溘�・
  - `std/fs` 縺ｨ `std/env/cliarg` 縺ｮ荳｡譁ｹ縺ｧ蜷後§荳�譎る�伜沺繝代ち繝ｼ繝ｳ縺ｫ謠・∴縺溘％縺ｨ縺ｧ縲～std` facade 蜀・・ target 萓晏ｭ伜ｮ溯｣・婿驥昴ｂ荳�閾ｴ縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/tests/cliarg.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/cliarg.n.md -n 2` -> pass (`stdout: "3"`)
  - `node nodesrc/tests.js -i stdlib/tests/cliarg.n.md -i stdlib/std/env/cliarg.nepl --no-stdlib --no-tree -o /tmp/tests-cliarg-current.json -j 15`
    - [邨先棡/縺代▲縺犠: `9/9 pass`
  - `/tmp/tests-cliarg-current.json`
    - [遒ｺ隱・縺九￥縺ｫ繧転: `summary.total = 9`, `summary.passed = 9`, `summary.failed = 0`
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/cli.js -i stdlib/std/env/cliarg.nepl -i stdlib/tests/cliarg.n.md -o html=/tmp/cliarg-doc-html`
    - [邨先棡/縺代▲縺犠: `generated 2 html file(s)`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - `std/env/cliarg` 縺ｮ focused regression 縺ｯ蠕ｩ譌ｧ縺励�～std` facade 縺ｮ縺・■ `stdio` / `fs` / `env/cliarg` 縺ｮ荳ｻ隕∝・蜿｣縺ｯ迴ｾ陦・move/effect 隕丞援縺ｫ霑ｽ蠕薙＠縺溘�・
  - todo 7 縺ｯ facade 蜈ｨ菴薙・謨ｴ蜷育｢ｺ隱阪→縲∝ｿ・ｦ√↑繧画ｮ九ｋ target 萓晏ｭ・API 縺ｮ謨ｴ逅・ｒ邯壹￠繧区ｮｵ髫弱↓蜈･縺｣縺溘�・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`std` facade 蜻ｨ霎ｺ縺ｮ隱ｬ譏弱・蜿ら・蜈医ｒ迴ｾ陦梧ｧ区・縺ｸ蜷梧悄)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - 逶ｴ霑代〒謠・∴縺・`std/stdio` / `std/fs` / `std/env/cliarg` 縺ｮ螳溯｣・↓蟇ｾ縺励�…omment / test 譁・擇 / docs 蛛ｴ縺ｮ蜿､縺・燕謠舌ｒ髯､蜴ｻ縺吶ｋ縲・
  - 螳溯｣・・騾壹▲縺ｦ縺・※繧ゅ�∬ｪｬ譏弱′譌ｧ讒区・縺ｮ縺ｾ縺ｾ縺�縺ｨ谺｡縺ｮ reboot 菴懈･ｭ縺ｧ隱､縺｣縺滓Φ螳壹ｒ蜀榊ｰ主・縺励ｄ縺吶＞縺溘ａ縲√％縺薙〒蜷梧悄縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `std/env/cliarg` 縺ｮ module comment 縺ｫ縺ｯ縲∽ｻ･蜑阪・螳溯｣・ｒ蠑輔″縺壹▲縺溘�悟叙蠕励＃縺ｨ縺ｫ繝｡繝｢繝ｪ繧堤｢ｺ菫昴＠縲∬ｧ｣謾ｾ縺励∪縺帙ｓ縲阪′谿九▲縺ｦ縺・◆縲・
  - `tests/stdlib/selfhost_req.n.md` 縺ｯ蟄伜惠縺励↑縺・`stdlib/tests/fs.nepl` 繧定ｦ∽ｻｶ遒ｺ隱阪・蜿ら・蜈医↓縺励※縺翫ｊ縲∫樟陦・repo 讒区・縺ｨ縺壹ｌ縺ｦ縺・◆縲・
  - `doc/testing.md` 繧よ立蜷・`std/cliarg` 縺ｨ譌ｧ `stdio` 隱ｬ譏弱ｒ谿九＠縺ｦ縺翫ｊ縲∫樟蝨ｨ縺ｮ `std/env/cliarg` / `stdio_read_all_bytes` 讒区・縺ｨ荳�閾ｴ縺励※縺・↑縺九▲縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/std/env/cliarg.nepl`
    - module comment 縺ｮ豕ｨ諢丈ｺ矩�・ｒ縲∬ｿ斐ｊ蛟､ `str` 縺ｯ譁ｰ隕冗｢ｺ菫昴＆繧後ｋ荳�譁ｹ縺ｧ蜀・Κ荳�譎ゅヰ繝・ヵ繧｡縺ｯ髢｢謨ｰ蜀・〒隗｣謾ｾ縺輔ｌ繧九�√→縺・≧迴ｾ陦悟ｮ溯｣・↓蜷医ｏ縺帙※譖ｴ譁ｰ縺励◆縲・
  - `tests/stdlib/selfhost_req.n.md`
    - file I/O 隕∽ｻｶ遒ｺ隱阪・蟇ｾ雎｡繝代せ繧偵�∝ｮ溷惠縺吶ｋ `stdlib/tests/fs.n.md` 縺ｸ螟画峩縺励◆縲・
  - `doc/testing.md`
    - `std/cliarg` 繧・`std/env/cliarg` 縺ｸ譖ｴ譁ｰ縺励◆縲・
    - `std/stdio` 縺ｮ隕∫ｴ・ｒ縲∝商縺・`read_all` / `read_line` 荳ｭ蠢・ｪｬ譏弱°繧峨�∫樟蝨ｨ縺ｮ `stdio_read_all_bytes` 繧貞性繧�讒区・縺ｸ譖ｴ譁ｰ縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - 縺薙・遞ｮ縺ｮ蟾ｮ蛻・・讖溯・霑ｽ蜉�縺ｧ縺ｯ縺ｪ縺・′縲〉eboot 荳ｭ縺ｯ縲悟商縺・ｪｬ譏弱′谿九ｋ縺薙→閾ｪ菴薙′荳榊・蜷医・蜈･蜿｣縲阪↓縺ｪ繧九◆繧√�∝ｮ溯｣・､画峩縺ｨ蜷後§蜆ｪ蜈亥ｺｦ縺ｧ謠・∴繧九∋縺阪→蛻､譁ｭ縺励◆縲・
  - `selfhost_req` 縺ｯ縲後◆縺ｾ縺溘∪騾壹ｋ蜿､縺・燕謠舌�阪ｒ谿九＆縺壹�∫樟蝨ｨ縺ｮ repo 縺ｫ蟄伜惠縺吶ｋ file 繧呈・遉ｺ逧・↓隱ｭ繧�蠖｢縺ｸ蟇・○縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tests/stdlib/selfhost_req.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/selfhost_req.n.md -i stdlib/tests/cliarg.n.md -i stdlib/std/env/cliarg.nepl -i stdlib/std/fs.nepl -i stdlib/std/stdio.nepl --no-stdlib --no-tree -o /tmp/tests-doc-followup.json -j 15`
    - [邨先棡/縺代▲縺犠: `47/47 pass`
  - `node nodesrc/tests.js -i tests/stdlib/streamio.n.md -i tests/stdlib/fs.n.md -i stdlib/tests/fs.n.md -i stdlib/tests/cliarg.n.md -i tests/stdlib/stdin.n.md -i tests/stdlib/stdout.n.md -i stdlib/std/streamio.nepl -i stdlib/std/stdio.nepl -i stdlib/std/fs.nepl -i stdlib/std/env/cliarg.nepl --no-stdlib --no-tree -o /tmp/tests-std-facade-sweep.json -j 15`
    - [邨先棡/縺代▲縺犠: `64/64 pass`
  - `node nodesrc/cli.js -i stdlib/std/env/cliarg.nepl -i tests/stdlib/selfhost_req.n.md -o html=/tmp/std-followup-doc-html`
    - [邨先棡/縺代▲縺犠: `generated 2 html file(s)`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - `std` facade 蜻ｨ霎ｺ縺ｮ螳溯｣・・comment繝ｻfocused test繝ｻ蛻ｩ逕ｨ閠・髄縺題｣懷勧 doc 縺ｮ蜑肴署縺御ｸ�閾ｴ縺励◆縲・
  - 谺｡谿ｵ縺ｧ縺ｯ `std` 譛ｬ菴薙・谿九ｊ target 萓晏ｭ・API 縺ｨ縲～features` / tutorials 蛛ｴ縺ｮ霑ｽ蠕鍋憾豕√ｒ隕九※縺・￥縲・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`features/tui` facade 繧定ｿｽ蜉�縺励�仝ASIX TUI API 繧・named struct 繝吶・繧ｹ縺ｸ謨ｴ逅・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - todo 8 縺ｮ `features` 螻､謨ｴ逅・→縺励※縲ゝUI 縺ｮ蛻ｩ逕ｨ閠・髄縺大・蜿｣繧・`platforms/wasix/tui` 逶ｴ蜿ら・縺九ｉ `features/tui` 縺ｫ蝗ｺ螳壹☆繧九�・
  - 譌ｧ `.Pair` 繝吶・繧ｹ縺ｮ蠎ｧ讓吶・繧ｵ繧､繧ｺ API 縺・current compiler / examples 縺ｧ荳榊ｮ牙ｮ壹↓縺ｪ縺｣縺ｦ縺・◆縺溘ａ縲｝ublic API 繧・named struct 繝吶・繧ｹ縺ｸ蟇・○繧九�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - repo 縺ｫ縺ｯ `stdlib/platforms/wasix/tui.nepl` 縺励°縺ｪ縺上�‘xamples 繧ょ・縺ｦ platform 逶ｴ import 縺励※縺・◆縺溘ａ縲〉eboot 譁ｹ驥昴・縲卦UI 縺ｯ `features` 螻､縲阪→縺・≧雋ｬ蜍吝・髮｢縺梧悴蜿肴丐縺�縺｣縺溘�・
  - 縺輔ｉ縺ｫ `get_terminal_size` 縺ｨ `editor_text::cursor_line_col` 縺・`.Pair` 繧定ｿ斐＠縲…all site 縺ｧ縺ｯ `get x 0` / `get x 1` 縺ｫ萓晏ｭ倥＠縺ｦ縺・◆縺後�［ulti-file 縺ｮ wasix examples 縺ｧ縺ｯ縺薙・邨瑚ｷｯ縺・`D3006` 繧定ｵｷ縺薙＠縺ｦ縺・◆縲・
  - `Tuple:` 閾ｪ菴薙ｒ謌ｻ繧雁�､縺ｫ菴ｿ縺・％縺ｨ縺ｧ縺ｯ縺ｪ縺上�√�悟､夜Κ API 縺ｨ helper 縺ｮ諢丞袖繧堤分蜿ｷ access 縺ｫ謚ｼ縺苓ｾｼ繧薙□縺薙→縲阪′荳榊ｮ牙ｮ壹＆縺ｨ蜿ｯ隱ｭ諤ｧ菴惹ｸ九・蜈ｱ騾壼次蝗�縺�縺｣縺溘�・
  - 縺ゅｏ縺帙※蝙区ｳｨ驥亥・縺ｮ `tui::TerminalSize` 縺ｮ繧医≧縺ｪ `::` path 縺ｯ迴ｾ迥ｶ parser 縺悟女縺台ｻ倥￠縺壹�∫ｷｨ髮・ｦ∵ｭ｢繝｡繝｢縺ｫ縺ゅｋ譛ｪ螳溯｣・�・岼縺ｨ陦晉ｪ√＠縺ｦ縺・◆縺溘ａ縲…all site 縺ｯ謗ｨ隲門燕謠舌↓縺吶ｋ蠢・ｦ√′縺ゅ▲縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/features/tui.nepl`
    - `platforms/wasix/tui` 繧・`@merge` 縺ｧ蜀榊・髢九☆繧句・蠑・facade 繧呈眠險ｭ縺励◆縲・
    - module comment 繧呈眠 policy 縺ｫ蜷医ｏ縺帙※險倩ｿｰ縺励�∝茜逕ｨ閠・髄縺・import path 繧・`features/tui` 縺ｫ蝗ｺ螳壹＠縺溘�・
  - `stdlib/platforms/wasix/tui.nepl`
    - `TerminalSize` struct 繧定ｿｽ蜉�縺励◆縲・
    - `get_terminal_size` 縺ｮ謌ｻ繧雁�､繧・`Tuple:` 縺九ｉ `TerminalSize` 縺ｸ螟画峩縺励◆縲・
    - parser error 縺ｮ蜴溷屏縺�縺｣縺・`if` layout 蜀・・荳崎ｦ√↑譛ｫ蟆ｾ `;` 3 邂・園繧帝勁蜴ｻ縺励◆縲・
  - `examples/tui_editor/editor_text.nepl`
    - `CursorLineCol` struct 繧定ｿｽ蜉�縺励�～cursor_line_col` 縺ｮ謌ｻ繧雁�､繧・named struct 蛹悶＠縺溘�・
  - `examples/tui_editor/editor_render.nepl`
    - `cursor_line_col` 縺ｮ蛻ｩ逕ｨ繧・`get p "line"` / `get p "col"` 縺ｫ螟画峩縺励◆縲・
  - `examples/wasix_tui_demo.nepl`
  - `examples/wasix_tui_fullscreen.nepl`
  - `examples/wasix_tui_menu.nepl`
  - `examples/wasix_tui_progress.nepl`
  - `examples/wasix_tui_text_render.nepl`
  - `examples/tui_editor/main.nepl`
  - `examples/tui_editor/editor_runtime.nepl`
  - 縺薙ｌ繧峨・ import 繧・`platforms/wasix/tui` 縺九ｉ `features/tui` 縺ｸ螟画峩縺励◆縲・
  - `examples/wasix_tui_demo.nepl` / `examples/wasix_tui_fullscreen.nepl` / `examples/wasix_tui_text_render.nepl` / `examples/tui_editor/main.nepl`
    - 遶ｯ譛ｫ繧ｵ繧､繧ｺ縺ｮ蜿ら・繧・`get size "cols"` / `get size "rows"` 縺ｫ螟画峩縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - TUI 縺ｮ facade 霑ｽ蜉�縺�縺代〒豁｢繧√★縲‘xample 縺ｾ縺ｧ `features/tui` 縺ｫ謠・∴縺溘・縺ｯ縲√�悟茜逕ｨ閠・′譛�蛻昴↓隕九ｋ path縲阪ｒ蝗ｺ螳壹＠縺ｪ縺・→ reboot 蠕後・雋ｬ蜍吝・髮｢縺悟ｮ夂捩縺励↑縺・◆繧√〒縺ゅｋ縲・
  - 遶ｯ譛ｫ繧ｵ繧､繧ｺ繧・cursor 蠎ｧ讓吶・ public helper 縺ｨ縺励※諢丞袖縺梧・遒ｺ縺ｪ縺ｮ縺ｧ縲∝諺蜷・tuple 繧医ｊ named struct 縺ｮ譁ｹ縺・API 縺ｨ縺励※螳牙ｮ壹〒縲’ield access 蟒・ｭ｢譁ｹ驥昴→繧よ紛蜷医☆繧九�・
  - 蝙区ｳｨ驥・path 譛ｪ蟇ｾ蠢懊・ compiler 蛛ｴ縺ｮ譛ｪ螳溯｣・ｺ矩�・↑縺ｮ縺ｧ縲∽ｻ雁屓縺ｯ library 蛛ｴ縺ｧ蝗樣∩荳崎・縺ｪ邂・園縺�縺・inference 縺ｫ蟇・○縲∵ｧ区枚諡｡蠑ｵ縺昴・繧ゅ・縺ｫ縺ｯ雕上∩霎ｼ縺ｾ縺ｪ縺九▲縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `target/debug/nepl-cli -i examples/wasix_tui_demo.nepl --target wasix --output /tmp/wasix-tui-demo-check` -> success
  - `target/debug/nepl-cli -i examples/tui_editor/main.nepl --target wasix --output /tmp/tui-editor-check` -> success
  - `target/debug/nepl-cli -i examples/wasix_tui_menu.nepl --target wasix --output /tmp/wasix-tui-menu-check` -> success
  - `target/debug/nepl-cli -i examples/wasix_tui_progress.nepl --target wasix --output /tmp/wasix-tui-progress-check` -> success
  - `target/debug/nepl-cli -i examples/wasix_tui_fullscreen.nepl --target wasix --output /tmp/wasix-tui-fullscreen-check` -> success
  - `target/debug/nepl-cli -i examples/wasix_tui_text_render.nepl --target wasix --output /tmp/wasix-tui-text-render-check` -> success
  - `node nodesrc/tui_regression.js --timeout-ms 8000`
    - [邨先棡/縺代▲縺犠: `ok: true`
    - [遒ｺ隱・縺九￥縺ｫ繧転: 蜈ｨ 16 scenario 縺・`exit_code = 0`
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/cli.js -i stdlib/features/tui.nepl -o html=/tmp/features-tui-doc-html`
    - [邨先棡/縺代▲縺犠: `generated 1 html file(s)`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - TUI 縺ｮ蛻ｩ逕ｨ閠・髄縺大・蜿｣縺ｯ `features/tui` 縺ｫ蝗ｺ螳壹＆繧後�》odo 8 縺ｮ縺・■ TUI 驟咲ｽｮ縺ｯ螳御ｺ・＠縺溘�・
  - `features` 螻､縺ｫ縺ｯ GUI / HTTP / 髻ｳ螢ｰ縺ｪ縺ｩ譛ｪ謨ｴ逅・・鬆伜沺縺梧ｮ九ｋ縺溘ａ縲》odo 8 閾ｪ菴薙・縲梧ｮ倶ｽ懈･ｭ謨ｴ逅・�阪→縺励※邯咏ｶ壹☆繧九�・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`features/tui` facade 縺ｮ focused regression 繧定ｿｽ蜉�)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - 逶ｴ蜑阪↓蜈･繧後◆ `features/tui` 縺ｸ縺ｮ遘ｻ陦後ｒ縲‘xamples compile 縺ｮ縺ｿ縺ｧ縺ｯ縺ｪ縺・`tests/stdlib` 縺ｮ focused case 縺ｨ縺励※蝗ｺ螳壹☆繧九�・
  - `platforms/wasix/tui` 逶ｴ蜿ら・縺ｸ縺ｮ騾・綾繧翫ｄ縲～TerminalSize` 縺ｮ field access 騾�陦後ｒ蟆上＆縺ｪ fixture 縺ｧ譌ｩ譛滓､懃衍縺ｧ縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - 逶ｴ蜑阪・螟画峩縺ｯ examples compile 縺ｨ runtime regression 縺ｧ縺ｯ遒ｺ隱阪〒縺阪※縺・◆縺後�《tdlib reboot 縺ｮ譛ｬ豬√〒菴ｿ縺・`tests/stdlib/*` 蛛ｴ縺ｫ蟆ら畑 fixture 縺悟ｭ伜惠縺励↑縺九▲縺溘�・
  - 縺昴・縺ｾ縺ｾ縺�縺ｨ縲∝ｰ・擂 `features/tui` facade 縺ｮ reexport 縺悟ｴｩ繧後※繧ゅ�・㍾縺・wasix example 繧貞�句挨縺ｫ蝗槭☆縺ｾ縺ｧ豌励▼縺代↑縺・憾諷九□縺｣縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tests/stdlib/features_tui.n.md`
    - `features_tui_facade_reexports_text_helpers` 繧定ｿｽ蜉�縺励�～features/tui` 邨檎罰縺ｧ `line_pad_to_cols` 縺ｨ `repeat_text` 縺御ｽｿ縺医ｋ縺薙→繧・stdout 縺ｧ蝗ｺ螳壹＠縺溘�・
    - `features_tui_terminal_size_uses_named_fields` 繧定ｿｽ蜉�縺励�～get_terminal_size` 縺ｮ謌ｻ繧雁�､縺ｫ蟇ｾ縺励※ `get size "cols"` / `"rows"` 縺御ｽｿ縺医ｋ縺薙→繧・`ret: 0` 縺ｧ蝗ｺ螳壹＠縺溘�・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - TTY 繧貞燕謠舌→縺吶ｋ raw mode 繧・full-screen 謠冗判縺ｯ驥阪￥螢翫ｌ譁ｹ繧ょ､壽ｧ倥↑縺ｮ縺ｧ縲’ocused regression 縺ｧ縺ｯ縲卦TY 縺ｪ縺励〒繧ょ・迴ｾ縺ｧ縺阪ｋ helper縲阪→縲系amed field access縲阪・ 2 轤ｹ縺ｫ雋ｬ蜍吶ｒ邨槭▲縺溘�・
  - 縺薙ｌ縺ｫ繧医ｊ縲～features/tui` facade 縺ｮ螂醍ｴ・擇縺・examples 繧医ｊ遏ｭ縺・・迴ｾ縺ｧ讀懆ｨｼ縺ｧ縺阪ｋ繧医≧縺ｫ縺ｪ縺｣縺溘�・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`features/tui` focused test 繧帝�壹☆縺溘ａ縺ｫ library / nodesrc 縺ｮ wasix 邨瑚ｷｯ繧呈弍豁｣)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - 霑ｽ蜉�縺励◆ `tests/stdlib/features_tui.n.md` 繧・`run_doctest.js` / `tests.js` 縺九ｉ螳牙ｮ壹＠縺ｦ螳溯｡後〒縺阪ｋ繧医≧縺ｫ縺吶ｋ縲・
  - `features/tui` facade 縺ｮ focused regression 繧偵�梧焔蜈・〒蛟句挨縺ｫ wasmer 繧貞娼縺代・騾壹ｋ縲咲憾諷九〒縺ｯ縺ｪ縺上�∵里蟄・nodesrc harness 縺ｧ蜀咲樟縺ｧ縺阪ｋ迥ｶ諷九↓謌ｻ縺吶�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `stdlib/platforms/wasix/tui.nepl` 縺ｯ module 蜀・〒 `print` / `print_i32` 繧剃ｽｿ縺｣縺ｦ縺・◆縺後�～std/stdio` 繧・import 縺励※縺翫ｉ縺壹�∝他縺ｳ蜃ｺ縺怜・ module 縺後◆縺ｾ縺溘∪ `std/stdio` 繧・import 縺励※縺・ｋ蜑肴署縺ｫ萓晏ｭ倥＠縺ｦ縺・◆縲・
  - 縺昴・縺溘ａ縲～features/tui` 縺�縺代ｒ import 縺吶ｋ focused test 縺ｧ縺ｯ `undefined identifier` 縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
  - 縺輔ｉ縺ｫ `nodesrc/run_test.js` 縺ｯ螳溯｡檎ｳｻ繧・Node.js 縺ｮ WASI preview1 縺ｫ蝗ｺ螳壹＠縺ｦ縺翫ｊ縲～#target wasix` doctest 繧貞ｮ溯｡後☆繧九→ `wasix_32v1` import 繧定ｧ｣豎ｺ縺ｧ縺阪↑縺九▲縺溘�・
  - `spawnSync wasmer` 縺ｸ蛻・ｊ譖ｿ縺医◆蛻晄悄譯医ｂ sandbox 荳九〒 `EPERM` 繧定ｵｷ縺薙＠縺溘◆繧√�『asix 螳溯｡檎ｵ瑚ｷｯ縺ｯ `tui_regression.js` 縺ｨ蜷後§ async `spawn` 縺ｸ謠・∴繧句ｿ・ｦ√′縺ゅ▲縺溘�・
  - 縺ゅｏ縺帙※ `wasmer run --dir=...` 縺ｮ deprecated warning 縺・stderr 繧呈ｱ壹＠縺ｦ縺翫ｊ縲！/O 豈碑ｼ・ｳｻ test 縺ｮ蟆・擂繝ｪ繧ｹ繧ｯ縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/platforms/wasix/tui.nepl`
    - `#import "std/stdio" as *` 繧定ｿｽ蜉�縺励�［odule 蜊倅ｽ薙〒 `print` 邉ｻ symbol 繧定ｧ｣豎ｺ縺ｧ縺阪ｋ繧医≧縺ｫ縺励◆縲・
  - `nodesrc/run_test.js`
    - source 縺九ｉ `#target` 繧定ｪｭ縺ｿ蜿悶ｊ縲～wasix` 縺ｮ蝣ｴ蜷医・ `runWasixBytes` 繧剃ｽｿ縺・・蟯舌ｒ霑ｽ蜉�縺励◆縲・
    - `runWasixBytes` 繧・async `spawn` 繝吶・繧ｹ縺ｧ螳溯｣・＠縲《tdin / stdout / stderr capture 縺ｨ timeout 繧呈戟縺､豎守畑 wasix 螳溯｡檎ｵ瑚ｷｯ縺ｫ縺励◆縲・
    - `wasmer run` 縺ｮ mount option 繧・`--dir` 縺九ｉ `--volume host:guest` 縺ｸ譖ｴ譁ｰ縺励�‥eprecated warning 繧帝勁蜴ｻ縺励◆縲・
  - `nodesrc/tui_regression.js`
    - 蜷後§縺・`--volume` 縺ｸ譖ｴ譁ｰ縺励�《cenario 螳溯｡梧凾縺ｮ stderr warning 繧帝勁蜴ｻ縺励◆縲・
  - `nodesrc/README.n.md`
    - `run_test.js` 縺・`#target wasix` 縺ｧ縺ｯ `wasmer run` 繧剃ｽｿ縺・％縺ｨ縺ｨ縲～WASMER_BIN` 縺ｧ override 縺ｧ縺阪ｋ縺薙→繧定ｿｽ險倥＠縺溘�・
  - `tests/stdlib/features_tui.n.md`
    - 霑ｽ蜉�貂医∩ focused test 繧呈ｭ｣蠑上↓蝗槫ｸｰ縺ｸ邨・∩霎ｼ繧薙□縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `platforms/wasix/tui` 縺ｮ繧医≧縺ｪ feature backend 縺ｯ縲∝他縺ｳ蜃ｺ縺怜・ import 縺ｫ萓晏ｭ倥○縺・self-contained 縺ｫ縺励※縺翫￥縺ｹ縺阪↑縺ｮ縺ｧ縲》est 蛛ｴ縺ｸ `std/stdio` 繧定ｶｳ縺吶・縺ｧ縺ｯ縺ｪ縺・library 蛛ｴ繧剃ｿｮ豁｣縺励◆縲・
  - wasix 螳溯｡後・ Node.js 讓呎ｺ・WASI 縺ｧ縺ｯ譛ｬ雉ｪ逧・↓謇ｱ縺医↑縺・◆繧√�》est harness 蛛ｴ縺ｧ target 蛻・ｲ舌ｒ謖√▽縺ｮ縺梧�ｹ譛ｬ菫ｮ豁｣縺ｨ蛻､譁ｭ縺励◆縲・
  - `tui_regression.js` 縺ｨ `run_test.js` 縺ｮ螳溯｡梧婿蠑上ｒ謠・∴縺溘％縺ｨ縺ｧ縲’ocused test 縺ｨ end-to-end regression 縺ｮ蟾ｮ縺梧ｸ帙▲縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tests/stdlib/features_tui.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/features_tui.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md -i stdlib/features/tui.nepl -i stdlib/platforms/wasix/tui.nepl --no-stdlib --no-tree -o /tmp/tests-features-tui.json -j 15`
    - [邨先棡/縺代▲縺犠: `3/3 pass`
  - `/tmp/tests-features-tui.json`
    - [遒ｺ隱・縺九￥縺ｫ繧転: `summary.total = 3`, `summary.passed = 3`, `summary.failed = 0`
  - `node nodesrc/tui_regression.js --timeout-ms 8000`
    - [邨先棡/縺代▲縺犠: `ok: true`
    - [遒ｺ隱・縺九￥縺ｫ繧転: 蜈ｨ 16 scenario 縺・`exit_code = 0`, `stderr_len = 0`
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/cli.js -i stdlib/features/tui.nepl -i tests/stdlib/features_tui.n.md -o html=/tmp/features-tui-tests-doc-html`
    - [邨先棡/縺代▲縺犠: `generated 2 html file(s)`
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - `features/tui` 縺ｯ facade 縺ｨ examples 縺�縺代〒縺ｪ縺上�’ocused doctest harness 縺九ｉ繧よ､懆ｨｼ縺ｧ縺阪ｋ迥ｶ諷九↓縺ｪ縺｣縺溘�・
  - nodesrc 蛛ｴ縺ｯ `#target wasix` 繧呈桶縺医ｋ繧医≧縺ｫ縺ｪ繧翫�∽ｻ雁ｾ後・ `features` 邉ｻ蝗槫ｸｰ霑ｽ蜉�縺ｧ繧ょ酔縺倡ｵ瑚ｷｯ繧貞・蛻ｩ逕ｨ縺ｧ縺阪ｋ縲・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`web/package.json` 繧・ESM 蛹悶＠縺ｦ nodesrc 縺ｮ module type warning 繧帝勁蜴ｻ)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `nodesrc` 螳溯｡梧凾縺ｫ豈主屓蜃ｺ縺ｦ縺・◆ `[MODULE_TYPELESS_PACKAGE_JSON]` warning 繧帝勁蜴ｻ縺励�》est 縺ｮ signal 繧定ｦ九ｄ縺吶￥縺吶ｋ縲・
  - `compiler_loader.js` 縺・`web/dist/nepl-web-*.js` 繧・ESM 縺ｨ縺励※ dynamic import 縺励※縺・ｋ蜑肴署繧・package scope 蛛ｴ縺ｧ繧よ・遉ｺ縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `nodesrc/compiler_loader.js` 縺ｯ wasm-bindgen 逕滓・迚ｩ縺ｮ `nepl-web-*.js` 繧・dynamic import 縺励※縺・ｋ縺後�∬ｦｪ繝・ぅ繝ｬ繧ｯ繝医Μ縺ｧ縺ゅｋ `web/` 縺ｮ `package.json` 縺ｫ `"type": "module"` 縺後↑縺九▲縺溘�・
  - 縺昴・縺溘ａ Node.js 縺ｯ荳�譌ｦ CommonJS 縺ｨ縺励※隗｣驥医＠繧医≧縺ｨ縺励※縺九ｉ ESM 縺ｨ縺励※蜀崎ｧ｣驥医＠縲～run_doctest.js` / `tests.js` / `cli.js` 螳溯｡梧凾縺ｫ豈主屓 warning 繧貞・縺励※縺・◆縲・
  - warning 閾ｪ菴薙・螟ｱ謨励〒縺ｯ縺ｪ縺・′縲’ocused test 縺ｮ stderr 繧呈ｱ壹＠縲”arness 謾ｹ菫ｮ譎ゅ・譛ｬ蠖薙・逡ｰ蟶ｸ縺ｨ隕句・縺代↓縺上￥縺ｪ縺｣縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `web/package.json`
    - `"type": "module"` 繧定ｿｽ蜉�縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - 蝠城｡後・ loader 蛛ｴ縺ｧ縺ｯ縺ｪ縺・package scope 縺ｮ螳｣險�荳崎ｶｳ縺ｪ縺ｮ縺ｧ縲『arning 繧・suppress 縺吶ｋ縺ｮ縺ｧ縺ｯ縺ｪ縺・package metadata 繧貞ｮ滓・縺ｫ蜷医ｏ縺帙ｋ縺ｮ縺梧�ｹ譛ｬ菫ｮ豁｣縺ｨ蛻､譁ｭ縺励◆縲・
  - `web/` 驟堺ｸ九・ Node tool 縺ｯ荳ｻ縺ｫ `tsc` / `trunk` 邨檎罰縺ｧ菴ｿ縺｣縺ｦ縺翫ｊ縲・SM 謖・ｮ壹ｒ霑ｽ蜉�縺励※繧ら樟陦碁°逕ｨ縺ｨ遏帷崟縺励↑縺・％縺ｨ繧・build 縺ｧ遒ｺ隱阪＠縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build` -> success
  - `node nodesrc/run_doctest.js -i tests/stdlib/features_tui.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md -i stdlib/features/tui.nepl -i stdlib/platforms/wasix/tui.nepl --no-stdlib --no-tree -o /tmp/tests-features-tui.json -j 15`
    - [邨先棡/縺代▲縺犠: `3/3 pass`
  - [遒ｺ隱・縺九￥縺ｫ繧転:
    - 荳願ｨ伜ｮ溯｡後°繧・`[MODULE_TYPELESS_PACKAGE_JSON]` warning 縺梧ｶ医∴縺溘�・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`doc/testing.md` 繧堤樟陦・nodesrc / reboot 驕狗畑縺ｸ蜈ｨ髱｢蜷梧悄)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - test 驕狗畑縺ｮ隱ｬ譏弱′譌ｧ `cargo run -p nepl-cli -- test` 荳ｭ蠢・・縺ｾ縺ｾ谿九▲縺ｦ縺・◆縺溘ａ縲∫樟蝨ｨ縺ｮ `nodesrc` 繝吶・繧ｹ驕狗畑縺ｸ蜷梧悄縺吶ｋ縲・
  - reboot 荳ｭ縺ｫ譁ｰ縺励＞蝗槫ｸｰ繧定ｿｽ蜉�縺吶ｋ莠ｺ縺後�～tests/stdlib` / `stdlib/tests` / doc comment doctest 縺ｮ蠖ｹ蜑ｲ繧貞叙繧企＆縺医↑縺・ｈ縺・↓縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `doc/testing.md` 縺ｫ縺ｯ蜿､縺・stdlib 隕∫ｴ・�∵立 tuple 險俶ｳ輔�∵立 test runner 蜑肴署縺梧ｮ九▲縺ｦ縺翫ｊ縲∫樟蝨ｨ縺ｮ repo 讒区・縺ｨ荳�閾ｴ縺励※縺・↑縺九▲縺溘�・
  - 迚ｹ縺ｫ `nodesrc/run_test.js` 縺ｮ wasix 蟇ｾ蠢懊′蜈･縺｣縺溷ｾ後ｂ縲√◎縺ｮ runtime 蛻・ｲ舌ｄ `run_doctest.js` / `tests.js` 荳ｭ蠢・・驕狗畑縺梧枚譖ｸ蛹悶＆繧後※縺・↑縺九▲縺溘�・
  - 縺昴・縺ｾ縺ｾ縺�縺ｨ縲∽ｻ翫・螳溯｣・ｒ蜑肴署縺ｫ test 繧定ｿｽ蜉�縺励ｈ縺・→縺励◆縺ｨ縺阪↓縲・俣驕輔▲縺溷・蜿｣繧・・鄂ｮ蜈医ｒ蜀榊ｰ主・縺吶ｋ繝ｪ繧ｹ繧ｯ縺後≠縺｣縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `doc/testing.md`
    - 譁・嶌蜈ｨ菴薙ｒ current workflow 縺ｫ蜷医ｏ縺帙※譖ｸ縺咲峩縺励◆縲・
    - `tests/compiler/*.n.md`縲～tests/stdlib/*.n.md`縲～stdlib/tests/*.n.md`縲～stdlib/**/*.nepl` doctest縲～tutorials/**/*.n.md` 縺ｮ蠖ｹ蜑ｲ繧呈紛逅・＠縺溘�・
    - 謗ｨ螂ｨ繧ｳ繝槭Φ繝峨ｒ `nodesrc/tests.js` / `run_doctest.js` / `cli.js` / `trunk build` 縺ｫ譖ｴ譁ｰ縺励◆縲・
    - `run_test.js` 縺・`#target wasix` 繧・`wasmer run` 縺ｧ螳溯｡後☆繧九％縺ｨ繧呈・險倥＠縺溘�・
    - 蜿､縺・tuple 險俶ｳ戊ｪｬ譏弱→縲∫樟迥ｶ縺ｫ蜷医ｏ縺ｪ縺・stdlib 荳�隕ｧ繧貞炎髯､縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `doc/testing.md` 縺ｯ detailed API reference 縺ｧ縺ｯ縺ｪ縺上�後←縺薙↓菴輔ｒ譖ｸ縺上°縲√←縺・ｮ溯｡後☆繧九°縲阪・驕狗畑譁・嶌縺ｪ縺ｮ縺ｧ縲∝・謖吝梛縺ｮ stdlib 繧ｫ繧ｿ繝ｭ繧ｰ縺ｧ縺ｯ縺ｪ縺・workflow 荳ｭ蠢・↓蜀肴ｧ区・縺励◆縲・
  - docs 縺ｮ蠖ｹ蜑ｲ荳翫�√％縺薙〒縺ｯ `.md` 蛻ｶ邏・↓蠕薙＞ ruby 縺ｯ菴ｿ繧上★縲∫ｰ｡貎斐↑ plain markdown 縺ｫ謠・∴縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/cli.js -i doc/testing.md -o html=/tmp/doc-testing-html`
    - [邨先棡/縺代▲縺犠: `generated 0 html file(s)`
    - [遒ｺ隱・縺九￥縺ｫ繧転: `.md` 縺ｯ HTML 逕滓・蟇ｾ雎｡螟悶〒縺ゅｊ縲∫焚蟶ｸ縺ｧ縺ｯ縺ｪ縺・�・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`tutorials/getting_started` 縺ｮ std entrypoint 縺ｸ遘ｻ陦・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - getting started tutorial 縺悟商縺・`#target wasi` 繧呈蕗縺医※縺・◆縺溘ａ縲〉eboot 蠕後・蜈ｬ髢句・蜿｣縺ｧ縺ゅｋ `#target std` 縺ｫ謠・∴繧九�・
  - 蛻晏ｭｦ閠・髄縺第枚譖ｸ縺悟・驛ｨ runtime 蜷阪〒縺ｯ縺ｪ縺・std facade 繧定ｵｷ轤ｹ縺ｫ隱ｬ譏弱☆繧九ｈ縺・↓縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `std/stdio` 縺ｪ縺ｩ縺ｮ蛻ｩ逕ｨ萓九′縺吶〒縺ｫ std facade 蜑肴署縺ｫ謨ｴ逅・＆繧後※縺・ｋ荳�譁ｹ縲》utorial 縺ｮ doctest 縺�縺第立 `wasi` target 縺ｮ縺ｾ縺ｾ谿九▲縺ｦ縺・◆縲・
  - 縺昴・縺溘ａ縲∫樟蝨ｨ縺ｮ險ｭ險亥憧蟄ｦ縺ｧ縺ゅｋ縲悟茜逕ｨ閠・・ raw platform 縺ｧ縺ｯ縺ｪ縺・std/features 繧貞・蜿｣縺ｫ縺吶ｋ縲阪→譁・嶌縺後★繧後※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tutorials/getting_started/01_hello_world.n.md`
    - 蜀帝�ｭ隱ｬ譏弱ｒ `#target std` 蜑肴署縺ｸ譖ｴ譁ｰ縺励◆縲・
    - 譛�蛻昴↓縺､縺ｾ縺壹″繧・☆縺・せ縺ｮ bullet 繧・`#target std` 縺ｫ蜷梧悄縺励◆縲・
  - `tutorials/getting_started/02_numbers_and_variables.n.md`
  - `tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md`
  - `tutorials/getting_started/03_functions.n.md`
  - `tutorials/getting_started/04_strings_and_stdio.n.md`
  - `tutorials/getting_started/05_option.n.md`
  - `tutorials/getting_started/06_result.n.md`
  - `tutorials/getting_started/07_while_and_block.n.md`
  - `tutorials/getting_started/08_if_layouts.n.md`
  - `tutorials/getting_started/09_import_and_structure.n.md`
  - `tutorials/getting_started/10_project_fizzbuzz.n.md`
  - `tutorials/getting_started/11_testing_workflow.n.md`
  - `tutorials/getting_started/12_pure_function_pipeline.n.md`
  - `tutorials/getting_started/13_type_driven_error_modeling.n.md`
  - `tutorials/getting_started/14_refactor_with_properties.n.md`
  - `tutorials/getting_started/15_match_patterns.n.md`
  - `tutorials/getting_started/16_debug_and_ansi.n.md`
  - `tutorials/getting_started/17_namespace_and_alias.n.md`
  - `tutorials/getting_started/18_recursion_and_termination.n.md`
  - `tutorials/getting_started/19_pipe_operator.n.md`
  - `tutorials/getting_started/20_generics_basics.n.md`
  - `tutorials/getting_started/21_trait_bounds_basics.n.md`
    - doctest 蜀・・ `#target wasi` 繧・`#target std` 縺ｫ譖ｴ譁ｰ縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - tutorial 縺ｯ蜀・Κ target 蜷阪ｒ謨吶∴繧句�ｴ謇�縺ｧ縺ｯ縺ｪ縺上�∝茜逕ｨ閠・′譛�蛻昴↓隗ｦ繧後ｋ public entrypoint 繧堤､ｺ縺吶∋縺阪↑縺ｮ縺ｧ縲～std` 縺ｸ謠・∴繧九・縺碁←蛻・→蛻､譁ｭ縺励◆縲・
  - 螟画峩縺ｯ tutorial 蜀・・ target 謖・ｮ壹→隱ｬ譏取枚縺�縺代↓髯仙ｮ壹＠縲√し繝ｳ繝励Ν譛ｬ菴薙・讒矩��繧・import 縺ｯ荳崎ｦ√↓隗ｦ繧峨↑縺九▲縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `rg -n "#target wasi|WASI 繧ｿ繝ｼ繧ｲ繝・ヨ|target wasi" tutorials/getting_started --glob '*.n.md'`
    - [邨先棡/縺代▲縺犠: 隧ｲ蠖薙↑縺・
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/01_hello_world.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/04_strings_and_stdio.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/09_import_and_structure.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/11_testing_workflow.n.md -n 1` -> pass

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`result` / `nm/parser` doctest failure 縺ｮ譬ｹ譛ｬ菫ｮ豁｣)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - old failure list 縺ｫ縺ゅ▲縺・`result.nepl doctest#5` 縺ｨ `parser.nepl doctest#2/#3` 繧偵�∫樟蝨ｨ縺ｮ莉墓ｧ倥→辣ｧ繧峨＠縺ｦ譬ｹ譛ｬ縺九ｉ逶ｴ縺吶�・
  - `parser` 蛻ｩ逕ｨ蛛ｴ縺ｮ `nm.n.md` 縺ｨ `html_gen` 縺ｾ縺ｧ focused 縺ｫ遒ｺ隱阪＠縲∝ｱ�謇�菫ｮ豁｣縺ｧ邨ゅｏ縺｣縺ｦ縺・↑縺・％縺ｨ繧堤｢ｺ縺九ａ繧九�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `stdlib/core/result.nepl`
    - `uwok` 縺ｮ菴ｿ逕ｨ萓九′譌ｧ pipe 隗｣驥医ｒ蜑肴署縺ｫ `assert_eq_i32 1 ok<i32, str> 1 |> uwok;` 縺ｨ譖ｸ縺九ｌ縺ｦ縺翫ｊ縲∫樟陦・parser 縺ｧ縺ｯ `assert_eq_i32` 蜻ｼ縺ｳ蜃ｺ縺励・騾比ｸｭ縺ｫ pipe 繧貞ｷｮ縺苓ｾｼ繧√↑縺・◆繧・`D3006` / `D3013` 縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
    - 縺薙ｌ縺ｯ compiler bug 縺ｧ縺ｯ縺ｪ縺上�‥octest 縺ｮ譁・ｳ募燕謠舌′蜿､縺九▲縺溘�・
  - `stdlib/nm/parser.nepl`
    - `close_one_section` / `close_to_level` / `close_all_sections` 縺ｯ `stack_push` 縺ｫ繧医ｊ `Stack<NestSection>` 繧呈峩譁ｰ縺吶ｋ縺ｮ縺ｫ縲｝ure signature 縺ｮ縺ｾ縺ｾ谿九▲縺ｦ縺・◆縲・
    - 縺昴・邨先棡縲［odule compile 譎ゅ↓ `D3025 pure context cannot call impure function` 縺ｨ `D3016` 縺檎匱逕溘＠縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/core/result.nepl`
    - `uwok` doctest 繧・`assert_eq_i32 1 uwok ok<i32, str> 1;` 縺ｫ譖ｴ譁ｰ縺励�∫樟陦・syntax 縺ｧ alias 縺ｮ諢丞袖縺御ｼ昴ｏ繧倶ｾ九↓蟾ｮ縺玲崛縺医◆縲・
  - `stdlib/nm/parser.nepl`
    - `close_one_section`
    - `close_to_level`
    - `close_all_sections`
      - signature 繧・`*>Vec<Node>` 縺ｫ譖ｴ譁ｰ縺励�～Stack` 譖ｴ譁ｰ繧定｡後≧ helper 縺ｨ縺励※ effect 繧呈・遉ｺ縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `result` 縺ｧ縺ｯ parser 繧堤ｷｩ繧√ｋ縺ｮ縺ｧ縺ｯ縺ｪ縺上�∫樟蝨ｨ縺ｮ險�隱樔ｻ墓ｧ倥↓蜷医≧ doctest 縺ｸ譖ｴ譁ｰ縺吶ｋ縺ｮ縺梧ｭ｣縺励＞縲・
  - `parser` 縺ｧ縺ｯ `stack_push` 繧帝國縺励※ pure 繧定｣・≧繧医ｊ縲”elper 閾ｪ霄ｫ繧・impure 縺ｨ譏守､ｺ縺吶ｋ譁ｹ縺・effect model 縺ｫ謨ｴ蜷医☆繧九�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/core/result.nepl -n 5` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/nm/parser.nepl -n 2` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/nm/parser.nepl -n 3` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/nm/html_gen.nepl -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/nm.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/nm.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/nm.n.md -i stdlib/nm/parser.nepl -i stdlib/nm/html_gen.nepl -i stdlib/core/result.nepl --no-stdlib --no-tree -o /tmp/tests-nm-result-focus.json -j 15`
    - [邨先棡/縺代▲縺犠: `12/12 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`move_check` 縺ｨ `vec/sort` 縺ｮ stale test 繧堤樟陦御ｻ墓ｧ倥∈蜷梧悄)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - old failure list 縺ｮ縺・■縲～move_check.n.md` 縺ｨ `sort.nepl doctest#3` 繧・focused 縺ｫ蜀咲樟縺励�∫樟蝨ｨ縺ｮ move model / collection API 縺ｫ蜷医ｏ縺帙※逶ｴ縺吶�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `tests/compiler/move_check.n.md`
    - 繝ｭ繝ｼ繧ｫ繝ｫ髱曚opy蝙九・蝗槫ｸｰ縺・`RegionToken` 縺ｨ縺・≧蜷榊燕縺ｮ縺ｾ縺ｾ std/prelude 譁・ц縺ｧ譖ｸ縺九ｌ縺ｦ縺翫ｊ縲∫樟陦・stdlib 縺ｮ `core/mem` 蛛ｴ `RegionToken<.T>` 縺ｨ陦晉ｪ√＠縺ｦ縺・◆縲・
    - 縺昴・邨先棡縲∵悽譚･隕九◆縺・move check 縺ｧ縺ｯ縺ｪ縺・constructor 隗｣譫先凾轤ｹ縺ｮ `D3016` 縺ｫ豬√ｌ縺ｦ縺・◆縲・
  - `stdlib/alloc/collections/vec/sort.nepl`
    - `sort_merge` 縺ｮ doctest 縺後�∝商縺・`Result` 霑泌唆蜑肴署縺ｧ `push ... |> uwok` 縺ｨ譖ｸ縺九ｌ縺ｦ縺・◆縲・
    - 迴ｾ陦後・ `Vec::push` 縺ｯ `Vec` 繧偵◎縺ｮ縺ｾ縺ｾ霑斐☆縺溘ａ縲｝ipe 縺ｮ騾比ｸｭ縺ｧ `uwok` 繧呈検繧�縺ｨ `D3006` / `D3013` 縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tests/compiler/move_check.n.md`
    - 蜷・snippet 繧・`#target core` 縺ｫ謠・∴縺溘�・
    - 繝ｭ繝ｼ繧ｫ繝ｫ蝙句錐繧・`RegionToken` 縺九ｉ `LocalToken` 縺ｸ螟画峩縺励�｝relude / stdlib 蜷阪→縺ｮ陦晉ｪ√ｒ驕ｿ縺代◆縲・
    - 髢｢騾｣縺吶ｋ field / borrow / consume / reassign 縺ｮ蝙区ｳｨ驥医ｂ蜷梧凾縺ｫ譖ｴ譁ｰ縺励◆縲・
  - `stdlib/alloc/collections/vec/sort.nepl`
    - `sort_merge` 縺ｮ菴ｿ逕ｨ萓九ｒ `new<i32> |> push ...` 蠖｢蠑上∈譖ｴ譁ｰ縺励�∽ｸ崎ｦ√↑ `uwok` 繧帝勁蜴ｻ縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `move_check` 縺ｯ compiler 縺ｮ move rule 繧呈ｸｬ繧句屓蟶ｰ縺ｪ縺ｮ縺ｧ縲《tdlib 蜷阪ｄ prelude 蠖ｱ髻ｿ繧貞女縺代ｋ迥ｶ諷九・縺ｾ縺ｾ縺ｫ縺帙★縲～#target core` + 繝ｭ繝ｼ繧ｫ繝ｫ蝙句錐縺ｧ髫秘屬縺吶ｋ縺ｮ縺碁←蛻・→蛻､譁ｭ縺励◆縲・
  - `sort` 縺ｯ API 繧呈・縺ｮ `Result` 蠖｢縺ｸ謌ｻ縺吶・縺ｧ縺ｯ縺ｪ縺上�∫樟蝨ｨ縺ｮ `Vec` chaining API 縺ｫ doctest 繧貞粋繧上○繧九・縺梧ｭ｣縺励＞縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tests/compiler/move_check.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i tests/compiler/move_check.n.md --no-stdlib --no-tree -o /tmp/tests-move-check.json -j 15`
    - [邨先棡/縺代▲縺犠: `13/13 pass`
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec/sort.nepl -n 3` -> pass
  - `node nodesrc/tests.js -i tests/compiler/move_check.n.md -i stdlib/alloc/collections/vec/sort.nepl --no-stdlib --no-tree -o /tmp/tests-move-sort-focus.json -j 15`
    - [邨先棡/縺代▲縺犠: `16/16 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`alloc/diag` 繧・move model 縺ｸ霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - old failure list 縺ｫ縺ゅ▲縺・`diag.n.md` / `error.n.md` 邉ｻ縺ｮ failure 繧堤樟陦・move model 縺ｧ蜀咲樟縺励�～alloc/diag` 縺ｮ蛟､繝｢繝・Ν螳溯｣・→ test 繧呈紛逅・☆繧九�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `stdlib/alloc/diag/diag.nepl`
    - `Diag` / `DiagKind` / `Vec<str>` 繧・`get` 繧・`vec_get` 縺ｧ菴募ｺｦ繧ょ盾辣ｧ縺吶ｋ譌ｧ螳溯｣・′谿九▲縺ｦ縺翫ｊ縲∫樟陦後・謇�譛画ｨｩ隗｣譫舌〒縺ｯ moved value 縺ｨ蛻､螳壹＆繧後※縺・◆縲・
    - 迚ｹ縺ｫ `diag_to_string` / `kind_str` / `diags_to_string_loop` 縺ｯ縲悟酔縺・owner 繧剃ｽ募ｺｦ繧りｪｭ繧�縲榊燕謠舌〒譖ｸ縺九ｌ縺ｦ縺・◆縲・
  - `stdlib/alloc/diag/error.nepl`
    - `diag_with_span` / `diag_with_source` / `diag_add_note` / `diag_add_help` 縺・`Diag` 繧貞・讒狗ｯ峨☆繧九→縺阪↓縲∝酔縺・`Diag` 縺九ｉ隍・焚 field 繧堤峩謗･蜿悶ｊ逶ｴ縺励※縺・◆縲・
    - `diags_has_errors_loop` 繧・`Vec<Diag>` 繧貞・蟶ｰ縺ｧ蜀榊茜逕ｨ縺励※縺翫ｊ縲∝酔縺伜撫鬘後ｒ謚ｱ縺医※縺・◆縲・
  - `stdlib/tests/error.n.md`
    - `Diag` / `Diags` / `Outcome` 繧剃ｸ�蠎ｦ `get` / helper 縺ｫ貂｡縺励◆縺ゅ→繧ょ酔縺伜�､繧貞・蛻ｩ逕ｨ縺吶ｋ縲∵立 move model 蜑肴署縺ｮ test 縺梧ｮ九▲縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/diag/diag.nepl`
    - `core/mem` 繧貞ｰ主・縺励◆縲・
    - `kind_str` 縺ｨ `diag_to_string` 繧・temporary memory 邨檎罰縺ｧ field 繧定ｪｭ縺ｿ蜃ｺ縺吝ｽ｢縺ｫ螟画峩縺励◆縲・
    - `diag_lines_loop` / `diag_help_loop` / `diags_to_string_loop` 縺ｯ `Vec` 蜈ｨ菴薙ｒ蜀榊ｸｰ縺ｧ謖√■蝗槭☆縺ｮ繧偵ｄ繧√�～data_ptr + len + index` 縺ｧ襍ｰ譟ｻ縺吶ｋ蠖｢縺ｫ螟画峩縺励◆縲・
  - `stdlib/alloc/diag/error.nepl`
    - `core/mem` 繧貞ｰ主・縺励◆縲・
    - `diag_with_span` / `diag_with_source` / `diag_add_note` / `diag_add_help` 繧・temporary memory 邨檎罰縺ｮ蜀肴ｧ狗ｯ峨↓螟画峩縺励◆縲・
    - `diags_has_errors` / `diags_has_errors_loop` 繧・`Vec<Diag>` 繧・raw data 襍ｰ譟ｻ縺ｸ螟画峩縺励◆縲・
  - `stdlib/tests/error.n.md`
    - `core/mem` 繧定ｿｽ蜉�縺励◆縲・
    - `Diag` / `Diags` / `Outcome` / `Result` 繧定､・焚蝗櫁ｦｳ蟇溘☆繧狗ｮ・園縺ｯ temporary memory 縺ｫ菫晏ｭ倥＠縲～load` 縺礼峩縺励※遒ｺ隱阪☆繧句ｽ｢縺ｸ譖ｴ譁ｰ縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `alloc/diag` 縺ｯ richer 縺ｪ險ｺ譁ｭ蛟､繝｢繝・Ν繧呈戟縺､縺後�～Diag` 閾ｪ菴薙ｒ `Copy` 縺ｫ縺ｯ縺ｧ縺阪↑縺・�ゅ＠縺溘′縺｣縺ｦ譬ｹ譛ｬ菫ｮ豁｣縺ｯ縲悟酔縺・owner 繧定､・焚蝗櫁ｪｭ繧�縲榊ｮ溯｣・ｒ繧・ａ繧九％縺ｨ縺�縺ｨ蛻､譁ｭ縺励◆縲・
  - `Vec` 繧貞・蟶ｰ縺ｫ縺昴・縺ｾ縺ｾ貂｡縺呵ｨｭ險医ｂ non-Copy collection 縺ｧ縺ｯ閼・＞縺溘ａ縲∵枚蟄怜・蛹悶・髮・ｴ・ｳｻ helper 縺ｯ raw backing store 繧剃ｸ�蠎ｦ蜿悶ｊ蜃ｺ縺励※縺九ｉ襍ｰ譟ｻ縺吶ｋ蠖｢縺ｸ蟇・○縺溘�・
  - test 蛛ｴ繧ら樟蝨ｨ縺ｮ ownership model 縺ｫ蜷医ｏ縺帙�∬ｦｳ蟇溷ｯｾ雎｡繧・memory 縺ｫ騾�驕ｿ縺励※蜀崎ｪｭ縺吶ｋ蠖｢縺ｸ謠・∴縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/tests/diag.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/diag.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/error.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/error.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/error.n.md -n 3` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/diag.n.md -i stdlib/tests/error.n.md -i stdlib/alloc/diag/diag.nepl -i stdlib/alloc/diag/error.nepl --no-stdlib --no-tree -o /tmp/tests-diag-error-focus.json -j 15`
    - [邨先棡/縺代▲縺犠: `7/7 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`std/test` collect API 縺ｨ `run_doctest` 縺ｮ豈碑ｼ・ｦ丞援繧貞酔譛・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - old failure list 縺ｫ縺ゅ▲縺・`test.nepl` / `std_test_collect.n.md` 邉ｻ繧・current move model 縺ｨ current nodesrc expectation 縺ｫ謠・∴繧九�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `stdlib/std/test.nepl`
    - collect API 縺・`Vec<Result<(),str>>` 繧貞・蟶ｰ縺ｧ縺昴・縺ｾ縺ｾ謖√■蝗槭＠縺ｦ縺翫ｊ縲∫樟陦・move model 縺ｧ縺ｯ `checks` 縺ｮ蜀榊茜逕ｨ縺・`D3053` 縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
    - `checks_has_err_*` / `checks_summary_*` / `checks_print_failures_*` / `finish_checks` 縺ｯ縲］on-Copy `Vec` 繧剃ｽ募ｺｦ繧りｪｭ縺ｿ逶ｴ縺呎立螳溯｣・・縺ｾ縺ｾ縺�縺｣縺溘�・
  - `nodesrc/run_doctest.js`
    - `tests.js` 縺ｨ驕輔▲縺ｦ `strip_ansi` / `normalize_newlines` 繧貞渚譏�縺励※縺翫ｉ縺壹�√＆繧峨↓ `should_panic` case 縺ｮ stdout 豈碑ｼ・ｂ繧ｹ繧ｭ繝・・縺励※縺・↑縺九▲縺溘�・
    - 縺昴・縺溘ａ `tests.js` 縺ｧ縺ｯ pass 縺吶ｋ `std_test_collect` 縺・`run_doctest.js` 縺ｧ縺ｯ ANSI 濶ｲ繧ｳ繝ｼ繝峨▽縺・stdout mismatch 縺ｧ fail 縺励※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/std/test.nepl`
    - collect API 縺ｮ蜀・Κ襍ｰ譟ｻ繧・`Vec` owner 縺ｮ蜀榊ｸｰ謖√■蝗槭＠縺九ｉ縲》emporary memory + raw data 襍ｰ譟ｻ縺ｸ螟画峩縺励◆縲・
    - `checks_has_err`
    - `checks_summary`
    - `checks_print_failures_loop`
    - `checks_report_failures`
    - `finish_checks`
      - 縺・★繧後ｂ backing store 繧・1 蝗槭□縺大叙繧雁・縺励※菴ｿ縺・ｽ｢縺ｫ謠・∴縺溘�・
    - 髢｢騾｣ doc comment 繧・raw data 襍ｰ譟ｻ繝吶・繧ｹ縺ｮ螳溯｣・ｪｬ譏弱∈譖ｴ譁ｰ縺励◆縲・
  - `nodesrc/run_doctest.js`
    - `normalize_newlines` 縺ｨ `strip_ansi` 繧・`tests.js` 縺ｨ蜷後§隕丞援縺ｧ驕ｩ逕ｨ縺吶ｋ繧医≧縺ｫ縺励◆縲・
    - `should_panic` case 縺ｮ I/O expectation 繧・`tests.js` 縺ｨ蜷梧ｧ倥↓繧ｹ繧ｭ繝・・縺吶ｋ繧医≧縺ｫ縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `std/test` 縺ｯ tutorial / stdlib doctest 縺ｮ蝓ｺ逶､縺ｪ縺ｮ縺ｧ縲》est data 繧・ANSI 縺ｪ縺励∈譖ｸ縺肴鋤縺医ｋ縺ｮ縺ｧ縺ｯ縺ｪ縺・collect API 縺ｨ runner 縺ｮ荳｡譁ｹ繧・current 莉墓ｧ倥∈謠・∴繧九・縺梧�ｹ譛ｬ菫ｮ豁｣縺ｨ蛻､譁ｭ縺励◆縲・
  - focused debugging 逕ｨ縺ｮ `run_doctest.js` 縺梧悽菴・runner 縺ｨ驕輔≧ expectation 隕丞援繧呈戟縺､縺ｮ縺ｯ蜊ｱ髯ｺ縺ｪ縺ｮ縺ｧ縲～tests.js` 縺ｨ蜷後§豈碑ｼ・そ繝槭Φ繝・ぅ繧ｯ繧ｹ縺ｫ蟇・○縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/std/test.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/std_test_collect.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/std_test_collect.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/std_test_collect.n.md -i stdlib/std/test.nepl --no-stdlib --no-tree -o /tmp/tests-std-test-collect-focus.json -j 15`
    - [邨先棡/縺代▲縺犠: `14/14 pass`
- [霑ｽ險・縺､縺・″]:
  - `doc/stdlib_doc_comment_policy.md` 繧端蜀咲｢ｺ隱・縺輔＞縺九￥縺ｫ繧転縺励�～stdlib/std/test.nepl` 縺ｮ莉雁屓[螟画峩/縺ｸ繧薙％縺・縺励◆ helper comment 繧・`##` / `###` [蠖｢蠑・縺代＞縺励″]縺ｸ[謠・縺昴ｍ]縺医◆縲・
  - [螳溯｣・縺倥▲縺昴≧]縺・raw data [襍ｰ譟ｻ/縺昴≧縺評縺ｸ[螟・縺犠繧上▲縺溘％縺ｨ縲［ove model 縺ｫ[蜷・縺・繧上○縺ｦ temporary memory 繧端菴ｿ/縺､縺犠縺・％縺ｨ縺・comment 縺ｫ[蜿肴丐/縺ｯ繧薙∴縺Ь縺輔ｌ縺ｦ縺・ｋ縺薙→繧端遒ｺ隱・縺九￥縺ｫ繧転縺励◆縲・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`std/test` 繧・trap 蜑肴署縺九ｉ `Result` 蜑肴署縺ｸ蜀崎ｨｭ險・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - [蠑ｷ蛻ｶ/縺阪ｇ縺・○縺Ь[邨ゆｺ・縺励ｅ縺・ｊ繧・≧]繝吶・繧ｹ縺ｮ[蜿､/縺ｵ繧犠縺・test [讖滓ｧ・縺阪％縺・繧端蟒・ｭ｢/縺ｯ縺・＠]縺励�～Result<(),str>` 繧端荳ｭ蠢・縺｡繧・≧縺励ｓ]縺ｫ縺励◆[螳牙・/縺ゅｓ縺懊ｓ]縺ｪ test API 縺ｸ[遘ｻ陦・縺・％縺・縺吶ｋ縲・
  - 縺ゅｏ縺帙※ `nodesrc` 蛛ｴ縺ｧ `ret:` 繧端螳滄圀/縺倥▲縺輔＞]縺ｫ[讀懈渊/縺代ｓ縺評縺ｧ縺阪ｋ繧医≧縺ｫ縺励�～Result` 繧・i32 縺ｮ[邨ゆｺ・縺励ｅ縺・ｊ繧・≧] code 縺ｸ[關ｽ/縺馨縺ｨ縺励※ runner 縺ｨ[謗･邯・縺帙▽縺槭￥]縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `stdlib/std/test.nepl`
    - `test_fail` / `finish_checks` / `assert_*` 縺・trap 繧端蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｫ縺励※縺翫ｊ縲〉eboot 縺ｮ縲啓螳牙・/縺ゅｓ縺懊ｓ] API [蜆ｪ蜈・繧・≧縺帙ｓ]縲阪�啓蛟､荳ｭ蠢・縺ゅ◆縺・■繧・≧縺励ｓ]繝ｻ[蠑乗欠蜷・縺励″縺励％縺・縲阪→[遏帷崟/繧�縺倥ｅ繧転縺励※縺・◆縲・
    - `check_*` 縺ｯ縺吶〒縺ｫ `Result<(),str>` 繧端霑・縺九∴]縺励※縺・◆縺ｮ縺ｫ縲ー譛�邨・縺輔＞縺励ｅ縺・[蜃ｺ蜿｣/縺ｧ縺舌■]縺�縺代′ trap 縺ｸ[貎ｰ/縺､縺ｶ]縺輔ｌ縺ｦ縺・◆縲・
  - `nodesrc`
    - doctest parser / runner 縺・`ret:` 繧端辟｡隕・繧�縺余縺励※縺・◆縺溘ａ縲ー螳牙・/縺ゅｓ縺懊ｓ]縺ｪ test [螟ｱ謨・縺励▲縺ｱ縺Ь繧端謌ｻ/繧ゅ←]繧骸蛟､/縺ゅ◆縺Ь縺ｧ runner 縺ｫ[莨・縺､縺歉縺医ｋ[邨瑚ｷｯ/縺代＞繧江縺啓蟄伜惠/縺昴ｓ縺悶＞]縺励↑縺九▲縺溘�・
    - Node WASI [螳溯｡・縺倥▲縺薙≧]繧・`_start` [邨檎罰/縺代＞繧・縺ｧ縺ｯ `main` 縺ｮ[謌ｻ/繧ゅ←]繧骸蛟､/縺ゅ◆縺Ь繧端謐ｨ/縺兢縺ｦ縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nodesrc/parser.js`
    - doctest meta 縺ｫ `ret:` 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - bare `ret: 0` 繧端譁・ｭ怜・/繧ゅ§繧後▽]縺ｧ縺ｯ縺ｪ縺充謨ｰ蛟､/縺吶≧縺｡]縺ｨ縺励※[隗｣驥・縺九＞縺励ｃ縺従縺吶ｋ `parseRetValue` 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
  - `nodesrc/run_test.js`
    - `wasi.start()` [荳�譛ｬ/縺・▲縺ｽ繧転縺ｧ縺ｯ縺ｪ縺上�～wasi.initialize({ exports: { memory, _initialize? } })` 縺ｮ縺ゅ→ exported `main` 繧端逶ｴ謗･/縺｡繧・￥縺帙▽][蜻ｼ/繧・縺ｶ[邨瑚ｷｯ/縺代＞繧江繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - 縺薙ｌ縺ｫ繧医ｊ stdout/stderr 繧端菫・縺溘ｂ]縺｣縺溘∪縺ｾ `main` 縺ｮ[謌ｻ/繧ゅ←]繧骸蛟､/縺ゅ◆縺Ь繧・`return_value` 縺ｨ縺励※[蜿門ｾ・縺励ｅ縺ｨ縺従縺ｧ縺阪ｋ繧医≧縺ｫ縺励◆縲・
    - `ret:` 縺・JSON [譁・ｭ怜・/繧ゅ§繧後▽]縺ｮ縺ｨ縺阪・ NEPL 縺ｮ `str` [陦ｨ迴ｾ/縺ｲ繧・≧縺偵ｓ]・・[len:i32][bytes...]`・峨→縺励※[蠕ｩ蜿ｷ/縺ｵ縺上＃縺・縺吶ｋ繧医≧縺ｫ縺励◆縲・
  - `nodesrc/tests.js` / `nodesrc/run_doctest.js`
    - `expected_ret` 繧・parser 縺九ｉ[蜿・縺・縺措蜿・縺ｨ]縺｣縺ｦ[豈碑ｼ・縺ｲ縺九￥]縺吶ｋ繧医≧縺ｫ縺励◆縲・
    - `std/test` 繧・import 縺励※縺・ｋ case 縺ｧ `FAIL:` [陦・縺弱ｇ縺・縺啓蜃ｺ/縺ｧ]縺溘・縺ｫ stdout expectation 縺啓譏守､ｺ/繧√＞縺肋縺輔ｌ縺ｦ縺・↑縺Ъ蝣ｴ蜷・縺ｰ縺ゅ＞]縺ｯ fail 縺ｨ縺吶ｋ繧医≧縺ｫ縺励◆縲・
  - `stdlib/std/test.nepl`
    - file header 縺ｨ[髢｢騾｣/縺九ｓ繧後ｓ] helper comment 繧・reboot 蠕後・ doc comment policy 縺ｫ[豐ｿ/縺拆縺｣縺ｦ[蜈ｨ髱｢逧・縺懊ｓ繧√ｓ縺ｦ縺江縺ｫ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
    - `test_fail` 繧・trap 縺ｧ縺ｯ縺ｪ縺・`Result<(),str>::Err msg` 繧端霑・縺九∴]縺・helper 縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `test_checked` 繧・`Result<(),str>::Ok ()` 繧端霑・縺九∴]縺・helper 縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `finish_checks` 繧・trap 縺ｧ縺ｯ縺ｪ縺・`Result<(),str>` 縺ｫ[逡ｳ/縺溘◆]繧� helper 縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `assert` / `assert_eq_i32` / `assert_ne` / `assert_str_eq` / `assert_ok_i32` / `assert_err_i32` 繧・`Result<(),str>` [霑泌唆/縺ｸ繧薙″繧・￥]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `result_exit_code` / `checks_exit_code` 繧端霑ｽ蜉�/縺､縺・°]縺励�～main <()*>i32>` 縺九ｉ runner 縺ｸ[螳牙・/縺ゅｓ縺懊ｓ]縺ｫ[蜷亥凄/縺斐≧縺ｲ]繧端霑・縺九∴]縺帙ｋ繧医≧縺ｫ縺励◆縲・
  - `tests/stdlib/std_test_collect.n.md`
    - success / failure case 繧・`ret: 0` / `ret: 1` + `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `[should_panic]` 縺ｯ[蜑企勁/縺輔￥縺倥ｇ]縺励◆縲・
  - `tutorials/getting_started/11_testing_workflow.n.md`
    - `std/test` 縺ｮ[迴ｾ陦・縺偵ｓ縺薙≧][謗ｨ螂ｨ/縺吶＞縺励ｇ縺・縺ｯ `Result<(),str>` + `checks_exit_code` / `result_exit_code` 縺ｧ縺ゅｋ縺薙→縺ｫ[蜷・縺・繧上○縺ｦ example 繧端譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - reboot 縺ｮ[譁ｹ驥・縺ｻ縺・＠繧転縺ｧ縺ｯ test helper 繧・蛟､/縺ゅ◆縺Ь繧端霑・縺九∴]縺吶∋縺阪〒縺ゅｊ縲》rap 縺ｯ public API 縺ｮ[譛�邨・縺輔＞縺励ｅ縺・[陦ｨ迴ｾ/縺ｲ繧・≧縺偵ｓ]縺ｫ[谿・縺ｮ縺転縺吶∋縺阪〒縺ｪ縺・→[蛻､譁ｭ/縺ｯ繧薙□繧転縺励◆縲・
  - [譌｢蟄・縺阪◎繧転縺ｮ unit-return test 繧端荳�蠎ｦ/縺・■縺ｩ]縺ｫ[蜈ｨ莉ｶ/縺懊ｓ縺代ｓ][譖ｸ/縺犠縺梗謠・縺犠縺医↑縺上※繧・螳牙・/縺ゅｓ縺懊ｓ]縺ｫ[遘ｻ陦・縺・％縺・縺ｧ縺阪ｋ繧医≧縲〉unner [蛛ｴ/縺後ｏ]縺ｧ `FAIL:` [蜃ｺ蜉・縺励ｅ縺､繧翫ｇ縺従繧端螟ｱ謨・縺励▲縺ｱ縺Ь縺ｨ[隕・縺ｿ]縺ｪ縺兌隕丞援/縺阪◎縺従繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
  - `ret:` 縺ｮ[譛ｪ螳溯｣・縺ｿ縺倥▲縺昴≧]繧端謾ｾ鄂ｮ/縺ｻ縺・■]縺励◆縺ｾ縺ｾ `std/test` 縺�縺・`Result` 蛹悶＠縺ｦ繧・蜃ｺ蜿｣/縺ｧ縺舌■]縺後↑縺・◆繧√�～nodesrc` [蛛ｴ/縺後ｏ]繧端蜈・縺輔″]縺ｫ[謨ｴ蛯・縺帙＞縺ｳ]縺吶ｋ縺ｮ縺啓譬ｹ譛ｬ菫ｮ豁｣/縺薙ｓ縺ｽ繧薙＠繧・≧縺帙＞]縺ｨ[蛻､譁ｭ/縺ｯ繧薙□繧転縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i /tmp/ret_probe.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/compiler/ret_string_example.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i tests/compiler/ret_string_example.n.md -i tests/stdlib/proptest.n.md --no-stdlib --no-tree -o /tmp/tests-ret-focus.json -j 4`
    - [邨先棡/縺代▲縺犠: `4/4 pass`
  - `node nodesrc/run_doctest.js -i stdlib/std/test.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/std_test_collect.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/std_test_collect.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/11_testing_workflow.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/11_testing_workflow.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i stdlib/std/test.nepl -i tests/stdlib/std_test_collect.n.md -i tutorials/getting_started/11_testing_workflow.n.md --no-stdlib --no-tree -o /tmp/tests-std-test-safe-result.json -j 4`
    - [邨先棡/縺代▲縺犠: `16/16 pass`
  - `node nodesrc/tests.js -i tests/compiler/ret_string_example.n.md -i tests/stdlib/proptest.n.md -i stdlib/std/test.nepl -i tests/stdlib/std_test_collect.n.md -i tutorials/getting_started/11_testing_workflow.n.md --no-stdlib --no-tree -o /tmp/tests-safe-test-ret-focus.json -j 4`
    - [邨先棡/縺代▲縺犠: `20/20 pass`
  - `node nodesrc/cli.js -i stdlib/std/test.nepl -i tutorials/getting_started/11_testing_workflow.n.md -o html=/tmp/std-test-safe-doc-html`
    - [邨先棡/縺代▲縺犠: `generated 2 html file(s)`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`Option` / `Result` 縺ｮ蜈･髢�邉ｻ doctest 繧貞ｮ牙・縺ｪ test 豬∝о縺ｸ霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `std/test` 縺ｮ `Result` [荳ｭ蠢・縺｡繧・≧縺励ｓ]險ｭ險医∈[蜷・縺・繧上○縺ｦ縲～core/result` / `core/option` 縺ｮ doctest 縺ｨ縲ー蟇ｾ蠢・縺溘＞縺翫≧]縺吶ｋ tutorial / stdlib fixture 繧端螳牙・/縺ゅｓ縺懊ｓ]縺ｪ `ret:` + `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[遘ｻ陦・縺・％縺・縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `stdlib/core/result.nepl` 縺ｨ `stdlib/core/option.nepl` 縺ｫ縲》rap [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｮ `neplg2:test[should_panic]` 縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
  - `tutorials/getting_started/05_option.n.md`, `tutorials/getting_started/06_result.n.md`, `stdlib/tests/option.n.md`, `stdlib/tests/result.n.md` 繧・unit-return + `test_fail` / `assert_*` [逶ｴ蜻ｼ/縺｡繧・￥繧・縺ｳ縺ｮ[蜿､/縺ｵ繧犠縺・test [豬∝о/繧翫ｅ縺・℃]縺ｮ縺ｾ縺ｾ縺�縺｣縺溘�・
  - `std/test` [蛛ｴ/縺後ｏ]縺ｯ縺吶〒縺ｫ `Result<(),str>` 繧端霑・縺九∴]縺吶ｈ縺・↓[螟・縺犠繧上▲縺ｦ縺・ｋ縺溘ａ縲ー蜈･髢�逕ｨ/縺ｫ繧・≧繧ゅｓ繧医≧]縺ｮ[譁・嶌/縺ｶ繧薙＠繧Ⅹ縺啓蜿､/縺ｵ繧犠縺・∪縺ｾ縺�縺ｨ reboot [蠕・縺脳縺ｮ[險ｭ險亥憧蟄ｦ/縺帙▲縺代＞縺ｦ縺､縺後￥]縺ｨ[隱ｬ譏・縺帙▽繧√＞]縺啓鬟・縺従縺Ъ驕・縺｡縺珪縺・�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/core/result.nepl`
    - file header 繧・reboot 蠕後・[譁ｹ驥・縺ｻ縺・＠繧転縺ｫ[豐ｿ/縺拆縺・隱ｬ譏・縺帙▽繧√＞]縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
    - `should_panic` doctest 繧端蜑企勁/縺輔￥縺倥ｇ]縺励�～ret: 0` + `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｮ[螳牙・/縺ゅｓ縺懊ｓ]縺ｪ doctest 縺ｸ[鄂ｮ謠・縺｡縺九ｓ]縺励◆縲・
  - `stdlib/core/option.nepl`
    - file header 繧・reboot 蠕後・[譁ｹ驥・縺ｻ縺・＠繧転縺ｫ[豐ｿ/縺拆縺・隱ｬ譏・縺帙▽繧√＞]縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
    - `should_panic` doctest 繧端蜑企勁/縺輔￥縺倥ｇ]縺励�～ret: 0` + `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｮ[螳牙・/縺ゅｓ縺懊ｓ]縺ｪ doctest 縺ｸ[鄂ｮ謠・縺｡縺九ｓ]縺励◆縲・
  - `tutorials/getting_started/05_option.n.md`
    - `match` [萓・繧後＞]縺ｨ `option_unwrap_or` [萓・繧後＞]繧・`ret: 0` + `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
    - `match` [蛻・ｲ・縺ｶ繧薙″]縺ｮ[荳ｭ/縺ｪ縺犠縺ｧ `checks_push` 縺ｧ縺阪ｋ繧医≧ `let mut checks` 縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `tutorials/getting_started/06_result.n.md`
    - `Ok/Err` [萓・繧後＞]縺ｨ `Result` 繧端霑・縺九∴]縺兌髢｢謨ｰ/縺九ｓ縺吶≧][萓・繧後＞]繧・`ret: 0` + `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
    - [蛻・ｲ・縺ｶ繧薙″]縺ｧ[闢・ｩ・縺｡縺上○縺江縺吶ｋ `checks` 繧・`let mut` 縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `stdlib/tests/result.n.md`, `stdlib/tests/option.n.md`
    - fixture 繧・`ret: 0` + `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - [騾先ｬ｡逧・縺｡縺上§縺ｦ縺江縺ｪ `assert_*` [逶ｴ蛻・縺｡繧・￥繧後▽]縺ｧ縺ｯ縺ｪ縺上�～checks_push` [邨檎罰/縺代＞繧・縺ｧ[蜿朱寔/縺励ｅ縺・＠繧・≧]縺吶ｋ[蠖｢/縺九◆縺｡]縺ｸ[謠・縺昴ｍ]縺医◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `unwrap` 邉ｻ helper [閾ｪ菴・縺倥◆縺Ь縺ｯ[莠呈鋤荳・縺斐°繧薙§繧・≧][谿・縺ｮ縺転縺励※縺・ｋ縺後�ー蜈･髢�逕ｨ/縺ｫ繧・≧繧ゅｓ繧医≧]縺ｮ doctest 縺ｧ trap [譛溷ｾ・縺阪◆縺Ь繧端謗ｨ螂ｨ/縺吶＞縺励ｇ縺・縺励↑縺・％縺ｨ繧端蜆ｪ蜈・繧・≧縺帙ｓ]縺励◆縲・
  - `core/result` / `core/option` 縺ｮ[隱ｬ譏・縺帙▽繧√＞]縺ｯ縲「nsafe helper 縺ｮ[蟄伜惠/縺昴ｓ縺悶＞]繧端豕ｨ諢・縺｡繧・≧縺Ь縺ｨ縺励※[譏手ｨ・繧√＞縺江縺励▽縺､縲ー騾壼ｸｸ/縺､縺・§繧・≧]縺ｯ `match` / `unwrap_or` 繧端蜆ｪ蜈・繧・≧縺帙ｓ]縺吶ｋ reboot [蠕・縺脳縺ｮ[蟋ｿ蜍｢/縺励○縺Ь縺ｸ[蟇・繧・縺帙◆縲・
  - tutorial / fixture 縺ｧ縺ｯ `FAIL:` [陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺�縺代↓[萓晏ｭ・縺・◇繧転縺帙★縲〉unner 縺ｨ[逶ｴ邨・縺｡繧・▲縺代▽]縺ｧ縺阪ｋ `ret:` [豈碑ｼ・縺ｲ縺九￥]繧端譏守､ｺ/繧√＞縺肋縺吶ｋ縺ｻ縺・′縲ー迴ｾ陦・縺偵ｓ縺薙≧] test [蜩ｲ蟄ｦ/縺ｦ縺､縺後￥]縺ｨ[謨ｴ蜷・縺帙＞縺斐≧]縺吶ｋ縺ｨ[蛻､譁ｭ/縺ｯ繧薙□繧転縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/core/result.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/core/option.nepl -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/06_result.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i stdlib/core/result.nepl -i stdlib/core/option.nepl -i stdlib/tests/result.n.md -i stdlib/tests/option.n.md -i tutorials/getting_started/05_option.n.md -i tutorials/getting_started/06_result.n.md --no-stdlib --no-tree -o /tmp/tests-option-result-safe.json -j 4`
    - [邨先棡/縺代▲縺犠: `12/12 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (tutorial 蛻晄悄遶�縺ｨ stdlib fixture 縺ｮ safe `Result` 蛹悶ｒ邯咏ｶ・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `std/test` 縺ｮ trap [蜑肴署/縺懊ｓ縺ｦ縺Ь繧端蟒・ｭ｢/縺ｯ縺・＠]縺励◆ reboot [蠕・縺脳縺ｮ test [豬∝о/繧翫ｅ縺・℃]縺ｫ[蜷・縺・繧上○縺ｦ縲》utorial [蛻晄悄/縺励ｇ縺江[遶�/縺励ｇ縺・縺ｨ `stdlib/tests` 縺ｮ[蟆・縺｡縺Ь縺輔＞ fixture [鄒､/縺舌ｓ]繧・`ret:` + `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[遘ｻ陦・縺・％縺・縺吶ｋ縲・
  - [驛ｨ蛻・縺ｶ縺ｶ繧転 test 繧端蟆丞・/縺薙ｏ]縺代↓縺励※縲ー驥・縺翫ｂ]縺Ъ蜈ｨ菴・縺懊ｓ縺溘＞] test 繧端鬆ｻ郢・縺ｲ繧薙・繧転縺ｫ[蝗・縺ｾ繧従縺輔★縺ｫ stale case 繧端蜿取據/縺励ｅ縺・◎縺従縺輔○繧九�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `tutorials/getting_started/02_numbers_and_variables.n.md` 縺ｨ `tutorials/getting_started/03_functions.n.md` 縺後�～assert_*` 繧・unit-return [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｧ[逶ｴ蛻・縺｡繧・￥繧後▽][螳溯｡・縺倥▲縺薙≧]縺励�～test_checked` 繧・蜑ｯ菴懃畑/縺ｵ縺上＆繧医≧]縺�縺代・ helper 縺ｨ縺励※[謇ｱ/縺ゅ▽縺犠縺・蜿､/縺ｵ繧犠縺Ъ譖ｸ/縺犠縺梗譁ｹ/縺九◆]縺ｮ縺ｾ縺ｾ縺�縺｣縺溘�・
  - `stdlib/tests/cast.n.md`, `stdlib/tests/math.n.md`, `stdlib/tests/vec.n.md` 繧・蜷梧ｧ・縺ｩ縺・ｈ縺・縺ｫ unit-return [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｧ縲～vec` 縺ｮ `None` [蛻・ｲ・縺ｶ繧薙″]縺ｧ縺ｯ `test_fail` 繧端蜊ｳ譎・縺昴￥縺肋[螳溯｡・縺倥▲縺薙≧]縺吶ｋ[讒矩��/縺薙≧縺槭≧]縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
  - `cast` fixture 縺ｯ pipe [荳ｭ/縺｡繧・≧]縺ｫ `cast` 繧端逶ｴ謗･/縺｡繧・￥縺帙▽][蝓・縺・繧ー霎ｼ/縺転繧薙〒縺・◆縺溘ａ縲《afe `Result` 蛹悶↓繧医ｊ `checks_push` 縺ｨ[邨・縺従縺ｿ[蜷・縺・繧上＆縺｣縺溘→縺・overload [隗｣豎ｺ/縺九＞縺代▽]縺啓蟠ｩ/縺上★]繧後ｋ[邂・園/縺九＠繧Ⅹ縺啓髴ｲ蜃ｺ/繧阪＠繧・▽]縺励◆縲・
  - `let checks <Vec<Result<(),str>>>:` [蠖｢蠑・縺代＞縺励″]縺ｧ縺ｯ縲ー譛�邨・縺輔＞縺励ｅ縺・[陦・縺弱ｇ縺・縺ｮ `;` 縺・block 縺ｮ[霑・縺九∴]繧骸蛟､/縺ゅ◆縺Ь繧・unit 縺ｫ縺励※縺励∪縺・�～Vec<Result<(),str>>` [譛溷ｾ・縺阪◆縺Ь縺ｨ[陦晉ｪ・縺励ｇ縺・→縺､]縺吶ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tutorials/getting_started/02_numbers_and_variables.n.md`
    - 5 [莉ｶ/縺代ｓ]縺ｮ doctest 縺吶∋縺ｦ縺ｫ `ret: 0` 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `fn main <()*> ()> ():` 繧・`fn main <()*>i32> ():` 縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励�～checks_new` / `checks_push` / `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[謠・縺昴ｍ]縺医◆縲・
    - `test_checked` 縺ｯ `Result<(),str>` 繧端霑・縺九∴]縺・helper 縺ｨ縺励※ `let _done <Result<(),str>> ...` 縺ｧ[蜿・縺・縺代ｋ[蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `tutorials/getting_started/03_functions.n.md`
    - 3 [莉ｶ/縺代ｓ]縺ｮ doctest 繧・`ret: 0` + `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
    - `if` / `if:` [萓・繧後＞]繧貞性繧�[蜈ｨ菴・縺懊ｓ縺溘＞]繧・safe `Result` [豬∝о/繧翫ｅ縺・℃]縺ｸ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励◆縲・
  - `stdlib/tests/cast.n.md`
    - `ret: 0` 繧端霑ｽ蜉�/縺､縺・°]縺励�’ixture [蜈ｨ菴・縺懊ｓ縺溘＞]繧・`checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - bool/i32 cast [遒ｺ隱・縺九￥縺ｫ繧転縺ｯ `cast` [邨先棡/縺代▲縺犠繧端蜈・縺輔″]縺ｫ[螻�謇�/縺阪ｇ縺上＠繧Ⅹ[螟画焚/縺ｸ繧薙☆縺・縺ｸ[譚溽ｸ・縺昴￥縺ｰ縺従縺励�√◎縺ｮ[蛟､/縺ゅ◆縺Ь繧・`assert_*` 縺ｧ[讀懈渊/縺代ｓ縺評縺吶ｋ[蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - 縺薙ｌ縺ｫ繧医ｊ pipe + overload [隗｣豎ｺ/縺九＞縺代▽]縺ｮ[譖匁乂/縺ゅ＞縺ｾ縺Ь縺輔ｒ[髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励◆縲・
  - `stdlib/tests/math.n.md`
    - `ret: 0` 繧端霑ｽ蜉�/縺､縺・°]縺励�ー蜈ｨ讀懈渊/縺懊ｓ縺代ｓ縺評繧・1 [譛ｬ/縺ｻ繧転縺ｮ `checks_new |> checks_push ...` 縺ｫ[髮・ｴ・縺励ｅ縺・ｄ縺従縺励◆縲・
    - `let checks:` block [譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｮ `;` 繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励�ー霑・縺九∴]繧骸蛟､/縺ゅ◆縺Ь縺・unit 縺ｫ[貎ｰ/縺､縺ｶ]繧後↑縺・ｈ縺・↓縺励◆縲・
  - `stdlib/tests/vec.n.md`
    - `ret: 0` 繧端霑ｽ蜉�/縺､縺・°]縺励�～let mut checks` [譁ｹ蠑・縺ｻ縺・＠縺江縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `match vec_get ...` 縺ｮ `None` [蛻・ｲ・縺ｶ繧薙″]繧・`test_fail` 繧・`checks_push` 縺ｧ[髮・ｴ・縺励ｅ縺・ｄ縺従縺吶ｋ[蠖｢/縺九◆縺｡]縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励�ー騾比ｸｭ/縺ｨ縺｡繧・≧] trap 縺励↑縺・ｈ縺・↓縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `std/test` 縺ｮ `Result<(),str>` [譁ｹ驥・縺ｻ縺・＠繧転縺ｸ[霑ｽ蠕・縺､縺・§繧・≧]縺吶ｋ縺�縺代〒縺ｪ縺上�》utorial [蜀帝�ｭ/縺ｼ縺・→縺・縺九ｉ縲荊est helper 繧・蛟､/縺ゅ◆縺Ь繧端霑・縺九∴]縺吶�阪→縺・≧ reboot [蠕・縺脳縺ｮ[萓｡蛟､隕ｳ/縺九■縺九ｓ]繧端荳�雋ｫ/縺・▲縺九ｓ]縺励※[遉ｺ/縺励ａ]縺吶％縺ｨ繧端蜆ｪ蜈・繧・≧縺帙ｓ]縺励◆縲・
  - `cast` fixture 縺ｮ[荳榊・蜷・縺ｵ縺舌≠縺Ь縺ｯ runner [蛛ｴ/縺後ｏ]縺ｧ縺ｯ縺ｪ縺上�｝ipe [荳ｭ/縺｡繧・≧]縺ｧ overload [譖匁乂/縺ゅ＞縺ｾ縺Ь縺ｪ[蠑・縺励″]繧端逶ｴ謗･/縺｡繧・￥縺帙▽][隧穂ｾ｡/縺ｲ繧・≧縺犠縺励※縺・◆[譖ｸ/縺犠縺梗譁ｹ/縺九◆]縺ｫ[蜴溷屏/縺偵ｓ縺・ｓ]縺後≠縺｣縺溘◆繧√�ー荳ｭ髢灘�､/縺｡繧・≧縺九ｓ縺｡]繧端譏守､ｺ/繧√＞縺肋縺吶ｋ[蠖｢/縺九◆縺｡]縺ｸ[豁｣隕丞喧/縺帙＞縺阪°]縺励◆縲・
  - `let checks:` block 縺ｮ[譛ｫ蟆ｾ/縺ｾ縺､縺ｳ] `;` 縺ｯ[讒区枚荳・縺薙≧縺ｶ繧薙§繧・≧]縺ｯ[蟆・縺｡縺Ь縺輔＞縺後�《afe `Result` [遘ｻ陦・縺・％縺・縺ｧ縺ｯ[譬ｹ譛ｬ逧・縺薙ｓ縺ｽ繧薙※縺江縺ｫ[蝙・縺九◆]繧端螢・縺薙ｏ]縺吶◆繧√�ー螻�謇�逧・縺阪ｇ縺上＠繧・※縺江縺ｪ蝗樣∩縺ｧ縺ｯ縺ｪ縺・fixture [蜈ｨ菴・縺懊ｓ縺溘＞]縺ｮ[譖ｸ/縺犠縺梗譁ｹ/縺九◆]繧端邨ｱ荳�/縺ｨ縺・＞縺､]縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/02_numbers_and_variables.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/vec.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/cast.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/math.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i tutorials/getting_started/02_numbers_and_variables.n.md -i tutorials/getting_started/03_functions.n.md -i stdlib/tests/cast.n.md -i stdlib/tests/math.n.md -i stdlib/tests/vec.n.md --no-stdlib --no-tree -o /tmp/tests-safe-result-batch1.json -j 4`
    - [邨先棡/縺代▲縺犠: `11/11 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (control-flow tutorial 縺ｮ safe `Result` 蛹悶ｒ邯咏ｶ・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `while` / `block` / `if` / `#import` 繧端隱ｬ譏・縺帙▽繧√＞]縺吶ｋ tutorial [鄒､/縺舌ｓ]繧ゅ�～std/test` 縺ｮ[迴ｾ陦・縺偵ｓ縺薙≧][譁ｹ驥・縺ｻ縺・＠繧転縺ｫ[蜷・縺・繧上○縺ｦ `ret:` + `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[邨ｱ荳�/縺ｨ縺・＞縺､]縺吶ｋ縲・
  - [蛻晏ｭｦ閠・縺励ｇ縺後￥縺励ｃ]縺・tutorial 繧端鬆・縺倥ｅ繧転縺ｫ[隱ｭ/繧・繧薙□縺ｨ縺阪�…hapter 縺斐→縺ｫ test [豬∝о/繧翫ｅ縺・℃]縺啓謠ｺ/繧・繧後↑縺・ｈ縺・↓縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `tutorials/getting_started/07_while_and_block.n.md`, `08_if_layouts.n.md`, `09_import_and_structure.n.md` 縺ｫ縲「nit-return 縺ｮ `main` 縺ｨ `assert_*` [逶ｴ蛻・縺｡繧・￥繧後▽][螳溯｡・縺倥▲縺薙≧]繧端蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｫ縺励◆[蜿､/縺ｵ繧犠縺Ъ譖ｸ/縺犠縺梗譁ｹ/縺九◆]縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
  - `11_testing_workflow` 縺�縺・safe `Result` [豬∝о/繧翫ｅ縺・℃]縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺輔ｌ縺ｦ繧ゅ�√◎繧後ｈ繧骸蜑・縺ｾ縺・縺ｮ tutorial 縺啓蜿､/縺ｵ繧犠縺・∪縺ｾ縺�縺ｨ reboot [蠕・縺脳縺ｮ test [蜩ｲ蟄ｦ/縺ｦ縺､縺後￥]縺啓騾比ｸｭ/縺ｨ縺｡繧・≧]縺ｧ[騾・綾/縺弱ｃ縺上ｂ縺ｩ]繧翫＠縺ｦ縺励∪縺・�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tutorials/getting_started/07_while_and_block.n.md`
    - `while` 縺ｨ `block:` 縺ｮ 2 [莉ｶ/縺代ｓ]縺ｮ doctest 縺ｫ `ret: 0` 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `fn main <()*> ()> ():` 繧・`fn main <()*>i32> ():` 縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励�～checks_new` / `checks_push` / `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[謠・縺昴ｍ]縺医◆縲・
  - `tutorials/getting_started/08_if_layouts.n.md`
    - 4 [莉ｶ/縺代ｓ]縺ｮ `if` [繝ｬ繧､繧｢繧ｦ繝・繧後＞縺ゅ≧縺ｨ]萓九ｒ `ret: 0` + `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
    - inline / `if:` / `then:` / mixed layout 縺ｮ[蜈ｨ萓・縺懊ｓ繧後＞]縺ｧ `core/result` 繧・import 縺励�～test_checked` 繧・`Result<(),str>` 縺ｨ縺励※[蜿・縺・縺代ｋ[蠖｢/縺九◆縺｡]縺ｫ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励◆縲・
  - `tutorials/getting_started/09_import_and_structure.n.md`
    - `std/test` 繧端菴ｿ/縺､縺犠縺・1 [莉ｶ/縺代ｓ]縺ｮ doctest 繧・safe `Result` [豬∝о/繧翫ｅ縺・℃]縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
    - `stdio` [蜃ｺ蜉・縺励ｅ縺､繧翫ｇ縺従縺�縺代ｒ[讀懆ｨｼ/縺代ｓ縺励ｇ縺・縺吶ｋ doctest 縺ｯ縲～ret:` [豈碑ｼ・縺ｲ縺九￥]繧端隕・繧医≧]縺励↑縺・◆繧√◎縺ｮ縺ｾ縺ｾ[邯ｭ謖・縺・§]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - tutorial [蜀帝�ｭ/縺ｼ縺・→縺・[驛ｨ/縺ｶ]縺ｯ[譁・ｳ・縺ｶ繧薙⊃縺・縺ｮ[隱ｬ譏・縺帙▽繧√＞]縺啓荳ｻ逶ｮ逧・縺励ｅ繧ゅ￥縺ｦ縺江縺�縺後�》est [蜈･蜿｣/縺・ｊ縺舌■]縺�縺措蜿､/縺ｵ繧犠縺・trap [豬∝о/繧翫ｅ縺・℃]繧端谿・縺ｮ縺転縺吶→縲～std/test` 縺ｮ reboot [蠕・縺脳[險ｭ險・縺帙▲縺代＞]縺ｨ[隱ｬ譏手ｲｬ莉ｻ/縺帙▽繧√＞縺帙″縺ｫ繧転縺啓遏帷崟/繧�縺倥ｅ繧転縺吶ｋ縲・
  - `stdout:` [豈碑ｼ・縺ｲ縺九￥]縺�縺代〒[蜊∝・/縺倥ｅ縺・・繧転縺ｪ case 縺ｾ縺ｧ[辟｡逅・繧�繧馨縺ｫ `std/test` 縺ｸ[蟇・繧・縺帙ｋ縺ｮ縺ｯ[荳崎ｦ・縺ｵ繧医≧]縺ｪ縺ｮ縺ｧ縲～09_import_and_structure` 縺ｮ I/O [萓・繧後＞]縺ｯ[譌｢蟄・縺阪◎繧転縺ｮ[雋ｬ蜍・縺帙″繧�]繧端邯ｭ謖・縺・§]縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/08_if_layouts.n.md -n 4` -> pass
  - `node nodesrc/tests.js -i tutorials/getting_started/07_while_and_block.n.md -i tutorials/getting_started/08_if_layouts.n.md -i tutorials/getting_started/09_import_and_structure.n.md --no-stdlib --no-tree -o /tmp/tests-safe-result-batch2.json -j 4`
    - [邨先棡/縺代▲縺犠: `8/8 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`Vec<Result<(),str>>` 縺ｮ test [邨先棡/縺代▲縺犠[陦ｨ遉ｺ/縺ｲ繧・≧縺肋繧・human / machine 縺ｫ[蛻・屬/縺ｶ繧薙ｊ])

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - reboot 縺ｮ縲啓蛟､荳ｭ蠢・縺ゅ◆縺・■繧・≧縺励ｓ]繝ｻ[螳牙・/縺ゅｓ縺懊ｓ] API [蜆ｪ蜈・繧・≧縺帙ｓ]繝ｻ[雋ｬ蜍・縺帙″繧�][蛻・屬/縺ｶ繧薙ｊ]縲阪↓[蠕・縺励◆縺珪縺・�～Vec<Result<(),str>>` 縺ｮ test [邨先棡/縺代▲縺犠[陦ｨ遉ｺ/縺ｲ繧・≧縺肋繧・machine [蜷・繧�]縺・summary 縺ｨ human [蜷・繧�]縺・ANSI [陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺ｸ[蛻・屬/縺ｶ繧薙ｊ]縺吶ｋ縲・
  - `finish_checks` 縺・failure [譎・縺肋縺�縺措譁ｭ迚・噪/縺�繧薙⊆繧薙※縺江縺ｫ[隧ｳ邏ｰ/縺励ｇ縺・＆縺Ь繧端蜃ｺ/縺�]縺兌譌ｧ譚･/縺阪ｅ縺・ｉ縺Ь縺ｮ[謖吝虚/縺阪ｇ縺ｩ縺・繧偵ｄ繧√�《uccess / failure [荳｡譁ｹ/繧翫ｇ縺・⊇縺・縺ｧ `Vec<Result>` [蜈ｨ菴・縺懊ｓ縺溘＞]繧端隱ｭ/繧・縺ｿ繧・☆縺充隕・縺ｿ]縺帙ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `stdlib/std/test.nepl` 縺ｮ `checks_summary` 縺ｯ `[ok,err,...]` 縺ｪ縺・＠ `[ok,err <msg>,...]` 縺ｮ 1 [陦・縺弱ｇ縺・ summary 縺ｫ[蛛・縺九◆繧・縺｣縺ｦ縺翫ｊ縲ー莠ｺ髢・縺ｫ繧薙￡繧転縺・`Vec<Result>` [蜈ｨ菴・縺懊ｓ縺溘＞]繧端霑ｽ/縺馨縺・↓縺ｯ[荳崎ｶｳ/縺ｵ縺昴￥]縺励※縺・◆縲・
  - failure [譎・縺肋縺ｮ `checks_report_failures` 繧・`Err` [隕∫ｴ�/繧医≧縺拆縺�縺代ｒ `check[i] ...` 縺ｨ縺励※[蜃ｺ/縺�]縺励※縺・◆縺溘ａ縲《uccess [鬆・岼/縺薙≧繧ゅ￥]縺ｨ縺ｮ[荳ｦ/縺ｪ繧云縺ｳ繧Ъ蜈ｨ菴灘ワ/縺懊ｓ縺溘＞縺槭≧]縺啓隕・縺ｿ]縺医↓縺上°縺｣縺溘�・
  - reboot.md 縺ｮ[險ｭ險亥次蜑・縺帙▲縺代＞縺偵ｓ縺昴￥]縺ｧ縺ｯ縲［achine [蜷・繧�]縺代→ human [蜷・繧�]縺代・[陦ｨ遉ｺ雋ｬ蜍・縺ｲ繧・≧縺倥○縺阪・]繧端蛻・繧従縺代ｋ縺ｹ縺阪〒縺ゅｊ縲√％縺薙′[譛ｪ謨ｴ逅・縺ｿ縺帙＞繧馨縺�縺｣縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/std/test.nepl`
    - `check_status_str` 繧・machine [蜷・繧�]縺・summary helper 縺ｨ縺励※[謨ｴ逅・縺帙＞繧馨縺励�～Err` 縺ｧ縺ｯ `err <msg>` 繧端霑・縺九∴]縺吶ｈ縺・↓縺励◆縲・
    - `checks_summary` 縺ｮ doc comment 繧偵�√�稽achine / log [蜷・繧�]縺代・[螳牙ｮ・縺ゅｓ縺ｦ縺Ь[陦ｨ迴ｾ/縺ｲ繧・≧縺偵ｓ]縲阪→縺励※[譏手ｨ・繧√＞縺江縺励◆縲・
    - `checks_print_human_line`
      - 1 [莉ｶ/縺代ｓ]縺ｮ `Result<(),str>` 繧・`[index] ok` / `[index] err <msg>` 縺ｧ[陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺吶ｋ helper 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
      - [豺ｻ蟄・縺昴∴縺肋縺ｯ轣ｰ濶ｲ縲～ok` 縺ｯ邱代�～err <msg>` 縺ｯ襍､縺ｧ[陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺吶ｋ縲・
    - `checks_print_human_loop` / `checks_print_human`
      - `Vec<Result<(),str>>` [蜈ｨ菴・縺懊ｓ縺溘＞]繧端鬆・縺倥ｅ繧転縺ｫ[濶ｲ莉・縺・ｍ縺･]縺梗陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺吶ｋ helper 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `finish_checks`
      - 縺ｾ縺・machine [蜷・繧�]縺・summary 繧・`Checked ...` / `FAIL: ...` 縺ｨ縺励※ 1 [陦・縺弱ｇ縺・[陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺励�√◎縺ｮ[蠕・縺ゅ→]縺ｧ `checks_print_human` 縺ｫ繧医ｊ[蜈ｨ隕∫ｴ�/縺懊ｓ繧医≧縺拆繧端濶ｲ莉・縺・ｍ縺･]縺梗陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺吶ｋ[蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
      - 縺薙ｌ縺ｫ繧医ｊ success / failure [荳｡譁ｹ/繧翫ｇ縺・⊇縺・縺ｧ `Vec<Result>` [蜈ｨ菴・縺懊ｓ縺溘＞]縺ｮ[蜿ｯ隕匁�ｧ/縺九＠縺帙＞]繧端謠・縺昴ｍ]縺医◆縲・
  - `tests/stdlib/std_test_collect.n.md`
    - success / failure [荳｡譁ｹ/繧翫ｇ縺・⊇縺・縺ｮ[譛溷ｾ・縺阪◆縺Ь stdout 繧偵�∵眠縺励＞ machine summary + human list [蠖｢蠑・縺代＞縺励″]縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - machine [蜷・繧�]縺・summary 縺ｯ `checks_summary` 縺ｮ 1 [陦・縺弱ｇ縺・[譁・ｭ怜・/繧ゅ§繧後▽]縺ｸ[谿・縺ｮ縺転縺励�〉unner / log / [豈碑ｼ・縺ｲ縺九￥]縺ｮ[螳牙ｮ壽�ｧ/縺ゅｓ縺ｦ縺・○縺Ь繧端邯ｭ謖・縺・§]縺励◆縲・
  - human [蜷・繧�]縺代↓縺ｯ `checks_print_human` 繧端蛻･/縺ｹ縺､][雋ｬ蜍・縺帙″繧�]縺ｨ縺励※[險ｭ/繧ゅ≧]縺代�、NSI color 繧端菴ｿ/縺､縺犠縺｣縺ｦ[謌仙粥/縺帙＞縺薙≧]縺ｨ[螟ｱ謨・縺励▲縺ｱ縺Ь繧端隕冶ｦ夂噪/縺励°縺上※縺江縺ｫ[蛻・屬/縺ｶ繧薙ｊ]縺励◆縲・
  - failure 縺�縺措隧ｳ邏ｰ/縺励ｇ縺・＆縺Ь繧端蜃ｺ/縺�]縺兌譁ｹ蠑・縺ｻ縺・＠縺江縺ｧ縺ｯ縺ｪ縺・success 繧・蜷ｫ/縺ｵ縺従繧√※[蜈ｨ莉ｶ/縺懊ｓ縺代ｓ]繧端陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺吶ｋ繧医≧縺ｫ縺励◆縺ｮ縺ｯ縲～Vec<Result>` [蜈ｨ菴・縺懊ｓ縺溘＞]縺ｮ[隱ｭ縺ｿ/繧・繧・☆縺輔ｒ[蜆ｪ蜈・繧・≧縺帙ｓ]縺励◆縺溘ａ縺ｧ縺ゅｋ縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tests/stdlib/std_test_collect.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/std_test_collect.n.md -i tutorials/getting_started/11_testing_workflow.n.md -i stdlib/tests/vec.n.md -i tutorials/getting_started/07_while_and_block.n.md -i tutorials/getting_started/08_if_layouts.n.md -i tutorials/getting_started/09_import_and_structure.n.md -i stdlib/std/test.nepl --no-stdlib --no-tree -o /tmp/tests-std-test-human-machine.json -j 4`
    - [邨先棡/縺代▲縺犠: `25/25 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (middle tutorial 縺ｮ safe `Result` 蛹悶ｒ邯咏ｶ・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `12_pure_function_pipeline`, `13_type_driven_error_modeling`, `14_refactor_with_properties` 繧偵�～std/test` 縺ｮ[迴ｾ陦・縺偵ｓ縺薙≧] safe `Result` [豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - [邏皮ｲ・縺倥ｅ繧薙☆縺Ь[髢｢謨ｰ/縺九ｓ縺吶≧]繝ｻ`Option` / `Result`繝ｻ[蝗槫ｸｰ/縺九＞縺江 test 縺ｮ chapter 縺ｧ繧ゅ�√�荊est helper 縺ｯ[蛟､/縺ゅ◆縺Ь繧端霑・縺九∴]縺吶�阪→縺・≧ reboot [蠕・縺脳縺ｮ[荳�雋ｫ諤ｧ/縺・▲縺九ｓ縺帙＞]繧端菫・縺溘ｂ]縺､縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - 3 [遶�/縺励ｇ縺・縺ｨ繧・`assert_*` 縺ｮ unit-return [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｨ `test_checked` 縺ｮ[蜑ｯ菴懃畑/縺ｵ縺上＆繧医≧] helper [蜑肴署/縺懊ｓ縺ｦ縺Ь縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
  - `14_refactor_with_properties.n.md` 縺ｮ `assert_same` 縺ｯ unit-return helper 縺�縺｣縺溘◆繧√�～checks_push` 縺ｫ[逶ｴ謗･/縺｡繧・￥縺帙▽][遨・縺､]繧√★縲《afe `Result` [豬∝о/繧翫ｅ縺・℃]縺ｸ[閾ｪ辟ｶ/縺励●繧転縺ｫ[謗･邯・縺帙▽縺槭￥]縺ｧ縺阪↑縺九▲縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tutorials/getting_started/12_pure_function_pipeline.n.md`
    - 2 [莉ｶ/縺代ｓ]縺ｮ doctest 縺ｫ `ret: 0` 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `core/result` 繧・import 縺励�～checks_new` / `checks_push` / `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `tutorials/getting_started/13_type_driven_error_modeling.n.md`
    - `Result` [萓・繧後＞]縺ｯ `let mut checks` 繧端蟆主・/縺ｩ縺・↓繧・≧]縺励�～match` [蛻・ｲ・縺ｶ繧薙″]縺斐→縺ｮ[謌仙凄/縺帙＞縺ｲ]繧・`checks_push` 縺ｧ[蜿朱寔/縺励ｅ縺・＠繧・≧]縺吶ｋ[蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `Option` [萓・繧後＞]繧・`ret: 0` + `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[謠・縺昴ｍ]縺医◆縲・
  - `tutorials/getting_started/14_refactor_with_properties.n.md`
    - [蜑榊濠/縺懊ｓ縺ｯ繧転縺ｮ[遲我ｾ｡諤ｧ/縺ｨ縺・°縺帙＞] doctest 繧・`checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `assert_same` 繧・`fn assert_same <(i32,i32)*>Result<(),str>>` 縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励�《afe `Result` [豬∝о/繧翫ｅ縺・℃]縺ｫ縺昴・縺ｾ縺ｾ[謗･邯・縺帙▽縺槭￥]縺ｧ縺阪ｋ helper 縺ｫ[蜀崎ｨｭ險・縺輔＞縺帙▲縺代＞]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `assert_same` 縺ｮ繧医≧縺ｪ chapter [蜀・縺ｪ縺Ь helper 縺薙◎縲〉eboot [蠕・縺脳縺ｯ unit-return 縺ｧ縺ｯ縺ｪ縺・`Result<(),str>` 繧端霑・縺九∴]縺吶⊇縺・′縲》est [蜷域・/縺斐≧縺帙＞]縺ｨ[雋ｬ蜍・縺帙″繧�]縺啓譏守｢ｺ/繧√＞縺九￥]縺ｫ縺ｪ繧九�・
  - `13` [遶�/縺励ｇ縺・縺ｯ縲啓蝙・縺九◆]縺ｧ[螟ｱ謨・縺励▲縺ｱ縺Ь繧端陦ｨ/縺ゅｉ繧従縺吶�阪′[荳ｻ鬘・縺励ｅ縺�縺Ь縺ｪ縺ｮ縺ｧ縲‥octest [閾ｪ菴・縺倥◆縺Ь繧・`Result` 繧端蛟､/縺ゅ◆縺Ь縺ｨ縺励※[蜿朱寔/縺励ｅ縺・＠繧・≧]縺吶ｋ[讒矩��/縺薙≧縺槭≧]縺ｸ[蟇・繧・縺帙ｋ縺ｮ縺啓閾ｪ辟ｶ/縺励●繧転縺�縺ｨ[蛻､譁ｭ/縺ｯ繧薙□繧転縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/14_refactor_with_properties.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tutorials/getting_started/12_pure_function_pipeline.n.md -i tutorials/getting_started/13_type_driven_error_modeling.n.md -i tutorials/getting_started/14_refactor_with_properties.n.md --no-stdlib --no-tree -o /tmp/tests-safe-result-batch3.json -j 4`
    - [邨先棡/縺代▲縺犠: `6/6 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`Vec<Result>` 縺ｮ print 繧・test [譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｮ[譏守､ｺ蜻ｼ/繧√＞縺倥ｈ]縺ｳ[蜃ｺ/縺�]縺励∈[邨ｱ荳�/縺ｨ縺・＞縺､])

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `checks_push` [荳ｭ/縺｡繧・≧]繧・`checks_exit_code` [蜀・Κ/縺ｪ縺・・]縺ｧ[蜍晄焔/縺九▲縺ｦ]縺ｫ stdout 繧端豎・繧医＃]縺輔★縲》est case [蛛ｴ/縺後ｏ]縺啓譛�蠕・縺輔＞縺脳縺ｫ[譏守､ｺ逧・繧√＞縺倥※縺江縺ｫ print 縺吶ｋ reboot [蠕・縺脳縺ｮ[豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - test tool [蛛ｴ/縺後ｏ]縺ｧ縺ｯ縺ｪ縺・test case [譛ｬ菴・縺ｻ繧薙◆縺Ь縺ｮ[險倩ｿｰ/縺阪§繧・▽]縺九ｉ縲啓菴・縺ｪ縺ｫ]繧端蜃ｺ/縺�]縺吶°縲阪ｒ[隱ｭ/繧・繧√ｋ繧医≧縺ｫ縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - 逶ｴ蜑阪・ `std/test` [謾ｹ菫ｮ/縺九＞縺励ｅ縺・縺ｧ human / machine [陦ｨ遉ｺ/縺ｲ繧・≧縺肋繧端蛻・屬/縺ｶ繧薙ｊ]縺励◆縺後�～checks_exit_code` 縺九ｉ[證鈴ｻ・縺ゅｓ繧ゅ￥]縺ｫ[陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺励※縺・◆[蜷肴ｮ・縺ｪ縺斐ｊ]繧端螳悟・/縺九ｓ縺懊ｓ]縺ｫ縺ｯ[譁ｭ/縺歉縺｡[蛻・縺江繧後※縺・↑縺九▲縺溘�・
  - `checks_print_machine` / `checks_print_human` 繧端蛻･縲・縺ｹ縺､縺ｹ縺､]縺ｫ[蜻ｼ/繧・縺ｶ[譖ｸ/縺犠縺梗譁ｹ/縺九◆]縺�縺ｨ縲》est [譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｫ 1 [蝗・縺九＞]縺�縺措譏守､ｺ/繧√＞縺肋縺励※[蜃ｺ/縺�]縺吶→縺・≧[諢丞峙/縺・→]縺啓蠑ｱ/繧医ｏ]縺九▲縺溘�・
  - 縺輔ｉ縺ｫ print helper 縺・`Vec<Result>` 繧端豸郁ｲｻ/縺励ｇ縺・・]縺励※縺励∪縺・→縲√◎縺ｮ[蠕・縺ゅ→]縺ｧ `checks_exit_code` 縺ｫ[貂｡/繧上◆]縺帙★縲ー蜷域・/縺斐≧縺帙＞]縺励↓縺上°縺｣縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/std/test.nepl`
    - `finish_checks`
      - [陦ｨ遉ｺ/縺ｲ繧・≧縺肋繧端螳悟・/縺九ｓ縺懊ｓ]縺ｫ[螟・縺ｯ縺咯縺励�～Vec<Result<(),str>> -> Result<(),str>` 縺ｮ[邏皮ｲ・縺倥ｅ繧薙☆縺Ь helper 縺ｫ[謌ｻ/繧ゅ←]縺励◆縲・
    - `checks_exit_code`
      - [蜀・Κ/縺ｪ縺・・]縺ｧ print 縺励↑縺・helper 縺ｧ縺ゅｋ縺薙→繧・doc comment 縺ｫ[譏手ｨ・繧√＞縺江縺励◆縲・
    - `checks_print_machine` / `checks_print_human`
      - [陦ｨ遉ｺ/縺ｲ繧・≧縺肋[蠕・縺脳縺ｫ[蜷・縺翫↑]縺・`Vec<Result<(),str>>` 繧端霑・縺九∴]縺・pipe [蜿ｯ閭ｽ/縺九・縺・ API 縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `checks_print_report`
      - test [譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｧ 1 [蝗・縺九＞]縺�縺措蜻ｼ/繧・縺ｶ[逕ｨ騾・繧医≧縺ｨ]縺ｮ helper 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
      - [蜀・Κ/縺ｪ縺・・]縺ｧ縺ｯ machine summary 縺ｮ[陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺ｨ human [蜷・繧�]縺措荳�隕ｧ/縺・■繧峨ｓ][陦ｨ遉ｺ/縺ｲ繧・≧縺肋繧端鬆・分/縺倥ｅ繧薙・繧転縺ｫ[陦・縺翫％縺ｪ]縺・�√◎縺ｮ[蠕・縺ゅ→]縺ｧ `Vec<Result<(),str>>` 繧端霑・縺九∴]縺吶�・
  - `tests/stdlib/std_test_collect.n.md`
    - `checks_print_machine |> checks_print_human` 縺ｮ[蛻・牡/縺ｶ繧薙°縺､]蜻ｼ縺ｳ[蜃ｺ/縺�]縺励ｒ繧・ａ縲～let shown checks_print_report checks` 縺ｫ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励◆縲・
    - [譛溷ｾ・縺阪◆縺Ь stdout 縺ｯ[邯ｭ謖・縺・§]縺励▽縺､縲√�継rint 縺ｯ test [譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｧ[譏守､ｺ逧・繧√＞縺倥※縺江縺ｫ[蜻ｼ/繧・縺ｶ縲阪％縺ｨ縺啓隱ｭ/繧・繧√ｋ fixture 縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `tutorials/getting_started/11_testing_workflow.n.md`
    - [隱ｬ譏取枚/縺帙▽繧√＞縺ｶ繧転繧偵�形Vec<Result<(),str>>` 縺ｮ[陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺ｯ test [譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｧ `checks_print_report` 繧端譏守､ｺ逧・繧√＞縺倥※縺江縺ｫ[蜻ｼ/繧・縺ｶ縲阪∈[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
    - example 繧・`let shown checks_print_report checks` 縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - reboot 縺ｮ縲啓蛟､荳ｭ蠢・縺ゅ◆縺・■繧・≧縺励ｓ]縲阪�啓譏守､ｺ逧・繧√＞縺倥※縺江 API縲阪�啓雋ｬ蜍・縺帙″繧�][蛻・屬/縺ｶ繧薙ｊ]縲阪↓[辣ｧ/縺ｦ]繧峨☆縺ｨ縲～checks_exit_code` 縺・stdout 繧端蜃ｺ/縺�]縺吶・縺ｯ[雋ｬ蜍咎℃螟・縺帙″繧�縺九◆]縺�縺｣縺溘�・
  - print helper 繧・pipe [蜿ｯ閭ｽ/縺九・縺・縺ｫ縺励◆縺ｮ縺ｯ縲¨EPLg2 縺ｮ[蜷域・/縺斐≧縺帙＞][蠢怜髄/縺励％縺・縺ｨ[謨ｴ蜷・縺帙＞縺斐≧]縺励�～checks_print_report checks |> checks_exit_code` [邉ｻ邨ｱ/縺代＞縺ｨ縺・縺ｮ[譖ｸ/縺犠縺梗譁ｹ/縺九◆]縺ｸ繧・諡｡蠑ｵ/縺九￥縺｡繧・≧]縺励ｄ縺吶＞縺溘ａ縺ｧ縺ゅｋ縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tests/stdlib/std_test_collect.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/std_test_collect.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/std_test_collect.n.md -i tutorials/getting_started/11_testing_workflow.n.md -i stdlib/std/test.nepl --no-stdlib --no-tree -o /tmp/tests-explicit-check-print.json -j 4`
    - [邨先棡/縺代▲縺犠: `16/16 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (late getting_started 縺ｨ `hash` fixture 繧・explicit print / safe `Result` 豬∝о縺ｸ霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `19_pipe_operator`, `20_generics_basics`, `21_trait_bounds_basics` 縺ｨ `stdlib/tests/hash.n.md` 繧偵�ー迴ｾ陦・縺偵ｓ縺薙≧]縺ｮ safe `Result` + explicit print [豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - tutorial [邨ら乢/縺励ｅ縺・・繧転縺ｧ繧・`std/test` 縺ｮ[蜿､/縺ｵ繧犠縺・unit-return / [證鈴ｻ・縺ゅｓ繧ゅ￥]陦ｨ遉ｺ[蜑肴署/縺懊ｓ縺ｦ縺Ь繧端谿・縺ｮ縺転縺輔↑縺・�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `19`縲彖21` 縺ｮ doctest 縺ｯ縲√∪縺� unit-return `main` 縺ｨ `assert_*` [逶ｴ蛻・縺｡繧・￥繧後▽][螳溯｡・縺倥▲縺薙≧]縺ｮ[譌ｧ豬∝о/縺阪ｅ縺・ｊ繧・≧縺讃縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
  - `stdlib/tests/hash.n.md` 繧・`test_checked` 繧端騾比ｸｭ/縺ｨ縺｡繧・≧]縺ｧ[蜻ｼ/繧・縺ｶ[蜿､/縺ｵ繧犠縺Ъ蠖｢/縺九◆縺｡]縺ｮ縺ｾ縺ｾ縺ｧ縲～Vec<Result>` 縺ｮ[髮・ｴ・縺励ｅ縺・ｄ縺従縺ｨ test [譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｮ explicit report [譁ｹ驥・縺ｻ縺・＠繧転縺ｫ[荵・縺ｮ]縺｣縺ｦ縺・↑縺九▲縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tutorials/getting_started/19_pipe_operator.n.md`
    - 2 [莉ｶ/縺代ｓ]縺ｮ doctest 縺ｫ `ret: 0` 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `core/result` 繧・import 縺励�～checks_new` / `checks_push` / `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `tutorials/getting_started/20_generics_basics.n.md`
    - generic `id` / generic `Option` 縺ｮ doctest 繧・safe `Result` [豬∝о/繧翫ｅ縺・℃]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `tutorials/getting_started/21_trait_bounds_basics.n.md`
    - trait / impl 縺ｨ trait bound generic 縺ｮ doctest 繧・safe `Result` [豬∝о/繧翫ｅ縺・℃]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `stdlib/tests/hash.n.md`
    - `ret: 0` 繧端霑ｽ蜉�/縺､縺・°]縺励�：NV-1a / `hash32_i32` / SHA-256 skeleton 縺ｮ[遒ｺ隱・縺九￥縺ｫ繧転繧・`Vec<Result<(),str>>` [髮・ｴ・縺励ｅ縺・ｄ縺従縺ｸ[遘ｻ/縺・▽]縺励◆縲・
    - stdout [遒ｺ隱・縺九￥縺ｫ繧転縺ｮ縺ゅｋ fixture 縺ｨ縺励※縲》est [譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｧ `checks_print_report checks` 繧端譏守､ｺ逧・繧√＞縺倥※縺江縺ｫ[蜻ｼ/繧・縺ｶ[蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - tutorial [蛛ｴ/縺後ｏ]縺ｯ stdout [譛溷ｾ・縺阪◆縺Ь縺後↑縺・◆繧√�～checks_exit_code` 縺�縺代ｒ[菴ｿ/縺､縺犠縺・譛�蟆・縺輔＞縺励ｇ縺・讒区・繧端邯ｭ謖・縺・§]縺励◆縲・
  - `hash.n.md` 縺ｯ[蝗槫ｸｰ/縺九＞縺江 fixture 縺ｨ縺励※ stdout [隕ｳ蟇・縺九ｓ縺輔▽]縺ｮ[萓｡蛟､/縺九■]縺後≠繧九◆繧√�～checks_print_report` 繧端蜈･/縺Ь繧後※ explicit print [譁ｹ驥・縺ｻ縺・＠繧転縺ｮ[螳滉ｾ・縺倥▽繧後＞]縺ｫ繧ゅ＠縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/tests/hash.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i tutorials/getting_started/19_pipe_operator.n.md -i tutorials/getting_started/20_generics_basics.n.md -i tutorials/getting_started/21_trait_bounds_basics.n.md -i stdlib/tests/hash.n.md --no-stdlib --no-tree -o /tmp/tests-safe-result-batch5.json -j 4`
    - [邨先棡/縺代▲縺犠: `7/7 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`match` / namespace / recursion tutorial 縺ｮ safe `Result` 蛹・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `15_match_patterns`, `17_namespace_and_alias`, `18_recursion_and_termination` 繧偵�ー迴ｾ陦・縺偵ｓ縺薙≧]縺ｮ safe `Result` [豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - `match` / `::` / [蜀榊ｸｰ/縺輔＞縺江縺ｨ縺・≧[險�隱・縺偵ｓ縺脳[荳ｭ蠢・縺｡繧・≧縺励ｓ]縺ｮ chapter 縺ｫ unit-return test 縺啓谿・縺ｮ縺転繧峨↑縺・ｈ縺・↓縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - 3 [遶�/縺励ｇ縺・縺ｨ繧・`fn main <()*>()> ():` 縺ｨ `assert_*` [逶ｴ蛻・縺｡繧・￥繧後▽][螳溯｡・縺倥▲縺薙≧]縺ｮ[譌ｧ豬∝о/縺阪ｅ縺・ｊ繧・≧縺讃縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
  - reboot [蠕・縺脳縺ｮ `std/test` 縺ｯ `Result<(),str>` [荳ｭ蠢・縺｡繧・≧縺励ｓ]縺ｫ[蜀崎ｨｭ險・縺輔＞縺帙▲縺代＞]縺輔ｌ縺ｦ縺・ｋ縺溘ａ縲√％縺薙′[譌ｧ譚･/縺阪ｅ縺・ｉ縺Ь縺ｮ縺ｾ縺ｾ縺�縺ｨ tutorial [蜈ｨ菴・縺懊ｓ縺溘＞]縺ｧ[豬∝о/繧翫ｅ縺・℃]縺啓謠ｺ/繧・繧後ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tutorials/getting_started/15_match_patterns.n.md`
    - `Option` / `Result` 縺ｮ `match` 萓九ｒ `ret: 0` + `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `tutorials/getting_started/17_namespace_and_alias.n.md`
    - alias [邨檎罰/縺代＞繧・縺ｮ[髢｢謨ｰ蜻ｼ/縺九ｓ縺吶≧繧・縺ｳ[蜃ｺ/縺�]縺励→ `Option::Some` / `Option::None` 萓九ｒ safe `Result` [豬∝о/繧翫ｅ縺・℃]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `tutorials/getting_started/18_recursion_and_termination.n.md`
    - `sum_to` / `fib` 縺ｮ[蜀榊ｸｰ/縺輔＞縺江萓九ｒ `ret: 0` + `checks_exit_code` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - 縺薙ｌ繧峨・ chapter 縺ｯ stdout [豈碑ｼ・縺ｲ縺九￥]繧端莨ｴ/縺ｨ繧ゅ↑]繧上↑縺・◆繧√�～checks_print_report` 縺ｯ[蜈･/縺Ь繧後★縲ー譛�蟆城剞/縺輔＞縺励ｇ縺・￡繧転縺ｮ safe `Result` 縺�縺代ｒ[驕ｩ逕ｨ/縺ｦ縺阪ｈ縺・縺励◆縲・
  - tutorial [譛ｬ譁・縺ｻ繧薙・繧転縺ｮ[荳ｻ鬘・縺励ｅ縺�縺Ь縺ｯ[讒区枚/縺薙≧縺ｶ繧転縺ｪ縺ｮ縺ｧ縲》est helper [蛛ｴ/縺後ｏ]縺ｮ[險倩ｿｰ驥・縺阪§繧・▽繧翫ｇ縺・縺ｯ[蠢・ｦ∵怙菴朱剞/縺ｲ縺､繧医≧縺輔＞縺ｦ縺・￡繧転縺ｫ[逡・縺ｨ縺ｩ]繧√◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/18_recursion_and_termination.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tutorials/getting_started/15_match_patterns.n.md -i tutorials/getting_started/17_namespace_and_alias.n.md -i tutorials/getting_started/18_recursion_and_termination.n.md --no-stdlib --no-tree -o /tmp/tests-safe-result-batch4.json -j 4`
    - [邨先棡/縺代▲縺犠: `6/6 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`list` / `hashset` / `hashset_str` fixture 繧・explicit report 豬∝о縺ｸ霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `stdlib/tests/list.n.md`, `stdlib/tests/hashset.n.md`, `stdlib/tests/hashset_str.n.md` 繧偵�ー迴ｾ陦・縺偵ｓ縺薙≧]縺ｮ `Vec<Result<(),str>>` + explicit report [豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - collection fixture [鄒､/縺舌ｓ]縺ｧ繧・`test_checked` 繧端騾比ｸｭ/縺ｨ縺｡繧・≧]縺ｧ[謖・縺ｯ縺評繧�[蜿､/縺ｵ繧犠縺Ъ譖ｸ/縺犠縺梗譁ｹ/縺九◆]繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励�》est [譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｧ 1 [蝗・縺九＞]縺�縺・`checks_print_report` 繧端蜻ｼ/繧・縺ｶ[讒矩��/縺薙≧縺槭≧]縺ｸ[邨ｱ荳�/縺ｨ縺・＞縺､]縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - 3 fixture 縺ｨ繧・`assert_*` / `test_fail` / `test_checked` 繧端騾先ｬ｡/縺｡縺上§][螳溯｡・縺倥▲縺薙≧]縺吶ｋ[譌ｧ豬∝о/縺阪ｅ縺・ｊ繧・≧縺讃縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
  - `list` 縺ｯ `Option` [蛻・ｲ・縺ｶ繧薙″]縺啓螟・縺翫♀]縺上�～hashset` / `hashset_str` 縺ｯ alias / remove / contains [遒ｺ隱・縺九￥縺ｫ繧転縺啓謨｣蝨ｨ/縺輔ｓ縺悶＞]縺励※縺・※縲ー騾比ｸｭ/縺ｨ縺｡繧・≧]縺ｧ[菴募ｺｦ/縺ｪ繧薙←]繧・success log 繧端蜃ｺ/縺�]縺励※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/tests/list.n.md`
    - `ret: 0` 縺ｨ `core/result` import 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `let mut checks` 繧端蟆主・/縺ｩ縺・↓繧・≧]縺励�～Option` [蛻・ｲ・縺ｶ繧薙″]縺ｮ `Some` / `None` [荳｡譁ｹ/繧翫ｇ縺・⊇縺・繧・`checks_push` 縺ｸ[髮・ｴ・縺励ｅ縺・ｄ縺従縺励◆縲・
    - test [譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｧ `checks_print_report checks` 繧端蜻ｼ/繧・縺ｳ縲√◎縺ｮ[蠕・縺ゅ→]縺ｫ `checks_exit_code` 繧端霑・縺九∴]縺兌蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `stdlib/tests/hashset.n.md`
    - `ret: 0` 繧端霑ｽ蜉�/縺､縺・°]縺励�（nsert / remove / alias [遒ｺ隱・縺九￥縺ｫ繧転繧・`Vec<Result<(),str>>` 縺ｫ[髮・ｴ・縺励ｅ縺・ｄ縺従縺励◆縲・
    - `test_checked "new"` 縺ｪ縺ｩ縺ｮ[騾比ｸｭ/縺ｨ縺｡繧・≧]繝ｭ繧ｰ縺ｯ[髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励�ー譛�蠕・縺輔＞縺脳縺ｫ 1 [蝗・縺九＞]縺�縺・report 繧端蜃ｺ/縺�]縺兌蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `stdlib/tests/hashset_str.n.md`
    - `ret: 0` 繧端霑ｽ蜉�/縺､縺・°]縺励�…ontent / remove / alias [遒ｺ隱・縺九￥縺ｫ繧転繧・`Vec<Result<(),str>>` 縺ｫ[髮・ｴ・縺励ｅ縺・ｄ縺従縺励◆縲・
    - [邨ゆｺ・縺励ｅ縺・ｊ繧・≧] report 縺ｯ `checks_print_report` + `checks_exit_code` [讒区・/縺薙≧縺帙＞]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `hashset` / `hashset_str` 縺ｯ stdout [豈碑ｼ・縺ｲ縺九￥]縺ｮ[萓｡蛟､/縺九■]縺後≠繧・collection fixture 縺ｪ縺ｮ縺ｧ縲》utorial 縺ｨ[逡ｰ/縺薙→]縺ｪ繧・explicit report 繧端谿・縺ｮ縺転縺励◆縲・
  - `list` 繧・騾比ｸｭ/縺ｨ縺｡繧・≧] success log 繧端遨・縺､]繧�繧医ｊ縲ー譛�蠕・縺輔＞縺脳縺ｫ[蜈ｨ菴・縺懊ｓ縺溘＞]繧端隕・縺ｿ]縺帙ｋ縺ｻ縺・′ `Vec<Result>` [險ｭ險・縺帙▲縺代＞]縺ｨ[謨ｴ蜷・縺帙＞縺斐≧]縺吶ｋ縺ｨ[蛻､譁ｭ/縺ｯ繧薙□繧転縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/tests/list.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/hashset.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/hashset_str.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/list.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/hashset_str.n.md --no-stdlib --no-tree -o /tmp/tests-collections-batch1.json -j 4`
    - [邨先棡/縺代▲縺犠: `3/3 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`hashmap` / `hashmap_str` / `rand` / `json` fixture 繧・explicit report 豬∝о縺ｸ霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `stdlib/tests/hashmap.n.md`, `stdlib/tests/hashmap_str.n.md`, `stdlib/tests/rand.n.md`, `stdlib/tests/json.n.md` 繧偵�ー迴ｾ陦・縺偵ｓ縺薙≧]縺ｮ `Vec<Result<(),str>>` + explicit report [豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - [騾比ｸｭ/縺ｨ縺｡繧・≧]縺ｧ `test_checked` 繧・`test_fail` 繧端蜻ｼ/繧・縺ｶ[譌ｧ譚･/縺阪ｅ縺・ｉ縺Ь縺ｮ[螳溯｡・縺倥▲縺薙≧]繝｢繝・Ν繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励�》est [邨ゆｺ・凾/縺励ｅ縺・ｊ繧・≧縺肋縺ｫ 1 [蝗・縺九＞]縺�縺措譏守､ｺ逧・繧√＞縺倥※縺江縺ｫ[陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - 4 fixture 縺ｨ繧・`fn main <()*>()> ():` 縺ｪ縺・＠蜷檎ｭ峨・ unit-return main 縺ｨ縲～assert_*` / `test_fail` / `test_checked` 繧端騾先ｬ｡/縺｡縺上§][螳溯｡・縺倥▲縺薙≧]縺吶ｋ[蜿､/縺ｵ繧犠縺Ъ譖ｸ/縺犠縺梗譁ｹ/縺九◆]縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
  - reboot [蠕・縺脳縺ｮ `std/test` 縺ｯ `finish_checks` 繧・pure 縺ｫ縺励�～checks_exit_code` 縺ｯ stdout 繧端豎・繧医＃]縺輔★縲》est case [蛛ｴ/縺後ｏ]縺・`checks_print_report` 繧端譏守､ｺ逧・繧√＞縺倥※縺江縺ｫ[蜻ｼ/繧・縺ｶ[譁ｹ驥・縺ｻ縺・＠繧転縺ｸ[遘ｻ/縺・▽]縺｣縺ｦ縺・ｋ縺溘ａ縲’ixture [鄒､/縺舌ｓ]縺啓譌ｧ豬∝о/縺阪ｅ縺・ｊ繧・≧縺讃縺ｮ縺ｾ縺ｾ縺�縺ｨ test [菴懈ｳ・縺輔⊇縺・縺啓豺ｷ蝨ｨ/縺薙ｓ縺悶＞]縺励※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/tests/hashmap.n.md`
    - [蜈ｨ/縺吶∋]縺ｦ縺ｮ[讀懈渊/縺代ｓ縺評繧・`checks_push` 縺ｫ[髮・ｴ・縺励ｅ縺・ｄ縺従縺励�～Option::None` [蛻・ｲ・縺ｶ繧薙″]繧・`Result::Err` 縺ｨ縺励※[菫晄戟/縺ｻ縺肋縺吶ｋ[蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - [譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｫ `checks_print_report` 縺ｨ `checks_exit_code` 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
  - `stdlib/tests/hashmap_str.n.md`
    - [譁・ｭ怜・/繧ゅ§繧後▽] key 迚医ｂ[蜷梧ｧ・縺ｩ縺・ｈ縺・縺ｫ縲…ontent [蜷悟�､/縺ｩ縺・■] / update / remove / alias [遒ｺ隱・縺九￥縺ｫ繧転繧・`Vec<Result<(),str>>` 縺ｫ[髮・ｴ・縺励ｅ縺・ｄ縺従縺励◆縲・
  - `stdlib/tests/rand.n.md`
    - [遒ｺ邇・噪/縺九￥繧翫▽縺ｦ縺江縺ｪ[讀懈渊/縺代ｓ縺評繧・`check_ne` [蛻・繧後▽]縺ｫ[謠・縺昴ｍ]縺医�ー邨ゆｺ・凾/縺励ｅ縺・ｊ繧・≧縺肋 report 縺ｮ縺ｿ[陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺吶ｋ[蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `stdlib/tests/json.n.md`
    - `Option::Some` / `Option::None` [蛻・ｲ・縺ｶ繧薙″]縺ｨ `json_is_null` / `json_as_*` [遒ｺ隱・縺九￥縺ｫ繧転繧・`checks_push` 縺ｸ[髮・ｴ・縺励ｅ縺・ｄ縺従縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - 4 fixture 縺ｨ繧・stdout [蜃ｺ蜉・縺励ｅ縺､繧翫ｇ縺従[蜀・ｮｹ/縺ｪ縺・ｈ縺・縺ｫ[隕ｳ貂ｬ萓｡蛟､/縺九ｓ縺昴￥縺九■]縺後≠繧九◆繧√�》utorial [蛛ｴ/縺後ｏ]縺ｮ繧医≧縺ｪ silent `checks_exit_code` [蜊倡峡/縺溘ｓ縺ｩ縺従縺ｧ縺ｯ縺ｪ縺上�～checks_print_report` 繧端譏守､ｺ逧・繧√＞縺倥※縺江縺ｫ[谿・縺ｮ縺転縺励◆縲・
  - `test_fail` 繧端騾比ｸｭ/縺ｨ縺｡繧・≧]縺ｧ[蜻ｼ/繧・縺ｰ縺壹�～Result::Err` 繧端遨・縺､]繧薙〒[譛�蠕・縺輔＞縺脳縺ｫ[陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺吶ｋ縺薙→縺ｧ縲√�啓騾比ｸｭ/縺ｨ縺｡繧・≧]縺ｧ縺ｯ print 縺励↑縺・�阪�罫eturn [逶ｴ蜑・縺｡繧・￥縺懊ｓ]縺ｫ[譏守､ｺ print/繧√＞縺・print] 縺吶ｋ縲阪→縺・≧ reboot [蠕・縺脳 test [蜩ｲ蟄ｦ/縺ｦ縺､縺後￥]縺ｸ[謠・縺昴ｍ]縺医◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/tests/hashmap.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/hashmap_str.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/rand.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/json.n.md -n 1` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/rand.n.md -i stdlib/tests/json.n.md --no-stdlib --no-tree -o /tmp/tests-stdlib-safe-result-batch2.json -j 4`
    - [邨先棡/縺代▲縺犠: `4/4 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`traits_hash` / `traits_serde` stdlib test 繧・explicit report 豬∝о縺ｸ霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `tests/stdlib/traits_hash.n.md` 縺ｨ `tests/stdlib/traits_serde.n.md` 繧偵�ー迴ｾ陦・縺偵ｓ縺薙≧]縺ｮ safe `Result` + explicit report [豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - trait [閭ｽ蜉・縺ｮ縺・ｊ繧・￥]縺昴・繧ゅ・縺ｮ[蝗槫ｸｰ/縺九＞縺江縺ｯ[邯ｭ謖・縺・§]縺励◆縺ｾ縺ｾ縲’ixture [蛛ｴ/縺後ｏ]縺�縺代ｒ reboot [蠕・縺脳縺ｮ test [蜩ｲ蟄ｦ/縺ｦ縺､縺後￥]縺ｸ[蟇・繧・縺帙ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `traits_hash` / `traits_serde` 縺ｯ reboot [蠕・縺脳縺ｫ[霑ｽ蜉�/縺､縺・°]縺輔ｌ縺・trait 蝗槫ｸｰ縺ｪ縺ｮ縺ｫ縲》est case [譛ｬ菴・縺ｻ繧薙◆縺Ь縺ｯ `assert_*` / `test_fail` / `test_checked` 繧端騾先ｬ｡/縺｡縺上§][螳溯｡・縺倥▲縺薙≧]縺吶ｋ[譌ｧ豬∝о/縺阪ｅ縺・ｊ繧・≧縺讃縺ｮ縺ｾ縺ｾ縺�縺｣縺溘�・
  - 縺ｨ縺上↓ `deserialize` 縺ｮ[逡ｰ蟶ｸ邉ｻ/縺・§繧・≧縺代＞]縺ｯ `ParseError` [蛻､螳・縺ｯ繧薙※縺Ь縺ｮ縺溘・縺ｫ[騾比ｸｭ/縺ｨ縺｡繧・≧]縺ｧ log 繧端蜃ｺ/縺�]縺励※縺翫ｊ縲～checks_print_report` 繧端譛�蠕・縺輔＞縺脳縺ｫ 1 [蝗・縺九＞]縺�縺措蜻ｼ/繧・縺ｶ縺ｨ縺・≧[迴ｾ陦・縺偵ｓ縺薙≧] test [譁ｹ驥・縺ｻ縺・＠繧転縺ｨ[荳肴紛蜷・縺ｵ縺帙＞縺斐≧]縺�縺｣縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tests/stdlib/traits_hash.n.md`
    - 2 [莉ｶ/縺代ｓ]縺ｮ doctest 縺ｨ繧・`Vec<Result<(),str>>` 繧端蟆主・/縺ｩ縺・↓繧・≧]縺励�～Hash` trait helper / hashmap / hashset [遒ｺ隱・縺九￥縺ｫ繧転繧・`checks_push` 縺ｫ[髮・ｴ・縺励ｅ縺・ｄ縺従縺励◆縲・
    - `Option::None` [蛻・ｲ・縺ｶ繧薙″]縺ｯ `Result::Err` 縺ｨ縺励※[菫晄戟/縺ｻ縺肋縺励�ー譛�蠕・縺輔＞縺脳縺ｫ `checks_print_report` + `checks_exit_code` 繧端蜻ｼ/繧・縺ｶ[讒矩��/縺薙≧縺槭≧]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `tests/stdlib/traits_serde.n.md`
    - `serialize` / `deserialize` 縺ｮ[蜷・､懈渊/縺九￥縺代ｓ縺評繧・`check_str_eq` / `check_eq_i32` / `check` 縺ｫ[鄂ｮ謠・縺｡縺九ｓ]縺励◆縲・
    - `ParseError` [蛻､螳・縺ｯ繧薙※縺Ь縺ｯ `test_checked` 繧端蜻ｼ/繧・縺ｰ縺・`Result::Ok ()` 繧端遨・縺､]繧�[蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励�『rong error kind 縺ｯ `Result::Err` 縺ｫ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - trait 蝗槫ｸｰ test 縺ｯ stdout [蜃ｺ蜉・縺励ｅ縺､繧翫ｇ縺従繧端隕ｳ蟇・縺九ｓ縺輔▽]縺励◆縺ｻ縺・′[螟ｱ謨礼ｮ・園/縺励▲縺ｱ縺・°縺励ｇ]繧端霑ｽ/縺馨縺・ｄ縺吶＞縺溘ａ縲》utorial 縺ｮ繧医≧縺ｪ silent exit code [蜊倡峡/縺溘ｓ縺ｩ縺従縺ｧ縺ｯ縺ｪ縺・explicit report 繧端谿・縺ｮ縺転縺励◆縲・
  - `Deserialize` 縺ｮ[逡ｰ蟶ｸ邉ｻ/縺・§繧・≧縺代＞]縺ｯ[螟壼・蟯・縺溘・繧薙″]縺�縺後�〉unner [蛛ｴ/縺後ｏ]縺ｮ trap 繧・early print 縺ｫ[鬆ｼ/縺溘ｈ]繧峨★縲ー蛟､/縺ゅ◆縺Ь縺ｨ縺励※[譛�蠕・縺輔＞縺脳縺ｾ縺ｧ[謖・繧・縺｡[驕・縺ｯ縺転縺ｶ[譁ｹ驥・縺ｻ縺・＠繧転繧端蜆ｪ蜈・繧・≧縺帙ｓ]縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tests/stdlib/traits_hash.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/traits_hash.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/traits_serde.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/traits_serde.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/traits_hash.n.md -i tests/stdlib/traits_serde.n.md --no-stdlib --no-tree -o /tmp/tests-traits-safe-result-batch.json -j 4`
    - [邨先棡/縺代▲縺犠: `4/4 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`fs` / `collections_diag` fixture 繧・explicit report 豬∝о縺ｸ霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `tests/stdlib/fs.n.md` 縺ｨ `tests/stdlib/collections_diag.n.md` 繧偵�ー迴ｾ陦・縺偵ｓ縺薙≧]縺ｮ `Vec<Result<(),str>>` + explicit report [豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - `Diag` / `Option` 縺ｮ[雋ｬ蜍・縺帙″繧�]繧端遒ｺ隱・縺九￥縺ｫ繧転縺吶ｋ fixture 繧偵�》rap 繧Ъ騾比ｸｭ/縺ｨ縺｡繧・≧] print 縺ｫ[鬆ｼ/縺溘ｈ]繧峨★縲ー譛�蠕・縺輔＞縺脳縺ｫ 1 [蝗・縺九＞]縺�縺措譏守､ｺ逧・繧√＞縺倥※縺江縺ｫ[陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺吶ｋ[蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `collections_diag` 縺ｮ 6 [莉ｶ/縺代ｓ]縺ｯ縲～Diag` / `Option` 縺ｮ[諢丞袖隲・縺・∩繧阪ｓ]縺ｯ reboot [蠕・縺脳縺ｮ縺ｾ縺ｾ縺ｪ縺ｮ縺ｫ縲’ixture [蛛ｴ/縺後ｏ]縺・`test_fail` / `assert_*` [逶ｴ蛻・縺｡繧・￥繧後▽][螳溯｡・縺倥▲縺薙≧]縺ｮ[譌ｧ豬∝о/縺阪ｅ縺・ｊ繧・≧縺讃縺�縺｣縺溘�・
  - `fs.n.md` 縺ｮ 2 [莉ｶ/縺代ｓ]逶ｮ縺ｯ縲‘xisting file read 繧・generic wasm runner 縺ｧ[遒ｺ隱・縺九￥縺ｫ繧転縺励ｈ縺・→縺励※縺・◆縺後�√％繧後・ host filesystem integration 縺ｫ[萓晏ｭ・縺・◇繧転縺励�《table 縺ｪ doctest [雋ｬ蜍・縺帙″繧�]繧端雜・縺転縺医※縺・◆縲・
  - `nodesrc/run_test.js` 縺ｫ preopen 繧端霑ｽ蜉�/縺､縺・°]縺励※繧・Node WASI [螳溯｡・縺倥▲縺薙≧]縺ｧ縺ｯ positive-path read 縺啓螳牙ｮ・縺ゅｓ縺ｦ縺Ь縺励↑縺九▲縺溘◆繧√�》est [蟇ｾ雎｡/縺溘＞縺励ｇ縺・縺昴・繧ゅ・繧端隕狗峩/縺ｿ縺ｪ縺馨縺兌蠢・ｦ・縺ｲ縺､繧医≧]縺後≠縺｣縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tests/stdlib/collections_diag.n.md`
    - 6 [莉ｶ/縺代ｓ]縺吶∋縺ｦ繧・`Vec<Result<(),str>>` [髮・ｴ・縺励ｅ縺・ｄ縺従縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励�～KeyNotFound` / `CapacityExceeded` / `Option::None` 縺ｮ[遒ｺ隱・縺九￥縺ｫ繧転繧・`check_str_eq` 縺ｾ縺溘・ `Result::Ok/Err` 縺ｨ縺励※[菫晄戟/縺ｻ縺肋縺吶ｋ[蠖｢/縺九◆縺｡]縺ｸ[謠・縺昴ｍ]縺医◆縲・
    - [蜷・縺九￥] doctest 縺ｮ[譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｧ `checks_print_report` + `checks_exit_code` 繧端蜻ｼ/繧・縺ｶ繧医≧縺ｫ縺励◆縲・
  - `tests/stdlib/fs.n.md`
    - missing file [遒ｺ隱・縺九￥縺ｫ繧転繧・explicit report [豬∝о/繧翫ｅ縺・℃]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - existing file read [遒ｺ隱・縺九￥縺ｫ繧転縺ｯ host FS integration 縺ｫ[萓晏ｭ・縺・◇繧転縺励※縺・◆縺溘ａ縲～ByteBuf -> str` helper 縺ｧ縺ゅｋ `fs_bytes_to_string` 縺ｮ[螳牙ｮ・縺ゅｓ縺ｦ縺Ь蝗槫ｸｰ縺ｸ[鄂ｮ謠・縺｡縺九ｓ]縺励◆縲・
    - 縺薙ｌ縺ｫ繧医ｊ縲～std/fs` 縺ｮ binary helper [雋ｬ蜍・縺帙″繧�]縺ｯ[邯ｭ謖・縺・§]縺励▽縺､縲〉unner [迺ｰ蠅・縺九ｓ縺阪ｇ縺・縺ｫ[蟾ｦ蜿ｳ/縺輔ｆ縺・縺輔ｌ繧・fixture 繧端謗帝勁/縺ｯ縺・§繧Ⅹ縺励◆縲・
  - `nodesrc/run_test.js`
    - repository root 繧・WASI preopen 縺ｫ[霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - 縺溘□縺励�∽ｻ雁屓縺ｮ `fs` positive-path case 縺ｯ preopen [霑ｽ蜉�蠕・縺､縺・°縺脳繧・螳牙ｮ・縺ゅｓ縺ｦ縺Ь縺励↑縺九▲縺溘◆繧√�∵怙邨ら噪縺ｪ[隗｣豎ｺ/縺九＞縺代▽]縺ｯ test [雋ｬ蜍・縺帙″繧�]縺ｮ[蛻・縺江繧骸蛻・繧従縺代〒[陦・縺翫％縺ｪ]縺｣縺溘�・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - reboot [譁ｹ驥・縺ｻ縺・＠繧転縺ｧ縺ｯ doctest 縺ｯ縲啓菴ｿ/縺､縺犠縺Ъ譁ｹ/縺九◆]縺ｮ[菫晁ｨｼ/縺ｻ縺励ｇ縺・縲阪′[荳ｻ逶ｮ逧・縺励ｅ繧ゅ￥縺ｦ縺江縺ｧ縺ゅｊ縲”ost 迺ｰ蠅ゼ萓晏ｭ・縺・◇繧転縺ｮ integration [謌仙凄/縺帙＞縺ｲ]縺ｾ縺ｧ[謚ｱ/縺九°]縺・霎ｼ/縺転繧�縺ｹ縺阪〒縺ｯ縺ｪ縺・�・
  - 縺昴・縺溘ａ `tests/stdlib/fs.n.md` 縺ｧ縺ｯ縲“eneric runner 縺ｧ[螳牙ｮ・縺ゅｓ縺ｦ縺Ь縺ｫ[菫晁ｨｼ/縺ｻ縺励ｇ縺・縺ｧ縺阪ｋ `Err` [邨瑚ｷｯ/縺代＞繧江縺ｨ `ByteBuf` helper [邨瑚ｷｯ/縺代＞繧江縺�縺代ｒ[谿・縺ｮ縺転縺励�’ilesystem positive path 縺ｯ[蛻･/縺ｹ縺､]縺ｮ integration [螻､/縺昴≧]縺ｧ[謇ｱ/縺ゅ▽縺犠縺・・縺啓螯･蠖・縺�縺ｨ縺・縺ｨ[蛻､譁ｭ/縺ｯ繧薙□繧転縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tests/stdlib/fs.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/fs.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 3` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 4` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 5` -> pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 6` -> pass
  - `node nodesrc/tests.js -i tests/stdlib/fs.n.md -i tests/stdlib/collections_diag.n.md --no-stdlib --no-tree -o /tmp/tests-fs-collections-diag-explicit.json -j 4`
    - [邨先棡/縺代▲縺犠: `8/8 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`cast` / `math` fixture 縺ｨ `02b` / `16_debug` tutorial 繧・explicit report 豬∝о縺ｸ霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `stdlib/tests/cast.n.md`, `stdlib/tests/math.n.md`, `tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md`, `tutorials/getting_started/16_debug_and_ansi.n.md` 繧偵�ー迴ｾ陦・縺偵ｓ縺薙≧]縺ｮ explicit report [豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - [蜊倡ｴ・縺溘ｓ縺倥ｅ繧転縺ｪ success log [萓晏ｭ・縺・◇繧転縺ｮ case 繧端蜈・縺輔″]縺ｫ[隗｣豸・縺九＞縺励ｇ縺・縺励�～error.n.md` 縺ｮ繧医≧縺ｪ[隍・尅/縺ｵ縺上＊縺､]蛻・ｲ・case 縺ｨ[蛻・屬/縺ｶ繧薙ｊ]縺励※[騾ｲ/縺吶☆]繧√ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `cast` / `math` fixture 縺ｯ縺吶〒縺ｫ `Vec<Result<(),str>>` 繧端蟆主・/縺ｩ縺・↓繧・≧]縺励※縺・◆縺後�ー譛�蠕・縺輔＞縺脳縺�縺・`test_checked` 縺ｫ[鬆ｼ/縺溘ｈ]繧擬驕取ｸ｡譛・縺九→縺江縺ｮ[蠖｢/縺九◆縺｡]縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
  - `02b` tutorial 縺ｯ `assert_*` 縺ｨ `test_checked` 縺ｮ[蜊倡匱/縺溘ｓ縺ｱ縺､] success log 縺ｫ[謌ｻ/繧ゅ←]縺｣縺ｦ縺翫ｊ縲《afe `Result` + explicit print 縺ｮ reboot [蠕・縺脳[譁ｹ驥・縺ｻ縺・＠繧転縺ｨ[荳堺ｸ�閾ｴ/縺ｵ縺・▲縺｡]縺�縺｣縺溘�・
  - `16_debug_and_ansi` 縺ｮ `std/test` 萓九ｂ縲～test_checked` 縺ｮ[譌ｧ/縺阪ｅ縺・ stdout [蠖｢蠑・縺代＞縺励″]繧端蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｫ縺励※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/tests/cast.n.md`
    - [譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｮ `test_checked "cast conversions"` 繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励�～checks_print_report` + `checks_exit_code` 縺ｸ[鄂ｮ謠・縺｡縺九ｓ]縺励◆縲・
  - `stdlib/tests/math.n.md`
    - `cast` fixture 縺ｨ[蜷梧ｧ・縺ｩ縺・ｈ縺・縺ｫ縲ー譛�蠕・縺輔＞縺脳縺ｮ success log 繧・explicit report [蠖｢/縺代＞]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md`
    - 5 [莉ｶ/縺代ｓ]縺ｮ doctest 繧・`i32` return + `Vec<Result<(),str>>` [髮・ｴ・縺励ｅ縺・ｄ縺従縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - [隗｣譫・縺九＞縺帙″]邉ｻ縺ｯ `Result::Err` 繧・message 縺､縺阪〒[遨・縺､]縺ｿ縲ー譛�蠕・縺輔＞縺脳縺ｫ `checks_print_report` 繧端譏守､ｺ逧・繧√＞縺倥※縺江縺ｫ[蜻ｼ/繧・縺ｶ[蠖｢/縺九◆縺｡]縺ｸ[謠・縺昴ｍ]縺医◆縲・
    - `from_i64 sub <i64> cast 0 <i64> cast 42` 縺ｯ `check_str_eq` [遘ｻ陦悟ｾ・縺・％縺・＃]縺ｫ overload [譖匁乂/縺ゅ＞縺ｾ縺Ь縺ｫ縺ｪ縺｣縺溘◆繧√�～neg42` [荳ｭ髢灘�､/縺｡繧・≧縺九ｓ縺｡]繧端蟆主・/縺ｩ縺・↓繧・≧]縺励※[蠑・縺励″][蠅・阜/縺阪ｇ縺・°縺Ь繧端譏守｢ｺ蛹・繧√＞縺九￥縺犠縺励◆縲・
  - `tutorials/getting_started/16_debug_and_ansi.n.md`
    - `std/test` 縺ｨ[邨・縺従縺ｿ[蜷・縺・繧上○繧倶ｾ九ｒ `checks_print_report` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励�《tdout [譛溷ｾ・�､/縺阪◆縺・■]繧・`Checked [ok]` / `[0] ok` [蠖｢蠑・縺代＞縺励″]縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - tutorial 縺ｧ繧・success [陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺ｮ[蜃ｺ謇�/縺ｧ縺ｩ縺薙ｍ]繧・test case [蛛ｴ/縺後ｏ]縺ｸ[蟇・繧・縺帙ｋ縺薙→縺ｧ縲√�罫unner 縺啓蜍晄焔/縺九▲縺ｦ]縺ｫ[陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺吶ｋ縲阪・縺ｧ縺ｯ縺ｪ縺上�荊est case 縺啓譛�蠕・縺輔＞縺脳縺ｫ[譏守､ｺ/繧√＞縺肋縺励※[陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺吶ｋ縲阪→縺・≧ reboot [蠕・縺脳 test [蜩ｲ蟄ｦ/縺ｦ縺､縺後￥]繧端荳�雋ｫ/縺・▲縺九ｓ]縺輔○縺溘�・
  - `16_debug_and_ansi` 縺ｯ ANSI [閾ｪ菴・縺倥◆縺Ь縺ｮ[遒ｺ隱・縺九￥縺ｫ繧転縺ｨ `std/test` [騾｣謳ｺ/繧後ｓ縺代＞]縺ｮ[遒ｺ隱・縺九￥縺ｫ繧転繧端蛻・屬/縺ｶ繧薙ｊ]縺励�ー蠕瑚�・縺薙≧縺励ｃ]縺ｯ `strip_ansi` [荳・縺犠縺ｧ繧・隱ｭ/繧・縺ｿ繧・☆縺・machine/human report [蠖｢蠑・縺代＞縺励″]縺ｸ[霑ｽ蠕・縺､縺・§繧・≧]縺輔○縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/16_debug_and_ansi.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/cast.n.md -i stdlib/tests/math.n.md -i tutorials/getting_started/02b_type_conversion_and_textual_conversion.n.md -i tutorials/getting_started/16_debug_and_ansi.n.md --no-stdlib --no-tree -o /tmp/tests-explicit-report-batch3.json -j 4`
    - [邨先棡/縺代▲縺犠: `9/9 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`error.n.md` 繧・explicit report 豬∝о縺ｸ霑ｽ蠕薙＠縲～todo.md` 縺ｮ莠ｺ蛛ｴ謨ｴ逅・ｒ蜿悶ｊ霎ｼ繧薙□)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `stdlib/tests/error.n.md` 繧偵�～Diag` / `Diags` / `Outcome` 縺ｮ[蛟､/縺ゅ◆縺Ь繝｢繝・Ν繧端菫・縺溘ｂ]縺｣縺溘∪縺ｾ explicit report [豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - [莠ｺ/縺ｲ縺ｨ]縺啓謨ｴ逅・縺帙＞繧馨縺励◆ `todo.md` 繧偵�∫樟迥ｶ縺ｮ reboot [譁ｹ驥・縺ｻ縺・＠繧転縺ｫ[豐ｿ/縺拆縺・蠖｢/縺九◆縺｡]縺ｧ螻･豁ｴ縺ｫ[蜿肴丐/縺ｯ繧薙∴縺Ь縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `stdlib/tests/error.n.md` 縺ｯ `Outcome` / `Diag` [蝗槫ｸｰ/縺九＞縺江縺ｮ[荳ｭ蠢・縺｡繧・≧縺励ｓ] fixture 縺ｪ縺ｮ縺ｫ縲∝推蛻・ｲ舌′ `test_fail` / `assert` 縺ｮ[騾先ｬ｡/縺｡縺上§][螳溯｡・縺倥▲縺薙≧]縺ｫ[逡・縺ｨ縺ｩ]縺ｾ縺｣縺ｦ縺・◆縲・
  - 縺薙ｌ縺ｧ縺ｯ reboot [蠕・縺脳縺ｮ縲啓螟ｱ謨・縺励▲縺ｱ縺Ь繧端蛟､/縺ゅ◆縺Ь縺ｨ縺励※[謖・繧・縺｡[驕・縺ｯ縺転縺ｳ縲》est [譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｧ[譏守､ｺ逧・繧√＞縺倥※縺江縺ｫ report 縺吶ｋ縲阪→縺・≧[譁ｹ驥・縺ｻ縺・＠繧転縺ｨ[荳肴紛蜷・縺ｵ縺帙＞縺斐≧]縺�縺｣縺溘�・
  - `todo.md` 縺ｯ[莠ｺ/縺ｲ縺ｨ]縺ｮ[邱ｨ髮・縺ｸ繧薙＠繧・≧]縺啓螳御ｺ・縺九ｓ繧翫ｇ縺・縺励�∫ｷｨ髮・ｦ∵ｭ｢[鬆伜沺/繧翫ｇ縺・＞縺江縺ｮ[隕句・/縺ｿ縺�]縺励ｄ莉雁ｾ後・[謖・､ｺ/縺励§]縺啓迴ｾ迥ｶ/縺偵ｓ縺倥ｇ縺・縺ｨ[蜷・縺・縺・蠖｢/縺九◆縺｡]縺ｫ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺輔ｌ縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/tests/error.n.md`
    - 3 [莉ｶ/縺代ｓ]縺ｮ doctest 縺吶∋縺ｦ繧・`Vec<Result<(),str>>` [髮・ｴ・縺励ｅ縺・ｄ縺従縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `StdErrorKind` 縺ｮ[螟壼・蟯・縺溘・繧薙″]繧・`Option::None` / `Result::Err` [蛻・ｲ・縺ｶ繧薙″]繧ゅ�ー騾比ｸｭ/縺ｨ縺｡繧・≧]縺ｧ trap 縺帙★ `Result::Ok/Err` 縺ｨ縺励※[菫晄戟/縺ｻ縺肋縺励�ー譛�蠕・縺輔＞縺脳縺ｫ `checks_print_report` + `checks_exit_code` 縺ｸ[逡ｳ/縺溘◆]繧�[蠖｢/縺九◆縺｡]縺ｸ[謠・縺昴ｍ]縺医◆縲・
    - `Outcome` / `Diag` 縺ｮ move model 繧Ъ蜀・Κ/縺ｪ縺・・][陦ｨ迴ｾ/縺ｲ繧・≧縺偵ｓ]縺ｯ[螟・縺犠縺医★縲’ixture [螳溯｡・縺倥▲縺薙≧]繝｢繝・Ν縺�縺代ｒ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
  - `todo.md`
    - [莠ｺ/縺ｲ縺ｨ]縺啓謨ｴ逅・縺帙＞繧馨縺励◆[蜀・ｮｹ/縺ｪ縺・ｈ縺・繧偵◎縺ｮ縺ｾ縺ｾ[蜿・縺ｨ]繧骸霎ｼ/縺転繧薙□縲・
    - LLM [邱ｨ髮・縺ｸ繧薙＠繧・≧][遖∵ｭ｢/縺阪ｓ縺余[鬆伜沺/繧翫ｇ縺・＞縺江縺ｮ[隕句・/縺ｿ縺�]縺励�～nm` 蜀埼幕逋ｺ縲´SP / target / tuple / pattern / [蝙・縺九◆][蜑咲ｽｮ/縺懊ｓ縺｡]險俶ｳ輔↑縺ｩ縺ｮ[谿玖ｪｲ鬘・縺悶ｓ縺九□縺Ь縺後�ー迴ｾ蝨ｨ/縺偵ｓ縺悶＞]縺ｮ reboot [蠕・縺脳[蝨ｰ蝗ｳ/縺｡縺咯縺ｨ縺励※[隱ｭ/繧・縺ｿ繧・☆縺Ъ蠖｢/縺九◆縺｡]縺ｫ縺ｪ縺｣縺溘�・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `error.n.md` 縺ｯ[隍・尅/縺ｵ縺上＊縺､]蛻・ｲ舌□縺後�√�啓騾比ｸｭ/縺ｨ縺｡繧・≧]縺ｧ[關ｽ/縺馨縺ｨ縺吶�阪・縺ｧ縺ｯ縺ｪ縺上�啓譛�蠕・縺輔＞縺脳縺ｾ縺ｧ[蛟､/縺ゅ◆縺Ь縺ｨ縺励※[驕・縺ｯ縺転縺ｶ縲阪％縺ｨ[閾ｪ菴・縺倥◆縺Ь縺・reboot [蠕・縺脳 test [險ｭ險・縺帙▲縺代＞]縺ｮ[荳�驛ｨ/縺・■縺ｶ]縺ｪ縺ｮ縺ｧ縲√◎縺ｮ[譁ｹ驥・縺ｻ縺・＠繧転繧貞━蜈医＠縺溘�・
  - `todo.md` 縺ｯ[莠ｺ/縺ｲ縺ｨ]縺ｮ[諢丞峙/縺・→]縺啓蜿肴丐/縺ｯ繧薙∴縺Ь縺輔ｌ縺歇譛�譁ｰ迚・縺輔＞縺励ｓ縺ｰ繧転繧貞ｱ･豁ｴ縺ｸ[蝗ｺ螳・縺薙※縺Ь縺励※縺翫￥縺ｻ縺・′縲∽ｻ･蠕後・閾ｪ蠕句ｮ溯｣・・[蜑肴署/縺懊ｓ縺ｦ縺Ь繧端蜈ｱ譛・縺阪ｇ縺・ｆ縺・縺励ｄ縺吶＞縺ｨ[蛻､譁ｭ/縺ｯ繧薙□繧転縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/tests/error.n.md -n 1` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/error.n.md -n 2` -> pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/error.n.md -n 3` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/error.n.md --no-stdlib --no-tree -o /tmp/tests-error-explicit.json -j 4`
    - [邨先棡/縺代▲縺犠: `3/3 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`Option` / `Result` / `while` tutorial 繧・explicit report 豬∝о縺ｸ霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `tutorials/getting_started/05_option.n.md`, `tutorials/getting_started/06_result.n.md`, `tutorials/getting_started/07_while_and_block.n.md` 繧偵�ー迴ｾ陦・縺偵ｓ縺薙≧]縺ｮ explicit report [豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - [蜈･髢�/縺ｫ繧・≧繧ゅｓ] chapter 縺ｫ[谿・縺ｮ縺転縺｣縺ｦ縺・ｋ `test_checked` / `test_fail` [荳ｭ蠢・縺｡繧・≧縺励ｓ]縺ｮ[譌ｧ譖ｸ豕・縺阪ｅ縺・＠繧・⊇縺・繧端貂・縺ｸ]繧峨＠縲√�啓譛�蠕・縺輔＞縺脳縺ｫ[譏守､ｺ print/繧√＞縺・print]縲阪☆繧・reboot [蠕・縺脳 test [譁ｹ驥・縺ｻ縺・＠繧転繧・tutorial [蜈ｨ菴・縺懊ｓ縺溘＞]縺ｸ[豬ｸ騾・縺励ｓ縺ｨ縺・縺輔○繧九�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - 3 chapter 縺ｨ繧・`Vec<Result<(),str>>` 縺ｯ[蟆主・/縺ｩ縺・↓繧・≧]縺輔ｌ縺ｦ縺・◆縺後�ー騾比ｸｭ/縺ｨ縺｡繧・≧]縺ｮ `test_fail` 縺ｨ[譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｮ `test_checked` 縺ｫ[萓晏ｭ・縺・◇繧転縺吶ｋ[驕取ｸ｡譛・縺九→縺江縺ｮ[蠖｢/縺九◆縺｡]縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
  - 縺ｨ縺上↓ `Option` / `Result` 縺ｮ[蜈･髢�/縺ｫ繧・≧繧ゅｓ]遶�縺ｧ old style 繧端谿・縺ｮ縺転縺吶→縲∝茜逕ｨ閠・↓縲罫unner 縺啓蜍晄焔/縺九▲縺ｦ]縺ｫ[謌仙粥/縺帙＞縺薙≧]繧端陦ｨ遉ｺ/縺ｲ繧・≧縺肋縺吶ｋ縲阪ｈ縺・↓[隕・縺ｿ]縺医※縺励∪縺・�ー迴ｾ陦・縺偵ｓ縺薙≧]譁ｹ驥昴→[鮨滄ｽｬ/縺昴＃]縺啓逕・縺励ｇ縺・縺倥ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tutorials/getting_started/05_option.n.md`
    - `Some` / `None` [蛻・ｲ・縺ｶ繧薙″]縺ｮ[遒ｺ隱・縺九￥縺ｫ繧転繧・`check_eq_i32` / `Result::Err` / `Result::Ok` 縺ｫ[謠・縺昴ｍ]縺医�ー譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｧ `checks_print_report` 繧端蜻ｼ/繧・縺ｶ[蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `option_unwrap_or` 縺ｮ case 繧・`check_eq_i32` + explicit report 縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `tutorials/getting_started/06_result.n.md`
    - `Ok` / `Err` [蛻・ｲ・縺ｶ繧薙″]縺ｮ[遒ｺ隱・縺九￥縺ｫ繧転繧・`check_eq_i32` / `check_str_eq` / `Result::Err` 縺ｫ[謠・縺昴ｍ]縺医◆縲・
    - `safe_div2` 縺ｮ example 繧・蜷梧ｧ・縺ｩ縺・ｈ縺・縺ｫ縲～checks_print_report` + `checks_exit_code` 縺ｸ[遘ｻ陦・縺・％縺・縺励◆縲・
  - `tutorials/getting_started/07_while_and_block.n.md`
    - `while` 縺ｨ `block` 縺ｮ[遒ｺ隱・縺九￥縺ｫ繧転繧・`check_eq_i32` + explicit report 縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - 縺薙ｌ繧峨・[險�隱・縺偵ｓ縺脳[蝓ｺ譛ｬ/縺阪⊇繧転縺ｮ chapter 縺ｪ縺ｮ縺ｧ縲》est helper 縺ｮ[險倩ｿｰ驥・縺阪§繧・▽繧翫ｇ縺・縺ｯ[蠅・縺ｵ]繧・＠縺吶℃縺壹�ー譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｮ report 縺�縺代ｒ[譏守､ｺ/繧√＞縺肋縺吶ｋ[譛�蟆・縺輔＞縺励ｇ縺・螟画峩縺ｫ[逡・縺ｨ縺ｩ]繧√◆縲・
  - `test_fail` 繧・helper 縺ｨ縺励※[菴ｿ/縺､縺犠縺Ъ邯・縺､縺･]縺代ｋ繧医ｊ縲～Result::Err` 繧端逶ｴ謗･/縺｡繧・￥縺帙▽][遨・縺､]繧�縺ｻ縺・′縲啓螟ｱ謨・縺励▲縺ｱ縺Ь繧・蛟､/縺ゅ◆縺Ь縺ｧ縺ゅｋ縲阪→縺・≧ reboot [蠕・縺脳縺ｮ test [蜩ｲ蟄ｦ/縺ｦ縺､縺後￥]縺ｫ[豐ｿ/縺拆縺・→[蛻､譁ｭ/縺ｯ繧薙□繧転縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/tests.js -i tutorials/getting_started/05_option.n.md -i tutorials/getting_started/06_result.n.md -i tutorials/getting_started/07_while_and_block.n.md --no-stdlib --no-tree -o /tmp/tests-tutorial-option-result-while.json -j 4`
    - [邨先棡/縺代▲縺犠: `6/6 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`if` / `import` / `testing workflow` tutorial 繧・explicit report 豬∝о縺ｸ霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `tutorials/getting_started/08_if_layouts.n.md`, `tutorials/getting_started/09_import_and_structure.n.md`, `tutorials/getting_started/11_testing_workflow.n.md` 繧偵�ー迴ｾ陦・縺偵ｓ縺薙≧]縺ｮ explicit report [豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - [譌ｧ/縺阪ｅ縺・ `test_checked` success log 縺ｨ縲√◎縺ｮ[蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｧ[譖ｸ/縺犠縺九ｌ縺ｦ縺・◆ `11_testing_workflow` 縺ｮ[隱ｬ譏・縺帙▽繧√＞]繧偵�…urrent 縺ｮ `checks_print_report` [荳ｭ蠢・縺｡繧・≧縺励ｓ] API 縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `08_if_layouts` 縺ｨ `09_import_and_structure` 縺ｯ縲～Vec<Result<(),str>>` 繧端菴ｿ/縺､縺犠縺｣縺ｦ縺・ｋ縺ｫ繧ゅ°縺九ｏ繧峨★縲ー譛�蠕・縺輔＞縺脳縺�縺・`test_checked` 縺ｫ[謌ｻ/繧ゅ←]繧擬驕取ｸ｡譛・縺九→縺江縺ｮ[譖ｸ/縺犠縺梗譁ｹ/縺九◆]縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
  - `11_testing_workflow` 縺ｯ chapter [閾ｪ菴・縺倥◆縺Ь縺啓譌ｧ豬∝о/縺阪ｅ縺・ｊ繧・≧縺讃縺ｮ[隱ｬ譏・縺帙▽繧√＞]繧端蜷ｫ/縺ｵ縺従繧薙〒縺翫ｊ縲～test_checked` 繧端逶ｴ謗･/縺｡繧・￥縺帙▽][蜻ｼ/繧・縺ｶ example 縺ｨ[譌ｧ stdout 譛溷ｾ・�､/縺阪ｅ縺・stdout 縺阪◆縺・■]縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tutorials/getting_started/08_if_layouts.n.md`
    - 4 [莉ｶ/縺代ｓ]縺ｮ doctest 繧・`check_eq_i32` + `checks_print_report` 縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `tutorials/getting_started/09_import_and_structure.n.md`
    - `pipeline_like` 縺ｮ[遒ｺ隱・縺九￥縺ｫ繧転繧・explicit report [蠖｢/縺代＞]縺ｸ[遘ｻ陦・縺・％縺・縺励◆縲・
  - `tutorials/getting_started/11_testing_workflow.n.md`
    - [譛ｬ譁・縺ｻ繧薙・繧転縺ｮ[隱ｬ譏・縺帙▽繧√＞]繧・`check_*` / `finish_checks` / `checks_print_report` [荳ｭ蠢・縺｡繧・≧縺励ｓ]縺ｸ[譖ｸ/縺犠縺梗謠・縺犠縺医◆縲・
    - `std/test` 縺ｨ[邨・縺従縺ｿ[蜷・縺・繧上○繧・example 縺ｯ `Vec<Result<(),str>>` 繧・2 [莉ｶ/縺代ｓ][遨・縺､]縺ｿ縲ー譛�蠕・縺輔＞縺脳縺ｫ `checks_print_report` 繧端譏守､ｺ/繧√＞縺肋縺吶ｋ[蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - stdout [譛溷ｾ・�､/縺阪◆縺・■]繧・`Checked [ok,ok]` / `[0] ok` / `[1] ok` [蠖｢蠑・縺代＞縺励″]縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `11_testing_workflow` 縺ｯ test [譁ｹ驥・縺ｻ縺・＠繧転縺昴・繧ゅ・繧端謨・縺翫＠]縺医ｋ chapter 縺ｪ縺ｮ縺ｧ縲√％縺薙′[迴ｾ陦・縺偵ｓ縺薙≧] API 縺ｨ[鬟・縺従縺Ъ驕・縺｡縺珪縺・→ repo [蜈ｨ菴・縺懊ｓ縺溘＞]縺ｮ[譁ｹ蜷・縺ｻ縺・％縺・繧端隱､隱伜ｰ・縺斐ｆ縺・←縺・縺吶ｋ縲ゅ◎縺ｮ縺溘ａ縲∝ｮ溯｣・､画峩縺�縺代〒縺ｪ縺充隱ｬ譏・縺帙▽繧√＞]繧・蜷梧凾/縺ｩ縺・§]縺ｫ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
  - tutorial [蛛ｴ/縺後ｏ]縺ｧ繧・success [陦ｨ遉ｺ/縺ｲ繧・≧縺肋繧・test case [譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｮ[譏守､ｺ print/繧√＞縺・print]縺ｸ[謠・縺昴ｍ]縺医ｋ縺薙→縺ｧ縲〉unner [萓晏ｭ・縺・◇繧転縺ｧ縺ｯ縺ｪ縺・code [閾ｪ菴・縺倥◆縺Ь縺ｮ[諢丞峙/縺・→]縺ｨ縺励※[隱ｭ/繧・繧√ｋ繧医≧縺ｫ縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/11_testing_workflow.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i tutorials/getting_started/08_if_layouts.n.md -i tutorials/getting_started/09_import_and_structure.n.md -i tutorials/getting_started/11_testing_workflow.n.md --no-stdlib --no-tree -o /tmp/tests-tutorial-if-import-testing.json -j 4`
    - [邨先棡/縺代▲縺犠: `8/8 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`02_numbers` / `03_functions` tutorial 繧・explicit report 豬∝о縺ｸ霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `tutorials/getting_started/02_numbers_and_variables.n.md` 縺ｨ `tutorials/getting_started/03_functions.n.md` 繧偵�ー迴ｾ陦・縺偵ｓ縺薙≧]縺ｮ explicit report [豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - [譛�蛻晄悄/縺輔＞縺励ｇ縺江 tutorial 縺ｫ[谿・縺ｮ縺転縺｣縺ｦ縺・ｋ `test_checked` success log 繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励�ー譁ｰ/縺ゅ◆繧云縺励＞ test [譖ｸ蠑・縺励ｇ縺励″]繧端蠎冗乢/縺倥ｇ縺ｰ繧転縺九ｉ[荳�雋ｫ/縺・▲縺九ｓ]縺励※[遉ｺ/縺励ａ]縺吶�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `02_numbers_and_variables` 縺ｨ `03_functions` 縺ｯ[荳ｻ鬘・縺励ｅ縺�縺Ь縺啓蝓ｺ譛ｬ/縺阪⊇繧転[讒区枚/縺薙≧縺ｶ繧転縺ｧ縺ゅｋ縺ｫ繧ゅ°縺九ｏ繧峨★縲》est 驛ｨ蛻・□縺代′[驕取ｸ｡譛・縺九→縺江縺ｮ `test_checked` [萓晏ｭ・縺・◇繧転縺ｮ縺ｾ縺ｾ縺�縺｣縺溘�・
  - [蛻ｩ逕ｨ閠・繧翫ｈ縺・＠繧ゾ縺啓譛�蛻・縺輔＞縺励ｇ]縺ｫ[隗ｦ/縺ｵ]繧後ｋ chapter 縺ｧ old style 縺啓谿・縺ｮ縺転縺｣縺ｦ縺・ｋ縺ｨ縲〉epo [蜈ｨ菴・縺懊ｓ縺溘＞]縺ｧ[謗｡逕ｨ/縺輔＞繧医≧]縺励※縺・ｋ reboot [蠕・縺脳 test [譁ｹ驥・縺ｻ縺・＠繧転縺啓莨・縺､縺歉繧上ｊ縺ｫ縺上＞縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tutorials/getting_started/02_numbers_and_variables.n.md`
    - 5 [莉ｶ/縺代ｓ]縺ｮ doctest 縺吶∋縺ｦ繧・`check_eq_i32` + `checks_print_report` [讒区・/縺薙≧縺帙＞]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `tutorials/getting_started/03_functions.n.md`
    - `function call`, `inline if expression`, `if colon form` 縺ｮ 3 [莉ｶ/縺代ｓ]繧端蜷梧ｧ・縺ｩ縺・ｈ縺・縺ｫ explicit report [蠖｢/縺代＞]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - 縺薙ｌ繧峨・ chapter 縺ｧ縺ｯ test helper 縺啓荳ｻ鬘・縺励ｅ縺�縺Ь縺ｧ縺ｯ縺ｪ縺・◆繧√�～check_eq_i32` 縺ｨ `checks_print_report` 縺�縺代ｒ[菴ｿ/縺､縺犠縺・譛�蟆・縺輔＞縺励ｇ縺・螟画峩縺ｧ[謠・縺昴ｍ]縺医◆縲・
  - [陦ｨ遉ｺ/縺ｲ繧・≧縺肋繧・test case [譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｫ[髮・ｴ・縺励ｅ縺・ｄ縺従縺吶ｋ[蠖｢/縺九◆縺｡]縺ｫ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励◆縺薙→縺ｧ縲√�罫unner 縺啓蜍晄焔/縺九▲縺ｦ]縺ｫ[謌仙粥/縺帙＞縺薙≧]繧端蜃ｺ/縺�]縺吶�阪・縺ｧ縺ｯ縺ｪ縺上�慶ode [蛛ｴ/縺後ｏ]縺啓譛�蠕・縺輔＞縺脳縺ｫ[譏守､ｺ/繧√＞縺肋縺吶ｋ縲阪→縺・≧ reboot [蠕・縺脳 test [蜩ｲ蟄ｦ/縺ｦ縺､縺後￥]縺ｸ[豐ｿ/縺拆繧上○縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/tests.js -i tutorials/getting_started/02_numbers_and_variables.n.md -i tutorials/getting_started/03_functions.n.md --no-stdlib --no-tree -o /tmp/tests-tutorial-numbers-functions.json -j 4`
    - [邨先棡/縺代▲縺犠: `8/8 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`12` / `13` / `14` tutorial 繧・explicit report 豬∝о縺ｸ霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `tutorials/getting_started/12_pure_function_pipeline.n.md`, `tutorials/getting_started/13_type_driven_error_modeling.n.md`, `tutorials/getting_started/14_refactor_with_properties.n.md` 繧偵�ー迴ｾ陦・縺偵ｓ縺薙≧]縺ｮ explicit report [豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - [邏皮ｲ・縺倥ｅ繧薙☆縺Ь[髢｢謨ｰ/縺九ｓ縺吶≧]縲～Result` / `Option` 縺ｫ繧医ｋ[螟ｱ謨・縺励▲縺ｱ縺Ь[陦ｨ迴ｾ/縺ｲ繧・≧縺偵ｓ]縲ー蝗槫ｸｰ/縺九＞縺江[豈碑ｼ・縺ｲ縺九￥] helper 縺ｧ繧ゅ�｛ld style success log 繧端谿・縺ｮ縺転縺輔↑縺・�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - 3 chapter 縺ｨ繧・`Vec<Result<(),str>>` 繧端菴ｿ/縺､縺犠縺｣縺ｦ縺・※繧ゅ�ー譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｮ `test_checked` 繧・`test_fail` 縺ｸ[謌ｻ/繧ゅ←]繧擬驕取ｸ｡譛・縺九→縺江縺ｮ[譖ｸ/縺犠縺梗譁ｹ/縺九◆]縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
  - 縺ｨ縺上↓ `14_refactor_with_properties.n.md` 縺ｮ `assert_same` 縺ｯ縲ー蟾ｮ蛻・縺輔・繧転 helper [閾ｪ菴・縺倥◆縺Ь縺・`assert_eq_i32` / `test_fail` 縺ｫ[萓晏ｭ・縺・◇繧転縺励※縺翫ｊ縲√�啓螟ｱ謨・縺励▲縺ｱ縺Ь繧・蛟､/縺ゅ◆縺Ь縺ｨ縺励※[謖・繧・縺､縲阪→縺・≧ reboot [蠕・縺脳譁ｹ驥昴′ helper [蜀・Κ/縺ｪ縺・・]縺ｧ[騾泌・/縺ｨ縺讃繧後※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tutorials/getting_started/12_pure_function_pipeline.n.md`
    - 2 [莉ｶ/縺代ｓ]縺ｮ doctest 繧・`check_eq_i32` + `checks_print_report` 縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `tutorials/getting_started/13_type_driven_error_modeling.n.md`
    - `checked_half` / `choose_positive` 縺ｮ[遒ｺ隱・縺九￥縺ｫ繧転繧・`check_eq_i32` / `check_str_eq` / `Result::Err` [逶ｴ謗･/縺｡繧・￥縺帙▽][遨・縺､]縺ｿ縺ｸ[謠・縺昴ｍ]縺医◆縲・
  - `tutorials/getting_started/14_refactor_with_properties.n.md`
    - `sum_to_loop` / `sum_to_formula` 縺ｮ[豈碑ｼ・縺ｲ縺九￥]繧・`check_eq_i32` [荳ｭ蠢・縺｡繧・≧縺励ｓ]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `assert_same` 縺ｯ `check_eq_i32` 繧端霑・縺九∴]縺励�［ismatch 縺ｯ `Result::Err` 繧端霑・縺九∴]縺・helper 縺ｸ[謨ｴ逅・縺帙＞繧馨縺励◆縲・
    - 縺薙ｌ縺ｫ繧医ｊ helper [閾ｪ菴・縺倥◆縺Ь繧・reboot [蠕・縺脳縺ｮ `Result<(),str>` [荳ｭ蠢・縺｡繧・≧縺励ｓ] test [蜩ｲ蟄ｦ/縺ｦ縺､縺後￥]縺ｫ[豐ｿ/縺拆縺・蠖｢/縺九◆縺｡]縺ｫ縺ｪ縺｣縺溘�・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `14_refactor` 縺ｮ helper 縺ｯ[荳�隕・縺・▲縺代ｓ]蟆上＆縺・′縲ー蟆・擂/縺励ｇ縺・ｉ縺Ь縺ｮ property-like [豈碑ｼ・縺ｲ縺九￥] helper 縺ｮ[髮帛ｽ｢/縺ｲ縺ｪ縺後◆]縺ｧ繧ゅ≠繧九◆繧√�√�径ssert helper 縺啓蜊ｳ蠎ｧ/縺昴￥縺望縺ｫ print/trap 縺吶ｋ縲阪・縺ｧ縺ｯ縺ｪ縺上�敬elper [閾ｪ菴・縺倥◆縺Ь縺・`Result` 繧端霑・縺九∴]縺吶�梗譁ｹ蜷・縺ｻ縺・％縺・縺ｸ[蟇・繧・縺帙◆縲・
  - `13_type_driven_error_modeling` 縺ｯ chapter [蜷・繧√＞]縺ｩ縺翫ｊ縲啓蝙・縺九◆]縺啓螟ｱ謨・縺励▲縺ｱ縺Ь繧端陦ｨ/縺ゅｉ繧従縺吶�阪％縺ｨ繧端謨・縺翫＠]縺医ｋ縺ｮ縺ｧ縲》est [譛ｬ菴・縺ｻ繧薙◆縺Ь繧・`Result::Err` 繧端逶ｴ謗･/縺｡繧・￥縺帙▽][遨・縺､]繧�[譖ｸ/縺犠縺梗譁ｹ/縺九◆]縺ｫ[邨ｱ荳�/縺ｨ縺・＞縺､]縺吶ｋ縺ｮ縺啓閾ｪ辟ｶ/縺励●繧転縺ｨ[蛻､譁ｭ/縺ｯ繧薙□繧転縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/tests.js -i tutorials/getting_started/12_pure_function_pipeline.n.md -i tutorials/getting_started/13_type_driven_error_modeling.n.md -i tutorials/getting_started/14_refactor_with_properties.n.md --no-stdlib --no-tree -o /tmp/tests-tutorial-pipeline-modeling-refactor.json -j 4`
    - [邨先棡/縺代▲縺犠: `6/6 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`15` / `17` / `18` / `19` / `20` / `21` tutorial 繧・explicit report 豬∝о縺ｸ霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `tutorials/getting_started/15_match_patterns.n.md`, `17_namespace_and_alias.n.md`, `18_recursion_and_termination.n.md`, `19_pipe_operator.n.md`, `20_generics_basics.n.md`, `21_trait_bounds_basics.n.md` 繧偵�ー迴ｾ陦・縺偵ｓ縺薙≧]縺ｮ explicit report [豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - tutorial [蠕悟濠/縺薙≧縺ｯ繧転縺ｫ[谿・縺ｮ縺転縺｣縺ｦ縺・◆ `test_checked` success log 繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励�ー`match` / namespace / recursion / pipe / generics / trait bounds] 縺ｮ萓九ｂ current 縺ｮ test [譖ｸ蠑・縺励ｇ縺励″]縺ｫ[邨ｱ荳�/縺ｨ縺・＞縺､]縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - 6 chapter 縺ｨ繧・讀懈渊/縺代ｓ縺評縺ｮ[譛ｬ菴・縺ｻ繧薙◆縺Ь縺ｯ縺吶〒縺ｫ `Vec<Result<(),str>>` [荳ｭ蠢・縺｡繧・≧縺励ｓ]縺ｸ[蟇・繧・縺｣縺ｦ縺・◆縺後�ー譛�蠕・縺輔＞縺脳縺�縺・old style 縺ｮ `test_checked` 縺ｫ[萓晏ｭ・縺・◇繧転縺励※縺・◆縲・
  - 縺薙ｌ縺啓谿・縺ｮ縺転繧九→縲碁�比ｸｭ縺ｯ new style縲∵怙蠕後□縺・old style縲阪→縺・≧[豺ｷ蝨ｨ/縺薙ｓ縺悶＞]縺啓邯・縺､縺･]縺阪�》utorial [蜈ｨ菴・縺懊ｓ縺溘＞]縺ｧ[荳�雋ｫ/縺・▲縺九ｓ]縺励◆[險倩ｿｰ/縺阪§繧・▽]縺ｫ縺ｪ繧峨↑縺・�・
- [螟画峩/縺ｸ繧薙％縺・:
  - 6 chapter 縺ｮ doctest 縺吶∋縺ｦ縺ｧ縲～assert_*` 繧・`check_*` 縺ｸ[鄂ｮ謠・縺｡縺九ｓ]縺励�ー譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｮ `test_checked` 繧・`checks_print_report` + `checks_exit_code` 縺ｸ[鄂ｮ謠・縺｡縺九ｓ]縺励◆縲・
  - `20_generics_basics.n.md` 縺ｧ縺ｯ `assert_str_eq` 繧・`check_str_eq` 縺ｸ[謠・縺昴ｍ]縺医◆縲・
  - `21_trait_bounds_basics.n.md` 繧・`trait and impl` / `trait bound generic` 縺ｮ 2 case 繧端蜷梧ｧ・縺ｩ縺・ｈ縺・縺ｫ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - 縺薙ｌ繧峨・[隱ｬ譏・縺帙▽繧√＞]縺ｮ[荳ｻ鬘・縺励ｅ縺�縺Ь縺啓險�隱・縺偵ｓ縺脳[讖溯・/縺阪・縺・縺昴・繧ゅ・縺ｧ縺ゅｊ縲》est helper 縺ｮ[邏ｰ驛ｨ/縺輔＞縺ｶ]繧端蠅・縺ｵ]繧・☆縺ｹ縺阪〒縺ｯ縺ｪ縺・�ゅ◎縺ｮ縺溘ａ縲～check_*` 縺ｨ `checks_print_report` 縺�縺代∈[蟇・繧・縺帙ｋ[譛�蟆・縺輔＞縺励ｇ縺・螟画峩縺ｫ[逡・縺ｨ縺ｩ]繧√◆縲・
  - tutorial [蠕悟濠/縺薙≧縺ｯ繧転縺ｧ繧・explicit report 繧端蠕ｹ蠎・縺ｦ縺｣縺ｦ縺Ь縺吶ｋ縺薙→縺ｧ縲〉epo [蜈ｨ菴・縺懊ｓ縺溘＞]縺ｨ縺励※縲茎uccess log 縺ｯ test case [譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｮ[譏守､ｺ print/繧√＞縺・print]縺九ｉ縺�縺措蜃ｺ/縺ｧ]繧九�阪→縺・≧[譁ｹ驥・縺ｻ縺・＠繧転繧端蜈ｱ譛・縺阪ｇ縺・ｆ縺・縺ｧ縺阪ｋ繧医≧縺ｫ縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/tests.js -i tutorials/getting_started/15_match_patterns.n.md -i tutorials/getting_started/17_namespace_and_alias.n.md -i tutorials/getting_started/18_recursion_and_termination.n.md -i tutorials/getting_started/19_pipe_operator.n.md -i tutorials/getting_started/20_generics_basics.n.md -i tutorials/getting_started/21_trait_bounds_basics.n.md --no-stdlib --no-tree -o /tmp/tests-tutorial-late-basics.json -j 4`
    - [邨先棡/縺代▲縺犠: `12/12 pass`

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`vec` / `list` / `fs` fixture 縺ｨ `23_competitive_sort` 繧・current 莉墓ｧ倥∈謠・∴縲～kpsearch` Vec wrapper 繧剃ｿｮ豁｣)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `stdlib/tests/vec.n.md`, `stdlib/tests/list.n.md`, `stdlib/tests/fs.n.md`, `tutorials/getting_started/23_competitive_sort_and_search.n.md` 縺ｫ[谿・縺ｮ縺転縺｣縺ｦ縺・◆ old helper [蜻ｼ/繧・縺ｳ[蜃ｺ/縺�]縺励ｒ[髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励�‘xplicit report [豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - `23_competitive_sort` 縺ｮ `lower_bound_vec_i32` / `upper_bound_vec_i32` / `count_equal_range_vec_i32` 縺・current move model 縺ｨ[謨ｴ蜷・縺帙＞縺斐≧]縺吶ｋ繧医≧縲～kpsearch` 譛ｬ菴薙・ `Vec<i32>` wrapper 繧端譬ｹ譛ｬ/縺薙ｓ縺ｽ繧転縺九ｉ[菫ｮ豁｣/縺励ｅ縺・○縺Ь縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `vec` / `list` fixture 縺ｯ explicit report [蠖｢/縺代＞]縺ｸ[蟇・繧・縺｣縺ｦ縺・◆縺後�～test_fail` / `assert_*` helper [蜻ｼ/繧・縺ｳ[蜃ｺ/縺�]縺励′[谿・縺ｮ縺転繧翫�∝ｮ悟・縺ｫ縺ｯ current [譁ｹ驥・縺ｻ縺・＠繧転縺ｸ[蜿取據/縺励ｅ縺・◎縺従縺励※縺・↑縺九▲縺溘�・
  - `stdlib/tests/fs.n.md` 繧・unit-return + `test_fail` [逶ｴ蜻ｼ/縺倥°繧・縺ｳ縺ｮ[蜿､/縺ｵ繧犠縺Ъ蠖｢/縺九◆縺｡]縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
  - `23_competitive_sort_and_search.n.md` 縺ｮ 2 [莉ｶ/縺代ｓ]逶ｮ縺啓遨ｺ蜃ｺ蜉・縺九ｉ縺励ｅ縺､繧翫ｇ縺従縺ｫ縺ｪ縺｣縺歇逵溷屏/縺励ｓ縺・ｓ]縺ｯ tutorial [蛛ｴ/縺後ｏ]縺ｧ縺ｯ縺ｪ縺上�ーstdlib/kp/kpsearch.nepl](/mnt/d/project/NEPLg2/stdlib/kp/kpsearch.nepl) 縺ｮ `*_vec_i32` wrapper 縺・`v` 繧・2 [蝗・縺九＞][隱ｭ/繧・繧�[螳溯｣・縺倥▲縺昴≧]縺ｧ current move model 縺ｨ[荳肴紛蜷・縺ｵ縺帙＞縺斐≧]縺�縺｣縺溘％縺ｨ縺�縺｣縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/tests/vec.n.md`
    - `assert` / `assert_eq_i32` / `test_fail` 繧・`check` / `check_eq_i32` / `Result::Err` [逶ｴ謗･/縺｡繧・￥縺帙▽][遨・縺､]縺ｿ縺ｸ[鄂ｮ謠・縺｡縺九ｓ]縺励�ー譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]繧・`checks_print_report` + `checks_exit_code` 縺ｸ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励◆縲・
  - `stdlib/tests/list.n.md`
    - `Option::None` [蛻・ｲ・縺ｶ繧薙″]縺ｮ `test_fail` 繧・`Result::Err` 縺ｸ[鄂ｮ謠・縺｡縺九ｓ]縺励�《uccess [蛛ｴ/縺後ｏ]繧・`check_*` [荳ｭ蠢・縺｡繧・≧縺励ｓ]縺ｸ[謠・縺昴ｍ]縺医◆縲・
  - `stdlib/tests/fs.n.md`
    - missing file case 繧・`i32` return + explicit report [蠖｢/縺代＞]縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
  - `tutorials/getting_started/23_competitive_sort_and_search.n.md`
    - `sort_quick on Vec<i32>` 繧・`check` + explicit report [蠖｢/縺代＞]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `lower_bound` / `upper_bound` / `count_equal_range` 萓九・縲『rapper 菫ｮ豁｣蠕後・ API 縺ｫ[萓晏ｭ・縺・◇繧転縺吶ｋ[蠖｢/縺九◆縺｡]縺ｧ縺昴・縺ｾ縺ｾ[蜍穂ｽ・縺ｩ縺・＆]縺吶ｋ繧医≧縺ｫ縺励◆縲・
  - `stdlib/kp/kpsearch.nepl`
    - `lower_bound_vec_i32` / `upper_bound_vec_i32` / `contains_vec_i32` / `count_equal_range_vec_i32` 縺ｧ縲～Vec<i32>` 繧・temporary memory 縺ｫ 1 [蝗・縺九＞]縺�縺措騾�驕ｿ/縺溘＞縺ｲ]縺励�√◎縺薙°繧・`data` / `len` 繧端謚ｽ蜃ｺ/縺｡繧・≧縺励ｅ縺､]縺励※ raw-array helper 縺ｸ[貂｡/繧上◆]縺兌螳溯｣・縺倥▲縺昴≧]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - 縺薙ｌ縺ｫ繧医ｊ `Vec` 繧・2 [蝗・縺九＞][隱ｭ/繧・繧�[讒矩��/縺薙≧縺槭≧]繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励�…urrent move model 縺ｸ[謨ｴ蜷・縺帙＞縺斐≧]縺輔○縺溘�・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `23_competitive_sort` 縺ｮ[螟ｱ謨・縺励▲縺ｱ縺Ь縺ｯ tutorial [蛛ｴ/縺後ｏ]縺ｮ[譖ｸ/縺犠縺梗譁ｹ/縺九◆]縺ｧ縺ｯ縺ｪ縺・wrapper 譛ｬ菴薙・[謇�譛画ｨｩ/縺励ｇ繧・≧縺代ｓ][謇ｱ/縺ゅ▽縺犠縺・′[蜿､/縺ｵ繧犠縺九▲縺溘％縺ｨ縺啓逵溷屏/縺励ｓ縺・ｓ]縺�縺｣縺溘◆繧√�》utorial 縺�縺代・[霑ょ屓/縺・°縺Ь縺ｧ縺ｯ縺ｪ縺・`kpsearch` 譛ｬ菴薙ｒ[菫ｮ豁｣/縺励ｅ縺・○縺Ь縺励◆縲・
  - `Vec` 縺九ｉ `ptr` / `len` 繧端蜿・縺ｨ]繧・wrapper 縺ｯ[莉雁ｾ・縺薙ｓ縺脳繧・蜀咲匱/縺輔＞縺ｯ縺､]縺励ｄ縺吶＞邂・園縺ｪ縺ｮ縺ｧ縲√�荊emporary memory 縺ｫ[騾�驕ｿ/縺溘＞縺ｲ]縺励※[荳�蠎ｦ/縺・■縺ｩ]縺�縺措隕ｳ蟇・縺九ｓ縺輔▽]縺吶ｋ縲阪→縺・≧[譁ｹ驥・縺ｻ縺・＠繧転繧端譏守､ｺ逧・繧√＞縺倥※縺江縺ｫ[謗｡逕ｨ/縺輔＞繧医≧]縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/23_competitive_sort_and_search.n.md -n 2` -> pass
  - `node nodesrc/tests.js -i stdlib/tests/vec.n.md -i stdlib/tests/list.n.md -i stdlib/tests/fs.n.md -i tutorials/getting_started/23_competitive_sort_and_search.n.md -i stdlib/kp/kpsearch.nepl --no-stdlib --no-tree -o /tmp/tests-stdlib-vec-list-fs-sort.json -j 4`
    - [邨先棡/縺代▲縺犠: `8/8 pass`
# 2026-03-10 io/streamio common read write facade

- `todo.md` 縺ｮ `stdio, io` 謖・､ｺ縺ｨ `doc/stdlib_breaking_reboot.md` 繧堤ｪ√″蜷医ｏ縺帙�∫樟陦・`std/streamio` / `std/io` 縺ｮ蜈ｬ髢矩擇縺・reboot 縺ｮ bare 蜷肴婿驥昴↓縺ｾ縺�螻翫＞縺ｦ縺・↑縺・％縺ｨ繧堤｢ｺ隱阪＠縺溘�・
- `alloc/io.nepl` 縺ｯ target 髱樔ｾ晏ｭ倥・ trait / `ByteBuf` helper 縺�縺代ｒ諡・ｽ薙☆繧句悄蜿ｰ縺ｨ縺励※謐ｮ縺医�√◎縺薙〒縺ｮ `ByteReader` / `TextReader` / `ByteWriter` / `TextWriter` / `Flush` / `Close` 繧・`std` facade 蛛ｴ縺九ｉ蜀榊茜逕ｨ縺吶ｋ譁ｹ驥昴↓縺励◆縲・
- `std/streamio.nepl` 縺ｫ縺ｯ `read` / `write` / `writeln` / `flush` / `close` 縺ｮ bare facade 繧堤ｽｮ縺阪�～stdin` / `stdout` / in-memory text / in-memory bytes 繧貞酔縺倩ｪ槫ｽ吶〒謇ｱ縺医ｋ繧医≧縺ｫ縺励◆縲・
- `std/io.nepl` 縺ｨ `std/iotarget.nepl` 繧定ｿｽ蜉�縺励�～IoReadTarget` / `IoWriteTarget` enum 繧帝�壹§縺ｦ `read target` / `write target data` / `data |> write target` 繧呈嶌縺代ｋ category facade 繧堤畑諢上＠縺溘�・
- `tests/stdlib/streamio.n.md` 縺ｨ `tests/stdlib/io.n.md` 縺ｯ縲∵眠 API 繧堤峩謗･菴ｿ縺・focused case 縺ｫ譖ｴ譁ｰ縺励◆縲・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`std/streamio` caller 縺�縺代ｒ譁ｰ縺励＞蜈ｱ騾壼錐縺ｸ鄂ｮ謠・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - [蛻ｩ逕ｨ蛛ｴ/繧翫ｈ縺・′繧従繝輔ぃ繧､繝ｫ縺�縺代〒縲～std/streamio` 縺ｮ old read/write API [蜻ｼ/繧・縺ｳ[蜃ｺ/縺�]縺励ｒ reboot [譁ｹ驥・縺ｻ縺・＠繧転縺ｩ縺翫ｊ `read` / `write` / `flush` / `stream io_*` 縺ｸ[蟇・繧・縺帙ｋ縲・
  - [謖・､ｺ/縺励§]縺ｩ縺翫ｊ `stdlib/std/streamio.nepl`, `stdlib/std/io.nepl`, `stdlib/alloc/io.nepl`, `stdlib/std/iotarget.nepl` 縺ｫ縺ｯ[隗ｦ/縺ｵ]繧後★縲～kp` wrapper / tests [蛛ｴ/縺後ｏ]縺�縺代ｒ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺吶ｋ縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/kp/kpread.nepl`
    - `stream_scanner_read_token` / `_i32` / `_i64` / `_f64` / `_f32` 繧・`read scanner_as_stream sc` 縺ｸ[鄂ｮ謠・縺｡縺九ｓ]縺励◆縲・
    - `u64` [隱ｭ/繧・縺ｿ縺�縺代・ current 縺ｮ common `read` overload 縺ｧ縺ｯ[隨ｦ蜿ｷ/縺ｵ縺斐≧]縺､縺・`i64` 縺ｨ[諢丞袖/縺・∩]縺啓荳�閾ｴ/縺・▲縺｡]縺励↑縺・◆繧√�～stream_scanner_read_u64` 繧端邯ｭ謖・縺・§]縺励◆縲・
  - `stdlib/kp/kpwrite.nepl`
    - `stream_writer_flush` / `_writeln` / `_write_str` / `_write_i32` / `_write_i64` / `_write_f64` / `_write_f32` 繧・`flush` / `write` / `write "\n"` 縺ｸ[鄂ｮ謠・縺｡縺九ｓ]縺励◆縲・
    - `writer_write_space` / `writer_write_*_ln` 縺ｮ[蜀・Κ/縺ｪ縺・・][螳溯｣・縺倥▲縺昴≧]縺ｨ doc comment 繧ゅ�～write inner " "` 縺ｨ `write inner v` + `write inner "\n"` 縺ｮ current [豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医◆縲・
    - `u64` / fixed precision 縺ｯ current common `write` overload 縺�縺代〒縺ｯ[諢丞袖隲・縺・∩繧阪ｓ]繧端菫・縺溘ｂ]縺ｦ縺ｪ縺・◆繧√�｛ld helper [蜻ｼ/繧・縺ｳ[蜃ｺ/縺�]縺励ｒ[邯ｭ謖・縺・§]縺励◆縲・
  - `tests/stdlib/streamio.n.md`
    - `stream_writer_write_*` / `stream_writer_writeln` / `stream_writer_write_space` / `stdout_stream` [逶ｴ蜻ｼ/縺倥°繧・縺ｳ繧偵�～write` / `flush` / `stream io_stdout` 縺ｸ[鄂ｮ謠・縺｡縺九ｓ]縺励◆縲・
    - scanner case 繧・`read sc` 縺ｸ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励◆縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - old caller 縺ｯ `stream_scanner_read_*` / `stream_writer_write_*` 縺ｮ[髟ｷ/縺ｪ縺珪縺Ъ蜷榊燕/縺ｪ縺ｾ縺・縺ｸ[逶ｴ謗･/縺｡繧・￥縺帙▽][萓晏ｭ・縺・◇繧転縺励※縺翫ｊ縲〉eboot 縺ｮ縲啓蛻ｩ逕ｨ閠・髄/繧翫ｈ縺・＠繧・・]縺措蜈･蜿｣/縺・ｊ縺舌■]縺ｯ facade 縺ｮ[蜈ｱ騾壼錐/縺阪ｇ縺・▽縺・ａ縺Ь縺ｸ[荳�譛ｬ蛹・縺・▲縺ｽ繧薙°]縺吶ｋ縲阪→縺・≧[險ｭ險・縺帙▲縺代＞]縺ｨ[鬟・縺従縺Ъ驕・縺｡縺珪縺｣縺ｦ縺・◆縲・
  - 縺ｨ縺上↓ `kp` wrapper 縺ｯ縲形std/streamio` 繧端阮・縺・☆]縺充蛹・縺､縺､]繧�縲梗蠖ｹ蜑ｲ/繧・￥繧上ｊ]縺ｪ縺ｮ縺ｫ縲ー蜀・Κ/縺ｪ縺・・]縺ｧ old helper [蜷・繧√＞]縺ｸ[蝗ｺ螳・縺薙※縺Ь縺輔ｌ縺ｦ縺翫ｊ縲～std` facade 縺ｮ[謾ｹ蜷・縺九＞繧√＞]繧端蛻ｩ逕ｨ蛛ｴ/繧翫ｈ縺・′繧従縺ｸ[豕｢蜿・縺ｯ縺阪ｅ縺・縺輔○繧擬讒矩��/縺薙≧縺槭≧]縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build`
    - [邨先棡/縺代▲縺犠: success
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 3`
    - [邨先棡/縺代▲縺犠: fail
    - [蜴溷屏/縺偵ｓ縺・ｓ]: caller [蛛ｴ/縺後ｏ]縺ｧ縺ｯ縺ｪ縺・`stdlib/std/streamio.nepl` [譛ｬ菴・縺ｻ繧薙◆縺Ь縺ｮ `read` / `write` overload [螳夂ｾｩ/縺ｦ縺・℃]縺後�√☆縺ｧ縺ｫ[蜑企勁/縺輔￥縺倥ｇ]繝ｻ[謾ｹ蜷・縺九＞繧√＞]縺輔ｌ縺・old helper [蜷・繧√＞] (`stream_scanner_read_token`, `stream_writer_write_str` 縺ｪ縺ｩ) 繧偵∪縺�[蜿ら・/縺輔ｓ縺励ｇ縺・縺励※縺翫ｊ縲〕ibrary compile error 縺ｧ[蛛懈ｭ｢/縺ｦ縺・＠]縺励◆縲・
  - `node nodesrc/run_doctest.js -i tests/stdlib/io.n.md -n 1`
    - [邨先棡/縺代▲縺犠: fail
    - [蜴溷屏/縺偵ｓ縺・ｓ]: [荳願ｨ・縺倥ｇ縺・″]縺ｨ[蜷・縺翫↑]縺・library [譛ｬ菴・縺ｻ繧薙◆縺Ь compile error 縺ｮ[蠖ｱ髻ｿ/縺医＞縺阪ｇ縺・縺ｧ縲～std/io` 邨檎罰 case 繧・騾夐℃/縺､縺・°]縺励↑縺・�・
- [迥ｶ豕・縺倥ｇ縺・″繧・≧]:
  - caller [蛛ｴ/縺後ｏ]縺ｧ[鄂ｮ謠・縺｡縺九ｓ]縺ｧ縺阪ｋ old read/write call site 縺ｯ縲～u64` / fixed precision 縺ｮ繧医≧縺ｪ current common overload [譛ｪ謠蝉ｾ・縺ｿ縺ｦ縺・″繧・≧]繧ｱ繝ｼ繧ｹ繧端髯､/縺ｮ縺枉縺・※[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
  - [莉雁屓/縺薙ｓ縺九＞]縺ｮ[謖・､ｺ遽・峇螟・縺励§縺ｯ繧薙＞縺後＞]縺ｧ縺ゅｋ `stdlib/std/streamio.nepl` [譛ｬ菴・縺ｻ繧薙◆縺Ь縺後�…urrent facade [蜷・繧√＞]縺ｸ縺ｮ[蜀・Κ/縺ｪ縺・・][霑ｽ蠕・縺､縺・§繧・≧]繧偵∪縺�[邨・縺馨縺医※縺・↑縺・◆繧√�√％縺薙〒縺ｯ library [蛛ｴ/縺後ｏ]繧端隗ｦ/縺輔ｏ]繧峨★縺ｫ[蛻・縺江繧骸蛻・繧従縺代□縺措谿・縺ｮ縺転縺励◆縲・

# 2026-03-10 菴懈･ｭ繝｡繝｢ (`io` / `streamio` 縺ｮ bare `read` / `stream` 繧・generic trait 蛹・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `reboot` 縺ｮ縲恵are 蜷・`read` / `write` / `writeln` / `flush` / `close` 縺ｫ邨ｱ荳�縺励�∝梛蟾ｮ縺ｯ trait / overload 縺ｧ陦ｨ縺吶�阪→縺・≧[譁ｹ驥・縺ｻ縺・＠繧転縺ｫ[豐ｿ/縺拆縺｣縺ｦ縲～std/io` / `std/streamio` 縺ｮ I/O facade 繧・current compiler 縺ｧ[螳牙ｮ・縺ゅｓ縺ｦ縺Ь縺励※[隗｣豎ｺ/縺九＞縺代▽]縺ｧ縺阪ｋ[蠖｢/縺九◆縺｡]縺ｫ[蝗ｺ螳・縺薙※縺Ь縺吶ｋ縲・
  - [蛻ｩ逕ｨ閠・繧翫ｈ縺・＠繧ゾ縺・`read sc` / `read io_stdin` / `stream io_stdout` 繧偵◎縺ｮ縺ｾ縺ｾ[譖ｸ/縺犠縺代ｋ繧医≧縺ｫ縺励▽縺､縲∬ｿ斐ｊ蛟､蝙九□縺代↓[萓晏ｭ・縺・◇繧転縺吶ｋ old overload 繧端謗帝勁/縺ｯ縺・§繧Ⅹ縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `std/streamio` 縺ｮ `read` / `stream` 縺ｨ `std/io` 縺ｮ `read` 縺ｯ縲～(StreamScanner)->i32` 縺ｨ `(StreamScanner)->f64` 縺ｮ繧医≧縺ｫ縲悟ｼ墓焚縺ｯ蜷後§縺ｧ霑斐ｊ蛟､縺�縺代′驕輔≧ overload縲阪↓[萓晏ｭ・縺・◇繧転縺励※縺・◆縲・
  - current compiler 縺ｯ縺昴・[蠖｢/縺九◆縺｡]繧・`let x <i32> read sc;` 縺ｮ繧医≧縺ｪ[譁・ц/縺ｶ繧薙∩繧・￥]縺�縺代〒蟶ｸ縺ｫ[隗｣豎ｺ/縺九＞縺代▽]縺ｧ縺阪★縲～ambiguous overload` 繧端襍ｷ/縺馨縺薙＠縺ｦ縺・◆縲・
  - `std/io` [蜀・Κ/縺ｪ縺・・]縺ｧ繧・`stream io_stdin` / `stream io_stdout` 縺ｮ[謌ｻ/繧ゅ←]繧骸蝙・縺後◆]縺啓譖匁乂/縺ゅ＞縺ｾ縺Ь縺ｫ縺ｪ繧翫�～match` [蜈ｨ菴・縺懊ｓ縺溘＞]縺・unit 縺ｫ[蟠ｩ/縺上★]繧後※ `read` facade [譛ｬ菴・縺ｻ繧薙◆縺Ь縺ｾ縺ｧ[騾｣骼・繧後ｓ縺評[謨・囿/縺薙＠繧・≧]縺励※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/std/streamio.nepl`
    - `StreamFromReadTarget` / `StreamFromWriteTarget` 繧端霑ｽ蜉�/縺､縺・°]縺励�～stream` 繧偵�瑚ｿ斐ｊ蛟､蝙・generic + trait dispatch縲阪〒[螳溯｣・縺倥▲縺昴≧]縺励◆縲・
    - `ScannerReadable` 繧端霑ｽ蜉�/縺､縺・°]縺励�～read sc` 繧・`str` / `i32` / `i64` / `f32` / `f64` 縺ｮ bare 蜷・generic 縺ｸ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励◆縲・
    - `StreamReadableResult` 繧端霑ｽ蜉�/縺､縺・°]縺励�～StdinStream` / `TextInputStream` / `ByteInputStream` 縺九ｉ縺ｮ `read` 繧りｿ斐ｊ蛟､蝙・generic 縺ｸ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励◆縲・
  - `stdlib/std/io.nepl`
    - `TargetReadable` 繧端霑ｽ蜉�/縺､縺・°]縺励�～read target` 繧・`ByteBuf` / `str` 縺ｮ generic facade 縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - [蜀・Κ/縺ｪ縺・・]縺ｧ縺ｯ `std/streamio` 縺ｮ generic `stream` 繧端蝙区ｳｨ驥・縺九◆縺｡繧・≧縺励ｃ縺従縺､縺・local binding 縺ｧ[蜿・縺・縺代�√◎縺ｮ[蠕・縺ゅ→]縺ｯ `alloc/io` helper (`io_read_all_bytes` / `io_read_all_text` / `io_write_bytes` / `io_write_str` / `io_flush` / `io_close`) 縺ｸ[蟋碑ｭｲ/縺・§繧・≧]縺吶ｋ[蠖｢/縺九◆縺｡]縺ｫ[謨ｴ逅・縺帙＞繧馨縺励◆縲・
  - `stdlib/kp/kpwrite.nepl`
    - `stream_writer_new` / `stream_writer_free` 縺ｮ old 蜿ら・繧・`writer` / `free` 縺ｸ[鄂ｮ謠・縺｡縺九ｓ]縺励�～stream_writer_flush` 繧・`flush` 縺ｸ[鄂ｮ謠・縺｡縺九ｓ]縺励◆縲・
  - `tests/stdlib/streamio.n.md`
    - duplicate 縺励※縺・◆ `stdout_binary_writer_pipe_data_to_target` case 繧・1 [莉ｶ/縺代ｓ][蜑企勁/縺輔￥縺倥ｇ]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - [蜷榊燕/縺ｪ縺ｾ縺・繧・bare 蛹悶☆繧九□縺代〒縺ｯ current compiler 縺ｮ overload 隗｣豎ｺ縺ｨ[遏帷崟/繧�縺倥ｅ繧転縺吶ｋ縺溘ａ縲～cast` / `deserialize` 縺ｨ[蜷・縺翫↑]縺倥￥縲瑚ｿ斐ｊ蛟､蝙・generic 繧・trait 縺ｧ[豎ｺ/縺江繧√ｋ縲梗蠖｢/縺九◆縺｡]縺ｸ[蟇・繧・縺帙◆縲・
  - 縺薙ｌ縺ｫ繧医ｊ `read` / `stream` 縺ｯ bare 蜷阪ｒ[邯ｭ謖・縺・§]縺励▽縺､縲《uffix 繧・compatibility alias 繧端蠅・縺ｵ]繧・＆縺壹↓[驕狗畑/縺・ｓ繧医≧]縺ｧ縺阪ｋ縲・
  - `std/io` [蜀・Κ/縺ｪ縺・・]縺ｮ stream [謫堺ｽ・縺昴≧縺評縺ｯ縲～std/streamio` 縺ｮ facade 蜷阪∈[萓晏ｭ・縺・◇繧転縺励☆縺弱ｋ縺ｨ[蜀榊ｸｰ逧・縺輔＞縺阪※縺江縺ｫ overload 繧端隍・尅蛹・縺ｵ縺上＊縺､縺犠縺吶ｋ縺溘ａ縲∝・騾・trait helper 縺ｸ[荳�谿ｵ/縺・■縺�繧転[關ｽ/縺馨縺ｨ縺励※[謨ｴ逅・縺帙＞繧馨縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/std/io.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/io.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/io.n.md -n 4`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 9`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 10`
    - [邨先棡/縺代▲縺犠: pass
  - `NO_COLOR=false trunk build`
    - [邨先棡/縺代▲縺犠: success

# 2026-03-11 菴懈･ｭ繝｡繝｢ (`streamio` / `io` 縺ｮ open/close 邨ｱ荳�縺ｨ scanner 謇�譛画ｨｩ繝｢繝・Ν縺ｮ蝗ｺ螳・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `reboot` 縺ｮ縲啓蝙・縺九◆]縺ｧ[蛹ｺ蛻･/縺上∋縺､]縺励�・未謨ｰ蜷阪〒縺ｯ[蛹ｺ蛻･/縺上∋縺､]縺励↑縺・�阪�啓蠕梧婿莠呈鋤/縺薙≧縺ｻ縺・＃縺九ｓ]縺ｯ[谿・縺ｮ縺転縺輔↑縺・�阪→縺・≧[譁ｹ驥・縺ｻ縺・＠繧転縺ｫ[蠕・縺励◆縺珪縺・�～std/streamio` / `std/io` 縺ｮ[蜈ｬ髢矩擇/縺薙≧縺九＞繧√ｓ]繧・`open` / `read` / `write` / `writeln` / `flush` / `close` 縺ｸ[邨ｱ荳�/縺ｨ縺・＞縺､]縺吶ｋ縲・
  - `ReadStream` / `WriteStream` 繧・enum target 縺ｨ縺励※[蝗ｺ螳・縺薙※縺Ь縺励�《tdin / stdout / in-memory text / bytes / fs path 繧端蜷・縺翫↑]縺麓隱槫ｽ・縺斐＞]縺ｧ[謇ｱ/縺ゅ▽縺犠縺医ｋ繧医≧縺ｫ縺吶ｋ縲・
  - [隍・焚/縺ｵ縺上☆縺・ stream 繧端蜷梧凾/縺ｩ縺・§]縺ｫ[邯ｭ謖・縺・§]縺ｧ縺阪ｋ繧医≧縲《canner / writer 縺ｮ[謇�譛画ｨｩ/縺励ｇ繧・≧縺代ｓ][隕丞援/縺阪◎縺従繧・current move model 縺ｨ[遏帷崟/繧�縺倥ｅ繧転縺励↑縺Ъ蠖｢/縺九◆縺｡]縺ｸ[蝗ｺ螳・縺薙※縺Ь縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `std/streamio` 縺ｮ `open(ReadStream)` / `open(WriteStream)` 縺ｯ high-level `StreamScanner` / `StreamWriter` 繧端霑・縺九∴]縺兌譁ｹ蜷・縺ｻ縺・％縺・縺ｸ[蟇・繧・縺｣縺ｦ縺・◆縺ｮ縺ｫ縲～std/io` 縺ｨ荳�驛ｨ test / tutorial 縺ｯ縺ｾ縺� `open -> StdinStream` / `StdoutStream` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｧ[谿・縺ｮ縺転縺｣縺ｦ縺翫ｊ縲∝・髢・API 縺ｨ caller 縺啓鬟・縺従縺Ъ驕・縺｡縺珪縺｣縺ｦ縺・◆縲・
  - `StreamScanner` 繧・non-copy resource 縺ｫ縺励◆縺ｾ縺ｾ `read sc` 縺ｮ bare API 縺ｸ[蟇・繧・縺帙◆縺溘ａ縲～read sc` 繧端隍・焚蝗・縺ｵ縺上☆縺・°縺Ь[譖ｸ/縺犠縺・current tutorial / kp case 縺・`D3053 use of moved value` 縺ｧ[螢・縺薙ｏ]繧後※縺・◆縲・
  - `close(StreamScanner)` [蜀・Κ/縺ｪ縺・・]縺ｫ繧・old helper `io_bytebuf_new` [蜿ら・/縺輔ｓ縺励ｇ縺・縺啓谿・縺ｮ縺転縺｣縺ｦ縺翫ｊ縲〕ibrary compile error 繧端隱倡匱/繧・≧縺ｯ縺､]縺励※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/std/iotarget.nepl`
    - `ReadStream` / `WriteStream` 縺ｮ enum target 繧・current public API 縺ｨ縺励※[蝗ｺ螳・縺薙※縺Ь縺励◆縲・
    - `WriteStream::Stdio` 縺ｯ payload 縺ｪ縺励�～ReadStream` 縺ｯ `Stdio` / `Fs <str>` / `Text <str>` / `Bytes <ByteBuf>` 繧端謖・繧・縺､[蠖｢/縺九◆縺｡]縺ｫ[謨ｴ逅・縺帙＞繧馨縺励◆縲・
  - `stdlib/std/streamio.nepl`
    - `open(ReadStream) -> Result<StreamScanner,str>` 縺ｨ `open(WriteStream) -> Result<StreamWriter,str>` 繧端蜈ｬ髢・縺薙≧縺九＞][蜈･蜿｣/縺・ｊ縺舌■]縺ｨ縺励※[蝗ｺ螳・縺薙※縺Ь縺励◆縲・
    - `StreamScanner` 縺ｫ `Copy` / `Clone` 繧端蠕ｩ豢ｻ/縺ｵ縺｣縺九▽]縺輔○縲…opy / clone 縺ｯ cursor / buffer 繧端蜈ｱ譛・縺阪ｇ縺・ｆ縺・縺吶ｋ alias 縺ｧ縺ゅｋ縺薙→繧・doc comment 縺ｫ[譏手ｨ・繧√＞縺江縺励◆縲・
    - `close(StreamScanner)` 縺ｮ old helper [蜿ら・/縺輔ｓ縺励ｇ縺・繧・`ByteBuf mem_ptr_wrap buf_addr len` 縺ｫ[鄂ｮ謠・縺｡縺九ｓ]縺励◆縲・
    - file header 縺ｨ `StreamScanner` / `StreamWriter` comment 繧・new policy / format 縺ｸ[蟇・繧・縺帙�ー隍・焚/縺ｵ縺上☆縺・ stream 蜷梧凾菫晄戟縺ｮ[諤ｧ雉ｪ/縺帙＞縺励▽]繧・霑ｽ險・縺､縺・″]縺励◆縲・
    - `stream_scanners_can_coexist` doctest 繧端霑ｽ蜉�/縺､縺・°]縺励�ー蛻･縲・縺ｹ縺､縺ｹ縺､]縺ｫ `open` 縺励◆ scanner 縺啓迢ｬ遶・縺ｩ縺上ｊ縺､]縺励※[隱ｭ/繧・繧√ｋ縺薙→繧端蝗ｺ螳・縺薙※縺Ь縺励◆縲・
  - `stdlib/std/io.nepl`
    - category facade [蜀・Κ/縺ｪ縺・・]繧・`open(ReadStream/WriteStream)` [萓晏ｭ・縺・◇繧転縺九ｉ[蛻・縺江繧骸髮｢/縺ｯ縺ｪ]縺励�～StdinStream ()` / `StdoutStream ()` 縺ｮ low-level handle 繧端蜀・Κ蛻ｩ逕ｨ/縺ｪ縺・・繧翫ｈ縺・縺吶ｋ[蠖｢/縺九◆縺｡]縺ｸ[謨ｴ逅・縺帙＞繧馨縺励◆縲・
    - 縺薙ｌ縺ｫ繧医ｊ `read ReadStream::Stdio` / `write WriteStream::Stdio ...` 縺ｮ facade 縺ｨ `streamio` 縺ｮ resource [逕滓・/縺帙＞縺帙＞]縺啓陦晉ｪ・縺励ｇ縺・→縺､]縺励↑縺・ｈ縺・↓縺励◆縲・
  - `tests/stdlib/io.n.md`, `tests/stdlib/streamio.n.md`, `tests/stdlib/kp.n.md`, `tests/stdlib/kp_i64.n.md`, `tests/stdlib/stdin.n.md`
    - old low-level `open -> StdinStream/StdoutStream` [蜑肴署/縺懊ｓ縺ｦ縺Ь繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励�…urrent public API 縺ｸ[霑ｽ蠕・縺､縺・§繧・≧]縺励◆縲・
    - `unwrap_ok open WriteStream::Stdio` 縺九ｉ `|> write` / `|> writeln` / `|> flush` / `|> close` 縺ｮ multiline pipe [豬∝о/繧翫ｅ縺・℃]縺ｸ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励◆縲・
    - scanner 繧端菴ｿ/縺､縺犠縺Ъ蛻・縺江縺｣縺・case 縺ｧ縺ｯ `close sc` 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
  - `tutorials/getting_started/22_competitive_io_and_arith.n.md`, `24_competitive_dp_basics.n.md`, `25_competitive_prefixsum_twopointers.n.md`, `27_competitive_algorithms_catalog.n.md`, `stdlib/kp/kpgraph.nepl`
    - `kpread` / `kpwrite` [蜑肴署/縺懊ｓ縺ｦ縺Ь繧・old writer/scanner helper [蜷・繧√＞]繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励�…urrent `std/streamio` [豬∝о/繧翫ｅ縺・℃]縺ｸ[譖ｸ/縺犠縺梗謠・縺犠縺医◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - scanner 繧・non-copy resource 縺ｮ縺ｾ縺ｾ縺ｫ縺吶ｋ縺ｨ `read sc` 繧端隍・焚蝗・縺ｵ縺上☆縺・°縺Ь[譖ｸ/縺犠縺剰・辟ｶ縺ｪ public API 縺ｨ[荳｡遶・繧翫ｇ縺・ｊ縺､]縺励↑縺・◆繧√�”andle [閾ｪ菴・縺倥◆縺Ь縺ｯ copyable alias縲｜uffer [隗｣謾ｾ/縺九＞縺ｻ縺・縺�縺・`close` 縺ｸ[髮・ｴ・縺励ｅ縺・ｄ縺従縺吶ｋ[蠖｢/縺九◆縺｡]縺ｫ[謌ｻ/繧ゅ←]縺励◆縲・
  - `std/io` 縺ｯ category facade縲～std/streamio` 縺ｯ resource facade 縺ｨ[雋ｬ蜍・縺帙″繧�]繧端蛻・屬/縺ｶ繧薙ｊ]縺励�∝酔縺・`open` [蜷・繧√＞]繧端菴ｿ/縺､縺犠縺｣縺ｦ繧・霑・縺九∴]繧骸蛟､/縺ゅ◆縺Ь縺ｨ target [蝙・縺九◆]縺ｧ[髱咏噪/縺帙＞縺ｦ縺江縺ｫ[蛻・繧従縺九ｌ繧擬讒矩��/縺薙≧縺槭≧]縺ｸ[蟇・繧・縺帙◆縲・
  - [蛻･縲・縺ｹ縺､縺ｹ縺､]縺ｫ `open` 縺励◆ scanner / writer 縺啓蜷梧凾/縺ｩ縺・§]縺ｫ[蟄伜惠/縺昴ｓ縺悶＞]縺ｧ縺阪ｋ縺薙→縺ｯ public API 縺ｮ[驥崎ｦ・縺倥ｅ縺・ｈ縺・縺ｪ[諤ｧ雉ｪ/縺帙＞縺励▽]縺ｪ縺ｮ縺ｧ縲‥octest 縺ｧ[蝗ｺ螳・縺薙※縺Ь縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 11`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 13`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/io.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/io.n.md -n 4`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 3`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/kp_i64.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/22_competitive_io_and_arith.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/27_competitive_algorithms_catalog.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass

# 2026-03-11 菴懈･ｭ繝｡繝｢ (`Drop` capability 縺ｮ source 螳｣險�縺ｨ auto drop 謖ｿ蜈･縺ｮ compiler 蝗ｺ螳・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - [謇�譛画ｨｩ/縺励ｇ繧・≧縺代ｓ]縺啓邨・縺馨繧上▲縺歇蛟､/縺ゅ◆縺Ь繧・compiler 縺啓閾ｪ蜍・縺倥←縺・縺ｧ[蠕悟ｧ区忰/縺ゅ→縺励∪縺､]縺ｧ縺阪ｋ[蝨溷床/縺ｩ縺�縺Ь繧偵�”ardcode 縺ｧ縺ｯ縺ｪ縺・`.nepl` [蛛ｴ/縺後ｏ]縺ｮ trait [螳｣險�/縺帙ｓ縺偵ｓ]縺ｨ縺励※[蝗ｺ螳・縺薙※縺Ь縺吶ｋ縲・
  - `reboot` / `memory_safety_compiler_design` 縺ｮ[譁ｹ驥・縺ｻ縺・＠繧転縺ｫ[蠕・縺励◆縺珪縺・�［ove check 縺ｨ[遏帷崟/繧�縺倥ｅ繧転縺励↑縺・auto drop 謖ｿ蜈･縺ｨ[隧ｳ邏ｰ/縺励ｇ縺・＆縺Ь test 繧端謨ｴ蛯・縺帙＞縺ｳ]縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - [譌｢蟄・縺阪◎繧転縺ｮ `drop_insertion` 縺ｯ lexical 縺ｫ `HirExprKind::Drop { name }` 繧端蟾ｮ/縺評縺吶□縺代〒縲…odegen 蛛ｴ縺ｧ縺ｯ no-op 縺ｮ縺ｾ縺ｾ縺�縺｣縺溘�ゅ◎縺ｮ縺溘ａ source [荳・縺倥ｇ縺・縺ｧ `Drop` 繧端螳｣險�/縺帙ｓ縺偵ｓ]縺励※繧・destructor 螳溯｡後↓[邨・繧�縺兢縺ｳ[莉・縺､]縺九↑縺九▲縺溘�・
  - [蠖灘・/縺ｨ縺・＠繧Ⅹ縺ｯ destructor 繧・`Self` [蛟､貂｡/縺ゅ◆縺・ｏ縺歉縺励↓縺励※縺・◆縺後�〉aw wasm ABI 縺ｧ縺ｯ[隍・粋蛟､/縺ｵ縺上＃縺・■]繧偵◎縺ｮ縺ｾ縺ｾ[貂｡/繧上◆]縺兌邨瑚ｷｯ/縺代＞繧江縺ｧ `unsupported function signature for wasm` 縺啓逋ｺ逕・縺ｯ縺｣縺帙＞]縺励※縺・◆縲・
  - Rust test fixture 繧・old 蜑肴署繧端蠑・縺ｲ]縺阪★縺｣縺ｦ縺翫ｊ縲～#entry main` [谺�關ｽ/縺代▽繧峨￥]繧・branch [譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｮ荳崎ｦ・`;` 縺ｧ validator failure / loader failure 繧端隱倡匱/繧・≧縺ｯ縺､]縺励※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nepl-core/src/ast.rs`, `parser.rs`, `hir.rs`, `typecheck.rs`, `types.rs`
    - trait capability 縺ｫ `Drop` 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - typecheck 縺ｧ `#capability drop` trait 繧端讀懷・/縺代ｓ縺励ｅ縺､]縺励�‥rop impl target 繧・`TypeCtx` 縺ｸ[逋ｻ骭ｲ/縺ｨ縺・ｍ縺従縺ｧ縺阪ｋ繧医≧縺ｫ縺励◆縲・
    - `TypeCtx::has_drop` 繧端霑ｽ蜉�/縺､縺・°]縺励�》uple / named / struct / enum / apply 縺ｮ[蜀榊ｸｰ逧・縺輔＞縺阪※縺江[蛻､螳・縺ｯ繧薙※縺Ь繧端謖・繧・縺溘○縺溘�・
  - `stdlib/core/traits/drop.nepl`
    - new policy / format 縺ｧ file header 縺ｨ trait comment 繧端謨ｴ蛯・縺帙＞縺ｳ]縺励◆縲・
    - destructor [鄂ｲ蜷・縺励ｇ繧√＞]繧・`fn drop <(&Self)*>()> (self)` 縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励�〉aw wasm ABI 縺ｨ[謨ｴ蜷・縺帙＞縺斐≧]縺吶ｋ繧医≧縺ｫ縺励◆縲・
  - `nepl-core/src/passes/drop_insertion.rs`
    - auto drop 繧・`HirExprKind::Drop` 縺ｧ縺ｯ縺ｪ縺・trait call [謖ｿ蜈･/縺昴≧縺ｫ繧・≧]縺ｸ[菴・縺､縺従繧骸逶ｴ/縺ｪ縺馨縺励◆縲・
    - monomorphize [蜑・縺ｾ縺・縺ｫ `Drop::drop` 蜻ｼ縺ｳ[蜃ｺ/縺�]縺励ｒ[蜈･/縺Ь繧後ｋ[蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励�∵里蟄倥・ trait 隗｣豎ｺ繝ｻmonomorphize [邨瑚ｷｯ/縺代＞繧江縺ｫ[荵・縺ｮ]縺帙◆縲・
    - [螟画焚/縺ｸ繧薙☆縺・[迥ｶ諷・縺倥ｇ縺・◆縺Ь縺ｯ `Valid` / `Moved` / `PossiblyMoved` 繧端霑ｽ霍｡/縺､縺・○縺江縺励�｜ranch merge 縺ｯ[菫晏ｮ育噪/縺ｻ縺励ｅ縺ｦ縺江縺ｫ `PossiblyMoved` 縺ｸ[蛟・縺溘♀]縺兌蠖｢/縺九◆縺｡]縺ｫ縺励◆縲・
    - auto drop call 縺ｯ local [逡ｪ蝨ｰ/縺ｰ繧薙■]繧端貂｡/繧上◆]縺吶◆繧・`AddrOf(Var(name))` 繧端菴ｿ/縺､縺犠縺・蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `nepl-core/src/compiler.rs`
    - `insert_drops` 繧・monomorphize [蜑・縺ｾ縺・縺ｸ[遘ｻ蜍・縺・←縺・縺励�》rait call 縺ｨ縺励※[隗｣豎ｺ/縺九＞縺代▽]縺ｧ縺阪ｋ繧医≧縺ｫ縺励◆縲・
  - `nepl-core/tests/drop.rs`
    - scope end / nested scope LIFO / branch local / shadowing / conditional move / loader-visible stdlib 縺ｮ 7 [莉ｶ/縺代ｓ]繧端隧ｳ邏ｰ/縺励ｇ縺・＆縺Ь縺ｫ[蝗ｺ螳・縺薙※縺Ь縺励◆縲・
    - fixture 縺ｯ field [隱ｭ/繧・縺ｿ繧・zero-field struct 縺ｫ[鬆ｼ/縺溘ｈ]繧峨★縲‥istinct guard [蝙・縺後◆]縺ｨ `#entry main` [莉・縺､]縺・minimal program 縺ｸ[謨ｴ逅・縺帙＞繧馨縺励◆縲・
  - `tests/compiler/drop.n.md`
    - Rust integration test 縺ｨ[蜷檎ｳｻ邨ｱ/縺ｩ縺・￠縺・→縺・縺ｮ compiler doctest 繧・skip 縺九ｉ螳・testcase 縺ｸ[鄂ｮ謠・縺｡縺九ｓ]縺励�］odesrc [邨瑚ｷｯ/縺代＞繧江縺ｧ繧・`Drop` 繧端蜷ｫ/縺ｵ縺従繧�蜈･蜉帙′ compile / run 縺ｧ縺阪ｋ縺薙→繧端蝗ｺ螳・縺薙※縺Ь縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - auto drop 縺ｯ codegen special case 縺ｫ縺帙★縲》rait call [謖ｿ蜈･/縺昴≧縺ｫ繧・≧]縺ｨ縺励※ HIR [荳・縺倥ｇ縺・縺ｧ[陦ｨ迴ｾ/縺ｲ繧・≧縺偵ｓ]縺励◆縺ｻ縺・′縲《ource [螳｣險�/縺帙ｓ縺偵ｓ]縺輔ｌ縺・capability 縺ｨ compiler [螳溯｣・縺倥▲縺昴≧]縺啓荳�閾ｴ/縺・▲縺｡]縺吶ｋ縲・
  - destructor 繧・`&Self` 縺ｫ縺励◆縺ｮ縺ｯ temporary / stack slot 縺ｮ[逡ｪ蝨ｰ/縺ｰ繧薙■]繧端貂｡/繧上◆]縺帙ｋ繧医≧縺ｫ縺吶ｋ縺溘ａ縺ｧ縲ヽust 縺ｮ drop glue 縺ｫ繧・霑・縺｡縺犠縺Ъ譁ｹ蜷・縺ｻ縺・％縺・縺ｧ縺ゅｋ縲・
  - runtime [鬆・ｺ・縺倥ｅ繧薙§繧Ⅹ test 縺ｯ Rust integration test縲］odesrc 蛛ｴ縺ｯ compile / run regression 縺ｨ縺・≧[雋ｬ蜍吝・諡・縺帙″繧�縺ｶ繧薙◆繧転縺ｫ縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `cargo test -p nepl-core --test drop -- --nocapture`
    - [邨先棡/縺代▲縺犠: `7/7 pass`
  - `node nodesrc/tests.js -i tests/compiler/drop.n.md --no-stdlib --no-tree -o /tmp/tests-compiler-drop.json -j 4`
    - [邨先棡/縺代▲縺犠: `4/4 pass`
    - output JSON: `/tmp/tests-compiler-drop.json`
  - `NO_COLOR=false trunk build`
    - [邨先棡/縺代▲縺犠: success
  - `cargo build -p nepl-cli`
    - [邨先棡/縺代▲縺犠: success

# 2026-03-11 菴懈･ｭ繝｡繝｢ (`HashKey` 縺ｨ hash collection 縺ｮ reboot 蜿取據)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `HashMap` / `HashSet` 繧・reboot [譁ｹ驥・縺ｻ縺・＠繧転縺ｩ縺翫ｊ bare API + trait [蟋碑ｭｲ/縺・§繧・≧]縺ｸ[蟇・繧・縺帙�《pecialized key helper [蜷・繧√＞]縺ｫ[萓晏ｭ・縺・◇繧転縺励↑縺・collection 縺ｫ縺吶ｋ縲・
  - custom trait 縺ｮ `#capability copy` 縺・generic bound 縺ｨ concrete impl 縺ｮ[荳｡譁ｹ/繧翫ｇ縺・⊇縺・縺ｧ[蜉ｹ/縺江縺上ｈ縺・↓ compiler [蛛ｴ/縺後ｏ]繧端菫ｮ豁｣/縺励ｅ縺・○縺Ь縺励�～HashKey` 縺ｮ custom key 縺・move check 縺ｧ[螢・縺薙ｏ]繧後↑縺・ｈ縺・↓縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `HashMap` / `HashSet` 縺ｮ custom key failure 縺ｯ collection [螳溯｣・縺倥▲縺昴≧]縺ｧ縺ｯ縺ｪ縺・compiler [蛛ｴ/縺後ｏ]縺�縺｣縺溘�ＡTypeCtx::is_copy` 縺ｯ generic type var 縺ｮ trait bound capability 縺ｨ縲～Copy` 莉･螟悶・ copy-capability trait impl target 繧端隕・縺ｿ]縺ｦ縺・↑縺九▲縺溘�・
  - 縺昴・縺溘ａ `.K: HashKey` 縺ｧ繧・move check 縺ｯ `key` 繧・non-copy 縺ｨ[蛻､螳・縺ｯ繧薙※縺Ь縺励�｝robing 荳ｭ縺ｮ[蜀榊茜逕ｨ/縺輔＞繧翫ｈ縺・縺ｧ `D3053` 繧端蜃ｺ/縺�]縺励※縺・◆縲・
  - `Hash` / `hash32` test 繧・old star import [蜑肴署/縺懊ｓ縺ｦ縺Ь縺啓谿・縺ｮ縺転縺｣縺ｦ縺翫ｊ縲”elper shadowing 縺ｨ bare overload [譖匁乂蛹・縺ゅ＞縺ｾ縺・°]縺ｧ `D3005` 繧端襍ｷ/縺馨縺薙＠縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nepl-core/src/types.rs`, `nepl-core/src/typecheck.rs`
    - type var 縺ｫ `copy_cap` / `clone_cap` / `drop_cap` 繧端菫晄戟/縺ｻ縺肋縺輔○縲》ype parameter bound 縺ｮ capability 繧・move check / drop 蛻､螳壹∈[莨晄眺/縺ｧ繧薙・]縺吶ｋ繧医≧縺ｫ縺励◆縲・
    - function instantiate 譎ゅ↓繧・fresh type var 縺ｸ capability flag 繧端蠑・縺ｲ]縺梗邯・縺､]縺舌ｈ縺・↓縺励◆縲・
    - compiler 縺・`Copy` trait 1 [蛟・縺転縺�縺代ｒ special case [謇ｱ/縺ゅ▽縺犠縺・＠縺ｦ縺・◆[邂・園/縺九＠繧Ⅹ繧端謾ｹ/縺ゅｉ縺歉繧√�～#capability copy` / `clone` / `drop` 繧端謖・繧・縺､ trait 繧・capability [蜊倅ｽ・縺溘ｓ縺Ь縺ｧ[隱崎ｭ・縺ｫ繧薙＠縺江縺吶ｋ繧医≧縺ｫ縺励◆縲・
  - `stdlib/core/traits/hash_key.nepl`
    - `HashMap` / `HashSet` [蜷・繧�]縺代↓ key capability `HashKey` 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `clone` / `eq` / `hash32` 繧・1 [蛟・縺転縺ｮ trait 縺ｸ[髮・ｴ・縺励ｅ縺・ｄ縺従縺励�｜uiltin key (`bool` / `i32` / `u8` / `i64` / `str`) 縺ｮ impl 繧端謨ｴ蛯・縺帙＞縺ｳ]縺励◆縲・
  - `stdlib/alloc/collections/hashmap.nepl`, `stdlib/alloc/collections/hashset.nepl`
    - `.K: HashKey` / `.T: HashKey` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｧ open addressing 螳溯｣・ｒ[謨ｴ逅・縺帙＞繧馨縺励◆縲・
    - internal helper [蜷・繧√＞]縺ｯ `hashmap_*` / `hashset_*` 縺ｸ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励�《tar import [陦晉ｪ・縺励ｇ縺・→縺､]繧端驕ｿ/縺評縺代◆縲・
  - `stdlib/alloc/hash/hash32.nepl`, `stdlib/core/traits/hash.nepl`
    - bare `hash32` overload 縺ｨ trait `Hash::hash32` 縺啓蜀榊ｸｰ/縺輔＞縺江繧Ъ譖匁乂蛹・縺ゅ＞縺ｾ縺・°]繧端襍ｷ/縺馨縺薙＆縺ｪ縺・ｈ縺・�｝rimitive hash [險育ｮ・縺代＞縺輔ｓ]繧端譏守､ｺ逧・繧√＞縺倥※縺江縺ｫ[螻暮幕/縺ｦ繧薙°縺Ь縺励◆縲・
  - `stdlib/tests/hashmap.n.md`, `stdlib/tests/hashmap_str.n.md`, `stdlib/tests/hashset.n.md`, `stdlib/tests/hashset_str.n.md`, `tests/stdlib/traits_hash.n.md`
    - current ownership model 縺ｫ[蜷・縺・繧上○縺ｦ fixture 繧端謨ｴ逅・縺帙＞繧馨縺励�…ustom `HashKey` key 繧端蜷ｫ/縺ｵ縺従繧� focused regression 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `traits_hash` 縺ｮ蜈磯�ｭ case 縺ｯ old bare import 豈碑ｼ・ｒ繧・ａ縲…urrent trait helper 縺ｮ deterministic / distinctness 繧端遒ｺ隱・縺九￥縺ｫ繧転縺吶ｋ[蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `tests/compiler/trait_capability_copy.n.md`
    - custom trait 縺ｮ `#capability copy` / `#capability clone` 縺・generic bound 縺ｫ[莨晄眺/縺ｧ繧薙・]縺励�～.T` 繧端隍・焚蝗・縺ｵ縺上☆縺・°縺Ь[菴ｿ/縺､縺犠縺｣縺ｦ繧・`D3053` 縺ｫ縺ｪ繧峨↑縺・％縺ｨ繧端蝗ｺ螳・縺薙※縺Ь縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - [迴ｾ迥ｶ/縺偵ｓ縺倥ｇ縺・縺ｮ險�隱樔ｻ墓ｧ倥〒縺ｯ multiple trait bound 縺啓譖ｸ/縺犠縺代↑縺・◆繧√�”ash collection 縺ｮ key [譚｡莉ｶ/縺倥ｇ縺・￠繧転縺ｯ `HashKey` 縺ｫ[髮・ｴ・縺励ｅ縺・ｄ縺従縺励◆縲ゅ％繧後・ `Eq + Hash + Clone/Copy` 縺ｮ collection [逕ｨ/繧医≧] capability 縺ｨ縺励※[謇ｱ/縺ゅ▽縺犠縺・�・
  - 縺溘□縺・compiler [蛛ｴ/縺後ｏ]縺ｯ `HashKey` 蟆ら畑 special case 縺ｫ縺帙★縲√�荊rait capability 繧・type system 縺啓荳�闊ｬ/縺・▲縺ｱ繧転縺ｫ[逅・ｧ｣/繧翫°縺Ь縺吶ｋ縲梗譁ｹ蜷・縺ｻ縺・％縺・縺ｸ[菫ｮ豁｣/縺励ｅ縺・○縺Ь縺励◆縲ゅ％繧後〒莉悶・ custom capability trait 縺ｫ繧・蜷・縺翫↑]縺倅ｿｮ豁｣縺啓蜉ｹ/縺江縺上�・
  - `btreemap.nepl` 縺ｮ蟾ｮ蛻・・縺薙・ batch 縺ｧ縺ｯ[隗ｦ/縺輔ｏ]縺｣縺ｦ縺翫ｉ縺壹�…ollection reboot 縺ｮ[谿倶ｻｶ/縺悶ｓ縺代ｓ]縺ｨ縺励※[蛻･/縺ｹ縺､]縺ｫ[邯夊｡・縺槭▲縺薙≧]縺吶ｋ縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `cargo test -p nepl-core --test drop -- --nocapture`
    - [邨先棡/縺代▲縺犠: `7/7 pass`
  - `NO_COLOR=false trunk build`
    - [邨先棡/縺代▲縺犠: success
  - `node nodesrc/tests.js -i tests/compiler/trait_capability_copy.n.md -i tests/stdlib/traits_hash.n.md -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/hashset_str.n.md --no-stdlib --no-tree -o /tmp/tests-hash-capability-focus.json -j 4`
    - [邨先棡/縺代▲縺犠: `8/8 pass`
    - output JSON: `/tmp/tests-hash-capability-focus.json`

# 2026-03-11 菴懈･ｭ繝｡繝｢ (`BTreeMap` / `BTreeSet` 縺ｮ reboot 霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `BTreeMap` / `BTreeSet` 繧・reboot [譁ｹ驥・縺ｻ縺・＠繧転縺ｩ縺翫ｊ bare API + trait [蟋碑ｭｲ/縺・§繧・≧]縺ｸ[謠・縺昴ｍ]縺医�｛ld `btreemap_*` / `btreeset_*` alias [蜑肴署/縺懊ｓ縺ｦ縺Ь繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺吶ｋ縲・
  - stdlib fixture 縺ｨ `pipe_collections` 繧・current ownership model / explicit report [豬∝о/繧翫ｅ縺・℃]縺ｸ[霑ｽ蠕・縺､縺・§繧・≧]縺輔○繧九�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `btreemap` / `btreeset` 縺ｯ collection reboot 縺ｮ[騾比ｸｭ/縺ｨ縺｡繧・≧]縺ｧ[豁｢/縺ｨ]縺ｾ縺｣縺ｦ縺翫ｊ縲～btreemap_*` / `btreeset_*` [蜻ｽ蜷・繧√＞繧√＞]縲～i32` 蝗ｺ螳・set縲｛ld comment format縲｛ld `ret: 1` fixture 縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
  - `btreemap` / `btreeset` 縺ｮ `insert` 縺ｯ capacity [蛻､螳・縺ｯ繧薙※縺Ь縺ｧ collection [譛ｬ菴・縺ｻ繧薙◆縺Ь繧端隍・焚蝗・縺ｵ縺上☆縺・°縺Ь[隱ｭ/繧・繧薙〒縺翫ｊ縲…urrent move model 縺ｧ縺ｯ `D3063` / `D3053` 縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
  - `BTreeSet<i32>` 縺ｮ bare `new<i32>` 縺ｯ `std/test` import [荳・縺犠縺ｧ overload [譖匁乂蛹・縺ゅ＞縺ｾ縺・°]縺励◆縲ゅ％繧後・ collection 蛛ｴ縺ｧ縺ｯ縺ｪ縺上�…urrent compiler 縺・no-arg generic constructor 繧・expected return type 縺�縺代〒縺ｯ[蜊∝・/縺倥ｅ縺・・繧転縺ｫ[邨・縺励⊂]繧後※縺・↑縺・％縺ｨ縺啓蜴溷屏/縺偵ｓ縺・ｓ]縺�縺｣縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/collections/btreemap.nepl`
    - bare `new` / `insert` / `get` / `contains` / `remove` / `len` / `clear` / `free` [讒区・/縺薙≧縺帙＞]繧端邯ｭ謖・縺・§]縺励▽縺､縲～insert` 縺ｮ capacity [蛻､螳・縺ｯ繧薙※縺Ь繧・`hdr0` / `len_init` / `cap_init` [蜈郁ｪｭ/縺輔″繧・縺ｿ縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励※ move error 繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励◆縲・
  - `stdlib/alloc/collections/btreeset.nepl`
    - file 蜈ｨ菴薙ｒ new policy / format 縺ｧ[譖ｸ/縺犠縺梗逶ｴ/縺ｪ縺馨縺励◆縲・
    - `struct BTreeSet<.T>` 縺ｨ `Ord` trait [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｮ generic set 縺ｫ[蜀崎ｨｭ險・縺輔＞縺帙▲縺代＞]縺励◆縲・
    - public API 繧・bare `new` / `insert` / `contains` / `remove` / `len` / `clear` / `free` 縺ｸ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励◆縲・
    - internal helper 縺ｯ `btreeset_*` 縺ｫ[髢・縺ｨ]縺倩ｾｼ繧√�｝ublic API 縺ｨ star import [陦晉ｪ・縺励ｇ縺・→縺､]縺励↑縺・ｈ縺・↓縺励◆縲・
    - `insert` 縺ｮ grow [蛻､螳・縺ｯ繧薙※縺Ь繧・`hdr0` / `len_init` / `cap_init` [蜈郁ｪｭ/縺輔″繧・縺ｿ縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励※ move error 繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励◆縲・
  - `stdlib/tests/btreemap.n.md`, `stdlib/tests/btreeset.n.md`
    - old alias API 縺ｨ `ret: 1` [蜑肴署/縺懊ｓ縺ｦ縺Ь繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励�～Vec<Result<(),str>>` + `checks_print_report` + `checks_exit_code` 縺ｮ explicit report [豬∝о/繧翫ｅ縺・℃]縺ｸ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励◆縲・
    - `btreeset` fixture 縺ｧ縺ｯ current compiler 蛻ｶ邏・・縺溘ａ `fn new_set ...: new<i32>` wrapper 繧端鄂ｮ/縺馨縺阪�｝ublic bare name 繧・expected type [莉・縺､]縺・helper [邨檎罰/縺代＞繧・縺ｧ[蜻ｼ/繧・縺ｶ[蠖｢/縺九◆縺｡]縺ｫ縺励◆縲・
  - `tests/stdlib/pipe_collections.n.md`
    - `btreemap` / `btreeset` 縺ｮ pipe section 繧・current bare API 縺ｸ[譖ｸ/縺犠縺梗謠・縺犠縺医◆縲・
    - [菴ｵ/縺ゅｏ]縺帙※ old `hashmap_*` / `hashset_*` alias section 繧・current bare API 縺ｸ[霑ｽ蠕・縺､縺・§繧・≧]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - public API 縺ｯ `btree*_*` alias 繧端谿・縺ｮ縺転縺輔★ bare name 繧端豁｣/縺帙＞]縺ｨ縺励◆縲Ｐld fixture 蛛ｴ縺�縺代ｒ譖ｸ縺肴鋤縺医※莠呈鋤螻､繧端菴・縺､縺従繧九％縺ｨ縺ｯ縺励※縺・↑縺・�・
  - `BTreeSet` 縺ｯ `HashSet` 縺ｨ[蜷梧ｧ・縺ｩ縺・ｈ縺・縺ｫ generic `.T: Ord` 縺ｸ[蟇・繧・縺帙�…ollection [蜷・繧√＞]縺ｧ縺ｪ縺・trait [蠅・阜/縺阪ｇ縺・°縺Ь縺啓諢丞袖隲・縺・∩繧阪ｓ]繧端豎ｺ/縺江繧√ｋ[讒矩��/縺薙≧縺槭≧]縺ｫ縺励◆縲・
  - `new<i32>` 縺ｮ wrapper 縺ｯ collection 險ｭ險医・[螯･蜊・縺�縺阪ｇ縺・縺ｧ縺ｯ縺ｪ縺・current compiler limitation 縺ｮ[蛻・縺江繧骸蛻・繧従縺代→縺励※[謇ｱ/縺ゅ▽縺犠縺・�Ｑublic API 閾ｪ菴薙・ bare `new` 縺ｮ縺ｾ縺ｾ[邯ｭ謖・縺・§]縺励�√％縺ｮ limitation 縺ｯ[蠕檎ｶ・縺薙≧縺槭￥]縺ｮ compiler overload [謾ｹ蝟・縺九＞縺懊ｓ]縺ｧ[隗｣豸・縺九＞縺励ｇ縺・縺吶∋縺阪ｂ縺ｮ縺ｨ縺励※[險倬鹸/縺阪ｍ縺従縺吶ｋ縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/tests.js -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i tests/stdlib/pipe_collections.n.md --no-stdlib --no-tree -o /tmp/tests-btree-focus.json -j 4`
    - [邨先棡/縺代▲縺犠: `14/14 pass`
    - output JSON: `/tmp/tests-btree-focus.json`

# 2026-03-11 菴懈･ｭ繝｡繝｢ (`alloc/hash` comment / fixture 縺ｮ reboot 霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `alloc/hash` [驟堺ｸ・縺ｯ縺・°]縺ｮ comment 縺ｨ fixture 繧・reboot [蠕・縺脳縺ｮ test [豬∝о/繧翫ｅ縺・℃]縺ｨ doc comment policy 縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - old `hash32_i32` / old `ret: 0` / old test output [蜑肴署/縺懊ｓ縺ｦ縺Ь繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励�…urrent bare API 縺ｨ explicit report [豬∝о/繧翫ｅ縺・℃]繧端蝗ｺ螳・縺薙※縺Ь縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `stdlib/tests/hash.n.md` 縺ｯ old success/failure [豬∝о/繧翫ｅ縺・℃]縺ｮ縺ｾ縺ｾ縺ｧ縲～checks_print_report` / `checks_exit_code` 縺ｫ繧医ｋ current safe test flow 縺ｨ[荳堺ｸ�閾ｴ/縺ｵ縺・▲縺｡]縺�縺｣縺溘�・
  - `alloc/hash/fnv1a32.nepl` / `alloc/hash/sha256.nepl` 縺ｮ comment 縺ｯ new policy / format 縺ｫ[豐ｿ/縺拆縺｣縺ｦ縺翫ｉ縺壹�ー迴ｾ迥ｶ/縺偵ｓ縺倥ｇ縺・縺ｮ scaffold [迥ｶ諷・縺倥ｇ縺・◆縺Ь繧Ъ豕ｨ諢冗せ/縺｡繧・≧縺・※繧転縺・file header 縺ｨ item comment 縺九ｉ[隱ｭ/繧・縺ｿ[蜿・縺ｨ]繧後↑縺九▲縺溘�・
  - `hash` fixture 縺ｯ old `hash32_i32` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺啓谿・縺ｮ縺転縺｣縺ｦ縺翫ｊ縲…urrent trait [蟋碑ｭｲ/縺・§繧・≧]縺ｮ隱ｬ譏弱→繧ｺ繝ｬ縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/hash/fnv1a32.nepl`
    - file header 縺ｨ `Fnv1a32` / `new_fnv1a32` / `fnv1a32_update` / `fnv1a32_finalize` 縺ｮ doc comment 繧・new policy / format 縺ｸ[譖ｸ/縺犠縺梗逶ｴ/縺ｪ縺馨縺励◆縲・
    - [證怜捷/縺ゅｓ縺斐≧]逕ｨ騾斐〒縺ｯ縺ｪ縺・％縺ｨ縲〕ightweight state 縺ｧ縺ゅｋ縺薙→縲～update` / `finalize` 縺ｮ O(1) 繧端譏手ｨ・繧√＞縺江縺励◆縲・
  - `stdlib/alloc/hash/sha256.nepl`
    - file header 縺ｨ `Sha256` / `new_sha256` / `sha256_update` / `sha256_finalize` 縺ｮ doc comment 繧・new policy / format 縺ｸ[譖ｸ/縺犠縺梗逶ｴ/縺ｪ縺馨縺励◆縲・
    - [迴ｾ迥ｶ/縺偵ｓ縺倥ｇ縺・縺ｧ縺ｯ SHA-256 digest 繧端險育ｮ・縺代＞縺輔ｓ]縺励※縺翫ｉ縺壹�｜uffering scaffold 縺ｧ縺ゅｋ縺薙→繧端譏手ｨ・繧√＞縺江縺励◆縲・
  - `stdlib/tests/hash.n.md`
    - `#entry main` + `Vec<Result<(),str>>` + `checks_print_report` + `checks_exit_code` 縺ｮ explicit report [豬∝о/繧翫ｅ縺・℃]縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
    - old `hash32_i32` 繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励�》rait [邨檎罰/縺代＞繧・縺ｮ `hash32_by_trait` 縺ｧ determinism / distinctness 繧端遒ｺ隱・縺九￥縺ｫ繧転縺吶ｋ繧医≧縺ｫ縺励◆縲・
    - `sha256_finalize` 縺ｯ scaffold [莉墓ｧ・縺励ｈ縺・縺ｨ縺励※ buffer len 繧端遒ｺ隱・縺九￥縺ｫ繧転縺吶ｋ test 縺ｫ[蛻・縺江繧骸譖ｿ/縺犠縺医◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `sha256` 縺ｯ[譛ｪ螳溯｣・縺ｿ縺倥▲縺昴≧] digest 繧偵�後〒縺阪※縺・ｋ繧医≧縺ｫ[隕・縺ｿ]縺帙ｋ縲阪％縺ｨ繧偵○縺壹�《caffold [谿ｵ髫・縺�繧薙°縺Ь縺ｧ[菫晁ｨｼ/縺ｻ縺励ｇ縺・縺励※縺・ｋ縺薙→縺�縺代ｒ test / comment [荳｡譁ｹ/繧翫ｇ縺・⊇縺・縺ｫ[譏手ｨ・繧√＞縺江縺励◆縲・
  - hash fixture 縺ｧ縺ｯ bare `hash32` overload 縺ｮ[譖匁乂諤ｧ/縺ゅ＞縺ｾ縺・○縺Ь繧端驕ｿ/縺評縺代ｋ縺溘ａ縲…urrent trait 險ｭ險医ｒ[陦ｨ/縺ゅｉ繧従縺・`hash32_by_trait` 繧端菴ｿ/縺､縺犠縺・蠖｢/縺九◆縺｡]縺ｫ縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/tests/hash.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/tests.js -i stdlib/tests/hash.n.md -i tests/stdlib/traits_hash.n.md --no-stdlib --no-tree -o /tmp/tests-hash-focus.json -j 4`
    - [邨先棡/縺代▲縺犠: `4/4 pass`
    - output JSON: `/tmp/tests-hash-focus.json`

# 2026-03-11 菴懈･ｭ繝｡繝｢ (collection fixture / selfhost_req 縺ｮ reboot 霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `tests/stdlib/collections_diag.n.md` 縺ｨ `tests/stdlib/selfhost_req.n.md` 縺ｫ[谿・縺ｮ縺転縺｣縺ｦ縺・◆ old collection API [蜿ら・/縺輔ｓ縺励ｇ縺・繧・current bare API 縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - host filesystem 縺ｮ preopen 縺ｫ[萓晏ｭ・縺・◇繧転縺吶ｋ unstable file I/O testcase 繧・reboot [譁ｹ驥・縺ｻ縺・＠繧転縺ｫ[豐ｿ/縺拆縺｣縺ｦ stable 縺ｪ `Result` [讀懆ｨｼ/縺代ｓ縺励ｇ縺・縺ｸ[謌ｻ/繧ゅ←]縺吶�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `collections_diag` 縺ｯ collection reboot [蜑・縺ｾ縺・縺ｮ `hashmap_new` / `hashmap_insert` / `hashset_new` / `hashset_insert` 縺啓谿句ｭ・縺悶ｓ縺昴ｓ]縺励※縺翫ｊ縲｝ublic API 縺ｨ fixture 縺啓荵夜屬/縺九＞繧馨縺励※縺・◆縲・
  - `selfhost_req` 縺ｮ string map case 縺ｯ[譌｢/縺吶〒]縺ｫ `HashMap<str,.V>` 縺ｸ[邨ｱ蜷・縺ｨ縺・＃縺・縺輔ｌ縺溷ｾ後ｂ `HashMapStr` / `hashmap_str_*` 蜿ら・縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
  - `selfhost_req` 縺ｮ file I/O case 縺ｯ host filesystem 縺ｮ positive-path read 繧・doctest [譛ｬ菴・縺ｻ繧薙◆縺Ь縺ｧ[譛溷ｾ・縺阪◆縺Ь縺励※縺翫ｊ縲｝reopen [譚｡莉ｶ/縺倥ｇ縺・￠繧転縺ｧ `ret: 0` 縺啓荳榊ｮ牙ｮ・縺ｵ縺ゅｓ縺ｦ縺Ь縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tests/stdlib/collections_diag.n.md`
    - `HashMap<i32,i32>` / `HashSet<i32>` 繧端譏守､ｺ/繧√＞縺肋縺励�～new` / `insert` / `remove` 縺ｮ bare API 縺ｸ[譖ｸ/縺犠縺梗謠・縺犠縺医◆縲・
    - [隱ｬ譏取枚/縺帙▽繧√＞縺ｶ繧転縺ｮ `hashmap_insert` / `hashset_insert` 繧・current 蜷・`insert` 縺ｸ[謠・縺昴ｍ]縺医◆縲・
  - `tests/stdlib/selfhost_req.n.md`
    - string map case 繧・`HashMap<str,i32>` + `new<str,i32>` / `insert<str,i32>` / `get<str,i32>` 縺ｸ[遘ｻ陦・縺・％縺・縺励◆縲・
    - compile-fail case 繧・`new<Point, str>` / `insert<Point, str>` 縺ｮ bare API [陦ｨ險・縺ｲ繧・≧縺江縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励�…urrent collection API 縺ｧ繧・`D3081` [譛溷ｾ・縺阪◆縺Ь縺啓蟠ｩ/縺上★]繧後↑縺・％縺ｨ繧端遒ｺ隱・縺九￥縺ｫ繧転縺励◆縲・
    - file I/O case 縺ｯ host positive-path read 繧偵ｄ繧√�［issing file 縺ｫ[蟇ｾ/縺溘＞]縺励※ `Result::Err` 縺啓霑・縺九∴]繧九％縺ｨ繧・stable 縺ｫ[讀懆ｨｼ/縺代ｓ縺励ｇ縺・縺吶ｋ testcase 縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `std/fs` 縺ｯ `tests/stdlib/fs.n.md` 縺ｧ繧・謨ｴ逅・縺帙＞繧馨縺励◆縺ｨ縺翫ｊ縲”ost preopen 縺ｫ[萓晏ｭ・縺・◇繧転縺吶ｋ positive-path read 繧・fixture 縺ｮ[謌仙粥譚｡莉ｶ/縺帙＞縺薙≧縺倥ｇ縺・￠繧転縺ｫ縺励↑縺・�ＡResult` 縺ｨ helper [諢丞袖隲・縺・∩繧阪ｓ]縺ｮ[讀懆ｨｼ/縺代ｓ縺励ｇ縺・繧・stable 縺ｫ[蝗ｺ螳・縺薙※縺Ь縺吶ｋ縲・
  - `selfhost_req` 縺ｯ Rust 蛛ｴ request 縺ｮ[逞戊ｷ｡/縺薙ｓ縺帙″]繧端谿・縺ｮ縺転縺励▽縺､繧ゅ�…urrent reboot public API 縺ｨ[荳�閾ｴ/縺・▲縺｡]縺吶ｋ[蠖｢/縺九◆縺｡]縺ｸ[霑ｽ蠕・縺､縺・§繧・≧]縺輔○繧九�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/selfhost_req.n.md -n 4`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/selfhost_req.n.md -n 6`
    - [邨先棡/縺代▲縺犠: pass (`compile_fail`)
  - `node nodesrc/tests.js -i tests/stdlib/collections_diag.n.md -i tests/stdlib/selfhost_req.n.md --no-stdlib --no-tree -o /tmp/tests-collections-selfhost-current.json -j 4`
    - [邨先棡/縺代▲縺犠: `12/12 pass`
    - output JSON: `/tmp/tests-collections-selfhost-current.json`

# 2026-03-11 菴懈･ｭ繝｡繝｢ (`HashMap` / `HashSet` custom hasher 繧呈髪縺医ｋ compiler 譬ｹ蝗�菫ｮ豁｣)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `HashMap<.K,.V,.H>` / `HashSet<.K,.H>` 縺・user-provided hasher 繧端蛟､/縺ゅ◆縺Ь縺ｨ縺励※[蜿・縺・縺措蜿・縺ｨ]繧後ｋ繧医≧縺ｫ縺励�～Hasher<.K>` trait [邨檎罰/縺代＞繧・縺ｮ dispatch 繧・current compiler / web compile path 縺ｮ[荳｡譁ｹ/繧翫ｇ縺・⊇縺・縺ｧ[螳牙ｮ・縺ゅｓ縺ｦ縺Ь縺輔○繧九�・
  - `field::get` 縺ｮ qualified call 縺ｨ bare `get` 縺ｮ collection API 縺啓陦晉ｪ・縺励ｇ縺・→縺､]縺励↑縺・ｈ縺・↓縺励�¨EPLg2 縺ｮ[蜑咲ｽｮ/縺懊ｓ縺｡][險俶ｳ・縺阪⊇縺・ + overload 隗｣豎ｺ縺ｫ[豐ｿ/縺拆縺｣縺・root fix 繧端蜈･/縺Ь繧後ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `tests/stdlib/traits_hash.n.md` 縺ｮ compile failure 縺ｯ stdlib 蛛ｴ縺ｮ hasher 螳溯｣・〒縺ｯ縺ｪ縺上�…ompiler 縺ｨ web compile path 縺ｮ[荳堺ｸ�閾ｴ/縺ｵ縺・▲縺｡]縺啓逵溷屏/縺励ｓ縺・ｓ]縺�縺｣縺溘�・
  - native / analysis [邨瑚ｷｯ/縺代＞繧江縺ｧ縺ｯ `SourceMap` 繧端菴ｿ/縺､縺犠縺｣縺ｦ qualified import alias 縺九ｉ `field::get` 繧端豁｣/縺溘□]縺励￥[驕ｸ/縺医ｉ]縺ｹ縺ｦ縺・◆縺後�～nepl-web` 縺ｮ compile [邨瑚ｷｯ/縺代＞繧江縺�縺代・ `compile_module(...)` 繧端騾・縺ｨ縺馨縺｣縺ｦ `SourceMap` 縺ｪ縺励〒 typecheck 縺励※縺・◆縲ゅ◎縺ｮ縺溘ａ `field::get` 縺・bare `get` 縺ｫ[蟠ｩ/縺上★]繧後�～HashMap::get` 縺ｨ[陦晉ｪ・縺励ｇ縺・→縺､]縺励※ unresolved trait call 縺ｾ縺ｧ[騾｣骼・繧後ｓ縺評縺励※縺・◆縲・
  - 縺輔ｉ縺ｫ trait impl lookup 繧・applied string 蜷阪〒縺ｯ縺ｪ縺・`base trait name + trait args` 縺ｮ[讒矩��/縺薙≧縺槭≧]縺ｧ[謇ｱ/縺ゅ▽縺犠繧上↑縺・→縲“eneric hasher impl 縺・monomorphize [蠕・縺脳繧・`FuncRef::Trait` 縺ｮ縺ｾ縺ｾ[谿・縺ｮ縺転繧九％縺ｨ縺啓蛻・繧従縺九▲縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nepl-core/src/typecheck.rs`
    - qualified import alias 縺九ｉ target file set 繧端蠑・縺ｲ]縺充莉慕ｵ・縺励￥]縺ｿ繧・`SourceMap` [蛻ｩ逕ｨ/繧翫ｈ縺・縺ｸ[謨ｴ逅・縺帙＞繧馨縺励�《elected qualified callable 縺ｯ `HirExprKind::FnValue(symbol)` 縺ｨ縺励※[菫晄戟/縺ｻ縺肋縺吶ｋ繧医≧縺ｫ縺励◆縲・
    - `field::get` 縺ｮ qualified call 縺・bare `get` 縺ｫ[蟠ｩ/縺上★]繧後★縲…ollection API 縺ｨ縺ｮ overload [陦晉ｪ・縺励ｇ縺・→縺､]繧端驕ｿ/縺評縺代ｉ繧後ｋ繧医≧縺ｫ縺励◆縲・
  - `nepl-core/src/loader.rs`
    - `SourceMap::iter_paths` 繧端霑ｽ蜉�/縺､縺・°]縺励�》ypecheck 縺・import alias 縺ｨ file path suffix 繧端蟇ｾ蠢應ｻ・縺溘＞縺翫≧縺･]縺代ｉ繧後ｋ繧医≧縺ｫ縺励◆縲・
  - `nepl-core/src/hir.rs`, `nepl-core/src/monomorphize.rs`, `nepl-core/src/compiler.rs`, `nepl-core/src/ast.rs`, `nepl-core/src/parser.rs`
    - generic trait / impl 縺ｮ trait args 繧・string 縺ｧ縺ｯ縺ｪ縺充讒矩��/縺薙≧縺槭≧]縺ｧ[菫晄戟/縺ｻ縺肋縺励�～Hasher<.K>` impl 縺ｮ dispatch 縺・monomorphize [蠕・縺脳縺ｫ concrete call 縺ｸ[關ｽ/縺馨縺｡繧九ｈ縺・↓縺励◆縲・
    - monomorphize [谿ｵ髫・縺�繧薙°縺Ь縺ｧ縺ｯ unresolved trait call 繧端讀懈渊/縺代ｓ縺評縺励�“eneric hasher 邨瑚ｷｯ縺ｫ[谿・縺ｮ縺転縺｣縺ｦ縺・↑縺・％縺ｨ繧端菫晁ｨｼ/縺ｻ縺励ｇ縺・縺吶ｋ繧医≧縺ｫ縺励◆縲・
  - `nepl-web/src/lib.rs`
    - web compile [邨瑚ｷｯ/縺代＞繧江繧・`compile_module_with_source_map(...)` 縺ｫ[蛻・縺江繧骸譖ｿ/縺犠縺医�］ative path 縺ｨ[蜷・縺翫↑]縺・`SourceMap` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｧ compile 縺吶ｋ繧医≧縺ｫ縺励◆縲・
    - 蛻・ｊ[蛻・繧従縺醍畑縺ｮ panic catch / debug export 縺ｯ[譛�邨ら噪/縺輔＞縺励ｅ縺・※縺江縺ｫ[髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励�∵￡荵・ｿｮ豁｣縺�縺代ｒ[谿・縺ｮ縺転縺励◆縲・
  - `nepl-core/tests/overload.rs`
    - grouped constructor / specific `get` overload / annotated `let` 縺ｮ regression 繧端霑ｽ蜉�/縺､縺・°]縺励�∽ｻ雁屓縺ｮ root fix 繧・compiler test 縺ｨ縺励※[蝗ｺ螳・縺薙※縺Ь縺励◆縲・
  - `stdlib/core/traits/hash.nepl`, `stdlib/core/traits/hash_key.nepl`, `stdlib/alloc/collections/hashmap.nepl`, `stdlib/alloc/collections/hashset.nepl`
    - custom hasher 繧端蜿・縺・縺措蜿・縺ｨ]繧・current reboot 蠖｢縺ｸ[謨ｴ逅・縺帙＞繧馨縺励◆縲・
  - `stdlib/tests/hashmap*.n.md`, `stdlib/tests/hashset*.n.md`, `tests/stdlib/traits_hash.n.md`, `tests/stdlib/collections_diag.n.md`, `tests/stdlib/pipe_collections.n.md`, `tests/stdlib/selfhost_req.n.md`
    - `DefaultHash32 ()` 縺ｮ繧医≧縺ｪ old [陦ｨ險・縺ｲ繧・≧縺江繧端谿・縺ｮ縺転縺輔★ `DefaultHash32` 縺ｸ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励�…urrent custom hasher / bare collection API [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
  - `stdlib/alloc/string.nepl`
    - hash focused 繧端騾・縺ｨ縺馨縺兌驕守ｨ・縺九※縺Ь縺ｧ[隕・縺ｿ]縺､縺九▲縺滉ｸ�譎・`RegionToken` [蜀榊茜逕ｨ/縺輔＞繧翫ｈ縺・縺ｮ move model [陦晉ｪ・縺励ｇ縺・→縺､]繧端隗｣豸・縺九＞縺励ｇ縺・縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `field::get` 縺ｨ `HashMap::get` 縺ｮ[遶ｶ蜷・縺阪ｇ縺・＃縺・縺ｯ library alias 繧端雜ｳ/縺歉縺励※[蝗樣∩/縺九＞縺ｲ]縺吶ｋ縺ｮ縺ｧ縺ｯ縺ｪ縺上�〈ualified name 隗｣豎ｺ縺ｨ蜑咲ｽｮ險俶ｳ・reduction 縺ｮ root fix 縺ｧ[隗｣豸・縺九＞縺励ｇ縺・縺励◆縲・
  - custom hasher 縺ｯ built-in special case 繧端蠅・縺ｵ]繧・＆縺壹�》rait impl 縺ｨ overload 隗｣豎ｺ縺ｧ[謾ｯ/縺輔＆]縺医ｋ reboot [譁ｹ驥・縺ｻ縺・＠繧転繧端邯ｭ謖・縺・§]縺励◆縲・
  - web path 縺�縺大挨謖吝虚縺ｫ縺ｪ繧九・縺ｯ[險ｭ險・縺帙▲縺代＞]縺ｨ縺励※[謔ｪ/繧上ｋ]縺・・縺ｧ縲‥ebug helper 繧端蟶ｸ險ｭ/縺倥ｇ縺・○縺､]縺帙★ compile path 閾ｪ菴薙ｒ native 縺ｨ[蜷・縺翫↑]縺・`SourceMap` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[謠・縺昴ｍ]縺医◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build`
    - [邨先棡/縺代▲縺犠: success
  - `node nodesrc/run_doctest.js -i tests/stdlib/traits_hash.n.md -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/tests.js -i tests/stdlib/traits_hash.n.md -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/hashset_str.n.md -i tests/stdlib/collections_diag.n.md --no-stdlib --no-tree -o /tmp/tests-hash-focus.json -j 4`
    - [邨先棡/縺代▲縺犠: `13/13 pass`
    - output JSON: `/tmp/tests-hash-focus.json`
  - `cargo test -p nepl-core --test overload grouped_argument_overload_uses_later_items_before_reduction -- --nocapture`
    - [邨先棡/縺代▲縺犠: pass
  - `cargo test -p nepl-core --test overload grouped_constructor_argument_can_flow_into_generic_new_call -- --nocapture`
    - [邨先棡/縺代▲縺犠: pass
  - `cargo test -p nepl-core --test overload more_specific_get_overload_beats_generic_catchall -- --nocapture`
    - [邨先棡/縺代▲縺犠: pass
  - `cargo test -p nepl-core --test overload annotated_let_prefers_specific_get_over_generic_field_get -- --nocapture`
    - [邨先棡/縺代▲縺犠: pass

# 2026-03-12 菴懈･ｭ繝｡繝｢ (stack / ringbuffer / queue 縺ｮ bare API 邨ｱ荳�)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `stack_` / `ringbuffer_` prefix 繧・public API 縺九ｉ[髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励�…ollection reboot [譁ｹ驥・縺ｻ縺・＠繧転縺ｩ縺翫ｊ `new` / `push` / `pop` / `peek` / `len` / `is_empty` / `clear` / `free` 縺ｮ bare 蜷阪∈[邨ｱ荳�/縺ｨ縺・＞縺､]縺吶ｋ縲・
  - `queue` 縺ｯ `ringbuffer` 縺ｮ public API 繧端蜀崎ｼｸ蜈･/縺輔＞繧・↓繧・≧]縺帙★縺ｫ current bare API 縺ｨ[陦晉ｪ・縺励ｇ縺・→縺､]縺励↑縺Ъ蠖｢/縺九◆縺｡]縺ｸ[菴・縺､縺従繧骸譖ｿ/縺犠縺医ｋ縲・
  - reboot [蠕・縺脳縺ｮ collection API 縺ｫ[蜷・縺・繧上○縺ｦ examples / parser / fixtures / compiler doctest 繧端霑ｽ蠕・縺､縺・§繧・≧]縺輔○繧九�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `stack` / `ringbuffer` 縺ｯ bare 蜷・wrapper alias 繧端蠕御ｻ・縺ゅ→縺･]縺代＠縺歇驕取ｸ｡譛・縺九→縺江縺ｮ縺ｾ縺ｾ縺ｧ縲∥ctual public defs 縺・`stack_new` / `ringbuffer_push_back` 縺ｪ縺ｩ譌ｧ prefix 蜷阪ｒ[菫晄戟/縺ｻ縺肋縺励※縺・◆縲・
  - `queue` 縺ｯ bare API 縺ｸ[蟇・繧・縺帙ｋ[騾比ｸｭ/縺ｨ縺｡繧・≧]縺ｧ `ringbuffer` module 繧・alias import 縺励※縺翫ｊ縲～new` / `push` 縺ｪ縺ｩ縺ｮ symbol 髮・粋縺・queue module 蜀・〒[豎壽沒/縺翫○繧転縺輔ｌ縺ｦ `new<i32>` 縺・ambiguous 縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
  - `stack` pipe fixture 縺ｫ[谿・縺ｮ縺転縺｣縺ｦ縺・◆ `let p s |> pop` 縺ｯ current parser / reduction 縺ｧ縺ｯ `let` [逶ｴ蠕・縺｡繧・￥縺脳縺ｮ pipe left-hand side 繧・1 [蛟､/縺ゅ◆縺Ь縺ｸ[逡ｳ/縺溘◆]繧√★縲～D3013` 繧端襍ｷ/縺馨縺薙＠縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/collections/stack.nepl`
    - `stack_new` / `stack_push` / `stack_pop` / `stack_peek` / `stack_len` / `stack_is_empty` / `stack_clear` / `stack_free` 繧・actual def 縺斐→ bare 蜷・`new` / `push` / `pop` / `peek` / `len` / `is_empty` / `clear` / `free` 縺ｸ[謾ｹ蜷・縺九＞繧√＞]縺励◆縲・
    - `stack_pop_keep` / `stack_peek_keep` 繧・`pop_keep` / `peek_keep` 縺ｸ[謠・縺昴ｍ]縺医�∵立 alias block 縺ｯ[蜑企勁/縺輔￥縺倥ｇ]縺励◆縲・
  - `stdlib/alloc/collections/ringbuffer.nepl`
    - `ringbuffer_new` / `ringbuffer_with_capacity` / `ringbuffer_push_back` / `ringbuffer_pop_front` / `ringbuffer_peek_front` / `ringbuffer_len` / `ringbuffer_cap` / `ringbuffer_is_empty` / `ringbuffer_clear` / `ringbuffer_free` 繧・bare 蜷阪∈[謾ｹ蜷・縺九＞繧√＞]縺励◆縲・
    - public wrapper alias 縺ｯ[謦､蜴ｻ/縺ｦ縺｣縺阪ｇ]縺励�”elper 蜷阪□縺代ｒ ringbuffer internal [逕ｨ/繧医≧]縺ｫ[谿・縺ｮ縺転縺励◆縲・
  - `stdlib/alloc/collections/queue.nepl`
    - `RingBuffer<.T>` handle 繧端蜀・桁/縺ｪ縺・⊇縺・縺励※蟋碑ｭｲ縺吶ｋ[蠖｢/縺九◆縺｡]繧偵ｄ繧√�〈ueue 閾ｪ霄ｫ縺・ringbuffer 縺ｨ[蜷・縺翫↑]縺・`[len, cap, head, data_ptr]` header / data layout 繧端逶ｴ謗･/縺｡繧・￥縺帙▽][謇�譛・縺励ｇ繧・≧]縺吶ｋ[螳溯｣・縺倥▲縺昴≧]縺ｸ[蛻・縺江繧骸譖ｿ/縺犠縺医◆縲・
    - 縺薙ｌ縺ｫ繧医ｊ `ringbuffer` module import 縺ｫ繧医ｋ public symbol [豎壽沒/縺翫○繧転繧端譁ｭ/縺歉縺｡縲～queue::new` / `queue::push` 縺ｮ ambiguity 繧・root fix 縺励◆縲・
  - `stdlib/nm/parser.nepl`, `examples/bf.nepl`, `examples/rpn.nepl`
    - stack API 繧・current bare 蜷阪∈[霑ｽ蠕・縺､縺・§繧・≧]縺励◆縲・
  - `stdlib/tests/stack.n.md`, `stdlib/tests/ringbuffer.n.md`, `stdlib/tests/queue.n.md`
    - current bare API + `Result` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
    - stack fixture 縺ｮ `let p s |> pop` 縺ｯ current reduction 縺ｧ stable 縺ｪ `let p <Option<i32>> pop s;` 縺ｸ[譖ｸ/縺犠縺梗謠・縺犠縺医◆縲・
  - `tests/stdlib/stack_collections.n.md`, `tests/stdlib/ringbuffer_collections.n.md`, `tests/stdlib/queue_collections.n.md`, `tests/stdlib/pipe_collections.n.md`, `tests/stdlib/collections_diag.n.md`
    - bare collection API 縺ｸ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励◆縲・
  - `tests/compiler/overload.n.md`
    - stack `new` 繧端菴ｿ/縺､縺犠縺・overload case 縺ｯ current collection [莉墓ｧ・縺励ｈ縺・縺ｩ縺翫ｊ impure main + bare `new` 縺ｸ[霑ｽ蠕・縺､縺・§繧・≧]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `queue` 縺ｮ蝠城｡後・ `ringbuffer` alias import 縺ｮ[荳・縺・∴]縺ｫ wrapper 繧端驥・縺九＆]縺ｭ繧九→[蜀咲匱/縺輔＞縺ｯ縺､]縺吶ｋ縺溘ａ縲｝ublic bare API 繧端蜈ｱ譛・縺阪ｇ縺・ｆ縺・縺励▽縺､ internal layout 縺�縺代ｒ[蜈ｱ譛・縺阪ｇ縺・ｆ縺・縺吶ｋ[蠖｢/縺九◆縺｡]縺ｸ[螟・縺犠縺医◆縲・
  - bare API 蛹悶・ alias 霑ｽ蜉�縺ｧ縺ｪ縺・actual def 縺ｮ rename 縺ｨ縺励※[陦・縺翫％縺ｪ]縺・�〉eboot 縺ｮ縲悟ｾ梧婿莠呈鋤繧端谿・縺ｮ縺転縺輔↑縺・�梗蜴溷援/縺偵ｓ縺昴￥]繧端螳・縺ｾ繧・縺｣縺溘�・
  - stack fixture 縺ｮ `let p s |> pop` 縺ｯ parser / reduction 縺ｮ蛻･隱ｲ鬘後→縺励※[蛻・縺江繧骸蛻・繧従縺代�…ollection reboot batch 縺ｧ縺ｯ current stable syntax 縺ｸ fixture 繧端蟇・繧・縺帙◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `target/debug/nepl-cli -i /tmp/queue_test.nepl --target std --output /tmp/queue-test-out`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/queue.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/queue.n.md -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/stack.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/ringbuffer.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/stack.n.md -n 5`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/stack.n.md -n 6`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/stack_collections.n.md -n 5`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/stack_collections.n.md -n 6`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/pipe_collections.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/pipe_collections.n.md -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 14`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 18`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 19`
    - [邨先棡/縺代▲縺犠: pass

# 2026-03-12 菴懈･ｭ繝｡繝｢ (vec bare API 謨ｴ逅・→ move model 霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `Vec` 縺ｮ public API 繧・alias 縺ｧ縺ｯ縺ｪ縺・actual def 縺ｨ縺励※ bare 蜷阪∈[謠・縺昴ｍ]縺医ｋ縲・
  - vec reboot 縺ｮ[蠖ｱ髻ｿ蜈・縺医＞縺阪ｇ縺・＆縺江縺ｧ縺ゅｋ sort / string / parser / tutorial / overload fixture 繧・current move model 縺ｫ[蜷・縺・繧上○縺ｦ[謨ｴ蜷・縺帙＞縺斐≧]縺輔○繧九�・
  - compiler / web compile path 繧端蜷ｫ/縺ｵ縺従繧�蜑・batch 縺ｮ Rust 蟾ｮ蛻・ｒ trunk build 縺ｧ[蜀咲｢ｺ隱・縺輔＞縺九￥縺ｫ繧転縺励◆縺・∴縺ｧ縲’ocused suite 繧端邱大喧/繧翫ｇ縺上°]縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `vec.nepl` 縺ｯ bare 蜷・wrapper 繧端謖・繧・縺｣縺ｦ縺・◆縺後�∥ctual def 縺・`vec_new` / `vec_push` [邉ｻ/縺代＞]縺ｮ縺ｾ縺ｾ[谿・縺ｮ縺転縺｣縺ｦ縺翫ｊ縲〉eboot 縺ｮ縲径lias 縺ｧ縺ｯ縺ｪ縺丞髪荳�縺ｮ public 蜷阪�梗蜴溷援/縺偵ｓ縺昴￥]縺ｫ[蜿・縺ｯ繧転縺励※縺・◆縲・
  - `set` 縺ｯ collection bare API 縺ｮ[蛟呵｣・縺薙≧縺ｻ]縺ｨ縺励※[閾ｪ辟ｶ/縺励●繧転縺�縺後�…urrent parser/compiler 縺ｧ縺ｯ[莠育ｴ・ｪ・繧医ｄ縺上＃]縺ｨ縺励※[謇ｱ/縺ゅ▽縺犠繧上ｌ繧九◆繧・public 蜷阪↓縺ｧ縺阪★縲」ec 縺ｮ write API 縺ｯ `replace` 繧端邯ｭ謖・縺・§]縺吶ｋ[蠢・ｦ・縺ｲ縺､繧医≧]縺後≠縺｣縺溘�・
  - `Vec` 繧・stack 縺ｫ[蜷・縺・繧上○縺ｦ蜊ｳ蠎ｧ縺ｫ `Result` 蛹悶☆繧九→縲～string` / `diag` / `parser` / `std/test` 縺ｾ縺ｧ impure 蛹悶′[騾｣骼・繧後ｓ縺評縺励�√％縺ｮ batch 縺ｮ[雋ｬ蜍・縺帙″繧�]繧端雜・縺転縺医ｋ縲ゅ％縺薙〒縺ｯ bare API 縺ｨ move fix 繧端蜆ｪ蜈・繧・≧縺帙ｓ]縺励�～Result` 譁ｹ驥昴・[邨ｱ荳�/縺ｨ縺・＞縺､]縺ｯ collection reboot 縺ｮ蠕檎ｶ・batch 縺ｫ[騾・縺翫￥]縺｣縺溘�・
  - tutorial 25 / 26 縺ｨ `traits_order` 縺ｯ `Vec` owner 繧・`len/get/get/...` 縺ｮ繧医≧縺ｫ[隍・焚蝗・縺ｵ縺上☆縺・°縺Ь[隱ｭ/繧・繧薙〒縺翫ｊ縲…urrent move model 縺ｧ縺ｯ[荳榊ｮ牙ｮ・縺ｵ縺ゅｓ縺ｦ縺Ь縺�縺｣縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/collections/vec.nepl`
    - actual def 繧・`new` / `with_capacity` / `len` / `cap` / `data_ptr` / `data_mem_ptr` / `data_len` / `is_empty` / `push` / `get` / `replace` / `pop` / `clear` / `free` 縺ｸ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励◆縲・
    - 譌ｧ alias block 縺ｯ[蜑企勁/縺輔￥縺倥ｇ]縺励◆縲・
    - `push` 縺ｮ[蜀咲｢ｺ菫・縺輔＞縺九￥縺ｻ]縺ｧ `cap = 0` 縺ｮ[譎・縺ｨ縺江縺ｫ `0 * 2 = 0` 縺ｨ縺ｪ縺｣縺ｦ縺・◆[谺�髯･/縺代▲縺九ｓ]繧端菫ｮ豁｣/縺励ｅ縺・○縺Ь縺励�・ [螳ｹ驥・繧医≧繧翫ｇ縺・縺九ｉ縺ｧ繧・1 縺ｸ[諡｡蠑ｵ/縺九￥縺｡繧・≧]縺吶ｋ繧医≧縺ｫ縺励◆縲・
    - doctest#2 縺ｯ `match` arm 縺ｮ unit/i32 [豺ｷ蝨ｨ/縺薙ｓ縺悶＞]繧端隗｣豸・縺九＞縺励ｇ縺・縺励�…urrent API [蠖｢/縺代＞]縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
  - `stdlib/alloc/string.nepl`
    - `sb_append` 縺ｫ[谿・縺ｮ縺転縺｣縺ｦ縺・◆ stale `uwok` 繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励�｝ure vec API 縺ｸ[霑ｽ蠕・縺､縺・§繧・≧]縺励◆縲・
  - `stdlib/nm/parser.nepl`
    - `Stack<NestSection>` 縺ｨ `Vec` 縺ｮ owner 繧端郢ｰ/縺従繧骸霑・縺九∴]縺夕隱ｭ/繧・繧薙〒縺・◆ helper 繧端謨ｴ逅・縺帙＞繧馨縺励�”eader / data+len 繧・1 [蝗・縺九＞]縺�縺措蜿・縺ｨ]繧骸蜃ｺ/縺�]縺励※ raw helper 縺ｸ[貂｡/繧上◆]縺兌螳溯｣・縺倥▲縺昴≧]縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - 縺薙ｌ縺ｫ繧医ｊ close-one / close-all / inline/json [蜻ｨ霎ｺ/縺励ｅ縺・∈繧転縺ｮ move error 繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励◆縲・
  - `tests/stdlib/traits_order.n.md`
    - sort [邨先棡/縺代▲縺犠縺ｮ[讀懆ｨｼ/縺代ｓ縺励ｇ縺・繧・`get` [蜿榊ｾｩ/縺ｯ繧薙・縺従縺九ｉ `data_len + raw load` 縺ｸ[蛻・縺江繧骸譖ｿ/縺犠縺医�｛wner 繧・1 [蝗・縺九＞]縺�縺措隱ｭ/繧・繧�[蠖｢/縺九◆縺｡]縺ｸ[謠・縺昴ｍ]縺医◆縲・
  - `tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md`
    - `VecDataLen` 縺ｨ raw load 繧端菴ｿ/縺､縺犠縺｣縺ｦ[遯・縺ｾ縺ｩ]縺ｮ[蟾ｦ蜿ｳ遶ｯ/縺輔ｆ縺・◆繧転繧端隱ｭ/繧・繧�[蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励�｝refixsum tutorial 縺ｮ move error 繧端隗｣豸・縺九＞縺励ｇ縺・縺励◆縲・
  - `tutorials/getting_started/26_competitive_graph_bfs.n.md`
    - `print_dist` 繧・`len/get/get/...` 縺九ｉ `data_len + raw load` 縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励�《tdout 縺啓遨ｺ/縺九ｉ]縺ｫ縺ｪ繧擬荳榊ｮ牙ｮ・縺ｵ縺ゅｓ縺ｦ縺Ь縺ｪ[謖吝虚/縺阪ｇ縺ｩ縺・繧端隗｣豸・縺九＞縺励ｇ縺・縺励◆縲・
  - `tests/compiler/overload.n.md`, `tests/compiler/overload_nested_generic_push.n.md`
    - vec pure API [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[謌ｻ/繧ゅ←]縺励�《tale `unwrap_ok` 繧端髯､蜴ｻ/縺倥ｇ縺阪ｇ]縺励◆縲・
    - current compiler [謖吝虚/縺阪ｇ縺ｩ縺・縺ｫ[蜷・縺・繧上○縺ｦ ret / compile_fail [譛溷ｾ・�､/縺阪◆縺・■]繧端譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
  - `nepl-core/src/lib.rs`
    - `compile_module_with_source_map` 縺ｮ re-export 繧端謌ｻ/繧ゅ←]縺励�∝燕 batch 縺ｧ[蟆主・/縺ｩ縺・↓繧・≧]縺励◆ web/CLI path [邨ｱ荳�/縺ｨ縺・＞縺､]繧・trunk build [蜿ｯ閭ｽ/縺九・縺・縺ｪ[迥ｶ諷・縺倥ｇ縺・◆縺Ь縺ｸ[菫・縺溘ｂ]縺｣縺溘�・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `replace` 縺ｯ螯･蜊斐〒縺ｯ縺ｪ縺・current parser/compiler 縺ｮ[蛻ｶ邏・縺帙＞繧・￥]繧端雕・縺ｵ]縺ｾ縺医◆ public 蜷阪〒縺ゅｋ縲Ａset` 繧端菴ｿ/縺､縺犠縺・↓縺ｯ keyword / parser [險ｭ險・縺帙▲縺代＞]縺ｮ reboot 縺啓蛻･騾・縺ｹ縺｣縺ｨ]蠢・ｦ√�・
  - `Vec` 縺ｮ `Result` 蛹悶・[蠢・ｦ・縺ｲ縺､繧医≧]縺�縺後�∽ｻ・batch 縺ｧ[謚ｼ/縺馨縺夕霎ｼ/縺転繧�縺ｨ pure/impure [蠅・阜/縺阪ｇ縺・°縺Ь縺ｮ[謨ｴ逅・縺帙＞繧馨縺ｪ縺励↓ library [蜈ｨ蝓・縺懊ｓ縺・″]縺ｸ[豕｢蜿・縺ｯ縺阪ｅ縺・縺吶ｋ縺溘ａ縲〉oot-cause 繧端蛻・屬/縺ｶ繧薙ｊ]縺励※蠕檎ｶ壹・ collection reboot batch 縺ｸ[騾・縺翫￥]縺｣縺溘�・
  - tutorial / trait fixture 縺ｮ move fix 縺ｯ `Copy` [蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｸ[謌ｻ/繧ゅ←]縺吶・縺ｧ縺ｯ縺ｪ縺上�～VecDataLen` 繧・raw load 縺ｧ owner 繧・1 [蝗・縺九＞]縺�縺措隕ｳ貂ｬ/縺九ｓ縺昴￥]縺吶ｋ current ownership model 縺ｫ[蟇・繧・縺帙◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/traits_order.n.md -n 3`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/25_competitive_prefixsum_twopointers.n.md -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tutorials/getting_started/26_competitive_graph_bfs.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/tests.js -i tests/compiler/overload.n.md -i tests/compiler/overload_nested_generic_push.n.md --no-stdlib --no-tree -o /tmp/tests-overload-vec.json -j 2`
    - [邨先棡/縺代▲縺犠: `46/46 pass`
    - output JSON: `/tmp/tests-overload-vec.json`

# 2026-03-12 菴懈･ｭ繝｡繝｢ (list bare API 邨ｱ荳�)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `list_nil` / `list_cons` / `list_push_front` / `list_head` / `list_tail` / `list_len` / `list_get` / `list_free` / `list_reverse` 繧・alias 縺ｧ縺ｯ縺ｪ縺・actual def 縺斐→ bare 蜷阪∈[邨ｱ荳�/縺ｨ縺・＞縺､]縺吶ｋ縲・
  - list fixture / pipe fixture / compiler fixture 繧・current collection reboot [譁ｹ驥・縺ｻ縺・＠繧転縺ｸ[霑ｽ蠕・縺､縺・§繧・≧]縺輔○繧九�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `list.nepl` 縺ｯ public API 繧・doctest 繧よ立 prefix 蜷阪・縺ｾ縺ｾ縺ｧ縲〉eboot 縺ｮ縲碁未謨ｰ蜷阪〒縺ｯ蛹ｺ蛻･縺励↑縺・�梗蜴溷援/縺偵ｓ縺昴￥]縺九ｉ譛�繧・螟・縺ｯ縺咯繧後※縺・◆縲・
  - list doctest 縺ｮ 2 莉ｶ逶ｮ縺ｯ string helper 繧・star import 縺励◆縺ｾ縺ｾ bare `new/head/get/len` 縺ｸ[蟇・繧・縺帙ｋ縺ｨ ambiguity 繧端襍ｷ/縺馨縺薙＠繧・☆縺上�∵ｯ碑ｼ・□縺代↓蠢・ｦ√↑ API 繧・trait 蛛ｴ bare `eq` 縺ｸ[鄂ｮ/縺馨縺梗謠・縺犠縺医ｋ蠢・ｦ√′縺ゅ▲縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/collections/list.nepl`
    - actual def 繧・`new` / `cons` / `push` / `head` / `tail` / `is_empty` / `len` / `get` / `free` / `reverse` 縺ｸ[謾ｹ蜷・縺九＞繧√＞]縺励◆縲・
    - file header 縺ｨ doctest 蜀・・ public 蜷阪ｂ current bare API 縺ｸ[謠・縺昴ｍ]縺医◆縲・
    - string doctest 縺ｯ `alloc/string` helper 縺ｧ縺ｯ縺ｪ縺・`core/traits/eq` 縺ｮ bare `eq` 繧端菴ｿ/縺､縺犠縺・ｈ縺・↓[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `stdlib/tests/list.n.md`
    - mk helper 縺ｨ蜈ｨ check case 繧・bare list API 縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
  - `tests/stdlib/pipe_collections.n.md`
    - list chain example 繧・`new |> push |> push ...` 縺ｨ `len/get` 縺ｮ current bare API 縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
  - `tests/compiler/list_dot_map.n.md`
    - compile_fail fixture 縺ｮ `list.list_nil` 繧・`list.new` 縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - list 縺ｯ allocation failure 繧・`Result` 縺ｧ[陦ｨ/縺ゅｉ繧従縺呎婿驥昴∈縺ｾ縺�[荵・縺ｮ]縺｣縺ｦ縺・↑縺・′縲√％縺ｮ batch 縺ｧ縺ｯ naming reboot 繧端蜆ｪ蜈・繧・≧縺帙ｓ]縺励◆縲・
  - 譁・ｭ怜・豈碑ｼ・・ string module helper 蜷阪↓[萓晏ｭ・縺・◇繧転縺吶ｋ繧医ｊ縲》rait 邨檎罰縺ｮ bare `eq` 縺ｫ[蟇・繧・縺帙◆縺ｻ縺・′ reboot 蜈ｨ菴薙・ naming [譁ｹ驥・縺ｻ縺・＠繧転縺ｨ[謨ｴ蜷・縺帙＞縺斐≧]縺吶ｋ縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/list.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/list.nepl -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/list.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass

# 2026-03-12 菴懈･ｭ繝｡繝｢ (btreemap / btreeset 縺ｮ new/insert Result 蛹・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `btreemap` / `btreeset` 縺ｮ allocation path 繧・stack 邉ｻ縺ｨ[謠・縺昴ｍ]縺医�～new` 縺ｨ grow 繧端莨ｴ/縺ｨ繧ゅ↑]縺・`insert` 繧・`Result<..., Diag>` 縺ｧ[霑・縺九∴]縺吶ｈ縺・↓縺吶ｋ縲・
  - reboot 蠕後・ collection [譁ｹ驥・縺ｻ縺・＠繧転縺ｫ[蜷・縺・繧上○縺ｦ pipe fixture 縺ｨ stdlib tests 繧端霑ｽ蠕・縺､縺・§繧・≧]縺輔○繧九�・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `btreemap` / `btreeset` 縺ｯ bare API 蛹悶％縺晞�ｲ繧薙〒縺・◆縺後�～alloc_raw` [螟ｱ謨・縺励▲縺ｱ縺Ь繧端蛟､/縺ゅ◆縺Ь縺ｧ[陦ｨ迴ｾ/縺ｲ繧・≧縺偵ｓ]縺帙★ pure value 繧端霑・縺九∴]縺励※縺翫ｊ縲＾OM 繧端謇ｱ/縺ゅ▽縺犠縺・collection [譁ｹ驥・縺ｻ縺・＠繧転縺九ｉ[螟・縺ｯ縺咯繧後※縺・◆縲・
  - `btreemap` 縺ｯ `core/field` 繧・bare import 縺励◆縺ｾ縺ｾ collection 閾ｪ霄ｫ縺ｮ `get` 繧端螳夂ｾｩ/縺ｦ縺・℃]縺励※縺翫ｊ縲～len` / `insert` [蜀・Κ/縺ｪ縺・・]縺ｮ `get hm "hdr"` 縺・`BTreeMap::get` 縺ｨ `field::get` 縺ｧ[陦晉ｪ・縺励ｇ縺・→縺､]縺励※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/collections/btreemap.nepl`
    - `new` 繧・`Result<BTreeMap<.K,.V>, Diag>` 縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励�〔eys / values / header [遒ｺ菫・縺九￥縺ｻ]縺ｮ[螟ｱ謨・縺励▲縺ｱ縺Ь繧・`diag_out_of_memory` 縺ｸ[螟画鋤/縺ｸ繧薙°繧転縺励◆縲・
    - `grow` 繧・`Result` 蛹悶＠縲〔eys / values [蜀咲｢ｺ菫・縺輔＞縺九￥縺ｻ]縺ｮ[螟ｱ謨・縺励▲縺ｱ縺Ь縺ｧ[騾比ｸｭ/縺ｨ縺｡繧・≧][隗｣謾ｾ/縺九＞縺ｻ縺・繧端陦・縺翫％縺ｪ]縺｣縺ｦ縺九ｉ `Diag` 繧端霑・縺九∴]縺吶ｈ縺・↓縺励◆縲・
    - `insert` 縺ｯ grow path 繧・`unwrap_ok ... grow` 縺ｧ[蜿・縺・縺代�｝ublic return 繧・`Result` 縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
    - `core/field` import 繧・`field` namespace 縺ｫ[蛻・縺江繧骸譖ｿ/縺犠縺医�”eader [蜿ら・/縺輔ｓ縺励ｇ縺・繧・`field::get` 縺ｫ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励◆縲・
  - `stdlib/alloc/collections/btreeset.nepl`
    - `new` 縺ｨ internal `btreeset_grow`縲√♀繧医・ public `insert` 繧・`Result<BTreeSet<.T>, Diag>` 縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `stdlib/tests/btreemap.n.md`, `stdlib/tests/btreeset.n.md`, `tests/stdlib/pipe_collections.n.md`
    - `must_map` / `must_set` helper 繧端蟆主・/縺ｩ縺・↓繧・≧]縺励�｝ipe 騾｣骼悶〒 `Result` 繧端譏守､ｺ逧・繧√＞縺倥※縺江縺ｫ[隗｣蛹・縺九＞縺ｻ縺・縺吶ｋ current style 縺ｸ[謠・縺昴ｍ]縺医◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `remove` / `contains` / `get` / `len` / `clear` / `free` 縺ｯ allocation 繧端莨ｴ/縺ｨ繧ゅ↑]繧上↑縺・◆繧√�√％縺ｮ batch 縺ｧ縺ｯ pure API 縺ｮ縺ｾ縺ｾ縺ｨ縺励◆縲・
  - `insert` 縺�縺代ｒ `Result` 蛹悶＠縺溘・縺ｯ縲“row 縺ｫ繧医ｋ OOM 縺啓襍ｷ/縺馨縺薙ｊ縺・ｋ[邨瑚ｷｯ/縺代＞繧江繧端豁｣遒ｺ/縺帙＞縺九￥]縺ｫ[蛟､/縺ゅ◆縺Ь縺ｧ[陦ｨ迴ｾ/縺ｲ繧・≧縺偵ｓ]縺吶ｋ縺溘ａ縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/tests/btreemap.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/btreeset.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/pipe_collections.n.md -n 3`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/pipe_collections.n.md -n 4`
    - [邨先棡/縺代▲縺犠: pass

# 2026-03-12 菴懈･ｭ繝｡繝｢ (sort fixture 縺ｮ bare Vec API 霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `tests/stdlib/sort.n.md` 縺ｫ谿九▲縺ｦ縺・◆譌ｧ `vec_*` 螳滉ｽ灘錐繧・current bare API 縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
  - sort return fixture 縺ｫ谿九▲縺ｦ縺・◆ stale expected 繧・current `Vec` [諢丞袖隲・縺・∩繧阪ｓ]縺ｸ[蜷梧悄/縺ｩ縺・″]縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `Vec` 譛ｬ菴薙・ actual def 縺・`new` / `push` / `data_len` 縺ｸ[遘ｻ陦・縺・％縺・縺励◆縺後�《ort fixture 縺�縺代′譌ｧ `vec_new` / `vec_push` / `vec_data_len` 縺ｮ縺ｾ縺ｾ[谿句ｭ・縺悶ｓ縺昴ｓ]縺励※縺・◆縲・
  - `sort_*_ret_vec_is_reusable_after_sort` 縺ｯ 2 [隕∫ｴ�/繧医≧縺拆繧・sort [蠕・縺脳縺ｫ 1 [隕∫ｴ�/繧医≧縺拆縺�縺措霑ｽ蜉�/縺､縺・°]縺励※ `len` 繧端隕・縺ｿ]繧・test 縺ｪ縺ｮ縺ｫ縲∵立 expected `5` 縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tests/stdlib/sort.n.md`
    - `vec_new` / `vec_push` / `vec_data_len` 繧・`new` / `push` / `data_len` 縺ｸ[鄂ｮ謠・縺｡縺九ｓ]縺励◆縲・
    - `sort_quick_ret_vec_is_reusable_after_sort`
    - `sort_heap_ret_vec_is_reusable_after_sort`
    - `sort_merge_ret_vec_is_reusable_after_sort`
      縺ｮ expected `ret` 繧・`3` 縺ｸ[菫ｮ豁｣/縺励ｅ縺・○縺Ь縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - 縺薙％縺ｧ縺ｮ failure 縺ｯ sort [螳溯｣・縺倥▲縺昴≧]縺ｮ bug 縺ｧ縺ｯ縺ｪ縺・fixture 縺ｮ[蜑肴署/縺懊ｓ縺ｦ縺Ь縺壹ｌ縺ｧ縺ゅｊ縲〕ibrary [譛ｬ菴・縺ｻ繧薙◆縺Ь縺ｯ[螟・縺犠縺医★ test 縺�縺代ｒ current public API 縺ｨ current `len` [諢丞袖隲・縺・∩繧阪ｓ]縺ｸ[蟇・繧・縺帙◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 6`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 11`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/sort.n.md -n 15`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/tests.js -i tests/stdlib/sort.n.md --no-stdlib --no-tree -o /tmp/tests-stdlib-sort.json -j 2`
    - [邨先棡/縺代▲縺犠: `22/22 pass`
    - output JSON: `/tmp/tests-stdlib-sort.json`

# 2026-03-12 菴懈･ｭ繝｡繝｢ (compiler fixture 縺ｮ bare List API 霑ｽ蠕・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `tests/compiler/neplg2.n.md` 縺ｫ谿九▲縺ｦ縺・◆ `list_nil` / `list_cons` / `list_get` 繧・current bare API 縺ｸ[謠・縺昴ｍ]縺医ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - list 譛ｬ菴薙・ actual def 縺・`new` / `cons` / `get` 縺ｸ[遘ｻ陦・縺・％縺・縺励◆縺後�…ompiler regression 1 莉ｶ縺�縺代′譌ｧ public 蜷阪・縺ｾ縺ｾ[谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `tests/compiler/neplg2.n.md`
    - `list_get_out_of_bounds_err` 縺ｮ[隱ｬ譏・縺帙▽繧√＞]縺ｨ snippet 繧・`new` / `cons` / `get` 縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i tests/compiler/neplg2.n.md -n 33`
    - [邨先棡/縺代▲縺犠: pass

# 2026-03-12 菴懈･ｭ繝｡繝｢ (StdErrorKind 縺ｮ lower-layer 遘ｻ險ｭ)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `Vec` 繧・`Result` 蛹悶☆繧擬蜑肴署/縺懊ｓ縺ｦ縺Ь縺ｨ縺励※縲～StdErrorKind` 繧・`Diag` [螻､/縺昴≧]縺九ｉ[蛻・縺江繧骸髮｢/縺ｯ縺ｪ]縺励※ lower layer 縺ｸ[遘ｻ/縺・▽]縺吶�・
  - reboot [譁ｹ驥・縺ｻ縺・＠繧転縺ｩ縺翫ｊ縲ー霆ｽ驥・縺代＞繧翫ｇ縺・ error kind 縺ｨ richer diagnostic 繧端蛻・屬/縺ｶ繧薙ｊ]縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `StdErrorKind` 縺・`stdlib/alloc/diag/error.nepl` 縺ｫ[鄂ｮ/縺馨縺九ｌ縺ｦ縺・◆縺溘ａ縲～Vec -> StdErrorKind` 繧端蟆主・/縺ｩ縺・↓繧・≧]縺吶ｋ縺ｨ `vec -> diag/error -> vec` 縺ｮ[蠕ｪ迺ｰ/縺倥ｅ繧薙°繧転[萓晏ｭ・縺・◇繧転縺ｫ縺ｪ繧九�・
  - reboot doc 縺ｮ[諢丞峙/縺・→]縺ｯ `Result<T, StdErrorKind>` 繧端霆ｽ驥・縺代＞繧翫ｇ縺・縺ｪ[蛻ｶ蠕｡/縺帙＞縺弱ｇ] error 縺ｨ縺励�～Diag` / `Outcome` 縺ｯ richer 縺ｪ[險ｺ譁ｭ/縺励ｓ縺�繧転[陦ｨ迴ｾ/縺ｲ繧・≧縺偵ｓ]縺ｨ縺励※[蛻･/縺ｹ縺､][螻､/縺昴≧]縺ｫ[鄂ｮ/縺馨縺上％縺ｨ縺�縺｣縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/core/result.nepl`
    - `StdErrorKind` enum 繧端遘ｻ險ｭ/縺・○縺､]縺励◆縲・
    - `std_error_kind_str` 繧端遘ｻ險ｭ/縺・○縺､]縺励◆縲・
  - `stdlib/alloc/diag/error.nepl`
    - `StdErrorKind` / `std_error_kind_str` 縺ｮ[螳夂ｾｩ/縺ｦ縺・℃]繧端蜑企勁/縺輔￥縺倥ｇ]縺励�～Diag` / `Diags` / `Outcome` [譛ｬ菴・縺ｻ繧薙◆縺Ь縺ｫ[髮・ｸｭ/縺励ｅ縺・■繧・≧]縺輔○縺溘�・
    - file header 繧・current [雋ｬ蜍・縺帙″繧�]縺ｸ[蜷梧悄/縺ｩ縺・″]縺励◆縲・
  - `stdlib/alloc/diag/diag.nepl`
    - `std_error_kind_str` 繧・`core/result` 縺九ｉ[隕・縺ｿ]繧九◆繧√・ import 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
  - `stdlib/tests/diag.n.md`
    - `StdErrorKind` import [蜈・繧ゅ→]縺ｮ[螟画峩/縺ｸ繧薙％縺・縺ｫ[霑ｽ蠕・縺､縺・§繧・≧]縺励◆縲・
    - old assert style 繧・current safe test flow (`checks_print_report` / `checks_exit_code`) 縺ｫ[謠・縺昴ｍ]縺医◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - 莉雁屓縺ｯ `StdErrorKind` 縺ｮ[鄂ｮ/縺馨縺梗蝣ｴ/縺ｰ]縺�縺代ｒ[謨ｴ逅・縺帙＞繧馨縺励�～Diag` helper 縺ｮ public 蜷阪ｄ `Outcome` API 縺ｯ[螟・縺犠縺医※縺・↑縺・�・
  - 縺薙ｌ縺ｧ `Vec` 縺・`Result<..., StdErrorKind>` 繧端霑・縺九∴]縺励※繧・`Diag` [螻､/縺昴≧]縺ｸ縺ｮ[騾・ｵ・縺弱ｃ縺上ｊ繧・≧]縺啓襍ｷ/縺馨縺阪↑縺Ъ蝨溷床/縺ｩ縺�縺Ь縺ｫ縺ｪ縺｣縺溘�・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/tests/error.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/diag.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/diag.n.md -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/traits_serde.n.md -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/io.n.md -n 3`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/tests.js -i stdlib/tests/error.n.md -i stdlib/tests/diag.n.md -i tests/stdlib/traits_serde.n.md -i tests/stdlib/io.n.md --no-stdlib --no-tree -o /tmp/tests-std-error-kind-core.json -j 4`
    - [邨先棡/縺代▲縺犠: `13/13 pass`
    - output JSON: `/tmp/tests-std-error-kind-core.json`
- 2026-03-12: `List` 繧・collection reboot 譁ｹ驥昴∈蟇・○繧九◆繧√�～stdlib/alloc/collections/list.nepl` 縺ｮ `new/cons/push/reverse` 繧・`Result<..., Diag>` 霑泌唆縺ｸ螟画峩縺励◆縲らｩｺ繝ｪ繧ｹ繝郁・菴薙・霑ｽ蜉�遒ｺ菫昴ｒ縺励↑縺・′縲∝・髢矩擇縺ｯ `stack` / `ringbuffer` / `queue` / `btree` 縺ｨ蜷後§ `Result` 譁ｹ驥昴∈謠・∴縺溘�・
- 2026-03-12: `stdlib/tests/list.n.md`, `tests/stdlib/pipe_collections.n.md`, `tests/compiler/list_dot_map.n.md`, `tests/compiler/neplg2.n.md` 繧・current API 縺ｸ霑ｽ蠕薙＠縺溘�Ａnew ... |> uwok |> push ... |> uwok` 縺ｮ荳�陦碁�｣骼悶∈邨ｱ荳�縺励�～reverse` 繧・`uwok` 邨檎罰縺ｧ蜿励￠繧句ｽ｢縺ｫ謠・∴縺溘�・
- 2026-03-12: collection 縺ｮ doc test / fixture 繧・current reboot API 縺ｫ蜷梧悄縺励◆縲・
  - `stdlib/alloc/collections/stack.nepl`
    - doc test 縺ｫ谿九▲縺ｦ縺・◆譌ｧ `new |> uwok` 繧・`unwrap_ok<Stack<...>, Diag> new<...>` 縺ｸ邨ｱ荳�縺励�～push ... |> uwok` 繧・1 陦後↓謠・∴縺溘�・
    - `node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl --no-stdlib --no-tree -o /tmp/tests-stack-docs.json -j 2` 縺ｧ `10/10 pass`縲・
  - `stdlib/alloc/collections/hashmap.nepl`, `stdlib/alloc/collections/hashset.nepl`
    - public API (`new/insert/get/contains/remove/len/free`) 縺ｮ comment 繧呈眠 format 縺ｫ蟇・○縲∝推髢｢謨ｰ縺ｮ usage doctest 繧定ｿｽ蜉�縺励◆縲・
    - hasher 莉倥″ `new` 縺ｮ萓九・縲∵里蟄倬�夐℃萓九↓蜷医ｏ縺帙※ `unwrap_ok<HashMap<...>, Diag> new DefaultHash32` / `unwrap_ok<HashSet<...>, Diag> new DefaultHash32` 縺ｸ謠・∴縺溘�・
    - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/hashmap.nepl -n 1` pass縲・
    - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/hashset.nepl -n 1` pass縲・
  - `stdlib/tests/btreeset.n.md`, `tests/stdlib/pipe_collections.n.md`
    - `BTreeSet` / `Stack` / `RingBuffer` / `Queue` / `HashMap` / `HashSet` 縺ｮ fixture 縺ｫ谿九▲縺ｦ縺・◆譖匁乂縺ｪ bare `new<i32>` 繧・立 pipe 譖ｸ豕輔ｒ current style 縺ｫ譖ｴ譁ｰ縺励◆縲・
  - `tests/compiler/list_dot_map.n.md`
    - `namespace_pathsep_map_with_result` 縺ｯ stale `compile_fail` 縺�縺｣縺溘・縺ｧ normal test (`ret: 2`) 縺ｫ逶ｴ縺励◆縲・
  - focused 讀懆ｨｼ
    - `node nodesrc/tests.js -i stdlib/tests/btreeset.n.md -i tests/stdlib/pipe_collections.n.md -i tests/compiler/list_dot_map.n.md --no-stdlib --no-tree -o /tmp/tests-collections-regression-slice.json -j 4` 縺ｧ `14/14 pass`縲・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (collection public API 縺ｮ doc comment / doctest 霑ｽ蜉�)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - reboot [譁ｹ驥・縺ｻ縺・＠繧転縺ｩ縺翫ｊ縲～alloc/collections` 縺ｮ public API 縺ｫ current bare 蜷阪→ `Result` / `Option` [豬∝о/繧翫ｅ縺・℃]繧端遉ｺ/縺励ａ]縺・usage doctest 繧端蠅・縺ｵ]繧・☆縲・
  - old comment 縺ｮ縺ｾ縺ｾ縲啓菴・縺ｪ縺ｫ]繧端霑・縺九∴]縺吶°縲阪□縺代〒[邨・縺馨繧上▲縺ｦ縺・ｋ髢｢謨ｰ縺ｸ縲…urrent [菴ｿ/縺､縺犠縺Ъ譁ｹ/縺九◆]繧端霑ｽ險・縺､縺・″]縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - collection reboot 縺ｧ public 蜷阪→ return [譁ｹ驥・縺ｻ縺・＠繧転縺ｯ[螟・縺犠繧上▲縺溘′縲～queue` / `ringbuffer` / `btreemap` / `btreeset` 縺ｮ comment 縺ｫ縺ｯ current style 縺ｮ譛�蟆丈ｾ九′[蜊∝・/縺倥ｅ縺・・繧転縺ｫ[辟｡/縺ｪ]縺九▲縺溘�・
  - `queue.clear` 縺ｮ doctest 縺ｧ縺ｯ let [譛ｬ菴・縺ｻ繧薙◆縺Ь縺ｮ[譛ｫ蟆ｾ/縺ｾ縺､縺ｳ]縺ｫ `;` 縺啓谿・縺ｮ縺転縺｣縺ｦ縺翫ｊ縲ー蠑・縺励″]縺・unit 縺ｫ[蟠ｩ/縺上★]繧後※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/collections/queue.nepl`
    - `new` / `with_capacity` / `len` / `is_empty` / `push` / `pop` / `peek` / `clear` / `free` 縺ｫ current style 縺ｮ usage doctest 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `clear` 縺ｮ snippet 縺ｯ `let q0 ...` 縺ｨ `let q clear q0` 縺ｫ[蛻・屬/縺ｶ繧薙ｊ]縺励�〕et [譛ｬ菴・縺ｻ繧薙◆縺Ь縺ｮ unit 蛹悶ｒ[驕ｿ/縺評縺代◆縲・
  - `stdlib/alloc/collections/ringbuffer.nepl`
    - `new` / `with_capacity` / `len` / `cap` / `is_empty` / `push` / `pop` / `peek` / `clear` / `free` 縺ｫ usage doctest 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
  - `stdlib/alloc/collections/btreemap.nepl`
    - `BTreeMap` struct comment 繧・current format 縺ｫ[陬懷ｼｷ/縺ｻ縺阪ｇ縺・縺励�～new` / `len` / `contains` / `get` / `insert` / `remove` / `clear` / `free` 縺ｮ usage doctest 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
  - `stdlib/alloc/collections/btreeset.nepl`
    - `BTreeSet` struct comment 繧・current format 縺ｫ[陬懷ｼｷ/縺ｻ縺阪ｇ縺・縺励�～new` / `len` / `contains` / `insert` / `remove` / `clear` / `free` 縺ｮ usage doctest 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - doctest 縺ｯ reboot doc 縺ｮ[譁ｹ驥・縺ｻ縺・＠繧転縺ｫ[蠕・縺励◆縺珪縺・�、PI 縺ｮ[譛�蟆・縺輔＞縺励ｇ縺・[菴ｿ逕ｨ萓・縺励ｈ縺・ｌ縺Ь縺ｨ current ownership / error [豬∝о/繧翫ｅ縺・℃]繧端遉ｺ/縺励ａ]縺兌逕ｨ騾・繧医≧縺ｨ]縺ｫ[髯仙ｮ・縺偵ｓ縺ｦ縺Ь縺励◆縲・
  - fixture [莉｣譖ｿ/縺�縺・◆縺Ь縺ｧ縺ｯ縺ｪ縺上�｝ublic 髢｢謨ｰ[逶ｴ蜑・縺｡繧・￥縺懊ｓ]縺ｫ鄂ｮ縺・※縲啓隕・縺ｿ]縺歇騾・縺ｨ縺馨繧翫↓[菴ｿ/縺､縺犠縺医ｋ縲阪％縺ｨ繧端菫晁ｨｼ/縺ｻ縺励ｇ縺・縺吶ｋ comment 縺ｸ[蟇・繧・縺帙◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/queue.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/queue.nepl -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/queue.nepl -n 8`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/ringbuffer.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/ringbuffer.nepl -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/ringbuffer.nepl -n 10`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/btreemap.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/btreemap.nepl -n 8`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/btreeset.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/btreeset.nepl -n 7`
    - [邨先棡/縺代▲縺犠: pass

# 2026-03-12 菴懈･ｭ繝｡繝｢ (`Deque` [霑ｽ蜉�/縺､縺・°]縺ｨ nullary `new` [譖ｸ蠑・縺励ｇ縺励″]縺ｮ[邨ｱ荳�/縺ｨ縺・＞縺､])

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `alloc/collections` 縺ｫ `Deque` 繧端霑ｽ蜉�/縺､縺・°]縺励�ー蜑榊ｾ・縺懊ｓ縺脳[荳｡遶ｯ/繧翫ｇ縺・◆繧転 queue 縺ｮ bare API 繧端讓呎ｺ・縺ｲ繧・≧縺倥ｅ繧転縺ｧ[謠・縺昴ｍ]縺医ｋ縲・
  - collection fixture 縺ｫ[谿・縺ｮ縺転縺｣縺ｦ縺・◆ `new<i32> |> unwrap_ok ...` / `new<i32> |> uwok` 繧偵�…urrent [謗ｨ螂ｨ/縺吶＞縺励ｇ縺・縺ｮ `unwrap_ok<..., Diag> new<i32>` [蠖｢/縺代＞]縺ｸ[邨ｱ荳�/縺ｨ縺・＞縺､]縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - nullary overload 縺ｮ `new` 縺ｯ pipe [襍ｷ轤ｹ/縺阪※繧転縺ｫ[鄂ｮ/縺馨縺上→ expected type 縺啓蜊∝・/縺倥ｅ縺・・繧転縺ｫ[莨晄眺/縺ｧ繧薙・]縺帙★縲～D3005 ambiguous overload` 繧端襍ｷ/縺馨縺薙＠縺ｦ縺・◆縲・
  - `Deque` [霑ｽ蜉�/縺､縺・°]蠕後・ fixture 縺ｧ繧ゅ％縺ｮ[譖ｸ蠑・縺励ｇ縺励″]繧偵◎縺ｮ縺ｾ縺ｾ[菴ｿ/縺､縺犠縺｣縺ｦ縺・◆縺溘ａ縲～peek_*` / `pop_*` [莉･蜑・縺・●繧転縺ｫ `new` 縺ｮ[谿ｵ髫・縺�繧薙°縺Ь縺ｧ[螟ｱ謨・縺励▲縺ｱ縺Ь縺励※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/collections/deque.nepl`
    - `Deque<.T>` 縺ｨ `new` / `with_capacity` / `len` / `cap` / `is_empty` / `push_front` / `push_back` / `pop_front` / `pop_back` / `peek_front` / `peek_back` / `clear` / `free` 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - [蜀・Κ/縺ｪ縺・・]縺ｯ ring buffer [逕ｱ譚･/繧・ｉ縺Ь縺ｮ `[len, cap, head, data_ptr]` header 縺ｧ[螳溯｣・縺倥▲縺昴≧]縺励◆縲・
    - public API 縺ｮ doc comment 縺ｯ new policy 縺ｫ[蠕・縺励◆縺珪縺｣縺ｦ usage doctest 繧端莉倅ｸ・縺ｵ繧・縺励◆縲・
  - `stdlib/tests/deque.n.md`, `tests/stdlib/deque_collections.n.md`
    - `Deque` fixture 繧端霑ｽ蜉�/縺､縺・°]縺励�～push_back` / `push_front` / `peek_front` / `peek_back` / `pop_front` / `pop_back` 縺ｮ[蝓ｺ譛ｬ/縺阪⊇繧転[蛻ｩ逕ｨ萓・繧翫ｈ縺・ｌ縺Ь繧端蝗ｺ螳・縺薙※縺Ь縺励◆縲・
  - `stdlib/tests/queue.n.md`, `stdlib/tests/ringbuffer.n.md`, `stdlib/tests/stack.n.md`
    - pipe [襍ｷ轤ｹ/縺阪※繧転縺ｮ `new` 繧・`unwrap_ok<..., Diag> new<...>` 縺ｫ[邨ｱ荳�/縺ｨ縺・＞縺､]縺励◆縲・
  - `tests/stdlib/queue_collections.n.md`, `tests/stdlib/ringbuffer_collections.n.md`, `tests/stdlib/stack_collections.n.md`
    - `new<i32> |> uwok` 繧・`unwrap_ok<..., Diag> new<i32>` 縺ｸ[鄂ｮ謠・縺｡縺九ｓ]縺励�～push ... |> uwok` 縺ｯ 1 [陦・縺弱ｇ縺・縺ｮ縺ｾ縺ｾ[邯ｭ謖・縺・§]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `new` 縺ｮ overload [譖匁乂/縺ゅ＞縺ｾ縺Ь縺輔ｒ fixture [蛛ｴ/縺後ｏ]縺ｧ[驕ｿ/縺評縺代ｋ[譖ｸ蠑・縺励ｇ縺励″]縺ｸ[謠・縺昴ｍ]縺医�～push ... |> uwok` 縺ｮ繧医≧縺ｪ result-based collection pipe [豬∝о/繧翫ｅ縺・℃]縺ｯ[邯ｭ謖・縺・§]縺励◆縲・
  - `Deque` 縺ｯ `Queue` 縺ｨ `RingBuffer` 縺ｮ[荳ｭ髢・縺｡繧・≧縺九ｓ] ADT 縺ｨ縺励※[鄂ｮ/縺馨縺阪�～alloc/collections` 縺ｫ queue family 繧端謠・縺昴ｍ]縺医ｋ[雜ｳ蝣ｴ/縺ゅ＠縺ｰ]縺ｨ縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/tests/deque.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/deque.n.md -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/deque_collections.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/queue.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/ringbuffer.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/stack.n.md -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/queue_collections.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/ringbuffer_collections.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/stack_collections.n.md -n 2`
    - [邨先棡/縺代▲縺犠: pass

# 2026-03-12 菴懈･ｭ繝｡繝｢ (`Fenwick` [霑ｽ蜉�/縺､縺・°]縺ｨ current collection regression [蝗槫庶/縺九＞縺励ｅ縺・)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `alloc/collections` 縺ｫ Fenwick Tree 繧端霑ｽ蜉�/縺､縺・°]縺励�｝refix sum / range sum 縺ｮ bare API 繧端讓呎ｺ・縺ｲ繧・≧縺倥ｅ繧転縺ｧ[謠蝉ｾ・縺ｦ縺・″繧・≧]縺吶ｋ縲・
  - 縺昴・[騾比ｸｭ/縺ｨ縺｡繧・≧]縺ｧ[髴ｲ蜃ｺ/繧阪＠繧・▽]縺励◆ `mem` / `string` / `vec` 縺ｮ current regression 繧端譬ｹ譛ｬ/縺薙ｓ縺ｽ繧転縺九ｉ[蝗槫庶/縺九＞縺励ｅ縺・縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `Fenwick` 縺ｯ owner 蝙九↑縺ｮ縺ｫ縲～field::get` 縺ｧ `fw` 繧端隍・焚蝗・縺ｵ縺上☆縺・°縺Ь[隱ｭ/繧・繧薙〒縺翫ｊ縲［ove model 縺ｨ[陦晉ｪ・縺励ｇ縺・→縺､]縺励※縺・◆縲・
  - `mem` / `string` / `vec` 縺ｫ縺ｯ縲ー蜑咲ｽｮ/縺懊ｓ縺｡][險俶ｳ・縺阪⊇縺・縺ｮ call 縺ｮ[蜀・Κ/縺ｪ縺・・]縺ｸ縺輔ｉ縺ｫ call 繧端蝓・縺・繧ー霎ｼ/縺転繧薙□[邂・園/縺九＠繧Ⅹ縺啓谿・縺ｮ縺転縺｣縺ｦ縺翫ｊ縲…urrent compiler 縺ｧ縺ｯ stack reduction 縺啓荳榊ｮ牙ｮ・縺ｵ縺ゅｓ縺ｦ縺Ь縺�縺｣縺溘�・
  - `mem` / `string` 縺ｮ doc comment doctest 縺ｫ old `assert_*` [豬∝о/繧翫ｅ縺・℃]縺啓谿・縺ｮ縺転縺｣縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/collections/fenwick.nepl`
    - `Fenwick` 縺ｨ `new` / `len` / `add` / `sum_prefix` / `sum_range` / `free` 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - owner [蛟､/縺ゅ◆縺Ь繧端隍・焚蝗・縺ｵ縺上☆縺・°縺Ь[隱ｭ/繧・縺ｾ縺ｪ縺・ｈ縺・�》emporary memory 縺ｨ raw helper 繧端菴ｿ/縺､縺犠縺｣縺ｦ `add` / `sum_prefix` / `sum_range` / `free` 繧端螳溯｣・縺倥▲縺昴≧]縺励◆縲・
    - public usage doctest 繧・new doc comment policy 縺ｫ[蠕・縺励◆縺珪縺｣縺ｦ[莉倅ｸ・縺ｵ繧・縺励◆縲・
  - `stdlib/tests/fenwick.n.md`, `tests/stdlib/fenwick_collections.n.md`
    - `Fenwick` fixture 繧端霑ｽ蜉�/縺､縺・°]縺励�～new |> add ... |> uwok` 縺ｨ `sum_prefix` / `sum_range` 縺ｮ[蝓ｺ譛ｬ/縺阪⊇繧転[蛻ｩ逕ｨ萓・繧翫ｈ縺・ｌ縺Ь繧端蝗ｺ螳・縺薙※縺Ь縺励◆縲・
    - owner [蛟､/縺ゅ◆縺Ь縺ｮ[蜀榊茜逕ｨ/縺輔＞繧翫ｈ縺・繧端驕ｿ/縺評縺代ｋ縺溘ａ縲〈uery 縺斐→縺ｫ[迢ｬ遶・縺ｩ縺上ｊ縺､]縺ｮ `Fenwick` 繧端菴・縺､縺従繧擬蠖｢/縺九◆縺｡]縺ｸ[謠・縺昴ｍ]縺医◆縲・
  - `stdlib/core/mem.nepl`
    - `store_i32 add ...` / `store_u8 add ...` / `load_u8 add ...` 縺ｮ繧医≧縺ｪ nested call 繧・temporary binding 縺ｫ[螻暮幕/縺ｦ繧薙°縺Ь縺励◆縲・
    - doc comment doctest #1 繧・current safe style 縺ｫ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
  - `stdlib/alloc/collections/vec.nepl`
    - `push` 縺ｮ `realloc_ptr` / constructor path 繧・temporary binding 縺ｫ[蛻・ｧ｣/縺ｶ繧薙°縺Ь縺励�…urrent compiler 縺ｧ[螳牙ｮ・縺ゅｓ縺ｦ縺Ь縺励※[隱ｭ/繧・繧√ｋ[蠖｢/縺九◆縺｡]縺ｸ[菫ｮ豁｣/縺励ｅ縺・○縺Ь縺励◆縲・
  - `stdlib/alloc/string.nepl`
    - `str_split` 縺ｨ `u128` parse path 縺ｮ nested call 繧・temporary binding 縺ｫ[螻暮幕/縺ｦ繧薙°縺Ь縺励◆縲・
    - `from_bool` / `from_i32` 縺ｮ doc comment doctest 繧・current safe style 縺ｫ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `Fenwick` 縺ｯ `kpfenwick` 繧偵◎縺ｮ縺ｾ縺ｾ[謖・繧・縺｡[荳・縺・縺偵ｋ縺ｮ縺ｧ縺ｪ縺上�〉eboot 蠕後・ bare API 縺ｨ `Result` [譁ｹ驥・縺ｻ縺・＠繧転縺ｫ[蜷・縺・繧上○縺ｦ `alloc/collections` 縺ｮ owner collection 縺ｨ縺励※[蜀崎ｨｭ險・縺輔＞縺帙▲縺代＞]縺励◆縲・
  - regression fix 縺ｯ fixture [蛛ｴ/縺後ｏ]縺�縺代〒縺ｪ縺上�］ested prefix call 繧・source [蛛ｴ/縺後ｏ]縺ｧ[謗帝勁/縺ｯ縺・§繧Ⅹ縺励※[譬ｹ譛ｬ/縺薙ｓ縺ｽ繧転縺九ｉ[菫ｮ豁｣/縺励ｅ縺・○縺Ь縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build`
    - [邨先棡/縺代▲縺犠: pass・・web/dist` 縺ｮ compiler [譖ｴ譁ｰ/縺薙≧縺励ｓ]繧端遒ｺ隱・縺九￥縺ｫ繧転・・
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/stack.n.md -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/stack.n.md -n 4`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/btreeset.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/core/mem.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/string.nepl -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/string.nepl -n 4`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/fenwick.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/fenwick.nepl -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/fenwick.nepl -n 3`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/fenwick.nepl -n 4`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/fenwick.nepl -n 5`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/fenwick.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/fenwick_collections.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/tests.js -i stdlib/tests/fenwick.n.md -i tests/stdlib/fenwick_collections.n.md --no-stdlib --no-tree -o /tmp/tests-fenwick.json -j 2`
    - [邨先棡/縺代▲縺犠: `3/3 pass`

# 2026-03-12 菴懈･ｭ繝｡繝｢ (`BinaryHeap` [霑ｽ蜉�/縺､縺・°]縺ｨ public doctest [謨ｴ蛯・縺帙＞縺ｳ])

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `alloc/collections` 縺ｫ `BinaryHeap` 繧端霑ｽ蜉�/縺､縺・°]縺励�～Ord` 繧端逕ｨ/繧ゅ■]縺・ｋ priority queue 繧・bare API 縺ｧ[謠蝉ｾ・縺ｦ縺・″繧・≧]縺吶ｋ縲・
  - public doc comment 縺ｫ reboot [譁ｹ驥・縺ｻ縺・＠繧転縺ｩ縺翫ｊ縺ｮ usage doctest 繧端霑ｽ蜉�/縺､縺・°]縺励�’ixture 縺ｨ[謨ｴ蜷・縺帙＞縺斐≧]縺吶ｋ[蠖｢/縺九◆縺｡]縺ｧ[蝗ｺ螳・縺薙※縺Ь縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `Vec` wrapper [譁ｹ蠑・縺ｻ縺・＠縺江縺ｧ縺ｯ縲～vec::Vec<.T>` 縺ｮ namespaced type [險俶ｳ・縺阪⊇縺・縺ｨ owner move model 縺・current compiler / stdlib [譁ｹ驥・縺ｻ縺・＠繧転縺ｫ[蜷・縺・繧上★縲～BinaryHeap` 縺ｮ owner [陦ｨ迴ｾ/縺ｲ繧・≧縺偵ｓ]縺ｨ縺励※[荳榊ｮ牙ｮ・縺ｵ縺ゅｓ縺ｦ縺Ь縺�縺｣縺溘�・
  - `push` / `peek` / `pop` 縺ｮ doc comment usage 繧・`let hp: ... |> push ... |> uwok` 縺ｮ[騾｣骼・繧後ｓ縺評縺ｧ[譖ｸ/縺犠縺上→縲『eb compile path 縺ｮ focused doctest 縺ｧ縺ｯ current overload / layout [蜃ｦ逅・縺励ｇ繧馨縺ｨ[陦晉ｪ・縺励ｇ縺・→縺､]縺励�’ile doctest 縺�縺代′ compile fail 縺励※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/collections/binary_heap.nepl`
    - `BinaryHeap<.T>` 繧・12 byte header `[len, cap, data_ptr]` 縺ｮ owner [讒矩��/縺薙≧縺槭≧]縺ｨ縺励※[霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `new` / `with_capacity` / `len` / `cap` / `is_empty` / `push` / `peek` / `pop` / `free` 繧・bare API 縺ｧ[螳溯｣・縺倥▲縺昴≧]縺励◆縲・
    - sift-up / sift-down 繧・raw header / data pointer helper 縺ｧ[邨・縺従縺ｿ縲｛wner [蛟､/縺ゅ◆縺Ь縺ｮ[螟夐㍾/縺溘§繧・≧][豸郁ｲｻ/縺励ｇ縺・・]繧端驕ｿ/縺評縺代◆縲・
    - public doc comment 縺ｫ usage doctest 繧端霑ｽ蜉�/縺､縺・°]縺励�’ile doctest 縺ｯ current compiler 縺ｧ[螳牙ｮ・縺ゅｓ縺ｦ縺Ь縺ｫ[騾・縺ｨ縺馨繧・explicit `unwrap_ok push hp item` [豬∝о/繧翫ｅ縺・℃]縺ｸ[謠・縺昴ｍ]縺医◆縲・
  - `stdlib/tests/binary_heap.n.md`
    - `push` / `peek` / `pop` / `with_capacity` 縺ｮ focused fixture 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
  - `tests/stdlib/binary_heap_collections.n.md`
    - pipe [險俶ｳ・縺阪⊇縺・縺ｧ `new |> push ... |> uwok` 繧端菴ｿ/縺､縺犠縺・collection-level usage fixture 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `BinaryHeap` 縺ｯ `Vec` 縺ｮ alias 縺ｧ縺ｯ縺ｪ縺上�～Stack` 縺ｨ[蜷檎ｳｻ邨ｱ/縺ｩ縺・￠縺・→縺・縺ｮ owner collection 縺ｨ縺励※[迢ｬ遶・縺ｩ縺上ｊ縺､] header 繧端謖・繧・縺､[蠖｢/縺九◆縺｡]縺ｫ縺励◆縲・
  - public docs 縺ｧ縺ｯ縲啓蠢・縺九↑繧云縺喙騾・縺ｨ縺馨繧・usage縲阪ｒ[蜆ｪ蜈・繧・≧縺帙ｓ]縺励�｝ipe [騾｣骼・繧後ｓ縺評縺ｯ `stdlib/tests` / `tests/stdlib` [蛛ｴ/縺後ｏ]縺ｮ fixture 縺ｧ[菫晁ｨｼ/縺ｻ縺励ｇ縺・縺吶ｋ[蛻・球/縺ｶ繧薙◆繧転縺ｫ縺励◆縲・
  - `let hp: ... |> push ... |> uwok` 縺ｮ file doctest compile fail 縺ｯ current web compiler 縺ｮ layout / overload [谿倶ｻｶ/縺悶ｓ縺代ｓ]縺ｨ縺励※[隱崎ｭ・縺ｫ繧薙＠縺江縺励�ー髢｢謨ｰ蝙・縺九ｓ縺吶≧縺後◆] style [諡｡蠑ｵ/縺九￥縺｡繧・≧] batch 縺ｧ[蜀崎ｨｪ/縺輔＞縺ｻ縺・縺吶ｋ縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/binary_heap.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/binary_heap.nepl -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/binary_heap.nepl -n 3`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/binary_heap.nepl -n 4`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/binary_heap.nepl -n 5`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/binary_heap.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/binary_heap_collections.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/tests.js -i stdlib/tests/binary_heap.n.md -i tests/stdlib/binary_heap_collections.n.md -i stdlib/alloc/collections/binary_heap.nepl --no-stdlib --no-tree -o /tmp/tests-binary-heap.json -j 2`
    - [邨先棡/縺代▲縺犠: `9/9 pass`

# 2026-03-12 菴懈･ｭ繝｡繝｢ (`BloomFilter` [霑ｽ蜉�/縺､縺・°])

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `alloc/collections` 縺ｫ[霑台ｼｼ/縺阪ｓ縺肋 membership test [逕ｨ騾・繧医≧縺ｨ]縺ｮ `BloomFilter<.T,.H>` 繧端霑ｽ蜉�/縺､縺・°]縺励�〉eboot [譁ｹ驥・縺ｻ縺・＠繧転縺ｩ縺翫ｊ bare API 縺ｧ[謠蝉ｾ・縺ｦ縺・″繧・≧]縺吶ｋ縲・
  - public doc comment 縺ｨ fixture 縺ｮ[荳｡譁ｹ/繧翫ｇ縺・⊇縺・縺ｧ `new` / `insert` / `contains` / `clear` / `free` 縺ｮ[菴ｿ/縺､縺犠縺Ъ譁ｹ/縺九◆]繧端蝗ｺ螳・縺薙※縺Ь縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `alloc/collections` 縺ｫ縺ｯ[豁｣遒ｺ/縺帙＞縺九￥]縺ｪ `Set` / `Map` 縺ｯ縺ゅ▲縺ｦ繧ゅ�ー遨ｺ髢・縺上≧縺九ｓ][蜉ｹ邇・縺薙≧繧翫▽]繧端蜆ｪ蜈・繧・≧縺帙ｓ]縺吶ｋ[霑台ｼｼ/縺阪ｓ縺肋髮・粋縺後↑縺上�［embership-heavy 縺ｪ[逕ｨ騾・繧医≧縺ｨ]繧・stdlib [讓呎ｺ・縺ｲ繧・≧縺倥ｅ繧転縺�縺代〒[陦ｨ迴ｾ/縺ｲ繧・≧縺偵ｓ]縺励↓縺上°縺｣縺溘�・
  - current web compiler / doctest path 縺ｧ縺ｯ縲｝ublic doc comment 縺ｮ pipe [騾｣骼・繧後ｓ縺評 usage 縺・`unwrap_ok new ... |> insert ...` 縺ｮ[蠖｢/縺九◆縺｡]縺ｧ[荳榊ｮ牙ｮ・縺ｵ縺ゅｓ縺ｦ縺Ь縺ｫ縺ｪ繧擬邂・園/縺九＠繧Ⅹ縺後≠繧翫�ー螳溯｣・縺倥▲縺昴≧]縺ｧ縺ｯ縺ｪ縺・snippet layout [蛛ｴ/縺後ｏ]縺ｧ compile fail 繧端襍ｷ/縺馨縺薙＠縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/collections/bloom_filter.nepl`
    - `BloomFilter<.T,.H>` 繧・`[bit 髟ｷ/縺｡繧・≧, byte 髟ｷ/縺｡繧・≧, bitset pointer, hasher]` 繧端謖・繧・縺､ owner collection 縺ｨ縺励※[霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `new` / `len` / `insert` / `contains` / `clear` / `free` 繧・bare API 縺ｧ[螳溯｣・縺倥▲縺昴≧]縺励◆縲・
    - bitset 縺ｯ byte [驟榊・/縺ｯ縺・ｌ縺､]縺ｧ[菫晄戟/縺ｻ縺肋縺励�・ [譛ｬ/縺ｼ繧転縺ｮ probe index 繧端菴ｿ/縺､縺犠縺・fixed-probe Bloom Filter 縺ｨ縺励◆縲・
    - `insert` / `contains` / `clear` 縺ｯ temporary raw storage 繧端菴ｿ/縺､縺犠縺｣縺ｦ field 縺ｮ[螟夐㍾/縺溘§繧・≧][隱ｭ/繧・縺ｿ繧端驕ｿ/縺評縺代�…urrent move model 縺ｫ[蜷・縺・繧上○縺溘�・
    - public doc comment 縺ｯ current compiler 縺ｧ[螳牙ｮ・縺ゅｓ縺ｦ縺Ь縺励※[騾・縺ｨ縺馨繧・explicit style 縺ｫ[謠・縺昴ｍ]縺医◆縲・
  - `stdlib/tests/bloom_filter.n.md`
    - `insert + contains` 縺ｨ `clear + invalid len` 縺ｮ focused fixture 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `#2` 縺ｯ nested prefix / generic call 縺ｮ[邨・縺従縺ｿ[蜷・縺・繧上○縺ｧ `main` 譛ｫ蟆ｾ縺・unit [謇ｱ/縺ゅ▽縺犠縺・＆繧後ｋ regression 縺後≠縺｣縺溘◆繧√�～contains` / `is_err` / invalid `new` 繧・1 step 縺壹▽[蛟､/縺ゅ◆縺Ь縺ｸ[關ｽ/縺馨縺ｨ縺・explicit style 縺ｫ[謠・縺昴ｍ]縺医◆縲・
  - `tests/stdlib/bloom_filter_collections.n.md`
    - pipe [險俶ｳ・縺阪⊇縺・縺ｧ `new |> insert ... |> clear` 繧端遒ｺ隱・縺九￥縺ｫ繧転縺吶ｋ collection-level usage fixture 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `BloomFilter` 縺ｯ[豁｣遒ｺ/縺帙＞縺九￥]縺ｪ[髮・粋/縺励ｅ縺・＃縺・縺ｧ縺ｪ縺上�系ot contained 繧端鬮倬�・縺薙≧縺昴￥]縺ｫ[蛻､螳・縺ｯ繧薙※縺Ь縺吶ｋ縲梗蟆ら畑/縺帙ｓ繧医≧][讒矩��/縺薙≧縺槭≧]縺ｨ縺励※ `alloc/collections` 縺ｫ[鄂ｮ/縺馨縺・◆縲・
  - hasher 縺ｯ `HashMap` / `HashSet` 縺ｨ[蜷・縺翫↑]縺倥￥ `.H: Hasher<.T>` 繧端蜿・縺・縺代ｋ owner value 縺ｫ縺励�「ser-provided hasher 繧偵◎縺ｮ縺ｾ縺ｾ[豬・縺ｪ縺珪縺帙ｋ[蠖｢/縺九◆縺｡]縺ｫ縺励◆縲・
  - public doctest 縺ｯ縲啓遒ｺ螳・縺九￥縺倥▽]縺ｫ[騾・縺ｨ縺馨繧・usage縲阪ｒ[蜆ｪ蜈・繧・≧縺帙ｓ]縺励�｝ipe [騾｣骼・繧後ｓ縺評縺ｯ `tests/stdlib` [蛛ｴ/縺後ｏ] fixture 縺ｧ[菫晁ｨｼ/縺ｻ縺励ｇ縺・縺吶ｋ[蛻・球/縺ｶ繧薙◆繧転縺ｫ縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bloom_filter.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bloom_filter.nepl -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bloom_filter.nepl -n 3`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/bloom_filter.nepl -n 4`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/bloom_filter.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/bloom_filter.n.md -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/bloom_filter_collections.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/tests.js -i stdlib/tests/bloom_filter.n.md -i tests/stdlib/bloom_filter_collections.n.md -i stdlib/alloc/collections/bloom_filter.nepl --no-stdlib --no-tree -o /tmp/tests-bloom-filter.json -j 2`
    - [邨先棡/縺代▲縺犠: `9/9 pass`

# 2026-03-12 菴懈･ｭ繝｡繝｢ (`DisjointSet` [霑ｽ蜉�/縺､縺・°])

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `alloc/collections` 縺ｫ `DisjointSet` 繧端霑ｽ蜉�/縺､縺・°]縺励�ゞnion-Find 繧・bare API 縺ｧ[讓呎ｺ匁署萓・縺ｲ繧・≧縺倥ｅ繧薙※縺・″繧・≧]縺吶ｋ縲・
  - public doc comment / `stdlib/tests` / `tests/stdlib` 縺ｮ 3 [螻､/縺昴≧]縺ｧ usage 繧端蝗ｺ螳・縺薙※縺Ь縺励�“raph 繧・grouping 縺ｮ[蝓ｺ逶､/縺阪・繧転繧端蠅・縺ｵ]繧・☆縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `alloc/collections` 縺ｫ縺ｯ queue / heap / tree / hash 縺ｯ[謠・縺昴ｍ]縺｣縺ｦ縺阪◆縺後�ー髮・粋蛻・牡/縺励ｅ縺・＃縺・・繧薙°縺､]繧端謇ｱ/縺ゅ▽縺犠縺・DSU 縺後↑縺上�゜ruskal 繧・connectivity check 縺ｮ[蝓ｺ逶､/縺阪・繧転縺啓谺�/縺犠縺代※縺・◆縲・
  - current owner model 縺ｧ縺ｯ query 繧・receiver 繧端豸郁ｲｻ/縺励ｇ縺・・]縺吶ｋ縺ｮ縺ｧ縲～same` / `size` / `find` 繧端蜷・縺翫↑]縺・owner 縺ｫ[邯・縺､縺･]縺代※[蜻ｼ/繧・縺ｶ fixture 縺ｯ moved-value compile fail 縺ｫ縺ｪ縺｣縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/collections/disjoint_set.nepl`
    - `DisjointSet` 繧・`[n, parent ptr, sizes ptr]` 繧端謖・繧・縺､ owner collection 縺ｨ縺励※[霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `new` / `len` / `find` / `union` / `same` / `size` / `free` 繧・bare API 縺ｧ[螳溯｣・縺倥▲縺昴≧]縺励◆縲・
    - [蜀・Κ/縺ｪ縺・・]縺ｯ `parent[i]` 縺ｨ `sizes[root]` 繧端謖・繧・縺､ classic Union-Find 縺ｧ縲～union` 縺ｯ union-by-size 繧端謗｡逕ｨ/縺輔＞繧医≧]縺励◆縲・
    - public API 縺ｯ pure query 繧端蜆ｪ蜈・繧・≧縺帙ｓ]縺励※ path compression 繧端蜈･/縺Ь繧後★縲～find` / `same` / `size` 縺ｯ[隱ｭ/繧・縺ｿ[蜿・縺ｨ]繧翫□縺代〒[螳檎ｵ・縺九ｓ縺代▽]縺吶ｋ[蠖｢/縺九◆縺｡]縺ｫ縺励◆縲・
  - `stdlib/tests/disjoint_set.n.md`
    - `union + same + size` 縺ｨ invalid index 縺ｮ focused fixture 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - query 縺・owner 繧端豸郁ｲｻ/縺励ｇ縺・・]縺吶ｋ current model 縺ｫ[蜷・縺・繧上○縺ｦ縲ー蜷悟�､/縺ｩ縺・■]縺ｪ DSU 繧端菴・縺､縺従繧骸逶ｴ/縺ｪ縺馨縺励※[蜷・｢ｺ隱・縺九￥縺九￥縺ｫ繧転繧端蛻・屬/縺ｶ繧薙ｊ]縺励◆縲・
  - `tests/stdlib/disjoint_set_collections.n.md`
    - pipe [險俶ｳ・縺阪⊇縺・縺ｧ `new |> union ... |> uwok` 繧端遒ｺ隱・縺九￥縺ｫ繧転縺吶ｋ collection-level usage fixture 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `DisjointSet` 縺ｯ owner [讒矩��/縺薙≧縺槭≧]縺�縺後�｝ublic query 繧・`Result<i32,Diag>` / `Result<bool,Diag>` 縺ｫ[菫・縺溘ｂ]縺､縺溘ａ縲｝ath compression 繧端隕矩�・縺ｿ縺翫￥]縺｣縺ｦ union-by-size 縺ｮ縺ｿ縺ｧ[蟷ｳ陦｡諤ｧ/縺ｸ縺・％縺・○縺Ь繧端遒ｺ菫・縺九￥縺ｻ]縺励◆縲・
  - path compression 繧・public API 縺ｫ[霈・縺ｮ]縺帙ｋ縺ｫ縺ｯ owner 縺ｨ query value 繧端荳�邱・縺・▲縺励ｇ]縺ｫ[霑・縺九∴]縺吝挨險ｭ險医′[隕・縺Ь繧九・縺ｧ縲ー髢｢謨ｰ蝙・縺九ｓ縺吶≧縺後◆] style [謾ｯ謠ｴ/縺励∴繧転 batch 縺ｧ[蜀肴､懆ｨ・縺輔＞縺代ｓ縺ｨ縺・縺吶ｋ縲・
  - doctest 縺ｨ fixture 縺ｯ縲慶urrent owner model 縺ｧ[遒ｺ螳・縺九￥縺倥▽]縺ｫ[騾・縺ｨ縺馨繧・usage縲阪ｒ[蜆ｪ蜈・繧・≧縺帙ｓ]縺励�∝酔縺・owner 縺ｮ[蜀榊茜逕ｨ/縺輔＞繧翫ｈ縺・繧端驕ｿ/縺評縺代◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/disjoint_set.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/disjoint_set.nepl -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/disjoint_set.nepl -n 3`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/disjoint_set.nepl -n 4`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/disjoint_set.nepl -n 5`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/disjoint_set.nepl -n 6`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/disjoint_set.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/disjoint_set.n.md -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/disjoint_set_collections.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/tests.js -i stdlib/tests/disjoint_set.n.md -i tests/stdlib/disjoint_set_collections.n.md -i stdlib/alloc/collections/disjoint_set.nepl --no-stdlib --no-tree -o /tmp/tests-disjoint-set.json -j 2`
    - [邨先棡/縺代▲縺犠: `9/9 pass`

# 2026-03-12 菴懈･ｭ繝｡繝｢ (`SegmentTree` [霑ｽ蜉�/縺､縺・°])

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `alloc/collections` 縺ｫ `SegmentTree` 繧端霑ｽ蜉�/縺､縺・°]縺励�ー轤ｹ譖ｴ譁ｰ/縺ｦ繧薙％縺・＠繧転縺ｨ[荳�闊ｬ蛹ｺ髢・縺・▲縺ｱ繧薙￥縺九ｓ] sum query 縺ｮ[蝨溷床/縺ｩ縺�縺Ь繧端讓呎ｺ門喧/縺ｲ繧・≧縺倥ｅ繧薙°]縺吶ｋ縲・
  - `Fenwick` 縺ｨ[蠖ｹ蜑ｲ/繧・￥繧上ｊ]繧端蛻・繧従縺代�～alloc/collections` 縺ｫ query-oriented tree 繧端蠅・縺ｵ]繧・☆縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `Fenwick` 縺ｯ prefix / range sum 縺ｫ縺ｯ[蜊∝・/縺倥ｅ縺・・繧転縺�縺後�ー螳溯｣・縺倥▲縺昴≧]縺ｮ[隕矩�・縺ｿ縺ｨ縺馨縺励ｄ[荳�闊ｬ蛹ｺ髢・縺・▲縺ｱ繧薙￥縺九ｓ]譛ｨ縺ｮ[蜈･蜿｣/縺・ｊ縺舌■]縺ｨ縺励※縺ｯ `SegmentTree` 繧・蠢・ｦ・縺ｲ縺､繧医≧]縺�縺｣縺溘�・
  - `set` 縺ｯ current parser 縺ｮ[莠育ｴ・ｪ・繧医ｄ縺上＃]縺ｧ public API 蜷阪↓縺ｧ縺阪★縲√◎縺ｮ縺ｾ縺ｾ縺ｧ縺ｯ file doctest 莉･蜑阪↓ source parse 縺啓螢・縺薙ｏ]繧後※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/collections/segment_tree.nepl`
    - `SegmentTree` 繧・`[n, base, data ptr]` 繧端謖・繧・縺､ owner collection 縺ｨ縺励※[霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `new` / `len` / `replace` / `add` / `sum_range` / `free` 繧・bare API 縺ｧ[螳溯｣・縺倥▲縺昴≧]縺励◆縲・
    - [蜀・Κ/縺ｪ縺・・]縺ｯ base 繧・2 [蜀ｪ/縺ｹ縺江縺ｫ[荳ｸ/縺ｾ繧犠繧√◆ iterative segment tree 縺ｨ縺励�〕eaf 縺ｯ `[base, base+n)` 縺ｫ[鄂ｮ/縺馨縺・◆縲・
    - current parser 縺ｮ[蛻ｶ邏・縺帙＞繧・￥]縺ｫ[蠕・縺励◆縺珪縺・�｝oint overwrite 縺ｯ `set` 縺ｧ縺ｪ縺・`replace` 繧・public 蜷阪→縺励◆縲・
  - `stdlib/tests/segment_tree.n.md`
    - `replace + add + sum_range` 縺ｨ invalid index/range 縺ｮ focused fixture 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
  - `tests/stdlib/segment_tree_collections.n.md`
    - pipe [險俶ｳ・縺阪⊇縺・縺ｧ `new |> replace ... |> add ...` 繧端遒ｺ隱・縺九￥縺ｫ繧転縺吶ｋ collection-level usage fixture 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `SegmentTree` 縺ｯ current reboot [谿ｵ髫・縺�繧薙°縺Ь縺ｧ縺ｯ `i32` sum 蟆ら畑縺ｫ[邨・縺励⊂]繧翫�∝ｰ・擂縺ｮ[髢｢謨ｰ蝙・縺九ｓ縺吶≧縺後◆] style / monoid [謾ｯ謠ｴ/縺励∴繧転 batch 縺ｧ generic aggregator 縺ｫ[諡｡蠑ｵ/縺九￥縺｡繧・≧]縺吶ｋ縲・
  - `set` 縺ｧ縺ｪ縺・`replace` 繧端驕ｸ/縺医ｉ]繧薙□縺ｮ縺ｯ `Vec` 縺ｨ[蜷・縺翫↑]縺・parser [蛻ｶ邏・縺帙＞繧・￥]縺ｫ繧医ｋ繧ゅ・縺ｧ縲∝多蜷梗荳肴紛蜷・縺ｵ縺帙＞縺斐≧]縺ｯ[險�隱槫・/縺偵ｓ縺斐′繧従縺ｮ reserved keyword [謨ｴ逅・縺帙＞繧馨 task 縺ｨ[謗･邯・縺帙▽縺槭￥]縺吶ｋ縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/segment_tree.nepl -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/segment_tree.nepl -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/segment_tree.nepl -n 3`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/segment_tree.nepl -n 4`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/segment_tree.nepl -n 5`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/segment_tree.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/segment_tree.n.md -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/stdlib/segment_tree_collections.n.md -n 1`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/tests.js -i stdlib/tests/segment_tree.n.md -i tests/stdlib/segment_tree_collections.n.md -i stdlib/alloc/collections/segment_tree.nepl --no-stdlib --no-tree -o /tmp/tests-segment-tree.json -j 2`
    - [邨先棡/縺代▲縺犠: `8/8 pass`

# 2026-03-12 菴懈･ｭ繝｡繝｢ (fix(compiler): composite size/load/store 繧貞ｮ滉ｽ薙し繧､繧ｺ縺ｫ蜷医ｏ縺帙ｋ)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `size_of<T>` 縺・multi-field struct / tuple / enum / generic apply 縺ｫ蟇ｾ縺励※[豁｣/縺溘□]縺励＞[螳滉ｽ・縺倥▲縺溘＞] size 繧端霑・縺九∴]縺吶ｈ縺・↓縺吶ｋ縲・
  - aggregate value 繧・`load<T>` / `store<T>` 縺ｧ[謇ｱ/縺ゅ▽縺犠縺・→縺阪�～i32` 1 [隱・縺脳縺ｧ縺ｯ縺ｪ縺充螳滉ｽ・縺倥▲縺溘＞] size 縺ｶ繧薙・ byte copy 縺ｨ縺励※ lowering 縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - wasm / llvm codegen 縺ｮ `size_of` / `align_of` 縺ｯ縲～u8` 縺ｨ 64-bit scalar [莉･螟・縺・′縺Ь繧端莠句ｮ滉ｸ・縺倥§縺､縺倥ｇ縺・ 4 byte [謇ｱ/縺ゅ▽縺犠縺・＠縺ｦ縺・◆縲・
  - 縺輔ｉ縺ｫ `load<T>` / `store<T>` 繧・`Struct` / `Tuple` / `Enum` 繧・`i32` 1 [隱・縺脳縺ｨ縺励※ lowering 縺励※縺翫ｊ縲∥ggregate value 縺ｮ round-trip 縺啓螢・縺薙ｏ]繧後※縺・◆縲・
  - wasm [蛛ｴ/縺後ｏ] aggregate `load` 縺ｮ[蛻晏屓/縺励ｇ縺九＞][螳溯｣・縺倥▲縺昴≧]縺ｧ縺ｯ `local.tee` 縺ｫ繧医ｊ[謌ｻ/繧ゅ←]繧・pointer 縺・stack 縺ｫ 2 [蛟・縺転[谿・縺ｮ縺転繧翫�」alidation failure 繧・襍ｷ/縺馨縺阪※縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nepl-core/src/codegen_wasm.rs`
    - `type_storage_size_bytes` / `type_storage_align_bytes` / `is_aggregate_storage_type` 繧端霑ｽ蜉�/縺､縺・°]縲・
    - generic apply 縺ｯ `TypeCtx` clone + type param substitution 縺ｧ field/payload 縺ｮ[螳滉ｽ・縺倥▲縺溘＞] size 繧端蜀榊ｸｰ逧・縺輔＞縺阪※縺江縺ｫ[險育ｮ・縺代＞縺輔ｓ]縲・
    - aggregate `load<T>` / `store<T>` 繧・byte copy lowering 縺ｫ[螟画峩/縺ｸ繧薙％縺・縲・
    - aggregate `load<T>` 縺ｮ `local.tee` 繧・`local.set` 縺ｫ[菫ｮ豁｣/縺励ｅ縺・○縺Ь縺励�仝ASM stack balance 繧端蠕ｩ譌ｧ/縺ｵ縺｣縺阪ｅ縺・縲・
  - `nepl-core/src/codegen_llvm.rs`
    - 蜷檎ｭ峨・ helper 繧端霑ｽ蜉�/縺､縺・°]縲・
    - aggregate `load<T>` / `store<T>` 繧・`i8` [蜊倅ｽ・縺溘ｓ縺Ь縺ｮ copy lowering 縺ｫ[螟画峩/縺ｸ繧薙％縺・縲・
  - `tests/compiler/sizeof.n.md`
    - `sizeof_multi_field_struct_regression` 繧端霑ｽ蜉�/縺､縺・°]縲・
    - `Pair{i32,i32}` 縺ｮ `8 byte`縲～WidePair{i64,i32}` 縺ｮ `12 byte` 繧端蝗ｺ螳・縺薙※縺Ь縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build`
    - [邨先棡/縺代▲縺犠: success
  - `node nodesrc/run_doctest.js -i tests/compiler/sizeof.n.md -n 4`
    - [邨先棡/縺代▲縺犠: pass
- [蟾ｮ逡ｰ/縺輔＞]繝｡繝｢:
  - 縺薙・[菫ｮ豁｣/縺励ｅ縺・○縺Ь縺ｧ `size_of` regression 縺ｯ[隗｣豸・縺九＞縺励ｇ縺・縺励◆縺後�～alloc/collections/trie` 縺ｮ non-empty insert 縺ｯ縺ｾ縺� runtime OOB 縺啓谿・縺ｮ縺転繧九�・
  - `Trie` [霑ｽ蜉�/縺､縺・°] batch 縺ｯ library [蛛ｴ/縺後ｏ] root cause 縺啓譛ｪ蜿取據/縺ｿ縺励ｅ縺・◎縺従縺ｮ縺溘ａ commit 縺励※縺・↑縺・�Ａtrie_build_suffix_chain` 縺ｨ node [謗･邯・縺帙▽縺槭￥] logic 繧・focused 縺ｫ[蜀崎ｪｿ譟ｻ/縺輔＞縺｡繧・≧縺評縺吶ｋ縲・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (alloc/collections/trie 隱ｿ譟ｻ縺ｮ縺ｿ繝ｻ譛ｪ commit)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `alloc/collections` 縺ｮ[遞ｮ鬘・縺励ｅ繧九＞][諡｡蜈・縺九￥縺倥ｅ縺・縺ｨ縺励※ `Trie` 繧端霑ｽ蜉�/縺､縺・°]縺ｧ縺阪ｋ縺九ｒ[隧穂ｾ｡/縺ｲ繧・≧縺犠縺吶ｋ縲・
- [蛻・ｊ蛻・￠/縺阪ｊ繧上￠]:
  - `TrieNode` 縺ｮ push / append / terminal 譖ｴ譁ｰ縺ｾ縺ｧ縺ｯ focused scratch 縺ｧ pass 縺励◆縲・
  - `Trie` owner [蛟､/縺ゅ◆縺Ь縺九ｉ `Vec<TrieNode>` 繧端蜿・縺ｨ]繧骸蜃ｺ/縺�]縺励※ prefix [謗｢邏｢/縺溘ｓ縺輔￥] loop 繧端蝗・縺ｾ繧従縺吶→縺薙ｍ縺ｧ runtime `unreachable` 縺啓蜀咲樟/縺輔＞縺偵ｓ]縺励◆縲・
  - `size_of` / aggregate byte copy [菫ｮ豁｣/縺励ｅ縺・○縺Ь蠕後ｂ[谿・縺ｮ縺転縺｣縺溘◆繧√�〕ibrary [螳溯｣・縺倥▲縺昴≧]縺ｧ縺ｪ縺・current compiler/runtime 縺ｮ縲経wner struct + aggregate field + loop縲阪ｒ縺ｾ縺溘＄ lowering 縺ｮ[蝠城｡・繧ゅｓ縺�縺Ь縺ｨ[蛻､譁ｭ/縺ｯ繧薙□繧転縺励◆縲・
  - `trie_find_child_index` 繧・helper 縺九ｉ inline 縺ｸ[螻暮幕/縺ｦ繧薙°縺Ь縺励※繧ゅ�～insert` / `contains` / `starts_with` 縺ｮ non-empty case 縺ｯ[蜿取據/縺励ｅ縺・◎縺従縺励↑縺九▲縺溘�・
- [蛻､譁ｭ/縺ｯ繧薙□繧転:
  - broken state 繧・stdlib 縺ｫ[豺ｷ/縺ｾ]縺懊↑縺・◆繧√�～trie.nepl` / `stdlib/tests/trie.n.md` / `tests/stdlib/trie_collections.n.md` 縺ｯ譛ｪ commit 縺ｮ縺ｾ縺ｾ[蜑企勁/縺輔￥縺倥ｇ]縺励※ worktree 縺九ｉ[螟・縺ｯ縺咯縺励◆縲・
  - `Trie` 縺ｯ stdlib task 縺ｨ縺励※縺ｯ[谿倶ｻｶ/縺悶ｓ縺代ｓ]縺�縺後�ー谺｡/縺､縺讃縺ｫ[騾ｲ/縺吶☆]繧�縺ｫ縺ｯ compiler/runtime [蛛ｴ/縺後ｏ]縺ｮ[譛�蟆丞・迴ｾ/縺輔＞縺励ｇ縺・＆縺・￡繧転 test 繧端蜈・縺輔″]縺ｫ[菴・縺､縺従繧九∋縺梗谿ｵ髫・縺�繧薙°縺Ь縺ｧ縺ゅｋ縲・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (alloc/collections/adjacency_list 隱ｿ譟ｻ縺ｮ縺ｿ繝ｻ譛ｪ commit)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - sparse graph [蜷・繧�]縺代・ `AdjacencyList` 繧・`alloc/collections` 縺ｫ[霑ｽ蜉�/縺､縺・°]縺ｧ縺阪ｋ縺九ｒ[隧穂ｾ｡/縺ｲ繧・≧縺犠縺吶ｋ縲・
- [蛻・ｊ蛻・￠/縺阪ｊ繧上￠]:
  - `heads | to | next` 縺ｮ 3 [驟榊・/縺ｯ縺・ｌ縺､]繧・1 [譛ｬ/縺ｻ繧転縺ｮ contiguous buffer 縺ｫ[隧ｰ/縺､]繧√ｋ library [險ｭ險・縺帙▲縺代＞]縺ｾ縺ｧ縺ｯ[菴懈・/縺輔￥縺帙＞]縺励◆縲・
  - native compiler 縺ｧ縺ｯ `new + insert + contains` 縺ｮ[譛�蟆丈ｾ・縺輔＞縺励ｇ縺・ｌ縺Ь縺啓騾・縺ｨ縺馨繧倶ｸ�譁ｹ縲『eb compile path 縺ｧ縺ｯ same-`from` edge 繧・2 [譛ｬ/縺ｻ繧転[霑ｽ蜉�/縺､縺・°]縺励◆ case 縺ｧ `contains` 縺・false 縺ｫ縺ｪ縺｣縺溘�・
  - owner aggregate 繧・temporary memory 縺ｫ[騾�驕ｿ/縺溘＞縺ｲ]縺吶ｋ蠖｢縺ｨ縲～hdr + buf_ptr` 縺ｸ[關ｽ/縺馨縺ｨ縺励◆ header-pointer owner 縺ｮ 2 [譯・縺ゅｓ]繧端隧ｦ/縺溘ａ]縺励◆縺後�√←縺｡繧峨ｂ web compile path 縺ｧ縺ｯ `insert` / `contains` / `remove` 縺・`RuntimeError: unreachable` 縺ｸ[蟠ｩ/縺上★]繧後◆縲・
  - [逞・憾/縺励ｇ縺・§繧・≧]縺ｯ library [蛛ｴ/縺後ｏ]縺ｮ linked-list [譖ｴ譁ｰ/縺薙≧縺励ｓ]繧医ｊ縲…urrent compiler/runtime 縺ｮ owner value lowering 縺ｨ aggregate/header [隱ｭ/繧・縺ｿ[蜃ｺ/縺�]縺励・[蠅・阜/縺阪ｇ縺・°縺Ь縺ｫ[萓晏ｭ・縺・◇繧転縺励※縺・ｋ縺ｨ[蛻､譁ｭ/縺ｯ繧薙□繧転縺励◆縲・
- [蛻､譁ｭ/縺ｯ繧薙□繧転:
  - broken state 繧・stdlib 縺ｫ[豺ｷ/縺ｾ]縺懊↑縺・◆繧√�～adjacency_list.nepl` / `stdlib/tests/adjacency_list.n.md` / `tests/stdlib/adjacency_list_collections.n.md` 縺ｯ譛ｪ commit 縺ｮ縺ｾ縺ｾ worktree 縺九ｉ[螟・縺ｯ縺咯縺励◆縲・
  - `AdjacencyList` 縺ｯ stdlib [谿倶ｻｶ/縺悶ｓ縺代ｓ]縺ｨ縺励※ note 縺ｫ[谿・縺ｮ縺転縺励�ー谺｡蝗・縺倥°縺Ь縺ｯ compiler/runtime [蛛ｴ/縺後ｏ]縺ｮ[譛�蟆丞・迴ｾ/縺輔＞縺励ｇ縺・＆縺・￡繧転 test 縺ｨ縺励※[蜈・縺輔″]縺ｫ[蛻・縺江繧骸蜃ｺ/縺�]縺吶�・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (alloc/collections/btreemultiset 隱ｿ譟ｻ縺ｮ縺ｿ繝ｻ譛ｪ commit)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - ordered multiset 繧・`alloc/collections` 縺ｫ[霑ｽ蜉�/縺､縺・°]縺励�ー驥崎､・縺｡繧・≧縺ｵ縺従 key 繧端蛟区焚/縺薙☆縺・縺､縺阪〒[菫晄戟/縺ｻ縺肋縺ｧ縺阪ｋ collection 繧端讓呎ｺ門喧/縺ｲ繧・≧縺倥ｅ繧薙°]縺吶ｋ縲・
- [蛻・ｊ蛻・￠/縺阪ｊ繧上￠]:
  - `BTreeMap<.T, i32>` 縺ｮ count wrapper 縺ｨ縺励※ `BTreeMultiSet` 繧端隧ｦ菴・縺励＆縺従縺励◆縲・
  - 縺励°縺・current owner model 縺ｧ縺ｯ wrapper owner 縺ｨ inner `BTreeMap` owner 縺ｮ[莠碁㍾/縺ｫ縺倥ｅ縺・[謇�譛・縺励ｇ繧・≧]繧端閾ｪ辟ｶ/縺励●繧転縺ｫ[謇ｱ/縺ゅ▽縺犠縺医★縲～insert` / `remove_one` / `clear` 縺ｮ[蜷・園/縺九￥縺励ｇ]縺ｧ `D3053 use of moved value` 縺啓騾｣骼・繧後ｓ縺評縺励◆縲・
  - raw header wrapper 縺ｫ[關ｽ/縺馨縺ｨ縺励※繧ゅ�‥octest fixture 縺ｧ縺ｯ `RuntimeError: unreachable` 縺啓谿・縺ｮ縺転繧翫�〕ibrary [蛛ｴ/縺後ｏ]縺�縺代〒[謨ｴ蜷・縺帙＞縺斐≧]縺励◆ API 縺ｫ[蜿取據/縺励ｅ縺・◎縺従縺励↑縺九▲縺溘�・
- [蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `BTreeMultiSet` 繧・broken state 繧・stdlib 縺ｫ[豺ｷ/縺ｾ]縺懊★縲∬ｩｦ菴懊ヵ繧｡繧､繝ｫ縺ｯ譛ｪ commit 縺ｮ縺ｾ縺ｾ worktree 縺九ｉ[螟・縺ｯ縺咯縺励◆縲・
  - ordered multiset 縺ｯ[譛臥畑/繧・≧繧医≧]縺�縺後�『rapper owner 縺ｨ inner owner 縺ｮ[蜷域・/縺斐≧縺帙＞]繧・current compiler/runtime 縺後←縺薙∪縺ｧ[謾ｯ/縺輔＆]縺医ｉ繧後ｋ縺九ｒ[蜈・縺輔″]縺ｫ[蜀崎ｩ穂ｾ｡/縺輔＞縺ｲ繧・≧縺犠縺吶ｋ縲・

# 2026-03-12 菴懈･ｭ繝｡繝｢ (feat(list): 髢｢謨ｰ蝙・helper 霑ｽ蜉�)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `List` 縺ｫ `map` / `filter` / `fold` / `reduce` / `find` / `any` / `all` 繧端霑ｽ蜉�/縺､縺・°]縺励�》utorial [蜑・縺ｾ縺・縺ｫ[髢｢謨ｰ蝙・縺九ｓ縺吶≧縺後◆] style 縺ｮ[蝓ｺ遉・縺阪◎] API 繧端謨ｴ/縺ｨ縺ｨ縺ｮ]縺医ｋ縲・
  - namespace call regression 縺ｨ縺ゅｏ縺帙※縲～list::map` 縺・current bare API / move model 縺ｧ[閾ｪ辟ｶ/縺励●繧転縺ｫ[菴ｿ/縺､縺犠縺医ｋ縺薙→繧端菫晁ｨｼ/縺ｻ縺励ｇ縺・縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - compiler [蛛ｴ/縺後ｏ]縺ｧ縺ｯ `TypeKind::Function` 縺・trait model 縺ｮ `Copy` [蛻､螳・縺ｯ繧薙※縺Ь縺ｧ false [謇ｱ/縺ゅ▽縺犠縺・・縺ｾ縺ｾ縺ｧ縲ー鬮倬嚴/縺薙≧縺九＞][髢｢謨ｰ/縺九ｓ縺吶≧]繧端蜀榊ｸｰ/縺輔＞縺江 helper 縺ｫ[貂｡/繧上◆]縺吶→ `D3053 use of moved value` 縺啓逋ｺ逕・縺ｯ縺｣縺帙＞]縺励※縺・◆縲・
  - library [蛛ｴ/縺後ｏ]縺ｧ縺ｯ `list_map_impl` 縺・`cons<.U> f load<.T> lst_ptr mapped_tail` 縺ｮ[蠖｢/縺九◆縺｡]縺ｧ nested call 繧偵◎縺ｮ縺ｾ縺ｾ[譖ｸ/縺犠縺・※縺翫ｊ縲∝燕鄂ｮ險俶ｳ輔・[逡ｳ/縺溘◆]縺ｿ[霎ｼ/縺転縺ｿ縺ｧ `f` 縺ｮ[邨先棡/縺代▲縺犠縺ｧ縺ｯ縺ｪ縺充髢｢謨ｰ蛟､/縺九ｓ縺吶≧縺｡]繧Ъ螢・縺薙ｏ]繧後◆[蛟､/縺ゅ◆縺Ь縺・`cons` 縺ｮ head 縺ｸ[豬・縺ｪ縺珪繧啓霎ｼ/縺転繧薙〒縺・◆縲・
  - `tests/compiler/list_dot_map.n.md` 縺ｮ `list_namespace_map_with_list` 繧・empty list 縺ｫ `map` 縺励※ `get 0 |> unwrap` 縺励※縺翫ｊ縲’ixture [蜑肴署/縺懊ｓ縺ｦ縺Ь縺啓隱､/縺ゅｄ縺ｾ]縺｣縺ｦ縺・◆縲・
- [螟画峩/縺ｸ繧薙％縺・:
  - `nepl-core/src/types.rs`
    - `is_copy_with_trait_model` 縺ｨ `is_copy_eligible_inner` 縺ｧ `TypeKind::Function` 繧・`Copy` / copy-eligible [謇ｱ/縺ゅ▽縺犠縺・↓[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
  - `stdlib/alloc/collections/list.nepl`
    - `map` / `filter` / `fold` / `reduce` / `find` / `any` / `all` 縺ｨ internal helper 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `list_map_impl` 縺ｯ `let mapped_head <.U> ...` 繧端邨檎罰/縺代＞繧・縺励※縺九ｉ `cons` 縺吶ｋ[蠖｢/縺九◆縺｡]縺ｸ[螟画峩/縺ｸ繧薙％縺・縺励�］ested call 縺ｮ[隱､隗｣驥・縺斐°縺・＠繧・￥]繧端髦ｲ豁｢/縺ｼ縺・＠]縺励◆縲・
    - public doc comment 縺ｯ current policy / format 縺ｫ[謠・縺昴ｍ]縺医◆縲・
  - `stdlib/tests/list.n.md`
    - `list_functional_helpers` 繧端霑ｽ蜉�/縺､縺・°]縺励�｛wner [蜀榊茜逕ｨ/縺輔＞繧翫ｈ縺・繧端驕ｿ/縺評縺代ｋ縺溘ａ source list 繧端蛟句挨/縺薙∋縺､]縺ｫ[蛻・屬/縺ｶ繧薙ｊ]縺励◆縲・
  - `tests/compiler/list_dot_map.n.md`
    - old compile-fail 繧・current namespace success case 縺ｸ[譖ｴ譁ｰ/縺薙≧縺励ｓ]縺励◆縲・
    - non-empty list 繧・`list::push` 縺ｧ[菴・縺､縺従縺｣縺ｦ縺九ｉ `list::map` 繧端蜻ｼ/繧・縺ｶ fixture 縺ｫ[螟画峩/縺ｸ繧薙％縺・縺励◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `cargo build -p nepl-cli`
    - [邨先棡/縺代▲縺犠: success
  - `NO_COLOR=false trunk build`
    - [邨先棡/縺代▲縺犠: success
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/list.nepl -n 9`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/list.nepl -n 10`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/list.n.md -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/compiler/list_dot_map.n.md -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/tests.js -i stdlib/tests/list.n.md -i tests/compiler/list_dot_map.n.md --no-stdlib --no-tree -o /tmp/tests-list-fp-short.json -j 2`
    - [邨先棡/縺代▲縺犠: `5/5 pass`

# 2026-03-12 菴懈･ｭ繝｡繝｢ (feat(vec): 髢｢謨ｰ蝙・helper 霑ｽ蜉�)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `Vec` 縺ｫ `map` / `filter` / `fold` / `reduce` / `find` / `any` / `all` 繧端霑ｽ蜉�/縺､縺・°]縺励�～List` / `Option` / `Result` 縺ｫ[邯・縺､縺･]縺充髢｢謨ｰ蝙・縺九ｓ縺吶≧縺後◆] style 縺ｮ[蝓ｺ譛ｬ謫堺ｽ・縺阪⊇繧薙◎縺・＆]繧端謠・縺昴ｍ]縺医ｋ縲・
  - bare `map` 縺・`Option` / `Result` 縺ｨ[蜷悟ｱ・縺ｩ縺・″繧Ⅹ縺励※繧ゅ�～Vec` [蛛ｴ/縺後ｏ]縺ｸ[豁｣/縺溘□]縺励￥[隗｣豎ｺ/縺九＞縺代▽]縺輔ｌ繧九％縺ｨ繧・fixture 縺ｧ[蝗ｺ螳・縺薙※縺Ь縺吶ｋ縲・
- [譬ｹ譛ｬ蜴溷屏/縺薙ｓ縺ｽ繧薙￡繧薙＞繧転:
  - `Vec` 縺ｯ owner [讒矩��/縺薙≧縺槭≧]縺ｪ縺ｮ縺ｧ縲～List` 縺ｮ繧医≧縺ｫ node 繧端蜀榊ｸｰ/縺輔＞縺江[讒狗ｯ・縺薙≧縺｡縺従縺吶ｋ縺�縺代〒縺ｪ縺上�ー蜃ｺ蜉・縺励ｅ縺､繧翫ｇ縺従繝舌ャ繝輔ぃ縺ｮ[遒ｺ菫・縺九￥縺ｻ]縺ｨ move model 繧端蜷梧凾/縺ｩ縺・§]縺ｫ[謨ｴ蜷・縺帙＞縺斐≧]縺輔○繧擬蠢・ｦ・縺ｲ縺､繧医≧]縺後≠縺｣縺溘�・
  - `fold` / `reduce` 繧・while loop + `set out f out item` 縺ｮ[蠖｢/縺九◆縺｡]縺ｧ[譖ｸ/縺犠縺上→縲“eneric accumulator `.U` / `.T` 縺・`Copy` 縺ｧ縺ｪ縺Ъ蝣ｴ蜷・縺ｰ縺ゅ＞]縺ｫ `D3054 use of potentially moved value` 縺ｫ縺ｪ縺｣縺溘�・
  - `find` 縺ｧ繧・mutable `Option<.T>` 繧・while [譚｡莉ｶ/縺倥ｇ縺・￠繧転縺ｧ[隱ｭ/繧・繧�縺ｨ縲～.T` 縺・non-`Copy` 縺ｮ[蝣ｴ蜷・縺ｰ縺ゅ＞]縺ｫ moved-value [蛻､螳・縺ｯ繧薙※縺Ь縺ｸ[關ｽ/縺馨縺｡縺溘�・
  - fixture [蛛ｴ/縺後ｏ]繧・`filtered` 繧・`len` 縺ｨ `get` 縺ｧ[蜀榊茜逕ｨ/縺輔＞繧翫ｈ縺・縺励※縺翫ｊ縲…urrent owner model 縺ｧ縺ｯ `D3053` 縺�縺｣縺溘�・
- [螟画峩/縺ｸ繧薙％縺・:
  - `stdlib/alloc/collections/vec.nepl`
    - `vec_read_at` / `vec_write_at` 縺ｨ縲～vec_fold_impl` / `vec_reduce_impl` / `vec_find_impl` 繧端霑ｽ蜉�/縺､縺・°]縺励◆縲・
    - `map` 縺ｯ exact capacity 繧端蜈・縺輔″]縺ｫ[遒ｺ菫・縺九￥縺ｻ]縺励※ raw loop 縺ｧ[隧ｰ/縺､]繧√ｋ[蠖｢/縺九◆縺｡]縺ｫ縺励◆縲・
    - `filter` 縺ｯ 2-pass・・蛟区焚/縺薙☆縺・[險域ｸｬ/縺代＞縺昴￥] -> exact capacity [遒ｺ菫・縺九￥縺ｻ] -> [霆｢蜀・縺ｦ繧薙＠繧ゾ・峨↓縺励�～push` 縺ｮ[騾先ｬ｡/縺｡縺上§][騾｣骼・繧後ｓ縺評繧端驕ｿ/縺評縺代◆縲・
    - `fold` / `reduce` / `find` 縺ｯ蜀榊ｸｰ helper 縺ｫ[蟇・繧・縺帙�“eneric owner / accumulator 縺ｮ moved-value 繧端譬ｹ譛ｬ/縺薙ｓ縺ｽ繧転縺九ｉ[隗｣豸・縺九＞縺励ｇ縺・縺励◆縲・
    - public doc comment 縺ｨ `neplg2:test` 繧・current policy / format 縺ｫ[謠・縺昴ｍ]縺医◆縲・
  - `stdlib/tests/vec.n.md`
    - `vec_functional_helpers` 繧端霑ｽ蜉�/縺､縺・°]縺励�～map` / `filter` / `fold` / `reduce` / `find` / `any` / `all` 縺ｮ focused fixture 繧端謨ｴ蛯・縺帙＞縺ｳ]縺励◆縲・
    - owner [蜀榊茜逕ｨ/縺輔＞繧翫ｈ縺・繧端驕ｿ/縺評縺代ｋ縺溘ａ縲～filtered` 縺ｮ[髟ｷ/縺ｪ縺珪縺票遒ｺ隱・縺九￥縺ｫ繧転縺ｨ[隕∫ｴ�/繧医≧縺拆[遒ｺ隱・縺九￥縺ｫ繧転縺ｯ source 繧端蛻・屬/縺ｶ繧薙ｊ]縺励◆縲・
  - `tests/compiler/list_dot_map.n.md`
    - `vec_map_with_star_alias_works` 繧端霑ｽ蜉�/縺､縺・°]縺励�～alloc/collections/vec` 縺ｨ `core/result` / `core/option` 繧・`as *` 縺ｧ[蜷梧凾/縺ｩ縺・§] import 縺励◆[迥ｶ諷・縺倥ｇ縺・◆縺Ь縺ｧ繧・bare `map<i32,i32>` 縺・`Vec` [迚・縺ｰ繧転縺ｸ[隗｣豎ｺ/縺九＞縺代▽]縺吶ｋ縺薙→繧端蝗ｺ螳・縺薙※縺Ь縺励◆縲・
- [險ｭ險・縺帙▲縺代＞][蛻､譁ｭ/縺ｯ繧薙□繧転:
  - `Vec` helper 縺ｯ[蜈ｨ菴・縺懊ｓ縺溘＞]繧端譁ｰ隕・縺励ｓ縺江 owner 縺ｨ縺励※[霑・縺九∴]縺吶◆繧√�～map` / `filter` 縺ｮ[遒ｺ菫・縺九￥縺ｻ][螟ｱ謨・縺励▲縺ｱ縺Ь縺ｯ `StdErrorKind::OutOfMemory` 縺ｫ[髮・ｴ・縺励ｅ縺・ｄ縺従縺励◆縲・
  - `filter` 繧・2-pass 縺ｫ縺励◆縺ｮ縺ｯ縲…urrent reboot [谿ｵ髫・縺�繧薙°縺Ь縺ｧ `Result` 繧端謖・繧・縺､ owner value 繧・loop [蜀・縺ｪ縺Ь縺ｧ[騾先ｬ｡/縺｡縺上§][譖ｴ譁ｰ/縺薙≧縺励ｓ]縺吶ｋ縺ｨ move model 縺ｨ[譌ｩ譛溯┳蜃ｺ/縺昴≧縺阪□縺｣縺励ｅ縺､]縺啓隍・尅/縺ｵ縺上＊縺､]縺ｫ縺ｪ繧九◆繧√〒縺ゅｋ縲・
  - `fold` / `reduce` / `find` 縺ｯ mutable owner / accumulator 繧端驕ｿ/縺評縺代ｋ縺溘ａ縺ｫ蜀榊ｸｰ helper 繧端驕ｸ/縺医ｉ]縺ｳ縲…ompiler [蛛ｴ/縺後ｏ]縺ｮ霑ｽ蜉�菫ｮ豁｣縺ｪ縺励〒 current model 縺ｫ[蜿・縺翫＆]繧√◆縲・
- [讀懆ｨｼ/縺代ｓ縺励ｇ縺・:
  - `NO_COLOR=false trunk build`
    - [邨先棡/縺代▲縺犠: success
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 4`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec.nepl -n 5`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i stdlib/tests/vec.n.md -n 2`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/run_doctest.js -i tests/compiler/list_dot_map.n.md -n 4`
    - [邨先棡/縺代▲縺犠: pass
  - `node nodesrc/tests.js -i stdlib/tests/vec.n.md -i tests/compiler/list_dot_map.n.md --no-stdlib --no-tree -o /tmp/tests-vec-fp-short.json -j 2`
    - [邨先棡/縺代▲縺犠: `6/6 pass`

# 2026-03-15 菴懈･ｭ繝｡繝｢ (doc: 髢狗匱險育判縺ｨ莉墓ｧ倥・蜀咲｢ｺ隱・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `nepl-core` 縺ｨ `doc/` 縺ｫ譖ｸ縺九ｌ縺ｦ縺・ｋ莉墓ｧ倥�√♀繧医・ `todo.md` 縺ｮ髢狗匱險育判縺ｫ縺､縺・※螳溽樟蜿ｯ閭ｽ縺九�・←蛻・°繧堤｢ｺ隱阪☆繧九�・
- [遒ｺ隱咲ｵ先棡/縺九￥縺ｫ繧薙￠縺｣縺犠:
  - `doc/memory_safety_migration_plan.md` 縺ｮ縲・5. 螳溯｣・━蜈磯�・ｽ阪�阪そ繧ｯ繧ｷ繝ｧ繝ｳ縺ｧ螳夂ｾｩ縺輔ｌ縺ｦ縺・ｋ Phase 1・亥渕逶､菫ｮ豁｣・峨♀繧医・ Phase 2・亥梛繝ｻAPI 蛻・屬・峨・蛹ｺ蛻・￠縺ｨ縲～todo.md` 縺ｮ縲・. 繝｡繝｢繝ｪ螳牙・蝙九Δ繝・Ν繧堤ｵｱ蜷井ｻ墓ｧ倥↓蝓ｺ縺･縺・※螳溯｣・☆繧九�阪・繧ｵ繝夜�・岼縺悟ｮ悟・縺ｫ荳�閾ｴ縺励※縺・ｋ縺薙→繧堤｢ｺ隱阪＠縺溘�・
  - 螳溯｣・婿驥晢ｼ・InternalAlloc` 縺ｮ蛻ｩ逕ｨ縲～MemPtr` 縺ｮ髫秘屬縲～List` 縺ｮ persistent 蛹悶�～VarState` 縺ｮ蟆主・縲・�伜沺謗ｨ隲悶↑縺ｩ・峨・縲；C繧剃ｽｿ逕ｨ縺帙★繝｡繝｢繝ｪ螳牙・諤ｧ繧堤｢ｺ菫昴☆繧起EPLg2縺ｮ逶ｮ讓咎＃謌舌・縺溘ａ縺ｫ髱槫ｸｸ縺ｫ隲也炊逧・°縺､螳溽樟蜿ｯ閭ｽ縺ｫ邨・∩遶九※繧峨ｌ縺ｦ縺・ｋ縲・
  - 縺励◆縺後▲縺ｦ縲∫樟蝨ｨ縺ｮ `todo.md` 縺翫ｈ縺ｳ `plan.md` 縺ｯ驕ｩ蛻・〒縺ゅｊ縲∽ｿｮ豁｣縺ｯ蠢・ｦ√↑縺・→蛻､譁ｭ縺励◆縲・

# 2026-03-15 菴懈･ｭ繝｡繝｢ (doc: todo.md 縺ｮ鬆・ｺ上→蜆ｪ蜈磯�・ｽ阪・驕ｩ豁｣蛹・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `todo.md` 縺ｮ縲茎tdlib 蜀肴ｧ狗ｯ・譛ｬ豬√�阪そ繧ｯ繧ｷ繝ｧ繝ｳ縺ｮ菴懈･ｭ鬆・ｺ上′縲√・繧､繧ｰ繝ｬ繝ｼ繧ｷ繝ｧ繝ｳ險育判 (`memory_safety_migration_plan.md`) 縺ｮ Phase 蛻・牡縺ｨ遏帷崟縺励�∫�ｴ邯ｻ縺励※縺・ｋ・亥・縺ｦ縺ｮ繝｡繝｢繝ｪ螳牙・蛹悶さ繝ｳ繝代う繝ｩ螳溯｣・′ `alloc` 繧・`std` 螻､縺ｮ繝昴う繝ｳ繧ｿ髫秘屬蜀肴ｧ狗ｯ峨・蜑阪↓鄂ｮ縺九ｌ縺ｦ縺・ｋ・牙撫鬘後ｒ隗｣豎ｺ縺励�√い繝ｼ繧ｭ繝・け繝√Ε縺ｮ萓晏ｭ倬未菫ゅ↓豐ｿ縺｣縺滄←蛻・↑鬆・ｺ上↓荳ｦ縺ｳ譖ｿ縺医ｋ縲・
- [菴懈･ｭ蜀・ｮｹ/縺輔℃繧・≧縺ｪ縺・ｈ縺・:
  - `todo.md` 蜀・・繧ｿ繧ｹ繧ｯ邯ｲ鄒・�ｧ繧貞ｮ悟・縺ｫ邯ｭ謖√＠縺､縺､縲∽ｾ晏ｭ伜ｺｦ縺ｮ蠑ｷ縺・渕逶､縺九ｉ騾ｲ繧√ｋ蜴溷援 (`diag/trait` -> `compiler 蜑肴署` -> `core/mem` -> `alloc` -> `繧ｳ繝ｳ繝代う繝ｩ蠕梧ｮｵ繝代せ` -> `runtimes` -> `std` -> `features`) 縺ｫ豐ｿ縺｣縺ｦ蜀肴ｧ区・縲・
  - 繝｡繝｢繝ｪ螳牙・繝槭う繧ｰ繝ｬ繝ｼ繧ｷ繝ｧ繝ｳ縺ｮ蜷・ｮｵ髫弱ｒ縲√Ξ繧､繝､繝ｼ縺斐→縺ｮ謨ｴ蛯吶ヵ繧ｧ繝ｼ繧ｺ縺ｫ驕ｩ蛻・↓蛻・淵繝ｻ驟咲ｽｮ縺励◆:
    - 繧ｳ繝ｳ繝代う繝ｩ蝓ｺ逶､縺ｨ險ｺ譁ｭ謨ｴ蛯・(Phase 0) 繧呈怙蠎冗乢縺ｫ驟咲ｽｮ縲・
    - `core/mem` 縺翫ｈ縺ｳ `alloc` 縺ｮ逕溘・繧､繝ｳ繧ｿ髫秘屬 (Phase 1, 2) 繧偵Λ繧､繝悶Λ繝ｪ螻､蜀肴ｧ狗ｯ峨・蜑榊濠縺ｫ驟咲ｽｮ縲・
    - Purity霑ｽ霍｡縺ｮ螟画峩縲ヽesource IR縺ｫ繧医ｋDrop Elaboration縲√♀繧医・Region 謗ｨ隲・(Phase 4, 5, 6) 繧偵�～alloc` 縺悟ｮ牙・蛹悶＆繧後◆蠕後・縲後さ繝ｳ繝代う繝ｩ隗｣譫舌ヱ繧ｹ縺ｮ蠑ｷ蛹悶�阪ち繧ｹ繧ｯ縺ｨ縺励※驟咲ｽｮ縲・
    - `std/io` 遲峨∈縺ｮ `ExternalIO` 蜉ｹ譫懷ｮ｣險�莉倅ｸ・(Phase 3) 繧・`std` 螻､讒狗ｯ峨ヵ繧ｧ繝ｼ繧ｺ縺ｫ驟咲ｽｮ縲・
- [遒ｺ隱咲ｵ先棡/縺九￥縺ｫ繧薙￠縺｣縺犠:
  - 繝昴う繝ｳ繧ｿ繧帝囈髮｢縺吶ｋ縺ｨ縺・≧繝ｩ繧､繝悶Λ繝ｪ蜑肴署繧偵け繝ｪ繧｢縺励※縺九ｉ繧ｳ繝ｳ繝代う繝ｩ縺ｮ謇�譛画ｨｩ遲峨↓繧医ｋ閾ｪ蜍慕ｮ｡逅・ｩ溯・ (Resource IR) 繧貞ｰ主・縺吶ｋ繧医≧霆碁％菫ｮ豁｣縺輔ｌ縲∫樟螳溽噪縺ｧ隲也炊遐ｴ邯ｻ縺ｮ縺ｪ縺・ち繧ｹ繧ｯ繝ｪ繧ｹ繝医↓菫ｮ豁｣縺輔ｌ縺溘％縺ｨ繧堤｢ｺ隱阪＠縺溘�・

# 2026-03-15 菴懈･ｭ繝｡繝｢ (doc: 蜈ｨ菴謎ｻ墓ｧ倥・萓晏ｭ伜梛繝ｻ蠖｢蠑剰ｨｼ譏弱ヱ繝ｩ繝�繧､繝�縺ｸ縺ｮ蛻ｰ驕泌ｮ溽樟諤ｧ隧穂ｾ｡)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `doc/` 莉･荳九・蜈ｨ繝峨く繝･繝｡繝ｳ繝医ｒ邊ｾ譟ｻ縺励�∵怙邨ら岼讓吶〒縺ゅｋ縲悟ｼｷ蜉帙↑髱咏噪繝ｻ蝙九・謇�譛画ｨｩ讀懈渊縲阪�悟ｮ悟・縺ｪ讀懆ｨｼ縲阪�√◎縺励※縲御ｾ晏ｭ伜梛・・ependent Types・峨↓繧医ｋ蠖｢蠑剰ｨｼ譏弱�阪′螳悟・縺ｫ螳溽樟蜿ｯ閭ｽ縺区・驥阪↓讀懆ｨ弱☆繧九�・
- [遒ｺ隱阪ヵ繧｡繧､繝ｫ/縺九￥縺ｫ繧薙ヵ繧｡繧､繝ｫ]:
  - `trait_system_design.md`, `move_effect_spec.md`, `error.md`, `shadowing.md`, `stdlib_breaking_reboot.md`, `testing.md`, `rewrite_plan.md`, `runtime.md`, `new_tutorial_plan.md` 縺翫ｈ縺ｳ繝｡繝｢繝ｪ螳牙・邉ｻ莉墓ｧ・
- [讀懆ｨ守ｵ先棡繝ｻ蛻・梵/縺代ｓ縺ｨ縺・￠縺｣縺九・縺ｶ繧薙○縺江:
  - **螳檎挑縺ｪ螳溽樟蜿ｯ閭ｽ諤ｧ繧堤｢ｺ隱・*: 迴ｾ蝨ｨ縺ｮ蜷・ｨｮ莉墓ｧ倥・縲∽ｾ晏ｭ伜梛繧・ｽ｢蠑剰ｨｼ譏弱・蟆主・繧貞ｦｨ縺偵ｋ縲梧囓鮟吶・蜑ｯ菴懃畑繝ｻ迥ｶ諷九・髱樊ｱｺ螳壽�ｧ縲阪ｒ繧ｳ繝ｳ繝代う繝ｩ繧｢繝ｼ繧ｭ繝・け繝√Ε縺ｮ譬ｹ蠎輔°繧牙ｾｹ蠎慕噪縺ｫ謗帝勁縺吶ｋ繧医≧險ｭ險医＆繧後※縺・ｋ縲・
  - **CTFE・・ompile-Time Function Evaluation・峨・蠑ｷ蜉帙↑蝨溷床**: `move_effect_spec.md` 縺ｨ `purity_ownership_memory_spec.md` 縺ｫ縺ゅｋ縲悟・驛ｨ逧・↑ `InternalAlloc`・育函繝｡繝｢繝ｪ謫堺ｽ懶ｼ峨ｒ Surface 縺ｮ `Pure` 縺ｫ逡ｳ縺ｿ霎ｼ繧�縲堺ｻ墓ｧ倥→縲窪scape Analysis縲阪・縲∽ｾ晏ｭ伜梛螳溽樟縺ｫ縺翫￠繧区怙蠑ｷ縺ｮ豁ｦ蝎ｨ縺ｨ縺ｪ繧九�ゅ�悟・驛ｨ逧・↓縺ｯ繝溘Η繝ｼ繧ｿ繝悶Ν縺ｧ鬮倬�溘↓螳溯｡後〒縺阪ｋ縺後�∝､夜Κ・亥梛繧ｷ繧ｹ繝・Β蛛ｴ・峨°繧峨・螳悟・縺ｫ邏皮ｲ九↑謨ｰ蟄ｦ逧・未謨ｰ縺ｫ隕九∴繧九�阪→縺・≧諤ｧ雉ｪ縺御ｿ晁ｨｼ縺輔ｌ縺ｦ縺・ｋ縺溘ａ縲∝梛繝√ぉ繝・き縺後さ繝ｳ繝代う繝ｫ譎ゅ↓螳牙・縺ｫ繧ｳ繝ｼ繝峨ｒ隧穂ｾ｡・医Γ繧ｿ險育ｮ暦ｼ峨〒縺阪ｋ縲・
  - **豎ｺ螳夊ｫ也噪縺ｪ蜷榊燕繝ｻ蝙玖ｧ｣豎ｺ**: `rewrite_plan.md` 縺ｮ縲轡efId 繝吶・繧ｹ縺ｮ繝｢繧ｸ繝･繝ｼ繝ｫ隗｣豎ｺ縲阪�形noshadow`縲阪ｄ縲～trait_system_design.md` 縺ｮ縲梧ｧ矩��逧・梛蜷悟�､縺ｫ繧医ｋ trait 蛻ｶ邏・�阪・縲∬ｨｼ譏弱・讀懆ｨｼ譎ゅ↓蠢・�医→縺ｪ繧九�悟錐蜑阪ｄ蝙九・豎ｺ螳夊ｫ也噪蜷御ｸ�諤ｧ縲阪ｒ諡・ｿ昴＠縺ｦ縺・ｋ縲・
  - **GC繝ｬ繧ｹ縺ｨ邏皮ｲ倶ｸ榊､画�ｧ縺ｮ荳｡遶・*: `runtime.md` 遲峨↓迚ｹ險倥＆繧後ｋ縲軍egion Inference 縺ｫ繧医ｋ Persistent 縺ｪ蛟､縺ｮ繧ｹ繧ｳ繝ｼ繝礼ｮ｡逅・�阪・縲∝�､縺悟享謇九↓譖ｸ縺肴鋤繧上ｉ縺ｪ縺・％縺ｨ・井ｸ榊､画�ｧ・峨・髱咏噪險ｼ譏弱◎縺ｮ繧ゅ・縺ｨ險�縺医ｋ縲ゆｾ晏ｭ伜梛縺ｮ蝙九ヱ繝ｩ繝｡繝ｼ繧ｿ縺ｫ螳牙・縺ｫ蛟､繧呈戟縺｡霎ｼ繧√ｋ縲・
- [邨占ｫ・縺代▽繧阪ｓ]:
  - NEPLg2 縺檎樟險育判縺ｧ逶ｮ謖・＠縺ｦ縺・ｋ縲檎ｴ皮ｲ区�ｧ縲阪�悟ｱ�謇�蜿ｯ螟臥憾諷九・髫秘屬縲阪�悟梛莉倥″繝｡繝｢繝ｪ謇�譛画ｨｩ縲阪・3譛ｬ譟ｱ縺ｯ縲√∪縺輔↓螳夂炊險ｼ譏取髪謠ｴ邉ｻ・・oq, Agda遲会ｼ臥嶌蠖薙・蝙九す繧ｹ繝・Β繧呈ｱ守畑繝励Ο繧ｰ繝ｩ繝溘Φ繧ｰ險�隱樔ｸ翫↓讒狗ｯ峨☆繧九◆繧√・ **Must-Have・亥ｿ・�郁ｦ∽ｻｶ・・* 繧偵☆縺ｹ縺ｦ貅�縺溘＠縺ｦ縺・ｋ縲・
  - 繧｢繝ｼ繧ｭ繝・け繝√Ε縺ｮ螟画峩繧貞・縺丞ｿ・ｦ√→縺帙★縲∫樟蝨ｨ縺ｮ繝槭う繝ｫ繧ｹ繝医・繝ｳ繧貞ｮ碁≠縺励◆蟒ｶ髟ｷ邱壻ｸ翫↓縲靴TFE縺ｮ諡｡蜈・�阪�卦otality・亥●豁｢諤ｧ・峨メ繧ｧ繝・け縺ｮ蟆主・縲阪�悟多鬘悟梛縺ｮ霑ｽ蜉�縲阪→縺・≧蠖｢縺ｧ閾ｪ辟ｶ縺ｫ萓晏ｭ伜梛繝ｻ蠖｢蠑剰ｨｼ譏弱ｒ謗･邯壼庄閭ｽ縺ｧ縺ゅｋ縺ｨ邨占ｫ悶▼縺代◆縲・

# 2026-03-15 菴懈･ｭ繝｡繝｢ (doc: 萓晏ｭ伜梛繝ｻ蠖｢蠑剰ｨｼ譏弱↓蜷代￠縺滉ｸ区嶌縺堺ｻ墓ｧ俶嶌縺ｮ菴懈・)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - 萓晏ｭ伜梛縺ｮ蟆主・縺ｨ蠖｢蠑剰ｨｼ譏弱ｒ隕区紺縺医◆ `doc/dependent_type_proof_plan.md` 繧剃ｽ懈・縺励�∫樟蝨ｨ騾ｲ陦御ｸｭ縺ｮ繝｡繝｢繝ｪ螳牙・繝ｻPurity蛹悶・繝励Ο繧ｸ繧ｧ繧ｯ繝医′縺ｩ縺ｮ繧医≧縺ｫ蟆・擂縺ｮ蠑ｷ蜉帙↑蝙区､懈渊・・TFE, Totality Checker, 蜻ｽ鬘悟梛・峨∈縺ｨ謗･邯壹＆繧後ｋ縺九・髱貞・逵溘ｒ谿九☆縲・
- [菴懈･ｭ蜀・ｮｹ/縺輔℃繧・≧縺ｪ縺・ｈ縺・:
  - `doc/dependent_type_proof_plan.md` 繧剃ｽ懈・縺励�∵悴譚･縺ｮ讒区Φ・井ｸ牙､ｧ霑ｽ蜉�隕∫ｴ�・峨→迴ｾ蝨ｨ縺ｮ蝨溷床・・scape Analysis, Region/Drop, 豎ｺ螳夊ｫ也噪隗｣豎ｺ・峨′鬮倥＞隕ｪ蜥梧�ｧ繧呈戟縺､縺薙→繧偵ラ繧ｭ繝･繝｡繝ｳ繝亥喧縺励◆縲・
  - Vec縺ｮ髟ｷ縺輔ｒ蝙九Ξ繝吶Ν縺ｧ霑ｽ霍｡縺吶ｋ讒区枚縺ｮ繧ｹ繧ｱ繝・メ繧定ｿｽ蜉�縺励�∝ｽ｢蠑剰ｨｼ譏主ｰ主・蠕後・繝励Ο繧ｰ繝ｩ繝溘Φ繧ｰ縺ｮ譛ｪ譚･蜒上ｒ謠千､ｺ縺励◆縲・

# 2026-03-27 菴懈･ｭ繝｡繝｢ (compare / migration 縺ｮ蜈ｨ譁・ｪｭ縺ｿ邯咏ｶ・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `doc/2.1spec/` 縺九ｉ螟悶＠縺滓立隱槫ｽ吶ｄ譛ｪ謗｡逕ｨ讖溯・縺後�～doc/compare/` 縺ｨ `doc/migration/` 縺ｫ谿九▲縺ｦ豁｣莉墓ｧ倥・繧医≧縺ｫ隱ｭ縺ｾ繧後↑縺・ｈ縺・紛逅・☆繧九�・
- [菴懈･ｭ蜀・ｮｹ/縺輔℃繧・≧縺ｪ縺・ｈ縺・:
  - `doc/compare/index.md` 縺ｮ縲瑚ｿｽ蜉�縺輔ｌ繧九ｂ縺ｮ縲阪°繧・`noshadow let` 縺ｨ縺・≧譁ｭ螳夊｡ｨ迴ｾ繧貞､悶＠縲∝酔荳�繧ｷ繧ｰ繝阪メ繝｣蜀榊ｮ夂ｾｩ縺ｮ菫晁ｭｷ縺ｯ蟆・擂諡｡蠑ｵ蛟呵｣懊→縺励※菫晉蕗荳ｭ縺�縺ｨ譏手ｨ倥＠縺溘�・
  - `doc/compare/module_system.md` 縺ｮ `module parser:` / `module lexer:` 萓九↓谿九▲縺ｦ縺・◆譌ｧ placeholder 繧偵�∫樟陦後・ `let <name> <expr>` 縺ｨ lambda 縺ｧ隱ｭ繧√ｋ萓九∈菫ｮ豁｣縺励◆縲・
  - `doc/migration/index.md` 縺ｮ tutorial 諠ｳ螳壹ヵ繧｡繧､繝ｫ蜷・`33_noshadow_and_overload.n.md` 繧偵�∫樟陦・core 莉墓ｧ倥→陦晉ｪ√＠縺ｪ縺・`33_overload_and_redefinition.n.md` 縺ｫ譖ｴ譁ｰ縺励◆縲・
- [plan.md縺ｨ縺ｮ蟾ｮ逡ｰ/縺評:
  - `plan.md` 縺ｮ逶ｮ讓呵・菴薙↓螟画峩縺ｯ縺ｪ縺・�・
  - 譁・嶌鄒､縺ｮ陬懷勧雉・侭蛛ｴ縺ｧ谿九▲縺ｦ縺・◆譌ｧ譯郁ｪ槫ｽ吶ｒ謨ｴ逅・＠縲～doc/2.1spec/` 繧呈ｭ｣縺ｮ莉墓ｧ倥→縺励※隱ｭ繧�蟆守ｷ壹ｒ陬懷ｼｷ縺励◆縲・

# 2026-03-27 菴懈･ｭ繝｡繝｢ (root doc / compare 縺ｮ蜈ｨ譁・ｪｭ縺ｿ邯咏ｶ・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `doc/README.md` 縺ｨ `doc/compare/` 縺ｮ陦ｨ迴ｾ繧偵�～doc/2.1spec/index.md` 縺ｮ迴ｾ蝨ｨ縺ｮ繧ｹ繝・・繧ｿ繧ｹ謨ｴ逅・→謠・∴繧九�・
- [菴懈･ｭ蜀・ｮｹ/縺輔℃繧・≧縺ｪ縺・ｈ縺・:
  - `doc/README.md` 縺ｮ `2.1spec/` 隱ｬ譏弱ｒ縲√�悟ｮ悟・縺ｪ莉墓ｧ倥�阪→譁ｭ螳壹☆繧玖｡ｨ迴ｾ縺九ｉ縲∝推遶�縺ｧ draft / 蟆・擂莉墓ｧ倥ｒ譏守､ｺ縺吶ｋ迴ｾ蝨ｨ縺ｮ謨ｴ逅・↓蜷医ｏ縺帙◆隱ｬ譏弱∈譖ｴ譁ｰ縺励◆縲・
  - `doc/compare/syntax.md` 縺ｮ 0 蠑墓焚髢｢謨ｰ萓九ｒ縲～let main \\(): ...` 縺ｨ縺・≧譌ｧ block 逵∫払縺ｮ隕九∴譁ｹ縺九ｉ縲∫樟蝨ｨ縺ｮ螳｣險�隱ｬ譏弱→鮨滄ｽｬ縺ｮ蟆代↑縺・`let main \() 0` 縺ｸ蟾ｮ縺玲崛縺医◆縲・
- [plan.md縺ｨ縺ｮ蟾ｮ逡ｰ/縺評:
  - `plan.md` 縺ｮ逶ｮ讓呵・菴薙↓螟画峩縺ｯ縺ｪ縺・�・
  - 豁｣縺ｮ莉墓ｧ倥ｒ譯亥・縺吶ｋ蜈･蜿｣譁・嶌縺ｨ豈碑ｼ・枚譖ｸ縺ｮ陦ｨ迴ｾ繧偵�∫樟蝨ｨ縺ｮ `2.1spec` 縺ｮ繧ｹ繝・・繧ｿ繧ｹ陦ｨ遉ｺ縺ｫ蜷医ｏ縺帙◆縲・

# 2026-03-27 菴懈･ｭ繝｡繝｢ (root tool doc 縺ｮ蜈ｨ譁・ｪｭ縺ｿ邯咏ｶ・

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - root 縺ｮ陬懷勧譁・嶌鄒､縺ｧ繧ゅ�∫樟陦・Bootstrap 螳溯｣・・隱ｬ譏弱→ NEPLg2.1 縺ｮ豁｣縺ｮ莉墓ｧ倥′豺ｷ縺悶▲縺ｦ隕九∴縺ｪ縺・ｈ縺・↓蠅・阜繧呈純縺医ｋ縲・
- [菴懈･ｭ蜀・ｮｹ/縺輔℃繧・≧縺ｪ縺・ｈ縺・:
  - `doc/debug.md` 縺ｫ蟇ｾ雎｡螳溯｣・ｳｨ險倥ｒ霑ｽ蜉�縺励�√％縺ｮ譁・嶌縺檎樟陦・`nepl-core` / `nepl-cli` 縺ｮ debug build 謖吝虚繧定ｪｬ譏弱☆繧九ｂ縺ｮ縺ｧ縲∵ｭ｣縺ｮ莉墓ｧ倥・ `doc/2.1spec/` 縺�縺ｨ譏手ｨ倥＠縺溘�・
  - `doc/llvm_ir_setup.md` 縺ｫ蟇ｾ雎｡螳溯｣・ｳｨ險倥ｒ霑ｽ蜉�縺励�√％縺ｮ譁・嶌縺・LLVM target 縺ｮ髢狗匱迺ｰ蠅・Γ繝｢縺ｧ縺ゅｊ縲》arget 險ｭ險医◎縺ｮ繧ゅ・縺ｯ `doc/2.1spec/platform.md` 繧貞盾辣ｧ縺吶∋縺阪□縺ｨ譏手ｨ倥＠縺溘�・
- [plan.md縺ｨ縺ｮ蟾ｮ逡ｰ/縺評:
  - `plan.md` 縺ｮ逶ｮ讓呵・菴薙↓螟画峩縺ｯ縺ｪ縺・�・
  - root 譁・嶌鄒､縺ｮ繧ｹ繝・・繧ｿ繧ｹ陦ｨ遉ｺ繧偵◎繧阪∴縲～doc/2.1spec/` 繧呈ｭ｣縺ｮ莉墓ｧ倥→縺励※隱ｭ繧�蟆守ｷ壹ｒ陬懷ｼｷ縺励◆縲・

# 2026-03-27 菴懈･ｭ繝｡繝｢ (2.1spec 谿倶ｻｶ縺ｮ謨ｴ蜷井ｿｮ豁｣)

- [逶ｮ逧・繧ゅ￥縺ｦ縺江:
  - `2.1spec` 縺ｮ遶�髢捺紛蜷医・谿倶ｻｶ繧呈紛逅・＠縲〇enn #1 / #2 繧呈ｭ｣縺ｨ縺励◆繧ｳ繧｢莉墓ｧ倥・蜻ｨ蝗ｲ縺ｫ谿九▲縺ｦ縺・◆譛ｪ螳夂ｾｩ隱槭ｄ陦ｨ險倥★繧後ｒ隗｣豸医☆繧九�・
- [菴懈･ｭ蜀・ｮｹ/縺輔℃繧・≧縺ｪ縺・ｈ縺・:
  - `doc/2.1spec/modules.md` 縺ｮ `merge` 繧帝�・ｺ上↑縺・multiset 縺ｧ縺ｯ縺ｪ縺・declaration sequence 縺ｨ縺励※螳夂ｾｩ縺礼峩縺励�√�悟ｾ瑚�・━蜈医�阪・諢丞袖縺檎ｵｱ蜷亥ｾ碁�・ｺ上〒豎ｺ縺ｾ繧九％縺ｨ繧呈・險倥＠縺溘�・
  - `doc/2.1spec/modules.md` / `syntax.md` / `platform.md` 縺ｫ `#if <cond_expr>:` 繧・2.1 縺ｮ豁｣隕上・蜑咲ｽｮ繝・ぅ繝ｬ繧ｯ繝・ぅ繝悶→縺励※霑ｽ蜉�縺励�∵立 `#if[target=...]` 隗呈峡蠑ｧ險俶ｳ輔ｒ 2.0 邉ｻ陦ｨ險倥→縺励※騾�縺代◆縲・
  - `doc/2.1spec/traits.md` 縺ｮ `merge` 萓九°繧我ｸ崎ｦ√↑ `.K` / `.V` 縺ｨ辟｡髢｢菫ゅ↑蛻ｶ邏・ｒ髯､蜴ｻ縺励�，oherence 驕募渚縺ｨ bare 蜷肴尠譏ｧ諤ｧ縺ｮ謇ｱ縺・ｒ蛻・屬縺励◆縲・
  - `doc/2.1spec/traits.md` 縺ｨ `doc/2.1spec/stdlib.md` 縺ｮ讓呎ｺ・trait 荳�隕ｧ繧呈純縺医�～Add .U .R`縲！/O 邉ｻ trait縲∥llocator 邉ｻ trait 繧貞・騾壼喧縺励�～RegionOwned` / `MemReadable` / `MemWritable` 縺ｯ蟆・擂蟆主・縺ｧ縺ゅｋ縺薙→繧呈・險倥＠縺溘�・
  - `doc/2.1spec/errors.md` 縺ｫ `Diags` 繧・`Diag` 縺ｮ蛻励ｒ陦ｨ縺呵｣懷勧蝙九□縺ｨ霑ｽ險倥＠縺溘�・
  - `doc/2.1spec/index.md` 縺ｨ `doc/README.md` 縺ｮ隱ｬ譏弱ｒ縲∝㍾邨先ｸ医∩繧ｳ繧｢莉墓ｧ倥→ draft / 蟆・擂莉墓ｧ倥・蜻ｨ霎ｺ鬆伜沺縺御ｽｵ蟄倥☆繧狗樟蝨ｨ縺ｮ謨ｴ逅・↓蜷医ｏ縺帙※陬懈ｭ｣縺励◆縲・
- [plan.md縺ｨ縺ｮ蟾ｮ逡ｰ/縺評:
  - `plan.md` 縺ｮ逶ｮ讓呵・菴薙↓螟画峩縺ｯ縺ｪ縺・�・
  - 莉墓ｧ俶嶌鄒､縺ｮ繧ｹ繝・・繧ｿ繧ｹ陦ｨ遉ｺ縺ｨ遶�髢灘盾辣ｧ繧呈紛逅・＠縲～2.1spec` 繧定ｪｭ繧�縺ｨ縺阪↓縲後←縺薙′蜃咲ｵ先ｸ医∩縺ｧ縲√←縺薙′蟆・擂莉墓ｧ倥°縲阪′霑ｽ縺・ｄ縺吶￥縺ｪ縺｣縺溘�・
# 2026-04-02 Web Playground editor 蜀埼幕逋ｺ險育判菴懈・

- [逶ｮ逧Ь:
  - Web Playground 縺ｮ editor 繧貞�ｴ蠖薙◆繧顔噪縺ｫ菫ｮ豁｣縺吶ｋ縺ｮ縺ｧ縺ｯ縺ｪ縺上�”ighlight / problems / hover / key input 繧貞性繧√※雋ｬ蜍吝・蜑ｲ縺九ｉ繧・ｊ逶ｴ縺吶◆繧√�∫樟迥ｶ隱ｿ譟ｻ縺ｨ蜀埼幕逋ｺ險育判繧呈紛逅・＠縺溘�・- [迴ｾ迥ｶ遒ｺ隱江:
  - `web/src/editor/editor.ts` 縺ｮ `CanvasEditor` 縺・text state, cursor/selection, undo/redo, folding, language provider 騾｣謳ｺ, Problems 譖ｴ譁ｰ縺ｾ縺ｧ謚ｱ縺医※縺翫ｊ縲∝・蜉帙・謠冗判繝ｻ迥ｶ諷九・險�隱樊ｩ溯・縺悟ｯ・ｵ仙粋縺ｫ縺ｪ縺｣縺ｦ縺・ｋ縲・  - `web/src/editor/editor-input-handler.ts` 縺・DOM event 縺ｨ editor state 譖ｴ譁ｰ繧堤峩邨舌＠縺ｦ縺・ｋ縺溘ａ縲《hortcut 繧・key input 縺ｮ繝・せ繝医ｒ CLI 縺�縺代〒蜀咲樟縺ｧ縺阪↑縺・�・  - `web/src/language/neplg2/neplg2-provider.ts` 縺・`window.wasmBindings` 縺ｫ逶ｴ邨舌＠縺､縺､縲”ighlight / hover / definition / completion / indentation / comment toggle 繧・1 繝輔ぃ繧､繝ｫ縺ｫ謚ｱ縺医※縺・ｋ縲・  - `nepl-web/src/lib.rs` 縺ｫ縺ｯ `analyze_semantics`, `analyze_semantics_with_vfs`, `analyze_name_resolution` 縺ｪ縺ｩ editor 蜀埼幕逋ｺ縺ｫ蠢・ｦ√↑隗｣譫・API 縺梧純縺｣縺ｦ縺・ｋ荳�譁ｹ縲‘ditor 蛛ｴ縺ｫ UI 髱樔ｾ晏ｭ倥・豁｣隕丞喧螻､縺後↑縺・�・  - `nodesrc/compiler_loader.js` 縺ｧ Trunk 謌先棡迚ｩ繧・Node.js 縺九ｉ隱ｭ縺ｿ霎ｼ繧√ｋ縺ｮ縺ｧ縲｜rowser 縺ｪ縺励〒 CLI 縺九ｉ editor 隗｣譫舌ユ繧ｹ繝医ｒ蝗槭☆蟆守ｷ壹・譌｢縺ｫ縺ゅｋ縲・- [plan.md縺ｨ縺ｮ蟾ｮ蛻・:
  - `plan.md` 縺ｫ縺ｯ playground editor 蜀崎ｨｭ險医・蜈ｷ菴楢ｨ育判繧・CLI 螳檎ｵ舌ユ繧ｹ繝域婿驥昴・縺ｾ縺�謨ｴ逅・＆繧後※縺・↑縺・�・  - 螟画峩謠先｡医→縺励※縲‘ditor 繧・pure 縺ｪ core/reducer 縺ｨ browser adapter 縺ｫ蛻・屬縺励�∬ｧ｣譫千ｵ先棡縺ｮ豁｣隕丞喧螻､繧定ｨｭ縺代ｋ險育判繧・`doc/web_playground_editor_redevelopment_plan.md` 縺ｫ險倬鹸縺励◆縲・  - 縺昴・蠕後・隕狗峩縺励〒縲〉epository 謖・､ｺ縺ｫ縺ゅｋ縲形trunk build` 蠕後↓ `nodesrc/cli.js` 縺ｮ繝・せ繝医ｒ螳溯｡後＠縲｛utput 縺ｮ JSON 繧堤｢ｺ隱阪☆繧九％縺ｨ縲阪ｒ貅�縺溘☆縺ｫ縺ｯ縲∝ｰら畑 runner 縺�縺代〒縺ｪ縺・`nodesrc/cli.js` 邨檎罰縺ｮ豁｣蠑丞ｰ守ｷ壹′蠢・ｦ√□縺ｨ蛻・°縺｣縺溘◆繧√�∬ｨ育判譖ｸ縺ｸ霑ｽ險倥＠縺溘�・- [霑ｽ蜉�縺励◆譁・嶌]:
  - `doc/web_playground_editor_redevelopment_plan.md` 繧定ｿｽ蜉�縺励�∫樟迥ｶ縺ｮ蝠城｡檎せ縲∵�ｹ譛ｬ蜴溷屏縲∬ｲｬ蜍吝・蜑ｲ譯医�”over/problems/highlight 縺ｮ蜀崎ｨｭ險域婿驥昴�，LI 螳檎ｵ舌ユ繧ｹ繝郁ｨ育判縲∵ｮｵ髫守噪縺ｪ螳溯｣・ヵ繧ｧ繝ｼ繧ｺ繧定ｨ倩ｿｰ縺励◆縲・  - `doc/README.md` 縺九ｉ譁ｰ縺励＞險育判譖ｸ縺ｸ霎ｿ繧後ｋ繧医≧縺ｫ繝ｪ繝ｳ繧ｯ繧定ｿｽ蜉�縺励◆縲・- [莉雁ｾ後・螳溯｣・ｫ也せ]:
  - `editor-core` 繧呈眠險ｭ縺励※ command/state/reducer/keymap/view-model 繧・pure 縺ｫ蛻・ｊ蜃ｺ縺吶�・  - `neplg2-provider` 繧定ｧ｣譫仙他縺ｳ蜃ｺ縺怜ｱ､縺ｨ hover/problems/highlight/navigation 逕滓・螻､縺ｸ蛻・牡縺吶ｋ縲・  - `nodesrc/playground_editor_test_runner.js` 縺ｯ荳倶ｽ・runner 縺ｨ縺励�∝ｮ御ｺ・｢ｺ隱阪→ CI 縺ｯ `nodesrc/cli.js` 邨檎罰縺ｮ JSON 蜃ｺ蜉帙↓邨ｱ荳�縺吶ｋ縲・  - `doc/testing.md` 縺ｨ `doc/web_playground.md` 繧ゅ�∝ｮ溯｣・ｮｵ髫弱〒縺ｯ playground editor 縺ｮ豁｣蠑乗､懆ｨｼ謇矩�・↓蜷医ｏ縺帙※譖ｴ譁ｰ蟇ｾ雎｡縺ｫ蜷ｫ繧√ｋ縲・  - 蜀阪Ξ繝薙Η繝ｼ縺ｮ邨先棡縲∵里蟄・editor 繧剃ｸ�豌励↓鄂ｮ縺肴鋤縺医ｋ險育判縺�縺ｨ縲御ｸ榊ｿ・ｦ√↑螟画峩繧貞刈縺医↑縺・�阪�悟ｰ上＆縺丞・蜑ｲ縺励※騾ｲ繧√ｋ縲阪�慶ommit 蜑阪↓繝・せ繝育｢ｺ隱阪�阪→縺・≧謖・､ｺ縺ｫ蜿阪＠繧・☆縺・→蛻・°縺｣縺溘◆繧√�∬ｨ育判譖ｸ縺ｸ谿ｵ髫守ｧｻ陦後→ commit/checkpoint 縺ｮ蛻ｶ邏・ｒ霑ｽ險倥＠縺溘�・  - fixture 蠖｢蠑上ｂ `source.nepl` / `vfs.json` / `commands.json` / `expected.json` 縺ｫ蝗ｺ螳壹＠縲．OM event 縺ｧ縺ｯ縺ｪ縺・editor core command 繧・CLI 縺九ｉ蜀咲樟縺吶ｋ譁ｹ驥昴ｒ譏取枚蛹悶＠縺溘�・# 2026-04-02 螳溯｣・Γ繝｢ (playground editor 螳溯｣・幕蟋・

- [莉雁屓逹�謇九＠縺溘％縺ｨ]:
  - `web/src/editor-core/` 繧定ｿｽ蜉�縺励�‘ditor state 縺ｮ譛�蟆丞腰菴阪→縺励※ `types.ts`, `state.ts`, `reducer.ts`, `keymap.ts`, `bridge.ts` 繧剃ｽ懈・縺励◆縲・  - 迴ｾ谿ｵ髫弱〒縺ｯ `select_all`, `toggle_overwrite`, `undo`, `redo`, `set_cursor`, `set_selection`, `replace_text`, `record_history` 繧・pure command 縺ｨ縺励※謇ｱ縺医ｋ縲・  - `web/src/main.ts` 縺九ｉ bridge 繧定ｪｭ縺ｿ霎ｼ縺ｿ縲∵里蟄・`CanvasEditor` / `EditorInputHandler` 縺九ｉ core keymap 繧堤ｵ檎罰縺励※ shortcut 繧貞・逅・☆繧区怙蛻昴・邨ｱ蜷医ｒ蜈･繧後◆縲・  - `nodesrc/playground_editor_test_runner.js` 繧定ｿｽ蜉�縺励�～tests/playground_editor/basic_shortcuts/` fixture 繧堤畑縺・◆ CLI snapshot 繝・せ繝医・譛�蟆丞ｰ守ｷ壹ｒ菴懈・縺励◆縲・- [遒ｺ隱阪〒縺阪◆縺薙→]:
  - `npm --prefix web run build:ts` 縺ｯ騾夐℃縺励◆縲・  - `node nodesrc/playground_editor_test_runner.js --case tests/playground_editor/basic_shortcuts` 縺ｯ騾夐℃縺励�～expected.json` 縺ｨ縺ｮ荳�閾ｴ遒ｺ隱阪∪縺ｧ縺ｧ縺阪ｋ迥ｶ諷九↓縺励◆縲・  - 迴ｾ蝨ｨ縺ｮ runner 縺ｯ `web/dist_ts/editor-core/bridge.js` 繧堤峩謗･隱ｭ繧�荳倶ｽ・runner 縺ｧ縺ゅｊ縲∬ｨ育判縺ｩ縺翫ｊ譛�邨ら噪縺ｪ豁｣蠑丞ｰ守ｷ壹・ `nodesrc/cli.js` 蛛ｴ縺ｸ蟇・○繧句ｿ・ｦ√′縺ゅｋ縲・- [莉雁屓隕九∴縺溷ｷｮ蛻・・譛ｪ隗｣豎ｺ]:
  - 譌｢蟄・browser editor 縺ｮ state 譖ｴ譁ｰ縺ｯ縺ｾ縺� `CanvasEditor` 蛛ｴ縺ｫ螟ｧ縺阪￥谿九▲縺ｦ縺翫ｊ縲…ore 縺ｯ shortcut 縺ｮ蜈･蜿｣縺�縺代ｒ蛻・ｊ蜃ｺ縺励◆谿ｵ髫弱�・  - hover / problems / highlight / definition / completion 縺ｮ豁｣隕丞喧螻､縺ｯ譛ｪ逹�謇九〒縲～neplg2-provider` 縺ｮ雋ｬ蜍吝・髮｢縺ｯ縺薙ｌ縺九ｉ縲・  - `AGENTS.md` 縺ｧ隕∵ｱゅ＆繧後※縺・ｋ `trunk build` 縺ｯ縺薙・迺ｰ蠅・〒縺ｯ `trunk` 繧ｳ繝槭Φ繝芽・菴薙′隕九▽縺九ｉ縺壽悴螳溯｡後�ら腸蠅・紛蛯吶∪縺溘・蟆主・謇矩�・・遒ｺ隱阪′蠢・ｦ√�・  - `nodesrc/cli.js` 縺ｫ縺ｯ playground editor 逕ｨ縺ｮ豁｣蠑上↑ test entry 縺後∪縺�辟｡縺上�∫樟迥ｶ縺ｯ陬懷勧 runner 縺ｮ縺ｿ縲・# 2026-04-02 螳溯｣・Γ繝｢ (playground editor CLI 繝・せ繝亥ｰ守ｷ壹・謨ｴ蛯・

- [莉雁屓騾ｲ繧√◆縺薙→]:
  - `nodesrc/playground_editor_test_runner.js` 繧・library 縺ｨ CLI 縺ｮ荳｡逕ｨ縺ｫ謨ｴ逅・＠縲…ase directory 縺ｮ蜀榊ｸｰ謗｢邏｢縲～keyboard_event` step 縺ｮ隗｣驥医�∥ggregate summary 縺ｮ逕滓・繧定ｿｽ蜉�縺励◆縲・  - `nodesrc/cli.js` 縺ｫ `--playground-editor-tests` 縺ｨ `-o json=...` 縺ｮ豁｣蠑丞ｰ守ｷ壹ｒ霑ｽ蜉�縺励�｝layground editor fixture 繧帝寔邏・ｮ溯｡後＠縺ｦ JSON 繧貞・蜉帙〒縺阪ｋ繧医≧縺ｫ縺励◆縲・  - `nodesrc/cli.js` 縺ｯ襍ｷ蜍墓凾縺ｫ `parser` / `html_gen` / `html_gen_playground` 繧堤┌譚｡莉ｶ縺ｫ require 縺励※縺・◆縺溘ａ縲｝layground editor test 縺ｮ繧医≧縺ｪ辟｡髢｢菫ゅ↑繝｢繝ｼ繝峨〒繧・`parser.ts` 譛ｪ繝薙Ν繝峨〒蜊ｳ豁ｻ縺励※縺・◆縲Ｓoot cause 縺ｯ top-level dependency 縺ｮ驕主臆隱ｭ縺ｿ霎ｼ縺ｿ縺�縺｣縺溘・縺ｧ縲∝ｿ・ｦ∵凾縺ｮ縺ｿ隱ｭ縺ｿ霎ｼ繧� lazy load 縺ｫ螟画峩縺励◆縲・  - `tests/playground_editor/` 縺ｫ `keyboard_shortcuts`, `keyboard_unmapped`, `text_edit_history` 繧定ｿｽ蜉�縺励�《hortcut繝ｻ譛ｪ蟇ｾ蠢・key繝ｻundo/redo 繧・fixture 蛹悶＠縺溘�・  - `doc/testing.md` 縺ｨ `doc/web_playground.md` 縺ｫ playground editor 縺ｮ豁｣蠑上↑ CLI 繝・せ繝域焔鬆・ｒ霑ｽ險倥＠縺溘�・- [遒ｺ隱咲ｵ先棡]:
  - `npm --prefix web run build:ts` 縺ｯ騾夐℃縺励◆縲・  - `node nodesrc/playground_editor_test_runner.js --case tests/playground_editor/basic_shortcuts` 縺ｯ騾夐℃縺励◆縲・  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` 縺ｯ騾夐℃縺励�～4/4 passed` 繧堤｢ｺ隱阪＠縺溘�・  - 蜃ｺ蜉・JSON 縺ｧ縺ｯ `basic_shortcuts`, `keyboard_shortcuts`, `keyboard_unmapped`, `text_edit_history` 縺ｮ蜈ｨ case 縺・`ok: true` 縺ｫ縺ｪ縺｣縺ｦ縺・ｋ縲・- [谿玖ｪｲ鬘珪:
  - editor browser adapter 蛛ｴ縺ｮ state 譖ｴ譁ｰ雋ｬ蜍吶・縺ｾ縺� `CanvasEditor` 縺ｫ螟壹￥谿九▲縺ｦ縺・ｋ縲・  - hover / problems / highlight / definition / completion 縺ｮ豁｣隕丞喧螻､縺ｯ譛ｪ逹�謇九�・  - `AGENTS.md` 縺ｧ隕∵ｱゅ＆繧後※縺・ｋ `trunk build` 縺ｯ縲√％縺ｮ迺ｰ蠅・〒縺ｯ `trunk` 繧ｳ繝槭Φ繝峨′蟄伜惠縺帙★譛ｪ螳溯｡後・縺ｾ縺ｾ縲・# 2026-04-03 螳溯｣・Γ繝｢ (playground editor 蜈･蜿｣鄂ｮ謠・

- [莉雁屓騾ｲ繧√◆縺薙→]:
  - `web/src/editor-core/browser-adapter.ts` 繧定ｿｽ蜉�縺励�『eb playground 縺檎峩謗･菴ｿ縺・眠縺励＞ editor API 縺ｨ縺励※ `PlaygroundEditor` / `createPlaygroundEditor` 繧貞ｮ夂ｾｩ縺励◆縲・  - 譁ｰ API 縺ｯ `setText`, `getText`, `setPath`, `getPath`, `focus`, `resizeEditor`, `setFontSize`, `showPopup`, `getCursorPosition`, `getTokenInsight` 繧呈署萓帙＠縲∵立 `CanvasEditor` 縺ｮ蜀・Κ隧ｳ邏ｰ繧・main 蛛ｴ縺九ｉ髫�縺吝ｽ｢縺ｫ縺励◆縲・  - `web/src/main.ts` 縺ｯ `CanvasEditorLibrary.createCanvasEditor(...)` 繧偵ｄ繧√※ `createPlaygroundEditor(...)` 繧剃ｽｿ縺・ｈ縺・↓螟画峩縺励◆縲ゅ％繧後↓繧医ｊ web playground 譛ｬ菴薙・ editor 蜈･蜿｣縺ｯ譁ｰ API 蛛ｴ縺ｸ鄂ｮ縺肴鋤繧上▲縺溘�・  - `web/src/library/tabs.ts` 縺ｨ `web/src/terminal/shell.ts` 繧・`path` 逶ｴ蜿ら・繧偵ｄ繧√�～getPath` / `setPath` 繧貞━蜈医＠縺ｦ菴ｿ縺・ｈ縺・↓螟画峩縺励◆縲・  - 莠呈鋤邨瑚ｷｯ縺ｨ縺励※ `web/src/library/canvas-editor-lib.ts` 繧・`window.PlaygroundEditorFactory` 縺後≠繧後・譁ｰ API 繧定ｿ斐☆繧医≧縺ｫ縺励◆縲・- [遒ｺ隱咲ｵ先棡]:
  - `npm --prefix web run build:ts` 縺ｯ騾夐℃縺励◆縲・  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` 縺ｯ蠑輔″邯壹″ `4/4 passed`縲・- [迴ｾ迥ｶ隱崎ｭ肋:
  - web playground 縺ｮ襍ｷ蜍募・蜿｣縺ｯ譁ｰ API 縺ｫ鄂ｮ縺肴鋤繧上▲縺溘′縲｜rowser adapter 縺ｮ蜀・Κ縺ｧ縺ｯ縺ｾ縺� `CanvasEditor` / renderer / input handler / DOM UI 繧貞・蛻ｩ逕ｨ縺励※縺・ｋ縲・  - 縺励◆縺後▲縺ｦ縲継layground 荳翫〒菴ｿ繧上ｌ繧・editor API 縺ｮ鄂ｮ謠帙�阪・縺ｧ縺阪◆縺後�√�悟・驛ｨ雋ｬ蜍吶・蜈ｨ髱｢蛻ｷ譁ｰ縲阪・譛ｪ螳御ｺ・�・
# 2026-04-03 螳溯｣・Γ繝｢ (playground editor analysis 螻､縺ｨ CLI 諡｡蠑ｵ)

- [莉雁屓縺ｮ螳溯｣・:
  - `web/src/editor-core/language-analysis.ts` 繧定ｿｽ蜉�縺励�～neplg2-provider` 縺ｮ highlight / problems / folding / semanticTokens / inlayHints / hover / definition / occurrences 繧・pure 縺ｪ蛻・梵螟画鋤螻､縺ｸ蛻・ｊ蜃ｺ縺励◆縲・  - `web/src/language/neplg2/neplg2-provider.ts` 縺ｯ WASM 縺ｮ逕・payload 繧剃ｿ晄戟縺励▽縺､縲‘ditor 蜷代￠ update payload 縺ｨ蜷・ｨｮ query 繧・`NEPLPlaygroundLanguageAnalysis` 縺ｸ蟋碑ｭｲ縺吶ｋ繧医≧縺ｫ螟画峩縺励◆縲・  - `web/src/editor-core/browser-adapter.ts` 縺ｫ `getHoverInfo`, `getDefinitionLocation`, `getOccurrences`, `getProblems`, `getHighlightSnapshot` 繧定ｿｽ蜉�縺励�『eb playground 蛛ｴ縺梧眠 API 縺九ｉ蛻・梵邨先棡繧呈桶縺医ｋ蜈･蜿｣繧呈純縺医◆縲・  - `nodesrc/playground_editor_test_runner.js` 繧呈僑蠑ｵ縺励�∝ｾ捺擂縺ｮ `commands.json` fixture 縺ｫ蜉�縺医※ `analysis.json` + `requests.json` fixture 繧貞ｮ溯｡後〒縺阪ｋ繧医≧縺ｫ縺励◆縲・  - `tests/playground_editor/analysis_payload_basic` 縺ｨ `tests/playground_editor/analysis_hover_definition` 繧定ｿｽ蜉�縺励�”ighlight payload縲‥iagnostics縲’olding縲（nlay hints縲”over縲‥efinition縲｛ccurrences 繧・CLI snapshot 縺ｧ蝗ｺ螳壼喧縺励◆縲・- [遒ｺ隱咲ｵ先棡]:
  - `npm --prefix web run build:ts` 縺ｯ騾夐℃縺励◆縲・  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` 縺ｯ `6/6 passed` 縺ｫ縺ｪ繧翫�〔eyboard/state 邉ｻ 4 case 縺ｨ analysis 邉ｻ 2 case 繧・JSON 縺ｧ遒ｺ隱阪＠縺溘�・- [plan.md 縺ｨ縺ｮ蟾ｮ蛻・:
  - browser adapter 縺ｮ蜈･蜿｣鄂ｮ謠帙→ analysis 豁｣隕丞喧螻､縲，LI 縺ｧ縺ｮ hover/problems/highlight 邉ｻ snapshot 縺ｾ縺ｧ縺ｯ蜈･縺｣縺溘�・  - 縺溘□縺怜・驛ｨ縺ｧ縺ｯ縺ｾ縺� `CanvasEditor` / renderer / input handler / DOM UI 繧貞・蛻ｩ逕ｨ縺励※縺翫ｊ縲∵緒逕ｻ縺ｨ state 譖ｴ譁ｰ雋ｬ蜍吶・螳悟・謦､蜴ｻ縺ｾ縺ｧ縺ｯ譛ｪ螳御ｺ・�・  - `AGENTS.md` 縺ｧ豎ゅａ繧峨ｌ縺ｦ縺・ｋ `trunk build` 縺ｯ縲√％縺ｮ迺ｰ蠅・〒縺ｯ `trunk` 繧ｳ繝槭Φ繝峨′蟄伜惠縺帙★譛ｪ螳溯｡後・縺ｾ縺ｾ縲ゅ％縺薙・迺ｰ蠅・紛蛯吶′谿九ち繧ｹ繧ｯ縲・
# 2026-04-03 螳溯｣・Γ繝｢ (playground editor 蜈･蜉・state 縺ｮ core 蛹・

- [莉雁屓縺ｮ螳溯｣・:
  - `web/src/editor-core/reducer.ts` 縺ｫ `insert_text`, `delete_backward`, `delete_forward` 繧定ｿｽ蜉�縺励�∵枚蟄怜・蜉帙→蜑企勁縺ｮ text / selection / undo 譖ｴ譁ｰ繧・pure reducer 蛛ｴ縺ｧ謇ｱ縺医ｋ繧医≧縺ｫ縺励◆縲・  - `web/src/editor/editor.ts` 縺ｯ core runtime state 繧・editor 螳滉ｽ薙∈蜿肴丐縺吶ｋ `applyCoreRuntimeState` 繧呈戟縺､繧医≧縺ｫ縺励�～applyCoreStateCommand` 縺ｯ蛟句挨蛻・ｲ舌〒縺ｯ縺ｪ縺・reducer 縺ｮ邨先棡繧帝←逕ｨ縺吶ｋ蠖｢縺ｸ蟇・○縺溘�・  - `web/src/editor/editor-input-handler.ts` 縺ｯ `input`, `Backspace`, `Delete` 繧偵∪縺・core command 縺ｧ蜃ｦ逅・＠縲∵立蜃ｦ逅・・ fallback 縺ｫ荳九￡縺溘�・  - `tests/playground_editor/core_text_input` 縺ｨ `tests/playground_editor/core_delete_selection` 繧定ｿｽ蜉�縺励�（nsert/backspace/delete 縺ｨ驕ｸ謚槫炎髯､縺ｮ history 繧・CLI fixture 縺ｧ蝗ｺ螳壼喧縺励◆縲・- [遒ｺ隱咲ｵ先棡]:
  - `npm --prefix web run build:ts` 縺ｯ騾夐℃縺励◆縲・  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` 縺ｯ `8/8 passed` 縺ｫ縺ｪ繧翫�∥nalysis 2 case縲《hortcut/state 4 case縲》ext edit 2 case 繧堤｢ｺ隱阪＠縺溘�・- [迴ｾ迥ｶ隱崎ｭ肋:
  - editor 縺ｮ蜈･蜉・state 縺ｯ荳�驛ｨ pure core 縺ｸ遘ｻ縺｣縺溘′縲｝ointer 謫堺ｽ懊�∬｡檎ｧｻ蜍輔�《croll縲∵緒逕ｻ縲…ompletion UI 縺ｯ縺ｾ縺� `CanvasEditor` 蛛ｴ縺ｮ雋ｬ蜍吶′螟ｧ縺阪＞縲・  - 縺昴・縺溘ａ縲∝・髱｢鄂ｮ謠帛ｮ御ｺ・↓縺ｯ縺ｾ縺�驕斐＠縺ｦ縺・↑縺・�ら音縺ｫ renderer / DOM UI / pointer 縺ｾ繧上ｊ縺ｮ蛻・屬縺梧ｮ九▲縺ｦ縺・ｋ縲・
# 2026-04-03 実装メモ (playground editor 左右移動の core 化)

- [今回の実装]:
  - `web/src/editor-core/types.ts` / `reducer.ts` に `move_cursor` を追加し、左右移動と shift 選択の更新を pure reducer で扱えるようにした。
  - `web/src/editor/editor-input-handler.ts` は `ArrowLeft` / `ArrowRight` の非 ctrl 系をまず core command で処理するように変更した。
  - `tests/playground_editor/core_cursor_move` を追加し、左右移動と選択解除の snapshot を固定化した。
- [確認結果]:
  - `npm --prefix web run build:ts` は通過した。
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json` は `9/9 passed`。
- [現状認識]:
  - 左右移動は core 側へ寄ったが、上下移動、Home/End、PageUp/PageDown、pointer drag、scroll、fold click、completion UI はまだ旧 editor 実装が主体。
  - このため、全面置換完了や WSL git commit の条件にはまだ届いていない。
# 2026-04-03 メモ (web playground editor surface の根本修正)

- [状況]:
  - hover は出るのに入力・削除が効かず、syntax highlight もずれる問題が surface 側に残っていたため、core 接続と改行処理を見直した。
- [原因]:
  - `CanvasEditor.getCoreState()` が core reducer に渡すべき runtime state ではなく `snapshotEditorRuntimeState(...)` の結果を返していた。
  - そのため `insert_text` / `delete_backward` / `delete_forward` などの command が surface 実行時だけ不正な state を受け取り、編集系操作が壊れていた。
  - 描画側は改行を実質 `LF` 前提で扱っているのに、tab / editor の境界では `CRLF` をそのまま持ち込めたため、token / diagnostic の index と描画列の対応がずれやすかった。
- [修正]:
  - `web/src/editor/editor.ts`
    - core に渡す state を snapshot ではなく runtime state に変更した。
    - editor 内部テキストを `LF` に正規化する `normalizeEditorText` を追加し、`setText` / `applyCoreRuntimeState` / `insertText` / `updateLines` / `updateText` を統一した。
  - `web/src/library/tabs.ts`
    - tab 読み込み・比較・保存でも `LF` 正規化を行うようにして、surface 全体で改行規約を揃えた。
  - `note.n.md` / `todo.md` と今回触った editor 関連ファイルは UTF-8 に変換した。
- [確認]:
  - `npm --prefix web run build:ts`: 通過
  - `trunk build --release --public-url ./`: 通過
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`: `11/11 passed`
- [plan.mdとの差分]:
  - 新 editor への全面移行前に、既存 `CanvasEditor` surface と new core の境界で壊れていた編集経路を先に修正した。これで surface から core を安全に使える土台ができた。
# 2026-04-03 メモ (web playground editor 解析更新の増分高速化)

- [状況]:
  - editor は動作するようになったが、1 ストロークごとに highlight 更新が重く、入力体感が悪かったため、surface から language provider までの更新経路を見直して増分処理を入れた。
- [原因]:
  - `CanvasEditor.applyCoreRuntimeState()` がカーソル移動のような非編集 command でも毎回 `updateText(this.text)` を呼んでおり、テキスト不変でも解析が走っていた。
  - `NEPLg2LanguageProvider.updateText()` は毎回全文再解析を予約し、前回解析結果を一切再利用していなかった。
- [修正]:
  - `web/src/editor/editor.ts`
    - `applyCoreRuntimeState()` でテキストが変化したときだけ `updateText()` を呼ぶように変更した。
  - `web/src/language/neplg2/neplg2-provider.ts`
    - 同一テキスト更新を no-op にした。
    - 前回 payload と新旧テキスト差分から provisional な token / diagnostic / folding / semantic payload を組み立てる増分更新を追加した。
    - 完全解析は debounce 後に `requestIdleCallback` 優先で流し、入力直後の main thread 負荷を下げた。
    - 差分範囲外の token などは位置シフトで再利用し、変更行付近だけ軽量 tokenizer で暫定再構築するようにした。
- [確認]:
  - `npm --prefix web run build:ts`: 通過
  - `trunk build --release`: 通過
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`: `11/11 passed`
# 2026-04-03 メモ (web playground editor hover 表示変更)

- [状況]:
  - hover では token 名寄りの情報を出していたが、式全体の把握に向かないため、token を含む式の抜き出しと型情報を優先する形に変更した。
- [修正]:
  - `web/src/editor-core/language-analysis.ts`
    - `exprSpan` から実際のソース断片を抜き出して整形する helper を追加した。
    - hover 内容を `expr: <source snippet>` と `type: <inferred type>` を先頭に出す形へ変更した。
    - token 単体文字列は、式断片と同一でないときだけ補助情報として出すようにした。
  - `tests/playground_editor/analysis_hover_definition/expected.json`
    - hover の期待値を新形式へ更新した。
- [確認]:
  - `npm --prefix web run build:ts`: 通過
  - `trunk build --release`: 通過
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`: `11/11 passed`
# 2026-04-08 Playground editability 判定修正

- 現象:
  - playground で「どのファイルが編集可能か」の判定が surface 側に存在せず、VFS・tab・editor input・shell sync の判断が分離していた。
  - そのため、read-only にしたいファイルでも通常入力や一部ショートカット経由で編集経路に入れる余地があり、逆に bundled file 全体を read-only 扱いすると examples まで編集不能になって playground の主用途を壊す状態だった。
- 原因:
  - 編集可能性の source of truth が無く、`CanvasEditor` は常に editable、`TabManager` は常に保存可能、`VFS` は常に書き込み可能という前提だった。
  - 初期 mount 時に file attribute が付与されていなかったため、bundled stdlib / README / examples の区別も無かった。
- 修正:
  - `web/src/runtime/vfs.ts` に read-only file 管理を追加し、`isEditable()` を VFS 起点で判定するようにした。
  - `web/src/main.ts` で mount helper を追加し、`/stdlib/**` と `/README` は read-only、`/examples/**` は editable として初期化するようにした。
  - `web/src/library/tabs.ts` で tab ごとに `isEditable` を保持し、tab 切替時に editor surface へ伝播、read-only tab では保存を抑止するようにした。
  - `web/src/editor/editor.ts` / `web/src/editor-core/browser-adapter.ts` に `setEditable()` / `getEditable()` を追加した。
  - `web/src/editor/editor-input-handler.ts` で paste / cut / input / Enter / Backspace / Delete / Tab / printable key / `Ctrl+/` を read-only 時に停止するようにした。
  - `web/src/terminal/shell.ts` で read-only editor view を実行時同期の対象から外し、VFS へ誤って書き戻さないようにした。
  - `nodesrc/playground_editability_test_runner.js` を追加し、read-only 判定と tab 保存挙動を CLI で回帰確認できるようにした。
- 確認:
  - `npm --prefix web run build:ts`
  - `node nodesrc/playground_editability_test_runner.js`
  - `node nodesrc/playground_editor_surface_test_runner.js`
  - `trunk build --release`
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`
  - `tmp/playground-editor-tests.json`: `caseCount=12`, `passedCount=12`, `failedCount=0`
- plan.md との差異:
  - plan にある editor 全面再設計のうち、今回は editability 判定の一貫化に限定して修正した。
  - examples を read-only にはせず editable のまま維持している。これは playground の初期編集対象を壊さないための判断。
# 2026-04-08 Playground panel workspace 再設計

- 現状:
  - Web playground の固定 3 分割 DOM を廃止し、split tree ベースの panel workspace に置き換えた。
  - Explorer / Editor / Terminal はすべて leaf panel として workspace tree に参加し、split ratio と focused panel を localStorage に保存・復元するようにした。
  - focused editor panel に対して file open / run / compile / help / save を解決するように main 側の導線を切り替えた。
  - editor panel は panel-local の tab state を持ち、workspace snapshot には editor ごとの paths / activePath を保存する。
  - split handle による比率変更、panel split right / split down / close、drag and drop による panel move を実装した。
  - center drop は editor panel 同士の tab merge にだけ対応し、Explorer の複製は禁止、最後の editor / explorer panel は close できないように保護している。
- 実装:
  - `web/src/workspace/panel-layout.ts`
    - `WorkspaceNode`, `SplitNodeSnapshot`, `LeafPanelSnapshot`, `WorkspaceSnapshot` を追加した。
    - split / close / move / normalize / restore の pure state 操作を分離し、panel manager から再利用できるようにした。
  - `web/src/workspace/panel-manager.ts`
    - split tree の DOM 描画、focused panel 管理、workspace restore/save、toolbar 対象解決、panel drag/drop、split resize をまとめる manager を追加した。
    - editor / terminal / explorer を runtime map で保持し、workspace redraw 後も leaf id 単位で再利用するようにした。
  - `web/src/library/tabs.ts`
    - editor panel ごとに tab state を持てるよう `restoreTabs`, `mergeFrom`, `getTabSnapshot`, `onStateChange` を追加した。
  - `web/src/main.ts`
    - 旧 resizer ベースの固定 pane 初期化をやめ、workspace root と `PlaygroundPanelManager` を使う構成へ切り替えた。
    - open / run / compile / help / clear / stop は focused panel 解決経路から実行するようにした。
  - `web/index.html` / `web/styles.css`
    - 固定 `explorer-pane` / `editor-pane` / `terminal-pane` を除去し、panel shell / tab bar / split handle を持つ workspace DOM と styles に更新した。
  - `nodesrc/playground_workspace_test_runner.js`
    - workspace snapshot の split / move / close / restore を browser なしで確認する CLI runner を追加した。
- 確認:
  - `npm --prefix web run build:ts`
  - `node nodesrc/playground_workspace_test_runner.js`
  - `node nodesrc/playground_editability_test_runner.js`
  - `node nodesrc/playground_editor_surface_test_runner.js`
  - `trunk build --release`
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`
  - `tmp/playground-editor-tests.json`: `caseCount=12`, `passedCount=12`, `failedCount=0`
- plan.md との差分:
  - split tree / localStorage restore / panel-local tab state / focused panel 解決 / split handle / drag and drop の骨格は実装した。
  - plan では terminal process を 1 本に維持し、複数 terminal panel は同一実行基盤の別 view とする方針だったが、現状は terminal panel を増やした場合に `CanvasTerminal` / `Shell` も panel ごとに独立して作成している。
  - center drop の合流は editor panel 同士の tab merge のみ対応で、terminal panel 同士の center merge と shared terminal session は未実装である。
  - mobile 縮退 layout は panel shell を縦積みにする簡易対応までで、drag/drop を touch 向けに最適化するところまでは未着手である。
# 2026-04-09 Playground explorer / focus / tab editing 修正

- 現状:
  - workspace 化後の playground で、explorer に file / folder の視覚的な区別が弱く、panel 切替時の editor / terminal cursor blink も実フォーカスと完全には揃っていなかった。
  - さらに editor tab 切替時に、tabbar click が editor panel 外扱いで blur される経路と、tab 切替時の focus / save 同期不足が重なり、2 個目以降の tab で編集しづらい状態になっていた。
- 実装:
  - `web/src/library/explorer.ts`
    - explorer item を disclosure / icon / label の 3 要素構成に変更し、folder open/close と file を class で描き分けるようにした。
  - `web/styles.css`
    - explorer 用の disclosure icon、folder icon、file icon、label overflow を追加し、開閉状態が見える見た目にした。
  - `web/src/editor/editor.ts`
    - editor focus state を `setFocusState()` に集約し、cursor blink と popup / completion の表示を実フォーカスに同期するようにした。
  - `web/src/editor/editor-input-handler.ts`
    - outside click 判定を `canvas.parentElement` ではなく editor panel 全体に広げ、tabbar click で editor が誤って blur されないようにした。
    - textarea の native focus / blur を editor state に反映するようにした。
  - `web/src/library/tabs.ts`
    - `setActiveTab()` に current tab 保存と focus 再同期を入れ、editable tab 間の切替で編集状態と path が正しく追従するようにした。
    - `restoreTabs()` や close 後の再選択では不要な focus を避けつつ、通常の tab 切替では editor を再 focus するように分けた。
  - `web/src/terminal/terminal.ts`
    - terminal に `isFocused` を持たせ、cursor blink と cursor 描画を textarea focus にだけ同期するようにした。
    - panel が非 focus になった terminal は blink を止めるようにした。
  - `web/src/workspace/panel-manager.ts`
    - focused leaf 切替時に、非 focus の editor / terminal を明示的に blur するようにして panel 間の cursor 状態を揃えた。
  - `nodesrc/playground_editability_test_runner.js`
    - editable tab を複数開いた状態で tab 切替しても editability が崩れず、切替時に前の tab の内容が保存されることを固定した。
- 確認:
  - `npm --prefix web run build:ts`
  - `node nodesrc/playground_editability_test_runner.js`
  - `node nodesrc/playground_editor_surface_test_runner.js`
  - `node nodesrc/playground_workspace_test_runner.js`
  - `trunk build --release`
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`
  - `tmp/playground-editor-tests.json`: `caseCount=12`, `passedCount=12`, `failedCount=0`
- plan.md との差分:
  - 今回は panel workspace 設計そのものではなく、workspace 化で表面化した explorer 見た目と focus / tab 編集不整合の修正に限定した。
# 2026-04-09 Playground cursor visibility 調整

- 現状:
  - focus を失った panel でも、editor 側では current line の境界線が残って見え、cursor が消え切っていないように見える状態があった。
- 実装:
  - `web/src/editor/editor-renderer.ts`
    - current line の描画も `editor.isFocused` 条件に揃え、focus 中の editor にだけ cursor 系の視覚表現が出るようにした。
- 確認:
  - `npm --prefix web run build:ts`
  - `node nodesrc/playground_editor_surface_test_runner.js`
  - `trunk build --release`
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`
  - `tmp/playground-editor-tests.json`: `caseCount=12`, `passedCount=12`, `failedCount=0`
# 2026-04-09 Playground panel-local zoom

- 現状:
  - `bemstudy` の tab zoom を参考に、playground でも panel-local の zoom を追加した。
  - editor は active tab ごとに独立した zoom を持ち、terminal は panel ごとの zoom を持つ。
  - `Ctrl+wheel`, `Ctrl++`, `Ctrl+-`, `Ctrl+0`, 2 本指 pinch で zoom を変更でき、操作中は panel 右上に倍率バッジをオーバーレイ表示する。
- 実装:
  - `web/src/workspace/panel-layout.ts`
    - workspace snapshot の leaf に `zoom` と `pathZooms` を追加し、normalize 時に zoom state を保持するようにした。
  - `web/src/library/tabs.ts`
    - tab state に `zoom` を追加し、`getTabSnapshot`, `restoreTabs`, `getActiveZoom`, `setActiveZoom` を通じて tab ごとの zoom を保持できるようにした。
  - `web/src/workspace/panel-manager.ts`
    - panel-local zoom の clamp / apply / persist / overlay badge を追加した。
    - `Ctrl+wheel` と keyboard shortcut、touch pinch を focused panel に対して解決し、editor は active tab、terminal は panel 単位で zoom を変えるようにした。
    - runtime 作成時や tab 切替時に zoom を再適用し、workspace restore 後も倍率が戻るようにした。
  - `web/styles.css`
    - zoom 操作時に panel 右上へ出る `panel-zoom-badge` を追加した。
  - `nodesrc/playground_workspace_test_runner.js`
    - leaf zoom state が normalize / clone で失われないことを固定した。
  - `nodesrc/playground_editability_test_runner.js`
    - editable tab ごとの zoom state が tab 切替後も維持されることを固定した。
- 確認:
  - `npm --prefix web run build:ts`
  - `node nodesrc/playground_workspace_test_runner.js`
  - `node nodesrc/playground_editability_test_runner.js`
  - `trunk build --release`
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`
  - `tmp/playground-editor-tests.json`: `caseCount=12`, `passedCount=12`, `failedCount=0`
- plan.md との差分:
  - plan に明示されていた機能ではないが、workspace 化後の panel-local state として zoom を追加した。
  - editor は tab ごとの zoom、terminal は panel ごとの zoom で、Explorer には zoom を持たせていない。
# 2026-04-09 Workspace root sizing 修正

- 現状:
  - panel workspace 化後、editor が panel 領域いっぱいに広がらず、小さく縮んで見えるケースがあった。
- 原因:
  - `#workspace-root` 自体ではなく、その親の `.workspace-shell` が block のままで、子の `.workspace` に付けた `flex: 1` が高さ確保に効いていなかった。
  - その結果、split tree の `height: auto` 連鎖になり、canvas 親要素の `getBoundingClientRect()` が期待より小さくなっていた。
- 実装:
  - `web/styles.css`
    - `.workspace-shell` を flex container に変更した。
    - `.workspace` と `.split-node` に `width: 100%` と `height: 100%` を追加し、panel shell から canvas container まで高さを確実に引き継ぐようにした。
- 確認:
  - `npm --prefix web run build:ts`
  - `trunk build --release`
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`
  - `tmp/playground-editor-tests.json`: `caseCount=12`, `passedCount=12`, `failedCount=0`
# 2026-04-09 Header layout reset button

- 現状:
  - workspace を分割・移動したあと、header からワンクリックで default layout に戻す導線がなかった。
- 実装:
  - `web/index.html`
    - header に `Layout` ボタンを追加した。
  - `web/src/workspace/panel-manager.ts`
    - `resetWorkspaceLayout()` を追加し、saved workspace snapshot を default split tree へ戻して redraw できるようにした。
  - `web/src/main.ts`
    - `Layout` ボタンから reset API を呼び、editor tab が空なら初期ドキュメントを開き、font size と focus を再同期するようにした。
- 確認:
  - `npm --prefix web run build:ts`
  - `trunk build --release`
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`
  - `tmp/playground-editor-tests.json`: `caseCount=12`, `passedCount=12`, `failedCount=0`
# 2026-04-09 メモ (playground terminal の UI blocking 解消)

- [原因]:
  - `web/src/terminal/shell.ts` の `neplg2 build/run` は `window.wasmBindings.compile_outputs_with_vfs(...)` を main thread で直接呼んでおり、重い compile 中に workspace 全体の UI が停止していた。
  - 一方で `wasmi` 実行だけは `web/src/runtime/worker.ts` の worker に分離されていたため、compile と run で stdio / interrupt / 実行状態の責務が分断されていた。
  - この構造だと compile 中は redraw・pointer・focus・terminal input が止まりやすく、長時間処理の途中出力も worker 側に統一できない。
- [実装]:
  - `web/src/runtime/compiler-assets.ts`
    - Trunk が生成する `modulepreload` / `preload` から compiler JS/WASM asset URL を解決する helper を追加した。
  - `web/src/main.ts`
    - 起動時に compiler asset URL を `window.NEPLg2CompilerAssets` として確定させ、terminal/shell が DOM 依存を増やさず worker へ渡せるようにした。
  - `web/src/runtime/worker.ts`
    - 旧 `run` 専用 worker を `run-wasm` と `execute-neplg2` の 2 系統を扱う実行 worker に再設計した。
    - worker 側で compiler module を dynamic import し、`compile_outputs_with_vfs` を worker 内で実行するようにした。
    - compile 結果は `compile_result` として main thread に返し、WASI 実行時の `stdout` / `stdin_request` / `exit` / `error` は従来どおり stream する構造に揃えた。
  - `web/src/terminal/shell.ts`
    - compile / run / wasmi をすべて worker process protocol に統一した。
    - main thread では VFS 同期、compile output の保存、terminal 描画だけを担当し、重い処理は worker に閉じ込めた。
    - `interrupt()`、SharedArrayBuffer stdin、`isRunning` は compile/run 共通で機能するように整理した。
  - `nodesrc/playground_shell_worker_test_runner.js`
    - compiler asset 解決、worker compile protocol 使用、compile output の VFS 反映、worker stdout stream を headless に確認する runner を追加した。
- [確認]:
  - `npm --prefix web run build:ts`: 通過
  - `node nodesrc/playground_shell_worker_test_runner.js`: 通過
  - `node nodesrc/playground_editor_surface_test_runner.js`: 通過
  - `node nodesrc/playground_workspace_test_runner.js`: 通過
  - `node nodesrc/playground_tab_transfer_test_runner.js`: 通過
  - `node nodesrc/playground_drag_drop_test_runner.js`: 通過
  - `node nodesrc/playground_editability_test_runner.js`: 通過
  - `trunk build --release`: 通過
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`: `caseCount=12`, `passedCount=12`, `failedCount=0`
- [plan.mdとの差分]:
  - 今回はまず compile/run の main-thread blocking を止めることを優先し、worker protocol は `compile_result` / `stdout` / `stdin_request` / `exit` / `error` まで実装した。
  - progress の細粒度通知や stderr 専用の UI 表示、複数 terminal panel 間の shared backend 化は未着手で、今後の todo に残している。
# 2026-04-09 メモ (directive / import ハイライト欠落の修正)

- [原因]:
  - `web/src/editor-core/language-analysis.ts` と `web/src/language/neplg2/neplg2-provider.ts` の token 正規化で、`DirEntry` / `DirTarget` / `DirImport` などの directive token が `keyword` ではなく `default` に落ちていた。
  - `#import "..." as *` の `as` も `Ident` として `variable` 扱いになっており、directive 行が全体としてほぼ無彩色に見えていた。
- [追加調査]:
  - 実際の `analyze_semantics()` 出力を確認すると、directive は `#entry` や `#import` だけではなく「行全体を 1 token」として返していた。
  - そのため `DirImport` を単に `keyword` にしても、`"core/math"` や `as *` を別色にできず、期待する見た目にはならなかった。
- [実装]:
  - directive token (`Dir*`) を `keyword` として扱うようにした。
  - `as` と `pub` は `Ident` でも文法キーワードとして `keyword` に寄せた。
  - `Ampersand` を `operator`、`UnitLiteral` を `punctuation` として補強した。
  - directive token は surface 用に source 行から再分解し、`#import` / string literal / `as` / `*` を個別 token として描画側へ渡すようにした。
  - `tests/playground_editor/analysis_directives_imports/` を追加して、`#entry` / `#target` / `#import` / import path string / `as` / `*` の色分けを formal CLI suite に固定した。
- [確認]:
  - `npm --prefix web run build:ts`: 通過
  - `trunk build --release`: 通過
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`: `caseCount=13`, `passedCount=13`, `failedCount=0`
- [plan.mdとの差分]:
  - 今回は surface や renderer ではなく token 正規化層の不備が原因だったので、修正は language analysis / provider の highlight 分類に限定した。
# 2026-04-10 メモ (examples の doctest 実働化とコメント整備)

- [原因]:
  - `examples/*.nepl` の先頭には `//: neplg2:test` が書かれていたが、`nodesrc/parser.{ts,js}` は fenced code block 付き doctest しか収集しておらず、examples の file-level doctest は実際には 1 件も走っていなかった。
  - さらに `stdout: mlstr:` の複数行メタデータも parser が解釈しておらず、examples の既存期待値記法がそのままでは検証に使えない状態だった。
  - `nodesrc/run_doctest.js` / `nodesrc/tests.js` の `strip_ansi` も色コードの `...m` しか除去しておらず、`counter2.nepl` の `\x1b[2K` のような ANSI 制御を比較で扱えなかった。
- [実装]:
  - `nodesrc/parser.ts`
    - `.nepl` では fenced code block が無い `neplg2:test` を file-level doctest として扱い、ファイル全体を source にする fallback を追加した。
    - `stdout: mlstr:` / `stdin: mlstr:` / `stderr: mlstr:` を `##:` 行から復元する処理を追加した。
    - 連続した `neplg2:test` を正しく分離できるように meta scan の停止条件を整理した。
  - `nodesrc/parser.js`
    - 上記の Node 実行用 JS 反映を行い、実運用の test runner でも同じ挙動になるようそろえた。
  - `nodesrc/run_doctest.js`, `nodesrc/tests.js`
    - `strip_ansi` を汎用 CSI シーケンスまで除去する形へ広げ、色コードだけでなく `\x1b[2K` などの制御も比較前に正規化できるようにした。
  - `examples/helloworld.nepl`, `examples/counter.nepl`, `examples/counter2.nepl`, `examples/fib.nepl`, `examples/stdio.nepl`, `examples/nm.nepl`, `examples/bf.nepl`, `examples/rpn_regacy.nepl`
    - `examples/rpn.nepl` に合わせた日本語のドキュメントコメントへ統一し、各 example の目的・実装・注意点が読めるようにした。
    - doctest を追加または修正して、example 自体が回帰確認できるようにした。
  - `examples/nm.nepl`
    - `--help` / 未知オプションのとき usage 表示後に stdin を読まず終了するように修正した。
  - `doc/examples.md`, `doc/testing.md`
    - examples を focused に確認するコマンドと運用方針を文書化した。
- [確認]:
  - `trunk build`: 通過
  - `node nodesrc/tests.js -i examples --no-tree -o tmp/examples-tests.json -j 1`: `total=12`, `passed=12`, `failed=0`, `errored=0`
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`: `13/13 passed`
- [実装状況]:
  - examples の短いサンプルは comment 形式と doctest 導線をそろえられた。
  - examples の file-level doctest は parser / runner 側から正式に解釈されるようになった。
- [plan.mdとの差分]:
  - `plan.md` 自体は変更していない。
  - 今回は examples 整備の過程で test infrastructure 側の未対応仕様が原因と分かったため、sample だけでなく `nodesrc/parser*` と runner の正規化処理も修正対象に含めた。
  - `examples/bf.nepl` では loop を含む Brainfuck サンプルが期待どおり動かない追加差異を確認したため、現時点では確実に通る sample と bracket error を固定し、loop 系の根本調査は `todo.md` に残した。
# 2026-04-09 メモ (playground highlight 経路の一本化と surface 正規化)

- [原因]:
  - syntax highlight の生成経路が `web/src/editor-core/language-analysis.ts` と `web/src/language/neplg2/neplg2-provider.ts` に二重化しており、final payload と provisional payload で token 分類規則が一致していなかった。
  - provider 側の provisional highlight は独自 token scanner で再字句解析していたため、directive/import や function 昇格の扱いが final analysis とずれ、タブ切替や差分更新で色が揺れていた。
  - `CanvasEditor.rebuildLanguageRenderCaches()` は line ごとの segment を単純 sort するだけで、overlap や重複の正規化がなく、複数 token が同じ列に重なったときの描画優先順位が不定だった。
- [修正]:
  - `web/src/editor-core/language-analysis.ts`
    - `analysis snapshot -> EditorUpdatePayload` を唯一の highlight 変換器として維持しつつ、差分更新用の `remapEditorUpdatePayloadForTextChange()` を追加した。
    - provisional 更新でも独自 tokenization は行わず、前回の final payload を安全に remap して影響範囲外だけ再利用する形に統一した。
  - `web/src/language/neplg2/neplg2-provider.ts`
    - `_tokenType`、`_tokenizeDirectiveSpan`、`_buildEditorTokens` などの独自 highlight helper を削除し、bridge 経由に一本化した。
    - provisional payload は `window.NEPLPlaygroundLanguageAnalysis.remapEditorUpdatePayloadForTextChange(...)` のみを使うようにした。
  - `web/src/editor/editor.ts`
    - `rebuildLanguageRenderCaches()` に overlap 正規化を追加し、token/diagnostic segment を「昇順・重複なし・隣接 merge 済み」の描画入力へ整形するようにした。
    - token priority は `comment > string > function > keyword > number/boolean > operator > punctuation > variable > default` に固定した。
  - `tests/playground_editor/analysis_directives_imports`
    - `#indent` を含むケースへ更新し、directive 行の keyword/string/number/operator/variable 分解を formal fixture に固定した。
  - `nodesrc/playground_editor_surface_test_runner.js`
    - overlap token/diagnostic の正規化と、`setText()` が full-document replace を使うことを headless で固定した。
- [確認]:
  - `npm --prefix web run build:ts`: 成功
  - `node nodesrc/playground_editor_surface_test_runner.js`: 成功
  - `trunk build --release`: 成功
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`: 13/13 passed
- [plan.mdとの差分]:
  - `analysis_payload_basic` がすでに function / punctuation / variable の fixture を持っていたため、新しい mixed fixture は追加せず、directive/import 側のケース拡張と surface runner の強化で回帰を固定した。

# 2026-04-10 メモ (examples/bf.nepl loop サンプルの根本修正)

- [原因]:
  - `examples/bf.nepl` の `eval_line` がテープ長として `mem_size` を参照していたが、ここで解決されていたのは `main` のローカル変数ではなく `core/mem` の `mem_size` だった。
  - そのため `>` / `<` の折り返し判定が 30000 セルではなく WASM memory page 数に依存し、loop を含む Brainfuck でポインタが不正に巻き戻っていた。
  - この誤判定のあとに不正な位置を触ることで、局所変数が壊れたような不安定な挙動に見えていた。
- [修正]:
  - `examples/bf.nepl`
    - `eval_line` の引数に `tape_len` を追加し、折り返し判定を明示的に呼び出し側から受け取るようにした。
    - `main` 側のローカル変数名も `mem_size` から `tape_len` に変更し、`core/mem` の `mem_size` と衝突しないようにした。
    - 一時的に入れていたデバッグ出力を除去した。
    - 先頭 doctest を loop を含む `++++++++[>++++++++<-]>+.` に更新し、bracket/jump を通る経路を常時検証するようにした。
- [確認]:
  - `node nodesrc/run_doctest.js -i examples/bf.nepl -n 1`: pass
  - `++[>++<-]>++.` を追加確認し、出力が `\x06` になることを確認
  - `trunk build`: success
  - `node nodesrc/tests.js -i examples --no-tree -o tmp/examples-tests.json -j 1`: `total=12`, `passed=12`, `failed=0`, `errored=0`
  - `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests.json`: `13/13 passed`
- [plan.mdとの差分]:
  - `plan.md` 自体の変更は不要。
  - 残件としてメモしていた `examples/bf.nepl` の loop サンプル不具合は解消したため `todo.md` から削除した。
