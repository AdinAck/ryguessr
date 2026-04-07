"use client";
import { useCookies } from "react-cookie";
import { useLoadScript } from "@react-google-maps/api";
import { useEffect, useState, useRef } from "react";
import { GameView } from "../game/GameView";

// types
import UserInit from "@/types/user-init";

// zustand
import { playerActions } from "@/store/useSettingsStore";
import { RoomSettingsActions } from "@/store/useSettingsStore";
import { useUserSettings } from "@/store/useSettingsStore";

export const Init = () => {
  const [responseCode, setResponseCode] = useState<number | undefined>();

  const hasInitialized = useRef(false);

  // Google Maps api
  const { isLoaded } = useLoadScript({
    googleMapsApiKey: process.env.NEXT_PUBLIC_GOOGLE_MAPS_API_KEY || "",
  });

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
          const init_response: UserInit = await response.json();
          console.log(init_response);
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

  return isLoaded && responseCode == 200 ? (
    <GameView />
  ) : (
    <div className="h-screen w-screen bg-black text-white flex items-center justify-center">
      Loading UwU...
    </div>
  );
};
