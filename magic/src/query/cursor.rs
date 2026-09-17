use crate::query::statement::BindArg;

// ---------------------------------------------------------------------------
// Cursor — represents a keyset position for cursor-based pagination.
//
// A cursor captures the values of the ORDER BY columns at a specific row,
// allowing efficient "give me the next page" queries without OFFSET.
//
// Usage:
//   let cursor = Cursor::new(vec![BindArg::Text("2026-09-08T12:00:00".into()), BindArg::I64(42)]);
//   Observation::query()
//       .filter("session_id", "=", 1)
//       .order_by("timestamp", true)
//       .order_by("id", true)
//       .after(cursor)
//       .limit(1000)
//       .fetch_all(&db)
//       .await?;
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Cursor {
    values: Vec<BindArg>,
}

impl Cursor {
    /// Create a new cursor from values.
    /// Values must match the ORDER BY columns in order.
    pub fn new(values: Vec<BindArg>) -> Self {
        Self { values }
    }

    /// Get the cursor values.
    pub fn values(&self) -> &[BindArg] {
        &self.values
    }

    /// Get the number of columns in the cursor.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if cursor is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

// ---------------------------------------------------------------------------
// CursorFromRow — trait to extract cursor values from a fetched row.
//
// This is used to automatically create cursors from query results.
// ---------------------------------------------------------------------------

pub trait CursorFromRow {
    /// Extract cursor values from this row based on the order_by columns.
    fn cursor_values(&self) -> Vec<BindArg>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_new() {
        let cursor = Cursor::new(vec![
            BindArg::Text("2026-09-08T12:00:00".into()),
            BindArg::I64(42),
        ]);
        assert_eq!(cursor.len(), 2);
        assert!(!cursor.is_empty());
    }

    #[test]
    fn test_cursor_empty() {
        let cursor = Cursor::new(vec![]);
        assert!(cursor.is_empty());
    }
}
