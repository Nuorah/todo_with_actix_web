use actix_cors::Cors;
use actix_web::body::BoxBody;
use actix_web::http::header;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{
    get, post, put, web, App, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError,
    Result,
};

use serde::{Deserialize, Serialize};

use std::fmt::Display;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
struct Todo {
    id: u32,
    description: String,
    done: bool,
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

#[get("/todo/{id}")]
async fn get_todo(id: web::Path<u32>, data: web::Data<AppState>) -> Result<Todo, ErrorNoId> {
    let todo_id: u32 = *id;
    let todos = data.todos.lock().unwrap();

    let todo: Vec<_> = todos.iter().filter(|x| x.id == todo_id).collect();

    if !todo.is_empty() {
        Ok(Todo {
            id: todo[0].id,
            description: String::from(&todo[0].description),
            done: todo[0].done,
        })
    } else {
        let response = ErrorNoId {
            id: todo_id,
            err: String::from("todo not found"),
        };
        Err(response)
    }
}

#[put("/todo/{id}")]
async fn check_todo(
    id: web::Path<u32>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, ErrorNoId> {
    let todo_id: u32 = *id;

    let mut todos = data.todos.lock().unwrap();

    let id_index = todos.iter().position(|x| x.id == todo_id);

    match id_index {
        Some(id) => {
            let new_todo = Todo {
                id: todo_id,
                description: String::from(&todos[id].description),
                done: dbg!(!&todos[id].done),
            };
            let response = serde_json::to_string(&new_todo).unwrap();
            todos[id] = new_todo;
            Ok(HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(response))
        }
        None => {
            let response = ErrorNoId {
                id: todo_id,
                err: String::from("todo not found"),
            };
            Err(response)
        }
    }
}

#[post("/todo")]
async fn post_todo(req: web::Json<Todo>, data: web::Data<AppState>) -> impl Responder {
    let mut todos = data.todos.lock().unwrap();

    let new_todo = Todo {
        id: todos.iter().map(|t| t.id).max().unwrap_or_default() + 1,
        description: String::from(&req.description),
        done: false,
    };

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
                done: false,
            },
            Todo {
                id: 2,
                description: String::from("Payer facture electricité"),
                done: true,
            },
            Todo {
                id: 3,
                description: String::from("Faire les courses"),
                done: false,
            },
        ]),
    });

    HttpServer::new(move || {
        let cors = Cors::permissive();
        //.allowed_origin("http://localhost:1234")
        //.allowed_methods(vec!["GET", "POST", "OPTIONS"])
        //.allowed_header(header::CONTENT_TYPE);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .service(get_todos)
            .service(get_todo)
            .service(post_todo)
            .service(check_todo)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
