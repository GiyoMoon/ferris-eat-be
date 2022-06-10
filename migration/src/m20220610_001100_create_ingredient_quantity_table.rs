use entity::entities::ingredient_quantity::{Column, Entity};
use entity::entities::{ingredient, recipe};
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220610_001100_create_ingredient_quantity_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
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
                    .col(ColumnDef::new(Column::RecipeId).integer().not_null())
                    .col(ColumnDef::new(Column::IngredientId).integer().not_null())
                    .col(ColumnDef::new(Column::Quantity).integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("ingredient_quantity_recipe_id_fkey")
                    .from(Entity, Column::RecipeId)
                    .to(recipe::Entity, recipe::Column::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("ingredient_quantity_ingredient_id_fkey")
                    .from(Entity, Column::IngredientId)
                    .to(ingredient::Entity, ingredient::Column::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("ingredient_quantity_ingredient_id_fkey")
                    .table(Entity)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("ingredient_quantity_recipe_id_fkey")
                    .table(Entity)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Entity).to_owned())
            .await
    }
}
