use crate::routes::{dynamic_page, index, sitemap};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use std::net::TcpListener;
use tera::Tera;

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    // Initialize Tera with all HTML files in templates directory
    let tera = Tera::new("templates/**/*").expect("Failed to parse tera templates");
    // To optionally disable autoescape to prevent SSI content getting escaped if needed:
    // We leave autoescape for HTML, but templates like `navbar.html` are also HTML. Tera standard is safe.

    let tera = web::Data::new(tera);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(tera.clone())
            // Configure static files serving from /static
            .service(actix_files::Files::new("/static", "./static"))
            // Additionally map /pictures directly for old HTML references
            .service(actix_files::Files::new("/pictures", "./static/pictures"))
            // Handle main index route
            .service(
                web::resource("/")
                    .route(web::get().to(index))
                    .route(web::head().to(index)),
            )
            // Sitemap XML route
            .service(
                web::resource("/sitemap.xml")
                    .route(web::get().to(sitemap))
                    .route(web::head().to(sitemap)),
            )
            // Handle dynamic page routes
            .service(
                web::resource("/{page}.html")
                    .route(web::get().to(dynamic_page))
                    .route(web::head().to(dynamic_page)),
            )
            // Fallback for static items at root level (e.g. /robots.txt)
            .service(actix_files::Files::new("/", "./static"))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
