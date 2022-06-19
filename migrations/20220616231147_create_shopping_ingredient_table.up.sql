CREATE TABLE shopping_ingredient (
    id SERIAL PRIMARY KEY,
    shopping_id integer NOT NULL,
    ingredient_id integer NOT NULL,
    checked boolean NOT NULL,
    CONSTRAINT shopping_ingredient_shopping_id_fkey FOREIGN KEY(shopping_id) REFERENCES shopping(id) ON DELETE CASCADE,
    CONSTRAINT shopping_ingredient_ingredient_id_fkey FOREIGN KEY(ingredient_id) REFERENCES ingredient(id) ON DELETE CASCADE
);
