// This Fountain Parsing library uses code derived from "Beat", by Lauri-Matti Parppei.
// "Beat" Github: https://github.com/lmparppei/Beat/

use std::fs;

pub mod fountain_consts;
pub mod fountain_line;
pub mod helper_funcs;
pub mod location_and_length;
pub mod static_fountain_parser;

use fountain_line::FNLine;

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{fountain_line::FNLine, static_fountain_parser};

    #[test]
    #[ignore = "reason"]
    fn print_all_line_types() {
        use super::fountain_consts::LineType;
        let line_types_vec: Vec<LineType> = LineType::vec_of_line_types();
        for t in line_types_vec {
            let t1 = t.clone() as i32;
            println!("Line type variant: {:?}, {:?}", t1, t);
        }
    }

    #[test]
    fn test_static_parser() {
        let file_path = "testfile.txt";
        let file_result: Result<String, std::io::Error> = fs::read_to_string(file_path);
        match file_result {
            Ok(text) => {
                let lines = static_fountain_parser::get_parsed_lines_from_raw_string(text);
                print_all_lines_with_line_type(lines);
            }
            Err(err) => {
                eprintln!("The test file or file path was not valid. {}", err)
            }
        }
    }

    pub fn print_all_lines_with_line_type(lines: Vec<FNLine>) {
        for ln in &lines {
            println!("{:?}\t\t\t{}", ln.fn_type, ln.string);
        }
    }
}
