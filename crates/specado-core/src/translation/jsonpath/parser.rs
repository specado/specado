//! JSONPath expression parser
//!
//! This module implements a recursive descent parser for JSONPath expressions
//! with comprehensive error reporting and recovery.
//!
//! Copyright (c) 2025 Specado Team
//! Licensed under the Apache-2.0 license

use super::ast::*;
use super::error::*;
use crate::Result;
use std::str::Chars;
use std::iter::Peekable;

/// JSONPath expression parser
pub struct Parser<'a> {
    /// Input string being parsed
    input: &'a str,
    /// Character iterator
    chars: Peekable<Chars<'a>>,
    /// Current position in input
    position: usize,
    /// Current line number (1-based)
    line: usize,
    /// Current column number (1-based)
    column: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given input
    pub fn new(input: &'a str) -> Result<Self> {
        if input.is_empty() {
            return Err(JSONPathError::parse("Empty JSONPath expression", 0, input).into());
        }
        
        Ok(Self {
            input,
            chars: input.chars().peekable(),
            position: 0,
            line: 1,
            column: 1,
        })
    }

    /// Parse the JSONPath expression into an AST
    pub fn parse(mut self) -> Result<Expression> {
        let root = self.parse_root()?;
        let mut selectors = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();
            if self.is_at_end() {
                break;
            }
            
            let selector = self.parse_selector()?;
            selectors.push(selector);
        }

        Ok(Expression::new(root, selectors))
    }

    /// Parse the root selector ($)
    fn parse_root(&mut self) -> Result<Selector> {
        self.skip_whitespace();
        
        if self.current_char() != Some('$') {
            return Err(JSONPathError::syntax(
                "JSONPath must start with $",
                self.position,
                self.input,
                vec!["$".to_string()],
                self.current_char().map(|c| c.to_string()).unwrap_or_else(|| "EOF".to_string()),
            ).into());
        }
        
        self.advance();
        Ok(Selector::Root)
    }

    /// Parse a selector
    fn parse_selector(&mut self) -> Result<Selector> {
        self.skip_whitespace();
        
        match self.current_char() {
            Some('.') => self.parse_dot_selector(),
            Some('[') => self.parse_bracket_selector(),
            Some('*') => {
                self.advance();
                Ok(Selector::Wildcard)
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                // Direct property access (can occur after recursive descent)
                let property = self.parse_identifier()?;
                Ok(Selector::Child(ChildSelector::Property(property)))
            }
            Some(ch) => Err(JSONPathError::syntax(
                "Unexpected character in selector",
                self.position,
                self.input,
                vec![".".to_string(), "[".to_string(), "*".to_string(), "identifier".to_string()],
                ch.to_string(),
            ).into()),
            None => Err(JSONPathError::parse(
                "Unexpected end of input",
                self.position,
                self.input,
            ).into()),
        }
    }

    /// Parse dot notation selector (.property or ..)
    fn parse_dot_selector(&mut self) -> Result<Selector> {
        self.advance(); // consume '.'
        
        self.skip_whitespace();
        
        // Check for recursive descent (..)
        if self.current_char() == Some('.') {
            self.advance(); // consume second '.'
            return Ok(Selector::RecursiveDescent);
        }
        
        // Check for wildcard (.*)
        if self.current_char() == Some('*') {
            self.advance();
            return Ok(Selector::Wildcard);
        }
        
        // Parse property name
        let property = self.parse_identifier()?;
        Ok(Selector::Child(ChildSelector::Property(property)))
    }

    /// Parse bracket notation selector
    fn parse_bracket_selector(&mut self) -> Result<Selector> {
        self.advance(); // consume '['
        self.skip_whitespace();

        // Check for filter expression
        if self.current_char() == Some('?') {
            return self.parse_filter_selector();
        }

        // Check for quoted property
        if self.current_char() == Some('\'') || self.current_char() == Some('"') {
            let property = self.parse_quoted_string()?;
            self.skip_whitespace();
            self.expect_char(']')?;
            return Ok(Selector::Child(ChildSelector::QuotedProperty(property)));
        }

        // Parse numeric index or slice
        let start_pos = self.position;
        let mut has_colon = false;
        let mut union_parts = Vec::new();
        let mut current_part = String::new();

        while self.current_char() != Some(']') && !self.is_at_end() {
            match self.current_char() {
                Some(':') => {
                    has_colon = true;
                    current_part.push(':');
                    self.advance();
                }
                Some(',') => {
                    union_parts.push(current_part.trim().to_string());
                    current_part.clear();
                    self.advance();
                    self.skip_whitespace();
                }
                Some(ch) if ch.is_ascii_digit() || ch == '-' || ch.is_whitespace() => {
                    if !ch.is_whitespace() {
                        current_part.push(ch);
                    }
                    self.advance();
                }
                Some(ch) => {
                    return Err(JSONPathError::syntax(
                        "Invalid character in bracket selector",
                        self.position,
                        self.input,
                        vec!["digit".to_string(), ":".to_string(), ",".to_string(), "]".to_string()],
                        ch.to_string(),
                    ).into());
                }
                None => {
                    return Err(JSONPathError::parse(
                        "Unterminated bracket selector",
                        start_pos,
                        self.input,
                    ).into());
                }
            }
        }

        if !current_part.trim().is_empty() {
            union_parts.push(current_part.trim().to_string());
        }

        self.expect_char(']')?;

        // Handle union
        if union_parts.len() > 1 {
            let mut selectors = Vec::new();
            for part in union_parts {
                if part.contains(':') {
                    selectors.push(self.parse_slice_from_string(&part)?);
                } else {
                    selectors.push(self.parse_index_from_string(&part)?);
                }
            }
            return Ok(Selector::Union(UnionSelector { selectors }));
        }

        // Handle single selector
        let part = union_parts.into_iter().next().unwrap_or_default();
        if has_colon || part.contains(':') {
            Ok(self.parse_slice_from_string(&part)?)
        } else {
            Ok(self.parse_index_from_string(&part)?)
        }
    }

    /// Parse a filter selector [?(...)]
    fn parse_filter_selector(&mut self) -> Result<Selector> {
        self.advance(); // consume '?'
        self.skip_whitespace();

        let filter_expr = self.parse_filter_expression()?;
        
        self.skip_whitespace();
        self.expect_char(']')?;

        Ok(Selector::Filter(FilterSelector { filter: filter_expr }))
    }

    /// Parse a filter expression
    fn parse_filter_expression(&mut self) -> Result<FilterExpression> {
        self.parse_logical_or()
    }

    /// Parse logical OR expression
    fn parse_logical_or(&mut self) -> Result<FilterExpression> {
        let mut expr = self.parse_logical_and()?;

        while self.match_operator("||") {
            let right = self.parse_logical_and()?;
            expr = FilterExpression::Binary {
                left: Box::new(expr),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parse logical AND expression
    fn parse_logical_and(&mut self) -> Result<FilterExpression> {
        let mut expr = self.parse_equality()?;

        while self.match_operator("&&") {
            let right = self.parse_equality()?;
            expr = FilterExpression::Binary {
                left: Box::new(expr),
                operator: BinaryOperator::And,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Parse equality/inequality expressions
    fn parse_equality(&mut self) -> Result<FilterExpression> {
        let mut expr = self.parse_comparison()?;

        loop {
            self.skip_whitespace();
            if self.match_operator("==") {
                let right = self.parse_comparison()?;
                expr = FilterExpression::Binary {
                    left: Box::new(expr),
                    operator: BinaryOperator::Equal,
                    right: Box::new(right),
                };
            } else if self.match_operator("!=") {
                let right = self.parse_comparison()?;
                expr = FilterExpression::Binary {
                    left: Box::new(expr),
                    operator: BinaryOperator::NotEqual,
                    right: Box::new(right),
                };
            } else if self.match_operator("=~") {
                let right = self.parse_comparison()?;
                expr = FilterExpression::Binary {
                    left: Box::new(expr),
                    operator: BinaryOperator::RegexMatch,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse comparison expressions
    fn parse_comparison(&mut self) -> Result<FilterExpression> {
        let mut expr = self.parse_unary()?;

        loop {
            self.skip_whitespace();
            if self.match_operator("<=") {
                let right = self.parse_unary()?;
                expr = FilterExpression::Binary {
                    left: Box::new(expr),
                    operator: BinaryOperator::LessThanOrEqual,
                    right: Box::new(right),
                };
            } else if self.match_operator(">=") {
                let right = self.parse_unary()?;
                expr = FilterExpression::Binary {
                    left: Box::new(expr),
                    operator: BinaryOperator::GreaterThanOrEqual,
                    right: Box::new(right),
                };
            } else if self.match_operator("<") {
                let right = self.parse_unary()?;
                expr = FilterExpression::Binary {
                    left: Box::new(expr),
                    operator: BinaryOperator::LessThan,
                    right: Box::new(right),
                };
            } else if self.match_operator(">") {
                let right = self.parse_unary()?;
                expr = FilterExpression::Binary {
                    left: Box::new(expr),
                    operator: BinaryOperator::GreaterThan,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse unary expressions
    fn parse_unary(&mut self) -> Result<FilterExpression> {
        self.skip_whitespace();
        
        if self.current_char() == Some('!') {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(FilterExpression::Unary {
                operator: UnaryOperator::Not,
                operand: Box::new(operand),
            });
        }
        
        if self.current_char() == Some('-') && 
           self.peek_char().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            self.advance();
            let operand = self.parse_primary()?;
            return Ok(FilterExpression::Unary {
                operator: UnaryOperator::Negate,
                operand: Box::new(operand),
            });
        }

        self.parse_primary()
    }

    /// Parse primary expressions
    fn parse_primary(&mut self) -> Result<FilterExpression> {
        self.skip_whitespace();

        match self.current_char() {
            Some('(') => {
                self.advance();
                let expr = self.parse_filter_expression()?;
                self.skip_whitespace();
                self.expect_char(')')?;
                Ok(expr)
            }
            Some('@') => {
                self.advance();
                self.parse_path_expression(FilterExpression::Current)
            }
            Some('$') => {
                self.advance();
                self.parse_path_expression(FilterExpression::Root)
            }
            Some('\'') | Some('"') => {
                let value = self.parse_quoted_string()?;
                Ok(FilterExpression::Literal(FilterLiteral::String(value)))
            }
            Some(ch) if ch.is_ascii_digit() => {
                let number = self.parse_number()?;
                Ok(FilterExpression::Literal(FilterLiteral::Number(number)))
            }
            Some(ch) if ch.is_alphabetic() => {
                let ident = self.parse_identifier()?;
                match ident.as_str() {
                    "true" => Ok(FilterExpression::Literal(FilterLiteral::Boolean(true))),
                    "false" => Ok(FilterExpression::Literal(FilterLiteral::Boolean(false))),
                    "null" => Ok(FilterExpression::Literal(FilterLiteral::Null)),
                    _ => {
                        // Check if it's a function call
                        self.skip_whitespace();
                        if self.current_char() == Some('(') {
                            self.advance();
                            let args = self.parse_function_args()?;
                            self.expect_char(')')?;
                            Ok(FilterExpression::Function { name: ident, args })
                        } else {
                            Err(JSONPathError::parse(
                                format!("Unexpected identifier: {}", ident),
                                self.position,
                                self.input,
                            ).into())
                        }
                    }
                }
            }
            Some(ch) => Err(JSONPathError::syntax(
                "Unexpected character in filter expression",
                self.position,
                self.input,
                vec!["@".to_string(), "$".to_string(), "'".to_string(), "\"".to_string(), "digit".to_string(), "identifier".to_string()],
                ch.to_string(),
            ).into()),
            None => Err(JSONPathError::parse(
                "Unexpected end of input in filter expression",
                self.position,
                self.input,
            ).into()),
        }
    }

    /// Parse path expression (property access, indexing)
    fn parse_path_expression(&mut self, base: FilterExpression) -> Result<FilterExpression> {
        let mut expr = base;

        loop {
            self.skip_whitespace();
            match self.current_char() {
                Some('.') => {
                    self.advance();
                    let property = self.parse_identifier()?;
                    expr = FilterExpression::Property {
                        base: Box::new(expr),
                        property,
                    };
                }
                Some('[') => {
                    self.advance();
                    let index = self.parse_number()? as i64;
                    self.expect_char(']')?;
                    expr = FilterExpression::Index {
                        base: Box::new(expr),
                        index,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Parse function arguments
    fn parse_function_args(&mut self) -> Result<Vec<FilterExpression>> {
        let mut args = Vec::new();
        
        self.skip_whitespace();
        if self.current_char() == Some(')') {
            return Ok(args);
        }

        loop {
            args.push(self.parse_filter_expression()?);
            self.skip_whitespace();
            
            if self.current_char() == Some(',') {
                self.advance();
                self.skip_whitespace();
            } else {
                break;
            }
        }

        Ok(args)
    }

    /// Parse slice from string representation
    fn parse_slice_from_string(&self, s: &str) -> Result<Selector> {
        let parts: Vec<&str> = s.split(':').collect();
        
        let start = if parts[0].is_empty() { 
            None 
        } else { 
            Some(parts[0].parse().map_err(|_| {
                JSONPathError::parse(
                    format!("Invalid slice start: {}", parts[0]),
                    self.position,
                    self.input,
                )
            })?)
        };
        
        let end = if parts.len() > 1 && !parts[1].is_empty() { 
            Some(parts[1].parse().map_err(|_| {
                JSONPathError::parse(
                    format!("Invalid slice end: {}", parts[1]),
                    self.position,
                    self.input,
                )
            })?)
        } else { 
            None 
        };
        
        let step = if parts.len() > 2 && !parts[2].is_empty() { 
            parts[2].parse().map_err(|_| {
                JSONPathError::parse(
                    format!("Invalid slice step: {}", parts[2]),
                    self.position,
                    self.input,
                )
            })?
        } else { 
            1 
        };

        Ok(Selector::Slice(SliceSelector { start, end, step }))
    }

    /// Parse index from string representation
    fn parse_index_from_string(&self, s: &str) -> Result<Selector> {
        let index: i64 = s.parse().map_err(|_| {
            JSONPathError::parse(
                format!("Invalid array index: {}", s),
                self.position,
                self.input,
            )
        })?;

        let index_selector = if index < 0 {
            IndexSelector::Negative((-index) as usize)
        } else {
            IndexSelector::Positive(index as usize)
        };

        Ok(Selector::Index(index_selector))
    }

    /// Parse an identifier
    fn parse_identifier(&mut self) -> Result<String> {
        let mut identifier = String::new();
        
        if !self.current_char().map(|c| c.is_alphabetic() || c == '_').unwrap_or(false) {
            return Err(JSONPathError::syntax(
                "Expected identifier",
                self.position,
                self.input,
                vec!["letter or _".to_string()],
                self.current_char().map(|c| c.to_string()).unwrap_or_else(|| "EOF".to_string()),
            ).into());
        }

        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        Ok(identifier)
    }

    /// Parse a quoted string
    fn parse_quoted_string(&mut self) -> Result<String> {
        let quote_char = self.current_char().unwrap();
        self.advance(); // consume opening quote

        let mut string = String::new();
        let mut escaped = false;

        while let Some(ch) = self.current_char() {
            if escaped {
                match ch {
                    'n' => string.push('\n'),
                    'r' => string.push('\r'),
                    't' => string.push('\t'),
                    '\\' => string.push('\\'),
                    '\'' => string.push('\''),
                    '"' => string.push('"'),
                    _ => {
                        string.push('\\');
                        string.push(ch);
                    }
                }
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote_char {
                self.advance(); // consume closing quote
                return Ok(string);
            } else {
                string.push(ch);
            }
            self.advance();
        }

        Err(JSONPathError::parse(
            "Unterminated string literal",
            self.position,
            self.input,
        ).into())
    }

    /// Parse a number
    fn parse_number(&mut self) -> Result<f64> {
        let mut number_str = String::new();
        
        // Handle negative sign
        if self.current_char() == Some('-') {
            number_str.push('-');
            self.advance();
        }

        // Parse integer part
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() {
                number_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Parse decimal part
        if self.current_char() == Some('.') {
            number_str.push('.');
            self.advance();
            
            while let Some(ch) = self.current_char() {
                if ch.is_ascii_digit() {
                    number_str.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Parse exponent part
        if let Some(ch) = self.current_char() {
            if ch == 'e' || ch == 'E' {
                number_str.push(ch);
                self.advance();
                
                if let Some(sign) = self.current_char() {
                    if sign == '+' || sign == '-' {
                        number_str.push(sign);
                        self.advance();
                    }
                }
                
                while let Some(ch) = self.current_char() {
                    if ch.is_ascii_digit() {
                        number_str.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
        }

        number_str.parse().map_err(|_| {
            JSONPathError::parse(
                format!("Invalid number: {}", number_str),
                self.position,
                self.input,
            ).into()
        })
    }

    /// Match and consume an operator
    fn match_operator(&mut self, op: &str) -> bool {
        let remaining: String = self.chars.clone().take(op.len()).collect();
        if remaining == op {
            for _ in 0..op.len() {
                self.advance();
            }
            true
        } else {
            false
        }
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Get current character without advancing
    fn current_char(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// Get next character without advancing
    fn peek_char(&mut self) -> Option<char> {
        let mut clone = self.chars.clone();
        clone.next(); // skip current
        clone.peek().copied()
    }

    /// Advance to next character
    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.chars.next() {
            self.position += ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    /// Check if at end of input
    fn is_at_end(&mut self) -> bool {
        self.current_char().is_none()
    }

    /// Expect a specific character
    fn expect_char(&mut self, expected: char) -> Result<()> {
        match self.current_char() {
            Some(ch) if ch == expected => {
                self.advance();
                Ok(())
            }
            Some(ch) => Err(JSONPathError::syntax(
                format!("Expected '{}'", expected),
                self.position,
                self.input,
                vec![expected.to_string()],
                ch.to_string(),
            ).into()),
            None => Err(JSONPathError::parse(
                format!("Expected '{}' but reached end of input", expected),
                self.position,
                self.input,
            ).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_root() {
        let parser = Parser::new("$").unwrap();
        let expr = parser.parse().unwrap();
        assert!(matches!(expr.root, Selector::Root));
        assert!(expr.selectors.is_empty());
    }

    #[test]
    fn test_parse_property() {
        let parser = Parser::new("$.store").unwrap();
        let expr = parser.parse().unwrap();
        assert!(matches!(expr.root, Selector::Root));
        assert_eq!(expr.selectors.len(), 1);
        assert!(matches!(expr.selectors[0], Selector::Child(ChildSelector::Property(_))));
    }

    #[test]
    fn test_parse_array_index() {
        let parser = Parser::new("$.books[0]").unwrap();
        let expr = parser.parse().unwrap();
        assert_eq!(expr.selectors.len(), 2);
        assert!(matches!(expr.selectors[1], Selector::Index(IndexSelector::Positive(0))));
    }

    #[test]
    fn test_parse_negative_index() {
        let parser = Parser::new("$.books[-1]").unwrap();
        let expr = parser.parse().unwrap();
        assert!(matches!(expr.selectors[1], Selector::Index(IndexSelector::Negative(1))));
    }

    #[test]
    fn test_parse_slice() {
        let parser = Parser::new("$.books[1:3]").unwrap();
        let expr = parser.parse().unwrap();
        assert!(matches!(expr.selectors[1], Selector::Slice(_)));
    }

    #[test]
    fn test_parse_wildcard() {
        let parser = Parser::new("$.store.*").unwrap();
        let expr = parser.parse().unwrap();
        assert!(matches!(expr.selectors[1], Selector::Wildcard));
    }

    #[test]
    fn test_parse_recursive_descent() {
        let parser = Parser::new("$..author").unwrap();
        let expr = parser.parse().unwrap();
        assert!(matches!(expr.selectors[0], Selector::RecursiveDescent));
        assert!(matches!(expr.selectors[1], Selector::Child(_)));
    }

    #[test]
    fn test_parse_bracket_notation() {
        let parser = Parser::new("$['store']['book']").unwrap();
        let expr = parser.parse().unwrap();
        assert_eq!(expr.selectors.len(), 2);
        assert!(matches!(expr.selectors[0], Selector::Child(ChildSelector::QuotedProperty(_))));
    }

    #[test]
    fn test_parse_filter() {
        let parser = Parser::new("$.books[?(@.price < 10)]").unwrap();
        let expr = parser.parse().unwrap();
        assert!(matches!(expr.selectors[1], Selector::Filter(_)));
    }

    #[test]
    fn test_parse_union() {
        let parser = Parser::new("$.books[0,1]").unwrap();
        let expr = parser.parse().unwrap();
        assert!(matches!(expr.selectors[1], Selector::Union(_)));
    }

    #[test]
    fn test_parse_complex_filter() {
        let parser = Parser::new("$.books[?(@.price > 5 && @.author == 'John')]").unwrap();
        let expr = parser.parse().unwrap();
        if let Selector::Filter(FilterSelector { filter }) = &expr.selectors[1] {
            assert!(matches!(filter, FilterExpression::Binary { .. }));
        } else {
            panic!("Expected filter selector");
        }
    }

    #[test]
    fn test_parse_error_empty_input() {
        let result = Parser::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_invalid_start() {
        let parser = Parser::new("invalid").unwrap();
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_unterminated_bracket() {
        let parser = Parser::new("$.test[").unwrap();
        let result = parser.parse();
        assert!(result.is_err());
    }
}