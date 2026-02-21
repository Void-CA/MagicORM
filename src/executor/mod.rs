use once_cell::sync::OnceCell;
use sqlx::PgPool;

static POOL: OnceCell<PgPool> = OnceCell::new();

pub fn init(pool: PgPool) {
    POOL.set(pool).expect("Pool already initialized");
}

pub fn get_pool() -> &'static PgPool {
    POOL.get().expect("Pool not initialized")
}