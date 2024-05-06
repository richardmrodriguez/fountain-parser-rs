//  static_fountain_parser.rs

//  Copyright © 2016 Hendrik Noeller. All rights reserved.
//  Parts copyright © 2019-2023 Lauri-Matti Parppei. All rights reserved.
//  Parts copyright © 2024 Richard Mamaril Rodriguez. All rights reserved.

// This parser is based upon the ContinuousFountainParser from Lauri-Matti Parppei's Beat,
// which itself is based upon Writer by Hendrik Noeller.

// This has been ported / translated from Objective-C to Rust.
// As a result, many omissions and structural changes are made,
// though the core parsing logic remains largely the same.

//  Relased under GPL

//! The static_fountain_parser

use std::vec;
use unicode_segmentation::UnicodeSegmentation;

use crate::fountain_enums::FNLineType;
use crate::fountain_line::FNLine;

// ----- Public Functions -----

/// Returns a `Vector` of fountain-parsed `FNLine` objects for a raw text document string.
///
/// Each `FNLine` contains the `string`, the `FNLineType` for the line, and other metadata as properties.
pub fn get_parsed_lines_from_raw_string(text: String) -> Vec<FNLine> {
    let lines: Vec<FNLine> = get_unparsed_line_array_from_raw_string(Some(text));

    get_parsed_lines_from_line_vec(lines)
}

/// Splits the document by newlines, then returns a list of Unparsed `FNLine` objects.
///
/// Each `FNLine` object contains a single line of text, as well as metadata and attributes such as `FNLineType`
/// and the line's position within the document string.
pub fn get_unparsed_line_array_from_raw_string(text: Option<String>) -> Vec<FNLine> {
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
            fn_type: FNLineType::Unparsed,
            string: r.to_string(),
            raw_string: r.to_string(),
            position: position,
            ..Default::default()
        });
        let grapheme_len = r.graphemes(true).count();
        position += (grapheme_len + 1) as i32; // +1 is to account for newline character
    }

    unparsed_lines
}

pub fn get_parsed_lines_from_line_vec(lines: Vec<FNLine>) -> Vec<FNLine> {
    // the actual parsing
    let mut index: usize = 0;

    let mut cloned_lines_vec: Vec<FNLine> = lines.clone();

    for (l, cur_line) in lines.iter().enumerate() {
        //println!("Index", index);
        let mut cur_clone = cur_line.clone();
        cur_clone.fn_type = parse_line_type_for(&cloned_lines_vec, index);

        // Check if previous line is supposed to actually be just action
        // (Characters need 1 empty line before and 1 NON-empty line after)

        if cur_clone.fn_type == FNLineType::Empty && l > 0 && cloned_lines_vec.len() > 0 {
            let prev: &mut FNLine = &mut cloned_lines_vec[l - 1].clone();

            if prev.fn_type == FNLineType::Character {
                prev.fn_type = FNLineType::Action;
            }
        }

        cloned_lines_vec[l] = cur_clone;
        index += 1;
    }

    cloned_lines_vec
}

// ----- Private Functions -----

