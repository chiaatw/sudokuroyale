use std::fmt

#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Form {
    #[default]
    Row,
    Column,
    Block,
}

impl Form {
    pub fn iter() -> FormIter {
        FormIter::new()
    }

    pub const fn new(index: u8) -> Self {
        debug_assert!(index <= 2);
        match index {
            0 => Self::Row,
            1 => Self::Column,
            2 => Self::Block,
            _ => unreachable!(),
        }
    }

    pub const fn usize(&self) -> usize {
        *self as usize
    }

    pub const fn label(&self) -> &str {
        match self {
            Form::Row => "Row",
            Form::Column => "Col",
            Form::Block => "Box",
        }
    }

    pub const fn is_row(&self) -> bool {
        matches!(self, Form::Row)
    }

    pub const fn is_column(&self) -> bool {
        matches!(self, Form::Column)
    }

    pub const fn is_block(&self) -> bool {
        matches!(self, Form::Block)
    }
};