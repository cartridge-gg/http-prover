use axum::response::IntoResponse;

use crate::temp_dir_middleware::TempDirHandle;

pub async fn root(TempDirHandle(path): TempDirHandle) -> impl IntoResponse {
    "Hello, World from Cairo1!"
}
