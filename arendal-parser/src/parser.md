# Informal Grammar used by the parser

This will be moved somewhere else at some point.

Definitions:
- `EOS` (end of statement): the separator beween module items. It can be either the end of input of a new line / whitespace separator.

```
input -> top_level_statement*

top_level_statement -> (definition | expression) EOS
definition -> ("pub" | "pkg" | "") (typedef | assignment)

typedef -> "type" TypeSymbol ("" | typedesc)
typedesc -> tupledesc
tupledesc -> "(" tupledescitems? ")"
tupledescitems -> tupledescitem ( "," tupledescitem )* ("" | ",")
tupledescitem -> qtsymbol

statement -> assignment | expression
assignment -> "let" symbol "=" expression
expression -> expr ( "then" expr )*
expr -> conditional | logterm
conditional -> "if" expr "then" expr "else" expr
logterm -> logfactor ( "||" logfactor )*
logfactor -> equality ( "&&" equality )*
equality -> comparison ( ("==" | "!=") comparison )*
comparison -> term ( (">" | ">=" | "<" | "<=") term )*
term -> factor ( ("+" | "-") factor )*
factor -> primary ( ("*" | "/") primary )*
primary -> (IntLiteral | TypeSymbol | Symbol | "(" expression ")" | "{" ( bstatement (EOS bstatement)* )? "}") ("" | type_ann) 
qsymbol -> ( ((Symbol | TypeSymbol ))"::" )* Symbol
qtsymbol -> ( ((Symbol | TypeSymbol ))"::" )* TSymbol
type_ann -> ":" qtsymbol

```
