<template>
  <div class="lyrics-page-shell" :class="{ 'as-pane': props.as_pane }">
    <audio
      ref="audio"
      class="audio"
      :src="current_music_mp3_url"
      @timeupdate="lyrics_scroll"
      @loadedmetadata="sync_progress_from_audio"
      @durationchange="sync_progress_from_audio"
      @ended="audio_end"
    />
    <div
      class="lyrics-page"
      :class="{ mobile: !is_desktop, desktop: is_desktop }"
    >
      <button
        v-if="!is_desktop"
        class="lyrics-back-btn"
        type="button"
        @click="emits('back_to_list')"
      >
        列表
      </button>
      <div ref="scroll_ref" class="scroll-container">
        <div v-if="show_placeholder" class="lyrics-placeholder">
          暂未选择歌曲
        </div>
        <div
          v-show="!show_placeholder"
          ref="music_lyrics_ref"
          class="music-lyrics"
          :class="{
            'lyrics-ready': lyrics_ready,
            'lyrics-no-animate': lyrics_no_animate,
          }"
        >
          <div
            v-for="(item, index) in state.music_lyrics"
            :key="index"
            :class="{ 'is-current': index === current_index }"
            :ref="(el) => set_lyric_item_ref(el as Element | null, index)"
          >
            {{ item.content }}
          </div>
        </div>
      </div>

      <PlayerControls
        class="lyrics-controls"
        :is_playing="music_store.is_playing"
        :play_disabled="!music_store.current_music"
        :current_time="current_time"
        :duration="duration"
        @audio_play="audio_play"
        @audio_pause="audio_pause"
        @previous_track="previous_track"
        @next_track="next_track"
        @seek_progress="seek_audio_progress"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  computed,
  nextTick,
  onMounted,
  reactive,
  ref,
  useTemplateRef,
} from 'vue';
import { useBreakpoints } from '@vueuse/core';
import { use_music_store } from '@/store/player';
import { desktop_breakpoint, music_resource_url } from '@/config';
import PlayerControls from '@/components/player_controls/index.vue';

const props = withDefaults(
  defineProps<{
    /** 作为主布局右侧面板时启用固定宽度规则 */
    as_pane?: boolean;
  }>(),
  {
    as_pane: false,
  },
);

const emits = defineEmits<{
  set_current_music: [music: MusicListItem];
  progress_change: [payload: { current_time: number; duration: number }];
  back_to_list: [];
}>();

const music_store = use_music_store();
const breakpoints = useBreakpoints({ mobile: desktop_breakpoint });

const is_desktop = breakpoints.greater('mobile');
const show_placeholder = computed(() => !music_store.current_music);

const state = reactive<{
  music_lyrics: Lyrics;
}>({
  music_lyrics: [],
});

/**
 * 每行单独 new 带 /g 的正则做 exec，避免复用同一 RegExp 时 lastIndex 在行间残留。
 * 去掉时间标签后的文案作为该组时间戳共用的 content。
 */
const trans_lyrics_string = (lyrics_string: string): Lyrics => {
  const string_array = lyrics_string.split('\n');
  const lyrics: Lyrics = [];

  for (const line of string_array) {
    const content = line.replace(/\[(\d+):(\d+)(?:\.(\d+))?\]/g, '').trim();
    const line_tags = /\[(\d+):(\d+)(?:\.(\d+))?\]/g;
    let match: RegExpExecArray | null;
    while ((match = line_tags.exec(line)) !== null) {
      const mm = parseInt(match[1], 10);
      const ss = parseInt(match[2], 10);
      const xx = parseInt(match[3] ?? '0', 10);
      const time = mm * 60 + ss + xx / 100;
      lyrics.push({ time, content });
    }
  }
  return lyrics;
};

const set_lyrics = (lyrics_string: string) => {
  lyrics_ready.value = false;
  lyrics_no_animate.value = true;
  state.music_lyrics = trans_lyrics_string(lyrics_string);
  lyric_item_ref_list.value = [];
  current_index.value = 0;
  // 仅在选歌加载歌词后居中首句；默认占位文案不做居中处理。
  nextTick(() => {
    // 定位成功后再显示真实歌词，避免“先在顶部闪一下再居中”。
    center_current_lyric_with_retry(0);
  });
};

