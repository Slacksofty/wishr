#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Extension;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use wishr::app::{shell, App};
    use wishr::server::db;

    simple_logger::init_with_level(log::Level::Info).expect("failed to init logger");

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let pool = db::init_db().await.expect("failed to init database");
    log::info!("database ready");

    let app = axum::Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(Extension(pool))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    log::info!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
