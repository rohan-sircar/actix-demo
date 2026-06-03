# Email Service: Mailpit Dev + Real Provider Prod Proposal

## Goal

Add email functionality (password reset, email verification) to actix-demo with a strategy that uses Mailpit locally for zero-friction development and a real SMTP provider in production.

---

## Architecture

```
┌─────────────────────────────────────────────┐
│              Mailer Trait                    │
│  send_reset_email(to, user_name, token)      │
│  send_verification_email(to, user_name, token)│
└──────────────┬──────────────────┬───────────┘
               │                  │
    ┌──────────▼──────┐  ┌───────▼──────────┐
    │  Dev Config      │  │  Prod Config      │
    │  (Mailpit)       │  │  (Brevo/Mailgun)  │
    │  localhost:1025  │  │  smtp.brevo.com   │
    │  no TLS          │  │  StartTLS         │
    └──────────────────┘  └──────────────────┘
```

Same `SmtpSender` implementation, different config. The app depends on the `Mailer` trait, not a concrete sender. Concrete implementation is injected at startup based on env config.

---

## Development: Mailpit

### What it is

Mailpit is a zero-dependency email testing tool for developers. It runs as a Docker container, accepts SMTP traffic, and provides a web UI to inspect sent emails. No signup, no verification, no limits.

### Docker Compose addition

```yaml
services:
  mailpit:
    image: axllent/mailpit:latest
    container_name: actix-demo-mailpit
    ports:
      - "1025:1025"   # SMTP
      - "8025:8025"   # Web UI
    environment:
      - MP_MAX_MESSAGES=1000
      - MP_DATABASE=/data/mailpit.db
    volumes:
      - mailpit_data:/data

volumes:
  mailpit_data:
```

### Config for dev

```bash
ACTIX_DEMO_SMTP_HOST=mailpit
ACTIX_DEMO_SMTP_PORT=1025
ACTIX_DEMO_SMTP_USERNAME=
ACTIX_DEMO_SMTP_PASSWORD=
ACTIX_DEMO_SMTP_FROM_EMAIL=noreply@localhost
ACTIX_DEMO_SMTP_TLS_MODE=none
```

### Web UI

Browse to `http://localhost:8025` during dev to inspect all emails sent by the app. No need to check real inboxes.

---

## Production: Real SMTP Provider

### Recommended: Brevo (Sendinblue)

| Item      | Value                                |
|-----------|--------------------------------------|
| Free tier | 300 emails/day                       |
| SMTP host | smtp.brevo.com                       |
| Port      | 587 (StartTLS)                       |
| Auth      | `smtpApiKey.<api-key>` as username   |

### Alternative: Mailgun

| Item      | Value                               |
|-----------|-------------------------------------|
| Free tier | 1000 emails/month (first 3 months)  |
| SMTP host | smtp.mailgun.org                    |
| Port      | 587 (StartTLS)                      |
| Auth      | `api:key-<key>` as username         |

### Config for production

```bash
ACTIX_DEMO_SMTP_HOST=smtp.brevo.com
ACTIX_DEMO_SMTP_PORT=587
ACTIX_DEMO_SMTP_USERNAME=smtpApiKey.your_api_key_here
ACTIX_DEMO_SMTP_PASSWORD=""
ACTIX_DEMO_SMTP_FROM_EMAIL=noreply@yourdomain.com
ACTIX_DEMO_SMTP_TLS_MODE=starttls
```

---

## Env Var Switching Matrix

| Mode    | SMTP_HOST        | SMTP_PORT | TLS_MODE   | FROM_EMAIL                  |
|---------|------------------|-----------|------------|-----------------------------|
| Dev     | mailpit          | 1025      | none       | noreply@localhost           |
| Staging | smtp.brevo.com   | 587       | starttls   | noreply-staging@domain.com  |
| Prod    | smtp.brevo.com   | 587       | starttls   | noreply@domain.com          |

---

## Implementation Plan

### Phase 1: Dependencies & Config

