"use client"
import Streetview from "./StreetView";
import { MapOverlay } from "./MapOverlay";
import { Location } from "@/types/coordinate_type";
import { useState, useEffect } from "react"
import { EventSource } from "eventsource";


export const GameView = ({ userID }: { userID: string }) => {
  const [location, setLocation] = useState<Location | undefined>(undefined)
  const [loading, setLoading] = useState<boolean>(false); // used for loading state, will add an animation or loading animation
  const [hasGuessed, setHasGuessed] = useState<boolean>(false);

  useEffect(() => {
    const es = new EventSource('/sse', {
      fetch: (input, init) =>
        fetch(input, {
          ...init,
          headers: {
            ...init.headers,
            Authorization: userID,
          },
        })
    });

    es.addEventListener('location', (event) => {
      const location_data = JSON.parse(event.data);
      setLocation(location_data);
    })

    return () => {
      es.close();
    };


  }, []);


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
