use std::{slice::Iter, iter::Peekable};

use crate::parsing::{AST, token::{Token}, ParserError};

use super::{Row, GridData};

#[derive(Debug, PartialEq)]
pub struct CSV {
	pub values: Vec<Row>
}

impl AST for CSV {
	fn parse_tokens(tokens: &mut Peekable<Iter<Token>>) -> Result<Self, ParserError>
			where Self: Sized {
		let mut rows = Vec::new();

		while tokens.len() > 0 {
			rows.push(Self::parse_row(tokens)?);
		}

		Ok( Self { values: rows } )
    }

	fn keywords() -> &'static [&'static str] {
		&["true", "false"]
    }

	fn operators() -> &'static [&'static str] {
		&[","]
    }

	fn ignore_whitespace() -> bool {
		false
    }
}

impl GridData for CSV {
	fn parse_data(tokens: &mut Peekable<Iter<Token>>) -> Result<Token, ParserError> {
		match tokens.peek() {
			Some(Token::String(_)) | Some(Token::Number(_)) | Some(Token::Keyword(_)) | Some(Token::Identifier(_)) => {
				Ok(tokens.next().unwrap().clone())
			}
			token => Err(ParserError::new(format!("Expected value, but got {:?}", token)))
		}
	}

	fn parse_row(tokens: &mut Peekable<Iter<Token>>) -> Result<Row, ParserError> {
		let mut values = Vec::new();
		
		while tokens.len() > 0 {
			values.push(Self::parse_data(tokens)?);
			match tokens.peek() {
				Some(Token::WhiteSpace(value)) if value == "\n" || value == "\r\n" => {
					tokens.next();
					break;
				}
				Some(Token::Operator(op)) if op == "," => {
					tokens.next();
				}
				None => break,
				token => {
					return Err(ParserError::new(format!("Expected ',', but got '{token:?}")));
				}
			}
		}

		Ok(Row { values } )
	}
}