#!/usr/bin/env python3
"""Author the original ACORN/Risk miniature recipes. Standard library only. MIT."""
import argparse
import importlib.util
import json
import math
import sys
from pathlib import Path

SPEC = importlib.util.spec_from_file_location("geometry", Path(__file__).with_name("model-geometry.py"))
G = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(G)
PI = math.pi
PALETTE_SPEC = importlib.util.spec_from_file_location("palette", Path(__file__).with_name("model-palette.py"))
BRAND = importlib.util.module_from_spec(PALETTE_SPEC)
PALETTE_SPEC.loader.exec_module(BRAND)
DETAIL_SPEC = importlib.util.spec_from_file_location("detail", Path(__file__).with_name("model-detail.py"))
DETAIL = importlib.util.module_from_spec(DETAIL_SPEC)
DETAIL_SPEC.loader.exec_module(DETAIL)
OTHER_SPEC = importlib.util.spec_from_file_location("other", Path(__file__).with_name("model-other.py"))
OTHER = importlib.util.module_from_spec(OTHER_SPEC)
OTHER_SPEC.loader.exec_module(OTHER)
LANDING_PAD_SPEC = importlib.util.spec_from_file_location("landing_pad", Path(__file__).with_name("model-landing-pad.py"))
LANDING_PAD = importlib.util.module_from_spec(LANDING_PAD_SPEC)
LANDING_PAD_SPEC.loader.exec_module(LANDING_PAD)


def part(mesh, material="ivory", center=(0, 0, 0), scale=(1, 1, 1), rotation=(0, 0, 0), smooth=False, group="body"):
    vertices, faces = G.transform(mesh, center, scale, rotation)
    return dict(shape="mesh", vertices=vertices, triangles=faces, material=material, group=group, smooth=smooth)


def B(center, size, material="ivory", rotation=(0, 0, 0)):
    return part(G.box(*size), material, center, rotation=rotation)


def C(center, radius, height, material="ivory", sides=32):
    b = min(height * 0.2, radius * 0.05)
    return part(G.lathe([(-height / 2, radius - b), (-height / 2 + b, radius), (height / 2 - b, radius), (height / 2, radius - b)], sides), material, center)


def E(points, depth, material="ivory", center=(0, 0, 0), rotation=(0, 0, 0)):
    return part(G.extrude(points, depth), material, center, rotation=rotation)


def L(center, profile, material="ivory", sides=32, scale=(1, 1, 1)):
    return part(G.lathe(profile, sides), material, center, scale=scale, smooth=True)


def P(center, size, material="ivory", top=0):
    return part(G.pyramid(*size, top), material, center)


def R(a, b, radius, material="ivory", end_radius=None):
    return part(G.beam(a, b, radius, end_radius=end_radius), material, smooth=True)


def S(center, scale, material="ivory", smooth=True):
    return part(G.sphere(sides=12 if max(scale) < 0.12 else 32, rings=8 if max(scale) < 0.12 else 20), material, center, scale, smooth=smooth)


def T(center, radius, tube, material="ivory", rotation=(0, 0, 0)):
    return part(G.torus(radius, tube), material, center, rotation=rotation, smooth=True)


def research_wing(center, x_radius, z_radius, floors):
    x, y, z = center
    top = y + 0.18 + floors * 0.42
    return [L((x, y, z), [(0, 1.02), (0.12, 1.06), (0.18, 1.02)], "white", 64, (x_radius, 1, z_radius)),
        *[L((x, y + 0.18 + level * 0.42, z), [(0, 1), (0.34, 1)], "water", 64, (x_radius, 1, z_radius)) for level in range(floors)],
        *[L((x, y + 0.52 + level * 0.42, z), [(0, 1.04), (0.03, 1.08), (0.08, 1.04)], "white", 64, (x_radius, 1, z_radius)) for level in range(floors)],
        *[part(G.torus(1.055, 0.012, sides=64), "light", (x, y + 0.56 + level * 0.42, z), (x_radius, 1, z_radius), smooth=True) for level in range(floors)],
        *[R((x + x_radius * math.cos(a), y + 0.18, z + z_radius * math.sin(a)),
            (x + x_radius * math.cos(a), top - 0.08, z + z_radius * math.sin(a)), 0.014, "steel") for a in [i * PI / 14 for i in range(28)]],
        part(G.sphere(sides=64, rings=16), "white", (x, top + 0.035, z), (x_radius * 1.17, 0.12, z_radius * 1.17), rotation=(0.05, 0, -0.055), smooth=True),
        *[B((x + dx * x_radius, top + 0.16, z - z_radius * 0.17), (0.18 * x_radius, 0.015, z_radius * 0.60), "teal", (0.05, 0, -0.055)) for dx in [-0.60, -0.30, 0, 0.30, 0.60]]]


def laboratory():
    building = research_wing((-0.35, 0, -0.20), 1.23, 0.70, 2)
    atrium = [C((0.70, 0.16, 0.63), 0.59, 0.18, "white", 64),
        L((0.70, 0.24, 0.63), [(0, 0.51), (0.69, 0.51), (0.85, 0.45), (0.95, 0.31), (1.0, 0)], "glass", 64),
        T((0.70, 0.27, 0.63), 0.52, 0.025, "light"),
        *[R((0.70 + 0.52 * math.cos(a), 0.24, 0.63 + 0.52 * math.sin(a)),
            (0.70 + 0.46 * math.cos(a), 1.07, 0.63 + 0.46 * math.sin(a)), 0.015, "white") for a in [i * PI / 4 for i in range(8)]],
        part(G.torus(0.61, 0.045, sides=64), "white", (0.70, 1.10, 0.63), (1.12, 0.75, 1), rotation=(0, 0, -0.13), smooth=True),
        part(G.torus(0.63, 0.012, sides=64), "light", (0.70, 1.10, 0.63), (1.12, 0.75, 1), rotation=(0, 0, -0.13), smooth=True)]
    instrument = [C((0.70, 0.42, 0.63), 0.23, 0.28, "steel"), T((0.70, 0.75, 0.63), 0.25, 0.045, "white", (PI / 2, 0, 0)),
        T((0.70, 0.75, 0.63), 0.22, 0.018, "light", (0.4, 0.2, 0.6)),
        S((0.70, 0.75, 0.63), (0.075, 0.075, 0.075), "light"),
        B((0.32, 0.50, 0.75), (0.17, 0.22, 0.05), "dark"), B((0.32, 0.51, 0.78), (0.13, 0.15, 0.012), "light")]
    entry = [B((0.70, 0.11, 1.39), (0.64, 0.10, 0.68), "ivory"),
        *[B((0.70, 0.035 + i * 0.035, 1.79 - i * 0.09), (0.76, 0.07, 0.20), "white") for i in range(3)],
        *[B((x, 0.55, 1.15), (0.025, 0.60, 0.035), "white") for x in [0.51, 0.70, 0.89]],
        B((0.70, 0.86, 1.15), (0.43, 0.035, 0.04), "white"),
        *[R((x, 0.18, 1.14), (x, 0.18, 1.66), 0.012, "light") for x in [0.40, 1.0]],
        C((-0.83, 0.12, 0.84), 0.28, 0.15, "ivory"), C((-0.83, 0.21, 0.84), 0.24, 0.04, "teal"),
        *[S((-0.83 + dx, 0.28, 0.84 + dz), (0.08, 0.10, 0.08), "teal") for dx,dz in [(-0.1,0),(0.07,-0.08),(0.07,0.09)]]]
    return building + atrium + instrument + entry


def doe_national_laboratory():
    campus = research_wing((0, 0, -0.72), 0.67, 0.61, 5) + [
        p for side in [-1, 1] for p in research_wing((side * 1.12, 0, 0.12), 0.69, 0.78, 3)]
    bridges = [p for side in [-1, 1] for p in [
        B((side * 0.60, 1.14, -0.37), (0.78, 0.33, 0.30), "glass", (0, side * 0.44, 0)),
        B((side * 0.60, 0.95, -0.37), (0.85, 0.055, 0.36), "white", (0, side * 0.44, 0)),
        B((side * 0.60, 1.34, -0.37), (0.85, 0.055, 0.36), "white", (0, side * 0.44, 0)),
        R((side * 0.31, 0.98, -0.35), (side * 0.91, 0.98, -0.07), 0.014, "light")]]
    atrium = [C((0, 0.16, 0.83), 0.66, 0.20, "white", 64),
        L((0, 0.26, 0.83), [(0, 0.58), (0.72, 0.58), (1.0, 0.50), (1.18, 0.34), (1.24, 0)], "glass", 64),
        T((0, 0.30, 0.83), 0.59, 0.027, "light"),
        *[R((0.59 * math.cos(a), 0.27, 0.83 + 0.59 * math.sin(a)),
            (0.51 * math.cos(a + 0.18), 1.26, 0.83 + 0.51 * math.sin(a + 0.18)), 0.018, "white") for a in [i * PI / 5 for i in range(10)]],
        part(G.torus(0.65, 0.045, sides=64), "white", (0, 1.29, 0.83), (1.15, 1, 1), rotation=(0, 0, -0.18), smooth=True),
        part(G.torus(0.67, 0.012, sides=64), "light", (0, 1.29, 0.83), (1.15, 1, 1), rotation=(0, 0, -0.18), smooth=True),
        C((0, 0.48, 0.83), 0.27, 0.28, "steel"),
        *[T((0, 0.92, 0.83), 0.28, 0.025, "light", rotation) for rotation in [(PI / 2, 0.35, 0), (0.4, 0, 0.6)]],
        part(G.dodecahedron(0.12), "white", (0, 0.92, 0.83), rotation=(0.2, 0.3, 0.1))]
    crown = [T((0, 2.46, -0.72), 0.46, 0.035, "light"),
        L((0, 2.43, -0.72), [(0, 0.25), (0.15, 0.21), (0.28, 0.10), (0.31, 0)], "glass", 48),
        *[R((0.35 * math.cos(a), 2.44, -0.72 + 0.35 * math.sin(a)), (0, 2.79, -0.72), 0.017, "white") for a in [i * PI / 2 for i in range(4)]]]
    plaza = [B((0, 0.12, 1.64), (0.75, 0.13, 0.95), "ivory"),
        *[B((0, 0.035 + i * 0.04, 2.16 - i * 0.10), (0.91, 0.08, 0.22), "white") for i in range(3)],
        *[R((x, 0.22, 1.35), (x, 0.22, 2.00), 0.014, "light") for x in [-0.35, 0.35]],
        *[B((x, 0.62, 1.41), (0.028, 0.72, 0.04), "white") for x in [-0.22, 0, 0.22]],
        B((0, 0.99, 1.41), (0.49, 0.035, 0.04), "white"),
        *[p for x in [-1.0, 1.0] for p in [C((x, 0.14, 1.22), 0.28, 0.18, "ivory"), C((x, 0.24, 1.22), 0.24, 0.04, "teal"),
            *[S((x + dx, 0.31, 1.22 + dz), (0.08, 0.09, 0.08), "teal") for dx,dz in [(-0.1,0),(0.07,-0.08),(0.07,0.09)]]]]]
    return campus + bridges + atrium + crown + plaza


def user_facility():
    def ribbon(inner, outer, bottom, top, start, end, material, wave=0, steps=72):
        rings = [[(radius * math.cos(a), height + wave * math.sin(a + 0.4), radius * math.sin(a) - 0.10)
                  for radius, height in [(inner, bottom), (inner, top), (outer, top), (outer, bottom)]]
                 for a in [start + (end - start) * i / steps for i in range(steps + 1)]]
        return part(G.ring_mesh(rings), material, smooth=True)
    wings = [ribbon(0.98, 1.78, 0.10, 0.22, 0.78 * PI, 2.22 * PI, "ivory"),
        *[ribbon(1.02, 1.69, 0.24 + level * 0.42, 0.60 + level * 0.42, 0.78 * PI, 2.22 * PI, "water") for level in range(3)],
        *[ribbon(0.97, 1.78, 0.60 + level * 0.42, 0.68 + level * 0.42, 0.76 * PI, 2.24 * PI, "white") for level in range(3)],
        ribbon(0.91, 1.87, 1.51, 1.60, 0.73 * PI, 2.27 * PI, "white", wave=0.13),
        ribbon(1.77, 1.80, 0.62, 0.64, 0.76 * PI, 2.24 * PI, "light"),
        ribbon(1.77, 1.80, 1.04, 1.06, 0.76 * PI, 2.24 * PI, "light"),
        *[R((1.70 * math.cos(a), 0.23, 1.70 * math.sin(a) - 0.10),
            (1.70 * math.cos(a), 1.47, 1.70 * math.sin(a) - 0.10), 0.018, "steel")
          for a in [0.79 * PI + i * 1.42 * PI / 32 for i in range(33)]],
        *[ribbon(1.15, 1.55, 1.68, 1.70, start, start + 0.16, "teal", wave=0.13, steps=8)
          for start in [1.04 * PI + i * 0.20 for i in range(14)]]]
    atrium = [C((0, 0.18, 0.08), 0.90, 0.20, "white", 64),
        C((0, 0.31, 0.08), 0.81, 0.08, "dark", 64),
        L((0, 0.35, 0.08), [(0, 0.76), (0.75, 0.76), (1.08, 0.72), (1.33, 0.61), (1.49, 0.43), (1.59, 0.20), (1.62, 0)], "glass", 64),
        T((0, 0.38, 0.08), 0.77, 0.035, "light"), T((0, 1.41, 0.08), 0.73, 0.035, "white"),
        *[p for a in [i * PI / 5 for i in range(10)] for p in [
            R((0.77 * math.cos(a), 0.36, 0.08 + 0.77 * math.sin(a)),
              (0.73 * math.cos(a + 0.20), 1.40, 0.08 + 0.73 * math.sin(a + 0.20)), 0.018, "white"),
            *[R((r1 * math.cos(a + 0.20), y1, 0.08 + r1 * math.sin(a + 0.20)),
                (r2 * math.cos(a + 0.20), y2, 0.08 + r2 * math.sin(a + 0.20)), 0.015, "white")
              for (r1, y1), (r2, y2) in zip([(0.73, 1.40), (0.61, 1.68), (0.43, 1.84), (0.20, 1.94)],
                                           [(0.61, 1.68), (0.43, 1.84), (0.20, 1.94), (0, 1.97)])]]],
        ribbon(0.87, 1.10, 1.69, 1.77, 0, 2 * PI, "white", wave=0.23),
        ribbon(1.095, 1.12, 1.70, 1.73, 0, 2 * PI, "light", wave=0.23)]
    instrument = [C((0, 0.48, 0.08), 0.34, 0.24, "steel"), C((0, 0.63, 0.08), 0.40, 0.07, "dark"),
        T((0, 1.00, 0.08), 0.36, 0.047, "white", (PI / 2, 0.30, 0)),
        T((0, 1.00, 0.08), 0.31, 0.024, "light", (0.40, 0, 0.65)),
        part(G.dodecahedron(0.14), "light", (0, 1.00, 0.08), rotation=(0.2, 0.3, 0.1)),
        *[R((x, 0.65, 0.08), (x, 0.94, 0.08), 0.022, "steel") for x in [-0.32, 0.32]]]
    entry = [B((0, 0.13, 1.34), (0.80, 0.12, 1.10), "ivory"),
        *[B((0, 0.04 + i * 0.04, 1.92 - i * 0.12), (0.91, 0.08, 0.25), "white") for i in range(3)],
        B((0, 0.71, 0.85), (0.49, 0.73, 0.035), "glass"),
        *[B((x, 0.71, 0.89), (0.035, 0.79, 0.04), "white") for x in [-0.27, 0, 0.27]],
        B((0, 1.12, 0.89), (0.57, 0.045, 0.055), "white"),
        *[R((x, 0.58, 0.93), (x, 0.80, 0.93), 0.011, "light") for x in [-0.05, 0.05]],
        *[R((x, 0.22, 0.94), (x, 0.22, 1.79), 0.015, "light") for x in [-0.37, 0.37]]]
    landscaping = [p for x in [-1.15, 1.15] for p in [
        C((x, 0.13, 1.08), 0.31, 0.15, "ivory"), C((x, 0.21, 1.08), 0.27, 0.04, "teal"),
        *[S((x + dx, 0.27, 1.08 + dz), (0.09, 0.08, 0.09), "teal") for dx, dz in [(-0.13, 0), (0.09, -0.1), (0.06, 0.12)]]]]
    supports = [0.85 * PI, 2.15 * PI] + [1.04 * PI + i * 0.20 + 0.18 for i in [1, 4, 7, 10, 13]]
    storage_ring = [part(G.torus(1.33, 0.045, sides=96), "light", (0, 2.05, -0.10), smooth=True),
        *[R((1.33 * math.cos(a), 1.58 + 0.13 * math.sin(a + 0.4), 1.33 * math.sin(a) - 0.10), (1.33 * math.cos(a), 2.03, 1.33 * math.sin(a) - 0.10), 0.022, "steel") for a in supports]]
    return wings + atrium + instrument + entry + landscaping + storage_ring


