import { invoke } from "@tauri-apps/api/core";

export const delay = async (delay_time: number): Promise<boolean> => {
  return new Promise((resolve) => {
    setTimeout(() => resolve(true), delay_time);
  });
};

export const click = async (music_chars: string) => {
  const char_array = Array.from(music_chars);
  await Promise.all(char_array.map((key) => invoke("click", { key })));
};
