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
