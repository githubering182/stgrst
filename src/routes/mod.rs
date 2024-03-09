use serde::Deserialize;

mod storage;
mod task;

pub use storage::{retrieve, upload};
pub use task::produce;

#[derive(Deserialize, Debug)]
struct RetrieveQuery {
    archive: Option<bool>,
}
