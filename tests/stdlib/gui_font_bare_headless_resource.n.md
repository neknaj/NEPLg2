# GUI font Bare / Headless resource providers

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_bare_headless_resource\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"headless explicit fixture\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"bare absent host unsupported\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/io" as *
#import "core/math" as math
#import "core/option" as *
#import "core/result" as *
#import "platforms/gui/bare" as *
#import "platforms/gui/headless" as *
#import "std/gui" as *
#import "std/test" as *

fn source_is_embedded %fn GuiFontResourceSource bool \source:
    match source:
        GuiFontResourceSource::EmbeddedBlob:
            true
        _:
            false

fn headless_fixture_ok %fn void bool \void:
    let path %GuiFontResourcePath unwrap_ok gui_font_resource_path_result "fonts/Test.ttf"
    let request %GuiFontResourceRequest unwrap_ok gui_font_resource_request path none none GuiFontDecodePolicy::SfntOnly
    let bytes %ByteBuf unwrap_ok io_bytebuf_from_str_result "AB"
    let resource %GuiFontResourceBytes unwrap_ok gui_headless_font_resource_request_bytes &request "fonts/Test.ttf" bytes
    let source %GuiFontResourceSource gui_font_resource_bytes_source &resource
    let source_ok %bool source_is_embedded source
    let len_ok %bool math::eq 2 gui_font_resource_bytes_len &resource
    gui_font_resource_bytes_free resource
    math::and source_ok len_ok

fn bare_absent_host_is_unsupported %impure fn void bool \void:
    let path %GuiFontResourcePath unwrap_ok gui_font_resource_path_result "fonts/Test.ttf"
    let request %GuiFontResourceRequest unwrap_ok gui_font_resource_request path none none GuiFontDecodePolicy::SfntOnly
    match gui_bare_font_resource_request_bytes &request:
        Result::Err kind:
            gui_font_resource_provider_error_kind_eq kind GuiFontResourceProviderErrorKind::UnsupportedProvider
        Result::Ok resource:
            gui_font_resource_bytes_free resource
            false

fn main %impure fn void i32 \void:
    let headless_ok %bool headless_fixture_ok
    let bare_ok %bool bare_absent_host_is_unsupported
    let report0 test_report_new "gui_font_bare_headless_resource"
    let report1 test_report_push report0 assert "headless explicit fixture" headless_ok
    let report2 test_report_push report1 assert "bare absent host unsupported" bare_ok
    test_report_exit_code test_report_print_stdout report2
```
