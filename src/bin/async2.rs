extern crate async_bench;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;

use futures::future;
use futures::{Future, Stream};
use futures_cpupool::CpuPool;
use hyper::server::{Http, Service, Request, Response};
use serde_json as json;
use std::time::Duration;
use tokio_core::net::TcpListener;
use tokio_core::reactor::{Core, Handle, Timeout};

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let app = App::new(&handle);
    let protocol = Http::new();

    let addr = "127.0.0.1:3000".parse().unwrap();
    let listener = TcpListener::bind(&addr, &handle).unwrap();
    println!("Listening on http://{}", listener.local_addr().unwrap());
    let server = listener.incoming().for_each(|(socket, addr)| {
        protocol.bind_connection(&handle, socket, addr, app.clone());
        Ok(())
    });
    core.run(server).unwrap();
}

#[derive(Clone)]
struct App {
    pool: CpuPool,
    handle: Handle,
}

impl App {
    fn new(handle: &Handle) -> Self {
        App {
            pool: CpuPool::new(32),
            handle: handle.clone(),
        }
    }
}

impl Service for App {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let pool = self.pool.clone();
        let handle = self.handle.clone();
        let result = req.body()
            .concat2()
            .map_err(|_| unimplemented!())
            .and_then(move |buffer| {
                // some json deserialization
                let nums = json::from_slice::<Vec<i64>>(&buffer).unwrap();
                pool.spawn_fn(move || {
                    let sum = nums.iter().fold(0, |sum, val| sum + val);
                    future::ok(sum)
                })
            })
            .and_then(move |sum| {
                // delay should represent a database query
                let sleep = Timeout::new(Duration::from_millis(20), &handle).unwrap();
                sleep.map_err(|_| unimplemented!()).map(move |_| sum)
            })
            .map(|sum| {
                let res = Response::default().with_body(format!("Sum: {}", sum));
                res
            });

        Box::new(result)
    }
}