/// Parses and returns the `LineType` for a given line.
fn parse_line_type_for(lines: &Vec<FNLine>, index: usize) -> FNLineType {
    let empty_line = FNLine {
        fn_type: FNLineType::Unparsed,
        ..Default::default()
    };
    let mut line: &FNLine = &empty_line;

    let line_option: Option<&FNLine> = lines.get(index);

    if let Some(line_ref) = line_option {
        line = line_ref;
    }

    //let mut next_line: Result<&FNLine, &str> = Result::Err("No next line.");
    let mut previous_line: Result<&FNLine, &str> = Result::Err("No previous line.");

    if !lines.is_empty() {
        if index > 0 {
            previous_line = Ok(&lines[index - 1]);
        }
        if { index + 1 } < lines.len() {
            //next_line = Ok(&lines[index + 1]);
        }
    }

    // Check if there is a previous line
    // If so, check if previous line is empty

    let previous_line_is_empty: bool = match previous_line {
        Ok(line) => line.fn_type == FNLineType::Empty,
        Err(_) => true,
    };

    // --------- Handle empty lines first
    let empty_lines_result: Option<FNLineType> = _check_if_empty_line(line);
    if let Some(line_type) = empty_lines_result {
        return line_type;
    }

    // --------- Check FORCED elements
    let forced_element_result: Option<FNLineType> =
        _check_if_forced_element(line, &previous_line_is_empty);

    if let Some(line_type) = forced_element_result {
        return line_type;
    }

    // --------- Title page
    let title_page_result: Option<FNLineType> = _check_if_title_page_element(line, &previous_line);
    if let Some(line_type) = title_page_result {
        return line_type;
    }

    // --------- Transitions
    let transition_result: Option<FNLineType> = _check_if_transition(line, &previous_line_is_empty);
    if let Some(line_type) = transition_result {
        return line_type;
    }

    // Handle items which require an empty line before them.

    // --------- Heading
    let heading_result: Option<FNLineType> = _check_if_heading(line, &previous_line_is_empty);
    if let Some(line_type) = heading_result {
        return line_type;
    }

    // --------- Check for Dual Dialogue
    let dual_dialogue_result = _check_if_dual_dialogue(line, &previous_line);
    if let Some(line_type) = dual_dialogue_result {
        return line_type;
    }
    // --------- Character

    let character_result: Option<FNLineType> = _check_if_character(line, &previous_line);
    if let Some(line_type) = character_result {
        return line_type;
    }

    // --------- Dialogue or Parenthetical
    let dialogue_or_parenthetical_result: Option<FNLineType> =
        _check_if_dialogue_or_parenthetical(line, &previous_line);
    if let Some(line_type) = dialogue_or_parenthetical_result {
        return line_type;
    }

    // --------- Default
    FNLineType::Action
}

// ---------- Parsing sub-functions ----------
fn _check_if_transition(line: &FNLine, previous_line_is_empty: &bool) -> Option<FNLineType> {
    if line.string.len() > 2
        && line.string.graphemes(true).last() == Some(":")
        && line.string == line.string.to_uppercase()
        && *previous_line_is_empty
    {
        return Some(FNLineType::TransitionLine);
    }

    None
}
fn _check_if_dialogue_or_parenthetical(
    line: &FNLine,
    previous_line: &Result<&FNLine, &str>,
) -> Option<FNLineType> {
    if let Ok(pl) = previous_line {
        if pl.is_dialogue() && pl.string.len() > 0 {
            if line.string.graphemes(true).nth(0) == Some("(") {
                return Some(FNLineType::Parenthetical);
            }
            return Some(FNLineType::Dialogue);
        }
        if pl.fn_type == FNLineType::Parenthetical {
            return Some(FNLineType::Dialogue);
        }
    }

    None
}
fn _check_if_heading(line: &FNLine, previous_line_is_empty: &bool) -> Option<FNLineType> {
    if !(*previous_line_is_empty && line.string.len() >= 3) {
        return None;
    }
    let first_3_graphemes = line
        .string
        .graphemes(true)
        .take(3)
        .collect::<Vec<&str>>()
        .join("");

    match first_3_graphemes.to_lowercase().as_str() {
        "int" => {}
        "ext" => {}
        "est" => {}
        "i/e" => {}
        _ => return None,
    }

    // To avoid words like "international" from becoming headings, the extension HAS to end with either dot, space or slash
    let next_grapheme = line.string.graphemes(true).nth(4);
    match next_grapheme {
        Some(".") | Some(" ") | Some("/") => {
            return Some(FNLineType::Heading);
        }
        _ => None,
    }
}

