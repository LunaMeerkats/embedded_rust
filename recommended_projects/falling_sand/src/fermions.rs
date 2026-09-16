
use crate::elementary_particle::{ElementaryParticle, ParticleProperties};

#[derive(Debug)]
pub(crate) enum QuarkFlavour{
    Up,
    Down,
    Charm,
    Strange,
    Top,
    Bottom,
}

#[derive(Debug)]
pub(crate) struct Quark {
    flavour: QuarkFlavour,
    properties: ParticleProperties,
}

impl ElementaryParticle for Quark {
    fn properties(&self) -> &ParticleProperties {
        &self.properties
    }
}

impl Quark {

    pub(crate) fn flavour(&self) -> &QuarkFlavour {
        &self.flavour
    }
    
    pub(crate) fn new(flavour: QuarkFlavour) -> Self {
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
pub(crate) enum LeptonFlavour{
    Electron,
    ElectronNeutrino,
    Muon,
    MuonNeutrino,
    Tau,
    TauNeutrino,
}

#[derive(Debug)]
pub(crate) struct Lepton {
    flavour: LeptonFlavour,
    properties: ParticleProperties,
}

impl ElementaryParticle for Lepton {
    fn properties(&self) -> &ParticleProperties {
        &self.properties
    }
}

impl Lepton {

    pub(crate) fn flavour(&self) -> &LeptonFlavour {
        &self.flavour
    }

    pub(crate) fn new(flavour: LeptonFlavour) -> Self {
        let (mass, twice_spin, charge_thirds) = match flavour {
            LeptonFlavour::Electron         => (0.511, 1, -3),
            LeptonFlavour::ElectronNeutrino => (0.8, 1, 0),
            LeptonFlavour::Muon             => (105.658, 1, -3), 
            LeptonFlavour::MuonNeutrino     => (0.17, 1, 0),
            LeptonFlavour::Tau              => (1776.93, 1, -3),
            LeptonFlavour::TauNeutrino      => (18.2, 1, 0),
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
