#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
use diesel::{prelude::*, table, Insertable, Queryable};
use rocket::{fairing::AdHoc, serde::json::Json, State};
use rocket_sync_db_pools::database;
use serde::{Deserialize, Serialize};

table! {
    users (id) {
        id -> Int4,
        first_name -> Varchar,
        last_name -> Varchar,
        email -> Varchar,
    }
}

#[database("my_db")]
pub struct Db(diesel::PgConnection);

#[derive(Serialize, Deserialize, Clone, Queryable, Debug, Insertable)]
#[table_name = "users"]
struct User {
    id: i32,
    first_name: String,
    last_name: String,
    email: String,
}

#[derive(Deserialize)]
struct Config {
    name: String,
}

#[get("/<id>")]
async fn get_user(connection: Db, id: i32) -> Json<User> {
    connection
        .run(move |c| users::table.filter(users::id.eq(id)).first(c))
        .await
        .map(Json)
        .expect("Failed to find user")
}

#[get("/")]
async fn get_all_users(connection: Db) -> Json<Vec<User>> {
    connection
        .run(|c| users::table.load(c))
        .await
        .map(Json)
        .expect("Failed to get all users")
}

#[post("/", data = "<user>")]
async fn create_user(connection: Db, user: Json<User>) -> Json<User> {
    connection.run(move |c| {
        diesel::insert_into(users::table)
        .values(&user.into_inner())
        .get_result(c)
    })
    .await
    .map(Json)
    .expect("failed to enter user into DB")
}


#[launch]
fn rocket() -> _ {
    let rocket = rocket::build();

    rocket
    .attach(Db::fairing())
    .attach(AdHoc::config::<Config>())
    .mount("/user",
           routes![
               get_random_user,
               get_user,
               get_all_users,
               create_user
           ])
}