
pub struct ContentDirectory{
    service_reset_token: ~str //This should probably be in persistend storage.
}

impl ContentDirectory {
    pub fn get_search_capabilities() -> ~str {~""}
    pub fn get_sort_capabilities() -> ~str {~""}
    pub fn get_feature_list() -> ~str {
        let features = r#"<?xml version="1.0" encoding="UTF-8"?>
    <Features
    xmlns="urn:schemas-upnp-org:av:avs"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="
    urn:schemas-upnp-org:av:avs
    http://www.upnp.org/schemas/av/avs.xsd">
    </Features>"#;
        features.to_owned()
    } 

    pub fn get_system_update_id() -> ~str {~""}
    pub fn get_service_reset_token() -> ~str {~""}
    pub fn browse() -> ~str{
        ~""
    }
}
