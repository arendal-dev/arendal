# Informal Grammar used by the parser

This will be moved somewhere else at some point

```

expression -> primary
term -> factor ( ("+" | "-") factor )*
factor -> primary ( ("*" | "/") primary )*
primary -> IntLiteral

```
