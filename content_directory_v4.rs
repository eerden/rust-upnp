use super::{http,sqlite,template};

use sqlite::database::Database;
use sqlite::types::{BindArg,Text,Integer};
use std::hashmap::HashMap;
use std::io::fs;
use std::option::Option;
use std::path::GenericPath;
use std::str;
use super::http::Request;
use xml::Element;

pub struct ContentDirectory{
    db: Database,
    library_dir: ~str
}

impl ContentDirectory {
    //TODO: Use prepared statements.
    //TODO: Find out how other projects handle this.
    //
    //MediaHouse tries to get `Filename.{srt,psb,mpl,ssa,txt...}` for subtitles. For this, 
    //this function :
    //1 - first finds the full path of a file from the database using the id of the db entry.
    //2 - overrides the extension of the file with the requested filename no matter what the 
    //      path in db has.
    //3 - sends a Some(Path) to the requested file if found or None if not found. None should
    //      result in a 404.
    pub fn get_item_path(&self, url: ~str) -> Option<Path> {

        //This means Someone asked for the root directory. Bubble this up to a 404.
        //first part of the url is "/MediaItems/"
        //TODO:Don't hardcode this here.
        if url.len() < 12 { return None} 

        let path = match Path::new_opt(url.as_slice()) {
            Some(p) => p,
            None    => return None
        };

        let id = match path.filestem_str() {
            Some(i) => i,
            None    => return None
        };

        let requested_extension : &str = match path.extension_str() {
            Some(e) => e,
            None    => return None
        };

        let sql = "select path from library where id = " + id;

        let cursor : sqlite::cursor::Cursor = match self.db.prepare(sql, &None) {
            Err(e) => fail!("Error: {}", e.to_str()),
            Ok(c)   => c
        };

        let sql_result :  Option<HashMap<~str, BindArg>> = match cursor.step_row() {
            Ok(r) => r,
            Err(e) => fail!("Error: {}", e.to_str()),
        };

        let row_map : HashMap<~str, BindArg> = match sql_result {
            Some(m) => m,
            None    => fail!()
        };

        //This is the full path from the db for the given id.
        let path_from_db : &BindArg = row_map.get(&~"path");

        match *path_from_db {
            Text(ref p) => {
                let mut path = Path::new(p.as_slice());
                //Extension gets overriden here.
                path.set_extension(requested_extension);
                debug!("Sending: {}", path.display().to_str());
                Some(path)
            },
            _          => None
        }
    }


    //TODO: Write prepared statements.
    pub fn update_db(&self) {
        println!("Root dir is: {}", self.library_dir);
        println!("Updating library...");
        let drop_sql = "drop table if exists library; create table library(id integer primary key, parent_id integer, is_dir integer, child_count integer default 0, path string)";
        match self.db.exec(drop_sql){
            Err(m) => fail!("Can't recreate table. Error: {}", m.to_str()),
            _   =>()
        }

        let path : ~Path = box from_str(self.library_dir).unwrap();
        let quote_escaped_str = str::replace(path.display().to_str(),"'","\\'");
        let sql = "insert into library (parent_id,path) values (NULL, \"" +quote_escaped_str+ "\")";
        match self.db.exec(sql){
            Err(m) => fail!("Can't insert root dir into library. Error: " + m.to_str()),
            _   =>()
        }
        let  rowid = self.db.get_last_insert_rowid();
        self.db.exec("BEGIN TRANSACTION");
        self.scan(path, rowid);
        self.db.exec("COMMIT TRANSACTION");
        let  rowid = self.db.get_last_insert_rowid();
        println!("{} items added to the library.", rowid);
    }

    fn scan(&self, dir: ~Path, parent_id: i64) -> uint {
        let ls =  fs::readdir(dir);
        for node in ls.iter(){
            let mut is_dir = 0i;
            if node.is_dir() {
                is_dir = 1;
            }
            let quote_escaped_str = str::replace(node.display().to_str(),"'","\\'");
            let sql_str = "insert into library (is_dir, path, parent_id) values ("+ is_dir.to_str() +",\""+quote_escaped_str+"\", "+ parent_id.to_str() +")";
            match self.db.exec(sql_str){
                Err(m) => fail!("Can't insert item into library. Error: " + m.to_str()),
                _   =>()
            }
            if node.is_dir() {
                let  rowid = self.db.get_last_insert_rowid();
                let child_count = self.scan(~node.clone(), rowid);
                debug!("Got childcount {}", child_count);
                let sql_str_upd = "update library set child_count = " + child_count.to_str() + " where id =  " + rowid.to_str();
                debug!("SQL string is `{}`", sql_str_upd);
                match self.db.exec(sql_str_upd){
                    Err(m) => fail!("Can't update child count. Error: " + m.to_str()),
                    _   =>()
                }
            }
        }

        ls.len()
    }

