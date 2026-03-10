//! Reusable database macros.

/// Builds a dynamic SQL query with optional WHERE clause and ORDER BY from a row struct.
///
/// This macro generates a SQL query string and parameter list based on which fields in the
/// provided row are `Some`. Fields with `None` values are excluded from the WHERE clause.
///
/// # Parameters
///
/// - `$row` - The row struct instance containing optional field values
/// - `$base` - The base SELECT query string (e.g., `"SELECT * FROM table"`)
/// - `$order_by` - Optional ORDER BY clause (typically `Self::ORDER_BY`)
/// - Field list with optional mapping expressions for type conversions
///
/// # Returns
///
/// A tuple of `(String, Vec<Box<dyn rusqlite::ToSql>>)` containing:
/// - The complete SQL query string with WHERE and ORDER BY clauses
/// - A vector of bound parameters for the query
///
/// # Examples
///
/// ```ignore
/// // Simple field filtering
/// let row = ActivityRow {
///     id: Some(42),
///     command: Some("check".to_string()),
///     user_path: None,  // This field will be excluded from WHERE clause
///     ..Default::default()
/// };
/// let query = "SELECT id, command, user_path FROM activity";
/// let (sql, params) = build_query!(row, query, Self::ORDER_BY, [
///     id,
///     command,
///     user_path,
/// ]);
/// // Results in: "SELECT ... FROM activity WHERE id = ? AND command = ? ORDER BY ..."
///
/// // With type mapping for boolean fields
/// let (sql, params) = build_query!(row, query, Self::ORDER_BY, [
///     id,
///     success => |value| value as i32,  // Convert bool to i32 for SQLite
/// ]);
/// ```
macro_rules! build_query {
    ($row:expr, $base:expr, $order_by:expr, [$( $field:ident $(=> $map:expr)? ),* $(,)?]) => {{
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        $(
            if let Some(value) = build_query!(@value $row.$field.clone() $(, $map)?) {
                conditions.push(concat!(stringify!($field), " = ?"));
                params.push(Box::new(value));
            }
        )*
        if conditions.is_empty() {
            match $order_by {
                | Some(order_by) => (format!("{} ORDER BY {}", $base, order_by), vec![]),
                | None => ($base.to_string(), vec![]),
            }
        } else {
            let where_clause = conditions.join(" AND ");
            match $order_by {
                | Some(order_by) => (format!("{} WHERE {} ORDER BY {}", $base, where_clause, order_by), params),
                | None => (format!("{} WHERE {}", $base, where_clause), params),
            }
        }
    }};
    (@value $value:expr) => {
        $value
    };
    (@value $value:expr, $map:expr) => {
        $value.map($map)
    };
}

pub(crate) use build_query;
