import { fetch } from '@tauri-apps/plugin-http';
import { songs_url } from '@/config';

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
