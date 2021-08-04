use pest::iterators::Pair;
use pest::Parser;

use crate::kurt::expr::Block;
use crate::kurt::Expr;
use crate::kurt::ERef;

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
                    Expr::Assoc(ERef::new(vec))
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
                    Expr::Block(ERef::new(Block {
                        params: params,
                        expr: Expr::Apply(ERef::new(exprs)),
                        env: Expr::Nil,
                        slf: Expr::Nil,
                    }))
                }

                Rule::apply => {
                    Expr::Apply(ERef::new(expr.into_inner().map(parse_value).collect()))
                }
                Rule::list => {
                    Expr::List(ERef::new(expr.into_inner().map(parse_value).collect()))
                }
                Rule::number => Expr::Num(expr.as_str().parse().unwrap()),
                Rule::boolean => Expr::Bool(expr.as_str().parse().unwrap()),
                Rule::string => Expr::Str(String::from(expr.as_str())),
                Rule::id => Expr::Id(String::from(expr.as_str())),
                Rule::prim => parse_value(expr.into_inner().next().unwrap()),
                Rule::expr => parse_value(expr.into_inner().next().unwrap()),

                Rule::quote => {
                    Expr::Quote(ERef::new(parse_value(expr.into_inner().next().unwrap())))
                }
                Rule::unquote => {
                    Expr::Unquote(ERef::new(parse_value(expr.into_inner().next().unwrap())))
                }

                _ => unreachable!(),
            }
        }

        let expr = file.into_inner().next().unwrap();
        parse_value(expr)
    }
}
