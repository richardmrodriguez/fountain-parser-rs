//
//  Line.m
//  Beat
//
//  Created by Hendrik Noeller on 01.04.16.
//  Copyright © 2016 Hendrik Noeller. All rights reserved.
//  (most) parts copyright © 2019-2021 Lauri-Matti Parppei / Lauri-Matti Parppei. All Rights reserved.

use std::collections::HashSet;

use unicode_segmentation::UnicodeSegmentation;

use crate::fountain_consts::LineType;
use crate::location_and_length::LocationAndLength;

#[derive(Clone, Debug, PartialEq)]
pub struct FNLine {
    pub fn_type: LineType,
    pub string: String,
    pub original_string: String,
    pub position: i32,        //  Position (starting index) )in document
    pub length: i32,          //  Length of string
    pub section_depth: i32, //  If the line is an outline element (section/heading) this value contains the section depth
    pub scene_number: String, //  If the line is an outline element, this value contains the scene number, but only after the outline structure has been updated
    pub color: String,        //  Color for outline element (`nil` or empty if no color is set)

    pub forced_character_cue: bool, //  This line was forced to be a character cue in editor

    // @interface Line() // syntax hurty : these 3 properties are private properties I guess
    //oldHash: i32,
    //cachedString: String,
    //beatRangesAndContents: HashMap,
    //pub lt: LineType, // ? does this need to be here idk lol

    //formattedAs: any
    //parser: any
    pub bold_ranges: HashSet<i32>,
    pub italic_ranges: HashSet<i32>,
    pub underlined_ranges: HashSet<i32>,
    pub bold_italic_ranges: HashSet<i32>,
    pub strikeout_ranges: HashSet<i32>,
    pub note_ranges: HashSet<i32>,
    pub omitted_ranges: HashSet<i32>,
    pub escape_ranges: HashSet<i32>,
    pub removal_suggestion_ranges: HashSet<i32>,
    //_uuid: uuid
}

impl Default for FNLine {
    fn default() -> Self {
        FNLine {
            fn_type: LineType::Unparsed,
            string: String::from(""),
            original_string: String::from(""),
            position: 0,
            length: 0,
            section_depth: 0,
            scene_number: String::from(""),
            color: String::from(""),
            forced_character_cue: false,
            bold_ranges: HashSet::default(),
            italic_ranges: HashSet::default(),
            underlined_ranges: HashSet::default(),
            bold_italic_ranges: HashSet::default(),
            strikeout_ranges: HashSet::default(),
            note_ranges: HashSet::default(),
            omitted_ranges: HashSet::default(),
            escape_ranges: HashSet::default(),
            removal_suggestion_ranges: HashSet::default(),
        }
    }
}

impl FNLine {
    pub fn get_loc_len(&self) -> LocationAndLength {
        LocationAndLength {
            location: self.position,
            length: self.string.len() as i32,
        }
    }

    //pragma mark - Element booleans
    

    pub fn can_be_split_paragraph(&self) -> bool{
        self.fn_type == LineType::Action
            ||self.fn_type == LineType::Lyrics 
            || self.fn_type == LineType::Centered
    }
    //  Returns TRUE for scene, section and synopsis elements
    pub fn is_outline_element(&self) -> bool{
        self.fn_type == LineType::Heading || self.fn_type == LineType::Section
    }

    //  Returns TRUE for any title page element
    pub fn is_title_page(&self) -> bool{
        if self.fn_type == LineType::TitlePageTitle ||
        self.fn_type == LineType::TitlePageCredit ||
        self.fn_type == LineType::TitlePageAuthor ||
        self.fn_type == LineType::TitlePageDraftDate ||
        self.fn_type == LineType::TitlePageContact ||
        self.fn_type == LineType::TitlePageSource // ||
        // self.fn_type == LineType::TitlePageUnknown
        {
            true
        }
        else {
            false
        }
                
    }
    

    //  Checks if the line is completely non-printing __in the eyes of parsing__.
    pub fn is_invisible(self) -> bool{
        self.fn_type == LineType::Section 
        //|| self.omitted
        || self.fn_type == LineType::Synopse 
        || self.is_title_page() // NOTE: ? why would title page be invisible?
    }

    //  Returns TRUE if the line type is forced
    /* pub fn is_forced(self) -> bool{
        self.numberOfPrecedingFormattingCharacters > 0
    } */
    


    //pragma mark Dialogue

    //  Returns `true` for ANY SORT OF dialogue element, including dual dialogue
    pub fn is_any_sort_of_dialogue(&self) -> bool{
        self.is_dialogue() || self.is_dual_dialogue()
    }

    //  Returns `true` for any dialogue element, including character cue
    pub fn is_dialogue(&self) -> bool{
        self.fn_type == LineType::Character 
        || self.fn_type == LineType::Parenthetical 
        || self.fn_type == LineType::Dialogue 
        || self.fn_type == LineType::More
    }

    //  Returns `true` for dialogue block elements, excluding character cues
    pub fn is_dialogue_element(&self) -> bool{
        //// Is SUB-DIALOGUE element
        self.fn_type == LineType::Parenthetical || self.fn_type == LineType::Dialogue
    }

    //  Returns `true` for any dual dialogue element, including character cue
    pub fn is_dual_dialogue(&self) -> bool{
        self.fn_type == LineType::DualDialogue 
        || self.fn_type == LineType::DualDialogueCharacter 
        || self.fn_type == LineType::DualDialogueParenthetical 
        || self.fn_type == LineType::DualDialogueMore
    }

    //  Returns `true` for dual dialogue block elements, excluding character cues
    pub fn is_dual_dialogue_element(&self) -> bool{
        self.fn_type == LineType::DualDialogueParenthetical 
        || self.fn_type == LineType::DualDialogue 
        || self.fn_type == LineType::DualDialogueMore
    }

    //  Returns `true` for ANY character cue (single || dual)
    pub fn is_any_character(&self) -> bool{
        self.fn_type == LineType::Character 
        || self.fn_type == LineType::DualDialogueCharacter
    }

    //  Returns `true` for ANY parenthetical line (single || dual)
    pub fn is_any_parenthetical(&self) -> bool{
        self.fn_type == LineType::Parenthetical 
        || self.fn_type == LineType::DualDialogueParenthetical
    }

    //  Returns `true` for ANY dialogue line (single || dual)
    pub fn is_any_dialogue(&self) -> bool{
    
        self.fn_type == LineType::Dialogue 
        ||self.fn_type == LineType::DualDialogue
    }

    // pragma mark - Title Page Stuff
    pub fn get_title_page_key(&self) -> String{
        if self.string.len() == 0{
            return String::from("");
        }
        if self.string.contains(":"){
            let i = self.string.find(":").unwrap();
            if i == 0 
                || self.string.graphemes(true).nth(0) == Some(" ")
                || self.string[..i].to_lowercase().ends_with(" to") // NOTE: maybe shouldn't be the responsibility of the title page key func to gatekeep transition lines
                {
                return String::from("");
            }
            return String::from(self.string[..i].to_lowercase());
        }
        String::from("")
    }

    /* pub fn getTitlePageValue(self) -> str:
        if ":" in self.string:
            i: int = self.string.index(":")
            if (i == None): 
                return self.string
            
            return self.string[i+1:].strip()
        elif self.string.strip() == "The Sequel": print("Amongus")
        else:
            return "" */
    

}
