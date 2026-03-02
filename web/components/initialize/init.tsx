"use client"
import { useCookies } from "react-cookie";
import { useLoadScript } from "@react-google-maps/api";
import { useEffect, useState } from "react";
import { GameView } from "../game/GameView";


export const Init = () => {
  // cookies
  const [cookies, setCookie, removeCookie] = useCookies(['USER_ID']);

  // Google Maps api
  const { isLoaded } = useLoadScript({ googleMapsApiKey: process.env.NEXT_PUBLIC_GOOGLE_MAPS_API_KEY || "" });

  useEffect(() => {
    const userID = crypto.randomUUID();
    setCookie('USER_ID', userID);

  }, []);


  return (
    (isLoaded && cookies.USER_ID
      ?
      <GameView userID={cookies.USER_ID} />
      :
      <div className="h-screen w-screen bg-black text-white flex items-center justify-center">Loading UwU...</div>
    )
  );
};
