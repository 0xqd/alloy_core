use crate::{kw, FunctionAttributes, Parameters, Returns, SolIdent, SolTuple, Type};
use proc_macro2::Span;
use std::fmt;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    token::{Brace, Paren},
    Attribute, Result, Token,
};

/// A function definition:
/// `function helloWorld() external pure returns(string memory);`
///
/// Solidity reference:
/// <https://docs.soliditylang.org/en/latest/grammar.html#a4.SolidityParser.functionDefinition>
pub struct ItemFunction {
    /// The `syn` attributes of the function.
    pub attrs: Vec<Attribute>,
    pub function_token: kw::function,
    pub name: SolIdent,
    pub paren_token: Paren,
    pub arguments: Parameters<Token![,]>,
    /// The Solidity attributes of the function.
    pub attributes: FunctionAttributes,
    /// The optional return types of the function.
    pub returns: Option<Returns>,
    pub semi_token: Token![;],
}

impl fmt::Debug for ItemFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Function")
            .field("name", &self.name)
            .field("arguments", &self.arguments)
            .field("attributes", &self.attributes)
            .field("returns", &self.returns)
            .finish()
    }
}

impl Parse for ItemFunction {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        fn parse_check_brace<T: Parse>(input: ParseStream<'_>) -> Result<T> {
            if input.peek(Brace) {
                Err(input.error("functions cannot have an implementation"))
            } else {
                input.parse()
            }
        }

        let content;
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            function_token: input.parse()?,
            name: input.parse()?,
            paren_token: parenthesized!(content in input),
            arguments: content.parse()?,
            attributes: parse_check_brace(input)?,
            returns: if input.peek(kw::returns) {
                Some(input.parse()?)
            } else {
                None
            },
            semi_token: parse_check_brace(input)?,
        })
    }
}

impl ItemFunction {
    pub fn span(&self) -> Span {
        self.name.span()
    }

    pub fn set_span(&mut self, span: Span) {
        self.name.set_span(span);
    }

    /// Returns true if the function returns nothing.
    pub fn is_void(&self) -> bool {
        match &self.returns {
            None => true,
            Some(returns) => returns.returns.is_empty(),
        }
    }

    /// Returns the function signature as a string.
    pub fn signature(&self) -> String {
        self.arguments.signature(self.name.as_string())
    }

    /// Returns the function's signature tuple type.
    pub fn call_type(&self) -> Type {
        let mut args = self
            .arguments
            .iter()
            .map(|arg| arg.ty.clone())
            .collect::<SolTuple>();
        // ensure trailing comma for single item tuple
        if !args.types.trailing_punct() && args.types.len() == 1 {
            args.types.push_punct(Default::default())
        }
        Type::Tuple(args)
    }
}