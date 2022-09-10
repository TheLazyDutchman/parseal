use std::{slice::Iter, iter::Peekable, collections::HashMap};

use crate::{parsing::{AST, ParserError, token::Token}, expect};

use super::{TreeData, Data};

#[derive(Debug, PartialEq)]
pub struct JSON {
	pub value: Data   
}

impl AST for JSON {
    fn parse_tokens(tokens: &mut Peekable<Iter<Token>>) -> Result<Self, ParserError>
			where Self: Sized {
        Ok(Self {value: Self::parse_data(tokens)?})
    }

    fn keywords() -> &'static [&'static str] {
        &["true", "false"]
    }

    fn operators() -> &'static [&'static str] {
        &["{","}",",","[","]",":"]
    }

    fn ignore_whitespace() -> bool {
        true
    }
}

impl TreeData for JSON {
	fn parse_data(tokens: &mut Peekable<Iter<Token>>) -> Result<Data, ParserError> {
		match tokens.peek().ok_or(ParserError::eof())? {
			Token::Operator(op) if *op == "[" => {
				Self::parse_list(tokens)
			}
			Token::Operator(op) if *op == "{" => {
				Self::parse_object(tokens)
			}
			Token::String(_) | Token::Number(_) => {
				Ok(Data::Immediate(tokens.next().unwrap().clone()))
			}
			token => {
				Err(ParserError::new(format!("Unexpected token '{:?}'", token).to_owned()))
			}
		}
	}

	fn parse_list(tokens: &mut Peekable<Iter<Token>>) -> Result<Data, ParserError> {
		let mut list = Vec::new();

		tokens.next();

		while tokens.len() > 0  {
			list.push(Self::parse_data(tokens)?);

			if let Some(Token::Operator(op)) = tokens.peek() {
				if *op == "]" {
					break;
				}

				if *op != "," {
					return Err(ParserError::new(format!("Expected ',' in list, but found '{}'.", op).to_owned()))
				}
				
				tokens.next();
			}
		}

		tokens.next().ok_or(ParserError::new(format!("Expected ']' after list")))?;

		Ok(Data::List(list))
	}

	fn parse_object(tokens: &mut Peekable<Iter<Token>>) -> Result<Data, ParserError> {
		let mut map = HashMap::new();

		tokens.next();

		while tokens.len() > 0 {
			let name = expect!(tokens, Token::String(value), value, "Expected property name, but got {:?}").to_owned();

			expect!(tokens, Token::Operator(op), op, op == ":", "Expected ':' but got '{:?}'");

			let value = Self::parse_data(tokens)?;

			map.insert(name, value);
			
			match tokens.peek() {
				Some(Token::Operator(op)) if *op == "}" => {
					break;
				}
				Some(Token::Operator(op)) if *op == "," => {
					tokens.next();
				}
				token => return Err(ParserError::new(format!("Expected ',' but got '{:?}'", token)))
			}
		}

		tokens.next().ok_or(ParserError::new("Expected '}' after object".to_owned()))?;

		Ok(Data::Object(map))
	}
}