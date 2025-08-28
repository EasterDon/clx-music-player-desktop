import { fetch } from "@tauri-apps/plugin-http";
import { base_url, songs_url } from "@/config";

export const get_songs_list = async () => {
  const response = await fetch(songs_url);
  const songs_list: MusicList = await response.json();
  return songs_list;
};

export const get_song_notation = async (music_id: number) => {
  const response = await fetch(`${songs_url}/${music_id}/notation`);
  const notation: MusicChars = await response.json();
  return notation;
};

export const get_song_lyrics = async (music_id: number) => {
  const response = await fetch(`${songs_url}/${music_id}/lyrics`);
  const lyrics: string = await response.text();
  return lyrics;
};

export const get_app_info = async (): Promise<{
  id: number;
  announcement: string;
  app_version: string;
  app_download_link: string;
  app_version_description: string;
}> => {
  const response = await fetch(`${base_url}/app`);
  const app_info = await response.json();
  return app_info;
};
