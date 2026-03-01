use crate::models::*;
use leptos::prelude::*;
use server_fn::ServerFnError;

// ── Lists ──────────────────────────────────────────────────────────────────────

#[server]
pub async fn get_item_lists_with_stats() -> Result<Vec<ItemListSummary>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::SqlitePool;

    let Extension(pool): Extension<SqlitePool> = extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let lists = sqlx::query_as::<_, ItemListSummary>(
        "SELECT
            l.id, l.name, l.created_at,
            COUNT(CASE WHEN w.status = 'active' THEN 1 END) as active_count,
            COALESCE(SUM(CASE WHEN w.status = 'active' THEN w.estimated_cost END), 0.0) as estimated_budget
         FROM item_lists l
         LEFT JOIN wish_items w ON w.list_id = l.id
         GROUP BY l.id, l.name, l.created_at
         ORDER BY l.created_at ASC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(lists)
}

#[server]
pub async fn get_item_list(id: i64) -> Result<ItemList, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::SqlitePool;

    let Extension(pool): Extension<SqlitePool> = extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let list =
        sqlx::query_as::<_, ItemList>("SELECT id, name, created_at FROM item_lists WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(list)
}

#[server]
pub async fn get_item_list_detail(id: i64) -> Result<ItemListSummary, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::SqlitePool;

    let Extension(pool): Extension<SqlitePool> = extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let detail = sqlx::query_as::<_, ItemListSummary>(
        "SELECT
            l.id, l.name, l.created_at,
            COUNT(CASE WHEN w.status = 'active' THEN 1 END) as active_count,
            COALESCE(SUM(CASE WHEN w.status = 'active' THEN w.estimated_cost END), 0.0) as estimated_budget
         FROM item_lists l
         LEFT JOIN wish_items w ON w.list_id = l.id
         WHERE l.id = ?
         GROUP BY l.id"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(detail)
}

#[server]
pub async fn rename_item_list(id: i64, name: String) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::SqlitePool;

    let Extension(pool): Extension<SqlitePool> = extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    sqlx::query("UPDATE item_lists SET name = ? WHERE id = ?")
        .bind(name.trim().to_string())
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

#[server]
pub async fn create_item_list(name: String) -> Result<i64, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::SqlitePool;

    let Extension(pool): Extension<SqlitePool> = extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let now = chrono::Utc::now().to_rfc3339();

    let result = sqlx::query("INSERT INTO item_lists (name, created_at) VALUES (?, ?)")
        .bind(name.trim().to_string())
        .bind(now)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(result.last_insert_rowid())
}

// ── Wishlist ───────────────────────────────────────────────────────────────────

#[server]
pub async fn get_wish_items(list_id: i64) -> Result<Vec<WishItem>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::SqlitePool;

    let Extension(pool): Extension<SqlitePool> = extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let items = sqlx::query_as::<_, WishItem>(
        "SELECT * FROM wish_items WHERE status = 'active' AND list_id = ?
         ORDER BY (want_level * need_level) DESC, created_at DESC",
    )
    .bind(list_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(items)
}

#[server]
pub async fn get_wish_item(id: i64) -> Result<WishItem, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::SqlitePool;

    let Extension(pool): Extension<SqlitePool> = extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let item = sqlx::query_as::<_, WishItem>("SELECT * FROM wish_items WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(item)
}

#[server]
#[allow(clippy::too_many_arguments)]
pub async fn create_wish_item(
    list_id: i64,
    name: String,
    description: String,
    estimated_cost: Option<f64>,
    want_level: i64,
    need_level: i64,
    where_to_buy: String,
    category: String,
    notes: String,
) -> Result<i64, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::SqlitePool;

    let Extension(pool): Extension<SqlitePool> = extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let now = chrono::Utc::now().to_rfc3339();

    let result = sqlx::query(
        "INSERT INTO wish_items
             (list_id, name, description, estimated_cost, want_level, need_level,
              where_to_buy, category, notes, status, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'active', ?)",
    )
    .bind(list_id)
    .bind(name)
    .bind(if description.is_empty() {
        None
    } else {
        Some(description)
    })
    .bind(estimated_cost)
    .bind(want_level)
    .bind(need_level)
    .bind(if where_to_buy.is_empty() {
        None
    } else {
        Some(where_to_buy)
    })
    .bind(if category.is_empty() {
        None
    } else {
        Some(category)
    })
    .bind(if notes.is_empty() { None } else { Some(notes) })
    .bind(now)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(result.last_insert_rowid())
}

