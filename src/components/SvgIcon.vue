<script setup lang="ts">
import { onMounted, shallowRef } from 'vue'

const props = defineProps({
  name: { type: String, required: true }, // Svg name without .svg
  dir: { type: String, default: 'assets' },
})

const dynamicIcon = shallowRef<any>(null)

onMounted(async () => {
  try {
    const module = await import(`../${props.dir}/icons/${props.name}.svg?component`)
    dynamicIcon.value = module.default
  } catch (error) {
    console.error(`SVG load error for name: ${props.name}`, error)
  }
})
</script>

<template>
  <component :is="dynamicIcon" v-bind="$attrs" />
</template>
