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
<key>      ::= "{" <ident> "}"         // done
<option>   ::= "$" <key>               // done
<constant> ::= "$" <ident>             // done
<ident>    ::= (<char> | [0-9])+       // done
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
### Case 3: <A> ::= "b"+
```rust
fn A(scanner: &mut Scanner) -> Result<(), ParseError> {
  scanner.scan(|sequence| match sequence.chars().last().unwrap() {
    'b' => Some(Action::Request),  // correct symbol: try to get another one
    _ => None,  // the new symbol in the sequence is not corret: finish
  })
}
```
`scan` can be used for different kind of sequences too E.g. using `Action::Return` for EBNF `*`.

> Any scanner methods which use callbacks like `scan` and `transform` need to be written
> from scratch inside the respective parsing method when porting to c++. This is because
> c++ doesn't offer pleasant passing of functions as arguments.

