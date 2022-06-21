use axum::{
    async_trait, body,
    extract::{FromRequest, RequestParts},
    http::StatusCode,
    response::{IntoResponse, Response},
    BoxError, Json,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use std::collections::HashMap;
use validator::{Validate, ValidationErrors};

pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, B> FromRequest<B> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    B: body::HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = ServerError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req)
            .await
            .map_err(ServerError::AxumJsonRejection)?;
        value.validate().map_err(ServerError::ValidationError)?;
        Ok(ValidatedJson(value))
    }
}

pub enum ServerError {
    ValidationError(validator::ValidationErrors),
    AxumJsonRejection(axum::extract::rejection::JsonRejection),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::ValidationError(e) => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "errors": resolve_error_type(&e) })),
            )
                .into_response(),
            ServerError::AxumJsonRejection(e) => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response(),
        }
    }
}

#[derive(Serialize)]
#[serde(untagged)]
enum ErrorMessages {
    Map(HashMap<String, ErrorMessages>),
    List(Vec<ListError>),
    Vec(Vec<String>),
}

#[derive(Serialize)]
struct ListError {
    pub index: usize,
    pub errors: ErrorMessages,
}

fn resolve_error_type(errors: &ValidationErrors) -> ErrorMessages {
    ErrorMessages::Map(
        errors
            .errors()
            .iter()
            .map(|error| {
                (
                    error.0.to_string(),
                    match error.1 {
                        validator::ValidationErrorsKind::Struct(error_struct) => {
                            resolve_error_type(error_struct)
                        }
                        validator::ValidationErrorsKind::Field(error_field) => ErrorMessages::Vec(
                            error_field
                                .iter()
                                .map(|e| match e.message.clone() {
                                    Some(msg) => msg.to_string(),
                                    None => format!("No error message defined for {}", e.code),
                                })
                                .collect(),
                        ),
                        validator::ValidationErrorsKind::List(error_list) => ErrorMessages::List(
                            error_list
                                .iter()
                                .map(|error| ListError {
                                    index: error.0.to_owned(),
                                    errors: resolve_error_type(error.1),
                                })
                                .collect(),
                        ),
                    },
                )
            })
            .collect(),
    )
}
