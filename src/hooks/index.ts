import { ref } from 'vue';

export const useImage = (
  id: number,
  idShowArray: boolean[],
  imgUrl: string,
) => {
  const loadedImages = ref({});

  const loadImage = () => {
    return new Promise((resolve) => {
      const img = new Image();
      img.src = imgUrl;
      img.onload = () => {
        idShowArray[id] = true;
        resolve(true);
      };
      img.onerror = () => {
        img.src = '/images/default_music_cover.png';
        idShowArray[id] = true;
        resolve(false);
      };
    });
  };

  return { loadedImages, loadImage };
};
