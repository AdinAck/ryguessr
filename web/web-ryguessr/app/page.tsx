"use client"
import { Suspense } from "react"
import Google_Map from "./ui/sv_map/google-map"
export default function Landing() {
  return (
    <main className="m-0 p-0">
      <Suspense>
        <Google_Map></Google_Map>
      </Suspense>
    </main>
  )
}
