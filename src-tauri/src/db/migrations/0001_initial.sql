-- Schéma initial du catalogue.
-- Les chemins sont relatifs à la racine de la bibliothèque. Les dates de
-- tournage acceptent l'imprécision : « 1998 », « 1998-12 » ou « 1998-12-24 ».

CREATE TABLE places (
    id        INTEGER PRIMARY KEY,
    name      TEXT NOT NULL UNIQUE,
    latitude  REAL,
    longitude REAL
);

CREATE TABLE people (
    id   INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE cassettes (
    id              INTEGER PRIMARY KEY,
    -- Identifiant stable, repris dans cassette.json.
    uuid            TEXT NOT NULL UNIQUE,
    -- Dossier de la cassette dans la bibliothèque (« 1998-noel-chez-mamie »).
    folder          TEXT NOT NULL UNIQUE,
    title           TEXT NOT NULL,
    format          TEXT NOT NULL CHECK (format IN
                      ('vhs', 'vhs_c', 'svhs', 'video8', 'hi8', 'digital8', 'minidv', 'file')),
    tv_standard     TEXT CHECK (tv_standard IN ('pal', 'secam', 'ntsc')),
    recorded_from   TEXT,
    recorded_to     TEXT,
    place_id        INTEGER REFERENCES places(id) ON DELETE SET NULL,
    notes           TEXT NOT NULL DEFAULT '',
    label_photo     TEXT,
    master_path     TEXT,
    master_blake3   TEXT,
    proxy_path      TEXT,
    duration_ms     INTEGER,
    status          TEXT NOT NULL DEFAULT 'to_capture' CHECK (status IN
                      ('to_capture', 'captured', 'ready', 'exported')),
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX cassettes_status ON cassettes(status);
CREATE INDEX cassettes_recorded ON cassettes(recorded_from);

CREATE TABLE chapters (
    id          INTEGER PRIMARY KEY,
    cassette_id INTEGER NOT NULL REFERENCES cassettes(id) ON DELETE CASCADE,
    position    INTEGER NOT NULL,
    start_ms    INTEGER NOT NULL CHECK (start_ms >= 0),
    end_ms      INTEGER NOT NULL CHECK (end_ms > start_ms),
    title       TEXT NOT NULL DEFAULT '',
    recorded_at TEXT,
    place_id    INTEGER REFERENCES places(id) ON DELETE SET NULL,
    thumbnail   TEXT,
    UNIQUE (cassette_id, position)
);

CREATE TABLE cassette_people (
    cassette_id INTEGER NOT NULL REFERENCES cassettes(id) ON DELETE CASCADE,
    person_id   INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    PRIMARY KEY (cassette_id, person_id)
);

CREATE TABLE chapter_people (
    chapter_id INTEGER NOT NULL REFERENCES chapters(id) ON DELETE CASCADE,
    person_id  INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    PRIMARY KEY (chapter_id, person_id)
);

-- Traitements longs (proxy, encodage, analyse, IA), repris après fermeture.
CREATE TABLE tasks (
    id          INTEGER PRIMARY KEY,
    kind        TEXT NOT NULL,
    cassette_id INTEGER REFERENCES cassettes(id) ON DELETE CASCADE,
    -- Paramètres propres au type de tâche, en JSON.
    payload     TEXT NOT NULL DEFAULT '{}',
    status      TEXT NOT NULL DEFAULT 'pending' CHECK (status IN
                  ('pending', 'running', 'done', 'failed', 'cancelled')),
    progress    REAL NOT NULL DEFAULT 0 CHECK (progress BETWEEN 0 AND 1),
    error       TEXT,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX tasks_status ON tasks(status);
