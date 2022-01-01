use std::fs;
use std::io::Result;
use std::path::Path;

fn default_socket_generator(_socket: usize) -> u64 {
    1234
}

fn default_domain_generator(_socket: usize, _domain: usize) -> u64 {
    1234
}

type SocketEnergyGenerator = Box<dyn Fn(usize) -> u64>;
type DomainEnergyGenerator = Box<dyn Fn(usize, usize) -> u64>;

pub struct MockBuilder {
    enabled: bool,
    sockets: usize,
    socket_energy_generator: SocketEnergyGenerator,
    socket_max_energy_range_generator: SocketEnergyGenerator,
    domain_energy_generator: Box<dyn Fn(usize, usize) -> u64>,
    domain_max_energy_range_generator: Box<dyn Fn(usize, usize) -> u64>,
}

impl Default for MockBuilder {
    fn default() -> Self {
        Self {
            enabled: true,
            sockets: 1,
            socket_energy_generator: Box::new(default_socket_generator),
            socket_max_energy_range_generator: Box::new(default_socket_generator),
            domain_energy_generator: Box::new(default_domain_generator),
            domain_max_energy_range_generator: Box::new(default_domain_generator),
        }
    }
}

impl MockBuilder {
    pub fn with_enabled(mut self, value: bool) -> Self {
        self.enabled = value;
        self
    }

    pub fn with_sockets(mut self, value: usize) -> Self {
        self.sockets = value;
        self
    }

    pub fn with_socket_energy_generator(mut self, value: SocketEnergyGenerator) -> Self {
        self.socket_energy_generator = value;
        self
    }

    pub fn with_socket_max_energy_range_generator(mut self, value: SocketEnergyGenerator) -> Self {
        self.socket_max_energy_range_generator = value;
        self
    }

    pub fn with_domain_energy_generator(mut self, value: DomainEnergyGenerator) -> Self {
        self.domain_energy_generator = value;
        self
    }

    pub fn with_domain_max_energy_range_generator(mut self, value: DomainEnergyGenerator) -> Self {
        self.domain_max_energy_range_generator = value;
        self
    }

    fn build_domain(&self, path: &Path, socket: usize, domain: usize, name: &str) -> Result<()> {
        fs::create_dir_all(&path)?;
        fs::write(&path.join("name"), name)?;
        fs::write(&path.join("enabled"), if self.enabled { "1" } else { "0" })?;
        fs::write(
            &path.join("energy_uj"),
            (self.domain_energy_generator)(socket, domain).to_string(),
        )?;
        fs::write(
            &path.join("max_energy_range_uj"),
            (self.domain_max_energy_range_generator)(socket, domain).to_string(),
        )?;
        Ok(())
    }

    fn build_socket(&self, path: &Path, socket: usize) -> Result<()> {
        fs::create_dir_all(&path)?;
        fs::write(&path.join("name"), format!("package-{}", socket))?;
        fs::write(&path.join("enabled"), if self.enabled { "1" } else { "0" })?;
        fs::write(
            &path.join("energy_uj"),
            (self.socket_energy_generator)(socket).to_string(),
        )?;
        fs::write(
            &path.join("max_energy_range_uj"),
            (self.socket_max_energy_range_generator)(socket).to_string(),
        )?;
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
