# Laundry API

This repository holds the source code for the api used to store and retreive reports from a Postgres database.
This application makes heavy use of the [actix-web](https://crates.io/crates/actix-web) and [sqlx](https://crates.io/crates/sqlx) crates.

Swagger documentation of the API is provided by the [utoipa](https://crates.io/crates/utoipa) crate and can be found with the server running at:
```
http://<server-address>:8080/docs/
```
