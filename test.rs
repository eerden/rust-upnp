extern mod upnp;
extern mod xml;
extern mod sqlite;
use std::io::stdio::println;
use std::path::Path;
use upnp::http::Request;
use std::io::net::tcp::TcpStream;
use std::io::{File,fs};
use std::comm::Chan;
use std::str;
use xml::{Element,CharacterNode};
use sqlite::database::Database;
use sqlite::types::{SQLITE_ROW,SQLITE_ERROR,BindArg,Text,Integer};

#[test]

fn test_http(){
    ////update_db();
    ////return;
    //let c = advertise();
    //do spawn{
        //upnp::http::listen("192.168.1.3:8900",handle);
    //}
}

fn handle(req:Request) -> ~[u8]{
    println("handle function called...");
    let (method, url)  = (req.method.clone(), req.url.clone());
    match (method, url) {
        (GET, ~"/icon.png") => {
            println("Icon requested.");
            send_icon("icon.png",req)
        },
        (GET, ~"/rootDesc.xml") => {
            println("Root doc requested.");
            send_xml_file("rootDesc.xml",req)
        },
        (GET,~"/content_dir.xml") => {
            println("Content directory service SCPD doc requested.");
            send_xml_file("content_dir.xml",req)
        },
        //(GET,~"/connection_manager.xml") => println("Connection manager service SCPD doc requested."),
        //(GET,~"/av_transport.xml") => println("AV Transport service SCPD doc requested."),

        (POST,~"/control/content_dir") => {
            println("Content directory service control command.");
            content_dir(req)

        }
        //(POST,~"/control/connection_manager") => println("Connection manager service control command."),
        //(POST,~"/control/av_transport") => println("AV transport service control command."),

        //(POST,~"/event/content_dir") => println("Content directory service event command."),
        //(POST,~"/event/connection_manager") => println("Connection manager service event command."),
        //(POST,~"/event/av_transport") => println("AV transport service event command."),

        (_,_) => send_empty()
    }
}

fn content_dir(req: Request) -> ~[u8] {
    let mut response : ~[u8] = ~[];
    let mut reqxml : Element = from_str(req.body.unwrap()).unwrap();
    //get_content_2(~reqxml);

    let result = get_content_2(~reqxml);
    //let result = get_content();
    println(result);

    let xml_headers = upnp::http::default_xml_headers();
    response.push_all_move(xml_headers);
    response.push_all_move(result.into_bytes());
    response

}

fn send_xml_file(filename: &str, req: Request) -> ~[u8]{
    let mut response : ~[u8] = ~[];
    let xml_headers = upnp::http::default_xml_headers();
    let path = Path::new("/home/ercan/rust/src/upnp/" + filename);
    let mut file = File::open(&path);
    let buf = file.read_to_end();
    response.push_all_move(xml_headers);
    response.push_all_move(buf);
    response
}

fn send_icon(filename: &str, req: Request) -> ~[u8] {
    let mut response : ~[u8] = ~[];
    let img_headers = upnp::http::default_img_headers();
    let path = Path::new("/home/ercan/rust/src/upnp/" + filename);
    let mut file = File::open(&path);
    let buf = file.read_to_end();
    response.push_all_move(img_headers);
    response.push_all_move(buf);
    response

}

fn advertise() -> std::comm::Chan<bool>{
    let messages = upnp::media_server_v4::get_messages(); 
    let kill_chan = upnp::ssdp::advertise(messages);
    kill_chan
}

fn send_empty() -> ~[u8]{
    ~[]
}

