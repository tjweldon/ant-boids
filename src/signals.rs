use bevy::prelude::*;
use std::collections::VecDeque;
use std::ops::{Add, Mul};
use std::fmt::Debug;

#[derive(Debug)]
pub struct KindIndex<T: Eq + Clone + Debug>(Vec<T>);

impl<T: Eq + Clone + Debug> KindIndex<T> {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn push(&mut self, kind: &T) {
        self.0.push(kind.clone());
    }

    pub fn scan(&self, kind: &T) -> Option<usize> {
        for (i, candidate) in self.0.iter().enumerate() {
            if *kind == *candidate {
                return Some(i);
            }
        }

        None
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Qualium<T: Eq + Clone + Debug, U: Add<U, Output = U> + Mul<f32, Output = U> + Copy + Clone + Debug> {
    pub amount: U,
    pub kind: T,
}

impl<T: Eq + Clone + Debug, U: Add<U, Output = U> + Mul<f32, Output = U> + Copy + Clone + Debug> Qualium<T, U> {
    fn acc(&mut self, other: Qualium<T, U>) {
        if other.kind != self.kind {
            panic!("Kind mismatch");
        } else {
            self.amount = self.amount + other.amount;
        }
    }
}

#[derive(Debug)]
pub struct Qualia<T: Eq + Clone + Debug, U: Add<U, Output = U> + Mul<f32, Output = U> + Copy + Clone + Debug> {
    index: KindIndex<T>,
    qualia: Vec<Qualium<T, U>>,
}

impl<T: Eq + Clone + Debug, U: Add<U, Output = U> + Mul<f32, Output = U> + Copy + Clone + Debug> Qualia<T, U> {
    pub fn new() -> Self {
        Self { index: KindIndex::new(), qualia: vec![] }
    }

    pub fn include(&mut self, qualium: Qualium<T, U>) {
        if let Some(idx) = self.index.scan(&qualium.kind) {
            self.qualia[idx].acc(qualium);
        } else {
            self.index.push(&qualium.kind);
            self.qualia.push(qualium);
        }
    }

    pub fn get_qualia(&self) -> Vec<Qualium<T, U>> {
        Vec::from_iter(self.qualia.iter().map(|q| q.clone()))
    }
}

#[derive(Debug)]
pub struct Source<T: Eq + Clone + Debug, U: Add<U, Output = U> + Mul<f32, Output = U> + Copy + Clone + Debug> {
    position: Vec2,
    age: f32,
    amount: U,
    kind: T,
}

impl<T: Eq + Clone + Debug, U: Add<U, Output = U> + Mul<f32, Output = U> + Copy + Clone + Debug> Source<T, U> {
    // 1.0/f32::sqrt(PI * 2.0);
    const COEFF: f32 = 0.3989422917366028;

    fn new(position: Vec2, amount: U, kind: T) -> Source<T, U> {
        Self {
            position,
            age: 0f32,
            amount,
            kind,
        }
    }

    fn update(&mut self, dt: f32) {
        self.age = self.age + dt;
    }

    fn sample(&self, sensor_pos: &Vec2) -> Qualium<T, U> {
        let width = self.age/5.0 + 0.5;
        let gaussian = f32::exp(-0.5*(sensor_pos.distance(self.position)/width).powi(2));
        Qualium::<T, U> {
            amount: self.amount * (Self::COEFF/width * gaussian),
            kind: self.kind.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Signals<
    T: Eq + Clone + Debug,
    U: Add<U, Output = U> + Mul<f32, Output = U> + Copy + Clone + Debug
> {
    sources: VecDeque<Source<T, U>>,
    max_age: f32,
}

impl<
    T: Eq + Clone + Debug,
    U: Add<U, Output = U> + Mul<f32, Output = U> + Copy + Clone + Debug
> Signals<T, U> {
    pub fn new() -> Signals<T, U> {
        Self {
            sources: VecDeque::<Source<T, U>>::new(),
            max_age: 120f32,
        }
    }

    pub fn sample(&self, position: Vec2) -> Qualia<T, U> {
        let mut q = Qualia::<T, U>::new();
        for source in self.sources.iter() {
            q.include(source.sample(&position))
        }

        q
    }

    pub fn leave_signal(&mut self, position: Vec2, amount: U, kind: T) {
        self.sources.push_back(Source::new(position, amount, kind));
    }

    pub fn update(&mut self, dt: f32) {
        let mut expiry_horizon = 0;
        for (idx, source) in self.sources.iter_mut().enumerate() {
            source.update(dt);
            if source.age > self.max_age {
                expiry_horizon = idx;
            }
        }

        for _ in 0..expiry_horizon {
            self.sources.pop_front();
        }
    }
}
