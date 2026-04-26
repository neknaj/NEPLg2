# NEPLg2 self-host source text

## builds_line_map_for_lf_and_eof

neplg2:test
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

fn main <()*>i32> ():
    let checks0 <Vec<Result<(),str>>> checks_new
    match source_text_new 7 "sample.nepl" "alpha\nbeta\n":
        Result::Ok text:
            let text_len <i32> source_text_len &text
            let line_count <i32> source_text_line_count &text
            let checks1 <Vec<Result<(),str>>>:
                checks0
                |> checks_push check_eq_i32 12 text_len
                |> checks_push check_eq_i32 3 line_count
            let checks2 <Vec<Result<(),str>>>:
                match source_text_location_for_offset &text 6:
                    Option::Some loc:
                        let loc_line <i32> field::get loc "line"
                        let loc_column <i32> field::get loc "column"
                        checks1
                        |> checks_push check_eq_i32 1 loc_line
                        |> checks_push check_eq_i32 0 loc_column
                    Option::None:
                        checks_push checks1 Result<(),str>::Err "offset 6 did not resolve"
            let checks3 <Vec<Result<(),str>>>:
                match source_text_location_for_offset &text 12:
                    Option::Some loc:
                        let loc_line <i32> field::get loc "line"
                        let loc_column <i32> field::get loc "column"
                        checks2
                        |> checks_push check_eq_i32 2 loc_line
                        |> checks_push check_eq_i32 0 loc_column
                    Option::None:
                        checks_push checks2 Result<(),str>::Err "EOF offset did not resolve"
            source_text_free text
            let shown <Vec<Result<(),str>>> checks_print_report checks3
            checks_exit_code shown
        Result::Err _e:
            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "source_text_new failed"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
```

## trims_crlf_line_spans

neplg2:test
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

fn main <()*>i32> ():
    let checks0 <Vec<Result<(),str>>> checks_new
    match source_text_new 3 "crlf.nepl" "a\r\nbc\r\nd":
        Result::Ok text:
            let checks1 <Vec<Result<(),str>>>:
                checks0
                |> checks_push check_eq_i32 3 source_text_line_count &text
            let checks2 <Vec<Result<(),str>>>:
                match source_text_line_span &text 0:
                    Option::Some span0:
                        checks1
                        |> checks_push check_eq_i32 0 field::get span0 "start"
                        |> checks_push check_eq_i32 1 field::get span0 "end"
                    Option::None:
                        checks_push checks1 Result<(),str>::Err "line 0 span missing"
            let checks3 <Vec<Result<(),str>>>:
                match source_text_line_span &text 1:
                    Option::Some span1:
                        checks2
                        |> checks_push check_eq_i32 3 field::get span1 "start"
                        |> checks_push check_eq_i32 5 field::get span1 "end"
                    Option::None:
                        checks_push checks2 Result<(),str>::Err "line 1 span missing"
            source_text_free text
            let shown <Vec<Result<(),str>>> checks_print_report checks3
            checks_exit_code shown
        Result::Err _e:
            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "source_text_new failed"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
```

## rejects_out_of_range_offsets_and_lines

neplg2:test
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
    let checks0 <Vec<Result<(),str>>> checks_new
    match source_text_new 2 "range.nepl" "abc":
        Result::Ok text:
            let checks1 <Vec<Result<(),str>>>:
                checks0
                |> checks_push check is_none<SelfhostSourceLocation> source_text_location_for_offset &text -1
                |> checks_push check is_none<SelfhostSourceLocation> source_text_location_for_offset &text 4
                |> checks_push check is_none<SelfhostSourceSpan> source_text_line_span &text -1
                |> checks_push check is_none<SelfhostSourceSpan> source_text_line_span &text 1
            source_text_free text
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
        Result::Err _e:
            let checks1 <Vec<Result<(),str>>> checks_push checks0 Result<(),str>::Err "source_text_new failed"
            let shown <Vec<Result<(),str>>> checks_print_report checks1
            checks_exit_code shown
```
