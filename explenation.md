# Database Schema Explanation

## Overview

The schema has 7 tables covering the full lifecycle: user management,
certificate infrastructure, election setup, vote casting, and audit.
Every design choice is driven by the zadatak requirements. Below is a
detailed walkthrough.

---

## 1. organizers

Stores accounts for election creators.

**`id`** — SERIAL PRIMARY KEY. Internal identifier used as a foreign key
target from `elections`. We use a synthetic integer PK instead of a natural
key (like `identification_number`) because natural keys can change and are
usually larger to index.

**`organization`** — TEXT NOT NULL. The name of the organizer's organization.
Required by the zadatak: *"Za organizatore se unosi naziv organizacije"*.

**`identification_number`** — TEXT NOT NULL UNIQUE. A business ID number
required by the zadatak: *"identifikacioni broj"*. Why does this exist?
The zadatak gives organizers an "identifikacioni broj" but does not give
them a "username". So this field doubles as their login handle during the
second step of the two-step login. The UNIQUE constraint prevents two
organizers from registering with the same ID.

**`password_hash`** — TEXT NOT NULL. Argon2id hash of the password. We
never store the plaintext password — only the hash is persisted. On login,
the user's password input is hashed and compared against this value.

**`certificate`** — TEXT (nullable). The PEM-encoded X.509 digital
certificate generated automatically during registration. Why PEM? It's a
standard text-based format that's easy to store in a TEXT column and
parse with any crypto library. Null until registration completes — this
lets us treat registration as a multi-step transaction if needed.

**`encrypted_private_key`** — TEXT (nullable). The user's RSA (or ECDSA)
private key, encrypted at rest. How does this work? When the user registers
and provides a password, we:
1. Generate a key pair (public + private).
2. Derive an encryption key from the user's password using a KDF (e.g.,
   Argon2id or PBKDF2).
3. Encrypt the private key with that derived key using AES-GCM.
4. Store the resulting ciphertext (PEM-encoded) in this column.

On login, the user provides their password again, we re-derive the same
encryption key, and decrypt the private key for use during the session.
This means the private key is never stored in plaintext, and an attacker
who dumps the database cannot recover private keys without also knowing
each user's password.

**`failed_login_attempts`** — INT NOT NULL DEFAULT 0. Counter that
increments on every failed login (wrong password or failed cert validation).
Why track this? The zadatak says: *"Sertifikati se automatski povlače u
slučaju tri neuspješne prijave"* — certificates are auto-revoked after 3
failed login attempts. When this counter reaches 3, the application sets
`certificate_revoked = true` and adds a CRL entry. The counter is reset
on successful login.

**`certificate_revoked`** — BOOLEAN NOT NULL DEFAULT false. A cached flag
indicating whether the user's certificate has been revoked. Why not just
check the CRL table? Performance — checking a boolean is instant, while
querying a CRL table and then validating would be slower on every login.
This flag is kept in sync with the CRL table; both are updated together
on revocation.

**`created_at`** — TIMESTAMPTZ NOT NULL DEFAULT now(). Audit trail.
Stored as TIMESTAMPTZ (timezone-aware) so that timestamps are unambiguous
regardless of the server's timezone setting.

---

## 2. voters

Stores accounts for people who cast votes.

**`id`** — SERIAL PRIMARY KEY. Same reasoning as organizers.

**`first_name`** / **`last_name`** — VARCHAR(30) NOT NULL. Required by the
zadatak: *"Za glasače se unosi ime, prezime"*. Capped at 30 characters to
prevent abuse and keep input sizes reasonable for the TUI.

**`username`** — VARCHAR(30) NOT NULL UNIQUE. The voter's login handle for
the second step of the two-step login. Unlike organizers (who use their
`identification_number`), voters get a dedicated username field as specified
in the zadatak: *"korisničko ime"*.

**`password_hash`** — Same as organizers.

**`certificate`** — Nullable, same mechanics as organizers, but issued by
the Voter CA instead of the Organizational CA.

**`encrypted_private_key`** — Same encryption scheme as organizers.

**`failed_login_attempts`** / **`certificate_revoked`** / **`created_at`** —
Identical semantics to the organizers table.

**Why separate tables instead of a single `users` table with a `type`
column?** Two reasons:
1. **Different fields.** Organizers have `organization` + `identification_number`;
   voters have `first_name` + `last_name` + `username`. A single-table approach
   would require nullable columns for type-specific fields, which is messy and
   loses the ability to use NOT NULL constraints.
2. **Different CA issuers.** Organizer certs come from the Organizational CA,
   voter certs from the Voter CA. Separate tables make it trivial to know which
   CA to use during registration without an extra type check.
