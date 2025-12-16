use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::{Key, SameSite}, delete, get, post, put, web, App, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use stoolap::api::{Database, ResultRow};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub email: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub email: String,
    pub name: String,
    pub is_admin: bool,
}

#[derive(Serialize, Deserialize)]
pub struct BlogPost {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub created_at: String, // Stored as String in Stoolap
    pub author_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub post_id: i64,
    pub author_name: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
}

#[derive(Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct AuthRequest {
    pub code: String,
    pub state: String,
}

#[get("/api/hello")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "In the Dusty Clockless Hours"
    }))
}

#[derive(Serialize, Deserialize)]
pub struct AdminUser {
    pub email: String,
    pub created_at: String, // Stored as String in Stoolap
}

#[derive(Deserialize)]
pub struct AddAdminRequest {
    pub email: String,
}

// Helper to check if a user is an admin
pub fn is_admin(db: &Database, email: &str) -> bool {
    // Case-insensitive check
    // Stoolap supports $1 binding
    let count: i64 = db.query_one(
        "SELECT COUNT(*) FROM admins WHERE lower(email) = lower($1)", 
        (email.trim(),)
    ).unwrap_or(0);

    count > 0
}

fn map_post(row: ResultRow) -> Result<BlogPost, stoolap::Error> {
    Ok(BlogPost {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        created_at: row.get(3)?,
        author_name: row.get(4)?,
    })
}

fn map_admin(row: ResultRow) -> Result<AdminUser, stoolap::Error> {
    Ok(AdminUser {
        email: row.get(0)?,
        created_at: row.get(1)?,
    })
}

fn map_comment(row: ResultRow) -> Result<Comment, stoolap::Error> {
    Ok(Comment {
        id: row.get(0)?,
        post_id: row.get(1)?,
        author_name: row.get(2)?,
        content: row.get(3)?,
        created_at: row.get(4)?,
    })
}

#[get("/api/posts")]
pub async fn get_posts(db: web::Data<Database>) -> impl Responder {
    let db = db.get_ref().clone(); // Cheap clone of Arc
    let result = web::block(move || -> Result<Vec<BlogPost>, String> {
        let mut posts = Vec::new();
        let rows = db.query("SELECT id, title, content, created_at, author_name FROM posts ORDER BY id DESC", ())
            .map_err(|e| e.to_string())?;
        
        for row in rows {
            let row = row.map_err(|e| e.to_string())?;
            posts.push(map_post(row).map_err(|e| e.to_string())?);
        }
        Ok(posts)
    }).await;

    match result {
        Ok(Ok(posts)) => HttpResponse::Ok()
            .append_header(("Cache-Control", "no-store"))
            .json(posts),
        Ok(Err(e)) => {
            println!("Error fetching posts: {}", e);
            HttpResponse::InternalServerError().body("Error fetching posts")
        }
        Err(e) => {
            println!("Blocking error: {}", e);
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}

#[get("/api/posts/{id}")]
pub async fn get_post(
    db: web::Data<Database>,
    id: web::Path<i64>,
) -> impl Responder {
    let db = db.get_ref().clone();
    let post_id = id.into_inner();
    
    let result = web::block(move || -> Result<Option<BlogPost>, String> {
        let mut rows = db.query("SELECT id, title, content, created_at, author_name FROM posts WHERE id = $1", (post_id,))
            .map_err(|e| e.to_string())?;
            
        if let Some(row) = rows.next() {
            let row = row.map_err(|e| e.to_string())?;
            Ok(Some(map_post(row).map_err(|e| e.to_string())?))
        } else {
            Ok(None)
        }
    }).await;

    match result {
        Ok(Ok(Some(post))) => HttpResponse::Ok()
            .append_header(("Cache-Control", "no-store"))
            .json(post),
        Ok(Ok(None)) => HttpResponse::NotFound().body("Post not found"),
        Ok(Err(e)) => {
            println!("Error fetching post: {}", e);
            HttpResponse::InternalServerError().body("Error fetching post")
        }
        Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

#[post("/admin/posts")]
pub async fn create_post(
    db: web::Data<Database>,
    session: Session,
    post: web::Json<CreatePostRequest>,
) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        let db = db.get_ref().clone();
        let email = user.email.clone();
        let name = user.name.clone();
        let post_data = post.into_inner();

        let result = web::block(move || -> Result<(), String> {
            if !is_admin(&db, &email) {
                return Err("Access Denied".to_string());
            }
            db.execute(
                "INSERT INTO posts (title, content, created_at, author_name) VALUES ($1, $2, $3, $4)",
                (post_data.title, post_data.content, Utc::now().to_rfc3339(), name),
            ).map_err(|e| e.to_string())?;
            Ok(())
        }).await;

        match result {
            Ok(Ok(_)) => HttpResponse::Ok().json(serde_json::json!({"status": "success"})),
            Ok(Err(e)) => {
                if e.contains("Access Denied") {
                    HttpResponse::Forbidden().body("Access Denied")
                } else {
                    println!("Error creating post: {}", e);
                    HttpResponse::InternalServerError().body("Error creating post")
                }
            }
            Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
        }
    } else {
        HttpResponse::Forbidden().body("Access Denied: Not logged in")
    }
}

#[get("/admin/users")]
pub async fn get_admins(db: web::Data<Database>, session: Session) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        let db = db.get_ref().clone();
        let email = user.email.clone();

        let result = web::block(move || -> Result<Vec<AdminUser>, String> {
             if !is_admin(&db, &email) {
                return Err("Access Denied".to_string());
            }
            let mut admins = Vec::new();
            let rows = db.query("SELECT email, created_at FROM admins ORDER BY created_at DESC", ())
                .map_err(|e| e.to_string())?;

            for row in rows {
                let row = row.map_err(|e| e.to_string())?;
                admins.push(map_admin(row).map_err(|e| e.to_string())?);
            }
            Ok(admins)
        }).await;

        match result {
            Ok(Ok(admins)) => HttpResponse::Ok()
                .append_header(("Cache-Control", "no-store"))
                .json(admins),
            Ok(Err(e)) => {
                 if e.contains("Access Denied") {
                    HttpResponse::Forbidden().body("Access Denied")
                } else {
                    println!("Error fetching admins: {}", e);
                    HttpResponse::InternalServerError().body("Error fetching admins")
                }
            }
            Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
        }
    } else {
        HttpResponse::Forbidden().body("Access Denied: Not logged in")
    }
}

