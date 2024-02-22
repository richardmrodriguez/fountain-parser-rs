# Fountain Parser RS

This library is for parsing [Fountain](https://fountain.io/syntax/)-formatted plain text files.

This provides the ability to categorize the elements of a Fountain-formatted screenplay, so that each element may be properly rendered by an external renderer (PDF, HTML, etc.).

## Status

The Static Parser now handles all screenplay elements, except for Notes and Boneyards. The static parser does not handle *emphasis* such as Bold, Italic, or Underlines.

See the tests in `lib.rs`.

## Special Thanks

This parser has been adapted from the parsing code for [Beat](https://github.com/lmparppei/Beat) by Lauri-Matti Parppei. It would not be possible without that.

Thank you, Lauri!