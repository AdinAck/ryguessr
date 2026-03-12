import { create } from "zustand";

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

