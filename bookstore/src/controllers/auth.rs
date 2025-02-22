use std::time::SystemTime;

use crate::{
    auth::AuthenticatedUser, db, entities::{prelude::*, user}, AppConfig
};

use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use rocket::{
    config, http::Status, serde::{json::Json, Deserialize, Serialize}, State
};
use sea_orm::*;

use super::{ErrorResponse, Response, SuccessResponse};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ReqSignIn {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ResSignIn {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Claims {
    sub: i32,
    role: String,
    exp: u64
}

#[post("/signin", data = "<req_sign_in>")]
pub async fn sign_in(
    db: &State<DatabaseConnection>,
    config: &State<AppConfig>,
    req_sign_in: Json<ReqSignIn>,
) -> Response<Json<ResSignIn>> {
    let db = db as &DatabaseConnection;
    let config = config as &AppConfig;

    let u = match User::find()
        .filter(user::Column::Email.eq(&req_sign_in.email))
        .one(db)
        .await?
    {
        Some(u) => u,
        None => {
            return Err(ErrorResponse((
                Status::Unauthorized,
                "Invalid Credentials".to_string(),
            )))
        }
    };
    
    if !verify(&req_sign_in.password, &u.password).unwrap() {
        return Err(ErrorResponse((
            Status::Unauthorized,
            "Invalid Credentials".to_string(),
        )));
    }

    let claims = Claims {
        sub: u.id,
        role: "user".to_string(),
        exp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 4 * 60 * 60
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .unwrap();

    Ok(SuccessResponse((Status::Ok, Json(ResSignIn { token }))))
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ReqSignUp {
    email: String,
    password: String,
    first_name: String,
    last_name: String,
}

#[post("/signup", data = "<req_sign_up>")]
pub async fn sign_up(
    db: &State<DatabaseConnection>,
    req_sign_up: Json<ReqSignUp>,
) -> Response<String> {
    let db = db as &DatabaseConnection;

    if User::find()
        .filter(user::Column::Email.eq(&req_sign_up.email))
        .one(db)
        .await?
        .is_some()
    {
        return Err(ErrorResponse((
            Status::UnprocessableEntity,
            "An account already exist with this email address".to_string(),
        )));
    }

    User::insert(user::ActiveModel {
        email: Set(req_sign_up.email.to_owned()),
        password: Set(hash(&req_sign_up.password, DEFAULT_COST).unwrap()),
        first_name: Set(req_sign_up.first_name.clone()),
        last_name: Set(req_sign_up.last_name.clone()),
        ..Default::default()
    })
    .exec(db)
    .await?;

    Ok(SuccessResponse((
        Status::Created,
        "Account created!".to_string(),
    )))
}

#[get("/me")]
pub async fn me(db: &State<DatabaseConnection>, user: AuthenticatedUser) -> Response<String> {
    Ok(SuccessResponse((
        Status::Ok,
        "User ID :".to_string() + user.id.to_string().as_str(),
    )))
}