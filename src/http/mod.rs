#![allow(dead_code)]

use std::{collections::HashMap, fmt, str::FromStr};

use tokio::io::AsyncRead;

pub struct BoxedBody {
    pub inner: Box<dyn AsyncRead + Send + Sync + 'static + Unpin>,
}

impl BoxedBody {
    pub fn empty() -> Self {
        BoxedBody {
            inner: Box::new(&[] as &[u8]),
        }
    }
}

//#[derive(Debug)]
pub struct Response {
    pub head: ResponseParts,
    pub body: BoxedBody,
}

impl Response {
    pub fn version(&self) -> Version {
        self.head.version
    }

    pub fn status(&self) -> Status {
        self.head.status
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.head.headers
    }
}

#[derive(Debug)]
pub struct ResponseParts {
    pub status: Status,

    pub version: Version,

    // Stupid:
    // 1) we used too many `String`s, once again,
    // 2) headers should really be a multimap,
    // 3) no need to store common header key and values in string,
    // 4) we have only a handful of headers most of the time,
    //    using `HashMap` would be an overkill for our usecase.
    //    (but really convenient)
    pub headers: HashMap<String, String>,
}

impl ResponseParts {
    pub fn version(&self) -> Version {
        self.version
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Status {
    OK = 200,
    NotFound = 404,
    IntnerlServerError = 500,
    NotImplemented = 501,
}

impl Status {
    pub fn reason(self) -> &'static str {
        match self {
            Status::OK => "OK",
            Status::NotFound => "Not Found",
            Status::IntnerlServerError => "Internal Server Error",
            Status::NotImplemented => "Not Implmeneted",
        }
    }

    pub fn as_str(self) -> String {
        (self as u16).to_string()
    }
}

//#[derive(Debug, PartialEq)]
pub struct Request {
    pub head: RequestParts,
    pub body: BoxedBody, // @FIXME
}

#[derive(Debug, PartialEq)]
pub struct RequestParts {
    pub method: Method,

    // This is so braindead, since we
    // 1) used `String`, and therefore there's no zero copy,
    // 2) used `String`, and therefore there's allocation,
    // 3) assume uri is UTF-8, is it?
    pub uri: String,

    pub version: Version,

    // Stupid:
    // 1) we used too many `String`s, once again,
    // 2) headers should really be a multimap,
    // 3) no need to store common header key and values in string,
    // 4) we have only a handful of headers most of the time,
    //    using `HashMap` would be an overkill for our usecase.
    //    (but really convenient)
    pub headers: HashMap<String, String>,
}

// Obiviously there's only two HTTP methods in the world!
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

#[derive(Debug)]
pub struct BadMethod;

impl FromStr for Method {
    type Err = BadMethod;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            _ => Err(Self::Err {}),
        }
    }
}

impl fmt::Debug for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Method::*;

        f.write_str(match self {
            Get => "GET",
            Post => "POST",
        })
    }
}

// We only really care about these two in this lab.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Version {
    Http10,
    Http11,
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Version::*;

        f.write_str(match self {
            Http10 => "HTTP/1.0",
            Http11 => "HTTP/1.1",
        })
    }
}

#[derive(Debug)]
pub struct BadVersion;

impl FromStr for Version {
    type Err = BadVersion;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/1.0" => Ok(Version::Http10),
            "HTTP/1.1" => Ok(Version::Http11),
            _ => Err(Self::Err {}),
        }
    }
}

pub mod parse;
