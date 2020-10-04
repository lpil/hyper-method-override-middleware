# Hyper Method Override Middleware

[![crates.io](https://img.shields.io/crates/v/hyper-method-override-middleware.svg)](https://crates.io/crates/hyper-method-override-middleware)
[![Documentation](https://docs.rs/hyper-method-override-middleware/badge.svg)](https://docs.rs/hyper-method-override-middleware)
[![Apache-2 licensed](https://img.shields.io/crates/l/hyper-method-override-middleware.svg)](./LICENCE)
<!-- [![CI](https://github.com/lpil/hyper-method-override-middleware/workflows/CI/badge.svg)](https://github.com/lpil/hyper-method-override-middleware/actions) -->

A middleware for Hyper that overrides an incoming POST request with a method
given in the request's `_method` query parameter. This is useful as web
browsers typically only support GET and POST requests, but our application may
expect other HTTP methods that are more semantically correct.

The methods PUT, PATCH, and DELETE are accepted for overriding, all others
are ignored.

The `_method` query paramerter can be specified in a HTML form like so:

```html
<form method="POST" action="/item/1?_method=DELETE">
    <button type="submit">Delete item</button>
</form>
```

And the middleware can be applied to our Hyper service like so:

```rust
let service = MethodOverrideMiddleware::new(service);
```

## Full example

Here's the example from the Hyper homepage with the middleware applied.

```rust
use std::{convert::Infallible, net::SocketAddr};
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

async fn handle(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("Hello, World!".into()))
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_svc = make_service_fn(|_conn| async {
        let service = MethodOverrideMiddleware::new(service_fn(handle));
        Ok::<_, Infallible>(service)
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
```
