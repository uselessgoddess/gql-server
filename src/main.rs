use std::error::Error;
use std::future::Future;
use std::sync::Arc;

use actix_web::{App, FromRequest, get, guard, HttpResponse, HttpServer, Responder, web};
use actix_web::rt::time::Instant;
use actix_web::web::Data;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql::http::GraphQLPlaygroundConfig;
use async_graphql::http::playground_source;
use async_graphql_actix_web::{Request, Response};
use async_std::sync::RwLock;
use doublets::doublets::mem::united::Links;
use doublets::mem::{HeapMem, ResizeableMem};

mod model;

type LinksSchema = Schema<
    model::Query<usize, Links<usize, HeapMem>>,
    model::Mutation<usize, Links<usize, HeapMem>>,
    EmptySubscription,
>;

async fn index(schema: web::Data<LinksSchema>, req: Request) -> Response {
    schema.execute(req.into_inner()).await.into()
}

async fn index_playground() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
        )))
}

#[deny(unused_must_use)]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mem = HeapMem::new();
    let links = Links::<usize, _>::new(mem);
    let lock = RwLock::new(links);
    let arc = Arc::new(lock);

    let schema = Schema::new(
        model::Query::new(arc.clone()),
        model::Mutation::new(arc.clone()),
        EmptySubscription,
    );

    println!("Playground: http://localhost:1410");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(schema.clone()))
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/").guard(guard::Get()).to(index_playground))
    })
        .bind("localhost:1410")?
        .run()
        .await
}