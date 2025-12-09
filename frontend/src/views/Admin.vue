<template>
  <div class="min-h-screen p-8 pb-20 gap-16 sm:p-20 font-sans">
    <main class="flex flex-col gap-8 items-center sm:items-start w-full max-w-2xl">
      <h1 class="text-4xl font-bold text-red-600">Admin Dashboard</h1>

      <div
        v-if="error"
        class="p-4 bg-red-100 border border-red-400 text-red-700 rounded w-full"
      >
        <h2 class="font-bold">Access Denied</h2>
        <p>{{ error }}</p>
        <router-link to="/" class="text-blue-600 hover:underline mt-2 block">Go Home</router-link>
      </div>

      <div v-else-if="data" class="w-full flex flex-col gap-8">
        <div class="p-4 bg-green-100 border border-green-400 text-green-700 rounded">
          <p class="font-mono text-xl">{{ data }}</p>
        </div>

        <!-- Blog Post Form -->
        <div class="p-6 bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
          <h2 class="text-2xl font-bold mb-4">{{ editingId ? 'Edit Post' : 'Create New Post' }}</h2>
          <form @submit.prevent="handleSubmit" class="flex flex-col gap-4">
            <div>
              <label class="block text-sm font-medium mb-1">Title</label>
              <input
                v-model="title"
                type="text"
                class="w-full p-2 border rounded dark:bg-gray-700 dark:border-gray-600"
                required
              />
            </div>
            <div>
              <label class="block text-sm font-medium mb-1">Content</label>
              <textarea
                v-model="content"
                class="w-full p-2 border rounded h-32 dark:bg-gray-700 dark:border-gray-600"
                required
              />
            </div>
            <div class="flex gap-2">
              <button
                type="submit"
                class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition"
              >
                {{ editingId ? 'Update Post' : 'Publish Post' }}
              </button>
              <button
                v-if="editingId"
                type="button"
                @click="cancelEdit"
                class="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600 transition"
              >
                Cancel
              </button>
            </div>
            <p v-if="message" class="text-sm font-semibold">{{ message }}</p>
          </form>
        </div>

        <!-- Manage Posts -->
        <div class="p-6 bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
          <h2 class="text-2xl font-bold mb-4">Manage Existing Posts</h2>
          <p v-if="postMessage" class="text-sm font-semibold mb-4">{{ postMessage }}</p>

          <ul class="list-disc list-inside space-y-2">
            <li
              v-for="post in posts"
              :key="post.id"
              class="text-gray-700 dark:text-gray-300 flex justify-between items-center"
            >
              <span>
                {{ post.title }}
                <span class="text-xs text-gray-500">({{ new Date(post.created_at).toLocaleDateString() }})</span>
              </span>
              <div class="flex gap-2 ml-4">
                <button
                  @click="handleEditPost(post)"
                  class="px-2 py-1 bg-yellow-600 text-white text-xs rounded hover:bg-yellow-700 transition"
                >
                  Edit
                </button>
                <button
                  @click="handleDeletePost(post.id)"
                  class="px-2 py-1 bg-red-600 text-white text-xs rounded hover:bg-red-700 transition"
                >
                  Delete
                </button>
              </div>
            </li>
          </ul>
          <p v-if="posts.length === 0" class="text-gray-500">No posts found.</p>
        </div>

        <!-- Admin Management -->
        <div class="p-6 bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
          <h2 class="text-2xl font-bold mb-4">Manage Admins</h2>

          <form @submit.prevent="handleAddAdmin" class="flex gap-2 mb-6">
            <input
              v-model="newAdminEmail"
              type="email"
              placeholder="new.admin@example.com"
              class="flex-1 p-2 border rounded dark:bg-gray-700 dark:border-gray-600"
              required
            />
            <button
              type="submit"
              class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700 transition"
            >
              Add Admin
            </button>
          </form>
          <p v-if="adminMessage" class="text-sm font-semibold mb-4">{{ adminMessage }}</p>

          <h3 class="font-bold mb-2">Current Admins</h3>
          <ul class="list-disc list-inside">
            <li
              v-for="admin in admins"
              :key="admin.email"
              class="text-gray-700 dark:text-gray-300 flex justify-between items-center"
            >
              <span>
                {{ admin.email }}
                <span class="text-xs text-gray-500">({{ new Date(admin.created_at).toLocaleDateString() }})</span>
              </span>
              <button
                @click="handleDeleteAdmin(admin.email)"
                class="px-2 py-1 bg-red-600 text-white text-xs rounded hover:bg-red-700 transition ml-4"
              >
                Delete
              </button>
            </li>
          </ul>
        </div>

        <router-link to="/" class="text-blue-600 hover:underline">Go Home</router-link>
      </div>

      <p v-else>Loading...</p>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface Admin {
  email: string
  created_at: string
}

