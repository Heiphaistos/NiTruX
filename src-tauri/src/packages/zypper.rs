use super::{PackageManager, PackageUpdate};

pub struct Zypper;

impl PackageManager for Zypper {
    fn id(&self) -> &'static str {
        "zypper"
    }

    fn list_upgradable(&self) -> Result<Vec<PackageUpdate>, String> {
        Ok(Vec::new())
    }
}
