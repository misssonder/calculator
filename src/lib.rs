use crate::ast::{Expression, Literal, Operation};
use crate::error::{Error, Result};
use crate::parse::Parser;
use std::fmt::{Display, Formatter};

mod ast;
mod error;
mod lexer;
mod parse;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(i) => f.write_str(i.to_string().as_ref()),
            Value::Float(i) => f.write_str(i.to_string().as_ref()),
        }
    }
}

impl From<ast::Literal> for Value {
    fn from(literal: Literal) -> Self {
        match literal {
            Literal::Integer(integer) => Value::Integer(integer),
            Literal::Float(float) => Value::Float(float),
        }
    }
}

trait Calculate {
    fn calculate(&self) -> Result<Value>;
}

impl<T: AsRef<str>> Calculate for T {
    fn calculate(&self) -> Result<Value> {
        Calculator::new(self.as_ref()).calculate()
    }
}

pub struct Calculator<'a> {
    parser: Parser<'a>,
}

impl Calculator<'_> {
    pub fn new(input: &str) -> Calculator {
        Calculator {
            parser: Parser::new(input),
        }
    }
    pub fn calculate(&mut self) -> Result<Value> {
        let expr = self.parser.parse()?;
        Self::calculate_expression(expr)
    }

    fn calculate_expression(expression: Expression) -> Result<Value> {
        Ok(match expression {
            Expression::Literal(literal) => literal.into(),
            Expression::Operation(operation) => match operation {
                Operation::Add(lhs, rhs) => {
                    match (
                        Self::calculate_expression(*lhs)?,
                        Self::calculate_expression(*rhs)?,
                    ) {
                        (Value::Integer(lhs), Value::Integer(rhs)) => Value::Integer(
                            lhs.checked_add(rhs)
                                .ok_or(Error::Value("Integer overflow".into()))?,
                        ),
                        (Value::Float(lhs), Value::Float(rhs)) => Value::Float(lhs + rhs),
                        (Value::Integer(lhs), Value::Float(rhs)) => Value::Float(lhs as f64 + rhs),
                        (Value::Float(lhs), Value::Integer(rhs)) => Value::Float(lhs + rhs as f64),
                    }
                }
                Operation::Assert(lhs) => Self::calculate_expression(*lhs)?,
                Operation::Divide(lhs, rhs) => match (
                    Self::calculate_expression(*lhs)?,
                    Self::calculate_expression(*rhs)?,
                ) {
                    (Value::Integer(lhs), Value::Integer(rhs)) => Value::Integer(lhs - rhs),
                    (Value::Float(lhs), Value::Float(rhs)) => Value::Float(lhs - rhs),
                    (Value::Integer(lhs), Value::Float(rhs)) => Value::Float(lhs as f64 - rhs),
                    (Value::Float(lhs), Value::Integer(rhs)) => Value::Float(lhs - rhs as f64),
                },
                Operation::Exponentiate(lhs, rhs) => match (
                    Self::calculate_expression(*lhs)?,
                    Self::calculate_expression(*rhs)?,
                ) {
                    (Value::Integer(lhs), Value::Integer(rhs)) if rhs >= 0 => Value::Integer(
                        lhs.checked_pow(rhs as u32)
                            .ok_or(Error::Value("Integer overflow".into()))?,
                    ),
                    (Value::Integer(lhs), Value::Integer(rhs)) => {
                        Value::Float((lhs as f64).powf(rhs as f64))
                    }
                    (Value::Float(lhs), Value::Float(rhs)) => Value::Float(lhs.powf(rhs)),
                    (Value::Integer(lhs), Value::Float(rhs)) => {
                        Value::Float((lhs as f64).powf(rhs))
                    }
                    (Value::Float(lhs), Value::Integer(rhs)) => Value::Float(lhs.powf(rhs as f64)),
                },
                Operation::Factorial(lhs) => match Self::calculate_expression(*lhs)? {
                    Value::Integer(i) if i < 0 => {
                        return Err(Error::Value(
                            "Can't take factorial of negative number".into(),
                        ));
                    }
                    Value::Integer(i) => Value::Integer((1..=i).product()),
                    other => {
                        return Err(Error::Value(format!("Can't take factorial of {}", other)));
                    }
                },
                Operation::Modulo(lhs, rhs) => match (
                    Self::calculate_expression(*lhs)?,
                    Self::calculate_expression(*rhs)?,
                ) {
                    (Value::Integer(_), Value::Integer(0)) => {
                        return Err(Error::Value("Can't divide by zero".into()));
                    }
                    (Value::Integer(lhs), Value::Integer(rhs)) => Value::Integer(lhs % rhs),
                    (Value::Float(lhs), Value::Float(rhs)) => Value::Float(lhs % rhs),
                    (Value::Integer(lhs), Value::Float(rhs)) => Value::Float(lhs as f64 % rhs),
                    (Value::Float(lhs), Value::Integer(rhs)) => Value::Float(lhs % rhs as f64),
                },
                Operation::Multiply(lhs, rhs) => match (
                    Self::calculate_expression(*lhs)?,
                    Self::calculate_expression(*rhs)?,
                ) {
                    (Value::Integer(lhs), Value::Integer(rhs)) => Value::Integer(
                        lhs.checked_mul(rhs)
                            .ok_or(Error::Value("Integer overflow".into()))?,
                    ),
                    (Value::Float(lhs), Value::Float(rhs)) => Value::Float(lhs * rhs),
                    (Value::Integer(lhs), Value::Float(rhs)) => Value::Float(lhs as f64 * rhs),
                    (Value::Float(lhs), Value::Integer(rhs)) => Value::Float(lhs * rhs as f64),
                },
                Operation::Negate(lhs) => match Self::calculate_expression(*lhs)? {
                    Value::Integer(i) => Value::Integer(-i),
                    Value::Float(f) => Value::Float(-f),
                },
                Operation::Subtract(lhs, rhs) => match (
                    Self::calculate_expression(*lhs)?,
                    Self::calculate_expression(*rhs)?,
                ) {
                    (Value::Integer(_), Value::Integer(0)) => {
                        return Err(Error::Value("Can't divide by zero".into()));
                    }
                    (Value::Integer(lhs), Value::Integer(rhs)) => Value::Integer(lhs / rhs),
                    (Value::Float(lhs), Value::Float(rhs)) => Value::Float(lhs / rhs),
                    (Value::Integer(lhs), Value::Float(rhs)) => Value::Float(lhs as f64 / rhs),
                    (Value::Float(lhs), Value::Integer(rhs)) => Value::Float(lhs / rhs as f64),
                },
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate() {
        {
            let calculator = "1+1".calculate();
            assert_eq!(calculator, Ok(Value::Integer(2)))
        }

        {
            let calculator = "1*1".calculate();
            assert_eq!(calculator, Ok(Value::Integer(1)))
        }

        {
            let calculator = "2*4".calculate();
            assert_eq!(calculator, Ok(Value::Integer(8)))
        }

        {
            let calculator = "4!".calculate();
            assert_eq!(calculator, Ok(Value::Integer(24)))
        }

        {
            let calculator = "31%15".calculate();
            assert_eq!(calculator, Ok(Value::Integer(1)))
        }

        {
            let calculator = "1*!1".calculate();
            assert!(calculator.is_err())
        }

        {
            let calculator = "(1+1)*2+4!".calculate();
            assert_eq!(calculator, Ok(Value::Integer(28)))
        }

        {
            let calculator = "(1.1+1.1)*2+4!".to_string().calculate();
            assert_eq!(calculator, Ok(Value::Float(28.4)))
        }
    }
}