fn get_content_2(xml_action: ~Element) -> ~str {
    let mut item_list : ~[~ResultItem] = ~[];
    let path = "/home/ercan/rust/src/upnp/library.db";
    let db = match sqlite::open(path) {
        Ok(db)  => db,
        Err(m)  => fail!(m)
    };

    let action = BrowseActionIn::new(xml_action);
    let parent_id = match action.object_id.clone() {
        ~0i64 => ~1,
        x => x
    };

    let sql = "select * from library where parent_id = " + parent_id.to_str() ;
    println(sql);

    let cursor = match db.prepare(sql, &None) {
        Err(e) => fail!(),
        Ok(c)   => c
    };

    loop {
        let res = match cursor.step_row() {
            Ok(r) => r,
            Err(e) => fail!() 
        };
        let mut row_map = match res {
            Some(m) => m,
            None    => break
        };

        let path = row_map.pop(&~"path").unwrap();
        let id = row_map.pop(&~"id").unwrap();
        let is_dir = row_map.pop(&~"is_dir").unwrap();
        let parent_id = row_map.pop(&~"parent_id").unwrap();

        match (id,path,is_dir,parent_id) {
            (Integer(ref i), Text(ref p), Integer(0),Integer(p_id)) =>{
                item_list.push(~ResultItem{id:*i as i64,is_dir: false,parent_id: p_id as i64, path: from_str(*p).unwrap()});
            },
            (Integer(ref i), Text(ref p), Integer(1),Integer(p_id)) =>{
                item_list.push(~ResultItem{id:*i as i64,is_dir: true,parent_id: p_id as i64, path: from_str(*p).unwrap()});
            },
            _       => ()
        }
    }

    let out = content_xml(item_list);
    out
}

fn content_xml(list: ~[~ResultItem]) -> ~str{
    let mut mid : ~[~str] = ~[];
    mid.push(~"<DIDL-Lite xmlns='urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/' xmlns:dc='http://purl.org/dc/elements/1.1/' xmlns:upnp='urn:schemas-upnp-org:metadata-1-0/upnp/' xmlns:dlna='urn:schemas-dlna-org:metadata-1-0/'>");

    let number_returned = list.len().to_str();
    let xml_top = ~r#"<?xml version="1.0" encoding="UTF-8"?>
    <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
    <s:Body>
    <u:BrowseResponse xmlns:u="urn:schemas-upnp-org:service:ContentDirectory:4">
    <Result>"#;
    let xml_bottom = ~"</Result>\n<NumberReturned>" + number_returned +"</NumberReturned>\n<TotalMatches>"+ number_returned +"</TotalMatches>\n<UpdateID>17</UpdateID>\n</u:BrowseResponse>\n</s:Body>\n</s:Envelope>\n";

    for item in list.iter() {
        mid.push(make_didl_item(item.clone()));
    }

    //xml_top + (mid.concat()) + xml_bottom
    xml_top + xml::escape(mid.concat()) + xml_bottom

}

fn make_didl_item(item: ~ResultItem) -> ~str {
    let mut out : ~str = ~"";
    if item.is_dir{
        let open_tag = "\n<container id=\'"+ item.id.to_str() +"\' parentID=\'" + item.parent_id.to_str() + "\' restricted=\'1\'>\n";
        let class = "<upnp:class>object.container.storageFolder</upnp:class>\n";
        let title = "<dc:title>" + item.path.display().to_str() + "</dc:title>\n";
        let storage_used = "<upnp:storageUsed>-1</upnp:storageUsed>\n";
        let close_tag = "</container>\n";
        out = open_tag + title + class + storage_used + close_tag;
    } else {
        let open_tag = "\n<item id=\'"+ item.id.to_str() +"\' parentID=\'" + item.parent_id.to_str() + "\' restricted=\'1\'>\n";
        let title = "<dc:title>" + item.path.display().to_str() + "</dc:title>\n";
        let class = "<upnp:class>object.item.videoItem</upnp:class>\n";
        let close_tag = "</item>\n";
        out = open_tag + title + class + close_tag;
    }

    out
}

#[deriving(Clone)]
struct ResultItem{
    id: i64,
    is_dir: bool,
    parent_id: i64,
    path: Path
}

