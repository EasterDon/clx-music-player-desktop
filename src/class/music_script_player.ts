import { reactive } from 'vue';
import { register, unregisterAll } from '@tauri-apps/plugin-global-shortcut';
import { message } from 'ant-design-vue';
import { get_songs_list, get_song_notation } from '@/api';
import { delay, click } from '@/util';

export class MusicScriptPlayer {
  private state = reactive<MusicScriptPlayerState>({
    songs_list: [],
    current_song_value: null,
    continue_play: false,
  });

  get songs_list() {
    return this.state.songs_list;
  }

  get current_song_value() {
    return this.state.current_song_value;
  }
  set current_song_value(song_value: MusicListItem) {
    this.state.current_song_value = song_value;
  }

  set_current_music(id: number) {
    const result = this.state.songs_list.find((item) => item.id === id);
    this.state.current_song_value = result ? result : null;
  }

  async init_songs_list() {
    this.state.songs_list = await get_songs_list();
  }

  async play() {
    if (!this.state.current_song_value) {
      message.warn('请先选择一首歌曲哦');
      return;
    }
    if (this.state.continue_play) {
      return;
    }
    const song_notation = await get_song_notation(
      this.state.current_song_value.id,
    );
    this.state.continue_play = true;
    for (const item of song_notation) {
      if (!this.state.continue_play) {
        break;
      }
      if (typeof item === 'number') {
        await delay(item);
      }
      if (typeof item === 'string') {
        await click(item);
      }
    }
  }

  stop() {
    this.state.continue_play = false;
  }

  async set_hotkey() {
    await unregisterAll();
    await register('F1', () => {
      this.play();
    });
    await register('F2', () => {
      this.stop();
    });
  }
}
