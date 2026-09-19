use crate::forms::SiteConfig;
use actix_web::{web, HttpResponse, Responder};
use chrono::{Datelike, Utc};
use tera::Tera;

pub fn context(page: &str, cfg: &SiteConfig) -> tera::Context {
    let mut ctx = tera::Context::new();
    ctx.insert("page_name", page);
    ctx.insert("current_year", &Utc::now().year());
    ctx.insert("local_preview", &!cfg.production);
    ctx.insert(
        "canonical",
        &format!(
            "https://pretiola.org{}",
            if page == "index" {
                "/".into()
            } else {
                format!("/{page}.html")
            }
        ),
    );
    ctx.insert("title", "Pretiola");
    ctx.insert(
        "description",
        "Practical expertise in service of charitable missions.",
    );
    ctx
}
pub async fn index(tera: web::Data<Tera>, cfg: web::Data<SiteConfig>) -> HttpResponse {
    render_page("index", tera, &cfg)
}
pub async fn dynamic_page(
    path: web::Path<String>,
    tera: web::Data<Tera>,
    cfg: web::Data<SiteConfig>,
) -> HttpResponse {
    let page = path.into_inner();
    if page == "index" {
        return HttpResponse::PermanentRedirect()
            .insert_header(("Location", "/"))
            .finish();
    }
    if !["privacy", "terms"].contains(&page.as_str()) {
        return HttpResponse::NotFound().body("Page not found");
    }
    render_page(&page, tera, &cfg)
}
fn render_page(page: &str, tera: web::Data<Tera>, cfg: &SiteConfig) -> HttpResponse {
    match tera.render(&format!("{page}.html"), &context(page, cfg)) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(_) => {
            log::error!("Page template failed to render");
            HttpResponse::InternalServerError().body("This page is temporarily unavailable.")
        }
    }
}
pub async fn sitemap() -> impl Responder {
    let urls = [
        "/",
        "/contribute.html",
        "/for_ministries.html",
        "/privacy.html",
        "/terms.html",
    ]
    .iter()
    .map(|path| format!("<url><loc>https://pretiola.org{path}</loc></url>"))
    .collect::<Vec<_>>()
    .join("\n");
    HttpResponse::Ok().content_type("application/xml").body(format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">{urls}</urlset>"))
}
