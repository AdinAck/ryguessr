"use client"
import { useLoadScript } from "@react-google-maps/api";
import Streetview from "./StreetView";
import { MapOverlay } from "./MapOverlay";
import { Location } from "@/types/coordinate_type";
import { useCallback, useState, useEffect, useMemo } from "react"
import { EventSource } from "eventsource";

export const GameView = () => {
  const [location, setLocation] = useState<Location | undefined>(undefined)
  const [loading, setLoading] = useState<boolean>(false); // used for loading state, will add an animation or loading animation
  const [hasGuessed, setHasGuessed] = useState<boolean>(false);

  const { isLoaded } = useLoadScript({ googleMapsApiKey: process.env.NEXT_PUBLIC_GOOGLE_MAPS_API_KEY || "" });

  // const fetch_location = useCallback(async () => {
  //   setLoading(true);
  //   try {
  //     const location_response = await fetch("/api/random-location");
  //     if (!location_response.ok) throw new Error("Failed to fetch");
  //
  //     const location_json: Location = await location_response.json();
  //     setLocation(location_json);
  //
  //   } catch (error) {
  //     console.log(error);
  //   } finally {
  //     setLoading(false);
  //   };
  //
  // }, [])

  useEffect(() => {
    if (isLoaded) {
      const es = new EventSource('/api/random-location');

      es.addEventListener('update', (event) => {
        const location_data = JSON.parse(event.data);
        setLocation(location_data);
      })
      return () => {
        es.close();
      };
    };


  }, [isLoaded]);

  if (!isLoaded) return <div className="h-screen w-screen bg-black text-white flex items-center justify-center">Loading UwU...</div>;

  return (
    <main className="relative h-screen w-screen overflow-hidden">
      <div className="absolute inset-0 z-0">
        <Streetview location={location} />
      </div>

      <div className={`bottom-5 left-5 flex flex-col gap-2 absolute z-10 aspect-video transition-all duration-300 origin-bottom-left ${hasGuessed
        ? " w-[calc(100vw-2.5rem)] max-h-[calc(100vh-2.5rem)] opacity-100 ease-out"
        : "w-40 md:w-64 lg:w-90 opacity-90 hover:w-[50vw] xl:hover:w-[40vw] ease-in-out"
        }`}>
        <MapOverlay location={location} hasGuessed={hasGuessed} setHasGuessed={setHasGuessed} />
      </div>

    </main>

  );
};
