use super::RetrieveQuery;
use crate::{core::DataBaseError, services::BucketService};
use actix_web::{
    get,
    http::header::{ContentDisposition, ContentRange, ContentRangeSpec, ACCEPT_RANGES},
    post,
    web::{Data, Path, Payload, Query},
    HttpRequest, HttpResponse, Responder, ResponseError, Result,
};
use mongodb::Client;

#[post("/file/{bucket}/")]
pub async fn upload(
    path: Path<String>,
    database: Data<Client>,
    payload: Payload,
) -> Result<impl Responder> {
    let bucket_name = path.into_inner();

    let bucket = BucketService::new(database, bucket_name);
    let id = bucket.upload(payload).await?;

    Ok(HttpResponse::Created().body(id))
}

#[get("/file/{bucket}/{file_id}/")]
pub async fn retrieve(
    request: HttpRequest,
    database: Data<Client>,
    path: Path<(String, String)>,
    archive_query: Query<RetrieveQuery>,
) -> Result<impl Responder, impl ResponseError> {
    let (bucket_name, file_id) = path.into_inner();

    let bucket = BucketService::new(database, bucket_name);
    let stream = bucket.retrieve(request.headers(), file_id).await?;

    let mut response = HttpResponse::Ok();
    match archive_query.archive {
        Some(archive) if archive => {
            response.append_header(ContentDisposition::attachment(stream.file_name.clone()));
        }
        _ => {
            response.append_header(ContentRange(ContentRangeSpec::Bytes {
                range: Some((stream.range.start, stream.range.end)),
                instance_length: Some(stream.range.read_length),
            }));
        }
    }

    Ok::<HttpResponse, DataBaseError>(response.streaming(stream))
}
