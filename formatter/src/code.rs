use crate::{
    config::{FormattingConfig, FunctionLineBreaks},
    format::DocAlgebra,
};

use parser::ast::{Arg, Args, Delimiter, Expression, IfConditional, TermExpr};
use tokenizer::tokens::CommentedToken;

use crate::format::{
    query_inline_position, CommonProperties, Doc, GroupDocProperties, InlineCommentPosition,
    ShouldBreak,
};
use std::rc::Rc;
use tokenizer::Token;

pub(crate) trait Code {
    fn to_docs(&self, config: &impl FormattingConfig, doc_ref: &mut usize) -> Rc<Doc>;
}

impl<T> Code for Option<T>
where
    T: Code,
{
    fn to_docs(&self, config: &impl FormattingConfig, doc_ref: &mut usize) -> Rc<Doc> {
        match self {
            Some(inner) => inner.to_docs(config, doc_ref),
            None => text!(""),
        }
    }
}

// Macro that creates a Doc::Group
macro_rules! group {
    ($doc:expr, $should_break:expr, $doc_ref:expr) => {{
        let doc: Rc<Doc> = $doc;
        let should_break: ShouldBreak = $should_break;
        let doc_ref: usize = $doc_ref;
        let properties = CommonProperties(query_inline_position(&doc), doc_ref);
        Rc::new(Doc::Group(
            GroupDocProperties(doc, should_break),
            properties,
        ))
    }};
}
pub(crate) use group;

// Macro that creates a Doc::Break
macro_rules! nl {
    ($txt:expr) => {
        Rc::new(Doc::Break($txt))
    };
}
pub(crate) use nl;

// Macro that creates a Doc::Text
macro_rules! text {
    ($txt:expr) => {{
        let txt: &str = $txt;
        Rc::new(Doc::Text(
            Rc::from(txt),
            txt.len(),
            CommonProperties(InlineCommentPosition::No, 0),
        ))
    }};
    ($txt:expr, $size:expr) => {{
        let txt: &str = $txt;
        let size: usize = $size;
        Rc::new(Doc::Text(
            Rc::from(txt),
            size,
            CommonProperties(InlineCommentPosition::No, 0),
        ))
    }};
    ($txt:expr, $size:expr, $comment_position:expr) => {{
        let txt: &str = $txt;
        let size: usize = $size;
        let position: InlineCommentPosition = $comment_position;
        Rc::new(Doc::Text(
            Rc::from(txt),
            size,
            CommonProperties(position, 0),
        ))
    }};
}
pub(crate) use text;

impl<'a> Code for Token<'a> {
    fn to_docs(&self, _: &impl FormattingConfig, _: &mut usize) -> Rc<Doc> {
        match self {
            Token::Symbol(s) | Token::Literal(s) => text!(*s),
            Token::Semicolon => text!(";"),
            Token::Newline => text!("\n"),
            Token::LParen => text!("("),
            Token::RParen => text!(")"),
            Token::LBrace => text!("{"),
            Token::RBrace => text!("}"),
            Token::LBracket => text!("["),
            Token::RBracket => text!("]"),
            Token::Comma => text!(","),
            Token::Continue => text!("continue"),
            Token::Break => text!("break"),
            Token::Stop => text!("stop"),
            Token::If => text!("if"),
            Token::Else => text!("else"),
            Token::While => text!("while"),
            Token::For => text!("for"),
            Token::Repeat => text!("repeat"),
            Token::In => text!("in"),
            Token::Function => text!("function"),
            Token::Lambda => text!("\\"),
            Token::LAssign => text!("<-"),
            Token::RAssign => text!("->"),
            Token::OldAssign => text!("="),
            Token::Equal => text!("=="),
            Token::NotEqual => text!("!="),
            Token::LowerThan => text!("<"),
            Token::GreaterThan => text!(">"),
            Token::LowerEqual => text!("<="),
            Token::GreaterEqual => text!(">="),
            Token::Power => text!("^"),
            Token::Divide => text!("/"),
            Token::Multiply => text!("*"),
            Token::Minus => text!("-"),
            Token::Plus => text!("+"),
            Token::Help => text!("?"),
            Token::And => text!("&&"),
            Token::VectorizedAnd => text!("&"),
            Token::Or => text!("||"),
            Token::VectorizedOr => text!("|"),
            Token::Dollar => text!("$"),
            Token::Pipe => text!("|>"),
            Token::Modulo => text!("%"),
            Token::NsGet => text!("::"),
            Token::NsGetInt => text!(":::"),
            Token::Tilde => text!("~"),
            Token::Colon => text!(":"),
            Token::Slot => text!("@"),
            Token::Special(s) => text!(*s),
            Token::UnaryNot => text!("!"),
            Token::InlineComment(s) => text!(*s, 0),
            Token::Comment(s) => text!(*s),
            Token::EOF => text!(""),
        }
    }
}

