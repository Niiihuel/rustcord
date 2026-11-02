//! `rustcord.exe`'s icon and version information, for Windows targets only:
//! Explorer, the taskbar, the installer's uninstall entry and the file's
//! properties show them. Other targets build nothing here.

fn main() {
    println!("cargo:rerun-if-changed=../../assets/icons/rustcord.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut resource = winresource::WindowsResource::new();
    resource
        .set_icon("../../assets/icons/rustcord.ico")
        .set("ProductName", "Rustcord")
        .set("FileDescription", "Rustcord")
        .set("OriginalFilename", "rustcord.exe")
        .set("InternalName", "rustcord");
    // Only the program: tests and examples need no icon.
    if let Err(error) = resource.compile_for("rustcord") {
        panic!("could not embed rustcord.exe's icon and version: {error}");
    }
}