3. **Different logic.** The dashboard, elections, and voting workflows diverge
   per type. Separate tables let foreign keys (like `elections.organizer_id`)
   be strongly typed.

---

## 3. certificate_authorities

Stores the three CA identities for the 2-level hierarchy.

**`id`** — SERIAL PRIMARY KEY.

**`ca_type`** — VARCHAR(20) NOT NULL with `CHECK (ca_type IN ('root',
'organizational', 'voter'))`. Identifies which CA this row represents.
The CHECK constraint acts as a poor man's enum — PostgreSQL rejects any
value outside the allowed set at the database level, providing defense in
depth even if the application layer has a bug.

**`certificate`** — TEXT NOT NULL. The CA's own X.509 certificate in PEM
format. For the root CA, this is self-signed. For the Organizational and
Voter CAs, this is signed by the root CA. Stored so the application can
present it during certificate validation (chain building).

**`private_key`** — TEXT NOT NULL. The CA's private key. Why do we need
this? Because the application must issue new certificates dynamically at
registration time. Each registration triggers:
1. Generate a user key pair.
2. Have the appropriate CA sign a new certificate using the CA's private key.
3. Store the result in the user's `certificate` column.

Without storing the CA private keys, the application would have to request
an external CA to sign every certificate, which defeats the purpose of an
integrated system.

**Why three rows in one table instead of three separate tables?** The three
CAs share the exact same structure (`ca_type`, `certificate`, `private_key`).
A single table with a CHECK constraint avoids repeating the same DDL three
times and makes it trivial to add operations that iterate over all CAs
(e.g., "rebuild all CRLs").

---

## 4. crl_entries

Certificate Revocation List entries, one row per revoked certificate.

**`id`** — SERIAL PRIMARY KEY.

**`ca_type`** — Same CHECK constraint as `certificate_authorities`. Indicates
which CA issued the revoked certificate. This is how we maintain "separate
CRLs per CA" as required by the zadatak: *"provjeru u odnosu na CRL listu
(posebna za svako CA tijelo)"*. When validating a certificate, the
application queries only the rows matching the issuing CA's type.

**`serial_number`** — TEXT NOT NULL. The serial number of the revoked
certificate. Serial numbers are unique within each CA's namespace. How is
this used? During certificate validation, the application extracts the
serial number from the user's certificate and checks whether it appears
in the CRL entries for the relevant CA.

**`revoked_at`** — TIMESTAMPTZ NOT NULL DEFAULT now(). When the revocation
happened. Useful for audit and for displaying revocation time.

**`reason`** — TEXT (nullable). Optional free-text reason, e.g., "3 failed
login attempts" or "manually revoked by admin".

**How does revocation work end-to-end?**
1. User fails login → `failed_login_attempts` incremented.
2. If `failed_login_attempts >= 3`:
   a. `certificate_revoked` set to `true` on the user row.
   b. A new row inserted into `crl_entries` with the user's cert serial
      number and the issuing CA's type.
   c. Subsequent login attempts check `certificate_revoked` and the CRL
      entry — both must indicate the cert is valid.

---

## 5. elections

Created by organizers. Each row is one voting event.

**`id`** — SERIAL PRIMARY KEY.

**`organizer_id`** — INT NOT NULL REFERENCES organizers(id). Links the
election to the organizer who created it. Foreign key ensures referential
integrity — you cannot have an election without a valid organizer.

**`title`** — TEXT NOT NULL. The election name displayed to voters.
Required by the zadatak: *"navodi se naslov"*.

**`description`** — TEXT NOT NULL DEFAULT ''. Optional description field.
Required by the zadatak: *"i opis"*.

**`starts_at`** / **`ends_at`** — TIMESTAMPTZ NOT NULL. The voting period.
The CONSTRAINT `valid_period CHECK (ends_at > starts_at)` prevents illogical
time ranges (e.g., ending before starting) at the database level.

**`status`** — VARCHAR(20) NOT NULL with CHECK constraint limiting to
`'pending'`, `'active'`, `'closed'`, `'counted'`. Represents the election
lifecycle:
- `pending`: Created but not yet open for voting.
- `active`: Voting is open (current time is between `starts_at` and
  `ends_at`). Visible to voters.
- `closed`: Voting period has ended, waiting for the organizer to start
  counting. Voters can no longer vote but can still verify their vote.
- `counted`: Organizer has triggered counting, results are available.

