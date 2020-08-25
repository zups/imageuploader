extern crate tiny_http;
use tiny_http::{Response, Request, Server, Method, Header, StatusCode};
use std::path::Path;
use std::fs::File;
use std::io::{self, Write};
use std::str::from_utf8;

fn get_file(path: &str) -> Result<File, String> {
    let path = path.replace("%20", " ");
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
    let index = get_file("index.html");
    request.respond(Response::from_file(index.unwrap())).expect("failed");
}

fn file_response(request: Request, path: &str) {
    let fileLocation = &format!("files/{}", path);
    match get_file(fileLocation) {
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
            file_response(request, path.as_str());
        },
    };
}

fn find_nth_newline_index<'a, I>(iter: I, nth: usize) -> usize 
where
    I: IntoIterator<Item = &'a u8>, {
        let mut times = 0;
        let mut index = 0;
        let newline: u8 = 10;

        for element in iter {
            if element == &newline {
                times += 1;
                if times == nth {
                    break;
                }
            }
            index += 1;
        }

        index
}

fn parse_multipart(body: &mut Vec<u8>) -> (Vec<u8>, &str) {
    let mut index = find_nth_newline_index(body.iter(), 4);
    let (headers, body) = body.split_at(index+1);
    
    index = find_nth_newline_index(body.iter().rev(), 6);
    let (body, _) = body.split_at(body.len() - (index+2));

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
    let file = File::create(path);

    file.unwrap().write_all(&body[..]);

    request.respond(redirect_response(filename));
}

fn redirect_response(path: &str) -> Response<io::Empty> {
    Response::new(
        StatusCode(302),
        vec![
            Header::from_bytes(&b"Location"[..], path.as_bytes()).unwrap()
        ],
        io::empty(),
        Some(0),
        None,
    )
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
