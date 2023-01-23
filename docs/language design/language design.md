# Design Goals

* Readability
	- Prefer keywords before special characters, see python, ruby, pascal
* High Level
	- Don't be afraid to hide complexity for the user, if the user does not care
* Expression of intent
	- If the user did not care about some detail, it should not be expressed in the code
* Formattable
	- Whitespace is often lost across editors/platforms, the language should be auto-formattable without relying on whitespace
* Normal Form
	- It's desireable for the language to be laid out and formatted the same, regardless of author
	- Should not have to rely on code standards, compiling code should be ok

Note: Python without significant whitespace is essentially just a built-in formatter. It will however allow the language to not rely on a formatter to keep consistency (allow some custom layouting)