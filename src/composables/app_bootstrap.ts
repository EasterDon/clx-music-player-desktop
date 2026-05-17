import { onMounted } from "vue";

/**
 * 应用启动：先注册全局快捷键，再拉曲库与软件信息。
 */
export const use_app_bootstrap = (
  register_shortcuts: () => Promise<void>,
  load_music_list: () => Promise<void>,
  load_app_info: () => Promise<void>,
) => {
  onMounted(async () => {
    await register_shortcuts();
    await load_music_list();
    await load_app_info();
  });
};
