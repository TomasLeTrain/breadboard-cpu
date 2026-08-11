use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::sync::Arc;

use miette::{Diagnostic, NamedSource, SourceSpan};
use pest::RuleType;

use pest::error::Error as PestParseError;
use pest::error::ErrorVariant as PestParseErrorVariant;

use crate::ast::AstSpan;
use crate::ast::Source;

// TODO: make error code generic to distinguish grammar parsing vs. ast parsing
#[derive(Debug, Diagnostic)]
#[diagnostic(code(assembler::parse_error))]
pub struct ParseError {
    #[source_code]
    pub source: NamedSource<Arc<str>>,
    #[label]
    pub snippet: Option<SourceSpan>,
    pub err_message: String,
    #[help]
    pub help: Option<String>,
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.err_message)
    }
}

impl ParseError {
    pub fn from_span(message: impl Into<String>, span: &AstSpan) -> Self {
        let span_start = span.start();
        let span_end = span.end();
        let len = span_end - span_start;
        let snippet = Some(SourceSpan::new(span_start.into(), len));

        ParseError {
            err_message: message.into(),
            snippet,
            help: None,
            source: span.to_miette_source_code(),
        }
    }

    /// creates error message with a help message detailing expected/unexpected rules
    pub fn from_expected<R: RuleType>(
        message: impl Into<String>,
        expected: Vec<R>,
        unexpected: Vec<R>,
        span: &AstSpan,
    ) -> Self {
        Self::from_pest_message(
            PestParseError::new_from_span(
                PestParseErrorVariant::ParsingError {
                    positives: expected,
                    negatives: unexpected,
                },
                span.to_span(),
            ),
            message,
            span.source(),
        )
    }

    pub fn from_pest_message<R: RuleType>(
        err: PestParseError<R>,
        err_message: impl Into<String>,
        source: &Source,
    ) -> ParseError {
        let help = Some(err.variant.message().to_string());

        let span = match err.location {
            pest::error::InputLocation::Pos(pos) => (pos, pos + 1),
            pest::error::InputLocation::Span((start, end)) => (start, end),
        };

        let snippet = Some(SourceSpan::new(span.0.into(), span.1 - span.0));
        println!("span {:?}", span);

        ParseError {
            err_message: err_message.into(),
            snippet,
            help,
            source: AstSpan::new(span.0, span.1, Arc::clone(source)).to_miette_source_code(),
        }
    }

    pub fn from_pest<R: RuleType>(err: PestParseError<R>, source: &Source) -> ParseError {
        let message = "Grammar parsing error".to_string();
        Self::from_pest_message(err, message, source)
    }
}
