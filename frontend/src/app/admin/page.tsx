"use client";

import { useEffect, useState } from "react";

export default function AdminPage() {
    const [data, setData] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);

    const [title, setTitle] = useState("");
    const [content, setContent] = useState("");
    const [message, setMessage] = useState("");

    const [admins, setAdmins] = useState<{ email: string; created_at: string }[]>([]);
    const [newAdminEmail, setNewAdminEmail] = useState("");
    const [adminMessage, setAdminMessage] = useState("");

    useEffect(() => {
        // Fetch dashboard data
        fetch("http://localhost:8080/admin/dashboard", {
            credentials: "include",
        })
            .then(async (res) => {
                if (res.ok) {
                    const json = await res.json();
                    setData(json.secret);
                } else {
                    const text = await res.text();
                    setError(text);
                }
            })
            .catch((err) => setError("Error fetching admin data: " + err));

        // Fetch admins list
        fetch("http://localhost:8080/admin/users", {
            credentials: "include",
        })
            .then((res) => res.json())
            .then((data) => setAdmins(data))
            .catch((err) => console.error("Error fetching admins:", err));
    }, []);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setMessage("Submitting...");

        try {
            const res = await fetch("http://localhost:8080/admin/posts", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                credentials: "include",
                body: JSON.stringify({ title, content }),
            });

            if (res.ok) {
                setMessage("Post created successfully!");
                setTitle("");
                setContent("");
            } else {
                const text = await res.text();
                setMessage("Error: " + text);
            }
        } catch (err) {
            setMessage("Error submitting post: " + err);
        }
    };

    const handleAddAdmin = async (e: React.FormEvent) => {
        e.preventDefault();
        setAdminMessage("Adding...");

        try {
            const res = await fetch("http://localhost:8080/admin/users", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                credentials: "include",
                body: JSON.stringify({ email: newAdminEmail }),
            });

            if (res.ok) {
                setAdminMessage("Admin added successfully!");
                setNewAdminEmail("");
                // Refresh list
                fetch("http://localhost:8080/admin/users", { credentials: "include" })
                    .then((res) => res.json())
                    .then((data) => setAdmins(data));
            } else {
                const text = await res.text();
                setAdminMessage("Error: " + text);
            }
        } catch (err) {
            setAdminMessage("Error adding admin: " + err);
        }
    };

    const handleDeleteAdmin = async (email: string) => {
        if (!confirm(`Are you sure you want to remove ${email}?`)) return;
        setAdminMessage(`Deleting ${email}...`);

        try {
            const res = await fetch(`http://localhost:8080/admin/users/${email}`, {
                method: "DELETE",
                credentials: "include",
            });

            if (res.ok) {
                setAdminMessage("Admin removed successfully!");
                // Refresh list
                fetch("http://localhost:8080/admin/users", { credentials: "include" })
                    .then((res) => res.json())
                    .then((data) => setAdmins(data));
            } else {
                const text = await res.text();
                setAdminMessage("Error: " + text);
            }
        } catch (err) {
            setAdminMessage("Error deleting admin: " + err);
        }
    };

    return (
        <div className="min-h-screen p-8 pb-20 gap-16 sm:p-20 font-[family-name:var(--font-geist-sans)]">
            <main className="flex flex-col gap-8 items-center sm:items-start w-full max-w-2xl">
                <h1 className="text-4xl font-bold text-red-600">Admin Dashboard</h1>

                {error ? (
                    <div className="p-4 bg-red-100 border border-red-400 text-red-700 rounded w-full">
                        <h2 className="font-bold">Access Denied</h2>
                        <p>{error}</p>
                        <a href="/" className="text-blue-600 hover:underline mt-2 block">Go Home</a>
                    </div>
                ) : data ? (
                    <div className="w-full flex flex-col gap-8">
                        <div className="p-4 bg-green-100 border border-green-400 text-green-700 rounded">
                            <p className="font-mono text-xl">{data}</p>
                        </div>

                        {/* Blog Post Form */}
                        <div className="p-6 bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
                            <h2 className="text-2xl font-bold mb-4">Create New Post</h2>
                            <form onSubmit={handleSubmit} className="flex flex-col gap-4">
                                <div>
                                    <label className="block text-sm font-medium mb-1">Title</label>
                                    <input
                                        type="text"
                                        value={title}
                                        onChange={(e) => setTitle(e.target.value)}
                                        className="w-full p-2 border rounded dark:bg-gray-700 dark:border-gray-600"
                                        required
                                    />
                                </div>
                                <div>
                                    <label className="block text-sm font-medium mb-1">Content</label>
                                    <textarea
                                        value={content}
                                        onChange={(e) => setContent(e.target.value)}
                                        className="w-full p-2 border rounded h-32 dark:bg-gray-700 dark:border-gray-600"
                                        required
                                    />
                                </div>
                                <button
                                    type="submit"
                                    className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition"
                                >
                                    Publish Post
                                </button>
                                {message && <p className="text-sm font-semibold">{message}</p>}
                            </form>
                        </div>

                        {/* Admin Management */}
                        <div className="p-6 bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
                            <h2 className="text-2xl font-bold mb-4">Manage Admins</h2>

                            <form onSubmit={handleAddAdmin} className="flex gap-2 mb-6">
                                <input
                                    type="email"
                                    value={newAdminEmail}
                                    onChange={(e) => setNewAdminEmail(e.target.value)}
                                    placeholder="new.admin@example.com"
                                    className="flex-1 p-2 border rounded dark:bg-gray-700 dark:border-gray-600"
                                    required
                                />
                                <button
                                    type="submit"
                                    className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 transition"
                                >
                                    Add Admin
                                </button>
                            </form>
                            {adminMessage && <p className="text-sm font-semibold mb-4">{adminMessage}</p>}

                            <h3 className="font-bold mb-2">Current Admins</h3>
                            <ul className="list-disc list-inside">
                                {admins.map((admin) => (
                                    <li key={admin.email} className="text-gray-700 dark:text-gray-300 flex justify-between items-center">
                                        <span>
                                            {admin.email} <span className="text-xs text-gray-500">({new Date(admin.created_at).toLocaleDateString()})</span>
                                        </span>
                                        <button
                                            onClick={() => handleDeleteAdmin(admin.email)}
                                            className="px-2 py-1 bg-red-600 text-white text-xs rounded hover:bg-red-700 transition ml-4"
                                        >
                                            Delete
                                        </button>
                                    </li>
                                ))}
                            </ul>
                        </div>

                        <a href="/" className="text-blue-600 hover:underline">Go Home</a>
                    </div>
                ) : (
                    <p>Loading...</p>
                )}
            </main>
        </div>
    );
}
