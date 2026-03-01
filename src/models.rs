use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ItemList {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ItemListSummary {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub active_count: i64,
    pub estimated_budget: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct WishItem {
    pub id: i64,
    pub list_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub estimated_cost: Option<f64>,
    pub want_level: i64,
    pub need_level: i64,
    pub where_to_buy: Option<String>,
    pub category: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct PurchaseRecord {
    pub id: i64,
    pub item_id: i64,
    pub actual_cost: Option<f64>,
    pub purchased_at: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivedItem {
    pub item: WishItem,
    pub record: PurchaseRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WishStats {
    pub total_spent: f64,
    pub pending_budget: f64,
    pub active_count: i64,
    pub purchased_count: i64,
}
