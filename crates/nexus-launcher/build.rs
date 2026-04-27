// SPDX-License-Identifier: AGPL-3.0-or-later
fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/nexus-launcher.ico");
        res.compile().unwrap();
    }
}
