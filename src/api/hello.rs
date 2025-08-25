use crate::db::post::Model as PostModel;
use actix_web::get;
use actix_web::web::Json;

#[utoipa::path(responses((status = OK, body = PostModel)))]
#[get("/post")]
pub async fn get_post() -> Json<PostModel> {
    Json(PostModel {
        id: 1,
        content: "test".to_string(),
        hash: "t2".to_string(),
        heart: 2,
        post_at: chrono::Utc::now().into(),
        igid: None,
        nick_id: None,
        supervisor_id: None,
        web_visible: false,
        ig_visible: false,
    })
}
