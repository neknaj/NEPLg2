# alloc/gui/font registered face

このファイルは `alloc/gui/font/registered_face` が platform font API や browser `FontFace` ではなく、provider bytes owner と SFNT metadata parser を接続することを確認する。

## gui_font_registered_face_binds_resource_bytes_to_sfnt_metadata

[目的/もくてき]:
- `GuiFontResourceBytes` に含まれる byte payload と face index を SFNT metadata parser に渡します。
- 成功時は resource id、face id、selected face index、resource owner、metadata を同じ owner として保持します。
- 失敗時は typed registered face error と parser error を返し、resource owner を回収して解放できます。
- WOFF decode 境界が来るまでは `SfntOnly` 以外の decode policy を parse 前に拒否します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_binds_resource_bytes_to_sfnt_metadata\" count=21 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"resource id\" expected=\"7\" actual=\"7\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"face id\" expected=\"11\" actual=\"11\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"selected face index\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"owner resource len\" expected=\"96\" actual=\"96\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"metadata face index\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"metadata face count\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=6 status=ok kind=eq_i32 label=\"units per em\" expected=\"2048\" actual=\"2048\" message=\"\"\nassertion index=7 status=ok kind=eq_i32 label=\"glyph count\" expected=\"321\" actual=\"321\" message=\"\"\nassertion index=8 status=ok kind=bool label=\"invalid face registered kind\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=9 status=ok kind=bool label=\"invalid face parse kind\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=10 status=ok kind=eq_i32 label=\"invalid face owner len\" expected=\"96\" actual=\"96\" message=\"\"\nassertion index=11 status=ok kind=bool label=\"malformed registered kind\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=12 status=ok kind=bool label=\"malformed parse kind\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=13 status=ok kind=eq_i32 label=\"malformed owner len\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=14 status=ok kind=bool label=\"unsupported decode kind\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=15 status=ok kind=bool label=\"unsupported decode no parse\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=16 status=ok kind=eq_i32 label=\"unsupported decode owner len\" expected=\"96\" actual=\"96\" message=\"\"\nassertion index=17 status=ok kind=bool label=\"invalid raw face id rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=18 status=ok kind=bool label=\"registered face table success\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=19 status=ok kind=bool label=\"registered face table duplicate recovery\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=20 status=ok kind=bool label=\"registered face table duplicate face recovery\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font" as *
#import "alloc/io" as *
#import "core/gui" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/gui/font_resource" as *
#import "std/test" as *

fn sfnt_tag4 %fn i32 fn i32 fn i32 fn i32 i32 \a\b\c\d:
    or or or shl a 24 shl b 16 shl c 8 d

fn sfnt_push_u8 %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\byte:
    match byte_builder_push_u8 builder byte:
        Result::Ok next:
            Result::Ok next
        Result::Err error:
            byte_builder_error_free error
            Result::Err "push_u8"

fn sfnt_push_u16_be %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\value:
    match sfnt_push_u8 builder and shr_u value 8 255:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            sfnt_push_u8 b1 and value 255

fn sfnt_push_u32_be %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\value:
    match sfnt_push_u8 builder and shr_u value 24 255:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u8 b1 and shr_u value 16 255:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u8 b2 and shr_u value 8 255:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_u8 b3 and value 255

fn sfnt_push_zero_run %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\count:
    if:
        le count 0
        then:
            Result::Ok builder
        else:
            match sfnt_push_u8 builder 0:
                Result::Err message:
                    Result::Err message
                Result::Ok next:
                    sfnt_push_zero_run next sub count 1

fn sfnt_push_header %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\table_count:
    match sfnt_push_u32_be builder 65536:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 table_count:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 0:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 0:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    sfnt_push_u16_be b4 0

fn sfnt_push_record %impure fn ByteBuilder impure fn i32 impure fn i32 impure fn i32 Result ByteBuilder str \builder\tag\offset\length:
    match sfnt_push_u32_be builder tag:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u32_be b1 0:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u32_be b2 offset:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_u32_be b3 length

fn sfnt_push_valid_records %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_record builder sfnt_tag4 'h' 'e' 'a' 'd' 60 20:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_record b1 sfnt_tag4 'h' 'h' 'e' 'a' 80 10:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    sfnt_push_record b2 sfnt_tag4 'm' 'a' 'x' 'p' 90 6

