#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate reqwest;
extern crate rocket;
extern crate sass_rs;
extern crate toml;

extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

mod category;
mod group;
mod rust_version;

use group::*;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use rocket::response::NamedFile;
use rocket::Error;
use rocket_contrib::Template;

use sass_rs::{compile_file, Options};

use category::Category;

#[derive(Serialize)]
struct Context {
    page: String,
    title: String,
    parent: String,
    is_landing: bool,
    data: Option<HashMap<String, Vec<Group>>>,
}

#[get("/")]
fn index() -> Template {
    #[derive(Serialize)]
    struct Context {
        page: String,
        title: String,
        parent: String,
        is_landing: bool,
        data: Option<HashMap<String, Vec<Group>>>,
        rust_version: String,
    }

    let page = "index".to_string();
    let title = format!("Rust - {}", page).to_string();

    let context = Context {
        page: "index".to_string(),
        title: title,
        parent: "layout".to_string(),
        is_landing: true,
        data: None,
        rust_version: rust_version::rust_version()
            .map_or(String::new(), |v| format!("Version {}", v)),
    };
    Template::render(page, &context)
}

#[get("/static/<file..>", rank = 1)]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

#[get("/<category>")]
fn category(category: Category) -> Template {
    let page = category.index();
    let title = format!("Rust - {}", page).to_string();
    let context = Context {
        page: category.name().to_string(),
        title: title,
        parent: "layout".to_string(),
        is_landing: false,
        data: load_governance_data(&page),
    };
    Template::render(page, &context)
}

fn load_governance_data(page: &str) -> Option<HashMap<String, Vec<Group>>> {
    let mut map: HashMap<String, Vec<Group>> = HashMap::new();
    if page == "governance/index" {
        map.insert(
            GroupType::Team.to_string(),
            group::get_toplevel_data(&GroupType::Team).expect("couldn't get teams data"),
        );
        map.insert(
            GroupType::WorkingGroup.to_string(),
            group::get_toplevel_data(&GroupType::WorkingGroup).expect("couldn't get wgs data"),
        );
        return Some(map);
    }
    None
}

#[get("/governance/<t>/<subject>", rank = 2)]
fn team(t: String, subject: String) -> Template {
    let page = "governance/group".to_string();
    let title = format!("Rust - {}", page).to_string();
    let t = get_type_from_string(&t).expect("couldnt figure out group type from path string");
    let context = Context {
        page: "farts".to_string(),
        title: title,
        parent: "layout".to_string(),
        is_landing: false,
        data: load_group_data(t, &subject),
    };
    Template::render(page, &context)
}

fn get_type_from_string(s: &str) -> Result<GroupType, Error> {
    match s {
        "wgs" => Ok(GroupType::WorkingGroup),
        "teams" => Ok(GroupType::Team),
        "peers" => Ok(GroupType::Peer),
        "shepards" => Ok(GroupType::Shepard),
        _ => Err(Error::Internal),
    }
}

fn load_group_data(t: GroupType, group: &str) -> Option<HashMap<String, Vec<Group>>> {
    let mut map: HashMap<String, Vec<Group>> = HashMap::new();
    map.insert(
        "info".to_string(),
        vec![group::get_info(&t, group).expect("couldn't get group info")],
    );
    let subteams =
        group::get_subs_data(&t, group, &GroupType::Team).expect("couldn't get subteams data");
    if subteams.len() > 0 {
        map.insert("subteams".to_string(), subteams);
    }
    let subwgs = group::get_subs_data(&t, group, &GroupType::WorkingGroup)
        .expect("couldn't get subwgs data");
    if subwgs.len() > 0 {
        map.insert("subwgs".to_string(), subwgs);
    }
    return Some(map);
}

#[get("/<category>/<subject>", rank = 4)]
fn subject(category: Category, subject: String) -> Template {
    let page = format!("{}/{}", category.name(), subject.as_str()).to_string();
    let title = format!("Rust - {}", page).to_string();
    let context = Context {
        page: subject,
        title: title,
        parent: "layout".to_string(),
        is_landing: false,
        data: None,
    };
    Template::render(page, &context)
}

#[catch(404)]
fn not_found() -> Template {
    let page = "404";
    let title = format!("Rust - {}", page).to_string();
    let context = Context {
        page: "404".to_string(),
        title: title,
        parent: "layout".to_string(),
        is_landing: false,
        data: None,
    };
    Template::render(page, &context)
}

#[catch(500)]
fn catch_error() -> Template {
    not_found()
}

fn compile_sass() {
    let scss = "./src/styles/app.scss";
    let css = compile_file(scss, Options::default()).expect("couldn't compile sass");
    let mut file = File::create("./static/styles/app.css").expect("couldn't make css file");
    file.write_all(&css.into_bytes())
        .expect("couldn't write css file");
}

fn main() {
    compile_sass();
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index, category, team, subject, files])
        .catch(catchers![not_found, catch_error])
        .launch();
}
