use actix_cors::Cors;
use actix_web::body::BoxBody;
use actix_web::http::header;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{
    get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError, Result,
};

use serde::{Deserialize, Serialize};

use std::fmt::Display;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
struct Todo {
    id: u32,
    description: String,
}

#[derive(Debug, Serialize)]
struct ErrorNoId {
    id: u32,
    err: String,
}

struct AppState {
    todos: Mutex<Vec<Todo>>,
}

impl Responder for Todo {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let res_body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(res_body)
    }
}

impl ResponseError for ErrorNoId {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let body = serde_json::to_string(&self).unwrap();
        let res = HttpResponse::new(self.status_code());
        res.set_body(BoxBody::new(body))
    }
}

impl Display for ErrorNoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[get("/todos")]
async fn get_todos(data: web::Data<AppState>) -> impl Responder {
    let todos = data.todos.lock().unwrap();

    let response = serde_json::to_string(&(*todos)).unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response)
}

#[post("/todo")]
async fn post_todo(req: web::Json<Todo>, data: web::Data<AppState>) -> impl Responder {
    let new_todo = Todo {
        id: req.id,
        description: String::from(&req.description),
    };

    let mut todos = data.todos.lock().unwrap();

    let response = serde_json::to_string(&new_todo).unwrap();

    todos.push(new_todo);
    HttpResponse::Created()
        .content_type(ContentType::json())
        .body(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        todos: Mutex::new(vec![
            Todo {
                id: 1,
                description: String::from("Faire une soupe à l'oignon"),
            },
            Todo {
                id: 2,
                description: String::from("Payer facture electricité"),
            },
        ]),
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:1234")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_header(header::CONTENT_TYPE);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .service(get_todos)
            // .service(check_todo)
            .service(post_todo)
        // .service(remove)
        // .service(remove_all)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