#[post("/admin/users")]
pub async fn add_admin(
    db: web::Data<Database>,
    session: Session,
    req: web::Json<AddAdminRequest>,
) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        let db = db.get_ref().clone();
        let email = user.email.clone();
        let new_admin = req.email.clone();

         let result = web::block(move || -> Result<(), String> {
            if !is_admin(&db, &email) {
                return Err("Access Denied".to_string());
            }
            db.execute(
                "INSERT INTO admins (email, created_at) VALUES ($1, $2)",
                (new_admin.trim(), Utc::now().to_rfc3339()) 
            ).map_err(|e| e.to_string())?;
            Ok(())
        }).await;

       match result {
            Ok(Ok(_)) => HttpResponse::Ok().json(serde_json::json!({"status": "success"})),
            Ok(Err(e)) => {
                 if e.contains("Access Denied") {
                    HttpResponse::Forbidden().body("Access Denied")
                } else {
                    println!("Error adding admin: {}", e);
                     HttpResponse::InternalServerError().body("Error adding admin (maybe already exists?)")
                }
            }
            Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
        }
    } else {
        HttpResponse::Forbidden().body("Access Denied: Not logged in")
    }
}

#[delete("/admin/users/{email}")]
pub async fn delete_admin(
    db: web::Data<Database>,
    session: Session,
    email: web::Path<String>,
) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        let db = db.get_ref().clone();
        let user_email = user.email.clone();
        let email_to_delete = email.into_inner();

        let result = web::block(move || -> Result<(), String> {
            if !is_admin(&db, &user_email) {
                 return Err("Access Denied".to_string());
            }

            // Check if this is the last admin
             let count: i64 = db.query_one("SELECT COUNT(*) FROM admins", ()).unwrap_or(0);
            if count <= 1 {
                 return Err("Cannot delete the last admin".to_string());
            }

            db.execute("DELETE FROM admins WHERE email = $1", (email_to_delete,))
                .map_err(|e| e.to_string())?;
            Ok(())
        }).await;

        match result {
            Ok(Ok(_)) => HttpResponse::Ok().json(serde_json::json!({"status": "success"})),
            Ok(Err(e)) => {
                let msg = e;
                if msg.contains("Access Denied") {
                     HttpResponse::Forbidden().body("Access Denied")
                } else if msg.contains("Cannot delete") {
                    HttpResponse::BadRequest().body(msg)
                } else {
                    println!("Error deleting admin: {}", msg);
                    HttpResponse::InternalServerError().body("Error deleting admin")
                }
            }
             Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
        }
    } else {
        HttpResponse::Forbidden().body("Access Denied: Not logged in")
    }
}

