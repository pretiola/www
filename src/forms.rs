use crate::{
    inquiries::{now, Inquiry, IntakeError, Store},
    routes::context,
};
use actix_web::{
    cookie::{time::Duration, Cookie, SameSite},
    http::StatusCode,
    web, HttpRequest, HttpResponse,
};
use std::{collections::BTreeMap, net::IpAddr};
use tera::Tera;
use uuid::Uuid;

#[derive(Clone)]
pub struct SiteConfig {
    pub origin: String,
    pub production: bool,
    pub trust_fly_proxy: bool,
}
impl SiteConfig {
    pub fn from_env() -> Result<Self, std::io::Error> {
        let production = std::env::var("APP_ENV").as_deref() == Ok("production");
        let origin = std::env::var("PUBLIC_ORIGIN").unwrap_or_else(|_| {
            format!(
                "http://localhost:{}",
                std::env::var("PORT").unwrap_or("8080".into())
            )
        });
        let origin = origin.trim_end_matches('/').to_string();
        if (production && !origin.starts_with("https://")) || origin.contains(['\r', '\n']) {
            return Err(std::io::Error::other(
                "Set PUBLIC_ORIGIN to the canonical HTTPS origin in production",
            ));
        }
        Ok(Self {
            origin,
            production,
            trust_fly_proxy: production
                && std::env::var("TRUST_FLY_PROXY").as_deref() == Ok("true"),
        })
    }
}
fn kind(page: &str) -> Option<&'static str> {
    match page {
        "contribute" => Some("contributor"),
        "for_ministries" => Some("ministry"),
        _ => None,
    }
}
fn session(req: &HttpRequest) -> String {
    req.cookie("pretiola_form")
        .filter(|c| Uuid::parse_str(c.value()).is_ok())
        .map(|c| c.value().to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}
fn network(req: &HttpRequest, cfg: &SiteConfig) -> String {
    // Never trust X-Forwarded-For. Fly mode is opt-in and requires no direct origin access.
    let ip = if cfg.trust_fly_proxy {
        req.headers()
            .get("Fly-Client-IP")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<IpAddr>().ok())
    } else {
        req.peer_addr().map(|a| a.ip())
    };
    match ip {
        Some(IpAddr::V4(ip)) => ip.to_string(),
        Some(IpAddr::V6(ip)) => format!(
            "{:x}:{:x}:{:x}:{:x}::/64",
            ip.segments()[0],
            ip.segments()[1],
            ip.segments()[2],
            ip.segments()[3]
        ),
        None => "unknown".into(),
    }
}
pub fn render_form(
    req: &HttpRequest,
    tera: &Tera,
    store: &Store,
    cfg: &SiteConfig,
    page: &str,
    mut form: Inquiry,
    errors: BTreeMap<String, String>,
    status: StatusCode,
) -> HttpResponse {
    let session = session(req);
    if !store.check_token(&form.token, &session, kind(page).unwrap(), now()) {
        form.token = store.token(&session, kind(page).unwrap(), now());
    }
    let mut ctx = context(page, cfg);
    ctx.insert("form", &form);
    ctx.insert("errors", &errors);
    ctx.insert("kind", kind(page).unwrap());
    let title = if page == "contribute" {
        "Contribute your experience"
    } else {
        "For ministries"
    };
    ctx.insert("title", title);
    ctx.insert("description", title);
    match tera.render(&format!("{page}.html"), &ctx) {
        Ok(html) => HttpResponse::build(status)
            .content_type("text/html; charset=utf-8")
            .insert_header(("Cache-Control", "no-store"))
            .insert_header((
                "Retry-After",
                if status == StatusCode::TOO_MANY_REQUESTS {
                    "3600"
                } else {
                    "0"
                },
            ))
            .cookie(
                Cookie::build("pretiola_form", session)
                    .path("/")
                    .http_only(true)
                    .same_site(SameSite::Lax)
                    .secure(cfg.production)
                    .max_age(Duration::days(1))
                    .finish(),
            )
            .body(html),
        Err(_) => {
            log::error!("Form template failed to render");
            HttpResponse::InternalServerError()
                .body("The form is unavailable. Please email contact@pretiola.org.")
        }
    }
}
pub async fn form_page(
    req: HttpRequest,
    tera: web::Data<Tera>,
    store: web::Data<Store>,
    cfg: web::Data<SiteConfig>,
) -> HttpResponse {
    let page = if req.path() == "/contribute.html" {
        "contribute"
    } else {
        "for_ministries"
    };
    render_form(
        &req,
        &tera,
        &store,
        &cfg,
        page,
        Inquiry::default(),
        BTreeMap::new(),
        StatusCode::OK,
    )
}
pub async fn submit(
    req: HttpRequest,
    path: web::Path<String>,
    body: Result<web::Form<Inquiry>, actix_web::Error>,
    tera: web::Data<Tera>,
    store: web::Data<Store>,
    cfg: web::Data<SiteConfig>,
) -> HttpResponse {
    let kind = path.into_inner();
    let page = match kind.as_str() {
        "contributor" => "contribute",
        "ministry" => "for_ministries",
        _ => return HttpResponse::NotFound().finish(),
    };
    let network = network(&req, &cfg);
    let Ok(mut form) = body.map(|f| f.into_inner()) else {
        return HttpResponse::BadRequest().content_type("text/html; charset=utf-8").body("<h1>We could not read this form</h1><p>The request may be too large or incorrectly formatted. Use your browser’s Back button to keep your text and shorten it, or email contact@pretiola.org.</p>");
    };
    form.normalize();
    let mut errors = BTreeMap::new();
    let mut status = StatusCode::UNPROCESSABLE_ENTITY;
    if !store.allow_attempt(&network, now()) {
        errors.insert("form".into(),"Too many attempts from this connection. Your text is below. Wait ten minutes before trying again, or email contact@pretiola.org.".into());
        status = StatusCode::TOO_MANY_REQUESTS;
    } else if req
        .headers()
        .get("Origin")
        .is_some_and(|v| v.to_str().ok() != Some(&cfg.origin))
        || req
            .headers()
            .get("Sec-Fetch-Site")
            .is_some_and(|v| v == "cross-site")
    {
        errors.insert(
            "form".into(),
            "Please open this form directly on our site and try again.".into(),
        );
        status = StatusCode::FORBIDDEN;
    } else if !store.check_token(&form.token, &session(&req), &kind, now()) {
        errors.insert("form".into(),"Your form session expired or cookies were unavailable. Your text is preserved below. Allow the essential form cookie, then submit again.".into());
        status = StatusCode::FORBIDDEN;
    } else {
        errors = form.validate(&kind);
    }
    if !errors.is_empty() {
        return render_form(&req, &tera, &store, &cfg, page, form, errors, status);
    }
    let data = form.clone();
    let state = store.clone();
    match web::block(move || state.save(&kind, &data, &network, now())).await {
        Ok(Ok(id)) => HttpResponse::SeeOther()
            .insert_header((
                "Location",
                format!("/inquiries/received?receipt={}", store.receipt(&id)),
            ))
            .insert_header(("Cache-Control", "no-store"))
            .finish(),
        failure => {
            let limited = matches!(failure, Ok(Err(IntakeError::Limited)));
            if !limited {
                log::error!("Inquiry could not be saved; inspect storage health and capacity");
            }
            errors.insert("form".into(),if limited {"We have reached a submission limit. Your text is preserved below. Please try later or email contact@pretiola.org."} else {"We could not save your inquiry. Your text is preserved below. Please retry later or email contact@pretiola.org."}.into());
            render_form(
                &req,
                &tera,
                &store,
                &cfg,
                page,
                form,
                errors,
                if limited {
                    StatusCode::TOO_MANY_REQUESTS
                } else {
                    StatusCode::SERVICE_UNAVAILABLE
                },
            )
        }
    }
}
#[derive(serde::Deserialize)]
pub struct Receipt {
    receipt: Option<String>,
}
pub async fn received(
    query: web::Query<Receipt>,
    tera: web::Data<Tera>,
    store: web::Data<Store>,
    cfg: web::Data<SiteConfig>,
) -> HttpResponse {
    if !query
        .receipt
        .as_deref()
        .is_some_and(|s| store.check_receipt(s))
    {
        return HttpResponse::NotFound()
            .body("No submission confirmation is available at this address.");
    }
    let mut ctx = context("received", &cfg);
    ctx.insert("title", "Your inquiry is saved");
    ctx.insert("description", "Inquiry confirmation");
    ctx.insert(
        "reference",
        query.receipt.as_ref().unwrap().split('.').next().unwrap(),
    );
    match tera.render("received.html",&ctx){Ok(html)=>HttpResponse::Ok().insert_header(("Cache-Control","no-store")).insert_header(("X-Robots-Tag","noindex, nofollow")).content_type("text/html; charset=utf-8").body(html),Err(_)=>HttpResponse::InternalServerError().body("Your inquiry was saved, but confirmation is unavailable. Please contact contact@pretiola.org.")}
}
