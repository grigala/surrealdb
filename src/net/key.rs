use crate::dbs::DB;
use crate::err::Error;
use crate::net::input::bytes_to_utf8;
use crate::net::output;
use crate::net::params::Params;
use axum::extract::{DefaultBodyLimit, Path, Query};
use axum::response::IntoResponse;
use axum::routing::options;
use axum::{Extension, Router, TypedHeader};
use bytes::Bytes;
use http_body::Body as HttpBody;
use serde::Deserialize;
use std::str;
use surrealdb::dbs::Session;
use surrealdb::sql::Value;
use tower_http::limit::RequestBodyLimitLayer;

use super::headers::Accept;

const MAX: usize = 1024 * 16; // 16 KiB

#[derive(Default, Deserialize, Debug, Clone)]
struct QueryOptions {
	pub limit: Option<String>,
	pub start: Option<String>,
}

pub(super) fn router<S, B>() -> Router<S, B>
where
	B: HttpBody + Send + 'static,
	B::Data: Send,
	B::Error: std::error::Error + Send + Sync + 'static,
	S: Clone + Send + Sync + 'static,
{
	Router::new()
		.route(
			"/key/:table",
			options(|| async {})
				.get(select_all)
				.post(create_all)
				.put(update_all)
				.patch(modify_all)
				.delete(delete_all),
		)
		.route_layer(DefaultBodyLimit::disable())
		.layer(RequestBodyLimitLayer::new(MAX))
		.merge(
			Router::new()
				.route(
					"/key/:table/:key",
					options(|| async {})
						.get(select_one)
						.post(create_one)
						.put(update_one)
						.patch(modify_one)
						.delete(delete_one),
				)
				.route_layer(DefaultBodyLimit::disable())
				.layer(RequestBodyLimitLayer::new(MAX)),
		)
}

// ------------------------------
// Routes for a table
// ------------------------------

async fn select_all(
	Extension(session): Extension<Session>,
	maybe_output: Option<TypedHeader<Accept>>,
	Path(table): Path<String>,
	Query(query): Query<QueryOptions>,
) -> Result<impl IntoResponse, impl IntoResponse> {
	// Get the datastore reference
	let db = DB.get().unwrap();
	// Specify the request statement
	let sql = format!(
		"SELECT * FROM type::table($table) LIMIT {l} START {s}",
		l = query.limit.unwrap_or_else(|| String::from("100")),
		s = query.start.unwrap_or_else(|| String::from("0")),
	);
	// Specify the request variables
	let vars = map! {
		String::from("table") => Value::from(table),
	};
	// Execute the query and return the result
	match db.execute(sql.as_str(), &session, Some(vars)).await {
		Ok(ref res) => match maybe_output.as_deref() {
			// Simple serialization
			Some(Accept::ApplicationJson) => Ok(output::json(&output::simplify(res))),
			Some(Accept::ApplicationCbor) => Ok(output::cbor(&output::simplify(res))),
			Some(Accept::ApplicationPack) => Ok(output::pack(&output::simplify(res))),
			// Internal serialization
			Some(Accept::Surrealdb) => Ok(output::full(&res)),
			// An incorrect content-type was requested
			_ => Err(Error::InvalidType),
		},
		// There was an error when executing the query
		Err(err) => Err(Error::from(err)),
	}
}

async fn create_all(
	Extension(session): Extension<Session>,
	maybe_output: Option<TypedHeader<Accept>>,
	Path(table): Path<String>,
	Query(params): Query<Params>,
	body: Bytes,
) -> Result<impl IntoResponse, impl IntoResponse> {
	// Get the datastore reference
	let db = DB.get().unwrap();
	// Convert the HTTP request body
	let data = bytes_to_utf8(&body)?;
	// Parse the request body as JSON
	match surrealdb::sql::value(data) {
		Ok(data) => {
			// Specify the request statement
			let sql = "CREATE type::table($table) CONTENT $data";
			// Specify the request variables
			let vars = map! {
				String::from("table") => Value::from(table),
				String::from("data") => data,
				=> params.parse()
			};
			// Execute the query and return the result
			match db.execute(sql, &session, Some(vars)).await {
				Ok(res) => match maybe_output.as_deref() {
					// Simple serialization
					Some(Accept::ApplicationJson) => Ok(output::json(&output::simplify(res))),
					Some(Accept::ApplicationCbor) => Ok(output::cbor(&output::simplify(res))),
					Some(Accept::ApplicationPack) => Ok(output::pack(&output::simplify(res))),
					// Internal serialization
					Some(Accept::Surrealdb) => Ok(output::full(&res)),
					// An incorrect content-type was requested
					_ => Err(Error::InvalidType),
				},
				// There was an error when executing the query
				Err(err) => Err(Error::from(err)),
			}
		}
		Err(_) => Err(Error::Request),
	}
}

