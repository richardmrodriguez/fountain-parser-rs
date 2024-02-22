use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::fountain_consts::{FNRangedElementType, LineType};
/// This module is responsible for handling the superposition of "partial lines"
/// and "true" lines. A "partial" line range is any line that is interrupted by a
/// multiline invisible (such as `Boneyard` or `Note`), which would be only a single line
/// if the multiline invisible were not present.
use crate::fountain_line::FNLine;

pub fn get_partial_line_map_for_ranged_element_type(
    lines: &Vec<FNLine>,
    ranged_element_type: &FNRangedElementType,
) -> Option<Vec<FNLine>> {
    let ranged_indices: Option<HashMap<String, HashSet<usize>>> =
        get_indices_of_ranged_element(&lines, ranged_element_type);
    return None;
}

/// Returns a HashMap:
/// "Opens": HashSet(line_idx_1, line_idx_2, ...)
/// "Closes": HashSet(line_idx_1, line_idx_2, ...)
/// NOTE: Opens and Closes have to be MATCHED and VALIDATED:
/// - Some opens or closes may be dangling because they don't have a match
/// - Some opens or closes may not make a valid pair because there is an empty line between them
pub fn get_indices_of_ranged_element(
    lines: &Vec<FNLine>,
    ranged_element_type: &FNRangedElementType,
) -> Option<HashMap<String, HashSet<usize>>> {
    let mut indicies_map: HashMap<String, HashSet<usize>> = HashMap::new();
    let mut indicies_opens_set: HashSet<usize> = HashSet::new();
    let mut indicies_closes_set: HashSet<usize> = HashSet::new();

    let (opens_pattern, closes_pattern) = match ranged_element_type {
        FNRangedElementType::Boneyard { open, close }
        | FNRangedElementType::Note { open, close }
        | FNRangedElementType::Other { open, close } => (open, close),
    };

    for (idx, ln) in lines.iter().enumerate() {
        if ln.string.contains(opens_pattern) {
            indicies_opens_set.insert(idx);
        }
        if ln.string.contains(closes_pattern) {
            indicies_closes_set.insert(idx);
        }
    }
    indicies_map.insert("Opens".to_string(), indicies_opens_set);
    indicies_map.insert("Closes".to_string(), indicies_closes_set);

    return Some(indicies_map);
}

pub fn get_true_lines_from_partials(partials: &HashMap<String, HashSet<usize>>) {
    todo!()
    // Returns a HashMap<partial_index, true_line_string>
}
