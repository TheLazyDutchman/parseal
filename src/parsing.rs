pub mod tokens;
pub mod charstream;

use std::fmt;

use self::{charstream::{CharStream, Position, WhitespaceType, Span}, tokens::Delimiter};

pub trait Parse: Clone {
	fn parse(value: &mut CharStream) -> Result<Self, ParseError> where Self: Sized;
	fn span(&self) -> Span;
}

#[derive(Clone)]
pub struct ParseError(String, Position);

impl ParseError {
	pub fn new(cause: &str, position: Position) -> Self {
		Self(cause.to_string(), position)
	}
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}:{}:Error: '{}'", self.1.row, self.1.column, self.0)
    }
}

/// A Group represents a delimited item.
/// Group has two Generic types:
/// - `D` is the delimiter tokens around the item, it has to a type that implements [`tokens::Delimiter`].
/// - `I` is the type of item inside the delimiters, it has to implement [`Parse`].
/// ```
/// # use parseal::parsing::{charstream::CharStream, tokens, Group, StringValue, Number, List, Parse};
/// # fn main() {
/// 	let buffer = "(\"Hello, World\")".to_owned();
/// 	let mut buffer = CharStream::new(buffer).build();
/// 
/// 	let value = Group::<tokens::Paren, StringValue>::parse(&mut buffer);
/// 	assert!(value.is_ok());
/// 
/// 	let buffer = "[0, 1, 2]".to_owned();
/// 	let mut buffer = CharStream::new(buffer).build();
/// 
/// 	let value = Group::<tokens::Bracket, List<Number, tokens::Comma>>::parse(&mut buffer);
/// 	assert!(value.is_ok());
/// # }
/// ```
#[derive(Clone)]
pub struct Group<D, I> where D: tokens::Delimiter, I: Parse {
	delimiter: D,
	item: I
}

impl<D, I> Parse for Group<D, I> where
	D: tokens::Delimiter,
	I: Parse
{
    fn parse(value: &mut CharStream) -> Result<Self, ParseError> where Self: Sized {
		let start = D::Start::parse(value)?;
		let item = I::parse(value)?;
		let end = match D::End::parse(value) {
			Ok(value) => value,
			Err(error) => return Err(error)
		};

		let delimiter = D::new(start, end);

		Ok(Self { delimiter, item })
    }

	fn span(&self) -> Span {
		self.delimiter.span()
	}
}

impl<D, I> fmt::Debug for Group<D, I> where
	D: tokens::Delimiter,
	I: Parse + fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Group({:#?}, delim: {}, from {})", self.item, D::name(), self.span())
    }
}


/// A List represents a collection of items, separated by a token.
/// It has two generic types:
/// - `I` is the type of item, it has to implement [`Parse`].
/// - `S` is the token that separates the items. it has to implement [`tokens::Token`].
/// ```
/// # use parseal::parsing::{charstream::CharStream, tokens, Group, StringValue, Number, List, Parse};
/// # fn main() {
/// 	let buffer = "0, 1, 5".to_owned();
/// 	let mut buffer = CharStream::new(buffer).build();
/// 
/// 	let value = List::<Number, tokens::Comma>::parse(&mut buffer);
/// 	assert!(value.is_ok());
/// 
/// 	let buffer = "".to_owned();
/// 	let mut buffer = CharStream::new(buffer).build();
/// 
/// 	let value = List::<StringValue, tokens::Pipe>::parse(&mut buffer);
/// 	assert!(value.is_ok()); 
/// 	// A List can also be empty.
/// 
/// 	let buffer = "1012".to_owned();
/// 	let mut buffer = CharStream::new(buffer).build();
/// 
/// 	let value = List::<StringValue, tokens::Pipe>::parse(&mut buffer);
/// 	assert!(value.is_ok()); 
/// 	// the parse function is not guaranteed to consume the entire buffer.
/// 	// in this case it will not consume anything from the buffer, yet return an Ok variant, as the List is allowed to be empty.
/// # }
/// ```
#[derive(Clone)]
pub struct List<I, S> where I: Parse, S: tokens::Token {
	items: Vec<(I, Option<S>)>,
	span: Span
}

