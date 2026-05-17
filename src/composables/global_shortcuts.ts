import { register, unregister } from '@tauri-apps/plugin-global-shortcut';
import { get_music_notation } from '@/api';
import { PlayMode, use_config_store } from '@/store/config';
import { use_music_store } from '@/store/player';

/**
 * F1：演奏模式下拉谱并执行脚本；聆听模式则提示用户去设置切换。
 * F2：停止脚本。
 */
export const use_global_shortcuts = (open_listen_mode_modal: () => void) => {
  const music_store = use_music_store();
  const config_store = use_config_store();

  const register_shortcuts = async () => {
    await unregister(['F1', 'F2']);

    await register('F1', async (e) => {
      if (!music_store.current_music) return void 0;
      if (e.state !== 'Pressed') return void 0;
      if (config_store.play_mode !== PlayMode.script) {
        open_listen_mode_modal();
        return void 0;
      }
      const music_notation = await get_music_notation(
        music_store.current_music.id,
      );
      music_store.play(music_notation);
    });

    await register('F2', (e) => {
      if (!music_store.current_music) return void 0;
      if (e.state !== 'Pressed') return void 0;
      music_store.stop();
    });
  };

  return { register_shortcuts };
};
