#![feature(conservative_impl_trait)]

extern crate futures;
extern crate hyper;
extern crate serde;
extern crate serde_json;

use futures::{Future, Stream};
use hyper::Body;
use serde::de::DeserializeOwned;
use serde_json as json;

pub fn deserialize_body(body: Body) -> impl Future<Item = Vec<i64>, Error = Error> {
    deserialize_json_body(body)
}

pub fn deserialize_json_body<T: DeserializeOwned>(
    body: Body,
) -> impl Future<Item = T, Error = Error> {
    body.concat2().from_err::<Error>().and_then(|buffer| {
        json::from_slice::<T>(&buffer).map_err(Error::from)
    })
}

#[derive(Debug)]
pub enum Error {
    Hyper(hyper::Error),
    Json(json::Error),
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Error::Hyper(err)
    }
}

impl From<json::Error> for Error {
    fn from(err: json::Error) -> Self {
        Error::Json(err)
    }
}
