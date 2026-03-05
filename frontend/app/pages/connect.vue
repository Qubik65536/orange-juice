<script setup lang="ts">
const config = useRuntimeConfig()

const { data, error, status } = await useFetch('/vue-connect', {
  baseURL: config.public.apiBase,
  server: false
})
</script>

<template>
  <UPageHero title="Backend Connection Test">
    <template #description>
      <div v-if="status === 'pending'">
        Loading...
      </div>
      <div v-else-if="error">
        <span class="text-red-500">Error: {{ error.message }}</span>
      </div>
      <div v-else>
        <span class="text-green-500">{{ data }}</span>
      </div>
    </template>
  </UPageHero>
</template>
