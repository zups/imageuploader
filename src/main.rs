extern crate tiny_http;
use tiny_http::{Response, Request, Server, Method};
use std::path::Path;
use std::fs::File;

fn getFile(path: String) -> Result<File, String> {
    let file = File::open(&Path::new(&path));
    let file = match file {
        Ok(file) => file,
        Err(error) => {
            return Err(String::from("Not found"))
        }
    };
    Ok(file)
}

fn index(request: Request) {
    let index = getFile(String::from("index.html"));
    request.respond(Response::from_file(index.unwrap()));
}

fn picture(request: Request, path: String) {
    let file = match getFile(path) {
        Ok(file) => request.respond(Response::from_file(file)),
        Err(err) => request.respond(Response::from_string("Not found")),
    };
}

fn getHandler(request: Request) {

    let path = match request.url() {
        "/" => {
            index(request);
        },
        _ => {
            let path = request.url().replace("/", "");
            picture(request, path);
        },
    };
}

fn postHandler(request: Request) {
    request.respond(Response::from_string("posted"));
}

fn main() {
    let server = Server::http("0.0.0.0:8080").unwrap();

    for request in server.incoming_requests() {
       match request.method() {
          Method::Get => getHandler(request), 
          Method::Post => postHandler(request),
          _ => (),
       }; 
    }
}
