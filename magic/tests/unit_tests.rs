use magic_orm::{prelude::*, register_models};
use magic_orm::query::statement::BindArg;
use magic_orm::dialect::{SqlDialect, SqliteDialect, PostgresDialect};
use magic_orm::relations::traits::RelationList;

// ---------------------------------------------------------------------------
// Test model — minimal para pruebas sin DB
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

#[derive(MagicModel, Debug)]
#[magic(table = "reactions")]
pub struct Reaction {
    pub id: i64,
    pub reaction_type: String,
    #[FK(Post)]
    pub post_id: i64,
    #[FK(User)]
    pub user_id: i64,
}

// Modelo sin #[magic(table)] — infiere: PostCategory → post_categories
#[derive(MagicModel, Debug)]
pub struct PostCategory {
    pub id: i64,
    pub name: String,
}

has_many!(User => Post, Reaction);
has_many!(Post => Reaction);

register_models!(User, Post, Reaction, PostCategory);

// =========================================================================
// BindArg
// =========================================================================
#[test]
fn test_bind_arg_from_i64() {
    assert!(matches!(BindArg::from(42i64), BindArg::I64(42)));
}

#[test]
fn test_bind_arg_from_i32() {
    assert!(matches!(BindArg::from(42i32), BindArg::I64(42)));
}

#[test]
fn test_bind_arg_from_f64() {
    let v = 3.14f64;
    let arg = BindArg::from(v);
    assert!(matches!(arg, BindArg::F64(x) if (x - 3.14).abs() < 0.001));
}

#[test]
fn test_bind_arg_from_string() {
    let arg = BindArg::from("hello".to_string());
    assert!(matches!(arg, BindArg::Text(s) if s == "hello"));
}

#[test]
fn test_bind_arg_from_str() {
    let arg = BindArg::from("world");
    assert!(matches!(arg, BindArg::Text(s) if s == "world"));
}

#[test]
fn test_bind_arg_from_bool() {
    assert!(matches!(BindArg::from(true), BindArg::Bool(true)));
    assert!(matches!(BindArg::from(false), BindArg::Bool(false)));
}

#[test]
fn test_bind_arg_from_ref_i64() {
    let v = &99i64;
    let arg = BindArg::from(v);
    assert!(matches!(arg, BindArg::I64(99)));
}

#[test]
fn test_bind_arg_from_ref_string() {
    let s = String::from("ref");
    let arg = BindArg::from(&s);
    assert!(matches!(arg, BindArg::Text(t) if t == "ref"));
}

// =========================================================================
// SqlDialect — placeholders
// =========================================================================
#[test]
fn test_sqlite_placeholder_always_question_mark() {
    assert_eq!(SqliteDialect::placeholder(1), "?");
    assert_eq!(SqliteDialect::placeholder(3), "?");
    assert_eq!(SqliteDialect::placeholder(999), "?");
}

#[test]
fn test_postgres_placeholder_positional() {
    assert_eq!(PostgresDialect::placeholder(1), "$1");
    assert_eq!(PostgresDialect::placeholder(2), "$2");
    assert_eq!(PostgresDialect::placeholder(10), "$10");
}

#[test]
fn test_sqlite_quote_identifier() {
    assert_eq!(SqliteDialect::quote_identifier("users"), "\"users\"");
    assert_eq!(SqliteDialect::quote_identifier("id"), "\"id\"");
}

#[test]
fn test_postgres_quote_identifier_lowercase() {
    assert_eq!(PostgresDialect::quote_identifier("Users"), "\"users\"");
    assert_eq!(PostgresDialect::quote_identifier("ID"), "\"id\"");
}

// =========================================================================
// SQL injection prevention
// =========================================================================
#[test]
fn test_filter_value_not_in_sql() {
    let sql = User::query()
        .filter("name", "=", "' OR 1=1 --")
        .build_sql();

    assert!(!sql.contains("OR 1=1"), "SQL should not contain injected value: {}", sql);
    assert!(sql.contains('?'), "SQL should use placeholder: {}", sql);
}

#[test]
fn test_filter_special_chars_are_parameterized() {
    let sql = User::query()
        .filter("name", "=", "it's a test")
        .build_sql();

    assert_eq!(sql, "SELECT * FROM users WHERE name = ?");
}

// =========================================================================
// QueryBuilder — build_sql output
// =========================================================================
#[test]
fn test_query_all_columns() {
    let sql = User::query().build_sql();
    assert_eq!(sql, "SELECT * FROM users");
}

