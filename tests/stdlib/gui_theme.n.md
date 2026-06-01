# alloc/gui theme

このファイルは GUI / TUI 共通 theme data が文字列状態や表示方式ごとの実体参照を持たず、`GuiColor`、`Option`、`Result` で扱えることを固定します。

## theme_palette_color_roles_are_typed

[目的/もくてき]:
- palette lookup が文字列 key ではなく `ThemeColorRole` で行われることを確認します。
- `GuiColor` から `Rgba8888` への[読/よ]み[出/だ]しが `Result` で明示されることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok]\n[0] ok\n[1] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/theme" as *
#import "core/cast" as *
#import "core/gui" as *
#import "core/result" as *
#import "std/test" as *

fn rgba_color %fn i32 fn i32 fn i32 GuiColor \r\g\b:
    let rr %u8 cast r
    let gg %u8 cast g
    let bb %u8 cast b
    let aa %u8 cast 255
    let rgba %Rgba8888 rgba8888_new rr gg bb aa
    gui_color_rgba8888 rgba

fn sample_palette %fn unit ThemePalette \unit:
    theme_palette:
        rgba_color 10 11 12
        rgba_color 20 21 22
        rgba_color 30 31 32
        rgba_color 40 41 42
        rgba_color 50 51 52
        rgba_color 60 61 62
        rgba_color 70 71 72
        rgba_color 80 81 82

fn main %impure fn unit i32 \unit:
    let palette %ThemePalette sample_palette
    let accent_color %GuiColor theme_palette_color &palette ThemeColorRole::Accent
    let accent_check match theme_color_as_rgba8888 accent_color:
        Result::Ok rgba:
            assert_eq_i32 50 cast rgba8888_r &rgba
        Result::Err _error:
            assert false
    let text_color %GuiColor theme_palette_color &palette ThemeColorRole::Text
    let text_check match theme_color_as_rgba8888 text_color:
        Result::Ok rgba:
            assert_eq_i32 31 cast rgba8888_g &rgba
        Result::Err _error:
            assert false
    let checks1 checks_push checks_new accent_check
    let checks checks_push checks1 text_check
    let shown checks_print_report checks
    checks_exit_code shown
```

## theme_metrics_validation_is_result

[目的/もくてき]:
- 負の metric が clamp されず、`ThemeError::InvalidMetric` として返ることを確認します。
- `Rgba8888` でない `GuiColor` を text cell style へ暗黙変換しないことを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok]\n[0] ok\n[1] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/theme" as *
#import "core/gui" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let invalid_metric %i32 sub 0 1
    let invalid_check match theme_metrics_checked 2 1 invalid_metric 1 1:
        Result::Ok _metrics:
            assert false
        Result::Err error:
            match error:
                ThemeError::InvalidMetric:
                    assert true
                _:
                    assert false
    let binary_color %GuiColor GuiColor::Binary BinaryColor::On
    let color_check match theme_color_as_rgba8888 binary_color:
        Result::Ok _rgba:
            assert false
        Result::Err error:
            match error:
                ThemeError::UnsupportedColorFormat:
                    assert true
                _:
                    assert false
    let checks1 checks_push checks_new invalid_check
    let checks checks_push checks1 color_check
    let shown checks_print_report checks
    checks_exit_code shown
```

## gui_theme_preserves_optional_font_and_text_cell_style

[目的/もくてき]:
- 既定 font が `Option FontId` として保持されることを確認します。
- palette role から `TextCellStyle` を作るとき、foreground と background が role 通りに取り出されることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/theme" as *
#import "core/cast" as *
#import "core/gui" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn rgba_color %fn i32 fn i32 fn i32 GuiColor \r\g\b:
    let rr %u8 cast r
    let gg %u8 cast g
    let bb %u8 cast b
    let aa %u8 cast 255
    let rgba %Rgba8888 rgba8888_new rr gg bb aa
    gui_color_rgba8888 rgba

fn sample_palette %fn unit ThemePalette \unit:
    theme_palette:
        rgba_color 10 11 12
        rgba_color 20 21 22
        rgba_color 30 31 32
        rgba_color 40 41 42
        rgba_color 50 51 52
        rgba_color 60 61 62
        rgba_color 70 71 72
        rgba_color 80 81 82

fn main %impure fn unit i32 \unit:
    let palette %ThemePalette sample_palette
    let metrics %ThemeMetrics unwrap_ok theme_metrics_checked 4 2 1 1 3
    let font_id %FontId font_id_new 7
    let default_font %Option FontId some font_id
    let theme %GuiTheme unwrap_ok gui_theme_checked ThemeScheme::Dark palette metrics default_font
    let font_check match gui_theme_default_font &theme:
        Option::Some font:
            assert_eq_i32 7 font_id_raw &font
        Option::None:
            assert false
    let metric_check assert_eq_i32 4 gui_theme_metric &theme ThemeMetricRole::PaddingInline
    let style_check match theme_text_cell_style &palette ThemeColorRole::AccentText ThemeColorRole::Background:
        Result::Ok style:
            let foreground %Rgba8888 text_cell_style_foreground &style
            let background %Rgba8888 text_cell_style_background &style
            let foreground_r %i32 cast rgba8888_r &foreground
            if eq foreground_r 60:
                then assert_eq_i32 10 cast rgba8888_r &background
                else assert false
        Result::Err _error:
            assert false
    let checks1 checks_push checks_new font_check
    let checks2 checks_push checks1 metric_check
    let checks checks_push checks2 style_check
    let shown checks_print_report checks
    checks_exit_code shown
```
