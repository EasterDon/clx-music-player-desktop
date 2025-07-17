<script setup lang="ts">
import {
  reactive,
  computed,
  onMounted,
  useTemplateRef,
  ref,
  nextTick,
} from 'vue';
import { message, Spin } from 'ant-design-vue';
import { useBreakpoints } from '@vueuse/core';
import {
  PlayCircleFilled,
  LeftCircleFilled,
  RightCircleFilled,
  PauseCircleFilled,
} from '@ant-design/icons-vue';
import MusicList from '@/components/MusicList.vue';
import { MusicScriptPlayer } from '@/class/music_script_player';
import { songs_resource_url } from '@/config';
import { get_song_lyrics } from '@/api';

const music_player = new MusicScriptPlayer();

const breakpoints = useBreakpoints({
  mobile: 768, // ≤768px 视为移动端
});

const isDesktop = breakpoints.greater('mobile');

const state = reactive<{ music_lyrics: Lyrics; isPlaying: boolean }>({
  music_lyrics: [],
  isPlaying: false,
});

let lyrics_length = 0;
const spinning = ref(false);
const audio_canplay = () => {
  spinning.value = false;
  state.music_lyrics = lyrics;
};
let lyrics: { time: number; content: string }[] = [];
const set_current_song = async (song: MusicListItem) => {
  if (
    music_player.current_song_value &&
    music_player.current_song_value.id === song.id
  ) {
    return;
  }
  audio_end();
  state.music_lyrics = [];
  music_player.current_song_value = song;
  spinning.value = true;
  try {
    const lyrics_string = await get_song_lyrics(song.id);
    lyrics = trans_lyrics_string(lyrics_string);
    lyrics_length = lyrics.length;
  } catch (err) {
    if (err instanceof Error) {
      message.error(err.message);
      spinning.value = false;
      return;
    }
    message.error('请求似乎出错了哦');
    spinning.value = false;
  }
};

// 音乐mp3源切换
const current_music_mp3_url = computed(
  () =>
    `${songs_resource_url}/${music_player.current_song_value?.id}/music.mp3`,
);
onMounted(async () => {
  await music_player.set_hotkey();
  await music_player.init_songs_list();
});
const audio = useTemplateRef('audio');
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
      (song) => song.id === current_song_id,
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
      (song) => song.id === current_song_id,
    );
    next = music_player.songs_list[index + 1];
  } else {
    next = music_player.songs_list[0];
  }
  await set_current_song(next);
  console.log(next);
  audio_play();
};

const audio_end = () => {
  if (state.isPlaying) state.isPlaying = false;
  lastIndex = 0;
  currentIndex.value = 0;
  lyricsRef.value!.style.marginTop = '0px';
};

// @future 分割歌词滚动页
const timeTagRegex = /\[(\d+):(\d+)(?:\.(\d+))?\]/g;
const trans_lyrics_string = (lyrics_string: string) => {
  const string_array = lyrics_string.split('\n');
  let lyrics: Array<{ time: number; content: string }> = [];
  string_array.forEach((line) => {
    let match;
    const content = line.replace(timeTagRegex, '').trim();

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
const lyricsRef = useTemplateRef('lyricsRef');
const currentIndex = ref(0);
let lastIndex = 0;
const lyrics_scroll = ({ target }: any) => {
  const { currentTime } = target;
  if (currentIndex.value === lyrics_length - 1) return;
  currentIndex.value = state.music_lyrics.findIndex(
    (item, index) =>
      item.time <= currentTime &&
      currentTime <= state.music_lyrics[index + 1].time,
  );

  if (lastIndex === currentIndex.value) return;
  lastIndex = currentIndex.value;

  const htmlElement = document.documentElement;
  const fontSize = parseFloat(window.getComputedStyle(htmlElement).fontSize);
  const marginTop = parseFloat(
    window.getComputedStyle(lyricsRef.value!).marginTop,
  );
  const reduceFontSize =
    state.music_lyrics[currentIndex.value].time ===
    state.music_lyrics[currentIndex.value - 1].time
      ? fontSize * 8
      : fontSize * 4;
  lyricsRef.value!.style.marginTop = `${marginTop - reduceFontSize}px`;
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
  <div class="left-bar" :class="{ mobile: !isDesktop, desktop: isDesktop }">
    <div class="music-list-container">
      <music-list
        :songs="music_player.songs_list"
        @set_current_song="set_current_song"
      />
    </div>
    <div class="player-button-container" v-if="!isDesktop">
      <LeftCircleFilled class="button" @click="previous_track" />
      <PlayCircleFilled
        class="button"
        v-show="!state.isPlaying"
        @click="audio_play"
      />
      <PauseCircleFilled
        class="button"
        v-show="state.isPlaying"
        @click="audio_pause"
      />
      <RightCircleFilled class="button" @click="next_track" />
    </div>
  </div>
  <div class="lyrics-page" :class="{ mobile: !isDesktop, desktop: isDesktop }">
    <div ref="lyricsRef" class="music-lyrics">
      <div v-show="spinning" style="width: 100%">
        <Spin :spinning size="large" tip="音乐加载中..." />
      </div>

      <div
        v-for="(item, index) in state.music_lyrics"
        :key="index"
        :class="{ 'is-current': index === currentIndex }"
      >
        {{ item.content }}
      </div>
    </div>

    <div class="player-button-container">
      <LeftCircleFilled class="button" @click="previous_track" />
      <PlayCircleFilled
        class="button"
        v-show="!state.isPlaying"
        @click="audio_play"
      />
      <PauseCircleFilled
        class="button"
        v-show="state.isPlaying"
        @click="audio_pause"
      />
      <RightCircleFilled class="button" @click="next_track" />
    </div>
  </div>
</template>

<style lang="less" scoped>
.audio {
  display: none;
}

.left-bar {
  height: 100%;
  display: flex;
  flex-flow: column nowrap;

  &.desktop {
    flex: 1;
  }

  &.mobile {
    position: relative;
    left: 0;
    z-index: 2;
  }
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
