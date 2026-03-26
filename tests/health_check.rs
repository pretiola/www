use actix_web::rt::spawn;
use pretiola::startup::run;
use std::net::TcpListener;

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to bind address");
    let _ = spawn(server);
    format!("http://127.0.0.1:{}", port)
}

#[actix_web::test]
async fn health_check_works() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let body = response.text().await.expect("Failed to get body");
    assert!(body.contains("Pretiola"));
    assert!(body.contains("Driving Real Change"));
    assert!(body.contains("Advisory"));
}

#[actix_web::test]
async fn all_pages_return_200() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let pages = [
        "/",
        "/index.html",
        "/terms.html",
        "/privacy.html",
    ];

    for page in pages {
        let response = client
            .get(&format!("{}{}", &address, page))
            .send()
            .await
            .unwrap_or_else(|_| panic!("Failed to request {}", page));
        assert!(
            response.status().is_success(),
            "Page {} returned {}",
            page,
            response.status()
        );
    }
}

#[actix_web::test]
async fn nonexistent_page_returns_404() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/nonexistent.html", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), 404);
}

#[actix_web::test]
async fn sitemap_returns_valid_xml() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/sitemap.xml", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(content_type.contains("xml"));

    let body = response.text().await.expect("Failed to get body");
    assert!(body.contains("<urlset"));
    assert!(body.contains("pretiola.org"));
    // Should include content pages
    assert!(body.contains("index.html"));
    assert!(body.contains("terms.html"));
    assert!(body.contains("privacy.html"));
    // Should NOT include partials
    assert!(!body.contains("navbar.html"));
    assert!(!body.contains("footer.html"));
}

#[actix_web::test]
async fn head_requests_work() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let pages = ["/", "/index.html", "/sitemap.xml"];

    for page in pages {
        let response = client
            .head(&format!("{}{}", &address, page))
            .send()
            .await
            .unwrap_or_else(|_| panic!("HEAD request failed for {}", page));
        assert!(
            response.status().is_success(),
            "HEAD {} returned {}",
            page,
            response.status()
        );
    }
}
