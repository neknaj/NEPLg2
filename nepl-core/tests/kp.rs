mod harness;

use harness::run_main_capture_stdout_with_stdin;

#[test]
fn kpread_to_stdio_stdout_i32() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "std/streamio" as *
#import "std/iotarget" as *
#import "std/stdio" as *

fn main <()*>()> ():
    let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;
    let a <i32> read sc;
    let b <i32> read sc;
    let c <i32> read sc;
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

fn main <()*>()> ():
    let line <str> read_line;
    unwrap_ok open WriteStream::Stdio
    |> write line
    |> flush
    |> close;
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
#import "core/mem" as *
#import "std/streamio" as *
#import "std/iotarget" as *

fn main <()*>()> ():
    let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;
    let n <i32> read sc;
    let q <i32> read sc;

    let pref_len <i32> add n 1;
    let pref <i32> unwrap_ok alloc mul pref_len 4;
    store_i32 pref 0;

    let mut i <i32> 1;
    while le i n:
        do:
            let a <i32> read sc;
            let im1 <i32> sub i 1;
            let prev_off <i32> mul im1 4;
            let prev_ptr <i32> add pref prev_off;
            let prev <i32> load_i32 prev_ptr;
            let cur <i32> add prev a;
            let cur_off <i32> mul i 4;
            let cur_ptr <i32> add pref cur_off;
            store_i32 cur_ptr cur;
            set i add i 1;

    let mut w <StreamWriter> unwrap_ok open WriteStream::Stdio;
    let mut k <i32> 0;
    while lt k q:
        do:
            let l1 <i32> read sc;
            let r1 <i32> read sc;
            let l <i32> sub l1 1;
            let left_off <i32> mul l 4;
            let right_off <i32> mul r1 4;
            let left_ptr <i32> add pref left_off;
            let right_ptr <i32> add pref right_off;
            let left <i32> load_i32 left_ptr;
            let right <i32> load_i32 right_ptr;
            let diff <i32> sub right left;
            set w writeln w diff;
            set k add k 1;

    set w flush w;
    close w;
    close sc;
    unwrap_ok dealloc pref mul pref_len 4;
"#;
    let out =
        run_main_capture_stdout_with_stdin(src, b"5 3\n1 2 3 4 5\n1 3\n2 5\n1 5\n");
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
    let n <i32> read sc;
    let ans <i64> ways n;
    close sc;
    unwrap_ok open WriteStream::Stdio
    |> writeln ans
    |> flush
    |> close;
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

fn main <()*>()> ():
    let v <i64> cast 13;
    unwrap_ok open WriteStream::Stdio
    |> writeln v
    |> flush
    |> close;
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

fn main <()*>()> ():
    unwrap_ok open WriteStream::Stdio
    |> writeln 6
    |> writeln 14
    |> writeln 15
    |> flush
    |> close;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"");
    assert_eq!(out, "6\n14\n15\n");
}

#[test]
fn kpread_to_kpwrite_f64() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "std/streamio" as *
#import "std/iotarget" as *

fn main <()*>()> ():
    let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;
    let a <f64> read sc;
    let b <f64> read sc;
    let c <f64> read sc;
    close sc;
    unwrap_ok open WriteStream::Stdio
    |> writeln a
    |> writeln b
    |> writeln c
    |> flush
    |> close;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"3.5 -2.25 1e2\n");
    assert_eq!(out, "3.500000\n-2.250000\n100.000000\n");
}

#[test]
fn kpread_to_kpwrite_f32() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "std/streamio" as *
#import "std/iotarget" as *

fn main <()*>()> ():
    let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;
    let v <f32> read sc;
    close sc;
    unwrap_ok open WriteStream::Stdio
    |> writeln v
    |> flush
    |> close;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"1.25\n");
    assert_eq!(out, "1.250000\n");
}

#[test]
fn kpread_scanner_functional() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "std/streamio" as *
#import "std/iotarget" as *

fn main <()*>()> ():
    let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;
    let a <i32> read sc;
    let b <i32> read sc;
    let c <i32> read sc;
    close sc;
    unwrap_ok open WriteStream::Stdio
    |> writeln a
    |> writeln b
    |> writeln c
    |> flush
    |> close;
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

fn main <()*>()> ():
    let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;
    let a <i32> read sc;
    let b <i32> read sc;
    close sc;
    let sum <i32> add a b;
    unwrap_ok open WriteStream::Stdio
    |> writeln sum
    |> flush
    |> close;
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
#import "std/stdio" as *

