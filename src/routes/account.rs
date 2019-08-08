//! /account

use crate::prelude::*;
use crate::views;

use crate::app_state::AppState;
use crate::auth::Auth;
use crate::handlers::account::create::*;
use crate::handlers::account::login::*;
use crate::handlers::account::update::*;
use actix_web::State;

/// POST /account
pub fn create(
    account: Json<AccountCreate>,
    state: State<AppState>,
) -> FutureResponse<HttpResponse> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct R {
        user_id: i32,
    }

    state
        .pg
        .send(account.0)
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(answer_success!(Created, R { user_id: user.id })),
            Err(err) => Ok(err.error_response()),
        })
        .responder()
}

fn update(state: State<AppState>, update: Json<AccountUpdate>) -> FutureResponse<HttpResponse> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct R {
        user: views::EncodableUserPrivate,
    }

    state
        .pg
        .send(update.0)
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(answer_success!(
                Ok,
                R {
                    user: user.encodable_private()
                }
            )),
            Err(err) => Ok(err.error_response()),
        })
        .responder()
}

/// POST /account/session
pub fn login(
    login_data: Json<SessionCreate>,
    state: State<AppState>,
) -> FutureResponse<HttpResponse> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct R {
        token: String,
        user: views::EncodableUserPrivate,
    }

    state
        .pg
        .send(login_data.0)
        .from_err()
        .and_then(|res| match res {
            Ok(login_info) => Ok(answer_success!(
                Ok,
                R {
                    token: login_info.token,
                    user: login_info.user.encodable_private(),
                }
            )),
            Err(err) => Ok(err.error_response()),
        })
        .responder()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionInfo {
    user: views::EncodableUserPrivate,
}

/// GET /account/session
pub fn get_session(auth: Auth) -> HttpResponse {
    answer_success!(
        Ok,
        SessionInfo {
            user: auth.user.encodable_private(),
        }
    )
}

#[inline]
pub fn scope(scope: Scope<AppState>) -> Scope<AppState> {
    scope
        .resource("/", |r| {
            r.post().with(self::create);
            r.get().with(self::update);
        })
        .resource("/session/", |r| {
            r.post().with(self::login);
            r.get().with(self::get_session)
        })
}
