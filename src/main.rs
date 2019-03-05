#![feature(range_contains)]

use std::cmp::max;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt;
use std::ops::Range;

#[derive(Copy, Clone)]
struct RegionDesc {
    base: usize,
    size: usize,
}

impl RegionDesc {
    fn missing() -> Self {
        RegionDesc {
            base: 0,
            size: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.size == 0
    }

    fn end(&self) -> usize {
        if self.size == 0 { 0 } else { self.base + self.size - 1 }
    }

    fn range(&self) -> Range<usize> {
        (self.base..self.base + self.size)
    }
}

const MAX_REGION_DESC: usize = 5;

struct Memblock {
    descriptors: [RegionDesc; MAX_REGION_DESC]
}

impl Memblock {
    fn new() -> Self {
        assert!(MAX_REGION_DESC > 0);
        Memblock { descriptors: [RegionDesc::missing(); MAX_REGION_DESC] }
    }

    fn merge(&mut self, idx: usize) {
        let i = if idx == 0 { 0 } else { idx - 1 };

        loop {
            let subject = self.descriptors[i + 1].clone();
            if subject.is_empty() {
                return;
            }
            let mut object = &mut self.descriptors[i];
            assert!(object.base <= subject.base);

            if object.end() + 1 < subject.base {
                return;  // [0~0xFF] [0x100~0x1FF] can be merged into [0~0x1FF]
            }
            object.size = max(subject.end() + 1 - object.base, object.size);

            self.shift_left(i + 1);
        }
    }

    fn shift_left(&mut self, idx: usize) {
        for i in (idx + 1)..self.descriptors.len() {
            self.descriptors[i - 1] = self.descriptors[i]
        }
        *self.descriptors.last_mut().unwrap() = RegionDesc::missing();
    }

    fn shift_right(&mut self, idx: usize) {
        let mut i = self.descriptors.len() - 1;
        while i > idx {
            self.descriptors[i] = self.descriptors[i - 1];
            i -= 1
        }
        self.descriptors[idx] = RegionDesc::missing();
    }

    fn insert(&mut self, idx: usize, desc: RegionDesc) {
        if self.descriptors[idx].is_empty() {
            self.descriptors[idx] = desc;
        } else {
            self.shift_right(idx);
            self.descriptors[idx] = desc;
        }
    }

    fn remove(&mut self, base: usize, size: usize) {
        if self.size() != 0 {
            for (i, desc) in self.descriptors.iter_mut().enumerate() {
                if desc.range().contains(&base) && desc.range().contains(&(base + size - 1)) {
                    if desc.base == base && desc.size == size {
                        self.shift_left(i)
                    } else if desc.base == base {
                        desc.base = base + size;
                        desc.size -= size;
                    } else if desc.end() + 1 == base + size {
                        desc.size -= size;
                    } else {
                        let front = RegionDesc { base: desc.base, size: base - desc.base };
                        let end = RegionDesc { base: base + size, size: desc.end() + 1 - (base + size) };
                        *desc = front;
                        self.insert(i + 1, end);
                    }
                    return;
                }
            }
        }

        panic!("Try to remove unavailable memory!")
    }

    fn is_intersecting(&self, base: usize, size: usize) -> bool {
        if self.descriptors[0].is_empty() {
            return false;
        }
        if base + size - 1 < self.descriptors[0].base {
            return false;
        }

        let mut idx = 0;
        for i in 0..(self.descriptors.len() - 1) {
            idx = i;
            if self.descriptors[i + 1].is_empty() {
                break;
            }
            if self.descriptors[i].range().contains(&base) ||
                self.descriptors[i + 1].range().contains(&(base + size - 1)) {
                return true;
            }
        }
        if self.descriptors[idx].base < base {
            return false;
        }
        false
    }

    fn is_subarea(&self, base: usize, size: usize) -> bool {
        if self.descriptors[0].is_empty() {
            return false;
        }
        for desc in self.descriptors.iter() {
            if desc.range().contains(&base) && desc.range().contains(&(base + size - 1)) {
                return true;
            }
        }
        false
    }

    fn size(&self) -> usize {
        let mut count = 0;
        for desc in self.descriptors.iter() {
            if desc.is_empty() {
                break;
            }
            count += 1
        }
        count
    }

    fn capacity(&self) -> usize {
        self.descriptors.len()
    }

    fn add(&mut self, base: usize, size: usize) {
        let mut idx = 0;
        if !(base < self.descriptors[0].base) {
            idx += 1;
            while idx < self.descriptors.len() {
                let desc1 = &self.descriptors[idx - 1];
                let desc2 = &self.descriptors[idx];
                if desc1.is_empty() {
                    idx -= 1;
                    break;
                } else if desc2.is_empty() {
                    break;
                } else if (desc1.base..desc2.base).contains(&base) {
                    break;
                }
                idx += 1;
            }
        }

        self.insert(idx, RegionDesc { base, size });
        self.merge(idx)
    }
}

impl Display for Memblock {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for i in 0..self.size() {
            let desc = self.descriptors[i];
            writeln!(f, "[0x{:X}-0x{:X}], size: 0x{:X}", desc.base, desc.end(), desc.size);
        }
        write!(f, "Used: {}/{}", self.size(), self.capacity())
    }
}

fn main() {
    let mut memblock = Memblock::new();
    /*
    [0x100-0x2EF], size: 0x1F0
    [0x300-0x3EF], size: 0x0F0
    [0x500-0x5EF], size: 0x0F0
    [0x600-0x6EF], size: 0x0F0
    [0x000-0x000], size: 0x000
    */
    memblock.add(0x200, 0xf0);
    memblock.add(0x300, 0xf0);
    memblock.add(0x110, 0xf0);
    memblock.add(0x100, 0xf0);
    memblock.add(0x500, 0xf0);
    memblock.add(0x600, 0xf0);

    memblock.remove(0x650, 0x10);

    println!("{}", memblock);
}
