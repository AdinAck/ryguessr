"use client"
import React, { useEffect } from 'react';
import { APIProvider, Map } from '@vis.gl/react-google-maps';
import { GoogleMap, StreetViewPanorama, useLoadScript } from "@react-google-maps/api";
const Streetview = () => {
  const { isLoaded } = useLoadScript({ googleMapsApiKey: process.env.NEXT_PUBLIC_GOOGLE_MAPS_API_KEY || "" });

  if (!isLoaded) {
    return <div>Loading.... UwU</div>;
  }

  const defaultPosition = { lat: 35.6586, lng: 139.7454 };

  return (
    <GoogleMap
      zoom={10}
      mapContainerStyle={{ width: "100vw", height: "100vh" }}
    >
      <StreetViewPanorama
        options={{
          visible: true,
          position: defaultPosition,
          addressControl: false,
          enableCloseButton: false,
          fullscreenControl: false,
          panControl: false,
          zoomControl: false
        }}
      />
    </GoogleMap>
  );
};

export default Streetview;
