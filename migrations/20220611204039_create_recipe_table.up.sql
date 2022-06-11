CREATE TABLE recipe (
    id SERIAL PRIMARY KEY,
    user_id uuid NOT NULL,
    name character varying NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT recipe_user_id_fkey FOREIGN KEY(user_id) REFERENCES "user"(id) ON DELETE CASCADE
);
