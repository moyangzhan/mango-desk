interface Window {
  $loadingBar: import('naive-ui').LoadingBarProviderInst;
  $dialog: import('naive-ui').DialogProviderInst;
  $message: import('naive-ui').MessageProviderInst;
  $notification: import('naive-ui').NotificationProviderInst;
}

interface AppState {
  theme: string;
  locale: string;
}

interface SettingState {
  activeTab: string;
}

interface ModelPlatform {
  id: number;
  name: string;
  title: string;
  base_url: string;
  api_key: string;
  remark: string;
  is_proxy_enable: boolean;
}

interface AiModel {
  name: string;
  title: string;
  platform: string;
  model_types: string; // text;image;vision;audio;video
  remark: string;
}

interface ProxyInfo {
  protocal: string;
  host: string;
  port: number;
}

interface SaveParsedContent {
  document: boolean;
  image: boolean;
  audio: boolean;
  video: boolean;
}

interface IndexerSetting {
  is_private: boolean;
  file_content_language: string;
  ignore_dirs: string[];
  ignore_exts: string[];
  ignore_files: string[];
  save_parsed_content: SaveParsedContent;
}

interface DownloadState {
  downloadId: string;
  url: string;
  progress: number;
  status: string;
}

interface SelectedItem {
  id: string;
  name: string;
  type: 'file' | 'directory';
  raw: File | null;
  path?: string;
  done: boolean; // for indexing task
}

interface IndexingTask {
  id: number;
  paths: string;
  total_cnt: number;
  content_processed_cnt: number;
  content_indexed_success_cnt: number;
  content_indexed_failed_cnt: number;
  content_indexed_skipped_cnt: number;
  duration: number;
}

interface FileMetadata {
  name: string;
  extension: string;
  category: string;
  size: number; //in bytes
  created: string;
  modified: string;
  author: string;
}

interface FileInfo {
  id: number;
  name: string;
  category: number; //1:document, 2:image, 3:audio, 4:video, 5:other
  path: string;
  content: string;
  metadata: FileMetadata;
  file_ext: string;
  file_size: number;
  md5: string;
  content_index_status: number;
  content_index_status_msg: string;
  meta_index_status: number;
  meta_index_status_msg: string;
  is_invalid: boolean;
  invalid_reason: string;
  file_create_time: string;
  file_update_time: string;
  create_time: string;
  update_time: string;

  // For UI
  file_data?: any // raw file data
  [key: string]: any;
  html_path: string;
}

interface CommandResult {
  success: boolean;
  message: string;
  data?: any;
  code: number;
}

interface WatchSetting {
  directories: string[];
  files: string[];
}

interface SearchResult {
  score: number,
  source: string,
  file_info: FileInfo,
  matched_keywords: string[],
  matched_chunk_ids: number[],
}