#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate diesel;

mod schema;
use rocket::request::Request;
use rocket_contrib::databases::diesel::PgConnection;
use rocket_contrib::json::Json;
use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[post("/create-user", format = "json", data = "<input>")]
fn create(input: Json<InsertablePerson>, db_conn: RustyDbConn) -> Json<Person> {
	Json(InsertablePerson::new(
		input.first_name,
		input.last_name,
		input.age,
		input.profession,
		input.salary,
		&db_conn.0
	))
}

#[get("/")]
fn index(_db_conn: RustyDbConn) -> &'static str {
	"Hello, from Rust! (with a database connection!)"
}

#[catch(503)]
fn service_not_available(_req: &Request) -> &'static str {
	"Service is not available. (Is the database up?)"
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug, Serialize, Deserialize)]
#[table_name = "people"]
pub struct Person {
	pub id: i32,
	pub first_name: String,
	pub last_name: String,
	pub age: i32,
	pub profession: String,
	pub salary: i32,
}

use schema::people;

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "people"]
pub struct InsertablePerson<'a> {
	pub first_name: &'a str,
	pub last_name: &'a str,
	pub age: i32,
	pub profession: &'a str,
	pub salary: i32,
}

impl<'a> InsertablePerson<'a> {
	fn new(
		first_name: &'a str,
		last_name: &'a str,
		age: i32,
		profession: &'a str,
		salary: i32, conn: &PgConnection
	) -> Person {
		use schema::people;

		let person = Self {
			first_name,
			last_name,
			age,
			profession,
			salary
		};

		diesel::insert_into(people::table)
			.values(&person)
			.get_result(conn)
			.expect("Could not make a user")
	}
}

#[database("rustydb")]
struct RustyDbConn(PgConnection);

fn main() {
	rocket::ignite()
		.attach(RustyDbConn::fairing())
		.register(catchers![service_not_available])
		.mount("/", routes![index, create])
		.launch();
}

#[cfg(test)]
mod test {
	use rocket::local::Client;
	use rocket::http::{ContentType, Status};
	use rocket_contrib::json::Json;
	use super::*;

	#[test]
	fn test_index() {
		let rocket = rocket::ignite()
			.attach(RustyDbConn::fairing())
			.register(catchers![service_not_available])
			.mount("/", routes![index, create]);
		let client = Client::new(rocket).expect("valid rocket instance");
		let mut response = client.get("/").dispatch();
		assert_eq!(response.body_string(), Some("Hello, from Rust! (with a database connection!)".into()))
	}

	#[test]
	fn test_create_user() {
		let rocket = rocket::ignite()
			.attach(RustyDbConn::fairing())
			.register(catchers![service_not_available])
			.mount("/", routes![index, create]);
		let client = Client::new(rocket).expect("valid rocket instance");
		let mut response = client.post("/create-user").header(ContentType::JSON)
			.body(r##"{
						"first_name": "Will",
						"last_name": "Lane",
						"age": 14,
						"profession": "Coder",
						"salary": 100
						}"##).dispatch();
		let response_body = response.body_string().expect("Response Body");
		let person: Person = serde_json::from_str(&response_body.as_str()).expect("Valid User Response");
		assert!(person.id.is_positive());
		assert_eq!(person.first_name, String::from("Will"));
		assert_eq!(person.last_name, String::from("Lane"));
		assert_eq!(person.age, 14);
		assert_eq!(person.profession, String::from("Coder"));
		assert_eq!(person.salary, 100);
	}
}