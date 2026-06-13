# NEPLg2 GUI font rendering specification

作成日: 2026-06-13

## 目的

この文書は GUI font rendering の public contract を定義する。詳細な table parsing、rasterization、layout algorithm は `gui_font_rendering_detailed_design.md` に置き、段階的な作業順序は `gui_font_rendering_implementation_plan.md` に置く。

NEPLg2 GUI は Web、native、bare、offscreen、headless で同じ application code を動かす。そのため、font は browser font object、DOM、Canvas2D text API、CoreText、DirectWrite、fontconfig を標準 API の authority にしない。Host は font bytes と resource access を提供し、NEPL 側の font/layout/rendering library が metrics、shaping、rasterization、描画後 bounds を計算する。

## Zenn 方針との対応

- platform 依存 resource access は `std/gui` と `platforms/gui/*` に閉じ込める。
- `core/gui` は no_alloc value、enum、struct、Result-friendly contract だけを持つ。
- 欠損や未対応は fallback せず、`Option` または typed `Result` として扱う。
- metrics、writing mode、font selection、glyph paint は raw number や string sentinel ではなく enum / struct で表す。
- error value と error display は分離する。
- 契約と現在の実装を文書で分ける。

## 必須 contract

### Font identity

Font identity は表示名や path suffix ではなく、登録済み resource と face index から作る。

```text
GuiFontFaceId:
    raw i32

GuiGlyphId:
    raw i32

GuiFontResourceId:
    raw i32

GuiResourceHash:
    raw i32

GuiFontResourcePath:
    path str

GuiFontResourceRequest:
    path GuiFontResourcePath
    face_index Option i32
    expected_hash Option GuiResourceHash
    decode_policy GuiFontDecodePolicy
```

`face_index` は collection font の face 選択を明示する。存在しない face index は先頭 face へ fallback せず error にする。

この文書で public name として使う font identity は `GuiFontFaceId`、`GuiGlyphId`、`GuiFontResourceId`、`GuiResourceHash`、`GuiFontResourcePath` に統一する。古い設計文書に残る `FontFaceId`、`GlyphId`、`ResourceHash` は同じ概念の旧表記であり、新規実装では使わない。

`GuiFontResourcePath` の `path str` は typed resource path string であり、display name、family name、path suffix、browser-provided font name の authority ではない。

`GuiFontResourcePath` の canonical 表記は leading slash を持たない resource path である。初期 bundled fixture は `fonts/HackGenConsoleNF-Regular.ttf` と表す。Web VFS の内部 file path はこの canonical path に `/` を 1 つだけ前置した `/fonts/HackGenConsoleNF-Regular.ttf` であり、public font identity ではない。Resource lookup は canonical path 全体の一致で行い、`HackGenConsoleNF-Regular.ttf` のような suffix match、font family name、browser-provided display name を authority として使ってはならない。

Resource path の normalization は platform boundary で一度だけ行う。空 path、absolute path、backslash、empty segment、`.`、`..` は typed error として拒否し、別 path へ推測変換しない。

### SFNT representative names

SFNT `name` table から得る display 用 metadata は、path suffix、browser-provided display name、OS font family lookup ではなく、font bytes 内の record だけを authority とする。

初期 slice が返す representative name は次に限定する。

```text
GuiSfntNames:
    family Option str      nameID 1
    subfamily Option str   nameID 2
    full_name Option str   nameID 4
```

選択順位は nameID ごとに次で固定する。

1. platformID 3, encodingID 1, languageID 0x0409: Windows Unicode BMP, UTF-16BE ASCII subset
2. platformID 3, any other encoding/language: selected if no rank 1 exists, then `UnsupportedNameEncoding`
3. platformID 1, encodingID 0, languageID 0: Macintosh Roman ASCII subset
4. platformID 1, any other encoding/language: selected if no higher rank exists, then `UnsupportedNameEncoding`

その他 platform の record は representative candidate ではない。nameID 1 / 2 / 4 の candidate が存在しない場合、その field は `Option::None` である。

`name` table が存在しない場合は `MissingTable`、format 0 以外は `UnsupportedNameTableFormat`、record range や UTF-16BE 奇数 length は `MalformedNameRecord`、ASCII subset 外の文字は `UnsupportedNameCharacter` で返す。空文字の selected representative は layout / UI display の metadata として不正なので `MalformedNameRecord` とする。

### SFNT cmap glyph mapping

SFNT `cmap` table から得る character-to-glyph mapping は、font bytes 内の mapping table だけを authority とする。Host font substitution、OS character map、browser text API、resource path suffix は使わない。

初期 slice は Unicode BMP の最小実用経路だけを扱う。

```text
gui_sfnt_lookup_glyph_id:
    bytes &ByteBuf
    face_index Option i32
    code_point i32
    -> Result GuiGlyphId GuiSfntParseError
```

Subtable selection は deterministic である。

1. `cmap` encoding record のうち platformID 3 / encodingID 1 を選択する。
2. 同じ platformID 3 / encodingID 1 が複数ある場合は最初に出現した record を使う。
3. それ以外の record は F4c の candidate ではない。対象 record がなければ `UnsupportedCmapEncoding` を返す。
4. 選択 record の subtable format が 4 でなければ `UnsupportedCmapTableFormat` を返す。別 record へ切り替えない。

`cmap` table 自体がない場合は `MissingTable` である。code point が BMP 外、つまり `0..65535` の範囲外の場合は、format 4 では表現できないので `UnsupportedCmapEncoding` で返す。BMP 内だが segment が存在しない場合、computed glyph id が 0 の場合、または glyphIdArray entry が 0 の場合は `MissingGlyphMapping` で返す。Glyph 0 を成功値として返してはならない。

