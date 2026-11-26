"""Asset-specific construction details for the original model collection. MIT."""
import math


def person(m, x=0, z=0, coat="ivory"):
    B, R, S = m.B, m.R, m.S
    return [B((x + side * 0.16, 0.08, z + 0.05), (0.23, 0.16, 0.38), "dark") for side in [-1, 1]] + [
        R((x + side * 0.16, 0.14, z), (x + side * 0.13, 0.72, z), 0.105, "dark") for side in [-1, 1]] + [
        m.L((x, 0.60, z), [(0, 0.28), (0.38, 0.25), (0.62, 0.30), (0.75, 0.16)], coat, scale=(1, 1, 0.65)),
        S((x, 1.56, z), (0.18, 0.23, 0.16), "ivory"), S((x, 1.72, z - 0.025), (0.18, 0.09, 0.15), "dark"),
        S((x, 1.55, z + 0.16), (0.035, 0.05, 0.035), "ivory"),
        *[S((x + side * 0.066, 1.61, z + 0.14), (0.018, 0.015, 0.012), "dark") for side in [-1, 1]],
        *[R((x + side * 0.27, 1.19, z), (x + side * 0.34, 0.78, z + 0.08), 0.085, coat) for side in [-1, 1]],
        *[S((x + side * 0.34, 0.72, z + 0.09), (0.07, 0.10, 0.06), "ivory") for side in [-1, 1]],
        B((x, 0.94, z + 0.18), (0.018, 0.62, 0.015), "dark"), B((x - 0.14, 1.12, z + 0.18), (0.11, 0.15, 0.025), "teal"),
        *[B((x + side * 0.14, 0.82, z + 0.18), (0.16, 0.10, 0.025), coat) for side in [-1, 1]]]