fn main <()*>()> ():
    let cap <i32> 64;
    let buf <i32> alloc_raw cap;
    let iov <i32> alloc_raw 8;
    let nread <i32> alloc_raw 4;

    store_i32 iov buf;
    store_i32 add iov 4 cap;
    store_i32 nread 0;

    let errno <i32> fd_read 0 iov 1 nread;
    let n <i32> load_i32 nread;

    print_i32 errno;
    print " ";
    print_i32 n;
    print " ";
    if lt 0 n:
        then:
            let b0 <i32> load_u8 buf;
            print_i32 b0
        else print_i32 -1;
    print " ";
    if lt 1 n:
        then:
            let b1 <i32> load_u8 add buf 1;
            print_i32 b1
        else print_i32 -1;
    print " ";
    if lt 2 n:
        then:
            let b2 <i32> load_u8 add buf 2;
            print_i32 b2
        else print_i32 -1;
    println "";
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"10 20 30\n");
    let parts: Vec<&str> = out.trim().split(' ').collect();
    assert_eq!(parts.len(), 5, "unexpected raw fd_read format: {out}");
    let errno: i32 = parts[0].parse().expect("errno parse");
    let n: i32 = parts[1].parse().expect("n parse");
    let b0: i32 = parts[2].parse().expect("b0 parse");
    let b1: i32 = parts[3].parse().expect("b1 parse");
    let b2: i32 = parts[4].parse().expect("b2 parse");
    assert_eq!(errno, 0, "fd_read errno should be 0: {out}");
    assert!(n > 0, "fd_read should read bytes: {out}");
    assert_eq!(b0, 49, "expected '1' at first byte: {out}");
    assert_eq!(b1, 48, "expected '0' at second byte: {out}");
    assert_eq!(b2, 32, "expected space at third byte: {out}");
}

#[test]
fn wasi_fd_read_raw_iovec_with_dealloc_debug() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/mem" as *
#import "std/stdio" as *

fn main <()*>()> ():
    let cap <i32> 64;
    let buf <i32> alloc_raw cap;
    let iov <i32> alloc_raw 8;
    let nread <i32> alloc_raw 4;

    store_i32 iov buf;
    store_i32 add iov 4 cap;
    store_i32 nread 0;

    let errno <i32> fd_read 0 iov 1 nread;
    let n <i32> load_i32 nread;
    dealloc_raw iov 8;
    dealloc_raw nread 4;

    print_i32 errno;
    print " ";
    print_i32 n;
    print " ";
    if lt 0 n:
        then:
            let b0 <i32> load_u8 buf;
            print_i32 b0
        else print_i32 -1;
    print " ";
    if lt 1 n:
        then:
            let b1 <i32> load_u8 add buf 1;
            print_i32 b1
        else print_i32 -1;
    print " ";
    if lt 2 n:
        then:
            let b2 <i32> load_u8 add buf 2;
            print_i32 b2
        else print_i32 -1;
    println "";
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"10 20 30\n");
    let parts: Vec<&str> = out.trim().split(' ').collect();
    assert_eq!(parts.len(), 5, "unexpected raw fd_read format: {out}");
    let errno: i32 = parts[0].parse().expect("errno parse");
    let n: i32 = parts[1].parse().expect("n parse");
    let b0: i32 = parts[2].parse().expect("b0 parse");
    let b1: i32 = parts[3].parse().expect("b1 parse");
    let b2: i32 = parts[4].parse().expect("b2 parse");
    assert_eq!(errno, 0, "fd_read errno should be 0: {out}");
    assert!(n > 0, "fd_read should read bytes: {out}");
    assert_eq!(b0, 49, "expected '1' at first byte: {out}");
    assert_eq!(b1, 48, "expected '0' at second byte: {out}");
    assert_eq!(b2, 32, "expected space at third byte: {out}");
}

#[test]
fn wasi_fd_read_then_alloc_header_debug() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/mem" as *
#import "std/stdio" as *

