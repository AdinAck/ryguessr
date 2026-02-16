"use client"
import { APIProvider, Map, MapCameraChangedEvent } from '@vis.gl/react-google-maps';
import { GoogleMap } from '@react-google-maps/api';

export const MapOverlay = () => {
  return (
    <div className="h-64 w-full rounded-lg overflow-hidden border-2 border-white shadow-xl">
      <GoogleMap
        mapContainerStyle={{ width: "100%", height: "100%" }}
        center={{ lat: 20, lng: 0 }}
        zoom={2}
        options={{ disableDefaultUI: true, clickableIcons: false, gestureHandling: "greedy" }} />
    </div>
  );
}