def research_project():
    bench = [B((0, 0.86, 0.12), (2.75, 0.14, 1.45), "ivory"),
             B((0, 0.76, 0.12), (2.65, 0.10, 1.35), "teal"),
             *[B((x, 0.36, z), (0.10, 0.72, 0.10), "steel") for x in [-1.2, 1.2] for z in [-0.45, 0.67]],
             *[B((x, 0.08, z), (0.14, 0.06, 0.14), "dark") for x in [-1.2, 1.2] for z in [-0.45, 0.67]],
             B((0, 0.30, -0.45), (2.45, 0.08, 0.08), "steel")]
    monitor = [B((0, 1.70, -0.46), (2.24, 1.20, 0.12), "dark"),
               B((0, 1.72, -0.393), (2.09, 1.04, 0.018), "pages"),
               B((0, 1.09, -0.48), (0.13, 0.34, 0.13), "steel"),
               B((0, 0.96, -0.40), (0.66, 0.06, 0.40), "steel"),
               S((0, 2.265, -0.39), (0.025, 0.025, 0.012), "water"),
               S((0.96, 1.15, -0.39), (0.015, 0.015, 0.01), "teal"),
               *[B((x, 1.85, -0.526), (0.022, 0.30, 0.015), "steel") for x in [-0.35 + i * 0.07 for i in range(11)]],
               *[B((x, 1.23, -0.53), (0.10, 0.055, 0.02), "steel") for x in [-0.45, -0.22, 0.22, 0.45]],
               R((0, 1.15, -0.54), (0, 0.98, -0.63), 0.02, "dark")]
    dashboard = [B((0, 2.18, -0.375), (2.06, 0.10, 0.018), "teal"),
                 *[S((x, 2.18, -0.36), (0.018, 0.018, 0.008), "pages") for x in [-0.95, -0.88, -0.81]],
                 *[B((x, y, -0.376), (0.86, 0.38, 0.012), "ivory") for x in [-0.51, 0.51] for y in [1.87, 1.40]],
                 *[B((-0.51, y, -0.365), (0.77, 0.008, 0.006), "steel") for y in [1.74, 1.84, 1.94, 2.02]],
                 R((-0.90, 1.71, -0.355), (-0.90, 2.04, -0.355), 0.008, "dark"),
                 R((-0.90, 1.71, -0.355), (-0.13, 1.71, -0.355), 0.008, "dark")]
    points = [(-0.86, 1.77, -0.35), (-0.73, 1.84, -0.35), (-0.60, 1.81, -0.35), (-0.47, 1.94, -0.35), (-0.32, 1.89, -0.35), (-0.17, 2.01, -0.35)]
    dashboard += [R(a, b, 0.015, "teal") for a, b in zip(points, points[1:])] + [S(point, (0.025, 0.025, 0.009), "teal") for point in points]
    dashboard += [B((0.23 + i * 0.14, 1.25 + height / 2, -0.35), (0.085, height, 0.018), "water" if i % 2 else "teal") for i, height in enumerate([0.10, 0.18, 0.13, 0.25, 0.29])]
    dashboard += [R((0.16, 1.23, -0.35), (0.88, 1.23, -0.35), 0.008, "dark")]
    dashboard += [R((0.51 + 0.13 * math.cos(i * PI / 16), 1.87 + 0.13 * math.sin(i * PI / 16), -0.35),
                    (0.51 + 0.13 * math.cos((i + 1) * PI / 16), 1.87 + 0.13 * math.sin((i + 1) * PI / 16), -0.35), 0.035, "teal" if i < 19 else "water" if i < 27 else "amber") for i in range(32)]
    nodes = [(-0.81, 1.40, -0.35), (-0.60, 1.52, -0.35), (-0.51, 1.29, -0.35), (-0.29, 1.44, -0.35)]
    dashboard += [R(nodes[a], nodes[b], 0.008, "steel") for a, b in [(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)]] + [S(point, (0.043, 0.043, 0.012), "water" if i % 2 else "teal") for i, point in enumerate(nodes)]
    keyboard = [B((-0.59, 0.97, 0.38), (1.10, 0.075, 0.36), "dark"),
                *[B((-1.04 + col * 0.09, 1.018, 0.27 + row * 0.075), (0.065, 0.026, 0.052), "ivory") for row in range(3) for col in range(11)],
                B((-0.59, 1.018, 0.505), (0.40, 0.026, 0.035), "ivory"),
                B((0.13, 0.94, 0.37), (0.26, 0.015, 0.36), "water"),
                S((0.13, 0.98, 0.37), (0.09, 0.055, 0.13), "ivory"),
                R((0.13, 1.035, 0.31), (0.13, 1.035, 0.40), 0.006, "dark")]
    notebook = [B((0.72, 0.96, 0.40), (0.90, 0.04, 0.66), "teal"),
                *[B((x, 0.99, 0.40), (0.39, 0.035, 0.58), "pages") for x in [0.49, 0.95]],
                *[B((x, 1.012, z), (0.28, 0.008, 0.013), "steel") for x in [0.49, 0.95] for z in [0.20, 0.29, 0.38, 0.47, 0.56]],
                R((0.72, 1.012, 0.10), (0.72, 1.012, 0.69), 0.015, "dark"),
                R((1.25, 0.96, 0.15), (1.25, 0.96, 0.64), 0.018, "teal"),
                R((1.25, 0.96, 0.64), (1.25, 0.96, 0.69), 0.018, "steel", 0.002)]
    papers = [B((-0.91 + i * 0.017, 0.94 + i * 0.009, -0.12 + i * 0.013), (0.55, 0.008, 0.45), "pages", (0, i * 0.04, 0)) for i in range(4)] + [
              B((-0.86, 0.979, -0.20 + i * 0.07), (0.32 if i else 0.39, 0.006, 0.012), "teal" if i == 0 else "steel") for i in range(4)] + [
              B((-0.98, 0.99, -0.30), (0.10, 0.016, 0.025), "steel")]
    accessories = [L((1.07, 0.94, -0.15), [(0, 0.10), (0.22, 0.11)], "teal"),
                   T((1.07, 1.16, -0.15), 0.106, 0.012, "ivory"),
                   C((1.07, 1.15, -0.15), 0.095, 0.012, "dark"),
                   T((1.20, 1.05, -0.15), 0.065, 0.018, "teal", (PI / 2, 0, 0)),
                   B((0.91, 0.40, -0.04), (0.36, 0.65, 0.55), "dark"),
                   B((0.91, 0.40, 0.242), (0.31, 0.60, 0.025), "steel"),
                   *[B((0.91, 0.22 + i * 0.04, 0.259), (0.23, 0.015, 0.01), "dark") for i in range(7)],
                   B((0.91, 0.63, 0.261), (0.08, 0.025, 0.012), "dark"),
                   S((1.01, 0.64, 0.267), (0.015, 0.015, 0.008), "teal")]
    return bench + monitor + dashboard + keyboard + notebook + papers + accessories


def dataset():
    shells = [p for y in [0.28, 0.86, 1.44] for p in [
        L((0, y - 0.22, 0), [(0, 0.89), (0.035, 0.95), (0.39, 0.95), (0.44, 0.89)], "ivory", 64),
        C((0, y + 0.23, 0), 0.88, 0.035, "dark", 64),
        T((0, y - 0.18, 0), 0.947, 0.012, "steel"),
        T((0, y + 0.17, 0), 0.947, 0.012, "steel"),
        B((0, y, 0.94), (0.61, 0.19, 0.06), "dark"),
        *[B((x, y + 0.025, 0.976), (0.055, 0.07, 0.012), "light") for x in [-0.21, -0.10, 0.01]],
        B((0.18, y + 0.025, 0.976), (0.12, 0.045, 0.012), "blue"),
        *[R((0.93 * math.cos(a), y - 0.10, 0.93 * math.sin(a)), (0.96 * math.cos(a), y + 0.11, 0.96 * math.sin(a)), 0.009, "dark") for a in [PI + i * PI / 18 for i in range(19)]],
        B((0, y, -0.95), (0.50, 0.14, 0.045), "dark"),
        *[B((x, y, -0.977), (0.12, 0.06, 0.015), "steel") for x in [-0.15, 0, 0.15]]]]
    interconnects = [p for y in [0.57, 1.15] for p in [C((0, y, 0), 0.84, 0.14, "teal", 64),
        T((0, y, 0), 0.846, 0.014, "light")]]
    top = [C((0, 1.69, 0), 0.79, 0.025, "teal", 64), T((0, 1.71, 0), 0.65, 0.009, "steel"),
        *[R((x, 1.71, -0.34), (x, 1.71, 0.34), 0.009, "light") for x in [-0.25, 0, 0.25]],
        *[R((-0.34, 1.71, z), (0.34, 1.71, z), 0.009, "light") for z in [-0.25, 0, 0.25]]]
    return shells + interconnects + top


def software():
    chevron = [(-0.18, 0), (0.13, 0.34), (0.22, 0.24), (0, 0), (0.22, -0.24), (0.13, -0.34)]
    frame = [B((0, 1.10, 0), (2.60, 2.05, 0.18), "dark"), B((0, 1.10, 0.10), (2.47, 1.91, 0.035), "water"),
        B((0, 1.96, 0.13), (2.43, 0.14, 0.025), "teal"),
        *[S((x, 1.96, 0.15), (0.027, 0.027, 0.012), "light") for x in [-1.07, -0.96, -0.85]],
        B((0, 0.08, 0), (1.72, 0.16, 0.68), "steel"), B((0, 1.10, -0.105), (2.34, 1.75, 0.03), "ivory"),
        *[B((x, 1.81, -0.13), (0.025, 0.17, 0.018), "dark") for x in [-0.40 + i * 0.08 for i in range(11)]],
        *[B((x, 0.33, -0.13), (0.16, 0.065, 0.025), "dark") for x in [-0.40, -0.15, 0.10, 0.35]],
        S((0, 2.095, 0.10), (0.018, 0.018, 0.01), "steel")]
    code = [E(chevron, 0.025, "white", (-0.90, 1.24, 0.14)), E(chevron, 0.025, "white", (0.02, 1.24, 0.14), (0, 0, PI)),
        B((-0.44, 1.24, 0.14), (0.07, 0.84, 0.025), "light", (0, 0, -0.28)),
        B((0.40, 1.08, 0.14), (0.018, 1.40, 0.025), "steel"),
        *[B((0.75, y, 0.14), (width, 0.035, 0.025), material) for y,width,material in [(1.67,0.46,"light"),(1.48,0.35,"white"),(1.29,0.46,"blue"),(1.10,0.28,"white"),(0.91,0.41,"light")]],
        *[B((-0.47, y, 0.14), (width, 0.025, 0.025), "steel") for y,width in [(0.66,1.05),(0.52,0.76),(0.38,0.91)]],
        B((0.75, 0.53, 0.14), (0.48, 0.21, 0.025), "teal"),
        R((0.64, 0.54, 0.16), (0.72, 0.47, 0.16), 0.015, "white"), R((0.72, 0.47, 0.16), (0.88, 0.61, 0.16), 0.015, "white")]
    return frame + code


def publication():
    return [B((-0.62, 0.20, 0), (1.3, 0.16, 1.8), "dark", (0, 0, -0.16)),
            B((0.62, 0.20, 0), (1.3, 0.16, 1.8), "dark", (0, 0, 0.16)),
            B((-0.62, 0.34, 0), (1.18, 0.20, 1.65), "pages", (0, 0, -0.16)),
            B((0.62, 0.34, 0), (1.18, 0.20, 1.65), "pages", (0, 0, 0.16)),
            R((0, 0.30, -0.88), (0, 0.30, 0.88), 0.10, "dark")] + [
            B((x, 0.49, z), (0.72, 0.028, 0.07), "teal", (0, 0, -0.16 if x < 0 else 0.16)) for x in [-0.65, 0.65] for z in [-0.43, -0.13, 0.17, 0.47]]


def research_team():
    profile = [(0, 0.18), (0.03, 0.21), (0.10, 0.23), (0.30, 0.24), (0.55, 0.27),
               (0.78, 0.31), (0.90, 0.32), (0.98, 0.29), (1.05, 0.22), (1.10, 0.12), (1.11, 0)]
    vertices, triangles = G.lathe(profile, 48)
    torso = ([(0.82 - 0.14 * (y / 1.11) ** 2 + x * 0.65, y, z * 1.20) for x, y, z in vertices], triangles)
    head = G.transform(G.sphere(sides=40, rings=24), center=(0.68, 1.40, 0), scale=(0.22, 0.24, 0.22))
    return [part(mesh, material, rotation=(0, angle + 0.55, 0), smooth=True)
            for angle, material in [(PI / 2, "teal"), (7 * PI / 6, "blue"), (11 * PI / 6, "ivory")] for mesh in [torso, head]] + [
            T((0, 0.085, 0), 0.82, 0.055, "ivory"), part(G.dodecahedron(0.32), "white", (0, 0.67, 0), rotation=(0.18, 0.35, 0.12))]


def funding():
    return [B((-0.35, 1.0, -0.25), (1.4, 2, 0.18)), C((-0.35, 0.75, -0.09), 0.0 + 0.22, 0.12, "gold"),
            S((-0.35, 0.75, -0.09), (0.24, 0.24, 0.09), "gold"),
            B((-0.44, 0.45, -0.1), (0.14, 0.33, 0.08), "gold", (0, 0, -0.15)),
            B((-0.23, 0.45, -0.1), (0.14, 0.33, 0.08), "gold", (0, 0, 0.15))] + [
            B((-0.35, y, -0.13), (0.9, 0.07, 0.05), "teal") for y in [1.35, 1.6, 1.8]] + [C((0.75, 0.10 + i * 0.19, 0.25), 0.42, 0.16, "gold") for i in range(4)]


def collaboration():
    nodes = [(-1, 0.42, 0.35), (1, 0.42, 0.35), (0, 1.85, -0.15)]
    return [S(p, (0.42, 0.42, 0.42)) for p in nodes] + [R(a, b, 0.12, "teal") for a, b in zip(nodes, nodes[1:] + nodes[:1])]


