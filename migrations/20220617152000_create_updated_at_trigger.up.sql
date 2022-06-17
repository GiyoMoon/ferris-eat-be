CREATE  FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_updated_at
    BEFORE UPDATE
    ON
        "user"
    FOR EACH ROW
EXECUTE PROCEDURE update_updated_at();

CREATE TRIGGER update_updated_at
    BEFORE UPDATE
    ON
        recipe
    FOR EACH ROW
EXECUTE PROCEDURE update_updated_at();
