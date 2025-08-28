<script setup lang="ts">
import {
  reactive,
  computed,
  onMounted,
  useTemplateRef,
  ref,
  nextTick,
  watch,
} from "vue";
import { message, Drawer, Button, Badge, Input } from "ant-design-vue";
import { useBreakpoints } from "@vueuse/core";
import {
  CaretRightOutlined,
  LeftOutlined,
  RightOutlined,
  PauseOutlined,
  MenuUnfoldOutlined,
} from "@ant-design/icons-vue";
import { spin } from "@/util/spin";
import { SpinInstance } from "@/util/types";
import MusicList from "@/components/MusicList.vue";
import SettingsDrawer from "@/components/SettingsDrawer.vue";
import { MusicScriptPlayer } from "@/class/music_script_player";
import { songs_resource_url } from "@/config";
import { get_app_info, get_song_lyrics } from "@/api";
import { register, unregisterAll } from "@tauri-apps/plugin-global-shortcut";
import { PlayMode, use_config_store } from "@/store/config";
import { app_version } from "@/config";
import { ChangeEvent } from "ant-design-vue/es/_util/EventInterface";

const music_player = new MusicScriptPlayer();
const config_store = use_config_store();

const song_state = reactive({
  choose: false,
  mp3: false,
  lyrics: false,
  script: true,
});

const breakpoints = useBreakpoints({
  mobile: 768, // ≤768px 视为移动端
});

const isDesktop = breakpoints.greater("mobile");

const state = reactive<{ music_lyrics: Lyrics; isPlaying: boolean }>({
  music_lyrics: [
    {
      time: 0,
      content: "暂未选择歌曲",
    },
  ],
  isPlaying: false,
});

let lyrics_length = 0;
const spinning = computed(() => {
  if (!song_state.choose) return false;
  return !(song_state.lyrics && song_state.mp3 && song_state.script);
});

let spinIntance: SpinInstance | null = null;
watch(
  () => spinning.value,
  (newVal, oldVal) => {
    if (newVal === oldVal) return;
    if (newVal) {
      spinIntance = spin();
    } else {
      spinIntance?.close();
    }
  }
);

const audio_canplay = async () => {
  song_state.mp3 = true;
};
const set_current_song = async (song: MusicListItem) => {
  song_state.choose = true;
  song_state.lyrics = false;
  song_state.mp3 = false;
  if (
    music_player.current_song_value &&
    music_player.current_song_value.id === song.id
  ) {
    return;
  }
  audio_end();
  state.music_lyrics = [];
  music_player.current_song_value = song;
  try {
    const lyrics_string = await get_song_lyrics(song.id);
    state.music_lyrics = trans_lyrics_string(lyrics_string);
    lyrics_length = state.music_lyrics.length;
    song_state.lyrics = true;
  } catch (err) {
    song_state.lyrics = false;
    song_state.mp3 = false;
    if (err instanceof Error) {
      message.error(err.message);
      return;
    }
    message.error("请求似乎出错了哦");
  }
};

// 音乐mp3源切换
const current_music_mp3_url = computed(() =>
  music_player.current_song_value
    ? `${songs_resource_url}/${music_player.current_song_value.id}/music.mp3`
    : ""
);

const set_hotkey = async () => {
  await unregisterAll();
  await register("F1", (e) => {
    if (!music_player.current_song_value) return void 0;
    if (e.state !== "Pressed") return void 0;
    if (config_store.play_mode !== PlayMode.script) {
      message.info("当前处于聆听模式，如需脚本演奏请点击左上角设置修改");
      return void 0;
    }
    music_player.play();
  });
  await register("F2", (e) => {
    if (!music_player.current_song_value) return void 0;
    if (e.state !== "Pressed") return void 0;
    music_player.stop();
  });
};
const init_app_info = async () => {
  const app_info = await get_app_info();
  Object.assign(config_store.app_info, app_info);
};
onMounted(async () => {
  await set_hotkey();
  await music_player.init_songs_list();
  await init_app_info();
  songs.value = [...music_player.songs_list];
});
const audio = useTemplateRef("audio");
const audio_play = () => {
  audio.value?.play();
  state.isPlaying = true;
};

