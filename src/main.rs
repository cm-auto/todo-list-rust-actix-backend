use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer, Responder, Scope};

mod models;
mod prototype_db;
mod routes;

async fn get_api_index() -> impl Responder {
    "welcome to my api"
}

#[actix_web::main]
async fn main() {
    let bind_address = "0.0.0.0";
    let port = 1337;

    let api_prefix = "/api";

    let app_data = web::Data::new(prototype_db::Database::new("db".to_string()).unwrap());

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    println!("Listening on {}:{}", bind_address, port);
    HttpServer::new(move || {
        let entries_scope = Scope::new(&format!("{}/entries", api_prefix))
            .configure(routes::entry::configure_routes);
        let lists_scope =
            Scope::new(&format!("{}/lists", api_prefix)).configure(routes::list::configure_routes);

        App::new()
            .app_data(app_data.clone())
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Trim,
            ))
            .wrap(middleware::Logger::default())
            .wrap(Cors::permissive())
            .route(&format!("{}", api_prefix), web::get().to(get_api_index))
            .service(lists_scope)
            .service(entries_scope)
    })
    .bind((bind_address, port))
    .unwrap()
    .run()
    .await
    .unwrap()
}
