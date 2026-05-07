# kp 補助ライブラリと streamio の組み合わせテスト

## stream_scanner_to_stdio_stdout_i32

neplg2:test[normalize_newlines]
stdin: "10 20 30\n"
stdout: "10\n20\n30\n"
```neplg2
#entry main
#indent 4
#target std

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
```

## stdio_stdin_to_stream_writer_stdout

neplg2:test[normalize_newlines]
stdin: "hello world\n"
stdout: "hello world"
```neplg2
#entry main
#indent 4
#target std

#import "std/stdio" as *
#import "std/streamio" as *
#import "std/iotarget" as *

fn main <()*>()> ():
    let line <str> read_line;
    unwrap_ok open WriteStream::Stdio
    |> write line
    |> flush
    |> close;
```

## stream_scanner_to_stream_writer_i32

neplg2:test[normalize_newlines]
stdin: "5 3\n1 2 3 4 5\n1 3\n2 5\n1 5\n"
stdout: "6\n14\n15\n"
```neplg2
#entry main
#indent 4
#target std

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
    fill_i32 pref pref_len 0;

    let mut i <i32> 1;
    while le i n:
        do:
            let a <i32> read sc;
            let im1 <i32> sub i 1;
            let prev_off <i32> mul im1 4;
            let prev_ptr <i32> add pref prev_off;
            let prev <i32> if and ge im1 0 lt im1 pref_len:
                then:
                    load_i32 prev_ptr
                else:
                    #intrinsic "unreachable" <> ()
            let cur <i32> add prev a;
            let cur_off <i32> mul i 4;
            let cur_ptr <i32> add pref cur_off;
            if and ge i 0 lt i pref_len:
                then:
                    store_i32 cur_ptr cur
                else:
                    #intrinsic "unreachable" <> ()
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
            let diff <i32> if and and ge l 0 lt l pref_len and ge r1 0 lt r1 pref_len:
                then:
                    let left <i32> load_i32 left_ptr;
                    let right <i32> load_i32 right_ptr;
                    sub right left
                else:
                    0
            set w writeln w diff;
            set k add k 1;

    set w flush w;
    close w;
    close sc;
    unwrap_ok dealloc pref mul pref_len 4;
```

## stream_scanner_to_stream_writer_i64

neplg2:test[normalize_newlines]
stdin: "6\n"
stdout: "13\n"
```neplg2
#entry main
#indent 4
#target std

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
```

## stream_scanner_to_stream_writer_f64

neplg2:test[normalize_newlines]
stdin: "3.5 -2.25 1e2\n"
stdout: "3.500000\n-2.250000\n100.000000\n"
```neplg2
#entry main
#indent 4
#target std

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
```

## stream_scanner_to_stream_writer_f32

neplg2:test[normalize_newlines]
stdin: "1.25\n"
stdout: "1.250000\n"
```neplg2
#entry main
#indent 4
#target std

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
```

## kpsearch_unique_and_count

neplg2:test[normalize_newlines]
stdout: "2 3\n1 2 5\n"
```neplg2
#entry main
#indent 4
#target std

#import "kp/kpsearch" as *
#import "core/result" as *
#import "core/mem" as *
#import "core/math" as *
#import "std/stdio" as *

fn main <()*>()> ():
    let len <i32> 6;
    let data <i32> unwrap_ok alloc mul len 4;
    fill_i32 data len 0;
    store_i32 add data 0 1;
    store_i32 add data 4 1;
    store_i32 add data 8 2;
    store_i32 add data 12 2;
    store_i32 add data 16 5;
    store_i32 add data 20 5;

    let cnt2 <i32> count_equal_range_i32 data len 2;
    let new_len <i32> unique_sorted_i32 data len;
    print_i32 cnt2;
    print " ";
    println_i32 new_len;

    let mut i <i32> 0;
    while lt i new_len:
        do:
            if gt i 0:
                then print " "
                else ()
            let off <i32> mul i 4;
            let ptr <i32> add data off;
            print_i32 load_i32 ptr;
            set i add i 1;
    println "";
    unwrap_ok dealloc data mul len 4;
```