#[server]
#[allow(clippy::too_many_arguments)]
pub async fn update_wish_item(
    id: i64,
    name: String,
    description: String,
    estimated_cost: Option<f64>,
    want_level: i64,
    need_level: i64,
    where_to_buy: String,
    category: String,
    notes: String,
) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::SqlitePool;

    let Extension(pool): Extension<SqlitePool> = extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    sqlx::query(
        "UPDATE wish_items
         SET name=?, description=?, estimated_cost=?, want_level=?, need_level=?,
             where_to_buy=?, category=?, notes=?
         WHERE id=?",
    )
    .bind(name)
    .bind(if description.is_empty() {
        None
    } else {
        Some(description)
    })
    .bind(estimated_cost)
    .bind(want_level)
    .bind(need_level)
    .bind(if where_to_buy.is_empty() {
        None
    } else {
        Some(where_to_buy)
    })
    .bind(if category.is_empty() {
        None
    } else {
        Some(category)
    })
    .bind(if notes.is_empty() { None } else { Some(notes) })
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

#[server]
pub async fn transfer_wish_item(item_id: i64, target_list_id: i64) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::SqlitePool;

    let Extension(pool): Extension<SqlitePool> = extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    sqlx::query("UPDATE wish_items SET list_id = ? WHERE id = ?")
        .bind(target_list_id)
        .bind(item_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

#[server]
pub async fn mark_as_purchased(
    id: i64,
    actual_cost: Option<f64>,
    notes: String,
) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::SqlitePool;

    let Extension(pool): Extension<SqlitePool> = extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("UPDATE wish_items SET status = 'purchased' WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    sqlx::query(
        "INSERT INTO purchase_records (item_id, actual_cost, purchased_at, notes)
         VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(actual_cost)
    .bind(now)
    .bind(if notes.is_empty() { None } else { Some(notes) })
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

#[server]
pub async fn delete_wish_item(id: i64) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::SqlitePool;

    let Extension(pool): Extension<SqlitePool> = extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    sqlx::query("DELETE FROM purchase_records WHERE item_id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    sqlx::query("DELETE FROM wish_items WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

// ── Archive ────────────────────────────────────────────────────────────────────

#[server]
pub async fn get_archive() -> Result<Vec<ArchivedItem>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::SqlitePool;

    let Extension(pool): Extension<SqlitePool> = extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Named struct avoids the 16-element tuple limit in sqlx's FromRow impls.
    #[derive(sqlx::FromRow)]
    struct ArchivedRow {
        w_id: i64,
        w_list_id: i64,
        w_name: String,
        w_description: Option<String>,
        w_estimated_cost: Option<f64>,
        w_want_level: i64,
        w_need_level: i64,
        w_where_to_buy: Option<String>,
        w_category: Option<String>,
        w_notes: Option<String>,
        w_status: String,
        w_created_at: String,
        p_id: i64,
        p_item_id: i64,
        p_actual_cost: Option<f64>,
        p_purchased_at: String,
        p_notes: Option<String>,
    }

    let rows = sqlx::query_as::<_, ArchivedRow>(
        "SELECT
            w.id          AS w_id,
            w.list_id     AS w_list_id,
            w.name        AS w_name,
            w.description AS w_description,
            w.estimated_cost AS w_estimated_cost,
            w.want_level  AS w_want_level,
            w.need_level  AS w_need_level,
            w.where_to_buy AS w_where_to_buy,
            w.category    AS w_category,
            w.notes       AS w_notes,
            w.status      AS w_status,
            w.created_at  AS w_created_at,
            p.id          AS p_id,
            p.item_id     AS p_item_id,
            p.actual_cost AS p_actual_cost,
            p.purchased_at AS p_purchased_at,
            p.notes       AS p_notes
         FROM wish_items w
         JOIN purchase_records p ON p.item_id = w.id
         WHERE w.status = 'purchased'
         ORDER BY p.purchased_at DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let archived = rows
        .into_iter()
        .map(|r| ArchivedItem {
            item: WishItem {
                id: r.w_id,
                list_id: r.w_list_id,
                name: r.w_name,
                description: r.w_description,
                estimated_cost: r.w_estimated_cost,
                want_level: r.w_want_level,
                need_level: r.w_need_level,
                where_to_buy: r.w_where_to_buy,
                category: r.w_category,
                notes: r.w_notes,
                status: r.w_status,
                created_at: r.w_created_at,
            },
            record: PurchaseRecord {
                id: r.p_id,
                item_id: r.p_item_id,
                actual_cost: r.p_actual_cost,
                purchased_at: r.p_purchased_at,
                notes: r.p_notes,
            },
        })
        .collect();

    Ok(archived)
}

#[server]
pub async fn get_archive_for_list(list_id: i64) -> Result<Vec<ArchivedItem>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::SqlitePool;

    let Extension(pool): Extension<SqlitePool> = extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    #[derive(sqlx::FromRow)]
    struct ArchivedRow {
        w_id: i64,
        w_list_id: i64,
        w_name: String,
        w_description: Option<String>,
        w_estimated_cost: Option<f64>,
        w_want_level: i64,
        w_need_level: i64,
        w_where_to_buy: Option<String>,
        w_category: Option<String>,
        w_notes: Option<String>,
        w_status: String,
        w_created_at: String,
        p_id: i64,
        p_item_id: i64,
        p_actual_cost: Option<f64>,
        p_purchased_at: String,
        p_notes: Option<String>,
    }

    let rows = sqlx::query_as::<_, ArchivedRow>(
        "SELECT
            w.id          AS w_id,
            w.list_id     AS w_list_id,
            w.name        AS w_name,
            w.description AS w_description,
            w.estimated_cost AS w_estimated_cost,
            w.want_level  AS w_want_level,
            w.need_level  AS w_need_level,
            w.where_to_buy AS w_where_to_buy,
            w.category    AS w_category,
            w.notes       AS w_notes,
            w.status      AS w_status,
            w.created_at  AS w_created_at,
            p.id          AS p_id,
            p.item_id     AS p_item_id,
            p.actual_cost AS p_actual_cost,
            p.purchased_at AS p_purchased_at,
            p.notes       AS p_notes
         FROM wish_items w
         JOIN purchase_records p ON p.item_id = w.id
         WHERE w.status = 'purchased' AND w.list_id = ?
         ORDER BY p.purchased_at DESC",
    )
    .bind(list_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let archived = rows
        .into_iter()
        .map(|r| ArchivedItem {
            item: WishItem {
                id: r.w_id,
                list_id: r.w_list_id,
                name: r.w_name,
                description: r.w_description,
                estimated_cost: r.w_estimated_cost,
                want_level: r.w_want_level,
                need_level: r.w_need_level,
                where_to_buy: r.w_where_to_buy,
                category: r.w_category,
                notes: r.w_notes,
                status: r.w_status,
                created_at: r.w_created_at,
            },
            record: PurchaseRecord {
                id: r.p_id,
                item_id: r.p_item_id,
                actual_cost: r.p_actual_cost,
                purchased_at: r.p_purchased_at,
                notes: r.p_notes,
            },
        })
        .collect();

    Ok(archived)
}

#[server]
pub async fn get_list_stats(list_id: i64) -> Result<WishStats, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::SqlitePool;

    let Extension(pool): Extension<SqlitePool> = extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let total_spent: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(p.actual_cost), 0.0)
         FROM purchase_records p
         JOIN wish_items w ON w.id = p.item_id
         WHERE w.list_id = ?",
    )
    .bind(list_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let pending_budget: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(estimated_cost), 0.0)
         FROM wish_items WHERE status = 'active' AND list_id = ?",
    )
    .bind(list_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let active_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM wish_items WHERE status = 'active' AND list_id = ?",
    )
    .bind(list_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let purchased_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM wish_items WHERE status = 'purchased' AND list_id = ?",
    )
    .bind(list_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(WishStats {
        total_spent,
        pending_budget,
        active_count,
        purchased_count,
    })
}

#[server]
pub async fn get_stats() -> Result<WishStats, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::SqlitePool;

    let Extension(pool): Extension<SqlitePool> = extract()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let total_spent: f64 =
        sqlx::query_scalar("SELECT COALESCE(SUM(actual_cost), 0.0) FROM purchase_records")
            .fetch_one(&pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

    let pending_budget: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(estimated_cost), 0.0) FROM wish_items WHERE status = 'active'",
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let active_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM wish_items WHERE status = 'active'")
            .fetch_one(&pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

    let purchased_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM wish_items WHERE status = 'purchased'")
            .fetch_one(&pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(WishStats {
        total_spent,
        pending_budget,
        active_count,
        purchased_count,
    })
}