def research(name, m):
    B, C, E, L, R, S, T = m.B, m.C, m.E, m.L, m.R, m.S, m.T
    if name == "laboratory":
        result = [B((x, 0.62, z), (0.022, 1.12, 0.022), "steel") for x in [-1.30, 1.30] for z in [-0.82, 0.82]] + [
            B((0, y, 0.852), (2.5, 0.024, 0.018), "steel") for y in [0.3, 0.5, 0.9]] + [
            B((x, 0.70, 0.86), (0.018, 0.32, 0.025), "steel") for x in [-1.07, -0.69, 0.69, 1.07]] + [
            B((x, 0.7, -0.835), (0.38, 0.48, 0.035), "water") for x in [-0.9, -0.3, 0.3, 0.9]] + [
            B((0, 1.30, -0.40), (0.65, 0.32, 0.40), "steel"),
            *[B((0, 1.48, -0.55 + i * 0.055), (0.52, 0.025, 0.02), "dark") for i in range(6)],
            *[R((x, 0.3, 0.99), (x, 0.65, 0.99), 0.018, "steel") for x in [-0.07, 0.07]],
            B((0, 0.14, 1.10), (0.92, 0.06, 0.44)), B((0, 0.02, 1.25), (1.18, 0.04, 0.72))]
    elif name == "doe-national-laboratory":
        result = [B((0, y, 1.22), (0.54, 0.025, 0.025), "steel") for y in [0.35, 0.65, 0.95, 1.25, 1.5]] + [
            B((x, 0.70, 1.20), (0.02, 0.35, 0.02), "steel") for x in [-1.29, -0.91, 0.91, 1.29]] + [
            B((x, 1.16, 0.55), (0.40, 0.13, 0.50), "steel") for x in [-1.1, 1.1]] + [
            R((1.125 * math.cos(a), 0.25, -0.5 + 1.125 * math.sin(a)), (1.125 * math.cos(a), 1.20, -0.5 + 1.125 * math.sin(a)), 0.015, "dark") for a in [i * math.pi / 20 for i in range(40)]] + [
            B((0, 0.10, 1.55), (1.25, 0.06, 0.40)), B((0, 0.16, 1.44), (1.10, 0.06, 0.30)),
            R((-1.75, 0, 1.1), (-1.75, 1.50, 1.1), 0.018, "steel"), B((-1.57, 1.30, 1.1), (0.35, 0.22, 0.025), "teal")]
    elif name == "user-facility":
        result = [p for a in [i * math.pi / 12 for i in range(24)] for p in [
            B((1.35 * math.cos(a), 0.65, 1.35 * math.sin(a)), (0.14, 0.48, 0.14), "teal", (0, -a, 0)),
            R((0.40 * math.cos(a), 0.72, 0.40 * math.sin(a)), (1.1 * math.cos(a), 0.72, 1.1 * math.sin(a)), 0.016, "gold")]] + [
            C((0, 1.12, 0), 0.22, 0.12, "steel"), T((0, 0.87, 0), 1.35, 0.025, "steel"), T((0, 0.42, 0), 1.35, 0.025, "steel")]
    elif name == "dataset":
        result = [T((0, y, 0), 0.92, 0.012, "steel") for y in [0.10, 0.42, 0.70, 1.02, 1.30, 1.62]] + [
            p for y in [0.25, 0.85, 1.45] for p in [B((0, y, 0.95), (0.50, 0.15, 0.028), "dark"),
            *[B((x, y, 0.97), (0.055, 0.06, 0.015), "teal") for x in [-0.15, -0.05, 0.05]], B((0.16, y, 0.97), (0.055, 0.06, 0.015), "amber")]]
    elif name == "publication":
        result = [B((x, y, 0.83), (1.1, 0.012, 0.012), "steel", (0, 0, -0.16 if x < 0 else 0.16)) for x in [-0.62, 0.62] for y in [0.28, 0.32, 0.36, 0.40]] + [
            B((x, 0.49, z), (0.75, 0.012, 0.018), "steel", (0, 0, -0.16 if x < 0 else 0.16)) for x in [-0.65, 0.65] for z in [-0.64, -0.53, -0.33, -0.23, -0.02, 0.08, 0.28, 0.38, 0.58, 0.68]] + [
            B((0.14, 0.48, 0.78), (0.12, 0.025, 0.50), "teal", (0.10, 0, 0))]
    elif name == "software":
        result = [B((0, 1.93, 0.18), (2.45, 0.18, 0.05), "dark"),
            *[S((x, 1.94, 0.22), (0.045, 0.045, 0.02), color) for x, color in [(-1.05, "red"), (-0.87, "amber"), (-0.69, "teal")]],
            B((0, 0.13, 0.18), (2.4, 0.045, 0.04), "steel"),
            *[B((0.9, 0.38 + i * 0.1, 0.19), (0.28, 0.02, 0.025), "steel") for i in range(3)],
            *[C((x, 0.04, 0), 0.14, 0.08, "dark") for x in [-0.88, 0.88]]]
    elif name == "funding":
        result = [T((0.75, 0.15 + i * 0.19, 0.25), 0.40, 0.012, "steel") for i in range(4)] + [
            R((0.75 + 0.42 * math.cos(a), 0.03 + i * 0.19, 0.25 + 0.42 * math.sin(a)), (0.75 + 0.42 * math.cos(a), 0.14 + i * 0.19, 0.25 + 0.42 * math.sin(a)), 0.005, "gold") for i in range(4) for a in [j * math.pi / 16 for j in range(32)]] + [
            B((-0.35, 1.06, -0.13), (0.85, 0.02, 0.02), "steel"), B((-0.52, 0.20, -0.13), (0.45, 0.035, 0.02), "dark", (0, 0, -0.08))]
    elif name == "field-site":
        result = [B((0.55, y, 0.56), (1.27, 0.012, 0.02), "steel") for y in [0.15, 0.30, 0.45, 0.6, 0.75, 0.90]] + [
            B((0.55, 1.18, -0.34), (0.85, 0.04, 0.42), "water", (-0.50, 0, 0)),
            *[B((x, 1.20, -0.34), (0.015, 0.045, 0.42), "steel", (-0.50, 0, 0)) for x in [0.22, 0.44, 0.66, 0.88]],
            R((-1, 1.7, 0), (-1.6, 0.05, 0.45), 0.012, "steel"), R((-1, 1.7, 0), (-0.55, 0.05, -0.6), 0.012, "steel"),
            B((-1.0, 0.65, 0), (0.24, 0.32, 0.18), "dark"), R((-1, 0.45, 0), (0, 0.45, 0), 0.018, "dark")]
    else:
        result = []
    return result


