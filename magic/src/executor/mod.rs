use once_cell::sync::OnceCell;

#[cfg(feature = "postgres")]
pub type DBPool = sqlx::PgPool;

#[cfg(feature = "sqlite")]
pub type DBPool = sqlx::SqlitePool;

static POOL: OnceCell<DBPool> = OnceCell::new();

pub fn init(pool: DBPool) {
    POOL.set(pool).expect("Pool already initialized");
}

pub fn get_pool() -> &'static DBPool {
    POOL.get().expect("Pool not initialized")
}