"use client"
import { useCookies } from "react-cookie"
import { useLoadScript } from "@react-google-maps/api"
import { useEffect } from "react"
import { GameView } from "../game/GameView"

// types
import UserInit from "@/types/user-init"

// zustand
import { playerActions } from "@/store/useSettingsStore"
import { RoomSettingsActions } from "@/store/useSettingsStore"
import { useUserSettings } from "@/store/useSettingsStore"
import { init } from "next/dist/compiled/webpack/webpack"

const api_init = async (currentUserID: string, savedUsername?: string, savedIconColor?: string) => {
  const response = await fetch("/api/init", {
    method: "POST",
    body: savedUsername && savedIconColor ? JSON.stringify({
      username: savedUsername, color: savedIconColor
    }) : savedUsername && !savedIconColor ? JSON.stringify({ username: savedUsername }) : !savedUsername && savedIconColor ? JSON.stringify({ color: savedIconColor }) : JSON.stringify({}),
    headers: {
      "Content-Type": "application/json",
      "Client-Id": currentUserID,
    },
  })
  if (!response.ok) {
    console.log(response);
  } else {
    const init_response: UserInit = await response.json();
    RoomSettingsActions.updateRoomCode(init_response.room_id);
    console.log(init_response.color);
    playerActions.setSessionData(init_response.username, init_response.color);
  }
}

export const Init = () => {
  // cookies
  const [cookies, setCookie, removeCookie] = useCookies(["USER_ID"]);

  const { savedUsername, savedIconColor } = useUserSettings.getState();

  // Google Maps api
  const { isLoaded } = useLoadScript({
    googleMapsApiKey: process.env.NEXT_PUBLIC_GOOGLE_MAPS_API_KEY || "",
  })

  useEffect(() => {
    let currentUserID = cookies.USER_ID;

    if (!currentUserID) {
      const currentUserID = crypto.randomUUID();
      setCookie("USER_ID", currentUserID);
    }

    api_init(currentUserID, savedUsername, savedIconColor);
  }, [cookies])

  return isLoaded && cookies.USER_ID ? (
    <GameView userID={cookies.USER_ID} />
  ) : (
    <div className="h-screen w-screen bg-black text-white flex items-center justify-center">
      Loading UwU...
    </div>
  )
}