const data = ref<string | null>(null)
const error = ref<string | null>(null)

const title = ref('')
const content = ref('')
const message = ref('')

const admins = ref<Admin[]>([])
const newAdminEmail = ref('')
const adminMessage = ref('')

const editingId = ref<number | null>(null)

interface BlogPost {
  id: number
  title: string
  content: string
  created_at: string
}

const posts = ref<BlogPost[]>([])
const postMessage = ref('')

onMounted(() => {
  // Fetch dashboard data
  fetch('/admin/dashboard', {
    credentials: 'include',
  })
    .then(async (res) => {
      if (res.ok) {
        const json = await res.json()
        data.value = json.secret
      } else {
        const text = await res.text()
        error.value = text
      }
    })
    .catch((err) => (error.value = 'Error fetching admin data: ' + err))

  // Fetch admins list
  fetch('/admin/users', {
    credentials: 'include',
  })
    .then((res) => res.json())
    .then((data) => (admins.value = data))
    .catch((err) => console.error('Error fetching admins:', err))

  // Fetch posts list
  fetchAdminPosts()
})

const fetchAdminPosts = () => {
    fetch('/api/posts', {
    credentials: 'include',
  })
    .then((res) => res.json())
    .then((data) => (posts.value = data))
    .catch((err) => console.error('Error fetching posts:', err))
}

const handleSubmit = async () => {
  message.value = 'Submitting...'

  const url = editingId.value ? `/admin/posts/${editingId.value}` : '/admin/posts'
  const method = editingId.value ? 'PUT' : 'POST'

  try {
    const res = await fetch(url, {
      method: method,
      headers: {
        'Content-Type': 'application/json',
      },
      credentials: 'include',
      body: JSON.stringify({ title: title.value, content: content.value }),
    })

    if (res.ok) {
      message.value = editingId.value ? 'Post updated successfully!' : 'Post created successfully!'
      cancelEdit()
      fetchAdminPosts() // Refresh list
    } else {
      const text = await res.text()
      message.value = 'Error: ' + text
    }
  } catch (err) {
    message.value = 'Error submitting post: ' + err
  }
}

const handleEditPost = (post: BlogPost) => {
  editingId.value = post.id
  title.value = post.title
  content.value = post.content
  message.value = ''
  // Scroll to form
  window.scrollTo({ top: 0, behavior: 'smooth' })
}

const cancelEdit = () => {
  editingId.value = null
  title.value = ''
  content.value = ''
  message.value = ''
}

const handleAddAdmin = async () => {
  adminMessage.value = 'Adding...'

  try {
    const res = await fetch('/admin/users', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      credentials: 'include',
      body: JSON.stringify({ email: newAdminEmail.value }),
    })

    if (res.ok) {
      adminMessage.value = 'Admin added successfully!'
      newAdminEmail.value = ''
      // Refresh list
      fetch('/admin/users', { credentials: 'include' })
        .then((res) => res.json())
        .then((data) => (admins.value = data))
    } else {
      const text = await res.text()
      adminMessage.value = 'Error: ' + text
    }
  } catch (err) {
    adminMessage.value = 'Error adding admin: ' + err
  }
}

const handleDeletePost = async (id: number) => {
  if (!confirm(`Are you sure you want to delete post #${id}?`)) return
  postMessage.value = `Deleting post #${id}...`

  try {
    const res = await fetch(`/admin/posts/${id}`, {
      method: 'DELETE',
      credentials: 'include',
    })

    if (res.ok) {
      postMessage.value = 'Post deleted successfully!'
      fetchAdminPosts()
      if (editingId.value === id) {
        cancelEdit()
      }
    } else {
      const text = await res.text()
      postMessage.value = 'Error: ' + text
    }
  } catch (err) {
    postMessage.value = 'Error deleting post: ' + err
  }
}

const handleDeleteAdmin = async (email: string) => {
  if (!confirm(`Are you sure you want to remove ${email}?`)) return
  adminMessage.value = `Deleting ${email}...`

  try {
    const res = await fetch(`/admin/users/${email}`, {
      method: 'DELETE',
      credentials: 'include',
    })

    if (res.ok) {
      adminMessage.value = 'Admin removed successfully!'
      // Refresh list
      fetch('/admin/users', { credentials: 'include' })
        .then((res) => res.json())
        .then((data) => (admins.value = data))
    } else {
      const text = await res.text()
      adminMessage.value = 'Error: ' + text
    }
  } catch (err) {
    adminMessage.value = 'Error deleting admin: ' + err
  }
}
</script>
