"""Asset-specific construction details for the original model collection. MIT."""
import math


def research(name, m):
    B, C, E, L, R, S, T = m.B, m.C, m.E, m.L, m.R, m.S, m.T
    if name == "publication":
        result = [B((x, y, 0.83), (1.1, 0.012, 0.012), "steel", (0, 0, -0.16 if x < 0 else 0.16)) for x in [-0.62, 0.62] for y in [0.28, 0.32, 0.36, 0.40]] + [
            B((x, 0.49, z), (0.75, 0.012, 0.018), "steel", (0, 0, -0.16 if x < 0 else 0.16)) for x in [-0.65, 0.65] for z in [-0.64, -0.53, -0.33, -0.23, -0.02, 0.08, 0.28, 0.38, 0.58, 0.68]] + [
            B((0.14, 0.48, 0.78), (0.12, 0.025, 0.50), "teal", (0.10, 0, 0))]
    elif name == "funding":
        result = [T((0.75, 0.15 + i * 0.19, 0.25), 0.40, 0.012, "steel") for i in range(4)] + [
            R((0.75 + 0.42 * math.cos(a), 0.03 + i * 0.19, 0.25 + 0.42 * math.sin(a)), (0.75 + 0.42 * math.cos(a), 0.14 + i * 0.19, 0.25 + 0.42 * math.sin(a)), 0.005, "gold") for i in range(4) for a in [j * math.pi / 16 for j in range(32)]] + [
            B((-0.35, 1.06, -0.13), (0.85, 0.02, 0.02), "steel"), B((-0.52, 0.20, -0.13), (0.45, 0.035, 0.02), "dark", (0, 0, -0.08))]
    else:
        result = []
    return result


def refine(name, parts, m, category):
    additions = {"research": research, "landmarks": landmarks, "risk": risk, "drone": drone}[category](name, m)
    return parts + additions


def landmarks(name, m):
    B, C, E, L, R, S, T = m.B, m.C, m.E, m.L, m.R, m.S, m.T
    if name == "golden-gate-bridge":
        stations = [-3.01 + i * 6.02 / 28 for i in range(29)]
        result = [B((x, 0.88, 0), (0.16, 0.008, 0.018), "white") for x in [-2.8 + i * 0.28 for i in range(21)]] + [
            R((-3.05, 0.97, z), (3.05, 0.97, z), 0.016, "red") for z in [-0.37, 0.37]] + [
            R((x, 0.83, z), (x, 0.97, z), 0.008, "red") for x in [-3 + i * 0.15 for i in range(41)] for z in [-0.37, 0.37]] + [
            R((-3.01, 0.53, z), (3.01, 0.53, z), 0.025, "red") for z in [-0.32, 0.32]] + [
            R((a, 0.70 if i % 2 == 0 else 0.53, z), (b, 0.53 if i % 2 == 0 else 0.70, z), 0.018, "red")
            for i, (a, b) in enumerate(zip(stations, stations[1:])) for z in [-0.32, 0.32]] + [
            R((x, 0.53, z), (x, 0.70, z), 0.018, "red") for x in [-3.01, 3.01] for z in [-0.32, 0.32]] + [
            R((x, y, -0.32), (x, y + 0.65, 0.32), 0.026, "red") for x in [-1.7, 1.7] for y in [1.23, 2.23]] + [
            B((x, 0.92, z), (0.13, 0.07, 0.075), "ivory") for x, z in [(-2, -0.12), (-0.7, 0.12), (0.4, -0.12), (2.4, 0.12)]]
    elif name == "us-capitol":
        dome_y, dome = m.CAPITOL_DOME
        result = [R((0.612 * math.cos(a), 1.18, 0.612 * math.sin(a)), (0.612 * math.cos(a), 1.62, 0.612 * math.sin(a)), 0.022, "white")
                  for a in [i * math.pi / 12 for i in range(24)]] + [
            R(((r1 + 0.006) * math.cos(a), dome_y + y1, (r1 + 0.006) * math.sin(a)), ((r2 + 0.006) * math.cos(a), dome_y + y2, (r2 + 0.006) * math.sin(a)), 0.014, "white")
            for a in [i * math.pi / 8 for i in range(16)] for (y1, r1), (y2, r2) in zip(dome, dome[1:])] + [
            R((0.553 * math.cos(a), 1.72, 0.553 * math.sin(a)), (0.553 * math.cos(a), 1.82, 0.553 * math.sin(a)), 0.016, "dark")
            for a in [(i + 0.5) * math.pi / 8 for i in range(16)]] + [
            B((x, 0.65, z), (0.045, 0.80, 0.045), "white") for x in [-2.25, -2.0, -1.7, -1.4, -1.1, 1.1, 1.4, 1.7, 2.0, 2.25] for z in [-0.65, 0.62]] + [
            B((x, 0.85, -0.67), (0.12, 0.26, 0.025), "dark") for x in [-2.1, -1.8, -1.5, -1.2, 1.2, 1.5, 1.8, 2.1]] + [
            T((0, 1.70, 0), 0.655, 0.02, "white"), T((0, 1.16, 0), 0.62, 0.035, "white"),
            L((0, 2.86, 0), [(0, 0.055), (0.14, 0.025)], "dark", 12), S((0, 3.03, 0), (0.035, 0.04, 0.035), "dark")]

    else:
        result = []
    return result