fn get_content() -> ~str {

    let doc_top_str = ~r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
        <s:Body>
            <u:BrowseResponse xmlns:u="urn:schemas-upnp-org:service:ContentDirectory:1">
                <Result>"#;
    let doc_bottom_str = ~r#"
                </Result>
                <NumberReturned>4</NumberReturned>
                <TotalMatches>4</TotalMatches>
                <UpdateID>17</UpdateID>
            </u:BrowseResponse>
        </s:Body>
    </s:Envelope>
    "#;

    let didl = ~r##"
<DIDL-Lite xmlns='urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/' xmlns:dc='http://purl.org/dc/elements/1.1/' xmlns:upnp='urn:schemas-upnp-org:metadata-1-0/upnp/' xmlns:dlna='urn:schemas-dlna-org:metadata-1-0/'>
<item id='64$3$8' parentID='64$3'>
    <dc:title>Eddie Izzard - Force Majeure</dc:title>
    <upnp:class>object.item.videoItem</upnp:class>
</item>
"##;



    //let didl = ~r##"<DIDL-Lite xmlns='urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/' xmlns:dc='http://purl.org/dc/elements/1.1/' xmlns:upnp='urn:schemas-upnp-org:metadata-1-0/upnp/' xmlns:dlna='urn:schemas-dlna-org:metadata-1-0/'>
//<container id='64$3$1' parentID='64$3' restricted='1' searchable='1' childCount='6'>
    //<dc:title>Almost Human</dc:title>
    //<upnp:class>object.container.storageFolder</upnp:class>
    //<upnp:storageUsed>-1</upnp:storageUsed>
//</container>
//<item id='64$3$8' parentID='64$3' restricted='1'>
    //<dc:title>Eddie Izzard - Force Majeure</dc:title>
    //<upnp:class>object.item.videoItem</upnp:class>
    //<dc:date>2013-12-26T23:11:55</dc:date>
    //<res size='371002902' duration='1:25:23.120' bitrate='72417' sampleFrequency='44100' nrAudioChannels='2' resolution='640x356' protocolInfo='http-get:*:video/mp4:DLNA.ORG_PN=AVC_MP4_BL_L3L_SD_AAC;DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=01700000000000000000000000000000'>http://192.168.1.3:8200/MediaItems/811.mp4</res>
//</item>
//<item id='64$3$9' parentID='64$3' restricted='1'>
    //<dc:title>The Future of God Debate Sam Harris and Michael Shermer vs Deepak Chopra and Jean Houston</dc:title>
    //<upnp:class>object.item.videoItem</upnp:class>
    //<dc:date>2012-08-17T18:57:06</dc:date>
    //<res size='309414544' duration='1:51:05.775' bitrate='46418' sampleFrequency='44100' nrAudioChannels='2' resolution='640x360' protocolInfo='http-get:*:video/mp4:DLNA.ORG_PN=AVC_MP4_BL_L3L_SD_AAC;DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=01700000000000000000000000000000'>http://192.168.1.3:8200/MediaItems/815.mp4</res>
//</item>
//<item id='64$3$5' parentID='64$3' restricted='1'><dc:title>The.Crazy.Ones.S01E01.HDTV.x264-2HD</dc:title><upnp:class>object.item.videoItem</upnp:class><dc:date>2013-12-13T01:21:10</dc:date><res size='160407354' duration='0:19:58.015' bitrate='133894' sampleFrequency='48000' nrAudioChannels='2' resolution='720x404' protocolInfo='http-get:*:video/mp4:DLNA.ORG_PN=AVC_MP4_HP_HD_AAC;DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=01700000000000000000000000000000'>http://192.168.1.3:8200/MediaItems/736.mp4</res>
//</item>
//"##;

    //doc_top_str + xml::escape(didl) + doc_bottom_str
    doc_top_str + (didl) + doc_bottom_str

}

