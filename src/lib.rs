#[cfg(feature = "modules")]
pub mod helper;
mod reader;

pub use crate::reader::ReadError;
use crate::reader::Reader;
use regex::Regex;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum BuildError {
    Io(IoError),
    Parse(String),
}

impl From<IoError> for BuildError {
    fn from(err: IoError) -> Self {
        Self::Io(err)
    }
}

#[derive(Debug)]
pub struct Domain {
    /// `id` of the current domain.
    pub id: u8,
    name: reader::FileReader,
    enabled: reader::FileReader,
    energy: reader::FileReader,
    max_energy_range: reader::FileReader,
}

impl From<(u8, PathBuf)> for Domain {
    /// Creates a socket instance using the speficied path.
    fn from((id, root): (u8, PathBuf)) -> Self {
        Self {
            id,
            name: root.join("name").into(),
            enabled: root.join("enabled").into(),
            energy: root.join("energy_uj").into(),
            max_energy_range: root.join("max_energy_range_uj").into(),
        }
    }
}

impl Domain {
    /// Returns the name of the current domain.
    pub fn name(&self) -> Result<String, ReadError> {
        self.name.read()
    }

    /// Returns wether the socket is enabled or not.
    pub fn enabled(&self) -> Result<bool, ReadError> {
        self.enabled.read()
    }

    /// Returns amount of energy used by the socket.
    /// The returned value is in micro joules.
    pub fn energy(&self) -> Result<u64, ReadError> {
        println!("Domain {:?}", self.energy);
        self.energy.read()
    }

    pub fn max_energy_range(&self) -> Result<u64, ReadError> {
        self.max_energy_range.read()
    }

    pub fn snapshot(&self) -> Result<DomainSnapshot, ReadError> {
        Ok(DomainSnapshot {
            id: self.id.clone(),
            name: self.name()?,
            enabled: self.enabled()?,
            energy: self.energy()?,
            max_energy_range: self.max_energy_range()?,
        })
    }
}

#[derive(Debug)]
pub struct DomainSnapshot {
    pub id: u8,
    pub name: String,
    pub enabled: bool,
    pub energy: u64,
    pub max_energy_range: u64,
}

#[derive(Debug)]
pub struct Socket {
    /// `id` of the current socket.
    pub id: u8,
    /// `domains` of the current socket.
    pub domains: HashMap<u8, Domain>,
    enabled: reader::FileReader,
    energy: reader::FileReader,
    max_energy_range: reader::FileReader,
}

impl TryFrom<(u8, PathBuf)> for Socket {
    type Error = BuildError;

    /// Creates a socket instance using the speficied path.
    fn try_from((id, root): (u8, PathBuf)) -> Result<Self, Self::Error> {
        let domain_name = Regex::new(r"intel-rapl:\d+:(\d+)$").expect("unable to prepare regex");
        let domains = fs::read_dir(&root)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|entry| entry.is_dir())
            .filter_map(|entry| {
                entry
                    .to_str()
                    .and_then(|name| {
                        domain_name
                            .captures(name)
                            .and_then(|cap| cap.get(1))
                            .and_then(|cap| cap.as_str().parse::<u8>().ok())
                    })
                    .map(|id| (id, Domain::from((id, entry))))
            })
            .collect();
        Ok(Self {
            id,
            domains,
            enabled: root.join("enabled").into(),
            energy: root.join("energy_uj").into(),
            max_energy_range: root.join("max_energy_range_uj").into(),
        })
    }
}

impl Socket {
    /// Returns wether the socket is enabled or not.
    pub fn enabled(&self) -> Result<bool, ReadError> {
        self.enabled.read()
    }

    /// Returns amount of energy used by the socket.
    /// The returned value is in micro joules.
    pub fn energy(&self) -> Result<u64, ReadError> {
        self.energy.read()
    }

    pub fn max_energy_range(&self) -> Result<u64, ReadError> {
        self.max_energy_range.read()
    }

    /// Returns the sum of energy used by the sockets and the domains.
    /// The returned value is in micro joules.
    pub fn total_energy(&self) -> Result<u64, ReadError> {
        let mut res = self.energy()?;
        for (_, item) in self.domains.iter() {
            res += item.energy()?;
        }
        Ok(res)
    }

