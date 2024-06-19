## Testing

### Run a PostgreSQL database instance for integration tests

```bash
docker run --name ps-db -e POSTGRES_PASSWORD=[YOUR_POSTGRES_PASSWORD] -p 5432:5432 -d postgres:15.2-alpine
```

### Run a Redis instance for integration tests

```bash
docker run --name redis-db -p "6379:6379" -d redis:7.0-alpine
```
