use crate::database::db::AppState;
use crate::controllers::controller_badge::{ get_all_badges };
use crate::services::service_auth::auth;

use std::sync::Arc;
use axum::{ routing::{ get, post, put, Router }, middleware };

pub fn badge_route(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/badges", get(get_all_badges))
        // .route(
        //     "/api/badge/:id",
        //     get(get_user_by_id).route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        // )
        .with_state(app_state)
}
