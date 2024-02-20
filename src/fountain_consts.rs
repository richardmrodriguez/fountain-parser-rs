use enum_iterator::{all, Sequence};

#[derive(Debug, PartialEq, Sequence, Clone)]
pub enum LineType {
    Empty = 0,
    Section = 1,
    Synopse = 2,
    TitlePageTitle = 3,
    TitlePageAuthor = 4,
    TitlePageCredit = 5,
    TitlePageSource = 6,
    TitlePageContact = 7,
    TitlePageDraftDate = 8,
    TitlePageUnknown = 9,
    Heading = 10,
    Action = 11,
    Character = 12,
    Parenthetical = 13,
    Dialogue = 14,
    DualDialogueCharacter = 15,
    DualDialogueParenthetical = 16,
    DualDialogue = 17,
    TransitionLine = 18,
    Lyrics = 19,
    PageBreak = 20,
    Centered = 21,
    Shot = 22,
    More = 23,             // fake element for exporting
    DualDialogueMore = 24, // fake element for exporting
    TypeCount = 25, // This is the the max number of line types, used in `for` loops and enumerations, can be ignored
    Unparsed = 99,
}

impl LineType {
    pub fn vec_of_line_types() -> Vec<LineType> {
        all::<LineType>().collect::<Vec<_>>()
    }
}
