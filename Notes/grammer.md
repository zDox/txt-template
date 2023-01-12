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
```bnf
<template>		::= <settings>? <token>+
<settings>    ::= <setting>+ "end-of-settings!"
<setting>     ::= <locale> /* |  any other settings */
<locale>      ::= "locale" <whitespaces> ":" <whitespaces> /* a valid locale value (managed externally) */
<token>			  ::= <text> | <key> | <option> | <constant>
<text>     		::= (<chars> | <whitspace> | [0-9])+
<key>      		::= "{" <ident> <default>? "}"
<option>   		::= "${" <ident> <default>? "}" 
<constant> 		::= "$" <ident>
<default>  		::= ":" <token>
<ident>    		::= (<char> | [0-9])+
<whitspace>		::= (" " | "\t" | "\n")
<whitspaces>	::= <whitspace>+
<char>     		::= ([A-Z] | [a-z])
<chars>    		::= <char>+
```