#[delete("/admin/posts/{id}")]
pub async fn delete_post(
    db: web::Data<Database>,
    session: Session,
    id: web::Path<i64>,
) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        let db = db.get_ref().clone();
        let email = user.email.clone();
        let post_id = id.into_inner();

        let result = web::block(move || -> Result<(), String> {
            if !is_admin(&db, &email) {
                return Err("Access Denied".to_string());
            }
            db.execute("DELETE FROM posts WHERE id = $1", (post_id,))
                .map_err(|e| e.to_string())?;
            Ok(())
        }).await;

        match result {
            Ok(Ok(_)) => HttpResponse::Ok().json(serde_json::json!({"status": "success"})),
            Ok(Err(e)) => {
                 if e.contains("Access Denied") {
                     HttpResponse::Forbidden().body("Access Denied")
                } else {
                    println!("Error deleting post: {}", e);
                    HttpResponse::InternalServerError().body("Error deleting post")
                }
            }
             Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
        }
    } else {
        HttpResponse::Forbidden().body("Access Denied: Not logged in")
    }
}

#[put("/admin/posts/{id}")]
pub async fn update_post(
    db: web::Data<Database>,
    session: Session,
    id: web::Path<i64>,
    post: web::Json<CreatePostRequest>,
) -> impl Responder {
     if let Ok(Some(user)) = session.get::<User>("user") {
        let db = db.get_ref().clone();
        let email = user.email.clone();
        let post_id = id.into_inner();
        let post_data = post.into_inner();

        let result = web::block(move || -> Result<(), String> {
            if !is_admin(&db, &email) {
                 return Err("Access Denied".to_string());
            }
            db.execute(
                "UPDATE posts SET title = $1, content = $2 WHERE id = $3",
                 (post_data.title, post_data.content, post_id)
            ).map_err(|e| e.to_string())?;
            Ok(())
        }).await;

        match result {
            Ok(Ok(_)) => HttpResponse::Ok().json(serde_json::json!({"status": "success"})),
            Ok(Err(e)) => {
                if e.contains("Access Denied") {
                     HttpResponse::Forbidden().body("Access Denied")
                } else {
                    println!("Error updating post: {}", e);
                    HttpResponse::InternalServerError().body("Error updating post")
                }
            }
            Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
        }
    } else {
        HttpResponse::Forbidden().body("Access Denied: Not logged in")
    }
}

#[get("/auth/login")]
pub async fn login(client: web::Data<BasicClient>, session: Session) -> impl Responder {
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_extra_param("prompt", "select_account")
        .url();

    session.insert("csrf_token", csrf_token.secret()).ok();
    HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish()
}

