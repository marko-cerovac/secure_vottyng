-- Secure Voting System — Database Schema
-- Run with: psql -U <user> -d secure_vottyng -f scripts/schema.sql

BEGIN;

DROP TABLE IF EXISTS votes          CASCADE;
DROP TABLE IF EXISTS candidates     CASCADE;
DROP TABLE IF EXISTS elections      CASCADE;
DROP TABLE IF EXISTS crl_entries    CASCADE;
DROP TABLE IF EXISTS certificate_authorities CASCADE;
DROP TABLE IF EXISTS voters         CASCADE;
DROP TABLE IF EXISTS organizers     CASCADE;

CREATE TABLE IF NOT EXISTS organizers (
    id                      SERIAL PRIMARY KEY,
    organization            TEXT NOT NULL,
    identification_number   TEXT NOT NULL UNIQUE,
    password_hash           TEXT NOT NULL,
    certificate             TEXT,
    encrypted_private_key   TEXT,
    failed_login_attempts   INT NOT NULL DEFAULT 0,
    certificate_revoked     BOOLEAN NOT NULL DEFAULT false,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS voters (
    id                      SERIAL PRIMARY KEY,
    first_name              VARCHAR(30) NOT NULL,
    last_name               VARCHAR(30) NOT NULL,
    username                VARCHAR(30) NOT NULL UNIQUE,
    password_hash           TEXT NOT NULL,
    certificate             TEXT,
    encrypted_private_key   TEXT,
    failed_login_attempts   INT NOT NULL DEFAULT 0,
    certificate_revoked     BOOLEAN NOT NULL DEFAULT false,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS certificate_authorities (
    id              SERIAL PRIMARY KEY,
    ca_type         VARCHAR(20) NOT NULL CHECK (ca_type IN ('root', 'organizational', 'voter')),
    certificate     TEXT NOT NULL,
    private_key     TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS crl_entries (
    id              SERIAL PRIMARY KEY,
    ca_type         VARCHAR(20) NOT NULL CHECK (ca_type IN ('root', 'organizational', 'voter')),
    serial_number   TEXT NOT NULL,
    revoked_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    reason          TEXT,
    UNIQUE (ca_type, serial_number)
);

CREATE TABLE IF NOT EXISTS elections (
    id              SERIAL PRIMARY KEY,
    organizer_id    INT NOT NULL REFERENCES organizers(id),
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    starts_at       TIMESTAMPTZ NOT NULL,
    ends_at         TIMESTAMPTZ NOT NULL,
    status          VARCHAR(20) NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'active', 'closed', 'counted')),
    results_report    TEXT,
    results_signature TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT valid_period CHECK (ends_at > starts_at)
);

CREATE TABLE IF NOT EXISTS candidates (
    id              SERIAL PRIMARY KEY,
    election_id     INT NOT NULL REFERENCES elections(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    position        INT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS votes (
    id                      SERIAL PRIMARY KEY,
    election_id             INT NOT NULL REFERENCES elections(id),
    voter_id                INT NOT NULL REFERENCES voters(id),
    encrypted_symmetric_key TEXT NOT NULL,
    encrypted_vote          TEXT NOT NULL,
    vote_hmac               TEXT NOT NULL,
    signature               TEXT NOT NULL,
    cast_at                 TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (election_id, voter_id)
);

COMMIT;
