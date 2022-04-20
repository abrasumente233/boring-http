// Largely inspired by https://gist.github.com/jFransham/e41835f409e264212e4e66313adafc8b

use std::marker::PhantomData;

use async_trait::async_trait;
use regex::Regex;

use crate::http::Response;
use crate::{
    handler::Handler,
    http::{Method, RequestParts},
};

#[derive(Debug)]
pub struct PathPattern<'a> {
    /// An regex to match against the URI
    re: Regex,

    /// Paramater names
    names: Vec<&'a str>,
}

pub type Parameters = Vec<(String, String)>;

impl<'t> PathPattern<'t> {
    fn new(pattern: &'t str) -> Self {
        let mut names = vec![];

        let pattern = pattern
            .split('/')
            .into_iter()
            .map(|s| {
                if s.starts_with('{') {
                    assert!(s.ends_with('}'));
                    let param = &s[1..s.len() - 1];
                    names.push(param);

                    r"([^/]*)"
                } else {
                    s
                }
            })
            .collect::<Vec<_>>()
            .join("/");

        dbg!("built re pattern: {}", pattern);

        PathPattern {
            re: Regex::new(&pattern).unwrap(),
            names,
        }
    }

    fn captures<'b>(&self, uri: &'b str) -> Option<Parameters> {
        Some(
            self.re
                .captures(uri)
                .into_iter()
                .next()?
                .iter()
                .skip(1)
                .map(|m| m.unwrap()) // @FIXME
                .enumerate()
                .map(|(i, m)| {
                    (
                        self.names[i].to_string(),
                        uri[m.start()..m.end()].to_string(),
                    )
                })
                .collect(),
        )
    }
    /*
    fn captures<'b>(&self, uri: &'b str) -> Option<Parameters<'t, 'b>> {
        Some(
            self.re
                .captures(uri)
                .into_iter()
                .next()?
                .iter()
                .skip(1)
                .map(|m| m.unwrap()) // @FIXME
                .enumerate()
                .map(|(i, m)| (self.names[i], &uri[m.start()..m.end()]))
                .collect(),
        )
    }
    */
}

// 'a is the lifetime of path pattern &str, sad, we're adding it back.
pub struct Router<'a, 'h, T: Handler<'h>> {
    method: Method,
    pattern: PathPattern<'a>,
    handler: T,
    phantom: PhantomData<&'h ()>,
}

impl<'t1, 'h1, T1: Handler<'h1>> Router<'t1, 'h1, T1> {
    // We need special handling for the first route, which is not nice.
    pub fn new(pattern: &'t1 str, method_wrapper: MethodWrapper<'h1, T1>) -> Self {
        Self {
            pattern: PathPattern::new(pattern),
            method: method_wrapper.1,
            handler: method_wrapper.0,
            phantom: PhantomData,
        }
    }

    pub fn route<'t2, 'h2, T2: Handler<'h2>>(
        self,
        pattern: &'t2 str,
        method_wrapper: MethodWrapper<'h2, T2>,
    ) -> Chain<Router<'t1, 'h1, T1>, Router<'t2, 'h2, T2>> {
        Chain {
            first: self,
            second: Router {
                pattern: PathPattern::new(pattern),
                method: method_wrapper.1,
                handler: method_wrapper.0,
                phantom: PhantomData,
            },
        }
    }
}

pub struct Chain<A, B> {
    first: A,
    second: B,
}

impl<T1, T2> Chain<T1, T2> {
    pub fn route<'t, 'h3, T3: Handler<'h3>>(
        self,
        pattern: &'t str,
        method_wrapper: MethodWrapper<'h3, T3>,
    ) -> Chain<Chain<T1, T2>, Router<'t, 'h3, T3>> {
        Chain {
            first: self,
            second: Router {
                pattern: PathPattern::new(pattern),
                method: method_wrapper.1,
                handler: method_wrapper.0,
                phantom: PhantomData,
            },
        }
    }
}

fn square(params: Parameters, x: i32) -> i32 {
    x * x
}

fn times2(params: Parameters, x: i32) -> i32 {
    x * 2
}

#[async_trait]
pub trait Dispatcher {
    async fn dispatch(&self, request: &RequestParts) -> Option<Response>;
}

#[async_trait]
impl<'t, 'a, T: Handler<'a> + Send + Sync> Dispatcher for Router<'t, 'a, T> {
    async fn dispatch(&self, request: &RequestParts) -> Option<Response> {
        // @TODO: Capture groups
        if let Some(params) = self.pattern.captures(&request.uri) {
            self.handler.handle(request, params).await // Assert Some
        } else {
            None
        }
    }
}

#[async_trait]
impl<T1: Dispatcher + Send + Sync, T2: Dispatcher + Send + Sync> Dispatcher for Chain<T1, T2> {
    async fn dispatch(&self, request: &RequestParts) -> Option<Response> {
        let r1 = self.first.dispatch(request).await;

        match r1 {
            Some(_) => r1,
            None => self.second.dispatch(request).await,
        }
    }
}

pub struct MethodWrapper<'a, H: Handler<'a>>(H, Method, PhantomData<&'a ()>);

pub fn get<'a, H: Handler<'a>>(handler: H) -> MethodWrapper<'a, H> {
    MethodWrapper(handler, Method::Get, PhantomData)
}

pub fn post<'a, H: Handler<'a>>(handler: H) -> MethodWrapper<'a, H> {
    MethodWrapper(handler, Method::Post, PhantomData)
}