fn main <()*>()> ():
    let cap <i32> 64;
    let buf <i32> alloc_raw cap;
    let iov <i32> alloc_raw 8;
    let nread <i32> alloc_raw 4;

    store_i32 iov buf;
    store_i32 add iov 4 cap;
    store_i32 nread 0;
    let errno <i32> fd_read 0 iov 1 nread;
    let n <i32> load_i32 nread;

    dealloc_raw iov 8;
    dealloc_raw nread 4;

    let sc <i32> alloc_raw 12;
    store_i32 sc buf;
    store_i32 add sc 4 n;
    store_i32 add sc 8 0;

    print_i32 errno;
    print " ";
    print_i32 n;
    print " ";
    print_i32 sc;
    print " ";
    print_i32 buf;
    print " ";
    if lt 0 n:
        then:
            let b0 <i32> load_u8 buf;
            print_i32 b0
        else print_i32 -1;
    print " ";
    if lt 1 n:
        then:
            let b1 <i32> load_u8 add buf 1;
            print_i32 b1
        else print_i32 -1;
    print " ";
    if lt 2 n:
        then:
            let b2 <i32> load_u8 add buf 2;
            print_i32 b2
        else print_i32 -1;
    println "";
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"10 20 30\n");
    let parts: Vec<&str> = out.trim().split(' ').collect();
    assert_eq!(parts.len(), 7, "unexpected raw alloc-header format: {out}");
    let errno: i32 = parts[0].parse().expect("errno parse");
    let n: i32 = parts[1].parse().expect("n parse");
    let sc: i32 = parts[2].parse().expect("sc parse");
    let buf: i32 = parts[3].parse().expect("buf parse");
    let b0: i32 = parts[4].parse().expect("b0 parse");
    let b1: i32 = parts[5].parse().expect("b1 parse");
    let b2: i32 = parts[6].parse().expect("b2 parse");
    assert_eq!(errno, 0, "fd_read errno should be 0: {out}");
    assert!(n > 0, "fd_read should read bytes: {out}");
    assert!(sc > 0 && buf > 0, "pointers should be non-zero: {out}");
    assert_eq!(b0, 49, "expected '1' at first byte: {out}");
    assert_eq!(b1, 48, "expected '0' at second byte: {out}");
    assert_eq!(b2, 32, "expected space at third byte: {out}");
}

#[test]
fn local_scanner_new_logic_debug() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/mem" as *
#import "std/stdio" as *

fn scanner_new_local <()*>i32> ():
    let mut cap <i32> 65536;
    let mut buf <i32> alloc_raw cap;
    if:
        eq buf 0
        then:
            let sc0 <i32> alloc_raw 12;
            store_i32 sc0 0;
            store_i32 add sc0 4 0;
            store_i32 add sc0 8 0;
            sc0
        else:
            let iov <i32> alloc_raw 8;
            let nread_ptr <i32> alloc_raw 4;
            let mut len <i32> 0;
            let mut done <i32> 0;

            while eq done 0:
                do:
                    if:
                        eq len cap
                        then:
                            let new_cap <i32> mul cap 2;
                            let new_buf <i32> realloc_raw buf cap new_cap;
                            if:
                                eq new_buf 0
                                then:
                                    set done 1;
                                else:
                                    set buf new_buf;
                                    set cap new_cap;
                        else:
                            ();

                    if:
                        eq done 0
                        then:
                            let write_ptr <i32> add buf len;
                            let rem <i32> sub cap len;
                            store_i32 iov write_ptr;
                            store_i32 add iov 4 rem;
                            store_i32 nread_ptr 0;
                            let errno <i32> fd_read 0 iov 1 nread_ptr;
                            if:
                                ne errno 0
                                then:
                                    set done 1;
                                else:
                                    let n <i32> load_i32 nread_ptr;
                                    if:
                                        eq n 0
                                        then:
                                            set done 1;
                                        else:
                                            set len add len n;
                        else:
                            ();

            dealloc_raw iov 8;
            dealloc_raw nread_ptr 4;

            let sc <i32> alloc_raw 12;
            store_i32 sc buf;
            store_i32 add sc 4 len;
            store_i32 add sc 8 0;
            sc

fn main <()*>()> ():
    let sc <i32> scanner_new_local;
    let buf <i32> load_i32 sc;
    let len <i32> load_i32 add sc 4;
    let b0 <i32> if lt 0 len load_u8 buf -1;
    let b1 <i32> if lt 1 len load_u8 add buf 1 -1;
    let b2 <i32> if lt 2 len load_u8 add buf 2 -1;
    print_i32 len;
    print " ";
    print_i32 b0;
    print " ";
    print_i32 b1;
    print " ";
    print_i32 b2;
    println "";
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"10 20 30\n");
    let parts: Vec<&str> = out.trim().split(' ').collect();
    assert_eq!(parts.len(), 4, "unexpected local scanner format: {out}");
    let len: i32 = parts[0].parse().expect("len parse");
    let b0: i32 = parts[1].parse().expect("b0 parse");
    let b1: i32 = parts[2].parse().expect("b1 parse");
    let b2: i32 = parts[3].parse().expect("b2 parse");
    assert!(len > 0, "input should be read: {out}");
    assert_eq!(b0, 49, "expected '1' at first byte: {out}");
    assert_eq!(b1, 48, "expected '0' at second byte: {out}");
    assert_eq!(b2, 32, "expected space at third byte: {out}");
}
