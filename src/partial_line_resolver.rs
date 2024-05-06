/// This module is responsible for handling "partial lines" and "true" lines.
///
/// A "partial" line range is any line that is interrupted by a
/// multiline invisible (such as `Boneyard` or `Note`), which would be only a single line
/// if the multiline invisible were not present.
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use enum_iterator::last;
use unicode_segmentation::UnicodeSegmentation;
use uuid::{uuid, Uuid};

use crate::fountain_enums::{FNLineType, FNPartialLineType, FNRangedElementType};
use crate::fountain_line::FNLine;
use crate::fountain_partial_line_range::{FNPartialLineRange, FNPartialMultilineRange};

/// Given an FNRangedElementType, Returns an optional HashMap of indices and corresponding FNLine objects with updated PartialLineType added.
/// These updated FNLines are to be used to handle extracting the printable text (if any) so that it may be handled by the `static_fountain_parser`
///
/// This only gives a map for one element type, so this function must be called at least twice - once for Notes, and once for Boneyards.
pub fn get_partial_fnline_map_for_ranged_element_type(
    lines: &Vec<FNLine>,
    ranged_element_type: &FNRangedElementType,
) -> Option<HashMap<usize, FNLine>> {
    //TODO: Make this function receive the global and local indices as args rather than calculate them in here
    //TODO: Make this function receive the partial_types_for_global_indices map as an arg instead of calculating in here
    //TODO: What do we do with the output of this god damn fuction aaahhhhhhh
    let mut partials_opens_map: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut partials_closes_map: HashMap<usize, Vec<usize>> = HashMap::new();

    let element_specific_global_indices =
        get_global_indices_of_ranged_element(lines, ranged_element_type);

    let mut partials_types_for_global_indices_map: HashMap<usize, FNPartialLineType> =
        HashMap::new();

    for global_idx in element_specific_global_indices {
        if let Some(ln) = lines.get(global_idx) {
            let (cur_opens_local_indices, cur_closes_local_indices) =
                get_local_indices_of_ranged_element(ln, ranged_element_type);
            partials_opens_map.insert(global_idx.clone(), cur_opens_local_indices);
            partials_closes_map.insert(global_idx.clone(), cur_closes_local_indices);

            let opens_locals_opt = partials_opens_map.get(&global_idx);
            let closes_locals_opt = partials_closes_map.get(&global_idx);
            let partials_type_opt = get_local_partial_type_for_single_line(
                ln,
                ranged_element_type,
                opens_locals_opt,
                closes_locals_opt,
            );

            match partials_type_opt {
                Some(cur_type) => {
                    partials_types_for_global_indices_map.insert(global_idx, cur_type);
                }
                None => {}
            }
        }
    }

    let mut sorted_partials_map_keys: Vec<usize> = partials_types_for_global_indices_map
        .keys()
        .copied()
        .collect();
    sorted_partials_map_keys.sort();

    let mut fnline_map: HashMap<usize, FNLine> = HashMap::new();

    for (_, global_idx) in sorted_partials_map_keys.iter().enumerate() {
        if let Some(cur_type) = partials_types_for_global_indices_map.get(global_idx) {
            if let Some(ln) = lines.get(global_idx.clone()) {
                let mut new_line = ln.clone();
                match ranged_element_type {
                    FNRangedElementType::Boneyard { open, close } => {
                        new_line.boneyard_type = Some(cur_type.clone());
                    }
                    FNRangedElementType::Note { open, close } => {
                        new_line.note_type = Some(cur_type.clone());
                    }
                    FNRangedElementType::Other { open, close } => {
                        continue; // Change this part if newer ranged element types are added to fountain
                    }
                }
                fnline_map.insert(global_idx.clone(), new_line);
            }
        }
    }

    Some(fnline_map)
}

pub fn get_copy_of_fnline_with_new_partial_type(
    mut line: FNLine,
    partial_type_opt: &Option<FNPartialLineType>,
    ranged_element_type: &FNRangedElementType,
) -> Option<FNLine> {
    match partial_type_opt {
        Some(partial_type) => match ranged_element_type {
            FNRangedElementType::Boneyard { open: _, close: _ } => {
                line.boneyard_type = Some(partial_type.clone())
            }
            FNRangedElementType::Note { open: _, close: _ } => {
                line.note_type = Some(partial_type.clone())
            }
            FNRangedElementType::Other { open: _, close: _ } => return None,
        },
        None => return None,
    }
    Some(line)
}

