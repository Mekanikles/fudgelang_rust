# Layout
Syntax for building layout blocks in code is core to the readability of the language. `Fudge` should not rely on line-breaks and [Indentation](indentation.md) to disambiguate grammar (TODO: add link to reasoning for this), so unlike other languages with significant whitespace, we also need to decide how to open/close layout blocks. 

It's also important for the `Fudge` language design that code looks as similar as possible across different code repositories, so given a block layout style, it should not invite style discussions around placement of tokens.

Let's explore using being/end style keywords vs using braces.

## Block Keywords
Using keywords like begin, end etc to open and close layout blocks seems more common in older languages that tried to be as close to natural language as possible. It can help increase readability but requires users to type more.

It might be difficult to sensibly use the same word in all contexts for opening and closing layout blocks, so a multitude of language keywords might have to be reserved for this function. Examples are: `Begin`, `End`, `Then`, `Else`, `Do`, `Case`.

One benefit, outside of readability, is that some keywords can be both openers and closers simultaneously.

```pascal
if expr then
	do_action()
end
else then
	do_else()
end
```

can become

```pascal
if expr then
	do_action()
else
	do_else()
end
```

Since the block keywords share the same "space" as other keywords (like if, for, while etc), it is necessary to delimit these from other non-layout statements. Without special characters, it seems that line breaks does a good job here.

## Braces
Using braces to structure code layout is common in modern languanges and a mainstay in any language with c-style syntax. If braces are forced for all layout blocks, it allows for easy unambiguous grammar, regardless of indentation.

It's common for brace-blocks to invite discussion and disagreements around placement, especially for the opening brace. Ususally, it is up to the code-standard and review process of a project to keep a certain layout, but it will likely lead to inconsistencies across different code repositories. Some lanuanges have a strong recommendations (and tooling) regarding style, like `Rust`, but do not enforce it.

Since a brace is directional, it can either close or open a block, but not both. A brace can also be considered a "high-frequency" character (as in the glyph containing lots of edges and details), and when code uses a lot of nesting this can lead to eye-strain. It also makes it harder to, at a glance, differentiate between left- and right-facing braces.

While a brace means less typing than the corresponding block keyword, some keyboard layouts have awkward placement and require key-modifiers to type braces. If used to layout code, braces will be very commonly typed characters, possibly leading to slower typing overall.

```
might_be_difficult_to_read({{x, y}, {a, b}})
```

However, if the language supports single-line (nested) blocks, braces will most likely be the only readable option.

## Summary

*Block Keywords:*
* Easy to read.
* More reserved keywords and characters to type.
* A single keyword can serve as both and opener and a closer.
* Requires line-breaks to delimit statements.

*Braces:*
* Easily produces unambiuous grammar.
* Has a lot of baggage around placement.
* Can look "noisy".
* Each block requires two distinct braces.
* Good editor support.

>Looking at other contemporary languanges, *Braces* seem like a safe alternative, but the "noisiness" and bike-shedding discussion around placement is discouraging. The inherent "blockiness" of keywords along with the ability to chain blocks in if-statements and switch-case constructs makes them seem more appropriate for `Fudge`.