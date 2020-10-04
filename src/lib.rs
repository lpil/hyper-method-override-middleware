//! A middleware that overrides an incoming POST request with a method given in
//! the request's `_method` query paramerter. This is useful as web browsers
//! typically only support GET and POST requests, but our application may
//! expect other HTTP methods that are more semantically correct.
//!
//! The methods PUT, PATCH, and DELETE are accepted for overriding, all others
//! are ignored.
//!
//! The `_method` query paramerter can be specified in a HTML form like so:
//!
//!    <form method="POST" action="/item/1?_method=DELETE">
//!      <button type="submit">Delete item</button>
//!    </form>
//!

use hyper::{service::Service, Method, Request};
use std::borrow::Borrow;
use std::task::{Context, Poll};
use url::form_urlencoded;

#[derive(Debug, Clone)]
pub struct MethodOverrideMiddleware<T> {
    inner_service: T,
}

impl<T> MethodOverrideMiddleware<T> {
    pub fn new(inner_service: T) -> Self {
        Self { inner_service }
    }
}

impl<InnerService, Body> Service<Request<Body>> for MethodOverrideMiddleware<InnerService>
where
    InnerService: Service<Request<Body>>,
{
    type Response = InnerService::Response;
    type Error = InnerService::Error;
    type Future = InnerService::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner_service.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        if let Some(new_method) = override_method(&req) {
            *req.method_mut() = new_method;
        }
        self.inner_service.call(req)
    }
}

fn override_method<Body>(req: &Request<Body>) -> Option<Method> {
    if req.method() != &Method::POST {
        return None;
    }

    form_urlencoded::parse(req.uri().query().unwrap_or("").as_bytes())
        .find(|(param_name, _)| param_name == "_method")
        .and_then(|(_, method)| match method.borrow() {
            "DELETE" => Some(Method::DELETE),
            "PATCH" => Some(Method::PATCH),
            "PUT" => Some(Method::PUT),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::service::{make_service_fn, service_fn};
    use hyper::{Body, Request, Response, Server};
    use std::convert::Infallible;

    async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let body = format!("{:?}", req.method()).into();
        Ok(Response::new(body))
    }

    async fn send(method: Method, url: &str) -> String {
        reqwest::Client::new()
            .execute(reqwest::Request::new(
                method,
                reqwest::Url::parse(url).unwrap(),
            ))
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn override_test() {
        let addr = ([127, 0, 0, 1], 1337).into();

        tokio::spawn(Server::bind(&addr).serve(make_service_fn(|_| async {
            let service = MethodOverrideMiddleware::new(service_fn(handle));
            Ok::<_, hyper::Error>(service)
        })));

        // No override requested
        assert_eq!(send(Method::GET, "http://127.0.0.1:1337").await, "GET");
        assert_eq!(send(Method::PUT, "http://127.0.0.1:1337").await, "PUT");
        assert_eq!(send(Method::POST, "http://127.0.0.1:1337").await, "POST");
        assert_eq!(send(Method::PATCH, "http://127.0.0.1:1337").await, "PATCH");

        // Successful overrides
        assert_eq!(
            send(Method::POST, "http://127.0.0.1:1337?a=1&b=2&_method=PATCH").await,
            "PATCH"
        );
        assert_eq!(
            send(Method::POST, "http://127.0.0.1:1337?a=1&b=2&_method=PUT").await,
            "PUT"
        );
        assert_eq!(
            send(Method::POST, "http://127.0.0.1:1337?a=1&b=2&_method=DELETE").await,
            "DELETE"
        );

        // Other methods cannot be specified for overrides
        assert_eq!(
            send(Method::POST, "http://127.0.0.1:1337?a=1&b=2&_method=GET").await,
            "POST"
        );
        assert_eq!(
            send(Method::POST, "http://127.0.0.1:1337?_method=OPTIONS").await,
            "POST"
        );

        // Non-POST requests don't get overriden
        assert_eq!(
            send(Method::GET, "http://127.0.0.1:1337?_method=PATCH").await,
            "GET"
        );
        assert_eq!(
            send(Method::DELETE, "http://127.0.0.1:1337?_method=PUT").await,
            "DELETE"
        );
        assert_eq!(
            send(Method::PATCH, "http://127.0.0.1:1337?_method=DELETE").await,
            "PATCH"
        );
    }
}
