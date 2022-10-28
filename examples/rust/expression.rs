use parseal::{
    parsing::{self, tokens, Group, Identifier, List, Parse, StringValue},
    Parsable,
};

use crate::typedata::TypePathReference;

#[derive(Parsable, Clone, Debug)]
pub enum Mutability {
    Mutable(#[value("mut")] Identifier),
    Immutable,
}

#[derive(Parsable, Clone, Debug)]
pub enum Reference {
    Reference(tokens::Ampersand),
    Mutable(tokens::Ampersand, #[value("mut")] Identifier),
}

#[derive(Parsable, Clone, Debug)]
pub struct Let {
    #[value("let")]
    keyword: Identifier,
    mutable: Mutability,
    name: Identifier,
    vartype: Option<(tokens::Colon, TypePathReference)>,
    token: tokens::Equal,
    value: Expression,
    semicolon: tokens::Semicolon,
}

#[derive(Parsable, Clone, Debug)]
pub enum Statement {
    Let(Let),
    Expression(Expression, tokens::Semicolon),
    ReturnExpression(Expression),
    ReturnStatement(#[value("return")] Identifier, Expression, tokens::Semicolon),
}

#[derive(Parsable, Clone, Debug)]
pub struct Expression {
    value: Read,
}

#[derive(Parsable, Clone, Debug)]
pub enum Read {
    Value(Call),
    List(Call, tokens::Period, List<Call, tokens::Period>),
}

#[derive(Parsable, Clone, Debug)]
pub enum Call {
    Value(Path),
    Call(Path, Group<tokens::Paren, List<Box<Expression>>>),
    Macro(
        Path,
        tokens::Bang,
        Group<tokens::Paren, List<Box<Expression>>>,
    ),
}

#[derive(Parsable, Clone, Debug)]
pub enum Path {
    Value(Value),
    ReferencedValue(Reference, Value),
    Path(
        Identifier,
        tokens::DoubleColon,
        List<Identifier, tokens::DoubleColon>,
    ),
    ReferencedPath(Reference, List<Identifier, tokens::DoubleColon>),
}

#[derive(Parsable, Clone, Debug)]
pub enum Value {
    Name(Identifier),
    String(StringValue),
}