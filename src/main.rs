use http::uri::{Authority, Scheme};
use http::{HeaderValue, Uri};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Display;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Middleware {
    name: String,
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
    T: Display,
{
    fn to_status_response(&self, status: u16) -> Response<Body> {
        Response::builder()
            .status(status)
            .body(self.to_string().into())
            .unwrap()
    }
}

async fn proxy(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Get destionation for the request based on the host header
    let proxy_map = PROXY_MAP.read().await;
    // We unwrap here since Host is a required HTTP request header
    let proxy_dest = {
        let dest = proxy_map.0.get(
            req.headers()
                .get("Host")
                .unwrap()
                .to_str()
                .unwrap_or_default(),
        );
        match dest {
            Some(v) => v,
            // If there is no proxy destination for this host send a 404 back
            None => return Ok("Route was not found".to_status_response(404)),
        }
    };

    // Rewrite the URI part of the request
    let (mut parts, body) = req.into_parts();
    let mut uri_parts = parts.uri.into_parts();
    // from_str shouldnt be able to fail here
    uri_parts.authority = Some(Authority::from_str(&proxy_dest.destination).unwrap());
    uri_parts.scheme = Some(Scheme::HTTP);
    // and neither should this, since we set both authority and scheme
    parts.uri = match Uri::from_parts(uri_parts) {
        Ok(v) => v,
        Err(e) => return Ok(e.to_status_response(500)),
    };
    let req = Request::from_parts(parts, body);

    // Send out the request to the actual service
    let res = Client::new().request(req).await;
    // If this fails, for example if the destination is not listening, send a 500 back, with the
    // error casted to a string as the body
    match res {
        Ok(r) => Ok(r),
        Err(e) => Ok(e.to_status_response(500)),
    }
}

const SERVER_HEADER_VALUE: HeaderValue = HeaderValue::from_static("rProx");

fn set_proxy_headers(mut res: Response<Body>) -> Result<Response<Body>, Infallible> {
    res.headers_mut().insert("Server", SERVER_HEADER_VALUE);
    Ok(res)
}

fn load_proxy_map<'a, P: AsRef<Path>>(path: P) -> ProxyMap {
    let fd = std::fs::OpenOptions::new().read(true).open(path).unwrap();
    serde_yaml::from_reader(fd).unwrap()
}

use tokio::sync::{broadcast, mpsc};

async fn start_proxy(addr: SocketAddr, mut shutdown: broadcast::Receiver<()>) {
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(service_fn(|req| async move {
            set_proxy_headers(proxy(req).await.unwrap())
        }))
    });

    let server = Server::bind(&addr).serve(make_svc);

    tokio::select! {
        _ = shutdown.recv() => {
            return
        }
        r = server  => {
            match r {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("ERR: Server failed: {}", e)
                },
            }
        }
    }
}

use notify::Watcher;

async fn start_watchdog<P: AsRef<Path>>(path: P, mut shutdown: broadcast::Receiver<()>) {
    let (tx, mut rx) = mpsc::channel(100);
    let mut watcher = notify::RecommendedWatcher::new(move |result: Result<notify::Event, notify::Error>| {
        tx.blocking_send(result).expect("couldnt send");
    }).unwrap();

    watcher.watch(path.as_ref(), notify::RecursiveMode::NonRecursive).unwrap();

    loop {
        tokio::select! {
            ev = rx.recv() => {
                match ev {
                    Some(Ok(notify::Event { kind: notify::EventKind::Modify(_), .. } )) => {
                        let mut guard = PROXY_MAP.write().await;
                        *guard = load_proxy_map(path.as_ref());
                    }
                    e => {
                        eprintln!("Unhandled file event: {:?}", e);
                    }
                }
            }
            _ = shutdown.recv() => {
                return
            }
        }
    }
}


#[tokio::main]
async fn main() {
    {
        let mut guard = PROXY_MAP.write().await;
        *guard = load_proxy_map("test.yaml");
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

    let (shutdown_send, proxy_shutdown) = broadcast::channel(1);
    let watchdog_shutdown = shutdown_send.subscribe();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    tokio::spawn(async move {
        start_proxy(addr, proxy_shutdown).await;
    });
    tokio::spawn(async move {
        start_watchdog("test.yaml", watchdog_shutdown).await;
    });

    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            shutdown_send.send(())
        },
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
            shutdown_send.send(())
        },
    }.unwrap();
}
