//! Macro compiler.

use crate::error::CompileResult;
use crate::{
    ast, CompileError, MacroContext, Options, Parse, ParseError, Parser, TokenStream, UnitBuilder,
};
use runestick::{Context, Hash, Item, Source, Span};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub(crate) struct MacroCompiler<'a> {
    pub(crate) item: Item,
    pub(crate) macro_context: &'a mut MacroContext,
    pub(crate) options: &'a Options,
    pub(crate) context: &'a Context,
    pub(crate) unit: Rc<RefCell<UnitBuilder>>,
    pub(crate) source: Arc<Source>,
}

impl MacroCompiler<'_> {
    /// Compile the given macro into the given output type.
    pub(crate) fn eval_macro<T>(&mut self, expr_call_macro: ast::ExprCallMacro) -> CompileResult<T>
    where
        T: Parse,
    {
        let span = expr_call_macro.span();

        if !self.options.macros {
            return Err(CompileError::experimental(
                "macros must be enabled with `-O macros=true`",
                span,
            ));
        }

        let item =
            self.unit
                .borrow()
                .convert_path(&self.item, &expr_call_macro.path, &*self.source)?;
        let hash = Hash::type_hash(&item);

        let handler = match self.context.lookup_macro(hash) {
            Some(handler) => handler,
            None => {
                return Err(CompileError::MissingMacro { span, item });
            }
        };

        let input_stream = &expr_call_macro.stream;

        self.macro_context.default_span = span;
        self.macro_context.end = Span::point(span.end);

        let result = handler(self.macro_context, input_stream);

        // reset to default spans.
        self.macro_context.default_span = Span::default();
        self.macro_context.end = Span::default();

        let output = match result {
            Ok(output) => output,
            Err(error) => {
                return match error.downcast::<ParseError>() {
                    Ok(error) => Err(CompileError::ParseError { error }),
                    Err(error) => Err(CompileError::CallMacroError { span, error }),
                };
            }
        };

        let token_stream = match output.downcast::<TokenStream>() {
            Ok(token_stream) => *token_stream,
            Err(..) => {
                return Err(CompileError::CallMacroError {
                    span,
                    error: runestick::Error::msg(format!(
                        "failed to downcast macro result, expected `{}`",
                        std::any::type_name::<TokenStream>()
                    )),
                });
            }
        };

        let mut parser = Parser::from_token_stream(&token_stream);
        let output = parser.parse::<T>()?;
        parser.parse_eof()?;
        Ok(output)
    }
}
