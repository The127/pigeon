<script setup lang="ts">
import { useRoute } from 'vue-router'
import AppLayout from '@/components/AppLayout.vue'
import ToastContainer from '@/components/ToastContainer.vue'

const route = useRoute()
</script>

<template>
  <ToastContainer>
    <AppLayout v-if="!route.meta.public">
      <RouterView />
    </AppLayout>
    <RouterView v-else />
  </ToastContainer>
</template>
