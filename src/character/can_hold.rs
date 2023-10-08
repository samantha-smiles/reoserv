use std::cmp;

use eo::data::{EOInt, EOShort};

use crate::ITEM_DB;

use super::Character;

impl Character {
    pub fn can_hold(&self, item_id: EOShort, max_amount: EOInt) -> EOInt {
        if self.weight > self.max_weight {
            return 0;
        }

        let item = ITEM_DB.items.get(item_id as usize - 1);

        if item.is_none() {
            return max_amount;
        }

        let item = item.unwrap();

        if item.weight == 0 {
            return max_amount;
        }

        let remaining_weight = self.max_weight - self.weight;
        let max_items = (remaining_weight as f64 / item.weight as f64).floor();
        cmp::min(max_items as EOInt, max_amount)
    }
}