fn get_global_indices_of_ranged_element(
    lines: &Vec<FNLine>,
    ranged_element_type: &FNRangedElementType,
) -> Vec<usize> {
    let (opens_pattern, closes_pattern) = ranged_element_type.get_open_and_close_patterns();
    let mut global_indices_vec: Vec<usize> = Vec::new();

    for (idx, ln) in lines.iter().enumerate() {
        if ln.raw_string.contains(&opens_pattern) || ln.raw_string.contains(&closes_pattern) {
            global_indices_vec.push(idx.clone());
        }
    }
    global_indices_vec
}

/// Returns a HashMap of Global and Local indices across a `Vector` of `FNLine` for "Opens" and "Closes" patterns for an `FNRangedElementType`.
///```
/// "Opens": Hashmap<global_index, local_index_set>>
/// "Closes": Hashmap<global_index, local_index_set>>
///```
/// NOTE: Opens and Closes have to be MATCHED and VALIDATED:
/// - Some opens or closes may be orphaned because they don't have a match
/// - Some opens or closes may not make a valid pair because there is an `empty line` between them
/// - An empty line in the context of Notes and Boneyards means a line with zero text OR if only whitespace, less than two spaces
pub fn get_global_and_local_indices_of_ranged_element(
    lines: &Vec<FNLine>,
    ranged_element_type: &FNRangedElementType,
) -> Option<HashMap<String, HashMap<usize, Vec<usize>>>> {
    let mut indices_opens_map: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut indices_closes_map: HashMap<usize, Vec<usize>> = HashMap::new();

    let (opens_pattern, closes_pattern) = ranged_element_type.get_open_and_close_patterns();
    //this for loop only puts the global indexes in with blank Vecs
    for (global_idx, ln) in lines.iter().enumerate() {
        if ln.string.contains(&opens_pattern) && !indices_opens_map.contains_key(&global_idx) {
            indices_opens_map.insert(global_idx, Vec::new());
        }
        if ln.string.contains(&closes_pattern) && !indices_closes_map.contains_key(&global_idx) {
            indices_closes_map.insert(global_idx, Vec::new());
        }
    }
    // this for loop actually populates the Vecs within the hashmap vals
    // TODO: this is very inefficent because it just iterates over all the lines again
    // only need to iterate over the lines that already matched open or closed
    for (global_idx, ln) in lines.iter().enumerate() {
        let (open_matches, close_matches) =
            get_local_indices_of_ranged_element(ln, ranged_element_type);
        for (local_idx, s) in open_matches.iter().enumerate() {
            if let Some(opens_locals_vec) = indices_opens_map.get_mut(&global_idx) {
                opens_locals_vec.push(local_idx);
            }
        }
        for (local_idx, s) in close_matches.iter().enumerate() {
            if let Some(closes_locals_vec) = indices_closes_map.get_mut(&global_idx) {
                closes_locals_vec.push(local_idx);
            }
        }
    }

    let mut indicies_map: HashMap<String, HashMap<usize, Vec<usize>>> = HashMap::new();
    indicies_map.insert("Opens".to_string(), indices_opens_map);
    indicies_map.insert("Closes".to_string(), indices_closes_map);

    Some(indicies_map)
}

//TODO:
// getting ranges of partial lines is not actually defined behavior in Fountain syntax.
// There are two possible strategies I see:
// 1. Easy mode -- only pair ORPHANED OPENS to ORPHANED CLOSES, and ONLY IF there are ZERO standalone
// partials between them.
// 2. - Tedious mode -- pair orphaned opens to the LAST VALID close. This means capturing any line in between as an
// InvisibleOnly.

