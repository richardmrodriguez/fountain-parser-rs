// This Fountain Parsing library uses code derived from "Beat", by Lauri-Matti Parppei.
// "Beat" Github: https://github.com/lmparppei/Beat/

//! This is a Fountain syntax parser built in Rust. Most if its code is translated and modified from Obj-C,
//! from "Beat" by Lauri-Matti Parppei.
//!
//! This parser in in alpha, and currently only supports "visible" screenplay elements.
//! This means that "invisible" elements like `Notes` and `Boneyards` are not yet supported.
//!
//! If you use this parser with a Fountain-formatted plaintext document,
//! and it doesn't have `Notes` or `Boneyards`, you can expect to be able to
//! parse each line in the document properly.
//!
//! The module you want to use and pay attention to is `static_fountain_parser`.

// use fountain_enums::FNRangedElementType;

pub mod fountain_enums;
pub mod fountain_line;
pub mod location_and_length;
pub mod static_fountain_parser;

mod fountain_partial_line_range;
mod helper_funcs;
mod partial_line_resolver;
mod static_fountain_preparser;

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs};
    use unicode_segmentation::*;

    use crate::{
        fountain_enums::FNRangedElementType, fountain_line::FNLine, partial_line_resolver,
        static_fountain_parser,
    };

    #[test]
    fn test_static_parser() {
        let file_path = "fountain_test_files/general_without_ranged_elements.txt";
        let file_result: Result<String, std::io::Error> = fs::read_to_string(file_path);
        match file_result {
            Ok(text) => {
                let lines = static_fountain_parser::get_parsed_lines_from_raw_string(text);
                print_all_lines_with_line_type(lines);
            }
            Err(_err) => {
                panic!("File or file path was invalid.")
            }
        }
    }

    #[test]
    fn test_ranged_indices() {
        let file_path = "fountain_test_files/ranged_items_partial_line_test";
        let file_result: Result<String, std::io::Error> = fs::read_to_string(file_path);
        if let Ok(document_string) = file_result {
            let unparsed_lines: Vec<FNLine> =
                static_fountain_parser::get_unparsed_line_array_from_raw_string(Some(
                    document_string,
                ));

            let indices: Option<
                std::collections::HashMap<String, std::collections::HashMap<usize, Vec<usize>>>,
            > = partial_line_resolver::get_global_and_local_indices_of_ranged_element(
                &unparsed_lines,
                &FNRangedElementType::note(),
            );

            println!("Notes Indices: {:?}", indices.unwrap());
        }
    }

    #[test]
    pub fn test_is_line_partial() {
        let file_path = "fountain_test_files/ranged_items_partial_line_test";
        let file_result: Result<String, std::io::Error> = fs::read_to_string(file_path);
        if let Ok(document_string) = file_result {
            let test_lines = static_fountain_parser::get_unparsed_line_array_from_raw_string(Some(
                document_string,
            ));

            let indices: Option<
                std::collections::HashMap<String, std::collections::HashMap<usize, Vec<usize>>>,
            > = partial_line_resolver::get_global_and_local_indices_of_ranged_element(
                &test_lines,
                &FNRangedElementType::note(),
            );
            let (opens, closes) = match indices {
                Some(map) => (
                    map.get("Opens").unwrap().clone(),
                    map.get("Closes").unwrap().clone(),
                ),
                None => panic!(),
            };
            for (n, ln) in test_lines.iter().enumerate() {
                let opens_locals_opt = opens.get(&n);
                let closes_locals_opt = closes.get(&n);

                let partial_test_result =
                    partial_line_resolver::get_local_partial_type_for_single_line(
                        ln,
                        &FNRangedElementType::note(),
                        opens_locals_opt,
                        closes_locals_opt,
                    );
                match partial_test_result {
                    Some(partial_result) => {
                        println!(
                            "Line # {}\tPartial Type: {:?}\tRaw String: {}",
                            n, partial_result, ln.raw_string
                        )
                    }
                    None => {
                        println!(
                            "Line # {}\tPartial Type: None\tRaw String: {}",
                            n, ln.raw_string
                        )
                    }
                }
            }
        }
    }

    #[test]
    pub fn test_copy_of_fnline_with_new_partial_type() {
        let ranged_element_type = &FNRangedElementType::note();

        if let Ok(document_string) = ranged_items_test_file_path_result() {
            let unparsed_lines = static_fountain_parser::get_unparsed_line_array_from_raw_string(
                Some(document_string),
            );

            let mut new_lines_map: HashMap<usize, FNLine> = HashMap::new();

            for (idx, ln) in unparsed_lines.iter().enumerate() {
                let (local_opens, local_closes) =
                    partial_line_resolver::get_local_indices_of_ranged_element(
                        ln,
                        ranged_element_type,
                    );
                let partial_fnline_result =
                    partial_line_resolver::get_local_partial_type_for_single_line(
                        ln,
                        &ranged_element_type,
                        Some(&local_opens),
                        Some(&local_closes),
                    );
                match partial_line_resolver::get_copy_of_fnline_with_new_partial_type(
                    ln.clone(),
                    &partial_fnline_result,
                    ranged_element_type,
                ) {
                    Some(new_line) => {
                        new_lines_map.insert(idx, new_line);
                    }
                    None => {
                        println!("Line #{} is not partial.", idx);
                    }
                }
            }

            let mut sorted_keys: Vec<usize> = new_lines_map.keys().copied().collect();
            sorted_keys.sort();

            for global_idx in sorted_keys {
                let line = new_lines_map.get(&global_idx).unwrap();
                println!(
                    "Line #{}\tPartial type from new line:{:?}\tRaw String from new line:{}",
                    global_idx, line.note_type, line.raw_string
                )
            }
        }
    }

    fn ranged_items_test_file_path_result() -> Result<String, std::io::Error> {
        let file_path = "fountain_test_files/ranged_items_partial_line_test.txt";
        fs::read_to_string(file_path)
    }

    #[test]
    pub fn test_get_partial_fnline_map() {
        let file_result: Result<String, std::io::Error> = ranged_items_test_file_path_result();
        if let Ok(document_string) = file_result {
            let unparsed_lines: Vec<FNLine> =
                static_fountain_parser::get_unparsed_line_array_from_raw_string(Some(
                    document_string,
                ));
            let partial_fnline_result =
                partial_line_resolver::get_partial_fnline_map_for_ranged_element_type(
                    &unparsed_lines,
                    &FNRangedElementType::note(),
                );
            match partial_fnline_result {
                Some(fnlinemap) => {
                    for (idx, fnline) in fnlinemap.iter() {
                        println!(
                            "Line Index #: {} \tNote type: {:?}\tRaw String:{}",
                            idx, fnline.note_type, fnline.raw_string
                        );
                    }
                }
                None => {
                    println!("The partial fnline result was None instead of Some")
                }
            }
        }
    }

    pub fn print_all_lines_with_line_type(lines: Vec<FNLine>) {
        for ln in &lines {
            println!("{:?}\t\t\t{}", ln.fn_type, ln.string);
        }
    }
    #[test]
    pub fn get_partial_multiline_ranges() {
        let file_result: Result<String, std::io::Error> = ranged_items_test_file_path_result();
        let ranged_element_type = FNRangedElementType::note();
        if let Ok(document_string) = file_result {
            println!("I am here");
            let unparsed_lines: Vec<FNLine> =
                static_fountain_parser::get_unparsed_line_array_from_raw_string(Some(
                    document_string,
                ));
            let partial_map_opt =
                partial_line_resolver::get_partial_fnline_map_for_ranged_element_type(
                    &unparsed_lines,
                    &ranged_element_type,
                );
            if let Some(partial_map) = partial_map_opt {
                let multiline_ranges =
                    partial_line_resolver::get_partial_multiline_ranges_from_partial_map(
                        &partial_map,
                        &unparsed_lines,
                        &ranged_element_type,
                    );
                for range in multiline_ranges {
                    let start = range.global_start.unwrap();
                    let end = range.global_end.unwrap();
                    println!("---------------------------");
                    for i in (start..=end) {
                        if let Some(ln) = unparsed_lines.get(i) {
                            let gphs: String = ln.raw_string.graphemes(true).take(50).collect();
                            println!("Line #{},\t Raw String:{}", i, ln.raw_string);
                        }
                    }
                    println!("Range: {:#?}", range);

                    println!("---------------------------");
                }
            }
        }
    }

    #[test]
    pub fn get_stripped_string_from_parsed_and_partial_lines() {
        let file_result: Result<String, std::io::Error> = ranged_items_test_file_path_result();
        let ranged_element_type = FNRangedElementType::note();
        if let Ok(document_string) = file_result {
            let unparsed_lines: Vec<FNLine> =
                static_fountain_parser::get_unparsed_line_array_from_raw_string(Some(
                    document_string,
                ));
            let partial_map_opt =
                partial_line_resolver::get_partial_fnline_map_for_ranged_element_type(
                    &unparsed_lines,
                    &ranged_element_type,
                );
            if let Some(partial_map) = partial_map_opt {
                let multiline_ranges =
                    partial_line_resolver::get_partial_multiline_ranges_from_partial_map(
                        &partial_map,
                        &unparsed_lines,
                        &ranged_element_type,
                    );
                // iterate over all unparsed_lines
                // If the current global index is the beginning of a multiline_partial_range, check if it is partial
                // If it's partial, get the whole visible text from the range and add and push it to a StrippedLines struct
                //
                // If it's InvisbleOnly, continue
                // If the current index is a part of a vec of single_line_partials, push a new line with only the visible text
                // If the current index is neither a multiline partial or single-line partial, just push it into stripped lines struct
                //
            }
        }
    }
}
