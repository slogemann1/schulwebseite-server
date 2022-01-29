mod website;

static ROOT_DIRECTORY: &str = "data";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    website::run().await
}

// Path beginnend mit ROOT_DIRECTORY
fn file_path_from_root(path: &str, file: &str) -> String {
    format!("{}/{}/{}", ROOT_DIRECTORY, path, file)
}