"use client"
import { useState, useCallback, useRef } from 'react';
import { GoogleMap, MarkerF } from '@react-google-maps/api';
import { Button } from '../ui/button';

export const MapOverlay = () => {
  const [selectedLocation, setSelectedLocation] = useState<google.maps.LatLngLiteral | null>(null);

  const mapRef = useRef<google.maps.Map | null>(null);

  const mapOptions = {
    disableDefaultUI: true, clickableIcons: false, gestureHandling: "greedy", draggableCursor: "crosshair", draggingCursor: "move"
  };

  const mapContainerStyle = { width: "100%", height: "100%" };

  const [hasGuessed, setHasGuessed] = useState<boolean>(false);

  const onMapClick = useCallback((e: google.maps.MapMouseEvent) => {
    if (e.latLng) {
      setSelectedLocation({ lat: e.latLng.lat(), lng: e.latLng.lng() })
    };
  }, []);

  const onLoad = useCallback((map: google.maps.Map) => {
    mapRef.current = map;

    map.setCenter({ lat: 20, lng: 0 });
    map.setZoom(2);
  }, []);

  return (
    <div className="flex gap-2 flex-col gap-2 h-full w-full">
      <div className='flex-grow w-full rounded-lg overflow-hidden border-2 shadow-xl relative z-0'>

        <GoogleMap
          mapContainerStyle={mapContainerStyle}
          onLoad={onLoad}
          options={mapOptions}
          onClick={onMapClick}
        >
          {selectedLocation && (<MarkerF
            position={selectedLocation}

          />)}
        </GoogleMap>
      </div>
      <div className='w-full z-10'>
        <Button onClick={() => setHasGuessed(true)} className={"w-full tracking-widest shwdow-lg transition-transform duration-150 ease-in-out active:scale-95"} size={"default"} variant={"default"}> Guess </Button>
      </div>
    </div>
  );
}