def drone(name, m):
    B, C, R, S, T = m.B, m.C, m.R, m.S, m.T
    specs = {
        "autel-evo-max-4t": (0.94, 0.72, 0.60, 0.06, 0.60, 1.08),
        "dji-matrice-350-rtk": (1.10, 0.85, 1.05, 0, 0.64, 0.88),
        "dji-mavic-3-enterprise": (0.85, 0.65, 0.42, 0, 0.48, 1.0),
        "dji-phantom-4-rtk": (0.83, 0.70, 0.85, 0, 0.60, 0.72),
        "freefly-astro": (1.15, 0.83, 0.95, 0, 0.84, 0.84),
        "holybro-x500-v2": (1.0, 0.83, 0.85, 0, 0.70, 0.8),
        "ornl-strelka": (1.35, 1.05, 1.15, 0, 0.90, 1.4),
        "parrot-anafi-ai": (0.95, 0.68, 0.48, 0, 0.38, 0.9),
        "skydio-x10": (1.02, 0.75, 0.58, 0.20, 0.45, 1.13),
    }
    if name in specs:
        sx, sz, y, lift, width, length = specs[name]
        result = [p for x in [-sx, sx] for z in [-sz, sz] for h in [y + (lift if z < 0 else 0)] for p in [
            T((x, h - 0.05, z), 0.105, 0.009, "dark"), T((x, h + 0.06, z), 0.105, 0.009, "steel"),
            *[R((x + 0.105 * math.cos(a), h - 0.04, z + 0.105 * math.sin(a)), (x + 0.105 * math.cos(a), h + 0.05, z + 0.105 * math.sin(a)), 0.006, "dark") for a in [i * math.pi / 6 for i in range(12)]],
            C((x, h + 0.21, z), 0.028, 0.022, "dark", 6),
            R((x * 0.20, y + 0.045, z * 0.25), (x * 0.88, h + 0.045, z * 0.88), 0.010, "teal"),
            C((x * 0.23, y + 0.06, z * 0.25), 0.025, 0.025, "steel", 8)]]
        result += [B((side * (width / 2 + 0.012), y + 0.03, z), (0.015, 0.06, 0.02), "dark") for side in [-1, 1] for z in [-length * 0.30 + i * length * 0.06 for i in range(10)]]
        result += [B((side * width * 0.43, y + 0.16, -length * 0.10), (0.015, 0.025, length * 0.60), "steel") for side in [-1, 1]]
        result += [C((x, y + 0.19, z), 0.018, 0.018, "steel", 8) for x in [-width * 0.38, width * 0.38] for z in [-length * 0.32, length * 0.32]]
        if name in ["holybro-x500-v2", "freefly-astro", "ornl-strelka"]:
            result += [B((0, y + 0.28, z), (width * 0.65, 0.025, 0.08), "dark") for z in [-0.20, 0.20]]
            result += [R((width / 2, y, -0.15), (width / 2 + 0.09, y - 0.28, 0.10), 0.018, "amber"), R((width / 2 + 0.09, y - 0.28, 0.10), (0, y - 0.32, 0.15), 0.018, "amber")]
        else:
            result += [B((0, y + 0.23, -0.18), (width * 0.3, 0.012, 0.15), "steel"),
                       *[S((x, y + 0.08, length / 2 + 0.02), (0.028, 0.028, 0.015), "water") for x in [-width * 0.30, width * 0.30]]]
    elif name in {"ebee-x", "wingtraone-gen-ii"}:
        wing_y = 0.35 if name == "ebee-x" else 0.75
        result = [R((side * 0.45, wing_y + 0.065, -0.20), (side * 1.83, wing_y + 0.065, -0.38), 0.009, "steel") for side in [-1, 1]] + [
            R((side * 1.1, wing_y + 0.065, -0.30), (side * 1.1, wing_y + 0.065, 0.20), 0.008, "steel") for side in [-1, 1]] + [
            B((0, wing_y + 0.205, 0.1), (0.19, 0.015, 0.43), "teal"), *[C((x, wing_y + 0.21, z), 0.016, 0.015, "steel", 8) for x in [-0.1, 0.1] for z in [-0.10, 0.28]],
            R((0, wing_y + 0.15, -0.32), (0, wing_y + 0.36, -0.36), 0.012, "dark")]
    else:
        result = []
    return result