fn sfnt_push_valid_tables %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_zero_run builder 18:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 2048:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_zero_run b2 4:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 1900:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_u16_be b4 65036:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            match sfnt_push_u16_be b5 200:
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok b6:
                                                    match sfnt_push_u32_be b6 65536:
                                                        Result::Err message:
                                                            Result::Err message
                                                        Result::Ok b7:
                                                            sfnt_push_u16_be b7 321

fn sfnt_finish %impure fn Result ByteBuilder str Result ByteBuf str \builder_result:
    match builder_result:
        Result::Err message:
            Result::Err message
        Result::Ok builder:
            match byte_builder_finish builder:
                Result::Err error:
                    byte_builder_error_free error
                    Result::Err "finish"
                Result::Ok bytes:
                    Result::Ok bytes

fn build_valid_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 96:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_header b0 3:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_valid_records b1:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                sfnt_push_valid_tables b2

fn registered_face_resource_from_bytes %fn ByteBuf fn Option i32 fn GuiFontDecodePolicy GuiFontResourceBytes \bytes\face_index\decode_policy:
    let path %GuiFontResourcePath unwrap_ok gui_font_resource_path_result "fonts/Test-Regular.ttf"
    let request %GuiFontResourceRequest unwrap_ok gui_font_resource_request path face_index none decode_policy
    gui_font_resource_bytes_new request GuiFontResourceSource::Vfs bytes

fn registered_face_error_kind_is %fn &GuiFontRegisteredFaceError fn GuiFontRegisteredFaceErrorKind bool \error\expected:
    gui_font_registered_face_error_kind_eq gui_font_registered_face_error_kind error expected

fn sfnt_parse_error_kind_is %fn GuiSfntParseErrorKind fn GuiSfntParseErrorKind bool \actual\expected:
    match actual:
        GuiSfntParseErrorKind::UnexpectedEof:
            match expected:
                GuiSfntParseErrorKind::UnexpectedEof:
                    true
                _:
                    false
        GuiSfntParseErrorKind::InvalidFaceIndex:
            match expected:
                GuiSfntParseErrorKind::InvalidFaceIndex:
                    true
                _:
                    false
        _:
            false

fn registered_face_parse_error_is %fn &GuiFontRegisteredFaceError fn GuiSfntParseErrorKind bool \error\expected:
    match gui_font_registered_face_error_parse_error error:
        Option::None:
            false
        Option::Some parse_error:
            sfnt_parse_error_kind_is gui_sfnt_parse_error_kind &parse_error expected

fn registered_face_parse_error_absent %fn &GuiFontRegisteredFaceError bool \error:
    match gui_font_registered_face_error_parse_error error:
        Option::None:
            true
        Option::Some _parse_error:
            false

fn invalid_raw_face_id_rejected %fn void bool \void:
    match gui_font_registered_face_request_from_raw 7 0:
        Result::Err error:
            match error:
                GuiError::InvalidCommand:
                    true
                _:
                    false
        Result::Ok _request:
            false

fn parse_valid_registered_face %impure fn void TestReport \void:
    match build_valid_sfnt:
        Result::Err _message:
            test_report_push test_report_new "gui_font_registered_face_binds_resource_bytes_to_sfnt_metadata" assert false
        Result::Ok bytes:
            let resource %GuiFontResourceBytes registered_face_resource_from_bytes bytes none GuiFontDecodePolicy::SfntOnly
            let registered_request %GuiFontRegisteredFaceRequest unwrap_ok gui_font_registered_face_request_from_raw 7 11
            let report %TestReport match gui_font_registered_face_register_bytes registered_request resource:
                Result::Err error:
                    gui_font_registered_face_error_free error
                    test_report_push test_report_new "gui_font_registered_face_binds_resource_bytes_to_sfnt_metadata" assert false
                Result::Ok face:
                    let resource_id %GuiFontResourceId gui_font_registered_face_resource_id &face
                    let face_id %GuiFontFaceId gui_font_registered_face_face_id &face
                    let owner_resource %&GuiFontResourceBytes gui_font_registered_face_resource_ref &face
                    let metadata %GuiSfntMetadata gui_font_registered_face_metadata &face
                    let metrics %GuiSfntMetrics gui_sfnt_metadata_metrics &metadata
                    let report0 %TestReport test_report_new "gui_font_registered_face_binds_resource_bytes_to_sfnt_metadata"
                    let report1 %TestReport test_report_push report0 assert_eq_i32 "resource id" 7 gui_font_resource_id_raw &resource_id
                    let report2 %TestReport test_report_push report1 assert_eq_i32 "face id" 11 gui_font_face_id_raw &face_id
                    let report3 %TestReport test_report_push report2 assert_eq_i32 "selected face index" 0 gui_font_registered_face_selected_face_index &face
                    let report4 %TestReport test_report_push report3 assert_eq_i32 "owner resource len" 96 gui_font_resource_bytes_len owner_resource
                    let report5 %TestReport test_report_push report4 assert_eq_i32 "metadata face index" 0 gui_sfnt_metadata_face_index &metadata
                    let report6 %TestReport test_report_push report5 assert_eq_i32 "metadata face count" 1 gui_sfnt_metadata_face_count &metadata
                    let report7 %TestReport test_report_push report6 assert_eq_i32 "units per em" 2048 gui_sfnt_metrics_units_per_em &metrics
                    let report8 %TestReport test_report_push report7 assert_eq_i32 "glyph count" 321 gui_sfnt_metrics_num_glyphs &metrics
                    gui_font_registered_face_free face
                    report8
            report

