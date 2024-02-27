use std::default;

use enum_iterator::{all, Sequence};

#[derive(Debug, PartialEq, Sequence, Clone, Default)]
pub enum FNLineType {
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
    #[default]
    Unparsed = 99,
    PartialLineSingle, // self contained partial line; may contain at least 1 ranged element, but also contains SOME non-invisible text
    PartialLineMiddle, // this line has ZERO opens or closes BUT it is between other valid PartialLine elements
    PartialLineMulti,  // This contains at least 1 dangling open or close
    SingularInvisible, // This line ONLY contains a singular, self-contained invisible line, with no other text
}

impl FNLineType {
    pub fn vec_of_line_types() -> Vec<FNLineType> {
        all::<FNLineType>().collect::<Vec<_>>()
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum FNRangedElementType {
    Boneyard { open: String, close: String },
    Note { open: String, close: String },
    Other { open: String, close: String },
}

impl FNRangedElementType {
    pub fn boneyard() -> Self {
        Self::Boneyard {
            open: String::from("/*"),
            close: String::from("*/"),
        }
    }

    pub fn note() -> Self {
        Self::Note {
            open: String::from("[["),
            close: String::from("]]"),
        }
    }

    pub fn get_open_and_close_patterns(&self) -> (String, String) {
        match self {
            FNRangedElementType::Boneyard { open, close }
            | FNRangedElementType::Note { open, close }
            | FNRangedElementType::Other { open, close } => (open.clone(), close.clone()),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum FNPartialLineType {
    SelfContained,
    OrphanedOpen,
    OrphanedClose,
    OrphanedOpenAndClose,
}