impl<I, S> Parse for List<I, S> where
	I: Parse,
	S: tokens::Token
{
	fn parse(value: &mut CharStream) -> Result<Self, ParseError> where Self: Sized {
        let mut items = Vec::new();
		let start = value.position();

		loop {
			let item = match I::parse(value) {
				Ok(value) => value,
				Err(error) => {
					if items.len() > 0 {
						return Err(error);
					}
					break
				}
			};

			let separator = match S::parse(value) {
				Ok(value) => Some(value),
				_ => {
					items.push((item, None));
					break;
				}
			};

			items.push((item, separator));
		}

		let end = value.position();

		Ok(Self { items, span: Span::new(start, end) })
    }

	fn span(&self) -> Span {
		self.span.clone()
	}
}

impl<I, S> fmt::Debug for List<I, S> where 
	I: Parse + fmt::Debug,
	S: tokens::Token + fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "List({:#?}, from {})", self.items, self.span())
    }
}

/// StringValue represents a string.
/// this is necessary because it needs to store some additional information for the AST, like the info necessary for [`Parse::span`].
/// ```
/// # use parseal::parsing::{StringValue, Parse, charstream::CharStream};
/// # fn main() {
/// 	let mut buffer = CharStream::new("\"Hello, world!\"".to_owned()).build();
/// 	let value = StringValue::parse(&mut buffer);
/// 
/// 	assert!(value.is_ok());
/// # }
/// ```
#[derive(Clone)]
pub struct StringValue {
	delim: tokens::Quote,
	value: String
}

impl Parse for StringValue {
	fn parse(value: &mut CharStream) -> Result<Self, ParseError> where Self: Sized {
		let left = <tokens::Quote as tokens::Delimiter>::Start::parse(value)?;
		let mut inner_value = String::new();

		let mut string_value = value.clone();

		let mut position = string_value.position();

		string_value.set_whitespace(WhitespaceType::KeepAll);
		loop {
			match string_value.next() {
				Some(value) if value != '"' => {
					inner_value.push(value);
					position = string_value.position();
				}
				_ => break
			}
		}

		value.goto(position)?;
		
		let right = <tokens::Quote as tokens::Delimiter>::End::parse(value)?;

		Ok(Self { delim: tokens::Delimiter::new(left, right), value: inner_value})
    }

	fn span(&self) -> Span {
		self.delim.span()
	}
}

impl fmt::Debug for StringValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "StringValue({}, from {})", self.value, self.span())
    }
}

/// An Identifier represents things like words and names.
/// ```
/// # use parseal::parsing::{charstream::CharStream, Identifier, Parse, tokens, self};
/// 
/// # fn main() {
/// 	let buffer = "hello world".to_owned();
/// 	let mut buffer = CharStream::new(buffer).build();
/// 	
/// 	let value = Vec::<Identifier>::parse(&mut buffer).unwrap();
/// 	assert_eq!(value.len(), 2);
/// 
/// 	#[cfg(feature="derive")]
/// 	{
/// 		# use parseal::Parsable;
/// 		#[derive(Parsable, Clone)]
/// 		enum Bool {
/// 			True(#[value("true")] Identifier),
/// 			False(#[value("false")] Identifier)
/// 		}
/// 
/// 		let mut buffer = CharStream::new("true | false".to_owned()).build();
/// 		let value = <(Bool, tokens::Pipe, Bool)>::parse(&mut buffer);
/// 		assert!(value.is_ok());
/// 	}
/// # }
/// ```
#[derive(Clone)]
pub struct Identifier {
	identifier: String,
	span: Span
}

impl Parse for Identifier {
	fn parse(value: &mut CharStream) -> Result<Self, ParseError> where Self: Sized {
		let mut identifier = String::new();
		let start = value.position();

		let mut ident_value = value.clone();
		match ident_value.next() {
			Some(chr) if chr.is_alphabetic() => {
				let mut position = ident_value.position();
				identifier.push(chr);

				ident_value.set_whitespace(WhitespaceType::KeepAll);

				loop {
					match ident_value.next() {
						Some(value) if value.is_alphanumeric() => {
							identifier.push(value);
							position = ident_value.position();
						}
						_ => break
					}
				}

				value.goto(position)?;
			}
			_ => return Err(ParseError("Did not find identifier".to_string(), ident_value.position()))
		}

		let end = value.position();

		Ok(Self { identifier , span: Span::new(start, end)})
    }

	fn span(&self) -> Span {
		self.span.clone()
	}
}

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Identifier({}, from {})", self.identifier, self.span)
    }
}

impl PartialEq<&str> for Identifier {
    fn eq(&self, other: &&str) -> bool {
        self.identifier == other.to_owned()
    }
}

