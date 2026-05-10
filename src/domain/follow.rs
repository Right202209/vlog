use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Follow {
    pub follower_id: i64,
    pub followee_id: i64,
    pub created_at: String,
}
