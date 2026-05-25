mod harness;

use harness::{
    run_main_capture_stdout_with_stdin, run_main_wasi_i32_with_stdin_raw_memory_boundary,
};

#[test]
fn kpread_to_stdio_stdout_i32() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "std/streamio" as *
#import "std/iotarget" as *
#import "std/stdio" as *
#import "core/result" as *

fn main <()*>()> ():
    let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;
    let a <i32> read &sc;
    let b <i32> read &sc;
    let c <i32> read &sc;
    close sc;
    println_i32 a;
    println_i32 b;
    println_i32 c;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"10 20 30\n");
    assert_eq!(out, "10\n20\n30\n");
}

#[test]
fn stdio_stdin_to_kpwrite_stdout() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "std/stdio" as *
#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main <()*>()> ():
    let line <str> read_line;
    let w0 <StreamWriter> unwrap_ok open WriteStream::Stdio;
    let w1 <StreamWriter> write w0 line;
    let w2 <StreamWriter> flush w1;
    close w2;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"hello world\n");
    assert_eq!(out, "hello world");
}

#[test]
fn kpread_to_kpwrite_prefixsum_i32() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/math" as *
#import "core/result" as *
#import "core/option" as *
#import "alloc/collections/vec" as *
#import "std/streamio" as *
#import "std/iotarget" as *

fn main <()*>()> ():
    let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;
    let n <i32> read &sc;
    let q <i32> read &sc;

    let pref_len <i32> add n 1;
    let pref <Vec<i32>> unwrap_ok filled<i32> pref_len 0;

    let mut i <i32> 1;
    while le i n:
        do:
            let a <i32> read &sc;
            let im1 <i32> sub i 1;
            let prev <i32> if and ge im1 0 lt im1 pref_len:
                then:
                    match get<i32> &pref im1:
                        Option::Some v:
                            v
                        Option::None:
                            #intrinsic "unreachable" <> ()
                else:
                    #intrinsic "unreachable" <> ()
            let cur <i32> add prev a;
            if and ge i 0 lt i pref_len:
                then:
                    replace<i32> &pref i cur
                else:
                    #intrinsic "unreachable" <> ()
            set i add i 1;

    let mut w <StreamWriter> unwrap_ok open WriteStream::Stdio;
    let mut k <i32> 0;
    while lt k q:
        do:
            let l1 <i32> read &sc;
            let r1 <i32> read &sc;
            let l <i32> sub l1 1;
            let diff <i32> if and and ge l 0 lt l pref_len and ge r1 0 lt r1 pref_len:
                then:
                    match get<i32> &pref l:
                        Option::Some left:
                            match get<i32> &pref r1:
                                Option::Some right:
                                    sub right left
                                Option::None:
                                    0
                        Option::None:
                            0
                else:
                    0
            set w writeln w diff;
            set k add k 1;

    set w flush w;
    close w;
    close sc;
    free<i32> pref;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"5 3\n1 2 3 4 5\n1 3\n2 5\n1 5\n");
    assert_eq!(out, "6\n14\n15\n");
}

#[test]
fn kpread_to_kpwrite_i64_dp() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/math" as *
#import "core/result" as *
#import "core/cast" as *
#import "std/streamio" as *
#import "std/iotarget" as *

fn ways <(i32)*>i64> (n):
    if le n 1:
        then <i64> cast 1
        else:
            let mut a <i64> cast 1;
            let mut b <i64> cast 1;
            let mut i <i32> 2;
            while le i n:
                do:
                    let c <i64> add a b;
                    set a b;
                    set b c;
                    set i add i 1;
            b

fn main <()*>()> ():
    let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;
    let n <i32> read &sc;
    let ans <i64> ways n;
    close sc;
    let w0 <StreamWriter> unwrap_ok open WriteStream::Stdio;
    let w1 <StreamWriter> writeln w0 ans;
    let w2 <StreamWriter> flush w1;
    close w2;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"6\n");
    assert_eq!(out, "13\n");
}

#[test]
fn kpwrite_i64_stdout_no_input() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/cast" as *
#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main <()*>()> ():
    let v <i64> cast 13;
    let w0 <StreamWriter> unwrap_ok open WriteStream::Stdio;
    let w1 <StreamWriter> writeln w0 v;
    let w2 <StreamWriter> flush w1;
    close w2;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"");
    assert_eq!(out, "13\n");
}