    pub fn new(lib_dir: ~str) -> ContentDirectory {
        if lib_dir.len() == 0 {
            fail!("No directory supplied");
        }
        //let path = "/home/ercan/rust/src/upnp/library.db";
        let path = ":memory:";
        let db = match sqlite::open(path) {
            Ok(db)  => db,
            Err(m)  => fail!(m)
        };
        ContentDirectory{db: db, library_dir: lib_dir}
    }

    pub fn get_search_capabilities(){}
    pub fn get_sort_capabilities(){}
    pub fn get_feature_list(){} 
    pub fn get_system_update_id(){}
    pub fn get_service_reset_token(){}

    pub fn browse(&self, mut req: Request){
        let mut response : ~[u8] = ~[];
        let reqxml : Element = from_str(req.body.clone().unwrap()).unwrap();
        let result = self.get_content_as_xml(~reqxml).into_bytes();
        let xml_headers = http::default_xml_headers();
        let content_length_header = ("Content-Length: " + result.len().to_str() + "\r\n\r\n").into_bytes();
        response.push_all_move(xml_headers);
        response.push_all_move(content_length_header);
        response.push_all_move(result);
        debug!("{}", ::std::str::from_utf8(response));
        req.stream.write(response);
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
            Err(e) => fail!("Error: {}", e.to_str()),
            Ok(c)   => c
        };

        loop {
            let res = match cursor.step_row() {
                Ok(r) => r,
                Err(e) => fail!("Error while iterating over cursor. Error: {}", e.to_str()) 
            };
            let mut row_map = match res {
                Some(m) => m,
                None    => break
            };

            let path = row_map.pop(&~"path").unwrap();
            let id = row_map.pop(&~"id").unwrap();
            let is_dir = row_map.pop(&~"is_dir").unwrap();
            let parent_id = row_map.pop(&~"parent_id").unwrap();
            let child_count = row_map.pop(&~"child_count").unwrap();

            match (id,path,is_dir,child_count, parent_id) {
                (Integer(ref i), Text(ref p), Integer(0),Integer(0), Integer(p_id)) =>{
                    item_list.push(~ResultItem{id:*i as i64,is_dir: false,parent_id: p_id as i64, child_count: 0i64,  path: from_str(*p).unwrap()});
                },
                (Integer(ref i), Text(ref p), Integer(1),Integer(child_count), Integer(p_id)) =>{
                    item_list.push(~ResultItem{id:*i as i64,is_dir: true,parent_id: p_id as i64, child_count: child_count as i64, path: from_str(*p).unwrap()});
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
    child_count: i64,
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
    let mut template = template::new("./xml_templates/browse.xml");

    mid.push(~r#"<DIDL-Lite xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:upnp="urn:schemas-upnp-org:metadata-1-0/upnp/" xmlns="urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/" xmlns:dlna="urn:schemas-dlna-org:metadata-1-0/">"#);

    let number_returned = list.len().to_str();
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
    let mut out : ~str;
    if item.is_dir{
        let open_tag = "<container id=\""+ item.id.to_str() +"\" parentID=\"" + item.parent_id.to_str() + "\" childCount=\""+ item.child_count.to_str() +"\" restricted=\"1\">";
        let class = "<upnp:class>object.container.storageFolder</upnp:class>";
        let title = "<dc:title>" + item.path.filename_str().unwrap() + "</dc:title>";
        let storage_used = "<upnp:storageUsed>-1</upnp:storageUsed>";
        let close_tag = "</container>";
        out = open_tag + title + class + storage_used + close_tag;
    } else {
        let extension = match item.path.extension_str() {
            Some(e) => e,
            None    => "",
        };
        let open_tag = "<item id=\""+ item.id.to_str() +"\" parentID=\"" + item.parent_id.to_str() + "\" restricted=\"1\">";
        let title = "<dc:title>" + item.path.filename_str().unwrap() + "</dc:title>";
        let res =  r#"<res protocolInfo="http-get:*:video/x-msvideo:*">http://192.168.1.3:8900/MediaItems/"#+ item.id.to_str() + "." + extension + "</res>";
        let class = "<upnp:class>object.item.videoItem</upnp:class>";
        let close_tag = "</item>";
        out = open_tag + title + res + class + close_tag;
    }
    out
}

impl BrowseActionIn {

    fn new(soap: ~Element) -> BrowseActionIn {
        let name: ~str = ~"Browse";
        let mut object_id: i64 = 0;
        let mut browse_flag: ~str = ~"";
        let mut filter: ~str = ~"";
        let mut starting_index: i64 = 0;
        let mut requested_count: i64 = 0;
        let mut sort_criteria: ~str = ~"";

        let body = soap.child_with_name_and_ns("Body", Some(~"http://schemas.xmlsoap.org/soap/envelope/" )).unwrap();
        match body.children[0].clone() {
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

fn escape_didl(mut s: ~str) -> ~str{
    s = s.replace("&", "&amp;amp;");
    s = s.replace("<", "&lt;");
    s = s.replace(">", "&gt;");
    s = s.replace("\"", "&quot;");
    s
}
