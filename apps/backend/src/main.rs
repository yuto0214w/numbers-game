use std::{error::Error, io, net::SocketAddr};

use axum::{
    Router, middleware,
    routing::{get, post},
};
use hyper::{Request, body::Incoming};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server,
};
use tokio::runtime::Builder as TokioRuntimeBuilder;
use tower::{Service as _, util::ServiceExt as _};

use self::util::{log_error, unwrap_infallible};

mod game;
mod util;

fn main() -> Result<(), Box<dyn Error>> {
    let rt = TokioRuntimeBuilder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(async {
        #[cfg(windows)]
        {
            util::windows_setup();
        }
        let app = Router::new()
            .route("/", get(game::http::server_info))
            .nest(
                "/rooms",
                Router::new()
                    .route("/", get(game::http::room_list))
                    .route("/new", post(game::http::new_room))
                    .nest(
                        "/{room_id}",
                        Router::new()
                            .route("/", get(game::http::room_info))
                            .route("/ws", get(game::http::serve_ws))
                            .layer(middleware::from_fn(game::http::room_existence_check)),
                    ),
            )
            .layer(middleware::from_fn(util::log_http_middleware));
        let addr = {
            #[cfg(debug_assertions)]
            {
                println!("This is a debug build.");
                SocketAddr::from(([127, 0, 0, 1], 3000))
            }
            #[cfg(not(debug_assertions))]
            {
                println!("This is a release build.");
                SocketAddr::from(([0, 0, 0, 0], 3000))
            }
        };
        println!("Listening at {}:{}", addr.ip(), addr.port());
        let mut make_service = app.into_make_service_with_connect_info::<SocketAddr>();
        let listener = tokio::net::TcpListener::bind(addr).await?;
        loop {
            let (socket, remote_addr) = listener.accept().await?;
            let tower_service = unwrap_infallible(make_service.call(remote_addr).await);
            tokio::spawn(async move {
                let socket = TokioIo::new(socket);
                let hyper_service =
                    hyper::service::service_fn(move |request: Request<Incoming>| {
                        tower_service.clone().oneshot(request)
                    });
                let mut hyper_server = server::conn::auto::Builder::new(TokioExecutor::new());
                hyper_server.http1().title_case_headers(true);
                if let Err(error) = hyper_server
                    .serve_connection_with_upgrades(socket, hyper_service)
                    .await
                {
                    if !error
                        .downcast_ref::<io::Error>()
                        .is_some_and(|e| e.kind() == io::ErrorKind::UnexpectedEof)
                    {
                        log_error!("serve", error);
                    }
                }
            });
        }
    })
}
