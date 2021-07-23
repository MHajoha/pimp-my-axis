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

use serde::{Deserialize, Deserializer};
use serde::de::{Error, Unexpected};

pub use parser::parse_expr;

use crate::config::Axis;

mod eval;
mod parser;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AxisExpression {
    AxisReference(String, Axis),
    Literal(i32),
    BiOp(Operator, Box<AxisExpression>, Box<AxisExpression>),
}

impl<'de> Deserialize<'de> for AxisExpression {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let string = String::deserialize(deserializer)?;
        return parse_expr(&string)
            .map_err(|err| D::Error::invalid_value(Unexpected::Str(&string), &err.as_str()));
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

impl FromStr for Operator {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        return match s.trim() {
            "+" => Ok(Operator::Add),
            "-" => Ok(Operator::Sub),
            "*" => Ok(Operator::Mul),
            "/" => Ok(Operator::Div),
            _ => Err(format!("Unknown operator: '{}'", s)),
        };
    }
}
