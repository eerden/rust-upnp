// Super simple template.
// TODO: Switch to something like rust-mustache.
// TODO: Write a new constructor for tests.

extern mod std;
use std::hashmap;
use std::str;
use std::io::File;

//Containers
pub struct Tag {
    start:  uint,
    end:    uint,
    name:   ~str
}

pub struct Template {
    name:     ~str,
    content:  ~str,
    vars:     hashmap::HashMap<~str, ~str>,
    tags:     ~[Tag]
}

// TODO:Check for illegal characters.
// TODO:Alternative constructor with different tags.

// Return variable name in tag.
// Finds the variable name in the given string, and checks for _some_ errors.
fn extract_tag_name(content: &str, otag_pos: uint , ctag_pos: uint ) -> ~str {
    let tag_content = content.slice_chars(otag_pos+2, ctag_pos);
    let var_name = tag_content.trim();
    check_tag_name(var_name.to_owned());
    let has_spaces = var_name.find_str(" ");
    match has_spaces {
        None  => {} ,
        _     => fail!("Spaces in variable names")
    }
    var_name.to_owned()
}

// Check if variable bame is legal, fail if not.
fn check_tag_name(var_name :~str) {
    let bad_chars = ~["!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "=","+"];
    for bad in bad_chars.iter() {
        match var_name.find_str(*bad) {
            None  => false,
            _     => fail!("Illegal character found in variable name."),
        };
    }

}

// Find all {{tag}} like items.
// Nesting things like "{{ {{ " or " }} }}"  is not allowed.
fn scan_tags(content : &str) -> ~[Tag] {
    let mut tags : ~[Tag] = ~[];
    //Tag open/close symbols
    let otag = "{{";
    let ctag = "}}";

    // This is set to true when an otag is found(while inside a tag)
    // and set to false when a ctag is found.
    let mut tag_open = false;

    let mut otag_pos :uint = 0;
    let mut ctag_pos :uint;
    let mut var_name :~str;

    //Main loop going scanning the file for tags
    let mut i = 0;
    while i < content.len() - 1 {
        let cursor = content.slice_chars(i, i+2);

        //There shouldn't be a ctag if a tag is not open.
        if !tag_open && cursor == ctag {
            fail!("Wrong tag: expected otag, found ctag");
        }

        if tag_open && cursor == otag {
            fail!("Wrong tag: expected ctag, found otag");
        }

        //Look for an opening tag
        if !tag_open && cursor == otag {
            otag_pos = i;
            tag_open = true;
        }

        //Look for a closing tag
        if tag_open && cursor == ctag {
            ctag_pos = i;
            tag_open = false;
            var_name = extract_tag_name(content, otag_pos, ctag_pos);
            tags.push(Tag { start:otag_pos, end:ctag_pos, name: var_name.clone() });
        }

        i+=1;
    }
    tags
}

///Default constructor.
///
///
///Create a Template from path "templates/default.rst"
pub fn new(filepath : &str) -> Template {
    let vars = hashmap::HashMap::new();
    let path = from_str(filepath).unwrap();

    let mut file = File::open(&path);
    let content = file.read_to_str(); 

    let tags = scan_tags(content.as_slice());
    return Template { name:~"default", content:content, vars:vars, tags: tags }
}

impl Template {
    // Returns the contents of the template file as a ~string.
    pub fn file_to_str(&self) -> ~str {
        self.content.clone()
    }


    /// Set a variable to be used in the template
    /// 
    /// #Arguments
    /// 
    /// `K` - String used as key in hashmap `vars`.
    /// 
    /// `V` - String value for key `K`.
    /// 
    /// #Failure
    /// 
    /// Fail if a variable with the same name is already set.
    pub fn set_var(&mut self, K: &str, V: &str) {
        if self.vars.contains_key_equiv(&K) {
            fail!("There's already a variable with the key: `" + K + "`");
        }
        self.vars.insert(K.to_owned(),V.to_owned());
    }

    ///
    /// Get the final output.
    ///
    /// Replaces missing template variables with empty string instead of crashing.
    ///
    /// #Return value
    ///
    /// A ~string containing the final output.
    ///
    ///
    pub fn render(&self) -> ~str {
        let mut output = ~"";
        let mut last_point = 0;

        for t in self.tags.iter() {
            let left   = self.content.slice(last_point, t.start);
            let var    = match self.vars.find(&t.name).clone() {
                Some(val) => val.to_str(),
                None => ~""
            };
            output = output +  left + var;
            last_point = t.end + 2;
        }
        // Text after the last tag.
        let right  = self.content.slice(last_point, self.content.len());
        output = output + right;
        output
    }
}

#[test]
//fn testing_vars() {
    //use self::Template;
    //let mut t = Template::new(~"hello", ~"templates");
    //t.set_var("planet", "Earth");
    //t.set_var("galaxy", "Milky Way");

    //assert!(~"Earth" == *t.vars.get(&~"planet"));
    //assert!(~"Milky Way" == *t.vars.get(&~"galaxy"));
//}

#[test]
fn test_scan_tags() {
    let template = ~"We are on planet {{planet}} in a galaxy called {{galaxy}}. Our planet orbits a star called {{star}}, also known as {{star_world_name}}.";
    let tags = scan_tags(template);

    assert!(tags[0].name == ~"planet");
    assert!(tags[0].start == 17u);
    assert!(tags[0].end == 25u);

    assert!(tags[1].name == ~"galaxy");
    assert!(tags[1].start == 47u);
    assert!(tags[1].end == 55u);

    assert!(tags[2].name == ~"star");
    assert!(tags[2].start == 91u);
    assert!(tags[2].end == 97u);

    assert!(tags[3].name == ~"star_world_name");
    assert!(tags[3].start == 115u);
    assert!(tags[3].end == 132u);
}

#[test]
fn test_extract_tags() {
    assert!(~"name" == extract_tag_name("{{name}}", 0, 6));
    assert!(~"name" == extract_tag_name("{{ name}}", 0, 7));
    assert!(~"name" == extract_tag_name("{{name }}", 0, 7));
    assert!(~"name" == extract_tag_name("{{ name }}", 0, 8));
}

#[test]
#[should_fail]
fn test_var_with_space_extraction_fail() {
    assert!(~"name" == extract_tag_name("{{ na me }}", 0, 8));
}

#[test]
#[should_fail]
fn test_check_tag_name_fail1() {
    check_tag_name(~"planet!");
}

#[test]
#[should_fail]
fn test_check_tag_name_fail2() {
    check_tag_name(~"planet@");
}

#[test]
#[should_fail]
fn test_check_tag_name_fail3() {
    check_tag_name(~"planet#");
}
