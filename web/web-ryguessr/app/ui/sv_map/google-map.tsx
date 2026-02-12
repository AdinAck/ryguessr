import React, { useEffect } from 'react';
import { APIProvider, Map } from '@vis.gl/react-google-maps';
const Google_Map = () => {

  const apiKey = process.env.NEXT_PUBLIC_GOOGLE_MAPS_API_KEY;
  const streetview_url = `https://www.google.com/maps/embed/v1/streetview?key=${apiKey}&location=46.414382,10.013988&heading=210&pitch=10&fov=35`

  if (!apiKey) {
    return <div>Error: Google Maps API key is missing.</div>;
  }
  useEffect(() => {
    console.log("hello world")
    console.log(streetview_url);
  }, [])

  return (
    <iframe
      src={streetview_url}
      className="fixed inset-0 w-screen h-screen border-none z-50"
      referrerPolicy="no-referrer-when-downgrade"
    />
    // <APIProvider apiKey={apiKey}>
    //   <Map
    //     style={{ width: '100vw', height: '100vh' }}
    //     defaultCenter={{ lat: 22.54992, lng: 0 }}
    //     defaultZoom={3}
    //     gestureHandling='greedy'
    //     disableDefaultUI
    //   />
    // </APIProvider>
  );
};

export default Google_Map;
