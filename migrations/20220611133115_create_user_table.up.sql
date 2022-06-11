CREATE TABLE "user" (
    id uuid NOT NULL PRIMARY KEY,
    username character varying NOT NULL UNIQUE,
    alias character varying NOT NULL,
    email character varying NOT NULL UNIQUE,
    password character varying NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