impl Code for CommentedToken<'_> {
    fn to_docs(&self, config: &impl FormattingConfig, doc_ref: &mut usize) -> Rc<Doc> {
        match (&self.leading_comments, self.inline_comment) {
            (None, None) => self.token.to_docs(config, doc_ref),
            (None, Some(inline_comment)) => self
                .token
                .to_docs(config, doc_ref)
                .cons(text!(" "))
                .cons(text!(inline_comment, 0, InlineCommentPosition::End)),
            (Some(leading_comments), None) => {
                let mut leading_comments_it = leading_comments.iter();
                let mut leading_comments = text!(leading_comments_it.next().unwrap());
                for comment in leading_comments_it {
                    leading_comments = leading_comments.cons(nl!("")).cons(text!(comment, 0));
                }
                let leading_comments = leading_comments
                    .nest_hanging()
                    .to_group(ShouldBreak::Yes, &mut 0);

                leading_comments
                    .cons(nl!(""))
                    .cons(
                        self.token
                            .to_docs(config, doc_ref)
                            .to_group(ShouldBreak::No, &mut 0),
                    )
                    .to_group(ShouldBreak::Yes, &mut 0)
            }
            (Some(leading_comments), Some(inline_comment)) => {
                let mut leading_comments_it = leading_comments.iter();
                let mut leading_comments = text!(leading_comments_it.next().unwrap());
                for comment in leading_comments_it {
                    leading_comments = leading_comments.cons(nl!("")).cons(text!(comment, 0));
                }
                let leading_comments = leading_comments
                    .nest_hanging()
                    .to_group(ShouldBreak::Yes, &mut 0);

                leading_comments
                    .cons(nl!(""))
                    .cons(
                        self.token
                            .to_docs(config, doc_ref)
                            .cons(text!(" "))
                            .cons(text!(inline_comment, 0, InlineCommentPosition::End))
                            .to_group(ShouldBreak::No, &mut 0),
                    )
                    .to_group(ShouldBreak::Yes, &mut 0)
            }
        }
    }
}

impl Code for Delimiter<'_> {
    fn to_docs(&self, config: &impl FormattingConfig, doc_ref: &mut usize) -> Rc<Doc> {
        match self {
            Delimiter::Paren(single) | Delimiter::SingleBracket(single) => {
                single.to_docs(config, doc_ref)
            }
            Delimiter::DoubleBracket((b1, b2)) => b1
                .to_docs(config, doc_ref)
                .cons(b2.to_docs(config, doc_ref)),
        }
    }
}

/// Returns a Doc::Group
fn join_docs<I, F>(
    docs: I,
    separator: Rc<Doc>,
    should_break: ShouldBreak,
    _config: &F,
    doc_ref: &mut usize,
) -> Rc<Doc>
where
    I: IntoIterator<Item = Rc<Doc>>,
    F: FormattingConfig,
{
    join_docs_ungroupped(docs, separator, _config).to_group(should_break, doc_ref)
}

/// Returns a Doc::Cons
fn join_docs_ungroupped<I, F>(docs: I, separator: Rc<Doc>, _config: &F) -> Rc<Doc>
where
    I: IntoIterator<Item = Rc<Doc>>,
    F: FormattingConfig,
{
    let mut docs = docs.into_iter();
    let mut res = Rc::new(Doc::Nil);

    if let Some(first_doc) = docs.next() {
        if !matches!(*first_doc, Doc::Nil) {
            res = res.cons(first_doc);
        }
    }

    for next_doc in docs {
        if !matches!(*next_doc, Doc::Nil) {
            res = res.cons(separator.clone()).cons(nl!(" ")).cons(next_doc);
        }
    }

    res
}

