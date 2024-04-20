use parser::ast::{
    Arg, Args, ElseIfConditional, Expression, ExpressionsBuffer, FunctionDefinition, IfConditional,
    IfExpression, TermExpr, TrailingElse, WhileExpression,
};
use parser::{parse, pre_parse};
use tokenizer::Tokenizer;

fn log_init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn test_cases() {
    log_init();
    let code = include_str!("./test_cases/001.R");
    let mut tokenizer = Tokenizer::new(code);
    let mut commented_tokens = tokenizer.tokenize();

    let tokens = pre_parse(&mut commented_tokens);
    let res = parse(&tokens).unwrap();
    let expected = vec![
        parser::ast::Expression::Literal(tokens[0]),
        parser::ast::Expression::Whitespace(&tokens[1..2]),
        parser::ast::Expression::EOF(tokens[3]),
    ];
    assert_eq!(res, expected);
}

#[test]
fn literal_with_parentheses() {
    log_init();
    let code = include_str!("./test_cases/002.R");
    let mut tokenizer = Tokenizer::new(code);
    let mut commented_tokens = tokenizer.tokenize();

    let tokens = pre_parse(&mut commented_tokens);
    let res = parse(&tokens).unwrap();
    let expected = vec![
        Expression::Term(Box::new(TermExpr {
            pre_delimiters: Some(tokens[0]),
            term: vec![Expression::Literal(tokens[1])],
            post_delimiters: Some(tokens[2]),
        })),
        Expression::EOF(tokens[4]),
    ];
    assert_eq!(res, expected);
}

#[test]
fn bop_with_parentheses() {
    log_init();
    let code = include_str!("./test_cases/003.R");
    let mut tokenizer = Tokenizer::new(code);
    let mut commented_tokens = tokenizer.tokenize();

    let tokens = pre_parse(&mut commented_tokens);
    let res = parse(&tokens).unwrap();
    let expected = vec![
        Expression::Term(Box::new(TermExpr {
            pre_delimiters: Some(tokens[0]),
            term: vec![Expression::Bop(
                tokens[2],
                Box::new(Expression::Literal(tokens[1])),
                Box::new(Expression::Literal(tokens[3])),
            )],
            post_delimiters: Some(tokens[4]),
        })),
        Expression::EOF(tokens[6]),
    ];
    assert_eq!(res, expected);
}

#[test]
fn empty_function_definition() {
    log_init();
    let code = include_str!("./test_cases/004.R");
    let mut tokenizer = Tokenizer::new(code);
    let mut commented_tokens = tokenizer.tokenize();

    let tokens = pre_parse(&mut commented_tokens);
    let res = parse(&tokens).unwrap();
    let expected = vec![
        Expression::FunctionDef(FunctionDefinition::new(
            tokens[0],
            Args::new(
                Box::new(Expression::Literal(tokens[1])),
                vec![],
                Box::new(Expression::Literal(tokens[2])),
            ),
            Box::new(Expression::Term(Box::new(TermExpr::new(
                Some(tokens[3]),
                vec![],
                Some(tokens[4]),
            )))),
        )),
        Expression::EOF(tokens[6]),
    ];

    assert_eq!(res, expected);
}

#[test]
fn function_def_with_one_arg() {
    log_init();
    let code = include_str!("./test_cases/005.R");
    let mut tokenizer = Tokenizer::new(code);
    let mut commented_tokens = tokenizer.tokenize();

    let tokens = pre_parse(&mut commented_tokens);
    let res = parse(&tokens).unwrap();
    let expected = vec![
        Expression::FunctionDef(FunctionDefinition::new(
            tokens[0],
            Args::new(
                Box::new(Expression::Literal(tokens[1])),
                vec![Arg(Expression::Symbol(tokens[2]), None)],
                Box::new(Expression::Literal(tokens[3])),
            ),
            Box::new(Expression::Term(Box::new(TermExpr::new(
                Some(tokens[4]),
                vec![],
                Some(tokens[5]),
            )))),
        )),
        Expression::EOF(tokens[7]),
    ];

    assert_eq!(
        res,
        expected,
        "res: {}, expected: {}",
        ExpressionsBuffer(&res),
        ExpressionsBuffer(&expected)
    );
}

