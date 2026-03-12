use core::ops::Index;

pub struct BooleanArray<'a> {
    data: &'a mut [u8],
}

impl<'a> BooleanArray<'a> {
    pub fn new(data: &'a mut [u8]) -> Self {
        BooleanArray { data }
    }

    pub fn set_all(&mut self) {
        for i in 0..self.data.len() {
            self.data[i] = 0b11111111;
        }
    }

    pub fn get(&self, index: usize) -> bool {
        assert!(index < self.data.len() * 8, "Index {index} out of bounds");

        let byte_index = index / 8;
        let bit_index = index % 8;
        (self.data[byte_index] >> bit_index) & 1 == 1
    }

    pub fn set(&mut self, index: usize, value: bool) {
        assert!(index < self.len(), "Index {index} out of bounds");

        let byte_index = index / 8;
        let bit_index = index % 8;
        if value {
            self.data[byte_index] |= 1 << bit_index;
        } else {
            self.data[byte_index] &= !(1 << bit_index);
        }
    }

    pub fn len(&self) -> usize {
        self.data.len() * 8
    }
}

impl<'a> Index<usize> for BooleanArray<'a> {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        if self.get(index) { &true } else { &false }
    }
}
