extern crate tiny_http;
use tiny_http::{Response, Server};
use std::path::Path;
use std::fs::File;

fn main() {
    let server = Server::http("localhost:8080").unwrap();

    for request in server.incoming_requests() {
        println!(
            "received request! method: {:?}, url: {:?}, headers: {:?}",
            request.method(),
            request.url(),
            request.headers()
        );

        //let response = Response::from_string("hello world");
        let response2 = tiny_http::Response::from_file(File::open(&Path::new("image.png")).unwrap());
        request.respond(response2);
    }
}