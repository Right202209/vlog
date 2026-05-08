use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub slug: String,
}

