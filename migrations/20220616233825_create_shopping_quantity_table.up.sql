CREATE TABLE shopping_quantity (
    id SERIAL PRIMARY KEY,
    ingredient_shopping_id integer NOT NULL,
    recipe_id integer,
    quantity integer NOT NULL,
    CONSTRAINT shopping_quantity_ingredient_shopping_id_fkey FOREIGN KEY(ingredient_shopping_id) REFERENCES ingredient_shopping(id) ON DELETE CASCADE,
    CONSTRAINT shopping_quantity_recipe_id_fkey FOREIGN KEY(recipe_id) REFERENCES recipe(id) ON DELETE CASCADE
);