#[get("/auth/callback")]
pub async fn callback(
    query: web::Query<AuthRequest>,
    client: web::Data<BasicClient>,
    session: Session,
) -> impl Responder {
    let code = AuthorizationCode::new(query.code.clone());
    let state = query.state.clone();

    if let Ok(Some(csrf_token)) = session.get::<String>("csrf_token") {
        if state != csrf_token {
            return HttpResponse::BadRequest().body("Invalid CSRF token");
        }
    } else {
        return HttpResponse::BadRequest().body("Missing CSRF token");
    }

    let token = client
        .exchange_code(code)
        .request_async(async_http_client)
        .await;

    match token {
        Ok(token) => {
            let client = Client::new();
            let user_info = client
                .get("https://www.googleapis.com/oauth2/v2/userinfo")
                .bearer_auth(token.access_token().secret())
                .send()
                .await;

            match user_info {
                Ok(response) => {
                    if let Ok(user) = response.json::<User>().await {
                        session.insert("user", &user).ok();
                        HttpResponse::Found()
                            .append_header(("Location", "/"))
                            .finish()
                    } else {
                        HttpResponse::InternalServerError().body("Failed to parse user info")
                    }
                }
                Err(_) => HttpResponse::InternalServerError().body("Failed to fetch user info"),
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to exchange token"),
    }
}

#[get("/auth/me")]
pub async fn me(db: web::Data<Database>, session: Session) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        let db = db.get_ref().clone();
        let email = user.email.clone();
        
        // Check if admin (blocking op)
        let is_admin = web::block(move || is_admin(&db, &email)).await.map(|r| r).unwrap_or(false);

        HttpResponse::Ok()
            .append_header(("Cache-Control", "no-store"))
            .json(UserResponse {
            email: user.email,
            name: user.name,
            is_admin,
        })
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[get("/auth/logout")]
pub async fn logout(session: Session) -> impl Responder {
    session.purge();
    HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish()
}

#[get("/admin/dashboard")]
pub async fn admin_dashboard(db: web::Data<Database>, session: Session) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        let db = db.get_ref().clone();
        let email = user.email.clone();
        
        let result = web::block(move || -> Result<bool, String> {
            Ok(is_admin(&db, &email))
        }).await;

        if let Ok(Ok(true)) = result {
             HttpResponse::Ok()
                .append_header(("Cache-Control", "no-store"))
                .json(serde_json::json!({
                "secret": format!("Welcome {}", user.email)
            }))
        } else {
             HttpResponse::Forbidden()
                .body(format!("Access Denied: Unauthorized email ({})", user.email))
        }
    } else {
        HttpResponse::Forbidden().body("Access Denied: Not logged in")
    }
}

#[get("/api/posts/{id}/comments")]
pub async fn get_comments(
    db: web::Data<Database>,
    id: web::Path<i64>,
) -> impl Responder {
    let db = db.get_ref().clone();
    let post_id = id.into_inner();
    println!("Backend: Fetching comments for post_id: {}", post_id);

    let result = web::block(move || -> Result<Vec<Comment>, String> {
        let mut comments = Vec::new();
        let rows = db.query(
            "SELECT id, post_id, author_name, content, created_at FROM comments WHERE post_id = $1 ORDER BY created_at ASC",
            (post_id,)
        ).map_err(|e| e.to_string())?;

        for row in rows {
            let row = row.map_err(|e| e.to_string())?;
            comments.push(map_comment(row).map_err(|e| e.to_string())?);
        }
        println!("Backend: Found {} comments for post {}", comments.len(), post_id);
        Ok(comments)
    }).await;

    match result {
        Ok(Ok(comments)) => HttpResponse::Ok()
            .append_header(("Cache-Control", "no-store"))
            .json(comments),
        Ok(Err(e)) => {
            println!("Error fetching comments: {}", e);
            HttpResponse::InternalServerError().body("Error fetching comments")
        }
        Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

#[post("/api/posts/{id}/comments")]
pub async fn create_comment(
    db: web::Data<Database>,
    session: Session,
    id: web::Path<i64>,
    req: web::Json<CreateCommentRequest>,
) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        let db = db.get_ref().clone();
        let post_id = id.into_inner();
        let comment_content = req.content.clone();
        let author_name = user.name.clone();

        println!("Backend: Received create_comment request for post_id: {}", post_id);


        let result = web::block(move || -> Result<Comment, String> {
            let created_at = Utc::now().to_rfc3339();
            let id: i64 = db.query_one(
                "INSERT INTO comments (post_id, author_name, content, created_at) VALUES ($1, $2, $3, $4) RETURNING id",
                (post_id, author_name.clone(), comment_content.clone(), created_at.clone())
            ).map_err(|e| e.to_string())?;
            
            println!("Backend: Successfully inserted comment. Returning ID: {}", id);

            Ok(Comment {
                id,
                post_id,
                author_name,
                content: comment_content,
                created_at,
            })
        }).await;

        match result {
            Ok(Ok(comment)) => HttpResponse::Ok().json(comment),
            Ok(Err(e)) => {
                println!("Error creating comment: {}", e);
                HttpResponse::InternalServerError().body("Error creating comment")
            }
            Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
        }
    } else {
        HttpResponse::Forbidden().body("Access Denied: Not logged in")
    }
}

