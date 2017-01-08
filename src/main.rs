extern crate iron;
extern crate router;
extern crate time;
extern crate rusqlite;

use iron::prelude::*;
use iron::status;
use router::Router;
use std::hash::{Hash, SipHasher, Hasher};
use rusqlite::Connection;

// Could probably use environment variable or config.
const DB_PATH: &'static str = "./db.sqlite3";

fn create_user(username: &str, userhash: u64) {
    let db = Connection::open(DB_PATH).unwrap();
    if db.query_row("SELECT 1 FROM accounts WHERE username = ?", &[&username],
                    |_| {}).is_err() {
        // Create a user in database if not exists.
        db.execute("INSERT INTO accounts (username, balance, secret)
                    VALUES (?1, 100, ?2)",
                   &[&username, &userhash.to_string()]).unwrap();
        println!("Create user: {} with default balance 100.0", username);
    }
}

fn login_handler(req: &mut Request) -> IronResult<Response> {
    let ref username = req.extensions.get::<Router>().unwrap().find("username").unwrap_or("/");
    // TODO: use the hash_map::DefaultHasher
    let mut s = SipHasher::new();
    username.hash(&mut s);
    let userhash = s.finish();

    create_user(username, userhash);

    Ok(Response::with((status::Ok, format!("Login with {}. secret={}", username, userhash))))
}

fn index_handler(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, format!("Welcome to Crab bank"))))
}

fn init_db() {
    let db = Connection::open(DB_PATH).unwrap();
    db.execute("CREATE TABLE IF NOT EXISTS accounts (
                id       INTEGER PRIMARY KEY,
                username TEXT NOT NULL,
                balance  REAL NOT NULL,
                secret   INTEGER NOT NULL
                )", &[]).unwrap();
    println!("Initialize accounts table.");
}

fn get_balance_by_secret(secret: &str) -> rusqlite::Result<f64> {
    let db = Connection::open(DB_PATH).unwrap();
    db.query_row("SELECT balance FROM accounts WHERE secret = ?", &[&secret],
                 |row| { row.get::<_, f64>(0) })
}

fn set_balance_by_secret(secret: &str, balance: f64) {
    let db = Connection::open(DB_PATH).unwrap();
    db.execute("UPDATE accounts SET balance = ?1 WHERE secret = ?2",
               &[&balance, &secret]).unwrap();
    println!("Update user {}'s balance to {}", secret, balance);
}

fn get_balance_by_username(username: &str) -> rusqlite::Result<f64> {
    let db = Connection::open(DB_PATH).unwrap();
    db.query_row("SELECT balance FROM accounts WHERE username = ?", &[&username],
                 |row| { row.get::<_, f64>(0) })
}

fn set_balance_by_username(username: &str, balance: f64) {
    let db = Connection::open(DB_PATH).unwrap();
    db.execute("UPDATE accounts SET balance = ?1 WHERE username = ?2",
               &[&balance, &username]).unwrap();
    println!("Update user {}'s balance to {}", username, balance);
}

fn balance_handler(req: &mut Request) -> IronResult<Response> {
    let secret = req.extensions.get::<Router>().unwrap().find("secret").unwrap_or("/");
    match get_balance_by_secret(secret) {
        Ok(balance) =>
            Ok(Response::with((status::Ok, format!("Your balance is {}.", balance)))),
        Err(_) =>
            Ok(Response::with((status::Ok, "Invalid secret."))),
    }
}

fn transfer_handler(req: &mut Request) -> IronResult<Response> {
    let router = req.extensions.get::<Router>().unwrap();
    let secret = router.find("secret").unwrap();
    let receiver = router.find("receiver").unwrap();
    let amount: f64 = router.find("amount").unwrap().parse().unwrap();
    if let Ok(sender_balance) = get_balance_by_secret(secret) {
        if let Ok(receiver_balance) = get_balance_by_username(receiver) {
            set_balance_by_secret(secret, sender_balance - amount);
            set_balance_by_username(receiver, receiver_balance + amount);
            Ok(Response::with((status::Ok, "Transfer complete.")))
        } else {
            Ok(Response::with((status::Ok, "User doesn't exist.")))
        }
    } else {
        Ok(Response::with((status::Ok, "Invalid secret.")))
    }
}

fn main() {
    init_db();

    let mut router = Router::new();
    router.get("/", index_handler, "index");
    router.get("/login/:username", login_handler, "username");
    router.get("/balance/:secret", balance_handler, "balance");

    // TODO: use post instead of get.
    router.get("/transfer/:secret/:receiver/:amount", transfer_handler, "transfer");

    //TODO: handle 404

    Iron::new(router).http("localhost:3000").unwrap();
}

