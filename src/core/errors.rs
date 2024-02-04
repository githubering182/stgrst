use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum DataBaseError {
    #[display(fmt = "DataBase Connection Error")]
    InternalError,
    #[display(fmt = "DataBase Connection Error")]
    ConnectionError,
    #[display(fmt = "File not found Error")]
    NotFoundError,
}

impl ResponseError for DataBaseError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            DataBaseError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            DataBaseError::ConnectionError => StatusCode::INTERNAL_SERVER_ERROR,
            DataBaseError::NotFoundError => StatusCode::NOT_FOUND,
        }
    }
}