const current_music_mp3_url = computed(() =>
  music_store.current_music
    ? `${music_resource_url}/${music_store.current_music.id}/music.mp3`
    : '',
);

const scroll_ref = useTemplateRef('scroll_ref');
const music_lyrics_ref = useTemplateRef('music_lyrics_ref');
const lyric_item_ref_list = ref<HTMLElement[]>([]);
const current_index = ref(0);
const current_time = ref(0);
const duration = ref(0);
const lyrics_ready = ref(false);
const lyrics_no_animate = ref(false);

const emit_progress_change = () => {
  emits('progress_change', {
    current_time: current_time.value,
    duration: duration.value,
  });
};

const sync_progress_from_audio = () => {
  if (!audio.value) return;
  current_time.value = Number.isFinite(audio.value.currentTime)
    ? audio.value.currentTime
    : 0;
  duration.value = Number.isFinite(audio.value.duration)
    ? audio.value.duration
    : 0;
  emit_progress_change();
};
const set_lyric_item_ref = (el: Element | null, index: number) => {
  if (!el) return;
  lyric_item_ref_list.value[index] = el as HTMLElement;
};

/**
 * 基于当前歌词真实 offsetTop/height 计算平移量，避免长歌词（多行）时按固定行高计算产生错位。
 */
const center_current_lyric = (index: number): boolean => {
  if (!scroll_ref.value || !music_lyrics_ref.value) return false;
  const target = lyric_item_ref_list.value[index];
  if (!target) return false;
  // 隐藏态或未完成布局时，高度可能为 0，此时不做位移计算，避免写入错误 transform。
  if (scroll_ref.value.clientHeight <= 0 || target.clientHeight <= 0) {
    return false;
  }
  const target_center = target.offsetTop + target.clientHeight / 2;
  const new_px = scroll_ref.value.clientHeight / 2 - target_center;
  music_lyrics_ref.value.style.transform = `translateY(${new_px}px)`;
  return true;
};

const center_current_lyric_with_retry = (index: number, retry = 30) => {
  const run = (rest: number) => {
    const ok = center_current_lyric(index);
    if (ok) {
      // 再等一帧做第二次校正，确保布局稳定后再显示，避免“先顶上再回中”。
      requestAnimationFrame(() => {
        const stable_ok = center_current_lyric(index);
        if (stable_ok) {
          lyrics_ready.value = true;
          requestAnimationFrame(() => {
            lyrics_no_animate.value = false;
          });
          return;
        }
        if (rest <= 0) {
          lyrics_ready.value = true;
          lyrics_no_animate.value = false;
          return;
        }
        requestAnimationFrame(() => run(rest - 1));
      });
      return;
    }
    if (rest <= 0) {
      // 兜底：避免极端情况下永久隐藏
      lyrics_ready.value = true;
      lyrics_no_animate.value = false;
      return;
    }
    requestAnimationFrame(() => run(rest - 1));
  };
  requestAnimationFrame(() => run(retry));
};

const update_lyric_position_by_time = (time: number) => {
  const lyrics = state.music_lyrics;
  const lyrics_length = lyrics.length;
  if (lyrics_length === 0 || !scroll_ref.value) return;

  let new_index = -1;
  for (let i = 0; i < lyrics_length; i++) {
    const current_lyric = lyrics[i];
    const next_lyric = lyrics[i + 1];

    if (next_lyric) {
      if (time >= current_lyric.time && time < next_lyric.time) {
        new_index = i;
        break;
      }
    } else if (time >= current_lyric.time) {
      new_index = i;
      break;
    }
  }

  if (new_index === -1 || new_index === current_index.value) return;
  current_index.value = new_index;
  center_current_lyric(new_index);
};

const lyrics_scroll = (e: Event) => {
  const { currentTime } = e.target as HTMLAudioElement;
  sync_progress_from_audio();
  update_lyric_position_by_time(currentTime);
};

