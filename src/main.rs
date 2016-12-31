extern crate iron;
extern crate router;
extern crate time;

use iron::prelude::*;
use iron::status;
use router::Router;
use std::hash::{Hash, SipHasher, Hasher};
use std::collections::HashMap;

fn login_handler(req: &mut Request) -> IronResult<Response> {
    let ref username = req.extensions.get::<Router>().unwrap().find("username").unwrap_or("/");
    // TODO: use the hash_map::DefaultHasher
    let mut s = SipHasher::new();
    username.hash(&mut s);
    let userhash = s.finish();
    Ok(Response::with((status::Ok, format!("Login with {}. secret={}", username, userhash))))
}

fn index_handler(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, format!("Welcome to Crab bank"))))
}

fn main() {
    let mut router = Router::new();
    router.get("/", index_handler, "index");
    router.get("/login/:username", login_handler, "username");

    // Use closure to avoid global mutable hash map, maybe there's a better way.
    let balances: HashMap<String, u64> = HashMap::new();
    let balance_handler = move |req : &mut Request| -> IronResult<Response> {
        let secret = req.extensions.get::<Router>().unwrap().find("secret").unwrap_or("/");
        let balance = match balances.get(secret) {
            None => 0,
            Some(&x) => x,
        };
        Ok(Response::with((status::Ok, format!("Your balance is {}.", balance))))
    };
    router.get("/balance/:secret", balance_handler, "balance");
    //TODO: handle 404

    Iron::new(router).http("localhost:3000").unwrap();
}

