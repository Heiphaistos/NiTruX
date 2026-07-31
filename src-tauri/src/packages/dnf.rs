use super::{PackageManager, PackageUpdate};

pub struct Dnf;

impl PackageManager for Dnf {
    fn id(&self) -> &'static str {
        "dnf"
    }

    fn list_upgradable(&self) -> Result<Vec<PackageUpdate>, String> {
        Ok(Vec::new())
    }
}