Add `lettre` dependency to [`Cargo.toml`](Cargo.toml):

```toml
lettre = { version = "0.11", features = ["tokio1-native-tls", "builder"] }
```

Add new config fields to [`src/config.rs`](src/config.rs):

```rust
pub enum TlsMode {
    None,
    StartTls,
    Tls,
}

// Deserialize impl for string -> TlsMode conversion

#[derive(Deserialize, Debug, Clone)]
pub struct EnvConfig {
    // ... existing fields ...
    #[serde(default = "models::defaults::default_tls_mode")]
    pub smtp_tls_mode: TlsMode,
    #[serde(default = "models::defaults::default_email_token_ttl_verification_secs")]
    pub email_token_ttl_verification_secs: u64,
    #[serde(default = "models::defaults::default_email_token_ttl_reset_secs")]
    pub email_token_ttl_reset_secs: u64,
    #[serde(default = "models::defaults::default_verification_link_template")]
    pub verification_link_template: String,
    #[serde(default = "models::defaults::default_password_reset_link_template")]
    pub password_reset_link_template: String,
    // Rate limiting
    #[serde(default = "models::defaults::default_rate_limit_registration_max_requests")]
    pub rate_limit_registration_max_requests: u32,
    #[serde(default = "models::defaults::default_rate_limit_registration_window_secs")]
    pub rate_limit_registration_window_secs: u64,
    #[serde(default = "models::defaults::default_rate_limit_password_reset_max_requests")]
    pub rate_limit_password_reset_max_requests: u32,
    #[serde(default = "models::defaults::default_rate_limit_password_reset_window_secs")]
    pub rate_limit_password_reset_window_secs: u64,
}
```

Add defaults to [`src/models/defaults.rs`](src/models/defaults.rs):

```rust
pub fn default_tls_mode() -> TlsMode {
    TlsMode::None
}

pub fn default_email_token_ttl_verification_secs() -> u64 {
    86400 // 24 hours
}

pub fn default_email_token_ttl_reset_secs() -> u64 {
    900 // 15 minutes
}

pub fn default_verification_link_template() -> String {
    "https://yourapp.com/verify?token={token}&user={user_name}".to_string()
}

pub fn default_password_reset_link_template() -> String {
    "https://yourapp.com/reset-password?token={token}&user={user_name}".to_string()
}

pub fn default_rate_limit_registration_max_requests() -> u32 {
    3
}

pub fn default_rate_limit_registration_window_secs() -> u64 {
    3600 // 1 hour
}

pub fn default_rate_limit_password_reset_max_requests() -> u32 {
    5
}

pub fn default_rate_limit_password_reset_window_secs() -> u64 {
    300 // 5 minutes
}
```

### Phase 2: Database Migration

Create migration `migrations/2026-06-02-000000_add_email_to_users/up.sql`:

```sql
ALTER TABLE users ADD COLUMN email VARCHAR(255) UNIQUE NOT NULL DEFAULT '';
CREATE INDEX idx_users_email ON users(email);
```

**Note:** Start with `NOT NULL DEFAULT ''` to avoid backfilling existing rows. New registrations will provide real emails. Add validation in the registration handler.

Create migration `migrations/2026-06-02-000001_create_email_tokens/up.sql`:

```sql
CREATE TABLE email_verification_tokens (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_email_verification_tokens_hash ON email_verification_tokens(token_hash);
CREATE INDEX idx_email_verification_tokens_expires ON email_verification_tokens(expires_at) WHERE expires_at > NOW();

CREATE TABLE password_reset_tokens (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_password_reset_tokens_hash ON password_reset_tokens(token_hash);
CREATE INDEX idx_password_reset_tokens_expires ON password_reset_tokens(expires_at) WHERE expires_at > NOW();
```

**Schema notes:**
- Tokens are **hashed** before storage — never store plaintext
- `TIMESTAMPTZ` for automatic timezone handling with Diesel
- Partial indexes on `expires_at > NOW()` for fast cleanup queries
- `ON DELETE CASCADE` so tokens are cleaned up when users are soft-deleted

