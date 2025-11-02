import { fetch } from '@tauri-apps/plugin-http';
import { app_url, music_url } from '@/config';

export const get_songs_list = async () => {
  const response = await fetch(music_url);
  const res: { data: MusicList } = await response.json();
  return res.data;
};

export const get_song_notation = async (music_id: number) => {
  const response = await fetch(`${music_url}/${music_id}/notation`);
  const res: { data: MusicChars } = await response.json();
  return res.data;
};

export const get_song_lyrics = async (music_id: number) => {
  const response = await fetch(`${music_url}/${music_id}/lyrics`);
  const res = await response.json();
  console.log(res);
  return res.data;
};

export const get_app_info = async (): Promise<{
  id: number;
  announcement: string;
  app_version: string;
  app_download_link: string;
  app_version_description: string;
}> => {
  const response = await fetch(`${app_url}`);
  const res = await response.json();
  return res.data;
};

class Api {
  static #base_url = 'http:127.0.0.1';
  static #version = 'v2';

  static get_lyrics_url(id: number) {
    return `${this.#base_url}/${this.#version}/${id}/lyrics`;
  }
}