fn _check_if_forced_element(line: &FNLine, previous_line_is_empty: &bool) -> Option<FNLineType> {
    let first_grapheme_option: Option<&str> = line.string.graphemes(true).nth(0);
    let last_grapheme_option: Option<&str> = line.string.graphemes(true).last();

    if first_grapheme_option == None || last_grapheme_option == None {
        return None;
    }

    let first_grapheme = first_grapheme_option.unwrap_or_default();
    let last_grapheme = last_grapheme_option.unwrap_or_default();

    // TODO: Handle escaped characters outside of the static parser
    // Check for escaped characters
    // if (firstChar == '\\'):
    //    first_unescaped = hf.find_first_unescaped_char_in_string(line.string)
    //    if first_unescaped == "":
    //        return LineType.action // every character in the line is escaped i guess lmao
    //    firstChar = first_unescaped

    // --------- Forced whitespace
    let contains_only_whitespace: bool = line.string.trim().is_empty();

    let contains_at_least_two_spaces: bool =
        first_grapheme == " " && last_grapheme == " " && line.string.len() > 1;

    if contains_only_whitespace && !contains_at_least_two_spaces {
        return Some(FNLineType::Empty);
    }

    // --------- Page Break
    if line.string == "===" {
        return Some(FNLineType::PageBreak);
    }

    // --------- FORCED Action or Shot
    if first_grapheme == "!" {
        // checks raw first char again, to enable escaping \\ a `!` char
        // Action or shot
        if line.string.len() > 1 {
            let second_grapheme_option = line.string.graphemes(true).nth(1);
            if second_grapheme_option == Some("!") {
                return Some(FNLineType::Shot);
            }
        }
        return Some(FNLineType::Action);
    }
    // --------- FORCED Heading / Slugline
    if first_grapheme == "." && !*previous_line_is_empty {
        // '.' forces a heading.
        // Because our American friends love to shoot their guns like we Finnish people love our booze,
        // screenwriters might start dialogue blocks with such "words" as '.44'
        if line.string.len() > 1 {
            let second_grapheme_option = line.string.graphemes(true).nth(1);

            if let Some(sg) = second_grapheme_option {
                if sg != "." {
                    return Some(FNLineType::Heading);
                }
            }
            return None;
        }

        return Some(FNLineType::Heading);
    }

    // Rest of the FORCED FNLine Types
    match first_grapheme {
        ">" => {
            if last_grapheme == "<" {
                return Some(FNLineType::Centered);
            }
            Some(FNLineType::TransitionLine)
        }
        "~" => Some(FNLineType::Lyrics),
        "=" => Some(FNLineType::Synopse),
        "#" => Some(FNLineType::Section),
        "@" => {
            if last_grapheme == "^" && *previous_line_is_empty {
                return Some(FNLineType::DualDialogueCharacter);
            }
            Some(FNLineType::Character)
        }
        "." => {
            if *previous_line_is_empty {
                return Some(FNLineType::Heading);
            }
            return None;
        }
        _ => None,
    }
}

fn _check_if_title_page_element(
    line: &FNLine,
    previous_line: &Result<&FNLine, &str>,
) -> Option<FNLineType> {
    if let Ok(pl) = previous_line {
        if !pl.is_title_page() {
            return None;
        }
    }

    let key: String = line.get_title_page_key();

    if key.len() > 0 && !key.is_empty() {
        match key.as_str() {
            "title" => return Some(FNLineType::TitlePageTitle),
            "author" => return Some(FNLineType::TitlePageAuthor),
            "authors" => return Some(FNLineType::TitlePageAuthor),
            "credit" => return Some(FNLineType::TitlePageCredit),
            "source" => return Some(FNLineType::TitlePageSource),
            "contact" => return Some(FNLineType::TitlePageContact),
            "contacts" => return Some(FNLineType::TitlePageContact),
            "contact info" => return Some(FNLineType::TitlePageContact),
            "draft date" => return Some(FNLineType::TitlePageDraftDate),
            "draft" => return Some(FNLineType::TitlePageDraftDate),
            _ => return Some(FNLineType::TitlePageUnknown),
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

fn _check_if_character(line: &FNLine, previous_line: &Result<&FNLine, &str>) -> Option<FNLineType> {
    use crate::helper_funcs::only_uppercase_until_parenthesis;
    if !(only_uppercase_until_parenthesis(&line.string) && line.string != "") {
        return None;
    }
    if line.string != line.string.trim() {
        if line.string.starts_with("  ") {
            return None;
        }
    }
    let last_char_opt = line.string.graphemes(true).last();

    if last_char_opt == Some("^") {
        return Some(FNLineType::DualDialogueCharacter);
    }
    // Check if this line is actually just an ALLCAPS action line
    if let Ok(pl) = previous_line {
        if pl.fn_type != FNLineType::Empty {
            return Some(FNLineType::Action);
        }
    }
    Some(FNLineType::Character)
}

fn _check_if_empty_line(line: &FNLine) -> Option<FNLineType> {
    if line.string.len() == 0 {
        Some(FNLineType::Empty)
    } else {
        None
    }
}
fn _check_if_dual_dialogue(
    line: &FNLine,
    previous_line: &Result<&FNLine, &str>,
) -> Option<FNLineType> {
    if let Ok(pl) = previous_line {
        if !pl.is_dual_dialogue() {
            return None;
        }

        if let Some(gp) = line.string.graphemes(true).nth(0) {
            if gp == "(" {
                return Some(FNLineType::DualDialogueParenthetical);
            }
            return Some(FNLineType::DualDialogue);
        }
        return None;
    }

    None
}
