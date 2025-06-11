type MusicListItem = {
  id: number;
  author: string;
  name: string;
};

type MusicList = Array<MusicListItem>;

type MusicChars = Array<string | number>;

type MusicScriptPlayerState = {
  songs_list: MusicList;
  current_song_value: null | MusicListItem;
  continue_play: boolean;
};

type Lyrics = Array<{ time: number; content: string }>;
