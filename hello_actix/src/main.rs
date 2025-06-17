use actix_web::{get, App, HttpServer, HttpResponse, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("<!doctype html>
<html>
  <head>
    <meta charset=\"utf-8\">
    <title>Hello</title>
  </head>
  <body>
    <h1>Hello, My Rust World!</h1>
  </body>
</html>")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 127.0.0.1:8080 에서 웹서버 실행
    println!("Server running at http://127.0.0.1:8080/");
    HttpServer::new(|| {
        App::new()
            .service(hello)       // "/" 라우트 등록
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