#[test]
fn kpwrite_i32_lines_no_input() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main <()*>()> ():
    let w0 <StreamWriter> unwrap_ok open WriteStream::Stdio;
    let w1 <StreamWriter> writeln w0 6;
    let w2 <StreamWriter> writeln w1 14;
    let w3 <StreamWriter> writeln w2 15;
    let w4 <StreamWriter> flush w3;
    close w4;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"");
    assert_eq!(out, "6\n14\n15\n");
}

#[test]
fn kpwrite_f64_stdout_no_input() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/cast" as *
#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main <()*>()> ():
    let v <f64> cast 1;
    let w0 <StreamWriter> unwrap_ok open WriteStream::Stdio;
    let w1 <StreamWriter> writeln w0 v;
    let w2 <StreamWriter> flush w1;
    close w2;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"");
    assert_eq!(out, "1.000000\n");
}

#[test]
fn kpwrite_f32_stdout_no_input() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/cast" as *
#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main <()*>()> ():
    let v <f32> cast 1;
    let w0 <StreamWriter> unwrap_ok open WriteStream::Stdio;
    let w1 <StreamWriter> writeln w0 v;
    let w2 <StreamWriter> flush w1;
    close w2;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"");
    assert_eq!(out, "1.000000\n");
}

#[test]
fn kpread_scanner_functional() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main <()*>()> ():
    let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;
    let a <i32> read &sc;
    let b <i32> read &sc;
    let c <i32> read &sc;
    close sc;
    let w0 <StreamWriter> unwrap_ok open WriteStream::Stdio;
    let w1 <StreamWriter> writeln w0 a;
    let w2 <StreamWriter> writeln w1 b;
    let w3 <StreamWriter> writeln w2 c;
    let w4 <StreamWriter> flush w3;
    close w4;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"10 20 30\n");
    assert_eq!(out, "10\n20\n30\n");
}

#[test]
fn kpread_scanner_read_bytes() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/math" as *
#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main <()*>()> ():
    let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;
    let a <i32> read &sc;
    let b <i32> read &sc;
    close sc;
    let sum <i32> add a b;
    let w0 <StreamWriter> unwrap_ok open WriteStream::Stdio;
    let w1 <StreamWriter> writeln w0 sum;
    let w2 <StreamWriter> flush w1;
    close w2;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"10 20 30\n");
    assert_eq!(out, "30\n");
}

#[test]
fn wasi_fd_read_raw_iovec_debug() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

#extern "wasi_snapshot_preview1" "fd_read" fn fd_read %impure fn i32 impure fn i32 impure fn i32 impure fn i32 i32

fn id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let cap <i32> 64;
    let buf <i32> alloc_raw cap;
    let iov <i32> alloc_raw 8;
    let nread <i32> alloc_raw 4;

    store_i32 iov add buf 0;
    store_i32 add iov 4 cap;
    store_i32 nread 0;

    let errno <i32> fd_read 0 iov 1 nread;
    let n <i32> load_i32 nread;
    let i0 <i32> id 0;
    let i1 <i32> id 1;
    let i2 <i32> id 2;
    let b0 <i32> if and ge i0 0 lt i0 n load_u8 add buf i0 -1;
    let b1 <i32> if and ge i1 0 lt i1 n load_u8 add buf i1 -1;
    let b2 <i32> if and ge i2 0 lt i2 n load_u8 add buf i2 -1;
    let ok_errno <bool> eq errno 0;
    let ok_n <bool> gt n 0;
    let ok_bytes <bool> and and eq b0 49 eq b1 48 eq b2 32;
    let raw_read_ok <bool> and and ok_errno ok_n ok_bytes;
    dealloc_raw buf cap;
    dealloc_raw iov 8;
    dealloc_raw nread 4;
    if raw_read_ok:
        then:
            1
        else:
            0
"#;
    let status = run_main_wasi_i32_with_stdin_raw_memory_boundary(src, b"10 20 30\n");
    assert_eq!(status, 1, "raw fd_read byte check failed");
}

#[test]
fn wasi_fd_read_raw_iovec_with_dealloc_debug() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

#extern "wasi_snapshot_preview1" "fd_read" fn fd_read %impure fn i32 impure fn i32 impure fn i32 impure fn i32 i32

