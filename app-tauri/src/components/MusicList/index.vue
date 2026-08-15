<script setup lang="ts">
import { reactive } from 'vue';
import { music_resource_url } from '@/config';
import { use_music_store } from '@/store/player';

const { music_list } = defineProps<{ music_list: MusicList }>();

const music_store = use_music_store();

const cover_src = (song: MusicListItem) => {
  return `${music_resource_url}/${song.id}/music.jpg`;
};

const emit = defineEmits(['set_current_music']);
const set_current_music = (item: MusicListItem) => {
  if (music_store.current_music && music_store.current_music.id === item.id) {
    return void 0;
  }
  emit('set_current_music', item);
};

const cover_loaded_map = reactive<Record<number, boolean>>({});
const on_cover_load = (id: number) => {
  cover_loaded_map[id] = true;
};
</script>

<template>
  <div class="music-list">
    <article
      v-for="item in music_list"
      :key="item.id"
      class="music-item"
      :class="{
        'music-item--cover-pending': !cover_loaded_map[item.id],
        'music-item--selected':
          music_store.current_music && music_store.current_music.id === item.id,
      }"
      @click="set_current_music(item)"
    >
      <div class="music-item__cover">
        <img
          :src="cover_src(item)"
          alt=""
          loading="lazy"
          decoding="async"
          @load="on_cover_load(item.id)"
        />
      </div>
      <div class="music-item__meta">
        <p class="music-item__title">{{ item.name }}</p>
        <p class="music-item__author">{{ item.author }}</p>
      </div>
    </article>
  </div>
</template>

<style lang="less" scoped>
.music-list {
  width: 100%;
  display: grid;
  /* 宽屏：卡片宫格，与原先 Card 网格一致 */
  grid-template-columns: repeat(auto-fill, 150px);
  grid-gap: 20px;
  justify-content: center;
  box-sizing: border-box;
  padding: 15px 0;
  transition:
    grid-template-columns 0.48s cubic-bezier(0.22, 1, 0.36, 1),
    gap 0.48s cubic-bezier(0.22, 1, 0.36, 1),
    padding 0.48s cubic-bezier(0.22, 1, 0.36, 1);
}

.music-item {
  display: flex;
  flex-direction: column;
  border-radius: 5px;
  overflow: hidden;
  cursor: pointer;
  transition:
    opacity 0.48s cubic-bezier(0.22, 1, 0.36, 1),
    box-shadow 0.28s cubic-bezier(0.22, 1, 0.36, 1),
    background-color 0.48s cubic-bezier(0.22, 1, 0.36, 1),
    border-radius 0.48s cubic-bezier(0.22, 1, 0.36, 1),
    padding 0.48s cubic-bezier(0.22, 1, 0.36, 1),
    gap 0.48s cubic-bezier(0.22, 1, 0.36, 1),
    transform 0.48s cubic-bezier(0.22, 1, 0.36, 1);
  animation: layout-fade-in 0.42s cubic-bezier(0.22, 1, 0.36, 1) both;

  &:hover {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  }
}

@keyframes layout-fade-in {
  from {
    opacity: 0.85;
    transform: translateY(4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.music-item__cover {
  width: 100%;
  aspect-ratio: 1 / 1;
  overflow: hidden;
  background: #f5f5f5;
  transition:
    width 0.48s cubic-bezier(0.22, 1, 0.36, 1),
    border-radius 0.48s cubic-bezier(0.22, 1, 0.36, 1);

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
}

.music-item--cover-pending {
  opacity: 0;
}

.music-item--cover-pending .music-item__cover img {
  opacity: 0;
}

.music-item__meta {
  padding: 8px 4px 0;
  min-height: 0;
  transition: padding 0.48s cubic-bezier(0.22, 1, 0.36, 1);
}

.music-item__title {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 400;
  color: #000;
  overflow-wrap: break-word;
  white-space: pre-wrap;
  transition: font-size 0.48s cubic-bezier(0.22, 1, 0.36, 1);
}

.music-item__author {
  margin: 4px 0 0;
  font-size: 0.85rem;
  color: #696969;
  transition: font-size 0.48s cubic-bezier(0.22, 1, 0.36, 1);
}

.music-item--selected {
  box-shadow: 0 0 2px 2px rgba(71, 167, 235, 0.86);
}

/* 窄屏 */
@media (max-width: @desktop-breakpoint) {
  .music-list {
    grid-template-columns: 1fr;
    grid-gap: 0;
    padding: 0;
  }

  .music-item {
    flex-direction: row;
    align-items: center;
    gap: 20px;
    padding: 8px 15px;
    border-radius: 5px;
    box-sizing: border-box;

    &:hover {
      background-color: #f0f0f0;
    }
  }

  .music-item--selected {
    box-shadow: none;
    background-color: #f0f0f0;
  }

  .music-item__cover {
    width: 60px;
    flex-shrink: 0;
    aspect-ratio: 1 / 1;
    border-radius: 50%;
    overflow: hidden;
  }

  .music-item--cover-pending {
    opacity: 1;
  }

  .music-item--cover-pending .music-item__cover img {
    opacity: 0;
  }

  .music-item__meta {
    flex: 1;
    padding: 0;
    min-width: 0;
  }

  .music-item__title {
    font-size: 1.2rem;
    text-align: left;
  }

  .music-item__author {
    text-align: left;
  }
}
</style>
