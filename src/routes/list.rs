use actix_web::{
    http::StatusCode,
    web::{self, ServiceConfig},
    HttpResponseBuilder, Responder,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    models::{list::List, parent_and_children::ParentAndChildren},
    prototype_db::Database,
};

async fn get_lists(db: web::Data<Database>) -> impl Responder {
    let list_collection_mutex = db.get_list_collection();
    let list_collection = list_collection_mutex.lock().unwrap();

    let lists = list_collection.get_all();
    HttpResponseBuilder::new(StatusCode::OK).json(lists)
}

async fn get_list(id: web::Path<String>, db: web::Data<Database>) -> impl Responder {
    let list_collection_mutex = db.get_list_collection();
    let list_collection = list_collection_mutex.lock().unwrap();

    // let lists = list_collection.get_all();
    let id = id.into_inner();
    let list_option = list_collection.find_one(|model| model.id == id);
    if list_option.is_none() {
        return HttpResponseBuilder::new(StatusCode::NOT_FOUND).json("not found");
    }
    let list = list_option.unwrap();
    HttpResponseBuilder::new(StatusCode::OK).json(list)
}

async fn get_list_and_its_entries(
    id: web::Path<String>,
    db: web::Data<Database>,
) -> impl Responder {
    let list_collection_mutex = db.get_list_collection();
    let list_collection = list_collection_mutex.lock().unwrap();

    let id = id.into_inner();
    let list_option = list_collection.find_one(|model| model.id == id);
    if list_option.is_none() {
        return HttpResponseBuilder::new(StatusCode::NOT_FOUND).json("not found");
    }
    let list = list_option.unwrap();

    let entry_collection_mutex = db.get_entry_collection();
    let entry_collection = entry_collection_mutex.lock().unwrap();
    let entries = entry_collection.find(|model| model.list_id == id);

    let body = ParentAndChildren {
        parent: list,
        children: &entries,
    };

    HttpResponseBuilder::new(StatusCode::OK).json(body)
}

#[derive(Deserialize)]
struct PostListRequestData {
    name: String,
}
async fn post_list(
    body: web::Json<PostListRequestData>,
    db: web::Data<Database>,
) -> impl Responder {
    let uuidv4 = Uuid::new_v4().to_string();
    let new_model = List {
        id: uuidv4,
        name: body.into_inner().name,
    };
    let list_collection_mutex = db.get_list_collection();
    let mut list_collection = list_collection_mutex.lock().unwrap();
    let save_result = list_collection.append(new_model.clone());
    if let Err(e) = save_result {
        return HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).json(e.to_string());
    }
    HttpResponseBuilder::new(StatusCode::CREATED).json(&new_model)
}

#[derive(Deserialize)]
struct PatchListRequestData {
    name: Option<String>,
}
async fn patch_list(
    body: web::Json<PatchListRequestData>,
    db: web::Data<Database>,
    id: web::Path<String>,
) -> impl Responder {
    let list_collection_mutex = db.get_list_collection();
    let mut list_collection = list_collection_mutex.lock().unwrap();
    let id = id.into_inner();
    let body = body.into_inner();
    let save_result = list_collection.patch_one(
        move |model| model.id == id,
        move |model| {
            if let Some(name) = &body.name {
                model.name = name.clone();
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
    let body = model_option.unwrap();
    HttpResponseBuilder::new(StatusCode::OK).json(&body)
}

#[derive(Deserialize)]
struct PutListRequestData {
    name: String,
}
async fn put_list(
    body: web::Json<PutListRequestData>,
    db: web::Data<Database>,
    id: web::Path<String>,
) -> impl Responder {
    let list_collection_mutex = db.get_list_collection();
    let mut list_collection = list_collection_mutex.lock().unwrap();
    let id = id.into_inner();
    let body = body.into_inner();
    let save_result = list_collection.put_one(
        |model| model.id == id.clone(),
        List {
            id: id.clone(),
            name: body.name.clone(),
        },
    );
    if let Err(e) = save_result {
        return HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).json(e.to_string());
    }
    let body = save_result.unwrap();
    HttpResponseBuilder::new(StatusCode::OK).json(&body)
}

async fn delete_list(id: web::Path<String>, db: web::Data<Database>) -> impl Responder {
    let id = id.into_inner();

    let entry_collection_mutex = db.get_entry_collection();
    let mut entry_collection = entry_collection_mutex.lock().unwrap();
    let delete_result = entry_collection.delete_many(|model| {
        // this does not compare the pointers like in C, but the actual values
        // the pointers are pointing to
        &model.list_id == &id
    });
    if let Err(e) = delete_result {
        return HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).json(e.to_string());
    }

    let list_collection_mutex = db.get_list_collection();
    let mut list_collection = list_collection_mutex.lock().unwrap();
    let delete_result = list_collection.delete_one(|model| &model.id == &id);
    if let Err(e) = delete_result {
        return HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).json(e.to_string());
    }
    let model_option = delete_result.unwrap();
    if model_option.is_none() {
        return HttpResponseBuilder::new(StatusCode::NOT_FOUND).json("not found");
    }
    HttpResponseBuilder::new(StatusCode::NO_CONTENT).finish()
}

pub fn configure_routes(config: &mut ServiceConfig) {
    config.route("", web::get().to(get_lists));
    config.route("/{id}", web::get().to(get_list));
    config.route("/{id}/entries", web::get().to(get_list_and_its_entries));
    config.route("", web::post().to(post_list));
    config.route("/{id}", web::patch().to(patch_list));
    config.route("/{id}", web::put().to(put_list));
    config.route("/{id}", web::delete().to(delete_list));
}
