//! scr/db/connect.rs

use sqlx::PgPool;

///
/// Opens a pool of connections to a Postgresql database
///
pub async fn create_pg_pool(db_url: &str) -> sqlx::Result<PgPool> {
    let pool = PgPool::connect(db_url).await?;
    Ok(pool)
}
