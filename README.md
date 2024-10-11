``` sh
docker build -t pumpman . --platform=linux/amd64
```

``` sh
diesel --database-url postgres://localhost/takeover print-schema > src/schema.rs
```
