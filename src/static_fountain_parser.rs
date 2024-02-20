//  static_fountain_parser.rs
//  Beat
//
//  Copyright © 2016 Hendrik Noeller. All rights reserved.
//  Parts copyright © 2019-2023 Lauri-Matti Parppei. All rights reserved.

//  Parts copyright © 2024 Richard Mamaril Rodriguez. All rights reserved.

// This parser is based upon the ContinuousFountainParser from Lauri-Matti Parppei's Beat,
// which itself is based upon Writer by Hendrik Noeller.

// This has been ported / translated from Objective-C to Python.
// As a result, many omissions and structural changes are made,
// though the core parsing logic remains largely the same.

//  Relased under GPL

use enum_iterator::last;
use std::io::empty;
use std::str::FromStr;
use std::vec;
use unicode_segmentation::UnicodeSegmentation;

//use enum_iterator::previous;

use crate::fountain_consts::LineType;
use crate::fountain_line::FNLine;

// ----- Public Functions -----
pub fn get_parsed_lines_from_raw_string(text: String) -> Vec<FNLine> {
    /*
    This function takes a raw string `text` which represents an entire document or file.

    This returns a list of fountain-parsed `FNLine` objects. Each `FNLine` contains the `string`, the `LineType` for the line, and other metadata as properties.
    */

    let lines: Vec<FNLine> = get_unparsed_line_array_from_raw_string(Some(text));

    get_parsed_lines_from_line_vec(lines)
}

// ----- Private Functions -----

fn get_parsed_lines_from_line_vec(lines: Vec<FNLine>) -> Vec<FNLine> {
    // the actual parsing
    let mut index: usize = 0;

    let mut cloned_lines_vec: Vec<FNLine> = lines.clone();

    for (l, cur_line) in lines.iter().enumerate() {
        //println!("Index", index);
        let mut cur_clone = cur_line.clone();
        cur_clone.fn_type = parse_line_type_for(&cloned_lines_vec, index);

        // Check if previous line is supposed to actually be just action
        // (Characters need 1 empty line before and 1 NON-empty line after)

        if cur_clone.fn_type == LineType::Empty && l > 0 && cloned_lines_vec.len() > 0 {
            let prev: &mut FNLine = &mut cloned_lines_vec[l - 1].clone();

            if prev.fn_type == LineType::Character {
                prev.fn_type = LineType::Action;
            }
        }

        cloned_lines_vec[l] = cur_clone;
        index += 1;
    }

    cloned_lines_vec
}

fn get_unparsed_line_array_from_raw_string(text: Option<String>) -> Vec<FNLine> {
    let mut unparsed_lines: Vec<FNLine> = vec![];

    let raw_text = match text {
        Some(the_text) => the_text,
        None => String::from(""),
    };

    // if (text == None): text = ""
    let fixed_text = raw_text.replace("\r\n", "\n"); // Replace MS Word/Windows line breaks with unix newlines

    let raw_lines = fixed_text.lines();
    let mut position: i32 = 0; // To track at which position every line begins

    for r in raw_lines {
        unparsed_lines.push(FNLine {
            fn_type: LineType::Unparsed,
            string: r.to_string(),
            original_string: r.to_string(),
            position: position,
            ..Default::default()
        });
        let grapheme_len = r.graphemes(true).count();
        position += (grapheme_len + 1) as i32; // +1 is to account for newline character
    }

    unparsed_lines
}
// Parses the line type for given line. It *has* to know its line index.

