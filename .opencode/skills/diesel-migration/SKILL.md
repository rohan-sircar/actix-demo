# Diesel Migration Workflow

## Purpose
This skill provides the workflow for running Diesel migrations, regenerating schema.rs, and verifying changes against the PostgreSQL Docker container.

## Prerequisites
- PostgreSQL running in Docker container named `actix-demo-postgres`
- Database name: `actix_demo`
- User: `postgres`
- Diesel CLI installed and configured

## Workflow

### 1. Create a Migration
```bash
diesel migration generate <migration_name>
```
This creates a new migration directory in `migrations/` with `up.sql` and `down.sql` files.

### 2. Write the Migration SQL
Edit the `up.sql` file in the newly created migration directory to define your schema changes.

### 3. Run the Migration
```bash
diesel migration run
```
This applies all pending migrations to the database.

### 4. Regenerate schema.rs
After running migrations, regenerate the Diesel schema file:
```bash
diesel print-schema > src/schema.rs
```
Or if using a custom output location:
```bash
diesel print-schema -o src/schema.rs
```

### 5. Verify Against Docker Container
Connect to the PostgreSQL Docker container to verify the changes:

```bash
# List all tables
docker exec actix-demo-postgres psql -U postgres -d actix_demo -c "\dt"

# Describe a specific table
docker exec actix-demo-postgres psql -U postgres -d actix_demo -c "\d <table_name>"

# Check enum types
docker exec actix-demo-postgres psql -U postgres -d actix_demo -c "\dT+ <enum_type_name>"

# List all schemas
docker exec actix-demo-postgres psql -U postgres -d actix_demo -c "\dn"

# Check migration status
docker exec actix-demo-postgres psql -U postgres -d actix_demo -c "SELECT * FROM __diesel_schema_migrations ORDER BY version DESC;"
```

### 6. Rollback a Migration (if needed)
```bash
diesel migration revert
```
This reverts the last migration. To revert multiple:
```bash
diesel migration revert -n <number_of_migrations>
```

## Environment Configuration
The database connection is read from the `.env` file via `DATABASE_URL` environment variable.

## Common Patterns

### Adding a new column to an existing table
```sql
ALTER TABLE users
ADD COLUMN new_column VARCHAR(255) DEFAULT NULL;
```

### Creating a custom enum type
```sql
CREATE TYPE status_type AS ENUM ('pending', 'active', 'completed');

ALTER TABLE orders
ADD COLUMN status status_type DEFAULT 'pending';
```

### Adding a unique index with condition
```sql
CREATE UNIQUE INDEX idx_table_column ON table_name (column)
WHERE column IS NOT NULL;
```

### Adding a foreign key
```sql
ALTER TABLE children
ADD CONSTRAINT fk_children_parent
FOREIGN KEY (parent_id) REFERENCES parents(id) ON DELETE CASCADE;
```

## Verification Checklist
- [ ] Migration runs without errors (`diesel migration run`)
- [ ] schema.rs is regenerated and matches the database
- [ ] Table structure verified in Docker container (`\d table_name`)
- [ ] Enum types verified (`\dT+ type_name`)
- [ ] Indexes verified in table description
- [ ] Foreign keys verified in table description
- [ ] No pending migrations (`diesel migration pending` returns `false`)

## Troubleshooting

### Migration fails with "relation already exists"
The migration was likely already applied. Check:
```bash
diesel migration revert
# Fix the up.sql if needed
diesel migration run
```

### schema.rs doesn't match database
Run `diesel print-schema` again to regenerate. Ensure you're connected to the correct database.

### Can't connect to Docker container
Verify the container is running:
```bash
docker ps | grep actix-demo-postgres
```
Restart if needed:
```bash
docker restart actix-demo-postgres
```
