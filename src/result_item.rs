use sqlite3::types::{Text,Integer};
use sqlite3::cursor::Cursor;

#[deriving(Clone)]
pub struct ResultItem {
    id: i64,
    is_dir: bool,
    parent_id: i64,
    child_count: i64,
    path: Path,
    mime: Option<~str>
}

impl ResultItem {
    pub fn to_didl(&self) -> ~str {

        let mut out : ~str;
        let filename_str = match self.path.filename_str() {
            Some(s) => s,
            None    => fail!("Can't produce a filename string from path.")
        };

        if self.is_dir {
            let open_tag = "<container id=\""+ self.id.to_str() +"\" parentID=\"" + self.parent_id.to_str()
                + "\" childCount=\""+ self.child_count.to_str() +"\" restricted=\"1\">";

            let class = "<upnp:class>object.container.storageFolder</upnp:class>";
            let title = "<dc:title>" + filename_str + "</dc:title>";
            let storage_used = "<upnp:storageUsed>-1</upnp:storageUsed>";
            let close_tag = "</container>";
            out = open_tag + title + class + storage_used + close_tag;

        } else {

            let extension = match self.path.extension_str() {
                Some(e) => e,
                None    => "",
            };

            let mime = match self.mime {
                Some(ref m) => m.as_slice(),
                None    => fail!("Can't retreive ResultItem::mime"),
            };

            let open_tag = "<item id=\""+ self.id.to_str() +"\" parentID=\"" + self.parent_id.to_str() + "\" restricted=\"1\">";
            let title = "<dc:title>" + filename_str + "</dc:title>";
            let res =  r#"<res protocolInfo="http-get:*:"# + mime + r#":*">http://192.168.1.3:8900/MediaItems/"#+ self.id.to_str() + "." + extension + "</res>";
            let class = "<upnp:class>object.item.videoItem</upnp:class>";
            let close_tag = "</item>";
            out = open_tag + title + res + class + close_tag;
        }
        out
    }
}

pub struct ResultItemIterator <'db> {
    cursor: Cursor<'db>,
}

impl <'db> ResultItemIterator <'db> {
    pub fn new<'db> (cursor: Cursor<'db> ) -> ResultItemIterator<'db> {
         ResultItemIterator { cursor: cursor }
    }
}

impl  <'db> Iterator <ResultItem>  for ResultItemIterator <'db> {
    fn next(&mut self) -> Option<ResultItem> {

        let res = match self.cursor.step_row() {
            Ok(r) => r,
            Err(e) => fail!("Error while iterating over cursor. Error: {}", e.to_str()) 
        };

        //Exit if there's no result.
        let mut row_map = match res {
            Some(m) => m,
            None    => return None,
        };

        let path = match row_map.pop(&~"path") {
            Some(p) => p,
            None => fail!("Can't find column `path` in row")
        };

        let id = match row_map.pop(&~"id") {
            Some(i) => i,
            None => fail!("Can't find column `id` in row ")
        };

        let is_dir = match row_map.pop(&~"is_dir") {
            Some(d) => d, None => fail!("Can't find column `is_dir` in row ")
        };

        let parent_id = match row_map.pop(&~"parent_id") {
            Some(pi) => pi, None => fail!("Can't find column `parent_id` in row ")
        };

        let child_count = match row_map.pop(&~"child_count") {
            Some(cc) => cc, None => fail!("Can't find column `child_count` in row ")
        };

        let mime = match row_map.pop(&~"mime") {
            Some(m) => m, None => fail!("Can't find column `mime` in row ")
        };

        //TODO: Check if the file actually exists. 
        //Otherwise this will cause problems if the MediaServer::update() is not run everytime, 
        //which is likely if a file is used for db insetad of memory.
        match (id,path,is_dir,child_count, parent_id, mime) {

            (Integer(ref id), Text(ref path_str), Integer(0),Integer(0), Integer(p_id), Text(ref m)) => {
                let path : Path = match from_str(*path_str) {
                    Some(p) => p,
                    None    => fail!("Can't make a Path using from_str() with the value of `path` column.") 
                };
                    let out = ResultItem {
                        id:*id as i64,
                        is_dir: false,
                        parent_id: p_id as i64,
                        child_count: 0i64,
                        path: path,
                        mime:Some(m.clone())
                    };
                    Some(out)
            },
            (Integer(ref i), Text(ref p), Integer(1),Integer(child_count), Integer(p_id), _) => {
                let path : Path = match from_str(*p) {
                    Some(p) => p,
                    None    => fail!("Can't make a Path using from_str() with the value of `path` column.") 
                };
                   let out =  ResultItem {
                        id:*i as i64,
                        is_dir: true,
                        parent_id: p_id as i64,
                        child_count: child_count as i64,
                        path: path,
                        mime: None
                    };
                   Some(out)
            },
            _       => fail!("Could not create a result item."),
        }
    }
}

