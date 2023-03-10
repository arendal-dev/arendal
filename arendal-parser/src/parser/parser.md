# Informal Grammar used by the parser

This will be moved somewhere else at some point

```

toplevelexpr -> assignment | expression
assignment -> "val" identifier "=" expression
expression -> term
term -> factor ( ("+" | "-") factor )*
factor -> primary ( ("*" | "/") primary )*
primary -> IntLiteral | TypeLiteral | identified | "(" expression ")" | "{" toplevelexpr* "}"

```
