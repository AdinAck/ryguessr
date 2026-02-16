"use client"
import { useLoadScript } from "@react-google-maps/api";
import Streetview from "./StreetView";
import { MapOverlay } from "./MapOverlay";

export const GameView = () => {

  const { isLoaded } = useLoadScript({ googleMapsApiKey: process.env.NEXT_PUBLIC_GOOGLE_MAPS_API_KEY || "" });

  if (!isLoaded) {
    return <div>Loading.... UwU</div>;
  }

  return (
    <main className="relative h-screen w-screen overflow-hidden">
      <div className="absolute inset-0 z-0">
        <Streetview />
      </div>

      <div className="absolute bottom-5 left-5 h-64 w-80 z-10">
        <MapOverlay />
      </div>

    </main>

  );
};