    pub fn snapshot(&self) -> Result<SocketSnapshot, ReadError> {
        Ok(SocketSnapshot {
            id: self.id.clone(),
            enabled: self.enabled()?,
            energy: self.energy()?,
            max_energy_range: self.max_energy_range()?,
            domains: self
                .domains
                .values()
                .map(|domain| domain.snapshot())
                .collect::<Result<Vec<DomainSnapshot>, ReadError>>()?,
        })
    }
}

#[derive(Debug)]
pub struct SocketSnapshot {
    pub id: u8,
    pub enabled: bool,
    pub energy: u64,
    pub max_energy_range: u64,
    pub domains: Vec<DomainSnapshot>,
}

#[derive(Debug)]
pub struct IntelRapl {
    pub sockets: HashMap<u8, Socket>,
}

impl TryFrom<PathBuf> for IntelRapl {
    type Error = BuildError;

    /// Creates an IntelRapl instance using the speficied path.
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let socket_name = Regex::new(r"intel-rapl:(\d+)$").expect("unable to prepare regex");
        let sockets = fs::read_dir(&value)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|entry| entry.is_dir())
            .filter_map(|entry| {
                entry
                    .to_str()
                    .and_then(|name| {
                        socket_name
                            .captures(name)
                            .and_then(|cap| cap.get(1))
                            .and_then(|cap| cap.as_str().parse::<u8>().ok())
                    })
                    .map(|id| (id, entry))
            })
            .filter_map(|(id, entry)| Socket::try_from((id, entry)).ok())
            .map(|socket| (socket.id, socket))
            .collect();
        Ok(Self { sockets })
    }
}

impl IntelRapl {
    /// Returns the sum of energies of the sockets and domains in the IntelRapl folder.
    /// The value's unit is in micro joules.
    pub fn total_energy(&self) -> Result<u64, reader::ReadError> {
        let mut res = 0;
        for (_, item) in self.sockets.iter() {
            res += item.energy()?;
        }
        Ok(res)
    }

    pub fn snapshot(&self) -> Result<IntelRaplSnapshot, ReadError> {
        Ok(IntelRaplSnapshot {
            sockets: self
                .sockets
                .values()
                .map(|socket| socket.snapshot())
                .collect::<Result<Vec<SocketSnapshot>, ReadError>>()?,
        })
    }
}

#[derive(Debug)]
pub struct IntelRaplSnapshot {
    pub sockets: Vec<SocketSnapshot>,
}

/// PowerCap folder representation.
#[derive(Debug)]
pub struct PowerCap {
    pub intel_rapl: IntelRapl,
}

impl PowerCap {
    /// Returns a PowerCap instance using the default powercap folder.
    pub fn try_default() -> Result<Self, BuildError> {
        Self::try_from(PathBuf::from("/sys/class/powercap"))
    }
}

impl TryFrom<PathBuf> for PowerCap {
    type Error = BuildError;

    /// Creates a PowerCap instance using the speficied powercap path.
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        Ok(Self {
            intel_rapl: IntelRapl::try_from(value.join("intel-rapl"))?,
        })
    }
}

impl TryFrom<&Path> for PowerCap {
    type Error = BuildError;

    /// Creates a PowerCap instance using the speficied powercap path.
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        Ok(Self {
            intel_rapl: IntelRapl::try_from(value.join("intel-rapl"))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::PowerCap;
    use std::convert::TryFrom;
    use std::path::PathBuf;

    #[test]
    fn build_and_measure() {
        let root = PathBuf::from(".").join("assets").join("success");
        let cap = PowerCap::try_from(root).unwrap();
        let value = cap.intel_rapl.total_energy().unwrap();
        assert_ne!(value, 0);
        for socket in cap.intel_rapl.sockets.values() {
            assert!(socket.enabled().is_ok());
            assert!(socket.energy().is_ok());
            assert!(socket.max_energy_range().is_ok());
            assert!(socket.total_energy().is_ok());
            for domain in socket.domains.values() {
                assert!(domain.name().is_ok());
                assert!(domain.energy().is_ok());
                assert!(domain.max_energy_range().is_ok());
            }
        }
        let snap = cap.intel_rapl.snapshot().unwrap();
        assert_eq!(snap.sockets.len(), cap.intel_rapl.sockets.len());
    }
}