async fn update_all(
	Extension(session): Extension<Session>,
	maybe_output: Option<TypedHeader<Accept>>,
	Path(table): Path<String>,
	Query(params): Query<Params>,
	body: Bytes,
) -> Result<impl IntoResponse, impl IntoResponse> {
	// Get the datastore reference
	let db = DB.get().unwrap();
	// Convert the HTTP request body
	let data = bytes_to_utf8(&body)?;
	// Parse the request body as JSON
	match surrealdb::sql::value(data) {
		Ok(data) => {
			// Specify the request statement
			let sql = "UPDATE type::table($table) CONTENT $data";
			// Specify the request variables
			let vars = map! {
				String::from("table") => Value::from(table),
				String::from("data") => data,
				=> params.parse()
			};
			// Execute the query and return the result
			match db.execute(sql, &session, Some(vars)).await {
				Ok(res) => match maybe_output.as_deref() {
					// Simple serialization
					Some(Accept::ApplicationJson) => Ok(output::json(&output::simplify(res))),
					Some(Accept::ApplicationCbor) => Ok(output::cbor(&output::simplify(res))),
					Some(Accept::ApplicationPack) => Ok(output::pack(&output::simplify(res))),
					// Internal serialization
					Some(Accept::Surrealdb) => Ok(output::full(&res)),
					// An incorrect content-type was requested
					_ => Err(Error::InvalidType),
				},
				// There was an error when executing the query
				Err(err) => Err(Error::from(err)),
			}
		}
		Err(_) => Err(Error::Request),
	}
}

async fn modify_all(
	Extension(session): Extension<Session>,
	maybe_output: Option<TypedHeader<Accept>>,
	Path(table): Path<String>,
	Query(params): Query<Params>,
	body: Bytes,
) -> Result<impl IntoResponse, impl IntoResponse> {
	// Get the datastore reference
	let db = DB.get().unwrap();
	// Convert the HTTP request body
	let data = bytes_to_utf8(&body)?;
	// Parse the request body as JSON
	match surrealdb::sql::value(data) {
		Ok(data) => {
			// Specify the request statement
			let sql = "UPDATE type::table($table) MERGE $data";
			// Specify the request variables
			let vars = map! {
				String::from("table") => Value::from(table),
				String::from("data") => data,
				=> params.parse()
			};
			// Execute the query and return the result
			match db.execute(sql, &session, Some(vars)).await {
				Ok(res) => match maybe_output.as_deref() {
					// Simple serialization
					Some(Accept::ApplicationJson) => Ok(output::json(&output::simplify(res))),
					Some(Accept::ApplicationCbor) => Ok(output::cbor(&output::simplify(res))),
					Some(Accept::ApplicationPack) => Ok(output::pack(&output::simplify(res))),
					// Internal serialization
					Some(Accept::Surrealdb) => Ok(output::full(&res)),
					// An incorrect content-type was requested
					_ => Err(Error::InvalidType),
				},
				// There was an error when executing the query
				Err(err) => Err(Error::from(err)),
			}
		}
		Err(_) => Err(Error::Request),
	}
}

async fn delete_all(
	Extension(session): Extension<Session>,
	maybe_output: Option<TypedHeader<Accept>>,
	Path(table): Path<String>,
	Query(params): Query<Params>,
) -> Result<impl IntoResponse, impl IntoResponse> {
	// Get the datastore reference
	let db = DB.get().unwrap();
	// Specify the request statement
	let sql = "DELETE type::table($table) RETURN BEFORE";
	// Specify the request variables
	let vars = map! {
		String::from("table") => Value::from(table),
		=> params.parse()
	};
	// Execute the query and return the result
	match db.execute(sql, &session, Some(vars)).await {
		Ok(res) => match maybe_output.as_deref() {
			// Simple serialization
			Some(Accept::ApplicationJson) => Ok(output::json(&output::simplify(res))),
			Some(Accept::ApplicationCbor) => Ok(output::cbor(&output::simplify(res))),
			Some(Accept::ApplicationPack) => Ok(output::pack(&output::simplify(res))),
			// Internal serialization
			Some(Accept::Surrealdb) => Ok(output::full(&res)),
			// An incorrect content-type was requested
			_ => Err(Error::InvalidType),
		},
		// There was an error when executing the query
		Err(err) => Err(Error::from(err)),
	}
}

