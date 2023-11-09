use crate::ast;
use crate::lexer::{Lexer, Token};

use crate::error::{Error, Result};

pub(crate) struct Parser<'a> {
    lexer: std::iter::Peekable<Lexer<'a>>,
}

impl Parser<'_> {
    pub fn new(query: &str) -> Parser {
        Parser {
            lexer: Lexer::new(query).peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<ast::Expression> {
        self.parse_expression(0)
    }

    fn parse_expression(&mut self, min_prec: u8) -> Result<ast::Expression> {
        let mut lhs = if let Some(prefix) = self.next_if_operator::<PrefixOperator>(min_prec)? {
            prefix.build(self.parse_expression(prefix.prec() + prefix.assoc())?)
        } else {
            self.parse_expression_atom()?
        };
        while let Some(postfix) = self.next_if_operator::<PostfixOperator>(min_prec)? {
            lhs = postfix.build(lhs)
        }
        while let Some(infix) = self.next_if_operator::<InfixOperator>(min_prec)? {
            lhs = infix.build(lhs, self.parse_expression(infix.prec() + infix.assoc())?)
        }
        Ok(lhs)
    }

    fn parse_expression_atom(&mut self) -> Result<ast::Expression> {
        Ok(match self.next()? {
            Token::Number(n) => {
                if n.chars().all(|c| c.is_ascii_digit()) {
                    ast::Literal::Integer(n.parse()?).into()
                } else {
                    ast::Literal::Float(n.parse()?).into()
                }
            }
            Token::OpenParen => {
                let expr = self.parse_expression(0)?;
                self.next_expect(Some(Token::CloseParen))?;
                expr
            }
            t => {
                return Err(Error::Parse(format!(
                    "Expected expression atom, found {}",
                    t
                )));
            }
        })
    }

    fn next(&mut self) -> Result<Token> {
        self.lexer
            .next()
            .unwrap_or(Err(Error::Parse("Unexpected end of input".into())))
    }

    fn peek(&mut self) -> Result<Option<Token>> {
        self.lexer.peek().cloned().transpose()
    }

    fn next_expect(&mut self, expect: Option<Token>) -> Result<Option<Token>> {
        if let Some(t) = expect {
            let token = self.next()?;
            if token == t {
                Ok(Some(token))
            } else {
                Err(Error::Parse(format!(
                    "Expected token {}, found {}",
                    t, token
                )))
            }
        } else if let Some(token) = self.peek()? {
            Err(Error::Parse(format!("Unexpected token {}", token)))
        } else {
            Ok(None)
        }
    }

    fn next_if_operator<O: Operator>(&mut self, min_prec: u8) -> Result<Option<O>> {
        if let Some(operator) = self
            .peek()
            .unwrap_or(None)
            .and_then(|t| O::from(&t))
            .filter(|o| o.prec() >= min_prec)
        {
            self.next()?;
            Ok(Some(operator))
        } else {
            Ok(None)
        }
    }
}

/// An operator trait, to help with parsing of operators
trait Operator: Sized {
    /// Looks up the corresponding operator for a token, if one exists
    fn from(token: &Token) -> Option<Self>;
    /// Augments an operator by allowing it to parse any modifiers.
    fn augment(self, parser: &mut Parser) -> Result<Self>;
    /// Returns the operator's associativity
    fn assoc(&self) -> u8;
    /// Returns the operator's precedence
    fn prec(&self) -> u8;
}

const ASSOC_LEFT: u8 = 1;
const ASSOC_RIGHT: u8 = 0;

enum PrefixOperator {
    Minus,
    Plus,
}

impl PrefixOperator {
    fn build(&self, lhs: ast::Expression) -> ast::Expression {
        let lhs = Box::new(lhs);
        match self {
            PrefixOperator::Minus => ast::Operation::Negate(lhs),
            PrefixOperator::Plus => ast::Operation::Assert(lhs),
        }
        .into()
    }
}

impl Operator for PrefixOperator {
    fn from(token: &Token) -> Option<Self> {
        match token {
            Token::Minus => Some(Self::Minus),
            Token::Plus => Some(Self::Plus),
            _ => None,
        }
    }

    fn augment(self, _parser: &mut Parser) -> Result<Self> {
        Ok(self)
    }

    fn assoc(&self) -> u8 {
        ASSOC_RIGHT
    }

    fn prec(&self) -> u8 {
        9
    }
}

enum InfixOperator {
    Add,
    Divide,
    Exponentiate,
    Multiply,
    Subtract,
    Modulo,
}

impl InfixOperator {
    fn build(&self, lhs: ast::Expression, rhs: ast::Expression) -> ast::Expression {
        let lhs = Box::new(lhs);
        let rhs = Box::new(rhs);
        match self {
            InfixOperator::Add => ast::Operation::Add(lhs, rhs),
            InfixOperator::Divide => ast::Operation::Divide(lhs, rhs),
            InfixOperator::Exponentiate => ast::Operation::Exponentiate(lhs, rhs),
            InfixOperator::Multiply => ast::Operation::Multiply(lhs, rhs),
            InfixOperator::Subtract => ast::Operation::Subtract(lhs, rhs),
            InfixOperator::Modulo => ast::Operation::Modulo(lhs, rhs),
        }
        .into()
    }
}

impl Operator for InfixOperator {
    fn from(token: &Token) -> Option<Self> {
        match token {
            Token::Plus => Some(Self::Add),
            Token::Minus => Some(Self::Divide),
            Token::Caret => Some(Self::Exponentiate),
            Token::Asterisk => Some(Self::Multiply),
            Token::Slash => Some(Self::Subtract),
            Token::Percent => Some(Self::Modulo),
            _ => None,
        }
    }

    fn augment(self, _parser: &mut Parser) -> Result<Self> {
        Ok(self)
    }

    fn assoc(&self) -> u8 {
        match self {
            Self::Exponentiate => ASSOC_RIGHT,
            _ => ASSOC_LEFT,
        }
    }

    fn prec(&self) -> u8 {
        match self {
            Self::Add | Self::Subtract => 5,
            Self::Multiply | Self::Divide | Self::Modulo => 6,
            Self::Exponentiate => 7,
        }
    }
}

enum PostfixOperator {
    Factorial,
}

impl PostfixOperator {
    fn build(&self, lhs: ast::Expression) -> ast::Expression {
        let lhs = Box::new(lhs);
        match self {
            PostfixOperator::Factorial => ast::Operation::Factorial(lhs),
        }
        .into()
    }
}

impl Operator for PostfixOperator {
    fn from(token: &Token) -> Option<Self> {
        match token {
            Token::Exclamation => Some(Self::Factorial),
            _ => None,
        }
    }

    fn augment(self, _parser: &mut Parser) -> Result<Self> {
        Ok(self)
    }

    fn assoc(&self) -> u8 {
        ASSOC_LEFT
    }

    fn prec(&self) -> u8 {
        8
    }
}