const reset_transform_y = () => {
  current_time.value = 0;
  current_index.value = 0;
  lyrics_ready.value = false;
  emit_progress_change();
  // 简化策略：默认状态不做“强制居中”，避免窄宽切换时额外位移干扰。
  if (music_lyrics_ref.value) {
    music_lyrics_ref.value.style.transform = 'translateY(0px)';
  }
};

/**
 * 拖动进度条后同步更新 audio 时间与歌词 transform。
 */
const seek_audio_progress = (value: number) => {
  if (!audio.value) return;
  const max = duration.value > 0 ? duration.value : audio.value.duration || 0;
  const target = Math.min(Math.max(value, 0), max);
  audio.value.currentTime = target;
  current_time.value = target;
  emit_progress_change();
  update_lyric_position_by_time(target);
};

const audio = useTemplateRef('audio');
const audio_play = () => {
  audio.value?.play();
  music_store.set_is_playing(true);
};

const audio_pause = () => {
  music_store.set_is_playing(false);

  audio.value?.pause();
};

const previous_track = async () => {
  if (!music_store.current_music?.id) return void 0;
  audio_pause();
  await nextTick();
  const length = music_store.music_list.length;
  const current_music_id = music_store.current_music.id;
  let pre: null | MusicListItem = null;
  if (music_store.music_list[0].id !== current_music_id) {
    const index = music_store.music_list.findIndex(
      (music) => music.id === current_music_id,
    );
    pre = music_store.music_list[index - 1];
  } else {
    pre = music_store.music_list[length - 1];
  }
  emits('set_current_music', pre);
};

const next_track = async () => {
  if (!music_store.current_music?.id) return void 0;
  audio_pause();
  const length = music_store.music_list.length;
  const current_music_id = music_store.current_music.id;
  let next: null | MusicListItem = null;
  if (music_store.music_list[length - 1].id !== current_music_id) {
    const index = music_store.music_list.findIndex(
      (music) => music.id === current_music_id,
    );
    next = music_store.music_list[index + 1];
  } else {
    next = music_store.music_list[0];
  }
  emits('set_current_music', next);
};

const audio_end = () => {
  if (music_store.is_playing) music_store.set_is_playing(false);
  current_time.value = 0;
  emit_progress_change();
  reset_transform_y();
};

onMounted(() => {
  reset_transform_y();
});

defineExpose({
  audio_play,
  audio_pause,
  audio_end,
  set_lyrics,
  previous_track,
  next_track,
  seek_audio_progress,
});
</script>

<style scoped lang="less">
.lyrics-page-shell {
  height: 100%;
  min-width: 0;
}

.lyrics-page-shell.as-pane {
  /* 右侧固定列宽（纯 px 范围），左侧自动吃满剩余空间 */
  flex: 0 0 480px;
  width: 480px;
  max-width: 820px;
}

.lyrics-page {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.lyrics-back-btn {
  position: absolute;
  top: 10px;
  left: 10px;
  z-index: 6;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 999px;
  padding: 2px 10px;
  font-size: 12px;
  cursor: pointer;
}

.lyrics-controls {
  display: flex;
}

.scroll-container {
  flex: 1;
  width: 100%;
  box-sizing: border-box;
  overflow: hidden;
  position: relative;
}

.lyrics-placeholder {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #7a7a7a;
  font-size: 1rem;
  text-align: center;
  pointer-events: none;
}

.music-lyrics {
  text-align: center;
  box-sizing: border-box;
  transition: all 0.5s;
  opacity: 0;

  &.lyrics-ready {
    opacity: 1;
  }

  &.lyrics-no-animate {
    transition: none;
  }

  div {
    color: gray;
    font-size: 1rem;
    min-height: 4rem;
    width: 100%;
    white-space: pre-wrap;
    overflow-wrap: break-word;
    word-break: break-word;

    &.is-current {
      font-size: 1.2rem;
      color: black;
    }
  }
}

@media (max-width: @desktop-breakpoint) {
  .lyrics-page-shell.as-pane {
    flex: 1;
    width: 100%;
    min-width: 0;
  }

  .lyrics-page {
    background: #fff;
  }
}
</style>