fn append_invalid_face_registered_case %impure fn TestReport TestReport \report0:
    match build_valid_sfnt:
        Result::Err _message:
            test_report_push report0 assert false
        Result::Ok bytes:
            let resource %GuiFontResourceBytes registered_face_resource_from_bytes bytes some 1 GuiFontDecodePolicy::SfntOnly
            let registered_request %GuiFontRegisteredFaceRequest unwrap_ok gui_font_registered_face_request_from_raw 7 11
            match gui_font_registered_face_register_bytes registered_request resource:
                Result::Ok face:
                    gui_font_registered_face_free face
                    test_report_push report0 assert false
                Result::Err error:
                    let kind_ok %bool registered_face_error_kind_is &error GuiFontRegisteredFaceErrorKind::InvalidFaceIndex
                    let parse_ok %bool registered_face_parse_error_is &error GuiSfntParseErrorKind::InvalidFaceIndex
                    let owner_len %i32 gui_font_resource_bytes_len gui_font_registered_face_error_resource_ref &error
                    gui_font_registered_face_error_free error
                    let report1 %TestReport test_report_push report0 assert "invalid face registered kind" kind_ok
                    let report2 %TestReport test_report_push report1 assert "invalid face parse kind" parse_ok
                    test_report_push report2 assert_eq_i32 "invalid face owner len" 96 owner_len

fn append_malformed_registered_case %impure fn TestReport TestReport \report0:
    let bytes %ByteBuf unwrap_ok io_bytebuf_from_str_result "AB"
    let resource %GuiFontResourceBytes registered_face_resource_from_bytes bytes none GuiFontDecodePolicy::SfntOnly
    let registered_request %GuiFontRegisteredFaceRequest unwrap_ok gui_font_registered_face_request_from_raw 7 11
    match gui_font_registered_face_register_bytes registered_request resource:
        Result::Ok face:
            gui_font_registered_face_free face
            test_report_push report0 assert false
        Result::Err error:
            let kind_ok %bool registered_face_error_kind_is &error GuiFontRegisteredFaceErrorKind::MalformedFontResource
            let parse_ok %bool registered_face_parse_error_is &error GuiSfntParseErrorKind::UnexpectedEof
            let owner_len %i32 gui_font_resource_bytes_len gui_font_registered_face_error_resource_ref &error
            gui_font_registered_face_error_free error
            let report1 %TestReport test_report_push report0 assert "malformed registered kind" kind_ok
            let report2 %TestReport test_report_push report1 assert "malformed parse kind" parse_ok
            test_report_push report2 assert_eq_i32 "malformed owner len" 2 owner_len

