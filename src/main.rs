use rocket::{ launch, routes, get, post };




#[ launch ]
fn start() -> _ {

    rocket::build()
        .mount("/", routes![])

}
