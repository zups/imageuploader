extern crate tiny_http;
use tiny_http::{Response, Request, Server, Method, Header, StatusCode};
use std::path::Path;
use std::fs::File;
use std::io::{self, Write, Read, Cursor};
use std::str::from_utf8;
use std::sync::Arc;
use std::{env, thread, time};
use log::{info};

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

fn upload(request: Request) {
    let upload = get_file("upload.html");
    let _ = request.respond(Response::from_file(upload.unwrap())).expect("failed at upload()");
}

fn index(request: Request) {
    let index = get_file("index.html");
    let _ = request.respond(Response::from_file(index.unwrap())).expect("failed at upload()");
}

fn file_response(request: Request, path: &str) {
    let file_location = &format!("files/{}", path);
    let _ = match get_file(file_location) {
        Ok(file) => {
            if contains_video(file_location) {
                request.respond(Response::from_file(file).with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"video/mp4"[..]).unwrap()))
            } else {
                request.respond(Response::from_file(file))
            }},
        Err(err) => request.respond(Response::from_string(err)),
    };
}

fn contains_video(path: &String) -> bool {
    return path.contains("mp4");
}



fn convert_handhistory_to_html(request: Request, path: &str) {
    let file_location = &format!("files/{}", path);
    let mut buffer = String::new();

    let file = get_file(file_location);

    if file.is_ok() {
        let _ = file.unwrap().read_to_string(&mut buffer).expect("failed at convert_hh()");
    } else {
        let _ = request.respond(Response::from_string(file.err().expect("Couldn't find the file.")));
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

    let mut html_string = String::new();

    for line in buffer.lines() {
        if line.contains("[") {
            let (start_part, card_line) = line.split_at(line.find("[").unwrap());
            let mut html_line = String::from("");
            
            html_line.push_str(start_part);

            let mut index = 0;

            for element in card_line.chars() {
                match element {
                    'c' => html_line.push_str("<img src=\"club.gif\"/>"),
                    'd' => html_line.push_str("<img src=\"diamond.gif\"/>"),
                    'h' => html_line.push_str("<img src=\"heart.gif\"/>"),
                    's' => html_line.push_str("<img src=\"spade.gif\"/>"),
                    '(' => { //Breakline for bracket text
                        html_line.push_str(card_line.split_at(index).1);
                        break;
                    },
                    'a' => { //Breakline for 'and' text
                        html_line.push_str(card_line.split_at(index).1);
                        break;
                    },
                    _ => html_line.push_str(&format!("{}{}{}", "<b>", element, "</b>")),
                }
                index += 1;
            }
            html_line.push_str("\n");

            // println!("{}", htmlLine);

            html_string.push_str(html_line.as_str());
        } else {
            html_string.push_str(line);
            html_string.push_str("\n");
        }
    }

    html_string.push_str("</pre>");
    html_string.insert_str(0, "<pre style=\"font-size: 120%;\">");
    html_string = html_string.replace("checks", "<b>checks</b>");
    html_string = html_string.replace("folds", "<b>folds</b>");
    html_string = html_string.replace("calls", "<b>calls</b>");
    html_string = html_string.replace("raises", "<b>raises</b>");
    html_string = html_string.replace("bets", "<b>bets</b>");

    
    let data_len = html_string.len();

    let response = Response::new(
        StatusCode(200),
        vec![
            Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=UTF-8"[..]).unwrap()
        ],
        Cursor::new(html_string),
        Some(data_len),
        None,
    );

    let _ = request.respond(response).expect("Failed at creating respond hh()");
}

fn get_handler(request: Request) {
    match request.url() {
        "/" => {
            index(request);
        },
        "/uploadz" => {
            upload(request);
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
    let _ = request.as_reader().read_to_end(&mut buffer).expect("Failed at post_handler as_reader()");

    let (body, headers) = parse_multipart(&mut buffer);

    let filename = &parse_filename(headers).replace("#", "r");

    if filename.is_empty() {
        let _ = request.respond(Response::from_string("Not found")).expect("Failed at not found post_handler()");
        return;
    }

    info!("filename: {}", filename);

    let path = format!("files/{}", filename);
    let mut file = match File::create(path) {
        Err(_e) => {
            let _ = request.respond(Response::from_string("Failed to create path")).expect("Failed at creating path");
            return;
        },
        Ok(f) => f,
    };

    let _ = file.write_all(&body[..]).expect("Failed at write_all post_handler()");
    let _ = request.respond(redirect_response(filename)).expect("Failed at creating response post_handler()");
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

fn create_server() {
    let server = Server::http("0.0.0.0:8080").unwrap();
    let server = Arc::new(server);
    let mut guards = Vec::with_capacity(4);

    for _ in 0..4 {
        let server = server.clone();

        let guard = thread::spawn(move || {
            loop {
                let request = server.recv().unwrap();
                info!("(Path: {}\n From: {:?})", request.url(), request.remote_addr() );
                match request.method() {
                    Method::Get => get_handler(request), 
                    Method::Post => post_handler(request),
                    _ => (),
                }; 
            }
        });

        guards.push(guard);
    }

    guards;
}

fn main() {
		log4rs::init_file("logging_config.yaml", Default::default()).unwrap();
		create_server();
    let site_url = env::var("IMAGEUPLOADER_URL").unwrap();
		loop {
				if reqwest::blocking::get(site_url.clone()).is_err() {
						create_server();
				}
				thread::sleep(time::Duration::from_secs(60));
		}
}
