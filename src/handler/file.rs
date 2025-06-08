use axum::{
    http::StatusCode,
    response::{IntoResponse as _, Response},
};

use crate::util::SimpleResponse;

macro_rules! create_function {
    ($func_name:ident, $file_name:expr, $mime:expr) => {
        pub async fn $func_name() -> Response {
            let content = {
                #[cfg(debug_assertions)]
                {
                    std::fs::read_to_string(concat!("web/dist/", $file_name)).unwrap()
                }
                #[cfg(not(debug_assertions))]
                {
                    include_str!(concat!("../../web/dist/", $file_name))
                }
            };
            SimpleResponse {
                status_code: StatusCode::OK,
                content_type: $mime,
                content,
            }
            .into_response()
        }
    };
}

create_function!(serve_index_html, "index.html", "text/html; charset=utf-8");
create_function!(serve_game_html, "game.html", "text/html; charset=utf-8");
create_function!(serve_game_js, "main.js", "text/javascript");
