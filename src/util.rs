/// A 2D square list of nodes visualized as such:
/// A₁,₁ A₁,₂ … A₁,ₙ
/// A₂,₁ A₂,₂ … A₂,ₙ
/// ⋮   ⋮     ⋮
/// Aₙ,₁ Aₙ,₂ … Aₙ,ₙ
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

/// A 2D square list of nodes visualized as such:
/// A₁,₁ A₁,₂ … A₁,ₘ
/// A₂,₁ A₂,₂ … A₂,ₘ
/// ⋮   ⋮     ⋮
/// Aₙ,₁ Aₙ,₂ … Aₙ,ₘ
pub struct NodeList<T> {
    items: Vec<T>,
    width: usize,
    _height: usize
}

impl<T> NodeList<T> {
    pub fn new(width: usize, height: usize) -> NodeList<T>
    where
        T: Default + Clone,
    {
        NodeList {
            width,
            _height: height,
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
