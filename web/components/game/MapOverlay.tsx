"use client"
import { useState, useCallback, useRef } from 'react';
import { GoogleMap, MarkerF } from '@react-google-maps/api';

export const MapOverlay = () => {
  const [selectedLocation, setSelectedLocation] = useState<google.maps.LatLngLiteral | null>(null);

  const mapRef = useRef<google.maps.Map | null>(null);

  const mapOptions = {
    disableDefaultUI: true, clickableIcons: false, gestureHandling: "greedy", draggableCursor: "crosshair", draggingCursor: "move"
  };

  const mapContainerStyle = { width: "100%", height: "100%" };

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
    <div className="h-full w-full rounded-lg overflow-hidden border-2 border-white shadow-xl cursor-crosshair">
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
  );
}
