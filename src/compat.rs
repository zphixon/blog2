use std::error::Error;

use crate::model;
use axum::{
    extract::{
        rejection::{FormRejection, JsonRejection},
        FromRequest,
    },
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(model::ApiError))]
pub struct MyJson<T>(pub T);

impl<T: Serialize> IntoResponse for MyJson<T> {
    fn into_response(self) -> axum::response::Response {
        let Self(value) = self;
        axum::Json(value).into_response()
    }
}

impl From<JsonRejection> for model::ApiError {
    fn from(value: JsonRejection) -> Self {
        model::ApiError::InvalidJson(format!("{}", value))
    }
}

impl IntoResponse for model::ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            model::ApiError::Markdown(markdown_error) => todo!(),

            model::ApiError::Content(content_error) => todo!(),

            model::ApiError::Database(db_error) => todo!(),

            model::ApiError::Publish(publish_response) => {
                match serde_json::to_value(publish_response) {
                    Ok(ok) => (StatusCode::OK, axum::Json(ok)),
                    Err(err) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(serde_json::json!({"err": err.to_string()})),
                    ),
                }
            }

            model::ApiError::InvalidJson(err) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                axum::Json(serde_json::json!({"err": err})),
            ),

            model::ApiError::Sqlx(_) => todo!(),
        }
        .into_response()
    }
}

#[derive(FromRequest)]
#[from_request(via(axum::Form), rejection(model::ApiError))]
pub struct MyForm<T>(pub T);

impl<T: Serialize> IntoResponse for MyForm<T> {
    fn into_response(self) -> axum::response::Response {
        let Self(value) = self;
        axum::Json(value).into_response()
    }
}

impl From<FormRejection> for model::ApiError {
    fn from(value: FormRejection) -> Self {
        let mut s = format!("{}", value);

        let mut source_ = value.source();
        while let Some(source) = source_ {
            s.push_str(&format!(": {}", source));
            source_ = source.source();
        }

        model::ApiError::InvalidJson(s)
    }
}
