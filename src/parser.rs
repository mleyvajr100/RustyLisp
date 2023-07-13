use crate::lisp_expression::LispExpression;
use crate::tokenizer::LispToken;

pub fn parse(tokens: &Vec<LispToken>) -> LispExpression {
    fn parse_expression(mut index: usize, tokens: &Vec<LispToken>) -> (usize, LispExpression) {
        let token = &tokens[index];

        match token {
            LispToken::Integer(num) => (index + 1, LispExpression::Integer(*num)),
            LispToken::Symbol(sym) => (index + 1, LispExpression::Symbol(sym.clone())),
            LispToken::RightParen => panic!("unmatched right parenthesis while trying to parse expression at index: {index}"),
            LispToken::LeftParen => {
                let mut expressions = Vec::new();
                index += 1;

                while index < tokens.len() && tokens[index] != LispToken::RightParen {
                    let (next_index, expression) = parse_expression(index, tokens);
                    index = next_index;
                    expressions.push(expression);
                }

                if index >= tokens.len() || tokens[index] != LispToken::RightParen {
                    panic!("missing right parenthesis while trying to parse expression");
                }

                return (index + 1, LispExpression::List(expressions));
            }
        }
    }

    if tokens.len() == 0 {
        panic!("nothing to parse!");
    }
    let (final_index, final_expression) = parse_expression(0, tokens);

    if final_index != tokens.len() {
        panic!("did not parse expression completely");
    }

    return final_expression;
}


// ============== TESTS ===============

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;

    #[test]
    #[should_panic]
    fn nothing_to_parse() {
        let tokens = tokenize("");
        parse(&tokens);
    }

    #[test]
    fn single_number() {
        let tokens = tokenize("1");
        let parsed_integer = parse(&tokens);
        assert_eq!(LispExpression::Integer(1), parsed_integer);
    }

    #[test]
    fn single_symbol() {
        let tokens = tokenize("x");
        let parsed_integer = parse(&tokens);
        assert_eq!(LispExpression::Symbol("x".to_string()), parsed_integer);
    }

    #[test]
    #[should_panic]
    fn single_open_parenthesis() {
        let tokens = tokenize("(");
        parse(&tokens);
    }

    #[test]
    #[should_panic]
    fn single_closed_parenthesis() {
        let tokens = tokenize(")");
        parse(&tokens);
    }

    #[test]
    fn single_list_expression() {
        let tokens = tokenize("(define x 2)");
        let define_expr = parse(&tokens);

        let expected = LispExpression::List(vec![
            LispExpression::Symbol("define".to_string()),
            LispExpression::Symbol("x".to_string()),
            LispExpression::Integer(2),
        ]);

        assert_eq!(expected, define_expr);
    }

    #[test]
    fn single_list_expression_with_comments() {
        let define_expr = parse(&tokenize("(define x 2)"));
        let define_expr_with_comments = parse(&tokenize("(define x 2); this is a comment"));

        let expected = LispExpression::List(vec![
            LispExpression::Symbol("define".to_string()),
            LispExpression::Symbol("x".to_string()),
            LispExpression::Integer(2),
        ]);
        
        assert_eq!(expected, define_expr);
        assert_eq!(expected, define_expr_with_comments);
    }

    #[test]
    #[should_panic]
    fn unfinished_expression() {
        let tokens = tokenize("(+ 2 3");
        parse(&tokens);
    }

    #[test]
    #[should_panic]
    fn list_expression_without_parenthesis() {
        let tokens = tokenize("+ 2 3");
        parse(&tokens);
    }
}