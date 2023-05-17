//! scr/db/connect.rs

use redis::{Client, RedisResult};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
//use std::env;

///
/// Returns a pool of connections to a Postgresql database
/// or a sqlx error
///
pub async fn create_pg_pool(db_url: String) -> sqlx::Result<Pool<Postgres>> {
    return match PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
    {
        Ok(pool) => {
            println!("✅ -- Connection to the database is successful! PgPool created");
            Ok(pool)
        }
        Err(err) => {
            println!("🔥 -- Failed to connect to the database: {:?}", err);
            std::process::exit(1)
        }
    };
}

///
/// Returns à redis client or a RedisError
///
pub async fn create_redis_client(redis_data_from_config: String) -> RedisResult<Client> {
    return match Client::open(redis_data_from_config) {
        Ok(client) => {
            println!("✅ -- Connection to Redis is successful!");
            Ok(client)
        }
        Err(err) => {
            println!("🔥 -- Error connecting to Redis: {}", err);
            std::process::exit(1);
        }
    };
}
