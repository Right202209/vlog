use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

