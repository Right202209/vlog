use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Asset {
    pub id: i64,
    pub original_name: String,
    pub stored_path: String,
    pub mime: String,
    pub byte_size: i64,
    pub created_at: String,
}
