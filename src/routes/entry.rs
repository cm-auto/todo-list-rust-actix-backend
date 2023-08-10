use actix_web::{
    http::StatusCode,
    web::{self, ServiceConfig},
    HttpResponseBuilder, Responder,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::prototype_db::Database;

async fn get_entries(db: web::Data<Database>) -> impl Responder {
    let entry_collection_mutex = db.get_entry_collection();
    let entry_collection = entry_collection_mutex.lock().unwrap();

    let entries = entry_collection.get_all();
    HttpResponseBuilder::new(StatusCode::OK).json(entries)
}

async fn get_entry(id: web::Path<String>, db: web::Data<Database>) -> impl Responder {
    let entry_collection_mutex = db.get_entry_collection();
    let entry_collection = entry_collection_mutex.lock().unwrap();
    let id = id.into_inner();
    let entry_option = entry_collection.find_one(|model| model.id == id);
    if entry_option.is_none() {
        return HttpResponseBuilder::new(StatusCode::NOT_FOUND).json("not found");
    }
    let entry = entry_option.unwrap();
    HttpResponseBuilder::new(StatusCode::OK).json(entry)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostEntryRequestData {
    list_id: String,
    name: String,
    done: Option<bool>,
}
async fn post_entry(
    body: web::Json<PostEntryRequestData>,
    db: web::Data<Database>,
) -> impl Responder {
    let list_collection_mutex = db.get_list_collection();
    let list_collection = list_collection_mutex.lock().unwrap();
    let request_data = body.into_inner();
    let list_id = request_data.list_id.clone();
    let list_option = list_collection.find_one(|model| model.id == list_id);
    if list_option.is_none() {
        return HttpResponseBuilder::new(StatusCode::NOT_FOUND).json("list not found");
    }
    let uuidv4 = Uuid::new_v4().to_string();
    let new_model = crate::models::entry::Entry {
        id: uuidv4,
        list_id: request_data.list_id,
        name: request_data.name,
        done: request_data.done.unwrap_or(false),
    };

    let entry_collection_mutex = db.get_entry_collection();
    let mut entry_collection = entry_collection_mutex.lock().unwrap();
    let save_result = entry_collection.append(new_model.clone());
    if let Err(e) = save_result {
        return HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).json(e.to_string());
    }
    HttpResponseBuilder::new(StatusCode::CREATED).json(&new_model)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PatchEntryRequestData {
    list_id: Option<String>,
    name: Option<String>,
    done: Option<bool>,
}
async fn patch_entry(
    body: web::Json<PatchEntryRequestData>,
    db: web::Data<Database>,
    id: web::Path<String>,
) -> impl Responder {
    let entry_collection_mutex = db.get_entry_collection();
    let mut entry_collection = entry_collection_mutex.lock().unwrap();
    let id = id.into_inner();
    let body = body.into_inner();

    // if a list_id is provided this checks if the list exists
    // and if it doesn't it returns an error message
    let list_id_checked_option = if let Some(list_id) = &body.list_id {
        let list_exists = db
            .get_list_collection()
            .lock()
            .unwrap()
            .find_one(|model| model.id == list_id.clone())
            .is_some();
        if !list_exists {
            return HttpResponseBuilder::new(StatusCode::NOT_FOUND).json("list not found");
        }
        Some(list_id.clone())
    } else {
        None
    };

    let save_result = entry_collection.patch_one(
        move |model| model.id == id,
        move |model| {
            if let Some(list_id) = list_id_checked_option {
                model.list_id = list_id;
            }
            if let Some(name) = &body.name {
                model.name = name.clone();
            }
            if let Some(done) = &body.done {
                model.done = *done;
            }
        },
    );
    if let Err(e) = save_result {
        return HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).json(e.to_string());
    }
    let model_option = save_result.unwrap();
    if model_option.is_none() {
        return HttpResponseBuilder::new(StatusCode::NOT_FOUND).json("not found");
    }
    let model = model_option.unwrap();
    HttpResponseBuilder::new(StatusCode::OK).json(model)
}

async fn delete_entry(db: web::Data<Database>, id: web::Path<String>) -> impl Responder {
    let entry_collection_mutex = db.get_entry_collection();
    let mut entry_collection = entry_collection_mutex.lock().unwrap();
    let id = id.into_inner();
    let save_result = entry_collection.delete_one(|model| model.id == id);
    if let Err(e) = save_result {
        return HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).json(e.to_string());
    }
    HttpResponseBuilder::new(StatusCode::NO_CONTENT).finish()
}

#[derive(Deserialize)]
struct PutEntryRequestData {
    list_id: String,
    name: String,
    done: bool,
}
async fn put_entry(
    db: web::Data<Database>,
    body: web::Json<PutEntryRequestData>,
    id: web::Path<String>,
) -> impl Responder {
    let entry_collection_mutex = db.get_entry_collection();
    let mut entry_collection = entry_collection_mutex.lock().unwrap();
    let request_data = body.into_inner();
    let list_id = request_data.list_id.clone();
    let list_option = entry_collection.find_one(|model| model.id == list_id);
    if list_option.is_none() {
        return HttpResponseBuilder::new(StatusCode::NOT_FOUND).json("list not found");
    }
    let id = id.into_inner();
    let new_model = crate::models::entry::Entry {
        id,
        list_id: request_data.list_id,
        name: request_data.name,
        done: request_data.done,
    };
    let save_result = entry_collection.append(new_model.clone());
    if let Err(e) = save_result {
        return HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).json(e.to_string());
    }
    HttpResponseBuilder::new(StatusCode::CREATED).json(&new_model)
}

pub fn configure_routes(config: &mut ServiceConfig) {
    config.route("", web::get().to(get_entries));
    config.route("/{id}", web::get().to(get_entry));
    config.route("", web::post().to(post_entry));
    config.route("/{id}", web::patch().to(patch_entry));
    config.route("/{id}", web::delete().to(delete_entry));
    config.route("/{id}", web::put().to(put_entry));
}