// ------------------------------
// Routes for a thing
// ------------------------------

async fn select_one(
	Extension(session): Extension<Session>,
	maybe_output: Option<TypedHeader<Accept>>,
	Path((table, id)): Path<(String, String)>,
) -> Result<impl IntoResponse, impl IntoResponse> {
	// Get the datastore reference
	let db = DB.get().unwrap();
	// Specify the request statement
	let sql = "SELECT * FROM type::thing($table, $id)";
	// Parse the Record ID as a SurrealQL value
	let rid = match surrealdb::sql::json(&id) {
		Ok(id) => id,
		Err(_) => Value::from(id),
	};
	// Specify the request variables
	let vars = map! {
		String::from("table") => Value::from(table),
		String::from("id") => rid,
	};
	// Execute the query and return the result
	match db.execute(sql, &session, Some(vars)).await {
		Ok(res) => match maybe_output.as_deref() {
			// Simple serialization
			Some(Accept::ApplicationJson) => Ok(output::json(&output::simplify(res))),
			Some(Accept::ApplicationCbor) => Ok(output::cbor(&output::simplify(res))),
			Some(Accept::ApplicationPack) => Ok(output::pack(&output::simplify(res))),
			// Internal serialization
			Some(Accept::Surrealdb) => Ok(output::full(&res)),
			// An incorrect content-type was requested
			_ => Err(Error::InvalidType),
		},
		// There was an error when executing the query
		Err(err) => Err(Error::from(err)),
	}
}

async fn create_one(
	Extension(session): Extension<Session>,
	maybe_output: Option<TypedHeader<Accept>>,
	Query(params): Query<Params>,
	Path((table, id)): Path<(String, String)>,
	body: Bytes,
) -> Result<impl IntoResponse, impl IntoResponse> {
	// Get the datastore reference
	let db = DB.get().unwrap();
	// Convert the HTTP request body
	let data = bytes_to_utf8(&body)?;
	// Parse the Record ID as a SurrealQL value
	let rid = match surrealdb::sql::json(&id) {
		Ok(id) => id,
		Err(_) => Value::from(id),
	};
	// Parse the request body as JSON
	match surrealdb::sql::value(data) {
		Ok(data) => {
			// Specify the request statement
			let sql = "CREATE type::thing($table, $id) CONTENT $data";
			// Specify the request variables
			let vars = map! {
				String::from("table") => Value::from(table),
				String::from("id") => rid,
				String::from("data") => data,
				=> params.parse()
			};
			// Execute the query and return the result
			match db.execute(sql, &session, Some(vars)).await {
				Ok(res) => match maybe_output.as_deref() {
					// Simple serialization
					Some(Accept::ApplicationJson) => Ok(output::json(&output::simplify(res))),
					Some(Accept::ApplicationCbor) => Ok(output::cbor(&output::simplify(res))),
					Some(Accept::ApplicationPack) => Ok(output::pack(&output::simplify(res))),
					// Internal serialization
					Some(Accept::Surrealdb) => Ok(output::full(&res)),
					// An incorrect content-type was requested
					_ => Err(Error::InvalidType),
				},
				// There was an error when executing the query
				Err(err) => Err(Error::from(err)),
			}
		}
		Err(_) => Err(Error::Request),
	}
}

