use actix_web::{ get, http::header, post, web, App, HttpResponse, HttpServer, ResponseError };
use askama::Template;
use thiserror::Error;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

use serde::Deserialize;

pub(crate) use env_logger;
// use actix_web::middleware::Logger;


#[derive(Deserialize)]
struct AddParams {
	text: String,
}

#[derive(Deserialize)]
struct DeleteParams {
	id: u32,
}

struct TodoEntry {
	id: u32,
	text: String,
}


#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
	entries: Vec<TodoEntry>,
}


#[derive(Error, Debug)]
enum MyError  {
	#[error("Failed to render HTML")]
	AskamaError(#[from] askama::Error),

	#[error("Failed to get connection")]
	ConnectionPoolError(#[from] r2d2::Error),

	#[error("Failed SQL execution")]
	SQLiteError(#[from] rusqlite::Error),
}


impl ResponseError for MyError {}


#[post("/add")]
async fn add_todo(
	params: web::Form<AddParams>,
	db: web::Data<r2d2::Pool<SqliteConnectionManager>>,
) -> Result<HttpResponse, MyError> {
	let conn = db.get()?;
	conn.execute("INSERT INTO todo (text) VALUES (?)", &[&params.text])?;
	Ok(HttpResponse::SeeOther()
		.header(header::LOCATION, "/")
		.finish())
}


#[post("/delete")]
async fn delete_todo(
	params: web::Form<DeleteParams>,
	db: web::Data<r2d2::Pool<SqliteConnectionManager>>,
) -> Result<HttpResponse, MyError> {
	let conn = db.get()?;
	conn.execute("DELETE FROM todo WHERE id = ?", [params.id])?;
	Ok(HttpResponse::SeeOther()
		.header(header::LOCATION, "/")
		.finish())
}


#[get("/")]
async fn index(db: web::Data<Pool<SqliteConnectionManager>>) -> Result<HttpResponse, MyError> {
	let conn = db.get()?;
	let mut statement = conn.prepare("SELECT id, text FROM todo")?;
	let rows = statement.query_map(params![], |row| {
		let id = row.get(0)?;
		let text = row.get(1)?;
		Ok(TodoEntry { id, text })
	})?;
	let mut entries = Vec::new();
	for row in rows {
		entries.push(row?);
	}
	// entries.push(TodoEntry{
	// 	id: 1,
	// 	text: "First entry".to_string(),
	// });

	// entries.push(TodoEntry {
	// 	id: 2,
	// 	text: "Second entry".to_string(),
	// });

	let html = IndexTemplate{ entries };

	let response_body = html.render()?;

	Ok(HttpResponse::Ok()
		.content_type("text/html")
		.body(response_body))
}


#[actix_rt::main]
async fn main() -> Result<(), actix_web::Error> {
	std::env::set_var("RUST_LOG", "info");
	env_logger::init();

	// debug!("debugです");

	let manager = SqliteConnectionManager::file("todo.db");

	let pool = Pool::new(manager).expect("Failed to initialize the connection pool.");
	let conn = pool
		.get()
		.expect("Failed to initialize the connection pool.");
	
	conn.execute(
		"CREATE TABLE IF NOT EXISTS todo (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			text TEXT NOT NULL
		)",
		params![],
		)
		.expect("Failed to create a table `todo`.");

	HttpServer::new(move || App::new()
		// .middleware(Logger::default())
		// .middleware(Logger::new("%a %{User-Agent}i"))
		.service(index)
		.service(add_todo)
		.service(delete_todo)
		.data(pool.clone()))
		.bind("0.0.0.0:8080")?
		.run()
		.await?;
	Ok(())
}
