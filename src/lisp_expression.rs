#[derive(Debug, Clone, PartialEq)]
pub enum LispExpression {
    Integer(i64),
    Symbol(String),
    List(Vec<LispExpression>),
}