fn update_db() {
    let path = "/home/ercan/rust/src/upnp/library.db";
    let db = &'static match sqlite::open(path) {
        Ok(db)  => db,
        Err(m)  => fail!(m)
    };

    let path : ~Path = box from_str("/home/ercan/StreamMedia/Movies").unwrap();
    let quote_escaped_str = str::replace(path.display().to_str(),"'","\\'");
    //TODO look here : https://stackoverflow.com/questions/8966667/valid-range-of-sqlite-rowid
    //id = 0 doesn't make sense in sqlite apparently
    let sql = "insert into library (parent_id,path) values (NULL, \"" +quote_escaped_str+ "\")";
    //println(sql);
    match db.exec(sql){
        Err(m) => fail!("Can't insert root dir into library. Error: " + m.to_str()),
        _   =>()
    }
    let  rowid = db.get_last_insert_rowid();

    scan(path, db, rowid);
}

fn scan(dir: ~Path, db: &Database, parent_id: i64) {
    let mut timer = std::io::timer::Timer::new().unwrap();

    //let db = 
    let mut dirs : ~[~Path] = ~[];
    let ls =  fs::readdir(dir);
    //do the root dir
    for node in ls.iter(){
        let mut is_dir = 0i;
        if node.is_dir() {
            is_dir = 1;
        }
        let quote_escaped_str = str::replace(node.display().to_str(),"'","\\'");
        let sql_str = "insert into library (is_dir, path, parent_id) values ("+ is_dir.to_str() +",\""+quote_escaped_str+"\", "+ parent_id.to_str() +")";
        match db.exec(sql_str){
            Err(m) => fail!("Can't insert item into library. Error: " + m.to_str()),
            _   =>()
        }
        if node.is_dir() {
            let  rowid = db.get_last_insert_rowid();
            scan(~node.clone(), db, rowid);
        }
    }
}

impl BrowseActionIn {

    fn new(soap: ~Element) -> BrowseActionIn {
        let mut name: ~str = ~"Browse";
        let mut object_id: i64 = 0;
        let mut browse_flag: ~str = ~"";
        let mut filter: ~str = ~"";
        let mut starting_index: i64 = 0;
        let mut requested_count: i64 = 0;
        let mut sort_criteria: ~str = ~"";

        let body = soap.child_with_name_and_ns("Body", Some(~"http://schemas.xmlsoap.org/soap/envelope/" )).unwrap();
        let action = match body.children[0].clone() {
            Element(e) => {
                for ch in e.children.iter(){
                    match *ch {
                        Element(ref e)  => {
                            match e.name {
                                ~"ObjectID" => object_id = from_str(e.content_str()).unwrap(),
                                ~"BrowseFlag" => browse_flag = e.content_str(),
                                ~"Filter" => filter = e.content_str(),
                                ~"StartingIndex" => starting_index = from_str(e.content_str()).unwrap(),
                                ~"RequestedCount" => requested_count = from_str(e.content_str()).unwrap(),
                                ~"SortCriteria" => sort_criteria = e.content_str(),
                                _           => ()

                            }

                        }
                        _           => (),

                    }
                }

            }
            _          => fail!("NO ELEMENT FOUND"),
        };

        BrowseActionIn{
            name: name,
            object_id: ~object_id,
            browse_flag: browse_flag,
            filter: filter,
            starting_index: ~starting_index,
            requested_count: ~requested_count,
            sort_criteria: sort_criteria,
        }
    }
}

impl ToStr for BrowseActionIn {
    fn to_str(&self) -> ~str {
        ~"name: " + self.name + "\n"
            +"object_id: " + self.object_id.to_str() + "\n"
            +"browse_flag: " + self.browse_flag + "\n"
            +"filter: " + self.filter + "\n"
            +"starting_index: " + self.starting_index.to_str() + "\n"
            +"requested_count: " + self.requested_count.to_str() + "\n"
            +"sort_criteria: " + self.sort_criteria + "\n"

    }
}

struct BrowseActionIn {
    name: ~str,
    object_id: ~i64,
    browse_flag: ~str,
    filter: ~str,
    starting_index: ~i64,
    requested_count: ~i64,
    sort_criteria: ~str,
}

struct BrowseActionOut {
    name: ~str,
    result: ~[~str],
    number_returned: i64,
    total_matches: i64,
    update_id: ~str
}

struct BrowseResult {
    items: ~[~str]
}
