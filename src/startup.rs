use crate::{
    forms::{self, SiteConfig},
    inquiries::Store,
    notifications::{self, Delivery},
    routes::{dynamic_page, index, sitemap},
};
use actix_web::{dev::Server, middleware::DefaultHeaders, web, App, HttpServer};
use std::{net::TcpListener, path::Path};
use tera::Tera;

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let cfg = SiteConfig::from_env()?;
    let db_path = std::env::var("INTAKE_DB").unwrap_or("private/intake.sqlite3".into());
    if cfg.production
        && (!Path::new(&db_path).is_absolute()
            || std::env::var("PERSISTENT_STORAGE_CONFIRMED").as_deref() != Ok("true"))
    {
        return Err(std::io::Error::other("Production requires absolute INTAKE_DB on verified persistent storage and PERSISTENT_STORAGE_CONFIRMED=true"));
    }
    if cfg.production {
        let mounts = std::fs::read_to_string("/proc/self/mountinfo")?;
        if db_path != "/data/pretiola/intake.sqlite3"
            || !mounts
                .lines()
                .any(|line| line.split_whitespace().nth(4) == Some("/data"))
        {
            return Err(std::io::Error::other("Production requires a real persistent /data mount and INTAKE_DB=/data/pretiola/intake.sqlite3"));
        }
    }
    let store = web::Data::new(Store::open(Path::new(&db_path)).map_err(std::io::Error::other)?);
    let delivery = Delivery::from_env(cfg.production).map_err(std::io::Error::other)?;
    let server = run_with_store(listener, store.clone(), cfg)?;
    notifications::start_worker(store, delivery);
    Ok(server)
}
pub fn run_with_store(
    listener: TcpListener,
    store: web::Data<Store>,
    cfg: SiteConfig,
) -> Result<Server, std::io::Error> {
    let tera = web::Data::new(Tera::new("templates/**/*").map_err(std::io::Error::other)?);
    let cfg = web::Data::new(cfg);
    Ok(HttpServer::new(move || {
        App::new()
            .app_data(tera.clone())
            .app_data(store.clone())
            .app_data(cfg.clone())
            .app_data(web::FormConfig::default().limit(65536))
            .wrap(
                DefaultHeaders::new()
                    .add(("X-Content-Type-Options", "nosniff"))
                    .add(("Referrer-Policy", "no-referrer"))
                    .add(("X-Frame-Options", "DENY")),
            )
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
            .service(web::resource("/for_ministries.html").route(web::get().to(forms::form_page)))
            .service(web::resource("/contribute.html").route(web::get().to(forms::form_page)))
            .service(web::resource("/inquiries/received").route(web::get().to(forms::received)))
            .service(web::resource("/inquiries/{kind}").route(web::post().to(forms::submit)))
            .service(
                web::resource("/{page}.html")
                    .route(web::get().to(dynamic_page))
                    .route(web::head().to(dynamic_page)),
            )
            .service(actix_files::Files::new("/static", "./static"))
            .service(actix_files::Files::new("/pictures", "./static/pictures"))
            .service(actix_files::Files::new("/", "./static"))
    })
    .listen(listener)?
    .run())
}
