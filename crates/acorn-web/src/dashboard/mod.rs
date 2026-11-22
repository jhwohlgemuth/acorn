use alloc::collections::BTreeMap;
use dioxus::prelude::*;

pub(crate) const GRID_COLUMNS: u8 = 4;
const GRID_ROWS: u8 = 8;
pub(crate) const PORTFOLIO_TILES: [(&str, &str); 6] = [
    ("activities", "Activities"),
    ("organizations", "Organizations"),
    ("contributors", "Contributors"),
    ("top-topics", "Top topics"),
    ("statuses", "Statuses"),
    ("identifiers", "Identifiers"),
];
#[derive(Clone, PartialEq)]
pub(crate) struct ChartSegment {
    pub(crate) label: String,
    pub(crate) count: usize,
    pub(crate) color: String,
}
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct TileLayout {
    pub(crate) order: Vec<String>,
    pub(crate) hidden: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) positions: BTreeMap<String, GridCell>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct GridCell {
    pub(crate) row: u8,
    pub(crate) column: u8,
}
impl Default for TileLayout {
    fn default() -> Self {
        Self {
            order: PORTFOLIO_TILES.iter().map(|(id, _)| (*id).into()).collect(),
            hidden: Vec::new(),
            positions: BTreeMap::new(),
        }
    }
}
impl GridCell {
    fn fits(self, id: &str, occupied: &[(Self, u8)]) -> bool {
        let span = tile_span(id);
        let Some(end_column) = self.column.checked_add(span) else {
            return false;
        };
        (1..=GRID_ROWS).contains(&self.row)
            && (1..=GRID_COLUMNS).contains(&self.column)
            && end_column <= GRID_COLUMNS.saturating_add(1)
            && occupied.iter().all(|(cell, width)| {
                cell.row != self.row
                    || end_column <= cell.column
                    || cell.column.checked_add(*width).is_some_and(|occupied_end| occupied_end <= self.column)
            })
    }
}
impl TileLayout {
    pub(crate) fn normalized(&self) -> Self {
        let known = |id: &str| PORTFOLIO_TILES.iter().any(|(tile, _)| *tile == id);
        let mut result = Self {
            order: Vec::new(),
            hidden: Vec::new(),
            positions: BTreeMap::new(),
        };
        for id in self.order.iter().map(String::as_str).chain(PORTFOLIO_TILES.iter().map(|(id, _)| *id)) {
            if known(id) && !result.order.iter().any(|current| current == id) {
                result.order.push(id.into());
            }
        }
        for id in &self.hidden {
            if known(id) && !result.hidden.contains(id) {
                result.hidden.push(id.clone());
            }
        }
        for (id, cell) in &self.positions {
            if known(id) && cell.fits(id, &[]) {
                result.positions.insert(id.clone(), *cell);
            }
        }
        result
    }
    pub(crate) fn restore_order(&self) -> Self {
        Self {
            order: Self::default().order,
            hidden: self.normalized().hidden,
            positions: BTreeMap::new(),
        }
    }
    pub(crate) fn move_by(&self, id: &str, offset: isize) -> Self {
        let mut next = self.normalized();
        let visible = next
            .order
            .iter()
            .enumerate()
            .filter_map(|(index, tile)| next.visible(tile).then_some(index))
            .collect::<Vec<_>>();
        if let Some((position, current)) = visible
            .iter()
            .copied()
            .enumerate()
            .find(|(_, index)| next.order.get(*index).is_some_and(|tile| tile == id))
        {
            if let Some(target) = position
                .checked_add_signed(offset)
                .and_then(|position| visible.get(position))
                .copied()
                .filter(|target| *target != current)
            {
                next.order.swap(current, target);
                next.positions.clear();
            }
        }
        next
    }
    pub(crate) fn move_to(&self, id: &str, target: &str) -> Self {
        let mut next = self.normalized();
        let positions = next
            .order
            .iter()
            .enumerate()
            .filter_map(|(index, tile)| next.visible(tile).then_some(index))
            .collect::<Vec<_>>();
        let mut tiles = next.order.iter().filter(|tile| next.visible(tile)).cloned().collect::<Vec<_>>();
        if let (Some(from), Some(to)) = (tiles.iter().position(|tile| tile == id), tiles.iter().position(|tile| tile == target)) {
            if from == to {
                return next;
            }
            let tile = tiles.remove(from);
            tiles.insert(to, tile);
            for (index, tile) in positions.into_iter().zip(tiles) {
                if let Some(slot) = next.order.get_mut(index) {
                    *slot = tile;
                }
            }
            next.positions.clear();
        }
        next
    }
    pub(crate) fn placement(&self, id: &str) -> Option<GridCell> {
        self.visible_positions().get(id).copied()
    }
    pub(crate) fn place_at(&self, id: &str, cell: GridCell) -> Self {
        let mut next = self.normalized();
        if !next.visible(id) || !PORTFOLIO_TILES.iter().any(|(tile, _)| *tile == id) {
            return next;
        }
        let mut positions = next.visible_positions();
        let occupied = positions
            .iter()
            .filter(|(tile, _)| tile.as_str() != id)
            .map(|(tile, position)| (*position, tile_span(tile)))
            .collect::<Vec<_>>();
        if cell.fits(id, &occupied) {
            positions.insert(id.into(), cell);
            next.positions.retain(|tile, _| next.hidden.contains(tile));
            next.positions.extend(positions);
        }
        next
    }
    pub(crate) fn empty_cells(&self) -> Vec<GridCell> {
        let positions = self.visible_positions();
        let occupied = positions.iter().map(|(tile, cell)| (*cell, tile_span(tile))).collect::<Vec<_>>();
        let rows = positions
            .values()
            .map(|cell| cell.row)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
            .min(GRID_ROWS);
        (1..=rows)
            .flat_map(|row| (1..=GRID_COLUMNS).map(move |column| GridCell { row, column }))
            .filter(|cell| {
                !occupied.iter().any(|(position, span)| {
                    position.row == cell.row
                        && position
                            .column
                            .checked_add(*span)
                            .is_some_and(|end| (position.column..end).contains(&cell.column))
                })
            })
            .collect()
    }
    pub(crate) fn can_place(&self, id: &str, cell: GridCell) -> bool {
        let positions = self.visible_positions();
        let occupied = positions
            .iter()
            .filter(|(tile, _)| tile.as_str() != id)
            .map(|(tile, position)| (*position, tile_span(tile)))
            .collect::<Vec<_>>();
        PORTFOLIO_TILES.iter().any(|(tile, _)| *tile == id) && cell.fits(id, &occupied)
    }
    fn visible_positions(&self) -> BTreeMap<String, GridCell> {
        let mut positions = BTreeMap::new();
        let mut occupied = Vec::new();
        for id in self.order.iter().filter(|id| self.visible(id)) {
            if let Some(cell) = self.positions.get(id).copied().filter(|cell| cell.fits(id, &occupied)) {
                positions.insert(id.clone(), cell);
                occupied.push((cell, tile_span(id)));
            }
        }
        for id in self.order.iter().filter(|id| self.visible(id)) {
            if positions.contains_key(id) {
                continue;
            }
            if let Some(cell) = (1..=GRID_ROWS)
                .flat_map(|row| (1..=GRID_COLUMNS).map(move |column| GridCell { row, column }))
                .find(|cell| cell.fits(id, &occupied))
            {
                positions.insert(id.clone(), cell);
                occupied.push((cell, tile_span(id)));
            }
        }
        positions
    }
    pub(crate) fn visible(&self, id: &str) -> bool {
        !self.hidden.iter().any(|hidden| hidden == id)
    }
    pub(crate) fn restore_hidden(&self) -> Self {
        let mut next = self.normalized();
        next.hidden.clear();
        next
    }
    pub(crate) fn set_visible(&self, id: &str, visible: bool) -> Self {
        let mut next = self.normalized();
        next.hidden.retain(|hidden| hidden != id);
        if !visible && PORTFOLIO_TILES.iter().any(|(tile, _)| *tile == id) {
            next.hidden.push(id.into());
        }
        next
    }
    pub(crate) fn sort_by_label(&self, descending: bool) -> Self {
        let mut next = self.normalized();
        next.positions.clear();
        next.order.sort_by(|left, right| {
            let label = |id: &str| PORTFOLIO_TILES.iter().find(|(tile, _)| *tile == id).map_or("", |(_, name)| *name);
            label(left).to_ascii_lowercase().cmp(&label(right).to_ascii_lowercase())
        });
        if descending {
            next.order.reverse();
        }
        next
    }
}
fn tile_span(id: &str) -> u8 {
    if matches!(id, "activities" | "organizations" | "contributors") {
        1
    } else {
        2
    }
}
#[component]
pub(crate) fn DashboardTile(tile_id: &'static str, class: &'static str, children: Element) -> Element {
    rsx! { div { class: "card dashboard-tile {class}", "data-tile-id": tile_id, {children} } }
}
fn donut_gradient(segments: &[ChartSegment]) -> String {
    let denominator = segments.iter().map(|segment| segment.count).sum::<usize>().max(1) as f64;
    segments
        .iter()
        .scan(0_usize, |offset, segment| {
            let start = *offset as f64 * 100.0 / denominator;
            *offset = offset.saturating_add(segment.count);
            let end = *offset as f64 * 100.0 / denominator;
            Some(format!("{} {start:.2}% {end:.2}%", segment.color))
        })
        .collect::<Vec<_>>()
        .join(", ")
}
#[component]
pub(crate) fn DonutTile(
    tile_id: &'static str,
    class: &'static str,
    title: &'static str,
    center_label: &'static str,
    empty_message: &'static str,
    segments: Vec<ChartSegment>,
    #[props(default)] description: Option<&'static str>,
) -> Element {
    let total = segments.iter().map(|segment| segment.count).sum::<usize>();
    let gradient = donut_gradient(&segments);
    rsx! { DashboardTile { tile_id, class,
        h2 { "{title}" }
        if let Some(description) = description { p { class: "small muted", "{description}" } }
        if total == 0 { p { class: "muted", "{empty_message}" } }
        else { div { class: "donut-content",
            div { class: "donut-chart", style: format!("background:conic-gradient({gradient})"), "aria-hidden": "true",
                div { class: "donut-center", strong { "{total}" } small { "{center_label}" } }
            }
            div { class: "donut-legend", for segment in segments {
                div { class: "donut-legend-row",
                    span { class: "donut-legend-label", span { class: "donut-swatch", style: format!("background:{}", segment.color), "aria-hidden": "true" } "{segment.label}" }
                    strong { "{segment.count}" }
                }
            } }
        } }
    } }
}
#[component]
pub(crate) fn MetricTile(tile_id: &'static str, label: &'static str, count: usize) -> Element {
    rsx! { DashboardTile { tile_id, class: "metric", strong { "{count}" } small { "{label}" } } }
}

#[cfg(test)]
mod tests;
