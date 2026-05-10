use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Like {
    pub user_id: i64,
    pub status_id: i64,
    pub created_at: String,
}
