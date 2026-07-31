use super::{PackageManager, PackageUpdate};

pub struct Pacman;

impl PackageManager for Pacman {
    fn id(&self) -> &'static str {
        "pacman"
    }

    fn list_upgradable(&self) -> Result<Vec<PackageUpdate>, String> {
        Ok(Vec::new())
    }
}
