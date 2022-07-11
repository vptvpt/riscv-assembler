pub fn tokenize_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    for c in line.chars() {
        match c {
            ' ' | '\t' | ',' | '\n' => {
                if !token.is_empty() {
                    tokens.push(token);
                    token = String::new();
                }
            }
            '#' => {
                if !token.is_empty() {
                    tokens.push(token);
                    token = String::new();
                }
                break;
            }
            ':' => {
                if !token.is_empty() {
                    tokens.push(token);
                    token = String::new();
                }
                tokens.push(":".to_string());
            }
            '(' => {
                if !token.is_empty() {
                    tokens.push(token);
                    token = String::new();
                }
                tokens.push("(".to_string());
            }
            ')' => {
                if !token.is_empty() {
                    tokens.push(token);
                    token = String::new();
                }
                tokens.push(")".to_string());
            }
            _ => token.push(c),
        }
    }
    if !token.is_empty() {
        tokens.push(token);
    }
    tokens
}
