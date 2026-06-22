# NEPLg2 GUI font rendering detailed design

作成日: 2026-06-13

## F5el real loop driver checkpoint

2026-06-18 の F5el では、std layer row tile RLE present host span operation presenter executor session turn virtual scheduler real loop driver boundary を追加する。`RealLoopDriverPolicy` は F5ef loop policy だけを保持し、F5ek step policy、scheduler policy、timer policy、backend executor、clock、queue を重複保持しない。`start` は F5ef `loop_step` と F5eg `loop_action_from_result` を 1 回ずつ呼び、`after_step` は F5ek result を `StateReady` / `YieldPending` / `Completed` として match する。`StateReady` は `loop_resume` へ戻し、`remaining_count == 0` は budget-yield semantics に従って yield action へ進め、error / completion / `CompleteAck` / fallback / silent no-op へ変換しない。

## F5em headless app-loop step checkpoint

2026-06-18 の F5em では、std layer row tile RLE present host span operation presenter executor session turn virtual scheduler headless app-loop step boundary を追加する。`HeadlessAppLoopStepPolicy` は F5el `RealLoopDriverPolicy` と F5ek `RealLoopStepPolicy` だけを保持し、F5ef loop policy、scheduler policy、timer policy、backend clock、executor backend、queue、platform API を直接保持しない。`start` は F5el `real_loop_driver_start` を 1 回だけ呼び、`advance` は previous `NeedInput` action と caller supplied F5ek input を受け、F5ek `real_loop_step` を 1 回、成功時だけ F5el `real_loop_driver_after_step` を 1 回呼ぶ。`Completed` は terminal output だけであり advance input ではない。`Complete` action は caller が `CompleteAck` を渡すまで `NeedInput` のまま保持し、F5em は ack を合成しない。`remaining_count == 0` は F5em で解釈せず、F5el / F5ec の budget-yield semantics に任せる。fallback と silent no-op は行わない。

## F5en bounded headless app-loop runner checkpoint

2026-06-18 の F5en では、std layer row tile RLE present host span operation presenter executor session turn virtual scheduler bounded headless app-loop runner boundary を追加する。これは fixed-slot script を使う deterministic test boundary であり、not long-running real backend loop である。`HeadlessAppLoopRunnerPolicy` は F5em `HeadlessAppLoopStepPolicy` と `max_advance_count` だけを保持し、F5ek / F5el の内部 policy、backend clock、executor backend、queue、platform API を保持しない。`HeadlessAppLoopRunnerScript` は 3 slot の `Option RealLoopStepInput`、`count`、`cursor` だけを保持し、slot hole、負 cursor、capacity 超過は `ScriptInvalid` として typed error にする。`InputMissing` は `NeedInput` に対する次 input が本当に存在しない場合だけ返し、`ClockDelta`、`ExecutorOutcome`、`CompleteAck` を合成しない。`BudgetExhausted` は `max_advance_count == 0` または bounded drain の budget を使い切った場合の terminal result であり、F5em `advance` を呼ばない。`Completed` は script を消費しない。fallback と silent no-op は行わない。

## F5eo backend clock delta checkpoint

2026-06-18 の F5eo では、std layer row tile RLE present host span operation presenter executor session turn virtual scheduler backend clock delta boundary を追加する。これは Web / native / bare / headless backend が取得した monotonic clock sample を、F5ek `RealLoopStepInput::ClockDelta` へ変換する pure std boundary である。`BackendClockPolicy` は `max_delta_ms` だけを保持し、`BackendClockSample` は caller supplied `monotonic_ms`、`BackendClockState` は previous `last_monotonic_ms` だけを保持する。sample / state は public value なので、`start` と `advance` は constructor を信用せず entry で再検査する。`start` は baseline state を返し delta を発行しない。`advance` は negative policy、negative sample、forged negative state、backward time、too-large delta を typed error として返し、error payload は policy / state / sample / previous / current / delta / max を回収可能な形で保持する。zero delta は no-op や error にせず `ClockDelta 0` として返す。delta が `max_delta_ms` を超えた場合は clamp せず `DeltaTooLarge` を返す。F5eo は actual clock source、sleep、timer backend、executor outcome、complete ack、queue、platform API、DOM、Canvas、minifb、video memory、fallback、silent no-op を実装しない。

## F5ep Web monotonic clock source checkpoint

2026-06-18 の F5ep では、Web formal monotonic clock source backend boundary を追加する。`platforms/gui/web/clock` は `nepl_gui_web.monotonic_clock_ms` の単一 `i32` return ABI を受け、0 以上を `performance.now` 由来の floored millisecond sample、-1 を unsupported、その他の負値を `BackendFailure` として扱う。Web worker は `performance.now` を呼んだ後、`Number.isFinite`、0 以上、`i32::MAX` 以下、integer 化後の妥当性を検査してから Wasm 境界へ返す。`i32::MAX` ms を超えた sample は wrap や clamp ではなく `BackendFailure` である。NEPL wrapper は negative sentinel を `GuiError` へ写した後だけ F5eo `BackendClockSample` constructor を呼ぶ。`Date.now`、`setTimeout`、`setInterval`、stdout protocol、polling loop、queue、DOM、Canvas、fallback、silent no-op は clock source として使わない。native / bare / headless の actual clock source は後続 slice で実装する。

## F5eq Headless scripted monotonic clock source checkpoint

2026-06-18 の F5eq では、Headless scripted monotonic clock source backend boundary を追加する。`platforms/gui/headless/clock` は deterministic headless / offscreen test 用の actual clock input source であり、wall clock ではなく fixed-slot script から F5eo `BackendClockSample` を 1 件ずつ返す。script は `Option BackendClockSample` の 3 slot、`count`、`cursor` だけを保持し、`count` は 0 から 3、`cursor` は 0 から `count`、slot は count に一致する `Some` / `None` shape でなければならない。constructor は raw i32 sample を F5eo constructor で検査してから保持し、poll も public script を信用せず count / cursor / slot shape / sample を再検査する。`cursor == count` は `Option::None` を返し、zero sample や delta を合成しない。timer、queue、host import、platform API、wall clock、fallback、silent no-op は使わない。native / bare actual clock source と long-running backend loop は後続 slice で実装する。

## F5er Native formal monotonic clock source checkpoint

2026-06-18 の F5er では、Native formal monotonic clock source backend boundary を追加する。`platforms/gui/native/clock` は `nepl_gui_native.monotonic_clock_ms` の単一 `i32` return ABI を受け、0 以上を native `Instant` 由来の monotonic millisecond sample、-1 を unsupported、その他の負値を `BackendFailure` として扱う。Rust `nepl-gui-native` 側は elapsed millisecond を `i32::MAX` 以下で検査し、超過は wrap や clamp ではなく backend failure sentinel にする。NEPL wrapper は negative sentinel を `GuiError` へ写した後だけ F5eo `BackendClockSample` constructor を呼ぶ。timer、sleep、queue、window loop、present、scheduler backend、minifb rendering、stdout protocol、fallback、silent no-op は clock source として使わない。bare actual clock source、native / bare scheduler backend、long-running real backend loop は後続 slice で実装する。

## F5es Bare formal monotonic clock source checkpoint

2026-06-18 の F5es では、Bare formal monotonic clock source backend boundary を追加する。`platforms/gui/bare/clock` は `nepl_gui_bare.monotonic_clock_ms` の単一 `i32` return ABI を受け、0 以上を embedding host が明示提供する monotonic millisecond sample、-1 を `Unsupported`、その他の負値を `BackendFailure` として扱う。Bare stdlib は universal wall clock を仮定せず、Web `performance.now`、native `Instant`、wall clock、timer、sleep、queue、window loop、present、scheduler backend、minifb rendering、stdout protocol、fallback、silent no-op を clock source として使わない。`nodesrc/run_test.js` の `nepl_gui_bare` 既定 import は doctest-only unsupported source であり hidden fallback や hidden mock ではない。native / bare scheduler backend、long-running real backend loop は後続 slice で実装する。

## F5et Native and Bare scheduler clock one-tick helper boundary

2026-06-18 の F5et では、Native and Bare scheduler clock one-tick helper boundary を追加する。これは not long-running scheduler backend であり、platform clock source と F5eo `BackendClockPolicy` / `BackendClockState` の接続だけを担当する。`start` は platform sample を 1 回取得して F5eo `backend_clock_start` へ渡し、`tick` は platform sample を 1 回取得して F5eo `backend_clock_advance` へ渡す。policy は F5eo `BackendClockPolicy` そのものであり、新しい scheduler policy、timer policy、loop policy は持たない。tick は `ClockDelta` を直接合成せず、F5eo `BackendClockAdvance` を返す。start sample failure は policy と `GuiError`、tick sample failure は policy / state / `GuiError` を保持し、F5eo lower error は再分類せず lower error として保持する。timer、sleep、queue、while loop、present、minifb、Canvas、video memory、fallback、silent no-op は使わない。

## F5eu Native and Bare scheduler clock action input helper boundary

2026-06-18 の F5eu では、Native and Bare scheduler clock action input helper boundary を追加する。これは action input helper only であり、not long-running scheduler backend である。`platforms/gui/native/scheduler_clock_input` と `platforms/gui/bare/scheduler_clock_input` は F5eg `LoopActionYieldToClock` / `LoopActionAwaitTimerAdvance` を typed authority として受け、F5et `gui_*_scheduler_clock_tick` を entry ごとに 1 回だけ呼ぶ。成功時は F5eo `BackendClockAdvance` が保持する新しい `BackendClockState` と F5ek `RealLoopStepInput` を取り出し、original action と一緒に success payload へ保存する。失敗時は original action、input clock state、lower platform scheduler clock error を owner-bearing error として返す。general `LoopAction`、`ExecuteHostAction`、`Complete` は対象外であり、`ExecutorOutcome` / `CompleteAck` / `ClockDelta` はこの層で合成しない。real loop driver、headless app-loop step、queue、timer backend、sleep、present、minifb、Canvas、DOM、video memory、fallback、silent no-op は持たない。

## F5ev Native and Bare scheduler executor outcome input helper boundary

2026-06-18 の F5ev では、Native and Bare scheduler executor outcome input helper boundary を追加する。これは backend-facing input boundary であり、not long-running scheduler backend である。`platforms/gui/native/scheduler_executor_input` と `platforms/gui/bare/scheduler_executor_input` は F5eg `LoopActionExecuteHostAction` typed payload と caller supplied `Result unit GuiError` outcome だけを受ける。helper は outcome を F5ek `RealLoopStepInput::ExecutorOutcome` として 1 回だけ total packaging し、original action と同じ ready payload に保存する。

この helper は does not return Result である。理由は、general `LoopAction`、`YieldToClock`、`AwaitTimerAdvance`、`Complete` が public entry の型で除外され、unsupported operation を runtime 分岐で検出する必要がないためである。executor success / failure は caller supplied outcome の中身であり、この boundary は `Result::Ok unit` や `Result::Err GuiError::*` を合成しない。F5ei executor complete、F5ek real loop step、action sink / driver、support validation、timer、sleep、queue、while loop、present、minifb、Canvas、DOM、video memory、fallback、silent no-op には進まない。

## F5fg Native presenter operation identity input boundary

2026-06-18 の F5fg では、Native presenter operation identity input boundary を追加する。これは presenter-facing input boundary であり、not long-running scheduler backend である。F5ev is the scheduler step input boundary; F5fg is the native presenter-facing identity input boundary. F5fg は Native 専用の presenter-facing ready value を作るが、scheduler completion wrapping は F5ev に委譲する。

`platforms/gui/native/presenter_input` は typed `ExecuteHostAction` だけを受ける。まず `execute_host_action_execute_ref` と `virtual_scheduler_execute_pending_ref` で action owner を消費せずに pending を読む。次に `turn_driver_pending_operation` で pending span operation identity を取り出し、`GuiNativePresenterInputOperationIdentity` に保持する。最後に original action と caller supplied `Result unit GuiError` outcome を `gui_native_scheduler_executor_input` に渡し、F5ev の `GuiNativeSchedulerExecutorInputReady` を保持する。

operation identity は `WindowBegin`、`WindowRunSpan`、`WindowEnd`、`OffscreenBegin`、`OffscreenRunSpan`、`OffscreenEnd`、`DeviceBegin`、`DeviceRunSpan`、`DeviceEnd` のいずれかであり、string tag や raw integer には落とさない。F5fg は backend execution、raw status mapping、scheduler step、queue、timer、window loop、minifb、Canvas、DOM、video memory、fallback、silent no-op を持たない。

## F5ew Native and Bare scheduler executor one-step bridge boundary

2026-06-18 の F5ew では、Native and Bare scheduler executor one-step bridge boundary を追加する。これは backend-facing one-step bridge であり、not long-running scheduler backend である。F5ev で作った `GuiNativeSchedulerExecutorInputReady` / `GuiBareSchedulerExecutorInputReady` と borrowed F5ek `RealLoopStepPolicy` を受け、ready payload 内の original `ExecuteHostAction` を `LoopAction::ExecuteHostAction` へ包み、packaged input を F5ek `real_loop_step` へ渡す。

F5ew は input packaging と F5ek の間の接続を固定するだけで、actual host action executor ではない。F5ek `real_loop_step` を 1 回だけ呼び、戻り値の `Result RealLoopStepResult RealLoopStepError` は再分類せずに返す。`Result::Ok unit` / `Result::Err GuiError` の合成、unsupported error 合成、F5ei executor complete の直接呼び出し、action sink / driver、support validation、clock / timer helper、queue、while loop、present、minifb、Canvas、DOM、video memory、fallback、silent no-op は持たない。長時間 scheduler backend は、F5ew の one-step bridge を使う別 slice として実装する。

## F5ex Native and Bare scheduler host action executor backend bridge

2026-06-18 の F5ex では、Native and Bare scheduler host action executor backend bridge を追加する。これは backend-facing bridge であり、not long-running scheduler backend である。F5ew までは caller supplied outcome を F5ek に渡すだけだったため、actual platform host import を呼ぶ境界が残っていた。F5ex は typed `ExecuteHostAction` だけを入口にし、borrowed accessor で `VirtualSchedulerExecute` と `TurnDriverPending` を読み、`turn_driver_pending_operation` で expected span operation を復元する。

operation 実行は target と operation kind を scalar ABI に展開する。Begin / End は `GuiRgba8888RowTileRlePresentDescriptor` から window、surface、frame、packet metadata を読む。Run は `GuiRgba8888RowTileRlePresentRunRowSpan` から x、y、width、height、RGBA channel を読む。F5fj 以降の Native は `nepl_gui_native` namespace の `window_presenter_session_begin`、`window_presenter_session_run`、`window_presenter_session_end` を呼び、F5fk 以降の Bare は `nepl_gui_bare` namespace の `display_presenter_session_begin`、`display_presenter_session_run`、`display_presenter_session_end` を呼ぶ。各 branch は該当 host import を 1 回だけ呼び、status `0` を `Ok unit`、負 status を `GuiError` に写して `Result unit GuiError` として返す。

F5ex は host import の outcome を F5ev/F5ew path に渡し、F5ek `real_loop_step` へ戻す。これにより platform executor bridge は actual host execution と status mapping だけに集中し、scheduler state mutation と completion authority は std layer に残る。Bare では host ABI が未提供なら `Unsupported` で fail closed し、fallback や silent no-op は行わない。long-running scheduler backend、queue、while loop、timer wait、present loop、FHD 60fps 実測、2D compositor drain、minifb / DOM / Canvas / video memory、old action sink / driver、raw RenderCommand accessor はこの phase に含めない。

## 責務分離

Font rendering は `core/gui`、`alloc/gui`、`std/gui`、`platforms/gui/*` で責務を分ける。

```text
core/gui/font
    id
    writing mode
    font request flags
    no_alloc metrics
    glyph paint bridge

alloc/gui/font
    sfnt table directory
    cmap / head / hhea / hmtx / maxp / name / os2
    loca / glyf / CFF / CFF2
    gsub / gpos / kern
    vhea / vmtx / vorg
    variation tables
    shaped run
    glyph outline
    glyph cache

alloc/gui/text
    inline layout
    bidi
    line break
    ruby
    vertical layout
    math inline bridge

std/gui/font_resource
    resource request
    VFS / filesystem / embedded blob source descriptor
    decode policy
    font collection registration boundary

platforms/gui/web
    bundled font VFS mapping
    resource bytes provider
    pixel presentation only

platforms/gui/native
    resource root / packaged resource provider
    framebuffer presenter

platforms/gui/bare
    embedded blob provider
    unsupported filesystem behavior

platforms/gui/headless
    explicit fixture resource provider
    no present / screenshot fallback
```

依存方向は上から下へ流れない。`core/gui/font` は `alloc`、`std`、`platforms` を import しない。`std/gui/font_resource` は platform resource provider の抽象 request だけを作り、OS handle や browser object を持たない。

## Resource flow

初期 fixture は repository 既存の `web/src/fonts/HackGenConsoleNF-Regular.ttf` である。ただし public API は HackGen 専用名を持たない。

```text
resource root
    -> fonts/HackGenConsoleNF-Regular.ttf
    -> GuiFontResourceRequest
    -> std/gui resource provider
    -> alloc/gui sfnt parser
    -> GuiFontFaceId
    -> shaped run / metrics / glyph masks
    -> render2d pixel buffer
```

Web では bundled resource manifest が canonical resource path `fonts/HackGenConsoleNF-Regular.ttf` と license text を登録し、VFS 内部では `/fonts/HackGenConsoleNF-Regular.ttf` と `/fonts/HackGen-LICENSE.txt` に mount する。`/fonts/...` は VFS transport の表現であり、font identity や lookup authority にはしない。Web Playground は startup で mount promise を開始し、`neplg2 run` / Wasm execution の直前に完了を待つ。Mount に失敗した場合は `GuiFontResourceMountError` の typed variant を terminal に表示し、execution は開始しない。Compile-only path は runtime font bytes を必要としないため mount 完了を待たない。

Native では packaged resource directory または configured resource root から同じ canonical path を読む。Native provider は `fonts/...` を root-relative path として扱い、OS font family lookup や current working directory の suffix scan へ逃がさない。Resource root が未設定、または path が存在しない場合は missing resource error を返す。

Bare では embedded blob table を provider とする。Blob table が未設定の環境では filesystem probing を行わず unsupported を返す。Blob table に canonical path が存在しない場合は missing resource error を返す。Headless / offscreen tests は explicit fixture resource provider を渡し、host font API には依存しない。

## Decode policy

```text
GuiFontDecodePolicy:
    SfntOnly
    SfntAndWoff
    AllSupportedContainers
```

`SfntOnly` は TTF / OTF / TTC / OTC の sfnt container だけを許す。WOFF / WOFF2 は compressed container decode が必要なため後続 phase とする。`AllSupportedContainers` でも、実装がまだ対応していない container は typed error で拒否する。

## Face selection

```text
GuiFontFaceSelection:
    resource GuiFontResourceId
    face_index Option i32
```

`Option::None` の意味は font container の face count に依存する。

- single face: `None` は face 0 を表す。
- collection face: `None` は invalid。`FaceIndexRequired` 相当の error にする。
- negative index: invalid。
- `face_count <= index`: invalid。

この規則は source policy と doctest で固定する。

F2 の `GuiFontResourceRequest` constructor は request shape だけを検査する。つまり path が empty でないこと、`face_index` が `Some n` の場合に `n >= 0` であること、hash value が typed `GuiResourceHash` であること、decode policy が enum value であることを確認する。Collection font の `face_count` を必要とする `FaceIndexRequired` / out-of-range 判定は、F4 の sfnt metadata parser または font registry が font bytes を読める段階で行う。

## SFNT name table

F4b は `alloc/gui/font/sfnt/name.nepl` が所有する。`alloc/gui/font/sfnt.nepl` は facade とし、numeric metadata parser は `alloc/gui/font/sfnt/metadata.nepl` に置く。`gui_sfnt_parse_metadata` は `name` table decode を行わず、`gui_sfnt_parse_names` は別 API として `GuiSfntNames` を返す。

```text
GuiSfntNames:
    family Option str
    subfamily Option str
    full_name Option str

GuiSfntNameEncodingKind:
    WindowsUnicodeBmpAscii
    MacintoshRomanAscii
```

Representative name selection is deterministic per nameID:

```text
nameID 1: family
nameID 2: subfamily
nameID 4: full name

rank 400:
    platformID 3
    encodingID 1
    languageID 0x0409
    decode as UTF-16BE ASCII subset

rank 300:
    platformID 3
    other encoding or language
    selected only when rank 400 is absent, then UnsupportedNameEncoding

rank 200:
    platformID 1
    encodingID 0
    languageID 0
    decode as Roman ASCII subset

rank 100:
    platformID 1
    other encoding or language
    selected only when higher ranks are absent, then UnsupportedNameEncoding
```

Other platform IDs are not representative candidates in F4b. This is not a substitute-font mechanism; it is a narrow metadata extraction rule. If no candidate exists for a representative nameID, the field is `Option::None`. If a higher-ranked candidate exists but cannot be decoded by the initial engine, parsing returns typed error rather than silently taking a lower-ranked candidate.

Name table validation:

- table format must be 0; otherwise `UnsupportedNameTableFormat`.
- `count * 12`, record area, and string storage offset must stay inside the `name` table.
- record string ranges are relative to `stringOffset`, not to the file start.
- UTF-16BE representative strings must have even byte length.
- selected representative strings must be non-empty.
- ASCII subset means all decoded scalar values are in byte range `0..127`.

## SFNT cmap table

F4c は `alloc/gui/font/sfnt/cmap.nepl` が所有する。`gui_sfnt_parse_metadata` は `cmap` table の有無を directory summary に記録するだけで、glyph lookup を行わない。`gui_sfnt_lookup_glyph_id` は別 API として font bytes、face index、Unicode code point を受け取り、`GuiGlyphId` を返す。

```text
GuiSfntCmapSubtableRecord:
    platform_id i32
    encoding_id i32
    offset i32

GuiSfntCmapEncodingKind:
    WindowsUnicodeBmpFormat4
```

Subtable selection は次だけを許す。

```text
selected:
    first record where platformID == 3 and encodingID == 1

absent:
    UnsupportedCmapEncoding

selected format != 4:
    UnsupportedCmapTableFormat
```

Platform 0、platform 3 encoding 10、Macintosh record、後続 record は F4c の代替 candidate ではない。format 4 が対応できない code point は `UnsupportedCmapEncoding`、format 4 の BMP 範囲内だが glyph mapping が存在しない場合は `MissingGlyphMapping` である。

Format 4 lookup は OpenType table layout の byte offsets から直接行う。

```text
format                  u16 offset 0
length                  u16 offset 2
language                u16 offset 4
segCountX2              u16 offset 6
searchRange             u16 offset 8
entrySelector           u16 offset 10
rangeShift              u16 offset 12
endCode[segCount]       offset 14
reservedPad             after endCode
startCode[segCount]
idDelta[segCount]
idRangeOffset[segCount]
glyphIdArray[]
```

Validation rules:

- `length` and `language` are readable.
- `length` is at least `16 + 8 * segCount` and remains inside the selected `cmap` table.
- selected subtable offset is not inside the encoding record array.
- `segCountX2` is even and greater than 0.
- `reservedPad` is 0.
- every segment array range is inside the subtable.
- `idRangeOffset == 0` uses `(code_point + idDelta) mod 65536`.
- `idRangeOffset != 0` computes glyph array address from the address of that idRangeOffset word, then adds `2 * (code_point - startCode)`.
- the computed glyph array address must remain inside the subtable.
- raw glyph 0, computed glyph 0, and no matching segment are `MissingGlyphMapping`.

## SFNT horizontal metrics table

F4d は `alloc/gui/font/sfnt/hmtx.nepl` が所有する。`gui_sfnt_parse_metadata` は `hmtx` table の有無を directory summary に記録するだけで、horizontal metric lookup を行わない。`gui_sfnt_lookup_horizontal_metric` は別 API として font bytes、face index、checked `GuiGlyphId` を受け取り、`GuiSfntHorizontalMetric` を返す。

```text
GuiSfntHorizontalMetric:
    glyph GuiGlyphId
    advance_width i32
    left_side_bearing i32
```

`numberOfHMetrics` は `hhea` table offset 34 の u16 である。F4a metadata parser は line metrics のために `hhea.length >= 10` だけを要求するが、F4d metric lookup は `numberOfHMetrics` を読むので `hhea.length >= 36` を要求する。

Validation rules:

- `hmtx` table must exist; otherwise `MissingTable`.
- `hhea.length >= 36` must hold for this lookup; otherwise `MalformedHmtxRecord`.
- `numberOfHMetrics > 0`.
- `numberOfHMetrics <= maxp.numGlyphs`.
- valid public metric lookup range is `1 <= glyphRaw < numGlyphs`; glyph 0 is not a successful renderable glyph in the GUI font contract.
- required declared `hmtx.length` is `numberOfHMetrics * 4 + (numGlyphs - numberOfHMetrics) * 2`.
- all reads use `hmtx.offset + table_relative_offset`, and each relative range must stay inside declared `hmtx.length`.

Lookup layout:

```text
longHorMetric[numberOfHMetrics]:
    advanceWidth u16
    leftSideBearing i16

leftSideBearing[numGlyphs - numberOfHMetrics]:
    i16
```

If `glyphRaw < numberOfHMetrics`, the metric offset is `glyphRaw * 4`. If `glyphRaw >= numberOfHMetrics`, advance width is read from `(numberOfHMetrics - 1) * 4`, and left side bearing is read from `numberOfHMetrics * 4 + (glyphRaw - numberOfHMetrics) * 2`.

`hmtx` does not provide ink bounds or outline geometry. F4d therefore returns `GuiSfntHorizontalMetric` rather than pretending to produce full `GuiGlyphMetrics`. Conversion to `GuiGlyphMetrics` happens after outline / bitmap bounds are available.

## SFNT glyph header bounds

F4e は `alloc/gui/font/sfnt/glyf.nepl` が所有する。`gui_sfnt_parse_metadata` は `loca` / `glyf` table の有無を directory summary に記録するだけで、glyph bounds lookup を行わない。`gui_sfnt_lookup_glyph_bounds` は別 API として font bytes、face index、checked `GuiGlyphId` を受け取り、`GuiSfntGlyphBounds` を返す。

```text
GuiSfntGlyphBounds:
    glyph GuiGlyphId
    x_min i32
    y_min i32
    x_max i32
    y_max i32
```

`indexToLocFormat` は `head` table offset 50 の i16 である。F4a metadata parser は `unitsPerEm` のために `head.length >= 20` だけを要求するが、F4e glyph bounds lookup は `indexToLocFormat` を読むので `head.length >= 52` を要求する。この要件を metadata parser へ持ち込んではならない。

Validation rules:

- `loca` / `glyf` table must exist; otherwise `MissingTable`.
- `head.length >= 52` must hold for this lookup; otherwise `MalformedGlyfRecord`.
- `indexToLocFormat == 0` uses short offsets: `loca[glyph]` and `loca[glyph + 1]` are u16 values multiplied by 2.
- `indexToLocFormat == 1` uses long offsets: `loca[glyph]` and `loca[glyph + 1]` are u32 values constrained to i32 range.
- other `indexToLocFormat` values are `UnsupportedLocaFormat`.
- required declared `loca.length` is `(numGlyphs + 1) * 2` for format 0 and `(numGlyphs + 1) * 4` for format 1.
- valid public glyph bounds lookup range is `1 <= glyphRaw < numGlyphs`; glyph 0 is not a successful renderable glyph in the GUI font contract.
- glyph offset pair must satisfy `start <= end <= glyf.length`.
- `start == end` is `MissingGlyphOutline`.
- `end - start < 10`, inverted x bounds, and inverted y bounds are `MalformedGlyfRecord`.
- all glyph reads use `glyf.offset + table_relative_offset`, and each relative range must stay inside declared `glyf.length`.

Lookup layout:

```text
loca format 0:
    offset[numGlyphs + 1] u16, actual offset = value * 2

loca format 1:
    offset[numGlyphs + 1] u32

glyf glyph header:
    numberOfContours i16
    xMin i16
    yMin i16
    xMax i16
    yMax i16
```

F4e reads `numberOfContours` only as part of the required 10 byte header. Simple glyph contour arrays, composite component recursion, CFF / CFF2 charstrings, fill / stroke rasterization, and `GuiGlyphMetrics` synthesis are later phases.

### SFNT simple glyph topology

F4f extends `alloc/gui/font/sfnt/glyf.nepl` with simple glyph topology. This remains in `glyf.nepl` because it shares the same `head` / `loca` / `glyf` table lookup and declared range validation as F4e. A later point decoder can split into `glyf/simple.nepl` or `outline.nepl` when flags, coordinates, and composite recursion grow beyond header topology.

```text
GuiSfntSimpleGlyphTopology:
    glyph GuiGlyphId
    bounds GuiSfntGlyphBounds
    contour_count i32
    point_count i32
    instruction_length i32
    point_data_offset i32
    point_data_length i32
```

`point_data_offset` is relative to the `glyf` table, not absolute file offset. The file offset is `glyf.offset + point_data_offset`. Keeping topology ranges table-relative lets headless, native, web, and bare providers share the same checked payload without path or host font authority.

Simple glyph layout:

```text
glyf glyph header:
    numberOfContours i16
    xMin i16
    yMin i16
    xMax i16
    yMax i16

simple glyph payload:
    endPtsOfContours[numberOfContours] u16
    instructionLength u16
    instructions[instructionLength] u8
    flags and coordinate stream
```

Validation:

- F4e `loca` / `glyf` range and bounds validation must pass first.
- `numberOfContours < 0` is `UnsupportedGlyphOutlineFormat`.
- `numberOfContours == 0` is `MissingGlyphOutline`.
- endpoint array range and `instructionLength` range must remain inside the selected glyph range.
- endpoints must be strict increasing; `point_count = last_endpoint + 1`.
- `point_count <= 0` or overflow is `MalformedGlyfRecord`.
- `point_data_offset = instruction_start + instructionLength`, `point_data_length = glyph_end - point_data_offset`.
- `numberOfContours > 0` and `point_count > 0` with zero point data length is `MalformedGlyfRecord`.

F4f deliberately does not parse flags or x/y deltas. That means it only proves the topology prefix and non-empty point stream boundary. Later outline phases must validate repeat flags, coordinate stream length, contour closure, and composite recursion.

### SFNT simple glyph point stream

F4g adds a point stream range decoder without constructing point values. The decoder consumes the raw flags stream, expands repeat counts only for counting, and derives the x/y coordinate byte ranges.

```text
GuiSfntSimpleGlyphPointStream:
    topology GuiSfntSimpleGlyphTopology
    flag_data_offset i32
    flag_data_length i32
    x_data_offset i32
    x_data_length i32
    y_data_offset i32
    y_data_length i32
    trailing_data_offset i32
    trailing_data_length i32
```

All offsets are relative to the `glyf` table. `flag_data_offset` equals `topology.point_data_offset`. `flag_data_length` is the raw consumed flag stream length, including repeat-count bytes; it is not the expanded logical point count.

Flag scan state:

```text
logical_point_count
raw_flag_cursor
x_coordinate_byte_count
y_coordinate_byte_count
```

Repeat semantics:

- A flag byte always contributes one logical point.
- If repeat bit 3 is set, the next byte is an additional repeat count.
- The total run is `1 + repeat_count`.
- `repeat_count = 0` is valid and means no additional logical point.
- A run that crosses `point_count` is malformed.
- Missing repeat count byte is malformed.

Coordinate byte length is derived without decoding values:

```text
xShort == 1:
    x bytes = 1
xShort == 0 and xSame == 1:
    x bytes = 0
xShort == 0 and xSame == 0:
    x bytes = 2

yShort == 1:
    y bytes = 1
yShort == 0 and ySame == 1:
    y bytes = 0
yShort == 0 and ySame == 0:
    y bytes = 2
```

When short is set, the same/positive bit controls sign, not byte length. F4g therefore records byte ranges only. Actual delta sign and cumulative coordinate reconstruction are F4h responsibilities.

Range derivation:

```text
flag_data_offset = topology.point_data_offset
x_data_offset = flag_data_offset + flag_data_length
y_data_offset = x_data_offset + x_data_length
trailing_data_offset = y_data_offset + y_data_length
trailing_data_length = glyph_end - trailing_data_offset
```

`trailing_data_length < 0` is `MalformedGlyfRecord`. Non-negative trailing data is returned explicitly, not treated as hidden fallback. Later sanitizer or outline decode phases can decide whether non-zero padding is accepted.

### SFNT simple glyph single point decode

F4h decodes one logical point from an already checked `GuiSfntSimpleGlyphPointStream`. It intentionally does not allocate a point `Vec`; the full outline builder is deferred until allocation failure and owner recovery are specified.

```text
GuiSfntSimpleGlyphPoint:
    glyph GuiGlyphId
    point_index i32
    x i32
    y i32
    on_curve bool
    end_of_contour bool
```

Public lookup:

```text
gui_sfnt_lookup_simple_glyph_point:
    &ByteBuf -> Option i32 -> GuiGlyphId -> i32
    -> Result GuiSfntSimpleGlyphPoint GuiSfntParseError
```

The lookup flow is:

```text
parse metadata
    -> unwrap head / loca / glyf
    -> gui_sfnt_glyf_simple_point_stream_with_tables
    -> validate point_index against topology.point_count
    -> decode flags and coordinate deltas only inside stream ranges
    -> read endpoint array through topology-derived endpoint offset
    -> return GuiSfntSimpleGlyphPoint
```

`point_index < 0` and `point_index >= point_count` return `MissingGlyphOutline`. These conditions are invalid public point requests, not malformed bytes.

Malformed byte structure remains `MalformedGlyfRecord`:

- repeat byte required by a flag is outside `flag_data`
- x/y coordinate read is outside F4g-derived x/y range
- endpoint array read fails despite topology validation
- internal offset arithmetic is inconsistent

Cursor semantics:

```text
flag_cursor = flag_data_offset
x_cursor = x_data_offset
y_cursor = y_data_offset
logical_index = 0
current_x = 0
current_y = 0
```

Each logical point consumes its flag first, then consumes x/y coordinate bytes according to that flag, applies deltas cumulatively, and only then compares `logical_index` with target. When target lies inside a repeated flag run, all earlier repeated points in that same run still consume coordinate bytes and update cumulative coordinates.

Delta formula:

```text
xShort and xPositive: +u8
xShort and not xPositive: -u8
not xShort and xSame: 0
not xShort and not xSame: i16be

yShort and yPositive: +u8
yShort and not yPositive: -u8
not yShort and ySame: 0
not yShort and not ySame: i16be
```

`on_curve` is flag bit 0. `end_of_contour` is true if `point_index` equals any endpoint value. The endpoint array offset is derived from topology:

```text
endpoint_array_length = contour_count * 2
endpoint_array_offset = point_data_offset - instruction_length - 2 - endpoint_array_length
```

F4h ignores `trailing_data_length` except that F4g already proved it is non-negative. It must not consume trailing bytes, require zero padding, call host font APIs, or use fixed-cell fallback.

### SFNT simple glyph contour span lookup

F4i returns one contour's inclusive logical point index span from checked simple glyph topology. It is a no-allocation boundary before the future outline builder, curve segment builder, and mask rasterizer.

```text
GuiSfntSimpleGlyphContourSpan:
    glyph GuiGlyphId
    contour_index i32
    start_point_index i32
    end_point_index i32
    point_count i32
```

Public lookup:

```text
gui_sfnt_lookup_simple_glyph_contour_span:
    &ByteBuf -> Option i32 -> GuiGlyphId -> i32
    -> Result GuiSfntSimpleGlyphContourSpan GuiSfntParseError
```

The lookup flow is:

```text
parse metadata
    -> unwrap head / loca / glyf
    -> gui_sfnt_glyf_simple_topology_with_tables
    -> validate contour_index against topology.contour_count
    -> read current endpoint from topology-derived endpoint array
    -> read previous endpoint when contour_index > 0
    -> return GuiSfntSimpleGlyphContourSpan
```

F4i deliberately depends on F4f topology validation only. It must not call `gui_sfnt_glyf_simple_point_stream_with_tables`, `gui_sfnt_lookup_simple_glyph_point_stream`, or `gui_sfnt_lookup_simple_glyph_point`.

Endpoint semantics:

```text
endpoint_array_length = contour_count * 2
endpoint_array_offset = point_data_offset - instruction_length - 2 - endpoint_array_length

previous_endpoint = -1 when contour_index == 0
previous_endpoint = endPtsOfContours[contour_index - 1] otherwise
end_point_index = endPtsOfContours[contour_index]
start_point_index = previous_endpoint + 1
point_count = end_point_index - start_point_index + 1
```

`end_point_index` is inclusive. `contour_index < 0` and `contour_index >= contour_count` return `MissingGlyphOutline`; malformed endpoint data observed through topology validation or endpoint reads returns `MalformedGlyfRecord`.

### SFNT simple glyph contour point lookup

F4j composes the F4i contour span and F4h single point decode. It is the first contour-local coordinate query, but it still does not allocate a point vector, contour vector, curve segment list, or raster mask.

```text
GuiSfntSimpleGlyphContourPoint:
    span GuiSfntSimpleGlyphContourSpan
    contour_point_index i32
    point GuiSfntSimpleGlyphPoint
```

Public lookup:

```text
gui_sfnt_lookup_simple_glyph_contour_point:
    &ByteBuf -> Option i32 -> GuiGlyphId -> i32 -> i32
    -> Result GuiSfntSimpleGlyphContourPoint GuiSfntParseError
```

The lookup flow is:

```text
parse metadata
    -> unwrap head / loca / glyf
    -> gui_sfnt_glyf_simple_contour_span_with_tables
    -> validate contour-local contour_point_index against span.point_count
    -> absolute_point_index = span.start_point_index + contour_point_index
    -> gui_sfnt_glyf_simple_point_with_tables
    -> return GuiSfntSimpleGlyphContourPoint
```

`contour_point_index` is contour-local. The nested `point.point_index` is absolute within the glyph and must equal `span.start_point_index + contour_point_index`.

The implementation must validate local point range before calling point decode. Otherwise a local out-of-range request could be hidden behind a coordinate stream error from a malformed point array. For a valid contour span, `contour_point_index < 0` and `contour_point_index >= span.point_count` return `MissingGlyphOutline`.

F4j uses internal table helpers, not public wrappers, after metadata is parsed:

```text
gui_sfnt_glyf_simple_contour_span_with_tables
gui_sfnt_glyf_simple_point_with_tables
```

This keeps table unwrap single-pass at the public boundary. F4j must not call platform font APIs, host text measurement, full outline builders, or hidden substitution paths.

### SFNT simple glyph contour edge lookup

F4k composes the F4i contour span and F4j contour-local point lookup into a topology point pair. The result is not a drawable line segment. It does not classify on-curve/off-curve combinations, insert implied on-curve points, build quadratic segments, decide winding, or rasterize pixels.

```text
GuiSfntSimpleGlyphContourEdge:
    start GuiSfntSimpleGlyphContourPoint
    end GuiSfntSimpleGlyphContourPoint
    edge_index i32
    next_contour_point_index i32
```

Public lookup:

```text
gui_sfnt_lookup_simple_glyph_contour_edge:
    &ByteBuf -> Option i32 -> GuiGlyphId -> i32 -> i32
    -> Result GuiSfntSimpleGlyphContourEdge GuiSfntParseError
```

The lookup flow is:

```text
parse metadata
    -> unwrap head / loca / glyf
    -> gui_sfnt_glyf_simple_contour_span_with_tables
    -> validate edge_index against span.point_count
    -> next_contour_point_index = wrap(edge_index + 1, span.point_count)
    -> gui_sfnt_glyf_simple_contour_point_with_tables for start
    -> gui_sfnt_glyf_simple_contour_point_with_tables for end
    -> return GuiSfntSimpleGlyphContourEdge
```

`edge_index` is contour-local and must equal `start.contour_point_index`. `next_contour_point_index` must equal `end.contour_point_index`. The nested `start.point.point_index` and `end.point.point_index` remain absolute within the glyph.

The implementation must validate `edge_index` before decoding either endpoint. This keeps the F4k public contract stable: a valid glyph with a missing requested edge returns `MissingGlyphOutline` instead of exposing a later point decode or coordinate error.

One-point contours return an explicit self-wrapping topology edge:

```text
edge_index = 0
next_contour_point_index = 0
start.contour_point_index = 0
end.contour_point_index = 0
start.point.point_index = end.point.point_index
```

This only preserves contour topology. The later segment builder decides whether the self-wrap contributes a visible segment or an unsupported outline condition.

F4k uses internal table helpers, not public wrappers, after metadata is parsed:

```text
gui_sfnt_glyf_simple_contour_span_with_tables
gui_sfnt_glyf_simple_contour_point_with_tables
```

F4k must not allocate `Vec GuiSfntSimpleGlyphContourEdge`, call platform font APIs, use host text measurement, build full outlines, or substitute another rendering path.

### SFNT simple glyph curve segment classification

F4l is the first drawable-shape classification layer above contour topology. It still does not own an outline, allocate a segment list, decide winding, rasterize masks, or call platform text APIs. It classifies exactly one edge start.

The type model uses enum payloads instead of a shared struct with inactive fields:

```text
GuiSfntSimpleGlyphCurveNoSegmentReason:
    SinglePointContour
    OffCurveStart
    MissingLookahead

GuiSfntSimpleGlyphCurveNoSegment:
    edge GuiSfntSimpleGlyphContourEdge
    reason GuiSfntSimpleGlyphCurveNoSegmentReason

GuiSfntSimpleGlyphLineSegment:
    edge GuiSfntSimpleGlyphContourEdge
    start_x2 i32
    start_y2 i32
    end_x2 i32
    end_y2 i32

GuiSfntSimpleGlyphQuadraticSegment:
    edge GuiSfntSimpleGlyphContourEdge
    lookahead GuiSfntSimpleGlyphContourPoint
    start_x2 i32
    start_y2 i32
    control_x2 i32
    control_y2 i32
    end_x2 i32
    end_y2 i32
    end_is_implied bool

GuiSfntSimpleGlyphCurveSegment:
    NoSegment GuiSfntSimpleGlyphCurveNoSegment
    Line GuiSfntSimpleGlyphLineSegment
    Quadratic GuiSfntSimpleGlyphQuadraticSegment
```

Coordinate fields are doubled font units:

```text
point coordinate:
    x2 = x * 2

explicit quadratic end:
    end_x2 = lookahead.x * 2
    end_y2 = lookahead.y * 2

implied quadratic end:
    end_x2 = control.x + lookahead.x
    end_y2 = control.y + lookahead.y
```

The implementation must not compute implied midpoint with integer division. A midpoint such as `(1 + 2) / 2` is representable as `end_x2 = 3`, not rounded to `1` or `2`.

Pure classifier flow:

```text
gui_sfnt_classify_simple_glyph_curve_segment edge lookahead
    -> read start/end/span from edge
    -> if span.point_count == 1
        return NoSegment SinglePointContour
    -> if start is off-curve
        return NoSegment OffCurveStart
    -> if end is on-curve
        return Line with doubled start/end coordinates
    -> if lookahead is None
        return NoSegment MissingLookahead
    -> if lookahead is on-curve
        return Quadratic with explicit doubled lookahead end
    -> otherwise
        return Quadratic with implied doubled midpoint end
```

Byte lookup flow:

```text
parse metadata
    -> unwrap head / loca / glyf
    -> gui_sfnt_glyf_simple_contour_edge_with_tables
    -> read start/end point flags from edge
    -> if start is on-curve and end is off-curve
        -> compute lookahead_contour_point_index = wrap(edge.next_contour_point_index + 1)
        -> gui_sfnt_glyf_simple_contour_point_with_tables for lookahead
        -> gui_sfnt_classify_simple_glyph_curve_segment edge (Some lookahead)
    -> otherwise
        -> gui_sfnt_classify_simple_glyph_curve_segment edge None
```

This deliberate conditional lookahead avoids surfacing unrelated later coordinate corruption for an edge that is already a line, a one-point no-segment, or an off-curve-start no-segment.

`NoSegment` is a successful classification state. Out-of-range `contour_index` / `edge_index` and malformed bytes remain `Result::Err GuiSfntParseError`. The classifier must not convert unsupported shape semantics into silent fallback drawing.

F4l uses internal table helpers, not public wrappers, after metadata is parsed:

```text
gui_sfnt_glyf_simple_contour_edge_with_tables
gui_sfnt_glyf_simple_contour_point_with_tables
```

F4l must not allocate `Vec GuiSfntSimpleGlyphCurveSegment`, use integer midpoint division, call platform font APIs, use host text measurement, build full outlines, or rasterize pixels.

### SFNT simple glyph path command projection

F4m is the first sink-facing projection layer above curve segment classification. It still does not allocate a full outline, own a path sink trait, decide winding/fill rules, emit render2d commands, or rasterize pixels. It maps exactly one `GuiSfntSimpleGlyphCurveSegment` to explicit move and draw command values.

The type model uses enum payloads:

```text
GuiSfntSimpleGlyphPathMoveTo:
    contour_index i32
    edge_index i32
    x2 i32
    y2 i32

GuiSfntSimpleGlyphPathLineTo:
    contour_index i32
    edge_index i32
    x2 i32
    y2 i32

GuiSfntSimpleGlyphPathQuadraticTo:
    contour_index i32
    edge_index i32
    control_x2 i32
    control_y2 i32
    end_x2 i32
    end_y2 i32
    end_is_implied bool

GuiSfntSimpleGlyphPathSkipNoSegment:
    contour_index i32
    edge_index i32
    reason GuiSfntSimpleGlyphCurveNoSegmentReason

GuiSfntSimpleGlyphPathCommand:
    MoveTo GuiSfntSimpleGlyphPathMoveTo
    LineTo GuiSfntSimpleGlyphPathLineTo
    QuadraticTo GuiSfntSimpleGlyphPathQuadraticTo
    SkipNoSegment GuiSfntSimpleGlyphPathSkipNoSegment
```

Projection flow:

```text
gui_sfnt_simple_glyph_curve_segment_move_to_command segment
    -> Line: MoveTo line.start_x2 line.start_y2
    -> Quadratic: MoveTo quadratic.start_x2 quadratic.start_y2
    -> NoSegment: SkipNoSegment no_segment

gui_sfnt_simple_glyph_curve_segment_draw_command segment
    -> Line: LineTo line.end_x2 line.end_y2
    -> Quadratic: QuadraticTo control/end doubled coordinates
    -> NoSegment: SkipNoSegment no_segment
```

There is no command index in this API. Returning `GuiSfntSimpleGlyphPathCommand` directly keeps the boundary pure while avoiding an invalid-index state that would otherwise need `Option` or `Result`. The caller chooses the move phase or draw phase explicitly.

`SkipNoSegment` is a typed command, not fallback drawing and not silent ignore. A later streaming sink may count, log, or explicitly skip it by matching `GuiSfntSimpleGlyphPathCommand::SkipNoSegment`.

The command payload must be compact. F4m does not copy the whole edge, line segment, quadratic segment, or no-segment value into the command. It projects those values to the source contour/edge index, doubled coordinates, and no-segment reason that a later sink needs. This keeps the abstraction cheap and prevents deeply nested payloads from becoming the path interface.

F4m must not allocate `Vec GuiSfntSimpleGlyphPathCommand`, call `gui_sfnt_parse_metadata`, call platform font APIs, use host text measurement, import render2d/backend modules, build full glyph outlines, or rasterize pixels.

### SFNT simple glyph path command public lookup

F4n is a thin public composition layer over F4l and F4m. It takes the same byte-backed lookup input shape as the existing curve segment public lookup and returns the path command value needed by the next layer.

```text
gui_sfnt_lookup_simple_glyph_move_to_command bytes face_index glyph contour_index edge_index
    -> gui_sfnt_lookup_simple_glyph_curve_segment bytes face_index glyph contour_index edge_index
    -> gui_sfnt_simple_glyph_curve_segment_move_to_command segment

gui_sfnt_lookup_simple_glyph_draw_command bytes face_index glyph contour_index edge_index
    -> gui_sfnt_lookup_simple_glyph_curve_segment bytes face_index glyph contour_index edge_index
    -> gui_sfnt_simple_glyph_curve_segment_draw_command segment
```

The implementation must not call `gui_sfnt_parse_metadata`, `gui_sfnt_glyf_simple_curve_segment_with_tables`, lower point/contour table helpers, renderer APIs, rasterizers, host text measurement, or platform APIs. Those responsibilities already belong to earlier lookup layers or later rendering phases.

Error flow is one-to-one with F4l byte-backed lookup:

```text
Result::Err parse_error
    -> Result::Err parse_error

Result::Ok segment
    -> Result::Ok path_command
```

`NoSegment` remains a successful path command state. Both public F4n helpers map it to `SkipNoSegment`; they do not return `Option::None`, synthesize an empty command, or silently ignore the edge.

### SFNT simple glyph path command pair lookup

F4o is a single-edge pair boundary. It is not a contour stream and does not define a command sequence for a full outline. The pair is an O(1) value that carries the two explicit phases already defined by F4m.

```text
GuiSfntSimpleGlyphPathCommandPair:
    move_command GuiSfntSimpleGlyphPathCommand
    draw_command GuiSfntSimpleGlyphPathCommand
```

Pure projection:

```text
gui_sfnt_simple_glyph_curve_segment_path_command_pair segment
    -> move = gui_sfnt_simple_glyph_curve_segment_move_to_command segment
    -> draw = gui_sfnt_simple_glyph_curve_segment_draw_command segment
    -> GuiSfntSimpleGlyphPathCommandPair move draw
```

Byte-backed public lookup:

```text
gui_sfnt_lookup_simple_glyph_path_command_pair bytes face_index glyph contour_index edge_index
    -> gui_sfnt_lookup_simple_glyph_curve_segment bytes face_index glyph contour_index edge_index
    -> gui_sfnt_simple_glyph_curve_segment_path_command_pair segment
```

The public byte-backed helper must call `gui_sfnt_lookup_simple_glyph_curve_segment` exactly once. It must not call the separate move and draw public lookup helpers because that would decode the same SFNT edge twice. It must not call `gui_sfnt_parse_metadata`, `gui_sfnt_glyf_simple_curve_segment_with_tables`, lower public lookup helpers, the curve classifier, renderer APIs, rasterizers, host text measurement, or platform APIs.

The pair is not a list. F4o does not expose `command_index`, `count`, `next`, mutable current point state, or `Vec GuiSfntSimpleGlyphPathCommand`. A later contour/path sink can choose how to consume the pair values, but F4o does not define contour closure or off-curve contour-start synthesis.

`NoSegment` remains explicit. Both `move_command` and `draw_command` are `SkipNoSegment`, preserving the reason value for later diagnostic, skip counting, or sink behavior.

### SFNT simple glyph path sink event adapter

F4p is a single-edge adapter from the F4o command pair to the event pair a later contour/path sink can consume. It is still not the real sink. It does not define contour stream traversal, full outline command order, contour closure, off-curve contour-start synthesis, winding, fill rules, rasterization, render2d commands, host text measurement, or platform presentation.

The adapter intentionally reuses the existing compact command enum instead of defining another path representation:

```text
GuiSfntSimpleGlyphPathSinkEvent:
    Command GuiSfntSimpleGlyphPathCommand

GuiSfntSimpleGlyphPathSinkEventPair:
    first_event GuiSfntSimpleGlyphPathSinkEvent
    second_event GuiSfntSimpleGlyphPathSinkEvent
```

The pure command wrapper is total:

```text
gui_sfnt_simple_glyph_path_command_sink_event command
    -> GuiSfntSimpleGlyphPathSinkEvent::Command command
```

The pair adapter uses only the F4o accessors:

```text
gui_sfnt_simple_glyph_path_command_pair_sink_event_pair pair
    -> move_command = gui_sfnt_simple_glyph_path_command_pair_move_command pair
    -> draw_command = gui_sfnt_simple_glyph_path_command_pair_draw_command pair
    -> first_event = gui_sfnt_simple_glyph_path_command_sink_event move_command
    -> second_event = gui_sfnt_simple_glyph_path_command_sink_event draw_command
    -> GuiSfntSimpleGlyphPathSinkEventPair first_event second_event
```

F4p must not return `Option` or `Result` from the pure adapter because a valid `GuiSfntSimpleGlyphPathCommandPair` already contains both commands. It must not call byte-backed lookup helpers, metadata parsers, `*_with_tables` helpers, lower point / contour helpers, or the curve classifier. It must not allocate `Vec GuiSfntSimpleGlyphPathSinkEvent`, expose `command_index`, `count`, `next`, or carry mutable current point state.

`SkipNoSegment` remains a typed event by wrapping the existing `GuiSfntSimpleGlyphPathCommand::SkipNoSegment` value. This keeps later sink behavior explicit without changing parse status.

### SFNT simple glyph path sink event kind classification

F4q adds a dispatch classification boundary above the F4p event wrapper. It is not a replacement path representation and not a compact payload enum. The authority for coordinates, contour index, edge index, and skip source remains the wrapped `GuiSfntSimpleGlyphPathCommand`.

```text
GuiSfntSimpleGlyphPathSinkEventKind:
    MoveTo
    LineTo
    QuadraticTo
    SkipNoSegment GuiSfntSimpleGlyphCurveNoSegmentReason

GuiSfntSimpleGlyphPathSinkEventKindPair:
    first_kind GuiSfntSimpleGlyphPathSinkEventKind
    second_kind GuiSfntSimpleGlyphPathSinkEventKind
```

The event kind helper is total and uses exhaustive `match` on the wrapped command:

```text
gui_sfnt_simple_glyph_path_sink_event_kind event
    -> command = gui_sfnt_simple_glyph_path_sink_event_command event
    -> match command:
        MoveTo _ -> MoveTo
        LineTo _ -> LineTo
        QuadraticTo _ -> QuadraticTo
        SkipNoSegment skip -> SkipNoSegment (reason skip)
```

`SkipNoSegment` carries only `GuiSfntSimpleGlyphCurveNoSegmentReason`. That reason is suitable for diagnostics, explicit skip counting, and branch selection, but it is not enough to recover the source contour/edge. Callers that need source indices or coordinates must read the original command payload from `GuiSfntSimpleGlyphPathSinkEvent`.

The kind pair helper must use only F4p event pair accessors and `gui_sfnt_simple_glyph_path_sink_event_kind`:

```text
gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair pair
    -> first_event = gui_sfnt_simple_glyph_path_sink_event_pair_first_event pair
    -> second_event = gui_sfnt_simple_glyph_path_sink_event_pair_second_event pair
    -> first_kind = gui_sfnt_simple_glyph_path_sink_event_kind first_event
    -> second_kind = gui_sfnt_simple_glyph_path_sink_event_kind second_event
    -> GuiSfntSimpleGlyphPathSinkEventKindPair first_kind second_kind
```

F4q must not add `contour_index`, `edge_index`, coordinate fields, control/end fields, a real sink trait, allocation ownership, contour stream traversal, contour closure, off-curve contour-start synthesis, winding, fill rules, rasterization, render2d commands, host text measurement, or platform presentation. The pure helpers must not return `Option` or `Result`, allocate `Vec GuiSfntSimpleGlyphPathSinkEventKind`, expose `command_index`, `count`, `next`, or call byte-backed lookup helpers, metadata parsers, `*_with_tables` helpers, lower point / contour helpers, or the curve classifier.

### SFNT simple glyph path sink event indexed selection

F4r adds typed slot selection over the F4p/F4q two-slot values. It is not a contour iterator, not a command sequence, and not a stream cursor. The goal is to keep the next sink layer from introducing numeric indexes while still letting it choose the first or second event of one already-decoded edge pair.

```text
GuiSfntSimpleGlyphPathSinkEventSlot:
    First
    Second

gui_sfnt_simple_glyph_path_sink_event_pair_event_at pair slot:
    match slot:
        First -> gui_sfnt_simple_glyph_path_sink_event_pair_first_event pair
        Second -> gui_sfnt_simple_glyph_path_sink_event_pair_second_event pair

gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at pair slot:
    match slot:
        First -> gui_sfnt_simple_glyph_path_sink_event_kind_pair_first_kind pair
        Second -> gui_sfnt_simple_glyph_path_sink_event_kind_pair_second_kind pair

gui_sfnt_simple_glyph_path_sink_event_pair_kind_at pair slot:
    event = gui_sfnt_simple_glyph_path_sink_event_pair_event_at pair slot
    gui_sfnt_simple_glyph_path_sink_event_kind event
```

The slot enum is the only selector. F4r must not accept an `i32` event index, because a numeric index would reintroduce impossible states that have to be reported at runtime. `First` and `Second` are exhaustive; helpers therefore must not return `Option` or `Result`.

`event_pair_kind_at` deliberately composes `event_at` and the F4q kind helper. It must not duplicate kind classification logic, build a `GuiSfntSimpleGlyphPathSinkEventKindPair`, read path coordinates, or call byte-backed glyph lookup. `kind_pair_kind_at` deliberately uses only the kind pair first / second accessors.

F4r must not allocate `Vec GuiSfntSimpleGlyphPathSinkEvent`, allocate `Vec GuiSfntSimpleGlyphPathSinkEventKind`, use `push`, expose `command_index`, `count`, `next`, mutable current point state, contour traversal, contour closure, off-curve contour-start synthesis, winding, fill rules, rasterization, render2d commands, host text measurement, or platform presentation.

### SFNT simple glyph path contour traversal step

F4s adds the first cursor-shaped traversal boundary above F4r. It advances exactly one sink event in a simple glyph contour. It is still not a full sink trait, not a path builder, not a `Vec` command stream, and not a renderer. Its responsibility is to make the next contour state explicit with enum data so that later sink ownership can remain deterministic and testable.

```text
GuiSfntSimpleGlyphPathContourCursor:
    glyph GuiGlyphId
    contour_index i32
    edge_index i32
    slot GuiSfntSimpleGlyphPathSinkEventSlot

GuiSfntSimpleGlyphPathContourNext:
    Continue GuiSfntSimpleGlyphPathContourCursor
    EndContour

GuiSfntSimpleGlyphPathContourStep:
    cursor GuiSfntSimpleGlyphPathContourCursor
    event GuiSfntSimpleGlyphPathSinkEvent
    kind GuiSfntSimpleGlyphPathSinkEventKind
    next GuiSfntSimpleGlyphPathContourNext
```

The public lookup is:

```text
gui_sfnt_lookup_simple_glyph_path_contour_step bytes face_index cursor
    -> glyph = cursor.glyph
    -> contour_index = cursor.contour_index
    -> edge_index = cursor.edge_index
    -> slot = cursor.slot
    -> span = gui_sfnt_lookup_simple_glyph_contour_span bytes face_index glyph contour_index
    -> pair = gui_sfnt_lookup_simple_glyph_path_command_pair bytes face_index glyph contour_index edge_index
    -> event_pair = gui_sfnt_simple_glyph_path_command_pair_sink_event_pair pair
    -> event = gui_sfnt_simple_glyph_path_sink_event_pair_event_at event_pair slot
    -> kind = gui_sfnt_simple_glyph_path_sink_event_kind event
    -> next = private validated cursor-next helper using span.point_count
    -> GuiSfntSimpleGlyphPathContourStep cursor event kind next
```

The next rule is fixed:

```text
First  -> Continue same glyph / same contour / same edge / Second
Second -> Continue same glyph / same contour / next edge / First
Second on final edge -> EndContour
```

`EndContour` is a successful step state, not an error and not `Option::None`. The lookup returns `Result::Err` only for parse, missing glyph, missing contour, or invalid edge range errors already reported by the lower checked lookups.

The next helper must remain private unless it is changed to return `Result`. As a private helper, it may assume the public lookup has already checked `span_point_count > 0` and `0 <= edge_index < span_point_count`. Exposing a total public helper over unchecked raw cursor data would make invalid states appear statically valid, which conflicts with the static-checking policy.

F4s deliberately leaves off-curve contour-start synthesis unchanged. The current `SkipNoSegment OffCurveStart` event remains a typed event. Contour closure insertion, actual sink ownership, outline allocation, path repair, rasterization, render2d command emission, font fallback, and platform presentation are later phases.

### SFNT simple glyph allocation-free path sink ownership boundary

F4t turns an F4s contour step into a sink-facing ownership decision. It is deliberately still a one-step value transformation. It does not own a real sink trait, allocate a path list, build an outline object, apply fill rules, rasterize, or emit render2d commands.

The policy is split into two independent axes:

```text
GuiSfntSimpleGlyphPathOffCurveStartPolicy:
    KeepTypedSkip
    RejectUnsupported

GuiSfntSimpleGlyphPathClosurePolicy:
    KeepOpen
    EmitCloseAfterFinalEvent

GuiSfntSimpleGlyphPathSinkPolicy:
    off_curve_start_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy
    closure_policy GuiSfntSimpleGlyphPathClosurePolicy
```

`off_curve_start_policy` only applies to `GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment OffCurveStart`. It must not reject `SinglePointContour` or `MissingLookahead`, because those states are already typed F4l/F4s success states and do not mean the sink is being asked to synthesize an implied contour start.

The sink decision is represented as data:

```text
GuiSfntSimpleGlyphPathSinkRejectReason:
    UnsupportedOffCurveStart

GuiSfntSimpleGlyphPathSinkPrimaryAction:
    EmitEvent GuiSfntSimpleGlyphPathSinkEvent
    Reject GuiSfntSimpleGlyphPathSinkRejectReason

GuiSfntSimpleGlyphPathContourClose:
    glyph GuiGlyphId
    contour_index i32

GuiSfntSimpleGlyphPathSinkTailAction:
    NoTailAction
    CloseContour GuiSfntSimpleGlyphPathContourClose

GuiSfntSimpleGlyphPathSinkStep:
    source_step GuiSfntSimpleGlyphPathContourStep
    primary_action GuiSfntSimpleGlyphPathSinkPrimaryAction
    tail_action GuiSfntSimpleGlyphPathSinkTailAction
```

`Reject` is not `GuiSfntParseError`. Parse/range failure is still the responsibility of F4s lookup and remains the only reason the byte-backed F4t helper returns `Result::Err GuiSfntParseError`. Policy rejection stays in the successful payload so caller code can distinguish malformed font data from a configured sink capability refusal.

The tail action depends on both the primary action and the F4s next state. This avoids the ambiguous state where a rejected final event also asks the sink to close the contour.

```text
if primary_action is Reject:
    tail_action = NoTailAction

else if step.next is Continue:
    tail_action = NoTailAction

else if step.next is EndContour and closure_policy is KeepOpen:
    tail_action = NoTailAction

else if step.next is EndContour and closure_policy is EmitCloseAfterFinalEvent:
    tail_action = CloseContour glyph contour_index
```

`CloseContour` uses the glyph and contour index from `step.cursor`. It is a marker for the next sink layer, not a renderer command and not a coordinate-bearing path segment. The future real sink can map it to its own close-path operation or reject unsupported closure semantics with a separate typed result.

The public pure helper is:

```text
gui_sfnt_simple_glyph_path_sink_step_from_contour_step policy step
    -> primary = gui_sfnt_simple_glyph_path_sink_primary_action_from_contour_step policy step
    -> tail = gui_sfnt_simple_glyph_path_sink_tail_action_from_contour_step policy step primary
    -> GuiSfntSimpleGlyphPathSinkStep step primary tail
```

The byte-backed helper is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_step bytes face_index cursor policy
    -> contour_step = gui_sfnt_lookup_simple_glyph_path_contour_step bytes face_index cursor
    -> gui_sfnt_simple_glyph_path_sink_step_from_contour_step policy contour_step
```

It must not re-parse metadata itself, call internal table helpers directly, bypass F4s, allocate `Vec`, call renderer/platform APIs, rasterize, or consult host font fallback. The helper is only the boundary where checked bytes become a checked one-step sink decision.

### SFNT simple glyph path sink action selection projection

F4u projects an F4t sink step into one explicitly selected action. It is a selection/projection layer for a future sink. It does not mutate a sink, store a sink trait object, allocate an outline list, define callback ownership, repair paths, rasterize, emit render2d commands, or present to a platform backend.

The action slot is a different axis from the event slot:

```text
GuiSfntSimpleGlyphPathSinkEventSlot:
    First
    Second

GuiSfntSimpleGlyphPathSinkActionSlot:
    Primary
    Tail
```

`First` / `Second` selects one of the two command events generated for a contour edge. `Primary` / `Tail` selects one of the two F4t actions attached to an already materialized `GuiSfntSimpleGlyphPathSinkStep`. F4u must not collapse these slots into an integer, a shared enum, or a command index.

The unified action value is:

```text
GuiSfntSimpleGlyphPathSinkAction:
    EmitEvent GuiSfntSimpleGlyphPathSinkEvent
    Reject GuiSfntSimpleGlyphPathSinkRejectReason
    CloseContour GuiSfntSimpleGlyphPathContourClose
    NoAction
```

`NoAction` is only the explicit projection of `GuiSfntSimpleGlyphPathSinkTailAction::NoTailAction`. It is not a fallback path, not an unsupported-operation success, and not a hidden no-op. Primary action projection must never return `NoAction`.

Pure projection helpers:

```text
gui_sfnt_simple_glyph_path_sink_primary_action_as_action primary_action
    EmitEvent event -> GuiSfntSimpleGlyphPathSinkAction::EmitEvent event
    Reject reason -> GuiSfntSimpleGlyphPathSinkAction::Reject reason

gui_sfnt_simple_glyph_path_sink_tail_action_as_action tail_action
    NoTailAction -> GuiSfntSimpleGlyphPathSinkAction::NoAction
    CloseContour close -> GuiSfntSimpleGlyphPathSinkAction::CloseContour close

gui_sfnt_simple_glyph_path_sink_step_action_at step slot
    Primary -> primary_action_as_action step.primary_action
    Tail -> tail_action_as_action step.tail_action
```

`gui_sfnt_simple_glyph_path_sink_step_action_at` is intentionally total over the two slot variants. It must not return `Option` or `Result`, expose `command_index`, accept a numeric action index, use a default arm, allocate `Vec`, call byte-backed lookup helpers, or duplicate F4t policy rules. It only reads the existing F4t action values and projects them into a unified action type.

The byte-backed helper is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action bytes face_index cursor policy slot
    -> sink_step = gui_sfnt_lookup_simple_glyph_path_sink_step bytes face_index cursor policy
    -> gui_sfnt_simple_glyph_path_sink_step_action_at sink_step slot
```

The byte-backed helper must call `gui_sfnt_lookup_simple_glyph_path_sink_step` exactly once. It must not call `gui_sfnt_lookup_simple_glyph_path_contour_step`, `gui_sfnt_lookup_simple_glyph_path_command_pair`, lower contour/curve helpers, `gui_sfnt_parse_metadata`, internal `*_with_tables` helpers, renderer/platform APIs, rasterizers, host font measurement, or font fallback. Its only failure channel remains the F4t lookup's `Result::Err GuiSfntParseError`; policy rejection remains `Result::Ok GuiSfntSimpleGlyphPathSinkAction::Reject`.

### SFNT simple glyph path sink action traversal step

F4v turns F4u action selection into a typed traversal step. It is still a contour-local value model, not a real sink. It does not mutate a sink, define callback ownership, allocate a command list, construct a full outline, repair paths, compute winding, rasterize, emit render2d commands, or present to a platform backend.

The traversal cursor combines the existing contour event cursor and the action slot:

```text
GuiSfntSimpleGlyphPathSinkActionCursor:
    contour_cursor GuiSfntSimpleGlyphPathContourCursor
    action_slot GuiSfntSimpleGlyphPathSinkActionSlot
```

The cursor intentionally carries the validated F4s contour cursor. It therefore contains the existing `contour_index` and `edge_index` through that nested value. F4v must not add a new numeric action index, command index, loop index, count field, or ad-hoc traversal counter. The action position is always the enum `GuiSfntSimpleGlyphPathSinkActionSlot`.

The next state is:

```text
GuiSfntSimpleGlyphPathSinkActionNext:
    Continue GuiSfntSimpleGlyphPathSinkActionCursor
    EndContour
```

`EndContour` is a successful terminal state of the contour-local action stream. It is not `Option::None`, not `Result::Err`, and not an implicit ignored event.

The step value is:

```text
GuiSfntSimpleGlyphPathSinkActionStep:
    cursor GuiSfntSimpleGlyphPathSinkActionCursor
    sink_step GuiSfntSimpleGlyphPathSinkStep
    action GuiSfntSimpleGlyphPathSinkAction
    next GuiSfntSimpleGlyphPathSinkActionNext
```

`action` and `next` are separate facts. `action` says what a future sink consumes. `next` says where traversal continues. Next-state computation must not inspect whether the action is `EmitEvent`, `Reject`, `CloseContour`, or `NoAction`.

Pure traversal rules:

```text
gui_sfnt_simple_glyph_path_sink_action_next_from_step sink_step action_slot
    Primary:
        source_step.cursor -> Continue same contour_cursor Tail

    Tail:
        source_step.next = Continue next_cursor
            -> Continue next_cursor Primary

        source_step.next = EndContour
            -> EndContour
```

This means `Primary -> Tail` happens even when the primary action is `Reject`. It also means `Tail -> source_step.next` happens even when the tail action is `NoAction`. Reject handling and no-action handling belong to the consumer of the action payload, not to traversal.

`gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step` must compose F4u rather than duplicate it:

```text
gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step sink_step action_slot
    -> cursor = GuiSfntSimpleGlyphPathSinkActionCursor source_step.cursor action_slot
    -> action = gui_sfnt_simple_glyph_path_sink_step_action_at sink_step action_slot
    -> next = gui_sfnt_simple_glyph_path_sink_action_next_from_step sink_step action_slot
    -> GuiSfntSimpleGlyphPathSinkActionStep cursor sink_step action next
```

The byte-backed helper is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index cursor policy
    -> contour_cursor = cursor.contour_cursor
    -> action_slot = cursor.action_slot
    -> sink_step = gui_sfnt_lookup_simple_glyph_path_sink_step bytes face_index contour_cursor policy
    -> gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step sink_step action_slot
```

The byte-backed helper must call `gui_sfnt_lookup_simple_glyph_path_sink_step` exactly once. It must not call F4s contour-step lookup, path command pair lookup, lower contour/curve helpers, `gui_sfnt_parse_metadata`, internal `*_with_tables` helpers, renderer/platform APIs, rasterizers, host font measurement, or font fallback.

### SFNT simple glyph path sink action start cursor

F4w adds the entry position for the F4v action traversal stream. The start cursor is not a glyph iterator, not a sink, not a first action lookup, and not a renderer command. It is a typed value that names the first action slot of one contour:

```text
contour cursor:
    glyph = input glyph
    contour_index = input contour_index
    edge_index = 0
    event_slot = First

action cursor:
    contour_cursor = contour cursor
    action_slot = Primary
```

The pure constructor has no byte access:

```text
gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph contour_index
    -> gui_sfnt_simple_glyph_path_contour_cursor glyph contour_index 0 First
    -> gui_sfnt_simple_glyph_path_sink_action_cursor contour_cursor Primary
```

This constructor is intentionally unchecked. It must not claim that the contour exists, that the contour has at least one point, or that a glyph record is well-formed. Those facts belong to the byte-backed entry point:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor bytes face_index glyph contour_index
    -> gui_sfnt_lookup_simple_glyph_contour_span bytes face_index glyph contour_index
    -> gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph contour_index
```

The byte-backed entry point validates only the contour span boundary by calling `gui_sfnt_lookup_simple_glyph_contour_span` exactly once. It must not call F4v action-step lookup, F4t sink-step lookup, F4s contour-step lookup, point/curve/path-command helpers, policy helpers, full outline allocation, renderer/platform APIs, rasterizers, host font measurement, or font fallback. This keeps F4w as the start-cursor authority and leaves action payload traversal to F4v.

### SFNT simple glyph path sink action start step

F4x adds the first-step entry point for callers that need the first action step rather than only the start cursor. It is a thin composition:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_step bytes face_index glyph contour_index policy
    -> start_cursor = gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph contour_index
    -> gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index start_cursor policy
```

This helper deliberately does not call `gui_sfnt_lookup_simple_glyph_path_sink_action_start_cursor`. The byte-backed start cursor helper validates contour span for cursor-only callers, while the action-step lookup already validates the same contour through the F4v -> F4t -> F4s path. Calling both would duplicate the contour span validation and would make F4x look like a new validation authority. F4x is not a new authority; it only connects the unchecked start cursor value to the existing checked action-step lookup.

Error and policy taxonomy are inherited unchanged:

```text
parse/range/table error
    -> Result::Err GuiSfntParseError

policy reject
    -> Result::Ok step where step.action = Reject reason
```

F4x must not call `gui_sfnt_lookup_simple_glyph_contour_span`, `gui_sfnt_lookup_simple_glyph_path_sink_step`, F4s contour-step lookup, lower point/curve/path helpers, metadata parser, internal table helpers, renderer/platform APIs, rasterizers, host font measurement, or font fallback.

### SFNT simple glyph path sink action step advance

F4y resolves one `GuiSfntSimpleGlyphPathSinkActionNext` value into either a checked next action step or a typed contour terminal state:

```text
GuiSfntSimpleGlyphPathSinkActionStepAdvance:
    Continue GuiSfntSimpleGlyphPathSinkActionStep
    EndContour
```

The type is separate from `GuiSfntSimpleGlyphPathSinkActionNext` because `Next` contains only the next cursor, while `StepAdvance` contains the byte-backed lookup result for that cursor. Returning `Option GuiSfntSimpleGlyphPathSinkActionStep` would lose the domain reason for termination, and returning `Result::Err` for `EndContour` would confuse a successful terminal state with malformed font data.

The helper is a one-step state transition:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance bytes face_index step policy
    -> next = gui_sfnt_simple_glyph_path_sink_action_step_next step
    -> match next
        Continue cursor:
            gui_sfnt_lookup_simple_glyph_path_sink_action_step bytes face_index cursor policy
                Err error -> Err error
                Ok next_step -> Ok Continue next_step
        EndContour:
            Ok EndContour
```

F4y does not inspect `step.action`, primary action, tail action, or unified action payload variants. `Reject`, `NoAction`, and `CloseContour` are payloads for a future consumer; they do not change the traversal rule. F4y must not loop, allocate a command list, mutate a sink, repair contours, rasterize, or present to any platform.

### SFNT simple glyph path sink action step item

F4z packages the current action step and the checked advance result into a single value:

```text
GuiSfntSimpleGlyphPathSinkActionStepItem:
    step GuiSfntSimpleGlyphPathSinkActionStep
    advance GuiSfntSimpleGlyphPathSinkActionStepAdvance
```

This is the first consumer-facing item boundary above F4y. It is deliberately not a contour iterator, not a sink trait, and not a command list. A later sink consumer can read `item.step.action` through the existing step accessor and use `item.advance` to decide whether the next item exists, but F4z itself does not interpret the action payload.

The byte-backed helper is a narrow composition:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index step policy
    -> gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance bytes face_index step policy
        Err error -> Err error
        Ok advance:
            stored_step = *step
            Ok ActionStepItem stored_step advance
```

The explicit `stored_step = *step` copy is part of the contract. The helper must not store a borrowed reference in the item, because the item is a value that can be passed to later consumers without aliasing the caller-owned reference.

F4z must not call start cursor helpers, start step helpers, F4v action step lookup, sink action lookup, sink step lookup, contour step lookup, lower point / curve / path helpers, metadata parser, `*_with_tables` helpers, renderer/platform APIs, rasterizers, host text measurement, or font fallback. It must not allocate `Vec`, push into a command list, loop over a contour, inspect `Reject` / `NoAction` / `CloseContour`, or introduce a silent no-op path.

### SFNT simple glyph path sink action start item

F4aa adds a first-item entry point above F4x and F4z. It is useful for a future contour consumer that wants the first item directly, but it remains a narrow composition:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_item bytes face_index glyph contour_index policy
    -> gui_sfnt_lookup_simple_glyph_path_sink_action_start_step bytes face_index glyph contour_index policy
        Err error -> Err error
        Ok start_step:
            gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index &start_step policy
                Err error -> Err error
                Ok item -> Ok item
```

F4aa does not introduce a new data type. The result type is the F4z `GuiSfntSimpleGlyphPathSinkActionStepItem`, so the current step and checked advance remain the only item payload. This preserves the typed value boundary and avoids a parallel "start item" structure that would duplicate state.

The helper must call `gui_sfnt_lookup_simple_glyph_path_sink_action_start_step` exactly once and `gui_sfnt_lookup_simple_glyph_path_sink_action_step_item` exactly once. It must not call the pure start-cursor helper, the byte-backed start-cursor helper, F4v action-step lookup, F4y advance helper, sink action lookup, sink step lookup, contour step lookup, lower point / curve / path helpers, metadata parser, `*_with_tables` helpers, renderer/platform APIs, rasterizers, host text measurement, or font fallback.

F4aa itself does not inspect the action payload. `Reject`, `NoAction`, `CloseContour`, and `EndContour` remain typed states inside the F4x/F4z result path. This keeps parse/range/table failures as `Result::Err`, domain terminal states as enum values, and unsupported future behavior out of the helper instead of hiding it through fallback.

### SFNT simple glyph path sink action item next

F4ab resolves the checked advance already stored in an F4z/F4aa action item. It is deliberately a one-item boundary, not a contour iterator and not a real sink. Its only job is to turn the item's stored `GuiSfntSimpleGlyphPathSinkActionStepAdvance` into either the next checked item or the contour terminal state:

```text
GuiSfntSimpleGlyphPathSinkActionItemNext:
    Continue GuiSfntSimpleGlyphPathSinkActionStepItem
    EndContour
```

The public helper is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_item_next bytes face_index item policy
    -> advance = gui_sfnt_simple_glyph_path_sink_action_step_item_advance item
    -> match advance
        Continue next_step:
            gui_sfnt_lookup_simple_glyph_path_sink_action_step_item bytes face_index &next_step policy
                Err error -> Err error
                Ok next_item -> Ok Continue next_item

        EndContour:
            Ok EndContour
```

`EndContour` is not an error. It means that the contour-local action stream reached its successful terminal state. Returning `Option::None` would make this indistinguishable from "no value was produced", and returning `Result::Err` would confuse a valid terminal contour with malformed font data.

F4ab must not inspect `item.step.action` or any nested `EmitEvent` / `Reject` / `NoAction` / `CloseContour` payload. Action payloads are what a future sink consumes; item-next only follows the checked traversal state that F4z already computed. This preserves the separation between "what to consume" and "where traversal continues".

The helper must call `gui_sfnt_simple_glyph_path_sink_action_step_item_advance` exactly once. In the `Continue` branch it must call `gui_sfnt_lookup_simple_glyph_path_sink_action_step_item` exactly once. It must not call start helpers, action-step lookup, action-step advance lookup, sink action lookup, sink step lookup, contour step lookup, lower point / curve / path helpers, metadata parser, `*_with_tables` helpers, renderer/platform APIs, rasterizers, host text measurement, or font fallback. It must not allocate `Vec`, push into a command list, loop over a contour, or introduce a hidden no-op path.

### SFNT simple glyph path sink action consumer item

F4ac packages one checked action item for the future sink consumer. F4z stores the checked source item as `step + advance`, and F4ab turns `advance` into `Continue next_item` or `EndContour`. F4ac intentionally does not replace either boundary. It reads the current action from the stored step and composes it with F4ab's checked next state:

```text
GuiSfntSimpleGlyphPathSinkActionConsumerItem:
    action GuiSfntSimpleGlyphPathSinkAction
    next GuiSfntSimpleGlyphPathSinkActionItemNext
```

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index item policy:
    stored_step = gui_sfnt_simple_glyph_path_sink_action_step_item_step item
    action = gui_sfnt_simple_glyph_path_sink_action_step_action &stored_step

    match gui_sfnt_lookup_simple_glyph_path_sink_action_item_next bytes face_index item policy:
        Err error:
            Err error

        Ok next:
            Ok GuiSfntSimpleGlyphPathSinkActionConsumerItem action next
```

The explicit `stored_step` copy is part of the ownership contract. The consumer item stores values, not references to caller-owned storage. A later real sink can consume `action` and inspect `next` without relying on hidden mutable current-point state.

F4ac is not a real sink, not an iterator, and not a contour-wide consumer. It does not call a callback, mutate a sink, allocate a command list, construct an outline, decide fill rules, rasterize, emit render2d commands, or present to a platform backend. It also does not classify the action payload: `EmitEvent`, `Reject`, `NoAction`, and `CloseContour` remain data in `GuiSfntSimpleGlyphPathSinkAction`.

The helper must call `gui_sfnt_simple_glyph_path_sink_action_step_item_step` exactly once, `gui_sfnt_simple_glyph_path_sink_action_step_action` exactly once, and `gui_sfnt_lookup_simple_glyph_path_sink_action_item_next` exactly once. It must not call F4z action item lookup, F4y action advance lookup, F4v action step lookup, F4x/F4aa start helpers, sink step lookup, contour step lookup, lower point / curve / path helpers, metadata parser, `*_with_tables` helpers, renderer/platform APIs, rasterizers, host text measurement, or font fallback. It must not allocate `Vec`, push into a command list, loop over a contour, use numeric action indexes, or introduce hidden fallback.

### SFNT simple glyph path sink action consumer item next

F4ad advances a consumer item by one already checked continuation. It is deliberately above F4ac and below any real sink loop. The current item's action payload is not part of the traversal authority; only `item.next` decides whether a next consumer packet exists.

```text
GuiSfntSimpleGlyphPathSinkActionConsumerItemNext:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerItem
    EndContour
```

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next bytes face_index item policy:
    next = gui_sfnt_simple_glyph_path_sink_action_consumer_item_next item

    match next:
        Continue next_item:
            match gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index &next_item policy:
                Err error:
                    Err error

                Ok next_consumer_item:
                    Ok Continue next_consumer_item

        EndContour:
            Ok EndContour
```

`EndContour` remains a successful terminal domain state. Returning `Option::None` would hide the difference between "no value exists" and "the contour stream completed"; returning `Result::Err` would confuse valid terminal state with malformed font data.

F4ad is not a loop and not a sink. It does not consume actions, does not decide whether `Reject` stops rendering, does not turn `NoAction` into skip, and does not map `CloseContour` to a renderer command. Those decisions belong to a later real sink or explicit consumer policy.

The helper must call `gui_sfnt_simple_glyph_path_sink_action_consumer_item_next` exactly once and `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item` exactly once in the `Continue` branch. It must not call F4ab item-next lookup, F4z item lookup, F4y action advance lookup, F4v action step lookup, F4x/F4aa start helpers, sink step lookup, contour step lookup, lower point / curve / path helpers, metadata parser, `*_with_tables` helpers, renderer/platform APIs, rasterizers, host text measurement, or font fallback. It must not read `item.action`, match action payload variants, allocate `Vec`, push into a command list, loop over a contour, or introduce hidden fallback.

### SFNT simple glyph path sink action apply state

F4ae is the first boundary that consumes the current action payload. F4ad intentionally keeps traversal authority separate and does not inspect `item.action`; F4ae accepts one `GuiSfntSimpleGlyphPathSinkAction` value and records what was consumed in a pure value state.

```text
GuiSfntSimpleGlyphPathSinkActionApplyStatus:
    EmittedEvent GuiSfntSimpleGlyphPathSinkEvent
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    ClosedContour GuiSfntSimpleGlyphPathContourClose
    NoAction

GuiSfntSimpleGlyphPathSinkActionApplyState:
    emitted_event_count i32
    reject_count i32
    close_contour_count i32
    no_action_count i32

GuiSfntSimpleGlyphPathSinkActionApplyStep:
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    status GuiSfntSimpleGlyphPathSinkActionApplyStatus
```

```text
gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action state action:
    match action:
        EmitEvent event:
            state.emitted_event_count += 1
            status = EmittedEvent event

        Reject reason:
            state.reject_count += 1
            status = Rejected reason

        CloseContour close:
            state.close_contour_count += 1
            status = ClosedContour close

        NoAction:
            state.no_action_count += 1
            status = NoAction
```

Only one counter changes per call. The counter state is not a cursor, not a continuation token, and not a replacement for F4ac/F4ad checked next state. A future contour-wide consumer may pair F4ad traversal with F4ae consumption, but this phase does not own that loop.

`Rejected` is a domain status, not `Result::Err`. It comes from a sink policy action selected from valid contour data and must stay distinguishable from malformed SFNT bytes. `NoAction` is also a consumed status, not an implicit skip or hidden no-op. Tests must match both variants explicitly.

The implementation must stay allocation-free and side-effect-free. It must not call lookup helpers, parse metadata, allocate an outline, push render commands, inspect current point state, rasterize, call a renderer, call platform APIs, or perform host text measurement.

### SFNT simple glyph path sink action consumer apply step

F4af composes F4ac and F4ae without taking over F4ad traversal. A consumer item already contains the current action and the checked item-level continuation. F4af applies the current action to an apply state, then stores the resulting apply step next to the already stored continuation.

```text
GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep:
    apply_step GuiSfntSimpleGlyphPathSinkActionApplyStep
    next GuiSfntSimpleGlyphPathSinkActionItemNext
```

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_item_apply state item:
    action = gui_sfnt_simple_glyph_path_sink_action_consumer_item_action item
    next = gui_sfnt_simple_glyph_path_sink_action_consumer_item_next item
    apply_step = gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action state action
    GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep apply_step next
```

The helper is total because it receives typed values that have already crossed the byte-backed parsing boundary. Adding `Result` here would confuse malformed SFNT data with `Rejected` / `NoAction` domain status.

`next` is the stored `GuiSfntSimpleGlyphPathSinkActionItemNext` value from the current consumer item. F4af must not construct `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext` and must not call `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next`, because that would resolve the next consumer packet and move F4ad's byte-backed traversal responsibility into this phase.

F4af must not match `GuiSfntSimpleGlyphPathSinkAction` variants directly. Payload interpretation belongs to F4ae. It also must not allocate `Vec`, push a command, loop over a contour, track current point state, perform lower SFNT lookup, parse metadata, rasterize, render, call a platform backend, or call host text measurement.

### SFNT simple glyph path sink action consumer apply terminal

F4ag turns a single F4af apply step into a pure terminal classification for a future consumer loop. It deliberately does not become that loop. It also does not resolve the next byte-backed consumer item.

```text
GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
```

The helper reads the inner apply status first:

```text
status = gui_sfnt_simple_glyph_path_sink_action_apply_step_status step.apply_step

match status:
    Rejected reason:
        Rejected reason

    otherwise:
        match step.next:
            Continue _:
                Continue step

            EndContour:
                EndContour step
```

`Rejected` has priority over stored next state because a policy rejection is a domain terminal. It is not a parse error and must not be wrapped as `Result::Err`. `EndContour` is also a domain terminal, but successful. `Continue` keeps the already computed F4af apply step so the future loop can inspect status and counts without recomputing payload interpretation.

`NoAction` must not be treated as an implicit skip. Its terminal state is decided only by the stored next value. This preserves the distinction between "nothing was emitted by this action" and "the traversal has ended".

F4ag must not construct `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext`, must not call `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next`, and must not call byte-backed lower lookup helpers. It must not allocate `Vec`, push commands, loop over contour data, inspect current point state, parse metadata, rasterize, render, call platform APIs, or call host text measurement.

### SFNT simple glyph path sink action consumer apply advance

F4ah is the first post-apply one-step advance boundary. It combines F4ag's terminal classification with F4ac lookup through the stored `GuiSfntSimpleGlyphPathSinkActionItemNext`. It is not a direct F4ad call, because F4ag/F4af intentionally no longer carry the original `GuiSfntSimpleGlyphPathSinkActionConsumerItem`.

```text
GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerItem
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour
```

The helper shape is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance bytes face_index step policy:
    terminal = gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step step

    match terminal:
        Rejected reason:
            Ok Rejected reason

        EndContour _:
            Ok EndContour

        Continue continue_step:
            next = gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_next continue_step

            match next:
                Continue next_item:
                    gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index next_item policy
                        |> map Continue

                EndContour:
                    Ok EndContour
```

`Rejected` remains a domain terminal, not a parse error. `EndContour` remains a successful terminal. The only byte-backed lookup in this phase is the F4ac consumer item lookup for the stored `next_item` in the `Continue` branch.

The apparent `Continue + EndContour` branch is defensive against future representation changes and keeps the function total over its input type. It must still return successful `EndContour`, not panic or silently skip.

F4ah must not call `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next`, because that helper requires the original consumer item and would obscure the F4ag/F4af ownership boundary. F4ah must not re-apply action payloads, match `GuiSfntSimpleGlyphPathSinkAction` variants, allocate `Vec`, push commands, own a loop, inspect current point state, parse metadata, call lower contour/curve lookup helpers directly, rasterize, render, call platform APIs, or call host text measurement.

### SFNT simple glyph path sink action consumer consume once

F4ai composes F4af and F4ah into the smallest useful future-loop step. It consumes exactly one already-checked consumer item and resolves the immediate post-apply continuation, but it still does not own a loop and does not become a sink.

The result must preserve both sides of the operation:

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep:
    apply_step GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
    advance GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance
```

`apply_step` is not redundant. It carries the updated apply state and the consumed action status. Returning only `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance` would hide whether the current item emitted an event, rejected, closed a contour, or consumed an explicit `NoAction`.

The helper shape is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once bytes face_index state item policy:
    apply_step = gui_sfnt_simple_glyph_path_sink_action_consumer_item_apply state item

    match gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance bytes face_index apply_step policy:
        Err error:
            Err error

        Ok advance:
            Ok GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep apply_step advance
```

F4ai must not call F4ag directly. F4ah owns terminal classification and stored-next advancement. F4ai must not call `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next` because it would bypass the F4af/F4ah split and would also lose the structured apply result unless wrapped again.

F4ai must not match action payload variants, allocate `Vec`, push commands, own a loop, inspect current point state, parse metadata, call lower contour/curve lookup helpers directly, rasterize, render, call platform APIs, or call host text measurement.

### SFNT simple glyph path sink action start consumer item

F4aj is the start boundary for the future consumer loop. It converts a glyph contour start into the first checked `GuiSfntSimpleGlyphPathSinkActionConsumerItem`, using the existing start-item and consumer-item contracts.

The helper shape is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item bytes face_index glyph contour_index policy:
    match gui_sfnt_lookup_simple_glyph_path_sink_action_start_item bytes face_index glyph contour_index policy:
        Err error:
            Err error

        Ok item:
            match gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item bytes face_index item policy:
                Err error:
                    Err error

                Ok consumer_item:
                    Ok consumer_item
```

F4aj intentionally does not create a new value type. The result type is the existing F4ac `GuiSfntSimpleGlyphPathSinkActionConsumerItem`, because the responsibility is only to provide a byte-backed entry point for the first consumer packet.

“No advance” in F4aj means no F4ad consumer-item-next call, no F4af apply, no F4ah post-apply advance, and no F4ai consume-once call. F4ac still resolves the checked `GuiSfntSimpleGlyphPathSinkActionItemNext` needed to construct a consumer item. That resolution remains inside F4ac and does not make F4aj a traversal authority.

F4aj must not construct `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext`, must not call `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next`, and must not call `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once`. It must not call F4af/F4ah/F4ab/F4z/F4y/F4v/lower lookup helpers directly, inspect action payload variants, allocate `Vec`, push commands, own a loop, inspect current point state, parse metadata, rasterize, render, call platform APIs, call host text measurement, or perform font fallback.

### SFNT simple glyph path sink action start consume once

F4ak is the first start-to-consume boundary. It takes a byte-backed glyph contour start and an existing apply state, creates the first consumer item through F4aj, and consumes exactly that item through F4ai.

The helper shape is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once bytes face_index state glyph contour_index policy:
    match gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item bytes face_index glyph contour_index policy:
        Err error:
            Err error

        Ok consumer_item:
            match gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once bytes face_index state consumer_item policy:
                Err error:
                    Err error

                Ok consume_step:
                    Ok consume_step
```

F4ak deliberately returns `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep`, not only `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance`. The start boundary must preserve the same diagnostic and future-loop information as F4ai: the consumed action's apply state/status and the post-consume advance.

F4ak is not a contour loop and not a real sink. It must not call F4aa/F4ac/F4ad/F4af/F4ah/F4ab/F4z/F4y/F4v/lower lookup helpers directly. It must not construct `GuiSfntSimpleGlyphPathSinkActionConsumerItemNext`, inspect action payload variants, allocate `Vec`, push commands, own a loop, inspect current point state, parse metadata, rasterize, render, call platform APIs, call host text measurement, or perform font fallback.

### SFNT simple glyph path sink action consumer consume step apply summary

F4al adds stable public projections over `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep`. The future loop needs the updated apply state and the consumed action status, but it should not depend on the nested storage layout of F4ai and F4af.

The state helper shape is:

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state step:
    consumer_apply_step = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_step step
    inner_apply_step = gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_apply_step consumer_apply_step
    gui_sfnt_simple_glyph_path_sink_action_apply_step_state inner_apply_step
```

The status helper shape is:

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status step:
    consumer_apply_step = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_step step
    inner_apply_step = gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_apply_step consumer_apply_step
    gui_sfnt_simple_glyph_path_sink_action_apply_step_status inner_apply_step
```

F4al deliberately does not read `advance`. The existing `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_advance` accessor remains the explicit API for terminal / next item state. Separating apply summary and advance keeps future loops from conflating consumed action diagnostics with traversal state.

F4al must not call byte-backed lookup helpers, consumer item next, consume-once, start helpers, or lower contour/curve lookup helpers. It must not allocate `Vec`, push commands, own a loop, inspect action payload variants, parse metadata, rasterize, render, call platform APIs, call host text measurement, or perform font fallback.

### SFNT simple glyph path sink action consumer consume summary

F4am adds a flat public value above F4al. The future loop needs three values after each consume step: the updated apply state, the status of the consumed action, and the already-computed post-consume advance enum. Reading these three values independently would repeatedly expose the `ConsumeStep -> ApplyStep -> inner apply step` storage path to future code. F4am makes that read boundary explicit without taking ownership of traversal.

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary:
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    status GuiSfntSimpleGlyphPathSinkActionApplyStatus
    advance GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance
```

The conversion helper shape is:

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step step:
    state = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_state step
    status = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_apply_status step
    advance = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_step_advance step
    gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary state status advance
```

This differs from F4al intentionally. F4al does not read `advance`, because it only exposes the apply side. F4am does read `advance`, because its contract is to assemble the full consume-step summary. The source policy must keep these two contracts separate: F4al still forbids `advance`, while F4am requires exactly one call to the existing advance accessor.

F4am must not call byte-backed lookup helpers, consumer item next lookup, consume-once, start helpers, lower contour/curve lookup helpers, or metadata parser. It must not match action payload variants, allocate `Vec`, push commands, own a loop, inspect current point state, rasterize, render, call platform APIs, call host text measurement, or perform font fallback. It must not reinterpret the advance enum; `Continue`, `Rejected`, and `EndContour` remain the F4ah domain states.

### SFNT simple glyph path sink action consumer consume summary terminal

F4an adds the next pure projection above F4am. F4am creates a flat value and deliberately does not interpret the stored advance. The future loop still needs one stable API that reads the summary and returns traversal control without depending on the storage name or the F4ah enum directly. F4an provides that boundary.

The value shape is:

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerItem
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour
```

The helper shape is:

```text
gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal summary:
    advance = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_advance summary
    match advance:
        GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance Continue item:
            GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal Continue item
        GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance Rejected reason:
            GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal Rejected reason
        GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance EndContour:
            GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal EndContour
```

Although the public enum name ends with `Terminal`, it is the future loop's traversal control state and includes `Continue`. This avoids forcing loop code to re-match the lower stored advance enum, while keeping the operation allocation-free and deterministic.

F4an must read `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_advance` exactly once. It must not call byte-backed lookup helpers, consumer item next lookup, consume-once, start helpers, lower contour/curve lookup helpers, metadata parser, or `*_with_tables`. It must not match action payload variants, allocate `Vec`, push commands, own a loop, inspect current point state, rasterize, render, call platform APIs, call host text measurement, or perform font fallback. Its only match target is `GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance`.

### SFNT simple glyph path sink action start consume summary

F4ap creates the initial consume summary for a future contour consumer. It is a thin composition of F4ak and F4am: F4ak finds and consumes the first action, and F4am projects the resulting consume step into the stable summary value used by F4ao.

The helper shape is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_summary bytes face_index state glyph contour_index policy:
    start = gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once bytes face_index state glyph contour_index policy
    match start:
        Err error:
            Err error
        Ok consume_step:
            summary = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step consume_step
            Ok summary
```

F4ap does not own traversal beyond the first consumed action. It does not call F4ao, does not loop, and does not allocate an outline or event list. The only byte-backed lookup it may call directly is `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once`.

F4ap must call `gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once` exactly once and `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step` exactly once in the success branch. It must not call start item helpers, start consumer item helpers, consumer item consume-once, summary advance-once, consumer item next lookup, lower contour/curve lookup helpers, metadata parser, or `*_with_tables`. It must not match action payload variants, allocate `Vec`, push commands, own a loop, inspect current point state, rasterize, render, call platform APIs, call host text measurement, or perform font fallback.

### SFNT simple glyph path sink action consumer consume summary advance once

F4ao is the first byte-backed boundary above the F4am/F4an summary projection. It advances from one completed consume summary to the next completed consume summary, but it still advances only one action. It is not a contour-wide loop and does not own sink mutation or outline storage.

The result value is:

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance:
    Continue GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason
    EndContour
```

The helper shape is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_advance_once bytes face_index summary policy:
    state = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state summary
    terminal = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal summary
    match terminal:
        Continue item:
            consume_once = gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once bytes face_index state item policy
            match consume_once:
                Err error:
                    Err error
                Ok consume_step:
                    next_summary = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step consume_step
                    Ok Continue next_summary
        Rejected reason:
            Ok Rejected reason
        EndContour:
            Ok EndContour
```

`Rejected` and `EndContour` are domain terminals. They must remain `Result::Ok` values, because they do not mean that the SFNT bytes failed to parse. Only the Continue branch can call the existing consume-once byte-backed helper and therefore only that branch can produce a parse error from byte lookup.

F4ao must call `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state` exactly once, `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal` exactly once, `gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once` exactly once in the Continue branch, and `gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step` exactly once after a successful Continue consume. It must not call start helpers, consumer item next lookup, lower contour/curve lookup helpers, metadata parser, or `*_with_tables`. It must not match action payload variants, allocate `Vec`, push commands, own a full loop, inspect current point state, rasterize, render, call platform APIs, call host text measurement, or perform font fallback.

### SFNT simple glyph path sink action consumer consume summary drain budget

F4aq is the first bounded traversal boundary above F4ap/F4ao. It exists so a caller can advance a contour action consumer without allocating an outline, building a command list, mutating a real sink, or relying on an unbounded recursive traversal. The result is still a typed value, not a renderer side effect.

The drain result carries the summary at which traversal stopped:

```text
GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain:
    EndContour GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    Rejected GuiSfntSimpleGlyphPathSinkRejectReason GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    StepBudgetExhausted GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
```

`StepBudgetExhausted` is not fallback. It is an explicit domain terminal that says the caller provided no more action steps for this slice. Both `remaining_steps == 0` and `remaining_steps < 0` produce the same typed terminal when the current summary still has `Continue`.

The helper shape is:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_drain_budget bytes face_index summary policy remaining_steps:
    terminal = gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal summary
    match terminal:
        Rejected reason:
            Ok Rejected reason current_summary
        EndContour:
            Ok EndContour current_summary
        Continue:
            if remaining_steps <= 0:
                Ok StepBudgetExhausted current_summary
            else:
                advance = gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_advance_once bytes face_index summary policy
                match advance:
                    Err error:
                        Err error
                    Ok Continue next_summary:
                        gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_drain_budget bytes face_index next_summary policy remaining_steps - 1
                    Ok Rejected reason:
                        Ok Rejected reason current_summary
                    Ok EndContour:
                        Ok EndContour current_summary
```

The current F4ao contract normally returns `Continue next_summary` when its input terminal is `Continue`. The F4ao `Rejected` and `EndContour` branches are still handled in F4aq as a compatibility-preserving domain branch. In those branches F4aq stores the same current summary that was passed to F4ao; it does not invent or reparse a new terminal summary.

The start helper composes F4ap and F4aq only:

```text
gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_summary_drain_budget bytes face_index state glyph contour_index policy remaining_steps:
    start = gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_summary bytes face_index state glyph contour_index policy
    match start:
        Err error:
            Err error
        Ok summary:
            gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_consume_summary_drain_budget bytes face_index summary policy remaining_steps
```

F4aq must not allocate `Vec`, push commands, match action payload variants, call lower contour/curve helpers directly, parse metadata, call `*_with_tables`, rasterize, render, call platform APIs, call host text measurement, or perform font fallback. It is the final bounded traversal boundary before later phases decide owner recovery, outline storage, and command emission.

## SFNT simple glyph outline storage capacity and owner recovery boundary

F5a is the value-only boundary between bounded path traversal and future outline storage allocation. It consumes only checked topology values and capacity limits. It does not consume font bytes, does not allocate an outline, does not mutate a sink, and does not render.

The data flow is:

```text
F4aq drain result
    EndContour summary
        -> summary owner remains with caller
        -> caller may plan contour/outline storage from topology

    StepBudgetExhausted summary
        -> summary owner remains with caller
        -> caller must request another work slice
        -> capacity planning is not successful and not attempted for that unfinished contour

    Rejected rejected
        -> summary owner remains with rejected payload
        -> caller reports policy/domain rejection
        -> outline storage is not attempted
```

F5a does not call F4aq. The caller owns the scheduling boundary and only passes `GuiSfntSimpleGlyphTopology` to F5a after it has a complete contour traversal. This keeps time slicing separate from capacity planning and prevents a capacity helper from hiding unbounded byte-backed work.

Capacity calculation is pure:

```text
capacity_from_topology topology:
    contour_count = topology.contour_count
    point_count = topology.point_count

    if contour_count <= 0:
        InvalidTopology topology
    else if point_count <= 0:
        InvalidTopology topology
    else if contour_count > point_count:
        InvalidTopology topology
    else if point_count > 1073741823:
        CommandCountOverflow topology
    else:
        Fits capacity:
            glyph = topology.glyph
            contour_count = contour_count
            point_count = point_count
            edge_count = point_count
            path_command_pair_count = point_count
            path_command_count = point_count * 2
```

Limit comparison is also pure:

```text
check_limit capacity limit:
    if limit.max_contours <= 0 or capacity.contour_count > limit.max_contours:
        Rejected ContourCapacityExceeded capacity limit
    else if limit.max_points <= 0 or capacity.point_count > limit.max_points:
        Rejected PointCapacityExceeded capacity limit
    else if limit.max_edges <= 0 or capacity.edge_count > limit.max_edges:
        Rejected EdgeCapacityExceeded capacity limit
    else if limit.max_path_commands <= 0 or capacity.path_command_count > limit.max_path_commands:
        Rejected CommandCapacityExceeded capacity limit
    else:
        Fits capacity
```

`InvalidTopology` and `CommandCountOverflow` do not carry a capacity value because capacity cannot be trusted. Capacity exceeded carries both `capacity` and `limit` so a later owner-taking allocation API can return the original owner plus enough data to present a precise typed error or retry with a different limit.

F5a source policy:

- `Vec`, `push`, point list, contour list, path command list, raster mask, render command, platform API, host text measurement, and font substitute logic are forbidden.
- No byte-backed lookup, metadata parser, `*_with_tables`, F4aq drain helper, lower contour helper, or point decoder is called from capacity helpers.
- `StepBudgetExhausted` is a continuation-required state, not capacity success.
- All public states are enum / struct values, not strings.

## SFNT simple glyph outline storage owner boundary

F5b is the first allocation boundary after F5a. It converts a trusted outline capacity into one empty owner-backed scalar slot table. The storage is deliberately narrower than a complete outline representation:

```text
GuiSfntSimpleGlyphOutlineStorage:
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    scalar_slots Vec i32
    scalar_slot_count i32
```

F5b does not decode point coordinates, does not synthesize implied on-curve points, does not push path commands, does not rasterize, and does not call a renderer or host text API. Those operations start in later phases after owner recovery and time slicing are stable.

The scalar slot count is:

```text
scalar_slot_count =
    contour_count
    + point_count
    + point_count
    + edge_count
    + path_command_count
```

The five terms reserve contour endpoint slots, x coordinate slots, y coordinate slots, edge slots, and path command tag slots. The `Vec` is allocated with that capacity and `len = 0`; later builders populate it under their own mutation contract.

Forged capacity values must not be converted into F5a `Rejected` payloads. A rejected payload means "trusted capacity exceeds caller limit", so F5b validates the capacity shape before limit comparison:

```text
shape_is_valid capacity:
    contour_count > 0
    point_count > 0
    contour_count <= point_count
    edge_count == point_count
    path_command_pair_count == point_count
    point_count <= 1073741823
    path_command_count == point_count * 2
```

The `point_count <= 1073741823` check is evaluated before `point_count * 2`. If shape validation fails, allocation returns `InvalidCapacity` and `capacity_check = None`. This is the only branch where the error has no capacity check value.

After shape validation, F5b calls `check_limit capacity limit`. `Rejected` is returned as `CapacityRejected` with `capacity_check = Some checked`. `Fits` proceeds to scalar slot count overflow checking. Unexpected `InvalidTopology` or `CommandCountOverflow` results are treated as `InvalidCapacity` with `Some checked`, because they cannot be produced by a valid direct limit check and indicate an invariant break.

Scalar slot count overflow is checked by staged residual subtraction from `2147483647`:

```text
remaining = 2147483647
subtract contour_count
subtract point_count
subtract point_count
subtract edge_count
subtract path_command_count
```

If any term is larger than the remaining capacity, the result is `ScalarSlotCountOverflow`. Only after all terms fit does F5b compute the final count using prefix `add` and call `Vec` allocation exactly once.

Allocation failure returns `ScalarSlotStorageAllocFailed` with `capacity_check = Some checked`. Since F5b owns only one `Vec i32`, there is no partially allocated multi-owner aggregate to recover. `gui_sfnt_simple_glyph_outline_storage_free` consumes the storage and calls `vec::free` once.

## SFNT simple glyph outline scalar slot mutation boundary

F5c is the first mutation boundary for `GuiSfntSimpleGlyphOutlineStorage`. It only appends an i32 scalar slot value to the owner-backed `scalar_slots` table. It does not decide which logical region the value belongs to, and it does not decode point bytes, synthesize contour closure, emit path commands, rasterize, render, or call host/platform APIs.

The public mutation result is:

```text
push_scalar_slot storage value:
    Result GuiSfntSimpleGlyphOutlineStorage GuiSfntSimpleGlyphOutlineStoragePushError
```

The error payload is owner-preserving:

```text
GuiSfntSimpleGlyphOutlineStoragePushError:
    storage GuiSfntSimpleGlyphOutlineStorage
    scalar_value i32
    error StdErrorKind
```

`scalar_value` is Copy, but it is still stored explicitly in the error payload. This keeps the API shape consistent with `VecPushError`: failure returns both the input owner and the rejected item, rather than relying on caller-local variables or hidden cleanup.

Implementation order:

```text
capacity = storage.capacity
scalar_slot_count = storage.scalar_slot_count
scalar_slots = storage.scalar_slots
push_result = vec::push scalar_slots value

Ok next_slots:
    Ok Storage capacity next_slots scalar_slot_count

Err e:
    error_kind = vec_push_error_kind &e
    returned_slots = vec_push_error_vec e
    returned_storage = Storage capacity returned_slots scalar_slot_count
    Err PushError returned_storage value error_kind
```

The error kind must be read before consuming `e` with `vec_push_error_vec`. F5c does not call `vec::with_capacity`, `vec::free`, `vec::filled`, `vec::replace`, or `vec::pop`; allocation growth remains inside `vec::push`, and cleanup remains the caller's explicit responsibility through the F5b free helper.

The recovery API has two forms:

```text
push_error_storage error -> storage owner
push_error_with error callback -> callback storage scalar_value error_kind
```

The consuming storage accessor is convenient for Copy scalar payloads. The eliminator preserves the pattern used by owner-bearing collection APIs and is the preferred shape if later push errors carry non-Copy payloads.

## SFNT simple glyph outline scalar region cursor boundary

F5d adds typed cursor movement over the F5b scalar slot storage. The boundary is deliberately still below point decode and path command synthesis. It only answers "which fixed scalar region is being filled next" and "is the storage owner synchronized with that cursor".

The region order is fixed:

```text
contour endpoints
x coordinates
y coordinates
edges
path command tags
```

The unchecked boundary helper is internal. Public cursor construction is fail-closed:

```text
try_from_capacity capacity region:
    if not shape_is_valid capacity:
        Err InvalidOperation
    else:
        match scalar_slot_count_check capacity:
            Fits expected:
                Ok from_valid_capacity capacity region
            Overflow:
                Err CapacityExceeded
```

This ordering matters because region start/end uses i32 addition. A forged capacity must not reach addition before shape validation and scalar slot count overflow checking have succeeded.

`push_region_scalar` validates storage and cursor before mutation:

```text
capacity = storage.capacity
scalar_slot_count = storage.scalar_slot_count
scalar_slots_len = len storage.scalar_slots
scalar_slots_cap = cap storage.scalar_slots

if not shape_is_valid capacity:
    Err StorageCapacityInvalid storage cursor value None
else:
    match scalar_slot_count_check capacity:
        Fits expected:
            if scalar_slot_count != expected:
                Err StorageCapacityInvalid storage cursor value None
            else if scalar_slots_cap != scalar_slot_count:
                Err StorageCapacityInvalid storage cursor value None
            else if not cursor_is_well_formed cursor:
                Err CursorInvalid storage cursor value None
            else if not cursor matches from_valid_capacity capacity cursor.region:
                Err CursorRegionMismatch storage cursor value None
            else if scalar_slots_len != cursor.next_index:
                Err StorageCursorMismatch storage cursor value None
            else if cursor.next_index >= cursor.end:
                Err RegionFull storage cursor value None
            else:
                call F5c push_scalar_slot exactly once
        Overflow:
            Err StorageCapacityInvalid storage cursor value None
```

`scalar_slots_cap == scalar_slot_count` is checked before the F5c call. F5c is a general owner-preserving push helper, but F5d is a fixed outline-region boundary; allowing Vec growth here would hide a broken storage invariant. `scalar_slots_len == cursor.next_index` is checked before `RegionFull` so that forged full cursors over shorter storage are classified as `StorageCursorMismatch`, not as a legitimately full region.

Both success and failure payloads return the storage owner. `GuiSfntSimpleGlyphOutlineRegionPush` and `GuiSfntSimpleGlyphOutlineRegionPushError` must not implement `Clone` or `Copy`. The cursor and error kind are value-only and may implement `Clone` / `Copy`.

## SFNT simple glyph contour endpoint population boundary

F5e is the first semantic population boundary over the F5d contour endpoint region. It accepts a typed endpoint slot:

```text
GuiSfntSimpleGlyphContourEndpointSlot:
    contour_index i32
    end_point_index i32
```

The helper does not read `glyf` bytes. It is intentionally usable with synthetic endpoint values in doctests so the owner/cursor contract can be stabilized before byte-backed endpoint-array decoding is wired in.

F5e keeps three invariants separate:

```text
storage capacity invariant
cursor position invariant
endpoint sequence invariant
```

The validation order is fail-closed:

```text
capacity = storage.capacity

if not shape_is_valid capacity:
    Err StorageCapacityInvalid
else if scalar_slot_count_check capacity is not Fits:
    Err StorageCapacityInvalid
else:
    contour_count = capacity.contour_count
    point_count = capacity.point_count

    if not cursor_is_well_formed cursor:
        Err CursorInvalid
    else if cursor.region != ContourEndpoint:
        Err CursorRegionMismatch
    else if endpoint.contour_index != cursor.next_index:
        Err ContourIndexMismatch
    else if endpoint.contour_index < 0 or endpoint.contour_index >= contour_count:
        Err ContourIndexMismatch
    else if endpoint.end_point_index < 0 or endpoint.end_point_index >= point_count:
        Err EndpointOutOfRange
    else:
        previous must satisfy 0 <= previous < point_count - 1 when present
        end_point_index must be greater than previous when present
        validate final or non-final endpoint
        commit through F5d region push exactly once
```

The previous endpoint contract is:

```text
None:
    contour_index must be 0

Some previous:
    contour_index must be greater than 0
    previous must satisfy 0 <= previous < point_count - 1
    end_point_index must be greater than previous
```

The final endpoint contract is:

```text
if contour_index + 1 == contour_count:
    end_point_index == point_count - 1
else:
    end_point_index < point_count - 1
```

The `point_count - 1` arithmetic happens only after capacity shape and scalar slot count validation. This prevents forged capacity values from reaching semantic endpoint checks before the lower storage contract has accepted the shape.

F5e wraps F5d region push failure without losing ownership:

```text
RegionPushFailed:
    storage = recovered storage from F5d error
    region_error_kind = Some F5d error kind
    push_error_kind = F5d underlying push_error_kind
```

Validation failures set `region_error_kind = None` and `push_error_kind = None`, because no lower region push was attempted. The success and error payloads own `GuiSfntSimpleGlyphOutlineStorage`, so neither implements `Clone` or `Copy`.

## SFNT simple glyph contour endpoint byte reader bridge

F5f connects the already checked `glyf` endpoint-array reader to the F5e contour endpoint storage push. It is intentionally a bridge, not a full outline builder. It reads one endpoint from bytes and either returns a read error before mutation or delegates to F5e exactly once.

The bridge keeps these failure domains separate:

```text
byte read failure
    Result::Err ReadFailed
    parse_error = Some GuiSfntParseError
    endpoint = None
    no F5e push was attempted
    storage is the original storage owner

storage push failure
    Result::Err PushFailed
    parse_error = None
    endpoint = Some read endpoint slot
    push_error_kind = Some F5e error kind
    region_error_kind = F5d error kind when present
    storage_push_error_kind = F5c error kind when present
    storage is recovered from the F5e error owner
```

The helper order is fixed:

```text
match gui_sfnt_glyf_read_contour_endpoint bytes glyf topology contour_index:
    Err parse_error:
        return ReadFailed with original storage and cursor

    Ok end_point_index:
        endpoint = GuiSfntSimpleGlyphContourEndpointSlot contour_index end_point_index
        match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage cursor endpoint previous_endpoint:
            Ok pushed:
                return storage, advanced cursor, previous endpoint

            Err push_error:
                endpoint_value = endpoint from push_error
                push_kind = kind from push_error
                region_kind = region_error_kind from push_error
                storage_push_kind = push_error_kind from push_error
                returned_storage = storage from push_error
                return PushFailed with returned_storage and metadata
```

The lower error metadata must be read before `returned_storage = storage from push_error`, because that accessor consumes the owner-bearing error. This ordering preserves both the byte-read endpoint value and the recovered storage owner.

F5f must not call point stream construction, point coordinate decode, path generation, rasterization, render2d, platform APIs, host text measurement, or direct `Vec` APIs. The only allowed byte-side call in the bridge body is `gui_sfnt_glyf_read_contour_endpoint`.

## SFNT simple glyph point x coordinate population boundary

F5g adds the first coordinate-specific storage boundary. It accepts a typed x coordinate slot and appends its scalar value into the F5d `PointX` region.

```text
GuiSfntSimpleGlyphPointXSlot:
    point_index i32
    x i32
```

F5g does not read bytes. The point stream and x delta decoder already exist for older lookup APIs, but this phase deliberately stabilizes the storage mutation contract before wiring byte-backed x decode into the outline builder.

The main risk is confusing scalar storage index with glyph logical point index. For a two-contour, four-point glyph, the `PointX` cursor starts at scalar slot 2:

```text
ContourEndpoint region: [0, 2)
PointX region:          [2, 6)
```

Therefore:

```text
logical_point_index = cursor.next_index - cursor.start
```

This subtraction is valid only after:

```text
capacity shape is valid
scalar slot count is Fits
cursor is well formed
cursor region is PointX
cursor boundaries match the checked capacity
```

The validation order is fail-closed:

```text
capacity = storage.capacity

if not shape_is_valid capacity:
    Err StorageCapacityInvalid
else if scalar_slot_count_check capacity is not Fits:
    Err StorageCapacityInvalid
else:
    point_count = capacity.point_count

    if not cursor_is_well_formed cursor:
        Err CursorInvalid
    else if cursor.region != PointX:
        Err CursorRegionMismatch
    else if cursor does not match checked capacity:
        Err CursorRegionMismatch
    else:
        logical_point_index = cursor.next_index - cursor.start

        if point.point_index != logical_point_index:
            Err PointIndexMismatch
        else if point.point_index < 0 or point.point_index >= point_count:
            Err PointIndexOutOfRange
        else:
            commit through F5d region push exactly once
```

F5g wraps F5d region push failure without losing ownership:

```text
RegionPushFailed:
    storage = recovered storage from F5d error
    region_error_kind = Some F5d error kind
    push_error_kind = F5d underlying push_error_kind
```

The F5d error kind and F5c push error kind must be read before consuming the owner-bearing F5d error via its storage accessor.

F5g must not call byte readers, `gui_sfnt_glyf_*` helpers, point stream construction, coordinate decode, path generation, rasterization, render2d, platform APIs, host text measurement, or direct `Vec` APIs. The only mutation call in the commit helper is `gui_sfnt_simple_glyph_outline_storage_push_region_scalar`.

## SFNT simple glyph point x byte reader bridge boundary

F5h connects the already checked point stream range to the F5g `PointX` storage mutation helper. It is deliberately x-only. It does not validate y ranges, endpoint arrays, contour spans, edges, paths, raster masks, render commands, or platform font APIs.

The input stream may come from the F4g checked point stream lookup or from a test/virtualized caller. F5h trusts the stream shape except for the flag/x byte reads needed to compute the requested x coordinate. A forged stream with a bad y range is not rejected by F5h. That case belongs to the later PointY or full point decode phase.

```text
GuiSfntSimpleGlyphPointXReadPush:
    storage GuiSfntSimpleGlyphOutlineStorage
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor

GuiSfntSimpleGlyphPointXReadPushErrorKind:
    ReadFailed
    PushFailed
```

The owner-bearing success and error payloads must not implement `Clone` or `Copy`. The success payload needs accessors for both advanced cursor and storage owner, because tests and later builders must be able to thread the linear storage owner without peeking into fields.

The error payload keeps both failure domains separate:

```text
ReadFailed:
    storage = original storage
    cursor = original cursor
    point_index = requested point index
    point = None
    parse_error = Some GuiSfntParseError
    push_error_kind = None
    region_error_kind = None
    storage_push_error_kind = None

PushFailed:
    storage = recovered storage from F5g error
    cursor = original cursor
    point_index = requested point index
    point = Some rejected PointX slot
    parse_error = None
    push_error_kind = Some F5g error kind
    region_error_kind = F5g lower F5d error kind
    storage_push_error_kind = F5g lower F5c error kind
```

The helper order is:

```text
read x from stream
    if read failed:
        return ReadFailed without calling F5g

point = PointXSlot point_index x
push point through F5g exactly once
    if push failed:
        read rejected point and lower metadata from F5g error
        recover storage owner from F5g error
        return PushFailed
```

The x-only reader may call only bounded flag reads and `gui_sfnt_glyf_decode_x_delta`. It must not call:

```text
gui_sfnt_glyf_decode_y_delta
gui_sfnt_glyf_decode_point_from_stream
gui_sfnt_glyf_decode_point_state_from_stream
gui_sfnt_glyf_decode_point_state_from_flag_run
gui_sfnt_glyf_decode_flag_run_state
GuiSfntSimpleGlyphPointDecodeState
gui_sfnt_glyf_point_is_contour_end
gui_sfnt_glyf_read_contour_endpoint
contour span helpers
public gui_sfnt_lookup_* wrappers
render / raster / platform / host APIs
direct Vec APIs
```

## SFNT simple glyph point y coordinate population boundary

F5i mirrors the F5g storage boundary for the `PointY` region. It accepts a typed y coordinate slot and appends its scalar value into the F5d `PointY` region.

```text
GuiSfntSimpleGlyphPointYSlot:
    point_index i32
    y i32
```

PointY starts after both contour endpoints and all x coordinate slots:

```text
ContourEndpoint [0, contour_count)
PointX          [contour_count, contour_count + point_count)
PointY          [contour_count + point_count, contour_count + point_count + point_count)
```

For the 2-contour / 4-point fixture, `PointY` starts at scalar index 6. Tests and later builders must populate endpoint slots and all four `PointX` slots before using a `PointY` cursor. A `PointY` push with storage length 2 or 4 is a storage/cursor mismatch, not a valid y push.

The logical point index mapping is:

```text
logical_point_index = cursor.next_index - cursor.start
```

This subtraction is valid only after:

```text
capacity shape is valid
scalar slot count is Fits
cursor is well formed
cursor region is PointY
cursor boundaries match the checked capacity
```

The validation order is fail-closed and must not read cursor start/next for point semantics before the boundary match.

F5i wraps F5d region push failure without losing ownership:

```text
RegionPushFailed:
    storage = recovered storage from F5d error
    region_error_kind = Some F5d error kind
    push_error_kind = F5d underlying push_error_kind
```

The F5d error kind and F5c push error kind must be read before consuming the owner-bearing F5d error via its storage accessor.

F5i must not call byte readers, `gui_sfnt_glyf_*` helpers, point stream construction, coordinate decode, path generation, rasterization, render2d, platform APIs, host text measurement, or direct `Vec` APIs. The only mutation call in the commit helper is `gui_sfnt_simple_glyph_outline_storage_push_region_scalar`.

## SFNT simple glyph point y byte reader bridge boundary

F5j connects the checked point stream range to the F5i `PointY` storage mutation helper. It is deliberately y-only. It does not validate x ranges, endpoint arrays, contour spans, edges, paths, raster masks, render commands, or platform font APIs.

The input stream may come from the F4g checked point stream lookup or from a test/virtualized caller. F5j trusts the stream shape except for the flag/y byte reads needed to compute the requested y coordinate. A forged stream with a bad x range is not rejected by F5j. That case belongs to the PointX or full point decode phase.

```text
GuiSfntSimpleGlyphPointYReadPush:
    storage GuiSfntSimpleGlyphOutlineStorage
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor

GuiSfntSimpleGlyphPointYReadPushErrorKind:
    ReadFailed
    PushFailed
```

The owner-bearing success and error payloads must not implement `Clone` or `Copy`. The success payload needs accessors for both advanced cursor and storage owner.

The error payload keeps both failure domains separate:

```text
ReadFailed:
    storage = original storage
    cursor = original cursor
    point_index = requested point index
    point = None
    parse_error = Some GuiSfntParseError
    push_error_kind = None
    region_error_kind = None
    storage_push_error_kind = None

PushFailed:
    storage = recovered storage from F5i error
    cursor = original cursor
    point_index = requested point index
    point = Some rejected PointY slot
    parse_error = None
    push_error_kind = Some F5i error kind
    region_error_kind = F5i lower F5d error kind
    storage_push_error_kind = F5i lower F5c error kind
```

The helper order is:

```text
read y from stream
    if read failed:
        return ReadFailed without calling F5i

point = PointYSlot point_index y
push point through F5i exactly once
    if push failed:
        read rejected point and lower metadata from F5i error
        recover storage owner from F5i error
        return PushFailed
```

The y-only reader may call only bounded flag reads and `gui_sfnt_glyf_decode_y_delta`. It must not call:

```text
gui_sfnt_glyf_decode_x_delta
gui_sfnt_glyf_decode_point_from_stream
gui_sfnt_glyf_decode_point_state_from_stream
gui_sfnt_glyf_decode_point_state_from_flag_run
gui_sfnt_glyf_decode_flag_run_state
GuiSfntSimpleGlyphPointDecodeState
gui_sfnt_glyf_point_is_contour_end
gui_sfnt_glyf_read_contour_endpoint
contour span helpers
public gui_sfnt_lookup_* wrappers
render / raster / platform / host APIs
direct Vec APIs
```

## SFNT simple glyph outline point coordinate read boundary

F5k projects already-populated outline storage into a typed coordinate pair. It is intentionally read-only: the storage owner is borrowed, no slot is appended, and no byte stream or renderer boundary is crossed.

The value returned by this phase is not `GuiSfntSimpleGlyphPoint`. The storage built by F5b-F5j contains endpoint scalar values and x/y coordinates, but it does not yet contain the on-curve flag or the per-point end-of-contour bit. Returning a full point value would require invented defaults, which is fallback behavior. F5k therefore introduces a smaller value:

```text
GuiSfntSimpleGlyphOutlinePointCoordinate:
    glyph GuiGlyphId
    point_index i32
    x i32
    y i32
```

The read error is value-only and carries enough storage context for diagnostics without owning the storage:

```text
GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind:
    StorageCapacityInvalid
    ScalarSlotCountMismatch
    ScalarStorageCapacityMismatch
    PointIndexOutOfRange
    CoordinateNotReady
    ScalarSlotMissing

GuiSfntSimpleGlyphOutlinePointCoordinateReadError:
    kind GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    point_index i32
    scalar_slot_count i32
    scalar_slots_len i32
    scalar_slots_cap i32
```

The coordinate indexes are derived from the same region layout used by F5d:

```text
x_slot_index = contour_count + point_index
y_slot_index = contour_count + point_count + point_index
```

Validation must be ordered so untrusted capacity and partial storage never flow into unchecked slot reads:

```text
capacity shape
scalar slot count Fits
storage scalar_slot_count == expected
scalar_slots_cap == scalar_slot_count
point_index range
scalar_slots_len > y_slot_index
private scalar slot get for x and y
```

Because the regions are populated sequentially, `scalar_slots_len > y_slot_index` proves that the matching x slot is also expected to be present. If `vec::get` returns `None` after this readiness check, the storage has an internal structural mismatch and the helper returns `ScalarSlotMissing`. It must not infer a zero coordinate or re-run byte decoding.

Only the private scalar getter may call `vec::get`. The public read helper must not call:

```text
direct Vec APIs
byte readers
GuiSfntSimpleGlyphPointStream
gui_sfnt_glyf_decode_x_delta
gui_sfnt_glyf_decode_y_delta
full point decode helpers
endpoint readers
contour span helpers
edge / path helpers
render / raster / platform / host APIs
```

## SFNT simple glyph outline point endpoint marker read boundary

F5l is the endpoint-side counterpart to F5k. It borrows `GuiSfntSimpleGlyphOutlineStorage` and projects the already-populated `ContourEndpoint` scalar region into a value that says which contour owns the requested logical point and whether the point is exactly the contour end.

```text
GuiSfntSimpleGlyphOutlinePointEndpointMarker:
    glyph GuiGlyphId
    point_index i32
    contour_index i32
    end_of_contour bool
```

The helper does not inspect flag bytes, does not read x/y coordinate slots, and does not construct `GuiSfntSimpleGlyphPoint`. It is deliberately only the endpoint marker projection needed by later full point and outline stream phases.

The read error is value-only:

```text
GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind:
    StorageCapacityInvalid
    ScalarSlotCountMismatch
    ScalarStorageCapacityMismatch
    PointIndexOutOfRange
    EndpointNotReady
    EndpointSlotMissing
    EndpointTopologyInvalid

GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadError:
    kind GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    point_index i32
    scalar_slot_count i32
    scalar_slots_len i32
    scalar_slots_cap i32
```

Validation must first establish the storage shape:

```text
capacity shape
scalar slot count Fits
storage scalar_slot_count == expected
scalar_slots_cap == scalar_slot_count
point_index range
scalar_slots_len >= contour_count
```

The endpoint scan then walks every endpoint slot, not only the first slot that contains `point_index`. The loop state is:

```text
contour_index
previous_endpoint
found bool
found_contour_index
found_end_of_contour
```

For each endpoint slot:

```text
None                     -> EndpointSlotMissing
endpoint < 0             -> EndpointTopologyInvalid
endpoint >= point_count  -> EndpointTopologyInvalid
endpoint <= previous     -> EndpointTopologyInvalid

if not found and point_index <= endpoint:
    record contour_index and endpoint == point_index

if final contour:
    endpoint must be point_count - 1
    found must be true
    return recorded marker
else:
    continue with previous_endpoint = endpoint
```

This complete-topology rule is required. A forged endpoint region such as `[1, 2]` for a four-point glyph must not return a successful marker for point 0. The final endpoint mismatch makes the whole endpoint topology invalid.

F5l may reuse the F5k private scalar slot getter. The public endpoint marker helper must not call:

```text
direct Vec APIs
byte readers
GuiSfntSimpleGlyphPointStream
GuiSfntSimpleGlyphPoint
coordinate read helpers
edge / path helpers
render / raster / platform / host APIs
```

## SFNT simple glyph point flag marker read boundary

F5m reads only the flag run metadata from a checked `GuiSfntSimpleGlyphPointStream`. It deliberately does not add a new scalar storage region. The existing scalar layout is already part of the F5 storage contract, so inserting `PointFlag` before `PointX` or after `PointY` would shift the region offsets used by previous phases.

```text
GuiSfntSimpleGlyphPointFlagMarker:
    glyph GuiGlyphId
    point_index i32
    raw_flag i32
    on_curve bool
```

The public helper uses the existing parse error type:

```text
gui_sfnt_glyf_read_point_flag_from_stream:
    ByteBuf
    GuiSfntTableRecord
    GuiSfntSimpleGlyphPointStream
    point_index
    -> Result GuiSfntSimpleGlyphPointFlagMarker GuiSfntParseError
```

This is a byte-backed read-only boundary, not a storage mutation boundary. `MissingGlyphOutline` represents a requested point outside `0 <= point_index < point_count`. `MalformedGlyfRecord` represents a broken flag stream, including missing repeat count bytes and repeat runs that exceed `point_count`.

The loop state is:

```text
logical_index
flag_cursor
```

For each flag run:

```text
read flag at flag_cursor within stream.flag_data range
if repeat bit 8:
    read repeat_count at flag_cursor + 1 within stream.flag_data range
    run_count = repeat_count + 1
    next_flag_cursor = flag_cursor + 2
else:
    run_count = 1
    next_flag_cursor = flag_cursor + 1

run_end_next = logical_index + run_count
if run_count <= 0:
    MalformedGlyfRecord
if run_end_next > point_count:
    MalformedGlyfRecord
if point_index < run_end_next:
    return marker from this flag
else:
    continue from run_end_next and next_flag_cursor
```

The `run_end_next > point_count` check must happen before the target membership check. This prevents a forged repeat run from returning a marker for an early target before the stream has been proven to cover a valid set of logical points.

F5m must not call:

```text
gui_sfnt_glyf_decode_x_delta
gui_sfnt_glyf_decode_y_delta
GuiSfntSimpleGlyphPointDecodeState
gui_sfnt_glyf_decode_point_from_stream
endpoint readers
coordinate storage readers
edge / path helpers
render / raster / platform / host APIs
```

## SFNT simple glyph outline point read boundary

F5n is the first F5 boundary that returns the existing full point value:

```text
GuiSfntSimpleGlyphPoint:
    glyph GuiGlyphId
    point_index i32
    x i32
    y i32
    on_curve bool
    end_of_contour bool
```

The helper is still read-only. It borrows `GuiSfntSimpleGlyphOutlineStorage`, reads the checked point stream, and composes only these previous boundaries:

```text
F5k coordinate read
F5l endpoint marker read
F5m flag marker read
```

It does not build path commands, edges, outline streams, masks, or render commands.

The public helper is:

```text
gui_sfnt_simple_glyph_outline_storage_read_point:
    ByteBuf
    GuiSfntTableRecord
    GuiSfntSimpleGlyphPointStream
    GuiSfntSimpleGlyphOutlineStorage
    point_index
    -> Result GuiSfntSimpleGlyphPoint GuiSfntSimpleGlyphOutlinePointReadError
```

Because the helper mixes a storage-derived view and a stream-derived view, it must reject shared precondition failures before calling component readers:

```text
capacity = storage.capacity
topology = stream.topology

if capacity shape is invalid:
    StorageCapacityInvalid

if capacity.glyph != topology.glyph:
    StorageStreamGlyphMismatch

if capacity.contour_count != topology.contour_count:
    StorageStreamContourCountMismatch

if capacity.point_count != topology.point_count:
    StorageStreamPointCountMismatch

if point_index < 0 or point_index >= capacity.point_count:
    PointIndexOutOfRange
```

Only after those checks may the helper call F5k, F5l, and F5m. The order is fixed:

```text
coordinate = F5k storage coordinate read
endpoint_marker = F5l storage endpoint marker read
flag_marker = F5m stream flag marker read
```

Each component helper is called exactly once. Later helpers are reached only after earlier helpers succeed. Their errors are wrapped in the F5n error payload:

```text
CoordinateReadFailed
EndpointMarkerReadFailed
FlagReadFailed
```

After the three component values are available, F5n checks that every component agrees with the shared request:

```text
component.glyph == capacity.glyph == topology.glyph
component.point_index == point_index
```

These checks are intentionally defensive. They should be unreachable when F5k/F5l/F5m keep their contracts, but they prevent a future regression from silently assembling a mixed point.

F5n must not call:

```text
vec::
gui_sfnt_simple_glyph_outline_storage_scalar_slot_get
gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker_loop
gui_sfnt_glyf_read_point_flag_from_stream_loop
gui_sfnt_glyf_read_point_flag_run_or_continue
gui_sfnt_glyf_decode_x_delta
gui_sfnt_glyf_decode_y_delta
edge / path helpers
render / raster / platform / host APIs
```

## SFNT simple glyph outline point read step boundary

F5o lifts the F5n single-point read into a no-allocation cursor step. It is deliberately not a collection builder. It does not allocate a `Vec`, mutate outline storage, synthesize path commands, rasterize glyphs, or emit render commands.

The step boundary is:

```text
GuiSfntSimpleGlyphOutlinePointReadCursor:
    next_point_index i32

GuiSfntSimpleGlyphOutlinePointReadStepStatus:
    Point
    End

GuiSfntSimpleGlyphOutlinePointReadStep:
    status GuiSfntSimpleGlyphOutlinePointReadStepStatus
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    next_cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    point Option GuiSfntSimpleGlyphPoint
```

The terminal step is a successful value, not an error:

```text
status = End
point = None
cursor.next_point_index = point_count
next_cursor.next_point_index = point_count
```

However, terminal success is only valid after the same storage/stream shared preconditions used by F5n are known to be true:

```text
capacity shape is valid
capacity.glyph == topology.glyph
capacity.contour_count == topology.contour_count
capacity.point_count == topology.point_count
```

This ordering prevents a forged stream from hiding behind an End step. The helper must not check `cursor.next_point_index == point_count` before validating the shared storage/stream relation.

The error value is:

```text
GuiSfntSimpleGlyphOutlinePointReadStepErrorKind:
    StorageCapacityInvalid
    StorageStreamGlyphMismatch
    StorageStreamContourCountMismatch
    StorageStreamPointCountMismatch
    CursorOutOfRange
    PointReadFailed

GuiSfntSimpleGlyphOutlinePointReadStepError:
    kind GuiSfntSimpleGlyphOutlinePointReadStepErrorKind
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    topology GuiSfntSimpleGlyphTopology
    point_error Option GuiSfntSimpleGlyphOutlinePointReadError
```

The public helper is:

```text
gui_sfnt_simple_glyph_outline_storage_read_point_step:
    ByteBuf
    GuiSfntTableRecord
    GuiSfntSimpleGlyphPointStream
    GuiSfntSimpleGlyphOutlineStorage
    GuiSfntSimpleGlyphOutlinePointReadCursor
    -> Result GuiSfntSimpleGlyphOutlinePointReadStep GuiSfntSimpleGlyphOutlinePointReadStepError
```

The fixed control flow is:

```text
capacity = storage.capacity
topology = stream.topology
validate capacity shape
validate glyph / contour_count / point_count agreement
point_index = cursor.next_point_index
if point_index < 0 or point_index > point_count:
    CursorOutOfRange
if point_index == point_count:
    End step with point None
else:
    point = F5n read_point exactly once
    Point step with point Some and next_cursor = point_index + 1
```

The only non-terminal point read delegate is:

```text
gui_sfnt_simple_glyph_outline_storage_read_point:
    called exactly once when point_index < point_count
```

The `point_index == point_count` branch must appear before the F5n call. F5n treats that same value as `PointIndexOutOfRange`, but F5o owns the iteration contract where this value is the normal terminal state.

F5o must not call:

```text
vec::
gui_sfnt_simple_glyph_outline_storage_read_point_coordinate
gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker
gui_sfnt_glyf_read_point_flag_from_stream
gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker_loop
gui_sfnt_glyf_read_point_flag_from_stream_loop
gui_sfnt_glyf_read_point_flag_run_or_continue
edge / path helpers
render / raster / platform / host APIs
```

## SFNT simple glyph outline point read drain budget

F5p adds a no-allocation drain boundary over F5o. It is a traversal contract, not a collection implementation. It does not allocate a full point list, mutate edge/path storage, synthesize path commands, rasterize glyphs, or emit render commands.

The drain summary is:

```text
GuiSfntSimpleGlyphOutlinePointReadDrainSummary:
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    points_read i32
    last_point Option GuiSfntSimpleGlyphPoint
```

`cursor` is the next cursor where traversal stopped. `points_read` counts only point steps consumed by this drain call. `last_point` is `None` when no point was consumed, including terminal-start and zero-budget-start cases.

The drain result is:

```text
GuiSfntSimpleGlyphOutlinePointReadDrain:
    End GuiSfntSimpleGlyphOutlinePointReadDrainSummary
    StepBudgetExhausted GuiSfntSimpleGlyphOutlinePointReadDrainSummary
```

`StepBudgetExhausted` is a successful typed terminal for the current work slice. It is not a substitute for `End`, and callers must schedule another slice if they need a complete traversal.

F5p has its own error value:

```text
GuiSfntSimpleGlyphOutlinePointReadDrainErrorKind:
    StorageCapacityInvalid
    StorageStreamGlyphMismatch
    StorageStreamContourCountMismatch
    StorageStreamPointCountMismatch
    CursorOutOfRange
    StepReadFailed
    StepInvariantInvalid

GuiSfntSimpleGlyphOutlinePointReadDrainError:
    kind GuiSfntSimpleGlyphOutlinePointReadDrainErrorKind
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    topology GuiSfntSimpleGlyphTopology
    step_error Option GuiSfntSimpleGlyphOutlinePointReadStepError
    step Option GuiSfntSimpleGlyphOutlinePointReadStep
```

`StepReadFailed` wraps an F5o `Result::Err`. `StepInvariantInvalid` is used only when F5o returns a success value that contradicts F5p's already validated non-terminal state or fails to advance by exactly one point:

```text
F5o returns End after F5p proved cursor.next_point_index < point_count
F5o returns Point with point None
F5o returns Point with next_cursor.next_point_index != current next_point_index + 1
```

Both cases are fail-closed invariant violations. They must not be converted into `End`, `StepBudgetExhausted`, or a point count increment.

The public helper is:

```text
gui_sfnt_simple_glyph_outline_storage_read_point_drain_budget:
    ByteBuf
    GuiSfntTableRecord
    GuiSfntSimpleGlyphPointStream
    GuiSfntSimpleGlyphOutlineStorage
    GuiSfntSimpleGlyphOutlinePointReadCursor
    remaining_steps
    -> Result GuiSfntSimpleGlyphOutlinePointReadDrain GuiSfntSimpleGlyphOutlinePointReadDrainError
```

The implementation keeps the current drain summary fields as local mutable state inside one bounded `while` body:

```text
current_cursor GuiSfntSimpleGlyphOutlinePointReadCursor
current_points_read i32
current_last_point Option GuiSfntSimpleGlyphPoint
current_remaining_steps i32
done bool
```

The shared storage/stream/cursor validation is split into a non-recursive validation helper before the terminal and budget checks. This keeps the iterative drain body small enough for current NEPLg2.1 codegen while preserving the same contract:

```text
validate shared preconditions
if terminal: End
else if budget exhausted: StepBudgetExhausted
else call F5o once and advance state
```

The fixed order is:

```text
validate shared storage/stream preconditions
validate cursor range, allowing cursor == point_count
if cursor == point_count:
    End summary
if remaining_steps <= 0:
    StepBudgetExhausted summary
step = F5o read_point_step exactly once
if step is Err:
    StepReadFailed
if step.status == Point and step.point == Some point:
    recurse with next_cursor, points_read + 1, last_point = Some point, remaining_steps - 1
if step.status == Point and step.point == None:
    StepInvariantInvalid
if step.status == End:
    StepInvariantInvalid
```

Terminal-before-budget is required so `cursor == point_count` succeeds even with budget 0. Budget-before-F5o is required so non-terminal budget exhaustion does not perform hidden point read work.

F5p must not call:

```text
vec::
gui_sfnt_simple_glyph_outline_storage_read_point
gui_sfnt_simple_glyph_outline_storage_read_point_coordinate
gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker
gui_sfnt_glyf_read_point_flag_from_stream
gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker_loop
gui_sfnt_glyf_read_point_flag_from_stream_loop
gui_sfnt_glyf_read_point_flag_run_or_continue
edge / path helpers
render / raster / platform / host APIs
```

## SFNT simple glyph outline point stream item classification boundary

F5q adds the first no-allocation item boundary after full point read/drain. F5p can traverse points and retain the last point, but later contour/path/raster phases need a stable value that combines the original point payload with a typed stream classification. F5q provides that O(1) value.

The new item kind is:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemKind:
    OnCurve
    OffCurve
    EndOnCurve
    EndOffCurve
```

The item is:

```text
GuiSfntSimpleGlyphOutlinePointStreamItem:
    point GuiSfntSimpleGlyphPoint
    kind GuiSfntSimpleGlyphOutlinePointStreamItemKind
```

The constructor does not accept `kind` from callers. Accepting both a point and an externally supplied kind would allow inconsistent values such as an off-curve endpoint point paired with `OnCurve`. That would be fallback-like data corruption because later phases would need to choose which field to trust. F5q therefore derives kind from the point exactly once:

```text
kind = gui_sfnt_simple_glyph_outline_point_stream_item_kind_from_point point
item = GuiSfntSimpleGlyphOutlinePointStreamItem point kind
```

The classification function is total and returns no `Result`:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_kind_from_point:
    GuiSfntSimpleGlyphPoint
    -> GuiSfntSimpleGlyphOutlinePointStreamItemKind
```

This is not a parser boundary, so there is no error to hide. Any invalid byte/storage state must already have been rejected by F5n/F5o/F5p before a full point reaches F5q.

The fixed classification order is:

```text
on_curve = point.on_curve
end_of_contour = point.end_of_contour
if end_of_contour:
    if on_curve:
        EndOnCurve
    else:
        EndOffCurve
else:
    if on_curve:
        OnCurve
    else:
        OffCurve
```

Endpoint is deliberately represented in the top-level kind. Later contour/path code should not need to re-read the endpoint boolean to distinguish a normal on-curve point from the final point of a contour.

F5q must not call byte readers, storage readers, drain loops, path sink helpers, rasterizers, renderer commands, platform APIs, or host text APIs. It also must not allocate a point vector. It may only call the existing `GuiSfntSimpleGlyphPoint` field accessors and construct the item value.

## SFNT simple glyph outline point stream item step boundary

F5r converts the successful value shape of F5o into the item shape introduced by F5q. It is deliberately a pure conversion boundary. It does not call byte-backed point readers, storage APIs, F5p drain, path sink helpers, rasterizers, render commands, platform APIs, or host text APIs.

The step status is:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemStepStatus:
    Item
    End
```

The step value is:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemStep:
    status GuiSfntSimpleGlyphOutlinePointStreamItemStepStatus
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    next_cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    item Option GuiSfntSimpleGlyphOutlinePointStreamItem
```

F5o already has a public constructor for `GuiSfntSimpleGlyphOutlinePointReadStep`, so F5r must not trust the shape blindly. It rechecks the invariants that are visible from the step value:

```text
Point:
    item source point must be Some
    next_cursor.next_point_index == cursor.next_point_index + 1

End:
    item source point must be None
    next_cursor.next_point_index == cursor.next_point_index
```

The only F5r error kind is:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemStepErrorKind:
    PointStepInvariantInvalid
```

The error stores the invalid F5o step:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemStepError:
    kind GuiSfntSimpleGlyphOutlinePointStreamItemStepErrorKind
    step GuiSfntSimpleGlyphOutlinePointReadStep
```

The conversion helper is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_step_from_point_step:
    GuiSfntSimpleGlyphOutlinePointReadStep
    -> Result GuiSfntSimpleGlyphOutlinePointStreamItemStep GuiSfntSimpleGlyphOutlinePointStreamItemStepError
```

F5r may call `gui_sfnt_simple_glyph_outline_point_stream_item` exactly once in the successful `Point + Some point + valid next cursor` branch. It must not call `gui_sfnt_simple_glyph_outline_point_stream_item_kind_from_point` directly. Keeping classification inside the F5q constructor prevents later phases from duplicating or drifting from the kind derivation contract.

The fixed conversion order is:

```text
read status, cursor, next_cursor, point option
read cursor indexes
if status is Point:
    require point Some
    require next == current + 1
    construct F5q item exactly once
    return Item step with Some item
if status is End:
    require point None
    require next == current
    return End step with None
otherwise:
    PointStepInvariantInvalid
```

## SFNT simple glyph outline point stream item drain boundary

F5s adds a no-allocation drain boundary over the F5o point step and the F5r item step conversion. It emits no `Vec`, no path command list, no raster mask, and no render command. Its purpose is to let later phases advance the classified item stream by bounded work slices while preserving the same typed cursor semantics on Web, native, bare, and headless hosts.

F5p and F5s share the same cursor precondition logic, but they do not share public drain errors. The shared logic is a private neutral validation helper:

```text
GuiSfntSimpleGlyphOutlinePointReadCursorValidation:
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    topology GuiSfntSimpleGlyphTopology
    point_index i32
    shared_point_count i32

GuiSfntSimpleGlyphOutlinePointReadCursorValidationRejectKind:
    StorageCapacityInvalid
    StorageStreamGlyphMismatch
    StorageStreamContourCountMismatch
    StorageStreamPointCountMismatch
    CursorOutOfRange

gui_sfnt_simple_glyph_outline_point_read_cursor_validate:
    storage &GuiSfntSimpleGlyphOutlineStorage
    stream GuiSfntSimpleGlyphPointStream
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    -> Result GuiSfntSimpleGlyphOutlinePointReadCursorValidation GuiSfntSimpleGlyphOutlinePointReadCursorValidationReject
```

The neutral helper is byte-free, path-free, render-free, raster-free, platform-free, and host-free. F5p converts its reject into `GuiSfntSimpleGlyphOutlinePointReadDrainErrorKind`. F5s converts the same reject into `GuiSfntSimpleGlyphOutlinePointStreamItemDrainErrorKind`. This prevents F5s from depending on F5p public drain behavior while still avoiding duplicated precondition logic.

The F5s summary and success value are:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemDrainSummary:
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    items_read i32
    last_item Option GuiSfntSimpleGlyphOutlinePointStreamItem

GuiSfntSimpleGlyphOutlinePointStreamItemDrain:
    End GuiSfntSimpleGlyphOutlinePointStreamItemDrainSummary
    StepBudgetExhausted GuiSfntSimpleGlyphOutlinePointStreamItemDrainSummary
```

The F5s error kind is:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemDrainErrorKind:
    StorageCapacityInvalid
    StorageStreamGlyphMismatch
    StorageStreamContourCountMismatch
    StorageStreamPointCountMismatch
    CursorOutOfRange
    PointStepReadFailed
    ItemStepConvertFailed
    ItemStepInvariantInvalid
```

`PointStepReadFailed` stores the F5o sub-error. `ItemStepConvertFailed` stores the F5r sub-error and the F5o step that failed conversion. `ItemStepInvariantInvalid` stores the F5o step and F5r item step that passed conversion but failed the drain-level defensive checks. That branch is expected to be unreachable when F5o and F5r are correct, but it remains part of the contract because public constructors and future internal edits can otherwise forge inconsistent values.

The fixed F5s order is:

```text
validate shared cursor context
if point_index == shared_point_count:
    return End summary
if remaining_steps <= 0:
    return StepBudgetExhausted summary
call F5o point step exactly once
if F5o Err:
    return PointStepReadFailed
call F5r item step conversion exactly once
if F5r Err:
    return ItemStepConvertFailed
if F5r Ok Item:
    require item Some
    require item_step.cursor.next_point_index == point_index
    require item_step.next_cursor.next_point_index == point_index + 1
    update cursor, items_read, last_item, remaining_steps
if F5r Ok End:
    return ItemStepInvariantInvalid
```

Terminal-before-budget and budget-before-F5o are contract requirements. F5s must not call `gui_sfnt_simple_glyph_outline_storage_read_point_drain_budget`, `gui_sfnt_simple_glyph_outline_point_stream_item_kind_from_point`, lower point readers, path helpers, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection boundary

F5t adds the first allocator-backed owner for classified point stream items. It intentionally does not drain F5s into the collection. That loop remains a later phase because the collection owner contract, the push invariants, and the typed read surface must be stable before traversal and allocation are coupled.

F5t uses a dedicated limit:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit:
    max_items i32
```

The dedicated limit is required because `GuiSfntSimpleGlyphOutlineStorageLimit` is about scalar outline storage regions, edge capacity, and path command capacity. Reusing it for a `Vec GuiSfntSimpleGlyphOutlinePointStreamItem` would let unrelated edge/path limits reject item collection allocation. F5t reads only `max_items` and compares it with `capacity.point_count`.

Allocation order is fixed:

```text
capacity shape
max_items > 0
point_count <= max_items
vec::with_capacity point_count
```

The collection owner is:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollection:
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    items Vec GuiSfntSimpleGlyphOutlinePointStreamItem
    item_count i32
```

`items.len == item_count` and `items.cap == capacity.point_count` are owner invariants. The owner is not `Clone` or `Copy`.

The free boundary consumes the collection owner and calls `vec::free` exactly once on the inner `items` Vec. Free does not inspect stream state and must not call path, raster, render, platform, or host APIs.

The push boundary validates every public-constructor-forgeable invariant before mutating the `Vec`:

```text
capacity shape
items.len == item_count
items.cap == capacity.point_count
item_count < capacity.point_count
item.point.glyph == capacity.glyph
item.point.point_index == item_count
item.kind == kind_from_point item.point
vec::push exactly once
```

`ItemKindMismatch` is important because `GuiSfntSimpleGlyphOutlinePointStreamItem` is a public struct and can be forged with a kind that does not match the point fields. The authority for the kind remains F5q `gui_sfnt_simple_glyph_outline_point_stream_item_kind_from_point`.

Push error keeps both owner recovery and diagnostic data:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushError:
    collection GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    item GuiSfntSimpleGlyphOutlinePointStreamItem
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind
    storage_error Option StdErrorKind
```

Validation errors set `storage_error = None`. A lower `vec::push` failure sets `storage_error = Some error_kind`. The implementation must read `vec::vec_push_error_kind &e` before consuming `e` with `vec::vec_push_error_vec e`.

The public read helper returns typed `Result`, not `Option`:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadErrorKind:
    InvalidCapacity
    CollectionLengthMismatch
    CollectionCapacityMismatch
    ItemIndexOutOfRange
    ItemStorageMissing
```

This avoids treating invariant failure, out-of-range request, and missing storage slot as the same `None`. The fixed read order is capacity shape, length invariant, capacity invariant, index range, then `vec::get` exactly once. In source policy terms, the read helper must call vec::get exactly once after every invariant check.

F5t helper bodies may call `vec::with_capacity`, `vec::free`, `vec::len`, `vec::cap`, `vec::push`, and `vec::get`. They must not call F5s drain, F5r conversion, F5o point step, F5p point drain, byte readers, path helpers, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection drain boundary

F5u is the first owner-preserving bridge between the no-allocation F5s stream drain and the allocator-backed F5t item collection. It still does not build a path command list, raster mask, render command stream, platform surface, or host text object.

The key design constraint is that F5s exposes only `last_item`, not the full list of items read during a drain call. Therefore F5u must never call F5s with the caller's full `remaining_steps`. Instead it derives a local `step_budget`:

```text
if current_remaining_steps <= 0:
    step_budget = 0
else:
    step_budget = 1
```

Then it calls F5s exactly once with `step_budget`. A budget of 0 delegates terminal-before-budget classification to F5s without allowing a read. A budget of 1 allows exactly one classified item to be returned and then committed to the collection with one F5t push.

The owner-bearing success payload is:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainSummary:
    collection GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    items_read i32
    last_item Option GuiSfntSimpleGlyphOutlinePointStreamItem

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain:
    End GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainSummary
    StepBudgetExhausted GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainSummary
```

The summary is not `Clone` or `Copy` because it owns the collection. `items_read` is the number of items committed to the collection during this F5u call. It is not the per-call F5s summary count.

The owner-bearing error payload is:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind:
    CollectionCursorMismatch
    ItemDrainFailed
    ItemDrainInvariantInvalid
    CollectionPushFailed

GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainError:
    collection GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    cursor GuiSfntSimpleGlyphOutlinePointReadCursor
    items_read i32
    last_item Option GuiSfntSimpleGlyphOutlinePointStreamItem
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind
    item_drain_error Option GuiSfntSimpleGlyphOutlinePointStreamItemDrainError
    item_drain_result Option GuiSfntSimpleGlyphOutlinePointStreamItemDrain
    push_error_kind Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind
    push_storage_error Option StdErrorKind
    rejected_item Option GuiSfntSimpleGlyphOutlinePointStreamItem
```

The error is also not `Clone` or `Copy`. `CollectionCursorMismatch` rejects a collection owner whose committed `item_count` differs from the cursor `next_point_index`; without this precondition a terminal cursor with an empty collection could be returned as a successful End. `ItemDrainFailed` stores the lower F5s error. `ItemDrainInvariantInvalid` stores the lower F5s success value in `item_drain_result`, because the bug is the impossible success shape itself. `CollectionPushFailed` stores both the lower F5s success value and the F5t push metadata. In that branch, the cursor and item count remain at the committed position, while `item_drain_result` and `rejected_item` describe the item that was read but not committed.

The fixed F5u order is:

```text
require collection.item_count == current_cursor.next_point_index
derive step_budget as 0 or 1
call F5s item drain exactly once with step_budget
if F5s Err:
    return ItemDrainFailed with collection owner
if F5s Ok:
    extract status and summary
    require summary.items_read in 0..1
    if summary.items_read == 0:
        return the F5s status with unchanged collection owner
    require step_budget == 1
    require summary.last_item Some
    call F5t collection push exactly once
    if push Err:
        read push kind, storage error, rejected item
        recover collection owner
        return CollectionPushFailed
    if push Ok:
        commit collection, cursor, items_read, last_item, remaining_steps
        return End if F5s returned End
        return StepBudgetExhausted if caller budget is exhausted
        otherwise repeat the loop
```

The push error branch must read `gui_sfnt_simple_glyph_outline_point_stream_item_collection_push_error_kind &push_error`, `gui_sfnt_simple_glyph_outline_point_stream_item_collection_push_error_storage_error &push_error`, and `gui_sfnt_simple_glyph_outline_point_stream_item_collection_push_error_item &push_error` before consuming `push_error` with `gui_sfnt_simple_glyph_outline_point_stream_item_collection_push_error_collection push_error`.

F5u may call F5s drain and F5t collection push. It must not call F5r conversion, F5o point step, F5p point drain, F5n point read, lower byte/point readers, `vec::` directly, path helpers, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection contour span boundary

F5v is the first collection-backed topology read over the F5u/F5t item collection. It does not re-read endpoint bytes and does not call the F4 byte-backed contour span helpers. The collection owner is read by borrow, so F5v does not consume or recover the collection owner.

The public boundary is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    -> Result GuiSfntSimpleGlyphContourSpan GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanError
```

The error payload is typed diagnostic data, not an owner-bearing payload:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanError:
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    contour_index i32
    item_index i32
    observed_contour_count i32
    last_endpoint_index i32
    item_count i32
    items_len i32
    items_cap i32
    read_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadError
    item Option GuiSfntSimpleGlyphOutlinePointStreamItem
```

The scan order is fixed:

```text
validate capacity shape
validate items.len == item_count
validate items.cap == capacity.point_count
validate item_count == capacity.point_count
validate requested contour_index range
for each item_index from 0 to point_count - 1:
    call collection_read_item exactly once for that item
    validate item.point.glyph == capacity.glyph
    validate item.point.point_index == item_index
    validate item.kind == kind_from_point item.point
    if item.kind is EndOnCurve or EndOffCurve:
        if observed_contour_count == requested contour_index:
            store start = previous_endpoint + 1
            store end = item_index
        previous_endpoint = item_index
        last_endpoint = item_index
        observed_contour_count += 1
after scan:
    require requested contour was found
    require observed_contour_count == capacity.contour_count
    require last_endpoint == capacity.point_count - 1
    require derived span point_count > 0
    return GuiSfntSimpleGlyphContourSpan
```

The final endpoint check is separate from the contour count check. `observed_contour_count == capacity.contour_count` can still be forged by endpoints `[1, 2]` with `contour_count = 2` and `point_count = 4`, leaving point 3 outside every contour. F5v must reject that shape as `FinalContourEndMismatch` before returning a span.

F5v may call `gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item`, `gui_sfnt_simple_glyph_outline_point_stream_item_kind_matches_point`, and the item / point / capacity accessors. It must not call F4 byte-backed contour helpers, F5 drains, F5 point steps, direct `vec::`, byte readers, path helpers, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection contour point boundary

F5w is the collection-backed equivalent of the old F4j contour point lookup. It composes F5v contour span lookup with one collection item read. It does not call the byte-backed contour point helper and does not consume the collection owner.

The public boundary is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    contour_point_index i32
    -> Result GuiSfntSimpleGlyphContourPoint GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointError
```

F5w intentionally accepts `contour_index` rather than a caller-provided `GuiSfntSimpleGlyphContourSpan`. `GuiSfntSimpleGlyphContourSpan` has a public constructor, so accepting it would allow callers to forge a span outside the collection topology. F5w must call F5v exactly once and use that checked span as its authority.

The error payload is diagnostic data, not owner recovery:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointError:
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind
    contour_index i32
    contour_point_index i32
    absolute_point_index i32
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    span Option GuiSfntSimpleGlyphContourSpan
    span_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanError
    read_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadError
    item Option GuiSfntSimpleGlyphOutlinePointStreamItem
    item_count i32
    items_len i32
    items_cap i32
```

The required order is:

```text
1. Read collection capacity / item_count / items_len / items_cap for diagnostics.
2. Call F5v collection contour span lookup exactly once.
3. Convert F5v error to ContourSpanFailed with span_error Some.
4. On F5v success, validate span.glyph == capacity.glyph.
5. Validate span.contour_index == contour_index.
6. Validate span.start_point_index >= 0.
7. Validate span.end_point_index >= span.start_point_index.
8. Validate span.end_point_index < capacity.point_count.
9. Validate span.point_count == span.end_point_index - span.start_point_index + 1.
10. Only after the span invariant succeeds, validate contour_point_index range.
11. Compute absolute_point_index = span.start_point_index + contour_point_index.
12. Revalidate absolute_point_index stays inside span and capacity.
13. Read collection_read_item exactly once at absolute_point_index.
14. Validate item.point.glyph, item.point.point_index, and item.kind.
15. Return gui_sfnt_simple_glyph_contour_point span contour_point_index point.
```

The span invariant checks are visible in F5w even though F5v should already guarantee them. This is the same fail-closed style as the F5 step/drain boundaries: a lower-boundary impossible success shape must not be reclassified as `ContourPointIndexOutOfRange`, and it must not flow into an item read.

`ContourPointIndexOutOfRange` is only for caller local-index mistakes after a valid span. It must be checked before `collection_read_item`. The error stores `absolute_point_index = -1` because no absolute index is part of the contract when the local index is invalid.

`ItemReadFailed` is a defensive branch for forged or future collection owners that pass the outer checks but fail the lower read helper. Normal public F5t/F5u collections should not reach it after F5v succeeds. It remains typed so source policy and future owner implementations do not replace it with an unchecked read.

F5w may call F5v, `gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item`, `gui_sfnt_simple_glyph_outline_point_stream_item_kind_matches_point`, and item / point / span / capacity accessors. It must not call F4 byte-backed contour helpers, F5 drains, F5 point steps, direct `vec::`, byte readers, edge/path helpers, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection contour edge boundary

F5x is the collection-backed equivalent of the old F4k contour edge lookup. It composes F5v contour span lookup with two F5w contour point lookups. It does not call the byte-backed contour edge helper and does not consume the collection owner.

The public boundary is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    edge_index i32
    -> Result GuiSfntSimpleGlyphContourEdge GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeError
```

F5x intentionally accepts `contour_index` rather than a caller-provided `GuiSfntSimpleGlyphContourSpan`. `GuiSfntSimpleGlyphContourSpan` has a public constructor, so accepting it would allow callers to forge a span outside the collection topology. F5x must call F5v exactly once and use that checked span as its authority.

The error payload is diagnostic data, not owner recovery:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeError:
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind
    contour_index i32
    edge_index i32
    next_contour_point_index i32
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    span Option GuiSfntSimpleGlyphContourSpan
    span_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanError
    start_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointError
    end_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointError
    start Option GuiSfntSimpleGlyphContourPoint
    end Option GuiSfntSimpleGlyphContourPoint
    item_count i32
    items_len i32
    items_cap i32
```

The required order is:

```text
1. Read collection capacity / item_count / items_len / items_cap for diagnostics.
2. Call F5v collection contour span lookup exactly once.
3. Convert F5v error to ContourSpanFailed with span_error Some.
4. On F5v success, validate span.glyph == capacity.glyph.
5. Validate span.contour_index == contour_index.
6. Validate span.start_point_index >= 0.
7. Validate span.end_point_index >= span.start_point_index.
8. Validate span.end_point_index < capacity.point_count.
9. Validate span.point_count == span.end_point_index - span.start_point_index + 1.
10. Only after the span invariant succeeds, validate edge_index range.
11. Compute next_contour_point_index = edge_index + 1, wrapping to 0 at span.point_count.
12. Call F5w contour point lookup for start at edge_index.
13. Call F5w contour point lookup for end at next_contour_point_index.
14. Validate start span matches F5v span.
15. Validate end span matches F5v span.
16. Validate start local index == edge_index.
17. Validate end local index == next_contour_point_index.
18. Validate start absolute point index == span.start_point_index + edge_index.
19. Validate end absolute point index == span.start_point_index + next_contour_point_index.
20. Return gui_sfnt_simple_glyph_contour_edge start end edge_index next_contour_point_index.
```

The span invariant checks are visible in F5x even though F5v should already guarantee them. This is the same fail-closed style as F5w: a lower-boundary impossible success shape must not be reclassified as `EdgeIndexOutOfRange`, and it must not flow into point lookup.

`EdgeIndexOutOfRange` is only for caller edge-index mistakes after a valid span. It must be checked before F5w point lookup. The error stores `next_contour_point_index = -1` because no wrapped next index is part of the contract when the edge index is invalid.

One-point contours are valid. For `span.point_count == 1` and `edge_index == 0`, `next_contour_point_index` is `0`, and start / end point to the same absolute point. F5x must preserve this topology value rather than discarding it as a no-segment or close command; curve classification and sink policy are later phases.

`StartPointFailed` and `EndPointFailed` are defensive branches for forged or future collection owners that pass span checks but fail lower F5w point lookup. Normal public F5t/F5u/F5v/F5w collections should not reach them after F5v succeeds. They remain typed so future owner implementations do not replace them with unchecked point access.

F5x may call F5v, F5w, point / span / edge / capacity accessors, and `gui_sfnt_simple_glyph_contour_edge`. It must not call F4 byte-backed contour helpers, F5 drains, F5 point steps, direct `vec::`, byte readers, path helpers, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection curve segment boundary

F5y is the collection-backed equivalent of the old F4l curve segment lookup. It composes F5x contour edge lookup with one optional F5w lookahead point lookup. It does not call the byte-backed curve segment helper and does not consume the collection owner.

The public boundary is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_curve_segment:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    edge_index i32
    -> Result GuiSfntSimpleGlyphCurveSegment GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError
```

The error payload is diagnostic data, not owner recovery:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError:
    kind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentErrorKind
    contour_index i32
    edge_index i32
    next_contour_point_index i32
    lookahead_contour_point_index i32
    capacity GuiSfntSimpleGlyphOutlineStorageCapacity
    edge_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeError
    lookahead_error Option GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointError
    edge Option GuiSfntSimpleGlyphContourEdge
    lookahead Option GuiSfntSimpleGlyphContourPoint
    item_count i32
    items_len i32
    items_cap i32
```

The required order is:

```text
1. Read collection capacity / item_count / items_len / items_cap for diagnostics.
2. Call F5x collection contour edge lookup exactly once.
3. Convert F5x error to ContourEdgeFailed with edge_error Some.
4. On F5x success, read edge start, edge end, edge index, next_contour_point_index, and start span.
5. Validate edge span glyph == capacity glyph.
6. Validate edge span contour_index == requested contour_index.
7. Validate edge span start_point_index >= 0.
8. Validate edge span end_point_index >= start_point_index.
9. Validate edge span end_point_index < capacity.point_count.
10. Validate edge span point_count == end_point_index - start_point_index + 1.
11. Validate edge_index metadata == requested edge_index.
12. Validate recomputed next index matches edge metadata.
13. Validate start span matches the edge span.
14. Validate end span matches the edge span.
15. Validate start local index == edge_index.
16. Validate end local index == next_contour_point_index.
17. Validate start absolute point index == span.start_point_index + edge_index.
18. Validate end absolute point index == span.start_point_index + next_contour_point_index.
19. Only after edge invariant succeeds, inspect start/end on-curve flags.
20. If start is on-curve and end is off-curve, compute lookahead_contour_point_index by wrapping next_contour_point_index + 1 at span.point_count.
21. Needed lookahead calls F5w contour point lookup exactly once.
22. Needed lookahead failure becomes LookaheadPointFailed with lookahead_error Some.
23. Needed lookahead success must validate lookahead span matches edge span.
24. Validate lookahead local index == lookahead_contour_point_index.
25. Validate lookahead absolute point index == span.start_point_index + lookahead_contour_point_index.
26. Return gui_sfnt_classify_simple_glyph_curve_segment edge Option::Some lookahead.
27. If lookahead is not needed, do not call F5w and return gui_sfnt_classify_simple_glyph_curve_segment edge Option::None.
```

F5y must not produce `MissingLookahead` by skipping a needed lookup. `MissingLookahead` remains part of the lower pure classifier contract for callers that pass `Option::None` directly, but the collection-backed boundary has enough topology to know when lookahead is required. If the required F5w lookup fails, the result is `LookaheadPointFailed`.

Single-point contours and off-curve start edges remain successful `NoSegment` classifications. They are valid topology states and must not be reclassified as F5y errors.

F5y may call F5x, F5w, point / span / edge / capacity accessors, and `gui_sfnt_classify_simple_glyph_curve_segment`. It must not call F4 byte-backed contour or curve helpers, F5 drains, F5 point steps, direct `vec::`, byte readers, path helpers, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection path command pair boundary

F5z is the collection-backed equivalent of the old F4o byte-backed path command pair lookup. It does not re-decode SFNT bytes, does not call the byte-backed path lookup, and does not introduce a command list. It composes exactly one F5y curve segment lookup with the existing pure path command pair projection.

The public boundary is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_pair:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    edge_index i32
    -> Result GuiSfntSimpleGlyphPathCommandPair GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError
```

F5z deliberately reuses the F5y error domain. The boundary adds no new operation that can fail: path command pair projection is a total value projection over `GuiSfntSimpleGlyphCurveSegment`. If F5y returns an error, F5z returns that exact error. If F5y returns `NoSegment`, F5z returns an explicit pair of `SkipNoSegment` commands through the existing F4o projection; it does not return `Option::None` and does not silently skip the edge.

The required order is:

```text
1. Call F5y collection curve segment lookup exactly once.
2. On F5y error, return Result::Err error without wrapping or changing the error kind.
3. On F5y success, call gui_sfnt_simple_glyph_curve_segment_path_command_pair exactly once.
4. Return Result::Ok pair.
```

F5z may call F5y and the pure `gui_sfnt_simple_glyph_curve_segment_path_command_pair` projection. It must not call byte-backed F4 lookup helpers, metadata parsers, `_with_tables` helpers, F5x/F5w lower collection lookups directly, F5 drain/point-step APIs, direct `vec::`, `push`, sink traversal, event consumer APIs, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection path sink event pair boundary

F5aa is the collection-backed equivalent of the old F4p pure event-pair projection, but it keeps the collection-backed authority chain intact. It does not re-decode SFNT bytes, does not call the byte-backed path lookup, and does not introduce sink traversal or event consumer state. It composes exactly one F5z path command pair lookup with the existing pure path sink event pair projection.

The public boundary is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_pair:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    edge_index i32
    -> Result GuiSfntSimpleGlyphPathSinkEventPair GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError
```

F5aa deliberately reuses the F5z error domain. The boundary adds no new operation that can fail: path sink event pair projection is a total value projection over `GuiSfntSimpleGlyphPathCommandPair`. If F5z returns an error, F5aa returns that exact error. If F5z returns a pair containing `SkipNoSegment`, F5aa wraps the skip commands as sink events; it does not return `Option::None` and does not silently skip the edge.

The required order is:

```text
1. Call F5z collection path command pair lookup exactly once.
2. On F5z error, return Result::Err error without wrapping or changing the error kind.
3. On F5z success, call gui_sfnt_simple_glyph_path_command_pair_sink_event_pair exactly once.
4. Return Result::Ok event_pair.
```

F5aa may call F5z and the pure `gui_sfnt_simple_glyph_path_command_pair_sink_event_pair` projection. It must not call byte-backed F4 lookup helpers, metadata parsers, `_with_tables` helpers, F5y/F5x/F5w lower collection lookups directly, F5 drain/point-step APIs, direct `vec::`, `push`, sink traversal, event consumer APIs, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection path sink event kind pair boundary

F5ab is the collection-backed equivalent of the old F4q pure event-kind-pair projection, but it keeps the collection-backed authority chain intact. It does not re-decode SFNT bytes, does not call the byte-backed path lookup, and does not introduce sink traversal or event consumer state. It composes exactly one F5aa path sink event pair lookup with the existing pure path sink event kind pair projection.

The public boundary is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_pair:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    edge_index i32
    -> Result GuiSfntSimpleGlyphPathSinkEventKindPair GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError
```

F5ab deliberately reuses the F5aa error domain. The boundary adds no new operation that can fail: path sink event kind pair projection is a total value projection over `GuiSfntSimpleGlyphPathSinkEventPair`. If F5aa returns an error, F5ab returns that exact error. If F5aa returns a pair containing `SkipNoSegment` events, F5ab keeps the `SkipNoSegment` reason in the kind pair; it does not return `Option::None` and does not silently skip the edge.

The required order is:

```text
1. Call F5aa collection path sink event pair lookup exactly once.
2. On F5aa error, return Result::Err error without wrapping or changing the error kind.
3. On F5aa success, call gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair exactly once.
4. Return Result::Ok kind_pair.
```

F5ab may call F5aa and the pure `gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair` projection. It must not call byte-backed F4 lookup helpers, metadata parsers, `_with_tables` helpers, F5z/F5y/F5x/F5w lower collection lookups directly, F5 drain/point-step APIs, direct `vec::`, `push`, sink traversal, event consumer/action APIs, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection path sink event kind at boundary

F5ac is the collection-backed equivalent of the F4r typed-slot kind projection, but it keeps the collection-backed authority chain intact. It does not re-decode SFNT bytes, does not call the byte-backed path lookup, and does not introduce sink traversal or event consumer state. It composes exactly one F5ab path sink event kind pair lookup with the existing pure typed-slot kind projection.

The public boundary is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_at:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    edge_index i32
    slot GuiSfntSimpleGlyphPathSinkEventSlot
    -> Result GuiSfntSimpleGlyphPathSinkEventKind GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError
```

F5ac deliberately reuses the F5ab error domain. The boundary adds no new operation that can fail: `GuiSfntSimpleGlyphPathSinkEventSlot` is a closed enum with only `First` and `Second`, and path sink event kind slot projection is a total value projection over `GuiSfntSimpleGlyphPathSinkEventKindPair`. If F5ab returns an error, F5ac returns that exact error. If F5ab returns a kind pair containing `SkipNoSegment`, F5ac preserves the selected `SkipNoSegment` reason; it does not return `Option::None`, does not silently skip the edge, and does not fall back to a byte-backed path.

The required order is:

```text
1. Call F5ab collection path sink event kind pair lookup exactly once.
2. On F5ab error, return Result::Err error without wrapping or changing the error kind.
3. On F5ab success, call gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at exactly once with the typed slot.
4. Return Result::Ok kind.
```

F5ac may call F5ab and the pure `gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at` projection. It must not call byte-backed F4 lookup helpers, metadata parsers, `_with_tables` helpers, F5aa/F5z/F5y/F5x/F5w lower collection lookups directly, F5 drain/point-step APIs, direct `vec::`, `push`, sink traversal, event consumer/action APIs, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection path sink event at boundary

F5ad is the collection-backed equivalent of the F4r typed-slot event projection, but it keeps the collection-backed authority chain intact. It does not re-decode SFNT bytes, does not call the byte-backed path lookup, and does not introduce sink traversal or event consumer state. It composes exactly one F5aa path sink event pair lookup with the existing pure typed-slot event projection.

The public boundary is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_at:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    edge_index i32
    slot GuiSfntSimpleGlyphPathSinkEventSlot
    -> Result GuiSfntSimpleGlyphPathSinkEvent GuiSfntSimpleGlyphOutlinePointStreamItemCollectionCurveSegmentError
```

F5ad deliberately reuses the F5aa error domain. The boundary adds no new operation that can fail: `GuiSfntSimpleGlyphPathSinkEventSlot` is a closed enum with only `First` and `Second`, and path sink event slot projection is a total value projection over `GuiSfntSimpleGlyphPathSinkEventPair`. If F5aa returns an error, F5ad returns that exact error. If F5aa returns a pair containing `SkipNoSegment`, F5ad preserves the selected event payload; it does not return `Option::None`, does not silently skip the edge, and does not fall back to a byte-backed path.

The required order is:

```text
1. Call F5aa collection path sink event pair lookup exactly once.
2. On F5aa error, return Result::Err error without wrapping or changing the error kind.
3. On F5aa success, call gui_sfnt_simple_glyph_path_sink_event_pair_event_at exactly once with the typed slot.
4. Return Result::Ok event.
```

F5ad may call F5aa and the pure `gui_sfnt_simple_glyph_path_sink_event_pair_event_at` projection. It must not call F5ab/F5ac kind helpers, byte-backed F4 lookup helpers, metadata parsers, `_with_tables` helpers, F5z/F5y/F5x/F5w/F5v lower collection lookups directly, F5 drain/point-step APIs, direct `vec::`, `push`, sink traversal, event consumer/action APIs, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection path contour step boundary

F5ae is the collection-backed contour step boundary. It mirrors the byte-backed F4s step shape, but its authority is the already built point-stream item collection rather than font bytes or table metadata. It does not traverse a whole contour, does not allocate a command list, and does not call a sink consumer.

The public boundary is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_contour_step:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    cursor GuiSfntSimpleGlyphPathContourCursor
    -> Result GuiSfntSimpleGlyphPathContourStep GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

F5ae introduces a dedicated error domain because the contour step has three different failure authorities:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepErrorKind:
    ContourSpanFailed
    CursorGlyphMismatch
    PathSinkEventFailed
```

`ContourSpanFailed` stores the F5v contour span error and does not invent an event error. `CursorGlyphMismatch` stores the collection capacity and cursor that failed the identity check and does not call event lookup. `PathSinkEventFailed` stores the F5ad event lookup error and does not hide it behind a generic parse error.

The required order is:

```text
read capacity and cursor fields
call collection contour span lookup exactly once
if span Err:
    return ContourSpanFailed
if span Ok:
    check cursor glyph against collection capacity glyph before event lookup
    if mismatch:
        return CursorGlyphMismatch
    call F5ad collection path sink event lookup exactly once
    if event Err:
        return PathSinkEventFailed
    derive kind from the returned event
    compute next with private cursor-next helper
    construct GuiSfntSimpleGlyphPathContourStep
```

F5ac remains a kind-only sibling boundary. F5ae must not call F5ac because doing so would derive the same edge twice through a second lookup chain. The returned event is the single source of truth for `GuiSfntSimpleGlyphPathSinkEventKind`.

F5ae may call F5v contour span lookup, F5ad event lookup, the pure event-kind projection, the private cursor-next helper, and the pure contour-step constructor. It must not call F5ac/F5ab/F5aa directly, byte-backed F4 lookup helpers, metadata parsers, `_with_tables` helpers, F5z/F5y/F5x/F5w lower collection lookups directly, F5 drain/point-step APIs, direct `vec::`, `push`, sink traversal, event consumer/action APIs, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection path sink step boundary

F5af is the collection-backed sink step boundary. It mirrors byte-backed F4t `gui_sfnt_lookup_simple_glyph_path_sink_step`, but its contour step authority is F5ae rather than font bytes or table metadata.

The public boundary is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_step:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    cursor GuiSfntSimpleGlyphPathContourCursor
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkStep GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

F5af intentionally does not introduce a new error type. It has no additional fallible data authority beyond F5ae, so F5ae errors must be propagated unchanged:

```text
match F5ae collection cursor:
    Err error:
        Err error
    Ok contour_step:
        sink_step_from_contour_step policy contour_step
```

Policy rejection is not an exceptional condition. `gui_sfnt_simple_glyph_path_sink_step_from_contour_step` keeps the existing contract: unsupported off-curve starts become `GuiSfntSimpleGlyphPathSinkPrimaryAction::Reject`, and reject steps have `NoTailAction`. Close-contour tail handling remains in the pure F4t helper.

F5af may call only F5ae and `gui_sfnt_simple_glyph_path_sink_step_from_contour_step`. It must not call F5ad/F5ac/F5aa directly, byte-backed F4 lookup helpers, metadata parsers, `_with_tables` helpers, lower collection edge/curve/path helpers, `vec::`, `push`, action step/traversal helpers, event consumers, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection path sink action step boundary

F5ag is the collection-backed sink action step boundary. It mirrors byte-backed F4v `gui_sfnt_lookup_simple_glyph_path_sink_action_step`, but its sink step authority is F5af rather than font bytes or table metadata.

The public boundary is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    cursor GuiSfntSimpleGlyphPathSinkActionCursor
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionStep GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

F5ag intentionally does not introduce a new error type. It adds no fallible authority beyond F5af, so F5af errors must be propagated unchanged.

The required order is:

```text
split action cursor into contour cursor and action slot
call F5af collection path sink step lookup exactly once
if F5af Err:
    return the same error
if F5af Ok:
    call pure action-step projection exactly once with sink step and action slot
    return GuiSfntSimpleGlyphPathSinkActionStep
```

The helper splits the action cursor with `gui_sfnt_simple_glyph_path_sink_action_cursor_contour_cursor` and `gui_sfnt_simple_glyph_path_sink_action_cursor_action_slot`. Only the contour cursor is passed to F5af. The action slot stays outside the collection lookup and is passed to the pure projection after F5af succeeds.

The action and next-state decision remains owned by `gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step`. Primary action slots advance to the tail slot of the same contour cursor. Tail action slots advance according to the source contour step next state. Policy rejection remains a successful action payload and is not converted to `Result::Err`.

F5ag may call only F5af and the pure action-step projection, plus the two action-cursor accessors needed to split the cursor. It must not call F5ae/F5ad/F5ac/F5aa directly, byte-backed F4 lookup helpers, metadata parsers, `_with_tables` helpers, lower collection edge/curve/path helpers, `vec::`, `push`, action advance/item/consumer helpers, event consumers, sink traversal, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection path sink action step advance and item boundary

F5ah is the collection-backed action step advance and item boundary. It mirrors byte-backed F4y/F4z, but its next-step authority is F5ag rather than font bytes or table metadata.

The public boundaries are:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step_advance:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    step &GuiSfntSimpleGlyphPathSinkActionStep
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionStepAdvance GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step_item:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    step &GuiSfntSimpleGlyphPathSinkActionStep
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionStepItem GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

F5ah intentionally does not introduce a new error type. The only fallible action is delegated F5ag lookup, so F5ag errors must be propagated unchanged.

The advance helper order is:

```text
read step.next exactly once
if next is Continue cursor:
    call F5ag collection path sink action step lookup exactly once
    if F5ag Err:
        return the same error
    if F5ag Ok:
        return Continue next_step
if next is EndContour:
    return Ok EndContour
```

`EndContour` is a successful terminal state and must not be represented as `Option::None`, `Result::Err`, or a hidden no-op. Policy rejection remains an action payload in the current or next step and does not change traversal in F5ah.

The item helper order is:

```text
call collection action step advance exactly once
if advance Err:
    return the same error
if advance Ok:
    copy the current step value
    construct GuiSfntSimpleGlyphPathSinkActionStepItem
```

The item helper must not inspect action payloads. `Reject`, `NoAction`, `CloseContour`, and actual sink mutation belong to later consumer phases.

F5ah may call only `gui_sfnt_simple_glyph_path_sink_action_step_next`, F5ag, the collection-backed F5ah advance helper from the item helper, the pure item constructor, and `*step` value copy. It must not call F5af/F5ae/F5ad/F5ac/F5aa directly, byte-backed F4 lookup helpers, metadata parsers, `_with_tables` helpers, lower collection edge/curve/path helpers, `vec::`, `push`, action consumer helpers, sink traversal, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection path sink action item next and consumer item boundary

F5ai is the collection-backed action item next and consumer item boundary. It mirrors byte-backed F4ab/F4ac, but it uses F5ah action step items as the only next-step authority and never returns to font bytes, table metadata, or byte-backed F4 lookup helpers.

The public boundaries are:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_item_next:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    item &GuiSfntSimpleGlyphPathSinkActionStepItem
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionItemNext GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_item:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    item &GuiSfntSimpleGlyphPathSinkActionStepItem
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerItem GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

F5ai intentionally does not introduce a new error type. The only fallible action is delegated F5ah item lookup, so F5ah errors must be propagated unchanged. `EndContour` is a successful terminal state and must not be represented as `Option::None`, `Result::Err`, or a hidden fallback.

The action item next helper order is:

```text
read checked advance exactly once
if advance is Continue next_step:
    call F5ah collection action step item lookup exactly once
    if F5ah Err:
        return the same error
    if F5ah Ok:
        return Continue next_item
if advance is EndContour:
    return Ok EndContour
```

The consumer item helper order is:

```text
read stored step exactly once
read action from stored step exactly once
call collection action item next exactly once
if next Err:
    return the same error
if next Ok:
    construct GuiSfntSimpleGlyphPathSinkActionConsumerItem
```

F5ai does not interpret action payloads. `EmitEvent`, `Reject`, `NoAction`, and `CloseContour` remain data in the copied action value. Consumer apply, consume-once, traversal, real sink mutation, outline construction, render2d command emission, rasterization, and platform presentation belong to later explicit phases.

F5ai may call only `gui_sfnt_simple_glyph_path_sink_action_step_item_advance`, F5ah collection action step item lookup, `gui_sfnt_simple_glyph_path_sink_action_step_item_step`, `gui_sfnt_simple_glyph_path_sink_action_step_action`, the F5ai action item next helper from the consumer item helper, and the pure consumer item constructor. It must not call F5ag/F5af/F5ae/F5ad/F5ac/F5aa directly, byte-backed F4 lookup helpers, metadata parsers, `_with_tables` helpers, lower collection edge/curve/path helpers, `vec::`, `push`, consumer apply/consume helpers, sink traversal, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection path sink action consumer next and consume once boundary

F5aj is the collection-backed consumer next and consume-once boundary. It mirrors byte-backed F4ad/F4ah/F4ai, but the only way to obtain a next consumer item is the F5ai collection-backed consumer item helper. This keeps byte buffers, table metadata, byte-backed lookup helpers, lower F5 collection traversal, real sink mutation, rasterization, render commands, and platform APIs outside this phase.

The public boundaries are:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_item_next:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    item &GuiSfntSimpleGlyphPathSinkActionConsumerItem
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerItemNext GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_apply_advance:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    step &GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_item_consume_once:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    item &GuiSfntSimpleGlyphPathSinkActionConsumerItem
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

F5aj intentionally does not introduce a new error type. The only fallible operation is the delegated F5ai consumer item lookup, so F5ai errors must be propagated unchanged. `Rejected` and `EndContour` are typed terminal success values and must not be collapsed into `Option::None`, `Result::Err`, silent no-op, or fallback.

The consumer item next helper order is:

```text
read consumer item next exactly once
if next is Continue next_item:
    call F5ai collection consumer item lookup exactly once
    if F5ai Err:
        return the same error
    if F5ai Ok:
        return Continue next_consumer_item
if next is EndContour:
    return Ok EndContour
```

The consumer apply advance helper order is:

```text
read apply terminal from apply step exactly once
if terminal is Continue continue_step:
    read saved next from continue_step exactly once
    if saved next is Continue next_item:
        call F5ai collection consumer item lookup exactly once
        if F5ai Err:
            return the same error
        if F5ai Ok:
            return Continue next_consumer_item
    if saved next is EndContour:
        return Ok EndContour
if terminal is Rejected reason:
    return Ok Rejected reason
if terminal is EndContour:
    return Ok EndContour
```

The Continue branch must not require or reconstruct the original consumer item. It must not re-read or interpret action payloads. The saved next stored in the apply step is the authority because the pure apply helper has already combined current action and previous next state.

The consume-once helper order is:

```text
call pure consumer item apply exactly once
call collection consumer apply advance exactly once
if advance Err:
    return the same error
if advance Ok:
    construct GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep exactly once
```

F5aj may call only `gui_sfnt_simple_glyph_path_sink_action_consumer_item_next`, F5ai collection consumer item lookup, `gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step`, `gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_next`, `gui_sfnt_simple_glyph_path_sink_action_consumer_item_apply`, the F5aj collection apply advance helper from consume-once, and the pure consume step constructor. It must not call byte-backed F4 lookup helpers, lower F5 collection helpers, `gui_sfnt_simple_glyph_path_sink_action_step_action`, action payload variants, `vec::`, `push`, consume summary helpers, sink traversal, real sink mutation, rasterizers, render commands, platform APIs, or host text APIs.

## SFNT simple glyph outline point stream item collection path sink action start consumer boundary

F5ak is the collection-backed contour start consumer boundary. It adds the first item and first consumer item entry point above F5aj, but it must not accept an external glyph. The collection already owns `GuiSfntSimpleGlyphOutlineStorageCapacity`, and that capacity is the authority for the glyph. The required sequence is `collection_capacity -> capacity.glyph -> start_cursor -> F5ag action step -> F5ah step item`.

The public boundaries are:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_item:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionStepItem GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consumer_item:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerItem GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consume_once:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consume_summary:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

F5ak intentionally reuses `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError`. The fallible operations are still collection-backed step lookup, step item lookup, F5ai consumer item lookup, and F5aj consume-once. Each delegated error is propagated unchanged. `Rejected` and `EndContour` remain typed terminal success values when they are produced by the consumer layer.

The start item helper order is:

```text
read collection capacity exactly once
read glyph from capacity exactly once
construct start cursor from capacity glyph and contour_index exactly once
call F5ag collection action step lookup exactly once
if F5ag Err:
    return the same error
if F5ag Ok:
    call F5ah collection action step item lookup exactly once
    if F5ah Err:
        return the same error
    if F5ah Ok:
        return item
```

The start consumer item helper order is:

```text
call F5ak start item exactly once
if start item Err:
    return the same error
if start item Ok:
    call F5ai collection consumer item lookup exactly once
    if F5ai Err:
        return the same error
    if F5ai Ok:
        return consumer_item
```

The start consume-once helper order is:

```text
call F5ak start consumer item exactly once
if start consumer item Err:
    return the same error
if start consumer item Ok:
    call F5aj collection consumer item consume-once exactly once
    if F5aj Err:
        return the same error
    if F5aj Ok:
        return consume_step
```

The start consume summary helper order is:

```text
call F5ak start consume-once exactly once
if start consume-once Err:
    return the same error
if start consume-once Ok:
    call pure consume summary projection exactly once
    return summary
```

Only the start item helper may call F5ag and F5ah directly. Higher F5ak helpers must use the immediately lower F5ak helper and the already established F5ai/F5aj authority. F5ak must not call byte-backed F4 helpers, accept caller supplied glyphs, inspect action payload variants, call consumer next, advance/drain summaries, allocate `Vec`, push items, traverse a real sink, mutate a sink, rasterize, emit render commands, call platform APIs, or call host text APIs.

## SFNT simple glyph outline point stream item collection path sink action consume summary drain boundary

F5al is the collection-backed bounded consume summary drain. It mirrors the F4ao/F4aq summary advance and drain shape, but its only traversal authority is the F5ak start summary plus F5aj consume-once chain. It never re-enters byte-backed glyph lookup.

The public boundaries are:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_consume_summary_advance_once:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    summary &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryAdvance GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_consumer_consume_summary_drain_budget:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    summary &GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    remaining_steps i32
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError

gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consume_summary_drain_budget:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    remaining_steps i32
    -> Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryDrain GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

The advance-once helper order is:

```text
read summary state exactly once
read summary terminal exactly once
if terminal is Continue item:
    call F5aj collection consume-once exactly once
    if F5aj Err:
        return the same error
    if F5aj Ok consume_step:
        call pure summary projection exactly once
        return Continue next_summary
if terminal is Rejected reason:
    return Ok Rejected reason
if terminal is EndContour:
    return Ok EndContour
```

`Rejected` and `EndContour` are typed success terminals. They are not converted into errors, ignored as no-op, or interpreted as a fallback path.

The drain helper order is:

```text
read summary terminal exactly once
if terminal is Rejected reason:
    return Ok Rejected reason current_summary
if terminal is EndContour:
    return Ok EndContour current_summary
if terminal is Continue:
    if remaining_steps <= 0:
        return Ok StepBudgetExhausted current_summary
    else:
        call F5al advance-once exactly once
        if advance Err:
            return the same error
        if advance Ok Continue next_summary:
            recurse with remaining_steps - 1
        if advance Ok Rejected reason:
            return Ok Rejected reason current_summary
        if advance Ok EndContour:
            return Ok EndContour current_summary
```

The start drain helper composes F5ak start summary and F5al drain only:

```text
call F5ak start consume summary exactly once
if start Err:
    return the same error
if start Ok summary:
    call F5al drain budget exactly once
```

F5al must not allocate `Vec`, push commands, match action payload variants, call lower collection path event / contour / step helpers directly, call F4 byte-backed lookup helpers, call `*_with_tables`, rasterize, render, call platform APIs, call host text measurement, or perform font fallback. The start drain helper additionally must not call F5al advance-once, F5aj consume-once, or F5ak lower start helpers directly; it owns only start summary to drain composition.

## SFNT simple glyph outline point stream item collection path sink action drain outcome boundary

F5am is the collection-backed drain outcome packet boundary. It does not advance traversal. Its only purpose is to attach the authoritative collection capacity to the F5al terminal drain result before later owner-taking outline/path boundaries decide whether an owner can be allocated.

The public boundary is intentionally narrow:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_start_consume_summary_drain_outcome_budget:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    state GuiSfntSimpleGlyphPathSinkActionApplyState
    contour_index i32
    policy &GuiSfntSimpleGlyphPathSinkPolicy
    remaining_steps i32
    -> Result GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainOutcome GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepError
```

There is a private projection helper, but it is not a public API. This prevents a caller from forging an arbitrary pairing between one collection capacity and a drain result produced from another collection or from another traversal policy.

The public helper order is:

```text
call F5al start drain exactly once
if F5al start drain Err:
    return the same error
if F5al start drain Ok drain:
    call private outcome projection exactly once
    return Ok outcome
```

The private projection order is:

```text
read collection capacity exactly once
match drain exactly once
if drain is EndContour summary:
    construct DrainSummary capacity summary
    return EndContour DrainSummary
if drain is Rejected rejected:
    construct DrainRejected capacity rejected
    return Rejected DrainRejected
if drain is StepBudgetExhausted summary:
    construct DrainSummary capacity summary
    return StepBudgetExhausted DrainSummary
```

`GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainSummary` stores `GuiSfntSimpleGlyphOutlineStorageCapacity` and `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary`. `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainRejected` stores `GuiSfntSimpleGlyphOutlineStorageCapacity` and the existing `GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryRejected`. The outcome enum has only `EndContour`, `Rejected`, and `StepBudgetExhausted`; each branch keeps enough typed data for the next boundary to use `match` instead of stringly state or fallback.

F5am must not allocate an owner, push path commands, consume another item, call lower F5 helpers, call F4 byte-backed lookup helpers, call table helpers, traverse a sink, mutate a sink, rasterize, render, call platform APIs, call host text measurement, or perform font fallback. The public helper must call F5al start drain once and the private projection once; the private projection must not call F5al start drain, F5al advance/drain, F5ak, F5aj, lower path helpers, or byte-backed lookup.

## SFNT simple glyph outline point stream item collection path sink action storage owner boundary

F5an consumes the F5am drain outcome and decides whether outline storage owner allocation is allowed. The F5am outcome is the only authority. F5an does not accept a separate collection, a separate drain value, byte-backed tables, path sink state, or rendering context.

The public boundary is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_drain_outcome_alloc_storage_owner:
    outcome GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionDrainOutcome
    limit &GuiSfntSimpleGlyphOutlineStorageLimit
    -> Result GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionStorageTerminal GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionStorageAllocError
```

The terminal and error shape separates domain terminals from allocation failure:

```text
StorageTerminal:
    Allocated StorageOwner
    Rejected DrainRejected
    StepBudgetExhausted DrainSummary

StorageAllocError:
    summary DrainSummary
    alloc_error GuiSfntSimpleGlyphOutlineStorageAllocError
```

`Rejected` and `StepBudgetExhausted` are not storage allocation failures. They are typed terminals produced by the preceding drain boundary, so they remain `Result::Ok` values. Only F5b storage allocation failure is returned as `Result::Err StorageAllocError`.

The owner allocation order is:

```text
match outcome exactly once
if outcome is EndContour drain_summary:
    read capacity from drain_summary exactly once
    call F5b storage allocation exactly once
    if allocation Ok storage:
        construct StorageOwner storage drain_summary
        return Ok Allocated StorageOwner
    if allocation Err alloc_error:
        construct StorageAllocError drain_summary alloc_error
        return Err StorageAllocError
if outcome is Rejected drain_rejected:
    return Ok Rejected drain_rejected
if outcome is StepBudgetExhausted drain_summary:
    return Ok StepBudgetExhausted drain_summary
```

`StorageOwner` keeps the allocated `GuiSfntSimpleGlyphOutlineStorage` and the F5am drain summary. It is an owner type and must not implement `Clone` or `Copy`. `StorageTerminal` includes that owner, so it also must not implement `Clone` or `Copy`. `StorageAllocError` keeps only copyable context and may implement `Clone` / `Copy`.

F5an intentionally allocates only empty F5b storage. It does not populate scalar slots, does not fill path command owners, does not traverse the sink, and does not render. Those steps need separate owner-recovery boundaries because each can fail or consume ownership independently.

F5an must not call F5al start/drain/advance, F5ak, F5aj, F4 byte-backed lookup helpers, lower collection path helpers, table helpers, `Vec`, `push`, path command fill, sink mutation, rasterization, rendering, platform APIs, host text measurement, or font fallback. Source policy must pin that `Rejected` and `StepBudgetExhausted` branches do not call storage allocation.

## SFNT simple glyph outline point stream item collection path sink action contour endpoint start boundary

F5ao consumes the F5an storage terminal and starts the contour endpoint scalar region only when F5an produced an allocated storage owner. This is still a planning boundary for scalar slot population. It starts a cursor and carries owner-recovery state; it does not push any endpoint slot.

The public boundary is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_storage_terminal_start_contour_endpoint:
    terminal GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionStorageTerminal
    -> Result GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointStartTerminal GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointStartError
```

The terminal and error shape is:

```text
ContourEndpointStartTerminal:
    Started ContourEndpointStartOwner
    Rejected DrainRejected
    StepBudgetExhausted DrainSummary

ContourEndpointStartOwner:
    storage GuiSfntSimpleGlyphOutlineStorage
    summary DrainSummary
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor
    previous_endpoint Option i32

ContourEndpointStartError:
    owner StorageOwner
    kind ContourEndpointStartErrorKind
    cursor_error Option StdErrorKind

ContourEndpointStartErrorKind:
    StorageSummaryCapacityMismatch
    CursorStartFailed
```

F5ao must treat the F5an `StorageOwner` as forgeable because it has a public constructor. The root guard is a non-consuming storage capacity accessor:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_storage_owner_storage_capacity:
    owner &StorageOwner
    -> GuiSfntSimpleGlyphOutlineStorageCapacity
```

This accessor borrows `owner.storage` with `field::get_ref owner "storage"` and calls `gui_sfnt_simple_glyph_outline_storage_capacity storage`. It must not call the consuming `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_storage_owner_storage owner` accessor.

The `Allocated owner` order is:

```text
borrow owner and compare summary capacity with storage capacity
if capacities mismatch:
    return Err ContourEndpointStartError owner StorageSummaryCapacityMismatch none
read summary from owner
read capacity from summary
call gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity ContourEndpoint
if cursor start Err cursor_error:
    return Err ContourEndpointStartError owner CursorStartFailed some cursor_error
if cursor start Ok cursor:
    consume owner.storage exactly once
    construct ContourEndpointStartOwner storage summary cursor none
    return Ok Started ContourEndpointStartOwner
```

The capacity comparison covers glyph, contour count, point count, edge count, path command pair count, and path command count. Glyph comparison uses the existing raw `GuiGlyphId` comparison helper so that two wrapper values only match when their raw glyph ids match.

`Rejected` and `StepBudgetExhausted` are pass-through domain terminals. Those branches do not call capacity match, non-consuming storage capacity read, cursor start, or consuming storage accessors. They remain `Result::Ok` values because no new fallible operation is attempted.

`ContourEndpointStartOwner`, `ContourEndpointStartError`, and `ContourEndpointStartTerminal` contain owner values and must not implement `Clone` or `Copy`. `ContourEndpointStartErrorKind` is a small value enum and may implement `Clone` / `Copy`.

F5ao must not call F5al start/drain/advance, F5ak, F5aj, F4 byte-backed lookup helpers, lower collection path helpers, table helpers, `Vec`, `push`, endpoint push, point or curve population, path command fill, sink mutation, rasterization, rendering, platform APIs, host text measurement, or font fallback.

## SFNT simple glyph outline point stream item collection path sink action contour endpoint push boundary

F5ap consumes the F5ao contour endpoint start terminal and pushes exactly one typed `GuiSfntSimpleGlyphContourEndpointSlot` when the terminal is `Started`. This boundary connects the action-level owner chain to the existing F5e storage endpoint push contract. It does not read `glyf` bytes, does not iterate all contour endpoints, and does not proceed to point x/y, edge, path command, raster, or render work.

The public boundary is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_contour_endpoint_start_terminal_push_endpoint:
    terminal GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointStartTerminal
    endpoint GuiSfntSimpleGlyphContourEndpointSlot
    -> Result GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointPushTerminal GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionContourEndpointPushError
```

The terminal and error shape is:

```text
ContourEndpointPushTerminal:
    Pushed ContourEndpointPushOwner
    Rejected DrainRejected
    StepBudgetExhausted DrainSummary

ContourEndpointPushOwner:
    storage GuiSfntSimpleGlyphOutlineStorage
    summary DrainSummary
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor
    previous_endpoint Option i32

ContourEndpointPushError:
    owner ContourEndpointStartOwner
    endpoint GuiSfntSimpleGlyphContourEndpointSlot
    push_error_kind GuiSfntSimpleGlyphContourEndpointPushErrorKind
    region_error_kind Option GuiSfntSimpleGlyphOutlineRegionPushErrorKind
    storage_push_error_kind Option StdErrorKind
```

The `Started owner` order is:

```text
borrow summary from start owner
borrow cursor from start owner
borrow previous endpoint from start owner
consume start owner storage exactly once
call F5e gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint exactly once
if F5e returns Ok pushed:
    read next_cursor from pushed
    read next_previous_endpoint_value from pushed
    wrap next_previous_endpoint_value as some
    consume pushed storage
    construct ContourEndpointPushOwner next_storage summary next_cursor some_previous_endpoint
    return Ok Pushed push_owner
if F5e returns Err push_error:
    read push_error_kind from push_error
    read region_error_kind from push_error
    read storage_push_error_kind from push_error
    consume returned storage from push_error
    reconstruct ContourEndpointStartOwner returned_storage summary cursor previous_endpoint
    construct ContourEndpointPushError recovered_owner endpoint push_error_kind region_error_kind storage_push_error_kind
    return Err ContourEndpointPushError
```

The error branch must read lower metadata before consuming `push_error` with `gui_sfnt_simple_glyph_contour_endpoint_push_error_storage push_error`. After that call the lower error owner is gone, so all diagnostics needed by the action-level error must already be copied.

The success branch must use F5e returned state, not recompute state from the input endpoint. `ContourEndpointPushOwner.cursor` comes from `gui_sfnt_simple_glyph_contour_endpoint_push_cursor &pushed`, storage comes from `gui_sfnt_simple_glyph_contour_endpoint_push_storage pushed`, and previous endpoint is `some` of `gui_sfnt_simple_glyph_contour_endpoint_push_previous_endpoint &pushed`.

`Rejected` and `StepBudgetExhausted` are pass-through domain terminals. Those branches do not inspect the endpoint argument, call F5e, consume storage, or construct owner/error payloads. They remain `Result::Ok` values because no new fallible endpoint push is attempted.

`ContourEndpointPushOwner`, `ContourEndpointPushError`, and `ContourEndpointPushTerminal` contain owner values and must not implement `Clone` or `Copy`.

F5ap may call only the typed F5e endpoint push helper. It must not call `gui_sfnt_glyf_read_push_contour_endpoint`, `gui_sfnt_glyf_read_contour_endpoint`, F4 byte-backed lookup helpers, F5al/F5ak/F5aj traversal helpers, lower collection path helpers, table helpers, `Vec`, path command fill, sink mutation, rasterization, rendering, platform APIs, host text measurement, or font fallback.

## SFNT simple glyph outline point stream item collection path sink action contour endpoint drain boundary

F5aq consumes an F5ap `ContourEndpointPushOwner`, fills the remaining contour endpoint slots from the already materialized point stream item collection, and starts the PointX scalar region cursor only after the contour endpoint region is complete. It is not a full outline builder and does not push PointX values.

Because `ContourEndpointPushOwner` is publicly constructible, F5aq treats it as forgeable. The public drain boundary must prove all three authorities match before interpreting the cursor or consuming storage:

```text
authority check order:
    read summary capacity from PushOwner
    read owner storage capacity without consuming PushOwner
    require summary capacity == owner storage capacity
    read PushOwner cursor
    require cursor well formed
    require cursor region is ContourEndpoint
    require cursor start/end match summary capacity ContourEndpoint region
    read collection capacity
    require collection capacity == summary capacity
```

The capacity comparison covers glyph raw id, contour count, point count, edge count, path command pair count, and path command count. These checks must precede any call to `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span`, any PointX cursor start, and any consuming owner storage accessor.

The public boundary is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_contour_endpoint_push_owner_drain_to_point_x_start_budget:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner ContourEndpointPushOwner
    remaining_steps i32
    -> Result ContourEndpointDrainTerminal ContourEndpointDrainError
```

The terminal and owner shape is:

```text
PointXStartOwner:
    storage GuiSfntSimpleGlyphOutlineStorage
    summary DrainSummary
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor

ContourEndpointDrainTerminal:
    PointXStarted PointXStartOwner
    StepBudgetExhausted ContourEndpointPushOwner
```

`PointXStarted` means only that the PointX cursor has been started. It does not mean any x coordinate value was decoded or pushed. That work belongs to the next PointX population boundary.

The error shape is:

```text
ContourEndpointDrainErrorKind:
    StorageSummaryCapacityMismatch
    CursorInvalid
    CursorRegionMismatch
    CursorCapacityMismatch
    CollectionSummaryCapacityMismatch
    EndpointSourceFailed
    EndpointPushFailed
    PointXCursorStartFailed

ContourEndpointDrainError:
    owner ContourEndpointPushOwner
    kind ContourEndpointDrainErrorKind
    contour_index i32
    source_error Option ContourSpanError
    endpoint Option GuiSfntSimpleGlyphContourEndpointSlot
    push_error_kind Option GuiSfntSimpleGlyphContourEndpointPushErrorKind
    region_error_kind Option GuiSfntSimpleGlyphOutlineRegionPushErrorKind
    storage_push_error_kind Option StdErrorKind
    cursor_error_kind Option StdErrorKind
```

Each authority failure returns `ContourEndpointDrainError` with the original PushOwner and no lower optional payload. `EndpointSourceFailed` stores the lower collection contour span error. `EndpointPushFailed` stores the rejected endpoint and lower F5e/F5d/F5c metadata. `PointXCursorStartFailed` stores the lower cursor start `StdErrorKind`.

After authority success, cursor interpretation is valid:

```text
if next_index == end:
    start PointX cursor from summary capacity
    if cursor start fails:
        return Err PointXCursorStartFailed owner lower_cursor_error
    consume PushOwner storage
    return Ok PointXStarted PointXStartOwner storage summary point_x_cursor

if next_index < end and remaining_steps <= 0:
    return Ok StepBudgetExhausted owner

if next_index < end and remaining_steps > 0:
    contour_index = next_index - start
    call collection contour span once
    on span failure:
        return Err EndpointSourceFailed owner lower_span_error
    endpoint = GuiSfntSimpleGlyphContourEndpointSlot contour_index span.end_point_index
    call internal PushOwner endpoint push helper once
    on push failure:
        return Err EndpointPushFailed recovered_push_owner lower_metadata
    recurse with returned PushOwner and remaining_steps - 1
```

The internal PushOwner endpoint push helper is the only F5aq helper allowed to call F5e `gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint`. It must borrow summary, cursor, and previous endpoint before consuming storage. If F5e fails, it must read `gui_sfnt_simple_glyph_contour_endpoint_push_error_kind &push_error`, `gui_sfnt_simple_glyph_contour_endpoint_push_error_region_error_kind &push_error`, and `gui_sfnt_simple_glyph_contour_endpoint_push_error_push_error_kind &push_error` before calling `gui_sfnt_simple_glyph_contour_endpoint_push_error_storage push_error`. The recovered storage plus saved summary/cursor/previous endpoint reconstruct the current `ContourEndpointPushOwner`.

F5aq may call `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span` only after the public authority checks pass. It must not call `gui_sfnt_glyf_read_push_contour_endpoint`, `gui_sfnt_glyf_read_contour_endpoint`, F4 byte-backed lookup helpers, F5al/F5ak/F5aj traversal helpers, lower collection path helpers, table helpers, `Vec`, path command fill, sink mutation, PointX push, rasterization, rendering, platform APIs, host text measurement, or font fallback.

`PointXStartOwner`, `ContourEndpointDrainError`, and `ContourEndpointDrainTerminal` contain owner values and must not implement `Clone` or `Copy`. `ContourEndpointDrainErrorKind` is a small value enum and may implement `Clone` / `Copy`.

## SFNT simple glyph outline point stream item collection path sink action PointX drain boundary

F5ar consumes an F5aq `PointXStartOwner`, fills PointX scalar slots from the already materialized point stream item collection, and starts the PointY scalar region cursor only after the PointX region is complete. It is not a full outline builder and does not push PointY values.

Because `PointXStartOwner` is publicly constructible, F5ar treats it as forgeable. The public drain boundary must prove all three authorities match before interpreting the cursor or consuming storage:

```text
authority check order:
    read summary capacity from PointXStartOwner
    read owner storage capacity without consuming PointXStartOwner
    require summary capacity == owner storage capacity
    read PointXStartOwner cursor
    require cursor well formed
    require cursor region is PointX
    require cursor start/end match summary capacity PointX region
    read collection capacity
    require collection capacity == summary capacity
```

The capacity comparison covers glyph raw id, contour count, point count, edge count, path command pair count, and path command count. These checks must precede any call to `gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item`, any PointX push, any PointY cursor start, and any consuming owner storage accessor.

The public boundary is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_point_x_start_owner_drain_to_point_y_start_budget:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner PointXStartOwner
    remaining_steps i32
    -> Result PointXDrainTerminal PointXDrainError
```

The terminal and owner shape is:

```text
PointYStartOwner:
    storage GuiSfntSimpleGlyphOutlineStorage
    summary DrainSummary
    cursor GuiSfntSimpleGlyphOutlineScalarRegionCursor

PointXDrainTerminal:
    PointYStarted PointYStartOwner
    StepBudgetExhausted PointXStartOwner
```

`PointYStarted` means only that the PointY cursor has been started. It does not mean any y coordinate value was pushed. That work belongs to the next PointY population boundary.

The error shape is:

```text
PointXDrainErrorKind:
    StorageSummaryCapacityMismatch
    CursorInvalid
    CursorRegionMismatch
    CursorCapacityMismatch
    CollectionSummaryCapacityMismatch
    PointSourceReadFailed
    PointSourceGlyphMismatch
    PointSourceIndexMismatch
    PointSourceKindMismatch
    PointXPushFailed
    PointYCursorStartFailed

PointXDrainError:
    owner PointXStartOwner
    kind PointXDrainErrorKind
    point_index i32
    read_error Option PointStreamItemCollectionReadError
    item Option PointStreamItem
    point Option PointXSlot
    push_error_kind Option PointXPushErrorKind
    region_error_kind Option GuiSfntSimpleGlyphOutlineRegionPushErrorKind
    storage_push_error_kind Option StdErrorKind
    cursor_error_kind Option StdErrorKind
```

Each authority failure returns `PointXDrainError` with the original PointXStartOwner and no lower optional payload. `PointSourceReadFailed` stores the lower collection read error. `PointSourceGlyphMismatch`, `PointSourceIndexMismatch`, and `PointSourceKindMismatch` store the rejected item. `PointXPushFailed` stores the rejected PointX slot and lower F5g/F5d/F5c metadata. `PointYCursorStartFailed` stores the lower cursor start `StdErrorKind`.

After authority success, cursor interpretation is valid:

```text
if next_index == end:
    start PointY cursor from summary capacity
    if cursor start fails:
        return Err PointYCursorStartFailed owner lower_cursor_error
    consume PointXStartOwner storage
    return Ok PointYStarted PointYStartOwner storage summary point_y_cursor

if next_index < end and remaining_steps <= 0:
    return Ok StepBudgetExhausted owner

if next_index < end and remaining_steps > 0:
    point_index = next_index - start
    call collection read item once
    on read failure:
        return Err PointSourceReadFailed owner lower_read_error
    validate item glyph == capacity glyph
    validate item point_index == point_index
    validate item kind matches point payload
    point_x = PointXSlot point_index item_point.x
    call internal PointXStartOwner point-x push helper once
    on push failure:
        return Err PointXPushFailed recovered_point_x_owner lower_metadata
    recurse with returned PointXStartOwner and remaining_steps - 1
```

The collection read helper validates collection length, capacity, and requested index. It does not prove that a public-constructor item payload still matches the item index, glyph, or kind. Therefore F5ar must perform glyph, point index, and kind checks after read and before PointX push. This caller-side validation is the boundary that makes forged collection items fail closed without changing the lower read helper.

The internal PointX push helper is the only F5ar helper allowed to call F5g `gui_sfnt_simple_glyph_outline_storage_push_point_x`. It must borrow summary and cursor before consuming storage. If F5g fails, it must read `gui_sfnt_simple_glyph_point_x_push_error_kind &push_error`. It must read `gui_sfnt_simple_glyph_point_x_push_error_point &push_error`. It must read `gui_sfnt_simple_glyph_point_x_push_error_region_error_kind &push_error`. It must read `gui_sfnt_simple_glyph_point_x_push_error_push_error_kind &push_error`. These reads must happen before calling `gui_sfnt_simple_glyph_point_x_push_error_storage push_error`. The recovered storage plus saved summary/cursor reconstruct the current `PointXStartOwner`.

F5ar may call `gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item` only after the public authority checks pass and only when `remaining_steps > 0`. It must not call `gui_sfnt_glyf_read_push_point_x`, `gui_sfnt_glyf_read_point_x_from_stream`, `gui_sfnt_glyf_read_push_point_y`, `gui_sfnt_glyf_read_point_y_from_stream`, F4 byte-backed lookup helpers, F5al/F5ak/F5aj traversal helpers, lower collection path helpers, table helpers, `Vec`, path command fill, sink mutation, PointY push, rasterization, rendering, platform APIs, host text measurement, or font fallback.

`PointYStartOwner`, `PointXDrainError`, and `PointXDrainTerminal` contain owner values and must not implement `Clone` or `Copy`. `PointXDrainErrorKind` is a small value enum and may implement `Clone` / `Copy`.

## SFNT simple glyph outline point stream item collection path sink action PointY drain boundary

F5as consumes an F5ar `PointYStartOwner`, fills PointY scalar slots from the already materialized point stream item collection, and starts the Edge scalar region cursor only after the PointY region is complete. It is not an edge builder and does not populate edge values.

Because `PointYStartOwner` is publicly constructible, F5as treats it as forgeable. The public drain boundary must prove all three authorities match before interpreting the cursor or consuming storage:

```text
authority check order:
    read summary capacity from PointYStartOwner
    read owner storage capacity without consuming PointYStartOwner
    require summary capacity == owner storage capacity
    read cursor from PointYStartOwner
    require cursor well formed
    require cursor region is PointY
    require cursor matches summary capacity PointY region
    read collection capacity
    require collection capacity == summary capacity
```

The capacity comparison covers glyph raw id, contour count, point count, edge count, path command pair count, and path command count. Cursor validation covers start / next / end bounds and the PointY region boundaries derived from summary capacity.

The public API shape is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_point_y_start_owner_drain_to_edge_start_budget:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner PointYStartOwner
    remaining_steps i32
    -> Result PointYDrainTerminal PointYDrainError
```

The owned types are:

```text
EdgeStartOwner:
    storage
    summary
    cursor

PointYDrainTerminal:
    EdgeStarted EdgeStartOwner
    StepBudgetExhausted PointYStartOwner

PointYDrainError:
    owner PointYStartOwner
    kind PointYDrainErrorKind
    point_index i32
    read_error Option CollectionReadError
    item Option PointStreamItem
    point Option PointYSlot
    push_error_kind Option PointYPushErrorKind
    region_error_kind Option RegionPushErrorKind
    storage_push_error_kind Option StdErrorKind
    cursor_error_kind Option StdErrorKind
```

`PointYDrainErrorKind` separates authority failure, source failure, push failure, and Edge cursor start failure:

```text
StorageSummaryCapacityMismatch
CursorInvalid
CursorRegionMismatch
CursorCapacityMismatch
CollectionSummaryCapacityMismatch
PointSourceReadFailed
PointSourceGlyphMismatch
PointSourceIndexMismatch
PointSourceKindMismatch
PointYPushFailed
EdgeCursorStartFailed
```

Each authority failure returns `PointYDrainError` with the original PointYStartOwner. No authority failure may consume storage or read a collection item.

The trusted drain body runs only after the public authority checks:

```text
cursor = owner.cursor
next_index = cursor.next_index
end = cursor.end

if next_index == end:
    start Edge cursor from summary capacity
    if cursor start fails:
        return EdgeCursorStartFailed with current PointYStartOwner
    consume storage from PointYStartOwner
    return EdgeStarted EdgeStartOwner

if remaining_steps <= 0:
    return StepBudgetExhausted PointYStartOwner

point_index = next_index - cursor.start
call collection read item once
validate item glyph == capacity glyph
validate item point_index == point_index
validate item kind matches point payload
build PointYSlot from item y coordinate
call internal PointYStartOwner point-y push helper once
recurse with remaining_steps - 1
```

Completion is checked before the step budget. This lets a caller finish a boundary and advance to `EdgeStartOwner` even with zero remaining mutation steps when the PointY region is already complete. StepBudgetExhausted is checked before `collection_read_item` and before PointY push, so a budget-limited caller can retry deterministically without duplicate reads or mutations.

The collection read helper validates collection length, capacity, and requested index. It does not prove that a public-constructor item payload still matches the item index, glyph, or kind. Therefore F5as must perform glyph, point index, and kind checks after read and before PointY push. This caller-side validation is the boundary that makes forged collection items fail closed without changing the lower read helper.

The internal PointY push helper is the only F5as helper allowed to call F5i `gui_sfnt_simple_glyph_outline_storage_push_point_y`. It must borrow summary and cursor before consuming storage. If F5i fails, it must read `gui_sfnt_simple_glyph_point_y_push_error_kind &push_error`. It must read `gui_sfnt_simple_glyph_point_y_push_error_point &push_error`. It must read `gui_sfnt_simple_glyph_point_y_push_error_region_error_kind &push_error`. It must read `gui_sfnt_simple_glyph_point_y_push_error_push_error_kind &push_error`. These reads must happen before calling `gui_sfnt_simple_glyph_point_y_push_error_storage push_error`. The recovered storage plus saved summary/cursor reconstruct the current `PointYStartOwner`.

F5as may call `gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item` only after the public authority checks pass and only when `remaining_steps > 0`. It may call `gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity` for `GuiSfntSimpleGlyphOutlineScalarRegion::Edge` only in the completion branch. It must not call `gui_sfnt_glyf_read_push_point_x`, `gui_sfnt_glyf_read_point_x_from_stream`, `gui_sfnt_glyf_read_push_point_y`, `gui_sfnt_glyf_read_point_y_from_stream`, F4 byte-backed lookup helpers, F5al/F5ak/F5aj traversal helpers, lower collection path helpers, table helpers, `Vec`, path command fill, sink mutation, PointX push, edge value population, path command population, rasterization, rendering, platform APIs, host text measurement, or font fallback.

`EdgeStartOwner`, `PointYDrainError`, and `PointYDrainTerminal` contain owner values and must not implement `Clone` or `Copy`. `PointYDrainErrorKind` is a value enum and may implement `Clone` and `Copy`.

## SFNT simple glyph outline point stream item collection path sink action Edge drain boundary

F5at consumes an F5as `EdgeStartOwner`, fills Edge scalar slots from owner storage endpoint markers and collection-backed contour sources, and starts the PathCommandTag scalar region cursor only after the Edge region is complete. It is not a curve segment classifier and does not populate path command tags.

The Edge scalar has a deliberately narrow meaning:

```text
global_edge_index = cursor.next_index - cursor.start
slot global_edge_index represents the absolute start point index
endpoint marker gives contour_index for global_edge_index
collection contour span gives span.start_point_index
contour_edge_index = global_edge_index - span.start_point_index
stored scalar value = contour_index
```

The Edge region stores only contour ownership. The local edge index is derived from the global edge index and the span. This keeps path command classification in the next PathCommandTag phase, where curve segment source can be read with a fresh authority check.

Because `EdgeStartOwner` is publicly constructible, F5at treats it as forgeable. The public drain boundary must prove all three authorities match before interpreting the cursor, reading endpoint marker, reading collection source, or consuming storage:

```text
authority check order:
    read summary capacity from EdgeStartOwner
    read owner storage capacity without consuming EdgeStartOwner
    require summary capacity == owner storage capacity
    read cursor from EdgeStartOwner
    require cursor well formed
    require cursor region is Edge
    require cursor matches summary capacity Edge region
    read collection capacity
    require collection capacity == summary capacity
```

The capacity comparison covers glyph raw id, contour count, point count, edge count, path command pair count, and path command count. Cursor validation covers start / next / end bounds and the Edge region boundaries derived from summary capacity. F5at must not require `cursor.next_index == cursor.start`; a resumed `StepBudgetExhausted EdgeStartOwner` may point inside the Edge region.

The public API shape is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_edge_start_owner_drain_to_path_command_tag_start_budget:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner EdgeStartOwner
    remaining_steps i32
    -> Result EdgeDrainTerminal EdgeDrainError
```

The owned and value types are:

```text
PathCommandTagStartOwner:
    storage
    summary
    cursor

EdgeSlot:
    edge_index i32
    contour_index i32
    contour_edge_index i32
    next_contour_point_index i32

EdgeDrainTerminal:
    PathCommandTagStarted PathCommandTagStartOwner
    StepBudgetExhausted EdgeStartOwner

EdgeDrainError:
    owner EdgeStartOwner
    kind EdgeDrainErrorKind
    edge_index i32
    endpoint_error Option PointEndpointMarkerReadError
    span_error Option CollectionContourSpanError
    span Option ContourSpan
    edge_error Option CollectionContourEdgeError
    edge Option ContourEdge
    edge_slot Option EdgeSlot
    scalar_value Option i32
    region_error_kind Option RegionPushErrorKind
    storage_push_error_kind Option StdErrorKind
    cursor_error_kind Option StdErrorKind
```

`EdgeDrainErrorKind` separates authority failure, endpoint marker failure, collection source failure, forged source failure, push failure, and PathCommandTag cursor start failure:

```text
StorageSummaryCapacityMismatch
CursorInvalid
CursorRegionMismatch
CursorCapacityMismatch
CollectionSummaryCapacityMismatch
EndpointMarkerReadFailed
EndpointMarkerGlyphMismatch
EndpointMarkerIndexMismatch
ContourSpanSourceFailed
ContourSpanInvariantMismatch
ContourEdgeSourceFailed
EdgeSourceContourMismatch
EdgeSourceIndexMismatch
EdgeSourceNextIndexMismatch
EdgePushFailed
PathCommandTagCursorStartFailed
```

Each authority failure returns `EdgeDrainError` with the original EdgeStartOwner. No authority failure may consume storage, read an endpoint marker, or read a collection contour source.

The trusted drain body runs only after the public authority checks:

```text
cursor = owner.cursor
next_index = cursor.next_index
end = cursor.end

if next_index == end:
    start PathCommandTag cursor from summary capacity
    if cursor start fails:
        return PathCommandTagCursorStartFailed with current EdgeStartOwner
    consume storage from EdgeStartOwner
    return PathCommandTagStarted PathCommandTagStartOwner

if remaining_steps <= 0:
    return StepBudgetExhausted EdgeStartOwner

edge_index = next_index - cursor.start
call private non-consuming endpoint marker helper once
validate marker glyph == capacity glyph
validate marker point_index == edge_index
call collection contour span once
validate span glyph/index/range/count and span contains edge_index
contour_edge_index = edge_index - span.start_point_index
call collection contour edge once
validate edge contour/local index/absolute start/wrap next
build EdgeSlot
call internal EdgeStartOwner edge push helper once
recurse with remaining_steps - 1
```

Completion is checked before the step budget. This lets a caller finish a boundary and advance to `PathCommandTagStartOwner` even with zero remaining mutation steps when the Edge region is already complete. StepBudgetExhausted is checked before endpoint marker read, collection span / edge source, and Edge push, so a budget-limited caller can retry deterministically without duplicate reads or mutations.

The private endpoint marker helper is:

```text
edge_start_owner_read_endpoint_marker:
    owner &EdgeStartOwner
    edge_index i32
    -> Result PointEndpointMarker PointEndpointMarkerReadError
```

It reads storage with `field::get_ref owner "storage"` and calls `gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker storage edge_index`. It never consumes the owner. The only F5at helper allowed to consume EdgeStartOwner storage during mutation is the internal push helper, and the only completion branch allowed to consume EdgeStartOwner storage is the successful PathCommandTag cursor start branch.

The internal Edge push helper is the only F5at helper allowed to call `gui_sfnt_simple_glyph_outline_storage_push_region_scalar`. It must borrow summary and cursor before consuming storage. If F5d fails, it must read `gui_sfnt_simple_glyph_outline_region_push_error_kind &push_error`. It must read `gui_sfnt_simple_glyph_outline_region_push_error_scalar_value &push_error`. It must read `gui_sfnt_simple_glyph_outline_region_push_error_push_error_kind &push_error`. These reads must happen before calling `gui_sfnt_simple_glyph_outline_region_push_error_storage push_error`. The recovered storage plus saved summary/cursor reconstruct the current `EdgeStartOwner`.

F5at may call `gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker` only through the private non-consuming EdgeStartOwner helper, and only after the public authority checks pass and `remaining_steps > 0`. It may call `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span` and `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge` only in the same branch. It may call `gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity` for `GuiSfntSimpleGlyphOutlineScalarRegion::PathCommandTag` only in the completion branch. It must not call `gui_sfnt_simple_glyph_outline_point_stream_item_collection_curve_segment`, byte-backed lookup helpers, table helpers, F5al/F5ak/F5aj traversal helpers, lower collection path step/event helpers, `Vec`, path command fill, sink mutation, PointX / PointY push, path command tag population, rasterization, rendering, platform APIs, host text measurement, or font fallback.

`PathCommandTagStartOwner`, `EdgeDrainError`, and `EdgeDrainTerminal` contain owner values and must not implement `Clone` or `Copy`. `EdgeSlot` and `EdgeDrainErrorKind` are small value types and may implement `Clone` and `Copy`.

## SFNT simple glyph outline point stream item collection path sink action PathCommandTag drain boundary

F5au consumes an F5at `PathCommandTagStartOwner`, fills PathCommandTag scalar slots from owner storage Edge owner scalars and collection-backed path sink event kind source, and returns a complete owner only after the PathCommandTag region is complete. It is not a path command stream builder and does not rasterize or render.

The PathCommandTag scalar has a stable value-only meaning:

```text
logical_path_command_index = cursor.next_index - cursor.start
edge_index = div_s logical_path_command_index 2
event_slot_ordinal = rem_s logical_path_command_index 2
event_slot_ordinal 0 -> First
event_slot_ordinal 1 -> Second

stored scalar values:
    MoveTo        1
    LineTo        2
    QuadraticTo   3
    SkipNoSegment 4
```

F5au must never use absolute cursor `next_index` as a command index. `SkipNoSegment` reason is not stored in the scalar slot. The later value/stream boundary must re-read the same collection-backed source to recover the reason.

Because `PathCommandTagStartOwner` is publicly constructible, F5au treats it as forgeable. The public drain boundary must prove all authorities match before interpreting the cursor, reading owner storage Edge scalar, reading collection source, or consuming storage:

```text
authority check order:
    read summary capacity from PathCommandTagStartOwner
    read owner storage capacity without consuming PathCommandTagStartOwner
    require summary capacity == owner storage capacity
    read cursor from PathCommandTagStartOwner
    require cursor well formed
    require cursor region is PathCommandTag
    require cursor matches summary capacity PathCommandTag region
    read collection capacity
    require collection capacity == summary capacity
```

Cursor validation covers start / next / end bounds and the PathCommandTag region boundaries derived from summary capacity. F5au must not require `cursor.next_index == cursor.start`; a resumed `StepBudgetExhausted PathCommandTagStartOwner` may point inside the PathCommandTag region.

The public API shape is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_tag_start_owner_drain_to_complete_budget:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner PathCommandTagStartOwner
    remaining_steps i32
    -> Result PathCommandTagDrainTerminal PathCommandTagDrainError
```

The owned and value types are:

```text
PathCommandTagSlot:
    path_command_index i32
    edge_index i32
    contour_index i32
    contour_edge_index i32
    event_slot PathSinkEventSlot
    tag PathCommandTag

PathCommandTagCompleteOwner:
    storage
    summary

PathCommandTagDrainTerminal:
    PathCommandTagCompleted PathCommandTagCompleteOwner
    StepBudgetExhausted PathCommandTagStartOwner

PathCommandTagDrainError:
    owner PathCommandTagStartOwner
    kind PathCommandTagDrainErrorKind
    path_command_index i32
    edge_index i32
    edge_owner_error Option EdgeOwnerReadError
    edge_owner Option EdgeOwnerMarker
    span_error Option CollectionContourSpanError
    span Option ContourSpan
    event_error Option CollectionCurveSegmentError
    event_kind Option PathSinkEventKind
    tag_slot Option PathCommandTagSlot
    scalar_value Option i32
    region_error_kind Option RegionPushErrorKind
    storage_push_error_kind Option StdErrorKind
```

`PathCommandTagDrainErrorKind` separates authority failure, logical index failure, non-consuming Edge owner scalar failure, collection source failure, and push failure:

```text
StorageSummaryCapacityMismatch
CursorInvalid
CursorRegionMismatch
CursorCapacityMismatch
CollectionSummaryCapacityMismatch
PathCommandIndexInvalid
EventSlotOrdinalInvalid
EdgeOwnerReadFailed
EdgeOwnerGlyphMismatch
EdgeOwnerIndexMismatch
ContourSpanSourceFailed
ContourSpanInvariantMismatch
EventKindSourceFailed
TagPushFailed
```

Each authority failure returns `PathCommandTagDrainError` with the original PathCommandTagStartOwner. No authority failure may consume storage, read Edge owner scalar, read collection source, or push PathCommandTag scalar.

The trusted drain body runs only after the public authority checks:

```text
cursor = owner.cursor
next_index = cursor.next_index
end = cursor.end

if next_index == end:
    consume storage from PathCommandTagStartOwner
    return PathCommandTagCompleted PathCommandTagCompleteOwner

if remaining_steps <= 0:
    return StepBudgetExhausted PathCommandTagStartOwner

path_command_index = next_index - cursor.start
validate 0 <= path_command_index < capacity.path_command_count
edge_index = div_s path_command_index 2
event_slot_ordinal = rem_s path_command_index 2
validate ordinal is 0 or 1
call private non-consuming Edge owner helper once
validate marker glyph == capacity glyph
validate marker edge_index == edge_index
call collection contour span once
validate span glyph/index/range/count and span contains edge_index
contour_edge_index = edge_index - span.start_point_index
call path sink event kind source once
map event kind to PathCommandTag
build PathCommandTagSlot
call internal PathCommandTagStartOwner tag push helper once
recurse with remaining_steps - 1
```

Completion is checked before the step budget. This lets a caller finish a boundary and advance to `PathCommandTagCompleteOwner` even with zero remaining mutation steps when the PathCommandTag region is already complete. StepBudgetExhausted is checked before Edge owner scalar read, collection source, and PathCommandTag push, so a budget-limited caller can retry deterministically without duplicate reads or mutations.

The private Edge owner helper is:

```text
path_command_tag_start_owner_read_edge_owner:
    owner &PathCommandTagStartOwner
    edge_index i32
    -> Result EdgeOwnerMarker EdgeOwnerReadError
```

It reads storage with `field::get_ref owner "storage"` and calls `gui_sfnt_simple_glyph_outline_storage_read_edge_owner storage edge_index`. It never consumes the owner. The storage-level helper validates storage capacity shape, scalar slot count, scalar storage capacity, edge range, Edge slot readiness, Edge slot presence, and stored contour index range.

The internal PathCommandTag push helper is the only F5au helper allowed to call `gui_sfnt_simple_glyph_outline_storage_push_region_scalar`. It must borrow summary and cursor before consuming storage. If F5d fails, it must read `gui_sfnt_simple_glyph_outline_region_push_error_kind &push_error`. It must read `gui_sfnt_simple_glyph_outline_region_push_error_scalar_value &push_error`. It must read `gui_sfnt_simple_glyph_outline_region_push_error_push_error_kind &push_error`. These reads must happen before calling `gui_sfnt_simple_glyph_outline_region_push_error_storage push_error`. The recovered storage plus saved summary/cursor reconstruct the current `PathCommandTagStartOwner`.

F5au may call `gui_sfnt_simple_glyph_outline_storage_read_edge_owner` only through the private non-consuming PathCommandTagStartOwner helper, and only after the public authority checks pass and `remaining_steps > 0`. It may call `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span` only in that same branch. It may call `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_at` only in the same branch after the span invariant checks pass. It must not call byte-backed lookup helpers, table helpers, F5al/F5ak/F5aj traversal helpers, old path sink action consumer helpers, path command pair construction, path command stream construction, sink mutation, rasterization, rendering, platform APIs, host text measurement, or font fallback.

`PathCommandTagCompleteOwner`, `PathCommandTagDrainError`, and `PathCommandTagDrainTerminal` contain owner values and must not implement `Clone` or `Copy`. `PathCommandTagSlot`, `PathCommandTagDrainErrorKind`, `PathCommandTag`, `EdgeOwnerMarker`, and `EdgeOwnerReadError` are small value types and may implement `Clone` and `Copy`.

## SFNT simple glyph outline point stream item collection path command value lookup boundary

F5av consumes no owner. It borrows a `PathCommandTagCompleteOwner`, validates it against the collection, reads one stored PathCommandTag scalar, re-reads the collection-backed source event, and returns one typed path command value only if the stored tag and source tag match. It is a value lookup boundary, not a stream builder.

The storage-level scalar read helper is:

```text
gui_sfnt_simple_glyph_outline_storage_read_path_command_tag:
    storage &GuiSfntSimpleGlyphOutlineStorage
    path_command_index i32
    -> Result PathCommandTag PathCommandTagReadError
```

It validates storage capacity shape, scalar slot count, scalar storage capacity, path command index range, PathCommandTag region readiness, scalar slot presence, and known scalar value. Scalar values are the same stable F5au values. Unknown scalar values return `PathCommandTagScalarUnknown` with the observed scalar in the error payload. No default tag is chosen.

The complete owner helpers are:

```text
path_command_tag_complete_owner_storage_capacity:
    owner &PathCommandTagCompleteOwner
    -> GuiSfntSimpleGlyphOutlineStorageCapacity

path_command_tag_complete_owner_read_path_command_tag:
    owner &PathCommandTagCompleteOwner
    path_command_index i32
    -> Result PathCommandTag PathCommandTagReadError

path_command_tag_complete_owner_read_edge_owner:
    owner &PathCommandTagCompleteOwner
    edge_index i32
    -> Result EdgeOwnerMarker EdgeOwnerReadError
```

All three helpers use `field::get_ref owner "storage"` and must not call the consuming `complete_owner_storage` accessor.

The value type is:

```text
PathCommandValue:
    path_command_index i32
    edge_index i32
    contour_index i32
    contour_edge_index i32
    event_slot PathSinkEventSlot
    stored_tag PathCommandTag
    source_tag PathCommandTag
    command PathCommand
```

It is value-only and may implement `Clone` / `Copy`. It carries the source mapping so later stream preparation can preserve diagnostics without recomputing the slot origin.

The error type is value-only:

```text
PathCommandValueError:
    kind PathCommandValueErrorKind
    capacity StorageCapacity
    path_command_index i32
    edge_index i32
    tag_error Option PathCommandTagReadError
    stored_tag Option PathCommandTag
    edge_owner_error Option EdgeOwnerReadError
    edge_owner Option EdgeOwnerMarker
    span_error Option CollectionContourSpanError
    span Option ContourSpan
    event_error Option CollectionCurveSegmentError
    source_event Option PathSinkEvent
    source_tag Option PathCommandTag
```

It contains no storage owner because the public API borrows complete owner and never moves storage.

The public API is:

```text
gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_tag_complete_owner_path_command_value:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner &PathCommandTagCompleteOwner
    path_command_index i32
    -> Result PathCommandValue PathCommandValueError
```

The authority order is strict:

```text
summary = owner.summary
summary_capacity = summary.capacity
storage_capacity = non-consuming complete owner storage capacity
require summary_capacity == storage_capacity
collection_capacity = collection.capacity
require collection_capacity == summary_capacity
require 0 <= path_command_index < summary_capacity.path_command_count
derive edge_index and event_slot
read stored tag
read Edge owner
read collection span
read source event
derive source tag
compare tags
return command
```

The source event read must call `gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_at` exactly once after span validation. The function then derives `source_kind` from that event and maps it to `source_tag`. F5av must not call `path_sink_event_kind_at` separately because that would duplicate source derivation and risk divergence between tag and payload.

This is where `SkipNoSegment` reason is recovered. The stored tag only proves that the scalar region recorded a skip command; the reason comes from `GuiSfntSimpleGlyphPathCommand::SkipNoSegment` inside the source event. A mismatch between stored tag and source tag is `TagMismatch`, never a no-op or inferred replacement command.

F5av may call the collection-backed source event helper and pure event/tag helpers. It must not call byte-backed lookup helpers, metadata parser, table helpers, old action traversal consumers, storage mutation, path command stream builders, `Vec`, rasterization, rendering, platform APIs, host text measurement, or font fallback.

## SFNT simple glyph outline point stream item collection path command stream cursor boundary

F5aw is the first stream preparation boundary after F5av. It does not materialize a path object and does not collect commands into `Vec`. It only gives callers a typed cursor, a one-step read, and a bounded drain terminal that can later be connected to scheduler / render preparation layers.

The cursor is value-only:

```text
PathCommandStreamCursor:
    next_index i32
    end_index i32
```

The public cursor constructor is:

```text
path_command_tag_complete_owner_path_command_stream_cursor:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner &PathCommandTagCompleteOwner
    start_index i32
    -> Result PathCommandStreamCursor PathCommandStreamCursorError
```

It borrows the complete owner. It must not call the consuming complete-owner storage accessor. The authority order is:

```text
summary = owner.summary
summary_capacity = summary.capacity
storage_capacity = non-consuming complete owner storage capacity
require summary_capacity == storage_capacity
collection_capacity = collection.capacity
require collection_capacity == summary_capacity
require capacity shape is valid
path_command_count = summary_capacity.path_command_count
require 0 <= start_index <= path_command_count
cursor = PathCommandStreamCursor start_index path_command_count
```

`start_index == path_command_count` is a completed cursor. It is not an empty-stream fallback. Forged empty / malformed capacity is rejected by the existing capacity-shape contract.

Cursor validation is shared by step:

```text
PathCommandStreamCursorErrorKind:
    StorageSummaryCapacityMismatch
    CollectionSummaryCapacityMismatch
    StorageCapacityInvalid
    StartIndexInvalid
    CursorInvalid
```

`CursorInvalid` covers `end_index != capacity.path_command_count`, `next_index < 0`, and `next_index > end_index`. It is a typed error instead of clamping the index.

The step terminal is an enum:

```text
PathCommandStreamStep:
    Emitted PathCommandValue PathCommandStreamCursor
    Completed PathCommandStreamCursor
```

The completed branch has no dummy value. The step function is:

```text
path_command_stream_step:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner &PathCommandTagCompleteOwner
    cursor PathCommandStreamCursor
    -> Result PathCommandStreamStep PathCommandStreamStepError
```

The step order is:

```text
validate collection / owner capacity authority
validate cursor against capacity
if cursor.next_index >= cursor.end_index:
    return Completed cursor
else:
    call F5av PathCommandValue lookup exactly once
    return Emitted value advanced_cursor
```

The F5av lookup call must be inside the non-terminal branch only. A completed cursor never reads source events and never re-enters F5av.

The bounded drain terminal is also explicit:

```text
PathCommandStreamDrainTerminal:
    Completed PathCommandStreamCursor emitted_count
    StepBudgetExhausted PathCommandStreamCursor emitted_count
```

The drain function is:

```text
path_command_stream_drain_budget:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner &PathCommandTagCompleteOwner
    cursor PathCommandStreamCursor
    remaining_steps i32
    -> Result PathCommandStreamDrainTerminal PathCommandStreamStepError
```

When `remaining_steps <= 0`, drain returns `StepBudgetExhausted cursor 0` and does not call the step helper. With a positive budget, drain calls only the F5aw step helper. It never calls F5av lookup directly. Recursive drain accumulates `emitted_count` and returns either the final completed cursor or the cursor where the budget ended.

F5aw may call:

```text
path_command_tag_complete_owner_summary
path_command_tag_complete_owner_storage_capacity
point_stream_item_collection_capacity
storage_capacity_shape_is_valid
storage_capacity_path_command_count
F5av PathCommandValue lookup from step only
F5aw step helper from drain only
```

F5aw must not call byte-backed lookup helpers, metadata parser, table helpers, old action traversal consumers, storage mutation, `Vec`, path object materialization, rasterization, rendering, platform APIs, host text measurement, or font fallback.

## SFNT simple glyph outline point stream item collection path command stream prepare boundary

F5ax is the first consumer-shaped boundary after F5aw, but it is still not a real sink. It consumes F5aw stream steps into a small value-only preparation summary so later phases can decide how to allocate or schedule command sink / raster mask / render2d work without forcing this phase to build a path object.

The prepare summary is value-only:

```text
PathCommandStreamPrepareSummary:
    total_count i32
    move_to_count i32
    line_to_count i32
    quadratic_to_count i32
    skip_no_segment_count i32
    last_path_command_index i32
```

The initial summary is:

```text
total_count = 0
move_to_count = 0
line_to_count = 0
quadratic_to_count = 0
skip_no_segment_count = 0
last_path_command_index = -1
```

The summary intentionally stores counts only. It does not store command payloads, references to the collection, storage owners, path objects, or renderer commands.

One prepared command is represented by a small domain action:

```text
PathCommandStreamPrepareAction:
    CountedMoveTo
    CountedLineTo
    CountedQuadraticTo
    CountedSkipNoSegment
```

The action is diagnostic / scheduling data only. It is not a render command, not a raster operation, and not a sink callback.

The private update helper reads the command payload from `PathCommandValue` exactly once:

```text
summary_increment_from_value summary value:
    path_command_index = path_command_value_path_command_index value
    command = path_command_value_command value
    match command:
        MoveTo:
            increment total_count and move_to_count
            last_path_command_index = path_command_index
            action = CountedMoveTo
        LineTo:
            increment total_count and line_to_count
            last_path_command_index = path_command_index
            action = CountedLineTo
        QuadraticTo:
            increment total_count and quadratic_to_count
            last_path_command_index = path_command_index
            action = CountedQuadraticTo
        SkipNoSegment:
            increment total_count and skip_no_segment_count
            last_path_command_index = path_command_index
            action = CountedSkipNoSegment
```

The helper returns both the action and the updated summary, so the public step never needs to match the command a second time.

The public prepare step is:

```text
path_command_stream_prepare_step:
    collection &GuiSfntSimpleGlyphOutlinePointStreamItemCollection
    owner &PathCommandTagCompleteOwner
    summary PathCommandStreamPrepareSummary
    cursor PathCommandStreamCursor
    -> Result PathCommandStreamPrepareStep PathCommandStreamPrepareStepError
```

The step terminal is explicit:

```text
PathCommandStreamPrepareStep:
    Prepared PathCommandStreamPrepareAction PathCommandStreamPrepareSummary PathCommandStreamCursor
    Completed PathCommandStreamPrepareSummary PathCommandStreamCursor
```

The completed branch does not carry a dummy action or dummy command value.

The step order is:

```text
call F5aw PathCommandStreamStep exactly once
if lower step returns Err:
    return PrepareStepError current_summary cursor lower_step_error
if lower step returns Completed completed_cursor:
    return Completed current_summary completed_cursor
if lower step returns Emitted value next_cursor:
    update = summary_increment_from_value current_summary value
    return Prepared update.action update.summary next_cursor
```

F5ax does not revalidate the cursor directly. Cursor / owner / collection authority stays in F5aw. F5ax preserves the typed lower error and attaches the current summary and cursor to the error context.

The bounded prepare drain terminal is:

```text
PathCommandStreamPrepareDrainTerminal:
    Completed PathCommandStreamPrepareSummary PathCommandStreamCursor emitted_count
    StepBudgetExhausted PathCommandStreamPrepareSummary PathCommandStreamCursor emitted_count
```

When `remaining_steps <= 0`, prepare drain returns `StepBudgetExhausted summary cursor 0` and does not call the prepare step helper. With a positive budget, drain calls only the F5ax prepare step helper. It never calls F5aw step or F5av lookup directly. Recursive drain accumulates `emitted_count` and carries the updated summary through each prepared step.

F5ax may call:

```text
F5aw PathCommandStreamStep from prepare step only
F5ax prepare step from prepare drain only
PathCommandValue path_command_index accessor
PathCommandValue command accessor
```

F5ax must not call byte-backed lookup helpers, metadata parser, table helpers, old action traversal consumers, F5av lookup, F5aw step from prepare drain, storage mutation, `Vec`, path object materialization, rasterization, rendering, platform APIs, host text measurement, or font fallback.

## SFNT simple glyph outline point stream item collection path command stream sink plan boundary

F5ay converts the completed F5ax prepare drain terminal into the first sink/raster capacity plan. It is deliberately still not a sink writer, not a mask writer, and not a renderer command emitter. Its only job is to prove that the completed command stream summary is internally consistent and to derive exact capacities for the following phase.

The input is the drain terminal, not the summary alone:

```text
sink_plan_from_prepare_drain_terminal terminal:
    match terminal:
        Completed summary cursor emitted_count:
            validate and derive plan
        StepBudgetExhausted summary cursor emitted_count:
            Err PrepareNotCompleted
```

This distinction is required because a budget-exhausted partial summary can have non-negative counts and a valid last index while still not representing the full stream. Treating that summary as final would turn scheduler state into a completed sink plan.

The plan is value-only:

```text
PathCommandStreamSinkPlan:
    total_count i32
    emitted_count i32
    draw_count i32
    move_to_count i32
    line_to_count i32
    quadratic_to_count i32
    skip_no_segment_count i32
    path_segment_capacity i32
    raster_edge_capacity i32
    last_path_command_index i32
```

The source summary remains the authority for command kind counts. The drain terminal `emitted_count` is an independent authority for how many steps were actually emitted by F5ax drain, so F5ay checks `emitted_count == total_count` before returning a plan.

The checked count guard rejects forged or partial count contexts before any derived capacity calculation:

```text
total_count >= 0
move_to_count >= 0
line_to_count >= 0
quadratic_to_count >= 0
skip_no_segment_count >= 0
emitted_count >= 0
total_count > 0
last_path_command_index >= 0
```

Every addition used to derive capacities must be guarded by remaining capacity:

```text
remaining = 2147483647 - left
if right > remaining:
    Err CountOverflow
else:
    Ok left + right
```

F5ay performs guarded addition in this order:

```text
move_line_count = move_to_count + line_to_count
path_segment_capacity = move_line_count + quadratic_to_count
prepared_count = path_segment_capacity + skip_no_segment_count
draw_count = line_to_count + quadratic_to_count
raster_edge_capacity = draw_count
```

Then it checks:

```text
prepared_count == total_count
emitted_count == total_count
draw_count == raster_edge_capacity
```

The `draw_count == raster_edge_capacity` check is redundant in the current implementation because both values are derived together, but it is an intentional source-policy anchor. Future changes must not let draw scheduling and raster edge capacity drift apart silently.

The error is also value-only and carries the original terminal plus extracted count context:

```text
PathCommandStreamSinkPlanError:
    kind PathCommandStreamSinkPlanErrorKind
    terminal PathCommandStreamPrepareDrainTerminal
    total_count i32
    emitted_count i32
    move_to_count i32
    line_to_count i32
    quadratic_to_count i32
    skip_no_segment_count i32
    last_path_command_index i32
```

F5ay may call:

```text
F5ax prepare summary accessors
private F5ay count guard
private F5ay checked add helper
private F5ay error-from-terminal helper
```

F5ay must not call F5ax drain, F5ax step, F5aw step, F5av lookup, byte-backed lookup helpers, metadata parser, table helpers, old action traversal consumers, storage mutation, `Vec`, path object materialization, rasterization, rendering, platform APIs, host text measurement, or font fallback.

## SFNT simple glyph outline point stream item collection path command stream sink owner boundary

F5az turns the value-only F5ay sink plan into the first allocation-backed owner for the future command sink writer and raster mask writer. This is deliberately still not a writer. The owner only reserves two scalar vectors and preserves the plan/capacity authority that later phases must consume.

The public input is a `PathCommandStreamSinkPlan`. Because the plan is a public value, F5az must treat it as forged until revalidated:

```text
sink_owner_capacity_from_plan plan:
    validate plan fields
    rederive path segment capacity
    rederive prepared count
    rederive raster edge capacity
    rederive scalar capacities
    return capacity
```

The capacity value is copyable:

```text
PathCommandStreamSinkOwnerCapacity:
    path_sink_scalar_capacity i32
    raster_mask_scalar_capacity i32
    path_segment_capacity i32
    raster_edge_capacity i32
```

The owner is not copyable:

```text
PathCommandStreamSinkOwner:
    plan PathCommandStreamSinkPlan
    capacity PathCommandStreamSinkOwnerCapacity
    path_sink_scalars Vec i32
    raster_mask_scalars Vec i32
```

F5az uses scalar formats that are intentionally fixed before the writer exists:

```text
MoveTo path sink command:
    tag
    x
    y

LineTo path sink command:
    tag
    x
    y

QuadraticTo path sink command:
    tag
    control_x
    control_y
    target_x
    target_y

LineTo raster mask edge:
    kind
    x0
    y0
    x1
    y1

QuadraticTo raster mask edge:
    kind
    cx
    cy
    x0
    y0
    x1
    y1
```

Therefore the scalar capacities are:

```text
path_sink_scalar_capacity =
    move_to_count * 3
    + line_to_count * 3
    + quadratic_to_count * 5

raster_mask_scalar_capacity =
    line_to_count * 5
    + quadratic_to_count * 7
```

`SkipNoSegment` contributes to `total_count` and `prepared_count`, but not to either scalar vector. A completed skip-only outline plan is valid and allocates two empty vector owners through `vec::with_capacity 0`. This matters because F5az is an owner boundary, not a rendering branch: it must not collapse a valid completed empty output into a no-op.

Validation failures are not collapsed into `InvalidPlan`. The error kind names the failing invariant:

```text
NegativeTotalCount
NegativeEmittedCount
NegativeMoveToCount
NegativeLineToCount
NegativeQuadraticToCount
NegativeSkipNoSegmentCount
NegativePathSegmentCapacity
NegativeRasterEdgeCapacity
NegativeDrawCount
LastPathCommandIndexInvalid
NoCommandsPrepared
PathSegmentCapacityMismatch
RasterEdgeCapacityMismatch
PreparedCountMismatch
EmittedCountMismatch
DrawCountMismatch
CountOverflow
PathSinkScalarStorageAllocFailed
RasterMaskScalarStorageAllocFailed
```

The allocation error payload always has a precise shape:

```text
PathCommandStreamSinkOwnerAllocError:
    kind PathCommandStreamSinkOwnerAllocErrorKind
    plan PathCommandStreamSinkPlan
    capacity Option PathCommandStreamSinkOwnerCapacity
    storage_error Option StdErrorKind
```

`capacity` is `None` until derivation succeeds. It is `Some capacity` for storage allocation failures. `storage_error` is `Some lower_std_error` only for allocation failures.

The allocation order is fixed:

```text
derive capacity
allocate path_sink_scalars
allocate raster_mask_scalars
return owner
```

If the first allocation fails, no owner has been created and `vec::free` is not called. If the second allocation fails, the lower raster allocation error is preserved, then the path sink vector is freed exactly once, then `RasterMaskScalarStorageAllocFailed` is returned.

F5az may call:

```text
F5ay sink plan accessors
private checked add helper
private checked multiply helper
vec::with_capacity
vec::len
vec::cap
vec::free
```

F5az must not call F5ax drain/step, F5aw step, F5av lookup, byte-backed lookup helpers, metadata parser, table helpers, old action traversal consumers, `vec::push`, path object materialization, rasterization, rendering, platform APIs, host text measurement, or font fallback.

## SFNT simple glyph outline point stream item collection path command stream sink writer boundary

F5ba consumes the F5az sink owner and writes the first real path sink scalar stream. It still does not build path objects, raster masks, glyph masks, render2d commands, screenshots, or platform surfaces.

The writer has two phases:

```text
SinkOwner -> WriterOwner
WriterOwner + PathCommandValue -> WriterStep
```

The input `PathCommandValue` is not trusted. It is public and can be forged, so the writer must revalidate:

```text
stored_tag == source_tag
command-derived tag == stored_tag
command-derived tag == source_tag
```

The command-derived tag is obtained only from the `GuiSfntSimpleGlyphPathCommand` enum payload. It must not be obtained by calling F5av lookup, F5aw stream step, byte-backed lookup, or old path sink traversal again.

The writer start boundary checks F5az owner shape in this exact order:

```text
read owner plan
derive capacity from plan
compare stored capacity and derived capacity
compare path sink Vec cap and path_sink_scalar_capacity
compare raster mask Vec cap and raster_mask_scalar_capacity
require path sink Vec len == 0
require raster mask Vec len == 0
initialize writer progress
```

`PathSinkScalarLenNotZero` and `RasterMaskScalarLenNotZero` are explicit start errors. They prevent an already-mutated owner from silently becoming a fresh writer.

The writer progress is authoritative for normal continuation:

```text
written_count
path_sink_scalar_count
move_to_count
line_to_count
quadratic_to_count
skip_no_segment_count
last_path_command_index
```

Before every push, F5ba revalidates the owner and progress:

```text
capacity_from_plan succeeds
stored capacity == derived capacity
path sink Vec cap == path_sink_scalar_capacity
raster mask Vec cap == raster_mask_scalar_capacity
path sink Vec len == path_sink_scalar_count
raster mask Vec len == 0
written_count is in 0..total_count
each command kind progress is non-negative and within the corresponding plan count
move + line + quadratic + skip == written_count
path_command_index == written_count
path_command_index < total_count
tag invariants hold
variant count has remaining room
path sink scalar capacity has remaining room
```

F5ba checks each command kind progress before computing the aggregate progress total. `MoveToProgressInvalid`, `LineToProgressInvalid`, `QuadraticToProgressInvalid`, and `SkipNoSegmentProgressInvalid` prevent forged writer owners from hiding negative or over-limit counts behind a matching aggregate total.

The `path_sink_scalars_len == path_sink_scalar_count` check is the fail-closed guard for partial append failure. F5ba does not roll back a partially appended Vec when a later scalar in a multi-scalar command fails. Instead, the error returns a writer owner whose Vec may be longer than progress. Such an owner is cleanup / diagnostic only; feeding it back into push is rejected by the len/count check.

The scalar format is fixed and shares F5au stable tag values:

```text
MoveTo:
    1
    x2
    y2

LineTo:
    2
    x2
    y2

QuadraticTo:
    3
    control_x2
    control_y2
    end_x2
    end_y2

SkipNoSegment:
    no scalar
```

`SkipNoSegment` is not a hidden no-op. It returns `SkippedNoSegment WriterOwner` and advances `written_count`, `skip_no_segment_count`, and `last_path_command_index`. It never calls `vec::push`.

For `vec::push` failure, the recovery order is fixed:

```text
read vec_push_error_kind by borrow
recover Vec owner
reconstruct F5az SinkOwner
reconstruct WriterOwner with unchanged progress
return WriterPushError
```

The lower `StdErrorKind`, rejected scalar, rejected `PathCommandValue`, and recovered writer owner are all preserved. Coarse `InvalidValue`, hidden fallback, or silent discard are forbidden.

F5ba may call:

```text
F5az owner accessors
F5az capacity_from_plan
F5az capacity accessors
F5au tag scalar helper
PathCommandValue accessors
PathCommand payload accessors
vec::push
vec push error accessors
```

F5ba must not call F5av path command value lookup, F5aw stream step or drain, byte-backed lookup helpers, old path sink traversal, raster mask writer, path object materialization, rasterization, rendering, platform APIs, host text measurement, or font fallback.

## SFNT simple glyph outline point stream item collection raster mask writer boundary

F5bb consumes the completed F5ba writer owner and writes the raster mask scalar stream reserved by F5az. This is still not a rasterizer. It serializes line and quadratic edge inputs in a stable scalar format for a later bounded raster mask builder.

The extra authority is current point state. Because current point cannot be proven from plan/count/capacity alone, F5bb keeps its writer owner transition-only:

```text
module-private struct
no public constructor
no Clone
no Copy
created only by start / private advance / private push failure recovery
```

The owner keeps separate progress counts:

```text
inner completed F5ba writer owner
written_count
raster_mask_scalar_count
move_to_count
line_to_count
quadratic_to_count
skip_no_segment_count
last_path_command_index
has_current_point
current_x2
current_y2
```

`MoveTo` and `SkipNoSegment` are both zero-scalar transitions, but they must not share one no-mask counter. `MoveTo` updates current point and is bounded by plan `move_to_count`; `SkipNoSegment` preserves current point and is bounded by plan `skip_no_segment_count`.

Start revalidates the completed inner writer:

```text
F5az plan/capacity revalidation
stored capacity equality
path sink Vec cap/len equals path sink capacity
raster mask Vec cap equals raster mask capacity
raster mask Vec len is zero
inner F5ba written_count equals plan.total_count
inner F5ba path_sink_scalar_count equals path sink capacity
inner F5ba kind counts equal plan kind counts
inner F5ba last index equals plan.last_path_command_index
```

Push validation keeps partial append failure fail-closed before checking the next command:

```text
F5az plan/capacity revalidation
path sink complete state remains complete
raster mask Vec len equals raster_mask_scalar_count
inner F5ba completed progress still matches plan
F5bb kind progress is nonnegative and within plan counts
aggregate F5bb progress equals written_count
PathCommandValue index equals written_count
stored/source/command tags match
variant-specific room and current-point checks
```

The raster scalar stream uses the F5au stable tag helper:

```text
LineTo:
    gui_sfnt_simple_glyph_path_command_tag_scalar_value LineTo
    start_x2
    start_y2
    end_x2
    end_y2

QuadraticTo:
    gui_sfnt_simple_glyph_path_command_tag_scalar_value QuadraticTo
    start_x2
    start_y2
    control_x2
    control_y2
    end_x2
    end_y2
```

`LineTo` and `QuadraticTo` require an existing current point. Missing current point is a typed `CurrentPointMissing` error. F5bb must not synthesize `(0, 0)`, skip the segment, or fall back to platform drawing.

Push failure recovery mirrors F5ba. The lower `vec::push` error kind is read before the failed Vec owner is recovered. F5az owner, F5ba writer owner, and F5bb writer owner are reconstructed with unchanged F5bb progress and current point. Partial append is not rolled back; the next push fails `raster_mask_scalars_len == raster_mask_scalar_count`.

F5bb may call:

```text
F5az owner accessors
F5az capacity_from_plan
F5ba writer owner accessors
PathCommandValue accessors
PathCommand payload accessors
F5au tag scalar helper
vec::push
vec push error accessors
```

F5bb must not call F5av path command value lookup, F5aw stream step or drain, byte-backed lookup helpers, old path sink traversal, path object materialization, rasterization, rendering, platform APIs, host text measurement, or font fallback.

## SFNT simple glyph outline point stream item collection raster edge owner boundary

F5bc converts the completed F5bb raster mask scalar stream into typed edge values. It is still an internal owner boundary. It does not scan-convert curves, generate coverage, emit render2d commands, call platform APIs, or ask any font fallback mechanism for substitute geometry.

Value edge types are scalar-only:

```text
GuiSfntSimpleGlyphRasterLineEdge:
    start_x2
    start_y2
    end_x2
    end_y2

GuiSfntSimpleGlyphRasterQuadraticEdge:
    start_x2
    start_y2
    control_x2
    control_y2
    end_x2
    end_y2

GuiSfntSimpleGlyphRasterEdge:
    Line GuiSfntSimpleGlyphRasterLineEdge
    Quadratic GuiSfntSimpleGlyphRasterQuadraticEdge
```

These value types may implement `Clone` / `Copy`. The storage owners must not:

```text
module-private drain owner
    F5bb raster mask writer owner
    Vec GuiSfntSimpleGlyphRasterEdge
    scalar_index
    edge_count
    line_edge_count
    quadratic_edge_count

module-private completed owner
    F5bb raster mask writer owner
    Vec GuiSfntSimpleGlyphRasterEdge
    final scalar and edge counts
```

Start performs a full authority check before allocating the typed edge Vec. It reuses F5az capacity derivation rather than trusting stored capacity alone, then verifies F5ba inner completion and F5bb outer completion. Expected edge count is `line_to_count + quadratic_to_count`; negative count or overflow is rejected before allocation. The Vec capacity is exactly `capacity.raster_edge_capacity`.

Start errors are owner-bearing. Every start failure returns the original F5bb writer owner, so the caller can recover or free it. Allocation failure keeps lower `StdErrorKind`; no default empty typed edge Vec is fabricated.

Scalar read is private and non-consuming. It reads the nested raster mask Vec by shared reference and returns typed scalar read errors:

```text
ScalarIndexNegative
ScalarIndexOutOfRange
ScalarStorageLengthMismatch
ScalarStorageCapacityMismatch
ScalarSlotMissing
```

`ScalarSlotMissing` is specifically `vec::get` returning `Option::None` after the explicit length/capacity checks. It is not converted to zero and does not retry through another representation.

Drain parses records with a budget:

```text
complete:
    return RasterEdgesCompleted completed_owner

not complete and budget exhausted:
    return StepBudgetExhausted drain_owner

tag 2:
    read exactly 5 scalar values
    push Line edge
    require scalar_index + 5 and line_edge_count + 1

tag 3:
    read exactly 7 scalar values
    push Quadratic edge
    require scalar_index + 7 and quadratic_edge_count + 1

tag 1 or tag 4:
    UnexpectedNonRasterTag

other tag:
    MalformedRasterMaskTag
```

The progress guard runs after a successful push and before recursion. It prevents a buggy step helper from returning an owner with a stale scalar index, double-incremented edge count, or mismatched line/quadratic count.

Push failure recovery owns exactly one drain owner. The lower `vec::push` error kind is read before the failed Vec owner is recovered, then the unchanged drain owner is reconstructed with the recovered Vec and returned in `RasterEdgeDrainError`. Completed owner and drain owner each have explicit free functions that close the typed edge Vec and then close the F5bb writer owner exactly once.

## SFNT simple glyph raster coverage mask writer owner boundary

F5bd allocates and owns the coverage cell buffer that a later scan conversion phase will fill. It consumes a completed F5bc raster edge owner and a validated-by-construction candidate config, then either returns a module-private writer owner or an owner-bearing start error. It does not inspect old byte-backed path helpers, scan-convert edges, compute coverage values, emit render2d commands, call host APIs, or request font fallback.

The config and shape are value records:

```text
GuiSfntSimpleGlyphRasterCoverageConfig:
    origin_x2
    origin_y2
    width_px
    height_px
    sample_scale
    max_cell_count

GuiSfntSimpleGlyphRasterCoverageShape:
    origin_x2
    origin_y2
    width_px
    height_px
    sample_scale
    coverage_max
    cell_count
```

`coverage_max = sample_scale * sample_scale`. This keeps subpixel sampling explicit while leaving packed storage, gamma handling, and anti-aliased color conversion to later renderer phases. The first storage representation is `Vec i32` with `cap == cell_count`. The vector length is the number of cells written by later scan conversion.

Shape derivation is fail-closed:

```text
if width_px <= 0:
    InvalidWidth
if height_px <= 0:
    InvalidHeight
if sample_scale <= 0:
    InvalidSampleScale
if max_cell_count <= 0:
    InvalidMaxCellCount
if sample_scale * sample_scale overflows:
    CoverageMaxOverflow
if width_px * height_px overflows:
    CellCountOverflow
if cell_count > max_cell_count:
    CellCountLimitExceeded
```

The edge owner is revalidated before allocation. F5bd reads the nested plan/capacity through the F5bc completed owner and verifies:

```text
edge_count == capacity.raster_edge_capacity
line_edge_count == plan.line_to_count
quadratic_edge_count == plan.quadratic_to_count
line_edge_count + quadratic_edge_count == edge_count
typed edge Vec len == edge_count
typed edge Vec cap == capacity.raster_edge_capacity
```

These checks duplicate F5bc completion checks intentionally. F5bd is a new trust boundary because it allocates a new coverage buffer and must not assume forged private state became valid merely because the type name matches.

Edge Vec storage mismatch uses distinct typed errors:

```text
EdgeStorageLenMismatch
EdgeStorageCapacityMismatch
```

Coverage cell allocation is attempted only after both count invariants and Vec storage invariants pass.

The writer owner is module-private:

```text
GuiSfntSimpleGlyphRasterCoverageMaskWriterOwner:
    edge_owner GuiSfntSimpleGlyphOutlinePointStreamItemCollectionRasterEdgeOwner
    shape GuiSfntSimpleGlyphRasterCoverageShape
    cells Vec i32
    written_cell_count i32
```

It has no public constructor and no `Clone` / `Copy` implementation. Start errors retain exactly one completed edge owner. Allocation failure keeps the lower `StdErrorKind` separately from validation errors. Free closes the coverage cell Vec first, then closes the F5bc edge owner exactly once.

Push is a writer mutation boundary, not scan conversion:

```text
validate cells.len == written_cell_count
validate cells.cap == shape.cell_count
validate written_cell_count < shape.cell_count
validate coverage_value >= 0
validate coverage_value <= shape.coverage_max
vec::push coverage_value
advance written_cell_count by 1
```

On `vec::push` failure, the lower error kind is read before recovering the failed Vec owner. The returned error owns the unchanged writer owner and the rejected coverage value. Completion requires exact fill:

```text
if written_cell_count == shape.cell_count:
    CoverageMaskCompleted completed_owner
else:
    CoverageMaskIncomplete writer_owner
```

There is no zero-fill fallback and no best-effort partial mask completion.

## SFNT simple glyph raster coverage scan converter boundary

F5be consumes the F5bd coverage mask writer owner and converts typed raster edges into coverage cells. It is the first boundary that computes coverage values, but it still stops before packed mask storage, render2d command emission, platform presentation, host font APIs, and font fallback.

The scan owner is module-private and resumable:

```text
GuiSfntSimpleGlyphRasterCoverageScanConfig:
    quadratic_segment_count i32

GuiSfntSimpleGlyphRasterCoverageScanOwner:
    writer GuiSfntSimpleGlyphRasterCoverageMaskWriterOwner
    config GuiSfntSimpleGlyphRasterCoverageScanConfig
    cell_index i32
```

`GuiSfntSimpleGlyphRasterCoverageScanConfig` is value-only and may implement `Clone` / `Copy`. `GuiSfntSimpleGlyphRasterCoverageScanOwner` must not implement `Clone` / `Copy` because it owns the F5bd writer and therefore transitively owns the coverage cell Vec and F5bc edge owner.

Start validation is fail-closed:

```text
quadratic_segment_count > 0
shape.width_px > 0
shape.height_px > 0
shape.sample_scale > 0
shape.coverage_max == shape.sample_scale * shape.sample_scale
shape.cell_count == shape.width_px * shape.height_px
writer.written_cell_count == 0
writer.cells.len == 0
writer.cells.cap == shape.cell_count
edge_count == capacity.raster_edge_capacity
line_edge_count == plan.line_to_count
quadratic_edge_count == plan.quadratic_to_count
typed edge Vec len == edge_count
typed edge Vec cap == capacity.raster_edge_capacity
```

This deliberately repeats the F5bd writer start checks and the F5bd shape derivation invariants. F5be is a new mutation boundary and must not trust stale private state when it is about to compute all remaining coverage cells. Shape validation must happen before any cell coordinate math, sample loop, edge scan, or `push_cell` call.

Start error kinds must distinguish at least:

```text
InvalidQuadraticSegmentCount
ShapeInvalidWidth
ShapeInvalidHeight
ShapeInvalidSampleScale
ShapeCoverageMaxMismatch
ShapeCellCountMismatch
WriterAlreadyStarted
CellStorageLenMismatch
CellStorageCapacityMismatch
EdgeCountMismatch
LineEdgeCountMismatch
QuadraticEdgeCountMismatch
EdgeCountSumMismatch
EdgeStorageLenMismatch
EdgeStorageCapacityMismatch
```

Overflow while rechecking `coverage_max` or `cell_count` is represented by the corresponding mismatch error because the stored shape is already outside the validated F5bd contract. The implementation may use checked helpers internally, but it must not continue with wrapped values.

Coverage sampling uses integer coordinates. The coverage shape is expressed in doubled font coordinates (`x2` / `y2`) and each pixel cell spans two doubled-coordinate units before scaling by `sample_scale`. For cell `cell_index`:

```text
cell_x = cell_index % shape.width_px
cell_y = cell_index / shape.width_px
sample_x = (shape.origin_x2 + cell_x * 2) * shape.sample_scale + (sample_x_index * 2 + 1)
sample_y = (shape.origin_y2 + cell_y * 2) * shape.sample_scale + (sample_y_index * 2 + 1)
edge_x = edge_x2 * shape.sample_scale
edge_y = edge_y2 * shape.sample_scale
```

`sample_x` / `sample_y` and edge coordinates are compared as i64. Line crossing uses the even-odd rule without division:

```text
active = (y0 > sample_y) != (y1 > sample_y)
left = (sample_x - x0) * (y1 - y0)
right = (x1 - x0) * (sample_y - y0)

if active and y1 > y0 and left < right:
    crossing
if active and y1 < y0 and left > right:
    crossing
```

Horizontal edges are not active because `y0 > sample_y` and `y1 > sample_y` are equal. This prevents double counting at shared vertices without special-case fallback.

Quadratic edges use explicit deterministic flattening controlled by `quadratic_segment_count`. For segment ordinal `i`, endpoints are computed at `t=i/n` and `t=(i+1)/n` using the integer quadratic Bezier formula in the same scaled coordinate space:

```text
B(t) = (1 - t)^2 * p0 + 2 * (1 - t) * t * p1 + t^2 * p2
```

The formula is evaluated as integer numerator over `n*n` and then divided with signed division. This is a current implementation choice, not a hidden fallback. A later analytical quadratic crossing boundary may replace it while preserving the same scan owner / coverage writer contract.

One scan step computes one cell coverage:

```text
coverage = 0
for sample_y_index in 0..sample_scale:
    for sample_x_index in 0..sample_scale:
        crossing_count = edge crossing count for sample point
        if crossing_count % 2 == 1:
            coverage += 1
push_cell writer coverage
cell_index += 1
```

The step function returns an owner-bearing error. If `push_cell` fails, the lower F5bd push error is kept separately and the scan owner is rebuilt with the recovered writer and the original cell index.

The bounded drain checks completion before the step budget:

```text
if cell_index < 0:
    CellIndexNegative
else if cell_index > shape.cell_count:
    CellIndexExceedsCellCount
else if cell_index == shape.cell_count:
    call F5bd completion
else if remaining_steps <= 0:
    StepBudgetExhausted owner
else:
    step one cell and recurse
```

After a successful step, a hard progress guard verifies:

```text
next.cell_index == old.cell_index + 1
next.writer.written_cell_count == old.writer.written_cell_count + 1
```

`StepBudgetExhausted` is a typed terminal, not a fallback. The caller may schedule another slice with the returned owner. It must not be converted into an empty mask or a successful render.

`CellIndexNegative` and `CellIndexExceedsCellCount` are owner-bearing errors. They are checked before completion and budget handling, so forged state cannot reach `%`, `/`, sample coordinate derivation, or `push_cell`.

F5be must not call byte-backed lookup helpers, old traversal helpers, packed mask conversion, render2d, platform APIs, host APIs, or font fallback. It must not zero-fill missing cells.

## SFNT simple glyph raster packed mask owner boundary

F5bf consumes the completed F5be coverage mask owner and produces a normalized alpha-cell packed mask owner. It is deliberately still an alloc/gui internal owner boundary. It does not emit render2d commands, create a pixel buffer, call a DrawTarget / RenderTarget, talk to Web/native/platform APIs, use host text measurement, or request font fallback.

The config is value-only:

```text
GuiSfntSimpleGlyphRasterPackedMaskConfig:
    alpha_max i32
```

`GuiSfntSimpleGlyphRasterPackedMaskConfig` may implement `Clone` / `Copy`. Transition and completed owners must not implement `Clone` / `Copy`.

```text
GuiSfntSimpleGlyphRasterPackedMaskPackOwner:
    coverage_owner GuiSfntSimpleGlyphRasterCoverageMaskOwner
    alpha_cells Vec i32
    config GuiSfntSimpleGlyphRasterPackedMaskConfig
    cell_index i32

GuiSfntSimpleGlyphRasterPackedMaskOwner:
    edge_owner GuiSfntSimpleGlyphOutlinePointStreamItemCollectionRasterEdgeOwner
    shape GuiSfntSimpleGlyphRasterCoverageShape
    alpha_cells Vec i32
    cell_count i32
    alpha_max i32
```

Start validation is fail-closed:

```text
alpha_max > 0
shape.width_px > 0
shape.height_px > 0
shape.sample_scale > 0
shape.coverage_max == shape.sample_scale * shape.sample_scale
shape.cell_count == shape.width_px * shape.height_px
shape.coverage_max * alpha_max does not overflow i32
coverage_owner.cell_count == shape.cell_count
coverage_owner.cells.len == shape.cell_count
coverage_owner.cells.cap == shape.cell_count
allocate alpha_cells with capacity shape.cell_count
```

Start error owns the original completed coverage owner and config. Allocation failure keeps the lower `StdErrorKind` separately from validation errors.

F5bf has an owner invariant that is checked before budget handling, raw cell read, alpha normalization, Vec push, and completion:

```text
cell_index >= 0
shape invariant is valid
cell_index <= shape.cell_count
alpha_cells.len == cell_index
alpha_cells.cap == shape.cell_count
coverage_owner.cell_count == shape.cell_count
coverage_owner.cells.len == shape.cell_count
coverage_owner.cells.cap == shape.cell_count
```

This invariant is intentionally repeated on every drain / step / completion entry. If `vec::push` returns a recovered alpha Vec whose length has already changed, the rebuilt owner is only a cleanup / diagnostic carrier until a later invariant check proves that `alpha_cells.len == cell_index`. It must not silently continue or complete from inconsistent state.

Raw cell read uses the coverage owner as the only authority:

```text
vec::get coverage_owner.cells cell_index
None -> RawCellSlotMissing
coverage < 0 -> RawCoverageNegative
coverage > shape.coverage_max -> RawCoverageExceedsMax
```

Alpha normalization is integer-only:

```text
if coverage > max_i32 / alpha_max:
    AlphaScaleOverflow
else:
    alpha = coverage * alpha_max / shape.coverage_max
```

Because `shape.coverage_max` is revalidated before use, division by zero is not a valid state. The formula is the current implementation contract for scalar alpha cells; later gamma-aware or byte-packed encodings must be a separate boundary and must preserve this owner recovery contract.

One step reads one raw coverage cell, normalizes it, pushes one alpha cell, and advances `cell_index` by exactly 1. Push failure reads the lower storage error kind before recovering the alpha Vec and rebuilding the pack owner with the unchanged `cell_index`.

Completion is exact:

```text
cell_index == shape.cell_count
alpha_cells.len == shape.cell_count
alpha_cells.cap == shape.cell_count
```

On completion, the raw coverage owner is destructured. F5bf frees the raw coverage cell Vec, moves the edge owner and shape into `GuiSfntSimpleGlyphRasterPackedMaskOwner`, and keeps only the alpha cell Vec in the completed owner. This prevents the final mask from retaining both raw coverage and normalized alpha buffers while still closing the edge owner exactly once.

This completion-time raw cell release is part of the F5bf contract, not an implementation detail that a backend may skip.

`StepBudgetExhausted` is a typed terminal with the pack owner. It is not a partial completed mask, not a fallback, and not a zero-fill path.

F5bf must not call byte-backed lookup helpers, old traversal helpers, zero-fill helpers, render2d, DrawTarget / RenderTarget, platform APIs, host APIs, font fallback, or any function that emits pixels or commands.

## SFNT simple glyph render fill alpha mask boundary

F5bg consumes the completed F5bf packed mask owner and produces a fill-alpha-mask owner for the later 2D renderer boundary. It is still an alloc/gui owner handoff. It does not emit `RenderCommand`, create a pixel buffer, call `DrawTarget` / `RenderTarget`, call platform or host APIs, or bind stroke / shadow paint.

F5bg deliberately does not accept `GuiGlyphPaint`. The F5bf alpha cells represent fill coverage. If this boundary accepted a full glyph paint, stroke-only, stroke-plus-fill, or shadow-bearing input could be silently reduced to fill coverage. That would hide a semantic loss. Stroke, shadow, and full glyph paint binding must be handled by a later explicit boundary.

The config is value-only:

```text
GuiSfntSimpleGlyphRenderFillAlphaMaskConfig:
    origin GuiPoint
    fill_paint GuiPaint
    blend GuiBlendMode
```

`GuiSfntSimpleGlyphRenderFillAlphaMaskConfig` may implement `Clone` / `Copy`. The completed owner and owner-bearing start error must not implement `Clone` / `Copy`.

```text
GuiSfntSimpleGlyphRenderFillAlphaMaskOwner:
    edge_owner GuiSfntSimpleGlyphOutlinePointStreamItemCollectionRasterEdgeOwner
    shape GuiSfntSimpleGlyphRasterCoverageShape
    alpha_cells Vec i32
    cell_count i32
    alpha_max i32
    origin GuiPoint
    fill_paint GuiPaint
    blend GuiBlendMode
```

The start boundary revalidates the completed packed owner before destructuring it:

```text
shape.width_px > 0
shape.height_px > 0
shape.sample_scale > 0
shape.coverage_max == shape.sample_scale * shape.sample_scale
shape.cell_count == shape.width_px * shape.height_px
packed_owner.alpha_max > 0
packed_owner.cell_count == shape.cell_count
packed_owner.alpha_cells.len == shape.cell_count
packed_owner.alpha_cells.cap == shape.cell_count
```

Failure is owner-bearing:

```text
GuiSfntSimpleGlyphRenderFillAlphaMaskStartError:
    kind GuiSfntSimpleGlyphRenderFillAlphaMaskStartErrorKind
    packed_owner GuiSfntSimpleGlyphRasterPackedMaskOwner
    config GuiSfntSimpleGlyphRenderFillAlphaMaskConfig
```

The consuming recovery accessor returns the original packed owner. The config is Copy and can be read by reference before recovering the owner. A start-error free helper closes the packed owner when the caller does not continue.

On success, F5bg moves the existing edge owner and alpha Vec into the render fill alpha mask owner. It also copies `origin`, `fill_paint`, and `blend` from the config into the completed owner without reducing or reconstructing them. The operation is a zero-copy owner handoff for the alpha storage.

The completed owner free helper releases `alpha_cells` before closing `edge_owner`, exactly once. It does not touch platform presentation or command output.

F5bg must not call byte-backed lookup helpers, old traversal helpers, zero-fill helpers, `RenderCommand` constructors, `DrawTarget` / `RenderTarget`, platform APIs, host APIs, font fallback, or any stroke / shadow binding helper.

## SFNT simple glyph render glyph paint binding boundary

F5bh is the explicit full `GuiGlyphPaint` binding boundary for the F5bg fill alpha mask owner. It accepts a full glyph paint value but only forwards the fill-only subset that the current alpha mask can represent. This is not a compositor boundary and not a render command boundary.

The config is value-only:

```text
GuiSfntSimpleGlyphRenderGlyphPaintConfig:
    origin GuiPoint
    paint GuiGlyphPaint
```

`GuiSfntSimpleGlyphRenderGlyphPaintConfig` may implement `Clone` / `Copy`. The start error owns a packed mask owner and must not implement `Clone` / `Copy`.

The start signature is:

```text
GuiSfntSimpleGlyphRasterPackedMaskOwner
GuiSfntSimpleGlyphRenderGlyphPaintConfig
-> Result GuiSfntSimpleGlyphRenderFillAlphaMaskOwner GuiSfntSimpleGlyphRenderGlyphPaintStartError
```

The validation order is part of the contract:

```text
stroke Some        -> UnsupportedStrokePaint
SingleShadow       -> UnsupportedShadowPaint
ShadowRun          -> UnsupportedShadowPaint
fill None          -> MissingFillPaint
fill Some + no unsupported paint -> delegate to F5bg
```

Stroke and shadow are checked before fill. This prevents stroke-only or shadow-only paint from being reported as a missing fill and makes unsupported drawing modes explicit. If stroke and shadow are both present, stroke wins because it is the first unsupported mode in the stable validation order.

The accepted path constructs the existing F5bg config from:

```text
origin from GuiSfntSimpleGlyphRenderGlyphPaintConfig
fill_paint from GuiGlyphPaint.fill
blend from GuiGlyphPaint.blend
```

It then calls `gui_sfnt_simple_glyph_render_fill_alpha_mask_owner_start`. The returned success owner is exactly `GuiSfntSimpleGlyphRenderFillAlphaMaskOwner`; F5bh does not introduce a second completed owner and does not copy the alpha storage.

Failure is owner-bearing:

```text
GuiSfntSimpleGlyphRenderGlyphPaintStartError:
    kind GuiSfntSimpleGlyphRenderGlyphPaintStartErrorKind
    packed_owner GuiSfntSimpleGlyphRasterPackedMaskOwner
    config GuiSfntSimpleGlyphRenderGlyphPaintConfig
    lower_kind Option GuiSfntSimpleGlyphRenderFillAlphaMaskStartErrorKind
```

For direct validation failures, `lower_kind` is `Option::None`. When F5bg start fails, F5bh reads `gui_sfnt_simple_glyph_render_fill_alpha_mask_start_error_kind &lower_error` before consuming `lower_error` through packed owner recovery. It then returns `FillAlphaMaskStartFailed` with `lower_kind = Option::Some lower_kind` and the recovered packed owner.

F5bh must not call `RenderCommand` constructors, DrawTarget / RenderTarget, platform APIs, host APIs, font fallback, stroke rasterizers, shadow rasterizers, or 2D compositor APIs. Unsupported stroke / shadow must not become a hidden fill-only success.

## SFNT simple glyph render stroke request boundary

F5kq introduces the first dedicated glyph stroke rasterization boundary. The authority is the completed path command stream writer owner produced by the collection-backed path command stream, not the raster edge owner and not the fill alpha mask owner. The completed path command stream writer owner still carries the ordered `MoveTo` / `LineTo` / `QuadraticTo` path authority and contour-derived progress counts. A raster edge owner is already shaped for fill coverage, and a fill alpha mask owner has already bound fill paint and alpha storage. F5kq therefore does not consume the fill alpha mask owner.

The request config accepts full paint:

```text
GuiSfntSimpleGlyphRenderStrokeRequestConfig:
    origin GuiPoint
    paint GuiGlyphPaint
```

The success owner is a request owner, not a raster owner:

```text
GuiSfntSimpleGlyphRenderStrokeRequestOwner:
    writer GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkWriterOwner
    origin GuiPoint
    fill Option GuiPaint
    stroke GuiStroke
    blend GuiBlendMode
```

The optional fill value is preserved only as metadata for a later composition-order boundary. F5kq does not turn fill+stroke into a fill render success and does not allocate or consume a fill alpha mask.

Validation order is part of the contract:

```text
stroke None -> MissingStrokePaint
stroke.width <= 0 -> StrokeWidthInvalid
SingleShadow / ShadowRun -> UnsupportedShadowPaint
non SourceOver blend -> UnsupportedBlendMode
F5az capacity derivation
stored capacity equality
path sink scalar capacity equality
raster mask scalar capacity equality
path sink scalar len equality
raster mask scalar len == 0
writer written_count equals plan.total_count
writer path_sink_scalar_count equals capacity.path_sink_scalar_capacity
writer kind counts equal plan kind counts
writer last index equals plan.last_path_command_index
```

unsupported blend modes are rejected before the request owner is created. The current stroke request does not emit a command, but accepting Copy / Multiply / Screen here would let a later stroke renderer accidentally treat unsupported blend as SourceOver.

Every failure is owner-bearing and returns the original path command writer authority. `GuiSfntSimpleGlyphRenderStrokeRequestStartError` keeps the config plus optional derived capacity context so validation failures can be diagnosed without reconstructing a fake owner. The owner and start error must not implement `Clone` / `Copy`; the config and error kind may implement `Clone` / `Copy`.

F5kq must not call byte-backed lookup helpers, old traversal helpers, fill alpha mask helpers, raster edge helpers, coverage / packed mask helpers, render command constructors, DrawTarget / RenderTarget, render2d surfaces, platform APIs, host text measurement, font fallback, stroke rasterizers, shadow rasterizers, or compositor APIs.

Later stroke phases are deliberately separate: stroke segment expansion plan, stroke edge owner, stroke coverage mask owner, packed stroke mask owner, and then glyph paint composition order. F5kq only proves that a caller has a validated stroke request over the correct geometry authority.

## SFNT simple glyph render stroke segment plan boundary

F5kr consumes the F5kq stroke request owner and creates a count-only stroke segment plan owner. The authority remains the completed path command stream writer owner contained inside the request owner. F5kr does not reinterpret fill alpha mask output as stroke geometry and does not consume the fill alpha mask owner.

When F5kr was introduced, `GuiStroke` stored color and width only, so F5kr could not choose a join, cap, dash, or miter policy. After F5kv, `GuiStroke` carries explicit style policy too, but F5kr still fixes only the input needed for the count plan: the completed path command counts, the drawable source segment count, the origin / fill / stroke / blend metadata, and the validated stroke width. It must not interpret stroke style policy as geometry.

```text
GuiSfntSimpleGlyphRenderStrokeSegmentPlanOwner:
    request_owner GuiSfntSimpleGlyphRenderStrokeRequestOwner
    origin GuiPoint
    fill Option GuiPaint
    stroke GuiStroke
    blend GuiBlendMode
    path_command_count i32
    move_to_count i32
    line_to_count i32
    quadratic_to_count i32
    skip_no_segment_count i32
    draw_segment_count i32
    path_sink_scalar_count i32
    stroke_width i32
```

Validation order is part of the contract:

```text
request writer plan/capacity derivation
stored capacity equality
path sink scalar capacity equality
raster mask scalar capacity equality
path sink scalar len equality
raster mask scalar len == 0
writer written_count equals plan.total_count
writer path_sink_scalar_count equals capacity.path_sink_scalar_capacity
writer kind counts equal plan kind counts
writer last index equals plan.last_path_command_index
stroke.width > 0
checked line_to_count + quadratic_to_count
draw_segment_count equals plan.draw_count
draw_segment_count equals derived raster edge capacity
draw_segment_count > 0
```

`NoDrawableStrokeSegments` does not mean the glyph topology is invalid. A skip-only completed output remains valid for the lower owner boundary because that boundary represents completed path command preparation. F5kr represents a drawable stroke segment plan, so `line_to_count + quadratic_to_count == 0` is rejected before a success owner is created.

Every failure is owner-bearing and returns the original request owner. The success owner and start error must not implement `Clone` / `Copy`.

F5kr must not call path command value lookup, stroke geometry expansion, fill alpha mask helpers, raster edge helpers, coverage / packed mask helpers, render command constructors, DrawTarget / RenderTarget, render2d surfaces, platform APIs, host text measurement, font fallback, stroke rasterizers, shadow rasterizers, or compositor APIs.

## SFNT simple glyph render stroke source segment cursor boundary

F5ks consumes the F5kr stroke segment plan owner and exposes a cursor over the completed path sink scalar stream. It is still before actual stroke offset geometry. It does not choose join, cap, dash, or miter behavior, and it does not allocate stroke edges, coverage masks, packed masks, commands, pixels, or platform resources.

The scalar stream authority is the completed path command stream writer owner inside the F5kq request owner held by F5kr. F5ks reads `path_sink_scalars` only after revalidating the F5kr plan invariants:

```text
request writer plan/capacity derivation
stored capacity equality
path sink scalar capacity equality
raster mask scalar capacity equality
path sink scalar len equality
raster mask scalar len == 0
writer written_count equals plan.total_count
writer path_sink_scalar_count equals capacity.path_sink_scalar_capacity
writer kind counts equal plan kind counts
writer last index equals plan.last_path_command_index
stroke.width > 0
checked line_to_count + quadratic_to_count
draw_segment_count equals plan.draw_count
draw_segment_count equals derived raster edge capacity
draw_segment_count > 0
stored F5kr owner counts equal plan counts
stored F5kr path_sink_scalar_count and stroke_width equal the rederived values
```

The source segment values are copyable:

```text
GuiSfntSimpleGlyphRenderStrokeSourceSegmentLine:
    segment_index i32
    start_x2 i32
    start_y2 i32
    end_x2 i32
    end_y2 i32
    stroke_width i32

GuiSfntSimpleGlyphRenderStrokeSourceSegmentQuadratic:
    segment_index i32
    start_x2 i32
    start_y2 i32
    control_x2 i32
    control_y2 i32
    end_x2 i32
    end_y2 i32
    stroke_width i32
```

The cursor owns the plan owner and tracks scalar progress plus read counts:

```text
GuiSfntSimpleGlyphRenderStrokeSourceSegmentCursor:
    plan_owner GuiSfntSimpleGlyphRenderStrokeSegmentPlanOwner
    scalar_index i32
    emitted_segment_count i32
    move_to_count i32
    line_segment_count i32
    quadratic_segment_count i32
    has_current_point bool
    current_x2 i32
    current_y2 i32
```

Step behavior:

```text
scalar_index == path_sink_scalar_count:
    validate read counts against F5kr plan counts and return Completed
MoveTo record:
    update current point, increment move_to_count, emit StateUpdated
LineTo record:
    require current point, emit LineSegment with current point as start, update current point to end
QuadraticTo record:
    require current point, emit QuadraticSegment with current point as start, update current point to end
unknown tag:
    PathSinkTagUnknown
SkipNoSegment tag:
    UnexpectedSkipNoSegmentTag
truncated record:
    typed record truncation error before reading payload scalars
```

SkipNoSegment participates in the lower path command count and skip count, but it has no scalar record in this stream. F5ks must not use `path_command_count` as a cursor bound and must not reconstruct skip reason from scalar storage.

Every failure is owner-bearing. Start errors return the plan owner, step errors return the cursor, and terminal/free helpers close the held F5kr plan owner exactly once.

## SFNT simple glyph render stroke source segment metric preparation boundary

F5kt consumes only borrowed F5ks source segment values and prepares checked metric values for later stroke offset geometry. It is not the stroke offset expansion boundary. It does not consume the F5ks cursor or terminal owner, does not allocate, and does not choose join, cap, dash, or miter behavior.

Prepared line metric:

```text
GuiSfntSimpleGlyphRenderStrokeSourceSegmentLineMetric:
    segment_index i32
    start_x2 i32
    start_y2 i32
    end_x2 i32
    end_y2 i32
    stroke_width i32
    dx i64
    dy i64
    length_squared i64
```

Prepared quadratic metric:

```text
GuiSfntSimpleGlyphRenderStrokeSourceSegmentQuadraticMetric:
    segment_index i32
    start_x2 i32
    start_y2 i32
    control_x2 i32
    control_y2 i32
    end_x2 i32
    end_y2 i32
    stroke_width i32
    start_control_dx i64
    start_control_dy i64
    control_end_dx i64
    control_end_dy i64
    start_control_length_squared i64
    control_end_length_squared i64
```

`GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetric` is a copyable enum with `Line` and `Quadratic` variants. All F5kt success values are value-only and implement `Clone` / `Copy`.

Validation order is part of the contract:

```text
segment_index >= 0
stroke_width > 0
cast source coordinates to i64
compute deltas with i64 subtraction
check each delta is within the safe square operand range
compute delta squares
check square sum against i64 max before addition
reject zero-length line
reject fully degenerate quadratic
```

The safe square operand range is derived inside F5kt from small literals as `3037000499`. This avoids relying on unchecked `i64` multiplication for values whose square cannot fit in signed i64. `i64::MAX` is also constructed from small literals and used before adding two nonnegative squared components.

Quadratic partial degeneracy remains explicit data. If `start_control_length_squared == 0` but `control_end_length_squared > 0`, or the reverse, F5kt returns a success metric with the zero tangent visible. The later offset geometry boundary must decide how to use that evidence; F5kt must not silently normalize or flatten it.

F5kt must not call F5ks cursor step, path sink scalar storage, stroke edge owner, coverage / packed mask helpers, render command constructors, DrawTarget / RenderTarget, render2d surfaces, platform APIs, host text measurement, font fallback, stroke rasterizers, shadow rasterizers, or compositor APIs.

## SFNT simple glyph render stroke source segment metric owner boundary

F5ku consumes a F5ks fresh cursor and builds a whole-sequence owner for prepared source segment metrics. It exists because the later stroke offset geometry boundary needs sequence-level ownership before it can reason about adjacency, joins, caps, and degenerate curve evidence without re-reading the path sink scalar stream.

The start function accepts `GuiSfntSimpleGlyphRenderStrokeSourceSegmentCursor`, not the lower F5kr plan owner. It re-runs F5ks cursor invariants and then requires the canonical fresh state:

```text
scalar_index == 0
emitted_segment_count == 0
move_to_count == 0
line_segment_count == 0
quadratic_segment_count == 0
has_current_point == false
current_x2 == 0
current_y2 == 0
```

Only after the fresh check does F5ku allocate `Vec GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetric` with capacity equal to `draw_segment_count`. The storage must start at len 0 and cap `draw_segment_count`. A malformed storage shape is rejected after freeing the just-created Vec; the cursor remains the recovery owner.

The drain owner stores:

```text
GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetricDrainOwner:
    cursor
    metrics
    metric_count
    line_metric_count
    quadratic_metric_count
```

The completed owner stores the F5kr plan owner returned by F5ks `Completed`:

```text
GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetricOwner:
    plan_owner
    metrics
    metric_count
    line_metric_count
    quadratic_metric_count
```

Drain step first validates F5ks cursor invariants and local metric owner invariants. Local invariants require nonnegative counts, `line_metric_count + quadratic_metric_count == metric_count`, count upper bounds against the plan, `vec::len metrics == metric_count`, and `vec::cap metrics == draw_segment_count`.

At `scalar_index == path_sink_scalar_count`, F5ku checks completion invariants before calling F5ks step. Completion requires exact draw, line, quadratic, len, and cap counts. F5ks `Completed` then transfers the plan owner into the F5ku completed owner.

For normal non-completion steps, F5ku delegates to F5ks cursor step:

```text
StateUpdated:
    carry updated cursor and unchanged metric Vec/count
LineSegment:
    prepare F5kt line metric and push it
QuadraticSegment:
    prepare F5kt quadratic metric and push it
```

push failure must return the advanced cursor, returned metric Vec, rejected metric value, and pre push counts. Metric prepare failure similarly returns the advanced cursor, metric Vec, pre push counts, rejected segment kind, segment index, and F5kt metric error kind. These failure states are owner-bearing recovery payloads, but they are not normal resumable drain owners because the F5ks cursor has already advanced past the rejected source segment.

F5ku must not call `path_sink_scalars` directly, SFNT byte lookup/parser helpers, fill alpha mask owners, raster edge owners, coverage / packed mask helpers, render command constructors, DrawTarget / RenderTarget, render2d surfaces, platform APIs, host text measurement, font fallback, stroke/shadow rasterizers, or compositor APIs. It may allocate one metric Vec and push through exactly one helper.

## Core GUI stroke style contract boundary

F5kv is a root API contract phase for glyph stroke geometry. The next offset geometry boundary cannot invent stroke join, cap, miter, or dash defaults, because the 2D renderer and text renderer must share the same stroke policy. F5kv therefore moves the policy into the core no_alloc `GuiStroke` value before any actual stroke offset point, edge, coverage, or compositor work.

The completed `GuiStroke` value stores:

```text
color Rgba8888
width i32
cap GuiStrokeCap
join GuiStrokeJoin
miter_limit f32
dash GuiStrokeDash
```

`GuiStrokeCap` has `Butt`, `Square`, and `Round`. `GuiStrokeJoin` has `Miter`, `Bevel`, and `Round`. `GuiStrokeDash::Solid` means explicit no-dash; it is not a fallback for unsupported dash patterns. Future dash patterns must become typed values or typed unsupported errors instead of being approximated as solid.

`miter_limit` remains `f32` to match the shared 2D stroke design. It is not an undocumented raw integer scale. The checked constructor accepts only `width > 0` and `miter_limit > 0.0`; every other value is `GuiError::InvalidCommand`. NaN also fails because the positive `gt miter_limit 0.0` comparison is false for NaN.

The old two-argument `gui_stroke_new color width` shape must not remain the formal constructor for geometry-capable stroke. The constructor takes every style policy explicitly so call sites show the policy they are requesting.

`GuiStroke` carries a module-private proof field. External callers cannot construct it directly from public fields, so invalid width or miter limit cannot bypass `gui_stroke_new`. Clone and Copy reconstruct the proof from the already-validated value inside the module.

F5kv is still before stroke geometry. It must not read metric owner storage, create offset points, resolve joins or caps, expand dashes, clip miters, allocate stroke edges, build coverage or packed masks, emit render commands, write pixels, or call platform APIs.

## SFNT simple glyph render stroke source contour authority boundary

F5kw consumes the completed F5ku source segment metric owner and borrows the F5av/F5aw path command authority: `GuiSfntSimpleGlyphOutlinePointStreamItemCollection` plus `GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagCompleteOwner`. This boundary exists because the metric owner alone has line/quadratic metric sequence data, but it does not own the contour, edge, path command, MoveTo, and skipped-segment provenance needed by later stroke geometry.

F5kw must read the ordered path command value stream, not the path sink scalar stream. F5ba/F5az scalar storage intentionally projects commands to compact scalar regions, and `SkipNoSegment` has no drawable scalar. Reconstructing adjacency from scalar coordinates would hide source gaps and make close/wrap behavior depend on coordinate equality. F5kw therefore treats the path command value stream as the provenance authority and treats the F5ku metric owner as the drawable metric authority.

Each drawable LineTo or QuadraticTo command produces one `GuiSfntSimpleGlyphRenderStrokeSourceMetricProvenance`:

```text
GuiSfntSimpleGlyphRenderStrokeSourceMetricProvenance:
    metric_index
    path_command_index
    edge_index
    contour_index
    contour_edge_index
    contour_start_edge_index
    contour_end_edge_index
    contour_edge_count
    event_slot
    command_tag
```

`contour_start_edge_index`, `contour_end_edge_index`, and `contour_edge_count` are checked against `gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span`. This keeps the later join/wrap phase from guessing whether a contour edge was skipped at the contour boundary. F5kw rejects any mismatch between command payload contour/edge values and the contour span derived from the collection.

The drain owner stores the metric owner, path command stream cursor, provenance Vec, total path command count, drawable provenance counts, and non-drawable MoveTo / SkipNoSegment counts. The provenance Vec is allocated once with capacity `draw_segment_count`; len starts at 0 and completion requires len/cap/counts to match the F5kr plan through the F5ku metric owner. MoveTo and SkipNoSegment are counted and advanced through the same cursor but never assigned a metric provenance slot.

LineTo and QuadraticTo handling first reads the next F5ku metric at `metric_provenance_count`, checks that the metric kind and segment index match the command, then builds provenance from the command value plus contour span. F5kw then re-reads the collection-backed curve segment for that contour-local edge and checks the F5ku metric coordinates against the source segment coordinates: line start/end and quadratic start/control/end must match. This coordinate check is only an origin guard; it does not infer contour boundaries from coordinate equality. A mismatch is a typed owner-bearing drain error. A Vec push failure returns the recovered owner with the returned Vec and rejected provenance.

F5kw does not create offset points, decide joins/caps, expand dashes, clip miters, allocate stroke edges, build coverage or packed masks, emit render commands, write pixels, call platform APIs, resolve font fallback, or infer contour boundaries from coordinate equality.

## SFNT simple glyph render stroke offset geometry boundary

F5kx consumes the completed F5kw source contour owner as the only geometry authority. This prevents the geometry expansion from falling back to the F5ku metric owner alone, the F5ba/F5az scalar stream, byte-backed glyph lookup, fill mask output, or a raster edge owner. The completed F5kw owner already ties each drawable metric slot to path command index, contour edge, contour span, command tag, and source coordinate guard, so F5kx must drain that owner rather than rediscovering source topology.

The drain owner stores:

```text
GuiSfntSimpleGlyphRenderStrokeOffsetGeometryDrainOwner:
    source_owner GuiSfntSimpleGlyphRenderStrokeSourceContourOwner
    geometry Vec GuiSfntSimpleGlyphRenderStrokeOffsetSegmentGeometry
    metric_index i32
    line_geometry_count i32
    quadratic_geometry_count i32
```

The completed owner stores the same source owner and geometry Vec with final geometry counts. Both owner types own resources and must not implement `Clone` / `Copy`.

`GuiSfntSimpleGlyphRenderStrokeOffsetNormal` is the numeric bridge from F5kt metrics to offset geometry:

```text
GuiSfntSimpleGlyphRenderStrokeOffsetNormal:
    tangent_dx i64
    tangent_dy i64
    length_squared i64
    length f32
    unit_normal_x f32
    unit_normal_y f32
    offset_x2 f32
    offset_y2 f32
    stroke_width i32
```

The normal builder uses the F5kt i64 tangent values. It casts `length_squared` to f32, calls `sqrt`, checks the result is finite and positive, then computes `(-dy / length, dx / length)`. The offset vector is that unit normal multiplied by `stroke_width`. Because glyph outlines are kept in doubled coordinates, `stroke_width` is the half-width distance in the doubled-coordinate space. A 1-pixel stroke has total doubled-coordinate width 2 and offset distance 1.

Line geometry stores both left and right offset endpoints. The input line metric still carries integer doubled start/end coordinates, while the expanded endpoints are f32 values guarded for non-finite results:

```text
left_start = start + offset
left_end = end + offset
right_start = start - offset
right_end = end - offset
```

Quadratic geometry does not pretend that the exact offset is another quadratic curve. It stores the source start/control/end coordinates, the start endpoint normal, the end endpoint normal, and the left/right offset endpoints. The start endpoint normally uses the start-control tangent and the end endpoint normally uses the control-end tangent. If one tangent length is zero, F5kx uses the nonzero tangent for that endpoint and stores the selected source as `GuiSfntSimpleGlyphRenderStrokeOffsetQuadraticTangentSource`. This preserves F5kt partial-degeneracy evidence instead of silently flattening or normalizing it away.

F5kx reads `GuiStroke` through accessors from the F5kr plan owner nested under the F5kw source owner. It rechecks positive width, finite positive miter limit, and the explicit cap / join / dash values. `GuiStrokeDash::Solid` remains explicit no-dash. If future dash variants are added, F5kx must add typed handling or typed unsupported errors; it must not approximate them as solid. Cap and join are not resolved in this boundary. They stay as style policy for the later stroke edge / join / cap phase.

Start validation order:

1. Revalidate the completed F5kw source owner shape: metric owner counts, path command count, provenance counts, MoveTo / SkipNoSegment counts, Vec len/cap, and completion counts.
2. Revalidate `GuiStroke` style via accessors.
3. Allocate the geometry Vec once with capacity equal to source drawable metric count.
4. Require initial Vec len 0 and exact capacity.

Step validation order:

1. Validate drain invariants and source owner invariants.
2. If `metric_index == metric_count`, require completion counts and return the completed owner.
3. Read provenance from F5kw owner and metric from F5ku metric owner at the same index.
4. Check provenance metric index, command tag, contour span shape, metric kind, segment index, and metric stroke width.
5. Build line or quadratic offset geometry and push it into the geometry Vec.

Push failure is owner-bearing. The returned Vec from `vec::push` is rewrapped with the pre-push metric, line, and quadratic counts, so line and quadratic failures recover the same state they had before the attempted push.

F5kx does not read path sink scalar storage, step a path command stream, call byte-backed lookup helpers, build stroke edge owners, coverage masks, packed masks, render commands, pixel buffers, platform resources, host text measurement, fallback text, shadows, or compositor output.

## SFNT simple glyph render stroke side edge owner boundary

F5ky consumes the completed F5kx offset geometry owner as the only direct authority. It must not return to the F5ba/F5az scalar stream, byte-backed glyph lookup, the F5ku metric owner by itself, a fresh F5kw cursor/drain, fill mask output, or the existing fill raster edge owner. The F5kx completed owner already ties each source geometry slot to F5kw provenance and guarded offset endpoints, so F5ky drains that owner instead of rebuilding source topology.

F5ky completed owner is a side edge record owner, not a closed stroke outline. This is intentional: line/quadratic side edges do not resolve joins, caps, miter clipping, or contour closure. A later stroke edge closure / join-cap owner must consume this owner before a stroke coverage mask owner is allowed to treat the data as a closed boundary.

The drain owner stores:

```text
GuiSfntSimpleGlyphRenderStrokeSideEdgeDrainOwner:
    geometry_owner GuiSfntSimpleGlyphRenderStrokeOffsetGeometryOwner
    edges Vec GuiSfntSimpleGlyphRenderStrokeSideEdgeRecord
    geometry_index i32
    side_phase GuiSfntSimpleGlyphRenderStrokeEdgeSide
    side_edge_count i32
    line_side_edge_count i32
    quadratic_side_edge_count i32
    left_side_edge_count i32
    right_side_edge_count i32
```

The completed owner stores the same F5kx geometry owner and side edge Vec with final counts. Both owner types own resources and must not implement `Clone` / `Copy`; the line/quadratic side edge records are value records and may be copied.

Capacity is derived as `geometry_count * 2` with an explicit overflow guard. The local invariant treats progress as `geometry_index + side_phase`: `Left` means no side edge has been emitted for the current geometry, and `Right` means the left record has been emitted and the right record is next. This makes each drain step perform exactly one `vec::push`. If a push fails, the returned Vec is rewrapped with the pre-push geometry index, side phase, and counts.

Line side edge records carry source/provenance, original source start/end, the F5kx normal, side, boundary direction, and directed side endpoints. Left side edges are source-forward (`left_start -> left_end`). Right side edges are source-reverse (`right_end -> right_start`), which prevents the later closure phase from accidentally treating the right side as source-forward.

Quadratic side edge records carry source/provenance, source start/control/end, start/end endpoint normal records, side, boundary direction, and directed side endpoints. F5ky does not invent an offset control point and does not claim that the exact offset of a quadratic is another quadratic curve. The record remains a side-edge authority for a later closure / approximation phase.

Completion requires:

1. `geometry_index == geometry_count` and `side_phase == Left`.
2. `side_edge_count == geometry_count * 2`.
3. `line_side_edge_count == line_geometry_count * 2`.
4. `quadratic_side_edge_count == quadratic_geometry_count * 2`.
5. `left_side_edge_count == right_side_edge_count == geometry_count`.
6. Vec len/cap both equal the derived side edge capacity.

F5ky does not build closure edges, coverage masks, packed masks, render commands, pixel buffers, platform resources, host text measurement, fallback text, shadows, or compositor output.

## SFNT simple glyph render stroke edge closure owner boundary

F5kz consumes the completed F5ky side edge owner as the only direct authority. It must not return to the F5ba/F5az scalar stream, byte-backed glyph lookup, the F5ku metric owner by itself, a fresh F5kw cursor/drain, the F5kx offset geometry drain, or the F5ky side edge drain. The source style is read only through the nested completed owner chain so that requested cap, join, and miter policy remain tied to the same stroke request.

F5kz stores join closure records rather than drawing join geometry. Each F5ky side edge produces exactly one `GuiSfntSimpleGlyphRenderStrokeJoinClosureRecord`, and the join Vec capacity is exactly `side_edge_count`. The completed owner requires `join_count == side_edge_count`, `left_join_count == left_side_edge_count`, `right_join_count == right_side_edge_count`, and Vec len/cap equal to `side_edge_count`.

The successor search is edge-order based:

1. Read the current F5ky side edge by checked side edge index.
2. Scan the F5ky side edge Vec for candidates with the same contour identity and the same side.
3. For `Left`, select the smallest candidate `edge_index` greater than the current edge; if none exists, wrap to the smallest candidate edge in the contour.
4. For `Right`, select the largest candidate `edge_index` smaller than the current edge; if none exists, wrap to the largest candidate edge in the contour.
5. Never reconstruct adjacency by endpoint coordinate equality.

The join record stores `from_side_edge_index`, `to_side_edge_index`, source metric indices, source edge indices, contour span metadata, directed endpoints, `GuiStrokeJoin`, `miter_limit`, `GuiSfntSimpleGlyphRenderStrokeEdgeClosureAdjacency`, and `source_edge_gap_count`. `DirectNeighbor` means the selected source edge is the next drawable edge in the side direction. `SkippedNoSegmentRange` means one or more source edges with no stroke segment were crossed without wrapping. `ContourWrap` records contour closure across the contour end/start boundary and still carries the skipped source edge count. `SelfTarget` is allowed only as explicit evidence for a contour with a single drawable side edge for that side.

Before a join is pushed, `gui_sfnt_simple_glyph_render_stroke_join_closure_record_invariants` rechecks from/to side edge index bounds, contour span, edge-order direction, adjacency, and `source_edge_gap_count`. This keeps a skipped no-segment range or a self-target closure from being silently treated as a normal neighboring join.

Cap handling is evidence-only in this phase. Simple glyph contours are closed, so F5kz records the requested `GuiStrokeCap` as `cap_policy` and records `ClosedContourNoCap` as typed evidence. This is not cap geometry and not a silent cap fallback.

Miter, bevel, and round joins remain policy records in F5kz. `GuiStrokeJoin` and finite positive `miter_limit` are carried forward; later stroke boundary phases are responsible for any geometry construction.

F5kz does not build coverage masks, packed masks, render commands, pixel buffers, platform resources, host text measurement, fallback text, shadows, or compositor output.

## SFNT simple glyph render stroke join geometry boundary

F5lc consumes the completed F5kz stroke edge closure owner as the only direct authority. It must not return to the F5ba/F5az scalar stream, byte-backed glyph lookup, the F5ku metric owner by itself, a fresh F5kw cursor/drain, the F5kx offset geometry drain, the F5ky side edge drain, or the F5kz closure drain.

F5lc materializes bevel, miter, and round join geometry from F5kz join closure records. Bevel keeps the directed `from_end -> to_start` connector chord. Miter reads the two referenced line side edges, intersects their directed infinite lines, and records the connector as `from_end -> miter` plus `miter -> to_start`. Round joins use an explicit source-center two-chord policy: F5lc reads the referenced line side edges, verifies that the directed source end and directed source start name the same source vertex, computes an arc midpoint from the endpoint radius directions, and records `from_end -> arc_mid -> to_start`. Miter / round joins that include a quadratic side edge do not invent line tangents, round centers, or offset control points; they materialize the F5kz connector chord as a bevel geometry record with `quadratic_bevel == true`. The completed owner stores the original F5kz closure owner, the geometry Vec, `join_count`, `bevel_join_count`, `miter_join_count`, and `round_join_count`; all counts and Vec len/cap are revalidated before later coverage work.

Miter clipping is explicit evidence, not an implicit bevel substitute. The miter threshold is `stroke_width * miter_limit`, where stroke width is read through the nested `GuiStroke` authority and miter limit is the value carried by the F5kz join record. Parallel lines, non-finite intersection coordinates, and threshold overflow or excess produce a bevel geometry record with `miter_clipped == true` and `quadratic_bevel == false`. Quadratic-involved bevel connector uses `miter_clipped == false` and `quadratic_bevel == true`, so it cannot be mistaken for miter-limit clipping.

Round joins fail closed only when the referenced line side edges have mismatched directed source centers, invalid stroke width evidence, or a two-chord midpoint that cannot be represented as finite f32 geometry. If either referenced side edge is quadratic, F5lc uses the typed quadratic bevel connector instead of pretending to have round geometry.

F5lc does not build coverage masks, packed masks, render commands, pixel buffers, platform resources, fallback text, shadows, or compositor output.

## SFNT simple glyph render stroke coverage mask writer owner boundary

F5la consumes the completed F5lc stroke join geometry owner as the only direct authority. It must not return to the F5ba/F5az scalar stream, byte-backed glyph lookup, the F5ku metric owner by itself, a fresh F5kw cursor/drain, the F5kx offset geometry drain, the F5ky side edge drain, the F5kz closure drain, or the F5lc join geometry drain.

F5la reuses the shared raster coverage config and shape validation helper without calling the fill coverage writer. The shared helper is allowed because it validates the pixel rectangle, sample scale, coverage max, and cell count arithmetic. The fill raster coverage writer owner, fill coverage scan converter, and fill packed mask owner remain separate authorities and are not reused directly for stroke.

F5la revalidates the completed F5lc join geometry owner before allocating the stroke coverage cells. The check reuses the nested F5kz closure invariant, requires the geometry Vec len/cap to equal `join_count`, and requires `bevel_join_count + miter_join_count + round_join_count == join_count`.

The writer owner stores the completed F5lc owner, the shared coverage shape, the i32 cell Vec, and `written_cell_count`. The completed owner is produced only when `written_cell_count == shape.cell_count` and Vec len/cap still match the exact cell count. F5la allocates the stroke coverage cell buffer but does not compute stroke coverage.

Start errors keep three independent payload channels: shape validation error, F5lc join geometry invariant error, and lower storage error. Push errors retain the writer owner and rejected coverage value; lower Vec push failure recovers the returned Vec and the pre-push `written_cell_count`.

A later stroke coverage scan converter must consume the F5la writer before packed stroke mask conversion. F5la does not build stroke coverage scans, packed masks, render commands, pixel buffers, platform resources, fallback text, shadows, or compositor output.

## SFNT simple glyph render stroke coverage scan converter boundary

F5lb consumes the F5la stroke coverage mask writer owner as the only direct authority. The completed F5lc join geometry owner, completed F5kz closure owner, and completed F5ky side edge owner are nested authority reachable through that writer. F5lb must not return to byte-backed glyph lookup, F5kx offset geometry drain, F5ky side edge drain, F5kz closure drain, F5lc join geometry drain, or the F5be fill raster coverage scan owner.

The scan owner stores the F5la writer and the current `cell_index`. Start revalidates the shared coverage shape, requires `written_cell_count == 0`, requires cell Vec len/cap to match the shape, and reruns the F5lc join geometry invariant through the F5la writer. This keeps the scan from treating stale join geometry as a closed stroke boundary.

F5lb scans line side edges, config-flattened quadratic side edges, and F5lc bevel/miter/round join geometry. Bevel geometry contributes one directed line segment, and miter and round geometry each contribute two directed line segments. The scan counts both miter segments and both round two-chord segments independently so crossing parity remains correct.

Quadratic side edges use an explicit approximation policy in F5lb. The scan owner carries a value-only `GuiSfntSimpleGlyphRenderStrokeCoverageScanConfig` with `quadratic_side_edge_segment_count > 0`. For each segment ordinal, F5lb evaluates the source quadratic point in f32, linearly interpolates the start/end endpoint normal offsets, and adds that offset to the source point. This is deterministic flattening of an approximate offset path, not an exact quadratic offset curve. Left side uses `+normal`, right side uses `-normal`, and right side segment order is reversed according to F5ky `SourceReverse`.

For every sample point, F5lb counts crossings from line side edges, quadratic side edges, and then from F5lc join geometry, using f32 scaled coordinates with finite checks. The parity of the combined crossing count determines whether the sample contributes to coverage. The scan does not write cells directly; each computed coverage value is pushed only through `gui_sfnt_simple_glyph_render_stroke_coverage_mask_writer_owner_push_cell`.

The bounded drain checks cell bounds before completion or stepping. When `cell_index == shape.cell_count`, it completes the F5la writer and produces the completed stroke coverage mask owner. Push failure and completion failure both recover the returned writer inside the scan owner, and progress is checked by requiring both `cell_index` and `written_cell_count` to advance by exactly one after a successful step.

F5lb does not build packed masks, render commands, pixel buffers, platform resources, fallback text, shadows, or compositor output. Packed stroke mask conversion and glyph paint composition remain later boundaries.

## SFNT simple glyph render stroke packed mask owner boundary

F5lf consumes the completed F5lb stroke coverage mask owner as the only direct authority. The existing F5bf fill raster packed mask owner is not reused directly because its authority is a `GuiSfntSimpleGlyphRasterCoverageMaskOwner` and completed raster edge owner, not the stroke `GuiSfntSimpleGlyphRenderStrokeJoinGeometryOwner` chain. F5lf may use the same integer alpha-normalization algorithm, but it has dedicated stroke owner types and dedicated recovery payloads.

The config is value-only:

```text
GuiSfntSimpleGlyphRenderStrokePackedMaskConfig:
    alpha_max i32
```

`GuiSfntSimpleGlyphRenderStrokePackedMaskConfig` may implement `Clone` / `Copy`. Transition, completed, error, and terminal owners must not implement `Clone` / `Copy`.

```text
GuiSfntSimpleGlyphRenderStrokePackedMaskPackOwner:
    coverage_owner GuiSfntSimpleGlyphRenderStrokeCoverageMaskOwner
    alpha_cells Vec i32
    config GuiSfntSimpleGlyphRenderStrokePackedMaskConfig
    cell_index i32

GuiSfntSimpleGlyphRenderStrokePackedMaskOwner:
    join_geometry_owner GuiSfntSimpleGlyphRenderStrokeJoinGeometryOwner
    shape GuiSfntSimpleGlyphRasterCoverageShape
    alpha_cells Vec i32
    cell_count i32
    alpha_max i32
```

Start validation is fail-closed:

```text
alpha_max > 0
shape.width_px > 0
shape.height_px > 0
shape.sample_scale > 0
shape.coverage_max == shape.sample_scale * shape.sample_scale
shape.cell_count == shape.width_px * shape.height_px
F5lc join geometry invariant is valid
shape.coverage_max * alpha_max does not overflow i32
coverage_owner.cell_count == shape.cell_count
coverage_owner.cells.len == shape.cell_count
coverage_owner.cells.cap == shape.cell_count
allocate alpha_cells with capacity shape.cell_count
```

The pack owner invariant is checked before budget handling, raw cell read, alpha normalization, Vec push, and completion:

```text
cell_index >= 0
shape invariant is valid
F5lc join geometry invariant is valid
cell_index <= shape.cell_count
alpha_cells.len == cell_index
alpha_cells.cap == shape.cell_count
coverage_owner.cell_count == shape.cell_count
coverage_owner.cells.len == shape.cell_count
coverage_owner.cells.cap == shape.cell_count
```

Raw cell read uses the completed stroke coverage owner as authority and rejects missing slots, negative coverage, and coverage above `shape.coverage_max`. Alpha normalization is integer-only and guards `coverage > max_i32 / alpha_max` before multiplying. Push failure reads the lower `StdErrorKind`, recovers the returned alpha Vec, and rebuilds the pack owner with the unchanged `cell_index`.

Completion succeeds only at exact full progress. It moves the nested `join_geometry_owner`, shape, alpha cell Vec, cell count, and alpha max into the completed packed stroke mask owner, and releases the raw stroke coverage cell Vec before returning the completed owner. The completed packed stroke mask owner never stores raw coverage cells or any F5bf raster edge owner.

F5lf does not bind glyph paint, emit render commands, write pixels, call DrawTarget / RenderTarget, call platform or host APIs, request font fallback, build shadows, or enter a 2D compositor.

## SFNT simple glyph render glyph paint composition order boundary

F5lg connects the already completed fill and stroke mask owners without crossing into command emission or resource registration. The direct inputs are:

```text
GuiSfntSimpleGlyphRenderFillAlphaMaskOwner
GuiSfntSimpleGlyphRenderStrokePackedMaskOwner
```

The boundary is deliberately fill+stroke only. A stroke packed mask whose nested stroke request chain has `fill == None` is rejected as `MissingStrokeFillMetadata`; a later sibling boundary can define stroke-only order without pretending that a missing fill owner exists.

Euler revised plan review は `PLAN_APPROVED` after the scope was narrowed to fill+stroke, both owner shapes were revalidated independently before tuple comparison, stroke-only was left out of scope, nested stroke metadata extraction was made explicit, and the error recovery contract was changed to recover both owners together.

The stroke metadata is not reconstructed from geometry. It is read through the existing owner chain:

```text
stroke_packed.join_geometry_owner.edge_closure_owner.side_edge_owner.geometry_owner.source_owner.metric_owner.plan_owner
```

That plan owner is the authority for `origin`, optional `fill`, `stroke`, and `blend`, because F5kq stored the original glyph paint subset before stroke segment expansion. F5lg preserves the `stroke` value from this chain in the completed order owner.

Validation order is:

1. Revalidate the completed fill owner shape and alpha storage: positive width/height/sample scale, `coverage_max == sample_scale * sample_scale`, `cell_count == width_px * height_px`, positive alpha max, and owner cell count / alpha len / alpha cap equal to shape cell count.
2. Revalidate the completed stroke packed owner shape and alpha storage with the same checks.
3. Revalidate the nested completed stroke join geometry invariant and keep the lower `GuiSfntSimpleGlyphRenderStrokeJoinGeometryErrorKind` if it fails.
4. Compare the full shape tuple: `origin_x2`, `origin_y2`, `width_px`, `height_px`, `sample_scale`, `coverage_max`, `cell_count`.
5. Read the optional fill from the nested stroke metadata and reject `None` as `MissingStrokeFillMetadata` before origin or blend mismatch checks.
6. Compare fill owner origin, fill paint, and blend with the nested stroke metadata.
7. Reject any non-SourceOver blend before producing the completed order owner.

The completed owner is module-private and owns both mask owners:

```text
GuiSfntSimpleGlyphRenderGlyphPaintCompositionOrderOwner:
    fill_owner GuiSfntSimpleGlyphRenderFillAlphaMaskOwner
    stroke_owner GuiSfntSimpleGlyphRenderStrokePackedMaskOwner
    origin GuiPoint
    fill_paint GuiPaint
    stroke GuiStroke
    blend GuiBlendMode
    fill_order i32
    stroke_order i32
```

`fill_order` is always `0`; `stroke_order` is always `1`. The owner, start error, and recovery payload do not implement `Clone` / `Copy`. The start error exposes kind and optional lower join error, but owner recovery is a single consuming recovery path returning both owners together. Free paths release the fill owner first and then the stroke owner.

The fill paint comparison uses typed `GuiPaint` / `Rgba8888` accessors. It must not compare source text or struct layout strings. F5lg does not inspect raw F5bf raster packed mask internals; it accepts only the F5bh/F5bg completed fill owner and F5lf completed stroke packed owner.

F5lg does not allocate alpha-mask ids, reserve/register resource table records, emit `RenderCommand`, write pixels, drain a software surface, touch platform/backend APIs, invoke fallback text, start shadow rasterization, or invoke the 2D compositor.

## SFNT simple glyph render stroke-only composition order boundary

F5lh is the stroke-only sibling of F5lg. It consumes one already completed stroke mask owner:

```text
GuiSfntSimpleGlyphRenderStrokePackedMaskOwner
```

It deliberately does not accept a fill owner. If the nested stroke request metadata still contains `fill == Some`, start returns `UnexpectedStrokeFillMetadata` so fill+stroke glyph paint remains routed through F5lg. This prevents a fake fill order from being invented for stroke-only text and prevents the stroke-only path from silently replacing the fill+stroke ordering contract.

Euclid plan review は `PLAN_APPROVED` after the scope was kept to stroke-only, the direct authority was limited to the completed stroke packed owner, `Some(fill)` was made a typed rejection, lower join geometry evidence was preserved, recovery was kept as a single stroke-owner path, and the F5lg source policy slice was required to end at the F5lh marker.

F5lh reads stroke metadata through the same completed owner chain as F5lg:

```text
stroke_packed.join_geometry_owner.edge_closure_owner.side_edge_owner.geometry_owner.source_owner.metric_owner.plan_owner
```

That plan owner is the authority for `origin`, optional `fill`, `stroke`, and `blend`. F5lh may reuse module-private F5lg helpers for the common stroke-owner invariant, nested plan-owner chain, and SourceOver predicate, but it maps failures to its own `GuiSfntSimpleGlyphRenderStrokeOnlyCompositionOrderStartErrorKind`. The F5lg fill+stroke error kind is not the public contract of F5lh.

Validation order is:

1. Revalidate the completed stroke packed owner shape and alpha storage: positive width/height/sample scale, `coverage_max == sample_scale * sample_scale`, `cell_count == width_px * height_px`, positive alpha max, and owner cell count / alpha len / alpha cap equal to shape cell count.
2. Revalidate the nested completed stroke join geometry invariant and keep the lower `GuiSfntSimpleGlyphRenderStrokeJoinGeometryErrorKind` if it fails.
3. Read the nested stroke segment plan owner from the completed stroke packed owner chain.
4. Require the plan owner fill metadata to be `None`; reject `Some(_)` as `UnexpectedStrokeFillMetadata`.
5. Reject any non-SourceOver blend.
6. Preserve origin, stroke, blend, and `stroke_order = 0` in the completed owner.

The completed owner is module-private and owns the stroke mask owner:

```text
GuiSfntSimpleGlyphRenderStrokeOnlyCompositionOrderOwner:
    stroke_owner GuiSfntSimpleGlyphRenderStrokePackedMaskOwner
    origin GuiPoint
    stroke GuiStroke
    blend GuiBlendMode
    stroke_order i32
```

The owner, start error, and recovery payload do not implement `Clone` / `Copy`. The start error exposes kind and optional lower join error, but owner recovery is a single consuming path returning the stroke owner. Free paths release the stroke packed mask owner once.

F5lh does not allocate alpha-mask ids, reserve/register resource table records, emit `RenderCommand`, write pixels, drain a software surface, touch platform/backend APIs, invoke fallback text, start shadow rasterization, invoke the 2D compositor, or inspect raw F5bf raster packed mask internals.

## SFNT simple glyph render shadow request boundary

F5li starts the shadow path without pretending to have a shadow rasterizer. It consumes the same kind of completed path command stream writer authority as F5kq and keeps exactly one `SingleShadow` value with the source paint metadata needed by later shadow mask phases.

The direct inputs are:

```text
GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkWriterOwner
GuiSfntSimpleGlyphRenderShadowRequestConfig
```

Socrates plan review 1 was `CHANGES_REQUESTED`. The plan had to add `stroke.width > 0` revalidation because F5li stores optional stroke metadata for later shadow source selection. It also had to avoid copying the full F5kq writer-authority validation into F5li. The revised plan was `PLAN_APPROVED` after the common writer authority helper and F5li-specific error mapping were added.

The common helper is module-private and neutral:

```text
gui_sfnt_simple_glyph_render_path_command_writer_authority
```

It borrows the completed path command writer and validates stored capacity against the owner plan, path sink scalar capacity, raster mask scalar capacity, path sink scalar len, zero raster mask scalar len, written count, path sink scalar count, move-to count, line-to count, quadratic-to count, skip-no-segment count, and last path command index. The helper returns value error metadata. F5kq maps it into `GuiSfntSimpleGlyphRenderStrokeRequestStartErrorKind`; F5li maps it into `GuiSfntSimpleGlyphRenderShadowRequestStartErrorKind`. This keeps the authority check in one place while preserving typed request-specific error surfaces.

F5li accepts only a single no_alloc shadow value:

```text
GuiShadowRef::NoShadow -> MissingShadowPaint
GuiShadowRef::ShadowRun -> UnsupportedShadowRun
GuiShadowRef::SingleShadow shadow -> continue
```

`ShadowRun` is intentionally rejected because this slice does not own or resolve an alloc-backed multi-shadow run. Accepting the id without a resolver would only move an opaque reference forward with no authority to expand it.

Validation order is:

1. Read `GuiGlyphPaint.shadows`.
2. Reject `NoShadow` and `ShadowRun`; accept `SingleShadow`.
3. Revalidate `gui_shadow_blur_radius &shadow >= 0` and `gui_shadow_spread &shadow >= 0`.
4. Read fill and stroke metadata.
5. Require at least one of fill or stroke as the shadow source metadata.
6. If stroke metadata is present, revalidate `gui_stroke_width &stroke > 0`.
7. Read glyph blend and require SourceOver.
8. Revalidate the completed path command writer authority through the common helper.
9. Produce the request owner.

The success owner is private and owner-bearing:

```text
GuiSfntSimpleGlyphRenderShadowRequestOwner:
    writer GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkWriterOwner
    origin GuiPoint
    fill Option GuiPaint
    stroke Option GuiStroke
    shadow GuiShadow
    blend GuiBlendMode
```

The owner, start error, and recovery payload do not implement `Clone` / `Copy`. Start error recovery consumes the error and returns writer and config together. Free paths release the path command writer. There is no fill owner, stroke owner, raw path-sink storage accessor, shadow mask, blur kernel, spread geometry, alpha-mask resource, render command, software surface, platform/backend call, fallback, or compositor drain in this phase.

F5li does not change F5bh or F5kq. The no-shadow fill and stroke paths continue to reject shadow-bearing `GuiGlyphPaint` so a caller cannot accidentally route a shadow paint through a non-shadow path.

## SFNT simple glyph render shadow source coverage config boundary

F5lj turns a F5li shadow request into a validated source coverage plan. It deliberately keeps the boundary before edge draining and before mask allocation. The input coverage dimensions are caller supplied, matching the F5bd rule that coverage shape is not inferred from edge storage.

The direct inputs are:

```text
GuiSfntSimpleGlyphRenderShadowRequestOwner
GuiSfntSimpleGlyphRenderShadowSourceCoverageConfig
```

Darwin plan review 1 was `CHANGES_REQUESTED`. The initial plan validated `spread + blur_radius` without storing the result, did not revalidate source fill/stroke metadata, and kept both coverage config and source shape in the success owner. The revised plan stores `shadow_extent`, revalidates source metadata, removes coverage config from the success owner, and makes `source_shape` the only canonical coverage authority. Darwin revised plan review was `PLAN_APPROVED`.

The config is value-only:

```text
GuiSfntSimpleGlyphRenderShadowSourceCoverageConfig:
    coverage_config GuiSfntSimpleGlyphRasterCoverageConfig
```

The success owner is owner-bearing and not copyable:

```text
GuiSfntSimpleGlyphRenderShadowSourceCoverageOwner:
    request_owner GuiSfntSimpleGlyphRenderShadowRequestOwner
    source_shape GuiSfntSimpleGlyphRasterCoverageShape
    source_fill Option GuiPaint
    source_stroke Option GuiStroke
    source_placement_origin GuiPoint
    shadow_offset GuiPoint
    shadow_blur_radius i32
    shadow_spread i32
    shadow_extent i32
    shadow_paint GuiPaint
    blend GuiBlendMode
```

`source_shape is the only canonical coverage authority` on success. `coverage_config` is only retained by config/start-error recovery, so later boundaries cannot accidentally rederive a different shape from the original config. Source policy pins this by checking that the success owner has no `coverage_config` field.

Validation order is:

1. Read the shadow from the request owner and revalidate `blur_radius >= 0`.
2. Revalidate `spread >= 0`.
3. Read fill/stroke metadata, require at least one source, and revalidate optional stroke width.
4. Read blend and require SourceOver.
5. Read caller supplied coverage config and validate it with `gui_sfnt_simple_glyph_raster_coverage_shape_from_config`.
6. Compute `source_placement_origin = request_origin + shadow.offset` with checked per-axis i32 overflow before `gui_point_new`.
7. Compute and store `shadow_extent = spread + blur_radius` with checked i32 overflow.
8. Read shadow paint and create the owner.

F5lj maps only shape/config coverage errors. Since `shape_from_config` does not inspect edge storage or allocate cells, edge count/storage/allocation lower errors are not expected. The generic `CoverageUnexpectedLowerError` is a fail-closed guard if that lower helper contract changes.

F5lj must not call raster edge owner start/drain, coverage mask writer start, scan conversion, packed mask conversion, fill/stroke composition owners, alpha-mask resource reservation, render command constructors, software drain, platform/backend APIs, font fallback, shadow rasterizer, or compositor APIs.

## SFNT simple glyph render shadow source edge drain owner boundary

F5lk consumes the F5lj shadow source coverage owner and creates a shadow-specific raster edge owner. It is intentionally not the generic raster edge drain boundary. The generic raster edge drain owner is not reused because it requires a completed raster mask writer owner and validates raster mask scalar len/count. At the F5lj boundary the only completed geometry stream is `path_sink_scalars`; no raster mask scalar writer exists yet.

Laplace plan review 1 was `CHANGES_REQUESTED`. The first plan tried to route through the generic raster edge drain, which would require a completed raster mask writer and would fail the F5lj writer state where raster mask scalar len is still 0. The revised plan moved to a shadow-specific path-sink scalar drain, but still treated `SkipNoSegment` as if it had a scalar record. Laplace revised plan 1 was `CHANGES_REQUESTED` because skip commands have no `path_sink_scalars` record. Laplace revised plan 2 is `PLAN_APPROVED`: F5lk reads only `MoveTo=1`, `LineTo=2`, and `QuadraticTo=3`, rejects tag 4 with `UnexpectedSkipNoSegmentTag`, keeps skip progress out of the drain owner, and allows zero drawable edges.

The value-only context duplicates the request metadata and F5lj canonical metadata so later drain steps can revalidate without retaining the F5lj request owner:

```text
GuiSfntSimpleGlyphRenderShadowSourceEdgeContext:
    request_origin GuiPoint
    request_fill Option GuiPaint
    request_stroke Option GuiStroke
    request_shadow GuiShadow
    request_blend GuiBlendMode
    source_shape GuiSfntSimpleGlyphRasterCoverageShape
    source_fill Option GuiPaint
    source_stroke Option GuiStroke
    source_placement_origin GuiPoint
    shadow_offset GuiPoint
    shadow_blur_radius i32
    shadow_spread i32
    shadow_extent i32
    shadow_paint GuiPaint
    blend GuiBlendMode
```

The drain owner owns the completed writer and edge storage:

```text
GuiSfntSimpleGlyphRenderShadowSourceEdgeDrainOwner:
    writer GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkWriterOwner
    context GuiSfntSimpleGlyphRenderShadowSourceEdgeContext
    edges Vec GuiSfntSimpleGlyphRasterEdge
    scalar_index i32
    edge_count i32
    line_edge_count i32
    quadratic_edge_count i32
    move_to_count i32
    has_current_point bool
    current_x2 i32
    current_y2 i32
```

Start validation borrows the F5lj owner, builds the context, revalidates source fill/stroke, SourceOver blend, shadow offset/blur/spread/paint, checked placement origin, checked extent, and shape arithmetic. It then validates the shared writer authority and checks `line_to_count + quadratic_to_count == raster_edge_capacity`. The edge Vec is allocated before the F5lj owner is consumed, so allocation failure returns the original owner in the start error. Only after successful allocation does start split the nested request owner and move out the path command writer.

The drain reads `path_sink_scalars` directly:

- `MoveTo`: requires three scalars and updates current point.
- `LineTo`: requires a current point and three scalars, then pushes one line raster edge.
- `QuadraticTo`: requires a current point and five scalars, then pushes one quadratic raster edge.
- `SkipNoSegment`: is never valid in `path_sink_scalars`; tag 4 returns `UnexpectedSkipNoSegmentTag`.

Completion requires `scalar_index == path_sink_scalar_count`, move/line/quadratic counts equal the writer plan, `edge_count == raster_edge_capacity`, and Vec len/cap match exactly. `line_to_count + quadratic_to_count == 0` completes as an empty exact-capacity owner rather than an error.

F5lk must not call coverage mask writer start, mask scan conversion, packed mask conversion, blur kernel construction, render command constructors, resource table, software surface, platform/backend APIs, font fallback, shadow rasterizer, or compositor APIs.

## SFNT simple glyph render shadow source coverage mask writer owner boundary

F5ll consumes the completed F5lk shadow source edge owner and creates the raw coverage cell writer for the later shadow source scan converter. It is writer-only. It allocates the raw cell Vec and exposes push / completion / free paths, but it does not compute coverage, build blur kernels, pack alpha cells, reserve resources, emit render commands, write pixels, call platform APIs, or drain a compositor.

Turing plan review 1 was `CHANGES_REQUESTED`. The initial plan allowed `edges.cap == edge_count or raster_edge_capacity` and did not explicitly require F5ll start to revalidate F5lk's completed owner against the nested writer plan / capacity. Turing revised plan review is `PLAN_APPROVED`: F5ll must require `line_edge_count == plan.line_to_count`, `quadratic_edge_count == plan.quadratic_to_count`, `edge_count == raster_edge_capacity`, checked `edge_count == line_edge_count + quadratic_edge_count`, `edges.len == edge_count`, and `edges.cap == raster_edge_capacity`.

The writer owner is:

```text
GuiSfntSimpleGlyphRenderShadowSourceCoverageMaskWriterOwner:
    edge_owner GuiSfntSimpleGlyphRenderShadowSourceEdgeOwner
    source_shape GuiSfntSimpleGlyphRasterCoverageShape
    cells Vec i32
    written_cell_count i32
```

The completed owner is:

```text
GuiSfntSimpleGlyphRenderShadowSourceCoverageMaskOwner:
    edge_owner GuiSfntSimpleGlyphRenderShadowSourceEdgeOwner
    source_shape GuiSfntSimpleGlyphRasterCoverageShape
    cells Vec i32
    cell_count i32
```

`source_shape` is a cached canonical value copied from the F5lk context. F5ll does not accept a new coverage config and does not create a second shape authority. Writer invariants compare the cached value against the nested F5lk context value and fail closed on mismatch.

Start revalidates the F5lk context invariants, shared writer authority, nested writer plan and capacity, edge count / storage exactness, then allocates `Vec<i32>` with exact `source_shape.cell_count`. Allocation happens after the owner has been validated; allocation failure returns the original F5lk edge owner inside the start error.

Push validation requires `cells.len == written_cell_count`, `cells.cap == source_shape.cell_count`, `written_cell_count <= source_shape.cell_count`, and `coverage_value` in `0..=source_shape.coverage_max`. Vec push failure reconstructs the writer owner with the returned Vec and the pre-push `written_cell_count`.

Completion returns `CoverageMaskCompleted` only when `written_cell_count == source_shape.cell_count`. If fewer cells have been written, it returns `CoverageMaskIncomplete`. Free paths release raw cells before the F5lk edge owner. A zero-edge glyph may still have nonzero cell count; F5ll does not reject it because the later scan converter is responsible for writing zero coverage cells.

F5ll must not call generic raster coverage writer / scan / packed-mask helpers, stroke coverage writer / scan / packed-mask helpers, path-sink scalar drains, generic raster edge owner/drain helpers, blur kernel construction, render command constructors, resource table, software surface, platform/backend APIs, font fallback, shadow rasterizer, or compositor APIs.

## SFNT simple glyph render shadow source coverage scan converter boundary

F5lm consumes the F5ll shadow source coverage mask writer owner as the only direct authority. It scans the nested F5lk shadow source raster edges and writes raw source coverage cells through the F5ll `push_cell` boundary. It does not allocate blur kernels, mutate spread geometry, pack alpha cells, reserve resources, emit render commands, write pixels, call platform APIs, or drain a compositor.

Kuhn plan review was `PLAN_APPROVED`. The review explicitly allowed reuse of owner-free integer geometry helpers from F5be (`sample_coordinate`, `scaled_edge_coordinate`, `line_crosses_scaled`, and `quadratic_point_scaled`) because they carry no F5be writer authority. F5lm must still keep its own config, owner, start error, scan error, terminal, recovery, and free paths, and source policy must forbid F5be owner / error / terminal / direct writer-owner paths.

The scan config is value-only:

```text
GuiSfntSimpleGlyphRenderShadowSourceCoverageScanConfig:
    quadratic_segment_count i32
```

The resumable scan owner is owner-bearing and non-copyable:

```text
GuiSfntSimpleGlyphRenderShadowSourceCoverageScanOwner:
    writer GuiSfntSimpleGlyphRenderShadowSourceCoverageMaskWriterOwner
    config GuiSfntSimpleGlyphRenderShadowSourceCoverageScanConfig
    cell_index i32
```

Start validation first rejects `quadratic_segment_count <= 0`, then calls `gui_sfnt_simple_glyph_render_shadow_source_coverage_mask_writer_owner_invariants`. Lower F5ll error kinds are stored in the start error payload instead of being collapsed into a string or silent failure. Shape validation is repeated at this trust boundary because scan conversion uses division, modulo, sample loops, and coverage range:

```text
width_px > 0
height_px > 0
sample_scale > 0
coverage_max == sample_scale * sample_scale
cell_count == width_px * height_px
```

The writer must not already be started. `written_cell_count == 0`, `cells.len == 0`, and `cells.cap == source_shape.cell_count` are checked before the scan owner is created.

Edge reads go through the F5ll writer into the nested F5lk edge owner. F5lm revalidates the F5lk edge owner before reading and checks edge index bounds, edge Vec length, capacity, and slot presence. The completed edge owner remains the source of edge authority; F5lm does not reconstruct a generic raster edge owner.

Cell coverage uses the same integer sampling model as F5be. Each cell is split into `sample_scale * sample_scale` subpixel samples. A sample is inside when the total line / quadratic crossing count is odd. Line crossing uses strict y activation and i64 cross-product comparison. Quadratic edges are evaluated with the F5lm `quadratic_segment_count` and source/control/end points; each segment is tested with the line crossing helper.

`gui_sfnt_simple_glyph_render_shadow_source_coverage_scan_owner_step` computes one cell coverage, then pushes through `gui_sfnt_simple_glyph_render_shadow_source_coverage_mask_writer_owner_push_cell`. Push failure reconstructs the scan owner with the returned writer and the pre-push `cell_index`. The bounded drain only calls completion when `cell_index == cell_count`; incomplete completion is an error, budget exhaustion is a typed terminal, and post-step progress must advance both `cell_index` and writer `written_cell_count` by exactly one.

Zero-edge glyphs are valid. With zero edges the crossing count is always zero, so F5lm writes zero coverage cells through the same loop instead of using a zero-fill shortcut or completing a partial mask.

## SFNT simple glyph render shadow source blur mask owner boundary

F5ln consumes the completed F5lm shadow source raw coverage mask owner and produces a completed shadow coverage mask after spread and blur. It is still pre-packing and pre-composition. It does not normalize to alpha cells, reserve resources, emit render commands, write pixels, call platform APIs, or drain a compositor.

Harvey plan review was `PLAN_APPROVED`. The review confirmed that blur should operate before packing, because F5ln still has raw coverage values and can preserve coverage semantics before later alpha normalization. The review also confirmed that the blur owner should keep the nested F5lk edge authority instead of copying shadow paint / blend into this boundary, and that completion must free the raw source cells.

The transition owner is owner-bearing and non-copyable:

```text
GuiSfntSimpleGlyphRenderShadowSourceBlurMaskBuildOwner:
    coverage_owner GuiSfntSimpleGlyphRenderShadowSourceCoverageMaskOwner
    shadow_shape GuiSfntSimpleGlyphRasterCoverageShape
    cells Vec i32
    cell_index i32
```

The completed owner stores both shapes and the blurred shadow cells:

```text
GuiSfntSimpleGlyphRenderShadowSourceBlurMaskOwner:
    edge_owner GuiSfntSimpleGlyphRenderShadowSourceEdgeOwner
    source_shape GuiSfntSimpleGlyphRasterCoverageShape
    shadow_shape GuiSfntSimpleGlyphRasterCoverageShape
    cells Vec i32
    cell_count i32
```

The completed owner does not store shadow paint or blend. Later packing / composition must read those values from the nested F5lk context. This keeps F5ln focused on coverage generation and avoids creating a second shadow metadata authority.

F5ln starts by validating the completed F5lm owner:

```text
F5lk completed edge owner invariant holds
cached source_shape equals the F5lk context source_shape
source shape has valid width / height / sample scale / coverage_max / cell_count
raw cell_count == source_shape.cell_count
raw cells.len == source_shape.cell_count
raw cells.cap == source_shape.cell_count
shadow blur radius >= 0
shadow spread >= 0
shadow_extent == spread + blur_radius
```

Shadow shape is derived, not supplied by the caller. `padding_px = shadow_extent`; origin uses `source_origin_x2/y2 - padding_px * 2`; dimensions use `source_width/height + padding_px * 2`; sample scale and coverage max are copied from the source shape. Origin, dimension, and cell-count arithmetic use checked i32 roundtrip logic before a `GuiSfntSimpleGlyphRasterCoverageShape` value is created.

The spread operation is an inclusive square max-filter spread. For a logical source coordinate `(x, y)`, F5ln reads every coordinate in `[x - spread, x + spread] x [y - spread, y + spread]` and keeps the maximum raw coverage. Read coordinates outside the source shape are zero; they do not reach `vec::get`.

The blur operation is an inclusive square box filter over the spread result. For a logical source coordinate `(x, y)`, F5ln sums the spread values for `[x - blur_radius, x + blur_radius] x [y - blur_radius, y + blur_radius]`, then divides by the kernel count using truncating integer division. Kernel validation checks `(2 * spread + 1)^2`, `(2 * blur_radius + 1)^2`, and `blur_kernel_count * coverage_max`, so accumulation stays within i32.

Step maps the shadow cell index to `(shadow_x, shadow_y)`, subtracts `shadow_extent` to get source logical coordinates, computes the blurred coverage, and pushes one cell into the output Vec. Push failure returns a build owner with the returned Vec and unchanged `cell_index`.

The bounded drain completes only when `cell_index == shadow_shape.cell_count`. Otherwise it either returns `StepBudgetExhausted` or advances by one cell and checks that both `cell_index` and output Vec length advanced by exactly one. Completion destructures the F5lm source coverage owner, frees raw source cells, and returns a completed blur mask owner with the original F5lk edge owner, source shape, shadow shape, blurred cells, and shadow cell count.

F5ln must not call generic raster coverage scan owners, packed-mask helpers, stroke coverage helpers, fill alpha mask helpers, render command constructors, resource table helpers, software surfaces, platform/backend APIs, font fallback, shadow rasterizer, or compositor APIs.

## SFNT simple glyph render shadow source packed mask owner boundary

F5lo consumes the completed F5ln shadow source blur mask owner and produces a completed shadow source alpha mask owner. It is still pre-composition and pre-render. It does not reserve resources, emit render commands, write pixels, call platform APIs, or drain a compositor.

Pasteur plan review was `PLAN_APPROVED`. The review confirmed that F5lo should use the completed F5ln blur owner as direct authority, normalize raw blurred coverage while it still owns those cells, keep shadow paint / blend in the nested F5lk context, and ensure raw blurred coverage cells are freed at completion.

The config is value-only:

```text
GuiSfntSimpleGlyphRenderShadowSourcePackedMaskConfig:
    alpha_max i32
```

The transition owner is owner-bearing and non-copyable:

```text
GuiSfntSimpleGlyphRenderShadowSourcePackedMaskPackOwner:
    blur_owner GuiSfntSimpleGlyphRenderShadowSourceBlurMaskOwner
    alpha_cells Vec i32
    config GuiSfntSimpleGlyphRenderShadowSourcePackedMaskConfig
    cell_index i32
```

The completed owner stores both shapes and the packed alpha cells:

```text
GuiSfntSimpleGlyphRenderShadowSourcePackedMaskOwner:
    edge_owner GuiSfntSimpleGlyphRenderShadowSourceEdgeOwner
    source_shape GuiSfntSimpleGlyphRasterCoverageShape
    shadow_shape GuiSfntSimpleGlyphRasterCoverageShape
    alpha_cells Vec i32
    cell_count i32
    alpha_max i32
```

The completed owner does not store shadow paint or blend. Later composition must read those values from the nested F5lk context. This keeps F5lo focused on alpha packing and avoids creating a second shadow metadata authority.

F5lo starts by validating:

```text
alpha_max > 0
F5ln completed blur owner invariant holds
raw blur cell_count == shadow_shape.cell_count
raw blur cells.len == shadow_shape.cell_count
raw blur cells.cap == shadow_shape.cell_count
coverage_max * alpha_max fits in i32
```

The conversion for each cell is:

```text
alpha = (coverage * alpha_max) / coverage_max
```

The division uses truncating integer division. Step revalidates the F5ln blur owner invariant, checks `cell_index` and alpha Vec exactness, reads the raw blurred cell at `cell_index`, rejects missing / negative / greater-than-coverage-max values, checks `coverage * alpha_max`, then pushes one alpha cell. Push failure returns a pack owner with the returned Vec and unchanged `cell_index`.

The bounded drain completes only when `cell_index == shadow_shape.cell_count`. Otherwise it returns `StepBudgetExhausted` or advances by one cell and checks that both `cell_index` and alpha Vec length advanced by exactly one. Completion destructures the F5ln blur owner, frees raw blurred coverage cells, and returns a completed packed mask owner with the original F5lk edge owner, source shape, shadow shape, alpha cells, shadow cell count, and alpha max.

F5lo also provides a completed owner invariant for later composition boundaries. It revalidates the nested F5lk edge owner, checks cached `source_shape` against the edge context, rederives the expected `shadow_shape`, verifies `alpha_max > 0`, and requires `cell_count`, alpha Vec len, and alpha Vec cap to equal the shadow cell count.

F5lo must not call generic raster packed mask owners, stroke packed mask owners, fill alpha mask helpers, glyph paint composition order helpers, render command constructors, resource table helpers, software surfaces, platform/backend APIs, font fallback, shadow rasterizer, or compositor APIs.

## SFNT simple glyph render shadow source composition order boundary

F5lp consumes the completed F5lo shadow source packed mask owner and fixes only the shadow contribution order relative to the source paint. It is still pre-sample, pre-resource, pre-command, pre-pixel, pre-platform, and pre-compositor.

Hume plan review was `PLAN_APPROVED`. The review confirmed that F5lp should use only the completed F5lo owner as direct authority, keep fill/stroke owners out of this boundary, preserve lower packed-mask and lower edge error evidence, and store shadow paint / blend / placement metadata only as downstream fixed evidence revalidated against the nested F5lk context.

The completed owner is owner-bearing and non-copyable:

```text
GuiSfntSimpleGlyphRenderShadowSourceCompositionOrderOwner:
    shadow_owner GuiSfntSimpleGlyphRenderShadowSourcePackedMaskOwner
    source_placement_origin GuiPoint
    shadow_offset GuiPoint
    shadow_extent i32
    shadow_paint GuiPaint
    blend GuiBlendMode
    shadow_order i32
    source_order i32
```

`shadow_order` is fixed to `0` and `source_order` is fixed to `1`. This represents shadow contribution before the source paint. It does not imply a render command or pixel write; it is metadata for the later bridge.

Start validates:

```text
F5lo completed packed mask owner invariant holds
nested F5lk edge owner invariant holds
context blend is supported by the shadow source edge helper
shadow_order == 0
source_order == 1
```

When the F5lo invariant fails, the start error stores the returned packed-mask error kind as lower packed error evidence. If the packed-mask invariant collapsed nested edge invalidity into `EdgeOwnerInvariantFailed`, F5lp probes the nested edge invariant again and stores the concrete lower edge error evidence alongside the packed error. When the explicit nested F5lk edge owner invariant fails after a successful F5lo invariant, the start error stores the returned lower edge error evidence. Recovery keeps the original F5lo owner so ownership can be closed exactly once.

The completed invariant revalidates the F5lo owner and nested F5lk edge owner, then compares the stored `source_placement_origin`, `shadow_offset`, `shadow_extent`, `shadow_paint`, and `blend` against the current F5lk context. Point, paint, and blend equality use the typed F5lk helpers rather than raw field comparison. The invariant also rechecks SourceOver-only support and the fixed order values.

F5lp must not consume fill/stroke owners, call F5lg or F5lh glyph paint composition helpers, allocate resource IDs, emit render commands, reserve tables, sample pixels, write software surfaces, call platform/backend APIs, invoke fallback, use shadow rasterizers, or drain a compositor.

## SFNT simple glyph render shadow source sample cursor boundary

F5lq consumes the completed F5lp shadow source composition order owner and exposes the shadow contribution as a cell-by-cell value sample stream. It is still pre-resource, pre-command, pre-pixel, pre-platform, and pre-compositor.

Carson plan review was `PLAN_APPROVED`. The review confirmed that F5lq should use only the completed F5lp owner as direct authority, keep fill/stroke owners and command/resource APIs out of this boundary, preserve lower F5lp order invariant evidence through `order_error`, and compute sample positions from the shadow shape local cell with checked arithmetic.

The sample is copyable and value-only:

```text
GuiSfntSimpleGlyphRenderShadowSourceSample:
    position GuiPoint
    alpha i32
    alpha_max i32
    shadow_paint GuiPaint
    blend GuiBlendMode
    shadow_order i32
    source_order i32
```

The cursor owns the F5lp composition order owner and the current cell index:

```text
GuiSfntSimpleGlyphRenderShadowSourceSampleCursor:
    owner GuiSfntSimpleGlyphRenderShadowSourceCompositionOrderOwner
    cell_index i32
```

The cursor, start error, step error, sampled terminal payload, and terminal are owner-bearing and must not implement `Clone` / `Copy`.

Start and step first re-run the F5lp completed owner invariant. If it fails, F5lq reports `CompositionOrderInvariantFailed` and stores the returned lower F5lp error kind in `order_error`. Start errors recover the original owner. Step errors recover the original cursor. This preserves the root composition-order reason instead of collapsing it into a generic sample read failure.

F5lq then revalidates the nested F5lo packed shadow storage through the F5lp owner:

```text
shadow shape dimensions and sample scale are positive
coverage_max == sample_scale * sample_scale
cell_count == width_px * height_px
alpha_max > 0
packed owner cell_count equals shadow cell_count
alpha Vec len and cap equal shadow cell_count
```

Read accepts only `cell_index < cell_count`. `cell_index == cell_count` is reserved for `step` completion and is rejected by read. `cell_index > cell_count` is rejected before completion, so forged cursor progress cannot become a successful completed state.

Position construction is:

```text
source_placement_origin + local - shadow_extent
```

where `local_y = cell_index / width_px` and `local_x = cell_index - local_y * width_px`. The implementation performs checked i32 subtraction for `local - shadow_extent` and checked i32 addition for the origin before calling `gui_point_new`.

The sample payload copies alpha from the nested F5lo alpha cells and copies `shadow_paint`, `blend`, `shadow_order`, and `source_order` from the F5lp owner. It does not allocate a resource id, emit a render command, or write pixels.

Free paths close exactly one F5lp composition order owner: cursor free closes the owner, start error free closes the recovered owner, step error free closes the recovered cursor, and terminal free closes either the sampled cursor or the completed owner.

F5lq must not call fill/stroke owners, F5lg/F5lh composition helpers, render command constructors, resource table or reservation helpers, software surfaces, platform/backend APIs, font fallback, shadow rasterizers, compositor APIs, or owner-bearing Vec allocation/push helpers.

## SFNT simple glyph render shadow source sample command bridge boundary

F5lr is a SourceOver only bridge from the F5lq shadow sample stream to the existing core `RenderCommand::FillRect` value. It is a correctness bridge for one shadow contribution command, not the final FHD 60fps compositor path, not a resource registration boundary, and not a backend fallback.

Aristotle plan review was `PLAN_APPROVED`. The review confirmed that F5lr should mirror the F5bj conversion-before-advance rule, keep resource/platform/compositor paths out of scope, preserve lower F5lp order evidence only for the explicit composition-order invariant precheck, and document that the emitted `FillRect` does not encode final shadow/source composition ordering.

The conversion validates the sample blend before command construction because `FillRectCommand` does not carry `GuiBlendMode`:

```text
GuiBlendMode::SourceOver -> Ok
other blend modes -> UnsupportedBlendMode
```

This prevents semantic loss. F5lr must not silently drop `sample.blend`, and it must not reinterpret unsupported blend modes as SourceOver.

The command paint keeps the RGB channels from `sample.shadow_paint` and scales only alpha:

```text
command_alpha = sample.alpha * shadow_paint.alpha / sample.alpha_max
```

The multiplication is checked before division and before the final `u8` cast. `sample.alpha == 0` or `shadow_paint.alpha == 0` returns a transparent shadow contribution command, not a silent skip.

The cursor command error is owner-bearing:

```text
GuiSfntSimpleGlyphRenderShadowSourceSampleCommandCursorError:
    kind GuiSfntSimpleGlyphRenderShadowSourceSampleCommandCursorErrorKind
    cursor GuiSfntSimpleGlyphRenderShadowSourceSampleCursor
    rejected_sample Option GuiSfntSimpleGlyphRenderShadowSourceSample
    order_error Option GuiSfntSimpleGlyphRenderShadowSourceCompositionOrderStartErrorKind
```

`order_error` is `Some` only when the first F5lp `composition_order_owner_invariants` precheck fails. Storage invariant failures, sample read failures, command conversion failures, and progress failures keep `order_error = None`.

The step order is:

```text
revalidate F5lp composition order owner invariant
revalidate F5lq shadow storage invariant
reject cell_index > cell_count
complete when cell_index == cell_count
read sample by borrowing the cursor
convert sample to command
move owner into the next cursor only after conversion succeeds
```

F5lr deliberately does not call F5lq `sample_cursor_step`; doing so before command conversion would advance ownership before conversion succeeds and could create a partial completion state. The command terminal uses an owner-bearing payload struct containing `RenderCommand` and the next cursor. That payload, the cursor error, and the terminal are not `Clone` / `Copy`.

The `FillRect` emitted by F5lr is only the shadow contribution command for one sample. It does not carry `shadow_order` / `source_order`, and it does not prove final composition with the source paint. Final ordering, resource transport, and compositor integration remain later boundaries.

F5lr may call `render_command_fill_rect`, `gui_paint_color`, `gui_paint_solid`, and `rgba8888_new`. It must not call fill/stroke owners, F5lg/F5lh composition helpers, resource table or reservation helpers, software surfaces, platform/backend APIs, font fallback, shadow rasterizers, compositor APIs, or owner-bearing Vec allocation/push helpers.

## SFNT simple glyph render shadow source resource reservation boundary

F5ls is the resource reservation counterpart to F5lr's per-sample correctness bridge. F5lp is the direct authority. The input is a completed `GuiSfntSimpleGlyphRenderShadowSourceCompositionOrderOwner`, not an F5lq cursor and not an F5lr command cursor. This keeps the high-throughput alpha-mask resource path separate from the diagnostic 1x1 `FillRect` bridge.

The reservation owner is private to `alloc/gui/font/sfnt/glyf`:

```text
GuiSfntSimpleGlyphRenderShadowSourceResourceReservationOwner:
    owner GuiSfntSimpleGlyphRenderShadowSourceCompositionOrderOwner
    mask_id AlphaMaskId
    rect GuiRect
    paint GuiPaint
    shadow_order i32
    source_order i32
```

The owner keeps the F5lp composition owner alive, and therefore keeps the packed shadow alpha storage alive. The `AlphaMaskId` is only checked as a nonzero value and kept with the owner. It is not registered, unique, uploaded, host-visible, or renderable. Later resource-table and prepared-command boundaries must consume this owner before creating a persistent `RenderCommand::AlphaMaskRect`, otherwise a copyable command could outlive the storage owner.

The value-only config is:

```text
GuiSfntSimpleGlyphRenderShadowSourceResourceReservationConfig:
    mask_id AlphaMaskId
```

The config may implement `Clone` / `Copy`. The success owner and start error own F5lp authority and must not implement `Clone` / `Copy`.

Validation order is stable:

```text
validate config mask id
    AlphaMaskId.raw <= 0 -> InvalidMaskId

validate F5lp composition order owner
    any lower F5lp invariant failure -> CompositionOrderInvariantFailed with order_error

validate shadow shape
    width_px <= 0 -> ShadowShapeInvalidWidth
    height_px <= 0 -> ShadowShapeInvalidHeight
    sample_scale <= 0 -> ShadowShapeInvalidSampleScale
    sample_scale * sample_scale overflow or mismatch -> ShadowShapeCoverageMaxMismatch
    width_px * height_px overflow or mismatch -> ShadowShapeCellCountMismatch

validate shadow alpha storage
    alpha_max <= 0 -> InvalidAlphaMax
    owner.cell_count != shape.cell_count -> ShadowAlphaCellCountMismatch
    alpha_cells.len != shape.cell_count -> ShadowAlphaStorageLenMismatch
    alpha_cells.cap != shape.cell_count -> ShadowAlphaStorageCapacityMismatch

validate blend
    SourceOver -> Ok
    other -> UnsupportedBlendMode

derive rect
    x = checked_i32 source_placement_origin.x - shadow_extent
    y = checked_i32 source_placement_origin.y - shadow_extent
```

The rect is `source_placement_origin - shadow_extent` for the top-left corner, with width and height copied from the validated shadow shape. Right/bottom overflow is not checked here because no command is emitted and no target extent is known yet; a later table / drain / compositor boundary must validate the record against the destination contract. F5ls copies `shadow_paint`, `shadow_order`, and `source_order` into the reservation owner so later boundaries can preserve the F5lp shadow-before-source ordering without reinterpreting command order.

F5ls uses its own resource-reservation error vocabulary for shadow storage invariant failures. It may share the same algorithm as F5lq, but it must not expose sample cursor errors, create a cursor, call `read`, call F5lq `step`, or call F5lr command helpers. `order_error` is `Some` only for the F5lp composition invariant precheck. Mask id errors, storage errors, blend errors, and rect overflow keep `order_error = None`.

Recovery is explicit. A start error keeps the original F5lp owner and config. A consuming success recovery helper returns the original `GuiSfntSimpleGlyphRenderShadowSourceCompositionOrderOwner`. Free helpers close the F5lp owner exactly once.

F5ls must not call `render_command_alpha_mask_rect`, `render_command_fill_rect`, F5lq cursor start/read/step, F5lr command helpers, resource table registration, DrawTarget, RenderTarget, platform APIs, host APIs, backend APIs, Canvas, DOM, minifb, font fallback, zero-fill fallback, shadow rasterizers, software surfaces, alpha Vec copy helpers, owner-bearing Vec payload storage, or a 2D compositor drain.

## SFNT simple glyph render shadow source resource table boundary

F5lt is the metadata table counterpart to F5ls. It consumes a `GuiSfntSimpleGlyphRenderShadowSourceResourceReservationOwner`, derives a Copy metadata record from the F5lp authority kept inside that reservation, and inserts only that record into a private table. The table is still internal to `alloc/gui/font/sfnt/glyf`; it does not upload storage, expose a backend texture, or prove renderability.

The table record is value-only:

```text
GuiSfntSimpleGlyphRenderShadowSourceResourceRecord:
    mask_id AlphaMaskId
    rect GuiRect
    paint GuiPaint
    width_px i32
    height_px i32
    cell_count i32
    alpha_max i32
    shadow_order i32
    source_order i32
```

The table owner stores `Vec GuiSfntSimpleGlyphRenderShadowSourceResourceRecord`. It must not store the reservation owner, the registered resource owner, the F5lp owner, or any alpha cell payload in the Vec. The registered resource owner keeps the original reservation and the stored record together. A successful registration returns a `GuiSfntSimpleGlyphRenderShadowSourceResourceTableRegistrationOwner` containing the updated table and the registered resource owner. No partial registration is allowed.

Registration revalidates the reservation before `vec::push`:

```text
AlphaMaskId.raw <= 0 -> InvalidMaskId
F5lp composition-order invariant failure -> CompositionOrderInvariantFailed with order_error = Some(lower kind)
shadow shape/storage failures -> mapped F5ls storage error, order_error = None
blend != SourceOver -> UnsupportedBlendMode
rect derivation overflow -> RectXOverflow / RectYOverflow
reservation rect != derived rect -> RectMetadataMismatch
reservation paint != F5lp shadow paint -> PaintMetadataMismatch
reservation shadow_order != F5lp shadow_order -> ShadowOrderMetadataMismatch
reservation source_order != F5lp source_order -> SourceOrderMetadataMismatch
existing table entry with same raw AlphaMaskId -> DuplicateMaskId
Vec push failure -> TablePushFailed with storage_error = Some(error)
```

The lower F5lp order error is intentionally preserved in the owner-bearing register error. Without that evidence, a later table registration failure would hide whether the root cause was a packed mask invariant, edge invariant, source placement, paint, blend, or order metadata mismatch.

Success and failure recovery use paired continuations. Success passes the updated table owner and registered resource owner together. Error recovery passes the table owner and reservation owner together through a rejected owner. F5lt does not expose split consuming accessors that return only the table or only the reservation.

F5lt must not call `render_command_alpha_mask_rect`, `render_command_fill_rect`, F5lq cursor start/read/step, F5lr command helpers, DrawTarget, RenderTarget, platform APIs, host APIs, backend APIs, Canvas, DOM, minifb, font fallback, zero-fill fallback, shadow rasterizers, software surfaces, alpha Vec copy helpers, owner-bearing Vec payload storage, or a 2D compositor drain.

## SFNT simple glyph render shadow source prepared command boundary

F5lu consumes the F5lt registered resource owner and prepares the `RenderCommand::AlphaMaskRect` value that will later be consumed by a formal transport or compositor drain owner. This is not command stream emission. The command is a Copy value, so exposing it through an accessor or arbitrary callback would allow callers to keep the command after dropping the registered resource owner. That would reintroduce a dangling `AlphaMaskId` command.

The prepared owner is private:

```text
GuiSfntSimpleGlyphRenderShadowSourceResourcePreparedCommandOwner:
    resource GuiSfntSimpleGlyphRenderShadowSourceRegisteredResourceOwner
    command RenderCommand
```

It exposes metadata through the stored record only. It must not expose the raw `RenderCommand`, a borrowed command, or a callback that receives the command.

Preparation is a revalidation boundary:

```text
read stored record from registered resource
reject AlphaMaskId.raw <= 0 before command construction
borrow the internal F5ls reservation
derive expected record through F5lt record_from_reservation
map F5lt/F5ls/F5lp failures to F5lu prepared-command errors
preserve lower F5lp order_error when record derivation fails at CompositionOrderInvariantFailed
compare stored and expected record fields including shadow_order and source_order
call render_command_alpha_mask_rect only after record equality succeeds
store the command inside the prepared owner
```

The prepared error owns the registered resource owner and preserves optional lower F5lp order evidence:

```text
GuiSfntSimpleGlyphRenderShadowSourceResourcePreparedCommandError:
    kind GuiSfntSimpleGlyphRenderShadowSourceResourcePreparedCommandErrorKind
    resource GuiSfntSimpleGlyphRenderShadowSourceRegisteredResourceOwner
    order_error Option GuiSfntSimpleGlyphRenderShadowSourceCompositionOrderStartErrorKind
```

`DuplicateMaskId` and `TablePushFailed` are table-mutation states and should not arise during prepare; if they are observed through shared mapping code, F5lu reports `UnexpectedTableRegisterState`. Error recovery returns the registered resource owner only. No command exists on the error path.

F5lu must not call `render_command_fill_rect`, F5lq cursor start/read/step, F5lr command helpers, resource table lookup/register/push, DrawTarget, RenderTarget, platform APIs, host APIs, backend APIs, Canvas, DOM, minifb, font fallback, zero-fill fallback, shadow rasterizers, software surfaces, alpha Vec copy helpers, owner-bearing Vec payload storage, tile / bitmap transport, or a 2D compositor drain. It may call `render_command_alpha_mask_rect` only in the validated success path and only to store the command inside the prepared owner.

## SFNT simple glyph render shadow source software drain-start boundary

F5lv consumes the F5lu prepared command owner and a shared render2d `GuiRgba8888SoftwareSurfaceOwner` together. It creates a cursor owner for the later bounded shadow SourceOver drain step. This is only a drain-start boundary: it does not write pixels, read pixels, run SourceOver, create dirty metadata, publish a tile or bitmap payload, call a host present API, or drain a compositor.

The owner is private:

```text
GuiSfntSimpleGlyphRenderShadowSourceSoftwareDrainOwner:
    prepared GuiSfntSimpleGlyphRenderShadowSourceResourcePreparedCommandOwner
    surface GuiRgba8888SoftwareSurfaceOwner
    cell_index i32
```

The start error owns both input owners and keeps lower F5lp order evidence:

```text
GuiSfntSimpleGlyphRenderShadowSourceSoftwareDrainStartError:
    kind GuiSfntSimpleGlyphRenderShadowSourceSoftwareDrainErrorKind
    prepared GuiSfntSimpleGlyphRenderShadowSourceResourcePreparedCommandOwner
    surface GuiRgba8888SoftwareSurfaceOwner
    order_error Option GuiSfntSimpleGlyphRenderShadowSourceCompositionOrderStartErrorKind
```

The validation helper uses value-level errors before the consuming start function builds the owner-bearing start error. That keeps the prepared owner and surface owner unmoved until all checks have completed. The validation order is fixed:

```text
borrow prepared and surface
read stored record from prepared.resource
borrow prepared.resource.reservation
derive expected record through F5lt record_from_reservation
map lower record errors to F5lv errors and preserve lower order_error
compare stored and expected record through F5lu record equality
validate record width, height, cell_count, and alpha_max
inspect the private command field only inside command payload validation
accept only RenderCommand::AlphaMaskRect
compare command mask id, rect, and paint to the rederived record
validate rect origin, positive size, checked right/bottom extents, and surface containment
return cell_index 0 owner only after success
```

The command field carveout is intentionally narrow. F5lv may borrow `prepared.command` only in the private command payload validator. It still must not expose the command, return it, pass it to arbitrary callbacks, or treat command validation as command stream emission.

Recovery is paired. Start error recovery returns a rejected owner containing both prepared and surface. `rejected_with` passes both owners to one callback. There are no consuming split accessors for only the prepared owner or only the surface owner. Free helpers close the prepared side first and then free the software surface; surface free failure maps to `SurfaceFreeFailed`.

F5lv must not call `gui_rgba8888_software_surface_write_pixel`, `gui_rgba8888_software_surface_read_pixel`, `gui_rgba8888_source_over_alpha_mask`, `render_command_fill_rect`, F5lq cursor start/read/step, F5lr command helpers, resource table lookup/register/push, DrawTarget, RenderTarget, platform APIs, host APIs, backend APIs, Canvas, DOM, minifb, font fallback, zero-fill fallback, dirty-region helpers, tile / bitmap transport, or a 2D compositor drain.

## SFNT simple glyph render shadow source software drain-step boundary

F5lw consumes the F5lv cursor owner as a bounded software compositing step. Its one-cell step mutates the owned RGBA8888 software surface through the checked render2d surface API and the shared `gui_rgba8888_source_over_alpha_mask` helper. The one-cell step does not create dirty metadata, publish pixels, call host present, or drain a 2D compositor.

The new owner-bearing values are:

```text
GuiSfntSimpleGlyphRenderShadowSourceSoftwareDrainCompletedOwner:
    prepared GuiSfntSimpleGlyphRenderShadowSourceResourcePreparedCommandOwner
    surface GuiRgba8888SoftwareSurfaceOwner
    dirty DirtyRegion

GuiSfntSimpleGlyphRenderShadowSourceSoftwareDrainTerminal:
    Completed GuiSfntSimpleGlyphRenderShadowSourceSoftwareDrainCompletedOwner
    StepBudgetExhausted GuiSfntSimpleGlyphRenderShadowSourceSoftwareDrainOwner

GuiSfntSimpleGlyphRenderShadowSourceSoftwareDrainStepError:
    kind GuiSfntSimpleGlyphRenderShadowSourceSoftwareDrainErrorKind
    owner GuiSfntSimpleGlyphRenderShadowSourceSoftwareDrainOwner
    order_error Option GuiSfntSimpleGlyphRenderShadowSourceCompositionOrderStartErrorKind
```

The step order is fixed:

```text
borrow prepared and surface from owner
run F5lv validate_start
preserve lower F5lp order_error on validation failure
rederive the resource record
reject negative or out-of-range cell_index
borrow prepared.resource.reservation.owner.shadow_owner.alpha_cells
read one alpha cell without Vec clone/copy
map cell_index to record rect position with checked addition
read destination pixel from the borrowed surface
derive source color from record paint
run gui_rgba8888_source_over_alpha_mask
move prepared/surface only before write
on write failure recover the returned surface and keep cell_index unchanged
on write success rebuild the owner with cell_index + 1
```

`to_complete_budget` checks completion before budget validity. A cursor already at `cell_count` completes even with zero budget. A non-completed cursor with `remaining_steps <= 0` fails with `InvalidBudget`. `StepBudgetExhausted` is returned only after at least one successful step and only if the next owner has not reached completion. The progress invariant requires the step to advance by exactly one cell.

The completed owner includes `DirtyRegion` only after F5lx completion. F5lw still only proves that the shadow mask was composited into the software surface. The dirty-region phase derives the record rect through the same prepared/resource authority and attaches checked dirty metadata without changing the one-cell SourceOver step contract.

F5lw one-cell step may call `gui_rgba8888_software_surface_read_pixel`, `gui_rgba8888_software_surface_write_pixel`, and `gui_rgba8888_source_over_alpha_mask`. It must not call F5lq cursor start/read/step, F5lr command helpers, `render_command_fill_rect`, raw `RenderCommand` accessors, resource table lookup/register/push, dirty-region helpers, DrawTarget, RenderTarget, platform APIs, host APIs, backend APIs, Canvas, DOM, minifb, font fallback, zero-fill fallback, tile / bitmap transport, or a 2D compositor drain.

## SFNT simple glyph render shadow source software drain dirty-region completion boundary

F5lx attaches a `DirtyRegion` value to the F5lw completed owner. This mirrors F5br for the shadow source path, but keeps aggregation and transport deferred. It does not create a generic render2d `surface + dirty` owner, does not push into a `DirtyRegionSet`, and does not publish a bitmap or tile payload.

The completed owner shape is:

```text
GuiSfntSimpleGlyphRenderShadowSourceSoftwareDrainCompletedOwner:
    prepared GuiSfntSimpleGlyphRenderShadowSourceResourcePreparedCommandOwner
    surface GuiRgba8888SoftwareSurfaceOwner
    dirty DirtyRegion
```

The owner remains non-Clone and non-Copy. `dirty` is Copy metadata and may be returned by a borrowed accessor. `prepared` and `surface` remain inaccessible as split accessors. The only way to take the surface is still the consuming finish helper that frees the prepared/resource side first. Callers that need both dirty metadata and surface ownership must read the dirty value before calling the finish helper.

The dirty value is created from the rederived shadow resource record rect through `dirty_region_rect_checked`. This is not a fallback. It gives `core/gui/dirty_region` authority over the dirty metadata contract. Even though F5lv/F5lw validation already checked rect geometry and surface containment, the dirty value is still constructed through the checked dirty-region constructor. Failure is mapped to `DirtyRegionInvalid` and returned as an owner-bearing step error with the original owner intact. Lower F5lp `order_error` evidence remains limited to validation and record rederive failures.

The completion branch order is fixed:

```text
validate existing prepared/surface pair
rederive resource record
read cell_index
if cell_index == cell_count:
    dirty = dirty_region_rect_checked record.rect
    if dirty fails:
        return owner-bearing DirtyRegionInvalid
    move prepared and surface out of owner
    return Completed prepared surface dirty
```

This order avoids losing the prepared/surface owners on dirty construction failure. F5lx must not call Web/native host APIs, video-memory publish helpers, tile or bitmap transport helpers, DrawTarget, RenderTarget, Canvas, DOM, minifb, the old F5lq/F5lr sample bridge, raw `RenderCommand` accessors, fallback paths, unchecked dirty-region helpers, or a 2D compositor drain.

## SFNT simple glyph render software drain dirty-owner bridge boundary

F5ly is the bridge between font-owned completed software drains and the render2d dirty surface owner. It is intentionally not the compositor drain itself. Fill F5br and shadow F5lx both end with a completed owner that still owns the prepared/resource side, the RGBA8888 software surface, and one `DirtyRegion`. Render2d F5bt already defines the shared `GuiRgba8888SoftwareSurfaceDirtyOwner` that later bitmap / row / tile boundaries consume. F5ly connects those two contracts without entering F5bu or host transport.

The bridge order is:

```text
dirty = completed_owner_dirty &completed
next_dirty = dirty_regions_push_region_checked dirty_regions_empty dirty
if next_dirty fails:
    return owner-bearing bridge error with original completed owner
surface = completed_owner_finish_surface completed
return GuiRgba8888SoftwareSurfaceDirtyOwner surface next_dirty
```

The key point is that dirty aggregation happens before `finish_surface`. A dirty-set failure therefore leaves the completed owner intact, including prepared and surface ownership. On success, `finish_surface` frees the prepared/resource side and returns only the surface owner, which is then packed with the already-checked `DirtyRegionSet`.

Fill and shadow have separate bridge error types and bridge functions because their completed owner and free error kinds differ. The policy is otherwise identical. The bridge error is owner-bearing and must not implement `Clone` / `Copy`; its free helper delegates to the completed-owner free path so that surface free failure remains visible. F5ly must not call bitmap frame prepare, row byte storage, tile or RLE transport, host present, video-memory publish helpers, DrawTarget, RenderTarget, Canvas, DOM, minifb, platform APIs, fallback paths, `dirty_regions_push_unchecked`, or `dirty_region_merge`.

## Render2d compositor frame entry boundary

F5lz is the first render2d compositor entry after F5ly. It consumes a `GuiRgba8888SoftwareSurfaceDirtyOwner` and a small config, then connects the already-existing pre-transport boundaries in a fixed order:

```text
frame_config = GuiRgba8888BitmapFrameConfig frame_id
frame = gui_rgba8888_bitmap_frame_prepare dirty_owner frame_config
plan_config = GuiRgba8888RowBatchPlanConfig max_rows_per_batch
plan = gui_rgba8888_row_batch_plan_prepare frame plan_config
metadata = copy plan frame id / shape / row span / batch count
cursor = gui_rgba8888_row_batch_cursor_start plan
return GuiRgba8888CompositorFrameEntryOwner cursor metadata
```

The config is intentionally not validated through ownerless `*_config_checked` calls inside `prepare`. Invalid `frame_id` or `max_rows_per_batch` must still return an owner-bearing error, so F5lz passes aggregate config values to lower prepare functions and wraps their errors. `BitmapFramePrepareFailed`, `RowBatchPlanPrepareFailed`, and `RowBatchCursorStartFailed` preserve the stage-specific lower kind, then normalize the recoverable owner back to `GuiRgba8888SoftwareSurfaceDirtyOwner`. Row plan and cursor-start failures recover through `gui_rgba8888_bitmap_frame_finish_dirty_owner` / `gui_rgba8888_row_batch_plan_finish_dirty_owner`, so dirty metadata is read before the frame/plan owner is consumed. Each wrapper reads lower kind and lower category before consuming the lower error to recover the owner.

The metadata copy is taken before `gui_rgba8888_row_batch_cursor_start` because the plan owner moves into the cursor on success. The metadata is only a summary for scheduling and diagnostics; it is not a row byte payload or transport descriptor.

F5lz is not row batch drain, row range, row byte storage, tile payload, RLE transport, std present, host import, platform backend, video memory, Canvas, DOM, minifb, fallback, or silent no-op behavior.

## Render2d compositor batch drain boundary

F5ma is the first scheduler continuation above the F5lz entry owner. It consumes `GuiRgba8888CompositorFrameEntryOwner` and a caller supplied batch budget, but it does not inspect row batch cursor status or call `next_batch` itself. The entire progress decision remains inside F5bx:

```text
metadata = copy entry metadata
cursor = gui_rgba8888_compositor_frame_entry_finish_cursor entry
terminal_or_error = gui_rgba8888_row_batch_drain_budget cursor remaining_batches
wrap lower terminal/error with metadata
```

This boundary exists because later compositor drain phases need the F5lz frame metadata even after the row batch cursor has moved through scheduler slices. `GuiRgba8888CompositorBatchDrainTerminal` keeps the mapped compositor status, the lower `GuiRgba8888RowBatchDrainTerminal`, and the copied metadata. `GuiRgba8888CompositorBatchDrainError` keeps `RowBatchDrainFailed lower_kind`, the lower category, the lower owner-bearing error, and the copied metadata. Both are owner-bearing and must not implement `Clone` / `Copy`.

The wrapper must copy metadata before `finish_cursor`. `terminal_finish_entry` and `error_finish_entry` must also copy wrapper metadata before consuming the lower terminal/error. This preserves a recoverable `GuiRgba8888CompositorFrameEntryOwner` for both `StepBudgetExhausted` continuation and owner-bearing error recovery. Empty dirty completion with a negative budget and ready-cursor negative budget remain lower row batch drain semantics; F5ma only preserves the compositor metadata and maps the lower status/kind.

F5ma must call `gui_rgba8888_row_batch_drain_budget` exactly once. It must not call `gui_rgba8888_row_batch_cursor_status`, `gui_rgba8888_row_batch_cursor_next_batch`, row batch range, row byte storage, row tile plan/payload, RLE encode, std present, host import, platform backend, video memory, Canvas, DOM, minifb, fallback, or silent no-op behavior.

## Render2d compositor batch range boundary

F5mb is the first payload metadata bridge above the F5lz entry owner. It consumes `GuiRgba8888CompositorFrameEntryOwner`, copies the entry metadata, takes the lower cursor, asks F5bw for exactly one batch, and then asks F5by to prepare the row range metadata:

```text
metadata = copy entry metadata
cursor = gui_rgba8888_compositor_frame_entry_finish_cursor entry
batch_or_error = gui_rgba8888_row_batch_cursor_next_batch cursor
range_or_error = gui_rgba8888_row_batch_range_prepare batch
wrap lower range owner with metadata
```

`GuiRgba8888CompositorBatchRangeOwner` keeps the lower `GuiRgba8888RowBatchRangeOwner` and copied `GuiRgba8888CompositorFrameEntryMetadata`. It exposes Copy range metadata through compositor-owned accessors so callers do not need to consume the lower owner merely to inspect frame/batch row range data. `owner_finish_entry` copies wrapper metadata before consuming the lower range owner and reconstructs `GuiRgba8888CompositorFrameEntryOwner` from the continuation cursor.

`GuiRgba8888CompositorBatchRangeError` does not keep heterogeneous lower owner-bearing errors. Error construction reads lower kind and category first, then normalizes the lower cursor or lower batch owner back into `GuiRgba8888CompositorFrameEntryOwner`. Cursor next-batch failure maps to `CursorNextBatchFailed lower_kind`; row range prepare failure maps to `RowBatchRangePrepareFailed lower_kind`. Complete cursor remains a lower next-batch error, usually `CursorIndexPastEnd`, so this bridge does not introduce a second completion terminal.

F5mb must call `gui_rgba8888_row_batch_cursor_next_batch` exactly once and `gui_rgba8888_row_batch_range_prepare` exactly once. It must not call `gui_rgba8888_row_batch_cursor_status`, row byte storage, row tile plan/payload, RLE encode, std present, host import, platform backend, video memory, Canvas, DOM, minifb, fallback, or silent no-op behavior.

## Render2d compositor byte storage boundary

F5mc is the compositor-side bridge from F5mb range metadata to F5bz copied byte storage. It consumes `GuiRgba8888CompositorBatchRangeOwner`, copies the compositor metadata first, extracts the lower `GuiRgba8888RowBatchRangeOwner`, and then calls `gui_rgba8888_row_byte_storage_prepare` exactly once. The result is `GuiRgba8888CompositorByteStorageOwner`, which keeps the lower `GuiRgba8888RowByteStorageOwner` and copied `GuiRgba8888CompositorFrameEntryMetadata`.

```text
metadata = copy range owner metadata
lower_range = finish compositor range owner to lower range owner
storage_or_error = gui_rgba8888_row_byte_storage_prepare lower_range
wrap lower byte storage owner with metadata
```

Prepare errors are normalized back to the compositor range owner boundary. `GuiRgba8888CompositorByteStoragePrepareError` reads the lower row byte storage prepare kind and category before consuming the lower error owner, then reconstructs `GuiRgba8888CompositorBatchRangeOwner` from the lower range owner plus copied metadata. It stores `kind/category/range`, not the lower owner-bearing error.

Finish errors are normalized back to the compositor entry owner boundary. `owner_finish_entry` copies metadata before consuming the lower byte storage owner. If `gui_rgba8888_row_byte_storage_finish_cursor` fails, F5mc reads the lower finish kind, consumes the lower finish error cursor, reconstructs `GuiRgba8888CompositorFrameEntryOwner`, and maps the category to `GuiError::BackendFailure`. It stores `kind/category/entry`, not the lower owner-bearing error.

`owner_free` delegates to `owner_finish_entry` and then `gui_rgba8888_compositor_frame_entry_owner_free`. It distinguishes `FinishFailed lower_finish_kind` from `EntryFreeFailed entry_free_kind`, where `entry_free_kind` is the `GuiRgba8888SoftwareSurfaceErrorKind` returned by frame entry teardown. A successful byte-storage finish followed by entry free failure is not collapsed into the storage finish path.

F5mc may expose checked byte count and checked byte read helpers by borrowing the lower copied byte storage owner. It must not expose `RegionToken`, `MemPtr`, source storage, destination raw storage, `row_byte_storage_validate_authority`, row tile plan/payload, RLE encode, std present, host import, platform backend, video memory, Canvas, DOM, minifb, fallback, or silent no-op behavior.

## Render2d compositor tile plan boundary

F5md is the compositor-side bridge from F5mc copied byte storage to F5ca row tile plan metadata. It consumes `GuiRgba8888CompositorByteStorageOwner`, copies the compositor metadata first, extracts the lower `GuiRgba8888RowByteStorageOwner`, and then calls `gui_rgba8888_row_tile_plan_prepare` exactly once. The result is `GuiRgba8888CompositorTilePlanOwner`, which keeps the lower `GuiRgba8888RowTilePlanOwner` and copied `GuiRgba8888CompositorFrameEntryMetadata`.

```text
metadata = copy byte storage owner metadata
lower_storage = finish compositor byte storage owner to lower byte storage owner
tile_plan_or_error = gui_rgba8888_row_tile_plan_prepare lower_storage config
wrap lower tile plan owner with metadata
```

Prepare errors are normalized back to the compositor byte storage boundary. `GuiRgba8888CompositorTilePlanPrepareError` reads the lower row tile plan prepare kind and category before consuming the lower error owner, then reconstructs `GuiRgba8888CompositorByteStorageOwner` from the lower byte storage owner plus copied metadata. It stores `kind/category/storage`, not the lower owner-bearing error.

Descriptor access remains metadata-only. `gui_rgba8888_compositor_tile_plan_owner_descriptor_at` borrows the lower row tile plan owner and delegates to `gui_rgba8888_row_tile_plan_descriptor_at`, so F5ca remains the owner of invariant validation and storage-relative descriptor calculation. F5md does not call `gui_rgba8888_row_byte_storage_validate_authority` directly; direct validation here would duplicate the lower boundary and make recovery ordering harder to reason about.

Finish errors are normalized back to the compositor entry owner boundary. `owner_finish_byte_storage` copies metadata before consuming the lower tile plan owner and then wraps the recovered `GuiRgba8888RowByteStorageOwner` as `GuiRgba8888CompositorByteStorageOwner`. `owner_finish_entry` delegates to F5mc `gui_rgba8888_compositor_byte_storage_owner_finish_entry`; if that returns a byte-storage finish error, F5md reads the lower finish kind and category before taking the recovered entry owner, and stores `kind/category/entry`.

`owner_free` delegates to `owner_finish_entry` and then `gui_rgba8888_compositor_frame_entry_owner_free`. It distinguishes `FinishFailed lower_finish_kind` from `EntryFreeFailed entry_free_kind`, where `entry_free_kind` is the `GuiRgba8888SoftwareSurfaceErrorKind` returned by frame entry teardown. A successful tile-plan finish followed by entry free failure is not collapsed into the byte-storage finish path.

F5md must not expose row tile plan storage refs, checked byte readers, `RegionToken`, `MemPtr`, source storage, destination raw storage, tile payload views, RLE encode, std present, host import, platform backend, video memory, Canvas, DOM, minifb, fallback, or silent no-op behavior. It is a no RLE / host present compositor tile plan bridge; payload transport and std present remain later compositor continuation boundaries.

## Render2d compositor tile payload boundary

F5me is the compositor-side bridge from F5md tile plan metadata to F5cb checked row tile payload view. It consumes `GuiRgba8888CompositorTilePlanOwner`, copies the compositor metadata first, extracts the lower `GuiRgba8888RowTilePlanOwner`, and then calls `gui_rgba8888_row_tile_payload_prepare` exactly once. The result is `GuiRgba8888CompositorTilePayloadOwner`, which keeps the lower `GuiRgba8888RowTilePayloadOwner` and copied `GuiRgba8888CompositorFrameEntryMetadata`.

```text
metadata = copy tile plan owner metadata
lower_plan = move lower row tile plan owner
payload_or_error = gui_rgba8888_row_tile_payload_prepare lower_plan tile_index
wrap lower payload owner with metadata
```

Prepare errors are normalized back to the compositor tile plan boundary. `GuiRgba8888CompositorTilePayloadPrepareError` reads the lower row tile payload prepare kind and category before consuming the lower error owner, then reconstructs `GuiRgba8888CompositorTilePlanOwner` from the lower row tile plan owner plus copied metadata. It stores `kind/category/plan`, not the lower owner-bearing error.

Descriptor and plan metadata access stay checked. `gui_rgba8888_compositor_tile_payload_owner_descriptor_checked` delegates to `gui_rgba8888_row_tile_payload_descriptor_checked`, and `gui_rgba8888_compositor_tile_payload_owner_plan_metadata_checked` delegates to `gui_rgba8888_row_tile_payload_plan_metadata_checked`. The compositor boundary never exposes the lower row tile plan storage ref or row byte storage accessor. `gui_rgba8888_compositor_tile_payload_owner_byte_at` is a tile-relative byte reader that delegates to lower `gui_rgba8888_row_tile_payload_byte_at`.

Finish errors are normalized back to the compositor entry owner boundary. `owner_finish_tile_plan` copies metadata before consuming the lower payload owner and then wraps the recovered `GuiRgba8888RowTilePlanOwner` as `GuiRgba8888CompositorTilePlanOwner`. `owner_finish_entry` delegates to F5md `gui_rgba8888_compositor_tile_plan_owner_finish_entry`; if that returns a tile-plan finish error, F5me reads the lower finish kind and category before taking the recovered entry owner, and stores `kind/category/entry`.

`owner_free` delegates to `owner_finish_entry` and then `gui_rgba8888_compositor_frame_entry_owner_free`. It distinguishes `FinishFailed lower_finish_kind` from `EntryFreeFailed entry_free_kind`, where `entry_free_kind` is the `GuiRgba8888SoftwareSurfaceErrorKind` returned by frame entry teardown. A successful tile-payload finish followed by entry free failure is not collapsed into the tile-plan finish path.

F5me must not expose row tile plan storage refs, row byte storage accessors, `RegionToken`, `MemPtr`, source storage, destination raw storage, RLE encode, std present, host import, host present, platform backend, video memory, Canvas, DOM, minifb, fallback, or silent no-op behavior. It is a no RLE / host present compositor tile payload bridge; payload transport and std present remain later compositor continuation boundaries.

## Render2d compositor tile RLE count boundary

F5mf is the compositor-side bridge from F5me tile payload ownership to the lower row tile RLE count start owner. It consumes `GuiRgba8888CompositorTilePayloadOwner`, copies compositor metadata first, extracts the lower `GuiRgba8888RowTilePayloadOwner`, calls `gui_rgba8888_row_tile_rle_cursor_start` exactly once, and then calls `gui_rgba8888_row_tile_rle_count_start` exactly once. The result is `GuiRgba8888CompositorTileRleCountOwner`, which keeps the lower `GuiRgba8888RowTileRleCountOwner` and copied `GuiRgba8888CompositorFrameEntryMetadata`.

```text
metadata = copy tile payload owner metadata
lower_payload = move lower row tile payload owner
cursor_or_error = gui_rgba8888_row_tile_rle_cursor_start lower_payload
count_or_error = gui_rgba8888_row_tile_rle_count_start cursor
wrap lower count owner with metadata
```

Start errors are normalized back to the compositor tile payload boundary. For cursor-start failure, `GuiRgba8888CompositorTileRleCountStartError` reads the lower cursor-start kind and category before taking the lower row tile payload owner, then reconstructs `GuiRgba8888CompositorTilePayloadOwner` from the payload owner plus copied metadata. For count-start failure, it reads the lower count-start kind and category before taking the lower cursor owner, finishes that cursor back to payload, and reconstructs the same metadata-bearing payload owner. The start error stores `kind/category/payload`, not the lower owner-bearing error.

Accessor methods remain lower-count views. `gui_rgba8888_compositor_tile_rle_count_owner_accumulated_run_count`, `gui_rgba8888_compositor_tile_rle_count_owner_cursor_next_pixel_index`, and `gui_rgba8888_compositor_tile_rle_count_owner_cursor_status` borrow the lower `GuiRgba8888RowTileRleCountOwner` and delegate to its checked helpers. F5mf does not call drain or count step to manufacture count evidence.

Finish errors are normalized back to the compositor entry owner boundary. `owner_finish_payload` copies metadata before consuming the lower count owner, then finishes lower count owner to cursor and lower cursor owner to payload before wrapping `GuiRgba8888CompositorTilePayloadOwner`. `owner_finish_entry` delegates to F5me `gui_rgba8888_compositor_tile_payload_owner_finish_entry`; if that returns a payload finish error, F5mf reads the lower finish kind and category before taking the recovered entry owner, and stores `kind/category/entry`.

`owner_free` delegates to `owner_finish_entry` and then `gui_rgba8888_compositor_frame_entry_owner_free`. It distinguishes `FinishFailed lower_finish_kind` from `EntryFreeFailed entry_free_kind`, where `entry_free_kind` is the `GuiRgba8888SoftwareSurfaceErrorKind` returned by frame entry teardown. A successful RLE count finish followed by entry free failure is not collapsed into the payload finish path.

F5mf must not expose row tile RLE drain, count step, completed count, encode cursor, writer plan, encoded storage, packet, tile payload direct byte reader, row byte storage accessors, `RegionToken`, `MemPtr`, source storage, destination raw storage, std present, host import, host present, platform backend, video memory, Canvas, DOM, minifb, fallback, or silent no-op behavior. It is a no drain / encode / present compositor tile RLE count bridge; payload transport and std present remain later compositor continuation boundaries.

## Render2d compositor tile RLE count step boundary

F5mg is the compositor-side count continuation after F5mf. It consumes `GuiRgba8888CompositorTileRleCountOwner`, copies compositor metadata first, extracts the lower `GuiRgba8888RowTileRleCountOwner`, and calls `gui_rgba8888_row_tile_rle_count_step_budget` exactly once. It returns `GuiRgba8888CompositorTileRleCountStep`, which stores the lower `GuiRgba8888RowTileRleCountStepStatus` and the next `GuiRgba8888CompositorTileRleCountOwner`.

```text
metadata = copy compositor count owner metadata
lower_count = move lower row tile RLE count owner
step_or_error = gui_rgba8888_row_tile_rle_count_step_budget lower_count remaining_steps
success = copy lower step status, move lower next count owner, wrap it with metadata
error = copy lower kind/category/progress, move cursor back to payload, wrap it with metadata
```

The error path deliberately does not recreate a count owner. The lower count boundary documents that some failures may have advanced the cursor while only preserving the prior accumulated count. F5mg therefore treats lower count errors as fatal to the current count continuation: it reads `kind`, `category`, `accumulated_run_count`, and `cursor_next_pixel_index`, then finishes the lower cursor back to the row tile payload and reconstructs `GuiRgba8888CompositorTilePayloadOwner`. This avoids a fake continuation while preserving recovery authority for free, finish, or restart decisions.

Success finish and free helpers delegate to F5mf `gui_rgba8888_compositor_tile_rle_count_owner_finish_entry` and `gui_rgba8888_compositor_tile_rle_count_owner_free`. Error finish and free helpers delegate to F5me `gui_rgba8888_compositor_tile_payload_owner_finish_entry` and `gui_rgba8888_compositor_tile_payload_owner_free`, because an error has only payload recovery authority, not a count owner.

F5mg must not expose direct row tile RLE drain, completed count, encode cursor, writer plan, encoded storage, packet, tile payload direct byte reader, row byte storage accessors, `RegionToken`, `MemPtr`, source storage, destination raw storage, std present, host import, host present, platform backend, video memory, Canvas, DOM, minifb, fallback, or silent no-op behavior. It is a no completed / encode / present compositor tile RLE count step bridge; completed count evidence and encoded transport remain later boundaries.

## Render2d compositor tile RLE completed count boundary

F5mh is the compositor-side completed evidence bridge after F5mg. It consumes `GuiRgba8888CompositorTileRleCountOwner`, copies compositor metadata first, extracts the lower `GuiRgba8888RowTileRleCountOwner`, and calls `gui_rgba8888_row_tile_rle_count_completed_prepare` exactly once. It returns `GuiRgba8888CompositorTileRleCountCompletedOwner`, which stores the lower completed owner plus metadata as exact capacity evidence for the later encoded transport.

```text
metadata = copy compositor count owner metadata
lower_count = move lower row tile RLE count owner
completed_or_error = gui_rgba8888_row_tile_rle_count_completed_prepare lower_count
success = wrap lower completed owner with metadata
error = copy lower kind/category/total/index, move original lower count owner, wrap it with metadata
```

The error path differs from F5mg. The lower completed error still owns the original count owner, so F5mh must not fall back to payload recovery or create a fake owner. It reads `kind`, `category`, `total_run_count`, and `cursor_next_pixel_index` before consuming the lower error, then reconstructs `GuiRgba8888CompositorTileRleCountOwner` with the preserved metadata. This keeps pending-count recovery available for free, finish, or retry decisions.

Success finish and free helpers first recover the compositor count owner through `gui_rgba8888_row_tile_rle_count_completed_owner_finish_count_owner`, then delegate to F5mf `gui_rgba8888_compositor_tile_rle_count_owner_finish_entry` and `gui_rgba8888_compositor_tile_rle_count_owner_free`. Error finish and free helpers use the recovered count owner stored in the error and delegate to the same F5mf count owner helpers.

F5mh must not expose count step reruns, direct row tile RLE drain, encode cursor, writer plan, encoded storage, packet, tile payload direct byte reader, row byte storage accessors, `RegionToken`, `MemPtr`, source storage, destination raw storage, std present, host import, host present, platform backend, video memory, Canvas, DOM, minifb, fallback, or silent no-op behavior. It is a no count step / encode / present compositor tile RLE completed count bridge; encoded transport and present continuation remain later boundaries.

## Render2d compositor tile RLE encode seed boundary

F5mi is the compositor-side payload seed bridge after F5mh. It consumes `GuiRgba8888CompositorTileRleCountCompletedOwner`, copies compositor metadata first, extracts the lower `GuiRgba8888RowTileRleCountCompletedOwner`, and calls `gui_rgba8888_row_tile_rle_encode_seed_prepare` exactly once. It returns `GuiRgba8888CompositorTileRleEncodeSeedOwner`, which stores the lower seed owner plus metadata for the later cursor restart boundary.

```text
metadata = copy compositor completed owner metadata
lower_completed = move lower row tile RLE completed owner
seed_or_error = gui_rgba8888_row_tile_rle_encode_seed_prepare lower_completed
success = wrap lower seed owner with metadata
error = copy lower kind/category/total, move lower completed owner, wrap it with metadata
```

The boundary deliberately does not restart the cursor. Lower F5cg and F5ch are split because invalid completed evidence recovers a completed owner, while cursor restart failure recovers a payload/start-error owner. F5mi keeps the same split on the compositor side: success owns payload seed authority, and error owns completed evidence authority.

Success finish and free helpers first recover `GuiRgba8888CompositorTilePayloadOwner` and then delegate to F5me `gui_rgba8888_compositor_tile_payload_owner_finish_entry` / `gui_rgba8888_compositor_tile_payload_owner_free`. Error finish and free helpers recover `GuiRgba8888CompositorTileRleCountCompletedOwner` and delegate to F5mh completed owner finish/free. This keeps success payload recovery and error completed-evidence recovery separate.

F5mi must not expose cursor restart, count step reruns, direct row tile RLE drain, encode cursor, writer plan, encoded storage, packet, tile payload direct byte reader, row byte storage accessors, `RegionToken`, `MemPtr`, source storage, destination raw storage, std present, host import, host present, platform backend, video memory, Canvas, DOM, minifb, fallback, or silent no-op behavior. It is a no cursor restart / encode / present compositor tile RLE encode seed bridge; cursor restart, encoded transport, and present continuation remain later boundaries.

## Render2d compositor tile RLE encode cursor boundary

F5mj is the compositor-side cursor restart bridge after F5mi. It consumes `GuiRgba8888CompositorTileRleEncodeSeedOwner`, copies compositor metadata first, extracts the lower `GuiRgba8888RowTileRleEncodeSeedOwner`, and calls `gui_rgba8888_row_tile_rle_encode_cursor_start` exactly once. It returns `GuiRgba8888CompositorTileRleEncodeCursorOwner`, which stores the lower ready cursor owner plus metadata for the later writer capacity boundary.

```text
metadata = copy compositor seed owner metadata
lower_seed = move lower row tile RLE seed owner
ready_or_error = gui_rgba8888_row_tile_rle_encode_cursor_start lower_seed
success = wrap lower ready cursor owner with metadata
error = copy lower kind/category/total/start-kind, move lower start error payload, wrap it with metadata
```

The ready cursor owner exposes copied metadata, total run count, cursor next pixel index, and cursor pixel count as non-consuming accessors. It does not expose the raw lower cursor as public recovery authority. Success finish and free helpers first recover the lower cursor from the lower ready owner, then recover the lower payload with `gui_rgba8888_row_tile_rle_cursor_finish_payload`, wrap it as `GuiRgba8888CompositorTilePayloadOwner`, and delegate to F5me payload finish/free.

The error path follows Zeno's plan review correction. Lower F5ch represents cursor restart failure as a lower encode-cursor error that owns a lower start error, but F5mj must not publish that lower error or raw cursor as compositor recovery state. It reads `kind`, `category`, `total_run_count`, and `start_kind` before consuming the lower error, recovers the lower start error payload, and normalizes it to metadata-wrapped compositor payload ownership. Error finish and free therefore delegate through F5me payload finish/free.

F5mj must not expose direct row tile cursor start, count step reruns, direct row tile RLE drain, writer plan, encoded storage, encoded seal, packet, packet record reader, tile payload direct byte reader, row byte storage accessors, `RegionToken`, `MemPtr`, raw byte load/store, source storage, destination raw storage, std present, host import, host present, platform backend, video memory, Canvas, DOM, minifb, fallback, or silent no-op behavior. It is a no writer / storage / packet / present compositor tile RLE encode cursor bridge; encoded transport and present continuation remain later boundaries.

## Render2d compositor tile RLE writer plan boundary

F5mk is the compositor-side writer capacity bridge after F5mj. It consumes `GuiRgba8888CompositorTileRleEncodeCursorOwner`, copies compositor metadata first, extracts the lower `GuiRgba8888RowTileRleEncodeCursorOwner`, and calls `gui_rgba8888_row_tile_rle_writer_plan_prepare` exactly once. It returns `GuiRgba8888CompositorTileRleWriterPlanOwner`, which stores the lower writer plan owner plus metadata for the later encoded storage / write boundary.

```text
metadata = copy compositor ready cursor owner metadata
lower_ready = move lower row tile RLE ready cursor owner
plan_or_error = gui_rgba8888_row_tile_rle_writer_plan_prepare lower_ready
success = wrap lower writer plan owner with metadata
error = copy lower kind/category/total, move lower ready cursor owner, wrap it with metadata
```

The writer plan owner exposes copied metadata, total run count, encoded byte count, cursor next pixel index, and cursor pixel count as non-consuming accessors. It does not expose the raw lower writer plan or raw lower cursor as public recovery authority. Success finish and free helpers first recover the lower cursor from the lower writer plan owner, then recover the lower payload with `gui_rgba8888_row_tile_rle_cursor_finish_payload`, wrap it as `GuiRgba8888CompositorTilePayloadOwner`, and delegate to F5me payload finish/free.

The error path follows Aquinas's plan review approval. Lower F5ci represents capacity-plan failure as a lower writer plan error that keeps the original ready cursor owner. F5mk must not publish that lower writer plan error or raw lower ready owner as compositor recovery state. It reads `kind`, `category`, and `total_run_count` before consuming the lower error, recovers the lower ready cursor owner, and normalizes it to metadata-wrapped F5mj ready cursor ownership. Error finish and free therefore delegate through F5mj ready cursor recovery and then F5me payload finish/free.

F5mk must not expose direct row tile cursor start, count step reruns, direct row tile RLE drain, storage allocation, write step, encoded seal, packet, packet record reader, tile payload direct byte reader, row byte storage accessors, `RegionToken`, `MemPtr`, raw byte load/store, source storage, destination raw storage, std present, host import, host present, platform backend, video memory, Canvas, DOM, minifb, fallback, or silent no-op behavior. It is a no storage / packet / present compositor tile RLE writer plan bridge; encoded storage, encoded transport, and present continuation remain later boundaries.

## Render2d compositor tile RLE storage boundary

F5ml is the compositor-side encoded storage allocation bridge after F5mk. It consumes `GuiRgba8888CompositorTileRleWriterPlanOwner`, copies compositor metadata first, extracts the lower `GuiRgba8888RowTileRleWriterPlanOwner`, and calls `gui_rgba8888_row_tile_rle_storage_prepare` exactly once. It returns `GuiRgba8888CompositorTileRleStorageOwner`, which stores the lower storage owner plus metadata for the later write / encoded / packet boundary.

```text
metadata = copy compositor writer plan owner metadata
lower_plan = move lower row tile RLE writer plan owner
storage_or_error = gui_rgba8888_row_tile_rle_storage_prepare lower_plan
success = wrap lower storage owner with metadata
error = copy lower kind/category/total/encoded counts, move lower writer plan owner, wrap it with metadata
```

The storage owner exposes copied metadata, total run count, encoded byte count, cursor next pixel index, and cursor pixel count as non-consuming accessors. It does not expose raw byte storage, `RegionToken`, or raw pointer access. `owner_finish_payload` is fallible because lower storage deallocation can fail. On success it converts the returned lower cursor to lower payload with `gui_rgba8888_row_tile_rle_cursor_finish_payload`, wraps it as `GuiRgba8888CompositorTilePayloadOwner`, and leaves entry finish to F5me. On lower deallocation failure it reads the lower finish kind, recovers the lower cursor from the lower finish error, converts it to the same compositor payload owner, and stores that payload in `GuiRgba8888CompositorTileRleStorageFinishError`.

The error path follows Pauli's plan review approval. Lower F5cj represents storage allocation failure as a lower storage prepare error that keeps the original writer plan owner. F5ml must not publish that lower storage prepare error as compositor recovery state. It reads `kind`, `category`, `total_run_count`, and `encoded_byte_count` before consuming the lower error, recovers the lower writer plan owner, and normalizes it to metadata-wrapped F5mk writer plan ownership. Error finish and free therefore delegate through F5mk writer plan recovery and then F5me payload finish/free.

F5ml intentionally does not provide `owner_finish_entry`. Storage finish failure and payload finish failure use different recovery owners and error domains, so combining them here would require a mixed-domain error that obscures the storage boundary. Callers that need an entry first call `owner_finish_payload`, then use F5me payload finish. `owner_free` delegates directly to lower storage owner free and wraps lower finish kind.

F5ml must not expose direct row tile write cursor start, write step, encoded seal, packet, packet record reader, tile payload direct byte reader, row byte storage accessors, `RegionToken`, `MemPtr`, raw byte load/store, source storage, destination raw storage, std present, host import, host present, platform backend, video memory, Canvas, DOM, minifb, fallback, or silent no-op behavior. It is a no write / encoded / packet / present compositor tile RLE storage bridge; encoded transport and present continuation remain later boundaries.

## SFNT simple glyph render fill alpha mask sample cursor boundary

F5bi exposes the completed F5bg fill alpha mask owner as a cell-by-cell sample stream. It is an alloc/gui owner cursor boundary. It does not emit render commands, allocate a pixel buffer, call DrawTarget / RenderTarget, call platform APIs, or introduce a compositor fallback.

The sample value is copyable:

```text
GuiSfntSimpleGlyphRenderFillAlphaMaskSample:
    position GuiPoint
    alpha i32
    alpha_max i32
    fill_paint GuiPaint
    blend GuiBlendMode
```

The cursor owns the completed fill alpha mask owner and the current cell index:

```text
GuiSfntSimpleGlyphRenderFillAlphaMaskSampleCursor:
    owner GuiSfntSimpleGlyphRenderFillAlphaMaskOwner
    cell_index i32
```

The cursor, start error, step error, and terminal are owner-bearing and must not implement `Clone` / `Copy`.

F5bi adds a completed-owner invariant helper because F5bg only validates a packed owner at the start boundary. The completed owner is rechecked before cursor start, read, and step:

```text
shape.width_px > 0
shape.height_px > 0
shape.sample_scale > 0
shape.coverage_max == sample_scale * sample_scale
shape.cell_count == width_px * height_px
owner.alpha_max > 0
owner.cell_count == shape.cell_count
owner.alpha_cells.len == shape.cell_count
owner.alpha_cells.cap == shape.cell_count
```

The cursor bounds order is part of the contract. `cell_index > cell_count is rejected before the completed state`. `cell_index == cell_count` is the only completed state. This prevents forged or corrupted cursor progress from being treated as a valid completed stream.

Position construction uses checked addition before `gui_point_new`:

```text
local_y = cell_index / width_px
local_x = cell_index - local_y * width_px
x = checked_add_nonnegative_delta origin.x local_x
y = checked_add_nonnegative_delta origin.y local_y
```

Negative local deltas are rejected as overflow even though valid invariants should make them unreachable. x overflow returns `PositionXOverflow`; y overflow returns `PositionYOverflow`. F5bi deliberately avoids `gui_point_add` because that helper does not encode checked i32 overflow.

Read validates the alpha storage at the requested cell:

```text
missing alpha slot -> AlphaSlotMissing
alpha < 0 -> AlphaNegative
alpha > alpha_max -> AlphaExceedsMax
```

On success, read returns a sample containing the absolute position, alpha, alpha_max, fill paint, and blend copied from the completed owner. It does not normalize again and does not inspect stroke / shadow.

Step returns:

```text
GuiSfntSimpleGlyphRenderFillAlphaMaskSampleCursorTerminal:
    Sampled sample next_cursor
    Completed owner
```

All step failures wrap the cursor in `GuiSfntSimpleGlyphRenderFillAlphaMaskSampleCursorError`. Start failures wrap the completed owner in `GuiSfntSimpleGlyphRenderFillAlphaMaskSampleCursorStartError`. Both error types have recovery accessors and free helpers. Terminal free closes the owner through either `Sampled` cursor or `Completed` owner.

F5bi must not call byte-backed lookup helpers, old traversal helpers, zero-fill helpers, `RenderCommand` constructors, DrawTarget / RenderTarget, platform APIs, host APIs, font fallback, stroke / shadow rasterizers, or 2D compositor APIs.

## SFNT simple glyph render fill alpha mask sample command bridge boundary

F5bj is a SourceOver only bridge from the F5bi sample stream to the existing core `RenderCommand::FillRect` value. It is a correctness bridge, not the final FHD 60fps compositor path and not a backend fallback. The bridge converts exactly one alpha-mask sample into exactly one 1x1 logical fill rectangle command.

The current `FillRectCommand` payload is:

```text
FillRectCommand:
    rect GuiRect
    paint GuiPaint
```

It does not carry `GuiBlendMode`. Therefore F5bj validates the sample blend before command construction:

```text
GuiBlendMode::SourceOver -> Ok
Copy / Multiply / Screen -> UnsupportedBlendMode
```

This prevents semantic loss. F5bj must not silently drop `sample.blend`, and it must not reinterpret unsupported blend modes as SourceOver.

The paint path reads the original fill paint through `gui_paint_color`, not through an ad-hoc field read. It preserves RGB channels and scales only alpha:

```text
base_color = gui_paint_color sample.fill_paint
paint_alpha = base_color.a
scaled_alpha = sample.alpha * paint_alpha / sample.alpha_max
command_color = rgba8888_new base.r base.g base.b scaled_alpha
command_paint = gui_paint_solid command_color
```

Alpha scaling is fail-closed:

```text
sample.alpha_max <= 0 -> InvalidAlphaMax
sample.alpha < 0 -> AlphaNegative
sample.alpha > sample.alpha_max -> AlphaExceedsMax
sample.alpha * paint_alpha overflow -> PaintAlphaMultiplyOverflow
scaled alpha outside 0..255 -> ScaledAlphaOutOfRange
```

`sample.alpha == 0` and `paint_alpha == 0` still produce a transparent `FillRect` command. They are not treated as skip/no-op states. This keeps the command bridge observable and avoids hidden control flow.

The cursor command step is deliberately not implemented by calling the owning F5bi `sample_cursor_step`. The rule is that conversion succeeds before the cursor advances:

```text
validate cursor invariants
if cell_index > cell_count:
    return error with cursor
if cell_index == cell_count:
    return Completed owner
read sample by reference
convert sample to command
if conversion failed:
    return error with original cursor and rejected_sample
move owner into next cursor
return Command command next_cursor
```

This avoids partial completion. A command conversion failure never consumes the cursor. A sample cursor invariant/read failure returns `rejected_sample = None`, while command conversion failure returns `rejected_sample = Some sample`. Progress invariant failure owns the next cursor and keeps the sample in the error payload.

The cursor command error kind keeps F5bi read/invariant errors separate from command conversion errors:

```text
GuiSfntSimpleGlyphRenderFillAlphaMaskSampleCommandCursorErrorKind:
    SampleCursorFailed GuiSfntSimpleGlyphRenderFillAlphaMaskSampleCursorErrorKind
    CommandFailed GuiSfntSimpleGlyphRenderFillAlphaMaskSampleCommandErrorKind
```

F5bj may call `render_command_fill_rect`. It must not call `RenderTarget`, `DrawTarget`, platform APIs, backend APIs, font fallback, zero-fill helpers, stroke / shadow rasterizers, or 2D compositor APIs.

## SFNT simple glyph alpha mask render command boundary

F5bk adds the core command shape that the later compositor and host transport can consume without expanding a glyph mask into per-sample `FillRect` commands. It is a core/gui contract boundary, not a renderer implementation and not a fallback path.

The new handle mirrors the existing `ImageId` / `TextRunId` pattern:

```text
AlphaMaskId:
    raw i32
```

The payload is a no_alloc value:

```text
AlphaMaskRectCommand:
    mask_id AlphaMaskId
    rect GuiRect
    paint GuiPaint
```

The corresponding command variant is:

```text
RenderCommand::AlphaMaskRect AlphaMaskRectCommand
```

`AlphaMaskRectCommand` is SourceOver only. It deliberately has no `GuiBlendMode` field. The renderer resolves `mask_id`, reads the mask resource outside core, and draws the mask into `rect` using `GuiPaint` as the source color. RGB comes from `GuiPaint`. Effective source alpha is derived from mask alpha and `GuiPaint` alpha under the SourceOver contract. If the glyph paint asks for `Copy`, `Multiply`, `Screen`, or any future non-SourceOver blend, the glyph renderer must fail before constructing `AlphaMaskRectCommand`.

This rule prevents the same semantic loss that F5bj had to avoid for `FillRectCommand`. The command boundary is a typed resource operation, not an instruction to reinterpret unsupported blend as SourceOver and not a request to fall back to a pixel loop.

Core does not store or inspect mask bytes. Core also does not know mask width, height, stride, texture upload state, cache entries, browser canvas handles, native surface handles, or font table owners. Those are alloc/std/platform/renderer responsibilities. If the renderer cannot resolve `AlphaMaskId`, if dimensions are incompatible with the target, or if the backend does not support the command, it returns `Result` / `GuiError` at that layer. It must not silently ignore the command.

The helper functions are deliberately O(1):

```text
alpha_mask_id_new raw
alpha_mask_id_raw id
alpha_mask_rect_command_mask_id command
alpha_mask_rect_command_rect command
alpha_mask_rect_command_paint command
render_command_alpha_mask_rect mask_id rect paint
```

F5bk must not introduce `Vec`, `String`, allocator calls, platform calls, RenderTarget / DrawTarget implementation, Canvas / DOM / minifb bindings, font fallback, byte-backed font lookup, or compositor drain. Later slices can bind a completed fill alpha mask owner to an `AlphaMaskId` resource table and can define tile / row / bitmap transport, but that storage boundary is not part of F5bk.

## SFNT simple glyph alpha mask resource reservation boundary

F5bl is the internal alloc/font reservation boundary between the completed fill alpha mask owner and the future resource table. It deliberately stops before table registration and before command construction. The reservation object proves only that a completed mask owner, a nonzero `AlphaMaskId`, a `GuiRect`, and a `GuiPaint` have been checked and kept together under one owner-bearing value.

F5bl must not claim that the id is registered, unique, uploaded, host-visible, or renderable. Those facts belong to a later resource-table boundary that consumes the reservation owner and registers the alpha storage before constructing `RenderCommand::AlphaMaskRect`.

The value-only config is:

```text
GuiSfntSimpleGlyphRenderFillAlphaMaskResourceReservationConfig:
    mask_id AlphaMaskId
```

The success owner is private to `alloc/gui/font/sfnt/glyf`:

```text
GuiSfntSimpleGlyphRenderFillAlphaMaskResourceReservationOwner:
    owner GuiSfntSimpleGlyphRenderFillAlphaMaskOwner
    mask_id AlphaMaskId
    rect GuiRect
    paint GuiPaint
```

`owner` keeps the alpha Vec and raster edge owner alive. `mask_id`, `rect`, and `paint` are copied values derived from the config and completed owner. The success owner and the start error own storage and therefore must not implement `Clone` or `Copy`. The config is value-only and may implement both.

Validation order is stable:

```text
validate config mask id
    AlphaMaskId.raw <= 0 -> InvalidMaskId

validate completed owner shape
    width_px <= 0 -> ShapeInvalidWidth
    height_px <= 0 -> ShapeInvalidHeight
    sample_scale <= 0 -> ShapeInvalidSampleScale
    sample_scale * sample_scale overflow or mismatch -> ShapeCoverageMaxMismatch
    width_px * height_px overflow or mismatch -> ShapeCellCountMismatch

validate completed owner alpha storage
    alpha_max <= 0 -> InvalidAlphaMax
    owner.cell_count != shape.cell_count -> AlphaCellCountMismatch
    alpha_cells.len != shape.cell_count -> AlphaStorageLenMismatch
    alpha_cells.cap != shape.cell_count -> AlphaStorageCapacityMismatch

validate blend
    SourceOver -> Ok
    other -> UnsupportedBlendMode
```

The rect is built from `owner.origin` and `owner.size`. No alpha cell is copied. No sample cursor is opened. No per-sample `FillRect` bridge is called. The success path only packs metadata and the existing owner into the reservation owner.

Recovery is explicit. A start error keeps the original completed fill alpha mask owner and the config. A consuming success recovery helper returns the original `GuiSfntSimpleGlyphRenderFillAlphaMaskOwner` so a later internal registration boundary can consume the reservation without field projection. Value accessors for `mask_id`, `rect`, and `paint` may be read before recovery.

F5bl must not call `render_command_alpha_mask_rect`, `render_command_fill_rect`, DrawTarget, RenderTarget, platform APIs, host APIs, backend APIs, Canvas, DOM, minifb, font fallback, zero-fill fallback, per-sample FillRect fallback, alpha Vec copy helpers, or a 2D compositor drain.

## SFNT simple glyph alpha mask resource table boundary

F5bm is the internal alloc/font registration boundary after F5bl. It consumes the reservation owner and inserts a Copy metadata record into a private resource table. The alpha storage owner is not stored inside the table `Vec`; it remains inside a registered resource owner that is returned together with the updated table owner.

This shape is intentional. A generic owner-bearing resource `Vec` would require a consuming destructor for every stored mask owner. Until that owner list / drain contract is explicitly proven, F5bm keeps the table as metadata-only and keeps the storage owner in a separate owner-bearing value. The success owner therefore contains both parts:

```text
GuiSfntSimpleGlyphRenderFillAlphaMaskResourceTableRegistrationOwner:
    table GuiSfntSimpleGlyphRenderFillAlphaMaskResourceTableOwner
    resource GuiSfntSimpleGlyphRenderFillAlphaMaskRegisteredResourceOwner
```

The table can answer metadata questions:

```text
contains AlphaMaskId -> bool
lookup AlphaMaskId -> Option ResourceRecord
len -> i32
```

`lookup` does not borrow or expose alpha storage. It proves only that the metadata id is present in the private table. It does not prove host upload, renderability, texture lifetime, backend support, or presentation readiness.

Registration order:

```text
1. re-read AlphaMaskId from reservation metadata
2. reject non-positive id
3. revalidate completed fill alpha mask owner shape and alpha storage invariants
4. revalidate SourceOver
5. rederive rect from owner origin and size and compare with reservation rect
6. compare reservation paint with owner fill paint
7. scan existing metadata records for duplicate id
8. push the new metadata record
9. return updated table + registered resource owner
```

The `vec::push` happens only after all semantic checks pass. A push failure returns the original table owner and reservation owner through a typed owner-bearing error. No partial registration is allowed, and no metadata-only success may escape without the corresponding storage owner.

The success continuation is callback-shaped: it consumes the registration owner and invokes the callback with the updated table owner and the registered resource owner at the same time. The error recovery path is also pair-shaped: the error can be converted into a rejected owner that keeps the table owner and reservation owner together, and the rejected owner can only be consumed by a callback receiving both owners. F5bm must not expose split consuming accessors that return only table or only reservation.

F5bm must not call `render_command_alpha_mask_rect`, `render_command_fill_rect`, DrawTarget, RenderTarget, platform APIs, host APIs, backend APIs, Canvas, DOM, minifb, font fallback, zero-fill fallback, per-sample FillRect bridge, sample cursor, alpha Vec copy helpers, owner-bearing Vec payload storage, or a 2D compositor drain.

## SFNT simple glyph alpha mask prepared command boundary

F5bn consumes a registered resource owner and prepares the `RenderCommand::AlphaMaskRect` value that will later be consumed by a formal transport or compositor drain owner. This is not command stream emission. The command is a Copy value, so exposing it through an accessor or arbitrary callback would allow callers to keep the command after dropping the registered resource owner. That would reintroduce a dangling `AlphaMaskId` command.

The F5bn owner is therefore a sealed internal owner-bearing value:

```text
GuiSfntSimpleGlyphRenderFillAlphaMaskResourcePreparedCommandOwner:
    resource GuiSfntSimpleGlyphRenderFillAlphaMaskRegisteredResourceOwner
    command RenderCommand
```

It may expose metadata derived from the resource record, but it must not expose the raw `RenderCommand`, a `&RenderCommand`, or a generic callback receiving `RenderCommand`. The only complete consumption path in this slice is free. A later formal transport / drain owner will consume this prepared owner and decide how command and resource lifetime are held together while the command is presented.

Prepare order:

```text
1. read the stored resource record
2. reject non-positive AlphaMaskId
3. rederive the expected record from the internal reservation owner
4. map reservation invariant, SourceOver, rect, and paint errors into typed prepared-command errors
5. compare mask id, rect, paint, width, height, cell count, and alpha max
6. call render_command_alpha_mask_rect only after the records match
7. store the returned RenderCommand inside the prepared owner without exposing it
```

The error path keeps the registered resource owner and never stores a command. Recovery may pass only the registered resource owner to a callback, or free it.

F5bn must not call DrawTarget, RenderTarget, platform APIs, host APIs, backend APIs, Canvas, DOM, minifb, font fallback, zero-fill fallback, per-sample FillRect bridge, sample cursor, resource table lookup, alpha Vec copy helpers, tile / bitmap transport, or a 2D compositor drain. It must not call `render_command_fill_rect`. It may call `render_command_alpha_mask_rect` only in the validated success path and only to store the command inside the prepared owner.

## Software RGBA8888 surface owner boundary

The software RGBA8888 surface owner is a render2d foundation, not a glyph parser detail. The first F5bo plan put a surface owner and SourceOver drain in `alloc/gui/font/sfnt/glyf.nepl`; review blocked that placement because a pixel buffer is shared by widget painting, offscreen rendering, screenshots, and future font raster output. The revised plan creates `alloc/gui/render2d` and keeps the first slice limited to storage ownership and checked pixel access.

The owner layout is row-major RGBA8888:

```text
stride_bytes = width * 4
byte_len = height * stride_bytes
pixel_byte_offset = y * stride_bytes + x * 4
channel order = r, g, b, a
```

`GuiRgba8888SoftwareSurfaceOwner` owns `RegionToken u8`. It is not Clone or Copy. The module must not export a raw pointer accessor, raw region accessor, or any helper that reveals `MemPtr` / raw address authority. Internal storage access goes through the safe `core/mem` facade only:

```text
alloc_region_bytes<u8>
region_ptr_at<u8,u8>
load_u8
store_u8
dealloc_region<u8>
```

The module must not import `core/mem/raw` or `core/mem/internal`.

Construction validation order:

```text
1. width > 0 and height > 0
2. width * height does not overflow i32
3. width * 4 does not overflow i32
4. height * stride_bytes does not overflow i32
5. alloc_region_bytes<u8> succeeds
6. zero initialization succeeds
```

The shape calculation returns `Result GuiRgba8888SoftwareSurfaceShape GuiRgba8888SoftwareSurfaceErrorKind`, so callers can inspect validation without allocating. Allocation failure is `OutOfMemory`. Pointer projection failure, load failure, and store failure are only produced after safe `core/mem` facade calls fail.

Write access consumes the surface owner:

```text
gui_rgba8888_software_surface_write_pixel
    owner x y color
    -> Ok owner
    -> Err GuiRgba8888SoftwareSurfaceWriteError
```

The write error carries the surface owner, so callers can recover, free, or retry. A write must validate bounds and byte offset before the first store. If a later safe store fails, the returned owner still represents the same allocation and must remain recoverable.

Read access borrows the owner:

```text
gui_rgba8888_software_surface_read_pixel
    &owner x y
    -> Result Rgba8888 GuiRgba8888SoftwareSurfaceErrorKind
```

Read may not consume or duplicate the `RegionToken u8` owner.

F5bo deliberately does not consume the F5bn prepared command owner. It does not implement SourceOver, alpha-mask drain, DrawTarget fallback, RenderTarget backend, Canvas, DOM, minifb, or platform present. The next slice should define a compositor owner that consumes both the prepared alpha-mask command owner and a software surface owner and then returns exactly one updated surface owner plus released font/resource owners.

## SourceOver alpha-mask software drain-start owner boundary

F5bp is the first bridge from the sealed F5bn prepared command owner to the shared F5bo software surface owner. The bridge is intentionally a drain-start / drain-cursor boundary. It is not a completed drain, and it must not write pixels yet. No SourceOver arithmetic is executed in this slice. The point of the boundary is to prove that a prepared `AlphaMaskRect` command, its still-owned alpha mask resource, and an RGBA8888 software surface are held together before the later bounded drain step mutates the surface.

The owner layout is:

```text
GuiSfntSimpleGlyphRenderFillAlphaMaskSoftwareDrainOwner:
    prepared GuiSfntSimpleGlyphRenderFillAlphaMaskResourcePreparedCommandOwner
    surface GuiRgba8888SoftwareSurfaceOwner
    cell_index i32
```

`cell_index` starts at zero. A later step function will consume this owner, process a bounded number of cells, and return either the next owner or a completed owner. F5bp does not define that step yet, because the next slice needs SourceOver arithmetic, alpha-cell reads, write failure recovery, dirty rect handling, and partial-progress semantics reviewed together.

Start validation must rederive the authoritative resource record from the internal reservation even though F5bn already did it. The prepared owner is a sealed lifetime boundary, not a permanent proof that future consumers can stop checking. The validation order is:

```text
1. borrow prepared.resource
2. read the stored record from the registered resource
3. borrow the registered resource reservation
4. rederive expected record from the reservation
5. compare stored and expected records
6. read prepared.command only inside the start validation helper
7. require RenderCommand::AlphaMaskRect
8. compare command mask id, rect, and paint with the rederived record
9. validate record width, height, cell count, and alpha max
10. validate rect origin and size
11. compute right and bottom through checked addition
12. compare right and bottom with surface width and height
```

The geometry validation may not call `gui_rect_right` or `gui_rect_bottom`, because those helpers perform unchecked addition. F5bp instead checks origin and size first, computes `max_i32 - extent`, then rejects overflow before adding.

Error recovery is paired:

```text
GuiSfntSimpleGlyphRenderFillAlphaMaskSoftwareDrainStartError:
    kind GuiSfntSimpleGlyphRenderFillAlphaMaskSoftwareDrainErrorKind
    prepared GuiSfntSimpleGlyphRenderFillAlphaMaskResourcePreparedCommandOwner
    surface GuiRgba8888SoftwareSurfaceOwner

GuiSfntSimpleGlyphRenderFillAlphaMaskSoftwareDrainRejected:
    prepared GuiSfntSimpleGlyphRenderFillAlphaMaskResourcePreparedCommandOwner
    surface GuiRgba8888SoftwareSurfaceOwner
```

There must be no split consuming accessor returning only the prepared owner or only the surface owner. Recovery goes through a rejected owner and a callback that receives both owners at once. This mirrors the earlier table-registration pair recovery and prevents the command lifetime and surface lifetime from drifting apart.

F5bp may import `alloc/gui/render2d` because the surface owner is a shared render2d foundation. `alloc/gui/render2d` must not import `alloc/gui/font/sfnt/glyf`, and the surface owner remains outside the font module. The bridge belongs in `glyf.nepl` only because the prepared command owner is sealed there and its private command field must not be exposed.

F5bp must not call `gui_rgba8888_software_surface_write_pixel`, read or write raw memory, DrawTarget, RenderTarget, backend APIs, platform APIs, Canvas, DOM, minifb, fallback paths, per-sample FillRect bridge, or command-stream emission. It may call surface width / height accessors and surface free.

## SourceOver alpha-mask software drain-step boundary

F5bq turns the F5bp cursor owner into the first real software compositing slice. The slice still stays in `alloc`: it mutates an owned RGBA8888 software surface, but it does not present to Web, native, mobile, bare display hardware, or a headless screenshot transport. Dirty metadata, presentation, dirty region aggregation, tile transport, command batching, stroke, and shadow are separate boundaries.

The reusable alpha compositing rule is located in `alloc/gui/render2d/composite.nepl`, not in `alloc/gui/font/sfnt/glyf.nepl`. The glyph module owns the sealed prepared command and alpha-mask resource. Render2d owns the reusable pixel math.

The render2d formula is:

```text
src_a = mask_alpha * source.a / mask_alpha_max
inv = 255 - src_a
out_alpha_num = src_a * 255 + dest.a * inv
out_a = out_alpha_num / 255
out_premul_num_c = source.c * src_a * 255 + dest.c * dest.a * inv
out_c = if out_alpha_num == 0 then 0 else out_premul_num_c / out_alpha_num
```

All divisions operate on nonnegative signed integers and therefore truncate as floor division. `mask_alpha_max <= 0`, negative mask alpha, mask alpha above max, source alpha multiplication overflow, scaled source alpha outside `0..255`, output alpha outside `0..255`, and output channel outside `0..255` are typed errors. The RGB calculation keeps `out_alpha_num` as the denominator instead of dividing the destination premultiplied term early. This prevents low-alpha unpremultiply overflow; for example, source RGBA `255 255 255 1` over destination RGBA `255 255 255 1` produces channel 255, not a value above 255. The worst intermediate `255 * 255 * 255` is below i32 max, so the numerator arithmetic is safe after input validation.

The software surface write path is strengthened before the drain uses it. `gui_rgba8888_software_surface_write_pixel` computes the pixel byte offset and then projects all four channel pointers before the first store. If any projection fails, no channel is modified. Under the invariant that `GuiRgba8888SoftwareSurfaceOwner` owns allocator-created `RegionToken` storage and exposes no raw constructor or raw storage accessor, a successful projection yields a positive `MemPtr`, and `store_u8` cannot fail after that point.

The drain terminal is owner-bearing:

```text
GuiSfntSimpleGlyphRenderFillAlphaMaskSoftwareDrainTerminal:
    Completed GuiSfntSimpleGlyphRenderFillAlphaMaskSoftwareDrainCompletedOwner
    StepBudgetExhausted GuiSfntSimpleGlyphRenderFillAlphaMaskSoftwareDrainOwner
```

`CompletedOwner` keeps the prepared owner and surface owner paired. It is not a split accessor. The only way to take the surface is to consume the completed owner, free the prepared/resource side, and return the surface as the explicit result of that finish helper.

The step contract is:

```text
validate existing prepared/surface pair
read cell_index
if cell_index == cell_count:
    return Completed
if remaining_steps <= 0:
    return InvalidBudget error
read alpha cell by borrowed Vec access
compute checked x/y
read dest pixel
source-over composite
write pixel
advance cell_index after successful write
```

Failures before write return the unchanged owner. Write failure recovers the surface owner from `GuiRgba8888SoftwareSurfaceWriteError`, reconstructs the drain owner with the same `cell_index`, and returns typed `SurfaceWriteFailed`. A successful write must advance by exactly one cell. Any other progress is `ProgressInvariantInvalid`.

Positive budget exhaustion is `StepBudgetExhausted`, not completion and not failure. `remaining_steps <= 0` on a non-completed owner is invalid caller scheduling and returns `InvalidBudget`.

F5bq may call `gui_rgba8888_software_surface_read_pixel`, the checked `gui_rgba8888_software_surface_write_pixel`, and `gui_rgba8888_source_over_alpha_mask`. It must not call F5bj sample command bridge, `render_command_fill_rect`, raw `RenderCommand` accessors, DrawTarget, RenderTarget, backend APIs, platform APIs, Canvas, DOM, minifb, byte-backed lookup helpers, old traversal helpers, font fallback, zero-fill fallback, alpha Vec clone/copy, or unchecked rect extent helpers.

## SourceOver alpha-mask dirty-region completion boundary

F5br attaches a `DirtyRegion` value to the F5bq completed owner. This is intentionally narrower than a general render2d `surface + dirty` owner. The next transport slice still has to decide whether multiple glyph drains are merged as a single `DirtyRegion`, a fixed-capacity `DirtyRegionSet`, or a tile list. Putting a generic render2d owner in this slice would force those aggregation choices too early.

The completed owner shape becomes:

```text
GuiSfntSimpleGlyphRenderFillAlphaMaskSoftwareDrainCompletedOwner:
    prepared GuiSfntSimpleGlyphRenderFillAlphaMaskResourcePreparedCommandOwner
    surface GuiRgba8888SoftwareSurfaceOwner
    dirty DirtyRegion
```

The owner remains non-Clone and non-Copy. `dirty` is Copy metadata and may be returned by a borrowed accessor. `prepared` and `surface` are still not exposed as split accessors. The only way to take the surface is still the consuming finish helper that frees the prepared/resource side first. Callers that need both dirty metadata and surface ownership must read the dirty value before calling the finish helper.

The dirty value is created from the rederived resource record rect through `dirty_region_rect_checked`. This is not a fallback or a duplicate workaround. It gives `core/gui/dirty_region` authority over the dirty metadata contract. Even though earlier start validation already checked rect geometry and surface containment, the dirty value is still constructed through the checked dirty-region constructor. Failure is mapped to `DirtyRegionInvalid` and returned as an owner-bearing step error; it is not converted to Full, Empty, or a silent no-op.

The completion branch order is fixed:

```text
validate existing prepared/surface pair
rederive resource record
read cell_index
if cell_index == cell_count:
    dirty = dirty_region_rect_checked record.rect
    if dirty fails:
        return owner-bearing DirtyRegionInvalid
    move prepared and surface out of owner
    return Completed prepared surface dirty
```

This order avoids losing the prepared/surface owners on dirty construction failure. It also preserves the F5bq rule that owner fields are moved only after all value-level checks needed for the terminal have succeeded.

F5br must not call Web/native host APIs, video-memory publish helpers, tile or bitmap transport helpers, DrawTarget, RenderTarget, Canvas, DOM, minifb, the old per-sample FillRect bridge, raw `RenderCommand` accessors, fallback paths, or unchecked dirty-region fallback helpers.

## SourceOver dirty region set aggregation boundary

F5bs chooses the first aggregation shape after F5br without committing to formal transport. The completed glyph drain still exposes one `DirtyRegion`; the pre-transport collector can now fold that value into a no_alloc fixed-capacity `DirtyRegionSet` by calling `dirty_regions_push_region_checked`.

The helper has one responsibility:

```text
DirtyRegion + DirtyRegionSet -> Result DirtyRegionSet GuiError
```

The variant policy is explicit.

```text
Empty
    return the existing set unchanged

Full
    return dirty_regions_full

Rect rect
    pass rect through dirty_regions_push_checked
```

This is not a fallback path. `Empty` is a valid source state meaning no pixels changed, and `Full` is a valid source state meaning all pixels are dirty. `Rect` still goes through checked insertion so a rect built by `dirty_region_rect_unchecked` is rejected if width or height is negative.

The helper deliberately does not use `dirty_region_merge`. A bounding `DirtyRegion` would discard the fixed-capacity two-rect policy too early. It also does not allocate a `Vec`, create a generic render2d `surface+dirty owner`, construct tile lists, publish bitmap payloads, or call a host present API. Those choices belong to later transport and scheduler slices.

F5bs source policy checks the new helper, its checked insertion path, the absence of unchecked push and `dirty_region_merge`, and the absence of allocator / platform / present / tile / bitmap / transport / fallback APIs.

## Render2d surface + dirty owner boundary

F5bt creates the shared render2d surface + dirty owner boundary that F5br deliberately deferred. The authority of this phase is the already-owned `GuiRgba8888SoftwareSurfaceOwner` from F5bo and the fixed-capacity `DirtyRegionSet` contract from F5bs.

```text
GuiRgba8888SoftwareSurfaceOwner
    + DirtyRegionSet
    -> GuiRgba8888SoftwareSurfaceDirtyOwner
```

The owner is not a platform surface and not a transport payload. It is an alloc/render2d ownership boundary that keeps pixel memory and dirty metadata together until a later tile / bitmap transport phase consumes both.

The update rule is ordered:

```text
read dirty Copy metadata from owner
call dirty_regions_push_region_checked dirty region
if error:
    return owner-bearing dirty push error with the original owner
if ok:
    move surface out of owner
    return new owner with surface and next_dirty
```

This order is the important part of the contract. `dirty_regions_push_region_checked` must happen before `field::get owner "surface"` so an invalid unchecked dirty rect cannot lose the surface owner. The error type carries both the `GuiError` and the original `GuiRgba8888SoftwareSurfaceDirtyOwner`.

The public surface is intentionally narrow. Width, height, stride, byte length, and dirty set are exposed as borrowed Copy metadata. There is no raw surface accessor, mutable surface accessor, or split accessor. `finish_surface` exists only as a recovery / teardown function that discards dirty metadata and returns the surface owner. A caller that needs dirty metadata must read it before `finish_surface`.

F5bt must not allocate tile lists, publish bitmap payloads, present to a host, call Web/native APIs, integrate font/glyf rendering directly, write pixels, use `dirty_region_merge`, use `dirty_regions_push_unchecked`, or introduce fallback behavior. Those decisions remain in later formal transport and scheduler phases.

## Render2d validated bitmap frame owner boundary

F5bu is the first boundary that gives the later formal transport a single validated frame owner. It consumes the F5bt dirty owner and a small config, but still does not publish pixels. The resulting owner type is `GuiRgba8888BitmapFrameOwner`; it carries only validated frame metadata, the dirty set, and the software surface owner.

```text
Dirty surface owner
    -> validate positive frame id
    -> revalidate surface shape metadata
    -> validate dirty rect bounds
    -> finish_surface
    -> Bitmap frame owner
```

The validation order is part of the design. `finish_surface` must not appear before `frame_id > 0`, shape, stride, byte length, and dirty bounds validation. This preserves the original dirty owner on every failure and prevents a forged public struct from becoming transport authority by construction alone.

Surface validation recomputes `GuiRgba8888SoftwareSurfaceShape` from width and height. Shape construction failure maps to `SurfaceInvalidGeometry`; expected stride and byte length mismatches map to `SurfaceStrideMismatch` and `SurfaceByteLengthMismatch`. These are typed domain errors, not string diagnostics.

Dirty validation treats `Empty` and `Full` as explicit valid states. `One` validates the first rect. `Two` validates the first and second rect. Rect validation requires non-negative origin, non-negative size, checked `x + width` / `y + height`, and containment in the surface bounds; containment failure maps to `DirtyRectOutOfBounds`. Zero-sized dirty rects remain valid metadata; deciding whether they generate host work belongs to the later transport scheduler, not to this owner boundary.

The prepare error carries a typed `GuiRgba8888BitmapFramePrepareErrorKind`, an optional coarse `GuiError` category, and the original dirty owner. The `category` field is only a classification for callers that need a general GUI error; it is not evidence that a lower host API failed.

F5bu deliberately does not expose surface raw storage, row copy helpers, byte payload builders, tile lists, video-memory calls, host present, Canvas, DOM, minifb, or fallback paths. Those choices remain in the formal transport and scheduler phases.

## Render2d row batch plan owner boundary

F5bv turns the validated bitmap frame owner into a row batch plan owner. The purpose is to give the later scheduler and transport layer a validated contiguous row span and batch count without exposing pixel storage or calling a host. The owner type is `GuiRgba8888RowBatchPlanOwner`; it carries the original `GuiRgba8888BitmapFrameOwner`, revalidated frame metadata, dirty metadata, row_start, row_count, batch_count, and max_rows_per_batch.

```text
Bitmap frame owner
    -> validate positive max_rows_per_batch
    -> revalidate frame_id
    -> revalidate frame shape metadata
    -> validate dirty rect bounds
    -> compute contiguous row span
    -> compute quotient/remainder batch count
    -> Row batch plan owner
```

The validation order is part of the design. Normal application code cannot directly forge an owner-backed aggregate because the compiler rejects owner aggregate constructors outside the memory boundary. F5bv still repeats the checks that matter for row planning because a compiler memory boundary or trusted producer can hand it malformed metadata: `frame_id > 0`, width / height shape construction, stride mismatch, byte length mismatch, dirty rect origin, dirty rect size, checked right / bottom extents, and surface containment. Representative typed errors include `MaxRowsPerBatchInvalid`, `FrameStrideMismatch`, `DirtyRectBottomOverflow`, and `DirtyRectOutOfBounds`.

Dirty state is mapped to a contiguous row span, not a payload. `Empty` becomes row_start 0 and row_count 0. `Full` becomes row_start 0 and row_count equal to frame height. `One` uses the checked rect y and bottom. `Two` validates both rects and uses min y plus max bottom to create one contiguous row span. This keeps the fixed-capacity dirty set deterministic while deferring tile splitting, byte copying, and queue scheduling to later phases.

Batch count uses quotient and remainder rather than `row_count + max_rows_per_batch - 1`, so the calculation does not introduce a new overflow path. A zero row span has zero batches. Non-zero row spans require positive max rows per batch and return a typed error otherwise.

The prepare error carries a typed `GuiRgba8888RowBatchPlanPrepareErrorKind`, an optional coarse `GuiError` category, and the original bitmap frame owner. `finish_frame` is the only consuming recovery path from a successful plan back to the bitmap frame owner. `finish_frame` is intentionally distinct from `finish_surface`; the row planner is not allowed to bypass the frame boundary and recover the underlying surface directly.

F5bv deliberately does not expose raw storage, allocate row bytes, create tile lists, publish video memory, call host present, touch Canvas / DOM / minifb, or implement fallback behavior. Those choices remain in the formal byte payload, transport, and scheduler phases.

## Render2d row batch cursor owner boundary

F5bw turns the row batch plan owner into a row batch cursor owner. The concrete owner type is `GuiRgba8888RowBatchCursorOwner`. The cursor is the first scheduler-facing boundary: it can report whether work remains and can emit one descriptor for the next contiguous row batch. It still does not expose raw bytes, allocate byte payloads, write rows, create tiles, publish video memory, call host present, touch Canvas / DOM / minifb, or implement fallback behavior.

```text
Row batch plan owner
    -> start validates full plan invariants
    -> Cursor owner at batch_index 0
    -> status reads Ready or Complete
    -> next_batch emits descriptor plus continuation cursor
```

`GuiRgba8888RowBatchCursorErrorKind` keeps plan validation precision by carrying `GuiRgba8888RowBatchPlanInvariantErrorKind` as the `PlanInvariant` payload. The cursor layer adds only cursor-local errors: negative index, past-end index, descriptor offset overflow, descriptor bounds invalid, and checked next-index overflow. This avoids duplicating the plan invariant enum while preserving typed `match`-visible failure state.

`GuiRgba8888RowBatchCursorStatus` is a Copy value enum. `Ready` means `0 <= batch_index < batch_count`; `Complete` means `batch_index == batch_count`; any value outside that range is an error. A complete cursor does not create an owner-bearing terminal wrapper. The caller recovers the plan with `gui_rgba8888_row_batch_cursor_finish_plan` or frees it through `gui_rgba8888_row_batch_cursor_free`.

`gui_rgba8888_row_batch_cursor_next_batch` consumes a cursor only when a descriptor is requested. It first calls `status`; `Complete` maps to `CursorIndexPastEnd`, not to silent success. For `Ready`, descriptor construction uses the stored row span and max rows per batch. The next cursor index is computed with checked arithmetic before the continuation cursor is constructed. The normal scheduler path is to read the descriptor, then call `gui_rgba8888_row_batch_cursor_batch_finish_cursor` to recover the continuation cursor owner for the next slice.

F5bw intentionally excludes drain / budget logic. Bounded draining is a scheduler policy built on top of `status` and `next_batch`; including it in the cursor slice would add another owner-bearing terminal and obscure the root boundary. This keeps the first cursor contract small enough for doctests and source policy to validate directly.

## Render2d row batch scheduler drain boundary

F5bx adds the scheduler progress boundary above F5bw without changing the cursor module. The concrete terminal type is `GuiRgba8888RowBatchDrainTerminal`. It is an owner-bearing struct, not a Copy enum, because it carries the continuation `GuiRgba8888RowBatchCursorOwner`. The terminal status is split into the Copy enum `GuiRgba8888RowBatchDrainStatus`:

```text
GuiRgba8888RowBatchDrainStatus:
    Completed
    StepBudgetExhausted

GuiRgba8888RowBatchDrainTerminal:
    status GuiRgba8888RowBatchDrainStatus
    cursor GuiRgba8888RowBatchCursorOwner
    emitted_count i32
```

This shape keeps owner recovery simple for the Resource checker. Callers read the status and emitted count by reference, then consume the terminal with `gui_rgba8888_row_batch_drain_terminal_finish_cursor` or free it. The terminal is not Clone or Copy.

The drain order is fixed:

```text
status cursor
    Complete -> Completed cursor emitted_count
    Ready ->
        remaining_steps < 0 -> InvalidBudget owner-bearing error
        remaining_steps == 0 -> StepBudgetExhausted cursor emitted_count
        remaining_steps > 0 -> next_batch once, validate progress, continue
```

Complete is intentionally checked before budget. A complete cursor remains complete even if a caller passed a stale or negative scheduler budget. Negative budget is still rejected for a non-complete cursor because it means the scheduler called the API with an invalid slice contract. Zero budget is a valid pause point.

F5bx is progress-only. It calls `next_batch` to prove cursor progress, but it does not store descriptors in a `Vec`, create row bytes, create tiles, publish video memory, call host present, or touch Canvas / DOM / minifb. The emitted count means "number of batch descriptors advanced in this call"; it is not transport payload emission and must not be treated as host-present authority.

After `next_batch`, the drain reads the Copy descriptor metadata and recovers the continuation cursor. It rejects any mismatch between the descriptor batch index and the previous cursor index, and also rejects any continuation cursor index other than `previous + 1`. The expected next index and emitted count are computed through checked arithmetic; failures are owner-bearing `ProgressInvariantInvalid` or `EmittedCountOverflow`.

## Render2d row batch range metadata boundary

F5by adds a row batch range metadata boundary above the cursor batch owner. The concrete success type is `GuiRgba8888RowBatchRangeOwner`. It is owner-bearing because it keeps the original `GuiRgba8888RowBatchCursorBatchOwner`; the Copy metadata value inside it is `GuiRgba8888RowBatchRange`.

```text
Row batch cursor batch owner
    -> validate descriptor authority against embedded plan
    -> validate descriptor row and byte range
    -> validate continuation cursor
    -> Row batch range owner
```

Descriptor authority belongs to `row_batch_cursor`, not to `row_batch_range`. The helper `gui_rgba8888_row_batch_cursor_batch_validate_descriptor_authority` borrows the embedded continuation cursor and embedded plan, calls the existing plan invariant path, recomputes the expected descriptor for `continuation_index - 1`, and compares every field: frame_id, batch_index, row_start, row_count, width, height, stride_bytes, and byte_len. A mismatch becomes `BatchDescriptorMismatch`, and a malformed embedded plan remains `PlanInvariant lower_kind`. This prevents a forged descriptor that is internally valid but not plan-derived from becoming later transport authority.

`gui_rgba8888_row_batch_range_prepare` calls that authority helper before it performs its own range arithmetic. The range arithmetic checks `width > 0`, `height > 0`, `width * 4 == stride_bytes`, `height * stride_bytes == byte_len`, nonnegative `row_start`, positive `row_count`, `row_start + row_count <= height`, `start_byte_offset = row_start * stride_bytes`, `byte_count = row_count * stride_bytes`, and `start_byte_offset + byte_count <= byte_len`. All multiplication and addition are checked and return typed enum errors such as `StrideOverflow`, `ByteLengthMismatch`, `RowExtentOutOfBounds`, `RangeOffsetOverflow`, and `RangeEndOutOfBounds`.

Range prepare keeps lower cursor errors visible. Descriptor / plan authority failures are wrapped as `BatchAuthorityInvalid %GuiRgba8888RowBatchCursorErrorKind`; invalid continuation cursor status is wrapped separately as `ContinuationCursorInvalid %GuiRgba8888RowBatchCursorErrorKind`. Continuation index mismatch is its own domain error. This split matters because an application or test harness can distinguish forged metadata, stale continuation, and local range arithmetic failure with `match`.

The later byte storage phase uses `gui_rgba8888_row_batch_range_owner_validate_authority` as a borrowed revalidation boundary. It recomputes descriptor authority, recomputes range metadata, compares the stored `GuiRgba8888RowBatchRange`, and returns `RangeMetadataMismatch` if any range field diverges. It does not consume the range owner.

F5by remains range-metadata-only. It does not allocate a `Vec`, expose raw storage, create row bytes, create tiles or RLE, publish video memory, call host present, touch Canvas / DOM / minifb, or implement fallback behavior. Success and error paths both retain the original batch owner until the caller explicitly finishes or frees it.

## Render2d row byte storage boundary

F5bz adds the first copied-byte boundary above the row batch range owner. The concrete success type is `GuiRgba8888RowByteStorageOwner`. It owns the continuation cursor, the Copy `GuiRgba8888RowBatchRange` metadata, and an exact `byte_count` scratch storage that contains a copy of the selected row bytes. It is deliberately no tile / RLE / host present; tile grouping, RLE payloads, video memory host calls, platform surfaces, and scheduling policy remain later phases.

```text
Row batch range owner
    -> revalidate range owner authority
    -> allocate exact byte_count storage
    -> copy source row bytes into scratch storage
    -> finish range owner only after full copy success
    -> Row byte storage owner
```

The source surface storage is not a public interface. `row_byte_storage` has a private sealed helper that walks the embedded owner graph and borrows the source `RegionToken u8` only inside the module. Public functions never return source `RegionToken`, `MemPtr`, split surface owners, or raw storage views. This preserves the same owner authority as the previous render2d layers while still allowing the byte copy to happen in one well-defined boundary.

Copy failure is typed. `SourceOffsetOverflow`, `DestinationIndexInvalid`, projection failures, load failure, and store failure are distinct enum variants. A failed copy attempts to deallocate the scratch storage before returning the original owner. If scratch deallocation fails, the prepare error becomes `ScratchDeallocFailed %GuiRgba8888RowByteStorageCopyErrorKind`; otherwise it becomes `CopyFailed lower_kind`. The continuation cursor is recovered only on success, so an error path cannot silently advance the scheduler.

The read helper on `GuiRgba8888RowByteStorageOwner` is a destination-copy verifier. It checks byte bounds and reads from copied storage only. It is not a source-surface escape hatch and does not make fallback behavior available.

## Render2d row tile plan boundary

F5ca introduces a metadata-only tile plan above `GuiRgba8888RowByteStorageOwner`. The success owner is `GuiRgba8888RowTilePlanOwner`. It contains the exact copied byte storage owner and a Copy `GuiRgba8888RowTilePlan`; it is explicitly no RLE / host present and does not split bytes, build payload buffers, encode RLE, publish video memory, touch Canvas / DOM / minifb, or implement fallback behavior.

```text
Row byte storage owner
    -> borrowed byte storage authority validation
    -> checked tile_rows and checked ceil tile_count
    -> Row tile plan owner
```

The byte storage authority helper is a prerequisite. F5bz stores a continuation cursor and copied range, not the original batch owner. Therefore the helper recomputes the previous batch by reading the continuation cursor index, subtracting one, validating the embedded row batch plan invariants, deriving the expected row range, and comparing every stored range field. This borrowed helper never calls `finish_cursor`, `free`, the byte reader, raw pointer helpers, or storage reads.

`descriptor_at` is intentionally borrowed:

```text
gui_rgba8888_row_tile_plan_descriptor_at:
    &GuiRgba8888RowTilePlanOwner
    -> i32
    -> Result GuiRgba8888RowTileDescriptor GuiRgba8888RowTilePlanDescriptorErrorKind
```

Borrowing matters because an application or scheduler may inspect several tile descriptors before deciding whether to pass the owner to a later payload phase. Consuming the owner during descriptor lookup would also make `finish_byte_storage` recovery impossible. Before descriptor computation, `gui_rgba8888_row_tile_plan_validate_invariants` revalidates the byte storage authority, compares plan metadata against the copied range, checks `stride_bytes == width * 4`, checks `row_start + row_count <= height`, checks `byte_count == row_count * stride_bytes`, and recomputes `tile_count == ceil(row_count / tile_rows)`.

## Render2d row tile payload view boundary

F5cb introduces `GuiRgba8888RowTilePayloadOwner` above `GuiRgba8888RowTilePlanOwner`. This owner is a tile-scoped byte payload view and a formal payload owner over existing copied row storage, not an owned payload buffer. It keeps the tile plan owner and a Copy `GuiRgba8888RowTileDescriptor`; therefore recovery can always move back to the tile plan owner and then to the byte storage owner.

The prepare path is:

```text
gui_rgba8888_row_tile_payload_prepare:
    input: GuiRgba8888RowTilePlanOwner, tile_index
    call gui_rgba8888_row_tile_plan_descriptor_at &plan tile_index
    on error:
        return GuiRgba8888RowTilePayloadPrepareError
            kind = DescriptorInvalid lower_kind
            plan = original owner
    on success:
        return GuiRgba8888RowTilePayloadOwner plan descriptor
```

The read path is:

```text
gui_rgba8888_row_tile_payload_byte_at:
    descriptor = owner.descriptor
    require 0 <= index < descriptor.byte_count
    storage_index = checked_add descriptor.byte_offset index
    storage = gui_rgba8888_row_tile_plan_storage_ref owner.plan
    call gui_rgba8888_row_byte_storage_byte_at storage storage_index
```

`gui_rgba8888_row_tile_plan_storage_ref` returns only `&GuiRgba8888RowByteStorageOwner`. It does not expose `RegionToken`, `MemPtr`, source storage, copied storage pointer, or any host handle. This keeps the abstraction at the typed owner boundary while allowing the payload view to reuse the exact copied row storage produced by F5bz.

F5cb is no RLE / host present. It does not allocate, copy, RLE-encode, publish video memory, call host present, touch Canvas / DOM / minifb, or implement fallback behavior. It exists to make the next transport phase consume a tile-scoped byte view with typed errors instead of reinterpreting descriptors ad hoc.

Descriptor offsets are explicitly storage-relative:

```text
local_row_start = tile_index * tile_rows
descriptor.row_start = plan.row_start + local_row_start
descriptor.byte_offset = local_row_start * plan.stride_bytes
descriptor.byte_count = descriptor.row_count * plan.stride_bytes
descriptor.byte_offset + descriptor.byte_count <= plan.byte_count
```

The frame-absolute row start and storage-relative byte offset are both needed. Rendering diagnostics and dirty-region accounting care about frame rows, while later payload phases need offsets into the copied row storage. This boundary carries both without exposing raw storage or reading byte values.

## Render2d row tile RLE cursor boundary

F5cc introduces `GuiRgba8888RowTileRleCursorOwner` above `GuiRgba8888RowTilePayloadOwner`. It is a streaming cursor over RGBA8888 pixel runs, not an encoded RLE buffer. The cursor owns the payload view and advances by returning a `GuiRgba8888RowTileRleStep` that contains the continuation cursor owner plus a Copy `GuiRgba8888RowTileRleRun`.

The start path is:

```text
gui_rgba8888_row_tile_rle_cursor_start:
    input: GuiRgba8888RowTilePayloadOwner
    byte_count = gui_rgba8888_row_tile_payload_byte_count &payload
    require byte_count > 0
    require byte_count % 4 == 0
    return cursor payload pixel_count next_pixel_index=0
```

Start errors are owner-bearing. `PayloadByteCountInvalid` and `PayloadByteCountNotRgbaAligned` both keep the original payload owner so the caller can free or recover the underlying tile plan and copied byte storage.

The step path is:

```text
gui_rgba8888_row_tile_rle_cursor_next_run:
    status = gui_rgba8888_row_tile_rle_cursor_status &cursor
    if status is Complete:
        return CursorComplete with cursor owner
    start = cursor.next_pixel_index
    color = read_pixel payload start
    scan until color changes or pixel_count is reached
    return continuation cursor and run metadata
```

The pixel read path is deliberately byte-view based:

```text
pixel_byte_offset = checked_mul pixel_index 4
g_offset = checked_add pixel_byte_offset 1
b_offset = checked_add pixel_byte_offset 2
a_offset = checked_add pixel_byte_offset 3
r/g/b/a = gui_rgba8888_row_tile_payload_byte_at payload offset
```

All four channel offsets are checked even though valid cursor state should keep them inside payload bounds. This keeps the public contract robust against forged owners and future alternate payload sources. Lower payload read failure is wrapped as `PayloadReadFailed lower_kind`, not flattened into a string or panic.

`GuiRgba8888RowTileRleRun` is Copy metadata: `pixel_offset`, `pixel_count`, `Rgba8888 color`. The cursor, step, and step error are owner-bearing values and do not implement Clone / Copy. `CursorComplete` is a typed error, not an unchanged cursor success, because repeated calls to `next_run` on complete cursor must have an explicit recovery path.

F5cc remains streaming-only. It does not allocate a `Vec`, build an encoded byte buffer, expose raw storage, call host present, publish video memory, touch Canvas / DOM / minifb, or implement fallback behavior. Formal tile / bitmap / row / RLE transport and host presentation remain later phases.

## Render2d row tile RLE drain boundary

F5cd introduces `GuiRgba8888RowTileRleDrainTerminal` above the F5cc cursor. It is a scheduler progress boundary, not an encoded RLE buffer and not a host presentation boundary. The drain consumes a cursor owner and a step budget, then returns either a terminal owner or an owner-bearing error.

```text
gui_rgba8888_row_tile_rle_drain_budget:
    input: GuiRgba8888RowTileRleCursorOwner
    input: remaining_steps i32
    output: Result GuiRgba8888RowTileRleDrainTerminal GuiRgba8888RowTileRleDrainError
```

The terminal is:

```text
GuiRgba8888RowTileRleDrainTerminal:
    status GuiRgba8888RowTileRleDrainStatus
    cursor GuiRgba8888RowTileRleCursorOwner
    emitted_run_count i32
```

`GuiRgba8888RowTileRleDrainStatus` is Copy metadata with `Completed` and `StepBudgetExhausted`. `GuiRgba8888RowTileRleDrainTerminal` and `GuiRgba8888RowTileRleDrainError` are owner-bearing and do not implement Clone / Copy.

The drain loop order is:

```text
status = gui_rgba8888_row_tile_rle_cursor_status &current_cursor
if status is Complete:
    return Completed terminal
if status error:
    return CursorStepFailed with current_cursor
if remaining_steps < 0:
    return InvalidBudget with current_cursor
if remaining_steps == 0:
    return StepBudgetExhausted terminal
step = gui_rgba8888_row_tile_rle_cursor_next_run current_cursor
validate discarded run metadata
advance continuation cursor
increment emitted_run_count
decrement remaining_steps
repeat
```

The key rule is status-before-budget. This keeps a complete cursor from being misreported as budget exhaustion or invalid budget. It also makes `StepBudgetExhausted` mean exactly “Ready cursor, no step budget left”.

After each successful `next_run`, the drain validates both the discarded Copy run and the continuation cursor:

```text
run.pixel_offset == previous_next_pixel_index
run.pixel_count > 0
previous_next_pixel_index + run.pixel_count == continuation.next_pixel_index
continuation.next_pixel_index <= previous_pixel_count
```

Failure is `ProgressInvariantInvalid`, not panic and not silent no-op. Lower `next_run` errors are wrapped as `CursorStepFailed lower_kind` while preserving the recovered cursor owner.

F5cd remains progress-only. It does not allocate a `Vec`, build an encoded byte buffer, mutate raw storage, call host present, publish video memory, touch Canvas / DOM / minifb, or implement fallback behavior. Formal encoded RLE transport, tile bitmap transport, and host presentation remain later owner boundaries.

## Render2d row tile RLE count boundary

F5ce adds `GuiRgba8888RowTileRleCountOwner` above the F5cd drain. The owner contains the continuation `GuiRgba8888RowTileRleCursorOwner` and an `accumulated_run_count`. This is the missing exact-count boundary between a scheduler time slice and a future encoded RLE transport allocation. It is intentionally not a run table, not a `Vec`, and not a host presentation API.

The start contract is strict:

```text
count_start cursor:
    inspect cursor_status
    Ready    -> CountOwner cursor 0
    Complete -> InitialCursorComplete error
    Invalid  -> InitialCursorInvalid lower_kind error
```

Rejecting a complete initial cursor is required. A complete cursor only says that the cursor is at the end of the pixel stream; it does not carry the number of runs already emitted. Accepting it as a fresh count owner would silently produce a false `accumulated_run_count == 0`, which would corrupt later exact allocation.

The step contract delegates all run traversal to F5cd:

```text
count_step_budget owner remaining_steps:
    accumulated = owner.accumulated_run_count
    cursor = finish owner.cursor
    drain_budget cursor remaining_steps
        Err drain_error:
            return DrainFailed lower_kind with recovered cursor and accumulated
        Ok terminal:
            next = checked_add accumulated terminal.emitted_run_count
            Completed            -> CountStep Completed CountOwner terminal.cursor next
            StepBudgetExhausted  -> CountStep Pending   CountOwner terminal.cursor next
```

`GuiRgba8888RowTileRleCountStepStatus` is Copy metadata. `GuiRgba8888RowTileRleCountStep`, `GuiRgba8888RowTileRleCountOwner`, and `GuiRgba8888RowTileRleCountError` are owner-bearing values and must not implement Clone / Copy.

`AccumulatedRunCountOverflow` is fatal for count continuation. The drain may already have advanced the cursor when the overflow is detected, so returning a `GuiRgba8888RowTileRleCountOwner` with the old count would create an inconsistent continuation state. The error therefore carries the advanced cursor plus the prior `accumulated_run_count` for teardown or restart, not for continuing the count pass.

F5ce remains count-only. It does not rescan runs with `next_run`, read payload bytes, allocate a `Vec`, build an encoded RLE buffer, expose raw storage, call host present, publish video memory, touch Canvas / DOM / minifb, or implement fallback behavior. The future encoded RLE transport must consume a successfully completed total run count as capacity evidence in a separate owner boundary.

## Render2d row tile RLE completed count boundary

F5cf adds `GuiRgba8888RowTileRleCountCompletedOwner` as that separate owner boundary. It consumes a `GuiRgba8888RowTileRleCountOwner`, borrows its cursor status through `gui_rgba8888_row_tile_rle_count_owner_cursor_status`, and only then publishes the total run count as completed evidence.

The validation order is part of the contract:

```text
count_completed_prepare count:
    count_owner_cursor_status count
        Err lower_kind -> CursorInvalid lower_kind with count owner
        Ready          -> CountNotCompleted with count owner
        Complete       -> validate accumulated_run_count > 0
            false      -> TotalRunCountInvalid with count owner
            true       -> CountCompletedOwner count total_run_count
```

The completed module does not access the count owner cursor or accumulated count through direct field inspection. `row_tile_rle_count.nepl` owns that representation and exposes borrowed helpers for status, cursor index, and accumulated count. This prevents later encoded transport code from depending on private count layout.

`GuiRgba8888RowTileRleCountCompletedErrorKind` is Copy metadata. `GuiRgba8888RowTileRleCountCompletedOwner` and `GuiRgba8888RowTileRleCountCompletedError` are owner-bearing values and must not implement Clone / Copy. The error keeps the original count owner so failed completion can be freed or inspected explicitly.

F5cf remains evidence-only. It does not call the drain, call `cursor_next_run`, read payload bytes, allocate `Vec`, build encoded RLE storage, expose raw storage, call host present, publish video memory, touch Canvas / DOM / minifb, or implement fallback behavior.

## Render2d row tile RLE encode seed boundary

F5cg adds `GuiRgba8888RowTileRleEncodeSeedOwner` as the next payload seed ownership boundary after completed count evidence. It consumes a `GuiRgba8888RowTileRleCountCompletedOwner`, validates that `total_run_count > 0`, and then closes the ownership chain back to the original payload:

```text
encode_seed_prepare completed:
    total = completed_total_run_count completed
    total <= 0 -> TotalRunCountInvalid with completed owner
    count   = completed_finish_count_owner completed
    cursor  = count_owner_finish_cursor count
    payload = cursor_finish_payload cursor
    EncodeSeedOwner payload total
```

This boundary intentionally does not call `cursor_start`. Restarting the cursor can fail with a start error that owns the payload, while the invalid-total path owns the completed owner. Mixing those two recovery owners in one slice would force a weaker error shape. F5cg therefore keeps a single owner-bearing error kind for invalid completed evidence and leaves cursor restart to the next encode-cursor or encode-writer boundary.

`GuiRgba8888RowTileRleEncodeSeedErrorKind` is Copy metadata. `GuiRgba8888RowTileRleEncodeSeedOwner` and `GuiRgba8888RowTileRleEncodeSeedError` are owner-bearing values and must not implement Clone / Copy. The error returns the original completed owner so caller recovery remains explicit.

F5cg remains payload-seed-only. It does not restart the cursor, call the drain, call `cursor_next_run`, read payload bytes, allocate `Vec`, build encoded RLE storage, expose raw storage, call host present, publish video memory, touch Canvas / DOM / minifb, or implement fallback behavior.

## Render2d row tile RLE encode cursor boundary

F5ch adds `GuiRgba8888RowTileRleEncodeCursorOwner` above the F5cg payload seed. It consumes a `GuiRgba8888RowTileRleEncodeSeedOwner`, preserves the seed's exact `total_run_count`, extracts the payload owner, and restarts the lower RLE cursor exactly once:

```text
encode_cursor_start seed:
    total = encode_seed_total_run_count seed
    payload = encode_seed_finish_payload seed
    cursor_start payload
        Err start_error -> CursorStartFailed lower_kind with start_error and total
        Ok cursor       -> EncodeCursorOwner cursor total
```

The success owner is intentionally small. It contains only the ready `GuiRgba8888RowTileRleCursorOwner` and the total run count that a future encoded writer will use as capacity evidence. It is not an encoded RLE buffer, not a run table, and not a host presentation object.

F5ch does not revalidate `total_run_count`. F5cg is the evidence boundary that rejects non-positive totals, and normal application code cannot call the seed owner constructor directly. Adding a second invalid-total path here would mix two recovery shapes: invalid total would recover the seed owner, while cursor start failure recovers the lower start error that owns the payload. F5ch therefore has a single owner-bearing failure shape for restart failure:

```text
GuiRgba8888RowTileRleEncodeCursorError:
    kind CursorStartFailed lower_start_kind
    category Option GuiError
    start_error GuiRgba8888RowTileRleStartError
    total_run_count i32
```

F5ch also does not call `cursor_status`. The F5cc `cursor_start` contract already checks that payload byte count is positive and RGBA8888-aligned, then constructs `next_pixel_index = 0` and a positive `pixel_count`. Under that contract the returned cursor is a ready cursor. A separate status validation phase can be added later if a future writer needs to validate a cursor received from a less restricted source.

`GuiRgba8888RowTileRleEncodeCursorErrorKind` is Copy metadata. `GuiRgba8888RowTileRleEncodeCursorOwner` and `GuiRgba8888RowTileRleEncodeCursorError` are owner-bearing values and must not implement Clone / Copy. F5ch remains cursor-only: it does not call `cursor_status`, drain, `cursor_next_run`, payload byte read, allocate `Vec`, build encoded RLE storage, expose raw storage, call host present, publish video memory, touch Canvas / DOM / minifb, or implement fallback behavior.

## Render2d row tile RLE writer plan boundary

F5ci converts the F5ch ready cursor owner into the first formal encoded RLE writer capacity plan. The boundary is deliberately still not a writer, not encoded storage, and not a host presentation object. Its only job is to derive exact byte capacity and preserve the cursor owner for the next phase.

The fixed row-tile RLE run layout is:

```text
pixel_offset i32
pixel_count  i32
rgba8888      4 bytes
```

Therefore one encoded run is 12 bytes and the encoded payload capacity is:

```text
encoded_byte_count = total_run_count * 12
```

This multiplication is checked. A forged negative count, zero count, or overflowing byte count must not become a silent empty payload and must not fall back to an uncompressed transport.

```text
GuiRgba8888RowTileRleWriterPlanOwner:
    cursor GuiRgba8888RowTileRleCursorOwner
    total_run_count i32
    encoded_byte_count i32

GuiRgba8888RowTileRleWriterPlanError:
    kind GuiRgba8888RowTileRleWriterPlanErrorKind
    category Option GuiError
    ready GuiRgba8888RowTileRleEncodeCursorOwner
    total_run_count i32
```

`TotalRunCountInvalid` and `EncodedByteCountOverflow` both return the original `GuiRgba8888RowTileRleEncodeCursorOwner`. F5ci must validate and compute capacity before calling `gui_rgba8888_row_tile_rle_encode_cursor_owner_finish_cursor`. On success, the ready owner is consumed exactly once and the underlying cursor owner is moved into `GuiRgba8888RowTileRleWriterPlanOwner`.

F5ci intentionally revalidates `total_run_count > 0`. F5cg already validates normal public construction, but F5ci is the transport capacity boundary. Revalidation keeps the later writer fail-closed if an internal constructor path or future refactor produces a forged owner.

F5ci must not call `cursor_status`, drain the cursor, call `cursor_next_run`, read payload bytes, allocate `Vec`, allocate raw storage, build `EncodedRleBuffer`, call host present, publish video memory, touch Canvas / DOM / minifb, or implement fallback behavior. The actual encoded byte writer and storage owner are separate later owner boundaries.

## Render2d row tile RLE encoded storage boundary

F5cj converts the F5ci writer plan into the first owned byte storage for a future encoded RLE writer. This phase is allocation / reservation only. It does not inspect runs, does not write the RLE payload, and does not create a transport frame.

```text
GuiRgba8888RowTileRleStorageOwner:
    cursor GuiRgba8888RowTileRleCursorOwner
    total_run_count i32
    encoded_byte_count i32
    storage RegionToken u8

GuiRgba8888RowTileRleStoragePrepareError:
    kind GuiRgba8888RowTileRleStoragePrepareErrorKind
    category Option GuiError
    plan GuiRgba8888RowTileRleWriterPlanOwner
```

`GuiRgba8888RowTileRleStoragePrepareErrorKind` contains `EncodedByteCountInvalid`, `TotalRunCountInvalid`, `EncodedByteCountOverflow`, `EncodedByteCountMismatch`, and `AllocationFailed`. Every prepare error keeps the original `GuiRgba8888RowTileRleWriterPlanOwner`. This is important because allocation is the first phase that can fail due to resource pressure; losing the plan owner would leak the underlying cursor and payload chain.

The prepare order is fixed:

```text
encoded_byte_count = plan.encoded_byte_count
reject encoded_byte_count <= 0
total_run_count = plan.total_run_count
reject total_run_count <= 0
recomputed = checked_mul total_run_count 12
reject overflow
reject recomputed != encoded_byte_count
storage = alloc_region_bytes encoded_byte_count
reject allocation failure
cursor = finish_cursor plan
StorageOwner cursor total_run_count encoded_byte_count storage
```

The allocation happens before `finish_cursor plan`. That order preserves owner recovery on allocation failure and on metadata mismatch. The plan owner is consumed only after a storage token exists and all capacity evidence has been revalidated.

`gui_rgba8888_row_tile_rle_storage_finish_cursor` deallocates `storage` before returning the continuation cursor. If storage deallocation fails, the error stores the cursor so the caller can still decide how to free the payload chain. `gui_rgba8888_row_tile_rle_storage_owner_free` then frees the cursor after successful deallocation, and maps cursor-free failures to `CursorFreeFailed`.

F5cj must not call `cursor_next_run`, drain the cursor, read payload bytes, call `load_u8` / `store_u8`, expose a raw storage accessor, allocate `Vec`, build `EncodedRleBuffer`, call host present, publish video memory, touch Canvas / DOM / minifb, or implement fallback behavior. The later run writer must be a separate owner boundary that writes success before advancing the cursor.

## Render2d row tile RLE run writer cursor boundary

F5ck converts `GuiRgba8888RowTileRleStorageOwner` into `GuiRgba8888RowTileRleWriteCursorOwner`. This is the first phase that writes encoded RLE bytes, but it still does not expose the bytes to callers and does not publish a transport frame.

```text
GuiRgba8888RowTileRleWriteCursorOwner:
    cursor GuiRgba8888RowTileRleCursorOwner
    total_run_count i32
    encoded_byte_count i32
    storage RegionToken u8
    written_run_count i32
    written_byte_count i32

GuiRgba8888RowTileRleWriteStepStatus:
    WroteRun
    Completed
```

`gui_rgba8888_row_tile_rle_write_cursor_start` revalidates `encoded_byte_count > 0`, `total_run_count > 0`, and `total_run_count * 12 == encoded_byte_count` before moving the cursor and storage into the writer owner. Start does not inspect payload bytes and does not write storage bytes.

F5ck adds two lower cursor helpers to `row_tile_rle`:

```text
gui_rgba8888_row_tile_rle_cursor_peek_run
    borrows cursor and returns Copy run metadata

gui_rgba8888_row_tile_rle_cursor_advance_by_run
    consumes cursor only after caller-side success
```

`peek_run` returns only `GuiRgba8888RowTileRleStepErrorKind` and does not move the owner. `advance_by_run` validates that the run starts at the current cursor position, has positive `pixel_count`, and ends within the pixel payload. Failure returns owner-bearing `GuiRgba8888RowTileRleStepError` with the original cursor.

`gui_rgba8888_row_tile_rle_write_cursor_step_one` must follow this order:

```text
validate stored counts and written counts
if written_run_count == total_run_count:
    require lower cursor Complete
else:
    check written_byte_count + 12 <= encoded_byte_count
    peek run by borrow
    write 12 bytes to current slot
    advance cursor by run
    increment written_run_count and written_byte_count
```

The encoded layout is pinned:

```text
pixel_offset i32 little-endian
pixel_count i32 little-endian
Rgba8888 r,g,b,a
```

Store / projection failure returns an owner-bearing `GuiRgba8888RowTileRleWriteStepError` with unchanged `written_run_count` and `written_byte_count`. Some bytes in the target slot may have been written before the failure; that slot is uncommitted, has no public reader, and retry overwrites all 12 bytes. This is not a fallback path.

F5ck must not call consuming `cursor_next_run`, read payload bytes directly, call `load_u8`, expose an encoded byte reader, allocate `Vec`, build a host frame, publish video memory, touch Canvas / DOM / minifb, or implement fallback behavior. `region_ptr_at` and `store_u8` are confined to byte projection / byte store helpers inside the writer module.

## Render2d row tile RLE sealed encoded owner boundary

F5cl converts a completed `GuiRgba8888RowTileRleWriteCursorOwner` into `GuiRgba8888RowTileRleEncodedOwner`. This is still not host present and still not a byte reader. It is a sealing gate that proves the writer cursor is complete before later tile transport can consider the storage host-visible.

```text
GuiRgba8888RowTileRleEncodedOwner:
    cursor GuiRgba8888RowTileRleCursorOwner
    total_run_count i32
    encoded_byte_count i32
    storage RegionToken u8
```

The seal function validates structural count invariants before it looks at lower cursor status:

```text
encoded_byte_count > 0
total_run_count > 0
total_run_count * 12 == encoded_byte_count
0 <= written_run_count <= total_run_count
0 <= written_byte_count <= encoded_byte_count
written_run_count * 12 == written_byte_count
written_run_count == total_run_count
written_byte_count == encoded_byte_count
lower cursor status == Complete
```

`WrittenByteCountMismatch` is used when `written_run_count * 12` does not match `written_byte_count`. `WriterNotComplete` is used when the counts are internally consistent but have not reached total / encoded completion. Lower cursor `Ready` is `CursorNotComplete`. Lower cursor status failure is wrapped as `CursorStatusInvalid`.

Every seal failure returns the original writer owner in `GuiRgba8888RowTileRleEncodedSealError`. The implementation must not reconstruct a fake owner because the storage and lower cursor are still linear resources. Seal success moves cursor and storage into the encoded owner. The encoded owner exposes only metadata accessors and teardown helpers.

F5cl must not expose storage pointer, read encoded bytes, call `region_ptr_at`, call `load_u8`, call `store_u8`, allocate `Vec`, build a host frame, publish video memory, touch Canvas / DOM / minifb, or implement fallback behavior. Formal tile transport and host present remain later boundaries.

## Render2d row tile RLE packet owner boundary

F5cm introduces the first packet-shaped owner after the sealed encoded owner. `GuiRgba8888RowTileRlePacketOwner` is still an alloc/render2d owner, not a platform surface and not a host call. It keeps `GuiRgba8888RowTileRleEncodedOwner` together with `GuiRgba8888RowTileRlePacketDescriptor`.

```text
GuiRgba8888RowTileRlePacketDescriptor:
    frame_id i32
    batch_index i32
    tile_index i32
    plan_row_start i32
    plan_row_count i32
    row_start i32
    row_count i32
    width i32
    height i32
    stride_bytes i32
    tile_rows i32
    tile_count i32
    pixel_count i32
    total_run_count i32
    encoded_byte_count i32
```

The packet descriptor is a transport contract seed. Later Web, native, bare, and headless backends can map it to a formal tile / bitmap transport without reinterpreting private cursor or payload layout. The descriptor does not contain a pointer, byte slice, JavaScript object handle, native handle, or video-memory id. `plan_row_start` and `plan_row_count` are deliberately carried because std-layer presentation must be able to rederive `tile_count` from `plan_row_count` and `tile_rows` without borrowing private plan owners.

The important new authority boundary is payload descriptor validation. F5cm adds metadata-only helpers so that packet construction does not poke at private layout ad hoc:

```text
gui_rgba8888_row_tile_payload_validate_descriptor_authority
gui_rgba8888_row_tile_payload_descriptor_checked
gui_rgba8888_row_tile_payload_plan_metadata_checked
gui_rgba8888_row_tile_rle_cursor_payload_descriptor_checked
gui_rgba8888_row_tile_rle_cursor_payload_plan_metadata_checked
gui_rgba8888_row_tile_rle_encoded_tile_descriptor_checked
gui_rgba8888_row_tile_rle_encoded_tile_plan_metadata_checked
```

`gui_rgba8888_row_tile_payload_validate_descriptor_authority` recomputes the descriptor from `GuiRgba8888RowTilePlanOwner` by calling `gui_rgba8888_row_tile_plan_descriptor_at` with the stored tile index and then compares tile index, row start, row count, byte offset, and byte count. This keeps forged payload descriptors from becoming packet authority.

Packet prepare validates in a fixed order: encoded count, total run count, checked `total_run_count * 12`, cursor completion, descriptor authority, `cursor_pixel_count * 4 == descriptor_byte_count`, `width * 4 == stride_bytes`, `row_count * stride_bytes == descriptor_byte_count`, checked row extent inside surface height, derived tile count, and tile index range. `PayloadDescriptorInvalid` wraps the lower authority error. All failure paths return the original sealed owner in `GuiRgba8888RowTileRlePacketPrepareError`; the owner moves into `GuiRgba8888RowTileRlePacketOwner` only after every check succeeds.

F5cm must not expose encoded bytes, call raw memory projection, allocate `Vec`, call host present, publish video memory, touch Canvas / DOM / minifb, or implement fallback behavior. The later formal transport phase must consume the packet owner rather than rebuild packet metadata from private lower owners.

## std layer row tile RLE present-frame owner

F5cn introduces the first std-layer owner for row tile RLE presentation. It consumes `GuiRgba8888RowTileRlePacketOwner` together with checked `SurfaceId` and `FrameId`, but it still does not call a host import. This keeps the standard boundary explicit: alloc/render2d proves the packet, std/gui proves the packet belongs to a surface/frame, and platform backends later consume the std owner.

```text
GuiRgba8888RowTileRlePresentDescriptor:
    surface SurfaceId
    frame FrameId
    packet GuiRgba8888RowTileRlePacketDescriptor

GuiRgba8888RowTileRlePresentFrameOwner:
    packet GuiRgba8888RowTileRlePacketOwner
    descriptor GuiRgba8888RowTileRlePresentDescriptor
```

The prepare function validates `surface_id_raw > 0`, `frame_id_raw > 0`, packet descriptor frame id equality, positive width / height / row counts / tile counts, plan row extent, tile row extent, stride `width * 4`, rederived tile count from `plan_row_count` and `tile_rows`, tile index range, tile pixel count `row_count * width`, and encoded byte count `total_run_count * 12`. All multiplication and addition use checked arithmetic. Failure is owner-bearing and returns the original packet owner through `GuiRgba8888RowTileRlePresentFramePrepareError`.

F5cn explicitly does not extend `GuiSurfacePresentCommand`, does not build `PresentPixelFrame`, and does not create `GuiPixelBufferDescriptor`. Those values are the older pixel-frame command boundary, not the tile RLE host import boundary. The future Web/native/headless presenter must consume `GuiRgba8888RowTileRlePresentFrameOwner` and define the real host import shape there.

## Row tile RLE packet typed record reader and present run cursor

F5co introduces the first quarantined typed record reader for the row tile RLE transport. The row tile RLE packet typed record reader is `alloc/gui/render2d/row_tile_rle_packet_record`, not an extension of `row_tile_rle_packet` or `row_tile_rle_encoded`. This preserves the earlier no-reader contract of those owners while giving presenter code a formal typed drain path.

```text
GuiRgba8888RowTileRlePacketRecordReadErrorKind:
    TotalRunCountInvalid
    EncodedByteCountInvalid
    EncodedByteCountOverflow
    EncodedByteCountMismatch
    PixelCountInvalid
    RecordIndexNegative
    RecordIndexOutOfBounds
    RecordBaseOverflow
    RecordByteOffsetOverflow
    RecordByteOutOfBounds
    PointerProjectionFailed
    ByteLoadFailed
    ByteValueOutOfRange
    DecodedI32Negative
    ChannelOutOfRange
    RunPixelOffsetNegative
    RunPixelCountInvalid
    RunEndOverflow
    RunEndOutOfBounds
```

The public read function borrows `&GuiRgba8888RowTileRlePacketOwner` and a record index, then returns `Result GuiRgba8888RowTileRleRun GuiRgba8888RowTileRlePacketRecordReadErrorKind`. It revalidates `total_run_count > 0`, `encoded_byte_count > 0`, `total_run_count * 12 == encoded_byte_count`, positive packet pixel count, record index range, record byte range, non-negative little-endian i32 values, RGBA channel range, and decoded run extent within packet pixel count.

This is not a general byte reader. It is a quarantined typed record reader: raw `RegionToken u8`, `MemPtr`, `region_ptr_at`, and `load_u8` appear only in private helpers that convert one exact record into a typed run. There is no raw storage accessor, no public byte-at API, no `Vec`, no host present, no video memory call, no Canvas / DOM / minifb dependency, and no fallback behavior.

F5co also adds `std/gui/tile_present_run_cursor` as the std-layer owner above `GuiRgba8888RowTileRlePresentFrameOwner`:

```text
GuiRgba8888RowTileRlePresentRunCursorOwner:
    present GuiRgba8888RowTileRlePresentFrameOwner
    next_record_index i32
    total_run_count i32

GuiRgba8888RowTileRlePresentRunCursorStepResult:
    RunReady GuiRgba8888RowTileRleRun
    Completed
```

The cursor start boundary revalidates the present descriptor count invariant and moves the present owner only on success. The step boundary treats `next_record_index == total_run_count` as explicit `Completed`, rejects `next_record_index > total_run_count` as `RecordIndexPastEnd`, and wraps lower packet-record read failures as `PacketRecordReadFailed`. A successful `RunReady` step advances `next_record_index` only after the typed record reader succeeds.

The std cursor does not call host imports and does not touch raw memory. Web, native, bare, and headless presenters can later consume this cursor and choose their transport mechanism, but no presenter is allowed to bypass the typed record reader by reaching into packet storage directly.

## Std layer row tile RLE present command cursor

F5cp introduces the std layer row tile RLE present command cursor. It is still not a host import. It is the presenter-facing frame stream that sits above the F5co run cursor and below Web / native / bare / headless host presenters.

```text
GuiRgba8888RowTileRlePresentCommand:
    BeginFrame GuiRgba8888RowTileRlePresentDescriptor
    Run GuiRgba8888RowTileRleRun
    EndFrame GuiRgba8888RowTileRlePresentDescriptor

GuiRgba8888RowTileRlePresentCommandCursorOwner:
    run_cursor GuiRgba8888RowTileRlePresentRunCursorOwner
    descriptor GuiRgba8888RowTileRlePresentDescriptor
    phase GuiRgba8888RowTileRlePresentCommandCursorPhase
```

The cursor keeps one typed output per public step. `BeginPending` emits `GuiRgba8888RowTileRlePresentCommand::BeginFrame` and moves to `RunPending`. `RunPending` calls F5co exactly through `gui_rgba8888_row_tile_rle_present_run_cursor_step`. Lower `RunReady` emits `Run` and keeps `RunPending`. Lower `Completed` emits `GuiRgba8888RowTileRlePresentCommand::EndFrame` in the same public step and moves to `Completed`. A later step in `Completed` returns terminal `Completed`.

Start and step errors are owner-bearing. Lower start failure recovers the present owner through the F5co start error finish helper. Lower step failure recovers the lower run cursor owner, rebuilds `GuiRgba8888RowTileRlePresentCommandCursorOwner` with the saved descriptor and `RunPending`, and returns an owner-bearing command cursor error. The command cursor does not bypass F5co: it must not call the packet record reader, packet storage, `RegionToken`, `MemPtr`, byte load helpers, host imports, platform APIs, or fallback paths directly.

## Std layer row tile RLE present host-command record

F5cq introduces the std layer row tile RLE present host-command record. It is still not a host import and does not submit a frame. It converts the F5cp command-cursor step output into a record shape that Web, native, bare, and headless presenters can receive through a later formal ABI.

```text
GuiRgba8888RowTileRlePresentHostCommandRecord:
    BeginFrame GuiRgba8888RowTileRlePresentDescriptor
    RunRecord GuiRgba8888RowTileRlePresentHostCommandRunRecord
    EndFrame GuiRgba8888RowTileRlePresentDescriptor

GuiRgba8888RowTileRlePresentHostCommandRunRecord:
    descriptor GuiRgba8888RowTileRlePresentDescriptor
    run GuiRgba8888RowTileRleRun

GuiRgba8888RowTileRlePresentHostCommandStepResult:
    Record GuiRgba8888RowTileRlePresentHostCommandRecord
    Completed
```

The record shape does not flatten to kind plus optional run. A host presenter can pattern-match a single enum and cannot observe an invalid state such as a `RunRecord` without a run payload or an `EndFrame` with one. The mapping function reads the descriptor through F5cp's public step descriptor accessor and the step output through F5cp's public step result accessor. It does not access the F5cp step owner field directly and does not bypass F5cp by reading F5co, packet records, raw storage, host imports, platform APIs, or fallback paths.

## Std layer row tile RLE present run-span boundary

F5df introduces the std layer row tile RLE present run-span boundary. It consumes an F5cq `GuiRgba8888RowTileRlePresentHostCommandRunRecord` and converts its tile-local linear pixel offset into row-contained spans before any Web, native, bare, or headless presenter reaches platform import code.

```text
GuiRgba8888RowTileRlePresentRunRowSpan:
    x i32
    y i32
    width i32
    color Rgba8888

GuiRgba8888RowTileRlePresentRunSpanCursor:
    record GuiRgba8888RowTileRlePresentHostCommandRunRecord
    next_pixel_offset i32
    remaining_pixel_count i32

GuiRgba8888RowTileRlePresentRunSpanStepResult:
    SpanReady GuiRgba8888RowTileRlePresentRunSpanReady
    Completed
```

The span is a dedicated row value, not a platform rectangle. It does not store a height field; `gui_rgba8888_row_tile_rle_present_run_row_span_height` returns 1 so consumers can fill a one-row rectangle without weakening the invariant. The offset contract is tile-local linear pixel offset. For every emitted span, the cursor computes `local_row = offset / width`, `x = offset % width`, and `y = row_start + local_row`. The span width is the smaller of the remaining run pixels and the remaining pixels in that row, so a run crossing a row boundary is split instead of stretching a fill across rows.

`start` performs descriptor and run validation before constructing the cursor. It rejects non-positive width or height, negative row start, non-positive row count, row extent overflow, row extent outside height, row count greater than tile rows, `row_count * width` overflow, descriptor pixel count mismatch, negative run offset, non-positive run count, run end overflow, and run end outside descriptor pixel count. All failures are enum `Result` values; no fallback or clamping is allowed. `step` revalidates cursor consistency enough to catch forged state, returns explicit Completed when the remaining count is zero, and otherwise returns exactly one span plus the next cursor. F5df does not call platform import and does not reach F5da-F5de action drivers, F5cs virtual drain, F5cp/F5co lower cursors, packet record readers, raw storage, queues, schedulers, DOM, Canvas, minifb, video memory, DrawTarget, RenderTarget, fallback paths, or silent no-op behavior.

F5cr introduces the std layer row tile RLE present host import request. It is still not the Web, native, or bare presenter implementation. It wraps an F5cq `GuiRgba8888RowTileRlePresentHostCommandRecord` into `GuiRgba8888RowTileRlePresentHostImportRequest` and selects an explicit `GuiRgba8888RowTileRlePresentHostImportTarget`.

The target enum contains only `Window WindowId`, `Offscreen`, and `Device`. Headless is not a presentation target. Headless tests should inspect host-command records or a later explicit virtual drain, not receive a fake presentation target. Text grid is also rejected because RGBA8888 row tile RLE is a pixel transport. The constructor checks `GuiCapabilities.color_format` before selecting a target and accepts only `ColorFormat::FormatRgba8888`. This is required because `SurfaceKind::DevicePixel` can use non-RGBA formats such as RGB565. The request boundary must fail with `GuiError::Unsupported` rather than shifting a color-format mismatch to a platform layer.

F5cs introduces the std layer row tile RLE present virtual drain. It is the explicit headless/test observation boundary for the F5cq host-command record stream. It does not present pixels, does not build a host import request, and does not consume F5cr.

The virtual drain keeps `GuiRgba8888RowTileRlePresentVirtualDrain` state as a small Copy value: phase, optional surface id, optional frame id, expected run count, seen run count, expected pixel count, and seen pixel count. BeginFrame is valid only in the initial phase and stores expected counts through std-layer `tile_present` descriptor accessors. RunRecord is valid only in the frame phase and requires `run_pixel_offset == seen_pixel_count`; this rejects gaps, overlaps, and reordered runs even when total run count and total pixel count would otherwise match. EndFrame is valid only after all expected runs and pixels are observed. Failures return a concrete `GuiRgba8888RowTileRlePresentVirtualDrainErrorKind` plus the previous drain state so a test harness can inspect the exact invalid transition without falling back to a presenter.

## Std layer row tile RLE present schedule boundary

F5ct introduces the std layer row tile RLE present schedule boundary. It sits above F5cq host-command records and F5cs virtual drain, and below actual Web / native / bare host import dispatch. The purpose is not to execute timers or enqueue commands. The purpose is to make a deterministic time-slice decision from a checked record stream before a platform presenter receives the record.

`GuiRgba8888RowTileRlePresentScheduleState` contains the F5cs virtual drain value plus two slice-local counters: command count and pixel count. F5ct deliberately uses F5cs virtual drain as the single stream-validation authority. Begin / Run / End ordering, descriptor consistency, expected count completion, and `run_pixel_offset == seen_pixel_count` are not reimplemented in the scheduler. This prevents the scheduler from becoming a second, weaker validator.

`GuiRgba8888RowTileRlePresentSchedulePolicy` carries `max_commands_per_slice` and `max_pixels_per_slice`. Both are positive values. Invalid policy values return `Result::Err` with enum kinds; the constructor must not clamp or infer defaults. `Yield means exact slice budget`: if the current valid record is consumed and the updated command count or pixel count is exactly equal to the policy budget, the step returns `Yield` with the updated state. If the updated counts exceed a budget, over-budget is a typed error and the previous schedule state is preserved. A single RunRecord whose pixel count is larger than `max_pixels_per_slice` is also a typed error; it must not be converted into a yielded slice because that would make the pixel budget non-authoritative.

The `Completed` decision is returned when F5cs reaches `Ended`. Completion wins over budget comparison for that record because EndFrame represents stream termination, not more pixel work. The implementation still uses checked arithmetic before producing the updated state. `resume_slice` resets only slice-local counters and preserves the F5cs drain state. F5ct must not allocate a queue, call a timer, construct F5cr requests, read raw packet storage, call host imports, expose platform API, or fallback to a silent no-op path.

## Std layer row tile RLE present scheduled dispatch boundary

F5cu introduces the std layer row tile RLE present scheduled dispatch boundary. It joins F5ct schedule state with F5cr host import request construction, but it still does not execute the host import. This is the last typed preparation value before a Web, native, bare, or offscreen presenter can consume one request.

`GuiRgba8888RowTileRlePresentDispatchState` contains only `GuiRgba8888RowTileRlePresentScheduleState`. It does not own a queue, timer, platform handle, raw packet cursor, video memory surface, or host import state. The step function enforces F5ct before F5cr: schedule validation and budget decision happen first, then request construction happens only for a valid scheduled record.

The success shape is intentionally `RequestReady request plus post phase`. `GuiRgba8888RowTileRlePresentDispatchReadyRequest` carries the `GuiRgba8888RowTileRlePresentHostImportRequest` and a `GuiRgba8888RowTileRlePresentDispatchPostPhase`. This avoids an invalid `Option request + phase` state and avoids losing information when a valid record exactly reaches a slice budget or completes the frame. A RunRecord that reaches the budget is still delivered as a request with `Yield`. An EndFrame is still delivered as a request with `Completed`.

All errors preserve previous dispatch state. F5ct errors wrap the lower schedule error kind and category. F5cr errors wrap the host request `GuiError` and do not adopt the schedule state that was computed in the same pure step, because no request was produced. F5cu must not call F5cs directly, bypass F5ct, call F5cp/F5co lower cursors, read raw packet storage, allocate queues, invoke timers, execute host imports, expose platform APIs, or fallback to a silent no-op path.

## Std layer row tile RLE present dispatch loop outcome boundary

F5cv introduces the std layer row tile RLE present dispatch loop outcome boundary. It sits immediately above F5cu and still does not execute a host import. Its job is to make the future Web, native, bare, or offscreen presenter interaction explicit: first create a pending request, then complete that pending request exactly once with the host outcome.

`GuiRgba8888RowTileRlePresentDispatchLoopState` wraps only `GuiRgba8888RowTileRlePresentDispatchState`. `GuiRgba8888RowTileRlePresentDispatchLoopPendingRequest` stores previous state, next state, `GuiRgba8888RowTileRlePresentHostImportRequest`, and post phase. The pending value is intentionally not Clone and not Copy. `complete_request consumes pending`, so a host outcome cannot be replayed by repeatedly completing a borrowed request. This keeps request submission as a one-shot state-machine boundary without adding a queue or scheduler.

`dispatch_loop_step_record` calls only F5cu `gui_rgba8888_row_tile_rle_present_dispatch_step_record`. A dispatch error is wrapped with the lower error kind, category, and previous state. A success returns a pending request that contains both rollback information and the state that should become visible after the executor succeeds.

`complete_request` consumes that pending value and a `Result unit GuiError` outcome supplied by a later platform executor. An Err outcome produces `HostImportExecutionFailed` and returns previous state in the error. An Ok outcome maps post phase to `Continue next state`, `Yield next state`, or `Completed next state`. F5cv must not call F5ct, F5cr, or F5cs directly, touch lower cursors, read raw packet storage, allocate queues, invoke timers or schedulers, execute host imports, expose platform APIs, or fallback to a silent no-op path.

## Std layer row tile RLE present host execution action boundary

F5cw introduces the std layer row tile RLE present host execution action boundary. It sits above F5cr and below actual Web, native, bare, or offscreen executor code. It still does not execute host imports. Its job is to turn a validated `GuiRgba8888RowTileRlePresentHostImportRequest` into a `GuiRgba8888RowTileRlePresentHostExecutionAction` that a backend can match without interpreting nested request and record state itself.

The action enum is a flat target x record action. It contains `WindowBegin`, `WindowRun`, `WindowEnd`, `OffscreenBegin`, `OffscreenRun`, `OffscreenEnd`, `DeviceBegin`, `DeviceRun`, and `DeviceEnd`. The window variants carry payload structs because they must preserve both `WindowId` and the descriptor or run record. Offscreen and Device encode the target in the variant name and carry the descriptor or run record directly.

The mapping function reads only through F5cr request accessor functions and F5cq host-command record values. It returns an action directly because F5cr has already validated target capability and color format; adding a new `Result` here would imply a failure mode that this boundary does not own. Executor failure remains a `Result unit GuiError` returned by the backend and consumed by F5cv `complete_request`. F5cw must not call F5cv, F5cu, F5ct, F5cs, F5cp, or F5co, construct a new F5cr request, read raw packet storage, allocate a queue, invoke timers or schedulers, touch host execution APIs, expose platform APIs, or fallback to a silent no-op path.

## Std layer row tile RLE present host span operation boundary

F5dg introduces the std layer row tile RLE present host span operation boundary. It consumes F5cw `GuiRgba8888RowTileRlePresentHostExecutionAction` values and produces a presenter-neutral `GuiRgba8888RowTileRlePresentHostSpanOperation` stream. This is still not the Web canvas, native framebuffer, bare display, or offscreen storage implementation. It is the last std-layer normalization step that makes Begin / End one-shot operations and Run row spans look identical to every presenter target.

The cursor is `GuiRgba8888RowTileRlePresentHostSpanOperationCursor`. Its phase is an invalid-state-free enum with only `SinglePending operation`, `RunPending target run_span_cursor`, and `Completed`. Begin and End actions start as SinglePending operation, step once to return `OperationReady operation next_cursor`, and then the next cursor returns explicit Completed. Run actions call F5df `run_span_start` exactly once in `start action`, store the returned F5df run-span cursor in `RunPending target run_span_cursor`, and never restart the run during public `step`.

`step cursor` calls F5df `run_span_step` at most once. A lower SpanReady is mapped to `WindowRunSpan`, `OffscreenRunSpan`, or `DeviceRunSpan` by preserving the target in the cursor. A lower Completed becomes F5dg Completed. F5df start failures become F5dg start errors carrying the original F5cw action, category, and lower error. F5df step failures become F5dg step errors carrying the current operation cursor, category, and lower error so the caller can recover without losing state. F5dg must not touch actual host import execution, F5da-F5de action drivers, F5cs virtual drain, F5cp/F5co lower cursors, packet record readers, raw storage, platform APIs, DOM, Canvas, minifb, video memory, DrawTarget, RenderTarget, queues, schedulers, fallback paths, or silent no-op behavior.

## Std layer row tile RLE present scheduled span operation boundary

F5dh introduces the std layer row tile RLE present scheduled span operation boundary. It sits after F5dg and before any actual Web, native, bare, or headless presenter consumes operations. The purpose is to apply a deterministic slice budget to the presenter-neutral F5dg operation stream without executing the presenter and without reusing F5ct as a second authority. The yield rule is exact budget only, and `resume_slice` keeps the F5dg cursor while resetting only slice counters.

F5ct and F5dh intentionally have different cost models. F5ct schedules F5cq host-command records before host import request construction. A RunRecord is counted as one record and uses the full run pixel count as its pixel cost. F5dg later splits that same Run action into one or more row spans. Reusing F5ct state after F5dg would either charge the same Run twice or reject large runs before they can be sliced. F5dh therefore owns a new `GuiRgba8888RowTileRlePresentScheduledSpanOperationPolicy` with `max_operations_per_slice` and `max_pixels_per_slice`, and a new `GuiRgba8888RowTileRlePresentScheduledSpanOperationState` with only the F5dg cursor and slice-local counters.

The stream authority is F5dg. `start action` calls F5dg start once and stores the returned cursor. `step policy state` validates the policy, reads the stored cursor, and calls F5dg step at most once. Completed is a terminal result and does not increment counters. OperationReady computes cost, checks the budget, then returns a ready value containing the operation, post phase, and next state together. This mirrors the F5cu lesson: exact-budget delivery must not be split from the value that needs to be executed.

Begin and End operations have operation cost 1 and pixel cost 0. RunSpan operations have operation cost 1 and pixel cost `span.width * span.height`, computed through F5df accessors with checked multiplication. A non-positive span extent, arithmetic overflow, single span exceeding the pixel budget, total counter overflow, total budget overflow, or lower F5dg failure is a typed error. `resume_slice` preserves the F5dg cursor and resets only the slice counters. F5dh must not call F5cs, F5ct, F5cu, F5da-F5de, host import constructors, raw packet readers, raw storage, platform APIs, DOM, Canvas, minifb, video memory, DrawTarget, RenderTarget, queues, timers, fallback paths, or silent no-op behavior.

## Std layer row tile RLE present host span operation attempt boundary

F5di introduces the std layer row tile RLE present host span operation attempt boundary. It sits after F5dh and before any completion, queue, or platform specific presenter machinery. The actual presenter reports which span operation it attempted and the caller supplied outcome it observed. The std layer then checks that this attempt still belongs to the scheduled ready value that produced the work.

The attempt value is deliberately small. `GuiRgba8888RowTileRlePresentHostSpanOperationAttempt` contains only the attempted `GuiRgba8888RowTileRlePresentHostSpanOperation` and a caller supplied outcome of type `Result unit GuiError`. F5di does not manufacture success, does not convert support failures into presenter failures, and does not synthesize platform errors. Unsupported target and mismatched operation are wrapper errors for the association boundary itself.

The validation order is support before equality before success. Support uses the F5cy `GuiRgba8888RowTileRlePresentHostExecutorSupport` enum only as a target support set. F5di must not call F5cy action validation or action equality because F5di works on F5dg span operations, not F5cw actions. Operation equality covers all 9 variants. Window variants compare `window_id_raw`. Begin and End compare descriptor identity through public descriptor and packet accessors. RunSpan compares x, y, width, height, and RGBA channels through public span and color accessors. RunSpan has no descriptor payload, so descriptor comparison is not required for RunSpan.

Unsupported and mismatch errors keep both the original scheduled ready and the attempted operation value. This preserves the F5dh next state and `Yield` phase for replay or diagnostics. Yield phase is data only in F5di. The boundary must not resume the scheduler, enqueue work, request timers, mutate a platform surface, call DOM, Canvas, minifb, video memory, raw storage, fallback paths, or silent no-op behavior.

## Std layer row tile RLE present host span operation completion boundary

F5dj introduces the std layer row tile RLE present host span operation completion boundary. It sits after F5di and before any real scheduler or platform presenter backend loop. Its input is AttemptStep only: a `GuiRgba8888RowTileRlePresentHostSpanOperationAttemptStep` already proves that support validation and operation identity association succeeded. F5dj therefore does not re-run F5di association validation and does not reinterpret F5cw actions.

`GuiRgba8888RowTileRlePresentHostSpanOperationCompletion` has only `Continue state` and `Yield state`. F5dh `Completed` is a terminal result with no operation attached, so per-operation completion does not create Completed. This distinction prevents a successful single operation from being mistaken for end-of-stream completion.

The completion order is outcome before phase publication. F5dj reads the attempt outcome through F5di public accessors. If the outcome is `Err host_error`, host outcome failure does not publish state. Instead, `GuiRgba8888RowTileRlePresentHostSpanOperationCompletionHostFailed` keeps `Some host_error`, the original ready value, and the attempt value. If the outcome is `Ok`, F5dj reads ready phase and ready state through F5dh public accessors and maps the phase to `Continue state` or `Yield state`.

F5dj must not call F5di `attempt_step`, F5dh `start`, `step`, or `resume_slice`, F5cs / F5ct / F5cu, F5cy action validation, F5cw action equality, F5da-F5de action drivers, host imports, platform APIs, DOM, Canvas, minifb, video memory, raw storage, queues, timers, schedulers, fallback paths, or silent no-op behavior.

## Std layer row tile RLE present host span operation presenter step boundary

F5dk introduces the std layer row tile RLE present host span operation presenter step boundary. It is the shared return-path contract for actual Web, native, bare, and headless presenter wrappers after they have attempted exactly one scheduled span operation. The boundary consumes only a support set, the F5dh ready value, and a presenter supplied attempt. It does not execute host imports, does not allocate platform resources, and does not synthesize success or failure outcomes.

The ordering rule is F5di before F5dj. `gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_step` first calls F5di `gui_rgba8888_row_tile_rle_present_host_span_operation_attempt_step`. If F5di rejects the attempt, F5dk returns `AttemptRejected` with support, ready, attempt, lower error, and the category obtained through F5di public category accessors. Only the F5di `Ok attempt_step` branch may call F5dj `gui_rgba8888_row_tile_rle_present_host_span_operation_completion_step`.

If F5dj rejects the attempt step, F5dk returns `CompletionRejected` with the attempt step, lower F5dj error, and the category obtained through F5dj public category accessors. Keeping the attempt step avoids forcing callers to decode lower variants just to recover ready and attempt context. On success, `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterStep` stores the F5dj completion step.

F5dk must not create Completed. F5dh `Completed` is an operation-less terminal and remains outside this per-operation presenter step. F5dk must not call F5dh `start`, `step`, or `resume_slice`, F5dg `start` or `step`, F5cy / F5cw action validation, F5da-F5de action drivers, F5cs / F5ct / F5cu, host imports, platform APIs, DOM, Canvas, minifb, video memory, raw storage, queues, timers, schedulers, fallback paths, or silent no-op behavior.

## Std layer row tile RLE present host span operation presenter loop boundary

F5dl introduces the std layer row tile RLE present host span operation presenter loop boundary. This is still not an actual Web, native, bare, or headless presenter implementation. It is the shared loop state contract that keeps platform code from calling F5dh and F5dk directly. `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterLoopState` is the LoopState value. It carries the target support set, the F5dh scheduling policy, and the current scheduled state together so the next request does not depend on separate side state.

`presenter_loop_start` calls F5dh `start` exactly once. It returns LoopState only after F5dh start succeeds. Start failure keeps support, policy, action, lower F5dh start error, and category from F5dh public accessors. It does not call F5dg start directly.

`presenter_loop_request` accepts LoopState and calls F5dh step exactly once. If F5dh returns `OperationReady`, F5dl packages support, policy, and ready into a presenter request. If F5dh returns `Completed`, F5dl returns loop `Completed`. This `Completed` is allowed because it is the operation-less F5dh terminal, not a per-operation F5dk / F5dj completion. Request failure keeps the original LoopState, lower F5dh step error, and category from F5dh public accessors.

`presenter_loop_complete` accepts a request and a presenter supplied attempt. It calls F5dk presenter step exactly once. If F5dk rejects the attempt or completion, F5dl keeps the request and lower F5dk error and does not publish next state. If F5dk succeeds, F5dl extracts the F5dj completion step, reads the completion enum, and rewraps Continue / Yield scheduled state into Continue / Yield LoopState with the same support and policy.

F5dl does not execute host imports, synthesize presenter success or failure, call F5dh `resume_slice`, call F5di or F5dj direct validation / completion functions, call F5dg start / step, call F5cs / F5ct / F5cu, call F5da-F5de drivers, use F5cy / F5cw action validation, access raw packet storage, touch platform APIs, DOM, Canvas, minifb, video memory, queues, timers, real schedulers, fallback paths, or silent no-op behavior.

## Std layer row tile RLE present host span operation presenter outcome boundary

F5dm introduces the std layer row tile RLE present host span operation presenter outcome boundary. This boundary still does not execute a Web, native, bare, or headless host operation. It only gives actual presenter glue a typed bridge between F5dl request and F5dl complete. `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterOutcomeRequest` stores the F5dl request and the expected operation read from F5dh ready. The type is intentionally not Clone or Copy. It represents one presenter-facing operation request and should not be replayed accidentally.

The flow is value-consuming. `presenter_outcome_request` reads F5dl request ready, then reads F5dh ready operation, and stores both the original request and operation. The actual presenter may borrow the OutcomeRequest to inspect the operation. After it obtains a caller supplied `Result unit GuiError`, `presenter_outcome_attempt` consumes the OutcomeRequest, calls the F5di attempt constructor exactly once, and stores the original F5dl request with the created attempt in `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterOutcomeAttempt`. This type is also not Clone or Copy because it binds one outcome to one original request.

`presenter_outcome_complete` consumes OutcomeAttempt and calls F5dl complete exactly once. On success it returns the F5dl loop completion unchanged. On failure it keeps the original request, the F5di attempt, the lower F5dl complete error, and the category obtained from the F5dl public category accessor. This keeps error reporting data available without letting the presenter parse F5dk / F5dj lower variants.

F5dm does not execute host imports, synthesize host success or failure, call F5di validation, call F5dk presenter step, call F5dj completion step, call F5dh start / step / resume, call F5dg / F5cs / F5ct / F5cu / F5da-F5de / F5cy / F5cw, access raw storage, touch platform APIs, DOM, Canvas, minifb, video memory, queues, timers, real schedulers, fallback paths, silent no-op behavior, or create loop `Completed`.

## Std layer row tile RLE present host span operation presenter driver boundary

F5dn introduces the std layer row tile RLE present host span operation presenter driver boundary. It is still not a host executor or scheduler. It gives actual Web, native, bare, and headless presenter loops one value-consuming driver API over F5dl and F5dm, so callers do not manually interleave F5dl start / request with F5dm outcome request / attempt / complete.

`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterDriverState` stores the F5dl loop state and is intentionally not Clone or Copy. A presenter request consumes this state by value, because two independent requests from the same state would replay the same scheduled operation. DriverRequestResult and DriverCompletion are also not Clone or Copy because they may carry an OutcomeRequest or a next DriverState.

`presenter_driver_start` delegates to F5dl start exactly once. On success it wraps the F5dl loop state in DriverState. On failure it keeps support, policy, action, lower F5dl start error, and the lower category obtained from the F5dl public accessor.

`presenter_driver_request` consumes DriverState, reads its F5dl loop state, and delegates to F5dl request exactly once. If F5dl returns a request error, F5dn keeps the original DriverState and lower request error; it does not call F5dm. If F5dl returns terminal `Completed`, F5dn returns driver `Completed` and still does not call F5dm. Only the F5dl `Request` branch is converted into `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterOutcomeRequest` by F5dm outcome request.

`presenter_driver_complete` consumes OutcomeRequest and a caller supplied host outcome. It calls F5dm outcome attempt and F5dm outcome complete exactly once. On success it rewraps F5dl Continue / Yield completion as DriverCompletion containing the next DriverState. On failure it keeps the F5dm lower complete error and category from the F5dm public accessor. F5dn does not call F5dl complete directly, does not call F5di constructor or validation directly, does not call F5dh start / step / resume directly, and does not synthesize `Result::Ok unit` or `GuiError`.

F5dn does not execute host imports, run real schedulers, call action drivers, access raw packet storage, touch platform APIs, DOM, Canvas, minifb, video memory, queues, timers, fallback paths, or silent no-op behavior.

F5do introduces the std layer row tile RLE present host span operation presenter executor boundary. It is the narrow bridge between F5dn OutcomeRequest and an actual presenter executor attempt, but it is still not a host backend, not a scheduler, and not a video memory presenter. Its request constructor consumes an OutcomeRequest, reads the F5dl request stored inside it through public accessors, derives support from that request, and stores the expected span operation with the OutcomeRequest in `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorRequest`.

Unsupported operation handling is value preserving. F5do does not call F5dn complete with a synthetic unsupported outcome. Instead it returns `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorRequestError::UnsupportedOperation`, whose payload keeps the executor request, support, expected operation, and `GuiError::Unsupported` category. The upper presenter loop or scheduler can then decide whether to recover, close, or report the request owner.

Executor completion is attempt based. The executor returns `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorAttempt`, which contains the span operation it actually executed and the caller supplied outcome. F5do compares the request operation and the attempt operation at span-operation payload level before calling F5dn complete. This comparison intentionally does not map back to F5cw run-record actions. It uses public span operation, descriptor, row span, window id, frame id, surface id, packet descriptor, and color accessors.

Only a matching request and attempt can reach F5dn complete. A mismatch returns `AttemptMismatch` and keeps the original request and attempt owners. A lower F5dn complete error is wrapped as `DriverCompleteRejected` with category derived from F5dn public accessors. F5do does not call F5dl complete, F5dm outcome attempt or complete, F5di attempt construction or validation, F5cw action mapping, host imports, platform APIs, DOM, Canvas, minifb, video memory, queues, timers, schedulers, fallback paths, silent no-op behavior, or synthetic `Result::Ok unit` / `GuiError` outcomes.

F5dp introduces the std layer row tile RLE present host span operation presenter executor loop boundary. It composes F5dn and F5do into one value-consuming loop contract, but it is not actual Web / native / bare / headless execution and not real scheduler policy. Its `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorLoopState` owns the F5dn driver state.

`presenter_executor_loop_start` delegates to F5dn start exactly once and wraps the resulting driver state as loop state. `presenter_executor_loop_request` consumes LoopState, extracts driver state, and delegates to F5dn request exactly once. A F5dn request error is wrapped with category while preserving the lower F5dn error, whose recovery path owns the original driver state. F5dn terminal `Completed` becomes loop `Completed` without calling F5do. Only F5dn `Operation` is converted through F5do executor request.

`presenter_executor_loop_complete` consumes the F5do executor request and executor attempt, calls F5do complete exactly once, and rewraps F5dn DriverCompletion into Continue / Yield loop completion. This keeps F5do unsupported and mismatch semantics intact: F5dp does not synthesize `Err Unsupported`, does not synthesize `Ok unit`, and does not complete owner-bearing requests outside F5do.

F5dp does not call F5dn complete directly, F5dm outcome request / attempt / complete, F5dl loop functions, F5di attempt construction / validation, F5dh scheduling, F5dk presenter step, F5dj completion, F5cw action mapping, action drivers, host imports, platform APIs, DOM, Canvas, minifb, video memory, queues, timers, schedulers, fallback paths, or silent no-op behavior.

F5dq introduces the std layer row tile RLE present host span operation presenter executor attempt driver boundary. It is a completion wrapper over F5dp for actual Web, native, bare, and headless presenter executors that already produced an executor supplied attempt. It does not run the executor, does not build an attempt, and does not create a success or failure outcome.

`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorAttemptDriverStep` is completion-only. F5dp complete consumes `ExecutorRequest` and `ExecutorAttempt` by value, so F5dq must not store those consumed values in the success step. This keeps owner movement explicit and prevents replay of the same attempt.

F5dq failure is also lower-authoritative. `CompleteRejected` stores only the category and the lower F5dp error. If request or attempt recovery is needed, the recovery authority is the lower F5dp error chain. F5dq does not reach into lower private fields, does not duplicate request or attempt, and does not reconstruct attempts.

The F5dq step function calls F5dp `presenter_executor_loop_complete request attempt` exactly once. On `Ok`, it returns the completion-only step. On `Err`, it wraps the lower F5dp error and category. It does not call F5do request or complete directly, F5dn/F5dm/F5dl/F5di/F5dh/F5dk/F5dj directly, old action paths, virtual executor, virtual drain, host imports, platform APIs, DOM, Canvas, minifb, video memory, raw storage, queues, timers, schedulers, fallback paths, silent no-op behavior, or synthetic `Result::Ok unit` / `Result::Err GuiError` outcomes.

F5dr introduces the std layer row tile RLE present host span operation presenter executor session boundary. It sits above F5dp and F5dq, but still below any actual Web, native, bare, or headless presenter loop. Its purpose is to give platform loops an explicit session state shape: ready state, pending executor request, completion result, and terminal completed state. No sentinel / null or fallback state is needed.

`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionState` is either `Ready` with an F5dp loop state or `Completed`. Requesting a `Completed` state returns the terminal `Completed` request result without calling F5dp again. This is an explicit terminal behavior, not silent no-op. Only `Ready` calls F5dp request exactly once.

`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionPending` owns the executor request while the actual backend performs the operation. `session_complete` consumes the pending request and executor attempt, calls the F5dq attempt driver step exactly once, then maps F5dp `Continue` / `Yield` loop completion into Ready session states. The lower F5dq error remains the recovery authority on completion failure.

F5dr does not execute host imports, does not schedule timers, does not run a real scheduler, does not touch DOM, Canvas, minifb, video memory, raw storage, platform APIs, queues, timers, fallback paths, or silent no-op behavior. It also does not call F5dp complete directly, F5do/F5dn/F5dm/F5dl/F5di/F5dh/F5dk/F5dj directly, old action paths, virtual executor, or virtual drain.

## Std layer row tile RLE present host execution report boundary

F5cx introduces the std layer row tile RLE present host execution report boundary. It sits above F5cw and below the actual Web, native, bare, or offscreen executor implementation. The report preserves action context and executor outcome in one value, so diagnostics and logging can identify which `GuiRgba8888RowTileRlePresentHostExecutionAction` succeeded or failed without reinterpreting the request.

`GuiRgba8888RowTileRlePresentHostExecutionReport` contains the action and `GuiRgba8888RowTileRlePresentHostExecutionReportKind`. The kind is either `Succeeded` or `Failed GuiError`; failure never becomes a string code, bool, fallback, or silent no-op. Report construction wraps an executor-supplied `Result unit GuiError` and has no new failure mode, so it returns the report directly.

F5cx is not actual execution and not pending completion. It does not call F5cv, F5cu, F5ct, F5cs, F5cp, or F5co, does not construct F5cr requests, and does not touch raw packet storage, host APIs, platform APIs, DOM, Canvas, minifb, video memory, queues, timers, or schedulers. The only bridge back to the dispatch loop is `report_outcome`, which reconstitutes the original `Result unit GuiError` so a caller can pass it to F5cv `complete_request`.

## Std layer row tile RLE present host executor boundary

F5cy introduces the std layer row tile RLE present host executor boundary. It sits between F5cw/F5cx and the actual Web, native, bare, or offscreen executor implementation. This boundary still does not execute host imports; it verifies that an executor supports the target it is about to handle and that a returned report belongs to the full action identity that was sent to that executor.

`GuiRgba8888RowTileRlePresentHostExecutorSupport` is a non-empty target support enum. It can express Window, Offscreen, Device, WindowOffscreen, WindowDevice, OffscreenDevice, and All, but it cannot represent a supports-nothing executor. Unsupported actions return `GuiRgba8888RowTileRlePresentHostExecutorError` with `UnsupportedAction`, category `GuiError::Unsupported`, the expected action, and `reported = None`.

Report validation uses `validate_report_for_action`. The function first requires the expected action to be supported. It then reads the action stored in `GuiRgba8888RowTileRlePresentHostExecutionReport` and compares full action identity, not only the enum variant. Full action identity includes target variant, window id where present, surface id, frame id, all packet metadata used by the present descriptor, run pixel offset, run pixel count, and RGBA channel values. A same-variant but different-payload report is `ReportActionMismatch` with category `GuiError::InvalidCommand` and `reported = Some reported_action`.

F5cy deliberately accepts a matching failed report. Association validation and executor outcome interpretation are separate contracts: failed reports are valid if they refer to the same action, and callers continue to use F5cx `report_outcome` before F5cv completion. F5cy does not call F5cv, F5cu, F5ct, F5cs, F5cp, F5co, does not construct F5cr requests, and does not touch raw packet storage, host APIs, platform APIs, DOM, Canvas, minifb, video memory, queues, timers, schedulers, fallback paths, or silent no-op behavior.

## Std layer row tile RLE present host report loop bridge boundary

F5cz introduces the std layer row tile RLE present host report loop bridge boundary. It sits above F5cv/F5cw/F5cx/F5cy and still below the actual Web, native, bare, or offscreen executor implementation. Its job is not to execute a host import; its job is to make the return path from an executor report to the dispatch loop explicit and one-shot.

The bridge owns validation before completion. It reads the request from `GuiRgba8888RowTileRlePresentDispatchLoopPendingRequest`, derives the expected `GuiRgba8888RowTileRlePresentHostExecutionAction` with F5cw, validates the returned `GuiRgba8888RowTileRlePresentHostExecutionReport` with F5cy, and only after successful validation extracts F5cx `report_outcome` and consumes the pending value with F5cv `complete_request`.

`GuiRgba8888RowTileRlePresentHostReportLoopBridgeError` carries `GuiRgba8888RowTileRlePresentHostReportLoopBridgeErrorKind`, category, and loop state. The error kind preserves the lower value: `ExecutorValidationFailed` holds the F5cy executor error, and `LoopCompletionFailed` holds the F5cv dispatch loop error. This keeps expected/reported action context available for validation failures while also preserving F5cv rollback state for executor failures.

F5cz deliberately distinguishes two failure families. Unsupported target support and wrong action reports stop before completion, returning previous loop state from the pending value. A matching failed report is valid executor output; the bridge passes `report_outcome` into `complete_request`, and the resulting `HostImportExecutionFailed` carries rollback state from F5cv. F5cz must not call F5cu, F5ct, F5cs, F5cp, or F5co, must not construct F5cr requests, and must not touch raw packet storage, host APIs, platform APIs, DOM, Canvas, minifb, video memory, queues, timers, schedulers, fallback paths, or silent no-op behavior.

## Std layer row tile RLE present host span operation presenter executor session turn boundary

F5ds introduces the std layer row tile RLE present host span operation presenter executor session turn boundary. It sits directly above F5dr and directly below any real scheduler, queue, timer, or platform executor. Its purpose is narrow: represent one scheduler turn as either an F5dr session state or an already-issued pending executor request.

`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnState` has only `Session` and `Pending`. There is no separate `Completed` turn state, so the contract can be summarized as no separate Completed turn state. This is intentional: F5dr already owns terminal completion through `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionState` and F5dr session request. Adding a second terminal variant at F5ds would create two possible representations, so the boundary keeps completion inside F5dr and lets `turn_poll` return a transient `Completed` poll result instead.

`turn_poll` consumes the turn state by value. When it receives `Pending`, it returns `Execute pending` and does not call F5dr session request. This is an owner transfer to the executor, not a silent no-op. When it receives `Session`, it calls F5dr session request exactly once and maps `Operation pending` to `Execute pending` and `Completed` to the transient poll result. F5ds does not inspect F5dr `SessionState::Ready` or `SessionState::Completed` directly.

`turn_complete` consumes the pending request and executor attempt by calling F5dr session complete exactly once. F5dr remains the authority for request completion and Continue / Yield classification. F5ds only wraps the returned F5dr session state back into `TurnState::Session`. F5ds must not call F5dp or F5dq directly, must not create synthetic executor outcomes, and must not touch platform APIs, raw packet storage, video memory, DOM, Canvas, minifb, queues, timers, schedulers, fallback paths, or silent no-op behavior.

## Std layer row tile RLE present host span operation presenter executor session turn step boundary

F5dt introduces the std layer row tile RLE present host span operation presenter executor session turn step boundary. It sits directly above F5ds and still below any real scheduler, queue, timer, or platform executor. Its purpose is not to run a scheduler; its purpose is to normalize F5ds poll and complete outcomes into one transient result enum that a future Web, native, bare, or headless driver can consume without inspecting lower session variants.

`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnStepResult` has `Execute`, `Continue`, `Yield`, and `Completed`. `Execute` owns the F5dr session pending request, while `Continue` and `Yield` own the F5ds turn state. `Completed` is a transient Completed result, not a persistent state. This keeps terminal ownership in F5dr / F5ds and prevents a second completed state from appearing in the step layer.

The start function delegates to F5ds `turn_start` exactly once and returns the F5ds turn state directly. This follows the rule that start is setup authority, not a scheduler tick outcome. The poll function delegates to F5ds `turn_poll` exactly once and maps `Execute` and `Completed` into the unified step result. The complete function delegates to F5ds `turn_complete` exactly once and maps `Continue` and `Yield` into the same step result enum.

F5dt must not call F5dr, F5dp, F5dq, presenter loop, old action driver, virtual executor, dispatch loop, raw storage, host import, platform API, DOM, Canvas, minifb, video memory, queue, timer, real scheduler, fallback path, or silent no-op path directly. Error recovery is also delegated to F5ds wrapper errors, with category values exposed only through F5ds category accessors.

## Std layer row tile RLE present host span operation presenter executor session turn driver boundary

F5du introduces the std layer row tile RLE present host span operation presenter executor session turn driver boundary. It sits directly above F5dt and still below any real scheduler, queue, timer, platform executor, DOM, Canvas, minifb, video memory presenter, or fallback path. Its purpose is to make the `Execute` branch usable by an actual executor without letting that executor choose the operation identity.

`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnDriverPending` owns the F5dr session pending request. A driver poll maps F5dt `Execute pending` into `TurnDriverPending`; Continue, Yield, and Completed pass through as the same transient step result shape. The driver pending value is intentionally owner-bearing and must not implement Clone or Copy.

The key contract is caller supplied outcome only. `turn_driver_pending_operation` borrows through the new F5ds pending request reference and then uses F5do `executor_request_operation` to read the borrowed expected operation without consuming pending. `turn_driver_complete` first reads that borrowed expected operation, then constructs exactly one F5do executor attempt from the operation and the caller supplied outcome, then consumes the pending value and delegates to F5dt `turn_step_complete` exactly once. This prevents operation mismatch at this boundary because the executor cannot provide a different operation.

F5du may use F5do only for `executor_request_operation` and `executor_attempt`. It must not call F5do complete or request, F5dr / F5dp / F5dq direct completion paths, old action paths, raw storage, host import, platform API, DOM, Canvas, minifb, video memory, queue, timer, real scheduler, fallback path, silent no-op path, synthetic `Result::Ok unit`, or synthetic `Result::Err GuiError::` construction.

## Std layer row tile RLE present host span operation presenter executor session turn scheduler decision boundary

F5dv introduces the std layer row tile RLE present host span operation presenter executor session turn scheduler decision boundary. It sits above F5du and below any real scheduler backend, queue, timer, platform executor, DOM, Canvas, minifb, video memory presenter, or fallback path. Its purpose is to map the owner-bearing F5du driver step result into a target-neutral scheduler decision that later Web, native, bare, and headless runtimes can interpret.

`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnSchedulerDecision` has four outcomes. `Execute` carries the F5du driver pending value, `ContinueNow` carries a turn state that can be polled immediately, `ScheduleOneShot` carries a scheduled state plus a validated delay, and `Completed` is a terminal transient decision. `ScheduleOneShot` does not register a timer itself. It is only a typed request for the next layer.

The scheduler policy contains `yield_delay_ms`. Both the public policy constructor and `scheduler_decide` validate that the delay is non-negative. The second validation is part of the contract because public structs can be manually constructed. If the policy is invalid during decision, the result is an owner-bearing policy error. The error stores `PolicyInvalid`, a `GuiError::InvalidCommand` category, and the original F5du driver step value so the caller can recover the pending executor request or turn state.

F5dv must not call F5du start, poll, complete, or pending operation helpers. It only consumes a driver step value that was supplied by the caller. It also must not call timer APIs, one-shot timer registration, queue APIs, real scheduler backend APIs, platform APIs, DOM, Canvas, minifb, video memory, raw storage, DrawTarget, RenderTarget, fallback paths, silent no-op paths, synthetic `Result::Ok unit`, or synthetic `Result::Err GuiError::` construction.

## Std layer row tile RLE present host span operation presenter executor session turn timer request boundary

F5dw introduces the std layer row tile RLE present host span operation presenter executor session turn timer request boundary. It sits above the F5dv scheduler decision and below actual Web, native, bare, and headless timer backends. Its purpose is to turn `ScheduleOneShot` into a target-neutral timer request value without registering that timer or selecting a platform scheduler.

`GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnTimerReady` has four outcomes. `Execute` and `ContinueNow` pass through scheduler work that can proceed immediately, `ScheduleTimer` carries an owner-bearing timer pending value, and `Completed` is terminal. The timer pending value owns the F5dv scheduled state and the std `TimerRequest`; it is intentionally not Clone or Copy because it is the authority for resuming that delayed turn.

The timer policy contains a checked `WindowId` and a `TimerId`. `TimerId` is still an opaque raw-id wrapper, so F5dw validates `timer_id_raw > 0` in both the public policy constructor and the decision interpreter. This second validation is required because public structs can be manually constructed. `TimerRequest` is created only after policy validation and scheduled delay validation. The request uses the checked window, checked timer, validated delay, and `repeating false`, so the boundary expresses exactly one one-shot wakeup request.

Invalid policy and invalid scheduled delay are owner-bearing interpret errors. The error stores the original scheduler decision, a typed error kind, and a `GuiError::InvalidCommand` category. Timer completion is also owner-bearing: pending request timer id, incoming `TimerEvent` timer id, and `TimerEvent` tick are validated before the scheduled turn state is consumed. Success returns the F5dv `SchedulerDecision::ContinueNow state`; failure keeps both pending and event in the complete error so the caller can recover the scheduled owner.

F5dw must not call F5du driver start, poll, complete, or pending operation helpers. It must not call `schedule_timer`, queue APIs, real scheduler backend APIs, platform APIs, DOM, Canvas, minifb, video memory, raw storage, DrawTarget, RenderTarget, fallback paths, silent no-op paths, synthetic `Result::Ok unit`, or synthetic `Result::Err GuiError::` construction.

F5dz introduces the std layer row tile RLE present host span operation presenter executor session turn virtual timer bridge. It sits between the F5dw target-neutral timer request boundary and the F5dy deterministic virtual timer scheduler. Its purpose is to let headless and offscreen tests drive the same scheduled turn continuation through `GuiEvent::Timer` without adding a real scheduler loop, queue, platform timer backend, DOM, Canvas, minifb, video memory, DrawTarget, RenderTarget, fallback path, silent no-op, or loop drain.

The bridge state is `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualTimerPending`. It owns the F5dw timer pending value and a `GuiVirtualTimerState`. Scheduling reads the `TimerRequest` from the F5dw pending value by borrow and calls `gui_virtual_timer_schedule` exactly once. A schedule failure keeps the original F5dw pending, the original virtual timer state, and the lower `GuiError`.

Advancing calls `gui_virtual_timer_advance` exactly once. If no event fires, the bridge returns a pending value with the original F5dw pending and the next virtual timer state. If `GuiEvent::Timer` fires, the bridge calls F5dw `turn_timer_complete` exactly once. A complete success returns the scheduler decision. A complete failure keeps the lower F5dw complete error and the advance-after virtual timer state. If F5dy ever yields a non-timer `GuiEvent`, the bridge returns an unexpected-event error that keeps the F5dw pending, the advance-after virtual timer state, and the event. This is intentionally not a silent ignore path.

F5ea introduces the std layer row tile RLE present host span operation presenter executor session turn virtual scheduler state boundary. It is not the real scheduler loop. Its public state is `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerState`, whose phases are `Turn`, `WaitingTimer`, `Execute`, and `Completed`. `Turn`, `Execute`, and `Completed` carry their current `GuiVirtualTimerState` directly. `WaitingTimer` carries the F5dz pending value, which already owns the F5dw timer pending and current virtual timer state. This keeps `GuiVirtualTimerState` as dynamic scheduler state rather than static policy.

The F5ea decision boundary takes a borrowed F5dw timer policy, the current `GuiVirtualTimerState`, and a scheduler decision. It calls F5dw `turn_timer_interpret_decision` exactly once. `TimerReady::Execute` becomes `Execute timer_state pending`. `TimerReady::ContinueNow` becomes `Turn timer_state turn_state`, so the next driver poll is explicit and the decision is not reused as a no-progress state. `TimerReady::ScheduleTimer` is the only branch that calls F5dz `virtual_timer_schedule`, and success becomes `WaitingTimer`. `TimerReady::Completed` becomes `Completed timer_state`. Interpret failure keeps the dynamic timer state and lower F5dw error. Schedule failure keeps the lower F5dz schedule error, whose payload retains original F5dw pending, original virtual timer state, and lower `GuiError`.

The F5ea timer advance boundary consumes a F5dz waiting timer pending and delta. It calls F5dz `virtual_timer_advance` exactly once. Pending advance remains `WaitingTimer`. Ready advance calls the same F5ea decision boundary with `gui_virtual_timer_empty` because F5dw emits one-shot timer requests and F5dy/F5dz clear the one-shot virtual timer before returning a ready decision. This `gui_virtual_timer_empty` handoff is a deliberate contract, not an implicit state loss. F5ea still does not implement loop drain, time-slice budget, actual Web / native / bare timer backend, platform event queue, DOM, Canvas, minifb, video memory, fallback path, or silent no-op.

F5eb introduces the std layer row tile RLE present host span operation presenter executor session turn virtual scheduler single step boundary. It consumes one `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerState` and either advances it once or returns an explicit blocked result. Its public result is `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerStepResult`, with `Advanced`, `BlockedWaitingTimer`, `BlockedExecute`, and `Completed`.

The F5eb Turn path is intentionally narrow. It calls F5du driver poll exactly once, then F5dv scheduler decide exactly once, then the F5ea timer decision boundary exactly once. A poll failure keeps the current `GuiVirtualTimerState` and the lower poll error. A scheduler decision failure keeps the same current timer state and the lower scheduler decision error. A timer decision failure keeps the F5ea lower error instead of unpacking and reclassifying ownership-sensitive recovery payloads.

F5eb does not pretend that blocked phases made progress. `WaitingTimer` becomes `BlockedWaitingTimer`, `Execute` becomes `BlockedExecute`, and `Completed` becomes `Completed`. These results let the later real scheduler loop decide whether to wait for virtual timer advance, call an executor, or terminate. F5eb has no loop drain, timeslice budget, queue, platform timer backend, DOM, Canvas, minifb, video memory, fallback path, or silent no-op.

F5ec introduces the std layer row tile RLE present host span operation presenter executor session turn virtual scheduler bounded drain boundary. It is intentionally still a deterministic std-layer boundary, not a Web, native, bare, or headless backend. Its policy contains the F5eb step policy and `max_advance_count`. It does not contain dynamic `GuiVirtualTimerState`, backend timer handles, queue ownership, or platform state.

The public result is `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerDrainResult`. It has `BudgetExhausted`, `BlockedWaitingTimer`, `BlockedExecute`, and `Completed`. `BudgetExhausted` is an explicit terminal, not an implicit successful `Advanced` state. A zero `max_advance_count` returns `BudgetExhausted` with the original state and remaining count without calling F5eb step. Negative `max_advance_count` is rejected both when constructing the policy and when entering drain, so forged or stale policy values cannot silently pass.

F5ec consumes budget only for F5eb `Advanced`. When F5eb returns `Advanced next_state`, F5ec decrements `remaining_count` by one and recurses with `next_state`. When F5eb returns `BlockedWaitingTimer`, `BlockedExecute`, or `Completed`, F5ec returns the corresponding drain terminal with the same remaining count, because no scheduler advance was consumed. When F5eb fails, F5ec returns `StepFailed` containing only the lower F5eb error. It does not duplicate the original scheduler state in the error payload because F5eb lower errors already carry the recovery payloads required by their boundary.

F5ec does not advance timers, complete executor actions, run the real scheduler loop, implement a time-slice backend, touch DOM / Canvas / minifb, manage video memory, add fallback paths, or introduce silent no-op behavior. Later real scheduler work must consume `BudgetExhausted`, `BlockedWaitingTimer`, `BlockedExecute`, and `Completed` explicitly.

F5ed introduces the std layer row tile RLE present host span operation presenter executor session turn virtual scheduler transition boundary. Its public action type is `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerTransition`. It has `YieldSlice`, `AwaitTimer`, `ExecuteHostAction`, and `Done`. `YieldSlice` maps from F5ec `BudgetExhausted`, `AwaitTimer` maps from `BlockedWaitingTimer`, `ExecuteHostAction` maps from `BlockedExecute`, and `Done` maps from `Completed`.

F5ed is a rewrap boundary, not a scheduler executor. It does not expose F5ec drain payload structs in transition payloads. Each branch reads `remaining_count` through the F5ec accessor before consuming the owner-bearing terminal payload, then stores the authority value and `remaining_count` in a transition-owned payload. The value is not normalized, decremented, or recomputed. F5ed does not re-run F5ec drain, call F5eb step, advance timers, complete executor actions, run a real scheduler loop, drain queues, touch platform APIs, touch DOM / Canvas / minifb, manage video memory, add fallback paths, or introduce silent no-op behavior.

F5ee introduces the std layer row tile RLE present host span operation presenter executor session turn virtual scheduler slice boundary. It connects F5ec bounded drain and F5ed transition as one public work slice. Its public result type is `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerSliceResult`. It has `YieldSlice`, `AwaitTimer`, `ExecuteHostAction`, and `Done`. The policy contains only the F5ec drain policy and `yield_delay_ms`; it does not contain dynamic timer state, backend handles, queue ownership, or platform state.

F5ee validates `yield_delay_ms >= 0` both when constructing the policy and when entering the slice. The public entry calls F5ec drain exactly once and, on success, calls F5ed transition mapping exactly once. It does not expose F5ec or F5ed payload structs in slice payloads. `YieldSlice` stores state, `remaining_count`, and `yield_delay_ms`; `AwaitTimer`, `ExecuteHostAction`, and `Done` store the corresponding pending / execute / completed authority with `remaining_count`. Drain failure stores only the lower F5ec error. F5ee does not call F5eb step directly, advance timers, complete executor actions, run a real scheduler loop, drain queues, touch platform APIs, touch DOM / Canvas / minifb, manage video memory, add fallback paths, or introduce silent no-op behavior.

F5ef introduces the std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop boundary. It does not implement an actual while loop. Instead, it calls F5ee `virtual_scheduler_slice` exactly once and maps the returned slice result into the loop-owned result that real scheduler loop / headless app-loop code will match. Its public result type is `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopResult`. It has `Yield`, `AwaitTimer`, `ExecuteHostAction`, and `Done`. The policy contains only the F5ee slice policy.

F5ef does not expose F5ee payload structs in loop payloads. `Yield` stores state, `remaining_count`, and `yield_delay_ms`; `AwaitTimer`, `ExecuteHostAction`, and `Done` store pending / execute / completed authority with `remaining_count`. Failure stores lower-only slice error, which is the lower F5ee slice error. F5ef does not directly call F5ec drain, F5ed transition, F5eb step, or F5ea helpers. It does not advance timers, complete executor actions, drain queues, touch platform APIs, touch DOM / Canvas / minifb, manage video memory, add fallback paths, or introduce silent no-op behavior.

F5eg introduces the std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop action boundary. It consumes only the F5ef loop result and maps it to `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopAction` by total mapping. The public `loop_action_from_result` entry maps `Yield` to `YieldToClock`, `AwaitTimer` to `AwaitTimerAdvance`, `ExecuteHostAction` to `ExecuteHostAction`, and `Done` to `Complete` with explicit match. It does not call F5ef `loop_step`; the caller supplies the loop result. F5eg payloads do not hold F5ef payload structs. They store state / pending / execute / completed authority plus `remaining_count` and `yield_delay_ms` as action-owned payload. Because the mapping is total, F5eg does not create a new error `Result`; later timer advance and executor completion authorities must return typed `Result` when they perform effects. F5eg does not advance timers, complete executor actions, implement a real scheduler loop, drain queues, touch native / bare / headless real backends, touch platform APIs, touch DOM / Canvas / minifb, manage video memory, add fallback paths, or introduce silent no-op behavior.

F5eh introduces the std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop timer advance boundary. It consumes only the F5eg `AwaitTimerAdvance` payload and calls F5ea `virtual_scheduler_advance_timer` exactly once. The public `loop_timer_advance` entry takes `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionAwaitTimerAdvance`, `TurnTimerPolicy`, and `delta_ms`; it does not take the general action enum and does not call `loop_action_from_result`. It reads `remaining_count` before consuming the pending owner, then passes the pending value to F5ea. On success it returns `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopTimerAdvanceCompleted` with the next scheduler state and original `remaining_count`. On failure it returns `AdvanceFailed` containing lower F5ea `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerAdvanceError` and original `remaining_count`. F5eh does not complete executor actions, handle yield-to-clock sleeping, run a real scheduler loop, drain queues, touch native / bare / headless real backends, touch platform APIs, touch DOM / Canvas / minifb, manage video memory, add fallback paths, or introduce silent no-op behavior.

F5ei introduces the std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop executor complete boundary. It consumes only the F5eg `ExecuteHostAction` payload and accepts a caller supplied `Result unit GuiError` outcome. The public `loop_executor_complete` entry takes `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopExecutorCompletePolicy`, `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionExecuteHostAction`, and the caller supplied outcome. The policy stores scheduler policy and timer policy only. The function reads `remaining_count`, consumes the execute payload, reads `timer_state`, consumes the driver pending owner, then calls F5du `turn_driver_complete`, F5dv `scheduler_decide`, and F5ea `virtual_scheduler_decide` exactly once each. On success it returns `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopExecutorCompleteCompleted` with the next scheduler state and original `remaining_count`. On F5du / F5dv failure it preserves `category`, `timer_state`, lower error, and original `remaining_count`; on F5ea decision failure it preserves lower error and original `remaining_count`. F5ei does not synthesize executor outcomes, does not call `loop_action_from_result`, does not handle YieldToClock or Complete actions, does not run a real scheduler loop, does not drain queues, does not touch native / bare / headless real backends, does not touch platform APIs, does not touch DOM / Canvas / minifb, does not manage video memory, does not add fallback paths, and does not introduce silent no-op behavior.

F5ej introduces the std layer row tile RLE present host span operation presenter executor session turn virtual scheduler loop yield complete boundary. It consumes only typed F5eg `YieldToClock` and `Complete` payloads. `loop_yield_complete_yield_advance` is the deterministic clock-delta authority used by a later actual real scheduler loop: it receives `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionYieldToClock` plus caller supplied `delta_ms`, reads `remaining_count` and `yield_delay_ms` before consuming the state owner, validates `delta_ms >= 0`, validates `yield_delay_ms >= 0`, and only then performs `sub yield_delay_ms delta_ms` under the `0 <= delta_ms < yield_delay_ms` pending branch. Invalid `delta_ms` returns `DeltaInvalid`; invalid payload delay returns `YieldDelayInvalid`; both map to `Option::Some GuiError::InvalidCommand` and keep the original action owner. Pending advancement returns `GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopYieldCompleteYieldAdvanceResult::YieldPending` with a reduced `YieldToClock` payload. Ready advancement returns `YieldReady` with the state owner and original `remaining_count`. `loop_yield_complete_complete_ack` reads `remaining_count` before consuming the completed owner and returns the terminal completed payload. F5ej does not run the actual real scheduler loop, does not handle timer advance, does not handle executor completion, does not call scheduler decision boundaries, does not drain queues, does not touch native / bare / headless backends, does not touch platform APIs, does not touch DOM / Canvas / minifb, does not manage video memory, does not add fallback paths, and does not introduce silent no-op behavior.

F5dx introduces the Web formal one-shot timer request backend boundary. It is the first actual Web platform consumer of the F5dw target-neutral `TimerRequest`, but it remains under `platforms/gui/web` and does not change the std/core/alloc contracts. `platforms/gui/web/timer` validates the request shape before crossing the scalar host import boundary: window id and timer id must be positive, interval must be non-negative, and `interval_ms == 0` is a clear request for the same window / timer pair.

The Web Shell timer registry stores the browser timer handle together with window id, timer id, interval, repeating mode, and tick. Existing timer reuse is allowed only when both interval and repeating mode are equal; changing either mode clears the old handle first. `repeating true` maps to `setInterval`; `repeating false` maps to `setTimeout`. Clearing must dispatch to `clearInterval` or `clearTimeout` according to the stored mode instead of treating every handle as an interval.

One-shot dispatch is clear-before-enqueue. The Shell reads the active timer state, validates that GUI input is still active and the window remains presented, computes the next tick, builds the `GuiEvent::Timer` payload, clears the active one-shot timer entry, and only then passes the event to the shared input queue. This ordering prevents a one-shot timer from firing twice if event handling synchronously causes another timer request or process shutdown. Repeating timers keep their state and update their tick before enqueue. Stale timers are cleared and do not enqueue events.

F5dx does not implement the general scheduler loop, time-slice policy, virtual scheduler / real scheduler unification, native backend, bare backend, or headless backend. It also does not introduce stdout fallback, stdout protocol fallback, polling fallback, DOM / Canvas handles in NEPL stdlib, or silent no-op semantics. Unsupported host runtime remains a typed status translated to `GuiError::Unsupported`, and invalid values remain `GuiError::InvalidCommand`.

## Std layer row tile RLE present host execution driver boundary

F5da introduces the std layer row tile RLE present host execution driver boundary. It still does not execute a host import. Its job is to hold the F5cv `GuiRgba8888RowTileRlePresentDispatchLoopPendingRequest` together with the F5cw `GuiRgba8888RowTileRlePresentHostExecutionAction` that an actual Web, native, bare, or headless executor must perform.

`GuiRgba8888RowTileRlePresentHostExecutionDriverPending` is an owner-bearing one-shot pending value. It stores the original pending request and the decoded action, but it does not implement Clone or Copy. `prepare` reads the request through the F5cv pending request accessor, derives the F5cw action exactly once, and then moves the original pending value into the driver pending record.

Executors read only `pending_action`. They return only `Result unit GuiError` to `complete_outcome`; they do not construct reports themselves. `complete_outcome` reads the stored action first, then moves the pending request out of the driver record, constructs an F5cx report, and delegates validation before completion to the F5cz bridge. The driver error wraps the F5cz bridge error as `BridgeFailed` while preserving category and dispatch loop state.

F5da must not call F5cv `complete_request` directly, must not reimplement F5cy support or full action identity validation, and must not construct F5cr requests. It also must not touch F5cu, F5ct, F5cs, F5cp, F5co, raw packet storage, host APIs, platform APIs, DOM, Canvas, minifb, video memory, queues, timers, schedulers, fallback paths, or silent no-op behavior.

## Std layer row tile RLE present virtual host executor boundary

F5db introduces the std layer row tile RLE present virtual host executor boundary. It is a deterministic headless/test executor over the same F5cw action shape that actual Web, native, and bare executors will consume later. It is not a fallback path, and it is not the actual platform presenter.

`GuiRgba8888RowTileRlePresentVirtualExecutor` stores a F5cy support value and a F5cs virtual drain. Execution starts by reading the F5da pending action. The virtual executor then runs F5cy `require_supported` before any drain mutation. This ordering matters because an unsupported target should consume the F5da driver pending for one-shot cleanup while keeping the virtual executor recovery state unchanged.

After support preflight succeeds, F5db maps the F5cw action into a F5cq `GuiRgba8888RowTileRlePresentHostCommandRecord`. The mapping is total over the nine target/record variants: Window / Offscreen / Device crossed with Begin / Run / End. Begin variants become `BeginFrame`, Run variants become `RunRecord`, and End variants become `EndFrame`.

Drain success updates the virtual executor and then completes the driver through F5da `complete_outcome Ok`. Drain failure keeps the original executor as the recovery state, converts the drain category into a `GuiError`, and still completes the F5da driver pending with `Err`. Support rejection similarly calls F5da `complete_outcome Ok` intentionally so that F5cz records validation failure before completion. If either expected failure cleanup path returns `Ok`, F5db reports `InconsistentCompletion`.

## Std layer row tile RLE present host action sink boundary

F5dc introduces the std layer row tile RLE present host action sink boundary. It sits between F5cw/F5cy and actual Web, native, or bare presenter code. The boundary packages an executor-supplied outcome together with the action that was sent to the executor, but it does not perform platform execution itself.

`GuiRgba8888RowTileRlePresentHostActionSinkStep` stores the F5cw action and the `Result unit GuiError` returned by the executor. This is deliberately not an accept/reject helper. F5dc does not manufacture success, and it does not turn unsupported work into a silent no-op. The only validation it owns is F5cy `require_supported support action` before step construction.

The boundary does not own F5da driver pending and does not call F5da completion. It also does not build F5cx reports or call F5cz bridge. Those layers remain responsible for one-shot completion and report validation. F5dc therefore gives Web/native/bare wrappers a shared typed preflight and outcome packaging contract without duplicating the dispatch-loop completion path or depending on DOM, Canvas, minifb, video memory, queue, timer, scheduler, raw packet storage, or fallback behavior.

`GuiRgba8888RowTileRlePresentVirtualExecutorError` carries `SupportRejected`, `DrainFailed`, `DriverFailed`, or `InconsistentCompletion`, plus category, recovery executor, and optional driver error. F5db must not call F5cv `complete_request` directly, must not call F5cz bridge directly, must not construct F5cr requests, and must not touch F5cu, F5ct, F5cp, F5co, raw packet storage, host APIs, platform APIs, DOM, Canvas, minifb, video memory, queues, timers, schedulers, fallback paths, or silent no-op behavior.

## Std layer row tile RLE present host action sink driver boundary

F5dd introduces the std layer row tile RLE present host action sink driver boundary. It is the shared bridge for actual Web, native, and bare executors after they have produced an executor-supplied `Result unit GuiError`. The bridge does not execute platform work. It coordinates ownership between F5dc outcome packaging and F5da one-shot completion.

`GuiRgba8888RowTileRlePresentHostActionSinkDriverStep` stores the F5dc `GuiRgba8888RowTileRlePresentHostActionSinkStep` and the F5da `GuiRgba8888RowTileRlePresentDispatchLoopCompletion`. This keeps diagnostic visibility into the action/outcome pair while returning the dispatch-loop completion that the runtime must continue from.

The ordering is fixed. F5dd first reads the action from `GuiRgba8888RowTileRlePresentHostExecutionDriverPending` by shared borrow. It then calls F5dc `gui_rgba8888_row_tile_rle_present_host_action_sink_step support action outcome`. If F5dc rejects the action, F5dd does not call F5da completion. It returns `SinkRejected` as an owner-bearing error that contains the F5dc sink error and the original driver pending. This preserves recovery authority for the actual executor wrapper and avoids cleanup by fabricated success.

If F5dc accepts the action, F5dd calls F5da `complete_outcome support driver outcome` with the same caller-supplied outcome. If F5da succeeds, F5dd returns the sink step plus completion. If F5da fails, the driver pending has already been consumed, so `DriverCompletionFailed` stores the F5da driver error and the accepted sink step only. F5dd therefore does not manufacture executor outcome. It never builds `Result::Ok unit` or synthetic `Result::Err` on behalf of the executor, and it never calls F5cv direct completion, F5cz bridge directly, F5cx report construction, F5cr request construction, F5db virtual executor, lower dispatch cursors, platform APIs, raw packet storage, queues, timers, schedulers, fallback paths, or silent no-op behavior.

## Std layer row tile RLE present host action attempt driver boundary

F5de introduces the std layer row tile RLE present host action attempt driver boundary. It is the contract between an actual Web, native, bare, or headless executor and F5dd completion. The executor reports the action it attempted and the outcome it observed; F5de verifies that the reported attempt still matches the one-shot F5da driver pending before any completion authority is used.

`GuiRgba8888RowTileRlePresentHostActionAttempt` stores the attempted F5cw action and the executor-supplied `Result unit GuiError`. It is a Copy value because it does not own the F5da driver pending. `GuiRgba8888RowTileRlePresentHostActionAttemptMismatch` owns the original F5da driver pending and therefore must not implement Clone or Copy. The mismatch payload also carries expected action, attempted action, and `GuiError::InvalidCommand` category for deterministic diagnostics.

The ordering is fixed. F5de first reads expected action from `GuiRgba8888RowTileRlePresentHostExecutionDriverPending` by shared borrow. It reads attempted action from the attempt value. It then calls F5cy `gui_rgba8888_row_tile_rle_present_host_executor_action_same &expected &attempted`, not a variant-only comparison. If the comparison fails, F5de returns `AttemptActionMismatch` and does not call F5dd. This keeps the pending owner recoverable for the actual executor wrapper and prevents stale asynchronous outcomes from completing the wrong pending request.

If the actions match, F5de reads the attempt outcome and calls F5dd `gui_rgba8888_row_tile_rle_present_host_action_sink_driver_step support driver outcome`. F5de does not reimplement F5dc support preflight or F5da completion; lower `SinkRejected` and `DriverCompletionFailed` errors are wrapped as `SinkDriverFailed`. F5de therefore does not manufacture executor outcome, does not call F5dc directly, and never reaches F5cv direct completion, F5cz bridge directly, F5cx report construction, F5cr request construction, F5db virtual executor, lower dispatch cursors, platform APIs, raw packet storage, queues, timers, schedulers, fallback paths, or silent no-op behavior.

## Metrics fixed-point

初期 core contract は i32 fixed-point value を使う。scale 単位は renderer/layout contract で決める。`GuiFontSize` は numerator/denominator を持つ。

```text
GuiFontSize:
    px_num i32
    px_den i32
```

`px_den <= 0` は invalid command である。Validated constructor は `Result GuiFontSize GuiError` を返す。Unchecked raw struct construction を public helper として推奨しない。

## Glyph metrics

Glyph metrics は logical advance と rendered bounds を分ける。

```text
GuiGlyphMetrics:
    glyph GuiGlyphId
    advance_x i32
    advance_y i32
    ink_bounds GuiRect
    allocation_bounds GuiRect
```

`advance_x` / `advance_y` は writing mode に依存する。縦書きでは `advance_y` が主 advance になり得る。`ink_bounds` は実際に塗られる可能性のある領域で、`allocation_bounds` は layout が確保する領域である。

## Rendered text metrics

Rendered text metrics は layout と screenshot / dirty region の接点である。

```text
GuiRenderedTextMetrics:
    logical_bounds GuiRect
    ink_bounds GuiRect
    allocation_bounds GuiRect
    baseline i32
```

`logical_bounds` は inline layout 上の box、`ink_bounds` は actual glyph mask / stroke / shadow の bounds、`allocation_bounds` は repaint / hit test / caret に使う保守的 bounds である。Shadow がある場合は `ink_bounds` に含める。

## Glyph paint

Glyph paint は `render2d` と同じ style model を使う。

```text
GuiBlendMode:
    SourceOver
    Copy
    Multiply
    Screen

GuiShadow:
    offset GuiPoint
    blur_radius i32
    spread i32
    paint GuiPaint

GuiGlyphPaint:
    fill Option GuiPaint
    stroke Option GuiStroke
    shadows GuiShadowRef
    blend GuiBlendMode
```

```text
GuiShadowRef:
    NoShadow
    SingleShadow GuiShadow
    ShadowRun GuiShadowRunId

GuiShadowRunId:
    raw i32
```

初期 slice は `Solid` 相当の既存 `GuiPaint` と既存 `GuiStroke` を再利用する。Gradient / pattern / layer は後続で `GuiPaint` の拡張または `render_style` 側の上位型として追加する。

High-level style は `Vec GuiShadow` を持てるが、no_alloc core command は `GuiShadowRef` だけを持つ。`GuiShadowRef::ShadowRun` は alloc layer が owns する shadow run を参照する id であり、core は `Vec` を import しない。F1 の実装は `NoShadow` と `SingleShadow` を扱い、`ShadowRun` は id value として保持するだけで展開しない。

## Validation

`core/gui/font` は no_alloc の O(1) validation helper だけを持つ。

- font size denominator が 0 以下なら invalid。
- font weight が 1..1000 外なら invalid。
- variation axis tag は four-byte tag として別 type 化する。
- fill と stroke が両方 `None` の glyph paint は invalid。
- shadow blur / spread が negative の場合は invalid。

F1/F2 の戻り値は既存 `Result T GuiError` を使う。font-specific reason は `GuiFontErrorKind` data value として定義し、将来の detailed diagnostic payload へ接続できるようにする。`GuiFontErrorKind` は表示文字列ではなく enum であり、error display は std/platform 側に置く。

## Test strategy

最小 slice では次をテストする。

- `GuiWritingMode` を match できる。
- `GuiFontSize` validation が invalid denominator を拒否する。
- `GuiRenderedTextMetrics` が logical / ink / allocation を別々に保持する。
- `GuiGlyphPaint` が fill-only、stroke-only、fill+stroke を受け、none+none を拒否する。
- `GuiShadowRef` が no shadow、single shadow、shadow run id を sentinel なしで保持する。
- `GuiFontResourceRequest` が path、face index、expected hash、decode policy を保持する。
- `std/gui/font_resource` が DOM / Canvas / OS handle を公開しない。
- Formal font renderer の contract が `MockTextMeasurer` / `HostTextMeasurer` を fallback として参照しない。

TTF table parsing、glyph outline、rasterization、ruby/vertical/math layout は後続 phase で別テストを追加する。
## F5ek std layer row tile RLE present host span operation presenter executor session turn virtual scheduler real loop step boundary

F5ek は F5eg `LoopAction` 全体を扱う最初の real loop step 境界である。ただし、この段階では actual while loop、queue drain、host backend、platform API、DOM、Canvas、minifb、video memory は実装しない。入力 action と explicit input を照合し、F5ej / F5eh / F5ei の既存 typed authority へ委譲する。

policy は `scheduler_policy` と `timer_policy` だけを保持する。Execute branch では F5ei の borrowed policy entry `loop_executor_complete_with_policy_refs` を呼び、executor 用に別の timer policy を持たない。これにより `AwaitTimerAdvance` と `ExecuteHostAction` が同じ timer policy authority を共有する。

`RealLoopStepInput` は `ClockDelta`、`ExecutorOutcome`、`CompleteAck` の enum で表す。`YieldToClock` / `AwaitTimerAdvance` / `ExecuteHostAction` / `Complete` と input の対応が崩れた場合は、対応する `YieldInputMismatch`、`TimerInputMismatch`、`ExecuteInputMismatch`、`CompleteInputMismatch` を返し、action owner と input owner を回収可能にする。
