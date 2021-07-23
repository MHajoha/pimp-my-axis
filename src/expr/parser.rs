/*
 * This file is part of Pimp-My-Axis.
 *
 * Pimp-My-Axis is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Pimp-My-Axis is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Pimp-My-Axis. If not, see <http://www.gnu.org/licenses/>.
 */

use std::str::FromStr;

use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest::prec_climber::{Assoc, Operator as PestOperator, PrecClimber};
use pest_derive::Parser;

use crate::config::Axis;
use crate::expr::{AxisExpression, Operator};

#[derive(Parser)]
#[grammar = "expr/grammar.pest"]
struct ExprParser;

type ExprResult = Result<AxisExpression, String>;

fn primary(pair: Pair<'_, Rule>, climber: &PrecClimber<Rule>) -> ExprResult {
    match pair.as_rule() {
        Rule::expr => climber.climb(pair.into_inner(), |pair| primary(pair, climber), infix),
        Rule::axis_ref => match pair.as_str().split_once(':') {
            Some((dev, axis)) => Ok(AxisExpression::AxisReference(
                dev.to_owned(),
                Axis::from_str(axis)?,
            )),
            None => Err(format!("Invalid axis reference: {}", pair.as_str())),
        },
        Rule::literal => Ok(AxisExpression::Literal(
            pair.as_str().trim().parse().unwrap(),
        )),
        _ => panic!(),
    }
}

fn infix(lhs: ExprResult, op: Pair<Rule>, rhs: ExprResult) -> ExprResult {
    Ok(AxisExpression::BiOp(
        Operator::from_str(op.as_str()).unwrap(),
        Box::new(lhs?),
        Box::new(rhs?),
    ))
}

pub fn parse_expr(input: &str) -> ExprResult {
    let climber = PrecClimber::new(vec![
        PestOperator::new(Rule::add_op, Assoc::Left),
        PestOperator::new(Rule::mul_op, Assoc::Left),
    ]);

    let pairs: Pairs<'_, Rule> =
        ExprParser::parse(Rule::main, input).map_err(|err| err.to_string().to_owned())?;

    return climber.climb(pairs, |pair| primary(pair, &climber), infix);
}

#[cfg(test)]
mod tests {
    use crate::expr::parser::parse_expr;

    #[test]
    fn test() {
        let parsed = parse_expr("2 + 1 * (1 + 2)").unwrap();
        println!("{:?}", parsed)
    }
}
