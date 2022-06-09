use entity::entities::{recipe, ingredient};
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
                    .table(IngredientQuantity::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(IngredientQuantity::Id)
                            .integer()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(IngredientQuantity::RecipeId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IngredientQuantity::IngredientId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IngredientQuantity::Quantity)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

            manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("ingredient_quantity_recipe_id_fkey")
                    .from(IngredientQuantity::Table, IngredientQuantity::RecipeId)
                    .to(recipe::Entity, recipe::Column::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

            manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("ingredient_quantity_ingredient_id_fkey")
                    .from(IngredientQuantity::Table, IngredientQuantity::IngredientId)
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
                    .table(IngredientQuantity::Table)
                    .to_owned(),
            )
            .await?;

            manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("ingredient_quantity_recipe_id_fkey")
                    .table(IngredientQuantity::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(IngredientQuantity::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum IngredientQuantity {
    Table,
    Id,
    RecipeId,
    IngredientId,
    Quantity,
}
