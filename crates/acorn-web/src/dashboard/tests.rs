use super::*;

#[test]
fn test_donut_gradient_handles_counts_and_empty_data() {
    let segments = vec![
        ChartSegment {
            label: "first".into(),
            count: 1,
            color: "red".into(),
        },
        ChartSegment {
            label: "second".into(),
            count: 3,
            color: "blue".into(),
        },
    ];
    assert_eq!(donut_gradient(&segments), "red 0.00% 25.00%, blue 25.00% 100.00%");
    assert!(donut_gradient(&[]).is_empty());
}
#[test]
fn test_tile_layout_restores_hidden_tiles_without_changing_order() {
    let ordered = TileLayout::default().sort_by_label(true);
    let hidden = ordered.set_visible("activities", false).set_visible("statuses", false);
    assert_eq!(hidden.hidden.len(), 2);
    assert_eq!(hidden.order, ordered.order);
    let restored = hidden.restore_hidden();
    assert_eq!(restored, ordered);
}
#[test]
fn test_tile_layout_moves_to_visible_target_without_moving_hidden_tiles() {
    let hidden = TileLayout::default().set_visible("organizations", false);
    let moved = hidden.move_to("activities", "statuses");
    assert_eq!(
        moved.order,
        ["contributors", "organizations", "top-topics", "statuses", "activities", "identifiers"]
    );
    assert_eq!(moved.hidden, hidden.hidden);
    assert_eq!(moved.move_to("organizations", "activities"), moved);
    assert_eq!(moved.move_to("activities", "unknown"), moved);
}
#[test]
fn test_tile_layout_normalizes_moves_and_visibility() {
    let default = TileLayout::default();
    assert_eq!(default.order.len(), PORTFOLIO_TILES.len());
    let malformed = TileLayout {
        order: vec!["statuses".into(), "statuses".into(), "unknown".into()],
        hidden: vec!["contributors".into(), "contributors".into(), "unknown".into()],
        positions: BTreeMap::new(),
    };
    let normalized = malformed.normalized();
    assert_eq!(normalized.order.first().map(String::as_str), Some("statuses"));
    assert_eq!(normalized.order.len(), PORTFOLIO_TILES.len());
    assert_eq!(normalized.hidden, ["contributors"]);
    assert_eq!(default.move_by("activities", -1), default);
    assert_eq!(default.move_by("statuses", -1).order.get(3).map(String::as_str), Some("statuses"));
    let hidden = default.set_visible("top-topics", false);
    assert!(!hidden.visible("top-topics"));
    assert_eq!(
        hidden.move_by("statuses", -1).order.get(2..5),
        Some(&["statuses".into(), "top-topics".into(), "contributors".into()][..])
    );
    assert_eq!(hidden.move_by("top-topics", -1), hidden);
    assert!(hidden.set_visible("top-topics", true).visible("top-topics"));
    assert_eq!(default.set_visible("unknown", false), default);
}
#[test]
fn test_tile_layout_restores_order_without_changing_visibility() {
    let hidden = TileLayout::default().set_visible("top-topics", false);
    let restored = hidden.sort_by_label(true).restore_order();
    assert_eq!(restored.order, TileLayout::default().order);
    assert_eq!(restored.hidden, hidden.hidden);
}
#[test]
fn test_tile_layout_sorts_labels_without_changing_visibility() {
    let hidden = TileLayout::default().set_visible("top-topics", false);
    let ascending = hidden.sort_by_label(false);
    assert_eq!(
        ascending.order,
        ["activities", "contributors", "identifiers", "organizations", "statuses", "top-topics"]
    );
    assert_eq!(ascending.hidden, hidden.hidden);
    assert_eq!(hidden.sort_by_label(true).order, ascending.order.into_iter().rev().collect::<Vec<_>>());
}

#[test]
fn test_tile_layout_places_tiles_in_empty_grid_cells() {
    let default = TileLayout::default();
    let target = GridCell { row: 1, column: 4 };
    assert_eq!(default.placement("activities"), Some(GridCell { row: 1, column: 1 }));
    assert!(default.empty_cells().contains(&target));

    let placed = default.place_at("activities", target);
    assert_eq!(placed.placement("activities"), Some(target));
    assert_eq!(placed.placement("organizations"), default.placement("organizations"));
    assert!(!placed.empty_cells().contains(&target));
    assert!(!placed.can_place("statuses", target));
    assert_eq!(placed.place_at("statuses", target), placed);
    assert_eq!(placed.place_at("activities", GridCell { row: 9, column: 1 }), placed);
    assert_eq!(placed.place_at("activities", GridCell { row: 1, column: u8::MAX }), placed);
    assert!(placed.sort_by_label(false).positions.is_empty());
    assert!(placed.restore_order().positions.is_empty());
}
