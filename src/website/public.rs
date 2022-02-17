// Public: fuer Seiten, die fuer die Offentlichkeit direkt zugaenglich sein sollen (also nur html Seiten)

use actix_web::{ Responder };
use crate::website::{ return_ok_or_error, read_static_page };

pub async fn index() -> impl Responder {
    return_ok_or_error(read_static_page("index.html"))
}