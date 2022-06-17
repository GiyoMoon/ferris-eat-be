CREATE TABLE shopping (
    id SERIAL PRIMARY KEY,
    user_id uuid NOT NULL,
    name character varying NOT NULL,
    CONSTRAINT shopping_user_id_fkey FOREIGN KEY(user_id) REFERENCES "user"(id) ON DELETE CASCADE
);
