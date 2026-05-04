; Function text objects
(function_definition) @function.around

(function_definition
  body: (block) @function.inside)

; Struct text objects
(struct_definition) @class.around

(struct_definition
  (struct_body) @class.inside)

; Enum text objects
(enum_definition) @class.around

(enum_definition
  (enum_body) @class.inside)

; Trait text objects
(trait_definition) @class.around

(trait_definition
  (trait_body) @class.inside)

; Impl block text objects
(impl_block) @class.around

(impl_block
  (impl_body) @class.inside)

; Parameter text objects
(parameter) @parameter.around

(parameters
  (parameter) @parameter.inside)

(closure_parameter) @parameter.around

; Block text objects
(block) @block.around

; Comment text objects
(line_comment) @comment.around
(block_comment) @comment.around

; Conditional text objects
(if_expression) @conditional.around

(if_expression
  consequence: (block) @conditional.inside)

(match_expression) @conditional.around

; Loop text objects
(for_statement) @loop.around

(for_statement
  body: (block) @loop.inside)

(while_statement) @loop.around

(while_statement
  body: (block) @loop.inside)

(loop_statement) @loop.around

(loop_statement
  (block) @loop.inside)

; Call text objects
(call_expression) @call.around

(method_call_expression) @call.around