impl<'a> Code for Expression<'a> {
    fn to_docs(&self, config: &impl FormattingConfig, doc_ref: &mut usize) -> Rc<Doc> {
        match self {
            Expression::Symbol(token)
            | Expression::Literal(token)
            | Expression::Comment(token)
            | Expression::Continue(token)
            | Expression::Break(token) => token.to_docs(config, doc_ref),
            Expression::Term(term_expr) => match &**term_expr {
                // Case for the embracing operator
                TermExpr {
                    pre_delimiters: Some(pre_delim),
                    term,
                    post_delimiters: Some(post_delim),
                } if config.embracing_op_no_nl()
                    && matches!(pre_delim.token, Token::LBrace)
                    && term.len() == 1
                    && matches!(term[0], Expression::Term { .. }) =>
                {
                    match &term[0] {
                        Expression::Term(inner_term_expr) => {
                            if inner_term_expr
                                .pre_delimiters
                                .is_some_and(|delim| matches!(delim.token, Token::LBrace))
                            {
                                let inner_docs: Vec<_> = inner_term_expr
                                    .term
                                    .iter()
                                    .map(|t| t.to_docs(config, doc_ref))
                                    .collect();
                                let inner_docs = join_docs(
                                    inner_docs,
                                    Rc::new(Doc::Nil),
                                    ShouldBreak::No,
                                    config,
                                    doc_ref,
                                );
                                pre_delim
                                    .to_docs(config, doc_ref)
                                    .cons(
                                        inner_term_expr
                                            .pre_delimiters
                                            .as_ref()
                                            .unwrap()
                                            .to_docs(config, doc_ref),
                                    )
                                    .cons(text!(" "))
                                    .cons(inner_docs)
                                    .cons(text!(" "))
                                    .cons(
                                        inner_term_expr
                                            .post_delimiters
                                            .as_ref()
                                            .unwrap()
                                            .to_docs(config, doc_ref),
                                    )
                                    .cons(post_delim.to_docs(config, doc_ref))
                                    .to_group(ShouldBreak::No, doc_ref)
                            } else {
                                let docs: Vec<_> =
                                    term.iter().map(|t| t.to_docs(config, doc_ref)).collect();
                                let inner = join_docs(
                                    docs,
                                    Rc::new(Doc::Nil),
                                    ShouldBreak::No,
                                    config,
                                    doc_ref,
                                );
                                pre_delim
                                    .to_docs(config, doc_ref)
                                    .cons(nl!(" ").cons(inner).nest(config.indent()))
                                    .cons(nl!(" "))
                                    .cons(post_delim.to_docs(config, doc_ref))
                                    .to_group(ShouldBreak::Yes, doc_ref)
                            }
                        }
                        _ => unreachable!("Already checked that term[0] is a Term"),
                    }
                }
                TermExpr {
                    pre_delimiters: Some(pre_delim),
                    term,
                    post_delimiters: Some(post_delim),
                } if matches!(pre_delim.token, Token::LBrace) => {
                    if term.is_empty() {
                        pre_delim
                            .to_docs(config, doc_ref)
                            .cons(nl!(""))
                            .nest(config.indent())
                            .cons(post_delim.to_docs(config, doc_ref))
                            .to_group(ShouldBreak::No, &mut 0)
                    } else {
                        let docs = term
                            .iter()
                            .map(|t| {
                                t.to_docs(config, doc_ref)
                                    .to_group(ShouldBreak::No, doc_ref)
                            })
                            .collect::<Vec<_>>();
                        let inner =
                            join_docs(docs, Rc::new(Doc::Nil), ShouldBreak::Yes, config, doc_ref);
                        pre_delim
                            .to_docs(config, doc_ref)
                            .cons(nl!(" ").cons(inner).nest(config.indent()))
                            .cons(nl!(" "))
                            .cons(post_delim.to_docs(config, doc_ref))
                            .to_group(ShouldBreak::Yes, doc_ref)
                    }
                }
                TermExpr {
                    pre_delimiters: None,
                    term,
                    post_delimiters: None,
                } => {
                    let docs = term
                        .iter()
                        .map(|t| {
                            t.to_docs(config, doc_ref)
                                .to_group(ShouldBreak::No, doc_ref)
                        })
                        .collect::<Vec<_>>();
                    join_docs(docs, Rc::new(Doc::Nil), ShouldBreak::Yes, config, doc_ref)
                }
                TermExpr {
                    pre_delimiters: Some(pre_delim),
                    term,
                    post_delimiters: Some(post_delim),
                } => {
                    if term.is_empty() {
                        pre_delim
                            .to_docs(config, doc_ref)
                            .cons(post_delim.to_docs(config, doc_ref))
                    } else {
                        let docs = term
                            .iter()
                            .map(|t| t.to_docs(config, doc_ref))
                            .collect::<Vec<_>>();
                        let inner =
                            join_docs(docs, Rc::new(Doc::Nil), ShouldBreak::No, config, doc_ref);
                        pre_delim
                            .to_docs(config, doc_ref)
                            .cons(nl!("").cons(inner).nest(config.indent()))
                            .cons(nl!(""))
                            .cons(post_delim.to_docs(config, doc_ref))
                            .to_group(ShouldBreak::No, doc_ref)
                    }
                }
                _ => panic!("Term with not matching delimiters found"),
            },
            Expression::Unary(op, expr) => op
                .to_docs(config, doc_ref)
                .cons(expr.to_docs(config, doc_ref)),
            Expression::Bop(op, lhs, rhs) => match op.token {
                Token::OldAssign | Token::LAssign if !config.allow_nl_after_assignment() => lhs
                    .to_docs(config, doc_ref)
                    .cons(text!(" "))
                    .cons(op.to_docs(config, doc_ref))
                    .cons(text!(" ").cons(rhs.to_docs(config, doc_ref)))
                    .to_group(ShouldBreak::No, doc_ref),
                Token::LAssign
                | Token::RAssign
                | Token::OldAssign
                | Token::Equal
                | Token::NotEqual
                | Token::LowerThan
                | Token::GreaterThan
                | Token::LowerEqual
                | Token::GreaterEqual
                | Token::Divide
                | Token::Multiply
                | Token::Minus
                | Token::Plus
                | Token::And
                | Token::VectorizedAnd
                | Token::Or
                | Token::VectorizedOr
                | Token::Pipe
                | Token::Modulo
                | Token::Tilde
                | Token::Special(_) => lhs
                    .to_docs(config, doc_ref)
                    .cons(text!(" "))
                    .cons(op.to_docs(config, doc_ref))
                    .cons(
                        nl!(" ")
                            .cons(rhs.to_docs(config, doc_ref))
                            .nest(config.indent()),
                    ),
                // .to_group(ShouldBreak::No, doc_ref),
                Token::Dollar
                | Token::NsGet
                | Token::NsGetInt
                | Token::Colon
                | Token::Slot
                | Token::Power
                | Token::Help => lhs
                    .to_docs(config, doc_ref)
                    .cons(op.to_docs(config, doc_ref))
                    .cons(rhs.to_docs(config, doc_ref))
                    .to_group(ShouldBreak::No, doc_ref),
                _ => panic!(
                    "Got a not a binary operator token inside a binary expression when \
                     formatting. Token: {:?}",
                    &op.token
                ),
            },
            Expression::Formula(tilde, term) => tilde
                .to_docs(config, doc_ref)
                .cons(if matches!(**term, Expression::Symbol(_)) {
                    text!("")
                } else {
                    text!(" ")
                })
                .cons(term.to_docs(config, doc_ref)),
            Expression::Newline(_) => Rc::new(Doc::Break("\n")),
            Expression::EOF(eof) => eof.to_docs(config, doc_ref),
            Expression::Whitespace(_) => text!(""),
            Expression::FunctionDef(function_def) => {
                let (keyword, args, body) = (
                    function_def.keyword,
                    &function_def.arguments,
                    &function_def.body,
                );
                match config.function_line_breaks() {
                    FunctionLineBreaks::Hanging => {
                        let args_doc = join_docs_ungroupped(
                            args.args.iter().map(|arg| {
                                arg.0
                                    .to_docs(config, doc_ref)
                                    .cons(
                                        arg.1
                                            .as_ref()
                                            .map(|sep| sep.to_docs(config, doc_ref))
                                            .unwrap_or(Rc::new(Doc::Nil)),
                                    )
                                    .to_group(ShouldBreak::No, doc_ref)
                            }),
                            Rc::new(Doc::Nil),
                            config,
                        );
                        let args_group = args
                            .left_delimeter
                            .to_docs(config, doc_ref)
                            .cons(args_doc.nest_hanging())
                            .cons(args.right_delimeter.to_docs(config, doc_ref))
                            .to_group(ShouldBreak::No, doc_ref);
                        keyword
                            .to_docs(config, doc_ref)
                            .cons(args_group)
                            .cons(text!(" "))
                            .cons(body.to_docs(config, doc_ref))
                            .to_group(ShouldBreak::No, doc_ref)
                    }
                    FunctionLineBreaks::Double => {
                        let args_doc = join_docs_ungroupped(
                            args.args.iter().map(|arg| {
                                arg.0
                                    .to_docs(config, doc_ref)
                                    .cons(
                                        arg.1
                                            .as_ref()
                                            .map(|sep| sep.to_docs(config, doc_ref))
                                            .unwrap_or(Rc::new(Doc::Nil)),
                                    )
                                    .to_group(ShouldBreak::No, doc_ref)
                            }),
                            Rc::new(Doc::Nil),
                            config,
                        );
                        let args_group = args
                            .left_delimeter
                            .to_docs(config, doc_ref)
                            .cons(nl!(""))
                            .cons(args_doc)
                            .nest(2 * config.indent())
                            .cons(nl!(""))
                            .cons(args.right_delimeter.to_docs(config, doc_ref))
                            .to_group(ShouldBreak::No, doc_ref);
                        keyword
                            .to_docs(config, doc_ref)
                            .cons(args_group)
                            .cons(text!(" "))
                            .cons(body.to_docs(config, doc_ref))
                            .to_group(ShouldBreak::No, doc_ref)
                    }
                    FunctionLineBreaks::Single => {
                        let args_doc = join_docs_ungroupped(
                            args.args.iter().map(|arg| {
                                arg.0
                                    .to_docs(config, doc_ref)
                                    .cons(
                                        arg.1
                                            .as_ref()
                                            .map(|sep| sep.to_docs(config, doc_ref))
                                            .unwrap_or(Rc::new(Doc::Nil)),
                                    )
                                    .to_group(ShouldBreak::No, doc_ref)
                            }),
                            Rc::new(Doc::Nil),
                            config,
                        );
                        let args_group = args
                            .left_delimeter
                            .to_docs(config, doc_ref)
                            .cons(nl!(""))
                            .cons(args_doc)
                            .nest(config.indent())
                            .cons(nl!(""))
                            .cons(args.right_delimeter.to_docs(config, doc_ref))
                            .to_group(ShouldBreak::No, doc_ref);
                        keyword
                            .to_docs(config, doc_ref)
                            .cons(args_group)
                            .cons(text!(" "))
                            .cons(body.to_docs(config, doc_ref))
                            .to_group(ShouldBreak::No, doc_ref)
                    }
                }
            }
            Expression::IfExpression(if_expression) => {
                let (if_conditional, else_ifs, trailing_else) = (
                    &if_expression.if_conditional,
                    &if_expression.else_ifs,
                    &if_expression.trailing_else,
                );

                let if_conditional_to_docs =
                    |if_conditional: &IfConditional<'_>, doc_ref: &mut usize| {
                        let (keyword, left_delim, condition, right_delim, body) = (
                            if_conditional.keyword,
                            if_conditional.left_delimiter,
                            &if_conditional.condition,
                            if_conditional.right_delimiter,
                            &if_conditional.body,
                        );
                        let condition_docs = left_delim
                            .to_docs(config, doc_ref)
                            .cons(nl!(""))
                            .cons(condition.to_docs(config, doc_ref))
                            .nest(config.indent())
                            .cons(nl!(""))
                            .cons(right_delim.to_docs(config, doc_ref))
                            .to_group(ShouldBreak::No, doc_ref);
                        keyword
                            .to_docs(config, doc_ref)
                            .cons(text!(" "))
                            .cons(condition_docs)
                            .cons(text!(" "))
                            .cons(body.to_docs(config, doc_ref))
                    };
                let mut docs = if_conditional_to_docs(if_conditional, doc_ref);
                for else_if in else_ifs {
                    let (else_keyword, conditional) =
                        (else_if.else_keyword, &else_if.if_conditional);
                    docs = docs
                        .cons(text!(" "))
                        .cons(else_keyword.to_docs(config, doc_ref))
                        .cons(text!(" "))
                        .cons(if_conditional_to_docs(conditional, doc_ref));
                }
                if let Some(trailing_else) = trailing_else {
                    let (else_keyword, body) = (&trailing_else.else_keyword, &trailing_else.body);
                    docs = docs
                        .cons(text!(" "))
                        .cons(else_keyword.to_docs(config, doc_ref))
                        .cons(text!(" "))
                        .cons(body.to_docs(config, doc_ref));
                }
                docs
            }
            Expression::WhileExpression(while_expression) => {
                let (keyword, condition, body) = (
                    &while_expression.while_keyword,
                    &while_expression.condition,
                    &while_expression.body,
                );
                keyword
                    .to_docs(config, doc_ref)
                    .cons(text!(" "))
                    .cons(condition.to_docs(config, doc_ref))
                    .cons(text!(" "))
                    .cons(body.to_docs(config, doc_ref))
                    .to_group(ShouldBreak::No, doc_ref)
            }
            Expression::RepeatExpression(repeat_expression) => {
                let (keyword, body) = (&repeat_expression.repeat_keyword, &repeat_expression.body);
                keyword
                    .to_docs(config, doc_ref)
                    .cons(body.to_docs(config, doc_ref))
                    .to_group(ShouldBreak::No, doc_ref)
            }
            Expression::FunctionCall(function_call) => {
                let (function_ref, args) = (&function_call.function_ref, &function_call.args);
                let group_ref = *doc_ref + 1;
                *doc_ref += 1;
                group!(
                    function_ref
                        .to_docs(config, doc_ref)
                        .cons(args_to_docs_with_conditional_nest(
                            args, config, doc_ref, group_ref
                        )),
                    should_break_args(args),
                    group_ref
                )
            }
            Expression::SubsetExpression(subset_expression) => {
                let (object_ref, args) = (&subset_expression.object_ref, &subset_expression.args);
                object_ref
                    .to_docs(config, doc_ref)
                    .cons(args.to_docs(config, doc_ref))
                    .to_group(should_break_args(args), doc_ref)
            }
            Expression::ForLoopExpression(for_loop) => {
                let (keyword, left_delim, identifier, in_keyword, collection, right_delim, body) = (
                    &for_loop.keyword,
                    &for_loop.left_delim,
                    &for_loop.identifier,
                    &for_loop.in_keyword,
                    &for_loop.collection,
                    &for_loop.right_delim,
                    &for_loop.body,
                );
                keyword
                    .to_docs(config, doc_ref)
                    .cons(
                        text!(" ")
                            .cons(left_delim.to_docs(config, doc_ref))
                            .cons(nl!(""))
                            .cons(identifier.to_docs(config, doc_ref))
                            .cons(text!(" "))
                            .cons(in_keyword.to_docs(config, doc_ref))
                            .cons(nl!(" "))
                            .cons(collection.to_docs(config, doc_ref))
                            .nest(config.indent()),
                    )
                    .cons(nl!(""))
                    .cons(right_delim.to_docs(config, doc_ref))
                    .to_group(ShouldBreak::No, doc_ref)
                    .cons(text!(" "))
                    .cons(body.to_docs(config, doc_ref))
                    .to_group(ShouldBreak::No, doc_ref)
            }
            Expression::LambdaFunction(lambda) => {
                let (keyword, args, body) = (&lambda.keyword, &lambda.args, &lambda.body);
                keyword
                    .to_docs(config, doc_ref)
                    .cons(args.to_docs(config, doc_ref))
                    .cons(text!(" "))
                    .cons(body.to_docs(config, doc_ref))
                    .to_group(ShouldBreak::No, doc_ref)
            }
        }
    }
}

