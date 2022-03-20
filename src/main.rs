use http::uri::{Authority, Scheme};
use anyhow::Result;
use http::Uri;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server};
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::fmt::Display;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Middleware {
    name: String
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ProxyDest {
    destination: String,
    middleware: Vec<Middleware>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ProxyMap(HashMap<String, ProxyDest>);

lazy_static::lazy_static! {
    static ref PROXY_MAP: Arc<RwLock<ProxyMap>> = {
        let m = Arc::new(RwLock::new(ProxyMap(HashMap::new())));
        m
    };
}

trait StatusResponse {
    fn to_status_response(&self, status: u16) -> Response<Body>;
}

impl<T> StatusResponse for T
where
    T: Display
{
    fn to_status_response(&self, status: u16) -> Response<Body> {
        Response::builder().status(status).body(self.to_string().into()).unwrap()
    }
}

async fn proxy(req: Request<Body>) -> Result<Response<Body>> {
    // Get destionation for the request based on the host header
    let proxy_map = PROXY_MAP.read().await;
    // We unwrap here since Host is a required HTTP request header
    let proxy_dest = proxy_map.0.get(req.headers().get("Host").unwrap().to_str()?);
    let proxy_dest = match proxy_dest {
        Some(v) => v,
        // If there is no proxy destination for this host send a 404 back
        None => {
            return Ok("Route was not found".to_status_response(404))
        }
    };

    // Rewrite the URI part of the request
    let (mut parts, body) = req.into_parts();
    let mut uri_parts = parts.uri.into_parts();
    uri_parts.authority = Some(Authority::from_str(&proxy_dest.destination).unwrap());
    uri_parts.scheme = Some(Scheme::HTTP);
    parts.uri = Uri::from_parts(uri_parts).unwrap();
    let req = Request::from_parts(parts, body);

    // Send out the request to the actual service
    let res = Client::new().request(req).await;
    // If this fails, for example if the destination is not listening, send a 500 back, with the
    // error casted to a string as the body
    match res {
        Ok(r) => Ok(r),
        Err(e) => Ok(e.to_status_response(500))
    }
}

async fn load_proxy_map<'a, P: AsRef<Path>>(path: P) -> ProxyMap {
    let fd = std::fs::OpenOptions::new().read(true).open(path).unwrap();
    serde_yaml::from_reader(fd).unwrap()
}

#[tokio::main]
async fn main() {
    {
        let mut guard = PROXY_MAP.write().await;
        *guard = load_proxy_map("test.yaml").await;
    }


    let port = if let Ok(true) = caps::has_cap(
        None,
        caps::CapSet::Permitted,
        caps::Capability::CAP_NET_BIND_SERVICE,
    ) {
        80
    } else {
        8000
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, hyper::Error>(service_fn(proxy)) });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("ERR: Server failed: {}", e);
    }
}
