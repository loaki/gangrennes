use actix_files::{ Files, NamedFile };
use actix_web::{ get, App, HttpServer, Result };

#[get("/")]
async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("./static/index.html")?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(index).service(Files::new("/", "./static").prefer_utf8(true))
    })
        .bind(("0.0.0.0", 8080))?
        .run().await
}