async fn update_one(
	Extension(session): Extension<Session>,
	maybe_output: Option<TypedHeader<Accept>>,
	Query(params): Query<Params>,
	Path((table, id)): Path<(String, String)>,
	body: Bytes,
) -> Result<impl IntoResponse, impl IntoResponse> {
	// Get the datastore reference
	let db = DB.get().unwrap();
	// Convert the HTTP request body
	let data = bytes_to_utf8(&body)?;
	// Parse the Record ID as a SurrealQL value
	let rid = match surrealdb::sql::json(&id) {
		Ok(id) => id,
		Err(_) => Value::from(id),
	};
	// Parse the request body as JSON
	match surrealdb::sql::value(data) {
		Ok(data) => {
			// Specify the request statement
			let sql = "UPDATE type::thing($table, $id) CONTENT $data";
			// Specify the request variables
			let vars = map! {
				String::from("table") => Value::from(table),
				String::from("id") => rid,
				String::from("data") => data,
				=> params.parse()
			};
			// Execute the query and return the result
			match db.execute(sql, &session, Some(vars)).await {
				Ok(res) => match maybe_output.as_deref() {
					// Simple serialization
					Some(Accept::ApplicationJson) => Ok(output::json(&output::simplify(res))),
					Some(Accept::ApplicationCbor) => Ok(output::cbor(&output::simplify(res))),
					Some(Accept::ApplicationPack) => Ok(output::pack(&output::simplify(res))),
					// Internal serialization
					Some(Accept::Surrealdb) => Ok(output::full(&res)),
					// An incorrect content-type was requested
					_ => Err(Error::InvalidType),
				},
				// There was an error when executing the query
				Err(err) => Err(Error::from(err)),
			}
		}
		Err(_) => Err(Error::Request),
	}
}

async fn modify_one(
	Extension(session): Extension<Session>,
	maybe_output: Option<TypedHeader<Accept>>,
	Query(params): Query<Params>,
	Path((table, id)): Path<(String, String)>,
	body: Bytes,
) -> Result<impl IntoResponse, impl IntoResponse> {
	// Get the datastore reference
	let db = DB.get().unwrap();
	// Convert the HTTP request body
	let data = bytes_to_utf8(&body)?;
	// Parse the Record ID as a SurrealQL value
	let rid = match surrealdb::sql::json(&id) {
		Ok(id) => id,
		Err(_) => Value::from(id),
	};
	// Parse the request body as JSON
	match surrealdb::sql::value(data) {
		Ok(data) => {
			// Specify the request statement
			let sql = "UPDATE type::thing($table, $id) MERGE $data";
			// Specify the request variables
			let vars = map! {
				String::from("table") => Value::from(table),
				String::from("id") => rid,
				String::from("data") => data,
				=> params.parse()
			};
			// Execute the query and return the result
			match db.execute(sql, &session, Some(vars)).await {
				Ok(res) => match maybe_output.as_deref() {
					// Simple serialization
					Some(Accept::ApplicationJson) => Ok(output::json(&output::simplify(res))),
					Some(Accept::ApplicationCbor) => Ok(output::cbor(&output::simplify(res))),
					Some(Accept::ApplicationPack) => Ok(output::pack(&output::simplify(res))),
					// Internal serialization
					Some(Accept::Surrealdb) => Ok(output::full(&res)),
					// An incorrect content-type was requested
					_ => Err(Error::InvalidType),
				},
				// There was an error when executing the query
				Err(err) => Err(Error::from(err)),
			}
		}
		Err(_) => Err(Error::Request),
	}
}

async fn delete_one(
	Extension(session): Extension<Session>,
	maybe_output: Option<TypedHeader<Accept>>,
	Path((table, id)): Path<(String, String)>,
) -> Result<impl IntoResponse, impl IntoResponse> {
	// Get the datastore reference
	let db = DB.get().unwrap();
	// Specify the request statement
	let sql = "DELETE type::thing($table, $id) RETURN BEFORE";
	// Parse the Record ID as a SurrealQL value
	let rid = match surrealdb::sql::json(&id) {
		Ok(id) => id,
		Err(_) => Value::from(id),
	};
	// Specify the request variables
	let vars = map! {
		String::from("table") => Value::from(table),
		String::from("id") => rid,
	};
	// Execute the query and return the result
	match db.execute(sql, &session, Some(vars)).await {
		Ok(res) => match maybe_output.as_deref() {
			// Simple serialization
			Some(Accept::ApplicationJson) => Ok(output::json(&output::simplify(res))),
			Some(Accept::ApplicationCbor) => Ok(output::cbor(&output::simplify(res))),
			Some(Accept::ApplicationPack) => Ok(output::pack(&output::simplify(res))),
			// Internal serialization
			Some(Accept::Surrealdb) => Ok(output::full(&res)),
			// An incorrect content-type was requested
			_ => Err(Error::InvalidType),
		},
		// There was an error when executing the query
		Err(err) => Err(Error::from(err)),
	}
}
