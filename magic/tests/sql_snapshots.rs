use magic_orm::{prelude::*, register_models};

// ---------------------------------------------------------------------------
// Test models
// ---------------------------------------------------------------------------
#[derive(MagicModel, Debug)]
#[magic(table = "users")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub age: i32,
}

#[derive(MagicModel, Debug)]
#[magic(table = "posts")]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    #[FK(User)]
    pub user_id: i64,
}

register_models!(User, Post);

// =========================================================================
// QueryBuilder SQL snapshots
// =========================================================================

#[test]
fn test_select_all() {
    insta::assert_snapshot!(User::query().build_sql());
}

#[test]
fn test_select_columns() {
    insta::assert_snapshot!(User::query().select(&["name", "age"]).build_sql());
}

#[test]
fn test_filter_eq() {
    insta::assert_snapshot!(User::query().filter("name", "=", "Alice").build_sql());
}

#[test]
fn test_filter_multiple() {
    insta::assert_snapshot!(
        User::query()
            .filter("age", ">=", 18)
            .filter("name", "LIKE", "%foo%")
            .build_sql()
    );
}

#[test]
fn test_order_by_asc() {
    insta::assert_snapshot!(User::query().order_by("name", true).build_sql());
}

#[test]
fn test_order_by_desc() {
    insta::assert_snapshot!(User::query().order_by("age", false).build_sql());
}

#[test]
fn test_limit() {
    insta::assert_snapshot!(User::query().limit(10).build_sql());
}

#[test]
fn test_offset() {
    insta::assert_snapshot!(User::query().limit(10).offset(20).build_sql());
}

#[test]
fn test_filter_order_limit() {
    insta::assert_snapshot!(
        User::query()
            .filter("age", ">=", 21)
            .order_by("name", true)
            .limit(5)
            .build_sql()
    );
}

#[test]
fn test_join() {
    insta::assert_snapshot!(User::query().join::<Post>().build_sql());
}

#[test]
fn test_join_with_filter() {
    insta::assert_snapshot!(
        User::query()
            .join::<Post>()
            .filter("users.name", "=", "Alice")
            .build_sql()
    );
}

#[test]
fn test_select_columns_with_filter() {
    insta::assert_snapshot!(
        User::query()
            .select(&["id", "name"])
            .filter("age", "<", 30)
            .build_sql()
    );
}

#[test]
fn test_sql_injection_placeholder() {
    // The value should NOT appear in the SQL, only a placeholder
    insta::assert_snapshot!(
        User::query()
            .filter("name", "=", "' OR 1=1 --")
            .build_sql()
    );
}
