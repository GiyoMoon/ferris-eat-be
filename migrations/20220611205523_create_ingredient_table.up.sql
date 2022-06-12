CREATE TABLE ingredient (
    id SERIAL PRIMARY KEY,
    user_id uuid NOT NULL,
    unit_id integer NOT NULL,
    name character varying NOT NULL,
    sort integer NOT NULL,
    CONSTRAINT ingredient_user_id_fkey FOREIGN KEY(user_id) REFERENCES "user"(id) ON DELETE CASCADE,
    CONSTRAINT ingredient_unit_id_fkey FOREIGN KEY(unit_id) REFERENCES unit(id) ON DELETE CASCADE
);
