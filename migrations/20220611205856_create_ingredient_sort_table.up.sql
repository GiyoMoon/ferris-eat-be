CREATE TABLE ingredient_sort (
    id SERIAL PRIMARY KEY,
    ingredient_id integer NOT NULL,
    order_id integer NOT NULL,
    order_after integer NOT NULL,
    CONSTRAINT ingredient_sort_ingredient_id_fkey FOREIGN KEY(ingredient_id) REFERENCES ingredient(id) ON DELETE CASCADE
);
