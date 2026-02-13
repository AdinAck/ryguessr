"use client"
import { Suspense } from "react"
import Streetview from "./ui/sv_map/google-map"
export default function Landing() {
  return (
    <main className="m-0 p-0">
      <Suspense>
        <Streetview></Streetview>
      </Suspense>
    </main>
  )
}
