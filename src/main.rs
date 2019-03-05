use std::cmp::max;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt;

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
}

const MAX_REGION_DESC: usize = 10;

struct Memblock {
    descriptors: [RegionDesc; MAX_REGION_DESC]
}

impl Memblock {
    fn new() -> Self {
        assert!(MAX_REGION_DESC > 0);
        Memblock { descriptors: [RegionDesc::missing(); MAX_REGION_DESC] }
    }

    fn merge(&mut self, idx: usize) {
        if idx + 1 == self.descriptors.len() {
            return;
        }

        let subject = self.descriptors[idx + 1].clone();
        if subject.is_empty() {
            return
        }
        let mut object = &mut self.descriptors[idx];
        assert!(object.base <= subject.base);

        if object.end() + 1 < subject.base { return; }  // [0~0xFF] [0x100~0x1FF] can be merged into [0~0x1FF]
        object.size = max(subject.end() + 1 - object.base, object.size);

        self.shift_left(idx + 1);
        self.merge(idx + 1)
    }

    fn shift_left(&mut self, idx: usize) {
        for i in (idx + 1)..self.descriptors.len() {
            self.descriptors[i - 1] = self.descriptors[i]
        }
        *self.descriptors.last_mut().unwrap() = RegionDesc::missing();
    }

    fn shift_right(&mut self, idx: usize) {
        if self.descriptors.len() == 1 {
            self.descriptors[0] = RegionDesc::missing();
            return
        }
        let mut i = self.descriptors.len() - 2;
        while i >= idx {
            self.descriptors[i + 1] = self.descriptors[i];
            i -= 1
        }
        self.descriptors[i + 1] = RegionDesc::missing();
    }

    fn insert(&mut self, idx: usize, desc: RegionDesc) {
        if self.descriptors[idx].is_empty() {
            self.descriptors[idx] = desc;
        } else {
            assert!(idx < self.descriptors.len() - 1,
                    "No enough memory to store region descriptor!");
            self.shift_right(idx + 1);
            self.descriptors[idx + 1] = desc;
        }
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
                break
            }
            if self.descriptors[i].end() < base && base + size - 1 < self.descriptors[i + 1].base {
                return false;
            }
        }
        if self.descriptors[idx].base < base {
            return false;
        }
        true
    }

    fn add_region(&mut self, base: usize, size: usize) {
        let mut idx = 0;
        for desc in self.descriptors.iter_mut() {
            if desc.is_empty() {
                idx = if idx == 0 { 0 } else { idx - 1 };
                break
            } else if desc.base < base {
                break;
            } else {
                idx += 1;
                continue;
            }
        };

        self.insert(idx, RegionDesc { base, size });
        self.merge(idx);
    }
}

impl Display for Memblock {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for desc in self.descriptors.iter() {
            writeln!(f, "[0x{:X}-0x{:X}], size: 0x{:X}", desc.base, desc.end(), desc.size);
        }
        write!(f, "")
    }
}

fn main() {
    let mut memblock = Memblock::new();
    memblock.add_region(0, 0x100);
    memblock.add_region(0x105, 0x100);
    let ist = memblock.is_intersecting(0xff, 5);
    println!("{}, {}", memblock, ist);
}
