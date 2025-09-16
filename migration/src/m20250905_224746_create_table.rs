use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::{
    prelude::{extension::postgres::Type, *},
    schema::*,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        // todo!();

        manager
            .create_table(
                Table::create()
                    .table(Tweet::Table)
                    .if_not_exists()
                    .col(pk_auto(Tweet::Id).not_null())
                    .col(string(Tweet::Content).not_null())
                    .col(string(Tweet::Hash).not_null())
                    .col(
                        timestamp_with_time_zone(Tweet::Postat)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(integer(Tweet::Heart).not_null().default(0))
                    .col(boolean(Tweet::Webvisible).default(true).not_null())
                    .col(boolean(Tweet::Igvisible).default(false).not_null())
                    .col(string_null(Tweet::Igid))
                    .col(integer_null(Tweet::SupervisorId))
                    .col(integer_null(Tweet::NickId))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Admin::Table)
                    .if_not_exists()
                    .col(pk_auto(Admin::Id))
                    .col(string(Admin::Name))
                    .col(string(Admin::Password))
                    .col(string(Admin::TotpSecret))
                    .col(small_integer(Admin::Priority))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Nick::Table)
                    .if_not_exists()
                    .col(pk_auto(Nick::Id))
                    .col(string(Nick::Name))
                    .col(string(Nick::Password))
                    .col(string(Nick::TotpSecret))
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(MediaTypeEnum)
                    .values(MediaType::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Media::Table)
                    .if_not_exists()
                    .col(pk_auto(Media::Id))
                    .col(integer(Media::TweetId))
                    .col(enumeration(
                        Media::MediaType,
                        Alias::new("media_type_enum"),
                        MediaType::iter(),
                    ))
                    .col(string(Media::Path))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .to_tbl(Admin::Table)
                    .to_col(Admin::Id)
                    .from_tbl(Tweet::Table)
                    .from_col(Tweet::SupervisorId)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .to_tbl(Nick::Table)
                    .to_col(Nick::Id)
                    .from_tbl(Tweet::Table)
                    .from_col(Tweet::NickId)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .from_tbl(Media::Table)
                    .from_col(Media::TweetId)
                    .to_tbl(Tweet::Table)
                    .to_col(Tweet::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("tweet_post_at")
                    .table(Tweet::Table)
                    .col(Tweet::Postat)
                    .to_owned(),
            )
            .await?;
        return Ok(());
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        // todo!();
        //
        // manager
        //     .drop_table(Table::drop().table(Tweet::Table).to_owned())
        //     .await
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Tweet {
    Table,
    Id,
    Content,
    Hash,
    Postat,
    Heart,
    Webvisible,
    Igvisible,
    Igid,
    SupervisorId,
    NickId,
}

#[derive(DeriveIden)]
enum Nick {
    Table,
    Id,
    Name,
    Password,
    TotpSecret,
}

#[derive(DeriveIden)]
enum Admin {
    Table,
    Id,
    Name,
    Password,
    TotpSecret,
    Priority,
}
#[derive(DeriveIden)]
struct MediaTypeEnum;
#[derive(Iden, EnumIter)]
pub enum MediaType {
    #[iden = "Pic"]
    Picture,
    #[iden = "Vid"]
    Video,
}

#[derive(DeriveIden)]
enum Media {
    Table,
    Id,
    TweetId,
    MediaType,
    Path,
}
