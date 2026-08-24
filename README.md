# diane

Inspired by Twin Peaks.

Simple TUI for capturing jounral records and looking at notes.

## Roadmap

### Bare minimun

- [x] Define the basic data model for the app
- [x] Make sure it compiles (kind of)
- [x] Settle for a file format
- [x] Write the file parsers, sink adn serializers
  - [x] Journal stream
  - [x] Archive stream
- [x] Write the capture as a CLI
  - [x] CLI takes simple text stream and appends to journal
- [x] Start drafting the TUI
  - [x] Popup for only capturing
    - [x] ASCii background (https://www.asciiart.eu/ as ref, pretty awesome)
  - [x] Draft a editable Theme to unify the style
  - [x] Default main UI screen
    - [x] Journal
    - [x] Archive
- [x] Refactor the code to reflect more TEA

### Nice to have

- [ ] TUI enhancements
  - [ ] Move every glyph and tidy up the usage of literals in ui.rs
  - [ ] Add scroll to the archive sidepane
  - [ ] Add scroll to the journal sidepane
- [ ] Usability
  - [ ] Add the possiblity of capturing the journal window
  - [ ] Allow navigating entries records and copying text to clipboard
  - [ ] Easy to configure default jounral adn archive paths
  - [ ] Edit archive notes inside Diane
  - [ ] Promote jounral entries to archives (?)
- [ ] TBD
