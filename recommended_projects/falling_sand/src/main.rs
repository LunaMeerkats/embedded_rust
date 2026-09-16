// Simple Physics Simulator
// Reference: https://youtu.be/-L5OQp2J46g

// First let's create some elementary particles!
// All elementary particles have three basic properties:

mod elementary_particle;
mod fermions;

use crate::elementary_particle::ElementaryParticle;
use crate::fermions::{QuarkFlavour, LeptonFlavour, Quark, Lepton};

fn main() {
    
    let quark = Quark::new(QuarkFlavour::Up);
    let lepton = Lepton::new(LeptonFlavour::Electron);

    println!("{quark:#?}");
    println!("Flavour: {:?}", quark.flavour());
    println!("Mass: {} MeV/c²", quark.mass());
    println!("Spin: {}", quark.spin());
    println!("Charge: {} e", quark.charge());

    println!("{lepton:#?}");
    println!("Flavour: {:?}", lepton.flavour());
    println!("Mass: {} MeV/c²", lepton.mass());
    println!("Spin: {}", lepton.spin());
    println!("Charge: {} e", lepton.charge());
}
