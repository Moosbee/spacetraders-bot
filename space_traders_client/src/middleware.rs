use hyper::http::Extensions;
use reqwest::{header::HeaderValue, Request, Response};
use reqwest_middleware::{Middleware, Next, Result};

pub struct ContentLengthFixMiddleware;

#[async_trait::async_trait]
impl Middleware for ContentLengthFixMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        if let None = req.body() {
            if !req.headers().contains_key("content-length") {
                req.headers_mut()
                    .append("content-length", HeaderValue::from_static("0"));
            }
        }
        let res = next.run(req, extensions).await;
        res
    }
}
