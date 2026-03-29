<script setup lang="ts">
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

const model = defineModel<string>({ default: '' })

defineProps<{
  placeholder: string
  options: { value: string; label: string }[]
}>()
</script>

<template>
  <Select v-model="model">
    <SelectTrigger class="w-auto min-w-[160px]">
      <SelectValue :placeholder="placeholder" />
    </SelectTrigger>
    <SelectContent>
      <SelectItem value="">{{ placeholder }}</SelectItem>
      <SelectItem
        v-for="opt in options"
        :key="opt.value"
        :value="opt.value"
      >
        {{ opt.label }}
      </SelectItem>
    </SelectContent>
  </Select>
</template>
