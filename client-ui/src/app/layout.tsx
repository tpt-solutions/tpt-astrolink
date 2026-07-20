// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "TPT AstroLink",
  description: "Project Cosmos — remote observatory control dashboard.",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