/// Returns a Vector of FNPartialMultilineRange objects. These objects are used to handle
/// the differences between the "raw" document and the visible lines at a high level.
/// Each FNPartialMultilineRange object has global and local indices for the start and end of multiline invisibles.
/// This implimentation ensures there are ZERO SelfContained or InvisibleOnly lines between an Orphaned Open and an Orphaned Close.
/// After receiving these ranges, you must iterate through the lines between and mark each line as InvisibleOnly.
/// In other words -- FNPartialMultilineRange objects can ONLY exist if there aren't any other opens or closes between the two.
/// Otherwise, it isn't a valid FNPartialMultilineRange.
/// This is done for simplicity and because I will throw my brain into a trash compactor if I don't.
pub fn get_partial_multiline_ranges_from_partial_map(
    partials_map: &HashMap<usize, FNLine>,
    lines: &Vec<FNLine>,
    ranged_element_type: &FNRangedElementType,
) -> Vec<FNPartialMultilineRange> {
    let mut sorted_partials_keys: Vec<usize> = partials_map.keys().copied().collect();
    sorted_partials_keys.sort();

    let mut last_unresolved_open_idx: Option<usize> = None;
    let mut last_unresolved_open_local_idx: Option<usize> = None;

    let mut partial_line_ranges_vec: Vec<FNPartialMultilineRange> = Vec::new();

    let (_, closes_pat) = ranged_element_type.get_open_and_close_patterns();

    for global_idx in sorted_partials_keys.iter() {
        if let Some(ln) = partials_map.get(global_idx) {
            let partial_type = match ranged_element_type {
                FNRangedElementType::Boneyard { open, close } => &ln.boneyard_type,
                FNRangedElementType::Note { open, close } => &ln.note_type,
                FNRangedElementType::Other { open, close } => &None,
            };
            if let Some(_last_unresolved_open) = last_unresolved_open_idx {
                match partial_type {
                    Some(FNPartialLineType::OrphanedClose)
                    | Some(FNPartialLineType::OrphanedOpenAndClose) => {
                        let new_multiline_partial_range = FNPartialMultilineRange {
                            id: None,
                            global_start: last_unresolved_open_idx.clone(),
                            local_start: last_unresolved_open_local_idx.clone(),
                            global_end: Some(global_idx.clone()),
                            local_end: get_first_match_in_string(
                                closes_pat.clone(),
                                ln.raw_string.clone(),
                            ),
                        };
                        partial_line_ranges_vec.push(new_multiline_partial_range);
                        match partial_type {
                            Some(FNPartialLineType::OrphanedClose) => {
                                last_unresolved_open_idx = None;
                                last_unresolved_open_local_idx = None;
                            }
                            Some(FNPartialLineType::OrphanedOpenAndClose) => {
                                last_unresolved_open_idx = Some(global_idx.clone());
                                let (open_locals, _) =
                                    get_local_indices_of_ranged_element(ln, ranged_element_type);
                                last_unresolved_open_local_idx =
                                    Some(open_locals.last().unwrap().clone())
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        continue;
                    }
                }
            }
            // there isn't currently an unresolved open
            match partial_type {
                Some(FNPartialLineType::OrphanedOpen)
                | Some(FNPartialLineType::OrphanedOpenAndClose) => {
                    last_unresolved_open_idx = Some(global_idx.clone());
                    //TODO store the orphaned open/close indices in the FNLine instead of recalculating them smh
                    let (open_locals, _) =
                        get_local_indices_of_ranged_element(ln, ranged_element_type);
                    last_unresolved_open_local_idx = Some(open_locals.last().unwrap().clone())
                }
                _ => {}
            }
        }
    }

    partial_line_ranges_vec
}

fn get_first_match_in_string(opens_pattern: String, line_string: String) -> Option<(usize)> {
    let mut indices = line_string.match_indices(&opens_pattern);
    if let Some((idx, _)) = indices.next() {
        return Some(idx);
    }
    None
}

fn get_last_valid_close_in_string(closes_pattern: String, line_string: String) -> Option<(usize)> {
    let indices = line_string.match_indices(&closes_pattern);
    if let Some((idx, _)) = indices.last() {
        return Some(idx);
    }
    None
}

pub fn get_local_indices_of_ranged_element(
    line: &FNLine,
    ranged_element_type: &FNRangedElementType,
) -> (Vec<usize>, Vec<usize>) {
    let (opens_pattern, closes_pattern) = ranged_element_type.get_open_and_close_patterns();
    let open_matches = line.raw_string.match_indices(&opens_pattern);
    let close_matches = line.raw_string.match_indices(&closes_pattern);

    let opens_local_vec: Vec<usize> = open_matches.map(|(index, _)| index).collect();
    let closes_local_vec: Vec<usize> = close_matches.map(|(index, _)| index).collect();
    (opens_local_vec, closes_local_vec)
}

/// Returns an optional `PartialLineType` for a given
/// `FNRangedElementType` and `FNLine`.
///
/// This function uses the `Opens` and `Closes` strings from the `FNRangedElementType` and
/// checks for the presence and/or pattern of valid pairs. If an open or close is unpaired, it is considered an orphan.
/// If it is a partial line, this returns a `Some(PartialLineType)`:
///
/// - `SelfContained` - A single line which contains both "invisible" text like `Note` or `Boneyard`, as well as printable text.
/// - `OrphanedOpens` - There is at least 1
/// - `OrphanedCloses`
/// - `OrphanedOpensAndCloses`
///
/// If there are no opens or closes, or if there is no non-invisble text, this returns `None`.
///
pub fn get_local_partial_type_for_single_line(
    line: &FNLine,
    ranged_element_type: &FNRangedElementType,
    opens_locals_opt: Option<&Vec<usize>>,
    closes_locals_opt: Option<&Vec<usize>>,
) -> Option<FNPartialLineType> {
    let (opens_pattern, closes_pattern) = ranged_element_type.get_open_and_close_patterns();

    let contains_opens: bool = line.raw_string.contains(&opens_pattern);
    let contains_closes: bool = line.raw_string.contains(&closes_pattern);

    if !contains_opens && !contains_closes {
        return None;
    }
    if contains_opens && !contains_closes {
        return Some(FNPartialLineType::OrphanedOpen);
    }
    if !contains_opens && contains_closes {
        return Some(FNPartialLineType::OrphanedClose);
    }
    // If program gets here, the string must contain both opens and closes

    let opens_local_indices: &Vec<usize> = opens_locals_opt.unwrap();
    let closes_local_indices: &Vec<usize> = closes_locals_opt.unwrap();

    // Handling DANGLING / ORPHANED opens or closes
    let has_orphaned_opens = opens_local_indices.last() > closes_local_indices.last();
    let has_orphaned_closes = closes_local_indices.first() < opens_local_indices.first();

    if has_orphaned_closes && has_orphaned_opens {
        return Some(FNPartialLineType::OrphanedOpenAndClose);
    }
    if has_orphaned_opens {
        return Some(FNPartialLineType::OrphanedOpen);
    }
    if has_orphaned_closes {
        return Some(FNPartialLineType::OrphanedClose);
    }

    // No more stray or dangling opens or closes after this point
    // The line MUST look like one of these at this point:
    //
    // [[ ... ]] or /* ...  */
    //
    // now we have to look at the actual string to determine if there's actually any text,
    // or if this whole line is just a standalone note / boneyard
    if !line.raw_string.starts_with(&opens_pattern) {
        //there might be text at the beginning

        return Some(FNPartialLineType::SelfContained);
    }
    if !line.raw_string.ends_with(&closes_pattern) {
        //there might be text at the end
        return Some(FNPartialLineType::SelfContained);
    }

    // find if there is text in the middle
    // look for case where an open is after a close: ]][[
    // the distance between the open and close should be >= 1: ]] [[
    for (_opn_meta_index, open_local_idx) in opens_local_indices.iter().enumerate() {
        for (_cls_meta_index, cls_local_idx) in closes_local_indices.iter().enumerate() {
            if open_local_idx > cls_local_idx {
                // This is the only close local index before the current open local index
                if open_local_idx - cls_local_idx > 0 {
                    return Some(FNPartialLineType::SelfContained);
                }
            }
        }
    }

    Some(FNPartialLineType::InvisibleOnly)
}

/// Returns an String with the given FNRangedElementType text removed
/// recursive function; calls itself until there are no more opens or closes patterns
pub fn delete_ranged_text_with_recursion(string: String) -> String {
    todo!()
    //      get first open in current string:
    //      if there are ZERO opens patterns in this string, then return the String
    //      if there are ZERO closes patterns in this string, then return the String
    //      else: iterate through closes until:
    //          if next close exists && next open exists:
    //              if next close index > next open index:
    //                  delete the text from current string, starting from the first until this exact close
    //                  pass new string to new iteration of this function (do a recursion)
    //                      return the output from the above call
    //          if next close exists && !next open exists:
    //                    continue;
    //          else:
    //              delete text from current string from open until this exact close
    //              pass new string to new iteration of this function (do a recursion)
    //                      return the output from the above call
}

// There are two types of ranged elements to handle:
// Single-line
// Multiline

//Handling single-line (SelfContained) ranged elements is easy.
//Just remove the ranged invisible text with the delete_ranged_text_with_recursion function.
// But what if this is actually the middle of a multi-line note?

/// Only creates FNPartialLineRange objects, NOT FNPartialMultilineRange objects.
/// These should be handled AFTER creating the FNPartialMultilineRange objects.
/// Example: If a SelfContained line is between the opening and closing line of a
/// FNPartialMultilineRange, then that SelfContained should become an InvisibleOnly line, even if it would
/// otherwise have valid printable text.
pub fn create_single_line_partial_line_ranges() {
    todo!()
}
