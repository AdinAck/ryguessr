"use client"
import { GoogleMap, StreetViewPanorama, useLoadScript } from "@react-google-maps/api";
import { coordinates } from "@/types/coordinate_type";

const Streetview = ({ initialLocation }: { initialLocation?: coordinates }) => {
  const { isLoaded } = useLoadScript({ googleMapsApiKey: process.env.NEXT_PUBLIC_GOOGLE_MAPS_API_KEY || "" });

  if (!isLoaded) {
    return <div>Loading.... UwU</div>;
  }

  const defaultPosition = { lat: 35.6586, lng: 139.7454 };



  return (
    <div className="h-full w-full">
      <GoogleMap
        zoom={10}
        mapContainerStyle={{ width: "100%", height: "100%" }}
      >
        <StreetViewPanorama
          options={{
            visible: true,
            position: initialLocation ? initialLocation : defaultPosition,
            addressControl: false,
            enableCloseButton: false,
            fullscreenControl: false,
            panControl: false,
            zoomControl: false
          }}
        />
      </GoogleMap>
    </div>
  );
};

export default Streetview;
