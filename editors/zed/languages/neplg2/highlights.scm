(comment) @comment
(doc_comment) @comment.doc

(directive
  name: (directive_name) @keyword)

(import_path) @string.special
(alias_clause
  alias: (identifier) @namespace)

(keyword) @keyword
(string) @string
(number) @number
(operator) @operator

(function_definition
  name: (identifier) @function)

(struct_definition
  name: [
    (identifier)
    (type_identifier)
  ] @type)

(enum_definition
  name: [
    (identifier)
    (type_identifier)
  ] @type)

(trait_definition
  name: [
    (identifier)
    (type_identifier)
  ] @type)

(field_definition
  name: (identifier) @property)

(enum_variant
  name: [
    (identifier)
    (type_identifier)
  ] @constant)

(type_annotation
  [
    (type_identifier)
  ] @type)

(parameter_list
  (identifier) @parameter)

(identifier) @variable
(type_identifier) @type
