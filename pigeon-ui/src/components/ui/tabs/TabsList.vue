<script setup lang="ts">
import type { TabsListProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { reactiveOmit } from "@vueuse/core"
import { TabsList } from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps<TabsListProps & { class?: HTMLAttributes["class"] }>()

const delegatedProps = reactiveOmit(props, "class")
</script>

<template>
  <TabsList
    data-slot="tabs-list"
    v-bind="delegatedProps"
    :class="cn(
      'tabs-list-separated bg-muted text-muted-foreground inline-flex h-9 w-fit items-center justify-center rounded-lg p-[3px]',
      props.class,
    )"
  >
    <slot />
  </TabsList>
</template>

<style scoped>
.tabs-list-separated :deep([data-slot="tabs-trigger"]:not(:last-child))::after {
  content: '';
  position: absolute;
  right: -1px;
  top: 20%;
  height: 60%;
  width: 1px;
  border-radius: 1px;
  background-color: var(--color-border);
}

.tabs-list-separated :deep([data-slot="tabs-trigger"]) {
  position: relative;
}

/* Hide separator next to active tab */
.tabs-list-separated :deep([data-slot="tabs-trigger"][data-state="active"])::after,
.tabs-list-separated :deep([data-slot="tabs-trigger"][data-state="active"] + [data-slot="tabs-trigger"])::before {
  display: none;
}

/* Also hide separator before active tab */
.tabs-list-separated :deep([data-slot="tabs-trigger"]:has(+ [data-state="active"]))::after {
  display: none;
}
</style>
