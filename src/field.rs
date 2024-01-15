use bevy::{prelude::*, reflect::Enum};
use std::fmt::Debug;

use crate::ant::SignalKind;

#[derive(Resource)]
pub struct Signals {
    pub exploring: Vec2Field<SignalKind>,
    pub retrieving: Vec2Field<SignalKind>,
}

impl Signals {
    pub fn get_field(&self, kind: SignalKind) -> &Vec2Field<SignalKind> {
        match kind {
            SignalKind::Exploring => &self.exploring,
            SignalKind::Retrieving => &self.retrieving,
        }
    }

    pub fn get_mut_field(&mut self, kind: SignalKind) -> &mut Vec2Field<SignalKind> {
        match kind {
            SignalKind::Exploring => &mut self.exploring,
            SignalKind::Retrieving => &mut self.retrieving,
        }
    }

    pub fn update(&mut self, &dt: &f32) {
        self.exploring.update(0.01, 0.03, &dt);
        self.retrieving.update(0.01, 0.03, &dt);
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum FoodType {
    Yummy
}

#[derive(Resource)]
pub struct Food {
    pub amount: Field<FoodType>,
}

impl Food {
    pub fn new(lattice: Vec2, size: Vec2) -> Self {
        Self {
            amount: Field::<FoodType>::new(
                FoodType::Yummy,
                lattice,
                size
            ),
        }
    }

    pub fn get_cells(&mut self) -> &[Cell] {
        self.amount.get_cells()
    }

    pub fn put(&mut self, area: Rect, depth: f32) {
        let area = Rect::from_center_size(Vec2::ZERO, self.amount.size).intersect(area);
        let along = self.amount.lattice.project_onto(Vec2::X);
        let up = self.amount.lattice.project_onto(Vec2::Y);
        let region_size = area.width() * area.height();
        
        let cell_size = self.amount.lattice.x * self.amount.lattice.y;
        let cell_count = region_size as f32 / cell_size as f32;

        let walk_start = 0.* along + 0.* up + area.min + 0.5 * self.amount.lattice;
        let mut cursor = walk_start.clone();
        for _ in 0..cell_count as usize {
            self.amount.set_cell_value(depth, cursor);

            // move along until out of area, then start again but above, break if that's outside
            match area.contains(cursor + along) {
                true => {
                    cursor = cursor + along
                },
                false => {
                    cursor = walk_start.project_onto(Vec2::X) + (cursor + up).project_onto(Vec2::Y)
                }
            };

            if !area.contains(cursor) {
                break;
            }
        }
    }

    pub fn update(&mut self, &dt: &f32) {
        self.amount.update(0.0001, 0., &dt);
    }
}

#[derive(Copy, Clone, Debug, Component)]
pub struct Cell {
    pub region: Rect,
    pub val: f32,
}

impl Cell {
    pub fn read_from(&mut self, field: & impl Cellular<f32>) {
        self.val = field.get_cell_value(self.region.center());
    }
}

#[derive(Copy, Clone, Debug, Component)]
pub struct Vec2Cell {
    pub region: Rect,
    pub val: Vec2,
}

impl Vec2Cell {
    pub fn read_from(&mut self, field: & impl Cellular<Vec2>) {
        self.val = field.get_cell_value(self.region.center());
    }
}

pub trait Cellular<C> {
    fn get_cell_value(&self, pos: Vec2) -> C;
}

#[derive(Copy, Clone)]
pub struct LatticeIndexer(usize, usize);

impl LatticeIndexer {
    pub fn linear_max(&self) -> usize {
        self.0 * self.1
    }

    pub fn to_linear(&self, grid_idx: (usize, usize)) -> Option<usize> {
        match (grid_idx.0 < self.0, grid_idx.1 < self.1) {
            (true, true) => Some(grid_idx.0 + grid_idx.1 * self.0),
            _ => None,
        }
    }

    pub fn to_grid(&self, linear_idx: usize) -> Option<(usize, usize)> {
        if linear_idx < self.linear_max() {
            return Some((linear_idx % self.0, linear_idx / self.0));
        }

        return None;
    }

    pub fn w(&self) -> usize {
        self.0
    }

    pub fn h(&self) -> usize {
        self.1
    }
}

#[derive(Resource)]
pub struct Vec2Field<T: Copy + Clone + Debug> {
    pub kind: T,
    x: Field<T>,
    y: Field<T>,
    cell_cache: Vec<Vec2Cell>,
}

impl<T: Copy + Clone + Debug> Vec2Field<T> {
    pub fn new(kind: T, lattice: Vec2, size: Vec2) -> Self {
        Self {
            kind,
            x: Field::new(kind, lattice.clone(), size.clone()),
            y: Field::new(kind, lattice, size),
            cell_cache: vec![],
        }
    }

    pub fn get_lattice(&self) -> Vec2 {
        return self.x.lattice;
    }

    pub fn get_dimensions(&self) -> LatticeIndexer {
        return self.x.dimensions;
    }

    pub fn fill_with(&mut self, generator: &mut dyn FnMut() -> f32) {
        self.x.fill_with(generator);
        self.y.fill_with(generator);
    }

    pub fn set_cell_value(&mut self, value: Vec2, pos: Vec2) {
        self.x.set_cell_value(value.x, pos.clone());
        self.y.set_cell_value(value.y, pos);
    }

    pub fn acc_cell_value(&mut self, value: Vec2, pos: Vec2) {
        self.x.acc_cell_value(value.x, pos.clone());
        self.y.acc_cell_value(value.y, pos);
    }

    pub fn set_cell_value_at_lattice_idx(&mut self, value: Vec2, lattice_idx: (usize, usize)) {
        self.x.set_cell_value_at_lattice_idx(value.x, lattice_idx);
        self.y.set_cell_value_at_lattice_idx(value.y, lattice_idx);
    }

    pub fn update(&mut self, diffusion_rate: f32, evapouration_rate: f32, &dt: &f32) {
        self.x.update(diffusion_rate, evapouration_rate, &dt);
        self.y.update(diffusion_rate, evapouration_rate, &dt);
    }


    pub fn sample(&self, pos: Vec2) -> Vec2 {
        self.get_cell_value(pos)
    }

    pub fn get_cells(&mut self) -> &[Vec2Cell] {
        if self.cell_cache.len() > 0 {
            return &self.cell_cache;
        }
        let x_cells = self.x.get_cells();
        let y_cells = self.y.get_cells();

        self.cell_cache = x_cells
            .iter()
            .zip(y_cells)
            .map(|(xc, yc)| Vec2Cell {
                region: xc.region,
                val: Vec2::new(xc.val, yc.val),
            })
            .collect();
        return &self.cell_cache;
    }
}

impl<T: Copy + Clone + Debug> Cellular<Vec2> for Vec2Field<T> {
    fn get_cell_value(&self, pos: Vec2) -> Vec2 {
        Vec2::new(
            self.x.get_cell_value(pos.clone()),
            self.y.get_cell_value(pos),
        )
    }
}

#[derive(Resource)]
pub struct Field<T: Copy + Clone + Debug> {
    pub kind: T,
    pub lattice: Vec2,
    pub size: Vec2,
    pub dimensions: LatticeIndexer,
    cells: Vec<f32>,
    cell_cache: Vec<Cell>,
}

impl<T: Copy + Clone + Debug> Field<T> {
    pub fn new(kind: T, lattice: Vec2, size: Vec2) -> Self {
        if lattice.x == 0f32 || lattice.y == 0f32 {
            panic!("Only nonzero x and y allowed for lattice vector. Got {lattice:?}");
        }
        let cells_x: usize = (size.x / lattice.x) as usize;
        let cells_y: usize = (size.y / lattice.y) as usize;
        let dimensions = LatticeIndexer(cells_x, cells_y);
        let cell_cache: Vec<Cell> = vec![];
        Self {
            kind,
            lattice,
            size,
            dimensions,
            cells: vec![0f32; cells_y * cells_x],
            cell_cache,
        }
    }

    fn value_lookup(&self, x_idx: usize, y_idx: usize) -> Option<f32> {
        match self.dimensions.to_linear((x_idx, y_idx)) {
            Some(idx) => Some(self.cells[idx]),
            _ => None,
        }
    }

    fn wrapped_value_lookup(&self, x_idx: usize, y_idx: usize) -> f32 {
        let Some(val) = self.value_lookup(x_idx % self.dimensions.w(), y_idx % self.dimensions.h())
        else {
            panic!("Could not lookup value at ({x_idx}, {y_idx}).")
        };
        val
    }

    fn lattice_idx_to_pos(&self, x_idx: usize, y_idx: usize) -> Vec2 {
        let pos_offset: Vec2 =
            Mat2::from_diagonal(Vec2::new(x_idx as f32, y_idx as f32)).mul_vec2(self.lattice);
        pos_offset - 0.5 * self.size
    }

    fn pos_to_lattice_idx(&self, pos: Vec2) -> (usize, usize) {
        let scale_transform: Mat2 = Mat2::from_diagonal(self.lattice).inverse();
        let cell_pos: IVec2 = scale_transform
            .mul_vec2((0.5 * self.size + pos).max(Vec2::ZERO).min(self.size))
            .as_ivec2();

        (cell_pos.x as usize, cell_pos.y as usize)
    }


    pub fn get_cells(&mut self) -> &[Cell] {
        if self.cell_cache.len() != self.cells.len() {
            self.cell_cache = self
                .cells
                .iter()
                .enumerate()
                .map(|(cell_idx, v)| (self.dimensions.to_grid(cell_idx).unwrap_or((0, 0)), v))
                .map(|((x_idx, y_idx), &v)| Cell {
                    region: Rect::from_center_size(
                        self.lattice_idx_to_pos(x_idx, y_idx),
                        self.lattice,
                    ),
                    val: v,
                })
                .collect();
        }
        return &self.cell_cache;
    }

    fn cache_dirty(&self) -> bool {
        self.cell_cache.len() > 0
    }

    fn reset_cache(&mut self) {
        if !self.cache_dirty() {
            return;
        }
        self.cell_cache.truncate(0);
    }

    pub fn set_cell_value_at_lattice_idx(&mut self, value: f32, lattice_idx: (usize, usize)) {
        self.reset_cache();
        match self.dimensions.to_linear(lattice_idx) {
            Some(idx) => {
                self.cells[idx] = value;
            }
            None => (),
        }
    }

    pub fn set_cell_value(&mut self, value: f32, pos: Vec2) {
        let lattice_idx = self.pos_to_lattice_idx(pos);
        self.set_cell_value_at_lattice_idx(value, lattice_idx);
    }

    pub fn acc_cell_value(&mut self, value: f32, pos: Vec2) {
        let lattice_idx = self.pos_to_lattice_idx(pos.clone());
        let current = self.get_cell_value(pos);
        self.set_cell_value_at_lattice_idx(current + value, lattice_idx);
    }

    pub fn fill_with(&mut self, generator: &mut dyn FnMut() -> f32) {
        self.reset_cache();
        for i in 0..self.dimensions.linear_max() {
            self.cells[i] = generator();
        }
    }

    pub fn update(&mut self, diffusion_rate: f32, evapouration_rate: f32, &dt: &f32) {
        #[allow(non_snake_case)]
        let (A, B): (f32, f32) = (1., 0.5 * std::f32::consts::FRAC_1_SQRT_2);

        let mut new_cells = vec![0f32; self.cells.len()];
        for (x, y) in (0..self.cells.len()).map(|i| self.dimensions.to_grid(i).unwrap_or((0, 0))) {
            let lx = match x == 0 {
                true => self.dimensions.w() - 1,
                false => x - 1,
            };

            let dy = match y == 0 {
                true => self.dimensions.h() - 1,
                false => y - 1,
            };

            let (l, b, r, t, bl, br, tl, tr) = (
                self.wrapped_value_lookup(lx, y),
                self.wrapped_value_lookup(x, dy),
                self.wrapped_value_lookup(x + 1, y),
                self.wrapped_value_lookup(x, y + 1),
                self.wrapped_value_lookup(lx, dy),
                self.wrapped_value_lookup(x + 1, dy),
                self.wrapped_value_lookup(lx, y + 1),
                self.wrapped_value_lookup(x + 1, y + 1),
            );

            let neighbour_avg =
                0.25 * (A / (A + B) * (l + b + r + t) + B / (A + B) * (bl + br + tl + tr));

            let current = self.wrapped_value_lookup(x, y);
            let change = diffusion_rate * dt * (neighbour_avg - current);
            let Some(linear) = self.dimensions.to_linear((x, y)) else {
                continue;
            };
            new_cells[linear] = (current + change) * (1. - evapouration_rate).powf(dt);
        }

        self.cells = new_cells;
        self.reset_cache();
    }
}

impl<T: Copy + Clone + Debug> Cellular<f32> for Field<T> {
    fn get_cell_value(&self, at_pos: Vec2) -> f32 {
        let (x_idx, y_idx) = self.pos_to_lattice_idx(at_pos);
        match self.value_lookup(x_idx, y_idx) {
            Some(val) => val,
            None => 0f32,
        }
    }
}
