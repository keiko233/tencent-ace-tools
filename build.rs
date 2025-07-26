fn main() {
    // 在Windows平台上嵌入manifest文件
    #[cfg(windows)]
    {
        embed_manifest::embed_manifest(embed_manifest::new_manifest("app.manifest"))
            .expect("unable to embed manifest file");
    }

    println!("cargo:rerun-if-changed=app.manifest");
}
