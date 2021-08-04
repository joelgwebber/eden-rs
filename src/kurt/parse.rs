use pest::iterators::Pair;
use pest::Parser;

use crate::kurt::expr::Apply;
use crate::kurt::expr::Assoc;
use crate::kurt::expr::Block;
use crate::kurt::expr::List;
use crate::kurt::ERef;
use crate::kurt::Expr;

use super::Kurt;

#[derive(Parser)]
#[grammar = "kurt/kurt.pest"]
struct KurtParser;

impl Kurt {
    pub fn parse(&self, src: String) -> Expr {
        let file = KurtParser::parse(Rule::file, &src)
            .expect("unsuccessful parse")
            .next()
            .unwrap();

        fn parse_value(expr: Pair<Rule>) -> Expr {
            match expr.as_rule() {
                Rule::dict => {
                    let mut vec = Vec::<(Expr, Expr)>::new();
                    expr.into_inner().for_each(|pair| match pair.as_rule() {
                        Rule::pair => {
                            let mut inner_rules = pair.into_inner();
                            let key = parse_value(inner_rules.next().unwrap());
                            let value = parse_value(inner_rules.next().unwrap());
                            vec.push((key, value));
                        }
                        _ => unreachable!(),
                    });
                    Expr::EAssoc(ERef::new(Assoc { pairs: vec }))
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
                    let exprs = rules.map(parse_value).collect();
                    Expr::EBlock(ERef::new(Block {
                        params: params,
                        expr: Expr::EApply(ERef::new(Apply { exprs: exprs })),
                        env: Expr::ENil,
                        slf: Expr::ENil,
                    }))
                }

                Rule::apply => Expr::EApply(ERef::new(Apply {
                    exprs: expr.into_inner().map(parse_value).collect(),
                })),

                Rule::list => Expr::EList(ERef::new(List {
                    exprs: expr.into_inner().map(parse_value).collect(),
                })),

                Rule::number => Expr::ENum(expr.as_str().parse().unwrap()),
                Rule::boolean => Expr::EBool(expr.as_str().parse().unwrap()),
                Rule::string => Expr::EStr(String::from(expr.as_str())),
                Rule::id => Expr::EId(String::from(expr.as_str())),
                Rule::prim => parse_value(expr.into_inner().next().unwrap()),
                Rule::expr => parse_value(expr.into_inner().next().unwrap()),

                Rule::quote => {
                    Expr::EQuote(ERef::new(parse_value(expr.into_inner().next().unwrap())))
                }
                Rule::unquote => {
                    Expr::EUnquote(ERef::new(parse_value(expr.into_inner().next().unwrap())))
                }

                _ => unreachable!(),
            }
        }

        let expr = file.into_inner().next().unwrap();
        parse_value(expr)
    }
}
