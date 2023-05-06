# Informal Grammar used by the parser

This will be moved somewhere else at some point.

Definitions:
- `EOI` (end of item): the separator beween module items. It can be either the end of input of a new line separator.

```
module -> moduleitem*
moduleitem -> (typedef | expression) EOI

typedef -> "type" TypeSymbol


expression -> assignment | conditional | subexpr
assignment -> "val" symbol "=" expression
conditional -> "if" expression "then" expression "else" expression
subexpr -> logterm
logterm -> logfactor ( "||" logfactor )*
logfactor -> equality ( "&&" equality )*
equality -> comparison ( ("==" | "!=") comparison )*
comparison -> term ( (">" | ">=" | "<" | "<=") term )*
term -> factor ( ("+" | "-") factor )*
factor -> primary ( ("*" | "/") primary )*
primary -> IntLiteral | TypeSymbol | symbol | "(" subexpr ")" | "{" ( toplevelexpr (EOI toplevelexpr)* )?   "}" 

```
