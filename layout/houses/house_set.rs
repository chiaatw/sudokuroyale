pub trait HouseSetLike: Copy + Sized {

    const fn empty(shape: Shape) -> Self;
    const fn full(shape: Shape) -> Self;
    const fn from_bits(shape: Shape, bits: u16) -> Self;
    const fn from_coords(shape: Shape, coords: i32) -> Self;

    fn shape(&self) -> Shape;
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;

    fn has_coord(&self, coord: Coord) -> bool;
    fn union(self, other: Self) -> Self;
    fn intersect(self, other: Self) -> Self;
    fn minus(self, other: Self) -> Self;
}