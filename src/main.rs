#[macro_use]
extern crate rocket;

use rocket::http::hyper::header;
use rocket::http::ContentType;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

const EMPTY_PNG: &[u8] = include_bytes!("1x1.png");
struct PixelResponder {}

impl<'r> Responder<'r, 'static> for PixelResponder {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .header(ContentType::new("image", "png"))
            .raw_header(header::CACHE_CONTROL.as_str(), "no-store, max-age=0")
            .sized_body(EMPTY_PNG.len(), std::io::Cursor::new(EMPTY_PNG))
            .ok()
    }
}

#[get("/visit.png?<url>")]
fn visit(url: &str) -> PixelResponder {
    println!("got url {}", url);
    PixelResponder {}
}

#[post("/exit.png?<url>")]
fn exit(url: &str) -> PixelResponder {
    println!("exiting url {}", url);
    PixelResponder {}
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, visit, exit])
}
