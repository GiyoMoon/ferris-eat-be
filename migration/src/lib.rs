pub use sea_orm_migration::prelude::*;

mod m20220603_220951_create_user_table;
mod m20220605_162820_create_recipe_table;
mod m20220605_192602_create_unit_table;
mod m20220605_192716_create_ingredient_table;
mod m20220610_001100_create_ingredient_quantity_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220603_220951_create_user_table::Migration),
            Box::new(m20220605_162820_create_recipe_table::Migration),
            Box::new(m20220605_192602_create_unit_table::Migration),
            Box::new(m20220605_192716_create_ingredient_table::Migration),
            Box::new(m20220610_001100_create_ingredient_quantity_table::Migration),
        ]
    }
}
