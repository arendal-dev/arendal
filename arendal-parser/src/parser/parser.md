# Informal Grammar used by the parser

This will be moved somewhere else at some point.

Definitions:
- `EOI` (end of item): the separator beween module items. It can be either the end of input of a new line separator.

```
module -> moduleitem*
moduleitem -> toplevelexpr EOI
toplevelexpr -> assignment | expression
assignment -> "val" identifier "=" expression
expression -> term
term -> factor ( ("+" | "-") factor )*
factor -> primary ( ("*" | "/") primary )*
primary -> IntLiteral | TypeLiteral | identified | "(" expression ")" | "{" toplevelexpr* "}"

```
