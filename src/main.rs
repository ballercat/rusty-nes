#[macro_use]
extern crate lazy_static;

use std::env;

pub mod cpu;
pub mod nes;

use cpu::base::Processor;
use nes::Nes;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filepath = &args[1];

    let cpu = Processor::new(None);
    let mut nes = Nes::new(cpu);
    nes.load_cartridge(filepath);

    println!("iNES Header {:?}", nes.cartridge.header);
    println!("ROM size {:?}", nes.cartridge.rom.len());

    // it's possible to run the nestest.nes w/o any GFX by starting execution at 0x0C000
    nes.run(Some(0x0C000));
}
