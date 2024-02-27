/// This module is responsible for handling "partial lines" and "true" lines.
///
/// A "partial" line range is any line that is interrupted by a
/// multiline invisible (such as `Boneyard` or `Note`), which would be only a single line
/// if the multiline invisible were not present.
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use unicode_segmentation::UnicodeSegmentation;

use crate::fountain_enums::{FNLineType, FNPartialLineType, FNRangedElementType};
use crate::fountain_line::FNLine;

pub fn get_partial_line_vec_for_ranged_element_type(
    lines: &Vec<FNLine>,
    ranged_element_type: &FNRangedElementType,
) -> Option<Vec<usize>> {
    let ranged_indices_map_opt: Option<HashMap<String, HashMap<usize, Vec<usize>>>> =
        get_global_and_local_indices_of_ranged_element(&lines, ranged_element_type);

    if ranged_indices_map_opt == None {
        return None;
    }

    let (sorted_opens_global_indices, sorted_closes_global_indices) =
        match get_sorted_open_close_global_indices(&ranged_indices_map_opt) {
            Ok(value) => value,
            Err(_) => return None,
        };

    // should be a vec of only unique indexes
    let sorted_opens_and_closes_global_indices: Vec<usize> = sorted_opens_global_indices
        .into_iter()
        .chain(sorted_closes_global_indices.into_iter())
        .collect::<HashSet<usize>>()
        .into_iter()
        .collect();

    let ranged_indices_map: HashMap<String, HashMap<usize, Vec<usize>>> =
        match ranged_indices_map_opt {
            Some(indices) => indices,
            None => return None,
        };

    let opens_global_and_local_indices: HashMap<usize, Vec<usize>> =
        ranged_indices_map.get("Opens").unwrap().clone();
    let closes_global_and_local_indices: HashMap<usize, Vec<usize>> =
        ranged_indices_map.get("Closes").unwrap().clone();

    let mut prev_opn_global_idx: Option<usize> = None;
    let mut prev_close_global_idx: Option<usize> = None;

    let mut partial_lines_vec: Vec<(
        HashMap<usize, usize>, // Open Global: Open Local (first valid open)
        HashMap<usize, usize>, // Close Global: Close Local (Last valid close)
        FNPartialLineType,
    )> = Vec::new();

    for (_, global_index) in sorted_opens_and_closes_global_indices.iter().enumerate() {
        if let Some(ln) = lines.get(global_index.clone()) {
            let cur_opens_locals_opt = opens_global_and_local_indices.get(&global_index);

            let cur_closes_locals_opt = closes_global_and_local_indices.get(&global_index);

            let cur_closes_locals = cur_closes_locals_opt.unwrap();
            let cur_opens_locals = cur_opens_locals_opt.unwrap();
            let cur_first_valid_open_local = cur_opens_locals.first().unwrap();
            let cur_last_valid_close_local = cur_closes_locals.last().unwrap();

            let mut cur_open_hashmap: Option<Option<&Vec<usize>>> =
                HashMap::new().insert(Some(global_index.clone()), cur_opens_locals_opt);
            let mut cur_close_hashmap: Option<Option<&Vec<usize>>> =
                HashMap::new().insert(Some(global_index.clone()), cur_closes_locals_opt);

            // if cur close or open locals.len() < 1 or somehow doesnt exist, it will be pushed as None, otherwise Some

            match get_local_partial_type_for_single_line(
                ln,
                ranged_element_type,
                Some(cur_opens_locals),
                Some(cur_closes_locals),
            ) {
                Some(FNPartialLineType::SelfContained) => {
                    todo!()
                }
                Some(FNPartialLineType::OrphanedClose) => {
                    todo!()
                }
                Some(FNPartialLineType::OrphanedOpen) => {
                    todo!()
                }
                Some(FNPartialLineType::OrphanedOpenAndClose) => {
                    todo!()
                }
                None => {}
            }
        }
    }
    // Check each line at each index
    // Is it a partial?

    // A partial OPEN means that the OPEN Pattern is NOT the first part of the string
    // A partial CLOSE means that the CLOSE pattern is NOT the last part of the string
    // Is there the same amount of opens and closes?
    //      If so, sort the opens and closes line indexes. Always match the first open to the first close.
    //      If the open and close are part of the same line AND there is no prefixing or trailing text, it's NOT a partial line.
    // What about a line that is sandwiched between two notes?
    //      The open/close matching enforces the idea that the 1st open should be paired with the 1st close.
    //      So, when checking the open and closes in pairs, it will catch that the first open-close pair is NOT the entirety
    //      of the line.
    //      If an open-close range appears to be overlapping -- like [[some [[text]] here]]
    //      Promote the next close -- when iterating , if it's a CLOSE, just overwrite the last pair's CLOSE index.
    // A document with lots of CLOSES and one OPEN will have a Note that spans from the one OPEN to the last CLOSE.
    // A document with multiple OPENS and one CLOSE will behave the same -- the first open will go to the last close.
    // This happens in the order of the document -- if there is more CLOSES than opens, but the first n notes are not "overlapping"
    // Those first n notes will not be affected by any unexpected behavior.
    // DANGLING OPEN: if there is an OPEN at the end of a document with no close after it, it is dangling. Dangling notes can still be partially valid...
    // DANGLING CLOSE: With this system, there technically isn't a "Dangling close", because the last note in a document will simply
    // extend to the last close of the document.
    // If an OPEN is within the previous range of notes (between the previous open and close pair),
    // We then check if the close is also within the previous range. If both are within the previous range, the pair is ignored.
    // If the new close is farther in the document than the old close, then the old close index is promoted to the new close index.
    return None;
}

