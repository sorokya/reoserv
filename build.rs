use std::{fs, io};
#[cfg(windows)]
use winres::WindowsResource;

fn main() -> io::Result<()> {
    #[cfg(windows)]
    {
        WindowsResource::new()
            // This path can be absolute, or relative to your crate root.
            .set_icon("assets/icon.ico")
            .compile()?;
    }

    let version = env!("CARGO_PKG_VERSION");
    fs::write("VERSION.txt", version)?;

    Ok(())
}
