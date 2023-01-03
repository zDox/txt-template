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

## EBNF
<template> ::= (<text> | <key> | <option> | <constant>)+
<text>     ::= <ws>? <char> (<char> | <ws>)*
<key>      ::= "{" <ident> "}"                             /* case 1 */ 
<option>   ::= "$" <key>                                   /* case 1 */
<constant> ::= "$" <ident>                                 /* case 1 */
<ident>    ::= <char>+
<ws>       ::= (" " | "\t" | "\n")+
<char>     ::= ([A-Z] | [a-z])

## Implementation of different production rules

### Case 1: <A> ::= terminal <B> terminal ...
```rust
fn A(scanner: &mut Scanner) -> Result<(), ParseError> {
  scanner.take(terminal)?;
  B(scanner)?;
  scanner.take(terminal)?;
  Ok(())
}
```
### Case 2: <A> ::= "b" | "c"
```rust
fn A(scanner: &mut Scanner) -> Result<(), ParseError> {
  scanner.transform(|character| match character {
    'b' => Ok(()),
    'c' => Ok(()),
  })
}
```
