<template>
  <div class="min-h-screen flex flex-col font-sans">
    <header class="w-full flex justify-end items-center p-4 gap-4">
      <template v-if="user">
        <span class="text-lg font-medium">Hello, {{ user.name }}!</span>
        <button
          @click="handleLogout"
          class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 transition"
        >
          Logout
        </button>
        <router-link
          to="/admin"
          class="px-4 py-2 bg-gray-800 text-white rounded hover:bg-gray-900 transition"
        >
          Admin Dashboard
        </router-link>
      </template>
      <template v-else>
        <button
          @click="handleLogin"
          class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition font-semibold"
        >
          Login with Google
        </button>
      </template>
    </header>

    <div class="flex-1 grid grid-rows-[5px_1fr_5px] items-center justify-items-center p-8 pb-20 gap-16 sm:p-20">
      <main class="flex flex-col gap-8 row-start-2 items-center sm:items-start w-full max-w-2xl">
        <h1 class="text-4xl font-bold text-center sm:text-left">
          {{ message || "Loading..." }}
        </h1>

        <div class="w-full mt-8">
          <h2 class="text-2xl font-bold mb-4">Latest Posts</h2>
          <div class="flex flex-col gap-4">
            <template v-if="posts.length > 0">
              <div
                v-for="post in posts"
                :key="post.id"
                class="p-6 bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700"
              >
                <h3 class="text-xl font-bold mb-2">{{ post.title }}</h3>
                <p class="text-gray-600 dark:text-gray-300 whitespace-pre-wrap">{{ post.content }}</p>
                <p class="text-sm text-gray-400 mt-4">
                  {{ new Date(post.created_at).toLocaleDateString() }}
                </p>
              </div>
            </template>
            <p v-else class="text-gray-500 italic">No posts yet.</p>
          </div>
        </div>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface User {
  email: string
  name: string
}

interface BlogPost {
  id: number
  title: string
  content: string
  created_at: string
}

const message = ref<string>('')
const user = ref<User | null>(null)
const posts = ref<BlogPost[]>([])

onMounted(() => {
  // Fetch hello message
  fetch('/api/hello')
    .then((res) => res.json())
    .then((data) => (message.value = data.message))
    .catch((err) => console.error('Error fetching message:', err))

  // Fetch user info
  fetch('/auth/me', {
    credentials: 'include',
  })
    .then((res) => {
      if (res.ok) return res.json()
      return null
    })
    .then((data) => (user.value = data))
    .catch((err) => console.error('Error fetching user:', err))

  // Fetch blog posts
  fetch('/api/posts')
    .then((res) => res.json())
    .then((data) => (posts.value = data))
    .catch((err) => console.error('Error fetching posts:', err))
})

const handleLogin = () => {
  window.location.href = '/auth/login'
}

const handleLogout = () => {
  window.location.href = '/auth/logout'
}
</script>
