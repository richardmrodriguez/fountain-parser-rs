// This Fountain Parsing library uses code derived from "Beat", by Lauri-Matti Parppei.
// "Beat" Github: https://github.com/lmparppei/Beat/

pub mod fountain_enums;
pub mod fountain_line;
pub mod helper_funcs;
pub mod location_and_length;
pub mod partial_line_resolver;
pub mod static_fountain_parser;
pub mod static_fountain_preparser;

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{
        fountain_enums::FNRangedElementType, fountain_line::FNLine, partial_line_resolver,
        static_fountain_parser,
    };

    #[test]
    #[ignore = "just prints line types from LineType enum"]
    fn print_all_line_types() {
        use super::fountain_enums::LineType;
        let line_types_vec: Vec<LineType> = LineType::vec_of_line_types();
        for t in line_types_vec {
            let t1 = t.clone() as i32;
            println!("Line type variant: {:?}, {:?}", t1, t);
        }
    }

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

    pub fn print_all_lines_with_line_type(lines: Vec<FNLine>) {
        for ln in &lines {
            println!("{:?}\t\t\t{}", ln.fn_type, ln.string);
        }
    }
}
