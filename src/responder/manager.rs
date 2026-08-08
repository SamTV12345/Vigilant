// Vigil
//
// Microservices Status Page
// Copyright: 2021, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use axum::{
    Router,
    body::Body,
    extract::Request,
    http::{StatusCode, header},
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post, put},
};
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, tower::StreamableHttpService,
};
use std::{sync::Arc, time::Duration};
use tera::Tera;
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::services::ServeDir;

use super::mcp;
use super::routes;
use crate::APP_CONF;

const MCP_SSE_KEEPALIVE_SECONDS: Duration = Duration::from_secs(30);

pub fn run() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(APP_CONF.server.workers)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        // Prepare templating engine (tera 2.x)
        let mut tera = Tera::default();
        let template_path = APP_CONF
            .assets
            .path
            .canonicalize()
            .unwrap()
            .join("templates")
            .join("index.tera");
        tera.add_template_file(&template_path, Some("index.tera"))
            .unwrap();

        let tera = Arc::new(tera);

        // Prepare MCP services (if enabled)
        let mcp_router = if APP_CONF.server.mcp_server {
            Some(build_mcp_router())
        } else {
            info!("mcp server is not enabled (this is an opt-in feature)");
            None
        };

        // Build axum router
        let app = build_router(tera.clone(), mcp_router);

        // Bind and serve
        let listener = tokio::net::TcpListener::bind(APP_CONF.server.inet)
            .await
            .unwrap();

        info!("http server listening on {}", APP_CONF.server.inet);

        axum::serve(listener, app).await.unwrap();
    });
}

fn build_router(tera: Arc<Tera>, mcp_router: Option<Router>) -> Router {
    let assets_path = APP_CONF.assets.path.canonicalize().unwrap();

    // Build authenticated sub-routers
    let reporter_routes = Router::new()
        .route("/{probe_id}/{node_id}", post(routes::reporter_report))
        .route(
            "/{probe_id}/{node_id}/{replica_id}",
            delete(routes::reporter_flush),
        )
        .layer(middleware::from_fn(basic_auth_reporter));

    let manager_routes = Router::new()
        .route("/announcements", get(routes::manager_announcements))
        .route("/announcement", post(routes::manager_announcement_insert))
        .route(
            "/announcement/{announcement_id}",
            delete(routes::manager_announcement_retract),
        )
        .route("/prober/alerts", get(routes::manager_prober_alerts))
        .route(
            "/prober/alerts/ignored",
            get(routes::manager_prober_alerts_ignored_resolve),
        )
        .route(
            "/prober/alerts/ignored",
            put(routes::manager_prober_alerts_ignored_update),
        )
        .layer(middleware::from_fn(basic_auth_manager));

    let mut app = Router::new()
        // Public routes
        .route("/", get(routes::index))
        .route("/robots.txt", get(routes::robots))
        .route("/status/text", get(routes::status_text))
        .route("/status/report", get(routes::status_report))
        .route("/badge/{kind}", get(routes::badge))
        // Authenticated sub-routers
        .nest("/reporter", reporter_routes)
        .nest("/manager", manager_routes)
        // Static assets
        .nest_service("/assets/fonts", ServeDir::new(assets_path.join("fonts")))
        .nest_service("/assets/images", ServeDir::new(assets_path.join("images")))
        .nest_service(
            "/assets/stylesheets",
            ServeDir::new(assets_path.join("stylesheets")),
        )
        .nest_service(
            "/assets/javascripts",
            ServeDir::new(assets_path.join("javascripts")),
        )
        .with_state(tera)
        .layer(NormalizePathLayer::trim_trailing_slash());

    // Nest MCP router if enabled
    if let Some(mcp) = mcp_router {
        app = app.nest("/mcp/probes", mcp);
    }

    app
}

fn build_mcp_router() -> Router {
    use rmcp::transport::streamable_http_server::tower::StreamableHttpServerConfig;

    let config = StreamableHttpServerConfig::default()
        .with_sse_keep_alive(Some(MCP_SSE_KEEPALIVE_SECONDS))
        .with_legacy_session_mode(true);

    let service = StreamableHttpService::new(
        || Ok(mcp::Probes::new()),
        Arc::new(LocalSessionManager::default()),
        config,
    );

    // Convert tower service to axum router
    let tower_service = tower::ServiceBuilder::new().service(service);

    // ponyail: serve tower service as catch-all for MCP endpoints
    Router::new().fallback_service(tower_service)
}

// -- Basic auth middleware --

async fn basic_auth_reporter(request: Request, next: Next) -> Result<Response, StatusCode> {
    basic_auth_check(request, next, &APP_CONF.server.reporter_token).await
}

async fn basic_auth_manager(request: Request, next: Next) -> Result<Response, StatusCode> {
    basic_auth_check(request, next, &APP_CONF.server.manager_token).await
}

async fn basic_auth_check(
    request: Request,
    next: Next,
    expected_token: &str,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Basic "));

    if let Some(encoded) = auth_header {
        // Decode base64 basic auth
        if let Ok(decoded) = base64_decode(encoded) {
            // Format is "username:password" — we only care about password
            if let Some(password) = decoded.split(':').nth(1) {
                if password == expected_token {
                    return Ok(next.run(request).await);
                }
            }
        }
    }

    // Unauthorized — return 403 with WWW-Authenticate challenge
    Ok(Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header(
            header::WWW_AUTHENTICATE,
            "Basic realm=\"Authentication Required\"",
        )
        .body(Body::empty())
        .unwrap())
}

// ponytail: no base64 crate needed, basic auth decode is trivial
fn base64_decode(input: &str) -> Result<String, ()> {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = Vec::new();
    let bytes: Vec<u8> = input.bytes().filter(|&b| b != b'=').collect();

    for chunk in bytes.chunks(4) {
        if chunk.len() < 2 {
            return Err(());
        }

        let mut buf = 0u32;
        for (i, &byte) in chunk.iter().enumerate() {
            let idx = CHARS.iter().position(|&c| c == byte).ok_or(())?;
            buf |= (idx as u32) << (6 * (3 - i));
        }

        output.push((buf >> 16) as u8);
        if chunk.len() > 2 {
            output.push((buf >> 8) as u8);
        }
        if chunk.len() > 3 {
            output.push(buf as u8);
        }
    }

    String::from_utf8(output).map_err(|_| ())
}
