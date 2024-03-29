use std::collections::BTreeMap;
use std::fmt;

/// A 2D square list of nodes visualized as such:
/// A₁,₁ A₁,₂ … A₁,ₙ
/// A₂,₁ A₂,₂ … A₂,ₙ
/// ⋮    ⋮      ⋮
/// Aₙ,₁ Aₙ,₂ … Aₙ,ₙ
/// This is a square and not a triangle because trying to create a triangular
/// data structure is hard, and probably not worth it
pub struct EdgeList<T> {
    items: Vec<T>,
    size: usize,
}

impl<T> EdgeList<T> {
    pub fn new(size: usize) -> EdgeList<T>
    where
        T: Default + Clone,
    {
        EdgeList {
            size,
            items: vec![T::default(); size * size],
        }
    }

    pub fn get(&self, i: usize, j: usize) -> &T {
        &self.items[i + j * self.size]
    }

    pub fn get_mut(&mut self, i: usize, j: usize) -> &mut T {
        &mut self.items[i + j * self.size]
    }

    pub fn set(&mut self, i: usize, j: usize, value: T) {
        self.items[i + j * self.size] = value;
    }
}

impl<T> fmt::Display for EdgeList<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for k in 0..self.size {
            for j in 0..self.size {
                if k >= j {
                    write!(f, "{:>5} ", self.get(j, k))?;
                } else {
                    write!(f, "{:>5} ", "")?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

/// A 2D square list of nodes visualized as such:
/// A₁,₁ A₁,₂ … A₁,ₘ
/// A₂,₁ A₂,₂ … A₂,ₘ
/// ⋮   ⋮     ⋮
/// Aₙ,₁ Aₙ,₂ … Aₙ,ₘ
pub struct NodeList<T> {
    items: Vec<T>,
    width: usize,
    height: usize,
}

impl<T> NodeList<T> {
    pub fn new(width: usize, height: usize) -> NodeList<T>
    where
        T: Default + Clone,
    {
        NodeList {
            width,
            height,
            items: vec![T::default(); width * height],
        }
    }

    pub fn get(&self, i: usize, j: usize) -> &T {
        &self.items[i + j * self.width]
    }

    pub fn get_mut(&mut self, i: usize, j: usize) -> &mut T {
        &mut self.items[i + j * self.width]
    }

    pub fn set(&mut self, i: usize, j: usize, value: T) {
        self.items[i + j * self.width] = value;
    }
}

impl<T> fmt::Display for NodeList<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for j in 0..self.width {
            for k in 0..self.height {
                write!(f, "{:>5} ", self.get(j, k))?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

// /// A 2D triangular list of edges visualized as such:
// /// A₁,₁   A₁,₂ … A₁,ₙ-₁ A₁,ₙ
// /// A₂,₁   A₂,₂ … A₂,ₙ-₁
// /// ⋮     ⋮
// /// Aₙ-₁,₁ Aₙ-₁,₂
// /// Aₙ,₁
// pub struct EdgeList2D<T> {
//     items: Vec<T>,
//     size: usize,
// }

// impl<T> EdgeList2D<T> {
//     pub fn new(size: usize) -> EdgeList2D<T>
//     where
//         T: Default + Clone,
//     {
//         let num_items = size * (size + 1) / 2;
//         EdgeList2D {
//             size,
//             items: vec![T::default(); num_items],
//         }
//     }
// }

#[derive(Clone)]
pub struct PrioritySet<T>
where
    T: Clone + PartialOrd + Ord + PartialEq + Eq,
{
    pub elements: BTreeMap<T, u32>,
}

impl<T> PrioritySet<T>
where
    T: Clone + PartialOrd + Ord + PartialEq + Eq,
{
    pub fn insert(&mut self, value: T) {
        let entry = self.elements.entry(value).or_insert(0);
        *entry += 1;
    }

    pub fn insert_with_priority(&mut self, value: T, p: u32) {
        let entry = self.elements.entry(value).or_insert(0);
        *entry = p;
    }

    pub fn pop(&mut self) -> Option<T> {
        // kinda inefficient since it's O(n), but what you gonna do about it
        let index = self
            .elements
            .iter()
            .max_by(|(ak, av), (bk, bv)| av.cmp(bv).then_with(|| ak.cmp(bk)));
        index.map(|i| i.0.clone()).map(|i| {
            self.elements.remove(&i);
            i
        })
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn new() -> PrioritySet<T> {
        PrioritySet {
            elements: BTreeMap::new(),
        }
    }
}

pub fn inc_maybe_print(value: &mut usize, amt: usize, step: usize) {
    if (*value + amt) / step != *value / step {
        println!("{}", *value + amt);
    }
    *value += amt;
}
