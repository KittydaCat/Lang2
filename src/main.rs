mod parser;
mod tokenizer;
mod typer;

use crate::{parser::parse_definitions, tokenizer::tokenize, typer::type_definition};

fn test(code: &str, input: &str) {
    let def = parse_definitions(&mut dbg!(tokenize(code)).into_iter().peekable());

    dbg!(type_definition(dbg!(def)));
}

fn main() {
    test(
        "
Type Option = Some () | None ();
Let z: Option -> () = |x| Match x {
    Some |y| (),
    None |y| (),
};
",
        "",
    );
}

#[cfg(test)]
pub mod tests {
    use crate::test;

    #[test]
    pub fn test4() {
        test(
            "
Let z: () = ( |x| x(()) ) ( |y|() );
",
            "",
        );
    }
    #[test]
    pub fn test3() {
        test(
            "
Let z: () -> () -> () = |x||y|();
",
            "",
        );
    }

    #[test]
    pub fn test1() {
        test(
            "
Type Option = Some () | None ();
Let x: () -> () = |y| ();
",
            "",
        );
    }

    #[test]
    pub fn test2() {
        test(
            "
Type Option = Some () | None ();
Let x: () = (|y| ())(());
",
            "",
        );
    }
}
