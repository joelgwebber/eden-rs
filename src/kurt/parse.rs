use pest::iterators::Pair;
use pest::Parser;

use crate::kurt::expr::Apply;
use crate::kurt::expr::Assoc;
use crate::kurt::expr::Block;
use crate::kurt::expr::List;
use crate::kurt::ERef;
use crate::kurt::Expr;
use crate::kurt::Loc;

use super::Kurt;

#[derive(Parser)]
#[grammar = "kurt/kurt.pest"]
struct KurtParser;

impl Kurt {
    pub fn parse(&self, name: &str, src: &str) -> Expr {
        let file = KurtParser::parse(Rule::file, src)
            .expect("unsuccessful parse")
            .next()
            .unwrap();

        let expr = file.into_inner().next().unwrap();
        self.parse_value(name, expr)
    }

    fn parse_value(&self, file: &str, expr: Pair<Rule>) -> Expr {
        let span = expr.as_span();
        match expr.as_rule() {
            Rule::dict => {
                let mut vec = Vec::<(Expr, Expr)>::new();
                expr.into_inner().for_each(|pair| match pair.as_rule() {
                    Rule::pair => {
                        let mut inner_rules = pair.into_inner();
                        let key = self.parse_value(file, inner_rules.next().unwrap());
                        let value = self.parse_value(file, inner_rules.next().unwrap());
                        vec.push((key, value));
                    }
                    _ => unreachable!(),
                });
                Expr::EAssoc(ERef::new(Assoc {
                    loc: Loc {
                        file: file.to_string(),
                        name: "".to_string(),
                        pos: span.start_pos().line_col(),
                    },
                    pairs: vec,
                }))
            }

            Rule::block => {
                let mut rules = expr.into_inner();
                let params = rules
                    .next()
                    .unwrap()
                    .into_inner()
                    .map(|expr| match expr.as_rule() {
                        Rule::id => expr.as_str().to_string(),
                        _ => panic!(),
                    })
                    .collect();
                let exprs = rules.map(|val| self.parse_value(file, val)).collect();
                Expr::EBlock(ERef::new(Block {
                    loc: Loc {
                        file: file.to_string(),
                        name: "".to_string(),
                        pos: span.start_pos().line_col(),
                    },
                    params: params,
                    expr: Expr::EApply(ERef::new(Apply {
                        loc: Loc {
                            file: file.to_string(),
                            name: "".to_string(),
                            pos: span.start_pos().line_col(),
                        },
                        exprs: exprs,
                    })),
                    env: Expr::ENil,
                    slf: Expr::ENil,
                }))
            }

            Rule::apply => Expr::EApply(ERef::new(Apply {
                loc: Loc {
                    file: file.to_string(),
                    name: "".to_string(),
                    pos: span.start_pos().line_col(),
                },
                exprs: expr
                    .into_inner()
                    .map(|val| self.parse_value(file, val))
                    .collect(),
            })),

            Rule::list => Expr::EList(ERef::new(List {
                loc: Loc {
                    file: file.to_string(),
                    name: "".to_string(),
                    pos: span.start_pos().line_col(),
                },
                exprs: expr
                    .into_inner()
                    .map(|val| self.parse_value(file, val))
                    .collect(),
            })),

            Rule::access => {
                let mut inner = expr.into_inner();
                let mut left = self.parse_value(file, inner.next().unwrap());
                loop {
                    match inner.next() {
                        Some(right_rule) => {
                            let mut right = self.parse_value(file, right_rule);
                            if let Expr::EId(_) = right {
                                right = Expr::EQuote(ERef::new(right));
                            }
                            left = Expr::EApply(ERef::new(Apply {
                                loc: Loc {
                                    file: file.to_string(),
                                    name: String::default(),
                                    pos: span.start_pos().line_col(),
                                },
                                exprs: vec![left, right],
                            }));
                        }
                        None => return left,
                    }
                }
            }

            Rule::non_access => self.parse_value(file, expr.into_inner().next().unwrap()),

            Rule::number => Expr::ENum(expr.as_str().parse().unwrap()),
            Rule::boolean => Expr::EBool(expr.as_str().parse().unwrap()),
            Rule::string => Expr::EStr(expr.as_str().to_string()),
            Rule::id => Expr::EId(expr.as_str().to_string()),
            Rule::prim => self.parse_value(file, expr.into_inner().next().unwrap()),
            Rule::expr => self.parse_value(file, expr.into_inner().next().unwrap()),

            Rule::quote => Expr::EQuote(ERef::new(
                self.parse_value(file, expr.into_inner().next().unwrap()),
            )),
            Rule::unquote => Expr::EUnquote(ERef::new(
                self.parse_value(file, expr.into_inner().next().unwrap()),
            )),

            _ => unreachable!(),
        }
    }
}
