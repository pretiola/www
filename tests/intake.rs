use actix_web::{rt::spawn, web};
use pretiola::{forms::SiteConfig, inquiries::Store, startup::run_with_store};
use std::collections::HashMap;
use std::net::TcpListener;
async fn app() -> (String, reqwest::Client, web::Data<Store>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let store = web::Data::new(Store::memory());
    let server = run_with_store(
        listener,
        store.clone(),
        SiteConfig {
            origin: address.clone(),
            production: false,
            trust_fly_proxy: false,
        },
    )
    .unwrap();
    spawn(server);
    (
        address,
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap(),
        store,
    )
}
async fn form(
    client: &reqwest::Client,
    address: &str,
    page: &str,
) -> (String, HashMap<String, String>) {
    let response = client
        .get(format!("{address}/{page}.html"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let cookie = response.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let html = response.text().await.unwrap();
    let token = html
        .split("name=\"token\" value=\"")
        .nth(1)
        .expect("native form token")
        .split('"')
        .next()
        .unwrap();
    let mut form = HashMap::new();
    for (key, value) in [
        ("token", token),
        ("name", "Test Person"),
        ("email", "synthetic@example.org"),
        ("message", "A synthetic test inquiry."),
        ("consent", "yes"),
    ] {
        form.insert(key.into(), value.into());
    }
    if page == "contribute" {
        form.insert("mode".into(), "professional".into());
    } else {
        form.insert("organization".into(), "Example Ministry".into());
        form.insert("country".into(), "Canada".into());
    }
    (cookie, form)
}
#[actix_web::test]
async fn native_forms_save_redirect_and_deduplicate() {
    let (address, client, _) = app().await;
    for (page, kind) in [
        ("contribute", "contributor"),
        ("for_ministries", "ministry"),
    ] {
        let (cookie, f) = form(&client, &address, page).await;
        let send = || {
            client
                .post(format!("{address}/inquiries/{kind}"))
                .header("Cookie", &cookie)
                .form(&f)
                .send()
        };
        let response = send().await.unwrap();
        assert_eq!(response.status(), 303);
        let location = response.headers()["location"].to_str().unwrap().to_owned();
        let again = send().await.unwrap();
        assert_eq!(again.status(), 303);
        assert_eq!(again.headers()["location"], location);
        let receipt = client
            .get(format!("{address}{location}"))
            .send()
            .await
            .unwrap();
        assert_eq!(receipt.status(), 200);
        let html = receipt.text().await.unwrap();
        assert!(html.contains("Your inquiry has been saved"));
        assert!(!html.contains("synthetic@example.org"));
    }
    assert_eq!(
        client
            .get(format!("{address}/inquiries/received"))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );
}
#[actix_web::test]
async fn errors_preserve_and_escape_input_without_saving() {
    let (address, client, _) = app().await;
    let (cookie, mut f) = form(&client, &address, "contribute").await;
    f.insert("email".into(), "not an email".into());
    f.insert("message".into(), "<script>alert('x')</script>".into());
    let response = client
        .post(format!("{address}/inquiries/contributor"))
        .header("Cookie", cookie)
        .form(&f)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 422);
    let html = response.text().await.unwrap();
    assert!(html.contains("not an email"));
    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("<script>alert"));
    assert!(html.contains("aria-invalid=\"true\""));
}
#[actix_web::test]
async fn cross_site_tokens_and_oversized_requests_are_rejected() {
    let (address, client, _) = app().await;
    let (cookie, f) = form(&client, &address, "contribute").await;
    let url = format!("{address}/inquiries/contributor");
    assert_eq!(
        client
            .post(&url)
            .header("Cookie", &cookie)
            .header("Origin", "https://evil.example")
            .form(&f)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        client.post(&url).form(&f).send().await.unwrap().status(),
        403
    );
    assert_eq!(
        client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body("message=".to_owned() + &"a".repeat(70000))
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
}
#[actix_web::test]
async fn spoofed_forwarding_headers_do_not_bypass_network_limits() {
    let (address, client, _) = app().await;
    for i in 0..6 {
        let (cookie, mut f) = form(&client, &address, "contribute").await;
        f.insert("email".into(), format!("test{i}@example.org"));
        f.insert("message".into(), format!("Synthetic inquiry {i}"));
        let r = client
            .post(format!("{address}/inquiries/contributor"))
            .header("Cookie", cookie)
            .header("X-Forwarded-For", format!("192.0.2.{i}"))
            .header("Fly-Client-IP", format!("192.0.2.{i}"))
            .form(&f)
            .send()
            .await
            .unwrap();
        assert_eq!(r.status().as_u16(), if i < 5 { 303 } else { 429 });
    }
}