impl Code for Args<'_> {
    fn to_docs(&self, config: &impl FormattingConfig, doc_ref: &mut usize) -> Rc<Doc> {
        let inside_delims = self
            .args
            .iter()
            .map(|arg| {
                arg.to_docs(config, doc_ref)
                    .to_group(ShouldBreak::No, doc_ref)
            })
            .reduce(|first, second| first.cons(nl!(" ")).cons(second));

        if let Some(inside_delims) = inside_delims {
            let nested_inside_delims = nl!("").cons(inside_delims).nest(config.indent());
            self.left_delimeter
                .to_docs(config, doc_ref)
                .cons(nested_inside_delims)
                .cons(nl!(""))
                .cons(self.right_delimeter.to_docs(config, doc_ref))
        } else {
            self.left_delimeter
                .to_docs(config, doc_ref)
                .cons(self.right_delimeter.to_docs(config, doc_ref))
        }
    }
}
impl Code for Arg<'_> {
    fn to_docs(&self, config: &impl FormattingConfig, doc_ref: &mut usize) -> Rc<Doc> {
        if let Some(comma) = &self.1 {
            self.0
                .to_docs(config, doc_ref)
                .cons(comma.to_docs(config, doc_ref))
        } else {
            self.0.to_docs(config, doc_ref)
        }
    }
}

