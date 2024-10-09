// use bigdecimal::BigDecimal;
use sqlparser::ast::{
    visit_statements_mut, Expr, GroupByExpr, Query, SelectItem, SetExpr, Statement,
    Value as AstValue,
};
use std::ops::ControlFlow;

mod result;
mod value;
mod wire;

pub use value::Value;

pub use result::{Error, R};

pub type Rv = Result<Value, Error>;

pub trait Parameters {
    fn get(&self, i: usize) -> Rv;
}

#[derive(Default)]
pub struct ParameterSet {
    pub values: Vec<Value>,
}

impl ParameterSet {
    pub fn add(&mut self, v: Value) -> usize {
        let pos = self.values.len();
        self.values.push(v);
        pos
    }
}

impl Parameters for ParameterSet {
    fn get(&self, i: usize) -> Rv {
        // TODO: resolve the placeholder to an actual Value.
        if i > 0 && i <= self.values.len() {
            return Ok(self.values[i - 1].clone());
        }
        // let n = BigDecimal::from(u32::try_from(i).unwrap_or(0u32));
        // Ok(Value::Number(n))
        Err(Error::Notfound(format!("${}", i)))
    }
}

fn placeholder_to_usize(p: &str) -> usize {
    let i_str = String::from_utf8(p.as_bytes()[1..].to_vec()).unwrap();
    i_str.as_str().parse::<usize>().unwrap_or(0)
}

pub fn resolve(ps: &dyn Parameters, p: &str) -> Rv {
    let i: usize = placeholder_to_usize(p);
    ps.get(i)
}

