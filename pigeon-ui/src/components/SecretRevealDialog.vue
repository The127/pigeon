<script setup lang="ts">
import { ref } from 'vue'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { AlertTriangle, Copy, Check } from 'lucide-vue-next'

const props = defineProps<{
  open: boolean
  secret: string
}>()

const emit = defineEmits<{
  confirmed: []
}>()

const copied = ref(false)

async function copySecret() {
  await navigator.clipboard.writeText(props.secret)
  copied.value = true
  setTimeout(() => { copied.value = false }, 2000)
}
</script>

<template>
  <Dialog :open="open">
    <DialogContent
      :show-close-button="false"
      @pointer-down-outside.prevent
      @escape-key-down.prevent
    >
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <AlertTriangle class="h-5 w-5 text-amber-500" />
          Save your signing secret
        </DialogTitle>
        <DialogDescription>
          This is your signing secret. Copy it now — you won't be able to see it again.
          Use it to verify webhook signatures on your endpoint.
        </DialogDescription>
      </DialogHeader>

      <div class="py-4">
        <div class="flex gap-2">
          <Input
            :model-value="secret"
            readonly
            class="font-mono text-sm"
          />
          <Button variant="outline" size="icon" @click="copySecret">
            <Check v-if="copied" class="h-4 w-4 text-green-500" />
            <Copy v-else class="h-4 w-4" />
          </Button>
        </div>
      </div>

      <DialogFooter>
        <Button @click="emit('confirmed')" :disabled="!copied">
          I've copied the secret
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
