pub fn only_uppercase_until_parenthesis(text: &String) -> bool {
    let until_parenthesis: Option<&str> = text.split("(").next();
    match until_parenthesis {
        Some(text) => {
            if text == text.to_uppercase() && text.len() > 0 {
                return true;
            }
            return false;
        }
        None => return false,
    }
}
