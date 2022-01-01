use std::fs;
use std::io::Result;
use std::path::Path;

pub struct MockBuilder {
    enabled: bool,
    sockets: usize,
}

impl Default for MockBuilder {
    fn default() -> Self {
        Self {
            enabled: true,
            sockets: 1,
        }
    }
}

impl MockBuilder {
    fn build_domain(&self, path: &Path, _socket: usize, _domain: usize, name: &str) -> Result<()> {
        fs::create_dir_all(&path)?;
        fs::write(&path.join("name"), name)?;
        fs::write(&path.join("enabled"), if self.enabled { "1" } else { "0" })?;
        fs::write(&path.join("energy_uj"), "123456")?;
        fs::write(&path.join("max_energy_range_uj"), "123456")?;
        Ok(())
    }

    fn build_socket(&self, path: &Path, socket: usize) -> Result<()> {
        fs::create_dir_all(&path)?;
        fs::write(&path.join("name"), format!("package-{}", socket))?;
        fs::write(&path.join("enabled"), if self.enabled { "1" } else { "0" })?;
        fs::write(&path.join("energy_uj"), "123456")?;
        fs::write(&path.join("max_energy_range_uj"), "123456")?;
        for (i, name) in ["core", "nocore", "dram"].iter().enumerate() {
            let domain_path = path.join(format!("intel-rapl:{}:{}", socket, i));
            self.build_domain(&domain_path, socket, i, name)?;
        }
        Ok(())
    }

    fn build_intel_rapl(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(&path)?;
        fs::write(&path.join("enabled"), if self.enabled { "1" } else { "0" })?;
        for i in 0..self.sockets {
            let socket_path = path.join(format!("intel-rapl:{}", i));
            self.build_socket(&socket_path, i)?;
        }
        Ok(())
    }

    pub fn build(&self, path: &Path) -> Result<()> {
        let intel_rapl = path.join("intel-rapl");
        self.build_intel_rapl(&intel_rapl)
    }
}
