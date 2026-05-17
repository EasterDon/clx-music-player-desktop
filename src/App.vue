<script setup lang="ts">
import { reactive, computed, useTemplateRef, ref } from 'vue';
import {
  message,
  Drawer,
  Badge,
  Input,
  Modal,
  Avatar,
  ConfigProvider,
} from 'ant-design-vue';
import zhCN from 'ant-design-vue/es/locale/zh_CN';
import { useBreakpoints } from '@vueuse/core';
import { MenuUnfoldOutlined } from '@ant-design/icons-vue';
import MusicList from '@/components/MusicList/index.vue';
import LyricsPage from '@/components/Lyrics/index.vue';
import SettingsDrawer from '@/components/SettingsDrawer/index.vue';
import PlayerControls from '@/components/player_controls/index.vue';
import { desktop_breakpoint, music_resource_url } from '@/config';
import { use_music_store } from '@/store/player';
import { get_app_info, get_music_lyrics, get_music_list } from '@/api';
import { use_config_store } from '@/store/config';
import { app_version } from '@/config';
import {
  use_filtered_music_list,
  use_global_shortcuts,
  use_selection_spin,
  use_app_bootstrap,
} from '@/composables';
console.log('服务器地址', import.meta.env.VITE_BASE_URL);

const music_store = use_music_store();
const config_store = use_config_store();

const { music_list_display, on_music_filter } = use_filtered_music_list();

const music_state = reactive({
  choose: false,
  lyrics: false,
  script: true,
});

const breakpoints = useBreakpoints({
  mobile: desktop_breakpoint,
});

const is_desktop = breakpoints.greater('mobile');

const spinning = computed(() => {
  if (!music_state.choose) return false;
  return !(music_state.lyrics && music_state.script);
});

use_selection_spin(spinning);

const modal_listen_mode = ref(false);
const { register_shortcuts } = use_global_shortcuts(() => {
  modal_listen_mode.value = true;
});

/**
 * 切换当前曲目：拉歌词并更新子组件；不自动播放，需用户点播放。
 * script 标记预留给后续谱面相关流程（与全局 loading 条件一致）。
 */
const set_current_music = async (music: MusicListItem) => {
  music_state.choose = true;
  music_state.lyrics = false;
  if (music_store.current_music && music_store.current_music.id === music.id) {
    return void 0;
  }
  // 切歌时只暂停，不重置歌词位移，避免上一首歌词先跳到顶部。
  audio_pause();
  music_store.current_music = music;
  try {
    const lyrics_string = await get_music_lyrics(music.id);
    set_lyrics(lyrics_string);
    music_state.lyrics = true;
  } catch (err) {
    music_state.lyrics = false;
    if (err instanceof Error) {
      message.error(err.message);
      return void 0;
    }
    message.error('请求似乎出错了哦');
  }
};

const init_music_list = async () => {
  const response = await get_music_list();
  music_store.set_music_list(response);
};

const init_app_info = async () => {
  const app_info = await get_app_info();
  Object.assign(config_store.app_info, app_info);
};

use_app_bootstrap(register_shortcuts, init_music_list, init_app_info);

const settings_modal = ref(false);
const open_settings_modal = () => (settings_modal.value = true);
const current_time = ref(0);
const duration = ref(0);
const mobile_show_lyrics = ref(false);

const cover_src = computed(() =>
  music_store.current_music
    ? `${music_resource_url}/${music_store.current_music.id}/music.jpg`
    : '',
);

const lyric_ref = useTemplateRef('lyric_ref');
const audio_play = () => lyric_ref.value?.audio_play();
const audio_pause = () => lyric_ref.value?.audio_pause();
const previous_track = () => lyric_ref.value?.previous_track();
const next_track = () => lyric_ref.value?.next_track();
const seek_progress = (value: number) =>
  lyric_ref.value?.seek_audio_progress(value);
const set_lyrics = (lyrics_string: string) =>
  lyric_ref.value?.set_lyrics(lyrics_string);
const on_progress_change = (payload: {
  current_time: number;
  duration: number;
}) => {
  current_time.value = payload.current_time;
  duration.value = payload.duration;
};
const open_mobile_lyrics = () => {
  mobile_show_lyrics.value = true;
};
const close_mobile_lyrics = () => {
  mobile_show_lyrics.value = false;
};
</script>

