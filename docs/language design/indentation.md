#  Indentation
It is important that this language has a standardized way of expressing structural layout. Code should share the same layout across repositories and the compiler should be able to leverage layout for error reporting.

There are 3 viable paths here:
* Tabs only
* Spaces only
* Mixed tabs and spaces

## Tabs Only
The primary point of tabs is to allow each user to define how wide indentation is. This is good for accessibility and readability across different user environments. An added bonus is that file sizes become smaller.

Using only tabs, this effectively disallows _vertical aligning_ of code, since you cannot trust any vertical alignment to be the same on different tab-widths.

>Wether or not vertical aligning is a good practise is questionable. It opens up for inconsistency across code and it certainly adds complexity for a compiler wanting to enforce indentation rules.

Without vertical aligning, multi-line strings and certain expressions become "uglier" as the opening characters will misalign the whole block.

## Spaces Only
Using only spaces, layout becomes less customizable for users. All code structure will look identical.

With this, vertical alignment comes very natural. You can now align any piece of code without worrying about readability for different users. Multi-line strings can be nicely formatted and any single-char alignment issues go away.

An issue for the compiler here is to differentiate between indentation and alignment. Python gets around this issue by requiring indentation to be at least a single space, but otherwise allow any length of indentation, as long as it stays consistent for the current scope.

>Possibly, a compiler could dictate that indentation is exactly 4 spaces, to get consistency across files and scopes within the same file.

Spaces-only also opens up for confusion around how a "layout block" looks. Should opening/closing tokens align vertically exactly, or should the closing token align with the opening statement?

>Haskell uses spaces and requires vertical alignment for sequences of statements. While this rule is very simple, I think it has a bit too much variance in how a block can be laid out. It also lends itself to blocks having a lot of empty space in front of them (leading to "big jumps" in indentation), if they were started at the end of a long expression. I would rather enforce a line break to bring the start of the block back to the current block indentation level.

## Mixed Tabs And Spaces
The only sensible rule here, is to say that indentation is done through tabs and alignment done through spaces. Any indentation would always be `n` tabs followed by `m` spaces. Anything else would not appear the same across different tab-widths, and is poorly supported across IDEs.

This means that any multi-line statement would need to use spaces to indent the lines following the first. 

>The compiler could enforce line-continuations to be indented with at least a single space. Checking for consistency for subsequent lines in the same statement.

Because of the tabs-before-spaces rule, this essentially disallows aligned inner expressions that open up a new scope, since that would require a line indentation of `n` tabs, `m` spaces, followed by another tab for the indented scope of that expression. Some languages disallow this, like Python.

>To not let this edge-case discard this whole approach, the compiler could possibly allow mixed tabs and spaces, as long as it would not lead to a `tab, space, tab` sequence. The user can always work around this by either not aligning the inner expression, or assigning the expression to a variable in a separate statement.

This choice would also require the user to differentiate between tabs and spaces when reading code, which can be awkward.

>It is recommended to use an editor which can separate indentation from alignment, a monospace font, and enabling rendering of whitespace. `VS Code` has an extension that preserves mixed tabs and spaces for indentation: [Tab-Indent Space-Align](https://marketplace.visualstudio.com/items?itemName=j-zeppenfeld.tab-indent-space-align).

## Summary

*Tabs Only:*
* Consistent and the most accessible.
* Allows for straight-forward layout rules.
* Does not support vertical alignment.

*Spaces Only:*
* Highest level of customizable layout.
* Can increase readability through vertical alignment.
* Can make layout rules more complex.
* Hard to enforce consistent indentation across scopes/files.
* Increases file size.

*Mixed Tabs And Spaces:*
* Solves inconsistency issues with Spaces Only while still allowing vertical aligment.
* Has edge cases that require workarounds.
* Poor editor support.
* Requires users to differentiate between tabs and spaces.

>Even though it cannot support vertical alignment, *Tabs Only* seems like the prudent choise for `Fudge`. The inconsistencies of *Spaces Only* or the workarounds needed for *Mixed Tabs And Spaces* feels hard to motivate. Possibly we can still support some vertical alignment by allowing arbitrary runs of spaces to exist within a line, just not directly after indentation.