def refine(name, parts, m, category):
    if name == "research-team":
        refined = [p for x, z, coat in [(-0.65, 0.10, "ivory"), (0, -0.28, "teal"), (0.65, 0.10, "amber")] for p in person(m, x, z, coat)]
    else:
        additions = {"research": research, "landmarks": landmarks, "wonders": wonders, "risk": risk, "drone": drone}[category](name, m)
        refined = parts + additions
    return refined


def landmarks(name, m):
    B, C, E, L, R, S, T = m.B, m.C, m.E, m.L, m.R, m.S, m.T
    if name == "statue-of-liberty":
        result = [S((0, 2.63, 0.188), (0.040, 0.075, 0.038), "copper"),
            *[S((x, 2.70, 0.168), (0.043, 0.018, 0.018), "copper") for x in [-0.08, 0.08]],
            R((-0.055, 2.53, 0.175), (0.055, 2.53, 0.175), 0.012, "copper"),
            *[S((x, 2.62, 0), (0.045, 0.08, 0.06), "copper") for x in [-0.21, 0.21]],
            *[R((0.17 * math.cos(a), 2.70, -0.14), (0.19 * math.cos(a), 2.34, -0.17), 0.022, "copper") for a in [i * math.pi / 6 for i in range(7)]],
            *[R((x, 0.82, -0.28), (x * 0.6, 1.95, -0.16), 0.022, "copper") for x in [-0.30, -0.22, -0.1, 0.04, 0.15, 0.27]],
            *[R((-0.28, 1.35 + i * 0.13, 0.26), (0.21, 1.1 + i * 0.13, 0.28), 0.022, "copper") for i in range(6)],
            *[R((-0.60 + i * 0.025, 3.03, 0.062), (-0.60 + i * 0.025, 3.18, 0.062), 0.013, "copper") for i in range(4)],
            *[B((0, y, 0), (size, 0.04, size * 0.95), "ivory") for y, size in [(0.12, 1.04), (0.34, 0.9), (0.40, 0.74), (0.72, 0.77)]],
            *[B((x, 0.54, 0.334), (0.07, 0.18, 0.018), "dark") for x in [-0.24, -0.12, 0.12, 0.24]],
            *[B((0.40, 1.88 - i * 0.06, 0.37), (0.16, 0.008, 0.008), "copper", (0.12, 0, -0.18)) for i in range(4)]]
    elif name == "golden-gate-bridge":
        result = [B((x, 0.88, 0), (0.16, 0.008, 0.018), "white") for x in [-2.8 + i * 0.28 for i in range(21)]] + [
            R((-3.05, 0.97, z), (3.05, 0.97, z), 0.016, "red") for z in [-0.37, 0.37]] + [
            R((x, 0.83, z), (x, 0.97, z), 0.008, "red") for x in [-3 + i * 0.15 for i in range(41)] for z in [-0.37, 0.37]] + [
            R((x - 0.22, 0.69, z), (x + 0.22, 0.48, z), 0.022, "red") for x in [-2.6 + i * 0.43 for i in range(13)] for z in [-0.32, 0.32]] + [
            R((x, y, -0.32), (x, y + 0.65, 0.32), 0.026, "red") for x in [-1.7, 1.7] for y in [1.23, 2.23]] + [
            B((x, 0.92, z), (0.13, 0.07, 0.075), "ivory") for x, z in [(-2, -0.12), (-0.7, 0.12), (0.4, -0.12), (2.4, 0.12)]]
    elif name == "us-capitol":
        result = [p for a in [i * math.pi / 12 for i in range(24)] for p in [
            R((0.60 * math.cos(a), 1.44, 0.60 * math.sin(a)), (0.22 * math.cos(a), 2.09, 0.22 * math.sin(a)), 0.017, "white"),
            R((0.612 * math.cos(a), 1.18, 0.612 * math.sin(a)), (0.612 * math.cos(a), 1.38, 0.612 * math.sin(a)), 0.022, "white")]] + [
            B((x, 0.65, z), (0.045, 0.80, 0.045), "white") for x in [-2.25, -2.0, -1.7, -1.4, -1.1, 1.1, 1.4, 1.7, 2.0, 2.25] for z in [-0.65, 0.62]] + [
            B((x, 0.85, -0.67), (0.12, 0.26, 0.025), "dark") for x in [-2.1, -1.8, -1.5, -1.2, 1.2, 1.5, 1.8, 2.1]] + [
            T((0, 1.40, 0), 0.63, 0.025, "white"), T((0, 1.16, 0), 0.62, 0.035, "white"),
            L((0, 2.50, 0), [(0, 0.055), (0.14, 0.025)], "dark", 12), S((0, 2.67, 0), (0.035, 0.04, 0.035), "dark")]
    elif name == "gateway-arch":
        result = [B((x, 0.025, 0), (0.38, 0.05, 0.4), "ivory") for x in [-1.28, 1.28]] + [
            R((x, 3.2 * (1 - (x / 1.4) ** 2), 0.105), (x * 0.825, 2.95 * (1 - (x / 1.4) ** 2), 0.105), 0.004, "dark") for x in [-1.35 + i * 0.09 for i in range(31)]] + [
            B((x, 0.006, z), (0.36, 0.012, 0.3), "steel") for x in [-0.8, -0.4, 0, 0.4, 0.8] for z in [-0.6, 0.6]]
    elif name == "grand-canyon":
        result = [L((x, y, z), [(0, 0.13), (0.12, 0.15), (0.20, 0.09), (0.34, 0.08)], "terracotta", 7, (1, 1, 0.7)) for x, y, z in [(-1.35, 1.08, -0.8), (1.55, 1.07, 0.8), (-1.7, 1.05, 0.6), (1.22, 0.9, -1.0)]] + [
            B((x, y, z), (0.20, 0.016, 0.08), "rose") for x, y, z in [(-0.62, 0.48, -0.9), (0.65, 0.48, 0.65), (-0.9, 0.72, 0.9)]]
    elif name == "yellowstone":
        result = [m.part(m.G.torus(r, 0.018), "water", (0, 0.30, 0), scale=(1, 0.12, 0.8)) for r in [0.17, 0.28, 0.40, 0.48]] + [
            m.part(m.G.torus(r, 0.025), "amber", (0, 0.255, 0), scale=(1, 0.22, 0.8)) for r in [0.82, 0.91]] + [
            S((1.05 + 0.1 * math.cos(i * 1.9), 0.85 + i * 0.045, -0.64 + 0.12 * math.sin(i * 1.9)), (0.08, 0.18, 0.08), "white") for i in range(18)] + [
            S((1.3 * math.cos(a), 0.12, math.sin(a)), (0.08, 0.07, 0.11), "sand", False) for a in [i * 0.7 for i in range(9)]]
    elif name == "kennedy-space-center":
        result = [C((-0.35, y, 0.15), 0.215, 0.025, "steel") for y in [0.40, 0.75, 1.0, 1.90, 2.2]] + [
            m.part(m.G.extrude([(0, 0), (0.30, 0), (0, 0.48)], 0.04), "white", (-0.35, 0.2, 0.15), rotation=(0, a, 0)) for a in [0, math.pi / 2, math.pi, 3 * math.pi / 2]] + [
            R((x, 0.20, 0.27), (x, 2.66, 0.27), 0.012, "white") for x in [0.65, 0.85]] + [
            R((0.65, y, 0.27), (0.85, y, 0.27), 0.009, "white") for y in [0.25 + i * 0.10 for i in range(24)]] + [
            R((0.5, y, -0.30), (1.0, y + 0.60, -0.30), 0.025, "steel") for y in [0.3, 0.9, 1.5, 2.1]] + [
            R((-0.35, 0.18, 0.15), (-0.35, 0.04, 0.15), 0.12, "dark", 0.18)]
    elif name == "hoover-dam":
        result = [p for x in [-0.75, 0.75] for p in [C((x, 0.98, -0.27), 0.11, 1.5), C((x, 1.76, -0.27), 0.15, 0.10),
            L((x, 1.81, -0.27), [(0, 0.15), (0.2, 0.07)], "steel")]] + [
            B((0, 0.30, 1.0), (1.5, 0.28, 0.30)), *[B((x, 0.34, 1.16), (0.10, 0.10, 0.025), "dark") for x in [-0.6, -0.4, -0.2, 0, 0.2, 0.4, 0.6]],
            *[R((1.60 * math.cos(a), 1.50, 1.60 * math.sin(a) - 0.7), (1.60 * math.cos(a), 1.62, 1.60 * math.sin(a) - 0.7), 0.008, "steel") for a in [0.22 * math.pi + i * 0.056 * math.pi for i in range(11)]]]
    else:
        result = []
    return result


