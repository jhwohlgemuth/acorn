"""ORNL colors shared with acorn-lib/src/util/constants/app.rs, in glTF linear RGB."""
ACCENTS = {"Tech cyan": "#00E5FF"}
COLORS = {
    "Dark Matter": "#373A36", "Energy": "#7DBA00", "Forge": "#FF9E1B", "Graphite": "#DBDCDB",
    "Hale Navy": "#00454D", "Hydro": "#005776", "Infinity": "#006BA6", "Mist": "#8BFEBF",
    "ORNL Green": "#00662C", "Plasma": "#B50094", "Polar": "#FFFFFF", "Pulsar": "#4E008E", "Spark": "#FE5000",
}
ROLES = {
    "amber": "Forge", "bezel": "Dark Matter", "blue": "Infinity", "cabinet": "Graphite", "copper": "ORNL Green",
    "dark": "Dark Matter", "glass": "Hydro", "gold": "Forge", "grass": "ORNL Green", "ivory": "Graphite", "leaf": "Energy",
    "light": "Mist", "magenta": "Plasma", "orange": "Spark", "pages": "Polar", "pedestal": "Hale Navy", "purple": "Pulsar",
    "red": "Spark", "rose": "Spark", "sand": "Graphite", "steel": "Graphite", "teal": "ORNL Green",
    "terracotta": "Forge", "water": "Hydro", "white": "Polar", "yellow": "Forge",
}

NATURAL = {
    "amber": ("Warm ochre", "#9F794B"), "gold": ("Antique brass", "#9B804A"),
    "orange": ("Burnt umber", "#9A563A"), "red": ("Oxide red", "#A5543B"),
    "rose": ("Weathered sandstone", "#A9755B"), "sand": ("Limestone", "#C2AF90"),
    "terracotta": ("Canyon clay", "#95674C"), "yellow": ("Warm ochre", "#9F794B"),
}


def linear(hex_color):
    srgb = [int(hex_color[i:i + 2], 16) / 255 for i in [1, 3, 5]]
    return [round(value / 12.92 if value <= 0.04045 else ((value + 0.055) / 1.055) ** 2.4, 8) for value in srgb]


def material(role, metallic=None, roughness=None):
    name = ROLES[role]
    finish = {"steel": (0.80, 0.32), "gold": (0.70, 0.30), "dark": (0.05, 0.45), "bezel": (0.05, 0.45),
              "cabinet": (0.25, 0.40), "water": (0.15, 0.18), "teal": (0.18, 0.40), "pedestal": (0.28, 0.35)}.get(role, (0.0, 0.78))
    metallic = finish[0] if metallic is None else metallic
    roughness = finish[1] if roughness is None else roughness
    color_name, hex_color = NATURAL.get(role, (name, COLORS[name]))
    return {"extras": {"brand": "natural finish" if role in NATURAL else "ORNL", "colorName": color_name, "srgbHex": hex_color},
            "pbrMetallicRoughness": {"baseColorFactor": [*linear(hex_color), 1], "metallicFactor": metallic, "roughnessFactor": roughness}}
