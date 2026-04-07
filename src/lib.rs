mod error;
pub mod types;

pub use error::Error;
pub use types::*;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use reqwest::Client;
use serde::Deserialize;
use std::{future::Future, net::SocketAddr, sync::Arc};

const TG_API: &str = "https://api.telegram.org";

pub async fn serve<F, Fut>(token: &str, url: &str, handler: F) -> Result<(), Error>
where
    F: Fn(Update) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    Bot::new(token, url).serve(handler).await
}

pub async fn set(token: &str, url: &str) -> Result<(), Error> {
    Bot::new(token, url).set().await
}

pub async fn delete(token: &str) -> Result<(), Error> {
    Bot::new(token, "").delete().await
}

pub async fn info(token: &str) -> Result<WebhookInfo, Error> {
    Bot::new(token, "").info().await
}

#[derive(Clone)]
pub struct Bot {
    token:           String,
    url:             String,
    path:            String,
    port:            u16,
    secret:          Option<String>,
    max_connections: Option<i64>,
    allowed_updates: Vec<String>,
    drop_pending:    bool,
    client:          Client,
}

impl Bot {
    pub fn new(token: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            token:           token.into(),
            url:             url.into(),
            path:            "/webhook".into(),
            port:            8080,
            secret:          None,
            max_connections: None,
            allowed_updates: vec![],
            drop_pending:    false,
            client:          Client::new(),
        }
    }

    pub fn port(mut self, port: u16) -> Self { self.port = port; self }
    pub fn path(mut self, path: impl Into<String>) -> Self { self.path = path.into(); self }
    pub fn secret(mut self, s: impl Into<String>) -> Self { self.secret = Some(s.into()); self }
    pub fn max_connections(mut self, n: i64) -> Self { self.max_connections = Some(n); self }
    pub fn drop_pending_updates(mut self) -> Self { self.drop_pending = true; self }

    pub fn allowed_updates(mut self, updates: Vec<impl Into<String>>) -> Self {
        self.allowed_updates = updates.into_iter().map(Into::into).collect();
        self
    }

    fn endpoint(&self, method: &str) -> String {
        format!("{}/bot{}/{}", TG_API, self.token, method)
    }

    async fn call<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        body: serde_json::Value,
    ) -> Result<T, Error> {
        let resp: TgResp<T> = self.client
            .post(self.endpoint(method))
            .json(&body)
            .send().await?
            .json().await?;

        if resp.ok {
            resp.result.ok_or_else(|| Error::Other("ok=true but result is null".into()))
        } else {
            Err(Error::Api {
                code:        resp.error_code.unwrap_or(0),
                description: resp.description.unwrap_or_else(|| "unknown".into()),
            })
        }
    }

    pub async fn set(&self) -> Result<(), Error> {
        let url = format!("{}{}", self.url.trim_end_matches('/'), self.path);
        let mut body = serde_json::json!({ "url": url });

        if let Some(ref s) = self.secret     { body["secret_token"]        = serde_json::json!(s); }
        if let Some(n) = self.max_connections { body["max_connections"]     = serde_json::json!(n); }
        if !self.allowed_updates.is_empty()   { body["allowed_updates"]     = serde_json::json!(self.allowed_updates); }
        if self.drop_pending                  { body["drop_pending_updates"] = serde_json::json!(true); }

        let ok: bool = self.call("setWebhook", body).await?;
        if ok { println!("webhook set -> {url}"); }
        Ok(())
    }

    pub async fn delete(&self) -> Result<(), Error> {
        let ok: bool = self.call(
            "deleteWebhook",
            serde_json::json!({ "drop_pending_updates": true }),
        ).await?;
        if ok { println!("webhook deleted"); }
        Ok(())
    }

    pub async fn info(&self) -> Result<WebhookInfo, Error> {
        self.call("getWebhookInfo", serde_json::json!({})).await
    }

    pub async fn serve<F, Fut>(&self, handler: F) -> Result<(), Error>
    where
        F: Fn(Update) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.set().await?;

        let state = Arc::new(ServerState {
            handler:      Arc::new(move |u| Box::pin(handler(u))),
            secret_token: self.secret.clone(),
        });

        let app = Router::new()
            .route(&self.path, post(receive_update))
            .with_state(state);

        let addr: SocketAddr = ([0, 0, 0, 0], self.port).into();
        println!("listening on 0.0.0.0:{}", self.port);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| Error::Other(format!("bind failed: {e}")))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| Error::Other(format!("serve error: {e}")))?;

        Ok(())
    }
}

#[derive(Deserialize)]
struct TgResp<T> {
    ok:           bool,
    result:       Option<T>,
    error_code:   Option<i64>,
    description:  Option<String>,
}

type BoxedHandler = Arc<
    dyn Fn(Update) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync
>;

struct ServerState {
    handler:      BoxedHandler,
    secret_token: Option<String>,
}

async fn receive_update(
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
    Json(update): Json<Update>,
) -> StatusCode {
    if let Some(ref expected) = state.secret_token {
        let got = headers
            .get("x-telegram-bot-api-secret-token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if got != expected {
            return StatusCode::FORBIDDEN;
        }
    }

    // return 200 right away; Telegram retries if it doesn't get 2xx
    let handler = Arc::clone(&state.handler);
    tokio::spawn(async move { (handler)(update).await; });

    StatusCode::OK
}