def field_site():
    shelter = [B((0.55, 0.09, 0), (1.48, 0.18, 1.30), "steel"), B((0.55, 0.65, 0), (1.30, 1.04, 1.10), "ivory"),
        E([(-0.78, 0), (0, 0.35), (0.78, 0)], 1.32, "teal", (0.55, 1.17, 0)),
        B((0.37, 0.55, 0.57), (0.40, 0.81, 0.045), "dark"), B((0.37, 0.56, 0.60), (0.34, 0.75, 0.025), "ivory"),
        B((0.49, 0.52, 0.63), (0.025, 0.12, 0.025), "steel"),
        B((0.91, 0.79, 0.58), (0.35, 0.35, 0.04), "steel"), B((0.91, 0.79, 0.605), (0.28, 0.28, 0.02), "water"),
        *[B((1.215, y, -0.15), (0.025, 0.025, 0.56), "dark") for y in [0.35 + i * 0.07 for i in range(5)]],
        *[B((0.55, y, 0.562), (1.27, 0.012, 0.018), "steel") for y in [0.26, 0.46, 0.66, 0.86, 1.06]],
        B((0.55, 1.58, 0), (1.08, 0.06, 0.88), "dark", (-0.18, 0, 0)),
        *[B((0.20 + col * 0.23, 1.62 + (row - 1) * 0.045, -0.25 + row * 0.25), (0.21, 0.025, 0.23), "water", (-0.18, 0, 0)) for col in range(4) for row in range(3)],
        *[R((x, 1.29, z), (x, 1.55, z), 0.02, "steel") for x in [0.15, 0.95] for z in [-0.30, 0.30]]]
    mast = [R((-1.03, 0, 0), (-1.03, 2.12, 0), 0.038, "steel"),
        *[R((-1.03, 1.60, 0), point, 0.009, "steel") for point in [(-1.55, 0.05, 0.42), (-0.49, 0.05, 0.42), (-1.03, 0.05, -0.61)]],
        *[C((-1.03, y, 0), 0.15, 0.055, "white") for y in [1.03, 1.12, 1.21, 1.30, 1.39]],
        B((-1.03, 0.69, 0.08), (0.27, 0.37, 0.20), "ivory"), B((-1.03, 0.73, 0.19), (0.15, 0.12, 0.025), "water"),
        R((-1.03, 0.46, 0.10), (-0.11, 0.26, 0.10), 0.014, "dark"),
        R((-1.03, 1.80, 0), (-0.50, 1.80, 0), 0.018, "steel"),
        E([(-0.15, 0), (0.15, 0.09), (0.15, -0.09)], 0.02, "teal", (-0.57, 1.80, 0)),
        C((-1.03, 2.12, 0), 0.05, 0.12, "dark")]
    cups = [p for a in [0, 2 * PI / 3, 4 * PI / 3] for x, z in [(-1.03 + 0.29 * math.cos(a), 0.29 * math.sin(a))] for p in [
        R((-1.03, 2.12, 0), (x, 2.12, z), 0.014, "steel"),
        part(G.lathe([(0, 0.095), (0.035, 0.105), (0.09, 0.075), (0.12, 0.01)], 24), "ivory", (x, 2.12, z), rotation=(PI / 2, a, 0), smooth=True),
        part(G.lathe([(-0.002, 0.079), (0, 0.079)], 24), "dark", (x, 2.12, z), rotation=(PI / 2, a, 0))]]
    return shelter + mast + cups


RESEARCH = [("laboratory", "Laboratory", laboratory), ("doe-national-laboratory", "DOE National Laboratory", doe_national_laboratory),
            ("user-facility", "User Facility", user_facility), ("research-project", "Research Project", research_project),
            ("dataset", "Dataset", dataset), ("software", "Software", software), ("publication", "Publication", publication),
            ("research-team", "Research Team", research_team), ("funding", "Funding", funding),
            ("collaboration", "Collaboration", collaboration), ("field-site", "Field Site", field_site)]


def recipe(category, name, label, parts, radius=2.2, max_height=3.6, pedestal=True):
    vertices = [v for p in parts for v in p["vertices"]]
    lower = [min(v[a] for v in vertices) for a in range(3)]
    upper = [max(v[a] for v in vertices) for a in range(3)]
    factor = min((radius * 1.70) / max(upper[0] - lower[0], upper[2] - lower[2]), max_height / (upper[1] - lower[1]))
    factor = factor if pedestal else 1.0
    offset = (-(upper[0] + lower[0]) / 2, -lower[1], -(upper[2] + lower[2]) / 2)
    fitted = [dict(p, vertices=[[round((v[a] + offset[a]) * factor + (0.18 if pedestal and a == 1 else 0), 7) for a in range(3)] for v in p["vertices"]]) for p in parts]
    base = [part(G.lathe([(0, radius - 0.04), (0.04, radius), (0.14, radius), (0.18, radius - 0.04)], 64), "pedestal", group="pedestal")] if pedestal else []
    final = [dict(p, name=f"{p['group']}-{i:03d}") for i, p in enumerate(base + fitted)]
    used = sorted({p["material"] for p in final})
    materials = {key: BRAND.material(key) for key in used}
    if name == "research-team":
        materials = {key: BRAND.material(key, metallic=0.08, roughness=0.46) for key in used}
    if name in {"doe-national-laboratory", "laboratory", "user-facility"}:
        materials["glass"]["pbrMetallicRoughness"].update(baseColorFactor=[*BRAND.linear(BRAND.COLORS["Hydro"]), 0.26], metallicFactor=0, roughnessFactor=0.08)
        materials["glass"].update(alphaMode="BLEND", doubleSided=True)
        materials["light"]["emissiveFactor"] = [round(value * 0.70, 8) for value in BRAND.linear(BRAND.COLORS["Mist"])]
    if name in {"dataset", "software"}:
        materials["light"]["emissiveFactor"] = [round(value * 0.25, 8) for value in BRAND.linear(BRAND.COLORS["Mist"])]
    if name == "brinc-responder-station":
        cyan = BRAND.linear(BRAND.ACCENTS["Tech cyan"])
        materials["light"].update(extras={"brand": "custom accent", "colorName": "Tech cyan", "srgbHex": "#00E5FF"}, emissiveFactor=[value * 0.8 for value in cyan])
        materials["light"]["pbrMetallicRoughness"].update(baseColorFactor=[*cyan, 1], roughnessFactor=0.22)
        materials["white"]["pbrMetallicRoughness"].update(roughnessFactor=0.38)
    return dict(id=f"{category}/{name}", category=category, label=label, revision=7 if name == "research-team" else 6 if name == "user-facility" else 5 if name in {"doe-national-laboratory", "laboratory", "research-project"} else 4, materials=materials, parts=final,
                provenance="Original ACORN procedural miniature with ORNL brand palette and requested natural earth finishes; simplified illustrative geometry, not a measured architectural or geographic replica. No imported meshes or vendor branding.")


QUALITY_REVISIONS = {
    "artillery": 5,
    "autel-evo-max-4t": 6,
    "bitcraze-crazyflie-2-1": 5,
    "brinc-responder-station": 2,
    "cavalry": 5,
    "dataset": 5,
    "dji-avata-2": 2,
    "dji-matrice-350-rtk": 5,
    "dji-mavic-3-enterprise": 6,
    "dji-phantom-4-rtk": 5,
    "elistair-khronos-station": 2,
    "field-site": 5,
    "freefly-astro": 5,
    "gateway-arch": 6,
    "golden-gate-bridge": 5,
    "grand-canyon": 5,
    "holybro-x500-v2": 5,
    "hoover-dam": 6,
    "kennedy-space-center": 5,
    "ornl-strelka": 5,
    "parrot-anafi-ai": 6,
    "skydio-x10": 5,
    "software": 5,
    "us-capitol": 5,
    "user-facility": 7,
    "zipline-delivery-droid": 2
}