#[test]
fn test_query_select_columns() {
    let sql = User::query()
        .select(&["name", "age"])
        .build_sql();
    assert_eq!(sql, "SELECT name, age FROM users");
}

#[test]
fn test_query_single_filter() {
    let sql = User::query()
        .filter("name", "=", "Alice")
        .build_sql();
    assert_eq!(sql, "SELECT * FROM users WHERE name = ?");
}

#[test]
fn test_query_multiple_filters() {
    let sql = User::query()
        .filter("name", "=", "Alice")
        .filter("age", ">", 18)
        .build_sql();
    assert_eq!(sql, "SELECT * FROM users WHERE name = ? AND age > ?");
}

#[test]
fn test_query_order_by_asc() {
    let sql = User::query()
        .order_by("name", true)
        .build_sql();
    assert_eq!(sql, "SELECT * FROM users ORDER BY name ASC");
}

#[test]
fn test_query_order_by_desc() {
    let sql = User::query()
        .order_by("age", false)
        .build_sql();
    assert_eq!(sql, "SELECT * FROM users ORDER BY age DESC");
}

#[test]
fn test_query_limit() {
    let sql = User::query()
        .limit(10)
        .build_sql();
    assert_eq!(sql, "SELECT * FROM users LIMIT 10");
}

#[test]
fn test_query_offset() {
    let sql = User::query()
        .limit(10)
        .offset(20)
        .build_sql();
    assert_eq!(sql, "SELECT * FROM users LIMIT 10 OFFSET 20");
}

#[test]
fn test_query_filter_order_limit() {
    let sql = User::query()
        .filter("age", ">=", 21)
        .order_by("name", true)
        .limit(5)
        .build_sql();
    assert_eq!(sql, "SELECT * FROM users WHERE age >= ? ORDER BY name ASC LIMIT 5");
}

// =========================================================================
// BindArg ordering — must match placeholder order
// =========================================================================
#[test]
fn test_query_values_match_filter_order() {
    let qb = User::query()
        .filter("name", "=", "Alice")
        .filter("age", ">", 18);

    let sql = qb.build_sql();
    assert_eq!(sql, "SELECT * FROM users WHERE name = ? AND age > ?");
    
    // Verify values are in the same order as placeholders
    assert_eq!(qb.values.len(), 2);
    assert!(matches!(&qb.values[0], BindArg::Text(s) if s == "Alice"));
    assert!(matches!(&qb.values[1], BindArg::I64(18)));
}

// =========================================================================
// Joins
// =========================================================================
#[test]
fn test_query_join_generates_left_join() {
    let sql = User::query()
        .join::<Post>()
        .build_sql();
    assert!(sql.contains("LEFT JOIN"));
    assert!(sql.contains("posts"));
    assert!(sql.contains("users.id = posts.user_id"));
}

#[test]
fn test_query_with_filter_and_join() {
    let sql = User::query()
        .join::<Post>()
        .filter("users.name", "=", "Alice")
        .build_sql();
    assert!(sql.contains("LEFT JOIN posts ON "));
    assert!(sql.contains("WHERE"));
    assert!(sql.contains("?"));
}

// =========================================================================
// Eager loading — build_sql from EagerQueryBuilder
// =========================================================================
#[test]
fn test_eager_query_build_sql() {
    let sql = User::query()
        .filter("age", ">", 18)
        .build_sql();
    assert_eq!(sql, "SELECT * FROM users WHERE age > ?");
}

#[test]
fn test_eager_query_with_filters() {
    let sql = User::query()
        .filter("name", "=", "Alice")
        .order_by("name", true)
        .limit(5)
        .build_sql();
    assert_eq!(sql, "SELECT * FROM users WHERE name = ? ORDER BY name ASC LIMIT 5");
}

// =========================================================================
// has_many macro — generated method names
// =========================================================================
#[test]
fn test_has_many_generates_methods() {
    use magic_orm::relations::traits::RelationList;
    let rels = <UserRelations as RelationList>::all();
    assert!(rels.contains(&"Post"));
    assert!(rels.contains(&"Reaction"));
}

#[test]
fn test_has_relations_trait() {
    use magic_orm::relations::traits::HasRelations;
    type Relations = <User as HasRelations>::HasMany;
    let rels = <Relations as RelationList>::all();
    assert_eq!(rels.len(), 2);
}

