# Informal Grammar used by the parser

This will be moved somewhere else at some point.

Definitions:
- `EOS` (end of statement): the separator beween module items. It can be either the end of input of a new line separator.

```
module -> statement*

statement -> (vstatement | expression) EOS
vstatement -> ("pub" | "pkg" | "") (typedef | assignment)

typedef -> "type" TypeSymbol ("" | typedesc)
typedesc -> tupledesc
tupledesc -> "(" tupledescitems? ")"
tupledescitems -> tupledescitem ( "," tupledescitem )* ("" | ",")
tupledescitem -> qtsymbol

assignment -> "let" symbol "=" expression
expression -> expr ( "then" expr )*
expr -> conditional | subexpr
conditional -> "if" expr "then" expr "else" expr
subexpr -> logterm
logterm -> logfactor ( "||" logfactor )*
logfactor -> equality ( "&&" equality )*
equality -> comparison ( ("==" | "!=") comparison )*
comparison -> term ( (">" | ">=" | "<" | "<=") term )*
term -> factor ( ("+" | "-") factor )*
factor -> primary ( ("*" | "/") primary )*
primary -> IntLiteral | TypeSymbol | Symbol | "(" expression ")" | "{" ( bstatement (EOS bstatement)* )?   "}" 
bstatement -> assignment | expression
qsymbol -> ( ((Symbol | TypeSymbol ))"::" )* Symbol
qtsymbol -> ( ((Symbol | TypeSymbol ))"::" )* TSymbol

```
