use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

use crossbeam_channel::Sender;
use log::debug;
use percent_encoding::percent_decode_str;
use MessAuto::{display_verify, read_config};

pub fn run(tx: Sender<String>) {
    let listener = TcpListener::bind("0.0.0.0:17096").unwrap();
    let addr = listener.local_addr().unwrap();
    let _ = tx.send(format!("http://{}:{}", addr.ip(), addr.port()));

    let flags = read_config().flags;

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream, &flags);
    }
}

fn handle_connection(mut stream: TcpStream, flags: &Vec<String>) {
    let buf_reader = BufReader::new(&mut stream);
    let header = buf_reader.lines().map(|r| r.unwrap()).next().unwrap();

    println!("Request: {:#?}", header);

    let response = if header.starts_with("GET /notification") {
        // get the request params
        let mut result = "";
        header
            .split_whitespace()
            .find(|x| x.starts_with("/notification"))
            .map(|params| {
                let params = params.split('?').last().unwrap();
                let params = params.split('&').collect::<Vec<&str>>();
                for param in params {
                    if param.starts_with("message=") {
                        result = param.split('=').last().unwrap();
                    }
                }
            });
        let decode = percent_decode_str(result).decode_utf8().unwrap();
        display_verify(&decode, flags);
        debug!("Payload: {:#?}", decode);
        "HTTP/1.1 200 OK\r\n\r\n"
    } else {
        "HTTP/1.1 404 NOT FOUND\r\n\r\n"
    };
    stream.write_all(response.as_bytes()).unwrap();
}
