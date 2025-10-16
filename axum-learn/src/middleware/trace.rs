use std::fmt::Display;
use std::time::Duration;

use axum::Router;
use axum::extract::Request;
use axum::http::Response;
use tower_http::trace::OnResponse;
use tower_http::trace::TraceLayer;
use tracing::span::Span;

#[derive(Clone)]
pub struct LatencyOnResponse;

impl<B> OnResponse<B> for LatencyOnResponse {
    fn on_response(self, response: &Response<B>, latency: Duration, _span: &Span) {
        tracing::info!(
            latency = %Latency(latency),
            status = %response.status().as_u16(),
            "finished processing request"
        );
    }
}

struct Latency(Duration);

impl Display for Latency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ms = self.0.as_millis();
        if ms > 0 {
            write!(f, "{} ms", ms)
        } else {
            let us = self.0.as_micros();
            write!(f, "{} µs", us)
        }
    }
}

pub fn add_tracing<S>(router: Router<S>) -> Router<S>
where
    S: Send + Sync + Clone + 'static,
{
    let tracing = TraceLayer::new_for_http()
        .make_span_with(|request: &Request| {
            let id = xid::new();
            let method = request.method();
            let path = request.uri().path();
            tracing::info_span!("HTTP Request", id= %id, method= %method, path= %path)
        })
        .on_request(())
        .on_failure(())
        .on_response(LatencyOnResponse);
    router.layer(tracing)
}
