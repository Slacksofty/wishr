use crate::models::*;
use leptos::prelude::*;
use server_fn::ServerFnError;

// ── Tests ───────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::models::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::SqlitePool;

    // ── Infrastructure ──────────────────────────────────────────────────────

    /// In-memory SQLite pool with migrations applied.
    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("failed to open in-memory SQLite");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrations failed");
        pool
    }

    /// Insert a row into item_lists and return its id.
    async fn seed_list(pool: &SqlitePool, name: &str, created_at: &str) -> i64 {
        sqlx::query("INSERT INTO item_lists (name, created_at) VALUES (?, ?)")
            .bind(name)
            .bind(created_at)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    /// Insert an active wish_item and return its id.
    async fn seed_item(
        pool: &SqlitePool,
        list_id: i64,
        name: &str,
        want_level: i64,
        need_level: i64,
        estimated_cost: Option<f64>,
        created_at: &str,
    ) -> i64 {
        sqlx::query(
            "INSERT INTO wish_items
             (list_id, name, description, estimated_cost, want_level, need_level,
              where_to_buy, category, notes, status, created_at)
             VALUES (?, ?, NULL, ?, ?, ?, NULL, NULL, NULL, 'active', ?)",
        )
        .bind(list_id)
        .bind(name)
        .bind(estimated_cost)
        .bind(want_level)
        .bind(need_level)
        .bind(created_at)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_rowid()
    }

    /// Mark an item as purchased and insert a matching purchase_record.
    async fn seed_purchase(
        pool: &SqlitePool,
        item_id: i64,
        actual_cost: Option<f64>,
        purchased_at: &str,
    ) {
        sqlx::query("UPDATE wish_items SET status = 'purchased' WHERE id = ?")
            .bind(item_id)
            .execute(pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO purchase_records (item_id, actual_cost, purchased_at, notes)
             VALUES (?, ?, ?, NULL)",
        )
        .bind(item_id)
        .bind(actual_cost)
        .bind(purchased_at)
        .execute(pool)
        .await
        .unwrap();
    }

    /// Run the get_list_stats SQL against a pool directly (mirrors items.rs).
    async fn query_list_stats(pool: &SqlitePool, list_id: i64) -> WishStats {
        let total_spent: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(p.actual_cost), 0.0)
             FROM purchase_records p
             JOIN wish_items w ON w.id = p.item_id
             WHERE w.list_id = ?",
        )
        .bind(list_id)
        .fetch_one(pool)
        .await
        .unwrap();

        let pending_budget: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(estimated_cost), 0.0)
             FROM wish_items WHERE status = 'active' AND list_id = ?",
        )
        .bind(list_id)
        .fetch_one(pool)
        .await
        .unwrap();

        let active_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM wish_items WHERE status = 'active' AND list_id = ?",
        )
        .bind(list_id)
        .fetch_one(pool)
        .await
        .unwrap();

        let purchased_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM wish_items WHERE status = 'purchased' AND list_id = ?",
        )
        .bind(list_id)
        .fetch_one(pool)
        .await
        .unwrap();

        WishStats {
            total_spent,
            pending_budget,
            active_count,
            purchased_count,
        }
    }

    /// Run the get_stats (global) SQL against a pool directly (mirrors items.rs).
    async fn query_global_stats(pool: &SqlitePool) -> WishStats {
        let total_spent: f64 =
            sqlx::query_scalar("SELECT COALESCE(SUM(actual_cost), 0.0) FROM purchase_records")
                .fetch_one(pool)
                .await
                .unwrap();

        let pending_budget: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(estimated_cost), 0.0) FROM wish_items WHERE status = 'active'",
        )
        .fetch_one(pool)
        .await
        .unwrap();

        let active_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM wish_items WHERE status = 'active'")
                .fetch_one(pool)
                .await
                .unwrap();

        let purchased_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM wish_items WHERE status = 'purchased'")
                .fetch_one(pool)
                .await
                .unwrap();

        WishStats {
            total_spent,
            pending_budget,
            active_count,
            purchased_count,
        }
    }

    /// Run the get_archive / get_archive_for_list SQL (mirrors items.rs).
    async fn query_archive(pool: &SqlitePool, list_id: Option<i64>) -> Vec<ArchivedItem> {
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

        let to_archived = |r: ArchivedRow| ArchivedItem {
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
        };

        let base = "SELECT
            w.id AS w_id, w.list_id AS w_list_id, w.name AS w_name,
            w.description AS w_description, w.estimated_cost AS w_estimated_cost,
            w.want_level AS w_want_level, w.need_level AS w_need_level,
            w.where_to_buy AS w_where_to_buy, w.category AS w_category,
            w.notes AS w_notes, w.status AS w_status, w.created_at AS w_created_at,
            p.id AS p_id, p.item_id AS p_item_id, p.actual_cost AS p_actual_cost,
            p.purchased_at AS p_purchased_at, p.notes AS p_notes
         FROM wish_items w
         JOIN purchase_records p ON p.item_id = w.id
         WHERE w.status = 'purchased'";

        if let Some(lid) = list_id {
            sqlx::query_as::<_, ArchivedRow>(&format!(
                "{} AND w.list_id = ? ORDER BY p.purchased_at DESC",
                base
            ))
            .bind(lid)
            .fetch_all(pool)
            .await
            .unwrap()
            .into_iter()
            .map(to_archived)
            .collect()
        } else {
            sqlx::query_as::<_, ArchivedRow>(&format!("{} ORDER BY p.purchased_at DESC", base))
                .fetch_all(pool)
                .await
                .unwrap()
                .into_iter()
                .map(to_archived)
                .collect()
        }
    }

    // ── List management ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn create_list_returns_nonzero_id() {
        let pool = test_pool().await;
        let id = seed_list(&pool, "Gifts", "2025-01-01T00:00:00Z").await;
        assert!(id > 0);
    }

    #[tokio::test]
    async fn create_list_multiple_have_unique_ids() {
        let pool = test_pool().await;
        let id1 = seed_list(&pool, "A", "2025-01-01T00:00:00Z").await;
        let id2 = seed_list(&pool, "B", "2025-01-02T00:00:00Z").await;
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn create_list_trims_whitespace() {
        let pool = test_pool().await;
        let name = "  Trimmed  ".trim().to_string();
        let id = sqlx::query(
            "INSERT INTO item_lists (name, created_at) VALUES (?, '2025-01-01T00:00:00Z')",
        )
        .bind(&name)
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_rowid();
        let stored: String = sqlx::query_scalar("SELECT name FROM item_lists WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(stored, "Trimmed");
    }

    #[tokio::test]
    async fn get_list_returns_correct_fields() {
        let pool = test_pool().await;
        let id = seed_list(&pool, "Birthday", "2025-03-01T12:00:00Z").await;
        let list = sqlx::query_as::<_, ItemList>(
            "SELECT id, name, created_at FROM item_lists WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(list.id, id);
        assert_eq!(list.name, "Birthday");
        assert_eq!(list.created_at, "2025-03-01T12:00:00Z");
    }

    #[tokio::test]
    async fn get_list_missing_id_returns_error() {
        let pool = test_pool().await;
        let result = sqlx::query_as::<_, ItemList>(
            "SELECT id, name, created_at FROM item_lists WHERE id = ?",
        )
        .bind(9999_i64)
        .fetch_one(&pool)
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn rename_list_persists() {
        let pool = test_pool().await;
        let id = seed_list(&pool, "OldName", "2025-01-01T00:00:00Z").await;
        sqlx::query("UPDATE item_lists SET name = ? WHERE id = ?")
            .bind("NewName")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
        let stored: String = sqlx::query_scalar("SELECT name FROM item_lists WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(stored, "NewName");
    }

    #[tokio::test]
    async fn rename_list_trims_whitespace() {
        let pool = test_pool().await;
        let id = seed_list(&pool, "OldName", "2025-01-01T00:00:00Z").await;
        let trimmed = "  Trimmed  ".trim().to_string();
        sqlx::query("UPDATE item_lists SET name = ? WHERE id = ?")
            .bind(&trimmed)
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
        let stored: String = sqlx::query_scalar("SELECT name FROM item_lists WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(stored, "Trimmed");
    }

    #[tokio::test]
    async fn lists_ordered_by_created_at_asc() {
        let pool = test_pool().await;
        let id_a = seed_list(&pool, "Alpha", "2025-01-01T00:00:00Z").await;
        let id_b = seed_list(&pool, "Beta", "2025-01-03T00:00:00Z").await;
        let id_c = seed_list(&pool, "Gamma", "2025-01-02T00:00:00Z").await;
        let lists = sqlx::query_as::<_, ItemList>(
            "SELECT id, name, created_at FROM item_lists ORDER BY created_at ASC",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        // General list from migration is also present; find positions by id
        let pos = |id: i64| lists.iter().position(|l| l.id == id).unwrap();
        assert!(pos(id_a) < pos(id_c), "Alpha (Jan 1) before Gamma (Jan 2)");
        assert!(pos(id_c) < pos(id_b), "Gamma (Jan 2) before Beta (Jan 3)");
    }

    #[tokio::test]
    async fn stats_empty_list_is_all_zeros() {
        let pool = test_pool().await;
        let id = seed_list(&pool, "Empty", "2025-01-01T00:00:00Z").await;
        let s = query_list_stats(&pool, id).await;
        assert_eq!(s.active_count, 0);
        assert_eq!(s.purchased_count, 0);
        assert!((s.pending_budget).abs() < 1e-9);
        assert!((s.total_spent).abs() < 1e-9);
    }

    #[tokio::test]
    async fn stats_counts_only_active_items() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "Mixed", "2025-01-01T00:00:00Z").await;
        seed_item(
            &pool,
            list_id,
            "Active",
            3,
            3,
            Some(10.0),
            "2025-01-01T00:00:00Z",
        )
        .await;
        let p = seed_item(
            &pool,
            list_id,
            "ToBuy",
            3,
            3,
            Some(20.0),
            "2025-01-02T00:00:00Z",
        )
        .await;
        seed_purchase(&pool, p, Some(18.0), "2025-01-03T00:00:00Z").await;
        let s = query_list_stats(&pool, list_id).await;
        assert_eq!(s.active_count, 1);
        assert_eq!(s.purchased_count, 1);
        assert!((s.pending_budget - 10.0).abs() < 1e-9); // only active item cost
        assert!((s.total_spent - 18.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn stats_sums_estimated_costs() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "Sums", "2025-01-01T00:00:00Z").await;
        seed_item(
            &pool,
            list_id,
            "A",
            3,
            3,
            Some(100.0),
            "2025-01-01T00:00:00Z",
        )
        .await;
        seed_item(
            &pool,
            list_id,
            "B",
            3,
            3,
            Some(50.0),
            "2025-01-02T00:00:00Z",
        )
        .await;
        seed_item(
            &pool,
            list_id,
            "C",
            3,
            3,
            Some(25.5),
            "2025-01-03T00:00:00Z",
        )
        .await;
        let s = query_list_stats(&pool, list_id).await;
        assert!((s.pending_budget - 175.5).abs() < 1e-9);
        assert_eq!(s.active_count, 3);
    }

    #[tokio::test]
    async fn stats_null_cost_treated_as_zero() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "NullCost", "2025-01-01T00:00:00Z").await;
        seed_item(&pool, list_id, "NoCost", 3, 3, None, "2025-01-01T00:00:00Z").await;
        seed_item(
            &pool,
            list_id,
            "WithCost",
            3,
            3,
            Some(50.0),
            "2025-01-02T00:00:00Z",
        )
        .await;
        let s = query_list_stats(&pool, list_id).await;
        assert!((s.pending_budget - 50.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn detail_matches_stats_query() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "Detail", "2025-01-01T00:00:00Z").await;
        seed_item(
            &pool,
            list_id,
            "Item",
            3,
            3,
            Some(30.0),
            "2025-01-01T00:00:00Z",
        )
        .await;
        let detail = sqlx::query_as::<_, ItemListSummary>(
            "SELECT l.id, l.name, l.created_at,
                    COUNT(CASE WHEN w.status = 'active' THEN 1 END) as active_count,
                    COALESCE(SUM(CASE WHEN w.status = 'active' THEN w.estimated_cost END), 0.0)
                        as estimated_budget
             FROM item_lists l
             LEFT JOIN wish_items w ON w.list_id = l.id
             WHERE l.id = ?
             GROUP BY l.id",
        )
        .bind(list_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(detail.active_count, 1);
        assert!((detail.estimated_budget - 30.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn detail_missing_id_returns_error() {
        let pool = test_pool().await;
        let result = sqlx::query_as::<_, ItemListSummary>(
            "SELECT l.id, l.name, l.created_at,
                    COUNT(CASE WHEN w.status = 'active' THEN 1 END) as active_count,
                    COALESCE(SUM(CASE WHEN w.status = 'active' THEN w.estimated_cost END), 0.0)
                        as estimated_budget
             FROM item_lists l
             LEFT JOIN wish_items w ON w.list_id = l.id
             WHERE l.id = ?
             GROUP BY l.id",
        )
        .bind(9999_i64)
        .fetch_one(&pool)
        .await;
        assert!(result.is_err());
    }

    // ── Wish items ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn create_item_returns_nonzero_id() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let id = seed_item(&pool, list_id, "Widget", 3, 3, None, "2025-01-01T00:00:00Z").await;
        assert!(id > 0);
    }

    #[tokio::test]
    async fn create_item_status_is_active() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let id = seed_item(&pool, list_id, "Widget", 3, 3, None, "2025-01-01T00:00:00Z").await;
        let status: String = sqlx::query_scalar("SELECT status FROM wish_items WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status, "active");
    }

    #[tokio::test]
    async fn create_item_empty_strings_become_null() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let desc = "";
        let wtb = "";
        let cat = "";
        let notes = "";
        let id = sqlx::query(
            "INSERT INTO wish_items
             (list_id, name, description, estimated_cost, want_level, need_level,
              where_to_buy, category, notes, status, created_at)
             VALUES (?, 'Item', ?, NULL, 3, 3, ?, ?, ?, 'active', '2025-01-01T00:00:00Z')",
        )
        .bind(list_id)
        .bind(if desc.is_empty() { None } else { Some(desc) })
        .bind(if wtb.is_empty() { None } else { Some(wtb) })
        .bind(if cat.is_empty() { None } else { Some(cat) })
        .bind(if notes.is_empty() { None } else { Some(notes) })
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_rowid();
        let item = sqlx::query_as::<_, WishItem>("SELECT * FROM wish_items WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(item.description.is_none());
        assert!(item.where_to_buy.is_none());
        assert!(item.category.is_none());
        assert!(item.notes.is_none());
    }

    #[tokio::test]
    async fn create_item_non_empty_optionals_preserved() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let id = sqlx::query(
            "INSERT INTO wish_items
             (list_id, name, description, estimated_cost, want_level, need_level,
              where_to_buy, category, notes, status, created_at)
             VALUES (?, 'Item', 'Desc', 29.99, 3, 3, 'amazon.com', 'Electronics', 'Deal', 'active',
                     '2025-01-01T00:00:00Z')",
        )
        .bind(list_id)
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_rowid();
        let item = sqlx::query_as::<_, WishItem>("SELECT * FROM wish_items WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(item.description.as_deref(), Some("Desc"));
        assert_eq!(item.where_to_buy.as_deref(), Some("amazon.com"));
        assert_eq!(item.category.as_deref(), Some("Electronics"));
        assert_eq!(item.notes.as_deref(), Some("Deal"));
        assert!((item.estimated_cost.unwrap() - 29.99).abs() < 1e-9);
    }

    #[tokio::test]
    async fn create_item_null_estimated_cost() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let id = seed_item(&pool, list_id, "Free", 3, 3, None, "2025-01-01T00:00:00Z").await;
        let cost: Option<f64> =
            sqlx::query_scalar("SELECT estimated_cost FROM wish_items WHERE id = ?")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(cost.is_none());
    }

    #[tokio::test]
    async fn create_item_float_cost_stored() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let id = seed_item(
            &pool,
            list_id,
            "Item",
            3,
            3,
            Some(99.99),
            "2025-01-01T00:00:00Z",
        )
        .await;
        let cost: f64 = sqlx::query_scalar("SELECT estimated_cost FROM wish_items WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!((cost - 99.99).abs() < 1e-9);
    }

    #[tokio::test]
    async fn get_item_returns_all_fields() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let id = sqlx::query(
            "INSERT INTO wish_items
             (list_id, name, description, estimated_cost, want_level, need_level,
              where_to_buy, category, notes, status, created_at)
             VALUES (?, 'Full', 'Desc', 49.99, 4, 5, 'shop.com', 'Toys', 'Nice', 'active',
                     '2025-06-15T10:00:00Z')",
        )
        .bind(list_id)
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_rowid();
        let item = sqlx::query_as::<_, WishItem>("SELECT * FROM wish_items WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(item.id, id);
        assert_eq!(item.list_id, list_id);
        assert_eq!(item.name, "Full");
        assert_eq!(item.description.as_deref(), Some("Desc"));
        assert!((item.estimated_cost.unwrap() - 49.99).abs() < 1e-9);
        assert_eq!(item.want_level, 4);
        assert_eq!(item.need_level, 5);
        assert_eq!(item.where_to_buy.as_deref(), Some("shop.com"));
        assert_eq!(item.category.as_deref(), Some("Toys"));
        assert_eq!(item.notes.as_deref(), Some("Nice"));
        assert_eq!(item.status, "active");
        assert_eq!(item.created_at, "2025-06-15T10:00:00Z");
    }

    #[tokio::test]
    async fn get_item_missing_returns_error() {
        let pool = test_pool().await;
        let result = sqlx::query_as::<_, WishItem>("SELECT * FROM wish_items WHERE id = ?")
            .bind(9999_i64)
            .fetch_one(&pool)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_items_excludes_purchased() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let active = seed_item(&pool, list_id, "Active", 3, 3, None, "2025-01-01T00:00:00Z").await;
        let p = seed_item(
            &pool,
            list_id,
            "Purchased",
            3,
            3,
            None,
            "2025-01-02T00:00:00Z",
        )
        .await;
        seed_purchase(&pool, p, None, "2025-01-03T00:00:00Z").await;
        let items = sqlx::query_as::<_, WishItem>(
            "SELECT * FROM wish_items WHERE status = 'active' AND list_id = ?
             ORDER BY (want_level * need_level) DESC, created_at DESC",
        )
        .bind(list_id)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, active);
    }

    #[tokio::test]
    async fn get_items_sorted_by_priority_desc() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let low = seed_item(&pool, list_id, "Low", 1, 1, None, "2025-01-01T00:00:00Z").await; // 1×1=1
        let high = seed_item(&pool, list_id, "High", 5, 5, None, "2025-01-02T00:00:00Z").await; // 5×5=25
        let mid = seed_item(&pool, list_id, "Mid", 3, 3, None, "2025-01-03T00:00:00Z").await; // 3×3=9
        let items = sqlx::query_as::<_, WishItem>(
            "SELECT * FROM wish_items WHERE status = 'active' AND list_id = ?
             ORDER BY (want_level * need_level) DESC, created_at DESC",
        )
        .bind(list_id)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(items[0].id, high);
        assert_eq!(items[1].id, mid);
        assert_eq!(items[2].id, low);
    }

    #[tokio::test]
    async fn get_items_secondary_sort_by_created_at_desc() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let older = seed_item(&pool, list_id, "Older", 3, 3, None, "2025-01-01T00:00:00Z").await;
        let newer = seed_item(&pool, list_id, "Newer", 3, 3, None, "2025-01-02T00:00:00Z").await;
        let items = sqlx::query_as::<_, WishItem>(
            "SELECT * FROM wish_items WHERE status = 'active' AND list_id = ?
             ORDER BY (want_level * need_level) DESC, created_at DESC",
        )
        .bind(list_id)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(items[0].id, newer, "newer item first when priority ties");
        assert_eq!(items[1].id, older);
    }

    #[tokio::test]
    async fn get_items_isolated_to_list() {
        let pool = test_pool().await;
        let list_a = seed_list(&pool, "A", "2025-01-01T00:00:00Z").await;
        let list_b = seed_list(&pool, "B", "2025-01-02T00:00:00Z").await;
        let item_a = seed_item(&pool, list_a, "ItemA", 3, 3, None, "2025-01-01T00:00:00Z").await;
        seed_item(&pool, list_b, "ItemB", 3, 3, None, "2025-01-01T00:00:00Z").await;
        let items = sqlx::query_as::<_, WishItem>(
            "SELECT * FROM wish_items WHERE status = 'active' AND list_id = ?
             ORDER BY (want_level * need_level) DESC, created_at DESC",
        )
        .bind(list_a)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, item_a);
    }

    #[tokio::test]
    async fn update_item_persists_changes() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let id = seed_item(&pool, list_id, "Old", 3, 3, None, "2025-01-01T00:00:00Z").await;
        sqlx::query(
            "UPDATE wish_items
             SET name=?, description=?, estimated_cost=?, want_level=?, need_level=?,
                 where_to_buy=?, category=?, notes=?
             WHERE id=?",
        )
        .bind("New")
        .bind(Some("Updated"))
        .bind(Some(55.0_f64))
        .bind(5_i64)
        .bind(4_i64)
        .bind(Some("shop.com"))
        .bind(Some("Gadgets"))
        .bind(Some("Notes"))
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
        let item = sqlx::query_as::<_, WishItem>("SELECT * FROM wish_items WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(item.name, "New");
        assert_eq!(item.description.as_deref(), Some("Updated"));
        assert!((item.estimated_cost.unwrap() - 55.0).abs() < 1e-9);
        assert_eq!(item.want_level, 5);
        assert_eq!(item.need_level, 4);
        assert_eq!(item.where_to_buy.as_deref(), Some("shop.com"));
        assert_eq!(item.category.as_deref(), Some("Gadgets"));
        assert_eq!(item.notes.as_deref(), Some("Notes"));
    }

    #[tokio::test]
    async fn update_item_empty_strings_become_null() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let id = sqlx::query(
            "INSERT INTO wish_items
             (list_id, name, description, estimated_cost, want_level, need_level,
              where_to_buy, category, notes, status, created_at)
             VALUES (?, 'Item', 'Desc', NULL, 3, 3, 'shop', 'Cat', 'Notes', 'active',
                     '2025-01-01T00:00:00Z')",
        )
        .bind(list_id)
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_rowid();
        let (desc, wtb, cat, notes) = ("", "", "", "");
        sqlx::query(
            "UPDATE wish_items
             SET name=?, description=?, estimated_cost=?, want_level=?, need_level=?,
                 where_to_buy=?, category=?, notes=?
             WHERE id=?",
        )
        .bind("Item")
        .bind(if desc.is_empty() { None } else { Some(desc) })
        .bind(None::<f64>)
        .bind(3_i64)
        .bind(3_i64)
        .bind(if wtb.is_empty() { None } else { Some(wtb) })
        .bind(if cat.is_empty() { None } else { Some(cat) })
        .bind(if notes.is_empty() { None } else { Some(notes) })
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
        let item = sqlx::query_as::<_, WishItem>("SELECT * FROM wish_items WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(item.description.is_none());
        assert!(item.where_to_buy.is_none());
        assert!(item.category.is_none());
        assert!(item.notes.is_none());
    }

    #[tokio::test]
    async fn update_item_does_not_change_status() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let id = seed_item(&pool, list_id, "Item", 3, 3, None, "2025-01-01T00:00:00Z").await;
        sqlx::query(
            "UPDATE wish_items
             SET name=?, description=?, estimated_cost=?, want_level=?, need_level=?,
                 where_to_buy=?, category=?, notes=?
             WHERE id=?",
        )
        .bind("Updated")
        .bind(None::<String>)
        .bind(None::<f64>)
        .bind(3_i64)
        .bind(3_i64)
        .bind(None::<String>)
        .bind(None::<String>)
        .bind(None::<String>)
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
        let status: String = sqlx::query_scalar("SELECT status FROM wish_items WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status, "active");
    }

    // ── Transfer ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn transfer_moves_item_to_target_list() {
        let pool = test_pool().await;
        let list_a = seed_list(&pool, "A", "2025-01-01T00:00:00Z").await;
        let list_b = seed_list(&pool, "B", "2025-01-02T00:00:00Z").await;
        let item = seed_item(&pool, list_a, "Item", 3, 3, None, "2025-01-01T00:00:00Z").await;
        sqlx::query("UPDATE wish_items SET list_id = ? WHERE id = ?")
            .bind(list_b)
            .bind(item)
            .execute(&pool)
            .await
            .unwrap();
        let stored_list_id: i64 = sqlx::query_scalar("SELECT list_id FROM wish_items WHERE id = ?")
            .bind(item)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(stored_list_id, list_b);
    }

    #[tokio::test]
    async fn transfer_item_visible_in_new_list() {
        let pool = test_pool().await;
        let list_a = seed_list(&pool, "A", "2025-01-01T00:00:00Z").await;
        let list_b = seed_list(&pool, "B", "2025-01-02T00:00:00Z").await;
        let item = seed_item(&pool, list_a, "Item", 3, 3, None, "2025-01-01T00:00:00Z").await;
        sqlx::query("UPDATE wish_items SET list_id = ? WHERE id = ?")
            .bind(list_b)
            .bind(item)
            .execute(&pool)
            .await
            .unwrap();
        let items_b = sqlx::query_as::<_, WishItem>(
            "SELECT * FROM wish_items WHERE status = 'active' AND list_id = ?",
        )
        .bind(list_b)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert!(items_b.iter().any(|i| i.id == item));
    }

    #[tokio::test]
    async fn transfer_item_absent_from_old_list() {
        let pool = test_pool().await;
        let list_a = seed_list(&pool, "A", "2025-01-01T00:00:00Z").await;
        let list_b = seed_list(&pool, "B", "2025-01-02T00:00:00Z").await;
        let item = seed_item(&pool, list_a, "Item", 3, 3, None, "2025-01-01T00:00:00Z").await;
        sqlx::query("UPDATE wish_items SET list_id = ? WHERE id = ?")
            .bind(list_b)
            .bind(item)
            .execute(&pool)
            .await
            .unwrap();
        let items_a = sqlx::query_as::<_, WishItem>(
            "SELECT * FROM wish_items WHERE status = 'active' AND list_id = ?",
        )
        .bind(list_a)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert!(!items_a.iter().any(|i| i.id == item));
    }

    // ── Delete ──────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn delete_active_item_removes_it() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let id = seed_item(
            &pool,
            list_id,
            "ToDelete",
            3,
            3,
            None,
            "2025-01-01T00:00:00Z",
        )
        .await;
        sqlx::query("DELETE FROM purchase_records WHERE item_id = ?")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM wish_items WHERE id = ?")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
        let result = sqlx::query_as::<_, WishItem>("SELECT * FROM wish_items WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete_purchased_item_removes_item_and_record() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let id = seed_item(
            &pool,
            list_id,
            "ToDelete",
            3,
            3,
            None,
            "2025-01-01T00:00:00Z",
        )
        .await;
        seed_purchase(&pool, id, Some(10.0), "2025-01-02T00:00:00Z").await;
        let record_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM purchase_records WHERE item_id = ?")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(record_count, 1, "precondition: record exists before delete");
        sqlx::query("DELETE FROM purchase_records WHERE item_id = ?")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM wish_items WHERE id = ?")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
        assert!(
            sqlx::query_as::<_, WishItem>("SELECT * FROM wish_items WHERE id = ?")
                .bind(id)
                .fetch_one(&pool)
                .await
                .is_err()
        );
        let after: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM purchase_records WHERE item_id = ?")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(after, 0);
    }

    // ── Mark as purchased ───────────────────────────────────────────────────

    #[tokio::test]
    async fn mark_purchased_changes_status() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let id = seed_item(&pool, list_id, "Item", 3, 3, None, "2025-01-01T00:00:00Z").await;
        seed_purchase(&pool, id, None, "2025-01-02T00:00:00Z").await;
        let status: String = sqlx::query_scalar("SELECT status FROM wish_items WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status, "purchased");
    }

    #[tokio::test]
    async fn mark_purchased_creates_record() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let id = seed_item(&pool, list_id, "Item", 3, 3, None, "2025-01-01T00:00:00Z").await;
        seed_purchase(&pool, id, Some(42.0), "2025-01-02T00:00:00Z").await;
        let record = sqlx::query_as::<_, PurchaseRecord>(
            "SELECT id, item_id, actual_cost, purchased_at, notes FROM purchase_records
             WHERE item_id = ?",
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(record.item_id, id);
        assert!((record.actual_cost.unwrap() - 42.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn mark_purchased_item_absent_from_active_items() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let id = seed_item(&pool, list_id, "Item", 3, 3, None, "2025-01-01T00:00:00Z").await;
        seed_purchase(&pool, id, None, "2025-01-02T00:00:00Z").await;
        let active = sqlx::query_as::<_, WishItem>(
            "SELECT * FROM wish_items WHERE status = 'active' AND list_id = ?",
        )
        .bind(list_id)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert!(!active.iter().any(|i| i.id == id));
    }

    #[tokio::test]
    async fn mark_purchased_null_cost_stored() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let id = seed_item(&pool, list_id, "Item", 3, 3, None, "2025-01-01T00:00:00Z").await;
        seed_purchase(&pool, id, None, "2025-01-02T00:00:00Z").await;
        let cost: Option<f64> =
            sqlx::query_scalar("SELECT actual_cost FROM purchase_records WHERE item_id = ?")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(cost.is_none());
    }

    #[tokio::test]
    async fn mark_purchased_notes_empty_becomes_null() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let id = seed_item(&pool, list_id, "Item", 3, 3, None, "2025-01-01T00:00:00Z").await;
        sqlx::query("UPDATE wish_items SET status = 'purchased' WHERE id = ?")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
        let notes = "";
        sqlx::query(
            "INSERT INTO purchase_records (item_id, actual_cost, purchased_at, notes)
             VALUES (?, NULL, '2025-01-02T00:00:00Z', ?)",
        )
        .bind(id)
        .bind(if notes.is_empty() { None } else { Some(notes) })
        .execute(&pool)
        .await
        .unwrap();
        let stored: Option<String> =
            sqlx::query_scalar("SELECT notes FROM purchase_records WHERE item_id = ?")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(stored.is_none());
    }

    #[tokio::test]
    async fn mark_purchased_notes_preserved() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let id = seed_item(&pool, list_id, "Item", 3, 3, None, "2025-01-01T00:00:00Z").await;
        sqlx::query("UPDATE wish_items SET status = 'purchased' WHERE id = ?")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO purchase_records (item_id, actual_cost, purchased_at, notes)
             VALUES (?, NULL, '2025-01-02T00:00:00Z', 'Got it on sale')",
        )
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
        let stored: Option<String> =
            sqlx::query_scalar("SELECT notes FROM purchase_records WHERE item_id = ?")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(stored.as_deref(), Some("Got it on sale"));
    }

    // ── Archive ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn archive_empty_when_nothing_purchased() {
        let pool = test_pool().await;
        let archive = query_archive(&pool, None).await;
        assert!(archive.is_empty());
    }

    #[tokio::test]
    async fn archive_contains_only_purchased_items() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        seed_item(&pool, list_id, "Active", 3, 3, None, "2025-01-01T00:00:00Z").await;
        let p = seed_item(
            &pool,
            list_id,
            "Purchased",
            3,
            3,
            None,
            "2025-01-02T00:00:00Z",
        )
        .await;
        seed_purchase(&pool, p, Some(10.0), "2025-01-03T00:00:00Z").await;
        let archive = query_archive(&pool, None).await;
        assert_eq!(archive.len(), 1);
        assert_eq!(archive[0].item.id, p);
    }

    #[tokio::test]
    async fn archive_ordered_by_purchased_at_desc() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let first = seed_item(&pool, list_id, "First", 3, 3, None, "2025-01-01T00:00:00Z").await;
        let second = seed_item(&pool, list_id, "Second", 3, 3, None, "2025-01-02T00:00:00Z").await;
        seed_purchase(&pool, first, None, "2025-01-03T00:00:00Z").await;
        seed_purchase(&pool, second, None, "2025-01-04T00:00:00Z").await;
        let archive = query_archive(&pool, None).await;
        assert_eq!(archive[0].item.id, second, "most recently purchased first");
        assert_eq!(archive[1].item.id, first);
    }

    #[tokio::test]
    async fn archive_item_has_correct_fields() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let item_id = seed_item(
            &pool,
            list_id,
            "Gadget",
            4,
            3,
            Some(29.99),
            "2025-01-01T00:00:00Z",
        )
        .await;
        seed_purchase(&pool, item_id, Some(25.0), "2025-01-02T00:00:00Z").await;
        let archive = query_archive(&pool, None).await;
        assert_eq!(archive.len(), 1);
        let a = &archive[0];
        assert_eq!(a.item.name, "Gadget");
        assert!((a.item.estimated_cost.unwrap() - 29.99).abs() < 1e-9);
        assert_eq!(a.record.item_id, item_id);
        assert!((a.record.actual_cost.unwrap() - 25.0).abs() < 1e-9);
        assert_eq!(a.record.purchased_at, "2025-01-02T00:00:00Z");
    }

    #[tokio::test]
    async fn list_archive_isolated_to_list() {
        let pool = test_pool().await;
        let list_a = seed_list(&pool, "A", "2025-01-01T00:00:00Z").await;
        let list_b = seed_list(&pool, "B", "2025-01-02T00:00:00Z").await;
        let item_a = seed_item(&pool, list_a, "A", 3, 3, None, "2025-01-01T00:00:00Z").await;
        let item_b = seed_item(&pool, list_b, "B", 3, 3, None, "2025-01-01T00:00:00Z").await;
        seed_purchase(&pool, item_a, None, "2025-01-03T00:00:00Z").await;
        seed_purchase(&pool, item_b, None, "2025-01-04T00:00:00Z").await;
        let archive_a = query_archive(&pool, Some(list_a)).await;
        assert_eq!(archive_a.len(), 1);
        assert_eq!(archive_a[0].item.id, item_a);
    }

    #[tokio::test]
    async fn list_archive_excludes_other_list_items() {
        let pool = test_pool().await;
        let list_a = seed_list(&pool, "A", "2025-01-01T00:00:00Z").await;
        let list_b = seed_list(&pool, "B", "2025-01-02T00:00:00Z").await;
        let item_b = seed_item(&pool, list_b, "B", 3, 3, None, "2025-01-01T00:00:00Z").await;
        seed_purchase(&pool, item_b, None, "2025-01-03T00:00:00Z").await;
        let archive_a = query_archive(&pool, Some(list_a)).await;
        assert!(archive_a.is_empty());
    }

    // ── Statistics ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn list_stats_all_zeros_empty_list() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "Empty", "2025-01-01T00:00:00Z").await;
        let s = query_list_stats(&pool, list_id).await;
        assert_eq!(s.active_count, 0);
        assert_eq!(s.purchased_count, 0);
        assert!(s.pending_budget.abs() < 1e-9);
        assert!(s.total_spent.abs() < 1e-9);
    }

    #[tokio::test]
    async fn list_stats_pending_budget_sums_active_estimated_costs() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        seed_item(
            &pool,
            list_id,
            "A",
            3,
            3,
            Some(100.0),
            "2025-01-01T00:00:00Z",
        )
        .await;
        seed_item(
            &pool,
            list_id,
            "B",
            3,
            3,
            Some(50.0),
            "2025-01-02T00:00:00Z",
        )
        .await;
        let s = query_list_stats(&pool, list_id).await;
        assert!((s.pending_budget - 150.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn list_stats_total_spent_sums_actual_costs() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let a = seed_item(&pool, list_id, "A", 3, 3, None, "2025-01-01T00:00:00Z").await;
        let b = seed_item(&pool, list_id, "B", 3, 3, None, "2025-01-02T00:00:00Z").await;
        seed_purchase(&pool, a, Some(30.0), "2025-01-03T00:00:00Z").await;
        seed_purchase(&pool, b, Some(20.0), "2025-01-04T00:00:00Z").await;
        let s = query_list_stats(&pool, list_id).await;
        assert!((s.total_spent - 50.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn list_stats_null_costs_treated_as_zero() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        let a = seed_item(&pool, list_id, "A", 3, 3, None, "2025-01-01T00:00:00Z").await;
        seed_purchase(&pool, a, None, "2025-01-02T00:00:00Z").await; // NULL actual_cost
        let s = query_list_stats(&pool, list_id).await;
        assert!(
            s.total_spent.abs() < 1e-9,
            "NULL actual_cost counts as zero"
        );
    }

    #[tokio::test]
    async fn list_stats_counts_are_correct() {
        let pool = test_pool().await;
        let list_id = seed_list(&pool, "L", "2025-01-01T00:00:00Z").await;
        seed_item(&pool, list_id, "A1", 3, 3, None, "2025-01-01T00:00:00Z").await;
        seed_item(&pool, list_id, "A2", 3, 3, None, "2025-01-02T00:00:00Z").await;
        let p = seed_item(&pool, list_id, "P1", 3, 3, None, "2025-01-03T00:00:00Z").await;
        seed_purchase(&pool, p, None, "2025-01-04T00:00:00Z").await;
        let s = query_list_stats(&pool, list_id).await;
        assert_eq!(s.active_count, 2);
        assert_eq!(s.purchased_count, 1);
    }

    #[tokio::test]
    async fn list_stats_isolated_from_other_lists() {
        let pool = test_pool().await;
        let list_a = seed_list(&pool, "A", "2025-01-01T00:00:00Z").await;
        let list_b = seed_list(&pool, "B", "2025-01-02T00:00:00Z").await;
        seed_item(
            &pool,
            list_a,
            "Big",
            3,
            3,
            Some(999.0),
            "2025-01-01T00:00:00Z",
        )
        .await;
        seed_item(
            &pool,
            list_b,
            "Small",
            3,
            3,
            Some(1.0),
            "2025-01-01T00:00:00Z",
        )
        .await;
        let s = query_list_stats(&pool, list_b).await;
        assert!((s.pending_budget - 1.0).abs() < 1e-9);
        assert_eq!(s.active_count, 1);
    }

    #[tokio::test]
    async fn global_stats_all_zeros_empty_db() {
        let pool = test_pool().await;
        // After migration the General list exists but has no items.
        let s = query_global_stats(&pool).await;
        assert_eq!(s.active_count, 0);
        assert_eq!(s.purchased_count, 0);
        assert!(s.pending_budget.abs() < 1e-9);
        assert!(s.total_spent.abs() < 1e-9);
    }

    #[tokio::test]
    async fn global_stats_aggregates_across_all_lists() {
        let pool = test_pool().await;
        let list_a = seed_list(&pool, "A", "2025-01-01T00:00:00Z").await;
        let list_b = seed_list(&pool, "B", "2025-01-02T00:00:00Z").await;
        seed_item(
            &pool,
            list_a,
            "A1",
            3,
            3,
            Some(100.0),
            "2025-01-01T00:00:00Z",
        )
        .await;
        seed_item(
            &pool,
            list_b,
            "B1",
            3,
            3,
            Some(50.0),
            "2025-01-01T00:00:00Z",
        )
        .await;
        let s = query_global_stats(&pool).await;
        assert_eq!(s.active_count, 2);
        assert!((s.pending_budget - 150.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn global_stats_consistency_with_list_stats() {
        let pool = test_pool().await;
        let list_a = seed_list(&pool, "A", "2025-01-01T00:00:00Z").await;
        let list_b = seed_list(&pool, "B", "2025-01-02T00:00:00Z").await;
        let a1 = seed_item(
            &pool,
            list_a,
            "A1",
            3,
            3,
            Some(80.0),
            "2025-01-01T00:00:00Z",
        )
        .await;
        let b1 = seed_item(
            &pool,
            list_b,
            "B1",
            3,
            3,
            Some(40.0),
            "2025-01-01T00:00:00Z",
        )
        .await;
        seed_purchase(&pool, a1, Some(75.0), "2025-01-02T00:00:00Z").await;
        seed_purchase(&pool, b1, Some(35.0), "2025-01-03T00:00:00Z").await;
        let sa = query_list_stats(&pool, list_a).await;
        let sb = query_list_stats(&pool, list_b).await;
        let global = query_global_stats(&pool).await;
        assert!((global.total_spent - (sa.total_spent + sb.total_spent)).abs() < 1e-9);
        assert_eq!(
            global.purchased_count,
            sa.purchased_count + sb.purchased_count
        );
    }
}

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
