#[derive(Debug, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
}

#[derive(Debug, PartialEq)]
pub enum Operation {
    Add(Box<Expression>, Box<Expression>),
    Assert(Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
    Exponentiate(Box<Expression>, Box<Expression>),
    Factorial(Box<Expression>),
    Modulo(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Negate(Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Operation(Operation),
}

impl From<Literal> for Expression {
    fn from(literal: Literal) -> Self {
        Expression::Literal(literal)
    }
}

impl From<Operation> for Expression {
    fn from(operation: Operation) -> Self {
        Expression::Operation(operation)
    }
}
