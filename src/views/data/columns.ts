import type { DataTableColumns } from 'naive-ui'
import { NTooltip } from 'naive-ui'
import { t } from '@/locales'

export const getFileColumns = (
  previewFn: (row: FileInfo) => void,
): DataTableColumns<FileInfo> => {
  return [
    {
      type: 'selection' as const,
      fixed: 'left' as const,
    },
    {
      title: 'ID',
      key: 'id',
      width: 70,
      fixed: 'left' as const,
      sorter: true,
    },
    {
      title: () => t('common.name'),
      key: 'name',
      width: 150,
      fixed: 'left' as const,
      sorter: true,
      render(row: FileInfo) {
        return h(NTooltip, { showDelay: 0, placement: 'top' }, {
          trigger: () => h(
            'span',
            {
              class: 'text-link truncate block',
              onClick: () => previewFn(row),
            },
            { default: () => row.name },
          ),
          default: () => t('common.view'),
        })
      },
    },
    {
      title: () => t('common.category'),
      key: 'category',
      width: 100,
      sorter: true,
      render(row: FileInfo) {
        let category = ''
        switch (row.category) {
          case 1:
            category = t('common.document')
            break
          case 2:
            category = t('common.image')
            break
          case 3:
            category = t('common.audio')
            break
          case 4:
            category = t('common.video')
            break
          case 5:
            category = t('common.other')
            break
          default:
        }
        return category
      },
    },
    {
      title: () => t('common.path'),
      key: 'path',
      sorter: true,
    },
    {
      title: () => t('common.extension'),
      key: 'file_ext',
      width: 100,
      sorter: true,
    },
    {
      title: () => t('common.fileSize'),
      key: 'file_size',
      width: 100,
      sorter: true,
      render(row: FileInfo) {
        if (row.file_size > 1024 * 1024 * 1024)
          return `${Math.floor(row.file_size / (1024 * 1024 * 1024))}G`
        else if (row.file_size > 1024 * 1024)
          return `${Math.floor(row.file_size / (1024 * 1024))}M`
        else if (row.file_size > 1024)
          return `${Math.floor(row.file_size / (1024 * 1024))}K`
        else
          return `${row.file_size}B`
      },
    },
    {
      title: () => t('common.contentIndexStatus'),
      key: 'content_index_status_msg',
      width: 130,
      sorter: true,
    },
    {
      title: () => t('common.metadataIndexStatus'),
      key: 'meta_index_status_msg',
      width: 150,
      sorter: true,
    },
    {
      title: () => t('common.fileCreateTime'),
      key: 'file_create_time',
      width: 130,
      sorter: true,
    },
    {
      title: () => t('common.fileUpdateTime'),
      key: 'file_update_time',
      width: 130,
      sorter: true,
    },
  ]
}
