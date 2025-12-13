use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::{Key, SameSite}, delete, get, post, put, web, App, HttpResponse, HttpServer, Responder};



#[put("/admin/posts/{id}")]
pub async fn update_post(
    pool: web::Data<Pool<Sqlite>>,
    session: Session,
    id: web::Path<i64>,
    post: web::Json<CreatePostRequest>,
) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        if is_admin(pool.get_ref(), &user.email).await {
            let post_id = id.into_inner();
            let result = sqlx::query(
                "UPDATE posts SET title = ?, content = ? WHERE id = ?",
            )
            .bind(&post.title)
            .bind(&post.content)
            .bind(post_id)
            .execute(pool.get_ref())
            .await;

            match result {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "success"})),
                Err(e) => {
                    println!("Error updating post: {}", e);
                    HttpResponse::InternalServerError().body("Error updating post")
                }
            }
        } else {
            HttpResponse::Forbidden().body("Access Denied")
        }
    } else {
        HttpResponse::Forbidden().body("Access Denied: Not logged in")
    }
}



use chrono::{DateTime, Utc};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::env;
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub email: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct BlogPost {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub author_name: String,
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

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct AdminUser {
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct AddAdminRequest {
    pub email: String,
}

// Helper to check if a user is an admin
pub async fn is_admin(pool: &Pool<Sqlite>, email: &str) -> bool {
    // Case-insensitive check
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM admins WHERE lower(email) = lower(?)",
    )
    .bind(email.trim())
    .fetch_one(pool)
    .await
    .unwrap_or((0,));

    count.0 > 0
}

#[get("/api/posts")]
pub async fn get_posts(pool: web::Data<Pool<Sqlite>>) -> impl Responder {
    let result = sqlx::query_as::<_, BlogPost>("SELECT id, title, content, created_at, author_name FROM posts ORDER BY created_at DESC")
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(posts) => HttpResponse::Ok().json(posts),
        Err(e) => {
            println!("Error fetching posts: {}", e);
            HttpResponse::InternalServerError().body("Error fetching posts")
        }
    }
}

#[get("/api/posts/{id}")]
pub async fn get_post(
    pool: web::Data<Pool<Sqlite>>,
    id: web::Path<i64>,
) -> impl Responder {
    let post_id = id.into_inner();
    let result = sqlx::query_as::<_, BlogPost>("SELECT id, title, content, created_at, author_name FROM posts WHERE id = ?")
        .bind(post_id)
        .fetch_optional(pool.get_ref())
        .await;

    match result {
        Ok(Some(post)) => HttpResponse::Ok().json(post),
        Ok(None) => HttpResponse::NotFound().body("Post not found"),
        Err(e) => {
            println!("Error fetching post: {}", e);
            HttpResponse::InternalServerError().body("Error fetching post")
        }
    }
}

#[post("/admin/posts")]
pub async fn create_post(
    pool: web::Data<Pool<Sqlite>>,
    session: Session,
    post: web::Json<CreatePostRequest>,
) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        if is_admin(pool.get_ref(), &user.email).await {
            let result = sqlx::query(
                "INSERT INTO posts (title, content, created_at, author_name) VALUES (?, ?, ?, ?)",
            )
            .bind(&post.title)
            .bind(&post.content)
            .bind(Utc::now())
            .bind(&user.name)
            .execute(pool.get_ref())
            .await;

            match result {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "success"})),
                Err(e) => {
                    println!("Error creating post: {}", e);
                    HttpResponse::InternalServerError().body("Error creating post")
                }
            }
        } else {
            HttpResponse::Forbidden()
                .body(format!("Access Denied: Unauthorized email ({})", user.email))
        }
    } else {
        HttpResponse::Forbidden().body("Access Denied: Not logged in")
    }
}

#[get("/admin/users")]
pub async fn get_admins(pool: web::Data<Pool<Sqlite>>, session: Session) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        if is_admin(pool.get_ref(), &user.email).await {
            let result = sqlx::query_as::<_, AdminUser>("SELECT * FROM admins ORDER BY created_at DESC")
                .fetch_all(pool.get_ref())
                .await;

            match result {
                Ok(admins) => HttpResponse::Ok().json(admins),
                Err(e) => {
                    println!("Error fetching admins: {}", e);
                    HttpResponse::InternalServerError().body("Error fetching admins")
                }
            }
        } else {
            HttpResponse::Forbidden().body("Access Denied")
        }
    } else {
        HttpResponse::Forbidden().body("Access Denied: Not logged in")
    }
}

