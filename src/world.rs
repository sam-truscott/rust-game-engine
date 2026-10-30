use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Default)]
struct Data {
    inventory: Vec<String>,
    hidden: HashSet<String>,
}

/// What scripts change: what the player carries and which things in the room are gone.
/// Clones share the same state, so the script runner and the game both see it.
#[derive(Clone, Default)]
pub struct World(Rc<RefCell<Data>>);

impl World {
    /// Adds an item to the inventory; carrying it already does nothing.
    pub fn give(&self, item: &str) {
        let mut data = self.0.borrow_mut();
        if !data.inventory.iter().any(|held| held == item) {
            data.inventory.push(item.to_owned());
        }
    }

    /// Removes an item from the inventory; false if it was not carried.
    pub fn take(&self, item: &str) -> bool {
        let mut data = self.0.borrow_mut();
        let before = data.inventory.len();
        data.inventory.retain(|held| held != item);
        data.inventory.len() != before
    }

    pub fn has(&self, item: &str) -> bool {
        self.0.borrow().inventory.iter().any(|held| held == item)
    }

    /// The carried items, in the order they were picked up.
    pub fn items(&self) -> Vec<String> {
        self.0.borrow().inventory.clone()
    }

    /// Hides the prop and the hotspot with this id.
    pub fn hide(&self, id: &str) {
        self.0.borrow_mut().hidden.insert(id.to_owned());
    }

    pub fn is_hidden(&self, id: &str) -> bool {
        self.0.borrow().hidden.contains(id)
    }
}
