"use client"
import { GoogleMap, StreetViewPanorama, useLoadScript } from "@react-google-maps/api";
import { memo } from "react"

// DefaultPosition static  
const defaultPosition = { lat: 35.6586, lng: 139.7454 };

const Streetview = memo(({ panoId }: { panoId?: string }) => {

  // console.log(panoId);


  return (
    <div className="h-full w-full">
      <GoogleMap
        zoom={10}
        mapContainerStyle={{ width: "100%", height: "100%" }}
      >
        <StreetViewPanorama
          options={{
            visible: true,
            pano: panoId,
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