fn args_to_docs_with_conditional_nest(
    args: &Args,
    config: &impl FormattingConfig,
    doc_ref: &mut usize,
    observed_doc: usize,
) -> Rc<Doc> {
    let inside_delims = args
        .args
        .iter()
        .map(|arg| {
            arg.to_docs(config, doc_ref)
                .to_group(ShouldBreak::No, doc_ref)
        })
        .reduce(|first, second| first.cons(nl!(" ")).cons(second));

    if let Some(inside_delims) = inside_delims {
        let nested_inside_delims = nl!("")
            .cons(inside_delims)
            .nest_if_break(config.indent(), observed_doc);
        args.left_delimeter
            .to_docs(config, doc_ref)
            .cons(nested_inside_delims)
            .cons(nl!(""))
            .cons(args.right_delimeter.to_docs(config, doc_ref))
    } else {
        args.left_delimeter
            .to_docs(config, doc_ref)
            .cons(args.right_delimeter.to_docs(config, doc_ref))
    }
}

fn should_break_args(args: &Args) -> ShouldBreak {
    // Tidyverse has some crazy breaking rules regarding curly braces
    // breaking. See this: https://style.tidyverse.org/syntax.html#indenting
    // Specifically, these are good examples:
    //
    // test_that("call1 returns an ordered factor", {
    //   expect_s3_class(call1(x, y), c("factor", "ordered"))
    // })
    //
    // tryCatch(
    //   {
    //     x <- scan()
    //     cat("Total: ", sum(x), "\n", sep = "")
    //   },
    //   interrupt = function(e) {
    //     message("Aborted by user")
    //   }
    // )
    //
    // The first one is just wack, because there are breaks inside the arguments,
    // but the inside of the parenthesis behaves like if it wasn't broken.
    // Notice that there is only single level of indent in the closure as well. Wack.
    //
    // And the second one does not try to emulate what the first one does,
    // it just does the breaking normally.
    //
    // I am going to create some custom rules for this behaviour (lots of ifs basically)
    // to specify what is the desired behaviour:
    // * if there is only one argument then support this:
    // f({
    //   2
    // }) (let the algorithm calculate the fits normally)
    // * if there are >= two arguments, and only the last one contains closures, let things happen
    // normally (so this wack behaviour from the first example is supported)
    // * if there are >= two arguments and not only the last one contains closures,
    // break all arguments

    if args.args.len() >= 2
        && args
            .args
            .iter()
            .take(args.args.len() - 1)
            .any(|arg| arg.0.iter().any(contains_closure))
    {
        ShouldBreak::Yes
    } else {
        ShouldBreak::No
    }
}