PROVENANCE = {
    "brinc-responder-station": "Original simplified illustration of the BRINC Responder Station, revised from manufacturer photographs: horizontally sliding tray shells, tapered dock, telescopic rails and skid supports. ORNL accents; not manufacturer CAD or a measured replica.",
    "landing-pad": "Original generic flight-operations landing mat with ORNL green, neutral rubber and reflective target markings. Standalone bottom-centered geometry; no helicopter-specific H, display pedestal, imported mesh or vendor branding."
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--category", choices=["drone", "research", "landmarks", "other", "risk"], required=True)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1] / "assets/models")
    args = parser.parse_args()
    collection = {"drone": DRONE, "research": RESEARCH, "landmarks": LANDMARKS, "other": OTHER.collection(sys.modules[__name__]), "risk": RISK}[args.category]
    folder = args.root / ("drones" if args.category == "drone" else args.category)
    folder.mkdir(parents=True, exist_ok=True)
    palette_path = args.root / "palette.json"
    previous_palette = json.loads(palette_path.read_text()) if palette_path.exists() else {}
    palette = previous_palette | {"brand": "ORNL", "source": "acorn-lib/src/util/constants/app.rs ORNL_COLOR_*",
        "srgb": previous_palette.get("srgb", {}) | BRAND.COLORS,
        "customAccents": previous_palette.get("customAccents", {}) | BRAND.ACCENTS,
        "naturalFinishes": previous_palette.get("naturalFinishes", {}) | dict(BRAND.NATURAL.values()) | {"Oxidized copper patina": "#78A99A"} | {value[0]: value[1] for value in OTHER.FINISHES.values()}}
    palette_path.write_text(json.dumps(palette, indent=2, sort_keys=True) + "\n")
    for name, label, create in collection:
        parts = create() if args.category == "other" else DETAIL.refine(name, create(), sys.modules[__name__], args.category)
        value = recipe(args.category, name, label, parts, radius=1.2 if args.category == "risk" else 2.2,
                       max_height=2.8 if args.category == "risk" else 3.6, pedestal=args.category != "other" and name != "landing-pad" and not name.startswith("base-"))
        if args.category == "other":
            value = OTHER.finish(value, BRAND)
        if args.category == "drone":
            if name in {entry[0] for entry in DRONE_ADDITIONS}:
                value["revision"] = 1
            value["provenance"] = f"Original simplified illustration inspired by {label}; ORNL palette; not manufacturer CAD or a flight/simulation model. See drones/README.md for source and selection context."
        value["revision"] = QUALITY_REVISIONS.get(name, value["revision"])
        value["provenance"] = PROVENANCE.get(name, value["provenance"])
        (folder / f"{name}.model.json").write_text(json.dumps(value, separators=(",", ":"), sort_keys=True) + "\n")
        print(f"Authored {value['id']}")


def golden_gate_bridge():
    def cable_y(x):
        return 1.15 + 1.9 * (x / 1.7) ** 2 if abs(x) <= 1.7 else 3.05 - (abs(x) - 1.7) * 1.5
    xs = [-3 + i * 0.15 for i in range(41)]
    return [B((0, 0.05, 0), (6.3, 0.1, 1.6), "water"), B((0, 0.75, 0), (6.2, 0.16, 0.75), "red"),
            B((0, 0.85, 0), (6.1, 0.05, 0.58), "dark")] + [
            item for x in [-1.7, 1.7] for item in [B((x, 1.6, -0.32), (0.17, 3.2, 0.17), "red"),
            B((x, 1.6, 0.32), (0.17, 3.2, 0.17), "red"), B((x, 0.25, 0), (0.50, 0.50, 1.0), "ivory"),
            *[B((x, y, 0), (0.19, 0.12, 0.75), "red") for y in [1.15, 2.15, 3.1]]]] + [
            R((a, cable_y(a), z), (b, cable_y(b), z), 0.04, "red") for a, b in zip(xs, xs[1:]) for z in [-0.32, 0.32]] + [
            R((x, 0.85, z), (x, cable_y(x), z), 0.018, "red") for x in xs[::2] for z in [-0.32, 0.32]]


CAPITOL_DOME = (1.86, [(0, 0.51), (0.12, 0.50), (0.28, 0.46), (0.44, 0.38), (0.57, 0.27), (0.66, 0.16), (0.70, 0.10)])


def us_capitol():
    dome_y, dome = CAPITOL_DOME
    return [B((0, 0.15, 0), (4.8, 0.3, 1.8)), B((0, 0.68, 0), (1.4, 1.0, 1.5)),
            *[B((x, 0.60, -0.05), (1.65, 0.90, 1.2)) for x in [-1.5, 1.5]],
            *[B((x, 1.1, -0.05), (1.78, 0.15, 1.35)) for x in [-1.5, 1.5]],
            L((0, 1.15, 0), [(0, 0.53), (0.47, 0.53)]), C((0, 1.65, 0), 0.65, 0.06), C((0, 1.77, 0), 0.55, 0.18),
            L((0, dome_y, 0), dome), C((0, 2.67, 0), 0.10, 0.22), L((0, 2.78, 0), [(0, 0.13), (0.08, 0)]),
            E([(-0.78, 0), (0, 0.36), (0.78, 0)], 0.38, center=(0, 1.0, 0.84)),
            *[B((0, 0.04 + i * 0.07, 1.2 - i * 0.1), (1.7, 0.08, 0.35)) for i in range(5)]] + [
            C((x, 0.62, 0.97), 0.055, 0.75, "ivory", 12) for x in [-0.55, -0.33, -0.11, 0.11, 0.33, 0.55]] + [
            B((x, y, 0.568), (0.12, 0.22, 0.03), "dark") for x in [-2.1, -1.8, -1.5, -1.2, -0.9, 0.9, 1.2, 1.5, 1.8, 2.1] for y in [0.46, 0.82]]


def gateway_arch():
    xs = [-1.40 + 2.80 * i / 96 for i in range(97)]
    def section(x):
        y = 0.10 + 2.90 * (math.cosh(1.40 / 0.62) - math.cosh(x / 0.62)) / (math.cosh(1.40 / 0.62) - 1)
        slope = -2.90 * math.sinh(x / 0.62) / (0.62 * (math.cosh(1.40 / 0.62) - 1))
        normal = (-slope / math.sqrt(1 + slope * slope), 1 / math.sqrt(1 + slope * slope))
        width = 0.085 + 0.23 * (abs(x) / 1.40) ** 1.6
        return [(x + u * width * normal[0], y + u * width * normal[1], v * width)
                for u, v in [(0.6667, 0), (-0.3333, -0.57735), (-0.3333, 0.57735)]]
    rings = [section(x) for x in xs]
    seams = [R(ring[i], ring[(i + 1) % 3], 0.0018, "dark") for ring in rings[4:-4:6] for i in range(3)]
    foundations = [B((x, 0.025, 0), (0.38, 0.05, 0.44), "ivory") for x in [-1.40, 1.40]]
    return [part(G.ring_mesh(rings), "steel")] + seams + foundations


def grand_canyon():
    columns, rows = 73, 57
    def channel(z):
        return 0.24 * math.sin(z * 1.7) + 0.10 * math.sin(z * 3.2 + 0.3)
    def terrace(distance, lower, upper):
        t = max(0, min(1, (distance - lower) / (upper - lower)))
        return t * t * (3 - 2 * t)
    def elevation(x, z):
        distance = abs(x - channel(z))
        distortion = 0.075 * math.sin(z * 4.1 + x * 2.3) + 0.045 * math.sin(z * 8.2 - x * 1.5)
        d = distance + distortion * min(1, distance * 2.5)
        height = 0.10 + 0.24 * terrace(d, 0.13, 0.42) + 0.25 * terrace(d, 0.43, 0.79) + 0.30 * terrace(d, 0.83, 1.21) + 0.23 * terrace(d, 1.22, 1.58)
        tributaries = sum(0.12 * math.exp(-((z - offset - slope * x) / 0.10) ** 2) for offset, slope in [(-0.82, 0.30), (0.28, -0.35), (1.04, 0.22)])
        texture = 0.018 * math.sin(x * 16 + z * 5) * math.sin(z * 11 - x * 3)
        return max(0.09, height - tributaries * min(1, distance) + texture * min(1, distance * 3))
    vertices = [((x / (columns - 1) - 0.5) * 4, elevation((x / (columns - 1) - 0.5) * 4, (z / (rows - 1) - 0.5) * 3), (z / (rows - 1) - 0.5) * 3)
                for z in range(rows) for x in range(columns)]
    faces = [face for z in range(rows - 1) for x in range(columns - 1) for a in [z * columns + x]
             for face in [(a, a + columns, a + 1), (a + 1, a + columns, a + columns + 1)]]
    bands = [(0, 0.34, "terracotta"), (0.34, 0.60, "rose"), (0.60, 0.91, "terracotta"), (0.91, 1.50, "sand")]
    surface = [part((vertices, [face for face in faces if lower <= sum(vertices[i][1] for i in face) / 3 < upper]), color)
               for lower, upper, color in bands]
    perimeter = list(range(columns)) + [z * columns + columns - 1 for z in range(1, rows)] + list(reversed(range((rows - 1) * columns, rows * columns - 1))) + [z * columns for z in reversed(range(1, rows - 1))]
    walls = []
    for lower, upper, color in bands:
        points, triangles = [], []
        for ai, bi in zip(perimeter, perimeter[1:] + perimeter[:1]):
            a, b = vertices[ai], vertices[bi]
            quad = [(a[0], min(lower, a[1]), a[2]), (b[0], min(lower, b[1]), b[2]), (b[0], min(upper, b[1]), b[2]), (a[0], min(upper, a[1]), a[2])]
            index = len(points)
            points += quad
            triangles += [(index, index + 2, index + 1)] if quad[2][1] > quad[1][1] else []
            triangles += [(index, index + 3, index + 2)] if quad[3][1] > quad[0][1] else []
        walls.append(part((points, triangles), color))
    river = [(channel(z) - 0.095, z) for z in [-1.5 + 3 * i / 64 for i in range(65)]] + [(channel(z) + 0.095, z) for z in [1.5 - 3 * i / 64 for i in range(65)]]
    return surface + walls + [B((0, 0.01, 0), (4, 0.02, 3), "terracotta"), E(river, 0.018, "water", (0, 0.125, 0), (PI / 2, 0, 0))]


def kennedy_space_center():
    core_x, core_z = -0.39, 0.15
    rocket = [L((core_x, 0.24, core_z), [(0, 0.19), (0.12, 0.22), (1.89, 0.22), (2.05, 0.17)], "orange"),
        L((core_x, 2.29, core_z), [(0, 0.17), (0.11, 0.15), (0.38, 0.15), (0.47, 0.13)], "white"),
        C((core_x, 2.65, core_z), 0.151, 0.055, "dark"),
        L((core_x, 2.76, core_z), [(0, 0.14), (0.17, 0.08), (0.24, 0.065)], "white"),
        R((core_x, 2.99, core_z), (core_x, 3.31, core_z), 0.035, "white", 0.008),
        *[C((core_x, y, core_z), 0.223, 0.023, "orange") for y in [0.55, 0.96, 1.65, 2.05]],
        R((core_x + 0.20, 0.40, core_z + 0.08), (core_x + 0.20, 2.20, core_z + 0.08), 0.018, "orange"),
        *[L((core_x + x, 0.10, core_z + z), [(0, 0.067), (0.07, 0.035), (0.17, 0.03)], "dark", 24) for x in [-0.10, 0.10] for z in [-0.10, 0.10]]]
    boosters = [p for x in [core_x - 0.36, core_x + 0.36] for p in [
        L((x, 0.18, core_z), [(0, 0.12), (0.12, 0.11), (1.85, 0.11), (2.05, 0.015)], "white"),
        *[C((x, y, core_z), 0.113, 0.018, "steel") for y in [0.48, 0.82, 1.16, 1.50, 1.82]],
        R((x, 0.25, core_z), (x, 0.10, core_z), 0.06, "dark", 0.09),
        *[R((x, y, core_z), (core_x, y, core_z), 0.035, "steel") for y in [0.55, 1.83]]]]
    tower = [R((x, 0.16, z), (x, 3.04, z), 0.04, "steel") for x in [0.54, 1.04] for z in [-0.34, 0.18]]
    tower += [B((0.79, y, -0.08), (0.62, 0.055, 0.64), "ivory") for y in [0.24 + i * 0.40 for i in range(8)]]
    tower += [R((0.54, y, z), (1.04, y + 0.40, z), 0.023, "steel") for y in [0.24 + i * 0.40 for i in range(7)] for z in [-0.34, 0.18]]
    tower += [B((0.20, y, 0.06), (0.82, 0.055, 0.14), "steel") for y in [0.64, 1.44, 2.22, 2.76]]
    tower += [R((x, 0.24, 0.23), (x, 2.98, 0.23), 0.012, "ivory") for x in [0.68, 0.89]]
    tower += [R((0.68, y, 0.23), (0.89, y, 0.23), 0.010, "ivory") for y in [0.28 + i * 0.10 for i in range(28)]]
    pad = [B((0, 0.07, 0), (2.20, 0.14, 1.73), "ivory"), B((core_x, 0.15, core_z), (1.08, 0.025, 0.90), "dark"),
        *[B((x, 0.15, -0.67), (0.15, 0.045, 0.10), "teal") for x in [-0.91, -0.62, -0.33, -0.04, 0.25, 0.54, 0.83]]]
    return rocket + boosters + tower + pad


def hoover_dam():
    # The arch center lies downstream (+Z), so the wall is convex toward the reservoir like a real arch-gravity dam.
    center = 2.27
    def point(radius, y, angle):
        return (radius * math.cos(angle), y, center - radius * math.sin(angle))
    def band(profile, start, end, material, steps=64):
        angles = [start + (end - start) * i / steps for i in range(steps + 1)]
        return part(G.ring_mesh([[point(r, y, a) for r, y in reversed(profile)] for a in angles]), material)
    start, end = 0.24 * PI, 0.76 * PI
    angles = [start + (end - start) * i / 64 for i in range(65)]
    wall = [band([(1.85, 0.16), (2.20, 1.65), (2.37, 1.65), (2.37, 0.16)], start, end, "ivory")]
    crest = [band([(2.175, 1.65), (2.175, 1.71), (2.395, 1.71), (2.395, 1.65)], start, end, "steel")]
    crest += [R(point(radius, 1.74, a), point(radius, 1.74, b), 0.013, "ivory")
              for radius in [2.20, 2.37] for a, b in zip(angles, angles[1:])]
    seams = [R(point(1.8547, 0.18, a), point(2.1953, 1.63, a), 0.004, "steel") for a in angles[4:-4:8]]
    face = center - math.sqrt(2.37 ** 2 - 0.71 ** 2)
    towers = [p for x in [-0.71, 0.71] for z in [face - 1.05, face - 0.45] for p in [
        C((x, 1.03, z), 0.105, 1.63, "ivory"), C((x, 1.84, z), 0.14, 0.10, "ivory"),
        L((x, 1.89, z), [(0, 0.14), (0.11, 0.12), (0.16, 0.07)], "steel"),
        *[B((x + dx, 1.80, z + 0.104), (0.025, 0.09, 0.012), "dark") for dx in [-0.045, 0.045]],
        B((x, 1.67, (z + face + 0.05) / 2), (0.10, 0.045, face + 0.05 - z), "ivory")]]
    rocks = [L((x, 0, -0.15), [(0, 0.66), (0.27, 0.77), (0.61, 0.69), (0.89, 0.63), (1.13, 0.72), (1.45, 0.59), (1.80, 0.46)], "terracotta", 9, (1, 1, 1.65)) for x in [-1.64, 1.64]]
    shoreline = [(p[0], p[2]) for p in [point(2.36, 0, a) for a in angles]] + [(-2.36 * math.cos(start), -1.40), (2.36 * math.cos(start), -1.40)]
    water = [E(shoreline, 1.13, "water", (0, 0.735, 0), (PI / 2, 0, 0)), B((0, 0.10, 1.16), (0.55, 0.07, 1.48), "water")]
    wings = [(0.30 * PI, 0.445 * PI), (0.555 * PI, 0.70 * PI)]
    powerhouse = [band([(1.50, 0.15), (1.50, 0.47), (1.83, 0.47), (1.83, 0.15)], a, b, "ivory", 16) for a, b in wings]
    powerhouse += [band([(1.493, 0.28), (1.493, 0.43), (1.503, 0.43), (1.503, 0.28)], a - 0.012, a + 0.012, "dark", 2)
                   for a0, a1 in wings for a in [a0 + (a1 - a0) * (i + 0.5) / 5 for i in range(5)]]
    return wall + crest + seams + towers + rocks + water + powerhouse


LANDMARKS = [("golden-gate-bridge", "Golden Gate Bridge", golden_gate_bridge),
             ("us-capitol", "U.S. Capitol", us_capitol), ("gateway-arch", "Gateway Arch", gateway_arch),
             ("grand-canyon", "Grand Canyon", grand_canyon),
             ("kennedy-space-center", "Kennedy Space Center", kennedy_space_center), ("hoover-dam", "Hoover Dam", hoover_dam)]
def infantry():
    return [B((-0.22, 0.12, 0.08), (0.30, 0.24, 0.55), "dark"), B((0.22, 0.12, 0.08), (0.30, 0.24, 0.55), "dark"),
            R((-0.22, 0.22, 0), (-0.18, 0.9, 0), 0.16, "teal"), R((0.22, 0.22, 0), (0.18, 0.9, 0), 0.16, "teal"),
            L((0, 0.75, 0), [(0, 0.36), (0.65, 0.42), (0.78, 0.20)], "teal", 12, (1, 1, 0.65)),
            S((0, 1.78, 0.02), (0.25, 0.28, 0.24), "ivory"), C((0, 1.97, 0), 0.32, 0.08, "teal"),
            L((0, 2.0, 0), [(0, 0.26), (0.12, 0.22), (0.18, 0)], "teal"),
            R((-0.36, 1.37, 0), (-0.44, 0.98, 0.24), 0.13, "teal"), R((-0.44, 0.98, 0.24), (0.25, 1.18, 0.34), 0.10, "ivory"),
            R((0.36, 1.37, 0), (0.40, 1.1, 0.31), 0.13, "teal"),
            R((-0.45, 0.94, 0.39), (0.65, 1.46, 0.39), 0.065, "dark"), B((-0.30, 1.0, 0.39), (0.36, 0.15, 0.16), "steel", (0, 0, 0.42))]


def cavalry():
    horse = [S((0, 1.02, 0), (0.69, 0.34, 0.29), "ivory"),
        S((0.42, 1.06, 0), (0.26, 0.37, 0.29), "ivory"),
        S((-0.44, 1.02, 0), (0.29, 0.34, 0.30), "ivory"),
        R((0.42, 1.08, 0), (0.67, 1.65, 0), 0.24, "ivory", 0.14),
        S((0.77, 1.67, 0), (0.24, 0.20, 0.17), "ivory"),
        S((0.99, 1.57, 0), (0.22, 0.12, 0.15), "ivory"),
        *[S((0.66, 1.88, z), (0.055, 0.14, 0.045), "ivory") for z in [-0.11, 0.11]],
        *[S((0.82, 1.74, z), (0.03, 0.03, 0.013), "dark") for z in [-0.16, 0.16]],
        *[S((1.11, 1.61, z), (0.025, 0.014, 0.010), "dark") for z in [-0.11, 0.11]],
        *[R((x, 0.96, z), (x + bend, 0.52, z), 0.115, "ivory", 0.065) for x, bend in [(-0.46, -0.09), (0.43, 0.03)] for z in [-0.22, 0.22]],
        *[S((x + bend, 0.52, z), (0.075, 0.08, 0.07), "ivory") for x, bend in [(-0.46, -0.09), (0.43, 0.03)] for z in [-0.22, 0.22]],
        *[R((x + bend, 0.52, z), (x + 0.08, 0.15, z), 0.055, "ivory", 0.065) for x, bend in [(-0.46, -0.09), (0.43, 0.03)] for z in [-0.22, 0.22]],
        *[B((x + 0.11, 0.09, z), (0.20, 0.17, 0.15), "dark") for x in [-0.46, 0.43] for z in [-0.22, 0.22]],
        R((-0.61, 1.15, 0), (-0.80, 0.70, 0), 0.10, "dark", 0.065), R((-0.80, 0.70, 0), (-0.73, 0.38, 0), 0.065, "dark", 0.025),
        *[R((0.53 - i * 0.035, 1.78 - i * 0.065, -0.02), (0.43 - i * 0.035, 1.70 - i * 0.065, -0.02), 0.045, "dark") for i in range(8)],
        B((-0.08, 1.31, 0), (0.61, 0.10, 0.66), "teal"), B((-0.08, 1.39, 0), (0.46, 0.09, 0.59), "dark"),
        *[R((0.90, 1.54, z), (0.55, 1.45, z), 0.015, "dark") for z in [-0.16, 0.16]],
        *[R((0.55, 1.45, z), (0.24, 1.66, z), 0.012, "dark") for z in [-0.16, 0.16]],
        R((1.03, 1.55, -0.15), (1.03, 1.55, 0.15), 0.015, "dark")]
    rider = [L((-0.08, 1.40, 0), [(0, 0.21), (0.39, 0.24), (0.51, 0.16)], "teal", 24, (1, 1, 0.83)),
        S((-0.08, 2.12, 0), (0.18, 0.22, 0.18), "ivory"),
        L((-0.08, 2.27, 0), [(0, 0.20), (0.04, 0.22), (0.13, 0.18), (0.18, 0.05)], "teal", 24),
        B((0.09, 2.28, 0), (0.20, 0.035, 0.33), "dark"),
        S((0.10, 2.13, 0), (0.03, 0.04, 0.035), "ivory"),
        *[S((0.08, 2.18, z), (0.012, 0.013, 0.014), "dark") for z in [-0.075, 0.075]],
        *[R((-0.07, 1.49, z), (0.12, 1.20, z * 1.19), 0.10, "teal") for z in [-0.28, 0.28]],
        *[R((0.12, 1.20, z), (-0.04, 0.93, z), 0.075, "dark") for z in [-0.33, 0.33]],
        *[B((0.02, 0.91, z), (0.24, 0.10, 0.14), "dark") for z in [-0.33, 0.33]],
        *[R((0.0, 1.80, z), (0.17, 1.57, z), 0.075, "teal") for z in [-0.17, 0.17]],
        *[R((0.17, 1.57, z), (0.30, 1.66, z), 0.065, "ivory") for z in [-0.17, 0.17]],
        B((-0.08, 1.51, 0), (0.44, 0.045, 0.36), "dark"),
        *[R((-0.17, 1.25, z), (-0.17, 0.91, z), 0.012, "steel") for z in [-0.36, 0.36]],
        *[B((-0.06, 0.88, z), (0.23, 0.025, 0.17), "steel") for z in [-0.36, 0.36]]]
    return [dict(p, vertices=G.transform((p["vertices"], p["triangles"]), rotation=(0, -PI / 2, 0))[0]) for p in horse + rider]


def artillery():
    wheels = [p for x in [-0.68, 0.68] for p in [T((x, 0.52, 0), 0.43, 0.065, "dark", (0, 0, PI / 2)),
        T((x, 0.52, 0), 0.36, 0.035, "gold", (0, 0, PI / 2)),
        R((x - 0.08, 0.52, 0), (x + 0.08, 0.52, 0), 0.12, "steel"),
        *[R((x, 0.52, 0), (x, 0.52 + 0.36 * math.cos(a), 0.36 * math.sin(a)), 0.035, "gold") for a in [i * PI / 4 for i in range(8)]],
        *[R((x - 0.015, 0.52 + 0.36 * math.cos(a), 0.36 * math.sin(a)), (x + 0.015, 0.52 + 0.36 * math.cos(a), 0.36 * math.sin(a)), 0.02, "steel") for a in [i * PI / 4 for i in range(8)]]]]
    barrel = part(G.lathe([(0, 0.21), (0.10, 0.25), (0.25, 0.23), (1.30, 0.16), (1.40, 0.20), (1.50, 0.20),
        (1.50, 0.125), (1.10, 0.125)], 48), "teal", (0, 0.85, -0.4), rotation=(PI / 2 - 0.12, 0, 0), smooth=False)
    carriage = [R((-0.80, 0.52, 0), (0.80, 0.52, 0), 0.065, "steel"),
        B((0, 0.51, -0.15), (0.84, 0.19, 0.62), "dark"),
        *[B((x, 0.71, -0.18), (0.12, 0.44, 0.38), "teal") for x in [-0.30, 0.30]],
        R((-0.40, 0.88, -0.15), (0.40, 0.88, -0.15), 0.075, "steel"),
        *[R((x * 0.8, 0.53, -0.18), (x, 0.08, -1.10), 0.09, "teal") for x in [-0.42, 0.42]],
        *[B((x, 0.08, -1.10), (0.25, 0.08, 0.20), "steel") for x in [-0.42, 0.42]],
        R((-0.36, 0.26, -0.73), (0.36, 0.26, -0.73), 0.045, "steel"),
        S((0, 0.83, -0.46), (0.095, 0.095, 0.11), "steel"),
        *[T((0, 0.85 + z * math.sin(0.12), -0.4 + z * math.cos(0.12)), radius, 0.015, "steel", (PI / 2 - 0.12, 0, 0)) for z, radius in [(0.16, 0.245), (0.95, 0.185), (1.45, 0.20)]],
        *[R((x - 0.02, 0.88, -0.15), (x + 0.02, 0.88, -0.15), 0.10, "gold") for x in [-0.38, 0.38]]]
    lining_mesh = G.lathe([(1.49, 0.124), (1.10, 0.124)], 48)
    lining = part((lining_mesh[0], lining_mesh[1][:96]), "dark", (0, 0.85, -0.4), rotation=(PI / 2 - 0.12, 0, 0))
    bore_bottom = part(G.lathe([(1.101, 0.124), (1.103, 0.124)], 48), "dark", (0, 0.85, -0.4), rotation=(PI / 2 - 0.12, 0, 0))
    return wheels + [barrel, lining, bore_bottom] + carriage


def scout():
    return [C((0, 0.16, 0), 0.55, 0.32, "steel", 6), P((0, 0.32, 0), (0.78, 1.8, 0.78), "teal", 0.32),
            P((0, 2.12, 0), (0.32, 0.45, 0.32), "ivory"), B((0, 1.10, 0.30), (0.30, 0.10, 0.05), "white")]


def squadron():
    return [C((0, 0.14, 0), 0.74, 0.28, "steel", 8), L((0, 0.28, 0), [(0, 0.60), (0.30, 0.45), (1.6, 0.45)], "teal", 8),
            C((0, 1.85, 0), 0.70, 0.22, "ivory", 8),
            *[B((x, 2.12, z), (0.32, 0.48, 0.32), "teal") for x in [-0.40, 0.40] for z in [-0.40, 0.40]],
            *[B((x, 1.15, 0.44), (0.11, 0.42, 0.045), "white") for x in [-0.15, 0.15]]]


def legion():
    return [C((0, 0.12, 0), 1.05, 0.24, "steel", 6), C((0, 0.70, 0), 0.88, 1.15, "teal", 6),
            C((0, 1.28, 0), 1.0, 0.18, "ivory", 6),
            *[C((0.78 * math.cos(a), 1.55, 0.78 * math.sin(a)), 0.25, 0.50, "teal", 6) for a in [i * PI / 3 for i in range(6)]],
            P((0, 1.38, 0), (0.66, 0.70, 0.66), "teal", 0.25),
            *[B((x, 0.78, 0.77), (0.10, 0.44, 0.035), "white") for x in [-0.25, 0, 0.25]]]


def faction_base(shape, color):
    polygons = {
        "diamond": [(0, -1), (0.75, 0), (0, 1), (-0.75, 0)],
        "plus": [(-0.3, -1), (0.3, -1), (0.3, -0.3), (1, -0.3), (1, 0.3), (0.3, 0.3), (0.3, 1), (-0.3, 1), (-0.3, 0.3), (-1, 0.3), (-1, -0.3), (-0.3, -0.3)],
        "square": [(-0.8, -0.8), (0.8, -0.8), (0.8, 0.8), (-0.8, 0.8)],
        "star": [((1 if i % 2 == 0 else 0.44) * math.cos(PI / 2 + i * PI / 5), (1 if i % 2 == 0 else 0.44) * math.sin(PI / 2 + i * PI / 5)) for i in range(10)],
        "triangle": [(-0.9, -0.7), (0.9, -0.7), (0, 0.9)],
    }
    symbols = [C((0, 0.22, z), 0.15, 0.06, "white") if shape == "circle" else
               E([(x * 0.16, y * 0.16) for x, y in polygons[shape]], 0.06, "white", (0, 0.22, z), (PI / 2, 0, 0)) for z in [-1.11, 1.11]]
    return [dict(C((0, 0.09, 0), 1.35, 0.18, color, 64), group="pedestal"),
            dict(C((0, 0.185, 0), 0.87, 0.01, color, 64), group="pedestal")] + [dict(p, group="symbol") for p in symbols]


RISK = [("artillery", "Artillery", artillery), ("cavalry", "Cavalry", cavalry), ("infantry", "Infantry", infantry),
        ("legion", "Legion", legion), ("scout", "Scout", scout), ("squadron", "Squadron", squadron)] + [
        (f"base-{shape}", f"{BRAND.NATURAL.get(color, (BRAND.ROLES[color],))[0]} / {shape.title()} Base", lambda s=shape, c=color: faction_base(s, c))
        for shape, color in [("circle", "teal"), ("diamond", "magenta"), ("plus", "orange"), ("square", "blue"), ("star", "purple"), ("triangle", "amber")]]



def propeller(center, radius=0.42, angle=0, rotation=(0, 0, 0)):
    outline = [(-radius, -0.025), (-0.82 * radius, -0.10), (-0.30 * radius, -0.12), (0.05, -0.04), (radius, 0.025), (0.82 * radius, 0.10), (0.30 * radius, 0.12), (-0.05, 0.04)]
    blade = G.transform(G.extrude(outline, 0.025), rotation=(PI / 2, angle, 0))
    hub = G.lathe([(-0.04, 0.06), (0.06, 0.06)], 12)
    return [part(blade, "dark", center, rotation=rotation), part(hub, "steel", center, rotation=rotation)]


def camera(center, size=0.22, lenses=1):
    x, y, z = center
    return [B(center, (size * 1.7, size, size), "dark"),
            *[T((x + (i - (lenses - 1) / 2) * size * 0.48, y, z + size * 0.67), size * 0.22, size * 0.035, "steel", (PI / 2, 0, 0)) for i in range(lenses)],
            *[R((x + (i - (lenses - 1) / 2) * size * 0.48, y, z + size / 2), (x + (i - (lenses - 1) / 2) * size * 0.48, y, z + size * 0.66), size * 0.20, "water") for i in range(lenses)]]


def quad_arms(span=1.1, length=0.8, y=0.75, rotor=0.45, material="ivory", rear_lift=0):
    return [p for x in [-span, span] for z in [-length, length] for p in
            [R((x * 0.18, y, z * 0.25), (x, y + (rear_lift if z < 0 else 0), z), 0.065, material),
             C((x, y + (rear_lift if z < 0 else 0), z), 0.105, 0.18, "dark", 16),
             *propeller((x, y + 0.14 + (rear_lift if z < 0 else 0), z), rotor, 0.45 if x * z > 0 else -0.45)]]


def landing_feet(points, ground, material="dark"):
    return [p for x, top, z in points for p in [R((x, top, z), (x, ground + 0.04, z), 0.028, material, 0.02),
            B((x, ground + 0.02, z), (0.07, 0.04, 0.11), material)]]


def skids(width=0.45, depth=0.70, height=0.65, material="dark"):
    return [p for x in [-width, width] for p in [R((x, 0.06, -depth), (x, 0.06, depth), 0.045, material),
            *[R((x * 0.65, height, z * 0.7), (x, 0.06, z), 0.035, material) for z in [-depth * 0.65, depth * 0.65]]]]


def dji_mavic_3_enterprise():
    return quad_arms(0.85, 0.65, 0.42, 0.36, "dark") + [
        drone_shell((0, 0.48, 0), 0.51, 0.30, 1.0, "ivory"),
        B((0, 0.65, -0.12), (0.34, 0.055, 0.54), "teal"), C((0, 0.76, -0.15), 0.12, 0.13, "ivory"),
        R((0, 0.40, 0.34), (0, 0.37, 0.39), 0.045, "steel"),
        *[B((x, 0.36, 0.43), (0.035, 0.16, 0.16), "dark") for x in [-0.19, 0.19]],
        *camera((0, 0.33, 0.49), 0.19, 2),
        *[S((x, 0.53, 0.45), (0.04, 0.035, 0.025), "water") for x in [-0.17, 0.17]],
        *[C((x, 0.45, z), 0.057, 0.12, "dark") for x in [-0.23, 0.23] for z in [-0.25, 0.28]],
        *landing_feet([(x, 0.34, z) for x in [-0.85, 0.85] for z in [-0.65, 0.65]], 0.205)]


def dji_matrice_350_rtk():
    return quad_arms(1.1, 0.85, 1.05, 0.50, "dark") + skids(0.55, 0.75, 1.05) + [
            B((0, 1.05, 0), (0.64, 0.38, 0.88)), B((0, 1.28, -0.07), (0.54, 0.12, 0.64), "teal"),
            *[R((x, 1.2, -0.25), (x, 1.65, -0.25), 0.035, "dark") for x in [-0.29, 0.29]],
            *[C((x, 1.65, -0.25), 0.09, 0.06) for x in [-0.29, 0.29]],
            R((0, 0.91, 0.20), (0, 0.55, 0.20), 0.07, "dark"), S((0, 0.48, 0.20), (0.22, 0.21, 0.20), "ivory"), *camera((0, 0.48, 0.38), 0.22, 2)]


def dji_phantom_4_rtk():
    return quad_arms(0.83, 0.70, 0.85, 0.40, "white") + skids(0.34, 0.58, 0.85, "white") + [
            S((0, 0.86, 0), (0.43, 0.21, 0.48), "white"), C((0, 1.20, 0), 0.12, 0.30, "ivory"),
            C((0, 1.38, 0), 0.20, 0.10, "teal"), R((0, 0.73, 0.20), (0, 0.48, 0.20), 0.05), *camera((0, 0.41, 0.27), 0.25)]


def parrot_anafi_ai():
    return quad_arms(0.95, 0.68, 0.48, 0.36, "white") + [
            S((0, 0.48, -0.07), (0.22, 0.16, 0.64), "white"), B((0, 0.60, -0.16), (0.30, 0.09, 0.62), "teal"),
            B((0, 0.55, 0.50), (0.24, 0.05, 0.07), "white"),
            *[S((x, 0.55, 0.50), (0.075, 0.075, 0.075), "white") for x in [-0.12, 0.12]],
            *[R((x, 0.55, 0.565), (x, 0.55, 0.585), 0.04, "dark") for x in [-0.12, 0.12]],
            *camera((0, 0.35, 0.61), 0.22), *landing_feet([(x, 0.40, z) for x in [-0.95, 0.95] for z in [-0.68, 0.68]], 0.22, "white")]


def autel_evo_max_4t():
    shell = drone_shell((0, 0.62, -0.02), 0.64, 0.34, 1.08, "ivory")
    return quad_arms(0.94, 0.72, 0.60, 0.40, "ivory", 0.06) + [shell,
        B((0, 0.80, -0.13), (0.45, 0.075, 0.63), "teal"),
        B((0, 0.65, 0.50), (0.42, 0.09, 0.045), "dark"),
        *[S((x, 0.65, 0.53), (0.035, 0.035, 0.018), "water") for x in [-0.15, 0.15]],
        R((0, 0.49, 0.35), (0, 0.45, 0.39), 0.05, "steel"),
        B((0, 0.43, 0.42), (0.51, 0.045, 0.25), "dark"),
        *[R((x, 0.44, 0.42), (x, 0.33, 0.48), 0.025, "steel") for x in [-0.23, 0.23]],
        *camera((0, 0.35, 0.50), 0.25, 3),
        *[C((x, 0.65, z), 0.065, 0.12, "dark") for x in [-0.30, 0.30] for z in [-0.28, 0.30]],
        *landing_feet([(x, 0.58 if z < 0 else 0.52, z) for x in [-0.94, 0.94] for z in [-0.72, 0.72]], 0.20)]


def skydio_x10():
    return quad_arms(1.02, 0.75, 0.58, 0.43, "dark", 0.20) + [
        drone_shell((0, 0.62, -0.02), 0.48, 0.25, 1.13, "dark"),
        B((0, 0.78, -0.18), (0.32, 0.08, 0.62), "teal"),
        R((0, 0.53, 0.36), (0, 0.40, 0.47), 0.045, "steel"),
        *[B((x, 0.41, 0.53), (0.035, 0.16, 0.21), "steel") for x in [-0.25, 0.25]],
        *camera((0, 0.37, 0.60), 0.26, 3),
        *[S((x, 0.66, 0.35), (0.055, 0.055, 0.045), "ivory") for x in [-0.24, 0.24]],
        *[S((x, 0.66, 0.39), (0.033, 0.033, 0.015), "water") for x in [-0.24, 0.24]],
        *[C((x, 0.61, z), 0.06, 0.11, "steel") for x in [-0.23, 0.23] for z in [-0.33, 0.30]],
        *[R((x, 0.55, z), (x, 0.06, z), 0.03, "dark") for x in [-0.40, 0.40] for z in [-0.43, 0.43]]]


def freefly_astro():
    return quad_arms(1.15, 0.83, 0.95, 0.52, "dark") + skids(0.6, 0.72, 0.95) + [
            B((0, 0.95, 0), (0.84, 0.12, 0.84), "dark"), *[B((x, 1.14, 0), (0.28, 0.28, 0.72), "teal") for x in [-0.22, 0.22]],
            C((0, 1.37, -0.18), 0.12, 0.10), R((0, 0.9, 0.12), (0, 0.52, 0.12), 0.05),
            *camera((0, 0.45, 0.22), 0.32), *[B((x, 0.45, 0.10), (0.06, 0.34, 0.32), "steel") for x in [-0.32, 0.32]]]


def holybro_x500_v2():
    return quad_arms(1.0, 0.83, 0.85, 0.44, "dark") + skids(0.50, 0.65, 0.85) + [
            B((0, 0.85, 0), (0.70, 0.06, 0.8), "dark"), B((0, 1.0, 0), (0.66, 0.05, 0.75), "dark"),
            B((0, 1.09, 0.05), (0.26, 0.12, 0.40), "teal"), B((0, 0.72, 0), (0.38, 0.20, 0.70), "ivory"),
            R((0, 1.03, -0.28), (0, 1.48, -0.28), 0.025, "dark"), C((0, 1.50, -0.28), 0.13, 0.06),
            *[C((x, 0.93, z), 0.025, 0.16, "steel", 8) for x in [-0.27, 0.27] for z in [-0.30, 0.30]],
            R((0.30, 1.02, 0.23), (0.35, 1.33, 0.23), 0.02, "dark")]


def crazyflie_2_1():
    pcb = [B((0, 0.22, 0), (0.54, 0.035, 0.62), "teal"),
        B((0, 0.32, 0.04), (0.29, 0.16, 0.43), "steel"),
        B((0, 0.405, 0.04), (0.28, 0.018, 0.20), "dark"),
        *[B((x, 0.25, z), (0.08, 0.035, 0.10), "dark") for x, z in [(-0.19, -0.19), (0.19, -0.19), (0.19, 0.20)]],
        *[B((x, 0.246, z), (0.022, 0.012, 0.04), "gold") for x in [-0.23, 0.23] for z in [-0.12, -0.04, 0.04, 0.12]],
        B((0, 0.25, -0.31), (0.15, 0.075, 0.08), "steel"), B((0, 0.25, -0.355), (0.10, 0.04, 0.018), "dark"),
        R((-0.24, 0.24, -0.25), (-0.29, 0.48, -0.32), 0.009, "dark"),
        R((0.15, 0.33, 0.20), (0.23, 0.25, 0.28), 0.009, "amber"),
        R((0.10, 0.33, 0.21), (0.18, 0.25, 0.28), 0.009, "dark")]
    arms = [p for x in [-0.75, 0.75] for z in [-0.75, 0.75] for p in [
        B((x * 0.60, 0.22, z * 0.60), (0.09, 0.035, 0.67), "teal", (0, PI / 4 if x * z > 0 else -PI / 4, 0)),
        C((x, 0.15, z), 0.052, 0.25, "steel", 16),
        T((x, 0.22, z), 0.065, 0.018, "dark"), T((x, 0.06, z), 0.06, 0.012, "dark"),
        R((x + 0.064, 0.045, z), (x + 0.064, 0.24, z), 0.015, "dark"),
        R((x - 0.064, 0.045, z), (x - 0.064, 0.24, z), 0.015, "dark"),
        *propeller((x, 0.33, z), 0.34, 0.45 if x * z > 0 else -0.45),
        R((x * 0.32, 0.245, z * 0.32), (x * 0.96, 0.245, z * 0.96), 0.007, "gold")]]
    return pcb + arms


def ornl_strelka():
    return quad_arms(1.35, 1.05, 1.15, 0.60, "dark") + skids(0.70, 0.92, 1.15) + [
            B((0, 1.14, 0), (0.90, 0.14, 1.4), "dark"), B((0, 1.40, -0.1), (0.65, 0.38, 0.9), "teal"),
            *[B((x, 1.36, -0.2), (0.26, 0.30, 1.05), "ivory") for x in [-0.53, 0.53]],
            B((0, 0.65, 0), (0.65, 0.58, 0.92), "ivory"), B((0, 0.34, 0), (0.60, 0.08, 0.87), "teal"),
            R((0, 1.6, -0.4), (0, 2.0, -0.4), 0.025, "dark"), C((0, 2.01, -0.4), 0.11, 0.07)]


def ebee_x():
    wing = [(-2, -0.65), (-1.9, 0.10), (-0.40, 0.75), (0, 0.80), (0.4, 0.75), (1.9, 0.1), (2, -0.65), (0.45, -0.25), (-0.45, -0.25)]
    return [E(wing, 0.10, "teal", (0, 0.35, 0), (PI / 2, 0, 0)), S((0, 0.36, 0.30), (0.37, 0.20, 0.72), "ivory"),
            *[E([(-0.3, 0), (0.22, 0), (0.12, 0.45), (-0.12, 0.6)], 0.045, "teal", (x, 0.35, -0.35), (0, PI / 2, 0)) for x in [-1.91, 1.91]],
            R((0, 0.37, -0.4), (0, 0.37, -0.69), 0.10, "dark"), *propeller((0, 0.37, -0.73), 0.35, 0.3, (PI / 2, 0, 0)),
            *camera((0, 0.15, 0.40), 0.20)]


def wingtraone_gen_ii():
    wing = [(-1.9, -0.42), (-1.9, 0.12), (-0.45, 0.40), (0.45, 0.40), (1.9, 0.12), (1.9, -0.42)]
    return [E(wing, 0.12, "teal", (0, 0.75, 0), (PI / 2, 0, 0)), S((0, 0.75, 0.20), (0.28, 0.21, 0.90), "ivory"),
            *[p for x in [-0.85, 0.85] for p in [R((x, 0.76, 0), (x, 0.76, 0.63), 0.11, "ivory"), *propeller((x, 0.76, 0.70), 0.43, 0.4, (PI / 2, 0, 0))]],
            B((0, 0.75, -1.0), (1.1, 0.07, 0.32), "teal"),
            *[E([(-0.25, 0), (0.18, 0), (0.1, 0.48), (-0.1, 0.55)], 0.045, "teal", (x, 0.76, -0.99), (0, PI / 2, 0)) for x in [-0.48, 0.48]],
            *camera((0, 0.55, 0.36), 0.22)]


def dji_agras_t50():
    tank = [P((0, 0.80, 0), (0.95, 0.85, 1.06), "ivory", 0.76),
            B((0, 1.24, 0), (1.02, 0.10, 1.14), "teal"), C((0, 1.32, 0.27), 0.16, 0.08, "dark"),
            B((0, 1.49, -0.12), (0.58, 0.42, 0.70), "dark"), B((0, 1.72, -0.12), (0.45, 0.055, 0.55), "teal"),
            *[B((x, 0.84, 0.51), (0.045, 0.55, 0.025), "steel") for x in [-0.30, 0.30]],
            *camera((0, 1.24, 0.65), 0.16, 2), S((0, 0.47, 0.55), (0.25, 0.12, 0.08), "dark")]
    rotors = [p for x in [-1.30, 1.30] for z in [-1.06, 1.06] for p in [
        R((x * 0.27, 1.20, z * 0.27), (x, 1.30, z), 0.085, "dark"),
        C((x, 1.28, z), 0.13, 0.30, "steel", 20),
        *propeller((x, 1.49, z), 0.58, 0.35), *propeller((x, 1.07, z), 0.58, -0.50),
        T((x, 1.35, z), 0.13, 0.018, "teal"),
        R((x * 0.30, 1.25, z * 0.30), (x * 0.93, 1.35, z * 0.93), 0.015, "steel")]]
    plumbing = [p for x in [-0.82, 0.82] for p in [
        R((0, 0.43, -0.15), (x, 0.43, -0.70), 0.035, "dark"),
        C((x, 0.36, -0.70), 0.09, 0.14, "steel"), C((x, 0.27, -0.70), 0.15, 0.025, "teal"),
        R((x * 0.2, 0.67, -0.48), (x, 0.43, -0.70), 0.018, "teal")]]
    return tank + rotors + plumbing + skids(0.73, 0.95, 1.18)


def dji_inspire_3():
    body = [S((0, 0.96, 0), (0.30, 0.24, 0.82), "steel"),
            B((0, 1.17, -0.20), (0.40, 0.15, 0.77), "teal"),
            *[B((x, 1.08, -0.40), (0.17, 0.30, 0.64), "dark") for x in [-0.20, 0.20]],
            *[B((x, 1.01, 0.15 + i * 0.055), (0.018, 0.13, 0.022), "dark") for x in [-0.28, 0.28] for i in range(6)],
            R((0, 0.82, 0.35), (0, 0.48, 0.35), 0.065, "steel"),
            *[R((x, 0.50, 0.35), (x, 0.28, 0.35), 0.035, "dark") for x in [-0.23, 0.23]],
            *camera((0, 0.28, 0.45), 0.30), R((0, 0.28, 0.65), (0, 0.28, 0.82), 0.13, "dark"),
            T((0, 0.28, 0.82), 0.125, 0.018, "steel", (PI / 2, 0, 0)),
            *camera((0, 0.99, 0.78), 0.10, 2)]
    arms = [p for x in [-1.08, 1.08] for p in [
        R((x * 0.22, 0.98, 0), (x, 1.21, 0), 0.09, "dark"),
        R((x, 1.21, -0.76), (x, 1.21, 0.76), 0.055, "dark"),
        C((x * 0.45, 1.04, 0), 0.12, 0.16, "steel"),
        *[q for z in [-0.76, 0.76] for q in [C((x, 1.21, z), 0.105, 0.16, "dark"),
            *propeller((x, 1.34, z), 0.44, 0.4 if z > 0 else -0.4),
            R((x, 1.15, z), (x * 1.10, 0.06, z), 0.035, "dark"),
            B((x * 1.10, 0.06, z), (0.17, 0.08, 0.22), "steel")]]]]
    return body + arms


def flyability_elios_3():
    center_y, radius = 1.38, 1.36
    rings = [[(radius * math.sin(phi) * math.cos(a), center_y + radius * math.cos(phi), radius * math.sin(phi) * math.sin(a))
              for a in [2 * PI * (i + (0.5 if row % 2 else 0)) / 10 for i in range(10)]]
             for row, phi in enumerate([PI / 6, PI / 3, PI / 2, 2 * PI / 3, 5 * PI / 6])]
    cage = [R(ring[i], ring[(i + 1) % 10], 0.018, "steel") for ring in rings for i in range(10)]
    cage += [R(rings[row][i], rings[row + 1][j % 10], 0.018, "steel")
             for row in range(4) for i in range(10) for j in [i, i + (1 if row % 2 else -1)]]
    cage += [R(pole, point, 0.018, "steel") for pole, ring in [((0, center_y + radius, 0), rings[0]), ((0, center_y - radius, 0), rings[-1])] for point in ring]
    aircraft = quad_arms(0.51, 0.51, 1.30, 0.31, "dark") + [
        B((0, 1.30, 0), (0.40, 0.20, 0.55), "teal"), B((0, 1.46, -0.15), (0.28, 0.17, 0.38), "dark"),
        C((0, 1.68, -0.15), 0.14, 0.24, "steel"), C((0, 1.72, -0.15), 0.145, 0.09, "dark"),
        *camera((0, 1.21, 0.61), 0.22, 2),
        *[T((x, 1.42, z), 0.34, 0.024, "dark") for x in [-0.51, 0.51] for z in [-0.51, 0.51]],
        *[B((x, 1.39, 0.67), (0.16, 0.09, 0.05), "white") for x in [-0.43, -0.22, 0.22, 0.43]],
        *[R((x * 0.48, 1.30, 0), (x, 1.38, 0), 0.035, "teal") for x in [-1.34, 1.34]]]
    return cage + aircraft


def freefly_alta_x():
    return quad_arms(1.35, 1.10, 1.12, 0.64, "dark") + skids(0.68, 0.82, 1.12) + [
        C((0, 1.13, 0), 0.53, 0.16, "dark", 8), C((0, 1.28, 0), 0.47, 0.12, "steel", 8),
        *[B((x, 1.49, 0), (0.34, 0.34, 0.85), "teal") for x in [-0.22, 0.22]],
        *[B((0, 1.67, z), (0.84, 0.025, 0.085), "dark") for z in [-0.25, 0.25]],
        *[R((x, 1.04, z), (x, 0.65, z), 0.035, "steel") for x in [-0.31, 0.31] for z in [-0.27, 0.27]],
        B((0, 0.65, 0), (0.76, 0.06, 0.65), "dark"),
        *[B((x, 0.43, 0), (0.055, 0.43, 0.30), "steel") for x in [-0.33, 0.33]],
        *camera((0, 0.30, 0.17), 0.38),
        R((0, 1.33, -0.37), (0, 1.88, -0.37), 0.025, "dark"), C((0, 1.90, -0.37), 0.12, 0.05),
        *[T((x, 1.18, z), 0.11, 0.018, "steel") for x in [-1.35, 1.35] for z in [-1.10, 1.10]],
        *[C((x, 1.37, z), 0.035, 0.025, "steel", 8) for x in [-0.36, 0.36] for z in [-0.30, 0.30]]]


def quantum_trinity_pro():
    wing = [(-2.15, -0.32), (-2.15, 0.08), (-0.40, 0.42), (0.40, 0.42), (2.15, 0.08), (2.15, -0.32)]
    fuselage = [E(wing, 0.10, "ivory", (0, 0.62, 0), (PI / 2, 0, 0)),
                S((0, 0.62, 0.24), (0.24, 0.22, 0.83), "ivory"),
                B((0, 0.82, 0.28), (0.28, 0.055, 0.62), "teal"),
                R((0, 0.63, -0.22), (0, 0.63, -1.44), 0.065, "dark", 0.035),
                *[B((side * 0.29, 0.79, -1.30), (0.66, 0.06, 0.33), "teal", (0, 0, side * 0.55)) for side in [-1, 1]],
                *camera((0, 0.40, 0.45), 0.20),
                *[R((x, 0.68, -0.26), (x, 0.68, 0.12), 0.008, "steel") for x in [-1.7, -1.0, 1.0, 1.7]],
                *[B((x, 0.68, -0.23), (1.02, 0.012, 0.028), "teal") for x in [-1.58, 1.58]]]
    propulsion = [p for x in [-0.89, 0.89] for p in [
        R((x, 0.63, -0.05), (x, 0.63, 0.76), 0.085, "dark"),
        S((x, 0.66, 0.78), (0.13, 0.10, 0.18), "teal"),
        C((x, 0.72, 0.83), 0.09, 0.13, "steel"), *propeller((x, 0.83, 0.83), 0.41, 0.3),
        R((x, 0.60, 0.50), (x, 0.08, 0.50), 0.025, "dark")]] + [
        C((0, 0.70, -1.40), 0.09, 0.14, "steel"), *propeller((0, 0.82, -1.40), 0.38, -0.3),
        R((0, 0.61, -1.2), (0, 0.08, -1.2), 0.025, "dark")]
    return fuselage + propulsion


def voliro_t():
    body = [S((0, 0.52, 0), (0.36, 0.20, 0.47), "ivory"),
            *[B((x, 0.75, -0.06), (0.25, 0.25, 0.54), "teal") for x in [-0.15, 0.15]],
            R((0, 0.52, 0.38), (0, 0.52, 1.17), 0.035, "steel"),
            R((0, 0.52, 1.16), (0, 0.52, 1.28), 0.10, "dark"),
            T((0, 0.52, 1.28), 0.10, 0.02, "teal", (PI / 2, 0, 0)),
            *camera((0, 0.48, 0.43), 0.15, 2),
            *[B((x, 0.61, 0.39), (0.10, 0.07, 0.04), "white") for x in [-0.27, 0.27]],
            *[R((x, 0.48, z), (x, 0.06, z), 0.025, "dark") for x, z in [(-0.30, 0.24), (0.30, 0.24), (0, -0.40)]]]
    arms = [p for x in [-1.0, 1.0] for p in [
        R((x * 0.28, 0.57, 0), (x * 0.65, 0.76, 0.05), 0.055, "dark"),
        R((x * 0.65, 0.76, 0.05), (x, 0.57, 0.08), 0.055, "dark"),
        R((x * 0.91, 0.57, 0.08), (x * 1.07, 0.57, 0.08), 0.11, "steel"),
        *propeller((x, 0.63, 0.08), 0.46, 0.2, (0, 0, x * 0.40)),
        *propeller((x, 0.63, 0.08), 0.46, PI / 2 + 0.2, (0, 0, x * 0.40))]]
    tail = [R((0, 0.53, -0.28), (0, 0.53, -1.10), 0.04, "dark"),
            C((0, 0.56, -1.10), 0.075, 0.12, "steel"), *propeller((0, 0.66, -1.10), 0.30, 0.1)]
    return body + arms + tail


def dji_avata_2():
    guards = [p for x in [-0.62, 0.62] for z in [-0.57, 0.57] for p in [
        T((x, 0.20, z), 0.49, 0.045, "dark"), T((x, 0.36, z), 0.49, 0.035, "ivory"),
        *[R((x + 0.49 * math.cos(a), 0.20, z + 0.49 * math.sin(a)), (x + 0.49 * math.cos(a), 0.36, z + 0.49 * math.sin(a)), 0.018, "dark") for a in [i * PI / 3 for i in range(6)]],
        R((x - 0.45, 0.19, z), (x + 0.45, 0.19, z), 0.025, "dark"),
        R((x, 0.19, z - 0.45), (x, 0.19, z + 0.45), 0.025, "dark"),
        C((x, 0.23, z), 0.09, 0.12, "steel"), *three_blade_propeller((x, 0.31, z), 0.43, 0.1)]]
    return guards + [S((0, 0.42, -0.08), (0.33, 0.24, 0.78), "ivory"),
        B((0, 0.60, -0.24), (0.39, 0.10, 0.65), "teal"), *camera((0, 0.40, 0.68), 0.28),
        *[B((x, 0.45, -0.27), (0.02, 0.12, 0.31), "dark") for x in [-0.31, 0.31]],
        *[S((x, 0.37, -0.73), (0.065, 0.065, 0.03), "water") for x in [-0.17, 0.17]],
        *[B((x, 0.07, z), (0.15, 0.14, 0.20), "dark") for x in [-0.50, 0.50] for z in [-0.65, 0.65]]]


def dji_flycart_30():
    propulsion = [p for x in [-1.38, 1.38] for z in [-1.10, 1.10] for p in [
        R((x * 0.26, 1.38, z * 0.30), (x, 1.43, z), 0.09, "dark"),
        C((x, 1.43, z), 0.13, 0.35, "steel"), T((x, 1.43, z), 0.14, 0.025, "teal"),
        *propeller((x, 1.66, z), 0.60, 0.3), *propeller((x, 1.20, z), 0.60, -0.4),
        R((x * 0.30, 1.44, z * 0.30), (x * 0.91, 1.49, z * 0.91), 0.015, "steel")]]
    cargo = [B((0, 0.51, 0), (1.02, 0.62, 1.34), "ivory"), B((0, 0.85, 0), (1.08, 0.08, 1.40), "teal"),
        *[B((x, 0.51, z), (0.055, 0.65, 0.055), "steel") for x in [-0.49, 0.49] for z in [-0.64, 0.64]],
        *[B((x, 0.70, 0.70), (0.12, 0.16, 0.045), "dark") for x in [-0.35, 0.35]],
        *[B((x, 0.52, 0.69), (0.035, 0.47, 0.03), "steel") for x in [-0.18, 0.18]],
        *[R((x, 0.86, z), (x, 1.31, z), 0.04, "steel") for x in [-0.44, 0.44] for z in [-0.48, 0.48]]]
    body = [B((0, 1.36, 0), (0.92, 0.20, 1.06), "dark"),
        *[B((x, 1.59, -0.10), (0.33, 0.30, 0.80), "teal") for x in [-0.23, 0.23]],
        B((0, 1.80, -0.28), (0.30, 0.16, 0.32), "ivory"),
        *camera((0, 1.42, 0.58), 0.18, 2), S((0, 1.15, 0.57), (0.23, 0.12, 0.08), "dark"),
        *[C((x, 1.56, z), 0.025, 0.04, "steel", 8) for x in [-0.39, 0.39] for z in [-0.43, 0.43]]]
    return propulsion + cargo + body + skids(0.83, 0.91, 1.34)


def inspired_flight_if1200a():
    arms = [p for a in [PI / 6 + i * PI / 3 for i in range(6)] for x, z in [(1.45 * math.cos(a), 1.45 * math.sin(a))] for p in [
        R((x * 0.22, 1.03, z * 0.22), (x, 1.03, z), 0.065, "dark"),
        R((x * 0.26, 1.08, z * 0.26), (x * 0.90, 1.08, z * 0.90), 0.012, "teal"),
        C((x, 1.05, z), 0.11, 0.19, "dark"), T((x, 1.08, z), 0.11, 0.014, "steel"),
        *propeller((x, 1.20, z), 0.52, a), C((x * 0.26, 1.09, z * 0.26), 0.035, 0.04, "steel", 8)]]
    return arms + skids(0.63, 0.74, 1.03) + [
        C((0, 1.05, 0), 0.46, 0.20, "ivory", 6), C((0, 1.19, 0), 0.43, 0.10, "dark", 6),
        *[B((x, 1.40, -0.05), (0.28, 0.30, 0.71), "teal") for x in [-0.18, 0.18]],
        *[B((0, 1.56, z), (0.68, 0.025, 0.07), "dark") for z in [-0.24, 0.18]],
        R((0, 1.28, -0.40), (0, 1.81, -0.40), 0.025, "dark"), C((0, 1.83, -0.40), 0.14, 0.055),
        R((0, 0.95, 0.16), (0, 0.56, 0.16), 0.07, "steel"),
        S((0, 0.45, 0.16), (0.25, 0.25, 0.22), "ivory"), *camera((0, 0.45, 0.34), 0.25, 2),
        *[R((x, 1.20, -0.20), (x, 1.52, -0.26), 0.018, "dark") for x in [-0.41, 0.41]]]


def jouav_cw_15():
    wing = [(-2.15, -0.32), (-2.15, 0.05), (-0.50, 0.35), (0.50, 0.35), (2.15, 0.05), (2.15, -0.32)]
    airframe = [E(wing, 0.10, "ivory", (0, 0.90, 0), (PI / 2, 0, 0)),
        S((0, 0.86, 0.12), (0.26, 0.26, 1.13), "ivory"), B((0, 1.10, 0.12), (0.30, 0.05, 0.64), "teal"),
        R((0, 0.88, -0.64), (0, 0.88, -1.50), 0.065, "dark", 0.035),
        *[B((side * 0.31, 1.06, -1.40), (0.75, 0.055, 0.33), "teal", (0, 0, side * 0.62)) for side in [-1, 1]],
        R((0, 0.86, 1.08), (0, 0.86, 1.25), 0.09, "dark"), *propeller((0, 0.86, 1.28), 0.39, 0.2, (PI / 2, 0, 0)),
        S((0, 0.52, 0.55), (0.19, 0.20, 0.19), "dark"), *camera((0, 0.51, 0.69), 0.18, 2),
        *[B((x, 0.96, -0.25), (0.93, 0.014, 0.03), "teal") for x in [-1.50, 1.50]],
        *[R((x, 0.96, -0.29), (x, 0.96, 0.13), 0.009, "steel") for x in [-1.9, -0.5, 0.5, 1.9]]]
    lift = [p for x in [-0.88, 0.88] for p in [R((x, 0.86, -1.04), (x, 0.86, 1.04), 0.05, "dark"),
        *[q for z in [-1.04, 1.04] for q in [C((x, 0.92, z), 0.09, 0.14, "steel"), *propeller((x, 1.04, z), 0.40, 0.3 if z > 0 else -0.3),
            R((x, 0.82, z * 0.6), (x, 0.06, z * 0.6), 0.028, "dark"), B((x, 0.05, z * 0.6), (0.13, 0.06, 0.22), "dark")]]]]
    return airframe + lift


def modalai_starling_2():
    board = [B((0, 0.37, 0), (0.42, 0.06, 0.57), "dark"), B((0, 0.53, 0), (0.38, 0.035, 0.51), "teal"),
        *[C((x, 0.45, z), 0.025, 0.15, "steel", 8) for x in [-0.16, 0.16] for z in [-0.21, 0.21]],
        *[B((x, 0.57, z), (0.09, 0.05, 0.12), "dark") for x,z in [(-0.10, -0.14), (0.10, -0.14), (0.10, 0.12)]],
        B((-0.09, 0.58, 0.10), (0.12, 0.07, 0.12), "steel"),
        *[part(G.lathe([(-0.34, 0.10), (0.34, 0.10)], 24), "teal", (x, 0.23, -0.03), rotation=(PI / 2, 0, 0), smooth=True) for x in [-0.11, 0.11]],
        *[B((0, 0.13, z), (0.45, 0.025, 0.07), "dark") for z in [-0.20, 0.16]],
        *camera((0, 0.45, 0.35), 0.12, 2), *camera((-0.19, 0.34, 0.12), 0.09),
        R((0.13, 0.55, -0.21), (0.25, 0.98, -0.36), 0.015, "dark"), C((0, 0.65, -0.17), 0.085, 0.06, "ivory"),
        *[R((x, 0.34, z), (x, 0.06, z), 0.022, "dark") for x in [-0.78, 0.78] for z in [-0.64, 0.64]]]
    return quad_arms(0.78, 0.64, 0.37, 0.38, "dark") + board


def wingcopter_198():
    wing = [(-2.0, -0.31), (-2.0, 0.12), (-0.48, 0.42), (0.48, 0.42), (2.0, 0.12), (2.0, -0.31)]
    airframe = [E(wing, 0.12, "ivory", (0, 0.98, 0), (PI / 2, 0, 0)),
        S((0, 0.98, 0.03), (0.31, 0.24, 1.05), "ivory"), B((0, 1.19, -0.09), (0.36, 0.07, 0.75), "teal"),
        *[B((side * 0.32, 1.17, -1.00), (0.76, 0.065, 0.36), "teal", (0, 0, side * 0.60)) for side in [-1, 1]],
        *[E([(-0.24, 0), (0.17, 0), (0.09, 0.39), (-0.08, 0.47)], 0.035, "teal", (x, 0.98, -0.05), (0, PI / 2, 0)) for x in [-1.98, 1.98]],
        B((0, 0.52, 0.12), (0.58, 0.55, 0.94), "ivory"), B((0, 0.24, 0.12), (0.59, 0.045, 0.95), "teal"),
        *[B((x, 0.52, 0.60), (0.04, 0.47, 0.025), "steel") for x in [-0.20, 0.20]],
        *[B((x, 1.05, -0.23), (0.95, 0.012, 0.025), "steel") for x in [-1.37, 1.37]],
        *camera((0, 0.90, 0.99), 0.11)]
    propulsion = [p for x in [-1.62, -0.65, 0.65, 1.62] for p in [
        R((x, 0.98, -0.85), (x, 0.98, 0.85), 0.05, "dark"),
        *[q for z in [-0.85, 0.85] for q in [C((x, 1.01, z), 0.085, 0.16, "steel"),
            B((x, 0.97, z), (0.19, 0.13, 0.23), "teal" if abs(x)>1 else "ivory"),
            *propeller((x, 1.15, z), 0.39, 0.4 if x*z>0 else -0.4)]]]]
    gear = [R((x, 0.91, z), (x, 0.07, z), 0.028, "dark") for x in [-0.65, 0.65] for z in [-0.61, 0.61]]
    return airframe + propulsion + gear


def brinc_responder():
    body = [B((0, 0.88, 0), (0.67, 0.30, 1.05), "ivory"),
        B((0, 1.07, -0.13), (0.53, 0.10, 0.65), "teal"),
        B((0, 0.72, -0.08), (0.54, 0.12, 0.70), "dark"),
        R((0, 0.75, 0.42), (0, 0.48, 0.42), 0.065, "steel"),
        S((0, 0.46, 0.44), (0.22, 0.19, 0.18), "ivory"), *camera((0, 0.46, 0.59), 0.21, 2),
        *[B((x, 0.90, 0.54), (0.17, 0.075, 0.04), "blue" if x < 0 else "teal") for x in [-0.23, 0.23]],
        *[S((x, 0.98, 0.52), (0.045, 0.045, 0.025), "water") for x in [-0.23, 0.23]],
        *[B((x, 0.88, z), (0.018, 0.10, 0.018), "dark") for x in [-0.34, 0.34] for z in [-0.30 + i * 0.06 for i in range(9)]],
        B((0, 0.50, -0.17), (0.43, 0.28, 0.45), "white"),
        B((0, 0.50, 0.06), (0.25, 0.055, 0.015), "teal"), B((0, 0.50, 0.065), (0.055, 0.20, 0.015), "teal"),
        *[R((x, 1.04, -0.37), (x, 1.35, -0.42), 0.018, "dark") for x in [-0.22, 0.22]],
        *[B((x, 1.13, z), (0.04, 0.018, 0.04), "steel") for x in [-0.22, 0.22] for z in [-0.34, 0.12]]]
    return quad_arms(1.03, 0.88, 0.90, 0.47, "dark", 0.10) + skids(0.46, 0.67, 0.82) + body


def brinc_responder_station():
    def outline(y, width, depth, corner):
        return [(x, y, z) for x, z in [(-width / 2 + corner, -depth / 2), (width / 2 - corner, -depth / 2),
            (width / 2, -depth / 2 + corner), (width / 2, depth / 2 - corner),
            (width / 2 - corner, depth / 2), (-width / 2 + corner, depth / 2),
            (-width / 2, depth / 2 - corner), (-width / 2, -depth / 2 + corner)]]
    body = [part(G.ring_mesh([outline(0.72, 2.20, 2.42, 0.33), outline(0.83, 2.52, 2.72, 0.34),
        outline(1.42, 2.90, 3.08, 0.35), outline(1.64, 2.94, 3.12, 0.35)]), "dark"),
        part(G.ring_mesh([outline(1.64, 2.85, 3.03, 0.34), outline(1.69, 2.85, 3.03, 0.34)]), "bezel"),
        *[B((x, 1.697, z), (0.065, 0.012, 0.97), "ivory") for x in [-0.045, 0.045] for z in [-0.50, 0.50]],
        *[B((0, 1.72, z), (0.64, 0.10, 0.17), "dark") for z in [-1.35, 1.35]],
        B((0, 1.67, 1.53), (1.50, 0.48, 0.09), "dark", (-0.22, 0, 0)),
        B((0, 1.67, 1.585), (0.48, 0.045, 0.016), "teal", (-0.22, 0, 0)),
        *[B((x, 1.66, 1.58), (0.12, 0.026, 0.026), "steel") for x in [-0.43, 0.43]],
        *[B((x, y, 1.51), (0.16, 0.025, 0.07), "bezel") for x in [-0.89, 0.89] for y in [1.48, 1.58, 1.68, 1.78]],
        *[B((x, 1.15, 1.48), (0.055, 0.30, 0.022), "light", (0.32, 0, 0)) for x in [-1.02, 1.02]],
        *[B((x, 1.10, -1.46), (0.10, 0.25, 0.035), "bezel", (-0.32, 0, 0)) for x in [-0.28, -0.14, 0, 0.14, 0.28]]]
    skids = [p for side in [-1, 1] for p in [
        B((side * 1.42, 0.09, 0), (0.16, 0.14, 2.96), "dark"),
        *[part(G.beam((side * 1.04, 0.90, z * 1.04), (side * 1.42, 0.16, z * 1.37), 0.11, 4), "dark") for z in [-1, 1]],
        *[B((side * 1.42, 0.035, z), (0.24, 0.07, 0.25), "dark") for z in [-1.36, 1.36]],
        *[C((side * 1.42, 0.17, z), 0.036, 0.02, "steel", 12) for z in [-1.31, 1.31]],
        *[B((side * 1.10, 0.80, z), (0.30, 0.14, 0.22), "dark") for z in [-1.02, 1.02]]]]
    rails = [p for z in [-1.38, 1.38] for p in [
        B((0, 1.61, z), (5.93, 0.10, 0.105), "steel"),
        B((0, 1.61, z + (0.06 if z > 0 else -0.06)), (5.67, 0.028, 0.02), "dark"),
        *[B((side * 2.55, 1.66, z), (0.76, 0.055, 0.15), "ivory") for side in [-1, 1]],
        *[B((side * 1.44, 1.59, z), (0.21, 0.21, 0.18), "steel") for side in [-1, 1]],
        *[C((x, 1.70, z), 0.028, 0.022, "dark", 12) for x in [-2.78, -1.45, 1.45, 2.78]]]]
    # The two inward-open tray shells translate laterally on rails; their roofs stay level.
    shells = []
    for side in [-1, 1]:
        shell = [part(G.box(1.64, 0.095, 3.10, bevel=0.16), "white", (side * 2.19, 2.34, 0)),
            B((side * 2.19, 2.279, 0), (1.46, 0.022, 2.88), "ivory"),
            B((side * 3.025, 1.94, 0), (0.075, 0.84, 3.08), "white", (0, 0, side * 0.22)),
            *[E([(side * x, y) for x, y in [(1.37, 2.36), (2.96, 2.36), (3.16, 1.53),
                (2.96, 1.53), (2.83, 1.92), (1.37, 1.92)]], 0.07, "white", (0, 0, z)) for z in [-1.51, 1.51]],
            *[B((side * 3.105, 1.65, z), (0.012, 0.18, 0.15), "dark", (0, 0, side * 0.22)) for z in [-1.22, -0.99, -0.76, -0.53, 0.53, 0.76, 0.99, 1.22]],
            B((side * 2.77, 2.395, 0), (0.14, 0.012, 2.35), "teal"),
            *[B((side * 2.77, 2.402, z), (0.15, 0.01, 0.075), "ivory", (0, side * 0.35, 0)) for z in [-0.85, -0.45, -0.05, 0.35, 0.75]],
            *[B((side * 2.67, 1.86, z), (0.43, 0.07, 0.06), "steel") for z in [-1.40, 1.40]]]
        shells += [dict(p, group="left-shell" if side < 0 else "right-shell") for p in shell]
    return body + [dict(p, group="skids") for p in skids] + [dict(p, group="slide-rails") for p in rails] + shells


def dji_dock_3():
    base = [B((0, 0.50, 0), (2.35, 0.80, 2.08), "white"), B((0, 0.14, 0), (2.42, 0.15, 2.15), "dark"),
        B((0, 0.93, 0), (2.38, 0.12, 2.12), "dark"), C((0, 1.00, 0), 0.89, 0.035, "ivory"),
        T((0, 1.03, 0), 0.65, 0.025, "teal"),
        *[B((x, 1.03, 0), (0.075, 0.015, 0.59), "teal") for x in [-0.17, 0.17]], B((0, 1.03, 0), (0.36, 0.015, 0.075), "teal"),
        *[B((x, 0.53, 1.05), (0.025, 0.28, 0.025), "dark") for x in [-0.70 + i * 0.10 for i in range(15)]],
        B((0.78, 0.66, 1.06), (0.18, 0.17, 0.03), "teal"),
        *[B((x, 0.07, z), (0.24, 0.14, 0.24), "steel") for x in [-0.95, 0.95] for z in [-0.82, 0.82]],
        R((-0.88, 0.95, -0.78), (-0.88, 1.64, -0.78), 0.026, "steel"),
        C((-0.88, 1.65, -0.78), 0.10, 0.035, "ivory"),
        *[R((-0.88, 1.61, -0.78), (-0.88 + 0.17 * math.cos(a), 1.61, -0.78 + 0.17 * math.sin(a)), 0.012, "steel") for a in [0, 2 * PI / 3, 4 * PI / 3]],
        *camera((0.82, 1.09, 0.80), 0.10)]
    lids = [p for side in [-1, 1] for p in [B((side * 1.63, 1.20, 0), (1.02, 0.15, 2.16), "white", (0, 0, side * 0.36)),
        B((side * 1.63, 1.11, 0), (0.83, 0.05, 1.92), "dark", (0, 0, side * 0.36)),
        R((side * 1.16, 0.99, -0.94), (side * 1.16, 0.99, 0.94), 0.045, "steel"),
        *[R((side * 1.08, 0.65, z), (side * 1.70, 1.10, z), 0.025, "steel") for z in [-0.70, 0.70]]]]
    return base + lids


def dji_matrice_4d():
    body = [S((0, 0.76, -0.04), (0.37, 0.20, 0.64), "ivory"), B((0, 0.92, -0.17), (0.48, 0.10, 0.65), "teal"),
        B((0, 0.75, 0.49), (0.59, 0.21, 0.17), "ivory"),
        *[S((x, 0.80, 0.58), (0.045, 0.045, 0.025), "water") for x in [-0.22, 0.22]],
        R((0, 0.64, 0.37), (0, 0.43, 0.37), 0.065, "steel"),
        *camera((0, 0.43, 0.52), 0.24, 3),
        C((0, 1.05, -0.27), 0.13, 0.14, "ivory"), C((0, 1.13, -0.27), 0.13, 0.025, "teal"),
        *[B((x, 0.77, z), (0.018, 0.10, 0.022), "dark") for x in [-0.35, 0.35] for z in [-0.30 + i * 0.055 for i in range(8)]],
        *[R((x, 0.72, z), (x * 1.04, 0.08, z), 0.035, "ivory") for x in [-1.02, 1.02] for z in [-0.78, 0.78]],
        *[B((x * 1.04, 0.07, z), (0.12, 0.09, 0.18), "dark") for x in [-1.02, 1.02] for z in [-0.78, 0.78]],
        *[T((x, 0.79, z), 0.106, 0.012, "steel") for x in [-1.02, 1.02] for z in [-0.78, 0.78]]]
    return quad_arms(1.02, 0.78, 0.77, 0.46, "ivory") + body


def elistair_khronos():
    body = [B((0, 0.80, 0), (0.64, 0.28, 0.90), "teal"), B((0, 0.97, -0.04), (0.55, 0.09, 0.76), "ivory"),
        C((0, 1.07, -0.23), 0.10, 0.10, "dark"),
        R((0, 0.67, 0.31), (0, 0.38, 0.31), 0.06, "steel"),
        S((0, 0.32, 0.33), (0.20, 0.22, 0.20), "ivory"), *camera((0, 0.32, 0.48), 0.20, 2),
        *[B((x, 0.82, z), (0.02, 0.13, 0.018), "dark") for x in [-0.33, 0.33] for z in [-0.30 + i * 0.06 for i in range(10)]],
        R((0, 0.68, -0.26), (0, 0.17, -0.26), 0.025, "dark"),
        T((0, 0.14, -0.26), 0.065, 0.015, "steel", (PI / 2, 0, 0)),
        *[C((x, 1.03, z), 0.025, 0.025, "steel", 8) for x in [-0.22, 0.22] for z in [-0.30, 0.24]]]
    return quad_arms(0.95, 0.77, 0.81, 0.43, "dark") + skids(0.49, 0.60, 0.78) + body


def elistair_khronos_station():
    base = [B((0, 0.49, 0), (2.30, 0.82, 1.91), "teal"), B((0, 0.13, 0), (2.40, 0.17, 2.01), "dark"),
        B((0, 0.94, 0), (2.36, 0.10, 1.98), "ivory"), B((0, 1.00, 0), (1.95, 0.045, 1.58), "dark"),
        *[B((x, 0.49, z), (0.09, 0.83, 0.09), "steel") for x in [-1.12, 1.12] for z in [-0.92, 0.92]],
        *[B((x, 0.50, 0.98), (0.035, 0.31, 0.025), "dark") for x in [-0.79 + i * 0.10 for i in range(8)]],
        B((0.57, 0.58, 0.99), (0.49, 0.24, 0.035), "dark"), B((0.57, 0.59, 1.01), (0.33, 0.12, 0.025), "water"),
        *[R((x - 0.14, 0.40, z), (x + 0.14, 0.40, z), 0.035, "steel") for x in [-0.75, 0.75] for z in [-1.0, 1.0]],
        B((0, 1.32, -1.20), (2.36, 0.10, 0.93), "teal", (-0.65, 0, 0)),
        B((0, 1.37, -1.20), (2.12, 0.025, 0.73), "dark", (-0.65, 0, 0)),
        C((0.56, 1.08, -0.32), 0.24, 0.10, "steel"),
        *[T((0.56, 1.15 + i * 0.018, -0.32), 0.19, 0.012, "dark") for i in range(5)],
        *[B((x, 1.03, 0.10), (0.055, 0.015, 0.64), "white") for x in [-0.25, 0.05]], B((-0.10, 1.03, 0.10), (0.32, 0.015, 0.06), "white")]
    points = [(0.56 + (0.20 + i * 0.002) * math.cos(i * PI / 12), 1.04, -0.32 + (0.20 + i * 0.002) * math.sin(i * PI / 12)) for i in range(37)]
    return base + [R(a, b, 0.012, "dark") for a, b in zip(points, points[1:])]


def swissdrones_sdo_50_v3():
    body = [S((0, 0.92, 0.17), (0.34, 0.38, 0.87), "ivory"),
        S((0, 1.17, -0.04), (0.29, 0.17, 0.62), "teal"),
        R((0, 0.99, -0.45), (0, 1.06, -1.74), 0.15, "ivory", 0.045),
        B((0, 1.10, -1.48), (0.90, 0.045, 0.30), "teal"),
        E([(-0.34, 0), (0.18, 0), (0.13, 0.45), (-0.12, 0.56)], 0.05, "teal", (0, 1.05, -1.65), (0, PI / 2, 0)),
        *[R((side * 0.16, 1.21, -0.06), (side * 0.31, 1.77, -0.06), 0.055, "steel") for side in [-1, 1]],
        *[S((side * 0.23, 1.36, -0.06), (0.10, 0.17, 0.12), "dark") for side in [-1, 1]],
        *propeller((-0.31, 1.79, -0.06), 1.57, 0.15, (0, 0, 0.18)),
        *propeller((0.31, 1.85, -0.06), 1.57, PI / 2 + 0.15, (0, 0, -0.18)),
        R((0, 0.61, 0.56), (0, 0.37, 0.56), 0.055, "steel"), S((0, 0.32, 0.56), (0.19, 0.19, 0.18), "ivory"),
        *camera((0, 0.32, 0.70), 0.19, 2),
        *[B((x, 0.99, z), (0.022, 0.14, 0.025), "dark") for x in [-0.33, 0.33] for z in [-0.27 + i * 0.055 for i in range(9)]],
        R((0, 1.09, -0.47), (0, 1.10, -0.71), 0.10, "steel"),
        R((0, 1.30, 0.37), (0, 1.62, 0.30), 0.017, "dark")]
    return body + skids(0.48, 0.88, 0.82)


def zipline_delivery_droid():
    body = [L((0, 0.12, 0), [(0, 0.20), (0.05, 0.34), (0.18, 0.43), (0.45, 0.46), (0.72, 0.42), (0.92, 0.30), (1.02, 0.14), (1.06, 0)], "white", 48, (1, 1, 1.30)),
        part(G.torus(0.455, 0.012, sides=48), "teal", (0, 0.52, 0), (1, 1, 1.30), smooth=True),
        part(G.torus(0.41, 0.010, sides=48), "dark", (0, 0.86, 0), (1, 1, 1.30), smooth=True),
        B((0, 0.11, 0), (0.50, 0.06, 0.70), "dark"),
        C((0, 1.20, 0), 0.06, 0.10, "steel"), T((0, 1.30, 0), 0.06, 0.014, "steel", (PI / 2, 0, 0)),
        *camera((0, 0.46, 0.58), 0.13, 2),
        *[B((x, 0.06, z), (0.12, 0.12, 0.15), "dark") for x in [-0.24, 0.24] for z in [-0.30, 0.30]],
        *[E([(0, 0), (0.30, 0.08), (0.30, 0.20), (0, 0.34)], 0.025, "teal", (0, 0.50, -0.50), (0, PI / 2 + side * 0.9, 0)) for side in [-1, 1]]]
    duct = (0, 0.62, -0.84)
    thrusters = [T(duct, 0.22, 0.05, "teal", (PI / 2, 0, 0)), *propeller(duct, 0.19, 0.2, (PI / 2, 0, 0)),
        R((-0.22, 0.62, -0.84), (0.22, 0.62, -0.84), 0.016, "dark"),
        *[R((x * 0.12, 0.62 + y * 0.12, -0.55), (x * 0.22, 0.62 + y * 0.22, -0.84), 0.022, "white") for x, y in [(-1, 0), (1, 0), (0, 1)]]]
    thrusters += [p for side in [-1, 1] for p in [T((side * 0.62, 0.72, 0), 0.13, 0.032, "teal", (0, 0, PI / 2)),
        *propeller((side * 0.62, 0.72, 0), 0.11, 0.2, (0, 0, PI / 2)),
        R((side * 0.62, 0.59, 0), (side * 0.62, 0.85, 0), 0.012, "dark"),
        *[R((side * 0.38, y, 0), (side * 0.62, y, 0), 0.020, "white") for y in [0.59, 0.85]]]]
    return body + thrusters


def zipline_platform_2():
    wing = [(-1.85, -0.27), (-1.85, 0.16), (-0.45, 0.44), (0.45, 0.44), (1.85, 0.16), (1.85, -0.27)]
    body = [E(wing, 0.11, "white", (0, 1.08, 0), (PI / 2, 0, 0)),
        S((0, 0.91, 0.18), (0.31, 0.36, 1.03), "white"), B((0, 0.65, 0.21), (0.49, 0.19, 0.67), "teal"),
        B((0, 1.24, 0.12), (0.36, 0.05, 0.61), "teal"),
        *[B((x, 1.14, -0.10), (0.21, 0.025, 0.41), "teal") for x in [-1.73, 1.73]],
        *[B((x, 1.15, -0.22), (0.76, 0.014, 0.025), "steel") for x in [-1.14, 1.14]],
        R((0, 1.00, -0.48), (0, 1.04, -1.62), 0.10, "white", 0.045),
        B((0, 1.11, -1.47), (1.13, 0.055, 0.32), "teal"),
        *camera((0, 0.84, 1.16), 0.11),
        C((0, 0.49, 0.18), 0.14, 0.09, "dark"),
        *[R((x, 0.82, z), (x, 0.09, z), 0.025, "dark") for x in [-0.40, 0.40] for z in [-0.44, 0.58]]]
    lift = [p for x in [-1.02, 1.02] for p in [R((x, 1.07, -0.94), (x, 1.07, 0.94), 0.065, "ivory"),
        *[q for z in [-0.94, 0.94] for q in [C((x, 1.11, z), 0.10, 0.15, "steel"), *propeller((x, 1.24, z), 0.46, 0.25),
            B((x, 1.07, z), (0.18, 0.12, 0.28), "teal")]]]]
    return body + lift + [C((0, 1.10, -1.62), 0.09, 0.12, "steel"), *propeller((0, 1.21, -1.62), 0.40, 0.1)]


def drone_shell(center, width, height, depth, material):
    outline = [(-0.38, -0.50), (0.38, -0.50), (0.50, -0.32), (0.48, 0.25), (0.29, 0.50), (-0.29, 0.50), (-0.48, 0.25), (-0.50, -0.32)]
    rings = [[(x * width * taper, y * height, z * depth * length) for x, z in outline]
             for y, taper, length in [(-0.5, 0.73, 0.85), (-0.25, 1, 1), (0.17, 1, 1), (0.5, 0.74, 0.85)]]
    return part(G.ring_mesh(rings), material, center)


def three_blade_propeller(center, radius, angle=0):
    outline = [(0.02, -0.035), (radius * 0.35, -0.085), (radius * 0.85, -0.07), (radius, -0.015), (radius * 0.98, 0.025), (radius * 0.35, 0.035), (0.02, 0.025)]
    return [part(G.transform(G.extrude(outline, 0.018), rotation=(PI / 2, angle + i * 2 * PI / 3, 0)), "dark", center) for i in range(3)] + [C(center, 0.055, 0.07, "steel", 16)]



DRONE_ADDITIONS = [
    ("brinc-responder", "BRINC Responder", brinc_responder),
    ("brinc-responder-station", "BRINC Responder Station", brinc_responder_station),
    ("dji-agras-t50", "DJI Agras T50", dji_agras_t50),
    ("dji-avata-2", "DJI Avata 2", dji_avata_2),
    ("dji-dock-3", "DJI Dock 3", dji_dock_3),
    ("dji-flycart-30", "DJI FlyCart 30", dji_flycart_30),
    ("dji-inspire-3", "DJI Inspire 3", dji_inspire_3),
    ("dji-matrice-4d", "DJI Matrice 4D", dji_matrice_4d),
    ("elistair-khronos", "Elistair Khronos", elistair_khronos),
    ("elistair-khronos-station", "Elistair Khronos Ground Station", elistair_khronos_station),
    ("flyability-elios-3", "Flyability Elios 3", flyability_elios_3),
    ("freefly-alta-x", "Freefly Alta X", freefly_alta_x),
    ("inspired-flight-if1200a", "Inspired Flight IF1200A", inspired_flight_if1200a),
    ("jouav-cw-15", "JOUAV CW-15", jouav_cw_15),
    ("landing-pad", "Landing Pad", lambda: LANDING_PAD.create(sys.modules[__name__])),
    ("modalai-starling-2", "ModalAI Starling 2", modalai_starling_2),
    ("quantum-trinity-pro", "Quantum Systems Trinity Pro", quantum_trinity_pro),
    ("swissdrones-sdo-50-v3", "SwissDrones SDO 50 V3", swissdrones_sdo_50_v3),
    ("voliro-t", "Voliro T", voliro_t),
    ("wingcopter-198", "Wingcopter 198", wingcopter_198),
    ("zipline-delivery-droid", "Zipline Delivery Droid", zipline_delivery_droid),
    ("zipline-platform-2", "Zipline Platform 2", zipline_platform_2),
]


DRONE = [("autel-evo-max-4t", "Autel EVO Max 4T", autel_evo_max_4t), ("bitcraze-crazyflie-2-1", "Bitcraze Crazyflie 2.1", crazyflie_2_1),
         ("dji-matrice-350-rtk", "DJI Matrice 350 RTK", dji_matrice_350_rtk), ("dji-mavic-3-enterprise", "DJI Mavic 3 Enterprise", dji_mavic_3_enterprise),
         ("dji-phantom-4-rtk", "DJI Phantom 4 RTK", dji_phantom_4_rtk), ("ebee-x", "eBee X", ebee_x),
         ("freefly-astro", "Freefly Astro", freefly_astro), ("holybro-x500-v2", "Holybro X500 V2", holybro_x500_v2),
         ("ornl-strelka", "ORNL Strelka", ornl_strelka), ("parrot-anafi-ai", "Parrot ANAFI Ai", parrot_anafi_ai),
         ("skydio-x10", "Skydio X10", skydio_x10), ("wingtraone-gen-ii", "WingtraOne GEN II", wingtraone_gen_ii)]

DRONE = sorted(DRONE + DRONE_ADDITIONS, key=lambda entry: entry[0])

if __name__ == "__main__":
    main()
