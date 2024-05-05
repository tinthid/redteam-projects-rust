use axum::{
    body::{Bytes, Body},
    http::{StatusCode, Request},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use maud::{html, Markup};
use std::{fs, io};

#[tokio::main]
async fn main() {
    
    let app = Router::new()
        .fallback(get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

async fn handler(req: Request<Body>) -> Markup {

    let mut path = req.uri().path()[1..].to_string();
    path.insert_str(0, "./");

    let metadata = match fs::metadata(path.clone()) {
        Ok(metadata) => metadata,
        Err(_) => {
            return html! {
                h1 { (format!("Error accessing {}", path)) }
            };
        }
    };


    let entries = fs::read_dir("./").unwrap()
    .map(|res| res.map(|e| (e.file_name(), e.path().is_dir())))
    .collect::<Result<Vec<_>, _>>().unwrap();

    html! {
        h1 { (path) }
        ol {
            @for e in &entries { 
                li { (e.0.clone().into_string().unwrap()) }
            }
        }
    }
}
