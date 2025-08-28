import { ref } from "vue";
import { defineStore } from "pinia";

export const songs = defineStore("songs", () => {
  const current_song = ref<MusicListItem | null>(null);

  return {
    current_song,
  };
});
