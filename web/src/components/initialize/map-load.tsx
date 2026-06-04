import { useLoadScript } from "@react-google-maps/api";
import { GameView } from "../game/GameView";

const GoogleMapLoader = ({ APIKEY }: { APIKEY: string }) => {
  const { isLoaded, loadError } = useLoadScript({
    googleMapsApiKey: APIKEY,
  });

  if (loadError || !isLoaded) {
    return (
      <div className="h-screen w-screen bg-black text-white flex items-center justify-center">
        Loading UwU...
      </div>
    );
  }
  return <GameView />;
};

export default GoogleMapLoader;
