<script setup lang="ts">
/**
 * AppTag - 统一的标签组件
 * 默认属性：bordered=false, round, size=tiny
 */
withDefaults(defineProps<{
  type?: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'error'
  size?: 'tiny' | 'small' | 'medium' | 'large'
}>(), {
  type: 'default',
  size: 'tiny',
})
</script>

<template>
  <NTag :type="type" :size="size" :bordered="false" round>
    <slot />
  </NTag>
</template>