Format 4 parser は `cmap` table header が declared table length 内に収まること、選択 subtable offset が encoding record array より後ろを指すこと、`length` が `cmap` table record 内に収まること、`segCountX2` が 0 でなく偶数であること、`reservedPad` が 0 であること、endCode / startCode / idDelta / idRangeOffset の各 array が subtable 内に収まること、idRangeOffset の指す glyphIdArray entry がその idRangeOffset word の位置から計算して subtable 内に収まることを検査する。不正な範囲は `MalformedCmapRecord` として返す。

### Supported font containers

標準設計は次を対象にする。

- TTF
- OTF
- TTC
- OTC
- variable font を含む OpenType variation tables
- WOFF / WOFF2 は後続 phase の decode container とする

初期実装で decode できない container は `Unsupported` として返す。別 container へ推測変換しない。

### Writing mode

横書き、右横書き、縦書きを同じ text layout model で扱う。

```text
GuiWritingMode:
    HorizontalLtr
    HorizontalRtl
    VerticalRl
    VerticalLr
```

`HorizontalRtl` は Arabic/Hebrew などの右横書きだけでなく、将来の bidi layout と同じ direction contract へ接続する。縦書きでは glyph orientation、advance、ruby placement、line progression が metrics に反映される。

### Metrics

Layout engine が使う metrics は rendering engine と同じ font face、size、variation、writing mode、feature set から生成する。

```text
GuiFontMetrics:
    ascent i32
    descent i32
    line_gap i32
    em_size i32

GuiGlyphMetrics:
    advance_x i32
    advance_y i32
    ink_bounds GuiRect
    allocation_bounds GuiRect

GuiRenderedTextMetrics:
    logical_bounds GuiRect
    ink_bounds GuiRect
    allocation_bounds GuiRect
    baseline i32
```

測定だけ host API、描画だけ別 engine という分離は禁止する。Host-provided mock measurer は explicit test utility であり、formal font renderer の代替ではない。

### Ruby, vertical text, math

Furigana / ruby は通常 text と同じ glyph pipeline を通る inline object とする。Ruby text だけを browser ruby layout や DOM に逃がさない。

数式描画は将来の math layout library が `GuiMathInlineBox` 相当の inline object を返し、font renderer と 2D renderer は glyph metrics、path、mask、baseline、ink bounds を共有する。

### Paint integration

Text は fill-only、stroke-only、fill+stroke、shadow をサポートする。Paint / stroke / shadow / blend mode は 2D renderer と共有し、text 専用 color model は作らない。

```text
GuiGlyphPaint:
    fill Option GuiPaint
    stroke Option GuiStroke
    shadows GuiShadowRef
    blend GuiBlendMode
```

fill と stroke が両方 `None` の描画 command は invalid である。透明 fallback や no-op にはしない。

`GuiShadowRef` は no_alloc core と alloc-backed multi-shadow の接点である。

```text
GuiShadowRef:
    NoShadow
    SingleShadow GuiShadow
    ShadowRun GuiShadowRunId

GuiShadowRunId:
    raw i32
```

`core/gui` の F1 実装は `NoShadow` と `SingleShadow` を O(1) value として扱う。複数 shadow は `alloc/gui/render2d` が owns する `Vec GuiShadow` を `GuiShadowRunId` で参照する。したがって high-level design の `shadows Vec Shadow` と no_alloc core の `GuiShadowRef` は矛盾しない。

## Error contract

F1/F2 は既存の `GuiError` を返すが、font-specific error category を public enum として同時に定義する。

```text
GuiFontErrorKind:
    InvalidFontSize
    InvalidFaceIndex
    FaceIndexRequired
    MissingFontResource
    UnsupportedFontContainer
    MissingGlyph
    InvalidGlyphPaint
```

最小 slice では `GuiFontErrorKind` を diagnostic/display に使うための data value とし、関数の戻り値は既存 `Result T GuiError` に揃える。詳細 error payload を `GuiError` へ接続する phase までは、対応は次で固定する。

- invalid request shape: `GuiError::InvalidCommand`
- invalid geometry or metrics bounds: `GuiError::InvalidGeometry`
- unsupported container / writing mode / feature: `GuiError::Unsupported`
- missing resource or missing glyph: `GuiError::Unsupported`
- capacity exhaustion: `GuiError::ResourceExhausted`

この対応は error display ではない。表示文言は `std/gui/error_display` または後続の font error display layer に置く。

## Current implementation

現在の stdlib には fixed-cell `MockTextMeasurer` と `HostTextMeasurer` wrapper がある。これは layout test、terminal text-grid、legacy smoke 用の deterministic utility であり、本格 font renderer ではない。

Formal font renderer は `core/gui/font` と `std/gui/font_resource` を通る。F1/F2 以降の font renderer 実装は `MockTextMeasurer`、`host_text_measurer`、`host_text_measurer_fixed` を fallback として import または呼び出してはならない。Fixed-cell 測定を使う場合は test utility または terminal backend として明示する。

次の実装 slice では、まず `core/gui/font` と `core/gui/render_style` に no_alloc contract を追加し、`std/gui/font_resource` に resource request boundary を追加する。TTF parsing や outline rasterization はこの slice では行わない。

## 非 goal

- Browser の `CanvasRenderingContext2D.fillText` を正式 renderer にすること。
- DOM / SVG text layout を標準 API に露出すること。
- OS font fallback を暗黙使用すること。
- Headless で fixed-cell fallback に暗黙切替すること。
- Missing glyph を別 glyph や tofu に暗黙置換すること。
