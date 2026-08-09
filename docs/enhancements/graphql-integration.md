# GraphQL Integration Plan

## Dependencies

```toml
[dependencies]
async-graphql = "4.0.0"
async-graphql-actix-web = "4.0.0"
```

## Architecture

### Code Structure

```
src/
  graphql/
    mod.rs         # Schema root
    schema.rs      # Query/Mutation definitions
    types/         # GraphQL type mappings
      user.rs
  routes/
    graphql.rs     # Actix-web route handlers
```

### Integration Points

1. **Database Access**

   - Use Actix-web's web::block for Diesel operations

   ```rust
   async fn get_user_by_id(
       db: web::Data<PgPool>,
       id: ID
   ) -> Result<User> {
       web::block(move || {
           let mut conn = db.get()?;
           users::table
               .filter(users::user_id.eq(id.to_string()))
               .first::<models::User>(&mut conn)
       })
       .await?
       .map_err(Into::into)
   }
   ```

2. **Error Handling**
   - Implement `async_graphql::Error` conversions for `DomainError`
   ```rust
   impl From<DomainError> for async_graphql::Error {
       fn from(e: DomainError) -> Self {
           async_graphql::Error::new(e.to_string())
               .extend_with(|_, e| match e {
                   DomainError::ValidationError(_) =>
                       e.set("code", "VALIDATION_FAILED"),
                   _ => e.set("code", "INTERNAL_ERROR")
               })
       }
   }
   ```

## Coexistence Strategy

### Route Mapping

```rust
// src/main.rs
app.service(
    web::scope("/api")
        .configure(routes::rest::config) // Existing REST API
        .service(routes::graphql::graphql_route) // New GraphQL endpoint
        .service(routes::ws::websocket_route) // Existing WebSocket
);
```

### Performance Considerations

- Batch database access using DataLoader pattern
- Schema complexity analysis with `async-graphql` built-in tools
- Query depth/recursion limits configured via middleware

## Implementation Steps

1. [x] Create architecture document
1. [ ] Add dependencies to Cargo.toml
1. [ ] Create basic GraphQL schema with User query
1. [ ] Configure Actix-web route integration
1. [ ] Add error conversion layer
1. [ ] Write integration tests in `tests/integration/graphql.rs`

## Compatibility Matrix

| Component     | GraphQL Compatibility |
| ------------- | --------------------- |
| REST API      | Full (side-by-side)   |
| WebSocket     | Full (separate path)  |
| Diesel Models | Direct mapping        |
| Auth System   | Shared middleware     |
