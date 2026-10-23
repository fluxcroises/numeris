-- Ordre des trames du master, estimé pendant la capture (filtre idet) :
-- le désentrelacement en a besoin, le master lui-même n'est pas retouché.
ALTER TABLE cassettes ADD COLUMN field_order TEXT
    CHECK (field_order IN ('tff', 'bff', 'progressive'));