def risk(name, m):
    B, C, L, R, S, T = m.B, m.C, m.L, m.R, m.S, m.T
    if name.startswith("base-"):
        result = [dict(T((0, y, 0), radius, 0.008, "steel"), group="pedestal") for y, radius in [(0.04, 1.32), (0.14, 1.32), (0.185, 0.9)]] + [
            dict(R((1.348 * math.cos(a), 0.065, 1.348 * math.sin(a)), (1.348 * math.cos(a), 0.115, 1.348 * math.sin(a)), 0.006, "steel"), group="pedestal") for a in [i * math.pi / 48 for i in range(96)]]
    elif name == "infantry":
        result = [B((0, 1.06, 0), (0.69, 0.075, 0.48), "dark"), B((0, 1.06, 0.25), (0.10, 0.09, 0.025), "steel"),
            B((0, 1.22, -0.30), (0.45, 0.44, 0.22), "dark"),
            *[R((x, 1.48, -0.20), (x, 1.10, 0.26), 0.03, "steel") for x in [-0.22, 0.22]],
            *[B((x, 1.2, 0.25), (0.15, 0.18, 0.05), "teal") for x in [-0.20, 0.20]],
            S((0, 1.78, 0.26), (0.04, 0.06, 0.035), "ivory"), *[S((x, 1.84, 0.23), (0.024, 0.014, 0.013), "dark") for x in [-0.08, 0.08]],
            R((-0.19, 1.89, 0.1), (0, 1.56, 0.12), 0.018, "dark"), R((0, 1.56, 0.12), (0.19, 1.89, 0.1), 0.018, "dark"),
            *[B((x, 0.15, 0.24), (0.16, 0.02, 0.03), "steel") for x in [-0.22, 0.22]],
            *[R((-0.4 + i * 0.04, 0.96 + i * 0.02, 0.45), (-0.38 + i * 0.04, 1.06 + i * 0.02, 0.45), 0.008, "steel") for i in range(5)]]


    elif name == "scout":
        result = [B((0, y, 0), (w, 0.04, w), "steel") for y, w in [(0.35, 0.76), (0.55, 0.70), (1.95, 0.38), (2.10, 0.34)]] + [
            B((x, 1.18, 0.29), (0.014, 0.80, 0.025), "steel") for x in [-0.13, 0.13]] + [
            S((0, 1.65, 0.24), (0.065, 0.065, 0.025), "gold")]
    elif name in ["squadron", "legion"]:
        radius = 0.45 if name == "squadron" else 0.88
        result = [T((0, y, 0), radius + 0.01, 0.012, "steel") for y in [0.48, 0.70, 0.92, 1.14]] + [
            R((radius * math.cos(a), y, radius * math.sin(a)), (radius * math.cos(a), y + 0.18, radius * math.sin(a)), 0.008, "steel") for y in [0.49, 0.93] for a in [i * math.pi / 6 for i in range(12)]] + [
            B((0, 0.48, radius), (0.22, 0.48, 0.05), "dark"), *[B((0, 0.04 + i * 0.04, radius + 0.22 - i * 0.05), (0.32, 0.05, 0.12), "steel") for i in range(5)],
            *[B((x, 0.50, radius + 0.03), (0.014, 0.42, 0.02), "gold") for x in [-0.08, 0, 0.08]]]
    else:
        result = []
    return result
