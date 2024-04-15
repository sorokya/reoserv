use eolib::protocol::net::Version;

pub fn is_deep(version: &Version) -> bool {
    version.minor > 0
}
