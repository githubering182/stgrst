pub mod core;
pub mod routes;

// use actix_cors::Cors;
// use actix_web::{
//     get,
//     http::header::{self, HeaderMap, Range},
//     middleware::Logger,
//     post,
//     web::{Bytes, BytesMut, Data, Header, Path, Payload},
//     App, HttpRequest, HttpResponse, HttpServer, Responder,
// };
// use env_logger::{init_from_env as init_logger_from_env, Env};
// use futures::{
//     io::ReadExact, AsyncReadExt, AsyncWriteExt, FutureExt, Stream, StreamExt, TryStreamExt,
// };
// use mongodb::{
//     bson::{doc, oid::ObjectId, Bson, Document},
//     gridfs::FilesCollectionDocument,
//     options::{ClientOptions, GridFsBucketOptions, GridFsUploadOptions},
//     results::InsertOneResult,
//     Client, Database, GridFsDownloadStream,
// };
// use std::{
//     io::{Error, Read},
//     pin::Pin,
//     str::FromStr,
//     sync::{Arc, RwLock},
//     task::{Context, Poll},
//     u64,
// };
