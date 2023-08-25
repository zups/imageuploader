extern crate tiny_http;
use tiny_http::{Response, Request, Server, Method, Header, StatusCode};
use std::path::Path;
use std::fs::File;
use std::io::{self, Write, Read, Cursor};
use std::str::from_utf8;
use log::{info, warn};

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



fn convert_handhistory_to_html(request: Request, path: &str) {
    let fileLocation = &format!("files/{}", path);
    let mut buffer = String::new();

    let file = get_file(fileLocation);

    if file.is_ok() {
        file.unwrap().read_to_string(&mut buffer);
    } else {
        request.respond(Response::from_string(file.err().unwrap()));
        return;
    }

//bugggaaa

    
    let mut heroname = "";

    for line in buffer.lines() {
        if line.contains("Dealt to") {
            let splitted: Vec<&str> = line.split(' ').collect();
            heroname = splitted.get(2).unwrap();
        }
    }

    buffer = buffer.replace(heroname, "<font color=\"blue\">Hero</font>");

    let mut htmlString = String::new();

    for line in buffer.lines() {
        if line.contains("[") {
            let (startPart, cardLine) = line.split_at(line.find("[").unwrap());
            let mut htmlLine = String::from("");
            
            htmlLine.push_str(startPart);

            let mut index = 0;

            for element in cardLine.chars() {
                match element {
                    'c' => htmlLine.push_str("<img src=\"club.gif\"/>"),
                    'd' => htmlLine.push_str("<img src=\"diamond.gif\"/>"),
                    'h' => htmlLine.push_str("<img src=\"heart.gif\"/>"),
                    's' => htmlLine.push_str("<img src=\"spade.gif\"/>"),
                    '(' => { //Breakline for bracket text
                        htmlLine.push_str(cardLine.split_at(index).1);
                        break;
                    },
                    'a' => { //Breakline for 'and' text
                        htmlLine.push_str(cardLine.split_at(index).1);
                        break;
                    },
                    _ => htmlLine.push_str(&format!("{}{}{}", "<b>", element, "</b>")),
                }
                index += 1;
            }
            htmlLine.push_str("\n");

            // println!("{}", htmlLine);

            htmlString.push_str(htmlLine.as_str());
        } else {
            htmlString.push_str(line);
            htmlString.push_str("\n");
        }
    }

    htmlString.push_str("</pre>");
    htmlString.insert_str(0, "<pre style=\"font-size: 120%;\">");
    htmlString = htmlString.replace("checks", "<b>checks</b>");
    htmlString = htmlString.replace("folds", "<b>folds</b>");
    htmlString = htmlString.replace("calls", "<b>calls</b>");
    htmlString = htmlString.replace("raises", "<b>raises</b>");
    htmlString = htmlString.replace("bets", "<b>bets</b>");

    
    let data_len = htmlString.len();

    let response = Response::new(
        StatusCode(200),
        vec![
            Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=UTF-8"[..]).unwrap()
        ],
        Cursor::new(htmlString),
        Some(data_len),
        None,
    );

    request.respond(response);
}

fn get_handler(request: Request) {
    match request.url() {
        "/upload" => {
            index(request);
        },
        _ => {
            info!("{}", request.url());
            let path = request.url().replace("/", "");
            if path.starts_with("HH") {
                convert_handhistory_to_html(request, path.as_str())
            } else {
                file_response(request, path.as_str());
            }
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

    let filename = &parse_filename(headers).replace("#", "r");

    if filename.is_empty() {
        request.respond(Response::from_string("Not found"));
        return;
    }

    info!("filename: {}", filename);

    let path = format!("files/{}", filename);
    let file = File::create(path);

    file.unwrap().write_all(&body[..]);


    request.respond(redirect_response(filename));
}

fn redirect_response(path: &str) -> Response<io::Empty> {
    Response::new(
        StatusCode(301),
        vec![
            Header::from_bytes(&b"Location"[..], path.as_bytes()).unwrap()
        ],
        io::empty(),
        Some(0),
        None,
    )
}

fn main() {
    log4rs::init_file("logging_config.yaml", Default::default()).unwrap();
    let server = Server::http("0.0.0.0:8080").unwrap();

    for request in server.incoming_requests() {
        info!("(Path: {}\n From: {:?})", request.url(), request.remote_addr() );
        match request.method() {
          Method::Get => get_handler(request), 
          Method::Post => post_handler(request),
          _ => (),
       }; 
    }
}
