# selfhost CLI file_io

## selfhost_cli_file_io_reads_root_source_into_vfs

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "neplg2/cli/file_io" as *
#import "neplg2/core/module/loader" as *
#import "std/fs" as *
#import "std/test" as *
#import "core/math" as *

fn main %impure fn () i32 \():
    let path %str "tmp/selfhost_file_io_read_root.nepl"
    match fs_write_to_string path "fn main <()->i32> ():\n    0\n":
        Result::Err _e:
            1
        Result::Ok _:
            match selfhost_cli_file_io_read_root_vfs path:
                Result::Err _diag:
                    2
                Result::Ok vfs:
                    let file_count %i32 selfhost_vfs_len &vfs
                    selfhost_vfs_free vfs
                    let checks:
                        checks_new
                        |> checks_push check_eq_i32 1 file_count
                    let shown checks_print_report checks
                    checks_exit_code shown
```

## selfhost_cli_file_io_missing_source_returns_diagnostic

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok]
    ##: [0] ok
    ##: [1] ok
```neplg2
#entry main
#indent 4
#target std

#import "core/field" as field
#import "core/result" as *
#import "neplg2/cli/file_io" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/module/loader" as *
#import "std/test" as *
#import "core/field" as *

fn main %impure fn () i32 \():
    match selfhost_cli_file_io_read_root_vfs "__selfhost_file_io_missing_source__.nepl":
        Result::Ok vfs:
            selfhost_vfs_free vfs
            1
        Result::Err diag:
            let checks:
                checks_new
                |> checks_push assert_str_eq "loader.source.read_failed" selfhost_diag_code_name field::get diag "code"
                |> checks_push assert_str_eq "failed to read source file" field::get diag "message"
            let shown checks_print_report checks
            checks_exit_code shown
```

## selfhost_cli_file_io_writes_text_artifact

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "neplg2/cli/file_io" as *
#import "std/fs" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let path %str "tmp/selfhost_file_io_text_artifact.txt"
    match selfhost_cli_file_io_write_text_artifact path "artifact text\n":
        Result::Err _diag:
            1
        Result::Ok _:
            match fs_read_to_string_checked path:
                Result::Err _e:
                    2
                Result::Ok text:
                    let checks:
                        checks_new
                        |> checks_push assert_str_eq "artifact text\n" text
                    let shown checks_print_report checks
                    checks_exit_code shown
```

## selfhost_cli_file_io_writes_binary_artifact

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#indent 4
#target std

#import "alloc/io" as *
#import "core/result" as *
#import "neplg2/cli/file_io" as *
#import "std/fs" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let path %str "tmp/selfhost_file_io_binary_artifact.bin"
    match io_bytebuf_from_str_result "A\x00B":
        Result::Err _e:
            1
        Result::Ok bytes:
            match selfhost_cli_file_io_write_binary_artifact path bytes:
                Result::Err _diag:
                    2
                Result::Ok _:
                    match fs_read_to_bytes path:
                        Result::Err _e:
                            3
                        Result::Ok read_buf:
                            let text %str fs_bytes_to_string read_buf
                            let checks:
                                checks_new
                                |> checks_push assert_str_eq "A\x00B" text
                            let shown checks_print_report checks
                            checks_exit_code shown
```