**Why use a status column instead of computing it from `starts_at` /
`ends_at`?** Computing status from timestamps is fragile — what if an
organizer wants to close voting early? What about the `counted` state,
which has no timestamp counterpart? A dedicated status column gives full
control over the lifecycle and makes queries like "find all active
elections" trivially fast (indexed VARCHAR lookup vs. range comparison
on two timestamp columns).

**`created_at`** — Audit timestamp.

---

## 6. candidates

Election options (2–5 per election as required).

**`id`** — SERIAL PRIMARY KEY.

**`election_id`** — INT NOT NULL REFERENCES elections(id) ON DELETE CASCADE.
Links to the parent election. CASCADE means that deleting an election
automatically removes its candidates — otherwise you'd get orphaned rows
and a foreign key violation.

**`name`** — TEXT NOT NULL. The candidate name or option label displayed
to voters.

**`position`** — INT NOT NULL DEFAULT 0. Display order. When rendering the
voting form, the application sorts candidates by this column. Without it,
the order would be unpredictable (database rows have no inherent order).

---

## 7. votes

The core table — stores every cast vote with its cryptographic protections.

**`id`** — SERIAL PRIMARY KEY.

**`election_id`** — FK → elections(id). Which election this vote belongs to.

**`voter_id`** — FK → voters(id). Who cast this vote.

**`encrypted_symmetric_key`** — TEXT NOT NULL. Here's how the encryption
works per the zadatak:
1. For each vote, the application generates a random AES-256 key
   (symmetric).
2. The vote content (which candidate was chosen) is encrypted with that
   symmetric key → `encrypted_vote`.
3. The symmetric key itself is encrypted with the *organizer's* RSA public
   key → this column.
Why this two-layer approach? This is hybrid encryption. The symmetric cipher
handles the bulk data (fast), while the asymmetric cipher protects the
symmetric key (solves key distribution). The organizer can decrypt all
votes because only their private key can unwrap the symmetric keys.
Why a random key per vote? If the same key were reused, a compromised key
would leak all votes. Random per-vote keys limit exposure.

**`encrypted_vote`** — TEXT NOT NULL. The actual vote content (e.g.,
"candidate_id=3"), encrypted with the per-vote symmetric key. The
application never sees the plaintext after encryption. Stored as
base64-encoded ciphertext.

**`vote_hmac`** — TEXT NOT NULL. HMAC-SHA256 computed over the encrypted
vote + encrypted symmetric key. Why HMAC? It provides integrity — if an
attacker tampers with the ciphertext, the HMAC won't match and the vote
is rejected during counting. The HMAC key is derived from the same per-vote
symmetric key or from a separate shared secret. Per the zadatak: *"njihov
integritet se štiti pomoću HMAC algoritma"*.

**`signature`** — TEXT NOT NULL. The voter's digital signature over the
entire payload (encrypted vote + HMAC). Why a signature? For non-repudiation
— the voter cannot later deny having cast a vote. Also used for the "verify
my vote" feature: the voter can check that their signature is present
without revealing the vote content. The signature is created with the
voter's private key (which they proved access to during login).

**`cast_at`** — TIMESTAMPTZ NOT NULL DEFAULT now(). When the vote was
recorded.

**`UNIQUE(election_id, voter_id)`** — Enforces one vote per voter per
election at the database level. Without this, a voter could vote multiple
times. The application also checks this before inserting, but the UNIQUE
constraint provides a last line of defense.

**How does vote verification work?**
The voter can query their vote row by `election_id` + `voter_id`. They
receive back the `encrypted_vote`, `vote_hmac`, and `signature`. They
can verify:
1. The `signature` is valid using their own public key (proves their vote
   is recorded).
2. The `vote_hmac` matches (proves integrity).
But they *cannot* decrypt the vote without the organizer's private key,
so the content remains secret — exactly as the zadatak requires.

---

## Key Design Decisions Summary

1. **Separate user tables** — Different fields, different CA issuers,
   different business logic. Avoids nullable columns and keeps foreign
   keys semantically clear.

2. **`identification_number` as organizer login** — No username field
   specified for organizers; the ID number fills that role.

3. **Private key encrypted at rest with user password** — Stored in the
   same row, safe because only the user's password can decrypt it.

4. **3-strikes revocation** — `failed_login_attempts` counters + CRL
   entries + `certificate_revoked` flags work together for the auto-revoke
   requirement.

5. **Hybrid encryption for votes** — AES per-vote (fast) + RSA-wrapped
   symmetric key (key distribution) + HMAC (integrity) + signature
   (non-repudiation). All four cryptographic primitives stored in one
   row for auditability.

6. **Election status lifecycle** — Explicit status column instead of
   computed-from-timestamps, giving control over early closing and the
   counted state.
