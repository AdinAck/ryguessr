import { create } from "zustand";
import { persist } from 'zustand/middleware'

type RoomState = {
  roomCode: string,
}

type RoomStateAction = {
  updateRoomCode: (roomCode: RoomState['roomCode']) => void,
}

type StreetviewState = {
  panningGesturesEnabled: boolean,
  streetNamesEnabled: boolean,
  zoomGesturesEnabled: boolean,
  userNavigationEnabled: boolean
}

type StreetviewStateAction = {
  setPanningGesturesEnabled: (panningGesturesEnabled: StreetviewState['panningGesturesEnabled']) => void,
  setStreetNamesEnabled: (streetNamesEnabled: StreetviewState['streetNamesEnabled']) => void,
  setZoomGesturesEnabled: (zoomGesturesEnabled: StreetviewState['zoomGesturesEnabled']) => void,
  setUserNavigationEnabled: (userNaviagtionEnabled: StreetviewState['userNavigationEnabled']) => void,
}

interface UserState {
  custom_username: string;
  gen_username: string;
  custom_iconColor: string;
  gen_iconColor: string,
}

type UserStateAction = {
  setCustomUsername: (username: UserState['custom_username']) => void,
  setGenUsername: (username: UserState['gen_username']) => void,
  setCustomIconColor: (iconColor: UserState['custom_iconColor']) => void
  setGenIconColor: (iconColor: UserState['gen_iconColor']) => void,
};

export const useRoomSettings = create<RoomState>()(() => ({
  roomCode: "",
}))

export const RoomSettingsActions: RoomStateAction = {
  updateRoomCode: (roomCode: string) => useRoomSettings.setState({ roomCode })
}

export const useStreetviewSettings = create<StreetviewState>()(() => ({
  panningGesturesEnabled: true,
  streetNamesEnabled: false,
  zoomGesturesEnabled: true,
  userNavigationEnabled: true
}))

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
  }

};

export const useUserSettings = create<UserState>()(
  persist(
    () => ({
      custom_username: "",
      gen_username: "",
      custom_iconColor: "",
      gen_iconColor: "",
    }),
    { 
      name: 'player-preferences-storage', 

      partialize: (state) => ({
        custom_username: state.custom_username,
        custom_iconColor: state.custom_iconColor,
      })
    }
  )
);

export const userStateActions: UserStateAction = {
  setCustomUsername: (custom_username: string) => {
    useUserSettings.setState({ custom_username })
  },
  setGenUsername: (gen_username: string) => {
    useUserSettings.setState({ gen_username })

  },
  setCustomIconColor: (custom_iconColor: string) => {
    useUserSettings.setState({ custom_iconColor })
  },
  setGenIconColor: (gen_iconColor: string) => {
    useUserSettings.setState({ gen_iconColor })
  },

};

