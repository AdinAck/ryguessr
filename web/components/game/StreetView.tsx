"use client"
import { GoogleMap, StreetViewPanorama, useLoadScript } from "@react-google-maps/api";
import { Location } from "@/types/coordinate_type";
import { memo } from "react"

// DefaultPosition static  
const defaultPosition = { lat: 35.6586, lng: 139.7454 };

const Streetview = memo(({ location }: { location?: Location }) => {




  return (
    <div className="h-full w-full">
      <GoogleMap
        zoom={10}
        mapContainerStyle={{ width: "100%", height: "100%" }}
      >
        <StreetViewPanorama
          options={{
            visible: true,
            position: location ? location : defaultPosition,
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
});

export default Streetview;
