use actix_web::{web, HttpResponse, Responder};
use chrono::{Datelike, Utc};
use tera::Tera;

pub async fn index(tera: web::Data<Tera>) -> impl Responder {
    render_page("index", tera)
}

pub async fn dynamic_page(path: web::Path<String>, tera: web::Data<Tera>) -> impl Responder {
    let page = path.into_inner();
    if page == "index" {
        return HttpResponse::PermanentRedirect()
            .append_header(("Location", "/"))
            .finish();
    }
    if !PUBLIC_PAGES.contains(&page.as_str()) {
        return HttpResponse::NotFound().body("Page not found");
    }
    render_page(&page, tera)
}

fn render_page(page: &str, tera: web::Data<Tera>) -> HttpResponse {
    let template_name = format!("{}.html", page);
    let mut context = tera::Context::new();
    context.insert("page_name", page);
    context.insert("current_year", &Utc::now().year());
    context.insert(
        "web3forms_access_key",
        &std::env::var("WEB3FORMS_ACCESS_KEY").unwrap_or_default(),
    );

    match tera.render(&template_name, &context) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => {
            log::error!("Template rendering error: {}", e);
            HttpResponse::NotFound().body("Page not found")
        }
    }
}

const PUBLIC_PAGES: &[&str] = &["privacy", "terms"];
const BASE_URL: &str = "https://pretiola.org";

pub async fn sitemap() -> impl Responder {
    let mut urls = vec![format!("  <url><loc>{}/</loc></url>", BASE_URL)];
    for page in PUBLIC_PAGES {
        urls.push(format!(
            "  <url><loc>{}/{}.html</loc></url>",
            BASE_URL, page
        ));
    }
    HttpResponse::Ok().content_type("application/xml").body(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{}\n</urlset>", urls.join("\n")
    ))
}
