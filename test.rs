extern mod upnp;

//#[test]
//fn build_device(){
    //let s1 = upnp::service::service::Service::new("urn:schemas-upnp-org:device:MediaServer:1",
    //"uuid:4d696e69-444c-164e-9d41-e0cb5ebb5910::urn:schemas-upnp-org:device:MediaServer:1");

    //let s2 = upnp::service::service::Service::new("urn:microsoft.com:service:X_MS_MediaReceiverRegistrar:1",
    //"uuid:4d696e69-444c-164e-9d41-e0cb5ebb5910::urn:microsoft.com:service:X_MS_MediaReceiverRegistrar:1");

    //let mut dev = upnp::device::device::Device{friendly_name: ~"LaleServer", services: ~[], location: ~"/device.xml"};

    //dev.add_service(s1);
    //dev.add_service(s2);

    //let messages = dev.generate_notify(Up);
    //println("---\n");
    //for m in messages.iter() {
        //println(m.to_str());
    //}
//}
//fn add_service(){

//}
#[test]
fn test_http(){
    upnp::http::listen("192.168.1.3:8900");
}