// =========================================================================
// Describe trait
// =========================================================================
#[test]
fn test_describe_trait_implemented() {
    use magic_orm::describe::Describe;
    let desc = <User as Describe>::descriptor();
    assert_eq!(desc.table, "users");
    assert_eq!(desc.columns.len(), 3);
    assert_eq!(desc.foreign_keys.len(), 0);
}

#[test]
fn test_describe_columns() {
    use magic_orm::describe::Describe;
    let desc = <User as Describe>::descriptor();
    let id_col = &desc.columns[0];
    assert_eq!(id_col.name, "id");
    assert!(id_col.primary_key);
    assert!(id_col.auto_increment);
    assert!(!id_col.nullable);

    let name_col = &desc.columns[1];
    assert_eq!(name_col.name, "name");
    assert!(!name_col.primary_key);
    assert!(!name_col.auto_increment);
    assert!(!name_col.nullable); // String is not Option
}

#[test]
fn test_describe_foreign_keys() {
    use magic_orm::describe::Describe;
    let desc = <Post as Describe>::descriptor();
    assert_eq!(desc.foreign_keys.len(), 1);
    let fk = &desc.foreign_keys[0];
    assert_eq!(fk.field, "user_id");
    assert_eq!(fk.related_table, "users");
    assert_eq!(fk.related_column, "id");
}

#[test]
fn test_describe_all_descriptors() {
    let descs = all_descriptors();
    assert_eq!(descs.len(), 4); // User, Post, Reaction, PostCategory
    let tables: Vec<&str> = descs.iter().map(|d| d.table.as_str()).collect();
    assert!(tables.contains(&"users"));
    assert!(tables.contains(&"posts"));
    assert!(tables.contains(&"reactions"));
    assert!(tables.contains(&"post_categories")); // inferred from struct name
}

#[test]
fn test_table_name_inferred() {
    // PostCategory sin #[magic(table)] → debe inferir "post_categories"
    let desc = <PostCategory as magic_orm::describe::Describe>::descriptor();
    assert_eq!(desc.table, "post_categories");
}

#[test]
fn test_describe_to_json() {
    use magic_orm::describe::descriptor_to_json;
    let desc = <User as magic_orm::describe::Describe>::descriptor();
    let json = descriptor_to_json(&desc).unwrap();
    assert!(json.contains("\"table\": \"users\""));
    assert!(json.contains("\"name\": \"id\""));
    assert!(json.contains("\"auto_increment\": true"));
}

// =========================================================================
// QueryBuilder — filter_in, or_filter, count
// =========================================================================
#[test]
fn test_filter_in() {
    let sql = User::query()
        .filter_in("id", [1i64, 2i64, 3i64])
        .build_sql();
    assert_eq!(sql, "SELECT * FROM users WHERE id IN (?, ?, ?)");
}

#[test]
fn test_filter_in_values_order() {
    let qb = User::query()
        .filter_in("id", [10i64, 20i64]);
    assert_eq!(qb.values.len(), 2);
    assert!(matches!(&qb.values[0], BindArg::I64(10)));
    assert!(matches!(&qb.values[1], BindArg::I64(20)));
}

#[test]
fn test_or_filter() {
    let sql = User::query()
        .filter("name", "=", "Alice")
        .or_filter("name", "=", "Bob")
        .build_sql();
    assert_eq!(sql, "SELECT * FROM users WHERE name = ? OR name = ?");
}

#[test]
fn test_and_or_combo() {
    let sql = User::query()
        .filter("age", ">", 18)
        .or_filter("name", "=", "Admin")
        .filter("active", "=", true)
        .build_sql();
    assert_eq!(sql, "SELECT * FROM users WHERE age > ? OR name = ? AND active = ?");
}

#[test]
fn test_count() {
    let sql = User::query()
        .count()
        .filter("age", ">", 18)
        .build_sql();
    assert_eq!(sql, "SELECT count(*) FROM users WHERE age > ?");
}

#[test]
fn test_count_no_filters() {
    let sql = User::query().count().build_sql();
    assert_eq!(sql, "SELECT count(*) FROM users");
}

#[test]
fn test_filter_in_empty() {
    let sql = User::query()
        .filter_in("id", std::iter::empty::<i64>())
        .build_sql();
    assert_eq!(sql, "SELECT * FROM users"); // no filter added
}