fn append_unsupported_decode_case %impure fn TestReport TestReport \report0:
    match build_valid_sfnt:
        Result::Err _message:
            test_report_push report0 assert false
        Result::Ok bytes:
            let resource %GuiFontResourceBytes registered_face_resource_from_bytes bytes none GuiFontDecodePolicy::SfntAndWoff
            let registered_request %GuiFontRegisteredFaceRequest unwrap_ok gui_font_registered_face_request_from_raw 7 11
            match gui_font_registered_face_register_bytes registered_request resource:
                Result::Ok face:
                    gui_font_registered_face_free face
                    test_report_push report0 assert false
                Result::Err error:
                    let kind_ok %bool registered_face_error_kind_is &error GuiFontRegisteredFaceErrorKind::UnsupportedDecodePolicy
                    let parse_absent %bool registered_face_parse_error_absent &error
                    let owner_len %i32 gui_font_resource_bytes_len gui_font_registered_face_error_resource_ref &error
                    gui_font_registered_face_error_free error
                    let report1 %TestReport test_report_push report0 assert "unsupported decode kind" kind_ok
                    let report2 %TestReport test_report_push report1 assert "unsupported decode no parse" parse_absent
                    test_report_push report2 assert_eq_i32 "unsupported decode owner len" 96 owner_len

fn registered_face_table_register_error_kind_is %fn &GuiFontRegisteredFaceTableRegisterError fn GuiFontRegisteredFaceTableRegisterErrorKind bool \error\expected:
    gui_font_registered_face_table_register_error_kind_eq gui_font_registered_face_table_register_error_kind error expected

fn registered_face_table_decode_policy_is_sfnt_only %fn GuiFontDecodePolicy bool \policy:
    match policy:
        GuiFontDecodePolicy::SfntOnly:
            true
        _:
            false

fn build_registered_face_for_table %impure fn i32 impure fn i32 Result GuiFontRegisteredFace str \resource_raw\face_raw:
    match build_valid_sfnt:
        Result::Err message:
            Result::Err message
        Result::Ok bytes:
            let resource %GuiFontResourceBytes registered_face_resource_from_bytes bytes none GuiFontDecodePolicy::SfntOnly
            let registered_request %GuiFontRegisteredFaceRequest unwrap_ok gui_font_registered_face_request_from_raw resource_raw face_raw
            match gui_font_registered_face_register_bytes registered_request resource:
                Result::Ok face:
                    Result::Ok face
                Result::Err error:
                    gui_font_registered_face_error_free error
                    Result::Err "register"

fn registered_face_table_success_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let record %GuiFontRegisteredFaceRecord gui_font_registered_face_table_entry_record &entry
    let resource_id %GuiFontResourceId gui_font_registered_face_record_resource_id &record
    let face_id %GuiFontFaceId gui_font_registered_face_record_face_id &record
    let face_ref %&GuiFontRegisteredFace gui_font_registered_face_table_entry_face_ref &entry
    let owner_resource %&GuiFontResourceBytes gui_font_registered_face_resource_ref face_ref
    let len_ok %bool eq 1 gui_font_registered_face_table_len &table
    let record_ok %bool and eq 13 gui_font_registered_face_record_resource_raw &record eq 17 gui_font_registered_face_record_face_raw &record
    let metadata_ok %bool and eq 0 gui_font_registered_face_record_selected_face_index &record and eq 1 gui_font_registered_face_record_face_count &record and eq 2048 gui_font_registered_face_record_units_per_em &record eq 321 gui_font_registered_face_record_glyph_count &record
    let resource_ok %bool and eq 96 gui_font_registered_face_record_byte_len &record and eq 96 gui_font_resource_bytes_len owner_resource registered_face_table_decode_policy_is_sfnt_only gui_font_registered_face_record_decode_policy &record
    let lookup_resource_ok %bool match gui_font_registered_face_table_lookup_resource_id &table resource_id:
        Option::Some lookup:
            eq 17 gui_font_registered_face_record_face_raw &lookup
        Option::None:
            false
    let lookup_face_ok %bool match gui_font_registered_face_table_lookup_face_id &table face_id:
        Option::Some lookup:
            eq 13 gui_font_registered_face_record_resource_raw &lookup
        Option::None:
            false
    gui_font_registered_face_table_free table
    gui_font_registered_face_table_entry_free entry
    and len_ok and record_ok and metadata_ok and resource_ok and lookup_resource_ok lookup_face_ok

fn registered_face_table_success_ok %impure fn void bool \void:
    match gui_font_registered_face_table_new 2:
        Result::Err _error:
            false
        Result::Ok table:
            match build_registered_face_for_table 13 17:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            false
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_table_success_callback

