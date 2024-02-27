# Fountain Parser RS

This library is for parsing [Fountain](https://fountain.io/syntax/)-formatted plain text files.

This provides the ability to categorize the elements of a Fountain-formatted screenplay, so that each element may be properly rendered by an external renderer (PDF, HTML, etc.).

## Status

The Static Parser now handles all screenplay elements, except for Notes and Boneyards. The static parser does not handle *emphasis* such as Bold, Italic, or Underlines.

Work is being done to handle ranged elements such as Notes, Boneyards, and Emphasis. 

The plan is to pre-parse the text by gathering indices for valid opening and closing patterns, for each ranged element type (Boneyards, Notes, etc.)

The indices will be used to create a stripped version of the document without any invisibles. (this includes any `*` or `_` which are specifically used to make bold, italic, or underlined text.)

To handle simple ranged elements, like a singular line that *only* contains a Note or a Boneyard, it is somewhat trivial. However, notes and boneyards can be more complex than this: they may be multiline, and they may share a Line with valid printable text.

```
[[Line with]] printable text
```

To effectively handle ranged elements, we must categorize any lines that contain the following:

- Notes or Boneyards alone on a singular line
- Notes or Boneyards in addition to printable text

Lines that contain both Notes/Boneyards AND printable text are `PartialLines`.

A `PartialLine` has basically only two states: `SelfContained` or `Orphaned`. Specifically, there are `OrphanedOpen`, `OrphanedClose`, and `OrphanedOpenAndClose`.

```
[[Orphaned open

Orphaned close]]


]]Orphaned open and close[[
```

Categorizing the state of lines that contain `Opens` `[[ , /*` or `Closes` `]] , */` patterns will enable us to determine where each note begins and ends. It also helps us recognize which lines specifically have valid printable text which must be extracted.

Using the indices and ranges of valid `Notes` and `Boneyards`, we can strip out these ranged elements into a copy of the lines in the document.

Then, the Static Parser simply parses the stripped lines to get the "actual" printable text and FNLineType for each line. However, the "real" indices for ranged elements are available, as is the "raw" copy of the document.

It will be the responsibility of the implementation to maintain parity between the "raw" lines and the "stripped" lines, using the indices for each ranged element type.

The pre-parsing and partial line handling is still being figured out, and
will be documented further as it becomes usable.

## Tests

To see which features are functional so far, the tests in `lib.rs`.

## Special Thanks

This parser has been adapted from the parsing code for [Beat](https://github.com/lmparppei/Beat) by Lauri-Matti Parppei. It would not be possible without that.

Thank you, Lauri!