fn parse_line_type_for(lines: &Vec<FNLine>, index: usize) -> LineType {
    let empty_line = FNLine {
        fn_type: LineType::Unparsed,
        ..Default::default()
    };
    let mut line: &FNLine = &empty_line;

    let line_option: Option<&FNLine> = lines.get(index);

    if let Some(line_ref) = line_option {
        line = line_ref;
    }

    let mut next_line: Result<&FNLine, &str> = Result::Err("No previous line.");
    let mut previous_line: Result<&FNLine, &str> = Result::Err("No next line.");

    if !lines.is_empty() {
        if index > 0 {
            previous_line = Ok(&lines[index - 1]);
        }
        if { index + 1 } < lines.len() {
            next_line = Ok(&lines[index + 1]);
        }
    }

    // Check if there is a previous line
    // If so, check if previous line is empty

    let previous_line_is_empty: bool = match previous_line {
        Ok(line) => line.fn_type == LineType::Empty,
        Err(_) => true,
    };

    // --------- Handle empty lines first
    let empty_lines_result: Option<LineType> = _check_if_empty_lines(line);
    if let Some(line_type) = empty_lines_result {
        return line_type;
    }

    // --------- Check FORCED elements
    let forced_element_result: Option<LineType> =
        _check_if_forced_element(line, &previous_line_is_empty);

    if let Some(line_type) = forced_element_result {
        return line_type;
    }

    // --------- Title page
    let title_page_result: Option<LineType> = _check_if_title_page_element(line, &previous_line);
    if let Some(line_type) = title_page_result {
        return line_type;
    }

    // --------- Transitions
    let transition_result: Option<LineType> = _check_if_transition(line, &previous_line_is_empty);
    if let Some(line_type) = transition_result {
        return line_type;
    }

    // Handle items which require an empty line before them.

    // --------- Heading
    let heading_result: Option<LineType> = _check_if_heading(line, &previous_line_is_empty);
    if let Some(line_type) = heading_result {
        return line_type;
    }

    /*

    // --------- Check for Dual Dialogue
    dual_dialogue_result: LineType  = cls._check_if_dual_dialogue(
        line=line,
        previous_line=previous_line,
        next_line=next_line
        )
    if dual_dialogue_result is not None:
        return dual_dialogue_result

    // --------- Character
    character_result: LineType  = cls._check_if_character(
        line=line,
        next_line=next_line,
        twoLinesOver=lines[index+2] if (index + 2 < len(lines)) else None,
        index=index,
        previous_line=previous_line
        )
    if character_result is not None:
        return character_result

    // --------- Dialogue or Parenthetical
    dialogue_or_parenthetical_result: LineType  = cls._check_if_dialogue_or_parenthetical(
        line=line,
        previous_line=previous_line,
    )
    if dialogue_or_parenthetical_result is not None:
        return dialogue_or_parenthetical_result */

    // --------- Default
    LineType::Action
}
// ---------- Parsing helper funcs ----------

/* def _only_uppercase_until_parenthesis(text: str): // Might want to move this func to helper_funcs to be cleaner
until_parenthesis = text.split("(")[0]
if (
    until_parenthesis == until_parenthesis.upper()
    and len(until_parenthesis) > 0
):
return True
return False */

// ---------- Parsing sub-functions ----------
fn _check_if_transition(line: &FNLine, previous_line_is_empty: &bool) -> Option<LineType> {
    if line.string.len() > 2
        && line.string.graphemes(true).last() == Some(":")
        && line.string == line.string.to_uppercase()
        && *previous_line_is_empty
    {
        return Some(LineType::TransitionLine);
    }

    None
}
/* def _check_if_dialogue_or_parenthetical(line: FNLine, previous_line: FNLine):
if line.string.startswith("  "):
    print("Non empty line here!")
if previous_line is None:
    return None

if (
    previous_line.isDialogue()
    and len(previous_line.string) > 0
    ):
    if (line.string[:1] == '(' ):
        return LineType.parenthetical
    return LineType.dialogue

if previous_line.fn_type == LineType.parenthetical:
    return LineType.dialogue

if line.string.startswith("  "):
    return LineType.dialogue */

fn _check_if_heading(line: &FNLine, previous_line_is_empty: &bool) -> Option<LineType> {
    if !previous_line_is_empty && line.string.len() >= 3 {
        return None;
    }
    let first_3_graphemes = line
        .string
        .graphemes(true)
        .take(3)
        .collect::<Vec<&str>>()
        .join("");

    let _ = match first_3_graphemes.as_str() {
        "int" => {}
        "ext" => {}
        "est" => {}
        "i/e" => {}
        _ => return None,
    };

    // To avoid words like "international" from becoming headings, the extension HAS to end with either dot, space or slash
    let next_grapheme = line.string.graphemes(true).skip(3).next();
    if next_grapheme == Some(".") || next_grapheme == Some(" ") || next_grapheme == Some("/") {
        return Some(LineType::Heading);
    }
    return None;
}