const audio_pause = () => {
  state.isPlaying = false;
  audio.value?.pause();
};

const previous_track = async () => {
  audio_pause();
  await nextTick();
  audio_end();
  const length = music_player.songs_list.length;
  const current_song_id = music_player.current_song_value.id;
  let pre: null | MusicListItem = null;
  if (music_player.songs_list[0].id !== current_song_id) {
    const index = music_player.songs_list.findIndex(
      (song) => song.id === current_song_id
    );
    pre = music_player.songs_list[index - 1];
  } else {
    pre = music_player.songs_list[length - 1];
  }
  await set_current_song(pre);
  audio_play();
};

const next_track = async () => {
  audio_pause();
  audio_end();
  const length = music_player.songs_list.length;
  const current_song_id = music_player.current_song_value.id;
  let next: null | MusicListItem = null;
  if (music_player.songs_list[length - 1].id !== current_song_id) {
    const index = music_player.songs_list.findIndex(
      (song) => song.id === current_song_id
    );
    next = music_player.songs_list[index + 1];
  } else {
    next = music_player.songs_list[0];
  }
  await set_current_song(next);
  audio_play();
};

const audio_end = () => {
  if (state.isPlaying) state.isPlaying = false;
  lastIndex = 0;
  currentIndex.value = 0;
  lyricsRef.value!.style.marginTop = "0px";
};

// @future 分割歌词滚动页
const timeTagRegex = /\[(\d+):(\d+)(?:\.(\d+))?\]/g;
const trans_lyrics_string = (lyrics_string: string) => {
  const string_array = lyrics_string.split("\n");
  let lyrics: Array<{ time: number; content: string }> = [];
  string_array.forEach((line) => {
    let match;
    const content = line.replace(timeTagRegex, "").trim();

    // 提取所有时间标签
    while ((match = timeTagRegex.exec(line)) !== null) {
      const mm = parseInt(match[1]);
      const ss = parseInt(match[2]);
      const xx = parseInt(match[3]) | 0;
      const time = mm * 60 + ss + xx / 100;

      lyrics.push({ time, content });
    }
  });
  return lyrics;
};

// 歌词滚动
const lyricsRef = useTemplateRef("lyricsRef");
const currentIndex = ref(0);
let lastIndex = 0;
const lyrics_scroll = ({ target }: any) => {
  const { currentTime } = target;
  if (currentIndex.value === lyrics_length - 1) return;
  currentIndex.value = state.music_lyrics.findIndex(
    (item, index) =>
      item.time <= currentTime &&
      currentTime <= state.music_lyrics[index + 1].time
  );

  if (lastIndex === currentIndex.value) return;
  lastIndex = currentIndex.value;

  const htmlElement = document.documentElement;
  const fontSize = parseFloat(window.getComputedStyle(htmlElement).fontSize);
  const marginTop = parseFloat(
    window.getComputedStyle(lyricsRef.value!).marginTop
  );
  const reduceFontSize =
    state.music_lyrics[currentIndex.value].time ===
    state.music_lyrics[currentIndex.value - 1].time
      ? fontSize * 8
      : fontSize * 4;
  lyricsRef.value!.style.marginTop = `${marginTop - reduceFontSize}px`;
};

const settings_modal = ref(false);
const open_settings_modal = () => (settings_modal.value = true);

const songs = ref<MusicList>([]);

const song_filter = (e: ChangeEvent) => {
  const text = e.target.value?.trim();
  if (!text) {
    songs.value = [...music_player.songs_list];
    return;
  }
  songs.value = music_player.songs_list.filter(
    (song) => song.name.toLowerCase().indexOf(text.toLowerCase()) > -1
  );
};
</script>

