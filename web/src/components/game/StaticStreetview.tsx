export const StaticStreetview = ({ panoId }: { panoId?: string }) => {
  const imageUrl = `https://maps.googleapis.com/maps/api/streetview?size=960x540&pano=${panoId}&pitch=0&fov=90&key=${process.env.NEXT_PUBLIC_GOOGLE_MAPS_API_KEY}`;

  return (
    <div className="h-full w-full bg-black">
      <img
        src={imageUrl}
        alt="Street location"
        className="w-full h-full object-cover"
        draggable={false}
      />
    </div>
  );
};
