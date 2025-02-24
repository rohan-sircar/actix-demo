# Messaging System Architecture Proposal

## Database Schema

```sql
CREATE TABLE messages (
    id SERIAL PRIMARY KEY,
    sender_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    receiver_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message_text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_read BOOLEAN NOT NULL DEFAULT false
);

-- Required indexes
CREATE INDEX idx_messages_sender ON messages(sender_id);
CREATE INDEX idx_messages_receiver ON messages(receiver_id);
CREATE INDEX idx_messages_created ON messages(created_at);
```

## Rust ORM Model

```rust
#[derive(Queryable, Identifiable, Associations)]
#[diesel(table_name = messages)]
pub struct Message {
    pub id: i32,
    pub sender_id: i32,
    pub receiver_id: i32,
    pub message_text: String,
    pub created_at: DateTime<Utc>,
    pub is_read: bool,
}

// Usage with joins returns (Message, User, User) tuples
```

## Performance Characteristics

### Query Profile

| Scenario             | Latency | Throughput   | Suitable For            |
| -------------------- | ------- | ------------ | ----------------------- |
| Single message fetch | 2-5ms   | 500-1000 QPS | Message details view    |
| 50-message list      | 15-25ms | 200 QPS      | Chat history pagination |
| Full scan            | 500ms+  | N/A          | Admin/reports only      |

## Optimization Strategy

1. **Indexing**

```sql
CREATE INDEX idx_msg_covering ON messages
  (sender_id, receiver_id)
  INCLUDE (message_text, created_at, is_read);
```

2. **Scaling Path**

- Phase 1 (<500k messages): Base indexes
- Phase 2 (500k-5M): Read replicas + connection pooling
- Phase 3 (>5M): Partition by created_at month

3. **Alternative Approaches**

- **Denormalization**: Add username columns (+15% storage)
- **Caching**: Redis cache for recent messages (TTL: 1h)
- **Materialized Views**: Pre-joined data updated hourly

## Trade-off Analysis

| Approach         | Pros                            | Cons              |
| ---------------- | ------------------------------- | ----------------- |
| **Normalized**   | Data integrity, Smaller storage | Join overhead     |
| **Denormalized** | Faster reads                    | Update complexity |
| **Cached**       | Best performance                | Stale data risk   |