### Phase 3: Email Service Layer

Create [`src/services/email/mod.rs`](src/services/email/mod.rs):

```rust
use crate::errors::DomainError;
use async_trait::AsyncTrait; // or use std::marker::Send + 'static

/// Trait for sending emails
#[async_trait::async_trait]
pub trait Mailer: Send + Sync {
    async fn send_verification_email(
        &self,
        to_email: &str,
        user_name: &str,
        token: &str,
    ) -> Result<(), DomainError>;

    async fn send_reset_email(
        &self,
        to_email: &str,
        user_name: &str,
        token: &str,
    ) -> Result<(), DomainError>;
}
```

Create [`src/services/email/smtp.rs`](src/services/email/smtp.rs):

```rust
use crate::errors::DomainError;
use crate::services::email::Mailer;
use lettre::{
    message::{MultiPart, SinglePart, ContentType},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

pub struct SmtpSender {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from_email: String,
    verification_link_template: String,
    password_reset_link_template: String,
}

impl SmtpSender {
    pub fn new(
        smtp_host: &str,
        smtp_port: u16,
        tls_mode: crate::config::TlsMode,
        username: &str,
        password: &str,
        from_email: &str,
        verification_link_template: &str,
        password_reset_link_template: &str,
    ) -> Result<Self, DomainError> {
        let mut builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(smtp_host)
            .port(smtp_port);

        // Apply TLS configuration based on mode
        builder = match tls_mode {
            crate::config::TlsMode::None => builder,
            crate::config::TlsMode::StartTls => builder.tls(lettre::transport::smtp::tls::Tls::Starttls(None)),
            crate::config::TlsMode::Tls => builder.tls(lettre::transport::smtp::tls::Tls::Required(
                lettre::transport::smtp::tls::Config::builder()
                    .dangerous_accept_invalid_certs(true) // Only for staging
                    .build(),
            )),
        };

        // Apply credentials if provided
        if !username.is_empty() {
            builder = builder.credentials(Credentials::new(
                username.to_owned(),
                password.to_owned(),
            ));
        }

        let transport = builder.build();

        Ok(Self {
            transport,
            from_email: from_email.to_owned(),
            verification_link_template: verification_link_template.to_owned(),
            password_reset_link_template: password_reset_link_template.to_owned(),
        })
    }

    fn build_email(
        from: &str,
        to: &str,
        subject: &str,
        body_html: &str,
        body_text: &str,
    ) -> Result<Message, DomainError> {
        Message::builder()
            .from(from.parse().map_err(|e| DomainError::InternalError {
                message: format!("Invalid from email: {e}"),
            })?)
            .to(to.parse().map_err(|e| DomainError::InternalError {
                message: format!("Invalid to email: {e}"),
            })?)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(body_text.to_owned()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(body_html.to_owned()),
                    ),
            )
            .map_err(|e| DomainError::InternalError {
                message: format!("Failed to build email message: {e}"),
            })
    }

    fn render_verification_link(&self, token: &str, user_name: &str) -> String {
        self.verification_link_template
            .replace("{token}", token)
            .replace("{user_name}", user_name)
    }

    fn render_reset_link(&self, token: &str, user_name: &str) -> String {
        self.password_reset_link_template
            .replace("{token}", token)
            .replace("{user_name}", user_name)
    }
}

#[async_trait::async_trait]
impl Mailer for SmtpSender {
    async fn send_verification_email(
        &self,
        to_email: &str,
        user_name: &str,
        token: &str,
    ) -> Result<(), DomainError> {
        let link = self.render_verification_link(token, user_name);
        let body_html = format!(
            "<html><body><p>Hello {},</p><p>Please verify your email address by clicking the button below:</p><p><a href=\"{}\" style=\"background-color: #4CAF50; color: white; padding: 14px 20px; text-decoration: none; border-radius: 4px;\">Verify Email</a></p><p>This link expires in 24 hours.</p></body></html>",
            user_name, link
        );
        let body_text = format!(
            "Hello {},\n\nPlease verify your email address by clicking the link below:\n{}\n\nThis link expires in 24 hours.",
            user_name, link
        );

        let email = Self::build_email(
            &self.from_email,
            to_email,
            "Verify your email address",
            &body_html,
            &body_text,
        )?;

        let _ = tracing::info!("Sending verification email to {to_email}");

        self.transport
            .send(email)
            .await
            .map_err(|e| DomainError::InternalError {
                message: format!("Failed to send verification email: {e}"),
            })?;

        Ok(())
    }

    async fn send_reset_email(
        &self,
        to_email: &str,
        user_name: &str,
        token: &str,
    ) -> Result<(), DomainError> {
        let link = self.render_reset_link(token, user_name);
        let body_html = format!(
            "<html><body><p>Hello {},</p><p>We received a request to reset your password.</p><p><a href=\"{}\" style=\"background-color: #2196F3; color: white; padding: 14px 20px; text-decoration: none; border-radius: 4px;\">Reset Password</a></p><p>If you didn't request this, please ignore this email.</p><p>This link expires in 15 minutes.</p></body></html>",
            user_name, link
        );
        let body_text = format!(
            "Hello {},\n\nWe received a request to reset your password. Click the link below to reset it:\n{}\n\nIf you didn't request this, please ignore this email.\n\nThis link expires in 15 minutes.",
            user_name, link
        );

        let email = Self::build_email(
            &self.from_email,
            to_email,
            "Password Reset Request",
            &body_html,
            &body_text,
        )?;

        let _ = tracing::info!("Sending password reset email to {to_email}");

        self.transport
            .send(email)
            .await
            .map_err(|e| DomainError::InternalError {
                message: format!("Failed to send password reset email: {e}"),
            })?;

        Ok(())
    }
}
```

