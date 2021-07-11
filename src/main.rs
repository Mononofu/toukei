#[macro_use]
extern crate rocket;

use rocket::http::hyper::header;
use rocket::http::ContentType;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::{self, Responder, Response};
use rocket::State;
use std::net::SocketAddr;

#[get("/")]
fn index() -> &'static str {
    "Please see https://github.com/Mononofu/toukei"
}

// A responder that returns an empty 1x1.png and disallows caching.
struct PixelResponder {}

const EMPTY_PNG: &[u8] = include_bytes!("1x1.png");

impl<'r> Responder<'r, 'static> for PixelResponder {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .header(ContentType::new("image", "png"))
            .raw_header(header::CACHE_CONTROL.as_str(), "no-store, max-age=0")
            .sized_body(EMPTY_PNG.len(), std::io::Cursor::new(EMPTY_PNG))
            .ok()
    }
}

#[derive(Debug)]
struct UserAgent(Option<String>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAgent {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(UserAgent(
            req.headers().get_one("user-agent").map(|s| s.to_owned()),
        ))
    }
}

#[get("/visit.png?<url>&<referrer>&<bot>")]
fn visit_pixel(
    url: &str,
    referrer: Option<&str>,
    bot: Option<&str>,
    ua: UserAgent,
    addr: SocketAddr,
    ip_reader: &State<maxminddb::Reader<Vec<u8>>>,
) -> PixelResponder {
    handle_visit(url, referrer, bot, ua, addr, ip_reader);
    PixelResponder {}
}

#[post("/visit?<url>&<referrer>&<bot>")]
fn visit_beacon(
    url: &str,
    referrer: Option<&str>,
    bot: Option<&str>,
    ua: UserAgent,
    addr: SocketAddr,
    ip_reader: &State<maxminddb::Reader<Vec<u8>>>,
) -> PixelResponder {
    handle_visit(url, referrer, bot, ua, addr, ip_reader);
    PixelResponder {}
}

#[post("/exit?<url>&<referrer>&<bot>")]
fn exit_beacon(
    url: &str,
    referrer: Option<&str>,
    bot: Option<&str>,
    ua: UserAgent,
    addr: SocketAddr,
    ip_reader: &State<maxminddb::Reader<Vec<u8>>>,
) -> PixelResponder {
    handle_visit(url, referrer, bot, ua, addr, ip_reader);
    PixelResponder {}
}

fn handle_visit(
    url: &str,
    referrer: Option<&str>,
    bot: Option<&str>,
    ua: UserAgent,
    addr: SocketAddr,
    ip_reader: &State<maxminddb::Reader<Vec<u8>>>,
) {
    let city: Option<maxminddb::geoip2::City> = ip_reader.lookup(addr.ip()).unwrap_or(None);
    println!(
        "got url {} from {} ({:?}) {:?}, coming from {:?} - bot {:?}",
        url, addr, city, ua, referrer, bot
    );
}

#[launch]
fn rocket() -> _ {
    let rocket = rocket::build();
    let figment = rocket.figment();

    let maxminddb_path: String = figment.extract_inner("maxminddb").expect("config");
    let reader = maxminddb::Reader::open_readfile(maxminddb_path).unwrap();

    let rocket = match figment.extract_inner::<String>("static_dir") {
        Ok(d) => rocket.mount("/", rocket::fs::FileServer::from(d)),
        _ => rocket,
    };

    rocket
        .manage(reader)
        .mount("/", routes![index, visit_pixel, visit_beacon, exit_beacon])
}
