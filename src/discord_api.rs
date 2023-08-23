use once_cell::sync::Lazy;

static REQWEST_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| reqwest::Client::new());

#[derive(serde::Serialize)]
struct WebhookObject<'a> {
    content: &'a str,
}

pub async fn execute_webhook(url: &str, content: &str) -> anyhow::Result<String> {
    let body = WebhookObject { content };
    let request = REQWEST_CLIENT
        .request(reqwest::Method::POST, url)
        .json(&body)
        .build()?;
    Ok(REQWEST_CLIENT.execute(request).await?.text().await?)
}
