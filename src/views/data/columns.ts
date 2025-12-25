import { NButton, NTooltip } from 'naive-ui'
import type { DataTableBaseColumn, DataTableColumns } from 'naive-ui'
import type { VNode, VNodeChild } from 'vue'
import { t } from '@/locales'

function renderTooltip(trigger: VNode, content: string): VNodeChild {
  return h(NTooltip, null, {
    trigger: () => trigger,
    default: () => content,
  })
}

export const getFileColumns = (openFn: (path: string) => void, deleteFn: (id: number) => void): DataTableColumns<FileInfo> => {
  return [
    {
      title: 'ID',
      key: 'id',
      width: 70,
      fixed: 'left' as const,
    },
    {
      title: () => t('common.name'),
      key: 'name',
      width: 150,
      fixed: 'left' as const,
    },
    {
      title: () => t('common.category'),
      key: 'category',
      width: 100,
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
      // render(row: FileInfo) {
      //   return h(
      //     NDropdown,
      //     {
      //       text: true,
      //       size: 'small',
      //       placement: "bottom-start",
      //       trigger: 'hover',
      //       options: options,
      //       onSelect: handleSelect,
      //     },
      //     {
      //       default: () => [
      //         h(
      //           'div',
      //           {
      //           },
      //           { default: () => row.path },
      //         ),
      //       ]
      //     },
      //   )
      // }
    },
    {
      title: () => t('common.extension'),
      key: 'file_ext',
      width: 100,
    },
    {
      title: () => t('common.fileSize'),
      key: 'file_size',
      width: 100,
      render(row: FileInfo) {
        if (row.file_size > 1024 * 1024 * 1024)
          return `${Math.floor(row.file_size / (1024 * 1024 * 1024))}G`
        else if (row.file_size > 1024 * 1024)
          return `${Math.floor(row.file_size / (1024 * 1024))}M`
        else if (row.file_size > 1024)
          return `${Math.floor(row.file_size / 1024)}K`
        else
          return `${row.file_size}B`
      },
    },
    {
      title: () => t('common.contentIndexStatus'),
      key: 'content_index_status_msg',
      width: 120,
    },
    {
      title: () => t('common.metadataIndexStatus'),
      key: 'meta_index_status_msg',
      width: 130,
    },
    {
      title: () => t('common.fileCreateTime'),
      key: 'file_create_time',
      width: 120,
    },
    {
      title: () => t('common.fileUpdateTime'),
      key: 'file_update_time',
      width: 120,
    },
    {
      title: (_column: DataTableBaseColumn<FileInfo>) => {
        return renderTooltip(
          h(
            'span',
            {
            },
            { default: () => `${t('common.action')}❔` },
          ),
          t('common.removeTip'),
        )
      },
      key: 'actions',
      width: 100,
      render(row: FileInfo) {
        return h(
          'div',
          { class: 'flex flex-col' },
          {
            default: () => [
              h(
                NButton,
                {
                  ghost: true,
                  size: 'tiny',
                  style: 'margin-bottom: 8px',
                  onClick: () => openFn(row.path),
                },
                { default: () => t('common.open') },
              ),
              h(
                NButton,
                {
                  ghost: true,
                  size: 'tiny',
                  type: 'error',
                  onClick: () => deleteFn(row.id),
                },
                { default: () => t('common.remove') },
              ),
            ],
          },
        )
      },
    },
  ]
}
