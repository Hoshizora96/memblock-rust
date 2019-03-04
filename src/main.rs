
#[derive(Copy, Clone)]
struct RegionDesc {
    base: usize,
    size: usize,
}

impl RegionDesc {
    fn missing() -> Self {
        RegionDesc {
            base: 0,
            size: 0
        }
    }

    fn is_empty(&self) -> bool {
        self.size == 0
    }
}

struct Memblock {
    descriptors: [RegionDesc; 60]
}

impl Memblock {
    fn new() -> Self {
        Memblock { descriptors: [RegionDesc::missing(); 60] }
    }

    fn merge(&mut self, idx: usize) {

    }
}

fn main() {
    let mut memblock = Memblock;

    println!("Hello, world!");
}