#[test]
fn function_def_with_one_arg_with_default_value() {
    log_init();
    let code = include_str!("./test_cases/006.R");
    let mut tokenizer = Tokenizer::new(code);
    let mut commented_tokens = tokenizer.tokenize();

    let tokens = pre_parse(&mut commented_tokens);
    let res = parse(&tokens).unwrap();
    let expected = vec![
        Expression::FunctionDef(FunctionDefinition::new(
            tokens[0],
            Args::new(
                Box::new(Expression::Literal(tokens[1])),
                vec![Arg(
                    Expression::Bop(
                        tokens[3],
                        Box::new(Expression::Symbol(tokens[2])),
                        Box::new(Expression::Literal(tokens[4])),
                    ),
                    None,
                )],
                Box::new(Expression::Literal(tokens[5])),
            ),
            Box::new(Expression::Term(Box::new(TermExpr::new(
                Some(tokens[6]),
                vec![],
                Some(tokens[7]),
            )))),
        )),
        Expression::EOF(tokens[9]),
    ];

    assert_eq!(
        res,
        expected,
        "res: {}, expected: {}",
        ExpressionsBuffer(&res),
        ExpressionsBuffer(&expected)
    );
}

#[test]
fn function_def_with_two_args() {
    log_init();
    let code = include_str!("./test_cases/007.R");
    let mut tokenizer = Tokenizer::new(code);
    let mut commented_tokens = tokenizer.tokenize();

    let tokens = pre_parse(&mut commented_tokens);
    let res = parse(&tokens).unwrap();
    let expected = vec![
        Expression::FunctionDef(FunctionDefinition::new(
            tokens[0],
            Args::new(
                Box::new(Expression::Literal(tokens[1])),
                vec![
                    Arg(
                        Expression::Symbol(tokens[2]),
                        Some(Expression::Literal(tokens[3])),
                    ),
                    Arg(Expression::Symbol(tokens[4]), None),
                ],
                Box::new(Expression::Literal(tokens[5])),
            ),
            Box::new(Expression::Term(Box::new(TermExpr::new(
                Some(tokens[6]),
                vec![],
                Some(tokens[7]),
            )))),
        )),
        Expression::EOF(tokens[9]),
    ];

    assert_eq!(
        res,
        expected,
        "res: {}, expected: {}",
        ExpressionsBuffer(&res),
        ExpressionsBuffer(&expected)
    );
}

#[test]
fn function_inline_body() {
    log_init();
    let code = include_str!("./test_cases/008.R");
    let mut tokenizer = Tokenizer::new(code);
    let mut commented_tokens = tokenizer.tokenize();

    let tokens = pre_parse(&mut commented_tokens);
    let res = parse(&tokens).unwrap();
    let expected = vec![
        Expression::FunctionDef(FunctionDefinition::new(
            tokens[0],
            Args::new(
                Box::new(Expression::Literal(tokens[1])),
                vec![],
                Box::new(Expression::Literal(tokens[2])),
            ),
            Box::new(Expression::Literal(tokens[3])),
        )),
        Expression::EOF(tokens[5]),
    ];

    assert_eq!(
        res,
        expected,
        "res: {}, expected: {}",
        ExpressionsBuffer(&res),
        ExpressionsBuffer(&expected)
    );
}

#[test]
fn function_multiline_body() {
    log_init();
    let code = include_str!("./test_cases/009.R");
    let mut tokenizer = Tokenizer::new(code);
    let mut commented_tokens = tokenizer.tokenize();
    let tokens = pre_parse(&mut commented_tokens);

    let res = parse(&tokens).unwrap();
    let expected = vec![
        Expression::FunctionDef(FunctionDefinition::new(
            tokens[0],
            Args::new(
                Box::new(Expression::Literal(tokens[1])),
                vec![],
                Box::new(Expression::Literal(tokens[2])),
            ),
            Box::new(Expression::Term(Box::new(TermExpr::new(
                Some(tokens[3]),
                vec![
                    Expression::Literal(tokens[5]),
                    Expression::Literal(tokens[7]),
                ],
                Some(tokens[9]),
            )))),
        )),
        Expression::EOF(tokens[11]),
    ];

    assert_eq!(
        res,
        expected,
        "res: {}\nexpected: {}",
        ExpressionsBuffer(&res),
        ExpressionsBuffer(&expected)
    );
}

#[test]
fn if_conditional() {
    log_init();

    let code = include_str!("./test_cases/010.R");
    let mut tokenizer = Tokenizer::new(code);
    let mut commented_tokens = tokenizer.tokenize();
    let tokens = pre_parse(&mut commented_tokens);

    let res = parse(&tokens).unwrap();
    let expected = vec![
        Expression::IfExpression(IfExpression {
            if_conditional: IfConditional {
                keyword: tokens[0],
                left_delimiter: tokens[1],
                condition: Box::new(Expression::Literal(tokens[2])),
                right_delimiter: tokens[3],
                body: Box::new(Expression::Term(Box::new(TermExpr::new(
                    Some(tokens[4]),
                    vec![],
                    Some(tokens[5]),
                )))),
            },
            else_ifs: vec![],
            trailing_else: None,
        }),
        Expression::EOF(tokens[7]),
    ];

    assert_eq!(
        res,
        expected,
        "res: {}\nexpected: {}",
        ExpressionsBuffer(&res),
        ExpressionsBuffer(&expected)
    );
}

