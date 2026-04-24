export function emptyModelPlatform() {
  return {
    id: 0,
    name: '',
    title: '',
    base_url: '',
    api_key: '',
    remark: '',
    is_proxy_enable: false,
  }
}

export function emptyProxyInfo() {
  return {
    protocol: 'http',
    host: '',
    port: 0,
  }
}

export function emptyIndexerSetting(): IndexerSetting {
  return {
    is_private: true,
    parser_mode: 'local',
    image_parser_mode: 'local',
    audio_parser_mode: 'local',
    file_content_language: 'en',
    ignore_dirs: [] as string[],
    ignore_exts: [] as string[],
    ignore_files: [] as string[],
    ignore_path_prefixes: [] as string[],
    save_parsed_content: {
      document: false,
      image: true,
      video: true,
      audio: true,
    },
  }
}

export function emptyDownloadState() {
  return {
    downloadId: '',
    url: '',
    progress: 0,
    status: 'success',
  }
}

export function emptyWatchSetting() {
  return {
    directories: [] as string[],
    files: [] as string[],
  }
}
