use crate::prelude::*;

use crate::app_state::AppState;
use crate::auth::AuthOptional;

use crate::models::{Card, User};
use crate::views::{EncodableUserPrivate, EncodableUserPublic};

type FutRes = FutureResponse<HttpResponse>;

#[derive(Deserialize)]
pub struct UserPath {
    user_id: u32,
}

#[derive(Fail, Debug)]
enum GetUserInfoError {
    #[fail(display = "user_not_found")]
    NotFound,
}

pub fn info(auth: AuthOptional, path: Path<UserPath>, state: State<AppState>) -> FutRes {
    use crate::handlers::users::get_user::*;

    #[derive(Serialize)]
    struct R {
        user: UserView,
    }

    #[derive(Serialize)]
    #[serde(untagged)]
    enum UserView {
        Public(EncodableUserPublic),
        Private(EncodableUserPrivate),
    }

    impl UserView {
        #[inline]
        fn answer(auth: AuthOptional, user: User) -> Self {
            if auth
                .user
                .map(|auth_user| auth_user.id == user.id)
                .unwrap_or(false)
            {
                UserView::Private(user.encodable_private())
            } else {
                UserView::Public(user.encodable_public())
            }
        }
    }

    state
        .pg
        .send(GetUser {
            user_id: path.user_id as i32,
        })
        .from_err()
        .and_then(|res| match res {
            Some(user) => Ok(answer_success!(
                Ok,
                R {
                    user: UserView::answer(auth, user),
                }
            )),
            None => Ok(answer_error!(
                NotFound,
                GetUserInfoError::NotFound.to_string()
            )),
        })
        .responder()
}

/// GET /users/{user_id}/cards/useful/
pub fn useful(_auth: AuthOptional, path: Path<UserPath>, state: State<AppState>) -> FutRes {
    use crate::handlers::users::useful_cards::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct R {
        cards: Vec<Card>,
    }

    state
        .pg
        .send(GetUsefulCardsForUser {
            user_id: path.user_id as i32,
        })
        .from_err()
        .and_then(|res| match res {
            Some(cards) => Ok(answer_success!(Ok, R { cards })),
            None => Ok(answer_success!(Ok, R { cards: vec![] })),
        })
        .responder()
}

/// GET /users/{user_id}/cards/authors/
/// Get cards by user
pub fn authors(_auth: AuthOptional, path: Path<UserPath>, state: State<AppState>) -> FutRes {
    use crate::handlers::users::cards_by_author::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct R {
        cards: Vec<Card>,
    }

    state
        .pg
        .send(GetCardsByAuthor {
            author_id: path.user_id as i32,
        })
        .from_err()
        .and_then(|res| match res {
            Some(cards) => Ok(answer_success!(Ok, R { cards })),
            None => Ok(answer_success!(Ok, R { cards: vec![] })),
        })
        .responder()
}

#[inline]
pub fn scope(scope: Scope<AppState>) -> Scope<AppState> {
    scope
        .resource("/{user_id}/", |r| {
            r.get().with(self::info);
        })
        .resource("/{user_id}/cards/useful/", |r| {
            r.get().with(self::useful);
        })
        .resource("/{user_id}/cards/authors/", |r| r.get().with(self::authors))
}
