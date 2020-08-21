extern crate tiny_http;
use tiny_http::{Response, Server};
use std::path::Path;
use std::fs::File;

fn main() {
    let server = Server::http("localhost:8080").unwrap();

    for request in server.incoming_requests() {
        let filename = request.url().replace("/", "");
        let file = File::open(&Path::new(&filename));

        let file = match file {
            Ok(file) => {
                let response = tiny_http::Response::from_file(file);
                request.respond(response);
            }
            Err(error) => {
                let response = Response::from_string("Not found");
                request.respond(response);
            }
        };
    }
}