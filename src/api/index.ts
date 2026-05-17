import { fetch } from '@tauri-apps/plugin-http';
import { app_url, music_url } from '@/config';

/**
 * 统一解析 JSON，并在 HTTP 非成功时抛出可读错误，避免把 HTML 错页当 JSON 解析。
 */
const fetch_json = async <T>(url: string): Promise<T> => {
  const response = await fetch(url);
  const text = await response.text();
  if (!response.ok) {
    throw new Error(
      `请求失败 ${response.status} ${response.statusText}：${text.slice(0, 160)}`,
    );
  }
  try {
    return JSON.parse(text) as T;
  } catch {
    throw new Error('响应不是合法 JSON');
  }
};

export const get_music_list = async (): Promise<MusicList> => {
  const res = await fetch_json<Api_Envelope<MusicList>>(music_url);
  return res.data;
};

export const get_music_notation = async (
  music_id: number,
): Promise<MusicChars> => {
  const res = await fetch_json<Api_Envelope<MusicChars>>(
    `${music_url}/${music_id}/notation`,
  );
  return res.data;
};

export const get_music_lyrics = async (music_id: number): Promise<string> => {
  const res = await fetch_json<Api_Envelope<string>>(
    `${music_url}/${music_id}/lyrics`,
  );
  return res.data;
};

export const get_app_info = async (): Promise<App_Info> => {
  const res = await fetch_json<Api_Envelope<App_Info>>(`${app_url}`);
  return res.data;
};
