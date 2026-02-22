"use client"
import { useState, useCallback, useRef, useMemo } from 'react';
import { GoogleMap, MarkerF } from '@react-google-maps/api';
import { Button } from '../ui/button';

// Location type
import { MapOverlayProps } from '@/types/map_overlay_props';

export const MapOverlay = ({ location, setHasGuessed, hasGuessed, }: MapOverlayProps) => {
  const [selectedLocation, setSelectedLocation] = useState<google.maps.LatLngLiteral | null>(null);

  const { defaultPosition, locationIconOptions, userIconOptions, mapContainerStyle } = useMemo(() => {
    return (
      {
        defaultPosition: { lat: 35.6586, lng: 139.7454 },
        locationIconOptions: { width: 30, height: 30, url: "/svg/flag.svg", anchor: new google.maps.Point(15, 30) },
        userIconOptions: { width: 30, height: 30, url: "/svg/user-map-pin.svg", anchor: new google.maps.Point(15, 30) },
        mapContainerStyle: {width: "100%", height: "100%" },
      }
    );

  }, []) 

  const mapRef = useRef<google.maps.Map | null>(null);

  const mapOptions = useMemo(() => {
    return { draggableCursor: hasGuessed ? "move" : "crosshair", disableDefaultUI: true, clickableIcons: false, gestureHandling: "greedy", draggingCursor: "move", minZoom: 2 }
  }, [hasGuessed]);


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

  const handleMapPanOnGuess = () => {
    setHasGuessed(true);
    if (location && selectedLocation && mapRef.current) {
      const bounds = new window.google.maps.LatLngBounds();
      bounds.extend(location);
      bounds.extend(selectedLocation);
      mapRef.current.fitBounds(bounds, 0);
    };

  }

  return (
    <div className="flex gap-2 flex-col gap-2 h-full w-full">
      <div className='flex-grow w-full rounded-lg overflow-hidden border-2 shadow-xl relative z-0'>

        <GoogleMap
          mapContainerStyle={mapContainerStyle}
          onLoad={onLoad}
          options={mapOptions}
          onClick={!hasGuessed ? onMapClick : undefined}

        >
          {selectedLocation && (<MarkerF
            position={selectedLocation}
            icon={userIconOptions}

          />)}
          {hasGuessed && selectedLocation && (<MarkerF icon={locationIconOptions} 
            position={location
            ? location
            : defaultPosition} />)}
        </GoogleMap>
      </div>
      <div className='w-full z-10'>
        <Button onClick={hasGuessed ? undefined : handleMapPanOnGuess} className={"w-full tracking-widest shwdow-lg transition-transform duration-150 ease-in-out active:scale-95"} size={"default"} variant={"default"}> {hasGuessed ? "Continue" : "Guess"} </Button>
      </div>
    </div>
  );
}
