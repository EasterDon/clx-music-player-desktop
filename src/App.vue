<script setup lang="ts">
import { reactive, computed, onMounted, useTemplateRef, ref } from 'vue';
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
const set_current_song = async (song: MusicListItem) => {
  if (
    music_player.current_song_value &&
    music_player.current_song_value.id === song.id
  ) {
    return;
  }
  audio_pause();
  music_player.current_song_value = song;
  const lyrics_string = await get_song_lyrics(song.id);
  state.music_lyrics = trans_lyrics_string(lyrics_string);
  lyrics_length = state.music_lyrics.length;
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
  state.isPlaying = true;
  audio.value?.play();
};

const audio_pause = () => {
  state.isPlaying = false;
  audio.value?.pause();
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
const lyrics = useTemplateRef('lyrics');
const currentIndex = ref(0);
let lastIndex = 0;
const lyrics_scroll = ({ target }: { target: HTMLAudioElement }) => {
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
    window.getComputedStyle(lyrics.value!).marginTop,
  );
  console.log(
    state.music_lyrics[currentIndex.value].time,
    state.music_lyrics[currentIndex.value + 1].time,
  );
  const reduceFontSize =
    state.music_lyrics[currentIndex.value].time ===
    state.music_lyrics[currentIndex.value - 1].time
      ? fontSize * 8
      : fontSize * 4;
  lyrics.value!.style.marginTop = `${marginTop - reduceFontSize}px`;
  console.log(`${reduceFontSize}`);
};
</script>

<template>
  <audio
    ref="audio"
    class="audio"
    :src="current_music_mp3_url"
    @timeupdate="lyrics_scroll"
  />
  <div class="left-bar">
    <div class="music-list-container">
      <music-list
        :songs="music_player.songs_list"
        @set_current_song="set_current_song"
      />
    </div>
    <div class="player-button-container" v-if="!isDesktop">
      <LeftCircleFilled class="button" />
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
      <RightCircleFilled class="button" />
    </div>
  </div>
  <div class="lyrics-page" v-show="isDesktop">
    <div ref="lyrics" class="music-lyrics">
      <div
        v-for="(item, index) in state.music_lyrics"
        :key="index"
        :class="{ 'is-current': index === currentIndex }"
      >
        {{ item.content }}
      </div>
    </div>
    <div class="player-button-container">
      <LeftCircleFilled class="button" />
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
      <RightCircleFilled class="button" />
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
  flex: 1;
}

.music-list-container {
  width: 100%;
  box-sizing: border-box;
  flex: 1;
  overflow-x: hidden;
  overflow-y: scroll;
}

.lyrics-page {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.music-lyrics {
  width: 100%;
  text-align: center;
  box-sizing: border-box;
  padding-top: calc((100% - 60px) / 2 + 4rem);
  margin: 0;
  overflow-x: hidden;
  overflow-y: scroll;
  scrollbar-width: none;
  transform: none;
  transition: all 0.5s;
  flex: 1;

  div {
    color: gray;
    font-size: 1.2rem;
    height: 4rem;
    line-clamp: 2;
    width: 100%;
    text-overflow: ellipsis;
    display: flex;
    justify-content: center;
    align-items: center;

    &.is-current {
      font-size: 1.5rem;
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
