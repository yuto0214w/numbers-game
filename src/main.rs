use std::{error::Error, io, net::SocketAddr};

use axum::{middleware, routing::get, Router};
use hyper::{body::Incoming, Request};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server,
};
use tower::{util::ServiceExt as _, Service as _};

use self::util::{log_error, unwrap_infallible};

mod handler;
mod util;

fn main() -> Result<(), Box<dyn Error>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(async {
        #[cfg(windows)]
        {
            util::windows_setup();
        }
        let app = Router::new()
            .route("/", get(handler::file::serve_index_html))
            .nest(
                "/room",
                Router::new()
                    .route("/new", get(handler::game::http::new_room))
                    .route("/main.js", get(handler::file::serve_game_js))
                    .nest(
                        "/:room_id",
                        Router::new()
                            .route("/", get(handler::file::serve_game_html))
                            .route("/room_data", get(handler::game::http::room_data))
                            .route("/ws", get(handler::game::http::serve_ws))
                            .route("/join_top", get(handler::game::http::join_top))
                            .route("/join_bottom", get(handler::game::http::join_bottom))
                            .route("/leave/:private_id", get(handler::game::http::leave))
                            .layer(middleware::from_fn(
                                handler::game::http::room_existence_check,
                            )),
                    ),
            )
            .layer(middleware::from_fn(util::log_http_middleware));
        let addr = {
            #[cfg(debug_assertions)]
            {
                println!("This is debug build.");
                SocketAddr::from(([127, 0, 0, 1], 80))
            }
            #[cfg(not(debug_assertions))]
            {
                println!("This is release build.");
                SocketAddr::from(([0, 0, 0, 0], 80))
            }
        };
        println!("Serving at http://localhost/");
        let mut make_service = app.into_make_service_with_connect_info::<SocketAddr>();
        let listener = tokio::net::TcpListener::bind(addr).await?;
        loop {
            let (socket, remote_addr) = listener.accept().await.unwrap();
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
