CREATE TABLE recipe_quantity (
    id SERIAL PRIMARY KEY,
    recipe_id integer NOT NULL,
    ingredient_id integer NOT NULL,
    quantity integer NOT NULL,
    CONSTRAINT recipe_quantity_recipe_id_fkey FOREIGN KEY(recipe_id) REFERENCES recipe(id) ON DELETE CASCADE,
    CONSTRAINT recipe_quantity_ingredient_id_fkey FOREIGN KEY(ingredient_id) REFERENCES ingredient(id) ON DELETE CASCADE
);