fn contains_closure(expr: &Expression) -> bool {
    match expr {
        Expression::Symbol(_)
        | Expression::Literal(_)
        | Expression::Formula(_, _)
        | Expression::Newline(_)
        | Expression::Whitespace(_)
        | Expression::EOF(_)
        | Expression::Break(_)
        | Expression::Continue(_)
        | Expression::Comment(_) => false,
        Expression::Term(term) => {
            if is_embracing_operator_closure(term) {
                false
            } else if let Some(pre_delim) = term.pre_delimiters {
                matches!(pre_delim.token, Token::LBrace)
            } else {
                term.term.iter().any(contains_closure)
            }
        }
        Expression::Unary(_, expr) => contains_closure(expr),
        Expression::Bop(_, expr1, expr2) => contains_closure(expr1) || contains_closure(expr2),
        Expression::FunctionDef(func_def) => {
            func_def
                .arguments
                .args
                .iter()
                .any(|arg| arg.0.iter().any(contains_closure))
                || contains_closure(&func_def.body)
        }
        Expression::LambdaFunction(_) => false,
        Expression::IfExpression(if_expr) => {
            contains_closure(&if_expr.if_conditional.condition)
                || contains_closure(&if_expr.if_conditional.body)
                || if_expr.else_ifs.iter().any(|else_if| {
                    contains_closure(&else_if.if_conditional.body)
                        || contains_closure(&else_if.if_conditional.condition)
                })
                || if_expr
                    .trailing_else
                    .iter()
                    .any(|trailing_else| contains_closure(&trailing_else.body))
        }
        Expression::WhileExpression(while_loop) => {
            contains_closure(&while_loop.condition) || contains_closure(&while_loop.body)
        }
        Expression::RepeatExpression(_) => true,
        Expression::FunctionCall(call) => call
            .args
            .args
            .iter()
            .any(|arg| arg.0.iter().any(contains_closure)),
        Expression::SubsetExpression(subset) => subset
            .args
            .args
            .iter()
            .any(|arg| arg.0.iter().any(contains_closure)),
        Expression::ForLoopExpression(for_loop) => {
            contains_closure(&for_loop.collection) || contains_closure(&for_loop.body)
        }
    }
}

