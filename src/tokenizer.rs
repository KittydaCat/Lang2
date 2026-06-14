#[derive(PartialEq, Debug)]
pub enum Token {
    Name(String),

    NumberLiteral(f64),

    Type,
    Let,
    Match,

    Bar,
    SemiColon,
    Colon,
    Equals,
    Minus,
    RAngle,
    Comma,

    LParens,
    RParens,
    LBrace,
    RBrace,
}

pub fn tokenize(file: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut cars = file.chars().peekable();

    let mut string = String::new();

    loop {
        let next = cars.next();

        if let Some(c @ ('0'..='9' | 'a'..='z' | 'A'..='Z' | '_')) = next {
            string.push(c);

            continue;
        } else if !string.is_empty() {
            if matches!(string.chars().next().unwrap(), '-' | '0'..='9') {
                // todo use string parse do determine if it is a var?
                tokens.push(Token::NumberLiteral(string.parse().unwrap()));

                string = String::new();
            } else {
                tokens.push(match string.as_str() {
                    "Type" => Token::Type,
                    "Let" => Token::Let,
                    "Match" => Token::Match,
                    _ => Token::Name(string),
                });

                string = String::new();
            }
        }

        if next.is_none() {
            break;
        }

        match next.unwrap() {
            ' ' | '\t' | '\n' => {}

            ';' => tokens.push(Token::SemiColon),
            ':' => tokens.push(Token::Colon),
            '|' => tokens.push(Token::Bar),
            '=' => tokens.push(Token::Equals),
            '-' => tokens.push(Token::Minus),
            '>' => tokens.push(Token::RAngle),
            ',' => tokens.push(Token::Comma),

            '{' => tokens.push(Token::LBrace),
            '}' => tokens.push(Token::RBrace),
            '(' => tokens.push(Token::LParens),
            ')' => tokens.push(Token::RParens),
            x => todo!("({}, {})", x, x as u32),
        }
    }

    tokens
}
