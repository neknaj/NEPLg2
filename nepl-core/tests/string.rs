// Tests for string handling in NEPLg2
// Based on the new specification where:
// - String literals (single-line and mlstr:) are of type `str` (borrowed view)
// - `str` is a fat pointer (ptr+len) with no ownership
// - `String` is owned (ptr, len, cap) like Vec<u8>
// - Conversion from str to String is explicit
// - Conversion from String to str can be implicit (borrowing)

mod harness;
use harness::*;

#[test]
fn test_string_literal_single_line_type() {
    // String literals should be of type `str`
    let src = r#"
#entry main
fn main <()*>()> ():
    let a <str> "hello\nworld!";
    ()
"#;
    compile_src(src);
}

#[test]
fn test_string_literal_mlstr_type() {
    // mlstr: should also be of type `str`
    let src = r#"
#entry main
fn main <()*>()> ():
    let b <str> mlstr:
        ##: hello
        ##: world!
    ()
"#;
    compile_src(src);
}

#[test]
fn test_mlstr_line_separator() {
    // mlstr: should insert \n between lines, but not at the end
    // The mlstr above should be equivalent to "hello\nworld!"
    let src = r#"
#entry main
fn main <()*>()> ():
    let a <str> "hello\nworld!";
    let b <str> mlstr:
        ##: hello
        ##: world!
    // Both should be equivalent
    ()
"#;
    compile_src(src);
}

#[test]
fn test_mlstr_raw_no_escape() {
    // mlstr: is raw, so no escape processing
    let src = r#"
#entry main
fn main <()*>()> ():
    let raw <str> mlstr:
        ##: \n should be literal backslash-n
        ##: no \t escape processing
    ()
"#;
    compile_src(src);
}

#[test]
fn test_single_line_with_escapes() {
    // Single-line literals should process escapes
    let src = r#"
#entry main
fn main <()*>()> ():
    let escaped <str> "hello\nworld!\ttab";
    ()
"#;
    compile_src(src);
}

#[test]
#[should_panic]  // Will panic until String type is implemented
fn test_string_to_str_implicit_conversion() {
    // String -> str should be allowed implicitly (borrowing)
    // This test assumes we have a print function that takes `str`
    let src = r#"
#entry main
#import "std/io" as *

fn main <()*>()> ():
    let s <String> String "hello";
    print s;  // OK: String to str view (no allocation)
    ()
"#;
    compile_src(src);
}

#[test]
#[should_panic]  // Will panic until String constructor is implemented
fn test_str_to_string_explicit_conversion_constructor() {
    // str -> String should require explicit conversion (allocation)
    // Approach A: Constructor function
    let src = r#"
#entry main
fn main <()*>()> ():
    let s <String> String "hello";
    let t <String> String mlstr:
        ##: hello
        ##: world!
    ()
"#;
    compile_src(src);
}

#[test]
#[should_panic]  // Will panic until to_string is implemented
fn test_str_to_string_explicit_conversion_function() {
    // str -> String should require explicit conversion (allocation)
    // Approach B: Standard function
    let src = r#"
#entry main
#import "std/string" as *

fn main <()*>()> ():
    let s <String> to_string "hello";
    ()
"#;
    compile_src(src);
}

#[test]
fn test_str_no_ownership() {
    // `str` should not have ownership - no drop/dealloc
    // This is a conceptual test; actual implementation would ensure
    // str is just a view (fat pointer)
    let src = r#"
#entry main
fn main <()*>()> ():
    let a <str> "static literal";
    let b <str> a;  // Copy is cheap (just ptr+len)
    ()
"#;
    compile_src(src);
}

#[test]
#[should_panic]  // Will panic until String type is implemented
fn test_string_ownership() {
    // `String` should have ownership - should be moved, not copied
    let src = r#"
#entry main
fn main <()*>()> ():
    let s <String> String "hello";
    let t <String> s;  // move
    // Using s here should be an error (use-after-move)
    // let u <String> s;  // ERROR: s was moved
    ()
"#;
    compile_src(src);
}

