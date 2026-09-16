// Simple Physics Simulator
// Reference: https://youtu.be/-L5OQp2J46g

// First let's create some elementary particles!
// All elementary particles have three basic properties:

#[derive(Debug)]
struct ParticleProperties {
    mass: f64,      // MeV/c²
    twice_spin: u8, // Stores 2s: 1 means spin ½
    charge_thirds: i8,    // Charge in units of e/3; 2 means +2e/3.
}


trait ElementaryParticle{
    // Each implementation must provide these methods.
    fn properties(&self) -> &ParticleProperties;

    fn mass(&self) -> f64{
        self.properties().mass
    }

    fn spin(&self) -> f64{
        f64::from(self.properties().twice_spin) / 2.0
    }

    fn charge(&self) -> f64 {
        f64::from(self.properties().charge_thirds) / 3.0
    }
}

#[derive(Debug)]
enum QuarkFlavour{
    Up,
    Down,
    Charm,
    Strange,
    Top,
    Bottom,
}

#[derive(Debug)]
struct Quark {
    flavour: QuarkFlavour,
    properties: ParticleProperties,
}

impl ElementaryParticle for Quark {
    fn properties(&self) -> &ParticleProperties {
        &self.properties
    }
}

impl Quark {
    fn new(flavour: QuarkFlavour) -> Self {
        let (mass, twice_spin, charge_thirds) = match flavour {
            QuarkFlavour::Up        => (2.4, 1, 2),
            QuarkFlavour::Down      => (4.7, 1, -1),
            QuarkFlavour::Charm     => (1280.0, 1, 2), 
            QuarkFlavour::Strange   => (96.0, 1, -1),
            QuarkFlavour::Top       => (173000.0, 1, 2),
            QuarkFlavour::Bottom    => (4180.0, 1, -1),
        };

        Self {
            flavour,
            properties: ParticleProperties {
                mass,
                twice_spin,
                charge_thirds
            }
        }
    }
}

#[derive(Debug)]
enum LeptonFlavour{
    Electron,
    ElectronNeutrino,
    Muon,
    MuonNeutrino,
    Tau,
    TauNeutrino,
}

#[derive(Debug)]
struct Lepton {
    flavour: LeptonFlavour,
    properties: ParticleProperties,
}

impl ElementaryParticle for Lepton {
    fn properties(&self) -> &ParticleProperties {
        &self.properties
    }
}

impl Lepton {
    fn new(flavour: LeptonFlavour) -> Self {
        let (mass, twice_spin, charge_thirds) = match flavour {
            LeptonFlavour::Electron         => (0.511, 1, -3),
            LeptonFlavour::ElectronNeutrino => (0.0, 1, 0),
            LeptonFlavour::Muon             => (105.658, 1, -3), 
            LeptonFlavour::MuonNeutrino     => (0.0, 1, 0),
            LeptonFlavour::Tau              => (1776.93, 1, -3),
            LeptonFlavour::TauNeutrino      => (0.0, 1, 0),
        };

        Self {
            flavour,
            properties: ParticleProperties {
                mass,
                twice_spin,
                charge_thirds
            }
        }
    }
}
fn main() {
    
    let quark = Quark::new(QuarkFlavour::Up);
    let lepton = Lepton::new(LeptonFlavour::Electron);

    println!("{quark:#?}");
    println!("Flavour: {:?}", quark.flavour);
    println!("Mass: {} MeV/c²", quark.mass());
    println!("Spin: {}", quark.spin());
    println!("Charge: {} e", quark.charge());

    println!("{lepton:#?}");
    println!("Flavour: {:?}", lepton.flavour);
    println!("Mass: {} MeV/c²", lepton.mass());
    println!("Spin: {}", lepton.spin());
    println!("Charge: {} e", lepton.charge());
}
