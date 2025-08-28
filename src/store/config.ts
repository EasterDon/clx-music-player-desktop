import { ref } from "vue";
import { defineStore } from "pinia";

export enum PlayMode {
  listen,
  script,
}

export const use_config_store = defineStore("config", () => {
  const play_mode = ref<PlayMode>(PlayMode.listen);

  const app_info = ref({
    id: 1,
    announcement: "",
    app_version: "",
    app_download_link: "",
    app_version_description: "",
  });

  return {
    play_mode,
    app_info
  };
});
