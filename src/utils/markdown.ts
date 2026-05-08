import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { marked } from 'marked'

let storagePathCache: string | null = null

async function getStoragePath(): Promise<string> {
  if (storagePathCache)
    return storagePathCache
  const dataPath = await invoke<string>('get_data_path')
  storagePathCache = `${dataPath}/storage`
  return storagePathCache
}

function fixImagePaths(markdown: string, storagePath: string): string {
  // Replace relative image paths like ../extracted_images/ or ../../extracted_images/
  return markdown.replace(
    /(!\[[^\]]*\])\((\.\.\/)+(extracted_images\/[^)]+)\)/g,
    (_match, prefix: string, _dots: string, relPath: string) => {
      const absPath = `${storagePath}/${relPath}`
      return `${prefix}(${convertFileSrc(absPath)})`
    },
  )
}

export async function renderMarkdown(content: string): Promise<string> {
  if (!content)
    return ''
  const storagePath = await getStoragePath()
  const fixed = fixImagePaths(content, storagePath)
  return await marked.parse(fixed) as string
}
