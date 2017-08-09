use futures;
use futures::future::FutureResult;
use futures_cpupool::{CpuFuture, CpuPool};

use hyper;
use hyper::StatusCode;
use hyper::header::{ContentLength, ContentType};
use hyper::server::{Http, Service, Request, Response};

use futures::{Async, Future, Poll};

use std;
use std::error::Error as StdError;
use std::fs::File;
use std::io::{self, Read};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use mime_guess::guess_mime_type;

#[derive(Clone)]
pub struct ServerConfig {
    pub addr: SocketAddr,
    pub root_dir: PathBuf,
    pub num_file_threads: usize,
    pub num_server_threads: u16,
}

pub enum Error {
    Io(io::Error),
    AddrParse(std::net::AddrParseError),
    Std(Box<StdError + Send + Sync>),
    ParseInt(std::num::ParseIntError),
}

pub fn serve(config:ServerConfig) -> Result<(), Error> {
    let ServerConfig {
        addr, root_dir, num_file_threads, ..
    } = config;

    // Create HTTP service, passing the document root directory and the
    // thread pool used for executing the file reading I/O on.
    let server = Http::new().bind(&addr, move || {
        Ok(HttpService {
            root_dir: root_dir.clone(),
            pool: CpuPool::new(num_file_threads),
        })
    }).unwrap();
    server.run().unwrap();
    Ok(())
}

pub fn serve_example() -> Result<(), Error> {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let server_config = ServerConfig {
        addr: addr,
        root_dir: PathBuf::from("."),
        num_file_threads: 4,
        num_server_threads: 4,
    };
    println!("about to serve!");
    serve(server_config)
}



struct HttpService {
    root_dir: PathBuf,
    pool: CpuPool,
}

// The HttpService knows how to build a ResponseFuture for each hyper Request
// that is received. Errors are turned into an Error response (404 or 500).
impl Service for HttpService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = ResponseFuture;
    fn call(&self, req: Request) -> Self::Future {
        let uri_path = req.uri().path();
        if let Some(path) = local_path_for_request(&uri_path, &self.root_dir) {
            ResponseFuture::File(self.pool.spawn(FileFuture { path }))
        } else {
            ResponseFuture::Error
        }
    }
}

enum ResponseFuture {
    File(CpuFuture<Response, Error>),
    Error,
}

impl Future for ResponseFuture {
    type Item = Response;
    type Error = hyper::Error;
    fn poll(&mut self) -> Poll<Response, hyper::Error> {
        match *self {
            // If this is a File variant, poll the contained CpuFuture
            // and propagate the result outward as a Response.
            ResponseFuture::File(ref mut f) => match f.poll() {
                Ok(Async::Ready(rsp)) => Ok(Async::Ready(rsp)),
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Err(_) => Ok(Async::Ready(internal_server_error())),
            },
            // For the Error variant, we can just return an error immediately.
            ResponseFuture::Error => Ok(Async::Ready(internal_server_error())),
        }
    }
}

struct FileFuture {
    path: PathBuf, // enum of this or index !?
}

impl Future for FileFuture {
    type Item = Response;
    type Error = Error;
    fn poll(&mut self) -> Poll<Response, Error> {
        match File::open(&self.path) {
            Ok(mut file) => {
                let mut buf = Vec::new();
                match file.read_to_end(&mut buf) {
                    Ok(_) => {
                        let mime = guess_mime_type(&self.path);
                        Ok(Async::Ready(Response::new()
                            .with_status(StatusCode::Ok)
                            .with_header(ContentLength(buf.len() as u64))
                            .with_header(ContentType(mime))
                            .with_body(buf)
                        ))
                    }
                    Err(_) => Ok(Async::Ready(internal_server_error())),
                }
            }
            Err(e) => {
                match e.kind() {
                    io::ErrorKind::NotFound => {
                        Ok(Async::Ready(Response::new()
                            .with_status(StatusCode::NotFound)))
                    },
                    _ => Ok(Async::Ready(internal_server_error())),
                }
            }
        }
    }
}

fn local_path_for_request(request_path: &str, root_dir: &Path) -> Option<PathBuf> {
    // This is equivalent to checking for hyper::RequestUri::AbsoluteUri
    if !request_path.starts_with("/") {
        return None;
    }
    // Trim off the url parameters starting with '?'
    let end = request_path.find('?').unwrap_or(request_path.len());
    let request_path = &request_path[0..end];

    // Append the requested path to the root directory
    let mut path = root_dir.to_owned();
    if request_path.starts_with('/') {
        path.push(&request_path[1..]);
    } else {
        return None;
    }

//    // Maybe turn directory requests into index.html requests
//    if request_path.ends_with('/') {
//        path.push("index.html");
//    }

    Some(path)
}

fn internal_server_error() -> Response {
    Response::new()
        .with_status(StatusCode::InternalServerError)
        .with_header(ContentLength(0))
}
