# Secure Voting System — AGENTS.md

## Toolchain

- **Rust edition 2024** — requires **nightly** Rust. Verify with `rustup show active-toolchain`.
- No formatter config, no clippy config, no CI/CD. Only `cargo check` / `cargo build` / `cargo run` for verification.

## Setup

1. **Database**: `psql -U <user> -d secure_vottyng -f scripts/schema.sql`
2. **`.env`** must contain `DATABASE_URL=postgres://secure_vottyng_user:kriptografija@localhost/secure_vottyng`
3. `dotenvy::dotenv().ok()` loads it at startup; DB worker panics if connection fails.

## Architecture

- **Entrypoint**: `src/main.rs` — event loop with 3 threads: input (crossterm), tick (50ms), DB worker (Postgres).
- **Scene pattern**: `Scene` enum (`Login`/`Register`/`Dashboard`) dispatches draw/handle/on_enter/on_exit/on_db_response. Scenes return `Action` (None/SwitchScene/Quit) to the App loop.
- **Inter-thread comms**: `mpsc` channels — `Event` enum (Input/Paste/Tick/DbResponse) and `DbRequest`/`DbResponse` enums. Scenes send `DbRequest` to the DB worker and receive `DbResponse` via `on_db_response`.
- **Service layer**: `src/services/` — database operations are organized into domain-specific services (`AuthService`, `UserService`, `ElectionService`, `VoteService`, `CertificateService`). Each service borrows `&mut postgres::Client` and encapsulates SQL queries. The DB worker in `db.rs` is a thin dispatcher that routes `DbRequest` variants to the appropriate service method.
- **Models**: `src/models/` — 1:1 structs for each DB table plus `AccountRegistrationForm` (UI-level enum for registration).

## Directory Structure

```
src/
  main.rs              # Entrypoint, thread spawning
  app.rs               # App struct, event loop, scene dispatch
  event.rs             # Event enum (Input/Paste/Tick/DbResponse)
  db.rs                # DbRequest/DbResponse enums, DB worker thread, dispatch to services
  models/              # Data structs matching DB tables
    acc_reg_form.rs    # AccountRegistrationForm enum (Organizer/User)
    organizer.rs       # Organizer struct
    voter.rs           # Voter struct
    election.rs        # Election struct, ElectionStatus enum
    candidate.rs       # Candidate struct
    vote.rs            # Vote struct
    certificate_authority.rs  # CertificateAuthority struct, CaType enum
    crl_entry.rs       # CrlEntry struct
  services/            # Database access layer
    auth.rs            # AuthService — login, failed attempts, revocation flags
    user.rs            # UserService — registration, lookup, certificate storage
    election.rs        # ElectionService — CRUD, status transitions, results storage
    vote.rs            # VoteService — cast, verify, fetch for counting
    certificate.rs     # CertificateService — CA CRUD, CRL entries, revocation checks
  scene/               # TUI scenes
    login.rs           # LoginScene
    input_cert.rs      # Certificate input scene
    register.rs        # RegisterScene (uses AccountRegistrationForm)
    dashboard.rs       # DashboardScene (placeholder)
  crypto/              # Cryptographic operations
    ca.rs              # CA hierarchy init/load/generate (Root -> Org CA, Voter CA)
```

## Current State

### Done
- TUI skeleton: login form, registration form (Organizer/Voter toggle via Ctrl+T), dashboard with progress bar.
- Service layer with 5 domain services covering all 7 DB tables.
- Models for all DB tables.
- CA hierarchy: 2-level PKI (Root CA -> Organizational CA + Voter CA) using `rcgen`. Auto-generated on first run, persisted to DB, loaded on subsequent runs.
- DB schema: 7 tables (`organizers`, `voters`, `certificate_authorities`, `crl_entries`, `elections`, `candidates`, `votes`).

### Not Done
- **Password hashing** — registration and login store/compare plaintext passwords. Need to add proper hashing (e.g. argon2/bcrypt).
- **User certificate generation** — on registration, system must auto-generate X.509 cert + RSA keypair for the user, signed by the appropriate CA (Org CA for organizers, Voter CA for voters). Certificate must contain user-identifying data. Private key must be encrypted with user's password.
- **Two-step login** — step 1: user provides their certificate PEM, system validates it (time validity, issuer chain, CRL check, user binding). Step 2: username + password. Certificate auto-revoked after 3 consecutive failed logins.
- **Organizer dashboard** — create elections (title, description, start/end time, 2-5 candidates), view list with statuses, trigger vote counting after election closes.
- **Voter dashboard** — view active elections, select and cast vote, verify own vote.
- **Vote encryption** — each vote encrypted with random AES key; AES key wrapped with organizer's RSA public key; HMAC for metadata integrity; voter's digital signature on the vote.
- **Vote counting** — organizer decrypts votes using their private key + AES keys, tallies results, system generates digitally signed results report.
- **Vote verification** — voter can verify their vote was correctly recorded without revealing content.
- **Certificate validation** — full chain validation: time validity, issuer verification, CRL lookup (per-CA), user binding check.
- **CRL management** — separate CRL per CA body, auto-revoke certificates after 3 failed logins.

## DB Schema

`scripts/schema.sql` has 7 tables: `organizers`, `voters`, `certificate_authorities`, `crl_entries`, `elections`, `candidates`, `votes`. Detailed design rationale in `doc/explenation.md`.

## Requirements (from doc/zadatak.pdf)

### Registration
- Two account types: Organizer (org name, identification number, password) and Voter (first name, last name, username, password).
- Auto-generate X.509 certificate + keypair on registration. Certificate data must be linked to user data. Private key encrypted with user password.

### Authentication
- Two-step login: (1) provide digital certificate, validate it; (2) username + password.
- After successful login, show appropriate interface based on account type.
- Auto-revoke certificate after 3 failed login attempts.

### Certificate Validation
- Check time validity.
- Verify issuer (chain to CA).
- Check against CRL (separate CRL per CA body).
- Verify certificate belongs to the given user.

### CA Hierarchy
- 2-level: Root CA signs subordinate CAs only.
- Organizational CA issues certificates to organizers only.
- Voter CA issues certificates to voters only.
- All certificates must contain appropriate extensions for their intended use.

### Organizer Features
- Create elections: title, description, time period (start/end), 2-5 candidates/options.
- View list of all created elections with statuses.
- After election ends: trigger vote counting, download results.

### Voter Features
- View list of active elections.
- Select election and cast vote.
- Vote is automatically encrypted and digitally signed.
- Receive confirmation of successful vote.
- Verify own vote at any time (without revealing content).

### Vote Encryption
- Each vote encrypted with symmetric algorithm (random key per vote).
- Symmetric key encrypted with organizer's public key.
- Vote metadata stored separately, integrity protected by HMAC.

### Vote Counting
- After election time expires, organizer triggers counting.
- System decrypts votes using organizer's private key + symmetric keys.
- Results displayed to organizer.
- Digitally signed results report auto-generated.

### Notes
- Unspecified details may be implemented freely.
- Any programming language and crypto library allowed.
- UI implementation will not be graded.
