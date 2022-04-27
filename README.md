时间太紧，没有完成，不用再测啦

# Boring HTTP

## TODOS

* [ ] Respond plain text content
    - [ ] Implement async handlers with `IntoResponse` implemented for `&str`
    - [ ] Dispatch `GET /check` to our handler

* [ ] Static file
    - [ ] `Router::new().route("/test", static_file_handler("test/"))`
    - [ ] Ensure safety. You can't do something like `http://example.com/test/../../etc/passwd`

* [ ] Error handling
    - [ ] Implement `IntoResponse` for `(StatusCode, B)` where `B` is `IntoResponse`
    - [ ] So that we can implement `IntoResponse` for the `Err` variant, simply be calling
          `(StatusCode, B).into_response()`
    - [ ] Implement `async fn static_file_handler(uri: Uri) -> Result<?>`

* [ ] URL parameter
    - [ ] Implement `FromReqeust` trait for many useful information that can be accessed thourgh the parameters of handlers
    - [ ] Store everything into a HashMap
    - [ ] Use `serde` to achieve type-sefe parsing of JSON, URL parameters and so on

* [ ] Uploading
    - [ ] Make `Request` support body type other than a `&str`
    - [ ] Make handlers able to read the data out of `Request`, asynchronously
    - [ ] What happens to multipart files?

* [ ] HTTP pipelining
    - [ ] Naive idea: spawn a new handler task for every `Request` inside `handle_connection()`
    - [ ] Take care of the order, probably with `tokio::sync::Notify`

* [ ] HTTP proxy
    - [ ] Fire up two async tasks waiting on the `client` and `server`'s `TcpStream`
    - [ ] ...

* [ ] Make it fast
    - [ ] Benchmark overall performance, e.g. requests per second
    - [ ] Peak into our async runtime, probably with `tracing` or `tokio-console`
    - [ ] Make hot-path allocation-free

* [ ] Fun
    - [ ] Write our own async runtime and backend. `io_uring` is a thing we can try.
    - [ ] Add middleware like timeout, analytics and caches
    - [ ] `tower` integration
