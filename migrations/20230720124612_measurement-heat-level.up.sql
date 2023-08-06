ALTER TABLE measurement ADD COLUMN heat_level FLOAT;

UPDATE measurement SET heat_level =
CASE heat
    WHEN TRUE THEN 1.0
    WHEN FALSE THEN 0.0
END;

ALTER TABLE measurement DROP COLUMN heat;
