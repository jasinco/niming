use actix_web::get;
use actix_web::web::Json;

#[derive(utoipa::ToSchema, serde::Serialize)]
struct Post<'a> {
    id: i32,
    content: &'a str,
}

#[utoipa::path(responses((status = OK, body = Post)))]
#[get("/post")]
pub async fn get_post() -> Json<Post<'static>> {
    Json(Post {
        id: 1,
        content: "TESTDOC",
    })
}

