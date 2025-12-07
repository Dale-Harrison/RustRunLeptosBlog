<template>
  <div class="min-h-screen flex flex-col font-sans">
    <header class="w-full flex justify-between items-center p-4 gap-4">
      <router-link to="/" class="text-xl font-bold hover:text-blue-600 transition">
        &larr; Back to Home
      </router-link>
    </header>

    <div class="flex-1 grid grid-rows-[5px_1fr_5px] items-start justify-items-center p-8 pb-20 gap-16 sm:p-20">
      <main class="flex flex-col gap-8 row-start-2 items-center sm:items-start w-full max-w-2xl">
        <div v-if="loading" class="text-center w-full">
          <p class="text-xl">Loading post...</p>
        </div>
        
        <div v-else-if="error" class="text-center w-full text-red-500">
          <p class="text-xl">{{ error }}</p>
        </div>

        <article v-else-if="post" class="w-full bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8 border border-gray-200 dark:border-gray-700">
          <h1 class="text-4xl font-bold mb-4">{{ post.title }}</h1>
          <div class="text-sm text-gray-400 mb-8 pb-4 border-b border-gray-200 dark:border-gray-700">
            Published on {{ new Date(post.created_at).toLocaleDateString() }}
          </div>
          <div class="prose dark:prose-invert max-w-none whitespace-pre-wrap">
            {{ post.content }}
          </div>
        </article>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'

interface BlogPost {
  id: number
  title: string
  content: string
  created_at: string
}

const route = useRoute()
const post = ref<BlogPost | null>(null)
const loading = ref(true)
const error = ref('')

onMounted(() => {
  const postId = route.params.id
  fetch(`/api/posts/${postId}`)
    .then((res) => {
      if (!res.ok) {
        throw new Error('Post not found')
      }
      return res.json()
    })
    .then((data) => {
      post.value = data
      loading.value = false
    })
    .catch((err) => {
      console.error('Error fetching post:', err)
      error.value = err.message
      loading.value = false
    })
})
</script>
