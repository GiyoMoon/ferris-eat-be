use entity::entities::unit::{ActiveModel, Column, Entity};
use sea_orm_migration::sea_orm::Set;
use sea_orm_migration::{prelude::*, sea_orm::ActiveModelTrait};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220605_192602_create_unit_table"
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
                    .col(ColumnDef::new(Column::Name).string().not_null())
                    .to_owned(),
            )
            .await;

        match create_table {
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        let connection = manager.get_connection();
        let units = vec!["g", "kg", "ml", "l", "none"];
        for unit in units {
            let insert_unit = ActiveModel {
                name: Set(unit.to_string()),
                ..Default::default()
            }
            .insert(connection)
            .await;

            match insert_unit {
                Ok(_) => (),
                Err(e) => return Err(e),
            };
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Entity).to_owned())
            .await
    }
}
