use bevy::prelude::*;

struct Walls {
    areas: Vec<Rect>,
}

impl Walls {
    pub fn contains(&self, point: Vec2) -> bool {
        let mut result = false;
        for region in self.areas.iter() {
            result |= region.contains(point);
        }

        return result;
    }
}


struct Food {
    position: Vec2,
}