<template>
  <audio
    ref="audio"
    class="audio"
    :src="current_music_mp3_url"
    @timeupdate="lyrics_scroll"
    @ended="audio_end"
    @canplay="audio_canplay"
  />

  <Drawer
    v-model:open="settings_modal"
    placement="left"
    :closable="false"
    width="300"
  >
    <SettingsDrawer />
  </Drawer>

  <div class="left-bar" :class="{ mobile: !isDesktop, desktop: isDesktop }">
    <div class="settings-open">
      <Badge :dot="app_version !== config_store.app_info.app_version">
        <MenuUnfoldOutlined @click="open_settings_modal" class="icon" />
      </Badge>
      <Input
        class="song-filter"
        @change="song_filter"
        placeholder="F1开始脚本演奏，F2停止脚本"
        style="text-align: center"
      ></Input>
    </div>

    <div class="music-list-container">
      <music-list :songs @set_current_song="set_current_song" />
    </div>
    <div class="player-button-container" v-if="!isDesktop">
      <Button @click="previous_track" shape="circle">
        <template #icon>
          <LeftOutlined />
        </template>
      </Button>

      <Button
        v-show="!state.isPlaying"
        @click="audio_play"
        shape="circle"
        size="large"
        :disabled="!music_player.current_song_value"
      >
        <template #icon>
          <CaretRightOutlined />
        </template>
      </Button>
      <Button
        v-show="state.isPlaying"
        @click="audio_pause"
        shape="circle"
        size="large"
      >
        <template #icon>
          <PauseOutlined />
        </template>
      </Button>

      <Button @click="next_track" shape="circle">
        <template #icon>
          <RightOutlined />
        </template>
      </Button>
    </div>
  </div>
  <div class="lyrics-page" :class="{ mobile: !isDesktop, desktop: isDesktop }">
    <div ref="lyricsRef" class="music-lyrics">
      <div
        v-for="(item, index) in state.music_lyrics"
        :key="index"
        :class="{ 'is-current': index === currentIndex }"
      >
        {{ item.content }}
      </div>
    </div>

    <div class="player-button-container">
      <Button @click="previous_track" shape="circle">
        <template #icon>
          <LeftOutlined />
        </template>
      </Button>

      <Button
        v-show="!state.isPlaying"
        @click="audio_play"
        shape="circle"
        size="large"
        :disabled="!music_player.current_song_value"
      >
        <template #icon>
          <CaretRightOutlined />
        </template>
      </Button>
      <Button
        v-show="state.isPlaying"
        @click="audio_pause"
        shape="circle"
        size="large"
      >
        <template #icon>
          <PauseOutlined />
        </template>
      </Button>

      <Button @click="next_track" shape="circle">
        <template #icon>
          <RightOutlined />
        </template>
      </Button>
    </div>
  </div>
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

.left-bar {
  height: 100%;
  display: flex;
  flex-flow: column nowrap;
  gap: 10px;

  &.desktop {
    flex: 1;
  }

  &.mobile {
    position: relative;
    left: 0;
    z-index: 2;
  }
}

.song-filter {
  width: clamp(200px, 80%, 500px);
}

.music-list-container {
  width: 100%;
  box-sizing: border-box;
  flex: 1;
  overflow-x: hidden;
  overflow-y: scroll;
}

.lyrics-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;

  &.desktop {
    flex: 1;
  }

  &.mobile {
    width: 1px;
    position: absolute;
    left: 0px;
    z-index: 1;
    opacity: 0;
  }
}

.music-lyrics {
  width: 100%;
  height: calc(100% - 60px);
  text-align: center;
  box-sizing: border-box;
  padding-top: calc((100vh - 60px) / 2 - 2rem);
  margin: 0;

  scrollbar-width: none;
  transform: none;
  transition: all 0.5s;
  flex: 1;

  div {
    color: gray;
    font-size: 1rem;
    height: 4rem;
    line-clamp: 2;
    width: 100%;
    text-overflow: ellipsis;
    overflow-wrap: break-word;
    overflow: hidden;

    &.is-current {
      font-size: 1.2rem;
      color: black;
    }
  }
}

.player-button-container {
  width: 100%;
  height: 60px;
  border-radius: 10px;
  background-color: #efefef;
  display: flex;
  justify-content: space-evenly;
  align-items: center;

  .button {
    font-size: 2.5rem;
    color: gray;
  }
}
</style>
