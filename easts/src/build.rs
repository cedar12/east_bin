use std::{path::{PathBuf}, fs};

#[cfg(windows)]
extern crate winres;

fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        
        // res.set_icon("path/to/your_icon.ico");
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        res.set("ProductName", "east server");
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.set("FileDescription", "TCP端口转发工具 服务端");
        res.set("LegalCopyright", "cedar12.zxd@qq.com");
        res.set("CompanyName", "cedar12.zxd@qq.com");
        
        res.compile().unwrap();
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root=manifest_dir.parent().unwrap();
    let target=root.join("target");
    let release=target.join("release");
    fs::copy(manifest_dir.join("easts.yml"), release.join("easts.yml")).unwrap();
}
