# Grammer of the templating language

## Examples which should be valid at some point:

```
Sehr ${a} {n},
{m}
$Mfg
$MyName  
```
```
Sehr ${Anrede} {name},
{nachricht}
$Mfg
$Sender
```
```
Sehr geehrte Frau {name},{nachricht}
Mit freundlichen Grüßen
Bar
```

## EBNF grammar
```
<template> ::= <locale>? <item>+
<locale>   ::= <chars> "\n"
<item> 	   ::= <text> | <key> | <option> | <constant>
<text>     ::= (<chars> | <ws>)+
<key>      ::= "{" <ident> <default>? "}"
<default>  ::= ":" <item>
<option>   ::= "$" <key>
<constant> ::= "$" <ident>
<ident>    ::= (<char> | [0-9])+
<ws>       ::= (" " | "\t" | "\n")+
<char>     ::= ([A-Z] | [a-z])
<chars>    ::= <char>+
```

### Corrections of the grammar
1. <text> ::= <ws>? <chars> (<chars> | <ws>)* ==> <text> ::= (<chars> | <ws>)+
It must be possible for the text non-terminal to only contain whitespace charactes too, so
multiple non-text elements can be chained together, only separated by a whitespace.


## Implementation of different production rules as pseudo rust code

### Case 1: <A> ::= terminal <B> terminal ...
```rust
fn A(scanner: &mut Scanner) -> Result<(), ParseError> {
  scanner.take(terminal)?;
  B(scanner)?;
  scanner.take(terminal)?;
  Ok(())
}
```
### Case 2: <A> ::= "b" | "c"+
```rust
fn A(scanner: &mut Scanner) -> Result<(), ParseError> {
  scanner.scan(|character| match character {
    'b' => Some(Action::Return),
    'c' => Some(Action::Request),
    _ => None,
  })
}
```
`scan` can be used for different kind of sequences too E.g. using `Action::Return` for EBNF `*`.

