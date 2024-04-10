use std::io::Read;

//okay so carts are basically giant blobs.
//i think its safe to just....make them a big u8 VEC(since we dont know their size)
//we might need to implement batttery backed ram at some point for game saves and such
#[derive(Default)]
pub struct Cart {
    pub rom: Vec<u8>,
}

impl Cart {
    pub fn new(rom: &str) -> Self {
        let mut file = std::fs::File::open(rom).unwrap();
        let mut buf = vec![0; file.metadata().unwrap().len() as usize];
        let bytes_read = file.read(&mut buf).unwrap();
        assert_eq!(bytes_read, file.metadata().unwrap().len() as usize);

        Cart { rom: buf }
    }
}
