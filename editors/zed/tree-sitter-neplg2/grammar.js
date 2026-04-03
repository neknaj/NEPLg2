module.exports = grammar({
  name: "neplg2",

  extras: _ => [
    /[ \t\r]+/,
  ],

  word: $ => $.identifier,

  rules: {
    source_file: $ => repeat(choice(
      $.comment,
      $.doc_comment,
      $.directive,
      $.function_definition,
      $.struct_definition,
      $.enum_definition,
      $.trait_definition,
      $.impl_definition,
      $.expression_statement,
      $.newline,
    )),

    newline: _ => /\n+/,

    comment: _ => token(seq("//", /.*/)),
    doc_comment: _ => token(seq("//:", /.*/)),

    directive: $ => seq(
      "#",
      field("name", $.directive_name),
      repeat(choice(
        $.import_path,
        $.alias_clause,
        $.identifier,
        $.type_identifier,
        $.number,
        $.string,
        $.operator,
      )),
      optional($.newline),
    ),

    directive_name: _ => /[A-Za-z_][A-Za-z0-9_]*/,

    import_path: _ => /"([^"\\]|\\.)*"/,

    alias_clause: $ => seq(
      "as",
      field("alias", $.identifier),
    ),

    function_definition: $ => seq(
      "fn",
      field("name", $.identifier),
      optional($.generic_params),
      optional(field("signature", $.type_annotation)),
      optional(field("parameters", $.parameter_list)),
      ":",
      optional(field("body", $.suite)),
    ),

    struct_definition: $ => seq(
      "struct",
      field("name", choice($.identifier, $.type_identifier)),
      optional($.generic_params),
      ":",
      optional(field("body", $.struct_body)),
    ),

    enum_definition: $ => seq(
      "enum",
      field("name", choice($.identifier, $.type_identifier)),
      optional($.generic_params),
      ":",
      optional(field("body", $.enum_body)),
    ),

    trait_definition: $ => seq(
      "trait",
      field("name", choice($.identifier, $.type_identifier)),
      optional($.generic_params),
      ":",
      optional(field("body", $.suite)),
    ),

    impl_definition: $ => seq(
      "impl",
      optional($.generic_params),
      repeat1(choice(
        $.identifier,
        $.type_identifier,
        $.operator,
        $.type_annotation,
      )),
      ":",
      optional(field("body", $.suite)),
    ),

    generic_params: $ => seq(
      "<",
      repeat1(choice(
        $.type_identifier,
        $.identifier,
        $.operator,
        $.type_annotation,
        ",",
      )),
      ">",
    ),

    type_annotation: $ => seq(
      "<",
      repeat1(choice(
        $.type_identifier,
        $.identifier,
        $.operator,
        $.number,
        $.string,
        $.type_annotation,
        $.parameter_list,
      )),
      ">",
    ),

    parameter_list: $ => seq(
      "(",
      optional(seq(
        $.identifier,
        repeat(seq(",", $.identifier)),
      )),
      ")",
    ),

    struct_body: $ => repeat1(choice(
      $.doc_comment,
      $.comment,
      $.field_definition,
      $.newline,
    )),

    field_definition: $ => seq(
      field("name", $.identifier),
      field("type", $.type_annotation),
      optional($.newline),
    ),

    enum_body: $ => repeat1(choice(
      $.doc_comment,
      $.comment,
      $.enum_variant,
      $.newline,
    )),

    enum_variant: $ => seq(
      field("name", choice($.identifier, $.type_identifier)),
      optional(field("payload", $.type_annotation)),
      optional($.newline),
    ),

    suite: $ => repeat1(choice(
      $.comment,
      $.doc_comment,
      $.directive,
      $.field_definition,
      $.enum_variant,
      $.expression_statement,
      $.newline,
    )),

    expression_statement: $ => seq(
      repeat1(choice(
        $.keyword,
        $.identifier,
        $.type_identifier,
        $.number,
        $.string,
        $.operator,
        $.type_annotation,
        $.parameter_list,
      )),
      optional(choice(";", $.newline)),
    ),

    keyword: _ => choice(
      "fn",
      "let",
      "mut",
      "set",
      "if",
      "then",
      "else",
      "while",
      "match",
      "struct",
      "enum",
      "trait",
      "impl",
      "block",
      "do",
      "cond",
      "pub",
      "use",
      "as",
    ),

    identifier: _ => /[a-z_][A-Za-z0-9_]*/,
    type_identifier: _ => /[A-Z.][A-Za-z0-9_.]*/,
    number: _ => /-?[0-9]+(\.[0-9]+)?/,
    string: _ => /"([^"\\]|\\.)*"/,
    operator: _ => choice(
      "->",
      "*>",
      "::",
      ";",
      ":",
      ",",
      ".",
      "@",
      "&",
      "*",
      "-",
      "=",
      "|>",
    ),
  }
});