fn id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let cap <i32> 64;
    let buf <i32> alloc_raw cap;
    let iov <i32> alloc_raw 8;
    let nread <i32> alloc_raw 4;

    store_i32 iov add buf 0;
    store_i32 add iov 4 cap;
    store_i32 nread 0;

    let errno <i32> fd_read 0 iov 1 nread;
    let n <i32> load_i32 nread;
    let i0 <i32> id 0;
    let i1 <i32> id 1;
    let i2 <i32> id 2;
    let b0 <i32> if and ge i0 0 lt i0 n load_u8 add buf i0 -1;
    let b1 <i32> if and ge i1 0 lt i1 n load_u8 add buf i1 -1;
    let b2 <i32> if and ge i2 0 lt i2 n load_u8 add buf i2 -1;
    let ok_errno <bool> eq errno 0;
    let ok_n <bool> gt n 0;
    let ok_bytes <bool> and and eq b0 49 eq b1 48 eq b2 32;
    let raw_read_ok <bool> and and ok_errno ok_n ok_bytes;
    dealloc_raw buf cap;
    dealloc_raw iov 8;
    dealloc_raw nread 4;
    if raw_read_ok:
        then:
            1
        else:
            0
"#;
    let status = run_main_wasi_i32_with_stdin_raw_memory_boundary(src, b"10 20 30\n");
    assert_eq!(status, 1, "raw fd_read byte check with dealloc failed");
}

#[test]
fn wasi_fd_read_then_alloc_header_debug() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

#extern "wasi_snapshot_preview1" "fd_read" fn fd_read %impure fn i32 impure fn i32 impure fn i32 impure fn i32 i32

fn id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let cap <i32> 64;
    let buf <i32> alloc_raw cap;
    let iov <i32> alloc_raw 8;
    let nread <i32> alloc_raw 4;

    store_i32 iov add buf 0;
    store_i32 add iov 4 cap;
    store_i32 nread 0;
    let errno <i32> fd_read 0 iov 1 nread;
    let n <i32> load_i32 nread;
    let zero <i32> id 0;

    let sc <i32> alloc_raw 12;
    store_i32 sc add buf 0;
    store_i32 add sc 4 n;
    store_i32 add sc 8 0;

    let header_buf <i32> load_i32 sc;
    let header_len <i32> load_i32 add sc 4;
    let header_pos <i32> load_i32 add sc 8;
    let ok_errno <bool> eq errno 0;
    let ok_n <bool> gt n zero;
    let ok_sc <bool> gt sc zero;
    let ok_buf <bool> eq header_buf buf;
    let ok_len <bool> eq header_len n;
    let ok_pos <bool> eq header_pos zero;
    let ok_left <bool> and and ok_errno ok_n ok_sc;
    let ok_right <bool> and and ok_buf ok_len ok_pos;
    dealloc_raw iov 8;
    dealloc_raw nread 4;
    dealloc_raw sc 12;
    dealloc_raw buf cap;
    if and ok_left ok_right:
        then:
            1
        else:
            0
"#;
    let status = run_main_wasi_i32_with_stdin_raw_memory_boundary(src, b"10 20 30\n");
    assert_eq!(status, 1, "fd_read raw header allocation check failed");
}

#[test]
/// Verifies that the raw-memory scanner header preserves the fd_read payload
/// owner through a stored raw pointer value without importing stdio helpers.
fn local_scanner_new_logic_debug() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

#extern "wasi_snapshot_preview1" "fd_read" fn fd_read %impure fn i32 impure fn i32 impure fn i32 impure fn i32 i32

fn id <(i32)->i32> (x):
    x