#[delete("/admin/comments/{id}")]
pub async fn delete_comment(
    db: web::Data<Database>,
    session: Session,
    id: web::Path<i64>,
) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        let db = db.get_ref().clone();
        let email = user.email.clone();
        let comment_id = id.into_inner();

        let result = web::block(move || -> Result<(), String> {
            if !is_admin(&db, &email) {
                return Err("Access Denied".to_string());
            }
            db.execute("DELETE FROM comments WHERE id = $1", (comment_id,))
                .map_err(|e| e.to_string())?;
            Ok(())
        }).await;

        match result {
            Ok(Ok(_)) => HttpResponse::Ok().json(serde_json::json!({"status": "success"})),
            Ok(Err(e)) => {
                 if e.contains("Access Denied") {
                     HttpResponse::Forbidden().body("Access Denied")
                } else {
                    println!("Error deleting comment: {}", e);
                    HttpResponse::InternalServerError().body("Error deleting comment")
                }
            }
             Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
        }
    } else {
        HttpResponse::Forbidden().body("Access Denied: Not logged in")
    }
}

pub async fn run_app() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let google_client_id = ClientId::new(
        env::var("GOOGLE_CLIENT_ID").expect("Missing GOOGLE_CLIENT_ID environment variable"),
    );
    let google_client_secret = ClientSecret::new(
        env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing GOOGLE_CLIENT_SECRET environment variable"),
    );
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization URL");
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
        .expect("Invalid token URL");

    let redirect_url = env::var("REDIRECT_URL")
        .unwrap_or_else(|_| "http://localhost:8080/auth/callback".to_string());

    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(
        RedirectUrl::new(redirect_url).expect("Invalid redirect URL"),
    );

    // Database setup - Stoolap
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "file://blog.db".to_string());
    println!("Connecting to Stoolap database at: {}", database_url);

    let db = Database::open(&database_url).expect("Failed to open database");

    // Create posts table
    db.execute(
        "CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTO_INCREMENT,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            author_name TEXT NOT NULL DEFAULT 'Anonymous'
        )",
        ()
    ).expect("Failed to create posts table");

    // Create admins table
    db.execute(
        "CREATE TABLE IF NOT EXISTS admins (
            id INTEGER PRIMARY KEY AUTO_INCREMENT,
            email TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        ()
    ).expect("Failed to create admins table");

    // Create comments table
    db.execute(
        "CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTO_INCREMENT,
            post_id INTEGER NOT NULL,
            author_name TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        ()
    ).expect("Failed to create comments table");

    // Bootstrap initial admin
    let initial_admin = "harrison.dale@googlemail.com";
    let count: i64 = db.query_one("SELECT COUNT(*) FROM admins", ()).unwrap_or(0);

    if count == 0 {
        println!("Bootstrapping initial admin: {}", initial_admin);
        db.execute(
            "INSERT INTO admins (email, created_at) VALUES ($1, $2)",
            (initial_admin, Utc::now().to_rfc3339())
        ).expect("Failed to insert initial admin");
    }

    // Load session key from environment or generate a random one
    let secret_key = if let Ok(key) = env::var("SESSION_KEY") {
        Key::from(key.as_bytes())
    } else {
        Key::generate()
    };

    println!("Database connection established.");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials(); // Important for cookies

        App::new()
            .wrap(cors)
            .wrap(SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key.clone(),
            )
            .cookie_secure(true) // Cloud Run uses HTTPS
            .cookie_same_site(SameSite::Lax)
            .build())
            .wrap(actix_web::middleware::DefaultHeaders::new()
                .add((
                    "Content-Security-Policy",
                    "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.tailwindcss.com; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data:; connect-src 'self';",
                ))
                .add((
                    "Strict-Transport-Security",
                    "max-age=31536000; includeSubDomains",
                ))
            )
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(db.clone()))
            .service(hello)
            .service(login)
            .service(callback)
            .service(me)
            .service(logout)
            .service(admin_dashboard)
            .service(get_posts)
            .service(get_post)
            .service(create_post)
            .service(get_admins)
            .service(add_admin)
            .service(delete_admin)
            .service(delete_post)
            .service(update_post)
            .service(get_comments)
            .service(create_comment)
            .service(delete_comment)
            .service(
                Files::new("/", "./static")
                    .index_file("index.html")
                    .default_handler(web::get().to(|| async {
                        NamedFile::open("./static/index.html")
                    }))
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
