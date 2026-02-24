"use client"
import { CookiesProvider, Cookies, useCookies } from "react-cookie";
import { useLoadScript } from "@react-google-maps/api";
import { useEffect } from "react";
import { GameView } from "../game/GameView";


export const Init = () => {
  // cookies
  const [cookies, setCookie, removeCookie] = useCookies(['USER_ID']);

  // Google Maps api
  const { isLoaded } = useLoadScript({ googleMapsApiKey: process.env.NEXT_PUBLIC_GOOGLE_MAPS_API_KEY || "" });

  useEffect(() => {
    const userID = crypto.randomUUID();
    setCookie('USER_ID', userID);
    console.log(typeof cookies);
  }, []);



  return (
    (isLoaded && cookies.USER_ID
      ?
      <CookiesProvider defaultSetOptions={{ path: '/' }}>
        <GameView props={cookies} />
      </CookiesProvider>
      :

      <div className="h-screen w-screen bg-black text-white flex items-center justify-center">Loading UwU...</div>

    )
  );
};
