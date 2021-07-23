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

use std::collections::HashMap;

use crate::config::Axis;
use crate::expr::AxisExpression;
use crate::expr::Operator;

impl AxisExpression {
    pub fn eval(&self, values: &HashMap<(String, Axis), i32>) -> Result<i32, String> {
        return match self {
            AxisExpression::AxisReference(dev, axis) => match values.get(&(dev.clone(), *axis)) {
                Some(value) => Ok(*value),
                None => Err(format!("No value is known for axis {}:{:?}", dev, axis)),
            },
            AxisExpression::Literal(value) => Ok(*value),
            AxisExpression::BiOp(op, left, right) => match op {
                Operator::Add => Ok(left.eval(values)? + right.eval(values)?),
                Operator::Sub => Ok(left.eval(values)? - right.eval(values)?),
                Operator::Mul => Ok(left.eval(values)? * right.eval(values)?),
                Operator::Div => Ok(left.eval(values)? / right.eval(values)?),
            },
        };
    }

    pub fn dependencies(&self) -> Vec<(String, Axis)> {
        return match self {
            AxisExpression::AxisReference(dev, axis) => vec![(dev.clone(), *axis)],
            AxisExpression::BiOp(_, left, right) => {
                let mut left_deps = left.dependencies();
                left_deps.append(&mut right.dependencies());
                left_deps.dedup();
                left_deps
            }
            AxisExpression::Literal(_) => Vec::new(),
        };
    }
}
