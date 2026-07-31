use super::{PackageManager, PackageUpdate};

pub struct Apt;

impl PackageManager for Apt {
    fn id(&self) -> &'static str {
        "apt"
    }

    fn list_upgradable(&self) -> Result<Vec<PackageUpdate>, String> {
        Ok(Vec::new())
    }
}
