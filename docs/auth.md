# Authentication System Documentation

## Architecture Overview

![Auth Flow Diagram](diagrams/auth_flow.png)

### Core Components

| Component           | Location                              | Responsibility                        |
| ------------------- | ------------------------------------- | ------------------------------------- |
| JWT Service         | `src/routes/auth.rs`                  | Token generation/validation           |
| Redis Session Store | `src/utils/redis_credentials_repo.rs` | Active session management             |
| User Model          | `src/models/users.rs`                 | Password storage & validation         |
| Auth Middleware     | `src/routes/auth.rs`                  | Request validation & role enforcement |

## Sequence Flow

```mermaid
sequenceDiagram
    participant Client
    participant AuthService
    participant Database
    participant Redis

    Client->>AuthService: POST /api/login {username, password}
    AuthService->>Database: Get user credentials
    Database-->>AuthService: User record
    AuthService->>AuthService: Verify bcrypt hash
    AuthService->>AuthService: Generate JWT (1yr expiry)
    AuthService->>Redis: Store token→user mapping
    AuthService-->>Client: Return X-AUTH-TOKEN

    Client->>AuthService: Request with Bearer token
    AuthService->>AuthService: Verify JWT signature
    AuthService->>Redis: Validate token exists
    Redis-->>AuthService: Session status
    AuthService-->>Client: Grant/Deny access
```

## Security Implementation

```rust
// Key security features in code
#[derive(Validator)]
#[validator(regex(regex::USERNAME_REG))]
pub struct Username(String);  // Enforces username format

#[derive(Validator)]
#[validator(line(char_length(max = 200)))]
pub struct Password(String);  // Password length constraint

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("**********")  // Mask passwords in logs
    }
}
```

### Session Management

- JWT Claims Structure:
  ```rust
  #[derive(Serialize, Deserialize)]
  pub struct VerifiedAuthDetails {
      pub user_id: UserId,
      pub username: Username,
      pub roles: Vec<RoleEnum>,  // Role-based access control
  }
  ```
- Redis Storage Format:
  ```rust
  credentials_repo.save(&user.id, &token)  // user_id → token mapping
  ```

## Recommended Improvements

```rust
// Suggested security enhancements
const TOKEN_EXPIRATION: Duration = Duration::from_days(30); // Reduced from 365
const MAX_LOGIN_ATTEMPTS: u8 = 5; // Add account lockout
```

| Security Control    | Status | Recommendation               |
| ------------------- | ------ | ---------------------------- |
| Token Rotation      | ❌     | Implement refresh tokens     |
| Password Complexity | ❌     | Add complexity rules         |
| Rate Limiting       | ❌     | Add login attempt throttling |
