type MusicListItem = {
  id: number;
  author: string;
  name: string;
};

type MusicList = Array<MusicListItem>;

type MusicChars = Array<string | number>;

type MusicScriptPlayerState = {
  music_list: MusicList;
  current_song_value: null | MusicListItem;
  continue_play: boolean;
};

type Lyrics = Array<{ time: number; content: string }>;

type App_Info = {
  id: number;
  announcement: string;
  app_version: string;
  app_download_link: string;
  app_version_description: string;
};

type Api_Envelope<T> = {
  data: T;
};