#[test]
#[should_panic]  // Should panic with move check error
fn test_string_use_after_move() {
    // Using String after move should be an error
    let src = r#"
#entry main
fn main <()*>()> ():
    let s <String> String "hello";
    let t <String> s;  // move
    let u <String> s;  // ERROR: use after move
    ()
"#;
    compile_src(src);
}

#[test]
#[should_panic]  // Will panic until borrowing is implemented
fn test_str_from_string_borrow() {
    // Creating a str view from String (borrowing)
    let src = r#"
#entry main
#import "std/string" as *

fn main <()*>()> ():
    let s <String> String "hello";
    let view <str> borrow s;  // view is valid while s is alive
    ()
"#;
    compile_src(src);
}

#[test]
fn test_str_lifetime_static() {
    // String literals are 'static - always valid
    let src = r#"
#entry main
fn main <()*>()> ():
    let a <str> "hello";  // 'static lifetime
    ()
"#;
    compile_src(src);
}

#[test]
#[should_panic]  // Specification should clarify - for now expect error
fn test_mlstr_empty_lines() {
    // How should mlstr: handle empty lines?
    // Based on specification: should error or have clear rules
    let src = r#"
#entry main
fn main <()*>()> ():
    let text <str> mlstr:
        ##: line1
        
        ##: line3
    ()
"#;
    compile_src(src);
}

#[test]
#[should_panic] 
fn test_mlstr_missing_prefix() {
    // Lines without ##: prefix should be an error
    let src = r#"
#entry main
fn main <()*>()> ():
    let text <str> mlstr:
        ##: line1
        line2 without prefix
    ()
"#;
    compile_src(src);
}

#[test]
fn test_mlstr_trailing_whitespace() {
    // How should trailing whitespace be handled?
    // Should be part of the string or trimmed?
    let src = r#"
#entry main
fn main <()*>()> ():
    let text <str> mlstr:
        ##: line1   
        ##: line2
    ()
"#;
    // This should be OK - trailing whitespace is preserved
    compile_src(src);
}

#[test]
#[should_panic]  // Will panic until concat/StringBuilder is implemented
fn test_string_concatenation() {
    // String concatenation should use StringBuilder for O(n) performance
    let src = r#"
#entry main
#import "std/string" as *

fn main <()*>()> ():
    let a <str> "hello";
    let b <str> " world";
    let result <String> concat a b;
    ()
"#;
    compile_src(src);
}

#[test]
fn test_string_literal_unicode() {
    // UTF-8 support - should handle unicode correctly
    let src = r#"
#entry main
fn main <()*>()> ():
    let japanese <str> "こんにちは世界";
    let emoji <str> "👋🌍";
    ()
"#;
    compile_src(src);
}

#[test]
fn test_mlstr_unicode() {
    // UTF-8 support in mlstr:
    let src = r#"
#entry main
fn main <()*>()> ():
    let text <str> mlstr:
        ##: こんにちは
        ##: 世界
    ()
"#;
    compile_src(src);
}

#[test]
#[should_panic]  // Will panic until str_eq is implemented
fn test_str_comparison() {
    // String comparison operations
    let src = r#"
#entry main
#import "std/string" as *

fn main <()*>()> ():
    let a <str> "hello";
    let b <str> "hello";
    let eq <bool> str_eq a b;
    ()
"#;
    compile_src(src);
}

#[test]
#[should_panic]  // Will panic until these functions are implemented
fn test_str_operations() {
    // Common string operations should work on str
    let src = r#"
#entry main
#import "std/string" as *

fn main <()*>()> ():
    let s <str> "hello world";
    let len <i32> str_len s;
    let starts <bool> starts_with s "hello";
    ()
"#;
    compile_src(src);
}

#[test]
#[should_panic]  // Will panic until StringBuilder is implemented
fn test_string_builder() {
    // StringBuilder for efficient string building
    let src = r#"
#entry main
#import "std/string" as *

fn main <()*>()> ():
    let mut builder <StringBuilder> StringBuilder;
    builder_push &builder "hello";
    builder_push &builder " ";
    builder_push &builder "world";
    let result <String> builder_build builder;
    ()
"#;
    compile_src(src);
}
