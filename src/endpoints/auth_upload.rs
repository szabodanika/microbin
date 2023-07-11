use crate::args::{Args, ARGS};
use crate::endpoints::errors::ErrorTemplate;
use crate::util::animalnumbers::to_u64;
use crate::util::hashids::to_u64 as hashid_to_u64;
use crate::util::misc::remove_expired;
use crate::AppState;
use actix_web::{get, web, HttpResponse};
use askama::Template;

#[derive(Template)]
#[template(path = "auth_upload.html")]
struct AuthPasta<'a> {
    args: &'a Args,
    id: String,
    status: String,
    encrypted_key: String,
    encrypt_client: bool,
    path: String,
}

#[get("/auth/{id}")]
pub async fn auth_upload(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_i, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html").body(
                AuthPasta {
                    args: &ARGS,
                    id: id.into_inner(),
                    status: String::from(""),
                    encrypted_key: pasta.encrypted_key.to_owned().unwrap_or_default(),
                    encrypt_client: pasta.encrypt_client,
                    path: String::from("upload"),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/auth/{id}/{status}")]
pub async fn auth_upload_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let (id, status) = param.into_inner();

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_i, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html").body(
                AuthPasta {
                    args: &ARGS,
                    id,
                    status,
                    encrypted_key: pasta.encrypted_key.to_owned().unwrap_or_default(),
                    encrypt_client: pasta.encrypt_client,
                    path: String::from("upload"),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/auth_raw/{id}")]
pub async fn auth_raw_pasta(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_i, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html").body(
                AuthPasta {
                    args: &ARGS,
                    id: id.into_inner(),
                    status: String::from(""),
                    encrypted_key: pasta.encrypted_key.to_owned().unwrap_or_default(),
                    encrypt_client: pasta.encrypt_client,
                    path: String::from("raw"),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/auth_raw/{id}/{status}")]
pub async fn auth_raw_pasta_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let (id, status) = param.into_inner();

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_i, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html").body(
                AuthPasta {
                    args: &ARGS,
                    id,
                    status,
                    encrypted_key: pasta.encrypted_key.to_owned().unwrap_or_default(),
                    encrypt_client: pasta.encrypt_client,
                    path: String::from("raw"),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/auth_edit_private/{id}")]
pub async fn auth_edit_private(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html").body(
                AuthPasta {
                    args: &ARGS,
                    id: id.into_inner(),
                    status: String::from(""),
                    encrypted_key: pasta.encrypted_key.to_owned().unwrap_or_default(),
                    encrypt_client: pasta.encrypt_client,
                    path: String::from("edit_private"),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/auth_edit_private/{id}/{status}")]
pub async fn auth_edit_private_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let (id, status) = param.into_inner();

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_i, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html").body(
                AuthPasta {
                    args: &ARGS,
                    id,
                    status,
                    encrypted_key: pasta.encrypted_key.to_owned().unwrap_or_default(),
                    encrypt_client: pasta.encrypt_client,
                    path: String::from("edit_private"),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/auth_file/{id}")]
pub async fn auth_file(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html").body(
                AuthPasta {
                    args: &ARGS,
                    id: id.into_inner(),
                    status: String::from(""),
                    encrypted_key: pasta.encrypted_key.to_owned().unwrap_or_default(),
                    encrypt_client: pasta.encrypt_client,
                    path: String::from("secure_file"),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/auth_file/{id}/{status}")]
pub async fn auth_file_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let (id, status) = param.into_inner();

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_i, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html").body(
                AuthPasta {
                    args: &ARGS,
                    id,
                    status,
                    encrypted_key: pasta.encrypted_key.to_owned().unwrap_or_default(),
                    encrypt_client: pasta.encrypt_client,
                    path: String::from("secure_file"),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/auth_remove_private/{id}")]
pub async fn auth_remove_private(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html").body(
                AuthPasta {
                    args: &ARGS,
                    id: id.into_inner(),
                    status: String::from(""),
                    encrypted_key: pasta.encrypted_key.to_owned().unwrap_or_default(),
                    encrypt_client: pasta.encrypt_client,
                    path: String::from("remove"),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}

#[get("/auth_remove_private/{id}/{status}")]
pub async fn auth_remove_private_with_status(
    data: web::Data<AppState>,
    param: web::Path<(String, String)>,
) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    remove_expired(&mut pastas);

    let (id, status) = param.into_inner();

    let intern_id = if ARGS.hash_ids {
        hashid_to_u64(&id).unwrap_or(0)
    } else {
        to_u64(&id).unwrap_or(0)
    };

    for (_i, pasta) in pastas.iter().enumerate() {
        if pasta.id == intern_id {
            return HttpResponse::Ok().content_type("text/html").body(
                AuthPasta {
                    args: &ARGS,
                    id,
                    status,
                    encrypted_key: pasta.encrypted_key.to_owned().unwrap_or_default(),
                    encrypt_client: pasta.encrypt_client,
                    path: String::from("remove"),
                }
                .render()
                .unwrap(),
            );
        }
    }

    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}
