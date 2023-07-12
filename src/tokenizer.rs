#[derive(Debug)]
#[derive(PartialEq)]
pub enum LispToken {
    Integer(i64),
    Symbol(String),
    LeftParen,
    RightParen,
}


pub fn tokenize(source: &str) -> Vec<LispToken> {
    let mut source_without_comments = String::new();
    // remove comments before separating source by parenthesis
    for line in source.split("\n") {
        for line_char in line.chars() {
            if line_char == ';' {
                break;
            }
            source_without_comments.push(line_char)
        }
    }

    // replace parenthesis with space-padded parenthesis to make splitting string easier
    let words = source_without_comments[..]
                    .replace("(", " ( ")
                    .replace(")", " ) ");

    let words = words.split_whitespace();

    let mut tokens = Vec::new();

    for word in words {
        match word {
            "(" => tokens.push(LispToken::LeftParen),
            ")" => tokens.push(LispToken::RightParen),
            _ => {
                let expression = word.parse::<i64>();
                if expression.is_ok() {
                    tokens.push(LispToken::Integer(expression.unwrap()));
                } else {
                    tokens.push(LispToken::Symbol(word.to_string()));
                }
            },
        }
    }
    return tokens;
}


// ============== TESTS ===============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_to_tokenize() {
        let empty_list: Vec<LispToken> = Vec::new();
        assert_eq!(empty_list, tokenize(""));
    }
    
    #[test]
    fn single_characters() {
        assert_eq!(vec![LispToken::LeftParen], tokenize("("));
        assert_eq!(vec![LispToken::RightParen], tokenize(")"));
        assert_eq!(vec![LispToken::Integer(0)], tokenize("0"));
        assert_eq!(vec![LispToken::Symbol("x".to_string())], tokenize("x"));
    }

    #[test]
    fn multicharacter_symbols() {
        assert_eq!(vec![LispToken::Symbol("hello".to_string())], tokenize("hello"));
        assert_eq!(vec![LispToken::Symbol("world".to_string())], tokenize("world"));
    }

    #[test]
    fn multidigit_integers() {
        assert_eq!(vec![LispToken::Integer(101)], tokenize("101"));
        assert_eq!(vec![LispToken::Integer(12345)], tokenize("12345"));
        assert_eq!(vec![LispToken::Integer(-404)], tokenize("-404"));
    }

    #[test]
    fn simple_definition() {
        let x_definition = "(define x 2)";
        let expected_tokens = vec![
            LispToken::LeftParen,
            LispToken::Symbol("define".to_string()),
            LispToken::Symbol("x".to_string()),
            LispToken::Integer(2),
            LispToken::RightParen,
        ];

        assert_eq!(expected_tokens, tokenize(x_definition));
    }

    #[test]
    fn multiline_function() {
        let add_one_function = "\
            (
                define 
                    add_one
                    (lambda 
                        (x)
                        (+ x 1)
                    )
            )";

        let add_one_function_with_comments = "\
        ( ; gonna define a function that adds 1 to an integer
            define ; starts here!
                add_one
                (lambda
                    (x)
                    (+ x 1)
                )
        )";

        
        let expected_tokens = vec![
            LispToken::LeftParen,
            LispToken::Symbol("define".to_string()),
            LispToken::Symbol("add_one".to_string()),
            LispToken::LeftParen,
            LispToken::Symbol("lambda".to_string()),
            LispToken::LeftParen,
            LispToken::Symbol("x".to_string()),
            LispToken::RightParen,
            LispToken::LeftParen,
            LispToken::Symbol("+".to_string()),
            LispToken::Symbol("x".to_string()),
            LispToken::Integer(1),
            LispToken::RightParen,
            LispToken::RightParen,
            LispToken::RightParen,
        ];

        assert_eq!(expected_tokens, tokenize(add_one_function));
        assert_eq!(expected_tokens, tokenize(add_one_function_with_comments));
    }

}