fn registered_face_table_duplicate_rejected_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFace bool \table\face:
    let face_resource %&GuiFontResourceBytes gui_font_registered_face_resource_ref &face
    let recovered_ok %bool and eq 1 gui_font_registered_face_table_len &table eq 96 gui_font_resource_bytes_len face_resource
    gui_font_registered_face_table_free table
    gui_font_registered_face_free face
    recovered_ok

fn registered_face_table_duplicate_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let first_record %GuiFontRegisteredFaceRecord gui_font_registered_face_table_entry_record &entry
    let resource_raw %i32 gui_font_registered_face_record_resource_raw &first_record
    match build_registered_face_for_table resource_raw 43:
        Result::Err _message:
            gui_font_registered_face_table_free table
            gui_font_registered_face_table_entry_free entry
            false
        Result::Ok duplicate_face:
            match gui_font_registered_face_table_register table duplicate_face:
                Result::Ok registration:
                    gui_font_registered_face_table_registration_free registration
                    gui_font_registered_face_table_entry_free entry
                    false
                Result::Err error:
                    let kind_ok %bool registered_face_table_register_error_kind_is &error GuiFontRegisteredFaceTableRegisterErrorKind::DuplicateResourceId
                    let storage_ok %bool is_none gui_font_registered_face_table_register_error_storage_error &error
                    let rejected %GuiFontRegisteredFaceTableRegisterRejected gui_font_registered_face_table_register_error_rejected error
                    let rejected_ok %bool gui_font_registered_face_table_register_rejected_with rejected @registered_face_table_duplicate_rejected_callback
                    gui_font_registered_face_table_entry_free entry
                    and kind_ok and storage_ok rejected_ok

fn registered_face_table_duplicate_recovery_ok %impure fn void bool \void:
    match gui_font_registered_face_table_new 2:
        Result::Err _error:
            false
        Result::Ok table:
            match build_registered_face_for_table 31 41:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            false
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_table_duplicate_callback

fn registered_face_table_duplicate_face_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let first_record %GuiFontRegisteredFaceRecord gui_font_registered_face_table_entry_record &entry
    let face_raw %i32 gui_font_registered_face_record_face_raw &first_record
    match build_registered_face_for_table 53 face_raw:
        Result::Err _message:
            gui_font_registered_face_table_free table
            gui_font_registered_face_table_entry_free entry
            false
        Result::Ok duplicate_face:
            match gui_font_registered_face_table_register table duplicate_face:
                Result::Ok registration:
                    gui_font_registered_face_table_registration_free registration
                    gui_font_registered_face_table_entry_free entry
                    false
                Result::Err error:
                    let kind_ok %bool registered_face_table_register_error_kind_is &error GuiFontRegisteredFaceTableRegisterErrorKind::DuplicateFaceId
                    let storage_ok %bool is_none gui_font_registered_face_table_register_error_storage_error &error
                    let rejected %GuiFontRegisteredFaceTableRegisterRejected gui_font_registered_face_table_register_error_rejected error
                    let rejected_ok %bool gui_font_registered_face_table_register_rejected_with rejected @registered_face_table_duplicate_rejected_callback
                    gui_font_registered_face_table_entry_free entry
                    and kind_ok and storage_ok rejected_ok

fn registered_face_table_duplicate_face_recovery_ok %impure fn void bool \void:
    match gui_font_registered_face_table_new 2:
        Result::Err _error:
            false
        Result::Ok table:
            match build_registered_face_for_table 47 59:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            false
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_table_duplicate_face_callback

fn main %impure fn void i32 \void:
    let report0 %TestReport parse_valid_registered_face
    let report1 %TestReport append_invalid_face_registered_case report0
    let report2 %TestReport append_malformed_registered_case report1
    let report3 %TestReport append_unsupported_decode_case report2
    let report4 %TestReport test_report_push report3 assert "invalid raw face id rejected" invalid_raw_face_id_rejected
    let report5 %TestReport test_report_push report4 assert "registered face table success" registered_face_table_success_ok
    let report6 %TestReport test_report_push report5 assert "registered face table duplicate recovery" registered_face_table_duplicate_recovery_ok
    let report7 %TestReport test_report_push report6 assert "registered face table duplicate face recovery" registered_face_table_duplicate_face_recovery_ok
    let shown test_report_print_stdout report7
    test_report_exit_code shown
```
