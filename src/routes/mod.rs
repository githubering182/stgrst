mod storage;
mod task;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct RetrieveQuery {
    archive: Option<bool>,
}

pub use storage::{retrieve, upload};
pub use task::produce;
