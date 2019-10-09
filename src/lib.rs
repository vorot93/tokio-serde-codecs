#![doc(html_root_url = "https://docs.rs/tokio-serde-json/0.3.0")]

//! `Stream` and `Sink` adaptors for serializing and deserializing values using
//! JSON.
//!
//! This crate provides adaptors for going from a stream or sink of buffers
//! ([`Bytes`]) to a stream or sink of values by performing JSON encoding or
//! decoding. It is expected that each yielded buffer contains a single
//! serialized JSON value. The specific strategy by which this is done is left
//! up to the user. One option is to use using [`length_delimited`] from
//! [tokio-io].
//!
//! # Examples
//!
//! ```no_run
//! use futures::prelude::*;
//!
//! use serde_json::json;
//!
//! use tokio::{codec::{FramedWrite, LengthDelimitedCodec}, net::TcpStream};
//!
//! use tokio_serde_json::WriteJson;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Bind a server socket
//!     let socket = TcpStream::connect("127.0.0.1:17653")
//!         .await
//!         .unwrap();
//!
//!     // Delimit frames using a length header
//!     let length_delimited = FramedWrite::new(socket, LengthDelimitedCodec::new());
//!
//!     // Serialize frames with JSON
//!     let mut serialized = WriteJson::new(length_delimited);
//!
//!     // Send the value
//!     serialized.send(json!({
//!       "name": "John Doe",
//!       "age": 43,
//!       "phones": [
//!         "+44 1234567",
//!         "+44 2345678"
//!       ]
//!     })).await.unwrap()
//! }
//! ```
//!
//! For a full working server and client example, see the [examples] directory.
//!
//! [`Bytes`]: https://docs.rs/bytes/0.4/bytes/struct.Bytes.html
//! [`length_delimited`]: https://docs.rs/tokio-io/0.1/tokio_io/codec/length_delimited/index.html
//! [tokio-io]: https://github.com/tokio-rs/tokio-io
//! [examples]: https://github.com/carllerche/tokio-serde-json/tree/master/examples

use bytes::{Buf, Bytes, BytesMut, IntoBuf};
use futures::prelude::*;
use pin_project::pin_project;
use serde::{Deserialize, Serialize};
use tokio_serde::{Deserializer, FramedRead, FramedWrite, Serializer};

use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Json<T> {
    ghost: PhantomData<T>,
}

impl<T> Deserializer<T> for Json<T>
where
    for<'a> T: Deserialize<'a>,
{
    type Error = serde_json::Error;

    fn deserialize(self: Pin<&mut Self>, src: &BytesMut) -> Result<T, Self::Error> {
        serde_json::from_reader(src.into_buf().reader())
    }
}

impl<T: Serialize> Serializer<T> for Json<T> {
    type Error = serde_json::Error;

    fn serialize(self: Pin<&mut Self>, item: &T) -> Result<Bytes, Self::Error> {
        serde_json::to_vec(item).map(Into::into)
    }
}
