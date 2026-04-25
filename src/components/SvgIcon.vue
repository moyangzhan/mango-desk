<script setup lang="ts">
import { shallowRef, watch } from 'vue'

const props = defineProps({
  name: { type: String, required: true },
  dir: { type: String, default: 'assets' },
})

const dynamicIcon = shallowRef<any>(null)

async function loadIcon(name: string) {
  try {
    const module = await import(`../${props.dir}/icons/${name}.svg?component`)
    dynamicIcon.value = module.default
  }
  catch (error) {
    console.error(`SVG load error for name: ${name}`, error)
  }
}

watch(() => props.name, loadIcon, { immediate: true })
</script>

<template>
  <component :is="dynamicIcon" v-bind="$attrs" />
</template>
