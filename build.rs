fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        
        // get version information from Cargo.toml
        let version = env!("CARGO_PKG_VERSION");
        let name = env!("CARGO_PKG_NAME");
        let description = env!("CARGO_PKG_DESCRIPTION");
        let authors = env!("CARGO_PKG_AUTHORS");
        let repository = env!("CARGO_PKG_REPOSITORY");
        
        // parse version into u64
        let version_parts: Vec<&str> = version.split('.').collect();
        let major = version_parts.get(0).unwrap_or(&"0").parse::<u64>().unwrap_or(0);
        let minor = version_parts.get(1).unwrap_or(&"0").parse::<u64>().unwrap_or(0);
        let patch = version_parts.get(2).unwrap_or(&"0").parse::<u64>().unwrap_or(0);
        let version_u64 = (major << 48) | (minor << 32) | (patch << 16);
        
        // set version information
        res.set_version_info(winres::VersionInfo::PRODUCTVERSION, version_u64);
        res.set_version_info(winres::VersionInfo::FILEVERSION, version_u64);
        
        // set other resource information
        res.set("CompanyName", if authors.is_empty() { "Open Source Project" } else { authors });
        res.set("FileDescription", description);
        res.set("FileVersion", &format!("{}.0", version));
        res.set("InternalName", name);
        res.set("LegalCopyright", "Copyright (c) 2025 Open Source Community");
        res.set("OriginalFilename", &format!("{}.exe", name));
        res.set("ProductName", "Tencent ACE Tools");
        res.set("ProductVersion", &format!("{}.0", version));
        res.set("Comments", &format!("{}. Source: {}", description, repository));
        
        // set language and character set
        res.set_language(0x0409); // English
        
        // embed manifest file
        res.set_manifest_file("app.manifest");
        
        // res.set_icon("icon.ico");
        
        if let Err(e) = res.compile() {
            eprintln!("Error: Failed to compile Windows resources: {}", e);
            std::process::exit(1);
        }
    }

    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=Cargo.toml");
}
