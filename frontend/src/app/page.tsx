"use client";

import { useEffect, useState } from "react";

interface User {
  email: String;
  name: String;
}

interface BlogPost {
  id: number;
  title: string;
  content: string;
  created_at: string;
}

export default function Home() {
  const [message, setMessage] = useState<string>("");
  const [user, setUser] = useState<User | null>(null);
  const [posts, setPosts] = useState<BlogPost[]>([]);

  useEffect(() => {
    // Fetch hello message
    fetch("http://localhost:8080/api/hello")
      .then((res) => res.json())
      .then((data) => setMessage(data.message))
      .catch((err) => console.error("Error fetching message:", err));

    // Fetch user info
    fetch("http://localhost:8080/auth/me", {
      credentials: "include",
    })
      .then((res) => {
        if (res.ok) return res.json();
        return null;
      })
      .then((data) => setUser(data))
      .catch((err) => console.error("Error fetching user:", err));

    // Fetch blog posts
    fetch("http://localhost:8080/api/posts")
      .then((res) => res.json())
      .then((data) => setPosts(data))
      .catch((err) => console.error("Error fetching posts:", err));
  }, []);

  const handleLogin = () => {
    window.location.href = "http://localhost:8080/auth/login";
  };

  const handleLogout = () => {
    window.location.href = "http://localhost:8080/auth/logout";
  };

  return (
    <div className="grid grid-rows-[20px_1fr_20px] items-center justify-items-center min-h-screen p-8 pb-20 gap-16 sm:p-20 font-[family-name:var(--font-geist-sans)]">
      <main className="flex flex-col gap-8 row-start-2 items-center sm:items-start w-full max-w-2xl">
        <h1 className="text-4xl font-bold text-center sm:text-left">
          {message || "Loading..."}
        </h1>

        <div className="flex flex-col items-center gap-4">
          {user ? (
            <>
              <p className="text-xl">Hello, {user.name}!</p>
              <div className="flex gap-2">
                <button
                  onClick={handleLogout}
                  className="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 transition"
                >
                  Logout
                </button>
                <a
                  href="/admin"
                  className="px-4 py-2 bg-gray-800 text-white rounded hover:bg-gray-900 transition"
                >
                  Admin Dashboard
                </a>
              </div>
            </>
          ) : (
            <button
              onClick={handleLogin}
              className="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition font-semibold shadow-lg"
            >
              Login with Google
            </button>
          )}
        </div>

        <div className="w-full mt-8">
          <h2 className="text-2xl font-bold mb-4">Latest Posts</h2>
          <div className="flex flex-col gap-4">
            {posts.length > 0 ? (
              posts.map((post) => (
                <div key={post.id} className="p-6 bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
                  <h3 className="text-xl font-bold mb-2">{post.title}</h3>
                  <p className="text-gray-600 dark:text-gray-300 whitespace-pre-wrap">{post.content}</p>
                  <p className="text-sm text-gray-400 mt-4">
                    {new Date(post.created_at).toLocaleDateString()}
                  </p>
                </div>
              ))
            ) : (
              <p className="text-gray-500 italic">No posts yet.</p>
            )}
          </div>
        </div>
      </main>
    </div>
  );
}
