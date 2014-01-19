use super::http;
use super::template;
use super::xml;
use std::io::stdio::println;
use super::http::Request;
use xml::{Element,CharacterNode};
use super::sqlite;
use sqlite::database::Database;
use sqlite::types::{SQLITE_ROW,SQLITE_ERROR,BindArg,Text,Integer};
use std::str;
use std::io::fs;
use sqlite::database::Database;

pub struct ContentDirectory{
    db: Database
}

impl ContentDirectory {

    pub fn new() -> ContentDirectory {
        let path = "/home/ercan/rust/src/upnp/library.db";
        let db = match sqlite::open(path) {
            Ok(db)  => db,
            Err(m)  => fail!(m)
        };

        ContentDirectory{db: db}
    }

    pub fn get_search_capabilities(){}
    pub fn get_sort_capabilities(){}
    pub fn get_feature_list(){} 
    pub fn get_system_update_id(){}
    pub fn get_service_reset_token(){}
    pub fn browse(&self, mut req: Request){
        let mut response : ~[u8] = ~[];
        let mut reqxml : Element = from_str(req.body.clone().unwrap()).unwrap();

        let result = self.get_content_as_xml(~reqxml).into_bytes();

        let xml_headers = http::default_xml_headers();
        let content_length_header = ("Content-Length: " + result.len().to_str() + "\r\n\r\n").into_bytes();
        response.push_all_move(xml_headers);
        response.push_all_move(content_length_header);
        response.push_all_move(result);
        debug!("{}", ::std::str::from_utf8(response));
        req.stream.write(response);

    }

    pub fn get_item_url(id: int) -> ~str{

        let sql = "select path from library where id = " + id.to_str() ;

        let path = "/home/ercan/rust/src/upnp/library.db";
        let db = match sqlite::open(path) {
            Ok(db)  => db,
            Err(m)  => fail!(m)
        };

        let cursor = match db.prepare(sql, &None) {
            Err(e) => fail!(),
            Ok(c)   => c
        };

        let sql_result = match cursor.step_row() {
            Ok(r) => r,
            Err(e) => fail!() 
        };

        let mut row_map = match sql_result {
            Some(m) => m,
            None    => fail!()
        };

        let path = row_map.get(&~"path");

        match *path {
            Text(ref t) => t.clone(),
            _       => fail!()

        }
    }

    //TODO: Write prepared statements.
    fn get_content_as_xml(&self, xml_action: ~Element) -> ~str {
    let mut item_list : ~[~ResultItem] = ~[];

    let action = BrowseActionIn::new(xml_action);

    // 0 means root object is requested. 
    // It's  not a good idea to have 0 as rowid in sqlite.
    let parent_id = match action.object_id.clone() {
        ~0i64 => ~1,
        x => x
    };

    let sql = "select * from library where parent_id = " + parent_id.to_str() ;

    let cursor = match self.db.prepare(sql, &None) {
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

}

#[deriving(Clone)]
struct ResultItem{
    id: i64,
    is_dir: bool,
    parent_id: i64,
    path: Path
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

fn content_xml(list: ~[~ResultItem]) -> ~str{
    let mut mid : ~[~str] = ~[];
    let mut template = template::new("/home/ercan/rust/src/upnp/xml_templates/browse.xml");

    mid.push(~r#"&lt;DIDL-Lite xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:upnp="urn:schemas-upnp-org:metadata-1-0/upnp/" xmlns="urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/" xmlns:dlna="urn:schemas-dlna-org:metadata-1-0/"&gt;"#);

    let number_returned = list.len().to_str();
    let xml_top = ~r#"<?xml version="1.0" encoding="UTF-8"?>
    <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
    <s:Body>
    <u:BrowseResponse xmlns:u="urn:schemas-upnp-org:service:ContentDirectory:4">
    <Result>
    "#;
    let xml_bottom = ~"</Result><NumberReturned>" + number_returned +"</NumberReturned><TotalMatches>"+ number_returned +"</TotalMatches><UpdateID>17</UpdateID></u:BrowseResponse></s:Body></s:Envelope>";

    for item in list.iter() {
        mid.push(make_didl_item(item.clone()));
    }

    template.set_var("result", escape_didl(mid.concat()));
    template.set_var("number_returned", number_returned);
    template.set_var("total_matches", number_returned);
    template.render()
}


//TODO: Find a way to use RustyXML more.
fn make_didl_item(item: ~ResultItem) -> ~str {
    let mut out : ~str = ~"";
    if item.is_dir{
        let open_tag = "<container id=\""+ item.id.to_str() +"\" parentID=\"" + item.parent_id.to_str() + "\" restricted=\"1\">";
        let class = "<upnp:class>object.container.storageFolder</upnp:class>";
        let title = "<dc:title>" + item.path.filename_str().unwrap() + "</dc:title>";
        let storage_used = "<upnp:storageUsed>-1</upnp:storageUsed>";
        let close_tag = "</container>";
        out = open_tag + title + class + storage_used + close_tag;
    } else {
        let open_tag = "<item id=\""+ item.id.to_str() +"\" parentID=\"" + item.parent_id.to_str() + "\" restricted=\"1\">";
        let title = "<dc:title>" + item.path.filename_str().unwrap() + "</dc:title>";
        let res =  r#"<res protocolInfo="http-get:*:video/x-msvideo:*">http://192.168.1.3:8900/MediaItems/"#+ item.id.to_str() +r#".avi</res>"#;
        let class = "<upnp:class>object.item.videoItem</upnp:class>";
        let close_tag = "</item>";
        out = open_tag + title + res + class + close_tag;
    }

    out
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


//TODO: Write prepared statements.
//TODO: Make the function return the number of children and update the dir.
//New column in db will be needed.(`childCount='xx'` in xml)
fn scan(dir: ~Path, db: &Database, parent_id: i64) {
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

fn escape_didl(mut s: ~str) -> ~str{
    s = s.replace("<", "&lt;");
    s = s.replace(">", "&gt;");
    s
}

//TODO: Write prepared statements.
pub fn update_db() {
    let path = "/home/ercan/rust/src/upnp/library.db";
    let db = &'static match sqlite::open(path) {
        Ok(db)  => db,
        Err(m)  => fail!(m)
    };
    let drop_sql = "drop table if exists library; create table library(id integer primary key, parent_id integer, is_dir integer, path string)";

    match db.exec(drop_sql){
        Err(m) => fail!("Can't recreate table."),
        _   =>()
    }

    let path : ~Path = box from_str("/home/ercan/StreamMedia/Movies/").unwrap();
    let quote_escaped_str = str::replace(path.display().to_str(),"'","\\'");
    let sql = "insert into library (parent_id,path) values (NULL, \"" +quote_escaped_str+ "\")";
    match db.exec(sql){
        Err(m) => fail!("Can't insert root dir into library. Error: " + m.to_str()),
        _   =>()
    }
    let  rowid = db.get_last_insert_rowid();

    scan(path, db, rowid);
}

