# Informal Grammar used by the parser

This will be moved somewhere else at some point

```

expression -> primary
term -> primary ( ("+" | "-") primary )*
primary -> IntLiteral

```