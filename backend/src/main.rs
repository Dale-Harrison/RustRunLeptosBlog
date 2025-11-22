use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::{Key, SameSite}, delete, get, post, web, App, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::env;

#[derive(Serialize, Deserialize)]
struct User {
    email: String,
    name: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
struct BlogPost {
    id: i64,
    title: String,
    content: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct CreatePostRequest {
    title: String,
    content: String,
}

#[derive(Deserialize)]
struct AuthRequest {
    code: String,
    state: String,
}

#[get("/api/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "In the Dusty Clockless Hours"
    }))
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
struct AdminUser {
    email: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct AddAdminRequest {
    email: String,
}

// Helper to check if a user is an admin
async fn is_admin(pool: &Pool<Sqlite>, email: &str) -> bool {
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
async fn get_posts(pool: web::Data<Pool<Sqlite>>) -> impl Responder {
    let result = sqlx::query_as::<_, BlogPost>("SELECT * FROM posts ORDER BY created_at DESC")
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

#[post("/admin/posts")]
async fn create_post(
    pool: web::Data<Pool<Sqlite>>,
    session: Session,
    post: web::Json<CreatePostRequest>,
) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        if is_admin(pool.get_ref(), &user.email).await {
            let result = sqlx::query(
                "INSERT INTO posts (title, content, created_at) VALUES (?, ?, ?)",
            )
            .bind(&post.title)
            .bind(&post.content)
            .bind(Utc::now())
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
async fn get_admins(pool: web::Data<Pool<Sqlite>>, session: Session) -> impl Responder {
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
async fn add_admin(
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
async fn delete_admin(
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

#[get("/auth/login")]
async fn login(client: web::Data<BasicClient>, session: Session) -> impl Responder {
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
async fn callback(
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
async fn me(session: Session) -> impl Responder {
    if let Ok(Some(user)) = session.get::<User>("user") {
        HttpResponse::Ok().json(user)
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[get("/auth/logout")]
async fn logout(session: Session) -> impl Responder {
    session.purge();
    HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish()
}

#[get("/admin/dashboard")]
async fn admin_dashboard(pool: web::Data<Pool<Sqlite>>, session: Session) -> impl Responder {
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
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

    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:8080/auth/callback".to_string())
            .expect("Invalid redirect URL"),
    );

    // Database setup
    let pool = SqlitePoolOptions::new()
        .connect("sqlite:blog.db?mode=rwc")
        .await
        .expect("Failed to connect to database");

    // Create posts table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at DATETIME NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create posts table");

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

    // Generate a random secret key for session cookies
    let secret_key = Key::generate();

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
            .cookie_secure(false)
            .cookie_same_site(SameSite::Lax)
            .build())
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(pool.clone()))
            .service(hello)
            .service(login)
            .service(callback)
            .service(me)
            .service(logout)
            .service(admin_dashboard)
            .service(get_posts)
            .service(create_post)
            .service(get_admins)
            .service(add_admin)
            .service(delete_admin)
            .service(
                Files::new("/", "./static")
                    .index_file("index.html")
                    .default_handler(web::get().to(|| async {
                        NamedFile::open("./static/index.html")
                    }))
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
