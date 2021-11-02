use std::collections::HashSet;

pub fn check_modules() -> Result<bool, procfs::ProcError> {
    let modules = procfs::modules()?;
    let set = modules
        .values()
        .map(|module| module.name.as_str())
        .collect::<HashSet<_>>();
    Ok(set.contains("intel_rapl")
        || (set.contains("intel_rapl_msr") && set.contains("intel_rapl_common")))
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_modules_are_loaded() {
        assert!(super::check_modules().unwrap());
    }
}
