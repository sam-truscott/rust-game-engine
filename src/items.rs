use macroquad::color::Color;
use mlua::Table;

const DEFAULT_COLOR: Color = Color::new(0.7, 0.7, 0.7, 1.0);

/// Something the player can carry. Until there is icon art it is drawn as a square of `color`.
#[derive(Clone, Debug)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub color: Color,
    /// A double click uses it on its own (drinking a beer) instead of picking it up to combine.
    pub self_use: bool,
}

#[derive(Default)]
pub struct Items(Vec<Item>);

impl Items {
    /// Reads `assets/items.lua`: `{ brass_key = { name = "brass key", color = { r, g, b } }, ... }`.
    pub fn from_lua(source: &str, name: &str) -> mlua::Result<Self> {
        let lua = crate::script::data_lua()?;
        let table: Table = lua.load(source).set_name(name).eval()?;

        let mut items = Vec::new();
        for pair in table.pairs::<String, Table>() {
            let (id, item) = pair?;
            let item_name: String = item.get("name")?;
            let color: Option<Table> = item.get("color")?;
            let color = match color {
                Some(rgb) => Color::new(rgb.get(1)?, rgb.get(2)?, rgb.get(3)?, 1.0),
                None => DEFAULT_COLOR,
            };
            items.push(Item {
                id,
                name: item_name,
                color,
                self_use: item.get::<Option<bool>>("self_use")?.unwrap_or(false),
            });
        }
        items.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(Items(items))
    }

    pub fn get(&self, id: &str) -> Option<&Item> {
        self.0.iter().find(|item| item.id == id)
    }

    /// The colour an item is drawn in; items missing from the file are grey.
    pub fn color(&self, id: &str) -> Color {
        self.get(id).map_or(DEFAULT_COLOR, |item| item.color)
    }

    /// The item's name, or the id itself if it is not in the file.
    pub fn name<'a>(&'a self, id: &'a str) -> &'a str {
        self.get(id).map_or(id, |item| item.name.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Item> {
        self.0.iter()
    }
}
