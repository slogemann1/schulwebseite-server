// Website: allgemeine Helper-Funktionen, die fuer die gesamte Webseite nuetzlich sind, und stellt den Server auf

use std::fs;
use std::io::Error;
use actix_web::{ web, App, HttpServer, HttpResponse, Responder };
use actix_web::body::Body;
use crate::file_path_from_root;

mod public;

pub async fn run() -> std::io::Result<()> {
    // Server definieren und starten
    HttpServer::new(|| {
        App::new()
        // Public
        .service(web::resource("/index.html").route(web::get().to(public::index)))
    })
    .bind("0.0.0.0:80")? // Server nimmt alle Anfragen auf port 80 
    .run()
    .await
}

// Gibt eine HTTP-Antwort wieder => Body mit dem Inhalt des Results, falls Ok, sonst die Seite mit einer Fehlermeldung
fn return_ok_or_error<B: Into<Body>>(body: Result<B, Error>) -> impl Responder {
    match body {
        Ok(body) => HttpResponse::Ok().body(body),
        Err(_) => HttpResponse::InternalServerError().body(get_static_page("internal_error.html").unwrap()) // TODO: Fehlermeldung-Seiten sollten am Anfang geladen werden, falls das Problem beim laden der Dateien liegt
    }
}

// Laedt Seite, die nicht dynamisch erstellt wird
// Seiten koennen nur Text-Dateien sein
fn get_static_page(file_name: &str) -> std::io::Result<String> {
    read_text_file(&file_path_from_root("website/static", file_name))
}

// Liest Text-Datei ein
fn read_text_file(file_name: &str) -> std::io::Result<String> {
    fs::read_to_string(file_name) // TODO: Sollte spaeter geaendert werden, sodass Dateien nicht gleichzeitig gelesen werden
}