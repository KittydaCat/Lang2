use std::iter::Peekable;
use std::vec::IntoIter;

use crate::tokenizer::Token;

// #[derive(Debug)]
// pub enum Definition {
//     TypeDefinition(RawTypeDefinition),
//     ConstantDefinition(String, Value),
// }

#[derive(Debug)]
pub struct RawTypeDefinition {
    pub name: String,
    pub variants: Vec<(String, RawType)>,
}

// #[derive(Clone, Debug)]
// pub struct Value {
//     val_type: Type,
//     val_source: ValueSource,
// }

#[derive(Clone, Debug)]
pub enum Value {
    FunctionDefinition(String, RawType, Box<Value>),
    FunctionCall(Box<Value>, Box<Value>),
    Match(Box<Value>, Vec<(String, Box<Value>)>),

    Constant(String),

    Number(f64),
    Unit,
}

// todo remove partial eq
#[derive(Clone, Debug)]
pub enum RawType {
    Unit,

    Function(Box<RawType>, Box<RawType>),

    TypeDef(String),
}

pub fn parse_definitions(
    tokens: &mut Peekable<IntoIter<Token>>,
) -> (Vec<RawTypeDefinition>, Vec<(String, Value, RawType)>) {
    let mut type_definitions = Vec::new();
    let mut const_def = Vec::new();

    loop {
        match tokens.next() {
            Some(Token::Type) => type_definitions.push(parse_type_definition(tokens)),
            Some(Token::Let) => {
                let Token::Name(var_name) = tokens.next().unwrap() else {
                    unimplemented!()
                };

                assert_eq!(tokens.next().unwrap(), Token::Colon);

                let typ = parse_type(tokens);

                assert_eq!(tokens.next().unwrap(), Token::Equals);

                let val = parse_value(tokens);

                const_def.push((var_name, val, typ));
            }

            Some(x) => unimplemented!("{x:?}"),

            None => break,
        }
        assert_eq!(tokens.next().unwrap(), Token::SemiColon);
    }

    (type_definitions, const_def)
}

fn parse_type_definition(tokens: &mut Peekable<IntoIter<Token>>) -> RawTypeDefinition {
    let Token::Name(name) = tokens.next().unwrap() else {
        unimplemented!()
    };

    assert_eq!(tokens.next().unwrap(), Token::Equals);

    let mut variants = Vec::new();

    while *tokens.peek().unwrap() != Token::SemiColon {
        let Token::Name(var_name) = tokens.next().unwrap() else {
            unimplemented!()
        };

        let var_type = parse_type(tokens);

        variants.push((var_name, var_type));
        match tokens.peek().unwrap() {
            Token::SemiColon => break,
            Token::Bar => {
                assert_eq!(tokens.next().unwrap(), Token::Bar);
            }

            _ => unimplemented!(),
        }
    }

    // assert!(state.type_def.insert(name.clone()));

    RawTypeDefinition { name, variants }
}

fn parse_value(tokens: &mut Peekable<IntoIter<Token>>) -> Value {
    let mut value = match tokens.next().unwrap() {
        Token::NumberLiteral(x) => Value::Number(x),

        Token::LParens => {
            if *tokens.peek().unwrap() == Token::RParens {
                assert_eq!(tokens.next().unwrap(), Token::RParens);
                Value::Unit
            } else {
                let val = parse_value(tokens);

                assert_eq!(tokens.next().unwrap(), Token::RParens);

                val
            }
        }

        Token::Bar => {
            let Token::Name(name) = tokens.next().unwrap() else {
                unimplemented!()
            };

            assert_eq!(tokens.next().unwrap(), Token::Colon);

            let t = parse_type(tokens);

            assert_eq!(tokens.next().unwrap(), Token::Bar);

            let ret_type = parse_value(tokens);

            Value::FunctionDefinition(name, t, Box::new(ret_type))
        }

        Token::Match => {
            let val = parse_value(tokens);

            assert_eq!(Token::LBrace, tokens.next().unwrap());

            let mut arms = Vec::new();

            while *tokens.peek().unwrap() != Token::RBrace {
                let Token::Name(name) = tokens.next().unwrap() else {
                    unimplemented!()
                };

                let val = parse_value(tokens);

                arms.push((name, Box::new(val)));

                assert_eq!(Token::Comma, tokens.next().unwrap());
            }

            assert_eq!(Token::RBrace, tokens.next().unwrap());

            Value::Match(Box::new(val), arms)
        }

        Token::Name(str) => Value::Constant(str),

        x => unimplemented!("{x:?}"),
    };

    loop {
        match tokens.peek().unwrap() {
            // todo remove
            Token::LParens => {
                assert_eq!(tokens.next().unwrap(), Token::LParens);

                let param = parse_value(tokens);

                assert_eq!(tokens.next().unwrap(), Token::RParens);

                value = Value::FunctionCall(Box::new(value), Box::new(param));
            }

            _ => return value,
        }
    }
}

fn parse_type(tokens: &mut Peekable<IntoIter<Token>>) -> RawType {
    let mut typ = match tokens.next().unwrap() {
        Token::LParens => {
            if *tokens.peek().unwrap() == Token::RParens {
                assert_eq!(tokens.next().unwrap(), Token::RParens);
                RawType::Unit
            } else {
                let t = parse_type(tokens);

                assert_eq!(tokens.next().unwrap(), Token::RParens);

                t
            }
        }

        Token::Name(str) => RawType::TypeDef(str),

        _ => unimplemented!(),
    };

    loop {
        match tokens.peek().unwrap() {
            // todo change this to be left assoc somehow
            Token::Minus => {
                assert_eq!(tokens.next().unwrap(), Token::Minus);
                assert_eq!(tokens.next().unwrap(), Token::RAngle);

                let ret = parse_type(tokens);

                typ = RawType::Function(Box::new(typ), Box::new(ret));
            }

            _ => return typ,
        };
    }
}
