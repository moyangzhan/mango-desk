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

interface SelfHostedPlatform {
  id: number;
  name: string;
  title: string;
  host: string;
  port: number;
  remark: string;
}

interface AiModel {
  id: number;
  name: string;
  title: string;
  platform: string;
  model_types: string;
  remark: string;
  setting: string;
  context_window: number;
  max_input_tokens: number;
  max_output_tokens: number;
  input_types: string;
  properties: string;
  is_reasoner: boolean;
  is_thinking_closable: boolean;
  is_free: boolean;
  is_enable: boolean;
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
  is_private: boolean; // Deprecated: kept for backward compatibility
  parser_mode: string; // "local" | "selfhosted" | "remote" | "mixed"
  image_parser_mode: string;
  audio_parser_mode: string;
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
  hit_types: string[],
  file_info: FileInfo,
  matched_keywords: string[],
  matched_chunk_ids: number[],
  similarity_type?: 'imageHash' | 'imageSemantic' | 'documentSemantic' | 'audioFingerprint' | 'audioTranscription',
}

//  Start { task_id: i64 },
//  Scan { task_id: i64, msg: String },
//  Stop { task_id: i64, msg: String },
//  Embed { task_id: i64, msg: String },
//  Finish { task_id: i64, msg: String },
interface IndexingEvent {
  event: string
  data: {
    taskId: number
    msg: string
  }
}