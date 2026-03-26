use actix_web::{web, HttpResponse, Responder};
use chrono::{Datelike, Utc};
use tera::Tera;

pub async fn index(tera: web::Data<Tera>) -> impl Responder {
    render_page("index", tera)
}

pub async fn dynamic_page(path: web::Path<String>, tera: web::Data<Tera>) -> impl Responder {
    render_page(&path.into_inner(), tera)
}

fn render_page(page: &str, tera: web::Data<Tera>) -> HttpResponse {
    let template_name = format!("{}.html", page);
    let mut context = tera::Context::new();
    context.insert("page_name", page);
    context.insert("current_year", &Utc::now().year());

    match tera.render(&template_name, &context) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => {
            log::error!("Template rendering error: {}", e);
            HttpResponse::NotFound().body("Page not found")
        }
    }
}

const PARTIALS: &[&str] = &["navbar.html", "footer.html"];
const BASE_URL: &str = "https://pretiola.org";

pub async fn sitemap(tera: web::Data<Tera>) -> impl Responder {
    let mut urls = Vec::new();
    urls.push(format!("  <url><loc>{}/</loc></url>", BASE_URL));

    let mut template_names: Vec<&str> = tera
        .get_template_names()
        .filter(|name| name.ends_with(".html") && !PARTIALS.contains(name))
        .collect();
    template_names.sort();

    for name in template_names {
        urls.push(format!("  <url><loc>{}/{}</loc></url>", BASE_URL, name));
    }

    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{}\n</urlset>\n",
        urls.join("\n")
    );
    HttpResponse::Ok().content_type("application/xml").body(xml)
}
