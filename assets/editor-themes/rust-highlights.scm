; Fallback source-editor captures. The bundled language rules precede these
; and retain their more specific function, type, constant and macro roles.
; Use grammar roles, so module paths and opaque macro token trees are excluded.
[
  (_expression/identifier)
  (_pattern/identifier)
  (shorthand_field_identifier)
] @variable
(captured_pattern (identifier) @variable)
(shorthand_field_initializer (identifier) @variable)
