use axum::{
    body::Body,
    http::{StatusCode, Request},
    response::{self, IntoResponse, Response},
    routing::get,
    Router,
};
use maud::{html, Markup};
use std::fs;

#[tokio::main]
async fn main() {
    
    let app = Router::new()
        .fallback(get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

async fn handler(req: Request<Body>) -> Response {

    let mut path = req.uri().path()[1..].to_string();
    let link_path = path.clone();



    path.insert_str(0, "./");

    let metadata = match fs::metadata(path.clone()) {
        Ok(metadata) => metadata,
        Err(_) => {
            return response::Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("File not found"))
                .unwrap();
        }
    };

    if metadata.is_file() {
        let file_data = std::fs::read(path.clone()).unwrap();
        return response::Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(file_data))
                .unwrap();
    }


    let entries = fs::read_dir(path.clone()).unwrap()
    .map(|res| res.map(|e| e.file_name().into_string().unwrap() ))
    .collect::<Result<Vec<_>, _>>().unwrap();

    let markup = html! {
        h1 { (format!("/{}", link_path)) }
        ol {
            @for e in &entries { 
                @if link_path.is_empty() {
                    li { a href={(format!("/{}", e))} 
                        { (e) } }
                } @else {
                    li { a href={(format!("/{}/{}", link_path, e))} 
                        { (e) } }
                }
            }
        }
    };
    return Html(markup).into_response();
}

struct Html(Markup);

impl IntoResponse for Html {
    fn into_response(self) -> Response {
        response::Html(self.0.into_string()).into_response()
    }
}
