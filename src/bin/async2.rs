extern crate async_bench;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate tokio_timer;

use futures::future;
use futures::{Future, Stream};
use futures_cpupool::CpuPool;
use hyper::server::{Http, Service, Request, Response};
use serde_json as json;
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
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let pool = self.pool.clone();
        let result = req.body().concat2().map_err(|_| unimplemented!())
        .and_then(move |buffer| {
            // some json deserialization
            let nums = json::from_slice::<Vec<i64>>(&buffer).unwrap();
            pool.spawn_fn(move || {
                let sum = nums.iter().fold(0, |sum, val| sum + val);
                future::ok(sum)
            })
        })
        .and_then(|sum| {
            // delay should represent a database query
            let timer = Timer::default();
            let sleep = timer.sleep(Duration::from_millis(20));

            sleep.map_err(|_| unimplemented!()).map(move |_| sum)
        })
        .map(|sum| {
            let res = Response::default().with_body(format!("Sum: {}", sum));
            res
        });

        Box::new(result)
    }
}