Create [`src/services/email/factory.rs`](src/services/email/factory.rs):

```rust
use crate::config::{EnvConfig, TlsMode};
use crate::errors::DomainError;
use crate::services::email::Mailer;
use crate::services::email::smtp::SmtpSender;

/// Creates the appropriate Mailer implementation based on config.
/// Always returns SmtpSender — config determines whether it connects to
/// a real SMTP server or a local Mailpit instance.
pub fn create_mailer(config: &EnvConfig) -> Result<Box<dyn Mailer>, DomainError> {
    let sender = SmtpSender::new(
        &config.smtp_host,
        config.smtp_port,
        config.smtp_tls_mode.clone(),
        &config.smtp_username,
        &config.smtp_password,
        &config.smtp_from_email,
        &config.verification_link_template,
        &config.password_reset_link_template,
    )?;

    Ok(Box::new(sender))
}
```

### Phase 4: Token Management

Create [`src/services/email/tokens.rs`](src/services/email/tokens.rs):

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Email token types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    EmailVerification,
    PasswordReset,
}

/// Token payload stored in Redis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailToken {
    pub user_id: String,
    pub token_type: TokenType,
    pub created_at: chrono::NaiveDateTime,
    pub expires_at: chrono::NaiveDateTime,
}

impl EmailToken {
    pub fn new(
        user_id: &str,
        token_type: TokenType,
        ttl_secs: u64,
        now: chrono::NaiveDateTime,
    ) -> Self {
        Self {
            user_id: user_id.to_owned(),
            token_type,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(ttl_secs as i64),
        }
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().naive_utc() > self.expires_at
    }
}

/// Generate a cryptographically secure random token
pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    data_encoding::BASE64URL_NOPAD.encode(&bytes)
}

