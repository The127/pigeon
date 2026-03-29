<script setup lang="ts">
import { computed } from 'vue'
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

const ALL = '__all__'

const internalValue = computed({
  get: () => model.value || ALL,
  set: (v: string) => { model.value = v === ALL ? '' : v },
})
</script>

<template>
  <Select v-model="internalValue">
    <SelectTrigger class="w-auto min-w-[160px]">
      <SelectValue :placeholder="placeholder" />
    </SelectTrigger>
    <SelectContent>
      <SelectItem :value="ALL">{{ placeholder }}</SelectItem>
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
