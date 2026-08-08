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
/// A tuple of `(String, Vec<Box<dyn ToSql>>)` containing:
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
///
/// ## Note on Self::FIELDS
///
/// The `build_query!` macro does not directly support `Self::FIELDS` because macros operate
/// at compile time while `Self::FIELDS` is a runtime value. You must explicitly list the
/// fields you want to include in the query.
///
/// If you want to use all fields from your struct, you can:
/// 1. Use the "expand all fields" feature of your IDE/editor to auto-complete the field list
/// 2. Manually list all the fields as shown in the examples above
macro_rules! build_query {
    ($row:expr, $base:expr, $order_by:expr, [$( $field:ident $(=> $map:expr)? ),* $(,)?]) => {{
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn crate::io::database::backend::ToSql>> = Vec::new();
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
/// Defines helper functions that convert one or more `DatabaseResult<T>` values
/// into a single tupled `DatabaseResult` while preserving first-error behavior.
macro_rules! define_required_fn {
    (1, $fn_name:ident, $A:ident, $a:ident) => {
        fn $fn_name<$A>($a: Option<$A>) -> crate::io::ApiResult<($A,)> {
            match $a.ok_or_else(|| color_eyre::eyre::eyre!("Failed to insert: {} is required", stringify!($a))) {
                | Ok($a) => Ok(($a,)),
                | Err(why) => Err(why),
            }
        }
    };
    (3, $fn_name:ident, $A:ident, $a:ident, $B:ident, $b:ident, $C:ident, $c:ident) => {
        fn $fn_name<$A, $B, $C>($a: Option<$A>, $b: Option<$B>, $c: Option<$C>) -> crate::io::ApiResult<($A, $B, $C)> {
            match (
                $a.ok_or_else(|| color_eyre::eyre::eyre!("Failed to insert: {} is required", stringify!($a))),
                $b.ok_or_else(|| color_eyre::eyre::eyre!("Failed to insert: {} is required", stringify!($b))),
                $c.ok_or_else(|| color_eyre::eyre::eyre!("Failed to insert: {} is required", stringify!($c))),
            ) {
                | (Ok($a), Ok($b), Ok($c)) => Ok(($a, $b, $c)),
                | (Err(why), _, _) => Err(why),
                | (_, Err(why), _) => Err(why),
                | (_, _, Err(why)) => Err(why),
            }
        }
    };
    (4, $fn_name:ident, $A:ident, $a:ident, $B:ident, $b:ident, $C:ident, $c:ident, $D:ident, $d:ident) => {
        fn $fn_name<$A, $B, $C, $D>($a: Option<$A>, $b: Option<$B>, $c: Option<$C>, $d: Option<$D>) -> crate::io::ApiResult<($A, $B, $C, $D)> {
            match (
                $a.ok_or_else(|| color_eyre::eyre::eyre!("Failed to insert: {} is required", stringify!($a))),
                $b.ok_or_else(|| color_eyre::eyre::eyre!("Failed to insert: {} is required", stringify!($b))),
                $c.ok_or_else(|| color_eyre::eyre::eyre!("Failed to insert: {} is required", stringify!($c))),
                $d.ok_or_else(|| color_eyre::eyre::eyre!("Failed to insert: {} is required", stringify!($d))),
            ) {
                | (Ok($a), Ok($b), Ok($c), Ok($d)) => Ok(($a, $b, $c, $d)),
                | (Err(why), _, _, _) => Err(why),
                | (_, Err(why), _, _) => Err(why),
                | (_, _, Err(why), _) => Err(why),
                | (_, _, _, Err(why)) => Err(why),
            }
        }
    };
    (5, $fn_name:ident, $A:ident, $a:ident, $B:ident, $b:ident, $C:ident, $c:ident, $D:ident, $d:ident, $E:ident, $e:ident) => {
        fn $fn_name<$A, $B, $C, $D, $E>(
            $a: Option<$A>,
            $b: Option<$B>,
            $c: Option<$C>,
            $d: Option<$D>,
            $e: Option<$E>,
        ) -> crate::io::ApiResult<($A, $B, $C, $D, $E)> {
            match (
                $a.ok_or_else(|| color_eyre::eyre::eyre!("Failed to insert: {} is required", stringify!($a))),
                $b.ok_or_else(|| color_eyre::eyre::eyre!("Failed to insert: {} is required", stringify!($b))),
                $c.ok_or_else(|| color_eyre::eyre::eyre!("Failed to insert: {} is required", stringify!($c))),
                $d.ok_or_else(|| color_eyre::eyre::eyre!("Failed to insert: {} is required", stringify!($d))),
                $e.ok_or_else(|| color_eyre::eyre::eyre!("Failed to insert: {} is required", stringify!($e))),
            ) {
                | (Ok($a), Ok($b), Ok($c), Ok($d), Ok($e)) => Ok(($a, $b, $c, $d, $e)),
                | (Err(why), _, _, _, _) => Err(why),
                | (_, Err(why), _, _, _) => Err(why),
                | (_, _, Err(why), _, _) => Err(why),
                | (_, _, _, Err(why), _) => Err(why),
                | (_, _, _, _, Err(why)) => Err(why),
            }
        }
    };
}

pub(crate) use build_query;
pub(crate) use define_required_fn;

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )]
    define_required_fn!(1, required1_test, A, a);
    define_required_fn!(3, required3_test, A, a, B, b, C, c);
    define_required_fn!(4, required4_test, A, a, B, b, C, c, D, d);

    #[test]
    fn required1_returns_value_tuple() {
        let result = required1_test::<i32>(Some(7));
        assert_eq!(result.ok(), Some((7,)));
    }
    #[test]
    fn required1_returns_error() {
        let result = required1_test::<i32>(None);
        let message = result.err().map(|why| why.to_string());
        assert_eq!(message.as_deref(), Some("Failed to insert: a is required"));
    }
    #[test]
    fn required3_returns_value_tuple() {
        let result = required3_test::<i32, i32, i32>(Some(1), Some(2), Some(3));
        assert_eq!(result.ok(), Some((1, 2, 3)));
    }
    #[test]
    fn required3_returns_first_error() {
        let result = required3_test::<i32, i32, i32>(None, None, None);
        let message = result.err().map(|why| why.to_string());
        assert_eq!(message.as_deref(), Some("Failed to insert: a is required"));
    }
    #[test]
    fn required4_returns_value_tuple() {
        let result = required4_test::<i32, i32, i32, i32>(Some(1), Some(2), Some(3), Some(4));
        assert_eq!(result.ok(), Some((1, 2, 3, 4)));
    }
    #[test]
    fn required4_returns_first_error() {
        let result = required4_test::<i32, i32, i32, i32>(Some(1), None, None, None);
        let message = result.err().map(|why| why.to_string());
        assert_eq!(message.as_deref(), Some("Failed to insert: b is required"));
    }
}
