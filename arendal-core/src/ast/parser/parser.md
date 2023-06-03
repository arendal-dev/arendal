# Informal Grammar used by the parser

This will be moved somewhere else at some point.

Definitions:
- `EOS` (end of statement): the separator beween module items. It can be either the end of input of a new line separator.

```
module -> statement*

statement -> (vstatement | expression) EOS
vstatement -> ("pub" | "pkg" | "") (typedef | assignment)
bstatement -> assignment | expression

typedef -> "type" TypeSymbol

assignment -> "let" symbol "=" expression
expression -> conditional | subexpr
conditional -> "if" expression "then" expression "else" expression
subexpr -> logterm
logterm -> logfactor ( "||" logfactor )*
logfactor -> equality ( "&&" equality )*
equality -> comparison ( ("==" | "!=") comparison )*
comparison -> term ( (">" | ">=" | "<" | "<=") term )*
term -> factor ( ("+" | "-") factor )*
factor -> primary ( ("*" | "/") primary )*
primary -> IntLiteral | TypeSymbol | Symbol | "(" subexpr ")" | "{" ( bstatement (EOS bstatement)* )?   "}" 
qsymbol -> ( ((Symbol | TypeSymbol ))"::" )* Symbol
qtsymbol -> ( ((Symbol | TypeSymbol ))"::" )* Symbol

```
