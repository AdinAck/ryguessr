"use client"
import { CookiesProvider } from "react-cookie";
import { Init } from "../initialize/init";

const CookieWrapper = () => {

  return (
    <CookiesProvider defaultSetOptions={{ path: '/' }}>
      <Init />
    </CookiesProvider>
  );
};

export default CookieWrapper;
