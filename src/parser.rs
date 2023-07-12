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

    let (final_index, final_expression) = parse_expression(0, tokens);

    if final_index != tokens.len() {
        panic!("did not parse expression completely");
    }

    return final_expression;
}