fn get_sorted_open_close_global_indices(
    ranged_indices: &Option<HashMap<String, HashMap<usize, Vec<usize>>>>,
) -> Result<(Vec<usize>, Vec<usize>), Option<Vec<usize>>> {
    match ranged_indices {
        Some(indices) => {
            let opens_opt: Option<&HashMap<usize, Vec<usize>>> = indices.get("Opens");
            let closes_opt: Option<&HashMap<usize, Vec<usize>>> = indices.get("Closes");

            let opens_indices_map: HashMap<usize, Vec<usize>> = match opens_opt {
                Some(opn_map) => opn_map.clone(),
                None => return Err(None),
            };
            let closes_indices_map: HashMap<usize, Vec<usize>> = match closes_opt {
                Some(cls_map) => cls_map.clone(),
                None => return Err(None),
            };
            let mut sorted_opens_vec: Vec<usize> = opens_indices_map.keys().cloned().collect();
            let mut sorted_closes_vec: Vec<usize> = closes_indices_map.keys().cloned().collect();
            sorted_opens_vec.sort();
            sorted_closes_vec.sort();

            Ok((sorted_opens_vec, sorted_closes_vec))
        }
        None => Err(None),
    }
}

fn get_sorted_indices(indices: usize) -> Vec<usize> {
    todo!()
}
/// Returns a HashMap of Global and Local indices across a `Vector` of `FNLine`
/// for "Opens" and "Closes" patterns from a given`FNRangedElementType`.
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
        if ln.string.contains(&opens_pattern) {
            let open_matches = ln.string.match_indices(&opens_pattern);
            for (local_idx, s) in open_matches {
                if let Some(opens_locals_vec) = indices_opens_map.get_mut(&global_idx) {
                    opens_locals_vec.push(local_idx);
                }
            }
        }

        if ln.string.contains(&closes_pattern) {
            let close_matches = ln.string.match_indices(&closes_pattern);
            for (local_idx, s) in close_matches {
                if let Some(closes_locals_vec) = indices_closes_map.get_mut(&global_idx) {
                    closes_locals_vec.push(local_idx);
                }
            }
        }
    }

    let mut indicies_map: HashMap<String, HashMap<usize, Vec<usize>>> = HashMap::new();
    indicies_map.insert("Opens".to_string(), indices_opens_map);
    indicies_map.insert("Closes".to_string(), indices_closes_map);

    Some(indicies_map)
}

pub fn get_true_lines_from_partials(partials: &HashMap<String, HashSet<usize>>) {
    todo!()
    // Returns a HashMap<partial_index, true_line_string>
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

    None
}

pub fn get_departialed_line_from_partial_range() -> FNLine {
    todo!()
}