/// A Number is a representation of a number, duh.
/// this representation is needed since it needs to store some additional information for the AST.
/// ```
/// # use parseal::parsing::{Number, Parse, charstream::CharStream};
/// # fn main() {
/// 	let mut buffer = CharStream::new("69420".to_owned()).build();
/// 	let value = Number::parse(&mut buffer);
/// 
/// 	assert!(value.is_ok());
/// # }
/// ```
#[derive(Clone)]
pub struct Number {
	value: String,
	span: Span
}

impl Parse for Number {
	fn parse(value: &mut CharStream) -> Result<Self, ParseError> where Self: Sized {
		let mut number = String::new();
		let start = value.position();
		
		let mut num_value = value.clone();
		match num_value.next() {
			Some(chr) if chr.is_numeric() => {
				let mut position = num_value.position();
				number.push(chr);

				num_value.set_whitespace(WhitespaceType::KeepAll);

				loop {
					match num_value.next() {
						Some(value) if value.is_numeric() => {
							number.push(value);
							position = num_value.position();
						}
						_ => break
					}
				}

				value.goto(position)?;
			}
			_ => return Err(ParseError("Did not find number".to_string(), num_value.position()))
		}

		let end = value.position();

		Ok(Number { value: number, span: Span::new(start, end)})
    }

	fn span(&self) -> Span {
		self.span.clone()
	}
}

impl fmt::Debug for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Number({}, from {})", self.value, self.span)
    }
}

#[derive(Clone)]
pub struct Indent<T> {
	values: Vec<T>,
	depth: u8
}

impl<T> Parse for Indent<T> where T: Parse {
    fn parse(value: &mut CharStream) -> Result<Self, ParseError> where Self: Sized {
        let mut values = Vec::new();

		let mut indent_value = value.clone();
		indent_value.set_whitespace(WhitespaceType::Indent);
		let mut position = indent_value.position();

		let mut item = T::parse(&mut indent_value);
		let depth= indent_value.indent();
		while item.is_ok() {
			position = indent_value.position();
			values.push(item?);
			item = T::parse(&mut indent_value);

			if indent_value.indent() != depth {
				break;
			}
		}

		if values.len() == 0 {
			Err(ParseError("Could not find Indent block.".to_string(), position))
		} else {
			Ok(Self { values, depth })
		}
    }

    fn span(&self) -> Span {
        Span::new(self.values.first().unwrap().span().start, self.values.last().unwrap().span().end)
    }
}

impl<T> fmt::Debug for Indent<T> where T: fmt::Debug + Parse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Indent({:#?}, from {}, depth {})", self.values, self.span(), self.depth)
    }
}

impl<T> Parse for Vec<T> where T: Parse {
	fn parse(value: &mut CharStream) -> Result<Self, ParseError> where Self: Sized {
		let mut vec = Vec::new();

		let mut item = T::parse(value);
		while item.is_ok() {
			vec.push(item?);
			item = T::parse(value);
		}

		if vec.len() == 0 {
			Err(ParseError("Could not find vector.".to_string(), value.position()))
		} else {
			Ok(vec)
		}
	}

	fn span(&self) -> Span {
		Span::new(self.first().unwrap().span().start, self.last().unwrap().span().start)
	}
}

impl<T, const N: usize> Parse for [T; N] where T: Parse + fmt::Debug {
    fn parse(value: &mut CharStream) -> Result<Self, ParseError> where Self: Sized {
        let mut result = Vec::new();

		for _ in 0..N {
			result.push(T::parse(value)?);
		}

		match <[T; N]>::try_from(result) {
			Ok(result) => Ok(result),
			Err(error) => Err(ParseError(format!("Could not create slice from parsed values. \nvalues where: {:?}", error), value.position()))
		}
    }

	fn span(&self) -> Span {
		Span::new(self[0].span().start, self[N - 1].span().end)
	}
}

//TODO: see if this can be more general
impl<A, B> Parse for (A, B) where
	A: Parse,
	B: Parse
{
    fn parse(value: &mut CharStream) -> Result<Self, ParseError> where Self: Sized {
        Ok((
			A::parse(value)?,
			B::parse(value)?
		))
    }
	
	fn span(&self) -> Span {
		Span::new(self.0.span().start, self.1.span().end)
	}
}

impl<A, B, C> Parse for (A, B, C) where
	A: Parse,
	B: Parse,
	C: Parse
{
    fn parse(value: &mut CharStream) -> Result<Self, ParseError> where Self: Sized {
        Ok((
			A::parse(value)?,
			B::parse(value)?,
			C::parse(value)?
		))
    }

	fn span(&self) -> Span {
		Span::new(self.0.span().start, self.2.span().end)
	}
}
