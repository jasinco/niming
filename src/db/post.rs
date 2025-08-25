pub use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, serde::Serialize, utoipa::ToSchema)]
#[sea_orm(table_name = "post")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: u32,
    pub content: String,
    pub hash: String,
    pub post_at: chrono::DateTime<chrono::FixedOffset>,
    pub heart: u32,
    pub web_visible: bool,
    pub ig_visible: bool,
    pub igid: Option<String>,
    pub supervisor_id: Option<i32>,
    pub nick_id: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::admin::Entity",
        from = "Column::SupervisorId",
        to = "super::admin::Column::Id"
    )]
    Supervisor,
    #[sea_orm(
        belongs_to = "super::nick::Entity",
        from = "Column::NickId",
        to = "super::nick::Column::Id"
    )]
    Nick,
}
impl Related<super::admin::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Supervisor.def()
    }
}
impl Related<super::nick::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Supervisor.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
