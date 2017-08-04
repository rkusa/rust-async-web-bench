extern crate async_bench;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
extern crate tokio_timer;

use async_bench::deserialize_body;
use futures::future::Future;
use futures_cpupool::{CpuPool, CpuFuture};
use hyper::server::{Http, Service, Request, Response};
use std::time::Duration;
use tokio_timer::Timer;

fn main() {
    let app = App::new();

    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = Http::new().bind(&addr, move || Ok(app.clone())).unwrap();
    println!("Listening on http://{}", server.local_addr().unwrap());
    server.run().unwrap();
}

#[derive(Clone)]
struct App {
  pool: CpuPool,
}

impl App {
  fn new() -> Self {
    App {
      pool: CpuPool::new(32),
    }
  }
}

impl Service for App
{
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = CpuFuture<Self::Response, Self::Error>;

    fn call(&self, req: Self::Request) -> Self::Future {
        self.pool.spawn_fn(move || {
            // some json deserialization
            let nums = deserialize_body(req.body()).wait().unwrap();
            let sum = nums.iter().fold(0, |sum, val| sum + val);

            // delay should represent a database query
            let timer = Timer::default();
            let sleep = timer.sleep(Duration::from_millis(20));
            sleep.wait().unwrap();

            let res = Response::default().with_body(format!("Sum: {}", sum));
            Ok(res)
        })
    }
}