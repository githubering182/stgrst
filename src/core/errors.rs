use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum DataBaseError {
    #[display(fmt = "DataBase Internal Error")]
    InternalError,
    #[display(fmt = "DataBase Connection Error")]
    ConnectionError,
    #[display(fmt = "File not found Error")]
    NotFoundError,
}

#[derive(Debug, Display, Error)]
pub enum JobError {
    #[display(fmt = "Job Internal Error")]
    InternalError,
    #[display(fmt = "Job Connection Error")]
    ConnectionError,
    #[display(fmt = "Job failed Error")]
    TaskFailed,
}

// impl ResponseError for DataBaseError {
//     fn error_response(&self) -> HttpResponse {
//         HttpResponse::build(self.status_code()).body(self.to_string())
//     }
//     fn status_code(&self) -> StatusCode {
//         match *self {
//             DataBaseError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
//             DataBaseError::ConnectionError => StatusCode::INTERNAL_SERVER_ERROR,
//             DataBaseError::NotFoundError => StatusCode::NOT_FOUND,
//         }
//     }
// }

// impl ResponseError for JobError {
//     fn error_response(&self) -> HttpResponse {
//         HttpResponse::build(self.status_code()).body(self.to_string())
//     }
//     fn status_code(&self) -> StatusCode {
//         match *self {
//             JobError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
//             JobError::ConnectionError => StatusCode::INTERNAL_SERVER_ERROR,
//             JobError::TaskFailed => StatusCode::INTERNAL_SERVER_ERROR,
//         }
//     }
// }