fn scanner_new_local <()*>i32> ():
    let mut cap <i32> 64;
    let mut buf <i32> alloc_raw cap;
    let iov <i32> alloc_raw 8;
    let nread_ptr <i32> alloc_raw 4;
    store_i32 iov add buf 0;
    store_i32 add iov 4 cap;
    store_i32 nread_ptr 0;
    let errno <i32> fd_read 0 iov 1 nread_ptr;
    let len <i32> load_i32 nread_ptr;

    dealloc_raw iov 8;
    dealloc_raw nread_ptr 4;

    let sc <i32> alloc_raw 16;
    store_i32 sc add buf 0;
    store_i32 add sc 4 len;
    store_i32 add sc 8 0;
    store_i32 add sc 12 cap;
    let header_buf <i32> load_i32 sc;
    let header_len <i32> load_i32 add sc 4;
    let header_pos <i32> load_i32 add sc 8;
    let header_cap <i32> load_i32 add sc 12;
    let ok_errno <bool> eq errno 0;
    let ok_len <bool> gt len 0;
    let ok_sc <bool> gt sc 0;
    let ok_header <bool> and and eq header_buf buf eq header_len len and eq header_pos 0 eq header_cap cap;
    let scanner_ok <bool> and and ok_errno ok_len and ok_sc ok_header;
    dealloc_raw buf cap;
    dealloc_raw sc 16;
    if scanner_ok:
        then:
            1
        else:
            0

fn main <()*>i32> ():
    scanner_new_local
"#;
    let status = run_main_wasi_i32_with_stdin_raw_memory_boundary(src, b"10 20 30\n");
    assert_eq!(status, 1, "local scanner construction check failed");
}

#[test]
/// Verifies that scanner growth preserves bytes that cross reallocation
/// boundaries while keeping the returned header range available for cleanup.
fn local_scanner_grow_loop_returns_header_range() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

#extern "wasi_snapshot_preview1" "fd_read" fn fd_read %impure fn i32 impure fn i32 impure fn i32 impure fn i32 i32

fn id <(i32)->i32> (x):
    x

fn scanner_read_all <()*>i32> ():
    let mut cap <i32> 4;
    let mut buf <i32> alloc_raw cap;
    memset_u8 buf cap 0;
    let iov <i32> alloc_raw 8;
    let nread_ptr <i32> alloc_raw 4;
    let mut len <i32> 0;
    let mut done <bool> false;
    while not done:
        do:
            if le cap len:
                then:
                    let next_cap <i32> mul cap 2;
                    let grown <i32> realloc_raw buf cap next_cap;
                    if eq grown 0:
                        then:
                            set done true
                        else:
                            set buf grown
                            memset_u8 add buf cap sub next_cap cap 0
                            set cap next_cap
                            store_i32 iov add buf len
                            store_i32 add iov 4 sub cap len
                            store_i32 nread_ptr 0
                            let errno <i32> fd_read 0 iov 1 nread_ptr;
                            let got <i32> load_i32 nread_ptr;
                            if or ne errno 0 eq got 0:
                                then:
                                    set done true
                                else:
                                    set len add len got
                else:
                    store_i32 iov add buf len
                    store_i32 add iov 4 sub cap len
                    store_i32 nread_ptr 0
                    let errno <i32> fd_read 0 iov 1 nread_ptr;
                    let got <i32> load_i32 nread_ptr;
                    if or ne errno 0 eq got 0:
                        then:
                            set done true
                        else:
                            set len add len got
    dealloc_raw iov 8;
    dealloc_raw nread_ptr 4;
    let sc <i32> alloc_raw 16;
    store_i32 sc buf;
    store_i32 add sc 4 len;
    store_i32 add sc 8 0;
    store_i32 add sc 12 cap;
    sc

fn main <()*>i32> ():
    let sc <i32> scanner_read_all;
    let data <i32> load_i32 sc;
    let len <i32> load_i32 add sc 4;
    let cap <i32> load_i32 add sc 12;
    let i0 <i32> id 0;
    let i4 <i32> id 4;
    let i8 <i32> id 8;
    let b0 <i32> if and and ge i0 0 lt i0 len lt i0 cap load_u8 add data i0 -1;
    let b4 <i32> if and and ge i4 0 lt i4 len lt i4 cap load_u8 add data i4 -1;
    let b8 <i32> if and and ge i8 0 lt i8 len lt i8 cap load_u8 add data i8 -1;
    let ok_shape <bool> and eq len 10 ge cap 16;
    let ok_bytes <bool> and and eq b0 97 eq b4 101 eq b8 105;
    let scanner_ok <bool> and ok_shape ok_bytes;
    dealloc_raw data cap;
    dealloc_raw sc 16;
    if scanner_ok:
        then:
            1
        else:
            0
"#;
    let status = run_main_wasi_i32_with_stdin_raw_memory_boundary(src, b"abcdefghi\n");
    assert_eq!(status, 1, "scanner grow-loop range check failed");
}
