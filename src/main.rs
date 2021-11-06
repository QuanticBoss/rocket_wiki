/*
cargo +nightly run
http://localhost:8000
*/

#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
// use rocket_dyn_templates::Template;
use rocket::response::content;
use rocket::response::Redirect;
use rocket::response::content::Json;
use rocket::request::Form;
use rocket_contrib::templates::Template;
use serde::Serialize;

use std::ffi::{OsStr, OsString};

use std::io;
use std::fs;
use std::path::{Path, PathBuf};

fn format_html(title: &str, body: &str) -> String {
    format!("<!DOCTYPE html><html lang='fr'>
<head>
<meta charset='utf-8' />
<title> {} </title>
</head>
<body> {} </body>
</html>
    ", title, body)
}

#[derive(FromForm, Debug, Serialize)]
struct Page {
    title: String,
    body: String,
}

impl Page {
    fn save(&self) -> io::Result<()> {
        let filename = format!("{}.txt", self.title);
        fs::write(filename, &self.body)?;
        Ok(())
    }

    fn load(title: &str) -> io::Result<Page> {
        let filename = format!("{}.txt", title);
        let body = fs::read_to_string(filename)?;
        let title = title.to_string();
        Ok(Page { title, body })
    }
}

#[test]
fn test_page() {
    let p1 = Page { title: "test".to_string(), body: "This is a test page!".to_string() };
    p1.save().expect("Saving page");
    let p2 = Page::load("test").expect("Loading page");
    println!("{}", p2.body);
    assert_eq!(p2.body, "This is a test page!".to_string());
}


// #[get("/<title>")]
// fn pages(title: String) -> content::Html<String> {
//     let body = format!("<h1>Hi my name's is {}</h1>", title);
//     //format_html(&title, &body)
//     content::Html(body)
// }

#[get("/<title>")]
// fn pages(title: String) -> String {
//     let body = format!("<h1>Hi my name's is {}</h1>", title);
//     // body
//     // format_html(&title, &body)
//     content::Html(body)
// }

fn page(title: String) -> content::Html<String> {
    let body = format!("<h1>Hi my name's is {}</h1>", title);
    content::Html(body)
}

// #[get("/view/<title>")]
// fn view_page(title: String) -> content::Html<String> {
//     let html = match Page::load(&title){
//         Ok(p)  => format!("<h1>{}</h1> <div>{}</div>", p.title, p.body),
//         Err(e) => format!("<h1>Loading Error</h1> <div>{:?}</div>", e),
//     };
//     content::Html(html)
// }

#[get("/view/<title>")]
fn view_page(title: String) -> Template {
    let page = match Page::load(&title){
        Ok(p)  => p,
        Err(e) => Page { title, body: "page not found".to_string() }
    };
    Template::render("view", page)
}

#[get("/edit/<title>")]
fn edit_page(title: String) -> Template {
    let page = match Page::load(&title){
        Ok(p)  => p,
        Err(e) => Page { title, body: String::new() },
    };    
    Template::render("edit", page)
}


#[post("/save", data = "<page_form>")]
fn save_page(page_form: Form<Page>) -> Redirect {
    let page: Page = page_form.into_inner();
    page.save();
    Redirect::to(uri!(view_page: page.title))
}


// #[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/")]
fn list() -> content::Html<String> {
    let mut body = format!("<h1> Contenu du site </h1>");
    if let Ok(list) = search_files(".", "txt") {
        for f in &list {
            let l = format!("<p><a href='/view/{}'>{}</a></p>", &f, &f);
            body.push_str(&l);
        }
    }
    content::Html(body)
}

fn search_files(dir: &str, ext: &str) -> std::io::Result<Vec<String>> {
    let mut list = vec![];
    for entry in fs::read_dir(dir)? {
        let file = entry?;     
        let path = file.path();
        let filename = file.file_name().into_string().unwrap();
        let extension = path.extension().unwrap_or(OsStr::new("")).to_os_string().into_string().unwrap();

        if file.file_type()?.is_file() && extension == ext {
            let len = filename.len();
            let fname = filename[..len-4].to_string();
            list.push(fname);
        }
    }
    list.sort();
    // println!("{:?}", list);
    Ok(list)
}


fn main() {
    let mut adress = "localhost".to_string();
    let mut port = 80;

    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        2 => port = args[1].parse::<u16>().unwrap_or(80),
        n if n >= 3 => { adress = args[1].to_string(); port = args[2].parse::<u16>().unwrap_or(80); },
        _ => (),
    };

    use rocket::config::{Config, Environment};
    let config = Config::build(Environment::Staging)
        // .address("127.0.0.1")
        .address(&adress)
        .port(port)
        .unwrap();

    rocket::custom(config)
    // rocket::ignite()
        // .mount("/", routes![index, view_page, edit_page, save_page])
        .mount("/", routes![list, view_page, edit_page, save_page])
    //  .register(catchers![not_found])
        .attach(Template::fairing())
        .launch();
}

/*
#[get("/")]
fn index() -> io::Result<NamedFile> {
  NamedFile::open("static/index.html")
}

#[get("/files/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
  NamedFile::open(Path::new("static/").join(file)).ok()
}

#[get("/api/<name>")]
fn api(name: String) -> String {
  format!("Hello, {}!", name.as_str())
}

#[catch(404)]
fn not_found(req: &Request) -> String {
  format!("Sorry, '{}' is not a valid path.", req.uri())
}

fn main() {
  rocket::ignite()
    .mount("/", routes![index, api, files])
    .register(catchers![not_found])
    .launch();
}
*/


