``` sh
docker build -t clearloop/takeover . --platform=linux/arm64
```

``` sh
diesel --database-url postgres://localhost/takeover print-schema > src/schema.rs
```
