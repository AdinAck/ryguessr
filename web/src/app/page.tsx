import CookieWrapper from "@/components/cookie/cookie-wrapper";
import { Suspense } from "react";

export default function Landing() {
  return (
    <main className="m-0 p-0">
      <Suspense>
        <CookieWrapper />
      </Suspense>
    </main>
  );
}

// location
// continuing
// score
// distance
// real-coordinates
