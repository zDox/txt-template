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
<template>		::= <locale>? <item>+
<locale>		  ::= <chars> "\n"
<item>			  ::= <text> | <key> | <option> | <constant>
<text>     		::= (<chars> | <whitspace> | [0-9])+
<key>      		::= "{" <ident> <default>? "}"
<default>  		::= ":" <item>
<option>   		::= "$" <key>
<constant> 		::= "$" <ident>
<ident>    		::= (<char> | [0-9])+
<whitspace>		::= (" " | "\t" | "\n")
<whitspaces>	::= <whitspace>+
<char>     		::= ([A-Z] | [a-z])
<chars>    		::= <char>+
```