fn _check_if_forced_element(line: &FNLine, previous_line_is_empty: &bool) -> Option<LineType> {
    //TODO: use graphemes instead of character indexing

    let first_grapheme_option: Option<&str> = line.string.graphemes(true).nth(0);
    let last_grapheme_option: Option<&str> = line.string.graphemes(true).last();

    if first_grapheme_option == None || last_grapheme_option == None {
        return None;
    }

    let first_grapheme = first_grapheme_option.unwrap_or_default();
    let last_grapheme = last_grapheme_option.unwrap_or_default();

    // TODO: convert local string index to global (document) string index in order to check if a char is an escape backslash or not
    // Check for escaped characters
    // if (firstChar == '\\'):
    //    first_unescaped = hf.find_first_unescaped_char_in_string(line.string)
    //    if first_unescaped == "":
    //        return LineType.action // every character in the line is escaped i guess lmao
    //    firstChar = first_unescaped

    // --------- Forced whitespace
    let contains_only_whitespace: bool = line.string.trim().is_empty();

    // Save to use again later
    let two_spaces: bool = first_grapheme == " " && last_grapheme == " " && line.string.len() > 1; // Contains at least two spaces

    if contains_only_whitespace && !two_spaces {
        return Some(LineType::Empty);
    }

    // --------- Page Break
    if line.string == "===" {
        return Some(LineType::PageBreak);
    }

    // --------- FORCED Action or Shot
    if first_grapheme == "!" {
        // checks raw first char again, to enable escaping \\ a `!` char
        // Action or shot
        if line.string.len() > 1 {
            let second_grapheme_option = line.string.graphemes(true).nth(1);
            if second_grapheme_option == Some("!") {
                return Some(LineType::Shot);
            }
        }
        return Some(LineType::Action);
    }
    // --------- FORCED Heading / Slugline
    if first_grapheme == "." && !*previous_line_is_empty {
        // '.' forces a heading.
        // Because our American friends love to shoot their guns like we Finnish people love our booze,
        // screenwriters might start dialogue blocks with such "words" as '.44'
        if line.string.len() > 1 {
            let second_grapheme_option = line.string.graphemes(true).nth(1);

            match second_grapheme_option {
                Some(sg) => {
                    if sg != "." {
                        return Some(LineType::Heading);
                    }
                    return None;
                }
                None => return None,
            };
        } else {
            return Some(LineType::Heading);
        }
    }

    // Rest of the FORCED FNLine Types
    match first_grapheme {
        ">" => {
            if last_grapheme == "<" {
                return Some(LineType::Centered);
            }
            return Some(LineType::TransitionLine);
        }
        "~" => Some(LineType::Lyrics),
        "=" => Some(LineType::Synopse),
        "#" => Some(LineType::Section),
        "@" => {
            if last_grapheme == "^" && *previous_line_is_empty {
                return Some(LineType::DualDialogueCharacter);
            }
            Some(LineType::Character)
        }
        "." => {
            if *previous_line_is_empty {
                return Some(LineType::Heading);
            }
            return None;
        }
        _ => None,
    }
}

fn _check_if_title_page_element(
    line: &FNLine,
    previous_line: &Result<&FNLine, &str>,
) -> Option<LineType> {
    if let Ok(pl) = previous_line {
        if !pl.is_title_page() {
            return None;
        }
    }

    let key: String = line.get_title_page_key();

    if key.len() > 0 && !key.is_empty() {
        match key.as_str() {
            "title" => {
                return Some(LineType::TitlePageTitle);
            }
            "author" => return Some(LineType::TitlePageAuthor),
            "authors" => return Some(LineType::TitlePageAuthor),
            "credit" => return Some(LineType::TitlePageCredit),
            "source" => return Some(LineType::TitlePageSource),
            "contact" => return Some(LineType::TitlePageContact),
            "contacts" => return Some(LineType::TitlePageContact),
            "contact info" => return Some(LineType::TitlePageContact),
            "draft date" => return Some(LineType::TitlePageDraftDate),
            "draft" => return Some(LineType::TitlePageDraftDate),
            _ => return Some(LineType::TitlePageUnknown),
        }
    }

    if let Ok(pl) = previous_line {
        let prev_key = pl.get_title_page_key();
        if prev_key.len() > 0 || line.string.starts_with("\t") || line.string.starts_with("   ") {
            return Some(pl.fn_type.clone());
        }
    }
    None
}

/* def _check_if_character(line: FNLine, next_line: FNLine, twoLinesOver: FNLine, index: i32, previous_line: FNLine) -> LineType:

if not (hf.only_uppercase_until_parenthesis(line.string) and line.string != ""):
    return None

if line.string != line.string.strip():
    if line.string.startswith("  "):
        return None

lastChar = line.string[-1:]
// A character line ending in ^ is a dual dialogue character
// (94 = ^, we'll compare the numerical value to avoid mistaking Tuskic alphabet character Ş as ^)
#if list(line.noteRanges) != []:
    #if sorted(list(line.noteRanges))[0] != 0: // get first ordered numerical value in noteRanges? #NOTE: Not 100% sure what this condition is tbh
if (ord(lastChar) == 94):

    // Note the previous character cue that it's followed by dual dialogue
    // self.makeCharacterAwareOfItsDualSiblingFrom(index) #NOTE: Does the parser need to be responsible for this?
    return LineType.dualDialogueCharacter

// Check if this line is actually just an ALLCAPS action line
if previous_line is not None:
    if previous_line.fn_type != LineType.empty:
        return LineType.action

return LineType.character */

fn _check_if_empty_lines(line: &FNLine) -> Option<LineType> {
    if line.string.len() == 0 {
        Some(LineType::Empty)
    } else {
        None
    }
}
/* def _check_if_dual_dialogue(line: FNLine, previous_line: FNLine, next_line: FNLine = None,) -> LineType:
if previous_line is not None:
    if (
        previous_line.isDualDialogue()
        )

        if line.string[0] == "(":
            return LineType.dualDialogueParenthetical
        else:
            return LineType.dualDialogue

else:
    return None */
