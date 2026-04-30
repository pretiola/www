use crate::routes::{dynamic_page, index, sitemap};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use std::net::TcpListener;
use tera::Tera;

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let tera = Tera::new("templates/**/*").expect("Failed to parse tera templates");
    let tera = web::Data::new(tera);

    if std::env::var("WEB3FORMS_ACCESS_KEY").ok().filter(|s| !s.is_empty()).is_none() {
        log::warn!(
            "WEB3FORMS_ACCESS_KEY is not set; the ministry intake form will fail until it is configured"
        );
    }

    let server = HttpServer::new(move || {
        App::new()
            .app_data(tera.clone())
            .service(actix_files::Files::new("/static", "./static"))
            .service(actix_files::Files::new("/pictures", "./static/pictures"))
            .service(
                web::resource("/")
                    .route(web::get().to(index))
                    .route(web::head().to(index)),
            )
            .service(
                web::resource("/sitemap.xml")
                    .route(web::get().to(sitemap))
                    .route(web::head().to(sitemap)),
            )
            .service(
                web::resource("/{page}.html")
                    .route(web::get().to(dynamic_page))
                    .route(web::head().to(dynamic_page)),
            )
            .service(actix_files::Files::new("/", "./static"))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
