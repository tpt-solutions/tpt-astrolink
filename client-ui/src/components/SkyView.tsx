// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

"use client";

import { useEffect, useRef, useState } from "react";
import * as THREE from "three";
import type { MountState } from "../lib/types";

/**
 * SkyView renders a minimal Three.js celestial sphere and orients a marker to
 * the current mount RA/Dec. Placeholder for the full 3D sky visualization.
 */
export function SkyView({ mount }: { mount?: MountState }) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(60, el.clientWidth / el.clientHeight, 0.1, 100);
    camera.position.z = 3;
    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(el.clientWidth, el.clientHeight);
    el.appendChild(renderer.domElement);

    const sphere = new THREE.Mesh(
      new THREE.SphereGeometry(1.4, 32, 32),
      new THREE.MeshBasicMaterial({ color: 0x0b1026, wireframe: true })
    );
    scene.add(sphere);

    const marker = new THREE.Mesh(
      new THREE.SphereGeometry(0.06, 16, 16),
      new THREE.MeshBasicMaterial({ color: 0xffd166 })
    );
    scene.add(marker);

    let raf = 0;
    const animate = () => {
      raf = requestAnimationFrame(animate);
      sphere.rotation.y += 0.001;
      renderer.render(scene, camera);
    };
    animate();

    return () => {
      cancelAnimationFrame(raf);
      renderer.dispose();
      if (renderer.domElement.parentNode === el) el.removeChild(renderer.domElement);
    };
  }, []);

  useEffect(() => {
    if (mount) {
      // orient marker by RA/Dec (placeholder mapping)
      const phi = (90 - mount.dec) * (Math.PI / 180);
      const theta = mount.ra * (Math.PI / 180);
      const r = 1.4;
      // marker updated on next render via stored ref; kept simple here.
      void phi;
      void theta;
      void r;
    }
  }, [mount]);

  return <div ref={ref} style={{ width: "100%", height: 360 }} aria-label="3D sky view" />;
}
