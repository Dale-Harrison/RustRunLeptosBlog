use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use wasm_bindgen::JsValue;
use chrono::DateTime;
use pulldown_cmark::{Parser, html, Options};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub email: String,
    pub name: String,
    #[serde(default)]
    pub is_admin: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub author_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdminUser {
    pub email: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub post_id: i64,
    pub author_name: String,
    pub content: String,
    pub created_at: String,
}

fn format_date(date_str: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        dt.format("%B %d, %Y at %I:%M %p").to_string()
    } else {
        date_str.to_string()
    }
}

fn render_markdown(markdown_input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(markdown_input, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

// --- API Helpers ---

async fn fetch_user() -> Option<User> {
    Request::get("/auth/me").send().await.ok()?.json().await.ok()
}

async fn fetch_posts() -> Vec<BlogPost> {
    Request::get(&format!("/api/posts?ts={}", js_sys::Date::now()))
        .send().await.unwrap().json().await.unwrap_or_default()
}

async fn fetch_post(id: i64) -> Option<BlogPost> {
    Request::get(&format!("/api/posts/{}?ts={}", id, js_sys::Date::now()))
        .send().await.ok()?.json().await.ok()
}

async fn fetch_comments(post_id: i64) -> Vec<Comment> {
    Request::get(&format!("/api/posts/{}/comments?ts={}", post_id, js_sys::Date::now()))
        .send().await.unwrap()
        .json().await.unwrap_or_default()
}

async fn create_comment_api(post_id: i64, content: String) -> Option<Comment> {
    Request::post(&format!("/api/posts/{}/comments", post_id))
        .header("Content-Type", "application/json")
        .body(JsValue::from_str(&serde_json::json!({ "content": content }).to_string()))
        .unwrap()
        .send().await.ok()?
        .json().await.ok()
}

async fn delete_comment_api(id: i64) -> bool {
    Request::delete(&format!("/admin/comments/{}", id))
         .send().await.map(|r| r.ok()).unwrap_or(false)
}

#[component]
pub fn NavBar(user: Resource<(), Option<User>>) -> impl IntoView {
    let login = move |_| { window().location().set_href("/auth/login").unwrap(); };
    let logout = move |_| { window().location().set_href("/auth/logout").unwrap(); };

    view! {
        <header class="w-full flex justify-end items-center p-4 gap-4 border-b">
            <Suspense fallback=move || view! { "Loading..." }>
                {move || {
                    user.get().map(|u| {
                        match u {
                            Some(user) => view! {
                                <span class="text-lg font-medium">"Hello, " {user.name} "!"</span>
                                <button class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 transition" on:click=logout>
                                    "Logout"
                                </button>
                                <A href="/admin" class="px-4 py-2 bg-gray-800 text-white rounded hover:bg-gray-900 transition">
                                    "Admin Dashboard"
                                </A>
                            }.into_view(),
                            None => view! {
                                <button class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition font-semibold" on:click=login>
                                    "Login with Google"
                                </button>
                            }.into_view()
                        }
                    })
                }}
            </Suspense>
        </header>
    }
}

#[component]
pub fn Home() -> impl IntoView {
    // ... (Resources)
    let user = create_resource(|| (), |_| async move { fetch_user().await });
    let posts = create_resource(|| (), |_| async move { fetch_posts().await });
    let hello_message = create_resource(|| (), |_| async move {
        Request::get("/api/hello")
            .send().await.unwrap()
            .json::<serde_json::Value>().await.ok()
            .and_then(|v| v["message"].as_str().map(String::from))
            .unwrap_or_else(|| "Loading...".to_string())
    });

    view! {
        <div class="min-h-screen flex flex-col font-sans">
            <NavBar user=user />
            <div class="flex-1 grid items-start justify-items-center p-4 pt-4 pb-20 gap-8 sm:p-10 sm:pt-8">
                <main class="flex flex-col gap-8 items-center sm:items-start w-full max-w-2xl">
                    <h1 class="text-4xl font-bold text-center sm:text-left">
                        <Suspense fallback=move || "Loading...">
                            {move || hello_message.get().unwrap_or_default()}
                        </Suspense>
                    </h1>
                    <div class="w-full mt-8">
                        <h2 class="text-2xl font-bold mb-4">"Latest Posts"</h2>
                        <div class="flex flex-col gap-4">
                            <Suspense fallback=move || "Loading posts...">
                                {move || {
                                    posts.get().map(|posts| {
                                        if posts.is_empty() {
                                            view! { <p class="text-gray-500 italic">"No posts yet."</p> }.into_view()
                                        } else {
                                            posts.into_iter().map(|post| {
                                                let snippet = post.content.split("\n\n").next().unwrap_or("").to_string();
                                                view! {
                                                    <div class="p-6 bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
                                                        <h3 class="text-xl font-bold mb-2">
                                                            <A href=format!("/posts/{}", post.id) class="hover:text-blue-600 transition">
                                                                {post.title}
                                                            </A>
                                                        </h3>
                                                        <div 
                                                            class="text-gray-600 dark:text-gray-300 mb-4 prose dark:prose-invert max-w-none"
                                                            inner_html=render_markdown(&snippet)
                                                        ></div>
                                                        <div class="flex justify-between items-center mt-4">
                                                            <div class="text-sm text-gray-400">
                                                                <span>"By " {post.author_name}</span>
                                                                <span class="mx-2">"•"</span>
                                                                <span>{format_date(&post.created_at)}</span>
                                                            </div>
                                                            <A href=format!("/posts/{}", post.id) class="text-blue-600 hover:text-blue-800 font-medium text-sm">
                                                                "Read more →"
                                                            </A>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()
                                        }
                                    })
                                }}
                            </Suspense>
                        </div>
                    </div>
                </main>
            </div>
        </div>
    }
}

#[component]
pub fn Post() -> impl IntoView {
    let params = use_params_map();
    let post = create_resource(
        move || params.get().get("id").cloned().unwrap_or_default(),
        |id| async move {
             if let Ok(id_num) = id.parse::<i64>() {
                 fetch_post(id_num).await
             } else {
                 None
             }
        }
    );

    let user = create_resource(|| (), |_| async move { fetch_user().await });

    let (comments_version, set_comments_version) = create_signal(0);
    let comments = create_resource(
        move || (params.get().get("id").cloned().unwrap_or_default(), comments_version.get()),
        |(id, _)| async move {
             leptos::logging::log!("Resource: Fetching comments for id: {:?}", id);
             if let Ok(id_num) = id.parse::<i64>() {
                 let res = fetch_comments(id_num).await;
                 leptos::logging::log!("Resource: Got {} comments from valid parse", res.len());
                 res
             } else {
                 vec![]
             }
        }
    );

    let (new_comment, set_new_comment) = create_signal(String::new());
    let submit_comment = create_action(move |(id, content): &(i64, String)| {
        let id = *id;
        let content = content.clone();
        leptos::logging::log!("Action: Submitting comment for post {}", id);
        async move {
            if create_comment_api(id, content).await.is_some() {
                leptos::logging::log!("API Success: Comment created. Clearing input.");
                set_new_comment.set("".to_string());
                
                spawn_local(async move {
                    leptos::logging::log!("Background: Waiting 100ms...");
                    gloo_timers::future::TimeoutFuture::new(100).await;
                    leptos::logging::log!("Background: Refreshing comments now.");
                    set_comments_version.update(|v| *v += 1);
                });
            }
        }
    });

    let delete_comment = create_action(move |id: &i64| {
        let id = *id;
        async move {
            if delete_comment_api(id).await {
                set_comments_version.update(|v| *v += 1);
            }
        }
    });

    let on_submit_comment = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
         if let Ok(id_num) = params.get().get("id").unwrap_or(&"".to_string()).parse::<i64>() {
             if !new_comment.get().trim().is_empty() {
                 submit_comment.dispatch((id_num, new_comment.get()));
             }
         }
    };

    view! {
        <div class="min-h-screen flex flex-col font-sans">
             <header class="w-full flex justify-between items-center p-4 gap-4">
                <A href="/" class="text-xl font-bold hover:text-blue-600 transition">"← Back to Home"</A>
             </header>

             <div class="flex-1 grid items-start justify-items-center p-4 pt-4 pb-20 gap-8 sm:p-10 sm:pt-8">
                <main class="flex flex-col gap-8 items-center sm:items-start w-full max-w-2xl">
                    <Suspense fallback=move || "Loading post...">
                        {move || {
                             post.get().map(|p| {
                                match p {
                                    Some(post) => view! {
                                        <div class="w-full">
                                            <article class="w-full bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8 border border-gray-200 dark:border-gray-700">
                                                <h1 class="text-4xl font-bold mb-4">{post.title}</h1>
                                                <div class="text-sm text-gray-400 mb-8 pb-4 border-b border-gray-200 dark:border-gray-700 flex gap-2">
                                                     <span class="font-semibold">{post.author_name}</span>
                                                     <span>"•"</span>
                                                     <span>{format_date(&post.created_at)}</span>
                                                </div>
                                                <div class="prose dark:prose-invert max-w-none" inner_html=render_markdown(&post.content)>
                                                </div>
                                            </article>

                                            // Comments Section
                                            <div class="w-full mt-10 max-w-2xl">
                                                 <h3 class="text-2xl font-bold mb-6">"Comments"</h3>
                                                 
                                                 <Suspense fallback=move || "">
                                                    {move || {
                                                         user.get().map(|u| {
                                                             match u {
                                                                 Some(_) => view! {
                                                                     <form on:submit=on_submit_comment class="mb-8 p-6 bg-gray-50 dark:bg-gray-800 rounded-lg">
                                                                         <label class="block text-sm font-medium mb-2">"Leave a comment"</label>
                                                                         <textarea
                                                                            class="w-full p-3 border rounded mb-4 dark:bg-gray-700 dark:border-gray-600"
                                                                            rows="3"
                                                                            prop:value=new_comment
                                                                            on:input=move |ev| set_new_comment.set(event_target_value(&ev))
                                                                            placeholder="Share your thoughts..."
                                                                            required
                                                                         />
                                                                         <button
                                                                            type="submit"
                                                                            class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition font-medium"
                                                                            disabled=move || submit_comment.pending().get()
                                                                         >
                                                                            {move || if submit_comment.pending().get() { "Posting..." } else { "Post Comment" }}
                                                                         </button>
                                                                     </form>
                                                                 }.into_view(),
                                                                 None => view! {
                                                                     <div class="mb-8 p-6 bg-blue-50 dark:bg-gray-900 border border-blue-200 dark:border-gray-700 rounded-lg text-center">
                                                                         <p class="mb-2">"Please login to leave a comment."</p>
                                                                         <a href="/auth/login" class="text-blue-600 hover:underline font-semibold">"Login with Google"</a>
                                                                     </div>
                                                                 }.into_view()
                                                             }
                                                         })
                                                    }}
                                                 </Suspense>

                                                 <div class="flex flex-col gap-6">
                                                     <Suspense fallback=move || "Loading comments...">
                                                         {move || {
                                                             comments.get().map(|comments| {
                                                                 if comments.is_empty() {
                                                                     view! { <p class="text-gray-500 italic">"No comments yet. Be the first to share your thoughts!"</p> }.into_view()
                                                                 } else {
                                                                     comments.into_iter().map(|comment| {
                                                                         view! {
                                                                             <div class="flex gap-4 p-4 bg-white dark:bg-gray-800 border-b border-gray-100 dark:border-gray-700 last:border-0">
                                                                                 <div class="flex-shrink-0 w-10 h-10 bg-gray-200 dark:bg-gray-700 rounded-full flex items-center justify-center text-gray-500 font-bold text-lg">
                                                                                      {comment.author_name.chars().next().unwrap_or('?')}
                                                                                 </div>
                                                                                 <div class="flex-1">
                                                                                     <div class="flex justify-between items-start">
                                                                                         <div class="flex items-center gap-2 mb-1">
                                                                                             <span class="font-bold">{comment.author_name}</span>
                                                                                             <span class="text-xs text-gray-400">{format_date(&comment.created_at)}</span>
                                                                                         </div>
                                                                                         {move || user.get().flatten().map(|u| {
                                                                                             if u.is_admin {
                                                                                                 view! {
                                                                                                     <button
                                                                                                         on:click=move |_| delete_comment.dispatch(comment.id)
                                                                                                         class="text-red-500 hover:text-red-700 text-xs font-bold"
                                                                                                     >
                                                                                                         "Delete"
                                                                                                     </button>
                                                                                                 }.into_view()
                                                                                             } else {
                                                                                                 view! { <span/> }.into_view()
                                                                                             }
                                                                                         })}
                                                                                     </div>
                                                                                     <p class="text-gray-700 dark:text-gray-300 whitespace-pre-wrap">{comment.content}</p>
                                                                                 </div>
                                                                             </div>
                                                                         }
                                                                     }).collect_view()
                                                                 }
                                                             })
                                                         }}
                                                     </Suspense>
                                                 </div>
                                            </div>
                                        </div>
                                    }.into_view(),
                                    None => view! { <p class="text-red-500">"Post not found"</p> }.into_view()
                                }
                             })
                        }}
                    </Suspense>
                </main>
             </div>
        </div>
    }
}

#[component]
pub fn Admin() -> impl IntoView {
    // State
    let (title, set_title) = create_signal(String::new());
    let (content, set_content) = create_signal(String::new());
    let (editing_id, set_editing_id) = create_signal(None::<i64>);
    let (message, set_message) = create_signal(String::new());
    
    let (new_admin_email, set_new_admin_email) = create_signal(String::new());
    let (admin_message, set_admin_message) = create_signal(String::new());
    let (post_message, set_post_message) = create_signal(String::new());

    // Dashboard Data Resource
    let dashboard_data = create_resource(|| (), |_| async move {
        Request::get(&format!("/admin/dashboard?ts={}", js_sys::Date::now()))
            .send().await.ok()?.json::<serde_json::Value>().await.ok()
    });

    // Admins List Resource
    let (admins_version, set_admins_version) = create_signal(0);
    let admins = create_resource(move || admins_version.get(), |_| async move {
        Request::get(&format!("/admin/users?ts={}", js_sys::Date::now()))
            .send().await.unwrap().json::<Vec<AdminUser>>().await.unwrap_or_default()
    });

    // Posts List Resource
    let (posts_version, set_posts_version) = create_signal(0);
    let posts = create_resource(move || posts_version.get(), |_| async move {
        fetch_posts().await
    });

    // Actions
    let submit_post = create_action(move |input: &(Option<i64>, String, String)| {
        let (id, t, c) = input.clone();
        async move {
            let result = if let Some(id) = id {
                Request::put(&format!("/admin/posts/{}", id))
                    .header("Content-Type", "application/json")
                    .body(JsValue::from_str(&serde_json::json!({ "title": t, "content": c }).to_string()))
                    .unwrap()
                    .send().await
            } else {
                 Request::post("/admin/posts")
                    .header("Content-Type", "application/json")
                    .body(JsValue::from_str(&serde_json::json!({ "title": t, "content": c }).to_string()))
                    .unwrap()
                    .send().await
            };
            
            match result {
                Ok(res) if res.ok() => {
                    set_message.set(if id.is_some() { "Post updated successfully!" } else { "Post created successfully!" }.to_string());
                    set_title.set("".to_string());
                    set_content.set("".to_string());
                    set_editing_id.set(None);
                    
                    spawn_local(async move {
                        gloo_timers::future::TimeoutFuture::new(100).await;
                        set_posts_version.update(|v| *v += 1);
                    });
                }
                Ok(res) => set_message.set(format!("Error: {}", res.status_text())),
                Err(e) => set_message.set(format!("Error submitting post: {}", e)),
            }
        }
    });

    let delete_post = create_action(move |id: &i64| {
        let id = *id;
        async move {
            if Request::delete(&format!("/admin/posts/{}", id)).send().await.is_ok() {
                 set_post_message.set("Post deleted successfully!".to_string());
                 
                 spawn_local(async move {
                     gloo_timers::future::TimeoutFuture::new(100).await;
                     set_posts_version.update(|v| *v += 1);
                 });
                 if editing_id.get_untracked() == Some(id) {
                     set_editing_id.set(None);
                     set_title.set("".to_string());
                     set_content.set("".to_string());
                 }
            } else {
                set_post_message.set("Error deleting post".to_string());
            }
        }
    });

    let add_admin = create_action(move |email: &String| {
        let email = email.clone();
        async move {
            let res = Request::post("/admin/users")
                .header("Content-Type", "application/json")
                .body(JsValue::from_str(&serde_json::json!({ "email": email }).to_string()))
                .unwrap()
                .send().await;
            
             match res {
                Ok(r) if r.ok() => {
                    set_admin_message.set("Admin added successfully!".to_string());
                    set_new_admin_email.set("".to_string());
                    
                    spawn_local(async move {
                         gloo_timers::future::TimeoutFuture::new(100).await;
                         set_admins_version.update(|v| *v += 1);
                    });
                }
                _ => set_admin_message.set("Error adding admin".to_string()),
            }
        }
    });

    let delete_admin = create_action(move |email: &String| {
        let email = email.clone();
        async move {
            if Request::delete(&format!("/admin/users/{}", email)).send().await.is_ok() {
                 set_admin_message.set("Admin removed successfully!".to_string());
                 
                 spawn_local(async move {
                      gloo_timers::future::TimeoutFuture::new(100).await;
                      set_admins_version.update(|v| *v += 1);
                 });
            } else {
                set_admin_message.set("Error removing admin".to_string());
            }
        }
    });

    // Handlers
    let on_submit_post = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        submit_post.dispatch((editing_id.get(), title.get(), content.get()));
    };

    let on_edit_post = move |post: BlogPost| {
        set_editing_id.set(Some(post.id));
        set_title.set(post.title);
        set_content.set(post.content);
        set_message.set("".to_string());
        window().scroll_to_with_x_and_y(0.0, 0.0);
    };

    let on_cancel_edit = move |_| {
        set_editing_id.set(None);
        set_title.set("".to_string());
        set_content.set("".to_string());
        set_message.set("".to_string());
    };

    let on_add_admin = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        add_admin.dispatch(new_admin_email.get());
    };

    view! {
        <div class="min-h-screen p-8 pb-20 gap-16 sm:p-20 font-sans">
             <main class="flex flex-col gap-8 items-center sm:items-start w-full max-w-none px-4">
                <h1 class="text-4xl font-bold text-red-600">"Admin Dashboard"</h1>

                <Suspense fallback=move || "Loading dashboard...">
                    {move || {
                        dashboard_data.get().map(|data| {
                             match data {
                                Some(d) => view! {
                                    <div class="w-full flex flex-col gap-8">
                                        <div class="p-4 bg-green-100 border border-green-400 text-green-700 rounded">
                                            <p class="font-mono text-xl">{d["secret"].as_str().unwrap_or("").to_string()}</p>
                                        </div>

                                        // Blog Post Form
                                        <div class="p-6 bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
                                            <h2 class="text-2xl font-bold mb-4">{move || if editing_id.get().is_some() { "Edit Post" } else { "Create New Post" }}</h2>
                                            <form on:submit=on_submit_post class="flex flex-col gap-4">
                                                <div>
                                                    <label class="block text-sm font-medium mb-1">"Title"</label>
                                                    <input
                                                        type="text"
                                                        class="w-full p-2 border rounded dark:bg-gray-700 dark:border-gray-600"
                                                        required
                                                        prop:value=title
                                                        on:input=move |ev| set_title.set(event_target_value(&ev))
                                                    />
                                                </div>
                                                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                                                    <div>
                                                        <label class="block text-sm font-medium mb-1">"Content (Markdown)"</label>
                                                        <textarea
                                                            class="w-full p-2 border rounded h-96 font-mono text-sm dark:bg-gray-700 dark:border-gray-600"
                                                            required
                                                            prop:value=content
                                                            on:input=move |ev| set_content.set(event_target_value(&ev))
                                                        />
                                                    </div>
                                                    <div>
                                                        <label class="block text-sm font-medium mb-1">"Preview"</label>
                                                        <div 
                                                            class="w-full p-4 border rounded h-96 overflow-y-auto bg-gray-50 dark:bg-gray-900 prose dark:prose-invert max-w-none"
                                                            inner_html=move || render_markdown(&content.get())
                                                        >
                                                        </div>
                                                    </div>
                                                </div>
                                                <div class="mt-2 p-4 bg-gray-50 dark:bg-gray-900 rounded border border-gray-200 dark:border-gray-700">
                                                    <p class="text-sm font-bold text-gray-500 mb-2">"Markdown Cheat Sheet"</p>
                                                    <div class="grid grid-cols-2 md:grid-cols-4 gap-2 text-xs font-mono text-gray-600 dark:text-gray-400">
                                                        <span>"# Heading 1"</span>
                                                        <span>"## Heading 2"</span>
                                                        <span>"**Bold**"</span>
                                                        <span>"*Italic*"</span>
                                                        <span>"[Link](url)"</span>
                                                        <span>"- List Item"</span>
                                                        <span>"> Blockquote"</span>
                                                        <span>"`Code`"</span>
                                                    </div>
                                                </div>
                                                <div class="flex gap-2">
                                                    <button
                                                        type="submit"
                                                        class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition"
                                                        disabled=move || submit_post.pending().get()
                                                    >
                                                        {move || if editing_id.get().is_some() { "Update Post" } else { "Publish Post" }}
                                                    </button>
                                                    {move || editing_id.get().is_some().then(|| view! {
                                                        <button
                                                            type="button"
                                                            on:click=on_cancel_edit
                                                            class="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600 transition"
                                                        >
                                                            "Cancel"
                                                        </button>
                                                    })}
                                                </div>
                                                <p class="text-sm font-semibold">{move || message.get()}</p>
                                            </form>
                                        </div>

                                        // Manage Posts
                                        <div class="p-6 bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
                                            <h2 class="text-2xl font-bold mb-4">"Manage Existing Posts"</h2>
                                            <p class="text-sm font-semibold mb-4">{move || post_message.get()}</p>

                                            <div class="flex flex-col gap-2">
                                                <Suspense fallback=move || "Loading posts...">
                                                    {move || posts.get().map(|posts| {
                                                        posts.into_iter().map(|post| {
                                                            let post_clone = post.clone();
                                                            view! {
                                                                <div class="text-gray-700 dark:text-gray-300 flex justify-between items-center bg-gray-50 p-2 rounded">
                                                                    <span>
                                                                        {post.title}
                                                                        <span class="text-xs text-gray-500 ml-2">"("{post.created_at}")"</span>
                                                                    </span>
                                                                    <div class="flex gap-2 ml-4">
                                                                        <button
                                                                             on:click=move |_| on_edit_post(post_clone.clone())
                                                                             class="px-2 py-1 bg-yellow-600 text-white text-xs rounded hover:bg-yellow-700 transition"
                                                                        >
                                                                            "Edit"
                                                                        </button>
                                                                        <button
                                                                             on:click=move |_| delete_post.dispatch(post.id)
                                                                             class="px-2 py-1 bg-red-600 text-white text-xs rounded hover:bg-red-700 transition"
                                                                        >
                                                                            "Delete"
                                                                        </button>
                                                                    </div>
                                                                </div>
                                                            }
                                                        }).collect_view()
                                                    })}
                                                </Suspense>
                                            </div>
                                        </div>

                                        // Manage Admins
                                        <div class="p-6 bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
                                            <h2 class="text-2xl font-bold mb-4">"Manage Admins"</h2>

                                            <form on:submit=on_add_admin class="flex gap-2 mb-6">
                                                <input
                                                    type="email"
                                                    placeholder="new.admin@example.com"
                                                    class="flex-1 p-2 border rounded dark:bg-gray-700 dark:border-gray-600"
                                                    required
                                                    prop:value=new_admin_email
                                                    on:input=move |ev| set_new_admin_email.set(event_target_value(&ev))
                                                />
                                                <button
                                                    type="submit"
                                                    class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 transition"
                                                >
                                                    "Add Admin"
                                                </button>
                                            </form>
                                            <p class="text-sm font-semibold mb-4">{move || admin_message.get()}</p>

                                            <h3 class="font-bold mb-2">"Current Admins"</h3>
                                            <div class="flex flex-col gap-2">
                                                <Suspense fallback=move || "Loading admins...">
                                                    {move || admins.get().map(|admins| {
                                                        admins.into_iter().map(|admin| {
                                                             let email = admin.email.clone();
                                                             view! {
                                                                <div class="text-gray-700 dark:text-gray-300 flex justify-between items-center bg-gray-50 p-2 rounded">
                                                                    <span>
                                                                        {admin.email}
                                                                        <span class="text-xs text-gray-500 ml-2">"("{admin.created_at}")"</span>
                                                                    </span>
                                                                    <button
                                                                        on:click=move |_| delete_admin.dispatch(email.clone())
                                                                        class="px-2 py-1 bg-red-600 text-white text-xs rounded hover:bg-red-700 transition ml-4"
                                                                    >
                                                                        "Delete"
                                                                    </button>
                                                                </div>
                                                             }
                                                        }).collect_view()
                                                    })}
                                                </Suspense>
                                            </div>
                                        </div>
                                         <A href="/" class="text-blue-600 hover:underline">"Go Home"</A>
                                    </div>
                                }.into_view(),
                                None => view! {
                                     <div class="p-4 bg-red-100 border border-red-400 text-red-700 rounded w-full">
                                        <h2 class="font-bold">"Access Denied"</h2>
                                        <p>"You must be logged in as an admin to view this page."</p>
                                        <A href="/" class="text-blue-600 hover:underline mt-2 block">"Go Home"</A>
                                    </div>
                                }.into_view()
                            }
                        })
                    }}
                </Suspense>
            </main>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/" view=Home />
                <Route path="/posts/:id" view=Post />
                <Route path="/admin" view=Admin />
            </Routes>
        </Router>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