#[post("/admin/users")]
pub async fn add_admin(
    pool: web::Data<Pool<Sqlite>>,
    session: Session,
    req: web::Json<AddAdminRequest>,
) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        if is_admin(pool.get_ref(), &user.email).await {
            let result = sqlx::query(
                "INSERT INTO admins (email, created_at) VALUES (?, ?)",
            )
            .bind(req.email.trim())
            .bind(Utc::now())
            .execute(pool.get_ref())
            .await;

            match result {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "success"})),
                Err(e) => {
                    println!("Error adding admin: {}", e);
                    HttpResponse::InternalServerError().body("Error adding admin (maybe already exists?)")
                }
            }
        } else {
            HttpResponse::Forbidden().body("Access Denied")
        }
    } else {
        HttpResponse::Forbidden().body("Access Denied: Not logged in")
    }
}

#[delete("/admin/users/{email}")]
pub async fn delete_admin(
    pool: web::Data<Pool<Sqlite>>,
    session: Session,
    email: web::Path<String>,
) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        if is_admin(pool.get_ref(), &user.email).await {
            let email_to_delete = email.into_inner();

            // Check if this is the last admin
            let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM admins")
                .fetch_one(pool.get_ref())
                .await
                .unwrap_or((0,));

            if count.0 <= 1 {
                return HttpResponse::BadRequest().body("Cannot delete the last admin");
            }

            let result = sqlx::query("DELETE FROM admins WHERE email = ?")
                .bind(email_to_delete)
                .execute(pool.get_ref())
                .await;

            match result {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "success"})),
                Err(e) => {
                    println!("Error deleting admin: {}", e);
                    HttpResponse::InternalServerError().body("Error deleting admin")
                }
            }
        } else {
            HttpResponse::Forbidden().body("Access Denied")
        }
    } else {
        HttpResponse::Forbidden().body("Access Denied: Not logged in")
    }
}

#[delete("/admin/posts/{id}")]
pub async fn delete_post(
    pool: web::Data<Pool<Sqlite>>,
    session: Session,
    id: web::Path<i64>,
) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        if is_admin(pool.get_ref(), &user.email).await {
            let post_id = id.into_inner();

            let result = sqlx::query("DELETE FROM posts WHERE id = ?")
                .bind(post_id)
                .execute(pool.get_ref())
                .await;

            match result {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "success"})),
                Err(e) => {
                    println!("Error deleting post: {}", e);
                    HttpResponse::InternalServerError().body("Error deleting post")
                }
            }
        } else {
            HttpResponse::Forbidden().body("Access Denied")
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
pub async fn me(session: Session) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        HttpResponse::Ok().json(user)
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
pub async fn admin_dashboard(pool: web::Data<Pool<Sqlite>>, session: Session) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        if is_admin(pool.get_ref(), &user.email).await {
            HttpResponse::Ok().json(serde_json::json!({
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

    // Database setup
    // Database setup
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:blog.db?mode=rwc".to_string());
    println!("Connecting to database at: {}", database_url);

    let mut options = sqlx::sqlite::SqliteConnectOptions::from_str(&database_url)
        .expect("Failed to parse DATABASE_URL");

    // Force settings for GCS compatibility
    options = options
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Memory)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Off);

    let pool = SqlitePoolOptions::new()
        .connect_with(options)
        .await
        .expect("Failed to connect to database");

    // Create posts table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            author_name TEXT NOT NULL DEFAULT 'Anonymous'
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create posts table");

    // Naive migration for existing tables
    let _ = sqlx::query("ALTER TABLE posts ADD COLUMN author_name TEXT NOT NULL DEFAULT 'Anonymous'")
        .execute(&pool)
        .await;

    // Create admins table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS admins (
            email TEXT PRIMARY KEY,
            created_at DATETIME NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create admins table");

    // Bootstrap initial admin
    let initial_admin = "harrison.dale@googlemail.com";
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM admins")
        .fetch_one(&pool)
        .await
        .expect("Failed to count admins");

    if count.0 == 0 {
        println!("Bootstrapping initial admin: {}", initial_admin);
        sqlx::query("INSERT INTO admins (email, created_at) VALUES (?, ?)")
            .bind(initial_admin)
            .bind(Utc::now())
            .execute(&pool)
            .await
            .expect("Failed to insert initial admin");
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
            .app_data(web::Data::new(pool.clone()))
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
