"use client";
import { useEffect, useState, useRef } from "react";

// types
import GameInit from "@/types/game-init";

// zustand
import {
  playerActions,
  RoomSettingsActions,
  useUserSettings,
  APIKEYActions,
  useAPIKEY,
} from "@/store/useSettingsStore";
import GoogleMapLoader from "./map-load";

export const Init = () => {
  const [responseCode, setResponseCode] = useState<number | undefined>();

  const hasInitialized = useRef(false);

  const google_maps_api_key = useAPIKEY((state) => state.google_maps_api_key);

  useEffect(() => {
    if (hasInitialized.current) return;
    hasInitialized.current = true;

    const initializeRyguessr = async () => {
      const { savedUsername, savedIconColor } = useUserSettings.getState();

      const payload: Record<string, string> = {};
      if (savedUsername) payload.username = savedUsername;
      if (savedIconColor) payload.color = savedIconColor;

      try {
        const response = await fetch("/api/init", {
          method: "POST",
          credentials: "same-origin",
          body:
            Object.keys(payload).length > 0
              ? JSON.stringify(payload)
              : JSON.stringify({}),
          headers: {
            "Content-Type": "application/json",
          },
        });

        if (response.ok) {
          const init_response: GameInit = await response.json();
          APIKEYActions.updateAPIKEY(init_response.api_key);
          RoomSettingsActions.updateRoomCode(init_response.room_id);
          playerActions.setSessionData(
            init_response.username,
            init_response.color,
          );
          setResponseCode(response.status);
        } else {
          console.error("Initialization failed... status:", response.status);
        }
      } catch (error) {
        console.error("Network error during init:", error);
      }
    };
    initializeRyguessr();
  }, []);

  return responseCode == 200 && google_maps_api_key ? (
    <GoogleMapLoader APIKEY={google_maps_api_key} />
  ) : (
    <div className="h-screen w-screen bg-black text-white flex items-center justify-center">
      Loading UwU...
    </div>
  );
};
