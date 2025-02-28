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
    loaders/       # DataLoader implementations
      user_loader.rs
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
app.service(
    web::scope("/api")
        .configure(routes::rest::config)
        .service(routes::graphql::handler)
        .service(routes::ws::websocket_route)
);
```

### Performance

- Configure Actix thread pool for blocking operations
- Use DataLoader with bounded concurrency
- Enable query caching in async-graphql

## Implementation Steps

1. [x] Create architecture document
2. [ ] Add async-graphql dependencies
3. [ ] Implement GraphQL types for core models
4. [ ] Create web::block wrapped database accessors
5. [ ] Configure Actix-web route integration
6. [ ] Add DataLoader implementations
7. [ ] Write integration tests
