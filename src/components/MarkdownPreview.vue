<script setup lang="ts">
import { renderMarkdown } from '@/utils/markdown'

const props = defineProps<{
  content: string
}>()

const rendered = ref('')

watch(() => props.content, async (val) => {
  rendered.value = val ? await renderMarkdown(val) : ''
}, { immediate: true })
</script>

<template>
  <div v-if="rendered" class="markdown-body" v-html="rendered" />
  <div v-else>
    <slot name="empty">
      {{ $t('common.noData') }}
    </slot>
  </div>
</template>

<style scoped>
.markdown-body {
  font-size: 14px;
  line-height: 1.6;
  word-wrap: break-word;
}

.markdown-body h1,
.markdown-body h2,
.markdown-body h3,
.markdown-body h4 {
  margin-top: 1em;
  margin-bottom: 0.5em;
  font-weight: 600;
}

.markdown-body h1 { font-size: 1.5em; }
.markdown-body h2 { font-size: 1.3em; }
.markdown-body h3 { font-size: 1.15em; }

.markdown-body p {
  margin: 0.5em 0;
}

.markdown-body img {
  max-width: 100%;
  height: auto;
  border-radius: 4px;
  margin: 0.5em 0;
}

.markdown-body hr {
  border: none;
  border-top: 1px solid var(--border-color);
  margin: 1em 0;
}

.markdown-body table {
  border-collapse: collapse;
  width: 100%;
  margin: 0.5em 0;
}

.markdown-body th,
.markdown-body td {
  border: 1px solid var(--border-color);
  padding: 6px 12px;
  text-align: left;
}

.markdown-body code {
  background: rgba(127, 127, 127, 0.1);
  padding: 0.2em 0.4em;
  border-radius: 3px;
  font-size: 0.9em;
}

.markdown-body pre {
  background: rgba(127, 127, 127, 0.1);
  padding: 1em;
  border-radius: 4px;
  overflow-x: auto;
}

.markdown-body pre code {
  background: none;
  padding: 0;
}
</style>