pub fn resolve_parameters_expr(ps: &dyn Parameters, x: &mut Expr) -> R {
    match x {
        Expr::Value(AstValue::Placeholder(p)) => {
            let v = resolve(ps, p)?;
            *x = v.into();
        }
        Expr::IsNull(bv) => {
            let v = bv.as_mut();
            resolve_parameters_expr(ps, v)?;
        }
        Expr::IsNotNull(bv) => {
            let v = bv.as_mut();
            resolve_parameters_expr(ps, v)?;
        }
        Expr::InList {
            expr,
            list,
            negated: _,
        } => {
            let x = expr.as_mut();
            resolve_parameters_expr(ps, x)?;
            for x in list.iter_mut() {
                resolve_parameters_expr(ps, x)?;
            }
        }
        Expr::InSubquery {
            expr,
            subquery,
            negated: _,
        } => {
            let x = expr.as_mut();
            resolve_parameters_expr(ps, x)?;
            let q = subquery.as_mut();
            resolve_parameters_query(ps, q)?;
        }
        Expr::Between {
            expr,
            negated: _,
            low,
            high,
        } => {
            let x = expr.as_mut();
            resolve_parameters_expr(ps, x)?;
            let vl = low.as_mut();
            let vh = high.as_mut();
            resolve_parameters_expr(ps, vl)?;
            resolve_parameters_expr(ps, vh)?;
        }
        Expr::Like {
            expr,
            negated: _,
            pattern,
            escape_char: _,
        } => {
            let x = expr.as_mut();
            resolve_parameters_expr(ps, x)?;
            let p = pattern.as_mut();
            resolve_parameters_expr(ps, p)?;
        }
        Expr::ILike {
            expr,
            negated: _,
            pattern,
            escape_char: _,
        } => {
            let x = expr.as_mut();
            resolve_parameters_expr(ps, x)?;
            let p = pattern.as_mut();
            resolve_parameters_expr(ps, p)?;
        }
        Expr::BinaryOp { left, op: _, right } => {
            let vl = left.as_mut();
            resolve_parameters_expr(ps, vl)?;
            let vr = right.as_mut();
            resolve_parameters_expr(ps, vr)?;
        }
        Expr::UnaryOp { op: _, expr } => {
            let x = expr.as_mut();
            resolve_parameters_expr(ps, x)?;
        }
        Expr::Nested(bv) => {
            let v = bv.as_mut();
            resolve_parameters_expr(ps, v)?;
        }
        Expr::Exists {
            subquery,
            negated: _,
        } => {
            let q = subquery.as_mut();
            resolve_parameters_query(ps, q)?;
        }
        Expr::Subquery(bq) => {
            let q = bq.as_mut();
            resolve_parameters_query(ps, q)?;
        }
        Expr::Case {
            operand,
            conditions,
            results,
            else_result,
        } => {
            if let Some(bv) = operand {
                let v = bv.as_mut();
                resolve_parameters_expr(ps, v)?;
            }
            for expr in conditions.iter_mut() {
                resolve_parameters_expr(ps, expr)?;
            }
            for expr in results.iter_mut() {
                resolve_parameters_expr(ps, expr)?;
            }
            if let Some(bv) = else_result {
                let v = bv.as_mut();
                resolve_parameters_expr(ps, v)?;
            }
        }
        Expr::Interval(interval) => {
            let v = interval.value.as_mut();
            resolve_parameters_expr(ps, v)?;
        }
        Expr::Array(array) => {
            for expr in array.elem.iter_mut() {
                resolve_parameters_expr(ps, expr)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn resolve_parameters_query(ps: &dyn Parameters, q: &mut Query) -> R {
    let body = q.body.as_mut();
    match body {
        SetExpr::Select(bs) => {
            let s = bs.as_mut();
            if let Some(ref mut selection) = s.selection {
                resolve_parameters_expr(ps, selection)?;
            }
            for select_item in s.projection.iter_mut() {
                match select_item {
                    SelectItem::UnnamedExpr(expr) => {
                        resolve_parameters_expr(ps, expr)?;
                    }
                    SelectItem::ExprWithAlias { expr, alias: _ } => {
                        resolve_parameters_expr(ps, expr)?;
                    }
                    _ => {
                        // todo!();
                    }
                }
            }
            if let GroupByExpr::Expressions(exprs, _) = &mut s.group_by {
                for expr in exprs.iter_mut() {
                    resolve_parameters_expr(ps, expr)?;
                }
            }
            if let Some(having) = &mut s.having {
                resolve_parameters_expr(ps, having)?;
            }
        }
        SetExpr::Values(values) => {
            for row in values.rows.iter_mut() {
                for expr in row.iter_mut() {
                    resolve_parameters_expr(ps, expr)?;
                }
            }
        }
        _ => {
            todo!();
        }
    }
    Ok(())
}

pub fn resolve_statement(ps: &dyn Parameters, s: &mut Statement) -> R {
    match s {
        Statement::Query(query) => {
            resolve_parameters_query(ps, query)?;
        }
        Statement::Insert(insert) => {
            if let Some(ref mut source) = insert.source {
                resolve_parameters_query(ps, source)?;
            }
        }
        Statement::Update {
            table: _,
            assignments,
            from: _,
            selection: Some(expr),
            returning: _,
        } => {
            for x in assignments.iter_mut() {
                resolve_parameters_expr(ps, &mut x.value)?;
            }
            resolve_parameters_expr(ps, expr)?;
        }
        Statement::Delete(delete) => {
            if let Some(ref mut expr) = delete.selection {
                resolve_parameters_expr(ps, expr)?;
            }
        }
        Statement::CreateTable(create_table) => {
            if let Some(ref mut query_boxed) = create_table.query {
                let query = query_boxed.as_mut();
                resolve_parameters_query(ps, query)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn resolve_all(ps: &dyn Parameters, s: &mut Vec<Statement>) -> R {
    let result: ControlFlow<Error, ()> =
        visit_statements_mut(s, |stmt| match resolve_statement(ps, stmt) {
            Ok(_) => ControlFlow::Continue(()),
            Err(e) => ControlFlow::Break(e),
        });
    if let ControlFlow::Break(e) = result {
        return Err(e);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{resolve_all, ParameterSet};
    use sqlparser::{dialect::GenericDialect, parser::Parser};

    #[test]
    fn sql_parsing_resolving() {
        let sql = "create table test(x int, y int, title varchar);
insert into test (x, y, title) values($1, $2, $3);
select $1 px, t.* from test t;";
        let dialect = GenericDialect {};
        let mut rs = Parser::parse_sql(&dialect, sql).unwrap();

        let mut ps = ParameterSet::default();
        ps.add(123.into());
        ps.add(456.into());
        ps.add("Hell!".into());

        resolve_all(&ps, &mut rs).unwrap();
        for statement in rs.iter() {
            println!("{}", statement);
        }
    }
}
