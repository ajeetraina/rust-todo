use actix_web::{ get, App, HttpResponse, HttpServer, ResponseError };
use askama::Template;
use thiserror::Error;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;


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
}


impl ResponseError for MyError {}


#[get("/")]
async fn index() -> Result<HttpResponse, MyError> {
	let mut entries = Vec::new();
	entries.push(TodoEntry{
		id: 1,
		text: "First entry".to_string(),
	});

	entries.push(TodoEntry {
		id: 2,
		text: "Second entry".to_string(),
	});

	let html = IndexTemplate{ entries };

	let response_body = html.render()?;

	Ok(HttpResponse::Ok()
		.content_type("text/html")
		.body(response_body))
}


#[actix_rt::main]
async fn main() -> Result<(), actix_web::Error> {
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

	HttpServer::new(move || App::new().service(index).app_data(pool.clone()))
		.bind("0.0.0.0:8080")?
		.run()
		.await?;
	Ok(())
}
