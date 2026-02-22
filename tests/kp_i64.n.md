# kp i64 入出力テスト

## kpread_kpwrite_i64_roundtrip

neplg2:test[normalize_newlines]
stdin: "-9223372036854775808 0 9223372036854775807 18446744073709551615\n"
stdout: "-9223372036854775808\n0\n9223372036854775807\n18446744073709551615\n"
```neplg2
#entry main
#indent 4
#target std

#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn main <()*>()> ():
    let sc <i32> scanner_new;
    let w <i32> writer_new;

    let a <i64> scanner_read_i64 sc;
    let b <i64> scanner_read_i64 sc;
    let c <i64> scanner_read_i64 sc;
    let d <i64> scanner_read_u64 sc;

    writer_write_i64 w a;
    writer_writeln w;
    writer_write_i64 w b;
    writer_writeln w;
    writer_write_i64 w c;
    writer_writeln w;
    writer_write_u64 w d;
    writer_writeln w;
    writer_flush w;
    writer_free w;
```

## kpread_i64_sign_and_plus

neplg2:test[normalize_newlines]
stdin: "+42 -17 +0\n"
stdout: "42\n-17\n0\n"
```neplg2
#entry main
#indent 4
#target std

#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn main <()*>()> ():
    let sc <i32> scanner_new;
    let w <i32> writer_new;

    writer_write_i64 w scanner_read_i64 sc;
    writer_writeln w;
    writer_write_i64 w scanner_read_i64 sc;
    writer_writeln w;
    writer_write_u64 w scanner_read_u64 sc;
    writer_writeln w;
    writer_flush w;
    writer_free w;
```

## kpread_kpwrite_i64_near_bounds

neplg2:test[normalize_newlines]
stdin: "-9223372036854775807 9223372036854775806 1000000000000000000 18446744073709551614\n"
stdout: "-9223372036854775807\n9223372036854775806\n1000000000000000000\n18446744073709551614\n"
```neplg2
#entry main
#indent 4
#target std

#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn main <()*>()> ():
    let sc <i32> scanner_new;
    let w <i32> writer_new;

    writer_write_i64 w scanner_read_i64 sc;
    writer_writeln w;
    writer_write_i64 w scanner_read_i64 sc;
    writer_writeln w;
    writer_write_i64 w scanner_read_i64 sc;
    writer_writeln w;
    writer_write_u64 w scanner_read_u64 sc;
    writer_writeln w;
    writer_flush w;
    writer_free w;
```
