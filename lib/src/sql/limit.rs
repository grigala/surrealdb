use crate::ctx::Context;
use crate::dbs::{Options, Transaction};
use crate::doc::CursorDoc;
use crate::err::Error;
use crate::sql::comment::shouldbespace;
use crate::sql::error::IResult;
use crate::sql::number::Number;
use crate::sql::value::{value, Value};
use nom::bytes::complete::tag_no_case;
use nom::combinator::opt;
use nom::sequence::tuple;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Serialize, Deserialize, Hash)]
pub struct Limit(pub Value);

impl Limit {
	pub(crate) async fn process(
		&self,
		ctx: &Context<'_>,
		opt: &Options,
		txn: &Transaction,
		doc: Option<&CursorDoc<'_>>,
	) -> Result<usize, Error> {
		match self.0.compute(ctx, opt, txn, doc).await {
			// This is a valid limiting number
			Ok(Value::Number(Number::Int(v))) if v >= 0 => Ok(v as usize),
			// An invalid value was specified
			Ok(v) => Err(Error::InvalidLimit {
				value: v.as_string(),
			}),
			// A different error occured
			Err(e) => Err(e),
		}
	}
}

impl fmt::Display for Limit {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "LIMIT {}", self.0)
	}
}

pub fn limit(i: &str) -> IResult<&str, Limit> {
	let (i, _) = tag_no_case("LIMIT")(i)?;
	let (i, _) = opt(tuple((shouldbespace, tag_no_case("BY"))))(i)?;
	let (i, _) = shouldbespace(i)?;
	let (i, v) = value(i)?;
	Ok((i, Limit(v)))
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn limit_statement() {
		let sql = "LIMIT 100";
		let res = limit(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!(out, Limit(Value::from(100)));
		assert_eq!("LIMIT 100", format!("{}", out));
	}

	#[test]
	fn limit_statement_by() {
		let sql = "LIMIT BY 100";
		let res = limit(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!(out, Limit(Value::from(100)));
		assert_eq!("LIMIT 100", format!("{}", out));
	}
}