<template>
  <ConfigProvider :locale="zhCN">
    <Drawer
      v-model:open="settings_modal"
      placement="left"
      :closable="false"
      width="300"
    >
      <SettingsDrawer />
    </Drawer>

    <div
      class="main-layout"
      :class="{
        'mobile': !is_desktop,
        'desktop': is_desktop,
        'show-lyrics': mobile_show_lyrics,
      }"
    >
      <div
        class="left-bar list-face"
        :class="{ mobile: !is_desktop, desktop: is_desktop }"
      >
        <div class="settings-open">
          <Badge :dot="app_version !== config_store.app_info.app_version">
            <MenuUnfoldOutlined @click="open_settings_modal" class="icon" />
          </Badge>
          <Input
            class="music-filter"
            @change="on_music_filter"
            placeholder="F1开始脚本演奏，F2停止脚本"
            style="text-align: center"
          ></Input>
        </div>

        <div class="music-list-container">
          <music-list
            :music_list="music_list_display"
            @set_current_music="set_current_music"
          />
        </div>
        <PlayerControls
          v-if="!is_desktop"
          :is_playing="music_store.is_playing"
          :play_disabled="!music_store.current_music"
          :current_time="current_time"
          :duration="duration"
          :show_lyrics_toggle="true"
          @audio_play="audio_play"
          @audio_pause="audio_pause"
          @previous_track="previous_track"
          @next_track="next_track"
          @seek_progress="seek_progress"
          @toggle_lyrics_view="open_mobile_lyrics"
        />
      </div>

      <lyrics-page
        :as_pane="is_desktop"
        class="lyrics-face"
        ref="lyric_ref"
        @set_current_music="set_current_music"
        @progress_change="on_progress_change"
        @back_to_list="close_mobile_lyrics"
      />
    </div>

    <Modal
      v-model:open="modal_listen_mode"
      title="注意！"
      centered
      @ok="modal_listen_mode = false"
    >
      <p>当前处于聆听模式，如需脚本演奏请点击左上角设置修改</p>
    </Modal>
    <Modal
      v-model:open="music_store.continue_play"
      title="自动演奏中..."
      centered
      :footer="null"
      :closable="false"
      :keyboard="false"
      :mask-closable="false"
    >
      <div class="current-music">
        <Avatar :size="100" :src="cover_src" />
        <p style="text-align: center">
          当前演奏歌曲：{{
            music_store.current_music ? music_store.current_music.name : ''
          }}
        </p>
        <p style="text-align: center">
          作者:{{
            music_store.current_music ? music_store.current_music.author : ''
          }}
        </p>
      </div>
    </Modal>
  </ConfigProvider>
</template>

<style lang="less" scoped>
.audio {
  display: none;
}

.settings-open {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 20px;
}

.icon {
  font-size: 24px;
}

.main-layout {
  height: 100%;
  position: relative;

  &.desktop {
    display: flex;
    width: 100%;
    flex: 1;
    min-width: 0;
    align-items: stretch;
    gap: 12px;
    perspective: 1600px;
  }

  &.desktop .list-face {
    transform-origin: right center;
    transform-style: preserve-3d;
  }

  &.desktop .lyrics-face {
    transform-origin: left center;
    transform-style: preserve-3d;
    backface-visibility: hidden;
  }
}

.left-bar {
  height: 100%;
  display: flex;
  flex-flow: column nowrap;
  gap: 10px;

  &.desktop {
    flex: 1;
    min-width: 0;
  }

  &.mobile {
    position: relative;
    left: 0;
    z-index: 2;
  }
}

.music-filter {
  width: clamp(200px, 80%, 500px);
}

.music-list-container {
  width: 100%;
  box-sizing: border-box;
  flex: 1;
  overflow-x: hidden;
  overflow-y: scroll;
  scrollbar-width: thin;
}

.current-music {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: 1rem;
}

@media (max-width: @desktop-breakpoint) {
  .main-layout.mobile {
    perspective: 1200px;
    overflow: hidden;
    position: relative;
  }

  .list-face,
  .lyrics-face {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    backface-visibility: hidden;
    transform-style: preserve-3d;
    transition: transform 0.6s cubic-bezier(0.22, 1, 0.36, 1);
  }

  /* 书页铰链：列表页绕右边缘，歌词页绕左边缘 */
  .main-layout.mobile .list-face {
    transform-origin: right center;
  }

  .main-layout.mobile .lyrics-face {
    transform-origin: left center;
  }

  .main-layout.mobile .list-face {
    transform: rotateY(0deg);
    pointer-events: auto;
  }

  .main-layout.mobile .lyrics-face {
    transform: rotateY(-180deg);
    pointer-events: none;
  }

  .main-layout.mobile.show-lyrics .list-face {
    transform: rotateY(180deg);
    pointer-events: none;
  }

  .main-layout.mobile.show-lyrics .lyrics-face {
    transform: rotateY(0deg);
    pointer-events: auto;
  }
}
</style>