/// Hash a token for database storage
pub fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
```

### Phase 5: Endpoints

| Endpoint | Method | Request Body | Response | Notes |
|---|---|---|---|---|
| `/api/registration` | POST | `{ username, email, password }` | `{ success, message }` | Updated — requires email, sends verification |
| `/api/email/verify` | POST | `{ token }` | `{ success, message }` | Validates token, sets verified flag |
| `/api/password-reset/request` | POST | `{ email }` | `{ success, message }` | Sends reset email (always returns success) |
| `/api/password-reset/complete` | POST | `{ token, new_password }` | `{ success, message }` | Validates token + resets password |
| `/api/users/me` | PATCH | `{ username?, email? }` | `{ user }` | Email change triggers re-verification |

### Phase 6: Rate Limiting

Add stricter rate limits for auth endpoints:

| Endpoint | Limit | Window | Rationale |
|---|---|---|---|
| `/api/registration` | 3 requests | 1 hour (3600s) | Prevent account spam |
| `/api/password-reset/request` | 5 requests | 5 minutes (300s) | Prevent email bombing |

### Phase 7: Integration Tests

Add tests for:
- Email verification flow (register -> verify -> login)
- Password reset flow (request -> complete -> login)
- Token expiry handling
- Invalid token rejection
- Duplicate email registration rejection
- Rate limiting on registration and password reset

---

## Risks & Mitigations

| Risk | Mitigation |
|---|---|
| Token theft from DB | Store **hashed** tokens, never plaintext |
| SMTP credentials in env | Use gitignored `.env` file; use secrets management (Vault, AWS Secrets Manager) in prod |
| Token expiry | 24 hours for verification, 15 minutes for reset |
| Email bombing | Rate limit password reset to 5 requests per 5 minutes per IP |
| Account spam | Rate limit registration to 3 requests per hour per IP |
| Token enumeration | Always return same success response even if email not found |
| Deliverability | Brevo/Mailgun handle this in prod; Mailpit only for dev |
| Existing users without email | Start with `NOT NULL DEFAULT ''` constraint, add validation on registration |

---

## File Structure

```
src/
├── config.rs                          # Add TlsMode enum + new fields
├── errors.rs                          # No changes needed
├── models/
│   └── defaults.rs                    # Add default functions for new config
├── services/
│   └── email/
│       ├── mod.rs                     # Mailer trait
│       ├── smtp.rs                    # SmtpSender implementation
│       ├── factory.rs                 # create_mailer()
│       └── tokens.rs                  # Token generation + hashing
├── actions/
│   ├── users.rs                       # Update register to require email
│   └── auth.rs                        # New: verify, password reset endpoints
├── routes/
│   └── auth.rs                        # Wire new endpoints
└── main.rs                            # Initialize Mailer at startup
```

---

## .env Additions

```bash
# Email token TTLs (seconds)
ACTIX_DEMO_EMAIL_TOKEN_TTL_VERIFICATION_SECS = 86400
ACTIX_DEMO_EMAIL_TOKEN_TTL_RESET_SECS        = 900

# Link templates (for building verification/reset URLs in emails)
ACTIX_DEMO_VERIFICATION_LINK_TEMPLATE        = https://yourapp.com/verify?token={token}&user={user_name}
ACTIX_DEMO_PASSWORD_RESET_LINK_TEMPLATE      = https://yourapp.com/reset-password?token={token}&user={user_name}

# TLS mode: none, starttls, or tls
ACTIX_DEMO_SMTP_TLS_MODE                     = none

# Rate limiting for registration and password reset
ACTIX_DEMO_RATE_LIMIT_REGISTRATION_MAX_REQUESTS = 3
ACTIX_DEMO_RATE_LIMIT_REGISTRATION_WINDOW_SECS  = 3600
ACTIX_DEMO_RATE_LIMIT_PASSWORD_RESET_MAX_REQUESTS = 5
ACTIX_DEMO_RATE_LIMIT_PASSWORD_RESET_WINDOW_SECS  = 300
```

---

## Dependencies to Add to [`Cargo.toml`](Cargo.toml)

```toml
[dependencies]
# ... existing dependencies ...
lettre       = { version = "0.11", features = ["tokio1-native-tls", "builder"] }
async-trait  = "0.1"
sha2         = "0.10"
hex          = "0.4"
data-encoding = "2.5"
```
