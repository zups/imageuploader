extern crate tiny_http;
use tiny_http::{Response, Request, Server, Method};
use std::path::Path;
use std::fs::File;
use std::io::Write;
use std::str::from_utf8;

fn get_file(path: String) -> Result<File, String> {
    let file = File::open(&Path::new(&path));
    let file = match file {
        Ok(file) => file,
        Err(_error) => {
            return Err(String::from("Not found"))
        }
    };
    Ok(file)
}

fn index(request: Request) {
    let index = get_file(String::from("index.html"));
    request.respond(Response::from_file(index.unwrap())).expect("failed");
}

fn picture(request: Request, path: &str) {
    match get_file(format!("files/{}", path)) {
        Ok(file) => request.respond(Response::from_file(file)),
        Err(err) => request.respond(Response::from_string(err)),
    };
}

fn get_handler(request: Request) {
    match request.url() {
        "/" => {
            index(request);
        },
        _ => {
            let path = request.url().replace("/", "");
            picture(request, path.as_str());
        },
    };
}
//split_at(index)
fn parse_multipart(body: &mut Vec<u8>) -> (Vec<u8>, &str) {
    let mut times = 0;
    let mut index = 0;
    let newline: u8 = 10;
    for element in body.iter() {
        if element == &newline {
            times += 1;
            if times == 4 {
                break;
            }
        }
        index += 1;
    }

    let (headers, body) = body.split_at(index+1);

    index = 0;
    times = 0;

    for element in body.iter().rev() {
        if element == &newline {
            times += 1;
            if times == 6 {
                break;
            }
        }
        index += 1;
    }

    let (body, left) = body.split_at(body.len() - (index+2));

    (body.to_vec(), from_utf8(headers).unwrap().trim())
}

fn parse_filename(headers: &str) -> &str {
    let mut index = headers.find("filename=\"");

    let (_, rest) = headers.split_at(index.unwrap() + 10);

    index = rest.find("\n");

    let (filename, _) = rest.split_at(index.unwrap()-2);

    filename.trim()
}

fn post_handler(mut request: Request) {
    let mut buffer = Vec::new();
    request.as_reader().read_to_end(&mut buffer);
    let (body, headers) = parse_multipart(&mut buffer);

    let filename = parse_filename(headers);

    let path = format!("files/{}", filename);
    let mut file = File::create(path);

    file.unwrap().write_all(&body[..]);
    
    picture(request, filename);
}

fn main() {
    let server = Server::http("0.0.0.0:8080").unwrap();

    for request in server.incoming_requests() {
        println!("(Path: {}\n From: {})", request.url(), request.remote_addr() );
       match request.method() {
          Method::Get => get_handler(request), 
          Method::Post => post_handler(request),
          _ => (),
       }; 
    }
}
