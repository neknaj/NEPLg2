# kp 入出力の組み合わせテスト

## kpread_to_stdio_stdout_i32

neplg2:test[normalize_newlines]
stdin: "10 20 30\n"
stdout: "10\n20\n30\n"
```neplg2
#entry main
#indent 4
#target wasi

#import "kp/kpread" as *
#import "std/stdio" as *

fn main <()*>()> ():
    let sc <i32> scanner_new;
    println_i32 scanner_read_i32 sc;
    println_i32 scanner_read_i32 sc;
    println_i32 scanner_read_i32 sc;
```

## stdio_stdin_to_kpwrite_stdout

neplg2:test[normalize_newlines]
stdin: "hello world\n"
stdout: "hello world"
```neplg2
#entry main
#indent 4
#target wasi

#import "std/stdio" as *
#import "kp/kpwrite" as *

fn main <()*>()> ():
    let line <str> read_line;
    let w <i32> writer_new;
    writer_write_str w line;
    writer_flush w;
    writer_free w;
```

## kpread_to_kpwrite_i32

neplg2:test[normalize_newlines]
stdin: "5 3\n1 2 3 4 5\n1 3\n2 5\n1 5\n"
stdout: "6\n14\n15\n"
```neplg2
#entry main
#indent 4
#target wasi

#import "core/math" as *
#import "core/mem" as *
#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn main <()*>()> ():
    let sc <i32> scanner_new;
    let n <i32> scanner_read_i32 sc;
    let q <i32> scanner_read_i32 sc;

    let pref_len <i32> add n 1;
    let pref <i32> alloc mul pref_len 4;
    store_i32 pref 0;

    let mut i <i32> 1;
    while le i n:
        do:
            let a <i32> scanner_read_i32 sc;
            let prev_off <i32> mul sub i 1 4;
            let prev_ptr <i32> add pref prev_off;
            let prev <i32> load_i32 prev_ptr;
            let cur <i32> add prev a;
            let cur_off <i32> mul i 4;
            let cur_ptr <i32> add pref cur_off;
            store_i32 cur_ptr cur;
            set i add i 1;

    let w <i32> writer_new;
    let mut k <i32> 0;
    while lt k q:
        do:
            let l1 <i32> scanner_read_i32 sc;
            let r1 <i32> scanner_read_i32 sc;
            let l <i32> sub l1 1;
            let left_off <i32> mul l 4;
            let right_off <i32> mul r1 4;
            let left <i32> load_i32 add pref left_off;
            let right <i32> load_i32 add pref right_off;
            writer_write_i32 w sub right left;
            writer_writeln w;
            set k add k 1;

    writer_flush w;
    writer_free w;
    dealloc pref mul pref_len 4;
```

## kpread_to_kpwrite_i64

neplg2:test[normalize_newlines]
stdin: "6\n"
stdout: "13\n"
```neplg2
#entry main
#indent 4
#target wasi

#import "core/math" as *
#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn ways <(i32)*>i64> (n):
    if le n 1:
        then i64_extend_i32_u 1
        else:
            let mut a <i64> i64_extend_i32_u 1;
            let mut b <i64> i64_extend_i32_u 1;
            let mut i <i32> 2;
            while le i n:
                do:
                    let c <i64> i64_add a b;
                    set a b;
                    set b c;
                    set i add i 1;
            b

fn main <()*>()> ():
    let sc <i32> scanner_new;
    let n <i32> scanner_read_i32 sc;
    let ans <i64> ways n;
    let w <i32> writer_new;
    writer_write_i64 w ans;
    writer_writeln w;
    writer_flush w;
    writer_free w;
```

## kpread_to_kpwrite_f64

neplg2:test[normalize_newlines]
stdin: "3.5 -2.25 1e2\n"
stdout: "3.500000\n-2.250000\n100.000000\n"
```neplg2
#entry main
#indent 4
#target wasi

#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn main <()*>()> ():
    let sc <i32> scanner_new;
    let a <f64> scanner_read_f64 sc;
    let b <f64> scanner_read_f64 sc;
    let c <f64> scanner_read_f64 sc;
    let w <i32> writer_new;
    writer_write_f64_ln w a;
    writer_write_f64_ln w b;
    writer_write_f64_ln w c;
    writer_flush w;
    writer_free w;
```

## kpread_to_kpwrite_f32

neplg2:test[normalize_newlines]
stdin: "1.25\n"
stdout: "1.250000\n"
```neplg2
#entry main
#indent 4
#target wasi

#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn main <()*>()> ():
    let sc <i32> scanner_new;
    let v <f32> scanner_read_f32 sc;
    let w <i32> writer_new;
    writer_write_f32_ln w v;
    writer_flush w;
    writer_free w;
```

## kpsearch_unique_and_count

neplg2:test[normalize_newlines]
stdout: "3 3\n1 2 5\n"
```neplg2
#entry main
#indent 4
#target wasi

#import "kp/kpsearch" as *
#import "core/mem" as *
#import "core/math" as *
#import "std/stdio" as *

fn main <()*>()> ():
    let len <i32> 6;
    let data <i32> alloc mul len 4;
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
                else ();
            print_i32 load_i32 add data mul i 4;
            set i add i 1;
    println "";
    dealloc data mul len 4;
```
