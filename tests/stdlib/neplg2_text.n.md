# NEPLg2 self-host source text

## builds_line_map_for_lf_and_eof

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/infra/text" as *
#import "std/test" as *
#import "core/field" as *
#import "core/math" as *

fn main <()*>i32> ():
    let checks0 checks_new
    match source_text_new 7 "sample.nepl" "alpha\nbeta\n":
        Result::Ok text:
            let text_len <i32> source_text_len &text
            let line_count <i32> source_text_line_count &text
            let checks1:
                checks0
                |> checks_push check_eq_i32 11 text_len
                |> checks_push check_eq_i32 3 line_count
            let checks2:
                match source_text_location_for_offset &text 6:
                    Option::Some loc:
                        let loc_line <i32> field::get loc "line"
                        let loc_column <i32> field::get loc "column"
                        checks1
                        |> checks_push check_eq_i32 1 loc_line
                        |> checks_push check_eq_i32 0 loc_column
                    Option::None:
                        checks_push checks1 Result<(),str>::Err "offset 6 did not resolve"
            let checks3:
                match source_text_location_for_offset &text 11:
                    Option::Some loc:
                        let loc_line <i32> field::get loc "line"
                        let loc_column <i32> field::get loc "column"
                        checks2
                        |> checks_push check_eq_i32 2 loc_line
                        |> checks_push check_eq_i32 0 loc_column
                    Option::None:
                        checks_push checks2 Result<(),str>::Err "EOF offset did not resolve"
            source_text_free text
            let shown checks_print_report checks3
            checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result<(),str>::Err "source_text_new failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## trims_crlf_line_spans

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/infra/text" as *
#import "std/test" as *
#import "core/field" as *

fn main <()*>i32> ():
    let checks0 checks_new
    match source_text_new 3 "crlf.nepl" "a\r\nbc\r\nd":
        Result::Ok text:
            let checks1:
                checks0
                |> checks_push check_eq_i32 3 source_text_line_count &text
            let checks2:
                match source_text_line_span &text 0:
                    Option::Some span0:
                        checks1
                        |> checks_push check_eq_i32 0 field::get span0 "start"
                        |> checks_push check_eq_i32 1 field::get span0 "end"
                    Option::None:
                        checks_push checks1 Result<(),str>::Err "line 0 span missing"
            let checks3:
                match source_text_line_span &text 1:
                    Option::Some span1:
                        checks2
                        |> checks_push check_eq_i32 3 field::get span1 "start"
                        |> checks_push check_eq_i32 5 field::get span1 "end"
                    Option::None:
                        checks_push checks2 Result<(),str>::Err "line 1 span missing"
            source_text_free text
            let shown checks_print_report checks3
            checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result<(),str>::Err "source_text_new failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## rejects_out_of_range_offsets_and_lines

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/infra/text" as *
#import "std/test" as *

fn main <()*>i32> ():
    let checks0 checks_new
    match source_text_new 2 "range.nepl" "abc":
        Result::Ok text:
            let checks1:
                checks0
                |> checks_push check is_none<SelfhostSourceLocation> source_text_location_for_offset &text -1
                |> checks_push check is_none<SelfhostSourceLocation> source_text_location_for_offset &text 4
                |> checks_push check is_none<SelfhostSourceSpan> source_text_line_span &text -1
                |> checks_push check is_none<SelfhostSourceSpan> source_text_line_span &text 1
            source_text_free text
            let shown checks_print_report checks1
            checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result<(),str>::Err "source_text_new failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## builds_large_line_map_without_stack_growth

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/collections/vec" as *
#import "alloc/string" as *
#import "core/field" as field
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/infra/text" as *
#import "std/test" as *
#import "core/field" as *

fn main <()*>i32> ():
    let mut sb <StringBuilder> unwrap_ok string_builder_new_result
    let mut i <i32> 0
    while lt i 4096:
        do:
            set sb unwrap_ok sb_append_result sb "x\n"
            set i add i 1
    let source <str> unwrap_ok sb_build_result sb
    let checks0 checks_new
    match source_text_new 11 "large.nepl" source:
        Result::Ok text:
            let text_len <i32> source_text_len &text
            let line_count <i32> source_text_line_count &text
            let checks1:
                checks0
                |> checks_push check_eq_i32 8192 text_len
                |> checks_push check_eq_i32 4097 line_count
            let checks2:
                match source_text_location_for_offset &text 8192:
                    Option::Some loc:
                        let loc_line <i32> field::get loc "line"
                        let loc_column <i32> field::get loc "column"
                        checks1
                        |> checks_push check_eq_i32 4096 loc_line
                        |> checks_push check_eq_i32 0 loc_column
                    Option::None:
                        checks_push checks1 Result<(),str>::Err "EOF offset did not resolve"
            let checks3:
                match source_text_line_span &text 4095:
                    Option::Some span:
                        let span_start <i32> field::get span "start"
                        let span_end <i32> field::get span "end"
                        checks2
                        |> checks_push check_eq_i32 8190 span_start
                        |> checks_push check_eq_i32 8191 span_end
                    Option::None:
                        checks_push checks2 Result<(),str>::Err "last content line span missing"
            source_text_free text
            let shown checks_print_report checks3
            checks_exit_code shown
        Result::Err _e:
            let checks1 checks_push checks0 Result<(),str>::Err "source_text_new failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```
