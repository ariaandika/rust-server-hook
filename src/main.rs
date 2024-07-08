use std::{future::Future, pin::Pin, sync::Arc};

mod github;

use bytes::Bytes;
use github::GithubHeader;
use http_body_util::{BodyExt as _, Either, Full};
use hyper::{body::{Body as _, Incoming}, server::conn::http1, service::Service, StatusCode};
use hyper_util::rt::TokioIo;
use serde_json::{json, Value};
use tokio::net::TcpListener;

type Request<T = Incoming> = hyper::Request<T>;
type Response<T = Either<Full<Bytes>, Incoming>> = hyper::Response<T>;
type Result<T = Response> = std::result::Result<T, Error>;
type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

#[tokio::main]
async fn main() {
    let tcp = setup_tcp().await;
    let pool = setup_sqlite_pool();

    let state = Arc::new(State { pool });

    println!("listening in: http://{}", tcp.local_addr().map_or(String::new(), |e|e.to_string()));

    loop {
        let handler = Handler(Arc::clone(&state));
        let (conn, _) = match tcp.accept().await {
            Ok(ok) => ok,
            Err(err) => {
                eprintln!("TcpError: {err}");
                continue;
            },
        };
        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(TokioIo::new(conn), handler)
                .with_upgrades().await
            {
                eprintln!("HyperError: {err}");
            }
        });
    }
}

struct State {
    pool: Pool
}

#[derive(Debug)]
enum Error {
    InternalError,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Either::Left(Full::default())).expect("invalid valid thing")
    }
}

trait IntoResponse {
    fn into_response(self) -> Response;
}

struct Handler(Arc<State>);

impl Service<Request> for Handler {
    type Response = Response;
    type Error = String;
    type Future = Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request) -> Self::Future {
        let state = Arc::clone(&self.0);
        Box::pin(async move {
            match handle(req, state).await {
                Ok(res) => Ok(res),
                Err(err) => Ok(err.into_response())
            }
        })
    }
}

async fn setup_tcp() -> TcpListener {
    let port = std::env::vars().find_map(|(key, val)|{
        if &key != "PORT" {
            return None;
        }
        val.parse::<u32>().ok()
    }).unwrap_or(3000);

    let addr = format!("127.0.0.1:{port}");

    TcpListener::bind(&addr).await.expect(&format!("cannot bind to {addr}"))
}

fn setup_sqlite_pool() -> Pool {
    let path = std::env::vars().find_map(|(key, val)|
        if key == "DB_PATH" { Some(val) } else { None }
    ).unwrap_or("./db.sqlite".into());

    let manager = r2d2_sqlite::SqliteConnectionManager::file(path);
    r2d2::Pool::new(manager).expect("failed to create connection pooling")
}

async fn handle(req: Request, state: Arc<State>) -> Result {
    let (parts, body) = req.into_parts();
    let path = parts.uri.path();

    if path == "/status" || path == "/healthchekc" {
        return Ok(json!({ "status": "ok" }).into_response());
    }

    let gh_header = GithubHeader::from_request_parts(&parts);
    println!("HEADER: {gh_header:?}");

    match &*gh_header.x_github_event {
        "ping" => Ok(Response::new(Either::Left(Full::default()))),
        github::PushEvent::HEADER_EVENT
            => handle_push_event(Request::from_parts(parts, body), state).await,
        x => {
            let json = json!({
                "error": "not supported",
                "event": x,
            });
            let vec = serde_json::to_vec(&json).expect("invalid json response");
            Ok(Response::builder()
                .status(StatusCode::NOT_IMPLEMENTED)
                .body(Either::Left(Full::new(Bytes::from(vec)))).expect("invalid response body"))
        },
    }
}

async fn handle_push_event(req: Request, _state: Arc<State>) -> Result {
    // Protect our server from massive bodies.
    let upper = req.body().size_hint().upper().unwrap_or(u64::MAX);
    if upper > 1024 * 64 {
        // *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
        return Ok(Response::builder()
            .status(StatusCode::PAYLOAD_TOO_LARGE)
            .body(Either::Left(Full::default()))
            .expect("invalid response body"));
    }

    // Await the whole body to be collected into a single `Bytes`...
    let body = req.collect().await?.to_bytes();

    let push_event: Value = serde_json::from_slice(&body)?;

    println!("{}", serde_json::to_string_pretty(&push_event).expect("invalid to string pretty"));

    Ok(Response::new(Either::Left(Full::default())))
}

impl<D> From<D> for Error where D: std::fmt::Display {
    fn from(value: D) -> Self {
        eprintln!("{value}");
        Self::InternalError
    }
}

impl IntoResponse for Value {
    fn into_response(self) -> Response {
        let vec = serde_json::to_vec(&self).expect("invalid json response");
        Response::new(Either::Left(Full::new(Bytes::from(vec))))
    }
}

