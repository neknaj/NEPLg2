# json_typed_values.n.md

## json_string_payload_is_typed_str

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/encoding/json" as *
#import "alloc/string" as *
#import "core/option" as *

fn main <()*>i32> ():
    match json_as_string json_string "hello":
        Option::Some s:
            if str_eq s "hello" 1 0
        Option::None:
            0
```

## json_array_payload_is_typed_vec

neplg2:test
ret: 2
```neplg2
#entry main
#indent 4
#target std

#import "alloc/encoding/json" as *
#import "alloc/collections/vec" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    let arr0 <Vec<JsonValue>> unwrap_ok json_array_new;
    let arr1 <Vec<JsonValue>> unwrap_ok json_array_push arr0 json_number 1;
    let arr2 <Vec<JsonValue>> unwrap_ok json_array_push arr1 json_bool true;
    match json_as_array json_array arr2:
        Option::Some xs:
            len<JsonValue> xs
        Option::None:
            0
```

## json_object_nested_serialize

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/encoding/json" as *
#import "alloc/collections/vec" as *
#import "alloc/string" as *
#import "core/result" as *

fn main <()*>i32> ():
    let arr0 <Vec<JsonValue>> unwrap_ok json_array_new;
    let arr1 <Vec<JsonValue>> unwrap_ok json_array_push arr0 json_number 1;
    let arr2 <Vec<JsonValue>> unwrap_ok json_array_push arr1 json_bool true;
    let obj0 <Vec<JsonMember>> unwrap_ok json_object_new;
    let obj1 <Vec<JsonMember>> unwrap_ok json_object_push obj0 "name" json_string "nepl";
    let obj2 <Vec<JsonMember>> unwrap_ok json_object_push obj1 "items" json_array arr2;
    let out <str> json_serialize json_object obj2;
    if str_eq out "{\"name\":\"nepl\",\"items\":[1,true]}" 1 0
```

## json_serialize_escapes_string_body

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/encoding/json" as *
#import "alloc/string" as *

fn main <()*>i32> ():
    let out <str> json_serialize json_string "a\"b\\c\n";
    if str_eq out "\"a\\\"b\\\\c\\n\"" 1 0
```

## json_string_rejects_raw_handle_at_compile_time

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target std

#import "alloc/encoding/json" as *

fn main <()*>i32> ():
    let _v <JsonValue> json_string 0;
    0
```

## json_object_rejects_raw_handle_at_compile_time

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target std

#import "alloc/encoding/json" as *

fn main <()*>i32> ():
    let _v <JsonValue> json_object 0;
    0
```

## json_array_rejects_raw_handle_at_compile_time

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target std

#import "alloc/encoding/json" as *

fn main <()*>i32> ():
    let _v <JsonValue> json_array 0;
    0
```
