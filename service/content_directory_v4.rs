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
        let out = 
r##"<?xml version="1.0" encoding="utf-8"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
    <s:Body>
        <u:BrowseResponse xmlns:u="urn:schemas-upnp-org:service:ContentDirectory:1">
            <Result>
                &lt;DIDL-Lite xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:upnp="urn:schemas-upnp-org:metadata-1-0/upnp/" xmlns="urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/" xmlns:dlna="urn:schemas-dlna-org:metadata-1-0/"&gt;
                &lt;container id="64$0" parentID="64" restricted="1" searchable="1" childCount="6"&gt;&lt;dc:title&gt;CSThings&lt;/dc:title&gt;&lt;upnp:class&gt;object.container.storageFolder&lt;/upnp:class&gt;&lt;upnp:storageUsed&gt;-1&lt;/upnp:storageUsed&gt;&lt;/container&gt;&lt;container id="64$1" parentID="64" restricted="1" searchable="1" childCount="16"&gt;&lt;dc:title&gt;Documentaries&lt;/dc:title&gt;&lt;upnp:class&gt;object.container.storageFolder&lt;/upnp:class&gt;&lt;upnp:storageUsed&gt;-1&lt;/upnp:storageUsed&gt;&lt;/container&gt;&lt;container id="64$2" parentID="64" restricted="1" searchable="1" childCount="35"&gt;&lt;dc:title&gt;Movies&lt;/dc:title&gt;&lt;upnp:class&gt;object.container.storageFolder&lt;/upnp:class&gt;&lt;upnp:storageUsed&gt;-1&lt;/upnp:storageUsed&gt;&lt;/container&gt;&lt;container id="64$3" parentID="64" restricted="1" searchable="1" childCount="7"&gt;&lt;dc:title&gt;random_stuff&lt;/dc:title&gt;&lt;upnp:class&gt;object.container.storageFolder&lt;/upnp:class&gt;&lt;upnp:storageUsed&gt;-1&lt;/upnp:storageUsed&gt;&lt;/container&gt;&lt;container id="64$4" parentID="64" restricted="1" searchable="1" childCount="8"&gt;&lt;dc:title&gt;Series&lt;/dc:title&gt;&lt;upnp:class&gt;object.container.storageFolder&lt;/upnp:class&gt;&lt;upnp:storageUsed&gt;-1&lt;/upnp:storageUsed&gt;&lt;/container&gt;&lt;/DIDL-Lite&gt;
            </Result>
            <NumberReturned>5</NumberReturned>
            <TotalMatches>5</TotalMatches>
            <UpdateID>15</UpdateID>
        </u:BrowseResponse>
    </s:Body>
</s:Envelope>"##;
            out.to_owned()
    }
}
