import { create } from "zustand";
import { persist } from "zustand/middleware";

type RoomState = {
  roomCode: string;
};

type RoomStateAction = {
  updateRoomCode: (roomCode: RoomState["roomCode"]) => void;
};

type settingsState = {
  settingsVisibility: boolean;
};

type settingsStateAction = {
  updateSettingsVisibility: (
    settingVisiblity: settingsState["settingsVisibility"],
  ) => void;
};

type APIKEYState = {
  google_maps_api_key: string;
};

type APIKEYActions = {
  updateAPIKEY: (APIKEY: APIKEYState["google_maps_api_key"]) => void;
};

type StreetviewState = {
  panningGesturesEnabled: boolean;
  streetNamesEnabled: boolean;
  zoomGesturesEnabled: boolean;
  userNavigationEnabled: boolean;
};

type AudioState = {
  markerAudioEnabled: boolean;
  lockAudioEnabled: boolean;
};

type AudioStateAction = {
  setMarkerAudioEnabled: (
    markerAudioEnabled: AudioState["markerAudioEnabled"],
  ) => void;

  setLockAudioEnabled: (
    lockAudioEnabled: AudioState["lockAudioEnabled"],
  ) => void;
};

type StreetviewStateAction = {
  setPanningGesturesEnabled: (
    panningGesturesEnabled: StreetviewState["panningGesturesEnabled"],
  ) => void;
  setStreetNamesEnabled: (
    streetNamesEnabled: StreetviewState["streetNamesEnabled"],
  ) => void;
  setZoomGesturesEnabled: (
    zoomGesturesEnabled: StreetviewState["zoomGesturesEnabled"],
  ) => void;
  setUserNavigationEnabled: (
    userNaviagtionEnabled: StreetviewState["userNavigationEnabled"],
  ) => void;
};

type UserState = {
  savedUsername?: string;
  savedIconColor?: string;
};

type GameSession = {
  activeUsername: string;
  activeIconColor: string;
};

export const useAudioState = create<AudioState>(() => ({
  markerAudioEnabled: true,
  lockAudioEnabled: true,
}));

export const audioStateActions: AudioStateAction = {
  setMarkerAudioEnabled: (markerAudioEnabled: boolean) =>
    useAudioState.setState({ markerAudioEnabled }),

  setLockAudioEnabled: (lockAudioEnabled: boolean) =>
    useAudioState.setState({ lockAudioEnabled }),
};

export const useSettingsState = create<settingsState>(() => ({
  settingsVisibility: false,
}));

export const settingsStateActions: settingsStateAction = {
  updateSettingsVisibility: (settingsVisibility: boolean) =>
    useSettingsState.setState({ settingsVisibility }),
};

export const useAPIKEY = create<APIKEYState>()(() => ({
  google_maps_api_key: "",
}));

export const APIKEYActions: APIKEYActions = {
  updateAPIKEY: (google_maps_api_key: string) =>
    useAPIKEY.setState({ google_maps_api_key }),
};

export const useRoomSettings = create<RoomState>()(() => ({
  roomCode: "",
}));

export const RoomSettingsActions: RoomStateAction = {
  updateRoomCode: (roomCode: string) => useRoomSettings.setState({ roomCode }),
};

export const useStreetviewSettings = create<StreetviewState>()(() => ({
  panningGesturesEnabled: true,
  streetNamesEnabled: false,
  zoomGesturesEnabled: true,
  userNavigationEnabled: true,
}));

export const StreetviewStateActions: StreetviewStateAction = {
  setPanningGesturesEnabled: (panningGesturesEnabled: boolean) => {
    useStreetviewSettings.setState({ panningGesturesEnabled });
  },

  setStreetNamesEnabled: (streetNamesEnabled: boolean) => {
    useStreetviewSettings.setState({ streetNamesEnabled });
  },

  setZoomGesturesEnabled: (zoomGesturesEnabled: boolean) => {
    useStreetviewSettings.setState({ zoomGesturesEnabled });
  },

  setUserNavigationEnabled: (userNavigationEnabled: boolean) => {
    useStreetviewSettings.setState({ userNavigationEnabled });
  },
};

export const useUserSettings = create<UserState>()(
  persist(
    (): UserState => ({
      savedUsername: undefined,
      savedIconColor: undefined,
    }),
    { name: "player-preferences-storage" },
  ),
);

export const useGameSession = create<GameSession>()(() => ({
  activeUsername: "Loading...",
  activeIconColor: "#FFFFFF",
}));

export const playerActions = {
  saveCustomUsername: (name: string) => {
    useUserSettings.setState({ savedUsername: name });
    useGameSession.setState({ activeUsername: name });
  },

  saveCustomColor: (color: string) => {
    useUserSettings.setState({ savedIconColor: color });
    useGameSession.setState({ activeIconColor: color });
  },

  setActiveUsername: (activeUsername: string) => {
    useGameSession.setState({ activeUsername });
  },

  setActiveColor: (activeIconColor: string) => {
    useGameSession.setState({ activeIconColor });
  },
};
