# Syntax

## Layout Blocks

The fundamental syntax structure in `Fudge` is the `Layout Block`.

A layout block is a statement or expression, like a function call, function declaration, or an if-statement. `Fudge` uses keywords for opening and closing complex layout block, unlike many other languanges that use braces.

This is done to increase readability and accessibility. See the [Layout Design Page](language%20design/layout.md) for more info.

The components of a `Layout Block` are:
 * Header
 * Opener
 * Body
 * Footer
 * Linker
 
*Example of a simple layout block* 
```pascal
myFunction(arg1, arg2) // This expression is a single layout block
```

*Example of a layout block including a statement body:* 
```pascal
if a then // Layout Block starts on this line
	b     // Layout Body
end       // Layout Block ends on this line
```
## Signifiant Indentation

`Fudge` enforces a strict set of rules around indentation, similar to languages that use indentation to control execution flow.

The grammar is, however, still independent of indentation or line breaks. This is to avoid ambiguous execution flow if such formatting is somehow lost or distorted.

Not many languages enforce indentation while not using it for grammar. This might be seen as an example of harmful [Syntactic Salt](https://wiki.c2.com/?SyntacticSalt) but we justify it with:
 * Clearer error messaging. The error reporter can more easily pinpoint problems and recover if there are clear layout rules for code.
 * Cleaner code across multiple collaborators.
 * No unintentional mixing of tabs and spaces.
 * It allows for things like indented multi-line strings to be unambiguous in terms of whitespace.
 
All indentation is done with `Tabs`, see the [Indentation Design Page](language%20design/indentation.md) for more info.

### Indentation Rules
 1. #### **A `Layout Block` must opened and closed on the same indentation level**
	```pascal
	// OK, the block is completed on the same line it started
	myFunction(arg1, arg2)
	
	// OK, the block is completed on the same indentation level it started
	if a then 
		b
	end
		
	// Invalid, the Footer `end` is not aligned with the Header `if`
	if a then b
		end
	```
 2. #### **A `Layout Component` that would otherwise be on a single line, may be broken up into several, with subsequent lines being indented once**
	```pascal
	// OK, this line has no explicit footer that needs to be aligned on the same indentation level
	myFunction(arg1,
		arg2)
		
	// OK, all subsequent lines have the same indentation
	myFunction(
		arg1,
		arg2)
		
	// Invalid, the line continuation is not indented
	myFunction(arg1,
	arg2)
	
	// Invalid, the line continuation is indented twice
	myFunction(arg1,
			arg2)
 3. #### **If a `Header` is spanning multiple lines, the `Opener` must be on a separate line and have the same indentation level as the `Header`**
    ```pascal
    // OK, the opener `then` is on the same indentation level as the header `if`
    if myFunction(
        arg1, arg2)
    then
        b
    end

    // Invalid, the opener `then` is not on a separate line
    if myFunction(
        arg1, arg2) then
        b
    end
    ```