def wonders(name, m):
    B, C, E, L, R, S, T = m.B, m.C, m.E, m.L, m.R, m.S, m.T
    if name == "chichen-itza":
        result = [p for i in range(9) for p in [
            B((0, 0.22 + i * 0.20, 0), (3.22 - i * 0.25, 0.025, 3.22 - i * 0.25), "ivory"),
            *[B((x, 0.12 + i * 0.20, (3.2 - i * 0.25) / 2 + 0.008), (0.012, 0.16, 0.012), "steel") for x in [-1.3 + j * 0.26 for j in range(11)] if abs(x) < (3.2 - i * 0.25) / 2]]]
        result += [m.part(m.G.transform((p['vertices'], p['triangles']), rotation=(0, a, 0)), p['material']) for a in [math.pi / 2, math.pi, 3 * math.pi / 2] for p in [B((0, 0.045 + i * 0.078, 1.65 - i * 0.049), (0.50, 0.09, 0.10), "ivory") for i in range(23)]]
        result += [R((x, 0.12, 1.68), (x, 1.87, 0.55), 0.065, "sand") for x in [-0.33, 0.33]] + [
            B((x, 2.04, 0.44), (0.08, 0.48, 0.08), "ivory") for x in [-0.34, 0.34]]
    elif name == "christ-the-redeemer":
        result = [S((0, 2.78, 0.18), (0.033, 0.06, 0.03)), R((-0.04, 2.69, 0.185), (0.04, 2.69, 0.185), 0.008, "steel"),
            *[S((x, 2.84, 0.16), (0.028, 0.012, 0.015), "steel") for x in [-0.065, 0.065]],
            *[R((x, 0.85, -0.19), (x * 0.4, 2.2, -0.19), 0.014, "pages") for x in [-0.29, -0.20, -0.1, 0.05, 0.15, 0.25]],
            *[R((side * 0.35, 2.25 + y, 0.14), (side * 1.35, 2.30 + y, 0.14), 0.012, "pages") for side in [-1, 1] for y in [-0.06, 0, 0.06]],
            *[R((side * 1.45, 2.34, z), (side * 1.65, 2.34, z), 0.014) for side in [-1, 1] for z in [-0.06, -0.02, 0.02, 0.06]],
            *[B((0, 0.10 + i * 0.05, 0.74 - i * 0.09), (0.65, 0.06, 0.20), "steel") for i in range(10)]]
    elif name == "colosseum":
        result = [m.part(m.G.arch(r, 0.10, 0.09, 0, 2 * math.pi - 0.001, 72), "sand", (0, y, 0), scale=(1, 0.74, 1), rotation=(math.pi / 2, 0, 0)) for r, y in [(1.05, 0.22), (1.15, 0.30), (1.25, 0.38), (1.35, 0.46)]] + [
            p for i in range(24) for a in [i * 2 * math.pi / 24] for p in [
            R((1.73 * math.sin(a), 0.15, 1.28 * math.cos(a)), (1.73 * math.sin(a), 1.02, 1.28 * math.cos(a)), 0.025, "ivory"),
            B((1.73 * math.sin(a), 0.61, 1.28 * math.cos(a)), (0.16, 0.06, 0.24), "ivory", (0, a, 0))]] + [
            B((x, 0.14, z), (0.04, 0.10, 0.54), "sand") for x in [-0.65, -0.35, 0, 0.35, 0.65] for z in [-0.25, 0.35]]
    elif name == "great-pyramid-of-giza":
        result = [p for x, z, width, height in [(-0.55, -0.1, 2.2, 1.7), (1.05, -0.35, 1.15, 0.9), (0.95, 0.85, 0.63, 0.48)] for i in range(1, 22) for y in [i * height / 23] for w in [width * (1 - y / height)] for p in [
            B((x, 0.16 + y, z + w / 2), (w, 0.009, 0.012), "steel"), B((x, 0.16 + y, z - w / 2), (w, 0.009, 0.012), "steel"),
            B((x + w / 2, 0.16 + y, z), (0.012, 0.009, w), "steel"), B((x - w / 2, 0.16 + y, z), (0.012, 0.009, w), "steel")]]
        result += [B((-0.55, 0.40, 0.85), (0.15, 0.14, 0.025), "dark", (-0.57, 0, 0))]
    elif name == "great-wall-of-china":
        result = []
        for i in range(16):
            x = -1.875 + i * 0.25
            y, z = 0.9 + 0.18 * math.cos(x * 1.7), 0.48 * math.sin(x * 1.6)
            angle = -math.atan(0.48 * 1.6 * math.cos(x * 1.6))
            result += [B((x, y + 0.34, z), (0.28, 0.03, 0.23), "steel", (0, angle, 0))]
            result += [B((x, y + dy, z + 0.195), (0.25, 0.012, 0.015), "steel", (0, angle, 0)) for dy in [-0.15, 0.03, 0.20]]
        result += [B((x, 1.47 + 0.18 * math.cos(x * 1.7), 0.48 * math.sin(x * 1.6) + 0.301), (0.16, 0.28, 0.02), "dark") for x in [-1.75, 0, 1.75]]
    elif name == "machu-picchu":
        result = [B((x, 0.16 + i * 0.19, 1.25 - i * 0.18), (0.015, 0.14, 0.018), "steel") for i in range(5) for x in [-1.35 + j * 0.20 for j in range(14)] if abs(x) < 1.5 - i * 0.15] + [
            B((0.95, 0.04 + i * 0.058, 1.1 - i * 0.06), (0.25, 0.065, 0.10), "sand") for i in range(14)] + [
            B((x, y + dy, z + 0.18), (0.44, 0.012, 0.018), "steel") for x, z, y in [(-0.8, 0.5, 0.85), (-0.1, 0.5, 1), (0.65, 0.5, 0.85), (-0.4, -0.05, 1)] for dy in [0.08, 0.19, 0.28]]
    elif name == "petra":
        result = [p for x in [-0.95, -0.60, 0.60, 0.95] for p in [C((x, 0.20, 0.13), 0.14, 0.09, "ivory", 16), C((x, 1.15, 0.13), 0.14, 0.09, "ivory", 16),
            *[R((x + 0.095 * math.cos(a), 0.27, 0.13 + 0.095 * math.sin(a)), (x + 0.095 * math.cos(a), 1.09, 0.13 + 0.095 * math.sin(a)), 0.009, "ivory") for a in [i * math.pi / 6 for i in range(12)]]]] + [
            B((x, 1.98, -0.025), (0.23, 0.47, 0.03), "dark") for x in [-0.77, 0.77]] + [
            B((0, y, 0.32), (2.25, 0.025, 0.035), "ivory") for y in [1.17, 1.28, 2.34, 2.41]] + [
            S((0, 2.79, 0.12), (0.09, 0.11, 0.09), "sand")]
    elif name == "taj-mahal":
        result = [p for side in [-1, 1] for x in [-0.62, 0, 0.62] for p in [
            E([(-0.12, 0), (0.12, 0), (0.12, 0.30), (0, 0.43), (-0.12, 0.30)], 0.018, "water", (x, 0.46, side * 0.684)),
            *[R((x + dx, 0.43, side * 0.70), (x + dx, 0.90, side * 0.70), 0.013, "ivory") for dx in [-0.15, 0.15]]]] + [
            p for x in [-1.37, 1.37] for z in [-1.03, 1.03] for p in [
            *[T((x, y, z), 0.13, 0.012, "steel") for y in [0.55, 0.90, 1.25]], C((x, 1.77, z), 0.03, 0.30, "gold", 12)]] + [
            C((0, 2.35, 0), 0.025, 0.28, "gold", 12), *[B((x, 1.23, 0), (0.02, 0.10, 1.35), "ivory") for x in [-0.88, 0.88]],
            *[C((x, 0.30, 1.0), 0.045, 0.12, "leaf", 8) for x in [-0.65, -0.45, 0.45, 0.65]]]
    else:
        result = []
    return result


