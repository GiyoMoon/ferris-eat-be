use entity::entities::ingredient::{Column, Entity};
use entity::entities::unit;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220605_192716_create_ingredient_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let create_table = manager
            .create_table(
                Table::create()
                    .table(Entity)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Column::Id)
                            .integer()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Column::UnitId).integer().not_null())
                    .col(ColumnDef::new(Column::Name).string().not_null())
                    .to_owned(),
            )
            .await;

        match create_table {
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("ingredient_unit_id_fkey")
                    .from(Entity, Column::UnitId)
                    .to(unit::Entity, unit::Column::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let drop_foreign_key = manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("ingredient_unit_id_fkey")
                    .table(Entity)
                    .to_owned(),
            )
            .await;

        match drop_foreign_key {
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        manager
            .drop_table(Table::drop().table(Entity).to_owned())
            .await
    }
}
