CREATE TABLE shopping_quantity (
    id SERIAL PRIMARY KEY,
    shopping_ingredient_id integer NOT NULL,
    recipe_id integer,
    quantity integer NOT NULL,
    CONSTRAINT shopping_quantity_shopping_ingredient_id_fkey FOREIGN KEY(shopping_ingredient_id) REFERENCES shopping_ingredient(id) ON DELETE CASCADE,
    CONSTRAINT shopping_quantity_recipe_id_fkey FOREIGN KEY(recipe_id) REFERENCES recipe(id) ON DELETE CASCADE
);