def drone(name, m):
    B, C, R, S, T = m.B, m.C, m.R, m.S, m.T
    specs = {
        "autel-evo-max-4t": (0.94, 0.72, 0.60, 0.06, 0.60, 1.08),
        "bitcraze-crazyflie-2-1": (0.75, 0.75, 0.16, 0, 0.55, 0.65),
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
            T((x, h - 0.05, z), 0.105, 0.009, "steel"), T((x, h + 0.06, z), 0.105, 0.009, "steel"),
            *[R((x + 0.105 * math.cos(a), h - 0.04, z + 0.105 * math.sin(a)), (x + 0.105 * math.cos(a), h + 0.05, z + 0.105 * math.sin(a)), 0.006, "steel") for a in [i * math.pi / 6 for i in range(12)]],
            C((x, h + 0.21, z), 0.028, 0.022, "dark", 6),
            R((x * 0.20, y + 0.045, z * 0.25), (x * 0.88, h + 0.045, z * 0.88), 0.010, "teal"),
            C((x * 0.23, y + 0.06, z * 0.25), 0.025, 0.025, "steel", 8)]]
        result += [B((side * (width / 2 + 0.012), y + 0.03, z), (0.015, 0.06, 0.02), "dark") for side in [-1, 1] for z in [-length * 0.30 + i * length * 0.06 for i in range(10)]]
        result += [B((side * width * 0.43, y + 0.16, -length * 0.10), (0.015, 0.025, length * 0.60), "steel") for side in [-1, 1]]
        result += [C((x, y + 0.19, z), 0.018, 0.018, "steel", 8) for x in [-width * 0.38, width * 0.38] for z in [-length * 0.32, length * 0.32]]
        if name == "bitcraze-crazyflie-2-1":
            result += [B((x, 0.235, z), (0.035, 0.012, 0.05), "gold") for x in [-0.23, 0.23] for z in [-0.20, -0.10, 0, 0.10, 0.20]]
        elif name in ["holybro-x500-v2", "freefly-astro", "ornl-strelka"]:
            result += [B((0, y + 0.28, z), (width * 0.65, 0.025, 0.08), "dark") for z in [-0.20, 0.20]]
            result += [R((width / 2, y, -0.15), (width / 2 + 0.09, y - 0.28, 0.10), 0.018, "amber"), R((width / 2 + 0.09, y - 0.28, 0.10), (0, y - 0.32, 0.15), 0.018, "amber")]
        else:
            result += [B((0, y + 0.23, -0.18), (width * 0.3, 0.012, 0.15), "steel"),
                       *[S((x, y + 0.08, length / 2 + 0.02), (0.028, 0.028, 0.015), "water") for x in [-width * 0.30, width * 0.30]]]
    else:
        wing_y = 0.35 if name == "ebee-x" else 0.75
        result = [R((side * 0.45, wing_y + 0.065, -0.20), (side * 1.83, wing_y + 0.065, -0.38), 0.009, "steel") for side in [-1, 1]] + [
            R((side * 1.1, wing_y + 0.065, -0.30), (side * 1.1, wing_y + 0.065, 0.20), 0.008, "steel") for side in [-1, 1]] + [
            B((0, wing_y + 0.205, 0.1), (0.19, 0.015, 0.43), "teal"), *[C((x, wing_y + 0.21, z), 0.016, 0.015, "steel", 8) for x in [-0.1, 0.1] for z in [-0.10, 0.28]],
            R((0, wing_y + 0.15, -0.32), (0, wing_y + 0.36, -0.36), 0.012, "dark")]
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
    elif name == "cavalry":
        result = [S((0.91, 1.61, 0), (0.20, 0.13, 0.16), "ivory"),
            *[S((0.79, 1.73, z), (0.035, 0.035, 0.018), "dark") for z in [-0.17, 0.17]],
            *[S((1.03, 1.64, z), (0.025, 0.018, 0.015), "dark") for z in [-0.12, 0.12]],
            *[R((0.90, 1.55, z), (0.52, 1.46, z), 0.018, "dark") for z in [-0.17, 0.17]],
            *[R((0.52, 1.46, z), (0.30, 1.65, z), 0.012, "dark") for z in [-0.17, 0.17]],
            *[R((0.5, 1.63 - i * 0.06, -0.02), (0.36 - i * 0.025, 1.52 - i * 0.06, -0.02), 0.035, "dark") for i in range(8)],
            B((0, 1.32, 0), (0.48, 0.10, 0.7), "dark"), *[R((x, 1.22, -0.30), (x, 0.72, -0.25), 0.02, "dark") for x in [-0.22, 0.22]],
            *[S((x, 0.55, z), (0.085, 0.11, 0.085), "ivory") for x in [-0.40, 0.40] for z in [-0.22, 0.22]],
            S((0.19, 2.18, 0), (0.035, 0.045, 0.035), "ivory"), *[S((0.16, 2.23, z), (0.015, 0.012, 0.016), "dark") for z in [-0.07, 0.07]],
            B((0, 1.60, 0), (0.44, 0.05, 0.40), "dark")]
    elif name == "artillery":
        result = [m.part(m.G.torus(0.23 - z * 0.03, 0.018), "steel", (0, 0.85 + z * 0.12, z - 0.4), rotation=(math.pi / 2 - 0.12, 0, 0)) for z in [0.12, 0.40, 0.9, 1.35]] + [
            R((-0.80, 0.52, 0), (0.80, 0.52, 0), 0.06, "steel"),
            *[R((x - 0.02, 0.52, 0), (x + 0.02, 0.52, 0), 0.14, "gold") for x in [-0.78, 0.78]],
            R((0, 0.80, -0.4), (0, 0.80, -0.58), 0.09, "steel"),
            *[C((x, 0.64, z), 0.025, 0.025, "steel", 6) for x in [-0.34, 0.34] for z in [-0.30, 0.03]]]
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