#[test]
fn if_with_else() {
    log_init();

    let code = include_str!("./test_cases/011.R");
    let mut tokenizer = Tokenizer::new(code);
    let mut commented_tokens = tokenizer.tokenize();
    let tokens = pre_parse(&mut commented_tokens);

    let res = parse(&tokens).unwrap();
    let expected = vec![
        Expression::IfExpression(IfExpression {
            if_conditional: IfConditional {
                keyword: tokens[0],
                left_delimiter: tokens[1],
                condition: Box::new(Expression::Literal(tokens[2])),
                right_delimiter: tokens[3],
                body: Box::new(Expression::Term(Box::new(TermExpr::new(
                    Some(tokens[4]),
                    vec![],
                    Some(tokens[5]),
                )))),
            },
            else_ifs: vec![],
            trailing_else: Some(TrailingElse {
                else_keyword: tokens[6],
                body: Box::new(Expression::Term(Box::new(TermExpr::new(
                    Some(tokens[7]),
                    vec![],
                    Some(tokens[8]),
                )))),
            }),
        }),
        Expression::EOF(tokens[10]),
    ];

    assert_eq!(
        res,
        expected,
        "res: {}\nexpected: {}",
        ExpressionsBuffer(&res),
        ExpressionsBuffer(&expected)
    );
}

#[test]
fn if_with_if_else_and_else() {
    log_init();

    let code = include_str!("./test_cases/012.R");
    let mut tokenizer = Tokenizer::new(code);
    let mut commented_tokens = tokenizer.tokenize();
    let tokens = pre_parse(&mut commented_tokens);

    let res = parse(&tokens).unwrap();
    let expected = vec![
        Expression::IfExpression(IfExpression {
            if_conditional: IfConditional {
                keyword: tokens[0],
                left_delimiter: tokens[1],
                condition: Box::new(Expression::Literal(tokens[2])),
                right_delimiter: tokens[3],
                body: Box::new(Expression::Term(Box::new(TermExpr::new(
                    Some(tokens[4]),
                    vec![],
                    Some(tokens[5]),
                )))),
            },
            else_ifs: vec![ElseIfConditional {
                else_keyword: tokens[6],
                if_conditional: IfConditional {
                    keyword: tokens[7],
                    left_delimiter: tokens[8],
                    condition: Box::new(Expression::Literal(tokens[9])),
                    right_delimiter: tokens[10],
                    body: Box::new(Expression::Term(Box::new(TermExpr::new(
                        Some(tokens[11]),
                        vec![],
                        Some(tokens[12]),
                    )))),
                },
            }],
            trailing_else: Some(TrailingElse {
                else_keyword: tokens[13],
                body: Box::new(Expression::Term(Box::new(TermExpr::new(
                    Some(tokens[14]),
                    vec![],
                    Some(tokens[15]),
                )))),
            }),
        }),
        Expression::EOF(tokens[17]),
    ];

    assert_eq!(
        res,
        expected,
        "res: {}\nexpected: {}",
        ExpressionsBuffer(&res),
        ExpressionsBuffer(&expected)
    );
}

#[test]
fn while_test() {
    log_init();

    let code = include_str!("./test_cases/013.R");
    let mut tokenizer = Tokenizer::new(code);
    let mut commented_tokens = tokenizer.tokenize();
    let tokens = pre_parse(&mut commented_tokens);

    let res = parse(&tokens).unwrap();
    let expected = vec![
        Expression::WhileExpression(WhileExpression {
            while_keyword: tokens[0],
            condition: Box::new(Expression::Term(Box::new(TermExpr::new(
                Some(tokens[1]),
                vec![Expression::Literal(tokens[2])],
                Some(tokens[3]),
            )))),
            body: Box::new(Expression::Term(Box::new(TermExpr::new(
                Some(tokens[4]),
                vec![],
                Some(tokens[5]),
            )))),
        }),
        Expression::EOF(tokens[7]),
    ];

    assert_eq!(
        res,
        expected,
        "res: {}\nexpected: {}",
        ExpressionsBuffer(&res),
        ExpressionsBuffer(&expected)
    );
}