fn is_embracing_operator_closure(term: &TermExpr) -> bool {
    match (term.pre_delimiters, term.term.first()) {
        (None, _) | (Some(_), None) => false,
        (Some(pre), Some(first)) => {
            if !matches!(pre.token, Token::LBrace) {
                return false;
            }
            if let Expression::Term(term) = first {
                term.pre_delimiters.is_some()
                    && matches!(term.pre_delimiters.unwrap().token, Token::LBrace)
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::FunctionLineBreaks,
        format::{format_to_sdoc, simple_doc_to_string, Mode},
    };

    use super::*;

    struct MockConfig;

    impl FormattingConfig for MockConfig {
        fn line_length(&self) -> i32 {
            120
        }
        fn indent(&self) -> i32 {
            0
        }
        fn embracing_op_no_nl(&self) -> bool {
            true
        }

        fn allow_nl_after_assignment(&self) -> bool {
            true
        }

        fn space_before_complex_rhs_in_formulas(&self) -> bool {
            true
        }

        fn strip_suffix_whitespace_in_function_defs(&self) -> bool {
            true
        }

        fn function_line_breaks(&self) -> FunctionLineBreaks {
            FunctionLineBreaks::Hanging
        }
    }
    impl std::fmt::Display for MockConfig {
        fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            unimplemented!()
        }
    }
    use std::collections::{HashSet, VecDeque};

    #[test]
    fn joining_docs_with_newlines_produces_newlines() {
        let docs = [text!("test"), text!("test2")];
        let mock_config = MockConfig {};
        let mut doc = VecDeque::from([(
            0,
            Mode::Flat,
            join_docs(
                docs,
                Rc::new(Doc::Nil),
                ShouldBreak::Yes,
                &mock_config,
                &mut 0,
            ),
        )]);

        let mut s = HashSet::default();
        let sdoc = Rc::new(format_to_sdoc(0, &mut doc, &mock_config, &mut s));

        assert_eq!(simple_doc_to_string(sdoc), "test\ntest2")
    }

    #[test]
    fn joinin_docs_with_newlines_does_nothing_for_just_one_doc() {
        let docs = [text!("test")];
        let mock_config = MockConfig {};
        let mut doc = VecDeque::from([(
            0,
            Mode::Flat,
            join_docs(
                docs,
                Rc::new(Doc::Nil),
                ShouldBreak::No,
                &mock_config,
                &mut 0,
            ),
        )]);

        let mut s = HashSet::default();
        let sdoc = Rc::new(format_to_sdoc(0, &mut doc, &mock_config, &mut s));

        assert_eq!(simple_doc_to_string(sdoc), "test")
    }
}
