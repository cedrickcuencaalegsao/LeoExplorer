use shared::OsInfo;

pub async fn get_os_info() -> Result<OsInfo, String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH.to_string();

    #[cfg(target_os = "windows")]
    let (name, version) = {
        use std::process::Command;
        let output = Command::new("cmd")
            .args(["/C", "ver"])
            .output()
            .map_err(|e| e.to_string())?;

        let raw = String::from_utf8_lossy(&output.stdout).to_string();
        // "Microsoft Windows [Version 10.0.22631.0]" → extract build number
        let version = raw
            .split("Version")
            .nth(1)
            .unwrap_or("")
            .trim()
            .trim_end_matches(']')
            .trim()
            .to_string();

        // Distinguish Windows 10 vs 11 by build number (Win11 >= 22000)
        let build: u32 = version
            .split('.')
            .nth(2)
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

        let name = if build >= 22000 {
            "Windows 11"
        } else {
            "Windows 10"
        };

        (name.to_string(), version)
    };

    #[cfg(target_os = "macos")]
    let (name, version) = {
        use std::process::Command;
        let output = Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .map_err(|e| e.to_string())?;

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // macOS major version → marketing name
        let major: u32 = version
            .split('.')
            .next()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

        let name = match major {
            15 => "macOS Sequoia",
            14 => "macOS Sonoma",
            13 => "macOS Ventura",
            12 => "macOS Monterey",
            11 => "macOS Big Sur",
            _  => "macOS",
        };

        (name.to_string(), version)
    };

    #[cfg(target_os = "linux")]
    let (name, version) = {
        use std::process::Command;

        // Try reading /etc/os-release first (works on all major distros)
        let release = std::fs::read_to_string("/etc/os-release").unwrap_or_default();

        let get_field = |key: &str| -> String {
            release
                .lines()
                .find(|l| l.starts_with(key))
                .and_then(|l| l.splitn(2, '=').nth(1))
                .map(|v| v.trim_matches('"').to_string())
                .unwrap_or_default()
        };

        let name    = get_field("PRETTY_NAME");
        let version = get_field("VERSION_ID");

        // Fallback: uname -r (kernel version)
        let (name, version) = if name.is_empty() {
            let output = Command::new("uname")
                .arg("-r")
                .output()
                .map_err(|e| e.to_string())?;
            let kernel = String::from_utf8_lossy(&output.stdout).trim().to_string();
            ("Linux".to_string(), kernel)
        } else {
            (name, version)
        };

        (name, version)
    };

    Ok(OsInfo {
        os: os.to_string(),
        name,
        version,
        arch,
    })
}